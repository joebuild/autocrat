use anchor_lang::prelude::*;

use crate::program::Autocrat;
use amm::cpi::accounts::CreatePosition as AmmCreatePosition;
use amm::program::Amm;

use crate::error::ErrorCode;
use crate::state::*;

#[derive(Accounts)]
pub struct CreatePosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub proposal: Box<Account<'info, Proposal>>,
    /// CHECK:
    pub amm: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK
    pub amm_position: UncheckedAccount<'info>,
    /// CHECK
    pub amm_auth_pda: UncheckedAccount<'info>,
    #[account(address = amm::ID)]
    pub amm_program: Program<'info, Amm>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreatePosition>) -> Result<()> {
    require!(
        ctx.accounts.proposal.pass_market_amm == ctx.accounts.amm.key()
            || ctx.accounts.proposal.fail_market_amm == ctx.accounts.amm.key(),
        ErrorCode::AmmProposalMismatch
    );

    require!(
        ctx.accounts.proposal.state == ProposalState::Pending,
        ErrorCode::ProposalIsNoLongerPending
    );

    let clock = Clock::get()?;
    assert!(
        clock.slot < ctx.accounts.proposal.slot_enqueued + ctx.accounts.proposal.slots_duration
    );

    // create proposer LP position
    let (_auth_pda, auth_pda_bump) =
        Pubkey::find_program_address(&[AMM_AUTH_SEED_PREFIX], &Autocrat::id());
    let seeds = &[AMM_AUTH_SEED_PREFIX, &[auth_pda_bump]];
    let signer = [&seeds[..]];

    let create_amm_position_ctx = ctx.accounts.into_create_amm_position_context(&signer);
    amm::cpi::create_position(create_amm_position_ctx)?;

    Ok(())
}

impl<'info> CreatePosition<'info> {
    fn into_create_amm_position_context<'a, 'b, 'c>(
        &'a self,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'_, '_, '_, 'info, AmmCreatePosition<'info>> {
        let cpi_accounts = AmmCreatePosition {
            user: self.user.to_account_info(),
            amm: self.amm.to_account_info(),
            amm_position: self.amm_position.to_account_info(),
            system_program: self.system_program.to_account_info(),
            auth_pda: Some(self.amm_auth_pda.to_account_info()),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
    }
}
