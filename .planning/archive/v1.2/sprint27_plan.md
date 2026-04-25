# Sprint 27 — Plan detaille (Output watermark SynthID + Couche 3 mature multi-forge + trust-web ONG bootstrap + Gate 3 showcase hardening + P2 batch S26)

**Ecrit** : 2026-04-25.
**Kickoff** : `.planning/active/sprint27_kickoff.md`
**Design review** : `.planning/active/sprint27_design_review.md` (G1 MOSTLY PASS)

---

## 1. Decisions recap

| D | Decision | Scope |
|---|---|---|
| D1 | SynthID-inspired z-test watermark | PRF bias additif (pas Tournament Sampling complet), BIRA-resistant |
| D2 | Couche 3 git-log parser offline | GPG/SSH signatures, SQLite LRU, cross-forge aggregation |
| D3 | Trust-web ONG bootstrap seeds | Placeholder FlowUP S27, Amnesty/HRW/CPJ/EFF S28 |
| D4 | P2 batch 7 items | 3 carry audit + 4 observes |
| D5 | Gate 3 showcase Gate 3 docs | HARDENING_ROADMAP + COMPUTE_THREATS + Gate 3 checklist |

---

## 2. Verification pre-commit (rappel §7.4)

Avant CHAQUE commit phase, lancer :

```bash
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release

# Python
uv run ruff format --check packages/ && uv run ruff check packages/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

# Frontend
cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run test:coverage && npm run build && \
  npm run size && npx playwright test && bash scripts/scan-en-strings.sh
```

---

## 3. Phase A — P2 batch S26 audit (7 fixes)

### 3.1 Contexte

7 P2 identifies par l'audit S26. Pattern Phase A P2 batch etabli
depuis S25. Cleanup d'abord, features ensuite.

### 3.2 Items

#### A.1 — P2-A-1 : validate_stage_guard_map wire dans Dispatcher

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py`
**Action** : dans `Dispatcher.__init__`, ajouter
`validate_stage_guard_map(stage_guards)` avant l'assignation
`self._stage_guards`. Importer la fonction depuis `guardrails.py`.
**Test** : ajouter test integration dans `test_dispatcher.py` :
`test_dispatcher_rejects_invalid_stage_guard_key` — passer une
StageGuardrailMap avec une cle invalide, verifier `ValueError`.

#### A.2 — P2-C-1 : emit_capability_event logging

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/capability_store.py`
**Action** : remplacer `pass` dans le except de `_emit_capability_event`
par `logger.debug("emit_capability_event failed", exc_info=True)`.
Importer `logging` si pas deja fait.
**Test** : test existant suffisant (le catch ne change pas le
comportement, seulement la tracabilite).

#### A.3 — P2-D-1 : TaskHandlerDescriptor description

**Fichier** : `packages/nexus-sdk/src/nexus_sdk/app.py` (dataclass) +
`packages/nexus-sdk/src/nexus_sdk/decorators.py` (decorateur)
**Action** :
1. Ajouter `description: str = ""` au dataclass `TaskHandlerDescriptor`
2. Dans le decorateur `@task_handler`, capturer `fn.__doc__ or ""`
   et le passer au `TaskHandlerDescriptor`
3. Dans `packages/nexus-coordinator/src/nexus_coordinator/api/apps.py`
   endpoint manifest, inclure le champ `description` dans le JSON
**Test** : ajouter test dans `test_decorators.py` ou `test_app.py` :
`test_task_handler_captures_docstring` — decorer une fonction avec
docstring, verifier que `descriptor.description` est renseigne.

#### A.4 — P2-C-2 : JsonFileWriter rotation taille-based

**Fichier** : `crates/nexus-events-core/src/lib.rs`
**Action** : ajouter un mecanisme de rotation dans `JsonFileWriter` :
1. Champ `max_bytes: u64` (defaut 10 MiB) dans la struct
2. Avant chaque `writeln!`, verifier la taille du fichier
3. Si taille >= max_bytes, renommer `audit.jsonl` → `audit.jsonl.1`
   (et decaler `.1` → `.2`, etc., max 5 fichiers)
