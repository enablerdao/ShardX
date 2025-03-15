use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus};
use crate::shard::ShardId;

/// チャートタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChartType {
    /// 折れ線グラフ
    Line,
    /// 棒グラフ
    Bar,
    /// 円グラフ
    Pie,
    /// エリアチャート
    Area,
    /// キャンドルスティック
    Candlestick,
    /// ヒートマップ
    Heatmap,
    /// 散布図
    Scatter,
    /// レーダーチャート
    Radar,
}

/// 時間枠
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeFrame {
    /// 分単位
    Minute(u32),
    /// 時間単位
    Hour(u32),
    /// 日単位
    Day(u32),
    /// 週単位
    Week(u32),
    /// 月単位
    Month(u32),
}

impl TimeFrame {
    /// 秒数に変換
    pub fn to_seconds(&self) -> i64 {
        match self {
            TimeFrame::Minute(n) => *n as i64 * 60,
            TimeFrame::Hour(n) => *n as i64 * 3600,
            TimeFrame::Day(n) => *n as i64 * 86400,
            TimeFrame::Week(n) => *n as i64 * 604800,
            TimeFrame::Month(n) => *n as i64 * 2592000, // 30日で計算
        }
    }
    
    /// 文字列表現を取得
    pub fn to_string(&self) -> String {
        match self {
            TimeFrame::Minute(n) => format!("{}分", n),
            TimeFrame::Hour(n) => format!("{}時間", n),
            TimeFrame::Day(n) => format!("{}日", n),
            TimeFrame::Week(n) => format!("{}週間", n),
            TimeFrame::Month(n) => format!("{}ヶ月", n),
        }
    }
    
    /// 期間内の時間枠の数を計算
    pub fn count_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> usize {
        let duration = end.signed_duration_since(start).num_seconds();
        let frame_seconds = self.to_seconds();
        
        (duration / frame_seconds) as usize + 1
    }
}

/// データポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 値
    pub value: f64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// OHLC（始値・高値・安値・終値）データポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCDataPoint {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 始値
    pub open: f64,
    /// 高値
    pub high: f64,
    /// 安値
    pub low: f64,
    /// 終値
    pub close: f64,
    /// 出来高
    pub volume: Option<f64>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// チャートデータシリーズ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSeries {
    /// シリーズID
    pub id: String,
    /// シリーズ名
    pub name: String,
    /// データポイント
    pub data_points: Vec<DataPoint>,
    /// 色
    pub color: Option<String>,
    /// 線のスタイル
    pub line_style: Option<String>,
    /// マーカーのスタイル
    pub marker_style: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// OHLCデータシリーズ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCDataSeries {
    /// シリーズID
    pub id: String,
    /// シリーズ名
    pub name: String,
    /// OHLCデータポイント
    pub data_points: Vec<OHLCDataPoint>,
    /// 上昇色
    pub up_color: Option<String>,
    /// 下降色
    pub down_color: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// チャート設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// チャートID
    pub id: String,
    /// チャートタイプ
    pub chart_type: ChartType,
    /// タイトル
    pub title: String,
    /// サブタイトル
    pub subtitle: Option<String>,
    /// X軸ラベル
    pub x_axis_label: Option<String>,
    /// Y軸ラベル
    pub y_axis_label: Option<String>,
    /// 凡例の表示
    pub show_legend: bool,
    /// グリッドの表示
    pub show_grid: bool,
    /// ツールチップの表示
    pub show_tooltip: bool,
    /// アニメーションの有効化
    pub enable_animation: bool,
    /// ズームの有効化
    pub enable_zoom: bool,
    /// エクスポートの有効化
    pub enable_export: bool,
    /// テーマ
    pub theme: Option<String>,
    /// 幅
    pub width: Option<u32>,
    /// 高さ
    pub height: Option<u32>,
    /// マージン
    pub margin: Option<ChartMargin>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// チャートマージン
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartMargin {
    /// 上
    pub top: u32,
    /// 右
    pub right: u32,
    /// 下
    pub bottom: u32,
    /// 左
    pub left: u32,
}

/// チャート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    /// 設定
    pub config: ChartConfig,
    /// データシリーズ
    pub series: Vec<DataSeries>,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
}

/// OHLCチャート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCChart {
    /// 設定
    pub config: ChartConfig,
    /// OHLCデータシリーズ
    pub series: Vec<OHLCDataSeries>,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
}

/// チャートビルダー
pub struct ChartBuilder {
    /// 設定
    config: ChartConfig,
    /// データシリーズ
    series: Vec<DataSeries>,
}

impl ChartBuilder {
    /// 新しいチャートビルダーを作成
    pub fn new(id: &str, chart_type: ChartType, title: &str) -> Self {
        let config = ChartConfig {
            id: id.to_string(),
            chart_type,
            title: title.to_string(),
            subtitle: None,
            x_axis_label: None,
            y_axis_label: None,
            show_legend: true,
            show_grid: true,
            show_tooltip: true,
            enable_animation: true,
            enable_zoom: true,
            enable_export: true,
            theme: None,
            width: None,
            height: None,
            margin: None,
            metadata: None,
        };
        
        Self {
            config,
            series: Vec::new(),
        }
    }
    
    /// サブタイトルを設定
    pub fn with_subtitle(mut self, subtitle: &str) -> Self {
        self.config.subtitle = Some(subtitle.to_string());
        self
    }
    
    /// X軸ラベルを設定
    pub fn with_x_axis_label(mut self, label: &str) -> Self {
        self.config.x_axis_label = Some(label.to_string());
        self
    }
    
    /// Y軸ラベルを設定
    pub fn with_y_axis_label(mut self, label: &str) -> Self {
        self.config.y_axis_label = Some(label.to_string());
        self
    }
    
