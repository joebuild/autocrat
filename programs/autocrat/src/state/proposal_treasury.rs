use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;

#[account]
pub struct ProposalTreasury {
    pub proposal: Pubkey,
}
