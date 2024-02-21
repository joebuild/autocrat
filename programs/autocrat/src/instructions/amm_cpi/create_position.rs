use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use solana_program::native_token::LAMPORTS_PER_SOL;

use amm::cpi::accounts::CreateAmm;
use amm::cpi::accounts::CreatePosition as AmmCreatePosition;
use amm::instructions::create_amm::CreateAmmParams;
use amm::program::Amm;

use crate::error::ErrorCode;
use crate::generate_vault_seeds;
use crate::program::Autocrat;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct CreatePosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        mut,
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    /// CHECK:
    pub amm: UncheckedAccount<'info>,
    /// CHECK
    pub amm_position: UncheckedAccount<'info>,
    #[account(address = amm::ID)]
    pub amm_program: Program<'info, Amm>,
    #[account(address = tx_instructions::ID)]
    /// CHECK:
    pub instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreatePosition>) -> Result<()> {
    let CreatePosition {
        user,
        proposal,
        dao,
        amm,
        amm_position,
        amm_program: _,
        instructions: _,
        system_program: _,
    } = ctx.accounts;

    assert!(proposal.pass_market_amm == amm.key() || proposal.fail_market_amm == amm.key());
    assert_eq!(proposal.state, ProposalState::Pending);

    // create proposer LP position
    let create_amm_position_ctx = ctx.accounts.into_create_amm_position_context();
    amm::cpi::create_position(create_amm_position_ctx)?;

    Ok(())
}

impl<'info> CreatePosition<'info> {
    fn into_create_amm_position_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, AmmCreatePosition<'info>> {
        let cpi_accounts = AmmCreatePosition {
            user: self.user.to_account_info(),
            amm: self.amm.to_account_info(),
            amm_position: self.amm_position.to_account_info(),
            instructions: self.instructions.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
