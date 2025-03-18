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
use log::{error, info, warn};
use node::{Node, NodeConfig};
use tokio::sync::Mutex;
use tokio::time::sleep;
use wallet::WalletManager;
use web_server::WebServer;
use futures;

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
    let mut start_node = true;      // ノードを起動するかどうか
    let mut start_api = true;       // APIサーバーを起動するかどうか
    let mut start_web = true;       // Webサーバーを起動するかどうか

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
            "--node-only" => {
                start_node = true;
                start_api = true;
                start_web = false;
                i += 1;
            }
            "--web-only" => {
                start_node = false;
                start_api = false;
                start_web = true;
                i += 1;
            }
            "--no-web" => {
                start_web = false;
                i += 1;
            }
            "--no-node" => {
                start_node = false;
                start_api = false;
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
                println!("  --node-only        ノードとAPIサーバーのみを起動 (Webサーバーは起動しない)");
                println!("  --web-only         Webサーバーのみを起動 (ノードとAPIサーバーは起動しない)");
                println!("  --no-web           Webサーバーを起動しない");
                println!("  --no-node          ノードとAPIサーバーを起動しない");
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

    // 起動するコンポーネントの情報を表示
    info!("起動するコンポーネント:");
    if start_node {
        info!("- ノード (ブロックチェーンノード)");
        info!("- APIサーバー (ポート: {})", api_port);
    }
    if start_web {
        info!("- Webサーバー (ポート: {})", web_port);
    }

    // ノードとAPIサーバーの変数を初期化
    let node;
    let wallet_manager;
    let dex_manager;
    let api_server_opt;

    // ノードとAPIサーバーを起動する場合
    if start_node {
        // ノード設定を作成
        let mut config = NodeConfig::default();
        config.node_id = node_id.clone();
        config.port = api_port;
        config.data_dir = data_dir.clone();
        config.shard_count = shard_count;

        info!("DAGを初期化中...");

        // ノードを作成
        let mut node_instance = Node::new(config);

        info!(
            "シャーディングマネージャを初期化中 (シャード数: {})...",
            shard_count
        );

        // ノードを起動
        info!("コンセンサスエンジンを初期化中...");
        node_instance.start().await;

        // ノードをArcでラップ
        node = Arc::new(Mutex::new(node_instance));

        // ウォレットマネージャーを作成
        wallet_manager = Arc::new(WalletManager::new());

        // DEXマネージャーを作成
        dex_manager = Arc::new(DexManager::new(Arc::clone(&wallet_manager)));

        // 初期データを設定
        initialize_demo_data(&wallet_manager, &dex_manager).await;

        info!("APIサーバーを初期化中 (ポート: {})...", api_port);

        // APIサーバーを作成
        api_server_opt = Some(ApiServer::new(
            Arc::clone(&node),
            Arc::clone(&wallet_manager),
            Arc::clone(&dex_manager),
            api_port,
        ));
    } else {
        // ノードとAPIサーバーを起動しない場合はダミーの値を設定
        node = Arc::new(Mutex::new(Node::new(NodeConfig::default())));
        wallet_manager = Arc::new(WalletManager::new());
        dex_manager = Arc::new(DexManager::new(Arc::clone(&wallet_manager)));
        api_server_opt = None;
    }

    // Webサーバーの変数を初期化
    let web_server_opt;

    // Webサーバーを起動する場合
    if start_web {
        info!("Webサーバーを初期化中 (ポート: {})...", web_port);
        info!("Webディレクトリパス: {}", web_dir);
        
        // Webディレクトリの存在を確認
        let web_dir_path = Path::new(&web_dir);
        if !web_dir_path.exists() {
            error!("Webディレクトリが存在しません: {}", web_dir);
            println!("エラー: Webディレクトリが存在しません: {}", web_dir);
            println!("現在の作業ディレクトリ: {}", std::env::current_dir().unwrap().display());
            return;
        }
        
        if !web_dir_path.is_dir() {
            error!("Webディレクトリがディレクトリではありません: {}", web_dir);
            println!("エラー: Webディレクトリがディレクトリではありません: {}", web_dir);
            return;
        }
        
        // index.htmlの存在を確認
        let index_path = web_dir_path.join("index.html");
        if !index_path.exists() {
            error!("index.htmlが存在しません: {}", index_path.display());
            println!("エラー: index.htmlが存在しません: {}", index_path.display());
            return;
        }
        
        info!("Webディレクトリの検証が完了しました: {}", web_dir);
        web_server_opt = Some(WebServer::new(web_dir.clone(), web_port));
        info!("Webサーバーが初期化されました");
    } else {
        web_server_opt = None;
    }
    
    // アクセス可能なURLを表示
    println!("\n=== ShardX サービスが起動しました ===");
    if start_web {
        println!("Web UI: http://localhost:{}/", web_port);
    }
    if start_node {
        println!("API エンドポイント: http://localhost:{}/", api_port);
        println!("ノード情報: http://localhost:{}/info", api_port);
    }
    println!("=====================================\n");

    // CLIモードの場合はCLIを起動
    if cli_mode && start_node {
        // CLIを作成
        let cli = CLI::new(Arc::clone(&node), Arc::clone(&wallet_manager));
        
        // CLIを起動
        cli.start().await;
    } else {
        // サーバーを並行して起動
        let mut handles = Vec::new();
        
        // APIサーバーを起動
        if let Some(api_server) = api_server_opt {
            info!("APIサーバーを起動します");
            let api_handle = tokio::spawn(async move {
                info!("APIサーバーの起動を試みます...");
                match api_server.start().await {
                    Ok(_) => info!("APIサーバーが正常に終了しました"),
                    Err(e) => error!("APIサーバーの起動に失敗しました: {}", e),
                }
            });
            handles.push(api_handle);
        }
        
        // Webサーバーを起動
        if let Some(web_server) = web_server_opt {
            info!("Webサーバーを起動します");
            let web_handle = tokio::spawn(async move {
                info!("Webサーバーの起動を試みます...");
                match web_server.start().await {
                    Ok(_) => info!("Webサーバーが正常に終了しました"),
                    Err(e) => {
                        error!("Webサーバーの起動に失敗しました: {}", e);
                        println!("エラー: Webサーバーの起動に失敗しました: {}", e);
                    }
                }
            });
            handles.push(web_handle);
        }
        
        // 少なくとも1つのサーバーが起動している場合
        if !handles.is_empty() {
            // メインスレッドを維持するためのダミータスク
            let dummy_handle = tokio::spawn(async {
                loop {
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                }
            });
            handles.push(dummy_handle);
            
            // 最初のタスクが完了するまで待機
            if let Some(handle) = futures::future::select_all(handles).await.0 {
                if let Err(e) = handle {
                    error!("サーバータスクの実行中にエラーが発生しました: {}", e);
                }
            }
        } else {
            error!("起動するサーバーがありません。--node-only、--web-only、または両方を指定してください。");
            println!("エラー: 起動するサーバーがありません。--node-only、--web-only、または両方を指定してください。");
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
