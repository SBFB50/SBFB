# Sprint 65 — Kickoff (Contrat Public)

**Ecrit** : 2026-05-18 (post-audit gate S64 PASS `b7469ae`).
**Type** : **sprint impair** — pas de phase dette obligatoire.
Un item 3/3 (Regle 2) a traiter : P2-FEED-INSERT-NO-AUTH-TIER.
**Tip master d'entree** : `b7469ae` (audit findings S64 PASS
0 P0, 0 P1, 2 P2, 1 P3).
**Phase 0 audit Sprint 64** : **DEJA JOUE** — `b7469ae` PASS.
Aucun fix bloquant requis.
**Version archive** : v2.1 — Confiance + Factory Canari + RRV.
**Roadmap source** :
`.planning/roadmap_v3_public_trust_factory_babel_rrv.md`
Sprint 1 sur 11 (Arc 1 Fondations).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-16 (2 jours).
  3 triggers evalues (INCHANGES depuis S64) :

  1. **iroh > 0.98 + iroh-docs > 0.98** : iroh = "1.0.0-rc.0",
     iroh-docs = "0.99.0". Inchange depuis S64 kickoff.
     **Decision** : reste deferred (upgrade iroh 1.0 = sprint
     dedie Arc 1/2 Gate 1).

  2. **arti-client > 0.41** : arti-client = "0.42.0". Inchange.
     **Decision** : deferred. 0 CVE entre 0.41 et 0.42.

  3. **frost-ed25519 > 3.0** : frost-ed25519 = "3.0.0". Trigger
     INACTIF (on utilise 3.0.0, trigger > 3.x).

- **G9 codebase factual scan** (agent Explore independant) :

  1. **FeedEntry.op** : `PublicFeedOperation` enum avec
     `#[serde(tag = "op_type")]`, 2 variants (ReleasePublished,
     SourceBecameStale). DB stocke `op_type` + `payload` en
     colonnes separees (`public_feed` table). Schema SQL inchange
     par la migration raw-op — seuls les types Rust changent.
     `FeedEntryCanonical.op` aussi typed enum. JCS (RFC 8785)
     avec `DOMAIN_FEED_V1`. `FEED_FORMAT_VERSION = 1`.

  2. **Auth tier** : `TrustTier` enum spec dans
     `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` l.129 (Auto, ConfirmPrompt,
     BiometricGate) mais PAS implemente dans le code. Tous les
     endpoints sont T0 (bearer + Host + Origin). `feed_insert()`
     (`feed_sync.rs:445`) n'a aucun check auth tier — accepte
     tout caller avec bearer valide.

  3. **Badge wording** : "Verifie" dans Browse.tsx:259 et
     BrowsedProject.tsx:281. "Projets open source verifies" dans
     GpuConsentDialog.tsx:56-84. "open source verifie" dans
     PUBLISH_MODEL.md:128. Pas de "F-Droid" ni "de confiance" en
     UI. Protocol Explorer (app.js) : "Provenance verifiee".

  4. **Deploy→feed gap** : `deploy.rs:237-250` — AUCUN
     `feed_insert()` apres `publish_announcement()`. Deploy
     broadcast gossip + browse_aggregator mais ne touche PAS le
     feed public. De plus, `deploy.rs:72` accepte `http://`
     (check `starts_with("http")` sans `s`), tandis que
     `validate_feed_operation()` l.217 exige HTTPS.

  5. **Playwright** : `playwright.config.ts` existe, 0 fichier
     `.spec.ts`. Infrastructure zombie a supprimer.

- **ROADMAP_COMMITMENTS check (G7 Regle 3)** :
  - LT-1 Kudos-v2 : **CLOSED S59**.
  - LT-2 Radicle : **trigger PENDING** — tag v1.0 pose
    localement, pas pousse. Pas encore actif.
  - LT-3/LT-4/LT-5 : latent. 0 condition declenchee.
  - LT-6 : RESOLVED S32.
  - LT-7 : gate satisfait (Tier 1+2 S55 + Tier 3 S60).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 64 a livre le 4eme sprint de la roadmap v2.0 "Public
