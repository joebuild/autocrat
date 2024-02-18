use anchor_lang::prelude::*;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct AnchorDecimal {
    data: [u8; 16], // serialized rust_decimal::Decimal (96 dec, 32 scale)
}

impl AnchorDecimal {
    pub fn deser(&self) -> Decimal {
        Decimal::deserialize(self.data)
    }

    pub fn ser(decimal: Decimal) -> AnchorDecimal {
        AnchorDecimal {
            data: decimal.serialize(),
        }
    }
}
