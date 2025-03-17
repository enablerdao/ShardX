use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::governance::proposal::{Proposal, ProposalType, ProposalStatus, ProposalOptions};
use crate::governance::voting::{VotingStrategy, VotingPeriod, Vote};
use crate::governance::treasury::{Treasury, Asset};
use crate::governance::role::{Role, RoleAssignment};
use crate::governance::policy::{Policy, PolicyRule};

/// DAOタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DAOType {
    /// プロトコルDAO
    Protocol,
    /// サービスDAO
    Service,
    /// 投資DAO
    Investment,
    /// ソーシャルDAO
    Social,
    /// 慈善DAO
    Philanthropy,
    /// コレクターDAO
    Collector,
    /// メディアDAO
    Media,
    /// 研究開発DAO
    ResearchAndDevelopment,
    /// カスタムDAO
    Custom(String),
}

/// DAOメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAOMetadata {
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// ウェブサイト
    pub website: Option<String>,
    /// ロゴ
    pub logo: Option<String>,
    /// メールアドレス
    pub email: Option<String>,
    /// ソーシャルメディア
    pub social_media: Option<HashMap<String, String>>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// タグ
    pub tags: Vec<String>,
    /// カテゴリ
    pub category: Option<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// DAOオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAOOptions {
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
    /// 最小参加率
    pub min_participation: Option<f64>,
    /// 提案作成に必要なトークン
    pub proposal_token_requirement: Option<u64>,
    /// 提案作成に必要なロール
    pub proposal_role_requirement: Option<Vec<String>>,
    /// 提案作成に必要な評判
    pub proposal_reputation_requirement: Option<u64>,
    /// 提案作成に必要な期間（日）
    pub proposal_membership_days_requirement: Option<u64>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl Default for DAOOptions {
    fn default() -> Self {
        Self {
            voting_strategy: Some(VotingStrategy::Simple),
            voting_period: Some(VotingPeriod::Duration(chrono::Duration::days(7))),
            quorum: Some(0.4),
            threshold: Some(0.5),
            min_votes: Some(1),
            min_participation: Some(0.1),
            proposal_token_requirement: None,
            proposal_role_requirement: None,
            proposal_reputation_requirement: None,
            proposal_membership_days_requirement: None,
            additional_properties: HashMap::new(),
        }
    }
}

