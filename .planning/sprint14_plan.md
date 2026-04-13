# Sprint 14 — Plan d'execution detaille

**Ecrit** : 2026-04-13
**Tip master d'entree** : `0253922`
**Base** : `sprint14_kickoff.md` valide par l'utilisateur

---

## 1. Etat verifie a l'entree

| Suite | Count | Commande |
|---|---|---|
| Rust workspace | 369 | `cargo test --workspace --locked` |
| Python SDK | 183 | `uv run pytest packages/nexus-sdk/tests/ -q` |
| Python coordinator | 99 + 1 skip | `uv run pytest packages/nexus-coordinator/tests/ -q` |
| Python app-gov | 46 | `uv run pytest packages/nexus-app-gov/tests/ -q` |
| Vitest | 191 | `cd web && npm run test:unit` |
| Playwright | 30 | `cd web && npx playwright test` |
| size-limit | 7/7 | `cd web && npm run size` |
| SPDX | 220/220 | `bash docs/spdx-check.sh` |
| clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| fmt | clean | `cargo fmt --all --check` |
| ruff | clean | `uv run ruff format --check packages/ && uv run ruff check packages/` |
| tsc | clean | `cd web && npx tsc --noEmit -p tsconfig.app.json` |
| eslint | clean | `cd web && npm run lint` |

---

## 2. Decisions Day 0 (gelees)

Cf. kickoff §4. Resume :

- **D1** : `POST /project/deploy-from-repo` remplace le deploy upload pour les apps publiques
- **D2** : provenance.json signe SLSA L1, domain `nexus-provenance-v1`
- **D3** : PA v4 + BrowseEntry `provenance_hash`, badge conditionnel
- **D4** : 7 protections clone (depth 1, 500 MB, 30s, no .git/, paths, no submodules, no symlinks)
- **D5** : P2 Sprint 13 (D-1 contraste, G-1 PAD_R, B-1 superseded)

---

## 3. Research consulte

- `sprint14_keyoxide_decision.md` (memory) : design complet du flow clone+verify+sign
- `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py` : endpoint existant (153 LOC)
- `packages/nexus-coordinator/tests/test_deploy.py` : 7 tests + _FakeDaemon pattern
- `crates/nexus-shell-daemon-core/src/publish.rs` : PA v3 structure (403 LOC, 20 tests)
- `crates/nexus-shell-daemon-core/src/browse.rs` : BrowseEntry structure (176 LOC)
- `crates/nexus-shell-daemon/src/http.rs` : PublishRequest + publish_project handler
- `crates/nexus-core-rs/src/canonical.rs` : JCS + domain separation pattern
- `crates/nexus-core-rs/src/crypto.rs` : KeyPair, sign, verify, blake3_hash
- `packages/nexus-coordinator/src/nexus_coordinator/keystore.py` : LoadedKeypair pattern
- `packages/nexus-coordinator/src/nexus_coordinator/kudos.py` : nacl.signing pattern Python
- `web/src/api/daemon.ts` : BrowseEntrySchema Zod (strict)
- `web/src/pages/Browse.tsx` : cards avec badges existants (repo_url, P2P, source)
- `web/src/pages/BrowsedProject.tsx` : top bar avec repo link

---

## 4. Phase A — Python deploy-from-repo + forge + provenance

### 4.1 Fichiers ajoutes

**`packages/nexus-coordinator/src/nexus_coordinator/forge.py`** (~100 LOC)
```
SPDX header
_FORGE_PATTERNS dict : regex → forge type (github, gitlab, codeberg, gitea)
detect_forge(url) → ForgeType | None
raw_file_url(repo_url, path, ref="HEAD") → str  # URL raw pour pre-check SBFB.json
is_repo_public(repo_url) → bool  # HEAD request, 200 = public
normalize_clone_url(repo_url) → str  # enleve les fragments, trailing slash
```

**`packages/nexus-coordinator/src/nexus_coordinator/provenance.py`** (~120 LOC)
```
SPDX header
PROVENANCE_SCHEMA_VERSION = 1
@dataclass ProvenanceRecord:
    schema_version: int
    repo_url: str
    commit_sha: str
    artifact_hash: str  # BLAKE3 hex du zip sans provenance.json
    node_id: str  # Ed25519 pubkey hex du coordinateur
    timestamp: str  # ISO 8601 UTC
    signature: str  # Ed25519 hex signature

generate_provenance(repo_url, commit_sha, artifact_hash, keypair) → ProvenanceRecord
    # 1. Construire le dict canonical (sans signature)
    # 2. canonical_bytes = jcs(dict) avec domain b"nexus-provenance-v1\x00"
    # 3. signature = nacl.signing.SigningKey(secret).sign(canonical_bytes).signature
    # 4. Retourner ProvenanceRecord complet

verify_provenance(record_json, public_key_bytes) → bool
    # 1. Parser le JSON, extraire signature
    # 2. Reconstruire canonical_bytes (meme domain)
    # 3. nacl.signing.VerifyKey(pub).verify(canonical_bytes, signature)

provenance_to_json(record) → str
    # json.dumps avec sort_keys pour lisibilite (pas pour la signature — 
    # la signature est calculee sur le canonical sans le champ signature)
```

