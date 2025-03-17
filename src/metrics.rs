use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// メトリクスコレクター
///
/// システム全体のメトリクスを収集し、モニタリングする。
/// - カウンター: 単調増加する値（例: リクエスト数）
/// - ゲージ: 任意の値（例: キューサイズ）
/// - ヒストグラム: 値の分布（例: レスポンス時間）
pub struct MetricsCollector {
    /// カウンター
    counters: Mutex<HashMap<String, u64>>,
    /// ゲージ
    gauges: Mutex<HashMap<String, f64>>,
    /// ヒストグラム
    histograms: Mutex<HashMap<String, Vec<f64>>>,
    /// 最終更新時刻
    last_updated: Mutex<Instant>,
}

impl MetricsCollector {
    /// 新しいMetricsCollectorを作成
    pub fn new() -> Self {
        Self {
            counters: Mutex::new(HashMap::new()),
            gauges: Mutex::new(HashMap::new()),
            histograms: Mutex::new(HashMap::new()),
            last_updated: Mutex::new(Instant::now()),
        }
    }

    /// カウンターをインクリメント
    pub fn increment_counter(&self, name: &str) {
        let mut counters = self.counters.lock().unwrap();
        let counter = counters.entry(name.to_string()).or_insert(0);
        *counter += 1;
        *self.last_updated.lock().unwrap() = Instant::now();
    }

    /// カウンターを特定の値だけインクリメント
    pub fn increment_counter_by(&self, name: &str, value: u64) {
        let mut counters = self.counters.lock().unwrap();
        let counter = counters.entry(name.to_string()).or_insert(0);
        *counter += value;
        *self.last_updated.lock().unwrap() = Instant::now();
    }

    /// ゲージを設定
    pub fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.lock().unwrap();
        gauges.insert(name.to_string(), value);
        *self.last_updated.lock().unwrap() = Instant::now();
    }

    /// ヒストグラムに値を追加
    pub fn observe_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.lock().unwrap();
        let histogram = histograms.entry(name.to_string()).or_insert_with(Vec::new);
        histogram.push(value);
        *self.last_updated.lock().unwrap() = Instant::now();
    }

    /// カウンターの値を取得
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        let counters = self.counters.lock().unwrap();
        counters.get(name).cloned()
    }

    /// ゲージの値を取得
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        let gauges = self.gauges.lock().unwrap();
        gauges.get(name).cloned()
    }

    /// ヒストグラムの値を取得
    pub fn get_histogram(&self, name: &str) -> Option<Vec<f64>> {
        let histograms = self.histograms.lock().unwrap();
        histograms.get(name).cloned()
    }

    /// ヒストグラムの平均値を計算
    pub fn get_histogram_average(&self, name: &str) -> Option<f64> {
        let histograms = self.histograms.lock().unwrap();
        if let Some(histogram) = histograms.get(name) {
            if histogram.is_empty() {
                return None;
            }
            let sum: f64 = histogram.iter().sum();
            Some(sum / histogram.len() as f64)
        } else {
            None
        }
    }

    /// ヒストグラムのパーセンタイルを計算
    pub fn get_histogram_percentile(&self, name: &str, percentile: f64) -> Option<f64> {
        let histograms = self.histograms.lock().unwrap();
        if let Some(histogram) = histograms.get(name) {
            if histogram.is_empty() {
                return None;
            }
            let mut sorted = histogram.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let index = (percentile / 100.0 * (sorted.len() - 1) as f64).round() as usize;
            Some(sorted[index])
        } else {
            None
        }
    }

    /// 全てのメトリクスを取得
    pub fn get_all_metrics(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        // カウンター
        let counters = self.counters.lock().unwrap();
        for (name, value) in counters.iter() {
            result.insert(format!("counter:{}", name), value.to_string());
        }

        // ゲージ
        let gauges = self.gauges.lock().unwrap();
        for (name, value) in gauges.iter() {
            result.insert(format!("gauge:{}", name), value.to_string());
        }

        // ヒストグラム（平均値）
        let histograms = self.histograms.lock().unwrap();
        for (name, values) in histograms.iter() {
            if !values.is_empty() {
                let sum: f64 = values.iter().sum();
                let avg = sum / values.len() as f64;
                result.insert(format!("histogram_avg:{}", name), avg.to_string());
            }
        }

        result
    }

    /// 最終更新時刻を取得
    pub fn get_last_updated(&self) -> Instant {
        *self.last_updated.lock().unwrap()
    }

    /// 最終更新からの経過時間を取得
    pub fn time_since_last_update(&self) -> Duration {
        self.get_last_updated().elapsed()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let metrics = MetricsCollector::new();
        metrics.increment_counter("test_counter");
        metrics.increment_counter("test_counter");
        assert_eq!(metrics.get_counter("test_counter"), Some(2));
    }

    #[test]
    fn test_gauge() {
        let metrics = MetricsCollector::new();
        metrics.set_gauge("test_gauge", 42.0);
        assert_eq!(metrics.get_gauge("test_gauge"), Some(42.0));
    }

    #[test]
    fn test_histogram() {
        let metrics = MetricsCollector::new();
        metrics.observe_histogram("test_histogram", 1.0);
        metrics.observe_histogram("test_histogram", 2.0);
        metrics.observe_histogram("test_histogram", 3.0);
        assert_eq!(metrics.get_histogram_average("test_histogram"), Some(2.0));
        assert_eq!(metrics.get_histogram_percentile("test_histogram", 50.0), Some(2.0));
    }
}