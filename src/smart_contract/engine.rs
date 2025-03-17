use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::Error;
use crate::shard::ShardId;
use crate::smart_contract::event::{ContractEvent, EventLog};
use crate::smart_contract::gas::{GasEstimator, GasPrice, GasSchedule, GasUsage};
use crate::smart_contract::storage::{ContractStorage, StorageError, StorageKey, StorageValue};
use crate::smart_contract::validator::{ContractValidator, ValidationError, ValidationResult};
use crate::smart_contract::vm::{ExecutionContext, ExecutionResult, VMError, VirtualMachine};
use crate::transaction::{Transaction, TransactionStatus};

/// コントラクトエンジン設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEngineConfig {
    /// 最大ガス制限
    pub max_gas_limit: u64,
    /// デフォルトガス制限
    pub default_gas_limit: u64,
    /// 最大コントラクトサイズ（バイト）
    pub max_contract_size: usize,
    /// 最大呼び出しスタック深度
    pub max_call_depth: usize,
    /// 最大メモリ使用量（バイト）
    pub max_memory_usage: usize,
    /// 最大ストレージ使用量（バイト）
    pub max_storage_usage: usize,
    /// 最大イベント数
    pub max_events: usize,
    /// 最大イベントサイズ（バイト）
    pub max_event_size: usize,
    /// 最大ログサイズ（バイト）
    pub max_log_size: usize,
    /// 最大コントラクト実行時間（ミリ秒）
    pub max_execution_time_ms: u64,
    /// 最大コントラクト数
    pub max_contracts: usize,
    /// 最大コントラクトメソッド数
    pub max_contract_methods: usize,
    /// 最大コントラクトイベント数
    pub max_contract_events: usize,
    /// 最大コントラクトフィールド数
    pub max_contract_fields: usize,
    /// 最大コントラクト引数数
    pub max_contract_args: usize,
    /// 最大コントラクト戻り値サイズ（バイト）
    pub max_return_value_size: usize,
    /// 最大コントラクト引数サイズ（バイト）
    pub max_arg_size: usize,
    /// 最大コントラクト名長
    pub max_contract_name_length: usize,
    /// 最大メソッド名長
    pub max_method_name_length: usize,
    /// 最大フィールド名長
    pub max_field_name_length: usize,
    /// 最大イベント名長
    pub max_event_name_length: usize,
    /// 最大引数名長
    pub max_arg_name_length: usize,
    /// 最大エラーメッセージ長
    pub max_error_message_length: usize,
    /// 最大ログメッセージ長
    pub max_log_message_length: usize,
    /// 最大コントラクトバージョン長
    pub max_contract_version_length: usize,
    /// 最大コントラクト作者長
    pub max_contract_author_length: usize,
    /// 最大コントラクト説明長
    pub max_contract_description_length: usize,
    /// 最大コントラクトライセンス長
    pub max_contract_license_length: usize,
    /// 最大コントラクトURL長
    pub max_contract_url_length: usize,
    /// 最大コントラクトメタデータサイズ（バイト）
    pub max_contract_metadata_size: usize,
    /// 最大コントラクトソースコードサイズ（バイト）
    pub max_contract_source_size: usize,
    /// 最大コントラクトABIサイズ（バイト）
    pub max_contract_abi_size: usize,
    /// 最大コントラクトバイトコードサイズ（バイト）
    pub max_contract_bytecode_size: usize,
    /// 最大コントラクトデプロイメントバイトコードサイズ（バイト）
    pub max_contract_deployment_bytecode_size: usize,
    /// 最大コントラクトランタイムバイトコードサイズ（バイト）
    pub max_contract_runtime_bytecode_size: usize,
    /// 最大コントラクトストレージキーサイズ（バイト）
    pub max_contract_storage_key_size: usize,
    /// 最大コントラクトストレージ値サイズ（バイト）
    pub max_contract_storage_value_size: usize,
    /// 最大コントラクトストレージエントリ数
    pub max_contract_storage_entries: usize,
    /// 最大コントラクトストレージサイズ（バイト）
    pub max_contract_storage_size: usize,
    /// 最大コントラクトイベントトピック数
    pub max_contract_event_topics: usize,
    /// 最大コントラクトイベントデータサイズ（バイト）
    pub max_contract_event_data_size: usize,
    /// 最大コントラクトイベント数
    pub max_contract_events_per_tx: usize,
    /// 最大コントラクトログ数
    pub max_contract_logs_per_tx: usize,
    /// 最大コントラクトログデータサイズ（バイト）
    pub max_contract_log_data_size: usize,
    /// 最大コントラクトログトピック数
    pub max_contract_log_topics: usize,
    /// 最大コントラクトログトピックサイズ（バイト）
    pub max_contract_log_topic_size: usize,
    /// 最大コントラクトログインデックス数
    pub max_contract_log_indices: usize,
    /// 最大コントラクトログインデックスサイズ（バイト）
    pub max_contract_log_index_size: usize,
    /// 最大コントラクトログブルームフィルタサイズ（バイト）
    pub max_contract_log_bloom_filter_size: usize,
    /// 最大コントラクトログブルームフィルタビット数
    pub max_contract_log_bloom_filter_bits: usize,
    /// 最大コントラクトログブルームフィルタハッシュ関数数
    pub max_contract_log_bloom_filter_hash_functions: usize,
    /// 最大コントラクトログブルームフィルタ誤検出率
    pub max_contract_log_bloom_filter_false_positive_rate: f64,
    /// 最大コントラクトログブルームフィルタ要素数
    pub max_contract_log_bloom_filter_elements: usize,
    /// 最大コントラクトログブルームフィルタサイズ（ビット）
    pub max_contract_log_bloom_filter_size_bits: usize,
    /// 最大コントラクトログブルームフィルタサイズ（バイト）
    pub max_contract_log_bloom_filter_size_bytes: usize,
    /// 最大コントラクトログブルームフィルタサイズ（ワード）
    pub max_contract_log_bloom_filter_size_words: usize,
    /// 最大コントラクトログブルームフィルタサイズ（ダブルワード）
    pub max_contract_log_bloom_filter_size_dwords: usize,
    /// 最大コントラクトログブルームフィルタサイズ（クワッドワード）
    pub max_contract_log_bloom_filter_size_qwords: usize,
    /// 最大コントラクトログブルームフィルタサイズ（オクタワード）
    pub max_contract_log_bloom_filter_size_owords: usize,
    /// 最大コントラクトログブルームフィルタサイズ（ヘクサワード）
    pub max_contract_log_bloom_filter_size_hwords: usize,
    /// 最大コントラクトログブルームフィルタサイズ（テトラワード）
    pub max_contract_log_bloom_filter_size_twords: usize,
    /// 最大コントラクトログブルームフィルタサイズ（ペンタワード）
    pub max_contract_log_bloom_filter_size_pwords: usize,
    /// 最大コントラクトログブルームフィルタサイズ（ヘキサワード）
    pub max_contract_log_bloom_filter_size_xwords: usize,
    /// 最大コントラクトログブルームフィルタサイズ（セプタワード）
    pub max_contract_log_bloom_filter_size_swords: usize,
    /// 最大コントラクトログブルームフィルタサイズ（オクタワード）
    pub max_contract_log_bloom_filter_size_owords2: usize,
    /// 最大コントラクトログブルームフィルタサイズ（ノナワード）
    pub max_contract_log_bloom_filter_size_nwords: usize,
    /// 最大コントラクトログブルームフィルタサイズ（デカワード）
    pub max_contract_log_bloom_filter_size_dwords2: usize,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

