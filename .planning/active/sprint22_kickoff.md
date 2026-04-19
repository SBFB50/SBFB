# Sprint 22 — Kickoff (Sybil-resistance composition 3 couches + rate-limit engine wire + GLiNER span-decoder + NVML baseline + watermark canari primitive)

**Écrit** : 2026-04-19 (session fraîche post-audit gate S21 `96a953b`).
**Type** : **sprint implementation** (consolide Gate 2 via Sybil
admission composée remplaçant « kudos-weighted » flag FAIRNESS +
débloquant C-ComputeTheft foundation S24 + wire les debts de
primitives S21 non-câblées en prod).
**Tip master d'entrée** : `96a953b` (chore(sprint21): audit gate
S21 — findings verdict PASS, no blocking fix).
**Phase 0 audit Sprint 21** : **DÉJÀ JOUÉ** — findings dans
`.planning/archive/v1.2/sprint21_audit_findings.md` (verdict
**PASS**, 0 P0 + 0 P1 + 6 P2 documentés dont 3 tracés audit_plan
S22 + 3 nouveaux + 4 P3 cosmétiques). Migré vers `archive/v1.2/`
en Phase F wrap-up S21 (`7887471`) + dernier findings migré dans
ce commit d'ouverture S22.

---

## Sources context7 + WebSearch consultées (pré-gel D1..D5)

**Deep research G2 lancée 2026-04-19** (6 agents parallèles,
~40+ sources académiques + OSS vérifiées) avant figer D1..D5 ;
`last_validated: 2026-04-19` HARDENING_ROADMAP = fresh, aucun
trigger_revalidate activé sur nouvelles docs long-life.

### Sybil-resistance gossip admission (D1)

