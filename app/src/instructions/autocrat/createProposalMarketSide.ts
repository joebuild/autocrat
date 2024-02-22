import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { getATA, getAmmAddr, getAmmPositionAddr, getDaoAddr, getProposalVaultAddr } from '../../utils';
import BN from "bn.js";
import { Keypair, PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";

export const createProposalMarketSideHandler = async (
    client: AutocratClient,
    proposalKeypair: Keypair,
    isPassMarket: boolean,
    condMetaToMint: BN,
    condUsdcToMint: BN,
    ammBaseAmountDeposit: BN,
    ammQuoteAmountDeposit: BN,
    ammProgram: PublicKey,
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let daoAddr = getDaoAddr(client.program.programId)[0]
    let dao = await client.program.account.dao.fetch(daoAddr)

    let proposalAddr = proposalKeypair.publicKey
    let proposalVaultAddr = getProposalVaultAddr(client.program.programId, proposalAddr)[0]

    let conditionalMetaMintKeypair = Keypair.generate()
    let conditionalMetaMintAddr = conditionalMetaMintKeypair.publicKey

    let conditionalUsdcMintKeypair = Keypair.generate()
    let conditionalUsdcMintAddr = conditionalUsdcMintKeypair.publicKey

    let ammAddr = getAmmAddr(ammProgram, conditionalMetaMintAddr, conditionalUsdcMintAddr, dao.ammSwapFeeBps.toNumber(), client.program.programId)[0]

    let ix = await client.program.methods
        .createProposalMarketSide(
            isPassMarket,
            condMetaToMint,
            condUsdcToMint,
            ammBaseAmountDeposit,
            ammQuoteAmountDeposit,
        )
        .accounts({
            proposer: client.provider.publicKey,
            proposal: proposalAddr,
            proposalVault: proposalVaultAddr,
            dao: daoAddr,
            amm: ammAddr,
            ammPosition: getAmmPositionAddr(ammProgram, ammAddr, client.provider.publicKey)[0],
            metaMint: dao.metaMint,
            usdcMint: dao.usdcMint,
            conditionalMetaMint: conditionalMetaMintAddr,
            conditionalUsdcMint: conditionalUsdcMintAddr,
            conditionalMetaProposerAta: getATA(conditionalMetaMintAddr, client.provider.publicKey)[0],
            conditionalUsdcProposerAta: getATA(conditionalUsdcMintAddr, client.provider.publicKey)[0],
            conditionalMetaAmmVaultAta: getATA(conditionalMetaMintAddr, ammAddr)[0],
            conditionalUsdcAmmVaultAta: getATA(conditionalUsdcMintAddr, ammAddr)[0],
            ammProgram,
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY
        })
        .instruction()

    return new InstructionHandler([ix], [proposalKeypair, conditionalMetaMintKeypair, conditionalUsdcMintKeypair], client)
};
