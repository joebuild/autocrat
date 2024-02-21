import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { getDaoAddr, getDaoTreasuryAddr } from '../../utils';
import { ProposalInstruction, UpdateDaoParams } from '../../types';
import { PublicKey } from "@solana/web3.js";

export const addProposalInstructionsHandler = async (
    client: AutocratClient,
    instructions: ProposalInstruction[],
    proposalInstructionsAddr: PublicKey,
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let ix = await client.program.methods
        .addProposalInstructions(instructions)
        .accounts({
            proposer: client.provider.publicKey,
            dao: getDaoAddr(client.program.programId)[0],
            proposalInstructions: proposalInstructionsAddr,
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
