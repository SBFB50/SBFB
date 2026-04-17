#!/usr/bin/env node
// .claude/hooks/sidecar-input.js
//
// Persistent readline terminal spawned by launch-sidecar-terminal.sh.
// The user types a question, press Enter — the line is appended to
// .claude/.sidecar-queue.jsonl with a timestamp. At Claude's next Stop
// event, the hook sidecar-drain-on-stop.js pops the oldest question
// and injects it as a block-decision so Claude answers it without the
// user having to touch the main TUI.
//
// The sidecar writes a heartbeat file every 5s so launch-sidecar-
// terminal.sh knows an instance is already running and skips the
// re-spawn on SessionStart (matches the narration-viewer pattern).

const fs = require('fs');
const path = require('path');
const readline = require('readline');

const cwd = process.env.CLAUDE_PROJECT_DIR || process.cwd();
const queuePath = path.join(cwd, '.claude', '.sidecar-queue.jsonl');
const heartbeatPath = path.join(cwd, '.claude', '.sidecar-terminal.heartbeat');

const RESET = '\x1b[0m';
const BOLD = '\x1b[1m';
const DIM = '\x1b[2m';
const CYAN = '\x1b[36m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const MAGENTA = '\x1b[35m';
const GRAY = '\x1b[38;5;244m';
const WHITE = '\x1b[97m';

function writeHeartbeat() {
  try { fs.writeFileSync(heartbeatPath, String(Date.now())); } catch {}
}

function banner() {
  const project = path.basename(cwd);
  const line = '═'.repeat(60);
  console.log(`${CYAN}${BOLD}╔${line}╗${RESET}`);
  console.log(`${CYAN}${BOLD}║${RESET}  ${BOLD}Nexus Sidecar Input${RESET}${GRAY}  —  ask while Claude works${RESET}`);
  console.log(`${CYAN}${BOLD}╠${line}╣${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}project${RESET}  ${WHITE}${project}${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}flow   ${RESET}  ${WHITE}type a question, press Enter${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}when   ${RESET}  ${WHITE}delivered within seconds (next tool call)${RESET}`);
  console.log(`${CYAN}${BOLD}╚${line}╝${RESET}`);
  console.log();
}

function queueCount() {
  if (!fs.existsSync(queuePath)) return 0;
  const raw = fs.readFileSync(queuePath, 'utf8');
  return raw.split('\n').filter(Boolean).length;
}

function submit(text) {
  const entry = { ts: new Date().toISOString(), question: text };
  fs.appendFileSync(queuePath, JSON.stringify(entry) + '\n');
  const pending = queueCount();
  const suffix = pending === 1 ? '' : `s (${pending} en attente)`;
  console.log(`  ${GREEN}✓${RESET} ${DIM}envoye — injecte au prochain tool call${suffix}${RESET}`);
}

banner();
writeHeartbeat();
setInterval(writeHeartbeat, 5000);

const pending = queueCount();
if (pending > 0) {
  console.log(`${YELLOW}note${RESET}  ${pending} question${pending === 1 ? '' : 's'} deja en attente de drain\n`);
}

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  prompt: `${MAGENTA}${BOLD}› ${RESET}`,
  terminal: true,
});

rl.prompt();
rl.on('line', (line) => {
  const trimmed = line.trim();
  if (!trimmed) { rl.prompt(); return; }
  try {
    submit(trimmed);
  } catch (e) {
    console.error(`${YELLOW}error${RESET} submit failed: ${e.message}`);
  }
  rl.prompt();
});

rl.on('close', () => {
  console.log(`\n${DIM}(sidecar closed)${RESET}`);
  process.exit(0);
});

process.on('SIGINT', () => {
  rl.close();
});
