# Manifest Client SDKs Documentation

## Overview

Manifest provides client SDKs in TypeScript and Rust for interacting with the orderbook. The SDKs offer high-level abstractions over the core program instructions, making it easy to integrate Manifest into applications.

## TypeScript SDK

### Installation
```bash
yarn add @cks-systems/manifest-sdk
# or
npm install @cks-systems/manifest-sdk
```

### Core Classes

#### ManifestClient
The main entry point for interacting with Manifest markets.

```typescript
export class ManifestClient {
  public connection: Connection;
  public wrapper: Wrapper | null;
  public market: Market;
  public baseGlobal: Global | null;
  public quoteGlobal: Global | null;
  public isBase22: boolean;
  public isQuote22: boolean;
}
```

**Static Factory Methods:**
```typescript
// Automatic setup with seat claiming
static async getClientForMarket(
  connection: Connection,
  marketPk: PublicKey,
  payerKeypair: Keypair
): Promise<ManifestClient>

// Manual setup (requires prior setup)
static async getClientForMarketNoPrivateKey(
  connection: Connection,
  marketPk: PublicKey,
  trader: PublicKey
): Promise<ManifestClient>

// Check what setup is needed
static async getSetupIxs(
  connection: Connection,
  marketPk: PublicKey,
  trader: PublicKey
): Promise<SetupData>
```

**Market Discovery:**
```typescript
// List all markets
static async listMarketPublicKeys(
  connection: Connection
): Promise<PublicKey[]>

// Find markets for specific token pair
static async listMarketsForMints(
  connection: Connection,
  baseMint: PublicKey,
  quoteMint: PublicKey,
  useApi?: boolean
): Promise<PublicKey[]>
```

### Order Management

#### Place Orders
```typescript
// Single order placement
placeOrderIx(params: WrapperPlaceOrderParamsExternal): TransactionInstruction

// Place order with automatic deposits
async placeOrderWithRequiredDepositIxs(
  payer: PublicKey,
  params: WrapperPlaceOrderParamsExternal
): Promise<TransactionInstruction[]>

// Batch operations
batchUpdateIx(
  placeParams: WrapperPlaceOrderParamsExternal[],
  cancelParams: WrapperCancelOrderParams[],
  cancelAll: boolean
): TransactionInstruction
```

**Order Parameters:**
```typescript
interface WrapperPlaceOrderParamsExternal {
  numBaseTokens: number;        // Order size in base tokens
  tokenPrice: number;           // Price in quote tokens per base token
  isBid: boolean;              // true = buy, false = sell
  lastValidSlot?: number;      // Expiration slot (optional)
  orderType: OrderType;        // Order behavior
  clientOrderId: number;       // Client-side order ID
  spreadBps?: number;          // Spread for reverse orders (optional)
}

enum OrderType {
  Limit,                       // Standard limit order
  ImmediateOrCancel,          // Take-only order
  PostOnly,                   // Maker-only order
  Global,                     // Uses global account funds
  Reverse,                    // Auto-flipping AMM-like order
}
```

#### Cancel Orders
```typescript
// Cancel specific orders
cancelOrderIx(params: WrapperCancelOrderParams): TransactionInstruction

// Cancel all orders
cancelAllIx(): TransactionInstruction

interface WrapperCancelOrderParams {
  clientOrderId: number;
}
```

### Token Management

#### Deposits and Withdrawals
```typescript
// Deposit tokens to market
depositIx(
  payer: PublicKey,
  mint: PublicKey,
  amountTokens: number
): TransactionInstruction

// Withdraw tokens from market
withdrawIx(
  payer: PublicKey,
  mint: PublicKey,
  amountTokens: number
): TransactionInstruction
```

#### Global Account Operations
```typescript
// Create global account
static createGlobalIx(
  payer: PublicKey,
  mint: PublicKey
): TransactionInstruction

// Add trader to global account
static globalAddTraderIx(
  payer: PublicKey,
  mint: PublicKey
): TransactionInstruction

// Deposit to global account
static globalDepositIx(
  payer: PublicKey,
  mint: PublicKey,
  amountTokens: number
): TransactionInstruction

// Withdraw from global account
static globalWithdrawIx(
  payer: PublicKey,
  mint: PublicKey,
  amountTokens: number
): TransactionInstruction
```

### Trading Operations

#### Swap (Immediate Trading)
```typescript
// Direct swap without resting orders
swapIx(params: SwapParams): TransactionInstruction

interface SwapParams {
  inAtoms: bignum;             // Input amount in atoms
  outAtoms: bignum;            // Expected output amount in atoms
  isBaseIn: boolean;           // true = base->quote, false = quote->base
  isExactIn: boolean;          // true = exact input, false = exact output
}
```

