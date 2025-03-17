use crate::error::Error;
use crate::governance::{
    Proposal, ProposalStatus, ProposalType, Vote, VotingPower, 
    DAO, DAORole, DAOMember, Treasury, TreasuryTransaction
};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};
use uuid::Uuid;

/// コミュニティツール設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunityToolsConfig {
    /// フォーラム有効フラグ
    pub enable_forum: bool,
    /// 投票ダッシュボード有効フラグ
    pub enable_voting_dashboard: bool,
    /// 財務ダッシュボード有効フラグ
    pub enable_treasury_dashboard: bool,
    /// メンバーディレクトリ有効フラグ
    pub enable_member_directory: bool,
    /// タスク管理有効フラグ
    pub enable_task_management: bool,
    /// カレンダー有効フラグ
    pub enable_calendar: bool,
    /// 通知有効フラグ
    pub enable_notifications: bool,
    /// 分析有効フラグ
    pub enable_analytics: bool,
    /// 多言語サポート有効フラグ
    pub enable_multilingual: bool,
    /// サポートされている言語
    pub supported_languages: Vec<String>,
}

/// フォーラムカテゴリ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForumCategory {
    /// カテゴリID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// 親カテゴリID
    pub parent_id: Option<String>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// フォーラムトピック
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForumTopic {
    /// トピックID
    pub id: String,
    /// タイトル
    pub title: String,
    /// 内容
    pub content: String,
    /// 作成者
    pub author: String,
    /// カテゴリID
    pub category_id: String,
    /// タグ
    pub tags: Vec<String>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// 閲覧数
    pub view_count: u64,
    /// 返信数
    pub reply_count: u64,
    /// いいね数
    pub like_count: u64,
    /// ピン留めフラグ
    pub is_pinned: bool,
    /// ロックフラグ
    pub is_locked: bool,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// フォーラム返信
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForumReply {
    /// 返信ID
    pub id: String,
    /// トピックID
    pub topic_id: String,
    /// 内容
    pub content: String,
    /// 作成者
    pub author: String,
    /// 親返信ID
    pub parent_id: Option<String>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// いいね数
    pub like_count: u64,
    /// ベストアンサーフラグ
    pub is_best_answer: bool,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// タスク
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    /// タスクID
    pub id: String,
    /// タイトル
    pub title: String,
    /// 説明
    pub description: String,
    /// 作成者
    pub creator: String,
    /// 担当者
    pub assignee: Option<String>,
    /// ステータス
    pub status: TaskStatus,
    /// 優先度
    pub priority: TaskPriority,
    /// 期限
    pub due_date: Option<DateTime<Utc>>,
    /// タグ
    pub tags: Vec<String>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// 完了日時
    pub completed_at: Option<DateTime<Utc>>,
    /// 関連提案ID
    pub related_proposal_id: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// タスクステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 未着手
    Todo,
    /// 進行中
    InProgress,
    /// レビュー中
    InReview,
    /// 完了
    Done,
    /// キャンセル
    Cancelled,
}

/// タスク優先度
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 緊急
    Urgent,
}

/// カレンダーイベント
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    /// イベントID
    pub id: String,
    /// タイトル
    pub title: String,
    /// 説明
    pub description: String,
    /// 作成者
    pub creator: String,
    /// 開始日時
    pub start_time: DateTime<Utc>,
    /// 終了日時
    pub end_time: DateTime<Utc>,
    /// 場所
    pub location: Option<String>,
    /// 参加者
    pub participants: Vec<String>,
    /// 繰り返し設定
    pub recurrence: Option<String>,
    /// 通知設定
    pub reminders: Vec<i64>,
    /// タグ
    pub tags: Vec<String>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// 関連提案ID
    pub related_proposal_id: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 通知
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    /// 通知ID
    pub id: String,
    /// 受信者
    pub recipient: String,
    /// タイトル
    pub title: String,
    /// 内容
    pub content: String,
    /// 通知タイプ
    pub notification_type: NotificationType,
    /// 関連ID
    pub related_id: Option<String>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 既読フラグ
    pub is_read: bool,
    /// 既読日時
    pub read_at: Option<DateTime<Utc>>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 通知タイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    /// 提案
    Proposal,
    /// 投票
    Vote,
    /// フォーラム
    Forum,
    /// タスク
    Task,
    /// カレンダー
    Calendar,
    /// システム
    System,
}

