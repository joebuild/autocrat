use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;

use crate::error::ErrorCode;
use crate::state::*;

#[derive(Accounts)]
pub struct CreateAmmPosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub hybrid_market: Account<'info, HybridMarket>,
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<AmmPosition>(),
        seeds = [
            hybrid_market.key().as_ref(),
            user.key().as_ref(),
        ],
        bump
    )]
    pub amm_position: Account<'info, AmmPosition>,
    /// CHECK:
    #[account(address = tx_instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateAmmPosition>) -> Result<()> {
    let CreateAmmPosition {
        user,
        hybrid_market,
        amm_position,
        instructions,
        system_program: _,
    } = ctx.accounts;

    if hybrid_market.permissioned {
        let ixns = instructions.to_account_info();
        let current_index = tx_instructions::load_current_index_checked(&ixns)? as usize;
        let current_ixn = tx_instructions::load_instruction_at_checked(current_index, &ixns)?;
        assert!(hybrid_market.permissioned_caller == current_ixn.program_id);
    }

    amm_position.user = user.key();
    amm_position.hybrid_market = hybrid_market.key();
    amm_position.ownership = 0;

    Ok(())
}
