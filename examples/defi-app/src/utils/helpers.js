/**
 * Truncate address for display
 * @param {string} address - Address to truncate
 * @param {number} startChars - Number of characters to show at the start
 * @param {number} endChars - Number of characters to show at the end
 * @returns {string} Truncated address
 */
export const truncateAddress = (address, startChars = 6, endChars = 4) => {
  if (!address) {
    return '';
  }
  
  if (address.length <= startChars + endChars) {
    return address;
  }
  
  return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
};

/**
 * Format amount with specified decimal places
 * @param {string|number} amount - Amount to format
 * @param {number} decimals - Number of decimal places
 * @returns {string} Formatted amount
 */
export const formatAmount = (amount, decimals = 8) => {
  if (!amount) {
    return '0';
  }
  
  const num = parseFloat(amount);
  return num.toFixed(decimals);
};

/**
 * Format amount with token symbol
 * @param {string|number} amount - Amount to format
 * @param {string} symbol - Token symbol
 * @param {number} decimals - Number of decimal places
 * @returns {string} Formatted amount with symbol
 */
export const formatAmountWithSymbol = (amount, symbol, decimals = 8) => {
  return `${formatAmount(amount, decimals)} ${symbol}`;
};

/**
 * Format timestamp as date string
 * @param {number} timestamp - Timestamp in milliseconds
 * @returns {string} Formatted date string
 */
export const formatTimestamp = (timestamp) => {
  if (!timestamp) {
    return '';
  }
  
  const date = new Date(timestamp);
  return date.toLocaleString();
};

/**
 * Calculate time difference from now
 * @param {number} timestamp - Timestamp in milliseconds
 * @returns {string} Human-readable time difference
 */
export const timeAgo = (timestamp) => {
  if (!timestamp) {
    return '';
  }
  
  const seconds = Math.floor((Date.now() - timestamp) / 1000);
  
  if (seconds < 0) {
    return 'in the future';
  }
  
  const intervals = [
    { seconds: 31536000, label: 'year' },
    { seconds: 2592000, label: 'month' },
    { seconds: 86400, label: 'day' },
    { seconds: 3600, label: 'hour' },
    { seconds: 60, label: 'minute' },
    { seconds: 1, label: 'second' }
  ];
  
  for (const interval of intervals) {
    const count = Math.floor(seconds / interval.seconds);
    if (count > 0) {
      return count === 1
        ? `1 ${interval.label} ago`
        : `${count} ${interval.label}s ago`;
    }
  }
  
  return 'just now';
};

/**
 * Format percentage
 * @param {string|number} value - Value to format
 * @param {number} decimals - Number of decimal places
 * @returns {string} Formatted percentage
 */
export const formatPercentage = (value, decimals = 2) => {
  if (!value) {
    return '0%';
  }
  
  const num = parseFloat(value);
  return `${num.toFixed(decimals)}%`;
};

/**
 * Generate random color
 * @returns {string} Random color in hex format
 */
export const randomColor = () => {
  return `#${Math.floor(Math.random() * 16777215).toString(16)}`;
};

/**
 * Sleep for specified duration
 * @param {number} ms - Duration in milliseconds
 * @returns {Promise} Promise that resolves after the specified duration
 */
export const sleep = (ms) => {
  return new Promise(resolve => setTimeout(resolve, ms));
};

/**
 * Retry a function with exponential backoff
 * @param {Function} fn - Function to retry
 * @param {number} maxRetries - Maximum number of retries
 * @param {number} initialDelay - Initial delay in milliseconds
 * @returns {Promise} Promise that resolves with the function result
 */
export const retry = async (fn, maxRetries = 3, initialDelay = 1000) => {
  let lastError;
  
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;
      
      // Calculate delay with exponential backoff
      const delay = initialDelay * Math.pow(2, i);
      
      // Wait before retrying
      await sleep(delay);
    }
  }
  
  throw lastError;
};