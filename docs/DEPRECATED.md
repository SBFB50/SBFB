# DEPRECATED — suppressions explicites avec rationale

Mécanisme canonique de suppression d'un item de dette ou d'un artefact
(`docs/claude/README.md` §6.2.1) : jamais un `git rm` muet. Chaque entrée
préserve le contrat grep-history — l'ID reste greppable ici avec son
rationale et le SHA qui permet `git show` du contenu intégral d'origine.

## Sprint 82 Phase E (2026-07-14) — zombies de l'ère Python (retirée S50-S51, purge `49782a9`)

Contexte : le projet est Rust+Frontend pur depuis S50-S51 (`git ls-files
packages/` = 0, `git ls-files nexus/` = 0). Les tickets ci-dessous du ledger
`docs/shell/PATTERNS.md` pointaient exclusivement du code Python supprimé —
leur concern est **N/A — Python path removed S50-S51**, pas « résolu ».
Contenu intégral d'origine : `git show c7b6790:docs/shell/PATTERNS.md`.
Les IDs purgés ne seront JAMAIS réutilisés pour de la dette NOUVELLE (pas
plus que le trou T19 shell) — nuance : les numéros T15/T16 restent portés
par les tickets Sprint 77 pré-existants du ledger shell, désormais uniques
(c'est la résolution de la collision, pas une réutilisation).

| ID | Titre d'origine | Rationale de purge |
|---|---|---|
| shell-T15 (a, S9) | SVG BOM UTF-8 false negative in magic bytes check | `files.py` supprimé ; l'upload/serve vit en Rust. Suppression = résolution de la collision d'ID avec T15 (S77, verify.sh) qui reste unique |
| shell-T16 (a, S9) | CAS manifest `content_type` is client-controlled | `files.py` supprimé ; concern MIGRÉ en Rust : le content-type servi est dérivé côté serveur (`blob_serve.rs:215` `detect_content_type`, extension-first + fallback magic-bytes), rien de client-contrôlé n'est stocké. Résout la collision avec T16 (S77, compute-shard E2E) |
| shell-T17 (S9) | `AppFileStore.open()` reads entire file into memory | `files.py` supprimé ; le serve Rust (`blob_serve.rs`) a son propre modèle mémoire (LRU + zip bomb limit) |
| shell-T18 (S9) | `test_concurrent_store_same_sha256_dedup_safe` flaky on Windows | test Python supprimé avec `packages/` |
| shell-T20 (S9) | `asyncio.wait_for()` in anyio-based SSE generator | `events.py` supprimé. **Namespace distinct de rust-T20** (relay cert-pinning, carry sécurité VIVANT — préservé) |
| shell-T21 (S9) | `useAppEvents` creates one EventSource per component mount | `web/src/hooks/useAppEvents.ts` n'existe plus (SSE coordinator Python-era) ; le shell actuel n'a pas d'EventSource par mount |
| shell-T22 (S9) | `test_gov_documents.py` schema diverges from `001_documents.sql` | app-gov Python supprimée intégralement |
| shell-T23 (S10) | SPDX scope excludes `nexus/` legacy Python files | `git ls-files nexus/` = 0 — le sujet du ticket n'existe plus ; `check-spdx.sh` couvre les modules actifs |
| shell-T44 (S14) | `_dir_size` check is post-clone, not streaming | `deploy.py` supprimé ; le deploy-from-repo vit en Rust (fork S74 Phase B, protections propres) |
| shell-T45 (S14) | `_git_rev_parse` has no timeout | `deploy.py` supprimé |
| shell-T46 (S14) | `startswith("http")` accepts `http://` | `deploy.py` supprimé ; le chemin Rust valide les URL (`isHttpsUrl` côté shell S74 B.5) |
| shell-T47 (S14) | `provenance.py` uses `json.dumps` instead of `jcs` | `provenance.py` supprimé ; le canonical Rust est traité par la note « serde_json vs JCS pre-launch » (`docs/rust/PATTERNS.md`) |
| shell-T48 (S14) | `verify_provenance` ignores `schema_version` | `provenance.py` supprimé ; la vérification vit en Rust (`nexus-coordinator-rs`) |
| shell-T50 (S14) | D4 clone protections lack dedicated tests | `test_deploy.py` supprimé ; les protections clone Rust ont leurs tests (S74 fork + `MAX_ARCHIVE_ENTRIES`, `strip_zip_member`) |
| shell-T51 (S14) | `_clone_repo` never exercised against a real subprocess | `test_deploy.py` supprimé |

Note : **T49 (S14) est EXCLU de cette purge** — son ancre est Rust et
vivante (`crates/nexus-shell-daemon-core/src/publish.rs`) ; il reste OPEN
dans le ledger, re-ancré (v4→v1, `:131`→`:183`) par S82 Phase E.

## Sprint 82 Phase E (2026-07-14) — scripts zombies

| Artefact | Rationale |
|---|---|
| `scripts/setup.sh` | Setup dev **intégralement Python-era** (uv venv + `uv sync` sur `packages/` + wheel maturin `nexus-core-py`) — toutes cibles supprimées S50-S51 ; le script échoue au premier sync du workspace Python (`uv sync` sur `packages/` absent) sur un checkout actuel. Setup réel : toolchain Rust (rustup) + `cd web && npm install`. Contenu : `git show c7b6790:scripts/setup.sh` |
| `scripts/verify.sh` steps 4-8 (+ préambule `.venv`) | `uv run ruff format/check packages/ examples/` + 3 × `uv run pytest packages/...` — abort garanti au step 4 sur checkout frais (`set -euo pipefail`, `packages/` absent). Retirés, steps renumérotés 1-16. Ferme shell-T15 (b, S77). Contenu d'origine : `git show c7b6790:scripts/verify.sh` |
| `.githooks/post-merge` | Hook opt-in S9 qui rappelait `./scripts/setup.sh` pour reconstruire le **wheel `nexus_core`** après un pull touchant `crates/nexus-core-py/` — crate et wheel supprimés S50-S51 ; le script rappelé est lui-même purgé (ci-dessus). Les hooks `.githooks/{pre-commit,commit-msg}` (agentctl lightcheck/auditor-gate) sont VIVANTS et non concernés. Contenu : `git show c7b6790:.githooks/post-merge` |

## Sprint 82 Phase E (2026-07-14) — doc contributeur réconciliée (pas une suppression)

`CONTRIBUTING.md` décrivait encore le monorepo Python-era (`packages/`
uv, `setup.sh`, standards ruff/pytest, « 18 steps ») — réconcilié avec
l'arborescence réelle (Rust + web + examples, verify.sh 16 steps).
