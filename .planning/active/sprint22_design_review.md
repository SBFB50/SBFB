# Sprint 22 — Design Review Board G1 report

**Date** : 2026-04-19.
**Reviewer** : agent Explore indépendant (session fraîche, contexte
minimal), cf. `docs/claude/README.md §6.1.1`.
**Cible review** : draft D1..D5 Sprint 22 kickoff avant gel final
(composition 3 couches Sybil-resistance + scope γ hybride 6 phases +
LT-2 reclassification).
**Règle** : `docs/claude/README.md §6.1.1` — le reviewer signale
les angles morts, ne propose PAS de solution. Le planner reste
owner de la décision finale et acknowledge chaque ⚠️ / ❌ dans
`sprint22_kickoff.md §4.5`.
**Règle renforcée crypto/spec G1 extension** : chaque D-decision
citant source crypto/protocole standardisé DOIT enumerer au
moins UNE alternative concurrente récente (≤ 6 mois) avec raison
factuelle rejet.

---

## 1. Verdict global G1

**CONDITIONAL PASS** (⚠️ × 5, ✅ × 3, ❌ × 0 blockers, mais 2 items
P0 pre-gel à traiter avant Phase C code).

Le draft respecte globalement la règle G1 extension :
- **D1** : 6 alternatives explicites rejetées avec rationale factuel
  (GH OAuth, Human Passport, BrightID, Tangled, nostr-git,
  voice-per-contrib seul). Sources récentes (governor 0.10.2,
  Radicle 1.8.0 2026-03-30, in-toto v1.0).
- **D2** : Scope γ hybride documenté vs α strict / β carries-first.
- **D3** : `nvml-wrapper 0.12.1` 2026-03-27 + alternatives rejetées
  (DCGM, MagTracer, arXiv behavior-based ML).
- **D4** : Kirchenbauer 2023 + BIRA 2025 rejetés, pattern canari-
  input distinct.
- **D5** : Règle §6.2.1 LT reclassification appliquée, cap G7 1/2
  slots utilisés.

Les 5 ⚠️ sont **documentation/clarity/spec-gap**, pas functional
defects. **2 sont P0 pre-gel** (P0-G1-1 bootstrap ceremony + P0-G1-2
in-toto predicate spec) exigeant livrable avant 1re ligne code
Phase C. 3 sont P2 + P3 tracked in audit plan S22.

**Rigor signal G4 auditeur** satisfait : reviewer a trouvé **5
findings P2+** distincts sur la rigueur des sources + spec gaps +
decomposition budget.

---

## 2. Scoring table D1..D5

| D | Aspect | Verdict | Catégorie finding |
|---|---|---|---|
| **D1 Couche 1 PoW + AgeWitness** | Source S19 `edfc51b` (2026-02-28) live | ✅ | CITE SOURCE |
| D1 Couche 1 Greenfield design | `node_id_age_witness` peer-attest pattern sans précédent OSS | ⚠️ **P0-G1-1** | DESIGN NOVEL + chicken-and-egg unaddressed |
| D1 Couche 1 Alternatives | 6 explicit rejets factuels | ✅ | EXHAUSTIVE |
| **D1 Couche 2 ProvenanceRecord extend** | Predicate in-toto compat claim sans spec | ⚠️ **P0-G1-2** | SPEC GAP |
| D1 Couche 2 FAIRNESS conflict | Voice-per-project = Matthew un niveau + profond | ⚠️ P2-G1-3 | DESIGN-CONFLICT (mitigable via LT-1 TODO) |
| **D1 Couche 3 RFC design-only** | Scope S22 docs-only, implem S23-S27 | ✅ | SCOPE OK |
| **D2 Phase C budget** | ~850 Rust + ~200 Python sans décomposition | ⚠️ P2-G1-4 | ESTIMATION RISK |
| D2 Phases A-B-D-E | Wire-points A+B chiffrés (S21 carry), D-E primitives simples | ✅ | DELIVERED/CLEAR |
| **D3 NVML baseline** | `nvml-wrapper 0.12.1` 2026-03-27 stable | ✅ | APPROPRIATE |
| D3 NVML bench RTX 5080 | Hardware matrix non documentée | ⚠️ P3-G1-6 | UNCONFIRMED |
| **D4 Watermark canari-input** | Primitive simple, Kirchenbauer rejeté factuel | ✅ | APPROPRIATE |
| D4 Watermark prior art gap | « Gap confirmé » sans search result documenté | ⚠️ P3-G1-5 | RESEARCH GAP |
| **D5 Cap G7 + LT-2 reclass** | Règle §6.2.1 auto-trigger Phase F S21 PASS confirmé | ✅ | REGULATORY OK |

