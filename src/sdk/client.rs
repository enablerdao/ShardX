use crate::error::Error;
use reqwest::{Client, ClientBuilder, StatusCode};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::time::Duration;
use log::{debug, error, info, warn};
use thiserror::Error;

/// クライアントエラー
#[derive(Error, Debug)]
pub enum ClientError {
    /// リクエストエラー
    #[error("Request error: {0}")]
    RequestError(String),
    
    /// レスポンスエラー
    #[error("Response error: {0}")]
    ResponseError(String),
    
    /// 認証エラー
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    /// タイムアウトエラー
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    /// サーバーエラー
    #[error("Server error: {0}")]
    ServerError(String),
    
    /// 不正なパラメータ
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// クライアント設定
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// エンドポイントURL
    pub endpoint: String,
    /// タイムアウト（ミリ秒）
    pub timeout_ms: u64,
    /// 最大リトライ回数
    pub max_retries: u32,
    /// APIキー
    pub api_key: Option<String>,
}

/// ShardXクライアント
pub struct ShardXClient {
    /// HTTPクライアント
    http_client: Client,
    /// 設定
    config: ClientConfig,
}

/// APIリクエスト
#[derive(Serialize, Deserialize, Debug)]
struct ApiRequest<T> {
    /// JSONRPCバージョン
    jsonrpc: String,
    /// メソッド
    method: String,
    /// パラメータ
    params: T,
    /// リクエストID
    id: u64,
}

/// APIレスポンス
#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse<T> {
    /// JSONRPCバージョン
    jsonrpc: String,
    /// 結果
    result: Option<T>,
    /// エラー
    error: Option<ApiError>,
    /// リクエストID
    id: u64,
}

/// APIエラー
#[derive(Serialize, Deserialize, Debug)]
struct ApiError {
    /// エラーコード
    code: i32,
    /// エラーメッセージ
    message: String,
    /// エラーデータ
    data: Option<Value>,
}

