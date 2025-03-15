use crate::ai::trade_prediction::{PricePredictionModel, TradePredictionManager, PredictionPeriod};
use crate::dex::{Trade, TradingPair};
use std::sync::Arc;
use uuid::Uuid;

// テスト用の取引ペアを作成
fn create_test_trading_pair() -> TradingPair {
    TradingPair {
        base: "BTC".to_string(),
        quote: "USD".to_string(),
    }
}

// テスト用の取引を作成
fn create_test_trade(price: f64) -> Trade {
    Trade {
        id: Uuid::new_v4().to_string(),
        buy_order_id: "buy1".to_string(),
        sell_order_id: "sell1".to_string(),
        pair: create_test_trading_pair(),
        price,
        amount: 1.0,
        executed_at: chrono::Utc::now(),
    }
}

#[test]
fn test_price_prediction_model() {
    let pair = create_test_trading_pair();
    let mut model = PricePredictionModel::new(pair, 100);
    
    // 価格履歴を追加
    for i in 0..20 {
        let price = 10000.0 + (i as f64 * 100.0);
        model.update_price(price);
    }
    
    // 予測を実行
    let prediction = model.predict_price(PredictionPeriod::Hour).unwrap();
    
    // 予測結果の検証
    assert_eq!(prediction.pair.base, "BTC");
    assert_eq!(prediction.pair.quote, "USD");
    assert_eq!(prediction.period, PredictionPeriod::Hour);
    assert!(prediction.predicted_price > 0.0);
    assert!(prediction.lower_bound <= prediction.predicted_price);
    assert!(prediction.upper_bound >= prediction.predicted_price);
    assert!(prediction.confidence > 0.0 && prediction.confidence <= 1.0);
    
    // 異なる期間での予測
    let day_prediction = model.predict_price(PredictionPeriod::Day).unwrap();
    assert_eq!(day_prediction.period, PredictionPeriod::Day);
    
    let week_prediction = model.predict_price(PredictionPeriod::Week).unwrap();
    assert_eq!(week_prediction.period, PredictionPeriod::Week);
    
    // 最後の予測結果を取得
    let last_prediction = model.get_last_prediction().unwrap();
    assert_eq!(last_prediction.period, PredictionPeriod::Week);
}

#[test]
fn test_trade_prediction_manager() {
    let manager = TradePredictionManager::new();
    let pair = create_test_trading_pair();
    
    // モデルを追加
    manager.add_model(pair.clone(), 100).unwrap();
    
    // 取引から価格履歴を更新
    for i in 0..20 {
        let price = 10000.0 + (i as f64 * 100.0);
        let trade = create_test_trade(price);
        manager.update_from_trade(&trade);
    }
    
    // 予測を実行
    let prediction = manager.predict_price(&pair, PredictionPeriod::Hour).unwrap();
    
    // 予測結果の検証
    assert_eq!(prediction.pair.base, "BTC");
    assert_eq!(prediction.pair.quote, "USD");
    assert_eq!(prediction.period, PredictionPeriod::Hour);
    assert!(prediction.predicted_price > 0.0);
    
    // 最後の予測結果を取得
    let last_prediction = manager.get_last_prediction(&pair).unwrap();
    assert_eq!(last_prediction.predicted_price, prediction.predicted_price);
    
    // 存在しない取引ペアの予測
    let nonexistent_pair = TradingPair {
        base: "ETH".to_string(),
        quote: "USD".to_string(),
    };
    let result = manager.predict_price(&nonexistent_pair, PredictionPeriod::Hour);
    assert!(result.is_err());
}

