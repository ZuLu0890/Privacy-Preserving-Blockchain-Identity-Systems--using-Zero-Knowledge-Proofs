/**
 * GET /commitment/:username
 *
 * Returns whether a username commitment is registered on-chain.
 * In production this calls IdentityRegistry.is_registered via Soroban RPC.
 * For now it returns a stub so the frontend and CI can exercise the endpoint.
 */
import { Router } from "express";

export const commitmentRouter = Router();

const USERNAME_RE = /^[a-zA-Z0-9_.-]{3,31}$/;

commitmentRouter.get("/:username", (req, res): void => {
  const { username } = req.params;

  if (!username || !USERNAME_RE.test(username)) {
    res.status(400).json({
      error:
        "username must be 3\u201331 alphanumeric characters (underscores, dots, and hyphens allowed)",
    });
    return;
  }

  // TODO: query IdentityRegistry contract via Soroban RPC
  res.json({
    username,
    registered: false, // stub -- replace with on-chain lookup
  });
});
