#!/usr/bin/env node
// .claude/hooks/narrate-action.js
//
// PostToolUse hook that generates a French product-owner-style paragraph
// describing what Claude just did, via Haiku. Writes to
// .claude/narration.log (tail -50 rolling). nexus-statusline.js reads
// the last line (truncated to 60 chars) and appends it to the status bar ;
// `tail -f .claude/narration.log` shows the full paragraph in a side pane.
//
// Returns in <30ms : spawns a detached child that does the LLM call,
// so Claude never waits on us.
//
// API resolution order:
//   1. $ANTHROPIC_API_KEY  -> direct https call (~400ms)
//   2. `claude -p`         -> reuses Claude Code credentials (~1.5s)
//   3. no-op               -> silent, no narration line

const fs = require('fs');
const os = require('os');
const path = require('path');
const crypto = require('crypto');
const { spawn, execSync } = require('child_process');
const https = require('https');

const PARENT_MODE = !process.env.__NARRATE_CHILD__;

function parentExit() {
  // Read stdin, spawn detached child with the JSON as arg, exit immediately.
  let input = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', c => input += c);
  process.stdin.on('end', () => {
    try {
      const child = spawn(process.execPath, [__filename], {
        env: { ...process.env, __NARRATE_CHILD__: '1', __NARRATE_STDIN__: input },
        detached: true,
        stdio: 'ignore',
        windowsHide: true,
      });
      child.unref();
    } catch (e) { /* silent */ }
    process.exit(0);
  });
}

