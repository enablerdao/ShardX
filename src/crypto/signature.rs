use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use rand::rngs::OsRng;
use crate::error::Error;
use crate::transaction::Transaction;

/// 署名マネージャー
pub struct SignatureManager {
    /// 署名キー
    signing_key: Option<SigningKey>,
    /// 検証キー
    verifying_key: Option<VerifyingKey>,
}

impl SignatureManager {
    /// 新しいSignatureManagerを作成
    pub fn new() -> Self {
        Self {
            signing_key: None,
            verifying_key: None,
        }
    }
    
    /// 新しいキーペアを生成
    pub fn generate_keypair(&mut self) -> Result<(), Error> {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        self.signing_key = Some(signing_key);
        self.verifying_key = Some(verifying_key);
        
        Ok(())
    }
    
    /// 署名キーを設定
    pub fn set_signing_key(&mut self, key_bytes: &[u8]) -> Result<(), Error> {
        if key_bytes.len() != 32 {
            return Err(Error::InvalidKey("Invalid signing key length".to_string()));
        }
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(key_bytes);
        
        let signing_key = SigningKey::from_bytes(&bytes);
        let verifying_key = signing_key.verifying_key();
        
        self.signing_key = Some(signing_key);
        self.verifying_key = Some(verifying_key);
        
        Ok(())
    }
    
    /// 検証キーを設定
    pub fn set_verifying_key(&mut self, key_bytes: &[u8]) -> Result<(), Error> {
        if key_bytes.len() != 32 {
            return Err(Error::InvalidKey("Invalid verifying key length".to_string()));
        }
        
        let verifying_key = VerifyingKey::from_bytes(key_bytes)
            .map_err(|e| Error::InvalidKey(format!("Invalid verifying key: {}", e)))?;
        
        self.verifying_key = Some(verifying_key);
        
        Ok(())
    }
    
    /// トランザクションに署名
    pub fn sign_transaction(&self, tx: &mut Transaction) -> Result<(), Error> {
        let signing_key = self.signing_key.as_ref()
            .ok_or_else(|| Error::InvalidKey("Signing key not set".to_string()))?;
        
        // トランザクションをシリアライズ（署名を除く）
        let serialized = bincode::serialize(&tx.to_signable())
            .map_err(|e| Error::SerializeError(format!("Failed to serialize transaction: {}", e)))?;
        
        // 署名を生成
        let signature = signing_key.sign(&serialized);
        
        // 署名をトランザクションに設定
        tx.signature = signature.to_bytes().to_vec();
        
        Ok(())
    }
    
    /// トランザクションの署名を検証
    pub fn verify_transaction(&self, tx: &Transaction) -> Result<(), Error> {
        let verifying_key = self.verifying_key.as_ref()
            .ok_or_else(|| Error::InvalidKey("Verifying key not set".to_string()))?;
        
        // 署名を取得
        if tx.signature.len() != 64 {
            return Err(Error::InvalidSignature);
        }
        
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&tx.signature);
        
        let signature = Signature::from_bytes(&sig_bytes);
        
        // トランザクションをシリアライズ（署名を除く）
        let serialized = bincode::serialize(&tx.to_signable())
            .map_err(|e| Error::SerializeError(format!("Failed to serialize transaction: {}", e)))?;
        
