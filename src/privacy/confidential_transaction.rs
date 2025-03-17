use crate::error::Error;
use crate::crypto::zk::{BulletproofProof, Bulletproof};
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_TABLE,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};
use rand::thread_rng;
use sha2::{Digest, Sha256};
use std::fmt;

/// ブラインディング係数
#[derive(Clone, Debug)]
pub struct BlindingFactor {
    /// スカラー値
    scalar: Scalar,
}

/// 機密金額
#[derive(Clone, Debug)]
pub struct ConfidentialAmount {
    /// 実際の金額
    amount: u64,
    /// ブラインディング係数
    blinding: BlindingFactor,
    /// ペダーセンコミットメント
    commitment: Option<CompressedRistretto>,
}

/// 機密トランザクション
#[derive(Clone, Debug)]
pub struct ConfidentialTransaction {
    /// 送信者
    sender: Vec<u8>,
    /// 受信者
    recipient: Vec<u8>,
    /// 機密金額
    amount: ConfidentialAmount,
    /// 手数料
    fee: u64,
    /// 範囲証明
    range_proof: BulletproofProof,
    /// 署名
    signature: Vec<u8>,
}

impl BlindingFactor {
    /// 新しいBlindingFactorを作成
    pub fn new(scalar: Scalar) -> Self {
        Self { scalar }
    }
    
    /// ランダムなBlindingFactorを生成
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let scalar = Scalar::random(&mut rng);
        Self { scalar }
    }
    
    /// スカラー値を取得
    pub fn scalar(&self) -> &Scalar {
        &self.scalar
    }
    
    /// バイト配列として取得
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.scalar.as_bytes()
    }
    
    /// バイト配列から作成
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let scalar = Scalar::from_canonical_bytes(bytes.try_into().unwrap_or([0u8; 32]))
            .ok_or_else(|| Error::DeserializationError("Invalid blinding factor format".to_string()))?;
        
        Ok(Self { scalar })
    }
}

impl ConfidentialAmount {
    /// 新しいConfidentialAmountを作成
    pub fn new(amount: u64, blinding: &BlindingFactor) -> Result<Self, Error> {
        let mut result = Self {
            amount,
            blinding: blinding.clone(),
            commitment: None,
        };
        
        // コミットメントを計算
        result.compute_commitment()?;
        
        Ok(result)
    }
    
    /// コミットメントを計算
    fn compute_commitment(&mut self) -> Result<(), Error> {
        // ペダーセンコミットメント: C = aG + bH
        // G: ベースポイント
        // H: 別のベースポイント（通常はハッシュから導出）
        // a: 金額
        // b: ブラインディング係数
        
        // 金額をスカラーに変換
        let amount_scalar = Scalar::from(self.amount);
        
        // 第2のベースポイントを導出
        let h_point = Self::derive_h_point()?;
        
        // コミットメントを計算
        let commitment = &amount_scalar * &RISTRETTO_BASEPOINT_TABLE + &self.blinding.scalar * &h_point;
        
        self.commitment = Some(commitment.compress());
        
        Ok(())
    }
    
    /// 第2のベースポイントを導出
    fn derive_h_point() -> Result<RistrettoPoint, Error> {
        // "ShardX Pedersen H" をハッシュ化
        let mut hasher = Sha256::new();
        hasher.update(b"ShardX Pedersen H");
        let hash = hasher.finalize();
        
        // ハッシュからポイントを導出
        let h_compressed = CompressedRistretto::from_slice(&hash[0..32])
            .map_err(|_| Error::CryptoError("Failed to create H point".to_string()))?;
        
        let h_point = h_compressed.decompress()
            .ok_or_else(|| Error::CryptoError("Failed to decompress H point".to_string()))?;
        
        Ok(h_point)
    }
    
    /// 金額を取得
    pub fn get_amount(&self) -> u64 {
        self.amount
    }
    
    /// ブラインディング係数を取得
    pub fn get_blinding(&self) -> &BlindingFactor {
        &self.blinding
    }
    
