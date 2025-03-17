use crate::error::Error;
use bulletproofs::{BulletproofGens, PedersenGens, RangeProof as BPRangeProof};
use curve25519_dalek::{
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};
use merlin::Transcript;
use rand::thread_rng;

/// Bulletproofs実装
pub struct Bulletproof;

/// Bulletproofの証明
#[derive(Clone, Debug)]
pub struct BulletproofProof {
    /// 内部的なbulletproofsのRangeProof
    pub inner: BPRangeProof,
}

/// 範囲証明
#[derive(Clone, Debug)]
pub struct RangeProof {
    /// 値
    pub value: u64,
    /// ブラインディング係数
    pub blinding: Scalar,
    /// コミットメント
    pub commitment: CompressedRistretto,
    /// 証明
    pub proof: BulletproofProof,
}

impl Bulletproof {
    /// 範囲証明を生成
    pub fn prove_range(value: u64, blinding_bytes: &[u8]) -> Result<BulletproofProof, Error> {
        // ブラインディング係数を作成
        let blinding = Self::bytes_to_scalar(blinding_bytes)?;

        // Pedersen生成器を作成
        let pc_gens = PedersenGens::default();

        // Bulletproof生成器を作成（64ビットの範囲証明用）
        let bp_gens = BulletproofGens::new(64, 1);

        // トランスクリプトを作成
        let mut transcript = Transcript::new(b"ShardX Range Proof");

        // 範囲証明を生成
        let (proof, _commitment) =
            BPRangeProof::prove_single(&bp_gens, &pc_gens, &mut transcript, value, &blinding, 64)
                .map_err(|e| Error::CryptoError(format!("Failed to create range proof: {}", e)))?;

        Ok(BulletproofProof { inner: proof })
    }

    /// 範囲証明を検証
    pub fn verify_range(proof: &BulletproofProof, commitment_bytes: &[u8]) -> Result<bool, Error> {
        // コミットメントをデシリアライズ
        let commitment = CompressedRistretto::from_slice(commitment_bytes)
            .map_err(|_| Error::DeserializationError("Invalid commitment format".to_string()))?;

        // Pedersen生成器を作成
        let pc_gens = PedersenGens::default();

        // Bulletproof生成器を作成（64ビットの範囲証明用）
        let bp_gens = BulletproofGens::new(64, 1);

        // トランスクリプトを作成
        let mut transcript = Transcript::new(b"ShardX Range Proof");

        // 範囲証明を検証
        let result = proof
            .inner
            .verify_single(&bp_gens, &pc_gens, &mut transcript, &commitment, 64)
            .is_ok();

        Ok(result)
    }

    /// 値とブラインディング係数からコミットメントを作成
    pub fn commit(value: u64, blinding_bytes: &[u8]) -> Result<Vec<u8>, Error> {
        // ブラインディング係数を作成
        let blinding = Self::bytes_to_scalar(blinding_bytes)?;

        // Pedersen生成器を作成
        let pc_gens = PedersenGens::default();

        // コミットメントを計算
        let commitment = pc_gens.commit(Scalar::from(value), blinding);

        // コミットメントをシリアライズ
        Ok(commitment.compress().to_bytes().to_vec())
    }

    /// 完全な範囲証明を生成（コミットメントと証明）
    pub fn create_range_proof(value: u64) -> Result<RangeProof, Error> {
        // ランダムなブラインディング係数を生成
        let blinding = Scalar::random(&mut thread_rng());

        // Pedersen生成器を作成
        let pc_gens = PedersenGens::default();

        // Bulletproof生成器を作成（64ビットの範囲証明用）
        let bp_gens = BulletproofGens::new(64, 1);

        // トランスクリプトを作成
        let mut transcript = Transcript::new(b"ShardX Range Proof");

        // 範囲証明を生成
        let (proof, commitment) =
            BPRangeProof::prove_single(&bp_gens, &pc_gens, &mut transcript, value, &blinding, 64)
                .map_err(|e| Error::CryptoError(format!("Failed to create range proof: {}", e)))?;

        Ok(RangeProof {
            value,
            blinding,
            commitment: commitment,
            proof: BulletproofProof { inner: proof },
        })
    }

    /// 範囲証明を検証（完全な証明）
    pub fn verify_complete_range_proof(range_proof: &RangeProof) -> Result<bool, Error> {
        // Pedersen生成器を作成
        let pc_gens = PedersenGens::default();

        // Bulletproof生成器を作成（64ビットの範囲証明用）
        let bp_gens = BulletproofGens::new(64, 1);

        // トランスクリプトを作成
        let mut transcript = Transcript::new(b"ShardX Range Proof");

        // 範囲証明を検証
        let result = range_proof
            .proof
            .inner
            .verify_single(
                &bp_gens,
                &pc_gens,
                &mut transcript,
                &range_proof.commitment,
                64,
            )
            .is_ok();

        Ok(result)
    }

