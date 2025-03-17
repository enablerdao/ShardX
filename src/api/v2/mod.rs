// APIv2モジュール
//
// このモジュールは、ShardXのAPIv2を提供します。
// 主な機能:
// - RESTful API
// - GraphQL API
// - WebSocket API
// - OpenAPI仕様
// - サードパーティ統合

// mod rest; // TODO: このモジュールが見つかりません
// mod graphql; // TODO: このモジュールが見つかりません
// mod websocket; // TODO: このモジュールが見つかりません
// mod openapi; // TODO: このモジュールが見つかりません
mod third_party;

pub use self::rest::{RestApi, RestEndpoint, RestHandler};
pub use self::graphql::{GraphQLApi, GraphQLSchema, GraphQLResolver};
pub use self::websocket::{WebSocketApi, WebSocketHandler, WebSocketConnection};
pub use self::openapi::{OpenApiGenerator, ApiSpec};
pub use self::third_party::{ThirdPartyIntegration, IntegrationConfig, IntegrationRegistry};

use crate::error::Error;
use crate::metrics::MetricsCollector;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use warp::Filter;
use axum::{Router, routing};
use serde::{Serialize, Deserialize};

/// API設定
#[derive(Clone, Debug)]
pub struct ApiConfig {
    /// ホスト
    pub host: String,
    /// ポート
    pub port: u16,
    /// ベースパス
    pub base_path: String,
    /// CORSを有効にするかどうか
    pub enable_cors: bool,
    /// 認証を有効にするかどうか
    pub enable_auth: bool,
    /// レート制限を有効にするかどうか
    pub enable_rate_limiting: bool,
    /// ドキュメントを有効にするかどうか
    pub enable_docs: bool,
    /// メトリクスを有効にするかどうか
    pub enable_metrics: bool,
    /// タイムアウト（ミリ秒）
    pub timeout_ms: u64,
    /// 最大リクエストサイズ（バイト）
    pub max_request_size: usize,
}

/// APIマネージャー
pub struct ApiManager {
    /// API設定
    config: ApiConfig,
    /// RESTful API
    rest_api: RestApi,
    /// GraphQL API
    graphql_api: GraphQLApi,
    /// WebSocket API
    websocket_api: WebSocketApi,
    /// OpenAPI生成器
    openapi_generator: OpenApiGenerator,
    /// サードパーティ統合レジストリ
    integration_registry: Arc<Mutex<IntegrationRegistry>>,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// 実行中フラグ
    running: Arc<Mutex<bool>>,
    /// シャットダウンチャネル
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl ApiManager {
    /// 新しいApiManagerを作成
    pub fn new(config: ApiConfig, metrics: Arc<MetricsCollector>) -> Self {
        Self {
            config: config.clone(),
            rest_api: RestApi::new(config.clone()),
            graphql_api: GraphQLApi::new(config.clone()),
            websocket_api: WebSocketApi::new(config.clone()),
            openapi_generator: OpenApiGenerator::new(config.clone()),
            integration_registry: Arc::new(Mutex::new(IntegrationRegistry::new())),
            metrics,
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: None,
        }
    }
    
    /// APIサーバーを起動
    pub async fn start(&mut self) -> Result<(), Error> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(Error::InvalidState("API server is already running".to_string()));
        }
        
        *running = true;
        drop(running);
        
        info!("Starting API server on {}:{}", self.config.host, self.config.port);
        
        // シャットダウンチャネルを作成
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        // RESTful APIを設定
        let rest_routes = self.rest_api.routes();
        
        // GraphQL APIを設定
        let graphql_routes = self.graphql_api.routes();
        
        // WebSocket APIを設定
        let websocket_routes = self.websocket_api.routes();
        
        // OpenAPIドキュメントを設定
        let openapi_routes = if self.config.enable_docs {
            self.openapi_generator.routes()
        } else {
            warp::any().map(|| "Documentation disabled").boxed()
        };
        
        // メトリクスを設定
        let metrics_routes = if self.config.enable_metrics {
            warp::path("metrics")
                .and(warp::get())
                .map(move || {
                    let metrics_data = self.metrics.collect_metrics();
                    warp::reply::json(&metrics_data)
                })
                .boxed()
        } else {
            warp::any().map(|| "Metrics disabled").boxed()
        };
        
        // CORSを設定
        let cors = if self.config.enable_cors {
            warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allow_headers(vec!["Content-Type", "Authorization"])
                .build()
        } else {
            warp::cors().build()
        };
        
        // すべてのルートを結合
        let routes = rest_routes
            .or(graphql_routes)
            .or(websocket_routes)
            .or(openapi_routes)
            .or(metrics_routes)
            .with(cors)
            .with(warp::log("api"));
        
