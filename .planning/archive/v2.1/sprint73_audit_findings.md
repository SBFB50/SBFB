# Sprint 73 Audit Findings

Date: 2026-06-04
Auditeur: session fraiche S74 Phase 0 (Cas A audit gate). Orchestration multi-agent
(11 agents, ~1.53M tokens, 314 tool-uses) : 7 agents de track (B/C/D/A+E/F+G/H+I/
off-sprint) + 3 skeptics adversariaux sur les candidats P1 + 1 synthese, sous regle
anti-anchoring (opinion formee depuis le code AVANT lecture des self-reports).
Sprint audite: 73 (recherche reseau cablee — FTS5 fraicheur + SearchResult enrichi
triplet provenance + barre shell + guardrail securite + fermeture dette worker-pump).
Diff: `845bea6..9472085` (code S73, 52 fichiers, +7601/-206) + reconciliation
off-sprint `9472085..5ffb628` (Phase F docs `409eb6a` + 2 hotfixes `9c3085a`/`5ffb628`).
Tip code: `9472085`. HEAD reel au demarrage S74: `5ffb628`.

## Verdict global: **PASS**

0 P0, 0 P1, 14 P2, 8 P3. Rigor G4 satisfait (>= 1 P2+ documente, pas CONCERN).
Les 3 candidats P1 du plan (section 6) sont TOUS REFUTED avec evidence file:line.
G1 design_review present + scoring. Le verdict gate uniquement sur P0/P1.
**S74 Phase A (atelier fork) demarre direct — aucun commit fix(sprint73) requis.**

