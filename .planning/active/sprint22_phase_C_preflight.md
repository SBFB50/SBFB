# Sprint 22 Phase C — preflight G8

Date : 2026-04-20
HEAD : `9676bd9`
Verdict : **EXECUTE plan-as-is**

Phase : Sybil-resistance composition 3 couches (Couches 1 + 2 live,
Couche 3 design-only). Plan §6 (lignes 268-546). ~950 Rust +
~150 Python + ~450 docs, delta tests +20 (+16 Rust + +4 Python
coord) vs baseline post-Phase B 666 Rust / 249+3 coord.

Mode sampling Step 3bis activé (>10 fichiers ciblés) : 3 groupes
rust-core / shell-daemon / python-coord, libs crypto + wire
format priorisées, `git log --max-count=100` par groupe.

---

## Scans

### S1 — SOTA 2026 vs design

Libs scannées (priorité crypto + wire + network-exposed) :

| Lib / spec | Pinned | Verdict | Evidence |
|---|---|---|---|
| `in-toto/attestation` v1.0 predicate | (spec ref, pas dep) | minor | v1.2 publiée backward-compat major (règle "same major → same semantics"). Champs `subject`/`predicateType`/`predicate` stables. https://github.com/in-toto/attestation/blob/main/spec/v1/README.md |
| `ed25519-dalek` | 2.1 workspace | clean | 0 advisory 2026 sur v2.x. RUSTSEC-2026-0075 = `libcrux-ed25519` (lib distincte, hors scope). https://rustsec.org/packages/ed25519-dalek.html |
| `toml` | 0.8 workspace | clean | 0.9 existe avec breaking API mais SBFB reste pinné 0.8. Pas de CVE. |
| `rusqlite` | 0.32 workspace | clean | 0 advisory 2025-2026 active. API `contributor_attestations` stable. https://rustsec.org/packages/rusqlite.html |
| `gossipsub-v1.1` spec (ref P₅ only) | (ref) | clean | Spec vivante, v1.2 additive (IDONTWANT). Référence prior-art P₅ application-specific valide. https://github.com/libp2p/specs/blob/master/pubsub/gossipsub/gossipsub-v1.1.md |
| Radicle Heartwood `did:key` (ref) | (ref) | clean | Radicle 1.8.0 "Drosera" released 2026-03-30, actif. `did:key` pattern référence valide pour node_id. https://radicle.xyz/2026/03/30/radicle-1.8.0 |

WebSearch CVE : `ed25519-dalek 2026`, `toml-rs 0.9 2026`, `rusqlite 2026` → 0 finding bloquant.

Context7 query : in-toto v1 spec confirmée stable majeure, breaking change post-2025-Q4 **absent**.

**Verdict S1 : clean** (1 minor in-toto v1.2 additive backward-compat major, non-bloquant).

### S2 — Décisions historiques traversées

