# Sprint 61 — Kickoff (spec executable + feed local rejouable)

**Ecrit** : 2026-05-13 (post-audit gate S60 PASS `32c07e2`).
**Type** : **sprint impair** — pas de phase dette obligatoire.
**Tip master d'entree** : `32c07e2` (migration S60 → archive/v1.2/).
**Phase 0 audit Sprint 60** : **DEJA JOUE** — `f02a600` PASS
(0 P0, 0 P1, 4 P2, 2 P3). Aucun fix bloquant requis.
**Version archive** : v2.0 — Public Verifiable Protocol Feed.
**Roadmap source** : `.planning/research/public_verifiable_feed_roadmap.md`
Sprint 1 sur 6 (5+1 reserve).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-12.
  7 fichiers avec triggers_revalidate. 3 triggers a evaluer :

  1. **iroh > 0.98** : `cargo search iroh` retourne `iroh = "1.0.0-rc.0"`
     (inchange depuis S60). Trigger ACTIF mais RC pre-release, pas stable.
     **Decision** : deja evalue et defere S60. Rester sur iroh 0.98.
     Upgrade iroh 1.0 stable = sprint dedie post-feed.

  2. **arti-client > 0.41** : `cargo search arti-client` retourne
     `arti-client = "0.42.0"`. Trigger **ACTIF** — 0.42 > 0.41 pin.
     **Evaluation** : feature `tor` sur nexus-core-rs est un gate
     optionnel (`dep:arti-client`, `tor_transport.rs`). Non cable en
     production. Sprint 61 = spec feed, aucune interaction Tor.
     **Decision** : evalue et defere. Upgrade arti-client 0.42 = sprint
     dedie post-feed ou avec upgrade iroh. 0 CVE signale entre 0.41
     et 0.42. Trigger documente.

  3. **wasmtime LTS bump** : `cargo search wasmtime` retourne
     `wasmtime = "44.0.1"`. wasmtime n'est pas une dep directe du
     projet (zone rouge R-wasmtime-cve P0 pour usage futur runtime
     isolation). Informatif. Pas d'action.

- **BLAKE3 hash-chain** : pattern existant dans kudos_ledger.rs
  (S36+S37). BLAKE3 digest 256-bit, prev_hash + entry_hash,
  verify_chain() read-only. Reutilisable directement.

- **JCS/RFC8785 canonical bytes** : 14 domaines dans canonical.rs.
  Pattern domain separation Ed25519 : `DOMAIN_{name}_V1` prefix,
  `canonical_bytes_jcs()` serde_jcs. Ajouter `DOMAIN_FEED_V1`.

- **SQLite migrations** : 8 migrations existantes (M1-M8). Pattern
  `db.rs` `MIGRATIONS` array. M9 sera la table `public_feed`.

- **Roadmap post-v1.0** : `.planning/research/public_verifiable_feed_roadmap.md`
  valide PO 2026-05-13. Sprint 1 = spec + feed local. 4 phases A-D.

- **ROADMAP_COMMITMENTS check (G7 Regle 3)** :
  - LT-1 Kudos-v2 : **CLOSED S59**. Pas d'action.
  - LT-2 Radicle : **trigger PENDING** — tag v1.0 pose localement
    mais pas pousse vers origin. Le push est prevu Sprint 5 (go-live
    public). LT-2 reste PENDING, pas carry actif.
  - LT-3/LT-4/LT-5 : latent. 0 condition declenchee.
  - LT-6 : RESOLVED S32.
  - LT-7 : **gate satisfait** (Tier 1+2 S55 + Tier 3 S60). Worker
    quorum E2E carry post-tag. Pas d'action S61.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 60 CLOSED + audit PASS (`32c07e2`). **Tag v1.0 pose** sur
master (local). Produit **end user ready** : 3 installers (Windows
NSIS 16.81 MB + Linux .deb 26.04 MB + macOS .dmg 23.32 MB), tray
icon Windows, P2P infra validee 3 machines WAN, 2 apps SBFB
(Protocol Explorer + Ideas Hub), CI Woodpecker + GHA.

Le reseau fonctionne pour deployer, browser, executer. Mais un
tiers ne peut pas encore rejouer une histoire complete "ce projet
a ete publie depuis ce repo, cette release pointe vers cette
provenance, ce build a atteint quorum" — il n'y a pas de feed
protocolaire.

