use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq, InitSpace)]
pub enum ProposalState {
    Initialize,
    Pending,
    Passed,
    Failed,
}

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub number: u64,
    pub proposer: Pubkey,
    #[max_len(100)]
    pub description_url: String,
    pub slot_enqueued: u64,
    pub slots_duration: u64,
    pub state: ProposalState,
    pub instructions: Pubkey,

    pub proposal_vault: Pubkey,

    pub is_pass_market_created: bool,
    pub is_fail_market_created: bool,

    pub meta_mint: Pubkey,
    pub usdc_mint: Pubkey,

    pub pass_market_amm: Pubkey,
    pub fail_market_amm: Pubkey,

    pub conditional_on_pass_meta_mint: Pubkey,
    pub conditional_on_pass_usdc_mint: Pubkey,

    pub conditional_on_fail_meta_mint: Pubkey,
    pub conditional_on_fail_usdc_mint: Pubkey,

    pub proposer_inititial_conditional_meta_minted: u64,
    pub proposer_inititial_conditional_usdc_minted: u64,
}

#[account]
pub struct ProposalInstructions {
    pub proposer: Pubkey,
    pub proposal: Pubkey,
    pub proposal_instructions_frozen: bool,
    pub instructions: Vec<ProposalInstruction>,
}

impl ProposalInstructions {
    pub const SERIALIZED_LEN: usize = 4 + 32 + 32 + 1 + 4;
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
