from typing import List, Optional, Dict, Any, Literal
from dataclasses import dataclass

# Transaction status type
TransactionStatus = Literal["pending", "confirmed", "failed"]

@dataclass
class NodeInfo:
    """Node information"""
    id: str
    version: str
    uptime: int
    peers: int
    shards: int
    height: int
    synced: bool

@dataclass
class NetworkStats:
    """Network statistics"""
    total_transactions: int
    tps: float
    avg_block_time: float
    total_accounts: int
    total_validators: int
    total_staked: str
    current_fee: str

@dataclass
class ShardInfo:
    """Shard information"""
    id: str
    name: str
    validators: int
    height: int
    tps: float
    status: Literal["active", "inactive", "syncing"]

@dataclass
class Transaction:
    """Transaction"""
    id: str
    status: TransactionStatus
    timestamp: int
    from_address: str
    to: str
    amount: str
    fee: str
    data: Optional[str] = None
    block_hash: Optional[str] = None
    block_height: Optional[int] = None
    shard_id: str = ""
    parent_ids: Optional[List[str]] = None

@dataclass
class TransactionRequest:
    """Transaction request"""
    from_address: str
    to: str
    amount: str
    signature: str
    data: Optional[str] = None

@dataclass
class Block:
    """Block"""
    hash: str
    height: int
    previous_hash: str
    timestamp: int
    validator: str
    transaction_count: int
    shard_id: str
    size: int
    transactions: Optional[List[Transaction]] = None

@dataclass
class Account:
    """Account"""
    address: str
    balance: str
    nonce: int
    type: Literal["standard", "contract", "multisig"]
    created_at: int
    last_activity: int
    staked: Optional[str] = None
    delegated: Optional[str] = None

@dataclass
class MultisigWallet:
    """Multisig wallet"""
    id: str
    name: str
    owner_id: str
    signers: List[str]
    required_signatures: int
    balance: str
    created_at: int

@dataclass
class MultisigWalletRequest:
    """Multisig wallet request"""
    name: str
    owner_id: str
    signers: List[str]
    required_signatures: int

@dataclass
class TradingPair:
    """Trading pair"""
    base: str
    quote: str
    display_name: str
    min_order_size: str
    price_precision: int

@dataclass
class HistoricalDataPoint:
    """Historical data point"""
    timestamp: int
    price: str

@dataclass
class Prediction:
    """AI prediction"""
    pair: TradingPair
    period: str
    current_price: str
    predicted_price: str
    confidence: float
    timestamp: int
    historical_data: Optional[List[HistoricalDataPoint]] = None

# Helper function to convert snake_case to camelCase for API requests
def to_camel_case(snake_str: str) -> str:
    components = snake_str.split('_')
    return components[0] + ''.join(x.title() for x in components[1:])

# Helper function to convert camelCase to snake_case for API responses
def to_snake_case(camel_str: str) -> str:
    result = [camel_str[0].lower()]
    for char in camel_str[1:]:
        if char.isupper():
            result.append('_')
            result.append(char.lower())
        else:
            result.append(char)
    return ''.join(result)

# Helper function to convert dictionary keys from camelCase to snake_case
def convert_keys_to_snake_case(data: Dict[str, Any]) -> Dict[str, Any]:
    if not isinstance(data, dict):
        return data
    
    result = {}
    for key, value in data.items():
        snake_key = to_snake_case(key)
        
        if isinstance(value, dict):
            result[snake_key] = convert_keys_to_snake_case(value)
        elif isinstance(value, list):
            result[snake_key] = [
                convert_keys_to_snake_case(item) if isinstance(item, dict) else item
                for item in value
            ]
        else:
            result[snake_key] = value
    
    return result

# Helper function to convert dictionary keys from snake_case to camelCase
def convert_keys_to_camel_case(data: Dict[str, Any]) -> Dict[str, Any]:
    if not isinstance(data, dict):
        return data
    
    result = {}
    for key, value in data.items():
        camel_key = to_camel_case(key)
        
        if isinstance(value, dict):
            result[camel_key] = convert_keys_to_camel_case(value)
        elif isinstance(value, list):
            result[camel_key] = [
                convert_keys_to_camel_case(item) if isinstance(item, dict) else item
                for item in value
            ]
        else:
            result[camel_key] = value
    
    return result