# Deployment Guide — Stellar Testnet

This guide walks you through deploying all three Soroban contracts, running the
Circom trusted setup, and starting the backend and frontend locally against the
Stellar testnet.

---

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust | stable | https://rustup.rs |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI | latest | https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli |
| Node.js | >= 20 | https://nodejs.org |
| Circom | 2.x | https://docs.circom.io/getting-started/installation |
| snarkjs | 0.7.x | `npm install -g snarkjs` |

---

## 1. Fund a Testnet Account

Visit the [Stellar Laboratory](https://laboratory.stellar.org/#account-creator?network=test)
and create + fund a testnet keypair.

Save the **secret key** — you will need it for contract deployment and as
`STELLAR_SECRET_KEY` in your `.env`.

```bash
export STELLAR_SECRET_KEY="S..."
```

---

## 2. Build Contracts

```bash
cd contracts
cargo build --target wasm32-unknown-unknown --release
```

The compiled `.wasm` files will be in
`target/wasm32-unknown-unknown/release/`.

---

## 3. Deploy Contracts

### 3.1 IdentityRegistry

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/identity_registry.wasm \
  --source "$STELLAR_SECRET_KEY" \
  --network testnet
```

Note the returned **contract address** — save it as `IDENTITY_REGISTRY`.

### 3.2 Groth16Verifier

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/groth16_verifier.wasm \
  --source "$STELLAR_SECRET_KEY" \
  --network testnet
```

Note the contract address — save it as `GROTH16_VERIFIER`.

Initialise the verifier (replace `G...` with your public key):

```bash
stellar contract invoke \
  --id "$GROTH16_VERIFIER" \
  --source "$STELLAR_SECRET_KEY" \
  --network testnet \
  -- init --admin "G..."
```

### 3.3 PaymentRouter

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payment_router.wasm \
  --source "$STELLAR_SECRET_KEY" \
  --network testnet
```

Initialise the router with the registry and verifier addresses:

```bash
stellar contract invoke \
  --id "$PAYMENT_ROUTER" \
  --source "$STELLAR_SECRET_KEY" \
  --network testnet \
  -- init \
    --registry "$IDENTITY_REGISTRY" \
    --verifier "$GROTH16_VERIFIER"
```

---

## 4. Compile Circuits and Run Trusted Setup

### 4.1 Install circomlib

```bash
cd circuits
npm init -y
npm install circomlib
```

### 4.2 Compile each circuit

```bash
# username_commitment
circom username_commitment.circom --r1cs --wasm --sym -o build/

# payment_proof
circom payment_proof.circom --r1cs --wasm --sym -o build/

# stealth_address
circom stealth_address.circom --r1cs --wasm --sym -o build/
```

### 4.3 Groth16 Trusted Setup

Download a Powers of Tau file (BN254, 2^16 constraints is sufficient):

```bash
wget https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_16.ptau
```

Run the setup for each circuit:

```bash
for circuit in username_commitment payment_proof stealth_address; do
  snarkjs groth16 setup \
    "build/${circuit}.r1cs" \
    powersOfTau28_hez_final_16.ptau \
    "build/${circuit}_0000.zkey"

  snarkjs zkey contribute \
    "build/${circuit}_0000.zkey" \
    "build/${circuit}_0001.zkey" \
    --name="first contribution" -v

  snarkjs zkey export verificationkey \
    "build/${circuit}_0001.zkey" \
    "build/${circuit}_verification_key.json"
done
```

### 4.4 Upload Verifying Key to Contract

Use the SDK or a script to convert the verification key JSON and call
`Groth16Verifier.set_verifying_key_json()`.

---

## 5. Configure Environment

Copy `.env.example` to `.env` and fill in the values:

```bash
cp .env.example .env
```

```
STELLAR_SECRET_KEY=S...
IDENTITY_REGISTRY=C...
PAYMENT_ROUTER=C...
PAYMENT_PROOF_WASM=circuits/build/payment_proof_js/payment_proof.wasm
PAYMENT_PROOF_ZKEY=circuits/build/payment_proof_0001.zkey
```

---

## 6. Run the Backend

```bash
cd packages/backend
npm install
npm run dev
```

The server starts on `http://localhost:3001`.

---

## 7. Run the Frontend

```bash
cd packages/frontend
npm install
npm run dev
```

The app starts on `http://localhost:3000`.

Set `NEXT_PUBLIC_API_URL=http://localhost:3001` if the backend runs on a
different host.

---

## 8. Run Tests

```bash
# Contract tests
cd contracts && cargo test

# SDK tests
cd packages/sdk && npm install && npm test

# Backend tests
cd packages/backend && npm test

# Integration tests (requires deployed contracts + .env)
cd packages/sdk && npm test -- --testPathPattern=integration
```

---

## Troubleshooting

| Issue | Solution |
|---|---|
| `cargo build` fails with wasm target | `rustup target add wasm32-unknown-unknown` |
| `stellar` CLI not found | Install from https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli |
| Testnet account has no funds | Fund at https://laboratory.stellar.org/#account-creator?network=test |
| `circom` not found | Install from https://docs.circom.io/getting-started/installation |
| `snarkjs` not found | `npm install -g snarkjs` |
