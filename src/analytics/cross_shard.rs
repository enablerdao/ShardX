use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::analytics::metrics::{MetricType, MetricValue};
use crate::error::Error;
use crate::shard::{ShardId, ShardInfo};
use crate::transaction::{Transaction, TransactionStatus};

/// クロスシャード分析
pub struct CrossShardAnalyzer {
    /// シャード情報
    shard_info: HashMap<ShardId, ShardInfo>,
    /// クロスシャードトランザクション
    cross_shard_transactions: Vec<CrossShardTransaction>,
    /// シャード間レイテンシ
    shard_latencies: HashMap<(ShardId, ShardId), Vec<ShardLatency>>,
    /// シャード間スループット
    shard_throughputs: HashMap<(ShardId, ShardId), Vec<ShardThroughput>>,
    /// シャード間フロー
    shard_flows: HashMap<(ShardId, ShardId), Vec<ShardFlow>>,
    /// メトリクス
    metrics: HashMap<String, Vec<(DateTime<Utc>, MetricValue)>>,
}

/// クロスシャードトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardTransaction {
    /// トランザクションID
    pub transaction_id: String,
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub target_shard_id: ShardId,
    /// 送信者
    pub sender: String,
    /// 受信者
    pub receiver: String,
    /// 金額
    pub amount: u64,
    /// 手数料
    pub fee: u64,
    /// 開始時刻
    pub start_time: DateTime<Utc>,
    /// 完了時刻
    pub completion_time: Option<DateTime<Utc>>,
    /// ステータス
    pub status: TransactionStatus,
    /// レイテンシ（ミリ秒）
    pub latency_ms: Option<u64>,
    /// ホップ数
    pub hops: Option<u32>,
    /// ルート
    pub route: Option<Vec<ShardId>>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// シャードレイテンシ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardLatency {
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub target_shard_id: ShardId,
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// レイテンシ（ミリ秒）
    pub latency_ms: u64,
    /// サンプル数
    pub sample_count: u32,
    /// 最小レイテンシ（ミリ秒）
    pub min_latency_ms: u64,
    /// 最大レイテンシ（ミリ秒）
    pub max_latency_ms: u64,
    /// 標準偏差
    pub std_dev_ms: f64,
    /// パーセンタイル
    pub percentiles: HashMap<u32, u64>,
}

/// シャードスループット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardThroughput {
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub target_shard_id: ShardId,
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 期間（秒）
    pub period_seconds: u64,
    /// トランザクション数
    pub transaction_count: u64,
    /// トランザクション量
    pub transaction_volume: u64,
    /// 1秒あたりのトランザクション数
    pub tps: f64,
    /// 1秒あたりのバイト数
    pub bytes_per_second: f64,
    /// 成功率
    pub success_rate: f64,
}

/// シャードフロー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardFlow {
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub target_shard_id: ShardId,
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 期間（秒）
    pub period_seconds: u64,
    /// 送信トランザクション数
    pub outgoing_transactions: u64,
    /// 受信トランザクション数
    pub incoming_transactions: u64,
    /// 送信トランザクション量
    pub outgoing_volume: u64,
    /// 受信トランザクション量
    pub incoming_volume: u64,
    /// 純フロー（トランザクション数）
    pub net_transaction_flow: i64,
    /// 純フロー（トランザクション量）
    pub net_volume_flow: i64,
}

impl CrossShardAnalyzer {
    /// 新しいクロスシャード分析器を作成
    pub fn new() -> Self {
        Self {
            shard_info: HashMap::new(),
            cross_shard_transactions: Vec::new(),
            shard_latencies: HashMap::new(),
            shard_throughputs: HashMap::new(),
            shard_flows: HashMap::new(),
            metrics: HashMap::new(),
        }
    }

    /// シャード情報を追加
    pub fn add_shard_info(&mut self, shard_id: ShardId, info: ShardInfo) {
        self.shard_info.insert(shard_id, info);
    }

    /// クロスシャードトランザクションを追加
    pub fn add_cross_shard_transaction(&mut self, transaction: CrossShardTransaction) {
        self.cross_shard_transactions.push(transaction);
    }

