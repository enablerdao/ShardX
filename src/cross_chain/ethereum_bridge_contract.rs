use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tokio::sync::mpsc;
use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};
use web3::{
    Web3,
    transports::Http,
    types::{Address, H256, U256, TransactionRequest, BlockNumber, Log, FilterBuilder},
    contract::{Contract, Options},
};
use ethers::{
    prelude::*,
    providers::{Provider, Http as EthersHttp},
    signers::{LocalWallet, Signer},
};

use crate::error::Error;
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use super::bridge::{CrossChainBridge, BridgeConfig, ChainType, BridgeStatus};
use super::messaging::{CrossChainMessage, MessageType, MessageStatus};
use super::transaction::{CrossChainTransaction, TransactionStatus, TransactionProof};

/// イーサリアムブリッジコントラクト連携
pub struct EthereumBridgeContract {
    /// コントラクトアドレス
    contract_address: Address,
    /// Web3クライアント
    web3: Web3<Http>,
    /// コントラクトインスタンス
    contract: Option<Contract<Http>>,
    /// バリデータウォレット
    validator_wallet: LocalWallet,
    /// サポートされているトークン
    supported_tokens: RwLock<HashMap<Address, String>>,
    /// 処理済みデポジットID
    processed_deposits: RwLock<HashMap<H256, bool>>,
    /// 処理済み引き出しID
    processed_withdrawals: RwLock<HashMap<H256, bool>>,
    /// 最後に処理したブロック番号
    last_processed_block: RwLock<u64>,
}

/// デポジットイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositEvent {
    /// トークンアドレス
    pub token: Address,
    /// 送信元アドレス
    pub from: Address,
    /// ShardX宛先アドレス
    pub to_shardx_address: String,
    /// 金額
    pub amount: U256,
    /// デポジットID
    pub deposit_id: H256,
}

/// 引き出しイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalEvent {
    /// トークンアドレス
    pub token: Address,
    /// 送信先アドレス
    pub to: Address,
    /// 金額
    pub amount: U256,
    /// 引き出しID
    pub withdrawal_id: H256,
}

impl EthereumBridgeContract {
    /// 新しいEthereumBridgeContractを作成
    pub fn new(
        contract_address: Address,
        web3: Web3<Http>,
        validator_wallet: LocalWallet,
    ) -> Self {
        Self {
            contract_address,
            web3: web3.clone(),
            contract: None,
            validator_wallet,
            supported_tokens: RwLock::new(HashMap::new()),
            processed_deposits: RwLock::new(HashMap::new()),
            processed_withdrawals: RwLock::new(HashMap::new()),
            last_processed_block: RwLock::new(0),
        }
    }

    /// コントラクトを初期化
    pub async fn initialize(&mut self) -> Result<(), Error> {
        // コントラクトABIを読み込み
        let contract_abi = include_bytes!("../../contracts/ethereum/abi/ShardXBridge.json");
        let contract_abi = String::from_utf8_lossy(contract_abi);
        
        // コントラクトインスタンスを作成
        let contract = Contract::from_json(
            self.web3.eth(),
            self.contract_address,
            contract_abi.as_bytes(),
        ).map_err(|e| Error::ContractError(format!("Failed to create contract instance: {}", e)))?;
        
        self.contract = Some(contract);
        
        // サポートされているトークンを取得
        self.update_supported_tokens().await?;
        
        // 最新のブロック番号を取得
        let latest_block = self.web3.eth().block_number().await
            .map_err(|e| Error::ConnectionError(format!("Failed to get latest block number: {}", e)))?;
        
        // 最後に処理したブロック番号を更新（最新のブロック番号から1000ブロック前）
        let start_block = latest_block.as_u64().saturating_sub(1000);
        *self.last_processed_block.write().unwrap() = start_block;
        
        info!("Ethereum bridge contract initialized. Contract address: {}, Starting from block: {}", 
            self.contract_address, start_block);
        
        Ok(())
    }
    
    /// サポートされているトークンを更新
    async fn update_supported_tokens(&self) -> Result<(), Error> {
        // 実際の実装では、コントラクトからサポートされているトークンのリストを取得
        // ここでは簡略化のため、ハードコードしたトークンを使用
        
        let mut tokens = self.supported_tokens.write().unwrap();
        
        // ETH（ラップドイーサ）
        let weth_address = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
            .map_err(|e| Error::ValidationError(format!("Invalid address: {}", e)))?;
        tokens.insert(weth_address, "WETH".to_string());
        
        // USDC
        let usdc_address = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .map_err(|e| Error::ValidationError(format!("Invalid address: {}", e)))?;
        tokens.insert(usdc_address, "USDC".to_string());
        
        // USDT
        let usdt_address = Address::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7")
            .map_err(|e| Error::ValidationError(format!("Invalid address: {}", e)))?;
        tokens.insert(usdt_address, "USDT".to_string());
        
        info!("Updated supported tokens. Count: {}", tokens.len());
        
        Ok(())
    }
    
