use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

use crate::error::Error;
use crate::transaction::Transaction;

/// トランザクション分析結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalysis {
    /// トランザクションID
    pub transaction_id: String,
    /// 基本情報
    pub basic_info: BasicInfo,
    /// 送信者情報
    pub sender_info: AddressInfo,
    /// 受信者情報
    pub receiver_info: AddressInfo,
    /// ネットワーク情報
    pub network_info: NetworkInfo,
    /// 関連トランザクション
    pub related_transactions: Vec<RelatedTransaction>,
    /// クロスシャード情報（該当する場合）
    pub cross_shard_info: Option<CrossShardInfo>,
    /// リスク評価
    pub risk_assessment: RiskAssessment,
}

/// 基本情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicInfo {
    /// トランザクション時刻
    pub timestamp: DateTime<Utc>,
    /// 金額
    pub amount: String,
    /// 手数料
    pub fee: String,
    /// 確認数
    pub confirmations: u64,
    /// 処理時間（ミリ秒）
    pub processing_time_ms: u64,
    /// トランザクションサイズ（バイト）
    pub size_bytes: u64,
    /// シャードID
    pub shard_id: String,
}

/// アドレス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    /// アドレス
    pub address: String,
    /// 残高
    pub balance: String,
    /// トランザクション数
    pub transaction_count: u64,
    /// 最初のトランザクション時刻
    pub first_seen: DateTime<Utc>,
    /// 最後のトランザクション時刻
    pub last_seen: DateTime<Utc>,
    /// アドレスタイプ
    pub address_type: AddressType,
    /// タグ（該当する場合）
    pub tags: Vec<String>,
}

/// アドレスタイプ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    /// 標準アドレス
    Standard,
    /// コントラクト
    Contract,
    /// マルチシグ
    Multisig,
    /// 取引所
    Exchange,
    /// マイナー
    Miner,
    /// その他
    Other,
}

/// ネットワーク情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// 伝播時間（ミリ秒）
    pub propagation_time_ms: u64,
    /// 確認時間（ミリ秒）
    pub confirmation_time_ms: u64,
    /// 最初に観測したノード
    pub first_seen_by: String,
    /// 含まれるブロック
    pub included_in_block: String,
    /// ブロック高
    pub block_height: u64,
    /// ブロック内のインデックス
    pub block_index: u64,
}

/// 関連トランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedTransaction {
    /// トランザクションID
    pub id: String,
    /// 関係タイプ
    pub relation_type: RelationType,
    /// 時間差（秒）
    pub time_difference_seconds: i64,
    /// 金額
    pub amount: String,
}

/// 関係タイプ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    /// 親トランザクション
    Parent,
    /// 子トランザクション
    Child,
    /// 同じ送信者からのトランザクション
    SameSender,
    /// 同じ受信者へのトランザクション
    SameReceiver,
    /// 同じブロック内のトランザクション
    SameBlock,
}

/// クロスシャード情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardInfo {
    /// 親トランザクションID
    pub parent_id: String,
    /// 子トランザクションID
    pub child_ids: Vec<String>,
    /// 関連するシャード
    pub involved_shards: Vec<String>,
    /// 完了までの時間（ミリ秒）
    pub completion_time_ms: u64,
    /// 状態
    pub status: String,
}

/// リスク評価
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// リスクスコア（0-100）
    pub risk_score: u8,
    /// リスクレベル
    pub risk_level: RiskLevel,
    /// リスク要因
    pub risk_factors: Vec<RiskFactor>,
}

/// リスクレベル
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 低リスク
    Low,
    /// 中リスク
    Medium,
    /// 高リスク
    High,
}

/// リスク要因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// 要因タイプ
    pub factor_type: String,
    /// 説明
    pub description: String,
    /// 重要度（0-100）
    pub severity: u8,
}

/// トランザクション分析マネージャー
pub struct TransactionAnalysisManager {
    /// アドレスキャッシュ
    address_cache: HashMap<String, AddressInfo>,
    /// タグ付きアドレス
    tagged_addresses: HashMap<String, Vec<String>>,
}