    /// コミットメントを取得
    pub fn get_commitment(&self) -> Result<Vec<u8>, Error> {
        match &self.commitment {
            Some(commitment) => Ok(commitment.as_bytes().to_vec()),
            None => {
                // コミットメントがまだ計算されていない場合は計算
                let mut clone = self.clone();
                clone.compute_commitment()?;
                
                match &clone.commitment {
                    Some(commitment) => Ok(commitment.as_bytes().to_vec()),
                    None => Err(Error::CryptoError("Failed to compute commitment".to_string())),
                }
            }
        }
    }
}

impl ConfidentialTransaction {
    /// 新しいConfidentialTransactionを作成
    pub fn new(
        sender: &[u8],
        recipient: &[u8],
        amount: ConfidentialAmount,
        fee: u64,
        range_proof: BulletproofProof,
        private_key: &[u8],
    ) -> Result<Self, Error> {
        let mut transaction = Self {
            sender: sender.to_vec(),
            recipient: recipient.to_vec(),
            amount,
            fee,
            range_proof,
            signature: Vec::new(),
        };
        
        // トランザクションに署名
        transaction.sign(private_key)?;
        
        Ok(transaction)
    }
    
    /// トランザクションに署名
    fn sign(&mut self, private_key: &[u8]) -> Result<(), Error> {
        // 署名対象のデータを準備
        let mut data = Vec::new();
        data.extend_from_slice(&self.sender);
        data.extend_from_slice(&self.recipient);
        data.extend_from_slice(&self.amount.get_commitment()?);
        data.extend_from_slice(&self.fee.to_le_bytes());
        
        // 秘密鍵をスカラーに変換
        let private_scalar = Scalar::from_canonical_bytes(private_key.try_into().unwrap_or([0u8; 32]))
            .ok_or_else(|| Error::DeserializationError("Invalid private key format".to_string()))?;
        
        // データをハッシュ化
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let message_hash = hasher.finalize();
        
        // ランダムな値を生成
        let mut rng = thread_rng();
        let k = Scalar::random(&mut rng);
        
        // R = k*G
        let r_point = &k * &RISTRETTO_BASEPOINT_TABLE;
        let r = r_point.compress().to_bytes();
        
        // e = H(R || P || m)
        let public_key = &private_scalar * &RISTRETTO_BASEPOINT_TABLE;
        
        let mut e_hasher = Sha256::new();
        e_hasher.update(&r);
        e_hasher.update(public_key.compress().as_bytes());
        e_hasher.update(&message_hash);
        let e_hash = e_hasher.finalize();
        let e = Scalar::from_bytes_mod_order_wide(&<[u8; 64]>::try_from(&e_hash[..]).unwrap_or([0u8; 64]));
        
        // s = k - e*x
        let s = k - e * private_scalar;
        
        // 署名 = (R, s)
        let mut signature = Vec::with_capacity(64);
        signature.extend_from_slice(&r);
        signature.extend_from_slice(&s.to_bytes());
        
        self.signature = signature;
        
        Ok(())
    }
    
    /// 署名を検証
    pub fn verify_signature(&self) -> Result<bool, Error> {
        if self.signature.len() != 64 {
            return Ok(false);
        }
        
        // 署名を分解
        let r_bytes = &self.signature[0..32];
        let s_bytes = &self.signature[32..64];
        
        let r_point = match CompressedRistretto::from_slice(r_bytes) {
            Ok(point) => point,
            Err(_) => return Ok(false),
        };
        
        let s = match Scalar::from_canonical_bytes(s_bytes.try_into().unwrap_or([0u8; 32])) {
            Some(scalar) => scalar,
            None => return Ok(false),
        };
        
        // 署名対象のデータを準備
        let mut data = Vec::new();
        data.extend_from_slice(&self.sender);
        data.extend_from_slice(&self.recipient);
        data.extend_from_slice(&self.amount.get_commitment()?);
        data.extend_from_slice(&self.fee.to_le_bytes());
        
        // データをハッシュ化
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let message_hash = hasher.finalize();
        
        // 公開鍵を取得（送信者のアドレスから）
        let public_key_bytes = &self.sender;
        let public_key = match CompressedRistretto::from_slice(public_key_bytes) {
            Ok(key) => key,
            Err(_) => return Ok(false),
        };
        
        let public_key_point = match public_key.decompress() {
            Some(point) => point,
            None => return Ok(false),
        };
        
        // e = H(R || P || m)
        let mut e_hasher = Sha256::new();
        e_hasher.update(r_bytes);
        e_hasher.update(public_key.as_bytes());
        e_hasher.update(&message_hash);
        let e_hash = e_hasher.finalize();
        let e = Scalar::from_bytes_mod_order_wide(&<[u8; 64]>::try_from(&e_hash[..]).unwrap_or([0u8; 64]));
        
        // R' = s*G + e*P
        let r_prime = &s * &RISTRETTO_BASEPOINT_TABLE + &e * &public_key_point;
        
        // R' == R
        Ok(r_prime.compress() == r_point)
    }
    
