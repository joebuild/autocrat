use anchor_lang::prelude::*;

#[account]
pub struct Dao {
    // treasury needed even though DAO is PDA for this reason: https://solana.stackexchange.com/questions/7667/a-peculiar-problem-with-cpis
    pub treasury_pda_bump: u8,
    pub treasury_pda: Pubkey,

    pub meta_mint: Pubkey,
    pub usdc_mint: Pubkey,

    pub proposal_count: u64,

    // the percentage, in basis points, the pass price needs to be above the
    // fail price in order for the proposal to pass
    pub pass_threshold_bps: u64,

    pub proposal_duration_slots: u64,
    pub finalize_window_slots: u64,

    // amm
    pub amm_initial_quote_liquidity_amount: u64, // amount of quote liquidity to be deposited per market by the proposer
    pub amm_swap_fee_bps: u64,
    pub amm_ltwap_decimals: u8,
}
