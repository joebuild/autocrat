import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
// import { BankrunProvider } from "anchor-bankrun";
import { MEMO_PROGRAM_ID } from "@solana/spl-memo";

import {
  startAnchor,
  Clock,
  BanksClient,
  ProgramTestContext,
} from "solana-bankrun";

import {
  createMint,
  createAssociatedTokenAccount,
  getAccount,
  mintTo,
} from "spl-token-bankrun";

import { assert } from "chai";

import { AutocratClient } from "../app/src/AutocratClient";
import {
  getATA,
  getAmmPositionAddr,
  getDaoAddr,
  getDaoTreasuryAddr,
  getProposalAddr,
  getProposalInstructionsAddr,
  sleep,
} from "../app/src/utils";
import { Keypair, PublicKey } from "@solana/web3.js";
import { AmmClient } from "../app/src/AmmClient";
import { InstructionHandler } from "../app/src/InstructionHandler";
import { BankrunProvider } from "anchor-bankrun";
import { expectFailure, fastForward } from "./utils";
import { AMM_PROGRAM_ID } from "../app/src/constants";
import { Dao } from "../app/src";

describe("autocrat", async function () {
  let provider: any,
    autocratClient: AutocratClient,
    ammClient: AmmClient,
    payer: Keypair,
    context: ProgramTestContext,
    banksClient: BanksClient,
    daoId: PublicKey,
    daoAddr: PublicKey,
    dao: Dao,
    daoTreasury: PublicKey,
    META: PublicKey,
    USDC: PublicKey,
    proposalInstructionsAddr: PublicKey,
    treasuryMetaAccount: PublicKey,
    treasuryUsdcAccount: PublicKey,
    userMetaAccount: PublicKey,
    userUsdcAccount: PublicKey,
    proposalNumber: number,
    proposalAddr: PublicKey;

  before(async function () {
    context = await startAnchor("./", [], []);
    banksClient = context.banksClient;
    provider = new BankrunProvider(context);
    anchor.setProvider(provider);

    autocratClient = await AutocratClient.createClient({ provider });
    ammClient = await AmmClient.createClient({ provider });

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
  });

  beforeEach(async function () {
    await fastForward(context, 1n);
  });

  describe("#initialize_dao", async function () {
    it("initializes the DAO", async function () {
      daoId = Keypair.generate().publicKey;

      let ixh = await autocratClient.initializeDao(daoId, META, USDC);
      await ixh.bankrun(banksClient);

      [daoAddr] = getDaoAddr(autocratClient.program.programId, daoId);
      [daoTreasury] = getDaoTreasuryAddr(
        autocratClient.program.programId,
        daoId
      );

      const daoAcc = await autocratClient.program.account.dao.fetch(daoAddr);

      assert(daoAcc.metaMint.equals(META));
      assert(daoAcc.usdcMint.equals(USDC));

      assert.equal(daoAcc.proposalCount.toNumber(), 10);
      assert.equal(daoAcc.passThresholdBps.toNumber(), 500);

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

      mintTo(
        banksClient,
        payer,
        META,
        userMetaAccount,
        payer.publicKey,
        1_000 * 10 ** 9
      );
      mintTo(
        banksClient,
        payer,
        USDC,
        userUsdcAccount,
        payer.publicKey,
        1_000_000 * 10 ** 6
      );
    });

    it("fails when reusing the ID", async function () {
      let ixh = await autocratClient.initializeDao(daoId, META, USDC);
      await expectFailure(ixh.bankrun(banksClient));
    });

    it("succeeds when using another ID", async function () {
      const otherDaoId = Keypair.generate().publicKey;

      let ixh = await autocratClient.initializeDao(otherDaoId, META, USDC);
      await ixh.bankrun(banksClient);
    });
  });

  describe("#update_dao", async function () {
    it("updates the DAO", async function () {
      let ixh = await autocratClient.updateDao(daoId, {
        passThresholdBps: new BN(123),
        proposalDurationSlots: new BN(69_420),
        finalizeWindowSlots: new BN(69_420),
        proposalFeeUsdc: new BN(1000 * 10 ** 6),
        ammInitialQuoteLiquidityAmount: new BN(100_000_005),
        ammSwapFeeBps: new BN(600),
        ammLtwapDecimals: 9,
      });
      await ixh.bankrun(banksClient);

      let [daoAddr] = getDaoAddr(autocratClient.program.programId, daoId);
      dao = await autocratClient.program.account.dao.fetch(daoAddr);

      proposalNumber = dao.proposalCount.toNumber();
      proposalAddr = getProposalAddr(
        autocratClient.program.programId,
        daoAddr,
        proposalNumber
      )[0];

      assert.equal(dao.passThresholdBps.toNumber(), 123);
      assert.equal(dao.proposalDurationSlots.toNumber(), 69_420);
      assert.equal(dao.finalizeWindowSlots.toNumber(), 69_420);
      assert.equal(dao.ammInitialQuoteLiquidityAmount.toNumber(), 100_000_005);
      assert.equal(dao.ammSwapFeeBps.toNumber(), 600);
      assert.equal(dao.ammLtwapDecimals, 9);
    });
  });

  describe("#create_proposal", async function () {
    it("creates a proposal", async function () {
      const proposalDescription = "https://based-proposals.com/10";

      let daoAddr = getDaoAddr(autocratClient.program.programId, daoId)[0];
      const daoAccStart = await autocratClient.program.account.dao.fetch(
        daoAddr
      );

      let ixh = await autocratClient.createProposal(
        daoId,
        proposalNumber,
        proposalDescription,
        new BN(10 * 10 ** 9),
        new BN(10_000 * 10 ** 6)
      );
      await ixh.bankrun(banksClient);

      const daoAccEnd = await autocratClient.program.account.dao.fetch(daoAddr);
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );

      assert.equal(proposalAcc.proposer.toBase58(), payer.publicKey.toBase58());

      assert.equal(
        proposalAcc.proposerInititialConditionalMetaMinted.toNumber(),
        10 * 10 ** 9
      );
      assert.equal(
        proposalAcc.proposerInititialConditionalUsdcMinted.toNumber(),
        10_000 * 10 ** 6
      );

      assert(proposalAcc.state["initialize"]);
      assert.equal(proposalAcc.descriptionUrl, proposalDescription);

      assert.equal(
        daoAccStart.proposalCount.toNumber() + 1,
        daoAccEnd.proposalCount.toNumber()
      );
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

      let ixh = await autocratClient.createProposalInstructions(
        daoId,
        proposalNumber,
        [memoInstruction]
      );
      await ixh.bankrun(banksClient);

      proposalInstructionsAddr = getProposalInstructionsAddr(
        autocratClient.program.programId,
        proposalAddr
      )[0];
      const instructionsAcc =
        await autocratClient.program.account.proposalInstructions.fetch(
          proposalInstructionsAddr
        );

      assert.equal(
        instructionsAcc.proposer.toBase58(),
        autocratClient.provider.publicKey.toBase58()
      );
    });
  });

  describe("#add_proposal_instructions", async function () {
    it("adds a proposal instruction to a proposal instruction account", async function () {
      const memoText = "Proposal #10 hereby passes! (Twice!)";

      const memoInstruction = {
        programId: new PublicKey(MEMO_PROGRAM_ID),
        data: Buffer.from(memoText),
        accounts: [],
      };

      let ixh = await autocratClient.addProposalInstructions(
        daoId,
        proposalNumber,
        [memoInstruction, memoInstruction]
      );
      await ixh.bankrun(banksClient);

      const instructionsAcc =
        await autocratClient.program.account.proposalInstructions.fetch(
          proposalInstructionsAddr
        );

      assert.equal(instructionsAcc.instructions.length, 3);
    });
  });

  describe("#create_proposal_market_side", async function () {
    it("creates a proposal [pass] market", async function () {
      let ixh = await autocratClient.createProposalMarketSide(
        daoId,
        proposalNumber,
        true,
        new BN(10 * 10 ** 9),
        new BN(10_000 * 10 ** 6)
      );
      await ixh.setComputeUnits(400_000).bankrun(banksClient);

      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );

      assert.equal(proposalAcc.isPassMarketCreated, true);
      assert.equal(proposalAcc.isFailMarketCreated, false);

      assert.equal(proposalAcc.proposer.toBase58(), payer.publicKey.toBase58());

      assert.equal(
        proposalAcc.proposerInititialConditionalMetaMinted.toNumber(),
        10 * 10 ** 9
      );
      assert.equal(
        proposalAcc.proposerInititialConditionalUsdcMinted.toNumber(),
        10_000 * 10 ** 6
      );

      const passMarketAmmAddr = proposalAcc.passMarketAmm;
      const ammPositionAddr = getAmmPositionAddr(
        ammClient.program.programId,
        passMarketAmmAddr,
        payer.publicKey
      )[0];

      let ammPosition = await ammClient.program.account.ammPosition.fetch(
        ammPositionAddr
      );

      assert.equal(ammPosition.user.toBase58(), payer.publicKey.toBase58());
      assert.equal(ammPosition.amm.toBase58(), passMarketAmmAddr.toBase58());
      assert.equal(ammPosition.ownership.toNumber(), 10 * 10 ** 9);
    });
  });

  describe("#create_proposal_market_side", async function () {
    it("creates a proposal [fail] market", async function () {
      let ixh = await autocratClient.createProposalMarketSide(
        daoId,
        proposalNumber,
        false,
        new BN(10 * 10 ** 9),
        new BN(10_000 * 10 ** 6)
      );
      await ixh.setComputeUnits(400_000).bankrun(banksClient);

      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );

      assert.equal(proposalAcc.isFailMarketCreated, true);

      const failMarketAmmAddr = proposalAcc.failMarketAmm;
      const ammPositionAddr = getAmmPositionAddr(
        ammClient.program.programId,
        failMarketAmmAddr,
        payer.publicKey
      )[0];

      let ammPosition = await ammClient.program.account.ammPosition.fetch(
        ammPositionAddr
      );

      assert.equal(ammPosition.user.toBase58(), payer.publicKey.toBase58());
      assert.equal(ammPosition.amm.toBase58(), failMarketAmmAddr.toBase58());
      assert.equal(ammPosition.ownership.toNumber(), 10 * 10 ** 9);
    });
  });

  describe("#submit_proposal", async function () {
    it("submit_proposal", async function () {
      const currentClock = await context.banksClient.getClock();
      let proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );
      let startUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.usdcMint, payer.publicKey)[0]
        )
      ).amount;

      let ixh = await autocratClient.submitProposal(daoId, proposalNumber);
      await ixh.setComputeUnits(400_000).bankrun(banksClient);

      let endUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.usdcMint, payer.publicKey)[0]
        )
      ).amount;
      proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );

      assert(proposalAcc.state["pending"]);
      assert(BigInt(proposalAcc.slotEnqueued.toNumber()) >= currentClock.slot);
      assert.equal(startUsdcBalance - endUsdcBalance, BigInt(1000 * 10 ** 6));
    });
  });

  describe("#mint_conditional_tokens", async function () {
    it("mint conditional tokens for proposal", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );

      const metaToMint = 100 * 10 ** 9;
      const usdcToMint = 100_000 * 10 ** 6;

      let startMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.metaMint, payer.publicKey)[0]
        )
      ).amount;
      let startUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.usdcMint, payer.publicKey)[0]
        )
      ).amount;

      let startCondPassMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondPassUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondFailMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondFailUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let ixh = await autocratClient.mintConditionalTokens(
        proposalAddr,
        new BN(metaToMint),
        new BN(usdcToMint)
      );
      await ixh.bankrun(banksClient);

      let endMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.metaMint, payer.publicKey)[0]
        )
      ).amount;
      let endUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.usdcMint, payer.publicKey)[0]
        )
      ).amount;

      let endCondPassMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondPassUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondFailMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondFailUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      assert.equal(
        endCondPassMetaBalance - startCondPassMetaBalance,
        BigInt(metaToMint)
      );
      assert.equal(
        endCondPassUsdcBalance - startCondPassUsdcBalance,
        BigInt(usdcToMint)
      );
      assert.equal(
        endCondFailMetaBalance - startCondFailMetaBalance,
        BigInt(metaToMint)
      );
      assert.equal(
        endCondFailUsdcBalance - startCondFailUsdcBalance,
        BigInt(usdcToMint)
      );

      assert.equal(startMetaBalance - endMetaBalance, BigInt(metaToMint));
      assert.equal(startUsdcBalance - endUsdcBalance, BigInt(usdcToMint));
    });
  });

  describe("#merge_conditional_tokens", async function () {
    it("merge conditional tokens for proposal", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );

      const metaToMerge = 1 * 10 ** 9;
      const usdcToMerge = 1_000 * 10 ** 6;

      let startMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.metaMint, payer.publicKey)[0]
        )
      ).amount;
      let startUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.usdcMint, payer.publicKey)[0]
        )
      ).amount;

      let startCondPassMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondPassUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondFailMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondFailUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let ixh = await autocratClient.mergeConditionalTokens(
        proposalAddr,
        new BN(metaToMerge),
        new BN(usdcToMerge)
      );
      await ixh.bankrun(banksClient);

      let endMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.metaMint, payer.publicKey)[0]
        )
      ).amount;
      let endUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.usdcMint, payer.publicKey)[0]
        )
      ).amount;

      let endCondPassMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondPassUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondFailMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondFailUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      assert.equal(
        startCondPassMetaBalance - endCondPassMetaBalance,
        BigInt(metaToMerge)
      );
      assert.equal(
        startCondPassUsdcBalance - endCondPassUsdcBalance,
        BigInt(usdcToMerge)
      );
      assert.equal(
        startCondFailMetaBalance - endCondFailMetaBalance,
        BigInt(metaToMerge)
      );
      assert.equal(
        startCondFailUsdcBalance - endCondFailUsdcBalance,
        BigInt(usdcToMerge)
      );

      assert.equal(endMetaBalance - startMetaBalance, BigInt(metaToMerge));
      assert.equal(endUsdcBalance - startUsdcBalance, BigInt(usdcToMerge));
    });
  });

  describe("#add_liquidity", async function () {
    it("add liquidity to an amm/amm position (pass)", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );
      const passMarketAmmAddr = proposalAcc.passMarketAmm;

      let ixh = await autocratClient.addLiquidityCpi(
        proposalAddr,
        passMarketAmmAddr,
        new BN(1 * 10 ** 9),
        new BN(1_000 * 10 ** 6),
        new BN(1 * 0.95 * 10 ** 9),
        new BN(1_000 * 0.95 * 10 ** 6)
      );
      await ixh.bankrun(banksClient);

      const ammPositionAddr = getAmmPositionAddr(
        ammClient.program.programId,
        passMarketAmmAddr,
        payer.publicKey
      )[0];

      let ammPosition = await ammClient.program.account.ammPosition.fetch(
        ammPositionAddr
      );

      assert.isAbove(ammPosition.ownership.toNumber(), 1_000_000_000);
    });

    it("add liquidity to an amm/amm position (fail)", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );
      const failMarketAmmAddr = proposalAcc.failMarketAmm;

      let ixh = await autocratClient.addLiquidityCpi(
        proposalAddr,
        failMarketAmmAddr,
        new BN(1 * 10 ** 9),
        new BN(1_000 * 10 ** 6),
        new BN(1 * 0.95 * 10 ** 9),
        new BN(1_000 * 0.95 * 10 ** 6)
      );
      await ixh.bankrun(banksClient);

      const ammPositionAddr = getAmmPositionAddr(
        ammClient.program.programId,
        failMarketAmmAddr,
        payer.publicKey
      )[0];

      let ammPosition = await ammClient.program.account.ammPosition.fetch(
        ammPositionAddr
      );

      assert.isAbove(ammPosition.ownership.toNumber(), 1_000_000_000);
    });
  });

  describe("#swap", async function () {
    it("swap quote to base (pass)", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );
      const passMarketAmmAddr = proposalAcc.passMarketAmm;

      let startCondPassMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondPassUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let ixh = await autocratClient.swapCpi(
        proposalAddr,
        passMarketAmmAddr,
        true,
        new BN(1_000 * 10 ** 6),
        new BN(1)
      );
      await ixh.bankrun(banksClient);

      let endCondPassMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondPassUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      assert(endCondPassMetaBalance > startCondPassMetaBalance);
      assert(startCondPassUsdcBalance > endCondPassUsdcBalance);
    });

    it("swap base to quote (pass)", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );
      const passMarketAmmAddr = proposalAcc.passMarketAmm;

      let startCondPassMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondPassUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let ixh = await autocratClient.swapCpi(
        proposalAddr,
        passMarketAmmAddr,
        false,
        new BN(1 * 10 ** 9),
        new BN(1)
      );
      await ixh.bankrun(banksClient);

      let endCondPassMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondPassUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      assert(endCondPassMetaBalance < startCondPassMetaBalance);
      assert(startCondPassUsdcBalance < endCondPassUsdcBalance);
    });

    it("swap quote to base (fail)", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );
      const failMarketAmmAddr = proposalAcc.failMarketAmm;

      let startCondFailMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondFailUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let ixh = await autocratClient.swapCpi(
        proposalAddr,
        failMarketAmmAddr,
        true,
        new BN(1_000 * 10 ** 6),
        new BN(1)
      );
      await ixh.bankrun(banksClient);

      let endCondFailMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondFailUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      assert(endCondFailMetaBalance > startCondFailMetaBalance);
      assert(startCondFailUsdcBalance > endCondFailUsdcBalance);
    });

    it("swap base to quote (fail)", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );
      const failMarketAmmAddr = proposalAcc.failMarketAmm;

      let startCondFailMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondFailUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let ixh = await autocratClient.swapCpi(
        proposalAddr,
        failMarketAmmAddr,
        false,
        new BN(1 * 10 ** 9),
        new BN(1)
      );
      await ixh.bankrun(banksClient);

      let endCondFailMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondFailUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      assert(endCondFailMetaBalance < startCondFailMetaBalance);
      assert(startCondFailUsdcBalance < endCondFailUsdcBalance);
    });
  });

  describe("#finalize_proposal", async function () {
    it("finalize proposal", async function () {
      await fastForward(
        context,
        BigInt(dao.proposalDurationSlots.toNumber() + 1)
      );

      let accounts = [
        {
          pubkey: MEMO_PROGRAM_ID,
          isSigner: false,
          isWritable: true,
        },
      ];

      let ixh = await autocratClient.finalizeProposal(
        daoId,
        proposalNumber,
        accounts
      );
      await ixh.bankrun(banksClient);

      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );
      console.log(proposalAcc.state);

      assert(!proposalAcc.state["pending"]);

      const passAmm = await ammClient.program.account.amm.fetch(
        proposalAcc.passMarketAmm
      );
      const failAmm = await ammClient.program.account.amm.fetch(
        proposalAcc.failMarketAmm
      );

      let passLtwap =
        passAmm.ltwapLatest.toNumber() / 10 ** passAmm.ltwapDecimals;
      let failLtwap =
        failAmm.ltwapLatest.toNumber() / 10 ** failAmm.ltwapDecimals;

      let thresholdFraction = dao.passThresholdBps.toNumber() / 10_000 + 1;

      if (passLtwap > failLtwap * thresholdFraction) {
        assert(proposalAcc.state["passed"]);
      } else {
        assert(proposalAcc.state["failed"]);
      }
    });
  });

  describe("#remove_liquidity", async function () {
    it("remove liquidity from an amm/amm position (pass)", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );
      const passMarketAmmAddr = proposalAcc.passMarketAmm;

      let startCondPassMetaUserBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondPassUsdcUserBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let startCondPassMetaAmmBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, passMarketAmmAddr)[0]
        )
      ).amount;
      let startCondPassUsdcAmmBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, passMarketAmmAddr)[0]
        )
      ).amount;

      let ixh = await autocratClient.removeLiquidityCpi(
        proposalAddr,
        passMarketAmmAddr,
        new BN(10_000) // 10_000 removes all liquidity
      );
      await ixh.bankrun(banksClient);

      const ammPositionAddr = getAmmPositionAddr(
        ammClient.program.programId,
        passMarketAmmAddr,
        payer.publicKey
      )[0];

      let ammPosition = await ammClient.program.account.ammPosition.fetch(
        ammPositionAddr
      );

      assert.equal(ammPosition.ownership.toNumber(), 0);

      let endCondPassMetaUserBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondPassUsdcUserBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let endCondPassMetaAmmBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, passMarketAmmAddr)[0]
        )
      ).amount;
      let endCondPassUsdcAmmBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, passMarketAmmAddr)[0]
        )
      ).amount;

      assert.equal(endCondPassMetaAmmBalance, BigInt(0));
      assert.equal(endCondPassUsdcAmmBalance, BigInt(0));

      assert.equal(
        endCondPassMetaUserBalance,
        startCondPassMetaUserBalance + startCondPassMetaAmmBalance
      );
      assert.equal(
        endCondPassUsdcUserBalance,
        startCondPassUsdcUserBalance + startCondPassUsdcAmmBalance
      );
    });

    it("remove liquidity from an amm/amm position (fail)", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );
      const failMarketAmmAddr = proposalAcc.failMarketAmm;

      let startCondFailMetaUserBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondFailUsdcUserBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let startCondFailMetaAmmBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, failMarketAmmAddr)[0]
        )
      ).amount;
      let startCondFailUsdcAmmBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, failMarketAmmAddr)[0]
        )
      ).amount;

      let ixh = await autocratClient.removeLiquidityCpi(
        proposalAddr,
        failMarketAmmAddr,
        new BN(10_000) // 10_000 removes all liquidity
      );
      await ixh.bankrun(banksClient);

      const ammPositionAddr = getAmmPositionAddr(
        ammClient.program.programId,
        failMarketAmmAddr,
        payer.publicKey
      )[0];

      let ammPosition = await ammClient.program.account.ammPosition.fetch(
        ammPositionAddr
      );

      assert.equal(ammPosition.ownership.toNumber(), 0);

      let endCondFailMetaUserBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondFailUsdcUserBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let endCondFailMetaAmmBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, failMarketAmmAddr)[0]
        )
      ).amount;
      let endCondFailUsdcAmmBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, failMarketAmmAddr)[0]
        )
      ).amount;

      assert.equal(endCondFailMetaAmmBalance, BigInt(0));
      assert.equal(endCondFailUsdcAmmBalance, BigInt(0));

      assert.equal(
        endCondFailMetaUserBalance,
        startCondFailMetaUserBalance + startCondFailMetaAmmBalance
      );
      assert.equal(
        endCondFailUsdcUserBalance,
        startCondFailUsdcUserBalance + startCondFailUsdcAmmBalance
      );
    });
  });

  describe("#redeem_conditional_tokens", async function () {
    it("redeem conditional tokens from proposal", async function () {
      const proposalAcc = await autocratClient.program.account.proposal.fetch(
        proposalAddr
      );

      let startMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.metaMint, payer.publicKey)[0]
        )
      ).amount;
      let startUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.usdcMint, payer.publicKey)[0]
        )
      ).amount;

      let startCondPassMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondPassUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let startCondFailMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let startCondFailUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let ixh = await autocratClient.redeemConditionalTokens(
        daoId,
        proposalAddr
      );
      await ixh.bankrun(banksClient);

      let endMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.metaMint, payer.publicKey)[0]
        )
      ).amount;
      let endUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.usdcMint, payer.publicKey)[0]
        )
      ).amount;

      let endCondPassMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondPassUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnPassUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      let endCondFailMetaBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailMetaMint, payer.publicKey)[0]
        )
      ).amount;
      let endCondFailUsdcBalance = (
        await getAccount(
          banksClient,
          getATA(proposalAcc.conditionalOnFailUsdcMint, payer.publicKey)[0]
        )
      ).amount;

      assert.equal(
        startCondPassMetaBalance - endCondPassMetaBalance,
        BigInt(endMetaBalance - startMetaBalance)
      );
      assert.equal(
        startCondPassUsdcBalance - endCondPassUsdcBalance,
        BigInt(endUsdcBalance - startUsdcBalance)
      );
      assert.equal(
        startCondFailMetaBalance - endCondFailMetaBalance,
        BigInt(endMetaBalance - startMetaBalance)
      );
      assert.equal(
        startCondFailUsdcBalance - endCondFailUsdcBalance,
        BigInt(endUsdcBalance - startUsdcBalance)
      );
    });
  });
});
