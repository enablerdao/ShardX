use crate::api_handlers::*;
use crate::dex::{DexManager, Order, OrderType, TradingPair, Trade};
use crate::node::Node;
use crate::transaction::Transaction;
use crate::wallet::{Account, WalletManager};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Filter, Rejection, Reply};
use base64;

/// APIサーバー
pub struct ApiServer {
    /// ノードの参照
    node: Arc<Mutex<Node>>,
    /// ウォレットマネージャーの参照
    wallet_manager: Arc<WalletManager>,
    /// DEXマネージャーの参照
    dex_manager: Arc<DexManager>,
    /// サーバーのポート
    port: u16,
}

/// トランザクション作成リクエスト
#[derive(Debug, Deserialize)]
struct CreateTransactionRequest {
    /// 親トランザクションのID
    parent_ids: Vec<String>,
    /// ペイロード（Base64エンコード）
    payload: String,
    /// 署名（Base64エンコード）
    signature: String,
}

/// ノード情報レスポンス
#[derive(Debug, Serialize)]
struct NodeInfoResponse {
    /// ノードID
    id: String,
    /// ノードの状態
    status: String,
    /// 現在のTPS
    tps: f64,
    /// 現在のシャード数
    shard_count: u32,
    /// 確認済みトランザクション数
    confirmed_transactions: usize,
}

/// トランザクション作成レスポンス
#[derive(Debug, Serialize)]
struct CreateTransactionResponse {
    /// トランザクションID
    id: String,
    /// 処理結果
    status: String,
}

impl ApiServer {
    /// 新しいAPIサーバーを作成
    pub fn new(
        node: Arc<Mutex<Node>>, 
        wallet_manager: Arc<WalletManager>,
        dex_manager: Arc<DexManager>,
        port: u16
    ) -> Self {
        Self { 
            node, 
            wallet_manager, 
            dex_manager, 
            port 
        }
    }
    
    /// サーバーを起動
    pub async fn start(&self) -> Result<(), String> {
        info!("Starting API server on port {}", self.port);
        
        // 各マネージャーの参照をクローン
        let node_clone = Arc::clone(&self.node);
        let wallet_manager_clone = Arc::clone(&self.wallet_manager);
        let dex_manager_clone = Arc::clone(&self.dex_manager);
        
        // ブロックチェーンAPI
        // ノード情報を取得するエンドポイント
        let node_info = warp::path("info")
            .and(warp::get())
            .and(with_node(Arc::clone(&node_clone)))
            .and_then(handle_node_info);
        
        // トランザクションを作成するエンドポイント
        let create_tx = warp::path("transactions")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_node(Arc::clone(&node_clone)))
            .and_then(handle_create_transaction);
        
