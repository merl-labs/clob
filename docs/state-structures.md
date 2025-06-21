# Manifest State Structures

## Overview

Manifest uses a sophisticated state management system based on the **DynamicAccount** pattern, where accounts have a fixed header followed by dynamic data organized in hypertree structures. All dynamic data uses uniform 80-byte (market) or 64-byte (global) nodes that can represent different data types.

## Core Design Pattern

### DynamicAccount<Fixed, Dynamic>
The fundamental pattern for all Manifest accounts:

```rust
pub struct DynamicAccount<Fixed, Dynamic> {
    pub fixed: Fixed,    // Fixed-size header with metadata
    pub dynamic: Dynamic, // Variable-size data section
}
```

**Type Aliases:**
- `MarketValue = DynamicAccount<MarketFixed, Vec<u8>>` (owned)
- `MarketRef<'a> = DynamicAccount<&'a MarketFixed, &'a [u8]>` (read-only)
- `MarketRefMut<'a> = DynamicAccount<&'a mut MarketFixed, &'a mut [u8]>` (mutable)

## Market State Structures

### MarketFixed (256 bytes)
The fixed header for market accounts.

```rust
pub struct MarketFixed {
    // Account identification
    pub discriminant: u64,           // 4859840929024028656
    version: u8,                     // Version number
    
    // Token configuration
    base_mint_decimals: u8,
    quote_mint_decimals: u8,
    base_vault_bump: u8,
    quote_vault_bump: u8,
    _padding1: [u8; 3],
    
    // Token addresses
    base_mint: Pubkey,               // Base token mint
    quote_mint: Pubkey,              // Quote token mint
    base_vault: Pubkey,              // Base token vault PDA
    quote_vault: Pubkey,             // Quote token vault PDA
    
    // Market state
    order_sequence_number: u64,      // Next order sequence number
    num_bytes_allocated: u32,        // Dynamic section size
    
    // Red-black tree indices
    bids_root_index: DataIndex,      // Root of bids tree
    bids_best_index: DataIndex,      // Best bid (highest price)
    asks_root_index: DataIndex,      // Root of asks tree  
    asks_best_index: DataIndex,      // Best ask (lowest price)
    claimed_seats_root_index: DataIndex, // Root of seats tree
    free_list_head_index: DataIndex, // Head of free memory list
    
    // Statistics
    quote_volume: QuoteAtoms,        // Lifetime trading volume
    
    // Padding for future use
    _padding2: [u32; 1],
    _padding3: [u64; 8],
}
```

**Key Methods:**
- `get_base_mint()`, `get_quote_mint()`: Token addresses
- `get_base_vault()`, `get_quote_vault()`: Vault addresses
- `has_free_block()`: Check if expansion is needed

### RestingOrder (64 bytes payload)
Represents limit orders on the orderbook.

```rust
pub struct RestingOrder {
    price: QuoteAtomsPerBaseAtom,    // Order price
    num_base_atoms: BaseAtoms,       // Order quantity
    sequence_number: u64,            // Unique order ID
    trader_index: DataIndex,         // Index to trader's seat
    last_valid_slot: u32,           // Expiration slot (0 = no expiry)
    is_bid: PodBool,                // true = buy, false = sell
    order_type: OrderType,          // Order behavior type
    reverse_spread: u16,            // Spread for reverse orders
    _padding: [u8; 20],
}
```

**Order Types:**
```rust
pub enum OrderType {
    Limit = 0,           // Standard limit order
    ImmediateOrCancel = 1, // Take-only, no resting
    PostOnly = 2,        // Maker-only, fails if crosses
    Global = 3,          // Uses global account funds
    Reverse = 4,         // Auto-flips sides when filled
}
```

**Key Methods:**
- `new()`: Create new order
- `get_trader_index()`: Get seat index
- `get_price()`, `get_num_base_atoms()`: Order details
- `is_expired()`: Check if order expired

### ClaimedSeat (64 bytes payload)
Represents a trader's position on a market.

```rust
pub struct ClaimedSeat {
    pub trader: Pubkey,              // Trader's public key
    pub base_withdrawable_balance: BaseAtoms,  // Available base tokens
    pub quote_withdrawable_balance: QuoteAtoms, // Available quote tokens
    pub quote_volume: QuoteAtoms,    // Lifetime trading volume
    _padding: [u8; 8],
}
```

**Key Features:**
- Balances exclude funds locked in open orders
- Volume tracking for monitoring (not security-critical)
- Sorted by trader public key in red-black tree

## Global State Structures

### GlobalFixed (96 bytes)
The fixed header for global accounts.

```rust
pub struct GlobalFixed {
    pub discriminant: u64,           // 10787423733276977665
    mint: Pubkey,                    // Token mint for this global
    vault: Pubkey,                   // Global vault address
    
    // Tree indices
    global_traders_root_index: DataIndex,   // Traders tree root
    global_deposits_root_index: DataIndex,  // Deposits tree root
    global_deposits_max_index: DataIndex,   // Min deposit (for eviction)
    free_list_head_index: DataIndex,        // Free memory list
    
    // State tracking
    num_bytes_allocated: DataIndex,  // Dynamic section size
    vault_bump: u8,                  // Vault PDA bump
    global_bump: u8,                 // Global PDA bump
    num_seats_claimed: u16,          // Number of active traders
}
```

**Constants:**
- `MAX_GLOBAL_SEATS`: Maximum traders per global account
- `GLOBAL_BLOCK_SIZE`: 64 bytes per node

### GlobalTrader (48 bytes payload)
Represents a trader in a global account.

