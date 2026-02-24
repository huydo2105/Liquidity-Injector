# Cycles Liquidity Injector

> **Privacy-Preserving Debt Clearing using TEEs and BFT Quorum Consensus**

A decentralized clearing house that discovers and settles circular debt obligations privately inside Trusted Execution Environments (TEEs), producing a quorum-attested settlement that executes atomically on-chain via CosmWasm.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![CosmWasm](https://img.shields.io/badge/CosmWasm-2.2-blue)
![Next.js](https://img.shields.io/badge/Next.js-16-black)
![License](https://img.shields.io/badge/License-MIT-green)

---

## Overview

Companies often owe each other in circles: **A owes B**, **B owes C**, **C owes A**. These circular debts can be cancelled without anyone moving money. The **Cycles Protocol** automates this process while keeping individual debt amounts **private**.

### How It Works

```
┌─────────────┐    encrypted     ┌───────────────────┐
│  Businesses  │───obligations──▶│   TEE Enclave      │
│  (on-chain)  │                 │  • Decrypt debts   │
└──────────────┘                 │  • Find cycles     │
                                 │  • Compute MTCS    │
                                 │  • Attest result   │
                                 └────────┬───────────┘
                                          │ proposal
                                          ▼
                                 ┌───────────────────┐
                                 │  Validator TEEs    │
                                 │  (2f+1 quorum)     │
                                 │  • Verify math     │
                                 │  • Sign proposal   │
                                 └────────┬───────────┘
                                          │ QC (Quorum Certificate)
                                          ▼
                                 ┌───────────────────┐
                                 │  CosmWasm Contract │
                                 │  (Juno Testnet)    │
                                 │  • Verify 2f+1 sigs│
                                 │  • Execute settle  │
                                 └───────────────────┘
```

### Key Innovation: Injection Liquidity

A small amount of **external liquidity** injected into the system can catalyze the clearing of **much larger** circular debt chains. For example, $5k of injection can clear $100k of debt — a **20x multiplier effect**.

---

## Project Structure

```
Liquidity Injector/
├── enclave/              # TEE enclave logic (Rust)
│   └── src/
│       ├── types.rs      # Core data structures
│       ├── crypto.rs     # AES-256-GCM, SHA-256, attestation
│       ├── graph.rs      # Johnson's cycle detection, MTCS flow solver
│       └── settlement.rs # End-to-end TEE settlement pipeline
│
├── consensus/            # BFT quorum layer (Rust + gRPC)
│   ├── proto/
│   │   └── quorum.proto  # Validator & Collector service definitions
│   └── src/
│       ├── validator.rs  # TEE validator node (verify + sign)
│       ├── collector.rs  # Vote aggregation → Quorum Certificate
│       └── main.rs       # CLI entry point
│
├── contracts/            # CosmWasm settlement contract (Rust)
│   └── src/
│       ├── msg.rs        # Instantiate / Execute / Query messages
│       ├── state.rs      # On-chain storage layout
│       ├── error.rs      # Custom error types
│       └── contract.rs   # Quorum verification + atomic settlement
│
├── frontend/             # Dashboard (Next.js + TypeScript)
│   └── app/
│       ├── components/
│       │   ├── CycleVisualizer.tsx  # D3 force-directed debt graph
│       │   ├── QuorumTracker.tsx    # Real-time signature progress
│       │   ├── SettlementPanel.tsx  # Settlement stats & execution
│       │   └── ObligationSubmit.tsx # Encrypted obligation form
│       └── hooks/
│           ├── useQuorum.ts         # Quorum polling hook
│           └── useCosmWasm.ts       # Keplr + CosmJS integration
│
├── docs/
│   └── Quartz.md         # Quartz TEE framework reference
├── agent.md              # Project vision document
└── Cargo.toml            # Workspace configuration
```

---

## Technology Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Enclave** | Rust, `petgraph`, `aes-gcm` | Cycle detection, MTCS, encryption |
| **Consensus** | Rust, `tonic`, `ed25519-dalek` | BFT quorum, gRPC validator network |
| **Contract** | Rust, CosmWasm 2.2, `cw-storage-plus` | On-chain settlement with QC verification |
| **Frontend** | Next.js 16, D3.js, Framer Motion | Visualization dashboard |
| **Wallet** | Keplr, CosmJS | Juno testnet interaction |

---

## Getting Started

### Prerequisites

- **Rust** (1.75+) with `cargo`
- **Node.js** (18+) with `npm`
- **protoc** (Protocol Buffers compiler) — for the consensus crate
- **Keplr Wallet** browser extension — for frontend interaction

### Build

```bash
# Build all Rust crates
cargo build

# Build only the enclave
cargo build -p enclave

# Build the CosmWasm contract
cargo build -p contracts

# Build the consensus layer (requires protoc)
cargo build -p consensus
```

### Test

```bash
# Run all tests
cargo test

# Run enclave tests (graph, crypto, settlement)
cargo test -p enclave

# Run contract tests (quorum verification, replay protection)
cargo test -p contracts

# Run consensus tests (validator signing, collector aggregation)
cargo test -p consensus
```

### Frontend

```bash
cd frontend
npm install
npm run dev
# Open http://localhost:3000
```

---

## Architecture Deep Dive

### Phase A — Enclave Logic

The TEE enclave performs all privacy-sensitive computation:

1. **Decrypt** AES-256-GCM encrypted obligations submitted on-chain
2. **Build** a directed debt graph using `petgraph`
3. **Detect** all elementary cycles using Johnson's algorithm
4. **Compute** optimal settlement flow using simplified MTCS (min-cost max-flow)
5. **Validate** flow invariants: `F ⊆ G`, balanced flow, correct amounts
6. **Attest** the result with a simulated SGX attestation quote

### Phase B — Quorum Consensus

A committee of `N = 3f + 1` validator TEE nodes ensures correctness:

- Each validator **independently** verifies the attestation and recalculates the MTCS flow
- Valid proposals are signed with Ed25519
- The **Collector** aggregates signatures until `2f + 1` threshold is met
- A **Quorum Certificate** (QC) is produced and submitted on-chain

### Phase C — On-Chain Settlement

The CosmWasm contract on Juno testnet:

- Stores encrypted obligations and injection liquidity deposits
- Verifies Ed25519 signatures from `2f + 1` registered validators
- Rejects duplicate signers, unknown validators, and replayed proposals
- Executes atomic debt state updates when QC is valid

### Phase D — Frontend Dashboard

The Next.js dashboard visualizes the entire pipeline:

- **Cycle Visualizer**: Interactive D3 force graph showing debt cycles with animated flow particles
- **Quorum Tracker**: Real-time SVG progress ring showing validator signature collection
- **Settlement Panel**: Stats (debt cleared, injection used, multiplier effect) and one-click settlement
- **Obligation Submit**: Form to encrypt and submit obligations to the TEE

---

## Configuration

### Contract (Juno Testnet)

```typescript
// frontend/app/hooks/useCosmWasm.ts
export const CONTRACT_CONFIG = {
  contractAddress: "",        // Set after deployment
  chainId: "uni-7",           // Juno testnet
  rpcEndpoint: "https://rpc.uni.junonetwork.io",
  denom: "ujunox",
};
```

### Consensus Committee

```bash
# Start a validator node
cargo run -p consensus -- --role=validator --port=50051

# Start the collector
cargo run -p consensus -- --role=collector --port=50060
```

---

## Security Model

| Property | Mechanism |
|----------|-----------|
| **Privacy** | Debt amounts encrypted with AES-256-GCM; only cleartext inside TEE |
| **Integrity** | MTCS flow validated against protocol invariants |
| **Authenticity** | SGX attestation quotes (simulated with `--mock-sgx`) |
| **Consensus** | BFT 2f+1 quorum — tolerates up to f Byzantine validators |
| **Replay Protection** | Settled proposal hashes stored on-chain |
| **Sybil Resistance** | Validator committee registered at contract instantiation |

---

## Development Mode

This project uses `--mock-sgx` for TEE simulation. The simulated attestation generates deterministic MRENCLAVE values based on the enclave version string. For production deployment, replace with real SGX DCAP attestation via the Quartz framework.

---

## License

MIT

---

## Acknowledgments

- [Cycles Protocol](https://arxiv.org/abs/2212.05481) — Academic foundation
- [Quartz](https://github.com/informalsystems/quartz) — TEE framework for CosmWasm
- [CosmWasm](https://cosmwasm.com/) — Smart contract platform
- [Juno Network](https://junonetwork.io/) — Target deployment chain
