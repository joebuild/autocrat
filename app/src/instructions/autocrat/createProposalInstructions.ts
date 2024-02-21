import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { ProposalInstruction } from '../../types';
import { Keypair } from "@solana/web3.js";

export const createProposalInstructionsHandler = async (
    client: AutocratClient,
    instructions: ProposalInstruction[],
    proposalInstructionsKeypair: Keypair,
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let ix = await client.program.methods
        .createProposalInstructions(instructions)
        .accounts({
            proposer: client.provider.publicKey,
            proposalInstructions: proposalInstructionsKeypair.publicKey,
        })
        .instruction()

    return new InstructionHandler([ix], [proposalInstructionsKeypair], client)
};
