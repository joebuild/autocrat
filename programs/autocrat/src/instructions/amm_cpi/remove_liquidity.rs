use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;

use crate::program::Autocrat;
use amm::cpi::accounts::RemoveLiquidity as AmmRemoveLiquidity;
use amm::cpi::accounts::UpdateLtwap;
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
            proposal.dao.as_ref(),
            PROPOSAL_VAULT_SEED_PREFIX,
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
    /// CHECK
    pub amm_auth_pda: UncheckedAccount<'info>,
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

    // stop the proposer from pulling liquidity before the proposal is concluded
    if ctx.accounts.proposal.proposer == ctx.accounts.user.key()
        && ctx.accounts.proposal.state == ProposalState::Pending
    {
        return err!(ErrorCode::ProposerCannotPullLiquidityWhileMarketIsPending);
    }

    let (_auth_pda, auth_pda_bump) =
        Pubkey::find_program_address(&[AMM_AUTH_SEED_PREFIX], &Autocrat::id());
    let seeds = &[AMM_AUTH_SEED_PREFIX, &[auth_pda_bump]];
    let signer = [&seeds[..]];

    // update LTWAP and pass the final slot, so that it will freeze in case the proposal duration has passed
    let final_slot = ctx
        .accounts
        .proposal
        .slot_enqueued
        .checked_add(ctx.accounts.proposal.slots_duration)
        .unwrap();

    let update_ltwap_ctx = ctx.accounts.into_update_ltwap_context(&signer);
    amm::cpi::update_ltwap(update_ltwap_ctx, Some(final_slot))?;

    // remove liquidity from LP position
    let add_liquidity_ctx = ctx.accounts.into_remove_liquidity_context(&signer);
    amm::cpi::remove_liquidity(add_liquidity_ctx, remove_bps)?;

    Ok(())
}

impl<'info> RemoveLiquidity<'info> {
    fn into_remove_liquidity_context<'a, 'b, 'c>(
        &'a self,
        signer_seeds: &'a [&'b [&'c [u8]]],
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
            system_program: self.system_program.to_account_info(),
            auth_pda: Some(self.amm_auth_pda.to_account_info()),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
    }

    fn into_update_ltwap_context<'a, 'b, 'c>(
        &'a self,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'_, '_, '_, 'info, UpdateLtwap<'info>> {
        let cpi_accounts = UpdateLtwap {
            user: self.user.to_account_info(),
            amm: self.amm.to_account_info(),
            system_program: self.system_program.to_account_info(),
            auth_pda: Some(self.amm_auth_pda.to_account_info()),
        };
        let cpi_program = self.amm_program.to_account_info();
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
    }
}
