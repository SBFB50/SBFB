# Sprint 21 Phase E — preflight G8

Date : 2026-04-19
HEAD : `f830579`
Verdict : **SCOPE-CUT-CONSISTENT**

Phase cible : **Tech debt batch** (E-1 canary JCS migration + E-2
CanaryRegistry verify Ed25519 at ingest + E-3 C-PLAN-1 plan docs
fix S20 wire-point + E-4 PATTERNS.md tech debt entries update),
plan §8 lignes 681-739.

---

## 1. Resume verdict

Day 0 D5 cap G7 carry-overs respectee — Phase E ferme 2/3 carries
P2 actifs S20 (T-NN canary JCS + T-NN+1 registry verify Ed25519,
laisse Meta-1 Radicle re-carry S22 et T-NN+2 iframe Rust-wasm
S22+ blocked). Trois findings non-bloquants detectes : binding
PyO3 `verify_canary` manquant a creer inline (plan §8.1 E-2
assume existant), `nexus-core-py` doit ajouter `nexus-shell-
daemon-core` path dep, et migration `canary_wire_bytes` invalide
toute signature canary historique (couvert pre-launch policy).
Procede Phase E, commit feat single (cohérent §8.4 cible).

---

## 2. Scans

### S1 — SOTA 2026 vs design

Libs/deps scannes :

- `serde_jcs` — **deja workspace** (`crates/nexus-worker-core/
  Cargo.toml`, `crates/nexus-core-rs/Cargo.toml`, `crates/nexus-
  core-rs/src/canonical.rs`, `crates/nexus-core-rs/src/task.rs`,
  `crates/nexus-worker-core/src/invite.rs`). Migration E-1 = 1-
  line change `serde_json::to_vec → serde_jcs::to_vec` dans
  `crates/nexus-shell-daemon-core/src/canary/mod.rs:241-243`.
  Pas de nouvelle dep externe.
- `nexus-shell-daemon-core` (path dep workspace) — sera ajoute a
  `crates/nexus-core-py/Cargo.toml` pour exposer le binding
  `verify_canary`. Path dep interne, zero impact externe.
- `pyo3` 0.28 — deja workspace, aucune nouvelle API touchee.
- `aws-lc-rs` Ed25519 / `nexus-core-rs::verify` — deja exporte
  via `verify_bytes` PyO3 (`crates/nexus-core-py/src/lib.rs:1112-
  1128`), zero changement.

CVE / advisory check : aucune zone rouge applicable
(R-wasmtime-cve / R-iroh-audit / R-libcrux-hax / R-pyodide-escape
hors-scope canary verify Ed25519 + JCS canonical).

**Verdict S1 : clean.**

### S2 — Decisions historiques traversees

Commandes lancees :

```bash
git log --all --oneline --grep="canary_wire_bytes|serde_jcs|canary.*verify.*ingest"
grep -rE "canary|verify" .planning/archive/v1.2/sprint20_*.md
```

**Decisions historiques pertinentes** :

