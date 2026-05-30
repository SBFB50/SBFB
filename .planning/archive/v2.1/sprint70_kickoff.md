# Sprint 70 — Kickoff (Process Portable Complete + Gate 1 dogfood)

**Ecrit** : 2026-05-24 (post-audit gate S69 PASS `c6c135f`).
**Type** : **sprint pair** — une phase dette reservee (Phase B,
Regle 1 §6.2.1). Un item 3/3 (Regle 2) a traiter : P2-I-3 body
docs minimaliste.
**Tip master d'entree** : `c6c135f` (audit findings S69 PASS
0 P0, 0 P1, 3 P2, 2 P3).
**Phase 0 audit Sprint 69** : **DEJA JOUE** — `c6c135f` PASS.
Aucun fix requis.
**Version archive** : v2.1 — Protocole Neutre + Factory/RRV.
**Roadmap source** : `.planning/roadmap_v4_neutral_protocol_factory_rrv.md`.
Sprint 1 sur 1 (Arc 2.5 Process Portable Complete). Premier
sprint post-Arc 2 COMPLET.

---

## Sources context7 + WebSearch consultees (pre-gel)

| # | Source | Type | Date | Finding cle |
|---|--------|------|------|-------------|
| 1 | WebSearch "AGENT_SYSTEM.md portable agent process documentation best practices LLM multi-model 2025 2026" | WebSearch | 2026-05-24 | AGENTS.md est devenu le standard multi-outil (Cursor, Claude Code, GitHub Copilot) pour les instructions agent machine-readable. Le format Markdown naturel s'integre dans tout projet. Separation README (humain) vs AGENTS.md (machine). |
| 2 | WebSearch "agent handoff protocol LLM context transfer portable session state 2025 2026" | WebSearch | 2026-05-24 | Portable Agent Memory protocol (arxiv 2605.11032, mai 2026) : disclosure selective, re-hydration 7 etapes, memoire procedurale vs memoire de travail. OpenAI Agents SDK handoffs. LangChain subgraph handoffs. Git Context Controller (arxiv 2508.00031) : context comme workspace persistant navigable. |
| 3 | WebSearch "agentctl CLI tool developer workflow observability sprint status 2025 2026" | WebSearch | 2026-05-24 | Claude Code, Gemini CLI, Codex CLI : les agents headless avec contexte structure surpassent les IDE copilots (2026). AgentOps : session replay + cost tracking. "Pick the simplest workflow, invest in tool design, grounding, state explicite, observabilite." |
| 4 | WebSearch "git hooks pre-commit agent process enforcement bypass prevention CI 2025 2026" | WebSearch | 2026-05-24 | Pre-commit hooks bypassables via --no-verify. Defense en couches : client (pre-commit rapide), serveur (pre-receive), CI (heavy). Hooks > 5s seront bypasses. Agent code = hooks non optionnels, c'est la gate entre output agent et repo. |
| 5 | WebSearch "RRV role-based verification system alias mapping software development process 2025 2026" | WebSearch | 2026-05-24 | Role verification = verifiable identification + cross-reference + verification labels. Pattern RBAC classique. Pas de standard RRV specifique — notre mapping @mode → role est un design maison coherent avec RBAC. |
| 6 | Code local `scripts/agent/agentctl.py` (754 lignes) | Code local | 2026-05-24 | 6 commandes : context, prompt, codex-prompt-path, verify-on-write, precommit-lightcheck, auditor-gate. Manquent : status-sprint, lint-planning, audit-commit. PROMPT_KINDS ne contient pas "handoff". |
| 7 | Code local `docs/agent/PROCESS.md` (214 lignes) | Code local | 2026-05-24 | Process vendor-neutral complet. 10 roles workflow + guarantee matrix G1-G10. Manque : registre de roles formel, mapping provider, artifact contract par role. |
| 8 | Code local `docs/agent/TOOLING.md` (112 lignes) | Code local | 2026-05-24 | Reference tooling. Documente les 6 commandes agentctl existantes + hook installation. A jour. |
| 9 | Code local `tests/test_agentctl.py` (217 lignes) | Code local | 2026-05-24 | 11 tests existants. Couvrent : prompt kinds, phase title parsing, scope mismatch, commit title BOM, context sources, codex prompt path, review gate reject, codex artifact missing, staged diff, body sections, verify-on-write. Manquent : tests pour status-sprint, lint-planning, audit-commit. |
| 10 | Code local `.claude/hooks/process-task-gate.sh` (113 lignes) | Code local | 2026-05-24 | Hardcode "sprint 67" et "phase c" aux lignes 77-79, 90-94. Stale — S67 est 3 sprints en arriere. |
| 11 | Code local `.claude/hooks/process-supervisor-stop.sh` (97 lignes) | Code local | 2026-05-24 | Hardcode "sprint67_phase_c_preflight.md" ligne 80. Stale — meme probleme. |
| 12 | Code local `prompts/agent/universal.md` (environ 21K) | Code local | 2026-05-24 | Prompt complet vendor-neutral. Pas de kind "handoff" dans PROMPT_KINDS. |
| 13 | Code local `AGENTS.md` (38 lignes) | Code local | 2026-05-24 | Reference stale : mentionne `packages/` (Python supprime S50), `setup.sh` (supprime). A mettre a jour. |
| 14 | Code local `.planning/research/process_portable_complete_s70.md` (279 lignes) | Artefact local | 2026-05-22 | Intake principal S70. 6 phases detaillees. Fact pattern, acceptance criteria. |
| 15 | Code local `docs/agent/codex-process/` (6 fichiers) | Code local | 2026-05-24 | Runbook Codex stricter layered on PROCESS.md. Session start, phase driving, review/audit, commit gate, domain smoke, automation backlog. |
| 16 | Audit findings S69 P2-C-1, P2-C-2, P2-I-1 | Artefact local | 2026-05-23 | P2-C-1 duplication canonical bytes Factory/coordinator (1/3). P2-C-2 serde_json vs serde_jcs (1/3). P2-I-1 docs dans chore (1/3). 3 P2 a router tech debt. |

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 69 a ferme l'Arc 2 (Factory + RRV @protocole + Canari) avec
5 phases livrees : preview cap + audit log, FG8/FG9 pipeline
complet, template static-reader Babel, Gate 1 test protocol, et
verification/wrap-up. L'audit S69 est PASS (0 P0, 0 P1, 3 P2,
2 P3). Gate 1 est pret a 8/9 criteres — G1-7 (stabilite 24h)
reste a valider par le pilote ferme.

