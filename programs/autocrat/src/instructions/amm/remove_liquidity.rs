use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::*;
use num_traits::ToPrimitive;

use crate::error::ErrorCode;
use crate::state::*;
use crate::{utils::*, BPS_SCALE};
use crate::generate_vault_seeds;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
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
        has_one = amm,
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
    ctx: Context<RemoveLiquidity>,
    remove_bps: u64,
    is_pass_market: bool,
) -> Result<()> {
    let RemoveLiquidity {
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
        system_program: _
    } = ctx.accounts;

    assert!(amm_position.ownership > 0);
    assert!(remove_bps > 0);
    assert!(remove_bps <= BPS_SCALE);

    if is_pass_market {
        assert_eq!(
            proposal.pass_market_amm,
            amm.key()
        );
        assert_eq!(
            proposal.conditional_on_pass_meta_mint,
            conditional_base_mint.key()
        );
        assert_eq!(
            proposal.conditional_on_pass_usdc_mint,
            conditional_quote_mint.key()
        );
    } else {
        assert_eq!(
            proposal.fail_market_amm,
            amm.key()
        );
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
    let can_ltwap_be_updated = clock.slot < proposal.slot_enqueued.checked_add(dao.slots_per_proposal).unwrap();
    
    if can_ltwap_be_updated {
        amm.update_ltwap()?;
    }

    if can_ltwap_be_updated && user.key() == proposal.proposer {
        return err!(ErrorCode::ProposerCannotPullLiquidityWhileMarketIsPending);
    }

    if amm_position.ownership > 0 && remove_bps == BPS_SCALE {
        amm.num_current_lps = amm.num_current_lps.checked_sub(1).unwrap();
    }
    
    let base_to_withdraw = (amm.conditional_base_amount as u128)
        .checked_mul(amm_position.ownership as u128).unwrap()
        .checked_mul(remove_bps as u128).unwrap()
        .checked_div(BPS_SCALE as u128).unwrap()
        .checked_div(amm.total_ownership as u128).unwrap()
        .to_u64().unwrap();

    let quote_to_withdraw = (amm.conditional_quote_amount as u128)
        .checked_mul(amm_position.ownership as u128).unwrap()
        .checked_mul(remove_bps as u128).unwrap()
        .checked_div(BPS_SCALE as u128).unwrap()
        .checked_div(amm.total_ownership as u128).unwrap()
        .to_u64().unwrap();

    let less_ownership = (amm_position.ownership as u128)
        .checked_mul(remove_bps as u128).unwrap()
        .checked_div(BPS_SCALE as u128).unwrap()
        .to_u64().unwrap();

    amm_position.ownership = if remove_bps == BPS_SCALE { 0 } else { amm_position.ownership.checked_sub(less_ownership).unwrap() };
    amm.total_ownership = amm.total_ownership.checked_sub(less_ownership).unwrap();

    let proposal_number_bytes = proposal.number.to_le_bytes();
    let seeds = generate_vault_seeds!(proposal_number_bytes, ctx.bumps.proposal);

    // send vault base tokens to user
    token_transfer_signed(
        base_to_withdraw,
        token_program,
        vault_ata_conditional_base,
        user_ata_conditional_base,
        proposal,
        seeds
    )?;

    // send vault quote tokens to user
    token_transfer_signed(
        quote_to_withdraw,
        token_program,
        vault_ata_conditional_quote,
        user_ata_conditional_quote,
        proposal,
        seeds
    )?;

    Ok(())
}
