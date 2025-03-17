#!/usr/bin/env python3

"""
ShardX ボトルネック分析ツール

このスクリプトは、ベンチマークとプロファイリングの結果を分析し、
パフォーマンスボトルネックを特定するためのツールです。
"""

import os
import sys
import json
import argparse
import re
import matplotlib.pyplot as plt
import numpy as np
from datetime import datetime
from pathlib import Path

# デフォルトのディレクトリ
DEFAULT_BENCHMARK_DIR = "target/benchmark"
DEFAULT_PROFILE_DIR = "target/profile"
DEFAULT_OUTPUT_DIR = "target/analysis"

def parse_args():
    """コマンドライン引数を解析する"""
    parser = argparse.ArgumentParser(description="ShardX ボトルネック分析ツール")
    parser.add_argument("--benchmark-dir", default=DEFAULT_BENCHMARK_DIR,
                        help=f"ベンチマーク結果のディレクトリ（デフォルト: {DEFAULT_BENCHMARK_DIR}）")
    parser.add_argument("--profile-dir", default=DEFAULT_PROFILE_DIR,
                        help=f"プロファイリング結果のディレクトリ（デフォルト: {DEFAULT_PROFILE_DIR}）")
    parser.add_argument("--output-dir", default=DEFAULT_OUTPUT_DIR,
                        help=f"分析結果の出力ディレクトリ（デフォルト: {DEFAULT_OUTPUT_DIR}）")
    parser.add_argument("--type", choices=["transaction", "sharding", "storage", "network", "all"],
                        default="all", help="分析するベンチマークの種類（デフォルト: all）")
    parser.add_argument("--format", choices=["text", "json", "html", "all"],
                        default="all", help="出力形式（デフォルト: all）")
    parser.add_argument("--threshold", type=float, default=10.0,
                        help="ボトルネックと見なすパフォーマンス低下の閾値（パーセント）（デフォルト: 10.0）")
    return parser.parse_args()

def find_latest_files(directory, pattern):
    """指定されたパターンに一致する最新のファイルを見つける"""
    files = list(Path(directory).glob(pattern))
    if not files:
        return None
    return max(files, key=os.path.getmtime)

def parse_benchmark_results(file_path):
    """ベンチマーク結果を解析する"""
    if not file_path or not os.path.exists(file_path):
        return None
    
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except json.JSONDecodeError:
        print(f"エラー: {file_path} の解析に失敗しました。有効なJSONファイルではありません。")
        return None
    except Exception as e:
        print(f"エラー: {file_path} の読み込み中にエラーが発生しました: {e}")
        return None

def parse_cpu_profile(file_path):
    """CPUプロファイル結果を解析する"""
    if not file_path or not os.path.exists(file_path):
        return None
    
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        # ホットスポットを抽出
        hotspots = []
        overhead_section = re.search(r'# Overhead.*?\n(.*?)(?:\n\n|\Z)', content, re.DOTALL)
        if overhead_section:
            lines = overhead_section.group(1).strip().split('\n')
            for line in lines:
                if not line.strip() or line.startswith('#'):
                    continue
                
                # 行を解析してホットスポット情報を抽出
                parts = re.split(r'\s+', line.strip(), maxsplit=5)
                if len(parts) >= 5:
                    try:
                        overhead = float(parts[0].strip('%'))
                        function = parts[-1]
                        hotspots.append({
                            'overhead': overhead,
                            'function': function
                        })
                    except (ValueError, IndexError):
                        continue
        
        return {
            'hotspots': hotspots
        }
    except Exception as e:
        print(f"エラー: {file_path} の読み込み中にエラーが発生しました: {e}")
        return None

def parse_memory_profile(file_path):
    """メモリプロファイル結果を解析する"""
    if not file_path or not os.path.exists(file_path):
        return None
    
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        # ヒープ概要を抽出
        heap_summary = {}
        summary_match = re.search(r'==\d+== Heap Summary:.*?\n(.*?)(?:==\d+==\n\n|\Z)', content, re.DOTALL)
        if summary_match:
            summary_text = summary_match.group(1)
            
            # 合計ヒープ使用量を抽出
            total_match = re.search(r'==\d+==\s+total heap usage:\s+([\d,]+)\s+allocs,\s+([\d,]+)\s+frees,\s+([\d,]+)\s+bytes allocated', summary_text)
            if total_match:
                heap_summary['total_allocs'] = int(total_match.group(1).replace(',', ''))
                heap_summary['total_frees'] = int(total_match.group(2).replace(',', ''))
                heap_summary['total_bytes'] = int(total_match.group(3).replace(',', ''))
        
        # メモリ割り当てのホットスポットを抽出
        memory_hotspots = []
        detailed_match = re.search(r'==\d+== \d+ bytes in \d+ blocks are definitely lost.*?\n(.*?)(?:==\d+==\n\n|\Z)', content, re.DOTALL)
        if detailed_match:
            detailed_text = detailed_match.group(1)
            
            # 各割り当てサイトを抽出
            for block in re.finditer(r'==\d+==\s+([\d,]+)\s+bytes.*?\n(.*?)(?:==\d+==\n\n|==\d+==\s+[\d,]+\s+bytes|\Z)', detailed_text, re.DOTALL):
                try:
                    bytes_lost = int(block.group(1).replace(',', ''))
                    allocation_stack = block.group(2).strip()
                    
                    # 関数名を抽出
                    function_match = re.search(r'==\d+==\s+by\s+\d+:\s+(.*?)(?:\(|\n)', allocation_stack)
                    function = function_match.group(1).strip() if function_match else "Unknown"
                    
                    memory_hotspots.append({
                        'bytes_lost': bytes_lost,
                        'function': function
                    })
                except (ValueError, IndexError, AttributeError):
                    continue
        
        return {
            'heap_summary': heap_summary,
            'memory_hotspots': memory_hotspots
        }
    except Exception as e:
        print(f"エラー: {file_path} の読み込み中にエラーが発生しました: {e}")
        return None

