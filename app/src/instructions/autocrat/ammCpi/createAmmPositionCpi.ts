import { PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";
import { AutocratClient } from "../../../AutocratClient";
import { InstructionHandler } from "../../../InstructionHandler";
import { getAmmPositionAddr } from '../../../utils';

export const createAmmPositionCpiHandler = async (
    client: AutocratClient,
    proposalAddr: PublicKey,
    amm: PublicKey,
    ammProgram: PublicKey,
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let ix = await client.program.methods
        .createPosition()
        .accounts({
            user: client.provider.publicKey,
            proposal: proposalAddr,
            amm,
            ammPosition: getAmmPositionAddr(ammProgram, amm, client.provider.publicKey)[0],
            ammProgram,
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
