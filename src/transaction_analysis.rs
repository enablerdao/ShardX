use crate::transaction::{Transaction, TransactionStatus, DAG};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// トランザクション分析結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalysis {
    /// 分析対象期間
    pub period: AnalysisPeriod,
    /// トランザクション総数
    pub total_transactions: usize,
    /// 確認済みトランザクション数
    pub confirmed_transactions: usize,
    /// 拒否されたトランザクション数
    pub rejected_transactions: usize,
    /// 保留中のトランザクション数
    pub pending_transactions: usize,
    /// 平均確認時間（秒）
    pub avg_confirmation_time: f64,
    /// トランザクションボリューム（時間帯別）
    pub volume_by_hour: HashMap<u32, usize>,
    /// トランザクションタイプ別の分布
    pub transaction_types: HashMap<String, usize>,
    /// 最も活発なアドレス（上位10件）
    pub top_active_addresses: Vec<(String, usize)>,
    /// トランザクションのグラフ特性
    pub graph_metrics: GraphMetrics,
}

/// 分析期間
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnalysisPeriod {
    /// 過去24時間
    Last24Hours,
    /// 過去7日間
    LastWeek,
    /// 過去30日間
    LastMonth,
    /// カスタム期間
    Custom {
        /// 開始日時
        start: DateTime<Utc>,
        /// 終了日時
        end: DateTime<Utc>,
    },
}

/// グラフメトリクス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    /// 平均次数（1つのトランザクションが参照する親トランザクションの平均数）
    pub avg_degree: f64,
    /// 最大次数
    pub max_degree: usize,
    /// クラスタリング係数
    pub clustering_coefficient: f64,
    /// 最長パス長
    pub longest_path: usize,
}

/// トランザクションパターン
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPattern {
    /// パターンID
    pub id: String,
    /// パターン名
    pub name: String,
    /// パターンの説明
    pub description: String,
    /// 検出回数
    pub occurrences: usize,
    /// 関連トランザクションID
    pub related_transactions: Vec<String>,
}

/// トランザクション分析マネージャー
pub struct TransactionAnalyzer {
    /// DAGの参照
    dag: Arc<DAG>,
}

impl TransactionAnalyzer {
    /// 新しいTransactionAnalyzerを作成
    pub fn new(dag: Arc<DAG>) -> Self {
        Self { dag }
    }
    
    /// 指定した期間のトランザクション分析を実行
    pub fn analyze(&self, period: AnalysisPeriod) -> TransactionAnalysis {
        let now = Utc::now();
        
        // 分析期間の開始・終了日時を決定
        let (start_time, end_time) = match period {
            AnalysisPeriod::Last24Hours => (now - Duration::hours(24), now),
            AnalysisPeriod::LastWeek => (now - Duration::weeks(1), now),
            AnalysisPeriod::LastMonth => (now - Duration::days(30), now),
            AnalysisPeriod::Custom { start, end } => (start, end),
        };
        
        // 期間内のトランザクションを抽出
        let transactions: Vec<Transaction> = self.dag
            .transactions
            .iter()
            .filter(|tx| tx.created_at >= start_time && tx.created_at <= end_time)
            .map(|tx| tx.clone())
            .collect();
        
        // 基本的な統計情報を計算
        let total = transactions.len();
        let confirmed = transactions.iter().filter(|tx| tx.status == TransactionStatus::Confirmed).count();
        let rejected = transactions.iter().filter(|tx| tx.status == TransactionStatus::Rejected).count();
        let pending = transactions.iter().filter(|tx| tx.status == TransactionStatus::Pending).count();
        
        // 確認時間の計算
        let confirmation_times: Vec<i64> = transactions
            .iter()
            .filter(|tx| tx.status == TransactionStatus::Confirmed)
            .map(|tx| {
                // 実際の実装では、確認時間のデータが必要
                // ここではランダムな値を使用
                let random_seconds = tx.id.as_bytes()[0] as i64 % 300;
                random_seconds
            })
            .collect();
        
        let avg_confirmation_time = if confirmation_times.is_empty() {
            0.0
        } else {
            confirmation_times.iter().sum::<i64>() as f64 / confirmation_times.len() as f64
        };
        
        // 時間帯別のボリューム
        let mut volume_by_hour: HashMap<u32, usize> = HashMap::new();
        for tx in &transactions {
            let hour = tx.created_at.hour();
            *volume_by_hour.entry(hour).or_insert(0) += 1;
        }
        
        // トランザクションタイプの分析（ペイロードの最初のバイトで簡易判定）
        let mut transaction_types: HashMap<String, usize> = HashMap::new();
        for tx in &transactions {
            let tx_type = if tx.payload.is_empty() {
                "empty".to_string()
            } else {
                match tx.payload[0] {
                    0..=50 => "transfer".to_string(),
                    51..=100 => "token_transfer".to_string(),
                    101..=150 => "swap".to_string(),
                    151..=200 => "liquidity".to_string(),
                    _ => "contract".to_string(),
                }
            };
            
            *transaction_types.entry(tx_type).or_insert(0) += 1;
        }
        
        // アクティブなアドレスの分析
        let mut address_activity: HashMap<String, usize> = HashMap::new();
        for tx in &transactions {
            // 実際の実装では、トランザクションからアドレスを抽出
            // ここでは簡易的に最初の8文字を使用
            if !tx.payload.is_empty() && tx.payload.len() >= 8 {
                let addr = hex::encode(&tx.payload[0..8]);
                *address_activity.entry(addr).or_insert(0) += 1;
            }
        }
        
        // 上位10件のアクティブアドレスを抽出
        let mut top_addresses: Vec<(String, usize)> = address_activity.into_iter().collect();
        top_addresses.sort_by(|a, b| b.1.cmp(&a.1));
        let top_active_addresses = top_addresses.into_iter().take(10).collect();
        
        // グラフメトリクスの計算
        let graph_metrics = self.calculate_graph_metrics(&transactions);
        
        TransactionAnalysis {
            period,
            total_transactions: total,
            confirmed_transactions: confirmed,
            rejected_transactions: rejected,
            pending_transactions: pending,
            avg_confirmation_time,
            volume_by_hour,
            transaction_types,
            top_active_addresses,
            graph_metrics,
        }
    }
    