### §1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP last_validated 2026-05-12. 3 triggers evalues
(voir §Sources ci-dessus) : iroh 1.0.0-rc.0 defere, arti-client
0.42 defere, wasmtime informatif. Aucune action requise pour S61.

### §1.3 Compteurs tests entree (tip `32c07e2`)

| Suite | Compte |
|---|---|
| Rust nextest | 1259 |
| Rust doctests | 6 (1 ignored) |
| Vitest | 258 |
| Playwright | 0 (global-setup fail pre-existant S50) |
| size-limit | 6/6 |
| **Total** | **~1523** |

### §1.4 Post-launch protocol policy

**Le tag v1.0 est pose.** La politique post-v1.0 s'applique
desormais aux wire formats :

- Chaque break du format bump `*_FORMAT_VERSION`
- Chaque decoder accepte un range de versions
- Chaque ajout de champ porte un `#[serde(default)]` pour compat
  ascendante
- Les tests "legacy decode" sont maintenant **legitimes** (ils
  protegent des vrais clients)

Le Public Feed format demarre a `FEED_FORMAT_VERSION = 1` sous
ce regime. Premier format du projet concu nativement post-v1.0.

---

## §2 Goal en une phrase

SBFB dispose d'un feed local append-only signe qui enregistre les
evenements publics du reseau (`ReleasePublished`, `SourceBecameStale`),
les rejoue depuis zero, et reconstruit une vue registre coherente
— posant la fondation pour la sync P2P Sprint 2.

**Critere SMART : toutes les rows fail-fast du verification.md
vertes, mesure binaire au Phase D wrap-up.** Le verification.md
§Fail-fast checklist est le critere mesurable du sprint.

---

## §3 Phase 0 — Audit gate du sprint precedent

Sprint 60 audit PASS (`f02a600`). 0 P0, 0 P1. 4 P2 confirmes
des phase reviews (P2-IMAGE-DEP, P2-NSIS-UNINSTALL, P2-G-1,
P2-PLAYWRIGHT-REFACTOR) — tous carries S61 documentes. 2 P3
cosmetic sans action. Sprint 61 Phase A demarre directement.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Feed operation type system

**Retenu** : enum Rust `PublicFeedOperation` avec 2 variants
Sprint 1 : `ReleasePublished` et `SourceBecameStale`. Les
variants futurs (`CuratorVouched`, `BuildQuorumReached`) sont
definis dans la spec Phase A mais implementes Sprint 2+.

Chaque operation porte : `op_type` (discriminant string),
`project_id` (NodeId hex), `payload` (operation-specific struct),
`author_pubkey` (Ed25519 public key), `signature` (Ed25519 sur
canonical bytes JCS), `timestamp` (u64 seconds epoch).

**Rejete** :
- Catch-all JSON blob : pas de type safety, impossible de faire
  exhaustive match. Pas de validation a la compilation.
- Trait object per operation : surconception pour 2 variants.
  Pattern enum est celui de `SecurityEvent` (nexus-events-core)
  et de `GossipCmd` (nexus-shell-daemon-core).

**Implications code** : nouveau fichier
`crates/nexus-coordinator-rs/src/public_feed.rs`.

### D2 — Feed storage backend

**Retenu** : table SQLite `public_feed` dans coordinator.db.
Migration M9. Schema append-only :

```sql
CREATE TABLE public_feed (
    seq       INTEGER PRIMARY KEY AUTOINCREMENT,
    op_type   TEXT NOT NULL,
    payload   BLOB NOT NULL,  -- JCS canonical bytes
    author    TEXT NOT NULL,   -- Ed25519 pubkey hex
    signature BLOB NOT NULL,   -- Ed25519 signature
    entry_hash BLOB NOT NULL,  -- BLAKE3(prev_hash || payload)
    prev_hash  BLOB NOT NULL,  -- BLAKE3 hash of previous entry (zeros for genesis)
    created_at INTEGER NOT NULL -- unix timestamp seconds
);
```

Pattern : gossip_outbox M6 (SQLite append, load_outbox/insert
au boot) + kudos_ledger hash-chain (prev_hash/entry_hash BLAKE3).

