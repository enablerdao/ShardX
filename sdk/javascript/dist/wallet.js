"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Wallet = void 0;
const crypto_js_1 = __importDefault(require("crypto-js"));
const buffer_1 = require("buffer");
const bs58_1 = __importDefault(require("bs58"));
const errors_1 = require("./errors");
/**
 * ShardX wallet
 *
 * Manages keys and signing transactions
 */
class Wallet {
    /**
     * Create a new wallet
     * @param options Wallet options
     */
    constructor(options) {
        this.client = options.client;
        if (options.privateKey) {
            this.privateKey = options.privateKey;
            this.publicKey = this.derivePublicKey(this.privateKey);
            this.address = this.deriveAddress(this.publicKey);
        }
    }
    /**
     * Create a new random wallet
     * @param client ShardX client
     * @returns New wallet
     */
    static createRandom(client) {
        // Generate random private key
        const privateKey = crypto_js_1.default.lib.WordArray.random(32).toString();
        return new Wallet({
            client,
            privateKey,
        });
    }
    /**
     * Create a wallet from mnemonic phrase
     * @param mnemonic Mnemonic phrase
     * @param client ShardX client
     * @returns Wallet
     */
    static fromMnemonic(mnemonic, client) {
        // In a real implementation, this would use BIP39/BIP44
        // For simplicity, we'll just hash the mnemonic to get a private key
        const privateKey = crypto_js_1.default.SHA256(mnemonic).toString();
        return new Wallet({
            client,
            privateKey,
        });
    }
    /**
     * Get wallet address
     * @returns Wallet address
     */
    getAddress() {
        if (!this.address) {
            throw new errors_1.WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
        }
        return this.address;
    }
    /**
     * Get wallet public key
     * @returns Public key (hex encoded)
     */
    getPublicKey() {
        if (!this.publicKey) {
            throw new errors_1.WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
        }
        return this.publicKey;
    }
    /**
     * Sign a message
     * @param message Message to sign
     * @returns Signature (hex encoded)
     */
    sign(message) {
        if (!this.privateKey) {
            throw new errors_1.WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
        }
        // In a real implementation, this would use proper cryptographic signing
        // For simplicity, we'll just use HMAC
        const signature = crypto_js_1.default.HmacSHA256(message, this.privateKey).toString();
        return signature;
    }
    /**
     * Create and sign a transaction
     * @param to Recipient address
     * @param amount Amount to send
     * @param data Optional transaction data
     * @returns Signed transaction
     */
    async createTransaction(to, amount, data) {
        if (!this.privateKey || !this.address) {
            throw new errors_1.WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
        }
        // Validate inputs
        if (!to) {
            throw new errors_1.ValidationError('Recipient address is required');
        }
        if (!amount || parseFloat(amount) <= 0) {
            throw new errors_1.ValidationError('Amount must be greater than 0');
        }
        // Create transaction request
        const txRequest = {
            from: this.address,
            to,
            amount,
            data,
            signature: '', // Will be filled below
        };
        // Create message to sign
        const message = `${txRequest.from}:${txRequest.to}:${txRequest.amount}:${txRequest.data || ''}`;
        // Sign the message
        txRequest.signature = this.sign(message);
        // Send transaction to the network
        return this.client.createTransaction(txRequest);
    }
    /**
     * Get account balance
     * @returns Account balance
     */
    async getBalance() {
        if (!this.address) {
            throw new errors_1.WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
        }
        const account = await this.client.getAccount(this.address);
        return account.balance;
    }
    /**
     * Get transaction history
     * @param limit Maximum number of transactions to return
     * @param offset Offset for pagination
     * @returns Array of transactions
     */
    async getTransactions(limit = 20, offset = 0) {
        if (!this.address) {
            throw new errors_1.WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
        }
        return this.client.getTransactionsByAddress(this.address, limit, offset);
    }
    /**
     * Derive public key from private key
     * @param privateKey Private key (hex encoded)
     * @returns Public key (hex encoded)
     */
    derivePublicKey(privateKey) {
        // In a real implementation, this would use proper cryptographic key derivation
        // For simplicity, we'll just hash the private key
        return crypto_js_1.default.SHA256(privateKey).toString();
    }
    /**
     * Derive address from public key
     * @param publicKey Public key (hex encoded)
     * @returns Address
     */
    deriveAddress(publicKey) {
        // In a real implementation, this would use proper address derivation
        // For simplicity, we'll hash the public key and encode in base58
        const hash = crypto_js_1.default.RIPEMD160(crypto_js_1.default.SHA256(publicKey)).toString();
        const hashWithPrefix = '00' + hash; // Add version prefix
        // Convert to Buffer and encode in base58
        const buffer = buffer_1.Buffer.from(hashWithPrefix, 'hex');
        return bs58_1.default.encode(buffer);
    }
}
exports.Wallet = Wallet;
//# sourceMappingURL=wallet.js.map