def analyze_transaction_performance(benchmark_results):
    """トランザクション処理のパフォーマンスを分析する"""
    if not benchmark_results or 'transaction_benchmark' not in benchmark_results:
        return None
    
    transaction_data = benchmark_results['transaction_benchmark']
    
    # スループットデータを分析
    throughput_analysis = None
    if 'throughput' in transaction_data:
        throughput = transaction_data['throughput']
        
        # トランザクション数ごとのスループットを分析
        tx_counts = [item['tx_count'] for item in throughput]
        tps_values = [item['throughput_tps'] for item in throughput]
        
        # スケーラビリティを評価
        scalability = []
        if len(tx_counts) > 1:
            for i in range(1, len(tx_counts)):
                tx_ratio = tx_counts[i] / tx_counts[i-1]
                tps_ratio = tps_values[i] / tps_values[i-1]
                efficiency = tps_ratio / tx_ratio
                scalability.append({
                    'tx_count_from': tx_counts[i-1],
                    'tx_count_to': tx_counts[i],
                    'tps_from': tps_values[i-1],
                    'tps_to': tps_values[i],
                    'efficiency': efficiency
                })
        
        throughput_analysis = {
            'data': throughput,
            'scalability': scalability,
            'min_tps': min(tps_values),
            'max_tps': max(tps_values),
            'avg_tps': sum(tps_values) / len(tps_values)
        }
    
    return {
        'throughput_analysis': throughput_analysis
    }

def analyze_sharding_performance(benchmark_results):
    """シャーディングのパフォーマンスを分析する"""
    if not benchmark_results or 'sharding_benchmark' not in benchmark_results:
        return None
    
    sharding_data = benchmark_results['sharding_benchmark']
    
    # シャード作成のパフォーマンスを分析
    creation_analysis = None
    if 'shard_creation' in sharding_data:
        creation = sharding_data['shard_creation']
        
        # シャード数ごとの作成スループットを分析
        shard_counts = [item['shard_count'] for item in creation]
        sps_values = [item['throughput_sps'] for item in creation]
        
        # スケーラビリティを評価
        scalability = []
        if len(shard_counts) > 1:
            for i in range(1, len(shard_counts)):
                count_ratio = shard_counts[i] / shard_counts[i-1]
                sps_ratio = sps_values[i] / sps_values[i-1]
                efficiency = sps_ratio / count_ratio
                scalability.append({
                    'shard_count_from': shard_counts[i-1],
                    'shard_count_to': shard_counts[i],
                    'sps_from': sps_values[i-1],
                    'sps_to': sps_values[i],
                    'efficiency': efficiency
                })
        
        creation_analysis = {
            'data': creation,
            'scalability': scalability,
            'min_sps': min(sps_values),
            'max_sps': max(sps_values),
            'avg_sps': sum(sps_values) / len(sps_values)
        }
    
    # クロスシャードトランザクションのパフォーマンスを分析
    cross_shard_analysis = None
    if 'cross_shard_transactions' in sharding_data:
        cross_shard = sharding_data['cross_shard_transactions']
        
        # トランザクション数ごとのスループットを分析
        tx_counts = [item['tx_count'] for item in cross_shard]
        tps_values = [item['throughput_tps'] for item in cross_shard]
        
        # スケーラビリティを評価
        scalability = []
        if len(tx_counts) > 1:
            for i in range(1, len(tx_counts)):
                tx_ratio = tx_counts[i] / tx_counts[i-1]
                tps_ratio = tps_values[i] / tps_values[i-1]
                efficiency = tps_ratio / tx_ratio
                scalability.append({
                    'tx_count_from': tx_counts[i-1],
                    'tx_count_to': tx_counts[i],
                    'tps_from': tps_values[i-1],
                    'tps_to': tps_values[i],
                    'efficiency': efficiency
                })
        
        cross_shard_analysis = {
            'data': cross_shard,
            'scalability': scalability,
            'min_tps': min(tps_values),
            'max_tps': max(tps_values),
            'avg_tps': sum(tps_values) / len(tps_values)
        }
    
    return {
        'creation_analysis': creation_analysis,
        'cross_shard_analysis': cross_shard_analysis
    }

