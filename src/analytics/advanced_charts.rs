use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::analytics::chart::{ChartData, ChartType, Dataset, Annotation, AnnotationType};
use crate::analytics::metrics::{MetricType, MetricValue};

/// 高度なチャートタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdvancedChartType {
    /// ヒートマップ
    Heatmap,
    /// サンキーダイアグラム
    Sankey,
    /// ツリーマップ
    Treemap,
    /// サンバースト
    Sunburst,
    /// ネットワークグラフ
    Network,
    /// 3D散布図
    Scatter3D,
    /// 3D表面図
    Surface3D,
    /// パラレル座標
    ParallelCoordinates,
    /// ボックスプロット
    BoxPlot,
    /// バイオリンプロット
    ViolinPlot,
    /// ファネルチャート
    Funnel,
    /// ゲージチャート
    Gauge,
    /// レーダーチャート
    Radar,
    /// 極座標チャート
    Polar,
    /// ストリームグラフ
    Stream,
    /// カレンダーヒートマップ
    CalendarHeatmap,
    /// 複合チャート
    Composite,
}

/// 高度なチャートデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedChartData {
    /// チャートID
    pub id: String,
    /// チャートタイプ
    pub chart_type: AdvancedChartType,
    /// タイトル
    pub title: String,
    /// 説明
    pub description: Option<String>,
    /// データ
    pub data: ChartDataType,
    /// オプション
    pub options: AdvancedChartOptions,
    /// 生成時刻
    pub generated_at: DateTime<Utc>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// チャートデータタイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ChartDataType {
    /// 時系列データ
    TimeSeries(TimeSeriesData),
    /// カテゴリデータ
    Categorical(CategoricalData),
    /// 階層データ
    Hierarchical(HierarchicalData),
    /// ネットワークデータ
    Network(NetworkData),
    /// 地理データ
    Geo(GeoData),
    /// 多次元データ
    MultiDimensional(MultiDimensionalData),
    /// 分布データ
    Distribution(DistributionData),
    /// 相関データ
    Correlation(CorrelationData),
    /// フローデータ
    Flow(FlowData),
}

/// 時系列データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    /// 時間軸
    pub timestamps: Vec<DateTime<Utc>>,
    /// シリーズ
    pub series: HashMap<String, Vec<f64>>,
    /// 注釈
    pub annotations: Option<Vec<TimeAnnotation>>,
}

/// カテゴリデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoricalData {
    /// カテゴリ
    pub categories: Vec<String>,
    /// シリーズ
    pub series: HashMap<String, Vec<f64>>,
    /// 色
    pub colors: Option<HashMap<String, String>>,
}

/// 階層データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalData {
    /// ルートノード
    pub root: HierarchicalNode,
}

/// 階層ノード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalNode {
    /// 名前
    pub name: String,
    /// 値
    pub value: Option<f64>,
    /// 子ノード
    pub children: Option<Vec<HierarchicalNode>>,
    /// 色
    pub color: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// ネットワークデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkData {
    /// ノード
    pub nodes: Vec<NetworkNode>,
    /// エッジ
    pub edges: Vec<NetworkEdge>,
}

/// ネットワークノード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    /// ID
    pub id: String,
    /// ラベル
    pub label: String,
    /// サイズ
    pub size: Option<f64>,
    /// 色
    pub color: Option<String>,
    /// グループ
    pub group: Option<String>,
    /// X座標
    pub x: Option<f64>,
    /// Y座標
    pub y: Option<f64>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// ネットワークエッジ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEdge {
    /// 送信元
    pub source: String,
    /// 送信先
    pub target: String,
    /// 値
    pub value: Option<f64>,
    /// ラベル
    pub label: Option<String>,
    /// 色
    pub color: Option<String>,
    /// 太さ
    pub width: Option<f64>,
    /// 方向
    pub directed: Option<bool>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 地理データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoData {
    /// タイプ
    pub geo_type: GeoType,
    /// 特徴
    pub features: Vec<GeoFeature>,
}

/// 地理タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GeoType {
    /// 世界
    World,
    /// 国
    Country,
    /// 地域
    Region,
    /// 都市
    City,
    /// カスタム
    Custom,
}

/// 地理特徴
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoFeature {
    /// ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 値
    pub value: f64,
    /// 色
    pub color: Option<String>,
    /// 座標
    pub coordinates: Option<GeoCoordinates>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 地理座標
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoCoordinates {
    /// 緯度
    pub latitude: f64,
    /// 経度
    pub longitude: f64,
}

/// 多次元データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiDimensionalData {
    /// 次元
    pub dimensions: Vec<String>,
    /// データポイント
    pub points: Vec<MultiDimensionalPoint>,
}

/// 多次元ポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiDimensionalPoint {
    /// 値
    pub values: HashMap<String, f64>,
    /// ラベル
    pub label: Option<String>,
    /// 色
    pub color: Option<String>,
    /// サイズ
    pub size: Option<f64>,
    /// グループ
    pub group: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 分布データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionData {
    /// タイプ
    pub distribution_type: DistributionType,
    /// シリーズ
    pub series: HashMap<String, Vec<f64>>,
    /// 統計
    pub statistics: Option<HashMap<String, DistributionStatistics>>,
}

/// 分布タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DistributionType {
    /// ヒストグラム
    Histogram,
    /// ボックスプロット
    BoxPlot,
    /// バイオリンプロット
    ViolinPlot,
    /// 確率密度関数
    PDF,
    /// 累積分布関数
    CDF,
}

/// 分布統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStatistics {
    /// 最小値
    pub min: f64,
    /// 最大値
    pub max: f64,
    /// 平均値
    pub mean: f64,
    /// 中央値
    pub median: f64,
    /// 標準偏差
    pub std_dev: f64,
    /// 四分位数
    pub quartiles: [f64; 3],
    /// 外れ値
    pub outliers: Option<Vec<f64>>,
}

/// 相関データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationData {
    /// 変数
    pub variables: Vec<String>,
    /// 相関行列
    pub matrix: Vec<Vec<f64>>,
    /// P値
    pub p_values: Option<Vec<Vec<f64>>>,
}

/// フローデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowData {
    /// ノード
    pub nodes: Vec<FlowNode>,
    /// リンク
    pub links: Vec<FlowLink>,
}

/// フローノード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    /// ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 色
    pub color: Option<String>,
    /// グループ
    pub group: Option<String>,
}

/// フローリンク
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowLink {
    /// 送信元
    pub source: String,
    /// 送信先
    pub target: String,
    /// 値
    pub value: f64,
    /// 色
    pub color: Option<String>,
    /// ラベル
    pub label: Option<String>,
}

