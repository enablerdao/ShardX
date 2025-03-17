// クロスチェーン相互運用性モジュール
//
// このモジュールは、ShardXと他のブロックチェーンネットワークとの相互運用性を提供します。
// 主な機能:
// - クロスチェーンブリッジ
// - アセットラッピング
// - メッセージパッシング
// - 状態証明
// - 相互運用性プロトコル

mod bridge;
mod wrapped_assets;
mod message_passing;
mod state_proof;
mod protocols;

pub use self::bridge::{Bridge, BridgeConfig, BridgeTransaction, BridgeEvent};
pub use self::wrapped_assets::{WrappedAsset, AssetRegistry, AssetMapping};
pub use self::message_passing::{MessageRouter, Message, MessageVerifier};
pub use self::state_proof::{StateProof, ProofVerifier, MerkleProof};
pub use self::protocols::{InteropProtocol, ProtocolAdapter, ProtocolRegistry};

use crate::error::Error;
use crate::crypto::hash::Hash;
use crate::transaction::Transaction;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use log::{debug, error, info, warn};

/// サポートされているチェーン
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChainType {
    /// イーサリアム
    Ethereum,
    /// ビットコイン
    Bitcoin,
    /// ポルカドット
    Polkadot,
    /// コスモス
    Cosmos,
    /// ソラナ
    Solana,
    /// アバランチ
    Avalanche,
    /// カスタム
    Custom(String),
}

/// チェーン設定
#[derive(Debug, Clone)]
pub struct ChainConfig {
    /// チェーンタイプ
    pub chain_type: ChainType,
    /// チェーンID
    pub chain_id: String,
    /// エンドポイントURL
    pub endpoint_url: String,
    /// コントラクトアドレス
    pub contract_address: Option<String>,
    /// 確認ブロック数
    pub confirmation_blocks: u64,
    /// タイムアウト（ミリ秒）
    pub timeout_ms: u64,
}

/// 相互運用性マネージャー
pub struct InteroperabilityManager {
    /// ブリッジ
    bridges: HashMap<(ChainType, ChainType), Arc<Bridge>>,
    /// アセットレジストリ
    asset_registry: Arc<Mutex<AssetRegistry>>,
    /// メッセージルーター
    message_router: Arc<MessageRouter>,
    /// プロトコルレジストリ
    protocol_registry: Arc<Mutex<ProtocolRegistry>>,
    /// チェーン設定
    chain_configs: HashMap<ChainType, ChainConfig>,
}

impl InteroperabilityManager {
    /// 新しいInteroperabilityManagerを作成
    pub fn new() -> Self {
        Self {
            bridges: HashMap::new(),
            asset_registry: Arc::new(Mutex::new(AssetRegistry::new())),
            message_router: Arc::new(MessageRouter::new()),
            protocol_registry: Arc::new(Mutex::new(ProtocolRegistry::new())),
            chain_configs: HashMap::new(),
        }
    }
    
    /// チェーン設定を追加
    pub fn add_chain_config(&mut self, config: ChainConfig) {
        self.chain_configs.insert(config.chain_type.clone(), config);
    }
    
    /// チェーン設定を取得
    pub fn get_chain_config(&self, chain_type: &ChainType) -> Option<&ChainConfig> {
        self.chain_configs.get(chain_type)
    }
    
    /// ブリッジを作成
    pub fn create_bridge(
        &mut self,
        source_chain: ChainType,
        target_chain: ChainType,
        config: BridgeConfig,
    ) -> Result<Arc<Bridge>, Error> {
        // ソースチェーンの設定を取得
        let source_config = self.chain_configs.get(&source_chain)
            .ok_or_else(|| Error::InvalidArgument(format!("Source chain config not found: {:?}", source_chain)))?;
        
        // ターゲットチェーンの設定を取得
        let target_config = self.chain_configs.get(&target_chain)
            .ok_or_else(|| Error::InvalidArgument(format!("Target chain config not found: {:?}", target_chain)))?;
        
        // ブリッジを作成
        let bridge = Arc::new(Bridge::new(
            source_chain.clone(),
            target_chain.clone(),
            source_config.clone(),
            target_config.clone(),
            config,
        )?);
        
        // ブリッジを登録
        self.bridges.insert((source_chain, target_chain), bridge.clone());
        
        Ok(bridge)
    }
    
    /// ブリッジを取得
    pub fn get_bridge(
        &self,
        source_chain: &ChainType,
        target_chain: &ChainType,
    ) -> Option<Arc<Bridge>> {
        self.bridges.get(&(source_chain.clone(), target_chain.clone())).cloned()
    }
    
