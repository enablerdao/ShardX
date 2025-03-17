use chrono::{Duration, Utc};
use shardx::ai::{
    EnhancedTransactionPredictor, ModelConfig, PredictionHorizon, PredictionModelType,
    PredictionTarget,
};
use shardx::transaction::{Transaction, TransactionStatus};
use std::collections::HashMap;

fn main() {
    // テストトランザクションを作成
    let transactions = create_test_transactions();

    println!("トランザクション数: {}", transactions.len());

    // モデル設定を作成
    let model_config = ModelConfig {
        model_type: PredictionModelType::Ensemble,
        hyperparameters: HashMap::new(),
        features: vec![
            "amount".to_string(),
            "fee".to_string(),
            "hour_of_day".to_string(),
            "day_of_week".to_string(),
            "is_weekend".to_string(),
        ],
        training_period_days: 30,
        prediction_horizon: PredictionHorizon::MediumTerm,
        confidence_level: 0.95,
    };

    // 予測器を作成
    let mut predictor = EnhancedTransactionPredictor::new(model_config);

    // モデルを学習
    println!("モデルを学習中...");
    match predictor.train(&transactions) {
        Ok(_) => println!("モデルの学習が完了しました"),
        Err(e) => {
            println!("モデルの学習に失敗しました: {:?}", e);
            return;
        }
    }

    // 特徴量重要度を取得
    println!("\n特徴量重要度:");
    match predictor.get_feature_importance() {
        Ok(features) => {
            for feature in features {
                println!(
                    "  {}: {:.4}",
                    feature.name,
                    feature.importance.unwrap_or(0.0)
                );
            }
        }
        Err(e) => println!("特徴量重要度の取得に失敗しました: {:?}", e),
    }

    // トランザクション数の予測
    println!("\nトランザクション数の予測:");
    match predictor.predict(PredictionTarget::TransactionCount) {
        Ok(result) => {
            println!(
                "  予測期間: {} から {}",
                result.start_time.format("%Y-%m-%d %H:%M"),
                result.end_time.format("%Y-%m-%d %H:%M")
            );

            println!("  予測データポイント:");
            for (i, point) in result.predictions.iter().enumerate() {
                if i < 5 || i >= result.predictions.len() - 5 {
                    println!(
                        "    {}: {:.2}",
                        point.timestamp.format("%Y-%m-%d %H:%M"),
                        point.value
                    );
                } else if i == 5 {
                    println!("    ...");
                }
            }

            if let Some(lower) = &result.confidence_lower {
                println!("  信頼区間下限 (最初と最後の5件):");
                for (i, point) in lower.iter().enumerate() {
                    if i < 5 || i >= lower.len() - 5 {
                        println!(
                            "    {}: {:.2}",
                            point.timestamp.format("%Y-%m-%d %H:%M"),
                            point.value
                        );
                    } else if i == 5 {
                        println!("    ...");
                    }
                }
            }

            if let Some(upper) = &result.confidence_upper {
                println!("  信頼区間上限 (最初と最後の5件):");
                for (i, point) in upper.iter().enumerate() {
                    if i < 5 || i >= upper.len() - 5 {
                        println!(
                            "    {}: {:.2}",
                            point.timestamp.format("%Y-%m-%d %H:%M"),
                            point.value
                        );
                    } else if i == 5 {
                        println!("    ...");
                    }
                }
            }

            println!("  予測精度: {:.2}%", result.accuracy.unwrap_or(0.0) * 100.0);
            println!("  RMSE: {:.4}", result.error_rmse.unwrap_or(0.0));
            println!("  MAE: {:.4}", result.error_mae.unwrap_or(0.0));
        }
        Err(e) => println!("予測に失敗しました: {:?}", e),
    }

    // 取引量の予測
    println!("\n取引量の予測:");
    match predictor.predict(PredictionTarget::TransactionVolume) {
        Ok(result) => {
            println!(
                "  予測期間: {} から {}",
                result.start_time.format("%Y-%m-%d %H:%M"),
                result.end_time.format("%Y-%m-%d %H:%M")
            );

            println!("  予測データポイント (最初と最後の5件):");
            for (i, point) in result.predictions.iter().enumerate() {
                if i < 5 || i >= result.predictions.len() - 5 {
                    println!(
                        "    {}: {:.2}",
                        point.timestamp.format("%Y-%m-%d %H:%M"),
                        point.value
                    );
                } else if i == 5 {
                    println!("    ...");
                }
            }

            println!("  予測精度: {:.2}%", result.accuracy.unwrap_or(0.0) * 100.0);
        }
        Err(e) => println!("予測に失敗しました: {:?}", e),
    }

    // 手数料の予測
    println!("\n手数料の予測:");
    match predictor.predict(PredictionTarget::TransactionFee) {
        Ok(result) => {
            println!(
                "  予測期間: {} から {}",
                result.start_time.format("%Y-%m-%d %H:%M"),
                result.end_time.format("%Y-%m-%d %H:%M")
            );

            println!("  予測データポイント (最初と最後の5件):");
            for (i, point) in result.predictions.iter().enumerate() {
                if i < 5 || i >= result.predictions.len() - 5 {
                    println!(
                        "    {}: {:.2}",
                        point.timestamp.format("%Y-%m-%d %H:%M"),
                        point.value
                    );
                } else if i == 5 {
                    println!("    ...");
                }
            }

            println!("  予測精度: {:.2}%", result.accuracy.unwrap_or(0.0) * 100.0);
        }
        Err(e) => println!("予測に失敗しました: {:?}", e),
    }

    // ネットワーク負荷の予測
    println!("\nネットワーク負荷の予測:");
    match predictor.predict(PredictionTarget::NetworkLoad) {
        Ok(result) => {
            println!(
                "  予測期間: {} から {}",
                result.start_time.format("%Y-%m-%d %H:%M"),
                result.end_time.format("%Y-%m-%d %H:%M")
            );

            println!("  予測データポイント (最初と最後の5件):");
            for (i, point) in result.predictions.iter().enumerate() {
                if i < 5 || i >= result.predictions.len() - 5 {
                    println!(
                        "    {}: {:.2}%",
                        point.timestamp.format("%Y-%m-%d %H:%M"),
                        point.value
                    );
                } else if i == 5 {
                    println!("    ...");
                }
            }

            println!("  予測精度: {:.2}%", result.accuracy.unwrap_or(0.0) * 100.0);
        }
        Err(e) => println!("予測に失敗しました: {:?}", e),
    }
}

