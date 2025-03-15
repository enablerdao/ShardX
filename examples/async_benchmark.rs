use std::time::{Duration, Instant};
use clap::{App, Arg};
use log::{info, error};
use prettytable::{Table, row, cell};

use shardx::async_runtime::{self, AsyncRuntimeConfig, TaskPriority};
use shardx::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // コマンドライン引数を解析
    let matches = App::new("ShardX Async Runtime Benchmark")
        .version("1.0")
        .author("ShardX Team")
        .about("Benchmarks async runtime performance")
        .arg(Arg::with_name("tasks")
            .short("t")
            .long("tasks")
            .value_name("COUNT")
            .help("Number of tasks to process")
            .default_value("10000"))
        .arg(Arg::with_name("workers")
            .short("w")
            .long("workers")
            .value_name("COUNT")
            .help("Number of worker threads")
            .default_value("4"))
        .arg(Arg::with_name("dependencies")
            .short("d")
            .long("dependencies")
            .help("Enable task dependencies"))
        .arg(Arg::with_name("priorities")
            .short("p")
            .long("priorities")
            .help("Enable task priorities"))
        .get_matches();
    
    // 引数を取得
    let task_count = matches.value_of("tasks")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(10000);
    
    let worker_count = matches.value_of("workers")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(4);
    
    let use_dependencies = matches.is_present("dependencies");
    let use_priorities = matches.is_present("priorities");
    
    // 非同期ランタイムを初期化
    let config = AsyncRuntimeConfig {
        worker_threads: worker_count,
        task_queue_capacity: task_count * 2,
        priority_levels: 3,
        scheduler_interval_ms: 10,
        max_task_execution_time_ms: 1000,
        max_task_retries: 3,
    };
    
    async_runtime::init(Some(config));
    
    println!("Running async runtime benchmark with {} tasks, {} workers", task_count, worker_count);
    println!("Dependencies: {}, Priorities: {}", use_dependencies, use_priorities);
    
    // ベンチマークを実行
    let start_time = Instant::now();
    
    // タスクを作成
    let mut task_ids = Vec::with_capacity(task_count);
    
    for i in 0..task_count {
        let dependencies = if use_dependencies && i > 0 && i % 10 == 0 {
            // 10個ごとに前のタスクに依存
            vec![task_ids[i - 1]]
        } else {
            Vec::new()
        };
        
        let priority = if use_priorities {
            match i % 3 {
                0 => TaskPriority::Low,
                1 => TaskPriority::Normal,
                _ => TaskPriority::High,
            }
        } else {
            TaskPriority::Normal
        };
        
        // タスクを作成（単純な計算タスク）
        let task_id = async_runtime::spawn_with_priority(async move {
            // 単純な計算を実行
            let mut sum = 0;
            for j in 0..1000 {
                sum += j;
            }
            
            // 少し待機
            tokio::time::sleep(Duration::from_micros(10)).await;
        }, priority, dependencies);
        
        task_ids.push(task_id);
    }
    
    // すべてのタスクが完了するのを待つ
    let mut completed = false;
    while !completed {
        let (pending, running, completed_count, failed) = async_runtime::get_task_count();
        
        if pending == 0 && running == 0 {
            completed = true;
        }
        
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis() as f64;
    let tasks_per_second = (task_count as f64) / (elapsed.as_secs_f64());
    
    // 結果を表示
    let mut table = Table::new();
    
    table.add_row(row!["Metric", "Value"]);
    table.add_row(row!["Task Count", task_count]);
    table.add_row(row!["Worker Threads", worker_count]);
    table.add_row(row!["Dependencies", use_dependencies]);
    table.add_row(row!["Priorities", use_priorities]);
    table.add_row(row!["Total Time", format!("{:.2} ms", elapsed_ms)]);
    table.add_row(row!["Tasks Per Second", format!("{:.2}", tasks_per_second)]);
    table.add_row(row!["Average Task Time", format!("{:.2} µs", elapsed_ms * 1000.0 / task_count as f64)]);
    
    table.printstd();
    
    // 非同期ランタイムを停止
    async_runtime::shutdown();
    
    Ok(())
}