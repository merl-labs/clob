# Manifest Wrapper Programs Documentation

## Overview

Manifest uses a **core + wrapper architecture** where the core program handles only essential orderbook operations, while wrapper programs provide enhanced features through Cross-Program Invocation (CPI). This design keeps the core program simple and secure while enabling rich functionality through composable wrappers.

## Architecture Benefits

### Core Program Focus
- **Minimal attack surface**: Only essential orderbook logic
- **Gas efficiency**: Optimized for high-frequency operations
- **Formal verification**: Simpler codebase enables mathematical proofs
- **Stability**: Core functionality remains unchanged

### Wrapper Program Extensions
- **Client order IDs**: Track orders with custom identifiers
- **Enhanced order types**: Fill-or-kill, post-only, immediate-or-cancel
- **Batch operations**: Complex multi-order transactions
- **State tracking**: Maintain trader state across markets
- **User experience**: Higher-level abstractions for applications

## Wrapper Program (Standard)

**Program ID**: `wMNFSTkir3HgyZTsB7uqu3i7FA73grFCptPXgrZjksL`

### Purpose
The standard wrapper provides enhanced trading features for applications and traders who need more sophisticated order management than the core program offers.

### State Structure

#### ManifestWrapperStateFixed (64 bytes)
```rust
pub struct ManifestWrapperStateFixed {
    pub discriminant: u64,           // Account type identifier
    pub trader: Pubkey,              // Owner of this wrapper
    pub num_bytes_allocated: u32,    // Dynamic section size
    pub free_list_head_index: DataIndex, // Free memory list
    pub market_infos_root_index: DataIndex, // Market data tree
    pub _padding: [u32; 3],          // Future expansion
}
```

#### MarketInfo (80 bytes payload)
Tracks trader state for each market:
```rust
pub struct MarketInfo {
    pub market: Pubkey,              // Market address
    pub orders_root_index: DataIndex, // Open orders tree
    pub trader_index: DataIndex,     // Seat index in market
    pub base_balance: BaseAtoms,     // Available base tokens
    pub quote_balance: QuoteAtoms,   // Available quote tokens
    pub quote_volume: QuoteAtoms,    // Lifetime trading volume
    pub last_updated_slot: u32,      // Last sync slot
    pub _padding: [u32; 3],
}
```

#### WrapperOpenOrder (80 bytes payload)
Tracks individual orders with client metadata:
```rust
pub struct WrapperOpenOrder {
    price: QuoteAtomsPerBaseAtom,    // Order price
    client_order_id: u64,           // Client-assigned ID
    order_sequence_number: u64,     // On-chain order ID
    num_base_atoms: BaseAtoms,      // Order size
    market_data_index: DataIndex,   // Link to market info
    last_valid_slot: u32,           // Expiration slot
    is_bid: PodBool,                // Order side
    order_type: OrderType,          // Order behavior
    _padding: [u8; 30],
}
```

### Instructions

#### CreateWrapper (0)
Creates a new wrapper account for a trader.

**Accounts:**
- `owner` (writable, signer): Wrapper owner
- `system_program`: Solana system program
- `wrapper_state` (writable): Wrapper account to create

**Parameters:** None

#### ClaimSeat (1)
Claims a seat on a market (handled automatically in other operations).

**Accounts:**
- `owner` (signer): Wrapper owner
- `wrapper_state` (writable): Wrapper account
- `market` (writable): Market account
- `system_program`: System program
- `manifest_program`: Core Manifest program

#### Deposit (2)
Deposits tokens to a market through the wrapper.

**Accounts:**
- `owner` (signer): Token owner
- `wrapper_state` (writable): Wrapper account
- `market` (writable): Market account
- `trader_token_account` (writable): Source token account
- `vault` (writable): Market vault
- `token_program`: Token program
- `manifest_program`: Core program

**Parameters:**
```rust
pub struct DepositParams {
    pub amount_atoms: u64,
}
```

#### Withdraw (3)
Withdraws tokens from a market through the wrapper.

**Accounts:** Same as Deposit

**Parameters:**
```rust
pub struct WithdrawParams {
    pub amount_atoms: u64,
}
```

#### BatchUpdate (4)
Places and cancels multiple orders atomically with enhanced features.

**Accounts:**
- `owner` (signer): Wrapper owner
- `wrapper_state` (writable): Wrapper account
- `market` (writable): Market account
- `system_program`: System program
- `manifest_program`: Core program
- Global account fields (optional): For global orders

**Parameters:**
```rust
pub struct WrapperBatchUpdateParams {
    pub cancels: Vec<WrapperCancelOrderParams>,
    pub cancel_all: bool,
    pub orders: Vec<WrapperPlaceOrderParams>,
}

pub struct WrapperCancelOrderParams {
    pub client_order_id: u64,       // Client-assigned order ID
}

pub struct WrapperPlaceOrderParams {
    pub client_order_id: u64,       // Client-assigned ID
    pub base_atoms: u64,            // Order size
    pub price_mantissa: u32,        // Price mantissa
    pub price_exponent: i8,         // Price exponent
    pub is_bid: bool,               // Order side
    pub last_valid_slot: u32,       // Expiration
    pub order_type: OrderType,      // Order behavior
}
```

#### Collect (5)
Synchronizes wrapper state with market state and collects any changes.

**Accounts:**
- `owner` (signer): Wrapper owner
- `wrapper_state` (writable): Wrapper account
- `market`: Market account
- `manifest_program`: Core program

### Enhanced Order Types

The wrapper enables additional order behaviors beyond the core program:

#### Fill-or-Kill (FOK)
- Order must be completely filled immediately or cancelled
- No partial fills allowed
- Implemented via `ImmediateOrCancel` with size validation

