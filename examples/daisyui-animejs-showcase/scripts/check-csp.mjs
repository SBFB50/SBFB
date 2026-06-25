// Build-time conformance lint. Fails (exit 1) if any *runtime* asset would
// violate the SBFB sandbox CSP served by the daemon on every blob-serve
// response (single source: crates/nexus-core-rs/src/csp.rs → BLOB_SERVE_CSP):
//   default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:;
//   connect-src 'none'; worker-src 'none'; frame-src 'none'; object-src 'none';
//   base-uri 'none'; form-action 'none'; frame-ancestors *; sandbox allow-scripts
// + COOP same-origin + COEP require-corp.
//
// This is the JS sibling of the authoritative Rust gate
// `sbfb-factory::gates::run_gate_csp_authoring` (Sprint 79 Phase E): both
// consume the shared contract `crates/nexus-core-rs/csp-contract.json` (the
// CSP string + the `'none'` directive set + the CSS URL allowlist) instead of
// re-deriving it, so the two cannot drift. The Rust gate is the blocking gate
// in the publish pipeline; this script is the showcase's local dev check.
//
// Three tiers, because "no http substring" is too literal for compiled CSS:
//   - scanned (index.html, app.js, app.css): no network primitive, no
//     <script type=module>, AND every absolute URL must be allowlisted (the
//     MIT license banner we must keep, plus the SVG/xlink XML *namespaces*
//     that live inside `data:` URIs — identifiers the browser never fetches).
//   - vendored (vendor/anime.umd.js): no live network primitive (its minified
//     body legitimately contains the SVG namespace string + a license banner).
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, ".."); // showcase dir (the app under test)
const repoRoot = join(here, "..", "..", ".."); // scripts -> showcase -> examples -> repo
const read = (rel) => readFileSync(join(root, rel), "utf8");

// Shared CSP contract — single source = crates/nexus-core-rs/src/csp.rs,
// mirrored (and verified by a Rust test) into csp-contract.json. Consumed here
// rather than re-hardcoded so the JS lint and the Rust gate never diverge.
const contract = JSON.parse(
  readFileSync(join(repoRoot, "crates", "nexus-core-rs", "csp-contract.json"), "utf8"),
);
const CSS_URL_ALLOW = contract.css_url_allow;

// Network-reaching primitives + the HTML-literal breaches of the `'none'`
// directives — forbidden in every scanned asset. Mirrors the Rust gate's
// CSP_RULES. HTML/CSS patterns are case-insensitive; the JS-identifier
// patterns are case-sensitive (the API name) with `\b` to avoid `prefetcher`.
// Tag patterns require a boundary after the tag name ([\s/>] for presence,
// [\s/] for attribute tags — `/` catches the <script/src=…> HTML quirk) so a
// custom element (<iframe-foo>, <form-x>) is not a false positive — kept
// identical to the Rust gate's CSP_RULES.
const NETWORK = [
  // default-src: remote resource loads (absolute https? or protocol-relative //).
  [/<link[\s/][^>]*href\s*=\s*["']?(?:https?:|\/\/)/i, "remote <link href>"],
  [/<script[\s/][^>]*src\s*=\s*["']?(?:https?:|\/\/)/i, "remote <script src>"],
  // Only REMOTE @import is a violation (local relative imports resolve same-origin).
  [/@import\s+url\(\s*["']?(?:https?:|\/\/)/i, "remote CSS @import url()"],
  [/@import\s+["'](?:https?:|\/\/)/i, "remote CSS @import"],
  [/url\(\s*["']?(?:https?:|\/\/)/i, "remote url() asset"],
  // connect-src
  [/\bfetch\s*\(/, "fetch()"],
  [/\bXMLHttpRequest\b/, "XMLHttpRequest"],
  [/\bWebSocket\b/, "WebSocket"],
  [/\bEventSource\b/, "EventSource"],
  [/navigator\.sendBeacon\b/, "navigator.sendBeacon"],
  // worker-src
  [/new\s+Worker\b/, "Web Worker"],
  [/new\s+SharedWorker\b/, "SharedWorker"],
  [/\bimportScripts\s*\(/, "importScripts"],
  [/navigator\.serviceWorker/, "Service Worker"],
  // frame-src
  [/<iframe[\s/>]/i, "<iframe> (nested frame)"],
  [/createElement\(\s*["']iframe["']/i, "createElement('iframe')"],
  // object-src
  [/<object[\s/>]/i, "<object>"],
  [/<embed[\s/>]/i, "<embed>"],
  // base-uri
  [/<base[\s/][^>]*href\s*=/i, "<base href> (base-uri hijack)"],
  [/createElement\(\s*["']base["']/i, "createElement('base')"],
  // form-action: remote action target + dynamic setAttribute('action', …)
  [/<form[\s/][^>]*action\s*=\s*["']?(?:https?:|\/\/)/i, "<form action> to remote URL"],
  [/\.setAttribute\(\s*["']action["']/i, "setAttribute('action', …)"],
];

// COEP require-corp + opaque origin: ES module scripts fail. Vendor classic.
const MODULE_SCRIPT = /<script[\s/][^>]*type\s*=\s*["']module["']/i;

let failed = false;
const fail = (file, rule) => {
  console.error(`  FAIL  ${file}: ${rule}`);
  failed = true;
};

const checkNetwork = (file, text) => {
  for (const [re, label] of NETWORK) if (re.test(text)) fail(file, label);
};

// 1) scanned source: no network, no module scripts, every absolute URL allowlisted.
for (const rel of ["index.html", "app.js", "app.css"]) {
  const text = read(rel);
  checkNetwork(rel, text);
  if (MODULE_SCRIPT.test(text)) fail(rel, "<script type=module> (fails under COEP require-corp)");
  const urls = text.match(/https?:\/\/[^\s"')>]*/g) || [];
  for (const u of urls) {
    // Match on an origin/path boundary (exact, or prefix + "/") so a look-alike
    // host like https://tailwindcss.com.evil.com/ is NOT allowlisted.
    const ok = CSS_URL_ALLOW.some((a) => u === a || (u.startsWith(a) && u[a.length] === "/"));
    if (!ok) fail(rel, `non-allowlisted absolute URL: ${u}`);
  }
}

// 2) vendored bundle: no live network primitive (namespace + license strings kept).
checkNetwork("vendor/anime.umd.js", read("vendor/anime.umd.js"));

if (failed) {
  console.error("\nCSP conformance: FAILED");
  process.exit(1);
}
console.log("CSP conformance: OK — no CDN, no network, no workers in runtime assets.");
console.log("(absolute URLs limited to the allowlisted SVG/xlink namespaces + license banner.)");
