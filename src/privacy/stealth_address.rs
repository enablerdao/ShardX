use crate::error::Error;
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_TABLE,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};
use rand::thread_rng;
use sha2::{Digest, Sha256};
use std::fmt;

/// ステルスアドレス
#[derive(Clone, Debug)]
pub struct StealthAddress {
    /// 公開鍵
    pub public_key: CompressedRistretto,
    /// ワンタイム公開鍵
    pub one_time_pubkey: CompressedRistretto,
    /// 共有シークレットのハッシュ
    pub shared_secret_hash: [u8; 32],
}

/// ステルスキーペア
#[derive(Clone)]
pub struct StealthKeyPair {
    /// 秘密鍵
    pub private_key: Scalar,
    /// 公開鍵
    pub public_key: RistrettoPoint,
    /// ワンタイム秘密鍵
    pub one_time_privkey: Scalar,
    /// ワンタイム公開鍵
    pub one_time_pubkey: RistrettoPoint,
}

/// ステルスアドレス生成器
pub struct StealthAddressGenerator {
    /// ベースポイント
    base_point: &'static RISTRETTO_BASEPOINT_TABLE,
}

impl StealthAddress {
    /// 新しいStealthAddressを作成
    pub fn new(
        public_key: CompressedRistretto,
        one_time_pubkey: CompressedRistretto,
        shared_secret_hash: [u8; 32],
    ) -> Self {
        Self {
            public_key,
            one_time_pubkey,
            shared_secret_hash,
        }
    }
    
    /// アドレスをシリアライズ
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(96);
        bytes.extend_from_slice(self.public_key.as_bytes());
        bytes.extend_from_slice(self.one_time_pubkey.as_bytes());
        bytes.extend_from_slice(&self.shared_secret_hash);
        bytes
    }
    
    /// アドレスをデシリアライズ
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 96 {
            return Err(Error::DeserializationError("Invalid stealth address length".to_string()));
        }
        
        let public_key = CompressedRistretto::from_slice(&bytes[0..32])
            .map_err(|_| Error::DeserializationError("Invalid public key format".to_string()))?;
        
        let one_time_pubkey = CompressedRistretto::from_slice(&bytes[32..64])
            .map_err(|_| Error::DeserializationError("Invalid one-time public key format".to_string()))?;
        
        let mut shared_secret_hash = [0u8; 32];
        shared_secret_hash.copy_from_slice(&bytes[64..96]);
        
        Ok(Self {
            public_key,
            one_time_pubkey,
            shared_secret_hash,
        })
    }
}

impl fmt::Display for StealthAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sx{}", hex::encode(self.to_bytes()))
    }
}

impl StealthKeyPair {
    /// 新しいStealthKeyPairを作成
    pub fn new(
        private_key: Scalar,
        public_key: RistrettoPoint,
        one_time_privkey: Scalar,
        one_time_pubkey: RistrettoPoint,
    ) -> Self {
        Self {
            private_key,
            public_key,
            one_time_privkey,
            one_time_pubkey,
        }
    }
    
    /// キーペアを生成
    pub fn generate() -> Self {
        let mut rng = thread_rng();
        
        // 秘密鍵を生成
        let private_key = Scalar::random(&mut rng);
        
        // 公開鍵を計算
        let public_key = &private_key * &RISTRETTO_BASEPOINT_TABLE;
        
        // ワンタイム秘密鍵を生成
        let one_time_privkey = Scalar::random(&mut rng);
        
        // ワンタイム公開鍵を計算
        let one_time_pubkey = &one_time_privkey * &RISTRETTO_BASEPOINT_TABLE;
        
        Self {
            private_key,
            public_key,
            one_time_privkey,
            one_time_pubkey,
        }
    }
    
    /// 署名を生成
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        // メッセージをハッシュ化
        let mut hasher = Sha256::new();
        hasher.update(message);
        hasher.update(self.one_time_pubkey.compress().as_bytes());
        let message_hash = hasher.finalize();
        
        // ランダムな値を生成
        let mut rng = thread_rng();
        let k = Scalar::random(&mut rng);
        