    /// グラフメトリクスを計算
    fn calculate_graph_metrics(&self, transactions: &[Transaction]) -> GraphMetrics {
        // トランザクションIDのセットを作成
        let tx_ids: HashSet<String> = transactions.iter().map(|tx| tx.id.clone()).collect();
        
        // 次数の計算
        let mut degrees = Vec::new();
        let mut max_degree = 0;
        
        for tx in transactions {
            // このトランザクションが参照している親トランザクションのうち、
            // 分析対象期間内に存在するものの数をカウント
            let degree = tx.parent_ids.iter().filter(|id| tx_ids.contains(*id)).count();
            degrees.push(degree);
            max_degree = max_degree.max(degree);
        }
        
        // 平均次数
        let avg_degree = if degrees.is_empty() {
            0.0
        } else {
            degrees.iter().sum::<usize>() as f64 / degrees.len() as f64
        };
        
        // クラスタリング係数（簡易計算）
        // 実際の実装では、より正確な計算が必要
        let clustering_coefficient = if transactions.len() < 3 {
            0.0
        } else {
            // ランダムな値（0.1〜0.5）
            0.1 + (transactions[0].id.as_bytes()[0] as f64 % 40.0) / 100.0
        };
        
        // 最長パス（簡易計算）
        // 実際の実装では、グラフ探索アルゴリズムを使用
        let longest_path = if transactions.is_empty() {
            0
        } else {
            // トランザクション数の平方根程度の値
            (transactions.len() as f64).sqrt() as usize + 1
        };
        
        GraphMetrics {
            avg_degree,
            max_degree,
            clustering_coefficient,
            longest_path,
        }
    }
    
    /// トランザクションパターンを検出
    pub fn detect_patterns(&self, period: AnalysisPeriod) -> Vec<TransactionPattern> {
        let now = Utc::now();
        
        // 分析期間の開始・終了日時を決定
        let (start_time, end_time) = match period {
            AnalysisPeriod::Last24Hours => (now - Duration::hours(24), now),
            AnalysisPeriod::LastWeek => (now - Duration::weeks(1), now),
            AnalysisPeriod::LastMonth => (now - Duration::days(30), now),
            AnalysisPeriod::Custom { start, end } => (start, end),
        };
        
        // 期間内のトランザクションを抽出
        let transactions: Vec<Transaction> = self.dag
            .transactions
            .iter()
            .filter(|tx| tx.created_at >= start_time && tx.created_at <= end_time)
            .map(|tx| tx.clone())
            .collect();
        
        // パターン検出（実際の実装では、より高度なアルゴリズムを使用）
        let mut patterns = Vec::new();
        
        // パターン1: 高頻度の小額送金
        let small_transfers: Vec<String> = transactions
            .iter()
            .filter(|tx| {
                !tx.payload.is_empty() && tx.payload[0] <= 50 && tx.payload.len() < 100
            })
            .map(|tx| tx.id.clone())
            .collect();
        
        if small_transfers.len() > 5 {
            patterns.push(TransactionPattern {
                id: "pattern_small_transfers".to_string(),
                name: "高頻度の小額送金".to_string(),
                description: "短時間に多数の小額送金が行われるパターン".to_string(),
                occurrences: small_transfers.len(),
                related_transactions: small_transfers,
            });
        }
        
        // パターン2: トークンスワップチェーン
        let token_swaps: Vec<String> = transactions
            .iter()
            .filter(|tx| {
                !tx.payload.is_empty() && tx.payload[0] > 100 && tx.payload[0] <= 150
            })
            .map(|tx| tx.id.clone())
            .collect();
        
        if token_swaps.len() > 3 {
            patterns.push(TransactionPattern {
                id: "pattern_token_swaps".to_string(),
                name: "トークンスワップチェーン".to_string(),
                description: "複数のトークンスワップが連続して行われるパターン".to_string(),
                occurrences: token_swaps.len(),
                related_transactions: token_swaps,
            });
        }
        
        // パターン3: 流動性提供
        let liquidity_ops: Vec<String> = transactions
            .iter()
            .filter(|tx| {
                !tx.payload.is_empty() && tx.payload[0] > 150 && tx.payload[0] <= 200
            })
            .map(|tx| tx.id.clone())
            .collect();
        
        if liquidity_ops.len() > 2 {
            patterns.push(TransactionPattern {
                id: "pattern_liquidity".to_string(),
                name: "流動性提供操作".to_string(),
                description: "DEXの流動性プールに対する操作パターン".to_string(),
                occurrences: liquidity_ops.len(),
                related_transactions: liquidity_ops,
            });
        }
        
        patterns
    }
    
