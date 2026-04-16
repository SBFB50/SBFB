# Sprint 19 Phase E — nexus-phase-auditor review

**HEAD pre-commit** : `2fd6c60`
**Draft commit title** : `feat(sprint19): Phase E — pkarr relay self-hosted docker image + ops doc`
**Auditor** : nexus-phase-auditor (session 2026-04-16)
**Timebox** : ~10 min (incluant fixes inline pre-commit)

---

## Verdict : PASS (apres fix P1 + P2-E3 apportes inline)

0 P0, 1 P1, 3 P2, 3 P3+nit identifies. **P1 + P2-E3 corriges inline
avant commit** (Option A : fix en place). Signal rigor G4 satisfait
(3 P2+ documentes dont 2 carries reels S20). Commit autorise.

---

## Dimensions

### Security

- **SPDX headers** : 6/6 fichiers code ont
  `SPDX-License-Identifier: AGPL-3.0-or-later` (directement ou via
  commentaire HTML frontmatter pour les .md).
- **Secrets / credentials** : aucun secret hardcode. Seul usage
  injecte : `${{ secrets.GITHUB_TOKEN }}` et identite OIDC keyless.
- **Path traversal** : non applicable (pas de zip extract, pas de
  user-controlled filename).
- **Unsafe Rust** : non applicable (zero code Rust).
- **Loopback / wire sans peer-creds** : non applicable (aucun
  nouveau endpoint loopback SBFB).
- **JCS canonical bytes** : non applicable (pas de wire format
  JSON signe).
- **Container non-root** : Dockerfile cree user `pkarr` UID 10001
  et fait `USER pkarr`. `--cap-drop=ALL`, `--read-only`,
  `--security-opt=no-new-privileges` presents dans systemd unit
  `PKARR_RELAY_OPS.md §3.5`.
- **tini init** : present (`ENTRYPOINT ["/usr/bin/tini", "--",
  ...]`) — propage SIGTERM correctement.
- **Semgrep** : non installe, fallback grep patterns effectue sur
  `{TODO|FIXME|XXX}|unsafe|eval\(|os\.system\(|pickle\.load` — 0
  finding.

### Patterns

- `docs/rust/PATTERNS.md` : aucun pattern applicable (zero Rust).
  Pas de drift.
- `docs/shell/PATTERNS.md` : aucun pattern applicable (zero
  Python / React).
- **Pattern positif nouveau** : Phase E introduit le pattern
  d'image Docker signable keyless (cosign + SLSA + SBOM + Trivy).
  Pas encore formalise dans `docs/shell/PATTERNS.md`. Ajout
  envisageable en Phase F S19 si SBFB distribue d'autres images
  containerisees futures (worker, coordinator).

### Working tree audit (G5)

| Fichier | Categorie |
|---|---|
| `docker/pkarr-relay/Dockerfile` | PHASE |
| `docker/pkarr-relay/config.toml` | PHASE |
| `docker/pkarr-relay/README.md` | PHASE |
| `.github/workflows/build-pkarr-image.yml` | PHASE |
| `docs/release/PKARR_RELAY_OPS.md` | PHASE |
| `tests/ci-smoke/pkarr-relay-healthcheck.sh` | PHASE |
| `.planning/research/S19_phase_E_pkarr_relay_design.md` | CRAFT (exception same-session errata, accompagne Phase E) |

Untracked hors Phase E :
- NOISE : `cc.json`, `node_modules/`, `site/`, `test_libc.*`,
  `.claude/settings.local.json`, `.claude/worktrees/` — bug
  .gitignore inline comments documente memory, hors-scope S19.
- CRAFT (non stage, decision kickoff §10 item 6) :
  `docs/DND_P2P_DESIGN.md`, `docs/VISION_USE_CASES.md`,
  `docs/apps/` — laisses trainer jusqu'au sprint docs apps
  futur.

**Aucun NOISE stage**. 7 fichiers PHASE+1 CRAFT same-session.

### Scope-cuts verification

| Scope cut kickoff §6 | Status |
|---|---|
| Encryption at rest (S20) | 0 fichier touche ✓ |
| Duress PIN + panic wipe (S20) | 0 ✓ |
| Rate-limit per-consumer (S21) | 0 ✓ (le `[rate_limiter]` dans config.toml est le rate-limiter **upstream pkarr-relay**, pas le SBFB per-consumer S21) |
| Kudos-weighted admission (S22) | 0 ✓ |
| Structured output grammar (S20) | 0 ✓ |
| Client-side redaction (S21) | 0 ✓ |
| Federated ONG-run pkarr concrets (S22+) | Livre image+doc, PAS de deploy reel ni outreach. Conforme ✓ |
| ML-DSA-65 + ML-KEM-1024 (S26+) | 0 ✓ |
| Domain fronting + Tor bridges (S24-25) | 0 ✓ |
| `actions/checkout@v4` SHA pin | Workflow utilise `@v4` non-SHA-pinned. Carry documente (P2-E1). ✓ |

