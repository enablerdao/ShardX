# ShardX インストールスクリプト

このディレクトリには、様々な環境やユースケース向けのShardXインストールスクリプトが含まれています。

## 利用可能なスクリプト

### 1. ワンクリックインストール

```bash
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/one_click_install.sh | bash
```

すべての依存関係を自動的にインストールし、ShardXを起動します。初めてのユーザーに最適です。

### 2. シンプルインストール

```bash
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/simple_install.sh | bash
```

Dockerがインストールされている環境向けの、対話なしの簡単インストールです。

### 3. 開発者向けインストール

```bash
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install_scripts/dev_install.sh | bash
```

開発環境のセットアップに必要なツール（Rust、Node.js、VSCodeなど）をインストールします。

### 4. エンタープライズインストール

```bash
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install_scripts/enterprise_install.sh | bash
```

高可用性構成でShardXをインストールします。複数ノード、Redisクラスター、モニタリング、バックアップ機能を含みます。

### 5. ミニマルインストール

```bash
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install_scripts/minimal_install.sh | bash
```

最小限のリソースでShardXを実行するための軽量インストールです。リソースが限られた環境に最適です。

## カスタマイズオプション

各スクリプトは、コマンドラインパラメータを使用してカスタマイズできます。例えば：

```bash
# エンタープライズインストール（ノード数を5に設定）
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install_scripts/enterprise_install.sh | bash -s -- --node-count=5

# ミニマルインストール（Webインターフェースを無効化）
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install_scripts/minimal_install.sh | bash -s -- --disable-web=true
```

各スクリプトの詳細なオプションについては、スクリプト内のコメントを参照してください。

## トラブルシューティング

インストール中に問題が発生した場合は、以下を確認してください：

1. **権限エラー**: `sudo` を使用してスクリプトを実行してみてください
2. **依存関係エラー**: 必要な依存関係が正しくインストールされているか確認してください
3. **ポートの競合**: 54867および54868ポートが他のアプリケーションで使用されていないか確認してください

詳細なトラブルシューティングについては、[ShardXドキュメント](https://github.com/enablerdao/ShardX/blob/main/README.md)を参照してください。