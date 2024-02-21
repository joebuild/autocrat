use anchor_lang::prelude::*;

use crate::state::*;
use crate::utils::get_instructions_size;

#[derive(Accounts)]
#[instruction(instructions: Vec<ProposalInstruction>)]
pub struct CreateProposalInstructions<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        init,
        payer = proposer,
        space = 8 + ProposalInstructions::SERIALIZED_LEN + get_instructions_size(&instructions),
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
        proposal_instructions,
        rent: _,
        system_program: _,
    } = ctx.accounts;

    proposal_instructions.proposer = proposer.key();
    proposal_instructions.instructions = instructions;

    Ok(())
}