        // R = k*G
        let r_point = &k * &RISTRETTO_BASEPOINT_TABLE;
        let r = r_point.compress().to_bytes();
        
        // e = H(R || P || m)
        let mut e_hasher = Sha256::new();
        e_hasher.update(&r);
        e_hasher.update(self.public_key.compress().as_bytes());
        e_hasher.update(&message_hash);
        let e_hash = e_hasher.finalize();
        let e = Scalar::from_bytes_mod_order_wide(&<[u8; 64]>::try_from(&e_hash[..]).unwrap_or([0u8; 64]));
        
        // s = k - e*x
        let s = k - e * self.private_key;
        
        // 署名 = (R, s)
        let mut signature = Vec::with_capacity(64);
        signature.extend_from_slice(&r);
        signature.extend_from_slice(&s.to_bytes());
        
        signature
    }
    
    /// 署名を検証
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        if signature.len() != 64 {
            return false;
        }
        
        // 署名を分解
        let r_bytes = &signature[0..32];
        let s_bytes = &signature[32..64];
        
        let r_point = match CompressedRistretto::from_slice(r_bytes) {
            Ok(point) => point,
            Err(_) => return false,
        };
        
        let s = match Scalar::from_canonical_bytes(s_bytes.try_into().unwrap_or([0u8; 32])) {
            Some(scalar) => scalar,
            None => return false,
        };
        
        // メッセージをハッシュ化
        let mut hasher = Sha256::new();
        hasher.update(message);
        hasher.update(self.one_time_pubkey.compress().as_bytes());
        let message_hash = hasher.finalize();
        
        // e = H(R || P || m)
        let mut e_hasher = Sha256::new();
        e_hasher.update(r_bytes);
        e_hasher.update(self.public_key.compress().as_bytes());
        e_hasher.update(&message_hash);
        let e_hash = e_hasher.finalize();
        let e = Scalar::from_bytes_mod_order_wide(&<[u8; 64]>::try_from(&e_hash[..]).unwrap_or([0u8; 64]));
        
        // R' = s*G + e*P
        let r_prime = &s * &RISTRETTO_BASEPOINT_TABLE + &e * &self.public_key;
        
        // R' == R
        r_prime.compress() == r_point
    }
}

impl StealthAddressGenerator {
    /// 新しいStealthAddressGeneratorを作成
    pub fn new() -> Self {
        Self {
            base_point: &RISTRETTO_BASEPOINT_TABLE,
        }
    }
    
    /// ステルスアドレスを生成
    pub fn generate_address(&self, public_key_bytes: &[u8]) -> Result<StealthAddress, Error> {
        // 公開鍵をデシリアライズ
        let public_key = CompressedRistretto::from_slice(public_key_bytes)
            .map_err(|_| Error::DeserializationError("Invalid public key format".to_string()))?;
        
        // 公開鍵をポイントに変換
        let public_key_point = public_key.decompress()
            .ok_or_else(|| Error::DeserializationError("Failed to decompress public key".to_string()))?;
        
        // ランダムな秘密鍵を生成
        let mut rng = thread_rng();
        let random_scalar = Scalar::random(&mut rng);
        
        // ワンタイム公開鍵を計算
        let one_time_pubkey_point = &random_scalar * self.base_point;
        let one_time_pubkey = one_time_pubkey_point.compress();
        
        // 共有シークレットを計算
        let shared_secret = &random_scalar * &public_key_point;
        
        // 共有シークレットをハッシュ化
        let mut hasher = Sha256::new();
        hasher.update(shared_secret.compress().as_bytes());
        let shared_secret_hash = hasher.finalize().into();
        
        Ok(StealthAddress {
            public_key,
            one_time_pubkey,
            shared_secret_hash,
        })
    }
    
