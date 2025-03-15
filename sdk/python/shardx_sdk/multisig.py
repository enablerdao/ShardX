import json
import binascii
from typing import List, Optional, Dict, Any
from dataclasses import dataclass

from .client import ShardXClient
from .wallet import Wallet
from .models import MultisigWallet, MultisigWalletRequest, Transaction
from .errors import MultisigError, ValidationError

@dataclass
class SignatureInfo:
    """Signature information"""
    signer: str
    signature: str

@dataclass
class MultisigTransaction(Transaction):
    """Multisig transaction"""
    signatures: List[SignatureInfo]
    required_signatures: int

class MultisigManager:
    """
    Multisig wallet manager
    
    Utility class for working with multisig wallets
    """
    
    def __init__(self, client: ShardXClient):
        """
        Initialize a new multisig manager
        
        Args:
            client: ShardX client
        """
        self.client = client
    
    async def create_wallet(
        self,
        wallet: Wallet,
        name: str,
        signers: List[str],
        required_signatures: int
    ) -> MultisigWallet:
        """
        Create a new multisig wallet
        
        Args:
            wallet: Owner wallet
            name: Wallet name
            signers: Signer addresses
            required_signatures: Required signatures
            
        Returns:
            Created multisig wallet
            
        Raises:
            ValidationError: If inputs are invalid
        """
        # Validate inputs
        if not name:
            raise ValidationError("Wallet name is required")
        
        if not signers or len(signers) == 0:
            raise ValidationError("At least one signer is required")
        
        if required_signatures <= 0 or required_signatures > len(signers):
            raise ValidationError(f"Required signatures must be between 1 and {len(signers)}")
        
        # Create wallet request
        wallet_request = MultisigWalletRequest(
            name=name,
            owner_id=wallet.get_address(),
            signers=signers,
            required_signatures=required_signatures
        )
        
        # Create multisig wallet
        return self.client.create_multisig_wallet(wallet_request)
    
    async def get_wallet(self, wallet_id: str) -> MultisigWallet:
        """
        Get multisig wallet by ID
        
        Args:
            wallet_id: Multisig wallet ID
            
        Returns:
            Multisig wallet
        """
        return self.client.get_multisig_wallet(wallet_id)
    
    async def get_wallets_by_owner(self, owner_address: str) -> List[MultisigWallet]:
        """
        Get multisig wallets by owner
        
        Args:
            owner_address: Owner address
            
        Returns:
            List of multisig wallets
        """
        return self.client.get_multisig_wallets_by_owner(owner_address)
    
    async def create_transaction(
        self,
        wallet: Wallet,
        multisig_id: str,
        to: str,
        amount: str,
        data: Optional[str] = None
    ) -> MultisigTransaction:
        """
        Create a multisig transaction
        
        Args:
            wallet: Signer wallet
            multisig_id: Multisig wallet ID
            to: Recipient address
            amount: Amount to send
            data: Optional transaction data
            
        Returns:
            Created multisig transaction
            
        Raises:
            MultisigError: If wallet is not a signer
        """
        # Get multisig wallet
        multisig_wallet = self.client.get_multisig_wallet(multisig_id)
        
        # Check if wallet is a signer
        signer_address = wallet.get_address()
        if signer_address not in multisig_wallet.signers:
            raise MultisigError(
                f"Address {signer_address} is not a signer for this multisig wallet",
                403,
                "not_a_signer"
            )
        
        # Create transaction data
        tx_data = {
            "multisigId": multisig_id,
            "to": to,
            "amount": amount,
            "data": data,
            "initiator": signer_address
        }
        
        # Encode transaction data
        encoded_data = binascii.hexlify(json.dumps(tx_data).encode()).decode()
        
        # Create transaction
        tx = await wallet.create_transaction(multisig_wallet.id, "0", encoded_data)
        
        # Convert to multisig transaction
        multisig_tx = MultisigTransaction(
            id=tx.id,
            status=tx.status,
            timestamp=tx.timestamp,
            from_address=tx.from_address,
            to=tx.to,
            amount=tx.amount,
            fee=tx.fee,
            data=tx.data,
            block_hash=tx.block_hash,
            block_height=tx.block_height,
            shard_id=tx.shard_id,
            parent_ids=tx.parent_ids,
            signatures=[
                SignatureInfo(
                    signer=signer_address,
                    signature=tx.data or ""
                )
            ],
            required_signatures=multisig_wallet.required_signatures
        )
        
        return multisig_tx
    
    async def sign_transaction(
        self,
        wallet: Wallet,
        tx_id: str
    ) -> MultisigTransaction:
        """
        Sign a multisig transaction
        
        Args:
            wallet: Signer wallet
            tx_id: Transaction ID
            
        Returns:
            Updated multisig transaction
            
        Raises:
            MultisigError: If wallet is not a signer
        """
        # Get transaction
        tx = self.client.get_transaction(tx_id)
        
        # Decode transaction data
        tx_data = json.loads(binascii.unhexlify(tx.data or "").decode())
        
        # Get multisig wallet
        multisig_wallet = self.client.get_multisig_wallet(tx_data["multisigId"])
        
        # Check if wallet is a signer
        signer_address = wallet.get_address()
        if signer_address not in multisig_wallet.signers:
            raise MultisigError(
                f"Address {signer_address} is not a signer for this multisig wallet",
                403,
                "not_a_signer"
            )
        
        # Create signature
        signature = wallet.sign(tx_id)
        
        # Create signature transaction
        signature_data = {
            "signer": signer_address,
            "signature": signature
        }
        
        encoded_data = binascii.hexlify(json.dumps(signature_data).encode()).decode()
        
        signature_tx = await wallet.create_transaction(
            tx.id,
            "0",
            encoded_data
        )
        
        # Get updated multisig transaction
        updated_tx = self.client.get_transaction(tx_id)
        
        # Convert to multisig transaction
        # In a real implementation, this would get the signatures from the API
        multisig_tx = MultisigTransaction(
            id=updated_tx.id,
            status=updated_tx.status,
            timestamp=updated_tx.timestamp,
            from_address=updated_tx.from_address,
            to=updated_tx.to,
            amount=updated_tx.amount,
            fee=updated_tx.fee,
            data=updated_tx.data,
            block_hash=updated_tx.block_hash,
            block_height=updated_tx.block_height,
            shard_id=updated_tx.shard_id,
            parent_ids=updated_tx.parent_ids,
            signatures=[
                SignatureInfo(
                    signer=signer_address,
                    signature=signature
                )
            ],
            required_signatures=multisig_wallet.required_signatures
        )
        
        return multisig_tx
    
    async def execute_transaction(
        self,
        wallet: Wallet,
        tx_id: str
    ) -> Transaction:
        """
        Execute a multisig transaction
        
        Args:
            wallet: Signer wallet
            tx_id: Transaction ID
            
        Returns:
            Executed transaction
            
        Raises:
            MultisigError: If wallet is not a signer
        """
        # Get transaction
        tx = self.client.get_transaction(tx_id)
        
        # Decode transaction data
        tx_data = json.loads(binascii.unhexlify(tx.data or "").decode())
        
        # Get multisig wallet
        multisig_wallet = self.client.get_multisig_wallet(tx_data["multisigId"])
        
        # Check if wallet is a signer
        signer_address = wallet.get_address()
        if signer_address not in multisig_wallet.signers:
            raise MultisigError(
                f"Address {signer_address} is not a signer for this multisig wallet",
                403,
                "not_a_signer"
            )
        
        # Create execution transaction
        execution_data = {
            "execute": True
        }
        
        encoded_data = binascii.hexlify(json.dumps(execution_data).encode()).decode()
        
        execution_tx = await wallet.create_transaction(
            tx.id,
            "0",
            encoded_data
        )
        
        return execution_tx