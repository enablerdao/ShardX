use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// コントラクトABI
///
/// スマートコントラクトのインターフェース定義。
/// - 関数
/// - イベント
/// - 型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractABI {
    /// コントラクト名
    pub name: String,
    /// コントラクトバージョン
    pub version: String,
    /// 関数一覧
    pub functions: Vec<ABIFunction>,
    /// イベント一覧
    pub events: Vec<ABIEvent>,
    /// 型定義
    pub types: HashMap<String, ABIType>,
}

/// ABI関数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABIFunction {
    /// 関数名
    pub name: String,
    /// 関数の説明
    pub description: Option<String>,
    /// 入力パラメータ
    pub inputs: Vec<ABIParameter>,
    /// 出力パラメータ
    pub outputs: Vec<ABIParameter>,
    /// 定数関数フラグ
    pub constant: bool,
    /// 支払い可能フラグ
    pub payable: bool,
    /// ステート変更フラグ
    pub state_mutability: String,
}

/// ABIイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABIEvent {
    /// イベント名
    pub name: String,
    /// イベントの説明
    pub description: Option<String>,
    /// パラメータ
    pub parameters: Vec<ABIParameter>,
    /// 匿名フラグ
    pub anonymous: bool,
}

/// ABIパラメータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABIParameter {
    /// パラメータ名
    pub name: String,
    /// パラメータの説明
    pub description: Option<String>,
    /// パラメータ型
    pub type_name: String,
    /// インデックス付きフラグ
    pub indexed: Option<bool>,
}

/// ABI型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ABIType {
    /// 基本型
    Basic(String),
    /// 配列型
    Array {
        /// 要素型
        element_type: Box<ABIType>,
        /// 配列長（固定長の場合）
        length: Option<usize>,
    },
    /// タプル型
    Tuple(Vec<ABIType>),
    /// マップ型
    Map {
        /// キー型
        key_type: Box<ABIType>,
        /// 値型
        value_type: Box<ABIType>,
    },
    /// 構造体型
    Struct {
        /// フィールド
        fields: Vec<ABIParameter>,
    },
    /// 列挙型
    Enum {
        /// バリアント
        variants: Vec<String>,
    },
}

impl ContractABI {
    /// 新しいContractABIを作成
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            functions: Vec::new(),
            events: Vec::new(),
            types: HashMap::new(),
        }
    }

    /// 関数を追加
    pub fn add_function(&mut self, function: ABIFunction) {
        self.functions.push(function);
    }

    /// イベントを追加
    pub fn add_event(&mut self, event: ABIEvent) {
        self.events.push(event);
    }

    /// 型を追加
    pub fn add_type(&mut self, name: &str, type_def: ABIType) {
        self.types.insert(name.to_string(), type_def);
    }

    /// 関数を名前で検索
    pub fn find_function(&self, name: &str) -> Option<&ABIFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// イベントを名前で検索
    pub fn find_event(&self, name: &str) -> Option<&ABIEvent> {
        self.events.iter().find(|e| e.name == name)
    }

    /// 型を名前で検索
    pub fn find_type(&self, name: &str) -> Option<&ABIType> {
        self.types.get(name)
    }

    /// JSONに変換
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// JSONから変換
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_abi() {
        let mut abi = ContractABI::new("TestContract", "1.0.0");

        // 関数を追加
        abi.add_function(ABIFunction {
            name: "transfer".to_string(),
            description: Some("Transfer tokens".to_string()),
            inputs: vec![
                ABIParameter {
                    name: "to".to_string(),
                    description: Some("Recipient address".to_string()),
                    type_name: "address".to_string(),
                    indexed: None,
                },
                ABIParameter {
                    name: "amount".to_string(),
                    description: Some("Amount to transfer".to_string()),
                    type_name: "uint256".to_string(),
                    indexed: None,
                },
            ],
            outputs: vec![ABIParameter {
                name: "success".to_string(),
                description: Some("Transfer success".to_string()),
                type_name: "bool".to_string(),
                indexed: None,
            }],
            constant: false,
            payable: false,
            state_mutability: "nonpayable".to_string(),
        });

        // イベントを追加
        abi.add_event(ABIEvent {
            name: "Transfer".to_string(),
            description: Some("Transfer event".to_string()),
            parameters: vec![
                ABIParameter {
                    name: "from".to_string(),
                    description: Some("Sender address".to_string()),
                    type_name: "address".to_string(),
                    indexed: Some(true),
                },
                ABIParameter {
                    name: "to".to_string(),
                    description: Some("Recipient address".to_string()),
                    type_name: "address".to_string(),
                    indexed: Some(true),
                },
                ABIParameter {
                    name: "amount".to_string(),
                    description: Some("Amount transferred".to_string()),
                    type_name: "uint256".to_string(),
                    indexed: Some(false),
                },
            ],
            anonymous: false,
        });

        // 型を追加
        abi.add_type(
            "TokenInfo",
            ABIType::Struct {
                fields: vec![
                    ABIParameter {
                        name: "name".to_string(),
                        description: Some("Token name".to_string()),
                        type_name: "string".to_string(),
                        indexed: None,
                    },
                    ABIParameter {
                        name: "symbol".to_string(),
                        description: Some("Token symbol".to_string()),
                        type_name: "string".to_string(),
                        indexed: None,
                    },
                    ABIParameter {
                        name: "decimals".to_string(),
                        description: Some("Token decimals".to_string()),
                        type_name: "uint8".to_string(),
                        indexed: None,
                    },
                ],
            },
        );

        // 検証
        assert_eq!(abi.functions.len(), 1);
        assert_eq!(abi.events.len(), 1);
        assert_eq!(abi.types.len(), 1);

        let function = abi.find_function("transfer").unwrap();
        assert_eq!(function.name, "transfer");
        assert_eq!(function.inputs.len(), 2);
        assert_eq!(function.outputs.len(), 1);

        let event = abi.find_event("Transfer").unwrap();
        assert_eq!(event.name, "Transfer");
        assert_eq!(event.parameters.len(), 3);

        let type_def = abi.find_type("TokenInfo").unwrap();
        if let ABIType::Struct { fields } = type_def {
            assert_eq!(fields.len(), 3);
        } else {
            panic!("Expected struct type");
        }

        // JSON変換
        let json = abi.to_json().unwrap();
        let abi2 = ContractABI::from_json(&json).unwrap();
        assert_eq!(abi2.name, "TestContract");
        assert_eq!(abi2.functions.len(), 1);
        assert_eq!(abi2.events.len(), 1);
        assert_eq!(abi2.types.len(), 1);
    }
}
