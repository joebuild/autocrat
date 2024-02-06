use anchor_lang::prelude::*;

#[account]
pub struct ProposalVault {
    pub number: u32,
}
