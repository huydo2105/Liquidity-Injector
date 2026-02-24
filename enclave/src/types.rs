use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

/// A unique identifier for a participant (business) in the debt network.
/// In production, this would be a Cosmos address.
pub type ParticipantId = String;

/// An obligation (debt edge) in the graph: debtor owes creditor some amount.
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Obligation {
    pub id: u64,
    pub debtor: ParticipantId,
    pub creditor: ParticipantId,
    pub amount: u128,
}

/// An encrypted obligation as stored on-chain (encrypted to TEE's xPub).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedObligation {
    pub id: u64,
    /// AES-256-GCM ciphertext
    pub ciphertext: Vec<u8>,
    /// AES-GCM nonce (96-bit)
    pub nonce: Vec<u8>,
}

/// A cycle detected in the debt graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cycle {
    /// Ordered list of participants forming the cycle.
    /// e.g., [A, B, C] means A→B→C→A
    pub participants: Vec<ParticipantId>,
    /// The edges (obligations) that form this cycle.
    pub edges: Vec<Obligation>,
    /// The maximum clearable amount (bottleneck = min edge in cycle).
    pub clearable_amount: u128,
}

/// A single flow element: a transfer from debtor to creditor.
/// Must satisfy: 0 < amount <= original obligation amount.
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct FlowElement {
    pub debtor: ParticipantId,
    pub creditor: ParticipantId,
    pub amount: u128,
}

/// The complete flow solution F ⊆ G produced by MTCS.
/// Must satisfy the balanced flow property: for each node, flow_in == flow_out.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlowSolution {
    pub flows: Vec<FlowElement>,
    /// Total debt cleared by this solution.
    pub total_cleared: u128,
    /// Amount of injection liquidity consumed.
    pub injection_used: u128,
}

/// A set-off notice encrypted to a specific user's public key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetOffNotice {
    pub debtor: ParticipantId,
    pub creditor: ParticipantId,
    pub amount: u128,
}

/// Simulated SGX attestation quote (mirrors DCAP structure).
/// In production, this comes from `quartz-enclave-core`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationQuote {
    /// Hash of the enclave binary (MRENCLAVE).
    pub mrenclave: [u8; 32],
    /// Hash of the enclave signer (MRSIGNER).
    pub mrsigner: [u8; 32],
    /// SHA-256 hash of the data being attested (the proposal).
    #[serde(with = "BigArray")]
    pub report_data: [u8; 64],
    /// Timestamp of attestation.
    pub timestamp: u64,
}

/// The complete settlement proposal produced by the Proposer TEE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementProposal {
    /// Unique identifier for this proposal.
    pub proposal_id: [u8; 32],
    /// The flow solution F.
    pub flow: FlowSolution,
    /// Attestation from the proposing TEE.
    pub attestation: AttestationQuote,
    /// SHA-256 hash of the serialized flow solution (used for signing).
    pub proposal_hash: [u8; 32],
}

/// A validator's vote on a settlement proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorVote {
    pub validator_pubkey: [u8; 32],
    pub proposal_hash: [u8; 32],
    pub signature: Vec<u8>,
}

/// The Quorum Certificate: aggregated 2f+1 validator signatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumCertificate {
    pub proposal_hash: [u8; 32],
    pub signatures: Vec<ValidatorVote>,
    pub threshold: u32,
}

/// Configuration for the TEE committee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeConfig {
    /// Total number of validators (N = 3f + 1).
    pub total_validators: u32,
    /// Maximum faulty nodes tolerated.
    pub max_faulty: u32,
    /// Quorum threshold (2f + 1).
    pub quorum_threshold: u32,
    /// Public keys of all validators.
    pub validator_pubkeys: Vec<[u8; 32]>,
}

impl CommitteeConfig {
    /// Create a new committee config with N validators.
    /// N must satisfy N = 3f + 1 for some non-negative integer f.
    pub fn new(validator_pubkeys: Vec<[u8; 32]>) -> Self {
        let n = validator_pubkeys.len() as u32;
        // f = (N - 1) / 3
        let f = (n - 1) / 3;
        Self {
            total_validators: n,
            max_faulty: f,
            quorum_threshold: 2 * f + 1,
            validator_pubkeys,
        }
    }
}

impl QuorumCertificate {
    /// Check if the quorum threshold is met.
    pub fn is_complete(&self) -> bool {
        self.signatures.len() as u32 >= self.threshold
    }
}
