// Build-time dynamics probe for the "Sismographe" section (NOT shipped).
// Serves the app, scrolls the seismograph into view, samples window.__seis over
// ~13s, and reports: peak freeze, min engine.speed, number of freeze pulses
// (felt quakes), self-limiting behaviour (freeze returns toward 0 between
// pulses), and that engine.speed resets to 1 once the section leaves view.
// Run: node scripts/seis-probe.mjs
import { createServer } from "node:http";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, extname } from "node:path";
import { chromium } from "playwright";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const MIME = { ".html": "text/html", ".css": "text/css", ".js": "text/javascript", ".json": "application/json" };
const ALLOW = /^\/(index\.html|app\.css|app\.js|SBFB\.json|vendor\/anime\.umd\.js)?$/;
const server = createServer((req, res) => {
  let p = decodeURIComponent(req.url.split("?")[0]);
  if (p === "/") p = "/index.html";
  if (!ALLOW.test(p)) { res.writeHead(404); return res.end("no"); }
  try { res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" }); res.end(readFileSync(join(root, p.slice(1)))); }
  catch { res.writeHead(404); res.end("404"); }
});
const PORT = 7801;
await new Promise((r) => server.listen(PORT, "127.0.0.1", r));

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 900 }, reducedMotion: "no-preference" });
const errs = [];
page.on("pageerror", (e) => errs.push(e.message));
await page.goto(`http://127.0.0.1:${PORT}/index.html`, { waitUntil: "load" });
await page.$eval("#sismographe", (el) => el.scrollIntoView({ block: "center" }));
await page.waitForTimeout(400);

const samples = [];
const START = Date.now();
while (Date.now() - START < 13000) {
  const s = await page.evaluate(() => window.__seis || null);
  if (s) samples.push({ t: Date.now() - START, ...s });
  await page.waitForTimeout(80);
}

// Scroll away → engine.speed must reset to 1.
await page.$eval("#contenu", (el) => el.scrollIntoView({ block: "center" }));
await page.waitForTimeout(900);
const speedAfterLeave = await page.evaluate(() => window.anime.engine.speed);

await browser.close();
server.close();

const freezes = samples.map((s) => s.freeze);
const speeds = samples.map((s) => s.speed);
const peakFreeze = Math.max(...freezes);
const minSpeed = Math.min(...speeds);
const maxQuakes = Math.max(...samples.map((s) => s.quakes));
// Count pulses: rising crossings of 0.5; "returns toward 0" = a sample < 0.08 between pulses.
let pulses = 0, last = 0, dippedLow = true, returnedBetween = true;
for (const f of freezes) {
  if (last < 0.5 && f >= 0.5) { pulses++; if (!dippedLow) returnedBetween = false; dippedLow = false; }
  if (f < 0.08) dippedLow = true;
  last = f;
}
// chatter metric: stddev of freeze deltas while elevated
let chatter = 0, n = 0;
for (let i = 1; i < freezes.length; i++) if (freezes[i] > 0.2 || freezes[i - 1] > 0.2) { chatter += Math.abs(freezes[i] - freezes[i - 1]); n++; }
const avgStep = n ? chatter / n : 0;

console.log("=== SEISMOGRAPHE — sonde dynamique ===");
console.log("samples:", samples.length, "| pageerrors:", errs.length ? errs.join(" | ") : "(none)");
console.log("peak freeze:", peakFreeze.toFixed(3), "| min engine.speed:", minSpeed.toFixed(3));
console.log("pulses (gel>=0.5):", pulses, "| secousses ressenties (compteur app):", maxQuakes);
console.log("auto-limitant (retour <0.08 entre pulses):", returnedBetween);
console.log("avg |Δfreeze|/frame en zone active:", avgStep.toFixed(3), "(chatter si > ~0.15)");
console.log("engine.speed après sortie de vue:", speedAfterLeave, "(doit être 1)");
const ok = errs.length === 0 && peakFreeze >= 0.6 && minSpeed <= 0.25 && pulses >= 1 && returnedBetween && speedAfterLeave === 1 && avgStep <= 0.18;
console.log(ok ? "\nVERDICT: PASS" : "\nVERDICT: A AJUSTER");
// timeline compacte (1 ligne / ~1s)
const line = [];
for (let i = 0; i < samples.length; i += 12) line.push(samples[i].freeze.toFixed(2));
console.log("freeze timeline:", line.join(" "));
