use anchor_lang::prelude::*;

#[account]
pub struct AmmPosition {
    pub user: Pubkey,
    pub hybrid_market: Pubkey,
    pub ownership: u64,
}
