use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use solana_program::native_token::LAMPORTS_PER_SOL;

use amm::cpi::accounts::AddLiquidity;
use amm::cpi::accounts::CreateAmm;
use amm::cpi::accounts::CreatePosition;
use amm::instructions::create_amm::CreateAmmParams;
use amm::program::Amm;

use crate::error::ErrorCode;
use crate::generate_vault_seeds;
use crate::state::*;

#[derive(Accounts)]
pub struct SubmitProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(mut)]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        init,
        payer = proposer,
        space = 8 + std::mem::size_of::<ProposalTreasury>(),
        seeds = [
            proposal.key().as_ref(),
        ],
        bump
    )]
    pub proposal_treasury: Box<Account<'info, ProposalTreasury>>,
    #[account(
        mut,
        constraint = proposal_instructions.proposer == proposer.key(),
        constraint = proposal_instructions.proposal_number == dao.proposal_count @ ErrorCode::NonConsecutiveProposalNumber,
    )]
    pub proposal_instructions: Box<Account<'info, ProposalInstructions>>,
    #[account(
        mut,
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SubmitProposal>, description_url: String) -> Result<()> {
    let SubmitProposal {
        proposer,
        proposal,
        proposal_treasury,
        proposal_instructions,
        dao,
        system_program: _,
    } = ctx.accounts;

    assert_eq!(proposal.proposer, proposer.key());

    assert!(proposal.is_pass_market_created);
    assert!(proposal.is_fail_market_created);

    assert_eq!(proposal.state, ProposalState::Initialize);
    proposal.state = ProposalState::Pending;

    proposal.description_url = description_url;
    proposal.proposal_treasury = proposal_treasury.key();
    proposal.instructions = proposal_instructions.key();
    proposal.slot_enqueued = Clock::get()?.slot;

    proposal_instructions.proposal_instructions_frozen = true;

    Ok(())
}
