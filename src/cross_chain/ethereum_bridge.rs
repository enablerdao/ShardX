use std::sync::{Arc, RwLock};
use std::str::FromStr;
use tokio::sync::mpsc;
use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};
use web3::{
    Web3,
    transports::Http,
    types::{Address, H256, U256, TransactionRequest, BlockNumber},
};
use ethers::{
    prelude::*,
    providers::{Provider, Http as EthersHttp},
    signers::{LocalWallet, Signer},
};

use crate::error::Error;
use crate::transaction::Transaction;
use super::bridge::{CrossChainBridge, BridgeConfig, ChainType, BridgeStatus, FeeSetting};
use super::messaging::{CrossChainMessage, MessageType, MessageStatus};
use super::transaction::{CrossChainTransaction, TransactionStatus, TransactionProof};

/// Ethereumブリッジ
pub struct EthereumBridge {
    /// 基本ブリッジ
    base_bridge: CrossChainBridge,
    /// Web3クライアント
    web3: Option<Web3<Http>>,
    /// Ethersプロバイダー
    provider: Option<Provider<EthersHttp>>,
    /// ウォレット
    wallet: Option<LocalWallet>,
    /// コントラクトアドレス
    contract_address: Option<Address>,
    /// ガス価格（Gwei）
    gas_price: RwLock<U256>,
    /// 最新のブロック番号
    latest_block: RwLock<u64>,
}

impl EthereumBridge {
    /// 新しいEthereumブリッジを作成
    pub fn new(
        config: BridgeConfig,
        message_sender: mpsc::Sender<CrossChainMessage>,
        message_receiver: mpsc::Receiver<CrossChainMessage>,
    ) -> Self {
        let base_bridge = CrossChainBridge::new(
            config.clone(),
            message_sender,
            message_receiver,
        );
        
        let contract_address = config.target_contract
            .as_ref()
            .and_then(|addr| Address::from_str(addr).ok());
        
        Self {
            base_bridge,
            web3: None,
            provider: None,
            wallet: None,
            contract_address,
            gas_price: RwLock::new(U256::from(5_000_000_000u64)), // 5 Gwei
            latest_block: RwLock::new(0),
        }
    }
    
    /// ブリッジを初期化
    pub async fn initialize(&mut self, private_key: &str) -> Result<(), Error> {
        info!("Initializing Ethereum bridge: {} -> {}", 
            self.base_bridge.get_config().source_chain, 
            self.base_bridge.get_config().target_chain);
        
        // Web3クライアントを初期化
        let endpoint = self.base_bridge.get_config().target_endpoint.clone();
        let http = Http::new(&endpoint)
            .map_err(|e| Error::ConnectionError(format!("Failed to connect to Ethereum node: {}", e)))?;
        
        let web3 = Web3::new(http);
        self.web3 = Some(web3.clone());
        
        // Ethersプロバイダーを初期化
        let provider = Provider::<EthersHttp>::try_from(endpoint.clone())
            .map_err(|e| Error::ConnectionError(format!("Failed to create Ethers provider: {}", e)))?;
        
        self.provider = Some(provider.clone());
        
        // ウォレットを初期化
        let wallet = private_key.parse::<LocalWallet>()
            .map_err(|e| Error::ValidationError(format!("Invalid private key: {}", e)))?;
        
        self.wallet = Some(wallet.clone());
        
        // 接続テスト
        let block_number = web3.eth().block_number().await
            .map_err(|e| Error::ConnectionError(format!("Failed to get block number: {}", e)))?;
        
        let block_number_u64 = block_number.as_u64();
        *self.latest_block.write().unwrap() = block_number_u64;
        
        info!("Connected to Ethereum network. Latest block: {}", block_number_u64);
        
        // ガス価格を取得
        let gas_price = web3.eth().gas_price().await
            .map_err(|e| Error::ConnectionError(format!("Failed to get gas price: {}", e)))?;
        
        *self.gas_price.write().unwrap() = gas_price;
        
        info!("Current gas price: {} Gwei", gas_price.as_u64() / 1_000_000_000);
        
        // コントラクトアドレスを検証
        if let Some(addr) = self.contract_address {
            let code = web3.eth().code(addr, None).await
                .map_err(|e| Error::ConnectionError(format!("Failed to get contract code: {}", e)))?;
            
            if code.0.is_empty() {
                return Err(Error::ValidationError(format!(
                    "No contract found at address: {}", addr
                )));
            }
            
            info!("Contract verified at address: {}", addr);
        }
        
        // 基本ブリッジを初期化
        self.base_bridge.initialize().await?;
        
        info!("Ethereum bridge initialized successfully");
        
        Ok(())
    }
    