**Couche 1 (age + PoW) — prior art** :
- **libp2p gossipsub v1.1 + v1.2** (2024 + 2025) : score P₁..P₇
  canonique. Seul point d'intégration node-age = `P₅ application-
  specific`. Filecoin Lotus ne code PAS `node_age` en P₅. Ethereum
  consensus-specs PR #2665 redacted tuning. [gossipsub-v1.1
  spec](https://github.com/libp2p/specs/blob/master/pubsub/gossipsub/gossipsub-v1.1.md).
- **Tor Guard flag** : seul age-gate production P2P déployé (day-8
  eligibility, WFU uptime, `guard-tk` consensus format) — mais
  **requiert dirauths centralisés**. [Tor consensus-formats](
  https://spec.torproject.org/dir-spec/consensus-formats.html).
- **Nostr NIP-13** : seul PoW gossip production, **brisé** à
  ≤ 20 bits (GPU miner OpenCL trivial). [nips.nostr.com/13](
  https://nips.nostr.com/13). ACM 2025 empirical analysis.
- **IEEE S&P 2024 formal analysis gossipsub** (arXiv 2212.05197)
  : prouve que score-based mesh admission peut être évadé par
  behaviour-faking adversaries. **Résultat négatif fort** contre
  toute approche pure score.
- **iroh 0.97 NodeId** : Ed25519 32 bytes, **pas de timestamp
  intrinsèque protocole**. Any node_age signal doit être attesté
  externally (peer-attestation ou coord ledger). [docs.rs/iroh](
  https://docs.rs/iroh/0.97.0/iroh/).
- **S19 Phase B `08f4e41` + `edfc51b`** : PoW Hashcash 2^18
  default gossip subscribe déjà live. Wire-point
  `crates/nexus-core-rs/src/gossip.rs:140-162` `join_topic()`.

**Couche 2 (ContributorAttestation extend ProvenanceRecord)** :
- **S14 `95807b1` verified-deploy-from-repo** : `ProvenanceRecord`
  signé Ed25519 déjà en prod, `DOMAIN_PROVENANCE_V1`, champs
  `repo_url` + `commit_sha` + `artifact_hash` BLAKE3 + `node_id`
  + `timestamp`. [provenance.py:48-101](
  packages/nexus-coordinator/src/nexus_coordinator/provenance.py).
- **in-toto attestation framework** (2024) : pattern predicate
  extensible (`provenance`, `vsa`, `vulns`, `sbom`). Nouveau
  predicate `contributor-attestation` compatible. [in-toto spec](
  https://github.com/in-toto/attestation).
- **Sigstore/Fulcio** : attestation OIDC Ed25519/ECDSA centralisé
  (Fulcio root Google-run) — pattern référence mais pas
  réutilisable charte P2P.
- **CuratorListEntry v1** : wire format signé Ed25519 avec rollback
  protection déjà en prod (S10), zéro check contribution. Wire-
  point extension `crates/nexus-core-rs/src/curator.rs:252-274`.

**Couche 3 (multi-forge décentralisé, design-only S22)** :
- **Radicle Heartwood 1.8.0 "Drosera"** (2026-03-30) : Ed25519
  `did:key` natif = compatible `node_id` SBFB. Vuln replay corrigée
  1.8.0, **pas de binaire Windows**, adoption niche. [radicle.xyz
  1.8.0](https://radicle.xyz/2026/03/30/radicle-1.8.0).
- **Git SSH signing Ed25519 + `allowed_signers`** (git 2.34+,
  2021) : **pattern universel 2026** adopté GitHub/GitLab/Codeberg/
  Forgejo/Gitea. `git log --show-signature` offline verify. [GitLab
  SSH signing docs](https://docs.gitlab.com/user/project/repository/signed_commits/ssh/).
- **Codeberg (Forgejo-based)** : 300k repos, 200k users 2025-11,
  API REST publique sans auth sur repos publics, **mirror existant
  S18 Phase E3** via `docs/release/MIRROR_FALLBACK.md §3`.

**Rejetés factuellement** :
- **Human Passport (ex-Gitcoin)** acquisition Holonym fin 2024,
  `passport_weights.py` 56 stamps, **API key obligatoire, pas de
  self-hosting** — viole charte "No central server". [passport.
  human.tech](https://passport.human.tech/).
- **BrightID** : `BrightID-AntiSybil` dernière release **octobre
  2021** (4.5 ans dormant), ~100k users actifs (vs 12-16M
  Worldcoin), Passport eux-mêmes poids **0.202**. [BrightID KAIST
  report 2025](https://web3classdao.github.io/kaist2025/reports/brightid/).
- **Tangled (ATProto $4.5M seed 2026-03, 7k users / 5k repos)** :
  `did:plc` majoritaire, **secp256k1/P-256** incompatible Ed25519
  SBFB. [atproto.com/specs/did](https://atproto.com/specs/did).
- **nostr-git NIP-34** : draft, bus factor 1 (OpenSats grant),
  secp256k1 incompat. [nips.nostr.com/34](https://nips.nostr.com/34).
- **Forgejo federation ActivityPub** : **toujours experimental**
  2025-10 v13.0, issue #59 ouvert depuis années, seule feature
  mergée = "star a repo cross-instance". ForgeFed spec pas
  finalisée. [codeberg.org/forgejo/forgejo/issues/59](https://codeberg.org/forgejo/forgejo/issues/59).
- **GitHub OAuth seul** : biais structurel OECD (Stack Overflow
  Developer Survey 2024 n>65k) = ~90% population mondiale exclue.
  Incompat populations Gate 4 LibanLive-class (T5 dissidents) + UC 3
  Afrique (43% sans internet mobile) + UC 13 COVID mesh 2G.
- **Full voice-per-contribution binaire seul** : fake commits bon
  marché sans friction (LayerZero 13% Sybil sur 6M wallets,
  Gitcoin fraud tax historique), 0 précédent P2P production (Radicle
  = delegates pas vote, Tangled = spec absent).

### NVML compute theft baseline (D3)

- **`nvml-wrapper 0.12.1`** (2026-03-27) : NVML 12 + CUDA 13.0+1,
  API `device.utilization_rates()` + `running_compute_processes()`
  avec `last_seen_timestamp` param depuis **0.11.0 (2025-03-28)**
  = primitive delta exacte requise. [rust-nvml/nvml-wrapper
  CHANGELOG](https://github.com/rust-nvml/nvml-wrapper/blob/main/CHANGELOG.md).
- **NVIDIA DCGM exporter** (référence pattern Prometheus) : trop
  lourd pour baseline log-only. [github.com/NVIDIA/dcgm-exporter](
  https://github.com/NVIDIA/dcgm-exporter).
- **MagTracer 2023 ACM MobiCom** : magnétomètre hardware requis,
  non applicable soft-only baseline.
- **arXiv 2408.14554 (août 2024)** : behavior-based ML cryptojacking
  detection, ambition S24+ après baseline.
- **Précédent nexus-grid** : `nviwatch` TUI ~2000 LOC Rust pure
  nvml-wrapper pour full-feature → baseline headless log-only cible
  ~300-400 LOC.

### Watermark canari-input spot-check (D4)

- **Kirchenbauer 2023 ICML (arXiv 2301.10226)** green/red list
  watermark-output : **vulnérable BIRA attack** arXiv 2509.23019
  (septembre 2025) query-free bias-inversion. Pattern **rejeté**
  pour spot-check consumer.
- **Kirchenbauer impossibility eprint 2023/1776** : strong
  watermarking theoretical limit.
- **LLM-Canary open source** : security test suite, pas
  distributed. Référence pattern mais pas distributed canary LLM
  service.
- **Gap prior art académique** : aucun papier "distributed canary
  token LLM service" trouvé → opportunité contribution OSS
  nexus-grid. Pattern canari-input (consumer glisse 1/N prompt
  known-answer) distinct watermark-output algorithmique.
- **Pattern SBFB reuse** : signature Ed25519 sur known-answer
  hash + tolerance semantic similarity (pas exact match pour
  LLM température > 0).

### Rate-limit wire-up (Phase A P2-S21-1 + P2-S21-2)

- **`governor 0.10.2`** (crates.io 2025-11-13, MIT, GCRA,
  DashMap keyed) : déjà intégré S21 Phase A `63afe4e`, primitive
  `RateLimiter` live dans `crates/nexus-worker-core/src/
  rate_limit.rs` mais **non-câblée** au chemin critique engine
  (P2-S21-1 audit findings).
- **`tower-governor 0.8`** : axum 0.8 middleware — pattern
  documentation Context7 22 snippets [shuttle.dev blog](
  https://www.shuttle.dev/blog/2024/02/22/api-rate-limiting-rust).
- **`governor` Arc swap pattern** : hot-reload P2-S21-2 via
  reconstruction `DefaultKeyedRateLimiter` (pas de reset atomique
  natif per-clé).

### GLiNER span-logits decoder (Phase B P2-S21-3)

- **`@xenova/transformers.js` v4** + **`onnxruntime-web 1.24.3`** :
  tokenizer + inference runtime déjà chargés S21 Phase B `d5b0035`.
- **GLiNER paper** (urchade, EMNLP 2023) output format : tensor
  scores par (span, entity-type) pair → sigmoid decode + greedy
  dedup + threshold. [github.com/urchade/GLiNER](https://github.com/urchade/GLiNER).
- **GLiNER.js** npm : dernière release mars 2025, référence
  algorithme décoder (TS ~1500-2000 LOC — nexus-grid fork
  minimal ~350 LOC ciblé).
- **Bandarra blog PyTorch→browser** (pattern E2E ONNX + Transformers.js)
  [bandarra.me](https://bandarra.me/posts/from-pytorch-to-browser-a-full-client-side-solution-with-onnx-and-transformers-js).

**Sources full listées** : cf. `.planning/research/S22_*.md`
(archive `chore(research): sprint22` post-kickoff).

---

## 1. But du Sprint (§2 goal G3)

**But SMART (goal-backward)** : à la clôture Phase F, la **fail-
fast checklist `sprint22_verification.md §Fail-fast` verte** (30+
rows exécutables, projetée depuis le plan §11) valide que :

1. **Rate-limit livré S21 devient effectif en prod** : `RateLimiter
   ::check()` appelé par engine worker avant `ClaimEntry` broadcast,
   hot-reload policy observable via mutation `~/.sbfb/rate_limit_
   policy.toml` (P2-S21-1 + P2-S21-2 résolus).
2. **PII iframe SDK livrable fonctionnel** : `OnnxModelHandle.
   detect(text)` retourne des spans GLiNER (pas `[]`), F1
   projection ≥ 0.75 sur fixture test (P2-S21-3 résolu).
3. **Sybil-resistance admission composée Couches 1+2 live** :
   - Couche 1 : `node_id_age ≥ 7j` + PoW S19 vérifié au
     `join_topic()` gossip. Witness peer-attestation format défini.
   - Couche 2 : `ContributorAttestation` predicate in-toto
     compatible signé Ed25519 par coordinator. Check `is_verified_
     contributor(project_id, pubkey)` wire au `CuratorListEntry::
     verify_signature()` + coord Python registry persist.
   - Couche 3 : RFC `SBFB.json::contributions[]` extension +
     delegation cert format documenté (pas de code S22).
4. **NVML baseline log-only foundation S24** : `nexus-worker-core::
   nvml_profile` persiste util + durée + `last_seen_timestamp` stats
   locales. Pas d'anomaly detection (foundation only).
5. **Watermark canari-input primitive** : consumer endpoint
   `/canary/inject` + curator endpoint `/canary/observed-divergence`
   + pattern known-answer Ed25519-signed rotatable.
6. **Process fixes** : P2-S21-4 règle README §4.X Phase F parse
   review files → audit_plan, P2-S21-5 GHA CI cross-check commit→
   review file + `.claude/.bypass_audit_trail.log` trace.

**Gate unlock** : Gate 2 (TransLingua, FamilyScan, EHPAD-Lien
apps T0-T2 PII-light) effectivement débloqué à la clôture S22.
Ship-blocker encryption at rest S20 + rate-limit S21 effectif +
Sybil base S22 + supply chain S18 tous livrés.

**Sprint 23 non bloqué** : carry-over findings et items deferrés
(redundancy voting + Couche 3 implem partielle) documentés dans
`sprint22_carry_summary.md` + `sprint22_audit_plan.md`.

---

## 2. État vérifié à l'entrée

### 2.1 Tip master + compteurs tests

- HEAD : `96a953b chore(sprint21): audit gate S21 — findings
  (verdict PASS, no blocking fix)` (2026-04-19).
- Range S21 audité : `b34d451..7887471` (18 commits + tooling
  G4/G8 chores inclus), verdict PASS.
- Compteurs tests entrée S22 (réplication `sprint21_verification.
  md §2` finals post-Phase F) :

| Suite | Count observé S22 entrée |
|---|---|
| Rust workspace nextest | **659** |
| Python SDK | 185 |
| Python coordinator | **249 + 3 skipped** |
| Python app-gov | 46 |
| Vitest unit | **256** |
| Playwright | 38 |
| size-limit | 7/7 |
| SPDX license hook | 246+ |
| **Total** | **~1436** |

**Delta S21 (livré) vs baseline S20** : **+65** (+17 Rust governor
+ canary JCS, +36 coord PII redactor + output filter + quarantine +
verify Ed25519 at ingest + 16 fix wheel-stale bonus Phase E, +15
Vitest PII iframe).

### 2.2 Clippy + lint + fmt

- `cargo clippy --workspace --all-targets --locked -- -D warnings`
  : 0 warning (vérifié S21 Phase F).
- `cargo fmt --all --check` : 0 diff.
- Frontend `npm run lint` + `npx tsc --noEmit` + `npm run build` +
  `npm run size` : tous verts S21 fin.
- Python `uv run ruff format --check packages/` + `uv run ruff
  check packages/` : tous verts S21 fin.

### 2.3 Audit gate S21 findings consolidé

Findings `sprint21_audit_findings.md §5` (migré archive) :

- **6 P2** : P2-S21-1 RateLimiter non-câblé, P2-S21-2 hot-reload
  incomplet (couplé -1), P2-S21-3 GLiNER scaffold, P2-S21-4
  review→audit_plan carry gap, P2-S21-5 hook coverage Phase D,
  P2-S21-6 HARDENING wording misleading.
- **4 P3** : burst_multiplier silent floor, hf transformers range
  ^4.0.0, build_canary JCS alignment, rate_limit_policy.toml.sample
  absent.

Tous résolus dans le scope S22 (cf. §4 D2 + §5 + §6).

### 2.4 Pre-launch protocol policy

- `BLOB_VERSION = 0x01`, `TASK_RESPONSE_VERSION = 1`,
  `CANARY_VERSION = 1`, `ANNOUNCEMENT_VERSION = 1` tous inchangés
  S22.
- **Nouveau wire format introduit S22** : `ContributorAttestation
  v1` (Couche 2) **en pre-launch policy** (pas de bump version,
  format stable redéfini jusqu'à v1.0 go-live). Documenté dans
  `sprint22_plan.md §Phase C wire invariants`.
- Aucun tolerant decoder multi-version introduit.

---

## 3. Phase 0 — Audit Sprint 21 (DÉJÀ JOUÉ — verdict PASS)

**Status** : JOUÉ session 2026-04-19, commit `96a953b chore(sprint21):
audit gate S21 — findings (verdict PASS, no blocking fix)`. Ne pas
rejouer. Cf. `sprint21_audit_findings.md` (migré archive/v1.2/ en
Phase F S21 + findings file migré dans ce commit d'ouverture S22).

**Commit stack du gate** :

```
96a953b chore(sprint21): audit gate S21 — findings (verdict PASS, no blocking fix)
7887471 chore(sprint21): Phase F — wrap-up + verification + audit plan S22 + migrate planning
```

Aucun `fix(sprint21): ...` requis (0 P0/P1). Les 6 P2 carry actifs
tracés dans `sprint22_carry_summary.md §1` (résolution phases S22
A/B/C/F) + 4 P3 opportunistes.

**Verdict final** : **PASS**. Sprint 22 Phase A non-bloqué.

---

## 4. Décisions Day 0 (D1..D5)

### D1 — Sybil-resistance admission gossip composée 3 couches

**Retenu** : composition non-exclusive 3 couches remplaçant
l'item §3 S22 ligne 251-264 roadmap original « Kudos-weighted
gossip admission » (flag FAIRNESS_VISION §7 design-conflict
explicite). Arbitrage utilisateur 2026-04-19 post-synthèse 6
agents research deep-dive (cf. sources §G2 ci-dessus).

#### Couche 1 — Réseau gossip (tout le monde) : age node_id ≥7j + PoW S19

**Réutilise** :
- `crates/nexus-core-rs/src/gossip.rs:140-162` `GossipClient::
  join_topic()` — PoW Hashcash 2^18 default live depuis S19 Phase B
  `edfc51b`.

**Ajoute** :
- `node_id_age_witness` Ed25519-signé : nouveau module
  `crates/nexus-core-rs/src/attestations/age_witness.rs` + PyO3
  binding.
- Format : `{node_id, first_seen_ts, witness_pubkey, witness_sig}`
  canonical JCS, domain `DOMAIN_AGE_WITNESS_V1`.
- Peer-attestation : ≥1 peer existant (node_id actif ≥30j connu du
  gossip) signe le witness. Évite faiblesse Tor Guard model
  (dirauths centralisés). Bootstrap initial = coordinator du
  publisher (self-witness au premier deploy-from-repo).
- Cap âge 7 jours (mirror Tor Guard 8d + EMA decay sur activité
  pour éviter Matthew temporel early-adopter bias).

**Adresse** : UC 1 (chantier), UC 3 (commerce P2P Afrique), UC 4
(pool juridique), UC 7-15 (Sybil réseau bootstrap invite-only
Gate 2).

**Rejetés** :
- **Tor Guard seul age-gate P2P production** : 8-day eligibility
  mais **requiert dirauths centralisés** — modèle incompatible
  charte "No central server".
- **Nostr NIP-13 PoW seul** : difficulté ≤ 20 bits empiriquement
  cassé par GPU miners OpenCL (ACM 2025 empirical analysis).
- **Pure PoW sans age** : formal impossibility IEEE S&P 2024 arXiv
  2212.05197 — score-based admission evadable via behaviour-faking.
- **Node_id timestamp intrinsèque iroh** : **pas disponible**
  protocole iroh 0.97 (confirmed docs.rs/iroh + iroh-gossip
  CHANGELOG). Attestation externally-signed requise.

#### Couche 2 — Apps gouvernance-forte (Gate 2+) : ContributorAttestation binaire extend ProvenanceRecord S14

**Réutilise** :
- `packages/nexus-coordinator/src/nexus_coordinator/provenance.py:
  48-101` `ProvenanceRecord` Ed25519 signé JCS canonical, domain
  `DOMAIN_PROVENANCE_V1` live depuis S14 `95807b1`.

**Extend** :
- Nouveau predicate in-toto compatible `contributor-attestation`
  (ajout au side des predicates standards SLSA `provenance`, `vsa`,
  `vulns`, `sbom` — pattern framework [in-toto/attestation](
  https://github.com/in-toto/attestation)).
- Structure : `{predicate_type: "nexus-grid/contributor-
  attestation/v1", subject: [{name: project_id, digest:
  {blake3: ...}}], predicate: {contributor_node_id, first_deploy_ts,
  commit_sha, repo_url, attestation_coord_sig}}`.
- Signé Ed25519 par le coordinator du publisher au moment du
  verified-deploy. Preuve « node_id X a déployé app A from commit C
  vérifié par coordinator Y ».

**Wire-point admission** :
- `crates/nexus-core-rs/src/curator.rs:252-274` `CuratorListEntry::
  verify_signature()` extend avec `is_verified_contributor(project_id,
  curator_pubkey)` check contre coord-side registry.
- Pattern voice-per-project binaire : ≥1 `ContributorAttestation`
  valide pour projet P → 1 voix gouvernance P (pas plus, pas
  kudos-pondéré — découple complètement volume contribution de
  voix politique).

**Adresse** : UC 2 hôpital souverain, UC 5 école sans bande
passante, UC 6 jeu monde persistent, UC 13 entraide COVID mesh
2G — use cases gouvernance-forte par-app où la contribution
irremplaçable n'est PAS du débit mais de la présence.

**Rejetés** :
- **Human Passport (ex-Gitcoin) API centralisée** : acquis
  Holonym fin 2024, API key obligatoire, pas self-host, dépend
  Ceramic Network — viole charte "No central server".
- **BrightID** : AntiSybil dormant oct 2021, Passport poids 0.202,
  usability "tedious + unclear" (arXiv 2502.16375).
- **GitHub OAuth pur** : biais OECD Stack Overflow Survey 2024
  (~90% population mondiale exclue), incompat Gate 4 T5
  LibanLive-class.

#### Couche 3 — Multi-forge décentralisé (Gate 3+ horizon S23-S27) : DESIGN-ONLY S22

**Retenu design-only S22** (pas de code Phase C) :
- RFC interne `SBFB.json::contributions[]` extension format.
- Delegation cert Ed25519 : node_id SBFB signe cert
  `{ssh_pubkey: "SHA256:xyz...", valid_until: ts, node_id_sig}`.
- Parser `git log --show-signature --pretty=format:"%H %G? %GS %GK %ai"`
  offline (Git 2.34+ SSH signing pattern adopté GitHub/GitLab/
  Codeberg/Forgejo/Gitea).
- Pattern cross-forge : le même Ed25519 node_id peut signer commits
  sur Radicle + Codeberg + Forgejo + GH simultanément via delegation
  cert.

**Implémentation distribuée S23-S27** :
- S23 : design doc finalisé + delegation cert format Rust struct.
- S25-S26 : parser `git log --show-signature` offline (~500 LOC
  Rust) + cache LRU SQLite (pattern `upload_queue.py` S19).
- S27 : **remplace** item ligne 371-372 HARDENING §3 S27 « Sybil
  kudos-weighted mature » (même flag FAIRNESS implicite) par
  Couche 3 mature : multi-forge cross-validate + trust-web Amnesty
  integration (~700 LOC).

**Rejetés** :
- **Radicle 1.8.0 intégration native S22** : pas de binaire
  Windows, vuln replay récente 1.8.0, adoption niche. Déferré
  S25-S26 post-Radicle mature.
- **Tangled ATProto seed $4.5M 2026-03** : secp256k1/P-256
  incompatible Ed25519 SBFB (refacto 2 clés + binding layer).
- **nostr-git NIP-34** : draft, bus factor 1, secp256k1 incompat.
- **Forgejo federation ActivityPub** : experimental 2025-10 v13.0,
  seule feature mergée = "star cross-instance".

**Niveau sécurité composition 3 couches** :
- **T2 criminal organisé** : mitigé (Couche 1 age+PoW +
  ContributorAttestation cost-of-contribution réel).
- **T3 corporate hostile** : partiellement mitigé (Couche 1 ralentit,
  Couche 2 force deploy-from-repo signé, mais ContributorAttestation
  farmable via N repos publics fake sans coût externe réel).
- **T3+ mitigation complète** : requiert Couche 3 mature S27 +
  audit externe Cure53/ToB S29 (Gate 3 unlock).
- **Conforme trajectoire HARDENING §3 S27-S29** : S22 Sybil base
  (T2) → S27 Sybil mature (T3) → S29 audit externe (Gate 3 unlock
  PolitiScan/NEXUS cold-case).

### D2 — Scope 6 phases γ hybride (LOC ~2400, tests delta +55/+65)

**Retenu** : option γ hybride recadré (arbitrage user post-
synthèse Agent research D2 + deep codebase analysis). Réaligne
item §3 S22 ligne 250-271 HARDENING avec réalité carries S21 P2 +
Gate 2 blocking criteria factuels.

**Items §3 S22 roadmap absorbés dans S22** :
- Item 1 ligne 251 « Kudos-weighted gossip admission » → **Phase C
  D1 composition 3 couches** (remplace flag FAIRNESS).
- Item 2 ligne 265 « NVML util + duree profile log-only baseline »
  → **Phase D scope-réduit stats-only** (foundation S24).
- Item 5 ligne 270 « Spot-check watermark canari » → **Phase E
  primitive-only** (distinct watermark-output Kirchenbauer
  vulnérable BIRA 2025).

**Items §3 S22 roadmap deferrés S23** (chore planning dédié
`HARDENING §3 S22/S23/S24` update) :
- Item 3 ligne 267 « Sandbox tool-calling allow-list strict + dry-
  run » → **deferré post-S25** (pas de surface tool-call live,
  seul S20 structured output existe, OWASP LLM06:2025 Excessive
  Agency ne se déclenche pas sans tool-registry ouvert).
- Item 4 ligne 268-269 « Redundancy voting Task.redundancy_factor
  (3 workers majority) » → **co-deferré S22→S23 + S24 dependency
  update ligne 311**. Justification : mitigue C-ResultSpoof tier
  **T5** (§1 threat matrix) — surdimensionné 3 gates au-dessus du
  Gate 2 cible (T0-T2). BOINC/F@H ont opéré 1-worker production
  20 ans. Gate 3 track explicite §7 ligne 554.

**Items absorbés wire-up debts S21** (pas carry-overs formels
G7 — wire-up primitive livrée, pattern `dispatcher.py §16 validator
post-task`, précédent S20 Phase C `16b94ba` PoW runtime wire carry
S19 A-2 absorbé) :
- **Phase A** : P2-S21-1 RateLimiter wire engine + P2-S21-2
  hot-reload + P2-S21-6 HARDENING §3 S21 wording fix.
- **Phase B** : P2-S21-3 GLiNER span-logits decoder.

**Process fixes** (Phase F) : P2-S21-4 règle README §4.X Phase F
parse review → audit_plan + P2-S21-5 GHA CI cross-check + hook
coverage audit trail log.

### D3 — NVML baseline log-only stats-only (foundation S24)

**Retenu** : `nvml-wrapper 0.12.1` (2026-03-27) + primitive
`last_seen_timestamp` depuis 0.11.0.
- Module nouveau `crates/nexus-worker-core/src/nvml_profile.rs`
  (~300 LOC).
- Log stats periodic (util + duree + VRAM + `last_seen_timestamp`
  delta) persist SQLite locale `~/.sbfb/nvml_profile.db`.
- **Pas d'anomaly detection** (foundation only, feeds S24 random
  re-run sampling + auto-report curator divergence).

**Rejetés** :
- **DCGM exporter** : trop lourd pour baseline, Prometheus-stack
  déploiement requis.
- **MagTracer ACM MobiCom 2023** : magnétomètre hardware requis.
- **arXiv 2408.14554 behavior-based ML** : S24+ après baseline.

### D4 — Watermark canari-input spot-check consumer 1/N primitive

**Retenu** : primitive only (pas de backend ML, pas de signature
complexe).
- Consumer endpoint `/canary/inject` : chaque N tasks (configurable
  1/100 default), insère prompt known-answer signé Ed25519 coord.
- Curator endpoint `/canary/observed-divergence` : rapport si
  divergence semantic similarity < threshold configurable
  (pattern S21 Phase C EED Levenshtein 0.85 reuse).
- Nouveau `packages/nexus-coordinator/src/nexus_coordinator/
  canary_input.py` (~300 Python).
- Rotatable : set of known-answer prompts signé, renouvelable via
  CLI `nexus-coordinator canary-rotate` (pattern S18 Phase D
  TokenRotator file-watcher reuse).

**Rejetés** :
- **Kirchenbauer 2023 green-list watermark-output** (ICML) :
  vulnérable BIRA attack 2025 arXiv 2509.23019.
- **LLM-Canary suite** : pas distributed.
- **Portkey canary-deployment terminology** : overloaded (deployment
  != prompt-probe).

### D5 — Carries G7 cap 1/2 + LT-2 reclassification + Meta-tracks

#### Cap G7 — 1/2 slot utilisé (distinction workflow G7 cap)

**Slot 1 — T-NN+2 iframe Rust-wasm (hors cap formel PATTERNS §P34)** :
- Ouvert S21 Phase E `49f0d32` fermeture tech debt batch.
- Triggers re-activation : tract opset 19 coverage OR ort
  wasm32-browser stable release OR gline-rs wasm-bindgen target OR
  NVIDIA Open Model License clarification vs AGPL-3.0.
- Hors cap formel G7 (PATTERNS tech debt tracking, pas
  carry-over).

**Slot 2 LIBRE** (pour audit findings carry-overs Phase F S22).

#### Wire-up debts S21 absorbés en phases dédiées S22 — PAS carry-overs formels G7

Distinction workflow `docs/claude/README.md §6.2.1` :
- **Carry-over formel G7** : item non-livré fonctionnellement que
  l'on porte au sprint suivant en re-engagement explicite.
- **Wire-up debt** : fonctionnalité livrée mais primitive non-câblée
  au chemin critique runtime. Absorption en phase dédiée S+1 est un
  **completement** pas un re-engagement. Précédent S20 Phase C
  `16b94ba` PoW runtime wire carry S19 A-2 absorbé sans compter
  slot G7.

P2-S21-1 (rate-limit engine wire) + P2-S21-2 (hot-reload) +
P2-S21-3 (GLiNER decoder) = **3 wire-up debts** absorbés Phase A/B
S22 **sans consommer slot G7**.

#### LT-2 Meta-1 Radicle-v1.0 activation tracking — RECLASSIFICATION

**Contexte** : Meta-1 Radicle-v1.0 re-carry S18→S19→S20→S21→S22 =
**5e sprint consécutif**. `ROADMAP_COMMITMENTS.md §6.2.1` stipule
« carry-over present dans 3 carry_summary consecutifs = promu
long-term commitment en Phase F wrap-up du sprint N+2 et sort du
cap G7 ». Reclassification LT-2 **attendue en Phase F S20** mais
oubliée (et re-carry S21 également).

**Rattrapage S22** : régularisation au kickoff S22 via `docs/
release/ROADMAP_COMMITMENTS.md §LT-2` nouvelle section détaillée
(réservation existante §Reservation IDs futurs ligne 108-115 lève
l'ambiguïté d'allocation). Condition de déclenchement = tag `v1.0`
go-live posé sur master (runbook `MIRROR_FALLBACK.md §3 "Flip
sequence Codeberg → Radicle"`). Sort du cap G7 formel.