        // 署名を検証
        verifying_key.verify(&serialized, &signature)
            .map_err(|_| Error::InvalidSignature)
    }
    
    /// データに署名
    pub fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let signing_key = self.signing_key.as_ref()
            .ok_or_else(|| Error::InvalidKey("Signing key not set".to_string()))?;
        
        // 署名を生成
        let signature = signing_key.sign(data);
        
        Ok(signature.to_bytes().to_vec())
    }
    
    /// 署名を検証
    pub fn verify_data(&self, data: &[u8], signature_bytes: &[u8]) -> Result<(), Error> {
        let verifying_key = self.verifying_key.as_ref()
            .ok_or_else(|| Error::InvalidKey("Verifying key not set".to_string()))?;
        
        // 署名を取得
        if signature_bytes.len() != 64 {
            return Err(Error::InvalidSignature);
        }
        
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature_bytes);
        
        let signature = Signature::from_bytes(&sig_bytes);
        
        // 署名を検証
        verifying_key.verify(data, &signature)
            .map_err(|_| Error::InvalidSignature)
    }
    
    /// 署名キーを取得
    pub fn get_signing_key_bytes(&self) -> Result<[u8; 32], Error> {
        let signing_key = self.signing_key.as_ref()
            .ok_or_else(|| Error::InvalidKey("Signing key not set".to_string()))?;
        
        Ok(signing_key.to_bytes())
    }
    
    /// 検証キーを取得
    pub fn get_verifying_key_bytes(&self) -> Result<[u8; 32], Error> {
        let verifying_key = self.verifying_key.as_ref()
            .ok_or_else(|| Error::InvalidKey("Verifying key not set".to_string()))?;
        
        Ok(verifying_key.to_bytes())
    }
    
    /// 検証キーを16進数文字列として取得
    pub fn get_verifying_key_hex(&self) -> Result<String, Error> {
        let bytes = self.get_verifying_key_bytes()?;
        Ok(hex::encode(bytes))
    }
}

impl Default for SignatureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::TransactionStatus;
    
    fn create_test_transaction() -> Transaction {
        Transaction {
            id: "tx1".to_string(),
            parent_ids: vec!["parent1".to_string()],
            timestamp: 12345,
            payload: vec![1, 2, 3, 4, 5],
            signature: vec![],
            status: TransactionStatus::Pending,
        }
    }
    
    #[test]
    fn test_generate_keypair() {
        let mut signature_manager = SignatureManager::new();
        
        // キーペアを生成
        let result = signature_manager.generate_keypair();
        assert!(result.is_ok());
        
        // 署名キーが設定されていることを確認
        let signing_key = signature_manager.get_signing_key_bytes();
        assert!(signing_key.is_ok());
        
        // 検証キーが設定されていることを確認
        let verifying_key = signature_manager.get_verifying_key_bytes();
        assert!(verifying_key.is_ok());
    }
    
    #[test]
    fn test_sign_and_verify_transaction() {
        let mut signature_manager = SignatureManager::new();
        signature_manager.generate_keypair().unwrap();
        
        // トランザクションを作成
        let mut tx = create_test_transaction();
        
        // トランザクションに署名
        let result = signature_manager.sign_transaction(&mut tx);
        assert!(result.is_ok());
        
        // 署名が設定されていることを確認
        assert_eq!(tx.signature.len(), 64);
        
        // 署名を検証
        let result = signature_manager.verify_transaction(&tx);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_sign_and_verify_data() {
        let mut signature_manager = SignatureManager::new();
        signature_manager.generate_keypair().unwrap();
        
        // データを作成
        let data = b"test data";
        
        // データに署名
        let signature = signature_manager.sign_data(data).unwrap();
        
        // 署名が正しいサイズであることを確認
        assert_eq!(signature.len(), 64);
        
        // 署名を検証
        let result = signature_manager.verify_data(data, &signature);
        assert!(result.is_ok());
        
        // 改ざんされたデータで検証
        let tampered_data = b"tampered data";
        let result = signature_manager.verify_data(tampered_data, &signature);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_key_export_import() {
        let mut signature_manager1 = SignatureManager::new();
        signature_manager1.generate_keypair().unwrap();
        
        // キーをエクスポート
        let signing_key_bytes = signature_manager1.get_signing_key_bytes().unwrap();
        let verifying_key_bytes = signature_manager1.get_verifying_key_bytes().unwrap();
        
        // 新しいマネージャーを作成
        let mut signature_manager2 = SignatureManager::new();
        
        // 署名キーをインポート
        let result = signature_manager2.set_signing_key(&signing_key_bytes);
        assert!(result.is_ok());
        
        // データを署名して検証
        let data = b"test data";
        let signature1 = signature_manager1.sign_data(data).unwrap();
        let signature2 = signature_manager2.sign_data(data).unwrap();
        
        // 両方のマネージャーで署名を検証
        assert!(signature_manager1.verify_data(data, &signature2).is_ok());
        assert!(signature_manager2.verify_data(data, &signature1).is_ok());
    }
}