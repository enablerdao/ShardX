use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// コンプライアンスフレームワーク
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComplianceFramework {
    /// フレームワークID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// フレームワークタイプ
    pub framework_type: FrameworkType,
    /// バージョン
    pub version: String,
    /// 発行機関
    pub authority: String,
    /// 発効日
    pub effective_date: DateTime<Utc>,
    /// ステータス
    pub status: FrameworkStatus,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// フレームワークタイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameworkType {
    /// セキュリティ
    Security,
    /// プライバシー
    Privacy,
    /// リスク管理
    RiskManagement,
    /// 業界固有
    IndustrySpecific,
    /// 地域固有
    RegionalSpecific,
    /// その他
    Other,
}

/// フレームワークバージョン
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameworkVersion {
    /// バージョン番号
    pub version: String,
    /// リリース日
    pub release_date: DateTime<Utc>,
    /// 変更内容
    pub changes: String,
    /// 前バージョン
    pub previous_version: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// フレームワークステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameworkStatus {
    /// ドラフト
    Draft,
    /// アクティブ
    Active,
    /// 廃止
    Deprecated,
    /// 置換
    Superseded,
    /// 削除
    Retired,
}

impl ComplianceFramework {
    /// 新しいComplianceFrameworkを作成
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        framework_type: FrameworkType,
        version: &str,
        authority: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            framework_type,
            version: version.to_string(),
            authority: authority.to_string(),
            effective_date: Utc::now(),
            status: FrameworkStatus::Active,
            metadata: HashMap::new(),
        }
    }
    
    /// メタデータを追加
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// メタデータを取得
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// ステータスを更新
    pub fn update_status(&mut self, status: FrameworkStatus) {
        self.status = status;
    }
    
    /// アクティブかどうかを確認
    pub fn is_active(&self) -> bool {
        self.status == FrameworkStatus::Active
    }
    
    /// 説明を更新
    pub fn update_description(&mut self, description: &str) {
        self.description = description.to_string();
    }
    
    /// バージョンを更新
    pub fn update_version(&mut self, version: &str) {
        self.version = version.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_framework_creation() {
        let framework = ComplianceFramework::new(
            "FW-GDPR",
            "General Data Protection Regulation",
            "EU regulation on data protection and privacy",
            FrameworkType::Privacy,
            "2016/679",
            "EU",
        );
        
        assert_eq!(framework.id, "FW-GDPR");
        assert_eq!(framework.name, "General Data Protection Regulation");
        assert_eq!(framework.framework_type, FrameworkType::Privacy);
        assert_eq!(framework.version, "2016/679");
        assert_eq!(framework.authority, "EU");
        assert_eq!(framework.status, FrameworkStatus::Active);
        assert!(framework.metadata.is_empty());
    }
    
    #[test]
    fn test_framework_metadata() {
        let mut framework = ComplianceFramework::new(
            "FW-GDPR",
            "General Data Protection Regulation",
            "EU regulation on data protection and privacy",
            FrameworkType::Privacy,
            "2016/679",
            "EU",
        );
        
        // メタデータを追加
        framework.add_metadata("url", "https://gdpr.eu/");
        framework.add_metadata("effective_date", "2018-05-25");
        
        // メタデータを取得
        assert_eq!(framework.get_metadata("url"), Some(&"https://gdpr.eu/".to_string()));
        assert_eq!(framework.get_metadata("effective_date"), Some(&"2018-05-25".to_string()));
        assert_eq!(framework.get_metadata("non_existent"), None);
    }
    
    #[test]
    fn test_framework_status_update() {
        let mut framework = ComplianceFramework::new(
            "FW-GDPR",
            "General Data Protection Regulation",
            "EU regulation on data protection and privacy",
            FrameworkType::Privacy,
            "2016/679",
            "EU",
        );
        
        // 初期ステータスをチェック
        assert_eq!(framework.status, FrameworkStatus::Active);
        assert!(framework.is_active());
        
        // ステータスを更新
        framework.update_status(FrameworkStatus::Deprecated);
        
        // 更新後のステータスをチェック
        assert_eq!(framework.status, FrameworkStatus::Deprecated);
        assert!(!framework.is_active());
    }
    
    #[test]
    fn test_framework_update() {
        let mut framework = ComplianceFramework::new(
            "FW-GDPR",
            "General Data Protection Regulation",
            "EU regulation on data protection and privacy",
            FrameworkType::Privacy,
            "2016/679",
            "EU",
        );
        
        // 説明を更新
        framework.update_description("Updated description for GDPR");
        assert_eq!(framework.description, "Updated description for GDPR");
        
        // バージョンを更新
        framework.update_version("2016/679-rev1");
        assert_eq!(framework.version, "2016/679-rev1");
    }
}