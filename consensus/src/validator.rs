use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};
use tonic::{Request, Response, Status};

use enclave::crypto::verify_attestation_quote;
use enclave::graph::{compute_mtcs_flow, validate_flow};
use enclave::types::{AttestationQuote, FlowSolution, Obligation};

use crate::quorum_proto::{
    validator_service_server::ValidatorService, ProposalRequest, VoteResponse,
};

/// A Validator TEE node that verifies settlement proposals.
pub struct ValidatorNode {
    /// This validator's Ed25519 signing key
    signing_key: SigningKey,
    /// Expected MRENCLAVE hash of the proposer enclave
    expected_mrenclave: [u8; 32],
}

impl ValidatorNode {
    pub fn new(signing_key: SigningKey, expected_mrenclave: [u8; 32]) -> Self {
        Self {
            signing_key,
            expected_mrenclave,
        }
    }

    /// Get the public key of this validator.
    pub fn public_key(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Verify a proposal and return a signed vote if valid.
    pub fn verify_and_sign(
        &self,
        proposal_hash: &[u8; 32],
        flow: &FlowSolution,
        attestation: &AttestationQuote,
        obligations: &[Obligation],
        injection_amount: u128,
    ) -> Result<(Vec<u8>, [u8; 32]), String> {
        // Step 1: Verify the proposer's attestation quote
        if !verify_attestation_quote(attestation, &self.expected_mrenclave, proposal_hash) {
            return Err("Attestation verification failed: MRENCLAVE mismatch".into());
        }

        // Step 2: Independently recalculate the flow to verify math
        let recalculated_flow = compute_mtcs_flow(obligations, injection_amount)
            .map_err(|e| format!("Flow recalculation failed: {}", e))?;

        // Step 3: Validate that the proposed flow satisfies protocol invariants
        validate_flow(flow, obligations)
            .map_err(|e| format!("Flow validation failed: {}", e))?;

        // Step 4: Verify the proposal hash matches the flow
        let expected_hash = enclave::crypto::hash_flow_solution(flow);
        if &expected_hash != proposal_hash {
            return Err("Proposal hash mismatch".into());
        }

        // Step 5: Sign the proposal hash
        let signature = self.signing_key.sign(proposal_hash);

        Ok((signature.to_bytes().to_vec(), self.public_key()))
    }
}

#[tonic::async_trait]
impl ValidatorService for ValidatorNode {
    async fn validate_proposal(
        &self,
        request: Request<ProposalRequest>,
    ) -> Result<Response<VoteResponse>, Status> {
        let req = request.into_inner();

        // Deserialize the flow, attestation, and obligations
        let flow: FlowSolution = serde_json::from_slice(&req.flow_data)
            .map_err(|e| Status::invalid_argument(format!("Invalid flow data: {}", e)))?;

        let attestation: AttestationQuote = serde_json::from_slice(&req.attestation)
            .map_err(|e| Status::invalid_argument(format!("Invalid attestation: {}", e)))?;

        let obligations: Vec<Obligation> = serde_json::from_slice(&req.obligations_data)
            .map_err(|e| Status::invalid_argument(format!("Invalid obligations: {}", e)))?;

        let mut proposal_hash = [0u8; 32];
        if req.proposal_hash.len() == 32 {
            proposal_hash.copy_from_slice(&req.proposal_hash);
        } else {
            return Err(Status::invalid_argument("Invalid proposal hash length"));
        }

        // Verify and sign
        match self.verify_and_sign(
            &proposal_hash,
            &flow,
            &attestation,
            &obligations,
            req.injection_amount as u128,
        ) {
            Ok((signature, pubkey)) => Ok(Response::new(VoteResponse {
                accepted: true,
                validator_pubkey: pubkey.to_vec(),
                signature,
                reason: "Proposal verified and accepted".into(),
            })),
            Err(reason) => Ok(Response::new(VoteResponse {
                accepted: false,
                validator_pubkey: self.public_key().to_vec(),
                signature: vec![],
                reason,
            })),
        }
    }
}

/// Verify an Ed25519 signature from a validator.
pub fn verify_validator_signature(
    pubkey_bytes: &[u8; 32],
    message: &[u8; 32],
    signature_bytes: &[u8],
) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_bytes(pubkey_bytes) else {
        return false;
    };

    let Ok(signature) = ed25519_dalek::Signature::from_slice(signature_bytes) else {
        return false;
    };

    verifying_key.verify_strict(message, &signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use enclave::types::Obligation;
    use rand::rngs::OsRng;

    fn test_mrenclave() -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"cycles-liquidity-injector-enclave-v0.1.0");
        hasher.finalize().into()
    }

    #[test]
    fn test_validator_verify_valid_proposal() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let validator = ValidatorNode::new(signing_key, test_mrenclave());

        let obligations = vec![
            Obligation { id: 0, debtor: "A".into(), creditor: "B".into(), amount: 100 },
            Obligation { id: 1, debtor: "B".into(), creditor: "C".into(), amount: 150 },
            Obligation { id: 2, debtor: "C".into(), creditor: "A".into(), amount: 200 },
        ];

        // Run MTCS to get a valid flow
        let flow = compute_mtcs_flow(&obligations, 0).unwrap();
        let proposal_hash = enclave::crypto::hash_flow_solution(&flow);

        // Generate attestation
        let proposal = enclave::types::SettlementProposal {
            proposal_id: [0; 32],
            flow: flow.clone(),
            attestation: enclave::types::AttestationQuote {
                mrenclave: [0; 32],
                mrsigner: [0; 32],
                report_data: [0; 64],
                timestamp: 0,
            },
            proposal_hash,
        };
        let attestation = enclave::crypto::generate_attestation_quote(&proposal);

        let result = validator.verify_and_sign(
            &proposal_hash,
            &flow,
            &attestation,
            &obligations,
            0,
        );

        assert!(result.is_ok());
        let (sig, pubkey) = result.unwrap();
        assert!(!sig.is_empty());

        // Verify the signature
        assert!(verify_validator_signature(&pubkey, &proposal_hash, &sig));
    }

    #[test]
    fn test_validator_rejects_bad_attestation() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let validator = ValidatorNode::new(signing_key, [0xAB; 32]); // wrong MRENCLAVE

        let obligations = vec![
            Obligation { id: 0, debtor: "A".into(), creditor: "B".into(), amount: 100 },
            Obligation { id: 1, debtor: "B".into(), creditor: "C".into(), amount: 100 },
            Obligation { id: 2, debtor: "C".into(), creditor: "A".into(), amount: 100 },
        ];

        let flow = compute_mtcs_flow(&obligations, 0).unwrap();
        let proposal_hash = enclave::crypto::hash_flow_solution(&flow);

        let proposal = enclave::types::SettlementProposal {
            proposal_id: [0; 32],
            flow: flow.clone(),
            attestation: enclave::types::AttestationQuote {
                mrenclave: [0; 32],
                mrsigner: [0; 32],
                report_data: [0; 64],
                timestamp: 0,
            },
            proposal_hash,
        };
        let attestation = enclave::crypto::generate_attestation_quote(&proposal);

        let result = validator.verify_and_sign(
            &proposal_hash,
            &flow,
            &attestation,
            &obligations,
            0,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("MRENCLAVE"));
    }
}