    /// トランザクションの関連性を分析
    pub fn analyze_transaction_relationships(&self, tx_id: &str) -> Option<TransactionRelationships> {
        // トランザクションを取得
        let tx = self.dag.get_transaction(tx_id)?;
        
        // 親トランザクション
        let parents: Vec<Transaction> = tx.parent_ids
            .iter()
            .filter_map(|id| self.dag.get_transaction(id))
            .collect();
        
        // 子トランザクション
        let children: Vec<Transaction> = if let Some(children_ids) = self.dag.children.get(tx_id) {
            children_ids
                .iter()
                .filter_map(|id| self.dag.get_transaction(id))
                .collect()
        } else {
            Vec::new()
        };
        
        // 兄弟トランザクション（同じ親を持つ他のトランザクション）
        let mut siblings = HashSet::new();
        for parent_id in &tx.parent_ids {
            if let Some(parent_children) = self.dag.children.get(parent_id) {
                for child_id in parent_children {
                    if child_id != tx_id {
                        siblings.insert(child_id.clone());
                    }
                }
            }
        }
        
        let siblings: Vec<Transaction> = siblings
            .iter()
            .filter_map(|id| self.dag.get_transaction(id))
            .collect();
        
        Some(TransactionRelationships {
            transaction: tx,
            parents,
            children,
            siblings,
        })
    }
}

/// トランザクションの関連性
#[derive(Debug, Clone)]
pub struct TransactionRelationships {
    /// 対象トランザクション
    pub transaction: Transaction,
    /// 親トランザクション
    pub parents: Vec<Transaction>,
    /// 子トランザクション
    pub children: Vec<Transaction>,
    /// 兄弟トランザクション（同じ親を持つ他のトランザクション）
    pub siblings: Vec<Transaction>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;
    
    fn create_test_transaction(id: &str, parent_ids: Vec<String>, payload: Vec<u8>) -> Transaction {
        Transaction {
            id: id.to_string(),
            parent_ids,
            timestamp: 12345,
            payload,
            signature: vec![1, 2, 3],
            status: TransactionStatus::Confirmed,
            created_at: Utc::now(),
        }
    }
    
    #[test]
    fn test_transaction_analyzer() {
        // DAGを作成
        let dag = Arc::new(DAG::new());
        
        // テスト用のトランザクションを追加
        let tx1 = create_test_transaction("tx1", vec![], vec![10, 20, 30]);
        let tx2 = create_test_transaction("tx2", vec!["tx1".to_string()], vec![40, 50, 60]);
        let tx3 = create_test_transaction("tx3", vec!["tx1".to_string()], vec![70, 80, 90]);
        let tx4 = create_test_transaction("tx4", vec!["tx2".to_string(), "tx3".to_string()], vec![100, 110, 120]);
        
        dag.add_transaction(tx1).unwrap();
        dag.add_transaction(tx2).unwrap();
        dag.add_transaction(tx3).unwrap();
        dag.add_transaction(tx4).unwrap();
        
        // アナライザーを作成
        let analyzer = TransactionAnalyzer::new(dag.clone());
        
        // 分析を実行
        let analysis = analyzer.analyze(AnalysisPeriod::Last24Hours);
        
        // 基本的な検証
        assert_eq!(analysis.total_transactions, 4);
        assert_eq!(analysis.confirmed_transactions, 4);
        
        // パターン検出
        let patterns = analyzer.detect_patterns(AnalysisPeriod::Last24Hours);
        assert!(!patterns.is_empty());
        
        // 関連性分析
        let relationships = analyzer.analyze_transaction_relationships("tx2");
        assert!(relationships.is_some());
        let relationships = relationships.unwrap();
        assert_eq!(relationships.parents.len(), 1);
        assert_eq!(relationships.parents[0].id, "tx1");
        assert_eq!(relationships.children.len(), 1);
        assert_eq!(relationships.children[0].id, "tx4");
        assert_eq!(relationships.siblings.len(), 1);
        assert_eq!(relationships.siblings[0].id, "tx3");
    }
}