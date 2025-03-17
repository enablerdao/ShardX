use crate::error::Error;
use crate::smart_contract::ContractABI;
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
    /// バリデータ名
    name: String,
    /// バリデータバージョン
    version: String,
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
    /// ABIエラー
    #[error("ABI error: {0}")]
    ABIError(String),
    /// コード解析エラー
    #[error("Code analysis error: {0}")]
    CodeAnalysisError(String),
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
    /// ABIを使用したバリデーションを実行
    fn validate_with_abi(&self, contract_code: &[u8], abi: &ContractABI) -> Result<ValidationResult, ValidationError> {
        // デフォルト実装はABIを無視して通常のバリデーションを実行
        self.validate(contract_code)
    }
    /// 重要度レベル（0-100）
    fn severity(&self) -> u8 {
        50 // デフォルトは中程度の重要度
    }
    /// カテゴリ
    fn category(&self) -> &str {
        "general" // デフォルトは一般カテゴリ
    }
}

impl ContractValidator {
    /// 新しいContractValidatorを作成
    pub fn new() -> Self {
        Self { 
            rules: Vec::new(),
            name: "Default Validator".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    /// 名前とバージョンを指定して新しいContractValidatorを作成
    pub fn with_name_version(name: &str, version: &str) -> Self {
        Self {
            rules: Vec::new(),
            name: name.to_string(),
            version: version.to_string(),
        }
    }

    /// バリデータ名を取得
    pub fn name(&self) -> &str {
        &self.name
    }

    /// バリデータバージョンを取得
    pub fn version(&self) -> &str {
        &self.version
    }

    /// バリデーションルールを追加
    pub fn add_rule<R: ValidationRule + 'static>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// バリデーションルールを取得
    pub fn rules(&self) -> &[Box<dyn ValidationRule>] {
        &self.rules
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

    /// ABIを使用したバリデーションを実行
    pub fn validate_with_abi(&self, contract_code: &[u8], abi: &ContractABI) -> Result<ValidationResult, Error> {
        let mut success = true;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut metrics = HashMap::new();

        for rule in &self.rules {
            match rule.validate_with_abi(contract_code, abi) {
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

    /// 特定のカテゴリのルールのみでバリデーションを実行
    pub fn validate_by_category(&self, contract_code: &[u8], category: &str) -> Result<ValidationResult, Error> {
        let mut success = true;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut metrics = HashMap::new();

        for rule in &self.rules {
            if rule.category() == category {
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
        }

        Ok(ValidationResult {
            success,
            warnings,
            errors,
            metrics,
        })
    }

    /// 特定の重要度以上のルールのみでバリデーションを実行
    pub fn validate_by_severity(&self, contract_code: &[u8], min_severity: u8) -> Result<ValidationResult, Error> {
        let mut success = true;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut metrics = HashMap::new();

        for rule in &self.rules {
            if rule.severity() >= min_severity {
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

/// セキュリティルール
pub struct SecurityRule {
    name: String,
    description: String,
    severity: u8,
    patterns: Vec<Vec<u8>>,
}

impl SecurityRule {
    /// 新しいセキュリティルールを作成
    pub fn new(name: &str, description: &str, severity: u8) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            severity,
            patterns: Vec::new(),
        }
    }

    /// パターンを追加
    pub fn add_pattern(&mut self, pattern: Vec<u8>) {
        self.patterns.push(pattern);
    }
}

impl ValidationRule for SecurityRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn severity(&self) -> u8 {
        self.severity
    }

    fn category(&self) -> &str {
        "security"
    }

    fn validate(&self, contract_code: &[u8]) -> Result<ValidationResult, ValidationError> {
        let mut success = true;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut metrics = HashMap::new();

        for pattern in &self.patterns {
            if contract_code.windows(pattern.len()).any(|window| window == pattern.as_slice()) {
                success = false;
                errors.push(format!(
                    "Found forbidden pattern: {:?}",
                    pattern
                ));
                metrics.insert("pattern_matches".to_string(), metrics.get("pattern_matches").unwrap_or(&0.0) + 1.0);
            }
        }

        metrics.insert("code_size".to_string(), contract_code.len() as f64);

        Ok(ValidationResult {
            success,
            warnings,
            errors,
            metrics,
        })
    }
}

/// ガス使用量ルール
pub struct GasUsageRule {
    name: String,
    description: String,
    severity: u8,
    max_gas: u64,
    gas_per_byte: f64,
}

impl GasUsageRule {
    /// 新しいガス使用量ルールを作成
    pub fn new(name: &str, description: &str, severity: u8, max_gas: u64, gas_per_byte: f64) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            severity,
            max_gas,
            gas_per_byte,
        }
    }
}

impl ValidationRule for GasUsageRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn severity(&self) -> u8 {
        self.severity
    }

    fn category(&self) -> &str {
        "gas"
    }

    fn validate(&self, contract_code: &[u8]) -> Result<ValidationResult, ValidationError> {
        let mut success = true;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut metrics = HashMap::new();

        let estimated_gas = (contract_code.len() as f64 * self.gas_per_byte) as u64;
        metrics.insert("estimated_gas".to_string(), estimated_gas as f64);

        if estimated_gas > self.max_gas {
            success = false;
            errors.push(format!(
                "Estimated gas usage exceeds maximum: {} > {}",
                estimated_gas, self.max_gas
            ));
        } else if estimated_gas > self.max_gas * 8 / 10 {
            warnings.push(format!(
                "Estimated gas usage is approaching maximum: {} > {}",
                estimated_gas, self.max_gas * 8 / 10
            ));
        }

        Ok(ValidationResult {
            success,
            warnings,
            errors,
            metrics,
        })
    }
}

/// コードサイズルール
pub struct CodeSizeRule {
    name: String,
    description: String,
    severity: u8,
    max_size: usize,
}

impl CodeSizeRule {
    /// 新しいコードサイズルールを作成
    pub fn new(name: &str, description: &str, severity: u8, max_size: usize) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            severity,
            max_size,
        }
    }
}

impl ValidationRule for CodeSizeRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn severity(&self) -> u8 {
        self.severity
    }

    fn category(&self) -> &str {
        "size"
    }

    fn validate(&self, contract_code: &[u8]) -> Result<ValidationResult, ValidationError> {
        let mut success = true;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut metrics = HashMap::new();

        let code_size = contract_code.len();
        metrics.insert("code_size".to_string(), code_size as f64);

        if code_size > self.max_size {
            success = false;
            errors.push(format!(
                "Code size exceeds maximum: {} > {}",
                code_size, self.max_size
            ));
        } else if code_size > self.max_size * 8 / 10 {
            warnings.push(format!(
                "Code size is approaching maximum: {} > {}",
                code_size, self.max_size * 8 / 10
            ));
        }

        Ok(ValidationResult {
            success,
            warnings,
            errors,
            metrics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRule {
        name: String,
        description: String,
        should_succeed: bool,
        severity: u8,
        category: String,
    }

    impl ValidationRule for TestRule {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn severity(&self) -> u8 {
            self.severity
        }

        fn category(&self) -> &str {
            &self.category
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

        fn validate_with_abi(&self, _contract_code: &[u8], _abi: &ContractABI) -> Result<ValidationResult, ValidationError> {
            if self.should_succeed {
                Ok(ValidationResult {
                    success: true,
                    warnings: Vec::new(),
                    errors: Vec::new(),
                    metrics: {
                        let mut m = HashMap::new();
                        m.insert("abi_validated".to_string(), 1.0);
                        m
                    },
                })
            } else {
                Ok(ValidationResult {
                    success: false,
                    warnings: vec!["Test ABI warning".to_string()],
                    errors: vec!["Test ABI error".to_string()],
                    metrics: {
                        let mut m = HashMap::new();
                        m.insert("abi_validated".to_string(), 0.0);
                        m
                    },
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
            severity: 50,
            category: "test".to_string(),
        });
        validator.add_rule(TestRule {
            name: "Test Rule 2".to_string(),
            description: "Test rule that always succeeds".to_string(),
            should_succeed: true,
            severity: 50,
            category: "test".to_string(),
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
            severity: 50,
            category: "test".to_string(),
        });
        validator.add_rule(TestRule {
            name: "Test Rule 2".to_string(),
            description: "Test rule that always fails".to_string(),
            should_succeed: false,
            severity: 50,
            category: "test".to_string(),
        });

        let result = validator.validate(&[]).unwrap();
        assert!(!result.success);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validator_with_name_version() {
        let validator = ContractValidator::with_name_version("Custom Validator", "2.0.0");
        assert_eq!(validator.name(), "Custom Validator");
        assert_eq!(validator.version(), "2.0.0");
    }

    #[test]
    fn test_validate_with_abi() {
        let mut validator = ContractValidator::new();
        validator.add_rule(TestRule {
            name: "Test Rule 1".to_string(),
            description: "Test rule that always succeeds".to_string(),
            should_succeed: true,
            severity: 50,
            category: "test".to_string(),
        });

        let abi = ContractABI::new("TestContract", "1.0.0");
        let result = validator.validate_with_abi(&[], &abi).unwrap();
        assert!(result.success);
        assert!(result.warnings.is_empty());
        assert!(result.errors.is_empty());
        assert_eq!(*result.metrics.get("abi_validated").unwrap(), 1.0);
    }

    #[test]
    fn test_validate_by_category() {
        let mut validator = ContractValidator::new();
        validator.add_rule(TestRule {
            name: "Test Rule 1".to_string(),
            description: "Test rule in category A".to_string(),
            should_succeed: true,
            severity: 50,
            category: "A".to_string(),
        });
        validator.add_rule(TestRule {
            name: "Test Rule 2".to_string(),
            description: "Test rule in category B that fails".to_string(),
            should_succeed: false,
            severity: 50,
            category: "B".to_string(),
        });

        // カテゴリAのみのバリデーション
        let result = validator.validate_by_category(&[], "A").unwrap();
        assert!(result.success);
        assert!(result.warnings.is_empty());
        assert!(result.errors.is_empty());

        // カテゴリBのみのバリデーション
        let result = validator.validate_by_category(&[], "B").unwrap();
        assert!(!result.success);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validate_by_severity() {
        let mut validator = ContractValidator::new();
        validator.add_rule(TestRule {
            name: "Low Severity Rule".to_string(),
            description: "Test rule with low severity that fails".to_string(),
            should_succeed: false,
            severity: 30,
            category: "test".to_string(),
        });
        validator.add_rule(TestRule {
            name: "High Severity Rule".to_string(),
            description: "Test rule with high severity that succeeds".to_string(),
            should_succeed: true,
            severity: 80,
            category: "test".to_string(),
        });

        // 高重要度のみのバリデーション
        let result = validator.validate_by_severity(&[], 70).unwrap();
        assert!(result.success);
        assert!(result.warnings.is_empty());
        assert!(result.errors.is_empty());

        // 低重要度を含むバリデーション
        let result = validator.validate_by_severity(&[], 20).unwrap();
        assert!(!result.success);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_security_rule() {
        let mut rule = SecurityRule::new(
            "Forbidden Pattern Rule",
            "Detects forbidden bytecode patterns",
            90,
        );
        rule.add_pattern(vec![0xFE, 0xFF]); // 禁止パターン

        // 禁止パターンを含まないコード
        let safe_code = vec![0x01, 0x02, 0x03];
        let result = rule.validate(&safe_code).unwrap();
        assert!(result.success);
        assert!(result.errors.is_empty());

        // 禁止パターンを含むコード
        let unsafe_code = vec![0x01, 0xFE, 0xFF, 0x03];
        let result = rule.validate(&unsafe_code).unwrap();
        assert!(!result.success);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("forbidden pattern"));
    }

    #[test]
    fn test_gas_usage_rule() {
        let rule = GasUsageRule::new(
            "Gas Usage Rule",
            "Checks if contract uses too much gas",
            70,
            1000, // 最大ガス
            10.0, // 1バイトあたり10ガス
        );

        // ガス使用量が少ないコード
        let small_code = vec![0; 50]; // 50 * 10 = 500ガス
        let result = rule.validate(&small_code).unwrap();
        assert!(result.success);
        assert!(result.errors.is_empty());
        assert_eq!(*result.metrics.get("estimated_gas").unwrap(), 500.0);

        // ガス使用量が多いコード
        let large_code = vec![0; 150]; // 150 * 10 = 1500ガス
        let result = rule.validate(&large_code).unwrap();
        assert!(!result.success);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("exceeds maximum"));
        assert_eq!(*result.metrics.get("estimated_gas").unwrap(), 1500.0);
    }

    #[test]
    fn test_code_size_rule() {
        let rule = CodeSizeRule::new(
            "Code Size Rule",
            "Checks if contract code is too large",
            60,
            100, // 最大サイズ
        );

        // サイズが小さいコード
        let small_code = vec![0; 50];
        let result = rule.validate(&small_code).unwrap();
        assert!(result.success);
        assert!(result.errors.is_empty());
        assert_eq!(*result.metrics.get("code_size").unwrap(), 50.0);

        // サイズが大きいコード
        let large_code = vec![0; 150];
        let result = rule.validate(&large_code).unwrap();
        assert!(!result.success);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("exceeds maximum"));
        assert_eq!(*result.metrics.get("code_size").unwrap(), 150.0);
    }
}