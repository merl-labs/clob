# MagicBlock Ephemeral Rollups Integration Guide for Manifest

## Overview

This guide provides step-by-step instructions for integrating MagicBlock's ephemeral rollups with the Manifest orderbook to achieve ultra-low latency trading (10ms block times), gasless transactions, and horizontal scaling while maintaining full Solana compatibility.

## What are MagicBlock Ephemeral Rollups?

MagicBlock's ephemeral rollups are temporary, high-performance execution environments that extend Solana's capabilities:

- **Ultra-low latency**: 10ms block times vs Solana's 400ms
- **Gasless transactions**: Near-zero transaction fees
- **Horizontal scaling**: Multiple rollups processing millions of TPS
- **Full SVM compatibility**: No code changes required for existing programs
- **State integrity**: Seamless synchronization with Solana base layer

## Why Integrate with Manifest?

Manifest's orderbook is perfect for ephemeral rollup integration because:

1. **High-frequency trading**: Benefits from 10ms latency for competitive order placement
2. **Market making**: Gasless transactions enable profitable micro-strategies
3. **Scalability**: Multiple markets can run on separate rollups
4. **Real-time updates**: Instant orderbook state changes for better UX
5. **Capital efficiency**: Global orders work seamlessly across rollups

## Prerequisites

Before starting the integration:

1. **Existing Manifest deployment** on Solana devnet/mainnet
2. **MagicBlock SDK** installed in your project
3. **Understanding of Manifest architecture** (core + wrapper programs)
4. **Solana development environment** set up

## Step 1: Install MagicBlock SDK

### For Rust Programs

```bash
cargo add ephemeral_rollups_sdk
```

### For TypeScript Clients

```bash
npm install @magicblock-labs/ephemeral-rollups-sdk
# or
yarn add @magicblock-labs/ephemeral-rollups-sdk
```

## Step 2: Identify Delegation Candidates

In Manifest, the best candidates for delegation are:

### Market Accounts

- **Market state**: Core orderbook data and trees
- **Benefits**: Ultra-fast order matching and execution
- **Considerations**: Large accounts may have higher delegation costs

### Trader Seats (ClaimedSeat)

- **Individual trader positions**: Balances and order tracking
- **Benefits**: Instant balance updates and order management
- **Considerations**: Per-trader delegation for isolated performance

### Global Accounts

- **Cross-market liquidity**: Global order management
- **Benefits**: Real-time capital reallocation across markets
- **Considerations**: Shared state requires careful coordination

## Step 3: Add Delegation Hooks to Manifest

### Option A: Extend Core Program (Recommended for New Deployments)

Add delegation instructions to the core Manifest program:

```rust
// programs/manifest/src/program/instruction.rs

use ephemeral_rollups_sdk::cpi::delegate_account;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelegateMarketParams {
    pub update_frequency_ms: u64,  // How often to sync with base layer
    pub time_limit: u64,           // 0 for no limit
}

// Add to ManifestInstruction enum
pub enum ManifestInstruction {
    // ... existing instructions
    DelegateMarket = 14,
    UndelegateMarket = 15,
    CommitMarket = 16,
}
```

### Delegation Processor

```rust
// programs/manifest/src/program/processor/delegate_market.rs

pub fn process_delegate_market(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let params: DelegateMarketParams = DelegateMarketParams::try_from_slice(data)?;
    let account_iter = &mut accounts.iter();
    
    let payer = next_account_info(account_iter)?;
    let market = next_account_info(account_iter)?;
    let delegation_program = next_account_info(account_iter)?;
    
    // Delegate the market account to ephemeral rollup
    delegate_account(
        payer,
        market,
        program_id,
        delegation_program,
        &[], // No seeds needed for market account
        params.time_limit,
        params.update_frequency_ms,
    )?;
    
    Ok(())
}
```

### Undelegation Processor

