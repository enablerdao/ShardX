import os
import binascii
import hashlib
import hmac
import base58
from typing import List, Optional, Dict, Any
from Crypto.Random import get_random_bytes

from .client import ShardXClient
from .models import Transaction, TransactionRequest
from .errors import WalletError, ValidationError

class Wallet:
    """
    ShardX wallet
    
    Manages keys and signing transactions
    """
    
    def __init__(
        self,
        client: ShardXClient,
        private_key: Optional[str] = None
    ):
        """
        Initialize a new wallet
        
        Args:
            client: ShardX client
            private_key: Private key (hex encoded)
        """
        self.client = client
        
        if private_key:
            self.private_key = private_key
            self.public_key = self._derive_public_key(self.private_key)
            self.address = self._derive_address(self.public_key)
        else:
            self.private_key = None
            self.public_key = None
            self.address = None
    
    @classmethod
    def create_random(cls, client: ShardXClient) -> "Wallet":
        """
        Create a new random wallet
        
        Args:
            client: ShardX client
            
        Returns:
            New wallet
        """
        # Generate random private key
        private_key = binascii.hexlify(get_random_bytes(32)).decode('ascii')
        
        return cls(client, private_key=private_key)
    
    @classmethod
    def from_mnemonic(cls, mnemonic: str, client: ShardXClient) -> "Wallet":
        """
        Create a wallet from mnemonic phrase
        
        Args:
            mnemonic: Mnemonic phrase
            client: ShardX client
            
        Returns:
            Wallet
        """
        # In a real implementation, this would use BIP39/BIP44
        # For simplicity, we'll just hash the mnemonic to get a private key
        private_key = hashlib.sha256(mnemonic.encode()).hexdigest()
        
        return cls(client, private_key=private_key)
    
    def get_address(self) -> str:
        """
        Get wallet address
        
        Returns:
            Wallet address
            
        Raises:
            WalletError: If wallet is not initialized with private key
        """
        if not self.address:
            raise WalletError("Wallet not initialized with private key", code="wallet_not_initialized")
        
        return self.address
    
    def get_public_key(self) -> str:
        """
        Get wallet public key
        
        Returns:
            Public key (hex encoded)
            
        Raises:
            WalletError: If wallet is not initialized with private key
        """
        if not self.public_key:
            raise WalletError("Wallet not initialized with private key", code="wallet_not_initialized")
        
        return self.public_key
    
    def sign(self, message: str) -> str:
        """
        Sign a message
        
        Args:
            message: Message to sign
            
        Returns:
            Signature (hex encoded)
            
        Raises:
            WalletError: If wallet is not initialized with private key
        """
        if not self.private_key:
            raise WalletError("Wallet not initialized with private key", code="wallet_not_initialized")
        
        # In a real implementation, this would use proper cryptographic signing
        # For simplicity, we'll just use HMAC
        signature = hmac.new(
            bytes.fromhex(self.private_key),
            message.encode(),
            hashlib.sha256
        ).hexdigest()
        
        return signature
    
    async def create_transaction(
        self,
        to: str,
        amount: str,
        data: Optional[str] = None
    ) -> Transaction:
        """
        Create and sign a transaction
        
        Args:
            to: Recipient address
            amount: Amount to send
            data: Optional transaction data
            
        Returns:
            Signed transaction
            
        Raises:
            WalletError: If wallet is not initialized with private key
            ValidationError: If inputs are invalid
        """
        if not self.private_key or not self.address:
            raise WalletError("Wallet not initialized with private key", code="wallet_not_initialized")
        
        # Validate inputs
        if not to:
            raise ValidationError("Recipient address is required")
        
        try:
            if float(amount) <= 0:
                raise ValidationError("Amount must be greater than 0")
        except ValueError:
            raise ValidationError("Invalid amount")
        
        # Create transaction request
        tx_request = TransactionRequest(
            from_address=self.address,
            to=to,
            amount=amount,
            data=data,
            signature=""  # Will be filled below
        )
        
        # Create message to sign
        message = f"{tx_request.from_address}:{tx_request.to}:{tx_request.amount}:{tx_request.data or ''}"
        
        # Sign the message
        tx_request.signature = self.sign(message)
        
        # Send transaction to the network
        return self.client.create_transaction(tx_request)
    
    async def get_balance(self) -> str:
        """
        Get account balance
        
        Returns:
            Account balance
            
        Raises:
            WalletError: If wallet is not initialized with private key
        """
        if not self.address:
            raise WalletError("Wallet not initialized with private key", code="wallet_not_initialized")
        
        account = self.client.get_account(self.address)
        return account.balance
    
    async def get_transactions(
        self,
        limit: int = 20,
        offset: int = 0
    ) -> List[Transaction]:
        """
        Get transaction history
        
        Args:
            limit: Maximum number of transactions to return
            offset: Offset for pagination
            
        Returns:
            List of transactions
            
        Raises:
            WalletError: If wallet is not initialized with private key
        """
        if not self.address:
            raise WalletError("Wallet not initialized with private key", code="wallet_not_initialized")
        
        return self.client.get_transactions_by_address(self.address, limit, offset)
    
    def _derive_public_key(self, private_key: str) -> str:
        """
        Derive public key from private key
        
        Args:
            private_key: Private key (hex encoded)
            
        Returns:
            Public key (hex encoded)
        """
        # In a real implementation, this would use proper cryptographic key derivation
        # For simplicity, we'll just hash the private key
        return hashlib.sha256(bytes.fromhex(private_key)).hexdigest()
    
    def _derive_address(self, public_key: str) -> str:
        """
        Derive address from public key
        
        Args:
            public_key: Public key (hex encoded)
            
        Returns:
            Address
        """
        # In a real implementation, this would use proper address derivation
        # For simplicity, we'll hash the public key and encode in base58
        sha256_hash = hashlib.sha256(bytes.fromhex(public_key)).digest()
        ripemd160_hash = hashlib.new('ripemd160', sha256_hash).digest()
        
        # Add version prefix (0x00)
        versioned_hash = b'\x00' + ripemd160_hash
        
        # Encode in base58
        return base58.b58encode(versioned_hash).decode('ascii')