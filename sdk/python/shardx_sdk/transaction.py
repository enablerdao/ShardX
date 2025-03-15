import json
import time
import binascii
from typing import Any, Dict, Optional, Union

from .client import ShardXClient
from .models import Transaction
from .errors import TransactionError

class TransactionManager:
    """
    Transaction manager
    
    Utility class for working with transactions
    """
    
    def __init__(self, client: ShardXClient):
        """
        Initialize a new transaction manager
        
        Args:
            client: ShardX client
        """
        self.client = client
    
    async def wait_for_confirmation(
        self,
        tx_id: str,
        timeout: int = 60,
        interval: int = 1
    ) -> Transaction:
        """
        Wait for transaction confirmation
        
        Args:
            tx_id: Transaction ID
            timeout: Timeout in seconds
            interval: Polling interval in seconds
            
        Returns:
            Confirmed transaction
            
        Raises:
            TransactionError: If transaction fails or times out
        """
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            status = self.client.get_transaction_status(tx_id)
            
            if status == "confirmed":
                return self.client.get_transaction(tx_id)
            
            if status == "failed":
                raise TransactionError(
                    f"Transaction {tx_id} failed",
                    400,
                    "transaction_failed"
                )
            
            # Wait for the next polling interval
            time.sleep(interval)
        
        raise TransactionError(
            f"Transaction {tx_id} confirmation timed out",
            408,
            "transaction_timeout"
        )
    
    async def get_transaction_with_analysis(self, tx_id: str) -> Dict[str, Any]:
        """
        Get transaction details with analysis
        
        Args:
            tx_id: Transaction ID
            
        Returns:
            Transaction with analysis
        """
        transaction = self.client.get_transaction(tx_id)
        analysis = self.client.get_transaction_analysis(tx_id)
        
        # Combine transaction and analysis
        result = transaction.__dict__.copy()
        result["analysis"] = analysis
        
        return result
    
    async def estimate_fee(
        self,
        from_addr: str,
        to_addr: str,
        amount: str,
        data: Optional[str] = None
    ) -> str:
        """
        Estimate transaction fee
        
        Args:
            from_addr: Sender address
            to_addr: Recipient address
            amount: Amount to send
            data: Optional transaction data
            
        Returns:
            Estimated fee
        """
        # In a real implementation, this would call a fee estimation API
        # For simplicity, we'll return a fixed fee
        return "0.001"
    
    def decode_transaction_data(self, data: Optional[str]) -> Any:
        """
        Decode transaction data
        
        Args:
            data: Hex encoded transaction data
            
        Returns:
            Decoded data
        """
        if not data:
            return None
        
        try:
            # Convert hex to string
            json_string = binascii.unhexlify(data).decode('utf-8')
            
            # Parse JSON
            return json.loads(json_string)
        except (binascii.Error, UnicodeDecodeError, json.JSONDecodeError):
            # If not valid JSON, return the raw string
            try:
                return binascii.unhexlify(data).decode('utf-8')
            except (binascii.Error, UnicodeDecodeError):
                return data
    
    def encode_transaction_data(self, data: Any) -> str:
        """
        Encode transaction data
        
        Args:
            data: Data to encode
            
        Returns:
            Hex encoded data
        """
        if data is None:
            return ""
        
        json_string: str
        
        if isinstance(data, str):
            json_string = data
        else:
            json_string = json.dumps(data)
        
        # Convert string to hex
        return binascii.hexlify(json_string.encode('utf-8')).decode('ascii')