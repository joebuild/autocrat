use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
#[instruction(instructions: Vec<ProposalInstructions>)]
pub struct AddProposalInstructions<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    #[account(
        mut,
        constraint = proposal_instructions.proposer == proposer.key(),
        realloc = proposal_instructions.to_account_info().data_len() + std::mem::size_of_val(&*instructions),
        realloc::payer = proposer,
        realloc::zero = false,
    )]
    pub proposal_instructions: Box<Account<'info, ProposalInstructions>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddProposalInstructions>,
    instructions: Vec<ProposalInstruction>
) -> Result<()> {
    let AddProposalInstructions {
        proposer,
        dao,
        proposal_instructions,
        rent: _,
        system_program: _,
    } = ctx.accounts;

    assert!(!proposal_instructions.proposal_submitted);

    proposal_instructions.instructions.extend(instructions.into_iter());

    Ok(())
}
