# Sovereign Prompt — Web

Marketing landing page for [SovereignPrompt](https://github.com/BMC-INC/Sovereign-Prompt), the first MCP-native prompt optimization engine built entirely in Rust.

## Live

**https://sovereignclaw.com/promptgen**

## Stack

- Next.js 14 (React 18)
- Tailwind CSS 3
- SovereignClaw brand system (Inter, Syne, Michroma, Roboto Mono)

## Deployment

Deployed as a Vercel sub-app of sovereignclaw.com:

- **Vercel project**: sovereign-prompt-web
- **basePath**: `/promptgen` (set in next.config.mjs)
- **Proxied via**: SovereignClaw-web Vercel rewrites

```bash
npm install
npm run dev      # localhost:3000/promptgen
npm run build    # production build
```

## Part of SovereignClaw

This is a product page under the [SovereignClaw](https://sovereignclaw.com) ecosystem by ExecLayer Inc.
