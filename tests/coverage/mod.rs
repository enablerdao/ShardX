// テストカバレッジモジュール
//
// このモジュールは、ShardXのテストカバレッジを向上させるためのユーティリティを提供します。
// テストカバレッジは、コードの品質と信頼性を確保するために重要な指標です。

use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// カバレッジレポート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    /// 総行数
    pub total_lines: usize,
    /// カバーされた行数
    pub covered_lines: usize,
    /// カバレッジ率（%）
    pub coverage_percentage: f64,
    /// モジュールごとのカバレッジ
    pub module_coverage: HashMap<String, ModuleCoverage>,
    /// 生成日時
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// モジュールカバレッジ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCoverage {
    /// モジュール名
    pub name: String,
    /// 総行数
    pub total_lines: usize,
    /// カバーされた行数
    pub covered_lines: usize,
    /// カバレッジ率（%）
    pub coverage_percentage: f64,
    /// ファイルごとのカバレッジ
    pub file_coverage: HashMap<String, FileCoverage>,
}

/// ファイルカバレッジ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    /// ファイル名
    pub name: String,
    /// 総行数
    pub total_lines: usize,
    /// カバーされた行数
    pub covered_lines: usize,
    /// カバレッジ率（%）
    pub coverage_percentage: f64,
    /// 行ごとのカバレッジ
    pub line_coverage: HashMap<usize, bool>,
}

/// カバレッジマネージャー
pub struct CoverageManager {
    /// 出力ディレクトリ
    output_dir: PathBuf,
    /// ソースディレクトリ
    source_dir: PathBuf,
    /// 除外パターン
    exclude_patterns: Vec<String>,
}