**Rejete** :
- iroh-docs comme stockage local : overkill pour Sprint 1 (feed
  local-only). iroh-docs = Sprint 2 (sync P2P). Separer storage
  local du transport P2P.
- Fichier SQLite separe : fragmentation inutile, coordinator.db
  gere deja tout l'etat (8 migrations). Single WAL = single
  contention point maitrise.
- Fichier JSONL append-only : pas de cursor efficace, pas de
  query par op_type, pas de replay partiel. SQLite gagne sur
  toute l'utilisation prevue.

**Implications code** : `crates/nexus-coordinator-rs/src/db.rs`
(migration M9).

### D3 — Hash-chain + signature scheme

**Retenu** : BLAKE3 hash-chain (coherent kudos_ledger.rs) +
Ed25519 signature avec domain separation `DOMAIN_FEED_V1`.
Serialization JCS/RFC8785 (coherent 14 domaines canonical.rs).

Hash-chain :
- Genesis : `prev_hash = [0u8; 32]`
- Entry N : `entry_hash = BLAKE3(prev_hash || canonical_bytes)`
- Verification : `verify_chain()` relit tout depuis genesis,
  recalcule chaque hash, verifie chaque signature

Signature :
- Canonical bytes via `serde_jcs::to_vec()` (RFC8785)
- Domain prefix : `DOMAIN_FEED_V1` + `b'\0'` + canonical_bytes
- Ed25519 sign/verify (ed25519-dalek, deja en dep)

**Rejete** :
- SHA-256 hash-chain : incoherent avec kudos_ledger BLAKE3 et
  le reste du codebase (tous les hash-chains utilisent BLAKE3)
- HMAC-based : complexite inutile pour un feed public append-only.
  HMAC protege l'integrite avec une cle partagee — pas le modele
  ici (signature par auteur public)
- Sans signature par entree : impossible d'attribuer les operations
  a leurs auteurs. Requis pour la verification tiers (Sprint 3)

**Implications code** :
- `crates/nexus-core-rs/src/canonical.rs` : nouveau domaine
  `DOMAIN_FEED_V1`
- `crates/nexus-coordinator-rs/src/public_feed.rs` : hash-chain
  logic

### D4 — Spec document format + location

**Retenu** : `docs/protocol/PUBLIC_FEED_SPEC.md` — spec executable
avec types Rust inline, exemples JSON, vectors de test, regles de
replay. Premier document du repertoire `docs/protocol/`. Post-v1.0
versioning policy explicite dans la spec (chaque break bumpe
`FEED_FORMAT_VERSION`).

Sections spec :
1. Overview + design goals
2. Operation types (enum + champs)
3. Canonical serialization (JCS/RFC8785 + domain separation)
4. Hash-chain construction (BLAKE3, genesis, verification)
5. Replay rules (ordering, idempotence, state transitions)
6. Cursor format (seq + entry_hash checkpoint)
7. Test vectors (JSON + expected hashes)
8. Versioning policy

**Rejete** :
- In-code doc only : non consommable par implementeurs tiers.
  L'objectif du roadmap est "credibilite publique protocole
  verifiable" — la spec doit etre lisible independamment du code
- Protobuf/JSON Schema : premature pour un seul implementeur.
  Le schema est implicite dans les types Rust + la spec markdown.
  Protobuf viendrait Sprint 4+ si demande interop
- OpenAPI spec pour les operations : le feed n'est pas une API
  HTTP, c'est un log append-only

**Implications code** : nouveau repertoire `docs/protocol/`,
nouveau fichier `PUBLIC_FEED_SPEC.md`.

### D5 — Integration boundary avec BrowseAggregator

**Retenu** : feed comme **source supplementaire** pour le
browse, pas comme remplacement. `PublicRegistryView` est
materialisee depuis le feed (Phase C). Le `BrowseAggregator`
existant continue de fonctionner pour l'etat live (gossip).
L'integration est minimale : le feed fournit une vue "historique
verifiable" complementaire a la vue "live gossip".

La materialisation reconstruit une vue depuis le feed :
- Quels projets sont publies (ReleasePublished)
- Lesquels ont un source stale (SourceBecameStale)

