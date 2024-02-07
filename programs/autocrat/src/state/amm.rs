use std::ops::{Div, Mul};

use anchor_lang::prelude::*;
use num_traits::ToPrimitive;

use crate::error::ErrorCode;
use crate::{utils::*, BPS_SCALE};
use crate::generate_vault_seeds;

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
    pub ltwap_liquidity_duration_aggregator: f64,          // running sum of: current_liquidity * slots_since_last_update
    pub ltwap_liquidity_duration_price_aggregator: f64,    // running sum of: current_liquidity * slots_since_last_update * price
    pub ltwap_latest: f64,
    pub ltwap_slot_updated: u64,
}

impl Amm {
    pub fn get_ltwap(&self) -> Result<f64> {
        if self.ltwap_liquidity_duration_aggregator == 0f64 {
            return Ok(0f64)
        }

        Ok(self.ltwap_liquidity_duration_price_aggregator.div(self.ltwap_liquidity_duration_aggregator))
    }

    pub fn update_ltwap(&mut self) -> Result<f64> {
        let slot = Clock::get()?.slot;
        let slot_difference = slot.checked_sub(self.ltwap_slot_updated).unwrap();

        /* 
            to calculate the liquidity of the whole pool, it would be:
                >> quote_units + price * base_units
                    or, when replacing the equation for price in an amm:
                >> quote_units + (quote_units / base_units) * base_units
                    which equals
                >> quote_units + quote_units = 2 * quote_units
                    so we can just use the quote_units instead, since this is a weighted average
        */
        let quote_liquidity_units = self.get_quote_liquidity_units()?;
        let liquidity_x_slot_diff = quote_liquidity_units.mul(slot_difference.to_f64().unwrap());

        let base_liquidity_units = self.get_base_liquidity_units()?;
        let price = quote_liquidity_units.div(base_liquidity_units);

        self.ltwap_liquidity_duration_aggregator += liquidity_x_slot_diff;
        self.ltwap_liquidity_duration_price_aggregator += liquidity_x_slot_diff.mul(price);

        self.ltwap_latest = self.ltwap_liquidity_duration_price_aggregator.div(self.ltwap_liquidity_duration_aggregator);

        self.ltwap_slot_updated = slot;

        Ok(self.ltwap_latest)
    }

    pub fn get_base_liquidity_units(&self) -> Result<f64> {
        let base_decimal_scale = get_decimal_scale_f64(self.conditional_base_mint_decimals)?;
        Ok(
            self.conditional_base_amount.to_f64().unwrap()
                .div(base_decimal_scale)
        )
    }

    pub fn get_quote_liquidity_units(&self) -> Result<f64> {
        let quote_decimal_scale = get_decimal_scale_f64(self.conditional_quote_mint_decimals)?;
        Ok(
            self.conditional_quote_amount.to_f64().unwrap()
                .div(quote_decimal_scale))
    }
 }