use crate::error::Error;
use crate::smart_contract::{ExecutionContext, ExecutionResult};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// エグゼキューター設定
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// 最大実行時間（ミリ秒）
    pub max_execution_time_ms: u64,
    /// 最大メモリ使用量（バイト）
    pub max_memory_bytes: u64,
    /// 最大スタックサイズ（バイト）
    pub max_stack_bytes: u64,
    /// 最大ストレージ使用量（バイト）
    pub max_storage_bytes: u64,
    /// 最大ガス使用量
    pub max_gas: u64,
    /// デバッグモードフラグ
    pub debug_mode: bool,
    /// トレースモードフラグ
    pub trace_mode: bool,
    /// 追加の設定
    pub extra_config: HashMap<String, String>,
}

/// 実行統計情報
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    /// 実行時間
    pub execution_time: Duration,
    /// メモリ使用量（バイト）
    pub memory_used_bytes: u64,
    /// ストレージ使用量（バイト）
    pub storage_used_bytes: u64,
    /// ガス使用量
    pub gas_used: u64,
    /// 命令実行回数
    pub instruction_count: u64,
    /// 関数呼び出し回数
    pub function_call_count: u64,
    /// ストレージ読み取り回数
    pub storage_read_count: u64,
    /// ストレージ書き込み回数
    pub storage_write_count: u64,
    /// 外部呼び出し回数
    pub external_call_count: u64,
    /// イベント発行回数
    pub event_count: u64,
}

/// コントラクトエグゼキューター
pub trait ContractExecutor: Send + Sync {
    /// エグゼキューター名
    fn name(&self) -> &str;
    /// エグゼキューターバージョン
    fn version(&self) -> &str;
    /// サポートするプラットフォーム
    fn supported_platforms(&self) -> Vec<String>;
    /// コードを実行
    fn execute(
        &self,
        code: &[u8],
        function_name: &str,
        args: &[Vec<u8>],
        context: &ExecutionContext,
        config: &ExecutorConfig,
    ) -> Result<ExecutionResult, Error>;
    /// 実行統計情報を取得
    fn get_stats(&self) -> ExecutionStats;
    /// 実行をデバッグ
    fn debug(
        &self,
        code: &[u8],
        function_name: &str,
        args: &[Vec<u8>],
        context: &ExecutionContext,
        config: &ExecutorConfig,
    ) -> Result<(ExecutionResult, Vec<String>), Error>;
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 1000,
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_stack_bytes: 1 * 1024 * 1024,    // 1 MB
            max_storage_bytes: 10 * 1024 * 1024, // 10 MB
            max_gas: 10_000_000,
            debug_mode: false,
            trace_mode: false,
            extra_config: HashMap::new(),
        }
    }
}

impl ExecutionStats {
    /// 新しいExecutionStatsを作成
    pub fn new() -> Self {
        Self {
            execution_time: Duration::from_secs(0),
            memory_used_bytes: 0,
            storage_used_bytes: 0,
            gas_used: 0,
            instruction_count: 0,
            function_call_count: 0,
            storage_read_count: 0,
            storage_write_count: 0,
            external_call_count: 0,
            event_count: 0,
        }
    }

    /// 実行時間を記録
    pub fn record_execution_time(&mut self, start_time: Instant) {
        self.execution_time = start_time.elapsed();
    }

    /// メモリ使用量を記録
    pub fn record_memory_used(&mut self, bytes: u64) {
        self.memory_used_bytes = bytes;
    }

    /// ストレージ使用量を記録
    pub fn record_storage_used(&mut self, bytes: u64) {
        self.storage_used_bytes = bytes;
    }

    /// ガス使用量を記録
    pub fn record_gas_used(&mut self, gas: u64) {
        self.gas_used = gas;
    }

    /// 命令実行回数を記録
    pub fn record_instruction_count(&mut self, count: u64) {
        self.instruction_count = count;
    }

    /// 関数呼び出し回数を記録
    pub fn record_function_call_count(&mut self, count: u64) {
        self.function_call_count = count;
    }

    /// ストレージ読み取り回数を記録
    pub fn record_storage_read_count(&mut self, count: u64) {
        self.storage_read_count = count;
    }

    /// ストレージ書き込み回数を記録
    pub fn record_storage_write_count(&mut self, count: u64) {
        self.storage_write_count = count;
    }

    /// 外部呼び出し回数を記録
    pub fn record_external_call_count(&mut self, count: u64) {
        self.external_call_count = count;
    }

    /// イベント発行回数を記録
    pub fn record_event_count(&mut self, count: u64) {
        self.event_count = count;
    }
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_contract::{ExecutionContext, ExecutionResult};

    struct TestExecutor {
        stats: ExecutionStats,
    }

