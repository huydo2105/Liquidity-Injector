# Agent Guideline: Resilient Cycles Liquidity Injector (R-CLI)

## 1. Project Vision
A decentralized clearing house that combines hardware privacy (TEEs) with distributed consensus (Quorum) to solve liquidity gridlock without single points of failure.

## 2. Technical Stack & PhD Alignment
- **Confidentiality:** Quartz (Informal Systems) / Intel SGX.
- **Resilience:** BFT Quorum logic (n = 3f + 1).
- **Finality:** Threshold signature aggregation for on-chain verification.
- **Frontend:** Next.js with wagmi/viem for transaction lifecycle management.

## 3. Implementation Priorities

### Phase A: The Distributed Enclave (Rust)
- **Proposer Role:** Finds cycles and broadcasts a `ProposedSettlement`.
- **Validator Role:** Verifies the Proposer's TEE Attestation Quote and recalculates the cycle clearing to ensure honesty.
- **Output:** A signed attestation of the settlement state transition.

### Phase B: Quorum Aggregation
- Implement a simple aggregation layer to collect signatures.
- **Goal:** Reach the $2f+1$ threshold required for Byzantine Fault Tolerance.

### Phase C: On-Chain Finality (Solidity)
- Use a `QuorumVerifier` contract.
- Validate that the signatures come from the authorized TEE committee.
- **Atomic Execution:** Only execute the clearing if the full quorum is satisfied.

### Phase D: UX & Visualization (Next.js)
- Build a "Consensus Monitor" showing the TEE committee reaching agreement in real-time.
- [cite_start]Use the `Thirdweb SDK` or `wagmi` for robust wallet interactions[cite: 47].

## 4. Guardrails & Performance
- **Minimize On-Chain Load:** Only the final Quorum Certificate and the Net Settlement result hit the chain.
- **Privacy:** Individual debt amounts must never leave the TEE cluster in plaintext.
- **Scalability:** Design the graph traversal to be gas-efficient and performant within the enclave.