use anchor_lang::prelude::*;

#[account]
pub struct DaoTreasury {
    pub dao: Pubkey,
    pub bump: u8,
}
