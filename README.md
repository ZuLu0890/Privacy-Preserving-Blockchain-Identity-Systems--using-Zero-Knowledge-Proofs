# Privacy-Preserving Blockchain Identity System using Zero-Knowledge Proofs

[![CI](https://github.com/vatal-system/Privacy-Preserving-Blockchain-Identity-Systems--using-Zero-Knowledge-Proofs/actions/workflows/ci.yml/badge.svg)](https://github.com/vatal-system/Privacy-Preserving-Blockchain-Identity-Systems--using-Zero-Knowledge-Proofs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A privacy layer for the **Stellar** network that lets users register human-readable usernames and send payments **without exposing wallet addresses on-chain**. Built with Groth16 zk-SNARKs, Pedersen commitments, and stealth addresses on [Soroban](https://soroban.stellar.org/) smart contracts.

---

## Table of Contents

- [Problem Statement](#problem-statement)
- [How It Works](#how-it-works)
- [Architecture Overview](#architecture-overview)
- [Key Features](#key-features)
- [Repository Structure](#repository-structure)
- [Tech Stack](#tech-stack)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Build Contracts](#build-contracts)
  - [Run Contract Tests](#run-contract-tests)
  - [Build Circuits](#build-circuits)
  - [Build & Run the SDK](#build--run-the-sdk)
  - [Run the Backend](#run-the-backend)
  - [Run the Frontend](#run-the-frontend)
- [Cryptographic Primitives](#cryptographic-primitives)
- [Security Properties](#security-properties)
- [Prior Art](#prior-art)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Problem Statement

Public blockchains like Stellar publish every transaction on an open ledger. While this ensures transparency and auditability, it also means that **anyone can trace the flow of funds between wallet addresses**. Once a wallet address is linked to a real-world identity — through an exchange KYC, a social media post, or a payment to a known merchant — the entire transaction history of that individual becomes public.

This project solves that problem by introducing a **privacy layer** that decouples human-readable usernames from wallet addresses using zero-knowledge cryptography. Users can send and receive payments by name, while the underlying wallet addresses remain hidden from the public ledger.

---

## How It Works

```
┌──────────────────────────────────────────────────────────────────────────┐
│                          PRIVACY-PRESERVING PAYMENT FLOW                │
│                                                                         │
│  1. REGISTER    Alice commits H(username, secret) on-chain.             │
│                 Her wallet address is NEVER stored.                      │
│                                                                         │
│  2. SEND        Bob generates a ZK proof (client-side) that he knows    │
│                 Alice's username, then sends payment to a fresh          │
│                 stealth address via PaymentRouter.                       │
│                                                                         │
│  3. VERIFY      The Groth16Verifier contract checks the proof on-chain  │
│                 using Stellar's native BN254 primitives. If valid,       │
│                 funds are released to the stealth address.               │
│                                                                         │
│  4. RECEIVE     Alice scans the chain with her private key to detect    │
│                 and claim incoming stealth payments.                     │
│                                                                         │
│  Result: No wallet address ever appears in the transaction graph.       │
│  The link between username and wallet is known only to the owner.       │
└──────────────────────────────────────────────────────────────────────────┘
```

### Step-by-step

1. **Register** — A user commits `H(username, secret)` to the on-chain `IdentityRegistry` contract. The wallet address is never stored — only a one-way cryptographic commitment.

2. **Send** — The sender generates a Groth16 proof (client-side) that they know the preimage of the recipient's commitment (i.e. the username), then routes the payment through `PaymentRouter` to a freshly derived one-time **stealth address**.

3. **Verify** — The `Groth16Verifier` contract checks the proof on-chain using Stellar's native BN254 primitives (Protocol 25 / X-Ray upgrade). If valid, funds are transferred to the stealth address.

4. **Receive** — The recipient scans the chain with their private key to detect and claim incoming stealth payments. A **view tag** optimization lets recipients skip ~99.6% of transactions without full ECDH computation.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  Client (browser / mobile)                                      │
│                                                                 │
│  sdk/src/commitment.ts  ──►  Poseidon(username, secret)         │
│  sdk/src/proof.ts       ──►  Groth16 proof (snarkjs, client)    │
│  sdk/src/stealth.ts     ──►  Stealth address derivation (ECDH)  │
│  sdk/src/client.ts      ──►  Soroban RPC calls                  │
└────────────────────────────────┬────────────────────────────────┘
                                 │ Soroban transactions
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│  Stellar / Soroban (Protocol 25+)                               │
│                                                                 │
│  IdentityRegistry                                               │
│    register(commitment, nullifier)                              │
│    is_registered(commitment) → bool                             │
│                                                                 │
│  PaymentRouter                                                  │
│    send(token, commitment, stealth_addr, proof, inputs, amount) │
│      ├─ calls IdentityRegistry.is_registered                    │
│      ├─ calls Groth16Verifier.verify                            │
│      ├─ transfers token to stealth_addr                         │
│      └─ emits PaymentSent event                                 │
│                                                                 │
│  Groth16Verifier                                                │
│    verify(proof, public_inputs) → bool                          │
│      └─ uses native BN254 + Poseidon host functions             │
└─────────────────────────────────────────────────────────────────┘
```

For a detailed Mermaid sequence diagram of the full register → send → receive flow, see [`docs/architecture.md`](docs/architecture.md).

---

## Key Features

- **Username-based payments** — Send funds to human-readable usernames instead of long wallet addresses.
- **Zero-knowledge identity** — Usernames are stored as Poseidon hash commitments; the plaintext never appears on-chain.
- **Stealth addresses** — Every payment goes to a unique one-time address, making recipient tracking impossible.
- **On-chain proof verification** — Groth16 zk-SNARK proofs are verified directly on Soroban using Stellar's native BN254 pairing support (Protocol 25).
- **Replay protection** — Nullifiers prevent double-registration and proof replay attacks.
- **Event-driven scanning** — `PaymentSent` events allow the SDK to efficiently scan for incoming stealth payments.
- **Client-side proof generation** — All ZK proofs are generated in the browser/client using snarkjs — no trusted server required.
- **Full-stack architecture** — Includes Soroban contracts, Circom circuits, TypeScript SDK, Express backend, and Next.js frontend.

---

## Repository Structure

```
contracts/                  Soroban smart contracts (Rust → Wasm)
├── identity_registry/        Username commitment registry
├── payment_router/           Stealth payment routing + event emission
├── groth16_verifier/         On-chain Groth16 proof verification
└── Cargo.toml                Workspace configuration

circuits/                   Circom ZK circuits
├── username_commitment.circom   Proves knowledge of username preimage
├── payment_proof.circom         Proves valid payment authorization
└── stealth_address.circom       Derives stealth address from recipient key

packages/
├── sdk/                    TypeScript client SDK
│   └── src/
│       ├── commitment.ts     Pedersen commitment helpers
│       ├── proof.ts          Client-side Groth16 proof generation (snarkjs)
│       ├── stealth.ts        Stealth address generation and scanning
│       ├── client.ts         Soroban contract interaction
│       └── index.ts          Public API
│
├── backend/                Express.js API server
│   └── src/
│       ├── app.ts            Express app setup
│       ├── routes/
│       │   ├── register.ts   POST /register — submit commitments
│       │   └── commitment.ts GET /commitment/:id — lookup commitments
│       └── index.ts          Server entry point
│
└── frontend/               Next.js web application
    └── src/app/
        ├── page.tsx          Register page — username registration
        ├── send/page.tsx     Send page — private payment form
        └── layout.tsx        App layout

docs/
├── architecture.md         System design, data flows, and cryptographic choices
├── circuits.md             Circuit design and trusted setup guide
└── CONTRIBUTING.md         How to contribute

.github/workflows/
└── ci.yml                  GitHub Actions CI (SDK, backend, frontend, contracts)
```

---

## Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| Smart contracts | **Rust** / **Soroban SDK** | On-chain logic compiled to Wasm |
| ZK circuits | **Circom** / **snarkjs** | Groth16 proof generation and verification |
| Curve | **BN254** (alt-bn128) | Pairing-friendly curve for zk-SNARKs |
| Hash function | **Poseidon** | ZK-friendly hash for commitments |
| Stealth addresses | **ECDH** (BabyJubJub) | One-time address derivation |
| Client SDK | **TypeScript** | Commitment generation, proof creation, scanning |
| Backend | **Express.js** | REST API for registration and commitment lookups |
| Frontend | **Next.js** (React) | User-facing registration and payment UI |
| Blockchain | **Stellar** / **Soroban** (Protocol 25+) | Settlement layer with native BN254 support |
| CI/CD | **GitHub Actions** | Automated testing for all packages |

---

## Getting Started

### Prerequisites

| Tool | Version | Install |
|---|---|---|
| [Rust](https://rustup.rs/) | stable | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli) | latest | See Stellar docs |
| [Node.js](https://nodejs.org/) | ≥ 20 | See nodejs.org |
| [Circom](https://docs.circom.io/getting-started/installation/) | 2.x | See Circom docs |
| [snarkjs](https://github.com/iden3/snarkjs) | 0.7.x | `npm install -g snarkjs` |

### Build Contracts

```bash
cd contracts
cargo build --target wasm32-unknown-unknown --release
```

### Run Contract Tests

```bash
cd contracts
cargo test
```

### Build Circuits

```bash
cd circuits

# Install circomlib
npm install circomlib

# Compile each circuit
circom username_commitment.circom --r1cs --wasm --sym -o build/
circom payment_proof.circom       --r1cs --wasm --sym -o build/
circom stealth_address.circom     --r1cs --wasm --sym -o build/
```

For trusted setup instructions (Powers of Tau ceremony + circuit-specific phase 2), see [`docs/circuits.md`](docs/circuits.md).

### Build & Run the SDK

```bash
cd packages/sdk
npm install
npm run build
npm test
```

### Run the Backend

```bash
cd packages/backend
npm install
npm run build
npm test
```

### Run the Frontend

```bash
cd packages/frontend
npm install
npm run build
# or for development:
npm run dev
```

### Environment Variables

Copy `.env.example` to `.env` and fill in values before running integration tests:

```bash
cp .env.example .env
```

| Variable | Description |
|---|---|
| `STELLAR_SECRET_KEY` | Funded Stellar testnet keypair |
| `IDENTITY_REGISTRY` | Deployed IdentityRegistry contract address |
| `PAYMENT_ROUTER` | Deployed PaymentRouter contract address |
| `PAYMENT_PROOF_WASM` | Path to compiled circuit Wasm |
| `PAYMENT_PROOF_ZKEY` | Path to circuit proving key |

---

## Cryptographic Primitives

| Primitive | Purpose | Where Used |
|---|---|---|
| **Poseidon hash** | ZK-friendly commitment `H(username, secret)` | Circuits, SDK, Soroban host functions |
| **Groth16 (BN254)** | Succinct proof of commitment preimage knowledge | Circuits, `Groth16Verifier` contract |
| **ECDH (BabyJubJub)** | Stealth address shared secret derivation | `stealth_address.circom`, SDK |
| **Pedersen commitment** | Binding, hiding commitment scheme | Conceptual layer over Poseidon |

---

## Security Properties

| Property | Mechanism |
|---|---|
| **Username privacy** | Commitment is a one-way hash; username never stored on-chain |
| **Sender privacy** | Groth16 proof reveals nothing about sender identity |
| **Receiver privacy** | Stealth address is unlinkable to recipient's public key |
| **Replay protection** | Nullifiers prevent re-registration and proof replay |
| **Soundness** | Groth16 proof is computationally binding under BN254 discrete log assumption |

### Known Limitations

- **Trusted setup** — Groth16 requires a per-circuit trusted setup ceremony. An MPC ceremony should be run before mainnet launch.
- **Stealth scanning cost** — Recipients must scan all transactions. View tags reduce this to O(n/256), but a dedicated scanning service would improve UX.
- **Amount privacy** — Payment amounts are currently public. Hiding amounts requires range proofs (e.g. Bulletproofs) — out of scope for v1.
- **Compliance** — An optional KYC hook (following Curvy Protocol's pattern) can be added without breaking the privacy model.

---

## Prior Art

| Project | Chain | Status | What We Borrow |
|---|---|---|---|
| [Curvy Protocol](https://curvy.box) | Ethereum, Solana, Starknet | Production (2026) | Stealth address + Groth16 pattern |
| [NickPay](https://arxiv.org/abs/2503.19872) | Ethereum | Research prototype | Nickname/commitment registry design |
| [EIP-7812](https://eips.ethereum.org/EIPS/eip-7812) | Ethereum | Draft | On-chain ZK commitment registry |
| Starknet Privacy | Starknet | Production | Note model, selective disclosure |

This project is the **first native implementation** of this pattern on Stellar/Soroban, taking advantage of the Groth16 verifier and BN254 support added in Protocol 25.

---

## Roadmap

- [x] Core contract scaffolding (IdentityRegistry, PaymentRouter, Groth16Verifier)
- [x] BN254 pairing check in Groth16Verifier
- [x] Admin guard for verifying key management
- [x] Cross-contract calls (PaymentRouter → IdentityRegistry, Groth16Verifier)
- [x] Nullifier replay protection
- [x] PaymentSent event emission for SDK scanning
- [x] Circom circuit definitions
- [x] TypeScript SDK with commitment and stealth helpers
- [x] Next.js frontend with registration and send pages
- [x] Express backend API
- [x] CI pipeline (GitHub Actions)
- [ ] Emit Registered event from IdentityRegistry
- [ ] Compile circuits and run trusted setup
- [ ] Connect frontend to real SDK commitment generation
- [ ] Implement Freighter wallet connection
- [ ] Deploy contracts to Stellar testnet
- [ ] End-to-end integration tests
- [ ] Security audit of ZK circuit constraints
- [ ] Dark mode support
- [ ] Deployment guide for testnet

---

## Contributing

We welcome contributions! See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for the full guide, including:

- How to pick up issues
- PR process and coding standards
- Project areas that need work (contracts, circuits, SDK, frontend, backend)

The codebase is intentionally scaffolded with `TODO` comments marking where implementation is needed. Each TODO is a potential contribution.

---

## License

[MIT](LICENSE)
