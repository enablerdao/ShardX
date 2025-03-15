//! 有向非巡回グラフ（DAG）トランザクションモジュール
//! 
//! このモジュールはShardXのDAGベースのトランザクション処理を実装します。
//! 従来のブロックチェーンと比較して、より高いスループットを実現します。

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::error::Error;
use crate::transaction::Transaction;

/// DAGノード
#[derive(Debug, Clone)]
pub struct DAGNode {
    /// トランザクション
    pub transaction: Transaction,
    /// 親ノードのID
    pub parents: Vec<String>,
    /// 子ノードのID
    pub children: Vec<String>,
    /// 確認済みかどうか
    pub confirmed: bool,
    /// 確認時刻
    pub confirmation_time: Option<u64>,
    /// 重み（トポロジカルソートに使用）
    pub weight: f64,
}

/// DAG（有向非巡回グラフ）
pub struct DAG {
    /// ノードマップ
    nodes: HashMap<String, DAGNode>,
    /// ルートノード
    roots: HashSet<String>,
    /// リーフノード
    leaves: HashSet<String>,
    /// 確認済みノード
    confirmed: HashSet<String>,
    /// 未確認ノード
    unconfirmed: HashSet<String>,
    /// 最大ノード数
    max_nodes: usize,
}

impl DAG {
    /// 新しいDAGを作成
    pub fn new(max_nodes: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(max_nodes),
            roots: HashSet::new(),
            leaves: HashSet::new(),
            confirmed: HashSet::new(),
            unconfirmed: HashSet::new(),
            max_nodes,
        }
    }
    
    /// ノードを追加
    pub fn add_node(&mut self, transaction: Transaction, parents: Vec<String>) -> Result<(), Error> {
        let tx_id = transaction.id().to_string();
        
        // 既に存在するノードかチェック
        if self.nodes.contains_key(&tx_id) {
            return Err(Error::ValidationError(format!("Node already exists: {}", tx_id)));
        }
        
        // 最大ノード数をチェック
        if self.nodes.len() >= self.max_nodes {
            return Err(Error::ValidationError("DAG is full".to_string()));
        }
        
        // 親ノードの存在をチェック
        for parent_id in &parents {
            if !self.nodes.contains_key(parent_id) {
                return Err(Error::ValidationError(format!("Parent node not found: {}", parent_id)));
            }
        }
        
        // 新しいノードを作成
        let node = DAGNode {
            transaction,
            parents: parents.clone(),
            children: Vec::new(),
            confirmed: false,
            confirmation_time: None,
            weight: 0.0,
        };
        
        // 親ノードの子リストを更新
        for parent_id in &parents {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                parent.children.push(tx_id.clone());
                
                // 親がリーフノードだった場合、リーフノードから削除
                if self.leaves.contains(parent_id) {
                    self.leaves.remove(parent_id);
                }
            }
        }
        
        // ノードをDAGに追加
        self.nodes.insert(tx_id.clone(), node);
        self.unconfirmed.insert(tx_id.clone());
        
        // 親がない場合はルートノードとして追加
        if parents.is_empty() {
            self.roots.insert(tx_id.clone());
        }
        
        // リーフノードとして追加
        self.leaves.insert(tx_id);
        
        Ok(())
    }
    
    /// ノードを確認済みとしてマーク
    pub fn confirm_node(&mut self, tx_id: &str, timestamp: u64) -> Result<(), Error> {
        let node = self.nodes.get_mut(tx_id)
            .ok_or_else(|| Error::ValidationError(format!("Node not found: {}", tx_id)))?;
        
        // 既に確認済みの場合はエラー
        if node.confirmed {
            return Err(Error::ValidationError(format!("Node already confirmed: {}", tx_id)));
        }
        
        // 親ノードがすべて確認済みかチェック
        for parent_id in &node.parents {
            if let Some(parent) = self.nodes.get(parent_id) {
                if !parent.confirmed {
                    return Err(Error::ValidationError(format!(
                        "Parent node not confirmed: {}",
                        parent_id
                    )));
                }
            }
        }
        
        // ノードを確認済みとしてマーク
        node.confirmed = true;
        node.confirmation_time = Some(timestamp);
        
        // 確認済みセットに追加
        self.confirmed.insert(tx_id.to_string());
        self.unconfirmed.remove(tx_id);
        
        Ok(())
    }
    
    /// トポロジカルソートを実行
    pub fn topological_sort(&self) -> Vec<String> {
        let mut result = Vec::with_capacity(self.nodes.len());
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        
        // すべてのルートノードから深さ優先探索を開始
        for root_id in &self.roots {
            self.visit(root_id, &mut visited, &mut temp_visited, &mut result);
        }
        
        // 結果を反転（親から子の順序にする）
        result.reverse();
        
        result
    }
    
    // 深さ優先探索のヘルパー関数
    fn visit(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        // 既に訪問済みならスキップ
        if visited.contains(node_id) {
            return;
        }
        
        // 一時的に訪問済みの場合は循環があるためスキップ
        if temp_visited.contains(node_id) {
            return;
        }
        
        // 一時的に訪問済みとしてマーク
        temp_visited.insert(node_id.to_string());
        
        // 子ノードを訪問
        if let Some(node) = self.nodes.get(node_id) {
            for child_id in &node.children {
                self.visit(child_id, visited, temp_visited, result);
            }
        }
        
        // 訪問済みとしてマーク
        temp_visited.remove(node_id);
        visited.insert(node_id.to_string());
        
        // 結果に追加
        result.push(node_id.to_string());
    }
    
    /// ノードを取得
    pub fn get_node(&self, tx_id: &str) -> Option<&DAGNode> {
        self.nodes.get(tx_id)
    }
    
    /// すべてのノードを取得
    pub fn get_all_nodes(&self) -> Vec<&DAGNode> {
        self.nodes.values().collect()
    }
    
    /// 確認済みノードを取得
    pub fn get_confirmed_nodes(&self) -> Vec<&DAGNode> {
        self.nodes.iter()
            .filter(|(id, _)| self.confirmed.contains(*id))
            .map(|(_, node)| node)
            .collect()
    }
    
    /// 未確認ノードを取得
    pub fn get_unconfirmed_nodes(&self) -> Vec<&DAGNode> {
        self.nodes.iter()
            .filter(|(id, _)| self.unconfirmed.contains(*id))
            .map(|(_, node)| node)
            .collect()
    }
    
    /// ルートノードを取得
    pub fn get_roots(&self) -> Vec<&DAGNode> {
        self.roots.iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }
    
    /// リーフノードを取得
    pub fn get_leaves(&self) -> Vec<&DAGNode> {
        self.leaves.iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }
    
    /// DAGをクリア
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.roots.clear();
        self.leaves.clear();
        self.confirmed.clear();
        self.unconfirmed.clear();
    }
}
