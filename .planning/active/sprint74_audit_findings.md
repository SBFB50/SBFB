# Sprint 74 — Audit findings (audit gate joue en S75 Phase 0)

## 1. Auditeur

Session Claude Code fraiche (S75 Phase 0, Cas A). L'agent `nexus-audit-gate`
n'etant pas enregistre comme `subagent_type`, fallback README §3 = audit joue
comme **workflow multi-agents anti-anchoring** :

- **9 tracks paralleles** a contexte frais (seed-sec, fork-sec, wire, db-quorum,
  pull-deploy, web, scope, meta-process, carry), chacun lit le CODE d'abord et
  traite `verification.md` / `*_review.md` / `*_codex_review.md` comme des
  **claims a challenger**, jamais comme verite (anti-anchoring).
- **Skeptics adversariaux refute-by-default** (2 lentilles : trigger reel +
  correctness) sur CHAQUE candidat P0/P1, pour eliminer les faux positifs.
- **Synthese** dedup + verdict.
- **Re-verification independante main-thread** des findings frolant le P1
  (WEB-1, META-1, CARRY-1) directement sur le code/git.

Cout : 10 agents, ~1.39M tokens, 355 tool-uses, ~26 min. Plus build/test health
empirique main-thread (Windows natif + web), en parallele.

## 2. Tip audite

- Range S74 : `457ca05^..bede850` (base `6acf638` = parent de Phase A ; 7 phases
  A-G : A `457ca05` / B `bcfc155` / C `9c2bd68` / D `4c1acc5` / E `b76a084` /
  F `66a9409` / G `bede850`).
- HEAD master a l'audit : `9b034c1` (`master...origin/master` propre, 0 ahead,
  tout pousse). 2 hotfixes Cas D post-G deja pousses (`6ca9702` self-heal
  storage/feed namespace + `43215f7` gate worker `PR_SET_PDEATHSIG` Linux) —
  hors range S74 (Cas D, non-phases) mais glances inclus.

## 3. Verdict global : **PASS**

**0 P0, 0 P1, 15 P2, 10 P3.** Tous les findings ont survecu a la refutation
adversariale (status CONFIRMED) ; aucun n'atteint la severite bloquante. Le
signal de rigueur G4 est satisfait (>= 1 P2+ documente, l'audit a trouve du
reel). Conformement a README §3.5 : **PASS = 0 P0 ET 0 P1 → S75 Phase A demarre
directement, aucun commit `fix(sprint74)` requis.**