4. Ouvrir un nouveau `audit.jsonl`
**Test** : test unitaire `test_json_file_writer_rotation` : ecrire
des events jusqu'a depasser max_bytes, verifier que les fichiers
`.1` et `.2` existent.

#### A.5 — P2-C-3 : EtwWriter → TracingWriter rename

**Fichier** : `crates/nexus-events-core/src/lib.rs` + tous les
fichiers qui referencent `EtwWriter`
**Action** : rename `EtwWriter` → `TracingWriter`. Grep exhaustif
dans le workspace pour mettre a jour toutes les references.
**Test** : tests existants passent avec le nouveau nom.

#### A.6 — P2-B-1 : MCP lifespan comment inline

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/api/app.py`
**Action** : ajouter un commentaire inline sur les lignes 70-77
expliquant pourquoi `__aenter__`/`__aexit__` sont explicites (context
manager doit span le yield du lifespan FastAPI).
**Test** : aucun (commentaire only).

#### A.7 — P2-E-1 : convention no-LOC-estimates

**Action** : informatif — la convention est deja integree dans
`docs/claude/README.md §6.7` post-S26. Pas d'action code.

### 3.3 Commit

```
feat(sprint27): Sprint 27 Phase A — P2 batch S26 audit 7 fixes

Resout les 7 P2 documentes dans sprint26_audit_findings.md :

- P2-A-1 : wire validate_stage_guard_map() dans Dispatcher.__init__
  pour rejeter les cles invalides StageGuardrailMap (dispatcher.py)
- P2-C-1 : ajouter logger.debug dans except _emit_capability_event
  (capability_store.py) pour tracabilite diagnostic
- P2-D-1 : ajouter champ description au TaskHandlerDescriptor +
  capturer fn.__doc__ dans @task_handler (app.py, decorators.py) +
  exposer dans manifest endpoint (apps.py)
- P2-C-2 : JsonFileWriter rotation taille-based 10 MiB + 5 fichiers
  max (nexus-events-core/lib.rs)
- P2-C-3 : rename EtwWriter → TracingWriter (nexus-events-core/lib.rs)
- P2-B-1 : commenter pattern __aenter__/__aexit__ MCP lifespan
  (api/app.py)
- P2-E-1 : informatif (convention no-LOC integree §6.7)

Delta tests : +3 (1 dispatcher integration, 1 decorator docstring,
1 rotation file writer)

Scope cuts respectes : aucun scope cut viole.
```

---

## 4. Phase B — Output watermark SynthID-inspired

### 4.1 Contexte

Le watermark output est complementaire au canary-input S22 (watermark
INPUT prompt probe). Il ajoute une couche de detection compute-theft
(C-ComputeTheft, COMPUTE_THREATS §4) basee sur l'analyse statistique
des tokens generes par le worker.

La technique SynthID-inspired utilise une PRF (pseudo-random function)
pour classifier les tokens en "green" vs "red" basee sur le contexte
(hash des tokens precedents + secret partage). Le bias additif
augmente la probabilite des tokens green durant le sampling. La
detection mesure la proportion de tokens green dans l'output via un
z-test binomial.

Difference avec Kirchenbauer KGW (rejete BIRA) : le partitioning
KGW est deterministe (meme token precedent → meme green-list), ce
qui permet l'attaque BIRA (iterative rewriting pour minimiser les
tokens green). Notre approche utilise un context hash multi-token
(window glissante) + secret rotatif, rendant le partitioning non-
reproductible par l'attaquant.

### 4.2 Composantes

#### B.1 — WatermarkDetector coordinator-side (Python)

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/watermark_detector.py`

```python
class WatermarkDetector:
    def __init__(self, secret: bytes, window_size: int = 4,
                 threshold_z: float = 2.0):
        self._secret = secret
        self._window = window_size
        self._threshold = threshold_z

    def is_watermarked(self, token_ids: list[int]) -> WatermarkResult:
        """Z-test binomial sur la proportion de tokens green."""
        ...

    def _prf_score(self, token_id: int, context: tuple[int, ...]) -> float:
        """HMAC-SHA256(secret, context || token_id) mod 1.0"""
        ...
```

