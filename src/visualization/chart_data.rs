use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Error;
use crate::transaction::Transaction;

/// チャートデータ期間
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChartPeriod {
    /// 1時間
    Hour,
    /// 1日
    Day,
    /// 1週間
    Week,
    /// 1ヶ月
    Month,
    /// 1年
    Year,
    /// すべて
    All,
}

impl ChartPeriod {
    /// 期間の開始時刻を計算
    pub fn get_start_time(&self, end_time: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            ChartPeriod::Hour => end_time - Duration::hours(1),
            ChartPeriod::Day => end_time - Duration::days(1),
            ChartPeriod::Week => end_time - Duration::weeks(1),
            ChartPeriod::Month => end_time - Duration::days(30),
            ChartPeriod::Year => end_time - Duration::days(365),
            ChartPeriod::All => DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        }
    }

    /// 期間の間隔を取得
    pub fn get_interval(&self) -> Duration {
        match self {
            ChartPeriod::Hour => Duration::minutes(5),
            ChartPeriod::Day => Duration::hours(1),
            ChartPeriod::Week => Duration::hours(6),
            ChartPeriod::Month => Duration::days(1),
            ChartPeriod::Year => Duration::days(7),
            ChartPeriod::All => Duration::days(30),
        }
    }

    /// 文字列から期間を解析
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_str() {
            "hour" => Ok(ChartPeriod::Hour),
            "day" => Ok(ChartPeriod::Day),
            "week" => Ok(ChartPeriod::Week),
            "month" => Ok(ChartPeriod::Month),
            "year" => Ok(ChartPeriod::Year),
            "all" => Ok(ChartPeriod::All),
            _ => Err(Error::ValidationError(format!(
                "Invalid chart period: {}",
                s
            ))),
        }
    }

    /// 期間を文字列に変換
    pub fn to_str(&self) -> &'static str {
        match self {
            ChartPeriod::Hour => "hour",
            ChartPeriod::Day => "day",
            ChartPeriod::Week => "week",
            ChartPeriod::Month => "month",
            ChartPeriod::Year => "year",
            ChartPeriod::All => "all",
        }
    }
}

/// チャートメトリック
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChartMetric {
    /// トランザクション数
    TransactionCount,
    /// トランザクション量
    TransactionVolume,
    /// 手数料
    Fees,
    /// アクティブアドレス数
    ActiveAddresses,
    /// 平均トランザクションサイズ
    AverageTransactionSize,
    /// 価格
    Price,
    /// シャード間トランザクション数
    CrossShardTransactions,
}

impl ChartMetric {
    /// 文字列からメトリックを解析
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_str() {
            "transactions" | "transaction_count" => Ok(ChartMetric::TransactionCount),
            "volume" | "transaction_volume" => Ok(ChartMetric::TransactionVolume),
            "fees" => Ok(ChartMetric::Fees),
            "active_addresses" => Ok(ChartMetric::ActiveAddresses),
            "avg_transaction_size" | "average_transaction_size" => {
                Ok(ChartMetric::AverageTransactionSize)
            }
            "price" => Ok(ChartMetric::Price),
            "cross_shard" | "cross_shard_transactions" => Ok(ChartMetric::CrossShardTransactions),
            _ => Err(Error::ValidationError(format!(
                "Invalid chart metric: {}",
                s
            ))),
        }
    }

    /// メトリックを文字列に変換
    pub fn to_str(&self) -> &'static str {
        match self {
            ChartMetric::TransactionCount => "transaction_count",
            ChartMetric::TransactionVolume => "transaction_volume",
            ChartMetric::Fees => "fees",
            ChartMetric::ActiveAddresses => "active_addresses",
            ChartMetric::AverageTransactionSize => "average_transaction_size",
            ChartMetric::Price => "price",
            ChartMetric::CrossShardTransactions => "cross_shard_transactions",
        }
    }

    /// メトリックの表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            ChartMetric::TransactionCount => "Transaction Count",
            ChartMetric::TransactionVolume => "Transaction Volume",
            ChartMetric::Fees => "Fees",
            ChartMetric::ActiveAddresses => "Active Addresses",
            ChartMetric::AverageTransactionSize => "Average Transaction Size",
            ChartMetric::Price => "Price",
            ChartMetric::CrossShardTransactions => "Cross-Shard Transactions",
        }
    }
}

/// データポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 値
    pub value: f64,
}

/// チャートデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    /// メトリック
    pub metric: ChartMetric,
    /// 期間
    pub period: ChartPeriod,
    /// 開始時刻
    pub start_time: DateTime<Utc>,
    /// 終了時刻
    pub end_time: DateTime<Utc>,
    /// データポイント
    pub data_points: Vec<DataPoint>,
}

/// チャートデータマネージャー
pub struct ChartDataManager {
    /// 価格データ
    price_data: HashMap<String, Vec<DataPoint>>,
}

