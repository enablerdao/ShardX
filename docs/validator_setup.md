# ShardX バリデータセットアップガイド

このガイドでは、ShardXネットワークのバリデータノードをセットアップする方法について説明します。

## バリデータになるための要件

ShardXネットワークでバリデータになるには、以下の要件を満たす必要があります：

### ハードウェア要件

- **CPU**: 8コア以上（推奨: 16コア）
- **RAM**: 32GB以上（推奨: 64GB）
- **ストレージ**: 1TB以上のSSD（推奨: 2TB NVMe SSD）
- **ネットワーク**: 1Gbps以上の安定した接続（推奨: 10Gbps）
- **冗長性**: 電源とネットワークの冗長性を確保することを推奨

### ソフトウェア要件

- **OS**: Ubuntu 20.04 LTS以降、またはDebian 11以降
- **Docker**: 最新バージョン（オプション）
- **Rust**: 最新の安定版

### トークン要件

ShardXはProof of Stake（PoS）コンセンサスメカニズムを採用しています。バリデータになるには、一定量のSHXトークンをステーキングする必要があります。

- **最小ステーキング量**: 10,000 SHX
- **推奨ステーキング量**: 50,000 SHX以上

## トークン取得方法

ShardXネットワークのバリデータになるためのSHXトークンを取得する方法はいくつかあります：

### 1. バリデータ初期配布プログラム

ShardXプロジェクトでは、ネットワーク立ち上げ時に貢献するバリデータ向けに初期配布プログラムを提供しています。

- **申請方法**: [validator@shardx.org](mailto:validator@shardx.org)にメールを送信し、以下の情報を提供してください：
  - バリデータ名
  - 技術的な経験
  - 運用予定のハードウェア仕様
  - 他のブロックチェーンでのバリデータ経験（もしあれば）

- **選考プロセス**: 申請は技術チームによってレビューされ、選考されたバリデータには初期テストネットへの参加招待が送られます。

### 2. テストネット参加報酬

テストネットに参加し、ネットワークの安定性とセキュリティに貢献することで、SHXトークンを獲得できます。

