pragma circom 2.1.6;

/*
 * UsernameCommitment
 * ------------------
 * Proves knowledge of (username, secret) such that:
 *
 *   commitment = Poseidon(username, secret)
 *
 * Public inputs:  commitment
 * Private inputs: username, secret
 *
 * The on-chain IdentityRegistry stores only `commitment`.
 * This circuit lets a user prove they own a registered username
 * without revealing the username or the secret.
 *
 * Poseidon is used because Stellar Protocol 25 added a native
 * Poseidon host function, making on-chain verification cheap.
 */

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/comparators.circom";
include "node_modules/circomlib/circuits/bitify.circom";

template UsernameCommitment() {
    // Public
    signal input commitment;

    // Private
    signal input username;
    signal input secret;
    signal input username_length; // byte-length of the UTF-8 username (3..=31)

    // --- Username length range check: 3 <= username_length <= 31 ---
    component gte3 = GreaterEqThan(8);
    gte3.in[0] <== username_length;
    gte3.in[1] <== 3;
    gte3.out === 1;

    component lte31 = LessEqThan(8);
    lte31.in[0] <== username_length;
    lte31.in[1] <== 31;
    lte31.out === 1;

    // Compute Poseidon(username, secret)
    component hasher = Poseidon(2);
    hasher.inputs[0] <== username;
    hasher.inputs[1] <== secret;

    // Constrain: computed hash must equal the public commitment
    commitment === hasher.out;
}

component main { public [commitment] } = UsernameCommitment();