L'Arc 1 (Fondations S65-S66) et l'Arc 2 (Factory S67-S69) sont
COMPLETS. Le projet entre dans l'Arc 2.5 : rendre le process agent
completement portable avant de construire RRV total, SearchManifest,
ou Factory process UI.

Le constat factuel (G9) sur l'infrastructure process existante :
- `agentctl.py` a 6 commandes, il en manque 3 (status-sprint,
  lint-planning, audit-commit)
- `PROCESS.md` est complet mais pas structure en registre de roles
  + artifact contract
- Pas de `AGENT_SYSTEM.md` (carte du systeme)
- Pas de `handoff.md` (prompt de transfert inter-provider)
- 2 hooks Claude hardcodent "sprint 67" (stale 3 sprints)
- `AGENTS.md` racine reference du Python supprime S50
- `tests/test_agentctl.py` a 11 tests, aucun pour les commandes
  manquantes
- Le superviseur process est optionnel (D17), hooks = backstop

### §1.2 Ancrage roadmap v4

Arc 2.5 (Process Portable Complete), sprint unique. Dependance
amont : S69 Arc 2 COMPLET + Gate 1 pret. Dependance aval : S71
consomme le process portable pour SearchManifest/RRV Core. H7
explicite : "RRV lit le process ; Factory le package plus tard.
Aucun des deux ne devient autorite process."

### §1.3 Compteurs tests entree (tip `c6c135f`)

| Suite | Count |
|---|---|
| Rust nextest | 1433 |
| Vitest | 279 |
| size-limit | 6/6 |
| **Total** | **~1718** |

### §1.4 Pre-launch protocol policy (rappel)

- `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` restent a 1 jusqu'au
  go-live public.
- Feed extensible via raw-op : pas de nouvelle feed op S70. Pas de
  bump `FEED_FORMAT_VERSION`.
- S70 est un sprint process/docs/tooling. Aucune modification de
  wire format protocolaire.
- `#[serde(default)]` reste legitime pour robustesse runtime.
- Factory gates (FG0-FG10) ne sont pas touches S70.

---

## §2 Goal

Rendre le process agent nexus-grid **completement portable** : tout
agent (Claude, Codex, GPT, LLM local, humain) peut reprendre le
travail a partir des fichiers du repo seuls, sans memoire de chat
privee. Les roles abstraits (driver, researcher, reviewer, auditor,
product, security, release, memory) sont formalises dans une carte
systeme. Le handoff est un prompt generable. L'observabilite process
passe par 3 nouvelles commandes agentctl. Les bypasses connus dans
les hooks sont fermes. Le tout est prouve par un dogfood reel et
contractualise pour RRV/Factory.

**Critere SMART : toutes les rows fail-fast vertes au
verification.md, mesure binaire au Phase F wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 69

Audit S69 execute en session fraiche `c6c135f`.
Verdict : **PASS** (0 P0, 0 P1, 3 P2, 2 P3).
Aucun fix bloquant. Sprint 70 autorise.

P2 documentes :
- P2-C-1 duplication canonical bytes Factory/coordinator (1/3)
- P2-C-2 serde_json vs serde_jcs provenance (1/3)
- P2-I-1 docs technique dans chore (1/3)

P3 documentes :
- P3-E-1 test skip_gates manquant pipeline.rs (nit)
- P3-I-1 recap cumule delta incorrect Phase D body (nit)

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — AGENT_SYSTEM.md comme carte derivee du process, pas nouvelle procedure