**Effet S22** : slot G7 disponible 1/2 (déjà noté), Meta-1 ne
consomme plus carry-over slot. Continuité tracking via registre
LT-2 revu annuellement ou post-trigger v1.0.

#### Meta-tracks S22

- **Meta-track G8 traceability** : 6/6 phases A-F doivent émettre
  `sprint22_phase_[A-F]_preflight.md` verdict EXECUTE / SCOPE-CUT-
  CONSISTENT / DESIGN-CONFLICT (Phase F trivial admet doc-only
  preflight).
- **Meta-track hook coverage** : GHA CI cross-check commit→review
  file ajouté Phase F (P2-S21-5 follow-up).
- **Meta-track HARDENING roadmap pivot S27** : documenter dans
  chore planning S22 opening que §3 S27 ligne 371-372 « Sybil
  kudos-weighted mature » pivote vers « Couche 3 mature (multi-forge
  cross-validate + trust-web Amnesty integration) » — même flag
  FAIRNESS implicite que S22 item 1 original.

---

## 5. Chore planning documents ouverture S22

Les chore planning suivants sont émis dans le **même commit
d'ouverture S22** que ce kickoff (cf. §7 references) :

1. **`docs/security/HARDENING_ROADMAP.md §3 S22`** : wording fix
   item 1 ligne 251-264 « Kudos-weighted gossip admission » →
   « Sybil-resistance admission composée 3 couches (age+PoW Couche 1
   réseau réutilise S19 + ContributorAttestation Couche 2 extend
   ProvenanceRecord S14 + Couche 3 design-only S22 / implem
   S23-S27) ». Fix P2-S21-6 ligne 18 audited_findings date bump
   `last_validated: 2026-04-19`.
