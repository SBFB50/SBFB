# Sprint 70 — Plan (Process Portable Complete + Gate 1 dogfood)

**Ecrit** : 2026-05-24.
**Tip master d'entree** : `c6c135f` (audit S69 PASS).
**Tip plan courant** : `3d6f4a9` (plan v3 ambitieux : serve + dashboard + 7 phases).
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
| D1 | AGENT_SYSTEM.md carte derivee (7 sections, pas duplication PROCESS.md) | `docs/agent/AGENT_SYSTEM.md` (NEW), `AGENTS.md` (update) |
| D2 | Prompt portability full + handoff inter-provider | `prompts/agent/handoff.md` (NEW), `prompts/agent/*` (ENRICH), `scripts/agent/agentctl.py`, `docs/agent/TOOLING.md` |
| D3 | Agentctl observabilite + serveur JSON local | `scripts/agent/agentctl.py` (status-sprint, lint-planning, audit-commit, serve), `tests/test_agentctl.py`, `docs/agent/TOOLING.md` |
| D4 | Hooks dynamiques + provider config + dogfood portable | `.claude/hooks/*`, `.claude/agents/*`, `scripts/agent/agentctl.py`, `docs/agent/PROVIDER_CONFIG.md` |
| D5 | Dashboard process operateur controle + contrat RRV/Factory | `tools/factory-dashboard/`, `docs/agent/RRV_FACTORY_CONTRACT.md` |

---

## §3 Graphe de dependances inter-phases

```
Phase A (AGENT_SYSTEM.md 7 sections)
  |
  +--> Phase B (dette pair + P2 absorbes)  [independante de A]
  |
  +--> Phase C (prompt portability full) [depend A : reference AGENT_SYSTEM]
         |
         +--> Phase D (agentctl 3 commandes + serve) [depend C : PROMPT_KINDS complet]
                |
                +--> Phase E (dashboard process operateur) [depend D : agentctl serve = backend]
                       |
                       +--> Phase F (agent refactor + hooks + provider config + dogfood via dashboard)
                              |
                              +--> Phase G (contrat RRV/Factory + verification + wrap-up)
```

Phase A et Phase B sont independantes. Phase C depend de A car
les prompts referencent AGENT_SYSTEM.md. Phase D depend de C car
PROMPT_KINDS doit etre complet + ajoute `agentctl serve` (JSON
API locale pour le dashboard). Phase E depend de D car le
dashboard appelle les endpoints agentctl serve. Phase F depend de
E car le dogfood utilise le dashboard pour prouver le process
portable end-to-end. Phase G ferme le sprint.

### §3.1 Reconciliation contrat process reel

Le contrat portable courant est `docs/agent/PROCESS.md` +
`docs/agent/TOOLING.md` + `AGENTS.md` +
`scripts/agent/agentctl.py` + `.githooks/`.
`docs/claude/README.md` reste le runbook Claude
historique : S70 doit l'aligner quand il contredit le contrat
portable, pas le traiter comme une autorite portable concurrente.

Etat reel observe avant S70 implementation :
- `agentctl.py` expose deja `context`, `prompt`,
  `codex-prompt-path`, `verify-on-write`, `precommit-lightcheck`,
  `auditor-gate`, `install-hooks`.
- `PROMPT_KINDS` contient deja `base`, `universal`, `preflight`,
  `phase-review`, `phase-auditor`, `commit-body`.
- `handoff`, `audit-gate`, `status-sprint`, `lint-planning`,
  `audit-commit`, `serve`, `--provider` et `context-pack` sont des
  livrables S70, pas des capacites deja acquises.
- Les hooks Git portables sont actifs via `.githooks/` quand
  `core.hooksPath=.githooks`; les hooks Claude restent des backstops
  de session, pas l'autorite portable.

Normalisations S70 obligatoires :
- verdict final exact : `## Verdict: PASS` uniquement ;
- S70 a 7 phases A-G ; les docs generiques doivent parler de
  "phase de sortie" ou "derniere phase planifiee", pas hardcoder
  Phase F/A-F ;
- tout commit `Sprint N Phase X` avec type valide
  (`feat|fix|docs|chore|test|refactor`) est un phase commit gate ;
- docs-only n'exempte jamais Codex, le review final PASS, ni le
  body 9 sections ; seules les suites lourdes peuvent etre exemptes
  avec justification ecrite dans review + commit body.

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

Mettre a jour `AGENTS.md` racine pour corriger seulement les
references stale averees (anciennes commandes SDK/setup obsoletes)
et pointer vers AGENT_SYSTEM.md. Ne pas supprimer les commandes
Python/agentctl encore utiles au process S70. Phase docs-only.

### §4.2 Livrables

| Fichier | Description |
|---|---|
| `docs/agent/AGENT_SYSTEM.md` | NEW. Carte systeme 7 sections. Truth Stack (repo > planning > commits > prompt > chat). Role Registry (8 roles avec droits/obligations/limites). Provider Mapping (Claude, Codex, GPT, local, humain). Lifecycle Modes (10 modes pointant vers prompts/). Gate Contract (verdict tree complet : preflight 4 verdicts, review 3 verdicts, codex 4 verdicts, audit 3 verdicts + artefact contract par gate). Prompt Registry (table kind → fichier → purpose → compatible providers). Non-Goals. |
| `AGENTS.md` | UPDATE. Supprimer references obsoletes ciblees (`packages/nexus-sdk`, `scripts/setup.sh`, anciens `uv run pytest packages/` si elles ne refletent plus le process actuel). Pointer vers docs/agent/AGENT_SYSTEM.md pour le detail roles. Garder les commandes Rust/Frontend/Python/agentctl actuelles et utiles. |

### §4.3 Tests plan

Phase docs-only : Codex review, final `## Verdict: PASS` et body
9 sections restent obligatoires. Les suites lourdes peuvent etre
exemptees seulement avec justification ecrite dans la review et le
commit body.
Verification :
1. `test -f docs/agent/AGENT_SYSTEM.md` — fichier existe
2. `rg -n "Truth Stack|Role Registry|Provider Mapping|Lifecycle Modes|Gate Contract|Prompt Registry|Non-Goals" docs/agent/AGENT_SYSTEM.md` — 7 sections presentes
3. `! rg "packages/nexus-sdk|scripts/setup\.sh|uv run pytest packages/" AGENTS.md` — references stale ciblees supprimees
4. `uv run pytest tests/test_agentctl.py -q` — tooling process portable toujours vert

### §4.4 Critere d'acceptation

