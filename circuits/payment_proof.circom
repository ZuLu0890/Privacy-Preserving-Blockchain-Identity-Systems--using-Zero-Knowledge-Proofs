pragma circom 2.1.6;

/*
 * PaymentProof
 * ------------
 * Proves that a sender is authorised to route a payment to a stealth address
 * for a recipient identified by their on-chain commitment.
 *
 * Public inputs:
 *   recipient_commitment  -- registered commitment of the recipient's username
 *   stealth_address_hash  -- hash of the stealth address for this payment
 *   amount                -- payment amount (in stroops)
 *   nullifier             -- prevents proof replay
 *
 * Private inputs:
 *   recipient_username    -- plaintext username of the recipient
 *   recipient_secret      -- secret used when the recipient registered
 *   sender_secret         -- sender's ephemeral secret for stealth derivation
 *   nonce                 -- random nonce for nullifier derivation
 *
 * Constraints
 * -----------
 * 1. Poseidon(recipient_username, recipient_secret) == recipient_commitment
 * 2. Poseidon(recipient_username, sender_secret)    == stealth_address_hash
 * 3. amount > 0  (64-bit range check via Num2Bits)
 * 4. nullifier == Poseidon(sender_secret, nonce)
 */

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/comparators.circom";
include "node_modules/circomlib/circuits/bitify.circom";

template PaymentProof() {
    // Public inputs
    signal input recipient_commitment;
    signal input stealth_address_hash;
    signal input amount;
    signal input nullifier;

    // Private inputs
    signal input recipient_username;
    signal input recipient_secret;
    signal input sender_secret;
    signal input nonce;

    // Constraint 1: verify recipient commitment
    component commitment_hasher = Poseidon(2);
    commitment_hasher.inputs[0] <== recipient_username;
    commitment_hasher.inputs[1] <== recipient_secret;
    recipient_commitment === commitment_hasher.out;

    // Constraint 2: verify stealth address derivation (simplified)
    component stealth_hasher = Poseidon(2);
    stealth_hasher.inputs[0] <== recipient_username;
    stealth_hasher.inputs[1] <== sender_secret;
    stealth_address_hash === stealth_hasher.out;

    // Constraint 3: amount must be positive (64-bit range check)
    component amount_bits = Num2Bits(64);
    amount_bits.in <== amount;

    component is_gt_zero = GreaterThan(64);
    is_gt_zero.in[0] <== amount;
    is_gt_zero.in[1] <== 0;
    is_gt_zero.out === 1;

    // Constraint 4: nullifier = Poseidon(sender_secret, nonce)
    component nullifier_hasher = Poseidon(2);
    nullifier_hasher.inputs[0] <== sender_secret;
    nullifier_hasher.inputs[1] <== nonce;
    nullifier === nullifier_hasher.out;
}

component main {
    public [recipient_commitment, stealth_address_hash, amount, nullifier]
} = PaymentProof();