Verifiable Protocol Feed" sur le theme hardening public cible.
Le protocole feed est desormais : spec executable (S61) + sync
P2P (S62) + verification tiers (S63) + hardening adversarial
(S64). 15 tests adversariaux, 1 E2E nouveau noeud, 2 MANDATORY
fermes, PUBLIC_FEED_SPEC.md 12 sections completes.

Sprint 65 ouvre l'**Arc 1 Fondations** de la roadmap v3
"Confiance + Factory Canari + RRV". Theme : contrat public —
aligner chaque texte public avec ce que le code garantit
reellement. Prerequis absolu pour la credibilite publique.

### §1.2 Ancrage roadmap v3

Sprint 65 = "S65 Contrat Public" dans
`roadmap_v3_public_trust_factory_babel_rrv.md`. Arc 1 Fondations,
sprint 1 sur 2 (S65 + S66 Durabilite). Dependances aval :
S67 Factory Foundation (vocabulaire + raw-op + SBFB.json v2 spec),
S70 RRV LocalOnly (vocabulaire confiance + labels),
S71 Proof Cards (niveaux S65 = schema proof cards).

### §1.3 Compteurs tests entree (tip `b7469ae`)

| Suite | Count |
|---|---|
| Rust nextest | 1326 |
| Vitest | 265 |
| size-limit | 6/6 |
| **Total** | **~1597** |

### §1.4 Pre-launch protocol policy (rappel)

`*_FORMAT_VERSION` reste a 1 jusqu'au go-live public.
`FEED_FORMAT_VERSION` reste a 1 malgre la migration raw-op —
le format de l'enveloppe `FeedEntry` ne change pas, seul le
type Rust de `op` passe de typed enum a `serde_json::Value`.
Le JSON on-wire reste identique pour les ops connues.
`#[serde(default)]` reste legitime pour robustesse runtime.

---

## §2 Goal

Le sprint aligne chaque texte public (badges, labels, docs) avec
les garanties reelles du code, et prepare le terrain technique
pour les operations feed extensibles via raw-op. Aucun badge ne
devra sur-promettre. Une taxonomie formelle a 6 niveaux de
confiance sera le referentiel unique. Le feed acceptera les
operations inconnues (store + forward) pour l'extensibilite
post-S65. Le deploy inserera automatiquement une operation
ReleasePublished dans le feed. Les gates Factory seront specifiees
pour guider les sprints S67-S69.
**Critere SMART : toutes les rows fail-fast vertes au
verification.md, mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 64

**DEJA JOUE** : `b7469ae` PASS (0 P0, 0 P1, 2 P2, 1 P3).
- P2-THREAT-MODEL-FEED-SURFACE (1/3) : THREAT_MODEL.md ne
  couvre pas le feed — carry S66.
- P2-FEED-INSERT-NO-AUTH-TIER (3/3 MANDATORY) : confirme, traite
  Phase A.
- P3-AUDIT-PLAN-COUNTER-DISCREPANCY : nit cosmetic.
Aucun fix bloquant. Ouverture S65 autorisee.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Feed raw-op : PublicFeedOperation → serde_json::Value

**Retenu** : `FeedEntry.op` et `FeedEntryCanonical.op` deviennent
`serde_json::Value`. Le champ `op` en JSON est un objet avec
`"op_type": "ReleasePublished"` (discriminant, car
`#[serde(tag = "op_type")]` actuel) + champs payload. La
methode `try_parse_op()` tente le parse vers
`PublicFeedOperation` pour les ops connues, renvoie `Ok(None)`
pour les inconnues. `validate_feed_operation()` valide les ops
connues strictement, accepte les inconnues (store + forward sans
validation semantique). Canonical bytes sur `Value` est
deterministe via JCS (crate `jcs` existant, RFC 8785).

Le schema SQL de la table `public_feed` ne change PAS : colonnes
`op_type TEXT` + `payload TEXT` deja generiques. Seul le Rust
change : `FeedEntry.op: serde_json::Value` au lieu de
`FeedEntry.op: PublicFeedOperation`. La reconstruction
`rows_to_entries()` dans `feed_materializer.rs` utilise
`serde_json::from_str::<Value>` au lieu de
`serde_json::from_str::<PublicFeedOperation>`.

