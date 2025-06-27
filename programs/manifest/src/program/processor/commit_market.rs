use crate::{
    logs::{emit_stack, CommitMarketLog},
    validation::loaders::CommitMarketContext,
};
use ephemeral_rollups_sdk::ephem::commit_accounts;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub(crate) fn process_commit_market(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    let commit_context: CommitMarketContext = CommitMarketContext::load(accounts)?;

    // Validate vault addresses match the market's expected vaults
    let market_data = commit_context.market.get_fixed()?;
    if *commit_context.base_vault.key != *market_data.get_base_vault() {
        return Err(solana_program::program_error::ProgramError::InvalidAccountData);
    }
    if *commit_context.quote_vault.key != *market_data.get_quote_vault() {
        return Err(solana_program::program_error::ProgramError::InvalidAccountData);
    }

    // Commit all accounts (market and both vaults) state to base layer without undelegating
    commit_accounts(
        commit_context.payer.as_ref(),
        vec![
            commit_context.market.as_ref(),
            commit_context.base_vault,
            commit_context.quote_vault,
        ],
        commit_context.magic_context,
        commit_context.magic_program.as_ref(),
    )?;

    emit_stack(CommitMarketLog {
        market: *commit_context.market.key,
    })?;

    Ok(())
}