2. **`docs/security/HARDENING_ROADMAP.md §3 S22`** : items 3 + 4
   deferrer annotation (sandbox tool-calling post-S25, redundancy
   voting S23).
3. **`docs/security/HARDENING_ROADMAP.md §3 S23`** : ajouter
   `Redundancy voting Task.redundancy_factor (3 workers majority)`
   (~400 LOC, carry S22). Scope-cut documenté : drop Exponential
   cooldown (redondant avec Couche 1 age gate) ou reporter Traffic
   padding design doc S28 (aligné Nym).
4. **`docs/security/HARDENING_ROADMAP.md §3 S24`** : dependency
   ligne 311 `S22 redundancy voting` → `S23 redundancy voting`.
5. **`docs/security/HARDENING_ROADMAP.md §3 S27`** : ligne 371-372
   « Sybil kudos-weighted mature » → « Couche 3 mature (multi-forge
   cross-validate + trust-web Amnesty integration, remplace kudos-
   weighted flag FAIRNESS) ».
6. **`docs/FAIRNESS_VISION.md §7`** : annotation note « Design-
   conflict S22 arbitré user 2026-04-19 — composition 3 couches
   retenue, flag S27 tracé pivot implicite ».
7. **`docs/release/ROADMAP_COMMITMENTS.md §LT-2`** : nouvelle
   section détaillée Meta-1 Radicle-v1.0 (rattrapage
   reclassification Phase F S20/S21 oublié).
