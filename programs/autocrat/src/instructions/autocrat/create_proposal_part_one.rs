use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;

use crate::error::ErrorCode;
use crate::state::*;

#[derive(Accounts)]
pub struct CreateProposalPartOne<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        init,
        payer = proposer,
        space = 8 + std::mem::size_of::<Proposal>() + 100,
        seeds = [
            b"proposal",
            dao.proposal_count.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,
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
    /// CHECK: never read
    #[account(
        mut,
        seeds = [dao.key().as_ref()],
        bump = dao.treasury_pda_bump
    )]
    pub dao_treasury: UncheckedAccount<'info>,
    #[account(
        init,
        payer = proposer,
        space = 8 + std::mem::size_of::<Amm>(),
        seeds = [
            b"pass_market_amm",
            dao.proposal_count.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub pass_market_amm: Box<Account<'info, Amm>>,
    #[account(
        init,
        payer = proposer,
        space = 8 + std::mem::size_of::<Amm>(),
        seeds = [
            b"fail_market_amm",
            dao.proposal_count.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub fail_market_amm: Box<Account<'info, Amm>>,
    #[account(
        constraint = meta_mint.key() == dao.meta_mint.key()
    )]
    pub meta_mint: Box<Account<'info, Mint>>,
    #[account(
        constraint = usdc_mint.key() == dao.usdc_mint.key()
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = proposer,
        mint::authority = proposal,
        mint::decimals = meta_mint.decimals,
        seeds = [
            b"conditional_on_pass_meta",
            dao.proposal_count.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub conditional_on_pass_meta_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = proposer,
        mint::authority = proposal,
        mint::decimals = usdc_mint.decimals,
        seeds = [
            b"conditional_on_pass_usdc",
            dao.proposal_count.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub conditional_on_pass_usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = proposer,
        mint::authority = proposal,
        mint::decimals = meta_mint.decimals,
        seeds = [
            b"conditional_on_fail_meta",
            dao.proposal_count.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub conditional_on_fail_meta_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = proposer,
        mint::authority = proposal,
        mint::decimals = usdc_mint.decimals,
        seeds = [
            b"conditional_on_fail_usdc",
            dao.proposal_count.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub conditional_on_fail_usdc_mint: Box<Account<'info, Mint>>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateProposalPartOne>,
    description_url: String,
) -> Result<()> {
    let CreateProposalPartOne {
        proposer,
        proposal,
        proposal_instructions,
        dao,
        dao_treasury,
        pass_market_amm,
        fail_market_amm,
        meta_mint,
        usdc_mint,
        conditional_on_pass_meta_mint,
        conditional_on_pass_usdc_mint,
        conditional_on_fail_meta_mint,
        conditional_on_fail_usdc_mint,
        associated_token_program: _,
        token_program: _,
        system_program: _,
    } = ctx.accounts;

    assert!(!proposal.part_one_complete);
    proposal.part_one_complete = true;

    proposal_instructions.proposal_instructions_frozen = true;

    proposal.number = dao.proposal_count;
    dao.proposal_count += 1;

    let clock = Clock::get()?;
    let slots_passed = clock.slot - dao.last_proposal_slot;
    let burn_amount = dao.base_burn_lamports.saturating_sub(
        dao.burn_decay_per_slot_lamports.saturating_mul(slots_passed),
    );
    dao.last_proposal_slot = clock.slot;

    let lockup_ix = solana_program::system_instruction::transfer(
        proposer.key,
        dao_treasury.key,
        burn_amount,
    );

    solana_program::program::invoke(
        &lockup_ix,
        &[
            proposer.to_account_info(),
            dao_treasury.to_account_info(),
        ],
    )?;

    proposal.proposer = proposer.key();
    proposal.description_url = description_url;
    
    proposal.state = ProposalState::Pending;
    proposal.instructions = proposal_instructions.key();
    proposal.pass_market_amm = pass_market_amm.key();
    proposal.fail_market_amm = fail_market_amm.key();

    proposal.meta_mint = dao.meta_mint;
    proposal.usdc_mint = dao.usdc_mint;

    proposal.conditional_on_pass_meta_mint = conditional_on_pass_meta_mint.key();
    proposal.conditional_on_pass_usdc_mint = conditional_on_pass_usdc_mint.key();
    proposal.conditional_on_fail_meta_mint = conditional_on_fail_meta_mint.key();
    proposal.conditional_on_fail_usdc_mint = conditional_on_fail_usdc_mint.key();

    // ==== pass market amm ====
    pass_market_amm.conditional_base_mint = conditional_on_pass_meta_mint.key();
    pass_market_amm.conditional_quote_mint = conditional_on_pass_usdc_mint.key();
    pass_market_amm.conditional_base_mint_decimals = meta_mint.decimals;
    pass_market_amm.conditional_quote_mint_decimals = usdc_mint.decimals;

    // ==== pass market amm ====
    fail_market_amm.conditional_base_mint = conditional_on_fail_meta_mint.key();
    fail_market_amm.conditional_quote_mint = conditional_on_fail_usdc_mint.key();
    fail_market_amm.conditional_base_mint_decimals = meta_mint.decimals;
    fail_market_amm.conditional_quote_mint_decimals = usdc_mint.decimals;

    Ok(())
}
