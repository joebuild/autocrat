use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::*;

use crate::error::ErrorCode;
use crate::generate_vault_seeds;
use crate::state::*;
use crate::utils::token::*;

#[derive(Accounts)]
pub struct RedeemConditionalTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Box<Account<'info, Dao>>,
    #[account(
        zero,
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

pub fn handler(ctx: Context<RedeemConditionalTokens>) -> Result<()> {
    let RedeemConditionalTokens {
        user,
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

    let c_pass_meta_user_balance = conditional_on_pass_meta_user_ata.amount;
    let c_pass_usdc_user_balance = conditional_on_pass_usdc_user_ata.amount;
    let c_fail_meta_user_balance = conditional_on_fail_meta_user_ata.amount;
    let c_fail_usdc_user_balance = conditional_on_fail_usdc_user_ata.amount;

    let proposal_state = proposal.state;

    require!(
        proposal_state != ProposalState::Pending,
        ErrorCode::ProposalStillPending
    );

    let proposal_number_bytes = proposal.number.to_le_bytes();
    let seeds = generate_vault_seeds!(proposal_number_bytes, ctx.bumps.proposal);

    token_burn(
        c_pass_meta_user_balance,
        token_program,
        conditional_on_pass_meta_mint,
        conditional_on_pass_meta_user_ata,
        user,
    )?;

    token_burn(
        c_pass_usdc_user_balance,
        token_program,
        conditional_on_pass_usdc_mint,
        conditional_on_pass_usdc_user_ata,
        user,
    )?;

    token_burn(
        c_fail_meta_user_balance,
        token_program,
        conditional_on_fail_meta_mint,
        conditional_on_fail_meta_user_ata,
        user,
    )?;

    token_burn(
        c_fail_usdc_user_balance,
        token_program,
        conditional_on_fail_usdc_mint,
        conditional_on_fail_usdc_user_ata,
        user,
    )?;

    if proposal_state == ProposalState::Passed {
        token_transfer_signed(
            c_pass_meta_user_balance,
            token_program,
            meta_vault_ata,
            meta_user_ata,
            proposal.as_ref(),
            seeds,
        )?;

        token_transfer_signed(
            c_pass_usdc_user_balance,
            token_program,
            usdc_vault_ata,
            usdc_user_ata,
            proposal.as_ref(),
            seeds,
        )?;
    } else {
        token_transfer_signed(
            c_fail_meta_user_balance,
            token_program,
            meta_vault_ata,
            meta_user_ata,
            proposal.as_ref(),
            seeds,
        )?;

        token_transfer_signed(
            c_pass_usdc_user_balance,
            token_program,
            usdc_vault_ata,
            usdc_user_ata,
            proposal.as_ref(),
            seeds,
        )?;
    }

    Ok(())
}