Le cursor persiste le dernier seq traite pour reprise apres
interruption.

**Rejete** :
- Remplacer BrowseAggregator : trop risque, browse fonctionne
  bien pour l'etat live. Le feed est append-only (historique),
  browse est ephemere (etat courant). Deux concerns differents
- Feed ecrit DANS BrowseAggregator directement : inversion du
  flux. Le feed est la source de verite, browse en consomme une
  projection
- Pas d'integration Sprint 1 : manquerait le critere "rejouable"
  — un feed sans materialisation ne prouve pas qu'il est
  exploitable

**Implications code** :
- `crates/nexus-coordinator-rs/src/feed_materializer.rs` (nouveau)
- `crates/nexus-shell-daemon-core/` (integration minimale)

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ✅, D3 ✅, D4 ✅, D5 ✅.
Rigor signal G4 satisfait (2 ⚠️ sur 5, 0 ❌).

D2 ⚠️ : BLOB vs TEXT hash encoding dans M9 schema. Decision :
Phase B clarifiera. Le schema kickoff utilise BLOB (32 bytes
compact, plus efficient pour hash comparison). Si le pattern
kudos_ledger (TEXT hex) est plus coherent avec le reste du
codebase, Phase B alignera. Pas de changement architectural.

D4 ⚠️ : p2panda research (65-75% narrative) a absorber dans la
spec. Decision : Phase A absorbera les conclusions pertinentes
(operation lifecycle, replay rules, is_open_source validation)
depuis `p2panda_public_protocol_briques.md`. Les items non
pertinents pour Sprint 1 (sync P2P, anti-spam) sont scope cuts
Sprint 2.

---

## §5 Plan Phase outline A..D

### Phase A — Spec executable + types Rust

Spec `docs/protocol/PUBLIC_FEED_SPEC.md` formalisee. Types Rust
`PublicFeedOperation` enum, `FeedEntry` struct, domaine
`DOMAIN_FEED_V1` dans canonical.rs. Serde + canonical bytes JCS.
`FEED_FORMAT_VERSION = 1`. Test vectors JSON dans la spec.
Politique versioning post-v1.0 explicite.

**Commit cible** : `feat(feed): Sprint 61 Phase A — spec executable + types PublicFeedOperation`

### Phase B — Feed local append-only store

Migration M9 table `public_feed`. `FeedStore` struct : insert,
replay_all, verify_chain, get_cursor. Hash-chain BLAKE3
(prev_hash/entry_hash). Signature Ed25519 par entree.
insert_operation() persiste en SQLite + maintient le hash-chain.
replay_from_zero() relit toute la table, verifie chaque hash et
signature.

**Commit cible** : `feat(feed): Sprint 61 Phase B — feed local SQLite append-only + hash-chain BLAKE3`

### Phase C — Materialisation + cursor

`FeedMaterializer` lit le feed et produit `PublicRegistryView` :
set de projets avec leur statut derive des operations. Cursor
persiste (dernier seq traite) pour reprise apres interruption.
Integration minimale : `PublicRegistryView` est requetable
independamment du BrowseAggregator.

**Commit cible** : `feat(feed): Sprint 61 Phase C — materialisation PublicRegistryView + cursor persistant`

### Phase D — Tests + wrap-up

Hash-chain integrity (tamper entry → detect). Transitions d'etat
(ReleasePublished → SourceBecameStale meme projet). Local Draft
ne peut pas apparaitre comme Verified Release dans la vue. Corruption
detection. Cursor restart (materialiser depuis seq=0, puis depuis
checkpoint, verifier meme resultat). Verification + audit_plan S62.

**Commit cible** : `feat(feed): Sprint 61 Phase D — tests hash-chain + transitions + cursor restart`

---

## §6 Items carry/dette