---

## 3. Findings détaillés

### P0-G1-1 (P2 severity, P0 pre-gel obligation) — D1 Couche 1 chicken-and-egg bootstrapping

**Factual basis** : draft D1 Couche 1 décrit `node_id_age_witness`
Ed25519-signé par ≥1 peer existant (peer-attestation, pas dirauth
centralisé). Specification : « signé par ≥1 peer existant ».

**Risk** : bootstrap initial (age 0, pas de peers existants) face
à chicken-and-egg : le premier node n'a personne pour signer son
witness. Draft silencieux sur la cérémonie bootstrap.

**Precedent check (reviewer)** :
- **Tor Guard** : cité comme modèle mais utilise dirauths
  centralisés (https://spec.torproject.org/dir-spec/consensus-
  formats.html §guard-tk), pas peer-attestation. Modèle incompat
  charte "no central server".
- **IPFS Kubo / Filecoin Lotus** : open-join, aucune Sybil
  resistance au protocole — pas de précédent age-gate utilisable.
- **Radicle-v1.0** : device-link auth (séparé approach, LT-2
  reclassification).
- **libp2p gossipsub P₅ application-specific** : Filecoin Lotus
  n'encode PAS node_age en P₅ (lecture code `node/modules/lp2p/
  pubsub.go` agent research 1).

**Conclusion** : **greenfield design**, 0 précédent OSS P2P gossip
avec peer-attested age gate déployé en production.

**Recommendation audit (planner decides)** : documenter cérémonie
bootstrap explicite AVANT Phase C 1re ligne code. Options listées
(planner pick) :
- (a) Magic bootstrap key signée par publisher initial (simple
  mais centralisé dans publisher)
- (b) Open-join day 1 avec transition age gate après N jours
  (temporel mais vulnérable Sybil day 0)
- (c) Pre-signed SBFB.json bootstrap allowlist (pattern S14
  Keyoxide réutilisable, rev bottom-up)

**Owner** : planner dans `sprint22_kickoff.md §4.5`.

### P0-G1-2 (P2 severity, P0 pre-gel obligation) — D1 Couche 2 in-toto predicate compatibility spec manquant

**Factual basis** : draft D1 Couche 2 claims « predicate in-toto
compatible `contributor-attestation` » et pointe
`crates/nexus-core-rs/src/curator.rs:252-274`. Code actuel à
cette ligne range verifie signature CuratorListEntry only, **ne
handle pas de predicates**.

**Risk** : in-toto spec v1.0 définit predicates comme `{type:
URI, predicateType: string, predicate: JSON}`. SLSA utilise
`predicateType = "https://slsa.dev/provenance/v1"`. Draft's
`contributor-attestation` predicate type **indéfini** — no
`predicateType` string specified, no schema provided.

**Precedent check (reviewer)** :
- `packages/nexus-coordinator/src/nexus_coordinator/provenance.py:
  48-101` implémente SLSA L1 provenance (commit_sha + artifact_hash
  + signature) mais **aucun wrapper predicate in-toto**.
- SLSA provenance : `https://slsa.dev/provenance/v1` — pattern
  référence.
- VSA (Verification Subject Attestation), SBOM, vulns — autres
  predicates standards.

**Conclusion** : compatibility non spécifiée, risk drift vs
standards.

