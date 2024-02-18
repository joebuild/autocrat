use anchor_lang::prelude::*;

use bytemuck::Pod;
use bytemuck::Zeroable;
use sokoban::*;

use crate::error::ErrorCode;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Voucher {
    pub base_amount: u64,
    pub quote_amount: u64,
}

unsafe impl Zeroable for Voucher {}
unsafe impl Pod for Voucher {}

const MAX_SIZE: usize = 256;
const NUM_BUCKETS: usize = MAX_SIZE >> 2;

#[account(zero_copy)]
pub struct Settlement {
    pub hybrid: Pubkey,
    pub voucher_map: HashTable<Pubkey, Voucher, NUM_BUCKETS, MAX_SIZE>,
}

impl Settlement {
    pub fn add_voucher(&mut self, user: Pubkey, base_amount: u64, quote_amount: u64) -> Result<()> {
        match self.voucher_map.get_mut(&user) {
            Some(voucher) => {
                voucher.base_amount += base_amount;
                voucher.quote_amount += quote_amount;
            }
            None => {
                if self.voucher_map.size() < self.voucher_map.capacity() {
                    self.voucher_map
                        .insert(
                            user,
                            Voucher {
                                base_amount,
                                quote_amount,
                            },
                        )
                        .unwrap();
                } else {
                    return err!(ErrorCode::SettlementFull);
                }
            }
        }

        Ok(())
    }

    pub fn pop_voucher(&mut self, user: Pubkey) -> Result<Option<Voucher>> {
        Ok(self.voucher_map.remove(&user))
    }
}
