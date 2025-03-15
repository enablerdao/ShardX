import CryptoJS from 'crypto-js';
import { Buffer } from 'buffer';
import bs58 from 'bs58';
import { ShardXClient } from './client';
import { Transaction, TransactionRequest } from './models';
import { WalletError, ValidationError } from './errors';

/**
 * Wallet options
 */
export interface WalletOptions {
  /** ShardX client */
  client: ShardXClient;
  /** Private key (hex encoded) */
  privateKey?: string;
}

/**
 * ShardX wallet
 * 
 * Manages keys and signing transactions
 */
export class Wallet {
  private readonly client: ShardXClient;
  private readonly privateKey?: string;
  private readonly publicKey?: string;
  private readonly address?: string;

  /**
   * Create a new wallet
   * @param options Wallet options
   */
  constructor(options: WalletOptions) {
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
  static createRandom(client: ShardXClient): Wallet {
    // Generate random private key
    const privateKey = CryptoJS.lib.WordArray.random(32).toString();
    
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
  static fromMnemonic(mnemonic: string, client: ShardXClient): Wallet {
    // In a real implementation, this would use BIP39/BIP44
    // For simplicity, we'll just hash the mnemonic to get a private key
    const privateKey = CryptoJS.SHA256(mnemonic).toString();
    
    return new Wallet({
      client,
      privateKey,
    });
  }

  /**
   * Get wallet address
   * @returns Wallet address
   */
  getAddress(): string {
    if (!this.address) {
      throw new WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
    }
    
    return this.address;
  }

  /**
   * Get wallet public key
   * @returns Public key (hex encoded)
   */
  getPublicKey(): string {
    if (!this.publicKey) {
      throw new WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
    }
    
    return this.publicKey;
  }

  /**
   * Sign a message
   * @param message Message to sign
   * @returns Signature (hex encoded)
   */
  sign(message: string): string {
    if (!this.privateKey) {
      throw new WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
    }
    
    // In a real implementation, this would use proper cryptographic signing
    // For simplicity, we'll just use HMAC
    const signature = CryptoJS.HmacSHA256(message, this.privateKey).toString();
    
    return signature;
  }

  /**
   * Create and sign a transaction
   * @param to Recipient address
   * @param amount Amount to send
   * @param data Optional transaction data
   * @returns Signed transaction
   */
  async createTransaction(to: string, amount: string, data?: string): Promise<Transaction> {
    if (!this.privateKey || !this.address) {
      throw new WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
    }
    
    // Validate inputs
    if (!to) {
      throw new ValidationError('Recipient address is required');
    }
    
    if (!amount || parseFloat(amount) <= 0) {
      throw new ValidationError('Amount must be greater than 0');
    }
    
    // Create transaction request
    const txRequest: TransactionRequest = {
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
  async getBalance(): Promise<string> {
    if (!this.address) {
      throw new WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
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
  async getTransactions(limit = 20, offset = 0): Promise<Transaction[]> {
    if (!this.address) {
      throw new WalletError('Wallet not initialized with private key', 0, 'wallet_not_initialized');
    }
    
    return this.client.getTransactionsByAddress(this.address, limit, offset);
  }

  /**
   * Derive public key from private key
   * @param privateKey Private key (hex encoded)
   * @returns Public key (hex encoded)
   */
  private derivePublicKey(privateKey: string): string {
    // In a real implementation, this would use proper cryptographic key derivation
    // For simplicity, we'll just hash the private key
    return CryptoJS.SHA256(privateKey).toString();
  }

  /**
   * Derive address from public key
   * @param publicKey Public key (hex encoded)
   * @returns Address
   */
  private deriveAddress(publicKey: string): string {
    // In a real implementation, this would use proper address derivation
    // For simplicity, we'll hash the public key and encode in base58
    const hash = CryptoJS.RIPEMD160(CryptoJS.SHA256(publicKey)).toString();
    const hashWithPrefix = '00' + hash; // Add version prefix
    
    // Convert to Buffer and encode in base58
    const buffer = Buffer.from(hashWithPrefix, 'hex');
    return bs58.encode(buffer);
  }
}