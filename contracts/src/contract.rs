use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};
use std::collections::HashSet;

use crate::error::ContractError;
use crate::msg::*;
use crate::state::*;

// ─── Instantiate ───────────────────────────────────────────

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let n = msg.validators.len() as u32;
    let f = (n - 1) / 3;
    let threshold = msg.quorum_threshold.unwrap_or(2 * f + 1);

    let committee = Committee {
        validators: msg.validators,
        quorum_threshold: threshold,
    };

    COMMITTEE.save(deps.storage, &committee)?;
    OBLIGATION_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("validators", n.to_string())
        .add_attribute("quorum_threshold", threshold.to_string()))
}

// ─── Execute ───────────────────────────────────────────────

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SubmitObligation {
            encrypted_data,
            nonce,
        } => execute_submit_obligation(deps, info, encrypted_data, nonce),
        ExecuteMsg::DepositInjection {} => execute_deposit_injection(deps, info),
        ExecuteMsg::SettleWithQuorum {
            proposal_hash,
            net_flows,
            signatures,
            injection_used,
        } => execute_settle_with_quorum(
            deps,
            env,
            proposal_hash,
            net_flows,
            signatures,
            injection_used,
        ),
        ExecuteMsg::WithdrawInjection { amount } => {
            execute_withdraw_injection(deps, env, info, amount)
        }
    }
}

fn execute_submit_obligation(
    deps: DepsMut,
    info: MessageInfo,
    encrypted_data: String,
    nonce: String,
) -> Result<Response, ContractError> {
    let mut count = OBLIGATION_COUNT.load(deps.storage)?;
    count += 1;

    let obligation = StoredObligation {
        id: count,
        encrypted_data,
        nonce,
        submitter: info.sender,
    };

    OBLIGATIONS.save(deps.storage, count, &obligation)?;
    OBLIGATION_COUNT.save(deps.storage, &count)?;

    Ok(Response::new()
        .add_attribute("action", "submit_obligation")
        .add_attribute("obligation_id", count.to_string()))
}

fn execute_deposit_injection(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Accept the first native coin sent
    let deposit = info
        .funds
        .first()
        .ok_or(ContractError::NoFundsDeposited)?;

    let current = BALANCES
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(Uint128::zero());

    BALANCES.save(deps.storage, &info.sender, &(current + deposit.amount))?;

    Ok(Response::new()
        .add_attribute("action", "deposit_injection")
        .add_attribute("depositor", info.sender.to_string())
        .add_attribute("amount", deposit.amount.to_string()))
}

