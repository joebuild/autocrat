use anchor_lang::prelude::*;

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "autocrat_v1",
    project_url: "https://themetadao.org",
    contacts: "email:metaproph3t@protonmail.com",
    policy: "The market will decide whether we pay a bug bounty.",
    source_code: "https://github.com/metaDAOproject/meta-dao",
    source_release: "v1",
    auditors: "None",
    acknowledgements: "DCF = (CF1 / (1 + r)^1) + (CF2 / (1 + r)^2) + ... (CFn / (1 + r)^n)"
}

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use crate::error::*;
use crate::instructions::*;
use crate::state::*;
use crate::utils::*;

declare_id!("66629qDqH5vJuz4ZgaL1HVpeAC9kJXnzamMpvMJfr3kE");

#[program]
pub mod autocrat {
    use super::*;

    // ==== dao
    pub fn initialize_dao(ctx: Context<InitializeDao>) -> Result<()> {
        instructions::dao::initialize::handler(ctx)
    }

    pub fn update_dao(ctx: Context<UpdateDao>, dao_params: UpdateDaoParams) -> Result<()> {
        instructions::dao::update::handler(ctx, dao_params)
    }

    // ==== autocrat
    pub fn create_proposal_instructions(ctx: Context<CreateProposalInstructions>, instructions: Vec<ProposalInstruction>) -> Result<()> {
        instructions::autocrat::create_proposal_instructions::handler(ctx, instructions)
    }

    pub fn add_proposal_instructions(ctx: Context<AddProposalInstructions>, instructions: Vec<ProposalInstruction>) -> Result<()> {
        instructions::autocrat::add_proposal_instructions::handler(ctx, instructions)
    }

    pub fn mint_conditional_tokens(ctx: Context<MintConditionalTokens>, meta_amount: u64, usdc_amount: u64) -> Result<()> {
        instructions::autocrat::mint_conditional_tokens::handler(ctx, meta_amount, usdc_amount)
    }

    pub fn redeem_conditional_tokens(ctx: Context<RedeemConditionalTokens>) -> Result<()> {
        instructions::autocrat::redeem_conditional_tokens::handler(ctx)
    }

    pub fn finalize_proposal(ctx: Context<FinalizeProposal>) -> Result<()> {
        instructions::autocrat::finalize_proposal::handler(ctx)
    }

    // ==== amm
    pub fn add_liquidity(ctx: Context<AddLiquidity>, max_base_amount: u64, max_quote_amount: u64, is_pass_market: bool) -> Result<()> {
        instructions::amm::add_liquidity::handler(ctx, max_base_amount, max_quote_amount, is_pass_market)
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, remove_bps: u64, is_pass_market: bool) -> Result<()> {
        instructions::amm::remove_liquidity::handler(ctx, remove_bps, is_pass_market)
    }

    pub fn swap(ctx: Context<Swap>, is_quote_to_base: bool, input_amount: u64, output_amount_min: u64, is_pass_market: bool) -> Result<()> {
        instructions::amm::swap::handler(ctx, is_quote_to_base, input_amount, output_amount_min, is_pass_market)
    }


}
