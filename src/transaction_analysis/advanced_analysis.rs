use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use super::detailed_analysis::{
    AddressRelationship, AnomalyDetectionResult, AnomalyType, DetailedTransactionAnalyzer,
    TransactionAnalysisResult, TransactionPattern,
};
use crate::chart::{ChartDataAggregator, DataPoint, TimeFrame};
use crate::error::Error;
use crate::shard::ShardId;
use crate::transaction::{Transaction, TransactionStatus};

/// 高度なトランザクション分析結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedTransactionAnalysisResult {
    /// 基本分析結果
    pub basic_analysis: TransactionAnalysisResult,
    /// トランザクションパターン
    pub patterns: Vec<TransactionPattern>,
    /// 異常検出結果
    pub anomalies: Vec<AnomalyDetectionResult>,
    /// アドレス間の関係
    pub relationships: Vec<AddressRelationship>,
    /// ネットワーク指標
    pub network_metrics: NetworkMetrics,
    /// 時系列予測
    pub time_series_predictions: Option<TimeSeriesPredictions>,
    /// クラスタリング結果
    pub clustering_results: Option<ClusteringResults>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// ネットワーク指標
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// ネットワーク密度
    pub network_density: f64,
    /// 平均クラスタリング係数
    pub average_clustering_coefficient: f64,
    /// 平均パス長
    pub average_path_length: f64,
    /// 中心性指標
    pub centrality_measures: HashMap<String, CentralityMeasures>,
    /// コミュニティ数
    pub community_count: usize,
    /// 最大コミュニティサイズ
    pub max_community_size: usize,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 中心性指標
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityMeasures {
    /// 次数中心性
    pub degree_centrality: f64,
    /// 近接中心性
    pub closeness_centrality: f64,
    /// 媒介中心性
    pub betweenness_centrality: f64,
    /// 固有ベクトル中心性
    pub eigenvector_centrality: f64,
}

