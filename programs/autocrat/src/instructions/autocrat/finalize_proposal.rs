use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::instruction::Instruction;

use amm::state::Amm;

use crate::error::ErrorCode;
use crate::state::*;

#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        has_one = pass_market_amm,
        has_one = fail_market_amm,
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        constraint = proposal_instructions.key() == proposal.instructions
    )]
    pub proposal_instructions: Account<'info, ProposalInstructions>,
    pub dao: Box<Account<'info, Dao>>,
    /// CHECK: never read
    #[account(
        signer,
        mut,
        seeds = [dao.key().as_ref()],
        bump = dao.treasury_pda_bump
    )]
    pub dao_treasury: UncheckedAccount<'info>,
    #[account(mut)]
    pub pass_market_amm: Account<'info, Amm>,
    #[account(mut)]
    pub fail_market_amm: Account<'info, Amm>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<FinalizeProposal>) -> Result<()> {
    let FinalizeProposal {
        user: _,
        proposal,
        proposal_instructions,
        dao,
        dao_treasury,
        pass_market_amm,
        fail_market_amm,
        system_program: _,
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

    let dao_pubkey = dao.key();
    let treasury_seeds = &[dao_pubkey.as_ref(), &[dao.treasury_pda_bump]];
    let signer = &[&treasury_seeds[..]];

    let threshold = (fail_market_amm.ltwap_latest as u128)
        .checked_mul(BPS_SCALE.checked_add(dao.pass_threshold_bps).unwrap() as u128)
        .unwrap()
        .checked_div(BPS_SCALE as u128)
        .unwrap();

    if (pass_market_amm.ltwap_latest as u128) > threshold {
        proposal.state = ProposalState::Passed;

        for ix in proposal_instructions.instructions.iter() {
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
