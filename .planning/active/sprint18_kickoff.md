# Sprint 18 — Kickoff (Quick wins + supply chain baseline + multi-relai phase 1)

**Ecrit** : 2026-04-15 (session fraiche post-S17 wrap-up).
**Type** : **sprint implementation** (retour au code apres le sprint recherche S17).
**Tip master d'entree** : `4f0727b` (audit-P1 S17 resolved docs-only).
**Phase 0 audit Sprint 17** : **DEJA JOUE** — findings dans
`.planning/archive/v1.2/sprint17_audit_findings.md` (migre via
`git mv` avec ce 1er commit S18), verdict CONDITIONAL PASS leve
via le seul commit `4f0727b` (7 P1 tous docs). La session fraiche
qui demarre Sprint 18 Phase A verifie via `git log` que le tip
master courant >= `4f0727b` et ne rejoue pas l'audit.

**Sources context7 + WebSearch consultees** pour cadrer D1..D5 :
- `bytecodealliance.org/articles/wasmtime-security-advisories`
  (12 CVE avril 2026, 2 Critical, fix 43.0.1/42.0.2/36.0.7/24.0.7)
- `rs_iroh` `RelayMode::Custom(RelayMap)` API (iroh 0.97)
- `embarkstudios_github_io_cargo-deny` advisories + bans + licenses
- `github.com/pypa/pip-audit` + `github.com/IBM/audit-ci` CI gates
- `slsa.dev/spec/v1.0` in-toto attestation + SOURCE_DATE_EPOCH
- `github.com/gsaslis/mirror-to-radicle` machine account pattern
- `nvd.nist.gov/developers` NVD API REST + CPE filter NVIDIA

---

## 1. Constat d'entree

### 1.1 D'ou on part

Sprint 17 a livre le modele d'adversaires T0-T5, la cartographie
P2P threats, la taxonomie compute-sharing threats, une roadmap
S18-30 chiffree, et un VALIDATED_BLUEPRINT 13 couches avec
50+ briques OSS validees contre docs 2026. **Zero code**, 100%
recherche.

Le projet sait maintenant :
- **qui** attaque (T0-T5 fiches),
- **comment** (27 threats matrix × surface × tier),
- **dans quel ordre durcir** (HARDENING_ROADMAP §3 Sprint 18-30),
- **avec quelles briques** (VALIDATED_BLUEPRINT OSS validees).

Sprint 18 est le **premier sprint d'implementation** de cette
roadmap. Son mandat est clair : **fermer les quick-wins
<100 LOC + installer la chaine supply-chain reproductible +
amorcer la federation multi-relai phase 1** pour debloquer Gate 1
(DnD Forge deployable en beta fermee fin S18).

### 1.2 Ancrage HARDENING_ROADMAP §3 Sprint 18

La roadmap Phase D S17 specifie Sprint 18 items :

| Item | LOC roadmap | Source |
|---|---|---|
| `cargo-audit` / `pip-audit` / `npm audit` CI | ~150 | §3 S18 |
| Reproducible builds Rust (`--locked`, `SOURCE_DATE_EPOCH`) + SHA256 | ~200 | §3 S18 |
| Radicle mirror + warrant canary minimal | ~300 | §3 S18 |
| Driver update check launcher (NVIDIA CVE scrape) | ~250 | §3 S18 |
| Multi-relai federation phase 1 (n0 + 2 fallbacks, round-robin) | ~400 | §3 S18 |
| DHT redundant lookup (3 pkarr paralleles, quorum 2/3) | ~200 | §3 S18 |
| **Carry S16** token rotation automatique | ~150 | §4 quick-wins table |
| **Dette S16** coord-side wire TaskEntry (`is_open_source` + estimates) | ~100 | sprint17_kickoff §3 dette |

Total roadmap+carry : **~1750 LOC**. Nous y ajoutons un 9e item
decouvert post-S17 (cf. D2 ci-dessous) : **wasmtime pin CVE
avril 2026** (~50 LOC config + CI check) — total ~1800 LOC.

**Gate unlock** fin S18 : Gate 1 (DnD Forge — Tier T0-T1).

### 1.3 Le declencheur wasmtime avril 2026

