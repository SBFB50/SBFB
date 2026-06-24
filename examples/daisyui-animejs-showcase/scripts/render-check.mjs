// Build-time visual debug loop (NOT shipped). Serves the app over a tiny local
// static server, loads it in headless Chromium, captures console errors +
// uncaught exceptions, triggers the interactive code paths, and screenshots
// each section so animation bugs become visible. Run: node scripts/render-check.mjs
import { createServer } from "node:http";
import { readFileSync, mkdirSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, extname } from "node:path";
import { chromium } from "playwright";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const shots = join(root, ".shots");
mkdirSync(shots, { recursive: true });

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".woff2": "font/woff2",
};

// Serve only the runtime files; ignore node_modules etc.
const ALLOW = /^\/(index\.html|app\.css|app\.js|SBFB\.json|vendor\/anime\.umd\.js)?$/;
const server = createServer((req, res) => {
  let p = decodeURIComponent(req.url.split("?")[0]);
  if (p === "/") p = "/index.html";
  if (!ALLOW.test(p)) { res.writeHead(404); return res.end("no"); }
  try {
    const buf = readFileSync(join(root, p.slice(1)));
    res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" });
    res.end(buf);
  } catch {
    res.writeHead(404); res.end("404");
  }
});

const PORT = 7799;
await new Promise((r) => server.listen(PORT, "127.0.0.1", r));
const base = `http://127.0.0.1:${PORT}/index.html`;

const logs = [];
const errors = [];

const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: { width: 1280, height: 900 },
  reducedMotion: "no-preference",
  deviceScaleFactor: 1,
});
page.on("console", (m) => {
  const t = m.type();
  if (t === "error" || t === "warning") logs.push(`[${t}] ${m.text()}`);
});
page.on("pageerror", (e) => errors.push(`PAGEERROR: ${e.message}`));
page.on("requestfailed", (r) => errors.push(`REQFAIL: ${r.url()} ${r.failure()?.errorText}`));

await page.goto(base, { waitUntil: "load", timeout: 15000 });
await page.waitForTimeout(800);

const SECTIONS = [
  "contenu", "design-system", "composants", "motion", "atelier", "sbfb-vivant", "babel", "combos", "sismographe", "engrenages", "css-natif",
];

// Pass 1 — scroll each section into view, let it animate, screenshot.
for (const id of SECTIONS) {
  const el = await page.$(`#${id}`);
  if (!el) { errors.push(`MISSING SECTION #${id}`); continue; }
  await el.scrollIntoViewIfNeeded().catch(() => {});
  await page.waitForTimeout(1400);
  await page.screenshot({ path: join(shots, `sec-${id}.png`) });
}

// Pass 2 — trigger interactions to exercise handlers.
const clicks = async (sel, label) => {
  const els = await page.$$(sel);
  for (const e of els) { await e.click({ timeout: 1500 }).catch((err) => errors.push(`CLICKFAIL ${label}: ${err.message}`)); await page.waitForTimeout(150); }
};
// theme switch
await page.selectOption("#theme", "dracula").catch(() => {});
await page.waitForTimeout(400);
// gallery filter
await clicks('#filters [data-filter]', "filter");
// babel
await clicks('#babel-langs [data-pair]', "babel-pair");
await clicks('[data-babel-run]', "babel-constellation-run");
await clicks('[data-babel-step]', "babel-glyph-step");
await clicks('[data-babel-prov-run]', "babel-prov-run");
// sbfb
await clicks('[data-gauge-replay]', "gauge");
await clicks('[data-sign-run]', "sign");
await clicks('[data-gc-task]', "gc-task");
await page.waitForTimeout(600);

// targeted screenshots of the trickier components
const TARGETS = ["sp-pipeline", "babel-card", "babel-morph", "babel-prov", "vc-ladder", "sc-coverage"];
for (const id of TARGETS) {
  const el = await page.$(`#${id}`);
  if (!el) continue;
  await el.scrollIntoViewIfNeeded().catch(() => {});
  await page.waitForTimeout(900);
  await el.screenshot({ path: join(shots, `comp-${id}.png`) }).catch((e) => errors.push(`SHOTFAIL ${id}: ${e.message}`));
}

await page.screenshot({ path: join(shots, "full.png"), fullPage: true });
await browser.close();
server.close();

console.log("=== CONSOLE errors/warnings ===");
console.log(logs.length ? logs.join("\n") : "(none)");
console.log("\n=== PAGE errors / failures ===");
console.log(errors.length ? errors.join("\n") : "(none)");
console.log(`\nScreenshots in ${shots}`);
