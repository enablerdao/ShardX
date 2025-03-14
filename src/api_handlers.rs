use crate::dex::{DexManager, Order, OrderType, TradingPair, Trade};
use crate::node::Node;
use crate::transaction::Transaction;
use crate::wallet::{Account, WalletManager};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Rejection, Reply};
use warp::reply::Response;

// ウォレットAPI用の構造体

/// アカウント作成リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    /// アカウント名
    pub name: String,
}

/// アカウント作成レスポンス
#[derive(Debug, Serialize)]
pub struct CreateAccountResponse {
    /// アカウントID
    pub id: String,
    /// 公開鍵
    pub public_key: String,
    /// アカウント名
    pub name: String,
    /// 残高
    pub balance: f64,
    /// 作成日時
    pub created_at: String,
}

/// 送金リクエスト
#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    /// 送信元アカウントID
    pub from_account_id: String,
    /// 送信先アカウントID
    pub to_account_id: String,
    /// 金額
    pub amount: f64,
    /// トークンID（オプション）
    pub token_id: Option<String>,
}

/// 送金レスポンス
#[derive(Debug, Serialize)]
pub struct TransferResponse {
    /// トランザクションID
    pub transaction_id: String,
    /// ステータス
    pub status: String,
}

// DEX API用の構造体

/// 取引ペア追加リクエスト
#[derive(Debug, Deserialize)]
pub struct AddTradingPairRequest {
    /// 基準通貨
    pub base: String,
    /// 相手通貨
    pub quote: String,
}

/// 取引ペア追加レスポンス
#[derive(Debug, Serialize)]
pub struct AddTradingPairResponse {
    /// 取引ペア
    pub pair: String,
    /// ステータス
    pub status: String,
}

/// 注文作成リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    /// アカウントID
    pub account_id: String,
    /// 基準通貨
    pub base: String,
    /// 相手通貨
    pub quote: String,
    /// 注文タイプ
    pub order_type: String,
    /// 価格
    pub price: f64,
    /// 数量
    pub amount: f64,
}

/// 注文作成レスポンス
#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
    /// 注文ID
    pub order_id: String,
    /// 取引ペア
    pub pair: String,
    /// 注文タイプ
    pub order_type: String,
    /// 価格
    pub price: f64,
    /// 数量
    pub amount: f64,
    /// ステータス
    pub status: String,
    /// 約定した取引
    pub trades: Vec<TradeInfo>,
}

/// 取引情報
#[derive(Debug, Serialize)]
pub struct TradeInfo {
    /// 取引ID
    pub id: String,
    /// 価格
    pub price: f64,
    /// 数量
    pub amount: f64,
    /// 取引日時
    pub executed_at: String,
}

/// 注文キャンセルクエリ
#[derive(Debug, Deserialize)]
pub struct CancelOrderQuery {
    /// アカウントID
    pub account_id: String,
}

/// 注文キャンセルレスポンス
#[derive(Debug, Serialize)]
pub struct CancelOrderResponse {
    /// 注文ID
    pub order_id: String,
    /// ステータス
    pub status: String,
}

/// オーダーブッククエリ
#[derive(Debug, Deserialize)]
pub struct OrderBookQuery {
    /// 基準通貨
    pub base: String,
    /// 相手通貨
    pub quote: String,
}

/// オーダーブックレスポンス
#[derive(Debug, Serialize)]
pub struct OrderBookResponse {
    /// 取引ペア
    pub pair: String,
    /// 買い注文
    pub bids: Vec<OrderInfo>,
    /// 売り注文
    pub asks: Vec<OrderInfo>,
}

/// 注文情報
#[derive(Debug, Serialize)]
pub struct OrderInfo {
    /// 価格
    pub price: f64,
    /// 数量
    pub amount: f64,
    /// 合計
    pub total: f64,
}

/// 取引履歴クエリ
#[derive(Debug, Deserialize)]
pub struct TradeHistoryQuery {
    /// 基準通貨
    pub base: String,
    /// 相手通貨
    pub quote: String,
    /// 最大件数
    pub limit: Option<usize>,
}

/// 取引履歴レスポンス
#[derive(Debug, Serialize)]
pub struct TradeHistoryResponse {
    /// 取引ペア
    pub pair: String,
    /// 取引履歴
    pub trades: Vec<TradeInfo>,
}