Post-S17 Phase E (VALIDATED_BLUEPRINT), l'audit des zones rouges
a identifie 3 risques P0 :

- **R-iroh-audit** : iroh 0.97 sans audit public + sans
  `SECURITY.md` → **hors-scope S18**, necessite partnership +
  budget audit externe (Gate 3 S29). Loggue en item tracking
  `docs/security/HARDENING_ROADMAP.md` mais pas d'action S18.
- **R-pyodide-escape** : CVE-2025-68668 classe n8n CVSS 9.9, besoin
  Wasmtime process isolation → **hors-scope S18**, depends Sprint 22+
  sandbox big-rock (encryption at rest prerequis S20 d'abord).
- **R-wasmtime-cve** : **12 CVE 9 avril 2026 Bytecode Alliance**,
  2 Critical (CVE-2026-34941 heap OOB UTF-16 transcoding,
  CVE-2026-34946 Winch table.fill panic). **Dans-scope S18** :
  SBFB **n'utilise pas encore** wasmtime runtime (pas de Pyodide
  sandbox cote worker pour le moment) mais `hello-world-app`
  example + future worker runtime utiliseront. Pin preemptif dans
  tous `Cargo.toml` qui declarent `wasmtime` OU mention
  explicite "wasmtime deps a ajouter uniquement en 43.0.1+ / LTS
  36.0.7+" + CI guard cargo-deny `[bans] deny = [wasmtime@<43.0.1]`.
  Couvert en Phase A item 9 (~50 LOC).

### 1.4 Compteurs de tests a l'entree (tip `4f0727b`)

| Suite | Count observe entree S18 |
|---|---|
| Rust workspace | 430 (425 tests + 5 doc-tests) |
| Python SDK | 183 |
| Python coordinator | 187 + 3 skipped |
| Python app-gov | 46 |
| Vitest unit | 239 |
| Playwright | 38 |
| size-limit | 7/7 |
| SPDX | 246+ |
| **Total** | **~1128 tests** |

**Delta Sprint 18 attendu : +50 a +60** (les CI linters ajoutent
0 test runtime mais Phase C multi-relai + Phase D driver-check +
token rotation + wire-through ajoutent tests unit). Repartition
estimee : +20 Rust (multi-relai + DHT quorum + token rotation),
+15 Coordinator (TaskEntry wire + pip-audit integration test),
+10 Web (size-limit update wasmtime-free, pas plus), +10
Playwright (smoke Gate 1 checklist).

### 1.5 Pre-launch protocol policy (rappel)

Sprint 17 a confirme la regle : `*_VERSION = 1` jusqu'au tag v1.0,
pas de tolerant decoder multi-version. Sprint 18 respecte : aucun
item liste ci-dessus ne touche un wire format. Si un item (ex:
item 7 wire-through TaskEntry) enrichit le JSON runtime, c'est un
field `#[serde(default)]` legitime pour runtime robustness, pas
un bump version.

---

## 2. Goal en une phrase

**Le projet livre une baseline supply-chain complete (`cargo-deny`
+ `pip-audit` + `npm audit` + `wasmtime` pin avril 2026 en CI
bloquant), passe en reproducible builds Rust avec SLSA in-toto
attestation + SHA256 per-artifact, amorce une federation
multi-relai phase 1 (iroh `RelayMode::Custom` bootstrap n0 + 2
fallbacks + DHT pkarr redundant 3-paralleles-quorum-2/3), ferme
la dette S16 (coord-side wire `is_open_source` + estimates dans
TaskEntry au craft, token rotation automatique), et installe un
check driver NVIDIA CVE au launcher startup + warrant canary
Ed25519-signe + Radicle mirror — debloquant Gate 1 (DnD Forge
beta fermee) fin de sprint.**

---

## 3. Phase 0 — Audit Sprint 17 (DEJA JOUE — verdict CONDITIONAL PASS → LEVE)

**Status** : JOUE session 2026-04-14 (~2h, 5 tracks parallele + 2
sequentiels). Ne pas rejouer. Cf.
`.planning/archive/v1.2/sprint17_audit_findings.md` (migre avec
ce 1er commit S18).

**Commit stack du gate (leve)** :

```
4f0727b fix(sprint17): audit-P1 — resolve 7 findings from S18 Phase 0 audit
```

