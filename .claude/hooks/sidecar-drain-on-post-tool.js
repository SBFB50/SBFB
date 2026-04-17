#!/usr/bin/env node
// .claude/hooks/sidecar-drain-on-post-tool.js
//
// PostToolUse hook — non-blocking "BTW" injection. After every tool
// call, drain all pending questions from .claude/.sidecar-queue.jsonl
// and surface them to Claude as additional context. Claude sees them
// on the next reasoning step and can acknowledge / answer them without
// being forced to stop its current work.
//
// Output contract : emit JSON with `hookSpecificOutput.additionalContext`
// so the string gets appended to the tool-result view that Claude
// reads on its next think step. No `decision: block` — the user
// explicitly does not want sidecar questions to interrupt ongoing work.
//
// Queue drain : batch, not one-at-a-time. If the user has queued 3
// questions, all 3 arrive in the SAME additionalContext block so
// Claude sees them together and can answer them coherently.
//
// Fail-safe : any fs / parse error exits 0 with empty additionalContext
// so a broken queue never wedges the tool loop.

const fs = require('fs');
const path = require('path');

const cwd = process.env.CLAUDE_PROJECT_DIR || process.cwd();
const queuePath = path.join(cwd, '.claude', '.sidecar-queue.jsonl');

let _stdin = '';
process.stdin.on('data', (c) => { _stdin += c; });
process.stdin.on('end', () => {
  try {
    if (!fs.existsSync(queuePath)) return exit0();
    const raw = fs.readFileSync(queuePath, 'utf8');
    const lines = raw.split('\n').filter(Boolean);
    if (lines.length === 0) return exit0();

    const entries = [];
    for (const line of lines) {
      try {
        const obj = JSON.parse(line);
        const q = (obj.question || '').toString().trim();
        if (q) entries.push({ ts: obj.ts || '', question: q });
      } catch {
        // Skip corrupt lines silently.
      }
    }
    if (entries.length === 0) {
      // Queue was present but all entries were unparseable — truncate
      // so we do not re-read the same garbage on every tool call.
      fs.writeFileSync(queuePath, '');
      return exit0();
    }

    // Drain : all consumed entries removed, queue cleared.
    fs.writeFileSync(queuePath, '');

    // Reverse channel : append a delivery-confirmation to the
    // sidecar replies log so the user's terminal shows a "◆
    // delivered" line right after they hit Enter. Sidecar-input.js
    // tails this file and renders it via renderReply().
    try {
      const repliesPath = path.join(cwd, '.claude', '.sidecar-replies.jsonl');
      const confirmation = {
        ts: new Date().toISOString(),
        kind: 'drained',
        count: entries.length,
      };
      fs.appendFileSync(repliesPath, JSON.stringify(confirmation) + '\n');
    } catch {
      // Silent — the reverse channel is a nice-to-have, never block
      // on its failure.
    }

    const header = entries.length === 1
      ? 'Un message sidecar est arrive pendant ton travail :'
      : `${entries.length} messages sidecar sont arrives pendant ton travail :`;
    const body = entries
      .map((e, i) => `  [${i + 1}] (${e.ts}) ${e.question}`)
      .join('\n');
    const footer = 'Tu peux les traiter maintenant (bref ack) ou les reprendre une fois l\'etape courante terminee — a ta main.';
    const additionalContext = `\n[BTW — sidecar]\n${header}\n${body}\n${footer}\n`;

    process.stdout.write(JSON.stringify({
      hookSpecificOutput: {
        hookEventName: 'PostToolUse',
        additionalContext,
      },
    }));
    process.exit(0);
  } catch (e) {
    try { process.stderr.write(`[sidecar-drain-post-tool] error: ${e.message}\n`); } catch {}
    exit0();
  }
});

function exit0() { process.exit(0); }