    /// 凡例の表示を設定
    pub fn with_legend(mut self, show: bool) -> Self {
        self.config.show_legend = show;
        self
    }
    
    /// グリッドの表示を設定
    pub fn with_grid(mut self, show: bool) -> Self {
        self.config.show_grid = show;
        self
    }
    
    /// ツールチップの表示を設定
    pub fn with_tooltip(mut self, show: bool) -> Self {
        self.config.show_tooltip = show;
        self
    }
    
    /// アニメーションの有効化を設定
    pub fn with_animation(mut self, enable: bool) -> Self {
        self.config.enable_animation = enable;
        self
    }
    
    /// ズームの有効化を設定
    pub fn with_zoom(mut self, enable: bool) -> Self {
        self.config.enable_zoom = enable;
        self
    }
    
    /// エクスポートの有効化を設定
    pub fn with_export(mut self, enable: bool) -> Self {
        self.config.enable_export = enable;
        self
    }
    
    /// テーマを設定
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.config.theme = Some(theme.to_string());
        self
    }
    
    /// サイズを設定
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.config.width = Some(width);
        self.config.height = Some(height);
        self
    }
    
    /// マージンを設定
    pub fn with_margin(mut self, top: u32, right: u32, bottom: u32, left: u32) -> Self {
        self.config.margin = Some(ChartMargin {
            top,
            right,
            bottom,
            left,
        });
        self
    }
    
    /// メタデータを設定
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.config.metadata = Some(metadata);
        self
    }
    
    /// データシリーズを追加
    pub fn add_series(mut self, series: DataSeries) -> Self {
        self.series.push(series);
        self
    }
    
    /// チャートを構築
    pub fn build(self) -> Chart {
        let now = Utc::now();
        
        Chart {
            config: self.config,
            series: self.series,
            created_at: now,
            updated_at: now,
        }
    }
}

/// OHLCチャートビルダー
pub struct OHLCChartBuilder {
    /// 設定
    config: ChartConfig,
    /// OHLCデータシリーズ
    series: Vec<OHLCDataSeries>,
}

impl OHLCChartBuilder {
    /// 新しいOHLCチャートビルダーを作成
    pub fn new(id: &str, title: &str) -> Self {
        let config = ChartConfig {
            id: id.to_string(),
            chart_type: ChartType::Candlestick,
            title: title.to_string(),
            subtitle: None,
            x_axis_label: None,
            y_axis_label: None,
            show_legend: true,
            show_grid: true,
            show_tooltip: true,
            enable_animation: true,
            enable_zoom: true,
            enable_export: true,
            theme: None,
            width: None,
            height: None,
            margin: None,
            metadata: None,
        };
        
        Self {
            config,
            series: Vec::new(),
        }
    }
    
    /// サブタイトルを設定
    pub fn with_subtitle(mut self, subtitle: &str) -> Self {
        self.config.subtitle = Some(subtitle.to_string());
        self
    }
    
    /// X軸ラベルを設定
    pub fn with_x_axis_label(mut self, label: &str) -> Self {
        self.config.x_axis_label = Some(label.to_string());
        self
    }
    
    /// Y軸ラベルを設定
    pub fn with_y_axis_label(mut self, label: &str) -> Self {
        self.config.y_axis_label = Some(label.to_string());
        self
    }
    
    /// 凡例の表示を設定
    pub fn with_legend(mut self, show: bool) -> Self {
        self.config.show_legend = show;
        self
    }
    
    /// グリッドの表示を設定
    pub fn with_grid(mut self, show: bool) -> Self {
        self.config.show_grid = show;
        self
    }
    
    /// ツールチップの表示を設定
    pub fn with_tooltip(mut self, show: bool) -> Self {
        self.config.show_tooltip = show;
        self
    }
    
    /// アニメーションの有効化を設定
    pub fn with_animation(mut self, enable: bool) -> Self {
        self.config.enable_animation = enable;
        self
    }
    
    /// ズームの有効化を設定
    pub fn with_zoom(mut self, enable: bool) -> Self {
        self.config.enable_zoom = enable;
        self
    }
    
    /// エクスポートの有効化を設定
    pub fn with_export(mut self, enable: bool) -> Self {
        self.config.enable_export = enable;
        self
    }
    
    /// テーマを設定
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.config.theme = Some(theme.to_string());
        self
    }
    
    /// サイズを設定
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.config.width = Some(width);
        self.config.height = Some(height);
        self
    }
    
    /// マージンを設定
    pub fn with_margin(mut self, top: u32, right: u32, bottom: u32, left: u32) -> Self {
        self.config.margin = Some(ChartMargin {
            top,
            right,
            bottom,
            left,
        });
        self
    }
    
    /// メタデータを設定
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.config.metadata = Some(metadata);
        self
    }
    
    /// OHLCデータシリーズを追加
    pub fn add_series(mut self, series: OHLCDataSeries) -> Self {
        self.series.push(series);
        self
    }
    
    /// OHLCチャートを構築
    pub fn build(self) -> OHLCChart {
        let now = Utc::now();
        
        OHLCChart {
            config: self.config,
            series: self.series,
            created_at: now,
            updated_at: now,
        }
    }
}

/// データシリーズビルダー
pub struct DataSeriesBuilder {
    /// シリーズID
    id: String,
    /// シリーズ名
    name: String,
    /// データポイント
    data_points: Vec<DataPoint>,
    /// 色
    color: Option<String>,
    /// 線のスタイル
    line_style: Option<String>,
    /// マーカーのスタイル
    marker_style: Option<String>,
    /// メタデータ
    metadata: Option<HashMap<String, String>>,
}

