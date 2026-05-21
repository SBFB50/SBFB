# Sprint 67 Phase D -- deep review

HEAD: e8ee13f | Agent: nexus-phase-review-deep (Opus 4.6 1M)

## Verdict : PASS

0 P0, 0 P1. 3 findings P2+ documentes (1 P2 + 2 P3).
Corrections post-FAIL verifiees. Codex 7/7 CONFIRME — promoted.

(Rigor signal : 3 findings P2+ documentes / >=1 requis pour PASS)

---

## Historique verdicts

| Pass | Verdict | Motif |
|------|---------|-------|
| 1 | FAIL | P0-D-1 clippy `std::io::Error::other` |
| 2 | **PASS-PENDING** | P0-D-1 CORRIGE, P2-D-1 CORRIGE, 1 P2 + 2 P3 acceptes |

## Memory consultation

- feedback_approach.md : pick deepest, research before code -- RESPECTE
  (provenance design from preflight S1a + 5 projets OSS)
- feedback_context7_systematic.md : context7 obligatoire avant code
  touchant lib/API -- RESPECTE (blake3 context7 queried in kickoff)
- sprint14_keyoxide_decision.md : deploy from source -- N/A
  (factory.provenance.json est un fichier local distinct du SLSA
  provenance du daemon)
- vision_model.md : OpenBSD solo maintainer -- N/A (outillage interne)
- nexus_grid_pivot.md : Factory hors daemon D2 v4 -- RESPECTE
  (provenance.rs dans sbfb-factory, zero dep daemon)

## Staging check

- Phase fichiers : 3 modified (main.rs, template_engine.rs, PATTERNS.md)
  + 2 untracked (provenance.rs, sprint67_phase_d_preflight.md)
- Planning/docs split : preflight.md est un artefact Phase D, pas un
  chore separe -- OK
- Untracked accidentels : 0

## Suites verification

Post-fix verification (pass 2) :

| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | - | - | ok |
| cargo clippy | - | - | - | ok (post-fix P0-D-1) |
| Rust nextest | 1379 | 1384 | +5 | ok |
| Rust doctests | ok | ok | - | ok |
| tsc --noEmit | - | - | - | ok |
| ESLint | - | - | - | ok (5 warnings T1 known) |
| Vitest | 270 | 270 | +0 | ok |
| Build web | N/A | N/A | - | N/A (Phase D ne touche pas web/) |
| size-limit | N/A | N/A | - | N/A |
| Playwright | N/A | N/A | - | N/A |
| scan-en-strings | N/A | N/A | - | N/A |
| Release build daemon | - | - | - | ok |
| Release build factory | - | - | - | ok |

Delta Rust : +5 (4 provenance unitaires + 1 wiring integration).
Plan estimait +3, livraison +5 (test wiring P2-D-1 fix + test
exclusion files ajoutent 2 de plus).

## Branch coverage semantique (deep)

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `Provenance::generate()` success | `test_provenance_hash_deterministic` | oui (2 appels) | oui (output_hash == output_hash) | same inputs only | DEEP-PASS |
| `Provenance::generate()` success | `test_provenance_json_parsable` | oui | oui (schema_version, hash lengths, generated_at) | - | DEEP-PASS |
| `Provenance::to_json()` | `test_provenance_json_parsable` | oui | oui (serde_json::from_str + field checks) | - | DEEP-PASS |
| `compute_output_hash()` EXCLUDED_FILES branch | `test_provenance_excludes_lock_and_provenance_files` | oui | oui (hash_before == hash_after with excluded files added) | both files | DEEP-PASS |
| `compute_output_hash()` walkdir error branch (l.53) | - | - | - | - | DEFENSIVE-OK (IO error propagation) |
| `compute_output_hash()` backslash normalization (l.62) | - | - | - | - | DEFENSIVE-OK (Windows-specific path normalization) |
| `template_hash` passthrough | `test_provenance_template_hash_matches_lock` | oui | oui (assert_eq template_hash input == output) | - | DEEP-PASS |
| Wiring in `template_engine::create()` (l.108-113) | `test_create_generates_provenance` | oui (full create()) | oui (exists + schema_version + 3 hashes 64 chars + template_hash cross-check lock) | - | **DEEP-PASS** (post-fix P2-D-1) |
| `mod provenance;` declaration (main.rs:6) | compilation | oui (compile) | - | - | DEEP-PASS |

**P2-D-1 CORRIGE** : `test_create_generates_provenance` (template_engine.rs:238-256)
verifie via le wiring complet `create()` :
(a) `factory.provenance.json` existe (l.244)
(b) JSON parsable (l.246)
(c) `schema_version == 1` (l.247)
(d) `output_hash`, `template_hash`, `variables_hash` sont des hex 64 chars (l.248-250)
(e) `template_hash` provenance == `template_hash` lock (l.252-255)
Signal promu de WIRING-UNTESTED a DEEP-PASS.