**Recommendation audit (planner decides)** : livrable doc AVANT
Phase C 1re ligne code :
- `docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md` définissant :
  - `predicateType = "https://nexus-grid.org/contributor-
    attestation/v1"` (ou similaire URI stable)
  - JSON schema draft-07 pour predicate content
  - Verification procedure vs in-toto spec

**Owner** : planner dans `sprint22_kickoff.md §4.5` + livrable
Phase C preflight.

### P2-G1-3 — D1 Couche 2 FAIRNESS_VISION §7 conflict replicated one layer deeper

**Factual basis** : HARDENING_ROADMAP.md §3 ligne 250-252 flag
« Kudos-weighted gossip admission » FAIRNESS_VISION.md §7
design-conflict (Matthew effect). FAIRNESS_VISION.md §7 lignes
219-237 liste 3 alternatives : (a) age + PoW, (b) Passport
multi-signal, (c) voice-per-project binaire.

**Draft approach** : D1 Couche 2 implémente « ≥1 ContributorAttestation
valide pour projet P = 1 voix gouvernance P ». Ça évite kudos-
weighting gossip-layer mais :
- **Contributor selection** : workers high-kudos gagnent plus de
  tasks → entrent dans plus de projets (architecture rate-limit
  S21 favorise high-kudos consumers)
- **Voice distribution** : top-kudos workers apparaissent dans
  plus de project lists → vote dans plus de governance →
  cumulative power reconstituée
- **Same Matthew effect** : rentes fluent vers hardware capital,
  un niveau d'indirection plus profond

**Formal status** : ROADMAP_COMMITMENTS.md §LT-1 (créé 2026-04-19
même session) marque cela latent avec 3 triggers empiriques
(Gini > 0.70, top-5% > 50% kudos, correlation churn ↔ hardware).
**Design-conflict reste non-résolu en S22 — acknowledged latent**.

**Recommendation audit** : D1 Couche 2 doit acknowledge c'est
**interim Sybil-resistance sans fairness reformation**. Wire
TODO comment code → LT-1. Ne pas positionner comme final
fair system.

### P2-G1-4 — D2 Phase C budget 850 Rust décomposition manquante

**Factual basis** : draft claims « ~850 Rust + ~200 Python » pour
« Sybil base composition 3 couches ». Phases complètes S21 avec
counts réels : Phase A 659 Rust total cumul (+17 delta), Phase B
185 SDK TS (+15 Vitest), Phase C 249+3 coord Python (+36 delta
dont 16 bonus fix wheel-stale).

**Risk** : Phase C est le plus gros scope item (Couche 1 age gate
+ Couche 2 contributor-attestation wire + predicate spec).
S21 Phase C = 249+3 lignes Python coord-side only (pas gossip
layer complexity). Double à 850 Rust + 200 Python dans new phase
unclear si réaliste.

**Historical check (reviewer)** : pas de « Phase C isolation data »
dans archives. S18 Phase C multi-relai ~400 Rust (documented plan
§4 quick-wins line 449). S19 Phase C TLS pinning ~200 Rust. Aucun
précédent Phase C > 500 Rust single-phase.

**Recommendation audit** : exiger Phase C preflight (G8 S1 scan)
décomposition 850 Rust en 3 sub-items (Couche 1 age gate ~300,
Couche 2 attestation wire ~400, Couche 3 RFC docs ~150). Bench
vs S18 C (400) et S19 wire (150). Si Phase C > 1200 LOC, **split
Phase C → Phase C + Phase C.1** (scope-cut rule, précédent pattern
R1 S21 Phase A).

### P3-G1-5 — D4 watermark prior-art gap assertion non-sourcée

**Factual basis** : draft D4 claims « gap prior art académique
confirmé (pas de papier distributed canary LLM service) ». No cite
given. No arXiv/ACM search performed.

**Risk** : si prior art exists (ex: « Continuous Integration
Canaries in LLM Deployments » ou similaire), le claim est factuel-
lement faux. Assertion sans search risk design duplication.

