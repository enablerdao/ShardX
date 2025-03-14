const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = process.env.PORT || 54867;
const NODE_URL = process.env.NODE_URL || 'http://localhost:54868';

// MIMEタイプのマッピング
const MIME_TYPES = {
  '.html': 'text/html',
  '.css': 'text/css',
  '.js': 'text/javascript',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml'
};

// リクエストハンドラ関数（テスト可能にするために分離）
function handleRequest(req, res) {
  // CORSヘッダーを設定
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS, PUT, DELETE');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  // OPTIONSリクエストに対応
  if (req.method === 'OPTIONS') {
    res.writeHead(200);
    res.end();
    return;
  }

  // URLからパスを取得
  let filePath = req.url;
  
  // APIリクエストをノードサーバーにプロキシ
  if (filePath.startsWith('/api/')) {
    // ノードサーバーのURLを構築
    const nodeUrl = `${NODE_URL}${filePath}`;
    
    // リクエストをプロキシ
    const proxyReq = http.request(
      nodeUrl,
      {
        method: req.method,
        headers: req.headers
      },
      (proxyRes) => {
        // レスポンスヘッダーをコピー
        res.writeHead(proxyRes.statusCode, proxyRes.headers);
        
        // レスポンスボディをパイプ
        proxyRes.pipe(res);
      }
    );
    
    // エラーハンドリング
    proxyReq.on('error', (err) => {
      console.error('Proxy error:', err);
      
      // モックデータを返す（ノードサーバーが利用できない場合）
      if (filePath === '/api/info') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({
          node_id: "mock_node_" + Math.random().toString(36).substring(2, 8),
          version: "1.0.0",
          uptime: Math.floor(Math.random() * 86400),
          peers: Math.floor(Math.random() * 10) + 1,
          current_tps: Math.floor(Math.random() * 50000) + 30000,
          shard_count: 256,
          confirmed_transactions: Math.floor(Math.random() * 1000000) + 1000000
        }));
      } else if (filePath.startsWith('/api/tx/list')) {
        // デモトランザクションデータを読み込む
        try {
          const demoData = fs.readFileSync(path.join(__dirname, 'assets/demo/transactions.json'));
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(demoData);
        } catch (error) {
          res.writeHead(500, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({ error: 'Demo data not available' }));
        }
      } else {
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'Proxy error', message: err.message }));
      }
    });
    
    // リクエストボディをパイプ
    if (['POST', 'PUT'].includes(req.method)) {
      req.pipe(proxyReq);
    } else {
      proxyReq.end();
    }
    
    return;
  }
  
  // ルートパスの場合はindex.htmlを返す
  if (filePath === '/') {
    filePath = '/index.html';
  }
  
  // ウォレット/DEXページへのリクエスト
  if (filePath === '/wallet' || filePath === '/dex') {
    filePath = '/wallet_dex.html';
  }
  
  // ファイルパスを構築
  const fullPath = path.join(__dirname, filePath);
  
  // ファイルの拡張子を取得
  const extname = path.extname(fullPath);
  
  // Content-Typeを設定
  const contentType = MIME_TYPES[extname] || 'application/octet-stream';
  
  // ファイルを読み込む
  fs.readFile(fullPath, (err, content) => {
    if (err) {
      if (err.code === 'ENOENT') {
        // ファイルが見つからない場合は404を返す
        res.writeHead(404, { 'Content-Type': 'text/html' });
        res.end('<h1>404 Not Found</h1>');
      } else {
        // サーバーエラーの場合は500を返す
        res.writeHead(500, { 'Content-Type': 'text/html' });
        res.end(`<h1>500 Internal Server Error</h1><p>${err.code}</p>`);
      }
    } else {
      // 成功した場合はファイルを返す
      res.writeHead(200, { 'Content-Type': contentType });
      res.end(content);
    }
  });
}

// サーバーを作成
const server = http.createServer(handleRequest);

// モジュールとして実行された場合のみサーバーを起動
if (require.main === module) {
  server.listen(PORT, '0.0.0.0', () => {
    console.log(`Server running at http://localhost:${PORT}/`);
  });
}

// テスト用にエクスポート
module.exports = {
  handleRequest
};
