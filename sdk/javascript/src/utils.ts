import { Buffer } from 'buffer';
import CryptoJS from 'crypto-js';
import BigNumber from 'bignumber.js';

/**
 * Format amount with specified decimal places
 * @param amount Amount to format
 * @param decimals Number of decimal places
 * @returns Formatted amount
 */
export function formatAmount(amount: string | number, decimals: number = 8): string {
  const bn = new BigNumber(amount);
  return bn.toFixed(decimals);
}

/**
 * Format amount with token symbol
 * @param amount Amount to format
 * @param symbol Token symbol
 * @param decimals Number of decimal places
 * @returns Formatted amount with symbol
 */
export function formatAmountWithSymbol(amount: string | number, symbol: string, decimals: number = 8): string {
  return `${formatAmount(amount, decimals)} ${symbol}`;
}

/**
 * Convert hex string to UTF-8 string
 * @param hex Hex string
 * @returns UTF-8 string
 */
export function hexToUtf8(hex: string): string {
  return Buffer.from(hex, 'hex').toString('utf8');
}

/**
 * Convert UTF-8 string to hex string
 * @param str UTF-8 string
 * @returns Hex string
 */
export function utf8ToHex(str: string): string {
  return Buffer.from(str, 'utf8').toString('hex');
}

/**
 * Calculate SHA-256 hash
 * @param data Data to hash
 * @returns Hex encoded hash
 */
export function sha256(data: string): string {
  return CryptoJS.SHA256(data).toString();
}

/**
 * Calculate RIPEMD-160 hash
 * @param data Data to hash
 * @returns Hex encoded hash
 */
export function ripemd160(data: string): string {
  return CryptoJS.RIPEMD160(data).toString();
}

/**
 * Truncate address for display
 * @param address Address to truncate
 * @param startChars Number of characters to show at the start
 * @param endChars Number of characters to show at the end
 * @returns Truncated address
 */
export function truncateAddress(address: string, startChars: number = 6, endChars: number = 4): string {
  if (!address) {
    return '';
  }
  
  if (address.length <= startChars + endChars) {
    return address;
  }
  
  return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
}

/**
 * Format timestamp as date string
 * @param timestamp Timestamp in milliseconds
 * @param format Date format (default: 'YYYY-MM-DD HH:mm:ss')
 * @returns Formatted date string
 */
export function formatTimestamp(timestamp: number, format?: string): string {
  const date = new Date(timestamp);
  
  // Simple formatting (in a real implementation, use a date library like date-fns)
  return date.toLocaleString();
}

/**
 * Calculate time difference from now
 * @param timestamp Timestamp in milliseconds
 * @returns Human-readable time difference
 */
export function timeAgo(timestamp: number): string {
  const seconds = Math.floor((Date.now() - timestamp) / 1000);
  
  let interval = Math.floor(seconds / 31536000);
  if (interval > 1) {
    return `${interval} years ago`;
  }
  if (interval === 1) {
    return '1 year ago';
  }
  
  interval = Math.floor(seconds / 2592000);
  if (interval > 1) {
    return `${interval} months ago`;
  }
  if (interval === 1) {
    return '1 month ago';
  }
  
  interval = Math.floor(seconds / 86400);
  if (interval > 1) {
    return `${interval} days ago`;
  }
  if (interval === 1) {
    return '1 day ago';
  }
  
  interval = Math.floor(seconds / 3600);
  if (interval > 1) {
    return `${interval} hours ago`;
  }
  if (interval === 1) {
    return '1 hour ago';
  }
  
  interval = Math.floor(seconds / 60);
  if (interval > 1) {
    return `${interval} minutes ago`;
  }
  if (interval === 1) {
    return '1 minute ago';
  }
  
  if (seconds < 10) {
    return 'just now';
  }
  
  return `${Math.floor(seconds)} seconds ago`;
}

/**
 * Sleep for specified duration
 * @param ms Duration in milliseconds
 * @returns Promise that resolves after the specified duration
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Retry a function with exponential backoff
 * @param fn Function to retry
 * @param maxRetries Maximum number of retries
 * @param initialDelay Initial delay in milliseconds
 * @returns Promise that resolves with the function result
 */
export async function retry<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
  initialDelay: number = 1000
): Promise<T> {
  let lastError: Error | undefined;
  
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error as Error;
      
      // Calculate delay with exponential backoff
      const delay = initialDelay * Math.pow(2, i);
      
      // Wait before retrying
      await sleep(delay);
    }
  }
  
  throw lastError;
}