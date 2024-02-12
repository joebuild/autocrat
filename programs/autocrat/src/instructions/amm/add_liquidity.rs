use std::borrow::BorrowMut;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::*;
use num_traits::ToPrimitive;

use crate::error::ErrorCode;
use crate::generate_vault_seeds;
use crate::state::*;
use crate::{utils::*, BPS_SCALE};

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Account<'info, Dao>,
    #[account(
        seeds = [
            b"proposal",
            proposal.number.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        has_one = conditional_base_mint,
        has_one = conditional_quote_mint,
    )]
    pub amm: Account<'info, Amm>,
    #[account(
        mut,
        has_one = user,
        has_one = amm,
        seeds = [
            amm.key().as_ref(),
            user.key().as_ref(),
        ],
        bump
    )]
    pub amm_position: Account<'info, AmmPosition>,
    pub conditional_base_mint: Account<'info, Mint>,
    pub conditional_quote_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = conditional_base_mint,
        associated_token::authority = user,
    )]
    pub user_ata_conditional_base: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = conditional_quote_mint,
        associated_token::authority = user,
    )]
    pub user_ata_conditional_quote: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = conditional_base_mint,
        associated_token::authority = proposal,
    )]
    pub vault_ata_conditional_base: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = conditional_quote_mint,
        associated_token::authority = proposal,
    )]
    pub vault_ata_conditional_quote: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddLiquidity>,
    max_base_amount: u64,
    max_quote_amount: u64,
    is_pass_market: bool,
) -> Result<()> {
    let AddLiquidity {
        user,
        dao,
        proposal,
        amm,
        amm_position,
        conditional_base_mint,
        conditional_quote_mint,
        user_ata_conditional_base,
        user_ata_conditional_quote,
        vault_ata_conditional_base,
        vault_ata_conditional_quote,
        token_program,
        associated_token_program: _,
        system_program: _,
    } = ctx.accounts;

    assert!(max_base_amount > 0);
    assert!(max_quote_amount > 0);

    if is_pass_market {
        assert_eq!(proposal.pass_market_amm, amm.key());
        assert_eq!(
            proposal.conditional_on_pass_meta_mint,
            conditional_base_mint.key()
        );
        assert_eq!(
            proposal.conditional_on_pass_usdc_mint,
            conditional_quote_mint.key()
        );
    } else {
        assert_eq!(proposal.fail_market_amm, amm.key());
        assert_eq!(
            proposal.conditional_on_fail_meta_mint,
            conditional_base_mint.key()
        );
        assert_eq!(
            proposal.conditional_on_fail_usdc_mint,
            conditional_quote_mint.key()
        );
    }

    let clock = Clock::get()?;
    let can_ltwap_be_updated = clock.slot
        < proposal
            .slot_enqueued
            .checked_add(dao.slots_per_proposal)
            .unwrap();

    if can_ltwap_be_updated {
        amm.update_ltwap()?;
    }

    if amm_position.ownership == 0u64 {
        amm.num_current_lps = amm.num_current_lps.checked_add(1).unwrap();
    }

    let mut temp_base_amount = 0u128;
    let mut temp_quote_amount = 0u128;

    // if there is no liquidity in the amm, then initialize with new ownership values
    if amm.conditional_base_amount == 0 && amm.conditional_quote_amount == 0 {
        temp_base_amount = max_base_amount as u128;
        temp_quote_amount = max_quote_amount as u128;

        // use the higher number for ownership, to reduce rounding errors
        let max_base_or_quote_amount = std::cmp::max(temp_base_amount, temp_quote_amount);

        amm_position.ownership = max_base_or_quote_amount.to_u64().unwrap();
        amm.total_ownership = max_base_or_quote_amount.to_u64().unwrap();
    } else {
        temp_base_amount = max_base_amount as u128;

        temp_quote_amount = temp_base_amount
            .checked_mul(amm.conditional_quote_amount as u128)
            .unwrap()
            .checked_div(amm.conditional_base_amount as u128)
            .unwrap();

        // if the temp_quote_amount calculation with max_base_amount led to a value higher than max_quote_amount,
        // then use the max_quote_amount and calculate in the other direction
        if temp_quote_amount > max_quote_amount as u128 {
            temp_quote_amount = max_quote_amount as u128;

            temp_base_amount = temp_quote_amount
                .checked_mul(amm.conditional_base_amount as u128)
                .unwrap()
                .checked_div(amm.conditional_quote_amount as u128)
                .unwrap();

            if temp_base_amount > max_base_amount as u128 {
                return err!(ErrorCode::AddLiquidityCalculationError);
            }
        }

        let additional_ownership = temp_base_amount
            .checked_mul(amm.total_ownership as u128)
            .unwrap()
            .checked_div(amm.conditional_base_amount as u128)
            .unwrap()
            .to_u64()
            .unwrap();

        amm_position.ownership = amm_position
            .ownership
            .checked_add(additional_ownership)
            .unwrap();
        amm.total_ownership = amm
            .total_ownership
            .checked_add(additional_ownership)
            .unwrap();
    }

    amm.conditional_base_amount = amm
        .conditional_base_amount
        .checked_add(temp_base_amount.to_u64().unwrap())
        .unwrap();

    amm.conditional_quote_amount = amm
        .conditional_quote_amount
        .checked_add(temp_quote_amount.to_u64().unwrap())
        .unwrap();

    // send user base tokens to vault
    token_transfer(
        temp_base_amount as u64,
        &token_program,
        user_ata_conditional_base,
        vault_ata_conditional_base,
        user,
    )?;

    // send user quote tokens to vault
    token_transfer(
        temp_quote_amount as u64,
        token_program,
        user_ata_conditional_quote,
        vault_ata_conditional_quote,
        user,
    )?;

    Ok(())
}
