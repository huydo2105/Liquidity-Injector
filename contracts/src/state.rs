use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

use crate::msg::ValidatorInfo;

/// The validator committee configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Committee {
    pub validators: Vec<ValidatorInfo>,
    pub quorum_threshold: u32,
}

/// An encrypted obligation stored on-chain.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredObligation {
    pub id: u64,
    pub encrypted_data: String,
    pub nonce: String,
    pub submitter: Addr,
}

// ─── Storage Items ─────────────────────────────────────────

/// The TEE validator committee.
pub const COMMITTEE: Item<Committee> = Item::new("committee");

/// Counter for obligation IDs.
pub const OBLIGATION_COUNT: Item<u64> = Item::new("obligation_count");

/// Encrypted obligations: id → StoredObligation.
pub const OBLIGATIONS: Map<u64, StoredObligation> = Map::new("obligations");

/// Injection liquidity balances: depositor address → amount.
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balances");

/// Debt states: (debtor, creditor) → amount.
pub const DEBT_STATES: Map<(&str, &str), Uint128> = Map::new("debt_states");

/// Settled proposals (replay protection): proposal_hash → true.
pub const SETTLED_PROPOSALS: Map<&str, bool> = Map::new("settled_proposals");
