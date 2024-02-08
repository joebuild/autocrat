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
import { getDaoAddr, getDaoTreasuryAddr, getProposalInstructionsAddr, sleep } from "../app/src/utils";
import { PublicKey } from "@solana/web3.js";
import { ProposalInstruction } from "../app/src/types";
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
    treasuryUsdcAccount;

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
});