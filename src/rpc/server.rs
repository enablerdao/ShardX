use jsonrpc_core::{Error as JsonRpcError, IoHandler, Params, Value};
use jsonrpc_http_server::{AccessControlAllowOrigin, DomainsValidation, Server, ServerBuilder};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ai::advanced_prediction::{Prediction, TradingRecommendation};
use crate::error::Error;
use crate::node::Node;
use crate::transaction::{Transaction, TransactionStatus};
use crate::visualization::{ChartData, ChartMetric, ChartPeriod};
use crate::wallet::multisig::{MultisigTransaction, MultisigTransactionStatus, MultisigWallet};

/// RPC サーバー
pub struct RpcServer {
    /// ノード
    node: Arc<Node>,
    /// サーバー
    server: Option<Server>,
}

/// ブロック情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    /// ブロックハッシュ
    pub hash: String,
    /// 前のブロックハッシュ
    pub previous_hash: String,
    /// ブロック高
    pub height: u64,
    /// タイムスタンプ
    pub timestamp: u64,
    /// トランザクション数
    pub transaction_count: usize,
    /// シャードID
    pub shard_id: String,
}

/// ネットワーク情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// バージョン
    pub version: String,
    /// ピア数
    pub peers: usize,
    /// 現在のブロック高
    pub height: u64,
    /// 1秒あたりのトランザクション数
    pub tps: f64,
    /// 合計トランザクション数
    pub total_transactions: u64,
    /// 平均ブロック時間（秒）
    pub avg_block_time: f64,
    /// 合計アカウント数
    pub total_accounts: u64,
    /// 現在の手数料
    pub current_fee: String,
    /// シャード数
    pub shard_count: usize,
}

impl RpcServer {
    /// 新しいRPCサーバーを作成
    pub fn new(node: Arc<Node>) -> Self {
        Self { node, server: None }
    }

