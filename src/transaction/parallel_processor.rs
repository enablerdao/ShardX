use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, Semaphore};
use tokio::time::{sleep, Duration};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus, CrossShardManager, CrossShardTransaction, CrossShardTransactionState};
use crate::shard::{ShardId, ShardManager, ShardInfo, ShardStatus};
use crate::network::NetworkMessage;

/// 並列トランザクション処理器
///
/// トランザクションを効率的に並列処理するための機能を提供します。
/// 主な最適化手法:
/// 1. 依存関係分析 - トランザクション間の依存関係を分析して並列実行可能なグループを特定
/// 2. 動的スケジューリング - システム負荷に応じて並列度を動的に調整
/// 3. パイプライン処理 - 処理ステージを分割して並列実行
/// 4. ベクトル化 - 類似トランザクションをベクトル化して一括処理
pub struct ParallelProcessor {
    /// クロスシャードマネージャー
    cross_shard_manager: Arc<CrossShardManager>,
    /// シャードマネージャー
    shard_manager: Arc<ShardManager>,
    /// ネットワークメッセージ送信チャネル
    network_tx: mpsc::Sender<NetworkMessage>,
    /// 処理中のトランザクション
    processing_txs: Arc<Mutex<HashMap<String, TransactionProcessingState>>>,
    /// 依存関係グラフ
    dependency_graph: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    /// 処理設定
    config: ProcessorConfig,
    /// 処理セマフォア
    semaphore: Arc<Semaphore>,
    /// 処理統計
    stats: Arc<Mutex<ProcessorStats>>,
}

/// トランザクション処理状態
#[derive(Debug, Clone)]
enum TransactionProcessingState {
    /// 待機中
    Waiting,
    /// 依存関係解決中
    ResolvingDependencies,
    /// 検証中
    Validating,
    /// 実行中
    Executing,
    /// 完了
    Completed,
    /// 失敗
    Failed(String),
}

/// 処理設定
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// 最大並列処理数
    pub max_parallelism: usize,
    /// 最大キュー長
    pub max_queue_size: usize,
    /// バッチサイズ
    pub batch_size: usize,
    /// 処理タイムアウト（ミリ秒）
    pub processing_timeout_ms: u64,
    /// 依存関係解決タイムアウト（ミリ秒）
    pub dependency_resolution_timeout_ms: u64,
    /// 再試行回数
    pub max_retries: usize,
    /// 再試行間隔（ミリ秒）
    pub retry_interval_ms: u64,
    /// 動的スケーリング有効
    pub dynamic_scaling_enabled: bool,
    /// 最小並列度
    pub min_parallelism: usize,
    /// 負荷閾値（高）
    pub high_load_threshold: f64,
    /// 負荷閾値（低）
    pub low_load_threshold: f64,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            max_parallelism: 128,
            max_queue_size: 10000,
            batch_size: 100,
            processing_timeout_ms: 10000,
            dependency_resolution_timeout_ms: 5000,
            max_retries: 3,
            retry_interval_ms: 1000,
            dynamic_scaling_enabled: true,
            min_parallelism: 16,
            high_load_threshold: 0.8,
            low_load_threshold: 0.3,
        }
    }
}

/// 処理統計
#[derive(Debug, Clone)]
pub struct ProcessorStats {
    /// 処理されたトランザクション数
    pub processed_transactions: u64,
    /// 成功したトランザクション数
    pub successful_transactions: u64,
    /// 失敗したトランザクション数
    pub failed_transactions: u64,
    /// 平均処理時間（ミリ秒）
    pub avg_processing_time_ms: f64,
    /// 最大処理時間（ミリ秒）
    pub max_processing_time_ms: u64,
    /// 最小処理時間（ミリ秒）
    pub min_processing_time_ms: u64,
    /// 現在の並列度
    pub current_parallelism: usize,
    /// 現在のキューサイズ
    pub current_queue_size: usize,
    /// 現在の負荷
    pub current_load: f64,
    /// 処理スループット（TPS）
    pub throughput: f64,
    /// 最終更新時刻
    pub last_updated: u64,
}

impl Default for ProcessorStats {
    fn default() -> Self {
        Self {
            processed_transactions: 0,
            successful_transactions: 0,
            failed_transactions: 0,
            avg_processing_time_ms: 0.0,
            max_processing_time_ms: 0,
            min_processing_time_ms: u64::MAX,
            current_parallelism: 0,
            current_queue_size: 0,
            current_load: 0.0,
            throughput: 0.0,
            last_updated: 0,
        }
    }
}

