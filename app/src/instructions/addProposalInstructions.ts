import { AutocratClient } from "../AutocratClient";
import { InstructionHandler } from "../InstructionHandler";
import { getDaoAddr, getDaoTreasuryAddr } from '../utils';
import { ProposalInstruction, UpdateDaoParams } from '../types';
import { PublicKey } from "@solana/web3.js";

export const addProposalInstructionsHandler = async (
    client: AutocratClient,
    instructions: ProposalInstruction[],
    proposalInstructionsAddr: PublicKey,
): Promise<InstructionHandler> => {
    let ix = await client.autocratProgram.methods
        .addProposalInstructions(instructions)
        .accounts({
            proposer: client.provider.publicKey,
            dao: getDaoAddr(client.autocratProgram.programId)[0],
            proposalInstructions: proposalInstructionsAddr,
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