fn create_test_transactions() -> Vec<Transaction> {
    let now = Utc::now();
    let mut transactions = Vec::new();

    // 過去30日間のトランザクションを生成
    for day in 0..30 {
        // 1日あたり10〜50件のトランザクションを生成
        let tx_count = 10 + (day % 5) * 10;

        for i in 0..tx_count {
            let timestamp = now - Duration::days(30 - day) + Duration::hours(i as i64 % 24);

            // 時間帯による変動
            let hour_factor = 1.0 + 0.5 * ((timestamp.hour() as f64 - 12.0).abs() / 12.0);

            // 曜日による変動
            let weekday = timestamp.weekday().num_days_from_monday();
            let day_factor = if weekday >= 5 { 0.7 } else { 1.3 };

            // 基本取引量
            let base_amount = 100.0 * hour_factor * day_factor;

            // トレンド（日が経つにつれて増加）
            let trend_factor = 1.0 + day as f64 / 30.0;

            // 最終的な取引量
            let amount = (base_amount * trend_factor) as u64;

            // 手数料は取引量の約1%
            let fee = (amount as f64 * 0.01) as u64;

            let transaction = Transaction {
                id: format!("tx-{}-{}", day, i),
                sender: format!("sender-{}", i % 100),
                receiver: format!("receiver-{}", (i + 1) % 100),
                amount,
                fee,
                timestamp: timestamp.timestamp(),
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            };

            transactions.push(transaction);
        }
    }

    transactions
}
