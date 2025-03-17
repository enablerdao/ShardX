/**
 * Format amount with specified decimal places
 * @param amount Amount to format
 * @param decimals Number of decimal places
 * @returns Formatted amount
 */
export declare function formatAmount(amount: string | number, decimals?: number): string;
/**
 * Format amount with token symbol
 * @param amount Amount to format
 * @param symbol Token symbol
 * @param decimals Number of decimal places
 * @returns Formatted amount with symbol
 */
export declare function formatAmountWithSymbol(amount: string | number, symbol: string, decimals?: number): string;
/**
 * Convert hex string to UTF-8 string
 * @param hex Hex string
 * @returns UTF-8 string
 */
export declare function hexToUtf8(hex: string): string;
/**
 * Convert UTF-8 string to hex string
 * @param str UTF-8 string
 * @returns Hex string
 */
export declare function utf8ToHex(str: string): string;
/**
 * Calculate SHA-256 hash
 * @param data Data to hash
 * @returns Hex encoded hash
 */
export declare function sha256(data: string): string;
/**
 * Calculate RIPEMD-160 hash
 * @param data Data to hash
 * @returns Hex encoded hash
 */
export declare function ripemd160(data: string): string;
/**
 * Truncate address for display
 * @param address Address to truncate
 * @param startChars Number of characters to show at the start
 * @param endChars Number of characters to show at the end
 * @returns Truncated address
 */
export declare function truncateAddress(address: string, startChars?: number, endChars?: number): string;
/**
 * Format timestamp as date string
 * @param timestamp Timestamp in milliseconds
 * @param format Date format (default: 'YYYY-MM-DD HH:mm:ss')
 * @returns Formatted date string
 */
export declare function formatTimestamp(timestamp: number, format?: string): string;
/**
 * Calculate time difference from now
 * @param timestamp Timestamp in milliseconds
 * @returns Human-readable time difference
 */
export declare function timeAgo(timestamp: number): string;
/**
 * Sleep for specified duration
 * @param ms Duration in milliseconds
 * @returns Promise that resolves after the specified duration
 */
export declare function sleep(ms: number): Promise<void>;
/**
 * Retry a function with exponential backoff
 * @param fn Function to retry
 * @param maxRetries Maximum number of retries
 * @param initialDelay Initial delay in milliseconds
 * @returns Promise that resolves with the function result
 */
export declare function retry<T>(fn: () => Promise<T>, maxRetries?: number, initialDelay?: number): Promise<T>;
