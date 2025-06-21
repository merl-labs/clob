# Manifest Program Instructions

## Overview

Manifest provides 14 core instructions for orderbook operations, divided into market operations, trading operations, and global account management.

**Program ID**: `MNFSTqtC93rEfYHB6hF82sKdZpUDFWkViLByLd1k1Ms`

## Market Management Instructions

### CreateMarket (0)
Creates a new trading pair market.

**Accounts:**
- `payer` (writable, signer): Account paying for market creation
- `market` (writable): Market account to initialize
- `system_program`: Solana system program
- `base_mint`: Base token mint
- `quote_mint`: Quote token mint  
- `base_vault` (writable): Base token vault PDA
- `quote_vault` (writable): Quote token vault PDA
- `token_program`: Token program
- `token_program_22`: Token 2022 program

**Parameters:** None

**Usage:**
```rust
let create_market_ix = create_market_instruction(
    &market_key,
    &base_mint,
    &quote_mint,
    &payer
);
```

### ClaimSeat (1)
Allocates a trader seat on a market for deposits and trading.

**Accounts:**
- `payer` (writable, signer): Account paying for seat allocation
- `market` (writable): Market account
- `system_program`: Solana system program

**Parameters:** None

### Expand (5)
Manually expands market account size (usually done automatically).

**Accounts:**
- `payer` (writable, signer): Account paying for expansion
- `market` (writable): Market account to expand
- `system_program`: Solana system program

**Parameters:** None

## Token Management Instructions

### Deposit (2)
Deposits tokens into a market for trading.

**Accounts:**
- `payer` (writable, signer): Account making deposit
- `market` (writable): Market account
- `trader_token` (writable): Trader's token account
- `vault` (writable): Market vault for the token
- `token_program`: Token program (or Token 2022)
- `mint`: Token mint (required for Token 2022)

**Parameters:**
```rust
pub struct DepositParams {
    pub amount_atoms: u64,
    pub trader_index_hint: Option<DataIndex>,
}
```

**Usage:**
```rust
let deposit_ix = deposit_instruction(
    &market_key,
    &payer,
    &mint,
    amount_atoms,
    &trader_token_account,
    token_program_id,
    trader_index_hint
);
```

### Withdraw (3)
Withdraws tokens from a market.

**Accounts:**
- `payer` (writable, signer): Account making withdrawal
- `market` (writable): Market account
- `trader_token` (writable): Trader's token account
- `vault` (writable): Market vault for the token
- `token_program`: Token program (or Token 2022)
- `mint`: Token mint (required for Token 2022)

**Parameters:**
```rust
pub struct WithdrawParams {
    pub amount_atoms: u64,
    pub trader_index_hint: Option<DataIndex>,
}
```

## Trading Instructions

### Swap (4)
Executes immediate trades using wallet funds.

**Accounts:**
- `payer` (writable, signer): Transaction payer
- `market` (writable): Market account
- `system_program`: Solana system program
- `trader_base` (writable): Trader's base token account
- `trader_quote` (writable): Trader's quote token account
- `base_vault` (writable): Market base vault
- `quote_vault` (writable): Market quote vault
- `token_program_base`: Token program for base
- `base_mint` (optional): Base mint for Token 2022
- `token_program_quote` (optional): Token program for quote
- `quote_mint` (optional): Quote mint for Token 2022
- `global` (optional, writable): Global account
- `global_vault` (optional, writable): Global vault

**Parameters:**
```rust
pub struct SwapParams {
    pub in_atoms: u64,
    pub out_atoms: u64,
    pub is_base_in: bool,
    pub is_exact_in: bool,
}
```

**Usage:**
```rust
let swap_params = SwapParams::new(
    in_atoms,
    out_atoms,
    is_base_in,
    is_exact_in
);
let swap_ix = swap_instruction(/* accounts */, swap_params);
```

### SwapV2 (13)
Enhanced swap with separate owner and payer accounts.

**Accounts:** Same as Swap plus:
- `owner` (writable, signer): Token account owner

### BatchUpdate (6)
Places and cancels multiple orders atomically.

**Accounts:**
- `payer` (writable, signer): Transaction payer
- `market` (writable): Market account
- `system_program`: Solana system program
- Global account fields (optional): For global order support

