use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
pub struct UpdateDao<'info> {
    #[account(
        mut,
        seeds = [dao.id.as_ref()],
        bump
    )]
    pub dao: Account<'info, Dao>,
    #[account(
        signer,
        seeds = [DAO_TREASURY_SEED_PREFIX, dao.key().as_ref()],
        bump = dao.treasury_pda_bump,
    )]
    pub dao_treasury: Signer<'info>,
}

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct UpdateDaoParams {
    pub pass_threshold_bps: Option<u64>,
    pub proposal_duration_slots: Option<u64>,
    pub finalize_window_slots: Option<u64>,
    pub proposal_fee_usdc: Option<u64>,
    pub amm_initial_quote_liquidity_amount: Option<u64>,
    pub amm_swap_fee_bps: Option<u64>,
    pub amm_ltwap_decimals: Option<u8>,
}

pub fn handler(ctx: Context<UpdateDao>, dao_params: UpdateDaoParams) -> Result<()> {
    let dao = &mut ctx.accounts.dao;

    if let Some(pass_threshold_bps) = dao_params.pass_threshold_bps {
        dao.pass_threshold_bps = pass_threshold_bps;
    }

    if let Some(proposal_duration_slots) = dao_params.proposal_duration_slots {
        dao.proposal_duration_slots = proposal_duration_slots;
    }

    if let Some(finalize_window_slots) = dao_params.finalize_window_slots {
        dao.finalize_window_slots = finalize_window_slots;
    }

    if let Some(proposal_fee_usdc) = dao_params.proposal_fee_usdc {
        dao.proposal_fee_usdc = proposal_fee_usdc;
    }

    if let Some(amm_initial_quote_liquidity_amount) = dao_params.amm_initial_quote_liquidity_amount
    {
        dao.amm_initial_quote_liquidity_amount = amm_initial_quote_liquidity_amount;
    }

    if let Some(amm_swap_fee_bps) = dao_params.amm_swap_fee_bps {
        dao.amm_swap_fee_bps = amm_swap_fee_bps;
    }

    if let Some(amm_ltwap_decimals) = dao_params.amm_ltwap_decimals {
        dao.amm_ltwap_decimals = amm_ltwap_decimals;
    }

    Ok(())
}
