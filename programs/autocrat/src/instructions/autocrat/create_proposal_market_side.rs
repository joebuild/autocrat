use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;

use amm::cpi::accounts::AddLiquidity;
use amm::cpi::accounts::CreateAmm;
use amm::cpi::accounts::CreatePosition;
use amm::instructions::create_amm::CreateAmmParams;
use amm::program::Amm;

use crate::error::ErrorCode;
use crate::generate_proposal_vault_seeds;
use crate::program::Autocrat;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct CreateProposalMarketSide<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"proposal",
            proposal.proposer.as_ref(),
            proposal.number.to_le_bytes().as_ref(),
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
        has_one = meta_mint,
        has_one = usdc_mint,
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    #[account(mut)]
    /// CHECK: initialized in the AMM program
    pub amm: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked in the AMM program
    pub amm_position: UncheckedAccount<'info>,
    pub meta_mint: Box<Account<'info, Mint>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = proposer,
        mint::authority = proposal_vault,
        mint::decimals = meta_mint.decimals,
    )]
    pub conditional_meta_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = proposer,
        mint::authority = proposal_vault,
        mint::decimals = usdc_mint.decimals,
    )]
    pub conditional_usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = proposer,
        associated_token::mint = conditional_meta_mint,
        associated_token::authority = proposer,
    )]
    pub conditional_meta_proposer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = proposer,
        associated_token::mint = conditional_usdc_mint,
        associated_token::authority = proposer,
    )]
    pub conditional_usdc_proposer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = proposer,
        associated_token::mint = conditional_meta_mint,
        associated_token::authority = amm,
    )]
    pub conditional_meta_amm_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = proposer,
        associated_token::mint = conditional_usdc_mint,
        associated_token::authority = amm,
    )]
    pub conditional_usdc_amm_vault_ata: Box<Account<'info, TokenAccount>>,
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

pub fn handler(
    ctx: Context<CreateProposalMarketSide>,
    is_pass_market: bool,
    amm_cond_meta_deposit: u64,
    amm_cond_usdc_deposit: u64,
) -> Result<()> {
    let CreateProposalMarketSide {
        proposer,
        proposal,
        proposal_vault,
        dao,
        amm,
        amm_position: _,
        meta_mint: _,
        usdc_mint: _,
        conditional_meta_mint,
        conditional_usdc_mint,
        conditional_meta_proposer_ata,
        conditional_usdc_proposer_ata,
        conditional_meta_amm_vault_ata: _,
        conditional_usdc_amm_vault_ata: _,
        amm_program: _,
        associated_token_program: _,
        token_program,
        instructions: _,
        system_program: _,
    } = ctx.accounts;

    assert_eq!(proposal.proposer, proposer.key());
    assert_eq!(proposal.state, ProposalState::Initialize);

    // set the corresponding pass/fail parameters
    if is_pass_market {
        assert!(!proposal.is_pass_market_created);
        proposal.is_pass_market_created = true;
        proposal.pass_market_amm = amm.key();
        proposal.conditional_on_pass_meta_mint = conditional_meta_mint.key();
        proposal.conditional_on_pass_usdc_mint = conditional_usdc_mint.key();
    } else {
        assert!(!proposal.is_fail_market_created);
        proposal.is_fail_market_created = true;
        proposal.fail_market_amm = amm.key();
        proposal.conditional_on_fail_meta_mint = conditional_meta_mint.key();
        proposal.conditional_on_fail_usdc_mint = conditional_usdc_mint.key();
    }

    // sanity check
    assert!(proposal.proposer_inititial_conditional_meta_minted >= amm_cond_meta_deposit);
    assert!(proposal.proposer_inititial_conditional_usdc_minted >= amm_cond_usdc_deposit);

    // mint the proposer's conditional tokens
    let proposal_vault_key = proposal_vault.key();
    let seeds = generate_proposal_vault_seeds!(proposal_vault_key, ctx.bumps.proposal_vault);

    token_mint_signed(
        proposal.proposer_inititial_conditional_meta_minted,
        token_program,
        conditional_meta_mint.as_ref(),
        conditional_meta_proposer_ata.as_ref(),
        proposal_vault.as_ref(),
        seeds,
    )?;

    token_mint_signed(
        proposal.proposer_inititial_conditional_usdc_minted,
        token_program,
        conditional_usdc_mint.as_ref(),
        conditional_usdc_proposer_ata.as_ref(),
        proposal_vault.as_ref(),
        seeds,
    )?;

    // make sure the quote amount meets liquidity requirements
    assert!(amm_cond_usdc_deposit >= dao.amm_initial_quote_liquidity_amount);
    assert!(amm_cond_meta_deposit > 0);

    // create amm
    let swap_fee_bps = dao.amm_swap_fee_bps;
    let ltwap_decimals = dao.amm_ltwap_decimals;
    let create_amm_ctx = ctx.accounts.into_create_amm_context();
    amm::cpi::create_amm(
        create_amm_ctx,
        CreateAmmParams {
            permissioned: true,
            permissioned_caller: Some(Autocrat::id()),
            swap_fee_bps,
            ltwap_decimals,
        },
    )?;

    // create proposer LP position
    let create_amm_position_ctx = ctx.accounts.into_create_amm_position_context();
    amm::cpi::create_position(create_amm_position_ctx)?;

    // add liquidity to proposer LP position
    let add_liquidity_ctx = ctx.accounts.into_add_liquidity_context();
    amm::cpi::add_liquidity(
        add_liquidity_ctx,
        amm_cond_meta_deposit,
        amm_cond_usdc_deposit,
    )?;

    Ok(())
}

impl<'info> CreateProposalMarketSide<'info> {
    fn into_create_amm_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateAmm<'info>> {
        let cpi_accounts = CreateAmm {
            user: self.proposer.to_account_info(),
            amm: self.amm.to_account_info(),
            base_mint: self.conditional_meta_mint.to_account_info(),
            quote_mint: self.conditional_usdc_mint.to_account_info(),
            vault_ata_base: self.conditional_meta_amm_vault_ata.to_account_info(),
            vault_ata_quote: self.conditional_usdc_amm_vault_ata.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'info> CreateProposalMarketSide<'info> {
    fn into_create_amm_position_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, CreatePosition<'info>> {
        let cpi_accounts = CreatePosition {
            user: self.proposer.to_account_info(),
            amm: self.amm.to_account_info(),
            amm_position: self.amm_position.to_account_info(),
            instructions: self.instructions.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'info> CreateProposalMarketSide<'info> {
    fn into_add_liquidity_context(&self) -> CpiContext<'_, '_, '_, 'info, AddLiquidity<'info>> {
        let cpi_accounts = AddLiquidity {
            user: self.proposer.to_account_info(),
            amm: self.amm.to_account_info(),
            amm_position: self.amm_position.to_account_info(),
            base_mint: self.conditional_meta_mint.to_account_info(),
            quote_mint: self.conditional_usdc_mint.to_account_info(),
            user_ata_base: self.conditional_meta_proposer_ata.to_account_info(),
            user_ata_quote: self.conditional_usdc_proposer_ata.to_account_info(),
            vault_ata_base: self.conditional_meta_amm_vault_ata.to_account_info(),
            vault_ata_quote: self.conditional_usdc_amm_vault_ata.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            instructions: self.instructions.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