/// 時間注釈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAnnotation {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// タイプ
    pub annotation_type: AnnotationType,
    /// ラベル
    pub label: String,
    /// 説明
    pub description: Option<String>,
    /// 色
    pub color: Option<String>,
}

/// 高度なチャートオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedChartOptions {
    /// 幅
    pub width: Option<u32>,
    /// 高さ
    pub height: Option<u32>,
    /// マージン
    pub margin: Option<ChartMargin>,
    /// 色パレット
    pub color_palette: Option<Vec<String>>,
    /// 背景色
    pub background_color: Option<String>,
    /// フォント
    pub font: Option<String>,
    /// フォントサイズ
    pub font_size: Option<u32>,
    /// タイトルフォントサイズ
    pub title_font_size: Option<u32>,
    /// 凡例表示
    pub show_legend: Option<bool>,
    /// 凡例位置
    pub legend_position: Option<LegendPosition>,
    /// グリッド表示
    pub show_grid: Option<bool>,
    /// ツールチップ表示
    pub show_tooltip: Option<bool>,
    /// ズーム可能
    pub zoomable: Option<bool>,
    /// パン可能
    pub pannable: Option<bool>,
    /// アニメーション
    pub animation: Option<bool>,
    /// アニメーション期間（ミリ秒）
    pub animation_duration: Option<u32>,
    /// 3D表示
    pub three_dimensional: Option<bool>,
    /// 軸表示
    pub show_axes: Option<bool>,
    /// X軸ラベル
    pub x_axis_label: Option<String>,
    /// Y軸ラベル
    pub y_axis_label: Option<String>,
    /// Z軸ラベル
    pub z_axis_label: Option<String>,
    /// X軸目盛り回転
    pub x_axis_tick_rotation: Option<i32>,
    /// Y軸目盛り回転
    pub y_axis_tick_rotation: Option<i32>,
    /// X軸最小値
    pub x_axis_min: Option<f64>,
    /// X軸最大値
    pub x_axis_max: Option<f64>,
    /// Y軸最小値
    pub y_axis_min: Option<f64>,
    /// Y軸最大値
    pub y_axis_max: Option<f64>,
    /// Z軸最小値
    pub z_axis_min: Option<f64>,
    /// Z軸最大値
    pub z_axis_max: Option<f64>,
    /// 対数スケール
    pub log_scale: Option<bool>,
    /// 積み上げ
    pub stacked: Option<bool>,
    /// 正規化
    pub normalized: Option<bool>,
    /// 補間方法
    pub interpolation: Option<String>,
    /// シンボルサイズ
    pub symbol_size: Option<u32>,
    /// シンボル形状
    pub symbol_shape: Option<String>,
    /// 線の太さ
    pub line_width: Option<u32>,
    /// 線のスタイル
    pub line_style: Option<String>,
    /// 塗りつぶし透明度
    pub fill_opacity: Option<f64>,
    /// ラベル表示
    pub show_labels: Option<bool>,
    /// ラベル位置
    pub label_position: Option<String>,
    /// ラベルフォーマット
    pub label_format: Option<String>,
    /// 値フォーマット
    pub value_format: Option<String>,
    /// 日時フォーマット
    pub date_format: Option<String>,
    /// テーマ
    pub theme: Option<String>,
    /// レスポンシブ
    pub responsive: Option<bool>,
    /// 追加オプション
    #[serde(flatten)]
    pub additional_options: HashMap<String, serde_json::Value>,
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

/// 凡例位置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LegendPosition {
    /// 上
    Top,
    /// 右
    Right,
    /// 下
    Bottom,
    /// 左
    Left,
    /// 内部（上右）
    TopRight,
    /// 内部（上左）
    TopLeft,
    /// 内部（下右）
    BottomRight,
    /// 内部（下左）
    BottomLeft,
}

impl Default for AdvancedChartOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            margin: Some(ChartMargin {
                top: 50,
                right: 50,
                bottom: 70,
                left: 70,
            }),
            color_palette: None,
            background_color: Some("#ffffff".to_string()),
            font: Some("Arial, sans-serif".to_string()),
            font_size: Some(12),
            title_font_size: Some(18),
            show_legend: Some(true),
            legend_position: Some(LegendPosition::Right),
            show_grid: Some(true),
            show_tooltip: Some(true),
            zoomable: Some(false),
            pannable: Some(false),
            animation: Some(true),
            animation_duration: Some(1000),
            three_dimensional: Some(false),
            show_axes: Some(true),
            x_axis_label: None,
            y_axis_label: None,
            z_axis_label: None,
            x_axis_tick_rotation: Some(0),
            y_axis_tick_rotation: Some(0),
            x_axis_min: None,
            x_axis_max: None,
            y_axis_min: None,
            y_axis_max: None,
            z_axis_min: None,
            z_axis_max: None,
            log_scale: Some(false),
            stacked: Some(false),
            normalized: Some(false),
            interpolation: Some("linear".to_string()),
            symbol_size: Some(6),
            symbol_shape: Some("circle".to_string()),
            line_width: Some(2),
            line_style: Some("solid".to_string()),
            fill_opacity: Some(0.7),
            show_labels: Some(false),
            label_position: Some("auto".to_string()),
            label_format: None,
            value_format: None,
            date_format: Some("%Y-%m-%d %H:%M:%S".to_string()),
            theme: Some("light".to_string()),
            responsive: Some(true),
            additional_options: HashMap::new(),
        }
    }
}

/// 高度なチャート生成器
pub struct AdvancedChartGenerator {
    /// デフォルトオプション
    default_options: AdvancedChartOptions,
    /// 色パレット
    color_palettes: HashMap<String, Vec<String>>,
    /// チャートキャッシュ
    chart_cache: HashMap<String, AdvancedChartData>,
}

impl AdvancedChartGenerator {
    /// 新しい高度なチャート生成器を作成
    pub fn new() -> Self {
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
        
        // ビビッドパレット
        color_palettes.insert(
            "vivid".to_string(),
            vec![
                "#e41a1c".to_string(),
                "#377eb8".to_string(),
                "#4daf4a".to_string(),
                "#984ea3".to_string(),
                "#ff7f00".to_string(),
                "#ffff33".to_string(),
                "#a65628".to_string(),
                "#f781bf".to_string(),
                "#999999".to_string(),
                "#66c2a5".to_string(),
            ],
        );
        
        // モノクロパレット
        color_palettes.insert(
            "monochrome".to_string(),
            vec![
                "#000000".to_string(),
                "#252525".to_string(),
                "#525252".to_string(),
                "#737373".to_string(),
                "#969696".to_string(),
                "#bdbdbd".to_string(),
                "#d9d9d9".to_string(),
                "#f0f0f0".to_string(),
                "#ffffff".to_string(),
            ],
        );
        
        Self {
            default_options: AdvancedChartOptions::default(),
            color_palettes,
            chart_cache: HashMap::new(),
        }
    }
    