impl DataSeriesBuilder {
    /// 新しいデータシリーズビルダーを作成
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            data_points: Vec::new(),
            color: None,
            line_style: None,
            marker_style: None,
            metadata: None,
        }
    }
    
    /// 色を設定
    pub fn with_color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }
    
    /// 線のスタイルを設定
    pub fn with_line_style(mut self, style: &str) -> Self {
        self.line_style = Some(style.to_string());
        self
    }
    
    /// マーカーのスタイルを設定
    pub fn with_marker_style(mut self, style: &str) -> Self {
        self.marker_style = Some(style.to_string());
        self
    }
    
    /// メタデータを設定
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// データポイントを追加
    pub fn add_data_point(mut self, timestamp: DateTime<Utc>, value: f64) -> Self {
        self.data_points.push(DataPoint {
            timestamp,
            value,
            metadata: None,
        });
        self
    }
    
    /// メタデータ付きデータポイントを追加
    pub fn add_data_point_with_metadata(mut self, timestamp: DateTime<Utc>, value: f64, metadata: HashMap<String, String>) -> Self {
        self.data_points.push(DataPoint {
            timestamp,
            value,
            metadata: Some(metadata),
        });
        self
    }
    
    /// 複数のデータポイントを追加
    pub fn add_data_points(mut self, points: Vec<(DateTime<Utc>, f64)>) -> Self {
        for (timestamp, value) in points {
            self.data_points.push(DataPoint {
                timestamp,
                value,
                metadata: None,
            });
        }
        self
    }
    
    /// データシリーズを構築
    pub fn build(self) -> DataSeries {
        DataSeries {
            id: self.id,
            name: self.name,
            data_points: self.data_points,
            color: self.color,
            line_style: self.line_style,
            marker_style: self.marker_style,
            metadata: self.metadata,
        }
    }
}

/// OHLCデータシリーズビルダー
pub struct OHLCDataSeriesBuilder {
    /// シリーズID
    id: String,
    /// シリーズ名
    name: String,
    /// OHLCデータポイント
    data_points: Vec<OHLCDataPoint>,
    /// 上昇色
    up_color: Option<String>,
    /// 下降色
    down_color: Option<String>,
    /// メタデータ
    metadata: Option<HashMap<String, String>>,
}

impl OHLCDataSeriesBuilder {
    /// 新しいOHLCデータシリーズビルダーを作成
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            data_points: Vec::new(),
            up_color: None,
            down_color: None,
            metadata: None,
        }
    }
    
    /// 上昇色を設定
    pub fn with_up_color(mut self, color: &str) -> Self {
        self.up_color = Some(color.to_string());
        self
    }
    
    /// 下降色を設定
    pub fn with_down_color(mut self, color: &str) -> Self {
        self.down_color = Some(color.to_string());
        self
    }
    
    /// メタデータを設定
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// OHLCデータポイントを追加
    pub fn add_data_point(mut self, timestamp: DateTime<Utc>, open: f64, high: f64, low: f64, close: f64) -> Self {
        self.data_points.push(OHLCDataPoint {
            timestamp,
            open,
            high,
            low,
            close,
            volume: None,
            metadata: None,
        });
        self
    }
    
    /// 出来高付きOHLCデータポイントを追加
    pub fn add_data_point_with_volume(mut self, timestamp: DateTime<Utc>, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        self.data_points.push(OHLCDataPoint {
            timestamp,
            open,
            high,
            low,
            close,
            volume: Some(volume),
            metadata: None,
        });
        self
    }
    
    /// メタデータ付きOHLCデータポイントを追加
    pub fn add_data_point_with_metadata(mut self, timestamp: DateTime<Utc>, open: f64, high: f64, low: f64, close: f64, volume: Option<f64>, metadata: HashMap<String, String>) -> Self {
        self.data_points.push(OHLCDataPoint {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            metadata: Some(metadata),
        });
        self
    }
    
    /// 複数のOHLCデータポイントを追加
    pub fn add_data_points(mut self, points: Vec<(DateTime<Utc>, f64, f64, f64, f64)>) -> Self {
        for (timestamp, open, high, low, close) in points {
            self.data_points.push(OHLCDataPoint {
                timestamp,
                open,
                high,
                low,
                close,
                volume: None,
                metadata: None,
            });
        }
        self
    }
    
    /// 複数の出来高付きOHLCデータポイントを追加
    pub fn add_data_points_with_volume(mut self, points: Vec<(DateTime<Utc>, f64, f64, f64, f64, f64)>) -> Self {
        for (timestamp, open, high, low, close, volume) in points {
            self.data_points.push(OHLCDataPoint {
                timestamp,
                open,
                high,
                low,
                close,
                volume: Some(volume),
                metadata: None,
            });
        }
        self
    }
    
    /// OHLCデータシリーズを構築
    pub fn build(self) -> OHLCDataSeries {
        OHLCDataSeries {
            id: self.id,
            name: self.name,
            data_points: self.data_points,
            up_color: self.up_color,
            down_color: self.down_color,
            metadata: self.metadata,
        }
    }
}

/// チャートデータアグリゲーター
pub struct ChartDataAggregator;

