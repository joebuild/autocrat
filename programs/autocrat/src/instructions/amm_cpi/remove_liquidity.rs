use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;

use amm::cpi::accounts::RemoveLiquidity as AmmRemoveLiquidity;
use amm::program::Amm;

use crate::error::ErrorCode;
use crate::state::*;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        has_one = meta_mint,
        has_one = usdc_mint,
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        mut,
        seeds = [
            b"proposal_vault",
            proposal.key().as_ref(),
        ],
        bump
    )]
    pub proposal_vault: Box<Account<'info, ProposalVault>>,
    #[account(mut)]
    /// CHECK:
    pub amm: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK
    pub amm_position: UncheckedAccount<'info>,
    pub meta_mint: Box<Account<'info, Mint>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mint::authority = proposal_vault,
        mint::decimals = meta_mint.decimals,
    )]
    pub conditional_meta_mint: Box<Account<'info, Mint>>,
    #[account(
        mint::authority = proposal_vault,
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

pub fn handler(ctx: Context<RemoveLiquidity>, remove_bps: u64) -> Result<()> {
    require!(
        ctx.accounts.proposal.pass_market_amm == ctx.accounts.amm.key()
            || ctx.accounts.proposal.fail_market_amm == ctx.accounts.amm.key(),
        ErrorCode::AmmProposalMismatch
    );

    require!(
        remove_bps <= BPS_SCALE && remove_bps > 0,
        ErrorCode::RemoveLiquidityBpsOutOfRange
    );

    // stop the proposer from rugging liqudity before the proposal is concluded
    if ctx.accounts.proposal.proposer == ctx.accounts.user.key()
        && ctx.accounts.proposal.state == ProposalState::Pending
    {
        return err!(ErrorCode::ProposerCannotPullLiquidityWhileMarketIsPending);
    }

    // remove liquidity from LP position
    let add_liquidity_ctx = ctx.accounts.into_remove_liquidity_context();
    amm::cpi::remove_liquidity(add_liquidity_ctx, remove_bps)?;

    Ok(())
}

impl<'info> RemoveLiquidity<'info> {
    fn into_remove_liquidity_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, AmmRemoveLiquidity<'info>> {
        let cpi_accounts = AmmRemoveLiquidity {
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
