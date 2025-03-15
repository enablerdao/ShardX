use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use log::{debug, error, info, warn};

/// 非同期ランタイム設定
#[derive(Debug, Clone)]
pub struct AsyncRuntimeConfig {
    /// ワーカースレッド数
    pub worker_threads: usize,
    /// タスクキューの容量
    pub task_queue_capacity: usize,
    /// タスクの優先度レベル数
    pub priority_levels: usize,
    /// スケジューラの実行間隔（ミリ秒）
    pub scheduler_interval_ms: u64,
    /// タスクの最大実行時間（ミリ秒）
    pub max_task_execution_time_ms: u64,
    /// タスクの最大再試行回数
    pub max_task_retries: usize,
}

impl Default for AsyncRuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            task_queue_capacity: 10000,
            priority_levels: 3,
            scheduler_interval_ms: 10,
            max_task_execution_time_ms: 1000,
            max_task_retries: 3,
        }
    }
}

/// タスク優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// 低優先度
    Low = 0,
    /// 通常優先度
    Normal = 1,
    /// 高優先度
    High = 2,
}

impl TaskPriority {
    /// 優先度レベルを取得
    pub fn level(&self) -> usize {
        *self as usize
    }
    
    /// 整数から優先度を取得
    pub fn from_level(level: usize) -> Self {
        match level {
            0 => TaskPriority::Low,
            1 => TaskPriority::Normal,
            _ => TaskPriority::High,
        }
    }
}

/// タスク状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// 待機中
    Pending,
    /// 実行中
    Running,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// キャンセル
    Canceled,
}

/// タスク情報
struct TaskInfo {
    /// タスクID
    id: u64,
    /// 優先度
    priority: TaskPriority,
    /// 状態
    state: TaskState,
    /// 作成時刻
    created_at: Instant,
    /// 最終実行時刻
    last_executed_at: Option<Instant>,
    /// 実行回数
    execution_count: usize,
    /// 依存タスク
    dependencies: Vec<u64>,
}

/// タスク
struct Task {
    /// タスク情報
    info: TaskInfo,
    /// フューチャー
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    /// ウェイカー
    waker: Option<Waker>,
}

/// タスクウェイカー
struct TaskWaker {
    /// タスクID
    task_id: u64,
    /// タスクキュー
    task_queue: Arc<Mutex<TaskQueue>>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        let mut task_queue = self.task_queue.lock().unwrap();
        task_queue.wake_task(self.task_id);
    }
    
    fn wake_by_ref(self: &Arc<Self>) {
        let mut task_queue = self.task_queue.lock().unwrap();
        task_queue.wake_task(self.task_id);
    }
}

/// タスクキュー
struct TaskQueue {
    /// 次のタスクID
    next_task_id: u64,
    /// タスクマップ
    tasks: HashMap<u64, Task>,
    /// 優先度キュー
    priority_queues: Vec<VecDeque<u64>>,
    /// 実行中のタスク
    running_tasks: HashMap<u64, Instant>,
    /// 完了したタスク
    completed_tasks: VecDeque<u64>,
    /// 失敗したタスク
    failed_tasks: VecDeque<u64>,
    /// 最大容量
    capacity: usize,
    /// 優先度レベル数
    priority_levels: usize,
    /// 最大実行時間
    max_execution_time: Duration,
    /// 最大再試行回数
    max_retries: usize,
}

impl TaskQueue {
    /// 新しいタスクキューを作成
    fn new(config: &AsyncRuntimeConfig) -> Self {
        let priority_queues = (0..config.priority_levels)
            .map(|_| VecDeque::new())
            .collect();
        
        Self {
            next_task_id: 1,
            tasks: HashMap::new(),
            priority_queues,
            running_tasks: HashMap::new(),
            completed_tasks: VecDeque::new(),
            failed_tasks: VecDeque::new(),
            capacity: config.task_queue_capacity,
            priority_levels: config.priority_levels,
            max_execution_time: Duration::from_millis(config.max_task_execution_time_ms),
            max_retries: config.max_task_retries,
        }
    }
    
    /// タスクを追加
    fn add_task<F>(&mut self, priority: TaskPriority, future: F, dependencies: Vec<u64>) -> u64
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // キューが満杯の場合はエラー
        if self.tasks.len() >= self.capacity {
            error!("Task queue is full, cannot add more tasks");
            return 0;
        }
        
        let task_id = self.next_task_id;
        self.next_task_id += 1;
        