    /// トランザクションからクロスシャードトランザクションを作成
    pub fn create_cross_shard_transaction(
        &self,
        transaction: &Transaction,
        source_shard_id: ShardId,
        target_shard_id: ShardId,
    ) -> CrossShardTransaction {
        CrossShardTransaction {
            transaction_id: transaction.id.clone(),
            source_shard_id,
            target_shard_id,
            sender: transaction.sender.clone(),
            receiver: transaction.receiver.clone(),
            amount: transaction.amount,
            fee: transaction.fee,
            start_time: transaction.timestamp,
            completion_time: None,
            status: transaction.status.clone(),
            latency_ms: None,
            hops: None,
            route: None,
            metadata: None,
        }
    }

    /// クロスシャードトランザクションを更新
    pub fn update_cross_shard_transaction(
        &mut self,
        transaction_id: &str,
        status: TransactionStatus,
        completion_time: Option<DateTime<Utc>>,
        latency_ms: Option<u64>,
    ) -> Result<(), Error> {
        let transaction = self
            .cross_shard_transactions
            .iter_mut()
            .find(|tx| tx.transaction_id == transaction_id)
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Cross-shard transaction not found: {}",
                    transaction_id
                ))
            })?;

        transaction.status = status;
        transaction.completion_time = completion_time;
        transaction.latency_ms = latency_ms;

        Ok(())
    }

    /// シャードレイテンシを追加
    pub fn add_shard_latency(&mut self, latency: ShardLatency) {
        let key = (
            latency.source_shard_id.clone(),
            latency.target_shard_id.clone(),
        );
        let latencies = self.shard_latencies.entry(key).or_insert_with(Vec::new);
        latencies.push(latency);
    }

    /// シャードスループットを追加
    pub fn add_shard_throughput(&mut self, throughput: ShardThroughput) {
        let key = (
            throughput.source_shard_id.clone(),
            throughput.target_shard_id.clone(),
        );
        let throughputs = self.shard_throughputs.entry(key).or_insert_with(Vec::new);
        throughputs.push(throughput);
    }

    /// シャードフローを追加
    pub fn add_shard_flow(&mut self, flow: ShardFlow) {
        let key = (flow.source_shard_id.clone(), flow.target_shard_id.clone());
        let flows = self.shard_flows.entry(key).or_insert_with(Vec::new);
        flows.push(flow);
    }

    /// メトリクスを追加
    pub fn add_metric(&mut self, name: &str, timestamp: DateTime<Utc>, value: MetricValue) {
        let metrics = self
            .metrics
            .entry(name.to_string())
            .or_insert_with(Vec::new);
        metrics.push((timestamp, value));
    }

    /// クロスシャードトランザクションを分析
    pub fn analyze_transactions(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> CrossShardAnalysisResult {
        // 期間内のトランザクションをフィルタリング
        let transactions: Vec<_> = self
            .cross_shard_transactions
            .iter()
            .filter(|tx| tx.start_time >= start_time && tx.start_time <= end_time)
            .collect();

        // トランザクション数
        let transaction_count = transactions.len();

        // 成功したトランザクション数
        let successful_transactions = transactions
            .iter()
            .filter(|tx| tx.status == TransactionStatus::Confirmed)
            .count();

        // 失敗したトランザクション数
        let failed_transactions = transactions
            .iter()
            .filter(|tx| tx.status == TransactionStatus::Failed)
            .count();

        // 保留中のトランザクション数
        let pending_transactions = transactions
            .iter()
            .filter(|tx| tx.status == TransactionStatus::Pending)
            .count();

        // 平均レイテンシ
        let latencies: Vec<_> = transactions.iter().filter_map(|tx| tx.latency_ms).collect();

        let average_latency = if !latencies.is_empty() {
            latencies.iter().sum::<u64>() as f64 / latencies.len() as f64
        } else {
            0.0
        };

        // シャードペアごとのトランザクション数
        let mut transactions_by_shard_pair: HashMap<(ShardId, ShardId), u64> = HashMap::new();

        for tx in &transactions {
            let key = (tx.source_shard_id.clone(), tx.target_shard_id.clone());
            *transactions_by_shard_pair.entry(key).or_insert(0) += 1;
        }

        // シャードペアごとの平均レイテンシ
        let mut latency_by_shard_pair: HashMap<(ShardId, ShardId), f64> = HashMap::new();

        for tx in &transactions {
            if let Some(latency) = tx.latency_ms {
                let key = (tx.source_shard_id.clone(), tx.target_shard_id.clone());
                let entry = latency_by_shard_pair.entry(key).or_insert(0.0);
                *entry = (*entry * 0.9) + (latency as f64 * 0.1); // 指数移動平均
            }
        }

        // シャードペアごとのスループット
        let mut throughput_by_shard_pair: HashMap<(ShardId, ShardId), f64> = HashMap::new();

        let period_seconds = (end_time - start_time).num_seconds() as f64;

        for ((source, target), count) in &transactions_by_shard_pair {
            let tps = *count as f64 / period_seconds;
            throughput_by_shard_pair.insert((source.clone(), target.clone()), tps);
        }

        // 結果を作成
        CrossShardAnalysisResult {
            start_time,
            end_time,
            transaction_count,
            successful_transactions,
            failed_transactions,
            pending_transactions,
            average_latency,
            transactions_by_shard_pair,
            latency_by_shard_pair,
            throughput_by_shard_pair,
        }
    }

    /// シャード間レイテンシを分析
    pub fn analyze_latencies(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> HashMap<(ShardId, ShardId), ShardLatencyAnalysis> {
        let mut result = HashMap::new();

        for ((source, target), latencies) in &self.shard_latencies {
            // 期間内のレイテンシをフィルタリング
            let filtered_latencies: Vec<_> = latencies
                .iter()
                .filter(|l| l.timestamp >= start_time && l.timestamp <= end_time)
                .collect();

            if filtered_latencies.is_empty() {
                continue;
            }

            // 平均レイテンシ
            let average_latency = filtered_latencies.iter().map(|l| l.latency_ms).sum::<u64>()
                as f64
                / filtered_latencies.len() as f64;

            // 最小レイテンシ
            let min_latency = filtered_latencies
                .iter()
                .map(|l| l.min_latency_ms)
                .min()
                .unwrap_or(0);

            // 最大レイテンシ
            let max_latency = filtered_latencies
                .iter()
                .map(|l| l.max_latency_ms)
                .max()
                .unwrap_or(0);

            // 標準偏差
            let std_dev = filtered_latencies.iter().map(|l| l.std_dev_ms).sum::<f64>()
                / filtered_latencies.len() as f64;

            // パーセンタイル
            let mut percentiles = HashMap::new();

            if !filtered_latencies.is_empty() {
                let last_latency = filtered_latencies.last().unwrap();
                percentiles = last_latency.percentiles.clone();
            }

            // 分析結果を作成
            let analysis = ShardLatencyAnalysis {
                source_shard_id: source.clone(),
                target_shard_id: target.clone(),
                average_latency,
                min_latency,
                max_latency,
                std_dev,
                percentiles,
                sample_count: filtered_latencies.len() as u32,
            };

            result.insert((source.clone(), target.clone()), analysis);
        }

        result
    }

    /// シャード間スループットを分析
    pub fn analyze_throughputs(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> HashMap<(ShardId, ShardId), ShardThroughputAnalysis> {
        let mut result = HashMap::new();

        for ((source, target), throughputs) in &self.shard_throughputs {
            // 期間内のスループットをフィルタリング
            let filtered_throughputs: Vec<_> = throughputs
                .iter()
                .filter(|t| t.timestamp >= start_time && t.timestamp <= end_time)
                .collect();

            if filtered_throughputs.is_empty() {
                continue;
            }

            // 平均TPS
            let average_tps = filtered_throughputs.iter().map(|t| t.tps).sum::<f64>()
                / filtered_throughputs.len() as f64;

            // 最大TPS
            let max_tps = filtered_throughputs
                .iter()
                .map(|t| t.tps)
                .fold(0.0, f64::max);

            // 平均バイト/秒
            let average_bytes_per_second = filtered_throughputs
                .iter()
                .map(|t| t.bytes_per_second)
                .sum::<f64>()
                / filtered_throughputs.len() as f64;

            // 平均成功率
            let average_success_rate = filtered_throughputs
                .iter()
                .map(|t| t.success_rate)
                .sum::<f64>()
                / filtered_throughputs.len() as f64;

            // 総トランザクション数
            let total_transactions = filtered_throughputs
                .iter()
                .map(|t| t.transaction_count)
                .sum::<u64>();

            // 総トランザクション量
            let total_volume = filtered_throughputs
                .iter()
                .map(|t| t.transaction_volume)
                .sum::<u64>();

            // 分析結果を作成
            let analysis = ShardThroughputAnalysis {
                source_shard_id: source.clone(),
                target_shard_id: target.clone(),
                average_tps,
                max_tps,
                average_bytes_per_second,
                average_success_rate,
                total_transactions,
                total_volume,
                sample_count: filtered_throughputs.len() as u32,
            };

            result.insert((source.clone(), target.clone()), analysis);
        }

        result
    }

    /// シャード間フローを分析
    pub fn analyze_flows(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> HashMap<(ShardId, ShardId), ShardFlowAnalysis> {
        let mut result = HashMap::new();

        for ((source, target), flows) in &self.shard_flows {
            // 期間内のフローをフィルタリング
            let filtered_flows: Vec<_> = flows
                .iter()
                .filter(|f| f.timestamp >= start_time && f.timestamp <= end_time)
                .collect();

            if filtered_flows.is_empty() {
                continue;
            }

            // 総送信トランザクション数
            let total_outgoing_transactions = filtered_flows
                .iter()
                .map(|f| f.outgoing_transactions)
                .sum::<u64>();

            // 総受信トランザクション数
            let total_incoming_transactions = filtered_flows
                .iter()
                .map(|f| f.incoming_transactions)
                .sum::<u64>();

            // 総送信トランザクション量
            let total_outgoing_volume = filtered_flows
                .iter()
                .map(|f| f.outgoing_volume)
                .sum::<u64>();

            // 総受信トランザクション量
            let total_incoming_volume = filtered_flows
                .iter()
                .map(|f| f.incoming_volume)
                .sum::<u64>();

            // 純フロー（トランザクション数）
            let net_transaction_flow =
                total_outgoing_transactions as i64 - total_incoming_transactions as i64;

            // 純フロー（トランザクション量）
            let net_volume_flow = total_outgoing_volume as i64 - total_incoming_volume as i64;

            // フロー比率（トランザクション数）
            let transaction_flow_ratio = if total_incoming_transactions > 0 {
                total_outgoing_transactions as f64 / total_incoming_transactions as f64
            } else {
                f64::INFINITY
            };

            // フロー比率（トランザクション量）
            let volume_flow_ratio = if total_incoming_volume > 0 {
                total_outgoing_volume as f64 / total_incoming_volume as f64
            } else {
                f64::INFINITY
            };

            // 分析結果を作成
            let analysis = ShardFlowAnalysis {
                source_shard_id: source.clone(),
                target_shard_id: target.clone(),
                total_outgoing_transactions,
                total_incoming_transactions,
                total_outgoing_volume,
                total_incoming_volume,
                net_transaction_flow,
                net_volume_flow,
                transaction_flow_ratio,
                volume_flow_ratio,
                sample_count: filtered_flows.len() as u32,
            };

            result.insert((source.clone(), target.clone()), analysis);
        }

        result
    }

    /// クロスシャードネットワークを分析
    pub fn analyze_network(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> CrossShardNetworkAnalysis {
        // トランザクション分析
        let transaction_analysis = self.analyze_transactions(start_time, end_time);

        // レイテンシ分析
        let latency_analysis = self.analyze_latencies(start_time, end_time);

        // スループット分析
        let throughput_analysis = self.analyze_throughputs(start_time, end_time);

        // フロー分析
        let flow_analysis = self.analyze_flows(start_time, end_time);

        // シャードノードを作成
        let mut nodes = HashMap::new();

        for (shard_id, info) in &self.shard_info {
            nodes.insert(
                shard_id.clone(),
                ShardNode {
                    shard_id: shard_id.clone(),
                    name: info.name.clone(),
                    transaction_count: 0,
                    outgoing_transactions: 0,
                    incoming_transactions: 0,
                    outgoing_volume: 0,
                    incoming_volume: 0,
                    average_latency: 0.0,
                    success_rate: 0.0,
                },
            );
        }

        // シャードエッジを作成
        let mut edges = HashMap::new();

        for ((source, target), count) in &transaction_analysis.transactions_by_shard_pair {
            let latency = latency_analysis
                .get(&(source.clone(), target.clone()))
                .map(|l| l.average_latency)
                .unwrap_or(0.0);

            let throughput = throughput_analysis
                .get(&(source.clone(), target.clone()))
                .map(|t| t.average_tps)
                .unwrap_or(0.0);

            let success_rate = throughput_analysis
                .get(&(source.clone(), target.clone()))
                .map(|t| t.average_success_rate)
                .unwrap_or(0.0);

            let flow = flow_analysis.get(&(source.clone(), target.clone()));

            let edge = ShardEdge {
                source_shard_id: source.clone(),
                target_shard_id: target.clone(),
                transaction_count: *count,
                average_latency: latency,
                throughput,
                success_rate,
                outgoing_transactions: flow.map(|f| f.total_outgoing_transactions).unwrap_or(0),
                incoming_transactions: flow.map(|f| f.total_incoming_transactions).unwrap_or(0),
                outgoing_volume: flow.map(|f| f.total_outgoing_volume).unwrap_or(0),
                incoming_volume: flow.map(|f| f.total_incoming_volume).unwrap_or(0),
            };

            edges.insert((source.clone(), target.clone()), edge);

            // ノード情報を更新
            if let Some(node) = nodes.get_mut(source) {
                node.transaction_count += *count;
                node.outgoing_transactions +=
                    flow.map(|f| f.total_outgoing_transactions).unwrap_or(0);
                node.outgoing_volume += flow.map(|f| f.total_outgoing_volume).unwrap_or(0);
            }

            if let Some(node) = nodes.get_mut(target) {
                node.transaction_count += *count;
                node.incoming_transactions +=
                    flow.map(|f| f.total_incoming_transactions).unwrap_or(0);
                node.incoming_volume += flow.map(|f| f.total_incoming_volume).unwrap_or(0);
            }
        }

        // 平均レイテンシと成功率を計算
        for node in nodes.values_mut() {
            let mut total_latency = 0.0;
            let mut latency_count = 0;
            let mut total_success_rate = 0.0;
            let mut success_rate_count = 0;

            for ((source, _), edge) in &edges {
                if source == &node.shard_id {
                    total_latency += edge.average_latency;
                    latency_count += 1;

                    total_success_rate += edge.success_rate;
                    success_rate_count += 1;
                }
            }

            if latency_count > 0 {
                node.average_latency = total_latency / latency_count as f64;
            }

            if success_rate_count > 0 {
                node.success_rate = total_success_rate / success_rate_count as f64;
            }
        }

        // ネットワーク分析結果を作成
        CrossShardNetworkAnalysis {
            start_time,
            end_time,
            nodes: nodes.values().cloned().collect(),
            edges: edges.values().cloned().collect(),
            transaction_count: transaction_analysis.transaction_count,
            successful_transactions: transaction_analysis.successful_transactions,
            failed_transactions: transaction_analysis.failed_transactions,
            pending_transactions: transaction_analysis.pending_transactions,
            average_latency: transaction_analysis.average_latency,
        }
    }

    /// クロスシャードトランザクションを取得
    pub fn get_cross_shard_transactions(&self) -> &[CrossShardTransaction] {
        &self.cross_shard_transactions
    }

    /// シャード情報を取得
    pub fn get_shard_info(&self, shard_id: &ShardId) -> Option<&ShardInfo> {
        self.shard_info.get(shard_id)
    }

    /// シャードレイテンシを取得
    pub fn get_shard_latencies(
        &self,
        source_shard_id: &ShardId,
        target_shard_id: &ShardId,
    ) -> Option<&[ShardLatency]> {
        self.shard_latencies
            .get(&(source_shard_id.clone(), target_shard_id.clone()))
            .map(|latencies| latencies.as_slice())
    }

    /// シャードスループットを取得
    pub fn get_shard_throughputs(
        &self,
        source_shard_id: &ShardId,
        target_shard_id: &ShardId,
    ) -> Option<&[ShardThroughput]> {
        self.shard_throughputs
            .get(&(source_shard_id.clone(), target_shard_id.clone()))
            .map(|throughputs| throughputs.as_slice())
    }

    /// シャードフローを取得
    pub fn get_shard_flows(
        &self,
        source_shard_id: &ShardId,
        target_shard_id: &ShardId,
    ) -> Option<&[ShardFlow]> {
        self.shard_flows
            .get(&(source_shard_id.clone(), target_shard_id.clone()))
            .map(|flows| flows.as_slice())
    }

    /// メトリクスを取得
    pub fn get_metrics(&self, name: &str) -> Option<&[(DateTime<Utc>, MetricValue)]> {
        self.metrics.get(name).map(|metrics| metrics.as_slice())
    }
}

/// クロスシャード分析結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardAnalysisResult {
    /// 開始時刻
    pub start_time: DateTime<Utc>,
    /// 終了時刻
    pub end_time: DateTime<Utc>,
    /// トランザクション数
    pub transaction_count: usize,
    /// 成功したトランザクション数
    pub successful_transactions: usize,
    /// 失敗したトランザクション数
    pub failed_transactions: usize,
    /// 保留中のトランザクション数
    pub pending_transactions: usize,
    /// 平均レイテンシ
    pub average_latency: f64,
    /// シャードペアごとのトランザクション数
    pub transactions_by_shard_pair: HashMap<(ShardId, ShardId), u64>,
    /// シャードペアごとの平均レイテンシ
    pub latency_by_shard_pair: HashMap<(ShardId, ShardId), f64>,
    /// シャードペアごとのスループット
    pub throughput_by_shard_pair: HashMap<(ShardId, ShardId), f64>,
}

/// シャードレイテンシ分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardLatencyAnalysis {
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub target_shard_id: ShardId,
    /// 平均レイテンシ
    pub average_latency: f64,
    /// 最小レイテンシ
    pub min_latency: u64,
    /// 最大レイテンシ
    pub max_latency: u64,
    /// 標準偏差
    pub std_dev: f64,
    /// パーセンタイル
    pub percentiles: HashMap<u32, u64>,
    /// サンプル数
    pub sample_count: u32,
}

/// シャードスループット分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardThroughputAnalysis {
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub target_shard_id: ShardId,
    /// 平均TPS
    pub average_tps: f64,
    /// 最大TPS
    pub max_tps: f64,
    /// 平均バイト/秒
    pub average_bytes_per_second: f64,
    /// 平均成功率
    pub average_success_rate: f64,
    /// 総トランザクション数
    pub total_transactions: u64,
    /// 総トランザクション量
    pub total_volume: u64,
    /// サンプル数
    pub sample_count: u32,
}

/// シャードフロー分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardFlowAnalysis {
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub target_shard_id: ShardId,
    /// 総送信トランザクション数
    pub total_outgoing_transactions: u64,
    /// 総受信トランザクション数
    pub total_incoming_transactions: u64,
    /// 総送信トランザクション量
    pub total_outgoing_volume: u64,
    /// 総受信トランザクション量
    pub total_incoming_volume: u64,
    /// 純フロー（トランザクション数）
    pub net_transaction_flow: i64,
    /// 純フロー（トランザクション量）
    pub net_volume_flow: i64,
    /// フロー比率（トランザクション数）
    pub transaction_flow_ratio: f64,
    /// フロー比率（トランザクション量）
    pub volume_flow_ratio: f64,
    /// サンプル数
    pub sample_count: u32,
}