- **テストネット報酬**: テストネット期間中の貢献度に応じて、最大50,000 SHXのトークンが配布されます。
- **参加方法**: [testnet.shardx.org](https://testnet.shardx.org)からテストネット参加申請を行ってください。

### 3. エコシステム開発助成金

ShardXエコシステムの発展に貢献するプロジェクトやツールを開発する場合、開発助成金プログラムを通じてSHXトークンを獲得できます。

- **助成金額**: プロジェクトの規模と影響力に応じて、5,000〜100,000 SHXの助成金が提供されます。
- **申請方法**: [grants@shardx.org](mailto:grants@shardx.org)に提案書を送信してください。

## バリデータノードのセットアップ

### 1. 環境準備

```bash
# システムを更新
sudo apt update && sudo apt upgrade -y

# 必要なパッケージをインストール
sudo apt install -y build-essential git curl jq pkg-config libssl-dev

# Rustをインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. ShardXをインストール

```bash
# リポジトリをクローン
git clone https://github.com/enablerdao/ShardX.git
cd ShardX

# ビルド
cargo build --release

# バイナリをコピー
sudo cp target/release/shardx_node /usr/local/bin/
```

### 3. バリデータ設定

```bash
# 設定ディレクトリを作成
mkdir -p ~/.shardx/validator

# 設定ファイルを作成
cat > ~/.shardx/validator/config.toml << EOF
# ShardX バリデータ設定

[node]
name = "YourValidatorName"
role = "validator"
network = "mainnet"  # または "testnet"

[validator]
stake_amount = 10000  # ステーキングするSHX量
commission_rate = 5   # 手数料率（%）
description = "Your validator description"
website = "https://yourwebsite.com"  # オプション
contact = "your@email.com"           # オプション

[network]
listen_addr = "0.0.0.0:54321"
external_addr = "your-server-ip:54321"
seed_nodes = [
  "seed1.shardx.org:54321",
  "seed2.shardx.org:54321"
]

[storage]
path = "~/.shardx/validator/data"

[logging]
level = "info"
file = "~/.shardx/validator/logs/node.log"
EOF

# ウォレットを作成
shardx_node wallet create --path ~/.shardx/validator/wallet.json
```

### 4. ステーキング

バリデータとして参加するには、SHXトークンをステーキングする必要があります。

```bash
# ステーキングトランザクションを作成
shardx_node stake --amount 10000 --wallet ~/.shardx/validator/wallet.json
```

### 5. バリデータノードを起動

```bash
# サービスファイルを作成
sudo tee /etc/systemd/system/shardx-validator.service > /dev/null << EOF
[Unit]
Description=ShardX Validator Node
After=network.target

[Service]
User=$USER
ExecStart=/usr/local/bin/shardx_node --config ~/.shardx/validator/config.toml
Restart=on-failure
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

# サービスを有効化して起動
sudo systemctl enable shardx-validator
sudo systemctl start shardx-validator

# ステータスを確認
sudo systemctl status shardx-validator
```

## バリデータ報酬

ShardXネットワークでバリデータを運用することで、以下の報酬を得ることができます：

1. **ブロック報酬**: 新しいブロックの検証と提案に対する報酬
2. **トランザクション手数料**: 処理したトランザクションの手数料の一部
3. **ステーキング報酬**: ステーキングしたSHXトークンに対する利息（年率約5-15%）

報酬は、バリデータのパフォーマンス、ステーキング量、ネットワーク参加期間などの要素に基づいて計算されます。

## バリデータランキングとスラッシング

ShardXネットワークでは、バリデータのパフォーマンスと信頼性を評価するためのランキングシステムを採用しています。

### ランキング要素

- **アップタイム**: ノードのオンライン率
- **レスポンス時間**: トランザクション処理の速度
- **正確性**: 正確なブロック検証の割合
- **ステーキング量**: ステーキングしているSHXトークンの量
- **コミュニティ貢献**: エコシステムへの貢献度

### スラッシング条件

以下の行為は、ステーキングしたトークンの一部が没収（スラッシング）される可能性があります：

- **二重署名**: 同じ高さの異なるブロックに署名
- **長時間のオフライン**: 24時間以上オフライン
- **悪意ある行為**: ネットワークに害を与える行為

スラッシングの割合は違反の重大度によって異なり、最大でステーキング量の100%が没収される可能性があります。

## バリデータコミュニティ

ShardXバリデータコミュニティに参加して、最新情報を入手し、他のバリデータと交流することをお勧めします：

- **Discord**: [discord.gg/shardx](https://discord.gg/shardx)
- **Telegram**: [t.me/shardx_validators](https://t.me/shardx_validators)
- **フォーラム**: [forum.shardx.org](https://forum.shardx.org)

## よくある質問

### Q: 最小ステーキング量は変更されますか？
A: ネットワークの成長と採用に応じて、最小ステーキング量は調整される可能性があります。変更がある場合は、十分な告知期間を設けます。

### Q: バリデータ報酬はどのように分配されますか？
A: 報酬は各エポック（約24時間）の終了時に自動的にバリデータのウォレットに分配されます。

### Q: ステーキングしたトークンはいつでも引き出せますか？
A: ステーキングを解除するには、21日間のアンボンディング期間が必要です。この期間中、トークンはロックされ、報酬は発生しません。

### Q: 複数のバリデータノードを運用できますか？
A: 技術的には可能ですが、ネットワークの分散化を促進するため、1つのエンティティが運用するバリデータノードの数には制限があります。

### Q: ハードウェア要件は変更されますか？
A: ネットワークの成長に伴い、ハードウェア要件は将来的に増加する可能性があります。定期的に公式ドキュメントを確認することをお勧めします。

---

ご質問やサポートが必要な場合は、[support@shardx.org](mailto:support@shardx.org)までお問い合わせください。