/// 分析データ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyticsData {
    /// データID
    pub id: String,
    /// データタイプ
    pub data_type: AnalyticsDataType,
    /// 期間
    pub period: String,
    /// データ
    pub data: serde_json::Value,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 分析データタイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalyticsDataType {
    /// 提案統計
    ProposalStats,
    /// 投票統計
    VotingStats,
    /// 参加統計
    ParticipationStats,
    /// 財務統計
    TreasuryStats,
    /// フォーラム統計
    ForumStats,
    /// タスク統計
    TaskStats,
}

/// コミュニティツールマネージャー
pub struct CommunityToolsManager {
    /// 設定
    config: CommunityToolsConfig,
    /// フォーラムカテゴリ
    forum_categories: HashMap<String, ForumCategory>,
    /// フォーラムトピック
    forum_topics: HashMap<String, ForumTopic>,
    /// フォーラム返信
    forum_replies: HashMap<String, ForumReply>,
    /// タスク
    tasks: HashMap<String, Task>,
    /// カレンダーイベント
    calendar_events: HashMap<String, CalendarEvent>,
    /// 通知
    notifications: HashMap<String, Vec<Notification>>,
    /// 分析データ
    analytics_data: HashMap<String, AnalyticsData>,
}

impl CommunityToolsManager {
    /// 新しいCommunityToolsManagerを作成
    pub fn new(config: CommunityToolsConfig) -> Self {
        Self {
            config,
            forum_categories: HashMap::new(),
            forum_topics: HashMap::new(),
            forum_replies: HashMap::new(),
            tasks: HashMap::new(),
            calendar_events: HashMap::new(),
            notifications: HashMap::new(),
            analytics_data: HashMap::new(),
        }
    }
    
    /// フォーラムカテゴリを作成
    pub fn create_forum_category(
        &mut self,
        name: &str,
        description: &str,
        parent_id: Option<&str>,
    ) -> Result<String, Error> {
        // フォーラムが有効かチェック
        if !self.config.enable_forum {
            return Err(Error::InvalidState("Forum is not enabled".to_string()));
        }
        
        // 親カテゴリが存在するかチェック
        if let Some(parent_id) = parent_id {
            if !self.forum_categories.contains_key(parent_id) {
                return Err(Error::NotFound(format!("Parent category not found: {}", parent_id)));
            }
        }
        
        // カテゴリIDを生成
        let category_id = Uuid::new_v4().to_string();
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // カテゴリを作成
        let category = ForumCategory {
            id: category_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            parent_id: parent_id.map(|id| id.to_string()),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        };
        
        // カテゴリを保存
        self.forum_categories.insert(category_id.clone(), category);
        
        info!("Forum category created: {} ({})", name, category_id);
        
        Ok(category_id)
    }
    
    /// フォーラムトピックを作成
    pub fn create_forum_topic(
        &mut self,
        title: &str,
        content: &str,
        author: &str,
        category_id: &str,
        tags: Vec<String>,
    ) -> Result<String, Error> {
        // フォーラムが有効かチェック
        if !self.config.enable_forum {
            return Err(Error::InvalidState("Forum is not enabled".to_string()));
        }
        
        // カテゴリが存在するかチェック
        if !self.forum_categories.contains_key(category_id) {
            return Err(Error::NotFound(format!("Category not found: {}", category_id)));
        }
        
        // トピックIDを生成
        let topic_id = Uuid::new_v4().to_string();
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // トピックを作成
        let topic = ForumTopic {
            id: topic_id.clone(),
            title: title.to_string(),
            content: content.to_string(),
            author: author.to_string(),
            category_id: category_id.to_string(),
            tags,
            created_at: now,
            updated_at: now,
            view_count: 0,
            reply_count: 0,
            like_count: 0,
            is_pinned: false,
            is_locked: false,
            metadata: HashMap::new(),
        };
        
        // トピックを保存
        self.forum_topics.insert(topic_id.clone(), topic);
        
        info!("Forum topic created: {} ({})", title, topic_id);
        
        Ok(topic_id)
    }
    