```rust
// programs/manifest/src/program/processor/undelegate_market.rs

pub fn process_undelegate_market(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let payer = next_account_info(account_iter)?;
    let market = next_account_info(account_iter)?;
    let magic_context = next_account_info(account_iter)?;
    let magic_program = next_account_info(account_iter)?;
    
    // Commit and undelegate the market account
    commit_and_undelegate_accounts(
        payer,
        vec![market],
        magic_context,
        magic_program,
    )?;
    
    Ok(())
}
```

### Option B: Wrapper Program Integration (Recommended for Existing Deployments)

Extend the wrapper program to handle delegation:

```rust
// programs/wrapper/src/instruction.rs

pub enum ManifestWrapperInstruction {
    // ... existing instructions
    DelegateMarket = 6,
    UndelegateMarket = 7,
    CommitMarket = 8,
}
```

```rust
// programs/wrapper/src/processors/delegate_market.rs

pub fn process_delegate_market(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let params: DelegateMarketParams = DelegateMarketParams::try_from_slice(data)?;
    let account_iter = &mut accounts.iter();
    
    let owner = Signer::new(next_account_info(account_iter)?)?;
    let wrapper_state = ManifestAccountInfo::<ManifestWrapperStateFixed>::new(
        next_account_info(account_iter)?
    )?;
    let market = next_account_info(account_iter)?;
    let delegation_program = next_account_info(account_iter)?;
    
    // Verify wrapper owns this market interaction
    let wrapper_fixed = wrapper_state.get_fixed()?;
    require!(
        wrapper_fixed.trader == *owner.key,
        ProgramError::InvalidAccountData,
        "Wrapper owner mismatch"
    )?;
    
    // Delegate the market account
    delegate_account(
        &owner,
        market,
        &manifest::ID, // Core program owns the market
        delegation_program,
        &[],
        params.time_limit,
        params.update_frequency_ms,
    )?;
    
    Ok(())
}
```

## Step 4: Configure RPC Endpoints

Set up MagicBlock's Smart Router for automatic transaction routing:

### Development Environment

```typescript
// Configure RPC endpoints
const MAGICBLOCK_DEVNET_RPC = "https://devnet-rpc.magicblock.app";
const SOLANA_DEVNET_RPC = "https://api.devnet.solana.com";
const EPHEMERAL_DEVNET_RPC = "https://devnet.magicblock.app";

// Use Smart Router for automatic routing
const connection = new Connection(MAGICBLOCK_DEVNET_RPC, 'confirmed');
```

### Production Environment

```typescript
// Production endpoints (contact MagicBlock for access)
const MAGICBLOCK_MAINNET_RPC = "https://mainnet-rpc.magicblock.app";
const connection = new Connection(MAGICBLOCK_MAINNET_RPC, 'confirmed');
```

## Step 5: Client Integration

### TypeScript Client Integration

