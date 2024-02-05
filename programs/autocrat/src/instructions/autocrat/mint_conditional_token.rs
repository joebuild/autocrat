use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::*;
use anchor_spl::token::Transfer;

use crate::error::ErrorCode;
use crate::state::*;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct MintConditionalTokens<'info> {
    pub authority: Signer<'info>,
    #[account(
        has_one = conditional_on_finalize_token_mint @ ErrorCode::InvalidConditionalTokenMint,
        has_one = conditional_on_revert_token_mint @ ErrorCode::InvalidConditionalTokenMint,
    )]
    pub vault: Account<'info, ConditionalVault>,
    #[account(mut)]
    pub conditional_on_finalize_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub conditional_on_revert_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = vault_underlying_token_account.key() == vault.underlying_token_account @  ErrorCode::InvalidVaultUnderlyingTokenAccount
    )]
    pub vault_underlying_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::authority = authority,
        token::mint = vault.underlying_token_mint,
        constraint = user_underlying_token_account.amount >= amount @ ErrorCode::InsufficientUnderlyingTokens
    )]
    pub user_underlying_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::authority = authority,
        token::mint = conditional_on_finalize_token_mint
    )]
    pub user_conditional_on_finalize_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::authority = authority,
        token::mint = conditional_on_revert_token_mint
    )]
    pub user_conditional_on_revert_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn handle(ctx: Context<MintConditionalTokens>, amount: u64) -> Result<()> {
    let accs = &ctx.accounts;

    let pre_user_conditional_on_finalize_balance = accs.user_conditional_on_finalize_token_account.amount;
    let pre_user_conditional_on_revert_balance = accs.user_conditional_on_revert_token_account.amount;
    let pre_vault_underlying_balance = accs.vault_underlying_token_account.amount;
    let pre_finalize_mint_supply = accs.conditional_on_finalize_token_mint.supply;
    let pre_revert_mint_supply = accs.conditional_on_revert_token_mint.supply;

    let vault = &accs.vault;

    let seeds = generate_vault_seeds!(vault);
    let signer = &[&seeds[..]];

    transfer(
        CpiContext::new(
            accs.token_program.to_account_info(),
            Transfer {
                from: accs.user_underlying_token_account.to_account_info(),
                to: accs.vault_underlying_token_account.to_account_info(),
                authority: accs.authority.to_account_info(),
            },
        ),
        amount,
    )?;

    mint_to(
        CpiContext::new_with_signer(
            accs.token_program.to_account_info(),
            MintTo {
                mint: accs.conditional_on_finalize_token_mint.to_account_info(),
                to: accs
                    .user_conditional_on_finalize_token_account
                    .to_account_info(),
                authority: accs.vault.to_account_info(),
            },
            signer,
        ),
        amount,
    )?;

    mint_to(
        CpiContext::new_with_signer(
            accs.token_program.to_account_info(),
            MintTo {
                mint: accs.conditional_on_revert_token_mint.to_account_info(),
                to: accs
                    .user_conditional_on_revert_token_account
                    .to_account_info(),
                authority: accs.vault.to_account_info(),
            },
            signer,
        ),
        amount,
    )?;

    ctx.accounts
        .user_conditional_on_finalize_token_account
        .reload()?;
    ctx.accounts
        .user_conditional_on_revert_token_account
        .reload()?;
    ctx.accounts.vault_underlying_token_account.reload()?;
    ctx.accounts.conditional_on_finalize_token_mint.reload()?;
    ctx.accounts.conditional_on_revert_token_mint.reload()?;

    let post_user_conditional_on_finalize_balance = ctx
        .accounts
        .user_conditional_on_finalize_token_account
        .amount;
    let post_user_conditional_on_revert_balance =
        ctx.accounts.user_conditional_on_revert_token_account.amount;
    let post_vault_underlying_balance = ctx.accounts.vault_underlying_token_account.amount;
    let post_finalize_mint_supply = ctx.accounts.conditional_on_finalize_token_mint.supply;
    let post_revert_mint_supply = ctx.accounts.conditional_on_revert_token_mint.supply;

    // Only the paranoid survive ;)
    assert_eq!(post_vault_underlying_balance, pre_vault_underlying_balance + amount);
    assert_eq!(post_user_conditional_on_finalize_balance, pre_user_conditional_on_finalize_balance + amount);
    assert_eq!(post_user_conditional_on_revert_balance, pre_user_conditional_on_revert_balance + amount);
    assert_eq!(post_finalize_mint_supply, pre_finalize_mint_supply + amount);
    assert_eq!(post_revert_mint_supply, pre_revert_mint_supply + amount);

    Ok(())
}