**`packages/nexus-coordinator/tests/test_forge.py`** (~80 LOC)
```
test_detect_forge_github / gitlab / codeberg / gitea / unknown
test_raw_file_url_github / gitlab / codeberg
test_normalize_clone_url_strips_fragment_and_trailing_slash
test_is_repo_public (mock httpx → 200 et 404)
```

**`packages/nexus-coordinator/tests/test_provenance.py`** (~80 LOC)
```
test_generate_provenance_produces_valid_record
test_verify_provenance_accepts_valid_signature
test_verify_provenance_rejects_tampered_hash
test_verify_provenance_rejects_wrong_key
test_provenance_domain_separation (signature invalide avec domain different)
```

### 4.2 Fichiers modifies

**`packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py`**
- Ajouter `POST /project/deploy-from-repo` endpoint (~130 LOC) :
  ```python
  @router.post("/project/deploy-from-repo")
  async def deploy_from_repo(request: Request, body: DeployFromRepoBody):
      # 1. Valider le body (repo_url requis, commit_sha optionnel)
      # 2. Verifier visibilite == public (sinon 400)
      # 3. Creer tmpdir
      # 4. git clone --depth 1 --single-branch (subprocess, timeout 30s)
      # 5. Verifier taille < 500 MB
      # 6. Lire SBFB.json, verifier node_id == daemon node_id
      # 7. Verifier index.html existe
      # 8. Recuperer commit SHA (git rev-parse HEAD)
      # 9. Zip le contenu (exclure .git/, valider paths)
      # 10. Calculer BLAKE3 du zip
      # 11. Generer provenance.json signe
      # 12. Ajouter provenance.json au zip
      # 13. Store blob via daemon
      # 14. Publish avec provenance_hash
      # 15. Cleanup tmpdir
      # return {"deployed": True, "hash": ..., "provenance_hash": ...}
  ```
- Ajouter `DeployFromRepoBody` Pydantic model
- Ajouter helper `_clone_repo()`, `_zip_directory()`, `_read_sbfb_json()`
- Modifier `_publish_with_archive()` pour accepter `provenance_hash` optionnel

**`packages/nexus-coordinator/tests/test_deploy.py`**
- Ajouter ~8 nouveaux tests :
  ```
  test_deploy_from_repo_happy_path
  test_deploy_from_repo_missing_sbfb_json
  test_deploy_from_repo_wrong_node_id
  test_deploy_from_repo_missing_index_html
  test_deploy_from_repo_private_rejected (seul le public utilise ce endpoint)
  test_deploy_from_repo_provenance_in_zip
  test_deploy_from_repo_path_traversal_rejected
  test_deploy_from_repo_clone_timeout (mock subprocess)
  ```
- Les tests utilisent un repo local (pas de vrai git clone) via un tmpdir
  avec la structure attendue (SBFB.json + index.html), et on mock
  `subprocess.run` pour simuler le clone

### 4.3 Dependances Python

- `jcs` (PyPI) pour JSON Canonicalization Scheme — ou implementation inline
  si le paquet n'est pas disponible (sort_keys=True + format canonique minimal
  suffisent pour notre cas simple ou les valeurs sont toutes des strings)
- `pynacl` deja present (utilise par kudos.py)
- `hashlib` stdlib pour BLAKE3 — **non**, BLAKE3 n'est pas dans hashlib Python.
  Utiliser `blake3` PyPI ou deleguer au binding Rust `nexus_core.blake3_hash()`
  si deja expose. A verifier.

### 4.4 Critere d'acceptation

- `POST /project/deploy-from-repo {"repo_url": "...", "commit_sha": "..."}` → 200
  avec `{"deployed": true, "hash": "...", "provenance_hash": "..."}`
- Le zip sur le daemon contient `provenance.json` signe
- Tous les cas d'erreur retournent le bon status (400/413/422/503)
- `uv run pytest packages/nexus-coordinator/tests/ -q` : baseline + ~16 nouveaux tests
- `uv run ruff format --check packages/ && uv run ruff check packages/`

