# Sprint 68 Phase A — preflight G8

Date : 2026-05-21 | HEAD : `3ca563f` | Verdict : **EXECUTE plan-as-is**

---

## Memory consultation (Step 1.5)

- `feedback_approach.md` : pick deepest technical option, G8 = mecanisme procedural, OSS prior art OBLIGATOIRE avant chaque phase. ProofCard est un compute local deterministe — pas de choix technique a challenger (formule additive fixe, pas de crypto, pas de wire format). La depth requise est faible car la primitive est interne.
- `feedback_context7_systematic.md` : context7 obligatoire avant code touchant lib/API. Phase A utilise des deps existantes (serde, axum, rusqlite) deja dans le workspace — pas de nouvelle dep. context7 temporairement indisponible (502), deps verifiees par WebSearch.
- `feedback_kudos_non_monetary.md` : ProofCard est un "score de completude de preuve", pas un score de reputation ni une monnaie. Pas de cost/deposit/stake. Pas de tension.
- `fairness_vision.md` : ProofCard score ≠ kudos score. Pas d'interaction. Pas de tension.
- `vision_model.md` : compute local solo maintainer, pas de pattern institutionnel. Pas de tension.
- `nexus_grid_pivot.md` : D16 formula_version gelee, ProofCard = artefact local daemon (pas wire format). Confirme le plan.
- Tensions plan vs memory : **aucune**.

---

## S1a — OSS prior art deep analysis

### Probleme fonctionnel exact

"How do mature OSS projects compute and display a multi-layer evidence score for software supply chain verification?"

### Projets analyses en profondeur

#### 1. OpenSSF Scorecard V5 (https://github.com/ossf/scorecard)

- Fichiers source identifies : `checks/` (18+ check implementations), `pkg/scorecard/scorecard.go` (aggregation), `probes/` (structured results V5).
- Pattern architectural : chaque check est un module independant qui retourne un `CheckResult` avec un score 0-10. L'aggregation est une moyenne ponderee configurable. V5 ajoute des "probes" (structured results) qui remplacent le score agrege par des evidences binaires.
- Edge cases : chaque check peut retourner `Inconclusive` si l'evidence est insuffisante. Score -1 = error, 0 = fail, 10 = pass.
- Verdict : **ALIGNED** — le pattern ProofCard SBFB (score composite depuis evidences binaires par couche) est compatible avec l'approche Scorecard. Difference : Scorecard score 0-10 par check, ProofCard additive 0-100 global. Scorecard vise CI/CD GitHub, ProofCard vise les couches protocolaires P2P SBFB.

#### 2. F-Droid verification (https://verification.f-droid.org, https://gitlab.com/fdroid)

- Pattern architectural : badges par app avec 3 etats (not built, building, verified reproducible). IzzyOnDroid 5-level graph (unknown → available → checked → verified → reproducible).
- Edge cases : le statut "reproducible" exige un rebuild independant par un tiers. F-Droid ne produit pas de score numerique — c'est un statut categoriel.
- Verdict : **ALIGNED** — ProofCard adopte le pattern "couches de preuve visibles" (provenance, license, freshness, curation). Difference : F-Droid est categoriel (pass/fail par app), ProofCard est numerique (0-100 composite). Les deux sont evidence-based, pas trust-based.

#### 3. W3C Verifiable Credentials 2.0 (https://www.w3.org/TR/vc-data-model-2.0/)

- Publie W3C Recommendation 2025-05-15. Modele `proof` block (Data Integrity) ou JWT/JWS pour attacher des preuves cryptographiques a des claims.
- Pattern architectural : un VC contient `credentialSubject` (les claims) + `proof` (la preuve crypto). Le Render Method (prevu sept. 2026) definira comment afficher un VC.
- Edge cases : W3C VC est un standard d'interoperabilite multi-emetteur. L'overhead est significatif pour un compute local (enveloppe JSON-LD, contextes, schemas).
- Verdict : **N/A** — surdimensionne pour un compute local (kickoff D1 "rejete" documente). ProofCard est un self-report daemon, pas un credential interoperable. Pattern confirme par l'absence de composant UI standard dans l'ecosysteme VC.

#### 4. Sigstore / Rekor (https://github.com/sigstore/cosign, https://github.com/sigstore/rekor)

