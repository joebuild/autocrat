import { PublicKey } from "@solana/web3.js";
import {AutocratClient} from "../AutocratClient";
import {InstructionHandler} from "../InstructionHandler";
import { getATA, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr } from '../utils';
import BN from "bn.js";

export const createProposalPartOneHandler = async (
    client: AutocratClient,
    descriptionUrl: string,
    proposalInstructionsAddr: PublicKey,
): Promise<InstructionHandler> => {
    let dao = await client.program.account.dao.fetch(getDaoAddr(client.program.programId)[0])

    let proposalAddr = getProposalAddr(client.program.programId, dao.proposalCount)[0]

    let conditionalOnPassMetaMint = getConditionalOnPassMetaMintAddr(client.program.programId, dao.proposalCount)[0]
    let conditionalOnPassUsdcMint = getConditionalOnPassUsdcMintAddr(client.program.programId, dao.proposalCount)[0]
    let conditionalOnFailMetaMint = getConditionalOnFailMetaMintAddr(client.program.programId, dao.proposalCount)[0]
    let conditionalOnFailUsdcMint = getConditionalOnFailUsdcMintAddr(client.program.programId, dao.proposalCount)[0]

    let ix = await client.program.methods
        .createProposalPartOne(
            descriptionUrl,
        )
        .accounts({
            proposer: client.provider.publicKey,
            proposal: proposalAddr,
            proposalInstructions: proposalInstructionsAddr,
            dao: getDaoAddr(client.program.programId)[0],
            daoTreasury: getDaoTreasuryAddr(client.program.programId)[0],
            passMarketAmm: getPassMarketAmmAddr(client.program.programId, dao.proposalCount)[0],
            failMarketAmm: getFailMarketAmmAddr(client.program.programId, dao.proposalCount)[0],
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
