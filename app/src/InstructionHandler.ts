import {
    Blockhash,
    ConfirmOptions,
    Keypair,
    Signer,
    TransactionInstruction,
    TransactionMessage,
    VersionedTransaction,
} from "@solana/web3.js";
import { BanksClient } from "solana-bankrun";
import { AutocratClient } from "./AutocratClient";

export type SignerOrKeypair = Signer | Keypair

export class InstructionHandler {
    public instructions: TransactionInstruction[];
    public signers: SignerOrKeypair[];
    public client: AutocratClient;

    constructor(
        instructions: TransactionInstruction[],
        signers: SignerOrKeypair[],
        client: AutocratClient,
    ) {
        this.instructions = instructions
        this.signers = signers
        this.client = client
    }

    async getVersionedTransaction(blockhash: Blockhash){
        const message = new TransactionMessage({
            payerKey: this.client.provider.wallet.publicKey,
            recentBlockhash: blockhash,
            instructions: this.instructions,
        }).compileToV0Message(this.client.luts);

        let tx = new VersionedTransaction(message)
        tx = await this.client.provider.wallet.signTransaction(tx)
        if (this.signers.length){
            tx.sign(this.signers)
        }

        return tx
    }

    async bankrun(banksClient: BanksClient){
        let [blockhash] = (await banksClient.getLatestBlockhash())!;
        const tx = await this.getVersionedTransaction(blockhash);
        return await banksClient.processTransaction(tx);
    }

    async rpc(opts?: ConfirmOptions){
        let blockhash = (await this.client.provider.connection.getLatestBlockhash()).blockhash
        const tx = await this.getVersionedTransaction(blockhash);
        return await this.client.provider.sendAndConfirm(tx, undefined, opts)
    }
}