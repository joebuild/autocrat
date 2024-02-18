use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::*;
use num_traits::ToPrimitive;
use sokoban::OrderedNodeAllocatorMap;

use crate::error::ErrorCode;
use crate::generate_vault_seeds;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct Settle<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        has_one = base_mint,
        has_one = quote_mint,
        has_one = settlement,
    )]
    pub hybrid_market: Account<'info, HybridMarket>,
    #[account(zero)]
    pub settlement: AccountLoader<'info, Settlement>,
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

pub fn handler(ctx: Context<Settle>) -> Result<()> {
    let Settle {
        user,
        hybrid_market,
        settlement: settlement_loader,
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

    let settlement = &mut settlement_loader.load_mut()?;

    let voucher = settlement.pop_voucher(user.key())?.unwrap();

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

    token_transfer_signed(
        voucher.base_amount,
        token_program,
        vault_ata_base,
        user_ata_base,
        hybrid_market,
        seeds,
    )?;

    token_transfer_signed(
        voucher.quote_amount,
        token_program,
        vault_ata_quote,
        user_ata_quote,
        hybrid_market,
        seeds,
    )?;

    Ok(())
}