    /// サーバーを起動
    pub fn start(&mut self, addr: SocketAddr) -> Result<(), Error> {
        info!("Starting RPC server on {}", addr);

        let node = self.node.clone();

        // JSONRPCハンドラを作成
        let mut io = IoHandler::default();

        // getInfo メソッド
        let node_clone = node.clone();
        io.add_method("getInfo", move |_params| {
            let node = node_clone.clone();

            async move {
                match get_network_info(node).await {
                    Ok(info) => Ok(json!(info)),
                    Err(e) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // getBlock メソッド
        let node_clone = node.clone();
        io.add_method("getBlock", move |params: Params| {
            let node = node_clone.clone();

            async move {
                let hash_or_height = match params.parse::<Value>() {
                    Ok(Value::String(hash)) => hash,
                    Ok(Value::Number(num)) => num.to_string(),
                    _ => {
                        return Err(JsonRpcError::invalid_params(
                            "Expected block hash or height",
                        ))
                    }
                };

                match get_block(node, &hash_or_height).await {
                    Ok(block) => Ok(json!(block)),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // getTransaction メソッド
        let node_clone = node.clone();
        io.add_method("getTransaction", move |params: Params| {
            let node = node_clone.clone();

            async move {
                let tx_id = match params.parse::<Value>() {
                    Ok(Value::String(id)) => id,
                    _ => return Err(JsonRpcError::invalid_params("Expected transaction ID")),
                };

                match get_transaction(node, &tx_id).await {
                    Ok(tx) => Ok(json!(tx)),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // sendTransaction メソッド
        let node_clone = node.clone();
        io.add_method("sendTransaction", move |params: Params| {
            let node = node_clone.clone();

            async move {
                let tx: Transaction = match params.parse() {
                    Ok(tx) => tx,
                    Err(_) => {
                        return Err(JsonRpcError::invalid_params("Invalid transaction format"))
                    }
                };

                match send_transaction(node, tx).await {
                    Ok(tx_id) => Ok(json!({ "transaction_id": tx_id })),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // getBalance メソッド
        let node_clone = node.clone();
        io.add_method("getBalance", move |params: Params| {
            let node = node_clone.clone();

            async move {
                let address = match params.parse::<Value>() {
                    Ok(Value::String(addr)) => addr,
                    _ => return Err(JsonRpcError::invalid_params("Expected address")),
                };

                match get_balance(node, &address).await {
                    Ok(balance) => Ok(json!({ "balance": balance })),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // getTransactions メソッド
        let node_clone = node.clone();
        io.add_method("getTransactions", move |params: Params| {
            let node = node_clone.clone();

            async move {
                let address = match params.parse::<Value>() {
                    Ok(Value::String(addr)) => addr,
                    _ => return Err(JsonRpcError::invalid_params("Expected address")),
                };

                match get_transactions(node, &address).await {
                    Ok(txs) => Ok(json!(txs)),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // getMultisigWallet メソッド
        let node_clone = node.clone();
        io.add_method("getMultisigWallet", move |params: Params| {
            let node = node_clone.clone();

            async move {
                let wallet_id = match params.parse::<Value>() {
                    Ok(Value::String(id)) => id,
                    _ => return Err(JsonRpcError::invalid_params("Expected wallet ID")),
                };

                match get_multisig_wallet(node, &wallet_id).await {
                    Ok(wallet) => Ok(json!(wallet)),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // createMultisigWallet メソッド
        let node_clone = node.clone();
        io.add_method("createMultisigWallet", move |params: Params| {
            let node = node_clone.clone();

            async move {
                #[derive(Deserialize)]
                struct CreateWalletParams {
                    name: String,
                    owner_id: String,
                    signers: Vec<String>,
                    required_signatures: usize,
                }

                let wallet_params: CreateWalletParams = match params.parse() {
                    Ok(p) => p,
                    Err(_) => {
                        return Err(JsonRpcError::invalid_params("Invalid wallet parameters"))
                    }
                };

                match create_multisig_wallet(
                    node,
                    &wallet_params.name,
                    &wallet_params.owner_id,
                    wallet_params.signers,
                    wallet_params.required_signatures,
                )
                .await
                {
                    Ok(wallet) => Ok(json!(wallet)),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // createMultisigTransaction メソッド
        let node_clone = node.clone();
        io.add_method("createMultisigTransaction", move |params: Params| {
            let node = node_clone.clone();

            async move {
                #[derive(Deserialize)]
                struct CreateTxParams {
                    wallet_id: String,
                    creator: String,
                    to: String,
                    amount: String,
                    data: Option<String>,
                    signature: String,
                }

                let tx_params: CreateTxParams = match params.parse() {
                    Ok(p) => p,
                    Err(_) => {
                        return Err(JsonRpcError::invalid_params(
                            "Invalid transaction parameters",
                        ))
                    }
                };

                match create_multisig_transaction(
                    node,
                    &tx_params.wallet_id,
                    &tx_params.creator,
                    &tx_params.to,
                    &tx_params.amount,
                    tx_params.data,
                    &tx_params.signature,
                )
                .await
                {
                    Ok(tx) => Ok(json!(tx)),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // signMultisigTransaction メソッド
        let node_clone = node.clone();
        io.add_method("signMultisigTransaction", move |params: Params| {
            let node = node_clone.clone();

            async move {
                #[derive(Deserialize)]
                struct SignTxParams {
                    transaction_id: String,
                    signer: String,
                    signature: String,
                }

                let sign_params: SignTxParams = match params.parse() {
                    Ok(p) => p,
                    Err(_) => {
                        return Err(JsonRpcError::invalid_params("Invalid signature parameters"))
                    }
                };

                match sign_multisig_transaction(
                    node,
                    &sign_params.transaction_id,
                    &sign_params.signer,
                    &sign_params.signature,
                )
                .await
                {
                    Ok(tx) => Ok(json!(tx)),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // getChartData メソッド
        let node_clone = node.clone();
        io.add_method("getChartData", move |params: Params| {
            let node = node_clone.clone();

            async move {
                #[derive(Deserialize)]
                struct ChartParams {
                    metric: String,
                    period: String,
                    start_time: Option<u64>,
                    end_time: Option<u64>,
                }

                let chart_params: ChartParams = match params.parse() {
                    Ok(p) => p,
                    Err(_) => return Err(JsonRpcError::invalid_params("Invalid chart parameters")),
                };

                match get_chart_data(
                    node,
                    &chart_params.metric,
                    &chart_params.period,
                    chart_params.start_time,
                    chart_params.end_time,
                )
                .await
                {
                    Ok(data) => Ok(json!(data)),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // getPricePrediction メソッド
        let node_clone = node.clone();
        io.add_method("getPricePrediction", move |params: Params| {
            let node = node_clone.clone();

            async move {
                #[derive(Deserialize)]
                struct PredictionParams {
                    pair: String,
                    period: String,
                }

                let pred_params: PredictionParams = match params.parse() {
                    Ok(p) => p,
                    Err(_) => {
                        return Err(JsonRpcError::invalid_params(
                            "Invalid prediction parameters",
                        ))
                    }
                };

                match get_price_prediction(node, &pred_params.pair, &pred_params.period).await {
                    Ok(prediction) => Ok(json!(prediction)),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // getTradingRecommendation メソッド
        let node_clone = node.clone();
        io.add_method("getTradingRecommendation", move |params: Params| {
            let node = node_clone.clone();

            async move {
                #[derive(Deserialize)]
                struct RecommendationParams {
                    pair: String,
                }

                let rec_params: RecommendationParams = match params.parse() {
                    Ok(p) => p,
                    Err(_) => {
                        return Err(JsonRpcError::invalid_params(
                            "Invalid recommendation parameters",
                        ))
                    }
                };

                match get_trading_recommendation(node, &rec_params.pair).await {
                    Ok(recommendation) => Ok(json!(recommendation)),
                    Err(_) => Err(JsonRpcError::internal_error()),
                }
            }
        });

        // サーバーを構築
        let server = ServerBuilder::new(io)
            .cors(DomainsValidation::AllowAll)
            .threads(4)
            .start_http(&addr)
            .map_err(|e| Error::Other(format!("Failed to start RPC server: {}", e)))?;

        self.server = Some(server);

        Ok(())
    }

    /// サーバーを停止
    pub fn stop(&mut self) {
        if let Some(server) = self.server.take() {
            info!("Stopping RPC server");
            server.close();
        }
    }
}

/// ネットワーク情報を取得
async fn get_network_info(node: Arc<Node>) -> Result<NetworkInfo, Error> {
    // 実際の実装では、ノードからネットワーク情報を取得
    // ここでは簡易的な実装として、ダミーデータを返す

    Ok(NetworkInfo {
        version: "1.0.0".to_string(),
        peers: 10,
        height: 12345,
        tps: 100.0,
        total_transactions: 1000000,
        avg_block_time: 2.5,
        total_accounts: 50000,
        current_fee: "0.001".to_string(),
        shard_count: 5,
    })
}

/// ブロック情報を取得
async fn get_block(node: Arc<Node>, hash_or_height: &str) -> Result<BlockInfo, Error> {
    // 実際の実装では、ノードからブロック情報を取得
    // ここでは簡易的な実装として、ダミーデータを返す

    Ok(BlockInfo {
        hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        previous_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
        height: 12345,
        timestamp: chrono::Utc::now().timestamp() as u64,
        transaction_count: 100,
        shard_id: "shard1".to_string(),
    })
}

/// トランザクション情報を取得
async fn get_transaction(node: Arc<Node>, tx_id: &str) -> Result<Transaction, Error> {
    // 実際の実装では、ノードからトランザクション情報を取得
    // ここでは簡易的な実装として、ダミーデータを返す

    Ok(Transaction {
        id: tx_id.to_string(),
        from: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        to: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
        amount: "100".to_string(),
        fee: "0.001".to_string(),
        data: None,
        nonce: 1,
        timestamp: chrono::Utc::now().timestamp() as u64,
        signature: "0xsignature".to_string(),
        status: TransactionStatus::Confirmed,
        shard_id: "shard1".to_string(),
        block_hash: Some(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        ),
        block_height: Some(12345),
        parent_id: None,
    })
}

/// トランザクションを送信
async fn send_transaction(node: Arc<Node>, tx: Transaction) -> Result<String, Error> {
    // 実際の実装では、ノードにトランザクションを送信
    // ここでは簡易的な実装として、トランザクションIDを返す

    Ok(tx.id)
}

/// 残高を取得
async fn get_balance(node: Arc<Node>, address: &str) -> Result<String, Error> {
    // 実際の実装では、ノードから残高を取得
    // ここでは簡易的な実装として、ダミーデータを返す

    Ok("1000.0".to_string())
}

/// トランザクション履歴を取得
async fn get_transactions(node: Arc<Node>, address: &str) -> Result<Vec<Transaction>, Error> {
    // 実際の実装では、ノードからトランザクション履歴を取得
    // ここでは簡易的な実装として、ダミーデータを返す

    let mut transactions = Vec::new();

    for i in 0..10 {
        let tx = Transaction {
            id: format!("tx{}", i),
            from: if i % 2 == 0 {
                address.to_string()
            } else {
                "0xabcdef1234567890abcdef1234567890abcdef12".to_string()
            },
            to: if i % 2 == 0 {
                "0xabcdef1234567890abcdef1234567890abcdef12".to_string()
            } else {
                address.to_string()
            },
            amount: format!("{}", 10 * (i + 1)),
            fee: "0.001".to_string(),
            data: None,
            nonce: i as u64,
            timestamp: chrono::Utc::now().timestamp() as u64 - i * 3600,
            signature: "0xsignature".to_string(),
            status: TransactionStatus::Confirmed,
            shard_id: "shard1".to_string(),
            block_hash: Some(
                "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            ),
            block_height: Some(12345 - i),
            parent_id: None,
        };

        transactions.push(tx);
    }

    Ok(transactions)
}

/// マルチシグウォレットを取得
async fn get_multisig_wallet(node: Arc<Node>, wallet_id: &str) -> Result<MultisigWallet, Error> {
    // 実際の実装では、ノードからマルチシグウォレット情報を取得
    // ここでは簡易的な実装として、ダミーデータを返す

    Ok(MultisigWallet {
        id: wallet_id.to_string(),
        name: "Test Wallet".to_string(),
        owner_id: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        signers: vec![
            "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
            "0x9876543210abcdef1234567890abcdef12345678".to_string(),
        ],
        required_signatures: 2,
        balance: "1000.0".to_string(),
        created_at: chrono::Utc::now().timestamp() as u64 - 86400,
        updated_at: chrono::Utc::now().timestamp() as u64,
        nonce: 0,
    })
}

/// マルチシグウォレットを作成
async fn create_multisig_wallet(
    node: Arc<Node>,
    name: &str,
    owner_id: &str,
    signers: Vec<String>,
    required_signatures: usize,
) -> Result<MultisigWallet, Error> {
    // 実際の実装では、ノードでマルチシグウォレットを作成
    // ここでは簡易的な実装として、ダミーデータを返す

    let wallet_id = format!("mw{}", chrono::Utc::now().timestamp());

    Ok(MultisigWallet {
        id: wallet_id,
        name: name.to_string(),
        owner_id: owner_id.to_string(),
        signers,
        required_signatures,
        balance: "0.0".to_string(),
        created_at: chrono::Utc::now().timestamp() as u64,
        updated_at: chrono::Utc::now().timestamp() as u64,
        nonce: 0,
    })
}

/// マルチシグトランザクションを作成
async fn create_multisig_transaction(
    node: Arc<Node>,
    wallet_id: &str,
    creator: &str,
    to: &str,
    amount: &str,
    data: Option<String>,
    signature: &str,
) -> Result<MultisigTransaction, Error> {
    // 実際の実装では、ノードでマルチシグトランザクションを作成
    // ここでは簡易的な実装として、ダミーデータを返す

    let tx_id = format!("mt{}", chrono::Utc::now().timestamp());
    let now = chrono::Utc::now().timestamp() as u64;

    let mut signatures = std::collections::HashMap::new();
    signatures.insert(creator.to_string(), signature.to_string());

    Ok(MultisigTransaction {
        id: tx_id,
        wallet_id: wallet_id.to_string(),
        to: to.to_string(),
        amount: amount.to_string(),
        data,
        signatures,
        required_signatures: 2,
        status: MultisigTransactionStatus::Pending,
        creator: creator.to_string(),
        created_at: now,
        executed_at: None,
        expires_at: now + 7 * 24 * 60 * 60, // 1週間後
        nonce: 0,
    })
}

/// マルチシグトランザクションに署名
async fn sign_multisig_transaction(
    node: Arc<Node>,
    transaction_id: &str,
    signer: &str,
    signature: &str,
) -> Result<MultisigTransaction, Error> {
    // 実際の実装では、ノードでマルチシグトランザクションに署名
    // ここでは簡易的な実装として、ダミーデータを返す

    let now = chrono::Utc::now().timestamp() as u64;

    let mut signatures = std::collections::HashMap::new();
    signatures.insert(
        "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        "0xsig1".to_string(),
    );
    signatures.insert(signer.to_string(), signature.to_string());

    Ok(MultisigTransaction {
        id: transaction_id.to_string(),
        wallet_id: "mw12345".to_string(),
        to: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
        amount: "100.0".to_string(),
        data: None,
        signatures,
        required_signatures: 2,
        status: MultisigTransactionStatus::Executed,
        creator: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        created_at: now - 3600,
        executed_at: Some(now),
        expires_at: now + 7 * 24 * 60 * 60, // 1週間後
        nonce: 0,
    })
}

/// チャートデータを取得
async fn get_chart_data(
    node: Arc<Node>,
    metric: &str,
    period: &str,
    start_time: Option<u64>,
    end_time: Option<u64>,
) -> Result<ChartData, Error> {
    // 実際の実装では、ノードからチャートデータを取得
    // ここでは簡易的な実装として、ダミーデータを返す

    // メトリックを解析
    let chart_metric = match metric {
        "transaction_count" => ChartMetric::TransactionCount,
        "transaction_volume" => ChartMetric::TransactionVolume,
        "fees" => ChartMetric::Fees,
        "active_addresses" => ChartMetric::ActiveAddresses,
        "average_transaction_size" => ChartMetric::AverageTransactionSize,
        "price" => ChartMetric::Price,
        "cross_shard_transactions" => ChartMetric::CrossShardTransactions,
        _ => {
            return Err(Error::ValidationError(format!(
                "Invalid chart metric: {}",
                metric
            )))
        }
    };

    // 期間を解析
    let chart_period = match period {
        "hour" => ChartPeriod::Hour,
        "day" => ChartPeriod::Day,
        "week" => ChartPeriod::Week,
        "month" => ChartPeriod::Month,
        "year" => ChartPeriod::Year,
        "all" => ChartPeriod::All,
        _ => {
            return Err(Error::ValidationError(format!(
                "Invalid chart period: {}",
                period
            )))
        }
    };

    // 開始時刻と終了時刻を設定
    let end = end_time
        .map(|t| chrono::DateTime::<chrono::Utc>::from_timestamp(t as i64, 0).unwrap())
        .unwrap_or_else(chrono::Utc::now);

    let start = start_time
        .map(|t| chrono::DateTime::<chrono::Utc>::from_timestamp(t as i64, 0).unwrap())
        .unwrap_or_else(|| chart_period.get_start_time(end));

    // ダミーデータポイントを生成
    let mut data_points = Vec::new();
    let interval = chart_period.get_interval();
    let mut current_time = start;

    while current_time <= end {
        data_points.push(crate::visualization::DataPoint {
            timestamp: current_time,
            value: rand::random::<f64>() * 100.0,
        });

        current_time = current_time + interval;
    }

    Ok(ChartData {
        metric: chart_metric,
        period: chart_period,
        start_time: start,
        end_time: end,
        data_points,
    })
}

/// 価格予測を取得
async fn get_price_prediction(
    node: Arc<Node>,
    pair: &str,
    period: &str,
) -> Result<Prediction, Error> {
    // 実際の実装では、ノードから価格予測を取得
    // ここでは簡易的な実装として、ダミーデータを返す

    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::hours(24);

    // ダミーの履歴データを生成
    let mut historical_data = Vec::new();
    for i in 0..24 {
        let timestamp = now - chrono::Duration::hours(i);
        historical_data.push(crate::ai::advanced_prediction::PricePoint {
            timestamp,
            price: 2.5 + rand::random::<f64>() * 0.5,
            volume: Some(10000.0 + rand::random::<f64>() * 5000.0),
        });
    }

    // 履歴データを時間順にソート
    historical_data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(Prediction {
        pair: pair.to_string(),
        period: period.to_string(),
        current_price: 2.75,
        predicted_price: 2.85,
        confidence: 0.85,
        timestamp: now,
        expires_at,
        historical_data,
    })
}

/// 取引推奨を取得
async fn get_trading_recommendation(
    node: Arc<Node>,
    pair: &str,
) -> Result<TradingRecommendation, Error> {
    // 実際の実装では、ノードから取引推奨を取得
    // ここでは簡易的な実装として、ダミーデータを返す

    Ok(TradingRecommendation {
        action: crate::ai::advanced_prediction::TradingAction::Buy,
        confidence: 0.85,
        reasoning: "Strong upward trend with increasing volume".to_string(),
        predicted_change_percent: 3.5,
        timestamp: chrono::Utc::now(),
    })
}
