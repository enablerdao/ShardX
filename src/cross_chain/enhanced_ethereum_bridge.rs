use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};
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
use super::bridge::{CrossChainBridge, BridgeConfig, ChainType, BridgeStatus};
use super::messaging::{CrossChainMessage, MessageType, MessageStatus};
use super::transaction::{CrossChainTransaction, TransactionStatus, TransactionProof};
use super::ethereum_bridge_contract::EthereumBridgeContract;
use super::token_registry::{TokenRegistry, TokenInfo};

/// 拡張イーサリアムブリッジ
pub struct EnhancedEthereumBridge {
    /// 基本ブリッジ
    base_bridge: CrossChainBridge,
    /// Web3クライアント
    web3: Option<Web3<Http>>,
    /// Ethersプロバイダー
    provider: Option<Provider<EthersHttp>>,
    /// ウォレット
    wallet: Option<LocalWallet>,
    /// ブリッジコントラクト
    bridge_contract: Option<EthereumBridgeContract>,
    /// トークンレジストリ
    token_registry: Arc<TokenRegistry>,
    /// ガス価格（Gwei）
    gas_price: RwLock<U256>,
    /// 最新のブロック番号
    latest_block: RwLock<u64>,
    /// 進行中のクロスチェーントランザクション
    pending_transactions: RwLock<HashMap<String, CrossChainTransaction>>,
    /// イベントポーリングタスクが実行中かどうか
    polling_active: RwLock<bool>,
}

impl EnhancedEthereumBridge {
    /// 新しいEnhancedEthereumBridgeを作成
    pub fn new(
        config: BridgeConfig,
        message_sender: mpsc::Sender<CrossChainMessage>,
        message_receiver: mpsc::Receiver<CrossChainMessage>,
        token_registry: Arc<TokenRegistry>,
    ) -> Self {
        let base_bridge = CrossChainBridge::new(
            config.clone(),
            message_sender,
            message_receiver,
        );
        
        Self {
            base_bridge,
            web3: None,
            provider: None,
            wallet: None,
            bridge_contract: None,
            token_registry,
            gas_price: RwLock::new(U256::from(5_000_000_000u64)), // 5 Gwei
            latest_block: RwLock::new(0),
            pending_transactions: RwLock::new(HashMap::new()),
            polling_active: RwLock::new(false),
        }
    }
    