Scenario conforme a l'attendu du plan section 6 : les invariants headline tiennent
(guardrail AVANT persist sur la surface recuperable ; worker-pump 3/3 empiriquement
vert sous le gate d'origine ; defer SearchManifest tranche zero-wire) ; les 14 P2 sont
des questions de couverture-test / precision-doc / hazards latents pour le browse-
indexing S74, non bloquants.

---

## Track A — Suites verification : PASS

Re-run independant complet, dual-platform (feedback_dual_platform / feedback_wsl_before_push).

| Suite | Resultat | Attendu plan |
|-------|----------|--------------|
| Windows natif `cargo nextest run --workspace` | **1566 run / 1566 passed / 0 skip** | 1566 / 0 skip OK |
| `cargo fmt --all --check` | exit 0 | OK |
| `cargo clippy --workspace --all-targets -D warnings` | exit 0 (0 warning) | OK |
| `cargo test --workspace --doc` | exit 0 | OK |
| **worker-pump `cargo test -p nexus-shell-daemon -p nexus-worker-core`** (gate d'origine du hang P2-A-1, shared-process) | **nexus-shell-daemon 190 passed / 0 failed / 0 ignored ; worker-core 279 passed ; factory 6/7 ; AUCUN hang** | 0 hang OK |
| Docker Linux `sbfb-ci` (rustc 1.95.0, apt libgtk-3-dev) nextest workspace | **1570 run / 1570 passed / 0 skip** | 1570 / 0 skip OK |
| Docker Linux doctests | exit 0 | OK |
| `web/` Vitest | **289 passed / 24 fichiers** | 289 OK |
| `web/` lint / tsc / build / scan-en-strings | 0 erreur (5 warnings lint pre-existants), tsc clean, build OK, "src/ is French-only, clean" | OK |
| `web/` size-limit | **6/6 sous limites** (main 25.93/50, vendor-react 275.49/290, vendor-query 102.48/120, vendor-ui 262.27/270, CommandPalette 9.81/20, css 122.81/130) | 6/6 OK |
| `tools/factory-operator` Vitest (`test:unit`) | **7 passed / 2 fichiers** | 7 OK |

**Reconciliation compte** : entree canonique Linux S72 = 1544 ; sortie S73 Linux = 1570
(+26). Windows = 1566 (+22). L'ecart +4 (Linux-only) = tests `#[cfg(unix)]` confirmes
structurels (cf. A.1, Track E.1), pas des skips masques. Deltas par phase Windows
A +5 / B +7 / C +5 / D +5 / E +0 = +22, coherent.

Findings: A.1 (P3, +4 cfg(unix) structurel + ambiguite baseline 1544 — informatif).
Le run Docker Linux 1570/1570 couvre bien M17 + hot-reindex (rustc 1.95.0, pas un
re-cite du 1556/1560 de Phase B anterieur a M17) : confirme.

## Track B — Security review : PASS

Coeur securite S73 = (1) guardrail AVANT persist 2 chemins (D5), (2) surface search,
(3) integrite triplet provenance.

- **B.1 (candidat P1) — REFUTED.** Grep exhaustif de tous les appelants `set_task_result` :
  l'UNIQUE writer prod du literal `status='completed'` + `result_text` est `db.rs:443`,
  appele uniquement par `validate_result_post_guardrail` (`validator.rs:158`), elle-meme
  appelee depuis `http.rs:1532` (apres `default_output_chain().run()` 1515, branche trip
  1516-1530 = 400 + zero persist + zero kudos) et `validator_loop.rs:91` (apres guardrail
  81, branche trip 82-90 = skip). `ResultValidator::validate` (`validator.rs:320`) et
  le `validate_result` test-mod (`validator.rs:349`) — les seules compositions pre+post
  guardrail-less — sont gate `#[cfg(test)]` (`validator.rs:299/304/330`). `update_task_status`
  prod ne pose JAMAIS Completed (validator.rs:215 AwaitingQuorum, :279 Rejected).
  `kudos_ledger::credit` prod = http.rs:1542 + validator_loop.rs:99, tous deux dans la
  branche Accepted+guardrail-passed. **Le bug headline S72 n'est PAS re-ouvert.** Residu
  (P2 -> PATTERNS) : `PendingResultPersist` ne porte aucune preuve de type que le guardrail
  a tourne — l'ordre est une convention d'appelant, pas un invariant de type ; deja
  signale par Codex Phase A Run1 PARTIAL -> Run2 ferme par le gate cfg(test).
- **B.2 (P2, CONFIRMED) -> S74.** Sur quorum (`redundancy>1`) ou le texte agree trip le
  guardrail (`validator.rs:239-277`), aucun statut terminal n'est pose -> la tache reste
  `awaiting_quorum` (zombie, re-trip a chaque soumission), contrairement a la branche
  divergence (`validator.rs:279` Rejected). Surface recuperable SAINE : aucun endpoint ne
  lit `task_results.sha256` brut (`get_task_results` db.rs:486 consomme uniquement par
  validator.rs:210 ; `search_handler` http.rs:1975 ne lit que search_index). Le claim
  "jamais persiste" doit etre qualifie "jamais sur la surface recuperable GET /result".
  Gap test : aucun test daemon n'exerce `redundancy>1 x guardrail-trip` (seul test quorum
  validator.rs:717 appelle la fn pre directement).
- **B.4 (P2, CONFIRMED, PRE-EXISTANT) -> PATTERNS.** Les 2 ingress construisent
  `GuardrailContext{ system_prompt:"" }` (`http.rs:1510-1514`, `validator_loop.rs:76-80`).
  `default_output_chain` (guardrails.rs:147) n'a qu'une regle (OutputSafety) dont 3 des 4
  sous-regles (PromptEcho exact/substring/eed, output_filter.rs:58/62/80) gatent sur un
  system_prompt non-vide -> inertes ici. Seul InvisibleText (prompt-independant) est actif
  sur la surface result-submission. Le coordinateur ne stocke pas le system_prompt de la
  tache. A documenter dans THREAT_MODEL section 14.
- **B.5 (P2, CONFIRMED, carry P2-3) -> S74.** `isHttpsUrl` (Browse.tsx:153) est correcte
  (javascript:/data:/http:/`//evil` tous rejetes, startsWith case-sensitive) + anchor
  target=_blank rel=noopener noreferrer. Mais appliquee uniquement sur la NOUVELLE
  SearchHitCard ; les 3 ancres `repo_url` pre-existantes (Browse.tsx:471,
  BrowsedProject.tsx:367, VerificationDetail.tsx:185) restent non gardees (mitigant : le
  feed `validate_known_operation` public_feed.rs:271 impose deja repo_url https://). Test
  XSS mono-vecteur (javascript: seul). Threat = DOM shell trusted, pas l'iframe sandbox.
- **B.6 (P2, CONFIRMED, latent S74) -> S74.** `extract_index_fields` (search.rs:~210) lit
  `is_open_source`/`provenance_hash` independamment sans cross-check ; `index_entry`
  (chemin browse) prend une Provenance arbitraire sans valider l'invariant
  `is_open_source => provenance_hash` (que `public_feed.rs:285` impose pour le feed).
  Aujourd'hui seul le feed valide atteint l'index -> inerte ; S74 browse-indexing doit
  re-appliquer l'invariant spec.
- **B.3 (P3, CONFIRMED).** HTTP->loop re-broadcast (`http.rs:1551-1553`) est un no-op
  doublement garde (statut completed + `set_task_result WHERE status IN (...)` db.rs:444)
  -> pas de double-credit kudos. Non teste sur ce chemin precis.
- **B.7 (P3, CONFIRMED).** `last_err` token-free : le message d'erreur surface `r.status()`
  / texte body, jamais le header `X-SBFB-Token` ni les URLs. Invariant tient.

## Track C — Patterns review : PASS

- P56 (FTS5 hot reindex + triplet UNINDEXED) : exact (rowid=seq, helper
  `extract_index_fields` partage, M17 DROP/recreate, tripwire collision rowid browse/feed).
- P53/P55 (lot doc S72) : corriges (rename ModelOptions/0.3.4, P2-A-2 ferme,
  Box<dyn LlmBackend> trait, PROVIDERS:&[&str]). Confirme.
- **C.2 = P2-FRESHNESS-RELEASE-UNINDEXED (P2, CONFIRMED) -> S74.** `extract_index_fields`
  (search.rs:199-225) ne lit que reason/comment ; `ReleasePublishedPayload`
  (public_feed.rs:32-40) n'a ni reason ni project_name ni category -> un hot-upsert d'un
  ReleasePublished indexe `description=""` -> invisible a la recherche full-text. Le claim
  "un projet gossipe devient cherchable a l'instant" est surcote pour l'op la plus
  importante (publication projet) ; seuls CuratorVouched/SourceBecameStale (porteurs de
  reason) deviennent matchables. Aucun des 5 tests Phase C ne couvre un ReleasePublished.
- **C.3 = P2-ROWID-PARTITION (P2, CONFIRMED, tripwire S74) -> S74.** `index_entry`
  (search.rs:67-97) INSERT sans rowid explicite -> FTS5 `max(rowid)+1`, partageant l'espace
  rowid des upserts feed (rowid=seq). Les 2 appelants `index_entry` (http.rs:6419, :6459)
  sont `#[tokio::test]` (browse-indexing test-only en prod). Aucun appelant prod n'a glisse.
  Si S74 cable le browse-indexing prod : un upsert feed seq=N clobbe une ligne browse
  rowid=N (INSERT OR REPLACE). Tripwire doc search.rs:241-244 presente.
- **C.4 = P2-UPSERT-NO-CATCHUP (P2, CONFIRMED) -> PATTERNS.** `feed_sync.rs:260-279` ne fait
  que `warn!` sur echec upsert (l'insert durable est deja commit) ; `rebuild_from_feed`
  (runtime.rs:778) est boot-only -> un echec SQLITE transitoire laisse 1 entree
  non-cherchable pour la vie du noeud, sans metrique de drift.
- **P3-runtime-1906-cfg (P3, REFUTED, cf. skeptic worker-pump).** `runtime.rs:1906`
  `rate_limit_gate_reloads_live_policy` est un test current_thread qui ne PILOTE pas le
  pump (jamais de spawn pump) -> exemption P54 genuine, pas une violation. Le run Windows
  nextest 1566 0-skip + cargo test 190 passed 0 hang l'inclut.

## Track D — Scope cuts compliance : PASS

14/14 scope cuts respectes (grep exhaustif `git diff --name-only` + cible). Aucune ligne
S73 ne touche SearchManifest reseau-large (#1), search/open/fork Factory (#2), projet cible
distinct (#3), reseau->fork (#4), templates etendus (#5), GPU cross-machine (#6), quorum
cross-MACHINE (#7), sharding (#8), Tantivy (#9), @dev tree-sitter (#10), webhook/SSE feed
push (#12), token-par-token WAN (#13), pagination boutons (#14).

- **D.1 (P2, RECLASSIFIED) -> S74.** Scope cut #11 (re-eval binaire rate-limit search Phase E)
  est respecte mais la justification "debounce de fait" est materiellement FAUSSE : il n'y a
  aucun debounce keystroke dans Browse.tsx (le seul rate-bound = React Query staleTime,
  contournable en tenant une cle). `GET /api/daemon/search` est bien loopback-only derriere
  auth (route http.rs:360 / layer :436) ; `q` sans longueur max, `offset:usize` non-clamp
  (search.rs:103,122). Le residual T-SEARCH-DOS repose donc uniquement sur loopback
  single-user, pas sur un debounce. THREAT_MODEL section 11 (lignes ~583-597) est stale sur
  ce point — a recadrer S74.
- **D.2 (P3, REFUTED, R6).** SearchManifest defere (D3) TRANCHE : design note
  `.planning/research/s73_searchmanifest_index_node_design.md` capture la forme correcte
  (noeud-index opt-in Ed25519, default OFF, requetes jamais broadcast, 7 modeles OSS) +
  ZERO code wire (`PublicFeedOperation` = 4 variantes exactes, SearchManifestPublished
  commentaire forward-compat :78, FEED_FORMAT_VERSION=1). Decide + documente + PO-13 honore.
- **D.3 (P3, REFUTED).** `default_model_for_provider("network")=llama3.2:latest`
  (operator_server.rs:331) avec echappatoire `SBFB_NETWORK_DEFAULT_MODEL` = la dette
  P2-OLLAMA-MODEL-PICKER fermee, pas un scope creep.

## Track E — Tests delta coherence : PASS

Deltas reconcilies (cf. Track A). +22 Windows = A+5/B+7/C+5/D+5/E+0 ; +4 cfg(unix) Linux.

- **E.1 (cf. A.1, P3).** +4 confirme `#[cfg(unix)]` (auth.rs:304 UDS peer-cred,
  e2e.rs:280, shell-daemon-core/auth.rs:1019, worker/e2e.rs:364), pas des skips.
- **E.2 (cf. B.2).** Les 5 tests Phase A ont des assertions load-bearing
  (`result_text.is_none()` + kudos==0 sur trip). Gap : branche post_guardrail->Err
  (http.rs:1531, validator_loop.rs:91) et `redundancy>1 x trip` untestees.
- **E.3 (P2, CONFIRMED) -> S74.** 3 tests Phase C/D promettent plus que leurs assertions :
  `reindex_hot_is_idempotent` (search.rs:516) asserte `total==1` mais pas le REWRITE du
  contenu ; `extract_index_fields_shared_with_rebuild` (search.rs:535) teste une op porteuse
  de reason, pas un ReleasePublished ; `migration_m17_recreates_index_unindexed`
  (search.rs:672) simule clear_all+rebuild in-memory, jamais un upgrade reel user_version
  16->17 sur table peuplee.
- **P2-SEARCH-VIEW-THROW-SKELETON (P2, NEW auditor-found, hors self-report) -> S74.**
  `callDaemon` THROW `ApiProtocolError` sur strict() parse-fail (daemon.ts:~249) ;
  `SearchResultsView` (Browse.tsx:200-210) n'a aucune branche `query.isError` -> un drift
  Rust<->Zod yield un LoadingSkeleton infini, pas une carte d'erreur. Non teste
  (daemon.test.ts:605 couvre le throw cote API mais pas le rendu).
- **E.4 (P3).** Vitest +10 vs plan +4 : le +6 adversarial exerce des branches distinctes
  (encode pathologique, 503->unavailable, triplet null, strict-omitted-key reject, XSS
  non-https, grille vide) — pas du padding. 289/289 confirme.

## Track F — Review files quality + presence : PASS

- 5 preflight (A-E) presents ; verdicts G8 A/B/C/D EXECUTE, E SCOPE-CUT-CONSISTENT.
- 5 reviews (A-E) toutes promues `## Verdict: PASS`. 5 codex_review bruts presents.
  Ratio 5/5. Phase A Codex Run1 PARTIAL (ResultValidator guardrail-less) -> Run2 ferme par
  le gate cfg(test) : reconciliation confirmee. Chaque PARTIEL/GAP des 5 reconcilie.
- **F-HEADER (P3, NEW cosmetic) -> PATTERNS.** `sprint73_phase_e_review.md:3` porte une
  variante d'espacement du header verdict (residu malgre le chore `5361fd8` de
  normalisation). Cosmetique, non bloquant.

## Track G + G1 — Carry-overs + design review : PASS

- **G1 (presence P1-bloquant) : PRESENT.** `sprint73_design_review.md` existe (committe
  `845bea6`) avec scoring G1 (D1 OK, D2 warn, D3 warn, D4 OK, D5 OK, D6 OK). Gate non
  bypasse. -> PASS.
- 12 carries CLOSED S73 verifies : Phase B 7 P2 (P2-A-1 worker-pump 3/3 PLUS JAMAIS CARRY,
  P2-TEST-ZOMBIE fixtures git hermetiques, P2-OPERATOR-TIMEOUT 30s, P2-OPERATOR-NO-TEST-
  RUNNER Vitest 7, P2-POLL-DIAGNOSTIC-LOSS last_err, P2-SYNC-FS-ASYNC spawn_blocking,
  P2-OLLAMA-MODEL-PICKER), Phase A 3 doc (P2-RESULT-TEXT-GUARDRAIL-ORDER, P2-TIER-MODEL,
  P2-HARDENING-ROADMAP-META-STALE), Phase F 2 process (P2-PREFLIGHT-TRANSITIVE-DEPTH,
  P2-PREFLIGHT-WIRE-CONTRACT-DEPTH — amendes concrets avec commandes cargo tree -d +
  exemple nullable-vs-optional dans prompts/agent/preflight.md + skill + agent-deep).
- Reconduits : aucun n'atteint 3 reports sans exemption (pas d'escalade G7 manquee).
  P2-A-1 rand upstream (exemption externe), P2-AUDIT-2 iroh transitives (pin 0.98 gele),
  T-NN+2 wasm (PATTERNS P34), P3-OS-1 operator_server OR duplique (non touche S73),
  **LT-2 Radicle trigger PENDING** (tag v1.0 pose localement, PAS pousse origin, 37+ ahead —
  a surveiller : le trigger arme au push origin).

## Track H — HARDENING review : PASS

THREAT_MODEL section 14 (guardrail AVANT persist) + section 11 (search surface, recadre
boot->hot) coherents avec le code. HARDENING_ROADMAP section 3 recadre (last_validated
2026-06-03) tient. M17 = schema local (pas un wire).

- **H.1 = P2-M17-BOOT-RECOVERY-WARN-ONLY (P2, CONFIRMED) -> S74.** Un noeud existant qui
  upgrade DROP son index peuple a M17 ; recovery via `rebuild_from_feed` best-effort au boot
  qui ne fait PAS fail boot (runtime.rs:778-781 `warn!`). Si le rebuild echoue post-DROP ->
  index silencieusement VIDE jusqu'a un reboot reussi. Pre-launch recoverable (feed durable),
  mais la recovery est gatee derriere un echec silencieux.
- **H.2 (P2, CONFIRMED) -> S74.** `rebuild_from_feed` ne restaure que `source_type='feed'`
  (search.rs:281-283) ; M17 DROP tout. Le claim "integralement reconstructible" vaut
  uniquement pour la tranche feed — les browse-rows (S74) seraient perdues. Carry.

## Track I — Meta-process : PASS

- Commit discipline : 5 phases (A/B fix, C/D feat search, E feat shell) + chores (kickoff
  845bea6, normalize 5361fd8, Phase F docs 409eb6a). Bodies 9 sections sur les phases code,
  section Codex verification presente, pas de --no-verify/--amend, pas d'emoji. SHA
  6f5ff30/a4e1542/47c9ff7/0f86e5a/9472085 confirmes. Phase F `docs(sprint73)` titre SANS
  "Phase X" -> hooks lightcheck Check 5/7/8/9 NON armes (precedent S71, attendu, README 3.3).
- Env process (inchange S72->S73) : `nexus-phase-review-deep` ET `nexus-process-supervisor`
  non enregistres -> reviews = fallback agent general-purpose ; supervision = hooks backstop
  D17. Reviews independantes existent et portent un verdict.
- P2-PREFLIGHT-* (Phase F) repondent concretement a la lecon meta S72 (2 DESIGN-CONFLICT
  consecutifs sous-estimaient deps transitives + contrat wire). Cet audit le confirme : 0
  DESIGN-CONFLICT en S73, amendements load-bearing (pas boilerplate).

## Off-sprint reconciliation (G5/G6) : CONCERN (RECONCILED, non bloquant)

Apres le tip code S73 (`9472085`) et le wrap-up Phase F (`409eb6a`), 2 hotfixes off-sprint
ont ete commit AVANT cet audit gate (absents de la memory et de verification.md S73). Tous
deux RECONCILES (root-cause, zero regression) mais sans test de non-regression :

- `409eb6a docs(sprint73)` (Phase F) : artefact de cloture LEGITIME. `git show --stat`
  confirme uniquement des .md + 2 fichiers .claude (skill/agent preflight) + prompts/agent —
  aucun code .rs/.ts fonctionnel deguise en docs.
- **OFF-SPRINT-1 (P3, RECONCILED).** `9c3085a fix(examples)` : SBFB.json bridge.methods (objet
  imbrique, forme correcte) vs bridge_methods. Le parseur (lib.rs:11-41) n'a PAS
  `deny_unknown_fields` -> l'ancienne cle bridge_methods etait silencieusement droppee
  (data-only, pas un parse error). Fix correct. Note process : la tolerance serde masque ce
  type de bug de manifest -> un test de fixture viewer l'aurait attrape.
- **OFF-SPRINT-2 (P2, RECONCILED) -> S74.** `5ffb628 fix(daemon)` : deploy.rs derive un
  project_id PER-APP (deploy.rs:141-143,842) pour qu'un noeud hebergeant plusieurs apps
  expose plusieurs Browse cards distinctes. Root-cause correct, pas band-aid. MAIS aucun
  test de non-regression.
- **OFF-SPRINT-2b (P2, NEW auditor-found) -> S74.** Le fix per-app est INCOMPLET : seuls le
  chemin deploy.rs derive le project_id per-app ; les chemins `/publish` (http.rs:1004) et
  gossip/announce (runtime.rs:1569, publish.rs:39-40) gardent encore le node_id comme cle ->
  un noeud multi-app reste partiellement collisionnant sur ces chemins. A completer S74.

---

## Findings list — sorted by severity

| ID | Sev | Track | Disposition | Titre | Route |
|----|-----|-------|-------------|-------|-------|
| (aucun) | P0 | - | - | - | - |
| (aucun) | P1 | - | - | - | - |
| B.2/E.2 | P2 | B+E | CONFIRMED | Quorum guardrail-trip zombie + no redundancy>1 trip test | S74 |
| P2-FRESHNESS-RELEASE-UNINDEXED (C.2) | P2 | C+H | CONFIRMED | ReleasePublished indexe texte matchable vide | S74 |
| P2-ROWID-PARTITION (C.3) | P2 | C+B | CONFIRMED | rowid partage browse/feed (tripwire avant browse-indexing prod) | S74 |
| B.6 | P2 | B | CONFIRMED | Invariant is_open_source=>provenance_hash non re-applique au chemin browse | S74 |
| C.4 | P2 | C+H | CONFIRMED | Hot upsert echec warn-only sans catch-up runtime | PATTERNS |
| H.1 | P2 | H | CONFIRMED | M17 boot-recovery warn-only -> index silencieusement vide | S74 |
| H.2 | P2 | H | CONFIRMED | Reconstructible limite a la tranche feed (pas browse-rows) | S74 |
| D.1 | P2 | D+H | RECLASSIFIED | Cut 11 "debounce de fait" faux + THREAT_MODEL 11 stale | S74 |
| B.4 | P2 | B | CONFIRMED | system_prompt vide -> 3/4 regles output-filter inertes (PRE-EXISTANT) | PATTERNS |
| B.5 | P2 | B | CONFIRMED | isHttpsUrl mono-vecteur + 3 ancres repo_url pre-existantes non gardees | S74 |
| E.3 | P2 | E | CONFIRMED | 3 tests Phase C/D promettent plus que les assertions prouvent | S74 |
| P2-SEARCH-VIEW-THROW-SKELETON | P2 | E+B | CONFIRMED (NEW) | SearchResultsView sans isError -> skeleton infini sur drift Zod | S74 |
| OFF-SPRINT-2 | P2 | off | RECONCILED | deploy project_id per-app root-cause mais 0 test non-regression | S74 |
| OFF-SPRINT-2b | P2 | off | NEW | Fix per-app incomplet : /publish + gossip gardent node_id | S74 |
| B.1 (residu) | P3* | B | REFUTED | 3e chemin guardrail-less REFUTED ; residu convention-not-type | PATTERNS |
| P3-runtime-1906-cfg | P3 | C+F | REFUTED | runtime.rs:1906 exemption P54 genuine (worker-pump P1 REFUTED) | PATTERNS |
| D.2 | P3 | D | REFUTED | SearchManifest defer tranche zero-wire (R6) | none |
| D.3 | P3 | D | REFUTED | Network default model = cloture dette, pas scope creep | S74 |
| A.1 | P3 | A | CONFIRMED | +4 Linux/Windows structurel cfg(unix) ; baseline 1544 a clarifier | PATTERNS |
| B.7 | P3 | B | CONFIRMED | last_err token-free invariant tient | none |
| B.3 | P3 | B | CONFIRMED | HTTP->loop re-broadcast no-op doublement garde, non teste | none |
| F-HEADER | P3 | F | NEW | sprint73_phase_e_review.md:3 variante espacement header verdict | PATTERNS |
| OFF-SPRINT-1 | P3 | off | RECONCILED | manifest bridge.methods correct ; serde silent-drop sans deny_unknown_fields | none |

(*) B.1 : le candidat P1 "3e chemin" est REFUTED ; l'audit logge le residu architectural
convention-not-type comme P2-watch-item -> PATTERNS. Listed P3 en disposition (refute),
P2 en routing-charge (cf. summary counts du workflow). Compte conservateur ci-dessous.

## Summary

| Severite | Count | Items |
|----------|-------|-------|
| P0 | 0 | - |
| P1 | 0 | (3 candidats refutes : B.1, worker-pump/1906, wire/SearchManifest) |
| P2 | 14 | B.2/E.2, FRESHNESS-RELEASE-UNINDEXED, ROWID-PARTITION, B.6, C.4, H.1, H.2, D.1, B.4, B.5, E.3, SEARCH-VIEW-THROW-SKELETON, OFF-SPRINT-2, OFF-SPRINT-2b |
| P3 | 8 | runtime-1906-cfg, D.2, D.3, A.1, B.7, B.3, F-HEADER, OFF-SPRINT-1 |

## Resolution des 3 candidats P1 (section 6 du plan)

1. **B.1 (3e chemin pre+post sans guardrail) — REFUTED.** `db.rs:443` = SEUL writer prod
   de result_text/completed ; unique appelant prod `validator.rs:158` post-guardrail ;
   `ResultValidator` guardrail-less gate cfg(test). Bug S72 non re-ouvert.
2. **A.1/C.2 (worker-pump hang sous cargo test / runtime.rs:1906 deadlock) — REFUTED.**
   `runtime.rs:1906` = exemption P54 genuine (ne spawn jamais le pump). Empiriquement :
   Windows nextest 1566/1566 + worker-pump `cargo test` shared-process 190 passed 0 hang.
   P2-A-1 3/3 CLOSED, plus jamais carry.
3. **D (wire bump / variante SearchManifest glissee) — REFUTED.** `public_feed.rs` INTOUCHE
   dans le diff, 4 variantes PublicFeedOperation, FEED_FORMAT_VERSION=1, M17 = schema local,
   SearchResult = type de reponse HTTP loopback (Serialize-only, pas un message gossip).

## Commits fix attendus avant kickoff S74

**Aucun.** 0 P0, 0 P1. S74 Phase A (atelier fork) demarre direct.

## P2/P3 a logger en tech debt (PATTERNS, sans code change)

B.1 residu convention-not-type, B.4 system_prompt vide regles inertes, C.4 upsert warn-only,
P3-runtime-1906-cfg exemption P54, A.1 baseline 1544 + cfg(unix), F-HEADER espacement.

## Carry-over to Sprint 74 (a inscrire au plan S74)

**Pre-requis browse-indexing (avant tout cablage prod)** : C.3 partition rowid browse/feed,
B.6 re-application invariant is_open_source=>provenance_hash. **Recherche/fraicheur** :
C.2 ReleasePublished non-indexe (nom de projet indexable), H.1 M17 recovery non-silencieuse,
H.2 reconstructibilite browse-rows. **Robustesse/securite** : B.2/E.2 quorum zombie +
statut terminal + test redundancy>1, D.1 recadre THREAT_MODEL 11 (residual loopback, pas
debounce), B.5 normaliser isHttpsUrl sur les 3 ancres + test multi-vecteur,
SEARCH-VIEW-THROW-SKELETON branche query.isError. **Off-sprint** : OFF-SPRINT-2 test
non-regression deploy per-app, OFF-SPRINT-2b completer per-app sur /publish + gossip.
**Tests** : E.3 renforcer les assertions des 3 tests Phase C/D. **Reconduits** : P2-A-1 rand
upstream, P2-AUDIT-2 iroh transitives (pin 0.98), T-NN+2 iframe wasm, P3-OS-1 operator_server
OR duplique, LT-2 Radicle (trigger au push origin — PENDING).

## Notes on audit completeness

- **Couvert** : les 9 tracks (A-I) + G1 presence + reconciliation off-sprint (G5/G6). Diff
  complet S73 (845bea6..9472085, 52 fichiers) ingere ; 2 hotfixes off-sprint + Phase F docs
  reconcilies. Suites re-jouees independamment dual-platform (Windows natif + Docker Linux
  sbfb-ci canonique rustc 1.95.0 + frontend web/factory-operator/size). Les 3 candidats P1
  tranches par skeptics adversariaux dedies.
- **Methode** : orchestration multi-agent (11 agents, ~1.53M tokens) ; regle anti-anchoring
  respectee (opinion code-first avant self-reports) ; PATTERNS.md lu seulement par le Track C
  apres formation d'opinion.
- **Non re-conçu (out of scope, section 4 du plan)** : D1..D6 gelees, 14 scope cuts, M17 =
  schema local (pas de migration wire exigee), FEED_FORMAT_VERSION=1, pre-launch policy
  (rien pousse). Les P2/P3 deja documentes sont routes S74+, pas re-implementes en Phase 0.
- **Limite connue** : le compte canonique baseline S72 (1544 Linux) et l'arithmetique +26
  Linux / +22 Windows reposent sur la decomposition cfg(unix) confirmee par grep (A.1, P3
  informatif), pas par un `nextest list` cross-plateforme exhaustif (les 2 runs etant verts
  0-skip, le risque de skip masque est ecarte).

**Exit gate** : fail-fast verification.md re-jouee verte (Windows 1566/0-skip + Docker Linux
1570/0-skip + worker-pump cargo test 190 passed 0 hang + web 289 + factory-operator 7 +
size 6/6) + 0 P0/P1 + G1 present = **S74 kickoff DEBLOQUE (atelier fork).**
