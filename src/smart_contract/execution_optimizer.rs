use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time;

use crate::error::Error;
use crate::metrics::MetricsCollector;
use crate::smart_contract::{
    Contract, ContractFunction, ContractState, ExecutionContext, ExecutionResult,
};

/// スマートコントラクト実行最適化器
///
/// スマートコントラクトの実行を最適化し、高速化する。
/// - JITコンパイル
/// - 並列実行
/// - 依存関係の解析
/// - ステート管理の最適化
/// - ホットパスの最適化
pub struct SmartContractExecutionOptimizer {
    /// コントラクトキャッシュ
    contract_cache: Arc<Mutex<lru::LruCache<String, Arc<Contract>>>>,
    /// コンパイル済みコード
    compiled_code: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    /// 実行キュー
    execution_queue: Arc<Mutex<VecDeque<ExecutionTask>>>,
    /// 実行中のタスク
    executing: Arc<Mutex<HashSet<String>>>,
    /// 依存関係グラフ
    dependency_graph: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    /// 最大並列度
    max_parallelism: usize,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// 最後の最適化時刻
    last_optimization: Arc<Mutex<Instant>>,
    /// 最適化間隔（秒）
    optimization_interval_secs: u64,
    /// JITコンパイル閾値（実行回数）
    jit_threshold: u32,
    /// 実行回数カウンター
    execution_counter: Arc<Mutex<HashMap<String, u32>>>,
    /// 実行中フラグ
    running: Arc<Mutex<bool>>,
    /// ホットパス検出閾値
    hot_path_threshold: u32,
    /// ホットパス
    hot_paths: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

/// 実行タスク
#[derive(Debug, Clone)]
pub struct ExecutionTask {
    /// タスクID
    pub id: String,
    /// コントラクトID
    pub contract_id: String,
    /// 関数名
    pub function_name: String,
    /// 引数
    pub arguments: Vec<Vec<u8>>,
    /// 実行コンテキスト
    pub context: ExecutionContext,
    /// 作成時刻
    pub created_at: Instant,
    /// 優先度
    pub priority: u8,
    /// 依存タスクID
    pub depends_on: Option<String>,
    /// 実行タイムアウト（ミリ秒）
    pub timeout_ms: u64,
}

/// 実行モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// インタープリタ
    Interpreter,
    /// JITコンパイル
    JIT,
    /// AOTコンパイル
    AOT,
}

/// 実行優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionPriority {
    /// 低
    Low = 0,
    /// 通常
    Normal = 1,
    /// 高
    High = 2,
    /// 最高
    Critical = 3,
}

