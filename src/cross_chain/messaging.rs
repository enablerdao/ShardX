use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::bridge::ChainType;
use super::transaction::TransactionProof;
use crate::error::Error;

/// メッセージの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// 送信中
    Sending,
    /// 送信済み
    Sent,
    /// 受信済み
    Received,
    /// 処理中
    Processing,
    /// 処理完了
    Processed,
    /// 失敗
    Failed,
    /// タイムアウト
    Timeout,
    /// OK
    Ok,
    /// エラー
    Error,
}

/// メッセージタイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// トランザクションリクエスト
    TransactionRequest,
    /// トランザクションレスポンス
    TransactionResponse {
        /// 成功したかどうか
        success: bool,
        /// エラーメッセージ（失敗時）
        error: Option<String>,
    },
    /// トランザクション証明
    TransactionProof {
        /// 証明データ
        proof: TransactionProof,
    },
    /// ステータスリクエスト
    StatusRequest,
    /// ステータスレスポンス
    StatusResponse {
        /// ステータス
        status: MessageStatus,
    },
}

/// クロスチェーンメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainMessage {
    /// メッセージID
    pub id: String,
    /// トランザクションID
    pub transaction_id: String,
    /// 送信元チェーン
    pub from_chain: ChainType,
    /// 送信先チェーン
    pub to_chain: ChainType,
    /// メッセージタイプ
    pub message_type: MessageType,
    /// メッセージデータ（オプション）
    pub data: Option<Vec<u8>>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 送信日時
    pub sent_at: Option<DateTime<Utc>>,
    /// 受信日時
    pub received_at: Option<DateTime<Utc>>,
    /// 処理日時
    pub processed_at: Option<DateTime<Utc>>,
    /// ステータス
    pub status: MessageStatus,
    /// リトライ回数
    pub retry_count: u32,
    /// エラーメッセージ（失敗時）
    pub error: Option<String>,
}

impl CrossChainMessage {
    /// 新しいクロスチェーンメッセージを作成
    pub fn new(
        transaction_id: String,
        from_chain: ChainType,
        to_chain: ChainType,
        message_type: MessageType,
        data: Option<Vec<u8>>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();

        Self {
            id,
            transaction_id,
            from_chain,
            to_chain,
            message_type,
            data,
            created_at: Utc::now(),
            sent_at: None,
            received_at: None,
            processed_at: None,
            status: MessageStatus::Sending,
            retry_count: 0,
            error: None,
        }
    }

    /// メッセージを送信済みに設定
    pub fn mark_as_sent(&mut self) {
        self.sent_at = Some(Utc::now());
        self.status = MessageStatus::Sent;
    }

    /// メッセージを受信済みに設定
    pub fn mark_as_received(&mut self) {
        self.received_at = Some(Utc::now());
        self.status = MessageStatus::Received;
    }

    /// メッセージを処理中に設定
    pub fn mark_as_processing(&mut self) {
        self.status = MessageStatus::Processing;
    }

    /// メッセージを処理完了に設定
    pub fn mark_as_processed(&mut self) {
        self.processed_at = Some(Utc::now());
        self.status = MessageStatus::Processed;
    }

    /// メッセージを失敗に設定
    pub fn mark_as_failed(&mut self, error: String) {
        self.status = MessageStatus::Failed;
        self.error = Some(error);
    }

    /// メッセージをタイムアウトに設定
    pub fn mark_as_timeout(&mut self) {
        self.status = MessageStatus::Timeout;
    }

    /// リトライ回数をインクリメント
    pub fn increment_retry_count(&mut self) {
        self.retry_count += 1;
    }
}

/// メッセージキュー
pub struct MessageQueue {
    /// 送信キュー
    send_queue: Mutex<Vec<CrossChainMessage>>,
    /// 受信キュー
    receive_queue: Mutex<Vec<CrossChainMessage>>,
    /// 処理済みメッセージ
    processed_messages: Mutex<Vec<CrossChainMessage>>,
    /// メッセージ送信チャネル
    message_sender: mpsc::Sender<CrossChainMessage>,
    /// メッセージ受信チャネル
    message_receiver: Mutex<Option<mpsc::Receiver<CrossChainMessage>>>,
    /// 最大キューサイズ
    max_queue_size: usize,
    /// 最大保持メッセージ数
    max_processed_messages: usize,
}

impl MessageQueue {
    /// 新しいメッセージキューを作成
    pub fn new(
        message_sender: mpsc::Sender<CrossChainMessage>,
        message_receiver: mpsc::Receiver<CrossChainMessage>,
        max_queue_size: usize,
        max_processed_messages: usize,
    ) -> Self {
        Self {
            send_queue: Mutex::new(Vec::with_capacity(max_queue_size)),
            receive_queue: Mutex::new(Vec::with_capacity(max_queue_size)),
            processed_messages: Mutex::new(Vec::with_capacity(max_processed_messages)),
            message_sender,
            message_receiver: Mutex::new(Some(message_receiver)),
            max_queue_size,
            max_processed_messages,
        }
    }