        let info = TaskInfo {
            id: task_id,
            priority,
            state: TaskState::Pending,
            created_at: Instant::now(),
            last_executed_at: None,
            execution_count: 0,
            dependencies,
        };
        
        let task = Task {
            info,
            future: Box::pin(future),
            waker: None,
        };
        
        self.tasks.insert(task_id, task);
        
        // 依存関係がない場合はキューに追加
        if dependencies.is_empty() {
            let priority_level = priority.level();
            if priority_level < self.priority_queues.len() {
                self.priority_queues[priority_level].push_back(task_id);
            } else {
                self.priority_queues[0].push_back(task_id);
            }
        }
        
        task_id
    }
    
    /// 次のタスクを取得
    fn next_task(&mut self) -> Option<(u64, &mut Task)> {
        // 優先度の高いキューから順に取得
        for priority in (0..self.priority_levels).rev() {
            let mut task_id_opt = None;
            
            if let Some(queue) = self.priority_queues.get_mut(priority) {
                if let Some(&task_id) = queue.front() {
                    // 依存関係をチェック
                    let dependencies_completed = {
                        if let Some(task) = self.tasks.get(&task_id) {
                            let dependencies = &task.info.dependencies;
                            dependencies.iter().all(|dep_id| {
                                if let Some(dep_task) = self.tasks.get(dep_id) {
                                    dep_task.info.state == TaskState::Completed
                                } else {
                                    true // 依存タスクが存在しない場合は完了とみなす
                                }
                            })
                        } else {
                            false
                        }
                    };
                    
                    if dependencies_completed {
                        // 依存関係が完了している場合はキューから取り出す
                        task_id_opt = queue.pop_front();
                    } else {
                        // 依存関係が完了していない場合は後ろに移動
                        if let Some(id) = queue.pop_front() {
                            queue.push_back(id);
                        }
                        continue;
                    }
                }
            }
            
            // タスクIDが取得できた場合
            if let Some(task_id) = task_id_opt {
                if let Some(task) = self.tasks.get_mut(&task_id) {
                    // タスクを実行中に設定
                    task.info.state = TaskState::Running;
                    task.info.last_executed_at = Some(Instant::now());
                    task.info.execution_count += 1;
                    
                    // 実行中のタスクに追加
                    self.running_tasks.insert(task_id, Instant::now());
                    
                    return Some((task_id, task));
                }
            }
        }
        
        None
    }
    
    /// タスクを起こす
    fn wake_task(&mut self, task_id: u64) {
        // タスクの状態と優先度を取得
        let task_info = {
            if let Some(task) = self.tasks.get_mut(&task_id) {
                if task.info.state == TaskState::Running {
                    // タスクが実行中の場合は何もしない
                    return;
                }
                
                // タスクを待機中に設定
                task.info.state = TaskState::Pending;
                
                // 情報をコピー
                (
                    task.info.state,
                    task.info.priority.level(),
                    task.info.dependencies.len() == 0 // 依存関係がないかどうか
                )
            } else {
                return;
            }
        };
        
        let (task_state, priority_level, no_dependencies) = task_info;
        
        // 依存関係がない場合はすぐにキューに追加
        if no_dependencies && task_state == TaskState::Pending {
            // キューに追加
            if priority_level < self.priority_queues.len() {
                if let Some(queue) = self.priority_queues.get_mut(priority_level) {
                    queue.push_back(task_id);
                }
            } else if !self.priority_queues.is_empty() {
                if let Some(queue) = self.priority_queues.get_mut(0) {
                    queue.push_back(task_id);
                }
            }
            return;
        }
        
        // 依存関係がある場合は依存関係をチェック
        let dependencies_completed = {
            let mut completed = true;
            if let Some(task) = self.tasks.get(&task_id) {
                for dep_id in &task.info.dependencies {
                    if let Some(dep_task) = self.tasks.get(dep_id) {
                        if dep_task.info.state != TaskState::Completed {
                            completed = false;
                            break;
                        }
                    }
                }
            }
            completed
        };
        
        if dependencies_completed && task_state == TaskState::Pending {
            // 依存関係が完了している場合はキューに追加
            if priority_level < self.priority_queues.len() {
                if let Some(queue) = self.priority_queues.get_mut(priority_level) {
                    queue.push_back(task_id);
                }
            } else if !self.priority_queues.is_empty() {
                if let Some(queue) = self.priority_queues.get_mut(0) {
                    queue.push_back(task_id);
                }
            }
        }
    }
    
    /// タスクを完了
    fn complete_task(&mut self, task_id: u64) {
        // タスクを完了状態に設定
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.info.state = TaskState::Completed;
            self.running_tasks.remove(&task_id);
            self.completed_tasks.push_back(task_id);
        } else {
            return;
        }
        
        // 依存タスクのIDを収集
        let dependent_task_ids: Vec<u64> = self.tasks.iter()
            .filter_map(|(id, task)| {
                if task.info.dependencies.contains(&task_id) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();
        
        // 依存タスクを起こす
        for id in dependent_task_ids {
            self.wake_task(id);
        }
    }
    
    /// タスクを失敗
    fn fail_task(&mut self, task_id: u64) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            // 最大再試行回数をチェック
            if task.info.execution_count < self.max_retries {
                // 再試行
                task.info.state = TaskState::Pending;
                let priority_level = task.info.priority.level();
                if priority_level < self.priority_queues.len() {
                    self.priority_queues[priority_level].push_back(task_id);
                } else {
                    self.priority_queues[0].push_back(task_id);
                }
            } else {
                // 最大再試行回数を超えた場合は失敗
                task.info.state = TaskState::Failed;
                self.running_tasks.remove(&task_id);
                self.failed_tasks.push_back(task_id);
            }
        }
    }
    
    /// タイムアウトしたタスクをチェック
    fn check_timeouts(&mut self) {
        let now = Instant::now();
        let timed_out_tasks: Vec<u64> = self.running_tasks.iter()
            .filter(|(_, start_time)| now.duration_since(**start_time) > self.max_execution_time)
            .map(|(task_id, _)| *task_id)
            .collect();
        
        for task_id in timed_out_tasks {
            warn!("Task {} timed out", task_id);
            self.fail_task(task_id);
        }
    }
    
    /// キューをクリーンアップ
    fn cleanup(&mut self) {
        // 完了したタスクを削除
        while let Some(task_id) = self.completed_tasks.pop_front() {
            self.tasks.remove(&task_id);
        }
        
        // 失敗したタスクを削除
        while let Some(task_id) = self.failed_tasks.pop_front() {
            self.tasks.remove(&task_id);
        }
    }
}