`FEED_FORMAT_VERSION` reste a 1. Le JSON on-wire est identique
pour les ops connues (backward compatible). Les ops inconnues
sont stockees, propagees et verifiees (hash-chain + signature)
sans etre interpretees.

**Rejete** :
- `#[serde(other)]` sur l'enum : serde ne supporte pas `other`
  sur les enums avec data (internally tagged). Requiert un
  wrapper `Unknown(Value)` qui complique pattern matching et
  canonical bytes.
- Champ `op_raw: String` + `op_parsed: Option<PublicFeedOperation>` :
  duplication, risque desync, canonical bytes ambigu (quel champ
  signer ?).
- Custom deserializer par variant : complexite O(n) par variant,
  non maintenable a 10+ variants post-S67.

**Implications code** : `public_feed.rs` (FeedEntry,
FeedEntryCanonical, validate_*, insert_*, tous tests),
`feed_materializer.rs` (rows_to_entries), `feed_sync.rs`
(ingest_doc_entry). Schema SQL inchange. `FEED_FORMAT_VERSION`
inchange.

### D2 — Taxonomie de confiance : 6 niveaux formels

**Retenu** : `docs/trust/TRUST_TAXONOMY.md` definit 6 niveaux
cumulatifs de confiance pour les apps SBFB :

| Niveau | Label | Assertion |
|---|---|---|
| N0 | Upload direct | Archive zip deployee sans depot source |
| N1 | Source lisible | Depot source public accessible |
| N2 | Provenance auto-attestee | SLSA L1 — build local, provenance signee Ed25519 |
| N3 | Signature verifiee live | Verification live par le daemon local |
| N4 | Build reproductible | Futur — build independant par tiers |
| N5 | Feed verifie hash-chain | Historique complet, hash-chain integre |

Plus 3 dimensions transversales : AGPL-3.0 (licence code SBFB),
Curator vouch (endorsement humain), Sandbox (iframe isolation).
Chaque niveau porte une **assertion positive** et une
**non-assertion explicite** (ce que le niveau ne garantit PAS).

**Rejete** :
- Taxonomie binaire verified/not-verified : perd la nuance et
  force des sur-promesses. Le badge "Verifie" actuel est un
  sur-engagement.
- Vocabulaire ad-hoc par badge : incoherence UI garantie entre
  Browse, BrowsedProject, GpuConsentDialog, Explorer.
- OpenSSF Scorecards (30+ checks) : trop granulaire pour un
  reseau P2P sans CI centralisee. Inadapte.

**Implications code** : `docs/trust/TRUST_TAXONOMY.md` (nouveau).
Les badges UI (Phase B) s'appuient sur les niveaux definis.

### D3 — Vocabulaire "source verifiable" : enforcement CI