**Parameters:**
```rust
pub struct BatchUpdateParams {
    pub trader_index_hint: Option<DataIndex>,
    pub cancels: Vec<CancelOrderParams>,
    pub orders: Vec<PlaceOrderParams>,
}

pub struct CancelOrderParams {
    order_sequence_number: u64,
    order_index_hint: Option<DataIndex>,
}

pub struct PlaceOrderParams {
    base_atoms: u64,
    price_mantissa: u32,
    price_exponent: i8,
    is_bid: bool,
    last_valid_slot: u32,
    order_type: OrderType,
}
```

**Order Types:**
```rust
pub enum OrderType {
    Limit = 0,           // Standard limit order
    ImmediateOrCancel = 1, // Take-only order
    PostOnly = 2,        // Maker-only order
    Global = 3,          // Uses global account funds
    Reverse = 4,         // AMM-like auto-flipping order
}
```

**Usage:**
```rust
let place_order = PlaceOrderParams::new(
    base_atoms,
    price_mantissa,
    price_exponent,
    is_bid,
    last_valid_slot,
    OrderType::Limit
);

let cancel_order = CancelOrderParams::new(order_sequence_number);

let batch_params = BatchUpdateParams::new(
    trader_index_hint,
    vec![cancel_order],
    vec![place_order]
);
```

## Global Account Instructions

Global accounts enable capital-efficient trading across multiple markets.

### GlobalCreate (7)
Creates a global account for a token.

**Accounts:**
- `payer` (writable, signer): Account paying for creation
- `global` (writable): Global account to create
- `system_program`: Solana system program
- `mint`: Token mint
- `global_vault` (writable): Global vault PDA
- `token_program`: Token program

**Parameters:** None

### GlobalAddTrader (8)
Adds a trader to a global account.

**Accounts:**
- `payer` (writable, signer): Trader to add
- `global` (writable): Global account
- `system_program`: Solana system program

**Parameters:** None

### GlobalDeposit (9)
Deposits tokens into a global account.

**Accounts:**
- `payer` (writable, signer): Account making deposit
- `global` (writable): Global account
- `mint`: Token mint
- `global_vault` (writable): Global vault
- `trader_token` (writable): Trader's token account
- `token_program`: Token program

**Parameters:**
```rust
pub struct GlobalDepositParams {
    pub amount_atoms: u64,
}
```

### GlobalWithdraw (10)
Withdraws tokens from a global account.

**Accounts:** Same as GlobalDeposit

**Parameters:**
```rust
pub struct GlobalWithdrawParams {
    pub amount_atoms: u64,
}
```

### GlobalEvict (11)
Evicts a trader from a global account by depositing more than them.

**Accounts:**
- `payer` (writable, signer): Account performing eviction
- `global` (writable): Global account
- `mint`: Token mint
- `global_vault` (writable): Global vault
- `trader_token`: Evictor's token account
- `evictee_token`: Evictee's token account
- `token_program`: Token program

**Parameters:**
```rust
pub struct GlobalEvictParams {
    amount_atoms: u64, // Must be > evictee's deposit
}
```

### GlobalClean (12)
Removes unfillable global orders and claims gas prepayment.

**Accounts:**
- `payer` (writable, signer): Cleaner receiving gas prepayment
- `market` (writable): Market account
- `system_program`: Solana system program
- `global` (writable): Global account

**Parameters:**
```rust
pub struct GlobalCleanParams {
    order_sequence_number: u64,
}
```

## Instruction Building Helpers

The program provides helper functions for building instructions:

```rust
// Market creation
create_market_instructions(market, base_mint, quote_mint, creator)

// Trading
batch_update_instruction(market, payer, trader_hint, cancels, orders, ...)
swap_instruction(accounts, swap_params)

// Global operations
global_deposit_instruction(mint, payer, trader_token, token_program, atoms)
global_withdraw_instruction(mint, payer, trader_token, token_program, atoms)
```

## Validation and Constraints

- **Seat Requirement**: Most trading operations require claiming a seat first
- **Token Program Compatibility**: Supports both SPL Token and Token 2022
- **Global Order Limits**: Maximum seats per global account enforced
- **Gas Prepayment**: Global orders require 5,000 lamport prepayment
- **Expiration**: Orders can have expiration slots for automatic cleanup
- **Price Precision**: Prices use mantissa/exponent format for flexibility

## Error Handling

Instructions return standard Solana `ProgramResult` with custom error types defined in `ManifestError` enum for specific failure cases like insufficient funds, invalid parameters, or market state violations.
