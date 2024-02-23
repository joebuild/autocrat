import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { getATA, getProposalVaultAddr } from '../../utils';
import { Keypair, PublicKey } from "@solana/web3.js";

export const submitProposalHandler = async (
    client: AutocratClient,
    proposalKeypair: Keypair,
    proposalInstructions: PublicKey,
    descriptionUrl: string
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let proposalAddr = proposalKeypair.publicKey
    let proposal = await client.program.account.proposal.fetch(proposalAddr)

    let proposalVaultAddr = getProposalVaultAddr(client.program.programId, proposalAddr)[0]

    let ix = await client.program.methods
        .submitProposal(
            descriptionUrl
        )
        .accounts({
            proposer: client.provider.publicKey,
            proposal: proposalAddr,
            proposalVault: proposalVaultAddr,
            proposalInstructions,
            metaMint: proposal.metaMint,
            usdcMint: proposal.usdcMint,
            metaProposerAta: getATA(proposal.metaMint, client.provider.publicKey)[0],
            usdcProposerAta: getATA(proposal.usdcMint, client.provider.publicKey)[0],
            metaVaultAta: getATA(proposal.metaMint, proposalVaultAddr)[0],
            usdcVaultAta: getATA(proposal.usdcMint, proposalVaultAddr)[0],
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};
