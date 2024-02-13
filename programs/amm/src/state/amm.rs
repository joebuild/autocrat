use std::ops::{Div, Mul};

use anchor_lang::prelude::*;
use num_traits::ToPrimitive;

use crate::error::ErrorCode;
use crate::{utils::*, BPS_SCALE};

#[account]
pub struct Amm {
    pub bump: u8,

    pub permissioned: bool,
    pub permissioned_caller: Pubkey,

    pub created_at_slot: u64,

    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,

    pub base_mint_decimals: u8,
    pub quote_mint_decimals: u8,

    pub base_amount: u64,
    pub quote_amount: u64,

    pub total_ownership: u64,

    pub swap_fee_bps: u32,

    // ltwap stands for: liquidity time weighted average price
    pub ltwap_slot_updated: u64,
    // running sum of: current_liquidity * slots_since_last_update
    pub ltwap_liquidity_duration_aggregator: u128,
    // running sum of: current_liquidity * slots_since_last_update * price
    pub ltwap_liquidity_duration_price_aggregator: u128,
    pub ltwap_decimals: u8,
    pub ltwap_latest: u128,
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

    // get base liquidity units, with decimal resolution of ltwap_decimals
    pub fn get_base_liquidity_units(&self) -> Result<u64> {
        let base_decimal_scale = get_decimal_scale_u64(self.base_mint_decimals)?;
        let ltwap_decimal_scale = get_decimal_scale_u64(self.ltwap_decimals)?;
        Ok((self.base_amount)
            .checked_mul(ltwap_decimal_scale)
            .unwrap()
            .checked_div(base_decimal_scale)
            .unwrap())
    }

    // get quote liquidity units, with decimal resolution of ltwap_decimals
    pub fn get_quote_liquidity_units(&self) -> Result<u64> {
        let quote_decimal_scale = get_decimal_scale_u64(self.quote_mint_decimals)?;
        let ltwap_decimal_scale = get_decimal_scale_u64(self.ltwap_decimals)?;
        Ok((self.quote_amount)
            .checked_mul(ltwap_decimal_scale)
            .unwrap()
            .checked_div(quote_decimal_scale)
            .unwrap())
    }
}
