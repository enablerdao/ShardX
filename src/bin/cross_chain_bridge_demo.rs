use log::{error, info};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use shardx::cross_chain::{BridgeConfig, BridgeStatus, ChainType, CrossChainBridge};
use shardx::error::Error;
use shardx::transaction::Transaction;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // ロガーを初期化
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Starting cross-chain bridge demo...");

    // メッセージチャネルを作成
    let (message_tx, message_rx) = mpsc::channel(1000);

    // ブリッジ設定を作成
    let bridge_config = BridgeConfig {
        id: "shardx-ethereum-bridge".to_string(),
        name: "ShardX-Ethereum Bridge".to_string(),
        source_chain: ChainType::ShardX,
        target_chain: ChainType::Ethereum,
        source_endpoint: "http://localhost:8545".to_string(),
        target_endpoint: "https://mainnet.infura.io/v3/your-project-id".to_string(),
        source_contract: None,
        target_contract: Some("0x1234567890123456789012345678901234567890".to_string()),
        max_transaction_size: 1024 * 1024, // 1MB
        max_message_size: 1024 * 1024,     // 1MB
        confirmation_blocks: 12,
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
    let bridge = CrossChainBridge::new(bridge_config, message_tx.clone(), message_rx);

    // ブリッジを初期化
    bridge.initialize().await?;

    // ブリッジの状態を確認
    let status = bridge.get_status();
    info!("Bridge status: {:?}", status);

    if status != BridgeStatus::Connected {
        error!("Bridge is not connected. Exiting...");
        return Err(Error::ConnectionError(
            "Bridge is not connected".to_string(),
        ));
    }

    // テストトランザクションを作成
    let transaction = create_test_transaction();

    // トランザクションを送信
    info!("Sending test transaction...");
    let tx_id = bridge.start_transaction(transaction).await?;
    info!("Transaction sent. ID: {}", tx_id);

    // トランザクションの状態を確認
    for i in 0..10 {
        tokio::time::sleep(Duration::from_secs(3)).await;

        let tx_status = bridge.get_transaction_status(&tx_id)?;
        info!("Transaction status ({}): {:?}", i, tx_status);

        if tx_status.is_completed() {
            let tx_details = bridge.get_transaction_details(&tx_id)?;

            if tx_details.is_successful() {
                info!("Transaction completed successfully!");
                if let Some(proof) = &tx_details.proof {
                    info!("Transaction proof: {}", proof.id);
                    info!("Block hash: {}", proof.block_hash);
                    info!("Block height: {}", proof.block_height);
                }
            } else {
                error!("Transaction failed: {:?}", tx_details.error);
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

/// テストトランザクションを作成
fn create_test_transaction() -> Transaction {
    Transaction {
        id: uuid::Uuid::new_v4().to_string(),
        from: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
        to: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        amount: "1.0".to_string(),
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
    }
}
