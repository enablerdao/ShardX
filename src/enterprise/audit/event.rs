use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// 監査イベント
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    /// イベントID
    pub id: String,
    /// イベントタイプ
    pub event_type: AuditEventType,
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// イベントソース
    pub source: EventSource,
    /// イベントターゲット
    pub target: EventTarget,
    /// アクション
    pub action: String,
    /// 結果
    pub result: String,
    /// メタデータ
    pub metadata: Value,
}

/// 監査イベントタイプ
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditEventType {
    /// 認証
    Authentication,
    /// 認可
    Authorization,
    /// データアクセス
    DataAccess,
    /// システム変更
    SystemChange,
    /// ユーザーアクティビティ
    UserActivity,
    /// セキュリティイベント
    SecurityEvent,
    /// コンプライアンスイベント
    ComplianceEvent,
    /// リソースイベント
    ResourceEvent,
    /// ネットワークイベント
    NetworkEvent,
    /// アプリケーションイベント
    ApplicationEvent,
    /// データベースイベント
    DatabaseEvent,
    /// APIイベント
    APIEvent,
    /// アクセス制御
    AccessControl,
    /// その他
    Other,
}

/// イベントソース
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSource {
    /// ユーザー
    User(String),
    /// システム
    System,
    /// サービス
    Service(String),
    /// アプリケーション
    Application(String),
    /// デバイス
    Device(String),
    /// 外部
    External(String),
}

/// イベントターゲット
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventTarget {
    /// リソース
    Resource(String),
    /// ユーザー
    User(String),
    /// システム
    System,
    /// サービス
    Service(String),
    /// アプリケーション
    Application(String),
    /// データ
    Data(String),
    /// ネットワーク
    Network(String),
}

impl AuditEvent {
    /// 新しいAuditEventを作成
    pub fn new(
        id: &str,
        event_type: AuditEventType,
        source: EventSource,
        target: EventTarget,
        action: &str,
        result: &str,
        metadata: Option<Value>,
    ) -> Self {
        Self {
            id: id.to_string(),
            event_type,
            timestamp: Utc::now(),
            source,
            target,
            action: action.to_string(),
            result: result.to_string(),
            metadata: metadata.unwrap_or_else(|| Value::Object(serde_json::Map::new())),
        }
    }

    /// メタデータを追加
    pub fn add_metadata(&mut self, key: &str, value: Value) {
        if let Value::Object(ref mut map) = self.metadata {
            map.insert(key.to_string(), value);
        }
    }

    /// メタデータを取得
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        if let Value::Object(ref map) = self.metadata {
            map.get(key)
        } else {
            None
        }
    }

    /// 成功したかどうかを確認
    pub fn is_success(&self) -> bool {
        self.result == "success" || self.result == "succeeded" || self.result == "allowed"
    }

    /// 失敗したかどうかを確認
    pub fn is_failure(&self) -> bool {
        self.result == "failure"
            || self.result == "failed"
            || self.result == "denied"
            || self.result == "error"
    }
}

impl fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditEventType::Authentication => write!(f, "Authentication"),
            AuditEventType::Authorization => write!(f, "Authorization"),
            AuditEventType::DataAccess => write!(f, "DataAccess"),
            AuditEventType::SystemChange => write!(f, "SystemChange"),
            AuditEventType::UserActivity => write!(f, "UserActivity"),
            AuditEventType::SecurityEvent => write!(f, "SecurityEvent"),
            AuditEventType::ComplianceEvent => write!(f, "ComplianceEvent"),
            AuditEventType::ResourceEvent => write!(f, "ResourceEvent"),
            AuditEventType::NetworkEvent => write!(f, "NetworkEvent"),
            AuditEventType::ApplicationEvent => write!(f, "ApplicationEvent"),
            AuditEventType::DatabaseEvent => write!(f, "DatabaseEvent"),
            AuditEventType::APIEvent => write!(f, "APIEvent"),
            AuditEventType::AccessControl => write!(f, "AccessControl"),
            AuditEventType::Other => write!(f, "Other"),
        }
    }
}

impl fmt::Display for EventSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventSource::User(user) => write!(f, "User({})", user),
            EventSource::System => write!(f, "System"),
            EventSource::Service(service) => write!(f, "Service({})", service),
            EventSource::Application(app) => write!(f, "Application({})", app),
            EventSource::Device(device) => write!(f, "Device({})", device),
            EventSource::External(source) => write!(f, "External({})", source),
        }
    }
}

