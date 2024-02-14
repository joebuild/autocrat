import { PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";
import { AutocratClient } from "../AutocratClient";
import { InstructionHandler } from "../InstructionHandler";
import { getATA, getAmmPositionAddr, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr } from '../utils';
import BN from "bn.js";

export const swapHandler = async (
    client: AutocratClient,
    ammAddr: PublicKey,
    isQuoteToBase: boolean,
    inputAmount: BN,
    minOutputAmount: BN,
): Promise<InstructionHandler> => {
    const amm = await client.ammProgram.account.amm.fetch(ammAddr);

    let ix = await client.ammProgram.methods
        .swap(
            isQuoteToBase,
            inputAmount,
            minOutputAmount,
        )
        .accounts({
            user: client.provider.publicKey,
            amm: ammAddr,
            baseMint: amm.baseMint,
            quoteMint: amm.quoteMint,
            userAtaBase: getATA(amm.baseMint, client.provider.publicKey)[0],
            userAtaQuote: getATA(amm.quoteMint, client.provider.publicKey)[0],
            vaultAtaBase: getATA(amm.baseMint, ammAddr)[0],
            vaultAtaQuote: getATA(amm.quoteMint, ammAddr)[0],
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};