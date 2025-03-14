#!/bin/bash

# ShardX インストールスクリプト
# このスクリプトは、ShardXノードとWebインターフェースをインストールして起動します。
# 注意: このスクリプトは対話的な入力を必要とします。
# 対話的な入力なしでインストールするには、auto_install.sh を使用してください。

set -e

# 自動インストールスクリプトにリダイレクト
echo "対話的な入力を必要とするインストールスクリプトは問題が発生する場合があります。"
echo "代わりに自動インストールスクリプトを使用します..."
echo ""

# auto_install.shをダウンロードして実行
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/auto_install.sh -o /tmp/auto_install.sh
chmod +x /tmp/auto_install.sh
/tmp/auto_install.sh

# 一時ファイルを削除
rm /tmp/auto_install.sh