use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::advanced_chart::{
    BasicChartExporter, Chart, ChartBuilder, ChartConfig, ChartDataAggregator, ChartExporter,
    ChartManager, ChartMargin, ChartRenderer, ChartType, DataPoint, DataSeries, DataSeriesBuilder,
    HTMLChartRenderer, OHLCChart, OHLCChartBuilder, OHLCDataPoint, OHLCDataSeries,
    OHLCDataSeriesBuilder, SVGChartRenderer, TimeFrame,
};
use crate::error::Error;
use crate::shard::ShardId;
use crate::transaction::{Transaction, TransactionStatus};

/// 拡張チャートタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnhancedChartType {
    /// 基本チャートタイプ
    Basic(ChartType),
    /// 複合チャート
    Combined(Vec<ChartType>),
    /// ネットワークグラフ
    Network,
    /// ツリーマップ
    Treemap,
    /// サンバースト
    Sunburst,
    /// パラレルコーディネート
    ParallelCoordinates,
    /// ボックスプロット
    BoxPlot,
    /// バイオリンプロット
    ViolinPlot,
    /// ストリームグラフ
    Streamgraph,
    /// 3Dサーフェス
    Surface3D,
}

/// 拡張データポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDataPoint {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 値
    pub value: f64,
    /// 補助値
    pub auxiliary_values: Option<HashMap<String, f64>>,
    /// カテゴリ
    pub category: Option<String>,
    /// グループ
    pub group: Option<String>,
    /// サイズ
    pub size: Option<f64>,
    /// 色
    pub color: Option<String>,
    /// 形状
    pub shape: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 拡張データシリーズ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDataSeries {
    /// シリーズID
    pub id: String,
    /// シリーズ名
    pub name: String,
    /// データポイント
    pub data_points: Vec<EnhancedDataPoint>,
    /// チャートタイプ
    pub chart_type: EnhancedChartType,
    /// Y軸
    pub y_axis: usize,
    /// 色
    pub color: Option<String>,
    /// 線のスタイル
    pub line_style: Option<String>,
    /// マーカーのスタイル
    pub marker_style: Option<String>,
    /// 塗りつぶしのスタイル
    pub fill_style: Option<String>,
    /// 可視性
    pub visible: bool,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 拡張チャート設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedChartConfig {
    /// 基本設定
    pub base_config: ChartConfig,
    /// Y軸の数
    pub y_axis_count: usize,
    /// Y軸のタイトル
    pub y_axis_titles: Vec<String>,
    /// Y軸の位置
    pub y_axis_positions: Vec<String>,
    /// Y軸のスケール
    pub y_axis_scales: Vec<String>,
    /// ズーム機能の有効化
    pub enable_zoom: bool,
    /// パン機能の有効化
    pub enable_pan: bool,
    /// ツールチップの有効化
    pub enable_tooltip: bool,
    /// 凡例の有効化
    pub enable_legend: bool,
    /// グリッドの有効化
    pub enable_grid: bool,
    /// アニメーションの有効化
    pub enable_animation: bool,
    /// テーマ
    pub theme: String,
    /// フォント
    pub font: String,
    /// フォントサイズ
    pub font_size: u32,
    /// 背景色
    pub background_color: String,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 拡張チャート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedChart {
    /// チャートID
    pub id: String,
    /// チャートタイトル
    pub title: String,
    /// チャート設定
    pub config: EnhancedChartConfig,
    /// データシリーズ
    pub series: Vec<EnhancedDataSeries>,
    /// 注釈
    pub annotations: Vec<ChartAnnotation>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// チャート注釈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartAnnotation {
    /// 注釈ID
    pub id: String,
    /// 注釈タイプ
    pub annotation_type: AnnotationType,
    /// X座標
    pub x: Option<DateTime<Utc>>,
    /// Y座標
    pub y: Option<f64>,
    /// テキスト
    pub text: Option<String>,
    /// 色
    pub color: Option<String>,
    /// 線のスタイル
    pub line_style: Option<String>,
    /// 塗りつぶしのスタイル
    pub fill_style: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 注釈タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnnotationType {
    /// 垂直線
    VerticalLine,
    /// 水平線
    HorizontalLine,
    /// 矩形
    Rectangle,
    /// 円
    Circle,
    /// テキスト
    Text,
    /// 矢印
    Arrow,
    /// カスタム
    Custom(String),
}

/// 拡張チャートビルダー
pub struct EnhancedChartBuilder {
    /// チャートID
    id: String,
    /// チャートタイトル
    title: String,
    /// チャート設定
    config: EnhancedChartConfig,
    /// データシリーズ
    series: Vec<EnhancedDataSeries>,
    /// 注釈
    annotations: Vec<ChartAnnotation>,
    /// メタデータ
    metadata: Option<HashMap<String, String>>,
}

impl EnhancedChartBuilder {
    /// 新しい拡張チャートビルダーを作成
    pub fn new(id: String, title: String) -> Self {
        // デフォルト設定を作成
        let base_config = ChartConfig {
            width: 800,
            height: 400,
            margin: ChartMargin {
                top: 20,
                right: 20,
                bottom: 40,
                left: 50,
            },
            x_axis_title: "Time".to_string(),
            y_axis_title: "Value".to_string(),
            show_grid: true,
            show_legend: true,
            interactive: true,
        };

        let config = EnhancedChartConfig {
            base_config,
            y_axis_count: 1,
            y_axis_titles: vec!["Value".to_string()],
            y_axis_positions: vec!["left".to_string()],
            y_axis_scales: vec!["linear".to_string()],
            enable_zoom: true,
            enable_pan: true,
            enable_tooltip: true,
            enable_legend: true,
            enable_grid: true,
            enable_animation: true,
            theme: "light".to_string(),
            font: "Arial".to_string(),
            font_size: 12,
            background_color: "#ffffff".to_string(),
            metadata: None,
        };

        Self {
            id,
            title,
            config,
            series: Vec::new(),
            annotations: Vec::new(),
            metadata: None,
        }
    }

    /// チャート設定を設定
    pub fn with_config(mut self, config: EnhancedChartConfig) -> Self {
        self.config = config;
        self
    }

    /// データシリーズを追加
    pub fn add_series(mut self, series: EnhancedDataSeries) -> Self {
        self.series.push(series);
        self
    }

    /// 注釈を追加
    pub fn add_annotation(mut self, annotation: ChartAnnotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// メタデータを設定
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// 拡張チャートを構築
    pub fn build(self) -> EnhancedChart {
        EnhancedChart {
            id: self.id,
            title: self.title,
            config: self.config,
            series: self.series,
            annotations: self.annotations,
            metadata: self.metadata,
        }
    }
}

/// 拡張データシリーズビルダー
pub struct EnhancedDataSeriesBuilder {
    /// シリーズID
    id: String,
    /// シリーズ名
    name: String,
    /// データポイント
    data_points: Vec<EnhancedDataPoint>,
    /// チャートタイプ
    chart_type: EnhancedChartType,
    /// Y軸
    y_axis: usize,
    /// 色
    color: Option<String>,
    /// 線のスタイル
    line_style: Option<String>,
    /// マーカーのスタイル
    marker_style: Option<String>,
    /// 塗りつぶしのスタイル
    fill_style: Option<String>,
    /// 可視性
    visible: bool,
    /// メタデータ
    metadata: Option<HashMap<String, String>>,
}

impl EnhancedDataSeriesBuilder {
    /// 新しい拡張データシリーズビルダーを作成
    pub fn new(id: String, name: String, chart_type: EnhancedChartType) -> Self {
        Self {
            id,
            name,
            data_points: Vec::new(),
            chart_type,
            y_axis: 0,
            color: None,
            line_style: None,
            marker_style: None,
            fill_style: None,
            visible: true,
            metadata: None,
        }
    }

    /// データポイントを追加
    pub fn add_data_point(mut self, data_point: EnhancedDataPoint) -> Self {
        self.data_points.push(data_point);
        self
    }

    /// データポイントを一括追加
    pub fn add_data_points(mut self, data_points: Vec<EnhancedDataPoint>) -> Self {
        self.data_points.extend(data_points);
        self
    }

    /// Y軸を設定
    pub fn with_y_axis(mut self, y_axis: usize) -> Self {
        self.y_axis = y_axis;
        self
    }

    /// 色を設定
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    /// 線のスタイルを設定
    pub fn with_line_style(mut self, line_style: String) -> Self {
        self.line_style = Some(line_style);
        self
    }

    /// マーカーのスタイルを設定
    pub fn with_marker_style(mut self, marker_style: String) -> Self {
        self.marker_style = Some(marker_style);
        self
    }

    /// 塗りつぶしのスタイルを設定
    pub fn with_fill_style(mut self, fill_style: String) -> Self {
        self.fill_style = Some(fill_style);
        self
    }

    /// 可視性を設定
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// メタデータを設定
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// 拡張データシリーズを構築
    pub fn build(self) -> EnhancedDataSeries {
        EnhancedDataSeries {
            id: self.id,
            name: self.name,
            data_points: self.data_points,
            chart_type: self.chart_type,
            y_axis: self.y_axis,
            color: self.color,
            line_style: self.line_style,
            marker_style: self.marker_style,
            fill_style: self.fill_style,
            visible: self.visible,
            metadata: self.metadata,
        }
    }
}

/// 拡張チャートレンダラー
pub trait EnhancedChartRenderer {
    /// チャートをレンダリング
    fn render(&self, chart: &EnhancedChart) -> Result<String, Error>;
}

/// SVG拡張チャートレンダラー
pub struct SVGEnhancedChartRenderer;

impl EnhancedChartRenderer for SVGEnhancedChartRenderer {
    fn render(&self, chart: &EnhancedChart) -> Result<String, Error> {
        // SVGレンダリングの実装（簡易版）
        let mut svg = format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            chart.config.base_config.width, chart.config.base_config.height
        );

        // タイトル
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" font-family="{}" font-size="{}" text-anchor="middle">{}</text>"#,
            chart.config.base_config.width / 2,
            chart.config.base_config.margin.top / 2,
            chart.config.font,
            chart.config.font_size + 4,
            chart.title
        ));

        // 各シリーズをレンダリング
        for series in &chart.series {
            if !series.visible || series.data_points.is_empty() {
                continue;
            }

            // シリーズの色
            let color = series
                .color
                .clone()
                .unwrap_or_else(|| "#1f77b4".to_string());

            match &series.chart_type {
                EnhancedChartType::Basic(ChartType::Line) => {
                    // 折れ線グラフの描画
                    let mut path_data = String::new();
                    let mut first = true;

                    for (i, point) in series.data_points.iter().enumerate() {
                        let x = chart.config.base_config.margin.left
                            + i as f64
                                * (chart.config.base_config.width
                                    - chart.config.base_config.margin.left
                                    - chart.config.base_config.margin.right)
                                / (series.data_points.len() - 1) as f64;
                        let y = chart.config.base_config.height
                            - chart.config.base_config.margin.bottom
                            - point.value
                                * (chart.config.base_config.height
                                    - chart.config.base_config.margin.top
                                    - chart.config.base_config.margin.bottom)
                                / 100.0;

                        if first {
                            path_data.push_str(&format!("M{},{}", x, y));
                            first = false;
                        } else {
                            path_data.push_str(&format!(" L{},{}", x, y));
                        }
                    }

                    svg.push_str(&format!(
                        r#"<path d="{}" stroke="{}" stroke-width="2" fill="none"/>"#,
                        path_data, color
                    ));
                }
                EnhancedChartType::Basic(ChartType::Bar) => {
                    // 棒グラフの描画
                    let bar_width = (chart.config.base_config.width
                        - chart.config.base_config.margin.left
                        - chart.config.base_config.margin.right)
                        / series.data_points.len() as f64
                        * 0.8;

                    for (i, point) in series.data_points.iter().enumerate() {
                        let x = chart.config.base_config.margin.left
                            + i as f64
                                * (chart.config.base_config.width
                                    - chart.config.base_config.margin.left
                                    - chart.config.base_config.margin.right)
                                / series.data_points.len() as f64;
                        let y = chart.config.base_config.height
                            - chart.config.base_config.margin.bottom
                            - point.value
                                * (chart.config.base_config.height
                                    - chart.config.base_config.margin.top
                                    - chart.config.base_config.margin.bottom)
                                / 100.0;

                        svg.push_str(&format!(
                            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
                            x - bar_width / 2.0,
                            y,
                            bar_width,
                            chart.config.base_config.height
                                - chart.config.base_config.margin.bottom
                                - y,
                            color
                        ));
                    }
                }
                _ => {
                    // その他のチャートタイプ（簡易実装では省略）
                }
            }
        }

        // 注釈をレンダリング
        for annotation in &chart.annotations {
            match annotation.annotation_type {
                AnnotationType::VerticalLine => {
                    if let Some(x_time) = annotation.x {
                        // X軸の位置を計算（簡易実装）
                        let x = chart.config.base_config.margin.left + 100.0; // 仮の位置

                        svg.push_str(&format!(
                            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1" stroke-dasharray="5,5"/>"#,
                            x,
                            chart.config.base_config.margin.top,
                            x,
                            chart.config.base_config.height - chart.config.base_config.margin.bottom,
                            annotation.color.clone().unwrap_or_else(|| "#ff0000".to_string())
                        ));
                    }
                }
                _ => {
                    // その他の注釈タイプ（簡易実装では省略）
                }
            }
        }

        svg.push_str("</svg>");

        Ok(svg)
    }
}

/// HTML拡張チャートレンダラー
pub struct HTMLEnhancedChartRenderer;

impl EnhancedChartRenderer for HTMLEnhancedChartRenderer {
    fn render(&self, chart: &EnhancedChart) -> Result<String, Error> {
        // HTML/JavaScript（Chart.js）を使用したレンダリング（簡易版）
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Enhanced Chart</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <div style="width: 100%; max-width: 800px; margin: 0 auto;">
        <canvas id="myChart"></canvas>
    </div>
    <script>
        const ctx = document.getElementById('myChart').getContext('2d');
        const myChart = new Chart(ctx, {
            type: 'line',
            data: {
                datasets: [
"#,
        );

        // データセットを追加
        for (i, series) in chart.series.iter().enumerate() {
            if !series.visible {
                continue;
            }

            let chart_type = match &series.chart_type {
                EnhancedChartType::Basic(ChartType::Line) => "line",
                EnhancedChartType::Basic(ChartType::Bar) => "bar",
                _ => "line", // デフォルト
            };

            let color = series
                .color
                .clone()
                .unwrap_or_else(|| "#1f77b4".to_string());

            html.push_str(&format!(
                r#"                    {{
                        label: '{}',
                        type: '{}',
                        data: ["#,
                series.name, chart_type
            ));

            // データポイントを追加
            for (j, point) in series.data_points.iter().enumerate() {
                html.push_str(&format!(
                    r#"
                            {{ x: new Date('{}'), y: {} }}"#,
                    point.timestamp.to_rfc3339(),
                    point.value
                ));

                if j < series.data_points.len() - 1 {
                    html.push_str(",");
                }
            }

            html.push_str(&format!(
                r#"
                        ],
                        borderColor: '{}',
                        backgroundColor: '{}',
                        yAxisID: 'y{}',
                    }}"#,
                color, color, series.y_axis
            ));

            if i < chart.series.len() - 1 {
                html.push_str(",");
            }
        }

        html.push_str(
            r#"
                ]
            },
            options: {
                responsive: true,
                plugins: {
                    title: {
                        display: true,
                        text: '"#,
        );

        html.push_str(&chart.title);

        html.push_str(
            r#"'
                    },
                    tooltip: {
                        mode: 'index',
                        intersect: false
                    },
                    legend: {
                        display: true,
                        position: 'top',
                    }
                },
                scales: {
                    x: {
                        type: 'time',
                        time: {
                            unit: 'day'
                        },
                        title: {
                            display: true,
                            text: '"#,
        );

        html.push_str(&chart.config.base_config.x_axis_title);

        html.push_str(
            r#"'
                        }
                    },
"#,
        );

        // Y軸の設定
        for i in 0..chart.config.y_axis_count {
            let position = if i < chart.config.y_axis_positions.len() {
                &chart.config.y_axis_positions[i]
            } else {
                "left"
            };

            let title = if i < chart.config.y_axis_titles.len() {
                &chart.config.y_axis_titles[i]
            } else {
                "Value"
            };

            html.push_str(&format!(
                r#"                    y{}: {{
                        type: 'linear',
                        display: true,
                        position: '{}',
                        title: {{
                            display: true,
                            text: '{}'
                        }}
                    }}"#,
                i, position, title
            ));

            if i < chart.config.y_axis_count - 1 {
                html.push_str(",");
            }
        }

        html.push_str(
            r#"
                }
            }
        });
    </script>
