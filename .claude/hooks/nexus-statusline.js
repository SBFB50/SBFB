#!/usr/bin/env node
// .claude/hooks/nexus-statusline.js
//
// Statusline nexus-aware : prefixe le statusline GSD existant avec le
// contexte sprint/phase courant + warning si memory drift.
//
// Architecture : delegue a ~/.claude/hooks/gsd-statusline.js pour le
// bloc model/task/dir/context, puis prepend "[S<N>/<X>]" avec les
// infos sprint detectees depuis .planning/active/ + git log.
//
// Fallback : si pas dans le repo nexus, output = GSD statusline brut.
//
// Output format exemple :
//   [S18/B d7ab281] <model> | <task?> | <dirname> <context_bar>
//   [S18/B ⚠drift] <model> | ...  (si memory tip != HEAD)

const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync, spawnSync } = require('child_process');

// Read all stdin async
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const data = JSON.parse(input);
    const cwd = data.workspace?.current_dir || process.cwd();

    // Delegate to GSD statusline (always)
    const gsdPath = path.join(os.homedir(), '.claude', 'hooks', 'gsd-statusline.js');
    let gsdOut = '';
    if (fs.existsSync(gsdPath)) {
      try {
        const r = spawnSync('node', [gsdPath], {
          input: input,
          encoding: 'utf8',
          timeout: 3000,
        });
        gsdOut = r.stdout || '';
      } catch (e) { /* silent */ }
    }

    // Detect if we're in the nexus repo
    const inNexus =
      fs.existsSync(path.join(cwd, 'Cargo.toml')) &&
      fs.existsSync(path.join(cwd, 'crates', 'nexus-core-rs'));

    if (!inNexus) {
      process.stdout.write(gsdOut);
      return;
    }

    // Parse sprint number from .planning/active/sprint{N}_kickoff.md
    let sprintN = '?';
    let phaseX = '?';
    try {
      const activeDir = path.join(cwd, '.planning', 'active');
      if (fs.existsSync(activeDir)) {
        const files = fs.readdirSync(activeDir);
        for (const f of files) {
          const m = f.match(/^sprint(\d+)_(?:kickoff|plan)\.md$/);
          if (m && (sprintN === '?' || Number(m[1]) > Number(sprintN))) {
            sprintN = m[1];
          }
        }
      }
    } catch (e) { /* silent */ }

    // Detect last phase from git log (most recent feat/fix/docs(sprintN))
    try {
      const log = execSync(
        `git log -20 --format=%s`,
        { cwd, encoding: 'utf8', timeout: 2000, stdio: ['ignore', 'pipe', 'ignore'] }
      );
      const lines = log.split('\n');
      for (const line of lines) {
        // Match: "feat(sprint18): Phase A — ..." or "docs(sprint17): Sprint 17 Phase D — ..."
        const m = line.match(/(?:feat|fix|docs|chore|test)\(sprint(\d+)\).*?Phase\s+([A-Z]\d?)/);
        if (m && m[1] === sprintN) {
          phaseX = m[2];
          break;
        }
      }
    } catch (e) { /* silent */ }

    // Memory drift : compare memory tip vs HEAD
    let driftFlag = '';
    try {
      const headSha = execSync(`git rev-parse --short HEAD`, {
        cwd, encoding: 'utf8', timeout: 1000, stdio: ['ignore', 'pipe', 'ignore']
      }).trim();
      const memFile = path.join(
        os.homedir(), '.claude', 'projects',
        'C--Users-FlowUP-Documents-Code-nexus', 'memory', 'nexus_grid_pivot.md'
      );
      if (fs.existsSync(memFile)) {
        const content = fs.readFileSync(memFile, 'utf8').substring(0, 4000);
        const m = content.match(/Tip `([a-f0-9]+)`/);
        const memTip = m ? m[1] : null;
        if (memTip && memTip !== headSha) {
          driftFlag = ` \x1b[33m⚠drift\x1b[0m`;
        }
      }
    } catch (e) { /* silent */ }

    // Compose prefix : "[S18/B]" or "[S18/B ⚠drift]"
    const prefix = `\x1b[36m[S${sprintN}/${phaseX}${driftFlag}\x1b[36m]\x1b[0m`;

    // Emit prefix + separator + GSD output
    if (gsdOut) {
      process.stdout.write(`${prefix} ${gsdOut}`);
    } else {
      process.stdout.write(prefix);
    }
  } catch (e) {
    // Silent fail — don't break statusline
    process.stdout.write(input ? '' : '');
  }
});
