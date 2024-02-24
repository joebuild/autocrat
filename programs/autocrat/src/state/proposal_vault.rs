use anchor_lang::prelude::*;

#[account]
pub struct ProposalVault {
    pub bump: u8,
    pub proposal: Pubkey,

    pub meta_vault_ata: Pubkey,
    pub usdc_vault_ata: Pubkey,
}
