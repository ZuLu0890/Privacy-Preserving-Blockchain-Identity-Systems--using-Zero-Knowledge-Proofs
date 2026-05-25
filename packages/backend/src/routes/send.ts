/**
 * POST /send
 *
 * Body: {
 *   recipient_commitment: string (hex-32),
 *   stealth_address: string (hex-32),
 *   proof: string (hex),
 *   public_inputs: string[] (hex-32 each),
 *   nullifier: string (hex-32),
 *   amount: string (numeric)
 * }
 *
 * Relays a private payment to the PaymentRouter Soroban contract.
 * In production this builds and submits a Stellar transaction.
 * For now it validates inputs and returns a stub tx hash.
 */
import { Router, Request, Response } from "express";

export const sendRouter = Router();

const HEX32 = /^[0-9a-f]{64}$/i;
const HEX = /^[0-9a-f]+$/i;
const NUMERIC = /^\d+$/;

sendRouter.post("/", (req: Request, res: Response): void => {
  const {
    recipient_commitment,
    stealth_address,
    proof,
    public_inputs,
    nullifier,
    amount,
  } = req.body as {
    recipient_commitment?: string;
    stealth_address?: string;
    proof?: string;
    public_inputs?: string[];
    nullifier?: string;
    amount?: string;
  };

  if (!recipient_commitment || !HEX32.test(recipient_commitment)) {
    res
      .status(400)
      .json({ error: "recipient_commitment must be a 32-byte hex string" });
    return;
  }
  if (!stealth_address || !HEX32.test(stealth_address)) {
    res
      .status(400)
      .json({ error: "stealth_address must be a 32-byte hex string" });
    return;
  }
  if (!proof || !HEX.test(proof) || proof.length < 2 || proof.length > 1024) {
    res.status(400).json({ error: "proof must be a valid hex string (1-512 bytes)" });
    return;
  }
  if (
    !Array.isArray(public_inputs) ||
    public_inputs.length === 0 ||
    !public_inputs.every((pi) => HEX32.test(pi))
  ) {
    res
      .status(400)
      .json({ error: "public_inputs must be a non-empty array of 32-byte hex strings" });
    return;
  }
  if (!nullifier || !HEX32.test(nullifier)) {
    res.status(400).json({ error: "nullifier must be a 32-byte hex string" });
    return;
  }
  if (!amount || !NUMERIC.test(amount) || BigInt(amount) <= 0n) {
    res
      .status(400)
      .json({ error: "amount must be a positive integer (stroops)" });
    return;
  }

  // TODO: submit to PaymentRouter via Soroban RPC
  res.status(202).json({
    txHash:
      "0000000000000000000000000000000000000000000000000000000000000000",
    status: "pending",
  });
});
