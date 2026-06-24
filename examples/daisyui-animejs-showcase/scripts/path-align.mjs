// Build-time ON-PATH verifier (NOT shipped). Catches the class of bug a plain
// trajectory sampler misses: "the moving dot animates, but is it actually ON its
// visible motion path?" For each {token, path} pair it samples over time and
// measures the SCREEN distance from the token center to the nearest projected
// point of the path (getScreenCTM + getPointAtLength). >~radius = off-path bug.
import { createServer } from "node:http";
import { readFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, extname } from "node:path";
import { chromium } from "playwright";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const out = join(root, ".shots", "align");
mkdirSync(out, { recursive: true });
const MIME = { ".html": "text/html; charset=utf-8", ".css": "text/css; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".json": "application/json; charset=utf-8" };
const ALLOW = /^\/(index\.html|app\.css|app\.js|SBFB\.json|vendor\/anime\.umd\.js)?$/;
const server = createServer((req, res) => {
  let p = decodeURIComponent(req.url.split("?")[0]); if (p === "/") p = "/index.html";
  if (!ALLOW.test(p)) { res.writeHead(404); return res.end("no"); }
  try { const b = readFileSync(join(root, p.slice(1))); res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" }); res.end(b); }
  catch { res.writeHead(404); res.end("404"); }
});
const PORT = 7796;
await new Promise((r) => server.listen(PORT, "127.0.0.1", r));

const PAIRS = [
  { name: "relais (sp-token)", token: "#sp-token", path: "#sp-path", scroll: "#sp-pipeline" },
  { name: "atelier (route-dot)", token: "#route-dot", path: "#route", scroll: "#atelier" },
  { name: "provenance (bp-token)", token: "#bp-token", path: "#bp-track", scroll: "#babel-prov" },
];

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 980 }, reducedMotion: "no-preference" });
await page.goto(`http://127.0.0.1:${PORT}/index.html`, { waitUntil: "load", timeout: 15000 });

const measure = (sel, pathSel) => {
  // injected into page; returns min screen distance token-center -> path polyline
  const tok = document.querySelector(sel);
  const path = document.querySelector(pathSel);
  if (!tok || !path) return null;
  const ctm = path.getScreenCTM();
  if (!ctm) return null;
  const L = path.getTotalLength();
  const r = tok.getBoundingClientRect();
  const cx = r.left + r.width / 2, cy = r.top + r.height / 2;
  let min = Infinity;
  for (let i = 0; i <= 240; i++) {
    const pt = path.getPointAtLength((L * i) / 240);
    const sx = ctm.a * pt.x + ctm.c * pt.y + ctm.e;
    const sy = ctm.b * pt.x + ctm.d * pt.y + ctm.f;
    const d = Math.hypot(sx - cx, sy - cy);
    if (d < min) min = d;
  }
  return { min: Math.round(min), tokenR: Math.round(r.width / 2) };
};

console.log("=== ON-PATH alignment (screen px from token center to nearest path point) ===");
for (const pair of PAIRS) {
  const sc = await page.$(pair.scroll);
  if (sc) { await sc.scrollIntoViewIfNeeded().catch(() => {}); await page.waitForTimeout(900); }
  const samples = [];
  for (let i = 0; i < 40; i++) {
    const m = await page.evaluate(([s, p]) => {
      const tok = document.querySelector(s), path = document.querySelector(p);
      if (!tok || !path) return null;
      const ctm = path.getScreenCTM(); if (!ctm) return null;
      const L = path.getTotalLength();
      const r = tok.getBoundingClientRect();
      const cx = r.left + r.width / 2, cy = r.top + r.height / 2;
      let min = Infinity;
      for (let k = 0; k <= 240; k++) {
        const pt = path.getPointAtLength((L * k) / 240);
        const sx = ctm.a * pt.x + ctm.c * pt.y + ctm.e;
        const sy = ctm.b * pt.x + ctm.d * pt.y + ctm.f;
        const d = Math.hypot(sx - cx, sy - cy);
        if (d < min) min = d;
      }
      return { min: Math.round(min), tokenR: Math.round(r.width / 2) };
    }, [pair.token, pair.path]);
    if (m) samples.push(m.min);
    await page.waitForTimeout(50);
  }
  if (!samples.length) { console.log(`  ${pair.name}: token/path not found`); continue; }
  samples.sort((a, b) => a - b);
  const med = samples[Math.floor(samples.length / 2)];
  const max = samples[samples.length - 1];
  const verdict = med <= 12 ? "ON path ✓" : `OFF path by ~${med}px  ✗`;
  console.log(`  ${pair.name.padEnd(22)} median=${med}px max=${max}px  -> ${verdict}`);
  // cropped screenshot of the svg region for the eye
  const svgEl = await page.$(pair.path);
  if (svgEl) { const host = await svgEl.evaluateHandle((e) => e.closest("svg")); const elh = host.asElement(); if (elh) await elh.screenshot({ path: join(out, `align-${pair.token.replace("#", "")}.png`) }).catch(() => {}); }
}
await browser.close();
server.close();
console.log(`\nCrops in ${out}`);