    /// 送信者を取得
    pub fn get_sender(&self) -> &[u8] {
        &self.sender
    }
    
    /// 受信者を取得
    pub fn get_recipient(&self) -> &[u8] {
        &self.recipient
    }
    
    /// 金額を取得
    pub fn get_amount(&self) -> &ConfidentialAmount {
        &self.amount
    }
    
    /// 手数料を取得
    pub fn get_fee(&self) -> u64 {
        self.fee
    }
    
    /// 範囲証明を取得
    pub fn get_range_proof(&self) -> &BulletproofProof {
        &self.range_proof
    }
    
    /// 署名を取得
    pub fn get_signature(&self) -> &[u8] {
        &self.signature
    }
    
    /// トランザクションをシリアライズ
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();
        
        // 送信者
        bytes.extend_from_slice(&(self.sender.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&self.sender);
        
        // 受信者
        bytes.extend_from_slice(&(self.recipient.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&self.recipient);
        
        // 金額コミットメント
        let commitment = self.amount.get_commitment()?;
        bytes.extend_from_slice(&commitment);
        
        // ブラインディング係数
        bytes.extend_from_slice(self.amount.blinding.as_bytes());
        
        // 手数料
        bytes.extend_from_slice(&self.fee.to_le_bytes());
        
        // 範囲証明
        let range_proof_bytes = Bulletproof::serialize_proof(&self.range_proof)?;
        bytes.extend_from_slice(&(range_proof_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&range_proof_bytes);
        
        // 署名
        bytes.extend_from_slice(&self.signature);
        
        Ok(bytes)
    }
    
    /// トランザクションをデシリアライズ
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 10 {
            return Err(Error::DeserializationError("Invalid transaction data".to_string()));
        }
        
        let mut pos = 0;
        
        // 送信者
        let sender_len = u16::from_le_bytes([bytes[pos], bytes[pos + 1]]) as usize;
        pos += 2;
        
        if pos + sender_len > bytes.len() {
            return Err(Error::DeserializationError("Invalid sender length".to_string()));
        }
        
        let sender = bytes[pos..pos + sender_len].to_vec();
        pos += sender_len;
        
        // 受信者
        if pos + 2 > bytes.len() {
            return Err(Error::DeserializationError("Invalid recipient data".to_string()));
        }
        
        let recipient_len = u16::from_le_bytes([bytes[pos], bytes[pos + 1]]) as usize;
        pos += 2;
        
        if pos + recipient_len > bytes.len() {
            return Err(Error::DeserializationError("Invalid recipient length".to_string()));
        }
        
        let recipient = bytes[pos..pos + recipient_len].to_vec();
        pos += recipient_len;
        
        // 金額コミットメント
        if pos + 32 > bytes.len() {
            return Err(Error::DeserializationError("Invalid commitment data".to_string()));
        }
        
        let commitment_bytes = &bytes[pos..pos + 32];
        pos += 32;
        
        // ブラインディング係数
        if pos + 32 > bytes.len() {
            return Err(Error::DeserializationError("Invalid blinding data".to_string()));
        }
        
        let blinding_bytes = &bytes[pos..pos + 32];
        let blinding = BlindingFactor::from_bytes(blinding_bytes)?;
        pos += 32;
        
        // 手数料
        if pos + 8 > bytes.len() {
            return Err(Error::DeserializationError("Invalid fee data".to_string()));
        }
        
        let fee = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
        pos += 8;
        
        // 範囲証明
        if pos + 4 > bytes.len() {
            return Err(Error::DeserializationError("Invalid range proof length data".to_string()));
        }
        
        let range_proof_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        
        if pos + range_proof_len > bytes.len() {
            return Err(Error::DeserializationError("Invalid range proof data".to_string()));
        }
        
        let range_proof_bytes = &bytes[pos..pos + range_proof_len];
        let range_proof = Bulletproof::deserialize_proof(range_proof_bytes)?;
        pos += range_proof_len;
        
        // 署名
        if pos + 64 > bytes.len() {
            return Err(Error::DeserializationError("Invalid signature data".to_string()));
        }
        
        let signature = bytes[pos..pos + 64].to_vec();
        
        // 金額を0として仮設定（実際の金額は秘匿されている）
        let amount = ConfidentialAmount {
            amount: 0,
            blinding,
            commitment: Some(CompressedRistretto::from_slice(commitment_bytes)
                .map_err(|_| Error::DeserializationError("Invalid commitment format".to_string()))?),
        };
        
        Ok(Self {
            sender,
            recipient,
            amount,
            fee,
            range_proof,
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    
    #[test]
    fn test_blinding_factor() {
        // ランダムなブラインディング係数を生成
        let blinding = BlindingFactor::random();
        
        // バイト配列に変換
        let bytes = blinding.as_bytes();
        
        // バイト配列から復元
        let recovered = BlindingFactor::from_bytes(bytes).unwrap();
        
        // 元のブラインディング係数と一致することを確認
        assert_eq!(blinding.scalar.to_bytes(), recovered.scalar.to_bytes());
    }
    
    #[test]
    fn test_confidential_amount() {
        // ランダムな金額を生成
        let amount = rand::thread_rng().gen_range(1..1000);
        
        // ブラインディング係数を生成
        let blinding = BlindingFactor::random();
        
        // 機密金額を作成
        let confidential_amount = ConfidentialAmount::new(amount, &blinding).unwrap();
        
        // コミットメントを取得
        let commitment = confidential_amount.get_commitment().unwrap();
        
        // 金額とブラインディング係数が正しく保存されていることを確認
        assert_eq!(confidential_amount.get_amount(), amount);
        assert_eq!(
            confidential_amount.get_blinding().scalar.to_bytes(),
            blinding.scalar.to_bytes()
        );
        
        // コミットメントが32バイトであることを確認
        assert_eq!(commitment.len(), 32);
    }
    
    #[test]
    fn test_confidential_transaction() {
        // 送信者と受信者のキーペアを生成
        let sender_keypair = curve25519_dalek::ristretto::RistrettoPoint::random(&mut thread_rng());
        let recipient_keypair = curve25519_dalek::ristretto::RistrettoPoint::random(&mut thread_rng());
        
        let sender = sender_keypair.compress().as_bytes().to_vec();
        let recipient = recipient_keypair.compress().as_bytes().to_vec();
        
        // 金額とブラインディング係数を生成
        let amount = rand::thread_rng().gen_range(1..1000);
        let blinding = BlindingFactor::random();
        
        // 機密金額を作成
        let confidential_amount = ConfidentialAmount::new(amount, &blinding).unwrap();
        
        // 範囲証明を生成（実際のアプリケーションでは、Bulletproofライブラリを使用）
        let range_proof = BulletproofProof {
            inner: bulletproofs::RangeProof::new(vec![amount], vec![64], vec![blinding.scalar], None, None, None).unwrap(),
        };
        
        // 秘密鍵を生成
        let private_key = Scalar::random(&mut thread_rng()).to_bytes();
        
        // 機密トランザクションを作成
        let transaction = ConfidentialTransaction::new(
            &sender,
            &recipient,
            confidential_amount,
            10, // 手数料
            range_proof,
            &private_key,
        ).unwrap();
        
        // トランザクションをシリアライズ
        let serialized = transaction.to_bytes().unwrap();
        
        // トランザクションをデシリアライズ
        let deserialized = ConfidentialTransaction::from_bytes(&serialized).unwrap();
        
        // 元のトランザクションと一致することを確認
        assert_eq!(transaction.get_sender(), deserialized.get_sender());
        assert_eq!(transaction.get_recipient(), deserialized.get_recipient());
        assert_eq!(transaction.get_fee(), deserialized.get_fee());
        assert_eq!(transaction.get_signature(), deserialized.get_signature());
    }
}