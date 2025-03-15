use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};

use crate::error::Error;
use crate::cross_chain::{
    bridge::{CrossChainBridge, BridgeConfig, ChainType, BridgeStatus},
    transaction::{CrossChainTransaction, TransactionStatus},
    token_registry::{TokenRegistry, TokenInfo},
    enhanced_ethereum_bridge::EnhancedEthereumBridge,
};

/// クロスチェーンAPIハンドラ
pub struct CrossChainApiHandler {
    /// トークンレジストリ
    token_registry: Arc<TokenRegistry>,
    /// イーサリアムブリッジ
    ethereum_bridge: Option<Arc<EnhancedEthereumBridge>>,
}

/// トークン転送リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransferRequest {
    /// トークンID
    pub token_id: String,
    /// 送信元アドレス（ShardXの場合）
    pub from_address: Option<String>,
    /// 送信先アドレス
    pub to_address: String,
    /// 金額
    pub amount: String,
    /// 送信元チェーン
    pub source_chain: ChainType,
    /// 送信先チェーン
    pub target_chain: ChainType,
}

/// トークン転送レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransferResponse {
    /// トランザクションID
    pub transaction_id: String,
    /// ステータス
    pub status: String,
    /// メッセージ
    pub message: String,
}

/// トランザクション状態リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusRequest {
    /// トランザクションID
    pub transaction_id: String,
}

/// トランザクション状態レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusResponse {
    /// トランザクションID
    pub transaction_id: String,
    /// ステータス
    pub status: String,
    /// 送信元チェーン
    pub source_chain: ChainType,
    /// 送信先チェーン
    pub target_chain: ChainType,
    /// 送信元アドレス
    pub from_address: String,
    /// 送信先アドレス
    pub to_address: String,
    /// 金額
    pub amount: String,
    /// トークンシンボル
    pub token_symbol: Option<String>,
    /// 確認数
    pub confirmations: Option<u64>,
    /// 送信元ブロックハッシュ
    pub source_block_hash: Option<String>,
    /// 送信元ブロック高
    pub source_block_height: Option<u64>,
    /// 送信先ブロックハッシュ
    pub target_block_hash: Option<String>,
    /// 送信先ブロック高
    pub target_block_height: Option<u64>,
    /// タイムスタンプ
    pub timestamp: u64,
    /// エラーメッセージ
    pub error_message: Option<String>,
}

/// サポートされているトークンレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedTokensResponse {
    /// トークンリスト
    pub tokens: Vec<TokenInfo>,
}

/// ブリッジ状態レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatusResponse {
    /// ブリッジ状態
    pub status: BridgeStatus,
    /// サポートされているチェーン
    pub supported_chains: Vec<ChainType>,
}

impl CrossChainApiHandler {
    /// 新しいCrossChainApiHandlerを作成
    pub fn new(
        token_registry: Arc<TokenRegistry>,
        ethereum_bridge: Option<Arc<EnhancedEthereumBridge>>,
    ) -> Self {
        Self {
            token_registry,
            ethereum_bridge,
        }
    }
    
    /// APIルートを作成
    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let api_context = warp::any().map(move || self.clone());
        
        let transfer_route = warp::path!("api" / "v1" / "cross-chain" / "transfer")
            .and(warp::post())
            .and(warp::body::json())
            .and(api_context.clone())
            .and_then(Self::handle_transfer);
        
        let status_route = warp::path!("api" / "v1" / "cross-chain" / "status" / String)
            .and(warp::get())
            .and(api_context.clone())
            .and_then(Self::handle_status);
        
        let tokens_route = warp::path!("api" / "v1" / "cross-chain" / "tokens")
            .and(warp::get())
            .and(api_context.clone())
            .and_then(Self::handle_tokens);
        
        let bridge_status_route = warp::path!("api" / "v1" / "cross-chain" / "bridge-status")
            .and(warp::get())
            .and(api_context.clone())
            .and_then(Self::handle_bridge_status);
        