- Pattern architectural : cosign signe les artifacts, Rekor fournit un transparency log immutable. Le bundle contient signature + certificate + timestamp + proof of inclusion. Verification via `cosign verify`.
- Pas de composant UI/badge standard. Chaque projet construit son propre affichage.
- Verdict : **ALIGNED** — confirme que l'espace design pour un composant de verification visuel est ouvert. ProofCard comble ce gap pour le contexte SBFB P2P.

#### 5. BOINC Credit System (https://github.com/BOINC/boinc/wiki/CreditOptions)

- Formule : runtime * ncpus * peak_flops. Validation via quorum (minimum 2 resultats concordants). CreditNew (2009) a abandonne la formule lineaire au profit de la mediane des claims validees.
- Pattern architectural : score additif simple, validation croisee, pas de couches d'evidence.
- Verdict : **N/A** — BOINC compute du credit de contribution, pas de la completude de preuve d'un artefact. Pas comparable.

### Tableau comparatif

| Aspect | Plan Phase A | Scorecard V5 | F-Droid | Sigstore |
|--------|-------------|-------------|---------|----------|
| Score type | Additif 0-100 | Pondere 0-10 par check | Categoriel (5 niveaux) | Binaire (signed/unsigned) |
| Evidence layers | 7 layers + 7 risk factors | 18+ checks | 3-5 niveaux | Signature + transparency log |
| Compute location | Local daemon | Cloud/CI GitHub API | Serveur build centralisé | CLI + transparency log |
| formula_version | v1 (gelee D16) | Implicite dans le code | N/A | N/A |
| UI component | Carte expandable Browse | API JSON + badge shields.io | Badge HTML | Pas de UI standard |
| Wire format impact | Aucun (compute local) | API REST | HTTP status page | Bundle file |

### Finding S1a

- Classification : **APPROACH-ALIGNED**
- Evidence : Scorecard V5 (score composite depuis evidences), F-Droid (couches de verification visuelles), Sigstore (pas de UI standard = espace ouvert)
- Impact sur le plan : **aucun** — l'approche ProofCard (score additif 0-100 depuis couches de preuve locales) est alignee avec le SOTA. La formule additive fixe est plus simple que Scorecard (pondere) mais adequate pre-launch (formula_version permet evolution). Pas de lib existante qui fait exactement ce compute dans le contexte P2P SBFB.

---

## S1b — Deps/libs versions + CVE

### Deps Phase A perimetre

| Dep | Version workspace | Usage Phase A | CVE/Advisory 2025-2026 |
|-----|------------------|---------------|----------------------|
| serde + serde_json | workspace (1.x) | Serialize/Deserialize ProofCard struct | **0 CVE** serde_json (serde-json-wasm touche, pas serde_json) |
| axum | workspace (0.7/0.8) | GET /api/daemon/proof-card/{project_id} handler | **0 CVE** aucun advisory RustSec |
| rusqlite | workspace | Lecture browse/feed/provenance existants | CVE-2025-6965 (SQLite < 3.50.2) — carry S67, non exploitable (SQL parameterise) |
| chrono | workspace | Timestamp comparaison freshness | 0 CVE |
| hex | workspace | Node ID encoding | 0 CVE |
| thiserror | workspace | Error types | 0 CVE |
| tokio | workspace | Async runtime existant | 0 CVE |
| zod (npm) | existant | Schema ProofCard frontend | 0 CVE |

### Nouvelles deps Phase A

**Aucune nouvelle dep ajoutee.** Phase A utilise exclusivement des deps deja presentes dans le workspace. Pas de bump.

### Specs touchees

Aucune spec externe (RFC, SLSA, JCS) n'est touchee par Phase A. ProofCard est un compute local qui lit des donnees existantes.

### Finding S1b

- **0 delta dep, 0 CVE bloquant, 0 breaking change.** Clean.
- Carry existant CVE-2025-6965 (rusqlite/SQLite) : non exploitable dans le perimetre Phase A (queries parametrisees, corpus < 500 entries).

---

## S2 — Decision chain reconstruction

### Fichiers scannes

