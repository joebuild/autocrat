use anchor_lang::prelude::*;

#[account]
pub struct Dao {
    // treasury needed even though DAO is PDA for this reason: https://solana.stackexchange.com/questions/7667/a-peculiar-problem-with-cpis
    pub treasury_pda_bump: u8,
    pub treasury_pda: Pubkey,

    pub meta_mint: Pubkey,
    pub usdc_mint: Pubkey,

    pub proposal_count: u32,
    pub last_proposal_slot: u64,

    // the percentage, in basis points, the pass price needs to be above the
    // fail price in order for the proposal to pass
    pub pass_threshold_bps: u64,

    // for anti-spam, proposers need to burn some SOL. the amount that they need
    // to burn is inversely proportional to the amount of time that has passed
    // since the last proposal.
    // burn_amount = base_lamport_burn - (lamport_burn_decay_per_slot * slots_passed)
    pub base_burn_lamports: u64,
    pub burn_decay_per_slot_lamports: u64,

    pub slots_per_proposal: u64,

    // atomic amount of quote liquidity to be deposited per market by the proposer
    pub amm_initial_quote_liquidity_atoms: u64,
    pub amm_swap_fee_bps: u64,
}
