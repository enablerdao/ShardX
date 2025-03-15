import requests
from typing import Dict, List, Optional, Union, Any
from .errors import ShardXError, NetworkError
from .models import (
    NodeInfo, NetworkStats, ShardInfo, Transaction, TransactionStatus,
    TransactionRequest, Block, Account, MultisigWallet, MultisigWalletRequest,
    TradingPair, Prediction
)

class ShardXClient:
    """
    ShardX API Client
    
    Main client for interacting with the ShardX blockchain platform.
    """
    
    def __init__(
        self,
        base_url: str = "http://localhost:54868/api/v1",
        api_key: Optional[str] = None,
        timeout: int = 30
    ):
        """
        Initialize a new ShardX client
        
        Args:
            base_url: Base URL for the ShardX API
            api_key: API key for authentication
            timeout: Request timeout in seconds
        """
        self.base_url = base_url
        self.api_key = api_key
        self.timeout = timeout
        self.session = requests.Session()
        
        # Set default headers
        self.session.headers.update({
            "Content-Type": "application/json",
            "Accept": "application/json"
        })
        
        # Set API key if provided
        if api_key:
            self.session.headers.update({"X-API-Key": api_key})
    
    def get_node_info(self) -> NodeInfo:
        """
        Get information about the node
        
        Returns:
            Node information
        """
        return self._request("GET", "/info")
    
    def get_network_stats(self) -> NetworkStats:
        """
        Get network statistics
        
        Returns:
            Network statistics
        """
        return self._request("GET", "/stats")
    
    def get_shards(self) -> List[ShardInfo]:
        """
        Get information about all shards
        
        Returns:
            List of shard information
        """
        return self._request("GET", "/shards")
    
    def get_shard(self, shard_id: str) -> ShardInfo:
        """
        Get information about a specific shard
        
        Args:
            shard_id: Shard ID
            
        Returns:
            Shard information
        """
        return self._request("GET", f"/shards/{shard_id}")
    
    def create_transaction(self, tx_data: TransactionRequest) -> Transaction:
        """
        Create a new transaction
        
        Args:
            tx_data: Transaction data
            
        Returns:
            Created transaction
        """
        return self._request("POST", "/transactions", json=tx_data)
    
    def get_transaction(self, tx_id: str) -> Transaction:
        """
        Get transaction by ID
        
        Args:
            tx_id: Transaction ID
            
        Returns:
            Transaction details
        """
        return self._request("GET", f"/transactions/{tx_id}")
    
    def get_transaction_status(self, tx_id: str) -> TransactionStatus:
        """
        Get transaction status
        
        Args:
            tx_id: Transaction ID
            
        Returns:
            Transaction status
        """
        response = self._request("GET", f"/transactions/{tx_id}/status")
        return response["status"]
    
    def get_transactions_by_address(
        self,
        address: str,
        limit: int = 20,
        offset: int = 0
    ) -> List[Transaction]:
        """
        Get transactions by address
        
        Args:
            address: Account address
            limit: Maximum number of transactions to return
            offset: Offset for pagination
            
        Returns:
            List of transactions
        """
        return self._request(
            "GET",
            f"/accounts/{address}/transactions",
            params={"limit": limit, "offset": offset}
        )
    
    def get_block(self, hash_or_height: Union[str, int]) -> Block:
        """
        Get block by hash or height
        
        Args:
            hash_or_height: Block hash or height
            
        Returns:
            Block details
        """
        return self._request("GET", f"/blocks/{hash_or_height}")
    
    def get_latest_blocks(self, limit: int = 10) -> List[Block]:
        """
        Get latest blocks
        
        Args:
            limit: Maximum number of blocks to return
            
        Returns:
            List of blocks
        """
        return self._request("GET", "/blocks", params={"limit": limit})
    
    def get_account(self, address: str) -> Account:
        """
        Get account information
        
        Args:
            address: Account address
            
        Returns:
            Account details
        """
        return self._request("GET", f"/accounts/{address}")
    
    def create_multisig_wallet(self, wallet_data: MultisigWalletRequest) -> MultisigWallet:
        """
        Create a new multisig wallet
        
        Args:
            wallet_data: Multisig wallet data
            
        Returns:
            Created multisig wallet
        """
        return self._request("POST", "/multisig/wallets", json=wallet_data)
    
    def get_multisig_wallet(self, wallet_id: str) -> MultisigWallet:
        """
        Get multisig wallet by ID
        
        Args:
            wallet_id: Multisig wallet ID
            
        Returns:
            Multisig wallet details
        """
        return self._request("GET", f"/multisig/wallets/{wallet_id}")
    
    def get_multisig_wallets_by_owner(self, owner_address: str) -> List[MultisigWallet]:
        """
        Get multisig wallets by owner
        
        Args:
            owner_address: Owner address
            
        Returns:
            List of multisig wallets
        """
        return self._request("GET", f"/accounts/{owner_address}/multisig")
    
    def get_prediction(self, pair: str, period: str = "hour") -> Prediction:
        """
        Get AI prediction for a trading pair
        
        Args:
            pair: Trading pair (e.g., "BTC/USD")
            period: Prediction period (e.g., "hour", "day", "week")
            
        Returns:
            Prediction details
        """
        return self._request("GET", f"/ai/predictions/{pair}", params={"period": period})
    
    def get_trading_pairs(self) -> List[TradingPair]:
        """
        Get available trading pairs
        
        Returns:
            List of trading pairs
        """
        return self._request("GET", "/ai/pairs")
    
    def get_transaction_analysis(self, tx_id: str) -> Dict[str, Any]:
        """
        Get detailed transaction analysis
        
        Args:
            tx_id: Transaction ID
            
        Returns:
            Transaction analysis details
        """
        return self._request("GET", f"/transactions/{tx_id}/analysis")
    
    def get_chart_data(
        self,
        metric: str,
        period: str,
        from_time: Optional[int] = None,
        to_time: Optional[int] = None
    ) -> Dict[str, Any]:
        """
        Get advanced charts data
        
        Args:
            metric: Metric to chart (e.g., "transactions", "volume", "fees")
            period: Period (e.g., "hour", "day", "week", "month")
            from_time: Start timestamp
            to_time: End timestamp
            
        Returns:
            Chart data
        """
        params = {"metric": metric, "period": period}
        if from_time:
            params["from"] = from_time
        if to_time:
            params["to"] = to_time
        
        return self._request("GET", "/charts", params=params)
    
    def _request(
        self,
        method: str,
        endpoint: str,
        params: Optional[Dict[str, Any]] = None,
        json: Optional[Dict[str, Any]] = None
    ) -> Any:
        """
        Make an API request
        
        Args:
            method: HTTP method
            endpoint: API endpoint
            params: Query parameters
            json: JSON body
            
        Returns:
            Response data
            
        Raises:
            ShardXError: If the API returns an error
            NetworkError: If there's a network error
        """
        url = f"{self.base_url}{endpoint}"
        
        try:
            response = self.session.request(
                method=method,
                url=url,
                params=params,
                json=json,
                timeout=self.timeout
            )
            
            # Raise exception for HTTP errors
            response.raise_for_status()
            
            # Return JSON response
            return response.json()
        except requests.exceptions.HTTPError as e:
            # Try to parse error response
            error_data = {}
            try:
                error_data = e.response.json()
            except:
                pass
            
            error_message = error_data.get("message", f"API request failed with status {e.response.status_code}")
            error_code = error_data.get("code")
            
            raise ShardXError(error_message, e.response.status_code, error_code) from e
        except requests.exceptions.RequestException as e:
            # Network error
            raise NetworkError(f"Request failed: {str(e)}") from e