</body>
</html>"#,
        );

        Ok(html)
    }
}

/// 拡張チャートエクスポーター
pub struct EnhancedChartExporter;

impl EnhancedChartExporter {
    /// SVGとしてエクスポート
    pub fn export_as_svg(&self, chart: &EnhancedChart) -> Result<String, Error> {
        let renderer = SVGEnhancedChartRenderer;
        renderer.render(chart)
    }

    /// HTMLとしてエクスポート
    pub fn export_as_html(&self, chart: &EnhancedChart) -> Result<String, Error> {
        let renderer = HTMLEnhancedChartRenderer;
        renderer.render(chart)
    }

    /// JSONとしてエクスポート
    pub fn export_as_json(&self, chart: &EnhancedChart) -> Result<String, Error> {
        match serde_json::to_string_pretty(chart) {
            Ok(json) => Ok(json),
            Err(e) => Err(Error::SerializationError(e.to_string())),
        }
    }
}

/// 拡張チャートマネージャー
pub struct EnhancedChartManager {
    /// チャートのコレクション
    charts: HashMap<String, EnhancedChart>,
}

impl EnhancedChartManager {
    /// 新しい拡張チャートマネージャーを作成
    pub fn new() -> Self {
        Self {
            charts: HashMap::new(),
        }
    }

