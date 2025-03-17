use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::bridge::ChainType;
use crate::error::Error;

/// トークン情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// トークンID
    pub id: String,
    /// トークン名
    pub name: String,
    /// トークンシンボル
    pub symbol: String,
    /// 小数点以下の桁数
    pub decimals: u8,
    /// チェーンタイプ
    pub chain_type: ChainType,
    /// チェーン固有のアドレス/識別子
    pub chain_address: String,
    /// 総供給量
    pub total_supply: Option<String>,
    /// アイコンURL
    pub icon_url: Option<String>,
    /// プロジェクトURL
    pub project_url: Option<String>,
    /// 説明
    pub description: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

impl TokenInfo {
    /// 新しいトークン情報を作成
    pub fn new(
        id: String,
        name: String,
        symbol: String,
        decimals: u8,
        chain_type: ChainType,
        chain_address: String,
    ) -> Self {
        Self {
            id,
            name,
            symbol,
            decimals,
            chain_type,
            chain_address,
            total_supply: None,
            icon_url: None,
            project_url: None,
            description: None,
            metadata: HashMap::new(),
        }
    }

    /// メタデータを設定
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// メタデータを取得
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// トークンレジストリ
pub struct TokenRegistry {
    /// トークンマップ (トークンID -> トークン情報)
    tokens: RwLock<HashMap<String, TokenInfo>>,
    /// チェーンアドレスマップ (チェーンタイプ, チェーンアドレス) -> トークンID
    chain_address_map: RwLock<HashMap<(ChainType, String), String>>,
    /// シンボルマップ (シンボル -> トークンID)
    symbol_map: RwLock<HashMap<String, Vec<String>>>,
}

impl TokenRegistry {
    /// 新しいトークンレジストリを作成
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
            chain_address_map: RwLock::new(HashMap::new()),
            symbol_map: RwLock::new(HashMap::new()),
        }
    }

    /// トークンを登録
    pub fn register_token(&self, token: TokenInfo) -> Result<(), Error> {
        let token_id = token.id.clone();
        let chain_type = token.chain_type;
        let chain_address = token.chain_address.clone();
        let symbol = token.symbol.clone();

        // トークンを追加
        {
            let mut tokens = self.tokens.write().unwrap();
            if tokens.contains_key(&token_id) {
                return Err(Error::ValidationError(format!(
                    "Token already registered: {}",
                    token_id
                )));
            }
            tokens.insert(token_id.clone(), token);
        }

        // チェーンアドレスマップを更新
        {
            let mut chain_address_map = self.chain_address_map.write().unwrap();
            chain_address_map.insert((chain_type, chain_address), token_id.clone());
        }

        // シンボルマップを更新
        {
            let mut symbol_map = self.symbol_map.write().unwrap();
            let token_ids = symbol_map.entry(symbol).or_insert_with(Vec::new);
            token_ids.push(token_id);
        }

        Ok(())
    }

    /// トークンを取得
    pub fn get_token(&self, token_id: &str) -> Option<TokenInfo> {
        let tokens = self.tokens.read().unwrap();
        tokens.get(token_id).cloned()
    }

    /// チェーンアドレスからトークンを取得
    pub fn get_token_by_chain_address(
        &self,
        chain_type: ChainType,
        chain_address: &str,
    ) -> Option<TokenInfo> {
        let chain_address_map = self.chain_address_map.read().unwrap();
        let token_id = chain_address_map.get(&(chain_type, chain_address.to_string()))?;

        let tokens = self.tokens.read().unwrap();
        tokens.get(token_id).cloned()
    }

    /// シンボルからトークンを取得
    pub fn get_tokens_by_symbol(&self, symbol: &str) -> Vec<TokenInfo> {
        let symbol_map = self.symbol_map.read().unwrap();
        let token_ids = match symbol_map.get(symbol) {
            Some(ids) => ids,
            None => return Vec::new(),
        };

        let tokens = self.tokens.read().unwrap();
        token_ids
            .iter()
            .filter_map(|id| tokens.get(id).cloned())
            .collect()
    }

    /// すべてのトークンを取得
    pub fn get_all_tokens(&self) -> Vec<TokenInfo> {
        let tokens = self.tokens.read().unwrap();
        tokens.values().cloned().collect()
    }

    /// チェーンタイプのトークンをすべて取得
    pub fn get_tokens_by_chain(&self, chain_type: ChainType) -> Vec<TokenInfo> {
        let tokens = self.tokens.read().unwrap();
        tokens
            .values()
            .filter(|token| token.chain_type == chain_type)
            .cloned()
            .collect()
    }

    /// トークンを削除
    pub fn remove_token(&self, token_id: &str) -> Result<(), Error> {
        // トークンを取得
        let token = {
            let tokens = self.tokens.read().unwrap();
            match tokens.get(token_id) {
                Some(token) => token.clone(),
                None => {
                    return Err(Error::ValidationError(format!(
                        "Token not found: {}",
                        token_id
                    )))
                }
            }
        };

        // トークンを削除
        {
            let mut tokens = self.tokens.write().unwrap();
            tokens.remove(token_id);
        }

        // チェーンアドレスマップを更新
        {
            let mut chain_address_map = self.chain_address_map.write().unwrap();
            chain_address_map.remove(&(token.chain_type, token.chain_address));
        }

        // シンボルマップを更新
        {
            let mut symbol_map = self.symbol_map.write().unwrap();
            if let Some(token_ids) = symbol_map.get_mut(&token.symbol) {
                token_ids.retain(|id| id != token_id);
                if token_ids.is_empty() {
                    symbol_map.remove(&token.symbol);
                }
            }
        }

        Ok(())
    }

    /// トークンを更新
    pub fn update_token(&self, token: TokenInfo) -> Result<(), Error> {
        let token_id = token.id.clone();

        // 既存のトークンを取得
        let old_token = {
            let tokens = self.tokens.read().unwrap();
            match tokens.get(&token_id) {
                Some(token) => token.clone(),
                None => {
                    return Err(Error::ValidationError(format!(
                        "Token not found: {}",
                        token_id
                    )))
                }
            }
        };

        // チェーンアドレスが変更された場合、チェーンアドレスマップを更新
        if old_token.chain_type != token.chain_type
            || old_token.chain_address != token.chain_address
        {
            let mut chain_address_map = self.chain_address_map.write().unwrap();
            chain_address_map.remove(&(old_token.chain_type, old_token.chain_address));
            chain_address_map.insert(
                (token.chain_type, token.chain_address.clone()),
                token_id.clone(),
            );
        }

        // シンボルが変更された場合、シンボルマップを更新
        if old_token.symbol != token.symbol {
            let mut symbol_map = self.symbol_map.write().unwrap();

            // 古いシンボルから削除
            if let Some(token_ids) = symbol_map.get_mut(&old_token.symbol) {
                token_ids.retain(|id| id != &token_id);
                if token_ids.is_empty() {
                    symbol_map.remove(&old_token.symbol);
                }
            }

            // 新しいシンボルに追加
            let token_ids = symbol_map
                .entry(token.symbol.clone())
                .or_insert_with(Vec::new);
            token_ids.push(token_id.clone());
        }

        // トークンを更新
        {
            let mut tokens = self.tokens.write().unwrap();
            tokens.insert(token_id, token);
        }

        Ok(())
    }

    /// デフォルトのトークンを登録
    pub fn register_default_tokens(&self) -> Result<(), Error> {
        // ShardX ネイティブトークン
        let shardx_token = TokenInfo::new(
            "shardx-native".to_string(),
            "ShardX".to_string(),
            "SHX".to_string(),
            18,
            ChainType::ShardX,
            "native".to_string(),
        );
        self.register_token(shardx_token)?;

        // Ethereum (ETH)
        let eth_token = TokenInfo::new(
            "ethereum-eth".to_string(),
            "Ethereum".to_string(),
            "ETH".to_string(),
            18,
            ChainType::Ethereum,
            "0x0000000000000000000000000000000000000000".to_string(),
        );
        self.register_token(eth_token)?;

        // Wrapped Ethereum (WETH)
        let weth_token = TokenInfo::new(
            "ethereum-weth".to_string(),
            "Wrapped Ethereum".to_string(),
            "WETH".to_string(),
            18,
            ChainType::Ethereum,
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        );
        self.register_token(weth_token)?;

        // USDC on Ethereum
        let usdc_token = TokenInfo::new(
            "ethereum-usdc".to_string(),
            "USD Coin".to_string(),
            "USDC".to_string(),
            6,
            ChainType::Ethereum,
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        );
        self.register_token(usdc_token)?;

        // USDT on Ethereum
        let usdt_token = TokenInfo::new(
            "ethereum-usdt".to_string(),
            "Tether USD".to_string(),
            "USDT".to_string(),
            6,
            ChainType::Ethereum,
            "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
        );
        self.register_token(usdt_token)?;

        // Solana (SOL)
        let sol_token = TokenInfo::new(
            "solana-sol".to_string(),
            "Solana".to_string(),
            "SOL".to_string(),
            9,
            ChainType::Solana,
            "native".to_string(),
        );
        self.register_token(sol_token)?;

        // USDC on Solana
        let solana_usdc_token = TokenInfo::new(
            "solana-usdc".to_string(),
            "USD Coin".to_string(),
            "USDC".to_string(),
            6,
            ChainType::Solana,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        );
        self.register_token(solana_usdc_token)?;

        info!("Registered default tokens");

        Ok(())
    }
}