```bash
test -f docs/agent/AGENT_SYSTEM.md && \
rg -c "Truth Stack|Role Registry|Provider Mapping|Lifecycle|Gate Contract|Prompt Registry|Non-Goals" docs/agent/AGENT_SYSTEM.md | awk -F: '{s+=$2} END {print s >= 7 ? "PASS" : "FAIL"}' && \
! rg "packages/nexus-sdk|scripts/setup\.sh|uv run pytest packages/" AGENTS.md && \
uv run pytest tests/test_agentctl.py -q
```
Condition : 7 sections headers presentes. AGENTS.md ne contient plus
les references stale ciblees et conserve les commandes Python/agentctl
reelles.

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

Normaliser aussi `docs/claude/README.md` contre le contrat portable :
- remplacer les exemples `## Verdict : PASS` par `## Verdict: PASS` ;
- remplacer les formulations generiques "A-F" / "Phase F wrap-up"
  par "phases planifiees" / "phase de sortie" quand ce n'est pas
  un exemple historique ;
- clarifier que tout commit `Sprint N Phase X` avec type valide
  (`feat|fix|docs|chore|test|refactor`) est un phase commit gate ;
- clarifier que docs-only n'exempte ni Codex, ni review PASS, ni
  body 9 sections.

### §5.2 Livrables

| Fichier | Description |
|---|---|
| `docs/rust/PATTERNS.md` | UPDATE. Ajouter T-NN+3 (canonical bytes duplication Factory/coordinator, plan extraction). Ajouter note P2-C-2 (serde_json vs JCS, rationale pre-launch). CLOSE P2-G-1 avec conditions reouverture. |
| `docs/claude/README.md` | UPDATE. Ajouter une note §4.1 ou §6 : docs techniques dans feat, pas chore. Aligner le runbook Claude avec le contrat portable (`## Verdict: PASS`, phases A-G/phase de sortie, tous types `Sprint N Phase X` gates, docs-only sans exemption Codex/body). |

### §5.3 Tests plan

Phase docs-only : Codex review, final `## Verdict: PASS` et body
9 sections restent obligatoires. Les suites lourdes peuvent etre
exemptees seulement avec justification ecrite.
Verification :
1. `rg "T-NN\+3|canonical.bytes.duplication" docs/rust/PATTERNS.md` — T-NN+3 present
2. `rg "P2-G-1|exe.lock" docs/rust/PATTERNS.md` — CLOSE documente
3. `rg "chore.feat.split|docs.techniques" docs/claude/README.md` — regle documentee
4. `! rg "^## Verdict : PASS$" docs/agent docs/claude .planning/active` — verdict espace non-canonique absent
5. `rg "phase de sortie|derniere phase planifiee" docs/claude/README.md` — pas de hardcode generique Phase F/A-F

### §5.4 Critere d'acceptation

```bash
rg -q "T-NN\+3" docs/rust/PATTERNS.md && \
rg -q "CLOSE.*P2-G-1\|P2-G-1.*CLOSE" docs/rust/PATTERNS.md && \
! rg "^## Verdict : PASS$" docs/agent docs/claude .planning/active && \
echo "PASS" || echo "FAIL"
```
Condition : T-NN+3 et P2-G-1 CLOSE documentes, et le runbook Claude
ne contredit plus le contrat portable sur le verdict final.

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

4. **commit-body.md** (ENRICH) : template 9 sections obligatoires
   (Contexte, Fichiers, Delta tests, Verification §7.4, Scope cuts
   respectes, G8 traceability, Pre-launch protocol, Codex
   verification, Carry closure) + regles de validation (regex per
   section, anti-patterns LOC/emoji/amend).

5. **audit-gate-checks.md** (NEW) : 9 tracks audit (A suites,
   B security, C patterns, D scope, E tests delta, F review files,
   G carry-overs, H HARDENING, I meta-process) avec commandes
   concretes par track, classification P0-P3, verdict tree
   PASS/CONDITIONAL/FAIL.

6. **phase-auditor.md** (ENRICH) : 7 dimensions review leger
   (security, patterns, scope-cuts, research, G8, body-format,
   horizon) avec opinion-first pattern check.

Wirer les kinds portables dans PROMPT_KINDS de agentctl.py :
`handoff`, `preflight`, `phase-review`, `commit-body`, `audit-gate`,
`phase-auditor`. Les aliases UI `review` -> `phase-review` et
`auditor` -> `phase-auditor` sont autorises seulement s'ils sont
testes explicitement. Inclure `AGENT_SYSTEM.md` dans `agentctl
context`.

Contrat bootstrap session fraiche :
1. `base.md` donne l'orientation invariante et les regles evidence.
2. `universal.md` donne le lifecycle sprint complet et les gates.
3. `agentctl context` + `prompt_context(depth=deep)` donnent les
   faits runtime : repo, sprint actif, phase, HEAD, dirty files,
   staged files, recent commits.
4. `handoff.md` donne l'etat point-in-time : phase courante,
   phases terminees, fichiers changes, derniers tests, verdict
   state, stop conditions, carries actifs, prochaines 1-3 actions.
5. Le prompt specialise (`preflight`, `phase-review`,
   `phase-auditor`, `commit-body`, `audit-gate`) donne la prochaine
   action de gate.

La memoire de chat privee et la memoire modele ne sont jamais
autoritaires. Si un fait n'est pas dans les fichiers repo, le
runtime context ou le handoff, l'agent receveur doit ecrire
`Not evidenced` au lieu de l'heriter.

### §6.2 Livrables

| Fichier | Description |
|---|---|
| `prompts/agent/handoff.md` | NEW. Template 9 sections transfert inter-provider. |
| `prompts/agent/preflight.md` | ENRICH. Ajouter procedures executables S1-S4 avec commandes concretes, verdict tree, template sortie, anti-patterns. |
| `prompts/agent/phase-review.md` | ENRICH. Ajouter 11 dimensions review-deep avec criteres, commandes, template sortie structuree. |
| `prompts/agent/commit-body.md` | ENRICH. Template 9 sections + validation regex + anti-patterns. |
| `prompts/agent/audit-gate-checks.md` | NEW. 9 tracks audit avec commandes, classification P0-P3, verdict tree. |
| `prompts/agent/phase-auditor.md` | ENRICH. 7 dimensions review leger avec opinion-first. |
| `scripts/agent/agentctl.py` | UPDATE. Kinds portables dans PROMPT_KINDS + aliases optionnels testes + AGENT_SYSTEM.md dans cmd_context(). |
| `docs/agent/PROCESS.md` | UPDATE. Remplacer le flou "universal = handoff complet" par la matrice explicite : base = orientation, universal = lifecycle, context = faits repo live, handoff = transfert point-in-time, prompts specialises = prochain gate. |
| `docs/agent/TOOLING.md` | UPDATE. Documenter les 6 kinds avec exemples d'usage operateur et details techniques. |

### §6.3 Tests plan