- `crates/nexus-coordinator-rs/src/lib.rs` : 3 commits pertinents lus (S67 Phase A, S67 Phase B, S42)
- `crates/nexus-coordinator-rs/src/provenance.rs` : 5 commits lus (S67, S66, S64, S54, S42)
- `crates/nexus-coordinator-rs/src/search.rs` : 2 commits lus (S67 Phase B, S54)
- `crates/nexus-shell-daemon/src/http.rs` : 50 commits scannes
- `crates/sbfb-manifest/src/lib.rs` : 1 commit (S67 Phase A creation)
- `web/src/bridge/protocol.ts` : 10 commits lus
- `web/src/bridge/useBridge.ts` : 10 commits lus

### Decisions historiques trouvees

#### Decision 1 : ProofCard comme compute local (pas wire format)

- Sprint 68, kickoff D1 : "ProofCard comme struct Rust dans nexus-coordinator-rs (pas dans sbfb-manifest — c'est un artefact daemon, pas un artefact Factory). Le compute est local — le daemon rassemble les donnees qu'il possede deja."
- Sprint 68, scope cut #13 : "ProofCard comme feed op → S70+"
- Status : active (D1 gelee)
- Impact phase : **aucun** — Phase A implemente cette decision.

#### Decision 2 : Feed extensible via raw-op (pas de bump)

