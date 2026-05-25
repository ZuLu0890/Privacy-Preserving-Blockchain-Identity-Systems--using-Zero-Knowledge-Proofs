# Security Audit — ZK Circuit Constraints

This document reviews the three Circom circuits for under-constrained signals
that could allow a malicious prover to forge proofs.

---

## 1. `username_commitment.circom`

### Signals
| Signal | Visibility | Constrained? | Notes |
|---|---|---|---|
| `commitment` | public | Yes | `=== hasher.out` |
| `username` | private | Yes | Input to Poseidon |
| `secret` | private | Yes | Input to Poseidon |
| `username_length` | private | Yes | Range-checked: `3 <= len <= 31` |

### Findings

- **[OK]** The Poseidon hash constraint binds `commitment` to `(username, secret)`.
  A prover cannot produce a valid proof without knowing the preimage.
- **[OK]** `username_length` is now range-checked via `GreaterEqThan` / `LessEqThan`.
- **[NOTE]** `username_length` is a private input and is not bound to the actual
  byte-length of `username`. A malicious prover could claim any length between
  3 and 31 regardless of the real username encoding. This is acceptable because
  the commitment is the security anchor, not the length. However, if username
  length is relied upon for on-chain logic, a binding constraint between
  `username` and `username_length` should be added (e.g., checking that the
  top `(32 - username_length)` bytes of the field element are zero).
- **[NOTE]** No domain-separation tag is used. Commitments from this circuit
  could collide with commitments from other circuits using the same Poseidon(2)
  configuration. Consider adding a domain tag as a third Poseidon input.

---

## 2. `payment_proof.circom`

### Signals
| Signal | Visibility | Constrained? | Notes |
|---|---|---|---|
| `recipient_commitment` | public | Yes | `=== commitment_hasher.out` |
| `stealth_address_hash` | public | Yes | `=== stealth_hasher.out` |
| `amount` | public | Yes | 64-bit range check + `> 0` |
| `nullifier` | public | Yes | `=== Poseidon(sender_secret, nonce)` |
| `recipient_username` | private | Yes | Input to both hashers |
| `recipient_secret` | private | Yes | Input to commitment hasher |
| `sender_secret` | private | Yes | Input to stealth + nullifier hashers |
| `nonce` | private | Yes | Input to nullifier hasher |

### Findings

- **[OK]** All signals are properly constrained.
- **[OK]** `amount` has a 64-bit range check via `Num2Bits(64)` and a
  `GreaterThan` check ensuring `amount > 0`.
- **[OK]** `nullifier = Poseidon(sender_secret, nonce)` prevents proof replay
  — the contract rejects spent nullifiers.
- **[NOTE]** Stealth derivation uses a simplified `Poseidon(username, sender_secret)`
  stub instead of real ECDH. This means the stealth address is deterministic
  given the username and sender secret, which leaks the link between username
  and stealth address to anyone who knows the sender secret. In production,
  this must be replaced with proper ECDH (see `stealth_address.circom`).
- **[NOTE]** The `nonce` is private and unbounded (beyond the implicit BN254
  field modulus). This is acceptable for nullifier derivation, but the prover
  must ensure nonce uniqueness to prevent nullifier collisions.

---

## 3. `stealth_address.circom`

### Signals
| Signal | Visibility | Constrained? | Notes |
|---|---|---|---|
| `stealth_address` | public | Yes | `=== addr_hasher.out` |
| `recipient_pubkey` | public | Yes | Input to ECDH stub |
| `view_tag` | public | Yes | `=== shared_secret` |
| `sender_secret` | private | Yes | Input to ECDH stub |
| `shared_secret` | private | Yes | `=== ecdh_stub.out` and `=== view_tag` |

### Findings

- **[OK]** `shared_secret` is fully constrained by both the ECDH stub output
  and the view_tag equality.
- **[CRITICAL]** The ECDH stub uses `Poseidon(recipient_pubkey, sender_secret)`
  instead of proper elliptic curve scalar multiplication. This means:
  - The "shared secret" is computable by anyone who knows `sender_secret`
    (which the sender publishes as an ephemeral public key on-chain).
  - A passive observer who sees the ephemeral key can derive the stealth
    address and identify the recipient. **This defeats the purpose of
    stealth addresses.**
  - **Mitigation**: Replace with `EscalarMulAny` from circomlib for proper
    BabyJubJub scalar multiplication.
- **[NOTE]** `view_tag === shared_secret` exposes the full shared secret
  rather than just the first byte. In production, extract only the first byte
  (8 bits) to preserve privacy while still enabling ~99.6% scan efficiency.

---

## Summary

| Circuit | Severity | Issue |
|---|---|---|
| `stealth_address` | CRITICAL | ECDH stub is not secure; replace with real scalar multiplication |
| `stealth_address` | MEDIUM | `view_tag` exposes full shared secret; should be truncated to 1 byte |
| `username_commitment` | LOW | `username_length` not cryptographically bound to `username` |
| `username_commitment` | LOW | No domain-separation tag |
| `payment_proof` | MEDIUM | Stealth derivation uses simplified stub |
| `payment_proof` | INFO | Nonce uniqueness is the prover's responsibility |

### Recommendations

1. **Replace ECDH stubs** in `stealth_address.circom` and `payment_proof.circom`
   with proper BabyJubJub scalar multiplication.
2. **Truncate view_tag** to 8 bits in `stealth_address.circom`.
3. **Add domain separation** to `username_commitment.circom` by including a
   constant tag as a third Poseidon input.
4. **Bind `username_length`** to the actual encoding of `username` if length is
   used in any on-chain logic.
