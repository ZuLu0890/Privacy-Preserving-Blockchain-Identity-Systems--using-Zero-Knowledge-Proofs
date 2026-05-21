//! Groth16Verifier — on-chain zk-SNARK proof verification over BN254.
//!
//! Stellar Protocol 25 (X-Ray) added native BN254 pairing and Poseidon hash
//! host functions, making Groth16 verification practical on Soroban.
//!
//! Proof format (Groth16 over BN254, Circom convention)
//! ----------------------------------------------------
//! pi_a  : G1 point  (64 bytes)
//! pi_b  : G2 point  (128 bytes)
//! pi_c  : G1 point  (64 bytes)
//! public_inputs : Vec<Fr> (32 bytes each)
//!
//! Verifying key layout (stored in instance storage)
//! --------------------------------------------------
//! alpha_g1 : G1 (64 bytes)
//! beta_g2  : G2 (128 bytes)
//! gamma_g2 : G2 (128 bytes)
//! delta_g2 : G2 (128 bytes)
//! ic       : [G1; n+1] (64*(n+1) bytes, n = number of public inputs)
//!
//! The four-pairing Groth16 check is:
//!   e(pi_a, pi_b) == e(alpha_g1, beta_g2) * e(vk_x, gamma_g2) * e(pi_c, delta_g2)
//! which is equivalent to the multi-pairing check:
//!   e(-pi_a, pi_b) * e(alpha_g1, beta_g2) * e(vk_x, gamma_g2) * e(pi_c, delta_g2) == 1

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    crypto::bn254::{Bn254Fr, Bn254G1Affine, Bn254G2Affine},
    vec, Address, Bytes, BytesN, Env, Vec,
};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Serialised Groth16 proof (pi_a || pi_b || pi_c = 256 bytes).
pub type Proof = Bytes;

/// A single BN254 field element (32 bytes).
pub type FieldElement = BytesN<32>;

#[contracttype]
pub enum DataKey {
    /// Serialised verification key.
    VerifyingKey,
    /// Contract admin (deployer).
    Admin,
}

// ---------------------------------------------------------------------------
// VK parsing helpers
// ---------------------------------------------------------------------------

fn g1_from_bytes(env: &Env, data: &Bytes, offset: u32) -> Bn254G1Affine {
    let mut buf = [0u8; 64];
    for i in 0..64u32 {
        buf[i as usize] = data.get(offset + i).unwrap();
    }
    Bn254G1Affine::from_array(env, &buf)
}

fn g2_from_bytes(env: &Env, data: &Bytes, offset: u32) -> Bn254G2Affine {
    let mut buf = [0u8; 128];
    for i in 0..128u32 {
        buf[i as usize] = data.get(offset + i).unwrap();
    }
    Bn254G2Affine::from_array(env, &buf)
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct Groth16Verifier;

#[contractimpl]
impl Groth16Verifier {
    /// One-time initialisation — records the deployer as admin.
    pub fn init(env: Env, admin: Address) {
        assert!(
            !env.storage().instance().has(&DataKey::Admin),
            "already initialised"
        );
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Store the verifying key. Only the admin (deployer) may call this.
    ///
    /// `vk` must be serialised as:
    ///   alpha_g1 (64) || beta_g2 (128) || gamma_g2 (128) || delta_g2 (128) || ic_0 (64) || ic_1 (64) || …
    pub fn set_verifying_key(env: Env, vk: Bytes) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialised");
        admin.require_auth();
        env.storage().instance().set(&DataKey::VerifyingKey, &vk);
    }

    /// Verify a Groth16 proof against the stored verifying key.
    ///
    /// Returns `true` if the proof is valid, `false` otherwise.
    ///
    /// # Arguments
    /// * `proof`         — Serialised pi_a (64) || pi_b (128) || pi_c (64) = 256 bytes.
    /// * `public_inputs` — Public witness values (field elements, 32 bytes each).
    pub fn verify(env: Env, proof: Proof, public_inputs: Vec<FieldElement>) -> bool {
        let vk: Bytes = env
            .storage()
            .instance()
            .get(&DataKey::VerifyingKey)
            .expect("verifying key not set");

        let bn254 = env.crypto().bn254();

        // --- Parse proof ---
        assert!(proof.len() == 256, "proof must be 256 bytes");
        let pi_a = g1_from_bytes(&env, &proof, 0);
        let pi_b = g2_from_bytes(&env, &proof, 64);
        let pi_c = g1_from_bytes(&env, &proof, 192);

        // --- Parse VK ---
        // Layout: alpha_g1(64) | beta_g2(128) | gamma_g2(128) | delta_g2(128) | ic[0](64) | ic[1](64) | ...
        let alpha_g1 = g1_from_bytes(&env, &vk, 0);
        let beta_g2 = g2_from_bytes(&env, &vk, 64);
        let gamma_g2 = g2_from_bytes(&env, &vk, 192);
        let delta_g2 = g2_from_bytes(&env, &vk, 320);

        // IC points start at offset 448; there must be n+1 of them for n public inputs
        let n = public_inputs.len();
        let ic_start: u32 = 448;
        assert!(
            vk.len() >= ic_start + 64 * (n + 1),
            "vk too short for public inputs"
        );

        // --- Compute vk_x = IC[0] + sum(public_inputs[i] * IC[i+1]) ---
        let mut vk_x = g1_from_bytes(&env, &vk, ic_start);
        for i in 0..n {
            let ic_i = g1_from_bytes(&env, &vk, ic_start + 64 + 64 * i);
            let scalar_bytes = public_inputs.get(i).unwrap();
            let mut scalar_arr = [0u8; 32];
            for j in 0..32u32 {
                scalar_arr[j as usize] = scalar_bytes.get(j).unwrap();
            }
            let scalar = Bn254Fr::from_array(&env, &scalar_arr);
            let term = bn254.g1_mul(&ic_i, &scalar);
            vk_x = bn254.g1_add(&vk_x, &term);
        }

        // --- Four-pairing Groth16 check ---
        // e(-pi_a, pi_b) * e(alpha_g1, beta_g2) * e(vk_x, gamma_g2) * e(pi_c, delta_g2) == 1
        let neg_pi_a = -pi_a;

        let g1_points: Vec<Bn254G1Affine> =
            vec![&env, neg_pi_a, alpha_g1, vk_x, pi_c];
        let g2_points: Vec<Bn254G2Affine> =
            vec![&env, pi_b, beta_g2, gamma_g2, delta_g2];

        bn254.pairing_check(g1_points, g2_points)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Bytes, Env, Vec};

    #[test]
    fn verify_rejects_zero_proof_without_vk() {
        let env = Env::default();
        let contract_id = env.register_contract(None, Groth16Verifier);
        let client = Groth16VerifierClient::new(&env, &contract_id);

        // init + set_verifying_key require auth; skip for this panic test
        // Just confirm that calling verify without a VK panics.
        let result = std::panic::catch_unwind(|| {
            let dummy_proof = Bytes::from_slice(&env, &[0u8; 256]);
            let inputs: Vec<BytesN<32>> = Vec::new(&env);
            client.verify(&dummy_proof, &inputs);
        });
        assert!(result.is_err());
    }
}
