use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::*;

use crate::state::*;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub dao: Account<'info, Dao>,
    #[account(mut)]
    pub amm: Account<'info, Amm>,
    pub conditional_base_mint: Account<'info, Mint>,
    pub conditional_quote_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_ata_conditional_base: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_ata_conditional_quote: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_ata_conditional_base: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_ata_conditional_quote: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle(
    ctx: Context<Swap>,
    is_quote_to_base: bool,
    input_amount: u64,
    output_amount_min: u64,
) -> Result<()> {

    // TODO

    Ok(())
}
