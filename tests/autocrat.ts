import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";

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
import { getDaoAddr, getDaoTreasuryAddr, sleep } from "../app/src/utils";
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
      
      let ix = await autocratClient.initializeDao(META, USDC);
      await ix.bankrun(banksClient);

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

      let ix = await autocratClient.updateDao({
        passThresholdBps: new BN(123),
        baseBurnLamports: null,
        burnDecayPerSlotLamports: null,
        slotsPerProposal: null,
        ammInitialQuoteLiquidityAtomic: null,
        ammSwapFeeBps: null
      });
      await ix.bankrun(banksClient);

      [dao] = getDaoAddr(autocratClient.program.programId);
      const daoAcc = await autocratClient.program.account.dao.fetch(dao);

      assert.equal(daoAcc.passThresholdBps, 123);
    });
  });
});