    /// アセットをラップ
    pub async fn wrap_asset(
        &self,
        source_chain: &ChainType,
        target_chain: &ChainType,
        asset_id: &str,
        amount: u64,
        sender: &str,
        recipient: &str,
    ) -> Result<String, Error> {
        // ブリッジを取得
        let bridge = self.get_bridge(source_chain, target_chain)
            .ok_or_else(|| Error::NotFound(format!("Bridge not found: {:?} -> {:?}", source_chain, target_chain)))?;
        
        // アセットマッピングを取得
        let asset_registry = self.asset_registry.lock().unwrap();
        let asset_mapping = asset_registry.get_mapping(source_chain, target_chain, asset_id)
            .ok_or_else(|| Error::NotFound(format!("Asset mapping not found: {}", asset_id)))?;
        
        // アセットをラップ
        let tx_id = bridge.wrap_asset(asset_mapping, amount, sender, recipient).await?;
        
        Ok(tx_id)
    }
    
    /// アセットをアンラップ
    pub async fn unwrap_asset(
        &self,
        source_chain: &ChainType,
        target_chain: &ChainType,
        asset_id: &str,
        amount: u64,
        sender: &str,
        recipient: &str,
    ) -> Result<String, Error> {
        // ブリッジを取得
        let bridge = self.get_bridge(source_chain, target_chain)
            .ok_or_else(|| Error::NotFound(format!("Bridge not found: {:?} -> {:?}", source_chain, target_chain)))?;
        
        // アセットマッピングを取得
        let asset_registry = self.asset_registry.lock().unwrap();
        let asset_mapping = asset_registry.get_mapping(source_chain, target_chain, asset_id)
            .ok_or_else(|| Error::NotFound(format!("Asset mapping not found: {}", asset_id)))?;
        
        // アセットをアンラップ
        let tx_id = bridge.unwrap_asset(asset_mapping, amount, sender, recipient).await?;
        
        Ok(tx_id)
    }
    
    /// クロスチェーンメッセージを送信
    pub async fn send_message(
        &self,
        source_chain: &ChainType,
        target_chain: &ChainType,
        message: &[u8],
        sender: &str,
        recipient: &str,
    ) -> Result<String, Error> {
        // ブリッジを取得
        let bridge = self.get_bridge(source_chain, target_chain)
            .ok_or_else(|| Error::NotFound(format!("Bridge not found: {:?} -> {:?}", source_chain, target_chain)))?;
        
        // メッセージを作成
        let cross_chain_message = Message {
            source_chain: source_chain.clone(),
            target_chain: target_chain.clone(),
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            payload: message.to_vec(),
            nonce: rand::random::<u64>(),
            timestamp: chrono::Utc::now(),
        };
        
        // メッセージを送信
        let tx_id = self.message_router.send_message(&cross_chain_message, bridge.as_ref()).await?;
        
        Ok(tx_id)
    }
    
    /// クロスチェーンメッセージを受信
    pub async fn receive_message(
        &self,
        source_chain: &ChainType,
        target_chain: &ChainType,
        message_id: &str,
    ) -> Result<Message, Error> {
        // ブリッジを取得
        let bridge = self.get_bridge(source_chain, target_chain)
            .ok_or_else(|| Error::NotFound(format!("Bridge not found: {:?} -> {:?}", source_chain, target_chain)))?;
        
        // メッセージを受信
        let message = self.message_router.receive_message(message_id, bridge.as_ref()).await?;
        
        Ok(message)
    }
    
    /// 状態証明を生成
    pub async fn generate_state_proof(
        &self,
        chain_type: &ChainType,
        block_id: &str,
        key: &[u8],
    ) -> Result<StateProof, Error> {
        // チェーン設定を取得
        let chain_config = self.chain_configs.get(chain_type)
            .ok_or_else(|| Error::InvalidArgument(format!("Chain config not found: {:?}", chain_type)))?;
        
        // プロトコルアダプターを取得
        let protocol_registry = self.protocol_registry.lock().unwrap();
        let adapter = protocol_registry.get_adapter(chain_type)
            .ok_or_else(|| Error::NotFound(format!("Protocol adapter not found: {:?}", chain_type)))?;
        
        // 状態証明を生成
        let proof = adapter.generate_state_proof(chain_config, block_id, key).await?;
        
        Ok(proof)
    }
    
    /// 状態証明を検証
    pub fn verify_state_proof(
        &self,
        chain_type: &ChainType,
        proof: &StateProof,
    ) -> Result<bool, Error> {
        // プロトコルアダプターを取得
        let protocol_registry = self.protocol_registry.lock().unwrap();
        let adapter = protocol_registry.get_adapter(chain_type)
            .ok_or_else(|| Error::NotFound(format!("Protocol adapter not found: {:?}", chain_type)))?;
        
        // 状態証明を検証
        let result = adapter.verify_state_proof(proof)?;
        
        Ok(result)
    }
    