## Scope cuts semantique (deep)

| # | Libelle | Intention | Grep mecanique | Diff semantique | Signal |
|---|---------|-----------|----------------|-----------------|--------|
| 1 | Preview ephemere | pas de POST /preview | 0 match | 0 code directe, 0 preparation | CLEAN |
| 2 | Diff engine | pas de diff command | 0 match | 0 code directe | CLEAN |
| 3 | Page React /factory | pas de frontend | 0 match (0 fichier web/) | 0 code | CLEAN |
| 4 | Proof Cards | pas de computation | 0 match | 0 code | CLEAN |
| 5 | SearchManifest wire | pas de wire format | 0 match | 0 code | CLEAN |
| 6 | Babel dogfood | pas de Babel | 0 match | 0 code | CLEAN |
| 7 | @dev tree-sitter | pas d'index | 0 match | 0 code | CLEAN |
| 8 | Bridge proof_card_get | pas de bridge | 0 match | 0 code | CLEAN |
| 9 | Template react-vite | pas de nouveau template | 0 match | 0 code | CLEAN |
| 10 | Factory audit log | pas de JSONL | 0 match | 0 code | CLEAN |
| 11 | CuratorVouched UI | pas de frontend | 0 match | 0 code | CLEAN |
| 12 | Publish path | pas de HTTP factory->daemon | 0 match | 0 code | CLEAN |
| 13 | Feed format version bump | pas de bump | 0 match | 0 code | CLEAN |
| 14 | Fuzzing cargo-fuzz | pas de fuzzing | 0 match | 0 code | CLEAN |

Tous les 14 scope cuts CLEAN.

## Research grounding (deep)

### Preflight G8

- Fichier : `.planning/active/sprint67_phase_d_preflight.md` existe
- Scans : 5/5 (S1a, S1b, S2, S3, S4)
- S1a OSS : 5 projets nommes (Copier, cargo-generate, Backstage,
  SLSA/in-toto, Gitleaks)
- Verdict : EXECUTE plan-as-is
- Coherence : PASS

### Deps/API

| Dep/API | Version | Trace Research | Coherence code-vs-doc | Signal |
|---------|---------|----------------|----------------------|--------|
| blake3 | 1.8.3 (workspace) | oui (context7 + WebSearch kickoff) | Hasher::new + update + finalize = correct | PASS |
| walkdir | 2.5.0 (workspace) | oui (S1b scan) | WalkDir::new + follow_links(false) = correct | PASS |
| time | 0.3.47 (workspace) | oui (S1b, RUSTSEC-2026-0009 patched) | OffsetDateTime::now_utc + Rfc3339 format = correct | PASS |
| serde_json | 1.0.149 (workspace) | oui (S1b) | to_string + to_string_pretty = correct | PASS |

Aucune dep ajoutee. Toutes deja dans le workspace. 0 delta deps.

### Coherence code-vs-source

- blake3 API : `blake3::hash()` + `blake3::Hasher::new()` +
  `update()` + `finalize()` + `to_hex()` -- conforme a l'API
  documentee (crates.io/blake3 1.8.x).
- serde_json : `to_string` sur `Value` produit du JSON compact
  deterministe (BTreeMap backing sans preserve_order feature).
  Conforme.
- time : `OffsetDateTime::now_utc()` + `format(Rfc3339)` est le
  pattern standard pour timestamps ISO 8601. Conforme.
- walkdir : `WalkDir::new(dir).follow_links(false)` est le pattern
  standard pour traversal sans suivre les symlinks. Conforme.

## Security deep

### Scan automatique

| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| provenance.rs | `unwrap_or_default()` | 25 | P3 | `serde_json::to_string(Value)` ne peut jamais echouer, defensive acceptable |
| provenance.rs | `unwrap_or_else` | 33 | P3 | `time::format Rfc3339` ne peut pratiquement jamais echouer, defensive acceptable |
| provenance.rs | ~~clippy violation~~ | 53 | ~~P0~~ **CORRIGE** | `std::io::Error::other(e)` applique. Clippy passe clean. |

### Analyse semantique

**Inputs non-trustes** : `provenance.rs` recoit :
- `output_dir: &Path` -- provient de `create()` qui le recoit du
  CLI `--output` (user local). Pas de path traversal ici car le
  path est deja valide (create_dir_all a reussi). Le walkdir est
  restreint a ce directory.
- `template_hash: &str` -- provient de `TemplateLock::template_hash`
  qui est un BLAKE3 hex genere en interne. Pas d'input externe.
