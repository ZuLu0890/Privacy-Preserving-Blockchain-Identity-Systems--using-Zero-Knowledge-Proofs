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
//!
//! snarkjs JSON VK format
//! ----------------------
//! snarkjs exports a `verification_key.json` with this structure:
//! {
//!   "vk_alpha_1": ["<x>", "<y>", "1"],          // G1 point (affine, decimal strings)
//!   "vk_beta_2":  [["<x0>","<x1>"],["<y0>","<y1>"],["1","0"]],  // G2 point
//!   "vk_gamma_2": [["<x0>","<x1>"],["<y0>","<y1>"],["1","0"]],
//!   "vk_delta_2": [["<x0>","<x1>"],["<y0>","<y1>"],["1","0"]],
//!   "IC": [["<x>","<y>","1"], ...]               // n+1 G1 points
//! }
//!
//! The TypeScript SDK converts each decimal string coordinate to a 32-byte
//! big-endian BytesN<32> and calls `set_verifying_key_json` with the
//! structured arguments. The contract serialises them into the binary layout
//! used by `verify` and stores the result.
//!
//! Binary VK layout (stored in instance storage)
//! -----------------------------------------------
//! alpha_g1 : G1 (64 bytes)
//! beta_g2  : G2 (128 bytes)
//! gamma_g2 : G2 (128 bytes)
//! delta_g2 : G2 (128 bytes)
//! ic[0]    : G1 (64 bytes)
//! ic[1]    : G1 (64 bytes)
//! ...

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, Vec};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Serialised Groth16 proof (pi_a || pi_b || pi_c = 256 bytes).
pub type Proof = Bytes;

/// A single BN254 field element (32 bytes, big-endian).
pub type FieldElement = BytesN<32>;

/// A G1 affine point: [x (32 bytes), y (32 bytes)] = 64 bytes total.
pub type G1Point = Vec<FieldElement>;

/// A G2 affine point: [x0 (32), x1 (32), y0 (32), y1 (32)] = 128 bytes total.
/// Coordinates are in the Fp2 extension field; each Fp2 element is (c0, c1).
pub type G2Point = Vec<FieldElement>;

#[contracttype]
pub enum DataKey {
    /// Serialised verification key (binary layout, set by deployer).
    VerifyingKey,
    /// Contract admin — only this address may call set_verifying_key*.
    Admin,
}

// ---------------------------------------------------------------------------
// Serialisation helpers
// ---------------------------------------------------------------------------

/// Append a G1 point (two 32-byte coordinates) to a Bytes buffer.
fn append_g1(buf: &mut Bytes, point: &G1Point) {
    assert!(point.len() >= 2, "G1 point must have at least 2 coordinates");
    let x = point.get(0).unwrap();
    let y = point.get(1).unwrap();
    for i in 0..32u32 {
        buf.push_back(x.get(i).unwrap());
    }
    for i in 0..32u32 {
        buf.push_back(y.get(i).unwrap());
    }
}

/// Append a G2 point (four 32-byte coordinates: x0, x1, y0, y1) to a Bytes buffer.
fn append_g2(buf: &mut Bytes, point: &G2Point) {
    assert!(point.len() >= 4, "G2 point must have 4 coordinates (x0,x1,y0,y1)");
    for idx in 0..4u32 {
        let coord = point.get(idx).unwrap();
        for i in 0..32u32 {
            buf.push_back(coord.get(i).unwrap());
        }
    }
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

    /// Store the verifying key as a raw binary blob.
    /// Only the admin may call this.
    pub fn set_verifying_key(env: Env, vk: Bytes) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialised");
        admin.require_auth();
        env.storage().instance().set(&DataKey::VerifyingKey, &vk);
    }

    /// Accept the verifying key in snarkjs JSON field format and serialise it
    /// into the binary layout expected by `verify`.
    ///
    /// The TypeScript SDK parses `verification_key.json` and converts each
    /// decimal-string coordinate to a 32-byte big-endian `BytesN<32>` before
    /// calling this function.
    ///
    /// # Arguments
    /// * `alpha_g1` — `[x, y]` (2 × 32 bytes)
    /// * `beta_g2`  — `[x0, x1, y0, y1]` (4 × 32 bytes, Fp2 coords)
    /// * `gamma_g2` — `[x0, x1, y0, y1]`
    /// * `delta_g2` — `[x0, x1, y0, y1]`
    /// * `ic`       — `[[x, y], ...]` — n+1 G1 points
    pub fn set_verifying_key_json(
        env: Env,
        alpha_g1: G1Point,
        beta_g2: G2Point,
        gamma_g2: G2Point,
        delta_g2: G2Point,
        ic: Vec<G1Point>,
    ) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialised");
        admin.require_auth();

        assert!(ic.len() >= 1, "IC must have at least one point");

        let mut vk = Bytes::new(&env);

        // alpha_g1 (64 bytes)
        append_g1(&mut vk, &alpha_g1);
        // beta_g2 (128 bytes)
        append_g2(&mut vk, &beta_g2);
        // gamma_g2 (128 bytes)
        append_g2(&mut vk, &gamma_g2);
        // delta_g2 (128 bytes)
        append_g2(&mut vk, &delta_g2);
        // IC points (64 bytes each)
        for i in 0..ic.len() {
            let ic_point = ic.get(i).unwrap();
            append_g1(&mut vk, &ic_point);
        }

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
    use soroban_sdk::{vec, Address, Bytes, BytesN, Env, Vec};

    fn fe(env: &Env, val: u8) -> BytesN<32> {
        BytesN::from_array(env, &[val; 32])
    }

    fn g1(env: &Env, x: u8, y: u8) -> Vec<BytesN<32>> {
        vec![env, fe(env, x), fe(env, y)]
    }

    fn g2(env: &Env, x0: u8, x1: u8, y0: u8, y1: u8) -> Vec<BytesN<32>> {
        vec![env, fe(env, x0), fe(env, x1), fe(env, y0), fe(env, y1)]
    }

    #[test]
    fn set_vk_json_serialises_correctly() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(Groth16Verifier, ());
        let client = Groth16VerifierClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        let alpha = g1(&env, 1, 2);
        let beta = g2(&env, 3, 4, 5, 6);
        let gamma = g2(&env, 7, 8, 9, 10);
        let delta = g2(&env, 11, 12, 13, 14);
        let ic0 = g1(&env, 15, 16);
        let ic1 = g1(&env, 17, 18);
        let ic: Vec<Vec<BytesN<32>>> = vec![&env, ic0, ic1];

        client.set_verifying_key_json(&alpha, &beta, &gamma, &delta, &ic);

        // VK should be 64 + 128 + 128 + 128 + 64 + 64 = 576 bytes for 1 public input
        let vk: Bytes = env
            .as_contract(&contract_id, || {
                env.storage()
                    .instance()
                    .get(&DataKey::VerifyingKey)
                    .unwrap()
            });
        assert_eq!(vk.len(), 576);
    }
}
