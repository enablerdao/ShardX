use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::analytics::metrics::{MetricType, MetricValue, MetricsCollector};
use crate::error::Error;

/// チャートデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    /// チャートタイプ
    pub chart_type: ChartType,
    /// タイトル
    pub title: String,
    /// 説明
    pub description: Option<String>,
    /// X軸ラベル
    pub x_axis_label: String,
    /// Y軸ラベル
    pub y_axis_label: String,
    /// カテゴリ
    pub categories: Vec<String>,
    /// データセット
    pub datasets: Vec<Dataset>,
    /// 注釈
    pub annotations: Vec<Annotation>,
    /// 生成時刻
    pub generated_at: DateTime<Utc>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// データセット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// ラベル
    pub label: String,
    /// データ
    pub data: Vec<f64>,
    /// 色
    pub color: Option<String>,
    /// 塗りつぶし色
    pub fill_color: Option<String>,
    /// 線の太さ
    pub line_width: Option<f32>,
    /// 点の半径
    pub point_radius: Option<f32>,
    /// 点の形状
    pub point_style: Option<String>,
    /// 表示順序
    pub order: Option<i32>,
    /// 表示/非表示
    pub hidden: bool,
}

/// 注釈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// タイプ
    pub annotation_type: AnnotationType,
    /// ラベル
    pub label: String,
    /// X位置
    pub x: Option<String>,
    /// Y位置
    pub y: Option<f64>,
    /// 色
    pub color: Option<String>,
    /// 線の太さ
    pub line_width: Option<f32>,
    /// 線のスタイル
    pub line_style: Option<String>,
    /// 説明
    pub description: Option<String>,
}

/// 注釈タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnnotationType {
    /// 線
    Line,
    /// 点
    Point,
    /// 範囲
    Range,
    /// ボックス
    Box,
    /// ラベル
    Label,
}

/// チャートタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChartType {
    /// 折れ線グラフ
    Line,
    /// 棒グラフ
    Bar,
    /// 円グラフ
    Pie,
    /// ドーナツグラフ
    Doughnut,
    /// レーダーチャート
    Radar,
    /// 散布図
    Scatter,
    /// バブルチャート
    Bubble,
    /// エリアチャート
    Area,
    /// 複合チャート
    Mixed,
    /// ヒートマップ
    Heatmap,
    /// キャンドルスティック
    Candlestick,
}

/// チャートオプション
#[derive(Debug, Clone)]
pub struct ChartOptions {
    /// 時間範囲
    pub time_range: Option<TimeRange>,
    /// 集計間隔
    pub aggregation_interval: Option<AggregationInterval>,
    /// データ制限
    pub data_limit: Option<usize>,
    /// フィルタ
    pub filters: HashMap<String, String>,
    /// ソート順
    pub sort_order: Option<SortOrder>,
    /// グループ化
    pub grouping: Option<String>,
    /// 色テーマ
    pub color_theme: Option<String>,
    /// 凡例表示
    pub show_legend: bool,
    /// グリッド表示
    pub show_grid: bool,
    /// アニメーション
    pub animation: bool,
    /// ツールチップ
    pub tooltips: bool,
    /// レスポンシブ
    pub responsive: bool,
    /// アスペクト比
    pub aspect_ratio: Option<f32>,
}

/// 時間範囲
#[derive(Debug, Clone)]
pub enum TimeRange {
    /// 過去N時間
    LastHours(u32),
    /// 過去N日
    LastDays(u32),
    /// 過去N週間
    LastWeeks(u32),
    /// 過去N月
    LastMonths(u32),
    /// カスタム範囲
    Custom(DateTime<Utc>, DateTime<Utc>),
}

/// 集計間隔
#[derive(Debug, Clone)]
pub enum AggregationInterval {
    /// 分単位
    Minute,
    /// 時間単位
    Hour,
    /// 日単位
    Day,
    /// 週単位
    Week,
    /// 月単位
    Month,
    /// カスタム間隔（秒）
    Custom(u64),
}

/// ソート順
#[derive(Debug, Clone)]
pub enum SortOrder {
    /// 昇順
    Ascending,
    /// 降順
    Descending,
    /// カスタム順
    Custom(Vec<String>),
}