- `variables: &serde_json::Value` -- construit dans `create()` a
  partir de `name` (CLI) et `version` (hardcode "0.1.0"). Le JSON
  est serialise puis hashe. Pas de vecteur d'attaque.

**Risque de DoS** : `compute_output_hash` parcourt tout le
repertoire de sortie. Pour un workspace genere par `create()`, le
nombre de fichiers est borne (4-6 fichiers du template). Le walkdir
ne suit pas les symlinks (`follow_links(false)`). Risque negligeable
en contexte CLI local.

**Hash collision sans separateur** : `compute_output_hash` concatene
`name.as_bytes() + content` sans separateur de longueur. Deux paires
(name1, content1) et (name2, content2) ou name1+content1 ==
name2+content2 byte-a-byte produiraient le meme hash. P3 theorique
(noms de fichiers structures, probabilite negligeable en pratique).

## Livrable verification (Claude pre-Codex)

| # | Livrable | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | factory.provenance.json generation | CONFIRME | provenance.rs:18-42 | `pub fn generate(output_dir, template_hash, variables) -> Result<Self, io::Error>` avec schema_version=1, template_hash, variables_hash, output_hash BLAKE3, generated_at RFC3339 |
| 2 | Test determinisme | CONFIRME | provenance.rs:100-111 | `test_provenance_hash_deterministic` : 2 appels, assert_eq output_hash + variables_hash + template_hash |
| 3 | P52 BlobStore pattern | CONFIRME | PATTERNS.md:2609-2642 | Section complete avec code sample node.rs l.111-126, usage guide, limitation documentee |
| 4 | P2-66-1 feed republish limitation | CONFIRME | PATTERNS.md:2646-2656 | Note avec reference runtime.rs l.1961, limitation expliquee, future path documentee |
| 5 | Wiring provenance dans create() | CONFIRME | template_engine.rs:108-113 | `Provenance::generate(out, &lock.template_hash, &variables)?` + `fs::write(out.join("factory.provenance.json"), prov.to_json()?)?` |
| 6 | EXCLUDED_FILES (determinisme) | CONFIRME | provenance.rs:7 | `const EXCLUDED_FILES: &[&str] = &["factory.template.lock", "factory.provenance.json"];` -- conforme a la note impl du preflight |
| 7 | Test exclusion files | CONFIRME | provenance.rs:142-159 | `test_provenance_excludes_lock_and_provenance_files` : hash stable apres ajout des fichiers exclus |
| 8 | Test wiring (post-fix) | CONFIRME | template_engine.rs:238-256 | `test_create_generates_provenance` : full create() + 5 assertions (exists, schema, hashes, cross-check lock) |

Resume : 8 livrables / 8 confirmes / 0 gaps / 0 partiels

## Patterns drift + horizon long-terme

### Patterns

- P52 ajoute dans cette phase -- coherent avec le code node.rs existant.
  Code sample verifie : `BlobStore` enum lignes 111-126 de node.rs
  correspondent au sample dans PATTERNS.md.
- P2-66-1 note ajoutee -- coherente avec le test existant l.1961 de
  runtime.rs. Limitation documentee avec future path (integration
  test cross-node).
- P51 raw-op pattern : non touche par Phase D -- OK.
- Tous les patterns existants respectes (pas de `serde_json::to_string`
  sur wire format, pas de `unwrap()` en production reachable).

### Horizon long-terme

- Design doc present : provenance design dans preflight S1a (5 projets
  OSS analyses). factory.provenance.json est un fichier local, pas un
  module structurant > 1 sprint de lifetime (le schema est simple et
  stable).
- D1..D5 avec alternatives + rationale : les D1-D5 ne sont pas touchees
  par Phase D. La provenance est une extension de D5 (sbfb-factory
  create).
- Solution la plus poussee : BLAKE3 hash est la solution standard
  (alignee SLSA L1 per preflight S1a). Ed25519 signature differee S68+
  est documentee.
- Aucune LOC estimee au plan : 0 match. OK.

## Commit body validation

### Titre

Titre cible :
`feat(factory): Sprint 67 Phase D -- factory provenance + P52 BlobStore pattern + dette`
Format regex match : `feat\((factory)\): Sprint 67 Phase D -- .+` -- OK

### 9 sections body

Draft body absent au moment de la review. CONCERN : draft-body-absent.
Rappel : le body doit contenir les 9 headers ## obligatoires :
Contexte, Fichiers, Delta tests, Verification, Scope cuts,
G8 traceability, Pre-launch protocol, Codex verification,
Carry closure.

Le body devra documenter P2-D-2 (canonicalization fragile) et
P3-D-1/P3-D-2 comme findings acceptes.

### Co-Authored-By

N/A pre-commit.

