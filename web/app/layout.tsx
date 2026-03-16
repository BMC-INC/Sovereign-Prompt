import type { Metadata } from "next";
import { Inter, Syne, Michroma, Roboto_Mono } from "next/font/google";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
  display: "swap",
});

const syne = Syne({
  subsets: ["latin"],
  weight: ["700", "800"],
  variable: "--font-syne",
  display: "swap",
});

const michroma = Michroma({
  subsets: ["latin"],
  weight: "400",
  variable: "--font-michroma",
  display: "swap",
});

const robotoMono = Roboto_Mono({
  subsets: ["latin"],
  variable: "--font-roboto-mono",
  display: "swap",
});

export const metadata: Metadata = {
  title: "SovereignPrompt — MCP-Native Prompt Optimization Engine",
  description:
    "The first MCP-native prompt optimization engine built entirely in Rust. Strip 30-75% wasted tokens, generate strategic variants, and get cryptographically signed audit trails — all locally.",
  keywords: [
    "MCP",
    "prompt optimization",
    "Rust",
    "Claude",
    "token savings",
    "SovereignClaw",
    "ExecLayer",
  ],
  openGraph: {
    title: "SovereignPrompt — Deterministic Prompt Optimization",
    description:
      "Strip 30-75% wasted tokens. 9 heuristic checks. 3 strategic variants. Zero telemetry. Built in Rust.",
    type: "website",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body
        className={`${inter.variable} ${syne.variable} ${michroma.variable} ${robotoMono.variable} antialiased`}
      >
        {children}
      </body>
    </html>
  );
}
