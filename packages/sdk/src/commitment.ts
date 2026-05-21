/**
 * commitment.ts
 *
 * Pedersen / Poseidon commitment helpers for username registration.
 */

import { buildPoseidon, PoseidonFn } from "circomlibjs";
import { randomBytes } from "crypto";

export type Commitment = Uint8Array; // 32 bytes, BN254 field element
export type Nullifier = Uint8Array;  // 32 bytes

let _poseidon: PoseidonFn | null = null;

async function getPoseidon(): Promise<PoseidonFn> {
  if (!_poseidon) _poseidon = await buildPoseidon();
  return _poseidon;
}

/**
 * Encode a UTF-8 username string as a BN254 field element (big-endian, 32 bytes).
 * Usernames longer than 31 bytes are rejected.
 */
export function encodeUsername(username: string): bigint {
  const bytes = new TextEncoder().encode(username);
  if (bytes.length > 31) throw new Error("username too long (max 31 bytes)");
  const padded = new Uint8Array(32);
  padded.set(bytes, 32 - bytes.length);
  return BigInt("0x" + Buffer.from(padded).toString("hex"));
}

/** Generate a cryptographically random 32-byte secret. */
export function generateSecret(): Uint8Array {
  return randomBytes(32);
}

/**
 * Compute the on-chain commitment: Poseidon(username_field, secret_field).
 * Returns 32-byte commitment and nullifier.
 */
export async function buildCommitment(
  username: string,
  secret: Uint8Array
): Promise<{ commitment: Commitment; nullifier: Nullifier }> {
  const p = await getPoseidon();

  const usernameFe = encodeUsername(username);
  const secretFe = BigInt("0x" + Buffer.from(secret).toString("hex"));

  // circomlibjs Poseidon returns Uint8Array(32) directly
  const commitment = p([usernameFe, secretFe]);
  const nullifier = p([secretFe, 1n]);

  return { commitment, nullifier };
}

/** Convert a BN254 field element (bigint) to a 32-byte big-endian Uint8Array. */
export function fieldElementToBytes(fe: bigint): Uint8Array {
  const hex = fe.toString(16).padStart(64, "0");
  return Uint8Array.from(Buffer.from(hex, "hex"));
}
