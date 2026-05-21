/**
 * stealth.ts — Stealth address generation and scanning.
 */

import { buildPoseidon } from "circomlibjs";
import { randomBytes } from "crypto";

export interface StealthKeyPair {
  privateKey: Uint8Array;
  publicKey: Uint8Array;
}

export interface StealthPayment {
  stealthAddress: string;
  ephemeralPublicKey: Uint8Array;
  amount: bigint;
}

/** Generate a fresh stealth keypair. */
export function generateStealthKeyPair(): StealthKeyPair {
  // TODO: use proper BabyJubJub key generation
  return { privateKey: randomBytes(32), publicKey: randomBytes(32) };
}

/**
 * Derive a stealth address for a payment to `recipientPublicKey`.
 */
export async function deriveStealthAddress(
  recipientPublicKey: Uint8Array
): Promise<{
  stealthAddress: Uint8Array;
  ephemeralPubKey: Uint8Array;
  stealthAddressHash: Uint8Array;
}> {
  const poseidon = await buildPoseidon();

  const senderSecret = randomBytes(32);
  const ephemeralPubKey = randomBytes(32); // TODO: senderSecret * G

  const recipientFe = BigInt("0x" + Buffer.from(recipientPublicKey).toString("hex"));
  const senderFe = BigInt("0x" + Buffer.from(senderSecret).toString("hex"));

  // circomlibjs Poseidon returns Uint8Array(32) directly
  const stealthAddress = poseidon([recipientFe, senderFe]);
  const stealthAddressHash = poseidon([
    BigInt("0x" + Buffer.from(stealthAddress).toString("hex")),
  ]);

  return { stealthAddress, ephemeralPubKey, stealthAddressHash };
}

/** Scan payments for those belonging to keyPair. TODO: implement. */
export async function scanPayments(
  _keyPair: StealthKeyPair,
  _payments: StealthPayment[]
): Promise<StealthPayment[]> {
  throw new Error("scanPayments not yet implemented");
}