    /// イベントをポーリング
    pub async fn poll_events(&self) -> Result<(), Error> {
        // 最後に処理したブロック番号を取得
        let last_block = *self.last_processed_block.read().unwrap();
        
        // 最新のブロック番号を取得
        let latest_block = self.web3.eth().block_number().await
            .map_err(|e| Error::ConnectionError(format!("Failed to get latest block number: {}", e)))?;
        
        let latest_block = latest_block.as_u64();
        
        // 処理するブロック範囲を決定（最大1000ブロック）
        let from_block = last_block + 1;
        let to_block = std::cmp::min(latest_block, from_block + 999);
        
        // 新しいブロックがない場合は終了
        if from_block > to_block {
            return Ok(());
        }
        
        info!("Polling events from block {} to {}", from_block, to_block);
        
        // デポジットイベントをポーリング
        self.poll_deposit_events(from_block, to_block).await?;
        
        // 引き出しイベントをポーリング
        self.poll_withdrawal_events(from_block, to_block).await?;
        
        // 最後に処理したブロック番号を更新
        *self.last_processed_block.write().unwrap() = to_block;
        
        Ok(())
    }
    
    /// デポジットイベントをポーリング
    async fn poll_deposit_events(&self, from_block: u64, to_block: u64) -> Result<(), Error> {
        let contract = self.contract.as_ref()
            .ok_or_else(|| Error::ContractError("Contract not initialized".to_string()))?;
        
        // デポジットイベントのフィルタを作成
        let filter = FilterBuilder::default()
            .address(vec![self.contract_address])
            .from_block(BlockNumber::Number(from_block.into()))
            .to_block(BlockNumber::Number(to_block.into()))
            .topics(
                Some(vec![H256::from_str("0x5548c837ab068cf56a2c2479df0882a4922fd203edb7517321831d95078c5f62").unwrap()]),
                None,
                None,
                None,
            )
            .build();
        
        // イベントを取得
        let logs = self.web3.eth().logs(filter).await
            .map_err(|e| Error::ContractError(format!("Failed to get deposit logs: {}", e)))?;
        
        info!("Found {} deposit events", logs.len());
        
        // 各イベントを処理
        for log in logs {
            self.process_deposit_event(log).await?;
        }
        
        Ok(())
    }
    
    /// デポジットイベントを処理
    async fn process_deposit_event(&self, log: Log) -> Result<(), Error> {
        // イベントをデコード
        let deposit_id = log.topics.get(3)
            .ok_or_else(|| Error::ContractError("Invalid deposit event format".to_string()))?;
        
        // 既に処理済みのイベントはスキップ
        {
            let processed_deposits = self.processed_deposits.read().unwrap();
            if processed_deposits.contains_key(deposit_id) {
                debug!("Deposit already processed: {:?}", deposit_id);
                return Ok(());
            }
        }
        
        // イベントデータをデコード
        let token = Address::from_slice(&log.topics[1][12..]);
        let from = Address::from_slice(&log.topics[2][12..]);
        
        // データフィールドをデコード
        let data = log.data.0.clone();
        if data.len() < 128 {
            return Err(Error::ContractError("Invalid deposit event data".to_string()));
        }
        
        // ShardXアドレスを取得（文字列）
        let to_shardx_address_offset = U256::from_big_endian(&data[0..32]).as_usize();
        let to_shardx_address_length = U256::from_big_endian(&data[to_shardx_address_offset..to_shardx_address_offset+32]).as_usize();
        let to_shardx_address_start = to_shardx_address_offset + 32;
        let to_shardx_address_end = to_shardx_address_start + to_shardx_address_length;
        
        if to_shardx_address_end > data.len() {
            return Err(Error::ContractError("Invalid deposit event data".to_string()));
        }
        
        let to_shardx_address = String::from_utf8(data[to_shardx_address_start..to_shardx_address_end].to_vec())
            .map_err(|e| Error::ContractError(format!("Invalid ShardX address: {}", e)))?;
        
        // 金額を取得
        let amount = U256::from_big_endian(&data[32..64]);
        
        info!("Processing deposit: token={}, from={}, to={}, amount={}, id={:?}", 
            token, from, to_shardx_address, amount, deposit_id);
        
        // ShardXでのトランザクションを作成
        self.create_shardx_deposit_transaction(token, from, to_shardx_address, amount, *deposit_id).await?;
        
        // 処理済みとしてマーク
        {
            let mut processed_deposits = self.processed_deposits.write().unwrap();
            processed_deposits.insert(*deposit_id, true);
        }
        
        Ok(())
    }
    
