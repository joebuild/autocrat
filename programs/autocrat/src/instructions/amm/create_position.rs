use std::borrow::BorrowMut;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::*;
use num_traits::ToPrimitive;

use crate::error::ErrorCode;
use crate::state::*;
use crate::{utils::*, BPS_SCALE};
use crate::generate_vault_seeds;

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
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreatePosition>,
) -> Result<()> {
    let CreatePosition {
        user,
        amm,
        amm_position,
        system_program: _
    } = ctx.accounts;

    amm_position.user = user.key();
    amm_position.amm = amm.key();
    amm_position.ownership = 0;

    Ok(())
}