impl Default for ContractEngineConfig {
    fn default() -> Self {
        Self {
            max_gas_limit: 10_000_000,
            default_gas_limit: 1_000_000,
            max_contract_size: 24_576, // 24KB
            max_call_depth: 1024,
            max_memory_usage: 33_554_432,  // 32MB
            max_storage_usage: 16_777_216, // 16MB
            max_events: 1024,
            max_event_size: 4096,        // 4KB
            max_log_size: 4096,          // 4KB
            max_execution_time_ms: 5000, // 5秒
            max_contracts: 1024,
            max_contract_methods: 256,
            max_contract_events: 256,
            max_contract_fields: 256,
            max_contract_args: 32,
            max_return_value_size: 1024, // 1KB
            max_arg_size: 1024,          // 1KB
            max_contract_name_length: 64,
            max_method_name_length: 64,
            max_field_name_length: 64,
            max_event_name_length: 64,
            max_arg_name_length: 64,
            max_error_message_length: 256,
            max_log_message_length: 256,
            max_contract_version_length: 32,
            max_contract_author_length: 64,
            max_contract_description_length: 256,
            max_contract_license_length: 64,
            max_contract_url_length: 128,
            max_contract_metadata_size: 4096,              // 4KB
            max_contract_source_size: 1_048_576,           // 1MB
            max_contract_abi_size: 65_536,                 // 64KB
            max_contract_bytecode_size: 24_576,            // 24KB
            max_contract_deployment_bytecode_size: 24_576, // 24KB
            max_contract_runtime_bytecode_size: 24_576,    // 24KB
            max_contract_storage_key_size: 64,
            max_contract_storage_value_size: 1024, // 1KB
            max_contract_storage_entries: 1024,
            max_contract_storage_size: 16_777_216, // 16MB
            max_contract_event_topics: 4,
            max_contract_event_data_size: 4096, // 4KB
            max_contract_events_per_tx: 256,
            max_contract_logs_per_tx: 256,
            max_contract_log_data_size: 4096, // 4KB
            max_contract_log_topics: 4,
            max_contract_log_topic_size: 32,
            max_contract_log_indices: 8,
            max_contract_log_index_size: 32,
            max_contract_log_bloom_filter_size: 256, // 256バイト
            max_contract_log_bloom_filter_bits: 2048, // 2048ビット
            max_contract_log_bloom_filter_hash_functions: 3,
            max_contract_log_bloom_filter_false_positive_rate: 0.01, // 1%
            max_contract_log_bloom_filter_elements: 1024,
            max_contract_log_bloom_filter_size_bits: 2048, // 2048ビット
            max_contract_log_bloom_filter_size_bytes: 256, // 256バイト
            max_contract_log_bloom_filter_size_words: 128, // 128ワード
            max_contract_log_bloom_filter_size_dwords: 64, // 64ダブルワード
            max_contract_log_bloom_filter_size_qwords: 32, // 32クワッドワード
            max_contract_log_bloom_filter_size_owords: 16, // 16オクタワード
            max_contract_log_bloom_filter_size_hwords: 8,  // 8ヘクサワード
            max_contract_log_bloom_filter_size_twords: 4,  // 4テトラワード
            max_contract_log_bloom_filter_size_pwords: 2,  // 2ペンタワード
            max_contract_log_bloom_filter_size_xwords: 1,  // 1ヘキサワード
            max_contract_log_bloom_filter_size_swords: 1,  // 1セプタワード
            max_contract_log_bloom_filter_size_owords2: 1, // 1オクタワード
            max_contract_log_bloom_filter_size_nwords: 1,  // 1ノナワード
            max_contract_log_bloom_filter_size_dwords2: 1, // 1デカワード
            metadata: None,
        }
    }
}

