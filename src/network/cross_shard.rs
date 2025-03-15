use serde::{Serialize, Deserialize};

use crate::transaction::Transaction;

/// ネットワークメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// クロスシャードトランザクションの準備フェーズ
    PrepareCrossShardTransaction {
        /// トランザクション
        transaction: Transaction,
    },
    /// クロスシャードトランザクションの準備完了通知
    PrepareCrossShardTransactionAck {
        /// トランザクションID
        transaction_id: String,
        /// シャードID
        shard_id: String,
    },
    /// クロスシャードトランザクションのコミットフェーズ
    CommitCrossShardTransaction {
        /// トランザクション
        transaction: Transaction,
    },
    /// クロスシャードトランザクションのコミット完了通知
    CommitCrossShardTransactionAck {
        /// トランザクションID
        transaction_id: String,
        /// シャードID
        shard_id: String,
    },
    /// クロスシャードトランザクションの中止
    AbortCrossShardTransaction {
        /// トランザクションID
        transaction_id: String,
    },
    /// クロスシャードトランザクションの中止通知
    AbortCrossShardTransactionAck {
        /// トランザクションID
        transaction_id: String,
        /// シャードID
        shard_id: String,
    },
}