#[test]
fn test_prediction_periods() {
    // 期間の長さを確認
    assert_eq!(PredictionPeriod::Hour.minutes(), 60);
    assert_eq!(PredictionPeriod::Day.minutes(), 60 * 24);
    assert_eq!(PredictionPeriod::Week.minutes(), 60 * 24 * 7);
    assert_eq!(PredictionPeriod::Month.minutes(), 60 * 24 * 30);
    
    assert_eq!(PredictionPeriod::Hour.hours(), 1);
    assert_eq!(PredictionPeriod::Day.hours(), 24);
    assert_eq!(PredictionPeriod::Week.hours(), 24 * 7);
    assert_eq!(PredictionPeriod::Month.hours(), 24 * 30);
    
    assert_eq!(PredictionPeriod::Hour.days(), 0);
    assert_eq!(PredictionPeriod::Day.days(), 1);
    assert_eq!(PredictionPeriod::Week.days(), 7);
    assert_eq!(PredictionPeriod::Month.days(), 30);
}

#[test]
fn test_multiple_models() {
    let manager = TradePredictionManager::new();
    
    // 複数の取引ペアを追加
    let btc_usd = TradingPair {
        base: "BTC".to_string(),
        quote: "USD".to_string(),
    };
    
    let eth_usd = TradingPair {
        base: "ETH".to_string(),
        quote: "USD".to_string(),
    };
    
    manager.add_model(btc_usd.clone(), 100).unwrap();
    manager.add_model(eth_usd.clone(), 100).unwrap();
    
    // BTC/USDの取引を追加
    for i in 0..20 {
        let price = 10000.0 + (i as f64 * 100.0);
        let trade = Trade {
            id: Uuid::new_v4().to_string(),
            buy_order_id: "buy1".to_string(),
            sell_order_id: "sell1".to_string(),
            pair: btc_usd.clone(),
            price,
            amount: 1.0,
            executed_at: chrono::Utc::now(),
        };
        manager.update_from_trade(&trade);
    }
    
    // ETH/USDの取引を追加
    for i in 0..20 {
        let price = 1000.0 + (i as f64 * 10.0);
        let trade = Trade {
            id: Uuid::new_v4().to_string(),
            buy_order_id: "buy1".to_string(),
            sell_order_id: "sell1".to_string(),
            pair: eth_usd.clone(),
            price,
            amount: 1.0,
            executed_at: chrono::Utc::now(),
        };
        manager.update_from_trade(&trade);
    }
    
    // すべての予測を取得
    let predictions = manager.get_all_predictions(PredictionPeriod::Hour);
    assert_eq!(predictions.len(), 2);
    
    // BTC/USDの予測を確認
    let btc_usd_key = format!("{}/{}", btc_usd.base, btc_usd.quote);
    assert!(predictions.contains_key(&btc_usd_key));
    let btc_prediction = predictions.get(&btc_usd_key).unwrap().as_ref().unwrap();
    assert_eq!(btc_prediction.pair.base, "BTC");
    assert_eq!(btc_prediction.pair.quote, "USD");
    
    // ETH/USDの予測を確認
    let eth_usd_key = format!("{}/{}", eth_usd.base, eth_usd.quote);
    assert!(predictions.contains_key(&eth_usd_key));
    let eth_prediction = predictions.get(&eth_usd_key).unwrap().as_ref().unwrap();
    assert_eq!(eth_prediction.pair.base, "ETH");
    assert_eq!(eth_prediction.pair.quote, "USD");
}

#[test]
fn test_prediction_accuracy_evaluation() {
    let pair = create_test_trading_pair();
    let mut model = PricePredictionModel::new(pair, 100);
    
    // 価格履歴を追加
    for i in 0..20 {
        let price = 10000.0 + (i as f64 * 100.0);
        model.update_price(price);
    }
    
    // 予測を実行
    let prediction = model.predict_price(PredictionPeriod::Hour).unwrap();
    
    // 予測の精度を評価（まだ予測対象の時間が来ていないため、Noneが返る）
    let accuracy = model.evaluate_prediction_accuracy();
    assert!(accuracy.is_none());
    
    // 予測対象の時間を過ぎた状態をシミュレート
    // 実際のテストでは時間を進めることができないため、ここではテストを省略
}