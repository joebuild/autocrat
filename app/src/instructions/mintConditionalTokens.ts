import {PublicKey} from '@solana/web3.js';
import {AutocratClient} from "../AutocratClient";
import {InstructionHandler} from "../InstructionHandler";
import { getATA, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getProposalAddr } from '../utils';
import BN from 'bn.js';

export const mintConditionalTokensHandler = async (
    client: AutocratClient,
    metaAmount: BN,
    usdcAmount: BN,
    proposalNumber: number
): Promise<InstructionHandler> => {

    let daoAddr = getDaoAddr(client.program.programId)[0]
    let dao = await client.program.account.dao.fetch(daoAddr)

    let proposalAddr = getProposalAddr(client.program.programId, proposalNumber)[0]

    let conditionalOnPassMetaMint = getConditionalOnPassMetaMintAddr(client.program.programId, proposalNumber)[0]
    let conditionalOnPassUsdcMint = getConditionalOnPassUsdcMintAddr(client.program.programId, proposalNumber)[0]
    let conditionalOnFailMetaMint = getConditionalOnFailMetaMintAddr(client.program.programId, proposalNumber)[0]
    let conditionalOnFailUsdcMint = getConditionalOnFailUsdcMintAddr(client.program.programId, proposalNumber)[0]

    let ix = await client.program.methods
        .mintConditionalTokens(metaAmount, usdcAmount)
        .accounts({
            user: client.provider.publicKey,
            dao: daoAddr,
            proposal: proposalAddr,
            metaMint: dao.metaMint,
            usdcMint: dao.usdcMint,
            conditionalOnPassMetaMint,
            conditionalOnPassUsdcMint,
            conditionalOnFailMetaMint,
            conditionalOnFailUsdcMint,
            metaUserAta: getATA(dao.metaMint, client.provider.publicKey)[0],
            usdcUserAta: getATA(dao.usdcMint, client.provider.publicKey)[0],
            conditionalOnPassMetaUserAta: getATA(conditionalOnPassMetaMint, client.provider.publicKey)[0],
            conditionalOnPassUsdcUserAta: getATA(conditionalOnPassUsdcMint, client.provider.publicKey)[0],
            conditionalOnFailMetaUserAta: getATA(conditionalOnFailMetaMint, client.provider.publicKey)[0],
            conditionalOnFailUsdcUserAta: getATA(conditionalOnFailUsdcMint, client.provider.publicKey)[0],
            metaVaultAta: getATA(dao.metaMint, proposalAddr)[0],
            usdcVaultAta: getATA(dao.usdcMint, proposalAddr)[0],
        })
        .instruction()
        
    return new InstructionHandler([ix], [], client)
};