```typescript
import { ManifestClient } from '@cks-systems/manifest-sdk';
import { Connection, PublicKey, Keypair } from '@solana/web3.js';

class EphemeralManifestClient extends ManifestClient {
  private isDelegated: boolean = false;
  
  constructor(
    connection: Connection,
    wrapper: Wrapper | null,
    market: Market,
    payer: PublicKey | null,
    baseMint: Mint,
    quoteMint: Mint,
    baseGlobal: Global | null,
    quoteGlobal: Global | null,
  ) {
    super(connection, wrapper, market, payer, baseMint, quoteMint, baseGlobal, quoteGlobal);
  }
  
  // Delegate market to ephemeral rollup
  async delegateMarket(
    updateFrequencyMs: number = 3000,
    timeLimit: number = 0
  ): Promise<string> {
    const delegateIx = this.createDelegateMarketInstruction(
      updateFrequencyMs,
      timeLimit
    );
    
    const tx = new Transaction().add(delegateIx);
    const signature = await sendAndConfirmTransaction(
      this.connection,
      tx,
      [this.payer as Keypair]
    );
    
    this.isDelegated = true;
    return signature;
  }
  
  // Undelegate market from ephemeral rollup
  async undelegateMarket(): Promise<string> {
    const undelegateIx = this.createUndelegateMarketInstruction();
    
    const tx = new Transaction().add(undelegateIx);
    const signature = await sendAndConfirmTransaction(
      this.connection,
      tx,
      [this.payer as Keypair]
    );
    
    this.isDelegated = false;
    return signature;
  }
  
  // Override placeOrder for ephemeral-aware execution
  async placeOrderEphemeral(params: WrapperPlaceOrderParamsExternal): Promise<string> {
    if (!this.isDelegated) {
      console.warn("Market not delegated - using standard execution");
      return super.placeOrder(params);
    }
    
    // Orders on delegated markets execute with 10ms latency
    const orderIx = this.placeOrderIx(params);
    const tx = new Transaction().add(orderIx);
    
    // This will be routed to ephemeral rollup automatically
    return await sendAndConfirmTransaction(
      this.connection,
      tx,
      [this.payer as Keypair]
    );
  }
}
```

## Step 6: Trading Strategy Implementation

### High-Frequency Market Making

```typescript
class EphemeralMarketMaker {
  private client: EphemeralManifestClient;
  private isRunning: boolean = false;
  
  constructor(client: EphemeralManifestClient) {
    this.client = client;
  }
  
  async start() {
    // Delegate market for ultra-low latency
    await this.client.delegateMarket(1000); // 1 second sync frequency
    
    this.isRunning = true;
    this.runMarketMaking();
  }
  
  private async runMarketMaking() {
    while (this.isRunning) {
      try {
        // Get current market state (10ms latency)
        await this.client.reload();
        const market = this.client.market;
        
        const bestBid = market.bestBidPrice();
        const bestAsk = market.bestAskPrice();
        
        if (bestBid && bestAsk) {
          const spread = bestAsk - bestBid;
          const midPrice = (bestBid + bestAsk) / 2;
          
          // Place tight spreads with gasless transactions
          const bidPrice = midPrice - 0.001;
          const askPrice = midPrice + 0.001;
          
          // These execute in ~10ms on ephemeral rollup
          await Promise.all([
            this.client.placeOrderEphemeral({
              numBaseTokens: 1.0,
              tokenPrice: bidPrice,
              isBid: true,
              orderType: OrderType.PostOnly,
              clientOrderId: Date.now(),
            }),
            this.client.placeOrderEphemeral({
              numBaseTokens: 1.0,
              tokenPrice: askPrice,
              isBid: false,
              orderType: OrderType.PostOnly,
              clientOrderId: Date.now() + 1,
            }),
          ]);
        }
        
        // High-frequency loop (possible due to gasless transactions)
        await new Promise(resolve => setTimeout(resolve, 50)); // 50ms loop
        
      } catch (error) {
        console.error('Market making error:', error);
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
  }
  
  async stop() {
    this.isRunning = false;
    
    // Cancel all orders and undelegate
    await this.client.cancelAllIx();
    await this.client.undelegateMarket();
  }
}
```

## Step 7: Multi-Market Scaling

### Horizontal Scaling with Multiple Rollups

