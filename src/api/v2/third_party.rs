use crate::error::Error;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

/// サードパーティ統合設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// 統合名
    pub name: String,
    /// 説明
    pub description: String,
    /// バージョン
    pub version: String,
    /// 有効かどうか
    pub enabled: bool,
    /// APIキー
    pub api_key: Option<String>,
    /// APIシークレット
    pub api_secret: Option<String>,
    /// エンドポイントURL
    pub endpoint_url: Option<String>,
    /// 追加設定
    pub additional_config: HashMap<String, String>,
}

/// サードパーティ統合トレイト
#[async_trait]
pub trait ThirdPartyIntegration: Send + Sync {
    /// 統合名を取得
    fn name(&self) -> &str;
    
    /// 説明を取得
    fn description(&self) -> &str;
    
    /// バージョンを取得
    fn version(&self) -> &str;
    
    /// 有効かどうかを確認
    fn is_enabled(&self) -> bool;
    
    /// 統合を初期化
    async fn initialize(&self) -> Result<(), Error>;
    
    /// 統合を終了
    async fn shutdown(&self) -> Result<(), Error>;
    
    /// 統合を実行
    async fn execute(&self, action: &str, params: &[u8]) -> Result<Vec<u8>, Error>;
    
    /// 統合設定を取得
    fn get_config(&self) -> IntegrationConfig;
    
    /// 統合設定を更新
    fn update_config(&mut self, config: IntegrationConfig) -> Result<(), Error>;
    
    /// サポートされているアクションを取得
    fn get_supported_actions(&self) -> Vec<String>;
    
    /// ヘルスチェック
    async fn health_check(&self) -> Result<bool, Error>;
}

/// サードパーティ統合レジストリ
pub struct IntegrationRegistry {
    /// 統合マップ
    integrations: HashMap<String, Box<dyn ThirdPartyIntegration>>,
}

impl IntegrationRegistry {
    /// 新しいIntegrationRegistryを作成
    pub fn new() -> Self {
        Self {
            integrations: HashMap::new(),
        }
    }
    
    /// 統合を登録
    pub fn register_integration(&mut self, integration: Box<dyn ThirdPartyIntegration>) -> Result<(), Error> {
        let name = integration.name().to_string();
        
        if self.integrations.contains_key(&name) {
            return Err(Error::DuplicateEntry(format!("Integration already exists: {}", name)));
        }
        
        self.integrations.insert(name, integration);
        
        Ok(())
    }
    
    /// 統合を取得
    pub fn get_integration(&self, name: &str) -> Option<Box<dyn ThirdPartyIntegration>> {
        self.integrations.get(name).map(|integration| {
            // クローンを作成
            let config = integration.get_config();
            Box::new(GenericIntegration::new(config)) as Box<dyn ThirdPartyIntegration>
        })
    }
    
    /// 統合を削除
    pub fn remove_integration(&mut self, name: &str) -> Result<(), Error> {
        if !self.integrations.contains_key(name) {
            return Err(Error::NotFound(format!("Integration not found: {}", name)));
        }
        
        self.integrations.remove(name);
        
        Ok(())
    }
    
    /// 統合のリストを取得
    pub fn get_integrations(&self) -> Vec<&dyn ThirdPartyIntegration> {
        self.integrations.values().map(|i| i.as_ref()).collect()
    }
    
    /// 統合名のリストを取得
    pub fn get_integration_names(&self) -> Vec<String> {
        self.integrations.keys().cloned().collect()
    }
    
    /// 統合設定を更新
    pub fn update_integration_config(&mut self, name: &str, config: IntegrationConfig) -> Result<(), Error> {
        let integration = self.integrations.get_mut(name)
            .ok_or_else(|| Error::NotFound(format!("Integration not found: {}", name)))?;
        
        integration.update_config(config)
    }
}

/// 汎用統合実装
pub struct GenericIntegration {
    /// 設定
    config: IntegrationConfig,
    /// サポートされているアクション
    supported_actions: Vec<String>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl GenericIntegration {
    /// 新しいGenericIntegrationを作成
    pub fn new(config: IntegrationConfig) -> Self {
        Self {
            config,
            supported_actions: vec![
                "ping".to_string(),
                "status".to_string(),
                "info".to_string(),
            ],
            initialized: false,
        }
    }
}

#[async_trait]
impl ThirdPartyIntegration for GenericIntegration {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn description(&self) -> &str {
        &self.config.description
    }
    
    fn version(&self) -> &str {
        &self.config.version
    }
    
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    async fn initialize(&self) -> Result<(), Error> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // 実際の実装では、サードパーティサービスへの接続を初期化
        // ここでは、テスト用のダミーロジックを提供
        