async function child() {
  try {
    const raw = process.env.__NARRATE_STDIN__ || '';
    const data = JSON.parse(raw);
    const tool = data.tool_name || 'unknown';
    const input = data.tool_input || {};
    const sessionId = (data.session_id || 'unknown').slice(0, 8);
    const inputStr = JSON.stringify(input).slice(0, 400);

    const cwd = data.cwd || process.cwd();
    const logPath = path.join(cwd, '.claude', 'narration.log');
    const archiveDir = path.join(cwd, '.claude', 'narration');

    const prompt = `Tu es un narrateur qui explique a un Product Owner du projet SBFB / nexus-grid ce que l'assistant de code vient de faire. Le PO est technique mais pas developpeur : il suit la roadmap, comprend l'architecture (P2P, iroh-blobs, coordinator FastAPI, daemon Rust, shell React, apps publiees depuis repos Git verifies, SLSA provenance, curator lists gossip, etc.), mais n'a pas le code sous les yeux.

Contexte produit : SBFB est une plateforme P2P universelle de compute et d'hebergement d'apps. Le projet est au Sprint 20 (v1.2 en cours) — focus hardening transport + encryption at rest + panic wipe. Chaque phase ajoute une brique concrete (PoW Hashcash, TLS pinning, delayed upload queue, pkarr relay, etc.).

Regles :
- UN paragraphe de 2 a 4 phrases, en francais
- Relie l'action au produit ("durcit la couche transport", "verifie la signature du deploy", "teste le decoder v5 de l'announcement") plutot qu'au code brut
- Garde le vocabulaire technique quand c'est pertinent (iroh, blobs, gossip, daemon, curator, provenance, SLSA, pkarr) mais explique la finalite produit
- Ne repete pas l'outil comme "Bash a ete execute" — dis ce que l'action accomplit
- Pas de guillemets ni de prefixes ("Claude fait...", "L'assistant..."). Commence directement par un verbe au present.
- Si l'entree n'a aucune valeur produit (cat d'un fichier random, ls), reste court : une seule phrase factuelle.

Outil invoque : ${tool}
Parametres : ${inputStr}

Reponds uniquement par le paragraphe, sans entete.`;

    let phrase = '';
    if (process.env.ANTHROPIC_API_KEY) {
      phrase = await callAnthropicApi(prompt).catch(() => '');
    }
    if (!phrase) {
      phrase = await callClaudeCli(prompt).catch(() => '');
    }

    // Collapse multi-line paragraph into a single log line (newlines -> space).
    // Strip stray wrapping quotes/backticks and repeated whitespace.
    phrase = (phrase || '')
      .replace(/\r/g, '')
      .replace(/\n+/g, ' ')
      .replace(/^["'`\s]+|["'`\s]+$/g, '')
      .replace(/\s{2,}/g, ' ')
      .trim();
    if (!phrase) return;

    const now = new Date();
    const ts = now.toTimeString().slice(0, 8);
    const isoTs = now.toISOString();
    const dayStamp = isoTs.slice(0, 10);

    // Context tags for the archive stream : short SHA + detected sprint/phase
    // so the terminal viewer can group entries by commit and phase. Failures
    // fall back to empty strings, keeping the hook best-effort.
    let gitSha = '';
    try {
      gitSha = execSync('git rev-parse --short HEAD', {
        cwd, stdio: ['ignore', 'pipe', 'ignore']
      }).toString().trim();
    } catch { /* silent */ }

    let sprintTag = '';
    try {
      const activeDir = path.join(cwd, '.planning', 'active');
      let sprintN = '';
      if (fs.existsSync(activeDir)) {
        for (const f of fs.readdirSync(activeDir)) {
          const m = f.match(/^sprint(\d+)_(?:kickoff|plan)\.md$/);
          if (m && (!sprintN || Number(m[1]) > Number(sprintN))) sprintN = m[1];
        }
      }
      if (sprintN) {
        let phaseX = '';
        try {
          const log = execSync('git log -20 --format=%s', {
            cwd, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore']
          });
          for (const line of log.split('\n')) {
            const m = line.match(/(?:feat|fix|docs|chore|test)\(sprint(\d+)\).*?Phase\s+([A-Z]\d?)/);
            if (m && m[1] === sprintN) { phaseX = m[2]; break; }
          }
        } catch { /* silent */ }
        sprintTag = phaseX ? `S${sprintN}/${phaseX}` : `S${sprintN}`;
      }
    } catch { /* silent */ }

    // Short line for the rolling log consumed by the statusline.
    const shortLine = `[${ts}] ${phrase}\n`;
    // Richer line for the append-only archive : ISO timestamp + short commit
    // + sprint tag + session prefix + tool + phrase. Parsed by
    // narration-viewer.js ; legacy entries without commit/sprint tags
    // remain parseable (optional capture groups).
    const commitPart = gitSha ? ` [${gitSha}]` : '';
    const sprintPart = sprintTag ? ` [${sprintTag}]` : '';
    const archiveLine = `[${isoTs}]${commitPart}${sprintPart} [session:${sessionId}] [tool:${tool}] ${phrase}\n`;

    try {
      fs.mkdirSync(path.dirname(logPath), { recursive: true });
      fs.appendFileSync(logPath, shortLine);
      // Rolling window : keep last 50 lines so the statusline read is fast.
      const content = fs.readFileSync(logPath, 'utf8');
      const lines = content.split('\n').filter(Boolean);
      if (lines.length > 50) {
        fs.writeFileSync(logPath, lines.slice(-50).join('\n') + '\n');
      }
    } catch (e) { /* silent */ }

    // Append-only daily archive — never rotated, preserves the full trail.
    try {
      fs.mkdirSync(archiveDir, { recursive: true });
      const archivePath = path.join(archiveDir, `${dayStamp}.log`);
      fs.appendFileSync(archivePath, archiveLine);
    } catch (e) { /* silent */ }
  } catch (e) { /* silent */ }
}

function callAnthropicApi(prompt) {
  return new Promise((resolve, reject) => {
    const body = JSON.stringify({
      model: 'claude-haiku-4-5-20251001',
      max_tokens: 400,
      messages: [{ role: 'user', content: prompt }],
    });
    const req = https.request({
      hostname: 'api.anthropic.com',
      path: '/v1/messages',
      method: 'POST',
      headers: {
        'x-api-key': process.env.ANTHROPIC_API_KEY,
        'anthropic-version': '2023-06-01',
        'content-type': 'application/json',
        'content-length': Buffer.byteLength(body),
      },
      timeout: 8000,
    }, res => {
      let buf = '';
      res.on('data', c => buf += c);
      res.on('end', () => {
        try {
          const j = JSON.parse(buf);
          resolve(j.content?.[0]?.text || '');
        } catch (e) { reject(e); }
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(new Error('timeout')); });
    req.write(body);
    req.end();
  });
}

function callClaudeCli(prompt) {
  return new Promise((resolve) => {
    // Windows: Node >=18 refuses to spawn .cmd without shell:true (EINVAL),
    // but shell:true breaks stdin piping. Workaround: write prompt to a
    // temp file and use `cmd /c "claude ... < tempfile"` shell redirection.
    const isWin = process.platform === 'win32';
    const tmpFile = path.join(os.tmpdir(), `narrate-${crypto.randomBytes(6).toString('hex')}.txt`);
    try {
      fs.writeFileSync(tmpFile, prompt, 'utf8');
    } catch (e) {
      return resolve('');
    }

    const cleanup = () => { try { fs.unlinkSync(tmpFile); } catch (e) { /* */ } };

    let child;
    try {
      // shell:true is required on Windows so the `< tmpFile` redirection is
      // interpreted by the shell (cmd.exe) rather than passed as a literal arg.
      // Direct spawn of .cmd fails with EINVAL on Node >=18.
      const cmdLine = `claude -p --model claude-haiku-4-5-20251001 < "${tmpFile}"`;
      child = spawn(cmdLine, [], { windowsHide: true, shell: true });
    } catch (e) {
      cleanup();
      return resolve('');
    }

    let stdout = '';
    let done = false;
    const finish = (val) => { if (!done) { done = true; cleanup(); resolve(val); } };

    const to = setTimeout(() => {
      try { child.kill(); } catch (e) { /* */ }
      finish('');
    }, 20000);

    child.stdout.on('data', c => stdout += c);
    child.on('error', () => { clearTimeout(to); finish(''); });
    child.on('close', () => { clearTimeout(to); finish(stdout.trim()); });
  });
}

if (PARENT_MODE) {
  parentExit();
} else {
  child();
}
