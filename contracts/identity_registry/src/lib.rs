//! IdentityRegistry — stores Pedersen commitments to (username, secret) pairs.
//!
//! No wallet address is ever stored. The commitment is the only on-chain artifact.
//! A user proves knowledge of the preimage via a Groth16 proof when sending payments.
//!
//! Storage layout
//! --------------
//! COMMITMENTS : Map<BytesN<32>, bool>   — set of registered commitments
//! NULLIFIERS  : Map<BytesN<32>, bool>   — spent nullifiers (replay protection)

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, BytesN, Env};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A Pedersen commitment: H(username || secret) over BN254.
/// 32 bytes = one field element on BN254.
pub type Commitment = BytesN<32>;

/// A nullifier prevents the same commitment from being re-registered.
pub type Nullifier = BytesN<32>;

#[contracttype]
pub enum DataKey {
    Commitment(Commitment),
    Nullifier(Nullifier),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct IdentityRegistry;

#[contractimpl]
impl IdentityRegistry {
    /// Register a new username commitment.
    ///
    /// # Arguments
    /// * `commitment` — Pedersen commitment H(username, secret).
    /// * `nullifier`  — Prevents double-registration of the same username.
    ///
    /// # Errors
    /// Panics if the commitment or nullifier already exists.
    ///
    /// # TODO for contributors
    /// - Add a Groth16 proof parameter and verify it via the `Groth16Verifier`
    ///   cross-contract call before accepting the registration.
    /// - Emit a `Registered` event so the SDK can index new commitments.
    pub fn register(env: Env, commitment: Commitment, nullifier: Nullifier) {
        // Guard: commitment must be fresh
        let committed: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Commitment(commitment.clone()))
            .unwrap_or(false);
        assert!(!committed, "commitment already registered");

        // Guard: nullifier must be fresh
        let spent: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Nullifier(nullifier.clone()))
            .unwrap_or(false);
        assert!(!spent, "nullifier already used");

        // Persist
        env.storage()
            .persistent()
            .set(&DataKey::Commitment(commitment), &true);
        env.storage()
            .persistent()
            .set(&DataKey::Nullifier(nullifier), &true);
    }

    /// Check whether a commitment is registered.
    pub fn is_registered(env: Env, commitment: Commitment) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Commitment(commitment))
            .unwrap_or(false)
    }

    /// Check whether a nullifier has been spent.
    pub fn is_nullifier_spent(env: Env, nullifier: Nullifier) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Nullifier(nullifier))
            .unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, BytesN};

    fn fresh_env() -> Env {
        Env::default()
    }

    fn commitment(env: &Env, seed: u8) -> BytesN<32> {
        BytesN::from_array(env, &[seed; 32])
    }

    #[test]
    fn register_and_query() {
        let env = fresh_env();
        let contract_id = env.register(IdentityRegistry, ());
        let client = IdentityRegistryClient::new(&env, &contract_id);

        let c = commitment(&env, 1);
        let n = commitment(&env, 2);

        assert!(!client.is_registered(&c));
        client.register(&c, &n);
        assert!(client.is_registered(&c));
        assert!(client.is_nullifier_spent(&n));
    }

    #[test]
    #[should_panic(expected = "commitment already registered")]
    fn double_register_panics() {
        let env = fresh_env();
        let contract_id = env.register(IdentityRegistry, ());
        let client = IdentityRegistryClient::new(&env, &contract_id);

        let c = commitment(&env, 1);
        let n1 = commitment(&env, 2);
        let n2 = commitment(&env, 3);

        client.register(&c, &n1);
        client.register(&c, &n2); // should panic
    }
}
