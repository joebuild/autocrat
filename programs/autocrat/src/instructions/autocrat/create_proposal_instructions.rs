use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
#[instruction(size: usize, instructions: Vec<ProposalInstructions>)]
pub struct CreateProposalInstructions<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    #[account(
        init,
        payer = proposer,
        space = 8 + size
    )]
    pub proposal_instructions: Box<Account<'info, ProposalInstructions>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateProposalInstructions>,
    size: usize,
    instructions: Vec<ProposalInstruction>
) -> Result<()> {
    let CreateProposalInstructions {
        proposer,
        dao,
        proposal_instructions,
        rent,
        system_program: _,
    } = ctx.accounts;

    proposal_instructions.number = dao.proposal_count;
    proposal_instructions.proposer = proposer.key();
    proposal_instructions.instructions = instructions;

    Ok(())
}
