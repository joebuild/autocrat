use std::borrow::BorrowMut;

use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::*;
use num_traits::ToPrimitive;

use crate::error::ErrorCode;
use crate::generate_vault_seeds;
use crate::state::*;
use crate::{utils::*, BPS_SCALE};

#[derive(Accounts)]
#[instruction(create_amm_params: CreateAmmParams)]
pub struct CreateAmm<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<Amm>(),
        seeds = [
            base_mint.key().as_ref(),
            quote_mint.key().as_ref(),
            create_amm_params.swap_fee_bps.to_le_bytes().as_ref(),
            create_amm_params.permissioned_caller.unwrap_or(Pubkey::default()).as_ref()
        ],
        bump
    )]
    pub amm: Account<'info, Amm>,
    pub base_mint: Account<'info, Mint>,
    pub quote_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = user,
        associated_token::authority = amm,
        associated_token::mint = base_mint
    )]
    pub vault_ata_base: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = user,
        associated_token::authority = amm,
        associated_token::mint = quote_mint
    )]
    pub vault_ata_quote: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct CreateAmmParams {
    pub permissioned: Option<bool>,
    pub permissioned_caller: Option<Pubkey>,
    pub swap_fee_bps: u64,
}

pub fn handler(ctx: Context<CreateAmm>, create_amm_params: CreateAmmParams) -> Result<()> {
    let CreateAmm {
        user,
        amm,
        base_mint,
        quote_mint,
        vault_ata_base,
        vault_ata_quote,
        associated_token_program: _,
        token_program,
        system_program: _,
    } = ctx.accounts;

    amm.permissioned = create_amm_params.permissioned.unwrap_or(false);
    if amm.permissioned {
        amm.permissioned_caller = create_amm_params.permissioned_caller.unwrap();
    }

    amm.created_at_slot = Clock::get()?.slot;

    assert!(create_amm_params.swap_fee_bps < BPS_SCALE);

    amm.swap_fee_bps = create_amm_params.swap_fee_bps;

    assert_ne!(base_mint.key(), quote_mint.key());

    amm.base_mint = base_mint.key();
    amm.quote_mint = quote_mint.key();

    Ok(())
}
