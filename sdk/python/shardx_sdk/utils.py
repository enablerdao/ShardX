import time
import hashlib
import binascii
from typing import TypeVar, Callable, Any, Optional
from decimal import Decimal

T = TypeVar('T')

def format_amount(amount: str, decimals: int = 8) -> str:
    """
    Format amount with specified decimal places
    
    Args:
        amount: Amount to format
        decimals: Number of decimal places
        
    Returns:
        Formatted amount
    """
    try:
        decimal_amount = Decimal(amount)
        return f"{decimal_amount:.{decimals}f}"
    except:
        return amount

def format_amount_with_symbol(amount: str, symbol: str, decimals: int = 8) -> str:
    """
    Format amount with token symbol
    
    Args:
        amount: Amount to format
        symbol: Token symbol
        decimals: Number of decimal places
        
    Returns:
        Formatted amount with symbol
    """
    return f"{format_amount(amount, decimals)} {symbol}"

def hex_to_utf8(hex_str: str) -> str:
    """
    Convert hex string to UTF-8 string
    
    Args:
        hex_str: Hex string
        
    Returns:
        UTF-8 string
    """
    try:
        return binascii.unhexlify(hex_str).decode('utf-8')
    except:
        return hex_str

def utf8_to_hex(utf8_str: str) -> str:
    """
    Convert UTF-8 string to hex string
    
    Args:
        utf8_str: UTF-8 string
        
    Returns:
        Hex string
    """
    return binascii.hexlify(utf8_str.encode('utf-8')).decode('ascii')

def sha256(data: str) -> str:
    """
    Calculate SHA-256 hash
    
    Args:
        data: Data to hash
        
    Returns:
        Hex encoded hash
    """
    return hashlib.sha256(data.encode()).hexdigest()

def ripemd160(data: str) -> str:
    """
    Calculate RIPEMD-160 hash
    
    Args:
        data: Data to hash
        
    Returns:
        Hex encoded hash
    """
    h = hashlib.new('ripemd160')
    h.update(data.encode())
    return h.hexdigest()

def truncate_address(address: str, start_chars: int = 6, end_chars: int = 4) -> str:
    """
    Truncate address for display
    
    Args:
        address: Address to truncate
        start_chars: Number of characters to show at the start
        end_chars: Number of characters to show at the end
        
    Returns:
        Truncated address
    """
    if not address:
        return ''
    
    if len(address) <= start_chars + end_chars:
        return address
    
    return f"{address[:start_chars]}...{address[-end_chars:]}"

def format_timestamp(timestamp: int, format_str: Optional[str] = None) -> str:
    """
    Format timestamp as date string
    
    Args:
        timestamp: Timestamp in milliseconds
        format_str: Date format (not used in this implementation)
        
    Returns:
        Formatted date string
    """
    from datetime import datetime
    
    # Convert to seconds if in milliseconds
    if timestamp > 1000000000000:
        timestamp = timestamp / 1000
    
    date = datetime.fromtimestamp(timestamp)
    
    # Simple formatting
    return date.strftime("%Y-%m-%d %H:%M:%S")

def time_ago(timestamp: int) -> str:
    """
    Calculate time difference from now
    
    Args:
        timestamp: Timestamp in milliseconds
        
    Returns:
        Human-readable time difference
    """
    # Convert to seconds if in milliseconds
    if timestamp > 1000000000000:
        timestamp = timestamp / 1000
    
    seconds = int(time.time() - timestamp)
    
    if seconds < 0:
        return "in the future"
    
    intervals = [
        (31536000, "year"),
        (2592000, "month"),
        (86400, "day"),
        (3600, "hour"),
        (60, "minute"),
        (1, "second")
    ]
    
    for seconds_in_interval, interval_name in intervals:
        interval = seconds // seconds_in_interval
        if interval > 1:
            return f"{interval} {interval_name}s ago"
        if interval == 1:
            return f"1 {interval_name} ago"
    
    return "just now"

def sleep(seconds: float) -> None:
    """
    Sleep for specified duration
    
    Args:
        seconds: Duration in seconds
    """
    time.sleep(seconds)

async def retry(
    fn: Callable[..., T],
    max_retries: int = 3,
    initial_delay: float = 1.0,
    *args: Any,
    **kwargs: Any
) -> T:
    """
    Retry a function with exponential backoff
    
    Args:
        fn: Function to retry
        max_retries: Maximum number of retries
        initial_delay: Initial delay in seconds
        *args: Arguments to pass to the function
        **kwargs: Keyword arguments to pass to the function
        
    Returns:
        Function result
        
    Raises:
        Exception: The last exception raised by the function
    """
    last_exception = None
    
    for i in range(max_retries):
        try:
            return await fn(*args, **kwargs)
        except Exception as e:
            last_exception = e
            
            # Calculate delay with exponential backoff
            delay = initial_delay * (2 ** i)
            
            # Wait before retrying
            sleep(delay)
    
    raise last_exception