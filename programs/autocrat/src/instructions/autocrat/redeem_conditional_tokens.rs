use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::*;
use anchor_spl::token::Transfer;

use crate::error::ErrorCode;
use crate::state::*;

#[derive(Accounts)]
pub struct RedeemConditionalTokens<'info> {
    #[account(
        has_one = conditional_on_finalize_token_mint @ ErrorCode::InvalidConditionalTokenMint,
        has_one = conditional_on_revert_token_mint @ ErrorCode::InvalidConditionalTokenMint,
        constraint = vault.status != VaultStatus::Active @ ErrorCode::CantRedeemConditionalTokens
    )]
    pub vault: Account<'info, ConditionalVault>,
    #[account(mut)]
    pub conditional_on_finalize_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub conditional_on_revert_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = vault_underlying_token_account.key() == vault.underlying_token_account @ ErrorCode::InvalidVaultUnderlyingTokenAccount
    )]
    pub vault_underlying_token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
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
    #[account(
        mut,
        token::authority = authority,
        token::mint = vault.underlying_token_mint
    )]
    pub user_underlying_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn handle(
    ctx: Context<RedeemConditionalTokens>,
) -> Result<()> {
    let accs = &ctx.accounts;

    // storing some numbers for later invariant checks
    let pre_vault_underlying_balance = accs.vault_underlying_token_account.amount;
    let pre_finalize_mint_supply = accs.conditional_on_finalize_token_mint.supply;
    let pre_revert_mint_supply = accs.conditional_on_revert_token_mint.supply;

    let vault = &accs.vault;
    let vault_status = vault.status;

    let seeds = generate_vault_seeds!(vault);
    let signer = &[&seeds[..]];

    let conditional_on_finalize_balance =
        accs.user_conditional_on_finalize_token_account.amount;
    let conditional_on_revert_balance = accs.user_conditional_on_revert_token_account.amount;

    // burn everything for good measure
    burn(
        CpiContext::new(
            accs.token_program.to_account_info(),
            Burn {
                mint: accs.conditional_on_finalize_token_mint.to_account_info(),
                from: accs
                    .user_conditional_on_finalize_token_account
                    .to_account_info(),
                authority: accs.authority.to_account_info(),
            },
        ),
        conditional_on_finalize_balance,
    )?;

    burn(
        CpiContext::new(
            accs.token_program.to_account_info(),
            Burn {
                mint: accs.conditional_on_revert_token_mint.to_account_info(),
                from: accs
                    .user_conditional_on_revert_token_account
                    .to_account_info(),
                authority: accs.authority.to_account_info(),
            },
        ),
        conditional_on_revert_balance,
    )?;

    if vault_status == VaultStatus::Finalized {
        transfer(
            CpiContext::new_with_signer(
                accs.token_program.to_account_info(),
                Transfer {
                    from: accs.vault_underlying_token_account.to_account_info(),
                    to: accs.user_underlying_token_account.to_account_info(),
                    authority: accs.vault.to_account_info(),
                },
                signer,
            ),
            conditional_on_finalize_balance,
        )?;
    } else {
        transfer(
            CpiContext::new_with_signer(
                accs.token_program.to_account_info(),
                Transfer {
                    from: accs.vault_underlying_token_account.to_account_info(),
                    to: accs.user_underlying_token_account.to_account_info(),
                    authority: accs.vault.to_account_info(),
                },
                signer,
            ),
            conditional_on_revert_balance,
        )?;
    }

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

    assert!(post_user_conditional_on_finalize_balance == 0);
    assert!(post_user_conditional_on_revert_balance == 0);
    assert!(
        post_finalize_mint_supply == pre_finalize_mint_supply - conditional_on_finalize_balance
    );
    assert!(post_revert_mint_supply == pre_revert_mint_supply - conditional_on_revert_balance);
    if vault_status == VaultStatus::Finalized {
        assert!(
            post_vault_underlying_balance
                == pre_vault_underlying_balance - conditional_on_finalize_balance
        );
    } else {
        assert!(vault_status == VaultStatus::Reverted);
        assert!(
            post_vault_underlying_balance
                == pre_vault_underlying_balance - conditional_on_revert_balance
        );
    }

    Ok(())
}
