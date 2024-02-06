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

// use crate::error::*;
use crate::instructions::*;
use crate::state::*;
use crate::utils::*;

declare_id!("66629qDqH5vJuz4ZgaL1HVpeAC9kJXnzamMpvMJfr3kE");

#[program]
pub mod autocrat {
    use super::*;

    pub fn initialize_dao(ctx: Context<InitializeDAO>) -> Result<()> {
        instructions::dao::initialize::handler(ctx)
    }

    // pub fn update_dao(ctx: Context<UpdateDAO>, dao_params: UpdateDaoParams) -> Result<()> {
    //     update_dao::handler(ctx, dao_params)
    // }

    // pub fn initialize_proposal(ctx: Context<InitializeProposal>, description_url: string, instruction: ProposalInstruction) -> Result<()> {
    //     initialize_proposal::handler(ctx, description_url, instruction)
    // }

    // pub fn mint_conditional_tokens(ctx: Context<MintConditionalTokens>) -> Result<()> {
    //     mint_conditional_tokens::handler(ctx)
    // }

    // pub fn redeem_conditional_for_underlying(ctx: Context<MintConditionalTokens>) -> Result<()> {
    //     redeem_conditional_tokens::handler(ctx)
    // }

}
