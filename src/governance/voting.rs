use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;

/// 投票タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VoteType {
    /// 賛成
    Yes,
    /// 反対
    No,
    /// 棄権
    Abstain,
}

/// 投票パワー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingPower {
    /// 値
    pub value: u64,
    /// 単位
    pub unit: String,
    /// ソース
    pub source: Option<String>,
    /// 計算方法
    pub calculation: Option<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl VotingPower {
    /// 新しい投票パワーを作成
    pub fn new(value: u64) -> Self {
        Self {
            value,
            unit: "vote".to_string(),
            source: None,
            calculation: None,
            additional_properties: HashMap::new(),
        }
    }
    
    /// 単位を設定
    pub fn with_unit(mut self, unit: String) -> Self {
        self.unit = unit;
        self
    }
    
    /// ソースを設定
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }
    
    /// 計算方法を設定
    pub fn with_calculation(mut self, calculation: String) -> Self {
        self.calculation = Some(calculation);
        self
    }
}

/// 投票戦略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VotingStrategy {
    /// 単純多数決
    Simple,
    /// 絶対多数決
    Absolute,
    /// 加重投票
    Weighted,
    /// 二重多数決
    DoubleMajority,
    /// 承認投票
    Approval,
    /// 優先順位投票
    RankedChoice,
    /// 累積投票
    Cumulative,
    /// 二項投票
    Binary,
    /// 確信度投票
    Conviction,
    /// 二次投票
    Quadratic,
    /// 液体民主主義
    Liquid,
    /// フターキー
    Futarchy,
    /// カスタム
    Custom(String),
}

/// 投票期間
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingPeriod {
    /// 期間
    Duration(Duration),
    /// 終了時刻
    EndTime(DateTime<Utc>),
}

