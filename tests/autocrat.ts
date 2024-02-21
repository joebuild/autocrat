import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
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
import { getATA, getAmmPositionAddr, getDaoAddr, getDaoTreasuryAddr } from "../app/src/utils";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import { InstructionHandler } from "../app/src/InstructionHandler";

describe("autocrat_v1", async function () {
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
        proposalKeypair,
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

        proposalKeypair = Keypair.generate()
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

            // let [proposalAddr] = getProposalAddr(autocratClient.program.programId, proposalNumber);
            // const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalAddr);

            // assert.equal(proposalAcc.descriptionUrl, descriptionUrl);

            // let endingLamports = (await banksClient.getAccount(payer.publicKey)).lamports

            // assert.isAbove(startingLamports, endingLamports + 10 ** 9) // is down at least 1 sol (considering tx fees)
        });

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

            // let [proposalAddr] = getProposalAddr(autocratClient.program.programId, proposalNumber);
            // const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalAddr);

            // assert.equal(proposalAcc.descriptionUrl, descriptionUrl);

            // let endingLamports = (await banksClient.getAccount(payer.publicKey)).lamports

            // assert.isAbove(startingLamports, endingLamports + 10 ** 9) // is down at least 1 sol (considering tx fees)
        });

        // it("creates a proposal (part one), using the already created proposal instructions", async function () {

        //     let descriptionUrl = "https://metadao.futarchy/proposal-10"

        //     let startingLamports = (await banksClient.getAccount(payer.publicKey)).lamports

        //     let ixh = await autocratClient.createProposalPartOne(
        //         descriptionUrl,
        //         proposalInstructionsAddr
        //     );
        //     await ixh.bankrun(banksClient);

        //     let [proposalAddr] = getProposalAddr(autocratClient.program.programId, proposalNumber);
        //     const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalAddr);

        //     assert.equal(proposalAcc.descriptionUrl, descriptionUrl);

        //     let endingLamports = (await banksClient.getAccount(payer.publicKey)).lamports

        //     assert.isAbove(startingLamports, endingLamports + 10 ** 9) // is down at least 1 sol (considering tx fees)
        // });
    });

    // describe("#create user ATAs for conditional mints", async function () {
    //     it("create user ATAs for conditional mints", async function () {
    //         let conditionalOnPassMetaMint = getConditionalOnPassMetaMintAddr(autocratClient.program.programId, proposalNumber)[0]
    //         let conditionalOnPassUsdcMint = getConditionalOnPassUsdcMintAddr(autocratClient.program.programId, proposalNumber)[0]
    //         let conditionalOnFailMetaMint = getConditionalOnFailMetaMintAddr(autocratClient.program.programId, proposalNumber)[0]
    //         let conditionalOnFailUsdcMint = getConditionalOnFailUsdcMintAddr(autocratClient.program.programId, proposalNumber)[0]

    //         let conditionalOnPassMetaUserATA = getATA(conditionalOnPassMetaMint, autocratClient.provider.publicKey)[0]
    //         let conditionalOnPassUsdcUserATA = getATA(conditionalOnPassUsdcMint, autocratClient.provider.publicKey)[0]
    //         let conditionalOnFailMetaUserATA = getATA(conditionalOnFailMetaMint, autocratClient.provider.publicKey)[0]
    //         let conditionalOnFailUsdcUserATA = getATA(conditionalOnFailUsdcMint, autocratClient.provider.publicKey)[0]

    //         let passMetaAtaIx = createAssociatedTokenAccountInstruction(
    //             autocratClient.provider.publicKey,
    //             conditionalOnPassMetaUserATA,
    //             autocratClient.provider.publicKey,
    //             conditionalOnPassMetaMint,
    //         )

    //         let passUsdcAtaIx = createAssociatedTokenAccountInstruction(
    //             autocratClient.provider.publicKey,
    //             conditionalOnPassUsdcUserATA,
    //             autocratClient.provider.publicKey,
    //             conditionalOnPassUsdcMint,
    //         )

    //         let failMetaAtaIx = createAssociatedTokenAccountInstruction(
    //             autocratClient.provider.publicKey,
    //             conditionalOnFailMetaUserATA,
    //             autocratClient.provider.publicKey,
    //             conditionalOnFailMetaMint,
    //         )

    //         let failUsdcAtaIx = createAssociatedTokenAccountInstruction(
    //             autocratClient.provider.publicKey,
    //             conditionalOnFailUsdcUserATA,
    //             autocratClient.provider.publicKey,
    //             conditionalOnFailUsdcMint,
    //         )

    //         let ixh = new InstructionHandler(
    //             [passMetaAtaIx, passUsdcAtaIx, failMetaAtaIx, failUsdcAtaIx],
    //             [],
    //             autocratClient
    //         )

    //         await ixh.bankrun(banksClient)
    //     });
    // });

    // describe("#create_position", async function () {
    //     it("create new pass-market amm position (just the account, adding liquidity is separate)", async function () {

    //         let [passMarketAmmAddr] = getPassMarketAmmAddr(autocratClient.program.programId, proposalNumber);

    //         let ixh = await autocratClient.createAmmPosition(
    //             passMarketAmmAddr
    //         );
    //         await ixh.bankrun(banksClient);

    //         let passMarketPositionAddr = getAmmPositionAddr(autocratClient.program.programId, passMarketAmmAddr, payer.publicKey)[0]
    //         const passMarketPosition = await autocratClient.program.account.ammPosition.fetch(passMarketPositionAddr);

    //         assert.equal(passMarketPosition.amm.toBase58(), passMarketAmmAddr.toBase58());
    //         assert.equal(passMarketPosition.user.toBase58(), payer.publicKey.toBase58());
    //     });

    //     it("create new fail-market amm position (just the account, adding liquidity is separate)", async function () {

    //         let [failMarketAmmAddr] = getFailMarketAmmAddr(autocratClient.program.programId, proposalNumber);

    //         let ixh = await autocratClient.createAmmPosition(
    //             failMarketAmmAddr
    //         );
    //         await ixh.bankrun(banksClient);

    //         let failMarketPositionAddr = getAmmPositionAddr(autocratClient.program.programId, failMarketAmmAddr, payer.publicKey)[0]
    //         const failMarketPosition = await autocratClient.program.account.ammPosition.fetch(failMarketPositionAddr);

    //         assert.equal(failMarketPosition.amm.toBase58(), failMarketAmmAddr.toBase58());
    //         assert.equal(failMarketPosition.user.toBase58(), payer.publicKey.toBase58());
    //     });
    // });

    // describe("#create_proposal_part_two", async function () {
    //     it("finish creating a proposal (part two), and deposit liquidity into amms", async function () {

    //         let initialPassMarketPriceQuoteUnitsPerBaseUnitBps = new BN(35.5 * 10000)
    //         let initialFailMarketPriceQuoteUnitsPerBaseUnitBps = new BN(24.2 * 10000)
    //         let quoteLiquidityAmountPerAmm = new BN(1000 * 10 ** 6)

    //         let startingLamports = (await banksClient.getAccount(payer.publicKey)).lamports

    //         let ixh = await autocratClient.createProposalPartTwo(
    //             initialPassMarketPriceQuoteUnitsPerBaseUnitBps,
    //             initialFailMarketPriceQuoteUnitsPerBaseUnitBps,
    //             quoteLiquidityAmountPerAmm
    //         );
    //         await ixh
    //             .setComputeUnits(400_000)
    //             .bankrun(banksClient);

    //         let proposalAddr = getProposalAddr(autocratClient.program.programId, proposalNumber)[0]
    //         const proposalAcc = await autocratClient.program.account.proposal.fetch(proposalAddr);
    //         assert.isAbove(proposalAcc.slotEnqueued.toNumber(), 0)

    //         let [passMarketAmmAddr] = getPassMarketAmmAddr(autocratClient.program.programId, proposalNumber);
    //         const passMarketAmmAcc = await autocratClient.program.account.amm.fetch(passMarketAmmAddr);
    //         assert.isAbove(passMarketAmmAcc.ltwapSlotUpdated.toNumber(), 0)

    //         let [failMarketAmmAddr] = getFailMarketAmmAddr(autocratClient.program.programId, proposalNumber);
    //         const failMarketAmmAcc = await autocratClient.program.account.amm.fetch(failMarketAmmAddr);
    //         assert.isAbove(failMarketAmmAcc.ltwapSlotUpdated.toNumber(), 0)

    //         let endingLamports = (await banksClient.getAccount(payer.publicKey)).lamports
    //         assert.isAbove(endingLamports, startingLamports + 0.95 * 10 ** 9) // is up more than 0.95 sol (considering tx fees)
    //     });
    // });

    // describe("#mint_conditional_tokens", async function () {
    //     it("mint conditional tokens for proposal", async function () {

    //         let ixh = await autocratClient.mintConditionalTokens(
    //             new BN(10 * 10 ** 9),
    //             new BN(100 * 10 ** 6),
    //             proposalNumber
    //         );
    //         await ixh.bankrun(banksClient);

    //         let conditionalOnPassMetaMint = getConditionalOnPassMetaMintAddr(autocratClient.program.programId, proposalNumber)[0]
    //         let conditionalOnPassUsdcMint = getConditionalOnPassUsdcMintAddr(autocratClient.program.programId, proposalNumber)[0]
    //         let conditionalOnFailMetaMint = getConditionalOnFailMetaMintAddr(autocratClient.program.programId, proposalNumber)[0]
    //         let conditionalOnFailUsdcMint = getConditionalOnFailUsdcMintAddr(autocratClient.program.programId, proposalNumber)[0]

    //         let conditionalOnPassMetaUserATA = getATA(conditionalOnPassMetaMint, autocratClient.provider.publicKey)[0]
    //         let conditionalOnPassUsdcUserATA = getATA(conditionalOnPassUsdcMint, autocratClient.provider.publicKey)[0]
    //         let conditionalOnFailMetaUserATA = getATA(conditionalOnFailMetaMint, autocratClient.provider.publicKey)[0]
    //         let conditionalOnFailUsdcUserATA = getATA(conditionalOnFailUsdcMint, autocratClient.provider.publicKey)[0]

    //         assert.isAbove(Number((await getAccount(banksClient, conditionalOnPassMetaUserATA)).amount), 0);
    //         assert.isAbove(Number((await getAccount(banksClient, conditionalOnPassUsdcUserATA)).amount), 0);
    //         assert.isAbove(Number((await getAccount(banksClient, conditionalOnFailMetaUserATA)).amount), 0);
    //         assert.isAbove(Number((await getAccount(banksClient, conditionalOnFailUsdcUserATA)).amount), 0);
    //     });
    // });

    // describe("#add_liquidity", async function () {
    //     it("add liquidity to an amm/amm position", async function () {

    //         let [passMarketAmmAddr] = getPassMarketAmmAddr(autocratClient.program.programId, proposalNumber);
    //         const passMarketAmmStart = await autocratClient.program.account.amm.fetch(passMarketAmmAddr);

    //         let passMarketPositionAddr = getAmmPositionAddr(autocratClient.program.programId, passMarketAmmAddr, payer.publicKey)[0]
    //         const passMarketPositionStart = await autocratClient.program.account.ammPosition.fetch(passMarketPositionAddr);

    //         let ixh = await autocratClient.addLiquidity(
    //             new BN(10 * 10 * 9),
    //             new BN(100 * 10 ** 6),
    //             true,
    //             proposalNumber
    //         );
    //         await ixh.bankrun(banksClient);

    //         const passMarketAmmEnd = await autocratClient.program.account.amm.fetch(passMarketAmmAddr);
    //         const passMarketPositionEnd = await autocratClient.program.account.ammPosition.fetch(passMarketPositionAddr);

    //         assert.isAbove(passMarketAmmEnd.totalOwnership.toNumber(), passMarketAmmStart.totalOwnership.toNumber());
    //         assert.isAbove(passMarketPositionEnd.ownership.toNumber(), passMarketPositionStart.ownership.toNumber());

    //         assert.isAbove(passMarketAmmEnd.conditionalBaseAmount.toNumber(), passMarketAmmStart.conditionalBaseAmount.toNumber());
    //         assert.isAbove(passMarketAmmEnd.conditionalQuoteAmount.toNumber(), passMarketAmmStart.conditionalQuoteAmount.toNumber());
    //     });
    // });

    // describe("#remove_liquidity", async function () {
    //     it("remove liquidity from an amm/amm position", async function () {

    //         let [passMarketAmmAddr] = getPassMarketAmmAddr(autocratClient.program.programId, proposalNumber);
    //         const passMarketAmmStart = await autocratClient.program.account.amm.fetch(passMarketAmmAddr);

    //         let passMarketPositionAddr = getAmmPositionAddr(autocratClient.program.programId, passMarketAmmAddr, payer.publicKey)[0]
    //         const passMarketPositionStart = await autocratClient.program.account.ammPosition.fetch(passMarketPositionAddr);

    //         let [proposalAddr] = getProposalAddr(autocratClient.program.programId, proposalNumber);
    //         const proposal = await autocratClient.program.account.proposal.fetch(proposalAddr);

    //         // change clock time to be after proposal is over, so that liquidity can be withdrawn 
    //         const currentClock = await banksClient.getClock();
    //         context.setClock(
    //             new Clock(
    //                 BigInt(proposal.slotEnqueued.toNumber() + dao.slotsPerProposal.toNumber() + 1),
    //                 currentClock.epochStartTimestamp,
    //                 currentClock.epoch,
    //                 currentClock.leaderScheduleEpoch,
    //                 50n,
    //             ),
    //         );

    //         let ixh = await autocratClient.removeLiquidity(
    //             new BN(10_000), // 10_000 removes all liquidity
    //             true,
    //             proposalNumber
    //         );
    //         await ixh.bankrun(banksClient);

    //         const passMarketAmmEnd = await autocratClient.program.account.amm.fetch(passMarketAmmAddr);
    //         const passMarketPositionEnd = await autocratClient.program.account.ammPosition.fetch(passMarketPositionAddr);

    //         assert.isBelow(passMarketAmmEnd.totalOwnership.toNumber(), passMarketAmmStart.totalOwnership.toNumber());
    //         assert.isBelow(passMarketPositionEnd.ownership.toNumber(), passMarketPositionStart.ownership.toNumber());
    //         assert.equal(passMarketPositionEnd.ownership.toNumber(), 0);

    //         assert.isBelow(passMarketAmmEnd.conditionalBaseAmount.toNumber(), passMarketAmmStart.conditionalBaseAmount.toNumber());
    //         assert.isBelow(passMarketAmmEnd.conditionalQuoteAmount.toNumber(), passMarketAmmStart.conditionalQuoteAmount.toNumber());
    //     });
    // });
});