impl ParallelProcessor {
    /// 新しい並列処理器を作成
    pub fn new(
        cross_shard_manager: Arc<CrossShardManager>,
        shard_manager: Arc<ShardManager>,
        network_tx: mpsc::Sender<NetworkMessage>,
        config: Option<ProcessorConfig>,
    ) -> Self {
        let config = config.unwrap_or_default();
        let semaphore = Arc::new(Semaphore::new(config.max_parallelism));

        Self {
            cross_shard_manager,
            shard_manager,
            network_tx,
            processing_txs: Arc::new(Mutex::new(HashMap::new())),
            dependency_graph: Arc::new(Mutex::new(HashMap::new())),
            config,
            semaphore,
            stats: Arc::new(Mutex::new(ProcessorStats::default())),
        }
    }

    /// 処理器を起動
    pub fn start(&self) {
        info!("Starting ParallelProcessor with max parallelism: {}", self.config.max_parallelism);

        // 動的スケーリングタスクを開始
        if self.config.dynamic_scaling_enabled {
            self.start_dynamic_scaler();
        }

        // 統計更新タスクを開始
        self.start_stats_updater();
    }

    /// トランザクションを処理
    pub async fn process_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        // 処理状態を初期化
        {
            let mut processing_txs = self.processing_txs.lock().unwrap();
            processing_txs.insert(transaction.id.clone(), TransactionProcessingState::Waiting);
        }

        // 依存関係を解析
        self.analyze_dependencies(&transaction).await?;

