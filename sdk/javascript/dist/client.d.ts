import { AxiosRequestConfig } from 'axios';
import { EventEmitter } from 'eventemitter3';
import { NodeInfo, Transaction, TransactionRequest, TransactionStatus, Block, Account, MultisigWallet, MultisigWalletRequest, TradingPair, Prediction, ShardInfo, NetworkStats } from './models';
/**
 * ShardX API Client options
 */
export interface ShardXClientOptions {
    /** Base URL for the ShardX API */
    baseUrl?: string;
    /** API key for authentication */
    apiKey?: string;
    /** Request timeout in milliseconds */
    timeout?: number;
    /** Additional axios request config */
    axiosConfig?: AxiosRequestConfig;
}
/**
 * ShardX API Client
 *
 * Main client for interacting with the ShardX blockchain platform.
 */
export declare class ShardXClient extends EventEmitter {
    private readonly baseUrl;
    private readonly apiKey?;
    private readonly axios;
    /**
     * Create a new ShardX client
     * @param options Client options
     */
    constructor(options?: ShardXClientOptions);
    /**
     * Get information about the node
     * @returns Node information
     */
    getNodeInfo(): Promise<NodeInfo>;
    /**
     * Get network statistics
     * @returns Network statistics
     */
    getNetworkStats(): Promise<NetworkStats>;
    /**
     * Get information about all shards
     * @returns Array of shard information
     */
    getShards(): Promise<ShardInfo[]>;
    /**
     * Get information about a specific shard
     * @param shardId Shard ID
     * @returns Shard information
     */
    getShard(shardId: string): Promise<ShardInfo>;
    /**
     * Create a new transaction
     * @param txData Transaction data
     * @returns Created transaction
     */
    createTransaction(txData: TransactionRequest): Promise<Transaction>;
    /**
     * Get transaction by ID
     * @param txId Transaction ID
     * @returns Transaction details
     */
    getTransaction(txId: string): Promise<Transaction>;
    /**
     * Get transaction status
     * @param txId Transaction ID
     * @returns Transaction status
     */
    getTransactionStatus(txId: string): Promise<TransactionStatus>;
    /**
     * Get transactions by address
     * @param address Account address
     * @param limit Maximum number of transactions to return
     * @param offset Offset for pagination
     * @returns Array of transactions
     */
    getTransactionsByAddress(address: string, limit?: number, offset?: number): Promise<Transaction[]>;
    /**
     * Get block by hash or height
     * @param hashOrHeight Block hash or height
     * @returns Block details
     */
    getBlock(hashOrHeight: string | number): Promise<Block>;
    /**
     * Get latest blocks
     * @param limit Maximum number of blocks to return
     * @returns Array of blocks
     */
    getLatestBlocks(limit?: number): Promise<Block[]>;
    /**
     * Get account information
     * @param address Account address
     * @returns Account details
     */
    getAccount(address: string): Promise<Account>;
    /**
     * Create a new multisig wallet
     * @param walletData Multisig wallet data
     * @returns Created multisig wallet
     */
    createMultisigWallet(walletData: MultisigWalletRequest): Promise<MultisigWallet>;
    /**
     * Get multisig wallet by ID
     * @param walletId Multisig wallet ID
     * @returns Multisig wallet details
     */
    getMultisigWallet(walletId: string): Promise<MultisigWallet>;
    /**
     * Get multisig wallets by owner
     * @param ownerAddress Owner address
     * @returns Array of multisig wallets
     */
    getMultisigWalletsByOwner(ownerAddress: string): Promise<MultisigWallet[]>;
    /**
     * Get AI prediction for a trading pair
     * @param pair Trading pair (e.g., "BTC/USD")
     * @param period Prediction period (e.g., "hour", "day", "week")
     * @returns Prediction details
     */
    getPrediction(pair: string, period?: string): Promise<Prediction>;
    /**
     * Get available trading pairs
     * @returns Array of trading pairs
     */
    getTradingPairs(): Promise<TradingPair[]>;
    /**
     * Get detailed transaction analysis
     * @param txId Transaction ID
     * @returns Transaction analysis details
     */
    getTransactionAnalysis(txId: string): Promise<any>;
    /**
     * Get advanced charts data
     * @param metric Metric to chart (e.g., "transactions", "volume", "fees")
     * @param period Period (e.g., "hour", "day", "week", "month")
     * @param from Start timestamp
     * @param to End timestamp
     * @returns Chart data
     */
    getChartData(metric: string, period: string, from?: number, to?: number): Promise<any>;
}
