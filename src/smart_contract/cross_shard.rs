use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Error;
use crate::shard::{ShardId, ShardInfo};
use crate::smart_contract::storage::{ContractStorage, StorageError, StorageKey, StorageValue};
use crate::smart_contract::vm::{ExecutionContext, ExecutionResult, VMError, VirtualMachine};
use crate::transaction::{Transaction, TransactionStatus};

/// クロスシャード呼び出し
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardCall {
    /// 呼び出しID
    pub id: String,
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub target_shard_id: ShardId,
    /// 送信元コントラクトアドレス
    pub source_contract: String,
    /// 送信先コントラクトアドレス
    pub target_contract: String,
    /// メソッド名
    pub method: String,
    /// 引数
    pub args: Vec<u8>,
    /// 値
    pub value: u64,
    /// ガス制限
    pub gas_limit: u64,
    /// ノンス
    pub nonce: u64,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 完了日時
    pub completed_at: Option<DateTime<Utc>>,
    /// ステータス
    pub status: CrossShardCallStatus,
    /// 結果
    pub result: Option<CrossShardResult>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// クロスシャード呼び出しステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CrossShardCallStatus {
    /// 保留中
    Pending,
    /// 送信済み
    Sent,
    /// 受信済み
    Received,
    /// 実行中
    Executing,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// タイムアウト
    TimedOut,
    /// キャンセル
    Cancelled,
}

/// クロスシャード結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardResult {
    /// 成功フラグ
    pub success: bool,
    /// 戻りデータ
    pub return_data: Vec<u8>,
    /// ガス使用量
    pub gas_used: u64,
    /// エラーメッセージ
    pub error_message: Option<String>,
    /// 完了日時
    pub completed_at: DateTime<Utc>,
}

/// クロスシャード実行器
pub struct CrossShardExecutor<V: VirtualMachine, S: ContractStorage> {
    /// 仮想マシン
    vm: V,
    /// ストレージ
    storage: S,
    /// シャード情報
    shard_info: HashMap<ShardId, ShardInfo>,
    /// 保留中の呼び出し
    pending_calls: HashMap<String, CrossShardCall>,
    /// 完了した呼び出し
    completed_calls: HashMap<String, CrossShardCall>,
    /// 現在のシャードID
    current_shard_id: ShardId,
    /// タイムアウト（秒）
    timeout_seconds: u64,
}

impl<V: VirtualMachine, S: ContractStorage> CrossShardExecutor<V, S> {
    /// 新しいクロスシャード実行器を作成
    pub fn new(vm: V, storage: S, current_shard_id: ShardId, timeout_seconds: u64) -> Self {
        Self {
            vm,
            storage,
            shard_info: HashMap::new(),
            pending_calls: HashMap::new(),
            completed_calls: HashMap::new(),
            current_shard_id,
            timeout_seconds,
        }
    }

    /// シャード情報を追加
    pub fn add_shard_info(&mut self, shard_id: ShardId, info: ShardInfo) {
        self.shard_info.insert(shard_id, info);
    }

    /// クロスシャード呼び出しを作成
    pub fn create_call(
        &mut self,
        source_contract: String,
        target_shard_id: ShardId,
        target_contract: String,
        method: String,
        args: Vec<u8>,
        value: u64,
        gas_limit: u64,
    ) -> Result<String, Error> {
        // 送信先シャードが存在するか確認
        if !self.shard_info.contains_key(&target_shard_id) {
            return Err(Error::InvalidInput(format!(
                "Target shard not found: {}",
                target_shard_id
            )));
        }

        // 送信元コントラクトが存在するか確認
        if !self.storage.has_contract(&source_contract)? {
            return Err(Error::NotFound(format!(
                "Source contract not found: {}",
                source_contract
            )));
        }

        // 呼び出しIDを生成
        let id = format!("cross_shard_call_{}", Utc::now().timestamp_nanos());

        // ノンスを生成
        let nonce = self.pending_calls.len() as u64 + self.completed_calls.len() as u64;

        // クロスシャード呼び出しを作成
        let call = CrossShardCall {
            id: id.clone(),
            source_shard_id: self.current_shard_id.clone(),
            target_shard_id,
            source_contract,
            target_contract,
            method,
            args,
            value,
            gas_limit,
            nonce,
            created_at: Utc::now(),
            completed_at: None,
            status: CrossShardCallStatus::Pending,
            result: None,
            metadata: None,
        };

        // 保留中の呼び出しに追加
        self.pending_calls.insert(id.clone(), call);

        Ok(id)
    }

