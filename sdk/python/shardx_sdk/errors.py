from typing import Optional

class ShardXError(Exception):
    """Base exception for ShardX SDK"""
    
    def __init__(self, message: str, status: int = 0, code: Optional[str] = None):
        """
        Initialize a new ShardX error
        
        Args:
            message: Error message
            status: HTTP status code
            code: Error code
        """
        super().__init__(message)
        self.status = status
        self.code = code
    
    def __str__(self) -> str:
        if self.code:
            return f"{self.message} (status: {self.status}, code: {self.code})"
        return f"{self.message} (status: {self.status})"

class TransactionError(ShardXError):
    """Exception for transaction-related errors"""
    
    def __init__(self, message: str, status: int = 0, code: Optional[str] = None):
        """
        Initialize a new transaction error
        
        Args:
            message: Error message
            status: HTTP status code
            code: Error code
        """
        super().__init__(message, status, code)

class WalletError(ShardXError):
    """Exception for wallet-related errors"""
    
    def __init__(self, message: str, status: int = 0, code: Optional[str] = None):
        """
        Initialize a new wallet error
        
        Args:
            message: Error message
            status: HTTP status code
            code: Error code
        """
        super().__init__(message, status, code)

class MultisigError(ShardXError):
    """Exception for multisig-related errors"""
    
    def __init__(self, message: str, status: int = 0, code: Optional[str] = None):
        """
        Initialize a new multisig error
        
        Args:
            message: Error message
            status: HTTP status code
            code: Error code
        """
        super().__init__(message, status, code)

class NetworkError(ShardXError):
    """Exception for network-related errors"""
    
    def __init__(self, message: str):
        """
        Initialize a new network error
        
        Args:
            message: Error message
        """
        super().__init__(message, 0, "network_error")

class ValidationError(ShardXError):
    """Exception for validation errors"""
    
    def __init__(self, message: str, details: Optional[dict] = None):
        """
        Initialize a new validation error
        
        Args:
            message: Error message
            details: Validation details
        """
        super().__init__(message, 400, "validation_error")
        self.details = details
    
    def __str__(self) -> str:
        if self.details:
            return f"{self.message} (details: {self.details})"
        return super().__str__()