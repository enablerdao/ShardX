# ShardX 静的解析ツール

このディレクトリには、ShardXプロジェクトの静的解析を実行するためのツールとスクリプトが含まれています。

## 概要

静的解析は、コードを実行せずにソースコードを分析し、潜在的な問題、バグ、スタイルの問題を特定するプロセスです。ShardXでは、以下の静的解析ツールを使用しています：

1. **Clippy**: Rustの公式lintツール。コードの品質、パフォーマンス、正確性に関する問題を検出します。
2. **rustfmt**: Rustの公式コードフォーマッタ。一貫したコードスタイルを維持します。
3. **rust-analyzer**: IDEと統合されたRust言語サーバー。リアルタイムでコードの問題を検出します。

## 使用方法

### 静的解析の実行

すべての静的解析ツールを実行するには：

```bash
./tools/static_analysis/run_analysis.sh --all
```

特定のツールのみを実行するには：

```bash
# Clippyのみを実行
./tools/static_analysis/run_analysis.sh --clippy

# rustfmtのみを実行
./tools/static_analysis/run_analysis.sh --fmt
```

### 問題の自動修正

可能な問題を自動的に修正するには、`--fix`オプションを使用します：

```bash
# Clippyの問題を自動修正
./tools/static_analysis/run_analysis.sh --clippy --fix

# フォーマットの問題を自動修正
./tools/static_analysis/run_analysis.sh --fmt --fix

# すべての問題を自動修正
./tools/static_analysis/run_analysis.sh --all --fix
```

### 詳細な出力

詳細な出力を表示するには、`--verbose`オプションを使用します：

```bash
./tools/static_analysis/run_analysis.sh --all --verbose
```

### ヘルプの表示

使用可能なオプションの完全なリストを表示するには：

```bash
./tools/static_analysis/run_analysis.sh --help
```

## 設定ファイル

### Clippy設定

Clippyの設定は`clippy_config.toml`ファイルで管理されています。このファイルでは、有効にするlint、無効にするlint、警告レベルなどを設定できます。

主な設定項目：

- `warn-level`: 警告レベルを設定します（allow, warn, deny, forbid）
- `lints`: 特定のlintの設定を指定します
- `lint-groups`: lintグループの有効/無効を設定します

### rustfmt設定

rustfmtの設定は`rustfmt.toml`ファイルで管理されています。このファイルでは、コードフォーマットのスタイルを設定できます。

主な設定項目：

- `max_width`: 最大行幅
- `tab_spaces`: インデントのスペース数
- `edition`: Rustのエディション
- `imports_layout`: インポートの配置方法
- `brace_style`: 波括弧のスタイル

## CI/CD統合

このツールはCI/CDパイプラインに統合されており、プルリクエストごとに自動的に実行されます。CI/CDパイプラインでは、以下のチェックが行われます：

1. Clippyによるコード品質チェック
2. rustfmtによるコードスタイルチェック

これらのチェックが失敗すると、プルリクエストはマージできません。

## ベストプラクティス

1. コードを変更する前に静的解析ツールを実行し、既存の問題を修正してください。
2. 新しいコードを追加した後、再度静的解析ツールを実行して、新しい問題が導入されていないことを確認してください。
3. 可能な限り、`--fix`オプションを使用して問題を自動修正してください。
4. 自動修正できない問題については、手動で修正してください。
5. 特定のlintを無効にする必要がある場合は、コード内で`#[allow(lint_name)]`属性を使用するのではなく、設定ファイルで設定してください。

## トラブルシューティング

### よくある問題

1. **「command not found」エラー**:
   スクリプトに実行権限があることを確認してください：
   ```bash
   chmod +x tools/static_analysis/run_analysis.sh
   ```

2. **Clippyのエラー**:
   Clippyが最新バージョンであることを確認してください：
   ```bash
   rustup update
   rustup component add clippy
   ```

3. **rustfmtのエラー**:
   rustfmtが最新バージョンであることを確認してください：
   ```bash
   rustup update
   rustup component add rustfmt
   ```

### サポート

静的解析ツールに関する問題やフィードバックがある場合は、GitHubのIssueを作成してください。