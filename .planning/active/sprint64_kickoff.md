# Sprint 64 — Kickoff (hardening public cible)

**Ecrit** : 2026-05-16 (post-audit gate S63 PASS `ebebe89`).
**Type** : **sprint pair** — phase dette obligatoire (Regle 1 §6.2.1).
Deux MANDATORY 3/3 (Regle 2) : F1 P2-VERSION-NOT-STORED +
F5 P2-IROH-INFRA-TIMEOUT.
**Tip master d'entree** : `ebebe89` (audit findings S63 P2 routing
metadata — PASS verdict `15d8fbf`).
**Phase 0 audit Sprint 63** : **DEJA JOUE** — `15d8fbf` PASS
(0 P0, 0 P1, 2 P2, 1 P3). Aucun fix bloquant requis.
**Version archive** : v2.0 — Public Verifiable Protocol Feed.
**Roadmap source** : `.planning/research/public_verifiable_feed_roadmap.md`
Sprint 4 sur 6 (5+1 reserve).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-13 (3 jours).
  3 triggers evalues (INCHANGES depuis S63) :

  1. **iroh > 0.98 + iroh-docs > 0.98** : iroh = "1.0.0-rc.0",
     iroh-docs = "0.99.0". Inchange. **Decision** : reste deferred
     (upgrade iroh 1.0 = sprint dedie post-feed).

  2. **arti-client > 0.41** : arti-client = "0.42.0". Inchange.
     **Decision** : deferred. 0 CVE entre 0.41 et 0.42.

  3. **frost-ed25519 > 3.0** : frost-ed25519 = "3.0.0". Trigger
     INACTIF (on utilise 3.0.0, trigger > 3.x).

- **Codebase audit** (agent Explore — 6 points verifies) :

  1. **F1 VERSION-NOT-STORED** : `db.rs` M12 `provenance_records`
     ne stocke PAS le `version` (tag/release identifier). Seuls
     `commit_sha` et `artifact_hash` sont presents. Fix = M13
     colonne `app_version TEXT` + insert dans `deploy.rs`. ~15 LOC.

  2. **F5 IROH-INFRA-TIMEOUT** : `feed_sync.rs:281`
     `spawn_feed_subscribe()` n'a pas de timeout sur le subscribe
     call iroh-docs. Tests SBFB_INTEGRATION absents du codebase
     Rust (les E2E multi-daemon existent mais ne testent pas le
     scenario timeout). Fix = timeout wrapper + test stabilite.
     ~20 LOC.

  3. **Tests adversariaux** : 5 tests adversariaux existants
     (4 dans `public_feed.rs` + 1 dans `feed_materializer.rs`).
     Couvrent : forged signature, tampered hash, out-of-order,
     multi-author, cursor hash mismatch. Manquent : fork-bomb
     spam, payloads oversized, PoW difficulty bypass, operations
     avec mauvais repo/hash, nouveau noeud from scratch.
     Estimation couverture cible : +8-10 scenarios, ~150-200 LOC.

  4. **FEED-SUBSCRIBE-JOINHANDLE** (2/3) : `spawn_feed_subscribe()`
     tokio::spawn sans retourner JoinHandle. Pas de join au
     shutdown. Resource leak potentiel. ~25 LOC.

  5. **BACKFILL-6PLUS-TEST** (2/3) : aucun test integration
     verifiant backfill >= 6 entries + dedup + rate limiter
     exempt. ~80 LOC.

  6. **P2-PROCESS-FORMAT** : carry documente, exit condition =
     supprimer §6 LOC plan.md OU ajouter exemption retroactive.
     Choix : ajouter exemption dans README.md §6.7. ~5 LOC doc.

- **ROADMAP_COMMITMENTS check (G7 Regle 3)** :
  - LT-1 Kudos-v2 : **CLOSED S59**.
  - LT-2 Radicle : **trigger PENDING** — tag v1.0 pose localement,
    pas pousse vers origin. Push prevu Sprint 65 (go-live). Pas
    encore actif pour S64.
  - LT-3/LT-4/LT-5 : latent. 0 condition declenchee.
  - LT-6 : RESOLVED S32.
  - LT-7 : gate satisfait (Tier 1+2 S55 + Tier 3 S60).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 63 a livre le 3eme sprint de la roadmap "Public Verifiable
