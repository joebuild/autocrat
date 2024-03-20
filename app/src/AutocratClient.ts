import { AnchorProvider, Program } from "@coral-xyz/anchor";
import {
  AccountMeta,
  AddressLookupTableAccount,
  Keypair,
  PublicKey,
} from "@solana/web3.js";

import {
  Autocrat as AutocratIDLType,
  IDL as AutocratIDL,
} from "./types/autocrat";

import * as ixs from "./instructions/autocrat";
import BN from "bn.js";
import {
  AMM_PROGRAM_ID,
  AUTOCRAT_LUTS,
  AUTOCRAT_PROGRAM_ID,
} from "./constants";
import {
  ProposalInstruction,
  UpdateDaoParams,
  Dao,
  DaoTreasury,
  Proposal,
  ProposalWrapper,
  ProposalVault,
  ProposalInstructions,
} from "./types";
import { getDaoAddr, getDaoTreasuryAddr, getProposalAddr } from "./utils";

export type CreateAutocratClientParams = {
  provider: AnchorProvider;
  programId?: PublicKey;
};

export class AutocratClient {
  public readonly provider: AnchorProvider;
  public readonly program: Program<AutocratIDLType>;
  public readonly luts: AddressLookupTableAccount[];

  constructor(
    provider: AnchorProvider,
    programId: PublicKey,
    luts: AddressLookupTableAccount[]
  ) {
    this.provider = provider;
    this.program = new Program<AutocratIDLType>(
      AutocratIDL,
      programId,
      provider
    );
    this.luts = luts;
  }

  public static async createClient(
    createAutocratClientParams: CreateAutocratClientParams
  ): Promise<AutocratClient> {
    let { provider, programId } = createAutocratClientParams;

    const getLuts = () =>
      Promise.all(
        AUTOCRAT_LUTS.map((lut) => {
          return provider.connection
            .getAddressLookupTable(lut)
            .then((res) => res.value as AddressLookupTableAccount);
        })
      );

    const luts = await getLuts();

    return new AutocratClient(
      provider,
      programId || AUTOCRAT_PROGRAM_ID,
      luts as AddressLookupTableAccount[]
    );
  }

  async initializeDao(
    daoId: PublicKey,
    metaMint: PublicKey,
    usdcMint: PublicKey
  ) {
    return ixs.initializeDaoHandler(this, daoId, metaMint, usdcMint);
  }

  // this won't ever be called directly (must be called via a proposal), but is here anyway for completeness / testing
  async updateDao(daoId: PublicKey, updateDaoParams: UpdateDaoParams) {
    return ixs.updateDaoHandler(this, daoId, updateDaoParams);
  }

  async createProposalInstructions(
    daoId: PublicKey,
    proposalNumber: number,
    instructions: ProposalInstruction[]
  ) {
    return ixs.createProposalInstructionsHandler(
      this,
      daoId,
      proposalNumber,
      instructions
    );
  }

  async addProposalInstructions(
    daoId: PublicKey,
    proposalNumber: number,
    instructions: ProposalInstruction[]
  ) {
    return ixs.addProposalInstructionsHandler(
      this,
      daoId,
      proposalNumber,
      instructions
    );
  }

  async createProposal(
    daoId: PublicKey,
    proposalNumber: number,
    descriptionUrl: string,
    condMetaToMint: BN,
    condUsdcToMint: BN
  ) {
    return ixs.createProposalHandler(
      this,
      daoId,
      proposalNumber,
      descriptionUrl,
      condMetaToMint,
      condUsdcToMint
    );
  }

  async createProposalMarketSide(
    daoId: PublicKey,
    proposalNumber: number,
    isPassMarket: boolean,
    ammBaseAmountDeposit: BN,
    ammQuoteAmountDeposit: BN,
    ammProgram = AMM_PROGRAM_ID
  ) {
    return ixs.createProposalMarketSideHandler(
      this,
      daoId,
      proposalNumber,
      isPassMarket,
      ammBaseAmountDeposit,
      ammQuoteAmountDeposit,
      ammProgram
    );
  }

  async submitProposal(
    daoId: PublicKey,
    proposalNumber: number,
    ammProgram = AMM_PROGRAM_ID
  ) {
    return ixs.submitProposalHandler(this, daoId, proposalNumber, ammProgram);
  }