1. `test_prompt_handoff_assembles` — agentctl prompt --kind handoff retourne contenu non-vide
2. `test_prompt_preflight_assembles` — agentctl prompt --kind preflight retourne contenu non-vide avec S1/S2/S3/S4
3. `test_prompt_phase_review_assembles` — agentctl prompt --kind phase-review retourne contenu avec 11 dimensions
4. `test_prompt_commit_body_assembles` — agentctl prompt --kind commit-body retourne template 9 sections
5. `test_prompt_audit_gate_assembles` — agentctl prompt --kind audit-gate retourne 9 tracks
6. `test_prompt_phase_auditor_assembles` — agentctl prompt --kind phase-auditor retourne 7 dimensions
7. `test_context_includes_agent_system` — cmd_context liste AGENT_SYSTEM.md
8. `test_process_docs_describe_fresh_session_bootstrap` — PROCESS.md documente base/universal/context/handoff/specialized sans memoire chat

### §6.4 Critere d'acceptation

```bash
for kind in handoff preflight phase-review commit-body audit-gate phase-auditor; do
  python scripts/agent/agentctl.py prompt --kind $kind --depth deep > /dev/null 2>&1 || exit 1
done && \
python scripts/agent/agentctl.py context | rg -q "AGENT_SYSTEM" && echo "PASS" || echo "FAIL"
```
Condition : les 6 kinds s'assemblent sans erreur, context reference AGENT_SYSTEM.md.

### §6.5 Commit cible

`feat(agent): Sprint 70 Phase C — prompt portability full (6 kinds executables)`

Body : 9 sections obligatoires. Delta tests : +8 Python.

---

## §7 Phase D — Agentctl observabilite + serve

### §7.1 Scope

Implanter 4 nouvelles commandes dans agentctl.py :
- `status-sprint` : etat sprint courant (sprint N, phases, artefacts)
- `lint-planning` : coherence artefacts planning
- `audit-commit --rev HEAD` : verifier un commit contre les regles
- `serve --port 3001` : JSON API locale pour le dashboard Factory
- `serve --port 3001 --once-smoke` : demarre, verifie `/api/status`,
  puis s'arrete (preuve CI-friendly sans serveur long-running)

La commande `serve` expose les endpoints JSON :
- `GET /api/status` → status-sprint en JSON
- `GET /api/lint` → lint-planning en JSON
- `GET /api/audit/{rev}` → audit-commit en JSON
- `GET /api/prompt/{kind}?provider={p}&depth={d}` → prompt assemble
- `GET /api/context` → agentctl context en JSON
- `POST /api/context-pack` -> cree un paquet de reprise pour une
  nouvelle session : snapshot du prompt de base, contexte repo courant,
  intention utilisateur, provider, sprint/phase, budget de contexte,
  prompt specialise, et details techniques repliables
- `GET /api/providers` → liste statique des providers supportes
  (Phase F enrichit avec `PROVIDER_CONFIG.md` et detection config)
- `POST /api/actions/run` → execute une action agentctl allowlistee
  (`status-sprint`, `lint-planning`, `audit-commit`, `prompt`) sans
  shell arbitraire via l'API dashboard. Les demandes shell restent
  des intents Agent Chat : elles ne peuvent etre executees que par
  une vraie session agent si le provider et l'environnement
  l'autorisent, puis journalisees et prouvees par gates + preuves
  repo-visibles.
- `POST /api/artifacts/draft` → ecrit un brouillon sur une allowlist
  repo-visible (`.planning/active/**`, `docs/agent/**`,
  `docs/claude/**`, `prompts/agent/**`, `AGENTS.md`, `CLAUDE.md`)
  avec garde-fous (path guard, preview diff, confirmation,
  interdiction de creer un verdict final PASS hors flow review/gate)
- `GET /api/actions/log` → journal JSONL des actions dashboard
- `POST /api/chat/session` -> cree une discussion operateur agent a
  partir d'un context-pack
- `POST /api/chat/message` -> ajoute un message utilisateur et
  retourne la reponse/action proposee ou executee par l'agent. Les
  demandes sensibles (`shell`, `commit`, `push`, promotion PASS)
  retournent `requires_gate` ou `requires_external_agent` si elles ne
  peuvent pas etre executees directement par une vraie session agent
  avec preuves repo-visibles.
- `GET /api/chat/{id}/log` -> transcript JSONL local de la discussion,
  avec liens vers prompts, actions, drafts et commandes lancees

Basee sur `http.server` stdlib Python (zero dep externe). CORS
permissif en local (`localhost:*`).

Schema canonique `context-pack` sans transcript de chat brut :
- `base_prompt` : path + hash court de `prompts/agent/base.md` ;
- `universal_prompt` : path + hash court de
  `prompts/agent/universal.md` ;
- `handoff_prompt` : path + hash court de `prompts/agent/handoff.md` ;
- `specialized_prompt` : kind, path + hash court du prompt de gate ;
- `runtime_context` : repo, branch, HEAD, sprint, phase,
  dirty/staged files, recent commits ;
- `agent_system` : path + hash court de `docs/agent/AGENT_SYSTEM.md` ;
- `process_docs` : path + hash court de `docs/agent/PROCESS.md`,
  `docs/agent/TOOLING.md`, `AGENTS.md`, `CLAUDE.md` ;
- `active_artifacts` : fichiers `.planning/active/` pertinents et
  hash court ;
- `operator_intent` : intention UI, role (`driver`, `reviewer`,
  `auditor`, `researcher`, `local`), provider cible, depth/budget ;
- `chat_history_authoritative: false` ;
- aucun champ `chat_history`.

Le paquet doit contenir la phrase explicite
`private chat history is non-authoritative`.

Pour que le dashboard Phase E soit utilisable avant la refactor Phase F,
Phase D ajoute aussi le parsing de base `agentctl prompt --provider
{claude,codex,gpt,local,human}` et le transmet a `/api/prompt`. La
politique fine d'adaptation provider (retirer WebSearch/context7 pour
local, etc.) est durcie en Phase F.

Ecrire les tests dans `tests/test_agentctl.py`. Mettre a jour
`docs/agent/TOOLING.md`. JSON output optionnel via `--json` pour
les 3 commandes CLI et documentation des endpoints serve/context-pack/chat.

### §7.2 Livrables

| Fichier | Description |
|---|---|
| `scripts/agent/agentctl.py` | UPDATE. 4 nouvelles commandes : `cmd_status_sprint()`, `cmd_lint_planning()`, `cmd_audit_commit()`, `cmd_serve()`. Parsers dans `build_parser()`. Serve utilise `http.server.HTTPServer` + handler JSON + `--once-smoke` CI-friendly. `prompt --provider` accepte les 5 providers pour que `/api/prompt` soit stable. Actions POST allowlistees, context-pack generation, Agent Chat, draft artifacts sur allowlist repo-visible, action log JSONL. |
| `tests/test_agentctl.py` | UPDATE. Tests couvrant les 3 commandes CLI + tests serve (start/stop, endpoints JSON, context-pack, Agent Chat, action guards). |
| `docs/agent/TOOLING.md` | UPDATE. Documenter les 4 commandes, endpoints serve/context-pack/chat, exemples et sorties attendues. |

