#!/bin/bash
set -e

# ShardX Fly.io デプロイスクリプト
echo "ShardX Fly.io デプロイスクリプト"
echo "================================"

# 引数をチェック
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
  echo "使用方法: ./fly.sh [オプション]"
  echo "オプション:"
  echo "  --deploy       アプリケーションをデプロイします"
  echo "  --launch       新しいアプリケーションを作成してデプロイします"
  echo "  --logs         アプリケーションのログを表示します"
  echo "  --status       アプリケーションのステータスを表示します"
  echo "  --scale N      アプリケーションをN個のインスタンスにスケールします"
  echo "  --restart      アプリケーションを再起動します"
  echo "  --destroy      アプリケーションを削除します（注意: すべてのデータが失われます）"
  echo "  --help, -h     このヘルプメッセージを表示します"
  exit 0
fi

# Fly CLIがインストールされているか確認
if ! command -v flyctl &> /dev/null; then
  echo "Fly CLIがインストールされていません。インストールしています..."
  curl -L https://fly.io/install.sh | sh
  export FLYCTL_INSTALL="/root/.fly"
  export PATH="$FLYCTL_INSTALL/bin:$PATH"
fi

# 認証確認
if ! flyctl auth whoami &> /dev/null; then
  echo "Fly.ioにログインしていません。ログインしてください..."
  flyctl auth login
fi

# アプリケーション名
APP_NAME="shardx"

# コマンド処理
case "$1" in
  --deploy)
    echo "アプリケーションをデプロイしています..."
    flyctl deploy --app $APP_NAME
    ;;
  --launch)
    echo "新しいアプリケーションを作成してデプロイしています..."
    flyctl launch --name $APP_NAME --no-deploy
    
    echo "ボリュームを作成しています..."
    flyctl volumes create shardx_data --size 10 --app $APP_NAME
    
    echo "アプリケーションをデプロイしています..."
    flyctl deploy --app $APP_NAME
    ;;
  --logs)
    echo "アプリケーションのログを表示しています..."
    flyctl logs --app $APP_NAME
    ;;
  --status)
    echo "アプリケーションのステータスを表示しています..."
    flyctl status --app $APP_NAME
    ;;
  --scale)
    if [ -z "$2" ]; then
      echo "エラー: スケールするインスタンス数を指定してください"
      exit 1
    fi
    echo "アプリケーションを$2個のインスタンスにスケールしています..."
    flyctl scale count $2 --app $APP_NAME
    ;;
  --restart)
    echo "アプリケーションを再起動しています..."
    flyctl restart --app $APP_NAME
    ;;
  --destroy)
    echo "警告: アプリケーションとすべてのデータを削除します。この操作は元に戻せません。"
    read -p "続行しますか？ (y/N): " confirm
    if [ "$confirm" == "y" ] || [ "$confirm" == "Y" ]; then
      echo "アプリケーションを削除しています..."
      flyctl destroy $APP_NAME --yes
    else
      echo "操作をキャンセルしました"
    fi
    ;;
  *)
    echo "コマンドが指定されていないか、無効なコマンドです。--help でヘルプを表示します。"
    exit 1
    ;;
esac

echo "完了しました！"
exit 0