### 4.5 Commit cible

`feat(coordinator): Sprint 14 Phase A — deploy from repo with Keyoxide + SLSA L1 provenance`

---

## 5. Phase B — Rust ProjectAnnouncement v4 + BrowseEntry provenance

### 5.1 Fichiers modifies

**`crates/nexus-shell-daemon-core/src/publish.rs`**
- `PROJECT_ANNOUNCEMENT_VERSION` : 3 → 4
- Ajouter champ `provenance_hash: Option<String>` avec
  `#[serde(default, skip_serializing_if = "Option::is_none")]`
- Ajouter builder `with_provenance_hash(self, hash: String) -> Self`
- `from_gossip_bytes()` : accepter v1, v2, v3, v4
  (changer `ann.v > PROJECT_ANNOUNCEMENT_VERSION` en `ann.v > 4`)
- Ajouter domaine `DOMAIN_PROVENANCE_V1` dans `canonical.rs` (reserve pour
  verification cote Rust si necessaire au futur — pas utilise ce sprint)
- Tests :
  ```rust
  v4_announcement_with_provenance_hash_round_trips()
  v3_announcement_parses_without_provenance_hash()
  v4_announcement_without_provenance_hash_omits_field()
  ```

**`crates/nexus-shell-daemon-core/src/browse.rs`**
- `BrowseEntry` : ajouter `provenance_hash: Option<String>` avec
  `#[serde(default, skip_serializing_if = "Option::is_none")]`

**`crates/nexus-shell-daemon/src/http.rs`**
- `PublishRequest` : ajouter `provenance_hash: Option<String>` avec `#[serde(default)]`
- `publish_project()` handler : propager `provenance_hash` dans :
  - `ProjectAnnouncement` via `.with_provenance_hash()`
  - `BrowseEntry` direct entry
- Pas de verification Rust du provenance dans ce sprint (le coordinateur
  est la source de confiance, la verification cote receveur est Sprint 15+)

**`crates/nexus-core-rs/src/canonical.rs`**
- Ajouter `DOMAIN_PROVENANCE_V1: &[u8] = b"nexus-provenance-v1"` (reserve)

**`web/src/api/daemon.ts`**
- `BrowseEntrySchema` : ajouter `provenance_hash: z.string().optional()`

### 5.2 Critere d'acceptation

- `cargo test --workspace --locked` : baseline + 3 nouveaux tests publish
- `cargo clippy --workspace --all-targets --locked -- -D warnings` : 0 warnings
- `cargo fmt --all --check` : clean
- Un daemon v4 publie une annonce avec `provenance_hash`, un v3 la parse sans erreur
- Zod `.parse()` accepte les deux formats (avec/sans provenance_hash)

### 5.3 Commit cible

`feat(p2p): Sprint 14 Phase B — ProjectAnnouncement v4 with provenance hash`

---

## 6. Phase C — Frontend badge "Verifie" + P2 tech debt

### 6.1 Fichiers modifies

**`web/src/pages/Browse.tsx`**
- Dans la section badges du card (apres le badge "P2P" / "Source"),
  ajouter un badge conditionnel :
  ```tsx
  {entry.provenance_hash && (
    <span className="... bg-emerald-500/15 text-emerald-400 ...">
      <ShieldCheck className="h-3 w-3" />
      Verifie
    </span>
  )}
  ```
- Import `ShieldCheck` de lucide-react

**`web/src/pages/BrowsedProject.tsx`**
- Dans la top bar (section auto-hide), ajouter badge "Verifie" similaire
- Fix D-1 : ligne 271, `text-white/30` → `text-white/40`

**`web/src/pages/ProjectDetail.tsx`**
- Fix D-1 : ligne 131, `text-white/30` → `text-white/40`

**`packages/nexus-sdk/src/nexus_sdk/html_render.py`**
- Fix G-1 : ligne 195, `_SVG_PAD_R = 16` → `_SVG_PAD_R = 32`

**`docs/shell/PATTERNS.md`**
- Ajouter T41 (B-1 superseded by verified deploy), T42 (D-1 fixed),
  T43 (G-1 fixed)

**`web/src/pages/__tests__/BrowsedProject.test.tsx`**
- Ajouter test : entry avec `provenance_hash` → badge "Verifie" present
- Ajouter test : entry sans `provenance_hash` → badge absent

**`web/src/pages/__tests__/Browse.test.tsx`** (si existant, sinon inline dans le fichier existant)
- Ajouter test : card avec `provenance_hash` → badge "Verifie" present

### 6.2 Critere d'acceptation

