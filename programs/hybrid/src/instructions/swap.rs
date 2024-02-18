use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::*;
use num_traits::ToPrimitive;

use crate::generate_vault_seeds;
use crate::state::*;
use crate::{utils::*, BPS_SCALE};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        has_one = base_mint,
        has_one = quote_mint,
    )]
    pub hybrid_market: Account<'info, HybridMarket>,
    pub base_mint: Account<'info, Mint>,
    pub quote_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = base_mint,
        associated_token::authority = user,
    )]
    pub user_ata_base: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = quote_mint,
        associated_token::authority = user,
    )]
    pub user_ata_quote: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = base_mint,
        associated_token::authority = hybrid_market,
    )]
    pub vault_ata_base: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = quote_mint,
        associated_token::authority = hybrid_market,
    )]
    pub vault_ata_quote: Account<'info, TokenAccount>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    /// CHECK:
    #[account(address = tx_instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Swap>,
    is_quote_to_base: bool,
    input_amount: u64,
    output_amount_min: u64,
) -> Result<()> {
    let Swap {
        user,
        hybrid_market,
        base_mint,
        quote_mint,
        user_ata_base,
        user_ata_quote,
        vault_ata_base,
        vault_ata_quote,
        associated_token_program: _,
        token_program,
        instructions,
        system_program: _,
    } = ctx.accounts;

    assert!(input_amount > 0);
    assert!(hybrid_market.total_ownership > 0);

    if hybrid_market.permissioned {
        let ixns = instructions.to_account_info();
        let current_index = tx_instructions::load_current_index_checked(&ixns)? as usize;
        let current_ixn = tx_instructions::load_instruction_at_checked(current_index, &ixns)?;
        assert!(hybrid_market.permissioned_caller == current_ixn.program_id);
    }

    hybrid_market.update_ltwap()?;

    let base_amount_start = hybrid_market.base_amount as u128;
    let quote_amount_start = hybrid_market.quote_amount as u128;

    let k = base_amount_start.checked_mul(quote_amount_start).unwrap();

    let input_amount_minus_fee = input_amount
        .checked_mul(
            BPS_SCALE
                .checked_sub(hybrid_market.swap_fee_bps as u64)
                .unwrap(),
        )
        .unwrap()
        .checked_div(BPS_SCALE)
        .unwrap() as u128;

    let base_mint_key = base_mint.key();
    let quote_mint_key = quote_mint.key();
    let swap_fee_bps_bytes = hybrid_market.swap_fee_bps.to_le_bytes();
    let permissioned_caller = hybrid_market.permissioned_caller;

    let seeds = generate_vault_seeds!(
        base_mint_key,
        quote_mint_key,
        swap_fee_bps_bytes,
        permissioned_caller,
        hybrid_market.bump
    );

    let output_amount = if is_quote_to_base {
        let temp_quote_amount = quote_amount_start
            .checked_add(input_amount_minus_fee)
            .unwrap();
        let temp_base_amount = k.checked_div(temp_quote_amount).unwrap();

        let output_amount_base = base_amount_start
            .checked_sub(temp_base_amount)
            .unwrap()
            .to_u64()
            .unwrap();

        hybrid_market.quote_amount = hybrid_market
            .quote_amount
            .checked_add(input_amount)
            .unwrap();
        hybrid_market.base_amount = hybrid_market
            .base_amount
            .checked_sub(output_amount_base)
            .unwrap();

        // send user quote tokens to vault
        token_transfer(
            input_amount,
            token_program,
            user_ata_quote,
            vault_ata_quote,
            &user,
        )?;

        // send vault base tokens to user
        token_transfer_signed(
            output_amount_base,
            token_program,
            vault_ata_base,
            user_ata_base,
            hybrid_market,
            seeds,
        )?;

        output_amount_base
    } else {
        let temp_base_amount = base_amount_start
            .checked_add(input_amount_minus_fee)
            .unwrap();
        let temp_quote_amount = k.checked_div(temp_base_amount).unwrap();

        let output_amount_quote = quote_amount_start
            .checked_sub(temp_quote_amount)
            .unwrap()
            .to_u64()
            .unwrap();

        hybrid_market.base_amount = hybrid_market.base_amount.checked_add(input_amount).unwrap();
        hybrid_market.quote_amount = hybrid_market
            .quote_amount
            .checked_sub(output_amount_quote)
            .unwrap();

        // send user base tokens to vault
        token_transfer(
            input_amount,
            token_program,
            &user_ata_base,
            &vault_ata_base,
            &user,
        )?;

        // send vault quote tokens to user
        token_transfer_signed(
            output_amount_quote,
            token_program,
            vault_ata_quote,
            user_ata_quote,
            hybrid_market,
            seeds,
        )?;

        output_amount_quote
    };

    let new_k = (hybrid_market.base_amount as u128)
        .checked_mul(hybrid_market.quote_amount as u128)
        .unwrap();

    assert!(new_k >= k); // with non-zero fees, k should always increase

    assert!(output_amount >= output_amount_min);

    Ok(())
}
