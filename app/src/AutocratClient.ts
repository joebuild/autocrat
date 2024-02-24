import { AnchorProvider, Program } from "@coral-xyz/anchor";
import {
    AccountMeta,
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

    async createProposal(
        proposalNumber: number,
        descriptionUrl: string,
        condMetaToMint: BN,
        condUsdcToMint: BN,
    ) {
        return ixs.createProposalHandler(
            this,
            proposalNumber,
            descriptionUrl,
            condMetaToMint,
            condUsdcToMint,
        )
    }

    async createProposalMarketSide(
        proposalNumber: number,
        isPassMarket: boolean,
        ammBaseAmountDeposit: BN,
        ammQuoteAmountDeposit: BN,
        ammProgram = AMM_PROGRAM_ID,
    ) {
        return ixs.createProposalMarketSideHandler(
            this,
            proposalNumber,
            isPassMarket,
            ammBaseAmountDeposit,
            ammQuoteAmountDeposit,
            ammProgram
        )
    }

    async submitProposal(
        proposalNumber: number,
        proposalInstructions: PublicKey,
        ammProgram = AMM_PROGRAM_ID,
    ) {
        return ixs.submitProposalHandler(
            this,
            proposalNumber,
            proposalInstructions,
            ammProgram
        )
    }

    async finalizeProposal(
        proposalNumber: number,
        accounts: AccountMeta[]
    ) {
        return ixs.finalizeProposalHandler(
            this,
            proposalNumber,
            accounts
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

    async mergeConditionalTokens(
        proposalAddr: PublicKey,
        metaAmount: BN,
        usdcAmount: BN,
    ) {
        return ixs.mergeConditionalTokensHandler(
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
        proposalAddr: PublicKey,
        amm: PublicKey,
        ammProgram = AMM_PROGRAM_ID,
    ) {
        return ixs.createAmmPositionCpiHandler(
            this,
            proposalAddr,
            amm,
            ammProgram
        )
    }

    async addLiquidityCpi(
        proposalAddr: PublicKey,
        ammAddr: PublicKey,
        maxBaseAmount: BN,
        maxQuoteAmount: BN,
        ammProgram = AMM_PROGRAM_ID,
    ) {
        return ixs.addLiquidityCpiHandler(
            this,
            proposalAddr,
            ammAddr,
            maxBaseAmount,
            maxQuoteAmount,
            ammProgram
        )
    }

    async removeLiquidityCpi(
        proposalAddr: PublicKey,
        ammAddr: PublicKey,
        removeBps: BN,
        ammProgram = AMM_PROGRAM_ID,
    ) {
        return ixs.removeLiquidityCpiHandler(
            this,
            proposalAddr,
            ammAddr,
            removeBps,
            ammProgram
        )
    }

    async swapCpi(
        proposalAddr: PublicKey,
        ammAddr: PublicKey,
        isQuoteToBase: boolean,
        inputAmount: BN,
        minOutputAmount: BN,
        ammProgram = AMM_PROGRAM_ID,
    ) {
        return ixs.swapCpiHandler(
            this,
            proposalAddr,
            ammAddr,
            isQuoteToBase,
            inputAmount,
            minOutputAmount,
            ammProgram
        )
    }
}

