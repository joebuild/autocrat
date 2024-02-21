use anchor_lang::prelude::*;
use crate::error::ErrorCode;

pub use token::*;
pub use seeds::*;

pub mod token;
pub mod seeds;

use crate::state::*;

pub fn get_decimal_scale_u64(decimals: u8) -> Result<u64> {
    match decimals {
        0u8 => Ok(1u64),
        1u8 => Ok(10u64),
        2u8 => Ok(100u64),
        3u8 => Ok(1000u64),
        4u8 => Ok(10000u64),
        5u8 => Ok(100000u64),
        6u8 => Ok(1000000u64),
        7u8 => Ok(10000000u64),
        8u8 => Ok(100000000u64),
        9u8 => Ok(1000000000u64),
        10u8 => Ok(10000000000u64),
        11u8 => Ok(100000000000u64),
        12u8 => Ok(1000000000000u64),
        13u8 => Ok(10000000000000u64),
        14u8 => Ok(100000000000000u64),
        15u8 => Ok(1000000000000000u64),
        _ => {
            msg!("{:?}", decimals);
            err!(ErrorCode::DecimalScaleError)
        }
    }
}

pub fn get_instructions_size(instructions: &Vec<ProposalInstruction>) -> usize {
    instructions.iter().fold(0, |accumulator, ix| {
        accumulator + 
        32 + // program id
        4 + // accounts vec prefix
        ix.accounts.len() * (32 + 1 + 1) + // pubkey + 2 bools per account
        4 + // data vec prefix
        ix.data.len()
    })
}
