use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;

use crate::error::ErrorCode;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct SubmitProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(mut)]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        init_if_needed,
        payer = proposer,
        space = 8 + std::mem::size_of::<ProposalVault>(),
        seeds = [
            proposal.key().as_ref(),
        ],
        bump
    )]
    pub proposal_vault: Box<Account<'info, ProposalVault>>,
    #[account(
        mut,
        constraint = proposal_instructions.proposer == proposer.key(),
    )]
    pub proposal_instructions: Box<Account<'info, ProposalInstructions>>,
    #[account(
        mut,
        associated_token::mint = proposal.meta_mint,
        associated_token::authority = proposer,
    )]
    pub meta_proposer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = proposal.usdc_mint,
        associated_token::authority = proposer,
    )]
    pub usdc_proposer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = proposal.meta_mint,
        associated_token::authority = proposal_vault,
    )]
    pub meta_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = proposal.usdc_mint,
        associated_token::authority = proposal_vault,
    )]
    pub usdc_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SubmitProposal>, description_url: String) -> Result<()> {
    let SubmitProposal {
        proposer,
        proposal,
        proposal_vault,
        proposal_instructions,
        meta_proposer_ata,
        usdc_proposer_ata,
        meta_vault_ata,
        usdc_vault_ata,
        associated_token_program: _,
        token_program,
        system_program: _,
    } = ctx.accounts;

    assert_eq!(proposal.proposer, proposer.key());

    assert!(proposal.is_pass_market_created);
    assert!(proposal.is_fail_market_created);

    assert_eq!(proposal.state, ProposalState::Initialize);
    proposal.state = ProposalState::Pending;

    proposal.description_url = description_url;
    proposal.proposal_vault = proposal_vault.key();
    proposal.instructions = proposal_instructions.key();
    proposal.slot_enqueued = Clock::get()?.slot;

    proposal_instructions.proposal_instructions_frozen = true;

    // transfer user meta to vault
    token_transfer(
        proposal.proposer_inititial_conditional_meta_minted,
        token_program,
        meta_proposer_ata.as_ref(),
        meta_vault_ata.as_ref(),
        proposer.as_ref(),
    )?;

    // transfer user usdc to vault
    token_transfer(
        proposal.proposer_inititial_conditional_usdc_minted,
        token_program,
        usdc_proposer_ata.as_ref(),
        usdc_vault_ata.as_ref(),
        proposer.as_ref(),
    )?;

    Ok(())
}
