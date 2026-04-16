#!/usr/bin/env node
// .claude/hooks/narration-viewer.js
//
// Persistent terminal that replays today's Haiku narrations and then
// follows the live archive log. Each line is formatted with timestamp,
// short commit SHA, sprint/phase tag, and originating tool.
//
// Spawned once per session by launch-narration-terminal.sh (SessionStart
// hook). Writes a heartbeat file every second so the launcher can detect
// an already-running viewer and skip re-spawning.

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const cwd = process.env.CLAUDE_PROJECT_DIR || process.cwd();
const archiveDir = path.join(cwd, '.claude', 'narration');
const heartbeatPath = path.join(cwd, '.claude', '.narration-terminal.heartbeat');

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
  console.log(`${CYAN}${BOLD}║${RESET}  ${BOLD}Nexus Narration Viewer${RESET}${GRAY}  —  live Haiku feed${RESET}`);
  console.log(`${CYAN}${BOLD}╠${line}╣${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}project${RESET}  ${WHITE}${project}${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}sprint ${RESET}  ${MAGENTA}S${sprintN}/${phaseX}${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}HEAD   ${RESET}  ${YELLOW}${sha}${RESET}`);
  console.log(`${CYAN}║${RESET}  ${GRAY}started${RESET}  ${WHITE}${now}${RESET}`);
  console.log(`${CYAN}${BOLD}╚${line}╝${RESET}`);
  console.log();
}

// Parses both new-format (with [commit] [sprint] tags) and legacy
// entries that only have [iso] [session] [tool] phrase.
function parseLine(line) {
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

function formatEntry(e) {
  const hhmmss = e.iso.slice(11, 19);
  const date = e.iso.slice(0, 10);
  const parts = [];
  parts.push(`${DIM}${date}${RESET} ${BOLD}${hhmmss}${RESET}`);
  if (e.commit) parts.push(`${YELLOW}${e.commit}${RESET}`);
  if (e.sprint) parts.push(`${MAGENTA}${e.sprint}${RESET}`);
  if (e.tool) parts.push(`${GREEN}${e.tool}${RESET}`);
  if (e.session) parts.push(`${DIM}${e.session.slice(0, 8)}${RESET}`);
  const header = parts.join(`${GRAY} · ${RESET}`);
  return `${header}\n  ${e.phrase}\n`;
}

function renderLine(line) {
  const parsed = parseLine(line);
  if (parsed) return formatEntry(parsed);
  return `${DIM}${line}${RESET}\n`;
}

function todayLogPath() {
  const today = new Date().toISOString().slice(0, 10);
  return path.join(archiveDir, `${today}.log`);
}

function replayToday() {
  const fp = todayLogPath();
  if (!fs.existsSync(fp)) {
    console.log(`${GRAY}(no archive yet for today — waiting for the first tool call)${RESET}\n`);
    return 0;
  }
  const content = fs.readFileSync(fp, 'utf8');
  const lines = content.split('\n').filter(Boolean);
  const replay = lines.slice(-50);
  console.log(`${GRAY}── replay ${replay.length} last entries ──────────────────────────${RESET}\n`);
  for (const l of replay) process.stdout.write(renderLine(l));
  console.log(`${GRAY}── live ────────────────────────────────────────────────────${RESET}\n`);
  return fs.statSync(fp).size;
}

function tailLoop(initialOffset) {
  let fp = todayLogPath();
  let offset = initialOffset;
  let lastDay = fp;

  setInterval(() => {
    try { fs.writeFileSync(heartbeatPath, String(Date.now())); } catch {}

    const currentFp = todayLogPath();
    if (currentFp !== lastDay) {
      console.log(`\n${GRAY}── day rollover → ${path.basename(currentFp)} ──${RESET}\n`);
      fp = currentFp;
      offset = 0;
      lastDay = currentFp;
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
      for (const l of newLines) process.stdout.write(renderLine(l));
      offset = stat.size;
    }
  }, 1000);
}

process.on('SIGINT', () => { process.exit(0); });

banner();
const startOffset = replayToday();
tailLoop(startOffset);
