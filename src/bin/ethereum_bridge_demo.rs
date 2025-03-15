use std::str::FromStr;
use tokio::sync::mpsc;
use log::{info, error};
use std::time::Duration;
use web3::types::{Address, H256};

use shardx::error::Error;
use shardx::transaction::Transaction;
use shardx::cross_chain::{
    EthereumBridge, BridgeConfig, ChainType, BridgeStatus,
    CrossChainTransaction, TransactionStatus,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // ロガーを初期化
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Starting Ethereum bridge demo...");

    // メッセージチャネルを作成
    let (message_tx, message_rx) = mpsc::channel(1000);

    // ブリッジ設定を作成
    let bridge_config = BridgeConfig {
        id: "shardx-ethereum-bridge".to_string(),
        name: "ShardX-Ethereum Bridge".to_string(),
        source_chain: ChainType::ShardX,
        target_chain: ChainType::Ethereum,
        source_endpoint: "http://localhost:8545".to_string(),
        // Ganacheのエンドポイントを使用
        target_endpoint: "http://localhost:8545".to_string(),
        source_contract: None,
        target_contract: None, // 直接送金のためコントラクト不要
        max_transaction_size: 1024 * 1024, // 1MB
        max_message_size: 1024 * 1024, // 1MB
        confirmation_blocks: 1, // ローカル環境では1ブロックで十分
        timeout_sec: 60,
        retry_count: 3,
        retry_interval_sec: 10,
        fee_settings: shardx::cross_chain::bridge::FeeSetting {
            base_fee: 0.001,
            fee_per_byte: 0.0001,
            fee_currency: "ETH".to_string(),
            min_fee: 0.001,
            max_fee: Some(0.1),
        },
    };

    // ブリッジを作成
    let mut bridge = EthereumBridge::new(
        bridge_config,
        message_tx.clone(),
        message_rx,
    );

    // テスト用の秘密鍵（Ganacheのデフォルトアカウント）
    // 注意: これは公開リポジトリに含めるべきではありません
    let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    // ブリッジを初期化
    match bridge.initialize(private_key).await {
        Ok(_) => info!("Bridge initialized successfully"),
        Err(e) => {
            error!("Failed to initialize bridge: {}", e);
            // Ganacheが起動していない場合は、起動方法を表示
            info!("Make sure Ganache is running. You can start it with:");
            info!("npx ganache-cli --deterministic");
            return Err(e);
        }
    }

    // ブリッジの状態を確認
    let status = bridge.get_status();
    info!("Bridge status: {:?}", status);

    if status != BridgeStatus::Connected {
        error!("Bridge is not connected. Exiting...");
        return Err(Error::ConnectionError("Bridge is not connected".to_string()));
    }

    // 送金先アドレス（Ganacheの2番目のアカウント）
    let to_address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";

    // 残高を確認
    let wallet_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"; // Ganacheの1番目のアカウント
    let balance = bridge.get_balance(wallet_address).await?;
    info!("Wallet balance: {} ETH", web3::types::U256::from(balance) / web3::types::U256::exp10(18));

    // テストトランザクションを作成
    let transaction = Transaction {
        id: uuid::Uuid::new_v4().to_string(),
        from: wallet_address.to_string(),
        to: to_address.to_string(),
        amount: "0.01".to_string(), // 0.01 ETH
        fee: "0.001".to_string(),
        data: None,
        nonce: 1,
        timestamp: chrono::Utc::now().timestamp() as u64,
        signature: "test-signature".to_string(),
        status: shardx::transaction::TransactionStatus::Pending,
        shard_id: "shard-1".to_string(),
        block_hash: None,
        block_height: None,
        parent_id: None,
    };

    // トランザクションを直接送信
    info!("Sending direct Ethereum transaction...");
    let tx_hash = bridge.send_transaction(&transaction).await?;
    info!("Transaction sent. Hash: {}", tx_hash);

    // トランザクションの状態を確認
    for i in 0..10 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // 最新のブロック番号を更新
        let latest_block = bridge.update_latest_block().await?;
        info!("Latest block: {}", latest_block);
        
        let tx_status = bridge.check_transaction_status(tx_hash).await?;
        info!("Transaction status ({}): {:?}", i, tx_status);
        
        if tx_status == TransactionStatus::Confirmed {
            info!("Transaction confirmed!");
            
            // トランザクション証明を作成
            let proof = bridge.create_transaction_proof(tx_hash, &transaction.id).await?;
            info!("Transaction proof created: {}", proof.id);
            info!("Block hash: {}", proof.block_hash);
            info!("Block height: {}", proof.block_height);
            
            break;
        }
    }

    // 送金先の残高を確認
    let recipient_balance = bridge.get_balance(to_address).await?;
    info!("Recipient balance: {} ETH", web3::types::U256::from(recipient_balance) / web3::types::U256::exp10(18));

    // クロスチェーントランザクションを開始
    info!("Starting cross-chain transaction...");
    let cross_tx_id = bridge.start_cross_chain_transaction(transaction.clone()).await?;
    info!("Cross-chain transaction started. ID: {}", cross_tx_id);

    // トランザクションの状態を確認
    for i in 0..10 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let tx_status = bridge.get_transaction_status(&cross_tx_id)?;
        info!("Cross-chain transaction status ({}): {:?}", i, tx_status);
        
        if matches!(tx_status, TransactionStatus::Confirmed | TransactionStatus::Verified | TransactionStatus::Failed) {
            let tx_details = bridge.get_transaction_details(&cross_tx_id)?;
            
            if tx_details.is_successful() {
                info!("Cross-chain transaction completed successfully!");
                if let Some(proof) = &tx_details.proof {
                    info!("Transaction proof: {}", proof.id);
                    info!("Block hash: {}", proof.block_hash);
                    info!("Block height: {}", proof.block_height);
                }
            } else {
                error!("Cross-chain transaction failed: {:?}", tx_details.error);
            }
            
            break;
        }
    }

    // ブリッジを停止
    info!("Shutting down bridge...");
    bridge.shutdown().await?;
    info!("Bridge shut down successfully");

    Ok(())
}