use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::*;

use crate::error::ErrorCode;
use crate::state::*;
use crate::utils::token::*;
use crate::generate_vault_seeds;

#[derive(Accounts)]
pub struct MintConditionalTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    #[account(
        has_one = conditional_on_pass_meta_mint,
        has_one = conditional_on_pass_usdc_mint,
        has_one = conditional_on_fail_meta_mint,
        has_one = conditional_on_fail_usdc_mint,
        seeds = [
            b"proposal",
            proposal.number.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        constraint = meta_mint.key() == dao.meta_mint.key()
    )]
    pub meta_mint: Box<Account<'info, Mint>>,
    #[account(
        constraint = usdc_mint.key() == dao.usdc_mint.key()
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub conditional_on_pass_meta_mint: Account<'info, Mint>,
    #[account(mut)]
    pub conditional_on_pass_usdc_mint: Account<'info, Mint>,
    #[account(mut)]
    pub conditional_on_fail_meta_mint: Account<'info, Mint>,
    #[account(mut)]
    pub conditional_on_fail_usdc_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = meta_mint,
        associated_token::authority = user,
    )]
    pub meta_user_ata: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = usdc_mint,
        associated_token::authority = user,
    )]
    pub usdc_user_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = conditional_on_pass_meta_mint,
        associated_token::authority = user,
    )]
    pub conditional_on_pass_meta_user_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = conditional_on_pass_usdc_mint,
        associated_token::authority = user,
    )]
    pub conditional_on_pass_usdc_user_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = conditional_on_fail_meta_mint,
        associated_token::authority = user,
    )]
    pub conditional_on_fail_meta_user_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = conditional_on_fail_usdc_mint,
        associated_token::authority = user,
    )]
    pub conditional_on_fail_usdc_user_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = meta_mint.key(),
        associated_token::authority = proposal,
    )]
    pub meta_vault_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = usdc_mint.key(),
        associated_token::authority = proposal,
    )]
    pub usdc_vault_ata: Account<'info, TokenAccount>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<MintConditionalTokens>, meta_amount: u64, usdc_amount: u64) -> Result<()> {
    let MintConditionalTokens {
        user: _,
        dao: _,
        proposal,
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

    let proposal_number_bytes = proposal.number.to_le_bytes();
    let seeds = generate_vault_seeds!(proposal_number_bytes, ctx.bumps.proposal);

    if meta_amount > 0 {
        // transfer user meta to vault
        token_transfer_signed(
            meta_amount,
            token_program,
            meta_user_ata,
            meta_vault_ata,
            proposal.as_ref(),
            seeds,
        )?;

        // mint conditional on-pass meta to user
        token_mint_signed(
            meta_amount,
            token_program,
            conditional_on_pass_meta_mint,
            conditional_on_pass_meta_user_ata,
            proposal.as_ref(),
            seeds,
        )?;

        // mint conditional on-fail meta to user
        token_mint_signed(
            meta_amount,
            token_program,
            conditional_on_fail_meta_mint,
            conditional_on_fail_meta_user_ata,
            proposal.as_ref(),
            seeds,
        )?;
    }

    if usdc_amount > 0 {
        // transfer user usdc to vault
        token_transfer_signed(
            usdc_amount,
            token_program,
            usdc_user_ata,
            usdc_vault_ata,
            proposal.as_ref(),
            seeds,
        )?;

        // mint conditional on-pass usdc to user
        token_mint_signed(
            usdc_amount,
            token_program,
            conditional_on_pass_usdc_mint,
            conditional_on_pass_usdc_user_ata,
            proposal.as_ref(),
            seeds,
        )?;

        // mint conditional on-fail usdc to user
        token_mint_signed(
            meta_amount,
            token_program,
            conditional_on_fail_usdc_mint,
            conditional_on_fail_usdc_user_ata,
            proposal.as_ref(),
            seeds,
        )?;
    }

    Ok(())
}
