import { AnchorProvider, Program } from "@coral-xyz/anchor";
import {
    AddressLookupTableAccount,
    Keypair,
    PublicKey,
} from "@solana/web3.js";

// @ts-ignore
import * as AutocratIDL from '../../target/idl/autocrat.json';
import { Autocrat as AutocratIDLType } from '../../target/types/autocrat';

import * as ixs from "./instructions/autocrat";
import BN from "bn.js";
import { AMM_PROGRAM_ID, AUTOCRAT_LUTS, AUTOCRAT_PROGRAM_ID } from "./constants";
import { ProposalInstruction, UpdateDaoParams } from "./types";

export type CreateAutocratClientParams = {
    provider: AnchorProvider,
    programId?: PublicKey,
}

export class AutocratClient {
    public readonly provider: AnchorProvider;
    public readonly program: Program<AutocratIDLType>;
    public readonly luts: AddressLookupTableAccount[];

    constructor(
        provider: AnchorProvider,
        programId: PublicKey,
        luts: AddressLookupTableAccount[],
    ) {
        this.provider = provider
        this.program = new Program<AutocratIDLType>(AutocratIDL, programId, provider)
        this.luts = luts
    }

    public static async createClient(createAutocratClientParams: CreateAutocratClientParams): Promise<AutocratClient> {
        let { provider, programId } = createAutocratClientParams;

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
            programId || AUTOCRAT_PROGRAM_ID,
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

    async createProposalMarketSide(
        proposalKeypair: Keypair,
        isPassMarket: boolean,
        condMetaToMint: BN,
        condUsdcToMint: BN,
        ammBaseAmountDeposit: BN,
        ammQuoteAmountDeposit: BN,
        ammProgram = AMM_PROGRAM_ID,
    ) {
        return ixs.createProposalMarketSideHandler(
            this,
            proposalKeypair,
            isPassMarket,
            condMetaToMint,
            condUsdcToMint,
            ammBaseAmountDeposit,
            ammQuoteAmountDeposit,
            ammProgram
        )
    }

    async submitProposal(
        proposalKeypair: Keypair,
        proposalInstructions: PublicKey,
        descriptionUrl: string
    ) {
        return ixs.submitProposalHandler(
            this,
            proposalKeypair,
            proposalInstructions,
            descriptionUrl,
        )
    }

    async finalizeProposal(
        proposalAddr: PublicKey
    ) {
        return ixs.finalizeProposalHandler(
            this,
            proposalAddr,
        )
    }

    async mintConditionalTokens(
        proposalAddr: PublicKey,
        metaAmount: BN,
        usdcAmount: BN,
    ) {
        return ixs.mintConditionalTokensHandler(
            this,
            proposalAddr,
            metaAmount,
            usdcAmount,
        )
    }

    async redeemConditionalTokens(
        proposalAddr: PublicKey
    ) {
        return ixs.redeemConditionalTokensHandler(
            this,
            proposalAddr,
        )
    }

    async createAmmPositionCpi(
        amm: PublicKey
    ) {
        return ixs.createAmmPositionCpiHandler(
            this,
            amm
        )
    }

    async addLiquidityCpi(
        ammAddr: PublicKey,
        ammPositionAddr: PublicKey,
        maxBaseAmount: BN,
        maxQuoteAmount: BN,
    ) {
        return ixs.addLiquidityCpiHandler(
            this,
            ammAddr,
            ammPositionAddr,
            maxBaseAmount,
            maxQuoteAmount
        )
    }

    async removeLiquidityCpi(
        proposalAddr: PublicKey,
        ammAddr: PublicKey,
        removeBps: BN,
    ) {
        return ixs.removeLiquidityCpiHandler(
            this,
            proposalAddr,
            ammAddr,
            removeBps
        )
    }

    async swapCpi(
        proposalAddr: PublicKey,
        ammAddr: PublicKey,
        isQuoteToBase: boolean,
        inputAmount: BN,
        minOutputAmount: BN,
    ) {
        return ixs.swapCpiHandler(
            this,
            proposalAddr,
            ammAddr,
            isQuoteToBase,
            inputAmount,
            minOutputAmount,
        )
    }
}

