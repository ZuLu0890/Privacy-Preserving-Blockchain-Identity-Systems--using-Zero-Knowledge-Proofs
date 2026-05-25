"use client";

import { useState, useEffect, useCallback } from "react";

type TxStatus = "idle" | "loading" | "pending" | "confirmed" | "error";

export default function RegisterPage() {
  const [username, setUsername] = useState("");
  const [status, setStatus] = useState<TxStatus>("idle");
  const [message, setMessage] = useState("");
  const [txHash, setTxHash] = useState<string | null>(null);

  const apiUrl = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3001";

  async function handleRegister(e: React.FormEvent) {
    e.preventDefault();
    if (username.length < 3 || username.length > 31) {
      setStatus("error");
      setMessage("Username must be 3\u201331 characters.");
      return;
    }

    setStatus("loading");
    try {
      const secret = Array.from(crypto.getRandomValues(new Uint8Array(32)))
        .map((b) => b.toString(16).padStart(2, "0"))
        .join("");

      const enc = new TextEncoder();
      const data = enc.encode(username + secret);
      const hashBuf = await crypto.subtle.digest("SHA-256", data);
      const commitment = Array.from(new Uint8Array(hashBuf))
        .map((b) => b.toString(16).padStart(2, "0"))
        .join("");

      const nullBuf = await crypto.subtle.digest("SHA-256", enc.encode(secret));
      const nullifier = Array.from(new Uint8Array(nullBuf))
        .map((b) => b.toString(16).padStart(2, "0"))
        .join("");

      const res = await fetch(`${apiUrl}/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ commitment, nullifier }),
      });

      if (!res.ok) {
        const err = await res.json();
        throw new Error(err.error ?? "Registration failed");
      }

      const body = await res.json();
      setTxHash(body.txHash);
      setStatus("pending");
      setMessage(`Submitted! tx: ${body.txHash}`);
    } catch (err: unknown) {
      setStatus("error");
      setMessage(err instanceof Error ? err.message : "Unknown error");
    }
  }

  const pollStatus = useCallback(async () => {
    if (!txHash) return;
    try {
      const res = await fetch(`${apiUrl}/health`);
      if (res.ok) {
        setStatus("confirmed");
        setMessage(`Confirmed! tx: ${txHash}`);
      }
    } catch {
      // keep polling
    }
  }, [txHash, apiUrl]);

  useEffect(() => {
    if (status !== "pending") return;
    const interval = setInterval(pollStatus, 3000);
    return () => clearInterval(interval);
  }, [status, pollStatus]);

  return (
    <>
      <h2>Register Username</h2>
      <p style={{ color: "var(--muted)" }}>
        Your wallet address is never stored. Only a zero-knowledge commitment is
        written on-chain.
      </p>
      <form onSubmit={handleRegister} style={{ display: "flex", flexDirection: "column", gap: 12 }}>
        <label>
          Username
          <input
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder="alice"
            minLength={3}
            maxLength={31}
            required
          />
        </label>
        <button type="submit" disabled={status === "loading" || status === "pending"}>
          {status === "loading" ? "Registering\u2026" : status === "pending" ? "Confirming\u2026" : "Register"}
        </button>
      </form>
      {status === "confirmed" && <p style={{ color: "var(--success)", marginTop: 12 }}>{message}</p>}
      {status === "pending" && <p style={{ color: "var(--accent)", marginTop: 12 }}>{message}</p>}
      {status === "error" && <p style={{ color: "var(--error)", marginTop: 12 }}>{message}</p>}
    </>
  );
}
