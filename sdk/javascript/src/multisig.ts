import { ShardXClient } from './client';
import { Wallet } from './wallet';
import { MultisigWallet, MultisigWalletRequest, Transaction } from './models';
import { MultisigError, ValidationError } from './errors';

/**
 * Multisig transaction
 */
export interface MultisigTransaction extends Transaction {
  /** Signatures collected so far */
  signatures: {
    /** Signer address */
    signer: string;
    /** Signature */
    signature: string;
  }[];
  /** Required signatures */
  requiredSignatures: number;
}

/**
 * Multisig wallet manager
 * 
 * Utility class for working with multisig wallets
 */
export class MultisigManager {
  private readonly client: ShardXClient;

  /**
   * Create a new multisig manager
   * @param client ShardX client
   */
  constructor(client: ShardXClient) {
    this.client = client;
  }

  /**
   * Create a new multisig wallet
   * @param wallet Owner wallet
   * @param name Wallet name
   * @param signers Signer addresses
   * @param requiredSignatures Required signatures
   * @returns Created multisig wallet
   */
  async createWallet(
    wallet: Wallet,
    name: string,
    signers: string[],
    requiredSignatures: number
  ): Promise<MultisigWallet> {
    // Validate inputs
    if (!name) {
      throw new ValidationError('Wallet name is required');
    }
    
    if (!signers || signers.length === 0) {
      throw new ValidationError('At least one signer is required');
    }
    
    if (requiredSignatures <= 0 || requiredSignatures > signers.length) {
      throw new ValidationError(`Required signatures must be between 1 and ${signers.length}`);
    }
    
    // Create wallet request
    const walletRequest: MultisigWalletRequest = {
      name,
      ownerId: wallet.getAddress(),
      signers,
      requiredSignatures
    };
    
    // Create multisig wallet
    return this.client.createMultisigWallet(walletRequest);
  }

  /**
   * Get multisig wallet by ID
   * @param walletId Multisig wallet ID
   * @returns Multisig wallet
   */
  async getWallet(walletId: string): Promise<MultisigWallet> {
    return this.client.getMultisigWallet(walletId);
  }

  /**
   * Get multisig wallets by owner
   * @param ownerAddress Owner address
   * @returns Array of multisig wallets
   */
  async getWalletsByOwner(ownerAddress: string): Promise<MultisigWallet[]> {
    return this.client.getMultisigWalletsByOwner(ownerAddress);
  }

  /**
   * Create a multisig transaction
   * @param wallet Signer wallet
   * @param multisigId Multisig wallet ID
   * @param to Recipient address
   * @param amount Amount to send
   * @param data Optional transaction data
   * @returns Created multisig transaction
   */
  async createTransaction(
    wallet: Wallet,
    multisigId: string,
    to: string,
    amount: string,
    data?: string
  ): Promise<MultisigTransaction> {
    // Get multisig wallet
    const multisigWallet = await this.client.getMultisigWallet(multisigId);
    
    // Check if wallet is a signer
    const signerAddress = wallet.getAddress();
    if (!multisigWallet.signers.includes(signerAddress)) {
      throw new MultisigError(
        `Address ${signerAddress} is not a signer for this multisig wallet`,
        403,
        'not_a_signer'
      );
    }
    
    // Create transaction data
    const txData = {
      multisigId,
      to,
      amount,
      data,
      initiator: signerAddress
    };
    
    // Encode transaction data
    const encodedData = Buffer.from(JSON.stringify(txData)).toString('hex');
    
    // Create transaction
    const tx = await wallet.createTransaction(multisigWallet.id, '0', encodedData);
    
    // Convert to multisig transaction
    const multisigTx: MultisigTransaction = {
      ...tx,
      signatures: [{
        signer: signerAddress,
        signature: tx.data || ''
      }],
      requiredSignatures: multisigWallet.requiredSignatures
    };
    
    return multisigTx;
  }

  /**
   * Sign a multisig transaction
   * @param wallet Signer wallet
   * @param txId Transaction ID
   * @returns Updated multisig transaction
   */
  async signTransaction(wallet: Wallet, txId: string): Promise<MultisigTransaction> {
    // Get transaction
    const tx = await this.client.getTransaction(txId);
    
    // Decode transaction data
    const txData = JSON.parse(Buffer.from(tx.data || '', 'hex').toString());
    
    // Get multisig wallet
    const multisigWallet = await this.client.getMultisigWallet(txData.multisigId);
    
    // Check if wallet is a signer
    const signerAddress = wallet.getAddress();
    if (!multisigWallet.signers.includes(signerAddress)) {
      throw new MultisigError(
        `Address ${signerAddress} is not a signer for this multisig wallet`,
        403,
        'not_a_signer'
      );
    }
    
    // Create signature
    const signature = wallet.sign(txId);
    
    // Create signature transaction
    const signatureTx = await wallet.createTransaction(
      tx.id,
      '0',
      Buffer.from(JSON.stringify({
        signer: signerAddress,
        signature
      })).toString('hex')
    );
    
    // Get updated multisig transaction
    const updatedTx = await this.client.getTransaction(txId);
    
    // Convert to multisig transaction
    // In a real implementation, this would get the signatures from the API
    const multisigTx: MultisigTransaction = {
      ...updatedTx,
      signatures: [
        {
          signer: signerAddress,
          signature
        }
      ],
      requiredSignatures: multisigWallet.requiredSignatures
    };
    
    return multisigTx;
  }

  /**
   * Execute a multisig transaction
   * @param wallet Signer wallet
   * @param txId Transaction ID
   * @returns Executed transaction
   */
  async executeTransaction(wallet: Wallet, txId: string): Promise<Transaction> {
    // Get transaction
    const tx = await this.client.getTransaction(txId);
    
    // Decode transaction data
    const txData = JSON.parse(Buffer.from(tx.data || '', 'hex').toString());
    
    // Get multisig wallet
    const multisigWallet = await this.client.getMultisigWallet(txData.multisigId);
    
    // Check if wallet is a signer
    const signerAddress = wallet.getAddress();
    if (!multisigWallet.signers.includes(signerAddress)) {
      throw new MultisigError(
        `Address ${signerAddress} is not a signer for this multisig wallet`,
        403,
        'not_a_signer'
      );
    }
    
    // Create execution transaction
    const executionTx = await wallet.createTransaction(
      tx.id,
      '0',
      Buffer.from(JSON.stringify({
        execute: true
      })).toString('hex')
    );
    
    return executionTx;
  }
}