import type { Metadata } from "next";
import "./globals.css";
import { ThemeProvider } from "./components/ThemeProvider";
import { ThemeToggle } from "./components/ThemeToggle";
import { WalletProvider } from "./components/WalletProvider";
import { WalletButton } from "./components/WalletButton";

export const metadata: Metadata = {
  title: "ZK Identity — Stellar",
  description: "Privacy-preserving identity and payments on Stellar using ZK proofs",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <ThemeProvider>
          <WalletProvider>
            <header style={{ marginBottom: 32 }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <h1 style={{ fontSize: 20, fontWeight: 700 }}>ZK Identity</h1>
                <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                  <WalletButton />
                  <ThemeToggle />
                </div>
              </div>
              <nav style={{ display: "flex", gap: 16, marginTop: 8 }}>
                <a href="/">Register</a>
                <a href="/send">Send Payment</a>
              </nav>
            </header>
            <main>{children}</main>
          </WalletProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
