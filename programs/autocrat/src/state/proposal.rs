use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;

// feels more idiomatic to put this up here
#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum ProposalState {
    Pending,
    Passed,
    Failed,
}

#[account]
pub struct Proposal {
    pub number: u32,
    pub proposer: Pubkey,
    pub description_url: String,
    pub slot_enqueued: u64,
    pub state: ProposalState,
    pub instructions: Pubkey,

    pub part_one_complete: bool,
    pub part_two_complete: bool,

    pub meta_mint: Pubkey,
    pub usdc_mint: Pubkey,

    pub pass_market_amm: Pubkey,
    pub fail_market_amm: Pubkey,

    pub conditional_on_pass_meta_mint: Pubkey,
    pub conditional_on_pass_usdc_mint: Pubkey,
    pub conditional_on_fail_meta_mint: Pubkey,
    pub conditional_on_fail_usdc_mint: Pubkey,
}

// Proph3t: you could save space if you used indexes for the accounts and stored all the accounts
// in here. idk, might not be worth it tho as it adds complexity which creates attack surface area.
#[account]
pub struct ProposalInstructions {
    pub proposal_number: u32,
    pub proposer: Pubkey,
    pub proposal_instructions_frozen: bool,
    pub instructions: Vec<ProposalInstruction>,
}

impl ProposalInstructions {
    pub const SERIALIZED_LEN: usize = 4 + 32 + 1 + 4;
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
