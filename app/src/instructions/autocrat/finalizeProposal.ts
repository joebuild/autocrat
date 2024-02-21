import { PublicKey } from "@solana/web3.js";
import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { getDaoAddr, getDaoTreasuryAddr } from '../../utils';

export const finalizeProposalHandler = async (
    client: AutocratClient,
    proposalAddr: PublicKey
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    const proposalAcc = await client.program.account.proposal.fetch(proposalAddr);

    let ix = await client.program.methods
        .finalizeProposal()
        .accounts({
            proposal: proposalAddr,
            instructions: proposalAcc.instructions,
            dao: getDaoAddr(client.program.programId)[0],
            daoTreasury: getDaoTreasuryAddr(client.program.programId)[0],
            passMarketAmm: proposalAcc.passMarketAmm,
            failMarketAmm: proposalAcc.failMarketAmm,
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
