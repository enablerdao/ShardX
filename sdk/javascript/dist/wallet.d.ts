import { ShardXClient } from './client';
import { Transaction } from './models';
/**
 * Wallet options
 */
export interface WalletOptions {
    /** ShardX client */
    client: ShardXClient;
    /** Private key (hex encoded) */
    privateKey?: string;
}
/**
 * ShardX wallet
 *
 * Manages keys and signing transactions
 */
export declare class Wallet {
    private readonly client;
    private readonly privateKey?;
    private readonly publicKey?;
    private readonly address?;
    /**
     * Create a new wallet
     * @param options Wallet options
     */
    constructor(options: WalletOptions);
    /**
     * Create a new random wallet
     * @param client ShardX client
     * @returns New wallet
     */
    static createRandom(client: ShardXClient): Wallet;
    /**
     * Create a wallet from mnemonic phrase
     * @param mnemonic Mnemonic phrase
     * @param client ShardX client
     * @returns Wallet
     */
    static fromMnemonic(mnemonic: string, client: ShardXClient): Wallet;
    /**
     * Get wallet address
     * @returns Wallet address
     */
    getAddress(): string;
    /**
     * Get wallet public key
     * @returns Public key (hex encoded)
     */
    getPublicKey(): string;
    /**
     * Sign a message
     * @param message Message to sign
     * @returns Signature (hex encoded)
     */
    sign(message: string): string;
    /**
     * Create and sign a transaction
     * @param to Recipient address
     * @param amount Amount to send
     * @param data Optional transaction data
     * @returns Signed transaction
     */
    createTransaction(to: string, amount: string, data?: string): Promise<Transaction>;
    /**
     * Get account balance
     * @returns Account balance
     */
    getBalance(): Promise<string>;
    /**
     * Get transaction history
     * @param limit Maximum number of transactions to return
     * @param offset Offset for pagination
     * @returns Array of transactions
     */
    getTransactions(limit?: number, offset?: number): Promise<Transaction[]>;
    /**
     * Derive public key from private key
     * @param privateKey Private key (hex encoded)
     * @returns Public key (hex encoded)
     */
    private derivePublicKey;
    /**
     * Derive address from public key
     * @param publicKey Public key (hex encoded)
     * @returns Address
     */
    private deriveAddress;
}