impl Default for ChartOptions {
    fn default() -> Self {
        Self {
            time_range: Some(TimeRange::LastHours(24)),
            aggregation_interval: Some(AggregationInterval::Hour),
            data_limit: Some(100),
            filters: HashMap::new(),
            sort_order: Some(SortOrder::Ascending),
            grouping: None,
            color_theme: Some("default".to_string()),
            show_legend: true,
            show_grid: true,
            animation: true,
            tooltips: true,
            responsive: true,
            aspect_ratio: Some(1.5),
        }
    }
}

/// チャート生成器
pub struct ChartGenerator {
    /// メトリクスコレクタ
    metrics_collector: Arc<MetricsCollector>,
    /// デフォルトオプション
    default_options: ChartOptions,
    /// カラーパレット
    color_palettes: HashMap<String, Vec<String>>,
}

impl ChartGenerator {
    /// 新しいチャート生成器を作成
    pub fn new(metrics_collector: Arc<MetricsCollector>, options: Option<ChartOptions>) -> Self {
        let default_options = options.unwrap_or_default();

        // カラーパレットを初期化
        let mut color_palettes = HashMap::new();

        // デフォルトパレット
        color_palettes.insert(
            "default".to_string(),
            vec![
                "#4e79a7".to_string(),
                "#f28e2c".to_string(),
                "#e15759".to_string(),
                "#76b7b2".to_string(),
                "#59a14f".to_string(),
                "#edc949".to_string(),
                "#af7aa1".to_string(),
                "#ff9da7".to_string(),
                "#9c755f".to_string(),
                "#bab0ab".to_string(),
            ],
        );

        // ダークパレット
        color_palettes.insert(
            "dark".to_string(),
            vec![
                "#1f77b4".to_string(),
                "#ff7f0e".to_string(),
                "#2ca02c".to_string(),
                "#d62728".to_string(),
                "#9467bd".to_string(),
                "#8c564b".to_string(),
                "#e377c2".to_string(),
                "#7f7f7f".to_string(),
                "#bcbd22".to_string(),
                "#17becf".to_string(),
            ],
        );

        // パステルパレット
        color_palettes.insert(
            "pastel".to_string(),
            vec![
                "#a1c9f4".to_string(),
                "#ffb482".to_string(),
                "#8de5a1".to_string(),
                "#ff9f9b".to_string(),
                "#d0bbff".to_string(),
                "#debb9b".to_string(),
                "#fab0e4".to_string(),
                "#cfcfcf".to_string(),
                "#fffea3".to_string(),
                "#b9f2f0".to_string(),
            ],
        );

        Self {
            metrics_collector,
            default_options,
            color_palettes,
        }
    }

