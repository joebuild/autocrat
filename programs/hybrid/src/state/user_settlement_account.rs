use anchor_lang::prelude::*;

#[account]
pub struct UserSettlementAccount {
    pub user: Pubkey,
}