8. **`docs/claude/SPRINT_LOG.md v1.2`** : row S22 ouverture avec
   thème + Gate 2 unlock target.

---

## 6. Risques identifiés (pre-gel)

| ID | Risque | Probabilité | Severity | Mitigation |
|---|---|---|---|---|
| R-S22-1 | Couche 1 `node_id_age_witness` greenfield (0 précédent OSS décentralisé) | MED | MED | Design doc Phase C prealable Phase C preflight G8 + G1 review agent indépendant (ce document) |
| R-S22-2 | Phase C budget 1050 LOC Rust+Python dépasse Phase S21 typique (~400-700 LOC) | MED | LOW | Réutilisation massive ProvenanceRecord S14 + gossip S19 existants ; split sous-tâches Phase C a/b/c |
| R-S22-3 | ContributorAttestation predicate non in-toto compatible | LOW | MED | Phase C preflight G8 S1 scan in-toto spec obligatoire pré-code |
| R-S22-4 | NVML Windows build (pattern S20 Phase A NASM deviation) | LOW | LOW | `nvml-wrapper 0.12.1` supporte Windows nativement, bench RTX 5080 Phase D |
| R-S22-5 | Watermark canari-input gap prior art = pattern nexus-grid-spécifique sans reference | MED | LOW | Primitive-only scope (pas ML, pas distributed consensus), opportunité OSS documented |
| R-S22-6 | G1 review finding P2+ post-gel requérant pivot | LOW | MED | G1 agent indépendant lancé pre-finalisation kickoff, scoring ⚠️/❌ acknowledgement obligatoire §4.5 |