Integre dans le result path du dispatcher : apres reception du
resultat worker, le detector verifie si l'output est watermarked.
Resultat = `WatermarkResult(is_watermarked: bool, z_score: float,
green_ratio: float)`. Si watermarked : log info. Si non-watermarked :
log warning (pas de rejection — worker peut etre non opt-in).

#### B.2 — WatermarkInjector worker-side (Rust, LlamaCppBackend)

**Fichier** : `crates/nexus-worker-core/src/llm/llama_cpp.rs`

Extension du `LlamaCppBackend::generate()` : avant le sampling step,
ajouter un delta logit `+watermark_delta` (defaut 2.0) aux tokens
dont le PRF score > 0.5. Le PRF utilise le meme HMAC-SHA256(secret,
context_window || token_id).

Config opt-in via `watermark.toml` :
```toml
[watermark]
enabled = true
delta_logit = 2.0
window_size = 4
```

Le secret est recu dans le Task dispatch payload (champ
`watermark_seed` redefinissant le canonical v1 — pre-launch protocol
applicable).

#### B.3 — Tests

- `test_watermark_detector_watermarked_output` : generer des token
  IDs avec bias green, verifier z_score > threshold
- `test_watermark_detector_non_watermarked_output` : token IDs
  aleatoires, verifier z_score < threshold
- `test_watermark_detector_edge_cases` : output vide, output court
  (< window_size), threshold boundary
- `test_watermark_prf_determinism` : meme secret + context → meme
  score (reproductibilite)
- `test_watermark_injector_config` : parse watermark.toml sample
- `test_watermark_injector_disabled_by_default` : injection OFF
  sans config
- Integration Rust : `test_watermark_logit_bias_applied` : verifier
  que le delta est ajoute aux logits (mock sampling pipeline)

### 4.3 Risk R-S27-4 (llguidance conflit)

Test integration specifique : activer watermark + grammar llguidance
simultanement. Si conflit (grammar constraint annule le bias) :
fallback `watermark.enabled = false` quand grammar active.

### 4.4 Commit

```
feat(sprint27): Sprint 27 Phase B — output watermark SynthID-inspired z-test detection + llama.cpp injection opt-in

Architecture 2 composantes watermark output model-side :

- WatermarkDetector coordinator-side (Python) : z-test binomial sur
  proportion tokens green via PRF HMAC-SHA256(secret, context ||
  token_id). Integre dans result path dispatcher. Non-bloquant
  (log warning si non-watermarked, pas de rejection).
- WatermarkInjector worker-side (Rust LlamaCppBackend) : delta logit
  +2.0 sur tokens green. Opt-in via watermark.toml.
  Ollama backend defere (API sans logit hook).

Technique SynthID-inspired (Nature 2024 Google DeepMind). BIRA-
resistant vs Kirchenbauer KGW rejete (arXiv:2509.23019 sept 2025).
Complementaire canary-input S22 (watermark INPUT prompt probe).

Delta tests : +7 (4 Python detector, 3 Rust injector)

Scope cuts respectes :
- Ollama backend watermark injection → S28+
- Full SynthID Tournament Sampling → S28+
```

---

## 5. Phase C — Couche 3 multi-forge cross-validate + trust-web bootstrap

### 5.1 Contexte

Couche 3 Sybil-resistance : la verification multi-forge cross-validate
est le mecanisme le plus robuste pour prouver qu'un contributeur est
reel (signatures sur 2+ forges = juridictions differentes = Sybil
couteux). Decrite dans `docs/security/CONTRIBUTOR_ATTESTATION_RFC.md`.

Existant :
- Couche 1 `AgeWitness` (S22, live) : 7j minimum presence
- Couche 2 `ContributorAttestation` (S22, live) : in-toto v1.0
- Couche 3 `DelegationCert` (S23, design-only) : format primitif dans
  `crates/nexus-core-rs/src/attestations/delegation.rs`