fn execute_settle_with_quorum(
    deps: DepsMut,
    _env: Env,
    proposal_hash: String,
    net_flows: Vec<NetFlowEntry>,
    signatures: Vec<ValidatorSignatureMsg>,
    injection_used: Uint128,
) -> Result<Response, ContractError> {
    // 1. Replay protection: check this proposal hasn't been settled already
    if SETTLED_PROPOSALS
        .may_load(deps.storage, &proposal_hash)?
        .unwrap_or(false)
    {
        return Err(ContractError::AlreadySettled {
            hash: proposal_hash,
        });
    }

    // 2. Load the committee
    let committee = COMMITTEE.load(deps.storage)?;

    // 3. Build set of known validator pubkeys
    let known_validators: HashSet<String> = committee
        .validators
        .iter()
        .map(|v| v.pubkey.clone())
        .collect();

    // 4. Verify we have enough signatures (>= 2f+1)
    if (signatures.len() as u32) < committee.quorum_threshold {
        return Err(ContractError::InsufficientQuorum {
            received: signatures.len() as u32,
            required: committee.quorum_threshold,
        });
    }

    // 5. Verify each signature is from a known validator and is valid
    let proposal_hash_bytes = hex::decode(&proposal_hash)
        .map_err(|e| ContractError::InvalidHex(e.to_string()))?;

    let mut seen_validators: HashSet<String> = HashSet::new();

    for sig in &signatures {
        // Check validator is in committee
        if !known_validators.contains(&sig.pubkey) {
            return Err(ContractError::UnknownValidator {
                pubkey: sig.pubkey.clone(),
            });
        }

        // Check no duplicate signers
        if seen_validators.contains(&sig.pubkey) {
            return Err(ContractError::DuplicateSignature {
                pubkey: sig.pubkey.clone(),
            });
        }
        seen_validators.insert(sig.pubkey.clone());

        // Verify Ed25519 signature
        let pubkey_bytes = hex::decode(&sig.pubkey)
            .map_err(|e| ContractError::InvalidHex(e.to_string()))?;
        let sig_bytes = hex::decode(&sig.signature)
            .map_err(|e| ContractError::InvalidHex(e.to_string()))?;

        verify_ed25519_signature(&pubkey_bytes, &proposal_hash_bytes, &sig_bytes)
            .map_err(|_| ContractError::InvalidSignature {
                pubkey: sig.pubkey.clone(),
            })?;
    }

    // 6. Execute net flows atomically — update debt states
    for flow in &net_flows {
        let current = DEBT_STATES
            .may_load(deps.storage, (&flow.debtor, &flow.creditor))?
            .unwrap_or(Uint128::zero());

        let new_amount = current.saturating_sub(flow.amount);
        DEBT_STATES.save(deps.storage, (&flow.debtor, &flow.creditor), &new_amount)?;
    }

    // 7. Mark proposal as settled (replay protection)
    SETTLED_PROPOSALS.save(deps.storage, &proposal_hash, &true)?;

    // 8. Calculate total cleared
    let total_cleared: Uint128 = net_flows.iter().map(|f| f.amount).sum();

    Ok(Response::new()
        .add_attribute("action", "settle_with_quorum")
        .add_attribute("proposal_hash", &proposal_hash)
        .add_attribute("total_cleared", total_cleared.to_string())
        .add_attribute("injection_used", injection_used.to_string())
        .add_attribute("signatures_verified", signatures.len().to_string()))
}

fn execute_withdraw_injection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let current = BALANCES
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(Uint128::zero());

    if current < amount {
        return Err(ContractError::InsufficientBalance {
            available: current.to_string(),
            required: amount.to_string(),
        });
    }

    BALANCES.save(deps.storage, &info.sender, &(current - amount))?;

    // Send the tokens back
    let bank_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: "ujuno".to_string(),
            amount,
        }],
    };

    Ok(Response::new()
        .add_message(bank_msg)
        .add_attribute("action", "withdraw_injection")
        .add_attribute("amount", amount.to_string()))
}

/// Verify an Ed25519 signature.
fn verify_ed25519_signature(
    pubkey: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<(), ContractError> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let pubkey_array: [u8; 32] = pubkey
        .try_into()
        .map_err(|_| ContractError::InvalidSignature {
            pubkey: hex::encode(pubkey),
        })?;

    let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
        .map_err(|_| ContractError::InvalidSignature {
            pubkey: hex::encode(pubkey),
        })?;

    let sig = Signature::from_slice(signature).map_err(|_| ContractError::InvalidSignature {
        pubkey: hex::encode(pubkey),
    })?;

    verifying_key
        .verify(message, &sig)
        .map_err(|_| ContractError::InvalidSignature {
            pubkey: hex::encode(pubkey),
        })
}