- `cd web && npm run test:unit` : baseline + ~3 nouveaux tests badge
- `cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json`
- `cd web && bash scripts/scan-en-strings.sh` : clean (badge en francais "Verifie")
- `cd web && npm run build && npm run size` : dans les budgets
- `uv run pytest packages/nexus-sdk/tests/ -q` : vert (G-1 fix ne casse rien)
- Les 2 instances `text-white/30` corrigees sont les seules sur du texte 11px

### 6.3 Commit cible

`feat(web): Sprint 14 Phase C — verified badge + P2 tech debt Sprint 13`

---

## 7. Phase D — Tests integration + polish

### 7.1 Fichiers modifies

**`packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py`**
- Modifier `POST /project/deploy` pour les apps publiques :
  retourner 400 avec message indiquant d'utiliser `deploy-from-repo`
  a la place (les publics ne peuvent plus uploader un zip directement)
- Les apps privees continuent d'utiliser `POST /project/deploy` normalement

**`packages/nexus-coordinator/tests/test_deploy.py`**
- Ajuster `test_deploy_public_with_repo_url_accepted` : maintenant 400
  avec redirect message (public doit utiliser deploy-from-repo)
- Ajouter test integration : deploy-from-repo → blob stored →
  publish avec provenance_hash dans le payload → FakeDaemon recoit
  le bon payload
- Ajouter test : provenance.json present dans le zip envoye au daemon

**Nouveaux fichiers avec SPDX headers**
- `forge.py` : AGPL-3.0-or-later
- `provenance.py` : AGPL-3.0-or-later
- `test_forge.py` : AGPL-3.0-or-later
- `test_provenance.py` : AGPL-3.0-or-later

### 7.2 Critere d'acceptation

- `uv run pytest packages/nexus-coordinator/tests/ -q` : tous verts
- Le flow complet public : deploy-from-repo → 200 + provenance
- L'ancien `POST /project/deploy` pour public → 400 avec message
- L'ancien `POST /project/deploy` pour prive → 200 (inchange)
- `cargo test --workspace --locked` : tous verts
- `cd web && npm run test:unit` : tous verts

### 7.3 Commit cible

`feat(coordinator): Sprint 14 Phase D — integration tests + deploy public redirect`

---

## 8. Phase E — Docs (verification + audit plan)

### 8.1 Fichiers ajoutes

**`.planning/sprint14_verification.md`**
- Checklist fail-fast remplie (cf. §10 ci-dessous)
- Commit stack
- Metriques sprint (avant/apres)
- Surface nouvelle livree

**`.planning/sprint14_audit_plan.md`**
- 7-9 tracks pour Sprint 15 Phase 0 :
  - Track A : securite du clone (git clone attack surface)
  - Track B : provenance signing correctness (domain, canonical bytes)
  - Track C : SBFB.json verification (node_id match, forge detection)
  - Track D : backward compat PA v4 (v3 receivers)
  - Track E : badge UI (conditionnel, accessibilite)
  - Track F : deploy public redirect (ancien endpoint bloque)
  - Track G : tests et couverture
  - Track H : P2 tech debt resolution

### 8.2 Fichiers modifies

**`docs/shell/PATTERNS.md`**
- P24+ : nouveau pattern "deploy verified from source"
- T41-T43 si pas deja fait en Phase C

**`docs/rust/PATTERNS.md`**
- P-something : PA v4 pattern, DOMAIN_PROVENANCE_V1 reserve

### 8.3 Commit cible

`docs(sprint14): verification + audit plan for Sprint 15`

---

## 9. Git plan

| # | Commit | Phase |
|---|---|---|
| 1 | `feat(coordinator): Sprint 14 Phase A — deploy from repo with Keyoxide + SLSA L1 provenance` | A |
| 2 | `feat(p2p): Sprint 14 Phase B — ProjectAnnouncement v4 with provenance hash` | B |
| 3 | `feat(web): Sprint 14 Phase C — verified badge + P2 tech debt Sprint 13` | C |
| 4 | `feat(coordinator): Sprint 14 Phase D — integration tests + deploy public redirect` | D |
| 5 | `docs(sprint14): verification + audit plan for Sprint 15` | E |

---

