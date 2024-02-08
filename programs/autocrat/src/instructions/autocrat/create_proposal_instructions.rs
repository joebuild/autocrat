use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
#[instruction(instructions: Vec<ProposalInstructions>)]
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
        space = 8 + std::mem::size_of::<ProposalInstructions>() + std::mem::size_of_val(&*instructions),
        seeds = [
            b"proposal_instructions",
            dao.proposal_count.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub proposal_instructions: Box<Account<'info, ProposalInstructions>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateProposalInstructions>,
    instructions: Vec<ProposalInstruction>
) -> Result<()> {
    let CreateProposalInstructions {
        proposer,
        dao,
        proposal_instructions,
        rent: _,
        system_program: _,
    } = ctx.accounts;

    proposal_instructions.proposal_number = dao.proposal_count;
    proposal_instructions.proposer = proposer.key();
    proposal_instructions.instructions = instructions;

    Ok(())
}
