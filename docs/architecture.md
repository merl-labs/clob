# Manifest Architecture Documentation

## Overview

Manifest is a next-generation limit order book exchange built on Solana that provides:
- **Feeless trading** - No trading fees forever
- **Atomic lot sizes** - Trade any amount without lot size restrictions
- **Capital efficiency** - Reuse capital across multiple markets with global orders
- **Low creation cost** - Only 0.007 SOL to create a market vs 2-3+ SOL on other platforms
- **Crankless operation** - No need for external cranks to process trades
- **Token 2022 support** - Full support for the new token standard when needed

## Core Design Principles

### 1. Infrastructure Layer Primitive
Manifest implements the orderbook as a pure infrastructure layer, focusing only on essential orderbook functionality. Complex features are handled by wrapper programs that interact with the core via Cross-Program Invocation (CPI).

### 2. Hypertree Data Structure
The innovation enabling Manifest's efficiency is the **hypertree** - a memory-efficient data structure where all market data fits into uniform 80-byte nodes. This allows multiple data structures (bids, asks, claimed seats, free list) to interleave in the same memory space.

### 3. Core vs Wrapper Architecture
- **Core Program**: Handles only essential orderbook operations (place, cancel, match orders)
- **Wrapper Programs**: Provide additional features like client order IDs, fill-or-kill orders, post-only orders, etc.

## Data Structure Architecture

### Market Account Layout
```
--------------------------------------------------------------------------------------------------------
|                   Header                    |                               Dynamic                   |
--------------------------------------------------------------------------------------------------------
| BaseMint, QuoteMint, BidsRootIndex, ...     | Bid | Ask | FreeListNode | Seat | Seat | Bid | Bid | Ask|
--------------------------------------------------------------------------------------------------------
```

### Hypertree Components
All data structures use 80-byte nodes that can represent:
- **RestingOrder**: Limit orders on the book
- **ClaimedSeat**: Trader positions and balances
- **FreeListNode**: Available memory blocks

### Red-Black Trees
The market maintains three red-black trees:
1. **Bids Tree**: Buy orders sorted by price (highest first)
2. **Asks Tree**: Sell orders sorted by price (lowest first)  
3. **Claimed Seats Tree**: Trader accounts sorted by trader key

## Order Types

### Regular Orders
Standard limit orders that lock tokens on the market until filled or cancelled.

### Global Orders
Capital-efficient orders that don't lock tokens on individual markets. The same tokens can back orders across multiple markets, with tokens moved just-in-time when fills occur.

**Benefits:**
- Reduced capital requirements for market makers
- Ability to provide liquidity across many markets simultaneously
- Automatic capital rebalancing

**Trade-offs:**
- Additional lock contention on global accounts
- 5,000 lamport gas prepayment per order (refunded when order is properly cancelled)
- Risk of eviction from global accounts

### Reverse Orders
Special orders that automatically flip sides when filled:
- Buy orders become sell orders using all proceeds
- Sell orders become buy orders using all proceeds
- Configurable spreads for customized market making
- Permanent discretized liquidity similar to AMM positions

## Program Architecture

### Core Program (manifest)
**Program ID**: `MNFSTqtC93rEfYHB6hF82sKdZpUDFWkViLByLd1k1Ms`

**Key Instructions:**
- `CreateMarket`: Initialize a new trading pair
- `ClaimSeat`: Allocate trader account on market
- `Deposit/Withdraw`: Move tokens to/from market
- `Swap`: Execute immediate trades
- `BatchUpdate`: Place/cancel multiple orders atomically
- `Global*`: Manage global order accounts

### Wrapper Program
**Program ID**: `WRAPYChf58WFCnyjXKJHtrPgzKXgHp6MD9aVDqJBbGh`

**Purpose**: Provides enhanced trading features through CPI calls to core program

**Features:**
- Client order IDs for order tracking
- Fill-or-kill and immediate-or-cancel orders
- Post-only orders with price sliding
- Automatic order adjustment for insufficient funds
- Batch operations with complex logic

### UI Wrapper Program
Specialized wrapper for user interface interactions with additional convenience features.

## Memory Management

### Free List System
Unused memory blocks are managed via a linked list of free nodes, enabling efficient allocation and deallocation without memory fragmentation.

### Block Allocation
- All nodes are exactly 80 bytes (MARKET_BLOCK_SIZE)
- Fixed header size: 256 bytes (MARKET_FIXED_SIZE)
- Dynamic section grows as needed for orders and seats
- Automatic expansion when more space is required

## Security Model

### Economic Disincentives
Manifest relies on economic assumptions to prevent spam and griefing:

1. **Small Order Spam**: Requires funds and rent, making attacks expensive
2. **CU Exhaustion**: Honest actors can reduce size and clear problematic orders
3. **Global Order Spam**: 5,000 lamport prepayment makes cleanup profitable
4. **Global Seat Squatting**: Eviction mechanism with deposit requirements

### Formal Verification
Manifest has undergone formal verification covering:
- Red-black tree properties and hypertree invariants
- Loss of funds prevention
- Operation availability guarantees  
- Matching mechanism correctness

## Performance Characteristics

### Compute Unit Efficiency
- Optimized for high-percentile CU improvements
- Significantly lower costs for active trading
- Efficient batch operations

### Account Requirements
- Swap operations: 8 accounts (theoretical minimum)
- No crank accounts needed
- Predictable account structure

### Capital Efficiency
- Global orders enable capital reuse across markets
- Reverse orders provide permanent liquidity
- Atomic lot sizes maximize price expressiveness

## Integration Patterns

### Direct Core Integration
For applications needing maximum efficiency and control:
```rust
// Direct instruction building
let swap_ix = create_swap_instruction(/* params */);
```

### Wrapper Integration  
For applications needing enhanced features:
```rust
// Wrapper with additional features
let batch_update_ix = create_batch_update_instruction(/* params */);
```

### Client SDK Integration
For TypeScript/JavaScript applications:
```typescript
// High-level client interface
const client = await ManifestClient.getClientForMarket(connection, marketPk, keypair);
await client.placeOrder(/* params */);
```

## Comparison with Other Orderbooks

| Feature | Openbook | Phoenix | Manifest |
|---------|----------|---------|----------|
| Crankless | No | Yes | Yes |
| Feeless | No | No | Yes |
| Atomic lot sizes | No | No | Yes |
| Creation Rent | 2 SOL | 3+ SOL | 0.007 SOL |
| Token 2022 | No | No | Yes |
| Capital Efficient | No | No | Yes |
| Composable wrapper | No | No | Yes |

## Next Steps

This architecture enables Manifest to serve as the foundation for various trading applications while maintaining simplicity, efficiency, and composability. The modular design allows developers to build custom trading experiences on top of the core orderbook primitive.
