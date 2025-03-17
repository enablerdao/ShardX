use crate::error::Error;
use std::collections::HashMap;
use std::path::Path;

/// コンパイラ設定
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// 最適化レベル
    pub optimization_level: u8,
    /// デバッグ情報を含めるフラグ
    pub include_debug_info: bool,
    /// ターゲットプラットフォーム
    pub target_platform: String,
    /// 追加のコンパイラフラグ
    pub compiler_flags: Vec<String>,
    /// 追加のインクルードパス
    pub include_paths: Vec<String>,
    /// 定義マクロ
    pub defines: HashMap<String, String>,
}

/// コンパイル結果
#[derive(Debug, Clone)]
pub struct CompilationResult {
    /// コンパイル成功フラグ
    pub success: bool,
    /// コンパイル済みコード
    pub compiled_code: Vec<u8>,
    /// デバッグ情報
    pub debug_info: Option<Vec<u8>>,
    /// 警告メッセージ
    pub warnings: Vec<String>,
    /// エラーメッセージ
    pub errors: Vec<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// コンパイルエラー
#[derive(Debug, thiserror::Error)]
pub enum CompilationError {
    /// 構文エラー
    #[error("Syntax error: {0}")]
    SyntaxError(String),
    /// 型エラー
    #[error("Type error: {0}")]
    TypeError(String),
    /// リンクエラー
    #[error("Linker error: {0}")]
    LinkerError(String),
    /// 最適化エラー
    #[error("Optimization error: {0}")]
    OptimizationError(String),
    /// その他のエラー
    #[error("Compilation error: {0}")]
    Other(String),
}

/// コンパイラトレイト
pub trait Compiler: Send + Sync {
    /// コンパイラ名
    fn name(&self) -> &str;
    /// コンパイラバージョン
    fn version(&self) -> &str;
    /// サポートする言語
    fn supported_languages(&self) -> Vec<String>;
    /// サポートするターゲットプラットフォーム
    fn supported_targets(&self) -> Vec<String>;
    /// コードをコンパイル
    fn compile(
        &self,
        source_code: &str,
        config: &CompilerConfig,
    ) -> Result<CompilationResult, CompilationError>;
    /// ファイルをコンパイル
    fn compile_file(
        &self,
        file_path: &Path,
        config: &CompilerConfig,
    ) -> Result<CompilationResult, CompilationError>;
    /// コンパイル済みコードを検証
    fn validate_compiled_code(&self, compiled_code: &[u8]) -> Result<bool, CompilationError>;
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            optimization_level: 1,
            include_debug_info: false,
            target_platform: "wasm32-unknown-unknown".to_string(),
            compiler_flags: Vec::new(),
            include_paths: Vec::new(),
            defines: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TestCompiler;

    impl Compiler for TestCompiler {
        fn name(&self) -> &str {
            "TestCompiler"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn supported_languages(&self) -> Vec<String> {
            vec!["test".to_string()]
        }

        fn supported_targets(&self) -> Vec<String> {
            vec!["wasm32-unknown-unknown".to_string()]
        }

        fn compile(
            &self,
            source_code: &str,
            config: &CompilerConfig,
        ) -> Result<CompilationResult, CompilationError> {
            if source_code.contains("error") {
                return Err(CompilationError::SyntaxError(
                    "Test syntax error".to_string(),
                ));
            }

            let mut compiled_code = Vec::new();
            compiled_code.extend_from_slice(b"COMPILED:");
            compiled_code.extend_from_slice(source_code.as_bytes());

            let mut metadata = HashMap::new();
            metadata.insert("compiler".to_string(), self.name().to_string());
            metadata.insert("version".to_string(), self.version().to_string());
            metadata.insert(
                "optimization_level".to_string(),
                config.optimization_level.to_string(),
            );

            Ok(CompilationResult {
                success: true,
                compiled_code,
                debug_info: if config.include_debug_info {
                    Some(b"DEBUG_INFO".to_vec())
                } else {
                    None
                },
                warnings: if source_code.contains("warning") {
                    vec!["Test warning".to_string()]
                } else {
                    Vec::new()
                },
                errors: Vec::new(),
                metadata,
            })
        }

        fn compile_file(
            &self,
            file_path: &Path,
            config: &CompilerConfig,
        ) -> Result<CompilationResult, CompilationError> {
            if !file_path.exists() {
                return Err(CompilationError::Other(format!(
                    "File not found: {}",
                    file_path.display()
                )));
            }

            // 実際のファイル読み込みは省略
            let source_code = "test source code";
            self.compile(source_code, config)
        }

        fn validate_compiled_code(&self, compiled_code: &[u8]) -> Result<bool, CompilationError> {
            let prefix = b"COMPILED:";
            Ok(compiled_code.starts_with(prefix))
        }
    }

    #[test]
    fn test_compiler_config_default() {
        let config = CompilerConfig::default();
        assert_eq!(config.optimization_level, 1);
        assert_eq!(config.include_debug_info, false);
        assert_eq!(config.target_platform, "wasm32-unknown-unknown");
        assert!(config.compiler_flags.is_empty());
        assert!(config.include_paths.is_empty());
        assert!(config.defines.is_empty());
    }

    #[test]
    fn test_compiler_compile_success() {
        let compiler = TestCompiler;
        let config = CompilerConfig::default();
        let result = compiler.compile("test code", &config).unwrap();
        assert!(result.success);
        assert_eq!(result.compiled_code, b"COMPILED:test code");
        assert!(result.debug_info.is_none());
        assert!(result.warnings.is_empty());
        assert!(result.errors.is_empty());
        assert_eq!(result.metadata.get("compiler").unwrap(), "TestCompiler");
    }

    #[test]
    fn test_compiler_compile_with_warning() {
        let compiler = TestCompiler;
        let config = CompilerConfig::default();
        let result = compiler.compile("test code with warning", &config).unwrap();
        assert!(result.success);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0], "Test warning");
    }

    #[test]
    fn test_compiler_compile_with_error() {
        let compiler = TestCompiler;
        let config = CompilerConfig::default();
        let result = compiler.compile("test code with error", &config);
        assert!(result.is_err());
        match result {
            Err(CompilationError::SyntaxError(msg)) => {
                assert_eq!(msg, "Test syntax error");
            }
            _ => panic!("Expected SyntaxError"),
        }
    }

    #[test]
    fn test_compiler_compile_with_debug_info() {
        let compiler = TestCompiler;
        let mut config = CompilerConfig::default();
        config.include_debug_info = true;
        let result = compiler.compile("test code", &config).unwrap();
        assert!(result.debug_info.is_some());
        assert_eq!(result.debug_info.unwrap(), b"DEBUG_INFO");
    }

    #[test]
    fn test_compiler_validate_compiled_code() {
        let compiler = TestCompiler;
        let valid_code = b"COMPILED:test code".to_vec();
        let invalid_code = b"INVALID:test code".to_vec();
        assert!(compiler.validate_compiled_code(&valid_code).unwrap());
        assert!(!compiler.validate_compiled_code(&invalid_code).unwrap());
    }
}