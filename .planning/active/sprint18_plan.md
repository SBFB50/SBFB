# Sprint 18 — Plan detaille (quick wins + supply chain baseline + multi-relai phase 1)

**Ecrit** : 2026-04-15 (session fraiche post-S17 wrap-up).
**Tip d'entree** : `4f0727b` (audit-P1 S17 docs-only).
**Phase 0** : **DEJA JOUE** session 2026-04-14 — verdict PASS
apres 1 commit `4f0727b`. Ne pas rejouer. Voir
`sprint18_kickoff.md §3` pour le detail et
`.planning/archive/v1.2/sprint17_audit_findings.md` pour les 7 P1
resolus.

**Sprint 18 phases attendues** : **6 phases A-F** (Phase E
decoupee en 3 subphases E1/E2/E3 avec commits distincts),
~2250 LOC total (~1460 code + ~350 tests + ~440 docs/config),
+50-60 delta tests. Retour au code apres S17 recherche.

---

## Vue d'ensemble

| Phase | Goal | LOC code | LOC tests | LOC docs | Commits |
|---|---|---|---|---|---|
| 0 | Audit Sprint 17 | 0 | 0 | 0 (migre) | `4f0727b` (pre-S18) |
| A | Supply chain CI baseline | ~250 | ~30 | ~20 | 1 : `feat(sprint18): Phase A — supply chain CI (cargo-deny + pip-audit + npm audit + wasmtime pin)` |
| B | Reproducible builds + SLSA | ~150 | ~20 | ~80 | 1 : `feat(sprint18): Phase B — reproducible builds + SLSA in-toto attestation` |
| C | Multi-relai + DHT redundant | ~450 | ~130 | ~20 | 1 : `feat(sprint18): Phase C — multi-relai federation + DHT redundant lookup` |
| D | Wire TaskEntry + token rotation | ~280 | ~100 | ~20 | 1 : `feat(sprint18): Phase D — coord-side TaskEntry wire-through + X-SBFB-Token rotation` |
| E1 | Driver check NVD | ~200 | ~40 | ~10 | 1 : `feat(sprint18): Phase E1 — NVIDIA driver CVE check at launcher startup` |
| E2 | Warrant canary | ~100 | ~30 | ~20 | 1 : `feat(sprint18): Phase E2 — warrant canary monthly Ed25519 gossip publish` |
| E3 | Radicle mirror | ~30 | 0 | ~20 | 1 : `feat(sprint18): Phase E3 — Radicle mirror GitHub Action` |
| F | Wrap-up + verif + audit plan S19 | 0 | 0 | ~250 | 1 : `chore(sprint18): Phase F — wrap-up + verification + audit plan S19 + migrate planning` |
| **Total S18** | | **~1460** | **~350** | **~440** | **8 commits + 1 chore(planning) ouverture** |

Plus 1 `chore(planning): close S17 + open Sprint 18` en day-0
(migration `sprint17_audit_findings.md` + ajout kickoff+plan S18).

---

## Ordre des phases (justifie)

1. **Phase A first** : installer CI gates supply-chain AVANT toute
   lander de code. Une PR Phase B-E qui introduirait un CVE critique
   serait automatiquement bloquee. Coherence defense-first.
2. **Phase B apres A** : reproducible builds + SLSA utilisent les
   outils validates par Phase A (cargo-deny passed). Artefact
   Gate 1 livrable en fin S18 requiert B.
3. **Phase C parallel-able avec D** mais sequencee ici car C
   touche iroh endpoint construction + discovery (risk regression
   plus large), mieux landed stand-alone. D est surface narrow
   coord+token.
4. **Phase D apres C** : D touche TaskEntry wire (coord emet vers
   daemon → worker → consume). Si C break endpoint, D tests E2E
   deviennent flaky. Ordre lineaire plus sur.
5. **Phase E1 E2 E3 apres D** : tous independants du core runtime,
   ops-facing. Peuvent etre landes dans n'importe quel ordre
   interne ; on fixe E1 > E2 > E3 pour commit log readability.
6. **Phase F last** : consolidation standard sprint closing.

**Dependencies hard** :
- B depend A (cargo-deny green requis pour artefact Gate 1 signed)
- C depend aucune (module P2P standalone)
- D depend C testing-wise (TaskEntry flow E2E necessite endpoint
  working, mais D lui-meme ne change pas endpoint)
- E1 E2 E3 depend aucune technique, depend F logique (F cloture)

---

## Phase 0 — Audit Sprint 17 (DONE pre-S18)

### Status : JOUE session 2026-04-14

Session fraiche a lu `.planning/archive/v1.2/sprint17_audit_plan.md`
et execute tracks A-G. Findings dans
`.planning/active/sprint17_audit_findings.md` (en cours de migration
→ `archive/v1.2/` avec ce 1er commit S18).

### Verdict

**CONDITIONAL PASS** initial (0 P0, 7 P1, 19 P2, 13 P3) →
**PASS** apres 1 commit fix docs-only :

```
4f0727b fix(sprint17): audit-P1 — resolve 7 findings from S18 Phase 0 audit
```

7 P1 fermes :
- **G-1** : 3 stubs RELEASE_GATES + PARTNERSHIPS + DISCLOSURE
  expliquant le scope-cut Phase E
- **D-1** : `HARDENING_ROADMAP §7` table Gate 3 → S29 (au lieu
  de S27) pour self-consistency audit externe
- **A-1** : mapping Tier T4 explicite dans Gate 3 partial (Tor
  S25 + mixnet futur)
- **A-2** : standardisation symboles ❌/⚠️/✅ scenarios ×
  HARDENING_ROADMAP matrix
- **B-1** : reclass Sybil Tier max T2+ pre-S19, T5 post-S19 PoW
  + S22 kudos-weighted
