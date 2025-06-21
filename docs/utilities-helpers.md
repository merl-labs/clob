# Manifest Utilities and Helpers Documentation

## Overview

Manifest provides a comprehensive set of utility functions, helper modules, and supporting components that facilitate development, testing, and integration. These utilities span validation, price calculations, account management, and client-side operations.

## Core Program Utilities

### Validation Framework

#### Account Validation
```rust
// programs/manifest/src/validation/manifest_checker.rs

pub trait ManifestAccount {
    fn verify_discriminant(&self) -> ProgramResult;
}

pub struct ManifestAccountInfo<'a, 'info, T: ManifestAccount + Pod + Clone> {
    pub info: &'a AccountInfo<'info>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, 'info, T: ManifestAccount + Get + Clone> ManifestAccountInfo<'a, 'info, T> {
    // Create validated account wrapper
    pub fn new(info: &'a AccountInfo<'info>) -> Result<Self, ProgramError>
    
    // Create for uninitialized accounts
    pub fn new_init(info: &'a AccountInfo<'info>) -> Result<Self, ProgramError>
    
    // Get typed reference to account data
    pub fn get_fixed(&self) -> Result<Ref<'_, T>, ProgramError>
}
```

#### Token Program Validation
```rust
// programs/manifest/src/validation/token_checkers.rs

pub struct MintAccountInfo<'a, 'info> {
    pub mint: Mint,
    pub info: &'a AccountInfo<'info>,
}

impl<'a, 'info> MintAccountInfo<'a, 'info> {
    pub fn new(info: &'a AccountInfo<'info>) -> Result<Self, ProgramError> {
        check_spl_token_program_account(info.owner)?;
        let mint: Mint = StateWithExtensions::<Mint>::unpack(&info.data.borrow())?.base;
        Ok(Self { mint, info })
    }
}

pub struct TokenAccountInfo<'a, 'info> {
    pub account: Account,
    pub info: &'a AccountInfo<'info>,
}
```

#### Solana Program Validation
```rust
// programs/manifest/src/validation/solana_checkers.rs

pub struct Program<'a, 'info> {
    pub info: &'a AccountInfo<'info>,
}

pub struct TokenProgram<'a, 'info> {
    pub info: &'a AccountInfo<'info>,
}

pub struct Signer<'a, 'info> {
    pub info: &'a AccountInfo<'info>,
}

impl<'a, 'info> Signer<'a, 'info> {
    pub fn new(info: &'a AccountInfo<'info>) -> Result<Self, ProgramError>
    pub fn new_payer(info: &'a AccountInfo<'info>) -> Result<Self, ProgramError>
}

pub struct EmptyAccount<'a, 'info> {
    pub info: &'a AccountInfo<'info>,
}
```

### Account Creation Utilities

#### Generic Account Creation
```rust
// programs/manifest/src/utils.rs

/// Send CPI for creating a new account on chain
pub fn create_account<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    new_account: &'a AccountInfo<'info>,
    system_program: &'a AccountInfo<'info>,
    program_owner: &Pubkey,
    rent: &Rent,
    space: u64,
    seeds: Vec<Vec<u8>>,
) -> ProgramResult
```

#### Discriminant Generation
```rust
/// Canonical discriminant of the given struct
pub fn get_discriminant<T>() -> Result<u64, ProgramError> {
    let type_name: &str = std::any::type_name::<T>();
    let discriminant: u64 = u64::from_le_bytes(
        keccak::hashv(&[crate::ID.as_ref(), type_name.as_bytes()]).as_ref()[..8]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?,
    );
    Ok(discriminant)
}
```

### State Management Utilities

#### Slot Utilities
```rust
// programs/manifest/src/state/utils.rs

#[cfg(not(feature = "no-clock"))]
pub fn get_now_slot() -> u32 {
    Clock::get().unwrap().slot as u32
}

#[cfg(feature = "no-clock")]
pub fn get_now_slot() -> u32 {
    u32::MAX
}
```

#### Token Transfer Utilities
```rust
pub fn transfer_tokens<'a, 'info>(
    from: &'a AccountInfo<'info>,
    to: &'a AccountInfo<'info>,
    authority: &'a AccountInfo<'info>,
    token_program: &'a AccountInfo<'info>,
    mint: &'a AccountInfo<'info>,
    amount: u64,
    seeds: Option<&[&[&[u8]]]>,
) -> ProgramResult
```

### PDA Derivation Functions

#### Market Vault Addresses
```rust
// programs/manifest/src/validation/mod.rs

pub fn get_vault_address(market: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"vault", market.as_ref(), mint.as_ref()],
        &crate::ID,
    )
}
```

#### Global Account Addresses
```rust
pub fn get_global_address(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"global", mint.as_ref()],
        &crate::ID,
    )
}

pub fn get_global_vault_address(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"global-vault", mint.as_ref()],
        &crate::ID,
    )
}
```

## Client-Side Utilities (TypeScript)

### Price Conversion Utilities