impl ChartDataAggregator {
    /// 時間枠でデータを集約
    pub fn aggregate_by_timeframe(data_points: &[DataPoint], time_frame: &TimeFrame) -> Vec<DataPoint> {
        if data_points.is_empty() {
            return Vec::new();
        }
        
        // データポイントを時間順にソート
        let mut sorted_points = data_points.to_vec();
        sorted_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // 最初と最後のタイムスタンプを取得
        let start_time = sorted_points.first().unwrap().timestamp;
        let end_time = sorted_points.last().unwrap().timestamp;
        
        // 時間枠の秒数を取得
        let frame_seconds = time_frame.to_seconds();
        
        // 結果を格納するベクター
        let mut result = Vec::new();
        
        // 現在の時間枠の開始時刻
        let mut current_frame_start = start_time;
        
        while current_frame_start <= end_time {
            // 現在の時間枠の終了時刻
            let current_frame_end = current_frame_start + Duration::seconds(frame_seconds);
            
            // 現在の時間枠に含まれるデータポイントを抽出
            let frame_points: Vec<&DataPoint> = sorted_points.iter()
                .filter(|p| p.timestamp >= current_frame_start && p.timestamp < current_frame_end)
                .collect();
            
            if !frame_points.is_empty() {
                // 値の平均を計算
                let avg_value = frame_points.iter().map(|p| p.value).sum::<f64>() / frame_points.len() as f64;
                
                // 集約されたデータポイントを作成
                let aggregated_point = DataPoint {
                    timestamp: current_frame_start,
                    value: avg_value,
                    metadata: None,
                };
                
                result.push(aggregated_point);
            }
            
            // 次の時間枠に移動
            current_frame_start = current_frame_end;
        }
        
        result
    }
    
    /// 時間枠でOHLCデータを集約
    pub fn aggregate_ohlc_by_timeframe(data_points: &[OHLCDataPoint], time_frame: &TimeFrame) -> Vec<OHLCDataPoint> {
        if data_points.is_empty() {
            return Vec::new();
        }
        
        // データポイントを時間順にソート
        let mut sorted_points = data_points.to_vec();
        sorted_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // 最初と最後のタイムスタンプを取得
        let start_time = sorted_points.first().unwrap().timestamp;
        let end_time = sorted_points.last().unwrap().timestamp;
        
        // 時間枠の秒数を取得
        let frame_seconds = time_frame.to_seconds();
        
        // 結果を格納するベクター
        let mut result = Vec::new();
        
        // 現在の時間枠の開始時刻
        let mut current_frame_start = start_time;
        
        while current_frame_start <= end_time {
            // 現在の時間枠の終了時刻
            let current_frame_end = current_frame_start + Duration::seconds(frame_seconds);
            
            // 現在の時間枠に含まれるデータポイントを抽出
            let frame_points: Vec<&OHLCDataPoint> = sorted_points.iter()
                .filter(|p| p.timestamp >= current_frame_start && p.timestamp < current_frame_end)
                .collect();
            
            if !frame_points.is_empty() {
                // 始値は期間内の最初のポイントの始値
                let open = frame_points.first().unwrap().open;
                
                // 高値は期間内の最大値
                let high = frame_points.iter().map(|p| p.high).fold(f64::NEG_INFINITY, f64::max);
                
                // 安値は期間内の最小値
                let low = frame_points.iter().map(|p| p.low).fold(f64::INFINITY, f64::min);
                
                // 終値は期間内の最後のポイントの終値
                let close = frame_points.last().unwrap().close;
                
                // 出来高は期間内の合計
                let volume = if frame_points.iter().all(|p| p.volume.is_some()) {
                    Some(frame_points.iter().filter_map(|p| p.volume).sum())
                } else {
                    None
                };
                
                // 集約されたOHLCデータポイントを作成
                let aggregated_point = OHLCDataPoint {
                    timestamp: current_frame_start,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    metadata: None,
                };
                
                result.push(aggregated_point);
            }
            
            // 次の時間枠に移動
            current_frame_start = current_frame_end;
        }
        
        result
    }
    
    /// 移動平均を計算
    pub fn calculate_moving_average(data_points: &[DataPoint], period: usize) -> Vec<DataPoint> {
        if data_points.len() < period {
            return Vec::new();
        }
        
        // データポイントを時間順にソート
        let mut sorted_points = data_points.to_vec();
        sorted_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // 結果を格納するベクター
        let mut result = Vec::new();
        
        // 移動平均を計算
        for i in period-1..sorted_points.len() {
            let sum: f64 = sorted_points[i-(period-1)..=i].iter().map(|p| p.value).sum();
            let avg = sum / period as f64;
            
            let ma_point = DataPoint {
                timestamp: sorted_points[i].timestamp,
                value: avg,
                metadata: None,
            };
            
            result.push(ma_point);
        }
        
        result
    }
    
    /// 指数移動平均を計算
    pub fn calculate_exponential_moving_average(data_points: &[DataPoint], period: usize) -> Vec<DataPoint> {
        if data_points.len() < period {
            return Vec::new();
        }
        
        // データポイントを時間順にソート
        let mut sorted_points = data_points.to_vec();
        sorted_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // 結果を格納するベクター
        let mut result = Vec::new();
        
        // 最初のSMAを計算
        let initial_sum: f64 = sorted_points[0..period].iter().map(|p| p.value).sum();
        let initial_sma = initial_sum / period as f64;
        
        // 平滑化係数
        let alpha = 2.0 / (period as f64 + 1.0);
        
        // 最初のEMAはSMAと同じ
        let mut current_ema = initial_sma;
        
        // 最初のEMAを追加
        result.push(DataPoint {
            timestamp: sorted_points[period-1].timestamp,
            value: current_ema,
            metadata: None,
        });
        
        // 残りのEMAを計算
        for i in period..sorted_points.len() {
            current_ema = alpha * sorted_points[i].value + (1.0 - alpha) * current_ema;
            
            result.push(DataPoint {
                timestamp: sorted_points[i].timestamp,
                value: current_ema,
                metadata: None,
            });
        }
        
        result
    }
    