---

## 7. References

- **Plan détaillé phases A-F + §11 tests projection** :
  `sprint22_plan.md` (émis dans le même commit d'ouverture S22).
- **Design Review Board G1** :
  `sprint22_design_review.md` (émis dans le même commit,
  acknowledgement ⚠️ findings §4.5 ci-dessous).
- **Carry summary** : `sprint22_carry_summary.md` (émis dans le
  même commit).
- **Research outputs** : `.planning/research/S22_*.md` (archivés
  post-kickoff par pattern S21 §6.11 rétroactif).
- **Sources primaires** : cf. §G2 ci-dessus (6 agents research +
  2 agents Explore codebase).

---

## 4.5 Acknowledgement G1 Design Review findings

G1 Design Review Board verdict **CONDITIONAL PASS** 2026-04-19
(rapport complet : `sprint22_design_review.md`). Rigor signal G4
satisfait : **5 findings** (3 P2 + 2 P3) + **2 items P0 pre-gel**
traités ici avant gel final D1..D5. Pattern §6.1.1 : planner reste
owner décision finale, chaque ⚠️/❌ acknowledgé avec décision
explicite.

### P0-G1-1 — Couche 1 bootstrap ceremony (chicken-and-egg)

**Finding G1** : `node_id_age_witness` Ed25519-signé par peer
existant = greenfield pattern sans précédent OSS. Premier node
bootstrap n'a pas de peer pour signer le witness. Tor Guard
(cité comme modèle) utilise dirauths centralisés — incompat charte.

**Décision planner** : **ACCEPT + mitigate in-phase Phase C
preflight G8**. Bootstrap ceremony spécifiée avant Phase C 1re
ligne code :

- **Choix technique** : `self-witness` pour les N premiers nodes
  marqués `bootstrap_phase` dans une `bootstrap_allowlist.toml`
  (maintenu par le publisher initial + PR communautaire, pattern
  seed list S18 multi-relai federation `bootstrap_peers`).
- **Cap** : `bootstrap_allowlist` ≤ 20 nodes, expire automatiquement
  via `expires_after: v1.0` (tag go-live trigger, aligné LT-2).
- **Verify logic** : `gossip.rs` `join_topic_with_age_witness()`
  accept self-witness SI node_id ∈ `bootstrap_allowlist`, sinon
  require peer-attestation ≥ 1 peer ancien.
- **Wire-point** : nouveau `crates/nexus-shell-daemon-core/src/
  bootstrap_allowlist.rs` + hot-reload pattern S20 `pow_policy_
  loader.rs`.

**Scope add Phase C** : ~100 LOC Rust supplémentaires (bootstrap
allowlist module + tests) → budget Phase C 950 Rust + 200 Python
(vs 850 initial). **Tolérable** dans enveloppe S22 ~2500 LOC totale.

### P0-G1-2 — Couche 2 in-toto predicate compatibility spec manquant

**Finding G1** : `ContributorAttestation` claimed "in-toto
compatible" mais aucune spec cite `predicateType` URI ni JSON
schema. Risk wire format drift vs SLSA provenance / VSA / SBOM
standards.

**Décision planner** : **ACCEPT + mitigate in-phase Phase C
preflight G8 S1 scan + livrable doc avant code**. 

**Livrable obligatoire avant 1re ligne code Phase C** :
- **`docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md`** (~200
  LOC docs) spécifiant :
  - `predicateType = "https://nexus-grid.org/contributor-
    attestation/v1"` (URI stable)
  - JSON schema draft-07 (pattern S20 Phase D
    `task_response.schema.json`)
  - Fields : `contributor_node_id` (hex), `first_deploy_ts`
    (int64 unix), `commit_sha` (string), `repo_url` (string),
    `attestation_coord_sig` (base64 Ed25519 64 bytes)
  - Envelope structure in-toto v1.0 (subject + predicateType +
    predicate)
  - Verification procedure vs in-toto lib (ou custom offline
    verifier réutilisant nexus_core::verify_bytes S14 pattern)
  - Exemple JSON minimal + multi-subject

**Pré-req G8 Phase C S1 scan** : fetch fresh spec in-toto v1.0
(https://github.com/in-toto/attestation) + vérifier pas de
breaking change post-2025-Q4.

### P2-G1-3 — Couche 2 FAIRNESS_VISION §7 replicates one level deeper

**Finding G1** : voice-per-project binaire évite kudos-weighting
gossip-layer mais **réplique conflit** un niveau plus profond :
workers high-kudos entrent plus de projets → voix cumulée
reconstituée.

**Décision planner** : **ACKNOWLEDGE + mitigate via LT-1 TODO
comment + design doc disclaimer**. 

- **Code comment obligatoire** Phase C dans `curator.rs::verify_
  with_contributor_registry()` et `contributor.rs::
  ContributorAttestation::build()` :
  ```rust
  // NOTE: Interim Sybil-resistance S22. Contributor selection
  // still biased toward high-kudos workers (Matthew effect
  // one layer deeper). Post-v1.0 LT-1 Kudos-v2 reform will
  // introduce log-utility + DRF + EMA trust to break this
  // cycle. See:
  // - docs/FAIRNESS_VISION.md §7 "Design-conflict S22"
  // - docs/release/ROADMAP_COMMITMENTS.md §LT-1
  ```
- **Design doc disclaimer** : ajouter section dans `docs/security/
  CONTRIBUTOR_ATTESTATION_PREDICATE.md §8 Limitations` :
  « Cette spec attest de la **contribution** mais pas de
  l'**équité distribution**. L'équité est gouvernée par LT-1
  post-v1.0 refonte kudos. »

**Trigger re-activation** : identique LT-1 (Gini > 0.70 OR top-5%
> 50% OR churn×hw correlation). Sortie du "interim" conditionnée
empirique, pas opinion.

### P2-G1-4 — Phase C budget 850 Rust décomposition manquante

**Finding G1** : budget 850 Rust + 200 Python sans décomposition
Couche 1 / Couche 2 / Couche 3 ni comparaison historique S16-S21
Phase C isolée.

**Décision planner** : **ACCEPT + G8 S1 preflight obligatoire
decomposition + Phase split si overflow**.

Décomposition initial (à valider preflight) :
- **Couche 1 age + bootstrap** : ~400 LOC (age_witness.rs 200 +
  bootstrap_allowlist.rs 100 + gossip.rs extend 50 + tests 50)
- **Couche 2 contributor attestation wire** : ~500 LOC (contributor.rs
  300 + curator.rs extend 50 + canonical.rs domain tags 20 +
  nexus-core-py PyO3 bindings 30 + coord Python registry.py 100)
- **Couche 3 RFC docs** : ~250 LOC docs (`CONTRIBUTOR_ATTESTATION_
  PREDICATE.md`)

**Total** : ~950 Rust + 100 Python + 250 docs (vs 850 Rust + 200
Python initial). Mise à jour §11 tests projection.

**Scope-cut rule** : si G8 S1 preflight découvre overflow >1200
Rust, **split Phase C → Phase C (Couche 1 + bootstrap) + Phase
C.1 (Couche 2 + predicate)** via chore planning mid-sprint.
Précédent : pattern scope-cut R1 S21 Phase A post-G8 drift.

### P2-G1-5 — Phase D watermark prior-art gap unconfirmed

**Finding G1** : assertion "gap prior art académique confirmé"
sans search result documenté. Risk design duplication si papers
exist.

**Décision planner** : **ACCEPT + search Phase E preflight
obligatoire**.

**Livrable Phase E preflight G8 S1 scan** :
- arXiv 2024-2026 search : `("canary" OR "watermark") AND
  ("distributed" OR "LLM service") AND ("prompt" OR "inference")`
- USENIX Security 2024-2026 + NDSS 2024-2026 venues
- Documentation résultats dans `sprint22_phase_E_preflight.md` :
  liste papers trouvés + disposition (match vs dismiss avec
  rationale factuel).

Si papers existants match le design → pivot G8 DESIGN-CONFLICT +
arbitrage user. Si gap confirmé → proceed EXECUTE.

### P3-G1-6 — Phase D NVML bench RTX 5080 hardware matrix

**Finding G1** : NVIDIA driver version minimum non confirmée, bench
RTX 5080 non cité.

**Décision planner** : **ACCEPT + Phase D preflight document
hardware matrix**.

**Livrable Phase D preflight** : matrix dans
`sprint22_phase_D_preflight.md` :
- NVIDIA driver version minimum (selon nvml-wrapper 0.12.1 CUDA
  13.0+1 requirement)
- Compatibility RTX 5080 confirmée via bench smoke test local
- Fallback graceful si GPU absent (`NvmlError::NotAvailable` pas
  panic)

**Pas de blocker S22** : Phase D teste avec `MockNvml` pattern CI
headless (ci tests ≠ bench production).

### P3-G1-7 — LT-2 reclassification timing regulatory

**Finding G1** : règle §6.2.1 auto-trigger Phase F sprint N+2.
Meta-1 Radicle = 4e consécutif en Phase F S21 (pas 5e). Rattrapage
kickoff S22 = régulier si S21 audit gate PASS.

**Décision planner** : **ACCEPT — S21 audit gate DÉJÀ PASS
(`96a953b` verdict PASS confirmé kickoff §3)**. LT-2 reclassification
automatique applicable = confirmed regulatory-compliant.

- Commit `96a953b chore(sprint21): audit gate S21 — findings
  (verdict PASS, no blocking fix)` acte le S21 closure Phase F
  audit pass.
- Rule §6.2.1 trigger met : reclassification Meta-1 → LT-2
  automatique au kickoff S22.
- `docs/release/ROADMAP_COMMITMENTS.md §LT-2` nouvelle section
  ajoutée dans le commit d'ouverture S22.

**Aucun revert D5 requis**. Slot G7 libre confirmé 1/2.

---

## 4.6 Impact net sur plan S22 post-G1

Récapitulatif ajustements vs draft initial §4 :

| Change | Phase | LOC Δ | Justification |
|---|---|---|---|
| + Bootstrap allowlist module | C | +100 Rust | P0-G1-1 ceremony |
| + `CONTRIBUTOR_ATTESTATION_PREDICATE.md` | C preflight | +200 docs | P0-G1-2 spec |
| + Code comment LT-1 TODO | C | 0 LOC (comments) | P2-G1-3 ack |
| Phase C split-rule reserve | C | si overflow >1200 | P2-G1-4 scope-cut |
| + arXiv search Phase E preflight | E preflight | +0 code (docs) | P2-G1-5 confirm |
| + NVML hardware matrix Phase D preflight | D preflight | +0 code (docs) | P3-G1-6 doc |

**Nouveau budget Phase C** : ~950 Rust + 100 Python + 250 docs
(vs 850 + 200 initial). Budget S22 total : ~2500 LOC (vs 2400
initial, +4%). Conforme roadmap §3 S22 nominal 2200 ±15%.

**Scope cuts préservés** :
- Items deferrés S23 inchangés (redundancy voting + traffic
  padding doc si besoin)
- Items deferrés post-S25 inchangés (sandbox tool-calling)
- LT-2 reclassification validée
- Cap G7 : toujours 1/2 slots utilisés + T-NN+2 hors cap

---

**Fin kickoff S22**. **Gel décisions D1..D5** validé post-G1
acknowledgement §4.5. Sprint 22 Phase A peut démarrer après
commit d'ouverture `chore(planning): open Sprint 22 — Sybil
composition 3 couches + rate-limit wire + GLiNER decoder + NVML
foundation + watermark canari + LT-2 reclassification + HARDENING
pivots §3 S22/S23/S24/S27`.