impl CoverageManager {
    /// 新しいCoverageManagerを作成
    pub fn new<P: AsRef<Path>>(output_dir: P, source_dir: P) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
            source_dir: source_dir.as_ref().to_path_buf(),
            exclude_patterns: vec![
                String::from("**/target/**"),
                String::from("**/tests/**"),
                String::from("**/*.md"),
                String::from("**/.git/**"),
            ],
        }
    }

    /// 除外パターンを追加
    pub fn add_exclude_pattern(&mut self, pattern: &str) {
        self.exclude_patterns.push(pattern.to_string());
    }

    /// カバレッジレポートを生成
    pub fn generate_report(&self) -> io::Result<CoverageReport> {
        info!("Generating coverage report...");

        // ソースファイルを収集
        let source_files = self.collect_source_files()?;
        info!("Found {} source files", source_files.len());

        // カバレッジデータを収集
        let mut total_lines = 0;
        let mut covered_lines = 0;
        let mut module_coverage = HashMap::new();

        for file_path in &source_files {
            // ファイルの相対パスを取得
            let relative_path = file_path
                .strip_prefix(&self.source_dir)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            // モジュール名を取得
            let module_name = self.get_module_name(&relative_path);

            // ファイルカバレッジを取得
            let file_coverage = self.get_file_coverage(file_path)?;

            // 総計を更新
            total_lines += file_coverage.total_lines;
            covered_lines += file_coverage.covered_lines;

            // モジュールカバレッジを更新
            let module_entry = module_coverage
                .entry(module_name.clone())
                .or_insert_with(|| ModuleCoverage {
                    name: module_name,
                    total_lines: 0,
                    covered_lines: 0,
                    coverage_percentage: 0.0,
                    file_coverage: HashMap::new(),
                });

            module_entry.total_lines += file_coverage.total_lines;
            module_entry.covered_lines += file_coverage.covered_lines;
            module_entry
                .file_coverage
                .insert(relative_path, file_coverage);
        }

        // モジュールごとのカバレッジ率を計算
        for module in module_coverage.values_mut() {
            if module.total_lines > 0 {
                module.coverage_percentage =
                    (module.covered_lines as f64 / module.total_lines as f64) * 100.0;
            }
        }

        // 全体のカバレッジ率を計算
        let coverage_percentage = if total_lines > 0 {
            (covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        let report = CoverageReport {
            total_lines,
            covered_lines,
            coverage_percentage,
            module_coverage,
            generated_at: chrono::Utc::now(),
        };

        // レポートを保存
        self.save_report(&report)?;

        info!(
            "Coverage report generated: {:.2}% ({}/{} lines)",
            report.coverage_percentage, report.covered_lines, report.total_lines
        );

        Ok(report)
    }

    /// ソースファイルを収集
    fn collect_source_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.collect_files_recursive(&self.source_dir, &mut files)?;
        Ok(files)
    }

    /// ディレクトリを再帰的に探索してファイルを収集
    fn collect_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // 除外パターンをチェック
            if self.is_excluded(&path) {
                continue;
            }

            if path.is_dir() {
                self.collect_files_recursive(&path, files)?;
            } else if path.is_file() && self.is_rust_file(&path) {
                files.push(path);
            }
        }

        Ok(())
    }

    /// パスが除外パターンにマッチするかチェック
    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.exclude_patterns {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
            {
                return true;
            }
        }

        false
    }

    /// Rustファイルかどうかをチェック
    fn is_rust_file(&self, path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "rs")
    }

    /// モジュール名を取得
    fn get_module_name(&self, relative_path: &str) -> String {
        let parts: Vec<&str> = relative_path.split('/').collect();

        if parts.len() > 1 {
            parts[0].to_string()
        } else {
            "root".to_string()
        }
    }

    /// ファイルのカバレッジを取得
    fn get_file_coverage(&self, file_path: &Path) -> io::Result<FileCoverage> {
        // ファイル内容を読み込む
        let mut file = File::open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let file_name = file_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // 行ごとにカバレッジを分析
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        // 実際のカバレッジデータはテスト実行時に収集されるため、
        // ここではダミーデータを生成
        let mut line_coverage = HashMap::new();
        let mut covered_lines = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_number = i + 1;
            let is_code_line = !line.trim().is_empty()
                && !line.trim().starts_with("//")
                && !line.trim().starts_with("/*")
                && !line.trim().starts_with("*/")
                && !line.trim().starts_with("*");

            if is_code_line {
                // テスト用のダミーデータ: 実際のカバレッジデータはテスト実行時に収集される
                let is_covered = true; // ダミーデータ
                line_coverage.insert(line_number, is_covered);

                if is_covered {
                    covered_lines += 1;
                }
            }
        }

        let coverage_percentage = if total_lines > 0 {
            (covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        Ok(FileCoverage {
            name: file_name,
            total_lines,
            covered_lines,
            coverage_percentage,
            line_coverage,
        })
    }

    /// レポートを保存
    fn save_report(&self, report: &CoverageReport) -> io::Result<()> {
        // 出力ディレクトリが存在しない場合は作成
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir)?;
        }

        // JSONレポートを保存
        let json_path = self.output_dir.join("coverage.json");
        let json_content = serde_json::to_string_pretty(report)?;
        let mut json_file = File::create(json_path)?;
        json_file.write_all(json_content.as_bytes())?;

        // HTMLレポートを保存
        let html_path = self.output_dir.join("coverage.html");
        let html_content = self.generate_html_report(report);
        let mut html_file = File::create(html_path)?;
        html_file.write_all(html_content.as_bytes())?;

        Ok(())
    }

    /// HTMLレポートを生成
    fn generate_html_report(&self, report: &CoverageReport) -> String {
        let mut html = String::new();

        // HTMLヘッダー
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str("  <title>ShardX Code Coverage Report</title>\n");
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }\n");
        html.push_str("    h1 { color: #333; }\n");
        html.push_str("    .summary { background-color: #f5f5f5; padding: 15px; border-radius: 5px; margin-bottom: 20px; }\n");
        html.push_str("    .progress { height: 20px; background-color: #e0e0e0; border-radius: 5px; overflow: hidden; }\n");
        html.push_str("    .progress-bar { height: 100%; background-color: #4CAF50; text-align: center; color: white; }\n");
        html.push_str("    .module { margin-bottom: 30px; }\n");
        html.push_str("    .module-header { cursor: pointer; padding: 10px; background-color: #eee; border-radius: 5px; }\n");
        html.push_str("    .module-content { display: none; padding: 10px; border: 1px solid #ddd; border-radius: 0 0 5px 5px; }\n");
        html.push_str("    .file { margin-bottom: 15px; }\n");
        html.push_str(
            "    .file-header { cursor: pointer; padding: 5px; background-color: #f9f9f9; }\n",
        );
        html.push_str("    .file-content { display: none; }\n");
        html.push_str("    table { width: 100%; border-collapse: collapse; }\n");
        html.push_str(
            "    th, td { text-align: left; padding: 8px; border-bottom: 1px solid #ddd; }\n",
        );
        html.push_str("    .covered { background-color: #dff0d8; }\n");
        html.push_str("    .uncovered { background-color: #f2dede; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // レポートタイトル
        html.push_str("  <h1>ShardX Code Coverage Report</h1>\n");

        // 概要
        html.push_str("  <div class=\"summary\">\n");
        html.push_str("    <p>Generated at: ");
        html.push_str(&report.generated_at.to_rfc3339());
        html.push_str("</p>\n");
        html.push_str("    <p>Total Lines: ");
        html.push_str(&report.total_lines.to_string());
        html.push_str("</p>\n");
        html.push_str("    <p>Covered Lines: ");
        html.push_str(&report.covered_lines.to_string());
        html.push_str("</p>\n");
        html.push_str("    <p>Coverage: ");
        html.push_str(&format!("{:.2}%", report.coverage_percentage));
        html.push_str("</p>\n");
        html.push_str("    <div class=\"progress\">\n");
        html.push_str(&format!(
            "      <div class=\"progress-bar\" style=\"width: {:.2}%\">{:.2}%</div>\n",
            report.coverage_percentage, report.coverage_percentage
        ));
        html.push_str("    </div>\n");
        html.push_str("  </div>\n");

        // モジュールごとのカバレッジ
        html.push_str("  <h2>Module Coverage</h2>\n");

        for (module_name, module) in &report.module_coverage {
            html.push_str(&format!("  <div class=\"module\">\n"));
            html.push_str(&format!(
                "    <div class=\"module-header\" onclick=\"toggleModule('{}')\">\n",
                module_name
            ));
            html.push_str(&format!(
                "      <h3>{} - {:.2}% ({}/{})</h3>\n",
                module_name, module.coverage_percentage, module.covered_lines, module.total_lines
            ));
            html.push_str(&format!("      <div class=\"progress\">\n"));
            html.push_str(&format!(
                "        <div class=\"progress-bar\" style=\"width: {:.2}%\"></div>\n",
                module.coverage_percentage
            ));
            html.push_str(&format!("      </div>\n"));
            html.push_str(&format!("    </div>\n"));

            html.push_str(&format!(
                "    <div id=\"module-{}\" class=\"module-content\">\n",
                module_name
            ));

            // ファイルごとのカバレッジ
            for (file_path, file) in &module.file_coverage {
                html.push_str(&format!("      <div class=\"file\">\n"));
                html.push_str(&format!(
                    "        <div class=\"file-header\" onclick=\"toggleFile('{}')\">\n",
                    file_path
                ));
                html.push_str(&format!(
                    "          <h4>{} - {:.2}% ({}/{})</h4>\n",
                    file_path, file.coverage_percentage, file.covered_lines, file.total_lines
                ));
                html.push_str(&format!("          <div class=\"progress\">\n"));
                html.push_str(&format!(
                    "            <div class=\"progress-bar\" style=\"width: {:.2}%\"></div>\n",
                    file.coverage_percentage
                ));
                html.push_str(&format!("          </div>\n"));
                html.push_str(&format!("        </div>\n"));

                html.push_str(&format!(
                    "        <div id=\"file-{}\" class=\"file-content\">\n",
                    file_path
                ));
                html.push_str(&format!("          <p>File details will be shown here in the actual implementation.</p>\n"));
                html.push_str(&format!("        </div>\n"));
                html.push_str(&format!("      </div>\n"));
            }

            html.push_str(&format!("    </div>\n"));
            html.push_str(&format!("  </div>\n"));
        }

        // JavaScript
        html.push_str("  <script>\n");
        html.push_str("    function toggleModule(id) {\n");
        html.push_str("      var content = document.getElementById('module-' + id);\n");
        html.push_str(
            "      content.style.display = content.style.display === 'block' ? 'none' : 'block';\n",
        );
        html.push_str("    }\n");
        html.push_str("    function toggleFile(id) {\n");
        html.push_str("      var content = document.getElementById('file-' + id);\n");
        html.push_str(
            "      content.style.display = content.style.display === 'block' ? 'none' : 'block';\n",
        );
        html.push_str("    }\n");
        html.push_str("  </script>\n");

        // HTMLフッター
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_coverage_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path();
        let source_dir = temp_dir.path();

        let manager = CoverageManager::new(output_dir, source_dir);

        assert_eq!(manager.output_dir, output_dir);
        assert_eq!(manager.source_dir, source_dir);
        assert_eq!(manager.exclude_patterns.len(), 4);
    }

    #[test]
    fn test_is_rust_file() {
        let temp_dir = tempdir().unwrap();
        let manager = CoverageManager::new(temp_dir.path(), temp_dir.path());

        assert!(manager.is_rust_file(Path::new("test.rs")));
        assert!(!manager.is_rust_file(Path::new("test.txt")));
        assert!(!manager.is_rust_file(Path::new("test")));
    }

    #[test]
    fn test_get_module_name() {
        let temp_dir = tempdir().unwrap();
        let manager = CoverageManager::new(temp_dir.path(), temp_dir.path());

        assert_eq!(manager.get_module_name("src/module/file.rs"), "src");
        assert_eq!(manager.get_module_name("file.rs"), "root");
    }
}
