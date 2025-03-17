/**
 * Node information
 */
export interface NodeInfo {
    /** Node ID */
    id: string;
    /** Node version */
    version: string;
    /** Node uptime in seconds */
    uptime: number;
    /** Number of connected peers */
    peers: number;
    /** Number of shards */
    shards: number;
    /** Current height */
    height: number;
    /** Sync status */
    synced: boolean;
}
/**
 * Network statistics
 */
export interface NetworkStats {
    /** Total number of transactions */
    totalTransactions: number;
    /** Transactions per second */
    tps: number;
    /** Average block time in seconds */
    avgBlockTime: number;
    /** Total number of accounts */
    totalAccounts: number;
    /** Total number of validators */
    totalValidators: number;
    /** Total staked amount */
    totalStaked: string;
    /** Current network fee in native token */
    currentFee: string;
}
/**
 * Shard information
 */
export interface ShardInfo {
    /** Shard ID */
    id: string;
    /** Shard name */
    name: string;
    /** Number of validators in the shard */
    validators: number;
    /** Current height */
    height: number;
    /** Transactions per second */
    tps: number;
    /** Shard status */
    status: 'active' | 'inactive' | 'syncing';
}
/**
 * Transaction status
 */
export type TransactionStatus = 'pending' | 'confirmed' | 'failed';
/**
 * Transaction
 */
export interface Transaction {
    /** Transaction ID */
    id: string;
    /** Transaction status */
    status: TransactionStatus;
    /** Transaction timestamp */
    timestamp: number;
    /** Sender address */
    from: string;
    /** Recipient address */
    to: string;
    /** Transaction amount */
    amount: string;
    /** Transaction fee */
    fee: string;
    /** Transaction data (hex encoded) */
    data?: string;
    /** Block hash */
    blockHash?: string;
    /** Block height */
    blockHeight?: number;
    /** Shard ID */
    shardId: string;
    /** Parent transaction IDs (for cross-shard transactions) */
    parentIds?: string[];
}
/**
 * Transaction request
 */
export interface TransactionRequest {
    /** Sender address */
    from: string;
    /** Recipient address */
    to: string;
    /** Transaction amount */
    amount: string;
    /** Transaction data (hex encoded) */
    data?: string;
    /** Transaction signature */
    signature: string;
}
/**
 * Block
 */
export interface Block {
    /** Block hash */
    hash: string;
    /** Block height */
    height: number;
    /** Previous block hash */
    previousHash: string;
    /** Block timestamp */
    timestamp: number;
    /** Validator address */
    validator: string;
    /** Number of transactions */
    transactionCount: number;
    /** Transactions in the block */
    transactions?: Transaction[];
    /** Shard ID */
    shardId: string;
    /** Block size in bytes */
    size: number;
}
/**
 * Account
 */
export interface Account {
    /** Account address */
    address: string;
    /** Account balance */
    balance: string;
    /** Account nonce */
    nonce: number;
    /** Account type */
    type: 'standard' | 'contract' | 'multisig';
    /** Account creation timestamp */
    createdAt: number;
    /** Last activity timestamp */
    lastActivity: number;
    /** Staked amount */
    staked?: string;
    /** Delegated amount */
    delegated?: string;
}
/**
 * Multisig wallet
 */
export interface MultisigWallet {
    /** Wallet ID */
    id: string;
    /** Wallet name */
    name: string;
    /** Owner address */
    ownerId: string;
    /** Signer addresses */
    signers: string[];
    /** Required signatures */
    requiredSignatures: number;
    /** Wallet balance */
    balance: string;
    /** Wallet creation timestamp */
    createdAt: number;
}
/**
 * Multisig wallet request
 */
export interface MultisigWalletRequest {
    /** Wallet name */
    name: string;
    /** Owner address */
    ownerId: string;
    /** Signer addresses */
    signers: string[];
    /** Required signatures */
    requiredSignatures: number;
}
/**
 * Trading pair
 */
export interface TradingPair {
    /** Base currency */
    base: string;
    /** Quote currency */
    quote: string;
    /** Display name */
    displayName: string;
    /** Minimum order size */
    minOrderSize: string;
    /** Price precision */
    pricePrecision: number;
}
/**
 * AI prediction
 */
export interface Prediction {
    /** Trading pair */
    pair: TradingPair;
    /** Prediction period */
    period: string;
    /** Current price */
    currentPrice: string;
    /** Predicted price */
    predictedPrice: string;
    /** Confidence level (0-1) */
    confidence: number;
    /** Prediction timestamp */
    timestamp: number;
    /** Historical data points */
    historicalData?: {
        /** Timestamp */
        timestamp: number;
        /** Price */
        price: string;
    }[];
}
