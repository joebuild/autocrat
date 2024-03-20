use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;

use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        mut,
        has_one = meta_mint,
        has_one = usdc_mint,
        seeds = [dao.id.as_ref()],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [
            PROPOSAL_SEED_PREFIX,
            dao.key().as_ref(),
            dao.proposal_count.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        init,
        payer = proposer,
        space = 8 + std::mem::size_of::<ProposalVault>(),
        seeds = [
            PROPOSAL_VAULT_SEED_PREFIX,
            proposal.key().as_ref(),
        ],
        bump
    )]
    pub proposal_vault: Box<Account<'info, ProposalVault>>,
    pub meta_mint: Box<Account<'info, Mint>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = meta_mint,
        associated_token::authority = proposer,
    )]
    pub meta_proposer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = proposer,
    )]
    pub usdc_proposer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = proposer,
        associated_token::mint = meta_mint,
        associated_token::authority = proposal_vault,
    )]
    pub meta_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = proposer,
        associated_token::mint = usdc_mint,
        associated_token::authority = proposal_vault,
    )]
    pub usdc_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateProposal>,
    description_url: String,
    mint_cond_meta: u64,
    mint_cond_usdc: u64,
) -> Result<()> {
    let CreateProposal {
        proposer,
        dao,
        proposal,
        proposal_vault,
        meta_mint: _,
        usdc_mint: _,
        meta_proposer_ata,
        usdc_proposer_ata,
        meta_vault_ata,
        usdc_vault_ata,
        associated_token_program: _,
        token_program,
        system_program: _,
    } = ctx.accounts;

    assert!(description_url.len() <= 100);

    assert!(mint_cond_meta > 0);
    assert!(mint_cond_usdc >= dao.amm_initial_quote_liquidity_amount);

    proposal.dao = dao.key();

    proposal.proposer = proposer.key();
    proposal.state = ProposalState::Initialize;
    proposal.description_url = description_url;
    proposal.proposal_vault = proposal_vault.key();

    proposal.meta_mint = dao.meta_mint;
    proposal.usdc_mint = dao.usdc_mint;

    proposal.proposer_inititial_conditional_meta_minted = mint_cond_meta;
    proposal.proposer_inititial_conditional_usdc_minted = mint_cond_usdc;

    proposal.number = dao.proposal_count;
    dao.proposal_count = dao.proposal_count.checked_add(1).unwrap();

    proposal_vault.proposal = proposal.key();
    proposal_vault.meta_vault_ata = meta_vault_ata.key();
    proposal_vault.usdc_vault_ata = usdc_vault_ata.key();

    // transfer user meta to vault
    token_transfer(
        proposal.proposer_inititial_conditional_meta_minted,
        token_program,
        meta_proposer_ata.as_ref(),
        meta_vault_ata.as_ref(),
        proposer,
    )?;

    // transfer user usdc to vault
    token_transfer(
        proposal.proposer_inititial_conditional_usdc_minted,
        token_program,
        usdc_proposer_ata.as_ref(),
        usdc_vault_ata.as_ref(),
        proposer,
    )?;

    Ok(())
}
