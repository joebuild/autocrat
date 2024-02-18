use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::*;
use num_traits::ToPrimitive;

use crate::error::ErrorCode;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct AddAmmLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        has_one = base_mint,
        has_one = quote_mint,
    )]
    pub hybrid_market: Account<'info, HybridMarket>,
    #[account(
        mut,
        has_one = user,
        has_one = hybrid_market,
        seeds = [
            hybrid_market.key().as_ref(),
            user.key().as_ref(),
        ],
        bump
    )]
    pub amm_position: Account<'info, AmmPosition>,
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
    ctx: Context<AddAmmLiquidity>,
    max_base_amount: u64,
    max_quote_amount: u64,
) -> Result<()> {
    let AddAmmLiquidity {
        user,
        hybrid_market,
        amm_position,
        base_mint: _,
        quote_mint: _,
        user_ata_base,
        user_ata_quote,
        vault_ata_base,
        vault_ata_quote,
        associated_token_program: _,
        token_program,
        instructions,
        system_program: _,
    } = ctx.accounts;

    assert!(max_base_amount > 0);
    assert!(max_quote_amount > 0);

    if hybrid_market.permissioned {
        let ixns = instructions.to_account_info();
        let current_index = tx_instructions::load_current_index_checked(&ixns)? as usize;
        let current_ixn = tx_instructions::load_instruction_at_checked(current_index, &ixns)?;
        assert!(hybrid_market.permissioned_caller == current_ixn.program_id);
    }

    hybrid_market.update_ltwap()?;

    let mut temp_base_amount: u128;
    let mut temp_quote_amount: u128;

    // if there is no liquidity in the amm, then initialize with new ownership values
    if hybrid_market.base_amount == 0 && hybrid_market.quote_amount == 0 {
        temp_base_amount = max_base_amount as u128;
        temp_quote_amount = max_quote_amount as u128;

        // use the higher number for ownership, to reduce rounding errors
        let max_base_or_quote_amount = std::cmp::max(temp_base_amount, temp_quote_amount);

        amm_position.ownership = max_base_or_quote_amount.to_u64().unwrap();
        hybrid_market.total_ownership = max_base_or_quote_amount.to_u64().unwrap();
    } else {
        temp_base_amount = max_base_amount as u128;

        temp_quote_amount = temp_base_amount
            .checked_mul(hybrid_market.quote_amount as u128)
            .unwrap()
            .checked_div(hybrid_market.base_amount as u128)
            .unwrap();

        // if the temp_quote_amount calculation with max_base_amount led to a value higher than max_quote_amount,
        // then use the max_quote_amount and calculate in the other direction
        if temp_quote_amount > max_quote_amount as u128 {
            temp_quote_amount = max_quote_amount as u128;

            temp_base_amount = temp_quote_amount
                .checked_mul(hybrid_market.base_amount as u128)
                .unwrap()
                .checked_div(hybrid_market.quote_amount as u128)
                .unwrap();

            if temp_base_amount > max_base_amount as u128 {
                return err!(ErrorCode::AddLiquidityCalculationError);
            }
        }

        let additional_ownership = temp_base_amount
            .checked_mul(hybrid_market.total_ownership as u128)
            .unwrap()
            .checked_div(hybrid_market.base_amount as u128)
            .unwrap()
            .to_u64()
            .unwrap();

        amm_position.ownership = amm_position
            .ownership
            .checked_add(additional_ownership)
            .unwrap();
        hybrid_market.total_ownership = hybrid_market
            .total_ownership
            .checked_add(additional_ownership)
            .unwrap();
    }

    hybrid_market.base_amount = hybrid_market
        .base_amount
        .checked_add(temp_base_amount.to_u64().unwrap())
        .unwrap();

    hybrid_market.quote_amount = hybrid_market
        .quote_amount
        .checked_add(temp_quote_amount.to_u64().unwrap())
        .unwrap();

    // send user base tokens to vault
    token_transfer(
        temp_base_amount as u64,
        &token_program,
        user_ata_base,
        vault_ata_base,
        user,
    )?;

    // send user quote tokens to vault
    token_transfer(
        temp_quote_amount as u64,
        token_program,
        user_ata_quote,
        vault_ata_quote,
        user,
    )?;

    Ok(())
}