Les 5 verrous anti-recentralisation tiennent (verifie dans le code) : seed
protocol, atelier fork, pin M18, et raw-op SeedAnnounced ; `seeder != auteur`
(la provenance reste signee de l'auteur quel que soit le seeder) ; le compteur
seed best-effort ne sert JAMAIS de verite de joignabilite (content-addressing
BLAKE3 + sonde live restent l'autorite). B.2 arithmetique quorum saine (pas
d'off-by-one terminant tot ni de zombie AwaitingQuorum). Aucune injection
git-arg (https-only + 40-hex sha + `--end-of-options`), zip-slip (`:`/ADS/symlink
rejetes), zip-bomb (caps compresse+decompresse), body-limit axum cap. Aucun bump
wire injustifie (`FEED_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION` restent 1 ;
M18/M19 LOCAUX ; SeedAnnounced raw-op TYPEE forward-compatible).

## 4. Build / test health empirique (clot le gap dual-platform differe par S74)

Le `verification.md` S74 admet honnetement avoir **differe** la suite
iroh-networked + Docker canonique (env bloque la session S74 : WSL wedge →
Docker engine 500 + reseau hote degrade → tests `create_node` hang). **Cet env
est RECUPERE** (machine rebootee depuis ; `docker ps` OK). Re-verification cette
session :

| Suite | Resultat |
|---|---|
| Rust Windows natif | `cargo fmt --all --check` 0 · `clippy --workspace --all-targets --locked -- -D warnings` 0 · **`nextest run --workspace --locked` 0 echec** (suite iroh-networked INCLUSE, repasse) · doctests 0. Zero token `FAIL[`/`TIMEOUT`/`LEAK`/`panicked` dans 392 Ko de log. |
| Web | `tsc` 0 · `lint` 0 (5 warnings cosmetiques fast-refresh) · **Vitest 331/331** · coverage **86.91 / 78.63 / 85.82 / 88.23** (≥ 85/85/78/85) · build OK · **size 6/6** · scan FR clean. |

→ Le self-report S74 (Windows + web) est **confirme empiriquement**. Le compteur
exact nextest n'est pas extractible proprement du log (encodage mixte UTF-8/UTF-16
du `*>>` PowerShell) mais exit-0 sur `--workspace` est autoritatif (nextest renvoie
non-zero des le 1er echec/timeout/leak). Docker Linux canonique (`rust:1.94` /
`sbfb-ci:latest`) reste le gate **avant PUSH** uniquement (pas ce tour — on ne
pousse rien ; [[feedback_wsl_before_push]]).

## 5. Verdict par track

| Track | Verdict | Findings |
|---|---|---|
| T-SEED-SEC (protocole seed cross-noeud) | CONCERN | SEED-1 P2, SEED-2 P2, SEED-3 P3 |
| T-FORK-SEC (atelier fork, contenu non fiable) | PASS | FORK-1 P2, FORK-2/3/4 P3 |
| T-WIRE (wire-format / pre-launch) | PASS | WIRE-1/2/3 P2 |
| T-DB-QUORUM (db M18/M19 + quorum B.2 + keep_online) | PASS | DBQ-1 P2 |
| T-PULL-DEPLOY (blob pull/fetch/pin/deploy) | CONCERN | PULL-1/2 P2, PULL-3 P3 |
| T-WEB (surface web A+G) | CONCERN | WEB-1 P2, WEB-2/3 P3 |
| T-SCOPE (scope cuts + tracabilite) | PASS | SCOPE-2 P3 |
| T-META (process A-G : body 9 sections, Codex, delta tests, G1) | PASS | META-1 P2, META-2 P3 |
| T-CARRY (carries herites S73 + re-routes S74) | CONCERN | CARRY-1/2/3/5 P2, CARRY-4 P3 |

Les CONCERN sont tous de la dette d'etat-d'affichage best-effort + hygiene-doc,
aucun bloquant. Aucune track FAIL.

## 6. Findings tries par severite

### P0 — aucun.
### P1 — aucun.

### P2 (15 — a logger en tech debt, absorbes par le plan S75)

| ID | Track | Titre | Evidence | Fix |
|---|---|---|---|---|
| SEED-1 | seed | TTL SeedRegistry battu par un `ts` SeedAnnounced date dans le futur | `feed_sync.rs:382-387` passe le feed-ts comme `seen_at` ; `seed_registry.rs:91` le stocke verbatim ; seul borne `public_feed.rs:622` 30j. Over-count display-only. | Clamp `seen_at` a `min(feed_ts, horloge_reception)`. |
| SEED-2 | seed | Nonce cache borne TTL seulement, pas en taille | `seed_protocol.rs:83-102` aucune borne max ; nonce enregistre AVANT le check invite. Borne par handshake+TTL+signature. | Cap la taille OU enregistrer apres le check invite. |
| FORK-1 | fork | `extract_zip` cap les octets mais pas le nombre d'entrees | `fork.rs:198-256` byte-caps seulement, pas d'entry-cap. Trigger operateur local. | Plafond `max_archive_entries`. |
| WIRE-1 | wire | Ops feed `ReleasePublished` invisibles a la recherche par nom (= carry FRESHNESS-RELEASE-UNINDEXED) | `public_feed.rs:31-40` pas de project_name ; `search.rs:223-265` index vide. Self-publish searchable, le feed-op non. Reconstructible. | `project_name`/`category` Option `#[serde(default, skip_serializing_if)]` (0-bump) + test. |
| WIRE-2 | wire | Compteur SeedAnnounced jette l'archive_hash | `seed_registry.rs:77` drop archive_hash. Over-count best-effort, ne sert jamais d'octets. | Documenter la granularite OU keyer sur (project_id, archive_hash). |
| WIRE-3 | wire | Reprovide par-boot fait croitre le feed monotone | `feed_sync.rs:160-200` emet par-row par-boot, ts frais → hash distinct, dedup ne coalesce jamais. Scope cut #5 (pilote-borne, modele IPFS reprovide). | Logger comme propriete pre-launch + compaction post-launch. |
| DBQ-1 | db | `set_keep_online` peut nuller un archive_hash connu sur la race boot | `db.rs:700-704` reecrit la row ; `http.rs:1068-1071` lit l'aggregator volatile vide post-boot → row nullee stoppe la re-annonce. Joignabilite non affectee (= carry KEEP-ONLINE-HASH-SOT). | Coalesce l'archive_hash au toggle + lier au GC reaper. |
| CARRY-2 | carry | Guardrail-trip-apres-Accept laisse la tache non-terminale | `validator.rs:143` retourne Accepted non-terminal ; branches trip `http.rs:1969` + `validator_loop.rs:82` retournent sans Rejected. **Securite correcte** (guardrail bloque toujours le persist), residu = zombie d'etat. | Set Rejected dans les 2 branches trip + test. |
| CARRY-3 | carry | B.6 sanitise seulement a l'index recherche, pas a l'aggregator browse | `runtime.rs:1779` stocke is_open_source verbatim ; `:1785` sanitise une struct separee ; `:1787` stocke l'entry non-sanitisee. Latent. | Downgrade a l'ingress aggregator + corriger le commentaire. |
| CARRY-5 | carry | Handler search clamp `limit` mais pas `offset` ni longueur `q` | `http.rs:2443` clamp limit ; `:2447` passe offset+q non bornes ; `sanitize_query:36-56` pas de cap. Loopback single-user. | Clamp offset + cap longueur q. |
| PULL-1 | pull | Fichier provenance duplique dans l'archive deployee | `deploy.rs:420-425` append sans retirer l'existant ; reconstruction fork en porte un. PAS un break lock-4, bloat stale. | Stripper la provenance existante avant re-append. |
| PULL-2 | pull | Fetch ne dial qu'un seul provider du ticket | `blobs.rs:179-190` single endpoint ; `http.rs:1157-1166` ticket-only ; endpoints registry inutilises. Scope cut #4 deferred non-commente. | Doc-comment le single-provider dial comme le lift failover (= cœur resilience pull S75). |
| WEB-1 | web | Toggle keep-online faux-ON a la reouverture, carry mal-etiquete CLOSED | `AvailabilitySheet.tsx:111` `useState(true)` jamais reconcilie avec `selfSeeding` fetche `:106` ; seul le demi-fix is_own a ship, pourtant logge CLOSED. Display-only single-user. | Seed le toggle depuis `selfSeeding`, re-ouvrir le carry, corriger le claim CLOSED. |
| META-1 | meta | Phase D committee avec un verdict Codex GAP non resolu | `sprint74_phase_d_codex_review.md:39` verdict overall GAP ; le commit DISCLOSE les 2 (GAP#1 ownership → ferme Phase G via is_own ; GAP#2 hash-SOT → re-route S75). Carry DISCLOSE, pas bypass silencieux. | Logger une regle PATTERNS : un GAP Codex au commit doit etre un carry explicitement disclose+tracke. |
| CARRY-1 | carry | Statut LT-2 perime : master sync et tag v1.0 POUSSE sur origin | `git ls-remote --tags origin v1.0` → `refs/tags/v1.0` present ; `origin/master..master` = 0. L'audit_plan dit « non pousse / 37 ahead » → PERIME. **Consequence : LT-2 Radicle sortie cap G7 trigger desormais ARME.** | Flipper LT-2 ARME dans les docs planning + memory ; programmer le dry-run Radicle prive ([[feedback_radicle_private]]). |

### P3 (10 — laisses sans action, optionnels)

| ID | Titre | Note |
|---|---|---|
| CARRY-4 | Cause racine flaky test mal-enoncee (`publish_returns_200`) | L'audit_plan blame un dial aggregate ; en realite l'aggregate self-short-circuit, la vraie cause = endpoint bind/relay. Fix = test-node relay-disabled. |
| FORK-2 | `extract_zip` laisse des fichiers partiels sur erreur zip-bomb | `fork.rs:246-253` pas de cleanup (vs l'arm clone). Cosmetique. |
| FORK-3 | Nom template non-echappe = self-XSS only | `template_engine.rs:155-159` plain replace ; name = arg CLI local ; iframe isole. Author-trusted. |
| FORK-4 | Clone deploy sans `--end-of-options`/`kill_on_drop` | `deploy.rs:771-811` vs `fork.rs:341/365` ; sha 40-hex valide donc injection bloquee. Backport parite. |
| META-2 | Commit Phase G prefixe `docs(` porte du code | Hook arme sur `docs` + « Phase » donc gate enforce ; pas un bypass. Preferer `fix`/`feat` pour un wrap-up porteur de code. |
| PULL-3 | `set_tag` sans presence-check au toggle/deploy | `http.rs:1092-1100` + `deploy.rs:456-461`. Inerte aujourd'hui, latent pour le reaper. |
| SCOPE-2 | §8 mappe « search/open/fork » comme si les 3 avaient ship | Seuls fork+redeploy existent ; search = shell browse ; open subsume par fork. Reformuler la row. |
| SEED-3 | Boot re-announce sans blob-presence check ; §8 omet 2 items | `feed_sync.rs:160-195` emet inconditionnel (benin) ; §8 omet Tantivy + tree-sitter non-livres. |
| WEB-2 | Etat « soutien volontaire » reset a la reouverture | `AvailabilitySheet.tsx:134` `useState(false)` ; re-clic idempotent. Deriver de selfSeeding. |
| WEB-3 | Branche erreur `SearchResultsView` sans test Browse-view | `Browse.tsx:206-216` correct mais non stubbe ; le throw schema-layer est teste dans daemon.test. |

## 7. Commits fix attendus

**AUCUN.** Verdict PASS (0 P0/P1). S75 Phase A demarre sans gate-fix.

## 8. P2 a logger en tech debt + routing S75

Les 15 P2 sont non-bloquants et logges ici. **Synergie forte avec le pivot
decouverte S75** (PULL node-centrique + ancre VPS) — a folder dans le kickoff
Cas C plutot que de les traiter en patches isoles :

- **Directement dans le scope du pivot decouverte** : WIRE-1 (indexer
  ReleasePublished par nom), WIRE-2 (granularite compteur seed), WIRE-3
  (croissance reprovide → compaction), SEED-1 (clamp ts registry), SEED-2 (cap
  nonce/registry), PULL-2 (multi-provider fallback = cœur resilience pull),
  CARRY-3 (sanitize aggregator byzantine), DBQ-1 (hash-SOT + GC reaper). Ces
  surfaces SONT le terrain du pivot — le kickoff S75 doit les concevoir, pas les
  rustiner.
- **Hygiene a traiter independamment (cheap, hors-pivot)** : CARRY-5 (clamp
  offset/q), CARRY-2 (Rejected terminal sur trip), PULL-1 (dedup provenance),
  FORK-1 (entry-cap), WEB-1 (seed toggle depuis selfSeeding).
- **Doc/process** : META-1 (regle PATTERNS GAP-carry), CARRY-1 (flipper LT-2
  ARME + dry-run Radicle).

A router dans `docs/rust/PATTERNS.md` / `docs/shell/PATTERNS.md` (tech debt
sections) et dans le `sprint75_plan.md` lors du kickoff Cas C.

## 9. P3 laisses sans action

Les 10 P3 ci-dessus sont des nits/cosmetiques/test-coverage, laisses tels quels
(optionnels). Plusieurs (FORK-4 parite, WEB-3 test branche) peuvent etre pris
opportunistement si une phase S75 touche la zone.

## 10. Notes on audit completeness

- **Couvert** : 9 tracks static-adversariaux sur le diff complet S74 +
  refutation skeptique de tout candidat P0/P1 + re-verification main-thread
  independante (git/code) de WEB-1/META-1/CARRY-1/B.2/SEED-1 + build/test health
  empirique Windows+web.
- **Correction sur le `verification.md`** : son §Env affirme l'env « bloque » et
  le full-workspace « differe ». Cet env est RECUPERE et le full-workspace
  Windows a ete re-joue VERT cette session (§4). La synthese workflow repete le
  claim perime « env bloque » (heritage du contexte fourni) — corrige ici par la
  mesure empirique.
- **Non re-derive independamment** (confiance dans le raisonnement static des
  tracks) : le flow seed 2-noeuds E2E live, l'enumeration exhaustive
  injection-fork/zip-slip au-dela du diff, les comptes de delta-tests exacts par
  phase, le re-run Codex. Aucun de ces axes n'a souleve de P0/P1 candidate.
- **Gate avant-push** : Docker Linux canonique + full iroh-networked E2E
  multi-machine restent gates AVANT PUSH (pas ce gate ; on ne pousse pas).
- **Verdict final : PASS** — aucun `fix(sprint74)` avant S75 Phase A ; les 25
  findings (15 P2 + 10 P3) sont des tolerances documentees, dont 8 P2 a concevoir
  dans le pivot decouverte S75.