Gap S27 : parser multi-forge + cache + trust-web bootstrap.

### 5.2 Pre-Phase C : DelegationCert format spec (G1 ⚠️ D3)

Avant d'etendre DelegationCert, documenter formellement le format
signature avec mapping C2PA Claim structures. Ajouter dans
`docs/security/CONTRIBUTOR_ATTESTATION_RFC.md` une section
"§DelegationCert v1 format specification" avec :
- Champs : issuer_fingerprint, delegatee_fingerprint, trust_level
  (1-5), valid_from, valid_until, scope (org_name, forge_urls[]),
  signature Ed25519
- Mapping C2PA : DelegationCert ↔ C2PA Assertion (claim_generator,
  signer_payload, trust_list)
- Serialisation JCS canonical (pattern DOMAIN_DELEGATION_CERT_V1
  existant)

### 5.3 Composantes

#### C.1 — ForgeParser Rust (git-log --show-signature)

**Fichier** : `crates/nexus-core-rs/src/attestations/forge_parser.rs`

```rust
pub struct ForgeContribution {
    pub fingerprint: String,     // GPG fingerprint ou SSH key hash
    pub commit_count: u32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub forge_url: String,       // origin URL du repo
    pub sig_type: SigType,       // GPG | SSH
}

pub fn parse_git_log(repo_path: &Path) -> Result<Vec<ForgeContribution>> {
    // Execute `git log --show-signature --format=%H|%aI|%GK|%G?|%GS`
    // Parse chaque ligne : SHA|date|key_id|status|signer
    // Filtre status == 'G' (Good signature)
    // Aggrege par fingerprint
}
```

Le parser execute un `Command::new("git")` local. Pas de dep
git2-rs (overhead pour un parser one-shot). Cross-platform
(git est prerequis SBFB).

#### C.2 — TrustCache SQLite LRU

**Fichier** : `crates/nexus-core-rs/src/attestations/trust_cache.rs`

```rust
pub struct TrustCache {
    db: Connection,  // rusqlite
}

impl TrustCache {
    pub fn get_or_parse(&self, repo_url: &str, repo_path: &Path,
                        ttl: Duration) -> Result<Vec<ForgeContribution>>;
    pub fn invalidate(&self, repo_url: &str) -> Result<()>;
}
```

Schema SQLite :
```sql
CREATE TABLE forge_contributions (
    repo_url TEXT,
    fingerprint TEXT,
    commit_count INTEGER,
    first_seen TEXT,
    last_seen TEXT,
    sig_type TEXT,
    cached_at TEXT,
    PRIMARY KEY (repo_url, fingerprint)
);
```

TTL 7 jours. Pattern `quarantine_queue.py` S21 Phase D pour le
schema setup + WAL mode.

#### C.3 — TrustWeb aggregator + gossip

**Fichier** : `crates/nexus-shell-daemon-core/src/trust_web.rs`

```rust
pub struct TrustWebManager {
    cache: TrustCache,
    seeds: Vec<TrustSeed>,      // from trust_web_seeds.toml
    delegation_certs: Vec<DelegationCert>,
}

impl TrustWebManager {
    pub fn compute_trust_score(&self, fingerprint: &str) -> TrustScore;
    pub fn verify_cross_forge(&self, fingerprint: &str) -> CrossForgeResult;
}
```

TrustScore : combinaison forge_count (nb forges distinctes) ×
commit_tenure (anciennete) × delegation_depth (distance au seed).
Decay : -1 trust_level par hop depuis l'anchor, minimum 1.

Gossip topic `nexus-grid/trust-web/v1` : les nodes publient les
DelegationCert signes. Les subscribers verifient la signature
Ed25519 + la chaine de delegation.

#### C.4 — Trust-web seed config

**Fichier** : `configs/trust_web_seeds.toml`

