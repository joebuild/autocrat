use anchor_lang::prelude::*;

use bytemuck::Pod;
use bytemuck::Zeroable;
use sokoban::*;

use crate::error::ErrorCode;
use crate::settlement;
use crate::state::*;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct OrderKey {
    pub quote_lots_per_base_lot: u64,
    pub order_number: u64,
}

unsafe impl Zeroable for OrderKey {}
unsafe impl Pod for OrderKey {}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Order {
    pub user: Pubkey,
    pub base_lots: u64,
}

unsafe impl Zeroable for Order {}
unsafe impl Pod for Order {}

const MAX_SIZE: usize = 512;

#[account(zero_copy)]
pub struct Orders {
    pub hybrid: Pubkey,
    pub quote_lot_size: u64,
    pub base_lot_size: u64,
    pub min_order_size_base_lots: u64,
    pub bid_tree: RedBlackTree<OrderKey, Order, MAX_SIZE>,
    pub ask_tree: RedBlackTree<OrderKey, Order, MAX_SIZE>,
    pub order_number: u64,
}

impl Orders {
    pub fn add_order(
        &mut self,
        is_bid_side: bool,
        quote_lots_per_base_lot: u64,
        base_lots: u64,
        user: Pubkey,
    ) -> Result<()> {
        if is_bid_side {
            let order_tree = &mut self.bid_tree;
            order_tree.insert(
                OrderKey {
                    quote_lots_per_base_lot,
                    order_number: u64::MAX.checked_sub(self.order_number).unwrap(), // on the bid side we want the ealier orders to be larger
                },
                Order { user, base_lots },
            );
        } else {
            let order_tree = &mut self.ask_tree;
            order_tree.insert(
                OrderKey {
                    quote_lots_per_base_lot,
                    order_number: self.order_number, // on the quote side we want the earlier orders to be smaller
                },
                Order { user, base_lots },
            );
        }

        self.order_number = self.order_number.wrapping_add(1); // can't catch me

        Ok(())
    }

    pub fn cancel_order(
        &mut self,
        is_bid_side: bool,
        quote_lots_per_base_lot: u64,
        order_number: u64,
        settlement: &mut Settlement,
    ) -> Result<()> {
        if is_bid_side {
            let order_tree = &mut self.bid_tree;
            let order_key = OrderKey {
                quote_lots_per_base_lot,
                order_number,
            };
            let order = order_tree.remove(&order_key).unwrap();

            let quote_amount = order
                .base_lots
                .checked_mul(quote_lots_per_base_lot)
                .unwrap()
                .checked_mul(self.quote_lot_size)
                .unwrap();

            settlement.add_voucher(order.user, 0u64, quote_amount)?;
        } else {
            let order_tree = &mut self.ask_tree;
            let order_key = OrderKey {
                quote_lots_per_base_lot,
                order_number,
            };
            let order = order_tree.remove(&order_key).unwrap();

            let base_amount = order.base_lots.checked_mul(self.base_lot_size).unwrap();

            settlement.add_voucher(order.user, base_amount, 0u64)?;
        }

        Ok(())
    }

    pub fn drop_worst_order(
        &mut self,
        is_bid_side: bool,
        settlement: &mut Settlement,
    ) -> Result<()> {
        if is_bid_side {
            let order_tree = &mut self.bid_tree;
            let (k, v) = order_tree.get_min().unwrap();
            let order = order_tree.remove(&k).unwrap();

            let quote_amount = order
                .base_lots
                .checked_mul(k.quote_lots_per_base_lot)
                .unwrap()
                .checked_mul(self.quote_lot_size)
                .unwrap();

            settlement.add_voucher(order.user, 0u64, quote_amount)?;
        } else {
            let order_tree = &mut self.ask_tree;
            let (k, v) = order_tree.get_max().unwrap();
            let order = order_tree.remove(&k).unwrap();

            let base_amount = order.base_lots.checked_mul(self.base_lot_size).unwrap();

            settlement.add_voucher(order.user, base_amount, 0u64)?;
        }

        Ok(())
    }
}
