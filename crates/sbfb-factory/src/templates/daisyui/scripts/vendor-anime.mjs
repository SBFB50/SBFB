// Build-time only. Copies the single-file anime.js v4 UMD bundle out of
// node_modules into ./vendor so the published archive carries it with ZERO
// network dependency (the SBFB sandbox CSP is `connect-src 'none'`).
//
// Why the UMD (classic) bundle and NOT the ESM build: the iframe runs at an
// OPAQUE origin (sandbox without allow-same-origin) under COEP require-corp.
// An ES module (loaded with type=module) is fetched in CORS mode, which an
// opaque-origin document cannot satisfy for its own assets. A classic script
// is no-cors and loads under `default-src 'self'`. This mirrors every shipped
// SBFB app: external CSS via a stylesheet link + classic JS. The UMD bundle
// exposes a single global `anime` with all named members.
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");

const pkg = JSON.parse(
  readFileSync(join(root, "node_modules/animejs/package.json"), "utf8"),
);
const src = join(root, "node_modules/animejs/dist/bundles/anime.umd.min.js");
const outDir = join(root, "vendor");
const out = join(outDir, "anime.umd.js");

const header =
  `/* anime.js v${pkg.version} — MIT (c) Julian Garnier — juliangarnier/anime\n` +
  `   Vendored build-time from node_modules into the SBFB archive. Do not edit. */\n`;

mkdirSync(outDir, { recursive: true });
writeFileSync(out, header + readFileSync(src, "utf8"));
console.log(`vendored anime.js v${pkg.version} -> vendor/anime.umd.js`);