1 seul commit docs-only (0 code touche) a ferme les 7 P1 :
G-1 3 stubs RELEASE_GATES + PARTNERSHIPS + DISCLOSURE,
D-1 Gate 3 Sprint 29 table clarif, A-1 T4 partial Gate 3 mapping,
A-2 standardisation symboles ❌/⚠️/✅, B-1 Sybil tier T2+
pre-S19, C-1 Carlini 2024 attribution, E-1 libp2p-gossipsub vs
iroh-gossip disambiguation. Les 19 P2 sont loggees comme dette
docs a reprendre au fil des sprints S18-S30. Les 13 P3 sans
action.

**Verdict final** : **PASS**. Sprint 18 Phase A non-bloque.

**Dette heritee Sprint 16 confirmee** : coord-side wire TaskEntry.
Le coord emet actuellement des tasks avec `is_open_source: false`
et `estimated_*: 0` par defaut. Les fonctions `should_accept_task`
+ `runtime.rs` les lisent correctement (fix C-1/C-2 S16), mais le
coord ne les REMPLIT pas encore cote craft. **Phase D S18 item 7**.

---

## 4. Decisions Day 0 (D1..D5)

### D1 — Perimetre items + ordre des phases

**Retenu** : 9 items (8 roadmap+carry + 1 wasmtime-pin). Regroupes
en 6 phases A-F selon couplage technique et risque.

| Phase | Items couverts | Rationale regroupement |
|---|---|---|
| A — Supply chain CI | cargo-deny (item 1) + pip-audit + npm audit + wasmtime pin (item 9) | Tous sont des guards CI independants, testables en 1 PR, livrables day 1 |
| B — Reproducible builds + SLSA | item 2 + attestation in-toto SHA256 | Prerequis pour tout artefact Gate 1+ livrable |
| C — Transport P2P durci | multi-relai federation (item 5) + DHT redundant (item 6) | Meme stack iroh, testable en integration tests E2E |
| D — Coord-side wire + token rotation | dette S16 TaskEntry (item 8) + token rotation (carry 7) | Touches coord + daemon, risque regression minimal |
| E — Radicle mirror + warrant canary + driver check | items 3 + 4 | Trois items externes ops (GitHub Actions + gossip + NVD scrape), independants du runtime P2P |
| F — Verification + audit plan S19 | consolidation + kickoff S19 audit | Fin de sprint standard |

**Rejete** :

- **Ordre alternatif "multi-relai first"** : Phase C fait defense P2P
  mais depend deja de `Endpoint::builder()` existant S13 — pas de
  prerequis supply-chain strict, donc A+B d'abord (moins de risque
  landing A/B, plus critique pour Gate 1).
- **Fusion A+B "1 phase supply-chain"** : A = CI config pure (YAML +
  cargo-deny.toml), B = build tooling + attestation. Scope distinct.
- **Phase E decoupage en 3 phases separees** : driver check +
  warrant canary + Radicle sont independants mais petits (~850 LOC
  cumule), 1 phase avec 3 commits internes ou 3 phases ? Decision :
  **1 phase 3 commits internes** pour eviter l'inflation de phases.

**Rationale ordre A→F** : on ferme les CI guards (A) avant de
lander n'importe quel code B-E (sinon les PRs nouvelles pourraient
introduire un CVE non detecte). B avant C-D-E car reproducible
builds est prerequis pour attestation des artefacts Gate 1 livres
en fin de sprint.

### D2 — wasmtime pin version politique

**Retenu** : deux tiers de politique, enforcee via `cargo-deny`.

1. **Mainline** (pre-Gate 2) : pin `wasmtime = "=43.0.1"` exact
   version match OU `">=43.0.1, <44"` si on veut accepter patches.
   Currently : SBFB ne declare pas encore `wasmtime` comme dep —
   on ajoute preemptivement dans `Cargo.toml` workspace `[workspace.metadata.deny.bans]`
   la regle : `{ name = "wasmtime", version = "<43.0.1" }` → CI
   echoue si un futur PR tente d'introduire wasmtime sans pin
   correct.
2. **LTS path** (si adoption grandit, Gate 3+) : bascule vers LTS
   36.0.7+ (support jusqu'a ~2027 via LTS 12-majors-cycle Bytecode
   Alliance). Decision re-evaluee Sprint 22+ si Wasmtime adopt.