def identify_bottlenecks(analysis_results, cpu_profile, memory_profile, threshold):
    """パフォーマンスボトルネックを特定する"""
    bottlenecks = []
    
    # トランザクション処理のボトルネックを特定
    if 'transaction_analysis' in analysis_results and analysis_results['transaction_analysis']:
        tx_analysis = analysis_results['transaction_analysis']
        
        # スループットのスケーラビリティを評価
        if 'throughput_analysis' in tx_analysis and tx_analysis['throughput_analysis']:
            throughput = tx_analysis['throughput_analysis']
            
            for item in throughput.get('scalability', []):
                if item['efficiency'] < (1.0 - threshold / 100):
                    bottlenecks.append({
                        'type': 'transaction_scalability',
                        'severity': 'high' if item['efficiency'] < 0.5 else 'medium',
                        'description': f"トランザクション数が{item['tx_count_from']}から{item['tx_count_to']}に増加した際のスケーラビリティが低下（効率: {item['efficiency']:.2f}）",
                        'recommendation': "並列処理の最適化、バッチ処理の導入、またはリソース競合の削減を検討してください。"
                    })
    
    # シャーディングのボトルネックを特定
    if 'sharding_analysis' in analysis_results and analysis_results['sharding_analysis']:
        shard_analysis = analysis_results['sharding_analysis']
        
        # シャード作成のスケーラビリティを評価
        if 'creation_analysis' in shard_analysis and shard_analysis['creation_analysis']:
            creation = shard_analysis['creation_analysis']
            
            for item in creation.get('scalability', []):
                if item['efficiency'] < (1.0 - threshold / 100):
                    bottlenecks.append({
                        'type': 'shard_creation_scalability',
                        'severity': 'high' if item['efficiency'] < 0.5 else 'medium',
                        'description': f"シャード数が{item['shard_count_from']}から{item['shard_count_to']}に増加した際のスケーラビリティが低下（効率: {item['efficiency']:.2f}）",
                        'recommendation': "シャード作成プロセスの並列化、メタデータ管理の最適化を検討してください。"
                    })
        
        # クロスシャードトランザクションのスケーラビリティを評価
        if 'cross_shard_analysis' in shard_analysis and shard_analysis['cross_shard_analysis']:
            cross_shard = shard_analysis['cross_shard_analysis']
            
            for item in cross_shard.get('scalability', []):
                if item['efficiency'] < (1.0 - threshold / 100):
                    bottlenecks.append({
                        'type': 'cross_shard_scalability',
                        'severity': 'high' if item['efficiency'] < 0.5 else 'medium',
                        'description': f"クロスシャードトランザクション数が{item['tx_count_from']}から{item['tx_count_to']}に増加した際のスケーラビリティが低下（効率: {item['efficiency']:.2f}）",
                        'recommendation': "シャード間通信の最適化、バッチ処理の導入、またはシャード配置アルゴリズムの改善を検討してください。"
                    })
    
    # CPUプロファイルからボトルネックを特定
    if cpu_profile and 'hotspots' in cpu_profile:
        for hotspot in cpu_profile['hotspots'][:5]:  # 上位5つのホットスポットを分析
            if hotspot['overhead'] > threshold:
                bottlenecks.append({
                    'type': 'cpu_hotspot',
                    'severity': 'high' if hotspot['overhead'] > 30 else ('medium' if hotspot['overhead'] > 15 else 'low'),
                    'description': f"CPU使用率の{hotspot['overhead']:.1f}%が関数 '{hotspot['function']}' に集中しています",
                    'recommendation': "この関数のアルゴリズムの最適化、キャッシュの活用、または並列処理の導入を検討してください。"
                })
    
    # メモリプロファイルからボトルネックを特定
    if memory_profile:
        if 'heap_summary' in memory_profile and memory_profile['heap_summary']:
            heap = memory_profile['heap_summary']
            
            # メモリリークを検出
            if 'total_allocs' in heap and 'total_frees' in heap:
                leak_count = heap['total_allocs'] - heap['total_frees']
                if leak_count > 0 and leak_count / heap['total_allocs'] > threshold / 100:
                    bottlenecks.append({
                        'type': 'memory_leak',
                        'severity': 'high',
                        'description': f"メモリリークの可能性: {leak_count}個のアロケーション（全体の{leak_count / heap['total_allocs'] * 100:.1f}%）が解放されていません",
                        'recommendation': "リソース管理を見直し、すべてのメモリが適切に解放されていることを確認してください。"
                    })
        
        if 'memory_hotspots' in memory_profile:
            for hotspot in memory_profile['memory_hotspots'][:5]:  # 上位5つのホットスポットを分析
                bottlenecks.append({
                    'type': 'memory_hotspot',
                    'severity': 'medium',
                    'description': f"関数 '{hotspot['function']}' で{hotspot['bytes_lost']}バイトのメモリが失われています",
                    'recommendation': "この関数のメモリ管理を見直し、すべてのリソースが適切に解放されていることを確認してください。"
                })
    
    return bottlenecks