### §7.3 Tests plan

1. `test_status_sprint_detects_active_kickoff` — monkeypatch des fichiers active/, verifie sprint number + kickoff/plan detectes
2. `test_status_sprint_json_output` — `--json` produit du JSON parsable
3. `test_status_sprint_no_active_sprint` — comportement quand `.planning/active/` est vide
4. `test_lint_planning_detects_orphan_files` — fichier sprint N-2, warning
5. `test_lint_planning_detects_pass_pending` — review PASS-PENDING, error
6. `test_lint_planning_clean` — retour 0 quand coherent
7. `test_audit_commit_valid_phase_commit` — PASS sur commit valide
8. `test_audit_commit_missing_review` — erreur quand review manque
9. `test_audit_commit_non_phase_commit` — commits non-phase ok sans review
10. `test_audit_commit_missing_body_sections` — detection sections manquantes
11. `test_serve_status_endpoint` — GET /api/status retourne JSON parsable
12. `test_serve_prompt_endpoint` — GET /api/prompt/preflight retourne contenu
13. `test_serve_once_smoke` — `serve --once-smoke` retourne 0 et libere le port
14. `test_prompt_provider_arg_accepted` — `prompt --provider local` ne crash pas
15. `test_serve_lint_endpoint` — GET /api/lint retourne JSON parsable
16. `test_serve_audit_endpoint` — GET /api/audit/{rev} retourne JSON parsable
17. `test_serve_context_endpoint` — GET /api/context retourne JSON parsable
18. `test_serve_providers_endpoint` — GET /api/providers retourne JSON parsable
19. `test_serve_actions_log_endpoint` — GET /api/actions/log retourne JSON parsable
20. `test_serve_action_rejects_unlisted_command` — pas de shell arbitraire via `/api/actions/run`
21. `test_serve_context_pack_schema_complete` — nouveau contexte contient path/hash base, universal, handoff, specialized_prompt, runtime_context, AGENT_SYSTEM, process docs, active artifacts, provider, intention
22. `test_serve_context_pack_includes_base_snapshot` — nouveau contexte contient path/hash `base.md`, HEAD, sprint/phase, provider, intention, sans dependance a l'historique chat
23. `test_serve_context_pack_rejects_chat_history_authority` — pas de champ `chat_history`, `chat_history_authoritative` vaut false, phrase non-authoritative presente
24. `test_serve_chat_message_endpoint` — POST /api/chat/message retourne reponse JSON parsable
25. `test_serve_chat_log_endpoint` — GET /api/chat/{id}/log retourne transcript JSONL/JSON parsable
26. `test_serve_chat_session_starts_from_context_pack` — une discussion agent demarre depuis base/universal/context/handoff, pas depuis chat history
27. `test_serve_chat_logs_messages_and_actions` — messages, actions, drafts et commandes sont journalises
28. `test_serve_chat_rejects_sensitive_action_execution` — shell/commit/push/PASS direct retourne `requires_gate` ou `requires_external_agent`
29. `test_serve_artifact_draft_rejects_pass_verdict` — impossible d'ecrire un PASS final via dashboard sans passer par le flow review/gate
30. `test_serve_artifact_draft_logs_action` — toute ecriture dashboard est journalisee

### §7.4 Critere d'acceptation

```bash
python scripts/agent/agentctl.py status-sprint && \
python scripts/agent/agentctl.py lint-planning && \
python scripts/agent/agentctl.py audit-commit --rev HEAD && \
python scripts/agent/agentctl.py serve --port 3001 --once-smoke && \
uv run pytest tests/test_agentctl.py -q && echo "PASS" || echo "FAIL"
```
Condition : les 4 commandes s'executent sans crash, les endpoints
serve/context-pack/chat repondent en JSON parsable, les actions
sensibles retournent un statut gate/external-agent, tests passes.

### §7.5 Commit cible

`feat(agent): Sprint 70 Phase D — agentctl observabilite + serve JSON API`

Body : 9 sections obligatoires. Delta tests : +30 Python.

---

## §8 Phase E — Dashboard process operateur (Factory-ready, action-gated)

### §8.1 Scope

Creer une app React standalone dans `tools/factory-dashboard/` qui
sert de cockpit operateur process et de fondation UI Factory future.
L'app appelle `agentctl serve` comme backend JSON API. Elle peut
declencher des actions controlees : relancer status/lint/audit,
generer un prompt, preparer un brouillon d'artefact process, et
journaliser l'action. Elle fournit aussi un module de discussion
operateur-agent : l'utilisateur peut parler au systeme comme dans le
chat actuel, demander une mise a jour, demander un audit, demander un
commit/push, ou demander la generation d'un nouveau contexte.

Frontiere d'autorite : le dashboard UI ne fabrique pas lui-meme un
verdict, un commit, un push ou une decision de gate. Il pilote une
session agent et rend les preuves visibles. Une session agent peut
effectuer les memes operations que le flux actuel si le provider et
l'environnement l'autorisent, mais les operations sensibles passent
par le contrat repo-visible : prompt/context-pack, action log, diff,
review, Codex/gate et commit body. Un `## Verdict: PASS` n'est valide
que s'il est produit par le flow review/gate, pas par un bouton UI.

Stack : Vite + React + TypeScript + Tailwind + shadcn/ui (meme
stack que web/ pour coherence et reutilisation des composants).

**Principe UX obligatoire :** aucun utilisateur ne doit voir une commande
`agentctl prompt --kind preflight --sprint 70 --phase C --provider local`
comme action principale. L'UI expose des intentions metier en francais, puis
traduit en commande interne dans un panneau "details techniques" repliable.

Glossaire UI :
- `preflight` → "Preparer la phase"
- `phase-review` → "Relire la phase"
- `audit-gate` → "Auditer le sprint"
- `phase-auditor` → "Verifier avant validation"
- `commit-body` → "Preparer le message de commit"
- `handoff` → "Transmettre a un autre agent"
- `provider local` → "Agent local/offline"
- `provider claude/codex/gpt` → "Agent Claude/Codex/GPT"

Flux operateur canonique :
1. Sprint Overview lit l'etat repo via `/api/status`.
2. L'utilisateur choisit "Qui code ?" et "Qui verifie ?".
3. L'operateur commence par coller un prompt de base existant ou
   generer un nouveau contexte depuis le repo.
4. "Nouveau contexte / Transmettre a un autre agent" ouvre un
   builder : intention, sprint/phase, provider cible, role,
   budget contexte, profondeur.
5. Le dashboard appelle `POST /api/context-pack`.
6. Le paquet assemble base/universal/context/handoff/prompt
   specialise avec path/hash, HEAD, dirty files,
   sprint/phase, provider, prompt specialise, fichiers a lire,
   stop conditions, limites, et `chat_history_authoritative: false`.
