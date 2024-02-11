import {AutocratClient} from "../AutocratClient";
import {InstructionHandler} from "../InstructionHandler";
import { getATA, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr } from '../utils';
import BN from "bn.js";

export const finalizeProposalHandler = async (
    client: AutocratClient,
    proposalNumber: number
): Promise<InstructionHandler> => {
    let daoAddr = getDaoAddr(client.program.programId)[0]
    let dao = await client.program.account.dao.fetch(daoAddr)

    let proposalAddr = getProposalAddr(client.program.programId, proposalNumber)[0]
    const proposalAcc = await client.program.account.proposal.fetch(proposalAddr);

    let ix = await client.program.methods
        .finalizeProposal()
        .accounts({
            proposal: proposalAddr,
            instructions: proposalAcc.instructions,
            dao: getDaoAddr(client.program.programId)[0],
            daoTreasury: getDaoTreasuryAddr(client.program.programId)[0],
            passMarketAmm: getPassMarketAmmAddr(client.program.programId, dao.proposalCount)[0],
            failMarketAmm: getFailMarketAmmAddr(client.program.programId, dao.proposalCount)[0],
        })
        .instruction()
        
    return new InstructionHandler([ix], [], client)
};
