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

use amm::cpi::accounts::AddLiquidity as AmmAddLiquidity;
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
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
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
        associated_token::authority = user,
    )]
    pub conditional_meta_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_usdc_mint,
        associated_token::authority = user,
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
    ctx: Context<AddLiquidity>,
    max_base_amount: u64,
    max_quote_amount: u64,
) -> Result<()> {
    let AddLiquidity {
        user,
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

    assert!(proposal.pass_market_amm == amm.key() || proposal.fail_market_amm == amm.key());
    assert_eq!(proposal.state, ProposalState::Pending);

    assert!(max_base_amount > 0);
    assert!(max_quote_amount > 0);

    // add liquidity to proposer LP position
    let add_liquidity_ctx = ctx.accounts.into_add_liquidity_context();
    amm::cpi::add_liquidity(add_liquidity_ctx, max_base_amount, max_quote_amount)?;

    Ok(())
}

impl<'info> AddLiquidity<'info> {
    fn into_add_liquidity_context(&self) -> CpiContext<'_, '_, '_, 'info, AmmAddLiquidity<'info>> {
        let cpi_accounts = AmmAddLiquidity {
            user: self.user.to_account_info(),
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