    /// トランザクションを送信
    pub async fn send_transaction(&self, transaction: &Transaction) -> Result<H256, Error> {
        // Web3クライアントを取得
        let web3 = self.web3.as_ref()
            .ok_or_else(|| Error::ConnectionError("Web3 client not initialized".to_string()))?;
        
        // ウォレットを取得
        let wallet = self.wallet.as_ref()
            .ok_or_else(|| Error::ValidationError("Wallet not initialized".to_string()))?;
        
        // 送信先アドレスを解析
        let to_address = Address::from_str(&transaction.to)
            .map_err(|e| Error::ValidationError(format!("Invalid to address: {}", e)))?;
        
        // 金額を解析
        let amount = U256::from_dec_str(&transaction.amount)
            .map_err(|e| Error::ValidationError(format!("Invalid amount: {}", e)))?;
        
        // ガス価格を取得
        let gas_price = *self.gas_price.read().unwrap();
        
        // トランザクションリクエストを作成
        let tx_request = TransactionRequest {
            from: wallet.address().into(),
            to: Some(to_address),
            gas: None, // 自動推定
            gas_price: Some(gas_price),
            value: Some(amount),
            data: None,
            nonce: None, // 自動取得
            condition: None,
        };
        
        // トランザクションを送信
        let tx_hash = web3.eth().send_transaction(tx_request).await
            .map_err(|e| Error::TransactionError(format!("Failed to send transaction: {}", e)))?;
        
        info!("Transaction sent: {}", tx_hash);
        
        Ok(tx_hash)
    }
    
    /// トランザクションの状態を確認
    pub async fn check_transaction_status(&self, tx_hash: H256) -> Result<TransactionStatus, Error> {
        // Web3クライアントを取得
        let web3 = self.web3.as_ref()
            .ok_or_else(|| Error::ConnectionError("Web3 client not initialized".to_string()))?;
        
        // トランザクションのレシートを取得
        let receipt = web3.eth().transaction_receipt(tx_hash).await
            .map_err(|e| Error::TransactionError(format!("Failed to get transaction receipt: {}", e)))?;
        
        if let Some(receipt) = receipt {
            // ブロック番号を取得
            let block_number = receipt.block_number
                .ok_or_else(|| Error::TransactionError("Missing block number in receipt".to_string()))?
                .as_u64();
            
            // 最新のブロック番号を取得
            let latest_block = *self.latest_block.read().unwrap();
            
            // 確認数を計算
            let confirmations = latest_block.saturating_sub(block_number) + 1;
            
            // 必要な確認数
            let required_confirmations = self.base_bridge.get_config().confirmation_blocks;
            
            if receipt.status == Some(U256::from(1)) {
                // トランザクションが成功
                if confirmations >= required_confirmations {
                    Ok(TransactionStatus::Confirmed)
                } else {
                    Ok(TransactionStatus::Confirming)
                }
            } else {
                // トランザクションが失敗
                Ok(TransactionStatus::Failed)
            }
        } else {
            // レシートがない場合はまだ処理中
            Ok(TransactionStatus::Sent)
        }
    }
    