        info!("Initializing integration: {}", self.config.name);
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), Error> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // 実際の実装では、サードパーティサービスへの接続を終了
        // ここでは、テスト用のダミーロジックを提供
        
        info!("Shutting down integration: {}", self.config.name);
        
        Ok(())
    }
    
    async fn execute(&self, action: &str, params: &[u8]) -> Result<Vec<u8>, Error> {
        if !self.config.enabled {
            return Err(Error::InvalidState(format!("Integration is disabled: {}", self.config.name)));
        }
        
        if !self.supported_actions.contains(&action.to_string()) {
            return Err(Error::InvalidArgument(format!("Unsupported action: {}", action)));
        }
        
        // 実際の実装では、サードパーティサービスにリクエストを送信
        // ここでは、テスト用のダミーロジックを提供
        
        match action {
            "ping" => {
                Ok(b"pong".to_vec())
            },
            "status" => {
                let status = format!("{{\"status\":\"ok\",\"name\":\"{}\",\"version\":\"{}\"}}", self.config.name, self.config.version);
                Ok(status.into_bytes())
            },
            "info" => {
                let info = format!("{{\"name\":\"{}\",\"description\":\"{}\",\"version\":\"{}\",\"enabled\":{}}}", 
                    self.config.name, 
                    self.config.description, 
                    self.config.version, 
                    self.config.enabled
                );
                Ok(info.into_bytes())
            },
            _ => {
                Err(Error::InvalidArgument(format!("Unsupported action: {}", action)))
            }
        }
    }
    
    fn get_config(&self) -> IntegrationConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: IntegrationConfig) -> Result<(), Error> {
        // 名前が一致するか確認
        if self.config.name != config.name {
            return Err(Error::InvalidArgument(format!(
                "Integration name mismatch: {} != {}",
                self.config.name,
                config.name
            )));
        }
        
        self.config = config;
        
        Ok(())
    }
    
    fn get_supported_actions(&self) -> Vec<String> {
        self.supported_actions.clone()
    }
    
    async fn health_check(&self) -> Result<bool, Error> {
        if !self.config.enabled {
            return Ok(false);
        }
        
        // 実際の実装では、サードパーティサービスのヘルスチェックを実行
        // ここでは、テスト用のダミーロジックを提供
        
        Ok(true)
    }
}

/// Slack統合
pub struct SlackIntegration {
    /// 設定
    config: IntegrationConfig,
    /// クライアント
    client: Option<reqwest::Client>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl SlackIntegration {
    /// 新しいSlackIntegrationを作成
    pub fn new(webhook_url: &str) -> Self {
        let mut config = IntegrationConfig {
            name: "slack".to_string(),
            description: "Slack integration for notifications".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            api_key: None,
            api_secret: None,
            endpoint_url: Some(webhook_url.to_string()),
            additional_config: HashMap::new(),
        };
        
        Self {
            config,
            client: None,
            initialized: false,
        }
    }
    
    /// メッセージを送信
    async fn send_message(&self, channel: &str, message: &str) -> Result<(), Error> {
        if !self.initialized || !self.config.enabled {
            return Err(Error::InvalidState("Slack integration is not initialized or disabled".to_string()));
        }
        
        let webhook_url = self.config.endpoint_url.as_ref()
            .ok_or_else(|| Error::InvalidState("Webhook URL is not set".to_string()))?;
        
        let client = self.client.as_ref().unwrap();
        
        // Slackメッセージペイロードを作成
        let payload = serde_json::json!({
            "channel": channel,
            "text": message,
        });
        
        // Webhookにリクエストを送信
        let response = client.post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::RequestError(format!("Failed to send Slack message: {}", e)))?;
        
        // レスポンスをチェック
        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(Error::ResponseError(format!("Slack API error: {}", error_text)));
        }
        
        Ok(())
    }
}

#[async_trait]
impl ThirdPartyIntegration for SlackIntegration {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn description(&self) -> &str {
        &self.config.description
    }
    
    fn version(&self) -> &str {
        &self.config.version
    }
    
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    async fn initialize(&self) -> Result<(), Error> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // HTTPクライアントを作成
        let client = reqwest::Client::new();
        
