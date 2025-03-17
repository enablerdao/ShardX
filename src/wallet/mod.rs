pub mod multisig;

// 旧マルチシグウォレット実装
pub use multisig::{
    MultisigTransaction as OldMultisigTransaction,
    MultisigTransactionStatus as OldMultisigTransactionStatus, MultisigWallet as OldMultisigWallet,
    MultisigWalletManager,
};

// 新マルチシグウォレット実装
pub use multisig::config::{
    ApprovalHierarchy, ApprovalLevel, AutoApprovalRule, MultisigConfig, NotificationDestination,
    NotificationSettings, NotificationType, RejectionRule,
};
pub use multisig::transaction::{
    MultisigTransaction, MultisigTransactionStatus, TransactionAction, TransactionHistoryEntry,
    TransactionStep, TransactionStepStatus,
};
pub use multisig::wallet::MultisigWallet;
