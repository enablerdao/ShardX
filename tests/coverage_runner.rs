// テストカバレッジランナー
//
// このモジュールは、ShardXのテストカバレッジを実行するためのエントリーポイントを提供します。
// cargo test --features coverage を実行すると、このモジュールが呼び出されます。

mod coverage;

use coverage::CoverageManager;
use std::path::Path;

fn main() {
    // ソースディレクトリと出力ディレクトリを設定
    let source_dir = Path::new("src");
    let output_dir = Path::new("target/coverage");

    // カバレッジマネージャーを作成
    let mut coverage_manager = CoverageManager::new(output_dir, source_dir);

    // 除外パターンを追加
    coverage_manager.add_exclude_pattern("**/target/**");
    coverage_manager.add_exclude_pattern("**/tests/**");
    coverage_manager.add_exclude_pattern("**/*.md");
    coverage_manager.add_exclude_pattern("**/.git/**");

    // カバレッジレポートを生成
    match coverage_manager.generate_report() {
        Ok(report) => {
            println!("Coverage report generated successfully!");
            println!("Total lines: {}", report.total_lines);
            println!("Covered lines: {}", report.covered_lines);
            println!("Coverage: {:.2}%", report.coverage_percentage);
            println!("Report saved to: {}", output_dir.display());
        }
        Err(e) => {
            eprintln!("Failed to generate coverage report: {}", e);
            std::process::exit(1);
        }
    }
}
