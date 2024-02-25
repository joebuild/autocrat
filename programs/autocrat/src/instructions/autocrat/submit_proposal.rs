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
        has_one = usdc_mint,
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    #[account(
        seeds = [dao.key().as_ref()],
        bump
    )]
    pub dao_treasury: Account<'info, DaoTreasury>,
    #[account(
        mut,
        has_one = proposer,
        has_one = pass_market_amm,
        has_one = fail_market_amm,
        seeds = [
            b"proposal",
            proposal.number.to_le_bytes().as_ref()
        ],
        bump
    )]
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
        has_one = proposal,
    )]
    pub proposal_instructions: Box<Account<'info, ProposalInstructions>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = proposer,
    )]
    pub usdc_proposer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = proposer,
        associated_token::mint = usdc_mint,
        associated_token::authority = dao_treasury,
    )]
    pub usdc_treasury_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// CHECK:
    pub pass_market_amm: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK:
    pub fail_market_amm: UncheckedAccount<'info>,
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

pub fn handler(ctx: Context<SubmitProposal>) -> Result<()> {
    let SubmitProposal {
        proposer,
        dao,
        dao_treasury,
        proposal,
        proposal_vault,
        proposal_instructions,
        usdc_mint,
        usdc_proposer_ata,
        usdc_treasury_vault_ata,
        pass_market_amm: _,
        fail_market_amm: _,
        amm_program: _,
        associated_token_program,
        token_program,
        instructions: _,
        system_program: _,
    } = ctx.accounts;

    assert_eq!(proposal.proposer, proposer.key());

    assert!(proposal.is_pass_market_created);
    assert!(proposal.is_fail_market_created);

    // pay proposal spam deterrent fee
    let usdc_fee = dao.proposal_fee_usdc;
    let active_proposals = dao.proposals_active;
    let fee_multiplier = 2u64.checked_pow(active_proposals).unwrap();
    let effective_usdc_fee = usdc_fee.checked_mul(fee_multiplier).unwrap();

    token_transfer(
        effective_usdc_fee,
        token_program,
        usdc_proposer_ata.as_ref(),
        usdc_treasury_vault_ata.as_ref(),
        proposer.as_ref(),
    )?;

    assert_eq!(proposal.state, ProposalState::Initialize);
    proposal.state = ProposalState::Pending;
    dao.proposals_active = dao.proposals_active.checked_add(1).unwrap();

    proposal.slot_enqueued = Clock::get()?.slot;
    proposal_instructions.proposal_instructions_frozen = true;

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