**Retenu** : Script `scripts/scan-trust-wording.sh` (grep les
termes interdits dans UI + docs publics). Termes interdits :
- "Verifie" sans qualification (OK si "Signature verifiee live")
- "open source" hors code SBFB AGPL (les apps sont "source
  verifiable", pas "open source")
- "de confiance" dans un contexte automatique
- "Le code sur le reseau = le code du depot" (sur-promesse)

Les textes sont remplaces par la nomenclature
`TRUST_TAXONOMY.md` : "Provenance" (N2), "Signature verifiee"
(N3), "Source verifiable" (plateforme).

**Rejete** :
- Pas de script CI : regression wording garantie au premier
  refactor.
- Linter ESLint custom : overhead pour un grep de 10 lignes.
- Traduction integrale anglais : le projet est francophone.

**Implications code** : `scripts/scan-trust-wording.sh` (nouveau),
`examples/sbfb-explorer/app.js`, `web/src/pages/Browse.tsx`,
`web/src/pages/BrowsedProject.tsx`, `web/src/pages/Network.tsx`,
`web/src/components/GpuConsentDialog.tsx`,
`web/src/pages/Curators.tsx`,
`docs/architecture/PUBLISH_MODEL.md`.

### D4 — Deploy→feed wiring + auth tier feed

**Retenu** : Apres `publish_announcement()` dans `deploy.rs`,
insertion automatique d'une operation `ReleasePublished` dans
le feed via `insert_feed_operation()` +
`publish_feed_entry_to_docs()`. Rollback si insert echoue :
l'annonce gossip est fire-and-forget, mais le feed insert est
transactionnel. `repo_url` en `http://` rejete dans `deploy.rs`
(actuellement `starts_with("http")` sans le `s`).

Pour l'auth tier feed : le handler `feed_insert()` ajoute un
guard en debut de fonction qui verifie que le caller est autorise
a inserer. Strategie minimale S65 : ajouter un header
`X-SBFB-Feed-Internal` que seul le daemon set sur les appels
internes (deploy→feed, bridge→feed). Les appels HTTP externes
(browser direct) qui n'ont pas ce header sont rejetes 403. Ce
n'est PAS le T1 CONFIRM_PROMPT complet — c'est un
defense-in-depth qui distingue "code daemon interne" de "client
HTTP externe via loopback". Le T1 complet viendra post-S65.

**Rejete** :
- Feed insert manuel par le client : requiert un appel API
  supplementaire que personne ne fera.
- Insert asynchrone via queue : complexite pour un append <10ms.
- Tolerer http:// : incompatible avec la verification provenance
  qui requiert TLS pour le clone repo.
- T1 CONFIRM_PROMPT complet en S65 : requiert UI integration
  (nonce + TTL + prompt React), hors scope S65.

**Implications code** : `deploy.rs` (wiring + https reject),
`feed_sync.rs` (auth check), `http.rs` (header injection sur
route interne).

### D5 — Factory gates spec + SBFB.json v2 spec

**Retenu** : `docs/factory/FACTORY_GATES.md` definit 11 gates
sequentielles qu'une app doit franchir pour etre publiee via
Factory (implementation S67-S69). Referentiel documentaire, pas
de code S65.

| Gate | Nom | Role |
|---|---|---|
| FG0 | Classification | Type d'app (static HTML, React, Pyodide, WASM) |
| FG1 | Scope | Permissions requises (bridge methods) |
| FG2 | Template | Scaffold genere depuis template |
| FG3 | Manifest | SBFB.json v2 valide |
| FG4 | Diff | Review des changements |
| FG5 | Sandbox | Test iframe sandbox |
| FG6 | Secrets/deps | Scan securite |
| FG7 | Preview | Preview live avant deploy |
| FG8 | Provenance | SLSA L1 signature |
| FG9 | Publish | Deploy sur le reseau |
| FG10 | Review | Curator review post-publish |

`docs/protocol/SBFB_JSON_V2.md` definit le manifest enrichi
(schema_version, name, display_name, description, category,
license, lang, bridge.methods, bridge.events, tech.type,
tech.build_command, requirements). Retro-compatible v1.

**Rejete** :
- Gates dynamiques configurables par projet : overhead config
  pour un flow lineaire.
- Gates integrees dans le code S65 : couplage docs/code premature
  (implementation S67).
- Pas de spec : Factory devient une boite noire.

**Implications code** : `docs/factory/FACTORY_GATES.md` (nouveau),
`docs/protocol/SBFB_JSON_V2.md` (nouveau). Pas de code Rust.

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ✅, D3 ✅, D4 ⚠️, D5 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 5).

D1 angle mort : absence de comparaison CBOR/MessagePack. Decision :
acknowledge — JCS (RFC 8785) n'existe que pour JSON, les formats
binaires (bincode/CBOR/MessagePack) n'ont pas de spec canonical.
Note ajoutee au commit body Phase A.

D2 angle mort : rejet OpenSSF Scorecard pas explicitement source.
Decision : adjust — ajouter section "Why not OpenSSF Scorecard"
dans TRUST_TAXONOMY.md (Scorecard assume CI/CD centralise).

D3 angle mort : faux positifs scan-trust-wording.sh. Decision :
acknowledge — Phase C incluera validation 0 false positives sur
codebase existante avant activation CI.