    /// チャートを追加
    pub fn add_chart(&mut self, chart: EnhancedChart) {
        self.charts.insert(chart.id.clone(), chart);
    }

    /// チャートを取得
    pub fn get_chart(&self, id: &str) -> Option<&EnhancedChart> {
        self.charts.get(id)
    }

    /// チャートを削除
    pub fn remove_chart(&mut self, id: &str) -> Option<EnhancedChart> {
        self.charts.remove(id)
    }

    /// すべてのチャートを取得
    pub fn get_all_charts(&self) -> Vec<&EnhancedChart> {
        self.charts.values().collect()
    }

    /// チャートをエクスポート
    pub fn export_chart(&self, id: &str, format: &str) -> Result<String, Error> {
        let chart = self
            .get_chart(id)
            .ok_or_else(|| Error::NotFound(format!("Chart with ID {} not found", id)))?;

        let exporter = EnhancedChartExporter;

        match format.to_lowercase().as_str() {
            "svg" => exporter.export_as_svg(chart),
            "html" => exporter.export_as_html(chart),
            "json" => exporter.export_as_json(chart),
            _ => Err(Error::InvalidInput(format!(
                "Unsupported export format: {}",
                format
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_chart_builder() {
        // チャートビルダーを作成
        let chart =
            EnhancedChartBuilder::new("test-chart".to_string(), "Test Chart".to_string()).build();

        assert_eq!(chart.id, "test-chart");
        assert_eq!(chart.title, "Test Chart");
        assert!(chart.series.is_empty());
    }

    #[test]
    fn test_enhanced_data_series_builder() {
        // データシリーズビルダーを作成
        let series = EnhancedDataSeriesBuilder::new(
            "test-series".to_string(),
            "Test Series".to_string(),
            EnhancedChartType::Basic(ChartType::Line),
        )
        .build();

        assert_eq!(series.id, "test-series");
        assert_eq!(series.name, "Test Series");
        assert!(series.data_points.is_empty());
    }

    #[test]
    fn test_svg_renderer() {
        // チャートを作成
        let chart =
            EnhancedChartBuilder::new("test-chart".to_string(), "Test Chart".to_string()).build();

        // SVGレンダラーを作成
        let renderer = SVGEnhancedChartRenderer;

        // レンダリングを実行
        let result = renderer.render(&chart);

        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_html_renderer() {
        // チャートを作成
        let chart =
            EnhancedChartBuilder::new("test-chart".to_string(), "Test Chart".to_string()).build();

        // HTMLレンダラーを作成
        let renderer = HTMLEnhancedChartRenderer;

        // レンダリングを実行
        let result = renderer.render(&chart);

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<canvas id=\"myChart\"></canvas>"));
    }
}
