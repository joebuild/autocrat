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
import { addComputeUnits, addPriorityFee } from "./utils";

export type SignerOrKeypair = Signer | Keypair

export class InstructionHandler {
    public instructions: TransactionInstruction[];
    public signers: Set<SignerOrKeypair>;
    public client: AutocratClient;

    public computeUnits = 200_000;
    public microLamportsPerComputeUnit = 0;

    public preInstructions: TransactionInstruction[];
    public postInstructions: TransactionInstruction[];

    constructor(
        instructions: TransactionInstruction[],
        signers: SignerOrKeypair[],
        client: AutocratClient,
    ) {
        this.instructions = instructions

        this.signers = new Set()
        signers.forEach(s => this.signers.add(s))

        this.client = client

        this.preInstructions = []
        this.postInstructions = []
    }

    addPreInstructions(instructions: TransactionInstruction[], signers: SignerOrKeypair[] = []): InstructionHandler {
        this.preInstructions = [
            ...instructions,
            ...this.preInstructions
        ]
        signers.forEach(s => this.signers.add(s))
        return this
    }

    addPostInstructions(instructions: TransactionInstruction[], signers: SignerOrKeypair[] = []): InstructionHandler {
        this.postInstructions = [
            ...instructions,
            ...this.postInstructions
        ]
        signers.forEach(s => this.signers.add(s))
        return this
    }

    async getVersionedTransaction(blockhash: Blockhash) {
        this.instructions = [
            ...this.preInstructions,
            ...this.instructions,
            ...this.postInstructions,
        ]

        if (this.microLamportsPerComputeUnit != 0) {
            this.instructions = [
                addPriorityFee(this.microLamportsPerComputeUnit),
                ...this.instructions
            ]
        }

        if (this.computeUnits != 200_000) {
            this.instructions = [
                addComputeUnits(this.computeUnits),
                ...this.instructions
            ]
        }

        const message = new TransactionMessage({
            payerKey: this.client.provider.wallet.publicKey,
            recentBlockhash: blockhash,
            instructions: this.instructions,
        }).compileToV0Message(this.client.luts);

        let tx = new VersionedTransaction(message)
        tx = await this.client.provider.wallet.signTransaction(tx)

        let signersArray = Array.from(this.signers)
        if (this.signers.size) {
            tx.sign(signersArray)
        }

        return tx
    }

    setComputeUnits(computeUnits: number): InstructionHandler {
        this.computeUnits = computeUnits
        return this
    }

    setPriorityFee(microLamportsPerComputeUnit: number): InstructionHandler {
        this.microLamportsPerComputeUnit = microLamportsPerComputeUnit
        return this
    }

    async bankrun(banksClient: BanksClient) {
        try {
            let [blockhash] = (await banksClient.getLatestBlockhash())!;
            const tx = await this.getVersionedTransaction(blockhash);
            return await banksClient.processTransaction(tx);
        } catch (e) {
            console.log(e)
            throw e
        }
    }

    async rpc(opts?: ConfirmOptions) {
        let blockhash = (await this.client.provider.connection.getLatestBlockhash()).blockhash
        const tx = await this.getVersionedTransaction(blockhash);
        return await this.client.provider.sendAndConfirm(tx, undefined, opts)
    }
}