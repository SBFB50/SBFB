# Sprint 19 — Audit findings (Sprint 20 Phase 0)

**Auditeur** : session Claude Code fraiche, ouverte 2026-04-16 post-Sprint 19
Phase F wrap-up + resolve placeholders (`bf7dd62`).
**Timebox observe** : ~2h (lectures kickoff/plan/verification/audit_plan +
5 agents Explore parallèles sur tracks A/B/C/D/E + grep ciblé
cross-docs + compteurs tests réels + rédaction).
**Tip audite** : `bf7dd62` (chore(sprint19) resolve Phase F placeholders
to real SHA `619059b`).
**Range commits audite** : `1a606a3..bf7dd62` (13 commits : 5 feat S19
phases A/B/C/D/E + 1 fix Phase B follow-up + 1 chore planning open S19
+ 2 chore planning inter-phase (guardrails G1..G7 + Phase D wrap) + 2
chore tooling G4 hors-sprint + 2 chore wrap-up Phase F + placeholders).

---

## Verdict global : **PASS**

- **0 finding P0** (aucun blocage sécurité, aucune régression critique,
  aucun wire bypass caché).
- **0 finding P1** (aucune promesse fausse assez critique pour bloquer
  S20 Phase A ; les deux candidats P1 ont été reclassés P2 après examen
  du commit body Phase A qui acte lui-même la nuance « sous config »).
- **9 findings P2** (gap promesse vs réalité documentaire sur Eclipse-
  by-DHT, cap carry-over G7 dépassé implicite, SPKI fail-open divergence
  plan, bench PoW chiffre non-archivé, pins bootstrap placeholders,
  Dockerfile pas @sha256, plaintext payload caveat peu visible, tests
  intégration Track A cas heureux manquant, ruff format non-clean sur 2
  fichiers Phase D vs verification claim).
- **2 findings P3** (D2 rationale cite Tor 2023 sans mentionner abandon
  Equi-X documenté post-hoc dans design doc §3.6 ; sprint19_phase_F_
  review.md résiduel dans active/ à migrer).

**Sprint 20 Phase A non-bloqué**. Les P2 peuvent être traités en batch
dans le commit `chore(sprint19): audit-P2 batch — ...` avant le premier
`feat(sprint20): Phase A`, ou intégrés au `chore(planning): open S20`
selon convenance. Aucun P0/P1 n'exige de `fix(sprint19):` atomique
avant S20 Phase A.

**Rigor signal G4 satisfait** : 8 P2 + 2 P3 documentés, pas de
« 0 finding » systématique. La dimension Horizon long-terme +
Research-grounding a été challengée spécifiquement sur D2 (SHA256 vs
Equi-X 2023) et a produit un finding P3 (doc trail découverte
post-hoc, pas violation).

Sprint 19 livre une **vraie progression transport hardening** (5
phases, +82 tests delta, 6 design docs `.planning/research/S19_*`, 4
sections nouvelles PATTERNS.md dont §P29 delayed upload + §TLS
pinning + §DHT canary + §PoW Hashcash). Le pattern dominant
**primitive livrée + wire opt-in / différé S20+** est reconduit trois
fois (Track A canary opt-in env var, Track B subscribe wire différé
S20 Phase 1, Track C wire iroh différé S20+ T20) et mérite d'être
nommé explicitement dans le kickoff S20 pour éviter l'accumulation
silencieuse de carry-overs (voir finding **P2-A2 cap G7**).

---

## Mode d'emploi suivi

