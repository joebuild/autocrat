import {AccountMeta, PublicKey} from "@solana/web3.js";
import {utils} from "@coral-xyz/anchor";
import { numToBytes32LE } from "./numbers";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";

export const getDaoAddr = (
    programId: PublicKey,
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [utils.bytes.utf8.encode("WWCACOTMICMIBMHAFTTWYGHMB")],
        programId,
    );
};

export const getDaoTreasuryAddr = (
    programId: PublicKey,
): [PublicKey, number] => {
    let [dao] = getDaoAddr(programId)
    return PublicKey.findProgramAddressSync(
        [dao.toBuffer()],
        programId,
    );
};

export const getProposalAddr = (
    programId: PublicKey,
    proposalNumber: number
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [utils.bytes.utf8.encode("proposal"), numToBytes32LE(proposalNumber)],
        programId,
    );
};

export const getPassMarketAmmAddr = (
    programId: PublicKey,
    proposalNumber: number
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [utils.bytes.utf8.encode("pass_market_amm"), numToBytes32LE(proposalNumber)],
        programId,
    );
};

export const getFailMarketAmmAddr = (
    programId: PublicKey,
    proposalNumber: number
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [utils.bytes.utf8.encode("fail_market_amm"), numToBytes32LE(proposalNumber)],
        programId,
    );
};

export const getAmmPositionAddr = (
    programId: PublicKey,
    amm: PublicKey,
    user: PublicKey
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [amm.toBuffer(), user.toBuffer()],
        programId,
    );
};

export const getConditionalOnPassMetaMintAddr = (
    programId: PublicKey,
    proposalNumber: number
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [utils.bytes.utf8.encode("conditional_on_pass_meta"), numToBytes32LE(proposalNumber)],
        programId,
    );
};

export const getConditionalOnPassUsdcMintAddr = (
    programId: PublicKey,
    proposalNumber: number
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [utils.bytes.utf8.encode("conditional_on_pass_usdc"), numToBytes32LE(proposalNumber)],
        programId,
    );
};

export const getConditionalOnFailMetaMintAddr = (
    programId: PublicKey,
    proposalNumber: number
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [utils.bytes.utf8.encode("conditional_on_fail_meta"), numToBytes32LE(proposalNumber)],
        programId,
    );
};

export const getConditionalOnFailUsdcMintAddr = (
    programId: PublicKey,
    proposalNumber: number
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [utils.bytes.utf8.encode("conditional_on_fail_usdc"), numToBytes32LE(proposalNumber)],
        programId,
    );
};

export const getATA = (mint: PublicKey, owner: PublicKey) => {
    return PublicKey.findProgramAddressSync(
        [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        ASSOCIATED_TOKEN_PROGRAM_ID,
    );
};