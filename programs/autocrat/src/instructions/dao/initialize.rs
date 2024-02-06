use anchor_lang::prelude::*;
use anchor_spl::token::*;

use crate::state::*;

#[derive(Accounts)]
pub struct InitializeDAO<'info> {
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
    #[account(mint::decimals = 9)]
    pub meta_mint: Account<'info, Mint>,
    #[account(
        mint::decimals = 6,
        constraint = usdc_mint.key() == USDC_MINT
    )]
    pub usdc_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeDAO>) -> Result<()> {
    let dao = &mut ctx.accounts.dao;

    let (treasury_pubkey, treasury_bump) = Pubkey::find_program_address(&[dao.key().as_ref()], ctx.program_id);
    dao.treasury_pda_bump = treasury_bump;
    dao.treasury_pda = treasury_pubkey;

    dao.meta_mint = ctx.accounts.meta_mint.key();
    dao.usdc_mint = ctx.accounts.usdc_mint.key();

    dao.proposal_count = 4;

    dao.pass_threshold_bps = DEFAULT_PASS_THRESHOLD_BPS;

    dao.base_burn_lamports = DEFAULT_BASE_BURN_LAMPORTS;
    dao.burn_decay_per_slot_lamports = DEFAULT_BURN_DECAY_PER_SLOT_LAMPORTS;

    dao.slots_per_proposal = PROPOSAL_DURATION_IN_SLOTS;

    dao.amm_initial_quote_liquidity_atoms = AMM_INITIAL_QUOTE_LIQUIDITY_ATOMS;

    assert!(AMM_SWAP_FEE_BPS <= AMM_SWAP_FEE_BPS_MAX);
    assert!(AMM_SWAP_FEE_BPS >= AMM_SWAP_FEE_BPS_MIN);
    dao.amm_swap_fee_bps = AMM_SWAP_FEE_BPS;

    Ok(())
}
