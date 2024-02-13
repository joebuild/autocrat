import { PublicKey } from "@solana/web3.js";
import { AutocratClient } from "../AutocratClient";
import { InstructionHandler } from "../InstructionHandler";
import { getATA, getAmmAddr, getAmmPositionAddr, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr } from '../utils';
import BN from "bn.js";

export const createAmmHandler = async (
    client: AutocratClient,
    baseMint: PublicKey,
    quoteMint: PublicKey,
    swapFeeBps: number,
    permissioned: boolean,
    permissionedCaller: PublicKey,
): Promise<InstructionHandler> => {
    let [ammAddr] = getAmmAddr(
        client.ammProgram.programId,
        baseMint,
        quoteMint,
        swapFeeBps,
        permissionedCaller
    )

    let [vaultAtaBase] = getATA(baseMint, ammAddr)
    let [vaultAtaQuote] = getATA(quoteMint, ammAddr)

    let ix = await client.ammProgram.methods
        .createAmm({
            permissioned,
            permissionedCaller,
            swapFeeBps: swapFeeBps,
        })
        .accounts({
            user: client.provider.publicKey,
            amm: ammAddr,
            baseMint,
            quoteMint,
            vaultAtaBase,
            vaultAtaQuote,
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