    /// トランザクション証明を作成
    pub async fn create_transaction_proof(
        &self,
        tx_hash: H256,
        tx_id: &str,
    ) -> Result<TransactionProof, Error> {
        // Web3クライアントを取得
        let web3 = self.web3.as_ref()
            .ok_or_else(|| Error::ConnectionError("Web3 client not initialized".to_string()))?;
        
        // トランザクションのレシートを取得
        let receipt = web3.eth().transaction_receipt(tx_hash).await
            .map_err(|e| Error::TransactionError(format!("Failed to get transaction receipt: {}", e)))?
            .ok_or_else(|| Error::TransactionError("Transaction receipt not found".to_string()))?;
        
        // ブロック情報を取得
        let block_number = receipt.block_number
            .ok_or_else(|| Error::TransactionError("Missing block number in receipt".to_string()))?;
        
        let block = web3.eth().block(BlockNumber::Number(block_number)).await
            .map_err(|e| Error::TransactionError(format!("Failed to get block: {}", e)))?
            .ok_or_else(|| Error::TransactionError("Block not found".to_string()))?;
        
        let block_hash = block.hash
            .ok_or_else(|| Error::TransactionError("Missing block hash".to_string()))?;
        
        // 証明データを作成
        let proof_data = serde_json::to_vec(&receipt)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
        
        // 署名を作成
        let wallet = self.wallet.as_ref()
            .ok_or_else(|| Error::ValidationError("Wallet not initialized".to_string()))?;
        
        let message = format!("{}:{}", tx_hash, block_hash);
        let signature = wallet.sign_message(message.as_bytes()).to_string();
        
        // 証明を作成
        let proof = TransactionProof::new(
            tx_id.to_string(),
            block_hash.to_string(),
            block_number.as_u64(),
            block.timestamp.as_u64(),
            proof_data,
            signature,
            wallet.address().to_string(),
        );
        
        Ok(proof)
    }
    
    /// 最新のブロック番号を更新
    pub async fn update_latest_block(&self) -> Result<u64, Error> {
        // Web3クライアントを取得
        let web3 = self.web3.as_ref()
            .ok_or_else(|| Error::ConnectionError("Web3 client not initialized".to_string()))?;
        
        // 最新のブロック番号を取得
        let block_number = web3.eth().block_number().await
            .map_err(|e| Error::ConnectionError(format!("Failed to get block number: {}", e)))?;
        
        let block_number_u64 = block_number.as_u64();
        *self.latest_block.write().unwrap() = block_number_u64;
        
        Ok(block_number_u64)
    }
    
    /// ガス価格を更新
    pub async fn update_gas_price(&self) -> Result<U256, Error> {
        // Web3クライアントを取得
        let web3 = self.web3.as_ref()
            .ok_or_else(|| Error::ConnectionError("Web3 client not initialized".to_string()))?;
        
        // ガス価格を取得
        let gas_price = web3.eth().gas_price().await
            .map_err(|e| Error::ConnectionError(format!("Failed to get gas price: {}", e)))?;
        
        *self.gas_price.write().unwrap() = gas_price;
        
        Ok(gas_price)
    }
    
    /// 残高を取得
    pub async fn get_balance(&self, address: &str) -> Result<U256, Error> {
        // Web3クライアントを取得
        let web3 = self.web3.as_ref()
            .ok_or_else(|| Error::ConnectionError("Web3 client not initialized".to_string()))?;
        
        // アドレスを解析
        let eth_address = Address::from_str(address)
            .map_err(|e| Error::ValidationError(format!("Invalid address: {}", e)))?;
        
        // 残高を取得
        let balance = web3.eth().balance(eth_address, None).await
            .map_err(|e| Error::ConnectionError(format!("Failed to get balance: {}", e)))?;
        
        Ok(balance)
    }
    
    /// クロスチェーントランザクションを開始
    pub async fn start_cross_chain_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<String, Error> {
        // 基本ブリッジを使用してトランザクションを開始
        let tx_id = self.base_bridge.start_transaction(transaction).await?;
        
        info!("Started cross-chain transaction: {}", tx_id);
        
        Ok(tx_id)
    }
    
    /// トランザクションの状態を取得
    pub fn get_transaction_status(&self, tx_id: &str) -> Result<TransactionStatus, Error> {
        self.base_bridge.get_transaction_status(tx_id)
    }
    
    /// トランザクションの詳細を取得
    pub fn get_transaction_details(&self, tx_id: &str) -> Result<CrossChainTransaction, Error> {
        self.base_bridge.get_transaction_details(tx_id)
    }
    
    /// ブリッジの状態を取得
    pub fn get_status(&self) -> BridgeStatus {
        self.base_bridge.get_status()
    }
    
    /// ブリッジの設定を取得
    pub fn get_config(&self) -> BridgeConfig {
        self.base_bridge.get_config()
    }
    
    /// ブリッジを停止
    pub async fn shutdown(&self) -> Result<(), Error> {
        self.base_bridge.shutdown().await
    }
}