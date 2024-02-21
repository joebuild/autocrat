use anchor_lang::prelude::*;

#[account]
pub struct ProposalVault {
    pub proposal: Pubkey,
}
