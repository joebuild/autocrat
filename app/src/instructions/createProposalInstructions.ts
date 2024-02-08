import {AutocratClient} from "../AutocratClient";
import {InstructionHandler} from "../InstructionHandler";
import { getDaoAddr, getDaoTreasuryAddr, getProposalInstructionsAddr } from '../utils';
import { ProposalInstruction, UpdateDaoParams } from '../types';

export const createProposalInstructionsHandler = async (
    client: AutocratClient,
    instructions: ProposalInstruction[]
): Promise<InstructionHandler> => {
    let dao = await client.program.account.dao.fetch(getDaoAddr(client.program.programId)[0])

    let ix = await client.program.methods
        .createProposalInstructions(instructions)
        .accounts({
            proposer: client.provider.publicKey,
            dao: getDaoAddr(client.program.programId)[0],
            proposalInstructions: getProposalInstructionsAddr(client.program.programId, dao.proposalCount)[0],
        })
        .instruction()
        
    return new InstructionHandler([ix], [], client)
};