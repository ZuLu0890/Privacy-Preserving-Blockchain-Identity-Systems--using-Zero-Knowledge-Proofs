"use client";

import { createContext, useContext, useState, useCallback, ReactNode } from "react";

interface WalletContextValue {
  publicKey: string | null;
  connecting: boolean;
  connect: () => Promise<void>;
  disconnect: () => void;
}

const WalletContext = createContext<WalletContextValue>({
  publicKey: null,
  connecting: false,
  connect: async () => {},
  disconnect: () => {},
});

export function useWallet() {
  return useContext(WalletContext);
}

export function WalletProvider({ children }: { children: ReactNode }) {
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);

  const connect = useCallback(async () => {
    setConnecting(true);
    try {
      const freighter = await import("@stellar/freighter-api");
      const address = await freighter.requestAccess();
      setPublicKey(address);
    } catch {
      setPublicKey(null);
    } finally {
      setConnecting(false);
    }
  }, []);

  const disconnect = useCallback(() => {
    setPublicKey(null);
  }, []);

  return (
    <WalletContext.Provider value={{ publicKey, connecting, connect, disconnect }}>
      {children}
    </WalletContext.Provider>
  );
}