```toml
[[seeds]]
org_name = "FlowUP (bootstrap)"
fingerprint = "80b439cb..."  # Ed25519 pubkey FlowUP
forge_urls = ["https://github.com/SBFB50/SBFB"]
description = "Bootstrap anchor — sera remplace par ONG S28"

# Placeholders S28 outreach
# [[seeds]]
# org_name = "Amnesty International"
# fingerprint = "..."
```

#### C.5 — Update DelegationCert

**Fichier** : `crates/nexus-core-rs/src/attestations/delegation.rs`

Ajouter champs a `DelegationCert` :
- `trust_level: u8` (1-5, defaut 3)
- `valid_until: Option<DateTime<Utc>>` (expiry, defaut None = perpetuel)
- `scope: DelegationScope { org_name, forge_urls }`

Le canonical JCS reste `DOMAIN_DELEGATION_CERT_V1` (pas de bump
version — pre-launch protocol).

### 5.4 Tests

- `test_forge_parser_gpg_signed_commits` : fixture git repo avec
  commits GPG-signes, verifier extraction fingerprint + count
- `test_forge_parser_ssh_signed_commits` : meme chose SSH
- `test_forge_parser_unsigned_commits_ignored` : commits non-signes
  filtres
- `test_trust_cache_ttl_expiry` : cache expire apres TTL, re-parse
- `test_trust_cache_invalidate` : invalidation manuelle fonctionne
- `test_trust_web_cross_forge_score` : meme fingerprint sur 2 repos
  differents → score plus eleve
- `test_trust_web_delegation_decay` : trust_level -1 par hop,
  minimum 1
- `test_delegation_cert_v1_with_trust_level` : serialisation/
  deserialisation avec nouveaux champs
- `test_delegation_cert_canonical_jcs` : canonical bytes JCS
  deterministes

### 5.5 Commit

```
feat(sprint27): Sprint 27 Phase C — Couche 3 multi-forge cross-validate + trust-web ONG bootstrap

Couche 3 Sybil-resistance mature (CONTRIBUTOR_ATTESTATION_RFC.md) :

- ForgeParser Rust : git log --show-signature parser offline
  (GPG RFC 4880 + SSH RFC 8709). Extrait fingerprint, commit count,
  tenure, forge URL. Cross-platform via git CLI.
- TrustCache SQLite LRU : cache contributions par repo, TTL 7j,
  WAL mode (pattern quarantine_queue S21).
- TrustWebManager : aggregation cross-forge (score = forge_count x
  tenure x delegation_depth, decay -1/hop). Gossip topic
  nexus-grid/trust-web/v1 pour publication DelegationCert.
- DelegationCert v1 etendu : trust_level 1-5 + valid_until +
  DelegationScope (org_name, forge_urls). Canonical JCS inchange
  (DOMAIN_DELEGATION_CERT_V1, pre-launch redefinition).
- Trust-web seed config : placeholder FlowUP bootstrap (ONG → S28).
- DelegationCert format spec ajoutee dans
  CONTRIBUTOR_ATTESTATION_RFC.md avec mapping C2PA (G1 ⚠️ D3 ack).

Delta tests : +9 (3 parser, 2 cache, 2 trust-web, 2 delegation)

Scope cuts respectes :
- Radicle native verification → LT-2 (tag v1.0)
- Live forge API polling → rejected D2
- ONG reelles → S28 outreach
```

---

## 6. Phase D — Gate 3 showcase hardening docs + Gate 3 prerequisites

### 6.1 Contexte

Phase docs-only. Met a jour les artefacts long-life pour refleter
les livraisons S22-S27 (watermark SynthID, Couche 3 mature) et
documente les prerequisites Gate 3 restants.

### 6.2 Items

#### D.1 — HARDENING_ROADMAP update

**Fichier** : `docs/security/HARDENING_ROADMAP.md`
- Ligne S27 : mettre a jour le goal et les items livres (SynthID
  remplace Kirchenbauer, Couche 3 parser + trust-web)