### Market Data Access

#### Market Class
```typescript
export class Market {
  address: PublicKey;
  
  // Load market data
  static async loadFromAddress({
    connection,
    address
  }: {
    connection: Connection;
    address: PublicKey;
  }): Promise<Market>
  
  // Reload market state
  async reload(connection: Connection): Promise<void>
}
```

**Market Data Methods:**
```typescript
// Basic market info
baseMint(): PublicKey
quoteMint(): PublicKey
baseDecimals(): number
quoteDecimals(): number

// Orderbook data
bidsL2(): RestingOrder[]              // All bids (best first)
asksL2(): RestingOrder[]              // All asks (best first)
bestBidPrice(): number | undefined    // Best bid price
bestAskPrice(): number | undefined    // Best ask price

// Market statistics
quoteVolume(): bignum                 // Lifetime trading volume
orderSequenceNumber(): bignum         // Next order sequence number

// Account balances
getClaimedSeat(trader: PublicKey): ClaimedSeat | undefined
```

**RestingOrder Interface:**
```typescript
interface RestingOrder {
  trader: PublicKey;           // Order owner
  numBaseTokens: number;       // Order size
  tokenPrice: number;          // Order price
  lastValidSlot: number;       // Expiration slot
  sequenceNumber: bignum;      // Unique order ID
  orderType: OrderType;        // Order behavior
}
```

#### Global Class
```typescript
export class Global {
  address: PublicKey;
  
  // Load global account data
  static async loadFromAddress({
    connection,
    address
  }: {
    connection: Connection;
    address: PublicKey;
  }): Promise<Global | null>
  
  // Get trader balance
  async getGlobalBalanceTokens(
    connection: Connection,
    trader: PublicKey
  ): Promise<number>
}
```

### Wrapper Integration

#### Wrapper Class
Tracks trader state across multiple markets:

```typescript
export class Wrapper {
  address: PublicKey;
  
  // Get market-specific data
  marketInfoForMarket(market: PublicKey): WrapperMarketInfo | undefined
  openOrdersForMarket(market: PublicKey): WrapperOpenOrder[]
  
  // Display wrapper state
  prettyPrint(): void
}

interface WrapperMarketInfo {
  market: PublicKey;
  baseBalanceAtoms: bignum;    // Available base tokens
  quoteBalanceAtoms: bignum;   // Available quote tokens
  quoteVolumeAtoms: bignum;    // Trading volume
  orders: WrapperOpenOrder[];  // Open orders
  lastUpdatedSlot: number;     // Last sync slot
}

interface WrapperOpenOrder {
  clientOrderId: number;       // Client order ID
  orderSequenceNumber: bignum; // On-chain order ID
  price: number;               // Order price
  numBaseTokens: number;       // Order size
  isBid: boolean;             // Order side
  lastValidSlot: number;       // Expiration
  orderType: OrderType;        // Order type
}
```

### Usage Examples

#### Basic Trading Flow
```typescript
import { ManifestClient, OrderType } from '@cks-systems/manifest-sdk';
import { Connection, Keypair, Transaction } from '@solana/web3.js';

// Initialize client
const connection = new Connection('https://api.mainnet-beta.solana.com');
const trader = Keypair.generate();
const marketPk = new PublicKey('...');

const client = await ManifestClient.getClientForMarket(
  connection,
  marketPk,
  trader
);

// Place a limit buy order
const buyOrder = client.placeOrderIx({
  numBaseTokens: 1.0,          // Buy 1 base token
  tokenPrice: 100.0,           // At 100 quote tokens per base
  isBid: true,                 // Buy order
  orderType: OrderType.Limit,  // Limit order
  clientOrderId: 1,            // Client tracking ID
});

// Execute transaction
const tx = new Transaction().add(buyOrder);
await sendAndConfirmTransaction(connection, tx, [trader]);

// Check order status
await client.reload();
const openOrders = client.wrapper.openOrdersForMarket(marketPk);
console.log('Open orders:', openOrders);
```

#### Market Data Reading
```typescript
// Load market without trading
const market = await Market.loadFromAddress({
  connection,
  address: marketPk
});

// Get orderbook data
const bids = market.bidsL2();
const asks = market.asksL2();
const spread = market.bestAskPrice() - market.bestBidPrice();

console.log(`Best bid: ${market.bestBidPrice()}`);
console.log(`Best ask: ${market.bestAskPrice()}`);
console.log(`Spread: ${spread}`);

// Display full orderbook
market.prettyPrint();
```

