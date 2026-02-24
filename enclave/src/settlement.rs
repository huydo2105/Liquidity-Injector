use sha2::{Digest, Sha256};

use crate::crypto::{
    decrypt_obligations, generate_attestation_quote, hash_flow_solution, CryptoError,
};
use crate::graph::{compute_mtcs_flow, validate_flow, GraphError};
use crate::types::{
    EncryptedObligation, FlowSolution, SetOffNotice, SettlementProposal,
};

/// Errors from settlement operations.
#[derive(Debug, thiserror::Error)]
pub enum SettlementError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),
    #[error("Settlement validation failed: {0}")]
    ValidationFailed(String),
}

/// The full settlement pipeline, orchestrating the TEE's role:
/// 1. Decrypt encrypted obligations using xPriv
/// 2. Build debt graph and run MTCS
/// 3. Validate the flow solution
/// 4. Generate attestation quote
/// 5. Return SettlementProposal + SetOffNotices
pub fn run_settlement_pipeline(
    encrypted_obligations: &[EncryptedObligation],
    tee_private_key: &[u8; 32],
    injection_amount: u128,
) -> Result<(SettlementProposal, Vec<SetOffNotice>), SettlementError> {
    // Step 1: Decrypt obligations inside the enclave
    let obligations = decrypt_obligations(encrypted_obligations, tee_private_key)?;

    // Step 2: Run MTCS to find optimal settlement flow
    let flow = compute_mtcs_flow(&obligations, injection_amount)?;

    // Step 3: Validate the flow (F ⊆ G, balanced, amount bounds)
    validate_flow(&flow, &obligations)?;

    // Step 4: Create proposal
    let proposal = create_proposal(flow)?;

    // Step 5: Generate set-off notices
    let notices = generate_setoff_notices(&proposal.flow);

    Ok((proposal, notices))
}

/// Create a SettlementProposal from a validated FlowSolution.
pub fn create_proposal(flow: FlowSolution) -> Result<SettlementProposal, SettlementError> {
    // Compute the proposal hash
    let proposal_hash = hash_flow_solution(&flow);

    // Generate a unique proposal ID
    let proposal_id = {
        let mut hasher = Sha256::new();
        hasher.update(&proposal_hash);
        hasher.update(b"proposal-id-salt");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes();
        hasher.update(&now);
        let result: [u8; 32] = hasher.finalize().into();
        result
    };

    let mut proposal = SettlementProposal {
        proposal_id,
        flow,
        attestation: crate::types::AttestationQuote {
            mrenclave: [0; 32],
            mrsigner: [0; 32],
            report_data: [0; 64],
            timestamp: 0,
        },
        proposal_hash,
    };

    // Generate attestation quote (in production: quartz-enclave-core)
    proposal.attestation = generate_attestation_quote(&proposal);

    Ok(proposal)
}

/// Generate set-off notices from a flow solution.
/// Each flow element produces two notices: one for the debtor and one for the creditor.
pub fn generate_setoff_notices(flow: &FlowSolution) -> Vec<SetOffNotice> {
    flow.flows
        .iter()
        .map(|f| SetOffNotice {
            debtor: f.debtor.clone(),
            creditor: f.creditor.clone(),
            amount: f.amount,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::encrypt_obligation;
    use crate::types::Obligation;

    #[test]
    fn test_full_settlement_pipeline() {
        let key = [42u8; 32];

        // Create a 3-node cycle
        let obligations = vec![
            Obligation { id: 0, debtor: "A".into(), creditor: "B".into(), amount: 100 },
            Obligation { id: 1, debtor: "B".into(), creditor: "C".into(), amount: 150 },
            Obligation { id: 2, debtor: "C".into(), creditor: "A".into(), amount: 200 },
        ];

        // Encrypt obligations
        let encrypted: Vec<EncryptedObligation> = obligations
            .iter()
            .map(|o| encrypt_obligation(o, &key).unwrap())
            .collect();

        // Run the full pipeline
        let (proposal, notices) = run_settlement_pipeline(&encrypted, &key, 0).unwrap();

        // Verify proposal
        assert!(!proposal.flow.flows.is_empty());
        assert!(proposal.flow.total_cleared > 0);
        assert_eq!(proposal.flow.injection_used, 0);
        assert_ne!(proposal.proposal_hash, [0; 32]);
        assert_ne!(proposal.attestation.mrenclave, [0; 32]);

        // Verify set-off notices
        assert!(!notices.is_empty());
    }

    #[test]
    fn test_pipeline_with_injection() {
        let key = [42u8; 32];

        let obligations = vec![
            Obligation { id: 0, debtor: "A".into(), creditor: "B".into(), amount: 100 },
            Obligation { id: 1, debtor: "B".into(), creditor: "C".into(), amount: 150 },
            Obligation { id: 2, debtor: "C".into(), creditor: "A".into(), amount: 200 },
        ];

        let encrypted: Vec<EncryptedObligation> = obligations
            .iter()
            .map(|o| encrypt_obligation(o, &key).unwrap())
            .collect();

        let (proposal, notices) = run_settlement_pipeline(&encrypted, &key, 50).unwrap();

        assert!(proposal.flow.total_cleared > 0);
        assert!(!notices.is_empty());
    }
}