/// 非同期ランタイム
pub struct AsyncRuntime {
    /// タスクキュー
    task_queue: Arc<Mutex<TaskQueue>>,
    /// ワーカースレッド
    _worker_threads: Vec<thread::JoinHandle<()>>,
    /// スケジューラスレッド
    _scheduler_thread: thread::JoinHandle<()>,
    /// 実行中フラグ
    running: Arc<Mutex<bool>>,
    /// 設定
    config: AsyncRuntimeConfig,
}

impl AsyncRuntime {
    /// 新しい非同期ランタイムを作成
    pub fn new(config: Option<AsyncRuntimeConfig>) -> Self {
        let config = config.unwrap_or_default();
        let task_queue = Arc::new(Mutex::new(TaskQueue::new(&config)));
        let running = Arc::new(Mutex::new(true));
        
        // ワーカースレッドを作成
        let mut worker_threads = Vec::with_capacity(config.worker_threads);
        
        for i in 0..config.worker_threads {
            let task_queue = task_queue.clone();
            let running = running.clone();
            
            let handle = thread::spawn(move || {
                info!("Worker thread {} started", i);
                
                while *running.lock().unwrap() {
                    // タスクを実行
                    let result = Self::execute_task(task_queue.clone());
                    
                    if !result {
                        // タスクがない場合は少し待機
                        thread::sleep(Duration::from_millis(1));
                    }
                }
                
                info!("Worker thread {} stopped", i);
            });
            
            worker_threads.push(handle);
        }
        
        // スケジューラスレッドを作成
        let task_queue_scheduler = task_queue.clone();
        let running_scheduler = running.clone();
        let scheduler_interval = Duration::from_millis(config.scheduler_interval_ms);
        
        let scheduler_thread = thread::spawn(move || {
            info!("Scheduler thread started");
            
            while *running_scheduler.lock().unwrap() {
                // タイムアウトをチェック
                {
                    let mut task_queue = task_queue_scheduler.lock().unwrap();
                    task_queue.check_timeouts();
                    task_queue.cleanup();
                }
                
                // 少し待機
                thread::sleep(scheduler_interval);
            }
            
            info!("Scheduler thread stopped");
        });
        
        Self {
            task_queue,
            _worker_threads: worker_threads,
            _scheduler_thread: scheduler_thread,
            running,
            config,
        }
    }
    
