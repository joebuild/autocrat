import {AccountMeta, PublicKey} from "@solana/web3.js";
import {utils} from "@coral-xyz/anchor";
import { numToBytes32LE } from "./numbers";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";

export const getDaoAddr = (
    programId: PublicKey,
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [encode("WWCACOTMICMIBMHAFTTWYGHMB")],
        programId,
    );
};

export const getProposalAddr = (
    programId: PublicKey,
    proposalNumber: number
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [encode("proposal"), numToBytes32LE(proposalNumber)],
        programId,
    );
};

export const getPassMarketAmmAddr = (
    programId: PublicKey,
    proposalNumber: number
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [encode("pass_market_amm"), numToBytes32LE(proposalNumber)],
        programId,
    );
};

export const getFailMarketAmmAddr = (
    programId: PublicKey,
    proposalNumber: number
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [encode("fail_market_amm"), numToBytes32LE(proposalNumber)],
        programId,
    );
};

export const encode = (x: string) => Buffer.from(utils.bytes.utf8.encode(x))

export const getATA = (mint: PublicKey, owner: PublicKey) => {
    return PublicKey.findProgramAddressSync(
        [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        ASSOCIATED_TOKEN_PROGRAM_ID,
    );
};