    /// ボリンジャーバンドを計算
    pub fn calculate_bollinger_bands(data_points: &[DataPoint], period: usize, std_dev_multiplier: f64) -> (Vec<DataPoint>, Vec<DataPoint>, Vec<DataPoint>) {
        if data_points.len() < period {
            return (Vec::new(), Vec::new(), Vec::new());
        }
        
        // データポイントを時間順にソート
        let mut sorted_points = data_points.to_vec();
        sorted_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // 移動平均を計算
        let middle_band = Self::calculate_moving_average(&sorted_points, period);
        
        // 上下のバンドを計算
        let mut upper_band = Vec::new();
        let mut lower_band = Vec::new();
        
        for (i, ma_point) in middle_band.iter().enumerate() {
            let start_idx = i;
            let end_idx = i + period;
            
            if end_idx <= sorted_points.len() {
                // 標準偏差を計算
                let mean = ma_point.value;
                let variance: f64 = sorted_points[start_idx..end_idx].iter()
                    .map(|p| (p.value - mean).powi(2))
                    .sum::<f64>() / period as f64;
                let std_dev = variance.sqrt();
                
                // 上下のバンドを計算
                let upper = mean + std_dev_multiplier * std_dev;
                let lower = mean - std_dev_multiplier * std_dev;
                
                upper_band.push(DataPoint {
                    timestamp: ma_point.timestamp,
                    value: upper,
                    metadata: None,
                });
                
                lower_band.push(DataPoint {
                    timestamp: ma_point.timestamp,
                    value: lower,
                    metadata: None,
                });
            }
        }
        
        (middle_band, upper_band, lower_band)
    }
    
    /// 相対力指数（RSI）を計算
    pub fn calculate_rsi(data_points: &[DataPoint], period: usize) -> Vec<DataPoint> {
        if data_points.len() <= period {
            return Vec::new();
        }
        
        // データポイントを時間順にソート
        let mut sorted_points = data_points.to_vec();
        sorted_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // 結果を格納するベクター
        let mut result = Vec::new();
        
        // 価格変動を計算
        let mut price_changes = Vec::new();
        for i in 1..sorted_points.len() {
            price_changes.push(sorted_points[i].value - sorted_points[i-1].value);
        }
        
        // 最初のRSIを計算
        let mut gains = 0.0;
        let mut losses = 0.0;
        
        for i in 0..period {
            if price_changes[i] >= 0.0 {
                gains += price_changes[i];
            } else {
                losses -= price_changes[i];
            }
        }
        
        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;
        
        let mut rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
        let mut rsi = 100.0 - (100.0 / (1.0 + rs));
        
        result.push(DataPoint {
            timestamp: sorted_points[period].timestamp,
            value: rsi,
            metadata: None,
        });
        
        // 残りのRSIを計算
        let mut current_avg_gain = avg_gain;
        let mut current_avg_loss = avg_loss;
        
        for i in period..price_changes.len() {
            // 平均利益と平均損失を更新
            if price_changes[i] >= 0.0 {
                current_avg_gain = (current_avg_gain * (period as f64 - 1.0) + price_changes[i]) / period as f64;
                current_avg_loss = (current_avg_loss * (period as f64 - 1.0)) / period as f64;
            } else {
                current_avg_gain = (current_avg_gain * (period as f64 - 1.0)) / period as f64;
                current_avg_loss = (current_avg_loss * (period as f64 - 1.0) - price_changes[i]) / period as f64;
            }
            
            // RSIを計算
            rs = if current_avg_loss == 0.0 { 100.0 } else { current_avg_gain / current_avg_loss };
            rsi = 100.0 - (100.0 / (1.0 + rs));
            
            result.push(DataPoint {
                timestamp: sorted_points[i+1].timestamp,
                value: rsi,
                metadata: None,
            });
        }
        
        result
    }
    
    /// 移動平均収束拡散（MACD）を計算
    pub fn calculate_macd(data_points: &[DataPoint], fast_period: usize, slow_period: usize, signal_period: usize) -> (Vec<DataPoint>, Vec<DataPoint>, Vec<DataPoint>) {
        if data_points.len() < slow_period + signal_period {
            return (Vec::new(), Vec::new(), Vec::new());
        }
        
        // データポイントを時間順にソート
        let mut sorted_points = data_points.to_vec();
        sorted_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // 短期EMAを計算
        let fast_ema = Self::calculate_exponential_moving_average(&sorted_points, fast_period);
        
        // 長期EMAを計算
        let slow_ema = Self::calculate_exponential_moving_average(&sorted_points, slow_period);
        
        // MACDラインを計算
        let mut macd_line = Vec::new();
        
        // 長期EMAが短期EMAより少ない場合、短期EMAを長期EMAの長さに合わせる
        let start_idx = slow_ema.len() - fast_ema.len();
        
        for i in 0..slow_ema.len() - start_idx {
            let macd_value = fast_ema[i].value - slow_ema[i + start_idx].value;
            
            macd_line.push(DataPoint {
                timestamp: slow_ema[i + start_idx].timestamp,
                value: macd_value,
                metadata: None,
            });
        }
        
        // シグナルラインを計算
        let signal_line = Self::calculate_exponential_moving_average(&macd_line, signal_period);
        
        // ヒストグラムを計算
        let mut histogram = Vec::new();
        
        for i in 0..signal_line.len() {
            let hist_value = macd_line[i + macd_line.len() - signal_line.len()].value - signal_line[i].value;
            
            histogram.push(DataPoint {
                timestamp: signal_line[i].timestamp,
                value: hist_value,
                metadata: None,
            });
        }
        
        (macd_line, signal_line, histogram)
    }
}

/// チャートレンダラー
pub trait ChartRenderer {
    /// チャートをレンダリング
    fn render_chart(&self, chart: &Chart) -> Result<String, Error>;
    
    /// OHLCチャートをレンダリング
    fn render_ohlc_chart(&self, chart: &OHLCChart) -> Result<String, Error>;
}

/// SVGチャートレンダラー
pub struct SVGChartRenderer;

