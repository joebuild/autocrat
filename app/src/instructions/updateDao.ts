import { AutocratClient } from "../AutocratClient";
import { InstructionHandler } from "../InstructionHandler";
import { getDaoAddr, getDaoTreasuryAddr } from '../utils';
import { UpdateDaoParams } from '../types';

export const updateDaoHandler = async (
    client: AutocratClient,
    updateDaoParams: UpdateDaoParams
): Promise<InstructionHandler> => {
    let ix = await client.autocratProgram.methods
        .updateDao(updateDaoParams)
        .accounts({
            dao: getDaoAddr(client.autocratProgram.programId)[0],
            daoTreasury: getDaoTreasuryAddr(client.autocratProgram.programId)[0],
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
