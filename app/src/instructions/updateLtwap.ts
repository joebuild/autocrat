import { PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";
import { AutocratClient } from "../AutocratClient";
import { InstructionHandler } from "../InstructionHandler";
import { getATA, getAmmPositionAddr, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr } from '../utils';
import BN from "bn.js";

export const updateLtwapHandler = async (
    client: AutocratClient,
    ammAddr: PublicKey,
): Promise<InstructionHandler> => {
    let ix = await client.ammProgram.methods
        .updateLtwap()
        .accounts({
            user: client.provider.publicKey,
            amm: ammAddr,
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