impl ChartRenderer for SVGChartRenderer {
    fn render_chart(&self, chart: &Chart) -> Result<String, Error> {
        // 実際の実装では、SVG形式でチャートをレンダリングする
        // ここでは簡易的な実装として、SVGのテンプレートを返す
        
        let width = chart.config.width.unwrap_or(800);
        let height = chart.config.height.unwrap_or(600);
        
        let svg = format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
                <title>{}</title>
                <rect width="100%" height="100%" fill="white"/>
                <text x="50%" y="30" font-family="Arial" font-size="20" text-anchor="middle">{}</text>
                <!-- ここに実際のチャートデータが描画される -->
            </svg>"#,
            width, height, chart.config.title, chart.config.title
        );
        
        Ok(svg)
    }
    
    fn render_ohlc_chart(&self, chart: &OHLCChart) -> Result<String, Error> {
        // 実際の実装では、SVG形式でOHLCチャートをレンダリングする
        // ここでは簡易的な実装として、SVGのテンプレートを返す
        
        let width = chart.config.width.unwrap_or(800);
        let height = chart.config.height.unwrap_or(600);
        
        let svg = format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
                <title>{}</title>
                <rect width="100%" height="100%" fill="white"/>
                <text x="50%" y="30" font-family="Arial" font-size="20" text-anchor="middle">{}</text>
                <!-- ここに実際のOHLCチャートデータが描画される -->
            </svg>"#,
            width, height, chart.config.title, chart.config.title
        );
        
        Ok(svg)
    }
}

/// HTMLチャートレンダラー
pub struct HTMLChartRenderer;

impl ChartRenderer for HTMLChartRenderer {
    fn render_chart(&self, chart: &Chart) -> Result<String, Error> {
        // 実際の実装では、HTML/JavaScript（例：Chart.js）を使用してチャートをレンダリングする
        // ここでは簡易的な実装として、HTMLのテンプレートを返す
        
        let width = chart.config.width.unwrap_or(800);
        let height = chart.config.height.unwrap_or(600);
        
        let html = format!(
            r#"<!DOCTYPE html>
            <html>
            <head>
                <title>{}</title>
                <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
            </head>
            <body>
                <div style="width:{}px;height:{}px;">
                    <canvas id="myChart"></canvas>
                </div>
                <script>
                    // ここに実際のChart.jsコードが入る
                    const ctx = document.getElementById('myChart');
                    new Chart(ctx, {{
                        type: '{}',
                        data: {{
                            // データ
                        }},
                        options: {{
                            // オプション
                        }}
                    }});
                </script>
            </body>
            </html>"#,
            chart.config.title, width, height, chart.config.chart_type.to_string().to_lowercase()
        );
        
        Ok(html)
    }
    
    fn render_ohlc_chart(&self, chart: &OHLCChart) -> Result<String, Error> {
        // 実際の実装では、HTML/JavaScript（例：TradingView Lightweight Charts）を使用してOHLCチャートをレンダリングする
        // ここでは簡易的な実装として、HTMLのテンプレートを返す
        
        let width = chart.config.width.unwrap_or(800);
        let height = chart.config.height.unwrap_or(600);
        
        let html = format!(
            r#"<!DOCTYPE html>
            <html>
            <head>
                <title>{}</title>
                <script src="https://unpkg.com/lightweight-charts/dist/lightweight-charts.standalone.production.js"></script>
            </head>
            <body>
                <div id="chart" style="width:{}px;height:{}px;"></div>
                <script>
                    // ここに実際のLightweight Chartsコードが入る
                    const chart = LightweightCharts.createChart(document.getElementById('chart'), {{
                        width: {},
                        height: {}
                    }});
                    
                    const candlestickSeries = chart.addCandlestickSeries();
                    // データ
                </script>
            </body>
            </html>"#,
            chart.config.title, width, height, width, height
        );
        
        Ok(html)
    }
}

/// チャートエクスポーター
pub trait ChartExporter {
    /// チャートをエクスポート
    fn export_chart(&self, chart: &Chart, format: &str) -> Result<Vec<u8>, Error>;
    
    /// OHLCチャートをエクスポート
    fn export_ohlc_chart(&self, chart: &OHLCChart, format: &str) -> Result<Vec<u8>, Error>;
}

/// 基本チャートエクスポーター
pub struct BasicChartExporter {
    /// レンダラー
    renderer: Box<dyn ChartRenderer>,
}

impl BasicChartExporter {
    /// 新しい基本チャートエクスポーターを作成
    pub fn new(renderer: Box<dyn ChartRenderer>) -> Self {
        Self { renderer }
    }
}

impl ChartExporter for BasicChartExporter {
    fn export_chart(&self, chart: &Chart, format: &str) -> Result<Vec<u8>, Error> {
        match format.to_lowercase().as_str() {
            "svg" => {
                let svg = self.renderer.render_chart(chart)?;
                Ok(svg.into_bytes())
            },
            "html" => {
                let html = self.renderer.render_chart(chart)?;
                Ok(html.into_bytes())
            },
            _ => Err(Error::InvalidInput(format!("未対応のエクスポート形式: {}", format))),
        }
    }
    
    fn export_ohlc_chart(&self, chart: &OHLCChart, format: &str) -> Result<Vec<u8>, Error> {
        match format.to_lowercase().as_str() {
            "svg" => {
                let svg = self.renderer.render_ohlc_chart(chart)?;
                Ok(svg.into_bytes())
            },
            "html" => {
                let html = self.renderer.render_ohlc_chart(chart)?;
                Ok(html.into_bytes())
            },
            _ => Err(Error::InvalidInput(format!("未対応のエクスポート形式: {}", format))),
        }
    }
}

/// チャートマネージャー
pub struct ChartManager {
    /// チャート
    charts: HashMap<String, Chart>,
    /// OHLCチャート
    ohlc_charts: HashMap<String, OHLCChart>,
    /// レンダラー
    renderer: Box<dyn ChartRenderer>,
    /// エクスポーター
    exporter: Box<dyn ChartExporter>,
}