        // サーバーを起動
        let (addr, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(
                ([0, 0, 0, 0], self.config.port),
                async move {
                    shutdown_rx.recv().await;
                    info!("API server shutting down");
                },
            );
        
        // サーバーを実行
        tokio::spawn(server);
        
        info!("API server started on {}", addr);
        
        // サードパーティ統合を初期化
        self.initialize_third_party_integrations().await?;
        
        Ok(())
    }
    
    /// APIサーバーを停止
    pub async fn stop(&mut self) -> Result<(), Error> {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Err(Error::InvalidState("API server is not running".to_string()));
        }
        
        // シャットダウン信号を送信
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        
        *running = false;
        
        info!("API server stopped");
        
        Ok(())
    }
    
    /// RESTエンドポイントを登録
    pub fn register_rest_endpoint<H>(&mut self, path: &str, handler: H) -> Result<(), Error>
    where
        H: RestHandler + Send + Sync + 'static,
    {
        self.rest_api.register_endpoint(path, handler)
    }
    
    /// GraphQLリゾルバーを登録
    pub fn register_graphql_resolver<R>(&mut self, resolver: R) -> Result<(), Error>
    where
        R: GraphQLResolver + Send + Sync + 'static,
    {
        self.graphql_api.register_resolver(resolver)
    }
    
    /// WebSocketハンドラーを登録
    pub fn register_websocket_handler<H>(&mut self, path: &str, handler: H) -> Result<(), Error>
    where
        H: WebSocketHandler + Send + Sync + 'static,
    {
        self.websocket_api.register_handler(path, handler)
    }
    
    /// サードパーティ統合を登録
    pub fn register_third_party_integration<I>(&mut self, integration: I) -> Result<(), Error>
    where
        I: ThirdPartyIntegration + Send + Sync + 'static,
    {
        let mut registry = self.integration_registry.lock().unwrap();
        registry.register_integration(Box::new(integration))
    }
    
    /// サードパーティ統合を初期化
    async fn initialize_third_party_integrations(&self) -> Result<(), Error> {
        let registry = self.integration_registry.lock().unwrap();
        
        for integration in registry.get_integrations() {
            match integration.initialize().await {
                Ok(_) => {
                    info!("Initialized third-party integration: {}", integration.name());
                },
                Err(e) => {
                    error!("Failed to initialize third-party integration {}: {}", integration.name(), e);
                }
            }
        }
        
        Ok(())
    }
    
    /// OpenAPI仕様を生成
    pub fn generate_openapi_spec(&self) -> Result<ApiSpec, Error> {
        self.openapi_generator.generate_spec()
    }
    
    /// API設定を取得
    pub fn get_config(&self) -> &ApiConfig {
        &self.config
    }
    
    /// 実行中かどうかを確認
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
    
    /// サードパーティ統合を取得
    pub fn get_third_party_integration(&self, name: &str) -> Option<Box<dyn ThirdPartyIntegration>> {
        let registry = self.integration_registry.lock().unwrap();
        registry.get_integration(name)
    }
    
    /// サードパーティ統合のリストを取得
    pub fn get_third_party_integrations(&self) -> Vec<String> {
        let registry = self.integration_registry.lock().unwrap();
        registry.get_integration_names()
    }
}

impl Drop for ApiManager {
    fn drop(&mut self) {
        if *self.running.lock().unwrap() {
            // 非同期コンテキスト外でのシャットダウン
            if let Some(tx) = self.shutdown_tx.take() {
                // チャネルを閉じる
                drop(tx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_config() {
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            base_path: "/api/v2".to_string(),
            enable_cors: true,
            enable_auth: true,
            enable_rate_limiting: true,
            enable_docs: true,
            enable_metrics: true,
            timeout_ms: 30000,
            max_request_size: 10 * 1024 * 1024, // 10MB
        };
        
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.base_path, "/api/v2");
        assert!(config.enable_cors);
        assert!(config.enable_auth);
        assert!(config.enable_rate_limiting);
        assert!(config.enable_docs);
        assert!(config.enable_metrics);
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.max_request_size, 10 * 1024 * 1024);
    }
    
    #[test]
    fn test_api_manager_creation() {
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            base_path: "/api/v2".to_string(),
            enable_cors: true,
            enable_auth: true,
            enable_rate_limiting: true,
            enable_docs: true,
            enable_metrics: true,
            timeout_ms: 30000,
            max_request_size: 10 * 1024 * 1024, // 10MB
        };
        
        let metrics = Arc::new(MetricsCollector::new("api"));
        let manager = ApiManager::new(config, metrics);
        
        assert_eq!(manager.get_config().host, "127.0.0.1");
        assert_eq!(manager.get_config().port, 8080);
        assert!(!manager.is_running());
    }
}