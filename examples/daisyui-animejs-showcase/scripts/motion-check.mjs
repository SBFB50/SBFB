// Build-time MOTION debug loop (NOT shipped). Screenshots can't show motion, so
// this (1) samples real per-frame trajectories via requestAnimationFrame inside
// the page to catch logic bugs (teleport jumps, NaN, out-of-bounds, stuck/never-
// reaches, jank), and (2) builds flipbook contact-sheets (a grid of timed frames)
// so motion progression is visible in one image. Run: node scripts/motion-check.mjs
import { createServer } from "node:http";
import { readFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, extname } from "node:path";
import { chromium } from "playwright";
import sharp from "sharp";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const out = join(root, ".shots", "motion");
mkdirSync(out, { recursive: true });

const MIME = { ".html": "text/html; charset=utf-8", ".css": "text/css; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".json": "application/json; charset=utf-8" };
const ALLOW = /^\/(index\.html|app\.css|app\.js|SBFB\.json|vendor\/anime\.umd\.js)?$/;
const server = createServer((req, res) => {
  let p = decodeURIComponent(req.url.split("?")[0]); if (p === "/") p = "/index.html";
  if (!ALLOW.test(p)) { res.writeHead(404); return res.end("no"); }
  try { const b = readFileSync(join(root, p.slice(1))); res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" }); res.end(b); }
  catch { res.writeHead(404); res.end("404"); }
});
const PORT = 7798;
await new Promise((r) => server.listen(PORT, "127.0.0.1", r));

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 900 }, reducedMotion: "no-preference" });
const pageErrors = [];
page.on("pageerror", (e) => pageErrors.push(e.message));
await page.goto(`http://127.0.0.1:${PORT}/index.html`, { waitUntil: "load", timeout: 15000 });
await page.waitForTimeout(600);

// Bring the motion-heavy sections into view so their IntersectionObservers start.
for (const id of ["motion", "atelier", "sbfb-vivant", "babel"]) {
  const el = await page.$(`#${id}`); if (el) { await el.scrollIntoViewIfNeeded().catch(() => {}); await page.waitForTimeout(500); }
}
// trigger one-shot anims so they are sampled mid-flight
await page.$$eval("[data-sign-run],[data-gauge-replay],[data-babel-run],[data-babel-prov-run]", (els) => els.forEach((e) => e.click()));
await page.waitForTimeout(200);

// ---- (1) per-frame trajectory sampling -------------------------------------
const SAMPLE = {
  "sp-token (relais)": "#sp-token",
  "babel-packet": "#babel-packet",
  "route-dot (atelier)": "#route-dot",
  "wave tile": "#wave i:nth-child(45)",
  "sd-node (orbite)": "#sd-stage .sd-node",
};
const traj = await page.evaluate(async (sel) => {
  const els = Object.fromEntries(Object.entries(sel).map(([k, s]) => [k, document.querySelector(s)]));
  const glyph = document.querySelector("#babel-glyph");
  const gauge = document.querySelector("#pc-gauge");
  const rec = {}; for (const k of Object.keys(els)) rec[k] = [];
  const dSeq = []; const valSeq = []; const deltas = []; let last = performance.now();
  const center = (el) => { const r = el.getBoundingClientRect(); return [Math.round(r.left + r.width / 2), Math.round(r.top + r.height / 2)]; };
  await new Promise((resolve) => {
    const t0 = performance.now();
    const tick = () => {
      const now = performance.now(); deltas.push(Math.round(now - last)); last = now;
      for (const [k, el] of Object.entries(els)) if (el) rec[k].push(center(el));
      if (glyph) dSeq.push(glyph.getAttribute("d") || "");
      if (gauge) valSeq.push(parseFloat(getComputedStyle(gauge).getPropertyValue("--value")) || 0);
      if (now - t0 < 3000) requestAnimationFrame(tick); else resolve();
    };
    requestAnimationFrame(tick);
  });
  return { rec, dSeq, valSeq, deltas };
}, SAMPLE);

const analyze = (pts) => {
  if (!pts || pts.length < 2) return "n/a";
  let moved = 0, maxJump = 0;
  for (let i = 1; i < pts.length; i++) {
    const d = Math.hypot(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]);
    maxJump = Math.max(maxJump, d);
  }
  const spanX = Math.max(...pts.map((p) => p[0])) - Math.min(...pts.map((p) => p[0]));
  const spanY = Math.max(...pts.map((p) => p[1])) - Math.min(...pts.map((p) => p[1]));
  moved = Math.max(spanX, spanY);
  const nan = pts.some((p) => Number.isNaN(p[0]) || Number.isNaN(p[1]));
  return `frames=${pts.length} amplitude=${moved}px maxJump/frame=${Math.round(maxJump)}px${nan ? " NaN!" : ""}`;
};

console.log("=== TRAJECTORIES (3s @ rAF) ===");
for (const [k] of Object.entries(SAMPLE)) console.log(`  ${k.padEnd(22)} ${analyze(traj.rec[k])}`);
const dChanged = new Set(traj.dSeq).size;
const dBad = traj.dSeq.some((d) => d.includes("NaN") || (d && !d.trim().startsWith("M")));
console.log(`  glyph 'd' morph        distinct=${dChanged} validStart=${!dBad}${dBad ? " BAD-d!" : ""}`);
const vMin = Math.min(...traj.valSeq), vMax = Math.max(...traj.valSeq);
console.log(`  proof gauge --value    ${vMin} -> ${vMax}`);
const ds = traj.deltas.slice(1);
const p95 = ds.sort((a, b) => a - b)[Math.floor(ds.length * 0.95)] || 0;
const jank = ds.filter((d) => d > 50).length;
console.log(`  frame delta p95=${p95}ms  jank(>50ms)=${jank}/${ds.length}`);
console.log(`  pageerrors: ${pageErrors.length ? pageErrors.join(" | ") : "none"}`);

// ---- (2) flipbook contact-sheets -------------------------------------------
const flip = async (id, { frames = 9, interval = 220, cols = 3 } = {}) => {
  const el = await page.$(`#${id}`); if (!el) return console.log(`  flip ${id}: missing`);
  await el.scrollIntoViewIfNeeded().catch(() => {});
  await page.waitForTimeout(300);
  const bufs = [];
  for (let i = 0; i < frames; i++) { bufs.push(await el.screenshot()); await page.waitForTimeout(interval); }
  const meta = await sharp(bufs[0]).metadata();
  const w = meta.width, h = meta.height, rows = Math.ceil(frames / cols), gap = 6;
  const sheet = sharp({ create: { width: w * cols + gap * (cols - 1), height: h * rows + gap * (rows - 1), channels: 4, background: { r: 17, g: 17, b: 17, alpha: 1 } } });
  const comp = await Promise.all(bufs.map(async (b, i) => ({ input: await sharp(b).resize(w, h, { fit: "fill" }).png().toBuffer(), left: (i % cols) * (w + gap), top: Math.floor(i / cols) * (h + gap) })));
  await sheet.composite(comp).png().toFile(join(out, `flip-${id}.png`));
  console.log(`  flip ${id}: ${frames} frames -> flip-${id}.png`);
};

console.log("\n=== FLIPBOOKS ===");
await flip("wave", { frames: 9, interval: 180 });
await flip("sp-pipeline", { frames: 9, interval: 320 });
await flip("babel-card", { frames: 9, interval: 280 });
await flip("babel-morph", { frames: 9, interval: 320 });

await browser.close();
server.close();
console.log(`\nContact sheets in ${out}`);