    /// ブリッジを初期化
    pub async fn initialize(&mut self, private_key: &str) -> Result<(), Error> {
        info!("Initializing enhanced Ethereum bridge: {} -> {}", 
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
        
        // ブリッジコントラクトを初期化
        let contract_address = match &self.base_bridge.get_config().target_contract {
            Some(addr) => Address::from_str(addr)
                .map_err(|e| Error::ValidationError(format!("Invalid contract address: {}", e)))?,
            None => return Err(Error::ValidationError("Bridge contract address not specified".to_string())),
        };
        
        let mut bridge_contract = EthereumBridgeContract::new(
            contract_address,
            web3.clone(),
            wallet.clone(),
        );
        
        bridge_contract.initialize().await?;
        self.bridge_contract = Some(bridge_contract);
        
        // 基本ブリッジを初期化
        self.base_bridge.initialize().await?;
        
        // トークンレジストリにデフォルトトークンを登録
        self.token_registry.register_default_tokens()?;
        
        // イベントポーリングタスクを開始
        self.start_event_polling().await?;
        
        info!("Enhanced Ethereum bridge initialized successfully");
        
        Ok(())
    }
    
    /// イベントポーリングタスクを開始
    async fn start_event_polling(&self) -> Result<(), Error> {
        // 既に実行中の場合は何もしない
        {
            let polling_active = self.polling_active.read().unwrap();
            if *polling_active {
                return Ok(());
            }
        }
        
        // ポーリングアクティブフラグを設定
        {
            let mut polling_active = self.polling_active.write().unwrap();
            *polling_active = true;
        }
        
        // ブリッジコントラクトを取得
        let bridge_contract = match &self.bridge_contract {
            Some(contract) => contract.clone(),
            None => return Err(Error::ContractError("Bridge contract not initialized".to_string())),
        };
        
        // ポーリング間隔（15秒）
        let mut interval = interval(Duration::from_secs(15));
        
        // ポーリングタスクを開始
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                // ポーリングが無効化された場合は終了
                {
                    let polling_active = self.polling_active.read().unwrap();
                    if !*polling_active {
                        break;
                    }
                }
                
                // イベントをポーリング
                if let Err(e) = bridge_contract.poll_events().await {
                    error!("Failed to poll events: {}", e);
                }
                
                // 最新のブロック番号を更新
                if let Err(e) = self.update_latest_block().await {
                    error!("Failed to update latest block: {}", e);
                }
                
                // 保留中のトランザクションを処理
                if let Err(e) = self.process_pending_transactions().await {
                    error!("Failed to process pending transactions: {}", e);
                }
            }
        });
        
        info!("Event polling task started");
        
        Ok(())
    }
    
    /// イベントポーリングタスクを停止
    pub fn stop_event_polling(&self) {
        let mut polling_active = self.polling_active.write().unwrap();
        *polling_active = false;
        info!("Event polling task stopped");
    }
    
    /// 最新のブロック番号を更新
    async fn update_latest_block(&self) -> Result<u64, Error> {
        // Web3クライアントを取得
        let web3 = match &self.web3 {
            Some(web3) => web3,
            None => return Err(Error::ConnectionError("Web3 client not initialized".to_string())),
        };
        
        // 最新のブロック番号を取得
        let block_number = web3.eth().block_number().await
            .map_err(|e| Error::ConnectionError(format!("Failed to get block number: {}", e)))?;
        
        let block_number_u64 = block_number.as_u64();
        *self.latest_block.write().unwrap() = block_number_u64;
        
        Ok(block_number_u64)
    }
    
    /// 保留中のトランザクションを処理
    async fn process_pending_transactions(&self) -> Result<(), Error> {
        // 処理するトランザクションのIDリストを取得
        let tx_ids: Vec<String> = {
            let pending_transactions = self.pending_transactions.read().unwrap();
            pending_transactions.keys().cloned().collect()
        };
        
        // 各トランザクションを処理
        for tx_id in tx_ids {
            self.check_transaction_status(&tx_id).await?;
        }
        
        Ok(())
    }
    
    /// トランザクションの状態を確認
    async fn check_transaction_status(&self, tx_id: &str) -> Result<(), Error> {
        // トランザクションを取得
        let transaction = {
            let pending_transactions = self.pending_transactions.read().unwrap();
            match pending_transactions.get(tx_id) {
                Some(tx) => tx.clone(),
                None => return Ok(()),
            }
        };
        
        // Ethereumトランザクションハッシュを取得
        let eth_tx_hash = match transaction.get_metadata("eth_tx_hash") {
            Some(hash) => H256::from_str(hash)
                .map_err(|e| Error::ValidationError(format!("Invalid transaction hash: {}", e)))?,
            None => return Ok(()),
        };
        
        // Web3クライアントを取得
        let web3 = match &self.web3 {
            Some(web3) => web3,
            None => return Err(Error::ConnectionError("Web3 client not initialized".to_string())),
        };
        
        // トランザクションのレシートを取得
        let receipt = web3.eth().transaction_receipt(eth_tx_hash).await
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
            
            let mut updated_transaction = transaction.clone();
            
            if receipt.status == Some(U256::from(1)) {
                // トランザクションが成功
                if confirmations >= required_confirmations {
                    // トランザクションが確認された
                    updated_transaction.status = TransactionStatus::Confirmed;
                    updated_transaction.set_metadata("confirmations".to_string(), confirmations.to_string());
                    
                    // ブロック情報を設定
                    updated_transaction.set_target_block_info(
                        receipt.block_hash.unwrap_or_default().to_string(),
                        block_number,
                    );
                    
                    // トランザクション証明を作成
                    if let Some(bridge_contract) = &self.bridge_contract {
                        if let Ok(proof) = self.create_transaction_proof(&updated_transaction, eth_tx_hash).await {
                            updated_transaction.mark_as_verified(proof);
                        }
                    }
                    
                    // 保留中のトランザクションから削除
                    {
                        let mut pending_transactions = self.pending_transactions.write().unwrap();
                        pending_transactions.remove(tx_id);
                    }
                    
                    info!("Transaction confirmed: {}", tx_id);
                } else {
                    // まだ確認中
                    updated_transaction.status = TransactionStatus::Confirming;
                    updated_transaction.set_metadata("confirmations".to_string(), confirmations.to_string());
                }
            } else {
                // トランザクションが失敗
                updated_transaction.mark_as_failed("Transaction failed on Ethereum".to_string());
                
                // 保留中のトランザクションから削除
                {
                    let mut pending_transactions = self.pending_transactions.write().unwrap();
                    pending_transactions.remove(tx_id);
                }
                
                info!("Transaction failed: {}", tx_id);
            }
            
            // トランザクションの状態を更新
            self.update_transaction(updated_transaction)?;
        }
        
        Ok(())
    }
    
    /// トランザクション証明を作成
    async fn create_transaction_proof(
        &self,
        transaction: &CrossChainTransaction,
        eth_tx_hash: H256,
    ) -> Result<TransactionProof, Error> {
        // Web3クライアントを取得
        let web3 = match &self.web3 {
            Some(web3) => web3,
            None => return Err(Error::ConnectionError("Web3 client not initialized".to_string())),
        };
        
        // トランザクションのレシートを取得
        let receipt = web3.eth().transaction_receipt(eth_tx_hash).await
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
        let wallet = match &self.wallet {
            Some(wallet) => wallet,
            None => return Err(Error::ValidationError("Wallet not initialized".to_string())),
        };
        
        let message = format!("{}:{}", eth_tx_hash, block_hash);
        let signature = wallet.sign_message(message.as_bytes()).to_string();
        
        // 証明を作成
        let proof = TransactionProof::new(
            transaction.id.clone(),
            block_hash.to_string(),
            block_number.as_u64(),
            block.timestamp.as_u64(),
            proof_data,
            signature,
            wallet.address().to_string(),
        );
        
        Ok(proof)
    }
    
    /// トランザクションの状態を更新
    fn update_transaction(&self, transaction: CrossChainTransaction) -> Result<(), Error> {
        // 基本ブリッジのトランザクションマップを更新
        // 実際の実装では、基本ブリッジのトランザクションマップを直接更新する方法を提供する必要がある
        
        Ok(())
    }
    
    /// ShardXからイーサリアムへのトークン転送
    pub async fn transfer_to_ethereum(
        &self,
        token_id: &str,
        recipient: &str,
        amount: &str,
        from_address: &str,
    ) -> Result<String, Error> {
        // トークン情報を取得
        let token = match self.token_registry.get_token(token_id) {
            Some(token) => token,
            None => return Err(Error::ValidationError(format!("Token not found: {}", token_id))),
        };
        
        // トークンがイーサリアムチェーンのものであることを確認
        if token.chain_type != ChainType::Ethereum {
            return Err(Error::ValidationError(format!("Token is not on Ethereum chain: {}", token_id)));
        }
        
        // 受取人アドレスを検証
        let recipient_address = Address::from_str(recipient)
            .map_err(|e| Error::ValidationError(format!("Invalid recipient address: {}", e)))?;
        
        // 金額を検証
        let amount_u256 = U256::from_dec_str(amount)
            .map_err(|e| Error::ValidationError(format!("Invalid amount: {}", e)))?;
        
        // トークンアドレスを取得
        let token_address = Address::from_str(&token.chain_address)
            .map_err(|e| Error::ValidationError(format!("Invalid token address: {}", e)))?;
        
        // ブリッジコントラクトを取得
        let bridge_contract = match &self.bridge_contract {
            Some(contract) => contract,
            None => return Err(Error::ContractError("Bridge contract not initialized".to_string())),
        };
        
        // トークンがサポートされているか確認
        if !bridge_contract.is_token_supported(token_address) {
            return Err(Error::ValidationError(format!("Token not supported by bridge: {}", token.symbol)));
        }
        
        // ShardXトランザクションを作成
        let tx_id = uuid::Uuid::new_v4().to_string();
        
        // 引き出しリクエストを作成
        let withdrawal_id = bridge_contract.request_withdrawal(
            token_address,
            recipient_address,
            amount_u256,
            &tx_id,
        ).await?;
        
        // クロスチェーントランザクションを作成
        let cross_tx = CrossChainTransaction::new(
            Transaction {
                id: tx_id.clone(),
                from: from_address.to_string(),
                to: recipient.to_string(),
                amount: amount.to_string(),
                fee: "0".to_string(),
                data: Some(format!("Transfer to Ethereum: token={}, recipient={}", token.symbol, recipient)),
                nonce: 0,
                timestamp: chrono::Utc::now().timestamp() as u64,
                signature: "".to_string(),
                status: crate::transaction::TransactionStatus::Pending,
                shard_id: "shard-1".to_string(),
                block_hash: None,
                block_height: None,
                parent_id: None,
            },
            ChainType::ShardX,
            ChainType::Ethereum,
        );
        
        // メタデータを設定
        let mut updated_tx = cross_tx.clone();
        updated_tx.set_metadata("token_id".to_string(), token_id.to_string());
        updated_tx.set_metadata("token_symbol".to_string(), token.symbol);
        updated_tx.set_metadata("withdrawal_id".to_string(), withdrawal_id.to_string());
        
        // 保留中のトランザクションに追加
        {
            let mut pending_transactions = self.pending_transactions.write().unwrap();
            pending_transactions.insert(tx_id.clone(), updated_tx.clone());
        }
        
        info!("Created cross-chain transaction from ShardX to Ethereum: {}", tx_id);
        
        Ok(tx_id)
    }
    
    /// イーサリアムからShardXへのトークン転送
    pub async fn transfer_from_ethereum(
        &self,
        token_id: &str,
        shardx_recipient: &str,
        amount: &str,
    ) -> Result<String, Error> {
        // トークン情報を取得
        let token = match self.token_registry.get_token(token_id) {
            Some(token) => token,
            None => return Err(Error::ValidationError(format!("Token not found: {}", token_id))),
        };
        
        // トークンがイーサリアムチェーンのものであることを確認
        if token.chain_type != ChainType::Ethereum {
            return Err(Error::ValidationError(format!("Token is not on Ethereum chain: {}", token_id)));
        }
        
        // ShardX受取人アドレスを検証
        if shardx_recipient.is_empty() {
            return Err(Error::ValidationError("Invalid ShardX recipient address".to_string()));
        }
        
        // 金額を検証
        let amount_u256 = U256::from_dec_str(amount)
            .map_err(|e| Error::ValidationError(format!("Invalid amount: {}", e)))?;
        
        // トークンアドレスを取得
        let token_address = Address::from_str(&token.chain_address)
            .map_err(|e| Error::ValidationError(format!("Invalid token address: {}", e)))?;
        
        // ウォレットを取得
        let wallet = match &self.wallet {
            Some(wallet) => wallet,
            None => return Err(Error::ValidationError("Wallet not initialized".to_string())),
        };
        
        // Web3クライアントを取得
        let web3 = match &self.web3 {
            Some(web3) => web3,
            None => return Err(Error::ConnectionError("Web3 client not initialized".to_string())),
        };
        
        // ブリッジコントラクトアドレスを取得
        let bridge_address = match &self.base_bridge.get_config().target_contract {
            Some(addr) => Address::from_str(addr)
                .map_err(|e| Error::ValidationError(format!("Invalid contract address: {}", e)))?,
            None => return Err(Error::ValidationError("Bridge contract address not specified".to_string())),
        };
        
        // ERC20コントラクトのABIを取得
        let erc20_abi = include_bytes!("../../contracts/ethereum/abi/ERC20.json");
        let erc20_abi = String::from_utf8_lossy(erc20_abi);
        
        // ERC20コントラクトインスタンスを作成
        let erc20_contract = Contract::from_json(
            web3.eth(),
            token_address,
            erc20_abi.as_bytes(),
        ).map_err(|e| Error::ContractError(format!("Failed to create ERC20 contract instance: {}", e)))?;
        
        // approve関数を呼び出し
        let approve_result = erc20_contract.call(
            "approve",
            (bridge_address, amount_u256),
            wallet.address(),
            Options::default(),
        ).await.map_err(|e| Error::ContractError(format!("Failed to approve token transfer: {}", e)))?;
        
        // approveトランザクションのレシートを待機
        let approve_receipt = web3.eth().transaction_receipt(approve_result).await
            .map_err(|e| Error::TransactionError(format!("Failed to get approve receipt: {}", e)))?
            .ok_or_else(|| Error::TransactionError("Approve receipt not found".to_string()))?;
        
        if approve_receipt.status != Some(U256::from(1)) {
            return Err(Error::TransactionError("Approve transaction failed".to_string()));
        }
        
        // ブリッジコントラクトのABIを取得
        let bridge_abi = include_bytes!("../../contracts/ethereum/abi/ShardXBridge.json");
        let bridge_abi = String::from_utf8_lossy(bridge_abi);
        
        // ブリッジコントラクトインスタンスを作成
        let bridge_contract = Contract::from_json(
            web3.eth(),
            bridge_address,
            bridge_abi.as_bytes(),
        ).map_err(|e| Error::ContractError(format!("Failed to create bridge contract instance: {}", e)))?;
        
        // deposit関数を呼び出し
        let deposit_result = bridge_contract.call(
            "deposit",
            (token_address, amount_u256, shardx_recipient),
            wallet.address(),
            Options::default(),
        ).await.map_err(|e| Error::ContractError(format!("Failed to deposit tokens: {}", e)))?;
        
        // トランザクションIDを生成
        let tx_id = uuid::Uuid::new_v4().to_string();
        
        // クロスチェーントランザクションを作成
        let cross_tx = CrossChainTransaction::new(
            Transaction {
                id: tx_id.clone(),
                from: wallet.address().to_string(),
                to: shardx_recipient.to_string(),
                amount: amount.to_string(),
                fee: "0".to_string(),
                data: Some(format!("Transfer from Ethereum: token={}", token.symbol)),
                nonce: 0,
                timestamp: chrono::Utc::now().timestamp() as u64,
                signature: "".to_string(),
                status: crate::transaction::TransactionStatus::Pending,
                shard_id: "shard-1".to_string(),
                block_hash: None,
                block_height: None,
                parent_id: None,
            },
            ChainType::Ethereum,
            ChainType::ShardX,
        );
        
        // メタデータを設定
        let mut updated_tx = cross_tx.clone();
        updated_tx.set_metadata("token_id".to_string(), token_id.to_string());
        updated_tx.set_metadata("token_symbol".to_string(), token.symbol);
        updated_tx.set_metadata("eth_tx_hash".to_string(), deposit_result.to_string());
        
        // 保留中のトランザクションに追加
        {
            let mut pending_transactions = self.pending_transactions.write().unwrap();
            pending_transactions.insert(tx_id.clone(), updated_tx.clone());
        }
        
        info!("Created cross-chain transaction from Ethereum to ShardX: {}", tx_id);
        
        Ok(tx_id)
    }
    
    /// トランザクションの状態を取得
    pub fn get_transaction_status(&self, tx_id: &str) -> Result<TransactionStatus, Error> {
        // 保留中のトランザクションから検索
        {
            let pending_transactions = self.pending_transactions.read().unwrap();
            if let Some(tx) = pending_transactions.get(tx_id) {
                return Ok(tx.status);
            }
        }
        
        // 基本ブリッジから検索
        self.base_bridge.get_transaction_status(tx_id)
    }
    
    /// トランザクションの詳細を取得
    pub fn get_transaction_details(&self, tx_id: &str) -> Result<CrossChainTransaction, Error> {
        // 保留中のトランザクションから検索
        {
            let pending_transactions = self.pending_transactions.read().unwrap();
            if let Some(tx) = pending_transactions.get(tx_id) {
                return Ok(tx.clone());
            }
        }
        
        // 基本ブリッジから検索
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
        // イベントポーリングを停止
        self.stop_event_polling();
        
        // 基本ブリッジを停止
        self.base_bridge.shutdown().await
    }
}