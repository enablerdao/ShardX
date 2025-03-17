"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ShardXClient = void 0;
const axios_1 = __importDefault(require("axios"));
const eventemitter3_1 = require("eventemitter3");
const errors_1 = require("./errors");
/**
 * ShardX API Client
 *
 * Main client for interacting with the ShardX blockchain platform.
 */
class ShardXClient extends eventemitter3_1.EventEmitter {
    /**
     * Create a new ShardX client
     * @param options Client options
     */
    constructor(options = {}) {
        super();
        this.baseUrl = options.baseUrl || 'http://localhost:54868/api/v1';
        this.apiKey = options.apiKey;
        // Create axios instance
        this.axios = axios_1.default.create({
            baseURL: this.baseUrl,
            timeout: options.timeout || 30000,
            headers: {
                'Content-Type': 'application/json',
                'Accept': 'application/json',
                ...(this.apiKey ? { 'X-API-Key': this.apiKey } : {}),
            },
            ...options.axiosConfig,
        });
        // Add response interceptor for error handling
        this.axios.interceptors.response.use((response) => response, (error) => {
            if (error.response) {
                // The request was made and the server responded with a status code
                // that falls out of the range of 2xx
                const errorData = error.response.data?.error || {};
                throw new errors_1.ShardXError(errorData.message || `API request failed with status ${error.response.status}`, error.response.status, errorData.code);
            }
            else if (error.request) {
                // The request was made but no response was received
                throw new errors_1.ShardXError('No response received from server', 0, 'network_error');
            }
            else {
                // Something happened in setting up the request that triggered an Error
                throw new errors_1.ShardXError(`Request failed: ${error.message}`, 0, 'request_error');
            }
        });
    }
    /**
     * Get information about the node
     * @returns Node information
     */
    async getNodeInfo() {
        const response = await this.axios.get('/info');
        return response.data;
    }
    /**
     * Get network statistics
     * @returns Network statistics
     */
    async getNetworkStats() {
        const response = await this.axios.get('/stats');
        return response.data;
    }
    /**
     * Get information about all shards
     * @returns Array of shard information
     */
    async getShards() {
        const response = await this.axios.get('/shards');
        return response.data;
    }
    /**
     * Get information about a specific shard
     * @param shardId Shard ID
     * @returns Shard information
     */
    async getShard(shardId) {
        const response = await this.axios.get(`/shards/${shardId}`);
        return response.data;
    }
    /**
     * Create a new transaction
     * @param txData Transaction data
     * @returns Created transaction
     */
    async createTransaction(txData) {
        const response = await this.axios.post('/transactions', txData);
        return response.data;
    }
    /**
     * Get transaction by ID
     * @param txId Transaction ID
     * @returns Transaction details
     */
    async getTransaction(txId) {
        const response = await this.axios.get(`/transactions/${txId}`);
        return response.data;
    }
    /**
     * Get transaction status
     * @param txId Transaction ID
     * @returns Transaction status
     */
    async getTransactionStatus(txId) {
        const response = await this.axios.get(`/transactions/${txId}/status`);
        return response.data.status;
    }
    /**
     * Get transactions by address
     * @param address Account address
     * @param limit Maximum number of transactions to return
     * @param offset Offset for pagination
     * @returns Array of transactions
     */
    async getTransactionsByAddress(address, limit = 20, offset = 0) {
        const response = await this.axios.get(`/accounts/${address}/transactions`, {
            params: { limit, offset }
        });
        return response.data;
    }
    /**
     * Get block by hash or height
     * @param hashOrHeight Block hash or height
     * @returns Block details
     */
    async getBlock(hashOrHeight) {
        const response = await this.axios.get(`/blocks/${hashOrHeight}`);
        return response.data;
    }
    /**
     * Get latest blocks
     * @param limit Maximum number of blocks to return
     * @returns Array of blocks
     */
    async getLatestBlocks(limit = 10) {
        const response = await this.axios.get('/blocks', {
            params: { limit }
        });
        return response.data;
    }
    /**
     * Get account information
     * @param address Account address
     * @returns Account details
     */
    async getAccount(address) {
        const response = await this.axios.get(`/accounts/${address}`);
        return response.data;
    }
    /**
     * Create a new multisig wallet
     * @param walletData Multisig wallet data
     * @returns Created multisig wallet
     */
    async createMultisigWallet(walletData) {
        const response = await this.axios.post('/multisig/wallets', walletData);
        return response.data;
    }
    /**
     * Get multisig wallet by ID
     * @param walletId Multisig wallet ID
     * @returns Multisig wallet details
     */
    async getMultisigWallet(walletId) {
        const response = await this.axios.get(`/multisig/wallets/${walletId}`);
        return response.data;
    }
    /**
     * Get multisig wallets by owner
     * @param ownerAddress Owner address
     * @returns Array of multisig wallets
     */
    async getMultisigWalletsByOwner(ownerAddress) {
        const response = await this.axios.get(`/accounts/${ownerAddress}/multisig`);
        return response.data;
    }
    /**
     * Get AI prediction for a trading pair
     * @param pair Trading pair (e.g., "BTC/USD")
     * @param period Prediction period (e.g., "hour", "day", "week")
     * @returns Prediction details
     */
    async getPrediction(pair, period = 'hour') {
        const response = await this.axios.get(`/ai/predictions/${pair}`, {
            params: { period }
        });
        return response.data;
    }
    /**
     * Get available trading pairs
     * @returns Array of trading pairs
     */
    async getTradingPairs() {
        const response = await this.axios.get('/ai/pairs');
        return response.data;
    }
    /**
     * Get detailed transaction analysis
     * @param txId Transaction ID
     * @returns Transaction analysis details
     */
    async getTransactionAnalysis(txId) {
        const response = await this.axios.get(`/transactions/${txId}/analysis`);
        return response.data;
    }
    /**
     * Get advanced charts data
     * @param metric Metric to chart (e.g., "transactions", "volume", "fees")
     * @param period Period (e.g., "hour", "day", "week", "month")
     * @param from Start timestamp
     * @param to End timestamp
     * @returns Chart data
     */
    async getChartData(metric, period, from, to) {
        const response = await this.axios.get('/charts', {
            params: { metric, period, from, to }
        });
        return response.data;
    }
}
exports.ShardXClient = ShardXClient;
//# sourceMappingURL=client.js.map