impl ChartManager {
    /// 新しいチャートマネージャーを作成
    pub fn new() -> Self {
        let renderer = Box::new(SVGChartRenderer);
        let exporter = Box::new(BasicChartExporter::new(Box::new(SVGChartRenderer)));
        
        Self {
            charts: HashMap::new(),
            ohlc_charts: HashMap::new(),
            renderer,
            exporter,
        }
    }
    
    /// チャートを作成
    pub fn create_chart(&mut self, chart: Chart) -> Result<(), Error> {
        if self.charts.contains_key(&chart.config.id) {
            return Err(Error::AlreadyExists(format!("チャート {} は既に存在します", chart.config.id)));
        }
        
        self.charts.insert(chart.config.id.clone(), chart);
        
        Ok(())
    }
    
    /// OHLCチャートを作成
    pub fn create_ohlc_chart(&mut self, chart: OHLCChart) -> Result<(), Error> {
        if self.ohlc_charts.contains_key(&chart.config.id) {
            return Err(Error::AlreadyExists(format!("OHLCチャート {} は既に存在します", chart.config.id)));
        }
        
        self.ohlc_charts.insert(chart.config.id.clone(), chart);
        
        Ok(())
    }
    
    /// チャートを取得
    pub fn get_chart(&self, id: &str) -> Result<&Chart, Error> {
        self.charts.get(id)
            .ok_or_else(|| Error::NotFound(format!("チャート {} が見つかりません", id)))
    }
    
    /// OHLCチャートを取得
    pub fn get_ohlc_chart(&self, id: &str) -> Result<&OHLCChart, Error> {
        self.ohlc_charts.get(id)
            .ok_or_else(|| Error::NotFound(format!("OHLCチャート {} が見つかりません", id)))
    }
    
    /// チャートを削除
    pub fn delete_chart(&mut self, id: &str) -> Result<(), Error> {
        if self.charts.remove(id).is_none() {
            return Err(Error::NotFound(format!("チャート {} が見つかりません", id)));
        }
        
        Ok(())
    }
    
    /// OHLCチャートを削除
    pub fn delete_ohlc_chart(&mut self, id: &str) -> Result<(), Error> {
        if self.ohlc_charts.remove(id).is_none() {
            return Err(Error::NotFound(format!("OHLCチャート {} が見つかりません", id)));
        }
        
        Ok(())
    }
    
    /// チャートをレンダリング
    pub fn render_chart(&self, id: &str) -> Result<String, Error> {
        let chart = self.get_chart(id)?;
        self.renderer.render_chart(chart)
    }
    
    /// OHLCチャートをレンダリング
    pub fn render_ohlc_chart(&self, id: &str) -> Result<String, Error> {
        let chart = self.get_ohlc_chart(id)?;
        self.renderer.render_ohlc_chart(chart)
    }
    
    /// チャートをエクスポート
    pub fn export_chart(&self, id: &str, format: &str) -> Result<Vec<u8>, Error> {
        let chart = self.get_chart(id)?;
        self.exporter.export_chart(chart, format)
    }
    
    /// OHLCチャートをエクスポート
    pub fn export_ohlc_chart(&self, id: &str, format: &str) -> Result<Vec<u8>, Error> {
        let chart = self.get_ohlc_chart(id)?;
        self.exporter.export_ohlc_chart(chart, format)
    }
    
    /// 全チャートを取得
    pub fn get_all_charts(&self) -> Vec<&Chart> {
        self.charts.values().collect()
    }
    