#### Mantissa and Exponent Conversion
```typescript
// client/ts/src/client.ts

export function toMantissaAndExponent(input: number): {
  priceMantissa: number;
  priceExponent: number;
} {
  let priceExponent = 0;
  let priceMantissa = input;
  const uInt32Max = 4_294_967_296;
  
  while (priceExponent > -20 && priceMantissa < uInt32Max / 100) {
    priceExponent -= 1;
    priceMantissa *= 10;
  }
  priceMantissa = Math.floor(priceMantissa);

  return { priceMantissa, priceExponent };
}
```

#### Number Conversion Utilities
```typescript
// client/ts/src/utils/numbers.ts

/**
 * Converts a beet.bignum to a number
 */
export function toNum(n: bignum): number {
  let target: number;
  if (typeof n === 'number') {
    target = n;
  } else {
    target = n.toString() as any as number;
  }
  return target;
}

/**
 * Converts a beet.bignum to a number after dividing by 10**18
 */
export function convertU128(n: bignum): number {
  if (typeof n === 'number') {
    return n;
  }

  let mantissa = n.clone();
  for (let exponent = -18; exponent < 20; exponent += 1) {
    if (mantissa.lte(BN_NUMBER_MAX)) {
      return mantissa.toNumber() * 10 ** exponent;
    }
    mantissa = mantissa.div(BN_10);
  }
  throw 'unreachable';
}
```

### Account Address Utilities

#### Market Vault Addresses
```typescript
// client/ts/src/utils/market.ts

export function getVaultAddress(market: PublicKey, mint: PublicKey): PublicKey {
  const [vaultAddress, _unusedBump] = PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), market.toBuffer(), mint.toBuffer()],
    PROGRAM_ID,
  );
  return vaultAddress;
}
```

#### Global Account Addresses
```typescript
// client/ts/src/utils/global.ts

export function getGlobalAddress(mint: PublicKey): PublicKey {
  const [globalAddress, _unusedBump] = PublicKey.findProgramAddressSync(
    [Buffer.from('global'), mint.toBuffer()],
    PROGRAM_ID,
  );
  return globalAddress;
}

export function getGlobalVaultAddress(mint: PublicKey): PublicKey {
  const [globalVaultAddress, _unusedBump] = PublicKey.findProgramAddressSync(
    [Buffer.from('global-vault'), mint.toBuffer()],
    PROGRAM_ID,
  );
  return globalVaultAddress;
}
```

### Data Deserialization Utilities

#### Red-Black Tree Deserialization
```typescript
// client/ts/src/utils/redBlackTree.ts

/**
 * Deserializes a RedBlackTree from a given buffer into a list
 */
export function deserializeRedBlackTree<Value>(
  data: Buffer,
  rootIndex: number,
  valueDeserializer: BeetArgsStruct<Value>,
): Value[] {
  const result: Value[] = [];
  
  // Find the minimum node
  let currentHeader = rootHeaderValue;
  let currentIndex = rootIndex;
  while (toNum(currentHeader.left) != NIL) {
    currentIndex = toNum(currentHeader.left);
    const [currentHeaderTemp] = redBlackTreeHeaderBeet.deserialize(
      data.subarray(currentIndex, currentIndex + NUM_TREE_HEADER_BYTES),
    );
    currentHeader = currentHeaderTemp;
  }

  // Traverse in-order using successor function
  const [currentValue] = valueDeserializer.deserialize(
    data.subarray(
      currentIndex + NUM_TREE_HEADER_BYTES,
      currentIndex + NUM_TREE_HEADER_BYTES + valueDeserializer.byteSize,
    ),
  );
  result.push(currentValue);
  
  while (getSuccessorIndex(data, currentIndex) != NIL) {
    currentIndex = getSuccessorIndex(data, currentIndex);
    const [currentValue] = valueDeserializer.deserialize(/* ... */);
    result.push(currentValue);
  }

  return result;
}
```

#### Discriminant Generation
```typescript
// client/ts/src/utils/discriminator.ts

export function genAccDiscriminator(accName: string) {
  return keccak256(
    Buffer.concat([
      Buffer.from(bs58.decode(PROGRAM_ID.toBase58())),
      Buffer.from(accName),
    ]),
  ).subarray(0, 8);
}
```

### Beet Serialization Utilities

#### Custom Beet Structures
```typescript
// client/ts/src/utils/beet.ts

/**
 * PublicKey deserializer
 */
export const publicKeyBeet = new BeetArgsStruct<PubkeyWrapper>(
  [['publicKey', beetPublicKey]],
  'PubkeyWrapper',
);

/**
 * RedBlackTreeHeader deserializer
 */
export const redBlackTreeHeaderBeet = new BeetArgsStruct<RedBlackTreeNodeHeader>(
  [
    ['left', u32],
    ['right', u32],
    ['parent', u32],
    ['color', u32],
  ],
  'redBlackTreeNodeHeader',
);
```

### Solana Connection Utilities

#### Cluster Detection
```typescript
// client/ts/src/utils/solana.ts

export type Cluster = 'mainnet-beta' | 'devnet' | 'localnet';

export async function getClusterFromConnection(
  connection: Connection,
): Promise<Cluster> {
  const hash = await connection.getGenesisHash();
  if (hash === '5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d') {
    return 'mainnet-beta';
  } else if (hash === 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG') {
    return 'devnet';
  } else {
    return 'localnet';
  }
}
```