L'ordre de lecture impose dans `sprint19_audit_plan.md §Mode d'emploi`
a été respecté :

1. `git log --oneline 1a606a3..HEAD` (range S19 complet)
2. `sprint19_kickoff.md` + `sprint19_plan.md` + `sprint19_verification.md`
3. `sprint19_audit_plan.md` (les 6 tracks A-F + meta-track Radicle)
4. **Code + tests + docs** par track **avant** lecture des phase
   reviews livreur (formation d'opinion indépendante). Délégation à
   5 agents Explore parallèles (A/B/C/D/E), chacun interdit de lire
   `sprint19_phase_{X}_review.md` pour éviter le biais confirmation.
5. Grep cross-docs pour vérifier la cohérence des claims
   (`pleinement active`, `DHT redundant lookup`, `[~]→[x]`) entre
   `CLAUDE.md`, `SPRINT_LOG.md`, `sprint18_verification.md`,
   `sprint19_verification.md`.
6. **Phase reviews livreur** lus en cross-check **après** formation
   d'opinion, uniquement pour valider que les agents Explore n'ont pas
   manqué un finding flaggé par le reviewer intra-sprint.
7. Compteurs tests réels via `cargo test --workspace --locked` (537
   confirmé) + `pytest nexus-sdk/` (185 confirmé) + `pytest nexus-app-
   gov/` (46 confirmé). Coord run obtient 192 passed + 16 failed sur
   `test_provenance.py` dans l'env local de l'auditeur — **non-finding
   S19** : failures dues à wheel `nexus-core-py` stale dans le venv
   (AttributeError: module 'nexus_core' has no attribute 'sign_bytes',
   fonction pourtant présente `crates/nexus-core-py/src/lib.rs:1097`).
   Le wheel doit être rebuild via `maturin develop --release` (pattern
   documenté CLAUDE.md §Commandes clés). CI GHA rebuild avant pytest.
   Verification claim 208 passed reste plausible.

Out of scope respecté : D1..D5 gelées du kickoff S19 non rebattues
(aucun finding sur PoW 2^18 vs 2^16/2^20, SPKI vs full-cert vs CA,
expo vs uniform, docker-only vs deploy-real). Les **rationales** des
D1..D5 sont en revanche challenge-ables — cf. P3-B2 sur D2 Tor 2023.

---

## Track A — DHT quorum runtime wire (carry S18 C-1) : **PASS avec 2 P2**

**Question centrale** : la primitive `dht_quorum::redundant_resolve`
S18 est-elle réellement câblée au runtime browse + curator, le fallback
2/3 exercé par tests intégration, le degraded mode observable ?

### Verifications effectuées

- Grep `redundant_resolve` + `PkarrQuorumResolver` dans tout `crates/` :
  **aucun shortcut single-node** détecté dans browse/curator. Le seul
  call site prod est `crates/nexus-shell-daemon-core/src/browse.rs`
  `probe_and_cache()` via `with_quorum_resolvers(...)` builder.
  Aucun `pkarr::Client::resolve` direct résiduel. ✅
- `crates/nexus-shell-daemon/src/runtime.rs:231-258` : le wire lit
  `SBFB_PKARR_RELAYS` env var → instancie N `PkarrQuorumResolver` via
  `load_quorum_resolvers_from_env()`. Logs INFO/WARN observés :
  - Ligne 248 : `pkarr quorum canary armed with a single relay —
    inter-relay cross-checking requires 2+ distinct URLs`
  - Ligne 253 : `pkarr quorum canary armed — Eclipse-by-DHT defence
    active`
  - Ligne 258 : `pkarr quorum canary disabled (SBFB_PKARR_RELAYS not
    set) — browse probes use the default iroh N0 discovery path` ✅
- Degraded mode : WARN logs à 4 niveaux (primitive
  `dht_quorum.rs:239-245` disagreement, browse aggregator
  `browse.rs:388-402` NoMajority + AllFailed, runtime boot
  `runtime.rs:246-258`). Observable ops-side. ✅
- TODO comment `browse.rs:256-267` ligne (plan §4.5 demandait
  suppression) : remplacé par doc-comment Sprint 19 Phase A
  `with_quorum_resolvers` builder + canary gate documentation
  `browse.rs:234-313`. ✅
- Tests intégration : `browse.rs:1131-1203` contient **2 tests**
  quorum (NoMajority skips dial, AllFailed skips dial). Tous deux
  **cas d'erreur**. Aucun test cas heureux (2/3 ou 3/3 agreement
  → probe_and_cache continue le dial et retourne Reachable). Gap de
  couverture → **finding P2-A3** ci-dessous.
- Flip S18 : `sprint18_verification.md:72` **est** `[x]` avec
  annotation `wire Phase A S19 ab6985c`. ✅

### Findings

**P2-A1** — **Claim-drift `pleinement active en runtime`** :

Le wording « Eclipse-by-DHT defense désormais pleinement active en
runtime » apparaît **5 fois** dans des docs canoniques consommées par
des sessions fraîches et futurs contributeurs :

- `CLAUDE.md:163`
- `docs/claude/SPRINT_LOG.md:22` (row S19) + `:179` (annotation)
- `.planning/archive/v1.2/sprint18_verification.md:72` (flip
  `[~]→[x]` §Gate 1)
- `.planning/archive/v1.2/sprint19_verification.md:74` + `:184`
  (findings carry-over)

Or le code `runtime.rs:258` confirme explicitement que le canary est
**disabled par défaut** (si `SBFB_PKARR_RELAYS` env var non set,
« browse probes use the default iroh N0 discovery path » = pas de
quorum). Le commit Phase A `ab6985c` body dit lui-même « runtime-
active **sous config** » — donc le livreur est honnête, mais le
downstream docs simplifie à « pleinement active ».

Risk : quelqu'un qui lit `CLAUDE.md §État actuel` pour planifier S20+
croit la défense Eclipse-by-DHT complète et ne priorise pas le
passage de canary opt-in à enforcement strict. Même pattern que S18
C-1 « primitive vs wire » (dont le fix a produit ce Phase A).

Fix recommandé inline (chore ouverture S20) : remplacer les 5
occurrences par `Eclipse-by-DHT defense runtime-active sous config
SBFB_PKARR_RELAYS (canary opt-in, enforcement strict visé sprint
Gate 2)`. Ou équivalent explicite.

**P2-A2** — **Cap carry-overs G7 dépassé implicitement** :

Le verification Phase F commit body déclare cap G7 respecté
(2/2 : Meta-1 Radicle + P2-2 gitignore NOISE). Mais l'examen des
commits et du code révèle **3 carry-overs additionnels implicites**,
tous non listés dans §6 "Items carry/dette" du kickoff S19 ni
explicités dans le verification :

1. **PoW runtime wire gossip subscribe** → S20 Phase 1 (cité
   explicitement dans `edfc51b` commit body : « integration
   intentionnellement différée Sprint 20+ pour éviter breakage flows
   gossip existants »). Primitive livrée, runtime wire S20+.
2. **TLS pinning wire iroh handshake** → S20+ T20 tech debt (design
   doc §5.1 Option B : iroh 0.97 n'expose pas `custom_cert_verifier`
   publiquement, wire = S20+). Primitive livrée, runtime wire S20+.
3. **DHT canary → enforcement strict** (implicite : passer de
   opt-in env var à default-on sur config federation n0). Pas dans
   audit plan S19.

Total carry-overs S19 → S20 réels : **5** (Meta-1 + P2-2 gitignore +
PoW wire + TLS wire + DHT strict) vs cap 2 (cf. `docs/claude/README.md
§6.2.1`). Le kickoff S20 doit trancher : soit intégrer ces items au
scope principal S20, soit les déclasser formellement en
`docs/DEPRECATED.md` avec rationale « défense-en-profondeur reportée
post-Gate 2 », ou les reclassifier comme tech debt long-terme
PATTERNS.md §T* plutôt que carry-over sprint.

Pattern observé : le livreur minimise les carries (2 documentés) mais
le pattern « primitive S{N} + wire différé S{N+1} » est en train de
devenir systématique (S18→S19 C-1, S19→S20 au moins 3 fois). Le cap
G7 vise précisément à rendre ce glissement visible.

**P2-A3** — **Tests intégration Track A : cas heureux absent** :

`crates/nexus-shell-daemon-core/src/browse.rs:1131-1203` contient 2
tests `probe_and_cache_skips_dial_when_quorum_has_no_majority` et
`...when_all_quorum_resolvers_fail`. Les deux sont des cas d'erreur
où le probe est **skippé**. Aucun test ne vérifie que **quand le
quorum accepte (2/3 ou 3/3 agreement), probe_and_cache continue le
dial et retourne status=Reachable**.

Impact : un futur refactor qui casserait la branche « quorum OK →
probe continues » passerait silencieusement les 2 tests existants
(puisqu'ils ne couvrent que le skip). Coverage partielle.

Fix : ajouter `probe_and_cache_with_quorum_majority_continues_dial`
(mock 3 resolvers qui renvoient le même payload → assert dial
effectif + status Reachable). ~30 LOC test.

### Pas de finding

- Signature `PkarrRelayClient::new(url, tls_config)` 2-arg cohérente
  iroh 0.97 (P3 #2 du phase A review déjà noté). ✅
- Flip S18 `[~]→[x]` exécuté correctement. ✅
- Degraded mode observable (WARN 4 niveaux). ✅

---

## Track B — PoW Hashcash gossip subscribe : **PASS avec 2 P2 + 1 P3**

**Question centrale** : primitive crypto-sound (pas de cross-topic
replay, pas de shortcut timestamp), difficulty 2^18 tient
(~100 ms bench), intégration fail-closed ?

### Verifications effectuées

- `crates/nexus-core-rs/src/pow.rs` : algorithme = leading-zero-bits
  sur `SHA256(canonical_bytes(challenge) || nonce_le_bytes)`. Domain
  separation `DOMAIN_POW_V1` via prefix dans canonical bytes
  (`canonical.rs:126`, `pow.rs:256-257`), test
  `canonical_bytes_are_deterministic_and_include_domain_tag`
  (`pow.rs:592-604`). ✅
- Binding challenge = `{v, topic, publisher_pubkey, issued_at,
  difficulty}` via JCS canonical. Cross-topic replay testé rejeté
  (`different_topics_yield_different_solutions` `pow.rs:516-530`).
  Cross-publisher replay testé rejeté (`pow.rs:533-547`). ✅
- Anti-replay timestamp : `issued_at` unix secs, `MAX_PROOF_AGE_SECS
  = 1800` (30 min), future-clock rejeté `IssuedInFuture` 
  (`pow.rs:550-568`), expired rejeté `verify_rejects_expired_proof`
  (`pow.rs:570-589`). ✅
- Bench `crates/nexus-core-rs/benches/pow.rs:47-58` : 3 benches
  définis (2^12 ~5 ms, 2^18 default target ~100 ms, 2^20 ~400 ms).
  Criterion sample_size adaptatif pour CI friendliness. **Mais aucun
  chiffre wall-clock archivé dans git** (output Criterion pas
  capturé) → **finding P2-B1**.
- Policy omission : `load_relay_pow_policy()`
  (`relay_pow_policy.rs:207-219`) → absent/malformed = loud error +
  DEFAULT_POLICY (2^18). **Fail-closed par design**. ✅
- Intégration `subscribe_with_pow` : **intentionnellement absente**
  du commit S19. Commit body `edfc51b` : « Cette integration est
  INTENTIONNELLEMENT differee Sprint 20+ pour (a) éviter risk
  breakage flows gossip existants... (b) permettre rollout sélectif
  per-topic... (c) laisser S20 Phase 0 auditer la primitive +
  envelope + caches isolément ». Aucun wire runtime S19. Le design
  doc `S19_phase_B_pow_hashcash_design.md` le trace explicitement.
  **Noté P2-A2 cap G7** ci-dessus comme carry implicite S20.
- Tests : 32 Rust (14 `pow.rs` + 6 `relay_pow_policy.rs` + 12
  `pow_gossip.rs`). Coverage edge cases solide (difficulty 0,
  difficulty max clamp, tampered nonce/hash/difficulty, version
  reject, leading-zero-bits boundaries, cache invalidation). ✅

### Findings

**P2-B1** — **Bench PoW chiffre non-archivé** :

`benches/pow.rs:89` dit « measured via `cargo bench --bench pow` »
mais aucune trace chiffrée n'est archivée dans git. La défense vis-à-
vis d'une régression runtime future (si le compiler flag change, si
un hash optimisé est remplacé, si une dep SHA256 downgrade) requiert
un chiffre de référence.

Fix suggéré S20+ : script CI qui parse l'output Criterion (`grep
"time:" criterion-output` ou équivalent) et fail si `time: 2^18 >
300ms`. Alternative : capturer un `docs/rust/PATTERNS.md §PoW bench
reference` avec mesures locale datées. Non-bloquant S20 Phase A.

**P2-B2** — **Divergence fail-open fail-close plan vs code Track C**
(ce finding est noté sous Track B par cohérence méta, mais concerne
Track C — voir Track C section dédiée **P2-C1**).

**P3-B1** — **D2 kickoff cite `Tor PoW 2023` sans mentionner
l'abandon de Hashcash pour Equi-X** :

Le kickoff S19 §4 D2 cite « Tor rend-point PoW 2023 » comme référence
pour 2^18 Hashcash SHA256. Or Tor 0.4.8.4 (août 2023) a abandonné
Hashcash pour **Equi-X** (HashX memory-hard, ~16 MB RAM). Le
rationale D2 est donc potentiellement obsolète.

Le design doc `S19_phase_B_pow_hashcash_design.md §3.6 + §6.2`
**rattrape explicitement** : analyse alternatives Equi-X (pas d'impl
Rust auditée, crypto custom non-RFC, sur-engineered pour S19 sans
difficulty adaptive), rejet documenté pour S19, re-évaluation
explicite Sprint 22+ (Kudos-weighted gossip admission + difficulty
dynamique). Doc trail clair.

Pas un P2 : le **design doc découvre et corrige post-hoc** le gap du
kickoff. Pattern souhaitable : le kickoff D2 devrait directement
citer l'alternative Equi-X et son rejet. Fix S20+ : pour les D1..D5
futurs touchant crypto/spec, le kickoff doit enumérer l'alternative
**avec** rejet cité (§6.1 Design Review Board G1 doit intercepter ce
type de drift — pattern à renforcer).

---

## Track C — TLS cert pinning relays : **PASS avec 2 P2**

**Question centrale** : SPKI extract RFC 7469 (subfield pas cert
complet), fail-closed pinset empty, rotation doc actionnable,
intégration iroh ?

### Verifications effectuées

- `crates/nexus-core-rs/src/tls_pinning.rs:291` :
  `let spki_der = cert.tbs_certificate.subject_pki.raw` — hash
  appliqué à **SPKI subfield DER-encoded** (RFC 7469 §2.4 conforme),
  pas au cert complet. Test
  `extract_spki_sha256_from_pem_matches_openssl_pipeline` (ligne
  593) valide bit-for-bit vs pipeline openssl documenté
  `RELAY_PIN_BOOTSTRAP.md §1`. ✅ **P0 évité** (le risque le plus
  critique de Track C).
- Fixture `relay_test_cert.pem` déterministe (hash fixe
  `Aq1c_N_zjopBnfg-mcHBozX8dgA64izVtd_zgdDioXs`, 43 chars base64url
  no-pad). ✅
- `PinValidator::validate()` : conforme sur pinset **non-empty** —
  mismatch → `PinError::SpkiMismatch`, relai absent du pinset non-
  empty → `PinError::NoPin` (**fail-closed**). ✅
- Pinset **empty** : `tls_pinning.rs:492-497` : `if file.pins.
  is_empty() { warn!(...); return Ok(()); }` — **fail-OPEN avec warn
  + fallback WebPKI**. ⚠️ Cette posture est défendue comme « opt-in-
  then-strict » par le livreur (test
  `validate_empty_pinset_fails_open_with_warn` ligne 698). Mais
  l'audit plan S19 §Track C demande explicitement « Pinset empty
  test → refuse (fail-closed) ». Divergence plan vs code →
  **finding P2-C1** ci-dessous.
- Intégration iroh : `PinValidator::validate()` défini et testé en
  isolation (17 tests Rust), mais **jamais appelé au handshake TLS
  iroh 0.97**. Design doc §5.1 Option B explicite : iroh 0.97 n'ex-
  pose pas `relay::client::ClientBuilder::custom_cert_verifier`
  publiquement (cfg(test) seulement). Runtime validation =
  **WebPKI-only jusqu'à S20+**. T20 tech debt PATTERNS.md tracé.
  Noté dans **P2-A2 cap G7** comme carry implicite S20.
- Doc `PATTERNS.md §TLS cert pinning` : rotation 3-step user-
  actionnable (`openssl s_client` → edit `~/.sbfb/relay-pins.json` →
  hot-reload <50 ms). Procédure complète. ✅
- Bootstrap pins `RELAY_PIN_BOOTSTRAP.md §3.1` : placeholders, pas
  pins réels n0 embarqués. Explicite « Sprint 19 ferme sans jeu
  bootstrap ». Daemon boot sans `~/.sbfb/relay-pins.json` → pinset
  empty → fail-open + warn (cf. finding P2-C1). → **finding P2-C2**.

### Findings

**P2-C1** — **Divergence fail-open pinset empty vs plan fail-closed
attendu** :

Le `sprint19_plan.md §6.4` écrit textuellement :
> `loader_missing_file_empty_pinset` : pas de fichier → empty pinset
> (tous relays refusent — documented behavior S19 pre-release, pre-
> launch on whitelist les 3 n0 defaults)

Le code `tls_pinning.rs:492-497` fait l'opposé : **pinset empty =
fail-open + warn + fallback WebPKI**. Le livreur défend la posture
« opt-in-then-strict » et teste explicitement ce comportement
(`validate_empty_pinset_fails_open_with_warn`). Le plan disait
explicitement « fail-closed ».

Les deux postures sont défendables :
- **Fail-closed** (plan) : zero-config daemon refuse tous les relays
  → UX cassée pré-launch sans bootstrap pins.
- **Fail-open + warn** (code) : zero-config daemon utilise WebPKI
  seul → UX préservée mais pinning désactivé silencieusement.

Le code choisit le pragmatisme (pas bootstrap pins embarqués S19 =
fail-closed serait inutilisable). Mais le plan disait l'inverse et
**aucun commit body n'explicite ce choix de design deviation**. Le
phase C review le documente mais pas le commit lui-même.

Fix : ajouter au kickoff S20 ou doc PATTERNS.md §TLS pinning une
note « S19 design deviation : fail-open sur pinset empty (vs plan
§6.4 fail-closed). Rationale : pas de bootstrap pins embarqués S19.
Bascule vers fail-closed quand bootstrap pins ajoutés release tag
(S20+) ». Sans cette trace, un auditeur futur sera perplexe.

**P2-C2** — **Bootstrap pins placeholders au lieu de pins n0 réels** :

`docs/release/RELAY_PIN_BOOTSTRAP.md §3.1` explicite « Sprint 19 ferme
sans jeu bootstrap embarqué — S20 après co-sig maintainer ». Le
daemon boot sans `~/.sbfb/relay-pins.json` → fail-open + warn
fallback WebPKI.

Impact : la protection TLS pinning est **opt-in manuel** pour tout
user S19. Un user qui install le daemon post-release S19 avant S20
est protégé **uniquement** par WebPKI. Un attaquant qui compromet
une CA WebPKI (pas impossible 2026, cf. BR incidents) peut MITM les
3 relays n0 sans détection pin. La promesse « TLS pinning relays
S19 » est partielle.

Fix : intégrer les 3 SPKI réels n0 (extraits via `openssl s_client`
sur `relay.iroh.network`, `relay-1.iroh.network`, `relay-2.iroh.
network`) au code + bundle `relay-pins.bootstrap.json` embarqué dans
le daemon binary. S20 target ou release tag. Non-bloquant S20 Phase A
(le daemon fonctionne en fail-open + warn).

### Pas de finding

- SPKI extract RFC 7469 §2.4 conforme (pas cert complet). ✅
- Rotation procedure actionnable <15 min. ✅
- `forbid(unsafe)` preserved, `deny_unknown_fields` on schemas. ✅
- 17 tests Rust (+9 vs plan §6.2). ✅

---

## Track D — Delayed upload queue : **PASS avec 1 P2**

**Question centrale** : jitter réellement anti-correlation,
bypass path absent, persistence assumée, concurrent submit safe ?

### Verifications effectuées

- `packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py:
  258-263` : formule `raw = -mean * log(u)` où `u ~ Uniform(0,1)`.
  C'est inverse-CDF **exponentielle** correcte (pas un biais
  exponential-vers-0 au sens incorrect). Avec `mean=90`,
  `max=300` (clamped interne à 270 pour observable <300 avec
  flush 30 s granularity) : median ≈ 62.4 s
  (= ln(2)×90), p63 ≈ 90 s, p87 ≈ 180 s, p99 ≈ 270 s. Fat tail
  intacte. ✅ Anti-correlation valide (cf. Cornell ESORICS 2006 +
  Loopix 2017 préfèrent exponential à uniform précisément pour
  k-anonymity via fat tail).
- Paths submit : grep `gossip.publish`, `gossip_emit`, `iroh_gossip`
  dans `packages/nexus-coordinator/src/nexus_coordinator/api/` →
  aucun call direct. Seule mention `daemon.py:275` est un
  commentaire. Tous les submit passent par
  `upload_queue.schedule()` (api/tasks.py:116). **Zéro bypass**. ✅
- Persistence : SQLite WAL schema ligne 148-158 `upload_queue.sqlite`
  table `delayed_uploads`. Design doc §5.2 justifie la deviation
  vs plan §7.4 (in-memory) : « coord crash within 90 s of submit →
  silent task loss unacceptable pre-launch ». Commit body et
  PATTERNS.md §P29 documentent le trade-off. ✅
- Concurrent submit : `upload_queue.py:192-216` cap check +
  `INSERT INTO delayed_uploads` dans le **même bloc** `async with
  self._lock`. Test regression
  `test_hard_cap_enforced_under_concurrency` (ligne 364-404) :
  20 `asyncio.gather(...)` vs cap=5 → exactement 5 OK + 15
  `QueueFullError`. Race fully mitigated. ✅
- Tests : 13 `test_upload_queue.py` primitive + 2
  `test_api_tasks_delayed.py` intégration + 4 edits
  `test_dispatcher.py` (preserve sync path testing) = **19 nets**.
  Dépasse le +15 commitment. Distribution coverage, concurrency
  regression, persistence durability, scheduler tick, backpressure
  429. ✅

### Findings

**P2-D2** — **Ruff format non-clean sur 2 fichiers Phase D** :

`uv run ruff format --check packages/` au tip `bf7dd62` retourne :

```
Would reformat: packages\nexus-coordinator\src\nexus_coordinator\upload_queue.py
Would reformat: packages\nexus-coordinator\tests\test_upload_queue.py
2 files would be reformatted, 102 files already formatted
```

Les 3 diffs concernent des line-breaks cosmétiques (ruff veut
réduire la segmentation : `SELECT COUNT(*) FROM delayed_uploads`
sur une ligne au lieu de 3, etc.). Aucun impact sémantique.

**Mais** : `sprint19_verification.md §CI & test suites` claim
explicitement « Ruff : `uv run ruff format --check packages/` +
`uv run ruff check packages/` clean ». C'est **faux** au tip
archivé. Le livreur a probablement run `ruff check` (qui passe,
style+lint) et pas `ruff format --check` (qui échoue), ou un pré-
run a été oublié après les last-minute edits Phase D.

Discipline README.md §4.3 « Tout rouge bloque le commit » pas
respectée : ruff format --check est un « rouge » qui aurait dû
bloquer le commit Phase D ou la Phase F wrap-up.

Fix : `uv run ruff format packages/` applique les 3 reformats.
Trivial, inclus dans le commit chore batch audit-P2 (ci-dessous).

Non-bloquant S20 Phase A (le code tourne, tests passent, c'est du
formatting ruff-opinion).

**P2-D1** — **Caveat plaintext payload peu visible dans PATTERNS.md
§P29** :

`docs/shell/PATTERNS.md` §P29 ligne 1825+ explique bien « Why SQLite-
persisted (vs in-memory as plan §7.4) ». Mais la mention « payload
stored plaintext dans `upload_queue.sqlite`, no encryption at rest »
est ligne 1899 **enfouie dans la sous-section Tech debt T-S19-D-1 ».
Un opérateur qui lit PATTERNS.md pour décider `enabled=true` dans
`coordinator.toml` ne voit pas le caveat « no encryption at rest »
en haut de la section.

Risk : un deploy mal informé activerait la queue sans savoir que les
tasks en transit sont sur disque en clair (pre-S20 encryption at
rest big rock).

Fix : ajouter un callout « ⚠️ WARNING : payload stored plaintext, no
encryption at rest (Sprint 20 big rock) » après la ligne 1818
(« Rule ») pour visibilité opérateur. 3 lignes doc. Non-bloquant
S20 Phase A.

### Pas de finding

- Distribution fat tail valide anti-correlation. ✅
- Tous paths submit via queue. ✅
- Race concurrent submit mitigé + regression test. ✅
- Tests +19 nets, dépasse commitment. ✅

---

## Track E — pkarr relay self-hosted docker image : **PASS avec 1 P2**

**Question centrale** : Dockerfile reproducible, workflow safe,
doc ops exécutable SRE 30 min, cross-ref Phase C TLS pinning ?

### Verifications effectuées

- `docker/pkarr-relay/Dockerfile`  : `FROM rust:1.94-slim-bookworm`
  (pinné version, pas `FROM rust` tagless — **P0 évité**).
  `USER` non-root, `tini` présent, `HEALTHCHECK` exposé, `pkarr-relay
  --version 2.1.*` pin range caret. Mais **pas de `@sha256:<digest>`
  pin** → **finding P2-E1** ci-dessous.
- `.github/workflows/build-pkarr-image.yml` : permissions `contents:
  read, packages: write, id-token: write, attestations: write` =
  **minimales**. `aquasecurity/trivy-action@0.24.0` **pinné version
  spécifique** (pas `@master`, P1 évité). `actions/checkout@v4` +
  `docker/setup-buildx-action@v3` + `docker/login-action@v3` +
  `docker/build-push-action@v5` + `sigstore/cosign-installer@v3` +
  `actions/attest-build-provenance@v1` = **version tags, pas SHA**.
  → explicitement carry-over S18 E3-2 P3 « quand pin SHA policy
  étend aux 4 workflows GHA en une fois ». Cohérent. ✅ (mentionné
  audit_plan §Track E).
- Trivy scan `exit-code: 1` sur `CRITICAL,HIGH`. Fail-closed. ✅
- Aucun `echo ${{ secrets.X }}` anti-pattern. `secrets.GITHUB_TOKEN`
  utilisé uniquement pour docker login standard. Pas de secret leak
  possible dans logs. ✅
- `docs/release/PKARR_RELAY_OPS.md` §1-§7 : lecture section par
  section. §1 rationale + coûts Hetzner CX22 clair. §2 chain of
  trust cosign verify 3 commandes exactes. §3 provisioning 18
  commandes copy-paste UFW + Docker + Caddy + systemd unit 20 lignes
  hardening (`NoNewPrivileges`, `ProtectSystem=strict`,
  `CapabilityBoundingSet=`, `SystemCallFilter=@system-service`) + §6
  smoke test 3 curls exacts. §4 rotation SPKI via `openssl s_client`
  extraction exacte + JSON template + cross-ref
  `RELAY_PIN_BOOTSTRAP.md` + 2 workflows (private deploy file-watcher
  hot-reload, federated PR relays.json). **Évaluation SRE 30 min :
  actionnable**. ✅
- Smoke test `tests/ci-smoke/pkarr-relay-healthcheck.sh` : pull image
  + run container localhost:6881 + curl `GET /` timeout 30 s. Exit
  codes 0/1/2/3. Invoqué après Trivy scan + cosign sign. ✅

### Findings

**P2-E1** — **Dockerfile base image pas `@sha256:<digest>` pinned** :

`docker/pkarr-relay/Dockerfile:14` : `FROM rust:1.94-slim-bookworm`
sans `@sha256:<digest>`. Le tag `rust:1.94-slim-bookworm` est mutable
upstream — Docker Hub peut push une nouvelle image sous le même tag
(CVE patch), cassant la reproducibilité strict du build.

Impact : un rebuild du workflow à N jours plus tard produit une
image différente (SLSA attestation différente, Trivy scan potentiel-
lement différent). Pour un binary de relay DHT en prod, la reproduc-
ibilité est une propriété défensive importante (audit indépendant
peut reproduire, tampering upstream détectable).

Fix : ajouter digest SHA256 de la version `rust:1.94-slim-bookworm`
stable actuelle. Ex: `FROM rust:1.94-slim-bookworm@sha256:<digest
SHA256 fixe>`. Renewal trimestriel (ou lors de security patch).
Non-bloquant S20 Phase A. S20+ ou release tag.

### Pas de finding

- FROM pinned version (`1.94-slim-bookworm`, pas `rust` tagless). ✅
- Permissions minimales (pas write-all). ✅
- Trivy pinned version, fail-closed CRITICAL/HIGH. ✅
- Secret leak anti-pattern absent. ✅
- §3 systemd unit présent et complet. ✅
- §7 rotation SPKI cross-ref Phase C complet. ✅
- SRE 30 min deployment actionnable. ✅

---

## Track F — Wrap-up coherence : **PASS**

**Question centrale** : docs Phase F cohérents entre eux et avec tip
final, migration PARA complète, flip S18 correct ?

### Verifications effectuées

- `git log --oneline 1a606a3..HEAD` = **13 commits** (audit plan §Mode
  d'emploi attendait ~11). Split en : 1 chore open S19 + 5 feat
  A/B/C/D/E + 1 fix Phase B follow-up + 2 chore planning
  inter-phase (guardrails G1..G7 `fe0a8fd` + Phase D wrap `2fd6c60`)
  + 2 chore tooling G4 hors-sprint (`4216436` + `c609a03`) + 2 wrap-
  up Phase F (`619059b` + `bf7dd62` placeholders). Cohérent avec ce
  qui est documenté dans verification §Commit stack (la verification
  mentionne bien les 2 chore tooling G4). ✅
- `CLAUDE.md §État actuel` : mentionne tip `bf7dd62`, compteurs
  537/185/208+3/46/239/38/7-7 ~1259 tests, Sprint 19 CLOSED entry,
  commits stack complet + re-carry Meta-1 S20. ✅ (claim-drift
  « pleinement active » **P2-A1** déjà noté).
- `docs/claude/SPRINT_LOG.md` row S19 ligne 22 : phase stack complet,
  theme « transport hardening », statut DONE, mention re-carry
  Meta-1. ✅ (claim-drift idem).
- Memory `nexus_grid_pivot.md` frontmatter description : tip
  `bf7dd62` sync (vérifié HEAD = MEM_TIP), compteurs cohérents,
  Sprint 20 READY mention. ✅ (HEAD_SHA == MEM_TIP = `bf7dd62` au
  démarrage de cette session audit).
- `ls .planning/active/` : **1 fichier** résiduel
  `sprint19_phase_F_review.md`. verification.md §Migration PARA
  l'explique comme artefact nécessaire à satisfaire le hook
  `phase-auditor-gate.sh` au commit Phase F. Pattern identique S18
  avec `sprint18_phase_F_review.md` (migré dans chore commit
  ultérieur). **Il doit être migré dans le commit chore d'ouverture
  S20**, sinon drift. ⚠️ (pas un finding — documenté comme
  pattern). Le pattern pourrait être fix dans le hook
  (accepter le fichier s'il existe dans `archive/v{X}/` AUSSI) mais
  c'est scope harness S20+ et hors-S19.
- `ls .planning/archive/v1.2/sprint19_*.md` : **10 fichiers** :
  kickoff + plan + verification + audit_plan + supervision_log + 5
  phase reviews A/B/C/D/E. ✅
- Flip S18 : `grep -E '\[x\].*DHT redundant' sprint18_verification.md`
  → 1 match ligne 72 avec annotation `wire Phase A S19 ab6985c`. ✅
  (claim « pleinement active » **P2-A1** déjà noté).
- Fail-fast 28 rows verification.md §10 : rempli post-hoc avec
  footnote (§Delta tests recapitulatif) qui reconnaît que les
  cumulants intermédiaires (485 / 514 / 523 / 525 / 537) sont
  reconstitués post-facto, pas mesurés par phase réellement. Le
  total final 537 fait foi. **Honnête mais signale que la discipline
  "mesurer per-phase" a dérivé**. Noté dans verification.md par le
  livreur — pas un finding audit, c'est self-remporté.
- G5 working tree audit sections dans commit bodies :
  - Phases A/B/B-fix/C `ab6985c`/`edfc51b`/`08f4e41`/`540bb51` :
    **absente** (ces phases sont ANTE `fe0a8fd` chore guardrails
    G1..G7 qui a instauré G5 en cours de sprint).
  - Phases D/E/F `f238d31`/`2fd4d72`/`619059b` : **présente**, bien
    formatée, catégorisation PHASE/CRAFT/DEBT/NOISE appliquée. ✅
  - Phase A/B/C/B-fix **ne peuvent pas être blâmées rétroactivement**
    pour l'absence G5 — ils sont pré-discipline. Non-finding.

### Pas de finding propre à Track F

Tout claim `pleinement active` renvoie au finding **P2-A1**.

---

## Meta-track — Radicle-v1.0 activation tracking re-carry S20 : **PASS**

### Verifications effectuées

- `sprint19_audit_plan.md §Meta-track Radicle-v1.0 activation
  tracking (re-carry S18 → S19 → S20)` : présent ligne 246+,
  explicite (owner FlowUP, deadline tag v1.0, runbook
  `MIRROR_FALLBACK.md §3.1-3.8` self-contained, 5 secrets GHA, 3
  checks post-activation). ✅
- `MIRROR_FALLBACK.md §3.1-3.8` : lecture rapide, self-contained
  (flip sequence GitHub private → public + Radicle activation + CANARY
  extension + status update). ✅
- Le pattern « re-carry annuel-ish tant que v1.0 pas tagged » est
  assumé. Pas de finding — meta-track fait son job.

### Pas de finding

---

## Findings sortés par sévérité

| # | ID | Severity | Track | Description courte |
|---|---|---|---|---|
| 1 | A-1 | P2 | A | Claim-drift « Eclipse-by-DHT pleinement active runtime » dans 5 docs vs code canary opt-in env var (`SBFB_PKARR_RELAYS`) default OFF |
| 2 | A-2 | P2 | A (cap G7) | Cap carry-overs G7 dépassé implicitement : 5 carries réels S19→S20 (Meta-1 + gitignore + PoW wire + TLS wire + DHT strict) vs cap 2. Kickoff S20 doit trancher |
| 3 | A-3 | P2 | A | Tests intégration cas heureux quorum absent (`probe_and_cache_with_quorum_majority_continues_dial` manquant). ~30 LOC test |
| 4 | B-1 | P2 | B | Bench PoW wall-clock chiffre non-archivé git. Regression detection nécessite parse CI criterion output |
| 5 | C-1 | P2 | C | Divergence fail-open (code) vs fail-closed (plan §6.4) sur pinset empty. Design deviation légitime mais sans trace commit body |
| 6 | C-2 | P2 | C | Bootstrap pins placeholders, pas SPKI réels n0 embarqués. TLS pinning opt-in manuel S19 |
| 7 | D-1 | P2 | D | Plaintext payload caveat enfoui dans PATTERNS.md §P29 tech debt section, pas visible en haut de la règle |
| 8 | D-2 | P2 | D | `ruff format --check packages/` échoue sur `upload_queue.py` + `test_upload_queue.py` (3 reformats cosmétiques). `sprint19_verification.md §CI` claim « Ruff clean » est faux. Discipline §4.3 « tout rouge bloque commit » pas respectée |
| 9 | E-1 | P2 | E | Dockerfile `FROM rust:1.94-slim-bookworm` sans `@sha256:<digest>`. Reproducibilité build dégradée |
| 10 | B-2 | P3 | B | D2 kickoff cite « Tor PoW 2023 » sans mentionner l'abandon de Hashcash pour Equi-X. Rattrapé post-hoc dans design doc §3.6 — pattern à renforcer Design Review Board G1 |
| 11 | A-4 | P3 | A | Pattern « sprint19_phase_F_review.md résiduel dans active/ » à migrer dans chore ouverture S20. Non-bloquant, répétition du pattern S18 |

**Total** : **0 P0**, **0 P1**, **9 P2**, **2 P3**. Rigor signal G4
satisfait (>1 P2+). Verdict : **PASS**.

---

## Commits fix attendus

Aucun `fix(sprint19):` atomique n'est exigé avant S20 Phase A (pas
de P0/P1).

Les 8 P2 + 2 P3 peuvent être traités au choix :

**Option A** — en batch dans un commit `chore(sprint19): audit-P2
batch` pré-ouverture S20, inline avant le premier
`feat(sprint20): Phase A`. Pattern S18 `1a606a3` (P3 batch) étendu
aux P2. Scope : 8 P2 + 2 P3 = ~200 LOC doc + ~30 LOC test A-3.

**Option B** — intégrer au `chore(planning): open S20` lui-même :
le kickoff S20 §4 D1..D5 adresse le cap G7 A-2 (trancher carries) +
documente dans §6 Scope cuts les reports explicits de PoW wire
S20 Phase 1, TLS wire S20+, DHT strict S-post-Gate 2. Les autres P2
(A-1 claim fix, A-3 test, B-1 bench, C-1 commit note, C-2 bootstrap
pins, D-1 PATTERNS callout, E-1 Dockerfile SHA, P3-B2 doc trail,
P3-A4 phase_F migration) peuvent rester P2 « traitables pendant le
sprint » sans bloquer Phase A.

**Option C** — pattern recommandé session fraîche : 1 commit
`chore(sprint19): audit-P2 batch — claim fix + cap G7 acknowledge
+ bootstrap pins + dockerfile SHA + PATTERNS callouts` + 1 commit
`test(sprint19): audit-P2 A-3 — probe_and_cache quorum majority
continues dial`. Deux commits atomiques thématiques. Laisse le
kickoff S20 propre des préoccupations S19.

Trancher avec l'utilisateur avant d'exécuter.

---

## Tech debt à logger (optionnel)

Les P2 suivants peuvent aussi être loggés dans
`docs/rust/PATTERNS.md` ou `docs/shell/PATTERNS.md` comme tech debt
tracked plutôt que fix inline :

- **A-3** (tests integration cas heureux manquant) → PATTERNS.md §T*
  « browse aggregator quorum canary — happy path integration test »
- **B-1** (bench PoW wall-clock absent) → PATTERNS.md §T* « PoW bench
  regression CI script »
- **C-2** (bootstrap pins placeholders) → PATTERNS.md §T* « TLS
  bootstrap pins S20+ » (déjà partiellement dans
  `RELAY_PIN_BOOTSTRAP.md §3.1`)
- **E-1** (Dockerfile @sha256) → tech debt ops « Docker image base
  digest pin sweep »

Les autres (A-1 claim fix, A-2 cap G7, C-1 commit note, D-1
callout, P3-B2, P3-A4) sont des fixes docs quick qui devraient être
directement commités.

---

## Carry-overs S19 → S20 (cap G7 review)

Décision à prendre au kickoff S20 §4 D1 et §6 Items carry/dette :

| # | Item | Source | Type | Recommandation |
|---|---|---|---|---|
| 1 | **Meta-1 Radicle-v1.0 activation tracking** | Re-carry S18→S19→S20 | Meta (tag v1.0 dependent) | Re-carry explicite S20 §Meta-track |
| 2 | **P2-2 .gitignore NOISE coverage** | S19 audit Phase F + commit body Phase F | Chore | Livre inline dans `chore(planning): open S20` |
| 3 | **PoW runtime wire gossip subscribe** | `edfc51b` body + P2-A2 ici | Carry implicit S20 Phase 1 | Explicit **dans kickoff S20 §D1 scope** (pas carry-over si intégré au scope) |
| 4 | **TLS pinning wire iroh handshake** | Design doc §5.1 + T20 tech debt + P2-A2 ici | Tech debt long-terme | Reclasser T20 dans `docs/rust/PATTERNS.md §Tech debt` plutôt que carry-over sprint |
| 5 | **DHT canary → enforcement strict** | P2-A1 + P2-A2 ici | Design decision post-Gate 2 | Document dans `HARDENING_ROADMAP §3` comme « Gate 2 pre-req, post-Gate-2 items »  — pas un carry S20 |

**Après reclassification** : cap G7 = 2 carries réels (Meta-1 +
gitignore) + 1 carry intégré au scope (PoW wire S20 Phase 1) + 2
tech debt long-terme (TLS wire, DHT strict). Cap 2 respecté **à
condition** que le kickoff S20 fasse la reclassification
explicitement. C'est l'objet du finding **P2-A2**.

---

## Notes on audit completeness

**Ce qui a été audité** :
- 6 tracks + meta-track, 5 délégués à agents Explore parallèles
  indépendants, 2 traités en direct (F + Meta).
- Compteurs tests réels Rust (537), SDK (185), app-gov (46)
  confirmés. Coord run local (192 passed + 16 failed) a révélé un
  wheel stale nexus-core-py dans le venv (non-finding S19, cf.
  §Mode d'emploi §7).
- 5 endroits de claim-drift « pleinement active » identifiés par
  grep cross-docs.
- Tous les design docs `.planning/research/S19_phase_{B,C,D,E}_*.md`
  ont été vérifiés présents. Pas de Phase A design doc (acceptable
  car carry S18 C-1, primitive déjà en S18).

**Ce qui n'a pas été audité** :
- Suite Vitest (239) + Playwright (38) + size-limit (7/7) pas
  re-runnées localement. Baseline verification fiable sur les
  autres suites, donc claim probablement correct.
- Pas de bench `cargo bench --bench pow` re-run (finding P2-B1
  noté).
- Pas de simulation distribution jitter 1000 samples runtime
  (accepté la validation formelle de l'agent Track D + test
  `test_schedule_median_around_theoretical` +/-25 s tolerance).
- Pas de review intégrale du design doc
  `S19_phase_B_pow_hashcash_design.md` (900 lignes). Agent Track B
  a couvert §3.6 et §6.2 pertinentes.
- Pas de re-run CI GHA locally (`supply-chain.yml`, `build-pkarr-
  image.yml`, etc.). Workflows pas modifiés substantiellement hors
  ajout `build-pkarr-image.yml` Phase E.

**Skip explicite** : les phase reviews
`sprint19_phase_{A,B,C,D,E}_review.md` ont été lus en cross-check
**après** formation d'opinion indépendante par les agents. Le
review Phase A corrobore P3 signature 2-arg (déjà noté). Le review
Phase B documente la deviation runtime wire S20+. Le review Phase C
documente le fail-open (divergence plan). Le review Phase D
documente la deviation SQLite vs plan in-memory. Le review Phase E
documente le SHA pin absent. Les phase reviews ont tous **flaggé
les mêmes P2 que l'audit indépendant**, ce qui est un signal de
qualité convergent (et indique que le skill `nexus-phase-auditor`
fait son job intra-sprint).

**Budget auditeur** : ~2h incluant lectures, délégation agents,
grep, compteurs. Plan audit_plan suggérait 2-3h. Conforme.

---

## Recommandation synthèse

**Verdict PASS. Sprint 20 Phase A non-bloqué.**

Avant le premier `feat(sprint20): Phase A`, traiter les 8 P2 + 2 P3
via **Option C** (2 commits thématiques : 1 chore claim/doc + 1 test
A-3), ou **Option B** (intégrer dans kickoff S20 et chore open). La
priorité est **A-1 claim fix** + **A-2 cap G7 acknowledge** (les 2
P2 macro qui affectent le thinking S20), les autres sont du doc
hygiene low-risk.

La discipline sprint livre un vrai progrès (transport hardening +
5 design docs + 4 sections PATTERNS neuves). Le pattern « primitive
S{N} + wire opt-in/différé S{N+1} » mérite d'être nommé et arbitré
explicitement au kickoff S20 — soit on l'accepte comme pattern
légitime (« défense-en-profondeur incrementale par primitives »), et
on le cadre dans HARDENING_ROADMAP, soit on impose que chaque
primitive livrée soit wire strict dans le même sprint (réduit la
velocity mais élimine le risque claim-drift).

C'est une décision S20 D1 pour l'utilisateur.
