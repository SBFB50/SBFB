# Sprint log — historique cross-version

Index synthetique de tous les sprints livres. Une ligne par sprint.
Detail des decisions, plans et verifications dans
[`.planning/archive/v{X}/sprint{N}_*.md`](../../.planning/archive/).

Pour la methodologie sprint elle-meme (lifecycle, audit gate,
conventions commit), voir [`README.md`](README.md).

Pour le sprint en cours, voir
[`.planning/active/`](../../.planning/active/).

---

## v1.2 — Security hardening (en cours)

| Sprint | Etat | Tip cloture | Nb commits | Docs |
|---|---|---|---|---|
| 16 | DONE | `<Phase E>` (docs + verification + audit plan) | 6 (Phase 0 gate + A-D + docs) | 5 docs (kickoff, plan, verification, audit_plan) + docs/security/ (README + THREAT_MODEL + RUNTIME_ISOLATION) |

**Faits saillants** :

- **Sprint 16** : loopback passe de `D` a `A-` via defense en
  profondeur — X-SBFB-Token 256-bit (`d7c265a`, launcher
  genere, perm 0600) + Host allowlist + Origin check
  (mitigation CVE-2025-49596 Anthropic MCP Inspector DNS
  rebinding, CVSS 9.4). UDS avec SO_PEERCRED (pattern Tailscale
  safesocket) + Named Pipes Windows avec DACL user-only via
  SDDL (`1cfde89`). GPU consent dialog 4 niveaux + whitelist L3
  manuelle + raccourci "Contribuer mon GPU" depuis Browse +
  caps W/VRAM/heures enforced worker-side via
  `should_accept_task` + `ConsentWatcher` (notify crate, 50 ms
  debounce) + usage.json daily counter reset minuit-local
  (`3247e88`). ProjectAnnouncement v5 avec `is_open_source`
  derive automatiquement par le coordinator (true pour
  deploy-from-repo, false pour zip prive, non-user-settable
  pattern npm provenance/cosign), backward compat v4 via
  decoder tolerant (`10bbc63`). Threat model STRIDE + LINDDUN
  livre dans `docs/security/` avec roadmap runtime isolation
  WSL2 / Virtualization.framework / systemd-nspawn pour Sprint
  17+. Compteurs : ~1136 tests (+~200 ce sprint).

Detail : [`.planning/archive/v1.2/`](../../.planning/archive/v1.2/).

---

## v1.1 — Verified deploy + bridge bidirectionnel + CPU watchdog (S14-15)

| Sprint | Etat | Tip cloture | Nb commits | Docs |
|---|---|---|---|---|
| 14 | DONE + CONDITIONAL PASS levé | `f6015b3` (A-1 commit_sha fix landed) | 5 + 1 (gate) | 5 docs (kickoff, plan, verification, audit_plan, audit_findings) |
| 15 | DONE | `4da0043` (Phase E docs) | 5 (A-D + docs) | 4 docs (kickoff, plan, verification, audit_plan) |

**Faits saillants** :

- **Sprint 14** : premier deploy verified-from-source. Le coordinator
  clone le repo, verifie SBFB.json (Keyoxide pattern Ed25519), zip
  le contenu, signe `provenance.json` (SLSA L1). ProjectAnnouncement
  v4 ajoute `provenance_hash` et `repo_url`. Multi-forge (GitHub,
  GitLab, Codeberg, Gitea generique). Badge "Verifie" cote shell.
  Audit conditional PASS leve via `542479f` (commit_sha SHA pinning
  full 40 hex).

- **Sprint 15** : bridge devient bidirectionnel via `sbfb-bridge-event`
  (host → iframe push, fire-and-forget). CPU watchdog via heartbeat
  `sbfb-bridge-heartbeat` (1s) + timeout 5s + overlay "App ne repond
  plus". CLI `sbfb init <type> <path>` scaffolds 3 templates
  (html/react/pyodide). E2E Playwright avec iframe reelle qui charge
  le vrai SDK. Compteurs : ~934 tests total (+26 ce sprint).

Detail : [`.planning/archive/v1.1/`](../../.planning/archive/v1.1/).

---

