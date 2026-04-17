#!/usr/bin/env node
// .claude/hooks/sidecar-drain-on-stop.js
//
// Stop hook — when Claude finishes a turn, pop the oldest entry from
// .claude/.sidecar-queue.jsonl (if any) and emit a block-decision with
// that entry's text as `reason`. Claude re-enters the loop and answers
// the question without the user having to type in the main TUI.
//
// Queue flow : one question per Stop event. If multiple questions are
// queued, they drain one at a time across successive Stop events, so
// Claude answers them in submission order.
//
// Bail-safe : any parse error or fs error exits 0 (non-blocking) so a
// broken queue never wedges the session.

const fs = require('fs');
const path = require('path');

const cwd = process.env.CLAUDE_PROJECT_DIR || process.cwd();
const queuePath = path.join(cwd, '.claude', '.sidecar-queue.jsonl');

// The Stop hook protocol feeds the event JSON on stdin. We do not need
// its contents — we only act on queue presence — but we still drain
// stdin so the runtime does not SIGPIPE us.
let _stdin = '';
process.stdin.on('data', (c) => { _stdin += c; });
process.stdin.on('end', () => {
  try {
    if (!fs.existsSync(queuePath)) return exit0();
    const raw = fs.readFileSync(queuePath, 'utf8');
    const lines = raw.split('\n').filter(Boolean);
    if (lines.length === 0) return exit0();

    let next;
    try {
      next = JSON.parse(lines[0]);
    } catch {
      // Corrupt first line — drop it and bail (no injection this round).
      const remaining = lines.slice(1);
      fs.writeFileSync(queuePath, remaining.join('\n') + (remaining.length ? '\n' : ''));
      return exit0();
    }

    // Pop and persist the shrunken queue.
    const remaining = lines.slice(1);
    fs.writeFileSync(queuePath, remaining.join('\n') + (remaining.length ? '\n' : ''));

    const ts = next.ts || new Date().toISOString();
    const question = (next.question || '').toString().trim();
    if (!question) return exit0();

    const reason = `[Message sidecar ${ts}] ${question}`;
    process.stdout.write(JSON.stringify({ decision: 'block', reason }));
    process.exit(0);
  } catch (e) {
    // Never block on an error — log to stderr (visible in Claude Code
    // debug mode) and exit non-blocking.
    try { process.stderr.write(`[sidecar-drain] error: ${e.message}\n`); } catch {}
    exit0();
  }
});

function exit0() { process.exit(0); }
