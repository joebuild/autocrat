import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { getATA, getDaoAddr, getProposalVaultAddr } from '../../utils';
import { Keypair, PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";

export const submitProposalHandler = async (
    client: AutocratClient,
    proposalKeypair: Keypair,
    proposalInstructions: PublicKey,
    descriptionUrl: string,
    ammProgram: PublicKey,
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let daoAddr = getDaoAddr(client.program.programId)[0]

    let proposalAddr = proposalKeypair.publicKey
    let proposal = await client.program.account.proposal.fetch(proposalAddr)

    let proposalVaultAddr = getProposalVaultAddr(client.program.programId, proposalAddr)[0]

    let ix = await client.program.methods
        .submitProposal(
            descriptionUrl
        )
        .accounts({
            proposer: client.provider.publicKey,
            dao: daoAddr,
            proposal: proposalAddr,
            proposalVault: proposalVaultAddr,
            proposalInstructions,
            passMarketAmm: proposal.passMarketAmm,
            failMarketAmm: proposal.failMarketAmm,
            metaMint: proposal.metaMint,
            usdcMint: proposal.usdcMint,
            metaProposerAta: getATA(proposal.metaMint, client.provider.publicKey)[0],
            usdcProposerAta: getATA(proposal.usdcMint, client.provider.publicKey)[0],
            metaVaultAta: getATA(proposal.metaMint, proposalVaultAddr)[0],
            usdcVaultAta: getATA(proposal.usdcMint, proposalVaultAddr)[0],
            ammProgram,
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
