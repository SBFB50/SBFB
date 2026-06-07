# Sprint 74 Phase B — Review (multi-dimension adversariale)

## Verdict: PASS

Date: 2026-06-07
HEAD (pré-commit): `457ca05` (Phase A) + Phase B staged.
Méthode: Workflow adversarial 5 agents indépendants (1M ctx) — fallback de
`nexus-phase-review-deep`. ~416k tokens, 89 tool-uses.
Verdict final: **PASS** (PASS-PENDING initial → 1 P0 + 2 P1 + 3 P2 corrigés
en-phase → promu PASS après réconciliation Codex).

## Dimensions & verdicts (avant résolution)

| Dimension | Verdict | Findings |
|---|---|---|
| CORRECTNESS | **FAIL** | 1 P0 + 1 P3 |
| SECURITY | CONCERN | 1 P1 + 1 P3 |
| SCOPE & PREFLIGHT-FIDELITY | PASS | 1 P3 |
| TESTS | CONCERN | 1 P1 + 2 P2 + 1 P3 |
| ARCHITECTURE & DEP-HYGIENE | PASS | 2 P2 + 1 P3 |

C'est une phase qui manipule du contenu forge/zip NON FIABLE — la review
adversariale a justement trouvé 2 vrais trous sécurité (P0 RCE + P1 zip-bomb)
que la rédaction initiale avait manqués en mirrorant deploy.rs incomplètement.

## Findings & résolutions (toutes traitées en-phase, fork.rs réécrit)