Protocol Feed" sur le theme verification tiers + UX. Le protocole
feed est desormais : spec executable (S61) + sync P2P durable +
anti-spam minimal (S62) + verification tiers HTTP/bridge/UI +
Protocol Explorer demo (S63). Il manque le hardening adversarial
pour rendre le protocole robuste face a des attaquants reels.

### §1.2 Ancrage HARDENING_ROADMAP

Le sprint 64 realise la majorite du Sprint 4 roadmap ("Hardening
public cible"). Les phases A-C adversariaux + nouveau noeud + doc
protocole correspondent directement aux phases decrites dans la
roadmap §Sprint 4.

### §1.3 Compteurs tests entree (tip `ebebe89`)

| Suite | Count |
|---|---|
| Rust nextest | 1305 |
| Vitest | 265 |
| size-limit | 6/6 |
| **Total** | **~1576** |

### §1.4 Pre-launch protocol policy (rappel)

`*_FORMAT_VERSION` reste a 1 jusqu'au premier tag v1.0 go-live
public. Un sprint qui change le canonical ne bumpe PAS la version.
Pas de tolerant decoder multi-version. `#[serde(default)]` reste
legitime pour robustesse runtime.

---

## §2 Goal

Le sprint durcit le feed public face aux attaques adversariales :
tests de forgery, spam, corruption, payloads invalides couvrent
les 7 primitives crypto × les wire formats. Un nouveau noeud from
scratch peut rattraper et verifier le feed complet. Les 2 MANDATORY
3/3 (version stockee + timeout stabilite) sont resolus, et la
dette pair absorbe les resource leaks identifies.
**Critere SMART : toutes les rows fail-fast vertes au
verification.md, mesure binaire au Phase E wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 63

**DEJA JOUE** : `15d8fbf` + `ebebe89` (audit findings PASS +
P2 routing metadata). 0 P0, 0 P1, 2 P2 documentes, 1 P3.
Aucun fix bloquant. Ouverture S64 autorisee.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Tests adversariaux : couverture ciblee feed + crypto

**Retenu** : Tests adversariaux organises en 2 phases distinctes
(feed adversarial Phase B + crypto adversarial Phase C). Chaque
test couvre un scenario specifique : forged signature, tampered
hash-chain, fork-bomb spam (1000 operations), payloads oversized
(> 64 KB), PoW difficulty bypass, operations avec mauvais
repo/hash, seq gap injection, prev_hash manipulation. Les tests
existants (5) sont preserves et enrichis. Cible : 15+ tests
adversariaux au total (existants + nouveaux).

**Rejete** :
- Fuzzing generique (cargo-fuzz/proptest) : overhead runtime trop
  eleve pour la couverture obtenue, les scenarios adversariaux sont
  mieux servis par des tests deterministes qui documentent le vecteur
  d'attaque exact. Fuzzing = post-v1.0 audit prep.
- Test adversarial unique monolithique : decouplage feed/crypto
  permet de diagnostiquer plus finement quel layer echoue.

**Implications code** : `public_feed.rs` (tests module), nouveau
fichier `adversarial_tests.rs` dans `nexus-coordinator-rs/src/tests/`
si le module tests depasse 1500 lignes.

### D2 — Nouveau noeud from scratch : scenario E2E multi-daemon

**Retenu** : Test E2E dans `multi_daemon.rs` (pattern existant
S57/S62) : daemon neuf → join reseau → sync feed entier → rebuild
Browse via materializer → verify toutes les preuves hash-chain +
Ed25519 → valider coherence curseur. Un seul test E2E, gate
`SBFB_INTEGRATION=1`.

**Rejete** :
- Scenario distribue reel (3+ machines) : le test E2E mono-machine
  suffit pour prouver le code path. Le scenario multi-machine est
  couvert par LT-7 Tier 3 (S60 valide, Helsinki VPS).
- Docker compose test : overhead infra pour un scenario prouvable
  en local avec le pattern DaemonCluster existant.

**Implications code** : `multi_daemon.rs` (1 nouveau test E2E),
`lib.rs` test-harness (helpers potentiels).

### D3 — MANDATORY F1 : version stockee via migration M13

**Retenu** : Migration M13 ajoute colonne `app_version TEXT`
dans `provenance_records`. Le `deploy.rs` insere la version
extraite du champ `version` dans `SBFB.json` (champ a ajouter
au schema — aujourd'hui seuls `node_id` et `name` sont presents).
L'endpoint `GET /api/v1/project/{id}/provenance` retourne le
champ additionnel. Les examples/ sont mis a jour avec un field
`version` exemplaire.

**Rejete** :
- Colonne NOT NULL : impossible sans default value car la table
  existe deja avec des rows (les deploys S63). `TEXT` nullable
  avec backfill = plus safe.
- Champ version dans l'annonce gossip : surcharge le wire format
  pour un champ qui appartient a la provenance, pas a l'annonce.
- Version derivee du commit SHA seul : le commit SHA est deja
  stocke dans `commit_sha`, le field `version` porte un semver
  humain lisible (ex: "1.0.0") qui n'est pas deductible du SHA.

**Implications code** : `db.rs` (M13), `deploy.rs` (insert),
`http.rs` (provenance endpoint response), `sbfb-bridge.js`
(provenance_get retourne version).

### D4 — MANDATORY F5 : timeout subscribe + stabilite SBFB_INTEGRATION

**Retenu** : `spawn_feed_subscribe()` enveloppe le subscribe
iroh-docs dans un `tokio::time::timeout(Duration::from_secs(30))`
avec retry backoff (pattern existant `feed_sync.rs:120-140`).
Un test SBFB_INTEGRATION verifie que le subscribe se reconnecte
apres timeout simule. Le critere de stabilite "0 timeout 5 runs
consecutifs" est verifie en CI via un script `scripts/feed-stability-check.sh`.

**Rejete** :
- Timeout global au niveau runtime : trop agressif, casse les
  subscribes longs (feed complet 1000+ entries). Le timeout
  s'applique au heartbeat/liveness, pas au bulk transfer.
- Pas de timeout (laisser iroh gerer) : iroh-docs 0.98 n'a pas
  de timeout interne sur subscribe. Absence = hang indefini.

**Implications code** : `feed_sync.rs` (timeout wrapper),
`multi_daemon.rs` (test stabilite).

### D5 — Documentation protocole PUBLIC_FEED_SPEC.md finalisee

**Retenu** : `docs/protocol/PUBLIC_FEED_SPEC.md` est complete
depuis S61 Phase A pour la spec core (9 sections, §1-§9 dont §9
= Versioning policy). Sprint 64 ajoute 3 sections manquantes :
§10 "Adversarial scenarios & mitigations" (table des vecteurs
couverts par les tests), §11 "New node bootstrap procedure"
(algorithme de rattrapage), §12 "Security considerations" (resume
threat model feed). Pas de nouvelle spec from scratch —
enrichissement du document existant.

**Rejete** :
- Spec separee par composant (feed spec + crypto spec + sync spec) :
  fragmentation pour un protocole qui fait < 2000 lignes total.
  Un seul document avec des sections claires.
- Spec formelle TLA+/Alloy : overkill pre-audit. Les tests
  adversariaux deterministes suffisent pour cette phase.

**Implications code** : `docs/protocol/PUBLIC_FEED_SPEC.md`
(3 sections ajoutees), pas de code Rust.

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅, D5 ❌.
Rigor signal G4 satisfait (1 ⚠️ + 1 ❌ sur 5).

D3 ⚠️ : SBFB.json examples ne contiennent pas de field `version`
(seulement `node_id` + `name`). Decision : adjust — Phase A inclut
l'ajout du field `version` dans SBFB.json schema + examples avant
migration M13. Le field est optionnel (nullable DB) pour ne pas
casser les deploys existants sans version.

D5 ❌ : §9 "Versioning policy" EXISTE DEJA dans PUBLIC_FEED_SPEC.md
(lignes 295-308). Le plan proposait §9/§10/§11 mais §9 est occupee.
Decision : adjust — les sections ajoutees sont §10/§11/§12 (pas §9).
Corrige dans le plan ci-dessus.

---

## §5 Plan Phase outline A..E

### Phase A — MANDATORY 3/3 (F1 version + F5 timeout) [MANDATORY]

Resout les 2 items 3/3 obligatoires :
- F1 : Migration M13 `app_version` + insert deploy.rs + endpoint
  response + bridge SDK
- F5 : Timeout subscribe + test stabilite SBFB_INTEGRATION

**Commit cible** : `feat(feed): Sprint 64 Phase A — MANDATORY version stored + subscribe timeout`
**Critere** : M13 schema OK, version visible endpoint, subscribe timeout 30s, test stabilite vert.

### Phase B — Dette pair (5 items P2) [DETTE OBLIGATOIRE §6.2.1]

Sprint pair — phase dette non-negociable. Absorbe 5 items :
- P2-FEED-SUBSCRIBE-JOINHANDLE (2/3) : JoinHandle trackee + join shutdown
- P2-BACKFILL-6PLUS-TEST (2/3) : test integration backfill >= 6
- P2-FEED-PUBLISH-ORPHAN (2/3) : retry/rollback split DB/iroh-docs insert
- P2-SUBSCRIBE-STREAM-BREAK (2/3) : reconnexion auto apres stream break
- P2-PROCESS-FORMAT (herite) : exemption retroactive README.md

**Commit cible** : `feat(feed+docs): Sprint 64 Phase B �� dette pair 5 items P2`
**Critere** : JoinHandle join au shutdown, test backfill 6+ vert, orphan retry/rollback, reconnexion test, exemption doc.

### Phase C — Tests adversariaux feed [HARDENING]

Scenarios adversariaux feed public :
- Fork-bomb 1000 operations spam (rate-limited)
- Payloads oversized (> 64 KB reject)
- Operations avec mauvais repo URL / mauvais artifact hash
- Seq gap injection (prev_hash != last entry)
- prev_hash manipulation (swap entre 2 entries)
- Signature cross-author forgery (author A signe entry B)

**Commit cible** : `feat(feed): Sprint 64 Phase C — adversarial tests feed public`
**Critere** : +6 tests adversariaux, tous rejettent correctement, 0 regression.

### Phase D — Tests adversariaux crypto + nouveau noeud [HARDENING]

- Tests crypto × wire formats : Ed25519 forgery across 7 primitives,
  BLAKE3 tampering canonical bytes, PoW nonce brute-force check,
  age witness future timestamp reject
- Nouveau noeud E2E : daemon neuf → join → sync → rebuild Browse →
  verify chain → coherence curseur

**Commit cible** : `feat(feed): Sprint 64 Phase D — adversarial crypto + new node E2E`
**Critere** : +4 tests crypto, +1 test E2E nouveau noeud, tous verts.

### Phase E — Documentation protocole + wrap-up

- PUBLIC_FEED_SPEC.md §10 adversarial scenarios, §11 bootstrap,
  §12 security considerations
- verification.md + audit_plan S65 + compteurs CLAUDE.md + SPRINT_LOG.md

**Commit cible** : `docs(protocol): Sprint 64 Phase E — spec finalisee + wrap-up`
**Critere** : 3 sections ajoutees, fail-fast checklist verte.

---

## §6 Items carry/dette

### MANDATORY 3/3 (resolus dans ce sprint)

| Item | Reports | Phase S64 | Exit condition |
|---|---|---|---|
| F1 P2-VERSION-NOT-STORED | 3/3 | Phase A | version stockee DB + visible endpoint |
| F5 P2-IROH-INFRA-TIMEOUT | 3/3 | Phase A | subscribe timeout + test stabilite |

### Dette Phase B (resolue dans ce sprint)

| Item | Reports | Phase S64 | Exit condition |
|---|---|---|---|
| P2-FEED-SUBSCRIBE-JOINHANDLE | 2/3 | Phase B | JoinHandle join shutdown |
| P2-BACKFILL-6PLUS-TEST | 2/3 | Phase B | test integration backfill 6+ |
| P2-FEED-PUBLISH-ORPHAN | 2/3 | Phase B | retry/rollback DB/iroh-docs |
| P2-SUBSCRIBE-STREAM-BREAK | 2/3 | Phase B | reconnexion auto stream break |
| P2-PROCESS-FORMAT | herite | Phase B | exemption retroactive README.md |

### Carries reconduits S65

| Item | Reports | Justification |
|---|---|---|
| P2-A-1 rand blocker | exemption externe | upstream rand 0.9 non publie |
| P2-AUDIT-2 iroh transitives | exemption externe | iroh 1.0 non stable |
| P2-G-1 exe lock | monitoring | non-reproductible 5+ builds |
| P2-PROVENANCE-404-BRIDGE | 1/3 → 2/3 | enrichissement UX post-hardening |
| P2-BADGE-WORDING-PREMATURE | pre-existant S14 | post-verification live completes |
| P2-COMMIT-TITLE-FORMAT | 1/3 → 2/3 | process clarification post-hardening |
| P2-REVIEW-ORDER | 1/3 → 2/3 | process clarification post-hardening |
| P2-PYTHON-BLOCK-EXEMPTION | 1/3 → 2/3 | SKILL.md hygiene post-hardening |
| P2-FEED-INSERT-NO-AUTH-TIER | 2/3 → 3/3 MANDATORY S65 | auth tier feed |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE | 1/3 → 2/3 | defensive hardening |
| P2-PLAYWRIGHT-SPECS-STALE | 1/3 → 2/3 | Playwright specs post-hardening |
| P2-VERIFY-LOCAL-KEY-ONLY | 1/3 → 2/3 | cross-node verification |
| P2-COVERAGE-DEPLOY-E2E | 1/3 → 2/3 | test coverage |

### Attention 3/3 S65

**P2-FEED-INSERT-NO-AUTH-TIER** passera 3/3 au sprint 65 — devra
etre resolu dans le plan S65 (feed_insert handler verifie auth tier
avant insert).

---

## §7 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | CuratorVouched operation implementation | S65 | scope go-live, pas hardening |
| 2 | BuildQuorumReached operation implementation | S65 | scope go-live, pas hardening |
| 3 | Quarantine feed hot path | S65 | glue code anti-spam renforce post-pilote |
| 4 | Age witness gate feed admission | S65 | idem post-pilote |
| 5 | Multi-forge feed sync | S65+ | feature post-go-live |
| 6 | Feed format version bump | post-launch | pre-launch policy |
| 7 | CLI verify-release (expose verify_provenance) | S65 | UX go-live |
| 8 | VerificationDetail niveau 3 (full proof chain UI) | S65+ | UI enrichissement |
| 9 | Fuzzing cargo-fuzz/proptest | S65+ post-audit | audit prep, pas sprint |
| 10 | Docker compose test distribue | S65+ | infra overkill vs DaemonCluster |
| 11 | Interop externe parsers tiers | post-plan | hors roadmap |
| 12 | SearchManifestPublished feed | S66 reserve | optionnel Sprint 6 reserve |

---

## §8 Tracabilite scope

| Item S63 "What's NOT" | Sprint + Phase S64 |
|---|---|
| CuratorVouched operation | Reconduit S65 (#1) |
| BuildQuorumReached operation | Reconduit S65 (#2) |
| Quarantine feed | Reconduit S65 (#3) |
| Age witness gate feed | Reconduit S65 (#4) |
| Multi-forge feed sync | Reconduit S65+ (#5) |
| Feed format version bump | Reconduit post-launch (#6) |
| Go-live public + tag push + pilote | S65 Phase C (roadmap) |
| CLI verify-release | Reconduit S65 (#7) |
| VerificationDetail niveau 3 | Reconduit S65+ (#8) |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Tests adversariaux revelent bug dans verify_chain | Medium | High | Fix inline comme P1 inter-phase, pas scope cut |
| R2 | M13 migration casse provenance existantes | Low | High | Migration additive (colonne nullable), backward safe |
| R3 | subscribe timeout trop agressif → faux positifs | Medium | Medium | Timeout 30s heartbeat, pas bulk transfer |
| R4 | Nouveau noeud E2E instable (timing iroh-docs) | Medium | Medium | Gate SBFB_INTEGRATION + retry backoff existant |
| R5 | Phase B dette > 1 phase budget | Low | Low | 5 items ~150 LOC total, precedent S62 Phase A dette OK |
| R6 | PUBLIC_FEED_SPEC.md drift vs code | Low | Medium | Sections adversarial generees DEPUIS les noms de tests |
| R7 | P2-FEED-INSERT-NO-AUTH-TIER passe 3/3 sans resolution S64 | Certain | Low | Documente carry 3/3 → MANDATORY S65 explicite |

---

## §10 Audit gate pattern — rappel

Sprint 64 ouvre par Phase 0 audit gate S63 (DEJA JOUE — PASS).
Phase E produira `sprint65_audit_plan.md`.

---

## §11 Checkpoint de validation

1. D1 — Tests adversariaux deterministes (pas fuzzing) : OK pour
   la couverture ciblee hardening ? Pas de proptest/cargo-fuzz ?
2. D2 — Un seul test E2E nouveau noeud (pas 3+) : suffisant pour
   prouver le code path bootstrap ?
3. D3 — Version nullable (pas NOT NULL) dans M13 : acceptable
   pour les rows existantes sans version ?
4. D4 — Timeout 30s heartbeat : pas trop court pour les feeds
   longs (1000+ entries sync initial) ?
5. D5 — Enrichissement spec existante (pas rewrite) : 3 sections
   ajoutees suffisent pour la doc pre-audit ?