- Ligne S22 : note BIRA dans la description watermark canari
- `last_validated` : update a `2026-04-25 S27`
- `audited_findings` : ajouter une entree S27 avec les decisions
  watermark + Couche 3

#### D.2 — COMPUTE_THREATS update

**Fichier** : `docs/security/COMPUTE_THREATS.md`
- §4.4 : remplacer "Watermarking outputs (Kirchenbauer 2023)" par
  "Watermarking outputs (SynthID-inspired PRF z-test, BIRA-resistant)"
- §4.4 : ajouter note "Kirchenbauer KGW rejete — vulnerable BIRA
  attack arXiv:2509.23019 sept 2025"
- §Sprint 27 line : update avec livraisons reelles

#### D.3 — Gate 3 prerequisites checklist + showcase reframing

**Fichier** : `docs/security/HARDENING_ROADMAP.md` §7 Gate 3
- Reframing Gate 3 : "PolitiScan, NEXUS cold-case" → "Alexandria,
  showcase apps" (cf. `docs/apps/LAUNCH_SHOWCASE.md`). Alexandria =
  bibliotheque de connaissance multilingue, premiere app showcase
  (stockage distribue + MCP tools, pas de GPU requis).
- Checklist : items livres S22-S27 (canary-input, redundancy, Couche
  1+2+3, watermark output, PoW, rate-limit, etc.)
- Items restants : audit externe Cure53/ToB S29, Tor transport S28+
- Timeline : Gate 3 effectif fin S29

#### D.4 — PATTERNS.md update (si applicable)

**Fichier** : `docs/rust/PATTERNS.md`
- Si Phase B/C introduisent des patterns notables (PRF watermark,
  ForgeParser), les documenter dans une section P37+.
- Sinon : aucune action.

#### D.5 — Self-distribution design doc

**Fichier** : `docs/release/SELF_DISTRIBUTION.md`

Design doc pour le concept "le protocole est son propre premier
contenu". SBFB binaries = blobs distribuables par le meme reseau
qu'ils creent. Spec consumee par le sprint d'implem (~S30 release
prep pre-v1.0).

Sections :
1. **Principe** : tout est un blob (apps, donnees, protocole).
   Pas d'exception, pas de CDN, pas de store.
2. **Format bundle** : zip signe Ed25519 contenant les binaires
   cross-platform (nexus-launcher, nexus-shell-daemon,
   nexus-coordinator wheel, nexus-worker) + config bootstrap
   minimale (relay URLs, trust seeds) + provenance.json SLSA L1
   (reutilise le pipeline verified deploy S14).
3. **Canaux de distribution** : iroh-blobs P2P (si B a deja un
   noeud qui relay), Bluetooth (~30-50 MB, ~2 min), WiFi Direct
   (~3 sec), carte SD/USB, HTTP fallback (download classique).
4. **Bootstrap problem** : le tout premier noeud d'un reseau
   isole vient forcement d'un canal externe (Bluetooth/USB/
   download). Ensuite le reseau est self-sustaining.
5. **Lien verified deploy S14** : meme signature Ed25519, meme
   provenance.json, meme verification. La seule difference : le
   payload est les binaires SBFB au lieu d'une app tierce.
6. **Endpoint daemon** : `GET /export-bundle` sur le shell daemon
   loopback. Package les binaires du noeud en cours + config.
7. **Update P2P** : un noeud existant peut recevoir une nouvelle
   version du bundle via gossip topic
   `nexus-grid/protocol-update/v1`. Verification signature
   mainteneur + hash BLAKE3 avant remplacement.
8. **Implem target** : ~S30 (release prep pre-v1.0). ~300 LOC
   Rust + CI cross-compile matrix (GitHub Actions).

### 6.3 Commit

```
docs(sprint27): Sprint 27 Phase D — Gate 3 showcase hardening docs + Gate 3 prerequisites update

- HARDENING_ROADMAP : update S27 (SynthID remplace Kirchenbauer,
  Couche 3 mature, last_validated 2026-04-25)
- COMPUTE_THREATS §4.4 : SynthID-inspired PRF z-test remplace KGW,
  note BIRA rejection
- Gate 3 prerequisites checklist : items S22-S27 livres, items
  restants (audit externe S29, Tor S28+)
- PATTERNS.md : P37+ si applicable
```

