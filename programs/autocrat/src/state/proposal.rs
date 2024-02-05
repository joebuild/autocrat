use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;

#[account]
pub struct Proposal {
    pub number: u32,
    pub proposer: Pubkey,
    pub description_url: String,
    pub slot_enqueued: u64,
    pub state: ProposalState,
    pub instructions: Pubkey,

    pub pass_market_amm: Pubkey,
    pub fail_market_amm: Pubkey,

    pub vault_pda: Pubkey,
    pub conditional_on_pass_meta_mint: Pubkey,
    pub conditional_on_pass_usdc_mint: Pubkey,
    pub conditional_on_fail_meta_mint: Pubkey,
    pub conditional_on_fail_usdc_mint: Pubkey,
}

#[account]
pub struct ProposalInstructions {
    pub number: u32,
    pub proposer: Pubkey,
    pub instructions: Vec<ProposalInstruction>,
}

#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum ProposalState {
    Pending,
    Passed,
    Failed,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ProposalAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&ProposalAccount> for AccountMeta {
    fn from(acc: &ProposalAccount) -> Self {
        Self {
            pubkey: acc.pubkey,
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        }
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ProposalInstruction {
    pub program_id: Pubkey,
    pub accounts: Vec<ProposalAccount>,
    pub data: Vec<u8>,
}

impl From<&ProposalInstruction> for Instruction {
    fn from(ix: &ProposalInstruction) -> Self {
        Self {
            program_id: ix.program_id,
            data: ix.data.clone(),
            accounts: ix.accounts.iter().map(Into::into).collect(),
        }
    }
}
