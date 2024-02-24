use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;

use crate::state::*;

#[derive(Accounts)]
pub struct UpdateLtwap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub amm: Account<'info, Amm>,
    /// CHECK:
    #[account(address = tx_instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<UpdateLtwap>) -> Result<()> {
    let UpdateLtwap {
        user: _,
        amm,
        instructions,
        system_program: _,
    } = ctx.accounts;

    if amm.permissioned {
        let ixns = instructions.to_account_info();
        let current_index = tx_instructions::load_current_index_checked(&ixns)? as usize;
        let current_ixn = tx_instructions::load_instruction_at_checked(current_index, &ixns)?;
        assert!(amm.permissioned_caller == current_ixn.program_id);
    }

    amm.update_ltwap()?;

    Ok(())
}
