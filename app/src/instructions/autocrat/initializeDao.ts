import { PublicKey } from '@solana/web3.js';
import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { getDaoAddr, getDaoTreasuryAddr } from '../../utils';

export const initializeDaoHandler = async (
    client: AutocratClient,
    daoId: PublicKey,
    metaMint: PublicKey,
    usdcMint: PublicKey
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let daoTreasuryAddr = getDaoTreasuryAddr(client.program.programId, daoId)[0]

    let ix = await client.program.methods
        .initializeDao(daoId)
        .accounts({
            payer: client.provider.wallet.publicKey,
            dao: getDaoAddr(client.program.programId, daoId)[0],
            daoTreasury: daoTreasuryAddr,
            metaMint: metaMint,
            usdcMint: usdcMint,
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
