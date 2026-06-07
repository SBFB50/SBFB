# Sprint 72 — Audit Plan (consomme par la session fraiche S73)

**Ecrit** : 2026-06-03 (Phase F Sprint 72).
**Sprint audite** : **Sprint 72** (ProviderRouter multi-LLM + Factory
hardening + UX intentions d'execution).
**Executeur** : session fraiche S73, Phase 0 (Cas A audit gate).
**Produit attendu** : `.planning/active/sprint72_audit_findings.md`
(verdict PASS / CONDITIONAL PASS / FAIL).
**Tip audite** : commit Phase F `docs(sprint72)` (HEAD au demarrage S73 ;
tip code phases = `95cae05`).

---

## §0 Mode d'emploi pour la session fraiche S73

**Ordre de lecture impose** (forme une opinion AVANT de lire les self-reports) :

1. Ce fichier (`sprint73_audit_plan.md`) — la feuille de route.
2. Le **diff complet** S72 : `git diff 0b4e7f3..95cae05` (entree → tip code).
3. `sprint72_kickoff.md` §4 (D1..D5 gelees) + §7 (16 scope cuts).
4. Le code livre, dans l'ordre des tracks ci-dessous.

**A NE PAS lire avant d'avoir forme une opinion** :
`sprint72_verification.md` (self-report — l'agent livreur a ecrit le code
ET la verification ; valeur de confirmation nulle pour un audit independant)
et les `sprint72_phase_*_review.md` (reviews du livreur). Les lire **apres**
pour comparer, pas pour se faire une opinion.

**Format du livrable** : `sprint72_audit_findings.md` (§7 ci-dessous).

**Contexte non-standard a connaitre** : S72 a connu **2 DESIGN-CONFLICT G8
consecutifs** (Phases C et D), tous deux resolus par **arbitrage PO sur une
Option A documentee** (bump schemars 0.8→1.2 ; route daemon `/result` +
colonne `result_text`). Ce ne sont PAS des derives agent — les deux portent
une evidence ground-truth (collision dep transitive ; gap wire
result-retrieval) et un pivot_proposal repo-visible. L'audit doit verifier
que ces decisions sont (a) techniquement saines, (b) coherentes avec les
Day-0 (D2 ollama-rs unique, PO-14 batch reseau), (c) sans bump wire illegitime.

---

## §1 Critere verdict audit S72

| Verdict | Condition |
|---------|-----------|
| **PASS** | 0 P0, 0 P1, >= 1 P2+ documente |
| **CONDITIONAL PASS** | 0 P0, 0 P1 mais conditions a surveiller S73 |
| **FAIL** | >= 1 P0 ou >= 1 P1 non resoluble dans l'audit |

Rigor signal G4 : PASS exige >= 1 P2+ documente. 0 P0/P1 **et** 0 P2+
= CONCERN (audit trop superficiel). S72 expose au minimum ~8 P2+ carries
(ci-dessous) — un audit qui n'en confirme aucun est suspect.

---

## §2 Tracks audit S72 (ce que Phase 0 S73 doit verifier)

### Track A — Suites verification

Relancer la fail-fast §3 du `sprint72_verification.md` (31 rows). Attendu :
**1544 Rust nextest / 0 skip canonique CI Linux**, 279 Vitest (`web/` non
touche), 6/6 size-limit, `factory-operator` tsc+eslint+build exit 0.

- **Compte canonique = CI Linux** (Docker `rust:1.94` pinne ou Woodpecker).
  Windows natif montre **1 flake env** (`operator_sprint_history_endpoint`,
  timeout sous charge parallele full-workspace) — **passe en re-run isole**.
  Ne PAS auditer le compte sur poste Windows seul (`feedback_wsl_before_push`).
- Verifier la coherence du delta : +16 (A 0 / B 2 / C 7 / D 7 / E 0) ;
  exit 1544 = +16 vs entree 1528. **Exact, 0 ecart** (contraste S71 ou +3
  residuel) — verifier que la decomposition tient au `nextest list`.

**Findings routes ici (dette test, Phase B + E)** :
- **P2-TEST-ZOMBIE** (Phase B) — `audit_commit_valid_phase_commit`
  (process_cli.rs ~:473) hardcode le SHA S70 `6fb95df` dont les artefacts
  sont archives. Pre-existant, echoue deja sur master pur. **De-hardcoder ou
  fixturer.** Signal : P2 (zombie qui protege un scenario inexistant — cf.
  CLAUDE.md « tests legacy decode = a supprimer »).
- **P2-OPERATOR-TIMEOUT** (Phase B / verification) — `operator_server`
  tests timeout sous charge/bind-mount (famille du flake row 3). Pre-existant,
  reproduit sur master pur, passe isole. **Root-cause OU exemption formelle
  CI-Linux-only ecrite.** Signal : P2.
- **P2-OPERATOR-NO-TESTS** (Phase E) — `tools/factory-operator` n'a aucun
  test runner (scripts build/tsc/vite/lint seulement). La logique critique
  (defense auto-reconnect EventSource, mapping StreamChunk, rendu gate) est
  couverte par revue manuelle, pas par test. **Decision PO : ajouter Vitest a
  l'Operator (infra) ?** Signal : P2 (dette structurelle de package, pas
  regression).

