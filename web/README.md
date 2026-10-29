# SovereignPrompt promotional website

This optional Next.js site presents the project. It is separate from the Rust MCP server, proxy, and built-in analytics dashboard. No live hosted URL is currently documented as available.

## Local development

```bash
cd web
npm ci
npm run dev
```

Open [localhost:3000/sovereign-prompt](http://localhost:3000/sovereign-prompt). Use `npm run build` for a production build and `npm start` to serve that build locally.

The [root README](../README.md) is the source of truth for supported service behavior, setup, and limitations. Review promotional claims here against current code and validation before publishing a site. Installing or running this website does not start the MCP or proxy services.

## Dependency maintenance

Next.js and its ESLint configuration are pinned together to 15.5.28. React remains
on version 18. The scoped PostCSS 8.5.32 override fixes advisories in Next's bundled
PostCSS dependency while preserving its major API version. Keep the override until
Next supplies a patched version, and rerun the production build, lint, and
`npm audit` after dependency updates.
