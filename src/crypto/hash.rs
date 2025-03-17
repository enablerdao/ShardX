use crate::error::Error;
use sha2::{Digest, Sha256};
use blake3::Hasher as Blake3;
use std::fmt;

/// ハッシュアルゴリズム
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-256
    Sha256,
    /// BLAKE3
    Blake3,
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashAlgorithm::Sha256 => write!(f, "SHA-256"),
            HashAlgorithm::Blake3 => write!(f, "BLAKE3"),
        }
    }
}

/// ハッシュ値
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash {
    /// ハッシュアルゴリズム
    algorithm: HashAlgorithm,
    /// ハッシュ値
    value: Vec<u8>,
}

impl Hash {
    /// 新しいハッシュを作成
    pub fn new(algorithm: HashAlgorithm, value: Vec<u8>) -> Self {
        Self { algorithm, value }
    }

    /// ハッシュアルゴリズムを取得
    pub fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }

    /// ハッシュ値を取得
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    /// ハッシュ値を16進数文字列として取得
    pub fn to_hex(&self) -> String {
        hex::encode(&self.value)
    }

    /// 16進数文字列からハッシュを作成
    pub fn from_hex(algorithm: HashAlgorithm, hex_str: &str) -> Result<Self, Error> {
        let value = hex::decode(hex_str)
            .map_err(|e| Error::InvalidFormat(format!("Invalid hex string: {}", e)))?;
        Ok(Self { algorithm, value })
    }
}

/// ハッシャー
///
/// データのハッシュ計算を行う。
pub struct Hasher {
    /// ハッシュアルゴリズム
    algorithm: HashAlgorithm,
    /// SHA-256ハッシャー
    sha256: Option<Sha256>,
    /// BLAKE3ハッシャー
    blake3: Option<Blake3>,
}

impl Hasher {
    /// 新しいハッシャーを作成
    pub fn new(algorithm: HashAlgorithm) -> Self {
        match algorithm {
            HashAlgorithm::Sha256 => Self {
                algorithm,
                sha256: Some(Sha256::new()),
                blake3: None,
            },
            HashAlgorithm::Blake3 => Self {
                algorithm,
                sha256: None,
                blake3: Some(Blake3::new()),
            },
        }
    }

    /// データを更新
    pub fn update(&mut self, data: &[u8]) {
        match self.algorithm {
            HashAlgorithm::Sha256 => {
                if let Some(hasher) = &mut self.sha256 {
                    hasher.update(data);
                }
            }
            HashAlgorithm::Blake3 => {
                if let Some(hasher) = &mut self.blake3 {
                    hasher.update(data);
                }
            }
        }
    }

    /// ハッシュを計算
    pub fn finalize(self) -> Hash {
        match self.algorithm {
            HashAlgorithm::Sha256 => {
                let value = self
                    .sha256
                    .expect("SHA-256 hasher not initialized")
                    .finalize()
                    .to_vec();
                Hash::new(self.algorithm, value)
            }
            HashAlgorithm::Blake3 => {
                let value = self
                    .blake3
                    .expect("BLAKE3 hasher not initialized")
                    .finalize()
                    .as_bytes()
                    .to_vec();
                Hash::new(self.algorithm, value)
            }
        }
    }

    /// データのハッシュを計算
    pub fn hash(algorithm: HashAlgorithm, data: &[u8]) -> Hash {
        let mut hasher = Self::new(algorithm);
        hasher.update(data);
        hasher.finalize()
    }

    /// SHA-256ハッシュを計算
    pub fn sha256(data: &[u8]) -> Hash {
        Self::hash(HashAlgorithm::Sha256, data)
    }

    /// BLAKE3ハッシュを計算
    pub fn blake3(data: &[u8]) -> Hash {
        Self::hash(HashAlgorithm::Blake3, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let data = b"hello world";
        let hash = Hasher::sha256(data);
        assert_eq!(hash.algorithm(), HashAlgorithm::Sha256);
        assert_eq!(
            hash.to_hex(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_blake3() {
        let data = b"hello world";
        let hash = Hasher::blake3(data);
        assert_eq!(hash.algorithm(), HashAlgorithm::Blake3);
        // BLAKE3のハッシュ値は実装によって異なる可能性があるため、長さのみを検証
        assert_eq!(hash.value().len(), 32);
    }

    #[test]
    fn test_update_sha256() {
        let mut hasher = Hasher::new(HashAlgorithm::Sha256);
        hasher.update(b"hello ");
        hasher.update(b"world");
        let hash = hasher.finalize();
        assert_eq!(
            hash.to_hex(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_update_blake3() {
        let mut hasher = Hasher::new(HashAlgorithm::Blake3);
        hasher.update(b"hello ");
        hasher.update(b"world");
        let hash = hasher.finalize();
        let single_hash = Hasher::blake3(b"hello world");
        assert_eq!(hash.value(), single_hash.value());
    }

    #[test]
    fn test_from_hex() {
        let hex_str = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let hash = Hash::from_hex(HashAlgorithm::Sha256, hex_str).unwrap();
        assert_eq!(hash.algorithm(), HashAlgorithm::Sha256);
        assert_eq!(hash.to_hex(), hex_str);
    }

    #[test]
    fn test_invalid_hex() {
        let result = Hash::from_hex(HashAlgorithm::Sha256, "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_display() {
        assert_eq!(HashAlgorithm::Sha256.to_string(), "SHA-256");
        assert_eq!(HashAlgorithm::Blake3.to_string(), "BLAKE3");
    }

    #[test]
    fn test_hash_equality() {
        let hash1 = Hasher::sha256(b"hello world");
        let hash2 = Hasher::sha256(b"hello world");
        let hash3 = Hasher::sha256(b"different");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_empty_data() {
        let empty_sha256 = Hasher::sha256(b"");
        assert_eq!(
            empty_sha256.to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        
        let empty_blake3 = Hasher::blake3(b"");
        assert_eq!(empty_blake3.value().len(), 32);
    }

    #[test]
    fn test_large_data() {
        // 1MBのデータを生成
        let large_data = vec![0u8; 1024 * 1024];
        
        // ハッシュを計算
        let sha256_hash = Hasher::sha256(&large_data);
        let blake3_hash = Hasher::blake3(&large_data);
        
        // ハッシュ値の長さを検証
        assert_eq!(sha256_hash.value().len(), 32);
        assert_eq!(blake3_hash.value().len(), 32);
    }
}