### Track B — Security review (NetworkProvider + nouvelle surface daemon)

Le coeur securite S72 = (1) le routage provider preserve le gate, (2) la
nouvelle route daemon `/result`, (3) le catalogue menace Operator.

- **Gate preserve quel que soit le provider** : verifier que
  `handle_chat_stream` (`operator_server.rs:~896`) applique
  `SENSITIVE_ACTIONS` **AVANT** `ExecutionTarget::from_provider(...).run()`,
  pour Claude **ET** Ollama **ET** Network. Test de reference
  `sensitive_action_gated_regardless_of_provider`. Verifier qu'aucun bras
  ne court-circuite le gate.
- **Route daemon `GET /api/v1/tasks/{id}/result`** (Phase D, Option A) :
  confirmer trust tier **T0 loopback read-only** (lecture du `result_text`,
  pas d'ecriture/spawn). Verifier l'auth (token X-SBFB-Token si requis comme
  les autres routes `/api/v1/tasks/*`). Verifier qu'un POST cross-origin est
  refuse. Confirmer que `result_text` (migration M16) ne fuit pas de donnees
  au-dela du resultat de tache.
- **NetworkProvider client** (`provider_router.rs`) : verifier que le submit
  (`POST /api/v1/tasks/submit`) reutilise le token daemon loopback (R3) et
  que le endpoint est overridable (`SBFB_DAEMON_ENDPOINT`, defaut loopback).
- **Catalogue menace Operator (P2-H-1 ferme Phase A)** : confirmer
  `THREAT_MODEL.md §14` (T-OPERATOR-CSRF + T-OPERATOR-SPAWN + anticipation
  NetworkProvider) et `LOOPBACK_ENDPOINTS_TRUST_TIERS.md §3.1` (Operator :3001
  trust tier + gates G7/G2).

**Findings routes ici (Phase A + D)** :
- **P2-TIER-MODEL** (Phase A) — l'Operator :3001 est declare en notation
  ad-hoc « T0 + gate SENSITIVE_ACTIONS » (§3.1) mais le vocabulaire formel
  des tiers (§2) et la matrice de couverture (§8) de `LOOPBACK_…TRUST_TIERS`
  n'ont pas d'entree Operator. **Definir un tier formel « T0+ActionGate » OU
  ajouter les rows Operator a la matrice §8.** Signal : P2 (integration tier
  incomplete, doc menace en retard sur le code).
- **P2-POLL-DIAGNOSTIC-LOSS** (Phase D) — la poll-loop du NetworkProvider
  boucle sur erreurs HTTP non-2xx jusqu'au timeout, emettant un « timed out »
  generique plutot que l'erreur reelle. Trade-off resilience vs diagnostic.
  **Memoriser la derniere erreur, la surfacer au timeout.** Signal : P2.

### Track C — Patterns review

Verifier la coherence post-S72 de :
- `docs/rust/PATTERNS.md §P55` (3 axes orthogonaux : `ExecutionTarget` chat
  routing / `Provider` prompt-adapt / `LlmBackend` worker runtime). Verifier
  que chaque axe pointe le bon module et que le contrat StreamChunk commun
  est decrit (chaque bras emet le MEME `StreamChunk`).
- §P53 (quorum deterministe + provider/backend axes) et §P54 (dispatch key
  + E2E + caveat Windows-pump) restent coherents apres le bump ollama-rs 0.3.4
  et la colonne `result_text`. Verifier la note « sha256 misnomer » (la
  colonne stocke `result_text` brut) toujours exacte.
- Absence de duplication canonical (sensibilite P2-C-1 / T-NN+3).

### Track D — Scope cuts compliance

16/16 scope cuts (kickoff §7 / plan §11) auto-reportes dans
`verification.md §6`. Verifier par grep exhaustif qu'aucune ligne S72 ne
touche : feed-distant/FTS5 (#2), SearchResult/barre/SearchManifest (#3-5),
search/open/fork + projet distinct + templates (#6-8), GPU/quorum cross-MACHINE
(#9-10), sharding (#11), token-par-token WAN (#12 — verifier
`dones.len()==1 && deltas==0`), logprobs (#13), kudos (#14), multi-cloud (#16).

- **Verifier specifiquement** : l'UX intentions (`/execute`) est IN-SCOPE
  (Phase E) ; le **packaging produit** (launcher, install) reste S74 (#1). La
  route `/result` + `result_text` NE SONT PAS des scope cuts — c'est l'Option
  A corrective du DESIGN-CONFLICT D, pas une feature de recherche/fork.

### Track E — Tests delta coherence

Verifier les deltas par phase : A +0, B +2, C +7, D +7, E +0. Total = +16.
- Verifier la decomposition annoncee (verification §4) vs `nextest list`.
- Phase C : 6 tests `provider_router` + ripple schema/executor (bump schemars).
  Verifier que les **4 tests quorum S71** (R7) sont **re-verts** (pas un ajout
  — critere de non-regression de la migration ollama-rs).
- Phase D : db.rs +2 + http.rs +1 + provider_router net +1 + operator_server +3.
  Verifier `network_provider_submit_poll_yields_single_done` asserte bien
  PO-14 (un Done, zero Delta).

### Track F — Review files quality + §4.4 presence exhaustive

- **5 preflight** (A-E) : A EXECUTE, B EXECUTE, C DESIGN-CONFLICT→EXECUTE
  (Option A schemars), D DESIGN-CONFLICT→EXECUTE (Option A /result), E
  PLAN-ADAPT. **5 reviews** (A-E) toutes promues `## Verdict: PASS` (format
  exact, pas `## Verdict : PASS`). **5 codex_review** bruts + 2 pivot_proposal
  (C, D).
- **§4.4 ratio** : `- [ ] Phase review files present: 5/5` (A-E ont chacune
  un review.md). Ratio < 5/5 = P2.
- Verifier que chaque PARTIEL/GAP Codex est reconcilie :
  - A : 1 PARTIAL (cross-ref §8 ambiguite) — clarification, pas de re-run.
  - C : 1 PARTIAL (`ollama_stream_maps_to_chunks` availability-gated) — ferme
    par addition d'un test mock deterministe.
  - E : 1 PARTIEL (cles i18n `networkStatus.{rejected,timed_out}` absentes) —
    ferme par addition 2 cles FR + 2 EN.
  - B : 5/5 CONFIRMED 0 GAP. D : 9/9 CONFIRMED 0 GAP.

### Track G — Carry-overs

- **CLOSED S72** : P2-H-1 (A), P2-F-3 3/3 (B — **plus jamais carry**),
  P2-A-2 (B), P3-A-3/B-1/B-2 (B documentes), DESIGN-CONFLICT C+D (resolus PO),
  P2-project-id-placeholder (resolu Phase E : `createSession` omet project_id,
  defaut serveur). Verifier chaque cloture (test + code).
- **Nouveaux S73** (P2/P3 non bloquants — router vers tracks ci-dessus) :
  P2-TIER-MODEL (A→Track B), P2-TEST-ZOMBIE (B→Track A), P2-OPERATOR-TIMEOUT
  (B→Track A), P2-sync-FS-async (D→Track C/perf), P2-POLL-DIAGNOSTIC-LOSS
  (D→Track B), P3-quorum-double-write (D documente §P53),
  P2-OLLAMA-MODEL-PICKER (E→feature S73/S74), P2-OPERATOR-NO-TESTS (E→Track A),
  P2-OPERATOR-VITEST-RUNNER (E, meme item infra). Verifier qu'ils sont
  documentes, pas oublies.
- **P2-A-1(S71) worker-pump iroh-docs Windows natif → 3/3 MANDATORY S73.**
  Doit **entrer dans le plan S73** (root-cause iroh-docs pump Windows OU
  exemption formelle CI-Linux-only ecrite), **pas reporte** une 4e fois. La
  famille s'est elargie S72 (`operator_sprint_history_endpoint` =
  P2-OPERATOR-TIMEOUT, meme classe de flake env).
- **Reconduits (exemptes / hors-scope)** : P2-A-1(rand upstream),
  P2-AUDIT-2(iroh transitives, pin 0.98), T-NN+2(iframe wasm),
  P3-OS-1(operator_server OR duplique, non touche S72), P3-F-1(recap body),
  LT-2(Radicle, trigger PENDING tag non pousse), LT-5/LT-7(post-v1.0/S75).
  Verifier qu'aucun n'atteint 3 reports sans exemption (sinon escalade G7).

### Track G1 presence (P1 bloquant si absent)

Verifier que `sprint72_design_review.md` existe dans `archive/v2.1/` (migre au
S73 Phase 0) avec scoring G1. Present au tip S72 dans `active/`. Absent sur
sprint feature non-trivial = **P1** (gate bypasse). Present sans scoring = P2.
Present avec 5/5 = OK.

### Track H — HARDENING review

S72 ajoute une **nouvelle surface reseau** : (1) NetworkProvider client
(Factory → daemon submit/poll/result), (2) route daemon
`GET /api/v1/tasks/{id}/result` (lecture resultat). Comparer
`HARDENING_ROADMAP.md §3` ligne S72 (items prescrits) vs livre :
- La surface Operator (token+Host+CORS+gate) etait deja S71 C ; S72 l'etend
  (catalogue P2-H-1) sans nouveau gate code (le gate SSE S71 couvre le
  dispatch provider).
- Verifier que `THREAT_MODEL.md` couvre la route `/result` (T0 read-only,
  Phase D l'a ajoutee §780) et la classe NetworkProvider (anticipation §14).
- Pour chaque item HARDENING prescrit S72 non livre : scope-cut justifie
  kickoff §7 ? blocker externe ? sinon **P2** (drift). Track informative (P2).

### Track I — Meta-process

- **2 DESIGN-CONFLICT G8 consecutifs (C, D)** : verifier qu'ils portent une
  evidence ground-truth concrete (collision schemars ; gap wire result) et un
  pivot_proposal repo-visible, et que l'arbitrage PO Option A (a) ne change
  aucune Day-0 unilateralement, (b) reste coherent D2/PO-14. **Signal meta a
  remonter** : le plan S72 a sous-estime 2 dependances structurelles du
  routage provider. Recommandation pour S73 : preflight S1b (chaine
  transitive deps) + S4 (contrat wire cross-composant) plus profonds sur les
  phases « cablage ».
- **Day-0 ajustees par arbitrage PO** : pin schemars 0.8→**1.2** + ollama-rs
  0.2.6→**0.3.4** partout (D2). Verifier que ces pins sont coherents (1 seule
  version ollama-rs ; schemars 1.2 snapshot `task_response.schema.json`
  draft-07→2020-12 regenere). Pre-launch : pas de consommateur externe du
  schema → bump libre.
- **Commit discipline** : 5 phases (A docs, B fix, C/D/E feat) + chores. Bodies
  9 sections phases code. Codex gate 5/5 (zero exemption, docs-only A n'a pas
  exempte). Verifier les SHA `105c054`/`08b6cb2`/`3c9ea1b`/`110c003`/`95cae05`.
- **Process env** : `nexus-phase-review-deep` et `nexus-process-supervisor`
  non enregistres → reviews = fallback agent `general-purpose` independant,
  supervision = hooks backstop (D17). Verifier que les reviews independantes
  existent et ont un verdict.

---

## §3 S73 Objective — Recherche reseau (contexte, hors audit)

Apres l'audit S72, S73 ouvre (roadmap v5 §3, Arc 3.5) la **recherche reseau** :
pont feed-distant → reindex FTS5 a chaud, enrichissement `SearchResult`
(repo_url + commit + archive_hash + provenance), barre de recherche shell
cablee `GET /api/daemon/search`, et **decision SearchManifest** (recherche
opt-in propagee). S72 a livre le **routage provider** (submit→poll vers le
reseau) sur lequel S73 construit le **chemin de decouverte** (chercher avant
de soumettre/forker). Le NetworkProvider S72 soumet une tache ; S73 apprend
a **trouver** une app/un projet sur le reseau. **Pre-requis a inscrire au
plan S73** : P2-A-1(S71) 3/3 MANDATORY (root-cause OU exemption CI-Linux).

---

## §4 Out of scope pour l'audit (NE PAS rebattre)

L'audit S72 **audite**, il ne re-conçoit pas. Ne pas rebattre :
- **D1..D5 gelees** : `ExecutionTarget` enum-dispatch (D1), ollama-rs 0.3.4
  partout (D2), NetworkProvider submit→poll → un seul Done (D3, PO-14),
  cablage `provider` backend (D4), 2 axes orthogonaux run/prompt (D5).
- **Pins arbitres PO** : schemars 1.2 + ollama-rs 0.3.4 (Option A Phase C).
- **Option A Phase D** : route daemon `/result` + colonne `result_text` (la
  correction du DESIGN-CONFLICT, pas une feature optionnelle a remettre en
  cause).
- **Les 16 scope cuts** (kickoff §7) — recherche S73, fork/templates/packaging
  S74, GPU/quorum cross-machine S75, sharding S76, token-par-token jamais.
- **Pre-launch policy** : pas de bump `*_VERSION` tant que rien n'est pousse ;
  canonical editable. Ne PAS exiger de migration wire.
- Re-corriger un P2/P3 deja documente (router vers S73+ phases, pas le
  re-implementer en Phase 0).

---

## §5 Track HARDENING drift (P2 informatif) — rappel

Cf. Track H. Drift cumule sur 3+ sprints sans justification → remonter le
signal pour revalider `HARDENING_ROADMAP.md` lui-meme. S72 = extension de
surface reseau (NetworkProvider + route /result), couverte par le catalogue
menace P2-H-1 (Phase A) + l'ajout `/result` T0 read-only (Phase D).

---

## §6 Verdict global attendu

- **PASS** : 0 P0, 0 P1 → S73 Phase A demarre direct. **Scenario attendu**
  (les ~8 P2 carries sont documentes, non bloquants ; les 2 DESIGN-CONFLICT
  sont resolus proprement ; G1 present).
- **CONDITIONAL PASS** : 1-3 P1 fixables → S73 Phase A bloque tant que les
  `fix(sprint72): ...` ne sont pas landed. Candidats P1 potentiels : G1
  design_review absent/sans scoring (improbable, present), gate
  SENSITIVE_ACTIONS court-circuite par un bras provider (a verifier Track B),
  route `/result` sans auth (a verifier Track B).
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle.

---

## §7 Livrable final attendu

`sprint72_audit_findings.md` (pattern Sprint 6/7), sections :
1. **Auditeur** — id session, duree.
2. **Tip audite** — SHA master pris comme base (tip code `95cae05`).
3. **Verdict global** — PASS / CONDITIONAL PASS / FAIL.
4. **Une section par track A-I** avec verdict (PASS / CONCERN / FAIL) +
   findings.
5. **Findings list sorted by severity** — table P0 → P3.
6. **Commits fix attendus** — si CONDITIONAL PASS, liste `fix(sprint72): ...`
   prealable au kickoff S73.
7. **P2 a logger en tech debt** — items vers `PATTERNS.md` sans code change.
8. **P3 laisses sans action** — nits ignores.
9. **Notes on audit completeness** — ce qui n'a pas ete couvert et pourquoi.

**Critere SMART** : la fail-fast §3 du `verification.md` rejoue verte en CI
Linux (1544 Rust / 0 skip) + 0 P0/P1 non resolu = S73 kickoff debloque +
P2-A-1(S71) 3/3 inscrit au plan S73.

**Exit Gate** : l'audit S72 est complet quand `sprint72_audit_findings.md`
porte un verdict avec >= 1 P2+ (G4), couvre les 9 tracks, ingere le diff
complet S72 (Phases 0 + A-E), et re-confirme que les 2 Option A
(schemars / route /result) sont saines et coherentes Day-0.