- **C-1** : attribution paper Carlini 2024 (pas Tramer)
- **E-1** : disambiguation iroh-gossip vs libp2p-gossipsub CVE

Les 19 P2 loggees comme dette docs. Les 13 P3 sans action. Pas de
code touche.

### Sprint 18 Phase A non-bloquee

Le commit `4f0727b` est le tip d'entree pour Sprint 18. Aucun
fix supplementaire requis.

---

## Phase A — Supply chain CI baseline (~300 LOC)

### Goal

Installer les 4 garde-fous CI qui bloquent toute PR introduisant
un CVE critical upstream avant landing :
1. `cargo-deny` (Rust workspace) — advisories + bans + licenses + sources
2. `pip-audit` (Python workspaces `packages/*`) — PyPI advisories
3. `npm audit` via `audit-ci` (web/) — npm advisories
4. **wasmtime pin** `>=43.0.1, <44` (via `cargo-deny [bans]`) —
   mitigation CVE-2026-34941 + CVE-2026-34946 (12 CVE avril 2026)

### Decisions techniques

- **cargo-deny > cargo-audit** : cargo-deny englobe advisories
  RUSTSEC + bans/licenses/sources en 1 tool. Source :
  [RustSec recommends cargo-deny-action](https://rustsec.org/)
- **Severity threshold** : `critical` → CI fail (block PR),
  `high` → warn annotation GitHub, `moderate`/`low` → silent
  (report only)
- **Weekly schedule** additionnel : re-audit sur cron meme si no
  PR, capture nouvelles advisories upstream publiees

### Livrables

**Nouveaux fichiers** :

- `deny.toml` racine workspace Rust :
  ```toml
  [advisories]
  db-path = "~/.cargo/advisory-db"
  db-urls = ["https://github.com/rustsec/advisory-db"]
  yanked = "deny"
  ignore = []  # empty S18 start

  [bans]
  multiple-versions = "warn"
  wildcards = "allow"  # tighten S19+
  deny = [
    { name = "wasmtime", version = "<43.0.1" },  # R-wasmtime-cve
  ]

  [licenses]
  allow = ["Apache-2.0", "MIT", "BSD-3-Clause", "BSD-2-Clause",
           "ISC", "Unicode-DFS-2016", "Unicode-3.0", "Zlib",
           "CC0-1.0"]
  confidence-threshold = 0.8

  [sources]
  unknown-registry = "deny"
  unknown-git = "deny"
  allow-git = []
  ```

- `.github/workflows/supply-chain.yml` :
  - 3 jobs paralleles `cargo-deny` + `pip-audit` + `npm-audit`
  - Trigger : `pull_request` + `schedule` cron weekly Monday 08:00 UTC
  - Each job : checkout + setup tools + run audit + fail on critical

- `web/audit-ci.json` :
  ```json
  {
    "critical": true,
    "report-type": "summary",
    "registry": "https://registry.npmjs.org/"
  }
  ```

**Fichiers modifies** :

- `pyproject.toml` (packages/nexus-coordinator, nexus-sdk,
  nexus-app-gov) : ajout dev-dep `pip-audit = "^2.9"`
- `web/package.json` : ajout devDep `audit-ci` + script
  `"audit:ci": "audit-ci --config audit-ci.json"`
- `docs/security/README.md` : section "Supply chain CI" (~20 lignes)
  + pointeur `deny.toml`

### Tests attendus (+10 ops)

Pas de tests unit runtime (les CI guards sont de l'ops, testes
par leur propre execution reussie). Tests integration :

- `tests/ci-smoke/supply-chain-green.sh` : execute les 3 audits
  en local, assert exit 0 sur master propre
- Dry-run GitHub Action via `act` (local) → ok

### Critere d'acceptation Phase A

- [ ] `cargo deny check --workspace` exit 0 sur master
- [ ] `cd packages/nexus-coordinator && uv run pip-audit` exit 0
- [ ] `cd web && npm run audit:ci` exit 0
- [ ] PR test-introducing-wasmtime-old (`wasmtime = "42.0.0"`) →
  cargo-deny fail avec message `denied crate wasmtime version
  42.0.0 < 43.0.1`
- [ ] Workflow `supply-chain.yml` run verte dans GitHub Actions

### Risques Phase A

- **False positives RUSTSEC** sur deps transitives unavoidable
  (ex: `RUSTSEC-YYYY-NNNN affecting crate-X via crate-Y`) →
  policy : investiguer chaque, si pas exploitable dans notre
  path → ajout dans `[advisories] ignore` avec reason explicite.
  **Acceptable** : 0-2 ignores S18 open, re-evalue S19+.
- **License violation** sur dep existant : si une crate ayant
  `GPL-3.0` apparait, policy stricte (on est AGPL-3.0 mais
  workspace `licenses.allow` doit etre coherent). **Fix** :
  swap la crate si incompatible, ou add to allow si coherent
  AGPL.

### Commit

`feat(sprint18): Phase A — supply chain CI (cargo-deny + pip-audit + npm audit + wasmtime pin)`

Body : liste deny.toml config sommaire, workflow supply-chain
3 jobs, wasmtime ban rule reference CVE-2026-34941 +
CVE-2026-34946 (Bytecode Alliance advisory 9 avril 2026), delta
tests 0 runtime / +1 CI job. Gate 1 prerequis 1/4 cleared.

---

## Phase B — Reproducible builds + SLSA in-toto attestation (~250 LOC)

### Goal

Chaque binary release (launcher, worker, shell-daemon, wheel
nexus-core-py) est :
1. **Reproductible** : 2 builds sur meme `SOURCE_DATE_EPOCH` →
   SHA256 identique
2. **Attested** : `.intoto.jsonl` SLSA provenance format v1 signe
   publie avec l'artefact

### Decisions techniques

- **Cargo deja 100% reproducible out-of-the-box** (hard-codes
  timestamps dans archive metadata, source : WebSearch
  `slsa.dev/spec/v1.0/faq` + `reproducible-builds.org`). Les
  seules sources de non-determinisme restantes :
  `CARGO_INCREMENTAL` (compilation cache), `SOURCE_DATE_EPOCH`
  (pour build scripts ayant des timestamps), target-cpu specific
  codegen.
- **SLSA v1.0 provenance format** : JSON in-toto
  `predicateType: https://slsa.dev/provenance/v1` avec
  `buildDefinition.resolvedDependencies` (hash Cargo.lock) +
  `runDetails.builder` + `runDetails.metadata`
- **SHA256 attestation** : incluse dans `subject[].digest.sha256`
  de chaque artefact dans le predicate

### Livrables

**Nouveaux fichiers** :

- `.cargo/config.toml` (workspace root) :
  ```toml
  [build]
  incremental = false  # reproducible default
  
  [env]
  # SOURCE_DATE_EPOCH injected by release script, not hardcoded
  ```

- `scripts/release-attest.sh` (~100 LOC bash) :
  - arg `$1` = target binary name (launcher/worker/daemon/wheel)
  - Export `SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)`
  - `cargo build --release --locked -p $BINARY`
  - Compute `sha256sum target/release/$BINARY > $BINARY.sha256`
  - Emit `$BINARY.intoto.jsonl` avec schema SLSA provenance v1
  - Optionnel : sign avec `cosign` si `COSIGN_KEY` present (env)

- `.github/workflows/release.yml` (new section ou existant
  update) :
  - Matrix build (linux/macos/windows × x86_64/arm64)
  - Chaque job : run `release-attest.sh` pour chaque binary
  - Upload artefacts + attestations en GitHub Release assets
  - Step `cosign attest` si `GITHUB_TOKEN` + OIDC configured
    (keyless signing via Sigstore)

- `docs/release/REPRODUCIBLE_BUILDS.md` (~80 lignes) :
  - §1 How to verify SHA256 (downloads artifact + sha256sum)
  - §2 How to verify SLSA provenance
    (`cosign verify-attestation` + `slsa-verifier`)
  - §3 How to rebuild locally deterministic
    (`SOURCE_DATE_EPOCH=X CARGO_INCREMENTAL=0 cargo build
    --release --locked`)

**Fichiers modifies** :

- `Cargo.toml` workspace : ajout `[profile.release]` keys
  deterministic : `lto = "fat"`, `codegen-units = 1`,
  `strip = "debuginfo"`, `debug = false`
- `docs/security/README.md` : pointeur vers
  `docs/release/REPRODUCIBLE_BUILDS.md`

### Tests attendus (+5 integration)

- `tests/ci-smoke/reproducible-build.sh` : run 2x build avec
  meme `SOURCE_DATE_EPOCH` → compare SHA256 identique
- `tests/ci-smoke/attestation-schema.sh` : validate
  `.intoto.jsonl` contre schema SLSA v1.0 (via `jsonschema` CLI)
- Unit-level Rust : impossible (bash scripts + JSON). Test via
  CI-smoke.

### Critere d'acceptation Phase B

- [ ] `./scripts/release-attest.sh nexus-launcher` produit
  `.sha256` + `.intoto.jsonl` valide schema SLSA v1.0
- [ ] 2 invocations successives SOURCE_DATE_EPOCH fixe → SHA256
  identical
- [ ] GitHub Actions `release.yml` dry-run (manual trigger) →
  artefacts + attestations publies en release draft

### Risques Phase B

- **Cross-platform reproducibility** : build Linux vs Windows
  produit SHA differents (target-specific linker). Acceptable :
  attestation par platform, verification per-platform.
- **Cranelift codegen non-determinism** : rare mais documente.
  Mitigation : `codegen-units = 1` force single-thread codegen.
- **Cosign keyless OIDC setup** : requires GitHub Actions OIDC
  enabled. Fallback : HMAC secret si pas OIDC.

### Commit

`feat(sprint18): Phase B — reproducible builds + SLSA in-toto attestation`

Body : deterministic build profile, release-attest.sh script
schema SLSA v1.0, cosign keyless via GitHub OIDC, doc verification
user-facing, delta tests +5 integration CI. Gate 1 prerequis 2/4
cleared.

---

## Phase C — Multi-relai federation + DHT redundant lookup (~600 LOC)

### Goal

1. **Multi-relai phase 1** : iroh `Endpoint` accepte relays custom
   via `RelayMode::Custom(RelayMap)` depuis config file ou env
   var. Fallback vers `prod::default_relay_map()` (3 relais n0
   NA/EU/AP) si pas de custom.
2. **DHT pkarr redundant** : lookup d'un `NodeId` via 3 pkarr
   relays en parallele, quorum 2/3 pour accepter le record
   (mitigation Eclipse-by-DHT single-point-of-failure).

### Decisions techniques

- **API iroh 0.97** : `RelayMode::Custom(RelayMap)` existe deja
  (source context7 `/websites/rs_iroh`). Pas besoin de patcher
  iroh upstream, juste construire `RelayMap` depuis config et
  injecter dans `Endpoint::builder()`.
- **Config file** : `~/.sbfb/relays.json` avec schema :
  ```json
  {
    "relays": [
      { "url": "https://relay.example.org", "quic": true }
    ]
  }
  ```
- **Env var override** : `SBFB_CUSTOM_RELAYS` = comma-separated
  URLs (precedence > config file). Patern `12-factor app`.
- **DHT quorum pattern custom** : pas de built-in pkarr (source
  WebSearch `pkarr crate 2026`). On code `redundant_resolve()`
  qui lance 3 lookups paralleles (`tokio::select!` avec join_all)
  et accepte si ≥2/3 retournent meme record (Ed25519 signature
  verif identique).

### Livrables

**Nouveaux fichiers** :

- `crates/nexus-core-rs/src/relay_config.rs` (~120 LOC) :
  - `pub fn load_relay_map() -> Result<RelayMap>` :
    1. Lit `$SBFB_CUSTOM_RELAYS` (split comma) → construct
       RelayMap si non-vide
    2. Sinon lit `~/.sbfb/relays.json` → construct RelayMap
    3. Sinon fallback `iroh::defaults::prod::default_relay_map()`
  - `pub fn validate_relay_url(url: &Url) -> Result<()>` :
    1. scheme `https://` obligatoire
    2. host pas localhost/127.0.0.1 (sauf `SBFB_DEV_MODE=1`)

- `crates/nexus-core-rs/src/dht_quorum.rs` (~180 LOC) :
  - `pub async fn redundant_resolve(node_id: NodeId,
    relays: &[pkarr::Relay], timeout: Duration) -> Result<SignedPacket>`
  - Internal : spawn 3 tasks via `tokio::spawn` + collect via
    `futures::future::join_all` avec deadline
  - Quorum check : `if ok_count >= 2 && all_match_signature { return Ok(record) }`
  - `else warn!("dht_quorum_fail: ok={}/3 match={}", ok_count, match_count)` + return `Err(DhtQuorumFailed)`

**Fichiers modifies** :

- `crates/nexus-core-rs/src/endpoint.rs` :
  - `Endpoint::builder()` integration : `relay_mode(load_relay_map().into_mode())`
  - Log info `"home_relay={}"` au bind pour diagnostic
- `crates/nexus-core-rs/src/lib.rs` : `pub mod relay_config; pub mod dht_quorum;`
- `crates/nexus-shell-daemon-core/src/lib.rs` : use `dht_quorum::redundant_resolve` dans le browse aggregator (si pattern actuel lookup single)

### Tests attendus (+20 Rust)

**Unit tests** (`crates/nexus-core-rs/src/relay_config.rs`) :

1. `load_relay_map_returns_defaults_when_empty_env` (cleared env + missing file → n0 prod)
2. `load_relay_map_parses_env_comma_separated`
3. `load_relay_map_parses_json_file`
4. `load_relay_map_env_overrides_file`
5. `validate_relay_url_rejects_http_scheme`
6. `validate_relay_url_rejects_localhost_non_dev`
7. `validate_relay_url_accepts_localhost_when_dev`

**Unit tests** (`crates/nexus-core-rs/src/dht_quorum.rs`) :

8. `redundant_resolve_returns_record_on_3_of_3_match`
9. `redundant_resolve_returns_record_on_2_of_3_match`
10. `redundant_resolve_errs_on_1_of_3_match`
11. `redundant_resolve_errs_on_signature_mismatch`
12. `redundant_resolve_times_out_after_deadline`

**Integration tests** (`crates/nexus-core-rs/tests/relay_federation.rs`) :

13. `connect_via_primary_relay_succeeds` (relay 1 up, 2 down)
14. `connect_via_fallback_when_primary_down` (relay 1 down, 2+3 up)
15. `connect_fails_all_relays_down` (all 3 down → timeout err)

**Daemon integration tests** (`crates/nexus-shell-daemon/tests/browse_dht_quorum.rs`) :

16. `browse_aggregator_uses_dht_quorum_when_lookup_available` (check trace log)

+4 tests env permutations + mock pkarr tests = **~20 tests total**.

### Critere d'acceptation Phase C

- [ ] `cargo test -p nexus-core-rs` green (+12 tests nouveaux)
- [ ] `cargo test -p nexus-shell-daemon` green (+1-2 tests)
- [ ] Launcher startup log affiche `home_relay=...` (ou `home_relay=fallback`)
- [ ] Modifier `~/.sbfb/relays.json` → relancher launcher → home_relay change observe
- [ ] `SBFB_CUSTOM_RELAYS="https://relay1.test,https://relay2.test" launcher` → 2 relays utilises

### Risques Phase C

- **iroh 0.97 API change inattendu** (`RelayNode` fields) : on
  verifiera en debut Phase C via `cargo doc --open -p iroh` +
  test smoke avant ecrire code. Si API a change, adapter.
- **DHT quorum timeout calibration** : 3 lookups en parallele
  peuvent diverger sur reseau lent → timeout 5s global + 2s
  per-lookup. Ajuster si test flaky.
- **Pkarr relay availability** : si n0 pkarr down pendant dev
  → tests flakey. Mitigation : tests utilisent mock pkarr
  in-memory (pattern iroh test utilities).

### Commit

`feat(sprint18): Phase C — multi-relai federation + DHT redundant lookup`

Body : RelayMode::Custom integration, load_relay_map precedence
env>file>defaults, dht_quorum.rs 2/3 majority Ed25519-verified,
+12 unit + +5 integration = +17 tests Rust. Gate 1 prerequis 3/4
cleared (defense B-Eclipse + B-BGP initial).

---

## Phase D — Coord-side TaskEntry wire-through + X-SBFB-Token rotation (~400 LOC)

### Goal

1. **Dette S16 C-1/C-2 coord-side** : coord emet TaskEntry avec
   `is_open_source` + `estimated_watts` + `estimated_vram_mb` +
   `estimated_hours` remplis depuis project metadata + app SDK
   `cost_estimate()`, plutot que defaults 0/false.
2. **Token rotation** : X-SBFB-Token (S16 Phase A) rotation
   automatique toutes les 24h avec overlap window 10min
   (old+new acceptes), enforcement launcher-cote genere, daemon
   valide les 2 tokens pendant overlap.

### Decisions techniques

- **Project metadata source of truth** : `is_open_source` derive
  automatiquement par coord au publish via chemin `repo_url`
  obligatoire (pattern S14 SBFB.json Keyoxide). Si deploy-from-repo
  → `true`. Si deploy-from-zip prive → `false`. Non-user-settable
  (pattern npm provenance, deja etabli S16 D-1).
- **Estimates source** : `nexus_sdk.NexusApp.cost_estimate()`
  abstract method. App SDK retourne tuple `(watts: int,
  vram_mb: int, hours_per_task: float)`. Defaults : apps existantes
  overridable, apps sans override → `(100W, 2000mb, 0.1h)`
  defaults conservateurs (inspires cost estimates Sprint 15-16).
- **Token rotation pattern** : launcher genere nouveau token
  toutes les 24h (cron interne tokio), `current` + `previous`
  conserves en `~/.sbfb/tokens.json` perm 0600. Daemon accepte
  les 2 pendant 10min overlap, puis discard `previous`.

### Livrables

**Nouveaux fichiers** :

- `crates/nexus-launcher/src/token_rotation.rs` (~100 LOC) :
  - `pub struct TokenRotator { current: String, previous: Option<String>, rotated_at: Instant }`
  - `pub fn spawn_rotation_loop(rotator: Arc<Mutex<TokenRotator>>, interval: Duration)`
  - Rotation logic : generate 32-byte random, write
    `~/.sbfb/tokens.json` atomically (tempfile + rename)
  - Overlap window : `previous` conserve 10min apres rotation,
    puis None

**Fichiers modifies** :

- `packages/nexus-coordinator/src/nexus_coordinator/tasks.py` :
  - `craft_task(project: Project, app_state: dict) -> TaskEntry` :
    - Lit `project.repo_url` : si present → `is_open_source=True`
    - Lit `project.app_instance.cost_estimate()` :
      `(watts, vram_mb, hours)`
    - Construit `TaskEntry` avec ces valeurs (plus de defaults 0/false)
  - Unit test coverage : `test_craft_task_open_source_from_repo_url`,
    `test_craft_task_private_zip_is_closed`,
    `test_craft_task_estimates_from_app_sdk`,
    `test_craft_task_estimates_fallback_defaults`

- `packages/nexus-sdk/src/nexus_sdk/app.py` :
  - Ajout abstract method `def cost_estimate(self) -> tuple[int, int, float]`
  - Default implementation `(100, 2000, 0.1)` avec `@classmethod`
  - Update existing apps (`packages/nexus-app-gov`, `-coldcase`,
    `-forensics`) : override `cost_estimate()` avec valeurs
    realistes

- `crates/nexus-shell-daemon/src/loopback.rs` :
  - `fn validate_token(request_token: &str, rotator: &TokenRotator) -> bool` :
    accept si match `current` OR (`previous` present AND rotated_at < 10min ago)

- `crates/nexus-launcher/src/main.rs` :
  - Spawn `token_rotation::spawn_rotation_loop(..., Duration::from_secs(86400))`
    au startup

### Tests attendus (+15)

**Coord tests** (`packages/nexus-coordinator/tests/test_tasks.py`) :

1. `test_craft_task_derives_is_open_source_true_from_repo_url`
2. `test_craft_task_derives_is_open_source_false_when_no_repo_url`
3. `test_craft_task_uses_app_cost_estimate`
4. `test_craft_task_falls_back_to_defaults_when_app_no_cost_estimate`
5. `test_craft_task_does_not_allow_client_override_is_open_source`
   (regression S16 D-1)

**SDK tests** (`packages/nexus-sdk/tests/test_app.py`) :

6. `test_cost_estimate_default_returns_conservative`
7. `test_cost_estimate_override_in_subclass`

**Rust token rotation tests** (`crates/nexus-launcher/src/token_rotation.rs`) :

8. `rotates_after_interval`
9. `keeps_previous_during_overlap_window`
10. `discards_previous_after_overlap`
11. `concurrent_rotation_safe` (thread safety via Mutex)
12. `persists_tokens_to_file_atomically`

**Daemon validation tests** (`crates/nexus-shell-daemon/tests/loopback_token.rs`) :

13. `accepts_current_token`
14. `accepts_previous_token_during_overlap`
15. `rejects_previous_token_after_overlap`

### Critere d'acceptation Phase D

- [ ] `uv run pytest packages/nexus-coordinator/tests/ -q` green (+5)
- [ ] `uv run pytest packages/nexus-sdk/tests/ -q` green (+2)
- [ ] `cargo test -p nexus-launcher -p nexus-shell-daemon` green (+8)
- [ ] Flow E2E : publish project avec repo_url → coord emet Task
  avec `is_open_source: true` + estimates remplis → worker
  consent watcher accepte selon level L2 → runtime.rs execute
- [ ] Launcher startup : observe token rotation log a h+24h
  dans `~/.sbfb/tokens.json` (manuel, sleep-based test optionnel)

### Risques Phase D

- **App SDK backward compat** : les apps existantes `gov`,
  `coldcase`, `forensics` n'ont pas `cost_estimate()` override
  → default applicable. Pas de breakage.
- **Token rotation race condition** : si daemon valide token
  juste pendant le flip current↔previous → use Arc<RwLock> pattern
  + tests 11 `concurrent_rotation_safe` assure.
- **File write atomicity** : `~/.sbfb/tokens.json` write doit
  etre atomic (tempfile + rename) sinon crash mid-write laisse
  launcher sans tokens. Test 12 `persists_tokens_to_file_atomically`.

### Commit

`feat(sprint18): Phase D — coord-side TaskEntry wire-through + X-SBFB-Token rotation`

Body : coord craft_task remplit is_open_source + estimates,
SDK cost_estimate() abstract + defaults conservateurs,
token rotation 24h + 10min overlap window, atomic file write,
+7 coord/SDK pytest + +8 Rust tests. Ferme dette S16 C-1/C-2.
Carry quick-win roadmap §4 delivered.

---

## Phase E1 — NVIDIA driver CVE check at launcher startup (~250 LOC)

### Goal

Au startup launcher, scrape NVD API filter CPE NVIDIA GPU Display
Driver, compare version installee local (via `nvml-wrapper` deja
present S15) avec CVE affecting versions, warn utilisateur si
vulnerabilite detectee.

### Decisions techniques

- **NVD API** : REST `https://services.nvd.nist.gov/rest/json/cves/2.0`
  avec query param
  `cpeName=cpe:2.3:o:nvidia:gpu_display_driver:*`. Source :
  [NVD API documentation](https://nvd.nist.gov/developers/vulnerabilities).
  Rate limit : 5 req/30s sans API key, 50 req/30s avec (nous =
  sans).
- **Cache 24h** : `~/.sbfb/nvd-cache.json` pour eviter spam NVD
  a chaque launcher start. TTL 24h, check `last_updated` field.
- **Warning-only** (pas block) : launcher print warning UI + log,
  user continue. Block-on-CVE critique trop agressif S18 (Gate 1
  not-yet-critical workloads).

### Livrables

**Nouveaux fichiers** :

- `crates/nexus-launcher/src/driver_check.rs` (~180 LOC) :
  - `pub async fn check_nvidia_drivers() -> Result<DriverCheckReport>`
  - Internal :
    1. `fetch_local_driver_version()` via `nvml-wrapper::Nvml`
    2. `load_cache() -> Option<NvdData>` lit cache si TTL < 24h
    3. `fetch_nvd_cves_if_stale()` → GET NVD API + parse JSON
    4. `filter_affecting_version(cves, local_version)` → Vec<Cve>
    5. Return `DriverCheckReport { local_version, cves_affecting: Vec<Cve>, critical_count }`
  - Errors : network fail → warn + return empty report (fail open, pas de bloquer launch)

**Fichiers modifies** :

- `crates/nexus-launcher/src/main.rs` :
  - Startup async call `check_nvidia_drivers()`, log report
  - Si `critical_count > 0` → `eprintln!("⚠ NVIDIA driver {local}
    has {n} critical CVE. Consider update.")` sans block

- `crates/nexus-launcher/Cargo.toml` : add dep `reqwest = { version = "0.12", features = ["json"] }` si pas deja present

### Tests attendus (+5)

Tests unit (`crates/nexus-launcher/src/driver_check.rs`) :

1. `version_affected_by_cve` (parse CPE match → true/false)
2. `cache_hit_within_ttl` (mock file with timestamp < 24h → no fetch)
3. `cache_miss_triggers_fetch` (mock file timestamp > 24h → fetch)
4. `offline_fallback_returns_empty_report_not_err` (network err
   → fail open)
5. `filter_critical_cves_only` (severity field parse, filter)

### Critere d'acceptation Phase E1

- [ ] `cargo test -p nexus-launcher driver_check` green (+5)
- [ ] Launcher startup fresh (cache empty) → NVD API call made
  observed via log
- [ ] Launcher 2nd startup within 24h → no NVD API call (cache hit)
- [ ] Manual test : patch local driver version to match known CVE
  → warning displayed

### Risques Phase E1

- **NVD rate limit** : 5 req/30s sans API key, lance aussi
  WebSearch resolve nvml-wrapper version bumps. Cache 24h mitige.
- **Offline user** : tests 4 garantit fail open (pas de block
  offline).
- **CPE format NVIDIA driver strings** : differ by platform (windows
  vs linux). Test carefully avec actual `Nvml::driver_version()`
  output.

### Commit

`feat(sprint18): Phase E1 — NVIDIA driver CVE check at launcher startup`

Body : reqwest GET NVD API filter CPE NVIDIA GPU driver, cache
24h, fail open offline, warning non-blocking, +5 tests unit.
Roadmap §3 S18 item driver-check delivered.

---

## Phase E2 — Warrant canary monthly Ed25519 gossip publish (~150 LOC)

### Goal

Publisher un message warrant canary mensuel signe Ed25519 (cle
node_id daemon) via iroh-gossip topic `sbfb-warrant-canary-v1`,
mirror dans `CANARY.txt` a la racine du repo (fichier commite
mensuellement).

### Decisions techniques

- **Gossip topic** : `sbfb-warrant-canary-v1` (TopicId fixe,
  hardcoded). Subscribers peuvent s'abonner et verifier.
- **Signature** : Ed25519 sign sur canonical bytes (JCS format,
  reutilise `nexus-core-rs::canonical::to_canonical_bytes`).
- **Headline injection** : user provides headline (argument CLI
  ou config `~/.sbfb/canary-headline.txt`). Pas de scrape auto
  (risque manipulation).
- **Frequency** : mensuel, date `ISO-8601` `YYYY-MM-DD`.
  Next-update-date dans le message = date + 45 jours (15 jours
  grace period).

### Livrables

**Nouveaux fichiers** :

- `crates/nexus-shell-daemon-core/src/canary.rs` (~80 LOC) :
  - `pub struct Canary { date: Date, headline: String, signature: Vec<u8>, pubkey: PublicKey }`
  - `pub fn build_canary(date: Date, headline: String, signer: &SecretKey) -> Canary`
  - `pub fn verify_canary(canary: &Canary) -> Result<()>`
  - `pub async fn publish_canary(canary: &Canary, gossip: &Gossip) -> Result<()>`

- `crates/nexus-shell-daemon/src/cli.rs` :
  - Subcommand `sbfb canary publish --headline "NYT 2026-04-15: ..."`
  - Reads `~/.sbfb/canary-key.key` (cle maintainer persistante
    creee/relue via `KeyPair::load_or_generate`, distincte de
    l'identite daemon iroh qui est ephemere per-boot), signs,
    publishes via gossip, writes `CANARY.txt` local (replace
    existing). Correction post-implementation : la doc d'origine
    mentionnait `~/.sbfb/node_id.key (existant S11+)` — c'etait
    une hypothese incorrecte, le daemon ne persiste pas son
    identite iroh. Le warrant canary necessite une pubkey stable
    multi-mois donc sa propre cle dediee.

- `CANARY.txt` racine repo :
  - Premier canary publie en Phase E2 commit meme (bootstrap)
  - Format ASCII plain text du §D5 kickoff
  - **Signature hex lowercase** (coherent avec le reste du
    codebase signed-payload — task/result/claim/invite/kudos/
    curator-list/provenance utilisent tous hex). Le §D5 kickoff
    mentionnait `base64-Ed25519-*` en document original — obsolete.

**Fichiers modifies** :

- `.github/workflows/canary-monthly.yml` :
  - Cron **weekly** (lundi 08:00 UTC)
  - Trigger sur push / PR touchant CANARY.txt + canary.rs + le
    workflow lui-meme, plus workflow_dispatch
  - Step : **verify** CANARY.txt signature + gate 45-day
    staleness (si date > 45j, fail le job → GitHub email le
    maintainer = notification dead-man switch)
  - **DEVIATION deliberee vs plan initial** : le plan demandait
    un auto-publisher (cron qui signe + commit + push `sbfb
    canary publish`). REJETE pour raisons threat-model. Stocker
    la cle Ed25519 en GHA secret ≡ compromission GHA =
    compromission cle. Un maintainer sous gag order pourrait
    etre contraint de "laisser tourner le cron" → signatures
    valides perpetuelles alors que le projet est backdoored =
    cassure du dead-man switch. Le warrant canary classique
    (rsync.net, IVPN) repose sur une re-signature **manuelle**
    intentionnelle du maintainer — le workflow livre est un
    verifier, jamais un signer. Rationale complete en tete du
    yml + review §Q1 + `sprint18_phase_E2_review.md`.

### Tests attendus (+5)

1. `build_canary_includes_date_headline_sig`
2. `verify_canary_accepts_valid_signature`
3. `verify_canary_rejects_tampered_message`
4. `verify_canary_rejects_wrong_pubkey`
5. `publish_canary_emits_gossip_event` (mock gossip)

### Critere d'acceptation Phase E2

- [ ] `cargo test -p nexus-shell-daemon-core canary` green (+5)
- [ ] `sbfb canary publish --headline "test headline"` command
  ecrit `CANARY.txt` avec format valide
- [ ] `CANARY.txt` lisible humain + verifiable via script
  `scripts/verify-canary.sh`

### Risques Phase E2

- **Headline manipulation** : si attaquant compromet dev et
  change headline, signature invalide auto-detectee. OK.
- **Key loss** : si cle Ed25519 daemon perdue (device wipe),
  canary stale → triggere user alert. OK pattern standard.
- **Gossip subscribers 0 S18** : personne ne va consommer le
  gossip topic en S18 (feature scheme Sprint 19+). Acceptable —
  le canary vaut aussi par le fichier git.

### Commit

`feat(sprint18): Phase E2 — warrant canary monthly Ed25519 gossip publish`

Body : Canary struct + build/verify/publish, CLI `sbfb canary
publish`, GitHub Action cron monthly, CANARY.txt bootstrap,
+5 tests unit. VALIDATED_BLUEPRINT couche 10 opsec premiere brique
effective.

---

## Phase E3 — Radicle mirror GitHub Action (~50 LOC)

### Goal

Automated mirror du repo GitHub principal vers Radicle
decentralized git hosting. Redondance decentralisee : si
GitHub.com subpoena'd, Radicle reste accessible.

### Decisions techniques

- **Machine account Radicle** : nouveau `rad` identity dediee
  CI (pas la cle perso dev). Stored as GitHub Actions secret
  (base64 encoded `.radicle/keys/radicle.json`).
- **Source action** : `gsaslis/mirror-to-radicle` (community,
  source : [GitHub repo](https://github.com/gsaslis/mirror-to-radicle)).
- **Trigger** : `push` to master + `schedule` daily 03:00 UTC
  fallback safety.

### Livrables

**Nouveaux fichiers** :

- `.github/workflows/radicle-mirror.yml` (~30 LOC YAML) :
  ```yaml
  name: Radicle mirror
  on:
    push:
      branches: [master]
    schedule:
      - cron: '0 3 * * *'
    workflow_dispatch:
  
  jobs:
    mirror:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
          with:
            fetch-depth: 0
        - uses: gsaslis/mirror-to-radicle@v1
          with:
            radicle-key: ${{ secrets.RADICLE_MACHINE_KEY }}
            project-name: sbfb
  ```

- `docs/release/RADICLE_MIRROR.md` (~20 lignes) :
  - §1 What is Radicle
  - §2 How to verify mirror
  - §3 How to clone from Radicle instead of GitHub
    (`rad clone <rid>`)

### Tests attendus

Aucun test unit Rust (pur ops CI). Verification manuelle.

### Critere d'acceptation Phase E3

- [ ] GitHub Actions workflow `radicle-mirror.yml` run verte
  sur master push
- [ ] `rad clone <rid>` fonctionne depuis machine tierce
- [ ] Doc verification user-facing ok

### Risques Phase E3

- **Secret setup** : le user doit generer `rad` identity manuellement
  et uploader `RADICLE_MACHINE_KEY` secret. Documente dans
  `RADICLE_MIRROR.md §setup`.
- **`gsaslis/mirror-to-radicle` action uptime** : dep externe.
  Mitigation : pin `@v1` (commit SHA) pas `@main`.

### Commit

`feat(sprint18): Phase E3 — Radicle mirror GitHub Action`

Body : workflow YAML cron daily + push master, machine account
pattern, doc RADICLE_MIRROR.md clone verification, redondance
decentralisee anti-subpoena. VALIDATED_BLUEPRINT couche 10 opsec
deuxieme brique.

---

## Phase F — Consolidation + verification + audit plan S19 (~250 LOC docs)

### Goal

Cloture standard sprint :
1. Update docs projet (CLAUDE.md, SPRINT_LOG.md, memory
   nexus_grid_pivot.md)
2. Ecrire `sprint18_verification.md` fail-fast checklist
3. Ecrire `sprint18_audit_plan.md` tracks A-F pour S19 Phase 0
4. Migrer `.planning/active/sprint18_*.md` →
   `.planning/archive/v1.2/`

### Livrables

**Nouveaux fichiers** :

- `.planning/active/sprint18_verification.md` (~100 lignes) :
  - Checklist A-F : CI green, test count delta, scope respecte,
    commits pattern, Gate 1 unlock criteria
  - Stack commits complete
  - Tests compteurs final

- `.planning/active/sprint18_audit_plan.md` (~150 lignes) :
  - Tracks A-F verification pour audit S18 par Sprint 19 Phase 0
  - Same template sprint17_audit_plan.md pattern
  - Methodes + P0/P1 probables par track

**Fichiers modifies** :

- `CLAUDE.md` section "Etat actuel" : Sprint 18 CLOSED + Gate 1
  unlocked, commits stack A-E3 + F
- `docs/claude/SPRINT_LOG.md` : row S18 v1.2 (theme : quick wins
  + supply chain + multi-relai + Gate 1 unlock)
- Memory `nexus_grid_pivot.md` frontmatter description : bump
  tip post-F, update compteurs tests (1128 → ~1183), mention
  Gate 1 unlock

**Migration PARA** :
- `git mv .planning/active/sprint18_kickoff.md .planning/archive/v1.2/`
- `git mv .planning/active/sprint18_plan.md .planning/archive/v1.2/`
- `git mv .planning/active/sprint18_verification.md .planning/archive/v1.2/`
- `git mv .planning/active/sprint18_audit_plan.md .planning/archive/v1.2/`

### Tests attendus

Aucun nouveau test. Verification suite complete :
- `cargo test --workspace --locked` → 430 + ~20 = **~450 Rust**
- `uv run pytest packages/nexus-sdk/tests/ -q` → 183 + ~2 = **~185**
- `uv run pytest packages/nexus-coordinator/tests/ -q` → 187 + ~5
  = **~192 + 3 skipped**
- `uv run pytest packages/nexus-app-gov/tests/ -q` → 46 (inchange)
- `cd web && npm run test:unit` → 239 (inchange + tests ops ne
  changent pas count vitest)
- `npx playwright test` → 38 (inchange)
- Size-limit 7/7 OK
- Compteur final estime : **~1183 tests** (1128 + 55)

### Critere d'acceptation Phase F

- [ ] Tous les tests suites green
- [ ] CLAUDE.md update reflete tip final S18
- [ ] SPRINT_LOG.md row S18 complete
- [ ] `sprint18_audit_plan.md` cree pour S19 Phase 0
- [ ] Files `.planning/active/sprint18_*.md` migres vers
  `archive/v1.2/` (4 files)
- [ ] Memory `nexus_grid_pivot.md` tip sync avec HEAD post-F

### Commit

`chore(sprint18): Phase F — wrap-up + verification + audit plan S19 + migrate planning`

Body : updates docs standards, verification fail-fast, audit plan
tracks A-F pour S19 Phase 0, migration PARA 4 files. Tests
compteur final ~1183 (+55 delta). Gate 1 effectively unlocked
(DnD Forge beta fermee deployable). Next : Sprint 19 Phase 0
(audit S18) puis PoW gossip + TLS pinning + DHT phase 2.

---

## Commit message body guidelines (rappel)

Tous les commits S18 suivent le pattern etabli (cf.
`docs/claude/README.md §4`) :

- **Titre** : `feat(sprint18): Phase X — titre court`
- **Body** riche :
  - 1 paragraphe contexte (quoi/pourquoi)
  - Liste changements techniques cles (files modifies, API
    publique changee)
  - Delta tests chiffre cumulatif (ex: "+17 tests Rust, total ~447")
  - Scope cuts respectes explicitement (pas de rabattement
    silencieux)
  - Reference issue upstream si applicable (CVE-2026-34941, etc.)

Exception Phase E decoupee 3 commits : chaque commit reference
"Phase E1/E2/E3" dans le titre + body mentionne "1 of 3 Phase E
commits".

---

## Scope cuts sprint (re-rappel depuis kickoff §6)

- Iroh audit externe → S29 Gate 3 prerequis
- Pyodide sandbox escape → S22+ wasmtime process isolation
- PoW gossip → S19
- Encryption at rest keypair → S20
- TLS cert pinning relays → S19
- Self-hosted pkarr relay → S19
- Federated ONG-run relays concrets → S19+
- ML-DSA-65 / ML-KEM-1024 PQC → S26+
- THREAT_MODEL.md cross-ref S17 docs → P2 tech debt
- Structured output llama.cpp grammar → S20

Ces items sont **explicitement hors-S18** et ne peuvent etre
"rattrapes" sans re-ouvrir le kickoff.

---

## Checklist day-0 S18 (avant de lancer Phase A)

- [x] `sprint18_kickoff.md` ecrit dans `.planning/active/`
- [x] `sprint18_plan.md` ecrit dans `.planning/active/`
- [ ] `git mv .planning/active/sprint17_audit_findings.md
  .planning/archive/v1.2/` (staged)
- [ ] Commit `chore(planning): close S17 + open Sprint 18 —
  quick wins + supply chain` (migration S17 findings +
  kickoff/plan S18)
- [ ] Session fraiche Phase A : lit kickoff §4 D1..D5, verifie
  tip master = commit chore(planning), lance Phase A supply-chain
  CI implementation
