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
    pub amm: Account<'info, Amm>,
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
        associated_token::authority = amm,
    )]
    pub vault_ata_base: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = quote_mint,
        associated_token::authority = amm,
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
        amm,
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
    assert!(amm.total_ownership > 0);

    if amm.permissioned {
        let ixns = instructions.to_account_info();
        let current_index = tx_instructions::load_current_index_checked(&ixns)? as usize;
        let current_ixn = tx_instructions::load_instruction_at_checked(current_index, &ixns)?;
        assert!(amm.permissioned_caller == current_ixn.program_id);
    }

    amm.update_ltwap()?;

    let base_mint_key = base_mint.key();
    let quote_mint_key = quote_mint.key();
    let swap_fee_bps_bytes = amm.swap_fee_bps.to_le_bytes();
    let permissioned_caller = amm.permissioned_caller;

    let seeds = generate_vault_seeds!(
        base_mint_key,
        quote_mint_key,
        swap_fee_bps_bytes,
        permissioned_caller,
        amm.bump
    );

    let output_amount = amm.swap(input_amount, is_quote_to_base)?;

    if is_quote_to_base {
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
            output_amount,
            token_program,
            vault_ata_base,
            user_ata_base,
            amm,
            seeds,
        )?;
    } else {
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
            output_amount,
            token_program,
            vault_ata_quote,
            user_ata_quote,
            amm,
            seeds,
        )?;
    }
    assert!(output_amount >= output_amount_min);

    Ok(())
}