#### SOL Airdrop Utility
```typescript
export async function airdropSol(connection: Connection, recipient: PublicKey) {
  console.log(`Requesting airdrop for ${recipient}`);
  const signature = await connection.requestAirdrop(recipient, 2_000_000_000);
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    blockhash,
    lastValidBlockHeight,
    signature,
  });
}
```

## Hypertree Utilities

### Memory Access Helpers
```rust
// lib/src/utils.rs

pub type DataIndex = u32;

/// Marker trait to emit warnings when using get_helper on the Value type
pub trait Get: bytemuck::Pod {}

/// Read a struct of type T in an array of data at a given index
pub fn get_helper<T: Get>(data: &[u8], index: DataIndex) -> &T {
    let index_usize: usize = index as usize;
    bytemuck::from_bytes(&data[index_usize..index_usize + size_of::<T>()])
}

/// Read a mutable struct of type T in an array of data at a given index
pub fn get_mut_helper<T: Get>(data: &mut [u8], index: DataIndex) -> &mut T {
    let index_usize: usize = index as usize;
    bytemuck::from_bytes_mut(&mut data[index_usize..index_usize + size_of::<T>()])
}
```

## Testing Utilities

### Test Fixtures
```rust
// programs/manifest/tests/program_test/fixtures.rs

pub struct TestFixture {
    pub context: ProgramTestContext,
    pub payer: Keypair,
    pub second_keypair: Keypair,
    pub sol: MintFixture,
    pub usdc: MintFixture,
}

pub struct MintFixture {
    pub key: Pubkey,
    pub decimals: u8,
}

pub struct TokenAccountFixture {
    pub key: Pubkey,
    pub mint: Pubkey,
    pub owner: Pubkey,
}
```

### Test Helpers
```typescript
// client/ts/tests/utils.ts

export const areFloatsEqual = (
  num1: number,
  num2: number,
  epsilon: number = 1e-10,
): boolean => Math.abs(num1 - num2) < epsilon;

export const sleep = async (ms: number) =>
  new Promise((resolve) => setTimeout(resolve, ms));
```

## Constants and Configuration

### Core Constants
```rust
// programs/manifest/src/state/constants.rs

pub const MARKET_FIXED_SIZE: usize = 256;
pub const MARKET_BLOCK_SIZE: usize = 80;
pub const GLOBAL_FIXED_SIZE: usize = 96;
pub const GLOBAL_BLOCK_SIZE: usize = 64;
pub const NO_EXPIRATION_LAST_VALID_SLOT: u32 = 0;
pub const MAX_GLOBAL_SEATS: u16 = 32;
```

### Client Constants
```typescript
// client/ts/src/constants.ts

export const FIXED_MANIFEST_HEADER_SIZE = 256;
export const FIXED_GLOBAL_HEADER_SIZE = 96;
export const FIXED_WRAPPER_HEADER_SIZE = 64;
export const NIL = 4294967295; // u32::MAX
export const NO_EXPIRATION_LAST_VALID_SLOT = 0;
```

## Error Handling Utilities

### Custom Error Types
```rust
// programs/manifest/src/program/error.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u32)]
pub enum ManifestError {
    InvalidMarket = 0,
    InvalidTrader = 1,
    InsufficientFunds = 2,
    OrderNotFound = 3,
    // ... more error types
}
```

### Error Macros
```rust
// programs/manifest/src/require.rs

#[macro_export]
macro_rules! require {
    ($condition:expr, $error:expr, $message:expr) => {
        if !$condition {
            solana_program::msg!($message);
            return Err($error.into());
        }
    };
}
```

## Performance Utilities

### Compute Unit Optimization
```typescript
// Client-side compute unit helpers

import { ComputeBudgetProgram } from '@solana/web3.js';

// Set compute unit limit
const cuLimitIx = ComputeBudgetProgram.setComputeUnitLimit({ 
  units: 200_000 
});

// Set compute unit price
const cuPriceIx = ComputeBudgetProgram.setComputeUnitPrice({ 
  microLamports: 10_000 
});
```

### Batch Operation Helpers
```typescript
// Efficient batch operations
const batchIx = client.batchUpdateIx(
  placeOrders,    // Multiple orders
  cancelOrders,   // Multiple cancels
  false          // Don't cancel all
);
```

## Integration Patterns

### Common Usage Patterns
```typescript
// Market data reading pattern
const market = await Market.loadFromAddress({ connection, address: marketPk });
const bids = market.bidsL2();
const asks = market.asksL2();

// Order placement pattern
const client = await ManifestClient.getClientForMarket(connection, marketPk, trader);
const orderIx = client.placeOrderIx(orderParams);

// Global account pattern
const globalIx = ManifestClient.createGlobalIx(trader.publicKey, mint);
const depositIx = ManifestClient.globalDepositIx(trader.publicKey, mint, amount);
```

These utilities provide the foundation for all Manifest operations, ensuring type safety, proper validation, and efficient data handling across the entire system.
