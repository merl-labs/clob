use crate::{
    logs::{emit_stack, DelegateMarketLog},
    validation::loaders::DelegateMarketContext,
};
use borsh::{BorshDeserialize, BorshSerialize};
use ephemeral_rollups_sdk::cpi::{delegate_account, DelegateAccounts, DelegateConfig};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelegateMarketParams {
    /// How often to sync state with base layer (in milliseconds)
    pub update_frequency_ms: u32,
    /// Time limit for delegation (0 = no limit)
    pub time_limit: u64,
}

pub(crate) fn process_delegate_market(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let params: DelegateMarketParams = DelegateMarketParams::try_from_slice(data)?;
    let delegate_context: DelegateMarketContext = DelegateMarketContext::load(accounts)?;

    // Validate vault addresses match the market's expected vaults
    let market_data = delegate_context.market.get_fixed()?;
    if *delegate_context.base_vault.key != *market_data.get_base_vault() {
        return Err(solana_program::program_error::ProgramError::InvalidAccountData);
    }
    if *delegate_context.quote_vault.key != *market_data.get_quote_vault() {
        return Err(solana_program::program_error::ProgramError::InvalidAccountData);
    }

    // 1. Delegate the market account (not a PDA)
    let market_delegate_accounts = DelegateAccounts {
        payer: delegate_context.payer.as_ref(),
        pda: delegate_context.market.as_ref(),
        owner_program: delegate_context.owner_program.as_ref(),
        buffer: delegate_context.market_delegation_buffer,
        delegation_record: delegate_context.market_delegation_record,
        delegation_metadata: delegate_context.market_delegation_metadata,
        delegation_program: delegate_context.delegation_program.as_ref(),
        system_program: delegate_context.system_program.as_ref(),
    };
    let market_pda_seeds: &[&[u8]] = &[]; // Market accounts don't use seeds
    let market_delegate_config = DelegateConfig {
        commit_frequency_ms: params.update_frequency_ms,
        validator: None, // Use default validator
    };
    delegate_account(
        market_delegate_accounts,
        market_pda_seeds,
        market_delegate_config,
    )?;

    // 2. Delegate the base vault (PDA with seeds)
    let base_vault_delegate_accounts = DelegateAccounts {
        payer: delegate_context.payer.as_ref(),
        pda: delegate_context.base_vault,
        owner_program: delegate_context.owner_program.as_ref(),
        buffer: delegate_context.base_vault_delegation_buffer,
        delegation_record: delegate_context.base_vault_delegation_record,
        delegation_metadata: delegate_context.base_vault_delegation_metadata,
        delegation_program: delegate_context.delegation_program.as_ref(),
        system_program: delegate_context.system_program.as_ref(),
    };
    let base_vault_bump = market_data.get_base_vault_bump();
    let base_vault_seeds_array = [
        b"vault".as_ref(),
        delegate_context.market.key.as_ref(),
        delegate_context.base_mint.info.key.as_ref(),
        &[base_vault_bump],
    ];
    let base_vault_seeds: &[&[u8]] = &base_vault_seeds_array;
    let base_vault_delegate_config = DelegateConfig {
        commit_frequency_ms: params.update_frequency_ms,
        validator: None, // Use default validator
    };
    delegate_account(
        base_vault_delegate_accounts,
        base_vault_seeds,
        base_vault_delegate_config,
    )?;

    // 3. Delegate the quote vault (PDA with seeds)
    let quote_vault_delegate_accounts = DelegateAccounts {
        payer: delegate_context.payer.as_ref(),
        pda: delegate_context.quote_vault,
        owner_program: delegate_context.owner_program.as_ref(),
        buffer: delegate_context.quote_vault_delegation_buffer,
        delegation_record: delegate_context.quote_vault_delegation_record,
        delegation_metadata: delegate_context.quote_vault_delegation_metadata,
        delegation_program: delegate_context.delegation_program.as_ref(),
        system_program: delegate_context.system_program.as_ref(),
    };
    let quote_vault_bump = market_data.get_quote_vault_bump();
    let quote_vault_seeds_array = [
        b"vault".as_ref(),
        delegate_context.market.key.as_ref(),
        delegate_context.quote_mint.info.key.as_ref(),
        &[quote_vault_bump],
    ];
    let quote_vault_seeds: &[&[u8]] = &quote_vault_seeds_array;
    let quote_vault_delegate_config = DelegateConfig {
        commit_frequency_ms: params.update_frequency_ms,
        validator: None, // Use default validator
    };
    delegate_account(
        quote_vault_delegate_accounts,
        quote_vault_seeds,
        quote_vault_delegate_config,
    )?;

    emit_stack(DelegateMarketLog {
        market: *delegate_context.market.key,
        update_frequency_ms: params.update_frequency_ms as u64,
        time_limit: params.time_limit,
    })?;

    Ok(())
}