- `6a3f199` (S20 Phase E) — federation foundations livree avec
  **deux tech debts deliberes documentes** :
  - `canary_wire_bytes` reste `serde_json::to_vec` non-canonical
    (T-NN, P2 carry actif S20 audit memory tip)
  - `CanaryRegistry::observe_canary` accepte sans verify Ed25519
    at ingest (T-NN+1, P2 carry "delibere-mais-documente" cf.
    nexus_grid_pivot.md "registry sans verify Ed25519 at ingest
    delibere-mais-documente").
- `04c9621` (S18 Phase E2) — interdit auto-publish scheduler
  signing canary. **Phase E ne signe PAS** (verify only at
  ingest). Conforme.
- `f209168` + `66a3a7c` (S20 Phase F + audit gate) — confirment
  les 2 carries P2 attendent fermeture sprint suivant. Phase E
  S21 = ce sprint suivant.
- `1c1fcfb` (S4 Day 0) — RFC 8785 JCS canonical adopte project-
  wide pour les Task/Result/Claim signing. **Canary etait l'odd-
  one-out** depuis S18 (serde_json wire). E-1 aligne canary sur
  pattern projet.

**Reverse-commit checks** : aucun commit n'a reverte les decisions
T-NN / T-NN+1. Carries actifs jusqu'a Phase E S21.

**Findings drift plan-vs-code (non-bloquants)** :

#### Finding S2-E1 — `verify_canary` PyO3 binding manquant

Plan §8.1 E-2 ligne 615-617 specifie :

> Modifier `packages/nexus-coordinator/src/nexus_coordinator/
> canary_registry.py` `POST /api/canary/observed` handler pour
> verify Ed25519 signature at ingest via `nexus-core-py`
> `verify_canary` binding.

Inventaire `crates/nexus-core-py/src/lib.rs` lignes 1145-1162 :

```
sign_task / verify_task_entry
sign_result / verify_result_entry
sign_claim / verify_claim_entry
sign_curator_list / verify_curator_list_entry
sign_bytes / verify_bytes (generic Ed25519 raw)
```

**Pas de `verify_canary` exporte.** Plan §8.1 E-2 assume binding
existant. Realignement : ajouter le binding inline Phase E,
pattern miroir `verify_task_entry(entry_json: &str) -> PyResult
<()>` qui parse JSON + appelle la fonction Rust correspondante.

**Reverse-commit check** : `git log --all -- crates/nexus-core-
py/src/lib.rs` ne montre aucune commit ayant cree puis supprime
`verify_canary`. Le binding n'a jamais existe.

Classification : drift **plan-vs-code naming/path** (le binding
n'existe pas mais doit exister selon le plan). Realignement =
creation inline binding, pas pivot Day 0.

#### Finding S2-E2 — `nexus-core-py` ne depend pas de `nexus-shell-daemon-core`

Inventaire `crates/nexus-core-py/Cargo.toml [dependencies]` :

```
nexus-core-rs = { path = "../nexus-core-rs" }
nexus-worker-core = { path = "../nexus-worker-core" }
pyo3, pyo3-async-runtimes, tokio, iroh-docs, serde_json,
futures-lite, tracing
```

**Pas de `nexus-shell-daemon-core` (ou le canary type vit
`crates/nexus-shell-daemon-core/src/canary/mod.rs`).** Pour
exposer `verify_canary`, le binding doit pouvoir importer
`nexus_shell_daemon_core::canary::{Canary, verify_canary}`.

Realignement : ajouter `nexus-shell-daemon-core = { path =
"../nexus-shell-daemon-core" }` aux `[dependencies]`. Path dep
interne workspace, zero impact externe (pattern miroir
`nexus-worker-core` deja path dep pour exposer mint/decode_invite).

Classification : drift **plan-vs-code dependency graph**.
Realignement trivial.

### S3 — Threat model coverage

HARDENING_ROADMAP `docs/security/HARDENING_ROADMAP.md` :

- E-1 (canary JCS migration) : aligne canary sur le pattern
  RFC 8785 JCS deja adopte project-wide (Task/Result/Claim
  S4 `1c1fcfb`). **Aucune regression** — au contraire, ferme un
  gap d'inconsistance interne.
- E-2 (verify Ed25519 at ingest) : **hardening** signature
  verification avant observation registry. Mitigation directe
  pour scenario "peer malicieux publie canary forgee + observers
  l'enregistrent en local mauvaise pubkey trust". S20 Phase E
  doc `WARRANT_CANARY_HARDENING.md` mentionne ce gap. Phase E
  ferme.
- E-3 (plan docs fix) : zero impact threat (doc archive
  correction).

**Pre-requirement HARDENING_ROADMAP §3 ligne S21** : aucune
mention specifique des items Phase E. Hors-scope HARDENING
roadmap, c'est du tech debt cleanup interne. Pas de pre-req
manquant.

**Verdict S3 : clean.**

### S4 — Wire format / pre-launch invariants

Scan `_VERSION` :

- `BLOB_VERSION = 0x01`, `TASK_RESPONSE_VERSION = 1`,
  `CANARY_VERSION = 1`, `ANNOUNCEMENT_VERSION = 1` — **tous
  inchanges Phase E**.
- `DOMAIN_WARRANT_CANARY_V1` constant — preserve
  (`crates/nexus-shell-daemon-core/src/canary/mod.rs:234`
  `canonical_bytes(&canary.signed, DOMAIN_WARRANT_CANARY_V1)`).

#### Finding S4-E3 — `canary_wire_bytes` migration invalide signatures historiques

`canary_wire_bytes()` ligne 241-243 actuel :

```rust
pub fn canary_wire_bytes(canary: &Canary) -> Result<Vec<u8>, CanaryError> {
    serde_json::to_vec(canary).map_err(|e| CanaryError::Canonical(e.to_string()))
}
```

Migration E-1 vers `serde_jcs::to_vec(canary)` change l'ordering
des fields + whitespace + unicode escapes. **Si un canary etait
sign sur le reseau avec les bytes serde_json**, sa signature ne
vérifierait plus.

**MAIS** :

1. `verify_canary()` Rust ligne 226-237 utilise `canonical_bytes
   (&canary.signed, DOMAIN_WARRANT_CANARY_V1)` (JCS canonical),
   **pas** `canary_wire_bytes()`. Donc les signatures **n'ont
   jamais utilise** `canary_wire_bytes()` — c'est un format
   gossip transport, pas un format signing.
2. Pre-launch protocol policy : aucun canary publish prod, donc
   meme si signing avait utilise `canary_wire_bytes`, zero
   impact reseau.

L'observation : `canary_wire_bytes` est utilise pour **gossip
broadcast** (ligne 351-357) — la transport layer serializes le
canary entier. Migration vers JCS donne un wire format byte-
identical cross-language (Python observation re-serializes
identique). Test cross-language E-1 valide cette propriete.

Classification : **non-bloquant** (pre-launch + signing
inchangee). A documenter inline le rationale.

**Verdict S4 : clean** (1 finding non-bloquant documente).

---

## 3. Findings consolides

| ID | Scan | Type | Severite | Action |
|---|---|---|---|---|
| S2-E1 | S2 | `verify_canary` binding manquant | Non-bloquant | Creer inline `crates/nexus-core-py/src/lib.rs` + register `m.add_function` |
| S2-E2 | S2 | `nexus-core-py` sans dep `nexus-shell-daemon-core` | Non-bloquant | Ajouter path dep `[dependencies]` |
| S4-E3 | S4 | `canary_wire_bytes` migration invalide sigs historiques | Non-bloquant | Pre-launch + signing-inchangee couvert ; documenter inline |

**0 finding bloquant** (S1, S3 clean ; S2 = 2 findings drift dep
graph + binding ; S4 = 1 finding pre-launch acceptable).

**Regle d'agregation §6 G8** : `0 bloquant + ≥1 non-bloquant` →
**SCOPE-CUT-CONSISTENT**.

---

## 4. Action

### 4.1 Pas de commit chore prealable

Contrairement a Phase D, les findings Phase E sont **dans le
scope code Phase E lui-meme** (creer le binding + ajouter dep =
parties integrales d'E-2). Pas de plan §8 a re-aligner — le plan
est juste **incomplet** (assume binding existant), Phase E
implementation l'absorbe naturellement.

Le preflight.md est livre dans le commit feat Phase E (categorie
CRAFT du working tree audit) ou commit chore separe minimal si
preferable. Choix executor : inclu dans commit feat pour
cohérence atomique.

### 4.2 Commit feat Phase E (single commit per §8.4 default)

Implementer dans cet ordre logique :

1. **`crates/nexus-shell-daemon-core/src/canary/mod.rs:241-243`** —
   E-1 migration `serde_json::to_vec → serde_jcs::to_vec`. Ajouter
   `serde_jcs` à `Cargo.toml [dependencies]`. Test Rust cross-
   language : `crates/nexus-shell-daemon-core/src/canary/mod.rs`
   tests block — `test_wire_bytes_is_jcs_canonical_cross_language`
   asserts les bytes match les bytes serializes par
   `serde_jcs::to_vec` (compare aussi a un snapshot byte-array
   hardcoded pour catch unintended drift).
2. **`crates/nexus-core-py/Cargo.toml`** — Add path dep
   `nexus-shell-daemon-core = { path = "../nexus-shell-daemon-
   core" }`. Pattern miroir `nexus-worker-core`.
3. **`crates/nexus-core-py/src/lib.rs`** — Add `fn verify_canary
   (canary_json: &str) -> PyResult<()>` PyO3 export. Pattern
   miroir `verify_task_entry` ligne 868-875. Register in
   `#[pymodule]` ligne 1145-1163.
4. **Rebuild wheel** :
   `unset CONDA_PREFIX CONDA_DEFAULT_ENV && \
    VIRTUAL_ENV=$PWD/.venv maturin develop --release \
      --manifest-path crates/nexus-core-py/Cargo.toml`
5. **`packages/nexus-coordinator/src/nexus_coordinator/api/
   canary.py`** — Modifier `POST /api/canary/observed` handler
   pour verify Ed25519 at ingest via `nexus_core.verify_canary
   (json.dumps(payload))` AVANT `coord.canary_registry.observe_
   canary(obs)`. Reject 401 si verify rate. Tests +2 Python
   coord :
   - `test_canary_registry.py::test_observed_endpoint_rejects_
     malformed_signature` — POST canary avec sig forge → 401.
   - `test_canary_registry.py::test_observed_endpoint_accepts_
     valid_canary` — POST canary signed via Rust → 200 + observed.
6. **`.planning/archive/v1.2/sprint20_plan.md`** — E-3 edit
   §6.2 + §6.4 note correction wire-point `runtime.rs::spawn_
   gossip_subscribe_task` (vs draft `iroh_runtime.rs::Gossip
   Client::subscribe`).
7. **`docs/rust/PATTERNS.md`** — E-4 add 3 entries tech debt :
   - T-NN canary JCS — **resolu Phase E S21** `<commit-sha>`.
   - T-NN+1 CanaryRegistry verify Ed25519 — **resolu Phase E S21**
     `<commit-sha>`.
   - T-NN+2 iframe Rust-wasm realignement Option G — **ouvert
     S22+ blocked** (tract opset 19 / ort wasm32-browser /
     gline-rs wasm-bindgen).

Tests delta visee : **+1 Rust** (canary JCS cross-lang) **+ 2
Python coord** (registry verify) = **+3 tests** (cohérent §8.2).

### 4.3 Commit cible Phase E (single feat, §8.4 default)

```
feat(sprint21): Phase E — tech debt batch (canary JCS + registry verify Ed25519 + plan docs fix)
```

Body inclut working tree audit + categorie CRAFT preflight.md +
findings P2/P3 review.

### 4.4 Carry-overs S22

- **Meta-1 Radicle-v1.0 activation tracking** : re-carry S18→
  S19→S20→S21→**S22**. Toujours pre-v1.0 go-live deferred.
- **T-NN+2 iframe Rust-wasm realignement Option G** : reste
  ouvert PATTERNS.md S22+ (blocked tract/ort/gline-rs).
- Aucun nouveau carry tech debt cree par Phase E (au contraire,
  ferme 2 carries).

---

## 5. Garde-fous §6 G8 verifies

- [x] **Evidence-based** : 3 findings sources sur grep code +
      `crates/nexus-core-py/src/lib.rs` lignes precises +
      Cargo.toml dep graph + plan §8 references precises
- [x] **Day 0 respect** : D5 cap carry-overs respecte (Phase E
      ferme 2 carries dans le cap, Meta-1 re-carry conforme),
      pas de pivot D1..D4
- [x] **Wire format** : `*_VERSION = 1` pre-launch policy
      intacte, signing canary inchangee (canonical_bytes JCS
      preserve), seul wire transport `canary_wire_bytes` migre
      JCS pour byte-identity cross-lang
- [x] **Test budget cap** : +3 tests = identique au plan §8.2,
      pas d'expansion
- [x] **Theme sprint** : Phase E = tech debt batch S20 explicit
      dans kickoff §1 ligne 1-15 « rate-limit + PII SDK + output
      filter + quarantine queue + tech debt batch »
- [x] **Pas YAGNI** : ferme 2 carries P2 actifs (T-NN + T-NN+1)
      avec cas d'usage existant (canary federation observable +
      registry trust hardening)
- [x] **Retrospective trackee** : findings ajoutes
      `sprint21_audit_plan.md` (track Phase E realignement
      binding nexus-core-py)

**Tous garde-fous green** — verdict SCOPE-CUT-CONSISTENT
definitif.
