use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;

use crate::program::Autocrat;
use amm::cpi::accounts::UpdateLtwap;
use amm::program::Amm;

use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct SubmitProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        mut,
        has_one = usdc_mint,
        seeds = [dao.id.as_ref()],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    #[account(
        seeds = [DAO_TREASURY_SEED_PREFIX, dao.key().as_ref()],
        bump = dao.treasury_pda_bump,
    )]
    pub dao_treasury: Account<'info, DaoTreasury>,
    #[account(
        mut,
        has_one = dao,
        has_one = proposer,
        has_one = pass_market_amm,
        has_one = fail_market_amm,
        seeds = [
            proposal.dao.as_ref(),
            PROPOSAL_SEED_PREFIX,
            proposal.number.to_le_bytes().as_ref()
            ],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        mut,
        has_one = proposal,
        seeds = [
            proposal.dao.as_ref(),
            PROPOSAL_VAULT_SEED_PREFIX,
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
    /// CHECK
    pub amm_auth_pda: UncheckedAccount<'info>,
    #[account(address = amm::ID)]
    pub amm_program: Program<'info, Amm>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SubmitProposal>) -> Result<()> {
    let SubmitProposal {
        proposer,
        dao,
        dao_treasury: _,
        proposal,
        proposal_vault: _,
        proposal_instructions,
        usdc_mint: _,
        usdc_proposer_ata,
        usdc_treasury_vault_ata,
        pass_market_amm: _,
        fail_market_amm: _,
        amm_auth_pda: _,
        amm_program: _,
        associated_token_program: _,
        token_program,
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
    proposal.slots_duration = dao.proposal_duration_slots;
    proposal_instructions.proposal_instructions_frozen = true;

    // start LTWAP
    let (_auth_pda, auth_pda_bump) =
        Pubkey::find_program_address(&[AMM_AUTH_SEED_PREFIX], &Autocrat::id());
    let seeds = &[AMM_AUTH_SEED_PREFIX, &[auth_pda_bump]];
    let signer = [&seeds[..]];

    let update_pass_market_ltwap_ctx = ctx.accounts.into_update_pass_market_ltwap_context(&signer);
    amm::cpi::update_ltwap(update_pass_market_ltwap_ctx, None)?;

    let update_fail_market_ltwap_ctx = ctx.accounts.into_update_fail_market_ltwap_context(&signer);
    amm::cpi::update_ltwap(update_fail_market_ltwap_ctx, None)?;

    Ok(())
}

impl<'info> SubmitProposal<'info> {
    fn into_update_pass_market_ltwap_context<'a, 'b, 'c>(
        &'a self,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'_, '_, '_, 'info, UpdateLtwap<'info>> {
        let cpi_accounts = UpdateLtwap {
            user: self.proposer.to_account_info(),
            amm: self.pass_market_amm.to_account_info(),
            system_program: self.system_program.to_account_info(),
            auth_pda: Some(self.amm_auth_pda.to_account_info()),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
    }

    fn into_update_fail_market_ltwap_context<'a, 'b, 'c>(
        &'a self,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'_, '_, '_, 'info, UpdateLtwap<'info>> {
        let cpi_accounts = UpdateLtwap {
            user: self.proposer.to_account_info(),
            amm: self.fail_market_amm.to_account_info(),
            system_program: self.system_program.to_account_info(),
            auth_pda: Some(self.amm_auth_pda.to_account_info()),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
    }
}