### P0 (CORRECTNESS/SECURITY) — git argument-injection via `commit_sha` → RCE
`commit_sha` (origine = feed op `ReleasePublished` d'un pair, non fiable) était
passé verbatim à `git fetch --depth 1 origin <sha>`. Un `--upload-pack=<cmd>`
est parsé comme option et EXÉCUTE la commande (classe CVE-2017-1000117). La
review l'a reproduit empiriquement. deploy.rs (le chemin béni mirroré) valide
`is_valid_sha` (40-hex) AVANT — j'avais omis cette garde.
**Résolu** : `is_valid_sha(sha)` (40 ascii-hex, parité deploy.rs) dans
`fork_from_forge` AVANT tout git + séparateur `--end-of-options` avant le sha
(défense en profondeur). Test `fork_from_forge_rejects_argument_injection_sha`
(payload `--upload-pack=touch SENTINEL` → `InvalidCommitSha` + sentinelle NON
créée). `fork.rs`.

### P1 (SECURITY) — zip-bomb : cap compressé seulement, pas décompressé
`fork_from_blob` ne plafonnait que `zip_bytes.len()` (compressé). Un
deflate-de-zéros ~500 Mo → ~500 Go remplit le disque. C'est le SEUL site
d'extraction zip-vers-disque de contenu non fiable du projet.
**Résolu** : `MAX_DECOMPRESSED_BYTES` (500 Mo) + comptabilité décompressée par
copie bornée `Read::take(remaining+1)` (parité `blob_serve::load`). Seam de test
`extract_zip(bytes, dest, cap)`. Test `fork_from_blob_rejects_zip_bomb` (cap
injecté 1024, entrée 10 000 octets → `ArchiveTooLarge`). Doc corrigée. `fork.rs`.

### P1 (TESTS) — `fork_from_search_hit_prefers_forge_then_blob` faux-vert
Le test ne construisait jamais le cas BOTH-present → la préférence forge-sur-blob
(cœur du dispatch) était non vérifiée.
**Résolu** : cas both-present avec `repo_url` https injoignable (`127.0.0.1:1`) +
blob bytes → assert `Err(Git|GitTimeout)` (prouve que le chemin forge a été pris,
pas le repli blob qui aurait rendu `Ok(Blob)`). `fork.rs`.

### P2 (TESTS) — clone test ne prouvait pas le pin d'un sha distinct
Fixture mono-commit → checkout(HEAD) indistinguable d'un no-op.
**Résolu** : fixture **deux commits** (`old`/`new`), checkout du sha `old`,
assert contenu == `old` (pas le HEAD `new`) → prouve le pin. `fork.rs`.

### P2 (TESTS) — branches défensives non couvertes (symlink / cap / canonicalize)
**Résolu** : `fork_from_blob_skips_symlink_entries` (vrai symlink via
`zip::add_symlink` + `ZipFile::is_symlink()` — détection robustifiée vs le
masquage `unix_mode` manuel qui ne lisait pas S_IFLNK) ;
`fork_from_blob_rejects_zip_bomb` (cap) ; `fork_from_blob_rejects_zip_slip_all_vectors`
(5 vecteurs : `..`, `/abs`, `\win`, `a\b`, nested). `fork.rs`.

### P2 (ARCH) — forge clone sans size-cap (parité deploy.rs)
**Résolu** : cap post-clone `dir_size(dest) > MAX_CLONE_BYTES` + cleanup
(`CloneTooLarge`), mirroir deploy.rs. `fork.rs`.

### P3 (traités)
- kill_on_drop(true) sur le `git` tokio (plus d'orphelin sur timeout).
- Parité guard `is_safe_archive_path_matches_canonical_rules` (table de fixtures
  vs `validate_zip_path`) — détecte un drift futur.
- zip-slip multi-vecteur (couvert par all_vectors ci-dessus).

### P3 (documentés, sans action)
- Duplication `is_safe_archive_path` vs `validate_zip_path` : choix délibéré v4 D2
  (Factory hors daemon — ne PAS coupler le client à daemon-core qui tire iroh/
  FROST/rusqlite). SCOPE-fidelity dimension = PASS, ARCH dimension = PASS sur ce
  point. Parité verrouillée par le test ci-dessus.
- Blob hash non re-vérifié vs archive_hash : l'autorité d'intégrité est la
  frontière fetch daemon (iroh-blobs content-addressed). Documenté dans le module
  doc.

## Suites (toutes vertes après résolution)
fmt 0 · clippy --workspace --all-targets 0 · nextest workspace **1616/1616**
0-skip (Windows) · doctests 0 · release build OK (daemon, inchangé par fork.rs).
Docker Linux : voir verification.md (lancé en parité).

## Codex reconciliation

Codex (GPT 5.5, `codex exec -o sprint74_phase_b_codex_review.md`) a audité les 7
livrables : **CONFIRMÉ sur tout le périmètre** (fork.rs workspace-only, clone git
CLI zéro-dep, https + 40-hex sha + `--end-of-options`, zip-slip + symlink + cap
compressé/décompressé, workspace hors repo_root, C.3 test-only, B.6 au chokepoint
protégeant le gossip-ingest, 0 wire/VERSION/migration/scope-C). **1 GAP L3 (low)**.

**GAP L3 — FIXÉ en-phase** : `is_safe_archive_path` ne rejetait pas lexicalement
un préfixe absolu Windows `C:/...` avant `create_dir_all(parent)`. Réel : sur
Windows `dest.join("C:/x")` échappe `dest` (join d'un chemin absolu remplace la
base). La défense-en-profondeur canonicalize l'attrapait déjà, mais un rejet
lexical est plus robuste (ne dépend pas de `canonicalize` réussir).
**Résolu** : `is_safe_archive_path` rejette désormais tout `:` (drive Windows
`C:/` + ADS `name:stream`) — plus strict que le `validate_zip_path` canonique
(qui sert en mémoire, ne joint jamais au disque). Doc + 2 tests étendus
(`is_safe_archive_path_matches_canonical_rules` + `..._rejects_zip_slip_all_vectors`
gagnent `C:/evil`, `C:evil`, `file.txt:stream`).

**Verdict final : PASS** (CONFIRMÉ + 1 L3 réconcilié par fix). Promu de
PASS-PENDING à PASS.
