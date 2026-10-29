# SovereignPrompt video source

Remotion source for the project video and README animations. This is an optional
media-authoring project; it does not run the Rust services. The
[root README](../README.md) describes current product capabilities and limitations.
Review the historical promotional script before publishing a new video.

## Work locally

```bash
npm ci
npm run studio
```

`npm run render` exports the main video to `out/SovereignPrompt.mp4`.
`npm run render:readme` exports the two README animations and additionally requires
FFmpeg. These commands overwrite their named outputs.

## Validation and dependencies

```bash
npx tsc --noEmit
npx remotion bundle src/index.tsx
npm audit
```

The Remotion packages are pinned together to 4.0.539. A Zod 4.6.7 override addresses
GHSA-mrhx-c7xc-m3w5 in the 4.5.4 version pinned by that Remotion release. This remains
within Zod's major version, but Remotion emits a version-match warning because its
expected version is exact. Type checking, bundling, and a representative still from
each of the three compositions have been validated with the override. Remove it
when Remotion supplies the patched dependency and repeat those checks.

A still-render check does not validate a complete video export, audio mix, or the
product claims in the script.