## Findings

### CORRIGE (pass 1 -> pass 2)

- ~~**P0-D-1**~~ : **CORRIGE** -- clippy `std::io::Error::other(e)`
  applique dans provenance.rs:53. Clippy passe clean (exit 0, 0
  warnings). Verifie par re-run `cargo clippy --workspace
  --all-targets --locked -- -D warnings`.

- ~~**P2-D-1**~~ : **CORRIGE** -- `test_create_generates_provenance`
  ajoute dans template_engine.rs:238-256. Verifie : le test appelle
  `create()` E2E, asserte existence + JSON parsing + schema_version +
  3 hashes 64 chars + cross-check template_hash provenance == lock.
  5 assertions specifiques. Signal promu WIRING-UNTESTED -> DEEP-PASS.
  Nextest : 1384 pass (delta +5 vs Phase C).

### ACTIF (acceptes, a documenter dans body)

- **P2-D-2** : `variables_hash` canonicalization fragile --
  `provenance.rs:25` hashe `serde_json::to_string(variables)`.
  Deterministe aujourd'hui grace au BTreeMap backing (pas de feature
  `preserve_order` dans Cargo.toml workspace). Fragile si
  `preserve_order` est active au workspace level. L'executeur doit
  documenter ce P2 dans le commit body section "Carry closure" ou
  "Codex verification" comme finding accepte avec justification
  (serde_json sans preserve_order = BTreeMap = deterministe).

- **P3-D-1** : hash collision theorique sans length prefix --
  `compute_output_hash` (l.74-78) concatene `name.as_bytes()` +
  `content` sans separateur de longueur. Probabilite negligeable
  pour des noms de fichiers structures. Nit.

- **P3-D-2** : `unwrap_or_default()` silencieux -- `provenance.rs:25`
  `serde_json::to_string(Value).unwrap_or_default()` produit une
  string vide si la serialization echoue (impossible en pratique
  pour serde_json::Value). Meme pattern ligne 33. Nit.

(Bilan : 0 P0, 0 P1, 1 P2, 2 P3 -- satisfait PASS-PENDING)

## Codex reconciliation

- Status : CLEAN
- Rapport Codex : sprint67_phase_d_codex_review.md (output brut GPT 5.5)
- Verdict Codex : 7/7 livrables CONFIRME, 0 GAP, 0 PARTIEL
- GAPs P0/P1 : 0
- P2/P3 documentes dans commit body : P2-D-2 (canonicalization)
- Suites relancees post-fix : clippy, nextest 1384, fmt — tous verts
- Promotion review : PASS-PENDING → PASS

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep unwrap/unsafe/allow sur provenance.rs + template_engine.rs, clippy re-run post-fix | provenance.rs (161 lignes), template_engine.rs (320 lignes), secret_scanner.rs (120 lignes) | 3 (P0-D-1 CORRIGE, P3-D-1, P3-D-2) |
| Patterns | PATTERNS.md lu (P51, P52, P2-66-1 note, References), node.rs l.105-135 lu, runtime.rs l.1955-1975 lu | PATTERNS.md, node.rs, runtime.rs | 0 (code samples corrects) |
| Scope-cuts | 14 items kickoff S7 lus + diff semantique | kickoff.md S7, diff complet | 0 (14/14 CLEAN) |
| Branch coverage | 5 tests lus en entier (4 provenance + 1 wiring post-fix), wiring template_engine.rs lu | provenance.rs tests l.82-160, template_engine.rs l.183-256 (post-fix) | 1 (P2-D-1 CORRIGE) |
| Research grounding | preflight lu, 4 deps verifiees, coherence code-vs-doc | preflight.md, Cargo.toml sbfb-factory, Cargo.toml workspace | 0 |
| Livrables | 8/8 verifies via Read (incl. test wiring post-fix) | provenance.rs, template_engine.rs, PATTERNS.md | 0 (8/8 CONFIRME) |
| Horizon long-terme | design doc via preflight, alternatives dans kickoff D5 | preflight S1a, kickoff D5 | 0 |
| Variables hash canonicalization | serde_json features verifie, BTreeMap backing confirme | Cargo.toml workspace (serde_json), provenance.rs:24-27 | 1 (P2-D-2 actif) |

## Recommendation

- Ready to commit : **OUI apres Codex** (verdict PASS-PENDING)
- Codex requis : invoquer Codex GPT 5.5 sur le diff, reconcilier
  les findings, promouvoir review.md a PASS.
- P2 a documenter dans body : P2-D-2 (canonicalization serde_json)
- Carry-overs S68 : P3-D-1 (hash sans length prefix), P3-D-2
  (unwrap_or_default silencieux)

## Post-commit obligatoire

- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
