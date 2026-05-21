//! PaymentRouter — routes XLM/token payments to stealth addresses.
//!
//! Flow
//! ----
//! 1. Sender calls `send` with:
//!    - `recipient_commitment` — the on-chain commitment of the recipient's username.
//!    - `stealth_address`      — a fresh one-time address derived by the sender from
//!                               the recipient's public key (ECDH + hash).
//!    - `proof`                — Groth16 proof that the sender knows the preimage of
//!                               `recipient_commitment` (i.e. the username).
//!    - `public_inputs`        — public witness: [commitment, stealth_address_hash, amount].
//!    - `amount`               — XLM stroops to transfer.
//!
//! 2. Router checks the commitment exists in `IdentityRegistry` (cross-contract).
//! 3. Router verifies the proof via cross-contract call to `Groth16Verifier`.
//! 4. Router transfers `amount` to `stealth_address`.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Bytes, BytesN, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

pub type Commitment = BytesN<32>;
pub type Proof = Bytes;
pub type FieldElement = BytesN<32>;

#[contracttype]
pub enum DataKey {
    /// Address of the deployed IdentityRegistry contract.
    RegistryAddress,
    /// Address of the deployed Groth16Verifier contract.
    VerifierAddress,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct PaymentRouter;

#[contractimpl]
impl PaymentRouter {
    /// One-time initialisation. Must be called by the deployer.
    pub fn init(env: Env, registry: Address, verifier: Address) {
        env.storage()
            .instance()
            .set(&DataKey::RegistryAddress, &registry);
        env.storage()
            .instance()
            .set(&DataKey::VerifierAddress, &verifier);
    }

    /// Send a private payment to a stealth address.
    ///
    /// Rejects the payment if the recipient commitment is not registered in
    /// `IdentityRegistry` (cross-contract call to `is_registered`).
    pub fn send(
        env: Env,
        token: Address,
        recipient_commitment: Commitment,
        stealth_address: Address,
        proof: Proof,
        public_inputs: Vec<FieldElement>,
        amount: i128,
    ) {
        // --- Verify commitment is registered in IdentityRegistry ---
        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("not initialised");

        let is_registered: bool = env.invoke_contract(
            &registry,
            &Symbol::new(&env, "is_registered"),
            soroban_sdk::vec![&env, recipient_commitment.to_val()],
        );
        assert!(is_registered, "unknown commitment");

        // TODO: verify ZK proof (see issue #5)
        let _ = (proof, public_inputs);

        // Transfer funds to the stealth address
        let sender = env.current_contract_address();
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&sender, &stealth_address, &amount);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    // Integration tests live in tests/integration/.
}
