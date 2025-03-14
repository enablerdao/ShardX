/**
 * HyperFlux.io 統合テスト
 * 
 * このスクリプトは、HyperFlux.ioの主要機能の統合テストを実行します。
 * - ノード情報の取得
 * - ウォレットの作成
 * - トランザクションの作成と確認
 * - シャーディング情報の取得
 */

const axios = require('axios');
const assert = require('assert');
const crypto = require('crypto');

// テスト設定
const API_BASE_URL = process.env.API_URL || 'http://localhost:54868';
const TEST_TIMEOUT = 30000; // 30秒

// ヘルパー関数
const generateRandomString = (length = 10) => {
  return crypto.randomBytes(length).toString('hex');
};

const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

// テスト実行
async function runTests() {
  console.log('🚀 HyperFlux.io 統合テストを開始します...');
  
  try {
    // ノード情報のテスト
    console.log('\n📊 ノード情報のテスト...');
    const nodeInfo = await testNodeInfo();
    console.log('✅ ノード情報のテスト成功');
    
    // ウォレット作成のテスト
    console.log('\n👛 ウォレット作成のテスト...');
    const wallet = await testWalletCreation();
    console.log('✅ ウォレット作成のテスト成功');
    
    // トランザクション作成のテスト
    console.log('\n💸 トランザクション作成のテスト...');
    const transaction = await testTransactionCreation(wallet);
    console.log('✅ トランザクション作成のテスト成功');
    
    // トランザクション確認のテスト
    console.log('\n🔍 トランザクション確認のテスト...');
    await testTransactionConfirmation(transaction.tx_id);
    console.log('✅ トランザクション確認のテスト成功');
    
    // シャーディング情報のテスト
    console.log('\n🧩 シャーディング情報のテスト...');
    await testShardingInfo();
    console.log('✅ シャーディング情報のテスト成功');
    
    console.log('\n🎉 すべてのテストが成功しました！');
    return true;
  } catch (error) {
    console.error('\n❌ テスト失敗:', error.message);
    if (error.response) {
      console.error('レスポンス:', JSON.stringify(error.response.data, null, 2));
    }
    return false;
  }
}

// ノード情報のテスト
async function testNodeInfo() {
  const response = await axios.get(`${API_BASE_URL}/info`);
  assert.strictEqual(response.status, 200, 'ステータスコードが200であること');
  assert.ok(response.data.node_id, 'node_idが存在すること');
  assert.ok(response.data.version, 'versionが存在すること');
  assert.ok(Number.isInteger(response.data.shard_count), 'shard_countが整数であること');
  return response.data;
}

// ウォレット作成のテスト
async function testWalletCreation() {
  const password = generateRandomString(12);
  const response = await axios.post(`${API_BASE_URL}/wallet/create`, {
    password
  });
  assert.strictEqual(response.status, 201, 'ステータスコードが201であること');
  assert.ok(response.data.wallet_id, 'wallet_idが存在すること');
  assert.ok(response.data.address, 'addressが存在すること');
  assert.ok(response.data.public_key, 'public_keyが存在すること');
  return response.data;
}

// トランザクション作成のテスト
async function testTransactionCreation(wallet) {
  const payload = `Test transaction ${generateRandomString(5)}`;
  const signature = generateRandomString(64); // 実際の実装では正しい署名が必要
  
  const response = await axios.post(`${API_BASE_URL}/tx/create`, {
    parent_ids: [], // 新規トランザクションなので親なし
    payload,
    signature
  });
  
  assert.strictEqual(response.status, 201, 'ステータスコードが201であること');
  assert.ok(response.data.tx_id, 'tx_idが存在すること');
  assert.strictEqual(response.data.status, 'pending', 'statusがpendingであること');
  return response.data;
}

// トランザクション確認のテスト
async function testTransactionConfirmation(txId) {
  let confirmed = false;
  let attempts = 0;
  const maxAttempts = 10;
  
  while (!confirmed && attempts < maxAttempts) {
    attempts++;
    try {
      const response = await axios.get(`${API_BASE_URL}/tx/${txId}`);
      if (response.data.status === 'confirmed') {
        confirmed = true;
        assert.ok(response.data.confirmation_time, 'confirmation_timeが存在すること');
        assert.ok(Number.isInteger(response.data.shard_id), 'shard_idが整数であること');
      } else {
        console.log(`⏳ トランザクション確認待ち... (${attempts}/${maxAttempts})`);
        await sleep(1000); // 1秒待機
      }
    } catch (error) {
      if (error.response && error.response.status === 404) {
        console.log(`⏳ トランザクションがまだ処理されていません... (${attempts}/${maxAttempts})`);
        await sleep(1000); // 1秒待機
      } else {
        throw error;
      }
    }
  }
  
  assert.ok(confirmed, `トランザクション ${txId} が確認されること`);
}

// シャーディング情報のテスト
async function testShardingInfo() {
  const response = await axios.get(`${API_BASE_URL}/shards/info`);
  assert.strictEqual(response.status, 200, 'ステータスコードが200であること');
  assert.ok(Number.isInteger(response.data.total_shards), 'total_shardsが整数であること');
  assert.ok(Number.isInteger(response.data.active_shards), 'active_shardsが整数であること');
  assert.ok(response.data.shard_distribution, 'shard_distributionが存在すること');
  return response.data;
}

// テスト実行（タイムアウト付き）
const testPromise = runTests();
const timeoutPromise = new Promise((_, reject) => {
  setTimeout(() => reject(new Error('テストがタイムアウトしました')), TEST_TIMEOUT);
});

Promise.race([testPromise, timeoutPromise])
  .then(result => {
    process.exit(result ? 0 : 1);
  })
  .catch(error => {
    console.error('テスト実行エラー:', error);
    process.exit(1);
  });