Aucun scope creep.

### Tests-delta

Annonce plan §9 Phase E : +1 (CI smoke docker build). Reel :

- Rust : 478 unchanged ✓
- SDK : 183 unchanged ✓
- coord : 187+3 unchanged ✓
- gov : 46 unchanged ✓
- Vitest : 239 unchanged ✓
- Playwright : 38 unchanged ✓
- size-limit : 7/7 unchanged ✓
- CI smoke : +1 fichier `tests/ci-smoke/pkarr-relay-
  healthcheck.sh` (bash, invoque via workflow GHA). Hors
  compteurs standards mais livre. ✓

Delta annonce vs reel : exact.

### Research-grounding

Diff `Cargo.toml`, `pyproject.toml`, `package.json` : 0
changement. Phase E est infra-as-docs, zero dependency bump.

APIs externes tracees (design doc §7 + §0 errata) :

| Ressource | Trace |
|---|---|
| `pkarr-relay ^0.11` | ✓ `§7.1` + errata `§0` (WebFetch lib.rs 2026-04-16 latest 0.11.5) |
| Routes upstream pkarr (`GET /`, `GET /:pubkey`, `PUT /:pubkey`) | ✓ errata `§0` (WebFetch relay/src/handlers.rs + lib.rs 2026-04-16) |
| Cosign keyless Fulcio/Rekor | ✓ `§7.10` (context7 `/sigstore/cosign`) |
| `docker/build-push-action@v5` SLSA+SBOM | ✓ `§7.7` |
| `actions/attest-build-provenance@v1` | ✓ `§4.6` |
| Trivy GHSA-69fq-xp46-6x23 (mars 2026) | ✓ `§7.5` |
| Caddy auto-HTTPS | ✓ `§7.6` + `§5.3` |
| Hetzner CX22 pricing | ✓ `§7.3` |

### Horizon long-terme + documentation amont (§6.7)

- Design doc present AVANT code : ✓
  `.planning/research/S19_phase_E_pkarr_relay_design.md` (1250+
  lignes, 6 alternatives strategiques §3.1-3.7, errata §0 post-
  fetch)
- D5 Day-0 cite 3 alternatives (k8s, compose, systemd raw) avec
  verdict ✓
- Solution la plus poussee : upstream-based (pas de fork), SLSA
  L2 cosign keyless + GitHub native attest, Trivy HIGH/CRITICAL
  fail-build, SBOM SPDX natif. ✓
- Aucune estimation LOC dans Phase E §8.2 : ✓ (la note "~2000
  LOC" design doc §3.2 decrit la taille du crate upstream,
  factuel, pas une estimation de livraison)

---

## Findings

- **P1 (fix inline applique)** — duplicate `env:` keys dans le
  step "Cosign keyless sign" de `.github/workflows/build-pkarr-
  image.yml`. YAML dedup le dernier bloc, `COSIGN_EXPERIMENTAL`
  n'etait pas defini au runtime. **Fix** : fusion des deux blocs
  `env:` en un seul (4 vars). Meme fix applique au step "Smoke
  healthcheck" (meme anti-pattern). Validation : `yaml.safe_load`
  OK post-fix.

- **P2-E1 carry (deja identifie)** — SHA pinning non-applique.
  `@v3/@v4/@v5` utilises pour parite avec `release.yml`,
  `rust-ci.yml` etc. du repo. Header comment du workflow note
  "carry S18 E3-2, repo-wide SHA sweep en sprint OpSec dedie".
  **Carry S20** `sprint19_audit_plan.md §track-E-1`.

- **P2-E2 carry (deja identifie)** — Caret version `^0.11` pour
  `cargo install pkarr-relay`. `--locked` assure reproductibilite
  single-build, pas cross-build. Entre 2 CI runs a 1 semaine
  d'ecart, 0.11.5 → 0.11.6 silent bump possible. Mitige par
  Trivy post-build + cosign signature per-image. **Carry S20**
  `sprint19_audit_plan.md §track-E-2` : pin version exacte au
  tag v1.0.

