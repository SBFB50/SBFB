# Sprint 18 — Audit findings (Sprint 19 Phase 0)

**Auditeur** : session Claude Code fraiche, ouverte 2026-04-15 post-Sprint 18
wrap-up commit `4453bfd`.
**Timebox observe** : ~2h30 (lectures memory + 4 docs S18 + 8 modules code +
1 cross-check rapide phase reviews livreur).
**Tip audite** : `4453bfd` (chore(sprint18) Phase F wrap-up + verification +
audit plan S19 + migrate planning).
**Range commits audite** : `4f0727b..4453bfd` (20 commits, dont 9 feat/chore
S18 directs + ~10 chore(claude) tooling hors-scope inclus par proximite
chronologique).

---

## Verdict global : **CONDITIONAL PASS**

- 0 finding **P0** (aucun blocage securite ou regression critique).
- 1 finding **P1** (D-1 : claim Gate 1 unlock contredit le carry-over admis
  en commit body Phase D — token rotation **primitive** livree mais NON
  cablee au router HTTP daemon).
- 4 findings **P2** (gap promesse vs realite : DHT quorum primitive non
  wirée, wheel sans attestation, cargo-deny `--workspace` flag obsolete,
  discrepancies docs hygiene).
- 5 findings **P3** (cosmetique / UX / hardening defense en profondeur).

Sprint 19 Phase A est **bloque** tant que le P1 D-1 n'est pas resolu via
**au moins** un commit `fix(sprint18): D-1 — ...` landed sur master. Deux
fixes acceptables, au choix de la session livreur (cf. §Commits fix
attendus).

Sprint 18 livre une **vraie progression** (supply chain CI vert localement,
reproducible builds testes, multi-relai config + DHT quorum primitive
testes, NVD scrape solide, warrant canary signed avec dead-man switch
preserve, Codeberg mirror auth-safe). Le seul vrai gap est la divergence
**narrative vs implementation** sur la rotation token et le wiring DHT —
aucun risque securite immediat, mais le verification.md §Gate 1 unlock
fait une promesse partielle.

---

## Mode d'emploi suivi