7. Les CTA restent metier ; `agentctl`, `kind`, `provider`,
   `preflight` restent dans "details techniques".
8. L'utilisateur peut continuer en discussion libre dans Agent Chat :
   "mets a jour le plan", "audite cette phase", "prepare le commit",
   "transmets a un agent local". Le chat utilise le context-pack comme
   base et journalise messages + actions.
9. Les actions allowlistees produisent resultats et logs. Les actions
   sensibles (shell/commit/push/verdict final) ne sont pas de simples
   raccourcis UI : elles passent par une vraie session agent + gates +
   preuves repo-visibles + journalisation.
10. Les drafts repo/docs sur allowlist exigent path guard, preview
    diff, confirmation et action log. Un `## Verdict: PASS` n'est
    accepte que via le flow review/gate.
11. La source d'autorite reste `.planning/active/`, reviews, commits
   et gates. localStorage, dashboard state et action log sont des
   projections/evidences.

**Pages / vues :**

1. **Sprint Overview** — statut sprint courant en temps reel
   (numero, phases avec badges etat, artefacts presents/manquants,
   verdicts par gate, compteurs tests). Appelle `GET /api/status`.

2. **Agent Selector** — choix "Qui code ?" + "Qui verifie ?" via
   dropdown en langage humain : Claude, Codex, GPT, agent local/offline,
   humain. Persiste dans localStorage. Appelle `GET /api/providers`.

3. **Assistant de phase** — workflow guide par intentions :
   "Preparer la phase", "Relire la phase", "Verifier avant validation",
   "Preparer le message de commit", "Transmettre a un autre agent",
   ou "Discuter avec l'agent".
   L'utilisateur choisit Sprint/Phase via select ou detection automatique.
   Le dashboard traduit ensuite vers le kind canonique interne
   (preflight, phase-review, audit-gate, commit-body, phase-auditor,
   handoff) et affiche la commande seulement dans "details techniques".
   Appelle `GET /api/prompt/{kind}`.

4. **Lint Dashboard** — resultats lint-planning en visuel
   (warnings/errors avec fichiers concernes). Appelle
   `GET /api/lint`.

5. **Commit Auditor** — entrer un SHA, afficher le resultat
   audit-commit (sections presentes/manquantes, review check,
   codex check). Appelle `GET /api/audit/{rev}`.

6. **Transfert agent / Nouveau contexte** — importe ou genere le
   prompt de base, assemble le context-pack, puis propose
   Copier/Ouvrir l'agent cible apres validation. Appelle
   `GET /api/prompt/handoff` + `POST /api/context-pack`.

7. **Context Pack Builder** — cree un nouveau contexte complet pour
   un provider/role cible via `POST /api/context-pack`, affiche le
   contenu en langage operateur et la commande seulement dans les
   details techniques.

8. **Action Center** — file d'actions proposees par Agent Chat et
   validees par l'operateur. Execution directe limitee a
   status/lint/audit/prompt/draft ; shell/commit/push restent des
   intentions guidees qui ouvrent checklist gates + session agent +
   preuve repo.

9. **Draft Artifact Dialog** — target path sur allowlist repo-visible,
   preview diff, confirmation explicite, rejet des verdicts PASS hors
   flow review/gate.

10. **Authority Boundary** — composant partage qui rappelle que le
    dashboard est projection/action surface, pas autorite finale.

11. **Agent Chat** — discussion operateur comme le chat actuel :
    saisie libre, reponses agent, actions proposees/executees,
    contexte visible, transcript local, et liens vers drafts/logs.
    Demarre toujours depuis un context-pack.

Design : dark theme (coherent avec le shell SBFB), responsive,
sidebar navigation, status bar avec tip HEAD + sprint number.

### §8.2 Livrables

| Fichier | Description |
|---|---|
| `tools/factory-dashboard/` | NEW. Projet Vite + React + TypeScript + Tailwind, action-gated. |
| `tools/factory-dashboard/package.json` | NEW. Deps : react, react-dom, vite, tailwindcss, typescript. |
| `tools/factory-dashboard/src/App.tsx` | NEW. Router + layout (sidebar + content). |
| `tools/factory-dashboard/src/pages/SprintOverview.tsx` | NEW. Status sprint + phases + verdicts. |
| `tools/factory-dashboard/src/pages/AgentSelector.tsx` | NEW. Dropdown "Qui code ?" + "Qui verifie ?" avec libelles humains. |
| `tools/factory-dashboard/src/pages/PhaseAssistant.tsx` | NEW. Intentions utilisateur → kind interne → prompt/action. |
| `tools/factory-dashboard/src/pages/LintDashboard.tsx` | NEW. Resultats lint visuels. |
| `tools/factory-dashboard/src/pages/CommitAuditor.tsx` | NEW. SHA → audit result. |
| `tools/factory-dashboard/src/pages/AgentTransfer.tsx` | NEW. Transfert agent / nouveau contexte : import/generation prompt de base, context-pack, copier/ouvrir agent cible. |
| `tools/factory-dashboard/src/pages/ContextPackBuilder.tsx` | NEW. Nouveau contexte / transfert agent via `/api/context-pack`. |
| `tools/factory-dashboard/src/pages/AgentChat.tsx` | NEW. Discussion operateur agent, basee sur context-pack, avec transcript + actions liees. |
| `tools/factory-dashboard/src/pages/ActionCenter.tsx` | NEW. Actions allowlistees + resultat + action id. |
| `tools/factory-dashboard/src/pages/ActionLog.tsx` | NEW. Journal des actions dashboard et brouillons ecrits. |
| `tools/factory-dashboard/src/hooks/useApi.ts` | NEW. Hook fetch vers agentctl serve. |
| `tools/factory-dashboard/src/components/` | NEW. StatusBadge, VerdictChip, MarkdownRenderer, CopyButton, ConfirmActionDialog, DiffPreview, DraftArtifactDialog, AuthorityBoundary, TechnicalDetails. |

### §8.3 Tests plan

1. `npm run build` — le projet compile sans erreur
2. `npm run lint` — 0 errors ESLint
3. `npx tsc --noEmit` — 0 errors TypeScript
4. Test integration manuelle : `agentctl serve` + ouvrir dashboard
   → Sprint Overview affiche le bon sprint, Assistant de phase propose
   "Preparer la phase" sans exposer `agentctl` en action principale,
   Lint Dashboard affiche les resultats, ActionLog montre les actions
   lancees, un brouillon d'artefact demande une confirmation + preview diff
5. Test UX : `rg -n "agentctl prompt --kind|preflight|phase-review|provider local" tools/factory-dashboard/src` ne trouve ces termes que dans un composant `TechnicalDetails` ou dans les tests, jamais comme libelle de bouton principal
6. ContextPackBuilder genere un paquet pour "Agent local/offline"
   sans chat history authoritative
