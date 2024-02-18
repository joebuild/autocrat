use anchor_lang::prelude::*;

use bytemuck::Pod;
use bytemuck::Zeroable;
use sokoban::*;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Record {
    is_orderbook_trade: bool,
    taker: Pubkey,
    maker: Pubkey,
    base_lots: u64,
    quote_lots: u64,
}

unsafe impl Zeroable for Record {}
unsafe impl Pod for Record {}

const MAX_SIZE: usize = 128;
const NUM_NODES: usize = MAX_SIZE << 1;

#[account(zero_copy)]
pub struct History {
    pub hybrid: Pubkey,
    pub order_number: u128,
    pub records_tree: Critbit<Record, NUM_NODES, MAX_SIZE>,
}
