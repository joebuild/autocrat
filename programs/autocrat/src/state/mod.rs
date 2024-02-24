use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

pub use dao::*;
pub use dao_treasury::*;
pub use proposal::*;
pub use proposal_vault::*;

pub mod dao;
pub mod dao_treasury;
pub mod proposal;
pub mod proposal_vault;

pub const SLOTS_PER_10_SECS: u64 = 25;
pub const PROPOSAL_DURATION_SLOTS: u64 = 3 * 24 * 60 * 6 * SLOTS_PER_10_SECS;
pub const FINALIZE_WINDOW_SLOTS: u64 = 1 * 24 * 60 * 6 * SLOTS_PER_10_SECS;

// by default, the pass price needs to be 5% higher than the fail price
pub const DEFAULT_PASS_THRESHOLD_BPS: u64 = 500;

// start at 10 SOL ($1000 at current prices), decay by ~5 SOL per day
pub const DEFAULT_BASE_BURN_LAMPORTS: u64 = 10 * LAMPORTS_PER_SOL;
pub const DEFAULT_BURN_DECAY_PER_SLOT_LAMPORTS: u64 = 23_150;

pub const AMM_INITIAL_QUOTE_LIQUIDITY_ATOMS: u64 = 1000 * 1_000_000; // $1000 * 10^6

pub const AMM_SWAP_FEE_BPS: u64 = 300; // 3%
pub const AMM_SWAP_FEE_BPS_MIN: u64 = 100; // 1%
pub const AMM_SWAP_FEE_BPS_MAX: u64 = 1000; // 10%

pub const BPS_SCALE: u64 = 100 * 100;
