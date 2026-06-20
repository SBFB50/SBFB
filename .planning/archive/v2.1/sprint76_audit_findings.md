# Sprint 76 — Audit findings (audit gate de S76, joue en S77 Phase 0)

Pattern Sprint 6/7. Verdict d'entree de S77 Phase 0.

---

## 1. Auditeur

- **Session** : session fraiche S77, Phase 0 (Cas A audit gate).
- **Methode** : orchestration **Workflow ultracode multi-agents anti-anchoring**
  (19 agents, ~1.55M tokens, 436 tool-uses). Fan-out des **10 tracks A-J**
  (1 agent independant par track, lecture STATIQUE no-compile), puis
  **verification adversariale** de chaque finding P0/P1/P2 (1 skeptic dedie par
  finding, mandate de REFUTER), puis **synthese** du verdict. Opinion formee
  depuis le **code livre + `git show`/`diff`/`log`** (plage `8b53c38..5cb3c72`)
  AVANT toute lecture des self-reports (`sprint76_verification.md`,
  `sprint76_phase_*_review.md`, `*_codex_review.md`), conformement au §0 du plan
  d'audit (ordre de lecture impose).
- **Suite autoritaire** : re-jouee EN PARALLELE par l'orchestrateur (main thread),
  hors des agents statiques (regle anti-OOM linker MSVC : 1 seul build Rust lourd
  a la fois). Resultats reels ci-dessous (§4 Track A) — pas une simple relecture
  du self-report.
- **4 candidats P1** du plan d'audit (§6) passes au crible d'un skeptic chacun :
  **les 4 REFUTES** (NON-P1), evidence file:line.
- **Self-reports** : lus APRES formation d'opinion, pour COMPARER. Les ecarts
  self-report vs audit sont signales explicitement (le plus net : 3 noms de tests
  cites dans `verification.md` §6 ne resolvent vers aucune fonction — corrige,
  cf. §6).

---

## 2. Tip audite

- **Base** : `8b53c38` (S75 Phase G wrap-up = `73831c0~1`, dernier commit pre-S76).
- **Tip code S76 (Phase 0 + 7 phases A-G)** : `5cb3c72` (Phase G wrap-up ; tip
  code A-F = `a547de6`).
- **Plage d'audit** : `8b53c38..5cb3c72`.
- **Commits feat/fix/docs** : Phase 0 `73831c0` (findings S75) + `23a08c9` (fix
  duress), kickoff `3faee6e`, A `ce43894`, B `6904cdd`, C `1cc28e7`, D `d75ae77`,
  E `768e235`, F `a547de6`, G `5cb3c72`. Chores intercales : verif self-reports
  `5b07472`/`1de6f8a`/`24bda54`/`df86bdc`, superviseur-supprime `42c7448`,
  README-bootstrap `a21aaad`.
- **Tip reel de l'arbre au moment de l'audit** : `52e70d1` (le fix-now doc-integrite
  de ce gate, cf. §6) au-dessus de `5cb3c72`.

---

## 3. Verdict global

### CONDITIONAL PASS

**Justification.** Le coeur de S76 (GPU partage cross-machine : panneau
« offrir ma puissance » + worker co-localise, E2E cross-machine compute B-3,
cohorte homogene advisory, quorum redundancy>1 deterministe + fix bridge
result-sync, dashboard contributeur + sanity-bound, quantization doc-only) est
**SAIN** : **0 P0, 0 P1 confirme**, 6 P2 + 9 P3 documentes. Les invariants
headline tiennent tous :

- **snapshot consent additif 0-bump** (SCHEMA_VERSION=1 inchange, level+caps
  copies seulement, fail-closed, aucune escalade de privilege) ;
- **fix bridge quorum** `dedup {hex(worker_pubkey)}:{task_id}` miroir EXACT du
  validator + DB `UNIQUE(task_id, worker_id)`, ferme le trou cross-machine
  (rouge-avant-vert structurellement valide) ;
