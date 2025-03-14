#!/bin/bash

# トランザクションテストスクリプト
cd "$(dirname "$0")"

echo "=== ShardX トランザクションテスト ==="

# アカウント作成テスト
echo "1. アカウント作成テスト"
echo "アカウント1を作成中..."
ACCOUNT1=$(curl -s -X POST http://localhost:54868/accounts -H "Content-Type: application/json" -d '{"name":"TestUser1"}' | jq -r '.id')
echo "アカウント1作成完了: $ACCOUNT1"

echo "アカウント2を作成中..."
ACCOUNT2=$(curl -s -X POST http://localhost:54868/accounts -H "Content-Type: application/json" -d '{"name":"TestUser2"}' | jq -r '.id')
echo "アカウント2作成完了: $ACCOUNT2"

# アカウント情報取得テスト
echo -e "\n2. アカウント情報取得テスト"
echo "アカウント1の情報を取得中..."
curl -s -X GET http://localhost:54868/accounts/$ACCOUNT1 | jq .

# 送金テスト
echo -e "\n3. 送金テスト"
echo "アカウント1からアカウント2へ100 SHDXを送金中..."
TRANSFER_RESULT=$(curl -s -X POST http://localhost:54868/transfer -H "Content-Type: application/json" -d "{
  \"from_account_id\": \"$ACCOUNT1\",
  \"to_account_id\": \"$ACCOUNT2\",
  \"amount\": 100,
  \"token_id\": \"SHDX\"
}")

TX_ID=$(echo $TRANSFER_RESULT | jq -r '.transaction_id')
echo "送金トランザクションID: $TX_ID"
echo "送金結果: $(echo $TRANSFER_RESULT | jq -r '.status')"

# トランザクション確認
echo -e "\n4. トランザクション確認"
echo "トランザクション $TX_ID の状態を確認中..."
curl -s -X GET http://localhost:54868/transactions/$TX_ID | jq .

# 送金後のアカウント残高確認
echo -e "\n5. 送金後のアカウント残高確認"
echo "アカウント1の残高:"
curl -s -X GET http://localhost:54868/accounts/$ACCOUNT1 | jq .
echo "アカウント2の残高:"
curl -s -X GET http://localhost:54868/accounts/$ACCOUNT2 | jq .

# DEXテスト
echo -e "\n6. DEXテスト"
echo "取引ペア (SHDX/USD) を追加中..."
curl -s -X POST http://localhost:54868/trading-pairs -H "Content-Type: application/json" -d '{
  "base": "SHDX",
  "quote": "USD"
}' | jq .

echo "買い注文を作成中..."
curl -s -X POST http://localhost:54868/orders -H "Content-Type: application/json" -d "{
  \"account_id\": \"$ACCOUNT1\",
  \"base\": \"SHDX\",
  \"quote\": \"USD\",
  \"order_type\": \"buy\",
  \"price\": 1.5,
  \"amount\": 50
}" | jq .

echo "売り注文を作成中..."
curl -s -X POST http://localhost:54868/orders -H "Content-Type: application/json" -d "{
  \"account_id\": \"$ACCOUNT2\",
  \"base\": \"SHDX\",
  \"quote\": \"USD\",
  \"order_type\": \"sell\",
  \"price\": 1.5,
  \"amount\": 25
}" | jq .

# オーダーブック確認
echo -e "\n7. オーダーブック確認"
curl -s -X GET "http://localhost:54868/order-book?base=SHDX&quote=USD" | jq .

# 取引履歴確認
echo -e "\n8. 取引履歴確認"
curl -s -X GET "http://localhost:54868/trade-history?base=SHDX&quote=USD" | jq .

echo -e "\n=== テスト完了 ==="