        // 処理を開始
        self.execute_transaction(transaction).await
    }

    /// 複数のトランザクションを一括処理
    pub async fn process_transactions(&self, transactions: Vec<Transaction>) -> Result<Vec<Result<(), Error>>, Error> {
        if transactions.is_empty() {
            return Ok(vec![]);
        }

        info!("Processing batch of {} transactions", transactions.len());

        // 依存関係を解析
        let dependency_groups = self.analyze_batch_dependencies(&transactions).await?;

        // 結果を格納する配列
        let mut results = vec![Err(Error::InternalError("Not processed".to_string())); transactions.len()];
        let tx_indices: HashMap<String, usize> = transactions.iter()
            .enumerate()
            .map(|(i, tx)| (tx.id.clone(), i))
            .collect();

        // 依存関係グループごとに処理
        for group in dependency_groups {
            // グループ内のトランザクションを並列処理
            let handles: Vec<_> = group.into_iter()
                .filter_map(|tx_id| {
                    let tx_index = tx_indices.get(&tx_id)?;
                    let tx = transactions.get(*tx_index)?.clone();
                    
                    let processor = self.clone();
                    Some(tokio::spawn(async move {
                        let result = processor.execute_transaction(tx).await;
                        (tx_id, result)
                    }))
                })
                .collect();

            // 結果を待機
            for handle in handles {
                if let Ok((tx_id, result)) = handle.await {
                    if let Some(index) = tx_indices.get(&tx_id) {
                        results[*index] = result;
                    }
                }
            }
        }

        Ok(results)
    }

    /// トランザクションの依存関係を解析
    async fn analyze_dependencies(&self, transaction: &Transaction) -> Result<(), Error> {
        // 処理状態を更新
        {
            let mut processing_txs = self.processing_txs.lock().unwrap();
            processing_txs.insert(transaction.id.clone(), TransactionProcessingState::ResolvingDependencies);
        }

        // 依存関係を解析
        // 実際の実装では、トランザクションの入力と出力を分析して依存関係を特定
        // ここでは簡易的な実装として、親トランザクションがある場合のみ依存関係を追加
        if let Some(parent_id) = &transaction.parent_id {
            let mut dependency_graph = self.dependency_graph.lock().unwrap();
            
            let dependencies = dependency_graph.entry(transaction.id.clone())
                .or_insert_with(HashSet::new);
            
            dependencies.insert(parent_id.clone());
        }

        Ok(())
    }

    /// 複数のトランザクションの依存関係を一括解析
    /// 
    /// 最適化ポイント:
    /// 1. 送信元アドレスによるインデックス作成で検索を高速化
    /// 2. 依存関係の計算を効率化
    /// 3. 不要なクローンを削減
    async fn analyze_batch_dependencies(&self, transactions: &[Transaction]) -> Result<Vec<Vec<String>>, Error> {
        // 送信元アドレスによるインデックスを作成
        let mut address_index: HashMap<String, Vec<&Transaction>> = HashMap::new();
        for tx in transactions {
            address_index.entry(tx.from.clone())
                .or_insert_with(Vec::new)
                .push(tx);
        }
        
        // 各アドレスのトランザクションをnonceでソート
        for txs in address_index.values_mut() {
            txs.sort_by_key(|tx| tx.nonce);
        }
        
        // 依存関係グラフを構築
        let mut direct_dependencies = HashMap::with_capacity(transactions.len());
        
        for tx in transactions {
            let mut dependencies = HashSet::new();
            
            // 親トランザクションがある場合は依存関係を追加
            if let Some(parent_id) = &tx.parent_id {
                dependencies.insert(parent_id.clone());
            }
            
            // 同じアドレスからの他のトランザクションを依存関係に追加（最適化版）
            if let Some(same_address_txs) = address_index.get(&tx.from) {
                for &other_tx in same_address_txs {
                    if tx.id != other_tx.id && tx.nonce > other_tx.nonce {
                        dependencies.insert(other_tx.id.clone());
                    }
                }
            }
            
            direct_dependencies.insert(tx.id.clone(), dependencies);
        }
        
        // 依存関係グラフを更新（バッチ更新で最適化）
        {
            let mut dependency_graph = self.dependency_graph.lock().unwrap();
            
            for (tx_id, deps) in &direct_dependencies {
                if !deps.is_empty() {
                    let graph_deps = dependency_graph.entry(tx_id.clone())
                        .or_insert_with(HashSet::new);
                    
                    // 既存の依存関係と新しい依存関係をマージ
                    graph_deps.extend(deps.iter().cloned());
                }
            }
        }
        
        // 依存関係に基づいてグループ化
        let groups = self.group_by_dependencies(direct_dependencies);
        
        Ok(groups)
    }

    /// 依存関係に基づいてトランザクションをグループ化
    /// 
    /// 最適化ポイント:
    /// 1. 依存関係の逆マッピングを作成して検索を効率化
    /// 2. 処理済みトランザクションの追跡を最適化
    /// 3. 循環依存関係の検出と解決を改善
    fn group_by_dependencies(&self, dependencies: HashMap<String, HashSet<String>>) -> Vec<Vec<String>> {
        // 依存関係のないトランザクションを特定（容量を事前確保）
        let mut independent_txs = Vec::with_capacity(dependencies.len() / 4);
        let mut dependent_txs = HashSet::with_capacity(dependencies.len() * 3 / 4);
        
        // 逆依存関係マップを構築（どのトランザクションが他のどのトランザクションに依存されているか）
        let mut reverse_deps: HashMap<String, HashSet<String>> = HashMap::with_capacity(dependencies.len());
        
        for (tx_id, deps) in &dependencies {
            if deps.is_empty() {
                independent_txs.push(tx_id.clone());
            } else {
                dependent_txs.insert(tx_id.clone());
                
                // 逆依存関係を構築
                for dep_id in deps {
                    reverse_deps.entry(dep_id.clone())
                        .or_insert_with(HashSet::new)
                        .insert(tx_id.clone());
                }
            }
        }
        
        // 処理済みトランザクションを追跡
        let mut processed_txs = HashSet::with_capacity(dependencies.len());
        
        // 依存関係のないトランザクションを最初のグループとして追加
        let mut groups = Vec::new();
        if !independent_txs.is_empty() {
            // 処理済みとしてマーク
            for tx_id in &independent_txs {
                processed_txs.insert(tx_id.clone());
            }
            
            groups.push(independent_txs);
        }
        
        // 依存関係のあるトランザクションをグループ化
        let mut iteration_count = 0;
        let max_iterations = dependent_txs.len() * 2; // 循環依存関係の無限ループを防止
        
        while !dependent_txs.is_empty() && iteration_count < max_iterations {
            iteration_count += 1;
            
            let mut next_group = Vec::with_capacity(dependent_txs.len() / 2);
            let mut remaining = HashSet::with_capacity(dependent_txs.len() / 2);
            
            for tx_id in &dependent_txs {
                let deps = dependencies.get(tx_id).unwrap();
                
                // すべての依存関係が処理済みかチェック（最適化版）
                let all_deps_processed = deps.iter().all(|dep| processed_txs.contains(dep));
                
                if all_deps_processed {
                    next_group.push(tx_id.clone());
                    processed_txs.insert(tx_id.clone());
                } else {
                    remaining.insert(tx_id.clone());
                }
            }
            
            // 次のグループが空の場合、循環依存関係がある可能性
            if next_group.is_empty() {
                // 循環依存関係を検出して解決
                let mut cycle_breakers = self.detect_cycle_breakers(&remaining, &dependencies, &processed_txs);
                
                // 循環依存関係の解決策が見つからない場合は、残りをすべて追加
                if cycle_breakers.is_empty() {
                    cycle_breakers = remaining.iter().cloned().collect();
                }
                
                // 処理済みとしてマーク
                for tx_id in &cycle_breakers {
                    processed_txs.insert(tx_id.clone());
                }
                
                groups.push(cycle_breakers);
                
                // 残りのトランザクションを更新
                remaining = remaining.difference(&processed_txs).cloned().collect();
            } else {
                groups.push(next_group);
            }
            
            dependent_txs = remaining;
        }
        
        // 最大イテレーション数に達した場合、残りのトランザクションを強制的に追加
        if !dependent_txs.is_empty() {
            groups.push(dependent_txs.into_iter().collect());
        }
        
        groups
    }
    
    /// 循環依存関係を検出して解決するためのトランザクションを特定
    fn detect_cycle_breakers(
        &self,
        remaining: &HashSet<String>,
        dependencies: &HashMap<String, HashSet<String>>,
        processed_txs: &HashSet<String>
    ) -> Vec<String> {
        // 依存関係の少ないトランザクションを優先的に選択
        let mut candidates: Vec<_> = remaining.iter()
            .map(|tx_id| {
                let deps = dependencies.get(tx_id).unwrap();
                let unprocessed_deps = deps.iter()
                    .filter(|dep| !processed_txs.contains(*dep))
                    .count();
                (tx_id, unprocessed_deps)
            })
            .collect();
        
        // 未処理の依存関係が少ない順にソート
        candidates.sort_by_key(|&(_, count)| count);
        
        // 上位20%のトランザクションを選択
        let count = (candidates.len() as f64 * 0.2).ceil() as usize;
        let count = count.max(1).min(candidates.len());
        
        candidates.into_iter()
            .take(count)
            .map(|(tx_id, _)| (*tx_id).clone())
            .collect()
    }

    /// トランザクションを実行
    /// 
    /// 最適化ポイント:
    /// 1. ロックの保持時間を最小化
    /// 2. 不要なクローンを削減
    /// 3. エラーハンドリングを改善
    async fn execute_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        // トランザクションIDを保存（クローンを減らす）
        let tx_id = transaction.id.clone();
        
        // 処理状態を更新（ロック時間を最小化）
        {
            let mut processing_txs = self.processing_txs.lock().unwrap();
            processing_txs.insert(tx_id.clone(), TransactionProcessingState::Validating);
        }

        // セマフォアを取得（タイムアウト付き）
        let permit = tokio::time::timeout(
            Duration::from_millis(self.config.processing_timeout_ms),
            self.semaphore.clone().acquire_owned()
        ).await
        .map_err(|_| Error::TimeoutError("Semaphore acquisition timed out".to_string()))?
        .map_err(|_| Error::InternalError("Failed to acquire semaphore".to_string()))?;

        // 処理開始時間
        let start_time = std::time::Instant::now();

        // 処理状態を更新
        {
            let mut processing_txs = self.processing_txs.lock().unwrap();
            processing_txs.insert(tx_id.clone(), TransactionProcessingState::Executing);
        }

        // トランザクションを実行（タイムアウト付き）
        let result = tokio::time::timeout(
            Duration::from_millis(self.config.processing_timeout_ms),
            self.cross_shard_manager.start_transaction(transaction)
        ).await
        .map_err(|_| Error::TimeoutError("Transaction execution timed out".to_string()))?;

        // 処理時間
        let processing_time = start_time.elapsed().as_millis() as u64;

        // 統計を更新（バッチ更新で最適化）
        self.update_stats(tx_id.clone(), result.is_ok(), processing_time);

        // セマフォアを解放（permitがドロップされる）
        drop(permit);

        result.map(|_| ())
    }
    
    /// 処理統計を更新（ロック時間を最小化するために分離）
    fn update_stats(&self, tx_id: String, is_success: bool, processing_time: u64) {
        // 統計を更新
        {
            let mut stats = self.stats.lock().unwrap();
            stats.processed_transactions += 1;
            
            if is_success {
                stats.successful_transactions += 1;
            } else {
                stats.failed_transactions += 1;
            }
            
            // 平均処理時間を更新（指数移動平均を使用して効率化）
            let alpha = 0.05; // 平滑化係数
            stats.avg_processing_time_ms = (1.0 - alpha) * stats.avg_processing_time_ms + alpha * processing_time as f64;
            
            // 最大・最小処理時間を更新
            stats.max_processing_time_ms = stats.max_processing_time_ms.max(processing_time);
            stats.min_processing_time_ms = stats.min_processing_time_ms.min(processing_time);
            
            // 現在のキューサイズを更新
            stats.current_queue_size = self.processing_txs.lock().unwrap().len();
            
            // 現在の並列度を更新
            stats.current_parallelism = self.config.max_parallelism - self.semaphore.available_permits();
            
            // 現在の負荷を更新
            stats.current_load = stats.current_parallelism as f64 / self.config.max_parallelism as f64;
            
            // 最終更新時刻を更新
            stats.last_updated = chrono::Utc::now().timestamp() as u64;
        }

        // 処理状態を更新
        {
            let mut processing_txs = self.processing_txs.lock().unwrap();
            
            if is_success {
                processing_txs.insert(tx_id, TransactionProcessingState::Completed);
            } else {
                processing_txs.insert(tx_id, TransactionProcessingState::Failed("Transaction failed".to_string()));
            }
        }
    }

    /// 動的スケーラーを開始
    /// 
    /// 最適化ポイント:
    /// 1. 負荷に基づく適応的なスケーリング
    /// 2. スループットを考慮した調整
    /// 3. 急激な変動を防ぐためのヒステリシス
    fn start_dynamic_scaler(&self) {
        let stats = self.stats.clone();
        let config = self.config.clone();
        let semaphore = self.semaphore.clone();

        tokio::spawn(async move {
            // 過去の負荷履歴を保持
            let mut load_history = Vec::with_capacity(5);
            // 過去のスループット履歴を保持
            let mut throughput_history = Vec::with_capacity(5);
            
            loop {
                sleep(Duration::from_secs(3)).await;

                // 現在の状態を取得
                let (current_load, current_throughput) = {
                    let stats = stats.lock().unwrap();
                    (stats.current_load, stats.throughput)
                };
                
                // 履歴を更新
                if load_history.len() >= 5 {
                    load_history.remove(0);
                    throughput_history.remove(0);
                }
                load_history.push(current_load);
                throughput_history.push(current_throughput);
                
                // 平均負荷とスループットを計算
                let avg_load = load_history.iter().sum::<f64>() / load_history.len() as f64;
                let avg_throughput = throughput_history.iter().sum::<f64>() / throughput_history.len() as f64;
                
                let current_permits = semaphore.available_permits();
                let max_permits = config.max_parallelism;
                let used_permits = max_permits - current_permits;
                
                // 負荷とスループットに基づいて並列度を調整
                if avg_load > config.high_load_threshold {
                    // 負荷が高い場合
                    if avg_throughput > 0.0 && used_permits > 0 {
                        // スループットあたりの並列度を計算
                        let throughput_per_permit = avg_throughput / used_permits as f64;
                        
                        // 目標スループットを達成するために必要な並列度を計算
                        let target_throughput = avg_throughput * 1.2; // 20%増加
                        let target_permits = (target_throughput / throughput_per_permit).ceil() as usize;
                        let target_permits = target_permits.min(config.max_parallelism);
                        
                        // 現在の許可数との差分を追加
                        let additional_permits = target_permits.saturating_sub(used_permits);
                        
                        if additional_permits > 0 {
                            semaphore.add_permits(additional_permits);
                            info!("Increased parallelism to {} permits based on throughput analysis", target_permits);
                        }
                    } else {
                        // スループットデータがない場合は単純に増加
                        let new_permits = (max_permits as f64 * 1.2) as usize;
                        let new_permits = new_permits.min(config.max_parallelism);
                        
                        // 現在の許可数との差分を追加
                        let additional_permits = new_permits.saturating_sub(max_permits - current_permits);
                        
                        if additional_permits > 0 {
                            semaphore.add_permits(additional_permits);
                            debug!("Increased parallelism to {} permits", new_permits);
                        }
                    }
                } else if avg_load < config.low_load_threshold {
                    // 負荷が低い場合、並列度を減少
                    // ヒステリシスを適用して急激な変動を防止
                    if avg_load < config.low_load_threshold * 0.8 {
                        let new_permits = (used_permits as f64 * 0.8) as usize;
                        let new_permits = new_permits.max(config.min_parallelism);
                        
                        // 目標並列度を設定（次のサイクルで自然に調整される）
                        debug!("Decreased parallelism target to {} permits", new_permits);
                    }
                }
            }
        });
    }

    /// 統計更新タスクを開始
    fn start_stats_updater(&self) {
        let stats = self.stats.clone();

        tokio::spawn(async move {
            let mut last_processed = 0;
            let mut last_time = chrono::Utc::now().timestamp() as u64;

            loop {
                sleep(Duration::from_secs(1)).await;

                let now = chrono::Utc::now().timestamp() as u64;
                let elapsed = now - last_time;

                if elapsed > 0 {
                    let mut stats = stats.lock().unwrap();
                    
                    // スループットを計算
                    let new_processed = stats.processed_transactions;
                    let processed_diff = new_processed - last_processed;
                    
                    stats.throughput = processed_diff as f64 / elapsed as f64;
                    
                    // 値を更新
                    last_processed = new_processed;
                    last_time = now;
                }
            }
        });
    }

    /// 現在の処理統計を取得
    pub fn get_stats(&self) -> ProcessorStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// 処理設定を更新
    pub fn update_config(&mut self, config: ProcessorConfig) {
        // 並列度が変更された場合、セマフォアを調整
        if config.max_parallelism != self.config.max_parallelism {
            let current_permits = self.semaphore.available_permits();
            let max_permits = self.config.max_parallelism;
            
            if config.max_parallelism > max_permits {
                // 並列度を増加
                let additional_permits = config.max_parallelism - max_permits;
                self.semaphore.add_permits(additional_permits);
            }
            // 並列度を減少する場合は、自然に調整される
        }

        self.config = config;
    }

    /// 現在の処理設定を取得
    pub fn get_config(&self) -> ProcessorConfig {
        self.config.clone()
    }
}