    /// クロスシャード呼び出しを送信
    pub fn send_call(&mut self, call_id: &str) -> Result<(), Error> {
        // 呼び出しを取得
        let call = self
            .pending_calls
            .get_mut(call_id)
            .ok_or_else(|| Error::NotFound(format!("Cross-shard call not found: {}", call_id)))?;

        // ステータスをチェック
        if call.status != CrossShardCallStatus::Pending {
            return Err(Error::InvalidState(format!(
                "Call is not pending: {:?}",
                call.status
            )));
        }

        // 送信先シャードが存在するか確認
        if !self.shard_info.contains_key(&call.target_shard_id) {
            return Err(Error::InvalidInput(format!(
                "Target shard not found: {}",
                call.target_shard_id
            )));
        }

        // 実際の実装では、送信先シャードにメッセージを送信する
        // ここでは簡易的に送信済みとする
        call.status = CrossShardCallStatus::Sent;

        Ok(())
    }

    /// クロスシャード呼び出しを受信
    pub fn receive_call(&mut self, call: CrossShardCall) -> Result<(), Error> {
        // 送信先シャードが現在のシャードか確認
        if call.target_shard_id != self.current_shard_id {
            return Err(Error::InvalidInput(format!(
                "Call is not for this shard: {}",
                call.target_shard_id
            )));
        }

        // 呼び出しIDが既に存在するか確認
        if self.pending_calls.contains_key(&call.id) || self.completed_calls.contains_key(&call.id)
        {
            return Err(Error::AlreadyExists(format!(
                "Call already exists: {}",
                call.id
            )));
        }

        // 送信先コントラクトが存在するか確認
        if !self.storage.has_contract(&call.target_contract)? {
            // 呼び出しを失敗として完了
            let mut failed_call = call;
            failed_call.status = CrossShardCallStatus::Failed;
            failed_call.completed_at = Some(Utc::now());
            failed_call.result = Some(CrossShardResult {
                success: false,
                return_data: Vec::new(),
                gas_used: 0,
                error_message: Some(format!(
                    "Target contract not found: {}",
                    failed_call.target_contract
                )),
                completed_at: Utc::now(),
            });

            self.completed_calls
                .insert(failed_call.id.clone(), failed_call);

            return Err(Error::NotFound(format!(
                "Target contract not found: {}",
                call.target_contract
            )));
        }

        // 呼び出しを保留中に追加
        let mut received_call = call;
        received_call.status = CrossShardCallStatus::Received;

        self.pending_calls
            .insert(received_call.id.clone(), received_call);

        Ok(())
    }

