use crate::error::Error;
use std::collections::HashMap;

/// ガス価格
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GasPrice {
    /// 基本ガス価格（wei/gas）
    pub base_price: u64,
    /// 優先ガス価格（wei/gas）
    pub priority_price: u64,
    /// 最大ガス価格（wei/gas）
    pub max_price: u64,
}

/// ガス使用量
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GasUsage {
    /// 使用したガス量
    pub used: u64,
    /// 割り当てられたガス量
    pub allocated: u64,
    /// 返金されたガス量
    pub refunded: u64,
}

/// ガススケジュール
///
/// 各操作のガスコストを定義する。
#[derive(Debug, Clone)]
pub struct GasSchedule {
    /// 基本操作のガスコスト
    pub base_costs: HashMap<String, u64>,
    /// 動的操作のガスコスト計算関数
    pub dynamic_costs: HashMap<String, Box<dyn Fn(&[u8]) -> u64 + Send + Sync>>,
}

/// ガス見積もり
pub struct GasEstimator {
    /// ガススケジュール
    gas_schedule: GasSchedule,
    /// 最小ガス量
    min_gas: u64,
    /// 最大ガス量
    max_gas: u64,
    /// バッファ係数（安全マージン）
    buffer_factor: f64,
}

impl GasPrice {
    /// 新しいGasPriceを作成
    pub fn new(base_price: u64, priority_price: u64, max_price: u64) -> Self {
        Self {
            base_price,
            priority_price,
            max_price,
        }
    }

    /// 実効ガス価格を計算
    pub fn effective_price(&self) -> u64 {
        let price = self.base_price + self.priority_price;
        if price > self.max_price {
            self.max_price
        } else {
            price
        }
    }

    /// ガス代を計算
    pub fn calculate_fee(&self, gas_used: u64) -> u64 {
        self.effective_price() * gas_used
    }
}

impl GasUsage {
    /// 新しいGasUsageを作成
    pub fn new(used: u64, allocated: u64, refunded: u64) -> Self {
        Self {
            used,
            allocated,
            refunded,
        }
    }

    /// 実効ガス使用量を計算
    pub fn effective_used(&self) -> u64 {
        if self.used > self.refunded {
            self.used - self.refunded
        } else {
            0
        }
    }

    /// 残りのガス量を計算
    pub fn remaining(&self) -> u64 {
        if self.allocated > self.used {
            self.allocated - self.used
        } else {
            0
        }
    }

    /// ガス使用率を計算
    pub fn usage_ratio(&self) -> f64 {
        if self.allocated == 0 {
            0.0
        } else {
            self.effective_used() as f64 / self.allocated as f64
        }
    }
}

impl GasSchedule {
    /// 新しいGasScheduleを作成
    pub fn new() -> Self {
        Self {
            base_costs: HashMap::new(),
            dynamic_costs: HashMap::new(),
        }
    }

    /// 基本操作のガスコストを設定
    pub fn set_base_cost(&mut self, operation: &str, cost: u64) {
        self.base_costs.insert(operation.to_string(), cost);
    }

    /// 動的操作のガスコスト計算関数を設定
    pub fn set_dynamic_cost<F>(&mut self, operation: &str, cost_fn: F)
    where
        F: Fn(&[u8]) -> u64 + Send + Sync + 'static,
    {
        self.dynamic_costs
            .insert(operation.to_string(), Box::new(cost_fn));
    }

    /// 操作のガスコストを取得
    pub fn get_cost(&self, operation: &str, data: &[u8]) -> u64 {
        if let Some(cost) = self.base_costs.get(operation) {
            return *cost;
        }

        if let Some(cost_fn) = self.dynamic_costs.get(operation) {
            return cost_fn(data);
        }

        // デフォルトコスト
        1
    }
}

impl GasEstimator {
    /// 新しいGasEstimatorを作成
    pub fn new(gas_schedule: GasSchedule, min_gas: u64, max_gas: u64, buffer_factor: f64) -> Self {
        Self {
            gas_schedule,
            min_gas,
            max_gas,
            buffer_factor,
        }
    }

    /// コントラクト実行のガス使用量を見積もる
    pub fn estimate_contract_execution(
        &self,
        code: &[u8],
        function_name: &str,
        args: &[Vec<u8>],
    ) -> Result<u64, Error> {
        // 実際の実装では、コードを静的解析してガス使用量を見積もる
        // ここでは簡易的な実装を提供

        // 基本コスト
        let mut gas = 21000; // 基本トランザクションコスト

        // コードサイズに基づくコスト
        gas += code.len() as u64 * 4;

        // 関数名に基づくコスト
        gas += function_name.len() as u64 * 8;

        // 引数に基づくコスト
        for arg in args {
            gas += arg.len() as u64 * 16;
        }

        // バッファを追加
        gas = (gas as f64 * self.buffer_factor) as u64;

        // 最小・最大の範囲内に収める
        gas = gas.max(self.min_gas).min(self.max_gas);

        Ok(gas)
    }

    /// トランザクションのガス使用量を見積もる
    pub fn estimate_transaction(
        &self,
        to: Option<&[u8]>,
        data: &[u8],
        value: u64,
    ) -> Result<u64, Error> {
        // 基本コスト
        let mut gas = 21000; // 基本トランザクションコスト

        // コントラクト作成の場合
        if to.is_none() {
            gas += 32000; // コントラクト作成の追加コスト
        }

        // データに基づくコスト
        for &byte in data {
            if byte == 0 {
                gas += 4; // ゼロバイトのコスト
            } else {
                gas += 16; // 非ゼロバイトのコスト
            }
        }

        // 値送金に基づくコスト
        if value > 0 {
            gas += 9000; // 値送金の追加コスト
        }

        // バッファを追加
        gas = (gas as f64 * self.buffer_factor) as u64;

        // 最小・最大の範囲内に収める
        gas = gas.max(self.min_gas).min(self.max_gas);

        Ok(gas)
    }