def generate_charts(analysis_results, output_dir):
    """分析結果からチャートを生成する"""
    os.makedirs(output_dir, exist_ok=True)
    charts = []
    
    # トランザクション処理のチャート
    if 'transaction_analysis' in analysis_results and analysis_results['transaction_analysis']:
        tx_analysis = analysis_results['transaction_analysis']
        
        if 'throughput_analysis' in tx_analysis and tx_analysis['throughput_analysis']:
            throughput = tx_analysis['throughput_analysis']
            
            if 'data' in throughput and throughput['data']:
                # トランザクションスループットのチャート
                plt.figure(figsize=(10, 6))
                tx_counts = [item['tx_count'] for item in throughput['data']]
                tps_values = [item['throughput_tps'] for item in throughput['data']]
                
                plt.plot(tx_counts, tps_values, 'o-', linewidth=2)
                plt.xlabel('トランザクション数')
                plt.ylabel('スループット (TPS)')
                plt.title('トランザクション数とスループットの関係')
                plt.grid(True)
                plt.tight_layout()
                
                chart_path = os.path.join(output_dir, 'transaction_throughput.png')
                plt.savefig(chart_path)
                plt.close()
                
                charts.append({
                    'title': 'トランザクション数とスループットの関係',
                    'path': chart_path,
                    'description': 'トランザクション数の増加に対するスループットの変化を示します。'
                })
                
                # スケーラビリティのチャート
                if 'scalability' in throughput and throughput['scalability']:
                    plt.figure(figsize=(10, 6))
                    tx_pairs = [f"{item['tx_count_from']}-{item['tx_count_to']}" for item in throughput['scalability']]
                    efficiency = [item['efficiency'] for item in throughput['scalability']]
                    
                    plt.bar(tx_pairs, efficiency)
                    plt.axhline(y=1.0, color='r', linestyle='-', alpha=0.3)
                    plt.xlabel('トランザクション数の範囲')
                    plt.ylabel('スケーラビリティ効率')
                    plt.title('トランザクション処理のスケーラビリティ')
                    plt.grid(True, axis='y')
                    plt.tight_layout()
                    
                    chart_path = os.path.join(output_dir, 'transaction_scalability.png')
                    plt.savefig(chart_path)
                    plt.close()
                    
                    charts.append({
                        'title': 'トランザクション処理のスケーラビリティ',
                        'path': chart_path,
                        'description': 'トランザクション数の増加に対するスケーラビリティ効率を示します。効率が1.0に近いほど理想的です。'
                    })
    
    # シャーディングのチャート
    if 'sharding_analysis' in analysis_results and analysis_results['sharding_analysis']:
        shard_analysis = analysis_results['sharding_analysis']
        
        # シャード作成のチャート
        if 'creation_analysis' in shard_analysis and shard_analysis['creation_analysis']:
            creation = shard_analysis['creation_analysis']
            
            if 'data' in creation and creation['data']:
                plt.figure(figsize=(10, 6))
                shard_counts = [item['shard_count'] for item in creation['data']]
                sps_values = [item['throughput_sps'] for item in creation['data']]
                
                plt.plot(shard_counts, sps_values, 'o-', linewidth=2)
                plt.xlabel('シャード数')
                plt.ylabel('スループット (シャード/秒)')
                plt.title('シャード作成のパフォーマンス')
                plt.grid(True)
                plt.tight_layout()
                
                chart_path = os.path.join(output_dir, 'shard_creation_performance.png')
                plt.savefig(chart_path)
                plt.close()
                
                charts.append({
                    'title': 'シャード作成のパフォーマンス',
                    'path': chart_path,
                    'description': 'シャード数の増加に対する作成スループットの変化を示します。'
                })
        
        # クロスシャードトランザクションのチャート
        if 'cross_shard_analysis' in shard_analysis and shard_analysis['cross_shard_analysis']:
            cross_shard = shard_analysis['cross_shard_analysis']
            
            if 'data' in cross_shard and cross_shard['data']:
                plt.figure(figsize=(10, 6))
                tx_counts = [item['tx_count'] for item in cross_shard['data']]
                tps_values = [item['throughput_tps'] for item in cross_shard['data']]
                
                plt.plot(tx_counts, tps_values, 'o-', linewidth=2)
                plt.xlabel('トランザクション数')
                plt.ylabel('スループット (TPS)')
                plt.title('クロスシャードトランザクションのパフォーマンス')
                plt.grid(True)
                plt.tight_layout()
                
                chart_path = os.path.join(output_dir, 'cross_shard_performance.png')
                plt.savefig(chart_path)
                plt.close()
                
                charts.append({
                    'title': 'クロスシャードトランザクションのパフォーマンス',
                    'path': chart_path,
                    'description': 'クロスシャードトランザクション数の増加に対するスループットの変化を示します。'
                })
    
    # ボトルネックの概要チャート
    if 'bottlenecks' in analysis_results and analysis_results['bottlenecks']:
        bottlenecks = analysis_results['bottlenecks']
        
        # ボトルネックタイプごとの数をカウント
        bottleneck_types = {}
        for bottleneck in bottlenecks:
            bottleneck_type = bottleneck['type']
            if bottleneck_type in bottleneck_types:
                bottleneck_types[bottleneck_type] += 1
            else:
                bottleneck_types[bottleneck_type] = 1
        
        if bottleneck_types:
            plt.figure(figsize=(10, 6))
            types = list(bottleneck_types.keys())
            counts = list(bottleneck_types.values())
            
            plt.bar(types, counts)
            plt.xlabel('ボトルネックタイプ')
            plt.ylabel('数')
            plt.title('ボトルネックの概要')
            plt.xticks(rotation=45, ha='right')
            plt.tight_layout()
            
            chart_path = os.path.join(output_dir, 'bottleneck_summary.png')
            plt.savefig(chart_path)
            plt.close()
            
            charts.append({
                'title': 'ボトルネックの概要',
                'path': chart_path,
                'description': '検出されたボトルネックのタイプごとの数を示します。'
            })
    
    return charts

