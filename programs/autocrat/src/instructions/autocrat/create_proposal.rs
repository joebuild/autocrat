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
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        init,
        payer = proposer,
        space = 8 + std::mem::size_of::<Proposal>(),
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
        seeds = [
            b"proposal_instructions",
            dao.proposal_count.to_le_bytes().as_ref(),
        ],
        bump
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
    pub meta_mint: Account<'info, Mint>,
    #[account(
        constraint = usdc_mint.key() == dao.usdc_mint.key()
    )]
    pub usdc_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = proposer,
        mint::authority = proposal,
        mint::decimals = meta_mint.decimals
    )]
    pub conditional_on_pass_meta_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = proposer,
        mint::authority = proposal,
        mint::decimals = usdc_mint.decimals
    )]
    pub conditional_on_pass_usdc_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = proposer,
        mint::authority = proposal,
        mint::decimals = meta_mint.decimals
    )]
    pub conditional_on_fail_meta_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = proposer,
        mint::authority = proposal,
        mint::decimals = usdc_mint.decimals
    )]
    pub conditional_on_fail_usdc_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = meta_mint.key(),
        associated_token::authority = proposer,
    )]
    pub meta_proposer_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = usdc_mint.key(),
        associated_token::authority = proposer,
    )]
    pub usdc_proposer_ata: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = proposer,
        associated_token::mint = conditional_on_pass_meta_mint,
        associated_token::authority = proposal,
    )]
    pub conditional_on_pass_meta_vault_ata: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = proposer,
        associated_token::mint = conditional_on_pass_usdc_mint,
        associated_token::authority = proposal,
    )]
    pub conditional_on_pass_usdc_vault_ata: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = proposer,
        associated_token::mint = conditional_on_fail_meta_mint,
        associated_token::authority = proposal,
    )]
    pub conditional_on_fail_meta_vault_ata: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = proposer,
        associated_token::mint = conditional_on_fail_usdc_mint,
        associated_token::authority = proposal,
    )]
    pub conditional_on_fail_usdc_vault_ata: Account<'info, TokenAccount>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateProposal>,
    description_url: String,
    initial_pass_market_price_units: f32, // human-readable price (i.e. units)
    initial_fail_market_price_units: f32, // human-readable price (i.e. units)
    quote_liquidity_atoms_per_amm: u64,
) -> Result<()> {
    let CreateProposal {
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
        meta_proposer_ata,
        usdc_proposer_ata,
        conditional_on_pass_meta_vault_ata,
        conditional_on_pass_usdc_vault_ata,
        conditional_on_fail_meta_vault_ata,
        conditional_on_fail_usdc_vault_ata,
        associated_token_program: _,
        token_program: _,
        system_program: _,
    } = ctx.accounts;

    proposal_instructions.proposal_submitted = true;

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
    proposal.slot_enqueued = clock.slot;
    proposal.state = ProposalState::Pending;
    proposal.instructions = proposal_instructions.key();

    proposal.pass_market_amm = pass_market_amm.key();
    proposal.fail_market_amm = fail_market_amm.key();

    proposal.conditional_on_pass_meta_mint = conditional_on_pass_meta_mint.key();
    proposal.conditional_on_pass_usdc_mint = conditional_on_pass_usdc_mint.key();
    proposal.conditional_on_fail_meta_mint = conditional_on_fail_meta_mint.key();
    proposal.conditional_on_fail_usdc_mint = conditional_on_fail_usdc_mint.key();

    // ==== pass market amm ====
    pass_market_amm.conditional_base_mint = conditional_on_pass_meta_mint.key();
    pass_market_amm.conditional_quote_mint = conditional_on_pass_usdc_mint.key();
    pass_market_amm.conditional_base_mint_decimals = meta_mint.decimals;
    pass_market_amm.conditional_quote_mint_decimals = usdc_mint.decimals;
    pass_market_amm.ltwap_slot_updated = clock.slot;

    // ==== pass market amm ====
    fail_market_amm.conditional_base_mint = conditional_on_fail_meta_mint.key();
    fail_market_amm.conditional_quote_mint = conditional_on_fail_usdc_mint.key();
    fail_market_amm.conditional_base_mint_decimals = meta_mint.decimals;
    fail_market_amm.conditional_quote_mint_decimals = usdc_mint.decimals;
    fail_market_amm.ltwap_slot_updated = clock.slot;

    // ==== deposit initial liquidity ====
    // TODO

    Ok(())
}