- **validator INCHANGE** (`git diff validator.rs` = un seul hunk, integralement
  dans `mod tests`) ;
- **P1 `generation_time_ms` fixe a la RACINE** (le worker mesure
  `gen_start.elapsed()` reel ; le hardcode 0 supprime ; propage aux 2 SEULS
  callers prod de `credit()`) ;
- **backend quantization INCHANGE** (`llama_cpp.rs` 0 diff,
  `with_split_mode`/`with_devices` = 0) ;
- **0 bump wire** (25 constantes `*_VERSION` inchangees, `canonical.rs` 0
  changement, additivite stricte `#[serde(default)]`) ;
- **fmt 0** sous les 2 toolchains (le fix `http.rs:8531` est la vraie cause ;
  faux-diagnostic « derive 1.95/1.94 » retracte honnetement) ;
- **acceptance LIVE honnete** (#26/#30 DIFFERE-materiel-operateur, jamais 38/38).

**Les 4 candidats P1 du plan sont TOUS refutes depuis le code** (cf. §4) :
(1) le quorum cross-machine EST prouve par 2 tests hermetiques qui exercent le
VRAI chemin `forward_result_entry -> validator_loop::run ->
validate_quorum_pre_guardrail -> DB` sur un node iroh reel sans mock ;
(2) le fix `generation_time_ms` se propage aux 2 seuls callers prod de `credit()`
(tous les autres sites hardcodes sont sous `mod tests`) -> kudos non-gonflable
par ce vecteur ; (3) `model_digest`/cohorte sont strictement **advisory**
(`Verifier::verify` 0 appelant prod, claim-gate continue-sans-claim, aucun chemin
d'acceptation de resultat ne lit le tuple — le quorum vote sur `sha256(result_text)`) ;
(4) les deltas de tests annonces (A4/B8/C10/D4/E10/F5/G0 = **41**) correspondent
EXACTEMENT au git-count statique des attributs `#[test]` ajoutes, `Cargo.lock` =
0 delta -> pas de faux-vert.

**Le SEUL motif de CONDITIONAL** (et non PASS sec) : un defaut d'integrite-documentaire
reel et PROPAGE — `verification.md` §6 cite **3 noms de tests qui ne resolvent
vers aucune fonction** (`grep "fn <name>" crates/` = 0), recopies dans
`sprint77_audit_plan.md` ET `b3_live_pc_vps.sh:112`. La couverture fonctionnelle
EXISTE et est verte (les tests survivants couvrent la propriete), mais les
pointeurs nommes mis-dirigent le prochain gate. **Corrige dans ce gate**
(commit `fix(sprint76)` `52e70d1`, cf. §6) -> condition levee, **S77 Phase A
debloque**.

---

## 4. Une section par track (A-J)

### Track A — Suites verification + test-count deltas — PASS

Re-run autoritaire de l'orchestrateur (Windows natif, sequentiel) :

| Suite | Observe | Attendu |
|---|---|---|
| Rust nextest `--workspace` | **1804/1804** 0 skip | 1804 |
| `cargo fmt --all --check` | **0** | fmt 0 (fix `http.rs:8531`) |
| `clippy --all-targets -D warnings` | **0** | 0 |
| Doctests | OK (6 core_rs, 1 ignored worker) | OK |
| Vitest web | **398/398** (37 files) | 398 |
| Coverage | **87.2 / 79.01 / 85.92 / 88.52** | >= seuils (valeurs exactes) |
| scan-en-strings | clean | clean |

Comptage STATIQUE des attributs `#[test]`/`#[tokio::test]` ajoutes par phase
(regex elargie pour les formes multi-lignes) = **A4 / B8 / C10 / D4 / E10 / F5 /
G0 = 41** = exactement +41 annonce (1763->1804). 0 attribut supprime (added=net),
chaque attribut mappe a une `fn` reelle nommee. `git diff 8b53c38..5cb3c72 --
Cargo.lock` = **0 ligne** (0 drift dependance). Phase G = reformatage
whitespace/line-wrap d'un `credit(...)` de test Phase E (0 net legitime). Vitest
in-sprint +19, +12 hotfixes off-sprint = +31 (367->398). **Docker canonique 1808
(+4 `#[cfg(unix)]`)** : NON re-execute ce gate (env WSL-wedge), self-reporte
S76-G, structurellement coherent (0 `cfg(unix)` ajoute en S76).
*P3 mineur : baseline Vitest presentee 367 (CLAUDE.md) vs 379 (table
verification.md) — reconcilie (+12 hotfixes), note de bas de table souhaitable.*

### Track B — Phase A offer-my-power + worker co-localise — PASS

Snapshot additif 0-bump (`SCHEMA_VERSION: u32 = 1` inchange,
`skip_serializing_if = Option::is_none`, test `consent_snapshot_serializes_additively`).
Pas d'escalade : `user_public_consent` (local_worker.rs:367-380) copie level+caps
SEULEMENT, jamais `own_node_id` ; renvoie None hors OpenSource/All ; fail-closed
(fichier absent/corrompu -> floor least-privilege). Fix prod route
`/api/v1/consent*` byte-for-byte client front. `CONSENT_LEVEL` named-const.
- **P2 (route-S77)** : le « floor own-doc » survit en DONNEE mais son EFFET est
  mort a OpenSource(L2)/All(L4) — `should_accept_task` ne consulte
  `allowed_project_ids` qu'au niveau Whitelist ; un user en L2 voit son propre
  doc PRIVE (`is_open_source=false`) rejete par son worker co-localise.
  Doc-comment imprecis, 0 test sur ce scenario. Decision PO (clarifier doc OU
  OR-floor inconditionnel).
- **P3 (log-debt)** : `Engine::consent_snapshot()` (helper runtime) sans test
  direct (les 2 tests construisent le snapshot a la main). Champ d'affichage
  non load-bearing.

### Track C — Phase B duress siblings + carries 2-reports — PASS

Les 3 carries 2-reports REELLEMENT fermes : **CARRY-3** (downgrade
`trustworthy_open_source` re-applique a l'INGRESS avant `add_direct_entry`),
**LOOPBACK-TIERS** (7 routes inscrites §3), **PULL-3** (`build_seed_fetch_chain`
chaine ordonnee, `delete_tag` avant tier suivant, codes preserves). Duress no-op
(`seed_voluntary`/`set_keep_online` early-return avant mutation, tests etat
negatif 0 row 0 tag).
- **P2 (route-S77)** : **B10** parite bridge-allowlist = 4 miroirs hand-maintenus
  (2 Rust + 2 TS), pas de source unique ; derive cross-langage possible si les 2
  cotes divergent simultanement (chaque cote est test-locke, mais pas depuis une
  fixture partagee).
- **P2 (route-S77)** : **B3** le tier `directory` (`directory_snapshot` deep-clone
  + `directory_pull_providers`) est resolu EAGER avant `build_seed_fetch_chain`,
  donc execute meme sur le happy-path ticket-tier-1. Cout RAM borne par requete
  `/seed` (pas de dial reseau).

### Track D — Phase C E2E cross-machine + cohorte homogene — PASS

Cohorte **advisory** au claim-gate worker (`continue` SANS emettre de ClaimEntry
sur tuple mismatch, gate APRES `verify_signature()`, la TaskEntry reste live pour
un pair homogene). Le dispatcher pose `required_runtime` SEULEMENT si
`verifiable && redundancy>1`. `model_digest = blake3(name)` = **champ MORT**
honnete (`Verifier::verify` 0 appelant prod, grep exhaustif). **Candidat P1 #3
REFUTE** : aucun chemin d'acceptation/scoring de resultat ne lit
`required_runtime`/`RuntimeTuple`/`model_digest` (grep validator/result_sync/
bridge/kudos = fixtures None seulement). Gate anti-regression
`e2e_network_execute_gate_real_http_no_frontier_mock` intact.
- **P2 (fix-now, CORRIGE)** : ancre harness/ledger vers test inexistant
  (cf. §6).
- **P3 (route-S77)** : `quant` Ollama = `""` rend la dimension quantization
  non-discriminante (wildcard-sur-vide) — 2 workers Ollama de quants differents
  consideres homogenes ; le gate cohorte ne protege que `(model, runtime_family)`.
  Scope cut assume (vrai quant = S77 backend file-exposing) ; le quorum
  `result_text` rattrape toute divergence.

### Track E — Phase D quorum redundancy>1 + fix bridge — PASS

Fix `forward_result_entry` (result_sync.rs:130-132) : dedup
`{hex(worker_pubkey)}:{task_id}` miroir EXACT du validator (validator.rs:115) +
DB `UNIQUE(task_id, worker_id)` (db.rs:126). Avant : dedup `task_id` seul -> 2e
worker jete au bridge avant le validator -> quorum cross-machine JAMAIS forme.
**Candidat P1 #1 REFUTE** : les 2 tests hermetiques (result_sync.rs:571/663)
construisent un VRAI node iroh, ecrivent 2 entrees sous 2 auteurs distincts, puis
spawnent le VRAI `validator_loop::run` (pas un mock) -> majorite stricte ;
red-before-green structurellement valide (rebasculer la cle a `task_id` seul
collapse les 2 votes -> AwaitingQuorum -> timeout). **Validator INCHANGE**
(diff tests-only). `verifiable_seed = u32::from_le_bytes(blake3(task_id)[..4])`
cable et teste. 3-noeuds E2E retire (timeout shared-process) -> quorum prouve PAR
COMPOSITION (couvre un chemin reel).
- **P2 (route-S77, deja P3-D-3)** : branche send-failure un-mark (`seen.remove`
  sur `tx.send` Err) non testee — les 2 tests spawn un validator_loop vivant donc
  `tx.send` toujours Ok. Symetrie de cle insert/remove verifiable par lecture
  (pas de blocage permanent).
- **P3 (route-S77)** : kudos quorum credite uniquement le worker declencheur
  (2e submitter byte-identique), pas tous les workers en accord. Pre-existant,
  hors scope Phase D (chemin non touche).

### Track F — Phase E dashboard contributeur + sanity-bound — PASS

`get_contributor_summary` reutilise `effective_score()` EMA EXACT
(`KUDOS_EMA_ALPHA=0.97`) ; index `idx_kudos_worker` PRE-EXISTANT 0-M20 (EXPLAIN
QUERY PLAN). **P1 `generation_time_ms` FIX A LA ROOT** : worker mesure
`Instant::now()` autour de `generate()` (runtime.rs:1110-1111), hardcode 0
supprime. **Candidat P1 #2 REFUTE** par tracage exhaustif : unique producteur
prod = worker ; uniques consommateurs prod de `credit()` = validator_loop.rs +
http.rs `/result`, tous deux passant `entry.payload.generation_time_ms` ; tous
les autres sites hardcodes sous `mod tests`. E2E `dispatch_loop.rs:312` assert
`generation_time_ms >= 1` via le chemin worker reel + signature Ed25519.
`sanity_bounded_tokens` clamp AVANT `log_utility` ; `raw_kudos` absent (champ mort
retire) ; GPU-heures LOCALES ; route dans `authed_routes`.
- **P2 (route-S77)** : sanity-bound asymetrique — un worker solo qui forge
  `tokens_generated` ET `generation_time_ms` coherents (meme payload signe, hors
  quorum) contourne le clamp. Frontiere de confiance assumee (plausibility-check,
  pas anti-Sybil), documentee THREAT_MODEL §15.3 sev M + PATTERNS §P61.
- **P3 (route-S77)** : `generation_time_ms` sans plafond superieur (le ceiling
  croit lineairement ; `log_utility` compresse l'incitation a <10x).
- **P3 (log-debt)** : `tasks_served` — unicite garantie en amont
  (`UNIQUE(task_id, worker_id)` + status-guard), pas par la table kudos ; pas de
  test d'idempotence de `credit()` local.

### Track G — Phase F quantization 4-bit doc-only — PASS

Backend **INCHANGE** (`git diff llama_cpp.rs` VIDE, `with_split_mode`/
`with_devices` = 0 ; seul `with_n_gpu_layers` cable). Test garde
`llama_cpp_unchanged_doc_only` bidirectionnel (assert presence + absence),
NON feature-gated (lit le source via `std::fs`) -> non-skippable. Doc honnete sur
les caps VRAM (lit `estimated_vram_mb` DECLARE, inerte si 0). 0 surpromesse
anti-Sybil (framing « meme GGUF = exactitude, PAS barriere de securite »).
Pointeur front genuinement non-cliquable (`<p>`, pas de lien mort 404).
- **2 P3 (log-debt)** : table VRAM §3 — cellules `<70B` (IQ4_XS/Q2_K) extrapolees
  presentees avec chiffres precis, avertissement seulement en note ; ligne 55
  melange contrainte 16Go et remarque 24Go hors-scope. Editorial ; le 70B mesure
  (frontiere decisionnelle) est exact, aucune decision GO/NO-GO affectee.

### Track H — Phase G wrap-up + fmt-fix + acceptance — CONCERN (leve par §6)

`fmt-fix http.rs:8531` = **vraie cause** : Phase E (`768e235`) a porte la ligne a
103 chars (> max_width 100, seul `credit(...)` au-dela) ; le fix est le wrapping
rustfmt standard. Faux-diagnostic « derive 1.95/1.94 » des suites D/E/F RETRACTE
explicitement (verification.md:176-178). Acceptance LIVE honnete (#26/#30 DIFFERE,
bilan 36/38, jamais 38/38). Harness palier 2 runnable (`REDUNDANCY` cable a
`redundancy_factor` + `verifiable:true` ssi >=2 ; le P1 review « verifiable
manquant -> cohort gate saute » est fixe). Docs longue-vie coherentes
(THREAT_MODEL v9, PATTERNS §P62/P38, SPRINT_LOG 1 row S76, 0 row STRIDE dupliquee).
- **P2 (fix-now, CORRIGE)** : `verification.md` §6 rows #27/#28 (+ #24) citent
  des noms de tests inexistants (cf. §6). **CONCERN -> leve** par `52e70d1`.
- **P3 (log-debt)** : inexactitudes mineures du commit body G (immuable) :
  `REDUNDANCY=""` defaute a 1 (pas rejete, bras `''` du `case` = code mort via
  `${REDUNDANCY:-1}`) ; `git diff -w http.rs` pas strictement vide (virgule de
  fin rustfmt, semantiquement inerte).

### Track I — Wire 0-bump + pre-launch policy — PASS

0 bump sur 25 constantes `*_VERSION` (`git diff` 0 +/- sur les definitions).
`canonical.rs` 0 changement (18 `DOMAIN_` intacts, 0 nouveau). `RuntimeTuple` +
`ConsentSnapshot` strictement additifs `#[serde(default)]`. 0 zombie legacy-decode
(le seul test version-bearing est forward runtime-tolerance, permis pre-launch).
- **P3 (route-S77)** : `required_runtime: None` serialise en
  `"required_runtime":null` dans le corps JCS signe (pas de `skip_serializing_if`)
  -> toute Task post-S76, meme sans cohorte, produit des bytes/signature
  differents de la Task pre-S76 byte-equivalente. **Permis par la pre-launch
  policy** (canonical editable avant v1.0, 0 noeud live) ; coherent avec
  `verifiable`/`watermark_seed`. A trancher au gel v1.0 (`skip_serializing_if`
  vs null assume).

### Track J — Meta-process — PASS

7/7 phase reviews `## Verdict: PASS` (E/F/G ont surface des CONCERN root-fixes,
pas des rubber-stamps : la review E a attrape le vrai `generation_time_ms: 0`
hardcode prod ; la review G a attrape le P1 harness palier-2 sans `verifiable`).
7/7 `*_codex_review.md` BRUTS authentiques (verdicts par-livrable, file:line,
disclaimers « Docker inaccessible » / « Tests non executes » qu'un re-write Claude
n'inventerait pas). 7/7 preflights avec verdicts. Superviseur proprement supprime
(`42c7448` : settings.json 3 events SessionStart/PreToolUse/PostToolUse, 0
registration ; 4 hooks survivants pointent des fichiers existants). Bootstrap
markers `<!-- BOOTSTRAP:BEGIN/END -->` greppables (README.md:1918/2347). Commit
bodies 9 sections (`## Scope cuts` exact) + deltas cumules.
- **2 P3 (log-debt)** : reference superviseur perimee `.claude/skills/nexus-phase-review/SKILL.md:459`
  (« committable apres supervisor ») apres `42c7448` ; auto-diagnostic fmt
  « FAUX POSITIF derive toolchain » du commit body F (`a547de6`, immuable) retracte
  honnetement en Phase G.

---

## 5. Findings list sorted by severity

| # | Sev | Track | Finding | Disposition |
|---|---|---|---|---|
| — | **P0** | — | (aucun) | — |
| — | **P1** | — | (aucun ; 4 candidats §6 tous REFUTES) | — |
| 1 | P2 | D/E/H | `verification.md` §6 cite 3 noms de tests inexistants, propages dans `sprint77_audit_plan.md` + `b3_live_pc_vps.sh:112` | **fix-now -> CORRIGE `52e70d1`** |
| 2 | P2 | B | floor own-doc effet mort a L2/L4 (doc-comment imprecis, 0 test) | route-S77 |
| 3 | P2 | C | B10 parite bridge-allowlist 4 miroirs hand-maintenus (pas de fixture partagee) | route-S77 |
| 4 | P2 | C | B3 tier directory resolu EAGER sur happy-path (cout RAM borne) | route-S77 |
| 5 | P2 | F | sanity-bound asymetrique (forge coherente tokens+gen_time, hors quorum) | route-S77 (deja MEDIAN-DE-GROUPE) |
| 6 | P2 | E | branche send-failure un-mark non testee | route-S77 (= P3-D-3) |
| 7 | P3 | I | `required_runtime:null` redefinit le canonical v1 (permis pre-launch) | route-S77 |
| 8 | P3 | D | `quant` Ollama="" non-discriminant (rattrape par quorum result_text) | route-S77 |
| 9 | P3 | F | `generation_time_ms` sans plafond superieur | route-S77 |
| 10 | P3 | E | kudos quorum credite seulement le worker declencheur (pre-existant) | route-S77 |
| 11 | P3 | B | `Engine::consent_snapshot()` helper sans test direct | log-debt |
| 12 | P3 | F | `tasks_served` idempotence garantie amont, non testee localement | log-debt |
| 13 | P3 | G | table VRAM cellules `<70B` extrapolees + ligne 55 ambigue | log-debt |
| 14 | P3 | J | reference superviseur perimee `SKILL.md:459` | log-debt |
| 15 | P3 | H | inexactitudes commit body G (REDUNDANCY="" / `git diff -w`) | log-debt |
| 16 | P3 | A | baseline Vitest 367 vs 379 (reconcilie, note de table souhaitable) | log-debt |
| 17 | P3 | J | auto-diagnostic fmt « faux positif » body F retracte honnetement | no-action (trace) |

**Total : 0 P0, 0 P1, 6 P2, 11 P3.** Rigor G4 satisfait (>= 1 P2+ confirme,
pas de sur-severisation).

---

## 6. Commits fix attendus (prealables au kickoff S77)

### `fix(sprint76): repointer les ancres de tests fantomes vers les fonctions reelles` — `52e70d1` (APPLIQUE)

Finding #1 (fix-now). Trois ancres d'evidence citaient des noms de tests sans
aucune fonction `#[test]` correspondante (derive plan->ledger non grep-verifiee,
post-reconciliation Codex Phase D ou le test 3-noeuds initial a ete retire au
profit de 2 tests hermetiques aux noms differents). Repoint :

- `dispatcher_routes_replicas_to_homogeneous_cohort` ->
  `cohort_gate_admits_homogeneous_worker`/`cohort_gate_blocks_non_homogeneous_worker`
  (runtime.rs:1950/2005) + `submit_sets_required_runtime_only_for_verifiable_redundant`
  (dispatcher.rs:212) ;
- `quorum_redundancy_two_stubworkers_byte_identical` ->
  `quorum_redundancy_two_workers_reach_validator` (result_sync.rs:571) ;
- `quorum_diverging_outputs_rejected` ->
  `quorum_redundancy_diverging_outputs_rejected` (result_sync.rs:663).

Fichiers : `sprint76_verification.md` (§6 rows #24/#27/#28 + note Codex D2 §5),
`sprint77_audit_plan.md` (Track D l.191 + Track E l.213-214),
`b3_live_pc_vps.sh:112`. 0 changement code/test (doc-integrite). `grep` residu =
0 ; `bash -n` clean. **Couverture fonctionnelle toujours verte** (les tests
survivants couvrent la propriete — ce n'etait PAS un faux-vert de comportement).

**Aucun autre fix prealable.** Tous les autres findings sont des P2/P3 a
router/logger (§7, §8) — non bloquants pour l'ouverture S77.

---

## 7. P2 a logger / router vers S77 (tech debt, sans code change ce gate)

A fusionner au kickoff S77 (G6 carry-over) avec les carries deja inscrits dans
`sprint77_audit_plan.md §3` :

- **OWN-DOC-FLOOR-L2L4** (Track B, P2 NOUVEAU) : decision PO — clarifier le
  doc-comment (« le floor n'a d'effet qu'aux niveaux <= Whitelist ») OU OR-floor
  inconditionnel si le produit veut garantir que l'user sert toujours son propre
  doc. 0 test couvre le scenario own-doc prive rejete a L2.
- **B10-PARITE-FIXTURE** (Track C, P2 — deja `sprint77_audit_plan.md §3`,
  DOC-P2) : fixture partagee unique pour l'allowlist bridge si une phase S77
  touche le bridge (4 miroirs hand-maintenus aujourd'hui).
- **DIRECTORY-EAGER-HAPPY-PATH** (Track C, P2 NOUVEAU) : rendre la resolution du
  tier `directory` paresseuse (post tier-1) plutot qu'eager. Cout RAM borne par
  requete, non bloquant.
- **SANITY-BOUND-ASYMETRIQUE / MEDIAN-DE-GROUPE** (Track F, P2 — deja
  `sprint77_audit_plan.md §3`, DOC-P2) : frontiere assumee ; durcir via median du
  groupe d'accord OU attestation runtime si priorise post-launch.
- **P3-D-3 SEND-FAILURE-UNMARK** (Track E, P2/P3 — deja `sprint77_audit_plan.md
  §3`) : test ciblant le chemin recepteur-droppe si une phase S77 touche
  `result_sync`.

**Carries deja inscrits `sprint77_audit_plan.md §3`, RECONFIRMES non-traites** :
SYBIL-SEEDER-TAIL (2/3, exemption « sharding »), REVISION-HOME-DURABILITY (2/3),
KNOWN-ENTRY-OVERCOUNT (2/3), seeder `catalog_len:0` (2/3, arbitrage PO design),
RE-DRIVE-ON-INGEST (2/3), T-NN+3 canonical_bytes dup JCS (open S70).
**Externes inchanges** (< 3 reports, exemptions tenues) : P2-A-1 rand,
P2-AUDIT-2 iroh, T-NN+2 iframe Rust-wasm, P3-OS-1. LT-2 ARME (dry-run prive fait,
flip = decision PO). Aucun n'atteint 3 reports sans exemption.

**Invariant de cloture NOUVEAU a inscrire** : « chaque nom de test cite dans
`verification.md` §6 doit grep-resoudre a une fonction `#[test]` avant cloture »
(un review/Codex qui s'appuie sur les noms l'aurait detecte ; la cause-racine
ici = recopie plan->ledger sans grep-verification).

---

## 8. P3 laisses sans action (nits)

Findings #7-#17 du tableau §5 : redefinition canonical permis pre-launch (a
trancher au gel v1.0), `quant` Ollama="" scope cut documente, pas de plafond
`generation_time_ms` (log_utility absorbe), kudos crediting fairness pre-existant,
helpers/idempotence non testes localement, editorial table VRAM, reference
superviseur perimee SKILL.md:459, inexactitudes commit body G immuable, baseline
Vitest reconcilie, auto-diagnostic fmt retracte. Aucun ne bloque S77 ; les
`log-debt` peuvent etre absorbes opportunistement par une phase S77 touchant le
fichier concerne.

---

## 9. Notes on audit completeness

- **Couverture** : 10 tracks A-J, chacun par un agent independant lecture-statique
  (no-compile, regle anti-OOM), + verification adversariale de chaque finding
  P0/P1/P2 (1 skeptic/finding, mandate REFUTER) + synthese. Les 4 candidats P1 du
  plan tranches explicitement (tous NON-P1, code-first).
- **Suite autoritaire RE-JOUEE** (pas seulement relue) : Rust nextest Win
  1804/1804 + fmt 0 + clippy 0 + doctests + Vitest 398/398 + coverage + scan, tous
  verts (§4 Track A). C'est une re-verification reelle, exactement aux compteurs
  S76.
- **NON couvert (env-bound, posture auditee a la place de la trace)** :
  - **Acceptance LIVE cross-machine** (#26 B-3 palier 1, #30 quorum palier 2) :
    DIFFERE-materiel-operateur (PC RTX 5080 + VPS Hetzner + Mac + SSH/WAN absents
    de l'environnement de session, precedent S74). L'audit a verifie la **posture
    honnete** (DIFFERE marque, jamais 38/38, critere <30s=BLOCK encode) + le
    **harness runnable** (`b3_live_pc_vps.sh` `REDUNDANCY` cable, `bash -n` clean),
    PAS la trace live. A consigner par l'operateur s'il execute les runs.
  - **Docker Linux canonique 1808** : NON re-execute ce gate (risque WSL-wedge +
    OOM linker si concurrent). Self-reporte S76-G comme gate-avant-push ; +4 vs
    Win = `#[cfg(unix)]` structurels stables (0 `cfg(unix)` ajoute S76). Re-run
    dual-platform a la discretion operateur avant tout push.
- **Correction in-gate** : le seul defaut bloquant-doc (3 noms de tests fantomes)
  a ete corrige (`52e70d1`) plutot que simplement consigne, pour ne pas leguer des
  pointeurs morts au gate S77.
- **Exit Gate** : `sprint76_audit_findings.md` porte un verdict avec 6 P2 + 11 P3
  (>= 1 P2+ G4), couvre A-J, ingere le diff complet S76, confirme les invariants
  headline (fix bridge ferme le trou quorum ; validator inchange ; P1 gen_time_ms
  root-fixe ; fmt root-fixe ; backend quant inchange ; 0 bump wire), et tranche
  les 4 candidats P1 (tous NON-P1). **S77 kickoff (sharding pipeline) debloque.**