#### Global Order Trading
```typescript
// Create and fund global account
const createGlobalIx = ManifestClient.createGlobalIx(
  trader.publicKey,
  baseMint
);

const depositGlobalIx = ManifestClient.globalDepositIx(
  trader.publicKey,
  baseMint,
  10.0  // Deposit 10 base tokens
);

// Place global order (uses global account funds)
const globalOrderIx = client.placeOrderIx({
  numBaseTokens: 5.0,
  tokenPrice: 100.0,
  isBid: false,                // Sell order
  orderType: OrderType.Global, // Global order
  clientOrderId: 2,
});

const tx = new Transaction()
  .add(createGlobalIx)
  .add(depositGlobalIx)
  .add(globalOrderIx);

await sendAndConfirmTransaction(connection, tx, [trader]);
```

## Rust SDK

### Jupiter AMM Integration
The Rust client implements the Jupiter AMM interface for routing integration:

```rust
use jupiter_amm_interface::{Amm, Quote, QuoteParams, SwapParams};
use manifest_client::{ManifestAmm, ManifestAmmGlobal};

// Standard AMM (no global orders)
let amm = ManifestAmm::new(market_account);
let quote = amm.quote(&QuoteParams {
    input_mint: base_mint,
    output_mint: quote_mint,
    amount: 1_000_000,  // 1 token (6 decimals)
})?;

// Global AMM (includes global orders)
let global_amm = ManifestAmmGlobal::new(market_account, global_accounts);
let quote = global_amm.quote(&quote_params)?;
```

### Direct Market Access
```rust
use manifest::{
    state::{MarketValue, DynamicAccount},
    program::AddOrderToMarketArgs,
};

// Load market data
let market_data: Vec<u8> = /* load from account */;
let market: MarketValue = DynamicAccount::new(market_fixed, market_data);

// Calculate impact
let quote_atoms = market.impact_quote_atoms(
    is_bid,
    base_atoms,
    &global_accounts
)?;

// Place order directly
let result = market.place_order(AddOrderToMarketArgs {
    market: market_key,
    trader_index,
    num_base_atoms: BaseAtoms::new(1_000_000),
    price: QuoteAtomsPerBaseAtom::try_from(100.0)?,
    is_bid: true,
    last_valid_slot: NO_EXPIRATION_LAST_VALID_SLOT,
    order_type: OrderType::Limit,
    global_trade_accounts_opts: &[None, None],
    current_slot: None,
})?;
```

## Error Handling

### TypeScript Error Types
```typescript
// Connection errors
try {
  await client.reload();
} catch (error) {
  console.error('Failed to reload market:', error);
}

// Transaction errors
try {
  const signature = await sendAndConfirmTransaction(connection, tx, [trader]);
} catch (error) {
  if (error.message.includes('insufficient funds')) {
    // Handle insufficient balance
  }
}
```

### Common Error Scenarios
1. **Insufficient Balance**: Not enough tokens for order
2. **Invalid Price**: Price outside valid range
3. **Market Full**: No available seats or memory
4. **Order Expired**: Order past expiration slot
5. **Global Account Missing**: Global order without global account

## Performance Considerations

### Batch Operations
```typescript
// Efficient: Single batch transaction
const batchIx = client.batchUpdateIx(
  [order1, order2, order3],  // Place multiple orders
  [cancel1, cancel2],        // Cancel multiple orders
  false                      // Don't cancel all
);

// Inefficient: Multiple separate transactions
const place1 = client.placeOrderIx(order1);
const place2 = client.placeOrderIx(order2);
const cancel1 = client.cancelOrderIx(cancelParams);
```

### Connection Management
```typescript
// Use connection pooling for high-frequency applications
const connection = new Connection(rpcUrl, {
  commitment: 'confirmed',
  wsEndpoint: wsUrl,
});

// Subscribe to account changes for real-time updates
connection.onAccountChange(marketPk, (accountInfo) => {
  const market = Market.loadFromBuffer({
    address: marketPk,
    buffer: accountInfo.data,
    slot: accountInfo.slot,
  });
  // Handle market update
});
```

### Memory Optimization
```typescript
// Reload only when necessary
if (needsUpdate) {
  await client.reload();
}

// Use read-only market access when not trading
const market = await Market.loadFromAddress({ connection, address: marketPk });
```

The Manifest client SDKs provide comprehensive, type-safe interfaces for all orderbook operations while abstracting away the complexity of the underlying program instructions and account management.
