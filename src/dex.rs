use crate::transaction::Transaction;
use crate::wallet::{Account, WalletManager};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 注文タイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// 買い注文
    Buy,
    /// 売り注文
    Sell,
}

/// 注文状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// 未約定
    Open,
    /// 一部約定
    PartiallyFilled,
    /// 約定済み
    Filled,
    /// キャンセル済み
    Canceled,
}

/// 注文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// 注文ID
    pub id: String,
    /// 注文者のアカウントID
    pub account_id: String,
    /// 取引ペア
    pub pair: TradingPair,
    /// 注文タイプ
    pub order_type: OrderType,
    /// 価格
    pub price: f64,
    /// 数量
    pub amount: f64,
    /// 約定済み数量
    pub filled_amount: f64,
    /// 注文状態
    pub status: OrderStatus,
    /// 作成日時
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新日時
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Order {
    /// 新しい注文を作成
    pub fn new(
        account_id: String,
        pair: TradingPair,
        order_type: OrderType,
        price: f64,
        amount: f64,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            account_id,
            pair,
            order_type,
            price,
            amount,
            filled_amount: 0.0,
            status: OrderStatus::Open,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// 注文が約定可能かどうかを確認
    pub fn can_match(&self, other: &Order) -> bool {
        // 同じ注文タイプの場合はマッチングしない
        if self.order_type == other.order_type {
            return false;
        }
        
        // 取引ペアが異なる場合はマッチングしない
        if self.pair != other.pair {
            return false;
        }
        
        // 価格条件を確認
        match self.order_type {
            OrderType::Buy => self.price >= other.price,
            OrderType::Sell => self.price <= other.price,
        }
    }
    
    /// 注文を更新
    pub fn update(&mut self, filled_amount: f64) {
        self.filled_amount += filled_amount;
        self.updated_at = chrono::Utc::now();
        
        // 注文状態を更新
        if self.filled_amount >= self.amount {
            self.status = OrderStatus::Filled;
        } else if self.filled_amount > 0.0 {
            self.status = OrderStatus::PartiallyFilled;
        }
    }
    
    /// 注文をキャンセル
    pub fn cancel(&mut self) {
        self.status = OrderStatus::Canceled;
        self.updated_at = chrono::Utc::now();
    }
}

/// 取引ペア
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradingPair {
    /// 基準通貨（例: BTC）
    pub base: String,
    /// 相手通貨（例: USD）
    pub quote: String,
}

impl TradingPair {
    /// 新しい取引ペアを作成
    pub fn new(base: String, quote: String) -> Self {
        Self { base, quote }
    }
    
    /// 取引ペアの文字列表現を取得
    pub fn to_string(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }
}

/// 取引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// 取引ID
    pub id: String,
    /// 買い注文ID
    pub buy_order_id: String,
    /// 売り注文ID
    pub sell_order_id: String,
    /// 取引ペア
    pub pair: TradingPair,
    /// 価格
    pub price: f64,
    /// 数量
    pub amount: f64,
    /// 取引日時
    pub executed_at: chrono::DateTime<chrono::Utc>,
}

impl Trade {
    /// 新しい取引を作成
    pub fn new(buy_order: &Order, sell_order: &Order, amount: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            buy_order_id: buy_order.id.clone(),
            sell_order_id: sell_order.id.clone(),
            pair: buy_order.pair.clone(),
            price: sell_order.price,
            amount,
            executed_at: chrono::Utc::now(),
        }
    }
}

/// オーダーブック
pub struct OrderBook {
    /// 取引ペア
    pair: TradingPair,
    /// 買い注文のキュー（価格の高い順）
    buy_orders: VecDeque<Order>,
    /// 売り注文のキュー（価格の低い順）
    sell_orders: VecDeque<Order>,
    /// 約定履歴
    trades: VecDeque<Trade>,
}

impl OrderBook {
    /// 新しいオーダーブックを作成
    pub fn new(pair: TradingPair) -> Self {
        Self {
            pair,
            buy_orders: VecDeque::new(),
            sell_orders: VecDeque::new(),
            trades: VecDeque::new(),
        }
    }
    