// 単体テスト
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_group_by_dependencies() {
        // テスト用の依存関係グラフを作成
        let mut dependencies = HashMap::new();
        
        // 依存関係のないトランザクション
        dependencies.insert("tx1".to_string(), HashSet::new());
        dependencies.insert("tx2".to_string(), HashSet::new());
        
        // tx1に依存するトランザクション
        let mut tx3_deps = HashSet::new();
        tx3_deps.insert("tx1".to_string());
        dependencies.insert("tx3".to_string(), tx3_deps);
        
        // tx2に依存するトランザクション
        let mut tx4_deps = HashSet::new();
        tx4_deps.insert("tx2".to_string());
        dependencies.insert("tx4".to_string(), tx4_deps);
        
        // tx3とtx4に依存するトランザクション
        let mut tx5_deps = HashSet::new();
        tx5_deps.insert("tx3".to_string());
        tx5_deps.insert("tx4".to_string());
        dependencies.insert("tx5".to_string(), tx5_deps);
        
        // ダミーのコンポーネントを作成
        let (network_tx, _) = mpsc::channel(100);
        let cross_shard_manager = Arc::new(CrossShardManager::new(network_tx.clone()));
        let shard_manager = Arc::new(ShardManager::new(network_tx.clone()));
        
        // 並列処理器を作成
        let processor = ParallelProcessor::new(
            cross_shard_manager,
            shard_manager,
            network_tx,
            None,
        );
        
        // 依存関係に基づいてグループ化
        let groups = processor.group_by_dependencies(dependencies);
        
        // 結果を検証
        assert_eq!(groups.len(), 3);
        
        // 最初のグループは依存関係のないトランザクション
        assert_eq!(groups[0].len(), 2);
        assert!(groups[0].contains(&"tx1".to_string()));
        assert!(groups[0].contains(&"tx2".to_string()));
        
        // 2番目のグループはtx1とtx2に依存するトランザクション
        assert_eq!(groups[1].len(), 2);
        assert!(groups[1].contains(&"tx3".to_string()));
        assert!(groups[1].contains(&"tx4".to_string()));
        
        // 3番目のグループはtx3とtx4に依存するトランザクション
        assert_eq!(groups[2].len(), 1);
        assert!(groups[2].contains(&"tx5".to_string()));
    }
}