/// コントラクトエンジン統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEngineStats {
    /// 総実行数
    pub total_executions: u64,
    /// 成功実行数
    pub successful_executions: u64,
    /// 失敗実行数
    pub failed_executions: u64,
    /// 総ガス使用量
    pub total_gas_used: u64,
    /// 平均ガス使用量
    pub average_gas_used: f64,
    /// 最大ガス使用量
    pub max_gas_used: u64,
    /// 最小ガス使用量
    pub min_gas_used: u64,
    /// 総実行時間（ミリ秒）
    pub total_execution_time_ms: u64,
    /// 平均実行時間（ミリ秒）
    pub average_execution_time_ms: f64,
    /// 最大実行時間（ミリ秒）
    pub max_execution_time_ms: u64,
    /// 最小実行時間（ミリ秒）
    pub min_execution_time_ms: u64,
    /// 総メモリ使用量（バイト）
    pub total_memory_used: u64,
    /// 平均メモリ使用量（バイト）
    pub average_memory_used: f64,
    /// 最大メモリ使用量（バイト）
    pub max_memory_used: u64,
    /// 最小メモリ使用量（バイト）
    pub min_memory_used: u64,
    /// 総ストレージ使用量（バイト）
    pub total_storage_used: u64,
    /// 平均ストレージ使用量（バイト）
    pub average_storage_used: f64,
    /// 最大ストレージ使用量（バイト）
    pub max_storage_used: u64,
    /// 最小ストレージ使用量（バイト）
    pub min_storage_used: u64,
    /// 総イベント数
    pub total_events: u64,
    /// 平均イベント数
    pub average_events: f64,
    /// 最大イベント数
    pub max_events: u64,
    /// 最小イベント数
    pub min_events: u64,
    /// 総ログ数
    pub total_logs: u64,
    /// 平均ログ数
    pub average_logs: f64,
    /// 最大ログ数
    pub max_logs: u64,
    /// 最小ログ数
    pub min_logs: u64,
    /// 総エラー数
    pub total_errors: u64,
    /// エラータイプごとの数
    pub errors_by_type: HashMap<String, u64>,
    /// 最も一般的なエラー
    pub most_common_error: Option<String>,
    /// 最も一般的なエラーの発生回数
    pub most_common_error_count: u64,
    /// 総コントラクト数
    pub total_contracts: u64,
    /// 総メソッド呼び出し数
    pub total_method_calls: u64,
    /// メソッドごとの呼び出し数
    pub calls_by_method: HashMap<String, u64>,
    /// 最も一般的なメソッド
    pub most_common_method: Option<String>,
    /// 最も一般的なメソッドの呼び出し回数
    pub most_common_method_count: u64,
    /// 総コントラクトデプロイ数
    pub total_contract_deployments: u64,
    /// 総コントラクト呼び出し数
    pub total_contract_calls: u64,
    /// 総コントラクト更新数
    pub total_contract_updates: u64,
    /// 総コントラクト削除数
    pub total_contract_deletions: u64,
    /// 総コントラクトストレージ読み取り数
    pub total_storage_reads: u64,
    /// 総コントラクトストレージ書き込み数
    pub total_storage_writes: u64,
    /// 総コントラクトストレージ削除数
    pub total_storage_deletes: u64,
    /// 総コントラクトイベント数
    pub total_contract_events: u64,
    /// 総コントラクトログ数
    pub total_contract_logs: u64,
    /// 最後の実行時刻
    pub last_execution_time: DateTime<Utc>,
    /// 最後のエラー時刻
    pub last_error_time: Option<DateTime<Utc>>,
    /// 最後のエラーメッセージ
    pub last_error_message: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