impl ChartDataManager {
    /// 新しいチャートデータマネージャーを作成
    pub fn new() -> Self {
        Self {
            price_data: HashMap::new(),
        }
    }

    /// トランザクションからチャートデータを生成
    pub fn generate_transaction_chart_data(
        &self,
        transactions: &[Transaction],
        metric: ChartMetric,
        period: ChartPeriod,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<ChartData, Error> {
        let end = end_time.unwrap_or_else(Utc::now);
        let start = start_time.unwrap_or_else(|| period.get_start_time(end));

        // 期間の間隔を取得
        let interval = period.get_interval();

        // データポイントを初期化
        let mut data_points = Vec::new();
        let mut current_time = start;

        while current_time <= end {
            data_points.push(DataPoint {
                timestamp: current_time,
                value: 0.0,
            });

            current_time = current_time + interval;
        }

        // トランザクションを集計
        match metric {
            ChartMetric::TransactionCount => {
                self.aggregate_transaction_count(
                    transactions,
                    &mut data_points,
                    start,
                    end,
                    interval,
                )?;
            }
            ChartMetric::TransactionVolume => {
                self.aggregate_transaction_volume(
                    transactions,
                    &mut data_points,
                    start,
                    end,
                    interval,
                )?;
            }
            ChartMetric::Fees => {
                self.aggregate_transaction_fees(
                    transactions,
                    &mut data_points,
                    start,
                    end,
                    interval,
                )?;
            }
            ChartMetric::ActiveAddresses => {
                self.aggregate_active_addresses(
                    transactions,
                    &mut data_points,
                    start,
                    end,
                    interval,
                )?;
            }
            ChartMetric::AverageTransactionSize => {
                self.aggregate_average_transaction_size(
                    transactions,
                    &mut data_points,
                    start,
                    end,
                    interval,
                )?;
            }
            ChartMetric::CrossShardTransactions => {
                self.aggregate_cross_shard_transactions(
                    transactions,
                    &mut data_points,
                    start,
                    end,
                    interval,
                )?;
            }
            ChartMetric::Price => {
                return Err(Error::ValidationError(
                    "Price metric requires price data".to_string(),
                ));
            }
        }

        Ok(ChartData {
            metric,
            period,
            start_time: start,
            end_time: end,
            data_points,
        })
    }

    /// 価格チャートデータを生成
    pub fn generate_price_chart_data(
        &self,
        pair: &str,
        period: ChartPeriod,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<ChartData, Error> {
        let end = end_time.unwrap_or_else(Utc::now);
        let start = start_time.unwrap_or_else(|| period.get_start_time(end));

        // 価格データを取得
        let price_data = self.price_data.get(pair).ok_or_else(|| {
            Error::ValidationError(format!("Price data not available for pair: {}", pair))
        })?;

        // 期間の間隔を取得
        let interval = period.get_interval();

        // データポイントを初期化
        let mut data_points = Vec::new();
        let mut current_time = start;

        while current_time <= end {
            // 現在の時間に最も近い価格データを検索
            let closest_price = price_data
                .iter()
                .filter(|point| point.timestamp <= current_time)
                .min_by_key(|point| (current_time - point.timestamp).num_seconds().abs() as u64)
                .map(|point| point.value)
                .unwrap_or(0.0);

            data_points.push(DataPoint {
                timestamp: current_time,
                value: closest_price,
            });

            current_time = current_time + interval;
        }

        Ok(ChartData {
            metric: ChartMetric::Price,
            period,
            start_time: start,
            end_time: end,
            data_points,
        })
    }

    /// 価格データを追加
    pub fn add_price_data(&mut self, pair: &str, timestamp: DateTime<Utc>, price: f64) {
        let entry = self
            .price_data
            .entry(pair.to_string())
            .or_insert_with(Vec::new);

        entry.push(DataPoint {
            timestamp,
            value: price,
        });

        // タイムスタンプでソート
        entry.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    /// トランザクション数を集計
    fn aggregate_transaction_count(
        &self,
        transactions: &[Transaction],
        data_points: &mut [DataPoint],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: Duration,
    ) -> Result<(), Error> {
        // 各間隔のトランザクション数をカウント
        for tx in transactions {
            let tx_time =
                DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0).ok_or_else(|| {
                    Error::ValidationError("Invalid transaction timestamp".to_string())
                })?;

            if tx_time < start || tx_time > end {
                continue;
            }

            // トランザクションが属する間隔を特定
            let index = ((tx_time - start).num_seconds() / interval.num_seconds()) as usize;

            if index < data_points.len() {
                data_points[index].value += 1.0;
            }
        }

        Ok(())
    }

    /// トランザクション量を集計
    fn aggregate_transaction_volume(
        &self,
        transactions: &[Transaction],
        data_points: &mut [DataPoint],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: Duration,
    ) -> Result<(), Error> {
        // 各間隔のトランザクション量を集計
        for tx in transactions {
            let tx_time =
                DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0).ok_or_else(|| {
                    Error::ValidationError("Invalid transaction timestamp".to_string())
                })?;

            if tx_time < start || tx_time > end {
                continue;
            }

            // トランザクションが属する間隔を特定
            let index = ((tx_time - start).num_seconds() / interval.num_seconds()) as usize;

            if index < data_points.len() {
                let amount = tx.amount.parse::<f64>().unwrap_or(0.0);
                data_points[index].value += amount;
            }
        }

        Ok(())
    }

    /// トランザクション手数料を集計
    fn aggregate_transaction_fees(
        &self,
        transactions: &[Transaction],
        data_points: &mut [DataPoint],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: Duration,
    ) -> Result<(), Error> {
        // 各間隔のトランザクション手数料を集計
        for tx in transactions {
            let tx_time =
                DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0).ok_or_else(|| {
                    Error::ValidationError("Invalid transaction timestamp".to_string())
                })?;

            if tx_time < start || tx_time > end {
                continue;
            }

            // トランザクションが属する間隔を特定
            let index = ((tx_time - start).num_seconds() / interval.num_seconds()) as usize;

            if index < data_points.len() {
                let fee = tx.fee.parse::<f64>().unwrap_or(0.0);
                data_points[index].value += fee;
            }
        }

        Ok(())
    }