D4 ⚠️ : le header `X-SBFB-Feed-Internal` n'est pas
cryptographiquement lie au daemon. Un process local qui connait
le header peut l'envoyer. Pas de rotation, pas de nonce.
Decision : adjust — (1) documenter dans
LOOPBACK_ENDPOINTS_TRUST_TIERS.md que feed_insert() = T0
temporaire, header != credential crypto ; (2) le vrai T1
(CONFIRM_PROMPT avec UI nonce + HMAC) sera implemente post-pilote
S69 ; (3) limitation acceptable pre-launch (aucun process tiers
sur la machine dev, loopback bearer deja requis).

D5 angle mort : strategie upgrade v1→v2 SBFB.json non detaillee.
Decision : adjust — Phase D ajoute section versioning strategy
dans SBFB_JSON_V2.md (schema_version parsing, #[serde(default)],
forward-compat).

---

## §5 Plan Phase outline A..D

### Phase A — Securite feed + raw-op migration + TRUST_TAXONOMY

Resout P2-FEED-INSERT-NO-AUTH-TIER (3/3 MANDATORY). Migre le feed
vers raw-op. Ecrit les documents fondateurs (TRUST_TAXONOMY.md,
COMMONS.md). Wire deploy→feed. Le gros du travail technique.

- Fix P2-FEED-INSERT-NO-AUTH-TIER : auth tier guard dans
  `feed_insert()` (X-SBFB-Feed-Internal header check)
- Fix P2-VERIFY-ENTRY-VERSION-GUARD : `verify_entry()` rejette
  version != FEED_FORMAT_VERSION
- Migration raw-op : FeedEntry.op → serde_json::Value + try_parse_op
- PUBLIC_FEED_SPEC.md §9 update : forward compatibility raw-op
- TRUST_TAXONOMY.md : 6 niveaux + 3 dimensions transversales
- COMMONS.md : convention anti-capture AGPL
- Wiring deploy→feed : ReleasePublished auto-insert
- deploy.rs http→https reject
- Carry absorbe : P2-COVERAGE-DEPLOY-E2E (test E2E deploy→feed)
- Tests : auth tier reject, version guard, unknown op roundtrip,
  canonical bytes Value vs typed, deploy→ReleasePublished,
  deploy http:// rejected, deploy failure→no feed entry

**Commit cible** : `feat(feed+trust): Sprint 65 Phase A — raw-op migration + auth tier + TRUST_TAXONOMY`
**Critere** : feed_insert rejette sans header, verify_entry rejette
mauvaise version, unknown ops stockees+verifiees, deploy→feed wire
vert, TRUST_TAXONOMY.md + COMMONS.md presents.

### Phase B — Migration badges UI

Migre tous les badges/labels UI vers la nomenclature
TRUST_TAXONOMY.md. Core du sprint "contrat public".

- Browse.tsx : "Verifie" + ShieldCheck → "Provenance" + FileCheck
- BrowsedProject.tsx : idem + etat dynamique post-verification
- GpuConsentDialog.tsx L2 : "Projets open source verifies" →
  "Apps deployees depuis un depot public (provenance auto-attestee)"
- Network.tsx : "L2 — Open source" → "L2 — Depot public"
- Curators.tsx : "curator de confiance" → "curator"
- Protocol Explorer (app.js) : corrections textes (auto-attestation,
  "inspire par F-Droid", "source verifiable")
- PUBLISH_MODEL.md : "open source verifie" → "Release avec
  provenance auto-attestee"
- Mise a jour tests existants (BrowsedProject.test.tsx,
  VerificationDetail.test.tsx)

**Commit cible** : `feat(trust): Sprint 65 Phase B — badges UI migration vocabulaire`
**Critere** : 0 occurrence "Verifie" sans qualification dans l'UI,
0 "open source" hors AGPL context, scan-en-strings.sh clean.

### Phase C — Badge dynamique post-verification + scan-trust-wording

Badge "Provenance" passe dynamiquement a "Signature verifiee"
(vert) ou "Echoue" (rouge) apres verification API automatique.
Script CI non-regression wording.

- Appel `provenance_verify` auto a l'ouverture de BrowsedProject
- Etat transitoire : "Verification..." pendant l'appel API
- Cache resultat session
- Script scan-trust-wording.sh (grep termes interdits)
- Tests Vitest etat dynamique badge