| Item | Compteur S61 | Classification | Justification |
|---|---|---|---|
| P2-A-1 rand blocker upstream | 21+/3 | exemption externe renouvelee | blocker upstream rand 0.9 crate. Pas de resolution possible cote SBFB. Exemption permanente. |
| P2-AUDIT-2 iroh transitives pre-release | herite | exemption externe renouvelee | herite du pin iroh 0.98. Tant qu'on pin 0.98, les transitives restent pre-release. Upgrade iroh 1.0 = sprint dedie. |
| P2-NSIS-UNINSTALL multi-binary | 1/3 → 2/3 | carry confirme S62 | residuel installer NSIS. Pas sur le chemin critique du feed. |
| P2-IMAGE-DEP image 0.25 footprint | 1/3 → 2/3 | carry confirme S62 | ~15 transitives tray icon. Pas sur le chemin critique du feed. |
| P2-G-1 exe lock intermittent | reouvert | carry confirme S62 | dev-env intermittent. Monitoring continu. |
| P2-PLAYWRIGHT-REFACTOR | 1/3 → 2/3 | carry confirme S62 | global-setup pyproject.toml. Refactor PW post-feed. |

**Items a 3/3 MANDATORY** : aucun. Aucun carry a 3 reports
consecutifs.

**Regle 1 dette sprint pair** : S61 est impair — pas de phase
dette obligatoire.

---

## §7 Scope cuts (12 items)

| # | Item | Sprint cible |
|---|---|---|
| 1 | Sync P2P durable (iroh-docs feed) | Sprint 62 (roadmap S2) |
| 2 | Anti-spam feed (PoW + rate-limit + quarantine branchement) | Sprint 62 Phase D |
| 3 | CuratorVouched operation implementation | Sprint 62+ |
| 4 | BuildQuorumReached operation implementation | Sprint 62+ |
| 5 | Endpoint HTTP verify-release | Sprint 63 (roadmap S3) |
| 6 | Bridge methods getProvenanceRecord/verifyRelease | Sprint 63 |
| 7 | UI proof-chain composant VerificationDetail | Sprint 63 |
| 8 | Tests adversariaux feed (corruption, spam, forge) | Sprint 64 (roadmap S4) |
| 9 | Go-live public + tag push + pilote externe | Sprint 65 (roadmap S5) |
| 10 | AppImage Linux | post-roadmap (linuxdeploy FUSE) |
| 11 | Interop externe / clients alternatifs | post-roadmap |
| 12 | Audit tiers formel (Trail of Bits RFP) | post-roadmap |

---

## §8 Tracabilite scope

Premier sprint du roadmap post-v1.0. Pas de "What's NOT" Sprint 60
a tracker — les scope cuts S60 etaient tous sur le theme installer/
tray (livres ou diferes post-v1.0). Le scope S61 vient directement
du roadmap `.planning/research/public_verifiable_feed_roadmap.md`
Sprint 1.

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | canonical.rs devient trop gros avec le 15e domaine | Low | Low | Le domaine est 3 lignes (constante + doc). canonical.rs est deja modularise par constantes |
| R2 | Hash-chain BLAKE3 incompatible avec kudos_ledger pattern | Low | Medium | Meme pattern exact (prev_hash/entry_hash). verify_chain() deja prouve |
| R3 | Migration M9 conflit avec M8 (storage_namespaces) | Low | Low | M9 est une nouvelle table independante. Pas de schema overlap |
| R4 | FeedMaterializer trop couple a BrowseAggregator | Medium | Medium | D5 fige l'integration comme "source supplementaire", pas remplacement. FeedMaterializer produit sa propre vue |
| R5 | Spec trop verbose, ralentit Phase A | Medium | Low | Limiter la spec aux 2 operations Sprint 1. Les operations Sprint 2+ sont mentionnees mais pas specifiees en detail |

---

## §10 Audit gate pattern — rappel

Phase 0 jouee (Sprint 60 audit PASS). Phase D devra produire
`sprint62_audit_plan.md` pour le prochain sprint. L'audit gate
reste actif a chaque transition de sprint.

---

## §11 Checkpoint de validation

1. Les D1..D5 gelees sont-elles coherentes avec le roadmap
   post-v1.0 Sprint 1 ?
2. Le scope "2 operations seulement" est-il suffisant pour
   prouver le feed rejouable ?
3. L'integration BrowseAggregator (D5) est-elle au bon niveau
   de couplage (supplementaire, pas remplacement) ?
4. La politique versioning post-v1.0 (D4) est-elle claire pour
   le premier format concu nativement sous ce regime ?
5. Les 6 carries sont-ils correctement re-confirmes sans action
   S61 ?