impl ShardXClient {
    /// 新しいShardXClientを作成
    pub fn new(config: ClientConfig) -> Result<Self, Error> {
        // HTTPクライアントを構築
        let mut client_builder = ClientBuilder::new()
            .timeout(Duration::from_millis(config.timeout_ms))
            .connect_timeout(Duration::from_millis(config.timeout_ms / 2));
        
        // APIキーがある場合はデフォルトヘッダーに追加
        if let Some(api_key) = &config.api_key {
            client_builder = client_builder.default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "X-API-Key",
                    reqwest::header::HeaderValue::from_str(api_key)
                        .map_err(|e| Error::InvalidArgument(format!("Invalid API key: {}", e)))?,
                );
                headers
            });
        }
        
        let http_client = client_builder
            .build()
            .map_err(|e| Error::InitializationError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            http_client,
            config,
        })
    }
    
    /// 設定を取得
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
    
    /// トランザクションを送信
    pub async fn send_transaction(&self, transaction: &[u8]) -> Result<String, Error> {
        // トランザクションをBase64エンコード
        let tx_base64 = base64::encode(transaction);
        
        // リクエストを作成
        let request = ApiRequest {
            jsonrpc: "2.0".to_string(),
            method: "tx_sendRawTransaction".to_string(),
            params: json!([tx_base64]),
            id: 1,
        };
        
        // リクエストを送信
        let response: ApiResponse<String> = self.send_request(request).await?;
        
        // レスポンスからトランザクションIDを取得
        match response.result {
            Some(tx_id) => Ok(tx_id),
            None => Err(Error::ResponseError("No transaction ID in response".to_string())),
        }
    }
    
    /// アカウント残高を取得
    pub async fn get_balance(&self, address: &str) -> Result<u64, Error> {
        // リクエストを作成
        let request = ApiRequest {
            jsonrpc: "2.0".to_string(),
            method: "account_getBalance".to_string(),
            params: json!([address]),
            id: 1,
        };
        
        // リクエストを送信
        let response: ApiResponse<String> = self.send_request(request).await?;
        
        // レスポンスから残高を取得
        match response.result {
            Some(balance_str) => {
                balance_str.parse::<u64>()
                    .map_err(|e| Error::ResponseError(format!("Invalid balance format: {}", e)))
            },
            None => Err(Error::ResponseError("No balance in response".to_string())),
        }
    }
    
    /// コントラクト関数を呼び出し
    pub async fn call_contract(
        &self,
        contract_id: &str,
        function_name: &str,
        args: &[Vec<u8>],
    ) -> Result<Vec<u8>, Error> {
        // 引数をBase64エンコード
        let encoded_args: Vec<String> = args.iter()
            .map(|arg| base64::encode(arg))
            .collect();
        
        // リクエストを作成
        let request = ApiRequest {
            jsonrpc: "2.0".to_string(),
            method: "contract_call".to_string(),
            params: json!([contract_id, function_name, encoded_args]),
            id: 1,
        };
        
        // リクエストを送信
        let response: ApiResponse<String> = self.send_request(request).await?;
        
        // レスポンスから結果を取得
        match response.result {
            Some(result_base64) => {
                base64::decode(result_base64)
                    .map_err(|e| Error::ResponseError(format!("Invalid result format: {}", e)))
            },
            None => Err(Error::ResponseError("No result in response".to_string())),
        }
    }
    
    /// ブロック情報を取得
    pub async fn get_block(&self, block_id: &str) -> Result<Value, Error> {
        // リクエストを作成
        let request = ApiRequest {
            jsonrpc: "2.0".to_string(),
            method: "block_getBlockByHash".to_string(),
            params: json!([block_id]),
            id: 1,
        };
        
        // リクエストを送信
        let response: ApiResponse<Value> = self.send_request(request).await?;
        
        // レスポンスからブロック情報を取得
        match response.result {
            Some(block_info) => Ok(block_info),
            None => Err(Error::ResponseError("No block info in response".to_string())),
        }
    }
    
    /// トランザクション情報を取得
    pub async fn get_transaction(&self, tx_id: &str) -> Result<Value, Error> {
        // リクエストを作成
        let request = ApiRequest {
            jsonrpc: "2.0".to_string(),
            method: "tx_getTransactionByHash".to_string(),
            params: json!([tx_id]),
            id: 1,
        };
        
        // リクエストを送信
        let response: ApiResponse<Value> = self.send_request(request).await?;
        
        // レスポンスからトランザクション情報を取得
        match response.result {
            Some(tx_info) => Ok(tx_info),
            None => Err(Error::ResponseError("No transaction info in response".to_string())),
        }
    }
    
    /// ネットワーク情報を取得
    pub async fn get_network_info(&self) -> Result<Value, Error> {
        // リクエストを作成
        let request = ApiRequest {
            jsonrpc: "2.0".to_string(),
            method: "net_info".to_string(),
            params: json!([]),
            id: 1,
        };
        
        // リクエストを送信
        let response: ApiResponse<Value> = self.send_request(request).await?;
        
        // レスポンスからネットワーク情報を取得
        match response.result {
            Some(net_info) => Ok(net_info),
            None => Err(Error::ResponseError("No network info in response".to_string())),
        }
    }
    
    /// APIリクエストを送信
    async fn send_request<T, R>(&self, request: ApiRequest<T>) -> Result<ApiResponse<R>, Error>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let mut retries = 0;
        let max_retries = self.config.max_retries;
        
        loop {
            // リクエストを送信
            let result = self.http_client
                .post(&self.config.endpoint)
                .json(&request)
                .send()
                .await;
            
            match result {
                Ok(response) => {
                    // ステータスコードをチェック
                    match response.status() {
                        StatusCode::OK => {
                            // レスポンスをJSONとしてパース
                            match response.json::<ApiResponse<R>>().await {
                                Ok(api_response) => {
                                    // エラーをチェック
                                    if let Some(error) = api_response.error {
                                        return Err(Error::ApiError(format!(
                                            "API error: {} (code: {})",
                                            error.message,
                                            error.code
                                        )));
                                    }
                                    
                                    return Ok(api_response);
                                },
                                Err(e) => {
                                    return Err(Error::ResponseError(format!(
                                        "Failed to parse response: {}",
                                        e
                                    )));
                                }
                            }
                        },
                        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                            return Err(Error::AuthenticationError(
                                "Authentication failed".to_string()
                            ));
                        },
                        StatusCode::TOO_MANY_REQUESTS => {
                            if retries < max_retries {
                                // リトライ
                                retries += 1;
                                let delay = Duration::from_millis(500 * 2u64.pow(retries));
                                tokio::time::sleep(delay).await;
                                continue;
                            } else {
                                return Err(Error::RateLimitError(
                                    "Rate limit exceeded".to_string()
                                ));
                            }
                        },
                        status if status.is_server_error() => {
                            if retries < max_retries {
                                // リトライ
                                retries += 1;
                                let delay = Duration::from_millis(500 * 2u64.pow(retries));
                                tokio::time::sleep(delay).await;
                                continue;
                            } else {
                                return Err(Error::ServerError(format!(
                                    "Server error: {}",
                                    status
                                )));
                            }
                        },
                        status => {
                            return Err(Error::ResponseError(format!(
                                "Unexpected status code: {}",
                                status
                            )));
                        }
                    }
                },
                Err(e) => {
                    if e.is_timeout() {
                        if retries < max_retries {
                            // リトライ
                            retries += 1;
                            let delay = Duration::from_millis(500 * 2u64.pow(retries));
                            tokio::time::sleep(delay).await;
                            continue;
                        } else {
                            return Err(Error::TimeoutError(
                                "Request timed out".to_string()
                            ));
                        }
                    } else if e.is_connect() {
                        if retries < max_retries {
                            // リトライ
                            retries += 1;
                            let delay = Duration::from_millis(500 * 2u64.pow(retries));
                            tokio::time::sleep(delay).await;
                            continue;
                        } else {
                            return Err(Error::ConnectionError(
                                "Connection failed".to_string()
                            ));
                        }
                    } else {
                        return Err(Error::RequestError(format!(
                            "Request failed: {}",
                            e
                        )));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};
    use tokio::runtime::Runtime;
    
    #[test]
    fn test_client_creation() {
        let config = ClientConfig {
            endpoint: "http://localhost:8545".to_string(),
            timeout_ms: 5000,
            max_retries: 3,
            api_key: None,
        };
        
        let client = ShardXClient::new(config);
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_client_with_api_key() {
        let config = ClientConfig {
            endpoint: "http://localhost:8545".to_string(),
            timeout_ms: 5000,
            max_retries: 3,
            api_key: Some("test-api-key".to_string()),
        };
        
        let client = ShardXClient::new(config);
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_get_balance() {
        let rt = Runtime::new().unwrap();
        
        // モックサーバーを設定
        let mock_server = mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"jsonrpc":"2.0","result":"1000","id":1}"#)
            .create();
        
        let config = ClientConfig {
            endpoint: server_url(),
            timeout_ms: 5000,
            max_retries: 3,
            api_key: None,
        };
        
        let client = ShardXClient::new(config).unwrap();
        
        // バランスを取得
        let balance = rt.block_on(client.get_balance("0x1234567890abcdef"));
        
        mock_server.assert();
        assert!(balance.is_ok());
        assert_eq!(balance.unwrap(), 1000);
    }
    
    #[test]
    fn test_send_transaction() {
        let rt = Runtime::new().unwrap();
        
        // モックサーバーを設定
        let mock_server = mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"jsonrpc":"2.0","result":"0xabcdef1234567890","id":1}"#)
            .create();
        
        let config = ClientConfig {
            endpoint: server_url(),
            timeout_ms: 5000,
            max_retries: 3,
            api_key: None,
        };
        
        let client = ShardXClient::new(config).unwrap();
        
        // トランザクションを送信
        let tx_id = rt.block_on(client.send_transaction(b"test transaction"));
        
        mock_server.assert();
        assert!(tx_id.is_ok());
        assert_eq!(tx_id.unwrap(), "0xabcdef1234567890");
    }
}