impl TransactionAnalysisManager {
    /// 新しいトランザクション分析マネージャーを作成
    pub fn new() -> Self {
        Self {
            address_cache: HashMap::new(),
            tagged_addresses: HashMap::new(),
        }
    }
    
    /// トランザクションを分析
    pub fn analyze_transaction(
        &mut self,
        transaction: &Transaction,
        related_transactions: &[Transaction],
        block_height: u64,
        current_height: u64,
    ) -> Result<TransactionAnalysis, Error> {
        // 基本情報を取得
        let basic_info = self.get_basic_info(transaction, block_height, current_height)?;
        
        // 送信者情報を取得
        let sender_info = self.get_address_info(&transaction.from, related_transactions)?;
        
        // 受信者情報を取得
        let receiver_info = self.get_address_info(&transaction.to, related_transactions)?;
        
        // ネットワーク情報を取得
        let network_info = self.get_network_info(transaction, block_height)?;
        
        // 関連トランザクションを取得
        let related_txs = self.get_related_transactions(transaction, related_transactions)?;
        
        // クロスシャード情報を取得
        let cross_shard_info = self.get_cross_shard_info(transaction, related_transactions)?;
        
        // リスク評価を行う
        let risk_assessment = self.assess_risk(
            transaction,
            &sender_info,
            &receiver_info,
            related_transactions,
            &cross_shard_info,
        )?;
        
        Ok(TransactionAnalysis {
            transaction_id: transaction.id.clone(),
            basic_info,
            sender_info,
            receiver_info,
            network_info,
            related_transactions: related_txs,
            cross_shard_info,
            risk_assessment,
        })
    }
    
    /// 基本情報を取得
    fn get_basic_info(
        &self,
        transaction: &Transaction,
        block_height: u64,
        current_height: u64,
    ) -> Result<BasicInfo, Error> {
        let timestamp = DateTime::<Utc>::from_timestamp(transaction.timestamp as i64, 0)
            .ok_or_else(|| Error::ValidationError("Invalid transaction timestamp".to_string()))?;
        
        // トランザクションサイズを計算（簡易的な実装）
        let size_bytes = transaction.data.as_ref().map(|d| d.len()).unwrap_or(0) + 200; // 基本サイズ + データサイズ
        
        // 確認数を計算
        let confirmations = if block_height > 0 {
            current_height.saturating_sub(block_height) + 1
        } else {
            0
        };
        
        // 処理時間はダミー値（実際の実装では、メモリプールに入った時刻とブロックに含まれた時刻の差を使用）
        let processing_time_ms = 1500;
        
        Ok(BasicInfo {
            timestamp,
            amount: transaction.amount.clone(),
            fee: transaction.fee.clone(),
            confirmations,
            processing_time_ms,
            size_bytes: size_bytes as u64,
            shard_id: transaction.shard_id.clone(),
        })
    }
    
    /// アドレス情報を取得
    fn get_address_info(
        &mut self,
        address: &str,
        transactions: &[Transaction],
    ) -> Result<AddressInfo, Error> {
        // キャッシュをチェック
        if let Some(info) = self.address_cache.get(address) {
            return Ok(info.clone());
        }
        
        // アドレスに関連するトランザクションをフィルタリング
        let related_txs: Vec<&Transaction> = transactions
            .iter()
            .filter(|tx| tx.from == address || tx.to == address)
            .collect();
        
        if related_txs.is_empty() {
            // 関連するトランザクションがない場合はダミーデータを返す
            let now = Utc::now();
            
            let info = AddressInfo {
                address: address.to_string(),
                balance: "0".to_string(),
                transaction_count: 0,
                first_seen: now,
                last_seen: now,
                address_type: AddressType::Standard,
                tags: Vec::new(),
            };
            
            self.address_cache.insert(address.to_string(), info.clone());
            
            return Ok(info);
        }
        
        // 最初と最後のトランザクション時刻を取得
        let mut first_seen = Utc::now();
        let mut last_seen = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
        
        for tx in &related_txs {
            let tx_time = DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0)
                .ok_or_else(|| Error::ValidationError("Invalid transaction timestamp".to_string()))?;
            
            if tx_time < first_seen {
                first_seen = tx_time;
            }
            
            if tx_time > last_seen {
                last_seen = tx_time;
            }
        }
        