impl Default for ContractEngineStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_gas_used: 0,
            average_gas_used: 0.0,
            max_gas_used: 0,
            min_gas_used: 0,
            total_execution_time_ms: 0,
            average_execution_time_ms: 0.0,
            max_execution_time_ms: 0,
            min_execution_time_ms: 0,
            total_memory_used: 0,
            average_memory_used: 0.0,
            max_memory_used: 0,
            min_memory_used: 0,
            total_storage_used: 0,
            average_storage_used: 0.0,
            max_storage_used: 0,
            min_storage_used: 0,
            total_events: 0,
            average_events: 0.0,
            max_events: 0,
            min_events: 0,
            total_logs: 0,
            average_logs: 0.0,
            max_logs: 0,
            min_logs: 0,
            total_errors: 0,
            errors_by_type: HashMap::new(),
            most_common_error: None,
            most_common_error_count: 0,
            total_contracts: 0,
            total_method_calls: 0,
            calls_by_method: HashMap::new(),
            most_common_method: None,
            most_common_method_count: 0,
            total_contract_deployments: 0,
            total_contract_calls: 0,
            total_contract_updates: 0,
            total_contract_deletions: 0,
            total_storage_reads: 0,
            total_storage_writes: 0,
            total_storage_deletes: 0,
            total_contract_events: 0,
            total_contract_logs: 0,
            last_execution_time: Utc::now(),
            last_error_time: None,
            last_error_message: None,
            metadata: None,
        }
    }
}

/// コントラクトエンジン
pub struct ContractEngine<V: VirtualMachine, S: ContractStorage> {
    /// 設定
    config: ContractEngineConfig,
    /// 仮想マシン
    vm: V,
    /// ストレージ
    storage: S,
    /// バリデータ
    validator: ContractValidator,
    /// ガス見積もり
    gas_estimator: GasEstimator,
    /// 統計
    stats: Arc<Mutex<ContractEngineStats>>,
    /// イベントログ
    event_logs: Vec<EventLog>,
}

impl<V: VirtualMachine, S: ContractStorage> ContractEngine<V, S> {
    /// 新しいコントラクトエンジンを作成
    pub fn new(config: ContractEngineConfig, vm: V, storage: S) -> Self {
        Self {
            config,
            vm,
            storage,
            validator: ContractValidator::new(),
            gas_estimator: GasEstimator::new(),
            stats: Arc::new(Mutex::new(ContractEngineStats::default())),
            event_logs: Vec::new(),
        }
    }

