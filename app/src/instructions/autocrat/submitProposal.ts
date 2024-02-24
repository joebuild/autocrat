import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import { AutocratClient } from "../../AutocratClient";
import { InstructionHandler } from "../../InstructionHandler";
import { getATA, getDaoAddr, getProposalAddr, getProposalVaultAddr } from '../../utils';
import { Keypair, PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";

export const submitProposalHandler = async (
    client: AutocratClient,
    proposalNumber: number,
    proposalInstructions: PublicKey,
    ammProgram: PublicKey,
): Promise<InstructionHandler<typeof client.program, AutocratClient>> => {
    let daoAddr = getDaoAddr(client.program.programId)[0]

    let proposalAddr = getProposalAddr(client.program.programId, proposalNumber)[0]
    let proposal = await client.program.account.proposal.fetch(proposalAddr)

    let proposalVaultAddr = getProposalVaultAddr(client.program.programId, proposalAddr)[0]

    let ix = await client.program.methods
        .submitProposal()
        .accounts({
            proposer: client.provider.publicKey,
            dao: daoAddr,
            proposal: proposalAddr,
            proposalVault: proposalVaultAddr,
            proposalInstructions,
            passMarketAmm: proposal.passMarketAmm,
            failMarketAmm: proposal.failMarketAmm,
            ammProgram,
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY
        })
        .instruction()

    return new InstructionHandler([ix], [], client)
};