        // ウォレットAPI
        // アカウント作成エンドポイント
        let create_account = warp::path("accounts")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_wallet_manager(Arc::clone(&wallet_manager_clone)))
            .and_then(handle_create_account);
        
        // アカウント情報取得エンドポイント
        let get_account = warp::path!("accounts" / String)
            .and(warp::get())
            .and(with_wallet_manager(Arc::clone(&wallet_manager_clone)))
            .and_then(handle_get_account);
        
        // 送金エンドポイント
        let transfer = warp::path("transfer")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_wallet_manager(Arc::clone(&wallet_manager_clone)))
            .and(with_node(Arc::clone(&node_clone)))
            .and_then(handle_transfer);
        
        // DEX API
        // 取引ペア追加エンドポイント
        let add_trading_pair = warp::path("trading-pairs")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_dex_manager(Arc::clone(&dex_manager_clone)))
            .and_then(handle_add_trading_pair);
        
        // 注文作成エンドポイント
        let create_order = warp::path("orders")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_dex_manager(Arc::clone(&dex_manager_clone)))
            .and_then(handle_create_order);
        
        // 注文キャンセルエンドポイント
        let cancel_order = warp::path!("orders" / String)
            .and(warp::delete())
            .and(warp::query::<CancelOrderQuery>())
            .and(with_dex_manager(Arc::clone(&dex_manager_clone)))
            .and_then(handle_cancel_order);
        
        // オーダーブック取得エンドポイント
        let get_order_book = warp::path("order-book")
            .and(warp::get())
            .and(warp::query::<OrderBookQuery>())
            .and(with_dex_manager(Arc::clone(&dex_manager_clone)))
            .and_then(handle_get_order_book);
        
        // 取引履歴取得エンドポイント
        let get_trade_history = warp::path("trade-history")
            .and(warp::get())
            .and(warp::query::<TradeHistoryQuery>())
            .and(with_dex_manager(Arc::clone(&dex_manager_clone)))
            .and_then(handle_get_trade_history);
        
        // CORSを設定
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
            .allow_headers(vec!["Content-Type"]);
        
        // ルートを結合
        let routes = node_info
            .or(create_tx)
            .or(create_account)
            .or(get_account)
            .or(transfer)
            .or(add_trading_pair)
            .or(create_order)
            .or(cancel_order)
            .or(get_order_book)
            .or(get_trade_history)
            .with(cors)
            .with(warp::log("api"));
        
        // サーバーを起動
        warp::serve(routes)
            .run(([0, 0, 0, 0], self.port))
            .await;
            
        Ok(())
    }
}

/// ノードの参照をフィルターに追加
fn with_node(
    node: Arc<Mutex<Node>>,
) -> impl Filter<Extract = (Arc<Mutex<Node>>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&node))
}

/// ウォレットマネージャーの参照をフィルターに追加
fn with_wallet_manager(
    wallet_manager: Arc<WalletManager>,
) -> impl Filter<Extract = (Arc<WalletManager>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&wallet_manager))
}

/// DEXマネージャーの参照をフィルターに追加
fn with_dex_manager(
    dex_manager: Arc<DexManager>,
) -> impl Filter<Extract = (Arc<DexManager>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&dex_manager))
}

/// ノード情報を取得するハンドラー
async fn handle_node_info(node: Arc<Mutex<Node>>) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    
    let response = NodeInfoResponse {
        id: node.id.clone(),
        status: format!("{:?}", node.get_status()),
        tps: node.get_tps(),
        shard_count: node.get_shard_count(),
        confirmed_transactions: node.dag.confirmed_count(),
    };
    
    Ok(warp::reply::json(&response))
}

/// トランザクションを作成するハンドラー
async fn handle_create_transaction(
    req: CreateTransactionRequest,
    node: Arc<Mutex<Node>>,
) -> Result<impl Reply, Rejection> {
    // Base64デコード
    let payload = match base64::decode(&req.payload) {
        Ok(p) => p,
        Err(_) => {
            return Ok(warp::reply::json(&CreateTransactionResponse {
                id: "".to_string(),
                status: "Invalid payload encoding".to_string(),
            }))
        }
    };
    
    let signature = match base64::decode(&req.signature) {
        Ok(s) => s,
        Err(_) => {
            return Ok(warp::reply::json(&CreateTransactionResponse {
                id: "".to_string(),
                status: "Invalid signature encoding".to_string(),
            }))
        }
    };
    
    // トランザクションを作成
    let tx = Transaction::new(req.parent_ids, payload, signature);
    let tx_id = tx.id.clone();
    
    // トランザクションを送信
    let result = {
        let node = node.lock().await;
        node.submit_transaction(tx).await
    };
    
    match result {
        Ok(_) => {
            info!("Transaction {} created successfully", tx_id);
            Ok(warp::reply::json(&CreateTransactionResponse {
                id: tx_id,
                status: "success".to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to create transaction: {}", e);
            Ok(warp::reply::json(&CreateTransactionResponse {
                id: tx_id,
                status: format!("error: {}", e),
            }))
        }
    }
}