// ─── Query ──────────────────────────────────────────────────

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDebtState { debtor, creditor } => {
            let amount = DEBT_STATES
                .may_load(deps.storage, (&debtor, &creditor))?
                .unwrap_or(Uint128::zero());
            to_json_binary(&DebtStateResponse {
                debtor,
                creditor,
                amount,
            })
        }
        QueryMsg::GetCommittee {} => {
            let committee = COMMITTEE.load(deps.storage)?;
            to_json_binary(&CommitteeResponse {
                validators: committee.validators,
                quorum_threshold: committee.quorum_threshold,
                total_validators: 0, // Will be set below
            })
        }
        QueryMsg::GetObligationCount {} => {
            let count = OBLIGATION_COUNT.load(deps.storage)?;
            to_json_binary(&ObligationCountResponse { count })
        }
        QueryMsg::GetProposalStatus { proposal_hash } => {
            let settled = SETTLED_PROPOSALS
                .may_load(deps.storage, &proposal_hash)?
                .unwrap_or(false);
            to_json_binary(&ProposalStatusResponse { settled })
        }
        QueryMsg::GetBalance { address } => {
            let addr = deps.api.addr_validate(&address)?;
            let amount = BALANCES
                .may_load(deps.storage, &addr)?
                .unwrap_or(Uint128::zero());
            to_json_binary(&BalanceResponse {
                address,
                amount,
            })
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{coins, Addr};
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    fn setup_contract(deps: DepsMut, validators: Vec<(String, SigningKey)>) -> Vec<SigningKey> {
        let validator_infos: Vec<ValidatorInfo> = validators
            .iter()
            .enumerate()
            .map(|(i, (pk, _))| ValidatorInfo {
                pubkey: pk.clone(),
                name: format!("Validator {}", i),
            })
            .collect();

        let msg = InstantiateMsg {
            validators: validator_infos,
            quorum_threshold: None,
        };

        let info = message_info(&Addr::unchecked("admin"), &[]);
        instantiate(deps, mock_env(), info, msg).unwrap();

        validators.into_iter().map(|(_, sk)| sk).collect()
    }

    fn generate_validators(n: usize) -> Vec<(String, SigningKey)> {
        (0..n)
            .map(|_| {
                let sk = SigningKey::generate(&mut OsRng);
                let pk_hex = hex::encode(sk.verifying_key().to_bytes());
                (pk_hex, sk)
            })
            .collect()
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let validators = generate_validators(5);
        setup_contract(deps.as_mut(), validators);

        let committee = COMMITTEE.load(&deps.storage).unwrap();
        assert_eq!(committee.validators.len(), 5);
        // N=5, f=1, threshold=3
        assert_eq!(committee.quorum_threshold, 3);
    }

    #[test]
    fn test_submit_obligation() {
        let mut deps = mock_dependencies();
        let validators = generate_validators(3);
        setup_contract(deps.as_mut(), validators);

        let info = message_info(&Addr::unchecked("business_a"), &[]);
        let msg = ExecuteMsg::SubmitObligation {
            encrypted_data: "deadbeef".into(),
            nonce: "aabbccdd".into(),
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes[0].value, "submit_obligation");
        assert_eq!(res.attributes[1].value, "1");
    }

    #[test]
    fn test_deposit_injection() {
        let mut deps = mock_dependencies();
        let validators = generate_validators(3);
        setup_contract(deps.as_mut(), validators);

        let info = message_info(&Addr::unchecked("depositor"), &coins(1000, "ujuno"));
        let msg = ExecuteMsg::DepositInjection {};

        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes[0].value, "deposit_injection");

        // Check balance
        let balance = BALANCES
            .load(&deps.storage, &Addr::unchecked("depositor"))
            .unwrap();
        assert_eq!(balance, Uint128::new(1000));
    }

    #[test]
    fn test_settle_with_valid_quorum() {
        let mut deps = mock_dependencies();
        let validators = generate_validators(5);
        let signing_keys = setup_contract(deps.as_mut(), validators.clone());

        // Create a proposal hash
        let proposal_hash_bytes = [42u8; 32];
        let proposal_hash_hex = hex::encode(proposal_hash_bytes);

        // Sign with 3/5 validators (quorum = 3)
        let sigs: Vec<ValidatorSignatureMsg> = signing_keys[0..3]
            .iter()
            .map(|sk| {
                let sig = sk.sign(&proposal_hash_bytes);
                ValidatorSignatureMsg {
                    pubkey: hex::encode(sk.verifying_key().to_bytes()),
                    signature: hex::encode(sig.to_bytes()),
                }
            })
            .collect();

        let msg = ExecuteMsg::SettleWithQuorum {
            proposal_hash: proposal_hash_hex.clone(),
            net_flows: vec![NetFlowEntry {
                debtor: "A".into(),
                creditor: "B".into(),
                amount: Uint128::new(100),
            }],
            signatures: sigs,
            injection_used: Uint128::zero(),
        };

        let info = message_info(&Addr::unchecked("anyone"), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes[0].value, "settle_with_quorum");

        // Verify replay protection
        let settled = SETTLED_PROPOSALS
            .load(&deps.storage, &proposal_hash_hex)
            .unwrap();
        assert!(settled);
    }

    #[test]
    fn test_reject_insufficient_quorum() {
        let mut deps = mock_dependencies();
        let validators = generate_validators(5);
        let signing_keys = setup_contract(deps.as_mut(), validators);

        let proposal_hash_bytes = [42u8; 32];
        let proposal_hash_hex = hex::encode(proposal_hash_bytes);

        // Only 2/5 signatures (need 3)
        let sigs: Vec<ValidatorSignatureMsg> = signing_keys[0..2]
            .iter()
            .map(|sk| {
                let sig = sk.sign(&proposal_hash_bytes);
                ValidatorSignatureMsg {
                    pubkey: hex::encode(sk.verifying_key().to_bytes()),
                    signature: hex::encode(sig.to_bytes()),
                }
            })
            .collect();

        let msg = ExecuteMsg::SettleWithQuorum {
            proposal_hash: proposal_hash_hex,
            net_flows: vec![],
            signatures: sigs,
            injection_used: Uint128::zero(),
        };

        let info = message_info(&Addr::unchecked("anyone"), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        match err {
            ContractError::InsufficientQuorum {
                received: 2,
                required: 3,
            } => {}
            _ => panic!("Expected InsufficientQuorum, got {:?}", err),
        }
    }

    #[test]
    fn test_reject_replay() {
        let mut deps = mock_dependencies();
        let validators = generate_validators(5);
        let signing_keys = setup_contract(deps.as_mut(), validators);

        let proposal_hash_bytes = [42u8; 32];
        let proposal_hash_hex = hex::encode(proposal_hash_bytes);

        let sigs: Vec<ValidatorSignatureMsg> = signing_keys[0..3]
            .iter()
            .map(|sk| {
                let sig = sk.sign(&proposal_hash_bytes);
                ValidatorSignatureMsg {
                    pubkey: hex::encode(sk.verifying_key().to_bytes()),
                    signature: hex::encode(sig.to_bytes()),
                }
            })
            .collect();

        let msg = ExecuteMsg::SettleWithQuorum {
            proposal_hash: proposal_hash_hex.clone(),
            net_flows: vec![],
            signatures: sigs.clone(),
            injection_used: Uint128::zero(),
        };

        let info = message_info(&Addr::unchecked("anyone"), &[]);

        // First settlement succeeds
        execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        // Replay attempt fails
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        match err {
            ContractError::AlreadySettled { .. } => {}
            _ => panic!("Expected AlreadySettled, got {:?}", err),
        }
    }

    #[test]
    fn test_reject_duplicate_signer() {
        let mut deps = mock_dependencies();
        let validators = generate_validators(5);
        let signing_keys = setup_contract(deps.as_mut(), validators);

        let proposal_hash_bytes = [42u8; 32];
        let proposal_hash_hex = hex::encode(proposal_hash_bytes);

        // Submit the same signature twice
        let sig = signing_keys[0].sign(&proposal_hash_bytes);
        let sigs = vec![
            ValidatorSignatureMsg {
                pubkey: hex::encode(signing_keys[0].verifying_key().to_bytes()),
                signature: hex::encode(sig.to_bytes()),
            },
            ValidatorSignatureMsg {
                pubkey: hex::encode(signing_keys[0].verifying_key().to_bytes()),
                signature: hex::encode(sig.to_bytes()),
            },
            ValidatorSignatureMsg {
                pubkey: hex::encode(signing_keys[1].verifying_key().to_bytes()),
                signature: hex::encode(signing_keys[1].sign(&proposal_hash_bytes).to_bytes()),
            },
        ];

        let msg = ExecuteMsg::SettleWithQuorum {
            proposal_hash: proposal_hash_hex,
            net_flows: vec![],
            signatures: sigs,
            injection_used: Uint128::zero(),
        };

        let info = message_info(&Addr::unchecked("anyone"), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        match err {
            ContractError::DuplicateSignature { .. } => {}
            _ => panic!("Expected DuplicateSignature, got {:?}", err),
        }
    }
}
