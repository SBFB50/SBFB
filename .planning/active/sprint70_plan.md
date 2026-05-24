# Sprint 70 — Plan (Process Portable Complete + Gate 1 dogfood)

**Ecrit** : 2026-05-24.
**Tip master** : `c6c135f`.
**Roadmap** : Sprint 1/1, v2.1 Arc 2.5 Process Portable Complete.

---

## §1 Etat verifie a l'entree

| Suite | Count | Commande | Observed |
|---|---|---|---|
| Rust nextest | 1433 | `cargo nextest run --workspace --locked` | |
| Rust doctests | ok | `cargo test --workspace --locked --doc` | |
| cargo fmt | 0 diff | `cargo fmt --all --check` | |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` | |
| Vitest | 279 | `(cd web && npm run test:unit)` | |
| size-limit | 6/6 | `(cd web && npm run size)` | |
| release build | ok | `cargo build -p nexus-shell-daemon --release` | |
| **Total** | **~1718** | | |

---

## §2 Decisions Day 0 (gelees)

| D# | Decision | Implication code |
|---|---|---|
| D1 | AGENT_SYSTEM.md carte derivee (5 sections, pas duplication PROCESS.md) | `docs/agent/AGENT_SYSTEM.md` (NEW), `AGENTS.md` (update) |
| D2 | Handoff prompt portable via agentctl prompt --kind handoff | `prompts/agent/handoff.md` (NEW), `scripts/agent/agentctl.py`, `docs/agent/TOOLING.md` |
| D3 | Agentctl 3 commandes observabilite (status-sprint, lint-planning, audit-commit) | `scripts/agent/agentctl.py` (3 commandes), `tests/test_agentctl.py` (tests), `docs/agent/TOOLING.md` |
| D4 | Fermer bypasses hooks + aligner routing agents | `.claude/hooks/process-task-gate.sh`, `.claude/hooks/process-supervisor-stop.sh`, `scripts/agent/agentctl.py`, `AGENTS.md` |
| D5 | Contrat RRV/Factory : modes @ = alias roles portables | `docs/agent/RRV_FACTORY_CONTRACT.md` (NEW) |

---

## §3 Graphe de dependances inter-phases

```
Phase A (AGENT_SYSTEM.md)
  |
  +--> Phase B (dette pair + P2 absorbes)  [independante de A]
  |
  +--> Phase C (handoff) [depend A : reference AGENT_SYSTEM.md]
         |
         +--> Phase D (agentctl 3 commandes) [depend C : handoff dans PROMPT_KINDS]
                |
                +--> Phase E (hooks + dogfood) [depend D : utilise status-sprint/lint-planning]
                       |
                       +--> Phase F (contrat RRV + verification + wrap-up) [depend E : dogfood done]