**Sources consultees** :
- WebSearch "AGENT_SYSTEM.md portable agent process documentation LLM multi-model" (2026-05-24) : AGENTS.md est le standard de facto (Cursor, Claude Code, Copilot). Format Markdown naturel. Separation README (humain) vs AGENTS.md (machine). Le document doit contenir les roles, workflows et regles pour que tout agent les consomme.
- WebSearch "agent handoff protocol LLM context transfer" (2026-05-24) : Portable Agent Memory protocol (arxiv 2605.11032) distingue memoire procedurale (roles, regles) de memoire de travail (etat courant). AGENT_SYSTEM.md = memoire procedurale portee par le repo.
- Code local `docs/agent/PROCESS.md` (2026-05-24) : process vendor-neutral complet avec guarantee matrix G1-G10, 10 etapes workflow, prompt contract. Manque : registre roles formel, mapping provider, artifact contract par role.
- Code local `AGENTS.md` (2026-05-24) : 38 lignes, reference stale Python. Orientation minimale. AGENT_SYSTEM.md serait la version complete.
- Code local `.planning/research/process_portable_complete_s70.md` §5 Phase A (2026-05-22) : specefie les sections requises : role registry, provider mapping, lifecycle modes, artifact contract, non-goals.

**Retenu** : `docs/agent/AGENT_SYSTEM.md` est cree comme une **carte
derivee** de `PROCESS.md` — elle ne remplace pas PROCESS.md, elle le
complete. Structure en 5 sections :

1. **Truth Stack** — hierarchie des sources de verite : repo files >
   `.planning/active/` > commit history > prompt > chat memory. La
   chat memory est explicitement NON-authoritative.
