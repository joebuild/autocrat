use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use solana_program::native_token::LAMPORTS_PER_SOL;

use crate::error::ErrorCode;
use crate::generate_vault_seeds;
use crate::state::*;

#[derive(Accounts)]
pub struct CreateProposalPartTwo<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"proposal",
            proposal.number.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        mut,
        seeds = [
            b"pass_market_amm",
            proposal.number.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub pass_market_amm: Box<Account<'info, Amm>>,
    #[account(
        mut,
        seeds = [
            b"fail_market_amm",
            proposal.number.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub fail_market_amm: Box<Account<'info, Amm>>,
    #[account(
        mut,
        seeds = [
            pass_market_amm.key().as_ref(),
            proposer.key().as_ref(),
        ],
        bump
    )]
    pub pass_market_amm_position: Box<Account<'info, AmmPosition>>,
    #[account(
        mut,
        seeds = [
            fail_market_amm.key().as_ref(),
            proposer.key().as_ref(),
        ],
        bump
    )]
    pub fail_market_amm_position: Box<Account<'info, AmmPosition>>,
    #[account(
        constraint = meta_mint.key() == proposal.meta_mint.key()
    )]
    pub meta_mint: Box<Account<'info, Mint>>,
    #[account(
        constraint = usdc_mint.key() == proposal.usdc_mint.key()
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            b"conditional_on_pass_meta",
            proposal.number.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub conditional_on_pass_meta_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            b"conditional_on_pass_usdc",
            proposal.number.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub conditional_on_pass_usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            b"conditional_on_fail_meta",
            proposal.number.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub conditional_on_fail_meta_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            b"conditional_on_fail_usdc",
            proposal.number.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub conditional_on_fail_usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = meta_mint,
        associated_token::authority = proposer,
    )]
    pub meta_proposer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = proposer,
    )]
    pub usdc_proposer_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = meta_mint,
        associated_token::authority = proposal,
    )]
    pub meta_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = proposal,
    )]
    pub usdc_vault_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_on_pass_meta_mint,
        associated_token::authority = proposer,
    )]
    pub conditional_on_pass_meta_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_on_pass_usdc_mint,
        associated_token::authority = proposer,
    )]
    pub conditional_on_pass_usdc_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_on_fail_meta_mint,
        associated_token::authority = proposer,
    )]
    pub conditional_on_fail_meta_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = conditional_on_fail_usdc_mint,
        associated_token::authority = proposer,
    )]
    pub conditional_on_fail_usdc_user_ata: Box<Account<'info, TokenAccount>>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateProposalPartTwo>,
    initial_pass_market_price_quote_units_per_base_unit_bps: u64,
    initial_fail_market_price_quote_units_per_base_unit_bps: u64,
    quote_liquidity_amount_per_amm: u64,
) -> Result<()> {
    let CreateProposalPartTwo {
        proposer,
        proposal,
        pass_market_amm,
        fail_market_amm,
        pass_market_amm_position,
        fail_market_amm_position,
        meta_mint,
        usdc_mint,
        conditional_on_pass_meta_mint,
        conditional_on_pass_usdc_mint,
        conditional_on_fail_meta_mint,
        conditional_on_fail_usdc_mint,
        meta_proposer_ata,
        usdc_proposer_ata,
        meta_vault_ata,
        usdc_vault_ata,
        conditional_on_pass_meta_user_ata,
        conditional_on_pass_usdc_user_ata,
        conditional_on_fail_meta_user_ata,
        conditional_on_fail_usdc_user_ata,
        associated_token_program: _,
        token_program: _,
        system_program: _,
    } = ctx.accounts;

    assert!(proposal.part_one_complete);
    assert!(!proposal.part_two_complete);
    proposal.part_two_complete = true;

    let clock = Clock::get()?;
    proposal.slot_enqueued = clock.slot;

    pass_market_amm.ltwap_slot_updated = clock.slot;
    fail_market_amm.ltwap_slot_updated = clock.slot;

    // recoup anti-spam measure of 1 SOL (sent in part one)
    proposal.sub_lamports(LAMPORTS_PER_SOL)?;
    proposer.add_lamports(LAMPORTS_PER_SOL)?;

    // ======== deposit initial liquidity to amms ========

    // ======== pass market amm ========

    // assert_eq!(proposal.pass_market_amm, pass_market_amm.key());
    // assert_eq!(
    //     proposal.conditional_on_pass_meta_mint,
    //     conditional_on_pass_meta_mint.key()
    // );
    // assert_eq!(
    //     proposal.conditional_on_pass_usdc_mint,
    //     conditional_on_pass_usdc_mint.key()
    // );

    // pass_market_amm.num_current_lps = 1;

    // let mut temp_base_amount = max_base_amount as u128;
    // let mut temp_quote_amount = max_quote_amount as u128;

    // // use the higher number for ownership, to reduce rounding errors
    // let max_base_or_quote_amount = std::cmp::max(temp_base_amount, temp_quote_amount);

    // amm_position.ownership = max_base_or_quote_amount.to_u64().unwrap();
    // amm.total_ownership = max_base_or_quote_amount.to_u64().unwrap();

    // amm.conditional_base_amount = amm
    //     .conditional_base_amount
    //     .checked_add(temp_base_amount.to_u64().unwrap())
    //     .unwrap();

    // amm.conditional_quote_amount = amm
    //     .conditional_quote_amount
    //     .checked_add(temp_quote_amount.to_u64().unwrap())
    //     .unwrap();

    // // send user base tokens to vault
    // token_transfer(
    //     temp_base_amount as u64,
    //     &token_program,
    //     user_ata_conditional_base,
    //     vault_ata_conditional_base,
    //     user,
    // )?;

    // // send user quote tokens to vault
    // token_transfer(
    //     temp_quote_amount as u64,
    //     token_program,
    //     user_ata_conditional_quote,
    //     vault_ata_conditional_quote,
    //     user,
    // )?;

    Ok(())
}
