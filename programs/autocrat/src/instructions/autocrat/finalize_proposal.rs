use std::borrow::Borrow;
use std::ops::Div;
use std::ops::Mul;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::instruction::Instruction;
use num_traits::ToPrimitive;

use crate::error::ErrorCode;
use crate::state::*;

#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(
        mut,
        has_one = pass_market_amm,
        has_one = fail_market_amm,
        has_one = instructions,
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        constraint = instructions.number == proposal.number,
        constraint = instructions.proposer == proposal.proposer
    )]
    pub instructions: Account<'info, ProposalInstructions>,
    pub dao: Box<Account<'info, Dao>>,
    /// CHECK: never read
    #[account(
        mut,
        seeds = [dao.key().as_ref()],
        bump = dao.treasury_pda_bump
    )]
    pub dao_treasury: UncheckedAccount<'info>,
    #[account(mut)]
    pub pass_market_amm: Account<'info, Amm>,
    #[account(mut)]
    pub fail_market_amm: Account<'info, Amm>,
}

pub fn handler(ctx: Context<FinalizeProposal>) -> Result<()> {
    let FinalizeProposal {
        proposal,
        instructions,
        dao,
        dao_treasury,
        pass_market_amm,
        fail_market_amm,
    } = ctx.accounts;

    let clock = Clock::get()?;

    require!(
            clock.slot >= proposal.slot_enqueued + dao.slots_per_proposal,
            ErrorCode::ProposalTooYoung
        );

    require!(
            proposal.state == ProposalState::Pending,
            ErrorCode::ProposalAlreadyFinalized
        );

    assert_ne!(pass_market_amm.ltwap_latest, 0f64);
    assert_ne!(fail_market_amm.ltwap_latest, 0f64);

    let treasury_seeds = &[dao.key().as_ref(), &[dao.treasury_pda_bump]];
    let signer = &[&treasury_seeds[..]];

    // f64 which is 1.05, or whatever (1.0 + %_to_pass) is
    let threshold_scale = BPS_SCALE
        .checked_add(dao.pass_threshold_bps).unwrap()
        .to_f64().unwrap()
        .div(BPS_SCALE.to_f64().unwrap());

    let threshold = fail_market_amm.ltwap_latest.mul(threshold_scale);

    if pass_market_amm.ltwap_latest > threshold {
        proposal.state = ProposalState::Passed;

        for ix in instructions.instructions.iter() {
            let mut svm_instruction: Instruction = ix.into();

            for acc in svm_instruction.accounts.iter_mut() {
                if &acc.pubkey == dao_treasury.key {
                    acc.is_signer = true;
                }
            }

            solana_program::program::invoke_signed(
                &svm_instruction,
                ctx.remaining_accounts,
                signer,
            )?;
        }
    } else {
        proposal.state = ProposalState::Failed;
    }

    Ok(())
}