```

Phase A et Phase B sont independantes et peuvent commencer en
parallele (A = docs agent, B = dette pair code/docs). Phase C
depend de A car le handoff reference AGENT_SYSTEM.md. Phase D
depend de C car le PROMPT_KINDS doit contenir "handoff" pour que
le build_parser() soit coherent. Phase E depend de D car le
dogfood utilise les commandes agentctl nouvelles. Phase F ferme le
sprint avec le contrat RRV/Factory et la verification.

---

## §4 Phase A — Canon portable (AGENT_SYSTEM.md + AGENTS.md)

### §4.1 Scope

Creer `docs/agent/AGENT_SYSTEM.md` comme carte du systeme agent.
7 sections derivees de PROCESS.md sans duplication : Truth Stack,
Role Registry (8 roles), Provider Mapping, Lifecycle Modes (10
modes), Gate Contract (verdict tree complet + artefact contract),
Prompt Registry (table des prompts executables dans prompts/agent/
avec kind, purpose, provider compatibility), Non-Goals.

Le Gate Contract formalise la logique de verdict aujourd'hui
enfermee dans les agents Claude :
- Preflight : EXECUTE / PLAN-ADAPT / SCOPE-CUT-CONSISTENT /
  DESIGN-CONFLICT (avec conditions de declenchement)
- Review : PASS-PENDING / CONCERN / FAIL (avec criteres par
  dimension)
- Codex : CLEAN / GAP-P0 / GAP-P1 / GAP-P2-P3
- Audit gate : PASS / CONDITIONAL PASS / FAIL (avec P0-P3)

Le Prompt Registry indique quel prompt portable utiliser pour
chaque gate, et quel provider peut l'executer avec quelle
profondeur.

Mettre a jour `AGENTS.md` racine pour corriger les references
stale (Python, setup.sh) et pointer vers AGENT_SYSTEM.md. Phase
docs-only.

### §4.2 Livrables

| Fichier | Description |
|---|---|
| `docs/agent/AGENT_SYSTEM.md` | NEW. Carte systeme 7 sections. Truth Stack (repo > planning > commits > prompt > chat). Role Registry (8 roles avec droits/obligations/limites). Provider Mapping (Claude, Codex, GPT, local, humain). Lifecycle Modes (10 modes pointant vers prompts/). Gate Contract (verdict tree complet : preflight 4 verdicts, review 3 verdicts, codex 4 verdicts, audit 3 verdicts + artefact contract par gate). Prompt Registry (table kind → fichier → purpose → compatible providers). Non-Goals. |
| `AGENTS.md` | UPDATE. Supprimer references Python/packages/setup.sh. Pointer vers docs/agent/AGENT_SYSTEM.md pour le detail roles. Garder la section Build/Test commands avec les commandes Rust/Frontend/agentctl actuelles. |

### §4.3 Tests plan

Phase docs-only. Pas de tests code.
Verification :
1. `test -f docs/agent/AGENT_SYSTEM.md` — fichier existe
2. `rg -n "Truth Stack|Role Registry|Provider Mapping|Lifecycle Modes|Gate Contract|Prompt Registry|Non-Goals" docs/agent/AGENT_SYSTEM.md` — 7 sections presentes
3. `! rg "packages/|setup\.sh|uv run pytest" AGENTS.md` — references stale supprimees

### §4.4 Critere d'acceptation

```bash
test -f docs/agent/AGENT_SYSTEM.md && \
rg -c "Truth Stack|Role Registry|Provider Mapping|Lifecycle|Gate Contract|Prompt Registry|Non-Goals" docs/agent/AGENT_SYSTEM.md | awk -F: '{s+=$2} END {print s >= 7 ? "PASS" : "FAIL"}'
```
Condition : 7 sections headers presentes. AGENTS.md ne contient plus de references Python.

### §4.5 Commit cible

`docs(agent): Sprint 70 Phase A — AGENT_SYSTEM.md canon portable + AGENTS.md cleanup`

Body : 9 sections obligatoires (Contexte, Fichiers, Delta tests, Verification §7.4, Scope cuts respectes, G8 traceability, Pre-launch protocol, Codex verification, Carry closure).

---

## §5 Phase B — Dette pair + P2-I-3 3/3 + P2 audit absorbables

### §5.1 Scope

Sprint pair, phase dette obligatoire (Regle 1 §6.2.1).
Resoudre P2-I-3 body docs minimaliste 3/3 MANDATORY : la
convention est que tout commit docs/feat significatif (>100 lignes)
a un body de 3-5 lignes minimum. La preuve est le body de la Phase
A (et de cette phase).

Absorber les P2 audit S69 :
- P2-C-1 : documenter dans PATTERNS.md la duplication canonical
  bytes Factory/coordinator comme dette connue T-NN+3 avec plan
  extraction nexus-core-rs post-S70.
- P2-C-2 : documenter dans PATTERNS.md la non-utilisation de JCS
  avec rationale pre-launch (payloads ASCII simples, pas de
  flottants).
- P2-I-1 : documenter dans docs/claude/README.md la regle chore/
  feat split : docs techniques (FACTORY_GATES.md, PATTERNS.md)
  appartiennent au commit feat correspondant, pas a un chore
  planning.
- P2-G-1 exe lock : CLOSE apres 8 sprints non-reproductibles.
  Documenter dans PATTERNS.md avec conditions de reouverture.

### §5.2 Livrables

| Fichier | Description |
|---|---|
| `docs/rust/PATTERNS.md` | UPDATE. Ajouter T-NN+3 (canonical bytes duplication Factory/coordinator, plan extraction). Ajouter note P2-C-2 (serde_json vs JCS, rationale pre-launch). CLOSE P2-G-1 avec conditions reouverture. |
| `docs/claude/README.md` | UPDATE. Ajouter une note §4.1 ou §6 : docs techniques dans feat, pas chore. |

### §5.3 Tests plan

Phase docs-only. Pas de tests code.
Verification :
1. `rg "T-NN\+3|canonical.bytes.duplication" docs/rust/PATTERNS.md` — T-NN+3 present
2. `rg "P2-G-1|exe.lock" docs/rust/PATTERNS.md` — CLOSE documente
3. `rg "chore.feat.split|docs.techniques" docs/claude/README.md` — regle documentee

### §5.4 Critere d'acceptation

```bash
rg -q "T-NN\+3" docs/rust/PATTERNS.md && \
rg -q "CLOSE.*P2-G-1\|P2-G-1.*CLOSE" docs/rust/PATTERNS.md && echo "PASS" || echo "FAIL"
```
Condition : T-NN+3 et P2-G-1 CLOSE documentes.

### §5.5 Commit cible

`docs(patterns): Sprint 70 Phase B — dette pair T-NN+3 + P2-G-1 CLOSE + chore/feat split`

Body : 9 sections obligatoires. P2-I-3 3/3 preuve = body >=3 lignes (mesurable). P2-C-1 1/3→documented. P2-C-2 1/3→documented. P2-I-1 1/3→documented. P2-G-1 monitoring→CLOSE.

---

## §6 Phase C — Prompt portability full (logique executable dans prompts/)

### §6.1 Scope

Migrer la logique executable des `.claude/agents/` vers
`prompts/agent/` pour que tout provider puisse executer les memes
gates. Les agents Claude deviennent des wrappers legers (Phase E).

6 prompts a creer ou enrichir :

1. **handoff.md** (NEW) : template 9 sections transfert
   inter-provider (sprint context, progress, changed files, test
   evidence, verdict state, stop conditions, assumptions NOT to
   inherit, active carries, next actions).

2. **preflight.md** (ENRICH) : ajouter les procedures executables
   des 5 scans S1-S4 avec commandes concretes (`git log --grep`,
   `rg -n`, `grep -rE`), verdict tree complet
   (EXECUTE/PLAN-ADAPT/SCOPE-CUT-CONSISTENT/DESIGN-CONFLICT),
   template de sortie structuree, et anti-patterns. Aujourd'hui le
   prompt decrit quoi scanner ; apres, il dit exactement comment
   avec quelles commandes et quel format de rapport.

3. **phase-review.md** (ENRICH) : ajouter les 11 dimensions du
   review-deep (staging coherence, scope-cuts semantique, branch
   coverage 4 criteres [appel reel/assertion/cas limites/inputs
   realistes], research grounding, security OWASP 9 patterns,
   patterns drift, horizon long-terme, livrables check, body
   format 9/9, codex reconciliation, carry routing). Template de
   sortie structuree avec verdict PASS-PENDING/CONCERN/FAIL.

4. **commit-body.md** (NEW) : template 9 sections obligatoires
   (Contexte, Fichiers, Delta tests, Verification §7.4, Scope cuts
   respectes, G8 traceability, Pre-launch protocol, Codex
   verification, Carry closure) + regles de validation (regex per
   section, anti-patterns LOC/emoji/amend).

5. **audit-gate-checks.md** (NEW) : 9 tracks audit (A suites,
   B security, C patterns, D scope, E tests delta, F review files,
   G carry-overs, H HARDENING, I meta-process) avec commandes
   concretes par track, classification P0-P3, verdict tree
   PASS/CONDITIONAL/FAIL.

6. **phase-auditor.md** (NEW) : 7 dimensions review leger
   (security, patterns, scope-cuts, research, G8, body-format,
   horizon) avec opinion-first pattern check.

Wirer les 6 kinds dans PROMPT_KINDS de agentctl.py. Inclure
`AGENT_SYSTEM.md` dans `agentctl context`.

### §6.2 Livrables

| Fichier | Description |
|---|---|
| `prompts/agent/handoff.md` | NEW. Template 9 sections transfert inter-provider. |
| `prompts/agent/preflight.md` | ENRICH. Ajouter procedures executables S1-S4 avec commandes concretes, verdict tree, template sortie, anti-patterns. |
| `prompts/agent/phase-review.md` | ENRICH. Ajouter 11 dimensions review-deep avec criteres, commandes, template sortie structuree. |
| `prompts/agent/commit-body.md` | NEW. Template 9 sections + validation regex + anti-patterns. |
| `prompts/agent/audit-gate-checks.md` | NEW. 9 tracks audit avec commandes, classification P0-P3, verdict tree. |
| `prompts/agent/phase-auditor.md` | NEW. 7 dimensions review leger avec opinion-first. |
| `scripts/agent/agentctl.py` | UPDATE. 6 kinds dans PROMPT_KINDS + AGENT_SYSTEM.md dans cmd_context(). |
| `docs/agent/TOOLING.md` | UPDATE. Documenter les 6 kinds avec exemples d'usage. |

### §6.3 Tests plan

1. `test_prompt_handoff_assembles` — agentctl prompt --kind handoff retourne contenu non-vide
2. `test_prompt_preflight_assembles` — agentctl prompt --kind preflight retourne contenu non-vide avec S1/S2/S3/S4
3. `test_prompt_review_assembles` — agentctl prompt --kind review retourne contenu avec 11 dimensions
4. `test_prompt_commit_body_assembles` — agentctl prompt --kind commit-body retourne template 9 sections
5. `test_prompt_audit_gate_assembles` — agentctl prompt --kind audit-gate retourne 9 tracks
6. `test_prompt_auditor_assembles` — agentctl prompt --kind auditor retourne 7 dimensions
7. `test_context_includes_agent_system` — cmd_context liste AGENT_SYSTEM.md

### §6.4 Critere d'acceptation

```bash
for kind in handoff preflight review commit-body audit-gate auditor; do
  python scripts/agent/agentctl.py prompt --kind $kind --depth deep > /dev/null 2>&1 || exit 1