    /// タスクを実行
    fn execute_task(task_queue: Arc<Mutex<TaskQueue>>) -> bool {
        // タスクを取得
        let task_id_opt = {
            let mut task_queue = task_queue.lock().unwrap();
            task_queue.next_task().map(|(id, _)| id)
        };
        
        if let Some(task_id) = task_id_opt {
            // ウェイカーを作成
            let waker = {
                let task_waker = Arc::new(TaskWaker {
                    task_id,
                    task_queue: task_queue.clone(),
                });
                
                Waker::from(task_waker)
            };
            
            let mut context = Context::from_waker(&waker);
            
            // フューチャーを実行
            let poll_result = {
                let mut task_queue = task_queue.lock().unwrap();
                if let Some(task) = task_queue.tasks.get_mut(&task_id) {
                    task.future.as_mut().poll(&mut context)
                } else {
                    return false;
                }
            };
            
            match poll_result {
                Poll::Ready(()) => {
                    // タスクが完了
                    let mut task_queue = task_queue.lock().unwrap();
                    task_queue.complete_task(task_id);
                    true
                }
                Poll::Pending => {
                    // タスクがまだ完了していない
                    true
                }
            }
        } else {
            false
        }
    }
    
    /// タスクを追加
    pub fn spawn<F>(&self, future: F) -> u64
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn_with_priority(future, TaskPriority::Normal, Vec::new())
    }
    
    /// 優先度付きでタスクを追加
    pub fn spawn_with_priority<F>(&self, future: F, priority: TaskPriority, dependencies: Vec<u64>) -> u64
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut task_queue = self.task_queue.lock().unwrap();
        task_queue.add_task(priority, future, dependencies)
    }
    
    /// ランタイムを停止
    pub fn shutdown(&self) {
        info!("Shutting down async runtime");
        
        // 実行中フラグをfalseに設定
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
    
    /// タスクの状態を取得
    pub fn get_task_state(&self, task_id: u64) -> Option<TaskState> {
        let task_queue = self.task_queue.lock().unwrap();
        
        task_queue.tasks.get(&task_id).map(|task| task.info.state)
    }
    
    /// タスクの数を取得
    pub fn get_task_count(&self) -> (usize, usize, usize, usize) {
        let task_queue = self.task_queue.lock().unwrap();
        
        let pending = task_queue.priority_queues.iter().map(|q| q.len()).sum();
        let running = task_queue.running_tasks.len();
        let completed = task_queue.completed_tasks.len();
        let failed = task_queue.failed_tasks.len();
        
        (pending, running, completed, failed)
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> AsyncRuntimeConfig {
        self.config.clone()
    }
}

impl Drop for AsyncRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// グローバル非同期ランタイム
static mut GLOBAL_RUNTIME: Option<AsyncRuntime> = None;

/// グローバル非同期ランタイムを初期化
pub fn init(config: Option<AsyncRuntimeConfig>) {
    unsafe {
        if GLOBAL_RUNTIME.is_none() {
            GLOBAL_RUNTIME = Some(AsyncRuntime::new(config));
        }
    }
}

/// グローバル非同期ランタイムを取得
pub fn runtime() -> &'static AsyncRuntime {
    unsafe {
        if GLOBAL_RUNTIME.is_none() {
            GLOBAL_RUNTIME = Some(AsyncRuntime::new(None));
        }
        
        GLOBAL_RUNTIME.as_ref().unwrap()
    }
}

/// タスクを追加
pub fn spawn<F>(future: F) -> u64
where
    F: Future<Output = ()> + Send + 'static,
{
    runtime().spawn(future)
}

/// 優先度付きでタスクを追加
pub fn spawn_with_priority<F>(future: F, priority: TaskPriority, dependencies: Vec<u64>) -> u64
where
    F: Future<Output = ()> + Send + 'static,
{
    runtime().spawn_with_priority(future, priority, dependencies)
}

/// タスクの状態を取得
pub fn get_task_state(task_id: u64) -> Option<TaskState> {
    runtime().get_task_state(task_id)
}

/// タスクの数を取得
pub fn get_task_count() -> (usize, usize, usize, usize) {
    runtime().get_task_count()
}

/// 非同期ランタイムを停止
pub fn shutdown() {
    unsafe {
        if let Some(runtime) = &GLOBAL_RUNTIME {
            runtime.shutdown();
        }
    }
}
