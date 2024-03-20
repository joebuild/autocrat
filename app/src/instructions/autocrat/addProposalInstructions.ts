import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { ProposalInstruction } from "../../types";
import { PublicKey } from "@solana/web3.js";
import {
  getDaoAddr,
  getProposalAddr,
  getProposalInstructionsAddr,
} from "../../utils";

export const addProposalInstructionsHandler = async (
  client: AutocratClient,
  daoId: PublicKey,
  proposalNumber: number,
  instructions: ProposalInstruction[]
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
  let daoAddr = getDaoAddr(client.program.programId, daoId)[0];

  let proposalAddr = getProposalAddr(
    client.program.programId,
    daoAddr,
    proposalNumber
  )[0];

  let proposalInstructionsAddr = getProposalInstructionsAddr(
    client.program.programId,
    proposalAddr
  )[0];

  let ix = await client.program.methods
    .addProposalInstructions(instructions)
    .accounts({
      proposer: client.provider.publicKey,
      proposal: proposalAddr,
      proposalInstructions: proposalInstructionsAddr,
    })
    .instruction();

  return new InstructionHandler([ix], [], client);
};
