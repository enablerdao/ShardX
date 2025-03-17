use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::chart::{ChartDataAggregator, DataPoint, TimeFrame};
use crate::error::Error;
use crate::shard::ShardId;
use crate::transaction::{Transaction, TransactionStatus};

/// トランザクション分析結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalysisResult {
    /// 分析ID
    pub id: String,
    /// 分析名
    pub name: String,
    /// 分析対象期間の開始
    pub start_time: DateTime<Utc>,
    /// 分析対象期間の終了
    pub end_time: DateTime<Utc>,
    /// 総トランザクション数
    pub total_transactions: usize,
    /// 成功したトランザクション数
    pub successful_transactions: usize,
    /// 失敗したトランザクション数
    pub failed_transactions: usize,
    /// 保留中のトランザクション数
    pub pending_transactions: usize,
    /// 総取引量
    pub total_volume: f64,
    /// 平均取引量
    pub average_volume: f64,
    /// 最大取引量
    pub max_volume: f64,
    /// 最小取引量
    pub min_volume: f64,
    /// 総手数料
    pub total_fees: f64,
    /// 平均手数料
    pub average_fee: f64,
    /// 時間帯別トランザクション数
    pub transactions_by_time: Vec<DataPoint>,
    /// 時間帯別取引量
    pub volume_by_time: Vec<DataPoint>,
    /// 時間帯別手数料
    pub fees_by_time: Vec<DataPoint>,
    /// 上位送信者
    pub top_senders: Vec<(String, usize)>,
    /// 上位受信者
    pub top_receivers: Vec<(String, usize)>,
    /// 上位送信者（取引量ベース）
    pub top_senders_by_volume: Vec<(String, f64)>,
    /// 上位受信者（取引量ベース）
    pub top_receivers_by_volume: Vec<(String, f64)>,
    /// シャード間トランザクション数
    pub cross_shard_transactions: usize,
    /// シャード内トランザクション数
    pub intra_shard_transactions: usize,
    /// シャード別トランザクション数
    pub transactions_by_shard: HashMap<ShardId, usize>,
    /// シャード別取引量
    pub volume_by_shard: HashMap<ShardId, f64>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
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
    /// 関連するアドレス
    pub addresses: Vec<String>,
    /// 発生回数
    pub occurrences: usize,
    /// 最初の発生時刻
    pub first_seen: DateTime<Utc>,
    /// 最後の発生時刻
    pub last_seen: DateTime<Utc>,
    /// 総取引量
    pub total_volume: f64,
    /// 平均取引量
    pub average_volume: f64,
    /// 関連するトランザクションID
    pub transaction_ids: Vec<String>,
    /// パターンの信頼度（0.0〜1.0）
    pub confidence: f64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// アドレスの関係
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressRelationship {
    /// 関係ID
    pub id: String,
    /// 送信者アドレス
    pub sender: String,
    /// 受信者アドレス
    pub receiver: String,
    /// トランザクション数
    pub transaction_count: usize,
    /// 最初のトランザクション時刻
    pub first_transaction: DateTime<Utc>,
    /// 最後のトランザクション時刻
    pub last_transaction: DateTime<Utc>,
    /// 総取引量
    pub total_volume: f64,
    /// 平均取引量
    pub average_volume: f64,
    /// 関連するトランザクションID
    pub transaction_ids: Vec<String>,
    /// 関係の強さ（0.0〜1.0）
    pub strength: f64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 異常検出結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionResult {
    /// 異常ID
    pub id: String,
    /// 異常タイプ
    pub anomaly_type: AnomalyType,
    /// 異常の説明
    pub description: String,
    /// 検出時刻
    pub detection_time: DateTime<Utc>,
    /// 異常発生時刻
    pub occurrence_time: DateTime<Utc>,
    /// 関連するアドレス
    pub addresses: Vec<String>,
    /// 関連するトランザクションID
    pub transaction_ids: Vec<String>,
    /// 異常の重大度（0.0〜1.0）
    pub severity: f64,
    /// 異常の信頼度（0.0〜1.0）
    pub confidence: f64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 異常タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnomalyType {
    /// 大量取引
    LargeTransaction,
    /// 不審な取引パターン
    SuspiciousPattern,
    /// 異常な取引頻度
    AbnormalFrequency,
    /// 循環取引
    CircularTransaction,
    /// 分割取引
    SplitTransaction,
    /// 統合取引
    MergeTransaction,
    /// その他
    Other(String),
}

/// 詳細トランザクション分析器
pub struct DetailedTransactionAnalyzer {
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
}

impl DetailedTransactionAnalyzer {
    /// 新しい詳細トランザクション分析器を作成
    pub fn new(
        transactions: Vec<Transaction>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        time_frame: TimeFrame,
        top_count: usize,
        anomaly_threshold: f64,
    ) -> Self {
        Self {
            transactions,
            start_time,
            end_time,
            time_frame,
            top_count,
            anomaly_threshold,
        }
    }

    /// 分析を実行
    pub fn analyze(&self) -> Result<TransactionAnalysisResult, Error> {
        // 分析対象期間内のトランザクションをフィルタリング
        let filtered_transactions: Vec<&Transaction> = self
            .transactions
            .iter()
            .filter(|tx| {
                let tx_time = Utc.timestamp(tx.timestamp, 0);
                tx_time >= self.start_time && tx_time <= self.end_time
            })
            .collect();

        if filtered_transactions.is_empty() {
            return Err(Error::InvalidInput(
                "分析対象期間内にトランザクションがありません".to_string(),
            ));
        }

        // 基本的な統計情報を計算
        let total_transactions = filtered_transactions.len();

        let successful_transactions = filtered_transactions
            .iter()
            .filter(|tx| tx.status == TransactionStatus::Confirmed)
            .count();

        let failed_transactions = filtered_transactions
            .iter()
            .filter(|tx| tx.status == TransactionStatus::Failed)
            .count();

        let pending_transactions = filtered_transactions
            .iter()
            .filter(|tx| tx.status == TransactionStatus::Pending)
            .count();

        // 取引量の統計を計算
        let volumes: Vec<f64> = filtered_transactions
            .iter()
            .map(|tx| tx.amount as f64)
            .collect();

        let total_volume: f64 = volumes.iter().sum();
        let average_volume = total_volume / total_transactions as f64;
        let max_volume = volumes.iter().fold(0.0, |max, &val| max.max(val));
        let min_volume = volumes.iter().fold(f64::MAX, |min, &val| min.min(val));

        // 手数料の統計を計算
        let fees: Vec<f64> = filtered_transactions
            .iter()
            .map(|tx| tx.fee as f64)
            .collect();

        let total_fees: f64 = fees.iter().sum();
        let average_fee = total_fees / total_transactions as f64;

        // 時間帯別の統計を計算
        let transactions_by_time = self.calculate_transactions_by_time(&filtered_transactions);
        let volume_by_time = self.calculate_volume_by_time(&filtered_transactions);
        let fees_by_time = self.calculate_fees_by_time(&filtered_transactions);

        // 上位送信者と受信者を計算
        let top_senders = self.calculate_top_senders(&filtered_transactions);
        let top_receivers = self.calculate_top_receivers(&filtered_transactions);
        let top_senders_by_volume = self.calculate_top_senders_by_volume(&filtered_transactions);
        let top_receivers_by_volume =
            self.calculate_top_receivers_by_volume(&filtered_transactions);

        // シャード関連の統計を計算
        let (cross_shard_transactions, intra_shard_transactions) =
            self.calculate_shard_transaction_counts(&filtered_transactions);
        let transactions_by_shard = self.calculate_transactions_by_shard(&filtered_transactions);
        let volume_by_shard = self.calculate_volume_by_shard(&filtered_transactions);

        // 分析結果を作成
        let result = TransactionAnalysisResult {
            id: format!("analysis-{}", Utc::now().timestamp()),
            name: format!(
                "トランザクション分析 ({} - {})",
                self.start_time.format("%Y-%m-%d %H:%M"),
                self.end_time.format("%Y-%m-%d %H:%M")
            ),
            start_time: self.start_time,
            end_time: self.end_time,
            total_transactions,
            successful_transactions,
            failed_transactions,
            pending_transactions,
            total_volume,
            average_volume,
            max_volume,
            min_volume,
            total_fees,
            average_fee,
            transactions_by_time,
            volume_by_time,
            fees_by_time,
            top_senders,
            top_receivers,
            top_senders_by_volume,
            top_receivers_by_volume,
            cross_shard_transactions,
            intra_shard_transactions,
            transactions_by_shard,
            volume_by_shard,
            metadata: None,
        };

        Ok(result)
    }

    /// 時間帯別トランザクション数を計算
    fn calculate_transactions_by_time(&self, transactions: &[&Transaction]) -> Vec<DataPoint> {
        // 時間枠ごとのトランザクション数をカウント
        let mut time_counts: HashMap<DateTime<Utc>, usize> = HashMap::new();

        // 現在の時間枠の開始時刻
        let mut current_frame_start = self.start_time;
        let frame_seconds = self.time_frame.to_seconds();

        // 時間枠ごとに初期化
        while current_frame_start <= self.end_time {
            time_counts.insert(current_frame_start, 0);
            current_frame_start = current_frame_start + Duration::seconds(frame_seconds);
        }

        // トランザクションをカウント
        for tx in transactions {
            let tx_time = Utc.timestamp(tx.timestamp, 0);
            let frame_start = self.get_frame_start(tx_time);

            if let Some(count) = time_counts.get_mut(&frame_start) {
                *count += 1;
            }
        }

        // DataPointに変換
        let mut data_points: Vec<DataPoint> = time_counts
            .iter()
            .map(|(timestamp, &count)| DataPoint {
                timestamp: *timestamp,
                value: count as f64,
                metadata: None,
            })
            .collect();

        // 時間順にソート
        data_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        data_points
    }

    /// 時間帯別取引量を計算
    fn calculate_volume_by_time(&self, transactions: &[&Transaction]) -> Vec<DataPoint> {
        // 時間枠ごとの取引量を集計
        let mut time_volumes: HashMap<DateTime<Utc>, f64> = HashMap::new();

        // 現在の時間枠の開始時刻
        let mut current_frame_start = self.start_time;
        let frame_seconds = self.time_frame.to_seconds();

        // 時間枠ごとに初期化
        while current_frame_start <= self.end_time {
            time_volumes.insert(current_frame_start, 0.0);
            current_frame_start = current_frame_start + Duration::seconds(frame_seconds);
        }

        // 取引量を集計
        for tx in transactions {
            let tx_time = Utc.timestamp(tx.timestamp, 0);
            let frame_start = self.get_frame_start(tx_time);

            if let Some(volume) = time_volumes.get_mut(&frame_start) {
                *volume += tx.amount as f64;
            }
        }

        // DataPointに変換
        let mut data_points: Vec<DataPoint> = time_volumes
            .iter()
            .map(|(timestamp, &volume)| DataPoint {
                timestamp: *timestamp,
                value: volume,
                metadata: None,
            })
            .collect();

        // 時間順にソート
        data_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        data_points
    }

    /// 時間帯別手数料を計算
    fn calculate_fees_by_time(&self, transactions: &[&Transaction]) -> Vec<DataPoint> {
        // 時間枠ごとの手数料を集計
        let mut time_fees: HashMap<DateTime<Utc>, f64> = HashMap::new();

        // 現在の時間枠の開始時刻
        let mut current_frame_start = self.start_time;
        let frame_seconds = self.time_frame.to_seconds();

        // 時間枠ごとに初期化
        while current_frame_start <= self.end_time {
            time_fees.insert(current_frame_start, 0.0);
            current_frame_start = current_frame_start + Duration::seconds(frame_seconds);
        }

        // 手数料を集計
        for tx in transactions {
            let tx_time = Utc.timestamp(tx.timestamp, 0);
            let frame_start = self.get_frame_start(tx_time);

            if let Some(fee) = time_fees.get_mut(&frame_start) {
                *fee += tx.fee as f64;
            }
        }

        // DataPointに変換
        let mut data_points: Vec<DataPoint> = time_fees
            .iter()
            .map(|(timestamp, &fee)| DataPoint {
                timestamp: *timestamp,
                value: fee,
                metadata: None,
            })
            .collect();

        // 時間順にソート
        data_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        data_points
    }

    /// 上位送信者を計算
    fn calculate_top_senders(&self, transactions: &[&Transaction]) -> Vec<(String, usize)> {
        // 送信者ごとのトランザクション数をカウント
        let mut sender_counts: HashMap<String, usize> = HashMap::new();

        for tx in transactions {
            let count = sender_counts.entry(tx.sender.clone()).or_insert(0);
            *count += 1;
        }

        // 上位N件を抽出
        let mut top_senders: Vec<(String, usize)> = sender_counts.into_iter().collect();
        top_senders.sort_by(|a, b| b.1.cmp(&a.1));
        top_senders.truncate(self.top_count);

        top_senders
    }

    /// 上位受信者を計算
    fn calculate_top_receivers(&self, transactions: &[&Transaction]) -> Vec<(String, usize)> {
        // 受信者ごとのトランザクション数をカウント
        let mut receiver_counts: HashMap<String, usize> = HashMap::new();

        for tx in transactions {
            let count = receiver_counts.entry(tx.receiver.clone()).or_insert(0);
            *count += 1;
        }

        // 上位N件を抽出
        let mut top_receivers: Vec<(String, usize)> = receiver_counts.into_iter().collect();
        top_receivers.sort_by(|a, b| b.1.cmp(&a.1));
        top_receivers.truncate(self.top_count);

        top_receivers
    }

    /// 上位送信者（取引量ベース）を計算
    fn calculate_top_senders_by_volume(&self, transactions: &[&Transaction]) -> Vec<(String, f64)> {
        // 送信者ごとの取引量を集計
        let mut sender_volumes: HashMap<String, f64> = HashMap::new();

        for tx in transactions {
            let volume = sender_volumes.entry(tx.sender.clone()).or_insert(0.0);
            *volume += tx.amount as f64;
        }

        // 上位N件を抽出
        let mut top_senders: Vec<(String, f64)> = sender_volumes.into_iter().collect();
        top_senders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top_senders.truncate(self.top_count);

        top_senders
    }

    /// 上位受信者（取引量ベース）を計算
    fn calculate_top_receivers_by_volume(
        &self,
        transactions: &[&Transaction],
    ) -> Vec<(String, f64)> {
        // 受信者ごとの取引量を集計
        let mut receiver_volumes: HashMap<String, f64> = HashMap::new();

        for tx in transactions {
            let volume = receiver_volumes.entry(tx.receiver.clone()).or_insert(0.0);
            *volume += tx.amount as f64;
        }

        // 上位N件を抽出
        let mut top_receivers: Vec<(String, f64)> = receiver_volumes.into_iter().collect();
        top_receivers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top_receivers.truncate(self.top_count);

        top_receivers
    }

    /// シャード間・シャード内トランザクション数を計算
    fn calculate_shard_transaction_counts(&self, transactions: &[&Transaction]) -> (usize, usize) {
        // シャード間・シャード内トランザクション数をカウント
        let mut cross_shard_count = 0;
        let mut intra_shard_count = 0;

        for tx in transactions {
            // 実際の実装では、送信者と受信者のシャードIDを取得して比較する
            // ここでは簡易的な実装として、送信者と受信者のアドレスの最初の文字を比較
            let sender_first_char = tx.sender.chars().next().unwrap_or('0');
            let receiver_first_char = tx.receiver.chars().next().unwrap_or('0');

            if sender_first_char == receiver_first_char {
                intra_shard_count += 1;
            } else {
                cross_shard_count += 1;
            }
        }

        (cross_shard_count, intra_shard_count)
    }

    /// シャード別トランザクション数を計算
    fn calculate_transactions_by_shard(
        &self,
        transactions: &[&Transaction],
    ) -> HashMap<ShardId, usize> {
        // シャードごとのトランザクション数をカウント
        let mut shard_counts: HashMap<ShardId, usize> = HashMap::new();

        for tx in transactions {
            // 実際の実装では、トランザクションのシャードIDを取得する
            // ここでは簡易的な実装として、送信者のアドレスの最初の文字をシャードIDとして使用
            let shard_id = tx.sender.chars().next().unwrap_or('0').to_string();

            let count = shard_counts.entry(shard_id).or_insert(0);
            *count += 1;
        }

        shard_counts
    }

    /// シャード別取引量を計算
    fn calculate_volume_by_shard(&self, transactions: &[&Transaction]) -> HashMap<ShardId, f64> {
        // シャードごとの取引量を集計
        let mut shard_volumes: HashMap<ShardId, f64> = HashMap::new();

        for tx in transactions {
            // 実際の実装では、トランザクションのシャードIDを取得する
            // ここでは簡易的な実装として、送信者のアドレスの最初の文字をシャードIDとして使用
            let shard_id = tx.sender.chars().next().unwrap_or('0').to_string();

            let volume = shard_volumes.entry(shard_id).or_insert(0.0);
            *volume += tx.amount as f64;
        }

        shard_volumes
    }

    /// 時間枠の開始時刻を取得
    fn get_frame_start(&self, time: DateTime<Utc>) -> DateTime<Utc> {
        let frame_seconds = self.time_frame.to_seconds();
        let seconds_since_start = (time - self.start_time).num_seconds();
        let frames = seconds_since_start / frame_seconds;

        self.start_time + Duration::seconds(frames * frame_seconds)
    }

    /// トランザクションパターンを検出
    pub fn detect_patterns(&self) -> Vec<TransactionPattern> {
        // 分析対象期間内のトランザクションをフィルタリング
        let filtered_transactions: Vec<&Transaction> = self
            .transactions
            .iter()
            .filter(|tx| {
                let tx_time = Utc.timestamp(tx.timestamp, 0);
                tx_time >= self.start_time && tx_time <= self.end_time
            })
            .collect();

        if filtered_transactions.is_empty() {
            return Vec::new();
        }

        // パターン検出の実装
        let mut patterns = Vec::new();

        // 循環取引パターンを検出
        let circular_patterns = self.detect_circular_transactions(&filtered_transactions);
        patterns.extend(circular_patterns);

        // 分割取引パターンを検出
        let split_patterns = self.detect_split_transactions(&filtered_transactions);
        patterns.extend(split_patterns);

        // 統合取引パターンを検出
        let merge_patterns = self.detect_merge_transactions(&filtered_transactions);
        patterns.extend(merge_patterns);

        patterns
    }

    /// 循環取引パターンを検出
    fn detect_circular_transactions(
        &self,
        transactions: &[&Transaction],
    ) -> Vec<TransactionPattern> {
        // アドレス間の取引グラフを構築
        let mut graph: HashMap<String, Vec<(String, &Transaction)>> = HashMap::new();

        for tx in transactions {
            let edges = graph.entry(tx.sender.clone()).or_insert_with(Vec::new);
            edges.push((tx.receiver.clone(), tx));
        }

        // 循環パスを検出
        let mut patterns = Vec::new();
        let mut visited = HashSet::new();

        for start_address in graph.keys() {
            if visited.contains(start_address) {
                continue;
            }

            let mut path = Vec::new();
            let mut path_txs = Vec::new();

            if self.find_cycle(
                start_address,
                &graph,
                &mut visited,
                &mut path,
                &mut path_txs,
                start_address,
                0,
            ) {
                // 循環が見つかった場合、パターンを作成
                if path.len() >= 3 {
                    let pattern_id = format!("circular-{}", Utc::now().timestamp());
                    let first_tx = path_txs.first().unwrap();
                    let last_tx = path_txs.last().unwrap();

                    let first_time = Utc.timestamp(first_tx.timestamp, 0);
                    let last_time = Utc.timestamp(last_tx.timestamp, 0);

                    let total_volume: f64 = path_txs.iter().map(|tx| tx.amount as f64).sum();
                    let average_volume = total_volume / path_txs.len() as f64;

                    let transaction_ids: Vec<String> =
                        path_txs.iter().map(|tx| tx.id.clone()).collect();

                    let pattern = TransactionPattern {
                        id: pattern_id,
                        name: format!("循環取引 ({} アドレス)", path.len()),
                        description: format!(
                            "{}個のアドレス間で循環する取引パターンが検出されました",
                            path.len()
                        ),
                        addresses: path,
                        occurrences: 1,
                        first_seen: first_time,
                        last_seen: last_time,
                        total_volume,
                        average_volume,
                        transaction_ids,
                        confidence: 0.8,
                        metadata: None,
                    };

                    patterns.push(pattern);
                }
            }
        }

        patterns
    }

    /// 循環パスを検索（深さ優先探索）
    fn find_cycle(
        &self,
        current: &str,
        graph: &HashMap<String, Vec<(String, &Transaction)>>,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
        path_txs: &mut Vec<&Transaction>,
        start: &str,
        depth: usize,
    ) -> bool {
        if depth > 0 && current == start {
            return true;
        }

        if depth > 0 && visited.contains(current) {
            return false;
        }

        visited.insert(current.to_string());
        path.push(current.to_string());

        if let Some(edges) = graph.get(current) {
            for (next, tx) in edges {
                if depth > 0 {
                    path_txs.push(tx);
                }

                if self.find_cycle(next, graph, visited, path, path_txs, start, depth + 1) {
                    return true;
                }

                if depth > 0 {
                    path_txs.pop();
                }
            }
        }

        path.pop();
        visited.remove(current);

        false
    }

    /// 分割取引パターンを検出
    fn detect_split_transactions(&self, transactions: &[&Transaction]) -> Vec<TransactionPattern> {
        // 送信者ごとのトランザクションをグループ化
        let mut sender_txs: HashMap<String, Vec<&Transaction>> = HashMap::new();

        for tx in transactions {
            let txs = sender_txs.entry(tx.sender.clone()).or_insert_with(Vec::new);
            txs.push(tx);
        }

        // 分割取引パターンを検出
        let mut patterns = Vec::new();

        for (sender, txs) in sender_txs {
            // 時間順にソート
            let mut sorted_txs = txs.clone();
            sorted_txs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            // 短時間に複数の小額取引を検出
            let mut i = 0;
            while i < sorted_txs.len() {
                let start_tx = sorted_txs[i];
                let start_time = Utc.timestamp(start_tx.timestamp, 0);

                // 1時間以内のトランザクションをグループ化
                let group: Vec<&Transaction> = sorted_txs
                    .iter()
                    .filter(|tx| {
                        let tx_time = Utc.timestamp(tx.timestamp, 0);
                        (tx_time - start_time).num_seconds().abs() <= 3600
                    })
                    .cloned()
                    .collect();

                // 3つ以上のトランザクションがあり、受信者が異なる場合
                if group.len() >= 3 {
                    let receivers: HashSet<String> =
                        group.iter().map(|tx| tx.receiver.clone()).collect();

                    if receivers.len() >= 3 {
                        let total_volume: f64 = group.iter().map(|tx| tx.amount as f64).sum();
                        let average_volume = total_volume / group.len() as f64;

                        // 平均取引量が全体の平均の20%未満の場合、分割取引と判断
                        let all_txs_avg =
                            transactions.iter().map(|tx| tx.amount as f64).sum::<f64>()
                                / transactions.len() as f64;

                        if average_volume < all_txs_avg * 0.2 {
                            let pattern_id = format!("split-{}", Utc::now().timestamp());
                            let first_tx = group.first().unwrap();
                            let last_tx = group.last().unwrap();

                            let first_time = Utc.timestamp(first_tx.timestamp, 0);
                            let last_time = Utc.timestamp(last_tx.timestamp, 0);

                            let transaction_ids: Vec<String> =
                                group.iter().map(|tx| tx.id.clone()).collect();

                            let mut addresses = vec![sender.clone()];
                            addresses.extend(receivers.into_iter());

                            let pattern = TransactionPattern {
                                id: pattern_id,
                                name: format!("分割取引 ({} 件)", group.len()),
                                description: format!("1つのアドレスから{}個の異なるアドレスへの短時間での分割取引が検出されました", group.len()),
                                addresses,
                                occurrences: group.len(),
                                first_seen: first_time,
                                last_seen: last_time,
                                total_volume,
                                average_volume,
                                transaction_ids,
                                confidence: 0.7,
                                metadata: None,
                            };

                            patterns.push(pattern);
                        }
                    }
                }

                i += group.len();
            }
        }

        patterns
    }

    /// 統合取引パターンを検出
    fn detect_merge_transactions(&self, transactions: &[&Transaction]) -> Vec<TransactionPattern> {
        // 受信者ごとのトランザクションをグループ化
        let mut receiver_txs: HashMap<String, Vec<&Transaction>> = HashMap::new();

        for tx in transactions {
            let txs = receiver_txs
                .entry(tx.receiver.clone())
                .or_insert_with(Vec::new);
            txs.push(tx);
        }

        // 統合取引パターンを検出
        let mut patterns = Vec::new();

        for (receiver, txs) in receiver_txs {
            // 時間順にソート
            let mut sorted_txs = txs.clone();
            sorted_txs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            // 短時間に複数の小額取引を検出
            let mut i = 0;
            while i < sorted_txs.len() {
                let start_tx = sorted_txs[i];
                let start_time = Utc.timestamp(start_tx.timestamp, 0);

                // 1時間以内のトランザクションをグループ化
                let group: Vec<&Transaction> = sorted_txs
                    .iter()
                    .filter(|tx| {
                        let tx_time = Utc.timestamp(tx.timestamp, 0);
                        (tx_time - start_time).num_seconds().abs() <= 3600
                    })
                    .cloned()
                    .collect();

                // 3つ以上のトランザクションがあり、送信者が異なる場合
                if group.len() >= 3 {
                    let senders: HashSet<String> =
                        group.iter().map(|tx| tx.sender.clone()).collect();

                    if senders.len() >= 3 {
                        let total_volume: f64 = group.iter().map(|tx| tx.amount as f64).sum();
                        let average_volume = total_volume / group.len() as f64;

                        // 平均取引量が全体の平均の20%未満の場合、統合取引と判断
                        let all_txs_avg =
                            transactions.iter().map(|tx| tx.amount as f64).sum::<f64>()
                                / transactions.len() as f64;

                        if average_volume < all_txs_avg * 0.2 {
                            let pattern_id = format!("merge-{}", Utc::now().timestamp());
                            let first_tx = group.first().unwrap();
                            let last_tx = group.last().unwrap();

                            let first_time = Utc.timestamp(first_tx.timestamp, 0);
                            let last_time = Utc.timestamp(last_tx.timestamp, 0);

                            let transaction_ids: Vec<String> =
                                group.iter().map(|tx| tx.id.clone()).collect();

                            let mut addresses = vec![receiver.clone()];
                            addresses.extend(senders.into_iter());

                            let pattern = TransactionPattern {
                                id: pattern_id,
                                name: format!("統合取引 ({} 件)", group.len()),
                                description: format!("{}個の異なるアドレスから1つのアドレスへの短時間での統合取引が検出されました", group.len()),
                                addresses,
                                occurrences: group.len(),
                                first_seen: first_time,
                                last_seen: last_time,
                                total_volume,
                                average_volume,
                                transaction_ids,
                                confidence: 0.7,
                                metadata: None,
                            };

                            patterns.push(pattern);
                        }
                    }
                }

                i += group.len();
            }
        }

        patterns
    }

    /// 異常を検出
    pub fn detect_anomalies(&self) -> Vec<AnomalyDetectionResult> {
        // 分析対象期間内のトランザクションをフィルタリング
        let filtered_transactions: Vec<&Transaction> = self
            .transactions
            .iter()
            .filter(|tx| {
                let tx_time = Utc.timestamp(tx.timestamp, 0);
                tx_time >= self.start_time && tx_time <= self.end_time
            })
            .collect();

        if filtered_transactions.is_empty() {
            return Vec::new();
        }

        // 異常検出の実装
        let mut anomalies = Vec::new();

        // 大量取引の異常を検出
        let large_tx_anomalies = self.detect_large_transaction_anomalies(&filtered_transactions);
        anomalies.extend(large_tx_anomalies);

        // 異常な取引頻度を検出
        let frequency_anomalies = self.detect_frequency_anomalies(&filtered_transactions);
        anomalies.extend(frequency_anomalies);

        anomalies
    }

    /// 大量取引の異常を検出
    fn detect_large_transaction_anomalies(
        &self,
        transactions: &[&Transaction],
    ) -> Vec<AnomalyDetectionResult> {
        // 取引量の統計を計算
        let volumes: Vec<f64> = transactions.iter().map(|tx| tx.amount as f64).collect();

        if volumes.is_empty() {
            return Vec::new();
        }

        let total_volume: f64 = volumes.iter().sum();
        let average_volume = total_volume / volumes.len() as f64;

        // 標準偏差を計算
        let variance: f64 = volumes
            .iter()
            .map(|v| (v - average_volume).powi(2))
            .sum::<f64>()
            / volumes.len() as f64;
        let std_dev = variance.sqrt();

        // 閾値を設定（平均 + 3 * 標準偏差）
        let threshold = average_volume + 3.0 * std_dev;

        // 閾値を超える取引を検出
        let mut anomalies = Vec::new();

        for tx in transactions {
            let volume = tx.amount as f64;

            if volume > threshold {
                let anomaly_id = format!("large-tx-{}", tx.id);
                let tx_time = Utc.timestamp(tx.timestamp, 0);

                // 異常度を計算（閾値からの乖離）
                let deviation = (volume - threshold) / threshold;
                let severity = (deviation * 10.0).min(1.0);

                // 信頼度を計算
                let confidence = if deviation > 1.0 { 0.9 } else { 0.7 };

                let anomaly = AnomalyDetectionResult {
                    id: anomaly_id,
                    anomaly_type: AnomalyType::LargeTransaction,
                    description: format!(
                        "通常の{}倍の大量取引が検出されました",
                        (volume / average_volume).round()
                    ),
                    detection_time: Utc::now(),
                    occurrence_time: tx_time,
                    addresses: vec![tx.sender.clone(), tx.receiver.clone()],
                    transaction_ids: vec![tx.id.clone()],
                    severity,
                    confidence,
                    metadata: None,
                };

                anomalies.push(anomaly);
            }
        }

        anomalies
    }

    /// 異常な取引頻度を検出
    fn detect_frequency_anomalies(
        &self,
        transactions: &[&Transaction],
    ) -> Vec<AnomalyDetectionResult> {
        // アドレスごとの取引頻度を計算
        let mut address_frequencies: HashMap<String, usize> = HashMap::new();

        for tx in transactions {
            let sender_count = address_frequencies.entry(tx.sender.clone()).or_insert(0);
            *sender_count += 1;

            let receiver_count = address_frequencies.entry(tx.receiver.clone()).or_insert(0);
            *receiver_count += 1;
        }

        if address_frequencies.is_empty() {
            return Vec::new();
        }

        // 平均取引頻度を計算
        let frequencies: Vec<usize> = address_frequencies.values().cloned().collect();
        let total_frequency: usize = frequencies.iter().sum();
        let average_frequency = total_frequency as f64 / frequencies.len() as f64;

        // 標準偏差を計算
        let variance: f64 = frequencies
            .iter()
            .map(|&f| (f as f64 - average_frequency).powi(2))
            .sum::<f64>()
            / frequencies.len() as f64;
        let std_dev = variance.sqrt();

        // 閾値を設定（平均 + 3 * 標準偏差）
        let threshold = average_frequency + 3.0 * std_dev;

        // 閾値を超えるアドレスを検出
        let mut anomalies = Vec::new();

        for (address, frequency) in address_frequencies {
            if frequency as f64 > threshold {
                // 関連するトランザクションを取得
                let related_txs: Vec<&Transaction> = transactions
                    .iter()
                    .filter(|tx| tx.sender == address || tx.receiver == address)
                    .cloned()
                    .collect();

                if related_txs.is_empty() {
                    continue;
                }

                // 最初と最後のトランザクション時刻を取得
                let mut sorted_txs = related_txs.clone();
                sorted_txs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

                let first_tx = sorted_txs.first().unwrap();
                let last_tx = sorted_txs.last().unwrap();

                let first_time = Utc.timestamp(first_tx.timestamp, 0);
                let last_time = Utc.timestamp(last_tx.timestamp, 0);

                // 期間を計算（秒）
                let duration = (last_time - first_time).num_seconds();

                // 1時間あたりの取引数を計算
                let hourly_rate = if duration > 0 {
                    (frequency as f64 * 3600.0) / duration as f64
                } else {
                    frequency as f64
                };

                // 異常度を計算
                let deviation = (frequency as f64 - threshold) / threshold;
                let severity = (deviation * 10.0).min(1.0);

                // 信頼度を計算
                let confidence = if deviation > 1.0 { 0.85 } else { 0.7 };

                let anomaly_id = format!("freq-{}-{}", address, Utc::now().timestamp());

                let transaction_ids: Vec<String> =
                    related_txs.iter().map(|tx| tx.id.clone()).collect();

                let anomaly = AnomalyDetectionResult {
                    id: anomaly_id,
                    anomaly_type: AnomalyType::AbnormalFrequency,
                    description: format!("アドレス {} の異常な取引頻度が検出されました（平均の{:.1}倍、{:.1}件/時間）", 
                        address, frequency as f64 / average_frequency, hourly_rate),
                    detection_time: Utc::now(),
                    occurrence_time: first_time,
                    addresses: vec![address],
                    transaction_ids,
                    severity,
                    confidence,
                    metadata: None,
                };

                anomalies.push(anomaly);
            }
        }

        anomalies
    }

    /// アドレス間の関係を分析
    pub fn analyze_address_relationships(&self) -> Vec<AddressRelationship> {
        // 分析対象期間内のトランザクションをフィルタリング
        let filtered_transactions: Vec<&Transaction> = self
            .transactions
            .iter()
            .filter(|tx| {
                let tx_time = Utc.timestamp(tx.timestamp, 0);
                tx_time >= self.start_time && tx_time <= self.end_time
            })
            .collect();

        if filtered_transactions.is_empty() {
            return Vec::new();
        }

        // アドレスペアごとのトランザクションをグループ化
        let mut pair_txs: HashMap<(String, String), Vec<&Transaction>> = HashMap::new();

        for tx in &filtered_transactions {
            let pair = (tx.sender.clone(), tx.receiver.clone());
            let txs = pair_txs.entry(pair).or_insert_with(Vec::new);
            txs.push(tx);
        }

        // アドレス間の関係を分析
        let mut relationships = Vec::new();

        for ((sender, receiver), txs) in pair_txs {
            if txs.len() < 2 {
                continue;
            }

            // 時間順にソート
            let mut sorted_txs = txs.clone();
            sorted_txs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            let first_tx = sorted_txs.first().unwrap();
            let last_tx = sorted_txs.last().unwrap();

            let first_time = Utc.timestamp(first_tx.timestamp, 0);
            let last_time = Utc.timestamp(last_tx.timestamp, 0);

            let total_volume: f64 = sorted_txs.iter().map(|tx| tx.amount as f64).sum();
            let average_volume = total_volume / sorted_txs.len() as f64;

            let transaction_ids: Vec<String> = sorted_txs.iter().map(|tx| tx.id.clone()).collect();

            // 関係の強さを計算（トランザクション数と総取引量に基づく）
            let max_txs = pair_txs.values().map(|v| v.len()).max().unwrap_or(1);
            let max_volume = pair_txs
                .values()
                .map(|v| v.iter().map(|tx| tx.amount as f64).sum::<f64>())
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(1.0);

            let tx_ratio = sorted_txs.len() as f64 / max_txs as f64;
            let volume_ratio = total_volume / max_volume;

            let strength = (tx_ratio * 0.7 + volume_ratio * 0.3).min(1.0);

            let relationship_id = format!("rel-{}-{}-{}", sender, receiver, Utc::now().timestamp());

            let relationship = AddressRelationship {
                id: relationship_id,
                sender,
                receiver,
                transaction_count: sorted_txs.len(),
                first_transaction: first_time,
                last_transaction: last_time,
                total_volume,
                average_volume,
                transaction_ids,
                strength,
                metadata: None,
            };

            relationships.push(relationship);
        }

        // 関係の強さでソート
        relationships.sort_by(|a, b| {
            b.strength
                .partial_cmp(&a.strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 上位N件を返す
        relationships.truncate(self.top_count);

        relationships
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transactions() -> Vec<Transaction> {
        let now = Utc::now();
        let base_timestamp = now.timestamp();

        vec![
            Transaction {
                id: "tx1".to_string(),
                sender: "addr1".to_string(),
                receiver: "addr2".to_string(),
                amount: 100,
                fee: 10,
                timestamp: base_timestamp,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx2".to_string(),
                sender: "addr2".to_string(),
                receiver: "addr3".to_string(),
                amount: 90,
                fee: 10,
                timestamp: base_timestamp + 3600,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx3".to_string(),
                sender: "addr3".to_string(),
                receiver: "addr1".to_string(),
                amount: 80,
                fee: 10,
                timestamp: base_timestamp + 7200,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx4".to_string(),
                sender: "addr4".to_string(),
                receiver: "addr5".to_string(),
                amount: 1000,
                fee: 20,
                timestamp: base_timestamp + 10800,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx5".to_string(),
                sender: "addr1".to_string(),
                receiver: "addr5".to_string(),
                amount: 50,
                fee: 5,
                timestamp: base_timestamp + 14400,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx6".to_string(),
                sender: "addr1".to_string(),
                receiver: "addr6".to_string(),
                amount: 30,
                fee: 5,
                timestamp: base_timestamp + 14410,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx7".to_string(),
                sender: "addr1".to_string(),
                receiver: "addr7".to_string(),
                amount: 20,
                fee: 5,
                timestamp: base_timestamp + 14420,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx8".to_string(),
                sender: "addr8".to_string(),
                receiver: "addr1".to_string(),
                amount: 10,
                fee: 2,
                timestamp: base_timestamp + 18000,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx9".to_string(),
                sender: "addr9".to_string(),
                receiver: "addr1".to_string(),
                amount: 15,
                fee: 2,
                timestamp: base_timestamp + 18010,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx10".to_string(),
                sender: "addr10".to_string(),
                receiver: "addr1".to_string(),
                amount: 25,
                fee: 3,
                timestamp: base_timestamp + 18020,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
        ]
    }

    #[test]
    fn test_transaction_analysis() {
        let transactions = create_test_transactions();
        let now = Utc::now();
        let start_time = now - Duration::hours(6);
        let end_time = now;

        let analyzer = DetailedTransactionAnalyzer::new(
            transactions,
            start_time,
            end_time,
            TimeFrame::Hour(1),
            5,
            3.0,
        );

        let result = analyzer.analyze().unwrap();

        assert_eq!(result.total_transactions, 10);
        assert_eq!(result.successful_transactions, 10);
        assert_eq!(result.failed_transactions, 0);
        assert_eq!(result.pending_transactions, 0);

        // 上位送信者をチェック
        assert_eq!(result.top_senders[0].0, "addr1");
        assert_eq!(result.top_senders[0].1, 4);

        // 上位受信者をチェック
        assert_eq!(result.top_receivers[0].0, "addr1");
        assert_eq!(result.top_receivers[0].1, 4);
    }

    #[test]
    fn test_pattern_detection() {
        let transactions = create_test_transactions();
        let now = Utc::now();
        let start_time = now - Duration::hours(6);
        let end_time = now;

        let analyzer = DetailedTransactionAnalyzer::new(
            transactions,
            start_time,
            end_time,
            TimeFrame::Hour(1),
            5,
            3.0,
        );

        let patterns = analyzer.detect_patterns();

        // 循環取引パターンをチェック
        let circular_patterns: Vec<&TransactionPattern> = patterns
            .iter()
            .filter(|p| p.name.contains("循環取引"))
            .collect();

        assert!(!circular_patterns.is_empty());
        assert_eq!(circular_patterns[0].addresses.len(), 3);

        // 分割取引パターンをチェック
        let split_patterns: Vec<&TransactionPattern> = patterns
            .iter()
            .filter(|p| p.name.contains("分割取引"))
            .collect();

        assert!(!split_patterns.is_empty());

        // 統合取引パターンをチェック
        let merge_patterns: Vec<&TransactionPattern> = patterns
            .iter()
            .filter(|p| p.name.contains("統合取引"))
            .collect();

        assert!(!merge_patterns.is_empty());
    }

    #[test]
    fn test_anomaly_detection() {
        let mut transactions = create_test_transactions();

        // 異常な大量取引を追加
        let now = Utc::now();
        let base_timestamp = now.timestamp();

        transactions.push(Transaction {
            id: "tx_large".to_string(),
            sender: "addr_large".to_string(),
            receiver: "addr_receiver".to_string(),
            amount: 10000,
            fee: 100,
            timestamp: base_timestamp + 20000,
            signature: None,
            status: TransactionStatus::Confirmed,
            data: None,
        });

        let start_time = now - Duration::hours(6);
        let end_time = now;

        let analyzer = DetailedTransactionAnalyzer::new(
            transactions,
            start_time,
            end_time,
            TimeFrame::Hour(1),
            5,
            3.0,
        );

        let anomalies = analyzer.detect_anomalies();

        // 大量取引の異常をチェック
        let large_tx_anomalies: Vec<&AnomalyDetectionResult> = anomalies
            .iter()
            .filter(|a| a.anomaly_type == AnomalyType::LargeTransaction)
            .collect();

        assert!(!large_tx_anomalies.is_empty());
        assert_eq!(large_tx_anomalies[0].transaction_ids[0], "tx_large");

        // 異常な取引頻度をチェック
        let frequency_anomalies: Vec<&AnomalyDetectionResult> = anomalies
            .iter()
            .filter(|a| a.anomaly_type == AnomalyType::AbnormalFrequency)
            .collect();

        assert!(!frequency_anomalies.is_empty());
        assert!(frequency_anomalies[0]
            .addresses
            .contains(&"addr1".to_string()));
    }

    #[test]
    fn test_address_relationships() {
        let transactions = create_test_transactions();
        let now = Utc::now();
        let start_time = now - Duration::hours(6);
        let end_time = now;

        let analyzer = DetailedTransactionAnalyzer::new(
            transactions,
            start_time,
            end_time,
            TimeFrame::Hour(1),
            5,
            3.0,
        );

        let relationships = analyzer.analyze_address_relationships();

        assert!(!relationships.is_empty());

        // 最も強い関係をチェック
        let strongest = &relationships[0];
        assert!(strongest.strength > 0.5);
        assert!(strongest.transaction_count >= 2);
    }
}