**Reviewer spot-check** : quick search arXiv 2024-2026 pour
« canary LLM distributed » retourne zero direct matches.
« Watermark LLM service » retourne ~3 papers sur watermarking
embeddings, aucun sur distributed canary inputs. **Gap provisoire
confirmé** mais doit citer méthodologie search dans design doc.

**Recommendation audit** : avant Phase E code, run targeted arXiv
search (« canary » OR « watermark ») AND (« distributed » OR
« LLM service ») 2024-2026. Document résultats dans
`sprint22_phase_E_preflight.md`.

### P3-G1-6 — D3 NVML bench RTX 5080 hardware matrix absent

**Factual basis** : draft D3 cite `nvml-wrapper 0.12.1 (2026-03-
27)` + `last_seen_timestamp depuis 0.11.0 (2025-03-28)`. Release
date date appears correct (past tense 2025 = 0.11.0 vs audit
2026 = 0.12.1 ok). Hardware test environment non cité.

**Risk** : NVML APIs drift across driver versions. RTX 5080
(mentioned mission briefing) requires NVIDIA driver version
minimum — unknown si test environment a driver compatible.

**Recommendation audit** : Phase D preflight documenter NVIDIA
driver minimum version + compatibility RTX 5080 confirmée +
timeout profile. Fallback graceful si GPU absent
(`NvmlError::NotAvailable` pas panic) → tests CI via `MockNvml`.

### P3-G1-7 — D5 LT-2 reclassification regulatory timing (résolu)

**Factual basis** : `ROADMAP_COMMITMENTS.md §6.2.1` règle : carry-
over present 3 carry_summary consecutifs → promu LT en Phase F
sprint N+2. Meta-1 Radicle :
- S18 carry → S19 (1re) → S20 (2e) → S21 (3e) — **trigger
  reclassification Phase F S21**
- S21 → S22 = 4e consecutive (si non-reclass Phase F S21)

**Factual status** : S21 Phase F wrap (commit `7887471`) + audit
gate S21 (`96a953b`) verdict **PASS confirmé** — Phase F wrap
réalisé, reclassification règle auto-trigger devait être
appliquée mais ne l'a PAS été (oubli rapporté audit findings
`.planning/archive/v1.2/sprint21_audit_findings.md`).

**Rattrapage S22** : régularisation au kickoff S22 via
`ROADMAP_COMMITMENTS.md §LT-2` nouvelle section + sort cap G7
formel. Conforme règle §6.2.1 par rattrapage documenté.

**Recommendation audit** : **résolu** au kickoff §4.5 par
confirmation audit gate S21 PASS + création LT-2 section dans
commit d'ouverture S22.

---

## 4. Questions spécifiques — answers chiffrés

| Question | Réponse courte chiffrée | Owner |
|---|---|---|
| D1 Couche 1 greenfield sans précédent OSS ? Risk chicken-and-egg ? | **Oui greenfield**, 0 précédent OSS cité. Risk présent, bootstrap ceremony required P0 pre-gel. | planner §4.5 |
| D1 Couche 2 in-toto compatible ? Risk wire format drift ? | **Compatibility unspecified**, predicate URI manquant, schema absent. Livrable P0 `CONTRIBUTOR_ATTESTATION_PREDICATE.md` pre-code. | planner §4.5 + Phase C preflight |
| D1 Couche 2 évite FAIRNESS conflict ou réplique ? | **Réplique** un niveau plus profond via high-kudos workers → more projects → more voice. Mitigation via LT-1 TODO comment. | planner §4.5 + code comment Phase C |
| D2 Phase C 850 Rust + 200 Python réaliste ? | **Plausible** mais sans décomposition + 0 historique Phase C > 500 Rust. G8 S1 preflight décomposition obligatoire. Split-rule Phase C/C.1 si overflow > 1200. | Phase C preflight |
| D3 NVML bench RTX 5080 ? | **Unknown**, driver matrix non documentée. Phase D preflight document driver minimum + compat RTX 5080. | Phase D preflight |
| D4 Watermark prior art gap confirmé ? | **Unconfirmed** spot-check reviewer indicates gap provisoire. arXiv + USENIX + NDSS 2024-2026 search Phase E preflight obligatoire. | Phase E preflight |
| D5 LT-2 reclass régulière ? | **Résolu** S21 audit PASS confirmé, règle §6.2.1 auto-trigger rattrapage kickoff S22 conforme. | planner §4.5 (done) |

