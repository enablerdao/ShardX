const assert = require('assert');
const http = require('http');
const path = require('path');
const fs = require('fs');

// サーバーモジュールをテスト
const serverPath = path.join(__dirname, 'server.js');
assert(fs.existsSync(serverPath), 'server.js file should exist');

// サーバーモジュールをロード
const server = require('./server');
console.log('Server module loaded successfully');

// APIエンドポイントのテスト関数
function testEndpoint(endpoint, method = 'GET', body = null) {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'localhost',
      port: 57273,
      path: endpoint,
      method: method,
      headers: {
        'Content-Type': 'application/json'
      }
    };

    const req = http.request(options, (res) => {
      let data = '';
      
      res.on('data', (chunk) => {
        data += chunk;
      });
      
      res.on('end', () => {
        try {
          const jsonData = JSON.parse(data);
          resolve({ statusCode: res.statusCode, data: jsonData });
        } catch (e) {
          reject(new Error(`Failed to parse response: ${e.message}`));
        }
      });
    });
    
    req.on('error', (e) => {
      reject(new Error(`Request failed: ${e.message}`));
    });
    
    if (body) {
      req.write(JSON.stringify(body));
    }
    
    req.end();
  });
}

// メインのテスト関数
async function runTests() {
  try {
    // モックデータエンドポイントのテスト
    const mockInfoResponse = await testEndpoint('/mock-info');
    assert(mockInfoResponse.statusCode === 200, 'Mock info endpoint should return 200');
    assert(mockInfoResponse.data.id, 'Mock info should have an ID');
    assert(mockInfoResponse.data.status === 'Running', 'Mock status should be Running');
    
    // テストデータエンドポイントのテスト
    const testInfoResponse = await testEndpoint('/test-info');
    assert(testInfoResponse.statusCode === 200, 'Test info endpoint should return 200');
    assert(testInfoResponse.data.id, 'Test info should have an ID');
    assert(testInfoResponse.data.status === 'Testing', 'Test status should be Testing');
    
    console.log('All tests passed!');
    process.exit(0);
  } catch (error) {
    console.error('Test failed:', error);
    process.exit(1);
  }
}

// サーバーを起動してテストを実行
const testServer = http.createServer((req, res) => {
  // サーバーモジュールのハンドラを呼び出す
  server.handleRequest(req, res);
});

testServer.listen(57273, () => {
  console.log('Test server started on port 57273');
  runTests();
});

// 10秒後にタイムアウト
setTimeout(() => {
  console.error('Tests timed out after 10 seconds');
  process.exit(1);
}, 10000);