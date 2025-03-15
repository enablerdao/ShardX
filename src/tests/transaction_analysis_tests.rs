use crate::transaction::{Transaction, TransactionStatus, DAG};
use crate::transaction_analysis::{TransactionAnalyzer, AnalysisPeriod};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{Duration, Utc};

// テスト用のトランザクションを作成
fn create_test_transaction(id: &str, parent_ids: Vec<String>, payload: Vec<u8>) -> Transaction {
    Transaction {
        id: id.to_string(),
        parent_ids,
        timestamp: 12345,
        payload,
        signature: vec![1, 2, 3],
        status: TransactionStatus::Confirmed,
        created_at: Utc::now(),
    }
}

// テスト用のDAGを作成
fn create_test_dag() -> Arc<DAG> {
    let dag = Arc::new(DAG::new());
    
    // テスト用のトランザクションを追加
    let tx1 = create_test_transaction("tx1", vec![], vec![10, 20, 30]);
    let tx2 = create_test_transaction("tx2", vec!["tx1".to_string()], vec![40, 50, 60]);
    let tx3 = create_test_transaction("tx3", vec!["tx1".to_string()], vec![70, 80, 90]);
    let tx4 = create_test_transaction("tx4", vec!["tx2".to_string(), "tx3".to_string()], vec![100, 110, 120]);
    let tx5 = create_test_transaction("tx5", vec!["tx4".to_string()], vec![130, 140, 150]);
    
    dag.add_transaction(tx1).unwrap();
    dag.add_transaction(tx2).unwrap();
    dag.add_transaction(tx3).unwrap();
    dag.add_transaction(tx4).unwrap();
    dag.add_transaction(tx5).unwrap();
    
    dag
}

#[test]
fn test_transaction_analyzer_basic() {
    let dag = create_test_dag();
    let analyzer = TransactionAnalyzer::new(dag.clone());
    
    // 分析を実行
    let analysis = analyzer.analyze(AnalysisPeriod::Last24Hours);
    
    // 基本的な検証
    assert_eq!(analysis.total_transactions, 5);
    assert_eq!(analysis.confirmed_transactions, 5);
    assert_eq!(analysis.rejected_transactions, 0);
    assert_eq!(analysis.pending_transactions, 0);
    
    // グラフメトリクスの検証
    assert!(analysis.graph_metrics.avg_degree > 0.0);
    assert!(analysis.graph_metrics.max_degree > 0);
    assert!(analysis.graph_metrics.clustering_coefficient >= 0.0);
    assert!(analysis.graph_metrics.longest_path > 0);
}

#[test]
fn test_transaction_analyzer_periods() {
    let dag = create_test_dag();
    let analyzer = TransactionAnalyzer::new(dag.clone());
    
    // 異なる期間での分析
    let day_analysis = analyzer.analyze(AnalysisPeriod::Last24Hours);
    let week_analysis = analyzer.analyze(AnalysisPeriod::LastWeek);
    let month_analysis = analyzer.analyze(AnalysisPeriod::LastMonth);
    
    // すべての期間で同じトランザクション数（テストデータはすべて現在時刻で作成）
    assert_eq!(day_analysis.total_transactions, 5);
    assert_eq!(week_analysis.total_transactions, 5);
    assert_eq!(month_analysis.total_transactions, 5);
    
    // カスタム期間での分析
    let custom_analysis = analyzer.analyze(AnalysisPeriod::Custom {
        start: Utc::now() - Duration::days(1),
        end: Utc::now(),
    });
    assert_eq!(custom_analysis.total_transactions, 5);
}

#[test]
fn test_transaction_pattern_detection() {
    let dag = create_test_dag();
    let analyzer = TransactionAnalyzer::new(dag.clone());
    
    // パターン検出
    let patterns = analyzer.detect_patterns(AnalysisPeriod::Last24Hours);
    
    // パターンが検出されていることを確認
    assert!(!patterns.is_empty());
    
    // 各パターンの基本情報を確認
    for pattern in &patterns {
        assert!(!pattern.id.is_empty());
        assert!(!pattern.name.is_empty());
        assert!(!pattern.description.is_empty());
        assert!(pattern.occurrences > 0);
        assert!(!pattern.related_transactions.is_empty());
    }
}