/// クロスシャードネットワーク分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardNetworkAnalysis {
    /// 開始時刻
    pub start_time: DateTime<Utc>,
    /// 終了時刻
    pub end_time: DateTime<Utc>,
    /// シャードノード
    pub nodes: Vec<ShardNode>,
    /// シャードエッジ
    pub edges: Vec<ShardEdge>,
    /// トランザクション数
    pub transaction_count: usize,
    /// 成功したトランザクション数
    pub successful_transactions: usize,
    /// 失敗したトランザクション数
    pub failed_transactions: usize,
    /// 保留中のトランザクション数
    pub pending_transactions: usize,
    /// 平均レイテンシ
    pub average_latency: f64,
}

/// シャードノード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardNode {
    /// シャードID
    pub shard_id: ShardId,
    /// 名前
    pub name: String,
    /// トランザクション数
    pub transaction_count: u64,
    /// 送信トランザクション数
    pub outgoing_transactions: u64,
    /// 受信トランザクション数
    pub incoming_transactions: u64,
    /// 送信トランザクション量
    pub outgoing_volume: u64,
    /// 受信トランザクション量
    pub incoming_volume: u64,
    /// 平均レイテンシ
    pub average_latency: f64,
    /// 成功率
    pub success_rate: f64,
}

/// シャードエッジ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardEdge {
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub target_shard_id: ShardId,
    /// トランザクション数
    pub transaction_count: u64,
    /// 平均レイテンシ
    pub average_latency: f64,
    /// スループット
    pub throughput: f64,
    /// 成功率
    pub success_rate: f64,
    /// 送信トランザクション数
    pub outgoing_transactions: u64,
    /// 受信トランザクション数
    pub incoming_transactions: u64,
    /// 送信トランザクション量
    pub outgoing_volume: u64,
    /// 受信トランザクション量
    pub incoming_volume: u64,
}