    impl ContractExecutor for TestExecutor {
        fn name(&self) -> &str {
            "TestExecutor"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn supported_platforms(&self) -> Vec<String> {
            vec!["wasm32-unknown-unknown".to_string()]
        }

        fn execute(
            &self,
            code: &[u8],
            function_name: &str,
            args: &[Vec<u8>],
            context: &ExecutionContext,
            config: &ExecutorConfig,
        ) -> Result<ExecutionResult, Error> {
            let start_time = Instant::now();

            // 実行をシミュレート
            let mut result = ExecutionResult {
                success: true,
                return_data: Vec::new(),
                gas_used: Some(1000),
                logs: Vec::new(),
                error: None,
            };

            if function_name == "echo" && !args.is_empty() {
                result.return_data = args[0].clone();
            } else if function_name == "fail" {
                result.success = false;
                result.error = Some("Test execution failure".to_string());
            }

            // 統計情報を更新
            let mut stats = self.stats.clone();
            stats.record_execution_time(start_time);
            stats.record_gas_used(1000);
            stats.record_instruction_count(500);
            stats.record_memory_used(1024);
            stats.record_storage_used(512);
            stats.record_function_call_count(1);
            stats.record_storage_read_count(5);
            stats.record_storage_write_count(2);
            stats.record_external_call_count(0);
            stats.record_event_count(1);

            Ok(result)
        }

        fn get_stats(&self) -> ExecutionStats {
            self.stats.clone()
        }

        fn debug(
            &self,
            code: &[u8],
            function_name: &str,
            args: &[Vec<u8>],
            context: &ExecutionContext,
            config: &ExecutorConfig,
        ) -> Result<(ExecutionResult, Vec<String>), Error> {
            let result = self.execute(code, function_name, args, context, config)?;
            let debug_output = vec![
                "Debug: Execution started".to_string(),
                format!("Debug: Function called: {}", function_name),
                format!("Debug: Arguments: {} items", args.len()),
                format!("Debug: Gas used: {:?}", result.gas_used),
                "Debug: Execution completed".to_string(),
            ];
            Ok((result, debug_output))
        }
    }

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.max_execution_time_ms, 1000);
        assert_eq!(config.max_memory_bytes, 100 * 1024 * 1024);
        assert_eq!(config.max_stack_bytes, 1 * 1024 * 1024);
        assert_eq!(config.max_storage_bytes, 10 * 1024 * 1024);
        assert_eq!(config.max_gas, 10_000_000);
        assert_eq!(config.debug_mode, false);
        assert_eq!(config.trace_mode, false);
        assert!(config.extra_config.is_empty());
    }

    #[test]
    fn test_execution_stats() {
        let mut stats = ExecutionStats::new();
        assert_eq!(stats.execution_time, Duration::from_secs(0));
        assert_eq!(stats.memory_used_bytes, 0);
        assert_eq!(stats.storage_used_bytes, 0);
        assert_eq!(stats.gas_used, 0);
        assert_eq!(stats.instruction_count, 0);
        assert_eq!(stats.function_call_count, 0);
        assert_eq!(stats.storage_read_count, 0);
        assert_eq!(stats.storage_write_count, 0);
        assert_eq!(stats.external_call_count, 0);
        assert_eq!(stats.event_count, 0);

        stats.record_gas_used(1000);
        assert_eq!(stats.gas_used, 1000);

        stats.record_instruction_count(500);
        assert_eq!(stats.instruction_count, 500);
    }

    #[test]
    fn test_executor_execute() {
        let executor = TestExecutor {
            stats: ExecutionStats::new(),
        };
        let config = ExecutorConfig::default();
        let context = ExecutionContext::default();
        let code = b"test code".to_vec();
        let args = vec![b"test arg".to_vec()];

        let result = executor
            .execute(&code, "echo", &args, &context, &config)
            .unwrap();
        assert!(result.success);
        assert_eq!(result.return_data, b"test arg");
        assert_eq!(result.gas_used, Some(1000));
    }

    #[test]
    fn test_executor_execute_failure() {
        let executor = TestExecutor {
            stats: ExecutionStats::new(),
        };
        let config = ExecutorConfig::default();
        let context = ExecutionContext::default();
        let code = b"test code".to_vec();
        let args = vec![];

        let result = executor
            .execute(&code, "fail", &args, &context, &config)
            .unwrap();
        assert!(!result.success);
        assert_eq!(result.error, Some("Test execution failure".to_string()));
    }

    #[test]
    fn test_executor_debug() {
        let executor = TestExecutor {
            stats: ExecutionStats::new(),
        };
        let config = ExecutorConfig::default();
        let context = ExecutionContext::default();
        let code = b"test code".to_vec();
        let args = vec![b"test arg".to_vec()];

        let (result, debug_output) = executor
            .debug(&code, "echo", &args, &context, &config)
            .unwrap();
        assert!(result.success);
        assert_eq!(result.return_data, b"test arg");
        assert_eq!(debug_output.len(), 5);
        assert!(debug_output[0].contains("Execution started"));
    }
}