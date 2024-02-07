import {BN} from "bn.js";
import {Decimal} from 'decimal.js';

export const numToBytes32LE = (num: number) => {
    let bytesU32 = Buffer.alloc(4);
    bytesU32.writeInt32LE(num)
    return bytesU32
}
