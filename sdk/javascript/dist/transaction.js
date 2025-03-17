"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TransactionManager = void 0;
const errors_1 = require("./errors");
/**
 * Transaction manager
 *
 * Utility class for working with transactions
 */
class TransactionManager {
    /**
     * Create a new transaction manager
     * @param client ShardX client
     */
    constructor(client) {
        this.client = client;
    }
    /**
     * Wait for transaction confirmation
     * @param txId Transaction ID
     * @param timeout Timeout in milliseconds
     * @param interval Polling interval in milliseconds
     * @returns Confirmed transaction
     */
    async waitForConfirmation(txId, timeout = 60000, interval = 1000) {
        const startTime = Date.now();
        while (Date.now() - startTime < timeout) {
            const status = await this.client.getTransactionStatus(txId);
            if (status === 'confirmed') {
                return this.client.getTransaction(txId);
            }
            if (status === 'failed') {
                throw new errors_1.TransactionError(`Transaction ${txId} failed`, 400, 'transaction_failed');
            }
            // Wait for the next polling interval
            await new Promise(resolve => setTimeout(resolve, interval));
        }
        throw new errors_1.TransactionError(`Transaction ${txId} confirmation timed out`, 408, 'transaction_timeout');
    }
    /**
     * Get transaction details with analysis
     * @param txId Transaction ID
     * @returns Transaction with analysis
     */
    async getTransactionWithAnalysis(txId) {
        const [transaction, analysis] = await Promise.all([
            this.client.getTransaction(txId),
            this.client.getTransactionAnalysis(txId)
        ]);
        return {
            ...transaction,
            analysis
        };
    }
    /**
     * Estimate transaction fee
     * @param from Sender address
     * @param to Recipient address
     * @param amount Amount to send
     * @param data Optional transaction data
     * @returns Estimated fee
     */
    async estimateFee(from, to, amount, data) {
        // In a real implementation, this would call a fee estimation API
        // For simplicity, we'll return a fixed fee
        return '0.001';
    }
    /**
     * Decode transaction data
     * @param data Hex encoded transaction data
     * @returns Decoded data
     */
    decodeTransactionData(data) {
        if (!data) {
            return null;
        }
        try {
            // Convert hex to string
            const jsonString = Buffer.from(data, 'hex').toString('utf8');
            // Parse JSON
            return JSON.parse(jsonString);
        }
        catch (error) {
            // If not valid JSON, return the raw string
            return Buffer.from(data, 'hex').toString('utf8');
        }
    }
    /**
     * Encode transaction data
     * @param data Data to encode
     * @returns Hex encoded data
     */
    encodeTransactionData(data) {
        let jsonString;
        if (typeof data === 'string') {
            jsonString = data;
        }
        else {
            jsonString = JSON.stringify(data);
        }
        // Convert string to hex
        return Buffer.from(jsonString, 'utf8').toString('hex');
    }
}
exports.TransactionManager = TransactionManager;
//# sourceMappingURL=transaction.js.map