done && \
python scripts/agent/agentctl.py context | rg -q "AGENT_SYSTEM" && echo "PASS" || echo "FAIL"
```
Condition : les 6 kinds s'assemblent sans erreur, context reference AGENT_SYSTEM.md.

### §6.5 Commit cible

`feat(agent): Sprint 70 Phase C — prompt portability full (6 kinds executables)`

Body : 9 sections obligatoires. Delta tests : +7 Python.

---

## §7 Phase D — Agentctl observabilite

### §7.1 Scope

Implanter 3 nouvelles commandes dans agentctl.py :
- `status-sprint` : etat sprint courant (sprint N, phases, artefacts)
- `lint-planning` : coherence artefacts planning
- `audit-commit --rev HEAD` : verifier un commit contre les regles

Ecrire les tests dans `tests/test_agentctl.py`. Mettre a jour
`docs/agent/TOOLING.md`. JSON output optionnel via `--json`.

### §7.2 Livrables

| Fichier | Description |
|---|---|
| `scripts/agent/agentctl.py` | UPDATE. 3 nouvelles commandes : `cmd_status_sprint()` (~60-80 lignes), `cmd_lint_planning()` (~80-100 lignes), `cmd_audit_commit()` (~60-80 lignes). Parsers dans `build_parser()`. Total : ~200-250 lignes ajoutees. |
| `tests/test_agentctl.py` | UPDATE. ~8-12 nouveaux tests couvrant les 3 commandes avec monkeypatch (pas besoin de repo reel). |
| `docs/agent/TOOLING.md` | UPDATE. Documenter les 3 commandes avec exemples et sorties attendues. |

### §7.3 Tests plan

1. `test_status_sprint_detects_active_kickoff` — monkeypatch des fichiers active/, verifie que status-sprint retourne le bon sprint number et detecte kickoff/plan
2. `test_status_sprint_json_output` — verifie que `--json` produit du JSON parsable
3. `test_status_sprint_no_active_sprint` — verifie comportement quand `.planning/active/` est vide
4. `test_lint_planning_detects_orphan_files` — injecte un fichier sprint N-2, verifie warning
5. `test_lint_planning_detects_pass_pending` — injecte un review avec PASS-PENDING, verifie error
6. `test_lint_planning_clean` — verifie retour 0 quand tout est coherent
7. `test_audit_commit_valid_phase_commit` — monkeypatch git log, verifie PASS sur un commit valide
8. `test_audit_commit_missing_review` — verifie erreur quand review manque pour un phase commit
9. `test_audit_commit_non_phase_commit` — verifie que les commits non-phase sont ok sans review
10. `test_audit_commit_missing_body_sections` — verifie detection des sections body manquantes

### §7.4 Critere d'acceptation

```bash
python scripts/agent/agentctl.py status-sprint && \
python scripts/agent/agentctl.py lint-planning && \
python scripts/agent/agentctl.py audit-commit --rev HEAD && \
uv run pytest tests/test_agentctl.py -q && echo "PASS" || echo "FAIL"
```
Condition : les 3 commandes s'executent sans crash, tests passes.

### §7.5 Commit cible

`feat(agent): Sprint 70 Phase D — agentctl status-sprint + lint-planning + audit-commit`

Body : 9 sections obligatoires. Delta tests : +10 Python (test_agentctl.py).

---

## §8 Phase E — Agent refactor + hooks + provider config + dogfood

### §8.1 Scope

3 volets :

**(a) Agent refactor** : les `.claude/agents/` deviennent des
wrappers legers qui referencent les prompts portables. La logique
executable vit dans `prompts/agent/`, les agents ajoutent les
outils Claude-specifiques (WebSearch, context7, Read 1M tokens).
Un provider sans ces outils execute le meme workflow mais avec
moins de profondeur (pas de prior art OSS live, pas de 1M tokens).

**(b) Hooks dynamises** : remplacer les hardcodes "sprint 67" dans
process-task-gate.sh et process-supervisor-stop.sh par detection
dynamique. Fermer le bypass chore(sprintN) Phase dans auditor-gate.

**(c) Provider config + dogfood** : creer
`docs/agent/PROVIDER_CONFIG.md` qui definit comment configurer le
driver LLM (qui code) et le verificateur LLM (qui review/audit).
Table des combinaisons supportees :
- Driver Claude + Verificateur Codex (actuel)
- Driver Claude + Verificateur Claude (fallback)
- Driver Codex/GPT/local + Verificateur Claude
- Driver LLM local + Verificateur LLM local (full offline)

`agentctl prompt` accepte `--provider {claude,codex,gpt,local,human}`
pour adapter le prompt assemble (ex: si provider=local, pas de
reference WebSearch/context7 dans les instructions).

Dogfood : generer un prompt preflight pour un provider non-Claude,
verifier que le format est executable, prouver que
status-sprint/lint-planning/audit-commit fonctionnent.

### §8.2 Livrables

| Fichier | Description |
|---|---|
| `.claude/agents/nexus-phase-preflight-deep.md` | REFACTOR. Garder les instructions Claude-specifiques (WebSearch, context7, Read 1M). Deleguer la logique des 5 scans au prompt portable `preflight.md` via reference. |
| `.claude/agents/nexus-phase-review-deep.md` | REFACTOR. Garder les instructions Claude-specifiques. Deleguer les 11 dimensions au prompt portable `phase-review.md`. |
| `.claude/agents/nexus-audit-gate.md` | REFACTOR. Garder les instructions Claude-specifiques. Deleguer les 9 tracks au prompt portable `audit-gate-checks.md`. |
| `.claude/agents/nexus-phase-auditor.md` | REFACTOR. Wrapper leger sur `phase-auditor.md` portable. Clarifier routing review-deep vs auditor. |
| `.claude/hooks/process-task-gate.sh` | UPDATE. Detection dynamique sprint courant. |
| `.claude/hooks/process-supervisor-stop.sh` | UPDATE. Detection dynamique sprint + phase. |
| `scripts/agent/agentctl.py` | UPDATE. Fix bypass chore(sprintN) Phase + `--provider` flag pour prompt assembly. |
| `docs/agent/PROVIDER_CONFIG.md` | NEW. Table driver/verificateur, combinaisons, instructions par provider. |

### §8.3 Tests plan

1. `test_auditor_gate_blocks_chore_sprint_phase` — chore(sprint70) Phase bloque sans review
2. `test_auditor_gate_allows_chore_planning` — chore(planning) passe
3. `test_prompt_provider_flag_local` — --provider local exclut WebSearch/context7
4. `test_prompt_provider_flag_claude` — --provider claude inclut tout

Dogfood :
5. `agentctl prompt --kind preflight --provider local --depth deep` — executable par LLM local
6. `agentctl status-sprint` + `lint-planning` + `audit-commit --rev HEAD`

### §8.4 Critere d'acceptation

```bash
! rg "sprint.?67" .claude/hooks/process-task-gate.sh .claude/hooks/process-supervisor-stop.sh && \
uv run pytest tests/test_agentctl.py -q -k "auditor or provider" && \
python scripts/agent/agentctl.py prompt --kind preflight --provider local --depth deep > /dev/null && \
python scripts/agent/agentctl.py prompt --kind handoff --depth deep > /dev/null && \
echo "PASS" || echo "FAIL"
```

### §8.5 Commit cible

`feat(agent): Sprint 70 Phase E — agent refactor wrappers + hooks dynamises + provider config + dogfood`

Body : 9 sections obligatoires. Delta tests : +4 Python.

---

## §9 Phase F — Contrat RRV/Factory + verification + wrap-up

### §9.1 Scope

Creer `docs/agent/RRV_FACTORY_CONTRACT.md` avec la table de mapping
modes→roles, le principe d'autorite, le contrat Factory, le
sequencing post-S70. Ecrire `sprint70_verification.md` fail-fast.
Ecrire `sprint71_audit_plan.md`. Mettre a jour CLAUDE.md (etat,
compteurs, carries), SPRINT_LOG.md, memory nexus_grid_pivot.md.

### §9.2 Livrables

| Fichier | Description |
|---|---|
| `docs/agent/RRV_FACTORY_CONTRACT.md` | NEW. Table mapping 5 modes @ → roles portables. Principe autorite (execution dans .planning/active/). Factory = consommateur. Babel = app. Sequencing post-S70. |
| `.planning/active/sprint70_verification.md` | NEW. Fail-fast checklist ~25-28 rows. Delta tests. Scope cuts compliance. G8 bilan. Carries. Commits. Checkpoint cloture. |
| `.planning/active/sprint71_audit_plan.md` | NEW. 9 tracks audit correspondant aux 9 tracks de sprint70_audit_plan.md. |
| `CLAUDE.md` | UPDATE. Tip, compteurs, carries, etat process. |
| `docs/claude/SPRINT_LOG.md` | UPDATE. Row S70. |

### §9.3 Tests plan

Phase docs-only. Pas de tests code.
Verification :
1. `test -f docs/agent/RRV_FACTORY_CONTRACT.md` — fichier existe
2. `rg -n "@research|@dev|@audit|@security|@product" docs/agent/RRV_FACTORY_CONTRACT.md` — 5 modes documentes
3. `test -f .planning/active/sprint70_verification.md` — verification ecrite
4. `test -f .planning/active/sprint71_audit_plan.md` — audit plan ecrit

### §9.4 Critere d'acceptation

```bash
test -f docs/agent/RRV_FACTORY_CONTRACT.md && \
test -f .planning/active/sprint70_verification.md && \
test -f .planning/active/sprint71_audit_plan.md && echo "PASS" || echo "FAIL"
```

### §9.5 Commit cible

`docs(sprint70): Sprint 70 Phase F — RRV/Factory contrat + verification + wrap-up`

Body : 9 sections obligatoires. Checkpoint cloture complet.

---

## §10 Delta tests estime

| Phase | Rust | Vitest | Python | Detail |
|---|---|---|---|---|
| A | +0 | +0 | +0 | docs-only (AGENT_SYSTEM.md 7 sections + AGENTS.md) |
| B | +0 | +0 | +0 | docs-only (PATTERNS.md + README.md) |
| C | +0 | +0 | +7 | 6 prompt kinds + context AGENT_SYSTEM |
| D | +0 | +0 | +10 | status-sprint, lint-planning, audit-commit |
| E | +0 | +0 | +4 | auditor gate bypass + provider flag |
| F | +0 | +0 | +0 | docs-only (RRV contract + verification) |
| **Total** | **+0** | **+0** | **+21** | |
| **Sortie estimee** | **1433** | **279** | **~32** | **~1744 + Python** |

Note : les tests Python ne sont pas comptes dans le total historique
Rust+Vitest. Le total test_agentctl.py passe de 11 a ~32.

---

## §11 Fail-fast checklist

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1433 |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build daemon | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | release build factory | `cargo build -p sbfb-factory --release` | ok |
| 7 | npm lint | `(cd web && npm run lint)` | 0 errors |
| 8 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors |
| 9 | Vitest | `(cd web && npm run test:unit)` | >= 279 |
| 10 | npm build | `(cd web && npm run build)` | ok |
| 11 | size-limit | `(cd web && npm run size)` | 6/6 |
| 12 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean |
| 13 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean |
| 14 | sync-bridge-sdk | `diff web/public/sbfb-bridge.js examples/*/sbfb-bridge.js` | identical |
| 15 | AGENT_SYSTEM.md exists | `test -f docs/agent/AGENT_SYSTEM.md` | exists |
| 16 | AGENT_SYSTEM 7 sections | `rg -c "Truth Stack\|Role Registry\|Provider Mapping\|Lifecycle\|Gate Contract\|Prompt Registry\|Non-Goals" docs/agent/AGENT_SYSTEM.md` | >= 7 |
| 17 | AGENTS.md no stale Python | `! rg "packages/\|setup\.sh\|uv run pytest" AGENTS.md` | absent |
| 18 | handoff.md exists | `test -f prompts/agent/handoff.md` | exists |
| 19 | agentctl 6 prompt kinds | `for k in handoff preflight review commit-body audit-gate auditor; do python scripts/agent/agentctl.py prompt --kind $k --depth deep > /dev/null; done` | exit 0 |
| 20 | agentctl status-sprint | `python scripts/agent/agentctl.py status-sprint` | exit 0 |
| 21 | agentctl lint-planning | `python scripts/agent/agentctl.py lint-planning` | exit 0 |
| 22 | agentctl audit-commit | `python scripts/agent/agentctl.py audit-commit --rev HEAD` | exit 0 |
| 23 | Python tests agentctl | `uv run pytest tests/test_agentctl.py -q` | >= 30 pass |
| 24 | hooks no stale S67 | `! rg "sprint.?67" .claude/hooks/process-task-gate.sh .claude/hooks/process-supervisor-stop.sh` | absent |
| 29 | PROVIDER_CONFIG.md | `test -f docs/agent/PROVIDER_CONFIG.md` | exists |
| 30 | provider flag works | `python scripts/agent/agentctl.py prompt --kind preflight --provider local --depth deep > /dev/null` | exit 0 |
| 25 | RRV_FACTORY_CONTRACT | `test -f docs/agent/RRV_FACTORY_CONTRACT.md` | exists |
| 26 | RRV 5 modes documented | `rg -c "@research\|@dev\|@audit\|@security\|@product" docs/agent/RRV_FACTORY_CONTRACT.md` | >= 5 |
| 27 | verification.md | `test -f .planning/active/sprint70_verification.md` | exists |
| 28 | audit_plan S71 | `test -f .planning/active/sprint71_audit_plan.md` | exists |

---

## §12 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | SearchManifest wire format + gossip | S71 | Protocole reseau hors scope process. |
| 2 | Page React /factory | S71+ | CLI suffit. UI Factory post-process. |
| 3 | @dev index tree-sitter | S71+ | @dev non bloquant Gate 1. |
| 4 | Template react-vite | S71+ | Pas de demande pilote. |
| 5 | CuratorVouched UI shell | S71+ | Vouch dans feed, UI post-pilote. |
| 6 | FG10 Review gate automatise | S71+ | Depend outillage post-Gate 1. |
| 7 | Fuzzing cargo-fuzz/proptest | post-Gate 1 | Hors scope fonctionnel. |
| 8 | Feed format version bump | post-launch | Pre-launch policy. |
| 9 | ProofCard comme feed op | S71+ | Candidat SearchManifest. |
| 10 | iroh 1.0 upgrade | Gate 1 decision | Evalue post-pilote. |
| 11 | CI process workflow | S71 | Stretch Phase E si temps, sinon S71. |
| 12 | Provider router multi-LLM | post-S75 | Manual copier-coller suffit. |
| 13 | sbfb-search app | S71+ | Depend SearchManifest. |
| 14 | Ingestion OSS broad | post-S75 | Mode source-only separe. |

---

## §13 Risks

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | agentctl status-sprint parse incorrectement artefacts S65-S69 | Medium | Medium | Tests sur patterns reels. |
| R2 | Hooks dynamises cassent sur sprint impair | Low | High | Tests manuels sur commits S69. |
| R3 | AGENT_SYSTEM.md duplique PROCESS.md | Medium | Low | Review G8 verifie non-duplication. |
| R4 | Handoff trop long pour LLM contexte court | Low | Medium | Mode standard (court) vs deep. |
| R5 | P2-I-3 3/3 non resolvable | Medium | Low | Convention simple (3-5 lignes). |
| R6 | Dogfood Phase E ne prouve rien si trivial | Medium | Medium | Changement reel obligatoire. |
| R7 | P2-G-1 CLOSE premature | Low | High | Conditions reouverture documentees. |

---

## §14 Checkpoint de cloture

- [ ] 30/30 fail-fast verts
- [ ] 6 commits : 2 docs (A + B) + 2 feat (C + D) + 1 feat (E) + 1 docs (F)
- [ ] verification.md + audit_plan S71 ecrits
- [ ] AGENT_SYSTEM.md cree (7 sections, Gate Contract + Prompt Registry)
- [ ] 6 prompt kinds executables dans prompts/agent/ (handoff, preflight, review, commit-body, audit-gate, auditor)
- [ ] agentctl prompt --kind X --provider Y fonctionne pour tout kind/provider
- [ ] .claude/agents/ refactored en wrappers legers sur prompts portables
- [ ] PROVIDER_CONFIG.md definit combinaisons driver/verificateur
- [ ] 3 commandes agentctl observabilite operationnelles
- [ ] Hooks stale S67 dynamises
- [ ] P2-I-3 3/3 CLOSED (body docs)
- [ ] P2-G-1 CLOSED (8 sprints non-repro)
- [ ] RRV_FACTORY_CONTRACT.md cree
- [ ] PATTERNS.md mis a jour (T-NN+3 + P2-G-1 CLOSE)
- [ ] Memory nexus_grid_pivot.md tip + compteurs a jour
- [ ] SPRINT_LOG.md row S70 ajoutee
