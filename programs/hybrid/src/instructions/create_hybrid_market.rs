use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::*;

use crate::error::ErrorCode;
use crate::state::*;
use crate::BPS_SCALE;

#[derive(Accounts)]
#[instruction(create_hybrid_market_params: CreateHybridMarketParams)]
pub struct CreateHybridMarket<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<HybridMarket>(),
        seeds = [
            base_mint.key().as_ref(),
            quote_mint.key().as_ref(),
            create_hybrid_market_params.swap_fee_bps.to_le_bytes().as_ref(),
            create_hybrid_market_params.permissioned_caller.unwrap_or(Pubkey::default()).as_ref()
        ],
        bump
    )]
    pub hybrid_market: Account<'info, HybridMarket>,
    pub base_mint: Account<'info, Mint>,
    pub quote_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = user,
        associated_token::authority = hybrid_market,
        associated_token::mint = base_mint
    )]
    pub vault_ata_base: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = user,
        associated_token::authority = hybrid_market,
        associated_token::mint = quote_mint
    )]
    pub vault_ata_quote: Account<'info, TokenAccount>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct CreateHybridMarketParams {
    pub permissioned: bool,
    pub permissioned_caller: Option<Pubkey>,
    pub swap_fee_bps: u32,
}

pub fn handler(
    ctx: Context<CreateHybridMarket>,
    create_hybrid_market_params: CreateHybridMarketParams,
) -> Result<()> {
    let CreateHybridMarket {
        user: _,
        hybrid_market,
        base_mint,
        quote_mint,
        vault_ata_base: _,
        vault_ata_quote: _,
        associated_token_program: _,
        token_program: _,
        system_program: _,
    } = ctx.accounts;

    hybrid_market.permissioned = create_hybrid_market_params.permissioned;
    if hybrid_market.permissioned {
        hybrid_market.permissioned_caller =
            create_hybrid_market_params.permissioned_caller.unwrap();
    }

    hybrid_market.created_at_slot = Clock::get()?.slot;

    assert!((create_hybrid_market_params.swap_fee_bps as u64) < BPS_SCALE);
    assert!((create_hybrid_market_params.swap_fee_bps as u64) > 0);

    hybrid_market.swap_fee_bps = create_hybrid_market_params.swap_fee_bps;

    assert_ne!(base_mint.key(), quote_mint.key());

    hybrid_market.base_mint = base_mint.key();
    hybrid_market.quote_mint = quote_mint.key();

    hybrid_market.base_mint_decimals = base_mint.decimals;
    hybrid_market.quote_mint_decimals = quote_mint.decimals;

    hybrid_market.bump = ctx.bumps.hybrid_market;

    Ok(())
}
