const express = require('express');
const path = require('path');
const cors = require('cors');
const fs = require('fs');

const app = express();
const PORT = process.env.PORT || 52153;

// CORSを有効化
app.use(cors());

// 静的ファイルを提供
app.use(express.static(path.join(__dirname, '../dist')));

// APIエンドポイント
app.get('/api/transactions/stats', (req, res) => {
  // サンプルデータ
  const stats = {
    "total_transactions": 12345,
    "successful_transactions": 12000,
    "failed_transactions": 300,
    "pending_transactions": 45,
    "total_volume": 1234567.89,
    "average_volume": 100.01,
    "max_volume": 5000.0,
    "min_volume": 10.0,
    "total_fees": 12345.67,
    "average_fee": 1.0,
    "cross_shard_transactions": 3456,
    "intra_shard_transactions": 8889
  };
  res.json(stats);
});

app.get('/api/predictions/:target', (req, res) => {
  const target = req.params.target;
  const now = new Date();
  const data = [];
  
  // 7日間の時間ごとのデータを生成
  for (let day = 0; day < 7; day++) {
    for (let hour = 0; hour < 24; hour++) {
      const timestamp = new Date(now);
      timestamp.setDate(timestamp.getDate() + day);
      timestamp.setHours(hour);
      timestamp.setMinutes(0);
      timestamp.setSeconds(0);
      
      // 時間帯による変動係数
      const hourFactor = 1.0 + 0.5 * (Math.abs(hour - 12) / 12);
      // 曜日による変動係数
      const dayOfWeek = (timestamp.getDay() + 1) % 7;
      const dayFactor = dayOfWeek >= 5 ? 0.7 : 1.3;
      
      // 基本値にランダム性を加える
      let baseValue;
      switch (target) {
        case 'transaction_count':
          baseValue = Math.round(100 * hourFactor * dayFactor * (1 + Math.random() * 0.2));
          break;
        case 'transaction_volume':
          baseValue = 10000 * hourFactor * dayFactor * (1 + Math.random() * 0.3);
          break;
        case 'transaction_fee':
          baseValue = 1000 * hourFactor * dayFactor * (1 + Math.random() * 0.25);
          break;
        case 'network_load':
          baseValue = Math.min(100, 50 * hourFactor * dayFactor * (1 + Math.random() * 0.15));
          break;
        default:
          baseValue = 100;
      }
      
      data.push({
        timestamp: timestamp.toISOString().replace('T', ' ').substring(0, 19),
        value: baseValue,
        lower: target === 'transaction_count' ? Math.round(baseValue * 0.9) : baseValue * 0.9,
        upper: target === 'transaction_count' ? Math.round(baseValue * 1.1) : baseValue * 1.1
      });
    }
  }
  
  res.json(data);
});

app.get('/api/transactions/patterns', (req, res) => {
  // サンプルデータ
  const patterns = [
    {
      "id": "pattern-1",
      "name": "循環取引 (3 アドレス)",
      "description": "3個のアドレス間で循環する取引パターンが検出されました",
      "addresses": ["addr1", "addr2", "addr3"],
      "occurrences": 5,
      "first_seen": "2023-05-01 10:30:00",
      "last_seen": "2023-05-02 14:45:00",
      "total_volume": 5000.0,
      "average_volume": 1000.0,
      "confidence": 0.85
    },
    {
      "id": "pattern-2",
      "name": "分割取引 (8 件)",
      "description": "1つのアドレスから8個の異なるアドレスへの短時間での分割取引が検出されました",
      "addresses": ["addr4", "addr5", "addr6", "addr7", "addr8", "addr9", "addr10", "addr11", "addr12"],
      "occurrences": 8,
      "first_seen": "2023-05-03 09:15:00",
      "last_seen": "2023-05-03 09:45:00",
      "total_volume": 8000.0,
      "average_volume": 1000.0,
      "confidence": 0.75
    },
    {
      "id": "pattern-3",
      "name": "統合取引 (5 件)",
      "description": "5個の異なるアドレスから1つのアドレスへの短時間での統合取引が検出されました",
      "addresses": ["addr13", "addr14", "addr15", "addr16", "addr17", "addr18"],
      "occurrences": 5,
      "first_seen": "2023-05-04 16:30:00",
      "last_seen": "2023-05-04 17:00:00",
      "total_volume": 5000.0,
      "average_volume": 1000.0,
      "confidence": 0.8
    }
  ];
  res.json(patterns);
});

app.get('/api/transactions/anomalies', (req, res) => {
  // サンプルデータ
  const anomalies = [
    {
      "id": "anomaly-1",
      "anomaly_type": "LargeTransaction",
      "description": "通常の10倍の大量取引が検出されました",
      "detection_time": "2023-05-05 11:30:00",
      "occurrence_time": "2023-05-05 11:25:00",
      "addresses": ["addr19", "addr20"],
      "severity": 0.9,
      "confidence": 0.95
    },
    {
      "id": "anomaly-2",
      "anomaly_type": "AbnormalFrequency",
      "description": "アドレス addr21 の異常な取引頻度が検出されました（平均の5.2倍、120.5件/時間）",
      "detection_time": "2023-05-06 14:45:00",
      "occurrence_time": "2023-05-06 14:00:00",
      "addresses": ["addr21"],
      "severity": 0.8,
      "confidence": 0.85
    },
    {
      "id": "anomaly-3",
      "anomaly_type": "CircularTransaction",
      "description": "複雑な循環取引パターンが検出されました",
      "detection_time": "2023-05-07 09:15:00",
      "occurrence_time": "2023-05-07 08:30:00",
      "addresses": ["addr22", "addr23", "addr24", "addr25", "addr26"],
      "severity": 0.7,
      "confidence": 0.75
    }
  ];
  res.json(anomalies);
});

app.get('/api/cross-shard/stats', (req, res) => {
  // サンプルデータ
  const stats = {
    "total": 5000,
    "completed": 4500,
    "failed": 300,
    "timed_out": 150,
    "cancelled": 50,
    "pending": 0,
    "avg_completion_time": 2500,  // milliseconds
    "transactions_by_source_shard": {
      "shard1": 1200,
      "shard2": 1500,
      "shard3": 1300,
      "shard4": 1000
    },
    "transactions_by_destination_shard": {
      "shard1": 1100,
      "shard2": 1400,
      "shard3": 1200,
      "shard4": 1300
    }
  };
  res.json(stats);
});

app.get('/api/feature-importance', (req, res) => {
  // サンプルデータ
  const features = [
    {"name": "amount", "importance": 0.35},
    {"name": "fee", "importance": 0.15},
    {"name": "hour_of_day", "importance": 0.25},
    {"name": "day_of_week", "importance": 0.20},
    {"name": "is_weekend", "importance": 0.05}
  ];
  res.json(features);
});

// その他のルートはindex.htmlにリダイレクト
app.get('*', (req, res) => {
  res.sendFile(path.join(__dirname, '../dist/index.html'));
});

// サーバー起動
app.listen(PORT, '0.0.0.0', () => {
  console.log(`Server running on http://0.0.0.0:${PORT}`);
});