# Sprint 61 — Audit findings

**Auditeur** : session fraiche Claude Code (pre-Sprint 62).
**Date** : 2026-05-13.
**Audit plan suivi** : `sprint62_audit_plan.md` (8 dimensions).
**Tip master audite** : `72a7adf` (fix(feed): verification.md +
audit_plan).
**Fichiers inspectes** : `public_feed.rs`, `feed_materializer.rs`,
`db.rs` (migrations M9/M10 + feed methods), `canonical.rs`
(DOMAIN_FEED_V1), `kudos_ledger.rs` (pattern hash-chain),
`PUBLIC_FEED_SPEC.md`, `sprint61_kickoff.md`, `sprint61_verification.md`.

---

## §1 Verification independante des compteurs

| Suite | Verification.md | Observe audit | Match |
|---|---|---|---|
| Rust nextest registered | 1282 | 1282 | OK |
| Vitest | 258 pass | 212 pass / 46 fail (env) | env variance connue (vitest_env_variance.md) |
| size-limit | 6/6 | non re-execute (pas touche S61) | OK trust |

Note : les 46 Vitest en echec sont le meme groupe connu
(BrowsedProject 15 + Deploy 2 + projectStore 22 + CommandPalette 7)
— mock localStorage, pre-existant avant S61. Sprint 61 n'a touche
aucun fichier frontend. Pas une regression.

---

## §2 Findings par dimension

### 2.1 Spec vs Code alignment

**Verdict : ALIGNE.**

- 8/8 sections de PUBLIC_FEED_SPEC.md ont un equivalent code Rust.
- Test vector JSON (hash `f81ced7da512d...569ae2a`) match le test
  `test_compute_feed_entry_hash_deterministic` dans `public_feed.rs:358-362`. OK.
- `FEED_FORMAT_VERSION = 1` : spec §1 header + code `public_feed.rs:17`.
  Match. OK.
- `FeedEntryCanonical` struct : 5 champs spec (version, op,
  author_pubkey, timestamp, prev_hash) = 5 champs code
  (`public_feed.rs:86-92`). OK.
- `DOMAIN_FEED_V1 = b"nexus-feed-v1"` : spec §3 + `canonical.rs:199`.
  Match. OK.
- Tagged union `#[serde(tag = "op_type")]` coherent avec spec §2
  JSON examples. OK.

**P3-SPEC-PAYLOAD** : le kickoff D2 dit `payload BLOB NOT NULL --
JCS canonical bytes` mais M9 utilise `payload TEXT NOT NULL` et le
code stocke du JSON serde (`serde_json::to_string` dans
`public_feed.rs:189`), pas des canonical bytes JCS. Les canonical
bytes sont recalcules a la volee pour verification. C'est
architecturalement correct (le payload lisible aide au debug) mais
diverge du commentaire D2. Le design review D2 ⚠️ avait prevu cet
ajustement. Cosmetic.

### 2.2 Hash-chain integrity

**Verdict : CORRECT, coherent avec kudos_ledger.**

- `verify_chain()` (`public_feed.rs:246-291`) implemente les 4 etapes
  spec §4 : prev_hash link, entry_hash recomputation, Ed25519 verify.
  OK.
- Pattern identique a `kudos_ledger.rs` : BLAKE3 hash via
  `canonical_bytes()` + domain separation, genesis = `"genesis"`,
  hex encoding pour hashes. OK.
- Tests couvrent : chain tamper (`test_verify_chain_tampered_hash`),
  forged signature (`test_verify_chain_forged_signature`), empty
  chain (`test_verify_chain_empty`), genesis prev_hash
  (`test_hash_chain_genesis`), multi-entry valid
  (`test_hash_chain_valid_with_ed25519`). OK.
- Persist + reopen verify (`test_feed_persist_reopen_verify`). OK.

**P3-VERIFY-RETURN** : kudos_ledger.verify_chain() retourne
`Result<bool, _>` (false on tamper), public_feed.verify_chain()
retourne `Result<(), String>` (Err with detail message). Divergence
cosmetique de convention. Le pattern public_feed (Err + message
detaille) est strictement superieur pour le diagnostic. Pas
d'action requise.

### 2.3 Migration safety

**Verdict : SAFE.**

