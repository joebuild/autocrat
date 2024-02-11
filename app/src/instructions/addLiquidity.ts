import {AutocratClient} from "../AutocratClient";
import {InstructionHandler} from "../InstructionHandler";
import { getATA, getAmmPositionAddr, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr } from '../utils';
import BN from "bn.js";

export const addLiquidityHandler = async (
    client: AutocratClient,
    maxBaseAmount: BN,
    maxQuoteAmount: BN,
    isPassMarket: boolean,
    proposalNumber: number
): Promise<InstructionHandler> => {
    let conditionalBaseMint,
        conditionalQuoteMint,
        amm;

    if (isPassMarket){
        conditionalBaseMint = getConditionalOnPassMetaMintAddr(client.program.programId, proposalNumber)[0]
        conditionalQuoteMint = getConditionalOnPassUsdcMintAddr(client.program.programId, proposalNumber)[0]
        amm = getPassMarketAmmAddr(client.program.programId, proposalNumber)[0]
    } else {
        conditionalBaseMint = getConditionalOnFailMetaMintAddr(client.program.programId, proposalNumber)[0]
        conditionalQuoteMint = getConditionalOnFailUsdcMintAddr(client.program.programId, proposalNumber)[0]
        amm = getFailMarketAmmAddr(client.program.programId, proposalNumber)[0]
    } 

    let proposalAddr = getProposalAddr(client.program.programId, proposalNumber)[0]

    let ix = await client.program.methods
        .addLiquidity(
            maxBaseAmount,
            maxQuoteAmount,
            isPassMarket
        )
        .accounts({
            user: client.provider.publicKey,
            dao: getDaoAddr(client.program.programId)[0],
            proposal: proposalAddr,
            amm,
            ammPosition: getAmmPositionAddr(client.program.programId, amm, client.provider.publicKey)[0],
            conditionalBaseMint,
            conditionalQuoteMint,
            userAtaConditionalBase: getATA(conditionalBaseMint, client.provider.publicKey)[0],
            userAtaConditionalQuote: getATA(conditionalQuoteMint, client.provider.publicKey)[0],
            vaultAtaConditionalBase: getATA(conditionalBaseMint, proposalAddr)[0],
            vaultAtaConditionalQuote: getATA(conditionalQuoteMint, proposalAddr)[0],
        })
        .instruction()
        
    return new InstructionHandler([ix], [], client)
};