    /// フォーラム返信を作成
    pub fn create_forum_reply(
        &mut self,
        topic_id: &str,
        content: &str,
        author: &str,
        parent_id: Option<&str>,
    ) -> Result<String, Error> {
        // フォーラムが有効かチェック
        if !self.config.enable_forum {
            return Err(Error::InvalidState("Forum is not enabled".to_string()));
        }
        
        // トピックが存在するかチェック
        let topic = self.forum_topics.get_mut(topic_id)
            .ok_or_else(|| Error::NotFound(format!("Topic not found: {}", topic_id)))?;
        
        // トピックがロックされていないかチェック
        if topic.is_locked {
            return Err(Error::InvalidState(format!("Topic is locked: {}", topic_id)));
        }
        
        // 親返信が存在するかチェック
        if let Some(parent_id) = parent_id {
            if !self.forum_replies.contains_key(parent_id) {
                return Err(Error::NotFound(format!("Parent reply not found: {}", parent_id)));
            }
        }
        
        // 返信IDを生成
        let reply_id = Uuid::new_v4().to_string();
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // 返信を作成
        let reply = ForumReply {
            id: reply_id.clone(),
            topic_id: topic_id.to_string(),
            content: content.to_string(),
            author: author.to_string(),
            parent_id: parent_id.map(|id| id.to_string()),
            created_at: now,
            updated_at: now,
            like_count: 0,
            is_best_answer: false,
            metadata: HashMap::new(),
        };
        
        // 返信を保存
        self.forum_replies.insert(reply_id.clone(), reply);
        
        // トピックの返信数を更新
        topic.reply_count += 1;
        topic.updated_at = now;
        
        info!("Forum reply created: {} ({})", reply_id, topic_id);
        
        Ok(reply_id)
    }
    
    /// タスクを作成
    pub fn create_task(
        &mut self,
        title: &str,
        description: &str,
        creator: &str,
        assignee: Option<&str>,
        priority: TaskPriority,
        due_date: Option<DateTime<Utc>>,
        tags: Vec<String>,
        related_proposal_id: Option<&str>,
    ) -> Result<String, Error> {
        // タスク管理が有効かチェック
        if !self.config.enable_task_management {
            return Err(Error::InvalidState("Task management is not enabled".to_string()));
        }
        
        // タスクIDを生成
        let task_id = Uuid::new_v4().to_string();
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // タスクを作成
        let task = Task {
            id: task_id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            creator: creator.to_string(),
            assignee: assignee.map(|a| a.to_string()),
            status: TaskStatus::Todo,
            priority,
            due_date,
            tags,
            created_at: now,
            updated_at: now,
            completed_at: None,
            related_proposal_id: related_proposal_id.map(|id| id.to_string()),
            metadata: HashMap::new(),
        };
        
        // タスクを保存
        self.tasks.insert(task_id.clone(), task);
        
        info!("Task created: {} ({})", title, task_id);
        
        Ok(task_id)
    }
    
    /// タスクステータスを更新
    pub fn update_task_status(
        &mut self,
        task_id: &str,
        status: TaskStatus,
        updater: &str,
    ) -> Result<(), Error> {
        // タスク管理が有効かチェック
        if !self.config.enable_task_management {
            return Err(Error::InvalidState("Task management is not enabled".to_string()));
        }
        
        // タスクを取得
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task not found: {}", task_id)))?;
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // ステータスを更新
        task.status = status.clone();
        task.updated_at = now;
        
        // 完了ステータスの場合は完了日時を設定
        if status == TaskStatus::Done {
            task.completed_at = Some(now);
        } else {
            task.completed_at = None;
        }
        
        // メタデータを更新
        task.metadata.insert("last_updated_by".to_string(), updater.to_string());
        
        info!("Task status updated: {} -> {:?}", task_id, status);
        
        Ok(())
    }
    
