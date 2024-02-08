use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
pub struct UpdateDao<'info> {
    #[account(
        mut,
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"],
        bump
    )]
    pub dao: Account<'info, Dao>,
    #[account(
        signer,
        seeds = [dao.key().as_ref()],
        bump = dao.treasury_pda_bump,
    )]
    pub dao_treasury: Signer<'info>,
}

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct UpdateDaoParams {
    pub pass_threshold_bps: Option<u64>,
    pub base_burn_lamports: Option<u64>,
    pub burn_decay_per_slot_lamports: Option<u64>,
    pub slots_per_proposal: Option<u64>,
    pub amm_initial_quote_liquidity_atoms: Option<u64>,
    pub amm_swap_fee_bps: Option<u64>,
}

pub fn handler(
    ctx: Context<UpdateDao>,
    dao_params: UpdateDaoParams,
) -> Result<()> {
    let dao = &mut ctx.accounts.dao;

    if let Some(pass_threshold_bps) = dao_params.pass_threshold_bps {
        dao.pass_threshold_bps = pass_threshold_bps;
    }

    if let Some(base_burn_lamports) = dao_params.base_burn_lamports {
        dao.base_burn_lamports = base_burn_lamports;
    }

    if let Some(burn_decay_per_slot_lamports) = dao_params.burn_decay_per_slot_lamports {
        dao.burn_decay_per_slot_lamports = burn_decay_per_slot_lamports;
    }

    if let Some(slots_per_proposal) = dao_params.slots_per_proposal {
        dao.slots_per_proposal = slots_per_proposal;
    }

    if let Some(amm_initial_quote_liquidity_atoms) = dao_params.amm_initial_quote_liquidity_atoms {
        dao.amm_initial_quote_liquidity_atoms = amm_initial_quote_liquidity_atoms;
    }

    if let Some(amm_swap_fee_bps) = dao_params.amm_swap_fee_bps {
        dao.amm_swap_fee_bps = amm_swap_fee_bps;
    }

    Ok(())
}
