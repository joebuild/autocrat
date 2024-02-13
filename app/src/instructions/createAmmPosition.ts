import { PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";
import { AutocratClient } from "../AutocratClient";
import { InstructionHandler } from "../InstructionHandler";
import { getATA, getAmmPositionAddr, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr } from '../utils';
import BN from "bn.js";

export const createAmmPositionHandler = async (
    client: AutocratClient,
    amm: PublicKey,
): Promise<InstructionHandler> => {
    let ix = await client.ammProgram.methods
        .createPosition()
        .accounts({
            user: client.provider.publicKey,
            amm,
            ammPosition: getAmmPositionAddr(client.ammProgram.programId, amm, client.provider.publicKey)[0],
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