/// DAOメンバー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAOMember {
    /// ID
    pub id: String,
    /// 名前
    pub name: Option<String>,
    /// アドレス
    pub address: String,
    /// 参加日時
    pub joined_at: DateTime<Utc>,
    /// トークン保有量
    pub tokens: u64,
    /// 評判
    pub reputation: u64,
    /// ロール
    pub roles: Vec<String>,
    /// 投票権
    pub voting_power: u64,
    /// 委任先
    pub delegate: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// DAOロール
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAORole {
    /// ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// 権限
    pub permissions: Vec<DAOPermission>,
    /// メンバー
    pub members: Vec<String>,
    /// 親ロール
    pub parent_role: Option<String>,
    /// 子ロール
    pub child_roles: Vec<String>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// メタデータ
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// DAO権限
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DAOPermission {
    /// 提案作成
    CreateProposal,
    /// 提案投票
    VoteOnProposal,
    /// 提案実行
    ExecuteProposal,
    /// 提案キャンセル
    CancelProposal,
    /// メンバー追加
    AddMember,
    /// メンバー削除
    RemoveMember,
    /// ロール作成
    CreateRole,
    /// ロール削除
    DeleteRole,
    /// ロール割り当て
    AssignRole,
    /// ロール解除
    RevokeRole,
    /// 資金送金
    TransferFunds,
    /// 資金受領
    ReceiveFunds,
    /// ポリシー作成
    CreatePolicy,
    /// ポリシー削除
    DeletePolicy,
    /// ポリシー更新
    UpdatePolicy,
    /// 設定更新
    UpdateSettings,
    /// カスタム権限
    Custom(String),
}

/// DAO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAO {
    /// ID
    pub id: String,
    /// タイプ
    pub dao_type: DAOType,
    /// メタデータ
    pub metadata: DAOMetadata,
    /// オプション
    pub options: DAOOptions,
    /// メンバー
    pub members: HashMap<String, DAOMember>,
    /// 提案
    pub proposals: HashMap<String, Proposal>,
    /// ロール
    pub roles: HashMap<String, DAORole>,
    /// ポリシー
    pub policies: HashMap<String, Policy>,
    /// 財務
    pub treasury: Treasury,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl DAO {
    /// 新しいDAOを作成
    pub fn new(
        id: String,
        name: String,
        description: String,
        dao_type: DAOType,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: id.clone(),
            dao_type,
            metadata: DAOMetadata {
                name,
                description,
                website: None,
                logo: None,
                email: None,
                social_media: None,
                created_at: now,
                updated_at: now,
                tags: Vec::new(),
                category: None,
                additional_properties: HashMap::new(),
            },
            options: DAOOptions::default(),
            members: HashMap::new(),
            proposals: HashMap::new(),
            roles: HashMap::new(),
            policies: HashMap::new(),
            treasury: Treasury::new(id),
            additional_properties: HashMap::new(),
        }
    }
    
    /// メンバーを追加
    pub fn add_member(&mut self, member: DAOMember) -> Result<(), Error> {
        if self.members.contains_key(&member.id) {
            return Err(Error::AlreadyExists(format!("Member already exists: {}", member.id)));
        }
        
        self.members.insert(member.id.clone(), member);
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// メンバーを削除
    pub fn remove_member(&mut self, member_id: &str) -> Result<(), Error> {
        if !self.members.contains_key(member_id) {
            return Err(Error::NotFound(format!("Member not found: {}", member_id)));
        }
        
        self.members.remove(member_id);
        self.metadata.updated_at = Utc::now();
        
        // ロールからメンバーを削除
        for (_, role) in self.roles.iter_mut() {
            role.members.retain(|id| id != member_id);
        }
        
        Ok(())
    }
    
    /// 提案を作成
    pub fn create_proposal(
        &mut self,
        title: String,
        description: String,
        proposal_type: ProposalType,
        author: String,
        options: Option<ProposalOptions>,
    ) -> Result<String, Error> {
        // メンバーをチェック
        if !self.members.contains_key(&author) {
            return Err(Error::NotFound(format!("Member not found: {}", author)));
        }
        
        // 提案作成要件をチェック
        let member = self.members.get(&author).unwrap();
        
        if let Some(token_requirement) = self.options.proposal_token_requirement {
            if member.tokens < token_requirement {
                return Err(Error::InvalidState(format!("Insufficient tokens to create proposal: {} < {}", member.tokens, token_requirement)));
            }
        }
        
        if let Some(role_requirements) = &self.options.proposal_role_requirement {
            let has_required_role = role_requirements.iter().any(|role| member.roles.contains(role));
            if !has_required_role {
                return Err(Error::InvalidState(format!("Member does not have required role to create proposal")));
            }
        }
        
        if let Some(reputation_requirement) = self.options.proposal_reputation_requirement {
            if member.reputation < reputation_requirement {
                return Err(Error::InvalidState(format!("Insufficient reputation to create proposal: {} < {}", member.reputation, reputation_requirement)));
            }
        }
        
        if let Some(days_requirement) = self.options.proposal_membership_days_requirement {
            let membership_days = (Utc::now() - member.joined_at).num_days();
            if membership_days < days_requirement as i64 {
                return Err(Error::InvalidState(format!("Insufficient membership duration to create proposal: {} < {}", membership_days, days_requirement)));
            }
        }
        
        // 提案IDを生成
        let proposal_id = format!("proposal_{}", Utc::now().timestamp_nanos());
        
        // 提案オプションを設定
        let proposal_options = options.unwrap_or_else(|| {
            ProposalOptions {
                voting_strategy: self.options.voting_strategy.clone(),
                voting_period: self.options.voting_period.clone(),
                quorum: self.options.quorum,
                threshold: self.options.threshold,
                min_votes: self.options.min_votes,
                min_participation: self.options.min_participation,
                early_execution: Some(false),
                early_execution_threshold: Some(0.66),
                delayed_execution: Some(false),
                delayed_execution_seconds: Some(86400), // 1日
                veto_enabled: Some(false),
                veto_holders: None,
                additional_properties: HashMap::new(),
            }
        });
        
        // 提案を作成
        let proposal = Proposal::new(
            proposal_id.clone(),
            title,
            description,
            proposal_type,
            author,
        );
        
        // 提案を保存
        self.proposals.insert(proposal_id.clone(), proposal);
        self.metadata.updated_at = Utc::now();
        
        Ok(proposal_id)
    }
    
    /// 提案を取得
    pub fn get_proposal(&self, proposal_id: &str) -> Result<&Proposal, Error> {
        self.proposals.get(proposal_id).ok_or_else(|| {
            Error::NotFound(format!("Proposal not found: {}", proposal_id))
        })
    }
    
    /// 提案を取得（可変）
    pub fn get_proposal_mut(&mut self, proposal_id: &str) -> Result<&mut Proposal, Error> {
        self.proposals.get_mut(proposal_id).ok_or_else(|| {
            Error::NotFound(format!("Proposal not found: {}", proposal_id))
        })
    }
    
    /// 提案に投票
    pub fn vote_on_proposal(&mut self, proposal_id: &str, voter_id: &str, vote: Vote) -> Result<(), Error> {
        // メンバーをチェック
        if !self.members.contains_key(voter_id) {
            return Err(Error::NotFound(format!("Member not found: {}", voter_id)));
        }
        
        // 提案を取得
        let proposal = self.get_proposal_mut(proposal_id)?;
        
        // 投票を追加
        proposal.add_vote(voter_id.to_string(), vote)?;
        
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// 提案を実行
    pub fn execute_proposal(&mut self, proposal_id: &str) -> Result<(), Error> {
        // 提案を取得
        let proposal = self.get_proposal_mut(proposal_id)?;
        
        // 提案を実行
        proposal.execute()?;
        
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// ロールを作成
    pub fn create_role(&mut self, role: DAORole) -> Result<(), Error> {
        if self.roles.contains_key(&role.id) {
            return Err(Error::AlreadyExists(format!("Role already exists: {}", role.id)));
        }
        
        self.roles.insert(role.id.clone(), role);
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// ロールを削除
    pub fn delete_role(&mut self, role_id: &str) -> Result<(), Error> {
        if !self.roles.contains_key(role_id) {
            return Err(Error::NotFound(format!("Role not found: {}", role_id)));
        }
        
        // 子ロールをチェック
        let has_children = self.roles.values().any(|role| {
            role.parent_role.as_ref().map_or(false, |parent| parent == role_id)
        });
        
        if has_children {
            return Err(Error::InvalidState(format!("Cannot delete role with children: {}", role_id)));
        }
        
        // ロールを削除
        self.roles.remove(role_id);
        
        // 親ロールから子ロール参照を削除
        for (_, role) in self.roles.iter_mut() {
            if let Some(parent) = &role.parent_role {
                if parent == role_id {
                    role.parent_role = None;
                }
            }
            
            role.child_roles.retain(|id| id != role_id);
        }
        
        // メンバーからロールを削除
        for (_, member) in self.members.iter_mut() {
            member.roles.retain(|id| id != role_id);
        }
        
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// ロールを割り当て
    pub fn assign_role(&mut self, member_id: &str, role_id: &str) -> Result<(), Error> {
        // メンバーをチェック
        if !self.members.contains_key(member_id) {
            return Err(Error::NotFound(format!("Member not found: {}", member_id)));
        }
        
        // ロールをチェック
        if !self.roles.contains_key(role_id) {
            return Err(Error::NotFound(format!("Role not found: {}", role_id)));
        }
        
        // ロールを割り当て
        let member = self.members.get_mut(member_id).unwrap();
        if !member.roles.contains(&role_id.to_string()) {
            member.roles.push(role_id.to_string());
        }
        
        // ロールにメンバーを追加
        let role = self.roles.get_mut(role_id).unwrap();
        if !role.members.contains(&member_id.to_string()) {
            role.members.push(member_id.to_string());
        }
        
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// ロールを解除
    pub fn revoke_role(&mut self, member_id: &str, role_id: &str) -> Result<(), Error> {
        // メンバーをチェック
        if !self.members.contains_key(member_id) {
            return Err(Error::NotFound(format!("Member not found: {}", member_id)));
        }
        
        // ロールをチェック
        if !self.roles.contains_key(role_id) {
            return Err(Error::NotFound(format!("Role not found: {}", role_id)));
        }
        
        // ロールを解除
        let member = self.members.get_mut(member_id).unwrap();
        member.roles.retain(|id| id != role_id);
        
        // ロールからメンバーを削除
        let role = self.roles.get_mut(role_id).unwrap();
        role.members.retain(|id| id != member_id);
        
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// ポリシーを作成
    pub fn create_policy(&mut self, policy: Policy) -> Result<(), Error> {
        if self.policies.contains_key(&policy.id) {
            return Err(Error::AlreadyExists(format!("Policy already exists: {}", policy.id)));
        }
        
        self.policies.insert(policy.id.clone(), policy);
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// ポリシーを削除
    pub fn delete_policy(&mut self, policy_id: &str) -> Result<(), Error> {
        if !self.policies.contains_key(policy_id) {
            return Err(Error::NotFound(format!("Policy not found: {}", policy_id)));
        }
        
        self.policies.remove(policy_id);
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// 資産を追加
    pub fn add_asset(&mut self, asset: Asset) -> Result<(), Error> {
        self.treasury.add_asset(asset)?;
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// 資産を送金
    pub fn transfer_asset(&mut self, asset_id: &str, to: &str, amount: u64) -> Result<(), Error> {
        self.treasury.transfer(asset_id, to, amount)?;
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// メンバーの投票権を計算
    pub fn calculate_voting_power(&self, member_id: &str) -> Result<u64, Error> {
        let member = self.members.get(member_id).ok_or_else(|| {
            Error::NotFound(format!("Member not found: {}", member_id))
        })?;
        
        // 実際の実装では、投票戦略に応じた投票権計算ロジックを実装
        // ここでは簡易的にトークン保有量を返す
        Ok(member.tokens)
    }
    
    /// メンバーが権限を持っているか確認
    pub fn has_permission(&self, member_id: &str, permission: &DAOPermission) -> Result<bool, Error> {
        let member = self.members.get(member_id).ok_or_else(|| {
            Error::NotFound(format!("Member not found: {}", member_id))
        })?;
        
        // メンバーのロールをチェック
        for role_id in &member.roles {
            if let Some(role) = self.roles.get(role_id) {
                if role.permissions.contains(permission) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// メンバーの評判を更新
    pub fn update_reputation(&mut self, member_id: &str, reputation: u64) -> Result<(), Error> {
        let member = self.members.get_mut(member_id).ok_or_else(|| {
            Error::NotFound(format!("Member not found: {}", member_id))
        })?;
        
        member.reputation = reputation;
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// メンバーのトークンを更新
    pub fn update_tokens(&mut self, member_id: &str, tokens: u64) -> Result<(), Error> {
        let member = self.members.get_mut(member_id).ok_or_else(|| {
            Error::NotFound(format!("Member not found: {}", member_id))
        })?;
        
        member.tokens = tokens;
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// メンバーの委任先を設定
    pub fn set_delegate(&mut self, member_id: &str, delegate_id: Option<String>) -> Result<(), Error> {
        let member = self.members.get_mut(member_id).ok_or_else(|| {
            Error::NotFound(format!("Member not found: {}", member_id))
        })?;
        
        // 委任先をチェック
        if let Some(delegate_id) = &delegate_id {
            if !self.members.contains_key(delegate_id) {
                return Err(Error::NotFound(format!("Delegate not found: {}", delegate_id)));
            }
            
            // 循環委任をチェック
            let mut current_delegate = delegate_id;
            let mut visited = vec![member_id.to_string()];
            
            while let Some(delegate) = self.members.get(current_delegate) {
                if let Some(next_delegate) = &delegate.delegate {
                    if visited.contains(next_delegate) {
                        return Err(Error::InvalidState(format!("Circular delegation detected")));
                    }
                    
                    visited.push(next_delegate.clone());
                    current_delegate = next_delegate;
                } else {
                    break;
                }
            }
        }
        
        member.delegate = delegate_id;
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// DAOの設定を更新
    pub fn update_options(&mut self, options: DAOOptions) -> Result<(), Error> {
        self.options = options;
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// DAOのメタデータを更新
    pub fn update_metadata(&mut self, metadata: DAOMetadata) -> Result<(), Error> {
        let created_at = self.metadata.created_at;
        self.metadata = metadata;
        self.metadata.created_at = created_at;
        self.metadata.updated_at = Utc::now();
        
        Ok(())
    }
}