/// 投票結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    /// 総投票数
    pub total_votes: u64,
    /// 賛成票
    pub yes_votes: u64,
    /// 反対票
    pub no_votes: u64,
    /// 棄権票
    pub abstain_votes: u64,
    /// 承認率
    pub approval_ratio: f64,
    /// 参加率
    pub participation_ratio: f64,
    /// クォーラム達成フラグ
    pub quorum_reached: bool,
    /// 閾値達成フラグ
    pub threshold_reached: bool,
    /// 最小投票数達成フラグ
    pub min_votes_reached: bool,
    /// 最小参加率達成フラグ
    pub min_participation_reached: bool,
    /// 可決フラグ
    pub passed: bool,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 投票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// 投票タイプ
    pub vote_type: VoteType,
    /// 投票パワー
    pub power: VotingPower,
    /// 投票日時
    pub timestamp: DateTime<Utc>,
    /// 理由
    pub reason: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl Vote {
    /// 新しい投票を作成
    pub fn new(vote_type: VoteType, power: VotingPower) -> Self {
        Self {
            vote_type,
            power,
            timestamp: Utc::now(),
            reason: None,
            metadata: None,
            additional_properties: HashMap::new(),
        }
    }
    
    /// 理由を設定
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
    
    /// メタデータを設定
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// 投票システム
pub trait VotingSystem {
    /// 投票を追加
    fn add_vote(&mut self, voter: String, vote: Vote) -> Result<(), Error>;
    
    /// 投票結果を計算
    fn calculate_result(&self) -> Result<VotingResult, Error>;
    
    /// 投票が可決されたか確認
    fn is_passed(&self) -> Result<bool, Error>;
    
    /// 投票が終了したか確認
    fn is_finished(&self) -> Result<bool, Error>;
    
    /// 投票期間を取得
    fn get_voting_period(&self) -> VotingPeriod;
    
    /// 投票戦略を取得
    fn get_voting_strategy(&self) -> VotingStrategy;
    
    /// 投票を取得
    fn get_votes(&self) -> HashMap<String, Vote>;
    
    /// 投票者を取得
    fn get_voters(&self) -> Vec<String>;
    
    /// 投票数を取得
    fn get_vote_count(&self) -> u64;
    
    /// 投票パワーを取得
    fn get_total_voting_power(&self) -> u64;
}

/// 単純投票システム
pub struct SimpleVotingSystem {
    /// 投票
    votes: HashMap<String, Vote>,
    /// 投票期間
    voting_period: VotingPeriod,
    /// クォーラム
    quorum: f64,
    /// 閾値
    threshold: f64,
    /// 最小投票数
    min_votes: u64,
    /// 最小参加率
    min_participation: f64,
    /// 総投票権
    total_voting_power: u64,
}

impl SimpleVotingSystem {
    /// 新しい単純投票システムを作成
    pub fn new(
        voting_period: VotingPeriod,
        quorum: f64,
        threshold: f64,
        min_votes: u64,
        min_participation: f64,
        total_voting_power: u64,
    ) -> Self {
        Self {
            votes: HashMap::new(),
            voting_period,
            quorum,
            threshold,
            min_votes,
            min_participation,
            total_voting_power,
        }
    }
}

impl VotingSystem for SimpleVotingSystem {
    fn add_vote(&mut self, voter: String, vote: Vote) -> Result<(), Error> {
        // 投票期間をチェック
        if self.is_finished()? {
            return Err(Error::InvalidState("Voting period has ended".to_string()));
        }
        
        // 投票を追加
        self.votes.insert(voter, vote);
        
        Ok(())
    }
    
    fn calculate_result(&self) -> Result<VotingResult, Error> {
        let mut total_votes = 0;
        let mut yes_votes = 0;
        let mut no_votes = 0;
        let mut abstain_votes = 0;
        
        // 投票を集計
        for (_, vote) in &self.votes {
            match vote.vote_type {
                VoteType::Yes => {
                    yes_votes += vote.power.value;
                },
                VoteType::No => {
                    no_votes += vote.power.value;
                },
                VoteType::Abstain => {
                    abstain_votes += vote.power.value;
                },
            }
            
            total_votes += vote.power.value;
        }
        
        // 投票結果を作成
        let approval_ratio = if total_votes > 0 {
            yes_votes as f64 / total_votes as f64
        } else {
            0.0
        };
        
        let participation_ratio = if self.total_voting_power > 0 {
            total_votes as f64 / self.total_voting_power as f64
        } else {
            0.0
        };
        
        let quorum_reached = participation_ratio >= self.quorum;
        let threshold_reached = approval_ratio >= self.threshold;
        let min_votes_reached = self.votes.len() as u64 >= self.min_votes;
        let min_participation_reached = participation_ratio >= self.min_participation;
        
        let passed = quorum_reached && threshold_reached && min_votes_reached && min_participation_reached;
        
        Ok(VotingResult {
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
        })
    }
    
    fn is_passed(&self) -> Result<bool, Error> {
        let result = self.calculate_result()?;
        Ok(result.passed)
    }
    
    fn is_finished(&self) -> Result<bool, Error> {
        match &self.voting_period {
            VotingPeriod::Duration(duration) => {
                // 実際の実装では、投票開始時刻を保存して、それからの経過時間をチェックする
                // ここでは簡易的に常にfalseを返す
                Ok(false)
            },
            VotingPeriod::EndTime(end_time) => {
                Ok(Utc::now() > *end_time)
            },
        }
    }
    
    fn get_voting_period(&self) -> VotingPeriod {
        self.voting_period.clone()
    }
    
    fn get_voting_strategy(&self) -> VotingStrategy {
        VotingStrategy::Simple
    }
    
    fn get_votes(&self) -> HashMap<String, Vote> {
        self.votes.clone()
    }
    
    fn get_voters(&self) -> Vec<String> {
        self.votes.keys().cloned().collect()
    }
    
    fn get_vote_count(&self) -> u64 {
        self.votes.len() as u64
    }
    
    fn get_total_voting_power(&self) -> u64 {
        self.total_voting_power
    }
}