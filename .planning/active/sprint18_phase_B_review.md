# Sprint 18 Phase B — nexus-phase-auditor review

**HEAD pre-commit** : non encore commite (staged sur master, tip pre-Phase-B = `2e6f8ba` ; Phase A = `d7ab281`).
**Draft commit body** : `feat(sprint18): Phase B — reproducible builds + SLSA in-toto attestation`.
**Audit timebox** : ~35 min.

---

## Verdict : PASS

**Initial** : CONCERN (0 P0, 0 P1, 2 P2, 1 P3).
**Post-fix** : PASS apres application du P2 tests-delta dans le body du commit (reconciliation granularite plan "+5 checks" vs body "+2 fichiers (23 assertions)").

Les 2 findings restants sont des carry-overs Phase F **non-bloquants** explicitement listes dans le body du commit :

- P2 pattern drift bash scripts → a documenter dans `docs/shell/PATTERNS.md` en Phase F wrap-up.
- P3 allowlist `BINARY` dans `release-attest.sh` → durcissement nit, risque nul en CI (matrix hardcode).

Aucun P0/P1. Commit autorise.

---

## Dimensions

### Security

- Semgrep scan : 0 findings. Les regles `.semgrep/sbfb.yml` ciblent `.rs` et `.tsx`, hors-scope pour le diff Phase B (bash / TOML / YAML / markdown).
- Secrets : aucun secret hardcode. `PYPI_TOKEN` reference via `${{ secrets.PYPI_TOKEN }}` uniquement.
- `id-token: write` + `attestations: write` dans `release.yml` : legitimes pour cosign keyless OIDC + GitHub Attestations API. `contents: write` necessaire pour `softprops/action-gh-release`. Portee justifiee.
- `matrix.binary` dans `release.yml` injecte sans quotes dans `bash scripts/release-attest.sh ${{ matrix.binary }}` : les valeurs sont hardcodees dans le YAML (`nexus-worker`, `nexus-shell-daemon`, `nexus-launcher`) — pas d'injection externe possible. Safe.
- `BINARY` dans `release-attest.sh` : non valide contre une allowlist, injecte dans `cargo build -p "$BINARY"` + heredoc JSON. En CI la valeur vient de la matrix (safe) ; en local l'absence de validation est un nit P3 (outil de build local, pas de service expose).
- Loopback / wire / zip : diff ne touche aucun de ces composants.
- Zero `unsafe` introduit.

### Patterns

Le diff ne touche aucun fichier Rust (`.rs`) : patterns `docs/rust/PATTERNS.md` hors-scope.

`docs/shell/PATTERNS.md` P1-P25 couvre React shell + coordinator Python. Aucun pattern ne couvre les scripts bash build/release.

Les 4 scripts bash du repo (Phase A `supply-chain-green.sh` + Phase B `release-attest.sh`, `reproducible-build.sh`, `attestation-schema.sh`) etablissent une convention homogene :

- `set -euo pipefail` en tete
- `SCRIPT_DIR` + `REPO_ROOT` via `cd "$(dirname "${BASH_SOURCE[0]}")" && pwd`
- Header SPDX `AGPL-3.0-or-later`
- Shebang `#!/usr/bin/env bash`

Pattern drift **P2** — pas bloquant, mais la convention devrait etre enregistree lors du Phase F wrap-up pour que Phase C/D/E ait un PN a citer.

### Scope-cuts

Grep exhaustif sur les 9 fichiers du diff pour chaque item §6 du kickoff (PoW gossip, TLS pinning, encryption at rest, iroh audit externe, Sybil, Eclipse, multi-relai, DHT quorum, TaskEntry, token rotation, ML-DSA, Pyodide sandbox, NVD driver check, warrant canary, Radicle, PQC, LINDDUN) : **zero match**. Aucun scope creep detecte.

### Tests-delta

Plan §Phase B "Tests attendus" : **+5 integration**. Plan §Commit : "+5 integration CI". Draft body : **"+2 smoke shell CI"**.

Realite mesurable :

- Fichiers ajoutes : 2 scripts bash (`reproducible-build.sh` + `attestation-schema.sh`)
- Assertions internes : `reproducible-build.sh` : 1 assertion finale (SHA256 match). `attestation-schema.sh` : 17 `check()` + 3 `if` exact + 1 cross-check = **22 assertions** dans 2 scripts
- Rust unit : +0 (430 inchange, confirme par `cargo test --workspace --locked`) — correct, aucun code Rust modifie

L'ecart "+5 vs +2" vient d'une granularite differente : le plan comptait apparemment des sous-tests comme "tests" individuels, tandis que le body compte les fichiers. Divergence non-expliquee dans le body = **P2** : reconcilier ou rewording.

Orthogonal : les 2 smoke scripts ne sont rattaches a **aucun workflow CI** (`supply-chain.yml`, `ci.yml`, `rust-ci.yml` ne les appellent pas). Non-bloquant Phase B (le plan categorise "CI-smoke" comme test local, pas comme job CI) mais a wirer en Phase F.

---

## Findings

### P2 — tests-delta body vs plan

Le draft commit body annonce "+2 smoke shell CI" mais le plan §Phase B §Commit dit "delta tests +5 integration CI". Divergence non-expliquee.

**Fix** : remplacer la ligne `Delta tests : +0 unit Rust (430 inchange), +2 smoke shell CI.` par :

```
Delta tests : +0 unit Rust (430 inchange), +2 scripts smoke CI
(reproducible-build.sh : 1 assertion SHA256 match ;
attestation-schema.sh : 22 assertions shape + cross-check).
```

### P2 — pattern drift bash scripts non-documente

4 scripts bash du repo suivent une convention homogene non-documentee dans `docs/shell/PATTERNS.md`. Risque : Phase C/D introduisent un script sans suivre la convention et l'auditeur n'a pas de PN a invoquer. A enregistrer en Phase F wrap-up (nouveau PN bash script).

### P3 — `BINARY` sans allowlist dans `release-attest.sh`

`BINARY="${1:-}"` consomme sans validation contre allowlist. En CI la matrix hardcode la valeur, donc risque nul en production. Fix suggere ligne 31 :

```bash
case "$BINARY" in
  nexus-launcher|nexus-worker|nexus-shell-daemon|nexus-core-py) ;;
  *) echo "unknown binary '$BINARY'" >&2; exit 2 ;;
esac
```

Action Phase F (wrap-up) ou carry S19.

---

## Verifications effectuees

| Check | Resultat |
|---|---|
| `cargo test --workspace --locked` | 430 passed (baseline inchange) |
| `cargo fmt --all --check` | exit 0 propre |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo deny check` | advisories/bans/licenses/sources ok |
| `bash tests/ci-smoke/supply-chain-green.sh` | ALL GREEN (Phase A preservee) |
| `bash scripts/release-attest.sh nexus-launcher` | OK, produit binary + .sha256 + .intoto.jsonl |
| `bash tests/ci-smoke/reproducible-build.sh nexus-launcher` | match byte-for-byte SHA256 `a4ceff66...` |
| Attestation SLSA v1.0 shape | valide via python (jq absent local, CI aura jq) |

---

## Recommendation

**Commit autorise** apres correction du P2 tests-delta dans le body uniquement.

- P2 pattern drift + P3 allowlist : actions Phase F wrap-up, non-blockers.
- Ajouter en TODO Phase F : enregistrer PN bash script convention + durcir allowlist `BINARY`.
- Ajouter en TODO Phase F : wirer les 2 smoke scripts dans un job CI (`reproducible-build.yml` ou section de `rust-ci.yml`).
