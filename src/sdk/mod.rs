// SDK モジュール
//
// このモジュールは、ShardXプラットフォームの開発者向けSDKを提供します。
// 主な機能:
// - クライアントライブラリ
// - スマートコントラクト開発ツール
// - トランザクション構築ユーティリティ
// - ウォレット統合
// - イベント監視

mod client;
mod contract_tools;
// mod transaction_builder; // TODO: このモジュールが見つかりません
// mod wallet; // TODO: このモジュールが見つかりません
// mod event_monitor; // TODO: このモジュールが見つかりません

pub use self::client::{ShardXClient, ClientConfig, ClientError};
pub use self::contract_tools::{ContractCompiler, ContractDeployer, ContractTemplate};
pub use self::transaction_builder::{TransactionBuilder, TransactionTemplate, BatchBuilder};
pub use self::wallet::{Wallet, KeyManager, WalletConfig};
pub use self::event_monitor::{EventMonitor, EventFilter, EventCallback};

use crate::error::Error;
use std::sync::Arc;
use log::{debug, error, info, warn};

/// SDK マネージャー
pub struct SDKManager {
    /// クライアント
    client: Arc<ShardXClient>,
    /// コントラクトコンパイラ
    contract_compiler: ContractCompiler,
    /// コントラクトデプロイヤー
    contract_deployer: ContractDeployer,
    /// トランザクションビルダー
    transaction_builder: TransactionBuilder,
    /// ウォレット
    wallet: Wallet,
    /// イベントモニター
    event_monitor: EventMonitor,
}

impl SDKManager {
    /// 新しいSDKManagerを作成
    pub fn new(endpoint: &str) -> Result<Self, Error> {
        // クライアント設定
        let config = ClientConfig {
            endpoint: endpoint.to_string(),
            timeout_ms: 5000,
            max_retries: 3,
            api_key: None,
        };
        
        // クライアントを作成
        let client = Arc::new(ShardXClient::new(config)?);
        
        // コントラクトコンパイラを作成
        let contract_compiler = ContractCompiler::new();
        
        // コントラクトデプロイヤーを作成
        let contract_deployer = ContractDeployer::new(client.clone());
        
        // トランザクションビルダーを作成
        let transaction_builder = TransactionBuilder::new(client.clone());
        
        // ウォレットを作成
        let wallet = Wallet::new(WalletConfig::default())?;
        
        // イベントモニターを作成
        let event_monitor = EventMonitor::new(client.clone());
        
        Ok(Self {
            client,
            contract_compiler,
            contract_deployer,
            transaction_builder,
            wallet,
            event_monitor,
        })
    }
    
    /// クライアントを取得
    pub fn client(&self) -> Arc<ShardXClient> {
        self.client.clone()
    }
    
    /// コントラクトコンパイラを取得
    pub fn contract_compiler(&self) -> &ContractCompiler {
        &self.contract_compiler
    }
    
    /// コントラクトデプロイヤーを取得
    pub fn contract_deployer(&self) -> &ContractDeployer {
        &self.contract_deployer
    }
    
    /// トランザクションビルダーを取得
    pub fn transaction_builder(&self) -> &TransactionBuilder {
        &self.transaction_builder
    }
    
    /// ウォレットを取得
    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }
    
    /// イベントモニターを取得
    pub fn event_monitor(&self) -> &EventMonitor {
        &self.event_monitor
    }
    
    /// スマートコントラクトをコンパイルしてデプロイ
    pub async fn compile_and_deploy_contract(
        &self,
        source_code: &str,
        constructor_args: &[Vec<u8>],
    ) -> Result<String, Error> {
        // コントラクトをコンパイル
        let compiled = self.contract_compiler.compile(source_code)?;
        
        // コントラクトをデプロイ
        let contract_id = self.contract_deployer.deploy(&compiled, constructor_args).await?;
        
        Ok(contract_id)
    }
    
    /// トランザクションを作成して送信
    pub async fn create_and_send_transaction(
        &self,
        recipient: &str,
        amount: u64,
        data: Option<Vec<u8>>,
    ) -> Result<String, Error> {
        // トランザクションを作成
        let transaction = self.transaction_builder.create_transaction(
            &self.wallet.get_address()?,
            recipient,
            amount,
            data,
        )?;
        
        // トランザクションに署名
        let signed_transaction = self.wallet.sign_transaction(&transaction)?;
        
        // トランザクションを送信
        let tx_id = self.client.send_transaction(&signed_transaction).await?;
        
        Ok(tx_id)
    }
    
    /// イベントの監視を開始
    pub async fn start_monitoring_events<F>(
        &self,
        filter: EventFilter,
        callback: F,
    ) -> Result<String, Error>
    where
        F: Fn(Vec<u8>) -> Result<(), Error> + Send + Sync + 'static,
    {
        self.event_monitor.start_monitoring(filter, callback).await
    }
    
    /// イベントの監視を停止
    pub async fn stop_monitoring_events(&self, monitor_id: &str) -> Result<(), Error> {
        self.event_monitor.stop_monitoring(monitor_id).await
    }
    
    /// アカウント残高を取得
    pub async fn get_account_balance(&self, address: &str) -> Result<u64, Error> {
        self.client.get_balance(address).await
    }
    
    /// コントラクト関数を呼び出し
    pub async fn call_contract_function(
        &self,
        contract_id: &str,
        function_name: &str,
        args: &[Vec<u8>],
    ) -> Result<Vec<u8>, Error> {
        self.client.call_contract(contract_id, function_name, args).await
    }
    
    /// コントラクト関数をトランザクションとして実行
    pub async fn execute_contract_function(
        &self,
        contract_id: &str,
        function_name: &str,
        args: &[Vec<u8>],
        value: u64,
    ) -> Result<String, Error> {
        // コントラクト呼び出しデータを作成
        let data = self.transaction_builder.create_contract_call_data(contract_id, function_name, args)?;
        
        // トランザクションを作成して送信
        self.create_and_send_transaction(contract_id, value, Some(data)).await
    }
    
    /// ブロック情報を取得
    pub async fn get_block_info(&self, block_id: &str) -> Result<serde_json::Value, Error> {
        self.client.get_block(block_id).await
    }
    
    /// トランザクション情報を取得
    pub async fn get_transaction_info(&self, tx_id: &str) -> Result<serde_json::Value, Error> {
        self.client.get_transaction(tx_id).await
    }
    
    /// ネットワーク情報を取得
    pub async fn get_network_info(&self) -> Result<serde_json::Value, Error> {
        self.client.get_network_info().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    
    #[test]
    fn test_sdk_manager_creation() {
        let sdk = SDKManager::new("http://localhost:8545");
        assert!(sdk.is_ok());
    }
    
    #[test]
    fn test_client_access() {
        let sdk = SDKManager::new("http://localhost:8545").unwrap();
        let client = sdk.client();
        assert_eq!(client.config().endpoint, "http://localhost:8545");
    }
    
    #[test]
    fn test_wallet_creation() {
        let sdk = SDKManager::new("http://localhost:8545").unwrap();
        let wallet = sdk.wallet();
        assert!(wallet.is_initialized());
    }
    
    #[test]
    fn test_transaction_builder() {
        let sdk = SDKManager::new("http://localhost:8545").unwrap();
        let tx_builder = sdk.transaction_builder();
        assert!(tx_builder.is_initialized());
    }
}