    /// カレンダーイベントを作成
    pub fn create_calendar_event(
        &mut self,
        title: &str,
        description: &str,
        creator: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        location: Option<&str>,
        participants: Vec<String>,
        recurrence: Option<&str>,
        reminders: Vec<i64>,
        tags: Vec<String>,
        related_proposal_id: Option<&str>,
    ) -> Result<String, Error> {
        // カレンダーが有効かチェック
        if !self.config.enable_calendar {
            return Err(Error::InvalidState("Calendar is not enabled".to_string()));
        }
        
        // 開始時刻と終了時刻をチェック
        if end_time <= start_time {
            return Err(Error::InvalidArgument("End time must be after start time".to_string()));
        }
        
        // イベントIDを生成
        let event_id = Uuid::new_v4().to_string();
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // イベントを作成
        let event = CalendarEvent {
            id: event_id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            creator: creator.to_string(),
            start_time,
            end_time,
            location: location.map(|l| l.to_string()),
            participants,
            recurrence: recurrence.map(|r| r.to_string()),
            reminders,
            tags,
            created_at: now,
            updated_at: now,
            related_proposal_id: related_proposal_id.map(|id| id.to_string()),
            metadata: HashMap::new(),
        };
        
        // イベントを保存
        self.calendar_events.insert(event_id.clone(), event);
        
        info!("Calendar event created: {} ({})", title, event_id);
        
        Ok(event_id)
    }
    
    /// 通知を作成
    pub fn create_notification(
        &mut self,
        recipient: &str,
        title: &str,
        content: &str,
        notification_type: NotificationType,
        related_id: Option<&str>,
    ) -> Result<String, Error> {
        // 通知が有効かチェック
        if !self.config.enable_notifications {
            return Err(Error::InvalidState("Notifications are not enabled".to_string()));
        }
        
        // 通知IDを生成
        let notification_id = Uuid::new_v4().to_string();
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // 通知を作成
        let notification = Notification {
            id: notification_id.clone(),
            recipient: recipient.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            notification_type,
            related_id: related_id.map(|id| id.to_string()),
            created_at: now,
            is_read: false,
            read_at: None,
            metadata: HashMap::new(),
        };
        
        // 通知を保存
        let recipient_notifications = self.notifications
            .entry(recipient.to_string())
            .or_insert_with(Vec::new);
        
        recipient_notifications.push(notification);
        
        info!("Notification created: {} for {}", notification_id, recipient);
        
        Ok(notification_id)
    }
    
    /// 通知を既読にする
    pub fn mark_notification_as_read(
        &mut self,
        recipient: &str,
        notification_id: &str,
    ) -> Result<(), Error> {
        // 通知が有効かチェック
        if !self.config.enable_notifications {
            return Err(Error::InvalidState("Notifications are not enabled".to_string()));
        }
        
        // 受信者の通知を取得
        let recipient_notifications = self.notifications.get_mut(recipient)
            .ok_or_else(|| Error::NotFound(format!("No notifications found for recipient: {}", recipient)))?;
        
        // 通知を検索
        let notification = recipient_notifications.iter_mut()
            .find(|n| n.id == notification_id)
            .ok_or_else(|| Error::NotFound(format!("Notification not found: {}", notification_id)))?;
        
        // 既読にする
        notification.is_read = true;
        notification.read_at = Some(Utc::now());
        
        info!("Notification marked as read: {} for {}", notification_id, recipient);
        
        Ok(())
    }
    
    /// 分析データを記録
    pub fn record_analytics_data(
        &mut self,
        data_type: AnalyticsDataType,
        period: &str,
        data: serde_json::Value,
    ) -> Result<String, Error> {
        // 分析が有効かチェック
        if !self.config.enable_analytics {
            return Err(Error::InvalidState("Analytics are not enabled".to_string()));
        }
        
        // データIDを生成
        let data_id = Uuid::new_v4().to_string();
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // 分析データを作成
        let analytics_data = AnalyticsData {
            id: data_id.clone(),
            data_type,
            period: period.to_string(),
            data,
            created_at: now,
            metadata: HashMap::new(),
        };
        
        // 分析データを保存
        self.analytics_data.insert(data_id.clone(), analytics_data);
        
        info!("Analytics data recorded: {} ({:?})", data_id, data_type);
        
        Ok(data_id)
    }
    