#[test]
fn test_transaction_relationships() {
    let dag = create_test_dag();
    let analyzer = TransactionAnalyzer::new(dag.clone());
    
    // tx2の関連性を分析
    let relationships = analyzer.analyze_transaction_relationships("tx2");
    assert!(relationships.is_some());
    
    let relationships = relationships.unwrap();
    
    // 親トランザクションの確認
    assert_eq!(relationships.parents.len(), 1);
    assert_eq!(relationships.parents[0].id, "tx1");
    
    // 子トランザクションの確認
    assert_eq!(relationships.children.len(), 1);
    assert_eq!(relationships.children[0].id, "tx4");
    
    // 兄弟トランザクションの確認
    assert_eq!(relationships.siblings.len(), 1);
    assert_eq!(relationships.siblings[0].id, "tx3");
    
    // tx4の関連性を分析
    let relationships = analyzer.analyze_transaction_relationships("tx4");
    assert!(relationships.is_some());
    
    let relationships = relationships.unwrap();
    
    // 親トランザクションの確認
    assert_eq!(relationships.parents.len(), 2);
    assert!(relationships.parents.iter().any(|tx| tx.id == "tx2"));
    assert!(relationships.parents.iter().any(|tx| tx.id == "tx3"));
    
    // 子トランザクションの確認
    assert_eq!(relationships.children.len(), 1);
    assert_eq!(relationships.children[0].id, "tx5");
    
    // 兄弟トランザクションの確認（tx4は兄弟を持たない）
    assert_eq!(relationships.siblings.len(), 0);
    
    // 存在しないトランザクションの関連性
    let relationships = analyzer.analyze_transaction_relationships("nonexistent");
    assert!(relationships.is_none());
}

#[test]
fn test_transaction_volume_by_hour() {
    let dag = Arc::new(DAG::new());
    
    // 異なる時間のトランザクションを追加
    let now = Utc::now();
    
    for i in 0..24 {
        let created_at = now - Duration::hours(i);
        let tx = Transaction {
            id: format!("tx_{}", i),
            parent_ids: vec![],
            timestamp: 12345,
            payload: vec![i as u8],
            signature: vec![1, 2, 3],
            status: TransactionStatus::Confirmed,
            created_at,
        };
        dag.add_transaction(tx).unwrap();
    }
    
    let analyzer = TransactionAnalyzer::new(dag.clone());
    
    // 分析を実行
    let analysis = analyzer.analyze(AnalysisPeriod::Last24Hours);
    
    // 時間帯別ボリュームの確認
    assert_eq!(analysis.volume_by_hour.len(), 24);
    
    // 各時間帯に1つのトランザクションがあることを確認
    for (_, count) in &analysis.volume_by_hour {
        assert_eq!(*count, 1);
    }
}

#[test]
fn test_transaction_types() {
    let dag = Arc::new(DAG::new());
    
    // 異なるタイプのトランザクションを追加
    let tx1 = create_test_transaction("tx1", vec![], vec![10]); // transfer
    let tx2 = create_test_transaction("tx2", vec![], vec![60]); // token_transfer
    let tx3 = create_test_transaction("tx3", vec![], vec![120]); // swap
    let tx4 = create_test_transaction("tx4", vec![], vec![180]); // liquidity
    let tx5 = create_test_transaction("tx5", vec![], vec![220]); // contract
    
    dag.add_transaction(tx1).unwrap();
    dag.add_transaction(tx2).unwrap();
    dag.add_transaction(tx3).unwrap();
    dag.add_transaction(tx4).unwrap();
    dag.add_transaction(tx5).unwrap();
    
    let analyzer = TransactionAnalyzer::new(dag.clone());
    
    // 分析を実行
    let analysis = analyzer.analyze(AnalysisPeriod::Last24Hours);
    
    // トランザクションタイプの確認
    assert_eq!(analysis.transaction_types.len(), 5);
    assert_eq!(*analysis.transaction_types.get("transfer").unwrap_or(&0), 1);
    assert_eq!(*analysis.transaction_types.get("token_transfer").unwrap_or(&0), 1);
    assert_eq!(*analysis.transaction_types.get("swap").unwrap_or(&0), 1);
    assert_eq!(*analysis.transaction_types.get("liquidity").unwrap_or(&0), 1);
    assert_eq!(*analysis.transaction_types.get("contract").unwrap_or(&0), 1);
}