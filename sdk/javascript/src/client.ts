import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
import { EventEmitter } from 'eventemitter3';
import { ShardXError } from './errors';
import {
  NodeInfo,
  Transaction,
  TransactionRequest,
  TransactionStatus,
  Block,
  Account,
  MultisigWallet,
  MultisigWalletRequest,
  TradingPair,
  Prediction,
  ShardInfo,
  NetworkStats
} from './models';

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
export class ShardXClient extends EventEmitter {
  private readonly baseUrl: string;
  private readonly apiKey?: string;
  private readonly axios: AxiosInstance;

  /**
   * Create a new ShardX client
   * @param options Client options
   */
  constructor(options: ShardXClientOptions = {}) {
    super();
    
    this.baseUrl = options.baseUrl || 'http://localhost:54868/api/v1';
    this.apiKey = options.apiKey;
    
    // Create axios instance
    this.axios = axios.create({
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
    this.axios.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response) {
          // The request was made and the server responded with a status code
          // that falls out of the range of 2xx
          const errorData = error.response.data?.error || {};
          throw new ShardXError(
            errorData.message || `API request failed with status ${error.response.status}`,
            error.response.status,
            errorData.code
          );
        } else if (error.request) {
          // The request was made but no response was received
          throw new ShardXError(
            'No response received from server',
            0,
            'network_error'
          );
        } else {
          // Something happened in setting up the request that triggered an Error
          throw new ShardXError(
            `Request failed: ${error.message}`,
            0,
            'request_error'
          );
        }
      }
    );
  }

  /**
   * Get information about the node
   * @returns Node information
   */
  async getNodeInfo(): Promise<NodeInfo> {
    const response = await this.axios.get<NodeInfo>('/info');
    return response.data;
  }

  /**
   * Get network statistics
   * @returns Network statistics
   */
  async getNetworkStats(): Promise<NetworkStats> {
    const response = await this.axios.get<NetworkStats>('/stats');
    return response.data;
  }

  /**
   * Get information about all shards
   * @returns Array of shard information
   */
  async getShards(): Promise<ShardInfo[]> {
    const response = await this.axios.get<ShardInfo[]>('/shards');
    return response.data;
  }

  /**
   * Get information about a specific shard
   * @param shardId Shard ID
   * @returns Shard information
   */
  async getShard(shardId: string): Promise<ShardInfo> {
    const response = await this.axios.get<ShardInfo>(`/shards/${shardId}`);
    return response.data;
  }

  /**
   * Create a new transaction
   * @param txData Transaction data
   * @returns Created transaction
   */
  async createTransaction(txData: TransactionRequest): Promise<Transaction> {
    const response = await this.axios.post<Transaction>('/transactions', txData);
    return response.data;
  }

  /**
   * Get transaction by ID
   * @param txId Transaction ID
   * @returns Transaction details
   */
  async getTransaction(txId: string): Promise<Transaction> {
    const response = await this.axios.get<Transaction>(`/transactions/${txId}`);
    return response.data;
  }

  /**
   * Get transaction status
   * @param txId Transaction ID
   * @returns Transaction status
   */
  async getTransactionStatus(txId: string): Promise<TransactionStatus> {
    const response = await this.axios.get<{ status: TransactionStatus }>(`/transactions/${txId}/status`);
    return response.data.status;
  }

  /**
   * Get transactions by address
   * @param address Account address
   * @param limit Maximum number of transactions to return
   * @param offset Offset for pagination
   * @returns Array of transactions
   */
  async getTransactionsByAddress(address: string, limit = 20, offset = 0): Promise<Transaction[]> {
    const response = await this.axios.get<Transaction[]>(`/accounts/${address}/transactions`, {
      params: { limit, offset }
    });
    return response.data;
  }

  /**
   * Get block by hash or height
   * @param hashOrHeight Block hash or height
   * @returns Block details
   */
  async getBlock(hashOrHeight: string | number): Promise<Block> {
    const response = await this.axios.get<Block>(`/blocks/${hashOrHeight}`);
    return response.data;
  }

  /**
   * Get latest blocks
   * @param limit Maximum number of blocks to return
   * @returns Array of blocks
   */
  async getLatestBlocks(limit = 10): Promise<Block[]> {
    const response = await this.axios.get<Block[]>('/blocks', {
      params: { limit }
    });
    return response.data;
  }

  /**
   * Get account information
   * @param address Account address
   * @returns Account details
   */
  async getAccount(address: string): Promise<Account> {
    const response = await this.axios.get<Account>(`/accounts/${address}`);
    return response.data;
  }

  /**
   * Create a new multisig wallet
   * @param walletData Multisig wallet data
   * @returns Created multisig wallet
   */
  async createMultisigWallet(walletData: MultisigWalletRequest): Promise<MultisigWallet> {
    const response = await this.axios.post<MultisigWallet>('/multisig/wallets', walletData);
    return response.data;
  }

  /**
   * Get multisig wallet by ID
   * @param walletId Multisig wallet ID
   * @returns Multisig wallet details
   */
  async getMultisigWallet(walletId: string): Promise<MultisigWallet> {
    const response = await this.axios.get<MultisigWallet>(`/multisig/wallets/${walletId}`);
    return response.data;
  }

  /**
   * Get multisig wallets by owner
   * @param ownerAddress Owner address
   * @returns Array of multisig wallets
   */
  async getMultisigWalletsByOwner(ownerAddress: string): Promise<MultisigWallet[]> {
    const response = await this.axios.get<MultisigWallet[]>(`/accounts/${ownerAddress}/multisig`);
    return response.data;
  }

  /**
   * Get AI prediction for a trading pair
   * @param pair Trading pair (e.g., "BTC/USD")
   * @param period Prediction period (e.g., "hour", "day", "week")
   * @returns Prediction details
   */
  async getPrediction(pair: string, period: string = 'hour'): Promise<Prediction> {
    const response = await this.axios.get<Prediction>(`/ai/predictions/${pair}`, {
      params: { period }
    });
    return response.data;
  }

  /**
   * Get available trading pairs
   * @returns Array of trading pairs
   */
  async getTradingPairs(): Promise<TradingPair[]> {
    const response = await this.axios.get<TradingPair[]>('/ai/pairs');
    return response.data;
  }

  /**
   * Get detailed transaction analysis
   * @param txId Transaction ID
   * @returns Transaction analysis details
   */
  async getTransactionAnalysis(txId: string): Promise<any> {
    const response = await this.axios.get<any>(`/transactions/${txId}/analysis`);
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
  async getChartData(metric: string, period: string, from?: number, to?: number): Promise<any> {
    const response = await this.axios.get<any>('/charts', {
      params: { metric, period, from, to }
    });
    return response.data;
  }
}