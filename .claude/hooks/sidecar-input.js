#!/usr/bin/env node
// .claude/hooks/sidecar-input.js
//
// Persistent terminal spawned by launch-sidecar-terminal.sh. Unified
// command center :
//
//   1. Input line at the bottom — the user types a question, press
//      Enter, the question is appended to .claude/.sidecar-queue.jsonl
//      with a timestamp. sidecar-drain-on-post-tool.js flushes it to
//      Claude's context at the next tool call.
//
//   2. Narration stream — tails today's .claude/narration/<date>.log
//      so every Haiku-narrated action Claude takes is visible here.
//      Same parse as narration-viewer.js : iso + commit + sprint +
//      tool + paragraph.
//
//   3. Delivery confirmation stream — tails .claude/.sidecar-replies.
//      jsonl which sidecar-drain-on-post-tool.js appends to whenever
//      it drains a queued question. Confirms the user that Claude
//      has seen the message.
//
// Readline + async output pattern : every event clears the current
// line, prints the event, re-renders the readline prompt + buffer.
// Uses `rl._refreshLine()` which is internal but stable since Node
// 12 and the only way to restore the user's typed buffer after an
// out-of-band print.

const fs = require('fs');
const path = require('path');
const readline = require('readline');
const { execSync } = require('child_process');

const cwd = process.env.CLAUDE_PROJECT_DIR || process.cwd();
const queuePath = path.join(cwd, '.claude', '.sidecar-queue.jsonl');
const repliesPath = path.join(cwd, '.claude', '.sidecar-replies.jsonl');
const heartbeatPath = path.join(cwd, '.claude', '.sidecar-terminal.heartbeat');
const narrationDir = path.join(cwd, '.claude', 'narration');

const RESET = '\x1b[0m';
const BOLD = '\x1b[1m';
const DIM = '\x1b[2m';
const CYAN = '\x1b[36m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const MAGENTA = '\x1b[35m';
const BLUE = '\x1b[34m';
const GRAY = '\x1b[38;5;244m';
const WHITE = '\x1b[97m';

function writeHeartbeat() {
  try { fs.writeFileSync(heartbeatPath, String(Date.now())); } catch {}
}

function gitShortSha() {
  try {
    return execSync('git rev-parse --short HEAD', {
      cwd, stdio: ['ignore', 'pipe', 'ignore']
    }).toString().trim();
  } catch { return ''; }
}

function detectSprint() {
  let sprintN = '?';
  let phaseX = '?';
  try {
    const files = fs.readdirSync(path.join(cwd, '.planning', 'active'));
    for (const f of files) {
      const m = f.match(/^sprint(\d+)_(?:kickoff|plan)\.md$/);
      if (m && (sprintN === '?' || Number(m[1]) > Number(sprintN))) sprintN = m[1];
    }
  } catch { /* silent */ }
  try {
    const log = execSync('git log -20 --format=%s', {
      cwd, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore']
    });
    for (const line of log.split('\n')) {
      const m = line.match(/(?:feat|fix|docs|chore|test)\(sprint(\d+)\).*?Phase\s+([A-Z]\d?)/);
      if (m && m[1] === sprintN) { phaseX = m[2]; break; }
    }
  } catch { /* silent */ }
  return { sprintN, phaseX };
}

function banner() {
  const sha = gitShortSha() || '(no git)';
  const { sprintN, phaseX } = detectSprint();
  const now = new Date().toISOString().replace('T', ' ').slice(0, 19);
  const project = path.basename(cwd);
  const line = '═'.repeat(60);
  console.log(`${CYAN}${BOLD}╔${line}╗${RESET}`);
  console.log(`${CYAN}${BOLD}║${RESET}  ${BOLD}Nexus Sidecar${RESET}${GRAY}  —  input + narration + delivery${RESET}`);
  console.log(`${CYAN}${BOLD}╠${line}╣${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}project${RESET}  ${WHITE}${project}${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}sprint ${RESET}  ${MAGENTA}S${sprintN}/${phaseX}${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}HEAD   ${RESET}  ${YELLOW}${sha}${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}flow   ${RESET}  ${WHITE}type -> next tool call injects your question${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}feed   ${RESET}  ${WHITE}narration + delivery confirmations live below${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}started${RESET}  ${WHITE}${now}${RESET}`);
  console.log(`${CYAN}${BOLD}╚${line}╝${RESET}`);
  console.log();
}

// Parse narration-log lines the same way narration-viewer.js does.
function parseNarrationLine(line) {
  const re = /^\[([^\]]+)\](?:\s+\[([0-9a-f]{5,10})\])?(?:\s+\[(S\d+(?:\/[A-Z]\d?)?)\])?(?:\s+\[session:([^\]]+)\])?(?:\s+\[tool:([^\]]+)\])?\s+(.*)$/;
  const m = line.match(re);
  if (!m) return null;
  return {
    iso: m[1],
    commit: m[2] || '',
    sprint: m[3] || '',
    session: m[4] || '',
    tool: m[5] || '',
    phrase: m[6] || '',
  };
}

function renderNarration(line) {
  const parsed = parseNarrationLine(line);
  if (!parsed) return `${DIM}${line}${RESET}`;
  const hhmmss = parsed.iso.slice(11, 19);
  const parts = [`${BOLD}${hhmmss}${RESET}`];
  if (parsed.commit) parts.push(`${YELLOW}${parsed.commit}${RESET}`);
  if (parsed.sprint) parts.push(`${MAGENTA}${parsed.sprint}${RESET}`);
  if (parsed.tool) parts.push(`${GREEN}${parsed.tool}${RESET}`);
  const header = parts.join(`${GRAY} · ${RESET}`);
  return `${header}\n  ${parsed.phrase}`;
}

