import { PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";
import { AutocratClient } from "../../../AutocratClient";
import { InstructionHandler } from "../../../InstructionHandler";
import { getAmmPositionAddr } from '../../../utils';

export const createAmmPositionCpiHandler = async (
    client: AutocratClient,
    amm: PublicKey,
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let ix = await client.program.methods
        .createPosition()
        .accounts({
            user: client.provider.publicKey,
            amm,
            ammPosition: getAmmPositionAddr(client.program.programId, amm, client.provider.publicKey)[0],
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