        // 設定を検証
        if self.config.endpoint_url.is_none() {
            return Err(Error::InvalidArgument("Webhook URL is required for Slack integration".to_string()));
        }
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), Error> {
        // 特に何もする必要はない
        Ok(())
    }
    
    async fn execute(&self, action: &str, params: &[u8]) -> Result<Vec<u8>, Error> {
        if !self.config.enabled {
            return Err(Error::InvalidState("Slack integration is disabled".to_string()));
        }
        
        match action {
            "send_message" => {
                // パラメータをパース
                let params_str = std::str::from_utf8(params)
                    .map_err(|e| Error::InvalidArgument(format!("Invalid UTF-8 in params: {}", e)))?;
                
                let params: serde_json::Value = serde_json::from_str(params_str)
                    .map_err(|e| Error::InvalidArgument(format!("Invalid JSON in params: {}", e)))?;
                
                let channel = params["channel"].as_str()
                    .ok_or_else(|| Error::InvalidArgument("Missing 'channel' parameter".to_string()))?;
                
                let message = params["message"].as_str()
                    .ok_or_else(|| Error::InvalidArgument("Missing 'message' parameter".to_string()))?;
                
                // メッセージを送信
                self.send_message(channel, message).await?;
                
                Ok(b"Message sent".to_vec())
            },
            _ => {
                Err(Error::InvalidArgument(format!("Unsupported action: {}", action)))
            }
        }
    }
    
    fn get_config(&self) -> IntegrationConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: IntegrationConfig) -> Result<(), Error> {
        // 名前が一致するか確認
        if self.config.name != config.name {
            return Err(Error::InvalidArgument(format!(
                "Integration name mismatch: {} != {}",
                self.config.name,
                config.name
            )));
        }
        
        self.config = config;
        
        Ok(())
    }
    
    fn get_supported_actions(&self) -> Vec<String> {
        vec!["send_message".to_string()]
    }
    
    async fn health_check(&self) -> Result<bool, Error> {
        if !self.config.enabled {
            return Ok(false);
        }
        
        // Webhookが設定されているか確認
        if self.config.endpoint_url.is_none() {
            return Ok(false);
        }
        
        Ok(true)
    }
}

/// Discord統合
pub struct DiscordIntegration {
    /// 設定
    config: IntegrationConfig,
    /// クライアント
    client: Option<reqwest::Client>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl DiscordIntegration {
    /// 新しいDiscordIntegrationを作成
    pub fn new(webhook_url: &str) -> Self {
        let mut config = IntegrationConfig {
            name: "discord".to_string(),
            description: "Discord integration for notifications".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            api_key: None,
            api_secret: None,
            endpoint_url: Some(webhook_url.to_string()),
            additional_config: HashMap::new(),
        };
        
        Self {
            config,
            client: None,
            initialized: false,
        }
    }
    
    /// メッセージを送信
    async fn send_message(&self, message: &str, username: Option<&str>, avatar_url: Option<&str>) -> Result<(), Error> {
        if !self.initialized || !self.config.enabled {
            return Err(Error::InvalidState("Discord integration is not initialized or disabled".to_string()));
        }
        
        let webhook_url = self.config.endpoint_url.as_ref()
            .ok_or_else(|| Error::InvalidState("Webhook URL is not set".to_string()))?;
        
        let client = self.client.as_ref().unwrap();
        
        // Discordメッセージペイロードを作成
        let mut payload = serde_json::json!({
            "content": message,
        });
        
        if let Some(username) = username {
            payload["username"] = serde_json::Value::String(username.to_string());
        }
        
        if let Some(avatar_url) = avatar_url {
            payload["avatar_url"] = serde_json::Value::String(avatar_url.to_string());
        }
        
        // Webhookにリクエストを送信
        let response = client.post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::RequestError(format!("Failed to send Discord message: {}", e)))?;
        
        // レスポンスをチェック
        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(Error::ResponseError(format!("Discord API error: {}", error_text)));
        }
        
        Ok(())
    }
}

#[async_trait]
impl ThirdPartyIntegration for DiscordIntegration {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn description(&self) -> &str {
        &self.config.description
    }
    
    fn version(&self) -> &str {
        &self.config.version
    }
    
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    async fn initialize(&self) -> Result<(), Error> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // HTTPクライアントを作成
        let client = reqwest::Client::new();
        