impl fmt::Display for EventTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventTarget::Resource(resource) => write!(f, "Resource({})", resource),
            EventTarget::User(user) => write!(f, "User({})", user),
            EventTarget::System => write!(f, "System"),
            EventTarget::Service(service) => write!(f, "Service({})", service),
            EventTarget::Application(app) => write!(f, "Application({})", app),
            EventTarget::Data(data) => write!(f, "Data({})", data),
            EventTarget::Network(network) => write!(f, "Network({})", network),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            "EVENT-123",
            AuditEventType::Authentication,
            EventSource::User("user123".to_string()),
            EventTarget::Resource("login-service".to_string()),
            "login",
            "success",
            Some(serde_json::json!({
                "ip_address": "192.168.1.1",
                "user_agent": "Mozilla/5.0",
            })),
        );

        assert_eq!(event.id, "EVENT-123");
        assert_eq!(event.event_type, AuditEventType::Authentication);
        assert_eq!(event.action, "login");
        assert_eq!(event.result, "success");

        match event.source {
            EventSource::User(ref user) => assert_eq!(user, "user123"),
            _ => panic!("Expected User source"),
        }

        match event.target {
            EventTarget::Resource(ref resource) => assert_eq!(resource, "login-service"),
            _ => panic!("Expected Resource target"),
        }

        assert_eq!(event.metadata["ip_address"], "192.168.1.1");
        assert_eq!(event.metadata["user_agent"], "Mozilla/5.0");
    }

    #[test]
    fn test_audit_event_metadata() {
        let mut event = AuditEvent::new(
            "EVENT-123",
            AuditEventType::Authentication,
            EventSource::User("user123".to_string()),
            EventTarget::Resource("login-service".to_string()),
            "login",
            "success",
            None,
        );

        // メタデータを追加
        event.add_metadata("ip_address", serde_json::json!("192.168.1.1"));
        event.add_metadata("user_agent", serde_json::json!("Mozilla/5.0"));

        // メタデータを取得
        assert_eq!(
            event.get_metadata("ip_address").unwrap(),
            &serde_json::json!("192.168.1.1")
        );
        assert_eq!(
            event.get_metadata("user_agent").unwrap(),
            &serde_json::json!("Mozilla/5.0")
        );
        assert_eq!(event.get_metadata("non_existent"), None);
    }

    #[test]
    fn test_audit_event_status() {
        let success_event = AuditEvent::new(
            "EVENT-123",
            AuditEventType::Authentication,
            EventSource::User("user123".to_string()),
            EventTarget::Resource("login-service".to_string()),
            "login",
            "success",
            None,
        );

        let failure_event = AuditEvent::new(
            "EVENT-124",
            AuditEventType::Authentication,
            EventSource::User("user123".to_string()),
            EventTarget::Resource("login-service".to_string()),
            "login",
            "failed",
            None,
        );

        assert!(success_event.is_success());
        assert!(!success_event.is_failure());

        assert!(!failure_event.is_success());
        assert!(failure_event.is_failure());
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(AuditEventType::Authentication.to_string(), "Authentication");
        assert_eq!(AuditEventType::Authorization.to_string(), "Authorization");
        assert_eq!(AuditEventType::DataAccess.to_string(), "DataAccess");
        assert_eq!(AuditEventType::SystemChange.to_string(), "SystemChange");
        assert_eq!(AuditEventType::UserActivity.to_string(), "UserActivity");
        assert_eq!(AuditEventType::SecurityEvent.to_string(), "SecurityEvent");
        assert_eq!(
            AuditEventType::ComplianceEvent.to_string(),
            "ComplianceEvent"
        );
        assert_eq!(AuditEventType::ResourceEvent.to_string(), "ResourceEvent");
        assert_eq!(AuditEventType::NetworkEvent.to_string(), "NetworkEvent");
        assert_eq!(
            AuditEventType::ApplicationEvent.to_string(),
            "ApplicationEvent"
        );
        assert_eq!(AuditEventType::DatabaseEvent.to_string(), "DatabaseEvent");
        assert_eq!(AuditEventType::APIEvent.to_string(), "APIEvent");
        assert_eq!(AuditEventType::AccessControl.to_string(), "AccessControl");
        assert_eq!(AuditEventType::Other.to_string(), "Other");
    }

    #[test]
    fn test_event_source_display() {
        assert_eq!(
            EventSource::User("user123".to_string()).to_string(),
            "User(user123)"
        );
        assert_eq!(EventSource::System.to_string(), "System");
        assert_eq!(
            EventSource::Service("auth".to_string()).to_string(),
            "Service(auth)"
        );
        assert_eq!(
            EventSource::Application("web".to_string()).to_string(),
            "Application(web)"
        );
        assert_eq!(
            EventSource::Device("mobile".to_string()).to_string(),
            "Device(mobile)"
        );
        assert_eq!(
            EventSource::External("api".to_string()).to_string(),
            "External(api)"
        );
    }

    #[test]
    fn test_event_target_display() {
        assert_eq!(
            EventTarget::Resource("login".to_string()).to_string(),
            "Resource(login)"
        );
        assert_eq!(
            EventTarget::User("user123".to_string()).to_string(),
            "User(user123)"
        );
        assert_eq!(EventTarget::System.to_string(), "System");
        assert_eq!(
            EventTarget::Service("auth".to_string()).to_string(),
            "Service(auth)"
        );
        assert_eq!(
            EventTarget::Application("web".to_string()).to_string(),
            "Application(web)"
        );
        assert_eq!(
            EventTarget::Data("user_data".to_string()).to_string(),
            "Data(user_data)"
        );
        assert_eq!(
            EventTarget::Network("internal".to_string()).to_string(),
            "Network(internal)"
        );
    }
}
