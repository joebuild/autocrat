import * as anchor from "@coral-xyz/anchor";
import { BN, IdlAccounts, Program } from "@coral-xyz/anchor";
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
  mintTo,
} from "spl-token-bankrun";

import * as chai from 'chai'

import { Amm } from "../target/types/amm";
import { AutocratClient } from "../app/src/AutocratClient";
import { getATA, getAmmAddr, getAmmPositionAddr, getConditionalOnFailMetaMintAddr, getConditionalOnFailUsdcMintAddr, getConditionalOnPassMetaMintAddr, getConditionalOnPassUsdcMintAddr, getDaoAddr, getDaoTreasuryAddr, getFailMarketAmmAddr, getPassMarketAmmAddr, getProposalAddr, sleep } from "../app/src/utils";
import { Keypair, PublicKey, Transaction } from "@solana/web3.js";
import { Proposal, ProposalInstruction } from "../app/src/types";
import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import { InstructionHandler } from "../app/src/InstructionHandler";
import { assert } from "chai";
const AmmIDL: Amm = require("../target/idl/amm.json");

describe("amm_v1", async function () {
  let provider,
    connection,
    autocratClient,
    payer,
    context,
    banksClient,
    permissionlessAmmAddr,
    permissionedAccessibleAmmAddr,
    permissionedInaccessibleAmmAddr,
    daoTreasury,
    META,
    USDC,
    proposalInstructionsAddr,
    treasuryMetaAccount,
    treasuryUsdcAccount,
    userMetaAccount,
    userUsdcAccount,
    proposalNumber
    ;

  before(async function () {
    context = await startAnchor(
      "./",
      [],
      []
    );
    banksClient = context.banksClient;
    provider = new BankrunProvider(context);
    anchor.setProvider(provider);
    autocratClient = await AutocratClient.createClient({ provider })
    payer = provider.wallet.payer;

    META = await createMint(
      banksClient,
      payer,
      payer.publicKey,
      payer.publicKey,
      9
    );
    USDC = await createMint(
      banksClient,
      payer,
      payer.publicKey,
      payer.publicKey,
      6
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

    mintTo(
      banksClient,
      payer,
      META,
      userMetaAccount,
      payer.publicKey,
      1000 * 10 ** 9,
    )
    mintTo(
      banksClient,
      payer,
      USDC,
      userUsdcAccount,
      payer.publicKey,
      10000 * 10 ** 6,
    )
  });

  describe("#create_amm", async function () {
    it("create a permissionless amm", async function () {
      let ixh = await autocratClient.createAmm(
        META,
        USDC,
        1,
        false,
      );
      await ixh.bankrun(banksClient);

      [permissionlessAmmAddr] = getAmmAddr(
        autocratClient.ammProgram.programId,
        META,
        USDC,
        1,
        PublicKey.default
      );

      const permissionlessAmmAcc = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      assert.equal(permissionlessAmmAcc.baseMint.toBase58(), META.toBase58());
      assert.equal(permissionlessAmmAcc.quoteMint.toBase58(), USDC.toBase58());
      assert.equal(permissionlessAmmAcc.baseMintDecimals, 9);
      assert.equal(permissionlessAmmAcc.quoteMintDecimals, 6);
      assert.equal(permissionlessAmmAcc.swapFeeBps, 1);
      assert.equal(permissionlessAmmAcc.permissionedCaller.toBase58(), PublicKey.default.toBase58());
    });

    it("create a permissioned amm (using the amm program as auth)", async function () {
      let ixh = await autocratClient.createAmm(
        META,
        USDC,
        300,
        true,
        autocratClient.ammProgram.programId,
      );
      await ixh.bankrun(banksClient);

      [permissionedAccessibleAmmAddr] = getAmmAddr(
        autocratClient.ammProgram.programId,
        META,
        USDC,
        300,
        autocratClient.ammProgram.programId,
      );

      const permissionedAccessibleAmmAcc = await autocratClient.ammProgram.account.amm.fetch(permissionedAccessibleAmmAddr);

      assert.equal(permissionedAccessibleAmmAcc.baseMint.toBase58(), META.toBase58());
      assert.equal(permissionedAccessibleAmmAcc.quoteMint.toBase58(), USDC.toBase58());
      assert.equal(permissionedAccessibleAmmAcc.baseMintDecimals, 9);
      assert.equal(permissionedAccessibleAmmAcc.quoteMintDecimals, 6);
      assert.equal(permissionedAccessibleAmmAcc.swapFeeBps, 300);
      assert.equal(permissionedAccessibleAmmAcc.permissionedCaller.toBase58(), autocratClient.ammProgram.programId.toBase58());
    });

    it("create a permissioned amm (uncontrolled program)", async function () {
      let randomAuthCaller = Keypair.generate().publicKey

      let ixh = await autocratClient.createAmm(
        META,
        USDC,
        200,
        true,
        randomAuthCaller,
      );
      await ixh.bankrun(banksClient);

      [permissionedInaccessibleAmmAddr] = getAmmAddr(
        autocratClient.ammProgram.programId,
        META,
        USDC,
        200,
        randomAuthCaller,
      );

      const permissionedInaccessibleAmmAcc = await autocratClient.ammProgram.account.amm.fetch(permissionedInaccessibleAmmAddr);

      assert.equal(permissionedInaccessibleAmmAcc.baseMint.toBase58(), META.toBase58());
      assert.equal(permissionedInaccessibleAmmAcc.quoteMint.toBase58(), USDC.toBase58());
      assert.equal(permissionedInaccessibleAmmAcc.baseMintDecimals, 9);
      assert.equal(permissionedInaccessibleAmmAcc.quoteMintDecimals, 6);
      assert.equal(permissionedInaccessibleAmmAcc.swapFeeBps, 200);
      assert.equal(permissionedInaccessibleAmmAcc.permissionedCaller.toBase58(), randomAuthCaller.toBase58());
    });
  });

  describe("#create_position", async function () {
    it("create new permissionless amm position", async function () {
      let ixh = await autocratClient.createAmmPosition(permissionlessAmmAddr);
      await ixh.bankrun(banksClient);

      let permissionlessMarketPositionAddr = getAmmPositionAddr(autocratClient.ammProgram.programId, permissionlessAmmAddr, payer.publicKey)[0]
      const permissionlessMarketPosition = await autocratClient.ammProgram.account.ammPosition.fetch(permissionlessMarketPositionAddr);

      assert.equal(permissionlessMarketPosition.amm.toBase58(), permissionlessAmmAddr.toBase58());
      assert.equal(permissionlessMarketPosition.user.toBase58(), payer.publicKey.toBase58());
    });

    it("create new permissioned amm position", async function () {
      let ixh = await autocratClient.createAmmPosition(permissionedAccessibleAmmAddr);
      await ixh.bankrun(banksClient);

      let permissionedMarketPositionAddr = getAmmPositionAddr(autocratClient.ammProgram.programId, permissionedAccessibleAmmAddr, payer.publicKey)[0]
      const permissionedMarketPosition = await autocratClient.ammProgram.account.ammPosition.fetch(permissionedMarketPositionAddr);

      assert.equal(permissionedMarketPosition.amm.toBase58(), permissionedAccessibleAmmAddr.toBase58());
      assert.equal(permissionedMarketPosition.user.toBase58(), payer.publicKey.toBase58());
    });

    // it("fail to create an unauthorized amm position", async function () {
    //   // todo: confirm that error is thrown
    //   // let ixh = await autocratClient.createAmmPosition(permissionedInaccessibleAmmAddr);
    //   // let sigPromise = ixh.bankrun(banksClient)
    // });
  });

  describe("#add_liquidity", async function () {
    it("add liquidity to an amm position", async function () {
      const permissionlessAmmStart = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      let ammPositionAddr = getAmmPositionAddr(autocratClient.ammProgram.programId, permissionlessAmmAddr, payer.publicKey)[0]
      const ammPositionStart = await autocratClient.ammProgram.account.ammPosition.fetch(ammPositionAddr);

      let ixh = await autocratClient.addLiquidity(
        permissionlessAmmAddr,
        ammPositionAddr,
        new BN(10 * 10 ** 9),
        new BN(100 * 10 ** 6),
      );
      await ixh.bankrun(banksClient);

      const permissionlessAmmEnd = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);
      const ammPositionEnd = await autocratClient.ammProgram.account.ammPosition.fetch(ammPositionAddr);

      assert.isAbove(permissionlessAmmEnd.totalOwnership.toNumber(), permissionlessAmmStart.totalOwnership.toNumber());
      assert.isAbove(ammPositionEnd.ownership.toNumber(), ammPositionStart.ownership.toNumber());

      assert.isAbove(permissionlessAmmEnd.baseAmount.toNumber(), permissionlessAmmStart.baseAmount.toNumber());
      assert.isAbove(permissionlessAmmEnd.quoteAmount.toNumber(), permissionlessAmmStart.quoteAmount.toNumber());
    });
  });

  describe("#swap", async function () {
    it("swap quote to base", async function () {
      const permissionlessAmmStart = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      let ixh = await autocratClient.swap(
        permissionlessAmmAddr,
        true,
        new BN(10 * 10 ** 6),
        new BN(0.8 * 10 ** 9),
      );
      await ixh.bankrun(banksClient);

      const permissionlessAmmEnd = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      assert.isBelow(permissionlessAmmEnd.baseAmount.toNumber(), permissionlessAmmStart.baseAmount.toNumber());
      assert.isAbove(permissionlessAmmEnd.quoteAmount.toNumber(), permissionlessAmmStart.quoteAmount.toNumber());
    });

    it("swap base to quote", async function () {
      const permissionlessAmmStart = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      let ixh = await autocratClient.swap(
        permissionlessAmmAddr,
        false,
        new BN(1 * 10 ** 9),
        new BN(8 * 10 ** 6),
      );
      await ixh.bankrun(banksClient);

      const permissionlessAmmEnd = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      assert.isAbove(permissionlessAmmEnd.baseAmount.toNumber(), permissionlessAmmStart.baseAmount.toNumber());
      assert.isBelow(permissionlessAmmEnd.quoteAmount.toNumber(), permissionlessAmmStart.quoteAmount.toNumber());
    });

    it("swap base to quote and back, should not be profitable", async function () {
      const permissionlessAmmStart = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      let startingBaseSwapAmount = 1 * 10 ** 9

      let ixh1 = await autocratClient.swap(
        permissionlessAmmAddr,
        false,
        new BN(startingBaseSwapAmount),
        new BN(1),
      );
      await ixh1.bankrun(banksClient);

      const permissionlessAmmMiddle = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);
      let quoteReceived = permissionlessAmmStart.quoteAmount.toNumber() - permissionlessAmmMiddle.quoteAmount.toNumber()

      let ixh2 = await autocratClient.swap(
        permissionlessAmmAddr,
        true,
        new BN(quoteReceived),
        new BN(1),
      );
      await ixh2.bankrun(banksClient);

      const permissionlessAmmEnd = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);
      let baseReceived = permissionlessAmmMiddle.baseAmount.toNumber() - permissionlessAmmEnd.baseAmount.toNumber()

      assert.isAbove(startingBaseSwapAmount, baseReceived);
      assert.isBelow(startingBaseSwapAmount * 0.999, baseReceived);
    });

    it("swap quote to base and back, should not be profitable", async function () {
      const permissionlessAmmStart = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      let startingQuoteSwapAmount = 1 * 10 ** 6

      let ixh1 = await autocratClient.swap(
        permissionlessAmmAddr,
        true,
        new BN(startingQuoteSwapAmount),
        new BN(1),
      );
      await ixh1.bankrun(banksClient);

      const permissionlessAmmMiddle = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);
      let baseReceived = permissionlessAmmStart.baseAmount.toNumber() - permissionlessAmmMiddle.baseAmount.toNumber()

      let ixh2 = await autocratClient.swap(
        permissionlessAmmAddr,
        false,
        new BN(baseReceived),
        new BN(1),
      );
      await ixh2.bankrun(banksClient);

      const permissionlessAmmEnd = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);
      let quoteReceived = permissionlessAmmMiddle.quoteAmount.toNumber() - permissionlessAmmEnd.quoteAmount.toNumber()

      assert.isAbove(startingQuoteSwapAmount, quoteReceived);
      assert.isBelow(startingQuoteSwapAmount * 0.999, quoteReceived);
    });

    it("ltwap should go up after buying base, down after selling base", async function () {
      let ixh1 = await autocratClient.updateLTWAP(permissionlessAmmAddr);
      await ixh1.bankrun(banksClient);

      const ltwapStart = await autocratClient.getLTWAP(permissionlessAmmAddr);

      let ixh2 = await autocratClient.swap(
        permissionlessAmmAddr,
        true,
        new BN(2 * 10 ** 9),
      );
      await ixh2.bankrun(banksClient);

      await fastForward(context, 100n)

      let ixh3 = await autocratClient.updateLTWAP(permissionlessAmmAddr);
      await ixh3.bankrun(banksClient);

      const ltwapMiddle = await autocratClient.getLTWAP(permissionlessAmmAddr);

      assert.isAbove(ltwapMiddle, ltwapStart);

      let ixh4 = await autocratClient.swap(
        permissionlessAmmAddr,
        false,
        new BN(20 * 10 ** 6),
      );
      await ixh4.bankrun(banksClient);

      await fastForward(context, 100n)

      let ixh5 = await autocratClient.updateLTWAP(permissionlessAmmAddr);
      await ixh5.bankrun(banksClient);

      const ltwapEnd = await autocratClient.getLTWAP(permissionlessAmmAddr);

      assert.isAbove(ltwapMiddle, ltwapEnd);
    });
  });

  describe("#remove_liquidity", async function () {
    it("remove some liquidity from an amm position", async function () {

      const permissionlessAmmStart = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      let ammPositionAddr = getAmmPositionAddr(autocratClient.ammProgram.programId, permissionlessAmmAddr, payer.publicKey)[0]
      const ammPositionStart = await autocratClient.ammProgram.account.ammPosition.fetch(ammPositionAddr);

      let ixh = await autocratClient.removeLiquidity(
        permissionlessAmmAddr,
        ammPositionAddr,
        new BN(5_000),
      );
      await ixh.bankrun(banksClient);

      const permissionlessAmmEnd = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);
      const ammPositionEnd = await autocratClient.ammProgram.account.ammPosition.fetch(ammPositionAddr);

      assert.isBelow(permissionlessAmmEnd.totalOwnership.toNumber(), permissionlessAmmStart.totalOwnership.toNumber());
      assert.isBelow(ammPositionEnd.ownership.toNumber(), ammPositionStart.ownership.toNumber());

      assert.isBelow(permissionlessAmmEnd.baseAmount.toNumber(), permissionlessAmmStart.baseAmount.toNumber());
      assert.isBelow(permissionlessAmmEnd.quoteAmount.toNumber(), permissionlessAmmStart.quoteAmount.toNumber());
    });

    it("remove all liquidity from an amm position", async function () {

      const permissionlessAmmStart = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      let ammPositionAddr = getAmmPositionAddr(autocratClient.ammProgram.programId, permissionlessAmmAddr, payer.publicKey)[0]
      const ammPositionStart = await autocratClient.ammProgram.account.ammPosition.fetch(ammPositionAddr);

      let ixh = await autocratClient.removeLiquidity(
        permissionlessAmmAddr,
        ammPositionAddr,
        new BN(10_000),
      );
      await ixh.bankrun(banksClient);

      const permissionlessAmmEnd = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);
      const ammPositionEnd = await autocratClient.ammProgram.account.ammPosition.fetch(ammPositionAddr);

      assert.isBelow(permissionlessAmmEnd.totalOwnership.toNumber(), permissionlessAmmStart.totalOwnership.toNumber());
      assert.isBelow(ammPositionEnd.ownership.toNumber(), ammPositionStart.ownership.toNumber());
      assert.equal(ammPositionEnd.ownership.toNumber(), 0);

      assert.isBelow(permissionlessAmmEnd.baseAmount.toNumber(), permissionlessAmmStart.baseAmount.toNumber());
      assert.isBelow(permissionlessAmmEnd.quoteAmount.toNumber(), permissionlessAmmStart.quoteAmount.toNumber());
    });
  });

});

const fastForward = async (context: ProgramTestContext, slots: bigint) => {
  const currentClock = await context.banksClient.getClock();
  context.setClock(
    new Clock(
      currentClock.slot + slots,
      currentClock.epochStartTimestamp,
      currentClock.epoch,
      currentClock.leaderScheduleEpoch,
      50n,
    ),
  );
}