    /// 注文を追加
    pub fn add_order(&mut self, order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        
        match order.order_type {
            OrderType::Buy => {
                // 買い注文の場合、売り注文とマッチングを試みる
                let mut remaining_order = order;
                
                while remaining_order.status == OrderStatus::Open && !self.sell_orders.is_empty() {
                    let mut sell_order = self.sell_orders.pop_front().unwrap();
                    
                    if remaining_order.can_match(&sell_order) {
                        // マッチング可能な場合、取引を実行
                        let trade_amount = f64::min(
                            remaining_order.amount - remaining_order.filled_amount,
                            sell_order.amount - sell_order.filled_amount,
                        );
                        
                        // 注文を更新
                        remaining_order.update(trade_amount);
                        sell_order.update(trade_amount);
                        
                        // 取引を記録
                        let trade = Trade::new(&remaining_order, &sell_order, trade_amount);
                        trades.push(trade);
                        
                        // 売り注文がまだ未約定の場合、キューに戻す
                        if sell_order.status != OrderStatus::Filled {
                            self.sell_orders.push_front(sell_order);
                        }
                    } else {
                        // マッチングできない場合、売り注文をキューに戻す
                        self.sell_orders.push_front(sell_order);
                        break;
                    }
                }
                
                // 買い注文がまだ未約定の場合、キューに追加
                if remaining_order.status != OrderStatus::Filled {
                    // 価格の高い順にソート
                    let pos = self.buy_orders.iter().position(|o| o.price < remaining_order.price)
                        .unwrap_or(self.buy_orders.len());
                    self.buy_orders.insert(pos, remaining_order);
                }
            }
            OrderType::Sell => {
                // 売り注文の場合、買い注文とマッチングを試みる
                let mut remaining_order = order;
                
                while remaining_order.status == OrderStatus::Open && !self.buy_orders.is_empty() {
                    let mut buy_order = self.buy_orders.pop_front().unwrap();
                    
                    if buy_order.can_match(&remaining_order) {
                        // マッチング可能な場合、取引を実行
                        let trade_amount = f64::min(
                            buy_order.amount - buy_order.filled_amount,
                            remaining_order.amount - remaining_order.filled_amount,
                        );
                        
                        // 注文を更新
                        buy_order.update(trade_amount);
                        remaining_order.update(trade_amount);
                        
                        // 取引を記録
                        let trade = Trade::new(&buy_order, &remaining_order, trade_amount);
                        trades.push(trade);
                        
                        // 買い注文がまだ未約定の場合、キューに戻す
                        if buy_order.status != OrderStatus::Filled {
                            self.buy_orders.push_front(buy_order);
                        }
                    } else {
                        // マッチングできない場合、買い注文をキューに戻す
                        self.buy_orders.push_front(buy_order);
                        break;
                    }
                }
                
                // 売り注文がまだ未約定の場合、キューに追加
                if remaining_order.status != OrderStatus::Filled {
                    // 価格の低い順にソート
                    let pos = self.sell_orders.iter().position(|o| o.price > remaining_order.price)
                        .unwrap_or(self.sell_orders.len());
                    self.sell_orders.insert(pos, remaining_order);
                }
            }
        }
        
        // 取引履歴を更新
        for trade in &trades {
            self.trades.push_front(trade.clone());
        }
        
        // 取引履歴は最大100件まで保持
        while self.trades.len() > 100 {
            self.trades.pop_back();
        }
        
        trades
    }
    
    /// 注文をキャンセル
    pub fn cancel_order(&mut self, order_id: &str) -> Option<Order> {
        // 買い注文から検索
        for i in 0..self.buy_orders.len() {
            if self.buy_orders[i].id == order_id {
                let mut order = self.buy_orders.remove(i).unwrap();
                order.cancel();
                return Some(order);
            }
        }
        
        // 売り注文から検索
        for i in 0..self.sell_orders.len() {
            if self.sell_orders[i].id == order_id {
                let mut order = self.sell_orders.remove(i).unwrap();
                order.cancel();
                return Some(order);
            }
        }
        
        None
    }
    
    /// 買い注文の一覧を取得
    pub fn get_buy_orders(&self) -> Vec<Order> {
        self.buy_orders.iter().cloned().collect()
    }
    
    /// 売り注文の一覧を取得
    pub fn get_sell_orders(&self) -> Vec<Order> {
        self.sell_orders.iter().cloned().collect()
    }
    
    /// 取引履歴を取得
    pub fn get_trades(&self) -> Vec<Trade> {
        self.trades.iter().cloned().collect()
    }
}

/// DEXマネージャー
pub struct DexManager {
    /// オーダーブックのマップ
    order_books: Mutex<HashMap<String, OrderBook>>,
    /// ウォレットマネージャーの参照
    wallet_manager: Arc<WalletManager>,
}

impl DexManager {
    /// 新しいDEXマネージャーを作成
    pub fn new(wallet_manager: Arc<WalletManager>) -> Self {
        Self {
            order_books: Mutex::new(HashMap::new()),
            wallet_manager,
        }
    }
    
    /// 取引ペアを追加
    pub fn add_trading_pair(&self, base: String, quote: String) -> TradingPair {
        let pair = TradingPair::new(base, quote);
        let pair_str = pair.to_string();
        
        let mut order_books = self.order_books.lock().unwrap();
        if !order_books.contains_key(&pair_str) {
            order_books.insert(pair_str.clone(), OrderBook::new(pair.clone()));
            info!("Trading pair added: {}", pair_str);
        }
        
        pair
    }
    