## v1.0 — Pivot SBFB → P2P → universal render → bridge postMessage (S0-13)

| Sprint | Etat | Tip cloture | Nb commits | Docs |
|---|---|---|---|---|
| 0 | DONE | `stabilize/compute` mergée | 9 | - |
| 1 | DONE | `e631325` | - | - |
| 2 | DONE + audité rétro | `ed2ea76` | 6 | audit rétro dans `audit_sprint2/` |
| 3 | DONE | `9476be8` | 12 (W1..W12) | `sprint3_verification.md` |
| 4 | DONE | `3b5c162` | 9 | `sprint4_kickoff`, `_plan`, `_verification`, `_verify_prompt` |
| 5 | DONE | `cdf4467` | 9 | `sprint5_kickoff` (monolithique), `_plan`, `_verification` |
| 6 | DONE + CONDITIONAL PASS levé | `504c6aa` puis `2926383` post-gate | 8 + 10 (gate) | 4 docs + `audit_findings` |
| 7 | DONE | `9cc0796` | 8 | 4 docs + attend `audit_findings` du Sprint 8 Phase 0 |
| 8 | DONE + CONDITIONAL PASS levé | `9339bb6` | 7 | 4 docs + `audit_findings` |
| 9 | DONE + CONDITIONAL PASS levé | `eb81c27` puis `48b332a` post-gate | 7 + 2 (gate) | 4 docs + `audit_findings` |
| 10 | DONE | `d07bfcf` (pre-Phase F) | 5 | 4 docs (kickoff, plan, verification, audit_plan) |
| 11 | DONE + CONDITIONAL PASS levé | `999fec6` puis `f2c94e3` post-gate | 6 + 2 (gate) | 4 docs + `audit_findings` |
| 12 | DONE + CONDITIONAL PASS levé | `bf3f009` puis `53a9e32` post-gate | 7 + 1 (gate) | 5 docs (kickoff, plan, verification, audit_plan, audit_findings) |
| 13 | DONE | `08853ff` (Phase E docs) | 6 (planning + A-D + docs) | 4 docs (kickoff, plan, verification, audit_plan) |

**Faits saillants** :

- **Sprint 6** est le premier a avoir les 4 docs planning complets des
  le demarrage.
- **Sprint 7** est le premier cycle complet de l'audit gate pattern
  (instaure post-S6 retro).
- **Sprint 10** est le premier sprint ops (CI/CD + 3 VPS bootstrap, pas
  de code applicatif).
- **Sprint 11** est le premier P2P end-to-end (publish + discovery +
  render plein ecran).
- **Sprint 12** est le premier rendu universel cross-node (archive zip
  → daemon blob-serve → iframe sandboxee).
- **Sprint 13** est le premier avec bridge iframe ↔ reseau
  (postMessage + open source enforcement + launcher Rust minimal).

Detail : [`.planning/archive/v1.0/`](../../.planning/archive/v1.0/).

---

## Conventions

### Quand mettre a jour ce log

- A la cloture du sprint N (Phase E commit) — ajouter la row dans la
  section v1.x correspondante avec etat `DONE`
- A la levee d'une CONDITIONAL PASS — mettre a jour l'etat avec
  `+ CONDITIONAL PASS levé` et le tip post-gate
- A l'ouverture d'une nouvelle version majeure — creer une nouvelle
  section `## v1.x — theme (Sx-Sy)` au-dessus de v1.x-1

### Quand creer un nouveau dossier `archive/v1.x/`

Quand le sprint qui s'ouvre adresse un theme suffisamment distinct
du precedent pour justifier une release majeure. Exemples :
- v1.0 → v1.1 : passage de "ca marche end-to-end" a "ca marche
  verifiable cryptographiquement"
- v1.1 → v1.2 : passage de "feature complete" a "production hardening"

Decision prise en kickoff §1 du sprint qui ouvre la nouvelle version.

### Migration historique

Avant Sprint 16, tous les `sprint{N}_*.md` vivaient a plat dans
`.planning/`. La migration vers `active/` + `archive/v{X}/` a ete
faite en Sprint 16 Phase 0 pour eviter que `docs/claude/README.md`
§10 ne devienne ingerable a 30+ sprints.
