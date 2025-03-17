import { ShardXClient } from './client';
import { Wallet } from './wallet';
import { MultisigWallet, Transaction } from './models';
/**
 * Multisig transaction
 */
export interface MultisigTransaction extends Transaction {
    /** Signatures collected so far */
    signatures: {
        /** Signer address */
        signer: string;
        /** Signature */
        signature: string;
    }[];
    /** Required signatures */
    requiredSignatures: number;
}
/**
 * Multisig wallet manager
 *
 * Utility class for working with multisig wallets
 */
export declare class MultisigManager {
    private readonly client;
    /**
     * Create a new multisig manager
     * @param client ShardX client
     */
    constructor(client: ShardXClient);
    /**
     * Create a new multisig wallet
     * @param wallet Owner wallet
     * @param name Wallet name
     * @param signers Signer addresses
     * @param requiredSignatures Required signatures
     * @returns Created multisig wallet
     */
    createWallet(wallet: Wallet, name: string, signers: string[], requiredSignatures: number): Promise<MultisigWallet>;
    /**
     * Get multisig wallet by ID
     * @param walletId Multisig wallet ID
     * @returns Multisig wallet
     */
    getWallet(walletId: string): Promise<MultisigWallet>;
    /**
     * Get multisig wallets by owner
     * @param ownerAddress Owner address
     * @returns Array of multisig wallets
     */
    getWalletsByOwner(ownerAddress: string): Promise<MultisigWallet[]>;
    /**
     * Create a multisig transaction
     * @param wallet Signer wallet
     * @param multisigId Multisig wallet ID
     * @param to Recipient address
     * @param amount Amount to send
     * @param data Optional transaction data
     * @returns Created multisig transaction
     */
    createTransaction(wallet: Wallet, multisigId: string, to: string, amount: string, data?: string): Promise<MultisigTransaction>;
    /**
     * Sign a multisig transaction
     * @param wallet Signer wallet
     * @param txId Transaction ID
     * @returns Updated multisig transaction
     */
    signTransaction(wallet: Wallet, txId: string): Promise<MultisigTransaction>;
    /**
     * Execute a multisig transaction
     * @param wallet Signer wallet
     * @param txId Transaction ID
     * @returns Executed transaction
     */
    executeTransaction(wallet: Wallet, txId: string): Promise<Transaction>;
}
