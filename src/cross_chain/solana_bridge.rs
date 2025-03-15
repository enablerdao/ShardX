use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};
use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction as SolanaTransaction,
    instruction::{Instruction, AccountMeta},
    system_instruction,
};
use solana_program::{
    program_pack::Pack,
    system_program,
};
use spl_token::{
    state::{Account as TokenAccount, Mint},
    instruction as token_instruction,
};
use spl_associated_token_account::instruction as associated_token_instruction;

use crate::error::Error;
use crate::transaction::Transaction;
use super::bridge::{CrossChainBridge, BridgeConfig, ChainType, BridgeStatus};
use super::messaging::{CrossChainMessage, MessageType, MessageStatus};
use super::transaction::{CrossChainTransaction, TransactionStatus, TransactionProof};
use super::token_registry::{TokenRegistry, TokenInfo};

/// Solanaブリッジ
pub struct SolanaBridge {
    /// 基本ブリッジ
    base_bridge: CrossChainBridge,
    /// RPCクライアント
    rpc_client: Option<RpcClient>,
    /// ウォレット
    wallet: Option<Keypair>,
    /// ブリッジプログラムID
    program_id: Option<Pubkey>,
    /// トークンレジストリ
    token_registry: Arc<TokenRegistry>,
    /// 進行中のクロスチェーントランザクション
    pending_transactions: RwLock<HashMap<String, CrossChainTransaction>>,
    /// 処理済みのシグネチャ
    processed_signatures: RwLock<HashMap<String, bool>>,
    /// 最後に処理したスロット
    last_processed_slot: RwLock<u64>,
    /// イベントポーリングタスクが実行中かどうか
    polling_active: RwLock<bool>,
}

/// Solanaブリッジ命令
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum SolanaBridgeInstruction {
    /// トークンをデポジット
    Deposit {
        /// 金額
        amount: u64,
        /// ShardX宛先アドレス
        to_shardx_address: String,
    },
    /// トークンを引き出し
    Withdraw {
        /// 金額
        amount: u64,
        /// Solana宛先アドレス
        to_solana_address: Pubkey,
    },
    /// バリデータを追加
    AddValidator {
        /// バリデータのアドレス
        validator: Pubkey,
    },
    /// バリデータを削除
    RemoveValidator {
        /// バリデータのアドレス
        validator: Pubkey,
    },
    /// サポートするトークンを追加
    AddSupportedToken {
        /// トークンのミントアドレス
        mint: Pubkey,
    },
    /// サポートするトークンを削除
    RemoveSupportedToken {
        /// トークンのミントアドレス
        mint: Pubkey,
    },
}