**Commit cible** : `feat(trust): Sprint 65 Phase C — badge dynamique + scan-trust-wording`
**Critere** : badge change dynamiquement apres verification,
scan-trust-wording.sh passe sans faux positif.

### Phase D — Gates Factory spec + dette pair + wrap-up

Spec documentaire pour Factory S67-S69. Dette pair process.
Wrap-up sprint.

- FACTORY_GATES.md : 11 gates FG0-FG10
- SBFB_JSON_V2.md : manifest app v2 spec
- Fix P2-COMMIT-TITLE-FORMAT : clarification process
- Fix P2-REVIEW-ORDER : doc amendment
- Reclassification P2-PYTHON-BLOCK-EXEMPTION : resolved by pivot S50
- Fix P2-EXPLORER-ESCAPE-SINGLE-QUOTE : 1 LOC escape `'`
- P2-PLAYWRIGHT-SPECS-STALE : suppression 12 fichiers zombies
- verification.md + audit_plan S66 + compteurs CLAUDE.md +
  SPRINT_LOG.md

**Commit cible** : `docs(factory+trust): Sprint 65 Phase D — gates Factory + dette pair + wrap-up`
**Critere** : FACTORY_GATES.md + SBFB_JSON_V2.md presents, 5
items dette fermes, fail-fast checklist verte.

---

## §6 Items carry/dette

### Items 3/3 (traitement Sprint 65)

| Item | Reports | Phase S65 | Exit condition |
|---|---|---|---|
| P2-FEED-INSERT-NO-AUTH-TIER | 3/3 | Phase A | feed_insert rejette sans header interne |

### Carry absorbes S65

| Item | Reports | Phase S65 | Exit condition |
|---|---|---|---|
| P2-VERIFY-ENTRY-VERSION-GUARD | 1/3 | Phase A | verify_entry rejette version != FEED_FORMAT_VERSION |
| P2-BADGE-WORDING-PREMATURE | pre-S14 | Phase B | 0 badge "Verifie" sans qualification |
| P2-COMMIT-TITLE-FORMAT | 2/3 | Phase D | process clarification |
| P2-REVIEW-ORDER | 2/3 | Phase D | doc amendment |
| P2-PYTHON-BLOCK-EXEMPTION | 2/3 | Phase D | reclassifie resolved (pivot S50 supprime Python) |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE | 2/3 | Phase D | escapeAttr single quote |
| P2-PLAYWRIGHT-SPECS-STALE | 2/3 | Phase D | suppression fichiers zombies |
| P2-COVERAGE-DEPLOY-E2E | 2/3 | Phase A | test E2E deploy→feed roundtrip |

### Carries reconduits S66

| Item | Reports | Justification |
|---|---|---|
| P2-A-1 rand blocker | exemption externe | upstream rand 0.9 non publie |
| P2-AUDIT-2 iroh transitives | exemption externe | iroh 1.0 non stable |
| P2-G-1 exe lock | monitoring | non-reproductible |
| P2-PROVENANCE-404-BRIDGE | 2/3 → 3/3 MANDATORY S66 | enrichissement UX |
| P2-VERIFY-LOCAL-KEY-ONLY | 2/3 → 3/3 MANDATORY S66 | cross-node verification |
| P2-FEED-JOIN-HANDLE-LEAK | 1/3 → 2/3 | feed reconnect |
| P2-ORPHAN-REPUBLISH-RECOVERY | 1/3 → 2/3 | feed resilience |
| P2-THREAT-MODEL-FEED-SURFACE | 1/3 (nouveau S64 audit) | doc gap THREAT_MODEL.md |

### Attention 3/3 S66

**P2-PROVENANCE-404-BRIDGE** et **P2-VERIFY-LOCAL-KEY-ONLY**
passeront 3/3 au Sprint 66 — devront etre resolus dans le plan
S66.

---

