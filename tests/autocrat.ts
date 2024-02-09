import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
import { MEMO_PROGRAM_ID } from "@solana/spl-memo";

import {
  startAnchor,
  Clock,
  BanksClient,
  ProgramTestContext,
} from "solana-bankrun";

import {
  createMint,
  createAccount,
  createAssociatedTokenAccount,
  mintToOverride,
  getMint,
  getAccount,
} from "spl-token-bankrun";

import { assert } from "chai";

import { Autocrat } from "../target/types/autocrat";
import { AutocratClient } from "../app/src/AutocratClient";
import { getATA, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr, getProposalInstructionsAddr, sleep } from "../app/src/utils";
import { Keypair, PublicKey, Transaction } from "@solana/web3.js";
import { ProposalInstruction } from "../app/src/types";
import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";
const AutocratIDL: Autocrat = require("../target/idl/autocrat.json");

describe("autocrat_v1", async function () {
  let provider,
    connection,
    autocratClient,
    payer,
    context,
    banksClient,
    dao,
    daoTreasury,
    META,
    USDC,
    treasuryMetaAccount,
    treasuryUsdcAccount,
    userMetaAccount,
    userUsdcAccount;

  before(async function () {
    context = await startAnchor(
      "./",
      [],
      []
    );
    banksClient = context.banksClient;
    provider = new BankrunProvider(context);
    anchor.setProvider(provider);
    autocratClient = await AutocratClient.createClient(provider)
    payer = provider.wallet.payer;

    USDC = await createMint(
      banksClient,
      payer,
      payer.publicKey,
      payer.publicKey,
      6
    );

    META = await createMint(banksClient, payer, dao, dao, 9);
  });

  describe("#initialize_dao", async function () {
    it("initializes the DAO", async function () {
      
      let ixh = await autocratClient.initializeDao(META, USDC);
      await ixh.bankrun(banksClient);

      [dao] = getDaoAddr(autocratClient.program.programId);
      [daoTreasury] = getDaoTreasuryAddr(autocratClient.program.programId);

      const daoAcc = await autocratClient.program.account.dao.fetch(dao);

      assert(daoAcc.metaMint.equals(META));
      assert(daoAcc.usdcMint.equals(USDC));

      assert.equal(daoAcc.proposalCount, 4);
      assert.equal(daoAcc.passThresholdBps, 500);
      
      assert.ok(daoAcc.baseBurnLamports.eq(new BN(1_000_000_000).muln(10)));
      assert.ok(daoAcc.burnDecayPerSlotLamports.eq(new BN(23_150)));

      treasuryMetaAccount = await createAssociatedTokenAccount(
        banksClient,
        payer,
        META,
        daoTreasury
      );
      treasuryUsdcAccount = await createAssociatedTokenAccount(
        banksClient,
        payer,
        USDC,
        daoTreasury
      );
      userMetaAccount = await createAssociatedTokenAccount(
        banksClient,
        payer,
        META,
        payer.publicKey
      );
      userUsdcAccount = await createAssociatedTokenAccount(
        banksClient,
        payer,
        USDC,
        payer.publicKey
      );
    });
  });

  describe("#update_dao", async function () {
    it("updates the DAO", async function () {

      let ixh = await autocratClient.updateDao({
        passThresholdBps: new BN(123),
        baseBurnLamports: new BN(11_000_000_000).muln(10),
        burnDecayPerSlotLamports: new BN(44_444),
        slotsPerProposal: new BN(69_420),
        ammInitialQuoteLiquidityAtoms: new BN(100_000_005),
        ammSwapFeeBps: new BN(600),
      });
      await ixh.bankrun(banksClient);

      [dao] = getDaoAddr(autocratClient.program.programId);
      const daoAcc = await autocratClient.program.account.dao.fetch(dao);

      assert.equal(daoAcc.passThresholdBps, 123);
      assert.equal(daoAcc.baseBurnLamports, 110_000_000_000);
      assert.equal(daoAcc.burnDecayPerSlotLamports, 44_444);
      assert.equal(daoAcc.slotsPerProposal, 69_420);
      assert.equal(daoAcc.ammInitialQuoteLiquidityAtoms, 100_000_005);
      assert.equal(daoAcc.ammSwapFeeBps, 600);
    });
  });

  describe("#create_proposal_instructions", async function () {
    it("creates a proposal instructions account", async function () {

      const memoText =
        "I, glorious autocrat of the divine Meta-DAO, " +
        "hereby endorse this endeavor to elevate the futarchy domain. " +
        "Recognize that my utterance echoes not the voice of a mortal but resonates as the proclamation of an omniscient market." +
        "Onward, futards, with the swiftness of the divine!";

      const memoInstruction = {
        programId: new PublicKey(MEMO_PROGRAM_ID),
        data: Buffer.from(memoText),
        accounts: [],
      };

      let ixh = await autocratClient.createProposalInstructions([memoInstruction]);
      await ixh.bankrun(banksClient);

      dao = await autocratClient.program.account.dao.fetch(getDaoAddr(autocratClient.program.programId)[0])

      let [instructionsAddr] = getProposalInstructionsAddr(autocratClient.program.programId, dao.proposalCount);
      const instructionsAcc = await autocratClient.program.account.proposalInstructions.fetch(instructionsAddr);

      assert.equal(instructionsAcc.proposalNumber, dao.proposalCount);
      assert.equal(instructionsAcc.proposer.toBase58(), autocratClient.provider.publicKey.toBase58());
    });
  });

  describe("#add_proposal_instructions", async function () {
    it("adds a proposal instruction to a proposal instruction account", async function () {

      const memoText = "Proposal #4 hereby passes! (Twice!)";

      const memoInstruction = {
        programId: new PublicKey(MEMO_PROGRAM_ID),
        data: Buffer.from(memoText),
        accounts: [],
      };

      dao = await autocratClient.program.account.dao.fetch(getDaoAddr(autocratClient.program.programId)[0])

      let ixh = await autocratClient.addProposalInstructions([memoInstruction, memoInstruction]);
      await ixh.bankrun(banksClient);

      let [instructionsAddr] = getProposalInstructionsAddr(autocratClient.program.programId, dao.proposalCount);
      const instructionsAcc = await autocratClient.program.account.proposalInstructions.fetch(instructionsAddr);

      assert.equal(instructionsAcc.instructions.length, 3);
    });
  });

  describe("#create_proposal_part_one", async function () {
    it("creates a proposal (part one), using the already created proposal instructions", async function () {

      let descriptionUrl = "https://metadao.futarchy/proposal-4"

      dao = await autocratClient.program.account.dao.fetch(getDaoAddr(autocratClient.program.programId)[0])
      let proposalNumber = dao.proposalCount

      let ixh = await autocratClient.createProposalPartOne(
        descriptionUrl,
      );
      await ixh.bankrun(banksClient);

      let [proposalAddr] = getProposalAddr(autocratClient.program.programId, proposalNumber);
      const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalAddr);

      assert.equal(proposalAcc.descriptionUrl, descriptionUrl);
    });
  });

  describe("#create_proposal_part_two", async function () {
    it("finish creating a proposal (part two), and deposit liquidity into amms", async function () {

      let initialPassMarketPriceUnits = 35.5
      let initialFailMarketPriceUnits = 24.2
      let quoteLiquidityAtomsPerAmm = new BN(1000 * 10 ** 6)

      let ixh = await autocratClient.createProposalPartTwo(
        initialPassMarketPriceUnits,
        initialFailMarketPriceUnits,
        quoteLiquidityAtomsPerAmm
      );

      await ixh
        .setComputeUnits(400_000)
        .bankrun(banksClient);

      dao = await autocratClient.program.account.dao.fetch(getDaoAddr(autocratClient.program.programId)[0])
      let proposalNumber = dao.proposalCount - 1

      let proposalAddr = getProposalAddr(autocratClient.program.programId, proposalNumber)[0]
      const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalAddr);
      assert.isAbove(proposalAcc.slotEnqueued.toNumber(), 0)

      let [passMarketAmmAddr] = getPassMarketAmmAddr(autocratClient.program.programId, proposalNumber);
      const passMarketAmmAcc = await autocratClient.program.account.amm.fetch(passMarketAmmAddr);
      assert.isAbove(passMarketAmmAcc.ltwapSlotUpdated.toNumber(), 0)

      let [failMarketAmmAddr] = getFailMarketAmmAddr(autocratClient.program.programId, proposalNumber);
      const failMarketAmmAcc = await autocratClient.program.account.amm.fetch(failMarketAmmAddr);
      assert.isAbove(failMarketAmmAcc.ltwapSlotUpdated.toNumber(), 0)
    });
  });
});