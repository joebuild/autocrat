use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::*;

use crate::error::ErrorCode;
use crate::generate_proposal_vault_seeds;
use crate::state::*;
use crate::utils::token::*;

#[derive(Accounts)]
pub struct MintConditionalTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        has_one = meta_mint,
        has_one = usdc_mint,
        has_one = conditional_on_pass_meta_mint,
        has_one = conditional_on_pass_usdc_mint,
        has_one = conditional_on_fail_meta_mint,
        has_one = conditional_on_fail_usdc_mint,
        seeds = [
            b"proposal",
            proposal.proposer.as_ref(),
            proposal.number.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        signer,
        mut,
        has_one = meta_vault_ata,
        has_one = usdc_vault_ata,
        seeds = [
            b"proposal_vault",
            proposal.key().as_ref(),
        ],
        bump
    )]
    pub proposal_vault: Box<Account<'info, ProposalVault>>,
    pub meta_mint: Box<Account<'info, Mint>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub conditional_on_pass_meta_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub conditional_on_pass_usdc_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub conditional_on_fail_meta_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub conditional_on_fail_usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = meta_mint,
        associated_token::authority = user,
    )]
    pub meta_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = user,
    )]
    pub usdc_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_on_pass_meta_mint,
        associated_token::authority = user,
    )]
    pub conditional_on_pass_meta_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_on_pass_usdc_mint,
        associated_token::authority = user,
    )]
    pub conditional_on_pass_usdc_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_on_fail_meta_mint,
        associated_token::authority = user,
    )]
    pub conditional_on_fail_meta_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_on_fail_usdc_mint,
        associated_token::authority = user,
    )]
    pub conditional_on_fail_usdc_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = meta_mint,
        associated_token::authority = proposal_vault,
    )]
    pub meta_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
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
    ctx: Context<MintConditionalTokens>,
    meta_amount: u64,
    usdc_amount: u64,
) -> Result<()> {
    let MintConditionalTokens {
        user,
        proposal: _,
        proposal_vault,
        meta_mint: _,
        usdc_mint: _,
        conditional_on_pass_meta_mint,
        conditional_on_pass_usdc_mint,
        conditional_on_fail_meta_mint,
        conditional_on_fail_usdc_mint,
        meta_user_ata,
        usdc_user_ata,
        conditional_on_pass_meta_user_ata,
        conditional_on_pass_usdc_user_ata,
        conditional_on_fail_meta_user_ata,
        conditional_on_fail_usdc_user_ata,
        meta_vault_ata,
        usdc_vault_ata,
        associated_token_program: _,
        token_program,
        system_program: _,
    } = ctx.accounts;

    let proposal_vault_key = proposal_vault.key();
    let seeds = generate_proposal_vault_seeds!(proposal_vault_key, ctx.bumps.proposal_vault);

    if meta_amount > 0 {
        // transfer user meta to vault
        token_transfer(
            meta_amount,
            token_program,
            meta_user_ata.as_ref(),
            meta_vault_ata.as_ref(),
            user.as_ref(),
        )?;

        // mint conditional on-pass meta to user
        token_mint_signed(
            meta_amount,
            token_program,
            conditional_on_pass_meta_mint.as_ref(),
            conditional_on_pass_meta_user_ata.as_ref(),
            proposal_vault.as_ref(),
            seeds,
        )?;

        // mint conditional on-fail meta to user
        token_mint_signed(
            meta_amount,
            token_program,
            conditional_on_fail_meta_mint.as_ref(),
            conditional_on_fail_meta_user_ata.as_ref(),
            proposal_vault.as_ref(),
            seeds,
        )?;
    }

    if usdc_amount > 0 {
        // transfer user usdc to vault
        token_transfer(
            usdc_amount,
            token_program,
            usdc_user_ata.as_ref(),
            usdc_vault_ata.as_ref(),
            user.as_ref(),
        )?;

        // mint conditional on-pass usdc to user
        token_mint_signed(
            usdc_amount,
            token_program,
            conditional_on_pass_usdc_mint.as_ref(),
            conditional_on_pass_usdc_user_ata.as_ref(),
            proposal_vault.as_ref(),
            seeds,
        )?;

        // mint conditional on-fail usdc to user
        token_mint_signed(
            usdc_amount,
            token_program,
            conditional_on_fail_usdc_mint.as_ref(),
            conditional_on_fail_usdc_user_ata.as_ref(),
            proposal_vault.as_ref(),
            seeds,
        )?;
    }

    Ok(())
}