7. Le choix "Qui code ?" / "Qui verifie ?" modifie provider/role du
   prompt produit
8. Meme provider driver + verifier affiche un warning
   d'independance
9. DraftArtifactDialog refuse un verdict final PASS hors flow review/gate
10. ActionCenter n'expose que status/lint/audit/prompt/draft,
    et route les demandes sensibles vers Agent Chat + gates
11. AgentChat accepte une demande libre ("mets a jour le plan") et
    l'associe a un context-pack + transcript + action log
12. Une demande shell/commit/push affiche les gates requis et ne peut
    etre marquee complete que si une vraie session agent + le flow
    repo-visible le prouvent
13. Smoke operateur : coller/generer base prompt → discuter dans
    Agent Chat → proposer draft repo/docs → demander commit/push →
    afficher gates requis + action log, sans bouton d'autorite finale

Acceptance UX bloquante : `agentctl`, `--kind`, `provider`,
`preflight`, `phase-review`, `phase-auditor`, `audit-gate`, et
`commit-body` peuvent apparaitre seulement dans `TechnicalDetails`,
logs, tests ou docs developpeur. Les boutons principaux, titres,
tabs, empty states et confirmations utilisent des intentions
operateur en francais.

### §8.4 Critere d'acceptation

```bash
(cd tools/factory-dashboard && npm install && npm run lint && \
  npx tsc --noEmit && npm run build) && echo "PASS" || echo "FAIL"
```
Condition : le projet compile, lint et typecheck passent.
Acceptance Phase E exige aussi un smoke operateur : coller/generer
base prompt -> discuter dans Agent Chat -> proposer draft repo/docs
sur allowlist -> demander shell/commit/push -> afficher gates requis
+ action log, sans bouton d'autorite finale.

### §8.5 Commit cible

`feat(factory): Sprint 70 Phase E — dashboard process operateur action-gated`

Body : 9 sections obligatoires. Delta tests : +0 Rust, +0 Vitest,
build/lint/tsc factory-dashboard.

---

## §9 Phase F — Agent refactor + hooks + provider config + dogfood

### §9.1 Scope

3 volets :

**(a) Agent refactor** : les `.claude/agents/` deviennent des
wrappers legers qui referencent les prompts portables. La logique
executable vit dans `prompts/agent/`, les agents ajoutent les
outils Claude-specifiques (WebSearch, context7, Read 1M tokens).
Un provider sans ces outils execute le meme workflow mais avec
moins de profondeur (pas de prior art OSS live, pas de 1M tokens).

**(b) Hooks dynamises** : remplacer les hardcodes "sprint 67" dans
process-task-gate.sh et process-supervisor-stop.sh par detection
dynamique. Fermer le bypass chore(sprintN) Phase dans
precommit-lightcheck / Codex / body checks. `auditor-gate` parse deja
les titres `chore(sprintN): Sprint N Phase X`, mais garde un test de
regression dedie.
Durcir aussi le contrat portable :
- `FINAL_PASS_RE` doit refuser `## Verdict : PASS` et accepter
  uniquement `## Verdict: PASS` ;
- `precommit-lightcheck --scope message` doit bloquer un phase commit
  sans artefact G8 preflight ou pivot documente, pour tous les types
  valides ;
- `phase_commit_requires_codex()` doit couvrir tout commit
  `Sprint N Phase X`, y compris `chore(sprintN)` ;
- `docs/agent/TOOLING.md` doit documenter que `agentctl` bloque aussi
  les artefacts Codex manquants/unstaged/rewrites ;
- `scripts/install-claude-tooling.sh` et `docs/claude/TOOLING.md`
  doivent soit restaurer `.claude/hooks/post-commit-memory.sh`, soit
  supprimer le wrapper post-commit casse.

Restriction jusqu'a ce fix : les phases A-E ne doivent pas utiliser
`chore(sprintN): Sprint N Phase X` comme titre de commit de phase.

**(c) Provider config + dogfood** : creer
`docs/agent/PROVIDER_CONFIG.md` qui definit comment configurer le
driver LLM (qui code) et le verificateur LLM (qui review/audit).
Table des combinaisons supportees :
- Driver Claude + Verificateur Codex (actuel)
- Driver Claude + Verificateur Claude (fallback)
- Driver Codex/GPT/local + Verificateur Claude
- Driver LLM local + Verificateur LLM local (full offline)

`agentctl prompt --provider {claude,codex,gpt,local,human}` existe
depuis Phase D ; Phase F ajoute la politique d'adaptation provider
(ex: si provider=local, pas de reference WebSearch/context7 dans les
instructions).

Dogfood : generer un prompt preflight pour un provider non-Claude,
verifier que le format est executable, prouver que
status-sprint/lint-planning/audit-commit/serve fonctionnent, et
piloter au moins un changement repo-visible depuis le dashboard.

### §9.2 Livrables

| Fichier | Description |
|---|---|
| `.claude/agents/nexus-phase-preflight-deep.md` | REFACTOR. Garder les instructions Claude-specifiques (WebSearch, context7, Read 1M). Deleguer la logique des 5 scans au prompt portable `preflight.md` via reference. |
| `.claude/agents/nexus-phase-review-deep.md` | REFACTOR. Garder les instructions Claude-specifiques. Deleguer les 11 dimensions au prompt portable `phase-review.md`. |
| `.claude/agents/nexus-audit-gate.md` | REFACTOR. Garder les instructions Claude-specifiques. Deleguer les 9 tracks au prompt portable `audit-gate-checks.md`. |
| `.claude/agents/nexus-phase-auditor.md` | REFACTOR. Wrapper leger sur `phase-auditor.md` portable. Clarifier routing review-deep vs auditor. |
| `.claude/hooks/process-task-gate.sh` | UPDATE. Detection dynamique sprint courant. |
| `.claude/hooks/process-supervisor-stop.sh` | UPDATE. Detection dynamique sprint + phase. |
| `scripts/agent/agentctl.py` | UPDATE. Fix bypass chore(sprintN) Phase + `--provider` flag pour prompt assembly + verdict exact + G8 preflight gate portable. |
| `docs/agent/PROVIDER_CONFIG.md` | NEW. Table driver/verificateur, combinaisons, instructions par provider. |
| `docs/agent/TOOLING.md` | UPDATE. Documenter enforcement reel : Codex artifacts, G8 preflight, verdict exact, phase commits tous types valides. |
| `scripts/install-claude-tooling.sh`, `docs/claude/TOOLING.md` | UPDATE. Corriger ou supprimer la reference au hook `.claude/hooks/post-commit-memory.sh` absent. |

### §9.3 Tests plan