// ウォレットAPIハンドラー

/// アカウント作成ハンドラー
pub async fn handle_create_account(
    req: CreateAccountRequest,
    wallet_manager: Arc<WalletManager>,
) -> Result<Response, Rejection> {
    match wallet_manager.create_account(req.name) {
        Ok(account) => {
            let response = CreateAccountResponse {
                id: account.id,
                public_key: account.public_key,
                name: account.name,
                balance: account.balance,
                created_at: account.created_at.to_rfc3339(),
            };
            Ok(warp::reply::json(&response).into_response())
        }
        Err(e) => {
            error!("Failed to create account: {}", e);
            let json_response = serde_json::json!({
                "error": format!("Failed to create account: {}", e)
            });
            Ok(warp::reply::with_status(
                warp::reply::json(&json_response),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response())
        }
    }
}

/// アカウント情報取得ハンドラー
pub async fn handle_get_account(
    account_id: String,
    wallet_manager: Arc<WalletManager>,
) -> Result<Response, Rejection> {
    match wallet_manager.get_account(&account_id) {
        Some(account) => {
            let response = serde_json::json!({
                "id": account.id,
                "public_key": account.public_key,
                "name": account.name,
                "balance": account.balance,
                "token_balances": account.token_balances,
                "created_at": account.created_at.to_rfc3339(),
            });
            Ok(warp::reply::json(&response).into_response())
        }
        None => {
            let json_response = serde_json::json!({
                "error": format!("Account {} not found", account_id)
            });
            Ok(warp::reply::with_status(
                warp::reply::json(&json_response),
                warp::http::StatusCode::NOT_FOUND,
            ).into_response())
        }
    }
}

/// 送金ハンドラー
pub async fn handle_transfer(
    req: TransferRequest,
    wallet_manager: Arc<WalletManager>,
    node: Arc<Mutex<Node>>,
) -> Result<Response, Rejection> {
    // トランザクションを作成
    match wallet_manager.create_transaction(
        &req.from_account_id,
        &req.to_account_id,
        req.amount,
        req.token_id,
        vec![],
    ) {
        Ok(tx) => {
            // ノードにトランザクションを送信
            let mut node = node.lock().await;
            match node.submit_transaction(tx.clone()).await {
                Ok(_) => {
                    let response = TransferResponse {
                        transaction_id: tx.id,
                        status: "success".to_string(),
                    };
                    Ok(warp::reply::json(&response).into_response())
                }
                Err(e) => {
                    error!("Failed to add transaction: {}", e);
                    let json_response = serde_json::json!({
                        "error": format!("Failed to add transaction: {}", e)
                    });
                    Ok(warp::reply::with_status(
                        warp::reply::json(&json_response),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ).into_response())
                }
            }
        }
        Err(e) => {
            error!("Failed to create transaction: {}", e);
            let json_response = serde_json::json!({
                "error": format!("Failed to create transaction: {}", e)
            });
            Ok(warp::reply::with_status(
                warp::reply::json(&json_response),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response())
        }
    }
}

// DEX APIハンドラー

/// 取引ペア追加ハンドラー
pub async fn handle_add_trading_pair(
    req: AddTradingPairRequest,
    dex_manager: Arc<DexManager>,
) -> Result<Response, Rejection> {
    let pair = dex_manager.add_trading_pair(req.base, req.quote);
    let response = AddTradingPairResponse {
        pair: pair.to_string(),
        status: "success".to_string(),
    };
    Ok(warp::reply::json(&response).into_response())
}

/// 注文作成ハンドラー
pub async fn handle_create_order(
    req: CreateOrderRequest,
    dex_manager: Arc<DexManager>,
) -> Result<Response, Rejection> {
    // 注文タイプを変換
    let order_type = match req.order_type.to_lowercase().as_str() {
        "buy" => OrderType::Buy,
        "sell" => OrderType::Sell,
        _ => {
            let json_response = serde_json::json!({
                "error": format!("Invalid order type: {}", req.order_type)
            });
            return Ok(warp::reply::with_status(
                warp::reply::json(&json_response),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response());
        }
    };
    
    // 取引ペアを作成
    let pair = TradingPair::new(req.base, req.quote);
    
    // 注文を作成
    match dex_manager.create_order(
        &req.account_id,
        pair.clone(),
        order_type,
        req.price,
        req.amount,
    ) {
        Ok((order, trades)) => {
            // 取引情報を変換
            let trade_infos = trades.iter().map(|trade| TradeInfo {
                id: trade.id.clone(),
                price: trade.price,
                amount: trade.amount,
                executed_at: trade.executed_at.to_rfc3339(),
            }).collect();
            
            let response = CreateOrderResponse {
                order_id: order.id,
                pair: pair.to_string(),
                order_type: format!("{:?}", order.order_type),
                price: order.price,
                amount: order.amount,
                status: format!("{:?}", order.status),
                trades: trade_infos,
            };
            Ok(warp::reply::json(&response).into_response())
        }
        Err(e) => {
            error!("Failed to create order: {}", e);
            let json_response = serde_json::json!({
                "error": format!("Failed to create order: {}", e)
            });
            Ok(warp::reply::with_status(
                warp::reply::json(&json_response),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response())
        }
    }
}

/// 注文キャンセルハンドラー
pub async fn handle_cancel_order(
    order_id: String,
    query: CancelOrderQuery,
    dex_manager: Arc<DexManager>,
) -> Result<Response, Rejection> {
    match dex_manager.cancel_order(&query.account_id, &order_id) {
        Ok(_) => {
            let response = CancelOrderResponse {
                order_id,
                status: "canceled".to_string(),
            };
            Ok(warp::reply::json(&response).into_response())
        }
        Err(e) => {
            error!("Failed to cancel order: {}", e);
            let json_response = serde_json::json!({
                "error": format!("Failed to cancel order: {}", e)
            });
            Ok(warp::reply::with_status(
                warp::reply::json(&json_response),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response())
        }
    }
}

/// オーダーブック取得ハンドラー
pub async fn handle_get_order_book(
    query: OrderBookQuery,
    dex_manager: Arc<DexManager>,
) -> Result<Response, Rejection> {
    let pair = TradingPair::new(query.base, query.quote);
    
    match dex_manager.get_order_book(&pair) {
        Ok((bids, asks)) => {
            // 買い注文情報を変換
            let bid_infos = bids.iter().map(|order| OrderInfo {
                price: order.price,
                amount: order.amount - order.filled_amount,
                total: order.price * (order.amount - order.filled_amount),
            }).collect();
            
            // 売り注文情報を変換
            let ask_infos = asks.iter().map(|order| OrderInfo {
                price: order.price,
                amount: order.amount - order.filled_amount,
                total: order.price * (order.amount - order.filled_amount),
            }).collect();
            
            let response = OrderBookResponse {
                pair: pair.to_string(),
                bids: bid_infos,
                asks: ask_infos,
            };
            Ok(warp::reply::json(&response).into_response())
        }
        Err(e) => {
            error!("Failed to get order book: {}", e);
            let json_response = serde_json::json!({
                "error": format!("Failed to get order book: {}", e)
            });
            Ok(warp::reply::with_status(
                warp::reply::json(&json_response),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response())
        }
    }
}

/// 取引履歴取得ハンドラー
pub async fn handle_get_trade_history(
    query: TradeHistoryQuery,
    dex_manager: Arc<DexManager>,
) -> Result<Response, Rejection> {
    let pair = TradingPair::new(query.base, query.quote);
    let limit = query.limit.unwrap_or(100);
    
    match dex_manager.get_trade_history(&pair) {
        Ok(trades) => {
            // 取引情報を変換
            let trade_infos = trades.iter()
                .take(limit)
                .map(|trade| TradeInfo {
                    id: trade.id.clone(),
                    price: trade.price,
                    amount: trade.amount,
                    executed_at: trade.executed_at.to_rfc3339(),
                })
                .collect();
            
            let response = TradeHistoryResponse {
                pair: pair.to_string(),
                trades: trade_infos,
            };
            Ok(warp::reply::json(&response).into_response())
        }
        Err(e) => {
            error!("Failed to get trade history: {}", e);
            let json_response = serde_json::json!({
                "error": format!("Failed to get trade history: {}", e)
            });
            Ok(warp::reply::with_status(
                warp::reply::json(&json_response),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response())
        }
    }
}