    /// クロスシャード呼び出しを実行
    pub fn execute_call(&mut self, call_id: &str) -> Result<CrossShardResult, Error> {
        // 呼び出しを取得
        let call = self
            .pending_calls
            .get_mut(call_id)
            .ok_or_else(|| Error::NotFound(format!("Cross-shard call not found: {}", call_id)))?;

        // ステータスをチェック
        if call.status != CrossShardCallStatus::Received
            && call.status != CrossShardCallStatus::Sent
        {
            return Err(Error::InvalidState(format!(
                "Call is not ready for execution: {:?}",
                call.status
            )));
        }

        // 送信先コントラクトが存在するか確認
        if !self.storage.has_contract(&call.target_contract)? {
            // 呼び出しを失敗として完了
            call.status = CrossShardCallStatus::Failed;
            call.completed_at = Some(Utc::now());

            let result = CrossShardResult {
                success: false,
                return_data: Vec::new(),
                gas_used: 0,
                error_message: Some(format!(
                    "Target contract not found: {}",
                    call.target_contract
                )),
                completed_at: Utc::now(),
            };

            call.result = Some(result.clone());

            // 完了した呼び出しに移動
            let completed_call = self.pending_calls.remove(call_id).unwrap();
            self.completed_calls
                .insert(call_id.to_string(), completed_call);

            return Ok(result);
        }

        // 実行コンテキストを作成
        let context = ExecutionContext {
            gas_limit: call.gas_limit,
            sender: call.source_contract.clone(),
            value: call.value,
            data: call.args.clone(),
            address: Some(call.target_contract.clone()),
            block_height: 0,
            block_time: Utc::now(),
            is_static: false,
            depth: 0,
        };

        // 呼び出しを実行中に更新
        call.status = CrossShardCallStatus::Executing;

        // コントラクトを呼び出し
        let vm_result =
            match self
                .vm
                .call(call.target_contract.clone(), call.method.clone(), context)
            {
                Ok(result) => result,
                Err(e) => {
                    // 呼び出しを失敗として完了
                    call.status = CrossShardCallStatus::Failed;
                    call.completed_at = Some(Utc::now());

                    let result = CrossShardResult {
                        success: false,
                        return_data: Vec::new(),
                        gas_used: 0,
                        error_message: Some(format!("VM error: {}", e)),
                        completed_at: Utc::now(),
                    };

                    call.result = Some(result.clone());

                    // 完了した呼び出しに移動
                    let completed_call = self.pending_calls.remove(call_id).unwrap();
                    self.completed_calls
                        .insert(call_id.to_string(), completed_call);

                    return Ok(result);
                }
            };

        // 結果を作成
        let result = CrossShardResult {
            success: vm_result.success,
            return_data: vm_result.return_data,
            gas_used: vm_result.gas_used,
            error_message: vm_result.error.map(|e| format!("{:?}", e)),
            completed_at: Utc::now(),
        };

        // 呼び出しを完了
        call.status = if result.success {
            CrossShardCallStatus::Completed
        } else {
            CrossShardCallStatus::Failed
        };

        call.completed_at = Some(result.completed_at);
        call.result = Some(result.clone());

        // 完了した呼び出しに移動
        let completed_call = self.pending_calls.remove(call_id).unwrap();
        self.completed_calls
            .insert(call_id.to_string(), completed_call);

        Ok(result)
    }

    /// クロスシャード呼び出しの結果を取得
    pub fn get_call_result(&self, call_id: &str) -> Result<Option<CrossShardResult>, Error> {
        // 完了した呼び出しから検索
        if let Some(call) = self.completed_calls.get(call_id) {
            return Ok(call.result.clone());
        }

        // 保留中の呼び出しから検索
        if let Some(call) = self.pending_calls.get(call_id) {
            return Ok(call.result.clone());
        }

        Err(Error::NotFound(format!(
            "Cross-shard call not found: {}",
            call_id
        )))
    }

    /// クロスシャード呼び出しのステータスを取得
    pub fn get_call_status(&self, call_id: &str) -> Result<CrossShardCallStatus, Error> {
        // 完了した呼び出しから検索
        if let Some(call) = self.completed_calls.get(call_id) {
            return Ok(call.status.clone());
        }

        // 保留中の呼び出しから検索
        if let Some(call) = self.pending_calls.get(call_id) {
            return Ok(call.status.clone());
        }

        Err(Error::NotFound(format!(
            "Cross-shard call not found: {}",
            call_id
        )))
    }

    /// クロスシャード呼び出しを取得
    pub fn get_call(&self, call_id: &str) -> Result<&CrossShardCall, Error> {
        // 完了した呼び出しから検索
        if let Some(call) = self.completed_calls.get(call_id) {
            return Ok(call);
        }

        // 保留中の呼び出しから検索
        if let Some(call) = self.pending_calls.get(call_id) {
            return Ok(call);
        }

        Err(Error::NotFound(format!(
            "Cross-shard call not found: {}",
            call_id
        )))
    }

    /// 保留中のクロスシャード呼び出しを取得
    pub fn get_pending_calls(&self) -> Vec<&CrossShardCall> {
        self.pending_calls.values().collect()
    }

    /// 完了したクロスシャード呼び出しを取得
    pub fn get_completed_calls(&self) -> Vec<&CrossShardCall> {
        self.completed_calls.values().collect()
    }

    /// 送信先シャード別の保留中呼び出しを取得
    pub fn get_pending_calls_by_target_shard(&self, shard_id: &ShardId) -> Vec<&CrossShardCall> {
        self.pending_calls
            .values()
            .filter(|call| call.target_shard_id == *shard_id)
            .collect()
    }