#### Post-Only
- Order must not cross the spread (maker-only)
- Automatically adjusts price if needed to avoid taking
- Fails if no valid maker price exists

#### Client Order ID Management
- Each order gets a client-assigned identifier
- Enables order tracking and management by external systems
- Cancellation by client ID instead of sequence number

### Batch Operations

The wrapper's batch update enables complex trading strategies:

```rust
// Example: Replace all orders with new ones
let batch_params = WrapperBatchUpdateParams {
    cancel_all: true,                    // Cancel existing orders
    cancels: vec![],                     // No specific cancels needed
    orders: vec![
        // New bid order
        WrapperPlaceOrderParams {
            client_order_id: 1,
            base_atoms: 1_000_000,       // 1 token
            price_mantissa: 1000,        // Price components
            price_exponent: -1,          // = 100.0
            is_bid: true,
            last_valid_slot: 0,          // No expiration
            order_type: OrderType::Limit,
        },
        // New ask order
        WrapperPlaceOrderParams {
            client_order_id: 2,
            base_atoms: 1_000_000,
            price_mantissa: 1020,
            price_exponent: -1,          // = 102.0
            is_bid: false,
            last_valid_slot: 0,
            order_type: OrderType::PostOnly,
        },
    ],
};
```

## UI Wrapper Program (Specialized)

**Program ID**: `UMnFStVeG1ecZFc2gc5K3vFy3sMpotq8C91mXBQDGwh`

### Purpose
Specialized wrapper designed for user interface applications with additional convenience features and different account ownership patterns.

### Key Differences from Standard Wrapper

#### Separate Owner and Payer
```rust
// UI wrapper allows separate owner and payer
CreateWrapper {
    owner,           // Can be a PDA
    payer,          // Pays for account creation
    wrapper_state,  // Account to create
}
```

#### Simplified Instructions
- **PlaceOrder**: Single order placement with automatic setup
- **CancelOrder**: Cancel by client order ID
- **SettleFunds**: Withdraw available balances
- **CreateWrapper**: Enhanced creation with PDA support

#### State Structure
```rust
pub struct ManifestWrapperUserFixed {
    pub discriminant: u64,           // Different discriminant
    pub trader: Pubkey,              // Owner (can be PDA)
    pub num_bytes_allocated: u32,    // Dynamic section size
    pub free_list_head_index: DataIndex, // Free memory
    pub market_infos_root_index: DataIndex, // Market data
    pub _padding: [u32; 3],
}
```

### Instructions

#### CreateWrapper (0)
Creates wrapper with separate owner and payer.

#### PlaceOrder (2)
Places a single order with automatic seat claiming and market expansion.

**Features:**
- Automatic seat claiming if needed
- Market expansion if required
- Deposit handling for insufficient funds
- Error recovery and retry logic

#### CancelOrder (4)
Cancels an order by client order ID.

#### SettleFunds (5)
Withdraws all available balances to trader token accounts.

## Integration Patterns

### Direct Core Integration
For maximum efficiency:
```rust
// Direct core program calls
let batch_update_ix = batch_update_instruction(
    market,
    trader,
    BatchUpdateParams { /* core params */ }
);
```

### Standard Wrapper Integration
For enhanced features:
```rust
// Wrapper with client order IDs and enhanced types
let wrapper_batch_ix = wrapper::batch_update_instruction(
    wrapper_state,
    market,
    WrapperBatchUpdateParams { /* wrapper params */ }
);
```

### UI Wrapper Integration
For user interfaces:
```rust
// Simplified single-order interface
let place_order_ix = ui_wrapper::place_order_instruction(
    owner,
    market,
    PlaceOrderParams { /* ui params */ }
);
```

## State Synchronization

### Automatic Sync
Wrapper operations automatically sync state with the core market:
- Updates balances from market claimed seat
- Removes filled/cancelled orders
- Updates market info metadata

### Manual Sync
The `Collect` instruction can be called to sync without trading:
```rust
let collect_ix = collect_instruction(wrapper_state, market);
```

### Sync Triggers
State synchronization occurs during:
- Order placement
- Order cancellation
- Deposit/withdrawal operations
- Explicit collect calls

## Performance Considerations

### Memory Usage
- Wrapper accounts use 96-byte blocks (vs 80-byte market blocks)
- Additional overhead for client order tracking
- Separate trees for each market's orders

### Compute Units
- Wrapper operations use more CU than direct core calls
- CPI overhead for calling core program
- State synchronization adds processing time

### Account Limits
- Each wrapper can track multiple markets
- Memory expansion handled automatically
- Free list prevents fragmentation

## Error Handling

### Wrapper-Specific Errors
```rust
pub enum WrapperError {
    ClientOrderIdAlreadyExists,      // Duplicate client order ID
    ClientOrderIdNotFound,           // Cancel non-existent order
    InsufficientFundsForOrder,       // Not enough balance
    MarketNotFound,                  // Market not in wrapper
    InvalidOrderType,                // Unsupported order type
}
```

### Recovery Patterns
- Automatic retry for transient failures
- Graceful degradation for partial fills
- State cleanup for failed operations

## Best Practices

### Wrapper Selection
- **Core Program**: High-frequency trading, maximum efficiency
- **Standard Wrapper**: Applications needing client order IDs
- **UI Wrapper**: User interfaces, simplified operations

### Order Management
- Use unique client order IDs across all markets
- Implement proper error handling for order operations
- Sync wrapper state regularly for accurate data

### Resource Management
- Monitor wrapper account size and expand as needed
- Clean up filled orders to free memory
- Use batch operations for multiple changes

The wrapper architecture enables Manifest to serve both high-performance trading applications and user-friendly interfaces while maintaining the security and efficiency of the core orderbook primitive.