    /// アクティブアドレス数を集計
    fn aggregate_active_addresses(
        &self,
        transactions: &[Transaction],
        data_points: &mut [DataPoint],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: Duration,
    ) -> Result<(), Error> {
        // 各間隔のアクティブアドレスを集計
        let mut active_addresses: Vec<HashMap<String, bool>> =
            vec![HashMap::new(); data_points.len()];

        for tx in transactions {
            let tx_time =
                DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0).ok_or_else(|| {
                    Error::ValidationError("Invalid transaction timestamp".to_string())
                })?;

            if tx_time < start || tx_time > end {
                continue;
            }

            // トランザクションが属する間隔を特定
            let index = ((tx_time - start).num_seconds() / interval.num_seconds()) as usize;

            if index < active_addresses.len() {
                active_addresses[index].insert(tx.from.clone(), true);
                active_addresses[index].insert(tx.to.clone(), true);
            }
        }

        // アクティブアドレス数をデータポイントに設定
        for (i, addresses) in active_addresses.iter().enumerate() {
            if i < data_points.len() {
                data_points[i].value = addresses.len() as f64;
            }
        }

        Ok(())
    }

    /// 平均トランザクションサイズを集計
    fn aggregate_average_transaction_size(
        &self,
        transactions: &[Transaction],
        data_points: &mut [DataPoint],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: Duration,
    ) -> Result<(), Error> {
        // 各間隔のトランザクション数と合計サイズを集計
        let mut tx_counts: Vec<u64> = vec![0; data_points.len()];
        let mut tx_sizes: Vec<u64> = vec![0; data_points.len()];

        for tx in transactions {
            let tx_time =
                DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0).ok_or_else(|| {
                    Error::ValidationError("Invalid transaction timestamp".to_string())
                })?;

            if tx_time < start || tx_time > end {
                continue;
            }

            // トランザクションが属する間隔を特定
            let index = ((tx_time - start).num_seconds() / interval.num_seconds()) as usize;

            if index < tx_counts.len() {
                tx_counts[index] += 1;

                // トランザクションサイズを計算（簡易的な実装）
                let size = tx.data.as_ref().map(|d| d.len()).unwrap_or(0) + 100; // 基本サイズ + データサイズ
                tx_sizes[index] += size as u64;
            }
        }

        // 平均トランザクションサイズを計算
        for i in 0..data_points.len() {
            if tx_counts[i] > 0 {
                data_points[i].value = tx_sizes[i] as f64 / tx_counts[i] as f64;
            } else {
                data_points[i].value = 0.0;
            }
        }

        Ok(())
    }

    /// クロスシャードトランザクション数を集計
    fn aggregate_cross_shard_transactions(
        &self,
        transactions: &[Transaction],
        data_points: &mut [DataPoint],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: Duration,
    ) -> Result<(), Error> {
        // 各間隔のクロスシャードトランザクション数をカウント
        for tx in transactions {
            let tx_time =
                DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0).ok_or_else(|| {
                    Error::ValidationError("Invalid transaction timestamp".to_string())
                })?;

            if tx_time < start || tx_time > end {
                continue;
            }

            // クロスシャードトランザクションのみをカウント
            if tx.parent_id.is_none() {
                continue;
            }

            // トランザクションが属する間隔を特定
            let index = ((tx_time - start).num_seconds() / interval.num_seconds()) as usize;

            if index < data_points.len() {
                data_points[index].value += 1.0;
            }
        }

        Ok(())
    }
}

impl Default for ChartDataManager {
    fn default() -> Self {
        Self::new()
    }
}