## 10. Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | cargo test | `cargo test --workspace --locked` | 369 → 372+ pass | |
| 4 | ruff format | `uv run ruff format --check packages/` | clean | |
| 5 | ruff check | `uv run ruff check packages/` | clean | |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | 183 pass | |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | 99 → 115+ pass | |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | |
| 9 | tsc | `cd web && npx tsc --noEmit -p tsconfig.app.json` | clean | |
| 10 | eslint | `cd web && npm run lint` | clean | |
| 11 | vitest | `cd web && npm run test:unit` | 191 → 194+ pass | |
| 12 | build | `cd web && npm run build` | success | |
| 13 | size-limit | `cd web && npm run size` | 7/7 under budget | |
| 14 | scan-en | `cd web && bash scripts/scan-en-strings.sh` | clean | |
| 15 | playwright | `cd web && npx playwright test` | 30 pass | |
| 16 | PA v4 round-trip | `cargo test -p nexus-shell-daemon-core v4_announcement` | pass | |
| 17 | PA v3 backward | `cargo test -p nexus-shell-daemon-core v3_announcement_parses` | pass | |
| 18 | deploy-from-repo happy | `uv run pytest -k test_deploy_from_repo_happy_path -q` | pass | |
| 19 | provenance sign/verify | `uv run pytest -k test_verify_provenance -q` | pass | |
| 20 | forge detection | `uv run pytest -k test_detect_forge -q` | pass | |
| 21 | badge verifie present | `cd web && npx vitest run --reporter=verbose -t "Verifie"` | pass | |
| 22 | deploy public redirect | `uv run pytest -k test_deploy_public_with_repo_url -q` | 400 | |
| 23 | SPDX new files | `grep -l SPDX packages/nexus-coordinator/src/nexus_coordinator/forge.py packages/nexus-coordinator/src/nexus_coordinator/provenance.py` | both found | |
| 24 | D-1 fix contrast | `grep -c "text-white/30" web/src/pages/BrowsedProject.tsx web/src/pages/ProjectDetail.tsx` | 0 instances in these specific lines | |
| 25 | G-1 fix PAD_R | `grep "_SVG_PAD_R = 32" packages/nexus-sdk/src/nexus_sdk/html_render.py` | found | |
| 26 | T41-T43 logged | `grep -c "T41\|T42\|T43" docs/shell/PATTERNS.md` | 3+ | |
| 27 | provenance_hash in Zod | `grep provenance_hash web/src/api/daemon.ts` | found | |
| 28 | DOMAIN_PROVENANCE_V1 | `grep DOMAIN_PROVENANCE_V1 crates/nexus-core-rs/src/canonical.rs` | found | |

---

## 11. Scope cuts

Copie du kickoff §6 :

- CPU watchdog iframe → Sprint 15
- Bridge push bidirectionnel → Sprint 15
- Runtime templates → Sprint 15
- Re-publish automatique sur repo update → Sprint 15
- Branding SBFB → Sprint 15
- Origin separee par subdomain → Sprint 15+
- 2 VPS supplementaires → Sprint 15
- MIME scan executables → Sprint 15
- Builds reproductibles → v1.2+
- Multi-writer iroh-docs → v1.1+
- Custom domain / DNS → v1.2+

---

## 12. Risques

| # | Risque | Mitigation |
|---|---|---|
| R1 | `git clone` en subprocess sur Windows : comportement different (pas de SIGTERM, chemins longs) | Utiliser `subprocess.run` avec `timeout` param (cross-platform), tester sur Windows |
| R2 | BLAKE3 pas dans la stdlib Python | Utiliser le package `blake3` PyPI ou deleguer au binding Rust `nexus_core` si expose. Fallback : hashlib SHA-256 avec note tech debt. |
| R3 | JCS en Python : package `jcs` peu maintenu | Implementation inline : `json.dumps(obj, sort_keys=True, ensure_ascii=False, separators=(',', ':'))` — suffisant pour notre schema plat (pas de floats, pas de nested complex). Documenter la decision. |
| R4 | Le redirect POST /project/deploy pour public casse les tests existants | Ajuster les tests existants dans la meme phase. Le test `test_deploy_public_with_repo_url_accepted` change de comportement attendu. |
| R5 | Taille du clone git > 500 MB pas verifiable avant la fin du clone | Check 1: `Content-Length` du HEAD HTTP (indicatif). Check 2: taille post-clone avant zip. Le timeout 30s protege contre les repos tres gros. |

---

## 13. Checkpoint de cloture

Le sprint est ferme quand :

1. 28/28 fail-fast checklist verts
2. 5 commits feat/docs landed sur master
3. `sprint14_verification.md` + `sprint14_audit_plan.md` ecrits
4. `docs/shell/PATTERNS.md` + `docs/rust/PATTERNS.md` a jour
5. Memory `nexus_grid_pivot.md` mise a jour avec tip et compteurs
6. Aucun `text-white/30` sur texte 11px dans BrowsedProject/ProjectDetail
7. `_SVG_PAD_R = 32` dans html_render.py
