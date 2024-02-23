import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
// import { BankrunProvider } from "anchor-bankrun";
import { MEMO_PROGRAM_ID } from "@solana/spl-memo";

import {
    startAnchor,
    Clock,
} from "solana-bankrun";

import {
    createMint,
    createAssociatedTokenAccount,
    getAccount,
    mintTo,
} from "spl-token-bankrun";

import { assert } from "chai";

import { AutocratClient } from "../app/src/AutocratClient";
import { getATA, getAmmPositionAddr, getDaoAddr, getDaoTreasuryAddr, sleep } from "../app/src/utils";
import { Keypair, PublicKey } from "@solana/web3.js";
import { InstructionHandler } from "../app/src/InstructionHandler";
import { BankrunProvider } from "anchor-bankrun";
import { fastForward } from "./utils";
import { AMM_PROGRAM_ID } from "../app/src/constants";

describe("autocrat", async function () {
    let provider,
        autocratClient,
        payer,
        context,
        banksClient,
        dao,
        daoTreasury,
        META,
        USDC,
        proposalInstructionsAddr,
        treasuryMetaAccount,
        treasuryUsdcAccount,
        userMetaAccount,
        userUsdcAccount,
        proposalNumber,
        proposalKeypair;

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

        proposalKeypair = Keypair.generate()
    });

    beforeEach(async function () {
        await fastForward(context, 1n)
    })

    describe("#initialize_dao", async function () {
        it("initializes the DAO", async function () {

            let ixh = await autocratClient.initializeDao(META, USDC);
            await ixh.bankrun(banksClient);

            [dao] = getDaoAddr(autocratClient.program.programId);
            [daoTreasury] = getDaoTreasuryAddr(autocratClient.program.programId);

            const daoAcc = await autocratClient.program.account.dao.fetch(dao);

            assert(daoAcc.metaMint.equals(META));
            assert(daoAcc.usdcMint.equals(USDC));

            assert.equal(daoAcc.proposalCount, 10);
            assert.equal(daoAcc.passThresholdBps, 500);

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
    });

    describe("#update_dao", async function () {
        it("updates the DAO", async function () {

            let ixh = await autocratClient.updateDao({
                passThresholdBps: new BN(123),
                slotsPerProposal: new BN(69_420),
                ammInitialQuoteLiquidityAmount: new BN(100_000_005),
                ammSwapFeeBps: new BN(600),
                ammLtwapDecimals: 9
            });
            await ixh.bankrun(banksClient);

            let [daoAddr] = getDaoAddr(autocratClient.program.programId);
            dao = await autocratClient.program.account.dao.fetch(daoAddr);
            proposalNumber = dao.proposalCount

            assert.equal(dao.passThresholdBps, 123);
            assert.equal(dao.slotsPerProposal, 69_420);
            assert.equal(dao.ammInitialQuoteLiquidityAmount, 100_000_005);
            assert.equal(dao.ammSwapFeeBps, 600);
            assert.equal(dao.ammLtwapDecimals, 9);
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

            let proposalInstructionsKeypair = Keypair.generate()
            proposalInstructionsAddr = proposalInstructionsKeypair.publicKey

            let ixh = await autocratClient.createProposalInstructions([memoInstruction], proposalInstructionsKeypair);
            await ixh.bankrun(banksClient);

            const instructionsAcc = await autocratClient.program.account.proposalInstructions.fetch(proposalInstructionsAddr);

            assert.equal(instructionsAcc.proposer.toBase58(), autocratClient.provider.publicKey.toBase58());
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

            let ixh = await autocratClient.addProposalInstructions([memoInstruction, memoInstruction], proposalInstructionsAddr);
            await ixh.bankrun(banksClient);

            const instructionsAcc = await autocratClient.program.account.proposalInstructions.fetch(proposalInstructionsAddr);

            assert.equal(instructionsAcc.instructions.length, 3);
        });
    });

    describe("#create_proposal_market_side", async function () {
        it("creates a proposal [pass] market", async function () {

            let ixh = await autocratClient.createProposalMarketSide(
                proposalKeypair,
                true,
                new BN(1_000_000_000),
                new BN(1_000_000_000),
                new BN(1_000_000_000),
                new BN(1_000_000_000),
            )
            await ixh
                .setComputeUnits(400_000)
                .bankrun(banksClient);

            const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalKeypair.publicKey);

            assert.equal(proposalAcc.isPassMarketCreated, true);
            assert.equal(proposalAcc.isFailMarketCreated, false);

            assert.equal(proposalAcc.proposer.toBase58(), payer.publicKey);

            assert.equal(proposalAcc.proposerInititialConditionalMetaMinted.toNumber(), 1_000_000_000);
            assert.equal(proposalAcc.proposerInititialConditionalUsdcMinted.toNumber(), 1_000_000_000);
        });
    });

    describe("#create_proposal_market_side", async function () {
        it("creates a proposal [fail] market", async function () {

            let ixh = await autocratClient.createProposalMarketSide(
                proposalKeypair,
                false,
                new BN(1_000_000_000),
                new BN(1_000_000_000),
                new BN(1_000_000_000),
                new BN(1_000_000_000),
            )
            await ixh
                .setComputeUnits(400_000)
                .bankrun(banksClient);

            const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalKeypair.publicKey);

            assert.equal(proposalAcc.isFailMarketCreated, true);
        });
    });

    describe("#submit_proposal", async function () {
        it("submit_proposal", async function () {

            let ixh = await autocratClient.submitProposal(
                proposalKeypair,
                proposalInstructionsAddr,
                "https://based-proposals.com/10"
            );
            await ixh
                .setComputeUnits(400_000)
                .bankrun(banksClient);

            const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalKeypair.publicKey);

            // TODO
        });
    });

    describe("#mint_conditional_tokens", async function () {
        it("mint conditional tokens for proposal", async function () {

            let ixh = await autocratClient.mintConditionalTokens(
                proposalKeypair.publicKey,
                new BN(10 * 10 ** 9),
                new BN(100 * 10 ** 6),
            );
            await ixh.bankrun(banksClient);

            // TODO
        });
    });

    describe("#create_amm_position", async function () {
        it("create an amm position", async function () {

            const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalKeypair.publicKey);
            const passMarketAmmAddr = proposalAcc.passMarketAmm

            let ixh = await autocratClient.createAmmPositionCpi(
                proposalKeypair.publicKey,
                passMarketAmmAddr
            );
            await ixh.bankrun(banksClient);

            // TODO
        });
    });

    describe("#add_liquidity", async function () {
        it("add liquidity to an amm/amm position", async function () {

            const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalKeypair.publicKey);
            const passMarketAmmAddr = proposalAcc.passMarketAmm

            let ixh = await autocratClient.addLiquidityCpi(
                proposalKeypair.publicKey,
                passMarketAmmAddr,
                new BN(10 * 10 * 9),
                new BN(100 * 10 ** 6),
            );
            await ixh.bankrun(banksClient);

            // TODO
        });
    });

    describe("#swap", async function () {
        it("swap", async function () {

            const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalKeypair.publicKey);
            const passMarketAmmAddr = proposalAcc.passMarketAmm

            let ixh = await autocratClient.swapCpi(
                proposalKeypair.publicKey,
                passMarketAmmAddr,
                true,
                new BN(10 * 10 ** 6),
                new BN(1),
            );
            await ixh.bankrun(banksClient);

            // TODO
        });
    });

    describe("#remove_liquidity", async function () {
        it("remove liquidity from an amm/amm position", async function () {

            const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalKeypair.publicKey);
            const passMarketAmmAddr = proposalAcc.passMarketAmm

            await fastForward(context, BigInt(dao.slotsPerProposal.toNumber() + 1))

            let ixh = await autocratClient.removeLiquidityCpi(
                proposalKeypair.publicKey,
                passMarketAmmAddr,
                new BN(10_000), // 10_000 removes all liquidity
            );
            await ixh.bankrun(banksClient);

            // TODO
        });
    });

    describe("#finalize_proposal", async function () {
        it("finalize proposal", async function () {
            let accounts = [{
                pubkey: MEMO_PROGRAM_ID,
                isSigner: false,
                isWritable: true,
            }]

            let ixh = await autocratClient.finalizeProposal(
                proposalKeypair.publicKey,
                accounts
            );
            await ixh.bankrun(banksClient);

            const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalKeypair.publicKey);

            // TODO
        });
    });

    describe("#redeem_conditional_tokens", async function () {
        it("redeem conditional tokens from proposal", async function () {

            let ixh = await autocratClient.redeemConditionalTokens(
                proposalKeypair.publicKey,
            );
            await ixh.bankrun(banksClient);

            // TODO
        });
    });
});