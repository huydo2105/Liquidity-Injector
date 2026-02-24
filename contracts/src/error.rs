use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Insufficient quorum: got {received} signatures, need {required}")]
    InsufficientQuorum { received: u32, required: u32 },

    #[error("Invalid signature from validator {pubkey}")]
    InvalidSignature { pubkey: String },

    #[error("Unknown validator: {pubkey}")]
    UnknownValidator { pubkey: String },

    #[error("Proposal already settled: {hash}")]
    AlreadySettled { hash: String },

    #[error("Duplicate signature from validator: {pubkey}")]
    DuplicateSignature { pubkey: String },

    #[error("Insufficient injection balance: have {available}, need {required}")]
    InsufficientBalance {
        available: String,
        required: String,
    },

    #[error("No funds sent with deposit")]
    NoFundsDeposited,

    #[error("Invalid hex encoding: {0}")]
    InvalidHex(String),

    #[error("Unauthorized")]
    Unauthorized,
}
