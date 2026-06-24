// Build-time dynamics probe for the "Engrenages / pipeline de shards" section.
// Confirms: gears actually rotate, the torque wave propagates upstream→downstream
// (drive heats before the tail), a bottleneck appears, RunProofs accumulate.
// Run: node scripts/gears-probe.mjs
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
  let p = decodeURIComponent(req.url.split("?")[0]); if (p === "/") p = "/index.html";
  if (!ALLOW.test(p)) { res.writeHead(404); return res.end("no"); }
  try { res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" }); res.end(readFileSync(join(root, p.slice(1)))); }
  catch { res.writeHead(404); res.end("404"); }
});
const PORT = 7803;
await new Promise((r) => server.listen(PORT, "127.0.0.1", r));

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 900 }, reducedMotion: "no-preference" });
const errs = [];
page.on("pageerror", (e) => errs.push(e.message));
await page.goto(`http://127.0.0.1:${PORT}/index.html`, { waitUntil: "load" });
await page.$eval("#engrenages", (el) => el.scrollIntoView({ block: "center" }));
await page.waitForTimeout(400);

const rotAngles = (sel) => page.$$eval(sel, (els) => els.map((e) => {
  const m = (e.getAttribute("transform") || "").match(/rotate\(([-0-9.]+)/);
  return m ? parseFloat(m[1]) : 0;
}));
const rot0 = await rotAngles(".gear-rot");

const samples = [];
const heatFirst = new Array(6).fill(null);
let shot = false;
const START = Date.now();
while (Date.now() - START < 11000) {
  const g = await page.evaluate(() => window.__gears || null);
  const heats = await page.$$eval(".gear", (els) => els.map((e) => parseFloat(e.style.getPropertyValue("--heat")) || 0));
  const tnow = Date.now() - START;
  if (!shot && heats.some((h) => h > 0.5)) {
    const fills = await page.$$eval(".gear-teeth", (els) => els.map((e) => getComputedStyle(e).fill));
    console.log("HOT heats:", heats.map((h) => h.toFixed(2)).join(" "));
    console.log("HOT fills:", fills.join(" | "));
    await page.$eval("#gears-stage", (el) => el.scrollIntoView({ block: "center" }));
    await page.screenshot({ path: join(root, ".shots", "gears-hot.png") }); shot = true;
  }
  heats.forEach((h, i) => { if (heatFirst[i] === null && h > 0.25) heatFirst[i] = tnow; });
  if (g) samples.push({ t: tnow, ...g });
  await page.waitForTimeout(70);
}
const rot1 = await rotAngles(".gear-rot");
await browser.close();
server.close();

const spun = rot0.map((a, i) => Math.abs(a - rot1[i])).filter((d) => d > 1).length;
const maxStrainSeen = Math.max(...samples.map((s) => s.maxStrain));
const maxProofs = Math.max(...samples.map((s) => s.proofs));
const bottlenecks = new Set(samples.map((s) => s.bottleneck));
const stages = new Set(samples.map((s) => s.stage));
const allHeated = heatFirst.every((v) => v !== null);
const propagates = stages.size >= 5; // le front de tension (argmax, gain-independant) balaie le train

console.log("=== ENGRENAGES — sonde dynamique ===");
console.log("samples:", samples.length, "| pageerrors:", errs.length ? errs.join(" | ") : "(none)");
console.log("roues qui tournent (Δangle>1°):", spun, "/", rot0.length);
console.log("max tension vue:", maxStrainSeen.toFixed(2), "(ondes si > ~0.5)");
console.log("RunProofs accumulés:", maxProofs);
console.log("goulots distincts visités:", [...bottlenecks].join(","), "| étapes actives distinctes:", [...stages].join(","));
console.log("chaleur a touché toutes les roues:", allHeated, "| temps 1er>0.25 par roue (ms):", heatFirst.join(" "));
console.log("propagation amont→aval (drive avant tail):", propagates);
const ok = errs.length === 0 && spun === rot0.length && maxStrainSeen > 0.5 && maxProofs >= 1 && allHeated && propagates && bottlenecks.size >= 1;
console.log(ok ? "\nVERDICT: PASS" : "\nVERDICT: A AJUSTER");
