/**
 * ShardX API Error
 */
export class ShardXError extends Error {
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
  constructor(message: string, status: number, code?: string) {
    super(message);
    this.name = 'ShardXError';
    this.status = status;
    this.code = code;
  }
}

/**
 * Transaction error
 */
export class TransactionError extends ShardXError {
  /**
   * Create a new transaction error
   * @param message Error message
   * @param status HTTP status code
   * @param code Error code
   */
  constructor(message: string, status: number, code?: string) {
    super(message, status, code);
    this.name = 'TransactionError';
  }
}

/**
 * Wallet error
 */
export class WalletError extends ShardXError {
  /**
   * Create a new wallet error
   * @param message Error message
   * @param status HTTP status code
   * @param code Error code
   */
  constructor(message: string, status: number, code?: string) {
    super(message, status, code);
    this.name = 'WalletError';
  }
}

/**
 * Multisig error
 */
export class MultisigError extends ShardXError {
  /**
   * Create a new multisig error
   * @param message Error message
   * @param status HTTP status code
   * @param code Error code
   */
  constructor(message: string, status: number, code?: string) {
    super(message, status, code);
    this.name = 'MultisigError';
  }
}

/**
 * Network error
 */
export class NetworkError extends ShardXError {
  /**
   * Create a new network error
   * @param message Error message
   */
  constructor(message: string) {
    super(message, 0, 'network_error');
    this.name = 'NetworkError';
  }
}

/**
 * Validation error
 */
export class ValidationError extends ShardXError {
  /** Validation details */
  details?: Record<string, string>;

  /**
   * Create a new validation error
   * @param message Error message
   * @param details Validation details
   */
  constructor(message: string, details?: Record<string, string>) {
    super(message, 400, 'validation_error');
    this.name = 'ValidationError';
    this.details = details;
  }
}