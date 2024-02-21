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
use crate::program::Autocrat;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
#[instruction(is_pass_market: bool)]
pub struct CreateProposalMarketSide<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = proposer,
        space = 8 + std::mem::size_of::<Proposal>() + 100,
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        mut,
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    /// CHECK:
    pub amm: UncheckedAccount<'info>,
    /// CHECK
    pub amm_position: UncheckedAccount<'info>,
    #[account(
        constraint = meta_mint.key() == dao.meta_mint.key()
    )]
    pub meta_mint: Box<Account<'info, Mint>>,
    #[account(
        constraint = usdc_mint.key() == dao.usdc_mint.key()
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mint::authority = proposal,
        mint::decimals = meta_mint.decimals,
    )]
    pub conditional_meta_mint: Box<Account<'info, Mint>>,
    #[account(
        mint::authority = proposal,
        mint::decimals = usdc_mint.decimals,
    )]
    pub conditional_usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = conditional_meta_mint,
        associated_token::authority = proposer,
    )]
    pub conditional_meta_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_usdc_mint,
        associated_token::authority = proposer,
    )]
    pub conditional_usdc_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_meta_mint,
        associated_token::authority = amm,
    )]
    pub conditional_meta_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_usdc_mint,
        associated_token::authority = amm,
    )]
    pub conditional_usdc_vault_ata: Box<Account<'info, TokenAccount>>,
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
    base_amount: u64,
    quote_amount: u64,
) -> Result<()> {
    let CreateProposalMarketSide {
        proposer,
        proposal,
        dao,
        amm,
        amm_position,
        meta_mint,
        usdc_mint,
        conditional_meta_mint,
        conditional_usdc_mint,
        conditional_meta_user_ata,
        conditional_usdc_user_ata,
        conditional_meta_vault_ata,
        conditional_usdc_vault_ata,
        amm_program: _,
        associated_token_program: _,
        token_program: _,
        instructions: _,
        system_program: _,
    } = ctx.accounts;

    // verify accounts
    assert_eq!(conditional_meta_mint.supply, 0);
    assert_eq!(conditional_usdc_mint.supply, 0);

    assert!(conditional_meta_mint.freeze_authority.is_none());
    assert!(conditional_usdc_mint.freeze_authority.is_none());

    // verify proposal submission steps are correct
    if proposal.is_pass_market_created || proposal.is_fail_market_created {
        assert_eq!(proposal.proposer, proposer.key());
    } else {
        proposal.proposer = proposer.key();
    }

    // if the proposal account was just created, then set some initial values
    if !proposal.is_pass_market_created && !proposal.is_fail_market_created {
        proposal.state = ProposalState::Initialize;
        proposal.number = dao.proposal_count;
        dao.proposal_count += 1;

        proposal.meta_mint = dao.meta_mint;
        proposal.usdc_mint = dao.usdc_mint;
    } else {
        // and if it wasn't just created, make sure we're still in the initialize state
        assert_eq!(proposal.state, ProposalState::Initialize);
    }

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

    // make sure the quote amount meets liquidity requirements
    assert!(quote_amount >= dao.amm_initial_quote_liquidity_amount);
    assert!(base_amount > 0);

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
    amm::cpi::add_liquidity(add_liquidity_ctx, base_amount, quote_amount)?;

    Ok(())
}

impl<'info> CreateProposalMarketSide<'info> {
    fn into_create_amm_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateAmm<'info>> {
        let cpi_accounts = CreateAmm {
            user: self.proposer.to_account_info(),
            amm: self.amm.to_account_info(),
            base_mint: self.conditional_meta_mint.to_account_info(),
            quote_mint: self.conditional_usdc_mint.to_account_info(),
            vault_ata_base: self.conditional_meta_vault_ata.to_account_info(),
            vault_ata_quote: self.conditional_usdc_vault_ata.to_account_info(),
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
            user_ata_base: self.conditional_meta_user_ata.to_account_info(),
            user_ata_quote: self.conditional_usdc_user_ata.to_account_info(),
            vault_ata_base: self.conditional_meta_vault_ata.to_account_info(),
            vault_ata_quote: self.conditional_usdc_vault_ata.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            instructions: self.instructions.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