- M9 (`db.rs:153-165`) : table `public_feed` standalone, 0 FK vers
  tables existantes (M1-M8). Schema coherent TEXT types pour hashes
  (aligne kudos_ledger, pas BLOB comme draft D2). OK.
- M10 (`db.rs:167-173`) : table `feed_cursor` singleton
  (`CHECK (id = 1)`). Schema minimal 3 colonnes (id, last_seq,
  last_entry_hash). 0 donnee sensible si DB inspectee. OK.
- Index `idx_feed_created` sur `public_feed(created_at)` : actuellement
  aucune requete ne filtre par created_at. Premature mais inoffensif. P3.
- D2 ⚠️ resolution (BLOB → TEXT) : correcte, alignee avec le pattern
  kudos_ledger hex. Documentee dans kickoff §4 "Acknowledged review
  findings". OK.

### 2.4 Materializer correctness

**Verdict : CORRECT.**

- `test_cursor_restart_consistency` : materialize_incremental(None) ==
  materialize_full() == materialize_full() (restart). Equivalence
  prouvee. OK.
- `test_cursor_hash_mismatch_triggers_full_rebuild` : cursor hash
  invalide → verify_chain + full rebuild + cursor update. OK.
- `test_materialize_source_stale` : ReleasePublished → SourceBecameStale
  transitions correctement. OK.
- `test_source_stale_without_release` : SourceBecameStale sans
  ReleasePublished prealable → published=false, source_stale=true.
  OK (conforme spec §5 "The feed does NOT enforce this ordering").
- `test_incremental_no_existing_view_rebuilds_prefix` : incremental
  sans view existante reconstruit le prefix depuis DB. OK.
- `test_cursor_persist_reopen_file_db` : cursor survit au reopen. OK.