    /// 全OHLCチャートを取得
    pub fn get_all_ohlc_charts(&self) -> Vec<&OHLCChart> {
        self.ohlc_charts.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chart_builder() {
        let chart = ChartBuilder::new("chart1", ChartType::Line, "テストチャート")
            .with_subtitle("サブタイトル")
            .with_x_axis_label("X軸")
            .with_y_axis_label("Y軸")
            .with_size(800, 600)
            .build();
        
        assert_eq!(chart.config.id, "chart1");
        assert_eq!(chart.config.chart_type, ChartType::Line);
        assert_eq!(chart.config.title, "テストチャート");
        assert_eq!(chart.config.subtitle, Some("サブタイトル".to_string()));
        assert_eq!(chart.config.x_axis_label, Some("X軸".to_string()));
        assert_eq!(chart.config.y_axis_label, Some("Y軸".to_string()));
        assert_eq!(chart.config.width, Some(800));
        assert_eq!(chart.config.height, Some(600));
    }
    
    #[test]
    fn test_ohlc_chart_builder() {
        let chart = OHLCChartBuilder::new("chart2", "OHLCテストチャート")
            .with_subtitle("サブタイトル")
            .with_x_axis_label("日付")
            .with_y_axis_label("価格")
            .with_size(800, 600)
            .build();
        
        assert_eq!(chart.config.id, "chart2");
        assert_eq!(chart.config.chart_type, ChartType::Candlestick);
        assert_eq!(chart.config.title, "OHLCテストチャート");
        assert_eq!(chart.config.subtitle, Some("サブタイトル".to_string()));
        assert_eq!(chart.config.x_axis_label, Some("日付".to_string()));
        assert_eq!(chart.config.y_axis_label, Some("価格".to_string()));
        assert_eq!(chart.config.width, Some(800));
        assert_eq!(chart.config.height, Some(600));
    }
    
    #[test]
    fn test_data_series_builder() {
        let now = Utc::now();
        
        let series = DataSeriesBuilder::new("series1", "テストシリーズ")
            .with_color("#FF0000")
            .with_line_style("solid")
            .add_data_point(now, 100.0)
            .add_data_point(now + Duration::hours(1), 110.0)
            .add_data_point(now + Duration::hours(2), 105.0)
            .build();
        
        assert_eq!(series.id, "series1");
        assert_eq!(series.name, "テストシリーズ");
        assert_eq!(series.color, Some("#FF0000".to_string()));
        assert_eq!(series.line_style, Some("solid".to_string()));
        assert_eq!(series.data_points.len(), 3);
        assert_eq!(series.data_points[0].value, 100.0);
        assert_eq!(series.data_points[1].value, 110.0);
        assert_eq!(series.data_points[2].value, 105.0);
    }
    
    #[test]
    fn test_ohlc_data_series_builder() {
        let now = Utc::now();
        
        let series = OHLCDataSeriesBuilder::new("series2", "OHLCテストシリーズ")
            .with_up_color("#00FF00")
            .with_down_color("#FF0000")
            .add_data_point(now, 100.0, 110.0, 95.0, 105.0)
            .add_data_point_with_volume(now + Duration::hours(1), 105.0, 115.0, 100.0, 110.0, 1000.0)
            .build();
        
        assert_eq!(series.id, "series2");
        assert_eq!(series.name, "OHLCテストシリーズ");
        assert_eq!(series.up_color, Some("#00FF00".to_string()));
        assert_eq!(series.down_color, Some("#FF0000".to_string()));
        assert_eq!(series.data_points.len(), 2);
        assert_eq!(series.data_points[0].open, 100.0);
        assert_eq!(series.data_points[0].high, 110.0);
        assert_eq!(series.data_points[0].low, 95.0);
        assert_eq!(series.data_points[0].close, 105.0);
        assert_eq!(series.data_points[0].volume, None);
        assert_eq!(series.data_points[1].volume, Some(1000.0));
    }
    
    #[test]
    fn test_chart_data_aggregator() {
        let now = Utc::now();
        
        // テスト用のデータポイントを作成
        let data_points = vec![
            DataPoint { timestamp: now, value: 100.0, metadata: None },
            DataPoint { timestamp: now + Duration::minutes(10), value: 110.0, metadata: None },
            DataPoint { timestamp: now + Duration::minutes(20), value: 105.0, metadata: None },
            DataPoint { timestamp: now + Duration::minutes(30), value: 115.0, metadata: None },
            DataPoint { timestamp: now + Duration::minutes(40), value: 120.0, metadata: None },
            DataPoint { timestamp: now + Duration::minutes(50), value: 125.0, metadata: None },
            DataPoint { timestamp: now + Duration::hours(1), value: 130.0, metadata: None },
        ];
        
        // 30分間隔で集約
        let aggregated = ChartDataAggregator::aggregate_by_timeframe(&data_points, &TimeFrame::Minute(30));
        
        assert_eq!(aggregated.len(), 3);
        assert_eq!(aggregated[0].value, 105.0); // (100 + 110 + 105) / 3
        assert_eq!(aggregated[1].value, 117.5); // (115 + 120) / 2
        assert_eq!(aggregated[2].value, 127.5); // (125 + 130) / 2
    }
    
    #[test]
    fn test_moving_average() {
        let now = Utc::now();
        
        // テスト用のデータポイントを作成
        let data_points = vec![
            DataPoint { timestamp: now, value: 100.0, metadata: None },
            DataPoint { timestamp: now + Duration::minutes(10), value: 110.0, metadata: None },
            DataPoint { timestamp: now + Duration::minutes(20), value: 105.0, metadata: None },
            DataPoint { timestamp: now + Duration::minutes(30), value: 115.0, metadata: None },
            DataPoint { timestamp: now + Duration::minutes(40), value: 120.0, metadata: None },
        ];
        
        // 3期間の移動平均を計算
        let ma = ChartDataAggregator::calculate_moving_average(&data_points, 3);
        
        assert_eq!(ma.len(), 3);
        assert_eq!(ma[0].value, 105.0); // (100 + 110 + 105) / 3
        assert_eq!(ma[1].value, 110.0); // (110 + 105 + 115) / 3
        assert_eq!(ma[2].value, 113.33333333333333); // (105 + 115 + 120) / 3
    }
    
    #[test]
    fn test_chart_manager() {
        let mut manager = ChartManager::new();
        
        // チャートを作成
        let chart = ChartBuilder::new("chart1", ChartType::Line, "テストチャート")
            .with_subtitle("サブタイトル")
            .build();
        
        manager.create_chart(chart).unwrap();
        
        // チャートを取得
        let retrieved_chart = manager.get_chart("chart1").unwrap();
        assert_eq!(retrieved_chart.config.title, "テストチャート");
        
        // OHLCチャートを作成
        let ohlc_chart = OHLCChartBuilder::new("chart2", "OHLCテストチャート")
            .with_subtitle("サブタイトル")
            .build();
        
        manager.create_ohlc_chart(ohlc_chart).unwrap();
        
        // OHLCチャートを取得
        let retrieved_ohlc_chart = manager.get_ohlc_chart("chart2").unwrap();
        assert_eq!(retrieved_ohlc_chart.config.title, "OHLCテストチャート");
        
        // 全チャートを取得
        let all_charts = manager.get_all_charts();
        assert_eq!(all_charts.len(), 1);
        
        // 全OHLCチャートを取得
        let all_ohlc_charts = manager.get_all_ohlc_charts();
        assert_eq!(all_ohlc_charts.len(), 1);
        
        // チャートを削除
        manager.delete_chart("chart1").unwrap();
        
        // OHLCチャートを削除
        manager.delete_ohlc_chart("chart2").unwrap();
        
        // 削除後の確認
        assert!(manager.get_chart("chart1").is_err());
        assert!(manager.get_ohlc_chart("chart2").is_err());
    }
}