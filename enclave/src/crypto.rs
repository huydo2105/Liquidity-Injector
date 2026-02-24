use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::types::{
    AttestationQuote, EncryptedObligation, Obligation, SettlementProposal, FlowSolution,
};

/// Errors from cryptographic operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
}

/// Encrypt an obligation using AES-256-GCM (encrypting to TEE's xPub).
/// In production, this uses ECDH with the TEE's public key to derive a shared secret.
/// Here we use a symmetric key directly for simplicity.
pub fn encrypt_obligation(
    obligation: &Obligation,
    key: &[u8; 32],
) -> Result<EncryptedObligation, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = serde_json::to_vec(obligation)
        .map_err(|e| CryptoError::SerializationFailed(e.to_string()))?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    Ok(EncryptedObligation {
        id: obligation.id,
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

/// Decrypt an obligation inside the TEE enclave using the TEE's private key (xPriv).
pub fn decrypt_obligation(
    encrypted: &EncryptedObligation,
    key: &[u8; 32],
) -> Result<Obligation, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    let nonce = Nonce::from_slice(&encrypted.nonce);

    let plaintext = cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    serde_json::from_slice(&plaintext)
        .map_err(|e| CryptoError::SerializationFailed(e.to_string()))
}

/// Decrypt a batch of encrypted obligations.
pub fn decrypt_obligations(
    encrypted_obligations: &[EncryptedObligation],
    key: &[u8; 32],
) -> Result<Vec<Obligation>, CryptoError> {
    encrypted_obligations
        .iter()
        .map(|e| decrypt_obligation(e, key))
        .collect()
}

/// Compute SHA-256 hash of a flow solution (used as proposal_hash for signing).
pub fn hash_flow_solution(flow: &FlowSolution) -> [u8; 32] {
    let serialized = serde_json::to_vec(flow).expect("flow serialization should not fail");
    let mut hasher = Sha256::new();
    hasher.update(&serialized);
    hasher.finalize().into()
}

/// Generate a simulated SGX attestation quote.
/// In production, this would call `quartz-enclave-core::produce_attestation()`.
pub fn generate_attestation_quote(proposal: &SettlementProposal) -> AttestationQuote {
    // Simulated MRENCLAVE — hash of the "enclave binary"
    let mrenclave = {
        let mut hasher = Sha256::new();
        hasher.update(b"cycles-liquidity-injector-enclave-v0.1.0");
        let result: [u8; 32] = hasher.finalize().into();
        result
    };

    // Simulated MRSIGNER — hash of the "enclave signer"
    let mrsigner = {
        let mut hasher = Sha256::new();
        hasher.update(b"informal-systems-quartz-signer");
        let result: [u8; 32] = hasher.finalize().into();
        result
    };

    // Report data: first 32 bytes = proposal hash, rest zeroed
    let mut report_data = [0u8; 64];
    report_data[..32].copy_from_slice(&proposal.proposal_hash);

    AttestationQuote {
        mrenclave,
        mrsigner,
        report_data,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

/// Verify an attestation quote against expected MRENCLAVE.
/// In production, this verifies the full DCAP certificate chain.
pub fn verify_attestation_quote(
    quote: &AttestationQuote,
    expected_mrenclave: &[u8; 32],
    expected_proposal_hash: &[u8; 32],
) -> bool {
    // 1. Check MRENCLAVE matches expected enclave binary
    if &quote.mrenclave != expected_mrenclave {
        return false;
    }

    // 2. Check the proposal hash is embedded in report_data
    if &quote.report_data[..32] != expected_proposal_hash {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Obligation;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32]; // test key
        let obligation = Obligation {
            id: 1,
            debtor: "alice".to_string(),
            creditor: "bob".to_string(),
            amount: 1000,
        };

        let encrypted = encrypt_obligation(&obligation, &key).unwrap();
        let decrypted = decrypt_obligation(&encrypted, &key).unwrap();

        assert_eq!(obligation, decrypted);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key = [42u8; 32];
        let wrong_key = [99u8; 32];
        let obligation = Obligation {
            id: 1,
            debtor: "alice".to_string(),
            creditor: "bob".to_string(),
            amount: 1000,
        };

        let encrypted = encrypt_obligation(&obligation, &key).unwrap();
        let result = decrypt_obligation(&encrypted, &wrong_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_flow_deterministic() {
        let flow = FlowSolution {
            flows: vec![],
            total_cleared: 0,
            injection_used: 0,
        };
        let h1 = hash_flow_solution(&flow);
        let h2 = hash_flow_solution(&flow);
        assert_eq!(h1, h2);
    }
}