    /// トランザクション処理速度チャートを生成
    pub fn generate_transaction_throughput_chart(
        &self,
        options: Option<ChartOptions>,
    ) -> Result<ChartData, Error> {
        let options = options.unwrap_or_else(|| self.default_options.clone());

        // メトリクスからデータを取得
        let metrics = self.metrics_collector.get_metrics(
            MetricType::TransactionThroughput,
            self.get_time_range(&options)?,
            self.get_aggregation_interval(&options)?,
        )?;

        // カテゴリを作成（時間ラベル）
        let categories = self.create_time_categories(&options)?;

        // データセットを作成
        let mut datasets = Vec::new();

        // 全体のスループット
        let mut total_data = vec![0.0; categories.len()];

        // メトリクスからデータを抽出
        for (timestamp, value) in metrics {
            if let MetricValue::Throughput(tps) = value {
                // タイムスタンプからインデックスを計算
                if let Some(index) = self.timestamp_to_index(timestamp, &options)? {
                    if index < total_data.len() {
                        total_data[index] = tps;
                    }
                }
            }
        }

        // データセットを追加
        datasets.push(Dataset {
            label: "トランザクション処理速度 (TPS)".to_string(),
            data: total_data,
            color: Some(self.get_color(0, &options)),
            fill_color: Some(self.get_fill_color(0, &options)),
            line_width: Some(2.0),
            point_radius: Some(3.0),
            point_style: Some("circle".to_string()),
            order: Some(1),
            hidden: false,
        });

        // チャートデータを作成
        let chart_data = ChartData {
            chart_type: ChartType::Line,
            title: "トランザクション処理速度".to_string(),
            description: Some("単位時間あたりの処理トランザクション数".to_string()),
            x_axis_label: "時間".to_string(),
            y_axis_label: "TPS".to_string(),
            categories,
            datasets,
            annotations: Vec::new(),
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        Ok(chart_data)
    }

    /// メモリ使用量チャートを生成
    pub fn generate_memory_usage_chart(
        &self,
        options: Option<ChartOptions>,
    ) -> Result<ChartData, Error> {
        let options = options.unwrap_or_else(|| self.default_options.clone());

        // メトリクスからデータを取得
        let metrics = self.metrics_collector.get_metrics(
            MetricType::MemoryUsage,
            self.get_time_range(&options)?,
            self.get_aggregation_interval(&options)?,
        )?;

        // カテゴリを作成（時間ラベル）
        let categories = self.create_time_categories(&options)?;

        // データセットを作成
        let mut datasets = Vec::new();

        // メモリ使用量データ
        let mut memory_data = vec![0.0; categories.len()];

        // メトリクスからデータを抽出
        for (timestamp, value) in metrics {
            if let MetricValue::Memory(bytes) = value {
                // タイムスタンプからインデックスを計算
                if let Some(index) = self.timestamp_to_index(timestamp, &options)? {
                    if index < memory_data.len() {
                        // バイトをMBに変換
                        memory_data[index] = bytes as f64 / (1024.0 * 1024.0);
                    }
                }
            }
        }

        // データセットを追加
        datasets.push(Dataset {
            label: "メモリ使用量 (MB)".to_string(),
            data: memory_data,
            color: Some(self.get_color(1, &options)),
            fill_color: Some(self.get_fill_color(1, &options)),
            line_width: Some(2.0),
            point_radius: Some(3.0),
            point_style: Some("circle".to_string()),
            order: Some(1),
            hidden: false,
        });

        // チャートデータを作成
        let chart_data = ChartData {
            chart_type: ChartType::Area,
            title: "メモリ使用量".to_string(),
            description: Some("時間経過によるメモリ使用量の推移".to_string()),
            x_axis_label: "時間".to_string(),
            y_axis_label: "メモリ使用量 (MB)".to_string(),
            categories,
            datasets,
            annotations: Vec::new(),
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        Ok(chart_data)
    }

    /// トランザクションレイテンシチャートを生成
    pub fn generate_transaction_latency_chart(
        &self,
        options: Option<ChartOptions>,
    ) -> Result<ChartData, Error> {
        let options = options.unwrap_or_else(|| self.default_options.clone());

        // メトリクスからデータを取得
        let metrics = self.metrics_collector.get_metrics(
            MetricType::TransactionLatency,
            self.get_time_range(&options)?,
            self.get_aggregation_interval(&options)?,
        )?;

        // カテゴリを作成（時間ラベル）
        let categories = self.create_time_categories(&options)?;

        // データセットを作成
        let mut datasets = Vec::new();

        // 平均レイテンシデータ
        let mut avg_latency_data = vec![0.0; categories.len()];
        // 最大レイテンシデータ
        let mut max_latency_data = vec![0.0; categories.len()];
        // 最小レイテンシデータ
        let mut min_latency_data = vec![0.0; categories.len()];

        // メトリクスからデータを抽出
        for (timestamp, value) in metrics {
            if let MetricValue::Latency { avg, max, min } = value {
                // タイムスタンプからインデックスを計算
                if let Some(index) = self.timestamp_to_index(timestamp, &options)? {
                    if index < avg_latency_data.len() {
                        avg_latency_data[index] = avg;
                        max_latency_data[index] = max;
                        min_latency_data[index] = min;
                    }
                }
            }
        }

        // 平均レイテンシデータセットを追加
        datasets.push(Dataset {
            label: "平均レイテンシ (ms)".to_string(),
            data: avg_latency_data,
            color: Some(self.get_color(2, &options)),
            fill_color: None,
            line_width: Some(2.0),
            point_radius: Some(3.0),
            point_style: Some("circle".to_string()),
            order: Some(1),
            hidden: false,
        });

        // 最大レイテンシデータセットを追加
        datasets.push(Dataset {
            label: "最大レイテンシ (ms)".to_string(),
            data: max_latency_data,
            color: Some(self.get_color(3, &options)),
            fill_color: None,
            line_width: Some(1.0),
            point_radius: Some(2.0),
            point_style: Some("triangle".to_string()),
            order: Some(2),
            hidden: false,
        });

        // 最小レイテンシデータセットを追加
        datasets.push(Dataset {
            label: "最小レイテンシ (ms)".to_string(),
            data: min_latency_data,
            color: Some(self.get_color(4, &options)),
            fill_color: None,
            line_width: Some(1.0),
            point_radius: Some(2.0),
            point_style: Some("rect".to_string()),
            order: Some(3),
            hidden: true, // デフォルトでは非表示
        });

        // チャートデータを作成
        let chart_data = ChartData {
            chart_type: ChartType::Line,
            title: "トランザクションレイテンシ".to_string(),
            description: Some("トランザクション処理にかかる時間".to_string()),
            x_axis_label: "時間".to_string(),
            y_axis_label: "レイテンシ (ms)".to_string(),
            categories,
            datasets,
            annotations: Vec::new(),
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        Ok(chart_data)
    }

    /// シャード負荷チャートを生成
    pub fn generate_shard_load_chart(
        &self,
        options: Option<ChartOptions>,
    ) -> Result<ChartData, Error> {
        let options = options.unwrap_or_else(|| self.default_options.clone());

        // メトリクスからデータを取得
        let metrics = self.metrics_collector.get_metrics(
            MetricType::ShardLoad,
            self.get_time_range(&options)?,
            self.get_aggregation_interval(&options)?,
        )?;

        // シャードIDのリストを取得
        let shard_ids = self.metrics_collector.get_shard_ids()?;

        // カテゴリを作成（シャードID）
        let categories = shard_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>();

        // データセットを作成
        let mut datasets = Vec::new();

        // 現在の負荷データ
        let mut current_load_data = vec![0.0; categories.len()];

        // メトリクスからデータを抽出（最新のデータのみ使用）
        let mut latest_timestamp = Utc::now() - Duration::days(365); // 1年前を初期値とする
        let mut latest_metrics = HashMap::new();

        for (timestamp, value) in metrics {
            if timestamp > latest_timestamp {
                latest_timestamp = timestamp;
                if let MetricValue::ShardLoad(shard_loads) = value {
                    latest_metrics = shard_loads;
                }
            }
        }

        // 最新のデータを使用してチャートデータを作成
        for (i, shard_id) in shard_ids.iter().enumerate() {
            if let Some(load) = latest_metrics.get(shard_id) {
                current_load_data[i] = *load;
            }
        }

        // データセットを追加
        datasets.push(Dataset {
            label: "シャード負荷 (%)".to_string(),
            data: current_load_data,
            color: Some(self.get_color(5, &options)),
            fill_color: Some(self.get_fill_color(5, &options)),
            line_width: None,
            point_radius: None,
            point_style: None,
            order: Some(1),
            hidden: false,
        });

        // チャートデータを作成
        let chart_data = ChartData {
            chart_type: ChartType::Bar,
            title: "シャード負荷".to_string(),
            description: Some("各シャードの現在の負荷状況".to_string()),
            x_axis_label: "シャードID".to_string(),
            y_axis_label: "負荷 (%)".to_string(),
            categories,
            datasets,
            annotations: Vec::new(),
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        Ok(chart_data)
    }

    /// トランザクションタイプ分布チャートを生成
    pub fn generate_transaction_type_distribution_chart(
        &self,
        options: Option<ChartOptions>,
    ) -> Result<ChartData, Error> {
        let options = options.unwrap_or_else(|| self.default_options.clone());

        // メトリクスからデータを取得
        let metrics = self.metrics_collector.get_metrics(
            MetricType::TransactionTypeDistribution,
            self.get_time_range(&options)?,
            self.get_aggregation_interval(&options)?,
        )?;

        // トランザクションタイプのリストを取得
        let transaction_types = self.metrics_collector.get_transaction_types()?;

        // カテゴリを作成（トランザクションタイプ）
        let categories = transaction_types
            .iter()
            .map(|t| format!("{:?}", t))
            .collect::<Vec<_>>();

        // データセットを作成
        let mut datasets = Vec::new();

        // 分布データ
        let mut distribution_data = vec![0.0; categories.len()];

        // メトリクスからデータを抽出（最新のデータのみ使用）
        let mut latest_timestamp = Utc::now() - Duration::days(365); // 1年前を初期値とする
        let mut latest_metrics = HashMap::new();

        for (timestamp, value) in metrics {
            if timestamp > latest_timestamp {
                latest_timestamp = timestamp;
                if let MetricValue::Distribution(type_distribution) = value {
                    latest_metrics = type_distribution;
                }
            }
        }

        // 最新のデータを使用してチャートデータを作成
        for (i, tx_type) in transaction_types.iter().enumerate() {
            if let Some(percentage) = latest_metrics.get(&format!("{:?}", tx_type)) {
                distribution_data[i] = *percentage;
            }
        }

        // データセットを追加
        datasets.push(Dataset {
            label: "トランザクションタイプ分布 (%)".to_string(),
            data: distribution_data,
            color: None,
            fill_color: Some(self.get_multi_colors(&options)),
            line_width: None,
            point_radius: None,
            point_style: None,
            order: Some(1),
            hidden: false,
        });

        // チャートデータを作成
        let chart_data = ChartData {
            chart_type: ChartType::Pie,
            title: "トランザクションタイプ分布".to_string(),
            description: Some("トランザクションタイプ別の割合".to_string()),
            x_axis_label: "".to_string(),
            y_axis_label: "".to_string(),
            categories,
            datasets,
            annotations: Vec::new(),
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        Ok(chart_data)
    }

    /// カスタムチャートを生成
    pub fn generate_custom_chart(
        &self,
        chart_type: ChartType,
        title: &str,
        metric_type: MetricType,
        options: Option<ChartOptions>,
    ) -> Result<ChartData, Error> {
        let options = options.unwrap_or_else(|| self.default_options.clone());

        // メトリクスからデータを取得
        let metrics = self.metrics_collector.get_metrics(
            metric_type.clone(),
            self.get_time_range(&options)?,
            self.get_aggregation_interval(&options)?,
        )?;

        // カテゴリとデータセットを作成
        let (categories, datasets) = match metric_type {
            MetricType::Custom(name) => {
                // カスタムメトリクスの場合は時間ベースのカテゴリを使用
                let categories = self.create_time_categories(&options)?;
                let mut data = vec![0.0; categories.len()];

                // メトリクスからデータを抽出
                for (timestamp, value) in metrics {
                    if let MetricValue::Custom(val) = value {
                        // タイムスタンプからインデックスを計算
                        if let Some(index) = self.timestamp_to_index(timestamp, &options)? {
                            if index < data.len() {
                                data[index] = val;
                            }
                        }
                    }
                }

                // データセットを作成
                let dataset = Dataset {
                    label: name,
                    data,
                    color: Some(self.get_color(0, &options)),
                    fill_color: Some(self.get_fill_color(0, &options)),
                    line_width: Some(2.0),
                    point_radius: Some(3.0),
                    point_style: Some("circle".to_string()),
                    order: Some(1),
                    hidden: false,
                };

                (categories, vec![dataset])
            }
            _ => {
                // その他のメトリクスタイプの場合はデフォルト実装を使用
                let categories = self.create_time_categories(&options)?;
                let mut data = vec![0.0; categories.len()];

                // メトリクスからデータを抽出
                for (timestamp, value) in metrics {
                    if let Some(val) = self.extract_metric_value(&value) {
                        // タイムスタンプからインデックスを計算
                        if let Some(index) = self.timestamp_to_index(timestamp, &options)? {
                            if index < data.len() {
                                data[index] = val;
                            }
                        }
                    }
                }

                // データセットを作成
                let dataset = Dataset {
                    label: format!("{:?}", metric_type),
                    data,
                    color: Some(self.get_color(0, &options)),
                    fill_color: Some(self.get_fill_color(0, &options)),
                    line_width: Some(2.0),
                    point_radius: Some(3.0),
                    point_style: Some("circle".to_string()),
                    order: Some(1),
                    hidden: false,
                };

                (categories, vec![dataset])
            }
        };

        // チャートデータを作成
        let chart_data = ChartData {
            chart_type,
            title: title.to_string(),
            description: Some(format!("メトリクス: {:?}", metric_type)),
            x_axis_label: "時間".to_string(),
            y_axis_label: "値".to_string(),
            categories,
            datasets,
            annotations: Vec::new(),
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        Ok(chart_data)
    }

    /// 時間範囲を取得
    fn get_time_range(
        &self,
        options: &ChartOptions,
    ) -> Result<(DateTime<Utc>, DateTime<Utc>), Error> {
        let now = Utc::now();

        match &options.time_range {
            Some(TimeRange::LastHours(hours)) => {
                let start = now - Duration::hours(*hours as i64);
                Ok((start, now))
            }
            Some(TimeRange::LastDays(days)) => {
                let start = now - Duration::days(*days as i64);
                Ok((start, now))
            }
            Some(TimeRange::LastWeeks(weeks)) => {
                let start = now - Duration::weeks(*weeks as i64);
                Ok((start, now))
            }
            Some(TimeRange::LastMonths(months)) => {
                let start = now - Duration::days(30 * *months as i64);
                Ok((start, now))
            }
            Some(TimeRange::Custom(start, end)) => Ok((*start, *end)),
            None => {
                // デフォルトは24時間
                let start = now - Duration::hours(24);
                Ok((start, now))
            }
        }
    }

    /// 集計間隔を取得
    fn get_aggregation_interval(&self, options: &ChartOptions) -> Result<Duration, Error> {
        match &options.aggregation_interval {
            Some(AggregationInterval::Minute) => Ok(Duration::minutes(1)),
            Some(AggregationInterval::Hour) => Ok(Duration::hours(1)),
            Some(AggregationInterval::Day) => Ok(Duration::days(1)),
            Some(AggregationInterval::Week) => Ok(Duration::weeks(1)),
            Some(AggregationInterval::Month) => Ok(Duration::days(30)),
            Some(AggregationInterval::Custom(seconds)) => Ok(Duration::seconds(*seconds as i64)),
            None => Ok(Duration::hours(1)), // デフォルトは1時間
        }
    }

    /// 時間カテゴリを作成
    fn create_time_categories(&self, options: &ChartOptions) -> Result<Vec<String>, Error> {
        let (start, end) = self.get_time_range(options)?;
        let interval = self.get_aggregation_interval(options)?;

        let mut categories = Vec::new();
        let mut current = start;

        while current <= end {
            let format = match interval {
                d if d <= Duration::minutes(1) => "%H:%M:%S",
                d if d <= Duration::hours(1) => "%H:%M",
                d if d <= Duration::days(1) => "%m-%d %H:%M",
                d if d <= Duration::weeks(1) => "%m-%d",
                _ => "%Y-%m-%d",
            };

            categories.push(current.format(format).to_string());
            current = current + interval;
        }

        Ok(categories)
    }

    /// タイムスタンプからインデックスを計算
    fn timestamp_to_index(
        &self,
        timestamp: DateTime<Utc>,
        options: &ChartOptions,
    ) -> Result<Option<usize>, Error> {
        let (start, end) = self.get_time_range(options)?;
        let interval = self.get_aggregation_interval(options)?;

        if timestamp < start || timestamp > end {
            return Ok(None);
        }

        let diff = timestamp - start;
        let index =
            (diff.num_milliseconds() as f64 / interval.num_milliseconds() as f64).floor() as usize;

        Ok(Some(index))
    }

    /// メトリック値を抽出
    fn extract_metric_value(&self, value: &MetricValue) -> Option<f64> {
        match value {
            MetricValue::Throughput(tps) => Some(*tps),
            MetricValue::Latency { avg, .. } => Some(*avg),
            MetricValue::Memory(bytes) => Some(*bytes as f64 / (1024.0 * 1024.0)), // バイトをMBに変換
            MetricValue::CPU(percentage) => Some(*percentage),
            MetricValue::Custom(val) => Some(*val),
            _ => None,
        }
    }

    /// 色を取得
    fn get_color(&self, index: usize, options: &ChartOptions) -> String {
        let theme = options.color_theme.as_deref().unwrap_or("default");

        if let Some(palette) = self.color_palettes.get(theme) {
            palette[index % palette.len()].clone()
        } else {
            // デフォルトパレットにフォールバック
            self.color_palettes.get("default").unwrap()[index % 10].clone()
        }
    }

    /// 塗りつぶし色を取得
    fn get_fill_color(&self, index: usize, options: &ChartOptions) -> String {
        let color = self.get_color(index, options);

        // 透明度を追加
        if color.starts_with('#') && color.len() == 7 {
            format!("{}80", color) // 50%の透明度を追加
        } else {
            color
        }
    }

    /// 複数の色を取得
    fn get_multi_colors(&self, options: &ChartOptions) -> Vec<String> {
        let theme = options.color_theme.as_deref().unwrap_or("default");

        if let Some(palette) = self.color_palettes.get(theme) {
            palette.clone()
        } else {
            // デフォルトパレットにフォールバック
            self.color_palettes.get("default").unwrap().clone()
        }
    }
}