        transfer_route
            .or(status_route)
            .or(tokens_route)
            .or(bridge_status_route)
    }
    
    /// トークン転送リクエストを処理
    async fn handle_transfer(
        request: TokenTransferRequest,
        handler: Self,
    ) -> Result<impl Reply, Rejection> {
        info!("Handling cross-chain transfer request: {:?}", request);
        
        // リクエストを検証
        if request.token_id.is_empty() {
            return Ok(warp::reply::json(&TokenTransferResponse {
                transaction_id: "".to_string(),
                status: "error".to_string(),
                message: "Token ID is required".to_string(),
            }));
        }
        
        if request.to_address.is_empty() {
            return Ok(warp::reply::json(&TokenTransferResponse {
                transaction_id: "".to_string(),
                status: "error".to_string(),
                message: "Recipient address is required".to_string(),
            }));
        }
        
        if request.amount.is_empty() {
            return Ok(warp::reply::json(&TokenTransferResponse {
                transaction_id: "".to_string(),
                status: "error".to_string(),
                message: "Amount is required".to_string(),
            }));
        }
        
        // トークン情報を取得
        let token = match handler.token_registry.get_token(&request.token_id) {
            Some(token) => token,
            None => return Ok(warp::reply::json(&TokenTransferResponse {
                transaction_id: "".to_string(),
                status: "error".to_string(),
                message: format!("Token not found: {}", request.token_id),
            })),
        };
        
        // チェーンタイプを検証
        if token.chain_type != request.source_chain && token.chain_type != request.target_chain {
            return Ok(warp::reply::json(&TokenTransferResponse {
                transaction_id: "".to_string(),
                status: "error".to_string(),
                message: format!("Token {} is not on chain {} or {}", 
                    token.symbol, request.source_chain, request.target_chain),
            }));
        }
        
        // トランザクションを作成
        let transaction_id = match (request.source_chain, request.target_chain) {
            (ChainType::ShardX, ChainType::Ethereum) => {
                // ShardXからイーサリアムへの転送
                let ethereum_bridge = match &handler.ethereum_bridge {
                    Some(bridge) => bridge,
                    None => return Ok(warp::reply::json(&TokenTransferResponse {
                        transaction_id: "".to_string(),
                        status: "error".to_string(),
                        message: "Ethereum bridge not initialized".to_string(),
                    })),
                };
                
                let from_address = match request.from_address {
                    Some(addr) => addr,
                    None => return Ok(warp::reply::json(&TokenTransferResponse {
                        transaction_id: "".to_string(),
                        status: "error".to_string(),
                        message: "From address is required for ShardX to Ethereum transfers".to_string(),
                    })),
                };
                
                match ethereum_bridge.transfer_to_ethereum(
                    &request.token_id,
                    &request.to_address,
                    &request.amount,
                    &from_address,
                ).await {
                    Ok(tx_id) => tx_id,
                    Err(e) => return Ok(warp::reply::json(&TokenTransferResponse {
                        transaction_id: "".to_string(),
                        status: "error".to_string(),
                        message: format!("Failed to create transaction: {}", e),
                    })),
                }
            },
            (ChainType::Ethereum, ChainType::ShardX) => {
                // イーサリアムからShardXへの転送
                let ethereum_bridge = match &handler.ethereum_bridge {
                    Some(bridge) => bridge,
                    None => return Ok(warp::reply::json(&TokenTransferResponse {
                        transaction_id: "".to_string(),
                        status: "error".to_string(),
                        message: "Ethereum bridge not initialized".to_string(),
                    })),
                };
                
                match ethereum_bridge.transfer_from_ethereum(
                    &request.token_id,
                    &request.to_address,
                    &request.amount,
                ).await {
                    Ok(tx_id) => tx_id,
                    Err(e) => return Ok(warp::reply::json(&TokenTransferResponse {
                        transaction_id: "".to_string(),
                        status: "error".to_string(),
                        message: format!("Failed to create transaction: {}", e),
                    })),
                }
            },
            _ => return Ok(warp::reply::json(&TokenTransferResponse {
                transaction_id: "".to_string(),
                status: "error".to_string(),
                message: format!("Unsupported chain combination: {} -> {}", 
                    request.source_chain, request.target_chain),
            })),
        };
        
        // 成功レスポンスを返す
        Ok(warp::reply::json(&TokenTransferResponse {
            transaction_id,
            status: "success".to_string(),
            message: format!("Transaction created successfully. Token: {}, Amount: {}", 
                token.symbol, request.amount),
        }))
    }
    
    /// トランザクション状態リクエストを処理
    async fn handle_status(
        transaction_id: String,
        handler: Self,
    ) -> Result<impl Reply, Rejection> {
        info!("Handling cross-chain transaction status request: {}", transaction_id);
        
        // トランザクションの状態を取得
        let transaction_details = match handler.ethereum_bridge {
            Some(ref bridge) => match bridge.get_transaction_details(&transaction_id) {
                Ok(tx) => Some(tx),
                Err(_) => None,
            },
            None => None,
        };
        
        // トランザクションが見つからない場合
        if transaction_details.is_none() {
            return Ok(warp::reply::json(&TransactionStatusResponse {
                transaction_id: transaction_id.clone(),
                status: "not_found".to_string(),
                source_chain: ChainType::Unknown,
                target_chain: ChainType::Unknown,
                from_address: "".to_string(),
                to_address: "".to_string(),
                amount: "0".to_string(),
                token_symbol: None,
                confirmations: None,
                source_block_hash: None,
                source_block_height: None,
                target_block_hash: None,
                target_block_height: None,
                timestamp: 0,
                error_message: Some("Transaction not found".to_string()),
            }));
        }
        
        // トランザクション詳細を取得
        let tx = transaction_details.unwrap();
        
        // トークンシンボルを取得
        let token_symbol = match tx.get_metadata("token_symbol") {
            Some(symbol) => Some(symbol.to_string()),
            None => {
                // トークンIDからシンボルを取得
                match tx.get_metadata("token_id") {
                    Some(token_id) => {
                        match handler.token_registry.get_token(token_id) {
                            Some(token) => Some(token.symbol),
                            None => None,
                        }
                    },
                    None => None,
                }
            },
        };
        
        // 確認数を取得
        let confirmations = match tx.get_metadata("confirmations") {
            Some(conf) => match conf.parse::<u64>() {
                Ok(n) => Some(n),
                Err(_) => None,
            },
            None => None,
        };
        
        // レスポンスを作成
        let response = TransactionStatusResponse {
            transaction_id: tx.id.clone(),
            status: tx.status.to_string(),
            source_chain: tx.source_chain,
            target_chain: tx.target_chain,
            from_address: tx.transaction.from.clone(),
            to_address: tx.transaction.to.clone(),
            amount: tx.transaction.amount.clone(),
            token_symbol,
            confirmations,
            source_block_hash: tx.source_block_hash.clone(),
            source_block_height: tx.source_block_height,
            target_block_hash: tx.target_block_hash.clone(),
            target_block_height: tx.target_block_height,
            timestamp: tx.transaction.timestamp,
            error_message: tx.error_message.clone(),
        };
        
        Ok(warp::reply::json(&response))
    }
    
    /// サポートされているトークンリクエストを処理
    async fn handle_tokens(
        handler: Self,
    ) -> Result<impl Reply, Rejection> {
        info!("Handling supported tokens request");
        
        // すべてのトークンを取得
        let tokens = handler.token_registry.get_all_tokens();
        
        // レスポンスを作成
        let response = SupportedTokensResponse {
            tokens,
        };
        
        Ok(warp::reply::json(&response))
    }
    
    /// ブリッジ状態リクエストを処理
    async fn handle_bridge_status(
        handler: Self,
    ) -> Result<impl Reply, Rejection> {
        info!("Handling bridge status request");
        
        // ブリッジ状態を取得
        let status = match &handler.ethereum_bridge {
            Some(bridge) => bridge.get_status(),
            None => BridgeStatus::Offline,
        };
        
        // サポートされているチェーンを取得
        let mut supported_chains = vec![ChainType::ShardX];
        
        if handler.ethereum_bridge.is_some() {
            supported_chains.push(ChainType::Ethereum);
        }
        
        // レスポンスを作成
        let response = BridgeStatusResponse {
            status,
            supported_chains,
        };
        
        Ok(warp::reply::json(&response))
    }
}