        // 設定を検証
        if self.config.endpoint_url.is_none() {
            return Err(Error::InvalidArgument("Webhook URL is required for Discord integration".to_string()));
        }
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), Error> {
        // 特に何もする必要はない
        Ok(())
    }
    
    async fn execute(&self, action: &str, params: &[u8]) -> Result<Vec<u8>, Error> {
        if !self.config.enabled {
            return Err(Error::InvalidState("Discord integration is disabled".to_string()));
        }
        
        match action {
            "send_message" => {
                // パラメータをパース
                let params_str = std::str::from_utf8(params)
                    .map_err(|e| Error::InvalidArgument(format!("Invalid UTF-8 in params: {}", e)))?;
                
                let params: serde_json::Value = serde_json::from_str(params_str)
                    .map_err(|e| Error::InvalidArgument(format!("Invalid JSON in params: {}", e)))?;
                
                let message = params["message"].as_str()
                    .ok_or_else(|| Error::InvalidArgument("Missing 'message' parameter".to_string()))?;
                
                let username = params["username"].as_str();
                let avatar_url = params["avatar_url"].as_str();
                
                // メッセージを送信
                self.send_message(message, username, avatar_url).await?;
                
                Ok(b"Message sent".to_vec())
            },
            _ => {
                Err(Error::InvalidArgument(format!("Unsupported action: {}", action)))
            }
        }
    }
    
    fn get_config(&self) -> IntegrationConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: IntegrationConfig) -> Result<(), Error> {
        // 名前が一致するか確認
        if self.config.name != config.name {
            return Err(Error::InvalidArgument(format!(
                "Integration name mismatch: {} != {}",
                self.config.name,
                config.name
            )));
        }
        
        self.config = config;
        
        Ok(())
    }
    
    fn get_supported_actions(&self) -> Vec<String> {
        vec!["send_message".to_string()]
    }
    
    async fn health_check(&self) -> Result<bool, Error> {
        if !self.config.enabled {
            return Ok(false);
        }
        
        // Webhookが設定されているか確認
        if self.config.endpoint_url.is_none() {
            return Ok(false);
        }
        
        Ok(true)
    }
}

impl Default for IntegrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_integration_config() {
        let config = IntegrationConfig {
            name: "test".to_string(),
            description: "Test integration".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            api_key: Some("test_key".to_string()),
            api_secret: Some("test_secret".to_string()),
            endpoint_url: Some("https://example.com/api".to_string()),
            additional_config: {
                let mut config = HashMap::new();
                config.insert("param1".to_string(), "value1".to_string());
                config.insert("param2".to_string(), "value2".to_string());
                config
            },
        };
        
        assert_eq!(config.name, "test");
        assert_eq!(config.description, "Test integration");
        assert_eq!(config.version, "1.0.0");
        assert!(config.enabled);
        assert_eq!(config.api_key, Some("test_key".to_string()));
        assert_eq!(config.api_secret, Some("test_secret".to_string()));
        assert_eq!(config.endpoint_url, Some("https://example.com/api".to_string()));
        assert_eq!(config.additional_config.len(), 2);
        assert_eq!(config.additional_config.get("param1"), Some(&"value1".to_string()));
        assert_eq!(config.additional_config.get("param2"), Some(&"value2".to_string()));
    }
    
    #[test]
    fn test_generic_integration() {
        let config = IntegrationConfig {
            name: "test".to_string(),
            description: "Test integration".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            api_key: None,
            api_secret: None,
            endpoint_url: None,
            additional_config: HashMap::new(),
        };
        
        let integration = GenericIntegration::new(config);
        
        assert_eq!(integration.name(), "test");
        assert_eq!(integration.description(), "Test integration");
        assert_eq!(integration.version(), "1.0.0");
        assert!(integration.is_enabled());
        
        let actions = integration.get_supported_actions();
        assert_eq!(actions.len(), 3);
        assert!(actions.contains(&"ping".to_string()));
        assert!(actions.contains(&"status".to_string()));
        assert!(actions.contains(&"info".to_string()));
    }
    
    #[test]
    fn test_integration_registry() {
        let mut registry = IntegrationRegistry::new();
        
        // 統合を作成
        let config1 = IntegrationConfig {
            name: "test1".to_string(),
            description: "Test integration 1".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            api_key: None,
            api_secret: None,
            endpoint_url: None,
            additional_config: HashMap::new(),
        };
        
        let config2 = IntegrationConfig {
            name: "test2".to_string(),
            description: "Test integration 2".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            api_key: None,
            api_secret: None,
            endpoint_url: None,
            additional_config: HashMap::new(),
        };
        
        let integration1 = Box::new(GenericIntegration::new(config1));
        let integration2 = Box::new(GenericIntegration::new(config2));
        
        // 統合を登録
        registry.register_integration(integration1).unwrap();
        registry.register_integration(integration2).unwrap();
        
        // 統合名のリストを取得
        let names = registry.get_integration_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"test1".to_string()));
        assert!(names.contains(&"test2".to_string()));
        
        // 統合を取得
        let integration = registry.get_integration("test1");
        assert!(integration.is_some());
        let integration = integration.unwrap();
        assert_eq!(integration.name(), "test1");
        
        // 存在しない統合を取得
        let integration = registry.get_integration("nonexistent");
        assert!(integration.is_none());
        
        // 統合を削除
        registry.remove_integration("test1").unwrap();
        
        // 統合名のリストを再取得
        let names = registry.get_integration_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"test2".to_string()));
    }
}