    /// コントラクトをデプロイ
    pub fn deploy_contract(
        &mut self,
        code: Vec<u8>,
        args: Vec<u8>,
        gas_limit: Option<u64>,
    ) -> Result<String, Error> {
        // コントラクトサイズを検証
        if code.len() > self.config.max_contract_size {
            return Err(Error::InvalidInput(format!(
                "Contract size exceeds maximum: {} > {}",
                code.len(),
                self.config.max_contract_size
            )));
        }

        // コントラクトを検証
        let validation_result = self.validator.validate_contract(&code)?;
        if !validation_result.is_valid {
            return Err(Error::InvalidInput(format!(
                "Contract validation failed: {}",
                validation_result
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        // ガス制限を設定
        let gas_limit = gas_limit.unwrap_or(self.config.default_gas_limit);
        if gas_limit > self.config.max_gas_limit {
            return Err(Error::InvalidInput(format!(
                "Gas limit exceeds maximum: {} > {}",
                gas_limit, self.config.max_gas_limit
            )));
        }

        // 実行コンテキストを作成
        let context = ExecutionContext {
            gas_limit,
            sender: "system".to_string(),
            value: 0,
            data: args,
            address: None,
            block_height: 0,
            block_time: Utc::now(),
            is_static: false,
            depth: 0,
        };

        // コントラクトをデプロイ
        let start_time = std::time::Instant::now();
        let result = self.vm.deploy(code, context)?;
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // 統計を更新
        self.update_stats_for_execution(&result, execution_time_ms);

        // イベントログを保存
        self.event_logs
            .extend(result.events.into_iter().map(|event| EventLog {
                address: result.address.clone(),
                topics: event.topics,
                data: event.data,
                block_height: context.block_height,
                block_time: context.block_time,
                transaction_hash: "".to_string(),
                transaction_index: 0,
                log_index: 0,
                removed: false,
            }));

        // コントラクトアドレスを返す
        Ok(result.address)
    }

    /// コントラクトを呼び出し
    pub fn call_contract(
        &mut self,
        address: String,
        method: String,
        args: Vec<u8>,
        gas_limit: Option<u64>,
    ) -> Result<Vec<u8>, Error> {
        // コントラクトの存在を確認
        if !self.storage.has_contract(&address)? {
            return Err(Error::NotFound(format!("Contract not found: {}", address)));
        }

        // ガス制限を設定
        let gas_limit = gas_limit.unwrap_or(self.config.default_gas_limit);
        if gas_limit > self.config.max_gas_limit {
            return Err(Error::InvalidInput(format!(
                "Gas limit exceeds maximum: {} > {}",
                gas_limit, self.config.max_gas_limit
            )));
        }

        // 実行コンテキストを作成
        let context = ExecutionContext {
            gas_limit,
            sender: "system".to_string(),
            value: 0,
            data: args,
            address: Some(address.clone()),
            block_height: 0,
            block_time: Utc::now(),
            is_static: false,
            depth: 0,
        };

        // コントラクトを呼び出し
        let start_time = std::time::Instant::now();
        let result = self.vm.call(address.clone(), method.clone(), context)?;
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // 統計を更新
        self.update_stats_for_execution(&result, execution_time_ms);

        // メソッド呼び出し統計を更新
        let method_key = format!("{}:{}", address, method);
        let mut stats = self.stats.lock().unwrap();
        stats.total_method_calls += 1;
        let count = stats.calls_by_method.entry(method_key.clone()).or_insert(0);
        *count += 1;

        if *count > stats.most_common_method_count {
            stats.most_common_method = Some(method_key);
            stats.most_common_method_count = *count;
        }

        // イベントログを保存
        self.event_logs
            .extend(result.events.into_iter().map(|event| EventLog {
                address: address.clone(),
                topics: event.topics,
                data: event.data,
                block_height: context.block_height,
                block_time: context.block_time,
                transaction_hash: "".to_string(),
                transaction_index: 0,
                log_index: 0,
                removed: false,
            }));

        // 結果を返す
        Ok(result.return_data)
    }

    /// コントラクトを更新
    pub fn update_contract(
        &mut self,
        address: String,
        code: Vec<u8>,
        gas_limit: Option<u64>,
    ) -> Result<(), Error> {
        // コントラクトの存在を確認
        if !self.storage.has_contract(&address)? {
            return Err(Error::NotFound(format!("Contract not found: {}", address)));
        }

        // コントラクトサイズを検証
        if code.len() > self.config.max_contract_size {
            return Err(Error::InvalidInput(format!(
                "Contract size exceeds maximum: {} > {}",
                code.len(),
                self.config.max_contract_size
            )));
        }

        // コントラクトを検証
        let validation_result = self.validator.validate_contract(&code)?;
        if !validation_result.is_valid {
            return Err(Error::InvalidInput(format!(
                "Contract validation failed: {}",
                validation_result
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        // ガス制限を設定
        let gas_limit = gas_limit.unwrap_or(self.config.default_gas_limit);
        if gas_limit > self.config.max_gas_limit {
            return Err(Error::InvalidInput(format!(
                "Gas limit exceeds maximum: {} > {}",
                gas_limit, self.config.max_gas_limit
            )));
        }

        // 実行コンテキストを作成
        let context = ExecutionContext {
            gas_limit,
            sender: "system".to_string(),
            value: 0,
            data: Vec::new(),
            address: Some(address.clone()),
            block_height: 0,
            block_time: Utc::now(),
            is_static: false,
            depth: 0,
        };

        // コントラクトを更新
        let start_time = std::time::Instant::now();
        let result = self.vm.update(address, code, context)?;
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // 統計を更新
        self.update_stats_for_execution(&result, execution_time_ms);

        // 更新統計を更新
        let mut stats = self.stats.lock().unwrap();
        stats.total_contract_updates += 1;

        // イベントログを保存
        self.event_logs
            .extend(result.events.into_iter().map(|event| EventLog {
                address: result.address.clone(),
                topics: event.topics,
                data: event.data,
                block_height: context.block_height,
                block_time: context.block_time,
                transaction_hash: "".to_string(),
                transaction_index: 0,
                log_index: 0,
                removed: false,
            }));

        Ok(())
    }

    /// コントラクトを削除
    pub fn delete_contract(
        &mut self,
        address: String,
        gas_limit: Option<u64>,
    ) -> Result<(), Error> {
        // コントラクトの存在を確認
        if !self.storage.has_contract(&address)? {
            return Err(Error::NotFound(format!("Contract not found: {}", address)));
        }

        // ガス制限を設定
        let gas_limit = gas_limit.unwrap_or(self.config.default_gas_limit);
        if gas_limit > self.config.max_gas_limit {
            return Err(Error::InvalidInput(format!(
                "Gas limit exceeds maximum: {} > {}",
                gas_limit, self.config.max_gas_limit
            )));
        }

        // 実行コンテキストを作成
        let context = ExecutionContext {
            gas_limit,
            sender: "system".to_string(),
            value: 0,
            data: Vec::new(),
            address: Some(address.clone()),
            block_height: 0,
            block_time: Utc::now(),
            is_static: false,
            depth: 0,
        };

        // コントラクトを削除
        let start_time = std::time::Instant::now();
        let result = self.vm.delete(address.clone(), context)?;
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // 統計を更新
        self.update_stats_for_execution(&result, execution_time_ms);

        // 削除統計を更新
        let mut stats = self.stats.lock().unwrap();
        stats.total_contract_deletions += 1;

        // イベントログを保存
        self.event_logs
            .extend(result.events.into_iter().map(|event| EventLog {
                address,
                topics: event.topics,
                data: event.data,
                block_height: context.block_height,
                block_time: context.block_time,
                transaction_hash: "".to_string(),
                transaction_index: 0,
                log_index: 0,
                removed: false,
            }));

        Ok(())
    }

    /// ガスを見積もり
    pub fn estimate_gas(
        &self,
        address: Option<String>,
        code: Option<Vec<u8>>,
        method: Option<String>,
        args: Vec<u8>,
    ) -> Result<u64, Error> {
        if let Some(address) = address {
            // コントラクト呼び出しのガスを見積もり
            if let Some(method) = method {
                // コントラクトの存在を確認
                if !self.storage.has_contract(&address)? {
                    return Err(Error::NotFound(format!("Contract not found: {}", address)));
                }

                // 実行コンテキストを作成
                let context = ExecutionContext {
                    gas_limit: self.config.max_gas_limit,
                    sender: "system".to_string(),
                    value: 0,
                    data: args,
                    address: Some(address.clone()),
                    block_height: 0,
                    block_time: Utc::now(),
                    is_static: true,
                    depth: 0,
                };

                // ガスを見積もり
                let gas = self
                    .gas_estimator
                    .estimate_call_gas(&address, &method, &context)?;

                Ok(gas)
            } else if let Some(code) = code {
                // コントラクト更新のガスを見積もり
                // コントラクトの存在を確認
                if !self.storage.has_contract(&address)? {
                    return Err(Error::NotFound(format!("Contract not found: {}", address)));
                }

                // 実行コンテキストを作成
                let context = ExecutionContext {
                    gas_limit: self.config.max_gas_limit,
                    sender: "system".to_string(),
                    value: 0,
                    data: Vec::new(),
                    address: Some(address.clone()),
                    block_height: 0,
                    block_time: Utc::now(),
                    is_static: true,
                    depth: 0,
                };

                // ガスを見積もり
                let gas = self
                    .gas_estimator
                    .estimate_update_gas(&address, &code, &context)?;

                Ok(gas)
            } else {
                // コントラクト削除のガスを見積もり
                // コントラクトの存在を確認
                if !self.storage.has_contract(&address)? {
                    return Err(Error::NotFound(format!("Contract not found: {}", address)));
                }

                // 実行コンテキストを作成
                let context = ExecutionContext {
                    gas_limit: self.config.max_gas_limit,
                    sender: "system".to_string(),
                    value: 0,
                    data: Vec::new(),
                    address: Some(address.clone()),
                    block_height: 0,
                    block_time: Utc::now(),
                    is_static: true,
                    depth: 0,
                };

                // ガスを見積もり
                let gas = self.gas_estimator.estimate_delete_gas(&address, &context)?;

                Ok(gas)
            }
        } else if let Some(code) = code {
            // コントラクトデプロイのガスを見積もり
            // 実行コンテキストを作成
            let context = ExecutionContext {
                gas_limit: self.config.max_gas_limit,
                sender: "system".to_string(),
                value: 0,
                data: args,
                address: None,
                block_height: 0,
                block_time: Utc::now(),
                is_static: true,
                depth: 0,
            };

            // ガスを見積もり
            let gas = self.gas_estimator.estimate_deploy_gas(&code, &context)?;

            Ok(gas)
        } else {
            Err(Error::InvalidInput(
                "Either address or code must be provided".to_string(),
            ))
        }
    }

    /// イベントログを取得
    pub fn get_event_logs(&self, filter: Option<&EventFilter>) -> Vec<EventLog> {
        if let Some(filter) = filter {
            // フィルタを適用
            self.event_logs
                .iter()
                .filter(|log| {
                    // アドレスフィルタ
                    if let Some(addresses) = &filter.addresses {
                        if !addresses.contains(&log.address) {
                            return false;
                        }
                    }

                    // トピックフィルタ
                    if let Some(topics) = &filter.topics {
                        for (i, topic) in topics.iter().enumerate() {
                            if let Some(topic) = topic {
                                if i >= log.topics.len() || log.topics[i] != *topic {
                                    return false;
                                }
                            }
                        }
                    }

                    // ブロック高さフィルタ
                    if let Some(from_block) = filter.from_block {
                        if log.block_height < from_block {
                            return false;
                        }
                    }

                    if let Some(to_block) = filter.to_block {
                        if log.block_height > to_block {
                            return false;
                        }
                    }

                    true
                })
                .cloned()
                .collect()
        } else {
            // フィルタなし
            self.event_logs.clone()
        }
    }

    /// 統計を取得
    pub fn get_stats(&self) -> ContractEngineStats {
        self.stats.lock().unwrap().clone()
    }

    /// 設定を取得
    pub fn get_config(&self) -> &ContractEngineConfig {
        &self.config
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: ContractEngineConfig) {
        self.config = config;
    }

    /// 統計をリセット
    pub fn reset_stats(&mut self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = ContractEngineStats::default();
    }

    /// イベントログをクリア
    pub fn clear_event_logs(&mut self) {
        self.event_logs.clear();
    }

    /// 実行統計を更新
    fn update_stats_for_execution(&self, result: &ExecutionResult, execution_time_ms: u64) {
        let mut stats = self.stats.lock().unwrap();

        // 基本統計を更新
        stats.total_executions += 1;
        if result.success {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;

            // エラー統計を更新
            if let Some(error) = &result.error {
                stats.total_errors += 1;
                let error_type = format!("{:?}", error);
                let count = stats.errors_by_type.entry(error_type.clone()).or_insert(0);
                *count += 1;

                if *count > stats.most_common_error_count {
                    stats.most_common_error = Some(error_type);
                    stats.most_common_error_count = *count;
                }

                stats.last_error_time = Some(Utc::now());
                stats.last_error_message = Some(error.to_string());
            }
        }

        // ガス使用量を更新
        stats.total_gas_used += result.gas_used;
        stats.average_gas_used = stats.total_gas_used as f64 / stats.total_executions as f64;
        stats.max_gas_used = stats.max_gas_used.max(result.gas_used);
        if stats.min_gas_used == 0 || result.gas_used < stats.min_gas_used {
            stats.min_gas_used = result.gas_used;
        }

        // 実行時間を更新
        stats.total_execution_time_ms += execution_time_ms;
        stats.average_execution_time_ms =
            stats.total_execution_time_ms as f64 / stats.total_executions as f64;
        stats.max_execution_time_ms = stats.max_execution_time_ms.max(execution_time_ms);
        if stats.min_execution_time_ms == 0 || execution_time_ms < stats.min_execution_time_ms {
            stats.min_execution_time_ms = execution_time_ms;
        }

        // メモリ使用量を更新
        stats.total_memory_used += result.memory_used;
        stats.average_memory_used = stats.total_memory_used as f64 / stats.total_executions as f64;
        stats.max_memory_used = stats.max_memory_used.max(result.memory_used);
        if stats.min_memory_used == 0 || result.memory_used < stats.min_memory_used {
            stats.min_memory_used = result.memory_used;
        }

        // ストレージ使用量を更新
        stats.total_storage_used += result.storage_used;
        stats.average_storage_used =
            stats.total_storage_used as f64 / stats.total_executions as f64;
        stats.max_storage_used = stats.max_storage_used.max(result.storage_used);
        if stats.min_storage_used == 0 || result.storage_used < stats.min_storage_used {
            stats.min_storage_used = result.storage_used;
        }

        // イベント数を更新
        let event_count = result.events.len() as u64;
        stats.total_events += event_count;
        stats.average_events = stats.total_events as f64 / stats.total_executions as f64;
        stats.max_events = stats.max_events.max(event_count);
        if stats.min_events == 0 || event_count < stats.min_events {
            stats.min_events = event_count;
        }

        // ログ数を更新
        let log_count = result.logs.len() as u64;
        stats.total_logs += log_count;
        stats.average_logs = stats.total_logs as f64 / stats.total_executions as f64;
        stats.max_logs = stats.max_logs.max(log_count);
        if stats.min_logs == 0 || log_count < stats.min_logs {
            stats.min_logs = log_count;
        }

        // コントラクト統計を更新
        if result.address.is_empty() {
            stats.total_contract_deployments += 1;
        } else {
            stats.total_contract_calls += 1;
        }

        // ストレージ操作統計を更新
        stats.total_storage_reads += result.storage_reads;
        stats.total_storage_writes += result.storage_writes;
        stats.total_storage_deletes += result.storage_deletes;

        // イベントとログ統計を更新
        stats.total_contract_events += event_count;
        stats.total_contract_logs += log_count;

        // 最後の実行時刻を更新
        stats.last_execution_time = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_contract::storage::{
        MockContractStorage, StorageError, StorageKey, StorageValue,
    };
    use crate::smart_contract::vm::{ExecutionResult, MockVirtualMachine, VMError};

    struct MockVirtualMachine;

    impl VirtualMachine for MockVirtualMachine {
        fn deploy(
            &self,
            code: Vec<u8>,
            context: ExecutionContext,
        ) -> Result<ExecutionResult, VMError> {
            // モックの実装
            Ok(ExecutionResult {
                success: true,
                return_data: Vec::new(),
                gas_used: 1000,
                memory_used: 1000,
                storage_used: 1000,
                storage_reads: 0,
                storage_writes: 1,
                storage_deletes: 0,
                events: Vec::new(),
                logs: Vec::new(),
                address: "contract_address".to_string(),
                error: None,
            })
        }

        fn call(
            &self,
            address: String,
            method: String,
            context: ExecutionContext,
        ) -> Result<ExecutionResult, VMError> {
            // モックの実装
            Ok(ExecutionResult {
                success: true,
                return_data: Vec::new(),
                gas_used: 500,
                memory_used: 500,
                storage_used: 0,
                storage_reads: 1,
                storage_writes: 0,
                storage_deletes: 0,
                events: Vec::new(),
                logs: Vec::new(),
                address,
                error: None,
            })
        }

        fn update(
            &self,
            address: String,
            code: Vec<u8>,
            context: ExecutionContext,
        ) -> Result<ExecutionResult, VMError> {
            // モックの実装
            Ok(ExecutionResult {
                success: true,
                return_data: Vec::new(),
                gas_used: 800,
                memory_used: 800,
                storage_used: 800,
                storage_reads: 1,
                storage_writes: 1,
                storage_deletes: 0,
                events: Vec::new(),
                logs: Vec::new(),
                address,
                error: None,
            })
        }

        fn delete(
            &self,
            address: String,
            context: ExecutionContext,
        ) -> Result<ExecutionResult, VMError> {
            // モックの実装
            Ok(ExecutionResult {
                success: true,
                return_data: Vec::new(),
                gas_used: 300,
                memory_used: 300,
                storage_used: 0,
                storage_reads: 1,
                storage_writes: 0,
                storage_deletes: 1,
                events: Vec::new(),
                logs: Vec::new(),
                address,
                error: None,
            })
        }
    }

    struct MockContractStorage;

    impl ContractStorage for MockContractStorage {
        fn get(&self, key: &StorageKey) -> Result<Option<StorageValue>, StorageError> {
            // モックの実装
            Ok(Some(vec![1, 2, 3]))
        }

        fn set(&mut self, key: StorageKey, value: StorageValue) -> Result<(), StorageError> {
            // モックの実装
            Ok(())
        }

        fn delete(&mut self, key: &StorageKey) -> Result<(), StorageError> {
            // モックの実装
            Ok(())
        }

        fn has(&self, key: &StorageKey) -> Result<bool, StorageError> {
            // モックの実装
            Ok(true)
        }

        fn has_contract(&self, address: &str) -> Result<bool, StorageError> {
            // モックの実装
            Ok(true)
        }

        fn get_contract(&self, address: &str) -> Result<Option<Vec<u8>>, StorageError> {
            // モックの実装
            Ok(Some(vec![1, 2, 3]))
        }

        fn set_contract(&mut self, address: &str, code: Vec<u8>) -> Result<(), StorageError> {
            // モックの実装
            Ok(())
        }

        fn delete_contract(&mut self, address: &str) -> Result<(), StorageError> {
            // モックの実装
            Ok(())
        }

        fn get_contract_storage(
            &self,
            address: &str,
            key: &StorageKey,
        ) -> Result<Option<StorageValue>, StorageError> {
            // モックの実装
            Ok(Some(vec![1, 2, 3]))
        }

        fn set_contract_storage(
            &mut self,
            address: &str,
            key: StorageKey,
            value: StorageValue,
        ) -> Result<(), StorageError> {
            // モックの実装
            Ok(())
        }

        fn delete_contract_storage(
            &mut self,
            address: &str,
            key: &StorageKey,
        ) -> Result<(), StorageError> {
            // モックの実装
            Ok(())
        }

        fn has_contract_storage(
            &self,
            address: &str,
            key: &StorageKey,
        ) -> Result<bool, StorageError> {
            // モックの実装
            Ok(true)
        }

        fn get_contract_storage_keys(
            &self,
            address: &str,
        ) -> Result<Vec<StorageKey>, StorageError> {
            // モックの実装
            Ok(Vec::new())
        }

        fn clear_contract_storage(&mut self, address: &str) -> Result<(), StorageError> {
            // モックの実装
            Ok(())
        }
    }

    #[test]
    fn test_contract_engine() {
        // コントラクトエンジンを作成
        let config = ContractEngineConfig::default();
        let vm = MockVirtualMachine;
        let storage = MockContractStorage;

        let mut engine = ContractEngine::new(config, vm, storage);

        // コントラクトをデプロイ
        let code = vec![1, 2, 3];
        let args = vec![4, 5, 6];
        let result = engine.deploy_contract(code, args, None);
        assert!(result.is_ok());

        // コントラクトを呼び出し
        let address = result.unwrap();
        let method = "test_method".to_string();
        let args = vec![7, 8, 9];
        let result = engine.call_contract(address.clone(), method, args, None);
        assert!(result.is_ok());

        // コントラクトを更新
        let code = vec![10, 11, 12];
        let result = engine.update_contract(address.clone(), code, None);
        assert!(result.is_ok());

        // コントラクトを削除
        let result = engine.delete_contract(address, None);
        assert!(result.is_ok());

        // 統計を確認
        let stats = engine.get_stats();
        assert_eq!(stats.total_executions, 4);
        assert_eq!(stats.successful_executions, 4);
        assert_eq!(stats.failed_executions, 0);
        assert_eq!(stats.total_contract_deployments, 1);
        assert_eq!(stats.total_contract_calls, 2);
        assert_eq!(stats.total_contract_updates, 1);
        assert_eq!(stats.total_contract_deletions, 1);
    }
}