    /// フォーラムカテゴリを取得
    pub fn get_forum_category(&self, category_id: &str) -> Result<&ForumCategory, Error> {
        // フォーラムが有効かチェック
        if !self.config.enable_forum {
            return Err(Error::InvalidState("Forum is not enabled".to_string()));
        }
        
        // カテゴリを取得
        self.forum_categories.get(category_id)
            .ok_or_else(|| Error::NotFound(format!("Category not found: {}", category_id)))
    }
    
    /// フォーラムカテゴリリストを取得
    pub fn get_forum_categories(&self) -> Result<Vec<&ForumCategory>, Error> {
        // フォーラムが有効かチェック
        if !self.config.enable_forum {
            return Err(Error::InvalidState("Forum is not enabled".to_string()));
        }
        
        // カテゴリリストを取得
        let categories: Vec<&ForumCategory> = self.forum_categories.values().collect();
        
        Ok(categories)
    }
    
    /// フォーラムトピックを取得
    pub fn get_forum_topic(&self, topic_id: &str) -> Result<&ForumTopic, Error> {
        // フォーラムが有効かチェック
        if !self.config.enable_forum {
            return Err(Error::InvalidState("Forum is not enabled".to_string()));
        }
        
        // トピックを取得
        self.forum_topics.get(topic_id)
            .ok_or_else(|| Error::NotFound(format!("Topic not found: {}", topic_id)))
    }
    
    /// カテゴリのフォーラムトピックリストを取得
    pub fn get_forum_topics_by_category(&self, category_id: &str) -> Result<Vec<&ForumTopic>, Error> {
        // フォーラムが有効かチェック
        if !self.config.enable_forum {
            return Err(Error::InvalidState("Forum is not enabled".to_string()));
        }
        
        // カテゴリが存在するかチェック
        if !self.forum_categories.contains_key(category_id) {
            return Err(Error::NotFound(format!("Category not found: {}", category_id)));
        }
        
        // カテゴリのトピックリストを取得
        let topics: Vec<&ForumTopic> = self.forum_topics.values()
            .filter(|t| t.category_id == category_id)
            .collect();
        
        Ok(topics)
    }
    
    /// フォーラム返信を取得
    pub fn get_forum_reply(&self, reply_id: &str) -> Result<&ForumReply, Error> {
        // フォーラムが有効かチェック
        if !self.config.enable_forum {
            return Err(Error::InvalidState("Forum is not enabled".to_string()));
        }
        
        // 返信を取得
        self.forum_replies.get(reply_id)
            .ok_or_else(|| Error::NotFound(format!("Reply not found: {}", reply_id)))
    }
    
    /// トピックのフォーラム返信リストを取得
    pub fn get_forum_replies_by_topic(&self, topic_id: &str) -> Result<Vec<&ForumReply>, Error> {
        // フォーラムが有効かチェック
        if !self.config.enable_forum {
            return Err(Error::InvalidState("Forum is not enabled".to_string()));
        }
        
        // トピックが存在するかチェック
        if !self.forum_topics.contains_key(topic_id) {
            return Err(Error::NotFound(format!("Topic not found: {}", topic_id)));
        }
        
        // トピックの返信リストを取得
        let replies: Vec<&ForumReply> = self.forum_replies.values()
            .filter(|r| r.topic_id == topic_id)
            .collect();
        
        Ok(replies)
    }
    
