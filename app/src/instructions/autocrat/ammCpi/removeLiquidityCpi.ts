import { PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";
import { AutocratClient } from "../../../AutocratClient";
import { InstructionHandler } from "../../../InstructionHandler";
import { getATA } from '../../../utils';
import BN from "bn.js";

export const removeLiquidityCpiHandler = async (
    client: AutocratClient,
    proposalAddr: PublicKey,
    ammAddr: PublicKey,
    removeBps: BN,
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    const proposal = await client.program.account.proposal.fetch(proposalAddr);

    if (proposal.passMarketAmm.toBase58() !== ammAddr.toBase58() && proposal.failMarketAmm.toBase58() !== ammAddr.toBase58()) {
        throw new Error("the amm address passed in removeLiquidityCpiHandler does not correspond to either the pass or fail market");
    }

    let baseMint: PublicKey;
    let quoteMint: PublicKey;

    if (proposal.passMarketAmm.toBase58() === ammAddr.toBase58()) {
        baseMint = proposal.conditionalOnPassMetaMint
        quoteMint = proposal.conditionalOnPassUsdcMint
    } else {
        baseMint = proposal.conditionalOnFailMetaMint
        quoteMint = proposal.conditionalOnFailUsdcMint
    }

    let userAtaBase = getATA(baseMint, client.provider.publicKey)[0]
    let userAtaQuote = getATA(quoteMint, client.provider.publicKey)[0]

    let vaultAtaBase = getATA(baseMint, ammAddr)[0]
    let vaultAtaQuote = getATA(quoteMint, ammAddr)[0]

    let ix = await client.program.methods
        .removeLiquidity(
            removeBps,
        )
        .accounts({
            user: client.provider.publicKey,
            amm: ammAddr,
            metaMint: proposal.metaMint,
            usdcMint: proposal.usdcMint,
            conditionalMetaMint: baseMint,
            conditionalUsdcMint: quoteMint,
            conditionalMetaUserAta: userAtaBase,
            conditionalUsdcUserAta: userAtaQuote,
            conditionalMetaVaultAta: vaultAtaBase,
            conditionalUsdcVaultAta: vaultAtaQuote,
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
