//! # モジュール名
//!
//! このモジュールの概要説明。
//! 複数行にわたって説明を書くことができます。
//!
//! ## 主な機能
//!
//! - 機能1の説明
//! - 機能2の説明
//! - 機能3の説明
//!
//! ## 使用例
//!
//! ```rust
//! use shardx::module_name::{Struct1, Struct2};
//!
//! let instance = Struct1::new();
//! let result = instance.do_something();
//! ```

/// 構造体の説明
///
/// より詳細な説明を書くことができます。
/// 複数行にわたって説明を書くことができます。
pub struct ExampleStruct {
    /// フィールド1の説明
    pub field1: String,
    /// フィールド2の説明
    pub field2: u32,
}

impl ExampleStruct {
    /// 新しいインスタンスを作成します。
    ///
    /// # 引数
    ///
    /// * `field1` - フィールド1の値
    /// * `field2` - フィールド2の値
    ///
    /// # 戻り値
    ///
    /// 新しく作成された`ExampleStruct`のインスタンス
    ///
    /// # 例
    ///
    /// ```
    /// use shardx::module_name::ExampleStruct;
    ///
    /// let instance = ExampleStruct::new("example".to_string(), 42);
    /// assert_eq!(instance.field1, "example");
    /// assert_eq!(instance.field2, 42);
    /// ```
    pub fn new(field1: String, field2: u32) -> Self {
        Self { field1, field2 }
    }

    /// 何か処理を行います。
    ///
    /// # 引数
    ///
    /// * `input` - 入力値
    ///
    /// # 戻り値
    ///
    /// 処理結果、または処理中に発生したエラー
    ///
    /// # エラー
    ///
    /// 以下の場合にエラーを返します：
    /// - 入力値が無効な場合
    /// - 内部処理でエラーが発生した場合
    ///
    /// # 例
    ///
    /// ```
    /// use shardx::module_name::ExampleStruct;
    ///
    /// let instance = ExampleStruct::new("example".to_string(), 42);
    /// let result = instance.do_something("input");
    /// ```
    pub fn do_something(&self, input: &str) -> Result<String, Error> {
        // 実装
        Ok(format!("Processed: {}", input))
    }
}

/// エラー型
///
/// このモジュールで発生する可能性のあるエラーを表します。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 無効な入力
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// 内部エラー
    #[error("Internal error: {0}")]
    Internal(String),

    /// I/Oエラー
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// 列挙型の例
///
/// 状態や種類を表す列挙型の例です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExampleEnum {
    /// 値なしのバリアント
    Variant1,
    /// 単一の値を持つバリアント
    Variant2(u32),
    /// 複数の値を持つバリアント
    Variant3 {
        /// フィールド1の説明
        field1: String,
        /// フィールド2の説明
        field2: u32,
    },
}

/// トレイトの例
///
/// このトレイトは、特定の操作を定義します。
/// このトレイトを実装する型は、これらの操作をサポートする必要があります。
pub trait ExampleTrait {
    /// 関連型の例
    type Output;

    /// メソッドの例
    ///
    /// # 引数
    ///
    /// * `input` - 入力値
    ///
    /// # 戻り値
    ///
    /// 処理結果
    fn process(&self, input: &str) -> Self::Output;

    /// デフォルト実装を持つメソッドの例
    ///
    /// # 戻り値
    ///
    /// 常に`true`を返します
    fn has_default_implementation(&self) -> bool {
        true
    }
}

/// 定数の例
///
/// このモジュールで使用される定数です。
pub const EXAMPLE_CONSTANT: u32 = 42;

/// 型エイリアスの例
///
/// 複雑な型に別名を付けることで、コードの可読性を向上させます。
pub type ExampleResult<T> = Result<T, Error>;

/// 関数の例
///
/// # 引数
///
/// * `input` - 入力値
///
/// # 戻り値
///
/// 処理結果
///
/// # 例
///
/// ```
/// use shardx::module_name::example_function;
///
/// let result = example_function("example");
/// assert_eq!(result, "Processed: example");
/// ```
pub fn example_function(input: &str) -> String {
    format!("Processed: {}", input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_struct_new() {
        let instance = ExampleStruct::new("example".to_string(), 42);
        assert_eq!(instance.field1, "example");
        assert_eq!(instance.field2, 42);
    }

    #[test]
    fn test_example_function() {
        let result = example_function("test");
        assert_eq!(result, "Processed: test");
    }
}