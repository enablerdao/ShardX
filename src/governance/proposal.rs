use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Error;
use crate::governance::voting::{Vote, VotingPeriod, VotingPower, VotingResult, VotingStrategy};

/// 提案タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProposalType {
    /// パラメータ変更
    ParameterChange,
    /// コード更新
    CodeUpgrade,
    /// 資金配分
    FundingAllocation,
    /// メンバー追加
    MemberAddition,
    /// メンバー削除
    MemberRemoval,
    /// ロール割り当て
    RoleAssignment,
    /// ポリシー変更
    PolicyChange,
    /// 報酬分配
    RewardDistribution,
    /// 紛争解決
    DisputeResolution,
    /// テキスト提案
    TextProposal,
    /// カスタム提案
    Custom(String),
}

/// 提案ステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProposalStatus {
    /// ドラフト
    Draft,
    /// 提出済み
    Submitted,
    /// 検討中
    UnderConsideration,
    /// 投票中
    Voting,
    /// 可決
    Accepted,
    /// 否決
    Rejected,
    /// 実行中
    Executing,
    /// 実行済み
    Executed,
    /// 失敗
    Failed,
    /// キャンセル
    Cancelled,
    /// 期限切れ
    Expired,
}

/// 提案メタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalMetadata {
    /// 作成者
    pub author: String,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// 提出日時
    pub submitted_at: Option<DateTime<Utc>>,
    /// 投票開始日時
    pub voting_started_at: Option<DateTime<Utc>>,
    /// 投票終了日時
    pub voting_ended_at: Option<DateTime<Utc>>,
    /// 実行日時
    pub executed_at: Option<DateTime<Utc>>,
    /// タグ
    pub tags: Vec<String>,
    /// カテゴリ
    pub category: Option<String>,
    /// 優先度
    pub priority: Option<String>,
    /// 難易度
    pub difficulty: Option<String>,
    /// 影響度
    pub impact: Option<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 提案オプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalOptions {
    /// 投票戦略
    pub voting_strategy: Option<VotingStrategy>,
    /// 投票期間
    pub voting_period: Option<VotingPeriod>,
    /// クォーラム
    pub quorum: Option<f64>,
    /// 閾値
    pub threshold: Option<f64>,
    /// 最小投票数
    pub min_votes: Option<u64>,
    /// 最小投票率
    pub min_participation: Option<f64>,
    /// 早期終了条件
    pub early_execution: Option<bool>,
    /// 早期終了閾値
    pub early_execution_threshold: Option<f64>,
    /// 遅延実行
    pub delayed_execution: Option<bool>,
    /// 遅延実行期間（秒）
    pub delayed_execution_seconds: Option<u64>,
    /// 拒否権
    pub veto_enabled: Option<bool>,
    /// 拒否権保持者
    pub veto_holders: Option<Vec<String>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl Default for ProposalOptions {
    fn default() -> Self {
        Self {
            voting_strategy: Some(VotingStrategy::Simple),
            voting_period: Some(VotingPeriod::Duration(Duration::days(7))),
            quorum: Some(0.4),
            threshold: Some(0.5),
            min_votes: Some(1),
            min_participation: Some(0.1),
            early_execution: Some(false),
            early_execution_threshold: Some(0.66),
            delayed_execution: Some(false),
            delayed_execution_seconds: Some(86400), // 1日
            veto_enabled: Some(false),
            veto_holders: None,
            additional_properties: HashMap::new(),
        }
    }
}

/// 提案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// ID
    pub id: String,
    /// タイトル
    pub title: String,
    /// 説明
    pub description: String,
    /// タイプ
    pub proposal_type: ProposalType,
    /// ステータス
    pub status: ProposalStatus,
    /// メタデータ
    pub metadata: ProposalMetadata,
    /// オプション
    pub options: ProposalOptions,
    /// 投票
    pub votes: HashMap<String, Vote>,
    /// 投票結果
    pub voting_result: Option<VotingResult>,
    /// 実行データ
    pub execution_data: Option<serde_json::Value>,
    /// 実行結果
    pub execution_result: Option<serde_json::Value>,
    /// 添付ファイル
    pub attachments: Option<Vec<Attachment>>,
    /// コメント
    pub comments: Option<Vec<Comment>>,
    /// 履歴
    pub history: Option<Vec<ProposalHistory>>,
    /// 関連提案
    pub related_proposals: Option<Vec<String>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 添付ファイル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: Option<String>,
    /// MIMEタイプ
    pub mime_type: String,
    /// サイズ（バイト）
    pub size: u64,
    /// URL
    pub url: String,
    /// ハッシュ
    pub hash: Option<String>,
    /// アップロード日時
    pub uploaded_at: DateTime<Utc>,
    /// アップロード者
    pub uploaded_by: String,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// コメント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// ID
    pub id: String,
    /// 内容
    pub content: String,
    /// 作成者
    pub author: String,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// 親コメントID
    pub parent_id: Option<String>,
    /// リアクション
    pub reactions: Option<HashMap<String, u64>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 提案履歴
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalHistory {
    /// 日時
    pub timestamp: DateTime<Utc>,
    /// アクション
    pub action: String,
    /// アクター
    pub actor: String,
    /// 前のステータス
    pub previous_status: Option<ProposalStatus>,
    /// 新しいステータス
    pub new_status: Option<ProposalStatus>,
    /// 説明
    pub description: Option<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl Proposal {
    /// 新しい提案を作成
    pub fn new(
        id: String,
        title: String,
        description: String,
        proposal_type: ProposalType,
        author: String,
    ) -> Self {
        let now = Utc::now();

        Self {
            id,
            title,
            description,
            proposal_type,
            status: ProposalStatus::Draft,
            metadata: ProposalMetadata {
                author,
                created_at: now,
                updated_at: now,
                submitted_at: None,
                voting_started_at: None,
                voting_ended_at: None,
                executed_at: None,
                tags: Vec::new(),
                category: None,
                priority: None,
                difficulty: None,
                impact: None,
                additional_properties: HashMap::new(),
            },
            options: ProposalOptions::default(),
            votes: HashMap::new(),
            voting_result: None,
            execution_data: None,
            execution_result: None,
            attachments: None,
            comments: None,
            history: Some(vec![ProposalHistory {
                timestamp: now,
                action: "create".to_string(),
                actor: "system".to_string(),
                previous_status: None,
                new_status: Some(ProposalStatus::Draft),
                description: Some("Proposal created".to_string()),
                additional_properties: HashMap::new(),
            }]),
            related_proposals: None,
            additional_properties: HashMap::new(),
        }
    }

    /// 提案を提出
    pub fn submit(&mut self) -> Result<(), Error> {
        if self.status != ProposalStatus::Draft {
            return Err(Error::InvalidState(format!(
                "Cannot submit proposal in state: {:?}",
                self.status
            )));
        }

        let now = Utc::now();
        let previous_status = self.status.clone();

        self.status = ProposalStatus::Submitted;
        self.metadata.submitted_at = Some(now);
        self.metadata.updated_at = now;

        // 履歴を追加
        if let Some(history) = &mut self.history {
            history.push(ProposalHistory {
                timestamp: now,
                action: "submit".to_string(),
                actor: "system".to_string(),
                previous_status: Some(previous_status),
                new_status: Some(self.status.clone()),
                description: Some("Proposal submitted".to_string()),
                additional_properties: HashMap::new(),
            });
        }

        Ok(())
    }

    /// 投票を開始
    pub fn start_voting(&mut self) -> Result<(), Error> {
        if self.status != ProposalStatus::Submitted
            && self.status != ProposalStatus::UnderConsideration
        {
            return Err(Error::InvalidState(format!(
                "Cannot start voting for proposal in state: {:?}",
                self.status
            )));
        }

        let now = Utc::now();
        let previous_status = self.status.clone();

        self.status = ProposalStatus::Voting;
        self.metadata.voting_started_at = Some(now);
        self.metadata.updated_at = now;

        // 投票終了日時を計算
        if let Some(VotingPeriod::Duration(duration)) = &self.options.voting_period {
            self.metadata.voting_ended_at = Some(now + *duration);
        } else if let Some(VotingPeriod::EndTime(end_time)) = &self.options.voting_period {
            self.metadata.voting_ended_at = Some(*end_time);
        }

        // 履歴を追加
        if let Some(history) = &mut self.history {
            history.push(ProposalHistory {
                timestamp: now,
                action: "start_voting".to_string(),
                actor: "system".to_string(),
                previous_status: Some(previous_status),
                new_status: Some(self.status.clone()),
                description: Some("Voting started".to_string()),
                additional_properties: HashMap::new(),
            });
        }

        Ok(())
    }

    /// 投票を追加
    pub fn add_vote(&mut self, voter: String, vote: Vote) -> Result<(), Error> {
        if self.status != ProposalStatus::Voting {
            return Err(Error::InvalidState(format!(
                "Cannot vote on proposal in state: {:?}",
                self.status
            )));
        }

        // 投票期間をチェック
        if let Some(end_time) = self.metadata.voting_ended_at {
            if Utc::now() > end_time {
                return Err(Error::InvalidState("Voting period has ended".to_string()));
            }
        }

        // 投票を追加
        self.votes.insert(voter, vote);
        self.metadata.updated_at = Utc::now();

        // 早期終了条件をチェック
        if let Some(true) = self.options.early_execution {
            if let Some(threshold) = self.options.early_execution_threshold {
                self.calculate_voting_result()?;

                if let Some(result) = &self.voting_result {
                    if result.approval_ratio >= threshold {
                        self.end_voting()?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 投票結果を計算
    pub fn calculate_voting_result(&mut self) -> Result<(), Error> {
        if self.votes.is_empty() {
            return Ok(());
        }

        let mut total_votes = 0;
        let mut yes_votes = 0;
        let mut no_votes = 0;
        let mut abstain_votes = 0;

        // 投票を集計
        for (_, vote) in &self.votes {
            match vote.vote_type {
                crate::governance::voting::VoteType::Yes => {
                    yes_votes += vote.power.value;
                }
                crate::governance::voting::VoteType::No => {
                    no_votes += vote.power.value;
                }
                crate::governance::voting::VoteType::Abstain => {
                    abstain_votes += vote.power.value;
                }
            }

            total_votes += vote.power.value;
        }

        // 投票結果を作成
        let approval_ratio = if total_votes > 0 {
            yes_votes as f64 / total_votes as f64
        } else {
            0.0
        };

        let participation_ratio = 0.0; // 実際の実装では、総投票権に対する投票率を計算

        let quorum_reached = if let Some(quorum) = self.options.quorum {
            participation_ratio >= quorum
        } else {
            true
        };

        let threshold_reached = if let Some(threshold) = self.options.threshold {
            approval_ratio >= threshold
        } else {
            approval_ratio > 0.5
        };

        let min_votes_reached = if let Some(min_votes) = self.options.min_votes {
            self.votes.len() as u64 >= min_votes
        } else {
            true
        };

        let min_participation_reached =
            if let Some(min_participation) = self.options.min_participation {
                participation_ratio >= min_participation
            } else {
                true
            };

        let passed =
            quorum_reached && threshold_reached && min_votes_reached && min_participation_reached;

        self.voting_result = Some(VotingResult {
            total_votes,
            yes_votes,
            no_votes,
            abstain_votes,
            approval_ratio,
            participation_ratio,
            quorum_reached,
            threshold_reached,
            min_votes_reached,
            min_participation_reached,
            passed,
            additional_properties: HashMap::new(),
        });

        Ok(())
    }

    /// 投票を終了
    pub fn end_voting(&mut self) -> Result<(), Error> {
        if self.status != ProposalStatus::Voting {
            return Err(Error::InvalidState(format!(
                "Cannot end voting for proposal in state: {:?}",
                self.status
            )));
        }

        let now = Utc::now();
        let previous_status = self.status.clone();

        // 投票結果を計算
        self.calculate_voting_result()?;

        // 提案のステータスを更新
        if let Some(result) = &self.voting_result {
            if result.passed {
                self.status = ProposalStatus::Accepted;
            } else {
                self.status = ProposalStatus::Rejected;
            }
        } else {
            self.status = ProposalStatus::Rejected;
        }

        self.metadata.voting_ended_at = Some(now);
        self.metadata.updated_at = now;

        // 履歴を追加
        if let Some(history) = &mut self.history {
            history.push(ProposalHistory {
                timestamp: now,
                action: "end_voting".to_string(),
                actor: "system".to_string(),
                previous_status: Some(previous_status),
                new_status: Some(self.status.clone()),
                description: Some(format!("Voting ended with result: {:?}", self.status)),
                additional_properties: HashMap::new(),
            });
        }

        Ok(())
    }

    /// 提案を実行
    pub fn execute(&mut self) -> Result<(), Error> {
        if self.status != ProposalStatus::Accepted {
            return Err(Error::InvalidState(format!(
                "Cannot execute proposal in state: {:?}",
                self.status
            )));
        }

        let now = Utc::now();
        let previous_status = self.status.clone();

        // 遅延実行をチェック
        if let Some(true) = self.options.delayed_execution {
            if let Some(delay_seconds) = self.options.delayed_execution_seconds {
                if let Some(voting_ended_at) = self.metadata.voting_ended_at {
                    let delay_end = voting_ended_at + Duration::seconds(delay_seconds as i64);
                    if now < delay_end {
                        return Err(Error::InvalidState(format!(
                            "Execution is delayed until {}",
                            delay_end
                        )));
                    }
                }
            }
        }

        // 提案を実行
        self.status = ProposalStatus::Executing;
        self.metadata.updated_at = now;

        // 履歴を追加
        if let Some(history) = &mut self.history {
            history.push(ProposalHistory {
                timestamp: now,
                action: "execute".to_string(),
                actor: "system".to_string(),
                previous_status: Some(previous_status),
                new_status: Some(self.status.clone()),
                description: Some("Proposal execution started".to_string()),
                additional_properties: HashMap::new(),
            });
        }

        // 実際の実装では、提案タイプに応じた実行ロジックを実装
        // ここでは簡易的に成功したとみなす
        self.complete_execution(true, Some(serde_json::json!({"result": "success"})))
    }

    /// 実行を完了
    pub fn complete_execution(
        &mut self,
        success: bool,
        result: Option<serde_json::Value>,
    ) -> Result<(), Error> {
        if self.status != ProposalStatus::Executing {
            return Err(Error::InvalidState(format!(
                "Cannot complete execution for proposal in state: {:?}",
                self.status
            )));
        }

        let now = Utc::now();
        let previous_status = self.status.clone();

        // 実行結果を設定
        self.execution_result = result;

        // 提案のステータスを更新
        if success {
            self.status = ProposalStatus::Executed;
        } else {
            self.status = ProposalStatus::Failed;
        }

        self.metadata.executed_at = Some(now);
        self.metadata.updated_at = now;

        // 履歴を追加
        if let Some(history) = &mut self.history {
            history.push(ProposalHistory {
                timestamp: now,
                action: "complete_execution".to_string(),
                actor: "system".to_string(),
                previous_status: Some(previous_status),
                new_status: Some(self.status.clone()),
                description: Some(format!(
                    "Execution completed with status: {:?}",
                    self.status
                )),
                additional_properties: HashMap::new(),
            });
        }

        Ok(())
    }

    /// 提案をキャンセル
    pub fn cancel(&mut self, reason: Option<String>) -> Result<(), Error> {
        if self.status == ProposalStatus::Executed
            || self.status == ProposalStatus::Failed
            || self.status == ProposalStatus::Cancelled
            || self.status == ProposalStatus::Expired
        {
            return Err(Error::InvalidState(format!(
                "Cannot cancel proposal in state: {:?}",
                self.status
            )));
        }

        let now = Utc::now();
        let previous_status = self.status.clone();

        self.status = ProposalStatus::Cancelled;
        self.metadata.updated_at = now;

        // 履歴を追加
        if let Some(history) = &mut self.history {
            history.push(ProposalHistory {
                timestamp: now,
                action: "cancel".to_string(),
                actor: "system".to_string(),
                previous_status: Some(previous_status),
                new_status: Some(self.status.clone()),
                description: reason.or_else(|| Some("Proposal cancelled".to_string())),
                additional_properties: HashMap::new(),
            });
        }

        Ok(())
    }

    /// コメントを追加
    pub fn add_comment(&mut self, content: String, author: String) -> Result<(), Error> {
        let now = Utc::now();

        let comment = Comment {
            id: format!("comment_{}", Utc::now().timestamp_nanos()),
            content,
            author,
            created_at: now,
            updated_at: now,
            parent_id: None,
            reactions: None,
            additional_properties: HashMap::new(),
        };

        if self.comments.is_none() {
            self.comments = Some(Vec::new());
        }

        if let Some(comments) = &mut self.comments {
            comments.push(comment);
        }

        self.metadata.updated_at = now;

        Ok(())
    }

    /// 添付ファイルを追加
    pub fn add_attachment(&mut self, attachment: Attachment) -> Result<(), Error> {
        if self.attachments.is_none() {
            self.attachments = Some(Vec::new());
        }

        if let Some(attachments) = &mut self.attachments {
            attachments.push(attachment);
        }

        self.metadata.updated_at = Utc::now();

        Ok(())
    }

    /// 関連提案を追加
    pub fn add_related_proposal(&mut self, proposal_id: String) -> Result<(), Error> {
        if self.related_proposals.is_none() {
            self.related_proposals = Some(Vec::new());
        }

        if let Some(related_proposals) = &mut self.related_proposals {
            if !related_proposals.contains(&proposal_id) {
                related_proposals.push(proposal_id);
            }
        }

        self.metadata.updated_at = Utc::now();

        Ok(())
    }

    /// タグを追加
    pub fn add_tag(&mut self, tag: String) -> Result<(), Error> {
        if !self.metadata.tags.contains(&tag) {
            self.metadata.tags.push(tag);
        }

        self.metadata.updated_at = Utc::now();

        Ok(())
    }

    /// カテゴリを設定
    pub fn set_category(&mut self, category: String) -> Result<(), Error> {
        self.metadata.category = Some(category);
        self.metadata.updated_at = Utc::now();

        Ok(())
    }

    /// 優先度を設定
    pub fn set_priority(&mut self, priority: String) -> Result<(), Error> {
        self.metadata.priority = Some(priority);
        self.metadata.updated_at = Utc::now();

        Ok(())
    }

    /// 難易度を設定
    pub fn set_difficulty(&mut self, difficulty: String) -> Result<(), Error> {
        self.metadata.difficulty = Some(difficulty);
        self.metadata.updated_at = Utc::now();

        Ok(())
    }

    /// 影響度を設定
    pub fn set_impact(&mut self, impact: String) -> Result<(), Error> {
        self.metadata.impact = Some(impact);
        self.metadata.updated_at = Utc::now();

        Ok(())
    }
}
