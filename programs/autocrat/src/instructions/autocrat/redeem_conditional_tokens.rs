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
pub struct RedeemConditionalTokens<'info> {
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
            PROPOSAL_SEED_PREFIX,
            proposal.dao.as_ref(),
            proposal.number.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        mut,
        has_one = meta_vault_ata,
        has_one = usdc_vault_ata,
        seeds = [
            PROPOSAL_VAULT_SEED_PREFIX,
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

pub fn handler(ctx: Context<RedeemConditionalTokens>) -> Result<()> {
    let RedeemConditionalTokens {
        user,
        proposal,
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

    let c_pass_meta_user_balance = conditional_on_pass_meta_user_ata.amount;
    let c_pass_usdc_user_balance = conditional_on_pass_usdc_user_ata.amount;
    let c_fail_meta_user_balance = conditional_on_fail_meta_user_ata.amount;
    let c_fail_usdc_user_balance = conditional_on_fail_usdc_user_ata.amount;

    let proposal_state = proposal.state;

    require!(
        proposal_state == ProposalState::Passed || proposal_state == ProposalState::Failed,
        ErrorCode::ProposalStillPending
    );

    let proposal_key = proposal.key();
    let seeds = generate_proposal_vault_seeds!(proposal_key, ctx.bumps.proposal_vault);

    token_burn(
        c_pass_meta_user_balance,
        token_program,
        conditional_on_pass_meta_mint.as_ref(),
        conditional_on_pass_meta_user_ata.as_ref(),
        user,
    )?;

    token_burn(
        c_pass_usdc_user_balance,
        token_program,
        conditional_on_pass_usdc_mint.as_ref(),
        conditional_on_pass_usdc_user_ata.as_ref(),
        user,
    )?;

    token_burn(
        c_fail_meta_user_balance,
        token_program,
        conditional_on_fail_meta_mint.as_ref(),
        conditional_on_fail_meta_user_ata.as_ref(),
        user,
    )?;

    token_burn(
        c_fail_usdc_user_balance,
        token_program,
        conditional_on_fail_usdc_mint.as_ref(),
        conditional_on_fail_usdc_user_ata.as_ref(),
        user,
    )?;

    if proposal_state == ProposalState::Passed {
        token_transfer_signed(
            c_pass_meta_user_balance,
            token_program,
            meta_vault_ata.as_ref(),
            meta_user_ata.as_ref(),
            proposal_vault.as_ref(),
            seeds,
        )?;

        token_transfer_signed(
            c_pass_usdc_user_balance,
            token_program,
            usdc_vault_ata.as_ref(),
            usdc_user_ata.as_ref(),
            proposal_vault.as_ref(),
            seeds,
        )?;
    } else if proposal_state == ProposalState::Failed {
        token_transfer_signed(
            c_fail_meta_user_balance,
            token_program,
            meta_vault_ata.as_ref(),
            meta_user_ata.as_ref(),
            proposal_vault.as_ref(),
            seeds,
        )?;

        token_transfer_signed(
            c_pass_usdc_user_balance,
            token_program,
            usdc_vault_ata.as_ref(),
            usdc_user_ata.as_ref(),
            proposal_vault.as_ref(),
            seeds,
        )?;
    } else {
        return err!(ErrorCode::ProposalStillPending); // redundant, for clarity
    }

    Ok(())
}