impl SolanaBridge {
    /// 新しいSolanaBridgeを作成
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
            rpc_client: None,
            wallet: None,
            program_id: None,
            token_registry,
            pending_transactions: RwLock::new(HashMap::new()),
            processed_signatures: RwLock::new(HashMap::new()),
            last_processed_slot: RwLock::new(0),
            polling_active: RwLock::new(false),
        }
    }
    
    /// ブリッジを初期化
    pub async fn initialize(&mut self, private_key: &[u8]) -> Result<(), Error> {
        info!("Initializing Solana bridge: {} -> {}", 
            self.base_bridge.get_config().source_chain, 
            self.base_bridge.get_config().target_chain);
        
        // RPCクライアントを初期化
        let endpoint = self.base_bridge.get_config().target_endpoint.clone();
        let rpc_client = RpcClient::new_with_commitment(endpoint, CommitmentConfig::confirmed());
        self.rpc_client = Some(rpc_client.clone());
        
        // ウォレットを初期化
        let wallet = Keypair::from_bytes(private_key)
            .map_err(|e| Error::ValidationError(format!("Invalid private key: {}", e)))?;
        
        self.wallet = Some(wallet.clone());
        
        // 接続テスト
        let slot = rpc_client.get_slot()
            .map_err(|e| Error::ConnectionError(format!("Failed to get slot: {}", e)))?;
        
        *self.last_processed_slot.write().unwrap() = slot;
        
        info!("Connected to Solana network. Latest slot: {}", slot);
        
        // ブリッジプログラムIDを設定
        let program_id = match &self.base_bridge.get_config().target_contract {
            Some(addr) => Pubkey::from_str(addr)
                .map_err(|e| Error::ValidationError(format!("Invalid program ID: {}", e)))?,
            None => return Err(Error::ValidationError("Bridge program ID not specified".to_string())),
        };
        
        self.program_id = Some(program_id);
        
        // 基本ブリッジを初期化
        self.base_bridge.initialize().await?;
        
        // イベントポーリングタスクを開始
        self.start_event_polling().await?;
        
        info!("Solana bridge initialized successfully");
        
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
        
        // RPCクライアントを取得
        let rpc_client = match &self.rpc_client {
            Some(client) => client.clone(),
            None => return Err(Error::ConnectionError("RPC client not initialized".to_string())),
        };
        
        // プログラムIDを取得
        let program_id = match self.program_id {
            Some(id) => id,
            None => return Err(Error::ValidationError("Program ID not initialized".to_string())),
        };
        
        // ポーリング間隔（5秒）
        let mut interval = interval(Duration::from_secs(5));
        
        // ポーリングタスクを開始
        let last_processed_slot = self.last_processed_slot.clone();
        let processed_signatures = self.processed_signatures.clone();
        let pending_transactions = self.pending_transactions.clone();
        let polling_active = self.polling_active.clone();
        
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                // ポーリングが無効化された場合は終了
                {
                    let polling_active_guard = polling_active.read().unwrap();
                    if !*polling_active_guard {
                        break;
                    }
                }
                
                // 最新のスロットを取得
                let current_slot = match rpc_client.get_slot() {
                    Ok(slot) => slot,
                    Err(e) => {
                        error!("Failed to get current slot: {}", e);
                        continue;
                    }
                };
                
                // 最後に処理したスロットを取得
                let last_slot = {
                    let last_slot = last_processed_slot.read().unwrap();
                    *last_slot
                };
                
                // 新しいスロットがない場合はスキップ
                if current_slot <= last_slot {
                    continue;
                }
                
                // プログラムの署名を取得
                let signatures = match rpc_client.get_signatures_for_address(
                    &program_id,
                    Some(last_slot),
                    Some(current_slot),
                    CommitmentConfig::confirmed(),
                ) {
                    Ok(sigs) => sigs,
                    Err(e) => {
                        error!("Failed to get signatures: {}", e);
                        continue;
                    }
                };
                
                // 各署名を処理
                for sig_info in signatures {
                    let signature_str = sig_info.signature.to_string();
                    
                    // 既に処理済みの署名はスキップ
                    {
                        let processed_signatures_guard = processed_signatures.read().unwrap();
                        if processed_signatures_guard.contains_key(&signature_str) {
                            continue;
                        }
                    }
                    
                    // トランザクション情報を取得
                    let tx_info = match rpc_client.get_transaction(
                        &sig_info.signature,
                        CommitmentConfig::confirmed(),
                    ) {
                        Ok(info) => info,
                        Err(e) => {
                            error!("Failed to get transaction info: {}", e);
                            continue;
                        }
                    };
                    
                    // TODO: トランザクションを解析してデポジットイベントを検出
                    // 実際の実装では、トランザクションのログを解析してデポジットイベントを検出する
                    
                    // 処理済みとしてマーク
                    {
                        let mut processed_signatures_guard = processed_signatures.write().unwrap();
                        processed_signatures_guard.insert(signature_str, true);
                    }
                }
                
                // 最後に処理したスロットを更新
                {
                    let mut last_slot_guard = last_processed_slot.write().unwrap();
                    *last_slot_guard = current_slot;
                }
                
                // 保留中のトランザクションを処理
                let tx_ids: Vec<String> = {
                    let pending_transactions_guard = pending_transactions.read().unwrap();
                    pending_transactions_guard.keys().cloned().collect()
                };
                
                for tx_id in tx_ids {
                    // トランザクションを取得
                    let tx = {
                        let pending_transactions_guard = pending_transactions.read().unwrap();
                        match pending_transactions_guard.get(&tx_id) {
                            Some(tx) => tx.clone(),
                            None => continue,
                        }
                    };
                    
                    // Solanaトランザクションシグネチャを取得
                    let signature = match tx.get_metadata("solana_signature") {
                        Some(sig) => sig,
                        None => continue,
                    };
                    
                    // トランザクションの状態を確認
                    let signature_obj = match Signature::from_str(signature) {
                        Ok(sig) => sig,
                        Err(_) => continue,
                    };
                    
                    let status = match rpc_client.get_signature_status(&signature_obj) {
                        Ok(Some(status)) => status,
                        Ok(None) => continue, // まだ処理中
                        Err(_) => continue,
                    };
                    
                    // トランザクションの状態を更新
                    let mut updated_tx = tx.clone();
                    
                    if status.is_ok() {
                        // 成功
                        updated_tx.status = TransactionStatus::Confirmed;
                        
                        // トランザクション情報を取得
                        if let Ok(tx_info) = rpc_client.get_transaction(
                            &signature_obj,
                            CommitmentConfig::confirmed(),
                        ) {
                            // ブロック情報を設定
                            updated_tx.set_target_block_info(
                                tx_info.transaction.signatures[0].to_string(),
                                tx_info.slot,
                            );
                        }
                        
                        // 保留中のトランザクションから削除
                        {
                            let mut pending_transactions_guard = pending_transactions.write().unwrap();
                            pending_transactions_guard.remove(&tx_id);
                        }
                    } else {
                        // 失敗
                        updated_tx.mark_as_failed(format!("Transaction failed: {:?}", status));
                        
                        // 保留中のトランザクションから削除
                        {
                            let mut pending_transactions_guard = pending_transactions.write().unwrap();
                            pending_transactions_guard.remove(&tx_id);
                        }
                    }
                    
                    // TODO: トランザクションの状態を更新
                    // 実際の実装では、基本ブリッジのトランザクションマップを更新する
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
    
    /// ShardXからSolanaへのトークン転送
    pub async fn transfer_to_solana(
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
        
        // トークンがSolanaチェーンのものであることを確認
        if token.chain_type != ChainType::Solana {
            return Err(Error::ValidationError(format!("Token is not on Solana chain: {}", token_id)));
        }
        
        // 受取人アドレスを検証
        let recipient_pubkey = Pubkey::from_str(recipient)
            .map_err(|e| Error::ValidationError(format!("Invalid recipient address: {}", e)))?;
        
        // 金額を検証
        let amount_u64 = amount.parse::<u64>()
            .map_err(|e| Error::ValidationError(format!("Invalid amount: {}", e)))?;
        
        // トークンミントアドレスを取得
        let mint_pubkey = Pubkey::from_str(&token.chain_address)
            .map_err(|e| Error::ValidationError(format!("Invalid token address: {}", e)))?;
        
        // RPCクライアントを取得
        let rpc_client = match &self.rpc_client {
            Some(client) => client,
            None => return Err(Error::ConnectionError("RPC client not initialized".to_string())),
        };
        
        // ウォレットを取得
        let wallet = match &self.wallet {
            Some(wallet) => wallet,
            None => return Err(Error::ValidationError("Wallet not initialized".to_string())),
        };
        
        // プログラムIDを取得
        let program_id = match self.program_id {
            Some(id) => id,
            None => return Err(Error::ValidationError("Program ID not initialized".to_string())),
        };
        
        // ブリッジのトークンアカウントを取得
        let bridge_token_account = spl_associated_token_account::get_associated_token_address(
            &program_id,
            &mint_pubkey,
        );
        
        // 引き出し命令を作成
        let instruction_data = bincode::serialize(&SolanaBridgeInstruction::Withdraw {
            amount: amount_u64,
            to_solana_address: recipient_pubkey,
        }).map_err(|e| Error::SerializationError(e.to_string()))?;
        
        // 命令を作成
        let instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(wallet.pubkey(), true),
                AccountMeta::new(bridge_token_account, false),
                AccountMeta::new(recipient_pubkey, false),
                AccountMeta::new_readonly(mint_pubkey, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        };
        
        // トランザクションを作成
        let blockhash = rpc_client.get_latest_blockhash()
            .map_err(|e| Error::TransactionError(format!("Failed to get blockhash: {}", e)))?;
        
        let transaction = SolanaTransaction::new_signed_with_payer(
            &[instruction],
            Some(&wallet.pubkey()),
            &[wallet],
            blockhash,
        );
        
        // トランザクションを送信
        let signature = rpc_client.send_transaction(&transaction)
            .map_err(|e| Error::TransactionError(format!("Failed to send transaction: {}", e)))?;
        
        // トランザクションIDを生成
        let tx_id = uuid::Uuid::new_v4().to_string();
        
        // クロスチェーントランザクションを作成
        let cross_tx = CrossChainTransaction::new(
            Transaction {
                id: tx_id.clone(),
                from: from_address.to_string(),
                to: recipient.to_string(),
                amount: amount.to_string(),
                fee: "0".to_string(),
                data: Some(format!("Transfer to Solana: token={}, recipient={}", token.symbol, recipient)),
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
            ChainType::Solana,
        );
        
        // メタデータを設定
        let mut updated_tx = cross_tx.clone();
        updated_tx.set_metadata("token_id".to_string(), token_id.to_string());
        updated_tx.set_metadata("token_symbol".to_string(), token.symbol);
        updated_tx.set_metadata("solana_signature".to_string(), signature.to_string());
        
        // 保留中のトランザクションに追加
        {
            let mut pending_transactions = self.pending_transactions.write().unwrap();
            pending_transactions.insert(tx_id.clone(), updated_tx.clone());
        }
        
        info!("Created cross-chain transaction from ShardX to Solana: {}", tx_id);
        
        Ok(tx_id)
    }
    
    /// SolanaからShardXへのトークン転送
    pub async fn transfer_from_solana(
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
        
        // トークンがSolanaチェーンのものであることを確認
        if token.chain_type != ChainType::Solana {
            return Err(Error::ValidationError(format!("Token is not on Solana chain: {}", token_id)));
        }
        
        // ShardX受取人アドレスを検証
        if shardx_recipient.is_empty() {
            return Err(Error::ValidationError("Invalid ShardX recipient address".to_string()));
        }
        
        // 金額を検証
        let amount_u64 = amount.parse::<u64>()
            .map_err(|e| Error::ValidationError(format!("Invalid amount: {}", e)))?;
        
        // トークンミントアドレスを取得
        let mint_pubkey = Pubkey::from_str(&token.chain_address)
            .map_err(|e| Error::ValidationError(format!("Invalid token address: {}", e)))?;
        
        // RPCクライアントを取得
        let rpc_client = match &self.rpc_client {
            Some(client) => client,
            None => return Err(Error::ConnectionError("RPC client not initialized".to_string())),
        };
        
        // ウォレットを取得
        let wallet = match &self.wallet {
            Some(wallet) => wallet,
            None => return Err(Error::ValidationError("Wallet not initialized".to_string())),
        };
        
        // プログラムIDを取得
        let program_id = match self.program_id {
            Some(id) => id,
            None => return Err(Error::ValidationError("Program ID not initialized".to_string())),
        };
        
        // 送信者のトークンアカウントを取得
        let sender_token_account = spl_associated_token_account::get_associated_token_address(
            &wallet.pubkey(),
            &mint_pubkey,
        );
        
        // ブリッジのトークンアカウントを取得
        let bridge_token_account = spl_associated_token_account::get_associated_token_address(
            &program_id,
            &mint_pubkey,
        );
        
        // ブリッジのトークンアカウントが存在するか確認
        let bridge_account_exists = rpc_client.get_account_data(&bridge_token_account).is_ok();
        
        // ブリッジのトークンアカウントが存在しない場合は作成
        let mut instructions = Vec::new();
        
        if !bridge_account_exists {
            instructions.push(
                associated_token_instruction::create_associated_token_account(
                    &wallet.pubkey(),
                    &program_id,
                    &mint_pubkey,
                ),
            );
        }
        
        // デポジット命令を作成
        let instruction_data = bincode::serialize(&SolanaBridgeInstruction::Deposit {
            amount: amount_u64,
            to_shardx_address: shardx_recipient.to_string(),
        }).map_err(|e| Error::SerializationError(e.to_string()))?;
        
        // トークン転送命令を作成
        instructions.push(
            token_instruction::transfer(
                &spl_token::id(),
                &sender_token_account,
                &bridge_token_account,
                &wallet.pubkey(),
                &[&wallet.pubkey()],
                amount_u64,
            ).map_err(|e| Error::TransactionError(format!("Failed to create transfer instruction: {}", e)))?
        );
        
        // デポジット命令を作成
        instructions.push(
            Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(wallet.pubkey(), true),
                    AccountMeta::new(sender_token_account, false),
                    AccountMeta::new(bridge_token_account, false),
                    AccountMeta::new_readonly(mint_pubkey, false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                ],
                data: instruction_data,
            }
        );
        
        // トランザクションを作成
        let blockhash = rpc_client.get_latest_blockhash()
            .map_err(|e| Error::TransactionError(format!("Failed to get blockhash: {}", e)))?;
        
        let transaction = SolanaTransaction::new_signed_with_payer(
            &instructions,
            Some(&wallet.pubkey()),
            &[wallet],
            blockhash,
        );
        
        // トランザクションを送信
        let signature = rpc_client.send_transaction(&transaction)
            .map_err(|e| Error::TransactionError(format!("Failed to send transaction: {}", e)))?;
        
        // トランザクションIDを生成
        let tx_id = uuid::Uuid::new_v4().to_string();
        
        // クロスチェーントランザクションを作成
        let cross_tx = CrossChainTransaction::new(
            Transaction {
                id: tx_id.clone(),
                from: wallet.pubkey().to_string(),
                to: shardx_recipient.to_string(),
                amount: amount.to_string(),
                fee: "0".to_string(),
                data: Some(format!("Transfer from Solana: token={}", token.symbol)),
                nonce: 0,
                timestamp: chrono::Utc::now().timestamp() as u64,
                signature: "".to_string(),
                status: crate::transaction::TransactionStatus::Pending,
                shard_id: "shard-1".to_string(),
                block_hash: None,
                block_height: None,
                parent_id: None,
            },
            ChainType::Solana,
            ChainType::ShardX,
        );
        
        // メタデータを設定
        let mut updated_tx = cross_tx.clone();
        updated_tx.set_metadata("token_id".to_string(), token_id.to_string());
        updated_tx.set_metadata("token_symbol".to_string(), token.symbol);
        updated_tx.set_metadata("solana_signature".to_string(), signature.to_string());
        
        // 保留中のトランザクションに追加
        {
            let mut pending_transactions = self.pending_transactions.write().unwrap();
            pending_transactions.insert(tx_id.clone(), updated_tx.clone());
        }
        
        info!("Created cross-chain transaction from Solana to ShardX: {}", tx_id);
        
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