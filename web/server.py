from flask import Flask, jsonify, request, send_from_directory
from flask_cors import CORS
import os
import json
import random
from datetime import datetime, timedelta

app = Flask(__name__, static_folder='dist')
CORS(app, resources={r"/*": {"origins": "*"}})

# Sample data for demonstration
def generate_sample_data():
    now = datetime.now()
    data = {
        "transaction_count": [],
        "transaction_volume": [],
        "transaction_fee": [],
        "network_load": []
    }
    
    # Generate 7 days of hourly data
    for day in range(7):
        for hour in range(24):
            timestamp = now + timedelta(days=day, hours=hour)
            timestamp_str = timestamp.strftime("%Y-%m-%d %H:%M:%S")
            
            # Time-based factors
            hour_factor = 1.0 + 0.5 * (abs(hour - 12) / 12)
            day_of_week = (timestamp.weekday() + 1) % 7
            day_factor = 0.7 if day_of_week >= 5 else 1.3
            
            # Base values with some randomness
            base_count = int(100 * hour_factor * day_factor * (1 + random.random() * 0.2))
            base_volume = 10000 * hour_factor * day_factor * (1 + random.random() * 0.3)
            base_fee = 1000 * hour_factor * day_factor * (1 + random.random() * 0.25)
            base_load = 50 * hour_factor * day_factor * (1 + random.random() * 0.15)
            
            # Add to data
            data["transaction_count"].append({
                "timestamp": timestamp_str,
                "value": base_count,
                "lower": int(base_count * 0.9),
                "upper": int(base_count * 1.1)
            })
            
            data["transaction_volume"].append({
                "timestamp": timestamp_str,
                "value": base_volume,
                "lower": base_volume * 0.85,
                "upper": base_volume * 1.15
            })
            
            data["transaction_fee"].append({
                "timestamp": timestamp_str,
                "value": base_fee,
                "lower": base_fee * 0.9,
                "upper": base_fee * 1.1
            })
            
            data["network_load"].append({
                "timestamp": timestamp_str,
                "value": min(100, base_load),
                "lower": min(100, base_load * 0.9),
                "upper": min(100, base_load * 1.1)
            })
    
    return data

SAMPLE_DATA = generate_sample_data()

# Routes
@app.route('/')
def index():
    return send_from_directory('dist', 'index.html')

@app.route('/<path:path>')
def static_files(path):
    return send_from_directory('dist', path)

@app.route('/api/predictions/<target>', methods=['GET'])
def get_predictions(target):
    if target in SAMPLE_DATA:
        return jsonify(SAMPLE_DATA[target])
    return jsonify({"error": "Invalid target"}), 404

@app.route('/api/transactions/stats', methods=['GET'])
def get_transaction_stats():
    # Sample transaction statistics
    stats = {
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
    }
    return jsonify(stats)

@app.route('/api/transactions/patterns', methods=['GET'])
def get_transaction_patterns():
    # Sample transaction patterns
    patterns = [
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
    ]
    return jsonify(patterns)

@app.route('/api/transactions/anomalies', methods=['GET'])
def get_transaction_anomalies():
    # Sample transaction anomalies
    anomalies = [
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
    ]
    return jsonify(anomalies)

@app.route('/api/cross-shard/stats', methods=['GET'])
def get_cross_shard_stats():
    # Sample cross-shard transaction statistics
    stats = {
        "total": 5000,
        "completed": 4500,
        "failed": 300,
        "timed_out": 150,
        "cancelled": 50,
        "pending": 0,
        "avg_completion_time": 2500,  # milliseconds
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
    }
    return jsonify(stats)

@app.route('/api/feature-importance', methods=['GET'])
def get_feature_importance():
    # Sample feature importance
    features = [
        {"name": "amount", "importance": 0.35},
        {"name": "fee", "importance": 0.15},
        {"name": "hour_of_day", "importance": 0.25},
        {"name": "day_of_week", "importance": 0.20},
        {"name": "is_weekend", "importance": 0.05}
    ]
    return jsonify(features)

if __name__ == '__main__':
    port = int(os.environ.get('PORT', 52153))
    app.run(host='0.0.0.0', port=port, debug=True)