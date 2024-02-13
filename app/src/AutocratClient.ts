import { AnchorProvider, Program } from "@coral-xyz/anchor";
import {
    AddressLookupTableAccount,
    Keypair,
    PublicKey,
    Signer,
    TransactionInstruction,
    TransactionMessage,
    VersionedTransaction
} from "@solana/web3.js";
import { Autocrat as AutocratIDLType } from '../../target/types/autocrat';
// @ts-ignore
import * as AutocratIDL from '../../target/idl/autocrat.json';

import { Amm as AmmIDLType } from '../../target/types/amm';
// @ts-ignore
import * as AmmIDL from '../../target/idl/amm.json';

import * as ixs from "./instructions";
import BN from "bn.js";
import { addComputeUnits } from "./utils";
import { AMM_PROGRAM_ID, AUTOCRAT_LUTS, AUTOCRAT_PROGRAM_ID } from "./constants";
import { ProposalInstruction, UpdateDaoParams } from "./types";

export type CreateAutocratClientParams = {
    provider: AnchorProvider,
    autocratProgramId?: PublicKey,
    ammProgramId?: PublicKey,
}

export class AutocratClient {
    public readonly provider: AnchorProvider;
    public readonly autocratProgram: Program<AutocratIDLType>;
    public readonly ammProgram: Program<AmmIDLType>;
    public readonly luts: AddressLookupTableAccount[];

    constructor(
        provider: AnchorProvider,
        autocratProgramId: PublicKey,
        ammProgramId: PublicKey,
        luts: AddressLookupTableAccount[],
    ) {
        this.provider = provider
        this.autocratProgram = new Program<AutocratIDLType>(AutocratIDL, autocratProgramId, provider)
        this.ammProgram = new Program<AmmIDLType>(AmmIDL, ammProgramId, provider)
        this.luts = luts
    }

    public static async createClient(createAutocratClientParams: CreateAutocratClientParams): Promise<AutocratClient> {
        let { provider, autocratProgramId, ammProgramId } = createAutocratClientParams;

        const getLuts = () => Promise.all(
            AUTOCRAT_LUTS.map(lut => {
                return provider.connection
                    .getAddressLookupTable(lut)
                    .then((res) => res.value as AddressLookupTableAccount)
            })
        )

        const luts = await getLuts()

        return new AutocratClient(
            provider,
            autocratProgramId || AUTOCRAT_PROGRAM_ID,
            ammProgramId || AMM_PROGRAM_ID,
            luts as AddressLookupTableAccount[],
        )
    }

    async initializeDao(
        metaMint?: PublicKey,
        usdcMint?: PublicKey
    ) {
        return ixs.initializeDaoHandler(
            this,
            metaMint,
            usdcMint
        )
    }

    // this won't ever be called directly (must be called via a proposal), but is here anyway for completeness / testing
    async updateDao(
        updateDaoParams: UpdateDaoParams
    ) {
        return ixs.updateDaoHandler(
            this,
            updateDaoParams
        )
    }

    async createProposalInstructions(
        instructions: ProposalInstruction[],
        proposalInstructionsKeypair: Keypair,
    ) {
        return ixs.createProposalInstructionsHandler(
            this,
            instructions,
            proposalInstructionsKeypair
        )
    }

    async addProposalInstructions(
        instructions: ProposalInstruction[],
        proposalInstructionsAddr: PublicKey,
    ) {
        return ixs.addProposalInstructionsHandler(
            this,
            instructions,
            proposalInstructionsAddr
        )
    }

    async createProposalPartOne(
        descriptionUrl: string,
        proposalInstructionsAddr: PublicKey,
    ) {
        return ixs.createProposalPartOneHandler(
            this,
            descriptionUrl,
            proposalInstructionsAddr
        )
    }

    async createProposalPartTwo(
        initialPassMarketPriceQuoteUnitsPerBaseUnitBps: BN,
        initialFailMarketPriceQuoteUnitsPerBaseUnitBps: BN,
        quoteLiquidityAmountPerAmm: BN,
    ) {
        return ixs.createProposalPartTwoHandler(
            this,
            initialPassMarketPriceQuoteUnitsPerBaseUnitBps,
            initialFailMarketPriceQuoteUnitsPerBaseUnitBps,
            quoteLiquidityAmountPerAmm,
        )
    }

    async mintConditionalTokens(
        metaAmount: BN,
        usdcAmount: BN,
        proposalNumber: number
    ) {
        return ixs.mintConditionalTokensHandler(
            this,
            metaAmount,
            usdcAmount,
            proposalNumber,
        )
    }

    async redeemConditionalTokens(
        proposalNumber: number
    ) {
        return ixs.redeemConditionalTokensHandler(
            this,
            proposalNumber,
        )
    }

    async finalizeProposal(
        proposalNumber: number
    ) {
        return ixs.finalizeProposalHandler(
            this,
            proposalNumber,
        )
    }

    async createAmm(
        baseMint: PublicKey,
        quoteMint: PublicKey,
        swapFeeBps: number,
        permissioned: boolean,
        permissionedCaller: PublicKey = PublicKey.default,
    ) {
        return ixs.createAmmHandler(
            this,
            baseMint,
            quoteMint,
            swapFeeBps,
            permissioned,
            permissionedCaller,
        )
    }

    async createAmmPosition(
        amm: PublicKey
    ) {
        return ixs.createAmmPositionHandler(
            this,
            amm
        )
    }

    async addLiquidity(
        ammAddr: PublicKey,
        ammPositionAddr: PublicKey,
        maxBaseAmount: BN,
        maxQuoteAmount: BN,
    ) {
        return ixs.addLiquidityHandler(
            this,
            ammAddr,
            ammPositionAddr,
            maxBaseAmount,
            maxQuoteAmount
        )
    }

    async removeLiquidity(
        ammAddr: PublicKey,
        ammPositionAddr: PublicKey,
        removeBps: BN,
    ) {
        return ixs.removeLiquidityHandler(
            this,
            ammAddr,
            ammPositionAddr,
            removeBps
        )
    }

    async swap(
        isQuoteToBase: boolean,
        inputAmount: BN,
        minOutputAmount: BN,
        isPassMarket: boolean,
        proposalNumber: number
    ) {
        return ixs.swapHandler(
            this,
            isQuoteToBase,
            inputAmount,
            minOutputAmount,
            isPassMarket,
            proposalNumber
        )
    }

}