```rust
pub struct GlobalTrader {
    trader: Pubkey,                  // Trader's public key
    deposit_index: DataIndex,        // Index to deposit record
    _padding: u32,
    _padding2: u64,
}
```

### GlobalDeposit (48 bytes payload)
Tracks token deposits in global accounts.

```rust
pub struct GlobalDeposit {
    trader: Pubkey,                  // Trader's public key
    balance_atoms: GlobalAtoms,      // Token balance
    _padding: u64,
}
```

**Key Features:**
- Sorted by balance (reversed) for eviction mechanics
- Smallest balance is at tree max for O(1) eviction lookup

## Wrapper State Structures

### ManifestWrapperStateFixed (64 bytes)
Header for wrapper program accounts.

```rust
pub struct ManifestWrapperStateFixed {
    pub discriminant: u64,           // Wrapper discriminant
    pub trader: Pubkey,              // Owner trader
    pub num_bytes_allocated: u32,    // Dynamic section size
    pub free_list_head_index: DataIndex, // Free memory list
    pub market_infos_root_index: DataIndex, // Market info tree
    pub _padding: [u32; 3],
}
```

## Memory Layout and Organization

### Hypertree Node Structure
All dynamic data is stored in uniform-sized nodes:

```
Market Nodes (80 bytes):
┌─────────────────┬──────────────────────────────────────────────────────────────┐
│ RBTree Overhead │                    Payload (64 bytes)                        │
│   (16 bytes)    │              RestingOrder or ClaimedSeat                     │
└─────────────────┴──────────────────────────────────────────────────────────────┘

Global Nodes (64 bytes):
┌─────────────────┬──────────────────────────────────────────────────────┐
│ RBTree Overhead │              Payload (48 bytes)                      │
│   (16 bytes)    │        GlobalTrader or GlobalDeposit                 │
└─────────────────┴──────────────────────────────────────────────────────┘
```

### Account Layout
```
Market Account:
┌──────────────────────┬─────────────────────────────────────────────────────────┐
│   MarketFixed        │                Dynamic Section                          │
│   (256 bytes)        │     Interleaved: Bids | Asks | Seats | FreeList       │
└──────────────────────┴─────────────────────────────────────────────────────────┘

Global Account:
┌──────────────────────┬─────────────────────────────────────────────────────────┐
│   GlobalFixed        │                Dynamic Section                          │
│   (96 bytes)         │     Interleaved: Traders | Deposits | FreeList        │
└──────────────────────┴─────────────────────────────────────────────────────────┘
```

### Red-Black Tree Organization
Each account maintains multiple red-black trees sharing the same memory space:

**Market Trees:**
- **Bids Tree**: Buy orders sorted by price (descending)
- **Asks Tree**: Sell orders sorted by price (ascending)  
- **Claimed Seats Tree**: Trader seats sorted by public key

**Global Trees:**
- **Traders Tree**: Global traders sorted by public key
- **Deposits Tree**: Deposits sorted by balance (reversed for eviction)

### Free List Management
Unused memory blocks are managed via linked lists:
- Each free block contains a pointer to the next free block
- Enables efficient allocation/deallocation without fragmentation
- Automatic expansion when free blocks are exhausted

## Data Access Patterns

### Reading Data
```rust
// Get market reference
let market: MarketRef = DynamicAccount { fixed: &market_fixed, dynamic: &dynamic_data };

// Access orderbook
let bids: BooksideReadOnly = market.get_bids();
let asks: BooksideReadOnly = market.get_asks();

// Iterate orders
for (index, order) in bids.iter::<RestingOrder>() {
    // Process order
}
```

### Modifying Data
```rust
// Get mutable market reference  
let mut market: MarketRefMut = DynamicAccount { 
    fixed: &mut market_fixed, 
    dynamic: &mut dynamic_data 
};

// Add order to market
let result = market.place_order(/* params */);
```

## Constants and Discriminants

```rust
// Size constants
pub const MARKET_FIXED_SIZE: usize = 256;
pub const GLOBAL_FIXED_SIZE: usize = 96;
pub const MARKET_BLOCK_SIZE: usize = 80;
pub const GLOBAL_BLOCK_SIZE: usize = 64;
pub const RESTING_ORDER_SIZE: usize = 64;
pub const CLAIMED_SEAT_SIZE: usize = 64;

// Account discriminants
pub const MARKET_FIXED_DISCRIMINANT: u64 = 4859840929024028656;
pub const GLOBAL_FIXED_DISCRIMINANT: u64 = 10787423733276977665;

// Special values
pub const NO_EXPIRATION_LAST_VALID_SLOT: u32 = 0;
pub const NIL: DataIndex = DataIndex::MAX; // Null pointer equivalent
```

## Validation and Safety

### Account Validation
All accounts implement `ManifestAccount` trait:
```rust
pub trait ManifestAccount {
    fn verify_discriminant(&self) -> ProgramResult;
}
```

### Memory Safety
- All structures are `Pod` (Plain Old Data) for safe byte-level access
- Static assertions ensure correct sizes and alignment
- Bounds checking on all array accesses
- Overflow protection on arithmetic operations

## Relationships and Dependencies

1. **Market ↔ ClaimedSeat**: Markets contain trader seats for balance tracking
2. **Market ↔ RestingOrder**: Markets contain orderbook with resting orders
3. **Global ↔ GlobalTrader**: Global accounts contain trader registrations
4. **Global ↔ GlobalDeposit**: Global accounts track token deposits
5. **RestingOrder → ClaimedSeat**: Orders reference trader seats via index
6. **GlobalTrader → GlobalDeposit**: Traders reference deposits via index

This state architecture enables Manifest's high-performance orderbook operations while maintaining data integrity and supporting complex trading features like global orders and reverse orders.