1. `test_auditor_gate_blocks_chore_sprint_phase` — chore(sprint70) Phase bloque sans review
2. `test_auditor_gate_allows_chore_planning` — chore(planning) passe
3. `test_precommit_lightcheck_blocks_chore_sprint_phase_without_codex` — un commit `chore(sprintN): Sprint N Phase X` doit porter le body 9 sections + Codex si c'est un vrai commit de phase
4. `test_review_gate_rejects_spaced_verdict` — `## Verdict : PASS` bloque
5. `test_precommit_lightcheck_blocks_phase_without_g8_preflight` — phase commit sans preflight/pivot bloque
6. `test_prompt_provider_flag_local` — --provider local exclut WebSearch/context7
7. `test_prompt_provider_flag_claude` — --provider claude inclut tout

Dogfood via dashboard :
8. Ouvrir dashboard operateur → Sprint Overview affiche S70 en cours
9. Assistant de phase → "Preparer la phase" + "Agent local/offline" → prompt executable
10. `agentctl status-sprint` + `lint-planning` + `audit-commit --rev HEAD` + endpoint `/api/status`

### §9.4 Critere d'acceptation

```bash
! rg "sprint.?67" .claude/hooks/process-task-gate.sh .claude/hooks/process-supervisor-stop.sh && \
! rg "^## Verdict : PASS$" docs/agent docs/claude .planning/active && \
uv run pytest tests/test_agentctl.py -q -k "auditor or provider" && \
python scripts/agent/agentctl.py prompt --kind preflight --provider local --depth deep > /dev/null && \
python scripts/agent/agentctl.py prompt --kind handoff --depth deep > /dev/null && \
python scripts/agent/agentctl.py serve --port 3001 --once-smoke && \
echo "PASS" || echo "FAIL"
```

### §9.5 Commit cible

`feat(agent): Sprint 70 Phase F — agent refactor wrappers + hooks dynamises + provider config + dogfood`

Body : 9 sections obligatoires. Delta tests : +7 Python.

---

## §10 Phase G — Contrat RRV/Factory + verification + wrap-up

### §10.1 Scope

Creer `docs/agent/RRV_FACTORY_CONTRACT.md` avec la table de mapping
modes→roles, le principe d'autorite, le contrat Factory, le
sequencing post-S70. Ecrire `sprint70_verification.md` fail-fast.
Ecrire `sprint71_audit_plan.md`. Mettre a jour CLAUDE.md (etat,
compteurs, carries), SPRINT_LOG.md, memory nexus_grid_pivot.md.
SPRINT_LOG.md doit annoncer S70 comme sprint 7 phases A-G, pas
reprendre l'ancien wording "6 phases A-F".

### §10.2 Livrables

| Fichier | Description |
|---|---|
| `docs/agent/RRV_FACTORY_CONTRACT.md` | NEW. Table mapping 5 modes @ → roles portables. Principe autorite (execution dans .planning/active/). Factory = consommateur. Babel = app. Sequencing post-S70. |
| `.planning/active/sprint70_verification.md` | NEW. Fail-fast checklist. Delta tests. Scope cuts compliance. G8 bilan. Carries. Commits. Checkpoint cloture. |
| `.planning/active/sprint71_audit_plan.md` | NEW. Tracks audit S70. |
| `CLAUDE.md` | UPDATE. Tip, compteurs, carries, etat process. |
| `docs/claude/SPRINT_LOG.md` | UPDATE. Row S70 avec 7 phases A-G et rappel dashboard operateur action-gated. |

### §10.3 Tests plan

Phase docs-only : Codex review, final `## Verdict: PASS` et body
9 sections restent obligatoires. Les suites lourdes peuvent etre
exemptees seulement avec justification ecrite.
Verification :
1. `test -f docs/agent/RRV_FACTORY_CONTRACT.md` — fichier existe
2. `rg -n "@research|@dev|@audit|@security|@product" docs/agent/RRV_FACTORY_CONTRACT.md` — 5 modes documentes
3. `test -f .planning/active/sprint70_verification.md` — verification ecrite
4. `test -f .planning/active/sprint71_audit_plan.md` — audit plan ecrit
5. `rg "7 phases|A-G" docs/claude/SPRINT_LOG.md CLAUDE.md` — etat sprint aligne

### §10.4 Critere d'acceptation

```bash
test -f docs/agent/RRV_FACTORY_CONTRACT.md && \
test -f .planning/active/sprint70_verification.md && \
test -f .planning/active/sprint71_audit_plan.md && \
rg -q "7 phases|A-G" docs/claude/SPRINT_LOG.md && echo "PASS" || echo "FAIL"
```

### §10.5 Commit cible

`docs(sprint70): Sprint 70 Phase G — RRV/Factory contrat + verification + wrap-up`

Body : 9 sections obligatoires. Checkpoint cloture complet.

---

## §11 Delta tests estime

| Phase | Rust | Vitest | Python | Factory | Detail |
|---|---|---|---|---|---|
| A | +0 | +0 | +0 | — | docs-only (AGENT_SYSTEM.md 7 sections + AGENTS.md) |
| B | +0 | +0 | +0 | — | docs-only (PATTERNS.md + README.md) |
| C | +0 | +0 | +8 | — | 6 prompt kinds + context AGENT_SYSTEM + bootstrap matrix |
| D | +0 | +0 | +30 | — | status-sprint, lint-planning, audit-commit, serve + endpoints JSON + context-pack + Agent Chat + once-smoke + provider arg + action guards |
| E | +0 | +0 | +0 | build+lint+tsc+action smoke | Dashboard operateur (React standalone) |
| F | +0 | +0 | +7 | — | chore phase Codex/body gate + verdict exact + G8 gate + provider policy + dogfood |
| G | +0 | +0 | +0 | — | docs-only (RRV contract + verification) |
| **Total** | **+0** | **+0** | **+45** | **build+lint+smoke** | |
| **Sortie estimee** | **1433** | **279** | **~59** | **compile** | **~1718 + 45 Python + Factory checks** |

Note : les tests Python ne sont pas comptes dans le total historique
Rust+Vitest. Le total test_agentctl.py passe de 14 a ~59. Le Factory
Dashboard a son propre build/lint/tsc + smoke dashboard/action mais pas
de tests unitaires S70 (candidat S71).

---

