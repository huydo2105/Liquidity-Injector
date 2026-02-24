use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tonic::{Request, Response, Status};

use enclave::types::{QuorumCertificate, ValidatorVote};

use crate::quorum_proto::{
    collector_service_server::CollectorService,
    QuorumCertificateProto, QuorumStatusResponse, StatusRequest,
    ValidatorSignature, ValidatorStatus, VoteRequest,
};
use crate::validator::verify_validator_signature;

/// State tracking for a single proposal.
#[derive(Debug, Clone)]
struct ProposalState {
    proposal_hash: [u8; 32],
    votes: Vec<ValidatorVote>,
    voter_set: std::collections::HashSet<[u8; 32]>,
}

/// The Collector service aggregates validator signatures into a Quorum Certificate.
pub struct CollectorNode {
    /// Known validator public keys (the committee).
    committee: Vec<[u8; 32]>,
    /// Quorum threshold (2f + 1).
    quorum_threshold: u32,
    /// Active proposals being collected.
    proposals: Arc<Mutex<HashMap<[u8; 32], ProposalState>>>,
}

impl CollectorNode {
    pub fn new(committee: Vec<[u8; 32]>, quorum_threshold: u32) -> Self {
        Self {
            committee,
            quorum_threshold,
            proposals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Submit a vote and return the current quorum status.
    pub fn submit_vote_sync(
        &self,
        proposal_hash: [u8; 32],
        validator_pubkey: [u8; 32],
        signature: Vec<u8>,
    ) -> Result<QuorumStatus, String> {
        // Verify the validator is in the committee
        if !self.committee.contains(&validator_pubkey) {
            return Err("Unknown validator: not in committee".into());
        }

        // Verify the signature
        if !verify_validator_signature(&validator_pubkey, &proposal_hash, &signature) {
            return Err("Invalid signature".into());
        }

        let mut proposals = self.proposals.lock().unwrap();
        let state = proposals.entry(proposal_hash).or_insert_with(|| ProposalState {
            proposal_hash,
            votes: Vec::new(),
            voter_set: std::collections::HashSet::new(),
        });

        // Check for duplicate votes
        if state.voter_set.contains(&validator_pubkey) {
            return Err("Duplicate vote from this validator".into());
        }

        // Add the vote
        state.voter_set.insert(validator_pubkey);
        state.votes.push(ValidatorVote {
            validator_pubkey,
            proposal_hash,
            signature,
        });

        let signed = state.votes.len() as u32;
        let quorum_reached = signed >= self.quorum_threshold;

        let certificate = if quorum_reached {
            Some(QuorumCertificate {
                proposal_hash,
                signatures: state.votes.clone(),
                threshold: self.quorum_threshold,
            })
        } else {
            None
        };

        Ok(QuorumStatus {
            proposal_hash,
            signatures_received: signed,
            signatures_required: self.quorum_threshold,
            quorum_reached,
            certificate,
            validator_states: self
                .committee
                .iter()
                .map(|pk| (*pk, state.voter_set.contains(pk)))
                .collect(),
        })
    }

    /// Get the current status of a proposal.
    pub fn get_status_sync(&self, proposal_hash: [u8; 32]) -> QuorumStatus {
        let proposals = self.proposals.lock().unwrap();
        match proposals.get(&proposal_hash) {
            Some(state) => {
                let signed = state.votes.len() as u32;
                let quorum_reached = signed >= self.quorum_threshold;
                QuorumStatus {
                    proposal_hash,
                    signatures_received: signed,
                    signatures_required: self.quorum_threshold,
                    quorum_reached,
                    certificate: if quorum_reached {
                        Some(QuorumCertificate {
                            proposal_hash,
                            signatures: state.votes.clone(),
                            threshold: self.quorum_threshold,
                        })
                    } else {
                        None
                    },
                    validator_states: self
                        .committee
                        .iter()
                        .map(|pk| (*pk, state.voter_set.contains(pk)))
                        .collect(),
                }
            }
            None => QuorumStatus {
                proposal_hash,
                signatures_received: 0,
                signatures_required: self.quorum_threshold,
                quorum_reached: false,
                certificate: None,
                validator_states: self.committee.iter().map(|pk| (*pk, false)).collect(),
            },
        }
    }
}

/// Status of a quorum for a proposal.
#[derive(Debug, Clone)]
pub struct QuorumStatus {
    pub proposal_hash: [u8; 32],
    pub signatures_received: u32,
    pub signatures_required: u32,
    pub quorum_reached: bool,
    pub certificate: Option<QuorumCertificate>,
    pub validator_states: Vec<([u8; 32], bool)>,
}

#[tonic::async_trait]
impl CollectorService for CollectorNode {
    async fn submit_vote(
        &self,
        request: Request<VoteRequest>,
    ) -> Result<Response<QuorumStatusResponse>, Status> {
        let req = request.into_inner();

        let mut proposal_hash = [0u8; 32];
        if req.proposal_hash.len() == 32 {
            proposal_hash.copy_from_slice(&req.proposal_hash);
        } else {
            return Err(Status::invalid_argument("Invalid proposal hash length"));
        }

        let mut validator_pubkey = [0u8; 32];
        if req.validator_pubkey.len() == 32 {
            validator_pubkey.copy_from_slice(&req.validator_pubkey);
        } else {
            return Err(Status::invalid_argument("Invalid pubkey length"));
        }

        match self.submit_vote_sync(proposal_hash, validator_pubkey, req.signature) {
            Ok(status) => Ok(Response::new(status_to_proto(status))),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn get_quorum_status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<QuorumStatusResponse>, Status> {
        let req = request.into_inner();

        let mut proposal_hash = [0u8; 32];
        if req.proposal_hash.len() == 32 {
            proposal_hash.copy_from_slice(&req.proposal_hash);
        } else {
            return Err(Status::invalid_argument("Invalid proposal hash length"));
        }

        let status = self.get_status_sync(proposal_hash);
        Ok(Response::new(status_to_proto(status)))
    }
}

fn status_to_proto(status: QuorumStatus) -> QuorumStatusResponse {
    QuorumStatusResponse {
        proposal_hash: status.proposal_hash.to_vec(),
        signatures_received: status.signatures_received,
        signatures_required: status.signatures_required,
        quorum_reached: status.quorum_reached,
        certificate: status.certificate.map(|qc| QuorumCertificateProto {
            proposal_hash: qc.proposal_hash.to_vec(),
            signatures: qc
                .signatures
                .iter()
                .map(|v| ValidatorSignature {
                    pubkey: v.validator_pubkey.to_vec(),
                    signature: v.signature.clone(),
                })
                .collect(),
            threshold: qc.threshold,
        }),
        validators: status
            .validator_states
            .iter()
            .map(|(pk, signed)| ValidatorStatus {
                pubkey: pk.to_vec(),
                has_signed: *signed,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    fn setup_committee(n: usize) -> (Vec<SigningKey>, Vec<[u8; 32]>) {
        let keys: Vec<SigningKey> = (0..n).map(|_| SigningKey::generate(&mut OsRng)).collect();
        let pubkeys: Vec<[u8; 32]> = keys.iter().map(|k| k.verifying_key().to_bytes()).collect();
        (keys, pubkeys)
    }

    #[test]
    fn test_quorum_3_of_5() {
        // N=5, f=1, threshold=2f+1=3
        let (keys, pubkeys) = setup_committee(5);
        let collector = CollectorNode::new(pubkeys.clone(), 3);

        let proposal_hash = [42u8; 32];

        // Submit 2 votes — not enough
        for i in 0..2 {
            let sig = keys[i].sign(&proposal_hash).to_bytes().to_vec();
            let status = collector
                .submit_vote_sync(proposal_hash, pubkeys[i], sig)
                .unwrap();
            assert!(!status.quorum_reached);
            assert_eq!(status.signatures_received, (i + 1) as u32);
        }

        // Submit 3rd vote — quorum!
        let sig = keys[2].sign(&proposal_hash).to_bytes().to_vec();
        let status = collector
            .submit_vote_sync(proposal_hash, pubkeys[2], sig)
            .unwrap();
        assert!(status.quorum_reached);
        assert_eq!(status.signatures_received, 3);
        assert!(status.certificate.is_some());

        let qc = status.certificate.unwrap();
        assert_eq!(qc.signatures.len(), 3);
        assert!(qc.is_complete());
    }

    #[test]
    fn test_reject_duplicate_vote() {
        let (keys, pubkeys) = setup_committee(3);
        let collector = CollectorNode::new(pubkeys.clone(), 2);

        let proposal_hash = [42u8; 32];
        let sig = keys[0].sign(&proposal_hash).to_bytes().to_vec();

        // First vote succeeds
        assert!(collector
            .submit_vote_sync(proposal_hash, pubkeys[0], sig.clone())
            .is_ok());

        // Duplicate vote fails
        assert!(collector
            .submit_vote_sync(proposal_hash, pubkeys[0], sig)
            .is_err());
    }

    #[test]
    fn test_reject_unknown_validator() {
        let (_keys, pubkeys) = setup_committee(3);
        let collector = CollectorNode::new(pubkeys, 2);

        let rogue_key = SigningKey::generate(&mut OsRng);
        let proposal_hash = [42u8; 32];
        let sig = rogue_key.sign(&proposal_hash).to_bytes().to_vec();

        let result = collector.submit_vote_sync(
            proposal_hash,
            rogue_key.verifying_key().to_bytes(),
            sig,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown validator"));
    }

    #[test]
    fn test_reject_invalid_signature() {
        let (keys, pubkeys) = setup_committee(3);
        let collector = CollectorNode::new(pubkeys.clone(), 2);

        let proposal_hash = [42u8; 32];
        let bad_sig = vec![0u8; 64]; // Invalid signature

        let result = collector.submit_vote_sync(proposal_hash, pubkeys[0], bad_sig);
        assert!(result.is_err());
    }
}