def generate_text_report(analysis_results, output_dir):
    """テキスト形式のレポートを生成する"""
    os.makedirs(output_dir, exist_ok=True)
    report_path = os.path.join(output_dir, 'bottleneck_analysis.txt')
    
    with open(report_path, 'w') as f:
        f.write("ShardX パフォーマンスボトルネック分析レポート\n")
        f.write("=" * 50 + "\n\n")
        f.write(f"生成日時: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        
        # ボトルネックの概要
        if 'bottlenecks' in analysis_results and analysis_results['bottlenecks']:
            bottlenecks = analysis_results['bottlenecks']
            f.write(f"検出されたボトルネック: {len(bottlenecks)}件\n\n")
            
            # 重要度ごとに分類
            high_severity = [b for b in bottlenecks if b['severity'] == 'high']
            medium_severity = [b for b in bottlenecks if b['severity'] == 'medium']
            low_severity = [b for b in bottlenecks if b['severity'] == 'low']
            
            f.write(f"重要度の高いボトルネック: {len(high_severity)}件\n")
            f.write(f"重要度の中程度のボトルネック: {len(medium_severity)}件\n")
            f.write(f"重要度の低いボトルネック: {len(low_severity)}件\n\n")
            
            # 詳細なボトルネック情報
            f.write("ボトルネックの詳細\n")
            f.write("-" * 50 + "\n\n")
            
            for i, bottleneck in enumerate(bottlenecks, 1):
                f.write(f"ボトルネック #{i} ({bottleneck['severity']})\n")
                f.write(f"タイプ: {bottleneck['type']}\n")
                f.write(f"説明: {bottleneck['description']}\n")
                f.write(f"推奨対策: {bottleneck['recommendation']}\n\n")
        else:
            f.write("ボトルネックは検出されませんでした。\n\n")
        
        # トランザクション分析
        if 'transaction_analysis' in analysis_results and analysis_results['transaction_analysis']:
            tx_analysis = analysis_results['transaction_analysis']
            f.write("トランザクション処理の分析\n")
            f.write("-" * 50 + "\n\n")
            
            if 'throughput_analysis' in tx_analysis and tx_analysis['throughput_analysis']:
                throughput = tx_analysis['throughput_analysis']
                f.write(f"最小スループット: {throughput['min_tps']:.2f} TPS\n")
                f.write(f"最大スループット: {throughput['max_tps']:.2f} TPS\n")
                f.write(f"平均スループット: {throughput['avg_tps']:.2f} TPS\n\n")
                
                if 'scalability' in throughput and throughput['scalability']:
                    f.write("スケーラビリティ分析:\n")
                    for item in throughput['scalability']:
                        f.write(f"  トランザクション数 {item['tx_count_from']} → {item['tx_count_to']}: ")
                        f.write(f"効率 = {item['efficiency']:.2f} ")
                        if item['efficiency'] >= 0.9:
                            f.write("(良好)\n")
                        elif item['efficiency'] >= 0.7:
                            f.write("(許容範囲)\n")
                        else:
                            f.write("(改善が必要)\n")
                    f.write("\n")
        
        # シャーディング分析
        if 'sharding_analysis' in analysis_results and analysis_results['sharding_analysis']:
            shard_analysis = analysis_results['sharding_analysis']
            f.write("シャーディングの分析\n")
            f.write("-" * 50 + "\n\n")
            
            if 'creation_analysis' in shard_analysis and shard_analysis['creation_analysis']:
                creation = shard_analysis['creation_analysis']
                f.write("シャード作成パフォーマンス:\n")
                f.write(f"最小スループット: {creation['min_sps']:.2f} シャード/秒\n")
                f.write(f"最大スループット: {creation['max_sps']:.2f} シャード/秒\n")
                f.write(f"平均スループット: {creation['avg_sps']:.2f} シャード/秒\n\n")
            
            if 'cross_shard_analysis' in shard_analysis and shard_analysis['cross_shard_analysis']:
                cross_shard = shard_analysis['cross_shard_analysis']
                f.write("クロスシャードトランザクションパフォーマンス:\n")
                f.write(f"最小スループット: {cross_shard['min_tps']:.2f} TPS\n")
                f.write(f"最大スループット: {cross_shard['max_tps']:.2f} TPS\n")
                f.write(f"平均スループット: {cross_shard['avg_tps']:.2f} TPS\n\n")
        
        # 推奨事項
        f.write("推奨事項\n")
        f.write("-" * 50 + "\n\n")
        
        if 'bottlenecks' in analysis_results and analysis_results['bottlenecks']:
            for bottleneck in analysis_results['bottlenecks']:
                f.write(f"- {bottleneck['recommendation']}\n")
        else:
            f.write("現在のパフォーマンスは良好です。定期的なモニタリングを継続してください。\n")
    
    return report_path

def generate_json_report(analysis_results, output_dir):
    """JSON形式のレポートを生成する"""
    os.makedirs(output_dir, exist_ok=True)
    report_path = os.path.join(output_dir, 'bottleneck_analysis.json')
    
    # 結果にタイムスタンプを追加
    report_data = analysis_results.copy()
    report_data['timestamp'] = datetime.now().isoformat()
    
    with open(report_path, 'w') as f:
        json.dump(report_data, f, indent=2)
    
    return report_path

def generate_html_report(analysis_results, charts, output_dir):
    """HTML形式のレポートを生成する"""
    os.makedirs(output_dir, exist_ok=True)
    report_path = os.path.join(output_dir, 'bottleneck_analysis.html')
    
    with open(report_path, 'w') as f:
        f.write("<!DOCTYPE html>\n")
        f.write("<html lang=\"ja\">\n")
        f.write("<head>\n")
        f.write("  <meta charset=\"UTF-8\">\n")
        f.write("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n")
        f.write("  <title>ShardX パフォーマンスボトルネック分析レポート</title>\n")
        f.write("  <style>\n")
        f.write("    body { font-family: Arial, sans-serif; margin: 0; padding: 20px; line-height: 1.6; }\n")
        f.write("    h1, h2, h3 { color: #333; }\n")
        f.write("    .container { max-width: 1200px; margin: 0 auto; }\n")
        f.write("    .summary { background-color: #f5f5f5; padding: 15px; border-radius: 5px; margin-bottom: 20px; }\n")
        f.write("    .bottleneck { margin-bottom: 15px; padding: 10px; border-left: 4px solid #ccc; }\n")
        f.write("    .bottleneck.high { border-color: #d9534f; background-color: #f9e6e6; }\n")
        f.write("    .bottleneck.medium { border-color: #f0ad4e; background-color: #fcf8e3; }\n")
        f.write("    .bottleneck.low { border-color: #5bc0de; background-color: #e8f4f8; }\n")
        f.write("    .chart { margin-bottom: 30px; }\n")
        f.write("    .chart img { max-width: 100%; height: auto; }\n")
        f.write("    table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }\n")
        f.write("    th, td { padding: 8px; text-align: left; border-bottom: 1px solid #ddd; }\n")
        f.write("    th { background-color: #f2f2f2; }\n")
        f.write("  </style>\n")
        f.write("</head>\n")
        f.write("<body>\n")
        f.write("  <div class=\"container\">\n")
        
        # レポートヘッダー
        f.write("    <h1>ShardX パフォーマンスボトルネック分析レポート</h1>\n")
        f.write(f"    <p>生成日時: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>\n")
        
        # ボトルネックの概要
        f.write("    <div class=\"summary\">\n")
        if 'bottlenecks' in analysis_results and analysis_results['bottlenecks']:
            bottlenecks = analysis_results['bottlenecks']
            high_severity = [b for b in bottlenecks if b['severity'] == 'high']
            medium_severity = [b for b in bottlenecks if b['severity'] == 'medium']
            low_severity = [b for b in bottlenecks if b['severity'] == 'low']
            
            f.write(f"      <h2>検出されたボトルネック: {len(bottlenecks)}件</h2>\n")
            f.write("      <ul>\n")
            f.write(f"        <li>重要度の高いボトルネック: {len(high_severity)}件</li>\n")
            f.write(f"        <li>重要度の中程度のボトルネック: {len(medium_severity)}件</li>\n")
            f.write(f"        <li>重要度の低いボトルネック: {len(low_severity)}件</li>\n")
            f.write("      </ul>\n")
        else:
            f.write("      <h2>ボトルネックは検出されませんでした</h2>\n")
            f.write("      <p>現在のパフォーマンスは良好です。定期的なモニタリングを継続してください。</p>\n")
        f.write("    </div>\n")
        
        # チャート
        if charts:
            f.write("    <h2>パフォーマンスチャート</h2>\n")
            for chart in charts:
                f.write("    <div class=\"chart\">\n")
                f.write(f"      <h3>{chart['title']}</h3>\n")
                f.write(f"      <img src=\"{os.path.basename(chart['path'])}\" alt=\"{chart['title']}\">\n")
                f.write(f"      <p>{chart['description']}</p>\n")
                f.write("    </div>\n")
        
        # ボトルネックの詳細
        if 'bottlenecks' in analysis_results and analysis_results['bottlenecks']:
            f.write("    <h2>ボトルネックの詳細</h2>\n")
            
            for bottleneck in analysis_results['bottlenecks']:
                f.write(f"    <div class=\"bottleneck {bottleneck['severity']}\">\n")
                f.write(f"      <h3>{bottleneck['type']}</h3>\n")
                f.write(f"      <p><strong>重要度:</strong> {bottleneck['severity']}</p>\n")
                f.write(f"      <p><strong>説明:</strong> {bottleneck['description']}</p>\n")
                f.write(f"      <p><strong>推奨対策:</strong> {bottleneck['recommendation']}</p>\n")
                f.write("    </div>\n")
        
        # トランザクション分析
        if 'transaction_analysis' in analysis_results and analysis_results['transaction_analysis']:
            tx_analysis = analysis_results['transaction_analysis']
            f.write("    <h2>トランザクション処理の分析</h2>\n")
            
            if 'throughput_analysis' in tx_analysis and tx_analysis['throughput_analysis']:
                throughput = tx_analysis['throughput_analysis']
                f.write("    <table>\n")
                f.write("      <tr><th>指標</th><th>値</th></tr>\n")
                f.write(f"      <tr><td>最小スループット</td><td>{throughput['min_tps']:.2f} TPS</td></tr>\n")
                f.write(f"      <tr><td>最大スループット</td><td>{throughput['max_tps']:.2f} TPS</td></tr>\n")
                f.write(f"      <tr><td>平均スループット</td><td>{throughput['avg_tps']:.2f} TPS</td></tr>\n")
                f.write("    </table>\n")
                
                if 'scalability' in throughput and throughput['scalability']:
                    f.write("    <h3>スケーラビリティ分析</h3>\n")
                    f.write("    <table>\n")
                    f.write("      <tr><th>トランザクション数の範囲</th><th>効率</th><th>評価</th></tr>\n")
                    
                    for item in throughput['scalability']:
                        efficiency = item['efficiency']
                        if efficiency >= 0.9:
                            evaluation = "良好"
                        elif efficiency >= 0.7:
                            evaluation = "許容範囲"
                        else:
                            evaluation = "改善が必要"
                        
                        f.write(f"      <tr><td>{item['tx_count_from']} → {item['tx_count_to']}</td>")
                        f.write(f"<td>{efficiency:.2f}</td><td>{evaluation}</td></tr>\n")
                    
                    f.write("    </table>\n")
        
        # シャーディング分析
        if 'sharding_analysis' in analysis_results and analysis_results['sharding_analysis']:
            shard_analysis = analysis_results['sharding_analysis']
            f.write("    <h2>シャーディングの分析</h2>\n")
            
            if 'creation_analysis' in shard_analysis and shard_analysis['creation_analysis']:
                creation = shard_analysis['creation_analysis']
                f.write("    <h3>シャード作成パフォーマンス</h3>\n")
                f.write("    <table>\n")
                f.write("      <tr><th>指標</th><th>値</th></tr>\n")
                f.write(f"      <tr><td>最小スループット</td><td>{creation['min_sps']:.2f} シャード/秒</td></tr>\n")
                f.write(f"      <tr><td>最大スループット</td><td>{creation['max_sps']:.2f} シャード/秒</td></tr>\n")
                f.write(f"      <tr><td>平均スループット</td><td>{creation['avg_sps']:.2f} シャード/秒</td></tr>\n")
                f.write("    </table>\n")
            
            if 'cross_shard_analysis' in shard_analysis and shard_analysis['cross_shard_analysis']:
                cross_shard = shard_analysis['cross_shard_analysis']
                f.write("    <h3>クロスシャードトランザクションパフォーマンス</h3>\n")
                f.write("    <table>\n")
                f.write("      <tr><th>指標</th><th>値</th></tr>\n")
                f.write(f"      <tr><td>最小スループット</td><td>{cross_shard['min_tps']:.2f} TPS</td></tr>\n")
                f.write(f"      <tr><td>最大スループット</td><td>{cross_shard['max_tps']:.2f} TPS</td></tr>\n")
                f.write(f"      <tr><td>平均スループット</td><td>{cross_shard['avg_tps']:.2f} TPS</td></tr>\n")
                f.write("    </table>\n")
        
        # 推奨事項
        f.write("    <h2>推奨事項</h2>\n")
        f.write("    <ul>\n")
        
        if 'bottlenecks' in analysis_results and analysis_results['bottlenecks']:
            for bottleneck in analysis_results['bottlenecks']:
                f.write(f"      <li>{bottleneck['recommendation']}</li>\n")
        else:
            f.write("      <li>現在のパフォーマンスは良好です。定期的なモニタリングを継続してください。</li>\n")
        
        f.write("    </ul>\n")
        
        f.write("  </div>\n")
        f.write("</body>\n")
        f.write("</html>\n")
    
    return report_path

def main():
    """メイン関数"""
    args = parse_args()
    
    # 出力ディレクトリを作成
    os.makedirs(args.output_dir, exist_ok=True)
    
    # ベンチマーク結果を解析
    benchmark_results = {}
    
    if args.type in ['transaction', 'all']:
        transaction_file = find_latest_files(args.benchmark_dir, 'transaction_benchmark_*.json')
        if transaction_file:
            benchmark_results['transaction'] = parse_benchmark_results(transaction_file)
    
    if args.type in ['sharding', 'all']:
        sharding_file = find_latest_files(args.benchmark_dir, 'sharding_benchmark_*.json')
        if sharding_file:
            benchmark_results['sharding'] = parse_benchmark_results(sharding_file)
    
    if args.type in ['storage', 'all']:
        storage_file = find_latest_files(args.benchmark_dir, 'storage_benchmark_*.json')
        if storage_file:
            benchmark_results['storage'] = parse_benchmark_results(storage_file)
    
    if args.type in ['network', 'all']:
        network_file = find_latest_files(args.benchmark_dir, 'network_benchmark_*.json')
        if network_file:
            benchmark_results['network'] = parse_benchmark_results(network_file)
    
    # プロファイリング結果を解析
    cpu_profile_file = find_latest_files(args.profile_dir, 'cpu_report_*.txt')
    cpu_profile = parse_cpu_profile(cpu_profile_file) if cpu_profile_file else None
    
    memory_profile_file = find_latest_files(args.profile_dir, 'memory_profile_*.txt')
    memory_profile = parse_memory_profile(memory_profile_file) if memory_profile_file else None
    
    # 分析結果
    analysis_results = {}
    
    # トランザクション処理の分析
    if 'transaction' in benchmark_results and benchmark_results['transaction']:
        analysis_results['transaction_analysis'] = analyze_transaction_performance(benchmark_results['transaction'])
    
    # シャーディングの分析
    if 'sharding' in benchmark_results and benchmark_results['sharding']:
        analysis_results['sharding_analysis'] = analyze_sharding_performance(benchmark_results['sharding'])
    
    # ボトルネックの特定
    analysis_results['bottlenecks'] = identify_bottlenecks(analysis_results, cpu_profile, memory_profile, args.threshold)
    
    # チャートの生成
    charts = generate_charts(analysis_results, args.output_dir)
    analysis_results['charts'] = [{'title': chart['title'], 'path': os.path.basename(chart['path']), 'description': chart['description']} for chart in charts]
    
    # レポートの生成
    reports = []
    
    if args.format in ['text', 'all']:
        text_report = generate_text_report(analysis_results, args.output_dir)
        reports.append(('テキスト', text_report))
    
    if args.format in ['json', 'all']:
        json_report = generate_json_report(analysis_results, args.output_dir)
        reports.append(('JSON', json_report))
    
    if args.format in ['html', 'all']:
        html_report = generate_html_report(analysis_results, charts, args.output_dir)
        reports.append(('HTML', html_report))
    
    # 結果を表示
    print("\nShardX ボトルネック分析が完了しました。")
    print(f"検出されたボトルネック: {len(analysis_results['bottlenecks'])}件")
    
    if analysis_results['bottlenecks']:
        print("\n重要なボトルネック:")
        for i, bottleneck in enumerate(analysis_results['bottlenecks'], 1):
            if bottleneck['severity'] == 'high':
                print(f"  {i}. {bottleneck['description']}")
    
    print("\n生成されたレポート:")
    for report_type, report_path in reports:
        print(f"  - {report_type}レポート: {report_path}")
    
    print("\n推奨事項:")
    if analysis_results['bottlenecks']:
        for bottleneck in analysis_results['bottlenecks']:
            print(f"  - {bottleneck['recommendation']}")
    else:
        print("  - 現在のパフォーマンスは良好です。定期的なモニタリングを継続してください。")

if __name__ == "__main__":
    main()