## §12 Fail-fast checklist

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
| 17 | AGENTS.md no stale refs | `! rg "packages/nexus-sdk\|scripts/setup\.sh\|uv run pytest packages/" AGENTS.md` | absent |
| 18 | handoff.md exists | `test -f prompts/agent/handoff.md` | exists |
| 19 | agentctl portable prompt kinds | `for k in handoff preflight phase-review commit-body audit-gate phase-auditor; do python scripts/agent/agentctl.py prompt --kind $k --depth deep > /dev/null; done` | exit 0 |
| 20 | agentctl status-sprint | `python scripts/agent/agentctl.py status-sprint` | exit 0 |
| 21 | agentctl lint-planning | `python scripts/agent/agentctl.py lint-planning` | exit 0 |
| 22 | agentctl audit-commit | `python scripts/agent/agentctl.py audit-commit --rev HEAD` | exit 0 |
| 23 | Python tests agentctl | `uv run pytest tests/test_agentctl.py -q` | >= 59 pass |
| 24 | hooks no stale S67 | `! rg "sprint.?67" .claude/hooks/process-task-gate.sh .claude/hooks/process-supervisor-stop.sh` | absent |
| 25 | RRV_FACTORY_CONTRACT | `test -f docs/agent/RRV_FACTORY_CONTRACT.md` | exists |
| 26 | RRV 5 modes documented | `rg -c "@research\|@dev\|@audit\|@security\|@product" docs/agent/RRV_FACTORY_CONTRACT.md` | >= 5 |
| 27 | verification.md | `test -f .planning/active/sprint70_verification.md` | exists |
| 28 | audit_plan S71 | `test -f .planning/active/sprint71_audit_plan.md` | exists |
| 29 | PROVIDER_CONFIG.md | `test -f docs/agent/PROVIDER_CONFIG.md` | exists |
| 30 | provider flag works | `python scripts/agent/agentctl.py prompt --kind preflight --provider local --depth deep > /dev/null` | exit 0 |
| 31 | agentctl serve smoke | `python scripts/agent/agentctl.py serve --port 3001 --once-smoke` | exit 0 |
| 32 | dashboard action guard smoke | `python scripts/agent/agentctl.py serve --port 3001 --once-smoke --include-actions` | exit 0 |
| 33 | factory-dashboard lint | `(cd tools/factory-dashboard && npm run lint)` | 0 errors |
| 34 | factory-dashboard tsc | `(cd tools/factory-dashboard && npx tsc --noEmit)` | 0 errors |
| 35 | factory-dashboard build | `(cd tools/factory-dashboard && npm run build)` | ok |
| 36 | verdict exact canonique | `! rg "^## Verdict : PASS$" docs/agent docs/claude .planning/active` | absent |
| 37 | context-pack no chat authority | `uv run pytest tests/test_agentctl.py -q -k "context_pack"` | pass |
| 38 | G8 portable gate | `uv run pytest tests/test_agentctl.py -q -k "preflight"` | pass |
| 39 | SPRINT_LOG S70 A-G | `rg -q "7 phases|A-G" docs/claude/SPRINT_LOG.md` | present |
| 40 | Agent Chat from context-pack | `uv run pytest tests/test_agentctl.py -q -k "chat"` | pass |

---

## §13 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | SearchManifest wire format + gossip | S71 | Protocole reseau hors scope process. |
| 2 | Page React `/factory` dans le shell produit | S71+ | Le dashboard S70 est un cockpit operateur process standalone, pas la page produit `/factory`. |
| 3 | @dev index tree-sitter | S71+ | @dev non bloquant Gate 1. |
| 4 | Template react-vite | S71+ | Pas de demande pilote. |
| 5 | CuratorVouched UI shell | S71+ | Vouch dans feed, UI post-pilote. |
| 6 | FG10 Review gate automatise | S71+ | Depend outillage post-Gate 1. |
| 7 | Fuzzing cargo-fuzz/proptest | post-Gate 1 | Hors scope fonctionnel. |
| 8 | Feed format version bump | post-launch | Pre-launch policy. |
| 9 | ProofCard comme feed op | S71+ | Candidat SearchManifest. |
| 10 | iroh 1.0 upgrade | Gate 1 decision | Evalue post-pilote. |
| 11 | CI process workflow lourd multi-provider | S71 | S70 livre le smoke `agentctl serve --once-smoke`; CI complete multi-provider post-S70. |
| 12 | Provider router multi-LLM | post-S75 | Manual copier-coller suffit. |
| 13 | sbfb-search app | S71+ | Depend SearchManifest. |
| 14 | Ingestion OSS broad | post-S75 | Mode source-only separe. |

---

## §14 Risks

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | agentctl status-sprint parse incorrectement artefacts S65-S69 | Medium | Medium | Tests sur patterns reels. |
| R2 | Hooks dynamises cassent sur sprint impair | Low | High | Tests manuels sur commits S69. |
| R3 | AGENT_SYSTEM.md duplique PROCESS.md | Medium | Low | Review G8 verifie non-duplication. |
| R4 | Handoff trop long pour LLM contexte court | Low | Medium | Mode standard (court) vs deep. |
| R5 | P2-I-3 3/3 non resolvable | Medium | Low | Convention simple (3-5 lignes). |
| R6 | Dogfood dashboard ne prouve rien si trivial | Medium | Medium | Changement repo-visible obligatoire pilote depuis le dashboard + smoke serve documente dans verification.md. |
| R7 | P2-G-1 CLOSE premature | Low | High | Conditions reouverture documentees. |
| R8 | Dashboard confondu avec autorite Factory | Medium | High | Agent Chat + actions journalisees + confirmations. Shell/commit/push/PASS final ne sont pas des boutons d'autorite UI ; ils passent par une vraie session agent, gates et preuves repo-visibles. |

---

## §15 Checkpoint de cloture

- [ ] 40/40 fail-fast verts
- [ ] 7 commits : 2 docs (A + B) + 2 feat (C + D) + 1 feat (E dashboard) + 1 feat (F refactor) + 1 docs (G)
- [ ] verification.md + audit_plan S71 ecrits
- [ ] AGENT_SYSTEM.md cree (7 sections, Gate Contract + Prompt Registry)
- [ ] 6 prompt kinds portables executables dans prompts/agent/
- [ ] agentctl prompt --kind X --provider Y fonctionne pour tout kind/provider
- [ ] agentctl serve expose JSON API pour le dashboard
- [ ] context-pack genere un nouveau contexte sans chat history authoritative
- [ ] Dashboard operateur compile (tools/factory-dashboard/)
- [ ] Dashboard connecte a agentctl serve (sprint status, prompt generator, lint, audit, context-pack, Agent Chat, draft artifact, action log)
- [ ] .claude/agents/ refactored en wrappers legers sur prompts portables
- [ ] PROVIDER_CONFIG.md definit combinaisons driver/verificateur
- [ ] 4 commandes agentctl observabilite operationnelles + serve smoke
- [ ] Verdict exact, G8 portable gate et chore(sprintN) phase gate durcis
- [ ] Hooks stale S67 dynamises
- [ ] P2-I-3 3/3 CLOSED (body docs)
- [ ] P2-G-1 CLOSED (8 sprints non-repro)
- [ ] RRV_FACTORY_CONTRACT.md cree
- [ ] PATTERNS.md mis a jour (T-NN+3 + P2-G-1 CLOSE)
- [ ] Memory nexus_grid_pivot.md tip + compteurs a jour
- [ ] SPRINT_LOG.md row S70 ajoutee
