mod ai;
mod api;
mod api_handlers;
mod consensus;
mod dex;
mod node;
mod sharding;
mod transaction;
mod wallet;

use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use api::ApiServer;
use dex::DexManager;
use log::{info, error};
use node::{Node, NodeConfig};
use tokio::sync::Mutex;
use wallet::WalletManager;

#[tokio::main]
async fn main() {
    // コマンドライン引数の解析
    let args: Vec<String> = env::args().collect();
    
    let mut node_id = format!("node_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    let mut port = 54868;
    let mut data_dir = "./data".to_string();
    let mut log_level = "info".to_string();
    let mut shard_count = 256;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--node-id" => {
                if i + 1 < args.len() {
                    node_id = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse() {
                        port = p;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--data-dir" => {
                if i + 1 < args.len() {
                    data_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--log-level" => {
                if i + 1 < args.len() {
                    log_level = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--shard-count" => {
                if i + 1 < args.len() {
                    if let Ok(sc) = args[i + 1].parse() {
                        shard_count = sc;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--help" => {
                println!("使用方法: shardx [オプション]");
                println!("");
                println!("オプション:");
                println!("  --node-id ID       ノードID (デフォルト: ランダム生成)");
                println!("  --port PORT        APIポート (デフォルト: 54868)");
                println!("  --data-dir DIR     データディレクトリ (デフォルト: ./data)");
                println!("  --log-level LEVEL  ログレベル (debug, info, warn, error) (デフォルト: info)");
                println!("  --shard-count N    シャード数 (デフォルト: 256)");
                println!("  --help             このヘルプメッセージを表示");
                return;
            }
            _ => {
                i += 1;
            }
        }
    }
    
    // ロガーを初期化
    std::env::set_var("RUST_LOG", log_level.clone());
    env_logger::init_from_env(env_logger::Env::default().default_filter_or(&log_level));
    
    // データディレクトリの作成
    if !Path::new(&data_dir).exists() {
        if let Err(e) = fs::create_dir_all(&data_dir) {
            error!("データディレクトリの作成に失敗しました: {}", e);
            return;
        }
    }
    
    info!("ShardX ノードを起動中...");
    info!("ノードID: {}", node_id);
    info!("ポート: {}", port);
    info!("データディレクトリ: {}", data_dir);
    info!("ログレベル: {}", log_level);
    
    // ノード設定を作成
    let mut config = NodeConfig::default();
    config.node_id = node_id;
    config.port = port;
    config.data_dir = data_dir;
    config.shard_count = shard_count;
    
    info!("DAGを初期化中...");
    
    // ノードを作成
    let mut node = Node::new(config);
    
    info!("シャーディングマネージャを初期化中 (シャード数: {})...", shard_count);
    
    // ノードを起動
    info!("コンセンサスエンジンを初期化中...");
    node.start().await;
    
    // ノードをArcでラップ
    let node = Arc::new(Mutex::new(node));
    
    // ウォレットマネージャーを作成
    let wallet_manager = Arc::new(WalletManager::new());
    
    // DEXマネージャーを作成
    let dex_manager = Arc::new(DexManager::new(Arc::clone(&wallet_manager)));
    
    // 初期データを設定
    initialize_demo_data(&wallet_manager, &dex_manager).await;
    
    info!("APIサーバーを起動中 (ポート: {})...", port);
    
    // APIサーバーを作成
    let api_server = ApiServer::new(
        Arc::clone(&node),
        Arc::clone(&wallet_manager),
        Arc::clone(&dex_manager),
        port
    );
    
    // APIサーバーを起動
    match api_server.start().await {
        Ok(_) => info!("APIサーバーが正常に終了しました"),
        Err(e) => error!("APIサーバーの起動に失敗しました: {}", e),
    }
}

/// デモデータを初期化する関数
async fn initialize_demo_data(wallet_manager: &WalletManager, dex_manager: &DexManager) {
    info!("Initializing demo data...");
    
    // デモアカウントを作成
    let alice = wallet_manager.create_account("Alice".to_string()).unwrap();
    let bob = wallet_manager.create_account("Bob".to_string()).unwrap();
    
    info!("Created demo accounts: Alice ({}), Bob ({})", alice.id, bob.id);
    
    // 取引ペアを追加
    let btc_usd = dex_manager.add_trading_pair("BTC".to_string(), "USD".to_string());
    let eth_usd = dex_manager.add_trading_pair("ETH".to_string(), "USD".to_string());
    let btc_eth = dex_manager.add_trading_pair("BTC".to_string(), "ETH".to_string());
    
    info!("Added trading pairs: {}, {}, {}", btc_usd.to_string(), eth_usd.to_string(), btc_eth.to_string());
}
