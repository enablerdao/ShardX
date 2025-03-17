use crate::error::Error;
use std::collections::HashMap;

/// コントラクトバリデーター
///
/// スマートコントラクトのバリデーションを行う。
/// - セキュリティチェック
/// - ガス使用量の見積もり
/// - 依存関係の解析
pub struct ContractValidator {
    /// バリデーションルール
    rules: Vec<Box<dyn ValidationRule>>,
}

/// バリデーション結果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// バリデーション成功フラグ
    pub success: bool,
    /// 警告メッセージ
    pub warnings: Vec<String>,
    /// エラーメッセージ
    pub errors: Vec<String>,
    /// メトリクス
    pub metrics: HashMap<String, f64>,
}

/// バリデーションエラー
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// セキュリティエラー
    #[error("Security error: {0}")]
    SecurityError(String),
    /// ガス使用量エラー
    #[error("Gas usage error: {0}")]
    GasUsageError(String),
    /// 依存関係エラー
    #[error("Dependency error: {0}")]
    DependencyError(String),
    /// その他のエラー
    #[error("Validation error: {0}")]
    Other(String),
}

/// バリデーションルール
pub trait ValidationRule: Send + Sync {
    /// ルール名
    fn name(&self) -> &str;
    /// ルールの説明
    fn description(&self) -> &str;
    /// バリデーションを実行
    fn validate(&self, contract_code: &[u8]) -> Result<ValidationResult, ValidationError>;
}

impl ContractValidator {
    /// 新しいContractValidatorを作成
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// バリデーションルールを追加
    pub fn add_rule<R: ValidationRule + 'static>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// バリデーションを実行
    pub fn validate(&self, contract_code: &[u8]) -> Result<ValidationResult, Error> {
        let mut success = true;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut metrics = HashMap::new();

        for rule in &self.rules {
            match rule.validate(contract_code) {
                Ok(result) => {
                    if !result.success {
                        success = false;
                    }
                    warnings.extend(result.warnings);
                    errors.extend(result.errors);
                    for (key, value) in result.metrics {
                        metrics.insert(key, value);
                    }
                }
                Err(e) => {
                    success = false;
                    errors.push(format!("{}: {}", rule.name(), e));
                }
            }
        }

        Ok(ValidationResult {
            success,
            warnings,
            errors,
            metrics,
        })
    }
}

impl Default for ContractValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRule {
        name: String,
        description: String,
        should_succeed: bool,
    }

    impl ValidationRule for TestRule {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn validate(&self, _contract_code: &[u8]) -> Result<ValidationResult, ValidationError> {
            if self.should_succeed {
                Ok(ValidationResult {
                    success: true,
                    warnings: Vec::new(),
                    errors: Vec::new(),
                    metrics: HashMap::new(),
                })
            } else {
                Ok(ValidationResult {
                    success: false,
                    warnings: vec!["Test warning".to_string()],
                    errors: vec!["Test error".to_string()],
                    metrics: HashMap::new(),
                })
            }
        }
    }

    #[test]
    fn test_validator_success() {
        let mut validator = ContractValidator::new();
        validator.add_rule(TestRule {
            name: "Test Rule 1".to_string(),
            description: "Test rule that always succeeds".to_string(),
            should_succeed: true,
        });
        validator.add_rule(TestRule {
            name: "Test Rule 2".to_string(),
            description: "Test rule that always succeeds".to_string(),
            should_succeed: true,
        });

        let result = validator.validate(&[]).unwrap();
        assert!(result.success);
        assert!(result.warnings.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validator_failure() {
        let mut validator = ContractValidator::new();
        validator.add_rule(TestRule {
            name: "Test Rule 1".to_string(),
            description: "Test rule that always succeeds".to_string(),
            should_succeed: true,
        });
        validator.add_rule(TestRule {
            name: "Test Rule 2".to_string(),
            description: "Test rule that always fails".to_string(),
            should_succeed: false,
        });

        let result = validator.validate(&[]).unwrap();
        assert!(!result.success);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.errors.len(), 1);
    }
}