    /// アセットマッピングを登録
    pub fn register_asset_mapping(
        &self,
        source_chain: &ChainType,
        target_chain: &ChainType,
        source_asset_id: &str,
        target_asset_id: &str,
    ) -> Result<(), Error> {
        let mut asset_registry = self.asset_registry.lock().unwrap();
        
        asset_registry.register_mapping(
            source_chain.clone(),
            target_chain.clone(),
            source_asset_id.to_string(),
            target_asset_id.to_string(),
        )?;
        
        Ok(())
    }
    
    /// プロトコルアダプターを登録
    pub fn register_protocol_adapter(
        &self,
        chain_type: &ChainType,
        adapter: Box<dyn ProtocolAdapter>,
    ) -> Result<(), Error> {
        let mut protocol_registry = self.protocol_registry.lock().unwrap();
        
        protocol_registry.register_adapter(chain_type.clone(), adapter)?;
        
        Ok(())
    }
    
    /// ブリッジイベントを監視
    pub async fn monitor_bridge_events(
        &self,
        source_chain: &ChainType,
        target_chain: &ChainType,
        callback: Box<dyn Fn(BridgeEvent) -> Result<(), Error> + Send + Sync>,
    ) -> Result<String, Error> {
        // ブリッジを取得
        let bridge = self.get_bridge(source_chain, target_chain)
            .ok_or_else(|| Error::NotFound(format!("Bridge not found: {:?} -> {:?}", source_chain, target_chain)))?;
        
        // イベントを監視
        let monitor_id = bridge.monitor_events(callback).await?;
        
        Ok(monitor_id)
    }
    
    /// ブリッジイベント監視を停止
    pub async fn stop_monitoring_bridge_events(
        &self,
        source_chain: &ChainType,
        target_chain: &ChainType,
        monitor_id: &str,
    ) -> Result<(), Error> {
        // ブリッジを取得
        let bridge = self.get_bridge(source_chain, target_chain)
            .ok_or_else(|| Error::NotFound(format!("Bridge not found: {:?} -> {:?}", source_chain, target_chain)))?;
        
        // イベント監視を停止
        bridge.stop_monitoring(monitor_id).await?;
        
        Ok(())
    }
    
    /// サポートされているチェーンを取得
    pub fn get_supported_chains(&self) -> Vec<ChainType> {
        self.chain_configs.keys().cloned().collect()
    }
    
    /// サポートされているブリッジを取得
    pub fn get_supported_bridges(&self) -> Vec<(ChainType, ChainType)> {
        self.bridges.keys().cloned().collect()
    }
}

impl Default for InteroperabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chain_config() {
        let mut manager = InteroperabilityManager::new();
        
        // チェーン設定を追加
        let config = ChainConfig {
            chain_type: ChainType::Ethereum,
            chain_id: "1".to_string(),
            endpoint_url: "https://mainnet.infura.io/v3/your-api-key".to_string(),
            contract_address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
            confirmation_blocks: 12,
            timeout_ms: 30000,
        };
        
        manager.add_chain_config(config);
        
        // チェーン設定を取得
        let retrieved_config = manager.get_chain_config(&ChainType::Ethereum);
        assert!(retrieved_config.is_some());
        
        let config = retrieved_config.unwrap();
        assert_eq!(config.chain_id, "1");
        assert_eq!(config.confirmation_blocks, 12);
    }
    
    #[test]
    fn test_supported_chains() {
        let mut manager = InteroperabilityManager::new();
        
        // チェーン設定を追加
        let ethereum_config = ChainConfig {
            chain_type: ChainType::Ethereum,
            chain_id: "1".to_string(),
            endpoint_url: "https://mainnet.infura.io/v3/your-api-key".to_string(),
            contract_address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
            confirmation_blocks: 12,
            timeout_ms: 30000,
        };
        
        let bitcoin_config = ChainConfig {
            chain_type: ChainType::Bitcoin,
            chain_id: "mainnet".to_string(),
            endpoint_url: "https://btc.example.com/api".to_string(),
            contract_address: None,
            confirmation_blocks: 6,
            timeout_ms: 30000,
        };
        
        manager.add_chain_config(ethereum_config);
        manager.add_chain_config(bitcoin_config);
        
        // サポートされているチェーンを取得
        let supported_chains = manager.get_supported_chains();
        assert_eq!(supported_chains.len(), 2);
        assert!(supported_chains.contains(&ChainType::Ethereum));
        assert!(supported_chains.contains(&ChainType::Bitcoin));
    }
}