## §7 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | CuratorVouched/CuratorDisendorsed implementation | S67 | Factory Foundation, pas contrat public |
| 2 | BuildQuorumReached feed implementation | S67+ | idem |
| 3 | Quarantine feed hot path | S67+ | glue code anti-spam post-Factory |
| 4 | Age witness gate feed admission | S67+ | idem |
| 5 | T1 CONFIRM_PROMPT complet (UI nonce) | post-pilote S69 | requiert integration UI React + nonce |
| 6 | SBFB.json v2 code implementation | S67 Phase A | S65 = spec seulement |
| 7 | node_id deprecation dans deploy.rs | S67 Phase A | S65 = spec seulement |
| 8 | Factory template scaffold | S67 Phase B+ | S65 = gates spec seulement |
| 9 | Fuzzing cargo-fuzz/proptest | post-audit | audit prep, pas sprint |
| 10 | CLI verify-release | S66+ | UX enrichissement |
| 11 | VerificationDetail niveau 3 | S66+ | UI enrichissement |
| 12 | Playwright E2E tests re-ecriture | S69 | suppression S65, re-ecriture post-Factory |
| 13 | THREAT_MODEL.md section feed | S66 | doc gap identifie audit S64, carry 1/3 |
| 14 | Feed format version bump | post-launch | pre-launch policy |

---

## §8 Tracabilite scope

| Item S64 "What's NOT" | Sprint + Phase S65 |
|---|---|
| CuratorVouched implementation | Reconduit S67 (#1) |
| BuildQuorumReached implementation | Reconduit S67+ (#2) |
| Quarantine feed hot path | Reconduit S67+ (#3) |
| Age witness gate feed | Reconduit S67+ (#4) |
| Multi-forge feed sync | Reconduit S67+ (roadmap v3) |
| Feed format version bump | Reconduit post-launch (#14) |
| CLI verify-release | Reconduit S66+ (#10) |
| VerificationDetail niveau 3 | Reconduit S66+ (#11) |
| Fuzzing cargo-fuzz | Reconduit post-audit (#9) |
| Docker compose test | Supprime (subsome par DaemonCluster E2E) |
| Interop externe parsers | Supprime (hors roadmap v3) |
| SearchManifestPublished feed | S72 (roadmap v3 Arc 3) |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Raw-op migration casse canonical bytes (hash mismatch) | Medium | High | Test "canonical bytes Value vs typed struct identical", FEED_FORMAT_VERSION inchange |
| R2 | Deploy→feed wiring insert echoue (rollback complexe) | Low | Medium | Insert APRES publish_announcement (gossip fire-and-forget), rollback = pas d'entry feed |
| R3 | scan-trust-wording.sh faux positifs sur mots legitimes | Medium | Low | Whitelist regles (ex: "verifie" OK si "Signature verifiee") |
| R4 | Badge dynamique provenance_verify lent (timeout UI) | Medium | Medium | Etat transitoire "Verification..." + cache session |
| R5 | Phase A trop large (8 livrables) | Medium | Medium | Decoupler si besoin : raw-op + auth tier = core, TRUST_TAXONOMY + COMMONS + wiring = docs + glue |
| R6 | Header X-SBFB-Feed-Internal forgeable par process local | Low (pre-launch) | Low | Documente comme limitation, T1 vrai post-pilote S69 |
| R7 | Textes francais UI regressions scan-en-strings.sh | Low | Low | Execution scan-en-strings.sh en meme temps que scan-trust-wording.sh |

---

## §10 Audit gate pattern — rappel

Sprint 65 ouvre par Phase 0 audit gate S64 (DEJA JOUE — PASS).
Phase D produira `sprint66_audit_plan.md`.

---

## §11 Checkpoint de validation

1. D1 — Raw-op via serde_json::Value (pas enum wrapper) : OK pour
   la determinisme JCS canonique ? La cle `op_type` est-elle
   toujours le discriminant ?
2. D2 — 6 niveaux de confiance : la progression N0→N5 est-elle
   coherente avec le deploy pipeline reel ?
3. D3 — scan-trust-wording.sh en CI : risque de faux positifs
   acceptable ?
4. D4 — Header interne (pas T1 complet) : defense-in-depth
   suffisante pre-launch ?
5. D5 — Spec-only pour Factory gates : les S67-S69 ont-ils assez
   de contexte pour implementer ?
