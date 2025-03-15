use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};
use std::path::Path;
use rand::{Rng, thread_rng, distributions::Distribution};
use rand::distributions::{Normal, Uniform};
use rand_distr::{Exp, Poisson};
use rayon::prelude::*;
use sha2::{Sha256, Digest};
use ed25519_dalek::{Keypair, Signer, Verifier, PublicKey, SecretKey, Signature};
use serde::{Serialize, Deserialize};
use chrono::Utc;

// トランザクション構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    id: String,
    from: String,
    to: String,
    amount: u64,
    fee: u64,
    nonce: u64,
    timestamp: u64,
    data: Option<String>,
    signature: Vec<u8>,
}

// ブロック構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    hash: String,
    prev_hash: String,
    height: u64,
    timestamp: u64,
    transactions: Vec<Transaction>,
    merkle_root: String,
    validator: String,
    signature: Vec<u8>,
}

// シャード構造体
#[derive(Debug)]
struct Shard {
    id: String,
    transactions: Vec<Transaction>,
    blocks: Vec<Block>,
    accounts: HashMap<String, u64>,
    pending_txs: Vec<Transaction>,
    keypair: Keypair,
}

// ネットワーク遅延シミュレータ
struct NetworkLatencySimulator {
    base_latency_ms: f64,
    jitter_ms: f64,
    packet_loss_rate: f64,
}

// ディスクI/Oシミュレータ
struct DiskIOSimulator {
    read_latency_ms: f64,
    write_latency_ms: f64,
    sync_latency_ms: f64,
    data_dir: String,
}

// コンセンサスシミュレータ
struct ConsensusSimulator {
    validation_time_ms: f64,
    finality_time_ms: f64,
    failure_rate: f64,
}

impl Transaction {
    // 新しいトランザクションを作成
    fn new(from: &str, to: &str, amount: u64, fee: u64, nonce: u64, keypair: &Keypair) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now().timestamp() as u64;
        
        let mut tx = Self {
            id,
            from: from.to_string(),
            to: to.to_string(),
            amount,
            fee,
            nonce,
            timestamp,
            data: None,
            signature: Vec::new(),
        };
        
        // トランザクションに署名
        tx.sign(keypair);
        