L'ordre de lecture impose dans `sprint18_audit_plan.md §Mode d'emploi` a
ete respecte :

1. memory (`MEMORY.md`, `nexus_grid_pivot.md`, `sprint_audit_gate.md`,
   `feedback_approach.md`)
2. `git log --oneline 4f0727b..HEAD` (range complet S18)
3. `sprint18_kickoff.md` (D1..D5 pris en note, NON rebattus)
4. `sprint18_plan.md`
5. `sprint18_verification.md`
6. `sprint18_audit_plan.md` (les 8 tracks A/B/C/D/E1/E2/E3/F + meta-track
   Radicle-v1.0)
7. **Code + tests + workflows** par track avant lecture des phase reviews
   livreur (formation d'opinion independante)
8. **Phase reviews livreur** B/C/D/E1/E2/E3 (skim cibles `Verdict` +
   `Findings`) en cross-check final pour challenger ou confirmer

Out of scope respecte : D1..D5 gelees du kickoff S18 non rebattues (aucun
finding sur le choix `cargo-deny > cargo-audit` D3, le pin wasmtime
`>=43.0.1` D2, le format warrant canary Ed25519 + JCS hex D5, etc.).

---

## Track A — Supply chain CI baseline : **PASS avec 1 P2**

**Question centrale** : les 3 gates configures correctement, pin wasmtime
effectif, workflow fail-closed ?

### Verifications effectuees

- `.github/workflows/supply-chain.yml` : 3 jobs paralleles cargo-deny +
  pip-audit (matrix sur 3 packages) + audit-ci npm. `permissions: contents:
  read`. Cron weekly Mondays 08:00 UTC. Triggers `pull_request` + `push` +
  `workflow_dispatch`. ✅
- `deny.toml` racine : `version = 2` schema cargo-deny moderne. `[bans]
  deny = [{ crate = "wasmtime:<43.0.1", reason = "CVE-2026-34941 +
  CVE-2026-34946" }]` — pin effectif format v2. `[advisories] yanked =
  "deny"`, `[sources] unknown-registry = "deny", unknown-git = "deny"`.
  License allowlist tight (Apache/MIT/BSD/ISC/Unicode/Zlib/CC0/MPL/AGPL/
  CDLA/OpenSSL).
- 1 ignore documente : `RUSTSEC-2026-0097` (rand 0.8 ThreadRng+custom log,
  pas applicable a SBFB qui utilise `OsRng` + tracing). Reason field rempli.
- `web/audit-ci.json` : `critical: true` (fail seulement sur critical).
  Coherent avec policy yml.
- `cargo deny check` execute localement : **`advisories ok, bans ok,
  licenses ok, sources ok`**. ✅
- Aucun `wasmtime` dans aucun `Cargo.toml` (kickoff §1.3 confirme : SBFB
  n'utilise pas encore wasmtime, le ban est preemptif).

### Findings

- **A-1 (P2)** — `arg: --workspace` dans le job cargo-deny (`supply-
  chain.yml:63`) est **rejete par cargo-deny moderne**. Sur la machine dev
  de l'auditeur (cargo-deny 0.19.2 local), `cargo deny check --workspace`
  produit `error: unexpected argument '--workspace' found`. Le flag est
  devenu le default depuis cargo-deny 0.14+. Si `EmbarkStudios/cargo-deny-
  action@v2` installe une version recente, **le job CI echoue avec un faux
  positif** au prochain PR — gate fail-closed devient gate broken.
  - Fix : retirer `arg: --workspace` (le default cargo-deny scan deja le
    workspace entier) ou pin une version cargo-deny ancienne via input
    `command-arguments` de l'action (non recommande).
  - Severite P2 : pas de regression securite (le CI fail loud, pas faux
    pass), mais la baseline supply chain serait inutilisable jusqu'au fix.
    A verifier sur le premier run reel CI post-S18 (le repo est prive,
    l'auditeur n'a pas pu tester sur GHA).

### Triggers P0/P1 evalues

- ✗ "CVE Critical dans dep actuelle qui passerait le gate" : aucun (cargo
  deny check local clean, 1 ignore RUSTSEC documente).
- ✗ "wasmtime pin absent" : present format v2, validate by `cargo deny
  check`.
- ✗ "license blocklist incomplete" : allowlist coherente, tests pass.

---

## Track B — Reproducible builds + SLSA in-toto : **PASS avec 1 P2 + 1 P3**

**Question centrale** : verification SHA256 deterministe, attestation in-
toto schema SLSA v1.0, signatures Ed25519 verifiables offline ?

### Verifications effectuees

- `scripts/release-attest.sh` (~150 LOC bash) : `set -euo pipefail`,
  `SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)` injecte par script (pas
  hardcoded), `cargo build --release --locked`, sha256 emit format
  compatible `sha256sum -c`, `.intoto.jsonl` JSON inline schema in-toto
  Statement v1 + predicate SLSA Provenance v1.
- Subject digest sha256 = artifact bytes (test smoke `attestation-
  schema.sh` cross-check). ✅
- `resolvedDependencies` : commit URI `git+...@<sha>` + `Cargo.lock`
  sha256. Conforme SLSA L1 minimum.
- Cosign keyless OIDC Sigstore via `COSIGN_EXPERIMENTAL=1` quand cosign
  present (release.yml `id-token: write` + cosign-installer v2.4.1).
- `tests/ci-smoke/reproducible-build.sh` : 2 builds back-to-back + `cargo
  clean --release -p $BINARY` entre les deux + assert sha256 match. ✅
- `tests/ci-smoke/attestation-schema.sh` : check structure jq + cross-check
  subject sha256 vs artifact bytes + predicateType exact match. ✅
- `.cargo/config.toml` : `incremental = false` + `[target.*-pc-windows-
  msvc] rustflags = ["-C", "link-arg=/Brepro"]` (PE TimeDateStamp + Debug
  GUID deterministes Windows). Documente clairement.
- `[profile.release]` racine `Cargo.toml` (vu line 193) : `codegen-units=1`,
  `lto="fat"`, `strip="symbols"`, `debug=false`.
- `docs/release/REPRODUCIBLE_BUILDS.md` (~150 lignes) : §1 verify SHA256
  copy-paste commands, §2 cosign verify-blob + slsa-verifier, §3 rebuild
  local deterministe, §4 profile release deterministe avec table par-cle,
  §5 limitations connues (cross-platform, Cranelift, wheel, cosign
  keyless).

### Findings

- **B-1 (P2)** — Wheel `nexus-core-py` **n'est pas inclus dans la matrix
  `release.yml`**. Le script `release-attest.sh:76-87` supporte le wheel
  via `maturin build`, mais `release.yml:39-43` matrix builde uniquement
  `nexus-worker / nexus-shell-daemon / nexus-launcher`. Le job `publish-
  pypi` builde les wheels mais sans appeler `release-attest.sh` →
  **aucun fichier `.intoto.jsonl` SLSA pour le wheel publie sur PyPI**.
  - Fix : etendre matrix `release.yml` avec un job `nexus-core-py` qui
    appelle `release-attest.sh nexus-core-py` (le script gere deja le
    chemin maturin), OU documenter explicitement le gap dans
    `REPRODUCIBLE_BUILDS.md §5 Limitations connues` (deja une note "best-
    effort S18, durci S19+" mais ambigue). Severite P2 : pas un risque
    securite immediat (PyPI n'est pas le chemin install dominant pre-
    launch), mais le claim "chaque binary release" du module doc est
    inexact.
- **B-2 (P3)** — `buildType` du predicate SLSA = `https://slsa.dev/
  container-based-build/v0.1?sbfb=release-attest.sh` (release-attest.sh:
  124). Semantiquement le build n'est pas container-based — c'est un
  script bash invoque depuis GHA Ubuntu. URI non-standard (pas dans la
  liste SLSA build types officielle). Fix : utiliser `https://slsa.dev/
  build-type/script/v1` (s'il existe en april 2026, sinon un custom
  `https://github.com/SBFB50/SBFB/build-types/release-attest@v1`).
  Severite P3 cosmetique : `slsa-verifier` accepte tout buildType pourvu
  que les digests matchent.

### Triggers P0/P1 evalues

- ✗ "determinisme casse" : tests smoke green, `/Brepro` Windows
  documente, `[profile.release]` durci.
- ✗ "signature utilise node_id ephemere" : cosign keyless OIDC = identite
  Sigstore liee a `SBFB50/SBFB.git@refs/heads/master + .github/workflows/
  release.yml` — persistante cross-releases.

---

## Track C — Multi-relai federation + DHT quorum : **CONCERN avec 1 P2**

**Question centrale** : fallback relais correct, DHT quorum 2/3 anti-
eclipse partial, tests E2E couvrent happy + degraded ?

### Verifications effectuees

- `crates/nexus-core-rs/src/relay_config.rs` (~280 LOC) : precedence env
  > file > defaults (`None` => caller preserve preset N0). HTTPS strict +
  loopback rejet (sauf `SBFB_DEV_MODE=1`). 6 unit tests + 2 tests
  integration. ✅
- `crates/nexus-core-rs/src/dht_quorum.rs` (~440 LOC) : trait
  `QuorumResolver` async (mockable) + `redundant_resolve()` avec quorum
  strict majority `total/2 + 1`. Bucket bytes + winning bucket. 8 unit
  tests + 5 integration tests = 13 tests pour la primitive. ✅
- `crates/nexus-core-rs/src/node.rs:240-289` : `Endpoint::builder(presets::
  N0).address_lookup(memory_lookup)` PUIS overwrite `relay_mode` si
  `load_relay_map() = Some(map)`. Fallback preserve byte-for-byte le
  comportement pre-S18 quand no custom config. ✅ Pas de regression sur
  Endpoint default (trigger P0 evite).
- Tests integration `relay_federation.rs` : 5 scenarios (env > file, http
  rejected, 1/3 success → NoMajority, 2/3 success → Ok, all down →
  AllFailed). Couvre happy + degraded. ✅

### Findings

- **C-1 (P2)** — **`redundant_resolve` n'est appele NULLE PART en
  production**. Grep `redundant_resolve|dht_quorum::|QuorumResolver` sur
  `crates/` retourne uniquement la definition + tests. Le browse
  aggregator (`nexus-shell-daemon-core`), le curator runtime, le
  `discovery.rs` ne wrappent pas leurs lookups dans la primitive 2/3
  quorum. La promesse "tout lookup DHT critique peut desormais exiger un
  accord 2/3" du commit body Phase C est livree comme **primitive prete**
  mais sans wiring. La defense Eclipse-by-DHT promise par la kickoff
  §1.2 + verification.md §Gate 1 ("DHT redundant lookup : 3 pkarr
  paralleles + quorum 2/3 (Phase C)") est **partiellement inactive en
  runtime**.
  - Fix : wire `redundant_resolve` dans au moins un chemin lookup
    critique (browse aggregator est le candidat le plus naturel — c'est
    la voie que le plan §C livrable mentionnait : "use `dht_quorum::
    redundant_resolve` dans le browse aggregator si pattern actuel lookup
    single"). OU : **explicitement** marquer dans verification.md §Gate 1
    unlock que C est livree comme primitive ready (S19 cable au
    runtime) plutot que cocher comme "complete".
  - Severite P2 (pas P1) parce que (a) la primitive est testee, prete a
    etre wirée sans changer son API ; (b) la kickoff § Phase C livrable
    a toujours dit "si pattern actuel lookup single" — le wiring est
    decrit conditionnellement ; (c) le code module doc §16-19 dit
    explicitement "deliberately generic over the resolver type" — design
    intentionnel "primitive d'abord, wiring case par case".
- **C-bonus** (informational, pas finding) — Pas de metric `relay_failover_
  count` Prometheus (anticipe par audit_plan). Pre-launch sans observabilite
  prod = OK, S19+ pourra ajouter.

### Triggers P0/P1 evalues

- ✗ "regression Endpoint default vers n0 obligatoires" : `load_relay_map()
  = None` preserve preset N0, audited line par line dans `node.rs:243`.
- ✗ "quorum accept 1/3 reponse au lieu de rejeter" : `quorum_threshold_for
  (3) = 2`, `redundant_resolve` retourne `NoMajority { ok_count: 1,
  max_agreement: 1 }` sur 1/3 success (test `quorum_end_to_end_primary_up_
  two_fallbacks_down` confirme).

---

## Track D — Coord-side wire + token rotation : **CONCERN avec 1 P1**

**Question centrale** : `is_open_source` + caps injectes cote coord AVANT
sign, token rotation automatique pas opt-in ?

### Verifications effectuees

**Coord wire (aspect 1)** :
- `packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py` :
  - `TaskCreateBody` Pydantic schema **n'a pas** les 4 champs is_open_source
    + estimated_*. Le client ne peut pas overrider (pydantic ignore les
    champs inconnus par defaut). ✅ Invariant S16 D-1 preserve.
  - `is_open_source = coord.config.identity.repo_url is not None` (line
    88) — server-derived. ✅
  - `_derive_cost_estimate(coord, body.app_name)` : lookup app par name →
    `app.cost_estimate()` → fallback `(100, 2000, 0.1)` defensif sur
    Exception. ✅
  - `dispatcher.submit(SubmitRequest(..., is_open_source=is_open_source,
    estimated_*=...))` — passe au dispatcher. ✅
- `packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py:107-126` :
  les 4 champs sont injectes dans `task_dict` AVANT `json.dumps(sort_
  keys=True)` AVANT `nexus_core.sign_task(task_json, secret)`. **Canonical
  bytes signe contient les 4 champs**. ✅
- `packages/nexus-sdk/src/nexus_sdk/app.py` : `cost_estimate()` ajoute
  comme methode concrete (defaults `(100, 2000, 0.1)`) — pas
  abstractmethod (drift plan, classe livreur P3 pour ne pas casser apps
  existantes). OK choix design.
- `packages/nexus-app-gov/src/nexus_app_gov/app.py:6` : `cost_estimate`
  override `(180, 6000, 0.15)` — coherent profil RAG+synthese.

**Token rotation (aspect 2)** :
- `crates/nexus-shell-daemon-core/src/auth.rs:421-607` : `TokenRotator`
  struct + `validate_token_with_rotator(request_token, &TokenRotator) ->
  bool` (constant-time check current, puis previous gated par
  `is_in_overlap_window()`). `TOKEN_OVERLAP_DURATION = 600s` (10min).
  `TokensFile` JSON serializable + `load()` / `write_atomic()` (tempfile
  + rename). ✅
- `crates/nexus-launcher/src/token_rotation.rs:46-76` : `spawn_rotation_
  loop(Arc<RwLock<TokenRotator>>, path, interval)` rotate via
  `generate_token()` + `write_atomic` toutes les 24h en prod
  (parameterise pour tests). Skip first immediate tick. ✅
- 5 tests primitive : `rotates_after_interval`, `keeps_previous_during_
  overlap_window`, `discards_previous_after_overlap`, `concurrent_
  rotation_safe` (8 readers + 1 writer 50 rotations), `persists_tokens_to_
  file_atomically`. Tous green. ✅

### Findings

- **D-1 (P1)** — **Le router HTTP daemon ne consomme PAS le `TokenRotator`
  ni `tokens.json`**.
  - Grep `validate_token_with_rotator|TokenRotator|tokens.json` dans
    `crates/nexus-shell-daemon/src/`, `crates/nexus-shell-daemon-core/src/`
    montre que `validate_token_with_rotator` est defini dans `auth.rs:597`
    mais **n'est appele que dans tests internes** du module (lignes 178-
    216 du `token_rotation.rs` test module).
  - `crates/nexus-shell-daemon/src/http.rs:137-150` : `build_router(state,
    token: String) -> Router` construit `AuthState::new(token)` — un
    **single token statique** capture au boot. Pas de `Arc<RwLock<
    TokenRotator>>`, pas de file-watcher sur `tokens.json`.
  - `crates/nexus-shell-daemon/src/runtime.rs:262 : let router =
    build_router(http_state, token);` confirme la chaine de wiring : le
    token est passe par valeur au demarrage, jamais re-lu.
  - **Consequence runtime** : la rotation `tokens.json` toutes les 24h
    tourne dans le launcher, mais le daemon valide chaque requete contre
    son token initial fige au boot. La rotation est **inactive dans le
    runtime production**.
  - **Le commit body Phase D admet ce gap** explicitement dans la section
    `[new]` carry-over : "Wire TokenRotator through build_router sous
    Arc<RwLock<_>> + notify file-watcher sur tokens.json cote daemon.
    Couvre le dernier delta entre 'primitive en place' et 'rotation
    effective en prod'." Le module `token_rotation.rs:22-27` re-explique
    le carry-over en prose.
  - **Probleme** : `sprint18_verification.md §Gate 1 unlock` ligne 67
    coche "**[x] Coord-side wire complete : TaskEntry craft + estimate
    caps + token rotation (Phase D)**". `CLAUDE.md §Etat actuel` ligne
    202 et `SPRINT_LOG.md row S18` annoncent egalement la rotation comme
    livree. La documentation utilisateur-facing **promet une fonction
    qui n'est pas active**.
  - **Severite P1** parce que la promesse Gate 1 unlock ("DnD Forge beta
    fermee deployable") repose en partie sur le claim rotation 24h
    operationnelle — un threat modeler externe lisant la verification.md
    et CLAUDE.md serait mis en erreur sur la posture rotation reelle.
    Pas de risque d'attaque immediat (le single token initial reste
    valide indefiniment, pas de break d'auth), mais le claim
    contredisant le carry-over admis = divergence narrative/code = audit
    fail.
  - **Fix attendu** : choisir l'une des deux options ci-dessous (cf.
    §Commits fix attendus).

- **D-bonus** (informational) — `_derive_cost_estimate` line 73-74 utilise
  `coord.apps.get(app_name)` ; vu que `coord.apps` n'est pas verifie nullable
  ici, un coord en cours de boot pourrait throw `AttributeError`. La
  defense `try/except Exception` line 75-78 catch et retourne fallback
  → resilient. OK.

### Triggers P0/P1 evalues

- ✗ "Wire bypass : worker accepte un task sans is_open_source" : `dispatcher.
  py:120` injecte explicitement `bool(req.is_open_source)` dans canonical
  → impossible de bypass.
- ✓ "token rotation casse session existant sans fallback" : **inverse mais
  applicable** — la rotation **ne s'applique pas du tout**, donc pas de
  break, mais aussi pas de benefice. Cf. D-1 ci-dessus.

---

## Track E1 — NVIDIA driver CVE check : **PASS**

**Question centrale** : NVD scrape rate-limit safe, cache 24h offline,
comparison version sans false-negative ?

### Verifications effectuees

- `crates/nexus-launcher/src/driver_check.rs` (~660 LOC) :
  - NVD endpoint constant `https://services.nvd.nist.gov/rest/json/cves/
    2.0`, CPE filter `cpe:2.3:o:nvidia:gpu_display_driver:*`. ✅
  - `CACHE_TTL = 24h` + cache `<sbfb_home>/nvd-cache.json` avec
    `CacheEnvelope { fetched_at_unix_secs, response }`. Atomic write via
    `path.with_extension("json.tmp")` + `std::fs::rename`. ✅
  - `FETCH_TIMEOUT = 10s` reqwest builder + `user_agent("sbfb-launcher/
    <version>")`. ✅
  - **Fail-open everywhere** : `check_nvidia_drivers()` returns
    `DriverCheckReport` jamais `Err`. Network fail → `fetch_failed: true`
    + empty `cves_affecting`. Missing NVIDIA driver → `local_version:
    None`. ✅
  - `filter_affecting_version` + `cpe_match_covers` : range bounds
    (start_including, start_excluding, end_including, end_excluding) +
    exact criteria match. Tests `version_range_bounds_include_and_
    exclude` valide `535.54.03 IN [535.0.0, 536.0.0)`, `536.0.0 OUT`,
    `534.99.99 OUT`. ✅
- 6 tests inline (vs +1 declare en verification.md §107 — discrepancy
  comptage cumulatif, voir Track F findings) : exact match, cache hit
  TTL, cache miss TTL, offline fallback, severity filter, range bounds.
  Tous green confirme local : `cargo test -p nexus-launcher driver_check
  → 6 passed`.

### Findings

- **E1-1 (P3)** — `parse_version` line 432-436 utilise `seg.parse::<u64>().
  unwrap_or(0)`. Sur un segment non-numerique (e.g. `"535.54.03-rc1"`),
  le segment `"03-rc1"` parse fail → defaut `0` → version devient `[535,
  54, 0]`. Aucun cas reel observed sur les drivers NVIDIA stables (qui
  sont `XXX.YY.ZZ` strict numeric), mais defense en profondeur : un
  driver beta/RC pourrait passer sous-evalue. Fix : log warn quand le
  parse fail, ou retourner Option<Vec<u64>> pour gerer l'erreur explicitement.
  Severite P3.

### Triggers P0/P1 evalues

- ✗ "false-negative version" : range bounds test `535.54.03 IN [535.0.0,
  536.0.0)` ✅, exact CPE match test ✅. Pas de gap demontre.
- ✗ "cache TTL non respecte" : test `cache_miss_when_ttl_expired` valide
  `25h > 24h → miss → fetch`. ✅

---

## Track E2 — Warrant canary : **PASS**

**Question centrale** : signing scheme strict (domain sep, JCS, pubkey
stable), GHA workflow VERIFIE pas SIGNE (dead-man switch), CANARY.txt
human+machine readable ASCII ?

### Verifications effectuees

- `crates/nexus-shell-daemon-core/src/canary.rs` (~480 LOC) :
  - `DOMAIN_WARRANT_CANARY_V1 = b"nexus-warrant-canary-v1"` distinct des
    7 autres domains (TASK, RESULT, CLAIM, INVITE, KUDOS, CURATOR_LIST,
    PROVENANCE). Pas de leak possible cross-domain. ✅
  - `build_canary` line 168-187 : `canonical_bytes(&signed,
    DOMAIN_WARRANT_CANARY_V1)` (JCS RFC 8785 via `nexus_core_rs::canonical`)
    + `signer.sign(&bytes)`. Hex lowercase pour pubkey + signature
    (coherent reste codebase). ✅
  - `verify_canary` line 197-208 : version check + decode_fixed_hex pour
    pubkey/sig + recompute canonical_bytes + verify. Test `verify_canary_
    rejects_wrong_pubkey` confirme : swap pubkey_hex → CanaryError::
    Signature. ✅
  - `MAX_HEADLINE_LEN = 512` cap (DoS guard gossip broadcast). ✅
  - `CanaryBroadcaster: async_trait` mockable + `publish_canary` glue.
  - `format_canary_txt` / `parse_canary_txt` : round-trip ASCII + parser
    line-oriented tolerant whitespace.
- 10 tests inline : build_canary, verify_canary {valid, tampered, wrong
  pubkey}, publish_canary mock, topic_id_deterministic, headline_reject_
  oversize, headline_accept_at_cap, format_contains_fields, parse_round_
  trip. Tous green.
- `.github/workflows/canary-monthly.yml` : header bloc explicite "**What
  this workflow does NOT do**" — refuse explicitement le pattern auto-
  publisher (gag order risk). `permissions: contents: read` only (pas
  write) — workflow ne peut pas commit changes. Job 1 = build daemon +
  run `verify-canary.sh`. Job 2 = grep `Date:` + compute age + fail si
  >45j (dead-man switch via GHA email maintainer). Warn >30j. ✅
- `CANARY.txt` racine repo : format ASCII conforme spec D5, date
  2026-04-15, sig 128 chars hex, pub `80b439cb...3a` 64 chars hex. ✅
- `scripts/verify-canary.sh` : portable bash 3.2+ (no jq, no gpg),
  resolve daemon binary debug→release→.exe, exec `nexus-shell-daemon
  canary verify --input <file>`. ✅

### Findings

- **E2-1 (P3)** — `format_canary_txt` line 238-247 affiche `Next scheduled
  update: {date+45j}` (validity max). Le workflow `canary-monthly.yml:113`
  warn quand age >30j. Le user lisant le `CANARY.txt` voit "Next 2026-05-
  30" et peut ne pas s'inquieter avant cette date, alors que la cadence
  cible est 30j. Ambiguity UX. Fix : afficher "**Next refresh target**:
  date+30, **Stale alarm**: date+45" pour separer cible + validity.
  Severite P3 cosmetique.

### Triggers P0/P1 evalues

- ✗ "GHA workflow auto-publisher" : YAML inspecte ligne par ligne, pas
  de step `canary publish`, `permissions: contents: read` only. Dead-man
  switch preserve.
- ✗ "domain separation leak" : `DOMAIN_WARRANT_CANARY_V1` distinct,
  greppe canonical.rs line 113 confirme. Pas de cross-domain replay.

---

## Track E3 — Codeberg mirror + pivot Radicle : **PASS avec 1 P3**

**Question centrale** : pivot justifie/trace, workflow secure, MIRROR_
FALLBACK §3 self-contained pour flip v1.0 ?

### Verifications effectuees

- `.github/workflows/mirror-codeberg.yml` (~53 LOC) :
  - SPDX header AGPL-3.0. ✅
  - `permissions: contents: read` only. ✅
  - `concurrency: group: mirror-codeberg, cancel-in-progress: false` =
    serialise pushes concurrents. ✅
  - `set -euo pipefail` + guard `CODEBERG_TOKEN` empty. ✅
  - Auth via `git -c "http.https://codeberg.org/.extraheader=
    Authorization: token ${TOKEN}"` — pas URL-embed credential, pas leak
    dans logs git verbose. ✅
  - `git push --mirror codeberg` (full sync : refs + tags + delete refs
    absentes source). ✅
  - timeout 15min + fetch-depth 0. ✅
- `docs/release/MIRROR_FALLBACK.md` (297 lignes) :
  - §1 Rationale + Notes cohabitation (orgs SBFB50 vs SBFB explicite,
    `git push --mirror` destructivite warning). ✅
  - §2 Clone fallback + verif SHA matching commands. ✅
  - §3 Flip sequence v1.0 : 8 sous-sections **3.1-3.8 self-contained** :
    visibilite GitHub+Codeberg, setup Radicle dual identite (maintainer
    + machine account), 5 secrets GHA tableau (RADICLE_IDENTITY_
    {ALIAS,PASSPHRASE,PRIVATE_KEY,PUBLIC_KEY} + REPOSITORY_ID + PROJECT_
    NAME), workflow YAML complet pin SHA `gsaslis/mirror-to-radicle@
    514707f3fc8411f91331f00d7524c76584c10d78`, canary update mirror_urls,
    verification, docs tracking, rotation procedure. **Un maintainer peut
    flip Radicle au tag v1.0 sans re-research**. ✅
  - §4 Maintainer setup Codeberg reference (already done 2026-04-15).
  - §5 Secret rotation 12 mois.
  - §6 Threat model fit (Protects against / Does NOT protect — explicit).
  - §7 Related docs.
- Pivot Radicle→Codeberg : trace dans plan §Phase E3 block dedie, kickoff
  §D5 + §Phase E + §Scope cuts (8 occurrences "Radicle differe v1.0"
  cohabitent), CLAUDE.md §Etat actuel l.208. Aucune occurrence orpheline.
  ✅

### Findings

- **E3-1 (P3)** — Inconsistance casing `RADICLE_PROJECT_NAME` :
  `MIRROR_FALLBACK.md §3.3` line 133 declare le secret avec valeur
  `"SBFB"` (uppercase), mais le workflow YAML §3.4 line 173 hardcode
  `radicle-project-name: sbfb` (lowercase). Le secret n'est pas
  reellement utilise (juste documente "pour coherence"), donc effet
  zero. Fix : aligner sur la valeur reelle utilisee par le workflow
  (`sbfb` lowercase), ou retirer la mention secret PROJECT_NAME du
  tableau §3.3 puisqu'il est inline dans YAML. Severite P3.
- **E3-2 (P3)** — `mirror-codeberg.yml:36` utilise `actions/checkout@v4`
  sans pin SHA. Accept derive intra-major v4 (potential regression sur
  un breaking change v4.x.y → v4.x.z). Pattern S18 a deja pin SHA pour
  `gsaslis/mirror-to-radicle@514707f3...` dans MIRROR_FALLBACK.md §3.4.
  Coherence supply chain : pin egalement `actions/checkout@v4` au SHA
  dans `mirror-codeberg.yml`, `canary-monthly.yml`, `release.yml`,
  `supply-chain.yml`. Severite P3 hardening defense en profondeur.

### Triggers P0/P1 evalues

- ✗ "leak token dans workflow logs" : auth via extraheader (pas URL-
  embed) + GHA mask `${{ secrets.* }}` automatique. Aucun leak detectable
  hors run reel (le repo etant prive, pas de logs publics).
- ✗ "MIRROR_FALLBACK §3 non-self-contained" : 8 sous-sections complete,
  commandes rad inline, tableau 5 secrets, workflow YAML pin SHA. Self-
  contained.

---

## Track F — Wrap-up coherence : **PASS avec 1 P2 + 1 P3**

**Question centrale** : docs Phase F coherents entre eux et avec tip
final, migration PARA complete ?

### Verifications effectuees

- `git log --oneline 4f0727b..HEAD` : 20 commits totaux dans le range,
  9 commits feat/chore(sprint18) directs (Phase A `d7ab281` + B `4ab0211`
  + C `9d0ad7a` + D `94cccb2` + E1 `9f4d19f` + E2 `04c9621` + E3
  `95807b1` + F `4453bfd` + chore(planning) ouverture `1f5cf42`) + ~10
  commits chore(claude) tooling (hors scope S18, documente verification
  §24).
- `ls .planning/active/` : un seul fichier residual `sprint18_phase_F_
  review.md` (review nexus-phase-auditor du commit Phase F, conserve
  local — pas migre car cree post-commit).
- `ls .planning/archive/v1.2/sprint18_*.md | wc -l` : **10 fichiers**
  (kickoff + plan + verification + audit_plan + 6 phase reviews B/C/D/E1/
  E2/E3). ✅
- `CLAUDE.md §Etat actuel` ligne 156-215 : tip post-Sprint 18 mention
  4 phases SHAs reels (4ab0211 / 9d0ad7a / 94cccb2 / 9f4d19f / 04c9621 /
  95807b1), date 2026-04-15, compteurs 474 Rust + 183 SDK + 187+3 coord +
  46 gov + 239 Vitest + 38 Playwright + 7/7 size + 246+ SPDX (~1172 tests),
  delta S18 +44 Rust, Gate 1 UNLOCKED, pivot E3 explicite. ✅
- `SPRINT_LOG.md row S18` ligne 21 : theme "quick wins + supply chain
  baseline + multi-relai phase 1 + Gate 1 unlock", commits stack reel,
  pivot Radicle→Codeberg explicite. ✅
- Memory `nexus_grid_pivot.md` : tip sync post-Phase F (reference au
  commit `4453bfd`).
- Tip courant `4453bfd` confirme par `git log --format=%H -1`.

### Findings

- **F-1 (P2)** — Discrepancies docs hygiene (4 sous-items, **deja flag
  par Phase F review livreur P2-1** mais merite escalade en audit S18
  parce qu'il y a en fait 4 docs concernes pas 2) :
  - `verification.md §57` dit "phase_E1_review.md (non-present — E1
    review absent, P2/P3 fixes inline au moment du commit)". **Realite**
    : `sprint18_phase_E1_review.md` existe en archive (cree au commit
    `9f4d19f` par nexus-phase-auditor). Le verification sous-rapporte le
    livrable.
  - `audit_plan.md §213` dit "9 files attendus" pour la migration PARA.
    **Realite** : 10 files (E1 review inclus).
  - `SPRINT_LOG.md row S18` ligne 21 dit "5 phase reviews B/C/D/E2/E3" —
    omet E1 (6 reviews reel B/C/D/E1/E2/E3).
  - `verification.md §1-5` + `SPRINT_LOG.md row 21` ont placeholders
    `<wrap-up>` `<this>` non resolus en tip reel `4453bfd`. Cosmetique
    pour la 2eme, mais embete une session fraiche qui cherche le SHA
    pour `git log <wrap-up>..HEAD`.
  - Fix : 1 commit `fix(sprint18): F-1 — resolve doc discrepancies on
    phase E1 review presence + tip placeholder` qui :
    1. Update verification.md §57 → "E2_review.md (PASS), E3_review.md
       (PASS), E1_review.md (PASS)" (plus de "non-present").
    2. Update audit_plan.md §213 → "10 files attendus".
    3. Update SPRINT_LOG.md row 21 → "6 phase reviews B/C/D/E1/E2/E3".
    4. Resoudre placeholders verification.md §1-5 + SPRINT_LOG row 21
       a `4453bfd`.
  - Severite P2 (pas P1) parce que (a) cosmetique, n'invalide pas le
    livrable code S18 ; (b) une session fraiche peut deduire les vrais
    SHAs depuis git log + ls archive/. Mais doit etre fixe avant audit
    S19 lit ces docs.
- **F-2 (P3)** — Phase A SHA `<A-sha>` placeholder dans verification.md
  §19 non resolu (reel = `d7ab281` confirme par git log). **Deja flag par
  Phase F review livreur P3-1**. Inclus dans le fix F-1 ci-dessus pour
  efficacite.

### Triggers P0/P1 evalues

- ✗ "Migration PARA incomplete" : 10 files migres, `.planning/active/`
  vide sauf `sprint18_phase_F_review.md` residual.
- ✗ "Tip incoherent entre docs" : SHAs reels CLAUDE.md = SPRINT_LOG.md =
  git log. Seul le placeholder `<wrap-up>` n'est pas resolu, mais deduisible.

---

## Meta-track — Radicle-v1.0 activation tracking : **PASS avec 1 P2 carry**

**Question centrale** : item Radicle-v1.0 a un landing spot durable
(deadline, owner, runbook) qui resiste a la cloture S18 + changements de
session ?

### Verifications effectuees

- `audit_plan.md §170-198` contient le block tracking complet :
  - Owner : maintainer (FlowUP) ✅
  - Deadline : jour du tag v1.0 ✅
  - Blocker : tag v1.0 prerequisite (flip GitHub+Codeberg public d'abord)
  - Runbook : `docs/release/MIRROR_FALLBACK.md §3` (8 sous-sections 3.1-
    3.8 self-contained, verifie Track E3 ci-dessus) ✅
  - Resources : VM Linux disponible, action pinned SHA, 5 secrets GHA a
    creer, post-activation checks (workflow vert, rad clone, Explorer,
    canary mirror_urls update, MIRROR_FALLBACK status update)
- Le item est dans `audit_plan.md S18` qui est lui-meme dans
  `archive/v1.2/` — il survit la cloture S18.

### Findings

- **Meta-1 (P2 carry)** — L'item Radicle-v1.0 n'est PAS automatiquement
  propage dans `sprint19_kickoff.md §3 items carry/dette` (pour la
  raison evidente que S19 n'est pas demarre). Si le kickoff S19 oublie
  de re-pointer ce tracking, l'item devient orphelin (audit_plan S18 ira
  en archive/v1.2/, plus jamais lu). **Le `audit_plan.md §198 Fix si
  omis`** mentionne deja cette mitigation : "ajouter au kickoff S19 (ou
  sprint release v1.0) §3 items carry/dette cet item avec deadline +
  owner".
  - Recommendation pour Sprint 19 Phase 0 (cette session) : **rappeler
    explicitement au kickoff writer** d'ajouter dans `sprint19_kickoff.md
    §3 items carry/dette` :
    ```
    [Radicle-v1.0 activation tracking]
    - Owner : FlowUP
    - Deadline : jour du tag v1.0 (sprint release)
    - Runbook : docs/release/MIRROR_FALLBACK.md §3 (self-contained)
    - Source : .planning/archive/v1.2/sprint18_audit_plan.md §170-198
    ```
  - Severite P2 : pas un blocage S19 Phase A, mais doit etre verifie au
    kickoff S19 sinon item perdu.

---

## Findings list sorted by severity

| ID | Severite | Track | Fichier(s) | Resume |
|---|---|---|---|---|
| **D-1** | **P1** | D | `crates/nexus-shell-daemon/src/http.rs:137-150`, `runtime.rs:262`, `verification.md §67` | TokenRotator primitive non-cablee au router daemon ; verification §Gate 1 unlock claim contredit |
| A-1 | P2 | A | `.github/workflows/supply-chain.yml:63` | `arg: --workspace` cargo-deny obsolete v0.14+, CI faux-fail risque |
| B-1 | P2 | B | `.github/workflows/release.yml:39-43` | Wheel `nexus-core-py` sans attestation SLSA in-toto |
| C-1 | P2 | C | `crates/nexus-core-rs/src/dht_quorum.rs` (call sites) | `redundant_resolve` non wirée à browse aggregator / curator runtime |
| F-1 | P2 | F | `verification.md §57`, `audit_plan.md §213`, `SPRINT_LOG.md row 21`, `verification.md §1-5` | 4 discrepancies docs : phase_E1_review presence, file count 9 vs 10, "5 reviews" omet E1, placeholders `<wrap-up>` non resolus |
| Meta-1 | P2 carry | Meta | `audit_plan.md §170-198` | Item Radicle-v1.0 a re-injecter explicitement dans `sprint19_kickoff.md §3` |
| B-2 | P3 | B | `scripts/release-attest.sh:124` | `buildType` URI non-standard `container-based-build` pour script bash |
| E1-1 | P3 | E1 | `crates/nexus-launcher/src/driver_check.rs:432` | `parse_version` silently default 0 sur segment non-numerique |
| E2-1 | P3 | E2 | `crates/nexus-shell-daemon-core/src/canary.rs:238-247` | "Next scheduled update" affiche +45j (validity) au lieu de +30j (target) — UX ambigue |
| E3-1 | P3 | E3 | `MIRROR_FALLBACK.md §3.3 vs §3.4` | Inconsistance casing `RADICLE_PROJECT_NAME` "SBFB" vs `sbfb` |
| E3-2 | P3 | E3 | `.github/workflows/*.yml` | `actions/checkout@v4` non-pin SHA (defense en profondeur supply chain) |
| F-2 | P3 | F | `verification.md §19` | Phase A SHA `<A-sha>` placeholder non resolu = `d7ab281` |

**Total** : 0 P0, 1 P1, 5 P2, 6 P3.

---

## Commits fix attendus avant Sprint 19 Phase A

### D-1 (P1, blocking) — choisir au moins une option

**Option A (fix code, recommandee si Gate 1 unlock veut etre tenu
litteralement)** :

```
fix(sprint18): D-1 — wire TokenRotator into shell-daemon HTTP router

Cable le TokenRotator existant (S18 Phase D primitive) au router HTTP
daemon via Arc<RwLock<_>> + notify file-watcher sur tokens.json.
Replace l'AuthState::new(token) statique de build_router par un
AuthState dynamique qui re-lit tokens.json chaque fois que le
file-watcher signal un changement (pattern S16 ConsentWatcher).

Files :
- crates/nexus-shell-daemon-core/src/auth.rs : extend AuthState avec
  une variante Rotated(Arc<RwLock<TokenRotator>>) en plus de Static.
  validate_token utilise validate_token_with_rotator quand Rotated.
- crates/nexus-shell-daemon/src/http.rs : build_router accept Either<
  String, Arc<RwLock<TokenRotator>>>.
- crates/nexus-shell-daemon/src/runtime.rs : load_or_init token au
  boot via TokenRotator::load(tokens_file_path) → build router avec
  variante Rotated.
- crates/nexus-shell-daemon-core/src/auth.rs : tests +3 (Rotated
  accepts current+previous, rejects after overlap, file change
  triggers re-load).

Tests delta : +3 Rust, total 477.
Closes finding D-1 sprint18_audit_findings.md.
```

LOC estimee : ~150 code + ~80 tests.

**Option B (fix docs, recommandee si Gate 1 unlock peut etre nuance)** :

```
fix(sprint18): D-1 — clarify token rotation status as primitive-only

Le carry-over admis dans le commit body Phase D (94cccb2) — wiring du
TokenRotator au router HTTP daemon — n'est pas reflete dans
verification.md §Gate 1 unlock ni CLAUDE.md §Etat actuel. Audit S18
finding D-1 le flag P1.

Cette commit corrige la divergence narrative en marquant explicitement
"primitive livree, runtime wiring deferred S19" dans :

- .planning/archive/v1.2/sprint18_verification.md §Gate 1 unlock
  ligne 67 : "[x] Coord-side wire complete : TaskEntry craft +
  estimate caps + token rotation **primitive** (Phase D, runtime
  wiring deferred S19 carry-over admis commit body)".
- CLAUDE.md §Etat actuel ligne 202 : ajouter "(rotation primitive
  livree, runtime cabling pending Sprint 19)".
- SPRINT_LOG.md row S18 : meme nuance.
- docs/security/HARDENING_ROADMAP §7 Gate 1 row : retirer "[x] Token
  rotation 24h" et flag "[~] Token rotation 24h primitive (wiring
  S19)".

Closes finding D-1 sprint18_audit_findings.md.
```

LOC estimee : ~30 lignes docs.

### F-1 (P2, recommande mais pas blocking)

```
fix(sprint18): F-1 — resolve doc discrepancies (phase E1 review present,
                     file count 10, tip SHA placeholders)

Quatre divergences docs hygiene flaggees par audit S18 finding F-1 :

1. verification.md §57 disait "phase_E1_review.md (non-present)" alors
   que le fichier existe en archive/v1.2/ (cree au commit 9f4d19f par
   nexus-phase-auditor).
2. audit_plan.md §213 disait "9 files attendus" → reel 10 (E1 inclus).
3. SPRINT_LOG.md row S18 disait "5 phase reviews B/C/D/E2/E3" → reel
   6 (B/C/D/E1/E2/E3).
4. verification.md §1-5 + SPRINT_LOG row 21 placeholders <wrap-up>
   non resolus → 4453bfd.

Plus Phase A SHA placeholder <A-sha> (verification.md §19) → d7ab281
(F-2 audit findings).

Closes findings F-1 + F-2 sprint18_audit_findings.md.
```

LOC estimee : ~15 lignes docs.

### Meta-1 (P2 carry, traiter au kickoff S19)

Au moment d'ecrire `sprint19_kickoff.md`, **ajouter explicitement**
dans la section §3 "items carry/dette" :

```
[Radicle-v1.0 activation tracking]
- Owner : FlowUP
- Deadline : jour du tag v1.0 (sprint release)
- Runbook : docs/release/MIRROR_FALLBACK.md §3 (self-contained 3.1-3.8)
- Resources : VM Linux + 5 secrets GHA + action pinned SHA
- Source : .planning/archive/v1.2/sprint18_audit_plan.md §170-198
```

Verifie au moment d'ecrire le kickoff. Pas de commit fix necessaire.

### P2 restants (A-1, B-1, C-1) : non-blocking pour S19 Phase A

- **A-1** : verifier au premier run reel CI post-S18. Si fail, fix dans
  un commit de tete `fix(sprint18): A-1 — drop --workspace from cargo-
  deny job (default since v0.14)`. ~3 LOC YAML.
- **B-1** : decision design — soit etendre matrix release.yml avec wheel
  attestation (~30 LOC YAML), soit documenter explicitement le gap dans
  `REPRODUCIBLE_BUILDS.md §5 Limitations connues` (~10 lignes docs).
  Reportable Sprint 20+ qui touche au release pipeline.
- **C-1** : decision design — wirer `redundant_resolve` au browse
  aggregator est ~80 LOC mais demande analyse du browse current single-
  lookup pattern. Reportable Sprint 19 (qui touche au DHT et P2P
  hardening) ou Sprint 22+. Documenter dans verification.md §Gate 1
  unlock entre temps : "[~] DHT redundant lookup primitive (wiring S19+)".

### P3 : laisses sans action

- **B-2** : `buildType` URI cosmetique. Slsa-verifier accepte tout pourvu
  que les digests matchent.
- **E1-1** : `parse_version` silently 0 — defense en profondeur, pas de
  cas reel observed. Loguer dans `docs/rust/PATTERNS.md` comme tech
  debt.
- **E2-1** : "Next scheduled update" UX. Loguer dans tech debt.
- **E3-1, E3-2** : nits docs casing + supply chain hardening. Reportable
  sprint security ops futur.
- **F-2** : inclus dans fix F-1 ci-dessus.

---

## Out of scope (respecte)

L'audit n'a pas rebattu :

- D1 ordre des phases (A→F sequenced)
- D2 wasmtime pin `>=43.0.1` vs `=43.0.1` exact
- D3 cargo-deny seul vs `cargo-audit` standalone
- D4 placeholders federation S18 vs concretes ONG-relays S19+
- D5 warrant canary format Ed25519 + JCS hex (vs PGP, vs base64, vs
  centralized keyserver)
- Pivot Radicle → Codeberg pre-launch (decision documente plan §E3 +
  kickoff §D5)

L'audit n'a pas teste l'aspect "wasmtime ban CI fail negatif" (PR fixture
introducing `wasmtime = "42.0.0"` → red gate) parce que le repo etant
prive, pas d'acces a un fork de test, et `cargo deny check` local n'a
pas wasmtime dans le dependency graph (le ban est preemptif). A verifier
au prochain PR qui touche aux deps Cargo.

---

## Notes on audit completeness

- **Skipped** : verifier `actions/cargo-deny-action@v2` quelle version
  cargo-deny installe par defaut (necessite WebFetch repo embarkstudios
  ou test sur GHA reel). Si l'action installe v0.13 ou anterieur, le flag
  `--workspace` est accepte et A-1 devient P3 cosmetique.
- **Skipped** : test repro builds end-to-end sur GHA Linux/macOS/Windows
  matrix (timebox + repo prive). Tests smoke locaux passes par audit
  manuel (`reproducible-build.sh` + `attestation-schema.sh`).
- **Skipped** : cosign verify-blob avec le certificat Sigstore reel d'un
  premier run release (pas encore de tag v1.x sur le repo).
- **Skipped** : profil de test SAST Semgrep sur l'ensemble du diff S18
  (delegued aux phase reviews livreur qui ont tous run `nexus-phase-
  review` skill avec scan Semgrep — convergent 0 finding sur fichiers .rs
  cibles).
- **Reproductibilite audit** : l'auditeur a re-run `cargo deny check`
  (vert) + `cargo test -p nexus-launcher driver_check` (6/6 passed) +
  `git log --oneline 4f0727b..HEAD` (20 commits) + grep multiple sur
  call sites `redundant_resolve` + `validate_token_with_rotator` +
  `build_router` — tous results inclus en context dans les findings
  ci-dessus.

---

## Resume executif (1 paragraphe)

Sprint 18 livre une **vraie progression supply-chain + repro builds +
warrant canary + Codeberg mirror** mais 1 finding P1 (D-1) bloque le
demarrage Sprint 19 Phase A : la rotation token X-SBFB-Token est livree
comme **primitive testee** mais **non cablee au router HTTP daemon**, et
le verification.md §Gate 1 unlock fait une promesse "Coord-side wire
complete + token rotation" qui contredit ce carry-over admis dans le
commit body Phase D. Resolution attendue via 1 commit `fix(sprint18):
D-1` au choix entre Option A (wire rotator au router, ~150 LOC) ou
Option B (clarifier verification.md / CLAUDE.md / SPRINT_LOG en
"primitive only, runtime wiring deferred S19", ~30 LOC docs). Apres
landing du fix D-1, Sprint 19 Phase A peut demarrer. Les 5 P2 + 6 P3
restants sont non-blocking, certains carry-overs naturels (DHT quorum
wiring S19, wheel attestation S20+, RADICLE_PROJECT_NAME casing nit) et
1 P2 docs hygiene (F-1) merite un commit fix vite-fait.