```typescript
class MultiMarketEphemeralTrader {
  private clients: Map<string, EphemeralManifestClient> = new Map();
  
  async addMarket(marketAddress: string, connection: Connection, trader: Keypair) {
    const client = await EphemeralManifestClient.getClientForMarket(
      connection,
      new PublicKey(marketAddress),
      trader
    );
    
    // Each market gets its own ephemeral rollup
    await client.delegateMarket(2000); // 2 second sync
    
    this.clients.set(marketAddress, client);
  }
  
  async arbitrageAcrossMarkets(tokenA: string, tokenB: string) {
    const marketA = this.clients.get(tokenA);
    const marketB = this.clients.get(tokenB);
    
    if (!marketA || !marketB) return;
    
    // Get prices from both markets (10ms latency each)
    await Promise.all([marketA.reload(), marketB.reload()]);
    
    const priceA = marketA.market.bestAskPrice();
    const priceB = marketB.market.bestBidPrice();
    
    if (priceA && priceB && priceB > priceA * 1.001) { // 0.1% profit threshold
      // Execute arbitrage with gasless transactions
      await Promise.all([
        marketA.placeOrderEphemeral({
          numBaseTokens: 1.0,
          tokenPrice: priceA,
          isBid: false, // Buy on market A
          orderType: OrderType.ImmediateOrCancel,
          clientOrderId: Date.now(),
        }),
        marketB.placeOrderEphemeral({
          numBaseTokens: 1.0,
          tokenPrice: priceB,
          isBid: true, // Sell on market B
          orderType: OrderType.ImmediateOrCancel,
          clientOrderId: Date.now() + 1,
        }),
      ]);
    }
  }
}
```

## Step 8: Monitoring and Management

### State Synchronization Monitoring

```typescript
class EphemeralStateMonitor {
  private client: EphemeralManifestClient;
  
  constructor(client: EphemeralManifestClient) {
    this.client = client;
  }
  
  async monitorSyncStatus() {
    setInterval(async () => {
      try {
        // Check if manual commit is needed
        const shouldCommit = await this.shouldCommitState();
        
        if (shouldCommit) {
          await this.client.commitMarket();
          console.log('Manual state commit completed');
        }
        
      } catch (error) {
        console.error('Sync monitoring error:', error);
      }
    }, 10000); // Check every 10 seconds
  }
  
  private async shouldCommitState(): Promise<boolean> {
    // Implement logic to determine if manual commit is needed
    // e.g., based on time since last commit, number of transactions, etc.
    return false;
  }
}
```

## Best Practices

### 1. Delegation Strategy

- **Market-level delegation**: For high-frequency trading on specific markets
- **Trader-level delegation**: For individual trader seat optimization
- **Global-level delegation**: For cross-market capital efficiency

### 2. Sync Frequency Optimization

- **High-frequency trading**: 1-3 second sync for rapid base layer updates
- **Market making**: 5-10 second sync for balance between performance and cost
- **Long-term positions**: 30+ second sync for cost optimization

### 3. Error Handling

- **Delegation failures**: Fallback to standard Solana execution
- **Sync issues**: Implement retry logic with exponential backoff
- **State conflicts**: Handle mixed delegated/undelegated account errors

### 4. Cost Management

- **Monitor delegation costs**: Track SOL spent on delegation operations
- **Optimize sync frequency**: Balance performance vs. base layer transaction costs
- **Batch operations**: Group multiple actions in single transactions

## Troubleshooting

### Common Issues

1. **Mixed delegation state errors**
   - Ensure all writable accounts have same delegation status
   - Use separate transactions for delegated and undelegated operations

2. **High delegation costs**
   - Optimize sync frequency based on trading patterns
   - Consider delegating smaller account subsets

3. **State synchronization delays**
   - Monitor base layer congestion
   - Implement manual commit triggers for critical operations

### Support Resources

- **MagicBlock Documentation**: <https://docs.magicblock.gg>
- **MagicBlock Discord**: <https://discord.com/invite/zHFtdVMA6e>
- **Manifest GitHub**: <https://github.com/CKS-Systems/manifest>

## Conclusion

Integrating MagicBlock's ephemeral rollups with Manifest enables:

- **10ms order execution** for competitive trading
- **Gasless micro-strategies** for profitable market making
- **Horizontal scaling** across multiple markets
- **Real-time orderbook updates** for superior UX
- **Full Solana compatibility** with existing infrastructure

This integration positions Manifest as the highest-performance orderbook on Solana, capable of supporting institutional-grade trading applications while maintaining decentralization and composability.
