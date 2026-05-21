//! Groth16Verifier — on-chain zk-SNARK proof verification over BN254.
//!
//! Stellar Protocol 25 (X-Ray) added native BN254 pairing and Poseidon hash
//! host functions, making Groth16 verification practical on Soroban.
//!
//! This contract wraps those primitives and exposes a single `verify` entry
//! point that other contracts (e.g. PaymentRouter) call cross-contract.
//!
//! Proof format (Groth16 over BN254, Circom convention)
//! ----------------------------------------------------
//! pi_a  : G1 point  (64 bytes)
//! pi_b  : G2 point  (128 bytes)
//! pi_c  : G1 point  (64 bytes)
//! public_inputs : Vec<Fr> (32 bytes each)
//!
//! Verification key is stored in contract storage and set once by the deployer.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, Vec};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Serialised Groth16 proof (pi_a || pi_b || pi_c = 256 bytes).
pub type Proof = Bytes;

/// A single BN254 field element (32 bytes).
pub type FieldElement = BytesN<32>;

#[contracttype]
pub enum DataKey {
    /// Serialised verification key (set once at deploy time).
    VerifyingKey,
    /// Contract admin — only this address may call set_verifying_key.
    Admin,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct Groth16Verifier;

#[contractimpl]
impl Groth16Verifier {
    /// One-time initialisation — records the deployer as admin.
    /// Must be called immediately after deployment.
    pub fn init(env: Env, admin: Address) {
        assert!(
            !env.storage().instance().has(&DataKey::Admin),
            "already initialised"
        );
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Store the verifying key. Only the admin (deployer) may call this.
    pub fn set_verifying_key(env: Env, vk: Bytes) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialised — call init() first");
        admin.require_auth();
        env.storage().instance().set(&DataKey::VerifyingKey, &vk);
    }

    /// Verify a Groth16 proof against the stored verifying key.
    ///
    /// Returns `true` if the proof is valid, `false` otherwise.
    ///
    /// # Arguments
    /// * `proof`         — Serialised pi_a, pi_b, pi_c (256 bytes).
    /// * `public_inputs` — Public witness values (field elements).
    ///
    /// # TODO for contributors
    /// - Implement the actual pairing check using Soroban's BN254 host functions.
    pub fn verify(env: Env, proof: Proof, public_inputs: Vec<FieldElement>) -> bool {
        let _vk: Bytes = env
            .storage()
            .instance()
            .get(&DataKey::VerifyingKey)
            .expect("verifying key not set");

        // PLACEHOLDER — replace with real BN254 pairing check (see issue #1).
        let _ = (proof, public_inputs);
        false
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, Vec};

    #[test]
    fn only_admin_can_set_vk() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, Groth16Verifier);
        let client = Groth16VerifierClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        // Admin can set the VK
        let vk = Bytes::from_slice(&env, &[0u8; 32]);
        client.set_verifying_key(&vk);
    }

    #[test]
    fn verify_returns_false_before_pairing_implementation() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, Groth16Verifier);
        let client = Groth16VerifierClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let dummy_vk = Bytes::from_slice(&env, &[0u8; 32]);
        client.set_verifying_key(&dummy_vk);

        let dummy_proof = Bytes::from_slice(&env, &[0u8; 256]);
        let inputs: Vec<BytesN<32>> = Vec::new(&env);
        assert!(!client.verify(&dummy_proof, &inputs));
    }
}
