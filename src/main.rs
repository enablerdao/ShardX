mod ai;
mod api;
mod api_handlers;
mod cli;
mod consensus;
mod dex;
mod node;
mod sharding;
mod transaction;
mod wallet;
mod web_server;

use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use api::ApiServer;
use cli::CLI;
use dex::DexManager;
use log::{error, info};
use node::{Node, NodeConfig};
use tokio::sync::Mutex;
use wallet::WalletManager;
use web_server::WebServer;

#[tokio::main]
async fn main() {
    // コマンドライン引数の解析
    let args: Vec<String> = env::args().collect();

    let mut node_id = format!(
        "node_{}",
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    );
    let mut api_port = 54868;
    let mut web_port = 54867;
    let mut data_dir = "./data".to_string();
    let mut web_dir = "./web".to_string();
    let mut log_level = "info".to_string();
    let mut shard_count = 256;
    let mut cli_mode = false;

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
            "--api-port" | "--port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse() {
                        api_port = p;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--web-port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse() {
                        web_port = p;
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
            "--web-dir" => {
                if i + 1 < args.len() {
                    web_dir = args[i + 1].clone();
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
            "--cli" => {
                cli_mode = true;
                i += 1;
            }
            "--help" => {
                println!("使用方法: shardx [オプション]");
                println!("");
                println!("オプション:");
                println!("  --node-id ID       ノードID (デフォルト: ランダム生成)");
                println!("  --api-port PORT    APIポート (デフォルト: 54868)");
                println!("  --web-port PORT    Webポート (デフォルト: 54867)");
                println!("  --data-dir DIR     データディレクトリ (デフォルト: ./data)");
                println!("  --web-dir DIR      Webディレクトリ (デフォルト: ./web)");
                println!(
                    "  --log-level LEVEL  ログレベル (debug, info, warn, error) (デフォルト: info)"
                );
                println!("  --shard-count N    シャード数 (デフォルト: 256)");
                println!("  --cli              コマンドラインインターフェイスモードで起動");
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
    info!("APIポート: {}", api_port);
    info!("Webポート: {}", web_port);
    info!("データディレクトリ: {}", data_dir);
    info!("Webディレクトリ: {}", web_dir);
    info!("ログレベル: {}", log_level);

    // ノード設定を作成
    let mut config = NodeConfig::default();
    config.node_id = node_id;
    config.port = api_port;
    config.data_dir = data_dir;
    config.shard_count = shard_count;

    info!("DAGを初期化中...");

    // ノードを作成
    let mut node = Node::new(config);

    info!(
        "シャーディングマネージャを初期化中 (シャード数: {})...",
        shard_count
    );

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

    info!("APIサーバーを起動中 (ポート: {})...", api_port);

    // APIサーバーを作成
    let api_server = ApiServer::new(
        Arc::clone(&node),
        Arc::clone(&wallet_manager),
        Arc::clone(&dex_manager),
        api_port,
    );

    // ウェブサーバーを作成
    info!("Webサーバーを初期化中 (ポート: {})...", web_port);
    info!("Webディレクトリパス: {}", web_dir);
    
    // Webディレクトリの存在を確認
    if !Path::new(&web_dir).exists() {
        error!("Webディレクトリが存在しません: {}", web_dir);
        println!("エラー: Webディレクトリが存在しません: {}", web_dir);
        println!("現在の作業ディレクトリ: {}", std::env::current_dir().unwrap().display());
        return;
    }
    
    let web_server = WebServer::new(web_dir.clone(), web_port);
    info!("Webサーバーが初期化されました");
    
    // アクセス可能なURLを表示
    println!("\n=== ShardX サービスが起動しました ===");
    println!("Web UI: http://localhost:{}/", web_port);
    println!("API エンドポイント: http://localhost:{}/", api_port);
    println!("ノード情報: http://localhost:{}/info", api_port);
    println!("=====================================\n");

    // CLIモードの場合はCLIを起動
    if cli_mode {
        // CLIを作成
        let cli = CLI::new(Arc::clone(&node), Arc::clone(&wallet_manager));
        
        // CLIを起動
        cli.start().await;
    } else {
        // 両方のサーバーを並行して起動
        info!("APIサーバーとWebサーバーを並行して起動します");
        
        // Webサーバーを別スレッドで起動
        let web_handle = tokio::spawn(async move {
            info!("Webサーバーの起動を試みます...");
            match web_server.start().await {
                Ok(_) => info!("Webサーバーが正常に終了しました"),
                Err(e) => error!("Webサーバーの起動に失敗しました: {}", e),
            }
        });
        
        // APIサーバーを起動
        info!("APIサーバーの起動を試みます...");
        match api_server.start().await {
            Ok(_) => info!("APIサーバーが正常に終了しました"),
            Err(e) => error!("APIサーバーの起動に失敗しました: {}", e),
        }
        
        // Webサーバーの終了を待機
        if let Err(e) = web_handle.await {
            error!("Webサーバータスクの実行中にエラーが発生しました: {}", e);
        }
    }
}

/// デモデータを初期化する関数
async fn initialize_demo_data(wallet_manager: &WalletManager, dex_manager: &DexManager) {
    info!("Initializing demo data...");

    // デモアカウントを作成
    let alice = wallet_manager.create_account("Alice".to_string()).unwrap();
    let bob = wallet_manager.create_account("Bob".to_string()).unwrap();

    info!(
        "Created demo accounts: Alice ({}), Bob ({})",
        alice.id, bob.id
    );

    // 取引ペアを追加
    let btc_usd = dex_manager.add_trading_pair("BTC".to_string(), "USD".to_string());
    let eth_usd = dex_manager.add_trading_pair("ETH".to_string(), "USD".to_string());
    let btc_eth = dex_manager.add_trading_pair("BTC".to_string(), "ETH".to_string());

    info!(
        "Added trading pairs: {}, {}, {}",
        btc_usd.to_string(),
        eth_usd.to_string(),
        btc_eth.to_string()
    );
}