    /// ShardXでのデポジットトランザクションを作成
    async fn create_shardx_deposit_transaction(
        &self,
        token: Address,
        from: Address,
        to_shardx_address: String,
        amount: U256,
        deposit_id: H256,
    ) -> Result<(), Error> {
        // トークンシンボルを取得
        let token_symbol = {
            let supported_tokens = self.supported_tokens.read().unwrap();
            supported_tokens.get(&token)
                .cloned()
                .unwrap_or_else(|| "UNKNOWN".to_string())
        };
        
        // ShardXトランザクションを作成
        let transaction = Transaction {
            id: format!("eth-deposit-{:?}", deposit_id),
            from: format!("eth:{}", from),
            to: to_shardx_address,
            amount: amount.to_string(),
            fee: "0".to_string(),
            data: Some(format!("Ethereum deposit: token={}, from={}, amount={} {}", 
                token, from, amount, token_symbol)),
            nonce: 0,
            timestamp: chrono::Utc::now().timestamp() as u64,
            signature: format!("eth-bridge:{:?}", deposit_id),
            status: crate::transaction::TransactionStatus::Pending,
            shard_id: "shard-1".to_string(),
            block_hash: None,
            block_height: None,
            parent_id: None,
        };
        
        // TODO: ShardXトランザクションを実行
        info!("Created ShardX transaction for Ethereum deposit: {}", transaction.id);
        
        Ok(())
    }
    
    /// 引き出しイベントをポーリング
    async fn poll_withdrawal_events(&self, from_block: u64, to_block: u64) -> Result<(), Error> {
        let contract = self.contract.as_ref()
            .ok_or_else(|| Error::ContractError("Contract not initialized".to_string()))?;
        
        // 引き出しイベントのフィルタを作成
        let filter = FilterBuilder::default()
            .address(vec![self.contract_address])
            .from_block(BlockNumber::Number(from_block.into()))
            .to_block(BlockNumber::Number(to_block.into()))
            .topics(
                Some(vec![H256::from_str("0x9b1bfa7fa9ee420a16e124f794c35ac9f90472acc99140eb2f6447c714cad8eb").unwrap()]),
                None,
                None,
                None,
            )
            .build();
        
        // イベントを取得
        let logs = self.web3.eth().logs(filter).await
            .map_err(|e| Error::ContractError(format!("Failed to get withdrawal logs: {}", e)))?;
        
        info!("Found {} withdrawal events", logs.len());
        
        // 各イベントを処理
        for log in logs {
            self.process_withdrawal_event(log).await?;
        }
        
        Ok(())
    }
    
    /// 引き出しイベントを処理
    async fn process_withdrawal_event(&self, log: Log) -> Result<(), Error> {
        // イベントをデコード
        let withdrawal_id = log.topics.get(3)
            .ok_or_else(|| Error::ContractError("Invalid withdrawal event format".to_string()))?;
        
        // 既に処理済みのイベントはスキップ
        {
            let processed_withdrawals = self.processed_withdrawals.read().unwrap();
            if processed_withdrawals.contains_key(withdrawal_id) {
                debug!("Withdrawal already processed: {:?}", withdrawal_id);
                return Ok(());
            }
        }
        
        // イベントデータをデコード
        let token = Address::from_slice(&log.topics[1][12..]);
        let to = Address::from_slice(&log.topics[2][12..]);
        
        // 金額を取得
        let amount = U256::from_big_endian(&log.data.0[0..32]);
        
        info!("Processing withdrawal: token={}, to={}, amount={}, id={:?}", 
            token, to, amount, withdrawal_id);
        
        // 処理済みとしてマーク
        {
            let mut processed_withdrawals = self.processed_withdrawals.write().unwrap();
            processed_withdrawals.insert(*withdrawal_id, true);
        }
        
        Ok(())
    }
    
    /// ShardXからイーサリアムへの引き出しリクエストを作成
    pub async fn request_withdrawal(
        &self,
        token: Address,
        recipient: Address,
        amount: U256,
        shardx_tx_id: &str,
    ) -> Result<H256, Error> {
        let contract = self.contract.as_ref()
            .ok_or_else(|| Error::ContractError("Contract not initialized".to_string()))?;
        
        // 引き出しIDを生成
        let withdrawal_id = H256::from_slice(&keccak256(
            format!("{}:{}:{}:{}", token, recipient, amount, shardx_tx_id).as_bytes()
        ));
        
        // コントラクトメソッドを呼び出し
        let result = contract.call(
            "requestWithdrawal",
            (withdrawal_id, token, recipient, amount),
            self.validator_wallet.address(),
            Options::default(),
        ).await.map_err(|e| Error::ContractError(format!("Failed to request withdrawal: {}", e)))?;
        
        info!("Requested withdrawal: token={}, recipient={}, amount={}, id={:?}", 
            token, recipient, amount, withdrawal_id);
        
        Ok(withdrawal_id)
    }
    
    /// サポートされているトークンかどうかを確認
    pub fn is_token_supported(&self, token: Address) -> bool {
        let supported_tokens = self.supported_tokens.read().unwrap();
        supported_tokens.contains_key(&token)
    }
    
    /// トークンシンボルを取得
    pub fn get_token_symbol(&self, token: Address) -> Option<String> {
        let supported_tokens = self.supported_tokens.read().unwrap();
        supported_tokens.get(&token).cloned()
    }
}

/// keccak256ハッシュを計算
fn keccak256(data: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(data);
    hasher.finalize(&mut output);
    output
}