# Sprint 69 Phase A — deep review

HEAD: `b930c34` (tip master, diff non committe) | Agent: nexus-phase-review-deep (Opus 1M)

## Verdict : PASS

Promu de PASS-PENDING apres reconciliation Codex GPT 5.5.

(Rigor signal : 3 findings P2+ documentes / >=1 requis pour PASS)

## Codex reconciliation

Codex report : `.planning/active/sprint69_phase_A_codex_review.md`
(fichier brut `codex exec -o`, non reecrit).

Codex verdict : 3 CONFIRME, 2 PARTIEL, 0 GAP.

Corrections appliquees :
- **P0 Codex (main.rs:152)** : `run_scan_secrets()` faisait `process::exit(1)`
  avant retour a `main()`, bypassant l'audit log. Corrige : retourne
  `Err("secrets detected in project")` au lieu de `process::exit(1)`.
  L'exit code 1 est desormais gere par le `if let Err(e)` de main()
  APRES l'ecriture audit log.
- **P2 Codex (count-tests.sh)** : total combine excluait les doctests.
  Corrige : doctests inclus dans le total.

Gaps documentes (non corriges, documentes dans commit body) :
- P2-A-3 (review Claude) : `gates_results` absent de `AuditEntry` — differe
  Phase B (pipeline FG4-FG8 n'existe pas encore).
- P2 Codex (count-tests.sh) : `|| true` avale les erreurs. Le script est
  un aide-memoire pour les compteurs, pas un gate fiable. Acceptable.
- P3 Codex (main.rs) : Create ne logge pas `--output`. Nit reproductibilite.

Suites relancees post-correction :
- cargo fmt: 0 diff
- cargo clippy: 0 warnings
- cargo nextest: 1424/1424 PASS (inchange, process::exit n'etait pas teste)
- Vitest/frontend: non relance (aucun fichier web/ touche)

## Memory consultation
- feedback_approach.md : pick deepest, no band-aid, research before code — respecte (reject error vs LRU silent eviction = choix le plus strict)
- feedback_context7_systematic.md : context7 obligatoire avant code touchant lib/API — respecte (preflight S1a serde_json context7 done)
- vision_model.md : no funding/startup patterns — N/A (Phase A ne touche pas)
- feedback_kudos_non_monetary.md : N/A (Phase A ne touche pas kudos)
- sprint14_keyoxide_decision.md : deploy from source — N/A (Phase A ne touche pas deploy path)
- nexus_grid_pivot.md : S69 OPEN, tip b930c34, Day 0 D4 = audit log + P2-I-2 + P2-B-1 — respecte (Phase A implemente D4)

## Staging check
- Phase fichiers : 3 modifies (preview.rs, main.rs, THREAT_MODEL.md)
- Untracked pertinents phase : 2 (audit_log.rs, count-tests.sh)
- Untracked planning : 1 (sprint69_phase_A_preflight.md) — a stager dans le commit feat ou dans un chore(planning) prealable
- Planning/docs split : le preflight.md untracked est un artefact planning. Decision mecanique : il peut etre inclus dans le commit feat (c'est l'artefact G8 de la phase, pas un fichier planning separe d'un autre sprint). Acceptable.
- Untracked accidentels : 0 (pas de node_modules, .env, cache, .pdb, build artefacts)

## Suites verification
| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | - | - | ok |
| cargo clippy | - | - | - | ok |
| Rust nextest | 1419 | 1423 | +4 | ok |
| Rust doctests | ok | ok | - | ok |
| tsc --noEmit | - | - | - | ok |
| ESLint | - | - | - | ok |
| Vitest | 279 | 279 | +0 | ok |
| Build web | - | - | - | ok |
| size-limit | 6/6 | 6/6 | - | ok |
| Playwright | N/A | N/A | - | N/A (pas lance — Phase A ne touche pas le frontend) |
| scan-en-strings | N/A | N/A | - | N/A (pas de modif frontend) |
| Release build daemon | - | - | - | ok |
| Release build factory | - | - | - | ok |

Note Playwright : la procedure exige les 3 blocs complets. Playwright n'a pas ete lance dans cette session de review. Le team lead a rapporte "tout vert". Le reviewer accept le report mais note que l'independance G4 stricte aurait voulu un run Playwright ici. Classe P3 (nit, Phase A ne touche aucun fichier web/).

## Branch coverage semantique (deep)

### preview.rs — MAX_PREVIEW_ENTRIES check (lines 55-59)

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `if guard.len() >= MAX_PREVIEW_ENTRIES && !guard.contains_key(&hash_hex)` branch TRUE | `preview_rejects_too_many_entries` | oui (load 10 + load 11e) | oui (`assert!(matches!(err, PreviewError::TooManyEntries { .. }))`) | 11e = 1 au dessus du max → true branch | DEEP-PASS |
| meme branch FALSE (sous le seuil) | `load_returns_blake3_hash` + 5 autres tests existants | oui | oui | happy path, 1 entry | DEEP-PASS |
| `!guard.contains_key(&hash_hex)` — re-load meme hash quand plein | — | — | — | — | **PARTIAL P2** (voir P2-A-1) |
| `preview_accepts_after_eviction` | `preview_accepts_after_eviction` | oui (load 10, sleep, evict, load 1) | oui (`assert!(store.has(&hash))`) | eviction TTL + re-load | DEEP-PASS |

### audit_log.rs — AuditEntry + log_entry()

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `log_entry()` fn | `audit_log_writes_jsonl` | **NON** — le test reproduit le pattern (OpenOptions+append+writeln) mais n'appelle PAS `log_entry()` | N/A | N/A | **SHALLOW-PASS P2** (voir P2-A-2) |
| `log_entry()` fn | `audit_log_appends` | **NON** — meme pattern : test reproduit manuellement | N/A | N/A | SHALLOW-PASS |
| `audit_log_path()` fn | — | jamais appele dans les tests | — | — | **UNTESTED** mais path trivial (directories::BaseDirs fallback) | DEFENSIVE-OK |
| `AuditEntry` struct serde | `audit_log_writes_jsonl` | oui (serde_json::to_string) | oui (parsed["command"] == "create") | 1 entry JSON valide | DEEP-PASS |

### main.rs — audit log wiring

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `audit_log::log_entry(&entry)` call (line 132) | — | — | — | — | **WIRING-UNTESTED P2** (voir P2-A-2) |
| `let _ = audit_log::log_entry(&entry)` — fire-and-forget | — | — | — | — | DEFENSIVE-OK (pre-launch, log failure non-bloquant par design) |
| `time::OffsetDateTime::now_utc().format(Rfc3339)` (line 125-126) | — | — | — | — | DEFENSIVE-OK (unwrap_or_default, jamais panic en pratique) |

### THREAT_MODEL.md — §13 Preview ephemere

Pas de code executable. Contenu semantiquement verifie :
- T-PREVIEW-EXHAUSTION : les mitigations citees (MAX_PREVIEW_BYTES, MAX_PREVIEW_ENTRIES, TTL, loopback, bearer) correspondent au code reel.
- Numerotation : §13 Preview ephemere insere correctement, §14 Revue et evolution. Conforme au preflight S2 attention.
- Historique v6 ajoute en fin de fichier. Coherent.

### count-tests.sh

Script utilitaire, pas de test. Verifie manuellement : parse nextest via grep -oP, parse vitest via grep -oP. Syntaxe bash correcte. `set -euo pipefail` present. `|| true` apres chaque commande pour eviter l'arret sur echec (nextest/vitest retournent non-zero quand des tests echouent). Acceptable.

## Scope cuts semantique (deep)

14 scope cuts kickoff §7. Verification exhaustive :

| # | Scope cut | Grep mecanique | Diff semantique | Signal |
|---|-----------|----------------|-----------------|--------|
| 1 | SearchManifest wire + gossip | 0 match | 0 code | CLEAN |
| 2 | Page React /factory | 0 match | 0 code frontend | CLEAN |
| 3 | @dev index tree-sitter | 0 match | 0 code | CLEAN |
| 4 | Template react-vite | 0 match | 0 code | CLEAN |
| 5 | CuratorVouched UI shell | 0 match | 0 code | CLEAN |
| 6 | FG10 Review gate | 0 match | 0 code | CLEAN |
| 7 | Fuzzing cargo-fuzz/proptest | 0 match | 0 code | CLEAN |
| 8 | Feed format version bump | 0 match | 0 code wire | CLEAN |
| 9 | ProofCard comme feed op | 0 match | 0 code | CLEAN |
| 10 | Diff engine avance | 0 match | 0 code | CLEAN |
| 11 | Multi-template switching UI | 0 match | 0 code | CLEAN |
| 12 | Factory update-check | 0 match | 0 code | CLEAN |
| 13 | Babel traduction live | 0 match | 0 code | CLEAN |
| 14 | iroh 1.0 upgrade | 0 match | 0 code | CLEAN |

Verdict scope cuts : **CLEAN** — aucun scope cut touche.

## Research grounding (deep)

### Preflight G8
- Fichier : `.planning/active/sprint69_phase_A_preflight.md` — **existe**
- Scans : 5/5 (S1a OSS prior art, S1b Deps/CVE, S2 Decision chain, S3 Threat model, S4 Wire format)
- S1a OSS : RustDesk, cargo-audit, vault-audit-tools, moka-rs, bootstrap_allowlist.rs interne — 5 projets nommes
- Verdict : **EXECUTE plan-as-is**
- Signal : **PASS**

### Deps/API
Aucune dep Cargo.toml ajoutee ni modifiee dans le diff. Pas de nouvelle dep npm.

| Dep/API | Version | Trace §Research | Coherence code-vs-doc | Signal |
|---------|---------|-----------------|----------------------|--------|
| serde_json | 1.0.149 | oui (preflight S1b) | to_string en audit_log.rs = standard | PASS |
| time | workspace | oui (preflight S1b) | OffsetDateTime::now_utc().format(Rfc3339) = standard | PASS |
| blake3 | 1.8.5 | oui (preflight S1b) | inchange, usage dans preview.rs pre-existant | PASS |

### Coherence code-vs-source
- audit_log.rs utilise `serde_json::to_string` + `writeln!` = pattern confirme RustDesk + vault-audit-tools (S1a)
- `OpenOptions::new().create(true).append(true)` = pattern JSONL standard
- `directories::BaseDirs::new()` pour `~/.sbfb/` = pattern interne existant
- Pas de divergence code-vs-source

## Security deep

### Scan automatique

| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| audit_log.rs | 0 unwrap prod | - | clean | unwrap uniquement dans #[cfg(test)] |
| audit_log.rs | 0 unsafe | - | clean | - |
| main.rs | `unwrap_or_default()` | 126 | clean | fallback safe sur format timestamp |
| preview.rs | 0 nouveau unwrap prod | - | clean | les unwrap sont pre-existants dans les tests |
| count-tests.sh | 0 secrets | - | clean | pas de token/cle |

### Checks specifiques par zone

| Zone touchee | Check obligatoire | Resultat |
|---|---|---|
| preview.rs (daemon-core, pas loopback HTTP direct) | pas de nouvelles routes → PeerCredsVerified N/A | N/A |
| audit_log.rs (factory CLI, pas daemon) | pas de wire format → JCS N/A | N/A |
| THREAT_MODEL.md (documentation) | pas de code executé | N/A |
| Pas de `unsafe` nouveau | - | clean |
| Pas de `#[cfg(not(test))]` nouveau | - | clean |
| Pas de `#[allow(dead_code)]` nouveau | - | clean |
| Pas de `#[ignore]` / `.skip()` | - | clean |

### Analyse semantique securite

1. **`let _ = audit_log::log_entry(&entry);`** (main.rs:132) — le `let _ =` ignore l'erreur de log. C'est intentionnel (pre-launch, le log est best-effort). Un attaquant qui corrompt le fichier log ou rend `~/.sbfb/` non-writable fait echouer silencieusement l'audit log. Acceptable car le log n'est pas un artefact de securite signe (cf. preflight S3 V4). **Pas de finding.**

2. **`audit_log_path()` fallback `PathBuf::from("factory-audit.log")`** (audit_log.rs:19) — si `BaseDirs::new()` echoue (edge case tres rare, env HOME non defini), le log est ecrit dans le CWD. Pas de risque de securite (le fichier est local, pas secret). **Pas de finding.**

3. **Preview `TooManyEntries` error message** expose `count` et `limit` — information leakage negligeable (valeurs constantes publiques). **Pas de finding.**

4. **`time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_default()`** — le `unwrap_or_default()` est correct, le format Rfc3339 ne peut pas echouer sur un `OffsetDateTime::now_utc()` valide. Le `or_default()` est une surete supplementaire. **Pas de finding.**

## Livrable verification (Claude pre-Codex, ne remplace pas Codex)

| # | Livrable | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | `audit_log.rs` — AuditEntry struct + log_entry() + audit_log_path() + 2 tests | CONFIRME | `crates/sbfb-factory/src/audit_log.rs:8-96` | `pub struct AuditEntry { timestamp, command, args, result }`, `pub fn log_entry(entry: &AuditEntry)`, `pub fn audit_log_path()`, tests `audit_log_writes_jsonl` et `audit_log_appends` |
| 2 | `main.rs` — mod audit_log + appel log_entry() apres chaque subcommand | CONFIRME | `crates/sbfb-factory/src/main.rs:6,119-132` | `mod audit_log;` ligne 6. Bloc `let (cmd_name, cmd_args, result)` avec capture par subcommand. `audit_log::log_entry(&entry)` ligne 132 avec timestamp RFC3339. |
| 3 | `preview.rs` — MAX_PREVIEW_ENTRIES + TooManyEntries + check dans load() + 2 tests | CONFIRME | `crates/nexus-shell-daemon-core/src/preview.rs:19,55-59,97-98,159-181` | `pub const MAX_PREVIEW_ENTRIES: usize = 10;`, check `guard.len() >= MAX_PREVIEW_ENTRIES`, variant `TooManyEntries`, tests `preview_rejects_too_many_entries` et `preview_accepts_after_eviction` |
| 4 | `THREAT_MODEL.md` — §13 Preview ephemere + renommage §13→§14 + historique v6 | CONFIRME | `docs/security/THREAT_MODEL.md:658-731` | §13 T-PREVIEW-EXHAUSTION avec vecteurs, mitigations, table STRIDE. §14 Revue et evolution. Historique v6 ligne 730. |
| 5 | `count-tests.sh` — parse nextest + vitest, compteurs structures | CONFIRME | `scripts/count-tests.sh:1-31` | `set -euo pipefail`, parse `grep -oP '\d+ passed'` pour nextest, `grep -oP 'Tests\s+\d+'` pour vitest, summary combine. |

Resume : 5 livrables / 5 confirmes / 0 gaps / 0 partiels

## Patterns drift + horizon long-terme

### Patterns
- `docs/rust/PATTERNS.md` lu. Le diff ne contredit aucun pattern numerote.
- `docs/shell/PATTERNS.md` lu. Aucun pattern viole.
- Le pattern JSONL append (audit_log.rs) est coherent avec les patterns existants (serde_json pour serialisation, thiserror pour erreurs).
- Pas de nouveau pattern a documenter (le code est trivial — struct + append fichier).

### Horizon long-terme
- Design doc present (nouveaux modules) : N/A — audit_log.rs est un module utilitaire trivial (~30 LOC prod), pas un module structurant.
- D1..D5 avec alternatives + rationale : oui — D4 dans kickoff documente 3 alternatives rejetees (tracing framework, ne pas resoudre P2-I-2, LRU au lieu de reject).
- Solution la plus poussee : oui — reject error > LRU silent eviction (choix le plus strict). JSONL simple > framework lourd.
- Aucune LOC estimee au plan : verifie, 0 match "LOC estim" dans plan.md.
- Signal : **PASS**

## Commit body validation

### Titre
- Format cible : `feat(factory): Sprint 69 Phase A — preview cap + audit log + P2-I-2 template`
- Regex match : `feat(factory): Sprint 69 Phase A — .+` = ok

### 9 sections body
Le body n'est pas encore ecrit (pre-commit). **CONCERN** "draft-body-absent". Rappel du template : les 9 headers obligatoires sont documentes dans `.claude/templates/commit_body_phase.txt` et le plan §4.5.

### Co-Authored-By
- Pas encore verifiable (body absent). Rappel : `Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>` obligatoire.

## Findings

- **P2-A-1** : `preview.rs:55` — Le check `!guard.contains_key(&hash_hex)` permet de re-load le meme hash quand le store est plein (car le `contains_key` renvoie true, donc le `if` est false, et l'insert ecrase l'ancien). C'est un comportement acceptable (idempotent re-upload du meme contenu). **Mais aucun test ne couvre cette branche specifique** : charger 10 entries distinctes, puis re-charger la meme entry → devrait reussir (pas TooManyEntries). Signal : **PARTIAL P2** — test manquant pour le edge case "re-load same hash when full".

- **P2-A-2** : `audit_log.rs:48-67` — Les 2 tests (`audit_log_writes_jsonl`, `audit_log_appends`) **ne testent PAS la fonction `log_entry()`**. Ils reproduisent manuellement le pattern (OpenOptions + append + writeln) dans un tempdir. La fonction `log_entry()` elle-meme (qui inclut `create_dir_all`, `audit_log_path()`, et le `writeln!` reel) n'est jamais appelee dans les tests. Le wiring dans `main.rs` (ligne 132) est egalement non teste. Signal : **SHALLOW-PASS P2** — les tests verifient le format JSONL et l'append, mais pas la fonction reelle. Direction fix : remplacer les tests par des appels directs a `log_entry()` avec un mock de `audit_log_path()` ou un tempdir override, ou ajouter un 3e test qui appelle `log_entry()` avec un env var pour overrider le path.

- **P2-A-3** : `audit_log.rs:13` — Le champ `gates_results` mentionne dans le plan §4.2 ("gates_results") est **absent** de la struct `AuditEntry`. La struct a 4 champs : `timestamp`, `command`, `args`, `result`. Le plan dit "Struct `AuditEntry` (timestamp, command, args, result, gates_results)". Le champ `gates_results` est sans doute prevu pour Phase B (quand les gates sont wirees dans le pipeline). C'est un delta plan-vs-code non documente. Signal : **P2** — le plan promet 5 champs, le code en livre 4. Le champ absent sera necessaire Phase B. Pas bloquant car `serde(default)` le rendra optionnel, mais le delta doit etre documente dans le commit body.

- **P3-A-1** : Playwright non lance dans la review (suites frontend validees par tsc+lint+vitest+build+size mais pas Playwright). Phase A ne touche aucun fichier web/ donc le risque de regression Playwright est negligeable. Nit process.

## Codex reconciliation
- Status : N/A pre-Codex
- Rapport Codex : a produire apres cette review
- GAPs P0/P1 : 0
- P2/P3 documentes dans body : N/A

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep unwrap/unsafe/allow(dead_code) sur audit_log.rs, main.rs, preview.rs | audit_log.rs (entier), main.rs (entier), preview.rs (entier) | 0 |
| Patterns | PATTERNS.md Rust lu (1646 lignes), PATTERNS.md shell lu (1598 lignes) | 2 fichiers | 0 |
| Scope-cuts | 14 items kickoff §7, grep + lecture semantique diff entier | kickoff.md §7, diff 3 fichiers + 3 untracked | 0 |
| Branch coverage | 4 nouvelles fonctions/branches, 4 tests lus en entier + 6 tests existants | preview.rs tests (8 tests), audit_log.rs tests (2 tests) | 3 (P2-A-1, P2-A-2, P2-A-3) |
| Research grounding | preflight G8 lu (5 scans), 0 deps ajoutees, coherence code-vs-source | preflight.md (entier), kickoff.md §Sources (16 sources) | 0 |
| Livrables | 5/5 verifies via Read avec numeros de ligne | 5 fichiers (audit_log.rs, main.rs, preview.rs, THREAT_MODEL.md, count-tests.sh) | 0 |
| Horizon long-terme | plan.md D4 alternatives, 0 LOC estim, design doc N/A (trivial) | plan.md §4, kickoff.md §4 D4 | 0 |

## Recommendation
- Ready to commit : non (verdict PASS-PENDING, Codex obligatoire)
- Carry-overs S70 : P2-A-1 (test edge case re-load same hash when full) si non corrige avant commit — acceptable comme P2 carry
- Corrections recommandees avant commit :
  - P2-A-2 : ajouter un test qui appelle `log_entry()` reellement (pas juste le pattern). Alternativement, documenter dans le commit body que les tests verifient le format mais pas la fonction.
  - P2-A-3 : documenter dans le commit body le delta `gates_results` absent vs plan (sera ajoute Phase B).
  - P2-A-1 : ajouter `test_preview_allows_same_hash_when_full` ou documenter comme carry.

## Post-commit obligatoire
- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