    /// メッセージを送信キューに追加
    pub fn enqueue_for_sending(&self, message: CrossChainMessage) -> Result<(), Error> {
        let mut send_queue = self.send_queue.lock().unwrap();

        if send_queue.len() >= self.max_queue_size {
            return Err(Error::CapacityError("Send queue is full".to_string()));
        }

        send_queue.push(message);
        Ok(())
    }

    /// メッセージを受信キューに追加
    pub fn enqueue_for_processing(&self, message: CrossChainMessage) -> Result<(), Error> {
        let mut receive_queue = self.receive_queue.lock().unwrap();

        if receive_queue.len() >= self.max_queue_size {
            return Err(Error::CapacityError("Receive queue is full".to_string()));
        }

        receive_queue.push(message);
        Ok(())
    }

    /// 送信キューからメッセージを取得
    pub fn dequeue_for_sending(&self) -> Option<CrossChainMessage> {
        let mut send_queue = self.send_queue.lock().unwrap();

        if send_queue.is_empty() {
            return None;
        }

        Some(send_queue.remove(0))
    }

    /// 受信キューからメッセージを取得
    pub fn dequeue_for_processing(&self) -> Option<CrossChainMessage> {
        let mut receive_queue = self.receive_queue.lock().unwrap();

        if receive_queue.is_empty() {
            return None;
        }

        Some(receive_queue.remove(0))
    }

    /// 処理済みメッセージを追加
    pub fn add_processed_message(&self, message: CrossChainMessage) {
        let mut processed_messages = self.processed_messages.lock().unwrap();

        // 最大保持数を超える場合は古いメッセージを削除
        if processed_messages.len() >= self.max_processed_messages {
            processed_messages.remove(0);
        }

        processed_messages.push(message);
    }

    /// 処理済みメッセージを取得
    pub fn get_processed_message(&self, message_id: &str) -> Option<CrossChainMessage> {
        let processed_messages = self.processed_messages.lock().unwrap();

        processed_messages
            .iter()
            .find(|m| m.id == message_id)
            .cloned()
    }

    /// 送信キューのサイズを取得
    pub fn send_queue_size(&self) -> usize {
        let send_queue = self.send_queue.lock().unwrap();
        send_queue.len()
    }

    /// 受信キューのサイズを取得
    pub fn receive_queue_size(&self) -> usize {
        let receive_queue = self.receive_queue.lock().unwrap();
        receive_queue.len()
    }

    /// 処理済みメッセージ数を取得
    pub fn processed_messages_count(&self) -> usize {
        let processed_messages = self.processed_messages.lock().unwrap();
        processed_messages.len()
    }

    /// メッセージ処理ループを開始
    pub async fn start_processing(&self) -> Result<(), Error> {
        // メッセージ受信チャネルを取得
        let mut receiver = {
            let mut receiver_guard = self.message_receiver.lock().unwrap();
            receiver_guard
                .take()
                .ok_or_else(|| Error::InternalError("Message receiver already taken".to_string()))?
        };

        // メッセージ処理ループを開始
        let message_sender = self.message_sender.clone();

        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                // 受信メッセージを処理
                let mut message = message.clone();
                message.mark_as_received();

                // 受信キューに追加
                if let Err(e) = self.enqueue_for_processing(message.clone()) {
                    error!("Failed to enqueue message for processing: {}", e);
                    continue;
                }

                // 送信キューからメッセージを取得して送信
                while let Some(mut send_message) = self.dequeue_for_sending() {
                    send_message.mark_as_sent();

                    if let Err(e) = message_sender.send(send_message.clone()).await {
                        error!("Failed to send message: {}", e);

                        // 失敗したメッセージを再度キューに追加
                        let mut failed_message = send_message.clone();
                        failed_message.increment_retry_count();
                        failed_message.mark_as_failed(format!("Failed to send: {}", e));

                        if let Err(e) = self.enqueue_for_sending(failed_message) {
                            error!("Failed to re-enqueue message: {}", e);
                        }
                    } else {
                        // 送信成功したメッセージを処理済みに追加
                        self.add_processed_message(send_message);
                    }
                }
            }
        });

        Ok(())
    }

    /// 古いメッセージをクリーンアップ
    pub fn cleanup_old_messages(&self, max_age_hours: u64) {
        let now = Utc::now();

        // 処理済みメッセージをクリーンアップ
        {
            let mut processed_messages = self.processed_messages.lock().unwrap();

            processed_messages.retain(|m| {
                let age = now.signed_duration_since(m.created_at);
                age.num_hours() < max_age_hours as i64
            });
        }

        // 送信キューをクリーンアップ
        {
            let mut send_queue = self.send_queue.lock().unwrap();

            send_queue.retain(|m| {
                let age = now.signed_duration_since(m.created_at);
                age.num_hours() < max_age_hours as i64
            });
        }

        // 受信キューをクリーンアップ
        {
            let mut receive_queue = self.receive_queue.lock().unwrap();

            receive_queue.retain(|m| {
                let age = now.signed_duration_since(m.created_at);
                age.num_hours() < max_age_hours as i64
            });
        }
    }
}
