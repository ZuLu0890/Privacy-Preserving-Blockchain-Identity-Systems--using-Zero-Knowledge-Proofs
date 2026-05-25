pragma circom 2.1.6;

/*
 * StealthAddress
 * --------------
 * Derives a one-time stealth address from a recipient's public key and a
 * sender's ephemeral secret, following the ERC-5564 stealth address scheme
 * adapted for BN254 (the curve used by Stellar Protocol 25).
 *
 * Scheme (simplified)
 * -------------------
 * Given:
 *   recipient_pubkey  -- recipient's public key (BN254 G1 point, x-coordinate)
 *   sender_secret     -- sender's ephemeral scalar (random per payment)
 *
 * Compute:
 *   shared_secret     = Poseidon(recipient_pubkey, sender_secret)  [ECDH stub]
 *   stealth_address   = Poseidon(shared_secret, 0)                 [derive address]
 *   view_tag          = shared_secret (first byte used off-chain for fast scanning)
 *
 * Public inputs:  stealth_address, recipient_pubkey, view_tag
 * Private inputs: sender_secret, shared_secret
 *
 * The recipient scans the chain, recomputes shared_secret using their private
 * key, and checks whether stealth_address matches. The view_tag allows
 * recipients to skip ~99.6% of transactions without full ECDH computation.
 *
 * TODO for contributors
 * ---------------------
 * - Replace the Poseidon-based ECDH stub with a proper BabyJubJub or BN254
 *   scalar multiplication using circomlib's EscalarMulAny component.
 */

include "node_modules/circomlib/circuits/poseidon.circom";

template StealthAddress() {
    // Public inputs
    signal input stealth_address;
    signal input recipient_pubkey;
    signal input view_tag;

    // Private inputs
    signal input sender_secret;
    signal input shared_secret; // = ECDH(recipient_pubkey, sender_secret)

    // Constrain shared_secret = Poseidon(recipient_pubkey, sender_secret)
    // TODO: replace with real scalar multiplication (BabyJubJub EscalarMulAny)
    component ecdh_stub = Poseidon(2);
    ecdh_stub.inputs[0] <== recipient_pubkey;
    ecdh_stub.inputs[1] <== sender_secret;
    shared_secret === ecdh_stub.out;

    // Derive stealth address from shared secret
    component addr_hasher = Poseidon(2);
    addr_hasher.inputs[0] <== shared_secret;
    addr_hasher.inputs[1] <== 0;
    stealth_address === addr_hasher.out;

    // View tag: expose shared_secret as public output for efficient scanning
    view_tag === shared_secret;
}

component main {
    public [stealth_address, recipient_pubkey, view_tag]
} = StealthAddress();