    /// バイト列をScalarに変換
    fn bytes_to_scalar(bytes: &[u8]) -> Result<Scalar, Error> {
        if bytes.len() != 32 {
            return Err(Error::InvalidArgument(
                "Blinding must be 32 bytes".to_string(),
            ));
        }

        let mut scalar_bytes = [0u8; 32];
        scalar_bytes.copy_from_slice(bytes);

        Ok(Scalar::from_bytes_mod_order(scalar_bytes))
    }

    /// 証明をシリアライズ
    pub fn serialize_proof(proof: &BulletproofProof) -> Result<Vec<u8>, Error> {
        let bytes = proof.inner.to_bytes();
        Ok(bytes)
    }

    /// 証明をデシリアライズ
    pub fn deserialize_proof(data: &[u8]) -> Result<BulletproofProof, Error> {
        let inner = BPRangeProof::from_bytes(data).map_err(|_| {
            Error::DeserializationError("Failed to deserialize Bulletproof".to_string())
        })?;

        Ok(BulletproofProof { inner })
    }

    /// 複数の値の範囲証明を一括で生成
    pub fn batch_prove_range(
        values: &[u64],
        blindings: &[&[u8]],
    ) -> Result<Vec<BulletproofProof>, Error> {
        if values.len() != blindings.len() {
            return Err(Error::InvalidArgument(
                "Number of values and blindings must match".to_string(),
            ));
        }

        let mut proofs = Vec::with_capacity(values.len());

        for (value, blinding) in values.iter().zip(blindings.iter()) {
            let proof = Self::prove_range(*value, blinding)?;
            proofs.push(proof);
        }

        Ok(proofs)
    }

    /// 複数の範囲証明を一括で検証
    pub fn batch_verify_range(
        proofs: &[BulletproofProof],
        commitments: &[&[u8]],
    ) -> Result<bool, Error> {
        if proofs.len() != commitments.len() {
            return Err(Error::InvalidArgument(
                "Number of proofs and commitments must match".to_string(),
            ));
        }

        for (proof, commitment) in proofs.iter().zip(commitments.iter()) {
            if !Self::verify_range(proof, commitment)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_range_proof() {
        // ランダムな値とブラインディングを生成
        let value = rand::thread_rng().gen_range(0..1000);
        let mut blinding = [0u8; 32];
        rand::thread_rng().fill(&mut blinding);

        // コミットメントを作成
        let commitment = Bulletproof::commit(value, &blinding).unwrap();

        // 範囲証明を生成
        let proof = Bulletproof::prove_range(value, &blinding).unwrap();

        // 範囲証明を検証
        let result = Bulletproof::verify_range(&proof, &commitment).unwrap();
        assert!(result);

        // 不正な値で検証
        let invalid_value = value + 1;
        let invalid_commitment = Bulletproof::commit(invalid_value, &blinding).unwrap();
        let result = Bulletproof::verify_range(&proof, &invalid_commitment).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_complete_range_proof() {
        // 完全な範囲証明を生成
        let value = rand::thread_rng().gen_range(0..1000);
        let range_proof = Bulletproof::create_range_proof(value).unwrap();

        // 範囲証明を検証
        let result = Bulletproof::verify_complete_range_proof(&range_proof).unwrap();
        assert!(result);
    }

    #[test]
    fn test_serialization() {
        // 範囲証明を生成
        let value = rand::thread_rng().gen_range(0..1000);
        let range_proof = Bulletproof::create_range_proof(value).unwrap();

        // 証明をシリアライズ
        let serialized = Bulletproof::serialize_proof(&range_proof.proof).unwrap();

        // 証明をデシリアライズ
        let deserialized = Bulletproof::deserialize_proof(&serialized).unwrap();

        // 元の証明と一致することを確認
        let re_serialized = Bulletproof::serialize_proof(&deserialized).unwrap();
        assert_eq!(serialized, re_serialized);
    }

    #[test]
    fn test_batch_operations() {
        // 複数の値とブラインディングを生成
        let mut values = Vec::new();
        let mut blindings = Vec::new();
        let mut blinding_refs = Vec::new();
        let mut commitments = Vec::new();
        let mut commitment_refs = Vec::new();

        for _ in 0..5 {
            let value = rand::thread_rng().gen_range(0..1000);
            let mut blinding = [0u8; 32];
            rand::thread_rng().fill(&mut blinding);

            values.push(value);
            blindings.push(blinding);
            blinding_refs.push(blindings.last().unwrap().as_slice());

            let commitment = Bulletproof::commit(value, &blinding).unwrap();
            commitments.push(commitment);
        }

        for commitment in &commitments {
            commitment_refs.push(commitment.as_slice());
        }

        // 一括で範囲証明を生成
        let proofs = Bulletproof::batch_prove_range(&values, &blinding_refs).unwrap();

        // 一括で範囲証明を検証
        let result = Bulletproof::batch_verify_range(&proofs, &commitment_refs).unwrap();
        assert!(result);
    }
}