impl SmartContractExecutionOptimizer {
    /// 新しいSmartContractExecutionOptimizerを作成
    pub fn new(cache_size: usize, max_parallelism: usize, metrics: Arc<MetricsCollector>) -> Self {
        Self {
            contract_cache: Arc::new(Mutex::new(lru::LruCache::new(cache_size))),
            compiled_code: Arc::new(Mutex::new(HashMap::new())),
            execution_queue: Arc::new(Mutex::new(VecDeque::new())),
            executing: Arc::new(Mutex::new(HashSet::new())),
            dependency_graph: Arc::new(Mutex::new(HashMap::new())),
            max_parallelism,
            metrics,
            last_optimization: Arc::new(Mutex::new(Instant::now())),
            optimization_interval_secs: 60,
            jit_threshold: 5,
            execution_counter: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
            hot_path_threshold: 10,
            hot_paths: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// コントラクトをキャッシュに追加
    pub fn cache_contract(&self, contract: Contract) -> Result<(), Error> {
        let mut contract_cache = self.contract_cache.lock().unwrap();
        contract_cache.put(contract.id.clone(), Arc::new(contract));
        Ok(())
    }

    /// コントラクトをキャッシュから取得
    pub fn get_cached_contract(&self, contract_id: &str) -> Option<Arc<Contract>> {
        let mut contract_cache = self.contract_cache.lock().unwrap();
        contract_cache.get(contract_id).cloned()
    }

    /// 実行タスクを追加
    pub fn add_execution_task(&self, task: ExecutionTask) -> Result<(), Error> {
        // 実行キューに追加
        let mut execution_queue = self.execution_queue.lock().unwrap();

        // 依存関係を解析
        if let Some(depends_on) = &task.depends_on {
            let mut dependency_graph = self.dependency_graph.lock().unwrap();
            let dependencies = dependency_graph
                .entry(task.id.clone())
                .or_insert_with(HashSet::new);
            dependencies.insert(depends_on.clone());
        }

        // タスクをキューに追加
        execution_queue.push_back(task);

        // メトリクスを更新
        self.metrics.increment_counter("execution_tasks_queued");
        self.metrics
            .set_gauge("execution_queue_size", execution_queue.len() as f64);

        Ok(())
    }

    /// 次の実行タスクを取得
    pub fn get_next_task(&self) -> Option<ExecutionTask> {
        let mut execution_queue = self.execution_queue.lock().unwrap();
        let executing = self.executing.lock().unwrap();

        if execution_queue.is_empty() {
            return None;
        }

        // 実行可能なタスクを探す
        let mut i = 0;
        while i < execution_queue.len() {
            let task = execution_queue.get(i).unwrap().clone();

            // 実行中でないか確認
            if !executing.contains(&task.id) {
                // 依存関係をチェック
                let dependency_graph = self.dependency_graph.lock().unwrap();
                let has_unresolved_dependencies =
                    if let Some(dependencies) = dependency_graph.get(&task.id) {
                        dependencies.iter().any(|dep_id| executing.contains(dep_id))
                    } else {
                        false
                    };

                if !has_unresolved_dependencies {
                    // タスクを取得
                    let task = execution_queue.remove(i).unwrap();
                    return Some(task);
                }
            }

            i += 1;
        }

        None
    }

    /// 実行モードを決定
    pub fn determine_execution_mode(
        &self,
        contract_id: &str,
        function_name: &str,
    ) -> ExecutionMode {
        // 実行回数を取得
        let execution_counter = self.execution_counter.lock().unwrap();
        let key = format!("{}:{}", contract_id, function_name);
        let count = execution_counter.get(&key).cloned().unwrap_or(0);

        // コンパイル済みコードをチェック
        let compiled_code = self.compiled_code.lock().unwrap();
        if compiled_code.contains_key(&key) {
            return ExecutionMode::JIT;
        }

        // 実行回数に基づいてモードを決定
        if count >= self.jit_threshold {
            ExecutionMode::JIT
        } else {
            ExecutionMode::Interpreter
        }
    }

    /// コントラクトをJITコンパイル
    pub fn jit_compile(&self, contract: &Contract, function_name: &str) -> Result<Vec<u8>, Error> {
        // 関数を取得
        let function = contract
            .functions
            .iter()
            .find(|f| f.name == function_name)
            .ok_or_else(|| {
                Error::ContractFunctionNotFound(format!("Function not found: {}", function_name))
            })?;

        // コードをコンパイル
        let compiled_code = self.compile_function(contract, function)?;

        // コンパイル済みコードを保存
        let key = format!("{}:{}", contract.id, function_name);
        let mut compiled_code_map = self.compiled_code.lock().unwrap();
        compiled_code_map.insert(key, compiled_code.clone());

        Ok(compiled_code)
    }

    /// 関数をコンパイル
    fn compile_function(
        &self,
        contract: &Contract,
        function: &ContractFunction,
    ) -> Result<Vec<u8>, Error> {
        // 実際の実装では、WebAssemblyやEVMのコードをネイティブコードにコンパイル
        // ここでは簡易的な実装を提供

        // コンパイル開始時間
        let start_time = Instant::now();

        // コンパイル（ダミー実装）
        let mut compiled_code = Vec::new();
        compiled_code.extend_from_slice(b"COMPILED:");
        compiled_code.extend_from_slice(function.code.as_slice());

        // メトリクスを更新
        self.metrics
            .observe_histogram("jit_compilation_time", start_time.elapsed().as_secs_f64());

        Ok(compiled_code)
    }

    /// 実行処理を開始
    pub async fn start_processing<F>(&self, executor: F) -> Result<(), Error>
    where
        F: Fn(ExecutionTask, ExecutionMode) -> Result<ExecutionResult, Error>
            + Send
            + Sync
            + 'static,
    {
        // 既に実行中かチェック
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(Error::InvalidState(
                "Execution optimizer is already running".to_string(),
            ));
        }

        *running = true;
        drop(running);

        // チャネルを作成
        let (task_tx, mut task_rx) = mpsc::channel(self.max_parallelism * 2);

        // タスク取得タスクを開始
        let execution_queue = self.execution_queue.clone();
        let executing = self.executing.clone();
        let dependency_graph = self.dependency_graph.clone();
        let metrics = self.metrics.clone();
        let last_optimization = self.last_optimization.clone();
        let optimization_interval_secs = self.optimization_interval_secs;
        let running = self.running.clone();

        tokio::spawn(async move {
            while *running.lock().unwrap() {
                // 次のタスクを取得
                let task_option = {
                    let self_ref = &self;
                    self_ref.get_next_task()
                };

                if let Some(task) = task_option {
                    // 実行中に追加
                    {
                        let mut executing = executing.lock().unwrap();
                        executing.insert(task.id.clone());
                    }

                    // タスクを送信
                    if let Err(e) = task_tx.send(task.clone()).await {
                        error!("Failed to send task: {}", e);

                        // 実行中から削除
                        let mut executing = executing.lock().unwrap();
                        executing.remove(&e.0.id);
                    }

                    // メトリクスを更新
                    metrics.increment_counter("execution_tasks_dispatched");
                } else {
                    // 最適化を実行
                    let mut last_opt = last_optimization.lock().unwrap();
                    if last_opt.elapsed().as_secs() >= optimization_interval_secs {
                        drop(last_opt);

                        // 依存関係グラフを最適化
                        Self::optimize_dependency_graph(
                            dependency_graph.clone(),
                            executing.clone(),
                            metrics.clone(),
                        );

                        // 最後の最適化時刻を更新
                        *last_optimization.lock().unwrap() = Instant::now();
                    }

                    // 少し待機
                    time::sleep(Duration::from_millis(10)).await;
                }
            }
        });

        // タスク実行タスクを開始
        let executing = self.executing.clone();
        let metrics = self.metrics.clone();
        let execution_counter = self.execution_counter.clone();
        let hot_paths = self.hot_paths.clone();
        let hot_path_threshold = self.hot_path_threshold;
        let running = self.running.clone();

        tokio::spawn(async move {
            // 並列処理用のセマフォ
            let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_parallelism));

            while *running.lock().unwrap() {
                // タスクを受信
                if let Some(task) = task_rx.recv().await {
                    // メトリクスを更新
                    metrics.increment_counter("execution_tasks_received");

                    // セマフォを取得
                    let permit = semaphore.clone().acquire_owned().await.unwrap();

                    // 実行モードを決定
                    let execution_mode = {
                        let self_ref = &self;
                        self_ref.determine_execution_mode(&task.contract_id, &task.function_name)
                    };

                    // 実行関数のクローン
                    let executor = executor.clone();
                    let executing = executing.clone();
                    let metrics = metrics.clone();
                    let execution_counter = execution_counter.clone();
                    let hot_paths = hot_paths.clone();

                    // タスクを実行
                    tokio::spawn(async move {
                        let start_time = Instant::now();

                        // 実行回数をインクリメント
                        {
                            let mut counter = execution_counter.lock().unwrap();
                            let key = format!("{}:{}", task.contract_id, task.function_name);
                            let count = counter.entry(key.clone()).or_insert(0);
                            *count += 1;

                            // ホットパスを検出
                            if *count >= hot_path_threshold {
                                let mut hot_paths = hot_paths.lock().unwrap();
                                let paths = hot_paths
                                    .entry(task.contract_id.clone())
                                    .or_insert_with(Vec::new);
                                if !paths.contains(&task.function_name) {
                                    paths.push(task.function_name.clone());
                                }
                            }
                        }

                        // タイムアウト付きで実行
                        let timeout_duration = Duration::from_millis(task.timeout_ms);
                        let execution_future = executor(task.clone(), execution_mode);

                        let result =
                            tokio::time::timeout(timeout_duration, async { execution_future })
                                .await;

                        match result {
                            Ok(Ok(execution_result)) => {
                                // 実行成功
                                let execution_time = start_time.elapsed();

                                // メトリクスを更新
                                metrics.observe_histogram(
                                    "execution_time",
                                    execution_time.as_secs_f64(),
                                );
                                metrics.increment_counter("execution_tasks_completed");

                                // 実行モードに応じたメトリクスを更新
                                match execution_mode {
                                    ExecutionMode::Interpreter => {
                                        metrics.increment_counter("interpreter_executions");
                                    }
                                    ExecutionMode::JIT => {
                                        metrics.increment_counter("jit_executions");
                                    }
                                    ExecutionMode::AOT => {
                                        metrics.increment_counter("aot_executions");
                                    }
                                }

                                // 実行結果に応じたメトリクスを更新
                                if execution_result.success {
                                    metrics.increment_counter("execution_success");
                                } else {
                                    metrics.increment_counter("execution_failure");
                                }

                                // ガス使用量を記録
                                if let Some(gas_used) = execution_result.gas_used {
                                    metrics.observe_histogram("gas_used", gas_used as f64);
                                }
                            }
                            Ok(Err(e)) => {
                                // 実行エラー
                                error!("Execution error: {}", e);
                                metrics.increment_counter("execution_errors");
                            }
                            Err(_) => {
                                // タイムアウト
                                error!("Execution timeout: {}", task.id);
                                metrics.increment_counter("execution_timeouts");
                            }
                        }

                        // 実行中から削除
                        let mut executing = executing.lock().unwrap();
                        executing.remove(&task.id);

                        // セマフォを解放（permitがドロップされる）
                        drop(permit);
                    });
                } else {
                    // チャネルが閉じられた場合は終了
                    break;
                }
            }
        });