        // アドレスタイプを推定
        let address_type = self.estimate_address_type(address, &related_txs);
        
        // タグを取得
        let tags = self.tagged_addresses.get(address).cloned().unwrap_or_default();
        
        // 残高を計算（簡易的な実装）
        let mut balance = 0.0;
        
        for tx in &related_txs {
            let amount = tx.amount.parse::<f64>().unwrap_or(0.0);
            
            if tx.to == address {
                balance += amount;
            }
            
            if tx.from == address {
                balance -= amount;
                
                // 手数料も引く
                let fee = tx.fee.parse::<f64>().unwrap_or(0.0);
                balance -= fee;
            }
        }
        
        let info = AddressInfo {
            address: address.to_string(),
            balance: format!("{:.8}", balance.max(0.0)),
            transaction_count: related_txs.len() as u64,
            first_seen,
            last_seen,
            address_type,
            tags,
        };
        
        // キャッシュに保存
        self.address_cache.insert(address.to_string(), info.clone());
        
        Ok(info)
    }
    
    /// アドレスタイプを推定
    fn estimate_address_type(&self, address: &str, transactions: &[&Transaction]) -> AddressType {
        // 実際の実装では、アドレスのパターンや取引パターンに基づいて推定
        // ここでは簡易的な実装として、トランザクション数に基づいて推定
        
        if transactions.len() > 1000 {
            return AddressType::Exchange;
        }
        
        if address.starts_with("0x") {
            return AddressType::Contract;
        }
        
        if address.contains("multi") {
            return AddressType::Multisig;
        }
        
        AddressType::Standard
    }
    
    /// ネットワーク情報を取得
    fn get_network_info(
        &self,
        transaction: &Transaction,
        block_height: u64,
    ) -> Result<NetworkInfo, Error> {
        // 実際の実装では、ネットワークログやブロック情報から取得
        // ここでは簡易的な実装として、ダミーデータを返す
        
        Ok(NetworkInfo {
            propagation_time_ms: 250,
            confirmation_time_ms: 1500,
            first_seen_by: "node1.shardx.io".to_string(),
            included_in_block: transaction.block_hash.clone().unwrap_or_default(),
            block_height,
            block_index: 10,
        })
    }
    
    /// 関連トランザクションを取得
    fn get_related_transactions(
        &self,
        transaction: &Transaction,
        all_transactions: &[Transaction],
    ) -> Result<Vec<RelatedTransaction>, Error> {
        let mut related_txs = Vec::new();
        let tx_time = DateTime::<Utc>::from_timestamp(transaction.timestamp as i64, 0)
            .ok_or_else(|| Error::ValidationError("Invalid transaction timestamp".to_string()))?;
        
        for tx in all_transactions {
            if tx.id == transaction.id {
                continue;
            }
            
            let other_tx_time = DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0)
                .ok_or_else(|| Error::ValidationError("Invalid transaction timestamp".to_string()))?;
            
            let time_diff = (other_tx_time - tx_time).num_seconds();
            
            let relation_type = if let Some(parent_id) = &transaction.parent_id {
                if &tx.id == parent_id {
                    Some(RelationType::Parent)
                } else {
                    None
                }
            } else if tx.parent_id.as_ref().map(|id| id == &transaction.id).unwrap_or(false) {
                Some(RelationType::Child)
            } else if tx.from == transaction.from {
                Some(RelationType::SameSender)
            } else if tx.to == transaction.to {
                Some(RelationType::SameReceiver)
            } else if tx.block_hash == transaction.block_hash && tx.block_hash.is_some() {
                Some(RelationType::SameBlock)
            } else {
                None
            };
            
            if let Some(relation) = relation_type {
                related_txs.push(RelatedTransaction {
                    id: tx.id.clone(),
                    relation_type: relation,
                    time_difference_seconds: time_diff,
                    amount: tx.amount.clone(),
                });
            }
        }
        
        // 最大10件に制限
        related_txs.sort_by(|a, b| a.time_difference_seconds.abs().cmp(&b.time_difference_seconds.abs()));
        related_txs.truncate(10);
        
        Ok(related_txs)
    }
    
    /// クロスシャード情報を取得
    fn get_cross_shard_info(
        &self,
        transaction: &Transaction,
        all_transactions: &[Transaction],
    ) -> Result<Option<CrossShardInfo>, Error> {
        // 親トランザクションの場合
        if transaction.parent_id.is_none() {
            // 子トランザクションを検索
            let child_txs: Vec<&Transaction> = all_transactions
                .iter()
                .filter(|tx| tx.parent_id.as_ref().map(|id| id == &transaction.id).unwrap_or(false))
                .collect();
            
            if child_txs.is_empty() {
                return Ok(None);
            }
            
            // 関連するシャードを収集
            let mut involved_shards = HashSet::new();
            involved_shards.insert(transaction.shard_id.clone());
            
            let child_ids: Vec<String> = child_txs
                .iter()
                .map(|tx| {
                    involved_shards.insert(tx.shard_id.clone());
                    tx.id.clone()
                })
                .collect();
            
            // 完了時間を計算
            let tx_time = DateTime::<Utc>::from_timestamp(transaction.timestamp as i64, 0)
                .ok_or_else(|| Error::ValidationError("Invalid transaction timestamp".to_string()))?;
            
            let latest_child_time = child_txs
                .iter()
                .filter_map(|tx| DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0))
                .max()
                .unwrap_or(tx_time);
            
            let completion_time_ms = (latest_child_time - tx_time).num_milliseconds().max(0) as u64;
            
            // 状態を判定
            let all_confirmed = child_txs.iter().all(|tx| tx.status == crate::transaction::TransactionStatus::Confirmed);
            let status = if all_confirmed {
                "Completed".to_string()
            } else {
                "Pending".to_string()
            };
            
            return Ok(Some(CrossShardInfo {
                parent_id: transaction.id.clone(),
                child_ids,
                involved_shards: involved_shards.into_iter().collect(),
                completion_time_ms,
                status,
            }));
        }
        
        // 子トランザクションの場合
        if let Some(parent_id) = &transaction.parent_id {
            // 親トランザクションを検索
            let parent_tx = all_transactions
                .iter()
                .find(|tx| tx.id == *parent_id);
            
            if let Some(parent) = parent_tx {
                // 他の子トランザクションを検索
                let sibling_txs: Vec<&Transaction> = all_transactions
                    .iter()
                    .filter(|tx| {
                        tx.id != transaction.id && tx.parent_id.as_ref().map(|id| id == parent_id).unwrap_or(false)
                    })
                    .collect();
                
                // 関連するシャードを収集
                let mut involved_shards = HashSet::new();
                involved_shards.insert(parent.shard_id.clone());
                involved_shards.insert(transaction.shard_id.clone());
                
                let mut child_ids = vec![transaction.id.clone()];
                
                for tx in &sibling_txs {
                    involved_shards.insert(tx.shard_id.clone());
                    child_ids.push(tx.id.clone());
                }
                
                // 完了時間を計算
                let parent_time = DateTime::<Utc>::from_timestamp(parent.timestamp as i64, 0)
                    .ok_or_else(|| Error::ValidationError("Invalid transaction timestamp".to_string()))?;
                
                let latest_child_time = sibling_txs
                    .iter()
                    .filter_map(|tx| DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0))
                    .chain(std::iter::once(DateTime::<Utc>::from_timestamp(transaction.timestamp as i64, 0).unwrap()))
                    .max()
                    .unwrap_or(parent_time);
                
                let completion_time_ms = (latest_child_time - parent_time).num_milliseconds().max(0) as u64;
                
                // 状態を判定
                let all_confirmed = std::iter::once(transaction)
                    .chain(sibling_txs.iter().copied())
                    .all(|tx| tx.status == crate::transaction::TransactionStatus::Confirmed);
                
                let status = if all_confirmed {
                    "Completed".to_string()
                } else {
                    "Pending".to_string()
                };
                
                return Ok(Some(CrossShardInfo {
                    parent_id: parent_id.clone(),
                    child_ids,
                    involved_shards: involved_shards.into_iter().collect(),
                    completion_time_ms,
                    status,
                }));
            }
        }
        
        Ok(None)
    }
    
    /// リスク評価を行う
    fn assess_risk(
        &self,
        transaction: &Transaction,
        sender_info: &AddressInfo,
        receiver_info: &AddressInfo,
        all_transactions: &[Transaction],
        cross_shard_info: &Option<CrossShardInfo>,
    ) -> Result<RiskAssessment, Error> {
        let mut risk_factors = Vec::new();
        let mut risk_score = 0;
        
        // 新しいアドレスからの大きな取引
        let amount = transaction.amount.parse::<f64>().unwrap_or(0.0);
        let now = Utc::now();
        let sender_age_days = (now - sender_info.first_seen).num_days();
        
        if sender_age_days < 7 && amount > 1000.0 {
            risk_factors.push(RiskFactor {
                factor_type: "NewAddressLargeTransaction".to_string(),
                description: "Large transaction from a new address".to_string(),
                severity: 70,
            });
            
            risk_score += 20;
        }
        
        // 異常なトランザクションパターン
        let sender_tx_count = sender_info.transaction_count;
        
        if sender_tx_count < 5 && amount > 10000.0 {
            risk_factors.push(RiskFactor {
                factor_type: "UnusualTransactionPattern".to_string(),
                description: "Unusually large transaction for an address with few transactions".to_string(),
                severity: 60,
            });
            
            risk_score += 15;
        }
        
        // タグ付きアドレス
        for tag in &sender_info.tags {
            if tag == "suspicious" || tag == "scam" || tag == "blacklisted" {
                risk_factors.push(RiskFactor {
                    factor_type: "TaggedAddress".to_string(),
                    description: format!("Sender address is tagged as '{}'", tag),
                    severity: 90,
                });
                
                risk_score += 30;
            }
        }
        
        for tag in &receiver_info.tags {
            if tag == "suspicious" || tag == "scam" || tag == "blacklisted" {
                risk_factors.push(RiskFactor {
                    factor_type: "TaggedAddress".to_string(),
                    description: format!("Receiver address is tagged as '{}'", tag),
                    severity: 80,
                });
                
                risk_score += 25;
            }
        }
        
        // クロスシャードトランザクションの複雑さ
        if let Some(cross_info) = cross_shard_info {
            if cross_info.involved_shards.len() > 3 {
                risk_factors.push(RiskFactor {
                    factor_type: "ComplexCrossShardTransaction".to_string(),
                    description: format!(
                        "Complex cross-shard transaction involving {} shards",
                        cross_info.involved_shards.len()
                    ),
                    severity: 40,
                });
                
                risk_score += 10;
            }
        }
        
        // リスクレベルを判定
        let risk_level = if risk_score >= 50 {
            RiskLevel::High
        } else if risk_score >= 20 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };
        
        Ok(RiskAssessment {
            risk_score: risk_score.min(100) as u8,
            risk_level,
            risk_factors,
        })
    }
    
    /// アドレスにタグを追加
    pub fn add_address_tag(&mut self, address: &str, tag: &str) {
        let entry = self.tagged_addresses.entry(address.to_string()).or_insert_with(Vec::new);
        
        if !entry.contains(&tag.to_string()) {
            entry.push(tag.to_string());
        }
        
        // キャッシュを更新
        if let Some(info) = self.address_cache.get_mut(address) {
            if !info.tags.contains(&tag.to_string()) {
                info.tags.push(tag.to_string());
            }
        }
    }
    
    /// アドレスからタグを削除
    pub fn remove_address_tag(&mut self, address: &str, tag: &str) {
        if let Some(tags) = self.tagged_addresses.get_mut(address) {
            tags.retain(|t| t != tag);
        }
        
        // キャッシュを更新
        if let Some(info) = self.address_cache.get_mut(address) {
            info.tags.retain(|t| t != tag);
        }
    }
    
    /// アドレスのタグを取得
    pub fn get_address_tags(&self, address: &str) -> Vec<String> {
        self.tagged_addresses.get(address).cloned().unwrap_or_default()
    }
    
    /// キャッシュをクリア
    pub fn clear_cache(&mut self) {
        self.address_cache.clear();
    }
}

impl Default for TransactionAnalysisManager {
    fn default() -> Self {
        Self::new()
    }
}