use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::*;

use crate::state::*;

#[derive(Accounts)]
pub struct InitializeDao<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<Dao>(),
        // We will create a civilization of the Mind in Cyberspace. May it be
        // more humane and fair than the world your governments have made before.
        //  - John Perry Barlow, A Declaration of the Independence of Cyberspace
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Account<'info, Dao>,
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 1,
        seeds = [dao.key().as_ref()],
        bump
    )]
    pub dao_treasury: Account<'info, DaoTreasury>,
    #[account(mint::decimals = 9)]
    pub meta_mint: Account<'info, Mint>,
    #[account(mint::decimals = 6)]
    pub usdc_mint: Account<'info, Mint>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeDao>) -> Result<()> {
    let InitializeDao {
        payer: _,
        dao,
        dao_treasury,
        meta_mint,
        usdc_mint,
        associated_token_program: _,
        token_program: _,
        system_program: _,
    } = ctx.accounts;

    dao_treasury.dao = dao.key();
    dao_treasury.bump = ctx.bumps.dao_treasury;

    dao.treasury_pda_bump = ctx.bumps.dao_treasury;
    dao.treasury_pda = dao_treasury.key();

    dao.meta_mint = meta_mint.key();
    dao.usdc_mint = usdc_mint.key();

    dao.proposal_count = 10;

    dao.pass_threshold_bps = DEFAULT_PASS_THRESHOLD_BPS;

    dao.proposal_duration_slots = PROPOSAL_DURATION_SLOTS;
    dao.finalize_window_slots = FINALIZE_WINDOW_SLOTS;

    dao.amm_initial_quote_liquidity_amount = AMM_INITIAL_QUOTE_LIQUIDITY_ATOMS;

    assert!(AMM_SWAP_FEE_BPS <= AMM_SWAP_FEE_BPS_MAX);
    assert!(AMM_SWAP_FEE_BPS >= AMM_SWAP_FEE_BPS_MIN);
    dao.amm_swap_fee_bps = AMM_SWAP_FEE_BPS;

    dao.amm_ltwap_decimals = 9;

    Ok(())
}
