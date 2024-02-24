use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;

use amm::cpi::accounts::UpdateLtwap;
use amm::program::Amm;

use crate::error::ErrorCode;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct SubmitProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        mut,
        has_one = meta_mint,
        has_one = usdc_mint,
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    #[account(mut)]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        signer,
        mut,
        seeds = [
            b"proposal_vault",
            proposal.key().as_ref(),
        ],
        bump
    )]
    pub proposal_vault: Box<Account<'info, ProposalVault>>,
    #[account(
        mut,
        has_one = proposer,
    )]
    pub proposal_instructions: Box<Account<'info, ProposalInstructions>>,
    #[account(mut)]
    /// CHECK:
    pub pass_market_amm: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK:
    pub fail_market_amm: UncheckedAccount<'info>,
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
        init_if_needed,
        payer = proposer,
        associated_token::mint = meta_mint,
        associated_token::authority = proposal_vault,
    )]
    pub meta_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = proposer,
        associated_token::mint = usdc_mint,
        associated_token::authority = proposal_vault,
    )]
    pub usdc_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(address = amm::ID)]
    pub amm_program: Program<'info, Amm>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = tx_instructions::ID)]
    /// CHECK:
    pub instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SubmitProposal>, description_url: String) -> Result<()> {
    let SubmitProposal {
        proposer,
        dao,
        proposal,
        proposal_vault,
        proposal_instructions,
        pass_market_amm: _,
        fail_market_amm: _,
        meta_mint: _,
        usdc_mint: _,
        meta_proposer_ata,
        usdc_proposer_ata,
        meta_vault_ata,
        usdc_vault_ata,
        amm_program: _,
        associated_token_program: _,
        token_program,
        instructions: _,
        system_program: _,
    } = ctx.accounts;

    assert_eq!(proposal.proposer, proposer.key());

    assert!(proposal.is_pass_market_created);
    assert!(proposal.is_fail_market_created);

    assert_eq!(proposal.state, ProposalState::Initialize);
    proposal.state = ProposalState::Pending;

    assert!(description_url.len() <= 50);

    proposal.description_url = description_url;
    proposal.proposal_vault = proposal_vault.key();
    proposal.instructions = proposal_instructions.key();
    proposal.slot_enqueued = Clock::get()?.slot;

    proposal.number = dao.proposal_count;
    dao.proposal_count += 1;

    proposal_instructions.proposal_instructions_frozen = true;

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

    // start LTWAP
    let update_pass_market_ltwap_ctx = ctx.accounts.into_update_pass_market_ltwap_context();
    amm::cpi::update_ltwap(update_pass_market_ltwap_ctx)?;

    let update_fail_market_ltwap_ctx = ctx.accounts.into_update_fail_market_ltwap_context();
    amm::cpi::update_ltwap(update_fail_market_ltwap_ctx)?;

    Ok(())
}

impl<'info> SubmitProposal<'info> {
    fn into_update_pass_market_ltwap_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, UpdateLtwap<'info>> {
        let cpi_accounts = UpdateLtwap {
            user: self.proposer.to_account_info(),
            amm: self.pass_market_amm.to_account_info(),
            instructions: self.instructions.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    fn into_update_fail_market_ltwap_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, UpdateLtwap<'info>> {
        let cpi_accounts = UpdateLtwap {
            user: self.proposer.to_account_info(),
            amm: self.fail_market_amm.to_account_info(),
            instructions: self.instructions.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