    /// ステルスアドレスからキーペアを復元
    pub fn recover_key_pair(
        &self,
        stealth_address: &StealthAddress,
        private_key_bytes: &[u8],
    ) -> Result<StealthKeyPair, Error> {
        // 秘密鍵をデシリアライズ
        let private_key = Scalar::from_canonical_bytes(private_key_bytes.try_into().unwrap_or([0u8; 32]))
            .ok_or_else(|| Error::DeserializationError("Invalid private key format".to_string()))?;
        
        // 公開鍵を計算
        let public_key = &private_key * self.base_point;
        
        // ワンタイム公開鍵をポイントに変換
        let one_time_pubkey_point = stealth_address.one_time_pubkey.decompress()
            .ok_or_else(|| Error::DeserializationError("Failed to decompress one-time public key".to_string()))?;
        
        // 共有シークレットを計算
        let shared_secret = &private_key * &one_time_pubkey_point;
        
        // 共有シークレットをハッシュ化
        let mut hasher = Sha256::new();
        hasher.update(shared_secret.compress().as_bytes());
        let calculated_hash: [u8; 32] = hasher.finalize().into();
        
        // ハッシュが一致するか確認
        if calculated_hash != stealth_address.shared_secret_hash {
            return Err(Error::AuthenticationError("Shared secret hash mismatch".to_string()));
        }
        
        // ワンタイム秘密鍵を導出
        let mut one_time_privkey_hasher = Sha256::new();
        one_time_privkey_hasher.update(private_key_bytes);
        one_time_privkey_hasher.update(&calculated_hash);
        let one_time_privkey_bytes = one_time_privkey_hasher.finalize();
        
        let one_time_privkey = Scalar::from_bytes_mod_order_wide(
            &<[u8; 64]>::try_from(&one_time_privkey_bytes[..])
                .unwrap_or([0u8; 64])
        );
        
        // ワンタイム公開鍵を計算
        let calculated_one_time_pubkey = &one_time_privkey * self.base_point;
        
        // ワンタイム公開鍵が一致するか確認
        if calculated_one_time_pubkey.compress() != stealth_address.one_time_pubkey {
            return Err(Error::AuthenticationError("One-time public key mismatch".to_string()));
        }
        
        Ok(StealthKeyPair {
            private_key,
            public_key,
            one_time_privkey,
            one_time_pubkey: calculated_one_time_pubkey,
        })
    }
}

impl Default for StealthAddressGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stealth_address_generation_and_recovery() {
        // キーペアを生成
        let key_pair = StealthKeyPair::generate();
        let public_key_bytes = key_pair.public_key.compress().as_bytes().to_vec();
        let private_key_bytes = key_pair.private_key.to_bytes().to_vec();
        
        // ステルスアドレス生成器を作成
        let generator = StealthAddressGenerator::new();
        
        // ステルスアドレスを生成
        let stealth_address = generator.generate_address(&public_key_bytes).unwrap();
        
        // ステルスアドレスをシリアライズ
        let serialized = stealth_address.to_bytes();
        
        // ステルスアドレスをデシリアライズ
        let deserialized = StealthAddress::from_bytes(&serialized).unwrap();
        
        // 元のアドレスと一致することを確認
        assert_eq!(
            stealth_address.public_key.as_bytes(),
            deserialized.public_key.as_bytes()
        );
        assert_eq!(
            stealth_address.one_time_pubkey.as_bytes(),
            deserialized.one_time_pubkey.as_bytes()
        );
        assert_eq!(
            stealth_address.shared_secret_hash,
            deserialized.shared_secret_hash
        );
        
        // キーペアを復元
        let recovered_key_pair = generator.recover_key_pair(&stealth_address, &private_key_bytes).unwrap();
        
        // 復元されたキーペアが正しいことを確認
        assert_eq!(
            key_pair.public_key.compress().as_bytes(),
            recovered_key_pair.public_key.compress().as_bytes()
        );
    }
    
    #[test]
    fn test_signature_generation_and_verification() {
        // キーペアを生成
        let key_pair = StealthKeyPair::generate();
        
        // メッセージを作成
        let message = b"Test message for signature";
        
        // 署名を生成
        let signature = key_pair.sign(message);
        
        // 署名を検証
        let valid = key_pair.verify(message, &signature);
        assert!(valid);
        
        // 改ざんされたメッセージで検証
        let tampered_message = b"Tampered message for signature";
        let invalid = key_pair.verify(tampered_message, &signature);
        assert!(!invalid);
    }
}