/// 時系列予測
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPredictions {
    /// 予測期間の開始
    pub start_time: DateTime<Utc>,
    /// 予測期間の終了
    pub end_time: DateTime<Utc>,
    /// 予測トランザクション数
    pub predicted_transactions: Vec<DataPoint>,
    /// 予測取引量
    pub predicted_volume: Vec<DataPoint>,
    /// 予測手数料
    pub predicted_fees: Vec<DataPoint>,
    /// 予測精度
    pub prediction_accuracy: f64,
    /// 信頼区間
    pub confidence_intervals: Option<Vec<(DataPoint, DataPoint)>>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// クラスタリング結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusteringResults {
    /// クラスタ数
    pub cluster_count: usize,
    /// アドレスクラスタ
    pub address_clusters: HashMap<String, usize>,
    /// クラスタサイズ
    pub cluster_sizes: Vec<usize>,
    /// クラスタ間トランザクション数
    pub inter_cluster_transactions: HashMap<(usize, usize), usize>,
    /// クラスタ内トランザクション数
    pub intra_cluster_transactions: HashMap<usize, usize>,
    /// シルエットスコア
    pub silhouette_score: f64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 高度なトランザクション分析器
pub struct AdvancedTransactionAnalyzer {
    /// 基本分析器
    detailed_analyzer: DetailedTransactionAnalyzer,
    /// 分析対象のトランザクション
    transactions: Vec<Transaction>,
    /// 分析対象期間の開始
    start_time: DateTime<Utc>,
    /// 分析対象期間の終了
    end_time: DateTime<Utc>,
    /// 時間枠
    time_frame: TimeFrame,
    /// 上位項目の表示数
    top_count: usize,
    /// 異常検出の閾値
    anomaly_threshold: f64,
    /// ネットワーク分析の有効化
    enable_network_analysis: bool,
    /// 時系列予測の有効化
    enable_time_series_prediction: bool,
    /// クラスタリングの有効化
    enable_clustering: bool,
}

impl AdvancedTransactionAnalyzer {
    /// 新しい高度なトランザクション分析器を作成
    pub fn new(
        transactions: Vec<Transaction>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        time_frame: TimeFrame,
        top_count: usize,
        anomaly_threshold: f64,
        enable_network_analysis: bool,
        enable_time_series_prediction: bool,
        enable_clustering: bool,
    ) -> Self {
        let detailed_analyzer = DetailedTransactionAnalyzer::new(
            transactions.clone(),
            start_time,
            end_time,
            time_frame,
            top_count,
            anomaly_threshold,
        );

        Self {
            detailed_analyzer,
            transactions,
            start_time,
            end_time,
            time_frame,
            top_count,
            anomaly_threshold,
            enable_network_analysis,
            enable_time_series_prediction,
            enable_clustering,
        }
    }

    /// 高度な分析を実行
    pub fn analyze(&self) -> Result<AdvancedTransactionAnalysisResult, Error> {
        // 基本分析を実行
        let basic_analysis = self.detailed_analyzer.analyze()?;

        // トランザクションパターンを検出
        let patterns = self.detailed_analyzer.detect_patterns();

        // 異常を検出
        let anomalies = self.detailed_analyzer.detect_anomalies();

        // アドレス間の関係を分析
        let relationships = self.detailed_analyzer.analyze_address_relationships();

        // ネットワーク指標を計算
        let network_metrics = if self.enable_network_analysis {
            self.calculate_network_metrics(&relationships)
        } else {
            NetworkMetrics {
                network_density: 0.0,
                average_clustering_coefficient: 0.0,
                average_path_length: 0.0,
                centrality_measures: HashMap::new(),
                community_count: 0,
                max_community_size: 0,
                metadata: None,
            }
        };

        // 時系列予測を実行
        let time_series_predictions = if self.enable_time_series_prediction {
            Some(self.predict_time_series(&basic_analysis))
        } else {
            None
        };

        // クラスタリングを実行
        let clustering_results = if self.enable_clustering {
            Some(self.perform_clustering(&relationships))
        } else {
            None
        };

        // 高度な分析結果を作成
        let result = AdvancedTransactionAnalysisResult {
            basic_analysis,
            patterns,
            anomalies,
            relationships,
            network_metrics,
            time_series_predictions,
            clustering_results,
            metadata: None,
        };

        Ok(result)
    }

    /// ネットワーク指標を計算
    fn calculate_network_metrics(&self, relationships: &[AddressRelationship]) -> NetworkMetrics {
        // アドレスの集合を作成
        let mut addresses = HashSet::new();
        for rel in relationships {
            addresses.insert(rel.sender.clone());
            addresses.insert(rel.receiver.clone());
        }

        let address_count = addresses.len();
        if address_count == 0 || relationships.is_empty() {
            return NetworkMetrics {
                network_density: 0.0,
                average_clustering_coefficient: 0.0,
                average_path_length: 0.0,
                centrality_measures: HashMap::new(),
                community_count: 0,
                max_community_size: 0,
                metadata: None,
            };
        }

        // ネットワーク密度を計算
        let max_edges = address_count * (address_count - 1);
        let actual_edges = relationships.len();
        let network_density = if max_edges > 0 {
            actual_edges as f64 / max_edges as f64
        } else {
            0.0
        };

        // 隣接リストを作成
        let mut adjacency_list: HashMap<String, Vec<String>> = HashMap::new();
        for rel in relationships {
            let neighbors = adjacency_list
                .entry(rel.sender.clone())
                .or_insert_with(Vec::new);
            neighbors.push(rel.receiver.clone());
        }

        // 中心性指標を計算
        let mut centrality_measures = HashMap::new();
        for address in &addresses {
            // 次数中心性
            let degree = adjacency_list
                .get(address)
                .map_or(0, |neighbors| neighbors.len());
            let degree_centrality = if address_count > 1 {
                degree as f64 / (address_count - 1) as f64
            } else {
                0.0
            };

            // 簡易的な中心性指標を計算
            let measures = CentralityMeasures {
                degree_centrality,
                closeness_centrality: 0.0,   // 簡易実装では計算省略
                betweenness_centrality: 0.0, // 簡易実装では計算省略
                eigenvector_centrality: 0.0, // 簡易実装では計算省略
            };

            centrality_measures.insert(address.clone(), measures);
        }

        // コミュニティ検出（簡易実装）
        let community_count = 1;
        let max_community_size = address_count;

        NetworkMetrics {
            network_density,
            average_clustering_coefficient: 0.0, // 簡易実装では計算省略
            average_path_length: 0.0,            // 簡易実装では計算省略
            centrality_measures,
            community_count,
            max_community_size,
            metadata: None,
        }
    }

    /// 時系列予測を実行
    fn predict_time_series(&self, analysis: &TransactionAnalysisResult) -> TimeSeriesPredictions {
        // 予測期間を設定
        let prediction_duration = self.end_time - self.start_time;
        let prediction_start = self.end_time;
        let prediction_end = prediction_start + prediction_duration;

        // 簡易的な予測モデル（移動平均）
        let transactions_by_time = &analysis.transactions_by_time;
        let volume_by_time = &analysis.volume_by_time;
        let fees_by_time = &analysis.fees_by_time;

        // 予測データポイントを生成
        let mut predicted_transactions = Vec::new();
        let mut predicted_volume = Vec::new();
        let mut predicted_fees = Vec::new();

        if !transactions_by_time.is_empty() {
            // 移動平均の窓サイズ
            let window_size = 3.min(transactions_by_time.len());

            // 最後のwindow_size個のデータポイントの平均を計算
            let avg_transactions = transactions_by_time
                .iter()
                .skip(transactions_by_time.len() - window_size)
                .map(|dp| dp.value)
                .sum::<f64>()
                / window_size as f64;

            let avg_volume = volume_by_time
                .iter()
                .skip(volume_by_time.len() - window_size)
                .map(|dp| dp.value)
                .sum::<f64>()
                / window_size as f64;

            let avg_fees = fees_by_time
                .iter()
                .skip(fees_by_time.len() - window_size)
                .map(|dp| dp.value)
                .sum::<f64>()
                / window_size as f64;

            // 予測期間の時間枠を生成
            let frame_seconds = self.time_frame.to_seconds();
            let mut current_time = prediction_start;

            while current_time <= prediction_end {
                // 予測値に小さなランダム変動を加える（実際の実装ではより高度なモデルを使用）
                let random_factor = 0.9 + (current_time.timestamp() % 20) as f64 * 0.01;

                predicted_transactions.push(DataPoint {
                    timestamp: current_time,
                    value: avg_transactions * random_factor,
                    metadata: None,
                });

                predicted_volume.push(DataPoint {
                    timestamp: current_time,
                    value: avg_volume * random_factor,
                    metadata: None,
                });

                predicted_fees.push(DataPoint {
                    timestamp: current_time,
                    value: avg_fees * random_factor,
                    metadata: None,
                });

                current_time = current_time + Duration::seconds(frame_seconds);
            }
        }

        TimeSeriesPredictions {
            start_time: prediction_start,
            end_time: prediction_end,
            predicted_transactions,
            predicted_volume,
            predicted_fees,
            prediction_accuracy: 0.8, // 簡易実装では固定値
            confidence_intervals: None,
            metadata: None,
        }
    }

    /// クラスタリングを実行
    fn perform_clustering(&self, relationships: &[AddressRelationship]) -> ClusteringResults {
        // アドレスの集合を作成
        let mut addresses = HashSet::new();
        for rel in relationships {
            addresses.insert(rel.sender.clone());
            addresses.insert(rel.receiver.clone());
        }

        let address_count = addresses.len();
        if address_count == 0 {
            return ClusteringResults {
                cluster_count: 0,
                address_clusters: HashMap::new(),
                cluster_sizes: Vec::new(),
                inter_cluster_transactions: HashMap::new(),
                intra_cluster_transactions: HashMap::new(),
                silhouette_score: 0.0,
                metadata: None,
            };
        }

        // 簡易的なクラスタリング（実際の実装ではより高度なアルゴリズムを使用）
        // ここでは、強い関係を持つアドレスを同じクラスタにグループ化

        // 関係の強さに基づいてエッジをソート
        let mut sorted_relationships = relationships.to_vec();
        sorted_relationships.sort_by(|a, b| {
            b.strength
                .partial_cmp(&a.strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Union-Findアルゴリズムを使用してクラスタを形成
        let mut address_clusters: HashMap<String, usize> = HashMap::new();
        let mut cluster_id = 0;

        for address in &addresses {
            if !address_clusters.contains_key(address) {
                address_clusters.insert(address.clone(), cluster_id);
                cluster_id += 1;
            }
        }

        // 強い関係を持つアドレスを同じクラスタにマージ
        for rel in sorted_relationships {
            if rel.strength < 0.5 {
                continue; // 弱い関係は無視
            }

            let sender_cluster = *address_clusters.get(&rel.sender).unwrap();
            let receiver_cluster = *address_clusters.get(&rel.receiver).unwrap();

            if sender_cluster != receiver_cluster {
                // 小さいクラスタIDを使用してマージ
                let (from_cluster, to_cluster) = if sender_cluster > receiver_cluster {
                    (sender_cluster, receiver_cluster)
                } else {
                    (receiver_cluster, sender_cluster)
                };

                // クラスタをマージ
                for (addr, cluster) in address_clusters.iter_mut() {
                    if *cluster == from_cluster {
                        *cluster = to_cluster;
                    }
                }
            }
        }

        // クラスタサイズを計算
        let mut cluster_sizes_map: HashMap<usize, usize> = HashMap::new();
        for &cluster in address_clusters.values() {
            *cluster_sizes_map.entry(cluster).or_insert(0) += 1;
        }

        let cluster_count = cluster_sizes_map.len();
        let mut cluster_sizes: Vec<usize> = cluster_sizes_map.values().cloned().collect();
        cluster_sizes.sort_by(|a, b| b.cmp(a));

        let max_community_size = cluster_sizes.first().cloned().unwrap_or(0);

        // クラスタ間・クラスタ内トランザクション数を計算
        let mut inter_cluster_transactions: HashMap<(usize, usize), usize> = HashMap::new();
        let mut intra_cluster_transactions: HashMap<usize, usize> = HashMap::new();

        for rel in relationships {
            let sender_cluster = *address_clusters.get(&rel.sender).unwrap();
            let receiver_cluster = *address_clusters.get(&rel.receiver).unwrap();

            if sender_cluster == receiver_cluster {
                *intra_cluster_transactions
                    .entry(sender_cluster)
                    .or_insert(0) += rel.transaction_count;
            } else {
                let cluster_pair = if sender_cluster < receiver_cluster {
                    (sender_cluster, receiver_cluster)
                } else {
                    (receiver_cluster, sender_cluster)
                };

                *inter_cluster_transactions.entry(cluster_pair).or_insert(0) +=
                    rel.transaction_count;
            }
        }

        ClusteringResults {
            cluster_count,
            address_clusters,
            cluster_sizes,
            inter_cluster_transactions,
            intra_cluster_transactions,
            silhouette_score: 0.7, // 簡易実装では固定値
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_analysis() {
        // テストデータを作成
        let now = Utc::now();
        let start_time = now - Duration::days(7);
        let end_time = now;

        let transactions = vec![
            Transaction {
                id: "tx1".to_string(),
                sender: "addr1".to_string(),
                receiver: "addr2".to_string(),
                amount: 100,
                fee: 10,
                timestamp: (start_time + Duration::hours(1)).timestamp(),
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx2".to_string(),
                sender: "addr2".to_string(),
                receiver: "addr3".to_string(),
                amount: 50,
                fee: 5,
                timestamp: (start_time + Duration::hours(2)).timestamp(),
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx3".to_string(),
                sender: "addr3".to_string(),
                receiver: "addr1".to_string(),
                amount: 25,
                fee: 3,
                timestamp: (start_time + Duration::hours(3)).timestamp(),
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
        ];

        // 高度な分析器を作成
        let analyzer = AdvancedTransactionAnalyzer::new(
            transactions,
            start_time,
            end_time,
            TimeFrame::Hour,
            10,
            3.0,
            true,
            true,
            true,
        );

        // 分析を実行
        let result = analyzer.analyze().unwrap();

        // 基本的な検証
        assert_eq!(result.basic_analysis.total_transactions, 3);
        assert_eq!(result.basic_analysis.successful_transactions, 3);

        // ネットワーク指標の検証
        assert!(result.network_metrics.network_density > 0.0);

        // 時系列予測の検証
        if let Some(predictions) = result.time_series_predictions {
            assert!(!predictions.predicted_transactions.is_empty());
        }

        // クラスタリング結果の検証
        if let Some(clustering) = result.clustering_results {
            assert!(clustering.cluster_count > 0);
        }
    }
}
