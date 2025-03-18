use log::{error, info};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::node::Node;
use crate::wallet::WalletManager;

/// コマンドラインインターフェイス
pub struct CLI {
    /// ノードの参照
    node: Arc<Mutex<Node>>,
    /// ウォレットマネージャーの参照
    wallet_manager: Arc<WalletManager>,
}

/// コマンド実行結果
#[derive(Debug, Serialize)]
struct CommandResult {
    /// 成功したかどうか
    success: bool,
    /// 結果メッセージ
    message: String,
    /// データ（オプション）
    data: Option<serde_json::Value>,
}

impl CLI {
    /// 新しいCLIを作成
    pub fn new(node: Arc<Mutex<Node>>, wallet_manager: Arc<WalletManager>) -> Self {
        Self {
            node,
            wallet_manager,
        }
    }

    /// CLIを起動
    pub async fn start(&self) {
        println!("\nShardX コマンドラインインターフェイス");
        println!("コマンド一覧を表示するには 'help' と入力してください");
        println!("終了するには 'exit' と入力してください\n");

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut handle = stdin.lock();
        let mut buffer = String::new();

        loop {
            print!("shardx> ");
            stdout.flush().unwrap();
            buffer.clear();
            
            if handle.read_line(&mut buffer).unwrap() == 0 {
                break;
            }
            
            let command = buffer.trim();
            if command.is_empty() {
                continue;
            }
            
            match command {
                "exit" | "quit" => {
                    println!("ShardX CLIを終了します");
                    break;
                }
                "help" => {
                    self.print_help();
                }
                "info" => {
                    self.show_node_info().await;
                }
                "accounts" => {
                    self.list_accounts().await;
                }
                cmd if cmd.starts_with("create-account ") => {
                    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
                    if parts.len() > 1 {
                        self.create_account(parts[1]).await;
                    } else {
                        println!("エラー: アカウント名を指定してください");
                    }
                }
                cmd if cmd.starts_with("account ") => {
                    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
                    if parts.len() > 1 {
                        self.show_account(parts[1]).await;
                    } else {
                        println!("エラー: アカウントIDを指定してください");
                    }
                }
                cmd if cmd.starts_with("transfer ") => {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let from = parts[1];
                        let to = parts[2];
                        let amount = parts[3].parse::<f64>().unwrap_or(0.0);
                        self.transfer(from, to, amount).await;
                    } else {
                        println!("エラー: 送金元、送金先、金額を指定してください");
                        println!("使用方法: transfer <送金元ID> <送金先ID> <金額>");
                    }
                }
                cmd if cmd.starts_with("status") => {
                    self.show_status().await;
                }
                _ => {
                    println!("不明なコマンドです。'help' と入力してコマンド一覧を表示してください");
                }
            }
        }
    }

    /// ヘルプを表示
    fn print_help(&self) {
        println!("\nShardX CLI コマンド一覧:");
        println!("  help                     - このヘルプを表示");
        println!("  info                     - ノード情報を表示");
        println!("  status                   - ノードのステータスを表示");
        println!("  accounts                 - アカウント一覧を表示");
        println!("  create-account <名前>    - 新しいアカウントを作成");
        println!("  account <ID>             - アカウント情報を表示");
        println!("  transfer <送金元> <送金先> <金額> - 送金を実行");
        println!("  exit, quit               - CLIを終了");
        println!("");
    }

    /// ノード情報を表示
    async fn show_node_info(&self) {
        let node = self.node.lock().await;
        println!("\nノード情報:");
        println!("  ID: {}", node.id);
        println!("  ステータス: {:?}", node.get_status());
        println!("  TPS: {:.2}", node.get_tps());
        println!("  シャード数: {}", node.get_shard_count());
        println!("  確認済みトランザクション: {}", node.dag.confirmed_count());
        println!("");
    }

    /// ノードのステータスを表示
    async fn show_status(&self) {
        let node = self.node.lock().await;
        println!("\nノードステータス:");
        println!("  ステータス: {:?}", node.get_status());
        println!("  実行中のシャード: {}", node.get_active_shards());
        println!("  メモリ使用量: {} MB", node.get_memory_usage() / (1024 * 1024));
        println!("  起動時間: {} 秒", node.get_uptime().as_secs());
        println!("");
    }

    /// アカウント一覧を表示
    async fn list_accounts(&self) {
        let accounts = self.wallet_manager.list_accounts();
        println!("\nアカウント一覧 ({}件):", accounts.len());
        for (i, account) in accounts.iter().enumerate() {
            println!("  {}. {} (ID: {})", i + 1, account.name, account.id);
        }
        println!("");
    }

    /// アカウントを作成
    async fn create_account(&self, name: &str) {
        match self.wallet_manager.create_account(name.to_string()) {
            Ok(account) => {
                println!("\nアカウントを作成しました:");
                println!("  名前: {}", account.name);
                println!("  ID: {}", account.id);
                println!("  公開鍵: {}", account.public_key);
                println!("");
            }
            Err(e) => {
                println!("\nエラー: アカウントの作成に失敗しました: {}", e);
                println!("");
            }
        }
    }

    /// アカウント情報を表示
    async fn show_account(&self, id: &str) {
        match self.wallet_manager.get_account(id) {
            Some(account) => {
                println!("\nアカウント情報:");
                println!("  名前: {}", account.name);
                println!("  ID: {}", account.id);
                println!("  公開鍵: {}", account.public_key);
                println!("  残高: {}", account.balance);
                println!("  作成日時: {}", account.created_at);
                println!("");
            }
            None => {
                println!("\nエラー: アカウントが見つかりません: {}", id);
                println!("");
            }
        }
    }

    /// 送金を実行
    async fn transfer(&self, from: &str, to: &str, amount: f64) {
        if amount <= 0.0 {
            println!("\nエラー: 送金額は0より大きい値を指定してください");
            println!("");
            return;
        }

        let from_account = match self.wallet_manager.get_account(from) {
            Some(account) => account,
            None => {
                println!("\nエラー: 送金元アカウントが見つかりません: {}", from);
                println!("");
                return;
            }
        };

        let to_account = match self.wallet_manager.get_account(to) {
            Some(account) => account,
            None => {
                println!("\nエラー: 送金先アカウントが見つかりません: {}", to);
                println!("");
                return;
            }
        };

        if from_account.balance < amount {
            println!("\nエラー: 残高不足です (残高: {}, 送金額: {})", from_account.balance, amount);
            println!("");
            return;
        }

        println!("\n送金を実行中...");
        println!("  送金元: {} ({})", from_account.name, from_account.id);
        println!("  送金先: {} ({})", to_account.name, to_account.id);
        println!("  金額: {}", amount);

        // 実際の送金処理はここに実装
        // この例では単純に残高を更新するだけ
        self.wallet_manager.update_balance(&from_account.id, -amount);
        self.wallet_manager.update_balance(&to_account.id, amount);

        println!("送金が完了しました");
        println!("");
    }
}