function renderReply(raw) {
  try {
    const obj = JSON.parse(raw);
    const ts = (obj.ts || '').slice(11, 19);
    if (obj.kind === 'drained') {
      const n = obj.count || 1;
      return `${BLUE}${BOLD}${ts}${RESET} ${BLUE}◆ delivered${RESET}${GRAY} — ${n} message${n > 1 ? 's' : ''} in Claude's context${RESET}`;
    }
    if (obj.kind === 'ack') {
      return `${GREEN}${BOLD}${ts}${RESET} ${GREEN}✓ Claude${RESET} ${obj.text || ''}`;
    }
    return `${DIM}${raw}${RESET}`;
  } catch {
    return `${DIM}${raw}${RESET}`;
  }
}

// -- async event printer that coexists with readline --

let rl; // set below
function printEvent(text) {
  if (!rl) {
    process.stdout.write(text + '\n');
    return;
  }
  // Clear current line (including the prompt + user buffer), write the
  // event, then re-render the prompt + buffer so the user never loses
  // their typing.
  process.stdout.write('\r\x1b[K');
  process.stdout.write(text + '\n');
  // `rl._refreshLine` is undocumented but stable and the canonical
  // way to re-paint the readline prompt after an out-of-band print.
  if (typeof rl._refreshLine === 'function') {
    rl._refreshLine();
  } else {
    rl.prompt(true);
  }
}

// -- today's narration log tail loop --

function todayNarrationLogPath() {
  const today = new Date().toISOString().slice(0, 10);
  return path.join(narrationDir, `${today}.log`);
}

function startNarrationTail() {
  let fp = todayNarrationLogPath();
  let offset = 0;
  // Do not replay historical entries on boot — the sidecar is for
  // live monitoring. A fresh launch starts at the tail.
  try {
    if (fs.existsSync(fp)) offset = fs.statSync(fp).size;
  } catch {}

  setInterval(() => {
    const currentFp = todayNarrationLogPath();
    if (currentFp !== fp) {
      printEvent(`${GRAY}── day rollover → ${path.basename(currentFp)} ──${RESET}`);
      fp = currentFp;
      offset = 0;
    }
    if (!fs.existsSync(fp)) return;
    const stat = fs.statSync(fp);
    if (stat.size < offset) offset = 0;
    if (stat.size > offset) {
      const fd = fs.openSync(fp, 'r');
      const buf = Buffer.alloc(stat.size - offset);
      fs.readSync(fd, buf, 0, buf.length, offset);
      fs.closeSync(fd);
      const chunk = buf.toString('utf8');
      const newLines = chunk.split('\n').filter(Boolean);
      for (const line of newLines) printEvent(renderNarration(line));
      offset = stat.size;
    }
  }, 1000);
}

// -- replies log tail loop (delivery / ack events written by hooks) --

function startRepliesTail() {
  let offset = 0;
  try {
    if (fs.existsSync(repliesPath)) offset = fs.statSync(repliesPath).size;
  } catch {}

  setInterval(() => {
    if (!fs.existsSync(repliesPath)) return;
    const stat = fs.statSync(repliesPath);
    if (stat.size < offset) offset = 0;
    if (stat.size > offset) {
      const fd = fs.openSync(repliesPath, 'r');
      const buf = Buffer.alloc(stat.size - offset);
      fs.readSync(fd, buf, 0, buf.length, offset);
      fs.closeSync(fd);
      const chunk = buf.toString('utf8');
      const newLines = chunk.split('\n').filter(Boolean);
      for (const line of newLines) printEvent(renderReply(line));
      offset = stat.size;
    }
  }, 500);
}

// -- queue helpers --

function queueCount() {
  if (!fs.existsSync(queuePath)) return 0;
  return fs.readFileSync(queuePath, 'utf8').split('\n').filter(Boolean).length;
}

function submit(text) {
  const entry = { ts: new Date().toISOString(), question: text };
  fs.appendFileSync(queuePath, JSON.stringify(entry) + '\n');
}

// -- main --

banner();
writeHeartbeat();
setInterval(writeHeartbeat, 5000);

const pending = queueCount();
if (pending > 0) {
  console.log(`${YELLOW}note${RESET}  ${pending} question${pending === 1 ? '' : 's'} deja en attente de drain`);
}

rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  prompt: `${MAGENTA}${BOLD}› ${RESET}`,
  terminal: true,
});

startNarrationTail();
startRepliesTail();

rl.prompt();
rl.on('line', (line) => {
  const trimmed = line.trim();
  if (!trimmed) { rl.prompt(); return; }
  try {
    submit(trimmed);
    const ts = new Date().toISOString().slice(11, 19);
    printEvent(`${MAGENTA}${BOLD}${ts}${RESET} ${MAGENTA}› user${RESET}${GRAY} — ${RESET}${trimmed}`);
  } catch (e) {
    printEvent(`${YELLOW}error${RESET} submit failed: ${e.message}`);
  }
  rl.prompt();
});

rl.on('close', () => {
  console.log(`\n${DIM}(sidecar closed)${RESET}`);
  process.exit(0);
});

process.on('SIGINT', () => { rl.close(); });
