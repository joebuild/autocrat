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
        400,
        false,
      );
      await ixh.bankrun(banksClient);

      [permissionlessAmmAddr] = getAmmAddr(
        autocratClient.ammProgram.programId,
        META,
        USDC,
        400,
        PublicKey.default
      );

      const permissionlessAmmAcc = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      assert.equal(permissionlessAmmAcc.baseMint.toBase58(), META.toBase58());
      assert.equal(permissionlessAmmAcc.quoteMint.toBase58(), USDC.toBase58());
      assert.equal(permissionlessAmmAcc.swapFeeBps, 400);
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

      const permissionedAmmAcc = await autocratClient.ammProgram.account.amm.fetch(permissionedAccessibleAmmAddr);

      assert.equal(permissionedAmmAcc.baseMint.toBase58(), META.toBase58());
      assert.equal(permissionedAmmAcc.quoteMint.toBase58(), USDC.toBase58());
      assert.equal(permissionedAmmAcc.swapFeeBps, 300);
      assert.equal(permissionedAmmAcc.permissionedCaller.toBase58(), autocratClient.ammProgram.programId.toBase58());
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

      const permissionedAmmAcc = await autocratClient.ammProgram.account.amm.fetch(permissionedInaccessibleAmmAddr);

      assert.equal(permissionedAmmAcc.baseMint.toBase58(), META.toBase58());
      assert.equal(permissionedAmmAcc.quoteMint.toBase58(), USDC.toBase58());
      assert.equal(permissionedAmmAcc.swapFeeBps, 200);
      assert.equal(permissionedAmmAcc.permissionedCaller.toBase58(), randomAuthCaller.toBase58());
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
        new BN(10 * 10 * 9),
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

  describe("#remove_liquidity", async function () {
    it("remove some liquidity from an amm position", async function () {

      const permissionlessAmmStart = await autocratClient.ammProgram.account.amm.fetch(permissionlessAmmAddr);

      let ammPositionAddr = getAmmPositionAddr(autocratClient.ammProgram.programId, permissionlessAmmAddr, payer.publicKey)[0]
      const ammPositionStart = await autocratClient.ammProgram.account.ammPosition.fetch(ammPositionAddr);

      let ixh = await autocratClient.removeLiquidity(
        permissionlessAmmAddr,
        ammPositionAddr,
        new BN(5_000), // 10_000 removes all liquidity
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
        new BN(10_000), // 10_000 removes all liquidity
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