"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.retry = exports.sleep = exports.timeAgo = exports.formatTimestamp = exports.truncateAddress = exports.ripemd160 = exports.sha256 = exports.utf8ToHex = exports.hexToUtf8 = exports.formatAmountWithSymbol = exports.formatAmount = void 0;
const buffer_1 = require("buffer");
const crypto_js_1 = __importDefault(require("crypto-js"));
const bignumber_js_1 = __importDefault(require("bignumber.js"));
/**
 * Format amount with specified decimal places
 * @param amount Amount to format
 * @param decimals Number of decimal places
 * @returns Formatted amount
 */
function formatAmount(amount, decimals = 8) {
    const bn = new bignumber_js_1.default(amount);
    return bn.toFixed(decimals);
}
exports.formatAmount = formatAmount;
/**
 * Format amount with token symbol
 * @param amount Amount to format
 * @param symbol Token symbol
 * @param decimals Number of decimal places
 * @returns Formatted amount with symbol
 */
function formatAmountWithSymbol(amount, symbol, decimals = 8) {
    return `${formatAmount(amount, decimals)} ${symbol}`;
}
exports.formatAmountWithSymbol = formatAmountWithSymbol;
/**
 * Convert hex string to UTF-8 string
 * @param hex Hex string
 * @returns UTF-8 string
 */
function hexToUtf8(hex) {
    return buffer_1.Buffer.from(hex, 'hex').toString('utf8');
}
exports.hexToUtf8 = hexToUtf8;
/**
 * Convert UTF-8 string to hex string
 * @param str UTF-8 string
 * @returns Hex string
 */
function utf8ToHex(str) {
    return buffer_1.Buffer.from(str, 'utf8').toString('hex');
}
exports.utf8ToHex = utf8ToHex;
/**
 * Calculate SHA-256 hash
 * @param data Data to hash
 * @returns Hex encoded hash
 */
function sha256(data) {
    return crypto_js_1.default.SHA256(data).toString();
}
exports.sha256 = sha256;
/**
 * Calculate RIPEMD-160 hash
 * @param data Data to hash
 * @returns Hex encoded hash
 */
function ripemd160(data) {
    return crypto_js_1.default.RIPEMD160(data).toString();
}
exports.ripemd160 = ripemd160;
/**
 * Truncate address for display
 * @param address Address to truncate
 * @param startChars Number of characters to show at the start
 * @param endChars Number of characters to show at the end
 * @returns Truncated address
 */
function truncateAddress(address, startChars = 6, endChars = 4) {
    if (!address) {
        return '';
    }
    if (address.length <= startChars + endChars) {
        return address;
    }
    return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
}
exports.truncateAddress = truncateAddress;
/**
 * Format timestamp as date string
 * @param timestamp Timestamp in milliseconds
 * @param format Date format (default: 'YYYY-MM-DD HH:mm:ss')
 * @returns Formatted date string
 */
function formatTimestamp(timestamp, format) {
    const date = new Date(timestamp);
    // Simple formatting (in a real implementation, use a date library like date-fns)
    return date.toLocaleString();
}
exports.formatTimestamp = formatTimestamp;
/**
 * Calculate time difference from now
 * @param timestamp Timestamp in milliseconds
 * @returns Human-readable time difference
 */
function timeAgo(timestamp) {
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
exports.timeAgo = timeAgo;
/**
 * Sleep for specified duration
 * @param ms Duration in milliseconds
 * @returns Promise that resolves after the specified duration
 */
function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}
exports.sleep = sleep;
/**
 * Retry a function with exponential backoff
 * @param fn Function to retry
 * @param maxRetries Maximum number of retries
 * @param initialDelay Initial delay in milliseconds
 * @returns Promise that resolves with the function result
 */
async function retry(fn, maxRetries = 3, initialDelay = 1000) {
    let lastError;
    for (let i = 0; i < maxRetries; i++) {
        try {
            return await fn();
        }
        catch (error) {
            lastError = error;
            // Calculate delay with exponential backoff
            const delay = initialDelay * Math.pow(2, i);
            // Wait before retrying
            await sleep(delay);
        }
    }
    throw lastError;
}
exports.retry = retry;
//# sourceMappingURL=utils.js.map