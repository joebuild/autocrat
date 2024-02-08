use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::*;
use num_traits::ToPrimitive;

use crate::state::*;
use crate::{utils::*, BPS_SCALE};
use crate::generate_vault_seeds;

#[derive(Accounts)]
pub struct Swap<'info> {
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
    ctx: Context<Swap>,
    is_quote_to_base: bool,
    input_amount: u64,
    output_amount_min: u64,
    is_pass_market: bool,
) -> Result<()> {
    let Swap {
        user,
        dao,
        proposal,
        amm,
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

    assert!(input_amount > 0);

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

    let conditional_base_amount_start = amm.conditional_base_amount as u128;
    let conditional_quote_amount_start = amm.conditional_quote_amount as u128;

    let k = conditional_base_amount_start.checked_mul(conditional_quote_amount_start).unwrap();

    let input_amount_minus_fee = input_amount
            .checked_mul(BPS_SCALE.checked_sub(dao.amm_swap_fee_bps).unwrap()).unwrap()
            .checked_div(BPS_SCALE).unwrap() as u128;
    
    let proposal_number_bytes = proposal.number.to_le_bytes();
    let seeds = generate_vault_seeds!(proposal_number_bytes, ctx.bumps.proposal);

    let output_amount = if is_quote_to_base {
        let temp_conditional_quote_amount = conditional_quote_amount_start.checked_add(input_amount_minus_fee).unwrap();
        let temp_conditional_base_amount = k.checked_div(temp_conditional_quote_amount).unwrap();
        
        let output_amount_base = conditional_base_amount_start
            .checked_sub(temp_conditional_base_amount).unwrap()
            .to_u64().unwrap();

        amm.conditional_quote_amount = amm.conditional_quote_amount.checked_add(input_amount).unwrap();
        amm.conditional_base_amount = amm.conditional_base_amount.checked_sub(output_amount_base).unwrap();

        // send user quote tokens to vault
        token_transfer(
            input_amount,
            token_program,
            user_ata_conditional_quote,
            vault_ata_conditional_quote,
            &user,
        )?;

        // send vault base tokens to user
        token_transfer_signed(
            output_amount_base,
            token_program,
            vault_ata_conditional_base,
            user_ata_conditional_base,
            proposal,
            seeds
        )?;

        output_amount_base
    } else {
        let temp_conditional_base_amount = conditional_base_amount_start.checked_add(input_amount_minus_fee).unwrap();
        let temp_conditional_quote_amount = k.checked_div(temp_conditional_base_amount).unwrap();
        
        let output_amount_quote = conditional_quote_amount_start
            .checked_sub(temp_conditional_quote_amount).unwrap()
            .to_u64().unwrap();

        amm.conditional_base_amount = amm.conditional_base_amount.checked_add(input_amount).unwrap();
        amm.conditional_quote_amount = amm.conditional_quote_amount.checked_sub(output_amount_quote).unwrap();

        // send user base tokens to vault
        token_transfer(
            input_amount,
            token_program,
            &user_ata_conditional_base,
            &vault_ata_conditional_base,
            &user,
        )?;

        // send vault quote tokens to user
        token_transfer_signed(
            output_amount_quote,
            token_program,
            vault_ata_conditional_quote,
            user_ata_conditional_quote,
            proposal,
            seeds
        )?;

        output_amount_quote
    };

    // TODO: add invariant checks

    Ok(())
}
