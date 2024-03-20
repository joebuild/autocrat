import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { AnchorProvider, BN, Wallet } from "@coral-xyz/anchor";
import path from "path";
import dotenv from "dotenv";
import { readFileSync } from "fs";
import { AutocratClient } from "../app/src/AutocratClient";
import { getDaoAddr } from "../app/src/utils";
import { MEMO_PROGRAM_ID } from "@solana/spl-memo";

async function submitProposal() {
  const rpcUrl = "my-rpc-node";
  const connectionConfig = {};
  const priorityFee = 10_000;

  const envPath = path.resolve(__dirname, "..", ".env"); // .env should include: KEYPAIR_PATH=/path/to/my/keypair.json
  dotenv.config({ path: envPath });

  const keypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(readFileSync(process.env.KEYPAIR_PATH, "utf-8")))
  );
  const wallet = new Wallet(keypair);

  const connection = new Connection(rpcUrl, connectionConfig);
  const provider = new AnchorProvider(
    connection,
    wallet,
    AnchorProvider.defaultOptions()
  );
  const autocratClient = await AutocratClient.createClient({ provider });

  const daoAddr = getDaoAddr(autocratClient.program.programId)[0];
  let daoAccount = await autocratClient.program.account.dao.fetch(daoAddr);

  // const proposalNumber = 10
  const proposalNumber = daoAccount.proposalCount.toNumber();

  const proposalUrl = "url-for-my-cool-proposal";

  const passMarketInitialPrice = 1030;
  const failMarketInitialPrice = 1000;

  // ==== calculate LP deposit amounts
  const minUsdcLiquidity =
    daoAccount.ammInitialQuoteLiquidityAmount.toNumber() / 10 ** 6;

  const cMetaForPassMarket =
    (minUsdcLiquidity / passMarketInitialPrice) * 10 ** 9;
  const cMetaForFailMarket =
    (minUsdcLiquidity / failMarketInitialPrice) * 10 ** 9;

  const cMetaToMint = Math.max(cMetaForPassMarket, cMetaForFailMarket);

  // ==== create proposal
  let createPropIxh = await autocratClient.createProposal(
    proposalNumber,
    proposalUrl,
    new BN(cMetaToMint),
    daoAccount.ammInitialQuoteLiquidityAmount
  );
  await createPropIxh.setPriorityFee(priorityFee).rpc();

  // ==== create proposal instruction account
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

  let createPropIxIxh = await autocratClient.createProposalInstructions(
    proposalNumber,
    [memoInstruction]
  );
  await createPropIxIxh.setPriorityFee(priorityFee).rpc();

  // ==== create pass market
  let createPassMarketIxh = await autocratClient.createProposalMarketSide(
    proposalNumber,
    true,
    new BN(cMetaForPassMarket),
    daoAccount.ammInitialQuoteLiquidityAmount
  );
  await createPassMarketIxh
    .setPriorityFee(priorityFee)
    .setComputeUnits(400_000)
    .rpc();

  // ==== create fail market
  let createFailMarketIxh = await autocratClient.createProposalMarketSide(
    proposalNumber,
    false,
    new BN(cMetaForFailMarket),
    daoAccount.ammInitialQuoteLiquidityAmount
  );
  await createFailMarketIxh
    .setPriorityFee(priorityFee)
    .setComputeUnits(400_000)
    .rpc();

  // ==== submit proposal
  let submitProposalIxh = await autocratClient.submitProposal(proposalNumber);
  await submitProposalIxh
    .setPriorityFee(priorityFee)
    .setComputeUnits(400_000)
    .rpc();
}

submitProposal();