    /// 注文を作成
    pub fn create_order(
        &self,
        account_id: &str,
        pair: TradingPair,
        order_type: OrderType,
        price: f64,
        amount: f64,
    ) -> Result<(Order, Vec<Trade>), String> {
        // アカウントを確認
        let account = self.wallet_manager.get_account(account_id)
            .ok_or_else(|| format!("Account {} not found", account_id))?;
        
        // 残高を確認
        match order_type {
            OrderType::Buy => {
                let required_balance = price * amount;
                if account.balance < required_balance {
                    return Err(format!("Insufficient balance: {} < {}", account.balance, required_balance));
                }
            }
            OrderType::Sell => {
                let token_balance = account.token_balances.get(&pair.base).unwrap_or(&0.0);
                if *token_balance < amount {
                    return Err(format!("Insufficient token balance: {} < {}", token_balance, amount));
                }
            }
        }
        
        // 注文を作成
        let order = Order::new(
            account_id.to_string(),
            pair.clone(),
            order_type,
            price,
            amount,
        );
        
        // オーダーブックに追加
        let pair_str = pair.to_string();
        let mut order_books = self.order_books.lock().unwrap();
        
        let order_book = order_books.entry(pair_str.clone())
            .or_insert_with(|| OrderBook::new(pair.clone()));
        
        let trades = order_book.add_order(order.clone());
        
        info!("Order created: {} {} {} at {} for {}", 
            order.id, 
            if order_type == OrderType::Buy { "BUY" } else { "SELL" },
            amount,
            price,
            pair_str
        );
        
        if !trades.is_empty() {
            info!("Trades executed: {}", trades.len());
            
            // 取引を処理
            for trade in &trades {
                self.process_trade(trade)?;
            }
        }
        
        Ok((order, trades))
    }
    
    /// 注文をキャンセル
    pub fn cancel_order(&self, account_id: &str, order_id: &str) -> Result<Order, String> {
        let mut order_books = self.order_books.lock().unwrap();
        
        // すべてのオーダーブックから検索
        for (pair_str, order_book) in order_books.iter_mut() {
            if let Some(order) = order_book.cancel_order(order_id) {
                // 注文者を確認
                if order.account_id != account_id {
                    return Err(format!("Order {} does not belong to account {}", order_id, account_id));
                }
                
                info!("Order canceled: {} for {}", order_id, pair_str);
                return Ok(order);
            }
        }
        
        Err(format!("Order {} not found", order_id))
    }
    
    /// 取引を処理
    fn process_trade(&self, trade: &Trade) -> Result<(), String> {
        // 買い手と売り手のアカウントを取得
        let buy_order = self.get_order(&trade.buy_order_id)?;
        let sell_order = self.get_order(&trade.sell_order_id)?;
        
        let buyer_id = buy_order.account_id.clone();
        let seller_id = sell_order.account_id.clone();
        
        // 残高を更新
        self.wallet_manager.process_trade(
            &buyer_id,
            &seller_id,
            &trade.pair.base,
            &trade.pair.quote,
            trade.price,
            trade.amount,
        )?;
        
        Ok(())
    }
    
    /// 注文を取得
    fn get_order(&self, order_id: &str) -> Result<Order, String> {
        let order_books = self.order_books.lock().unwrap();
        
        // すべてのオーダーブックから検索
        for order_book in order_books.values() {
            // 買い注文から検索
            for order in &order_book.buy_orders {
                if order.id == order_id {
                    return Ok(order.clone());
                }
            }
            
            // 売り注文から検索
            for order in &order_book.sell_orders {
                if order.id == order_id {
                    return Ok(order.clone());
                }
            }
        }
        
        Err(format!("Order {} not found", order_id))
    }
    
    /// オーダーブックを取得
    pub fn get_order_book(&self, pair: &TradingPair) -> Result<(Vec<Order>, Vec<Order>), String> {
        let pair_str = pair.to_string();
        let order_books = self.order_books.lock().unwrap();
        
        let order_book = order_books.get(&pair_str)
            .ok_or_else(|| format!("Trading pair {} not found", pair_str))?;
        
        Ok((order_book.get_buy_orders(), order_book.get_sell_orders()))
    }
    
    /// 取引履歴を取得
    pub fn get_trade_history(&self, pair: &TradingPair) -> Result<Vec<Trade>, String> {
        let pair_str = pair.to_string();
        let order_books = self.order_books.lock().unwrap();
        
        let order_book = order_books.get(&pair_str)
            .ok_or_else(|| format!("Trading pair {} not found", pair_str))?;
        
        Ok(order_book.get_trades())
    }
}