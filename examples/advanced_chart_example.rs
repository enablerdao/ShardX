use chrono::{Utc, Duration};
use std::collections::HashMap;
use shardx::chart::{
    ChartType, DataSeriesBuilder, ChartBuilder, ChartDataAggregator,
    OHLCDataSeriesBuilder, OHLCChartBuilder, TimeFrame, ChartManager
};

fn main() {
    // 現在時刻を取得
    let now = Utc::now();
    
    // 価格データを作成
    let price_data = vec![
        (now, 100.0),
        (now + Duration::hours(1), 105.0),
        (now + Duration::hours(2), 103.0),
        (now + Duration::hours(3), 107.0),
        (now + Duration::hours(4), 110.0),
        (now + Duration::hours(5), 108.0),
        (now + Duration::hours(6), 112.0),
        (now + Duration::hours(7), 115.0),
        (now + Duration::hours(8), 113.0),
        (now + Duration::hours(9), 118.0),
        (now + Duration::hours(10), 120.0),
        (now + Duration::hours(11), 123.0),
        (now + Duration::hours(12), 121.0),
        (now + Duration::hours(13), 125.0),
        (now + Duration::hours(14), 128.0),
        (now + Duration::hours(15), 130.0),
        (now + Duration::hours(16), 133.0),
        (now + Duration::hours(17), 131.0),
        (now + Duration::hours(18), 135.0),
        (now + Duration::hours(19), 138.0),
        (now + Duration::hours(20), 140.0),
        (now + Duration::hours(21), 137.0),
        (now + Duration::hours(22), 142.0),
        (now + Duration::hours(23), 145.0),
    ];
    
    // OHLCデータを作成
    let ohlc_data = vec![
        (now, 100.0, 105.0, 99.0, 103.0, 1000.0),
        (now + Duration::hours(4), 103.0, 110.0, 102.0, 108.0, 1200.0),
        (now + Duration::hours(8), 108.0, 115.0, 107.0, 113.0, 1500.0),
        (now + Duration::hours(12), 113.0, 125.0, 112.0, 121.0, 1800.0),
        (now + Duration::hours(16), 121.0, 134.0, 120.0, 131.0, 2000.0),
        (now + Duration::hours(20), 131.0, 142.0, 130.0, 140.0, 2200.0),
    ];
    
    // 価格データシリーズを作成
    let price_series = DataSeriesBuilder::new("price", "価格")
        .with_color("#4285F4")
        .with_line_style("solid")
        .add_data_points(price_data)
        .build();
    
    // 移動平均を計算
    let ma_data = ChartDataAggregator::calculate_moving_average(&price_series.data_points, 5);
    
    // 移動平均データシリーズを作成
    let ma_series = DataSeriesBuilder::new("ma", "5時間移動平均")
        .with_color("#DB4437")
        .with_line_style("dashed")
        .add_data_points(ma_data.iter().map(|p| (p.timestamp, p.value)).collect())
        .build();
    
    // ボリンジャーバンドを計算
    let (middle_band, upper_band, lower_band) = ChartDataAggregator::calculate_bollinger_bands(
        &price_series.data_points, 10, 2.0
    );
    
    // ボリンジャーバンドのデータシリーズを作成
    let middle_band_series = DataSeriesBuilder::new("middle_band", "中央バンド")
        .with_color("#0F9D58")
        .with_line_style("dotted")
        .add_data_points(middle_band.iter().map(|p| (p.timestamp, p.value)).collect())
        .build();
    
    let upper_band_series = DataSeriesBuilder::new("upper_band", "上部バンド")
        .with_color("#F4B400")
        .with_line_style("dotted")
        .add_data_points(upper_band.iter().map(|p| (p.timestamp, p.value)).collect())
        .build();
    
    let lower_band_series = DataSeriesBuilder::new("lower_band", "下部バンド")
        .with_color("#F4B400")
        .with_line_style("dotted")
        .add_data_points(lower_band.iter().map(|p| (p.timestamp, p.value)).collect())
        .build();
    
    // OHLCデータシリーズを作成
    let ohlc_series = OHLCDataSeriesBuilder::new("ohlc", "OHLC")
        .with_up_color("#0F9D58")
        .with_down_color("#DB4437")
        .add_data_points_with_volume(ohlc_data)
        .build();
    
    // チャートを作成
    let price_chart = ChartBuilder::new("price_chart", ChartType::Line, "価格チャート")
        .with_subtitle("24時間の価格推移")
        .with_x_axis_label("時間")
        .with_y_axis_label("価格")
        .with_size(800, 400)
        .add_series(price_series)
        .add_series(ma_series)
        .build();
    
    let bollinger_chart = ChartBuilder::new("bollinger_chart", ChartType::Line, "ボリンジャーバンド")
        .with_subtitle("10期間、標準偏差2.0")
        .with_x_axis_label("時間")
        .with_y_axis_label("価格")
        .with_size(800, 400)
        .add_series(middle_band_series)
        .add_series(upper_band_series)
        .add_series(lower_band_series)
        .build();
    
    let ohlc_chart = OHLCChartBuilder::new("ohlc_chart", "OHLCチャート")
        .with_subtitle("4時間足")
        .with_x_axis_label("時間")
        .with_y_axis_label("価格")
        .with_size(800, 400)
        .add_series(ohlc_series)
        .build();
    
    // チャートマネージャーを作成
    let mut chart_manager = ChartManager::new();
    
    // チャートを登録
    chart_manager.create_chart(price_chart).unwrap();
    chart_manager.create_chart(bollinger_chart).unwrap();
    chart_manager.create_ohlc_chart(ohlc_chart).unwrap();
    
    // チャートをレンダリング
    let price_chart_svg = chart_manager.render_chart("price_chart").unwrap();
    let bollinger_chart_svg = chart_manager.render_chart("bollinger_chart").unwrap();
    let ohlc_chart_svg = chart_manager.render_ohlc_chart("ohlc_chart").unwrap();
    
    // SVGをファイルに保存
    std::fs::write("price_chart.svg", price_chart_svg).unwrap();
    std::fs::write("bollinger_chart.svg", bollinger_chart_svg).unwrap();
    std::fs::write("ohlc_chart.svg", ohlc_chart_svg).unwrap();
    
    println!("チャートが生成されました。");
    println!("- price_chart.svg");
    println!("- bollinger_chart.svg");
    println!("- ohlc_chart.svg");
}