    /// ガススケジュールを取得
    pub fn get_gas_schedule(&self) -> &GasSchedule {
        &self.gas_schedule
    }

    /// 最小ガス量を取得
    pub fn get_min_gas(&self) -> u64 {
        self.min_gas
    }

    /// 最大ガス量を取得
    pub fn get_max_gas(&self) -> u64 {
        self.max_gas
    }

    /// バッファ係数を取得
    pub fn get_buffer_factor(&self) -> f64 {
        self.buffer_factor
    }
}

impl Default for GasSchedule {
    fn default() -> Self {
        let mut schedule = Self::new();

        // 基本操作のコストを設定
        schedule.set_base_cost("add", 3);
        schedule.set_base_cost("sub", 3);
        schedule.set_base_cost("mul", 5);
        schedule.set_base_cost("div", 5);
        schedule.set_base_cost("mod", 5);
        schedule.set_base_cost("sload", 200);
        schedule.set_base_cost("sstore", 5000);
        schedule.set_base_cost("balance", 400);
        schedule.set_base_cost("call", 700);
        schedule.set_base_cost("create", 32000);
        schedule.set_base_cost("log", 375);
        schedule.set_base_cost("event", 375);

        // 動的操作のコスト計算関数を設定
        schedule.set_dynamic_cost("memory", |data| {
            // メモリ使用量に基づくコスト
            let size = data.len() as u64;
            let words = (size + 31) / 32; // 32バイトワード数
            3 * words + words * words / 512 // メモリ拡張コスト
        });

        schedule.set_dynamic_cost("sha3", |data| {
            // SHA3操作のコスト
            let size = data.len() as u64;
            let words = (size + 31) / 32; // 32バイトワード数
            30 + 6 * words // 基本コスト + データサイズに基づくコスト
        });

        schedule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_price() {
        let gas_price = GasPrice::new(10, 5, 20);
        assert_eq!(gas_price.base_price, 10);
        assert_eq!(gas_price.priority_price, 5);
        assert_eq!(gas_price.max_price, 20);
        assert_eq!(gas_price.effective_price(), 15);
        assert_eq!(gas_price.calculate_fee(1000), 15000);

        let gas_price = GasPrice::new(10, 15, 20);
        assert_eq!(gas_price.effective_price(), 20);
        assert_eq!(gas_price.calculate_fee(1000), 20000);
    }

    #[test]
    fn test_gas_usage() {
        let gas_usage = GasUsage::new(8000, 10000, 1000);
        assert_eq!(gas_usage.used, 8000);
        assert_eq!(gas_usage.allocated, 10000);
        assert_eq!(gas_usage.refunded, 1000);
        assert_eq!(gas_usage.effective_used(), 7000);
        assert_eq!(gas_usage.remaining(), 2000);
        assert_eq!(gas_usage.usage_ratio(), 0.7);

        let gas_usage = GasUsage::new(10000, 10000, 2000);
        assert_eq!(gas_usage.effective_used(), 8000);
        assert_eq!(gas_usage.remaining(), 0);
        assert_eq!(gas_usage.usage_ratio(), 0.8);
    }

    #[test]
    fn test_gas_schedule() {
        let mut gas_schedule = GasSchedule::new();
        gas_schedule.set_base_cost("add", 3);
        gas_schedule.set_base_cost("mul", 5);
        gas_schedule.set_dynamic_cost("memory", |data| data.len() as u64);

        assert_eq!(gas_schedule.get_cost("add", &[]), 3);
        assert_eq!(gas_schedule.get_cost("mul", &[]), 5);
        assert_eq!(gas_schedule.get_cost("memory", &[1, 2, 3, 4]), 4);
        assert_eq!(gas_schedule.get_cost("unknown", &[]), 1); // デフォルトコスト
    }

    #[test]
    fn test_gas_estimator() {
        let gas_schedule = GasSchedule::default();
        let gas_estimator = GasEstimator::new(gas_schedule, 21000, 1000000, 1.1);

        let code = b"contract Test { function test() public { } }";
        let function_name = "test";
        let args: Vec<Vec<u8>> = vec![];

        let estimated_gas = gas_estimator
            .estimate_contract_execution(code, function_name, &args)
            .unwrap();
        assert!(estimated_gas >= 21000);

        let data = b"test data";
        let estimated_tx_gas = gas_estimator
            .estimate_transaction(Some(b"address"), data, 0)
            .unwrap();
        assert!(estimated_tx_gas >= 21000);

        let estimated_create_gas = gas_estimator
            .estimate_transaction(None, data, 1000)
            .unwrap();
        assert!(estimated_create_gas > estimated_tx_gas);
    }

    #[test]
    fn test_gas_schedule_default() {
        let gas_schedule = GasSchedule::default();
        assert_eq!(gas_schedule.get_cost("add", &[]), 3);
        assert_eq!(gas_schedule.get_cost("sload", &[]), 200);
        assert_eq!(gas_schedule.get_cost("sstore", &[]), 5000);
        assert_eq!(gas_schedule.get_cost("call", &[]), 700);
        assert_eq!(gas_schedule.get_cost("create", &[]), 32000);

        // 動的コスト
        let memory_cost = gas_schedule.get_cost("memory", &vec![0; 64]);
        assert!(memory_cost > 0);

        let sha3_cost = gas_schedule.get_cost("sha3", &vec![0; 64]);
        assert!(sha3_cost > 0);
    }
}
