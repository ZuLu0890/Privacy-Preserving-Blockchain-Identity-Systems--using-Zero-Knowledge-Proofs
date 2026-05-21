/**
 * POST /register
 *
 * Body: { commitment: string (hex-32), nullifier: string (hex-32) }
 *
 * Relays a registration to the IdentityRegistry Soroban contract.
 * In production this builds and submits a Stellar transaction.
 * For now it validates inputs and returns a stub tx hash.
 */
import { Router, Request, Response } from "express";

export const registerRouter = Router();

const HEX32 = /^[0-9a-f]{64}$/i;

registerRouter.post("/", (req: Request, res: Response): void => {
  const { commitment, nullifier } = req.body as {
    commitment?: string;
    nullifier?: string;
  };

  if (!commitment || !HEX32.test(commitment)) {
    res.status(400).json({ error: "commitment must be a 32-byte hex string" });
    return;
  }
  if (!nullifier || !HEX32.test(nullifier)) {
    res.status(400).json({ error: "nullifier must be a 32-byte hex string" });
    return;
  }

  // TODO: submit to IdentityRegistry via Soroban RPC
  res.status(202).json({
    txHash: "0000000000000000000000000000000000000000000000000000000000000000",
    status: "pending",
  });
});
