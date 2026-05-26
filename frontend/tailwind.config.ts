import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        display: ["var(--font-space-grotesk)", "sans-serif"],
        body: ["var(--font-outfit)", "sans-serif"],
        mono: ["var(--font-ibm-plex-mono)", "monospace"],
      },
      colors: {
        bg: "hsl(var(--bg))",
        "bg-elevated": "hsl(var(--bg-elevated))",
        panel: "hsl(var(--panel))",
        "panel-border": "hsl(var(--panel-border))",
        text: "hsl(var(--text))",
        "text-muted": "hsl(var(--text-muted))",
        primary: "hsl(var(--primary))",
        secondary: "hsl(var(--secondary))",
        success: "hsl(var(--success))",
        warning: "hsl(var(--warning))",
        danger: "hsl(var(--danger))",
        "skeleton-base": "hsl(var(--skeleton-base))",
        "skeleton-shimmer": "hsl(var(--skeleton-shimmer))",
        "variant-safe": "hsl(var(--variant-safe))",
        "variant-raw": "hsl(var(--variant-raw))",
        "variant-arena": "hsl(var(--variant-arena))",
        "variant-std": "hsl(var(--variant-std))",
        "variant-other": "hsl(var(--variant-other))",
        "chart-grid": "hsl(var(--chart-grid))",
        "chart-grid-subtle": "hsl(var(--chart-grid-subtle))",
        "chart-axis": "hsl(var(--chart-axis))",
        "chart-label": "hsl(var(--chart-label))",
      },
      borderRadius: {
        sm: "var(--radius-sm)",
        md: "var(--radius-md)",
        lg: "var(--radius-lg)",
      },
      boxShadow: {
        glow: "0 0 18px hsla(var(--primary) / 0.35)",
        panel: "0 12px 36px hsla(226 56% 4% / 0.38)",
      },
      keyframes: {
        shimmer: {
          "0%": { backgroundPosition: "-200% 0" },
          "100%": { backgroundPosition: "200% 0" },
        },
        "float-in": {
          "0%": { opacity: "0", transform: "translateY(10px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
      },
      animation: {
        shimmer: "shimmer 2.1s linear infinite",
        "float-in": "float-in 400ms ease-out",
      },
      spacing: {
        1: "0.25rem",
        2: "0.5rem",
        3: "0.75rem",
        4: "1rem",
        5: "1.25rem",
        6: "1.5rem",
        8: "2rem",
        10: "2.5rem",
        12: "3rem",
        16: "4rem",
      },
    },
  },
  plugins: [],
};

export default config;
