import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "var(--black)",
        foreground: "var(--text)",
        sc: {
          black: "#030306",
          iron: "#0d0d0d",
          steel: "#1a1a1a",
          panel: "#161616",
          border: "#222222",
          "border-mid": "#2a2a2a",
          red: "#c8102e",
          "red-dim": "#8a0a1f",
          "red-bright": "#e01235",
          text: "#f0f0f0",
          "text-dim": "#b8b8b8",
          "text-muted": "#444444",
        },
      },
      fontFamily: {
        sans: ["var(--font-inter)", "Inter", "system-ui", "sans-serif"],
        heading: ["var(--font-syne)", "Syne", "sans-serif"],
        cta: ["var(--font-michroma)", "Michroma", "sans-serif"],
        mono: ["var(--font-roboto-mono)", "Roboto Mono", "monospace"],
      },
    },
  },
  plugins: [],
};
export default config;