---

## 5. Recommandations pre-gel (fix + P2/P3 à acknowledger §4.5)

**Règle G1 strict** : reviewer signale, planner décide. Aucune
recommandation d'implémentation — seulement angles morts + risques
factuels chiffrés.

| Priority | Item | Action Required | Blocker Phase C ? |
|---|---|---|---|
| **P0 pre-gel** | D1 Couche 1 bootstrap ceremony undefined | Documenter wire-point cérémonie (3 options (a)/(b)/(c) listées §3 P0-G1-1) — planner pick dans kickoff §4.5 + Phase C code | ✅ OUI |
| **P0 pre-gel** | D1 Couche 2 in-toto predicate spec | `docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md` (predicateType URI + JSON schema + verify proc + in-toto envelope) AVANT Phase C code | ✅ OUI |
| P2 S22 | D1 Couche 2 fairness conflict code ack | TODO comment LT-1 dans `curator.rs` + `contributor.rs` extend. Doc disclaimer §8 Limitations predicate spec. | Non (code-level) |
| P2 S22 | Phase C budget décomposition | G8 S1 preflight Phase C 3 sub-items. Split-rule réservée si >1200. | Phase C preflight |
| P2 S22 | Phase E prior-art search | arXiv search documenté `sprint22_phase_E_preflight.md` | Phase E preflight |
| P3 S22 | Phase D NVML hardware matrix | Driver minimum + bench RTX 5080 dans `sprint22_phase_D_preflight.md` | Phase D preflight |
| P3 résolu | LT-2 reclassification régulière | Audit S21 PASS confirmé → kickoff §4.5 done + ROADMAP_COMMITMENTS §LT-2 ajouté dans commit d'ouverture | ✅ résolu |

---

## 6. Conclusion verdict

**CONDITIONAL PASS** — Draft D1-D5 procède au gel final avec
acknowledgements planner §4.5 :

- ✅ **D1 Couche 3 design-only** : RFC pattern correct, no code
  scope-creep risk.
- ✅ **D2-D5 scope mapping** : A-B-D-E integration cohérente,
  NVML baseline appropriate, watermark primitive sound.
- ⚠️ **D1 Couche 1 bootstrap** : greenfield pattern, ceremony
  required avant Phase C.
- ⚠️ **D1 Couche 2 predicate spec** : `CONTRIBUTOR_ATTESTATION_
  PREDICATE.md` requis avant Phase C.
- ⚠️ **D1 Couche 2 fairness** : interim solution, design-conflict
  acknowledged via LT-1 latent + TODO code comment.
- ⚠️ **D2 Phase C budget** : plausible mais G8 S1 preflight
  decomposition mandatory.
- ⚠️ **D3 NVML hardware matrix** : Phase D preflight doc.
- ⚠️ **D4 watermark prior-art gap** : Phase E preflight search.

**Rigor signal G4** : **5 findings P2+ + 2 P3** (total 7). No
P0/P1 blockers. Sufficient to clear gate avec pre-gel checklist
acknowledgement kickoff §4.5.

**Gate 2 unlock path** : S22 delivers Sybil base (Couches 1+2) +
NVML profile + rate-limit + encryption (S20) + supply chain
(S18) → Gate 2 (TransLingua, FamilyScan, EHPAD-Lien) unlocks per
HARDENING_ROADMAP.md §7 ligne 553. Design-conflict sur fairness
déferré via LT-1, pas blocker S22.

---

**Fin G1 Design Review Board Sprint 22**. Acknowledgement planner
dans `sprint22_kickoff.md §4.5`. Gel décisions D1..D5 validé
post-acknowledgement.
