import { AutocratClient } from "../AutocratClient";
import { InstructionHandler } from "../InstructionHandler";
import { getATA, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr } from '../utils';
import BN from "bn.js";

export const finalizeProposalHandler = async (
    client: AutocratClient,
    proposalNumber: number
): Promise<InstructionHandler> => {
    let daoAddr = getDaoAddr(client.autocratProgram.programId)[0]
    let dao = await client.autocratProgram.account.dao.fetch(daoAddr)

    let proposalAddr = getProposalAddr(client.autocratProgram.programId, proposalNumber)[0]
    const proposalAcc = await client.autocratProgram.account.proposal.fetch(proposalAddr);

    let ix = await client.autocratProgram.methods
        .finalizeProposal()
        .accounts({
            proposal: proposalAddr,
            instructions: proposalAcc.instructions,
            dao: getDaoAddr(client.autocratProgram.programId)[0],
            daoTreasury: getDaoTreasuryAddr(client.autocratProgram.programId)[0],
            passMarketAmm: getPassMarketAmmAddr(client.autocratProgram.programId, dao.proposalCount)[0],
            failMarketAmm: getFailMarketAmmAddr(client.autocratProgram.programId, dao.proposalCount)[0],
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
