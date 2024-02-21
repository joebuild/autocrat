use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;

use crate::error::ErrorCode;
use crate::state::*;

#[derive(Accounts)]
pub struct CreatePosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub amm: Account<'info, Amm>,
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<AmmPosition>(),
        seeds = [
            amm.key().as_ref(),
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

pub fn handler(ctx: Context<CreatePosition>) -> Result<()> {
    let CreatePosition {
        user,
        amm,
        amm_position,
        instructions,
        system_program: _,
    } = ctx.accounts;

    if amm.permissioned {
        let ixns = instructions.to_account_info();
        let current_index = tx_instructions::load_current_index_checked(&ixns)? as usize;
        let current_ixn = tx_instructions::load_instruction_at_checked(current_index, &ixns)?;
        assert!(amm.permissioned_caller == current_ixn.program_id);
    }

    amm_position.user = user.key();
    amm_position.amm = amm.key();
    amm_position.ownership = 0;

    Ok(())
}
