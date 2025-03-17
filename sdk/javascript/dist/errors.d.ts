/**
 * ShardX API Error
 */
export declare class ShardXError extends Error {
    /** HTTP status code */
    status: number;
    /** Error code */
    code?: string;
    /**
     * Create a new ShardX error
     * @param message Error message
     * @param status HTTP status code
     * @param code Error code
     */
    constructor(message: string, status: number, code?: string);
}
/**
 * Transaction error
 */
export declare class TransactionError extends ShardXError {
    /**
     * Create a new transaction error
     * @param message Error message
     * @param status HTTP status code
     * @param code Error code
     */
    constructor(message: string, status: number, code?: string);
}
/**
 * Wallet error
 */
export declare class WalletError extends ShardXError {
    /**
     * Create a new wallet error
     * @param message Error message
     * @param status HTTP status code
     * @param code Error code
     */
    constructor(message: string, status: number, code?: string);
}
/**
 * Multisig error
 */
export declare class MultisigError extends ShardXError {
    /**
     * Create a new multisig error
     * @param message Error message
     * @param status HTTP status code
     * @param code Error code
     */
    constructor(message: string, status: number, code?: string);
}
/**
 * Network error
 */
export declare class NetworkError extends ShardXError {
    /**
     * Create a new network error
     * @param message Error message
     */
    constructor(message: string);
}
/**
 * Validation error
 */
export declare class ValidationError extends ShardXError {
    /** Validation details */
    details?: Record<string, string>;
    /**
     * Create a new validation error
     * @param message Error message
     * @param details Validation details
     */
    constructor(message: string, details?: Record<string, string>);
}
