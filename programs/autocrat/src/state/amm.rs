use std::ops::{Div, Mul};

use anchor_lang::prelude::*;
use num_traits::ToPrimitive;

use crate::error::ErrorCode;
use crate::{utils::*, AddLiquidity, RemoveLiquidity, Swap, BPS_SCALE};

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

    pub fn swap(&mut self, ctx: &Context<Swap>, is_quote_to_base: bool, input_amount: u64, can_ltwap_be_updated: bool) -> Result<u64> {
        let Swap {
            user,
            dao,
            amm,
            conditional_base_mint,
            conditional_quote_mint,
            user_ata_conditional_base,
            user_ata_conditional_quote,
            vault_ata_conditional_base,
            vault_ata_conditional_quote,
            token_program,
            associated_token_program: _,
            system_program: _
        } = ctx.accounts;

        assert!(input_amount > 0);

        if can_ltwap_be_updated {
            self.update_ltwap()?;
        }

        let conditional_base_amount_start = self.conditional_base_amount as u128;
        let conditional_quote_amount_start = self.conditional_quote_amount as u128;

        let k = conditional_base_amount_start.checked_mul(conditional_quote_amount_start).unwrap();

        let input_amount_minus_fee = input_amount
                .checked_mul(BPS_SCALE.checked_sub(dao.amm_swap_fee_bps).unwrap()).unwrap()
                .checked_div(BPS_SCALE).unwrap() as u128;
        
        let output_amount = if is_quote_to_base {
            let temp_conditional_quote_amount = conditional_quote_amount_start.checked_add(input_amount_minus_fee).unwrap();
            let temp_conditional_base_amount = k.checked_div(temp_conditional_quote_amount).unwrap();
            
            let output_amount_base = conditional_base_amount_start
                .checked_sub(temp_conditional_base_amount).unwrap()
                .to_u64().unwrap();

            self.conditional_quote_amount = self.conditional_quote_amount.checked_add(input_amount).unwrap();
            self.conditional_base_amount = self.conditional_base_amount.checked_sub(output_amount_base).unwrap();

            // send user quote tokens to vault
            token_transfer(
                input_amount,
                token_program,
                &user_ata_conditional_quote,
                &vault_ata_conditional_quote,
                &user,
            )?;

            // send vault base tokens to user
            token_transfer_signed(
                output_amount_base,
                token_program,
                &vault_ata_conditional_base,
                &user_ata_conditional_base,
                &xyz,
                seeds
            )?;

            output_amount_base
        } else {
            let temp_conditional_base_amount = conditional_base_amount_start.checked_add(input_amount_minus_fee).unwrap();
            let temp_conditional_quote_amount = k.checked_div(temp_conditional_base_amount).unwrap();
            
            let output_amount_quote = conditional_quote_amount_start
                .checked_sub(temp_conditional_quote_amount).unwrap()
                .to_u64().unwrap();

            self.conditional_base_amount = self.conditional_base_amount.checked_add(input_amount).unwrap();
            self.conditional_quote_amount = self.conditional_quote_amount.checked_sub(output_amount_quote).unwrap();

            // send user base tokens to vault
            token_transfer(
                input_amount,
                token_program,
                &user_ata_conditional_base,
                &vault_ata_conditional_base,
                &user,
            )?;

            // send vault quote tokens to user
            token_transfer_signed(
                output_amount_quote,
                token_program,
                &vault_ata_conditional_quote,
                &user_ata_conditional_quote,
                &xyz,
                seeds
            )?;

            output_amount_quote
        };

        Ok(output_amount)
    }

    pub fn add_liquidity(&self, ctx: &Context<AddLiquidity>, max_base_amount: u64, max_quote_amount: u64, can_ltwap_be_updated: bool) -> Result<()> {
        let AddLiquidity {
            user,
            dao,
            amm,
            amm_position,
            conditional_base_mint,
            conditional_quote_mint,
            user_ata_conditional_base,
            user_ata_conditional_quote,
            vault_ata_conditional_base,
            vault_ata_conditional_quote,
            token_program,
            associated_token_program: _,
            system_program: _
        } = ctx.accounts;

        assert!(max_base_amount > 0);
        assert!(max_quote_amount > 0);

        if can_ltwap_be_updated {
            self.update_ltwap()?;
        }

        if amm_position.ownership == 0u64 {
            amm.num_current_lps = amm.num_current_lps.checked_add(1).unwrap();
        }

        let mut temp_base_amount = max_base_amount as u128;

        let mut temp_quote_amount = temp_base_amount
            .checked_mul(self.conditional_quote_amount as u128).unwrap()
            .checked_div(self.conditional_base_amount as u128).unwrap();

        // if the temp_quote_amount calculation with max_base_amount led to a value higher than max_quote_amount,
        // then use the max_quote_amount and calculate in the other direction
        if temp_quote_amount > max_quote_amount as u128 {
            temp_quote_amount = max_quote_amount as u128;

            temp_base_amount = temp_quote_amount
                .checked_mul(self.conditional_base_amount as u128).unwrap()
                .checked_div(self.conditional_quote_amount as u128).unwrap();

            if temp_base_amount > max_base_amount as u128 {
                return err!(ErrorCode::AddLiquidityCalculationError);
            }
        }

        let additional_ownership = temp_base_amount
            .checked_mul(amm.total_ownership as u128).unwrap()
            .checked_div(self.conditional_base_amount as u128).unwrap()
            .to_u64().unwrap();

        amm_position.ownership = amm_position.ownership.checked_add(additional_ownership).unwrap();
        amm.total_ownership = amm.total_ownership.checked_add(additional_ownership).unwrap();

        // send user base tokens to vault
        token_transfer(
            temp_base_amount as u64,
            &token_program,
            &user_ata_conditional_base,
            &vault_ata_conditional_base,
            &user,
        )?;

        // send user quote tokens to vault
        token_transfer(
            temp_quote_amount as u64,
            &token_program,
            &user_ata_conditional_quote,
            &vault_ata_conditional_quote,
            &user,
        )?;

        Ok(())
    }

    pub fn remove_liquidity(&self, ctx: &Context<RemoveLiquidity>, remove_bps: u64, can_ltwap_be_updated: bool) -> Result<()> {
        let RemoveLiquidity {
            user,
            dao,
            proposal,
            amm,
            amm_position,
            conditional_base_mint,
            conditional_quote_mint,
            user_ata_conditional_base,
            user_ata_conditional_quote,
            vault_ata_conditional_base,
            vault_ata_conditional_quote,
            token_program,
            associated_token_program: _,
            system_program: _
        } = ctx.accounts;

        assert!(remove_bps > 0);
        assert!(remove_bps <= BPS_SCALE);

        if can_ltwap_be_updated {
            self.update_ltwap()?;
        }

        if can_ltwap_be_updated && user.key() == proposal.proposer {
            return err!(ErrorCode::ProposerCannotPullLiquidityWhileMarketIsPending);
        }

        if amm_position.ownership > 0 && remove_bps == BPS_SCALE {
            amm.num_current_lps = amm.num_current_lps.checked_sub(1).unwrap();
        }
        
        let base_to_withdraw = (self.conditional_base_amount as u128)
            .checked_mul(amm_position.ownership as u128).unwrap()
            .checked_mul(remove_bps as u128).unwrap()
            .checked_div(BPS_SCALE as u128).unwrap()
            .checked_div(amm.total_ownership as u128).unwrap()
            .to_u64().unwrap();

        let quote_to_withdraw = (self.conditional_quote_amount as u128)
            .checked_mul(amm_position.ownership as u128).unwrap()
            .checked_mul(remove_bps as u128).unwrap()
            .checked_div(BPS_SCALE as u128).unwrap()
            .checked_div(amm.total_ownership as u128).unwrap()
            .to_u64().unwrap();

        let less_ownership = (amm_position.ownership as u128)
            .checked_mul(remove_bps as u128).unwrap()
            .checked_div(BPS_SCALE as u128).unwrap()
            .to_u64().unwrap();

        amm_position.ownership = if remove_bps == BPS_SCALE { 0 } else { amm_position.ownership.checked_sub(less_ownership).unwrap() };
        amm.total_ownership = amm.total_ownership.checked_sub(less_ownership).unwrap();

        // send vault base tokens to user
        token_transfer_signed(
            base_to_withdraw,
            &token_program,
            &vault_ata_conditional_base,
            &user_ata_conditional_base,
            &xyz,
            seeds
        )?;

        // send vault quote tokens to user
        token_transfer_signed(
            quote_to_withdraw,
            &token_program,
            &vault_ata_conditional_quote,
            &user_ata_conditional_quote,
            &xyz,
            seeds
        )?;

        Ok(())
    }
    
 }