  async finalizeProposal(
    daoId: PublicKey,
    proposalNumber: number,
    accounts: AccountMeta[]
  ) {
    return ixs.finalizeProposalHandler(this, daoId, proposalNumber, accounts);
  }

  async mintConditionalTokens(
    proposalAddr: PublicKey,
    metaAmount: BN,
    usdcAmount: BN
  ) {
    return ixs.mintConditionalTokensHandler(
      this,
      proposalAddr,
      metaAmount,
      usdcAmount
    );
  }

  async mergeConditionalTokens(
    proposalAddr: PublicKey,
    metaAmount: BN,
    usdcAmount: BN
  ) {
    return ixs.mergeConditionalTokensHandler(
      this,
      proposalAddr,
      metaAmount,
      usdcAmount
    );
  }

  async redeemConditionalTokens(daoId: PublicKey, proposalAddr: PublicKey) {
    return ixs.redeemConditionalTokensHandler(this, proposalAddr);
  }

  // amm cpi functions

  async createAmmPositionCpi(
    proposalAddr: PublicKey,
    amm: PublicKey,
    ammProgram = AMM_PROGRAM_ID
  ) {
    return ixs.createAmmPositionCpiHandler(this, proposalAddr, amm, ammProgram);
  }

  async addLiquidityCpi(
    proposalAddr: PublicKey,
    ammAddr: PublicKey,
    maxBaseAmount: BN,
    maxQuoteAmount: BN,
    minBaseAmount: BN,
    minQuoteAmount: BN,
    ammProgram = AMM_PROGRAM_ID
  ) {
    return ixs.addLiquidityCpiHandler(
      this,
      proposalAddr,
      ammAddr,
      maxBaseAmount,
      maxQuoteAmount,
      minBaseAmount,
      minQuoteAmount,
      ammProgram
    );
  }

  async removeLiquidityCpi(
    proposalAddr: PublicKey,
    ammAddr: PublicKey,
    removeBps: BN,
    ammProgram = AMM_PROGRAM_ID
  ) {
    return ixs.removeLiquidityCpiHandler(
      this,
      proposalAddr,
      ammAddr,
      removeBps,
      ammProgram
    );
  }

  async swapCpi(
    proposalAddr: PublicKey,
    ammAddr: PublicKey,
    isQuoteToBase: boolean,
    inputAmount: BN,
    minOutputAmount: BN,
    ammProgram = AMM_PROGRAM_ID
  ) {
    return ixs.swapCpiHandler(
      this,
      proposalAddr,
      ammAddr,
      isQuoteToBase,
      inputAmount,
      minOutputAmount,
      ammProgram
    );
  }

  // getter functions

  async getDao(daoId: PublicKey): Promise<Dao> {
    return await this.program.account.dao.fetch(
      getDaoAddr(this.program.programId, daoId)[0]
    );
  }

  async getDaoTreasury(daoId: PublicKey): Promise<DaoTreasury> {
    return await this.program.account.daoTreasury.fetch(
      getDaoTreasuryAddr(this.program.programId, daoId)[0]
    );
  }

  async getAllProposals(daoId: PublicKey): Promise<ProposalWrapper[]> {
    return await this.program.account.proposal.all([
      {
        memcmp: {
          offset: 8,
          bytes: getDaoAddr(this.program.programId, daoId)[0].toBase58(),
        },
      },
    ]);
  }

  async getProposalByNumber(
    daoId: PublicKey,
    proposalNumber: number
  ): Promise<Proposal> {
    const daoAddr = getDaoAddr(this.program.programId, daoId)[0];
    return await this.program.account.proposal.fetch(
      getProposalAddr(this.program.programId, daoAddr, proposalNumber)[0]
    );
  }

  async getProposalInstructionsByNumber(
    daoId: PublicKey,
    proposalNumber: number
  ): Promise<ProposalInstructions> {
    const daoAddr = getDaoAddr(this.program.programId, daoId)[0];
    const proposal = await this.getProposalByNumber(daoAddr, proposalNumber);
    return await this.program.account.proposalInstructions.fetch(
      proposal.instructions
    );
  }
}
