use anchor_lang::prelude::*;
use crate::error::ErrorCode;

pub use seeds::*;
pub use token::*;

pub mod seeds;
pub mod token;

use crate::state::*;

pub fn get_decimal_scale_f64(decimals: u8) -> Result<f64> {
    match decimals {
        0u8 => Ok(1f64),
        1u8 => Ok(10f64),
        2u8 => Ok(100f64),
        3u8 => Ok(1000f64),
        4u8 => Ok(10000f64),
        5u8 => Ok(100000f64),
        6u8 => Ok(1000000f64),
        7u8 => Ok(10000000f64),
        8u8 => Ok(100000000f64),
        9u8 => Ok(1000000000f64),
        10u8 => Ok(10000000000f64),
        11u8 => Ok(100000000000f64),
        12u8 => Ok(1000000000000f64),
        13u8 => Ok(10000000000000f64),
        14u8 => Ok(100000000000000f64),
        15u8 => Ok(1000000000000000f64),
        _ => {
            msg!("{:?}", decimals);
            err!(ErrorCode::DecimalScaleError)
        }
    }
}

// yeah.. not so sure about this
pub fn get_instructions_size(instructions: &Vec<ProposalInstruction>) -> usize {
    instructions.iter().fold(0, |acc, x| {
        acc + 
        &x.accounts.capacity() * 32 +   // 32 per account
        x.data.capacity() * 2 +         // 2 per u8, easy 'buffer' multiple
        32                              // 32 for program id
    })
}