**Rejete** :

- **Pin exact "=43.0.1"** : trop strict, patches minor securite
  futurs bloques. On prefere `>=43.0.1, <44` permissif sur patches.
- **LTS 36.0.7 des maintenant** : LTS 36 est older (release avril
  2025 + 24 mois support → oct 2026). S18 S22 auront 43.x LTS
  48.x dispo, preferer latest stable + CVE patches direct.
- **Skip wasmtime pin** (car SBFB ne l'utilise pas encore) : un
  futur PR (Gate 2+ sandbox) peut introduire wasmtime sans
  verification — Phase A item 9 pose le garde preemptif.

**Source** :
[Bytecode Alliance advisory April 9 2026](https://bytecodealliance.org/articles/wasmtime-security-advisories)
+ [Wasmtime release docs stability-release.md](https://github.com/bytecodealliance/wasmtime/blob/main/docs/stability-release.md)
via context7 `/bytecodealliance/wasmtime`.

### D3 — cargo-deny plutot que cargo-audit

**Retenu** : **`cargo-deny`** comme seule CI step Rust supply-chain.

Sources (context7 `/websites/embarkstudios_github_io_cargo-deny`
+ WebSearch "cargo-audit vs cargo-deny 2026") concordent : RustSec
recommande officiellement `cargo-deny-action` GitHub Actions comme
frontend audits + bans + licenses + sources. `cargo-audit` seul
couvre advisories RUSTSEC uniquement, `cargo-deny` couvre 4
categories en 1 tool.

Configuration `deny.toml` :

```toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
yanked = "deny"
ignore = []  # empty at S18 start, P2 tech-debt comes later

[bans]
multiple-versions = "warn"
wildcards = "allow"  # S19+ tighten
deny = [
  { name = "wasmtime", version = "<43.0.1" },  # R-wasmtime-cve
]

[licenses]
allow = ["Apache-2.0", "MIT", "BSD-3-Clause", "BSD-2-Clause",
         "ISC", "Unicode-DFS-2016", "Unicode-3.0", "Zlib", "CC0-1.0"]
confidence-threshold = 0.8

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-git = []  # explicit allowlist per-use S19+
```

**Rejete** :

- **Les deux en parallele** : duplication checks RUSTSEC, double
  CI time sans gain. `cargo-deny` englobe tout.
- **`cargo-audit` seul** : rate les licenses + yanked + duplicates.
  Moins defense in depth.

**Source** : [RustSec recommends cargo-deny-action](https://rustsec.org/)
+ [cargo-deny advisories docs](https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html).

### D4 — Multi-relai bootstrap list phase 1

**Retenu** : `RelayMode::Custom(RelayMap::from_iter([n0_na, n0_eu, n0_ap]))` 3 relais n0 officiels (deja `prod::default_relay_map()`
equivalent), **plus** 2 "placeholder slots" documentes
`# TODO S19: replace with federation ONG-run relay when available`.

Le code S18 phase 1 n'ajoute PAS encore de vrais relais ONG
self-hosted (hors-capacite solo sprint). Il installe le
**mecanisme** (retry round-robin, health check 30s, failover
automatique) et les **hooks de configuration** (env var
`SBFB_CUSTOM_RELAYS` + fichier `~/.sbfb/relays.json`) pour que
Phase S19 ou un contributeur communaute puisse ajouter un relai
custom sans toucher le code core.

**Rejete** :

- **Ajouter 2 relais publics externes (ex: public iroh-relay
  community)** : dependance externe non-auditee, risk cascade
  de compromission. Mieux : configuration-based pour que
  l'utilisateur pro-actif ajoute son propre relai.
- **Pas de placeholders** : UX pauvre, l'utilisateur ne comprend
  pas qu'il peut federer.

**API iroh 0.97 cible** (source context7 `/websites/rs_iroh`) :

```rust
use iroh::{Endpoint, RelayMode};
use iroh_relay::{RelayMap, RelayNode};

let custom_relays = load_custom_relays_from_config()?;
let relay_map = if custom_relays.is_empty() {
    // Default n0 prod relays (3 regions NA/EU/AP)
    iroh::defaults::prod::default_relay_map()
} else {
    // User-provided federated relays
    RelayMap::from_iter(custom_relays)
};

let ep = Endpoint::builder()
    .relay_mode(RelayMode::Custom(relay_map))
    .bind()
    .await?;
```

**Decision round-robin + failover** : `iroh` v0.97 gere deja
`home_relay` selection automatique via `RelayMap` multi-node. Le
travail S18 se limite a : (a) charger custom config, (b) ecrire
test integration "relay 1 down → connect via relay 2", (c)
loguer `home_relay` selection pour diagnostic.

### D5 — Warrant canary format

**Retenu** : message Ed25519-signe (cle node_id local du
developpeur principal) publie via `iroh-gossip` monthly sur un
topic dedie `sbfb-warrant-canary-v1`, mirror GitHub dans
`CANARY.txt` a la racine du repo.

Format du message :

```
SBFB Warrant Canary
Date: 2026-04-15 (UTC)
Headline: <copy-paste d'un titre majeur news day-of, ex: "NYT 2026-04-15: ...">

Declaration:
  As of the date above, the SBFB project maintainer(s) have NOT:
  - received any National Security Letter, secret subpoena,
    or gag order from any government agency
  - been compelled to modify or backdoor any code or cryptographic
    key material used by the project
  - been compelled to provide user data to any third party

  This canary is signed and published monthly. If the canary is
  not updated for >45 days, assume it has been compromised OR
  the project has been compelled and cannot disclose.

Next scheduled update: 2026-05-15

Signed:
  Ed25519 signature over the above bytes (SHA256 canonical UTF-8)
  sig: <base64-Ed25519-signature>
  pub: <base64-Ed25519-pubkey matching node_id>
```

Le canary est **aussi mirror sur Radicle** (via GitHub Action
`gsaslis/mirror-to-radicle`, cf. item 3) pour redondance
decentralisee : si GitHub repo est saisi, Radicle mirror persiste.

**Rejete** :

- **PGP (format classique warrant canary VPN)** : dep externe
  (gpg install sur launcher Windows mal supporte). On a deja
  Ed25519 via node_id existant — reutilisation crypto primitive.
- **Gossip-only sans GitHub mirror** : un nouvel installateur sans
  historique gossip (frais boot) ne peut pas verifier. Mirror git
  rend le canary accessible a tout cloner.

**Source** :
- Format declaration : [Wikipedia Warrant canary](https://en.wikipedia.org/wiki/Warrant_canary)
  + exemples [rsync.net canary](https://www.rsync.net/resources/notices/canary.txt)
  et [IVPN canary](https://www.ivpn.net/resources/canary.txt)
- Publication pattern Ed25519 signed + mirror : innovation SBFB
  (pas de precedent public trouve 2026-04-15, mais coherent avec
  pattern `SBFB.json` Keyoxide Ed25519 deja etabli S14).

---

## 5. Plan Phase outline

### Phase 0 — Audit Sprint 17 (DEJA JOUE, verdict PASS)

Migration `sprint17_audit_findings.md` → `archive/v1.2/` via
`git mv` dans le 1er commit S18 (pattern `f75b2c6` S17 open).

### Phase A — Supply chain CI (~300 LOC config, +10 tests ops)

**Scope** :
- `deny.toml` racine workspace Rust (config `cargo-deny`
  advisories + bans + licenses + sources)
- `.github/workflows/supply-chain.yml` (3 jobs paralleles :
  `cargo-deny` / `pip-audit` / `npm audit`)
- Wasmtime ban rule (item 9) dans `deny.toml` `[bans]`
- `pyproject.toml` dev-dep `pip-audit` + script `uv run pip-audit`
- `web/package.json` script `npm run audit:ci` via
  `IBM/audit-ci` avec threshold `critical`
- CI gate : `critical` severity → PR fail, `high` severity → PR
  warn (annotation), `moderate`/`low` → no action

**Livrable commit** : `feat(sprint18): Phase A — supply chain CI baseline (cargo-deny + pip-audit + npm audit + wasmtime pin)`

### Phase B — Reproducible builds + SLSA provenance (~250 LOC, +5 tests)

**Scope** :
- `.cargo/config.toml` workspace : `build.rustflags` deterministe
- `build.rs` ou env vars expose via `shadow-rs` : `SOURCE_DATE_EPOCH` +
  `CARGO_INCREMENTAL=0`
- `scripts/release-attest.sh` : build `--locked` + compute SHA256
  + emit in-toto attestation JSON (schema `slsa.dev/provenance/v1`)
  pour chaque binary release (`nexus-launcher`, `nexus-worker`,
  `nexus-shell-daemon`, wheel `nexus-core-py`)
- `.github/workflows/release.yml` step attestation + publish
  `attestations/*.intoto.jsonl` avec les artefacts GitHub Release
- Doc `docs/release/REPRODUCIBLE_BUILDS.md` (~80 lignes : how to
  verify SHA256 + attestation signature)
- Tests : script `verify-reproducible.sh` qui rebuild twice sur
  meme SOURCE_DATE_EPOCH + compare SHA256 identique

**Livrable commit** : `feat(sprint18): Phase B — reproducible builds + SLSA in-toto attestation`

### Phase C — Multi-relai federation + DHT redundant (~600 LOC, +20 tests)

**Scope** :
- `crates/nexus-core-rs/src/relay_config.rs` : load custom relays
  depuis `~/.sbfb/relays.json` + env `SBFB_CUSTOM_RELAYS`
- `crates/nexus-core-rs/src/endpoint.rs` : integration
  `RelayMode::Custom(RelayMap)` avec fallback n0 prod defaults
- `crates/nexus-core-rs/src/discovery.rs` : DHT pkarr redundant
  lookup (3 pkarr relays en parallele via `tokio::try_join!` +
  majority quorum 2/3 → accept, 1/3 ou 0/3 → reject + log
  warn)
- Tests unit Rust : 12 tests (3 config load, 4 endpoint relay
  mode, 5 DHT quorum scenarios)
- Tests integration : 2 tests E2E (`connect_via_primary_relay`,
  `connect_via_fallback_when_primary_down`)
- Doc `docs/rust/PATTERNS.md` update : section "relay federation"

**Livrable commit** : `feat(sprint18): Phase C — multi-relai federation + DHT redundant lookup`

### Phase D — Coord-side wire TaskEntry + token rotation (~400 LOC, +15 tests)

**Scope** :
- `packages/nexus-coordinator/src/nexus_coordinator/tasks.py` :
  `craft_task()` remplit `is_open_source` (depuis project
  metadata) + `estimated_watts` / `estimated_vram_mb` /
  `estimated_hours` (depuis app SDK `cost_estimate()`)
- Coord emet TaskEntry complet → runtime.rs / consent.rs
  reussissent a filtrer correctement avec ces valeurs (plus de
  default 0/false)
- Tests coord : 10 tests pytest (project open-source true/false
  paths, estimates reels vs default, missing fields fallback)
- `crates/nexus-launcher/src/token.rs` + `nexus-shell-daemon/src/loopback.rs` :
  rotation automatique X-SBFB-Token toutes les 24h avec overlap
  10min (old+new acceptees pendant rotation)
- Tests token : 5 tests rotation (generation, overlap window,
  expire old, reject pre-generation, concurrent requests mid-rotation)

**Livrable commit** : `feat(sprint18): Phase D — coord-side TaskEntry wire-through + X-SBFB-Token rotation`

### Phase E — Driver check + warrant canary + Radicle mirror (~450 LOC, +10 tests)

**Scope subphase E1 — Driver update check (~250 LOC)** :
- `crates/nexus-launcher/src/driver_check.rs` : startup scrape
  `services.nvd.nist.gov/rest/json/cves/2.0` filter
  `cpeName=cpe:2.3:o:nvidia:gpu_display_driver:*` + compare
  driver local version (via `nvml-wrapper` deja dep) + warn si
  CVE affecting version installee
- Cache 24h dans `~/.sbfb/nvd-cache.json` pour eviter rate-limit
  NVD (5 requests/30s sans API key)
- Tests : 5 tests (version comparison, cache hit/miss, NVD API
  mock response, offline fallback, CVE severity filter)

**Scope subphase E2 — Warrant canary (~150 LOC)** :
- `crates/nexus-shell-daemon-core/src/canary.rs` : publish
  monthly via iroh-gossip topic `sbfb-warrant-canary-v1`, mirror
  `CANARY.txt` racine repo
- CLI `sbfb canary publish` (commande manuelle mensuelle
  dev-side) + cron template GitHub Action pour auto-publish

**Scope subphase E3 — Radicle mirror (~50 LOC config)** :
- `.github/workflows/radicle-mirror.yml` (adapt
  `gsaslis/mirror-to-radicle`)
- Doc setup dans `docs/release/RADICLE_MIRROR.md`

**Livrable commit 3 internes** :
1. `feat(sprint18): Phase E1 — NVIDIA driver CVE check at launcher startup`
2. `feat(sprint18): Phase E2 — warrant canary monthly Ed25519 gossip publish`
3. `feat(sprint18): Phase E3 — Radicle mirror GitHub Action`

### Phase F — Consolidation + verification + audit plan S19 (~250 LOC docs)

**Scope** :
- Update `CLAUDE.md` section "Etat actuel" : Sprint 18 CLOSED +
  Gate 1 unlock + commits stack
- Update `docs/claude/SPRINT_LOG.md` : row S18 v1.2
- Update memory `nexus_grid_pivot.md` frontmatter description
- `.planning/active/sprint18_verification.md` : checklist
  fail-fast (CI green, 1128+55 tests, Gate 1 checklist, scope
  respecte)
- `.planning/active/sprint18_audit_plan.md` : tracks A-F pour
  Sprint 19 Phase 0
- Migration planning `.planning/active/sprint18_*.md` →
  `.planning/archive/v1.2/` dans le wrap-up commit

**Livrable commit** : `chore(sprint18): Phase F — wrap-up + verification + audit plan S19 + migrate planning`

---

## 6. Scope cuts (PAS dans ce sprint)

**Iroh audit externe** (R-iroh-audit P0 post-S17) : necessite
partnership + budget Cure53/ToB ~15k€, Gate 3 prerequis (S29).
S18 ajoute un tracking item dans `HARDENING_ROADMAP §tracking-audits`.

**Pyodide sandbox escape mitigation** (R-pyodide-escape P0 post-S17) :
requires wasmtime process isolation big-rock, Sprint 22+ apres
encryption at rest (S20). S18 bloque uniquement les nouveaux deps
wasmtime sans le pin (Phase A item 9).

**PoW gossip** : Sprint 19 (`HARDENING_ROADMAP §3 S19`). Blocker
dependency pour rate-limit Sybil-resistant Sprint 21.

**Encryption at rest keypair** : Sprint 20 big-rock (`§3 S20`).
Keychain/DPAPI wrapping. Gate 2 prerequis.

**TLS cert pinning relays** : Sprint 19 (`§3 S19`), depends S18
multi-relai.

**Self-hosted pkarr relay** : Sprint 19 (`§3 S19`).

**Federated ONG-run relays concrets** : Sprint 19+ outreach (non
solo-implementable, necessite partnership Amnesty/HRW).

**ML-DSA-65 + ML-KEM-1024 hybrid (PQC migration)** : Sprint 26+
(`VALIDATED_BLUEPRINT couche 1`). Ed25519 acceptable jusque-la.

**`THREAT_MODEL.md` cross-ref Sprint 17 docs** : P2 tech debt
S17, reprendre dans sprint qui touche docs/security/. Pas S18.

**Structured output llama.cpp JSON grammar** : Sprint 20 (`§3 S20`).

---

## 7. Tracabilite scope

Items **nouveaux Sprint 18** :
- `cargo-deny` config + CI (nouveau)
- `pip-audit` CI (nouveau)
- `npm audit` via `audit-ci` (nouveau)
- wasmtime pin CVE avril 2026 (nouveau, post-S17 BLUEPRINT)
- Reproducible builds Rust + SLSA in-toto (nouveau)
- Multi-relai federation config loader (nouveau)
- DHT pkarr redundant lookup (nouveau)
- NVIDIA driver CVE check launcher (nouveau)
- Warrant canary (nouveau format SBFB Ed25519)
- Radicle mirror (nouveau, infra ops)

Items **carry/dette** :
- Coord-side wire TaskEntry (dette S16 C-1/C-2 partial fix)
- X-SBFB-Token rotation (carry quick-win roadmap §4)

Items **differes** :
- Multi-relai self-hosted → S19
- PoW gossip → S19
- TLS pinning → S19
- Encryption at rest → S20
- Structured output grammar → S20

---

## 8. Audit gate pattern — rappel

Phase 0 Sprint 17 audit joue pre-S18 session 2026-04-14, verdict
PASS apres 1 commit `4f0727b`. Phase F S18 produit
`sprint18_audit_plan.md` pour Sprint 19 Phase 0. Pattern permanent
depuis Sprint 7.

---

## 9. Estimations LOC

| Phase | LOC code | LOC tests | LOC docs | Total |
|---|---|---|---|---|
| 0 — Audit S17 | 0 | 0 | 0 (migre existant) | 0 |
| A — Supply chain CI | ~250 | ~30 (integration tests ops) | ~20 | ~300 |
| B — Reproducible builds + SLSA | ~150 | ~20 | ~80 | ~250 |
| C — Multi-relai + DHT redundant | ~450 | ~130 | ~20 | ~600 |
| D — Wire TaskEntry + token rotation | ~280 | ~100 | ~20 | ~400 |
| E1 — Driver check NVD | ~200 | ~40 | ~10 | ~250 |
| E2 — Warrant canary | ~100 | ~30 | ~20 | ~150 |
| E3 — Radicle mirror | ~30 (YAML) | 0 | ~20 | ~50 |
| F — Consolidation + verif + audit plan | 0 | 0 | ~250 | ~250 |
| **Total** | **~1460** | **~350** | **~440** | **~2250** |

**Delta tests** : +50 a +60 (Rust ~20, coord ~15, web/playwright ~10,
ops CI ~10). Compteur final estime : **~1183 tests** (1128 + 55).

LOC total (~2250) > roadmap estimate (~1500) : l'ecart vient de
(a) tests volume ajoutes (roadmap ne compte pas tests), (b) docs
non-comptees par roadmap, (c) item 9 wasmtime pin post-S17 non
liste par roadmap, (d) subphase decoupage E1/E2/E3 fait apparaitre
docs ops.

---

## 10. Checkpoint de validation

Status : **draft**, a discuter avant Phase A si besoin. Hypothese
retenue : l'utilisateur a ecrit "fait au mieux" → je trance sur
D1..D5 sur base recherche context7 + WebSearch documentee. Si
ecart constate Phase A kickoff session fraiche, re-ouvrir
discussion Day-0.

Points de validation souhaitables (non-bloquants si le user
confirme l'approche "autonome") :

1. **D2 wasmtime pin `>=43.0.1, <44`** : tu preferes pin exact
   `=43.0.1` ou range permissif patches ?
2. **D3 cargo-deny seul** : OK de skipper `cargo-audit` standalone
   (cargo-deny l'englobe) ou prefere redondance ?
3. **D4 relais federaux placeholders seulement S18** : OK de ne
   PAS ajouter relais externes publics ce sprint (attendre S19
   partnership) ou pousser 1-2 relais community public maintenant ?
4. **D5 warrant canary format Ed25519 + mirror GitHub+Radicle** :
   format retenu ou preferes PGP classique ?
5. **Phase E decoupage 3 subphases (E1/E2/E3) dans 1 phase**
   mono-commit vs 3 commits distincts : preference ?
6. **Fichiers untracked root** : `cc.json`, `test_libc.exe`,
   `test_libc.pdb`, `node_modules/`, `site/`, `docs/DND_P2P_DESIGN.md`,
   `docs/VISION_USE_CASES.md`, `docs/apps/` — a gitignorer
   (artefacts), commiter separement (docs apps), ou laisser
   traine ? **Decision par defaut** : **laisser traine S18** (hors
   scope security, non-bloquant). Sprint futur dedie docs apps
   les integrera.

---

**Note de placement** : ce kickoff est ecrit directement dans
`.planning/active/` (Sprint 17 deja migre archive/v1.2/ via
wrap-up `60b539a`). Le seul mouvement S18-day-1 est `git mv
sprint17_audit_findings.md → archive/v1.2/` (staged mais pas
encore commite — c'est fait avec le 1er commit chore(planning) S18).
