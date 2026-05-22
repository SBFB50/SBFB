# Sprint 69 Phase B — preflight G8

Date : 2026-05-22 | HEAD : `c92e656` | Verdict : **PLAN-ADAPT**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, no band-aid, research before code — applicable : FG8 provenance verification touche Ed25519 verify vs verify_strict (context7 query done, finding ci-dessous).
- feedback_context7_systematic.md : context7 obligatoire avant code touchant lib/API — applique sur ed25519-dalek (context7 `/dalek-cryptography/ed25519-dalek` done 2026-05-22).
- vision_model.md : no funding/startup patterns — N/A (Phase B est du code interne).
- feedback_kudos_non_monetary.md : N/A (Phase B ne touche pas kudos).
- sprint14_keyoxide_decision.md : deploy from source Keyoxide Ed25519 — pertinent : FG8 verifie la provenance SLSA L1 generee par le daemon. La verification reutilise le meme pipeline crypto (DOMAIN_PROVENANCE_V1 + canonical bytes). Coherent.
- nexus_grid_pivot.md : S69 OPEN, tip c92e656, Day 0 D1-D5 gelees. Phase B touche D1 (FG8) et D3 (FG9 pipeline). Pre-launch policy : aucun VERSION bump.
- Tensions plan vs memory : aucune tension sur Day 0. PLAN-ADAPT sur le *comment* (dep nexus-coordinator-rs vs local verify), pas sur le *quoi* (FG8 + FG9 restent identiques).

## S1a — OSS prior art deep analysis

### Probleme fonctionnel exact

"How do mature OSS projects implement post-publish provenance verification in a CLI tool, and how do they gate a sequential publish pipeline with pre/post checks?"

### Projets analyses en profondeur

#### [F-Droid fdroidserver] — https://gitlab.com/fdroid/fdroidserver
- Fichiers source : `fdroidserver/publish.py` (pipeline), `fdroidserver/verify.py` (verification)
- Pattern : pipeline sequentiel — metadata → build → sign → verify → publish. Chaque etape abort si echec. Le verificateur est un composant separe du builder.
- Edge cases geres : signature invalide, hash mismatch, timeout, archive corrompue.
- Patterns abandonnes : verification integree dans le publish (separee pour permettre un tiers de confiance).
- Verdict : APPROACH-ALIGNED — pipeline sequentiel FG4→FG5→FG6→publish→FG8 conforme.

#### [sigstore/cosign] — https://github.com/sigstore/cosign
- Fichiers source : `cmd/cosign/cli/verify/verify_blob.go` (verification), `cmd/cosign/cli/sign/sign_blob.go` (signature)
- Pattern : sign blob → generate attestation → verify blob. La verification est locale (cle publique connue), pas de reseau requis. Verification = reconstruire les canonical bytes + comparer la signature.
- Edge cases : keyless OIDC signing (hors scope SBFB, on a des cles locales).
- Verdict : APPROACH-ALIGNED — FG8 fait la meme chose : reconstruire canonical bytes + verifier signature Ed25519 localement.

#### [sigstore-verification (Rust crate)] — https://crates.io/crates/sigstore-verification
- Pattern Rust : crate leger de verification pure, pas de deps lourdes (pas de runtime, pas de DB).
- Pertinence : confirme que la verification doit etre dans un module leger, pas dans un framework lourd.
- Verdict : APPROACH-ALIGNED avec CONCERN — la dep nexus-coordinator-rs tire des deps lourdes inutiles pour la verification pure.

#### [slsa-verifier] — https://github.com/slsa-framework/slsa-verifier
- Fichiers source : `verifiers/internal/gha/verifier.go` (verification logic)
- Pattern : CLI Go separe du builder. La verification est autonome — elle ne depend pas du builder pour verifier. Elle reconstruit l'attestation et compare.
- Verdict : APPROACH-ALIGNED — FG8 est un verifieur autonome integre dans Factory (le client verifie le daemon).

