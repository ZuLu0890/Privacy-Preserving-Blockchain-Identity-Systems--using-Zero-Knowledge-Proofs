"use client";

import { useWallet } from "./WalletProvider";

export function WalletButton() {
  const { publicKey, connecting, connect, disconnect } = useWallet();

  if (publicKey) {
    const short = publicKey.slice(0, 6) + "..." + publicKey.slice(-4);
    return (
      <button
        onClick={disconnect}
        style={{
          background: "none",
          border: "1px solid currentColor",
          borderRadius: 6,
          padding: "4px 10px",
          cursor: "pointer",
          fontSize: 14,
          color: "inherit",
        }}
      >
        {short}
      </button>
    );
  }

  return (
    <button
      onClick={connect}
      disabled={connecting}
      style={{
        background: "none",
        border: "1px solid currentColor",
        borderRadius: 6,
        padding: "4px 10px",
        cursor: "pointer",
        fontSize: 14,
        color: "inherit",
      }}
    >
      {connecting ? "Connecting..." : "Connect Wallet"}
    </button>
  );
}
