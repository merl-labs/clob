use crate::{
    logs::{emit_stack, UndelegateMarketLog},
    validation::loaders::UndelegateMarketContext,
};
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

pub(crate) fn process_undelegate_market(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    let undelegate_context: UndelegateMarketContext = UndelegateMarketContext::load(accounts)?;

    // Validate vault addresses match the market's expected vaults
    let market_data = undelegate_context.market.get_fixed()?;
    if *undelegate_context.base_vault.key != *market_data.get_base_vault() {
        return Err(ProgramError::InvalidAccountData);
    }
    if *undelegate_context.quote_vault.key != *market_data.get_quote_vault() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Commit and undelegate all accounts (market and both vaults)
    commit_and_undelegate_accounts(
        undelegate_context.payer.as_ref(),
        vec![
            undelegate_context.market.as_ref(),
            undelegate_context.base_vault,
            undelegate_context.quote_vault,
        ],
        undelegate_context.magic_context,
        undelegate_context.magic_program.as_ref(),
    )?;

    emit_stack(UndelegateMarketLog {
        market: *undelegate_context.market.key,
    })?;

    Ok(())
}
