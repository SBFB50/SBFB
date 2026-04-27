# Sprint 32 — Kickoff (dette pair : iroh 0.98 upgrade + carries batch)

**Ecrit** : 2026-04-27 (session fraiche post-audit gate S31 `1cc4734`).
**Type** : **sprint pair dette** (§6.2.1 Regle 1 — phase dette
obligatoire). LT-6 trigger met (iroh 0.98.0 publie 2026-04-17).
Day 0 #3 (iroh 0.97 pin) **LEVE** pour ce sprint.
**Tip master d'entree** : `1cc4734` (chore(planning): sprint 31
audit findings — verdict PASS, 0 P0/P1, 2 P2, 3 P3).
**Phase 0 audit Sprint 31** : **DEJA JOUE** — findings dans
`.planning/active/sprint31_audit_findings.md` (verdict **PASS**,
0 P0/P1, 2 P2, 3 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-27) : HARDENING_ROADMAP last_validated
  `2026-04-27` (S31 Phase D). **3 triggers ACTIFS** inchanges depuis
  last_validated :

  1. **iroh 0.98.0** (2026-04-17, n0-computer/iroh) — trigger ACTIF.
     **ADRESSE CE SPRINT**. Pre-research confirmee via crates.io API
     + context7 + WebSearch (2026-04-27) :

     Breaking changes 0.97→0.98 (8 PR) :
     - `SecretKey::generate()` ne prend plus d'arg Rng (#4075)
     - Types publics marques `#[non_exhaustive]` (#4107)
     - `CustomAddr::as_vec` renomme `to_vec` (#4074)
     - Types d'erreur tiers plus exposes dans iroh-base (#4073)
     - Protocole relay v2 avec Health frame (#3955)
     - `Endpoint::online()` attend la connexion relay (#4115)
     - Address lookup registry restructure (#4130)
     - Timeouts corriges sur connexions relay (#4083)

     Nouvelles features : pluggable crypto backends (#3992),
     external addresses configurables (#4098), rate limiting
     router (#3951), pkarr vendored (#4026).

     Matrice deps confirmee crates.io API :
     - iroh-docs 0.98.0 requiert iroh ^0.98 + iroh-blobs ^0.100
       + iroh-gossip ^0.98
     - iroh-gossip 0.98.0 requiert iroh ^0.98
     - iroh-blobs 0.100.0 requiert iroh ^0.98
     - iroh-blobs 0.99 requiert iroh ^0.97 (**incompatible** 0.98)
     → **4 crates doivent upgrader simultanement**.

     Sources : [n0-computer/iroh releases](https://github.com/n0-computer/iroh/releases),
     [iroh CHANGELOG.md](https://github.com/n0-computer/iroh/blob/main/CHANGELOG.md),
     context7 `/websites/rs_iroh` (1743 snippets, score 84.8),
     crates.io API `/api/v1/crates/{iroh-blobs,iroh-gossip,iroh-docs}/*/dependencies`.

  2. **arti-client 0.41.0** (2026-03-30, crate crates.io) — trigger
     ACTIF. Note : le blog Tor Project annonce "Arti 2.0" (version
     projet), mais le crate Rust est `arti-client 0.41.0`.
     **INTEGRE S31 Phase C** (config + wire). Dep activation
     bloquee par rusqlite conflict → **ADRESSE Phase B ce sprint**.

  3. **openai-agents-python 0.14.6** (2026-04-25) — trigger ACTIF.
     Informationnel uniquement, pas de dep directe SBFB.

  Triggers INACTIFS : frost-ed25519 (2.1.0), wasmtime, Tor PoW
  hspow, NIST PQC FIPS, NVIDIA H100 CCM, RFC 9591, MCP spec,
  microsoft/sudo.

- **G9 Codebase Exploration (2026-04-27)** :

  Agent Explore iroh usage : `Endpoint::builder(presets::N0)` dans
  `node.rs`, `RelayMode::Custom(RelayMap)` dans `relay_config.rs`,
  `SecretKey::from_bytes()` dans `node.rs` (pas `generate`),
  iroh-docs `Doc`/`Author`/`LiveEvent` dans `docs.rs`, iroh-gossip
  `Gossip`/`GossipTopic`/`Event` dans `gossip.rs`, iroh-blobs
  `BlobsProtocol`/`BlobTicket`/`Downloader`/`MemStore` dans
  `blobs.rs`, `PkarrRelayClient` dans `pkarr_resolver.rs`.
  **Aucun usage de `ConnectionType`** (supprime en 0.98, pas de
  migration requise). Principal crate impacte : `nexus-core-rs`.
  Shell-daemon-core et worker-core consomment via les abstractions
  de core-rs.

- **rusqlite 0.32→0.36** : crates.io confirme rusqlite 0.36 utilise
  `libsqlite3-sys ^0.34`. Latest rusqlite = 0.39.0. Cible 0.36 =
  minimum suffisant pour unbloc arti-client, delta migration minimal
  vs 0.32 (API bundled inchangee, schema migrations compat).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 31 CLOSED. 4 phases A-D livrees :
- Phase A : task_runner reel Ollama wire LlmBackend executor
- Phase B : §9.5 output filter E2E post-verify + WebAppFrame cleanup
- Phase C : Tor transport phase 1 config + feature gate + coordinator
  wire (dep arti-client differee S32 par conflit rusqlite)
- Phase D : P2 batch S30 carries 6/6 fermes + G2 HARDENING refresh +
  4 HTTP FROST tests

Audit gate S31 : **PASS** (0 P0/P1, 2 P2 audit + 3 P3 audit).

### §1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP §3 ne prescrit pas de ligne specifique S32
(roadmap Sprint 18-30). Le roadmap v1.0 Alexandria (`c50976a` S30)
prescrivait iroh 0.98 en S31 Phase C — devie vers S32 (D5 S31).
S32 pair = phase dette = moment ideal pour absorber l'upgrade.

### §1.3 Compteurs tests entree (tip `1cc4734`)

| Suite | Count | Delta vs S31 sortie |
|---|---|---|
| Rust (cargo nextest) | 878 | 0 |
| SDK (pytest) | 195 (1 flaky Windows) | 0 |
| Coordinator (pytest) | 406 passed + 36 failed (PyO3 stale) + 6 skipped | 0 |
| Gov (pytest) | 46 | 0 |
| Vitest | 267 | 0 |
| Playwright | 41+2f (env) = 43 | 0 |
| size-limit | 7/7 | 0 |
| **Total** | **~1877** | **0** |

Note : les 2 commits post-S31 wrap-up (audit findings + config pin)
ne touchent aucun code = compteurs identiques S31 sortie.

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` = 1 partout. L'upgrade
iroh 0.97→0.98 est une migration **interne** (deps Cargo) — aucun
wire format P2P ne change. Les wire formats SBFB (Task, ProjectAnnouncement,
CuratorList, CanarySigned, etc.) restent v1. Le protocole relay iroh
passe en v2 (#3955) mais c'est transparent pour SBFB (iroh gere le
relay en interne). Pas de bump version, pas de tolerant decoder.

---

## §2 Goal en une phrase

Sprint 32 leve le pin iroh 0.97 (Day 0 #3) en upgradeant les 4
crates iroh vers 0.98/0.100, debloque l'activation arti-client via
rusqlite 0.36, et resout les carries P2 audit S31.
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 31

**DEJA JOUE** — commit `1cc4734`.

Verdict : **PASS** (0 P0/P1, 2 P2 audit + 3 P3 audit).

Findings integres dans ce kickoff :
- P2-AUDIT-1 : executor silent param drops (max_tokens) → Phase C
- P2-AUDIT-2 : HARDENING compteurs stale → Phase C
- P3-AUDIT-1 : tor feature gate compile trap → Phase B (arti dep activation)
- P3-AUDIT-2 : FROST error path tests → Phase C
- P3-AUDIT-3 : Tor boot log misleading → Phase C

ROADMAP_COMMITMENTS check (G7 Regle 3) :
- LT-1 a LT-5 : conditions latentes, aucun declenchement.
- **LT-6** : trigger "iroh > 0.97" met (iroh 0.98.0 2026-04-17).
  Day 0 #3 pin **LEVE** ce sprint. LT-6 passe de "scheduled S32"
  a **INTEGRE Phase A**. ROADMAP_COMMITMENTS mis a jour avec note
  "resolved S32 Phase A".

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — iroh stack : upgrade simultane 4 crates

**Retenu** : Upgrade simultane workspace-wide :
- `iroh` 0.97 → 0.98
- `iroh-docs` 0.97 → 0.98
- `iroh-gossip` 0.97 → 0.98
- `iroh-blobs` 0.99 → 0.100

Les 4 crates forment une chaine de deps : iroh-docs 0.98 requiert
iroh ^0.98 + iroh-blobs ^0.100 + iroh-gossip ^0.98. Pas de
resolution mixte possible. Day 0 #3 (pin iroh 0.97) est **leve**.

Migration ciblee : 8 breaking changes PR (#4075 SecretKey::generate,
#4107 non_exhaustive, #4074 CustomAddr::to_vec, #4073 error types,
#3955 relay-v2, #4115 Endpoint::online, #4130 address lookup,
#4083 relay timeouts). Impact principal sur `nexus-core-rs/src/`
(node.rs, relay_config.rs, discovery.rs, blobs.rs, docs.rs, gossip.rs).

**Rejete** :
- *Upgrade partiel (juste iroh core, garder blobs 0.99)* — impossible,
  iroh-docs 0.98 requiert iroh-blobs ^0.100. Resolution cargo echoue.
- *Attendre iroh 1.0* — pas de timeline publiee. 0.98 est le dernier
  stable depuis 10 jours. Le trigger LT-6 est actif depuis 2026-04-17.
  Chaque sprint de plus = tech debt supplementaire.
- *Upgrade opportuniste en phase feature* — risque cascade (8 breaking
  changes workspace-wide). Sprint pair dette = contexte dedie, pas
  d'autre feature concurrente.

**Implications code** : `Cargo.toml` workspace deps (4 lignes),
`crates/nexus-core-rs/src/` (node.rs, relay_config.rs, discovery.rs,
blobs.rs, docs.rs, gossip.rs — compilation fix), possiblement
`crates/nexus-shell-daemon-core/src/` si les abstractions ne suffisent
pas. Tests existants : 878 Rust tous doivent rester verts.

### D2 — rusqlite 0.32→0.36 + arti-client 0.41 dep activation

**Retenu** : Upgrade `rusqlite` 0.32 → 0.36 (`bundled`, libsqlite3-sys
0.34) workspace-wide. Ensuite, decommenter la dep `arti-client = "0.41"`
dans `crates/nexus-core-rs/Cargo.toml` et remplir la feature
`tor = ["dep:arti-client", "dep:tor-rtcompat"]`. Le module
`tor_transport.rs` (S31 Phase C) reference deja `arti_client::TorClient`
sous `#[cfg(feature = "tor")]` — la dep est le dernier chainon.

La cible rusqlite 0.36 (pas 0.39 latest) minimise la surface de
migration tout en resolvant le conflit `libsqlite3-sys` avec
arti-client 0.41. L'API bundled est quasi-identique 0.32→0.36.

**Rejete** :
- *rusqlite 0.39 (latest)* — delta migration plus grand (3 versions
  intermediaires). Pas de feature requise au-dela de 0.36. Minimiser
  le risque en sprint dette.
- *Garder rusqlite 0.32 + skip arti activation* — P2-REVIEW-C-1 passe
  a 2/3. La feature gate `tor = []` reste un compile trap (P3-AUDIT-1).
  Pas de raison valide de differer quand le conflit est resolvable.
- *rusqlite sans bundled* — casserait le principe "single binary Rust"
  (Day 0 #10). La feature bundled embarque SQLite, pas de dep systeme.
- *arti-client via SOCKS proxy* — rejete S31 D3 (overhead daemon
  supplementaire, API directe plus simple).

**Implications code** : `Cargo.toml` workspace deps (rusqlite ligne),
`crates/nexus-core-rs/Cargo.toml` (decommenter arti-client + tor-rtcompat,
feature `tor = ["dep:arti-client", "dep:tor-rtcompat"]`),
`crates/nexus-core-py/Cargo.toml` (passthrough feature si necessaire),
verification que `cargo build --features tor` compile sans erreur.
Tests : `cargo build -p nexus-core-rs --features tor` doit passer.

### D3 — P2-AUDIT-1 executor param drops : wire max_tokens

**Retenu** : Wire `max_tokens` dans `GenerationRequest` via
`GenerationOptions::default().num_predict(params.max_tokens)` dans
`task_runner.rs`. Resout le gap de fidelite wire contract identifie
par l'audit S31 Track A (executor ignore silently les params du
coordinator).

Les champs `grammar` et `watermark_config` restent **P3 carry S33** :
- `grammar` : Ollama ne supporte pas GBNF natif, le champ est
  design-only. Wire quand le structured output Ollama sera teste.
- `watermark_config` : defense-in-depth, le SynthID watermark est
  inject worker-side (llama_cpp.rs), pas executor-side.

**Rejete** :
- *Wire les 3 champs d'un coup* — grammar et watermark_config n'ont
  pas de backend Ollama. Les wirer = dead code. Fidelite partielle
  (max_tokens seul) > fidelite cosmetique (3 champs dont 2 noop).
- *Refactor TaskExecuteParams pour supprimer les champs non-wires* —
  les champs existent pour le contrat IPC futur (llama.cpp executor,
  multiple backends). Les supprimer casserait le design intent.
- *Skip* — P2-AUDIT-1, identifie comme gap wire fidelite. Le
  coordinator croit controler max_tokens, l'executor l'ignore.
  Fix trivial (~5 LOC).

**Implications code** : `crates/nexus-executor/src/task_runner.rs`
(~5 LOC wire GenerationOptions), test
`execute_task_ollama_mock_respects_max_tokens` (~20 LOC).

### D4 — P2 batch audit S31 : docs stale + Tor log + FROST errors

**Retenu** : Batch des items P2/P3 audit S31 :

1. **P2-AUDIT-2** : HARDENING_ROADMAP.md compteurs frontmatter stale
   (~401 coord → 406). Fix doc ~5 LOC + update S32 entry.
2. **P3-AUDIT-3** : Tor boot log misleading (`coordinator.py:377-380`
   log "not available" quand `enabled=false` — differencier disabled
   vs unavailable). Fix ~5 LOC.
3. **P3-AUDIT-2** : FROST HTTP error path tests (k>n, malformed JSON,
   wrong participant, invalid nonces). Nice-to-have, ~40 LOC tests.
4. **P2-REVIEW-A-1** : LOC plan meta-process (1/3) — discipline
   plan-writing, pas d'action code. Note carry S33 si non resolu.
5. **P2-REVIEW-B-1-S30** : Playwright COEP iframe test (2/3). Tenter
   resolution si env Playwright stable. Sinon documenter exemption
   blocker externe (env instable) et noter **MANDATORY S33**.

**Rejete** :
- *Skip P2-AUDIT-2* — doc stale = drift G2. Fix trivial, pas de raison
  de differer.
- *Skip Playwright attempt* — atteindrait 3/3 S33, MANDATORY sans
  exemption possible. Tenter au moins une approche.
- *FROST error paths en phase separee* — trop petit pour une phase
  dediee. Batch avec les autres nits.

**Implications code** : `docs/security/HARDENING_ROADMAP.md` (compteurs
+ S32 entry), `coordinator.py` (log fix ~5 LOC), `http.rs` (~40 LOC
tests FROST error), possiblement `web/` (Playwright test ~30 LOC).

### D5 — Day 0 #3 : levee formelle du pin iroh 0.97

**Retenu** : Le pin iroh 0.97 (Decision Day 0 #3, actee au pivot
2026-04-10) est **formellement leve** par ce sprint. La decision
Day 0 #3 originale etait "iroh 0.97 pinne, **upgrade volontaire**"
— le terme "volontaire" implique un sprint dedie, pas un upgrade
opportuniste. S32 est ce sprint dedie.

Post-S32, la decision Day 0 #3 devient : "iroh stack upgrade
volontaire par sprint dedie — le pin est au niveau livre par le
dernier upgrade (0.98 post-S32)". La mecanique reste identique :
pas d'upgrade opportuniste, toujours un sprint dedie.

**Rejete** :
- *Garder le pin 0.97 un sprint de plus* — LT-6 trigger actif depuis
  10 jours. S32 est le sprint cible annonce dans S31 D5. Chaque
  sprint supplementaire est du carry injustifie.
- *Passer directement a iroh 1.0* — pas de release 1.0 publiee.
  0.98 est le dernier stable.

**Implications** : `CLAUDE.md §Decisions architecturales gelees` item 3
mis a jour ("iroh 0.98 pinne post-S32"), memory `nexus_grid_pivot.md`
§Day 0 mise a jour.

### Acknowledged review findings (G1)

Scoring : D1 ⚠️, D2 ❌ (corrige), D3 ✅, D4 ⚠️, D5 ✅.
Rigor signal G4 satisfait (1 ❌ corrige + 2 ⚠️ sur 5).

**D1 ⚠️** : "iroh 0.98.1 patch (2026-04-20) unremarked". Decision :
acknowledge — iroh 0.98.1 est un patch bug-fix au-dessus de 0.98.0.
La spec `iroh = "0.98"` dans Cargo.toml resout automatiquement vers
0.98.1 via semver `^0.98`. Pas de breaking change supplementaire.
Pas de modification du plan.

**D2 ❌** : "arti-client 2.0.0 factually wrong — actual 0.41.0".
Decision : **CORRIGE** — toutes les references `arti-client 2.0` dans
kickoff + plan remplacees par `arti-client 0.41`. Le blog Tor Project
annonce "Arti 2.0" (version projet), mais le crate Rust est
`arti-client 0.41.0` (crates.io, 2026-03-30). Error propagee depuis
S31 kickoff §D3 qui citait la version projet sans verifier crates.io.
La logique de resolution deps est inchangee.

**D4 ⚠️** : "Playwright env failure root cause not re-verified".
Decision : acknowledge — Phase C fera un fresh env check Playwright
avant de tenter le COEP test. Si root cause differente de "coordinator
not running", l'exemption sera re-evaluee avec evidence fraiche.

---

## §5 Plan Phase outline A..D

### Phase A — iroh stack upgrade 0.97→0.98 (dette pair, LT-6)

Phase dette obligatoire (§6.2.1 Regle 1 sprint pair).
- Bump `Cargo.toml` workspace deps : iroh 0.98, iroh-docs 0.98,
  iroh-gossip 0.98, iroh-blobs 0.100
- Resoudre les 8 breaking changes dans nexus-core-rs :
  - `SecretKey::generate()` sans Rng (#4075) — grep usage, adapter
  - `#[non_exhaustive]` sur types publics (#4107) — ajouter `..`
    dans les pattern matches si necessaire
  - `CustomAddr::as_vec` → `to_vec` (#4074) — rename
  - Error types iroh-base (#4073) — adapter les `?` / `From` impls
  - Relay v2 (#3955) — transparent si pas de relay config custom
  - `Endpoint::online()` semantique (#4115) — verifier le boot path
  - Address lookup (#4130) — adapter si usage direct
  - Relay timeouts (#4083) — transparent
- Verifier compilation + 878 tests Rust verts
- Commit cible : `feat(sprint32): Sprint 32 Phase A — iroh stack
  upgrade 0.97→0.98 workspace-wide`

### Phase B — rusqlite 0.36 + arti-client dep activation

- Bump `rusqlite` 0.32 → 0.36 workspace-wide
- Decommenter `arti-client = "0.41"` + `tor-rtcompat` dans
  nexus-core-rs Cargo.toml
- Feature `tor = ["dep:arti-client", "dep:tor-rtcompat"]`
- Verifier `cargo build -p nexus-core-rs --features tor` compile
- Resoudre P3-AUDIT-1 (compile trap `tor = []` → fonctionnel)
- Commit cible : `feat(sprint32): Sprint 32 Phase B — rusqlite 0.36
  + arti-client dep activation tor feature`

### Phase C — P2 batch carries audit S31

- P2-AUDIT-1 : wire max_tokens dans task_runner.rs + test
- P2-AUDIT-2 : fix compteurs HARDENING_ROADMAP.md + S32 entry
- P3-AUDIT-3 : fix Tor boot log misleading (disabled vs unavailable)
- P3-AUDIT-2 : FROST HTTP error path tests (k>n, malformed JSON)
- P2-REVIEW-B-1-S30 : tenter Playwright COEP iframe test (si env
  stable) OU documenter exemption (MANDATORY S33)
- Commit cible : `feat(sprint32): Sprint 32 Phase C — P2 batch
  carries audit S31 + Playwright COEP attempt`

### Phase D — Wrap-up + verification + audit plan S33

Standard wrap-up :
- sprint32_verification.md (fail-fast 28+ rows)
- sprint32_carry_summary.md
- sprint33_audit_plan.md
- SPRINT_LOG.md row S32
- CLAUDE.md §Etat actuel update (iroh 0.98 reference)
- Memory update nexus_grid_pivot.md + MEMORY.md
- HARDENING_ROADMAP.md update (iroh 0.98 trigger → resolved)
- ROADMAP_COMMITMENTS.md update (LT-6 resolved)
- Migration active/ → archive/v1.2/
- Commit cible : `chore(sprint32): Phase D — wrap-up + verification
  + audit plan S33 + migration`

---

## §6 Items carry/dette (G7)

### Carry S31 — resolution prevue S32

| ID | Description | Reports | Resolution S32 | Phase |
|---|---|---|---|---|
| P2-REVIEW-C-1 | rusqlite 0.32→0.36 + arti dep | **1/3** | Upgrade + activation | B |
| P2-AUDIT-1 | Executor silent param drops | NEW | Wire max_tokens | C |
| P2-AUDIT-2 | HARDENING compteurs stale | NEW | Fix doc | C |
| P2-REVIEW-B-1-S30 | Playwright COEP iframe test | **2/3** | Tentative | C |
| P3-AUDIT-1 | tor feature gate compile trap | NEW | Resolu par B | B |
| P3-AUDIT-2 | FROST HTTP error path tests | NEW | Tests | C |
| P3-AUDIT-3 | Tor boot log misleading | NEW | Fix log | C |
| LT-6 | iroh 0.98 upgrade (trigger met) | scheduled | **INTEGRE** | A |

### Items differes S33+

| ID | Description | Reports apres S32 | Sprint cible | Justification |
|---|---|---|---|---|
| P2-REVIEW-A-1 | LOC plan meta-process | 2/3 | S33 | Discipline plan-writing, pas d'action code. **3/3 → MANDATORY S34** |
| P3-grammar | Executor grammar field wire | NEW P3 | S33+ | Ollama ne supporte pas GBNF natif |
| P3-watermark | Executor watermark_config wire | NEW P3 | S33+ | Defense-in-depth, SynthID inject worker-side |

Note P2-REVIEW-B-1-S30 : si **non resolu** Phase C, passe 3/3 S33
= **MANDATORY** per §6.2.1 Regle 2. Exemption possible si blocker
externe (env Playwright instable) est document avec evidence
renouvelee.

### Phase dette S32 (§6.2.1 Regle 1)

S32 est pair — phase dette **obligatoire**. Phase A est la phase
dette dedie : iroh 0.98 upgrade (LT-6 trigger met, le plus gros
item differe du backlog). Phase B (rusqlite + arti) est aussi de
la dette.

### Items long-terme (ROADMAP_COMMITMENTS)

| ID | Condition | Status |
|---|---|---|
| LT-1 | v1.0 + design doc + Gini > 0.70 | Latent |
| LT-2 | tag v1.0 | Latent |
| LT-3 | v1.0 + 3+ contrib non-compute | Latent |
| LT-4 | v1.0 + N1 FROST + partnership | Latent |
| LT-5 | multi-worker deploy OR v1.0 | Latent |
| LT-6 | iroh > 0.97 OR v1.0 | **RESOLVING S32** Phase A |

---

## §7 Scope cuts

Ce que S32 ne fait PAS :

1. **iroh relay over Tor** — scope-cut S33+ (iroh 0.98 n'ajoute
   pas de proxy config Endpoint, meme situation que 0.97)
2. **Nym mixnet phase 1** — re-defere S33+ (SDK paused crates.io)
3. **TEE H100 attestation** — scope-cut (pas hardware partenaire)
4. **DKG distribue FROST** — post-v1.0 (trusted dealer suffisant N=3)
5. **Recrutement mainteneurs** — ops post-v1.0
6. **Onion service hosting** — post phase 1 Tor
7. **Full process isolation blob-serve** — LT rewrite architectural
8. **openai-agents-python upgrade** — pas de dep directe SBFB
9. **llama.cpp executor support** — S33+ si demande
10. **Output filter client-side (iframe defense-in-depth)** — S34
11. **iroh 1.0 wait** — pas de release publiee, 0.98 est le cible
12. **rusqlite 0.39 (latest)** — 0.36 suffisant, delta minimal

---

## §8 Tracabilite scope

Table mappant les items S31 "What's NOT" sur leur traitement S32 :

| Item S31 scope-cut | Sprint + Phase S32 | Status |
|---|---|---|
| iroh 0.98 upgrade | S32 Phase A | **INTEGRE** (LT-6) |
| iroh relay over Tor | S33+ | SCOPE-CUT (meme raison 0.97→0.98) |
| Nym mixnet phase 1 | S33+ | RE-DEFERE |
| TEE H100 attestation | post-v1.0 | SCOPE-CUT |
| DKG distribue FROST | post-v1.0 | SCOPE-CUT |
| Playwright COEP iframe test | S32 Phase C (tentative) | INTEGRE |
| Onion service hosting | post phase 1 | SCOPE-CUT |
| Full process isolation | LT | SCOPE-CUT |
| openai-agents-python | informationnel | INCHANGE |
| llama.cpp executor | S33+ | SCOPE-CUT |
| Output filter client-side | S34 | SCOPE-CUT |
| Carries P2-AUDIT-1/2 | S32 Phase C | **INTEGRE** |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | iroh 0.98 breaking changes touchent plus de fichiers que prevu | Medium | Medium | Explore agent a cartographie 6 fichiers dans nexus-core-rs. Shell-daemon-core et worker-core consomment via abstractions. Impact localise. |
| R2 | iroh-blobs 0.99→0.100 casse le ticket serialization wire | Low | High | BlobTicket est un format iroh-interne. SBFB ne persiste pas de tickets cross-version. Si format change, les vieux tickets sont invalides mais aucun noeud tiers ne les a (pre-launch). |
| R3 | arti-client 0.41 pulls ~15-20 deps transitives, build time +30s | Medium | Low | Feature-gated `tor`, CI compile sans `tor` par defaut. Seul `cargo build --features tor` paie le cout. |
| R4 | rusqlite 0.32→0.36 casse schema migration pattern | Low | Medium | rusqlite_migration 1.3 fonctionne avec rusqlite 0.32-0.39. API bundled inchangee. Tests SQLite existants (quarantine, trust cache, allowlist) couvrent le path. |
| R5 | Playwright env instable empeche resolution P2-REVIEW-B-1-S30 | High | Low | Si env fail, documenter exemption et MANDATORY S33. Le Playwright COEP test est un test de regression isolation, pas un feature blocker. |
| R6 | iroh-docs 0.98 change Doc/Author API | Medium | Medium | nexus-core-rs/src/docs.rs ~300 LOC. Les types principaux (Doc, Author, LiveEvent) sont stables dans l'API iroh-docs. |

---

## §10 Audit gate pattern — rappel

Phase 0 audit S31 **jouee** — verdict PASS, commit `1cc4734`.
Phase D produira :
- `sprint32_verification.md` (self-report fail-fast)
- `sprint33_audit_plan.md` (plan pour S33 Phase 0)
- `sprint32_carry_summary.md`

---

## §11 Checkpoint de validation

5 questions pour arbitrage user AVANT le plan detaille :

1. **D1 iroh upgrade** : 4 crates simultanement (iroh 0.98, docs 0.98,
   gossip 0.98, blobs 0.100). Pas d'alternative (deps liees). Day 0
   #3 leve. Suffisant ?
2. **D2 rusqlite 0.36** : minimum suffisant pour unbloc arti-client.
   Pas 0.39 (latest). Activation du feature `tor` avec vrais deps.
   Acceptable ?
3. **D3 max_tokens** : wire seul, grammar + watermark_config restent
   P3 carry. Fidelite partielle. OK ?
4. **D4 batch** : HARDENING compteurs + Tor log + FROST errors +
   Playwright tentative. Trop ou pas assez ?
5. **D5 pin leve** : iroh 0.97→0.98. Pas d'attente iroh 1.0. Day 0
   #3 reformulee comme "upgrade volontaire par sprint dedie". OK ?
