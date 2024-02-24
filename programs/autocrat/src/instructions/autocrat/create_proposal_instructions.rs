use anchor_lang::prelude::*;

use crate::state::*;
use crate::utils::get_instructions_size;

#[derive(Accounts)]
#[instruction(instructions: Vec<ProposalInstruction>)]
pub struct CreateProposalInstructions<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"proposal",
            proposal.number.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        init,
        payer = proposer,
        space = 8 + ProposalInstructions::SERIALIZED_LEN + get_instructions_size(&instructions),
        seeds = [
            b"proposal_instructions",
            proposal.key().as_ref()
        ],
        bump
    )]
    pub proposal_instructions: Box<Account<'info, ProposalInstructions>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateProposalInstructions>,
    instructions: Vec<ProposalInstruction>,
) -> Result<()> {
    let CreateProposalInstructions {
        proposer,
        proposal,
        proposal_instructions,
        rent: _,
        system_program: _,
    } = ctx.accounts;

    proposal_instructions.proposer = proposer.key();
    proposal_instructions.proposal = proposal.key();
    proposal_instructions.instructions = instructions;

    proposal.instructions = proposal_instructions.key();

    Ok(())
}