- **P2-E3 (fix inline applique)** — divergence doc design vs
  runbook sur rate-limit Caddy. `config.toml` original
  commentait "ONG sysadmin + Caddy limite deja" mais le
  `Caddyfile` livre dans `PKARR_RELAY_OPS.md §3.4` n'avait pas
  de bloc `rate_limit`. Caddy community OSS ne rate-limite pas
  nativement (besoin xcaddy + module). **Fix** : (a) commentaire
  `config.toml` corrige pour ne plus pretendre que Caddy limite ;
  (b) PKARR_RELAY_OPS.md §3.4 ajoute un paragraphe explicite
  "Rate-limiting" qui clarifie l'absence de rate-limit Caddy par
  defaut + procedure xcaddy optionnelle.

- **P3-nit-1 (fix inline applique)** — design doc §4.1 contenait
  encore un vestige "snapshot crates.io de `pkarr-relay 2.x.*`"
  du draft pre-errata. **Fix** : ligne 419 remplacee par
  "`pkarr-relay ^0.11` (latest 0.11.5)". Seule reference
  restante "2.*" est dans l'errata §0 elle-meme, intentionnelle
  (explique ce que le pre-errata draft avait faux).

- **P3-nit-2 (fix inline applique)** — design doc §4.5.1 YAML
  snippet avait deux cles `push:` top-level (non-fonctionnel en
  YAML — la 2eme ecrase la 1re). Le workflow reel livre est
  correct (une seule `push:` avec `branches`, `paths`, `tags`
  combines). **Fix** : snippet §4.5.1 aligne sur le workflow
  reel.

- **P3-nit-3 carry** — healthcheck depend de `GET /` route qui
  pourrait changer en `pkarr-relay 0.12.x`. Mitige par
  `triggers_revalidate` frontmatter `PKARR_RELAY_OPS.md`. Pas de
  fix requis S19.

- **P2-pkarr-audit carry (HLT-1)** — design doc §6.6 reconnait
  explicitement "R-pkarr-audit P2 — pas d'audit upstream pkarr
  public". Ce carry doit etre trace formellement dans
  `sprint19_audit_plan.md §track-E-pkarr-audit` pour que l'audit
  S20 Phase 0 le retrouve (plutot qu'enfoui dans le design doc).
  A ajouter en Phase F S19 wrap-up.

---

## Fixes appliques pre-commit (trace)

1. `.github/workflows/build-pkarr-image.yml:106-114` — merge 2
   blocs `env:` du step "Cosign keyless sign" en un seul (4 vars
   : `COSIGN_EXPERIMENTAL`, `REGISTRY`, `IMAGE_NAME`, `DIGEST`).
2. `.github/workflows/build-pkarr-image.yml:121-125` — meme fix
   preventif sur le step "Smoke healthcheck" (blocs env+run
   reorganises).
3. `docker/pkarr-relay/config.toml:6-10` — commentaire rate-
   limit corrige pour ne plus faux-impliquer Caddy.
4. `docs/release/PKARR_RELAY_OPS.md §3.4` — paragraphe "Rate-
   limiting" ajoute explicitant l'absence de rate-limit Caddy
   par defaut + procedure xcaddy optionnelle.
5. `.planning/research/S19_phase_E_pkarr_relay_design.md §4.1` —
   vestige `pkarr-relay 2.x.*` remplace par `^0.11 (latest
   0.11.5)`.
6. `.planning/research/S19_phase_E_pkarr_relay_design.md §4.5.1`
   — YAML snippet push: duplicate corrige pour aligner sur le
   workflow reel livre.

Re-verification post-fixes :
- `uv run python -c "import yaml; yaml.safe_load(...)"` → OK
- `bash -n tests/ci-smoke/pkarr-relay-healthcheck.sh` → OK
- `grep -c SPDX-License-Identifier` sur 6 fichiers → 6/6
- `grep -n "2\.x\.\*\|2\.\*"` design doc → 1 match (ligne 21
  errata §0, intentionnel)

---

## Recommendation

**Commit autorise apres fixes inline applique** (voir trace
ci-dessus). Le body commit phase inclut la section "Working
tree audit" et les 3 carry-overs P2 a logger dans
`sprint19_audit_plan.md` Phase F wrap-up :

- `carry-E1` : SHA pinning workflow actions (sprint OpSec dedie)
- `carry-E2` : pin version pkarr-relay `^0.11` → pin exact au
  tag v1.0
- `carry-E-pkarr-audit` : audit interne code upstream pkarr-
  relay (S22+)

Compteurs tests : 478 Rust / 183 SDK / 187+3 coord / 46 gov /
239 Vitest / 38 Playwright / 7/7 size / 246+ SPDX + 1 CI smoke
bash inchange par rapport a Phase D.
