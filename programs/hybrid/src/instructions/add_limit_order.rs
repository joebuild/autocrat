use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::*;
use num_traits::ToPrimitive;
use sokoban::NodeAllocatorMap;
use sokoban::OrderedNodeAllocatorMap;

use crate::error::ErrorCode;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct AddLimitOrder<'info> {
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
    pub orders: AccountLoader<'info, Orders>,
    #[account(zero)]
    pub settlement: AccountLoader<'info, Settlement>,
    #[account(
        seeds = [
            USER_SETTLEMENT_ACCOUNT_SEED,
            user.key().as_ref(),
        ],
        bump
    )]
    pub user_settlement_account: Account<'info, UserSettlementAccount>,
    #[account(
        associated_token::mint = base_mint,
        associated_token::authority = user_settlement_account,
    )]
    pub user_settlement_ata_base: Account<'info, TokenAccount>,
    #[account(
        associated_token::mint = quote_mint,
        associated_token::authority = user_settlement_account,
    )]
    pub user_settlement_ata_quote: Account<'info, TokenAccount>,
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
    ctx: Context<AddLimitOrder>,
    is_bid_side: bool,
    quote_lots_per_base_lot: u64,
    base_lots: u64,
) -> Result<()> {
    let AddLimitOrder {
        user,
        hybrid_market: _,
        orders: order_loader,
        settlement: settlement_loader,
        user_settlement_account,
        user_settlement_ata_base,
        user_settlement_ata_quote,
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

    let orders = &mut order_loader.load_mut()?;
    let settlement = &mut settlement_loader.load_mut()?;

    assert!(base_lots > orders.min_order_size_base_lots);

    let order_tree = if is_bid_side {
        &mut orders.bid_tree
    } else {
        &mut orders.ask_tree
    };

    if order_tree.size() == order_tree.capacity() {
        orders.drop_worst_order(is_bid_side, settlement)?;
    }

    orders.add_order(is_bid_side, quote_lots_per_base_lot, base_lots, user.key())?;

    if is_bid_side {
        token_transfer(
            quote_lots_per_base_lot.checked_mul(base_lots).unwrap(),
            token_program,
            user_ata_quote,
            vault_ata_quote,
            user,
        )?;
    } else {
        token_transfer(
            base_lots,
            token_program,
            user_ata_base,
            vault_ata_base,
            user,
        )?;
    }

    Ok(())
}