    /// タスクを取得
    pub fn get_task(&self, task_id: &str) -> Result<&Task, Error> {
        // タスク管理が有効かチェック
        if !self.config.enable_task_management {
            return Err(Error::InvalidState("Task management is not enabled".to_string()));
        }
        
        // タスクを取得
        self.tasks.get(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task not found: {}", task_id)))
    }
    
    /// タスクリストを取得
    pub fn get_tasks_by_status(&self, status: Option<TaskStatus>) -> Result<Vec<&Task>, Error> {
        // タスク管理が有効かチェック
        if !self.config.enable_task_management {
            return Err(Error::InvalidState("Task management is not enabled".to_string()));
        }
        
        // タスクリストを取得
        let tasks: Vec<&Task> = self.tasks.values()
            .filter(|t| status.as_ref().map_or(true, |s| t.status == *s))
            .collect();
        
        Ok(tasks)
    }
    
    /// カレンダーイベントを取得
    pub fn get_calendar_event(&self, event_id: &str) -> Result<&CalendarEvent, Error> {
        // カレンダーが有効かチェック
        if !self.config.enable_calendar {
            return Err(Error::InvalidState("Calendar is not enabled".to_string()));
        }
        
        // イベントを取得
        self.calendar_events.get(event_id)
            .ok_or_else(|| Error::NotFound(format!("Event not found: {}", event_id)))
    }
    
    /// 期間のカレンダーイベントリストを取得
    pub fn get_calendar_events_by_period(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<&CalendarEvent>, Error> {
        // カレンダーが有効かチェック
        if !self.config.enable_calendar {
            return Err(Error::InvalidState("Calendar is not enabled".to_string()));
        }
        
        // 期間のイベントリストを取得
        let events: Vec<&CalendarEvent> = self.calendar_events.values()
            .filter(|e| e.start_time < end_time && e.end_time > start_time)
            .collect();
        
        Ok(events)
    }
    
    /// 受信者の通知リストを取得
    pub fn get_notifications_by_recipient(&self, recipient: &str) -> Result<Vec<&Notification>, Error> {
        // 通知が有効かチェック
        if !self.config.enable_notifications {
            return Err(Error::InvalidState("Notifications are not enabled".to_string()));
        }
        
        // 受信者の通知リストを取得
        let notifications = self.notifications.get(recipient)
            .map(|notifications| notifications.iter().collect::<Vec<&Notification>>())
            .unwrap_or_default();
        
        Ok(notifications)
    }
    
    /// 未読通知数を取得
    pub fn get_unread_notification_count(&self, recipient: &str) -> Result<usize, Error> {
        // 通知が有効かチェック
        if !self.config.enable_notifications {
            return Err(Error::InvalidState("Notifications are not enabled".to_string()));
        }
        
        // 未読通知数を取得
        let count = self.notifications.get(recipient)
            .map(|notifications| notifications.iter().filter(|n| !n.is_read).count())
            .unwrap_or(0);
        
        Ok(count)
    }
    
    /// 分析データを取得
    pub fn get_analytics_data(&self, data_id: &str) -> Result<&AnalyticsData, Error> {
        // 分析が有効かチェック
        if !self.config.enable_analytics {
            return Err(Error::InvalidState("Analytics are not enabled".to_string()));
        }
        
        // 分析データを取得
        self.analytics_data.get(data_id)
            .ok_or_else(|| Error::NotFound(format!("Analytics data not found: {}", data_id)))
    }
    
    /// タイプと期間の分析データリストを取得
    pub fn get_analytics_data_by_type_and_period(
        &self,
        data_type: AnalyticsDataType,
        period: &str,
    ) -> Result<Vec<&AnalyticsData>, Error> {
        // 分析が有効かチェック
        if !self.config.enable_analytics {
            return Err(Error::InvalidState("Analytics are not enabled".to_string()));
        }
        
        // タイプと期間の分析データリストを取得
        let data: Vec<&AnalyticsData> = self.analytics_data.values()
            .filter(|d| d.data_type == data_type && d.period == period)
            .collect();
        
        Ok(data)
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &CommunityToolsConfig {
        &self.config
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: CommunityToolsConfig) {
        self.config = config;
    }
}

impl Default for CommunityToolsConfig {
    fn default() -> Self {
        Self {
            enable_forum: true,
            enable_voting_dashboard: true,
            enable_treasury_dashboard: true,
            enable_member_directory: true,
            enable_task_management: true,
            enable_calendar: true,
            enable_notifications: true,
            enable_analytics: true,
            enable_multilingual: true,
            supported_languages: vec![
                "en".to_string(),
                "ja".to_string(),
                "zh".to_string(),
                "ko".to_string(),
                "es".to_string(),
                "fr".to_string(),
                "de".to_string(),
                "ru".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_community_tools_config() {
        let config = CommunityToolsConfig::default();
        
        assert!(config.enable_forum);
        assert!(config.enable_voting_dashboard);
        assert!(config.enable_treasury_dashboard);
        assert!(config.enable_member_directory);
        assert!(config.enable_task_management);
        assert!(config.enable_calendar);
        assert!(config.enable_notifications);
        assert!(config.enable_analytics);
        assert!(config.enable_multilingual);
        assert_eq!(config.supported_languages.len(), 8);
    }
    
    #[test]
    fn test_forum_functionality() {
        let config = CommunityToolsConfig::default();
        let mut manager = CommunityToolsManager::new(config);
        
        // カテゴリを作成
        let category_id = manager.create_forum_category(
            "General Discussion",
            "General discussion about the project",
            None,
        ).unwrap();
        
        // カテゴリを取得
        let category = manager.get_forum_category(&category_id).unwrap();
        assert_eq!(category.name, "General Discussion");
        
        // トピックを作成
        let topic_id = manager.create_forum_topic(
            "Welcome to the forum",
            "This is the first post in our forum",
            "admin",
            &category_id,
            vec!["welcome".to_string(), "announcement".to_string()],
        ).unwrap();
        
        // トピックを取得
        let topic = manager.get_forum_topic(&topic_id).unwrap();
        assert_eq!(topic.title, "Welcome to the forum");
        
        // 返信を作成
        let reply_id = manager.create_forum_reply(
            &topic_id,
            "Thanks for the welcome!",
            "user1",
            None,
        ).unwrap();
        
        // 返信を取得
        let reply = manager.get_forum_reply(&reply_id).unwrap();
        assert_eq!(reply.content, "Thanks for the welcome!");
        
        // トピックの返信リストを取得
        let replies = manager.get_forum_replies_by_topic(&topic_id).unwrap();
        assert_eq!(replies.len(), 1);
        
        // トピックの返信数を確認
        let topic = manager.get_forum_topic(&topic_id).unwrap();
        assert_eq!(topic.reply_count, 1);
    }
    
    #[test]
    fn test_task_management() {
        let config = CommunityToolsConfig::default();
        let mut manager = CommunityToolsManager::new(config);
        
        // タスクを作成
        let task_id = manager.create_task(
            "Implement feature X",
            "We need to implement feature X for the next release",
            "admin",
            Some("developer1"),
            TaskPriority::High,
            Some(Utc::now() + chrono::Duration::days(7)),
            vec!["feature".to_string(), "development".to_string()],
            None,
        ).unwrap();
        
        // タスクを取得
        let task = manager.get_task(&task_id).unwrap();
        assert_eq!(task.title, "Implement feature X");
        assert_eq!(task.status, TaskStatus::Todo);
        
        // タスクステータスを更新
        manager.update_task_status(&task_id, TaskStatus::InProgress, "developer1").unwrap();
        
        // 更新されたタスクを取得
        let task = manager.get_task(&task_id).unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
        
        // タスクリストを取得
        let tasks = manager.get_tasks_by_status(Some(TaskStatus::InProgress)).unwrap();
        assert_eq!(tasks.len(), 1);
    }
    
    #[test]
    fn test_notifications() {
        let config = CommunityToolsConfig::default();
        let mut manager = CommunityToolsManager::new(config);
        
        // 通知を作成
        let notification_id = manager.create_notification(
            "user1",
            "New proposal",
            "A new proposal has been created",
            NotificationType::Proposal,
            Some("proposal-123"),
        ).unwrap();
        
        // 通知リストを取得
        let notifications = manager.get_notifications_by_recipient("user1").unwrap();
        assert_eq!(notifications.len(), 1);
        
        // 未読通知数を取得
        let unread_count = manager.get_unread_notification_count("user1").unwrap();
        assert_eq!(unread_count, 1);
        
        // 通知を既読にする
        manager.mark_notification_as_read("user1", &notification_id).unwrap();
        
        // 未読通知数を再取得
        let unread_count = manager.get_unread_notification_count("user1").unwrap();
        assert_eq!(unread_count, 0);
    }
}