#### [ed25519-dalek] — /dalek-cryptography/ed25519-dalek (context7 2026-05-22)
- `VerifyingKey::verify()` permet les weak keys (8 points d'ordre 8 specifiques).
- `VerifyingKey::verify_strict()` rejette les weak keys — recommande pour les provenance attestations.
- `VerifyingKey::is_weak()` pour pre-validation.
- Le code actuel `nexus_core_rs::crypto::verify()` (crypto.rs:169-175) utilise `verifying.verify(message, &sig)` — pas verify_strict.
- Verdict : CONCERN Low — le risque de weak key est theorique (probabilite ~2^-252 avec OsRng), mais le SOTA recommande verify_strict pour les attestations de provenance. Un upgrade futur de `nexus_core_rs::crypto::verify()` vers verify_strict est recommandable mais hors scope Phase B (changerait le comportement de tous les verifiers du projet).

### Tableau comparatif

| Aspect | Plan Phase B | F-Droid | cosign | slsa-verifier | sigstore-verification (Rust) |
|--------|-------------|---------|--------|---------------|------------------------------|
| Pipeline sequentiel | FG4→FG5→FG6→publish→FG8 | metadata→build→sign→verify→publish | sign→attest→verify | build→attest→verify | verify only |
| Abort on failure | Oui (bloquant sauf FG4) | Oui | Oui | Oui | N/A |
| Verification autonome | Via dep nexus-coordinator-rs | Module separe | CLI separe | CLI separe | Crate leger |
| Dep verification | Tire rusqlite+tokio+chrono... | Python standard lib | Go standard lib | Go standard lib | Deps minimales |
| verify_strict | Non (verify non-strict) | N/A (pas Ed25519) | N/A (Sigstore/Fulcio) | N/A | N/A |

### Finding S1a

**Classification : APPROACH-ALIGNED avec PLAN-ADAPT mineur**

Le pipeline sequentiel FG4→FG5→FG6→publish→FG8 est conforme au SOTA (tous les projets majeurs utilisent un pipeline sequentiel avec abort). L'architecture (client verifie builder) est conforme au pattern F-Droid rebuilder et cosign verify-blob.

**PLAN-ADAPT** : la dep `nexus-coordinator-rs` pour `verify_provenance()` tire des deps lourdes (rusqlite, tokio, chrono, hmac, sha2, rand, strsim, toml) dont aucune n'est utilisee par la fonction cible. Les projets OSS de reference (sigstore-verification, cosign) isolent la verification dans un module leger.

**Approche corrigee** : au lieu de dependre de `nexus-coordinator-rs`, la Phase B doit :
1. Dependre directement de `nexus-core-rs` (deja dans le workspace, donne acces a `DOMAIN_PROVENANCE_V1` + `crypto::verify` + `serde_jcs`).
2. Dependre de `hex` (deja workspace dep).
3. Implanter `verify_provenance()` localement dans `crates/sbfb-factory/src/gates.rs` — la fonction fait ~25 LOC utiles (deserialize JSON, extraire les champs, reconstruire canonical bytes, appeler `crypto::verify`).
4. Ne PAS dependre de `nexus-coordinator-rs`.

**Justification** : `nexus-core-rs` est le crate fondation qui contient les primitives crypto et les domain tags. Il est la dep naturelle pour tout crate du workspace qui a besoin de verifier des signatures. Le couplage `sbfb-factory → nexus-core-rs` est minimal et semantiquement correct (Factory verifie des signatures = Factory utilise les primitives crypto). Le couplage `sbfb-factory → nexus-coordinator-rs` est semantiquement incorrect (Factory n'est pas un coordinator).

**IMPORTANT** : ceci ne modifie PAS la Day 0 D1. D1 dit "FG8 Provenance Ed25519 verification dans Factory publish" — la destination (FG8 dans gates.rs) est preservee. Seul le *chemin d'import* change (nexus-core-rs au lieu de nexus-coordinator-rs). Le resultat fonctionnel est identique.

**Evidence** :
- sigstore-verification crate (https://crates.io/crates/sigstore-verification) : verification pure sans deps lourdes.
- `nexus-coordinator-rs/src/provenance.rs` imports (lignes 11-13) : seules deps utilisees sont `nexus_core_rs::canonical::DOMAIN_PROVENANCE_V1`, `nexus_core_rs::crypto::{KeyPair, blake3_hash}`, `serde`, `serde_json`, `hex`. Zero import rusqlite/tokio/chrono.
- `sbfb-factory/Cargo.toml` : n'a actuellement aucune dep nexus-core-rs ni nexus-coordinator-rs.

**NOTE workspace** : dans le contexte du workspace Cargo.toml actuel, toutes les deps sont deja compilees. Le cout marginal de `nexus-coordinator-rs` est nul en compilation workspace. La PLAN-ADAPT est une question de **couplage semantique et maintenabilite**, pas de performance de build. Si sbfb-factory est un jour publie comme crate independant (crates.io), la dep nexus-coordinator-rs serait problematique.

**Fichiers impactes vs plan** :
- `crates/sbfb-factory/Cargo.toml` : dep `nexus-core-rs` + `hex` au lieu de `nexus-coordinator-rs`.
- `crates/sbfb-factory/src/gates.rs` : `verify_provenance()` locale (~25 LOC, meme logique que nexus-coordinator-rs).
- Pas d'impact sur les tests (meme API, meme comportement).

## S1b — Deps/libs versions + CVE

### ed25519-dalek
- Version workspace : `"2.1"` → resolue `2.2.0` dans Cargo.lock.
- CVE check : 1 advisory RUSTSEC-2022-0093 (double public key oracle) — corrige dans ed25519-dalek 2.0+. Version 2.2.0 = non affectee.
- Version 3.0.0-pre.6 presente transitoirement via iroh — pas utilisee directement par nexus-core-rs.
- WebSearch "ed25519-dalek CVE 2026" : 0 resultat nouveau.
- **Finding** : clean.

### serde / serde_json
- Versions workspace stables (1.x). 0 CVE recents.
- **Finding** : clean.

### hex
- Deja workspace dep. Version stable. 0 CVE.
- **Finding** : clean.

### nexus-core-rs (PLAN-ADAPT target)
- Interne workspace, pas de CVE externe. Les deps crypto (ed25519-dalek 2.2.0, blake3, serde_jcs) sont stables.
- **Finding** : clean.

### DOMAIN_PROVENANCE_V1
- Defini `canonical.rs:104` : `b"nexus-provenance-v1"`. Inchange depuis Sprint 14.
- **Finding** : clean.

## S2 — Decision chain reconstruction

### Fichiers scannes
- `crates/sbfb-factory/src/gates.rs` : 1 commit lu (a201b3e3, S68 Phase C)
- `crates/sbfb-factory/src/publish.rs` : 1 commit lu (1d53f18c, S68 Phase B)
- `crates/sbfb-factory/src/daemon_client.rs` : 1 commit lu (1d53f18c, S68 Phase B)
- `crates/sbfb-factory/Cargo.toml` : 3 commits lus (49d6bcd0, 1d53f18c, a201b3e3)
- `crates/nexus-coordinator-rs/src/provenance.rs` : 3 commits lus (aaa2e182, 9b8abfa8, daa3a8ed)
- `crates/nexus-core-rs/src/crypto.rs` : 2 commits lus (4c2cba60, e51123ee)

### Decisions historiques trouvees

#### Decision 1 : Deploy verifie provenance Ed25519 SLSA L1
- Sprint 14, kickoff : decision deploy from source avec Keyoxide Ed25519.
  Memory `sprint14_keyoxide_decision.md` : "Deploy from source (clone+Keyoxide+SLSA L1 provenance)"
- Sprint 42, sha `aaa2e182` : port provenance.py → provenance.rs dans nexus-coordinator-rs.
  Body extrait : "genere provenance SLSA L1 (Ed25519 sign + blake3 artifact hash)"
- Sprint 64, sha `9b8abfa8` : fix provenance hash stability (app_version skip_serializing).
  Body extrait : "P1 provenance_hash drift: ProvenanceRecord.app_version now skip_serializing_if Option::is_none"
- Reverse-commit check : aucune reversion trouvee.
- Status : **active**
- Impact phase : aucun — Phase B ajoute FG8 qui **verifie** la provenance deja generee. Coherent avec la chaine de decisions.

#### Decision 2 : Factory hors daemon (crate sbfb-factory)
- Sprint 67 Phase C, sha `49d6bcd0` : creation du crate sbfb-factory.
  Body extrait : "Nouveau crate sbfb-factory : outil CLI autonome de scaffolding pour apps SBFB. Decision D2 v4 : Factory hors daemon, crate independant."
- Reverse-commit check : aucune reversion.
- Status : **active**
- Impact phase : confirme que FG8 doit etre dans sbfb-factory (CLI client), pas dans le daemon.

#### Decision 3 : verify() utilise VerifyingKey::verify (non-strict)
- Sprint 2, sha `4c2cba60` : creation crypto.rs avec `verify()`.
  Body extrait : pas de mention explicite strict vs non-strict.
- Code actuel `crypto.rs:173` : `verifying.verify(message, &sig)` — verify non-strict.
- Aucune decision documentee pour choisir verify over verify_strict.
- Reverse-commit check : aucune modification de verify() trouvee apres S2.
- Status : **active** (inchangee depuis S2)
- Impact phase : CONCERN Low. Le kickoff S69 D1 sources consultees mentionne "verify_strict recommande" mais la decision "Retenu" utilise verify_provenance() qui utilise verify non-strict. Le gap est accepte pre-launch (weak keys = 2^-252 probabilite avec OsRng). Un sprint dedie pourrait migrer crypto::verify → verify_strict globalement.

### Memory constraints
- feedback_approach.md : "pick deepest" → la PLAN-ADAPT (dep legere nexus-core-rs) est la plus profonde techniquement.
- feedback_context7_systematic.md : context7 ed25519-dalek fait, confirmant la distinction verify/verify_strict.
- sprint14_keyoxide_decision.md : "Deploy from source (clone+Keyoxide+SLSA L1 provenance)" — FG8 est l'extension naturelle (verification cote client).

## S3 — Threat model analysis

### Primitive analysee : FG8 Provenance Ed25519 verification + FG9 Publish pipeline

### Assets en jeu
- A1 Provenance signatures (A3 THREAT_MODEL) : criticite **Haute** (integrite). FG8 verifie que la signature provenance est valide apres publish.
- A2 Keypair Ed25519 node_id (A1 THREAT_MODEL) : criticite **Critique**. FG8 utilise la cle publique du daemon pour verifier — ne manipule PAS la cle secrete.
- A3 Archives zip (A6 THREAT_MODEL) : criticite **Haute** (integrite). Le pipeline FG5 sandbox + FG6 secrets protege l'archive avant publish.

### Threat actors
- TA1 Daemon compromis : daemon malveillant ou modifie qui genere une provenance invalide ou signee avec une autre cle. FG8 detecte ce cas.
- TA2 MITM loopback : attaquant qui intercepte le HTTP loopback entre Factory et daemon. Pre-mitige par bearer token + Host header (T0-T5 existants).
- TA3 Archive corrompue : le daemon reecrit l'archive pendant le deploy, divergence hash vs provenance. FG8 detecte (signature invalide car artifact_hash diverge).

### Attack vectors identifies
1. V1 Forge de provenance (injection) : attaquant soumet un record provenance signe avec une autre cle → FG8 rejette car node_id mismatch. Couvert.
2. V2 Tampered artifact_hash (tampering) : daemon corrompt le zip entre signature et stockage → FG8 verifie la signature sur les canonical bytes qui incluent artifact_hash. Si tampered, signature invalide. Couvert.
3. V3 Replay provenance d'un ancien deploy (replay) : attaquant rejoue une provenance valide d'un deploy precedent → FG8 ne detecte PAS (la provenance est valide, signee par le bon daemon). CONCERN Low : le replay d'un ancien deploy est detectable par les timestamps et commit_sha. Le daemon genere un nouveau record a chaque deploy.
4. V4 DoS via timeout pipeline (DoS) : Factory bloque indument le pipeline → le pipeline est local CLI, l'utilisateur peut Ctrl+C. Non critique.
5. V5 Bypass --skip-gates (privilege escalation) : utilisateur bypasse les gates pre-publish → par design, --skip-gates ne bypasse PAS FG8 (post-publish, toujours execute). Day 0 D3 explicite.
6. V6 Supply chain dep nexus-core-rs (supply chain) : dep interne workspace, pas de risque crates.io. OK.
7. V7 TOCTOU publish→verify (temporal) : entre le POST deploy-from-repo et le GET provenance, le daemon pourrait modifier la provenance. CONCERN Low : le daemon ecrit la provenance en DB avant de repondre. La fenetre temporelle est negligeable (meme processus daemon).

### Mitigations existantes
- T0-T5 couvrent V2 (loopback auth) : bearer token, Host header, Origin check.
- A3 (provenance signatures) couvre V1 : Ed25519 domain-separated canonical bytes.
- DOMAIN_PROVENANCE_V1 couvre le cross-domain replay : une provenance ne peut pas etre rejouee comme un task/claim/invite signature.

### Gaps identifies
- GAP1 V3 replay : severity **Low**. Le timestamp dans la provenance et le commit_sha rendent le replay detectable a l'inspection humaine. Un check automatise (reject si same commit_sha deja deploye) est recommandable post-Gate 1.
- GAP2 verify vs verify_strict : severity **Low**. Weak key acceptance theorique. Probabilite negligeable avec OsRng. Recommandation : upgrade global `crypto::verify` → `verify_strict` dans un sprint dedie.

### Regression check
- La primitive ne diminue pas l'efficacite d'une mitigation T0-T5 existante.
- La primitive ne cree pas de vecteur non couvert critique (GAP1/GAP2 sont Low).
- Aucun nouveau T necessaire.

**Verdict S3** : clean (2 gaps Low, 0 regression T0-T5)

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui (296 lignes)

Phase B ne modifie aucune struct dans canonical.rs. FG8 **lit** DOMAIN_PROVENANCE_V1 (ligne 104) pour reconstruire les canonical bytes localement — pas de modification.

### Structs verifiees

#### ProvenanceRecord (provenance.rs:17-29)
- PROVENANCE_SCHEMA_VERSION = 1 : OK, inchange.
- #[derive(Serialize, Deserialize)] : present.
- #[serde(default)] : present sur app_version (Option) avec skip_serializing_if. Rationale : runtime tolerance + hash stability (fix `9b8abfa8`).
- DOMAIN_PROVENANCE_V1 signature : OK (canonical_bytes() utilise DOMAIN_PROVENANCE_V1 + 0x00 separator).
- Serialization : canonical_bytes() dans provenance.rs:102-124 utilise `serde_json::json!()` + `serde_json::to_string()` — **pas serde_jcs**. C'est un choix delibere : les champs sont specifies manuellement en ordre alphabetique dans le json!() macro (artifact_hash, commit_sha, node_id, repo_url, schema_version, timestamp). L'ordre est fixe par le code, pas par JCS. CONCERN : si un champ est ajoute sans respecter l'ordre alphabetique, la signature casserait. Mais pre-launch, c'est acceptable (on redefinit la v1 courante).

### Day 0 check
- D1..D5 sprint courant : aucune contredite par Phase B.
  - D1 (FG8 provenance) : implante. Methode adaptee (dep nexus-core-rs au lieu de nexus-coordinator-rs) — PLAN-ADAPT, pas D1 contredite.
  - D3 (FG9 pipeline) : implante tel quel.
- Decisions actees pivot.md : aucune contredite.
  - "Deploy verifie from source" (D14) : FG8 renforce cette decision (verification cote client).
  - "Factory hors daemon" (D2 v4) : FG8 dans sbfb-factory, coherent.

### Pre-launch policy
- *_VERSION = 1 : OK, aucun bump.
- Pas de tolerant decoder multi-version : OK.
- Pas de tests "legacy decode" zombie : OK.

## Plan adaptation

**Evidence OSS** : sigstore-verification crate (https://crates.io/crates/sigstore-verification) — module de verification pure sans deps lourdes. `nexus-coordinator-rs/src/provenance.rs` lignes 11-13 : les seules imports sont `nexus_core_rs::canonical::DOMAIN_PROVENANCE_V1`, `nexus_core_rs::crypto::{verify}`, `serde`, `serde_json`, `hex`. Zero import rusqlite/tokio/chrono.

**Plan proposait** : dep `nexus-coordinator-rs` dans `sbfb-factory/Cargo.toml` pour reutiliser `verify_provenance()`.

**OSS montre** : la verification doit etre dans un module leger, sans deps non necessaires. Le couplage semantique `CLI tool → coordinator business logic crate` est incorrect meme si le cout de compilation workspace est nul.

**Approche corrigee** :
1. **Dep `nexus-core-rs`** dans `sbfb-factory/Cargo.toml` (acces a `canonical::DOMAIN_PROVENANCE_V1` + `crypto::verify`).
2. **Dep `hex`** dans `sbfb-factory/Cargo.toml` (hex decode de la signature).
3. **Fonction `verify_provenance_record()`** locale dans `gates.rs` (~25 LOC) : meme logique que `nexus_coordinator_rs::provenance::verify_provenance()` — deserialize JSON, extraire champs, reconstruire canonical bytes avec DOMAIN_PROVENANCE_V1 + 0x00 + serde_json payload alphabetise, appeler `nexus_core_rs::crypto::verify()`.
4. **PAS de dep `nexus-coordinator-rs`** dans sbfb-factory.

**Endpoint correction** : le plan mentionne `GET /api/daemon/status` pour obtenir le node_id. L'endpoint reel est `GET /api/daemon/info` (retourne `DaemonStateSnapshot` avec `node_id` hex 64 chars). A corriger dans l'implementation.

**Fichiers impactes vs plan** :
- `Cargo.toml` : dep `nexus-core-rs` + `hex` au lieu de `nexus-coordinator-rs`.
- `gates.rs` : `verify_provenance_record()` locale au lieu d'importer `nexus_coordinator_rs::provenance::verify_provenance`.
- `daemon_client.rs` : `get_node_id()` utilise `/api/daemon/info` au lieu de `/api/daemon/status`.
- Tests impactes : aucun — meme API publique de `run_gate_fg8_provenance()`.

## Findings

| # | Scan | Finding | Classification | Bloquant | Detail |
|---|------|---------|---------------|----------|--------|
| F1 | S1a | Dep nexus-coordinator-rs tire des deps lourdes inutiles | PLAN-ADAPT | Oui (S1a) | Corriger via dep nexus-core-rs + verify locale |
| F2 | S1a | verify() non-strict vs verify_strict() recommande | CONCERN Low | Non | Upgrade global hors scope Phase B. Risque theorique 2^-252 |
| F3 | S1a | Endpoint `/api/daemon/status` n'existe pas | Correction plan | Non | Utiliser `/api/daemon/info` a la place |
| F4 | S3 | GAP1 replay provenance ancien deploy | CONCERN Low | Non | Detectable par timestamp/commit_sha. Post-Gate 1 |
| F5 | S3 | GAP2 weak key acceptance | CONCERN Low | Non | Idem F2 |
| F6 | S4 | canonical_bytes provenance utilise serde_json pas serde_jcs | CONCERN Pre-launch | Non | Ordre alphabetique fixe par json!() macro. Acceptable pre-launch |

## Telemetrie preflight (agent deep)

- Duree totale : ~8m
- S1a : 5 projets OSS analyses (F-Droid, cosign, slsa-verifier, sigstore-verification, ed25519-dalek) / 2 context7 queries (ed25519-dalek verify + resolve) / 3 WebSearch queries / finding : PLAN-ADAPT (dep nexus-coordinator-rs → nexus-core-rs)
- S1b : 4 libs scannees (ed25519-dalek, serde, hex, nexus-core-rs) / 1 CVE search (RUSTSEC ed25519-dalek) / finding : clean
- S2 : 10 commits bodies lus / 0 archive files pertinents / 3 memory files lus / finding : clean (3 decisions actives, 0 conflit)
- S3 : FULL / 7 vectors analyses / 2 gaps Low
- S4 : FULL / 1 struct verifiee (ProvenanceRecord) / canonical.rs lu integralement : oui

## Action

Proceder code phase B avec approche corrigee :
- Dep `nexus-core-rs` + `hex` au lieu de `nexus-coordinator-rs`.
- `verify_provenance_record()` locale dans gates.rs.
- Endpoint `/api/daemon/info` au lieu de `/api/daemon/status`.

Commit body documente la deviation vs plan §5 :
"Plan sPhase B proposait dep nexus-coordinator-rs pour verify_provenance(). Preflight S1a a identifie que la dep tire des deps lourdes inutiles (rusqlite, tokio, chrono). Adapte vers dep nexus-core-rs + verify locale (~25 LOC). Meme logique, couplage semantique correct."
