// Build-time conformance gate. Fails (exit 1) if any *runtime* asset would
// violate the SBFB sandbox CSP:
//   default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:;
//   connect-src 'none'; worker-src 'none'; frame-src 'none'; object-src 'none';
//
// Three tiers, because "no http substring" is too literal for compiled CSS:
//   - authored (index.html, app.js): ZERO absolute http(s) URL, zero network.
//   - compiled (app.css): no remote url()/@import/network, AND every absolute
//     URL must be in an allowlist (the MIT license banner we must keep, plus
//     the SVG/xlink XML *namespaces* that live inside `data:` URIs — these are
//     identifiers the browser never fetches).
//   - vendored (anime.esm.js): no live network primitives (its minified body
//     legitimately contains the SVG namespace string).
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const read = (rel) => readFileSync(join(root, rel), "utf8");

// Network-reaching primitives — forbidden everywhere.
const NETWORK = [
  [/<link[^>]+href\s*=\s*["']?https?:/i, "remote <link href>"],
  [/<script[^>]+src\s*=\s*["']?https?:/i, "remote <script src>"],
  [/@import\s+url\(/i, "CSS @import url()"],
  [/url\(\s*["']?https?:/i, "remote url() asset"],
  [/\bfetch\s*\(/, "fetch()"],
  [/\bXMLHttpRequest\b/, "XMLHttpRequest"],
  [/\bWebSocket\b/, "WebSocket"],
  [/\bEventSource\b/, "EventSource"],
  [/navigator\.sendBeacon\b/, "navigator.sendBeacon"],
  [/new\s+Worker\b/, "Web Worker"],
  [/new\s+SharedWorker\b/, "SharedWorker"],
  [/\bimportScripts\s*\(/, "importScripts"],
  [/navigator\.serviceWorker/, "Service Worker"],
];

// Absolute URLs allowed to appear in compiled app.css (non-fetched).
const CSS_URL_ALLOW = [
  "http://www.w3.org/2000/svg", // SVG xmlns inside data: URIs
  "http://www.w3.org/1999/xlink", // xlink xmlns inside data: URIs
  "https://tailwindcss.com", // MIT license banner (must be preserved)
];

let failed = false;
const fail = (file, rule) => {
  console.error(`  FAIL  ${file}: ${rule}`);
  failed = true;
};

const checkNetwork = (file, text) => {
  for (const [re, label] of NETWORK) if (re.test(text)) fail(file, label);
};

// 1) authored: strict — zero absolute http(s) URL + zero network primitive.
for (const rel of ["index.html", "app.js"]) {
  const text = read(rel);
  checkNetwork(rel, text);
  if (/https?:\/\//.test(text)) fail(rel, "absolute http(s) URL (authored files must be clean)");
}

// 2) compiled CSS: no network + every absolute URL allowlisted.
{
  const rel = "app.css";
  const text = read(rel);
  checkNetwork(rel, text);
  const urls = text.match(/https?:\/\/[^\s"')]*/g) || [];
  for (const u of urls) {
    if (!CSS_URL_ALLOW.some((a) => u.startsWith(a))) fail(rel, `unexpected absolute URL: ${u}`);
  }
}

// 3) vendored bundle: no live network primitives.
checkNetwork("vendor/anime.umd.js", read("vendor/anime.umd.js"));

if (failed) {
  console.error("\nCSP conformance: FAILED");
  process.exit(1);
}
console.log("CSP conformance: OK — no CDN, no network, no workers in runtime assets.");
console.log("(app.css contains only the MIT license banner + SVG xmlns inside data: URIs — neither is fetched.)");
