/**
 * proof.ts
 *
 * Client-side Groth16 proof generation using snarkjs.
 *
 * The prover runs entirely in the browser or Node.js — no server sees
 * the private inputs (username, secret).
 *
 * Workflow
 * --------
 * 1. Compile circuits with Circom (see circuits/).
 * 2. Run a Groth16 trusted setup to produce a .zkey file.
 * 3. Export the verifying key and deploy it to the Groth16Verifier contract.
 * 4. At payment time, call `generatePaymentProof` to produce a proof that
 *    the PaymentRouter contract will verify on-chain.
 */

import * as snarkjs from "snarkjs";
import { encodeUsername, fieldElementToBytes } from "./commitment";

export interface PaymentProofInputs {
  /** Plaintext username of the recipient (private). */
  recipientUsername: string;
  /** Secret used when the recipient registered (private). */
  recipientSecret: Uint8Array;
  /** Sender's ephemeral secret for stealth address derivation (private). */
  senderSecret: Uint8Array;
  /** On-chain commitment of the recipient (public). */
  recipientCommitment: Uint8Array;
  /** Hash of the stealth address for this payment (public). */
  stealthAddressHash: Uint8Array;
  /** Payment amount in stroops (public). */
  amount: bigint;
}

export interface Groth16Proof {
  /** Serialised proof bytes (pi_a || pi_b || pi_c). */
  proofBytes: Uint8Array;
  /** Public inputs as 32-byte field elements. */
  publicInputs: Uint8Array[];
}

/**
 * Generate a Groth16 proof for a private payment.
 *
 * @param inputs   Proof inputs (see PaymentProofInputs).
 * @param wasmPath Path to the compiled circuit .wasm file.
 * @param zkeyPath Path to the Groth16 .zkey file (from trusted setup).
 */
export async function generatePaymentProof(
  inputs: PaymentProofInputs,
  wasmPath: string,
  zkeyPath: string
): Promise<Groth16Proof> {
  const circuitInputs = {
    recipient_commitment: bytesToBigInt(inputs.recipientCommitment).toString(),
    stealth_address_hash: bytesToBigInt(inputs.stealthAddressHash).toString(),
    amount: inputs.amount.toString(),
    recipient_username: encodeUsername(inputs.recipientUsername).toString(),
    recipient_secret: bytesToBigInt(inputs.recipientSecret).toString(),
    sender_secret: bytesToBigInt(inputs.senderSecret).toString(),
  };

  const { proof, publicSignals } = await snarkjs.groth16.fullProve(
    circuitInputs,
    wasmPath,
    zkeyPath
  );

  return {
    proofBytes: serializeProof(proof),
    publicInputs: publicSignals.map((s: string) =>
      fieldElementToBytes(BigInt(s))
    ),
  };
}

/**
 * Verify a proof locally (useful for testing before submitting on-chain).
 */
export async function verifyProofLocally(
  proof: Groth16Proof,
  vkeyPath: string
): Promise<boolean> {
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const vkey = JSON.parse(require("fs").readFileSync(vkeyPath, "utf8"));
  const snarkProof = deserializeProof(proof.proofBytes);
  const publicSignals = proof.publicInputs.map((b) =>
    bytesToBigInt(b).toString()
  );
  return snarkjs.groth16.verify(vkey, publicSignals, snarkProof);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function bytesToBigInt(bytes: Uint8Array): bigint {
  return BigInt("0x" + Buffer.from(bytes).toString("hex"));
}

/**
 * Serialise a snarkjs proof object to bytes (pi_a || pi_b || pi_c).
 * Format matches what the Groth16Verifier contract expects.
 *
 * TODO: align byte layout with the Soroban contract's deserialisation logic.
 */
function serializeProof(proof: snarkjs.Groth16Proof): Uint8Array {
  const encode = (point: string[]): Uint8Array => {
    const buf = new Uint8Array(point.length * 32);
    point.forEach((coord, i) => {
      const hex = BigInt(coord).toString(16).padStart(64, "0");
      buf.set(Buffer.from(hex, "hex"), i * 32);
    });
    return buf;
  };

  const a = encode(proof.pi_a.slice(0, 2));
  const b = encode(proof.pi_b.flat().slice(0, 4));
  const c = encode(proof.pi_c.slice(0, 2));

  const result = new Uint8Array(a.length + b.length + c.length);
  result.set(a, 0);
  result.set(b, a.length);
  result.set(c, a.length + b.length);
  return result;
}

function deserializeProof(_bytes: Uint8Array): snarkjs.Groth16Proof {
  // TODO: implement the inverse of serializeProof
  throw new Error("deserializeProof not yet implemented");
}
