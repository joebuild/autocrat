import { PublicKey } from "@solana/web3.js";
import { AutocratClient } from "../AutocratClient";
import { InstructionHandler } from "../InstructionHandler";
import { getATA, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr } from '../utils';
import BN from "bn.js";

export const createProposalPartOneHandler = async (
    client: AutocratClient,
    descriptionUrl: string,
    proposalInstructionsAddr: PublicKey,
): Promise<InstructionHandler> => {
    let dao = await client.autocratProgram.account.dao.fetch(getDaoAddr(client.autocratProgram.programId)[0])

    let proposalAddr = getProposalAddr(client.autocratProgram.programId, dao.proposalCount)[0]

    let conditionalOnPassMetaMint = getConditionalOnPassMetaMintAddr(client.autocratProgram.programId, dao.proposalCount)[0]
    let conditionalOnPassUsdcMint = getConditionalOnPassUsdcMintAddr(client.autocratProgram.programId, dao.proposalCount)[0]
    let conditionalOnFailMetaMint = getConditionalOnFailMetaMintAddr(client.autocratProgram.programId, dao.proposalCount)[0]
    let conditionalOnFailUsdcMint = getConditionalOnFailUsdcMintAddr(client.autocratProgram.programId, dao.proposalCount)[0]

    let ix = await client.autocratProgram.methods
        .createProposalPartOne(
            descriptionUrl,
        )
        .accounts({
            proposer: client.provider.publicKey,
            proposal: proposalAddr,
            proposalInstructions: proposalInstructionsAddr,
            dao: getDaoAddr(client.autocratProgram.programId)[0],
            passMarketAmm: getPassMarketAmmAddr(client.autocratProgram.programId, dao.proposalCount)[0],
            failMarketAmm: getFailMarketAmmAddr(client.autocratProgram.programId, dao.proposalCount)[0],
            metaMint: dao.metaMint,
            usdcMint: dao.usdcMint,
            conditionalOnPassMetaMint,
            conditionalOnPassUsdcMint,
            conditionalOnFailMetaMint,
            conditionalOnFailUsdcMint,
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
