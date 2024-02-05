use anchor_lang::prelude::*;
use anchor_lang::solana_program;

use crate::state::*;

#[account]
pub struct ConditionalVault {
    pub status: VaultStatus,
    /// The account that can either finalize the vault to make conditional tokens
    /// redeemable for underlying tokens or revert the vault to make deposit
    /// slips redeemable for underlying tokens.
    pub settlement_authority: Pubkey,
    /// The mint of the tokens that are deposited into the vault.
    pub underlying_token_mint: Pubkey,
    /// A nonce to allow a single account to be the settlement authority of multiple
    /// vaults with the same underlying token mints.
    pub nonce: u64,
    /// The vault's storage account for deposited funds.
    pub underlying_token_account: Pubkey,
    pub conditional_on_finalize_token_mint: Pubkey,
    pub conditional_on_revert_token_mint: Pubkey,
    pub pda_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum VaultStatus {
    Active,
    Finalized,
    Reverted,
}