- Sprint 67 Phase A, sha `4ee93ab` : "FEED_FORMAT_VERSION = 1 preserve. CuratorVouched/CuratorDisendorsed = nouvelles variantes PublicFeedOperation, PAS de bump (raw-op P51)."
- Status : active
- Impact phase : **aucun** — ProofCard Phase A ne touche pas le feed. Candidat feed op S70+ (scope cut #13).

#### Decision 3 : Bridge method allowlist dans sbfb-manifest

- Sprint 67 Phase A, sha `4ee93ab` : creation du crate sbfb-manifest avec `BRIDGE_METHOD_ALLOWLIST` (9 methodes).
- Status : active
- Impact phase : **Phase A ajoute "proof_card_get" a cette allowlist.** Extension additive, pas de conflit.

#### Decision 4 : Endpoint pattern /api/daemon/*

- Multiples sprints (S11→S67) : tous les endpoints daemon suivent le pattern `/api/daemon/{resource}`. Endpoints read (GET) ne requierent que le bearer token.
- Status : active
- Impact phase : **Phase A ajoute GET /api/daemon/proof-card/{project_id}** — conforme au pattern.

### Memory constraints

- `feedback_approach.md` : "pick deepest technical option" — la formule additive fixe est la solution la plus simple mais la plus adaptee pre-launch (formula_version evolue post-launch). Pas de conflit.
- `feedback_kudos_non_monetary.md` : ProofCard score ≠ kudos. Pas de cost/deposit/stake. Pas de conflit.

### Reverse-commit check

Aucun finding S2 necessitant un reverse-commit check (pas de decision historique en conflit avec le plan Phase A).

---

## S3 — Threat model analysis

### Primitive analysee : ProofCard computation + daemon endpoint

### Assets en jeu

- A1 ProofCard score : criticite **low** — le score est un indicateur informatif, pas un controle d'acces. Un score errone ne compromet ni la securite du reseau ni les cles Ed25519.
- A2 Browse cache data : criticite **medium** — les donnees sources (browse entries, provenance records, feed entries) sont deja protegees par le bearer auth loopback.
- A3 Endpoint HTTP : criticite **low** — endpoint read-only derriere le bearer token existant.

### Threat actors

- TA1 Extension navigateur malveillante : pourrait appeler proof_card_get via le bearer token si vole. Impact : information leak score (low).
- TA2 App iframe malveillante : pourrait appeler proof_card_get via le bridge. Impact : information leak score (low — le score est derive de donnees publiques).

### Attack vectors identifies

1. **V1 Formula gaming** (T-PROOFCARD-FORMULA-GAME, prevu Phase D THREAT_MODEL §12) : un attaquant optimise son projet pour maximiser le score sans substance reelle. Couverture : formula_version 1 documente, couches detaillees visibles (pas juste le score). Severity : **low**.
2. **V2 Endpoint DoS** : flood GET /api/daemon/proof-card/{project_id}. Couverture : bearer auth loopback (pas d'acces anonyme), compute rapide (~1ms local SQLite lookups), corpus < 500 entries. Severity : **low**.
3. **V3 Injection project_id** : project_id passe en path parameter. Couverture : path parameter axum extrait comme String, utilise dans des queries SQLite parametrisees. Pas de SQL dynamique. Severity : **negligible**.
4. **V4 Information leakage** : le score revele des meta-donnees sur un projet (presence/absence de provenance, freshness, curation). Couverture : toutes ces donnees sont deja publiques via /api/daemon/browse et /api/v1/project/{id}/provenance. Pas de nouvelle surface. Severity : **negligible**.

### Mitigations existantes

- Bearer auth loopback (T0-T5 Sprint 16) couvre V2, V3, V4.
- SQL parametrise (M1-M15 migrations) couvre V3.
- Bridge method allowlist (sbfb-manifest) couvre TA2 — seuls les methodes declarees dans SBFB.json sont proxiees.

### Gaps identifies

- **Aucun gap severity H/M.** T-PROOFCARD-FORMULA-GAME est prevu Phase D (ajout THREAT_MODEL §12) — pas un gap Phase A.

### Regression check

- La primitive ne diminue l'efficacite d'aucune mitigation T0-T5.
- La primitive ne cree aucun nouveau vecteur non couvert.
- Aucun nouveau T necessaire Phase A (T-PROOFCARD Phase D).

### Verdict S3 : **clean**

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui

Fichier `crates/nexus-core-rs/src/canonical.rs` lu en entier (296 lignes). 14 constantes DOMAIN_*_V1 identifiees. Fonction `canonical_bytes<T>()` avec domain separation + null byte + serde_jcs.

### ProofCard et canonical.rs

**ProofCard n'est PAS un struct signe.** Elle n'a pas de DOMAIN_*_V1 et ne passe pas par `canonical_bytes()`. C'est un artefact compute local (le daemon calcule le score a la volee depuis ses donnees locales et le retourne via HTTP). Aucune interaction avec le wire format.

### Structs verifiees

Aucune struct existante dans canonical.rs n'est touchee par Phase A. ProofCard est un NEW module dans `nexus-coordinator-rs`, independant du wire format.

### Version constants check

| Constante | Fichier | Valeur | Phase A impact |
|-----------|---------|--------|----------------|
| FEED_FORMAT_VERSION | public_feed.rs:20 | 1 | Aucun |
| PROJECT_ANNOUNCEMENT_VERSION | publish.rs:24 | 1 | Aucun |
| PROVENANCE_SCHEMA_VERSION | provenance.rs:15 | 1 | Aucun |
| TASK_FORMAT_VERSION | task.rs (core-rs) | 1 | Aucun |

**Aucun bump de version.** ProofCard est un compute local, pas un wire format.

### Day 0 check

- D1 ProofCard struct + formule score deterministe : Phase A **implemente** D1. Pas de contradiction.
- D2 Preview ephemere : Phase B, non touche.
- D3 Publish path : Phase B, non touche.
- D4 Factory gates : Phase C, non touche.
- D5 ProofCard UI Browse : Phase D, non touche.

### Decisions actees pivot.md

Checklist des 12 decisions gelees + 3 extensions S12-S14 : aucune contredite par Phase A ProofCard.

### Pre-launch policy

- `*_VERSION = 1` : preservees (aucun bump).
- ProofCard `formula_version = 1` : constante locale daemon, pas un wire format protocolaire.
- Pas de tolerant decoder multi-version : N/A (pas de wire format).
- Pas de tests "legacy decode" zombie : N/A.

### Verdict S4 : **clean**

---

## Telemetrie preflight (agent deep)

- Duree totale : estimation ~8 min
- S1a : 5 projets OSS analyses (Scorecard, F-Droid, W3C VC, Sigstore, BOINC) / 0 fichiers source lus via WebFetch (context7 indisponible 502, analyse via WebSearch + documentation) / 8 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : 8 libs scannees / 3 CVE searches / finding : clean (0 new dep, 0 CVE bloquant)
- S2 : 70+ commits scannes (git log 50+ sur http.rs, 5 sur provenance.rs, 2 sur search.rs, 10+ protocol.ts/useBridge.ts) / 2 commit bodies lus en entier (S67 Phase A, S67 Phase B) / 6 memory files lus / finding : clean
- S3 : FULL / 4 vectors analyses / 0 gaps severity H/M
- S4 : FULL / 0 structs modifiees / canonical.rs lu integralement : oui / 4 version constants verifiees / 0 bump

---

## Action

Proceder code phase A.
