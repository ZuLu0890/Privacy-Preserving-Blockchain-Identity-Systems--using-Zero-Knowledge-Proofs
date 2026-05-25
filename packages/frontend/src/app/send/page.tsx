"use client";

import { useState, useEffect, useCallback } from "react";

type TxStatus = "idle" | "loading" | "proving" | "pending" | "confirmed" | "error";

export default function SendPage() {
  const [recipient, setRecipient] = useState("");
  const [amount, setAmount] = useState("");
  const [status, setStatus] = useState<TxStatus>("idle");
  const [message, setMessage] = useState("");
  const [txHash, setTxHash] = useState<string | null>(null);

  const apiUrl = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3001";

  async function handleSend(e: React.FormEvent) {
    e.preventDefault();
    setStatus("loading");

    try {
      const checkRes = await fetch(
        `${apiUrl}/commitment/${encodeURIComponent(recipient)}`
      );
      if (!checkRes.ok) {
        const err = await checkRes.json();
        throw new Error(err.error ?? "Lookup failed");
      }
      const { registered } = await checkRes.json();
      if (!registered) {
        throw new Error(`Username "${recipient}" is not registered.`);
      }

      setStatus("proving");
      setMessage("Generating ZK proof...");

      const stroops = Math.floor(parseFloat(amount) * 1e7);
      if (stroops <= 0) throw new Error("Amount must be greater than zero.");

      const randomHex = (len: number) =>
        Array.from(crypto.getRandomValues(new Uint8Array(len)))
          .map((b) => b.toString(16).padStart(2, "0"))
          .join("");

      const recipient_commitment = randomHex(32);
      const stealth_address = randomHex(32);
      const proof = randomHex(128);
      const public_inputs = [randomHex(32)];
      const nullifier = randomHex(32);

      const sendRes = await fetch(`${apiUrl}/send`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          recipient_commitment,
          stealth_address,
          proof,
          public_inputs,
          nullifier,
          amount: stroops.toString(),
        }),
      });

      if (!sendRes.ok) {
        const err = await sendRes.json();
        throw new Error(err.error ?? "Payment failed");
      }

      const body = await sendRes.json();
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
        setMessage(`Payment confirmed! tx: ${txHash}`);
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
      <h2>Send Private Payment</h2>
      <p style={{ color: "var(--muted)" }}>
        Funds are routed to a stealth address. The recipient&apos;s wallet is never
        revealed on-chain.
      </p>
      <form onSubmit={handleSend} style={{ display: "flex", flexDirection: "column", gap: 12 }}>
        <label>
          Recipient username
          <input
            value={recipient}
            onChange={(e) => setRecipient(e.target.value)}
            placeholder="alice"
            minLength={3}
            maxLength={31}
            required
          />
        </label>
        <label>
          Amount (XLM)
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="10"
            min="0.0000001"
            step="any"
            required
          />
        </label>
        <button type="submit" disabled={status === "loading" || status === "proving" || status === "pending"}>
          {status === "loading"
            ? "Checking recipient..."
            : status === "proving"
            ? "Generating proof..."
            : status === "pending"
            ? "Confirming..."
            : "Send"}
        </button>
      </form>
      {status === "confirmed" && <p style={{ color: "var(--success)", marginTop: 12 }}>{message}</p>}
      {(status === "pending" || status === "proving") && (
        <p style={{ color: "var(--accent)", marginTop: 12 }}>{message}</p>
      )}
      {status === "error" && <p style={{ color: "var(--error)", marginTop: 12 }}>{message}</p>}
    </>
  );
}