2. **Role Registry** — 8 roles abstraits : `driver`, `researcher`,
   `reviewer`, `auditor`, `product`, `security`, `release`, `memory`.
   Chaque role avec ses droits (quels fichiers il ecrit), ses
   obligations (quels artefacts il doit produire), et ses limites
   (ce qu'il ne fait PAS).
3. **Provider Mapping** — table Claude / Codex / GPT / LLM local /
   humain avec leur mode d'invocation et leur couverture de roles.
   Provider-agnostique : meme contrat, memes artefacts.
4. **Lifecycle Modes** — les 10 modes de la boucle sprint : sprint
   start, preflight, implementation, review, codex audit, research,
   release audit, product decision, security review, memory carry.
   Chaque mode pointe vers le prompt correspondant dans
   `prompts/agent/`.
5. **Non-Goals** — pas de memoire privee comme autorite, pas de
   RRV parallele, pas de dependance a un provider specifique.

AGENT_SYSTEM.md NE duplique PAS PROCESS.md. PROCESS.md reste le
"quoi faire" (workflow sequentiel). AGENT_SYSTEM.md est le "qui fait
quoi avec quels droits" (carte des roles et providers).

`AGENTS.md` racine sera mis a jour pour pointer vers AGENT_SYSTEM.md
et corriger les references stale (Python supprime S50).

**Rejete** :
- **Fusionner dans PROCESS.md** : PROCESS.md est deja 214 lignes.
  L'ajouter alourdirait un document qui sert de reference rapide
  workflow. La carte roles/providers est un complement, pas une
  extension du workflow. (Source : separation of concerns, cf.
  pattern README vs AGENTS.md observe dans l'ecosysteme)
- **Creer dans prompts/agent/** : les prompts sont des instructions
  injectees dans un contexte LLM. AGENT_SYSTEM.md est un artefact
  repo permanent lu par humains ET machines. Pas le meme cycle de
  vie. (Source : docs/agent/ contient deja PROCESS.md et TOOLING.md)
- **Ne pas le creer (status quo)** : le gap est reel — aucun
  fichier repo ne formalise les roles, les droits d'ecriture par
  role, ou le mapping provider. L'intake S70 §4 le documente
  explicitement. (Source : process_portable_complete_s70.md §4
  "What is still missing")

**Implications code** : `docs/agent/AGENT_SYSTEM.md` (NEW),
`AGENTS.md` (update references stale).

### D2 — Handoff prompt portable avec generation agentctl

**Sources consultees** :
- WebSearch "agent handoff protocol LLM context transfer portable" (2026-05-24) : OpenAI Agents SDK handoffs : transfert de conversation d'un agent a un autre avec contexte structure. LangChain subgraph handoffs pour valid conversation history. Portable Agent Memory (2605.11032) : selective disclosure, 7-stage re-hydration pipeline.
- WebSearch "Git Context Controller manage context LLM agents" (2026-05-24) : GCC (arxiv 2508.00031) eleve le contexte agent au rang de workspace persistant navigable avec operations explicites de checkpoint et transfer.
- Code local `scripts/agent/agentctl.py` cmd_prompt() (2026-05-24) : assemble les prompts existants (base, universal, preflight, phase-review, phase-auditor, commit-body). Pas de kind "handoff". Le contexte genere inclut repo, sprint, phase, git status, diff stat. Le mode "deep" ajoute branch, HEAD, staged/unstaged names, recent commits.
- Code local `prompts/agent/universal.md` (2026-05-24) : prompt complet ~21K pour handoff full sprint. Manque : etat explicite du sprint courant (phase en cours, fichiers changes, tests evidence, verdicts, stop conditions, assumptions a ne PAS heriter).
- Code local `process_portable_complete_s70.md` §5 Phase B (2026-05-22) : specifies handoff content — active sprint, phase, changed files, test evidence, verdict state, stop conditions, assumptions not to inherit.

**Retenu** : `prompts/agent/handoff.md` est cree comme un template
Markdown contenant les sections structurees pour un transfert
inter-provider :

1. **Sprint context** : sprint N, phase X, tip SHA
2. **Progress state** : phases completees, phase en cours, % avancement
3. **Changed files** : liste des fichiers touches (staged + unstaged)
4. **Test evidence** : derniers compteurs Rust/Vitest, tests specifiques
5. **Verdict state** : preflight/review/codex verdicts existants
6. **Stop conditions** : quand s'arreter (ex: "ne PAS commit avant
   Codex review")
7. **Assumptions NOT to inherit** : decisions chat-only qui ne sont
   PAS dans le repo (explicitement listees pour que le nouveau
   provider ne les presuppose pas)
8. **Active carries** : items P2+ ouverts
9. **Next actions** : 1-3 actions concretes pour le provider suivant

`agentctl prompt --kind handoff` est ajoute a PROMPT_KINDS. Le
handoff combine le template `handoff.md` avec le contexte runtime
genere par `prompt_context()` (deep mode). Le resultat est un
document autosuffisant qu'on peut copier-coller dans n'importe quel
LLM.

`AGENT_SYSTEM.md` est reference dans `agentctl context` pour que
toute invocation de context inclue la carte systeme.

**Rejete** :
- **Handoff automatise (API inter-provider)** : hors scope. Le
  projet n'a pas d'infra multi-provider orchestree. Le handoff est
  un document copie-colle. (Source : vision_model.md pattern solo
  maintainer)
- **Etendre universal.md au lieu de creer handoff.md** : universal.md
  est un prompt generaliste pour une session complete. Le handoff
  est specifique a un moment (mid-phase, fin de phase, changement
  de provider). Les deux ont des cycles de vie differents. (Source :
  PROCESS.md §Prompt Contract distingue deja universal/base/preflight)
- **Ne pas wirer dans agentctl** : sans `--kind handoff`, le template
  reste un document mort. L'integration agentctl permet la generation
  avec contexte runtime injecte. (Source : TOOLING.md "prompt assembly")

**Implications code** : `prompts/agent/handoff.md` (NEW),
`scripts/agent/agentctl.py` (PROMPT_KINDS + cmd_prompt update),
`docs/agent/TOOLING.md` (documenter handoff), `docs/agent/AGENT_SYSTEM.md`
(reference dans agentctl context).

### D3 — Agentctl observabilite : status-sprint + lint-planning + audit-commit

**Sources consultees** :
- WebSearch "agentctl CLI developer workflow observability 2025 2026" (2026-05-24) : tendance 2026 — CLI agents headless avec contexte structure. AgentOps : session replay, cost tracking, integrations framework. Recommandation : "simplest workflow, invest in tool design, state explicite, observabilite."
- Code local `scripts/agent/agentctl.py` (2026-05-24) : 754 lignes, 6 commandes. `current_sprint()` existe deja (rglob sprint*_*.md, max). Les fonctions `read_text()`, `git()`, `run()`, `repo_root()` sont des utilitaires reutilisables.
- Code local `tests/test_agentctl.py` (2026-05-24) : 11 tests, pattern monkeypatch bien etabli. `load_agentctl()` importe le module dynamiquement.
- Code local `process_portable_complete_s70.md` §5 Phase C (2026-05-22) : specefie `status-sprint`, `lint-planning`, `audit-commit --rev HEAD` avec JSON output et tests dans `tests/test_agentctl.py`.

**Retenu** : 3 nouvelles commandes ajoutees a `agentctl.py` :

**(a) `agentctl status-sprint`** — affiche l'etat du sprint courant
en texte structure (ou JSON avec `--json`). Lit `.planning/active/`
pour detecter : kickoff present, plan present, phases avec preflight/
review/codex_review, verification present, audit_plan present.
Affiche le sprint number, la liste des phases avec leur statut
(planned/preflight/reviewed/committed), les carries ouverts, le tip
HEAD. N'execute PAS de tests — c'est de l'observabilite pure sur les
fichiers existants.

**(b) `agentctl lint-planning`** — verifie la coherence des artefacts
planning dans `.planning/active/`. Checks :
- kickoff et plan existent pour le sprint courant
- chaque phase declaree dans le plan a un preflight si elle a un
  review (coherence G8)
- pas de `PASS-PENDING` residuel dans les reviews
- pas de fichier sprint orphelin (sprint N-2 encore dans active/)
- scope cuts du kickoff sont coherents avec le plan
Retourne 0 si clean, 1 si warnings, 2 si blockers.

**(c) `agentctl audit-commit --rev HEAD`** — verifie un commit
specifique contre les regles process :
- le titre suit le pattern feat/fix/docs/chore(scope)
- si phase commit : review PASS existe, codex review existe
- les 9 sections body sont presentes (si phase commit)
- les fichiers references dans le body existent
Ne remplace PAS le hook pre-commit — c'est un outil de diagnostic
post-hoc ou de verification manuelle.

Toutes les commandes utilisent la stdlib Python uniquement (pas de
dep externe). Le JSON output permet la consommation future par
RRV/Factory.

**Rejete** :
- **Dashboard web** : overkill pour un CLI solo maintainer. Le texte
  en terminal suffit. JSON output couvre la consommation machine.
  (Source : vision_model.md)
- **Base de donnees pour l'etat process** : l'etat est deja dans les
  fichiers `.planning/active/`. Pas besoin de duplication. (Source :
  AGENT_SYSTEM.md truth stack — repo files > tout le reste)
- **Tests lourds dans audit-commit (re-run nextest)** : audit-commit
  est un diagnostic rapide sur un commit. Les tests lourds sont dans
  le fail-fast checklist. (Source : hooks < 5s rule, cf. WebSearch
  pre-commit)

**Implications code** : `scripts/agent/agentctl.py` (+3 commandes
~200-250 lignes), `tests/test_agentctl.py` (+8-12 tests),
`docs/agent/TOOLING.md` (documenter 3 commandes).

### D4 — Fermer les bypasses hooks + aligner le routing agents

**Sources consultees** :
- WebSearch "git hooks pre-commit bypass prevention enforcement CI 2025 2026" (2026-05-24) : hooks bypassables via --no-verify. Defense en couches : client (rapide), serveur (pre-receive), CI. Pattern 2026 : hooks = gate non-optionnelle pour agents. Si hooks > 5s, bypasses.
- Code local `.claude/hooks/process-task-gate.sh` lignes 77-94 (2026-05-24) : hardcode "sprint 67 phase c" et "sprint67_phase_c_preflight.md". Stale depuis 3 sprints. Le check ne bloque plus rien car les fichiers S67 sont en archive. Mais il empeche de bloquer le VRAI sprint courant.
- Code local `.claude/hooks/process-supervisor-stop.sh` ligne 80 (2026-05-24) : hardcode "sprint67_phase_c_preflight.md". Meme probleme.
- Code local `docs/agent/PROCESS.md` §Phase Commit Finality (2026-05-24) : `PASS-PENDING` non-commitable. `## Verdict: PASS` requis exactement. Phase identity doit matcher preflight/review/commit.
- Code local `.claude/agents/nexus-phase-auditor.md` vs `.claude/agents/nexus-phase-review-deep.md` (2026-05-24) : deux agents review distincts. `nexus-phase-auditor` est subsume par `nexus-phase-review-deep` (cf. CLAUDE.md tableau agents).

**Retenu** : 4 actions concretes :

**(a) Dynamiser les hooks Claude** : remplacer les hardcodes
"sprint 67" par une detection dynamique du sprint courant. Le hook
`process-task-gate.sh` utilise deja Python inline — la fonction
`current_sprint()` de agentctl est reutilisable. Le pattern : lire
`.planning/active/sprint*_*.md`, extraire le sprint max, construire
les noms d'artefacts dynamiquement.

**(b) Aligner auditor/review-deep** : CLAUDE.md dit que
`nexus-phase-auditor` est "subsume par review-deep". Clarifier dans
les agents que review-deep EST l'agent review principal et que
phase-auditor est un fallback plus leger. Pas de suppression — le
fallback reste utile pour les sessions legeres.

**(c) Fermer le bypass chore(sprintN) Phase** : le `auditor-gate`
dans agentctl.py ligne 663 skip les commits `chore(planning):`.
Mais un commit `chore(sprint70): Sprint 70 Phase A ...` passerait
le gate car le scope n'est pas `planning`. Le fix : le gate doit
verifier que tout commit contenant "Sprint N Phase X" dans le titre
(quelque soit le type) a une review PASS, pas seulement les
feat/fix/docs.

**(d) AGENTS.md cleanup** : mettre a jour les references stale
(supprimer Python, pointer vers AGENT_SYSTEM.md, corriger les
commandes).

**Rejete** :
- **Supprimer nexus-phase-auditor** : il sert de fallback leger
  quand review-deep est trop lourd. Le garder comme option. (Source :
  CLAUDE.md "subsume" != "supprime")
- **Ajouter un serveur pre-receive** : pas de serveur Git self-hosted.
  Le repo est sur GitHub. Les pre-receive hooks ne sont pas
  controlables sans GitHub Enterprise. (Source : GitHub limitation)
- **CI process workflow complet** : hors scope imminent. Le CI
  Woodpecker est operationnel pour les suites Rust/Frontend. Un CI
  process (agentctl lint) est candidat Phase D si le temps le permet,
  sinon scope cut S71.

**Implications code** : `.claude/hooks/process-task-gate.sh`
(dynamiser sprint), `.claude/hooks/process-supervisor-stop.sh`
(dynamiser sprint), `scripts/agent/agentctl.py` (fix bypass chore),
`AGENTS.md` (update), `.claude/agents/nexus-phase-auditor.md`
(clarification routing).

### D5 — Contrat RRV/Factory : modes @ comme alias de roles portables

**Sources consultees** :
- WebSearch "RRV role-based verification system alias mapping" (2026-05-24) : role verification = identification verifiable + cross-reference + labels. Pattern RBAC classique. Pas de standard "RRV" — design maison.
- Code local `.planning/research/rrv_sprint_intake_s70.md` (2026-05-24) : modes `@research`, `@dev`, `@audit`, `@security`, `@product` comme alias. RRV consomme status-sprint, lint-planning, audit-commit. Factory package les templates/contrats.
- Code local `.planning/roadmap_v4_neutral_protocol_factory_rrv.md` §Ordonnancement RRV (2026-05-19) : @protocole d'abord (S67 DONE), @dev S71+ (pas S70), @web post-pilote S73+.
- Code local `process_portable_complete_s70.md` §5 Phase F (2026-05-22) : RRV modes = aliases over roles, not authority. Factory = consumer/packager, not verification authority. Babel = app, not process.
- CLAUDE.md §Decisions architecturales gelees (2026-05-24) : "Ingestion OSS GitHub generique = futur mode source-only/source-index, distinct d'une app SBFB verifiee."

**Retenu** : Phase F produit un document de contrat
`docs/agent/RRV_FACTORY_CONTRACT.md` qui formalise :

1. **Table de mapping modes → roles** :
   - `@research` → `researcher` (fact-finding, no code)
   - `@dev` → `driver` (implementation, bounded edits)
   - `@audit` → `reviewer` + `auditor` (quality pass, security)
   - `@security` → `security` (threat model, loopback, sandbox)
   - `@product` → `product` (intake, decisions, scope)

2. **Principe d'autorite** : les modes `@` sont des alias de
   commodite. L'autorite d'execution reste dans `.planning/active/`
   et les gates. RRV affiche l'etat et les evidences, il ne prend
   pas de decisions d'execution.

3. **Factory comme consommateur** : Factory cree des templates,
   publie des apps, mais ne possede pas le verdict de qualite. Le
   daemon signe la provenance, le process valide la qualite.

4. **Babel est une app** : creee avec Factory, pas le process
   lui-meme. La distinction est importante pour eviter que le
   dogfood Babel soit confondu avec le process portable.

5. **Sequencing post-S70** : @dev LocalOnly, seed source-only,
   sbfb-search, provider router, SearchManifest → planifies
   seulement apres que le contrat process est repo-natif.

**Rejete** :
- **Implementer RRV total** : hors scope S70. Le contrat est un
  document, pas du code. RRV total = S71+ apres process portable.
  (Source : roadmap v4 D18 "pas de RRV total")
- **Integrer les modes @ dans agentctl** : premature. Les modes @
  sont des conventions documentaires avant d'etre du code. Le code
  viendra quand RRV a un corpus a indexer. (Source : rrv_sprint_intake
  §2 "research files are inputs, not executable plans")
- **Deplacer l'autorite dans Factory** : explicitement interdit par
  H7. Factory package, elle ne verifie pas. (Source : roadmap v4
  dependance H7)

**Implications code** : `docs/agent/RRV_FACTORY_CONTRACT.md` (NEW).

---

**Acknowledged review findings (G1)** :

Scoring : D1 ok, D2 ok, D3 ok, D4 warning, D5 ok.
Rigor signal G4 satisfait (1 warning sur 5).

D4 warning : les hooks Claude ne sont pas couverts par une source
< 90 jours specifique au pattern "dynamiser les sprint hardcodes".
C'est un pattern trivial (regex + glob) qui ne necessite pas de
recherche externe. Le warning est acknowledge, pas adjust. Le code
local lu (process-task-gate.sh lignes 77-94) fournit l'evidence
suffisante du gap.

---

## §5 Plan Phase outline A..F

### Phase A — Canon portable (AGENT_SYSTEM.md + AGENTS.md update)

Scope : creer `docs/agent/AGENT_SYSTEM.md` avec 5 sections (truth
stack, role registry, provider mapping, lifecycle modes, non-goals).
Mettre a jour `AGENTS.md` racine (corriger references stale Python,
pointer vers AGENT_SYSTEM.md). Commit cible : docs-only.

### Phase B — Dette pair + P2-I-3 3/3 + P2 audit absorbables

Scope : sprint pair, phase dette obligatoire (Regle 1 §6.2.1).
Resoudre P2-I-3 body docs minimaliste 3/3 MANDATORY (body docs
complet pour les phases docs-only). Absorber P2 audit S69 : P2-C-1
duplication canonical bytes, P2-C-2 serde_json/JCS alignment,
P2-I-1 docs chore split. Route AGENTS.md stale fix ici aussi si pas
Phase A.

### Phase C — Handoff portable + agentctl prompt --kind handoff

Scope : creer `prompts/agent/handoff.md`. Ajouter "handoff" dans
PROMPT_KINDS de agentctl.py. Wirer dans `agentctl prompt --kind
handoff --depth deep`. Mettre a jour TOOLING.md.

### Phase D — Agentctl observabilite (status-sprint + lint-planning + audit-commit)

Scope : implanter les 3 nouvelles commandes dans agentctl.py. Ecrire
les tests dans `tests/test_agentctl.py`. Mettre a jour TOOLING.md.
JSON output optionnel. Pas de dep externe (stdlib Python).

### Phase E — Gates, hooks, CI + dogfood

Scope : dynamiser les hooks Claude (remplacer hardcodes S67).
Fermer le bypass chore(sprintN) Phase dans auditor-gate. Aligner
documentation auditor/review-deep. Prouver le process portable par
un dogfood reel : generer un handoff, verifier que agentctl
status-sprint et lint-planning retournent un etat coherent, verifier
que audit-commit fonctionne sur les commits S69.

### Phase F — Contrat RRV/Factory + verification + wrap-up

Scope : creer `docs/agent/RRV_FACTORY_CONTRACT.md`. Ecrire
`sprint70_verification.md` fail-fast. Ecrire `sprint71_audit_plan.md`.
Mettre a jour CLAUDE.md, SPRINT_LOG.md, memory.

---

## §6 Items carry/dette

### Items 3/3 (traitement Sprint 70)

| Item | Reports | Phase S70 | Exit condition |
|---|---|---|---|
| P2-I-3 body docs minimaliste | 3/3 | Phase B | Tout commit docs(sprint)/docs(release) significatif (>100 lignes) a un body 3-5 lignes minimum. Verification : body du commit Phase A ou B. |

### Carry absorbes S70

| Item | Reports | Phase S70 | Exit condition |
|---|---|---|---|
| P2-C-1 canonical bytes duplication | 1/3 | Phase B | Documenter dans PATTERNS.md comme dette connue ou extraire dans nexus-core-rs. |
| P2-C-2 serde_json vs JCS | 1/3 | Phase B | Documenter dans PATTERNS.md comme dette connue avec rationale pre-launch. |
| P2-I-1 docs dans chore | 1/3 | Phase B | Documenter dans PROCESS.md ou README.md la regle chore/feat split pour docs techniques. |
| P2-G-1 exe lock intermittent | candidat CLOSE | Phase B | Non-repro 8 sprints consecutifs. CLOSE avec justification dans PATTERNS.md. |

### Carries reconduits S71

| Item | Reports | Justification |
|---|---|---|
| P2-A-1 rand blocker upstream | exemption→exemption | upstream rand 0.9 non publie (derniere verif crates.io 2026-05-24). Dep transitive iroh 0.98. Aucune action possible. |
| P2-AUDIT-2 iroh transitives | exemption→exemption | herite du pin iroh 0.98. Pas d'action avant evaluation upgrade iroh 1.0 post-Gate 1. |
| T-NN+2 iframe Rust-wasm | bloque→bloque | toolchain gaps wasm32 inchanges. Trigger : ort stable wasm32 ou tract opset 19. Verifie : pas de changement ecosystem mai 2026. |
| LT-2 Radicle sortie | trigger PENDING | tag v1.0 pose localement, pas pousse origin. Condition "push tag + GitHub Release" non remplie. Decision utilisateur, pas agent. |
| LT-5 redundancy persistence | hors-sprint | reclassifie S26. Condition : premier deploiement multi-worker OU tag v1.0 go-live. |
| LT-7 worker quorum E2E | post-tag | Tier 1+2 DONE (S55), Tier 3 P2P validee (S60). Worker quorum E2E carry post-tag. |

### Attention 3/3 S71

| Item | Reports apres S70 | Raison |
|---|---|---|
| P2-C-1 canonical bytes | 2/3 | Si pas resolu Phase B, passera 3/3 au S71. |
| P2-C-2 serde_json/JCS | 2/3 | Si pas resolu Phase B, passera 3/3 au S71. |

---

## §7 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | SearchManifest wire format + gossip | S71 | Protocole reseau hors scope process. Prereq Arc 3. |
| 2 | Page React /factory | S71+ | CLI suffit. UI Factory est post-process portable. |
| 3 | @dev index tree-sitter | S71+ | Decision PO 2026-05-21 : @dev non bloquant Gate 1. |
| 4 | Template react-vite | S71+ | 3 templates suffisent. Pas de demande pilote. |
| 5 | CuratorVouched UI shell | S71+ | Vouch dans le feed (S67). UI de curation post-pilote. |
| 6 | FG10 Review gate automatise | S71+ | Depend outillage post-Gate 1. |
| 7 | Fuzzing cargo-fuzz/proptest | post-Gate 1 | Hors scope fonctionnel. |
| 8 | Feed format version bump | post-launch | Pre-launch policy. |
| 9 | ProofCard comme feed op | S71+ | Candidat SearchManifest. |
| 10 | iroh 1.0 upgrade | Gate 1 decision | Evalue post-pilote. |
| 11 | CI process workflow Woodpecker | S71 | CI Rust/Frontend operationnel. CI process = stretch si Phase E le permet, sinon S71. |
| 12 | Provider router multi-LLM | post-S75 | Pattern manual copier-coller suffit pour 1 maintainer. |
| 13 | sbfb-search app | S71+ | Depend SearchManifest. |
| 14 | Ingestion OSS broad comme apps | post-S75 | Mode source-only separe d'une app SBFB verifiee. |

---

## §8 Tracabilite scope

| Item S69 "What's NOT" | Sprint + Phase S70 |
|---|---|
| SearchManifest wire format + gossip | Reconduit S71 |
| Page React /factory | Reconduit S71+ |
| @dev index tree-sitter | Reconduit S71+ |
| Template react-vite | Reconduit S71+ |
| CuratorVouched UI shell | Reconduit S71+ |
| FG10 Review gate | Reconduit S71+ |
| Fuzzing cargo-fuzz/proptest | Reconduit post-Gate 1 |
| Feed format version bump | Reconduit post-launch |
| ProofCard comme feed op | Reconduit S71+ |
| Diff engine avance | Reconduit S71+ (pas de gap S70 — diff basique fonctionnel) |
| Multi-template switching UI | Reconduit S71+ |
| Factory update-check | Reconduit post-launch |
| Babel traduction live | Reconduit post-launch |
| iroh 1.0 upgrade | Reconduit Gate 1 decision |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | agentctl status-sprint parse incorrectement les artefacts S65-S69 | Medium | Medium | Tests sur les patterns reels des artefacts existants (pas de mock generique). |
| R2 | Hooks Claude dynamises cassent sur un sprint impair | Low | High | Tests manuels sur les commits S69 existants avant commit Phase E. |
| R3 | AGENT_SYSTEM.md duplique PROCESS.md | Medium | Low | Review G8 preflight Phase A verifie la non-duplication. Chaque section AGENT_SYSTEM doit pointer vers PROCESS.md pour le detail. |
| R4 | Handoff prompt trop long pour les LLM a contexte court | Low | Medium | Mode --depth standard (court) vs deep (complet). Le prompt court suffit pour les LLM locaux. |
| R5 | P2-I-3 3/3 non resolvable car les phases docs sont difficiles a enrichir | Medium | Low | La convention est simple (3-5 lignes body minimum). Le risk est process, pas technique. |
| R6 | Le dogfood Phase E ne prouve rien si le changement est trivial | Medium | Medium | Le changement doit etre un vrai artefact Nexus (ex: corriger un scope cut, mettre a jour une doc), pas un placeholder. |
| R7 | P2-G-1 CLOSE premature — le bug reapparait | Low | High | CLOSE documente dans PATTERNS.md avec conditions de reouverture. Monitoring passif continue. |

---

## §10 Audit gate pattern — rappel

Phase 0 S69 jouee (`c6c135f` PASS). Phase F du sprint devra produire :
- `sprint70_verification.md` (self-report fail-fast)
- `sprint71_audit_plan.md` (plan pour Phase 0 S71)
- Mise a jour `docs/rust/PATTERNS.md` si P2 tech debt routes
- Mise a jour `docs/agent/PROCESS.md` si process changes

---

## §11 Checkpoint de validation

1. **D1 (AGENT_SYSTEM.md)** — Le document carte 5 sections derive de
   PROCESS.md sans le dupliquer. Si certains roles (ex: `memory`) sont
   peu pertinents pour un projet sans equipe, faut-il les garder pour
   completude ou les retirer pour simplicite ?

2. **D2 (handoff.md)** — Le handoff est un template injecte via
   agentctl. La section "Assumptions NOT to inherit" est-elle necessaire
   (risque de confusion), ou le handoff devrait-il se limiter au contexte
   factuel repo-visible ?

3. **D3 (agentctl 3 commandes)** — `audit-commit --rev HEAD` verifie
   un commit post-hoc. Faut-il aussi verifier les N derniers commits
   (`--rev HEAD~5..HEAD`), ou le scope HEAD seul suffit pour S70 ?

4. **D4 (hooks + bypass)** — La dynamisation des hooks Claude est
   Claude-specifique. Faut-il investir davantage dans les hooks portable
   `.githooks/` (qui fonctionnent pour tous les providers) et limiter
   les hooks Claude au minimum ?

5. **D5 (RRV/Factory contrat)** — Le contrat est un document
   descriptif sans code. Est-ce suffisant pour S70, ou faut-il un
   minimum de code (ex: `agentctl mode @research` qui alias vers le
   prompt researcher) pour prouver le mapping ?
