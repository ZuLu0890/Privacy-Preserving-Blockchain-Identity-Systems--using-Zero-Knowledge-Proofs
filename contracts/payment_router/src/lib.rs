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
//!    - `nullifier`            — unique value derived from the proof; prevents replay.
//!    - `amount`               — XLM stroops to transfer.
//!
//! 2. Router checks the nullifier has not been spent (replay protection).
//! 3. Router checks the commitment exists in `IdentityRegistry` (cross-contract).
//! 4. Router verifies the proof via cross-contract call to `Groth16Verifier`.
//! 5. Router marks the nullifier as spent.
//! 6. Router transfers `amount` to `stealth_address`.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Bytes, BytesN, Env, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

pub type Commitment = BytesN<32>;
pub type Nullifier = BytesN<32>;
pub type Proof = Bytes;
pub type FieldElement = BytesN<32>;

#[contracttype]
pub enum DataKey {
    /// Address of the deployed IdentityRegistry contract.
    RegistryAddress,
    /// Address of the deployed Groth16Verifier contract.
    VerifierAddress,
    /// Spent nullifier set — maps Nullifier → bool.
    Nullifier(Nullifier),
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

    /// Check whether a nullifier has already been spent.
    pub fn is_nullifier_spent(env: Env, nullifier: Nullifier) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Nullifier(nullifier))
            .unwrap_or(false)
    }

    /// Send a private payment to a stealth address.
    ///
    /// The `nullifier` must be a fresh value derived from the proof (e.g.
    /// H(proof_randomness, recipient_commitment)). It is stored on-chain after
    /// a successful payment so the same proof cannot be replayed.
    pub fn send(
        env: Env,
        token: Address,
        recipient_commitment: Commitment,
        stealth_address: Address,
        proof: Proof,
        public_inputs: Vec<FieldElement>,
        nullifier: Nullifier,
        amount: i128,
    ) {
        // --- Replay protection: nullifier must be fresh ---
        let spent: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Nullifier(nullifier.clone()))
            .unwrap_or(false);
        assert!(!spent, "nullifier already spent");

        // TODO: verify commitment is registered (see issue #4)
        let _ = &recipient_commitment;

        // TODO: verify ZK proof (see issue #5)
        let _ = (proof, public_inputs);

        // --- Mark nullifier as spent before transferring (checks-effects-interactions) ---
        env.storage()
            .persistent()
            .set(&DataKey::Nullifier(nullifier), &true);

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
    use super::*;
    use soroban_sdk::{Env, BytesN};

    #[test]
    fn nullifier_starts_unspent() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PaymentRouter);
        let client = PaymentRouterClient::new(&env, &contract_id);

        let n = BytesN::from_array(&env, &[1u8; 32]);
        assert!(!client.is_nullifier_spent(&n));
    }
}
