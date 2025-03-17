"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ValidationError = exports.NetworkError = exports.MultisigError = exports.WalletError = exports.TransactionError = exports.ShardXError = void 0;
/**
 * ShardX API Error
 */
class ShardXError extends Error {
    /**
     * Create a new ShardX error
     * @param message Error message
     * @param status HTTP status code
     * @param code Error code
     */
    constructor(message, status, code) {
        super(message);
        this.name = 'ShardXError';
        this.status = status;
        this.code = code;
    }
}
exports.ShardXError = ShardXError;
/**
 * Transaction error
 */
class TransactionError extends ShardXError {
    /**
     * Create a new transaction error
     * @param message Error message
     * @param status HTTP status code
     * @param code Error code
     */
    constructor(message, status, code) {
        super(message, status, code);
        this.name = 'TransactionError';
    }
}
exports.TransactionError = TransactionError;
/**
 * Wallet error
 */
class WalletError extends ShardXError {
    /**
     * Create a new wallet error
     * @param message Error message
     * @param status HTTP status code
     * @param code Error code
     */
    constructor(message, status, code) {
        super(message, status, code);
        this.name = 'WalletError';
    }
}
exports.WalletError = WalletError;
/**
 * Multisig error
 */
class MultisigError extends ShardXError {
    /**
     * Create a new multisig error
     * @param message Error message
     * @param status HTTP status code
     * @param code Error code
     */
    constructor(message, status, code) {
        super(message, status, code);
        this.name = 'MultisigError';
    }
}
exports.MultisigError = MultisigError;
/**
 * Network error
 */
class NetworkError extends ShardXError {
    /**
     * Create a new network error
     * @param message Error message
     */
    constructor(message) {
        super(message, 0, 'network_error');
        this.name = 'NetworkError';
    }
}
exports.NetworkError = NetworkError;
/**
 * Validation error
 */
class ValidationError extends ShardXError {
    /**
     * Create a new validation error
     * @param message Error message
     * @param details Validation details
     */
    constructor(message, details) {
        super(message, 400, 'validation_error');
        this.name = 'ValidationError';
        this.details = details;
    }
}
exports.ValidationError = ValidationError;
//# sourceMappingURL=errors.js.map