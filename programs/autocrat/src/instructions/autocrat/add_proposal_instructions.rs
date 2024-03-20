use anchor_lang::prelude::*;

use crate::state::*;
use crate::utils::get_instructions_size;

#[derive(Accounts)]
#[instruction(instructions: Vec<ProposalInstruction>)]
pub struct AddProposalInstructions<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        mut,
        has_one = proposer,
        seeds = [
            proposal.dao.as_ref(),
            PROPOSAL_SEED_PREFIX,
            proposal.number.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        mut,
        has_one = proposal,
        has_one = proposer,
        realloc = proposal_instructions.to_account_info().data_len() + get_instructions_size(&instructions),
        realloc::payer = proposer,
        realloc::zero = false,
        seeds = [
            proposal.dao.as_ref(),
            PROPOSAL_INSTRUCTIONS_SEED_PREFIX,
            proposal.key().as_ref()
        ],
        bump
    )]
    pub proposal_instructions: Box<Account<'info, ProposalInstructions>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddProposalInstructions>,
    instructions: Vec<ProposalInstruction>,
) -> Result<()> {
    let AddProposalInstructions {
        proposer: _,
        proposal,
        proposal_instructions,
        rent: _,
        system_program: _,
    } = ctx.accounts;

    assert_eq!(proposal_instructions.key(), proposal.instructions);

    assert!(!proposal_instructions.proposal_instructions_frozen);
    assert_eq!(proposal.state, ProposalState::Initialize);

    proposal_instructions
        .instructions
        .extend(instructions.into_iter());

    Ok(())
}