        Ok(())
    }

    /// 処理を停止
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }

    /// 依存関係グラフを最適化
    fn optimize_dependency_graph(
        dependency_graph: Arc<Mutex<HashMap<String, HashSet<String>>>>,
        executing: Arc<Mutex<HashSet<String>>>,
        metrics: Arc<MetricsCollector>,
    ) {
        // 依存関係グラフを取得
        let mut dependency_graph = dependency_graph.lock().unwrap();
        let executing = executing.lock().unwrap();

        // 実行中でないタスクの依存関係を削除
        dependency_graph.retain(|task_id, _| executing.contains(task_id));

        // 循環依存を検出
        let mut cycles = 0;

        for (task_id, dependencies) in dependency_graph.iter() {
            // 依存関係のパスを探索
            let mut visited = HashSet::new();
            let mut path = Vec::new();

            fn detect_cycle(
                task_id: &str,
                dependency_graph: &HashMap<String, HashSet<String>>,
                visited: &mut HashSet<String>,
                path: &mut Vec<String>,
            ) -> bool {
                if path.contains(&task_id.to_string()) {
                    return true;
                }

                if visited.contains(task_id) {
                    return false;
                }

                visited.insert(task_id.to_string());
                path.push(task_id.to_string());

                if let Some(deps) = dependency_graph.get(task_id) {
                    for dep in deps {
                        if detect_cycle(dep, dependency_graph, visited, path) {
                            return true;
                        }
                    }
                }

                path.pop();
                false
            }

            if detect_cycle(task_id, &dependency_graph, &mut visited, &mut path) {
                cycles += 1;
            }
        }

        // メトリクスを更新
        metrics.set_gauge("dependency_graph_size", dependency_graph.len() as f64);
        metrics.set_gauge("dependency_cycles", cycles as f64);
    }

    /// キューのサイズを取得
    pub fn get_queue_size(&self) -> usize {
        self.execution_queue.lock().unwrap().len()
    }

    /// 実行中のタスク数を取得
    pub fn get_executing_count(&self) -> usize {
        self.executing.lock().unwrap().len()
    }

    /// コンパイル済みコード数を取得
    pub fn get_compiled_code_count(&self) -> usize {
        self.compiled_code.lock().unwrap().len()
    }

    /// ホットパスを取得
    pub fn get_hot_paths(&self, contract_id: &str) -> Vec<String> {
        let hot_paths = self.hot_paths.lock().unwrap();
        hot_paths.get(contract_id).cloned().unwrap_or_default()
    }

    /// JITコンパイル閾値を設定
    pub fn set_jit_threshold(&mut self, threshold: u32) {
        self.jit_threshold = threshold;
    }

    /// 最大並列度を設定
    pub fn set_max_parallelism(&mut self, max_parallelism: usize) {
        self.max_parallelism = max_parallelism;
    }

    /// 最適化間隔を設定
    pub fn set_optimization_interval_secs(&mut self, optimization_interval_secs: u64) {
        self.optimization_interval_secs = optimization_interval_secs;
    }

    /// ホットパス検出閾値を設定
    pub fn set_hot_path_threshold(&mut self, threshold: u32) {
        self.hot_path_threshold = threshold;
    }
}
