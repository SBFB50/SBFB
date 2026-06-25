// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Build the two CSP self-check fixture archives (Sprint 79 Phase H).
 *
 * Produces `clean.zip` and `dirty.zip` from `src/clean/` and `src/dirty/`.
 * The archives are COMMITTED (the E2E spec seeds them into the hermetic
 * daemon via `POST /api/daemon/publish-blob`, which needs a real .zip body
 * the daemon decompresses with the `zip` crate). They are committed rather
 * than generated at test time because neither Node nor Playwright ships a
 * native zip writer and the runtime-0-dep rule forbids adding jszip/fflate.
 *
 * This builder uses only Node built-ins (`node:zlib` deflate + a hand-rolled
 * ZIP local-file-header writer) — no dependency, cross-platform, and
 * deterministic (fixed mtime, no extra fields), so the committed binaries are
 * reproducible. Regenerate after editing a `src/` file:
 *
 *   node web/e2e/fixtures/app-authoring/build-fixtures.mjs
 *
 * It also asserts the dirty app's base64 target decodes to the documented
 * external URL, so the fixture can never silently stop exercising the
 * `connect-src 'none'` violation.
 */

import { deflateRawSync } from "node:zlib";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// CRC-32 (IEEE 802.3) — the only checksum ZIP requires; tiny table-less impl.
function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc ^= buf[i];
    for (let b = 0; b < 8; b++) {
      crc = crc & 1 ? (crc >>> 1) ^ 0xedb88320 : crc >>> 1;
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

/**
 * Minimal ZIP writer: one deflated entry per [name, bytes], deterministic
 * (DOS time/date pinned to 0). Sufficient for the `zip` crate to read.
 */
function makeZip(entries) {
  const chunks = [];
  const central = [];
  let offset = 0;

  for (const [name, content] of entries) {
    const nameBuf = Buffer.from(name, "utf-8");
    const data = Buffer.from(content);
    const compressed = deflateRawSync(data, { level: 9 });
    const crc = crc32(data);

    const local = Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50, 0); // local file header signature
    local.writeUInt16LE(20, 4); // version needed
    local.writeUInt16LE(0, 6); // flags
    local.writeUInt16LE(8, 8); // method = deflate
    local.writeUInt16LE(0, 10); // mod time (pinned)
    local.writeUInt16LE(0x21, 12); // mod date (pinned, 1980-01-01)
    local.writeUInt32LE(crc, 14);
    local.writeUInt32LE(compressed.length, 18);
    local.writeUInt32LE(data.length, 22);
    local.writeUInt16LE(nameBuf.length, 26);
    local.writeUInt16LE(0, 28); // extra field length
    chunks.push(local, nameBuf, compressed);

    const cen = Buffer.alloc(46);
    cen.writeUInt32LE(0x02014b50, 0); // central dir signature
    cen.writeUInt16LE(20, 4); // version made by
    cen.writeUInt16LE(20, 6); // version needed
    cen.writeUInt16LE(0, 8); // flags
    cen.writeUInt16LE(8, 10); // method
    cen.writeUInt16LE(0, 12); // mod time
    cen.writeUInt16LE(0x21, 14); // mod date
    cen.writeUInt32LE(crc, 16);
    cen.writeUInt32LE(compressed.length, 20);
    cen.writeUInt32LE(data.length, 24);
    cen.writeUInt16LE(nameBuf.length, 28);
    cen.writeUInt16LE(0, 30); // extra len
    cen.writeUInt16LE(0, 32); // comment len
    cen.writeUInt16LE(0, 34); // disk number
    cen.writeUInt16LE(0, 36); // internal attrs
    cen.writeUInt32LE(0, 38); // external attrs
    cen.writeUInt32LE(offset, 42); // local header offset
    central.push(Buffer.concat([cen, nameBuf]));

    offset += local.length + nameBuf.length + compressed.length;
  }

  const centralBuf = Buffer.concat(central);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0); // end of central dir signature
  end.writeUInt16LE(entries.length, 8); // entries on this disk
  end.writeUInt16LE(entries.length, 10); // total entries
  end.writeUInt32LE(centralBuf.length, 12); // central dir size
  end.writeUInt32LE(offset, 16); // central dir offset
  return Buffer.concat([...chunks, centralBuf, end]);
}

function buildOne(variant) {
  const html = readFileSync(
    resolve(__dirname, "src", variant, "index.html"),
  );
  const out = resolve(__dirname, `${variant}.zip`);
  writeFileSync(out, makeZip([["index.html", html]]));
  return out;
}

// Honesty assertion: the dirty fixture must keep exercising the violation.
const dirtyHtml = readFileSync(
  resolve(__dirname, "src", "dirty", "index.html"),
  "utf-8",
);
const m = dirtyHtml.match(/atob\("([^"]+)"\)/);
if (!m) throw new Error("dirty fixture lost its atob() runtime target");
const decoded = Buffer.from(m[1], "base64").toString("utf-8");
if (!decoded.startsWith("https://")) {
  throw new Error(`dirty atob target is not an external URL: ${decoded}`);
}
console.log(`dirty runtime target decodes to ${decoded} (violates connect-src 'none')`);

for (const v of ["clean", "dirty"]) {
  console.log(`wrote ${buildOne(v)}`);
}
