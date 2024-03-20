import { PublicKey } from "@solana/web3.js";
import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { getDaoAddr, getDaoTreasuryAddr } from '../../utils';
import { UpdateDaoParams } from '../../types';

export const updateDaoHandler = async (
    client: AutocratClient,
    daoId: PublicKey,
    updateDaoParams: UpdateDaoParams
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let ix = await client.program.methods
        .updateDao(updateDaoParams)
        .accounts({
            dao: getDaoAddr(client.program.programId, daoId)[0],
            daoTreasury: getDaoTreasuryAddr(client.program.programId, daoId)[0],
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