    /// 時系列チャートを作成
    pub fn create_time_series_chart(
        &mut self,
        id: &str,
        title: &str,
        description: Option<&str>,
        timestamps: Vec<DateTime<Utc>>,
        series_data: HashMap<String, Vec<f64>>,
        options: Option<AdvancedChartOptions>,
    ) -> Result<AdvancedChartData, Error> {
        // オプションを設定
        let mut chart_options = options.unwrap_or_else(|| self.default_options.clone());
        
        // X軸とY軸のラベルを設定
        if chart_options.x_axis_label.is_none() {
            chart_options.x_axis_label = Some("時間".to_string());
        }
        
        if chart_options.y_axis_label.is_none() {
            chart_options.y_axis_label = Some("値".to_string());
        }
        
        // データを作成
        let data = ChartDataType::TimeSeries(TimeSeriesData {
            timestamps,
            series: series_data,
            annotations: None,
        });
        
        // チャートを作成
        let chart = AdvancedChartData {
            id: id.to_string(),
            chart_type: AdvancedChartType::Composite,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            data,
            options: chart_options,
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // キャッシュに保存
        self.chart_cache.insert(id.to_string(), chart.clone());
        
        Ok(chart)
    }
    
    /// ヒートマップチャートを作成
    pub fn create_heatmap_chart(
        &mut self,
        id: &str,
        title: &str,
        description: Option<&str>,
        x_categories: Vec<String>,
        y_categories: Vec<String>,
        values: Vec<Vec<f64>>,
        options: Option<AdvancedChartOptions>,
    ) -> Result<AdvancedChartData, Error> {
        // データの検証
        if values.len() != y_categories.len() {
            return Err(Error::InvalidInput(format!(
                "Values rows count ({}) does not match y_categories count ({})",
                values.len(),
                y_categories.len()
            )));
        }
        
        for (i, row) in values.iter().enumerate() {
            if row.len() != x_categories.len() {
                return Err(Error::InvalidInput(format!(
                    "Values row {} length ({}) does not match x_categories count ({})",
                    i,
                    row.len(),
                    x_categories.len()
                )));
            }
        }
        
        // 多次元データを作成
        let mut points = Vec::new();
        
        for (y_idx, y_category) in y_categories.iter().enumerate() {
            for (x_idx, x_category) in x_categories.iter().enumerate() {
                let value = values[y_idx][x_idx];
                
                let mut values_map = HashMap::new();
                values_map.insert("x".to_string(), x_idx as f64);
                values_map.insert("y".to_string(), y_idx as f64);
                values_map.insert("value".to_string(), value);
                
                let point = MultiDimensionalPoint {
                    values: values_map,
                    label: Some(format!("{}: {}", x_category, y_category)),
                    color: None, // 値に基づいて色が自動的に割り当てられる
                    size: None,
                    group: None,
                    metadata: Some({
                        let mut map = HashMap::new();
                        map.insert("x_category".to_string(), x_category.clone());
                        map.insert("y_category".to_string(), y_category.clone());
                        map
                    }),
                };
                
                points.push(point);
            }
        }
        
        let data = ChartDataType::MultiDimensional(MultiDimensionalData {
            dimensions: vec!["x".to_string(), "y".to_string(), "value".to_string()],
            points,
        });
        
        // オプションを設定
        let mut chart_options = options.unwrap_or_else(|| self.default_options.clone());
        
        // X軸とY軸のラベルを設定
        if chart_options.x_axis_label.is_none() {
            chart_options.x_axis_label = Some("X軸".to_string());
        }
        
        if chart_options.y_axis_label.is_none() {
            chart_options.y_axis_label = Some("Y軸".to_string());
        }
        
        // チャートを作成
        let chart = AdvancedChartData {
            id: id.to_string(),
            chart_type: AdvancedChartType::Heatmap,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            data,
            options: chart_options,
            generated_at: Utc::now(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("x_categories".to_string(), x_categories.join(","));
                map.insert("y_categories".to_string(), y_categories.join(","));
                map
            },
        };
        
        // キャッシュに保存
        self.chart_cache.insert(id.to_string(), chart.clone());
        
        Ok(chart)
    }
    
    /// ネットワークチャートを作成
    pub fn create_network_chart(
        &mut self,
        id: &str,
        title: &str,
        description: Option<&str>,
        nodes: Vec<NetworkNode>,
        edges: Vec<NetworkEdge>,
        options: Option<AdvancedChartOptions>,
    ) -> Result<AdvancedChartData, Error> {
        // データの検証
        let node_ids: std::collections::HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();
        
        for edge in &edges {
            if !node_ids.contains(&edge.source) {
                return Err(Error::InvalidInput(format!("Source node not found: {}", edge.source)));
            }
            
            if !node_ids.contains(&edge.target) {
                return Err(Error::InvalidInput(format!("Target node not found: {}", edge.target)));
            }
        }
        
        // データを作成
        let data = ChartDataType::Network(NetworkData {
            nodes,
            edges,
        });
        
        // オプションを設定
        let chart_options = options.unwrap_or_else(|| self.default_options.clone());
        
        // チャートを作成
        let chart = AdvancedChartData {
            id: id.to_string(),
            chart_type: AdvancedChartType::Network,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            data,
            options: chart_options,
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // キャッシュに保存
        self.chart_cache.insert(id.to_string(), chart.clone());
        
        Ok(chart)
    }
    
    /// サンキーダイアグラムを作成
    pub fn create_sankey_chart(
        &mut self,
        id: &str,
        title: &str,
        description: Option<&str>,
        nodes: Vec<FlowNode>,
        links: Vec<FlowLink>,
        options: Option<AdvancedChartOptions>,
    ) -> Result<AdvancedChartData, Error> {
        // データの検証
        let node_ids: std::collections::HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();
        
        for link in &links {
            if !node_ids.contains(&link.source) {
                return Err(Error::InvalidInput(format!("Source node not found: {}", link.source)));
            }
            
            if !node_ids.contains(&link.target) {
                return Err(Error::InvalidInput(format!("Target node not found: {}", link.target)));
            }
        }
        
        // データを作成
        let data = ChartDataType::Flow(FlowData {
            nodes,
            links,
        });
        
        // オプションを設定
        let chart_options = options.unwrap_or_else(|| self.default_options.clone());
        
        // チャートを作成
        let chart = AdvancedChartData {
            id: id.to_string(),
            chart_type: AdvancedChartType::Sankey,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            data,
            options: chart_options,
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // キャッシュに保存
        self.chart_cache.insert(id.to_string(), chart.clone());
        
        Ok(chart)
    }
    
    /// ツリーマップを作成
    pub fn create_treemap_chart(
        &mut self,
        id: &str,
        title: &str,
        description: Option<&str>,
        root: HierarchicalNode,
        options: Option<AdvancedChartOptions>,
    ) -> Result<AdvancedChartData, Error> {
        // データを作成
        let data = ChartDataType::Hierarchical(HierarchicalData {
            root,
        });
        
        // オプションを設定
        let chart_options = options.unwrap_or_else(|| self.default_options.clone());
        
        // チャートを作成
        let chart = AdvancedChartData {
            id: id.to_string(),
            chart_type: AdvancedChartType::Treemap,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            data,
            options: chart_options,
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // キャッシュに保存
        self.chart_cache.insert(id.to_string(), chart.clone());
        
        Ok(chart)
    }
    
    /// サンバーストチャートを作成
    pub fn create_sunburst_chart(
        &mut self,
        id: &str,
        title: &str,
        description: Option<&str>,
        root: HierarchicalNode,
        options: Option<AdvancedChartOptions>,
    ) -> Result<AdvancedChartData, Error> {
        // データを作成
        let data = ChartDataType::Hierarchical(HierarchicalData {
            root,
        });
        
        // オプションを設定
        let chart_options = options.unwrap_or_else(|| self.default_options.clone());
        
        // チャートを作成
        let chart = AdvancedChartData {
            id: id.to_string(),
            chart_type: AdvancedChartType::Sunburst,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            data,
            options: chart_options,
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // キャッシュに保存
        self.chart_cache.insert(id.to_string(), chart.clone());
        
        Ok(chart)
    }
    
    /// 3D散布図を作成
    pub fn create_scatter3d_chart(
        &mut self,
        id: &str,
        title: &str,
        description: Option<&str>,
        points: Vec<MultiDimensionalPoint>,
        options: Option<AdvancedChartOptions>,
    ) -> Result<AdvancedChartData, Error> {
        // データの検証
        for point in &points {
            if !point.values.contains_key("x") || !point.values.contains_key("y") || !point.values.contains_key("z") {
                return Err(Error::InvalidInput("Each point must have x, y, and z values".to_string()));
            }
        }
        
        // データを作成
        let data = ChartDataType::MultiDimensional(MultiDimensionalData {
            dimensions: vec!["x".to_string(), "y".to_string(), "z".to_string()],
            points,
        });
        
        // オプションを設定
        let mut chart_options = options.unwrap_or_else(|| self.default_options.clone());
        chart_options.three_dimensional = Some(true);
        
        // X軸、Y軸、Z軸のラベルを設定
        if chart_options.x_axis_label.is_none() {
            chart_options.x_axis_label = Some("X軸".to_string());
        }
        
        if chart_options.y_axis_label.is_none() {
            chart_options.y_axis_label = Some("Y軸".to_string());
        }
        
        if chart_options.z_axis_label.is_none() {
            chart_options.z_axis_label = Some("Z軸".to_string());
        }
        
        // チャートを作成
        let chart = AdvancedChartData {
            id: id.to_string(),
            chart_type: AdvancedChartType::Scatter3D,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            data,
            options: chart_options,
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // キャッシュに保存
        self.chart_cache.insert(id.to_string(), chart.clone());
        
        Ok(chart)
    }
    
    /// ボックスプロットを作成
    pub fn create_boxplot_chart(
        &mut self,
        id: &str,
        title: &str,
        description: Option<&str>,
        categories: Vec<String>,
        series_data: HashMap<String, Vec<f64>>,
        options: Option<AdvancedChartOptions>,
    ) -> Result<AdvancedChartData, Error> {
        // 統計を計算
        let mut statistics = HashMap::new();
        
        for (series_name, values) in &series_data {
            if values.is_empty() {
                continue;
            }
            
            // 値をソート
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            
            // 統計を計算
            let min = *sorted_values.first().unwrap();
            let max = *sorted_values.last().unwrap();
            let len = sorted_values.len();
            let mean = sorted_values.iter().sum::<f64>() / len as f64;
            
            // 中央値
            let median = if len % 2 == 0 {
                (sorted_values[len / 2 - 1] + sorted_values[len / 2]) / 2.0
            } else {
                sorted_values[len / 2]
            };
            
            // 四分位数
            let q1_idx = len / 4;
            let q3_idx = len * 3 / 4;
            let q1 = sorted_values[q1_idx];
            let q3 = sorted_values[q3_idx];
            
            // 標準偏差
            let variance = sorted_values.iter()
                .map(|v| (*v - mean) * (*v - mean))
                .sum::<f64>() / len as f64;
            let std_dev = variance.sqrt();
            
            // 外れ値を検出
            let iqr = q3 - q1;
            let lower_bound = q1 - 1.5 * iqr;
            let upper_bound = q3 + 1.5 * iqr;
            
            let outliers: Vec<f64> = sorted_values.iter()
                .filter(|v| **v < lower_bound || **v > upper_bound)
                .cloned()
                .collect();
            
            statistics.insert(series_name.clone(), DistributionStatistics {
                min,
                max,
                mean,
                median,
                std_dev,
                quartiles: [q1, median, q3],
                outliers: if outliers.is_empty() { None } else { Some(outliers) },
            });
        }
        
        // データを作成
        let data = ChartDataType::Distribution(DistributionData {
            distribution_type: DistributionType::BoxPlot,
            series: series_data,
            statistics: Some(statistics),
        });
        
        // オプションを設定
        let mut chart_options = options.unwrap_or_else(|| self.default_options.clone());
        
        // X軸とY軸のラベルを設定
        if chart_options.x_axis_label.is_none() {
            chart_options.x_axis_label = Some("カテゴリ".to_string());
        }
        
        if chart_options.y_axis_label.is_none() {
            chart_options.y_axis_label = Some("値".to_string());
        }
        
        // チャートを作成
        let chart = AdvancedChartData {
            id: id.to_string(),
            chart_type: AdvancedChartType::BoxPlot,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            data,
            options: chart_options,
            generated_at: Utc::now(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("categories".to_string(), categories.join(","));
                map
            },
        };
        
        // キャッシュに保存
        self.chart_cache.insert(id.to_string(), chart.clone());
        
        Ok(chart)
    }
    
    /// レーダーチャートを作成
    pub fn create_radar_chart(
        &mut self,
        id: &str,
        title: &str,
        description: Option<&str>,
        categories: Vec<String>,
        series_data: HashMap<String, Vec<f64>>,
        options: Option<AdvancedChartOptions>,
    ) -> Result<AdvancedChartData, Error> {
        // データの検証
        for (series_name, values) in &series_data {
            if values.len() != categories.len() {
                return Err(Error::InvalidInput(format!(
                    "Series {} values count ({}) does not match categories count ({})",
                    series_name,
                    values.len(),
                    categories.len()
                )));
            }
        }
        
        // データを作成
        let data = ChartDataType::Categorical(CategoricalData {
            categories,
            series: series_data,
            colors: None,
        });
        
        // オプションを設定
        let chart_options = options.unwrap_or_else(|| self.default_options.clone());
        
        // チャートを作成
        let chart = AdvancedChartData {
            id: id.to_string(),
            chart_type: AdvancedChartType::Radar,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            data,
            options: chart_options,
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // キャッシュに保存
        self.chart_cache.insert(id.to_string(), chart.clone());
        
        Ok(chart)
    }
    
    /// 相関行列を作成
    pub fn create_correlation_matrix(
        &mut self,
        id: &str,
        title: &str,
        description: Option<&str>,
        variables: Vec<String>,
        matrix: Vec<Vec<f64>>,
        p_values: Option<Vec<Vec<f64>>>,
        options: Option<AdvancedChartOptions>,
    ) -> Result<AdvancedChartData, Error> {
        // データの検証
        if matrix.len() != variables.len() {
            return Err(Error::InvalidInput(format!(
                "Matrix rows count ({}) does not match variables count ({})",
                matrix.len(),
                variables.len()
            )));
        }
        
        for (i, row) in matrix.iter().enumerate() {
            if row.len() != variables.len() {
                return Err(Error::InvalidInput(format!(
                    "Matrix row {} length ({}) does not match variables count ({})",
                    i,
                    row.len(),
                    variables.len()
                )));
            }
        }
        
        if let Some(p_values) = &p_values {
            if p_values.len() != variables.len() {
                return Err(Error::InvalidInput(format!(
                    "P-values rows count ({}) does not match variables count ({})",
                    p_values.len(),
                    variables.len()
                )));
            }
            
            for (i, row) in p_values.iter().enumerate() {
                if row.len() != variables.len() {
                    return Err(Error::InvalidInput(format!(
                        "P-values row {} length ({}) does not match variables count ({})",
                        i,
                        row.len(),
                        variables.len()
                    )));
                }
            }
        }
        
        // データを作成
        let data = ChartDataType::Correlation(CorrelationData {
            variables,
            matrix,
            p_values,
        });
        
        // オプションを設定
        let chart_options = options.unwrap_or_else(|| self.default_options.clone());
        
        // チャートを作成
        let chart = AdvancedChartData {
            id: id.to_string(),
            chart_type: AdvancedChartType::Heatmap,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            data,
            options: chart_options,
            generated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // キャッシュに保存
        self.chart_cache.insert(id.to_string(), chart.clone());
        
        Ok(chart)
    }
    
    /// チャートをHTMLに変換
    pub fn to_html(&self, chart_id: &str) -> Result<String, Error> {
        let chart = self.chart_cache.get(chart_id).ok_or_else(|| {
            Error::NotFound(format!("Chart not found: {}", chart_id))
        })?;
        
        // チャートデータをJSON文字列に変換
        let chart_json = serde_json::to_string(chart)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize chart: {}", e)))?;
        
        // チャートタイプに応じたライブラリを選択
        let (library, render_function) = match chart.chart_type {
            AdvancedChartType::Network => ("vis-network", "renderNetworkChart"),
            AdvancedChartType::Sankey => ("d3-sankey", "renderSankeyChart"),
            AdvancedChartType::Treemap | AdvancedChartType::Sunburst => ("d3-hierarchy", "renderHierarchyChart"),
            AdvancedChartType::Scatter3D | AdvancedChartType::Surface3D => ("plotly", "renderPlotlyChart"),
            AdvancedChartType::Heatmap => ("echarts", "renderHeatmapChart"),
            AdvancedChartType::BoxPlot | AdvancedChartType::ViolinPlot => ("echarts", "renderDistributionChart"),
            AdvancedChartType::Radar => ("echarts", "renderRadarChart"),
            AdvancedChartType::ParallelCoordinates => ("d3-parcoords", "renderParallelCoordinatesChart"),
            _ => ("echarts", "renderChart"),
        };
        
        // HTMLテンプレートを作成
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}
        #chart-container {{ width: 100%; height: 600px; }}
    </style>
    <!-- ECharts -->
    <script src="https://cdn.jsdelivr.net/npm/echarts@5.4.3/dist/echarts.min.js"></script>
    <!-- Plotly.js -->
    <script src="https://cdn.plot.ly/plotly-2.24.1.min.js"></script>
    <!-- D3.js -->
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <!-- vis-network -->
    <script src="https://unpkg.com/vis-network/standalone/umd/vis-network.min.js"></script>
    <!-- Additional libraries based on chart type -->
    <script>
        // Chart data
        const chartData = {};
        
        // Render function
        function {}(container, data) {{
            // Implementation depends on chart type
            switch(data.chart_type) {{
                case "Heatmap":
                    renderHeatmapChart(container, data);
                    break;
                case "Sankey":
                    renderSankeyChart(container, data);
                    break;
                case "Treemap":
                    renderTreemapChart(container, data);
                    break;
                case "Sunburst":
                    renderSunburstChart(container, data);
                    break;
                case "Network":
                    renderNetworkChart(container, data);
                    break;
                case "Scatter3D":
                    renderScatter3DChart(container, data);
                    break;
                case "BoxPlot":
                    renderBoxPlotChart(container, data);
                    break;
                case "Radar":
                    renderRadarChart(container, data);
                    break;
                case "Composite":
                    renderCompositeChart(container, data);
                    break;
                default:
                    renderDefaultChart(container, data);
            }}
        }}
        
        // Render heatmap chart using ECharts
        function renderHeatmapChart(container, data) {{
            const chart = echarts.init(container);
            
            // Extract data based on data type
            let xCategories = [];
            let yCategories = [];
            let values = [];
            
            if (data.data.type === "MultiDimensional") {{
                // Extract unique x and y values
                const xValues = new Set();
                const yValues = new Set();
                
                data.data.data.points.forEach(point => {{
                    if (point.metadata) {{
                        if (point.metadata.x_category) xValues.add(point.metadata.x_category);
                        if (point.metadata.y_category) yValues.add(point.metadata.y_category);
                    }}
                }});
                
                xCategories = Array.from(xValues);
                yCategories = Array.from(yValues);
                
                // Create values array
                data.data.data.points.forEach(point => {{
                    values.push([
                        xCategories.indexOf(point.metadata.x_category),
                        yCategories.indexOf(point.metadata.y_category),
                        point.values.value
                    ]);
                }});
            }} else if (data.data.type === "Correlation") {{
                xCategories = data.data.data.variables;
                yCategories = data.data.data.variables;
                
                for (let i = 0; i < data.data.data.matrix.length; i++) {{
                    for (let j = 0; j < data.data.data.matrix[i].length; j++) {{
                        values.push([j, i, data.data.data.matrix[i][j]]);
                    }}
                }}
            }}
            
            const option = {{
                title: {{
                    text: data.title,
                    left: 'center'
                }},
                tooltip: {{
                    position: 'top',
                    formatter: function (params) {{
                        return `${yCategories[params.value[1]]}, ${xCategories[params.value[0]]}: ${params.value[2].toFixed(2)}`;
                    }}
                }},
                grid: {{
                    height: '70%',
                    top: '10%'
                }},
                xAxis: {{
                    type: 'category',
                    data: xCategories,
                    splitArea: {{
                        show: true
                    }},
                    axisLabel: {{
                        rotate: data.options.x_axis_tick_rotation || 0
                    }}
                }},
                yAxis: {{
                    type: 'category',
                    data: yCategories,
                    splitArea: {{
                        show: true
                    }}
                }},
                visualMap: {{
                    min: Math.min(...values.map(v => v[2])),
                    max: Math.max(...values.map(v => v[2])),
                    calculable: true,
                    orient: 'horizontal',
                    left: 'center',
                    bottom: '5%'
                }},
                series: [{{
                    name: data.title,
                    type: 'heatmap',
                    data: values,
                    label: {{
                        show: data.options.show_labels || false
                    }},
                    emphasis: {{
                        itemStyle: {{
                            shadowBlur: 10,
                            shadowColor: 'rgba(0, 0, 0, 0.5)'
                        }}
                    }}
                }}]
            }};
            
            chart.setOption(option);
            
            // Handle responsive resizing
            window.addEventListener('resize', function() {{
                chart.resize();
            }});
        }}
        
        // Render network chart using vis-network
        function renderNetworkChart(container, data) {{
            if (data.data.type !== "Network") return;
            
            const nodes = data.data.data.nodes.map(node => ({{
                id: node.id,
                label: node.label,
                color: node.color,
                size: node.size || 25,
                group: node.group,
                x: node.x,
                y: node.y,
                title: node.metadata ? JSON.stringify(node.metadata) : undefined
            }}));
            
            const edges = data.data.data.edges.map(edge => ({{
                from: edge.source,
                to: edge.target,
                value: edge.value,
                label: edge.label,
                color: edge.color,
                width: edge.width,
                arrows: edge.directed ? 'to' : undefined,
                title: edge.metadata ? JSON.stringify(edge.metadata) : undefined
            }}));
            
            const network = new vis.Network(container, {{
                nodes: new vis.DataSet(nodes),
                edges: new vis.DataSet(edges)
            }}, {{
                physics: {{
                    stabilization: true,
                    barnesHut: {{
                        gravitationalConstant: -80000,
                        centralGravity: 0.3,
                        springLength: 95,
                        springConstant: 0.04,
                        damping: 0.09,
                        avoidOverlap: 0.1
                    }}
                }},
                interaction: {{
                    tooltipDelay: 200,
                    hideEdgesOnDrag: true,
                    multiselect: true
                }}
            }});
        }}
        
        // Render sankey chart using D3
        function renderSankeyChart(container, data) {{
            if (data.data.type !== "Flow") return;
            
            const width = container.clientWidth;
            const height = container.clientHeight;
            
            const svg = d3.select(container)
                .append("svg")
                .attr("width", width)
                .attr("height", height);
            
            const sankey = d3.sankey()
                .nodeWidth(15)
                .nodePadding(10)
                .extent([[1, 1], [width - 1, height - 5]]);
            
            const graph = {{
                nodes: data.data.data.nodes.map(node => ({{
                    name: node.name,
                    id: node.id,
                    color: node.color
                }})),
                links: data.data.data.links.map(link => ({{
                    source: link.source,
                    target: link.target,
                    value: link.value,
                    color: link.color
                }}))
            }};
            
            // Convert node IDs to indices
            const nodeMap = new Map();
            graph.nodes.forEach((node, i) => {{
                nodeMap.set(node.id, i);
                node.index = i;
            }});
            
            graph.links.forEach(link => {{
                link.source = nodeMap.get(link.source);
                link.target = nodeMap.get(link.target);
            }});
            
            const {{ nodes, links }} = sankey(graph);
            
            svg.append("g")
                .selectAll("rect")
                .data(nodes)
                .join("rect")
                .attr("x", d => d.x0)
                .attr("y", d => d.y0)
                .attr("height", d => d.y1 - d.y0)
                .attr("width", d => d.x1 - d.x0)
                .attr("fill", d => d.color || "#69b3a2")
                .attr("stroke", "#000");
            
            svg.append("g")
                .attr("fill", "none")
                .selectAll("g")
                .data(links)
                .join("path")
                .attr("d", d3.sankeyLinkHorizontal())
                .attr("stroke", d => d.color || "#aaa")
                .attr("stroke-width", d => Math.max(1, d.width));
            
            svg.append("g")
                .style("font", "10px sans-serif")
                .selectAll("text")
                .data(nodes)
                .join("text")
                .attr("x", d => d.x0 < width / 2 ? d.x1 + 6 : d.x0 - 6)
                .attr("y", d => (d.y1 + d.y0) / 2)
                .attr("dy", "0.35em")
                .attr("text-anchor", d => d.x0 < width / 2 ? "start" : "end")
                .text(d => d.name);
        }}
        
        // Render treemap chart using ECharts
        function renderTreemapChart(container, data) {{
            if (data.data.type !== "Hierarchical") return;
            
            const chart = echarts.init(container);
            
            // Convert hierarchical data to ECharts format
            function convertNode(node) {{
                const result = {{
                    name: node.name,
                    value: node.value,
                    itemStyle: node.color ? {{ color: node.color }} : undefined
                }};
                
                if (node.children && node.children.length > 0) {{
                    result.children = node.children.map(convertNode);
                }}
                
                return result;
            }}
            
            const treeData = convertNode(data.data.data.root);
            
            const option = {{
                title: {{
                    text: data.title,
                    left: 'center'
                }},
                tooltip: {{
                    formatter: function (info) {{
                        return [
                            '<div class="tooltip-title">' + info.name + '</div>',
                            'Value: ' + info.value
                        ].join('');
                    }}
                }},
                series: [{{
                    type: 'treemap',
                    data: [treeData],
                    label: {{
                        show: true
                    }},
                    upperLabel: {{
                        show: true,
                        height: 30
                    }},
                    itemStyle: {{
                        borderColor: '#fff'
                    }},
                    levels: [
                        {{
                            itemStyle: {{
                                borderWidth: 0,
                                gapWidth: 5
                            }}
                        }},
                        {{
                            itemStyle: {{
                                borderWidth: 5,
                                gapWidth: 1
                            }}
                        }},
                        {{
                            colorSaturation: [0.35, 0.5],
                            itemStyle: {{
                                borderWidth: 5,
                                gapWidth: 1,
                                borderColorSaturation: 0.6
                            }}
                        }}
                    ]
                }}]
            }};
            
            chart.setOption(option);
            
            // Handle responsive resizing
            window.addEventListener('resize', function() {{
                chart.resize();
            }});
        }}
        
        // Render sunburst chart using ECharts
        function renderSunburstChart(container, data) {{
            if (data.data.type !== "Hierarchical") return;
            
            const chart = echarts.init(container);
            
            // Convert hierarchical data to ECharts format
            function convertNode(node) {{
                const result = {{
                    name: node.name,
                    value: node.value,
                    itemStyle: node.color ? {{ color: node.color }} : undefined
                }};
                
                if (node.children && node.children.length > 0) {{
                    result.children = node.children.map(convertNode);
                }}
                
                return result;
            }}
            
            const sunburstData = convertNode(data.data.data.root);
            
            const option = {{
                title: {{
                    text: data.title,
                    left: 'center'
                }},
                tooltip: {{
                    formatter: function (info) {{
                        return [
                            '<div class="tooltip-title">' + info.name + '</div>',
                            'Value: ' + info.value
                        ].join('');
                    }}
                }},
                series: [{{
                    type: 'sunburst',
                    data: [sunburstData],
                    radius: ['20%', '90%'],
                    label: {{
                        rotate: 'radial'
                    }}
                }}]
            }};
            
            chart.setOption(option);
            
            // Handle responsive resizing
            window.addEventListener('resize', function() {{
                chart.resize();
            }});
        }}
        
        // Render 3D scatter chart using Plotly
        function renderScatter3DChart(container, data) {{
            if (data.data.type !== "MultiDimensional") return;
            
            // Group points by group if available
            const groups = new Map();
            
            data.data.data.points.forEach(point => {{
                const group = point.group || 'default';
                if (!groups.has(group)) {{
                    groups.set(group, {{
                        x: [],
                        y: [],
                        z: [],
                        text: [],
                        marker: {{
                            size: [],
                            color: []
                        }}
                    }});
                }}
                
                const groupData = groups.get(group);
                groupData.x.push(point.values.x);
                groupData.y.push(point.values.y);
                groupData.z.push(point.values.z);
                groupData.text.push(point.label || '');
                groupData.marker.size.push(point.size || 6);
                groupData.marker.color.push(point.color || '#1f77b4');
            }});
            
            const traces = Array.from(groups.entries()).map(([group, data]) => ({{
                type: 'scatter3d',
                mode: 'markers',
                name: group === 'default' ? 'Points' : group,
                x: data.x,
                y: data.y,
                z: data.z,
                text: data.text,
                marker: {{
                    size: data.marker.size,
                    color: data.marker.color,
                    opacity: 0.8
                }}
            }}));
            
            const layout = {{
                title: data.title,
                scene: {{
                    xaxis: {{ title: data.options.x_axis_label || 'X' }},
                    yaxis: {{ title: data.options.y_axis_label || 'Y' }},
                    zaxis: {{ title: data.options.z_axis_label || 'Z' }}
                }},
                margin: {{
                    l: 0,
                    r: 0,
                    b: 0,
                    t: 50
                }}
            }};
            
            Plotly.newPlot(container, traces, layout);
        }}
        
        // Render box plot chart using ECharts
        function renderBoxPlotChart(container, data) {{
            if (data.data.type !== "Distribution") return;
            
            const chart = echarts.init(container);
            
            // Extract data
            const seriesNames = Object.keys(data.data.data.series);
            const categories = data.metadata.categories.split(',');
            
            // Prepare box plot data
            const boxData = [];
            const outliers = [];
            
            seriesNames.forEach(seriesName => {{
                const stats = data.data.data.statistics[seriesName];
                
                // Box data format: [min, Q1, median, Q3, max]
                boxData.push([
                    stats.min,
                    stats.quartiles[0],
                    stats.quartiles[1],
                    stats.quartiles[2],
                    stats.max
                ]);
                
                // Add outliers if any
                if (stats.outliers && stats.outliers.length > 0) {{
                    stats.outliers.forEach(value => {{
                        outliers.push([seriesNames.indexOf(seriesName), value]);
                    }});
                }}
            }});
            
            const option = {{
                title: {{
                    text: data.title,
                    left: 'center'
                }},
                tooltip: {{
                    trigger: 'item',
                    formatter: function(params) {{
                        if (params.seriesIndex === 1) {{
                            // Outlier point
                            return `${seriesNames[params.data[0]]}: ${params.data[1]} (outlier)`;
                        }}
                        
                        // Box plot
                        return `${seriesNames[params.dataIndex]}:
                            <br>Maximum: ${params.data[4]}
                            <br>Upper Quartile: ${params.data[3]}
                            <br>Median: ${params.data[2]}
                            <br>Lower Quartile: ${params.data[1]}
                            <br>Minimum: ${params.data[0]}`;
                    }}
                }},
                grid: {{
                    left: '10%',
                    right: '10%',
                    bottom: '15%'
                }},
                xAxis: {{
                    type: 'category',
                    data: seriesNames,
                    boundaryGap: true,
                    nameGap: 30,
                    splitArea: {{
                        show: false
                    }},
                    axisLabel: {{
                        formatter: '{value}'
                    }},
                    splitLine: {{
                        show: false
                    }}
                }},
                yAxis: {{
                    type: 'value',
                    name: data.options.y_axis_label || 'Value',
                    splitArea: {{
                        show: true
                    }}
                }},
                series: [
                    {{
                        name: 'BoxPlot',
                        type: 'boxplot',
                        datasetIndex: 0,
                        data: boxData,
                        tooltip: {{
                            formatter: function(param) {{
                                return [
                                    `${seriesNames[param.dataIndex]}:`,
                                    `Maximum: ${param.data[4]}`,
                                    `Upper Quartile: ${param.data[3]}`,
                                    `Median: ${param.data[2]}`,
                                    `Lower Quartile: ${param.data[1]}`,
                                    `Minimum: ${param.data[0]}`
                                ].join('<br/>');
                            }}
                        }}
                    }},
                    {{
                        name: 'Outliers',
                        type: 'scatter',
                        data: outliers
                    }}
                ]
            }};
            
            chart.setOption(option);
            
            // Handle responsive resizing
            window.addEventListener('resize', function() {{
                chart.resize();
            }});
        }}
        
        // Render radar chart using ECharts
        function renderRadarChart(container, data) {{
            if (data.data.type !== "Categorical") return;
            
            const chart = echarts.init(container);
            
            // Extract data
            const categories = data.data.data.categories;
            const seriesNames = Object.keys(data.data.data.series);
            
            // Prepare radar series data
            const series = seriesNames.map((name, index) => ({{
                name: name,
                type: 'radar',
                data: [{{
                    value: data.data.data.series[name],
                    name: name
                }}],
                symbol: 'circle',
                symbolSize: 6,
                lineStyle: {{
                    width: 2
                }},
                areaStyle: {{
                    opacity: 0.3
                }}
            }}));
            
            const option = {{
                title: {{
                    text: data.title,
                    left: 'center'
                }},
                tooltip: {{
                    trigger: 'item'
                }},
                legend: {{
                    data: seriesNames,
                    bottom: 0
                }},
                radar: {{
                    indicator: categories.map(category => ({{
                        name: category,
                        max: Math.max(...seriesNames.map(name => 
                            Math.max(...data.data.data.series[name])
                        )) * 1.2
                    }}))
                }},
                series: series
            }};
            
            chart.setOption(option);
            
            // Handle responsive resizing
            window.addEventListener('resize', function() {{
                chart.resize();
            }});
        }}
        
        // Render composite chart using ECharts
        function renderCompositeChart(container, data) {{
            if (data.data.type !== "TimeSeries") return;
            
            const chart = echarts.init(container);
            
            // Extract data
            const timestamps = data.data.data.timestamps.map(ts => new Date(ts));
            const seriesNames = Object.keys(data.data.data.series);
            
            // Prepare series data
            const series = seriesNames.map((name, index) => {{
                const values = data.data.data.series[name];
                const seriesData = timestamps.map((ts, i) => [ts, values[i]]);
                
                return {{
                    name: name,
                    type: 'line',
                    data: seriesData,
                    smooth: true,
                    symbol: 'circle',
                    symbolSize: 6,
                    lineStyle: {{
                        width: 2
                    }},
                    areaStyle: data.options.fill_opacity > 0 ? {{
                        opacity: data.options.fill_opacity
                    }} : undefined
                }};
            }});
            
            const option = {{
                title: {{
                    text: data.title,
                    left: 'center'
                }},
                tooltip: {{
                    trigger: 'axis',
                    formatter: function(params) {{
                        const ts = new Date(params[0].value[0]);
                        let result = ts.toLocaleString() + '<br/>';
                        
                        params.forEach(param => {{
                            result += `${param.seriesName}: ${param.value[1]}<br/>`;
                        }});
                        
                        return result;
                    }}
                }},
                legend: {{
                    data: seriesNames,
                    bottom: 0
                }},
                grid: {{
                    left: '3%',
                    right: '4%',
                    bottom: '10%',
                    containLabel: true
                }},
                xAxis: {{
                    type: 'time',
                    name: data.options.x_axis_label || 'Time',
                    axisLabel: {{
                        formatter: function(value) {{
                            const date = new Date(value);
                            return date.toLocaleDateString();
                        }}
                    }}
                }},
                yAxis: {{
                    type: 'value',
                    name: data.options.y_axis_label || 'Value'
                }},
                series: series
            }};
            
            chart.setOption(option);
            
            // Handle responsive resizing
            window.addEventListener('resize', function() {{
                chart.resize();
            }});
        }}
        
        // Default chart renderer
        function renderDefaultChart(container, data) {{
            container.innerHTML = `<div style="padding: 20px; text-align: center;">
                <h2>${data.title}</h2>
                <p>Chart type '${data.chart_type}' rendering not implemented.</p>
            </div>`;
        }}
        
        // Initialize chart when page loads
        document.addEventListener('DOMContentLoaded', function() {{
            const container = document.getElementById('chart-container');
            {}(container, chartData);
        }});
    </script>
</head>
<body>
    <div id="chart-container"></div>
</body>
</html>"#,
            chart.title,
            chart_json,
            render_function
        );
        
        Ok(html)
    }
    
    /// チャートをJSONに変換
    pub fn to_json(&self, chart_id: &str) -> Result<String, Error> {
        let chart = self.chart_cache.get(chart_id).ok_or_else(|| {
            Error::NotFound(format!("Chart not found: {}", chart_id))
        })?;
        
        serde_json::to_string_pretty(chart)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize chart: {}", e)))
    }
    
    /// チャートを取得
    pub fn get_chart(&self, chart_id: &str) -> Option<&AdvancedChartData> {
        self.chart_cache.get(chart_id)
    }
    
    /// すべてのチャートを取得
    pub fn get_all_charts(&self) -> Vec<&AdvancedChartData> {
        self.chart_cache.values().collect()
    }
    
    /// チャートを削除
    pub fn delete_chart(&mut self, chart_id: &str) -> Result<(), Error> {
        if !self.chart_cache.contains_key(chart_id) {
            return Err(Error::NotFound(format!("Chart not found: {}", chart_id)));
        }
        
        self.chart_cache.remove(chart_id);
        
        Ok(())
    }
    
    /// デフォルトオプションを設定
    pub fn set_default_options(&mut self, options: AdvancedChartOptions) {
        self.default_options = options;
    }
    
    /// 色パレットを追加
    pub fn add_color_palette(&mut self, name: &str, colors: Vec<String>) {
        self.color_palettes.insert(name.to_string(), colors);
    }
    
    /// 色パレットを取得
    pub fn get_color_palette(&self, name: &str) -> Option<&[String]> {
        self.color_palettes.get(name).map(|p| p.as_slice())
    }
    
    /// 色を取得
    pub fn get_color(&self, palette_name: &str, index: usize) -> Option<String> {
        if let Some(palette) = self.color_palettes.get(palette_name) {
            if !palette.is_empty() {
                return Some(palette[index % palette.len()].clone());
            }
        }
        
        // デフォルトパレットから取得
        if let Some(default_palette) = self.color_palettes.get("default") {
            if !default_palette.is_empty() {
                return Some(default_palette[index % default_palette.len()].clone());
            }
        }
        
        None
    }
}