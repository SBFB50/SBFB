// Build-time DEEP motion audit (NOT shipped). Three OSS signals:
//  (1) Lighthouse `non-composited-animations` + CLS  → objective jank/quality of CSS anims.
//  (2) Playwright page.clock                          → DETERMINISTIC frames (same instants each run).
//  (3) per-component motion profiles (positions / d / --value at each step) → JSON for analysis.
// Outputs: .shots/audit/flip-<id>.png, motion-profile.json, lighthouse-audit.json
import { createServer } from "node:http";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, extname } from "node:path";
import { chromium } from "playwright";
import sharp from "sharp";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const out = join(root, ".shots", "audit");
mkdirSync(out, { recursive: true });

const MIME = { ".html": "text/html; charset=utf-8", ".css": "text/css; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".json": "application/json; charset=utf-8" };
const ALLOW = /^\/(index\.html|app\.css|app\.js|SBFB\.json|vendor\/anime\.umd\.js)?$/;
const server = createServer((req, res) => {
  let p = decodeURIComponent(req.url.split("?")[0]); if (p === "/") p = "/index.html";
  if (!ALLOW.test(p)) { res.writeHead(404); return res.end("no"); }
  try { const b = readFileSync(join(root, p.slice(1))); res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" }); res.end(b); }
  catch { res.writeHead(404); res.end("404"); }
});
const PORT = 7797;
await new Promise((r) => server.listen(PORT, "127.0.0.1", r));
const URL = `http://127.0.0.1:${PORT}/index.html`;

// ─── (1) Lighthouse: non-composited animations + CLS ────────────────────────
const lighthouseAudit = async () => {
  try {
    const lighthouse = (await import("lighthouse")).default;
    const chromeLauncher = await import("chrome-launcher");
    const chrome = await chromeLauncher.launch({ chromeFlags: ["--headless=new", "--no-sandbox", "--disable-gpu"] });
    const res = await lighthouse(URL, { port: chrome.port, output: "json", logLevel: "error", onlyAudits: ["non-composited-animations", "cumulative-layout-shift", "total-blocking-time"] });
    await chrome.kill();
    const a = res.lhr.audits;
    const nca = a["non-composited-animations"];
    const report = {
      nonComposited: {
        score: nca?.score,
        displayValue: nca?.displayValue || null,
        offenders: (nca?.details?.items || []).map((it) => ({
          node: it.node?.snippet || it.node?.selector || "?",
          reasons: (it.animations || []).map((x) => x.failureReasonsMask ?? x.name).join(", "),
        })),
      },
      cls: { score: a["cumulative-layout-shift"]?.score, value: a["cumulative-layout-shift"]?.numericValue },
      tbt: { score: a["total-blocking-time"]?.score, value: a["total-blocking-time"]?.numericValue },
    };
    writeFileSync(join(out, "lighthouse-audit.json"), JSON.stringify(report, null, 2));
    return report;
  } catch (e) {
    return { error: String(e.message || e) };
  }
};

console.log("=== (1) Lighthouse ===");
const lh = await lighthouseAudit();
if (lh.error) console.log("  lighthouse skipped:", lh.error);
else {
  console.log(`  non-composited-animations: score=${lh.nonComposited.score} offenders=${lh.nonComposited.offenders.length}`);
  for (const o of lh.nonComposited.offenders) console.log(`    - ${o.node}  [${o.reasons}]`);
  console.log(`  CLS=${lh.cls.value?.toFixed(4)} (score ${lh.cls.score})  TBT=${Math.round(lh.tbt.value || 0)}ms`);
}

// ─── (2)+(3) Deterministic flipbooks + motion profiles via page.clock ────────
const COMPONENTS = [
  { id: "wave", trigger: null, samples: { tile: "#wave i:nth-child(45)" }, attr: { tile: "transform" } },
  { id: "sp-pipeline", trigger: null, samples: { token: "#sp-token" } },
  { id: "babel-card", trigger: "[data-babel-run]", samples: { packet: "#babel-packet" } },
  { id: "babel-morph", trigger: null, samples: { glyph: "#babel-glyph" }, attr: { glyph: "d" } },
  { id: "babel-prov", trigger: "[data-babel-prov-run]", samples: { token: "#bp-token" } },
];
const STEP = 130, STEPS = 12, COLS = 4;
const profile = {};

const auditComponent = async (browser, c) => {
  const page = await browser.newPage({ viewport: { width: 1280, height: 980 }, reducedMotion: "no-preference" });
  const errs = [];
  page.on("pageerror", (e) => errs.push(e.message));
  await page.clock.install();
  await page.goto(URL, { waitUntil: "load", timeout: 15000 });
  await page.clock.runFor(50);
  const el = await page.$(`#${c.id}`);
  if (!el) { await page.close(); return { error: "missing" }; }
  await el.scrollIntoViewIfNeeded().catch(() => {});
  await page.clock.runFor(60); // let IntersectionObserver register + start the timeline
  if (c.trigger) { const t = await page.$(c.trigger); if (t) await t.click().catch(() => {}); }

  const frames = [];
  const series = {};
  for (const k of Object.keys(c.samples)) series[k] = [];
  for (let i = 0; i < STEPS; i++) {
    await page.clock.runFor(STEP);
    frames.push(await el.screenshot());
    const snap = await page.evaluate(({ samples, attr }) => {
      const o = {};
      for (const [k, sel] of Object.entries(samples)) {
        const e = document.querySelector(sel);
        if (!e) { o[k] = null; continue; }
        if (attr && attr[k]) { o[k] = e.getAttribute(attr[k]) || ""; }
        else { const r = e.getBoundingClientRect(); o[k] = [Math.round(r.left + r.width / 2), Math.round(r.top + r.height / 2)]; }
      }
      return o;
    }, { samples: c.samples, attr: c.attr || {} });
    for (const k of Object.keys(c.samples)) series[k].push(snap[k]);
  }

  // contact sheet
  const meta = await sharp(frames[0]).metadata();
  const w = meta.width, h = meta.height, rows = Math.ceil(STEPS / COLS), gap = 5;
  const comp = await Promise.all(frames.map(async (b, i) => ({ input: await sharp(b).resize(w, h, { fit: "fill" }).png().toBuffer(), left: (i % COLS) * (w + gap), top: Math.floor(i / COLS) * (h + gap) })));
  await sharp({ create: { width: w * COLS + gap * (COLS - 1), height: h * rows + gap * (rows - 1), channels: 4, background: { r: 17, g: 17, b: 17, alpha: 1 } } }).composite(comp).png().toFile(join(out, `flip-${c.id}.png`));

  await page.close();

  // analysis per series
  const analysis = {};
  for (const [k, seq] of Object.entries(series)) {
    if (c.attr && c.attr[k]) {
      const distinct = new Set(seq).size;
      const bad = seq.some((d) => typeof d === "string" && (d.includes("NaN") || (d && !d.trim().startsWith("M") && c.id === "babel-morph")));
      analysis[k] = { kind: "attr", distinct, changed: distinct > 1, bad, sample: seq.slice(0, 3) };
    } else {
      const pts = seq.filter(Boolean);
      let maxJump = 0; for (let i = 1; i < pts.length; i++) maxJump = Math.max(maxJump, Math.hypot(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]));
      const xs = pts.map((p) => p[0]), ys = pts.map((p) => p[1]);
      const amp = Math.max(Math.max(...xs) - Math.min(...xs), Math.max(...ys) - Math.min(...ys));
      const nan = pts.some((p) => Number.isNaN(p[0]) || Number.isNaN(p[1]));
      analysis[k] = { kind: "pos", frames: pts.length, amplitudePx: amp, maxJumpPx: Math.round(maxJump), nan, points: pts };
    }
  }
  return { analysis, pageerrors: errs };
};

console.log("\n=== (2)+(3) Deterministic per-component (page.clock) ===");
const browser = await chromium.launch();
for (const c of COMPONENTS) {
  const r = await auditComponent(browser, c).catch((e) => ({ error: String(e.message || e) }));
  profile[c.id] = r;
  if (r.error) { console.log(`  ${c.id}: ERROR ${r.error}`); continue; }
  const parts = Object.entries(r.analysis).map(([k, a]) =>
    a.kind === "attr" ? `${k}: distinct=${a.distinct} changed=${a.changed}${a.bad ? " BAD" : ""}`
      : `${k}: amp=${a.amplitudePx}px maxJump=${a.maxJumpPx}px${a.nan ? " NaN" : ""}`);
  console.log(`  ${c.id}: ${parts.join(" | ")}${r.pageerrors.length ? " ERR:" + r.pageerrors.join(";") : ""}`);
}
await browser.close();
server.close();

writeFileSync(join(out, "motion-profile.json"), JSON.stringify({ lighthouse: lh, components: profile }, null, 2));
console.log(`\nArtifacts in ${out} (flip-*.png, motion-profile.json, lighthouse-audit.json)`);
