use std::ops::{Div, Mul};

use anchor_lang::prelude::*;
use num_traits::ToPrimitive;

use crate::error::ErrorCode;
use crate::generate_vault_seeds;
use crate::{utils::*, BPS_SCALE};

#[account]
pub struct Amm {
    pub conditional_base_mint: Pubkey,
    pub conditional_quote_mint: Pubkey,

    pub conditional_base_amount: u64,
    pub conditional_quote_amount: u64,

    pub conditional_base_mint_decimals: u8,
    pub conditional_quote_mint_decimals: u8,

    pub total_ownership: u64,
    pub num_current_lps: u64,

    // ltwap stands for: liquidity time weighted average price

    // running sum of: current_liquidity * slots_since_last_update
    pub ltwap_liquidity_duration_aggregator: u128,
    // running sum of: current_liquidity * slots_since_last_update * price
    pub ltwap_liquidity_duration_price_aggregator: u128,
    pub ltwap_latest: u128,
    pub ltwap_slot_updated: u64,
}

impl Amm {
    pub fn get_ltwap(&self) -> Result<u128> {
        if self.ltwap_liquidity_duration_aggregator == 0u128 {
            return Ok(0u128);
        }

        Ok(self
            .ltwap_liquidity_duration_price_aggregator
            .checked_div(self.ltwap_liquidity_duration_aggregator)
            .unwrap())
    }

    pub fn update_ltwap(&mut self) -> Result<u128> {
        let slot = Clock::get()?.slot;
        let slot_difference = slot.checked_sub(self.ltwap_slot_updated).unwrap() as u128;

        /*
            to calculate the liquidity of the whole pool, it would be:
                >> quote_units + price * base_units
                    or, when replacing the equation for price in an amm:
                >> quote_units + (quote_units / base_units) * base_units
                    which equals
                >> quote_units + quote_units = 2 * quote_units
                    so we can just use the quote_units instead, since this is a weighted average
        */
        let quote_liquidity_units = self.get_quote_liquidity_units()? as u128;
        let liquidity_x_slot_diff = quote_liquidity_units.checked_mul(slot_difference).unwrap();

        let base_liquidity_units = self.get_base_liquidity_units()? as u128;
        let price = if base_liquidity_units == 0u128 {
            0u128
        } else {
            quote_liquidity_units
                .checked_div(base_liquidity_units)
                .unwrap()
        };

        self.ltwap_liquidity_duration_aggregator += liquidity_x_slot_diff;
        self.ltwap_liquidity_duration_price_aggregator += liquidity_x_slot_diff.mul(price);

        if self.ltwap_liquidity_duration_aggregator != 0u128 {
            self.ltwap_latest = self
                .ltwap_liquidity_duration_price_aggregator
                .div(self.ltwap_liquidity_duration_aggregator);
        }

        self.ltwap_slot_updated = slot;

        Ok(self.ltwap_latest)
    }

    // get base liquidity units, with decimal resolution of 10^6
    pub fn get_base_liquidity_units(&self) -> Result<u64> {
        let base_decimal_scale = get_decimal_scale_u64(self.conditional_base_mint_decimals)?;
        Ok((self.conditional_base_amount)
            .checked_mul(1_000_000)
            .unwrap()
            .checked_div(base_decimal_scale)
            .unwrap())
    }

    // get quote liquidity units, with decimal resolution of 10^6
    pub fn get_quote_liquidity_units(&self) -> Result<u64> {
        let quote_decimal_scale = get_decimal_scale_u64(self.conditional_quote_mint_decimals)?;
        Ok((self.conditional_quote_amount)
            .checked_mul(1_000_000)
            .unwrap()
            .checked_div(quote_decimal_scale)
            .unwrap())
    }
}
