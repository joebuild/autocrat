use anchor_lang::prelude::*;

use crate::state::*;
use crate::utils::get_instructions_size;

#[derive(Accounts)]
#[instruction(instructions: Vec<ProposalInstruction>)]
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
        realloc = proposal_instructions.to_account_info().data_len() + get_instructions_size(&instructions),
        realloc::payer = proposer,
        realloc::zero = false,
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

    assert!(!proposal_instructions.proposal_instructions_frozen);

    proposal_instructions.instructions.extend(instructions.into_iter());

    Ok(())
}
