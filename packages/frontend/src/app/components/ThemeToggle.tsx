"use client";

import { useTheme } from "./ThemeProvider";

export function ThemeToggle() {
  const { theme, toggle } = useTheme();
  return (
    <button
      onClick={toggle}
      aria-label="Toggle dark mode"
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
      {theme === "dark" ? "Light" : "Dark"}
    </button>
  );
}
