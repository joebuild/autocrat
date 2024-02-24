import { AccountMeta, PublicKey } from "@solana/web3.js";
import { utils } from "@coral-xyz/anchor";
import { numToBytes32LE, numToBytes64LE } from "./numbers";
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
    proposer: PublicKey,
    proposalNumber: number,
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [utils.bytes.utf8.encode("proposal"), proposer.toBuffer(), numToBytes64LE(proposalNumber)],
        programId,
    );
};

export const getProposalVaultAddr = (
    programId: PublicKey,
    proposal: PublicKey
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [utils.bytes.utf8.encode("proposal_vault"), proposal.toBuffer()],
        programId,
    );
};

export const getAmmAddr = (
    programId: PublicKey,
    baseMint: PublicKey,
    quoteMint: PublicKey,
    swapFeeBps: number,
    permissionedCaller: PublicKey
): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
        [baseMint.toBuffer(), quoteMint.toBuffer(), numToBytes64LE(swapFeeBps), permissionedCaller.toBuffer()],
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

export const getATA = (mint: PublicKey, owner: PublicKey) => {
    return PublicKey.findProgramAddressSync(
        [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        ASSOCIATED_TOKEN_PROGRAM_ID,
    );
};