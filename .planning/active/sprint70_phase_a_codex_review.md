Contexte vérifié : branche courante `master`. `git status --short` indique `M AGENTS.md` et `?? docs/agent/AGENT_SYSTEM.md`; l’audit ci-dessous porte donc sur le working tree actuel, pas sur un HEAD propre.

### Livrable 1 : `docs/agent/AGENT_SYSTEM.md` NEW
- Statut : PARTIEL
- Fichier(s) : `docs/agent/AGENT_SYSTEM.md:12`, `docs/agent/AGENT_SYSTEM.md:31`, `docs/agent/AGENT_SYSTEM.md:58`, `docs/agent/AGENT_SYSTEM.md:80`, `docs/agent/AGENT_SYSTEM.md:111`, `docs/agent/AGENT_SYSTEM.md:197`, `docs/agent/AGENT_SYSTEM.md:234`
- Evidence :
```md
80: ## 4. Lifecycle Modes
82: 10 modes du cycle de vie sprint. Chaque mode pointe vers le(s)
83: prompt(s) portable(s) a utiliser.
85: | # | Mode | Prompt(s) | Role(s) actif(s) | Artefact produit |
87: | 1 | `kickoff` | `universal.md` | `kickoff-author` | `sprint{N}_kickoff.md`, `sprint{N}_plan.md`, `sprint{N}_design_review.md` |
```
```md
184: | Gate | Fichier requis | Staged au commit | Verdict final |
186: | Preflight | `sprint{N}_phase_{X}_preflight.md` | Non (pre-code) | EXECUTE ou equivalent |
187: | Review | `sprint{N}_phase_{X}_review.md` | Oui | `## Verdict: PASS` |
188: | Codex | `sprint{N}_phase_{X}_codex_review.md` | Oui | CLEAN ou GAP-P2-P3 documentes |
189: | Audit | `sprint{N}_audit_findings.md` | Oui (commit propre) | PASS ou CONDITIONAL PASS |
```
- Si GAP : le document contient bien les 7 sections attendues, 9 rôles avec droits/obligations/limites (`:37-47`), les providers Claude/Codex/GPT/Local/Humain (`:63-69`), les 10 modes (`:87-96`), les verdicts complets et le contrat d’artefacts (`:117-189`). Gap réel : deux prompts documentés ne sont pas présents dans `prompts/agent/` : `audit-gate-checks.md` (`:94`) et `handoff.md` (`:96`, `:225`). Le Prompt Registry met aussi `handoff` et `audit-gate` en `*(Phase C)*` au lieu d’un fichier réel (`:212-213`), ce qui ne satisfait pas entièrement “table kind -> fichier”.

### Livrable 2 : `AGENTS.md` UPDATE
- Statut : CONFIRME
- Fichier(s) : `AGENTS.md:14`, `AGENTS.md:16`, `AGENTS.md:17`, `AGENTS.md:20`, `AGENTS.md:52`
- Evidence :
```md
16: - `cargo build -p nexus-shell-daemon --release`: release build (daemon).
17: - `cargo build -p sbfb-factory --release`: release build (factory tooling).
18: - `cd web && npm run dev`: start the Vite shell.
19: - `cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && npm run test:unit && npm run build && npm run size`: frontend lint, typecheck, unit tests, build, and size-limit.
20: - `python scripts/agent/agentctl.py context`: print vendor-neutral agent context (legacy Python).
```
```md
52: See `docs/agent/AGENT_SYSTEM.md` for the system map: roles, providers,
53: lifecycle modes, gate contracts, and prompt registry.
55: Use `docs/agent/PROCESS.md` as the vendor-neutral sprint workflow for Claude,
58: `scripts/agent/agentctl.py` (legacy Python) and `crates/sbfb-factory` (Rust)
59: enforce the portable gates.
```
- Si GAP : aucun. Recherche ciblée dans `AGENTS.md` : aucune occurrence de `packages/nexus-sdk`, `scripts/setup.sh`, ni `uv run pytest packages`.

## Resume final

- Total livrables : 2
- Confirmes : 1
- Gaps : 0
- Partiels : 1