        tx
    }
    
    // トランザクションに署名
    fn sign(&mut self, keypair: &Keypair) {
        let message = self.serialize_for_signing();
        self.signature = keypair.sign(message.as_bytes()).to_bytes().to_vec();
    }
    
    // 署名用のシリアライズ
    fn serialize_for_signing(&self) -> String {
        format!("{}:{}:{}:{}:{}:{}",
            self.from, self.to, self.amount, self.fee, self.nonce, self.timestamp)
    }
    
    // 署名を検証
    fn verify_signature(&self, public_key: &PublicKey) -> bool {
        let message = self.serialize_for_signing();
        
        if self.signature.len() != 64 {
            return false;
        }
        
        let signature = match Signature::from_bytes(&self.signature) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        
        match public_key.verify(message.as_bytes(), &signature) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl Block {
    // 新しいブロックを作成
    fn new(prev_hash: &str, height: u64, transactions: Vec<Transaction>, validator: &str, keypair: &Keypair) -> Self {
        let timestamp = Utc::now().timestamp() as u64;
        let merkle_root = Self::calculate_merkle_root(&transactions);
        
        let mut block = Self {
            hash: String::new(),
            prev_hash: prev_hash.to_string(),
            height,
            timestamp,
            transactions,
            merkle_root,
            validator: validator.to_string(),
            signature: Vec::new(),
        };
        
        // ブロックハッシュを計算
        block.calculate_hash();
        
        // ブロックに署名
        block.sign(keypair);
        
        block
    }
    
    // ブロックハッシュを計算
    fn calculate_hash(&mut self) {
        let data = format!("{}:{}:{}:{}:{}",
            self.prev_hash, self.height, self.timestamp, self.merkle_root, self.validator);
        
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        
        self.hash = hex::encode(result);
    }
    
    // マークルルートを計算
    fn calculate_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        }
        
        let mut hashes: Vec<String> = transactions.iter()
            .map(|tx| {
                let mut hasher = Sha256::new();
                hasher.update(tx.id.as_bytes());
                hex::encode(hasher.finalize())
            })
            .collect();
        
        while hashes.len() > 1 {
            let mut new_hashes = Vec::new();
            
            for i in (0..hashes.len()).step_by(2) {
                if i + 1 < hashes.len() {
                    let mut hasher = Sha256::new();
                    hasher.update(format!("{}{}", hashes[i], hashes[i + 1]).as_bytes());
                    new_hashes.push(hex::encode(hasher.finalize()));
                } else {
                    new_hashes.push(hashes[i].clone());
                }
            }
            
            hashes = new_hashes;
        }
        
        hashes[0].clone()
    }
    
    // ブロックに署名
    fn sign(&mut self, keypair: &Keypair) {
        let message = self.hash.clone();
        self.signature = keypair.sign(message.as_bytes()).to_bytes().to_vec();
    }
    
    // 署名を検証
    fn verify_signature(&self, public_key: &PublicKey) -> bool {
        if self.signature.len() != 64 {
            return false;
        }
        
        let signature = match Signature::from_bytes(&self.signature) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        
        match public_key.verify(self.hash.as_bytes(), &signature) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl Shard {
    // 新しいシャードを作成
    fn new(id: &str) -> Self {
        let mut csprng = rand::thread_rng();
        let keypair = Keypair::generate(&mut csprng);
        
        let genesis_block = Block {
            hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            prev_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            height: 0,
            timestamp: Utc::now().timestamp() as u64,
            transactions: Vec::new(),
            merkle_root: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            validator: id.to_string(),
            signature: Vec::new(),
        };
        
        Self {
            id: id.to_string(),
            transactions: Vec::new(),
            blocks: vec![genesis_block],
            accounts: HashMap::new(),
            pending_txs: Vec::new(),
            keypair,
        }
    }
    
    // アカウントを初期化
    fn initialize_accounts(&mut self, num_accounts: usize, initial_balance: u64) {
        for i in 0..num_accounts {
            let address = format!("account_{}", i);
            self.accounts.insert(address, initial_balance);
        }
    }
    
    // トランザクションを処理
    fn process_transaction(&mut self, tx: &Transaction, network: &NetworkLatencySimulator, disk: &DiskIOSimulator) -> bool {
        // ネットワーク遅延をシミュレート
        network.simulate_latency();
        
        // 署名検証
        let from_key_bytes = hex::decode("3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c").unwrap();
        let public_key = PublicKey::from_bytes(&from_key_bytes).unwrap();
        
        if !tx.verify_signature(&public_key) {
            return false;
        }
        
        // 残高チェック
        let sender_balance = *self.accounts.get(&tx.from).unwrap_or(&0);
        if sender_balance < tx.amount + tx.fee {
            return false;
        }
        
        // トランザクションを実行
        *self.accounts.get_mut(&tx.from).unwrap() -= tx.amount + tx.fee;
        *self.accounts.entry(tx.to.clone()).or_insert(0) += tx.amount;
        
        // ディスクI/Oをシミュレート
        disk.simulate_write();
        
        self.transactions.push(tx.clone());
        true
    }
    
    // ブロックを作成
    fn create_block(&mut self, consensus: &ConsensusSimulator, disk: &DiskIOSimulator) -> Option<Block> {
        if self.pending_txs.is_empty() {
            return None;
        }
        
        // コンセンサスをシミュレート
        if !consensus.simulate_validation() {
            return None;
        }
        
        let max_txs = 1000;
        let txs_to_include: Vec<Transaction> = self.pending_txs.drain(..std::cmp::min(max_txs, self.pending_txs.len())).collect();
        
        let prev_block = self.blocks.last().unwrap();
        let new_block = Block::new(
            &prev_block.hash,
            prev_block.height + 1,
            txs_to_include,
            &self.id,
            &self.keypair
        );
        
        // ディスクI/Oをシミュレート
        disk.simulate_write();
        
        // コンセンサスのファイナリティをシミュレート
        consensus.simulate_finality();
        
        self.blocks.push(new_block.clone());
        Some(new_block)
    }
}

impl NetworkLatencySimulator {
    // 新しいネットワーク遅延シミュレータを作成
    fn new(base_latency_ms: f64, jitter_ms: f64, packet_loss_rate: f64) -> Self {
        Self {
            base_latency_ms,
            jitter_ms,
            packet_loss_rate,
        }
    }
    
    // ネットワーク遅延をシミュレート
    fn simulate_latency(&self) {
        let mut rng = thread_rng();
        
        // パケットロスをシミュレート
        if rng.gen::<f64>() < self.packet_loss_rate {
            // パケットロスが発生した場合、より長い遅延を追加
            thread::sleep(Duration::from_millis((self.base_latency_ms * 5.0) as u64));
            return;
        }
        
        // 基本遅延 + ジッター
        let normal = Normal::new(self.base_latency_ms, self.jitter_ms).unwrap();
        let latency = normal.sample(&mut rng).max(0.0);
        
        thread::sleep(Duration::from_millis(latency as u64));
    }
}

impl DiskIOSimulator {
    // 新しいディスクI/Oシミュレータを作成
    fn new(read_latency_ms: f64, write_latency_ms: f64, sync_latency_ms: f64, data_dir: &str) -> Self {
        // データディレクトリを作成
        std::fs::create_dir_all(data_dir).unwrap_or_default();
        
        Self {
            read_latency_ms,
            write_latency_ms,
            sync_latency_ms,
            data_dir: data_dir.to_string(),
        }
    }
    
    // 読み込み操作をシミュレート
    fn simulate_read(&self) {
        let mut rng = thread_rng();
        let exp = Exp::new(1.0 / self.read_latency_ms).unwrap();
        let latency = exp.sample(&mut rng);
        
        // 実際にファイルを読み込む
        let file_path = Path::new(&self.data_dir).join("data.bin");
        if file_path.exists() {
            let mut file = File::open(file_path).unwrap_or_else(|_| {
                File::create(Path::new(&self.data_dir).join("data.bin")).unwrap()
            });
            
            let mut buffer = [0u8; 4096];
            let _ = file.read(&mut buffer);
        }
        
        thread::sleep(Duration::from_millis(latency as u64));
    }
    
    // 書き込み操作をシミュレート
    fn simulate_write(&self) {
        let mut rng = thread_rng();
        let exp = Exp::new(1.0 / self.write_latency_ms).unwrap();
        let latency = exp.sample(&mut rng);
        
        // 実際にファイルに書き込む
        let file_path = Path::new(&self.data_dir).join("data.bin");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();
        
        let data = [1u8; 4096];
        let _ = file.write(&data);
        
        thread::sleep(Duration::from_millis(latency as u64));
        
        // fsyncをシミュレート
        if rng.gen::<f64>() < 0.1 {  // 10%の確率でfsync
            self.simulate_sync();
        }
    }
    
    // fsync操作をシミュレート
    fn simulate_sync(&self) {
        let mut rng = thread_rng();
        let exp = Exp::new(1.0 / self.sync_latency_ms).unwrap();
        let latency = exp.sample(&mut rng);
        
        // 実際にfsyncを呼び出す
        let file_path = Path::new(&self.data_dir).join("data.bin");
        if file_path.exists() {
            let mut file = OpenOptions::new()
                .write(true)
                .open(file_path)
                .unwrap();
            
            let _ = file.sync_all();
        }
        
        thread::sleep(Duration::from_millis(latency as u64));
    }
}

impl ConsensusSimulator {
    // 新しいコンセンサスシミュレータを作成
    fn new(validation_time_ms: f64, finality_time_ms: f64, failure_rate: f64) -> Self {
        Self {
            validation_time_ms,
            finality_time_ms,
            failure_rate,
        }
    }
    
    // 検証プロセスをシミュレート
    fn simulate_validation(&self) -> bool {
        let mut rng = thread_rng();
        
        // 検証失敗をシミュレート
        if rng.gen::<f64>() < self.failure_rate {
            // 失敗した場合でも時間はかかる
            thread::sleep(Duration::from_millis(self.validation_time_ms as u64));
            return false;
        }
        
        // 検証時間をシミュレート
        let exp = Exp::new(1.0 / self.validation_time_ms).unwrap();
        let latency = exp.sample(&mut rng);
        
        thread::sleep(Duration::from_millis(latency as u64));
        true
    }
    
    // ファイナリティをシミュレート
    fn simulate_finality(&self) {
        let mut rng = thread_rng();
        let exp = Exp::new(1.0 / self.finality_time_ms).unwrap();
        let latency = exp.sample(&mut rng);
        
        thread::sleep(Duration::from_millis(latency as u64));
    }
}

// ベンチマーク設定
struct BenchmarkConfig {
    num_shards: usize,
    num_accounts_per_shard: usize,
    initial_balance: u64,
    num_transactions: usize,
    transaction_rate: f64,  // 1秒あたりのトランザクション数
    block_time_ms: u64,     // ブロック生成間隔（ミリ秒）
    network_latency_ms: f64,
    network_jitter_ms: f64,
    packet_loss_rate: f64,
    disk_read_latency_ms: f64,
    disk_write_latency_ms: f64,
    disk_sync_latency_ms: f64,
    consensus_validation_time_ms: f64,
    consensus_finality_time_ms: f64,
    consensus_failure_rate: f64,
}

// ベンチマーク結果
struct BenchmarkResult {
    total_time_ms: u64,
    transactions_processed: usize,
    transactions_per_second: f64,
    blocks_created: usize,
    average_block_time_ms: f64,
    average_transaction_latency_ms: f64,
    failed_transactions: usize,
    failure_rate: f64,
}

// ベンチマークを実行
fn run_benchmark(config: &BenchmarkConfig) -> BenchmarkResult {
    println!("Starting realistic benchmark with {} shards...", config.num_shards);
    println!("Network latency: {}ms ± {}ms, Packet loss: {:.2}%", 
        config.network_latency_ms, config.network_jitter_ms, config.packet_loss_rate * 100.0);
    println!("Disk I/O: Read {}ms, Write {}ms, Sync {}ms", 
        config.disk_read_latency_ms, config.disk_write_latency_ms, config.disk_sync_latency_ms);
    println!("Consensus: Validation {}ms, Finality {}ms, Failure rate: {:.2}%",
        config.consensus_validation_time_ms, config.consensus_finality_time_ms, config.consensus_failure_rate * 100.0);
    
    // シャードを初期化
    let mut shards: Vec<Shard> = (0..config.num_shards)
        .map(|i| {
            let mut shard = Shard::new(&format!("shard_{}", i));
            shard.initialize_accounts(config.num_accounts_per_shard, config.initial_balance);
            shard
        })
        .collect();
    
    // シミュレータを初期化
    let network = NetworkLatencySimulator::new(
        config.network_latency_ms,
        config.network_jitter_ms,
        config.packet_loss_rate
    );
    
    let disk = DiskIOSimulator::new(
        config.disk_read_latency_ms,
        config.disk_write_latency_ms,
        config.disk_sync_latency_ms,
        "benchmark_data"
    );
    
    let consensus = ConsensusSimulator::new(
        config.consensus_validation_time_ms,
        config.consensus_finality_time_ms,
        config.consensus_failure_rate
    );
    
    // トランザクション生成間隔（マイクロ秒）
    let tx_interval_us = (1_000_000.0 / config.transaction_rate) as u64;
    
    // ブロック生成間隔（マイクロ秒）
    let block_interval_us = config.block_time_ms * 1000;
    
    // 統計情報
    let transactions_processed = Arc::new(Mutex::new(0));
    let failed_transactions = Arc::new(Mutex::new(0));
    let blocks_created = Arc::new(Mutex::new(0));
    let transaction_latencies = Arc::new(Mutex::new(Vec::new()));
    
    // ベンチマーク開始時間
    let start_time = Instant::now();
    
    // トランザクション生成スレッド
    let tx_processed = transactions_processed.clone();
    let tx_failed = failed_transactions.clone();
    let tx_latencies = transaction_latencies.clone();
    let tx_thread = thread::spawn(move || {
        let mut rng = thread_rng();
        
        for _ in 0..config.num_transactions {
            let tx_start_time = Instant::now();
            
            // ランダムなシャードを選択
            let shard_idx = rng.gen_range(0..config.num_shards);
            let shard = &mut shards[shard_idx];
            
            // ランダムな送信元と送信先を選択
            let from_idx = rng.gen_range(0..config.num_accounts_per_shard);
            let to_idx = rng.gen_range(0..config.num_accounts_per_shard);
            
            let from = format!("account_{}", from_idx);
            let to = format!("account_{}", to_idx);
            
            // ランダムな金額を選択（残高の1%〜10%）
            let sender_balance = *shard.accounts.get(&from).unwrap_or(&0);
            let amount = rng.gen_range((sender_balance / 100).max(1)..=(sender_balance / 10).max(1));
            let fee = amount / 100;  // 手数料は金額の1%
            
            // トランザクションを作成
            let tx = Transaction::new(&from, &to, amount, fee, rng.gen(), &shard.keypair);
            
            // トランザクションを処理
            let success = shard.process_transaction(&tx, &network, &disk);
            
            if success {
                shard.pending_txs.push(tx);
                *tx_processed.lock().unwrap() += 1;
                
                // トランザクション処理時間を記録
                let latency = tx_start_time.elapsed().as_millis() as u64;
                tx_latencies.lock().unwrap().push(latency);
            } else {
                *tx_failed.lock().unwrap() += 1;
            }
            
            // トランザクション生成間隔を待機
            thread::sleep(Duration::from_micros(tx_interval_us));
        }
    });
    
    // ブロック生成スレッド
    let blk_created = blocks_created.clone();
    let block_thread = thread::spawn(move || {
        let mut last_block_time = Instant::now();
        
        loop {
            // 経過時間をチェック
            let elapsed = last_block_time.elapsed();
            if elapsed.as_micros() as u64 >= block_interval_us {
                // 各シャードでブロックを生成
                for shard in &mut shards {
                    if let Some(_) = shard.create_block(&consensus, &disk) {
                        *blk_created.lock().unwrap() += 1;
                    }
                }
                
                last_block_time = Instant::now();
            }
            
            // すべてのトランザクションが処理されたかチェック
            let processed = *transactions_processed.lock().unwrap();
            let failed = *failed_transactions.lock().unwrap();
            
            if processed + failed >= config.num_transactions {
                break;
            }
            
            thread::sleep(Duration::from_millis(10));
        }
    });
    
    // スレッドの終了を待機
    tx_thread.join().unwrap();
    block_thread.join().unwrap();
    
    // 経過時間
    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis() as u64;
    
    // 結果を集計
    let transactions_processed = *transactions_processed.lock().unwrap();
    let failed_transactions = *failed_transactions.lock().unwrap();
    let blocks_created = *blocks_created.lock().unwrap();
    let transaction_latencies = transaction_latencies.lock().unwrap().clone();
    
    let average_transaction_latency = if !transaction_latencies.is_empty() {
        transaction_latencies.iter().sum::<u64>() as f64 / transaction_latencies.len() as f64
    } else {
        0.0
    };
    
    let average_block_time = if blocks_created > 0 {
        elapsed_ms as f64 / blocks_created as f64
    } else {
        0.0
    };
    
    let tps = if elapsed_ms > 0 {
        transactions_processed as f64 * 1000.0 / elapsed_ms as f64
    } else {
        0.0
    };
    
    let failure_rate = if transactions_processed + failed_transactions > 0 {
        failed_transactions as f64 / (transactions_processed + failed_transactions) as f64
    } else {
        0.0
    };
    
    BenchmarkResult {
        total_time_ms: elapsed_ms,
        transactions_processed,
        transactions_per_second: tps,
        blocks_created,
        average_block_time_ms: average_block_time,
        average_transaction_latency_ms: average_transaction_latency,
        failed_transactions,
        failure_rate,
    }
}

fn main() {
    // ベンチマーク設定
    let config = BenchmarkConfig {
        num_shards: 10,
        num_accounts_per_shard: 1000,
        initial_balance: 1_000_000,
        num_transactions: 100_000,
        transaction_rate: 10_000.0,  // 1秒あたり10,000トランザクション
        block_time_ms: 500,          // 500ミリ秒ごとにブロックを生成
        network_latency_ms: 10.0,    // 平均10ミリ秒のネットワーク遅延
        network_jitter_ms: 5.0,      // 5ミリ秒のジッター
        packet_loss_rate: 0.01,      // 1%のパケットロス
        disk_read_latency_ms: 1.0,   // 平均1ミリ秒の読み込み遅延
        disk_write_latency_ms: 5.0,  // 平均5ミリ秒の書き込み遅延
        disk_sync_latency_ms: 20.0,  // 平均20ミリ秒のfsync遅延
        consensus_validation_time_ms: 50.0,  // 平均50ミリ秒の検証時間
        consensus_finality_time_ms: 200.0,   // 平均200ミリ秒のファイナリティ時間
        consensus_failure_rate: 0.005,       // 0.5%の検証失敗率
    };
    
    // ベンチマークを実行
    let result = run_benchmark(&config);
    
    // 結果を表示
    println!("\nBenchmark Results:");
    println!("Total time: {:.2} seconds", result.total_time_ms as f64 / 1000.0);
    println!("Transactions processed: {}", result.transactions_processed);
    println!("Transactions per second (TPS): {:.2}", result.transactions_per_second);
    println!("Blocks created: {}", result.blocks_created);
    println!("Average block time: {:.2} ms", result.average_block_time_ms);
    println!("Average transaction latency: {:.2} ms", result.average_transaction_latency_ms);
    println!("Failed transactions: {} ({:.2}%)", 
        result.failed_transactions, result.failure_rate * 100.0);
    
    // 異なる設定でのベンチマーク
    println!("\nRunning benchmark with different configurations...");
    
    // 1. 高負荷環境（高いトランザクションレート、高いネットワーク遅延）
    let high_load_config = BenchmarkConfig {
        transaction_rate: 50_000.0,  // 1秒あたり50,000トランザクション
        network_latency_ms: 50.0,    // 平均50ミリ秒のネットワーク遅延
        network_jitter_ms: 20.0,     // 20ミリ秒のジッター
        ..config
    };
    
    println!("\nHigh Load Configuration:");
    let high_load_result = run_benchmark(&high_load_config);
    println!("TPS: {:.2}", high_load_result.transactions_per_second);
    
    // 2. 最適環境（低いネットワーク遅延、高速なディスクI/O）
    let optimal_config = BenchmarkConfig {
        network_latency_ms: 1.0,     // 平均1ミリ秒のネットワーク遅延
        network_jitter_ms: 0.5,      // 0.5ミリ秒のジッター
        disk_read_latency_ms: 0.1,   // 平均0.1ミリ秒の読み込み遅延
        disk_write_latency_ms: 0.5,  // 平均0.5ミリ秒の書き込み遅延
        disk_sync_latency_ms: 2.0,   // 平均2ミリ秒のfsync遅延
        ..config
    };
    
    println!("\nOptimal Configuration:");
    let optimal_result = run_benchmark(&optimal_config);
    println!("TPS: {:.2}", optimal_result.transactions_per_second);
    
    // 3. 多シャード環境
    let multi_shard_config = BenchmarkConfig {
        num_shards: 100,  // 100シャード
        ..config
    };
    
    println!("\nMulti-Shard Configuration (100 shards):");
    let multi_shard_result = run_benchmark(&multi_shard_config);
    println!("TPS: {:.2}", multi_shard_result.transactions_per_second);
    
    // 結果の比較
    println!("\nComparison of TPS across configurations:");
    println!("Default: {:.2} TPS", result.transactions_per_second);
    println!("High Load: {:.2} TPS", high_load_result.transactions_per_second);
    println!("Optimal: {:.2} TPS", optimal_result.transactions_per_second);
    println!("Multi-Shard: {:.2} TPS", multi_shard_result.transactions_per_second);
}