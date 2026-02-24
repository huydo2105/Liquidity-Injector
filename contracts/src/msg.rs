use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

/// Information about a TEE validator node.
#[cw_serde]
pub struct ValidatorInfo {
    /// Ed25519 public key (32 bytes, hex-encoded)
    pub pubkey: String,
    /// Human-readable name
    pub name: String,
}

/// A signed vote from a validator.
#[cw_serde]
pub struct ValidatorSignatureMsg {
    /// Ed25519 public key (hex-encoded)
    pub pubkey: String,
    /// Ed25519 signature over the proposal hash (hex-encoded)
    pub signature: String,
}

/// A net flow entry: how much debt is cleared between two parties.
#[cw_serde]
pub struct NetFlowEntry {
    pub debtor: String,
    pub creditor: String,
    pub amount: Uint128,
}

/// Instantiate message: set up the validator committee.
#[cw_serde]
pub struct InstantiateMsg {
    /// The TEE validator committee
    pub validators: Vec<ValidatorInfo>,
    /// Quorum threshold (2f + 1). If not set, computed from N = 3f + 1.
    pub quorum_threshold: Option<u32>,
}

/// Execute messages for the settlement contract.
#[cw_serde]
pub enum ExecuteMsg {
    /// Submit an encrypted obligation to the on-chain graph Ĝ.
    SubmitObligation {
        /// The encrypted obligation data (hex-encoded ciphertext)
        encrypted_data: String,
        /// The AES-GCM nonce (hex-encoded)
        nonce: String,
    },

    /// Deposit injection liquidity (sends native tokens with this message).
    DepositInjection {},

    /// Execute settlement with a verified Quorum Certificate.
    /// Only succeeds if >= 2f+1 unique valid signatures from registered validators.
    SettleWithQuorum {
        /// SHA-256 hash of the flow solution (hex-encoded, 32 bytes)
        proposal_hash: String,
        /// The net settlement flows to execute
        net_flows: Vec<NetFlowEntry>,
        /// Validator signatures forming the Quorum Certificate
        signatures: Vec<ValidatorSignatureMsg>,
        /// Total injection amount consumed
        injection_used: Uint128,
    },

    /// Withdraw remaining injection liquidity (only depositor).
    WithdrawInjection {
        amount: Uint128,
    },
}

/// Query messages.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the debt state between two parties.
    #[returns(DebtStateResponse)]
    GetDebtState {
        debtor: String,
        creditor: String,
    },

    /// Get the current validator committee and quorum status.
    #[returns(CommitteeResponse)]
    GetCommittee {},

    /// Get all stored encrypted obligations count.
    #[returns(ObligationCountResponse)]
    GetObligationCount {},

    /// Check if a proposal has already been settled (replay protection).
    #[returns(ProposalStatusResponse)]
    GetProposalStatus {
        proposal_hash: String,
    },

    /// Get a depositor's injection balance.
    #[returns(BalanceResponse)]
    GetBalance {
        address: String,
    },
}

// ─── Query Responses ──────────────────────────────────────

#[cw_serde]
pub struct DebtStateResponse {
    pub debtor: String,
    pub creditor: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct CommitteeResponse {
    pub validators: Vec<ValidatorInfo>,
    pub quorum_threshold: u32,
    pub total_validators: u32,
}

#[cw_serde]
pub struct ObligationCountResponse {
    pub count: u64,
}

#[cw_serde]
pub struct ProposalStatusResponse {
    pub settled: bool,
}

#[cw_serde]
pub struct BalanceResponse {
    pub address: String,
    pub amount: Uint128,
}
