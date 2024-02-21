import { PublicKey } from '@solana/web3.js';
import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { getDaoAddr } from '../../utils';
import { META_MINT, USDC_MINT } from '../../constants';

export const initializeDaoHandler = async (
    client: AutocratClient,
    metaMint?: PublicKey,
    usdcMint?: PublicKey
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let ix = await client.program.methods
        .initializeDao()
        .accounts({
            payer: client.provider.wallet.publicKey,
            dao: getDaoAddr(client.program.programId)[0],
            metaMint: metaMint || META_MINT,
            usdcMint: usdcMint || USDC_MINT,
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
