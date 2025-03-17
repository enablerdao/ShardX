import { ShardXClient } from './client';
import { Transaction } from './models';
/**
 * Transaction manager
 *
 * Utility class for working with transactions
 */
export declare class TransactionManager {
    private readonly client;
    /**
     * Create a new transaction manager
     * @param client ShardX client
     */
    constructor(client: ShardXClient);
    /**
     * Wait for transaction confirmation
     * @param txId Transaction ID
     * @param timeout Timeout in milliseconds
     * @param interval Polling interval in milliseconds
     * @returns Confirmed transaction
     */
    waitForConfirmation(txId: string, timeout?: number, interval?: number): Promise<Transaction>;
    /**
     * Get transaction details with analysis
     * @param txId Transaction ID
     * @returns Transaction with analysis
     */
    getTransactionWithAnalysis(txId: string): Promise<any>;
    /**
     * Estimate transaction fee
     * @param from Sender address
     * @param to Recipient address
     * @param amount Amount to send
     * @param data Optional transaction data
     * @returns Estimated fee
     */
    estimateFee(from: string, to: string, amount: string, data?: string): Promise<string>;
    /**
     * Decode transaction data
     * @param data Hex encoded transaction data
     * @returns Decoded data
     */
    decodeTransactionData(data?: string): any;
    /**
     * Encode transaction data
     * @param data Data to encode
     * @returns Hex encoded data
     */
    encodeTransactionData(data: any): string;
}