    /// 送信元シャード別の保留中呼び出しを取得
    pub fn get_pending_calls_by_source_shard(&self, shard_id: &ShardId) -> Vec<&CrossShardCall> {
        self.pending_calls
            .values()
            .filter(|call| call.source_shard_id == *shard_id)
            .collect()
    }

    /// 送信先コントラクト別の保留中呼び出しを取得
    pub fn get_pending_calls_by_target_contract(&self, contract: &str) -> Vec<&CrossShardCall> {
        self.pending_calls
            .values()
            .filter(|call| call.target_contract == *contract)
            .collect()
    }

    /// 送信元コントラクト別の保留中呼び出しを取得
    pub fn get_pending_calls_by_source_contract(&self, contract: &str) -> Vec<&CrossShardCall> {
        self.pending_calls
            .values()
            .filter(|call| call.source_contract == *contract)
            .collect()
    }

    /// タイムアウトした呼び出しを処理
    pub fn process_timeouts(&mut self) -> Vec<String> {
        let now = Utc::now();
        let timeout_duration = chrono::Duration::seconds(self.timeout_seconds as i64);
        let mut timed_out_calls = Vec::new();

        // タイムアウトした呼び出しを検索
        for (id, call) in self.pending_calls.iter_mut() {
            if call.status != CrossShardCallStatus::Completed
                && call.status != CrossShardCallStatus::Failed
            {
                let elapsed = now - call.created_at;

                if elapsed > timeout_duration {
                    call.status = CrossShardCallStatus::TimedOut;
                    call.completed_at = Some(now);
                    call.result = Some(CrossShardResult {
                        success: false,
                        return_data: Vec::new(),
                        gas_used: 0,
                        error_message: Some("Call timed out".to_string()),
                        completed_at: now,
                    });

                    timed_out_calls.push(id.clone());
                }
            }
        }

        // タイムアウトした呼び出しを完了した呼び出しに移動
        for id in &timed_out_calls {
            if let Some(call) = self.pending_calls.remove(id) {
                self.completed_calls.insert(id.clone(), call);
            }
        }

        timed_out_calls
    }

    /// 保留中の呼び出しをクリーンアップ
    pub fn cleanup_pending_calls(&mut self, max_age_seconds: u64) -> Vec<String> {
        let now = Utc::now();
        let max_age_duration = chrono::Duration::seconds(max_age_seconds as i64);
        let mut cleaned_calls = Vec::new();

        // 古い呼び出しを検索
        for (id, call) in self.pending_calls.iter() {
            let elapsed = now - call.created_at;

            if elapsed > max_age_duration {
                cleaned_calls.push(id.clone());
            }
        }

        // 古い呼び出しを削除
        for id in &cleaned_calls {
            self.pending_calls.remove(id);
        }

        cleaned_calls
    }

    /// 完了した呼び出しをクリーンアップ
    pub fn cleanup_completed_calls(&mut self, max_age_seconds: u64) -> Vec<String> {
        let now = Utc::now();
        let max_age_duration = chrono::Duration::seconds(max_age_seconds as i64);
        let mut cleaned_calls = Vec::new();

        // 古い呼び出しを検索
        for (id, call) in self.completed_calls.iter() {
            if let Some(completed_at) = call.completed_at {
                let elapsed = now - completed_at;

                if elapsed > max_age_duration {
                    cleaned_calls.push(id.clone());
                }
            }
        }

        // 古い呼び出しを削除
        for id in &cleaned_calls {
            self.completed_calls.remove(id);
        }

        cleaned_calls
    }

    /// タイムアウト時間を設定
    pub fn set_timeout(&mut self, timeout_seconds: u64) {
        self.timeout_seconds = timeout_seconds;
    }

    /// 現在のシャードIDを取得
    pub fn get_current_shard_id(&self) -> &ShardId {
        &self.current_shard_id
    }

    /// 現在のシャードIDを設定
    pub fn set_current_shard_id(&mut self, shard_id: ShardId) {
        self.current_shard_id = shard_id;
    }

    /// シャード情報を取得
    pub fn get_shard_info(&self, shard_id: &ShardId) -> Option<&ShardInfo> {
        self.shard_info.get(shard_id)
    }

    /// すべてのシャード情報を取得
    pub fn get_all_shard_info(&self) -> &HashMap<ShardId, ShardInfo> {
        &self.shard_info
    }
}