---

## 7. Phase E — Wrap-up

### 7.1 Livrables

1. `sprint27_verification.md` — fail-fast checklist 25+ rows
2. `sprint28_audit_plan.md` — tracks A-D + meta-track G8 + carry
3. Update `docs/claude/SPRINT_LOG.md` — row S27
4. Update `CLAUDE.md` — compteurs tests, etat actuel
5. Update memory `nexus_grid_pivot.md` — tip + compteurs
6. Update memory `MEMORY.md` — description SBFB pivot
7. Migration `.planning/active/sprint27_*.md` →
   `.planning/archive/v1.2/` (si v1.2 continue)

### 7.2 Commit

```
chore(sprint27): Phase E — wrap-up + verification + audit plan S28 + migration

Sprint 27 clos. 4 phases A-D livrees (P2 batch + watermark SynthID +
Couche 3 multi-forge + Gate 3 showcase docs).

verification.md : XX/XX fail-fast rows verts.
audit_plan S28 : X tracks + meta-track G8 traceability.
```

---

## 8. Dependencies inter-phases

```
Phase A (P2 batch) ─── independant
Phase B (watermark) ── independant (mais utilise Task wire pre-launch)
Phase C (Couche 3) ─── dep D3 pre-Phase C (DelegationCert spec G1 ⚠️)
Phase D (docs) ──────── dep B+C (documente les livraisons)
Phase E (wrap-up) ───── dep A+B+C+D
```

Phase A et B sont independantes. Phase C requiert la spec
DelegationCert (sous-tache pre-Phase C). Phase D documente les
phases precedentes. Phase E ferme le sprint.

---

## 9. Fail-fast checklist preview

| # | Check | Phase | Status |
|---|---|---|---|
| 1 | validate_stage_guard_map wiree Dispatcher | A | [ ] |
| 2 | emit_capability_event logger.debug | A | [ ] |
| 3 | TaskHandlerDescriptor.description renseigne | A | [ ] |
| 4 | JsonFileWriter rotation fonctionne | A | [ ] |
| 5 | TracingWriter rename complet | A | [ ] |
| 6 | MCP lifespan commente | A | [ ] |
| 7 | WatermarkDetector z-test watermarked → True | B | [ ] |
| 8 | WatermarkDetector non-watermarked → False | B | [ ] |
| 9 | WatermarkDetector edge cases (vide, court) | B | [ ] |
| 10 | PRF determinism (meme input → meme score) | B | [ ] |
| 11 | WatermarkInjector config parse | B | [ ] |
| 12 | WatermarkInjector disabled by default | B | [ ] |
| 13 | Logit bias applique dans sampling | B | [ ] |
| 14 | ForgeParser GPG commits | C | [ ] |
| 15 | ForgeParser SSH commits | C | [ ] |
| 16 | ForgeParser unsigned filtres | C | [ ] |
| 17 | TrustCache TTL expiry | C | [ ] |
| 18 | TrustCache invalidate | C | [ ] |
| 19 | TrustWeb cross-forge score | C | [ ] |
| 20 | TrustWeb delegation decay | C | [ ] |
| 21 | DelegationCert v1 trust_level | C | [ ] |
| 22 | DelegationCert canonical JCS | C | [ ] |
| 23 | HARDENING_ROADMAP S27 updated | D | [ ] |
| 24 | COMPUTE_THREATS §4.4 SynthID | D | [ ] |
| 25 | Gate 3 checklist documentes | D | [ ] |
| 26 | SELF_DISTRIBUTION.md design doc livre | D | [ ] |
| 27 | Rust fmt + clippy + nextest + doc | E | [ ] |
| 27 | Python ruff + pytest SDK/coord/gov | E | [ ] |
| 28 | Frontend lint + tsc + vitest + build + size + PW | E | [ ] |