**P3-PERF-ENTRY-LOOKUP** : `entry_hash_at_seq()` (`feed_materializer.rs:164-174`)
charge toutes les entrees puis filtre lineairement — O(N) pour un
point lookup. `save_cursor_from_db()` (`feed_materializer.rs:207-216`)
fait pareil. Pas un probleme pour les volumes Sprint 1 (dizaines
d'entrees) mais a optimiser Sprint 2+ quand le feed grandira. P3
performance.

### 2.5 Post-v1.0 policy compliance

**Verdict : CONFORME Sprint 1, gap documente pour Sprint 2.**

- `FEED_FORMAT_VERSION = 1`. OK.
- `#[serde(default)]` sur `provenance_hash: Option<String>`
  (`public_feed.rs:35`). Seul champ optionnel, correctement annote. OK.
- Pas de tolerant decoder multi-version (`verify_chain()` ne gere
  qu'un seul format). Correct pour Sprint 1. OK.

**P2-VERSION-NOT-STORED** : la table `public_feed` (M9) ne contient
pas de colonne `version`. `replay_all()` (`public_feed.rs:229`)
hardcode `version: FEED_FORMAT_VERSION` pour chaque entree relue.
Le `FeedEntryCanonical` contient `version` (il est hashe/signe),
donc la hash inclut le numero de version. Si `FEED_FORMAT_VERSION`
est bumpe a 2, `replay_all()` reconstruira les anciennes entrees
avec `version=2` au lieu de `version=1`, les hashes ne matcheront
plus, et `verify_chain()` echouera.

Consequence : avant tout bump de `FEED_FORMAT_VERSION`, il faut
ajouter une colonne `version INTEGER NOT NULL DEFAULT 1` a la table
et adapter `replay_all()` pour lire la version stockee. Pas bloquant
Sprint 1 (version unique), mais carry **obligatoire** avant le
premier breaking change de format (probablement Sprint 4+).

### 2.6 Scope cuts respect

**Verdict : 12/12 respectes.**

| # | Scope cut | Grep code | Resultat |
|---|---|---|---|
| 1 | Sync P2P iroh-docs | grep `iroh_docs\|iroh-docs` dans public_feed/feed_materializer | 0 hit |
| 2 | Anti-spam feed | grep `pow\|rate.limit\|spam` dans public_feed/feed_materializer | 0 hit |
| 3 | CuratorVouched impl | grep `CuratorVouched` — present uniquement dans spec §2.2 + doc comments | 0 code actif |
| 4 | BuildQuorumReached impl | grep `BuildQuorumReached` — present uniquement dans spec §2.2 + doc comments | 0 code actif |
| 5 | HTTP endpoint verify | grep `verify.release\|verify_release` dans daemon/coordinator | 0 hit |
| 6 | Bridge provenance | grep `getProvenanceRecord\|verifyRelease` dans bridge | 0 hit |
| 7 | UI proof-chain | grep `VerificationDetail\|proof.chain` dans web/ | 0 hit |
| 8 | Tests adversariaux hardening | Phase D tests = correctness basique, pas hardening-level | OK |
| 9 | Go-live public | tag v1.0 local only, pas pousse | OK |
| 10 | AppImage Linux | grep `AppImage\|linuxdeploy` | 0 hit |
| 11 | Interop externe | 0 client alternatif | OK |
| 12 | Audit tiers formel | 0 RFP Trail of Bits | OK |

### 2.7 Carries review

**Verdict : CORRECT.**

| Item | Compteur kickoff | Compteur verification | Coherence |
|---|---|---|---|
| P2-A-1 rand blocker | 21+/3 | 22+/3 | OK (increment 1 sprint) |
| P2-AUDIT-2 iroh transitives | herite | herite | OK |
| P2-NSIS-UNINSTALL | 1/3 → 2/3 | 2/3 | OK — MANDATORY S63 si pas resolu S62 |
| P2-IMAGE-DEP | 1/3 → 2/3 | 2/3 | OK — MANDATORY S63 si pas resolu S62 |
| P2-G-1 exe lock | reouvert | reouvert | OK |
| P2-PLAYWRIGHT-REFACTOR | 1/3 → 2/3 | 2/3 | OK — MANDATORY S63 si pas resolu S62 |

3 items a 2/3 — aucun encore a 3/3 MANDATORY. Si aucun n'est
resolu en S62, les 3 deviennent MANDATORY S63 (Regle 2 §6.2.1).

### 2.8 Findings review externe GPT 5.5

Les 6 items de la review externe sont verifies :

**P2-IROH-INFRA-TIMEOUT** : documente dans verification.md §3.1.
Pre-existant (nexus-core-rs n'a eu qu'un ajout DOMAIN_FEED_V1 en
S61). Resolution proposee : gate SBFB_INTEGRATION ou slow-timeout
ajuste. Carry S62. OK.

**P2-INCREMENTAL-NO-VERIFY** : confirme. `materialize_incremental()`
quand le cursor matche (`feed_materializer.rs:122-141`) traite les
nouvelles entrees sans `verify_chain()`. Acceptable single-writer
local Sprint 1. **Re-carry obligatoire AVANT sync P2P Sprint 62**
— quand le feed recoit des entrees d'autres noeuds, chaque entree
doit etre verifiee individuellement ou par batch.

**P2-SPEC-TRUST-CONTRACT** : partiellement documente. Le docstring
de `materialize_full()` (`feed_materializer.rs:80`) dit "does NOT
verify... call verify_chain first if untrusted". Mais la spec
`PUBLIC_FEED_SPEC.md §5` ne mentionne pas explicitement le modele
trust local-only vs remote. P3 doc gap (la spec devra l'expliciter
Sprint 2 quand le sync P2P introduit du contenu untrusted).

**P2-VALIDATION-STRICTE** : confirme. `validate_feed_operation()`
(`public_feed.rs:134-149`) valide uniquement la coherence
is_open_source/provenance. Pas de validation format pour :
project_id, repo_url (URL valide?), commit_sha (hex 40?),
artifact_hash (hex 64?), reason (enum fermee dans spec §2.1 mais
string libre dans code). Acceptable single-writer Sprint 1 (les
valeurs viennent du coordinator trusted). Carry S62 pour sync P2P.

**P2-TRANSACTION-ATOMIQUE** : confirme. `insert_feed_operation()`
(`public_feed.rs:155-216`) fait `get_last_feed_entry_hash()` puis
`insert_feed_entry()` sans transaction SQLite explicite. Risque
theorique de race condition si 2 threads ecrivent simultanement
(double prev_hash). Pratiquement null Sprint 1 (single-writer
single-thread coordinator). Carry S62 (wrap dans BEGIN/COMMIT ou
mutex avant multi-source).

**P2-PLAN-DELTA** : Phase D prevoyait +4 tests, reel +2
(test_source_stale_without_release + test_cursor_restart_consistency).
Les 2 autres (test_chain_tamper_detect + test_signature_verify_reject_forged)
avaient ete pre-livres dans les fix commits inter-phases. Total
depasse la cible plan (1282 > 1274). Calibration plans futurs —
les fix commits inter-phases doivent etre comptabilises dans le
plan delta. P3 process.

---

## §3 G1 presence check

`sprint61_design_review.md` present dans `.planning/active/`. OK.
Scoring documente dans kickoff §4 "Acknowledged review findings" :
D1 ✅, D2 ✅, D3 ✅, D4 ✅, D5 ✅ (2 ⚠️ resolus inline). OK.

---

## §4 G4 rigor signal

L'audit a identifie **7 P2** et **5 P3** findings. Le seuil G4
(≥1 P2+ documente) est satisfait. Pas de concern "0 finding" =
audit superficiel.

---

## §5 Summary findings

| ID | Severite | Dimension | Description | Action |
|---|---|---|---|---|
| F1 | P2 | §2.5 | version non stockee en DB — replay_all() hardcode FEED_FORMAT_VERSION | Carry S62+. Ajouter colonne `version` avant tout bump de FEED_FORMAT_VERSION. |
| F2 | P2 | §2.8 | incremental ne verifie pas la chain sur les nouvelles entrees (cursor matche) | Carry S62 obligatoire AVANT sync P2P. |
| F3 | P2 | §2.8 | validate_feed_operation ne verifie pas les formats (hex, URL, enum reason) | Carry S62 pour sync P2P. |
| F4 | P2 | §2.8 | insert_feed_operation : get + insert sans transaction atomique | Carry S62 (mutex ou transaction). |
| F5 | P2 | §2.8 | iroh infra timeouts intermittents (0-8 tests, pre-existant) | Carry S62+ (gate SBFB_INTEGRATION). |
| F6 | P2 | §2.8 | spec §5 ne documente pas le trust contract local vs remote | Carry S62 (enrichir spec). |
| F7 | P2 | §2.8 | plan delta Phase D +4 prevu / +2 reel (2 pre-livres fix inter-phases) | Process — calibration plans futurs. |
| F8 | P3 | §2.1 | D2 payload "BLOB JCS" vs implementation "TEXT JSON serde" — cosmetic | 0 action (resolu D2 ⚠️). |
| F9 | P3 | §2.2 | verify_chain return type diverge (bool vs Result<(), String>) entre kudos et feed | 0 action (feed pattern superieur). |
| F10 | P3 | §2.3 | idx_feed_created premature (0 query par created_at actuellement) | 0 action (inoffensif). |
| F11 | P3 | §2.4 | entry_hash_at_seq O(N) + save_cursor_from_db O(N) | Carry performance S62+. |
| F12 | P3 | §2.8 | plan delta accounting — fix inter-phases a comptabiliser | Process. |

---

## §6 Verdict

**PASS** — 0 P0, 0 P1, 7 P2, 5 P3.

Sprint 62 Phase A peut demarrer directement. Les 7 P2 sont tous
acceptables pour Sprint 1 (single-writer, local-only) et
documentes comme carries pour Sprint 2 (sync P2P). Aucun ne
constitue un risque pour le fonctionnement actuel du feed local.

Les P2 critiques pour Sprint 62 (F2 incremental no-verify + F3
validation stricte + F4 transaction atomique) sont sur le chemin
critique de la sync P2P — ils doivent etre resolus AVANT que le
feed accepte des entrees provenant d'autres noeuds.

F1 (version non stockee) est moins urgent — il ne devient critique
que si `FEED_FORMAT_VERSION` est bumpe, ce qui n'est pas prevu
avant Sprint 4+ (hardening).

---

## §7 Out of scope (non re-debattu)

- D1..D5 gelees du kickoff S61
- Choix BLAKE3 vs SHA-256 (D3)
- SQLite vs iroh-docs pour storage local (D2)
- Format spec markdown vs protobuf (D4)
- Integration supplementaire vs remplacement BrowseAggregator (D5)
- Pin iroh 0.98 (exemption externe)
- Pin rand 0.8 (exemption externe P2-A-1)
