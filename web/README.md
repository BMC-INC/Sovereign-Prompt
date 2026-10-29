# SovereignPrompt promotional website

This optional Next.js site presents the project. It is separate from the Rust MCP server, proxy, and built-in analytics dashboard. No live hosted URL is currently documented as available.

## Local development

```bash
cd web
npm ci
npm run dev
```

Open http://localhost:3000. Use `npm run build` for a production build and `npm start` to serve that build locally.

The [root README](../README.md) is the source of truth for supported service behavior, setup, and limitations. Review promotional claims here against current code and validation before publishing a site. Installing or running this website does not start the MCP or proxy services.