Scan git log `--max-count=100` + archive v1.2/*.md + memory feedback.

Commits touchant fichiers modifiés Phase C avec mots-clés
"DEVIATION|rejected|scope-cut|deliberate|threat-model" :

| SHA | Message | Applicabilité Phase C |
|---|---|---|
| `04c9621` | Sprint 18 Phase E2 warrant canary monthly Ed25519 gossip publish (rejected auto-publish scheduler) | **NON-applicable** — ContributorAttestation est signée par coordinator volontaire au `/api/deploy` verified-deploy (S14), pas par scheduler temporel. Plan §6.1 Scan S2 l'acknowledge explicitement ligne 281-283. Reverse-commit check : décision `04c9621` est threat-model forbidding, **pas** une approche rejetée → pas de reversion requise. ✓ |
| `b0656ff` | Sprint 4 Phase C invite v2 hard bump + CLI + API | NON-applicable (module `invite`, pas `attestations`). |
| `1726dec` | Sprint 2 S5-S8 docs/gossip/blobs/discovery wrappers | NON-applicable (initialisation modules). |
| `94cccb2` | Sprint 18 Phase D coord-side TaskEntry wire + X-SBFB-Token rotation | NON-applicable (endpoint différent, pas `/api/deploy`). |
| `2c896a8` | Sprint 7 Phase A headless daemon HTTP skeleton | NON-applicable (initialisation `http.rs`). |

Archive scan `.planning/archive/v1.2/*.md` mots-clés
`sybil|attestation|age.witness|contributor|in-toto|provenance|gossip.join` + DEVIATION/rejected : 0 finding.

Memory `feedback_*.md` mots-clés avoid/reject + sybil/attestation :
0 finding.

**Verdict S2 : clean** — 5 commits touchant fichiers Phase C avec
mots-clés, tous non-applicables (module différent ou acknowledge
explicite plan §6.1). Reverse-commit check non-requis (pas
d'approche historiquement rejetée touchant Phase C).

### S3 — Threat model coverage

Threat matrix `docs/security/THREAT_MODEL.md` §7 + `HARDENING_ROADMAP.md §3 S22` + audit findings récents.

Mapping Phase C vs T0-T5 :

| Threat | Ligne HARDENING_ROADMAP | Couverture Phase C | Regression ? |
|---|---|---|---|
| **B-Sybil** identity flood | T2+ pre-S19 / T5 post-S19+S22 | **Bump T2→T3 mitigation partielle** (Couches 1+2 live). Couche 3 complet S27. | Non (bump conforme). |
| **B-GossipPoison** | T5 M, mitigation = B-Sybil (PoW pre-req) | Renforce PoW via age gate | Non. |
| **A-S12** Fake curator keypair theft | T5 H | Couche 2 extend `curator.rs` `verify_signature` check `ContributorRegistry` si flag gouvernance-forte | Non (additif). |
| **C-ResultSpoof** | redundancy-voting | **Out-of-scope Phase C** (deferred S22→S23 per kickoff §6). | Non (deferred documenté). |
| **C-PromptLeak** | ephemeral-workers+TEE | Out-of-scope Phase C (S25-S30). | Non. |

Pre-requirements HARDENING_ROADMAP §3 S22 ligne 251-264 pour Phase
C : **AUCUN manquant**. S19 PoW live (`edfc51b`), S14 Provenance
live (`95807b1`), S16 is_open_source live (`10bbc63`).

Audit findings récents : `sprint21_audit_findings.md` verdict
PASS 0 P0/P1. Reviews Phase A+B (`sprint22_phase_A_review.md` +
`sprint22_phase_B_review.md`) verdict PASS. 6 carries S22 audit_plan
trackés (P2-E-DURESS-ACK + P2-E-WIRE-PRE-LAUNCH-FIX + P3-E-2 JCS +
Meta-track hook Phase D + Phase A R1 toml.sample + Phase B drift
Playwright) — aucun ne bloque Phase C.

**Verdict S3 : clean** — Phase C bump B-Sybil T2→T3 cohérent
roadmap. Pas de régression introduite. HARDENING_ROADMAP pre-req
satisfaits.

### S4 — Wire format / pre-launch invariants

`_VERSION` fields pre-launch gelés (cf. CLAUDE.md "Pre-launch
protocol policy") :

| Marker | Valeur actuelle | Touchée Phase C ? |
|---|---|---|
| `BLOB_VERSION` | 0x01 | **Non** |
| `TASK_RESPONSE_VERSION` | 1 | Non |
| `CANARY_VERSION` | 1 | Non |
| `ANNOUNCEMENT_VERSION` | 1 | Non |
| `INVITE_VERSION` | 2 (S4) | Non |

DOMAIN tags existants `canonical.rs` (TASK/RESULT/CLAIM/INVITE/
KUDOS/CURATOR_LIST/PROVENANCE/WARRANT_CANARY/POW/DURESS_ACK) :
**inchangés**.

Nouveaux DOMAIN tags Phase C (additifs pre-launch stable) :

- `DOMAIN_AGE_WITNESS_V1 = b"nexus-age-witness-v1"` (Couche 1)
- `DOMAIN_CONTRIBUTOR_ATTESTATION_V1 = b"nexus-contributor-attestation-v1"` (Couche 2)
- `DOMAIN_DELEGATION_CERT_V1` — **design-only Couche 3 S22, PAS
  ajouté au code S22** (implémentation S23-S27).

Nouveaux structs : `AgeWitness`, `ContributorAttestation`,
`ContributorPredicate` — **pas de `#[serde(default)]` requis**
(nouveaux structs → pas de tolérance runtime JSON client Python
legacy à gérer). Plan ne mentionne aucun serde(default).

D1..D5 Day 0 du sprint 22 :

- **D1** composition 3 couches : Phase C **implémente** D1
  (kickoff §4 lignes 311-400) exactement comme arbitré user
  2026-04-19. Pas de rebattue.
- D2 γ hybride 6 phases : Phase C est item (c) dans D2, conforme.
- D3-D5 : non touchées par Phase C.

Decisions actées `nexus_grid_pivot.md §Decisions actées` :

- "public = open source repo_url" (S13) → compatible (Phase C
  extend ProvenanceRecord S14).
- "deploy verified from source Keyoxide + SLSA L1" (S14) → Phase
  C hook `ContributorRegistry.record(attestation)` dans `api/
  deploy.py` après `generate_provenance()` — cohérent.
- "pre-launch protocol policy" → respectée (nouveaux DOMAIN
  additifs, 0 bump `_VERSION`).

**Verdict S4 : clean** — 0 bump version, DOMAIN additifs
conformes pre-launch, D1..D5 préservées, decisions actées non
contredites.

---

## Synthèse

```
S1 = clean (1 minor in-toto v1.2 additive, non-bloquant)
S2 = clean (5 commits non-applicables, reverse-commit check non-requis)
S3 = clean (bump T2→T3 B-Sybil cohérent HARDENING_ROADMAP S22)
S4 = clean (0 bump _VERSION, 3 DOMAIN additifs, D1..D5 préservées)
```

**Verdict global : EXECUTE plan-as-is**

0 finding bloquant. 0 finding non-bloquant. Plan Phase C prêt à
implémenter tel quel.

## Action

Procéder écriture code Phase C per plan §6.2 (Couches 1 + 2 +
Couche 3 design-only RFC). Livrer :

1. **Pre-code obligatoire (P0-G1-2 ack)** :
   `docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md` (~200 LOC
   docs spec stable URI) AVANT 1re ligne code Rust.
2. **Couche 1** (~450 LOC Rust) : `bootstrap_allowlist.rs` +
   `attestations/mod.rs` + `attestations/age_witness.rs` +
   `canonical.rs` DOMAIN add + `gossip.rs` extend `join_topic_with_age_witness()` + PyO3 binding.
3. **Couche 2** (~450 LOC Rust + ~150 LOC Python) :
   `attestations/contributor.rs` + `curator.rs` extend verify +
   TODO comment LT-1 Matthew (P2-G1-3 ack) + PyO3 binding + coord
   `contributor_registry.py` + `api/deploy.py` hook + `http.rs` proxy.
4. **Couche 3** (~250 LOC docs) :
   `docs/security/CONTRIBUTOR_ATTESTATION_RFC.md` design-only S22,
   implem S23-S27.
5. **Tests** : +16 Rust + +4 Python coord (plan §6.3).

Critère acceptation plan §6.4 :
- `cargo nextest run --workspace` ≥ 682 tests
- `cargo clippy --workspace --all-targets -- -D warnings` 0
- `pytest packages/nexus-coordinator/tests/ -q` ≥ 253+ tests
- 2 docs créés (PREDICATE + RFC)
- `_VERSION` inchangés

Commit cible plan §6.5 :
```
feat(sprint22): Phase C — Sybil-resistance composition 3 couches (age witness + contributor attestation + Couche 3 RFC)
```

Aucun carry-over S+1 doc requis (SCOPE-CUT-CONSISTENT non-
déclenché).

## Garde-fous G8 §6.9 verified

- [x] Pivot evidence-based — N/A (verdict EXECUTE, pas pivot)
- [x] Day 0 respect — D1 implémenté conformément arbitrage user
  2026-04-19, D2 γ hybride intact, D3-D5 non touchées.
- [x] Wire format — 0 bump `*_VERSION`, DOMAIN additifs conformes
  pre-launch policy.
- [x] Test budget — +20 tests vs cap phase ~30 plan S22 (sous cap
  2.5x).
- [x] Thème sprint — Sybil composition 3 couches = thème central
  kickoff §1.
- [x] Pas YAGNI — Couche 3 design-only S22 a consommateurs clairs
  S23-S27 (multi-forge cross-validate + trust-web Amnesty).
- [x] Retrospective trackée — dimension G8 traceability meta-track
  `sprint22_audit_plan` (déjà établi Phase A/B preflight trace).

## Refs

- Plan : `.planning/active/sprint22_plan.md` §6 lignes 268-546
- Kickoff D1 : `.planning/active/sprint22_kickoff.md` §4 lignes 309-400
- HARDENING_ROADMAP §3 S22 : `docs/security/HARDENING_ROADMAP.md`
- Pre-launch policy : `CLAUDE.md` §"Pre-launch protocol policy"
- G8 source-of-truth : `docs/claude/README.md §6.9`
- Design Review : `.planning/active/sprint22_design_review.md`
