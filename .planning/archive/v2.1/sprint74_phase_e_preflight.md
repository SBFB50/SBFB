# Sprint 74 Phase E Preflight

Date: 2026-06-08
HEAD: `4c1acc5` (Phase D landed: keep_online M18 + blob tag + boot re-announce gate)
Verdict: **SCOPE-CUT-CONSISTENT**

> [DETER] crypto-spec OBLIGATOIRE — la section "## [DETER] Crypto Spec
> (load-bearing)" plus bas est le livrable principal. Le main thread
> l'implemente sans re-decider. Toute deviation = nouveau preflight.

## Evidence Rules
- Claim policy: chaque affirmation cite un path:ligne, une sortie de commande,
  une URL/date, ou une hypothese explicite.
- Local sources read: `prompts/agent/preflight.md` ; `.planning/active/sprint74_plan.md`
  (§Phase E 284-339, §7 scope cuts, §8 R1-R8) ; `.planning/research/s74_disponibilite_ux_design.md`
  (§6 strings, §8 5 verrous, §13 seed volontaire) ; `.planning/active/sprint74_kickoff.md`
  (D1-D5, §4) ; `.planning/active/sprint74_design_review.md` (G1, D1/D4 ⚠️) ;
  `crates/nexus-core-rs/src/canonical.rs` (entier) ; `crates/nexus-core-rs/src/crypto.rs`
  (entier) ; `crates/nexus-core-rs/src/task.rs` (sign/verify pattern) ;
  `crates/nexus-core-rs/src/node.rs` (Router 3 ALPN) ; `crates/nexus-core-rs/src/blobs.rs`
  (set_tag/delete_tag/fetch_ticket) ; `crates/nexus-coordinator-rs/src/invite.rs`
  (ledger revocable) ; `crates/nexus-coordinator-rs/src/db.rs` (M18 261-273, 628-683) ;
  `crates/nexus-coordinator-rs/src/public_feed.rs` (raw-op + FEED_FORMAT_VERSION) ;
  `crates/nexus-shell-daemon/src/runtime.rs` (boot node create 305-332, restore
  1700-1805) ; `crates/nexus-shell-daemon/src/deploy.rs` (faux-vert NAT 670-686,
  keep_online_tag 712) ; `crates/nexus-shell-daemon/src/http.rs` (AppState 84/138) ;
  `docs/security/THREAT_MODEL.md` (§5.4 iroh stack, §10 feed surface, §14 Operator).
- iroh 0.98.2 source read (`~/.cargo/registry/src/index.crates.io-*/iroh-0.98.2/`):
  `src/protocol.rs` (ProtocolHandler trait 229-289, RouterBuilder::accept 485-493,
  Router has NO post-spawn registration 405-448) ; `src/endpoint.rs` (connect 981,
  set_alpns) ; `src/endpoint/connection.rs` (remote_id 1077, open_bi 834, accept_bi
  855, read_to_end usage).
- Commands run:
  - `grep -A1 '^name = "iroh"' Cargo.lock` -> `iroh 0.98.2` (et iroh-blobs 0.100.0,
    ed25519-dalek 2.2.0, serde_jcs 0.2.0, rand 0.8.6 + 0.9.4, all already present).
  - `cargo tree -d` -> aucun duplicate introduit par Phase E (les dups existants =
    iroh/GPU/TUI transitifs, pre-existants ; curve25519-dalek v4.1.3 vs v5.0.0-pre.6
    deja en arbre, pas touche par E).
  - `git log --oneline -- crates/nexus-core-rs/src/blobs.rs node.rs` -> dernier =
    `4c1acc5` (Phase D set_tag) ; `f3ea1c3` (S66 FsStore) ; aucun revert.
  - `grep -rn "SeedRequest|SeedResponse|sbfb/seed|DOMAIN_SEED" crates/` -> 0 hit
    (greenfield, rien a re-ouvrir).

## Scope
- Plan source: `.planning/active/sprint74_plan.md` §Phase E (284-339).
- Target files (plan E.2, corriges file:line apres lecture reelle):
  - `crates/nexus-core-rs/src/seed.rs` (NEW) — `SeedRequest`/`SeedResponse` types +
    canonical bytes + sign/verify (PRIMITIVES PURES, zero iroh/tokio, comme
    `task.rs`/`canonical.rs`).
  - `crates/nexus-core-rs/src/canonical.rs` — ajouter `DOMAIN_SEED_REQUEST_V1` (+
    `DOMAIN_SEED_RESPONSE_V1` si la reponse est signee, cf. spec) au bloc des
    domain constants (apres `DOMAIN_FEED_V1:199`).
  - `crates/nexus-core-rs/src/node.rs` — `NodeConfig` gagne un champ
    `extra_protocols` (factory closure `FnOnce(Store) -> Vec<(Vec<u8>, Box<dyn
    DynProtocolHandler>)>`) injecte au `Router::builder` AVANT `.spawn()` (341-345).
    **Finding architectural NF-1 ci-dessous.**
  - `crates/nexus-shell-daemon/src/seed_protocol.rs` (NEW) ou `seed.rs` daemon-side —
    le `SeedProtocol` handler ALPN (impl `ProtocolHandler`) + le client dial
    `request_seed(...)`. **Vit dans le daemon, PAS dans nexus-core-rs** (il porte
    DB + keypair + blobs).
  - `crates/nexus-core-rs/src/blobs.rs` — `fetch_and_pin(...)` = `fetch_ticket` +
    `set_tag` (R3 gap : `fetch_ticket` 170-193 ne tag PAS aujourd'hui).
  - `crates/nexus-coordinator-rs/src/db.rs` — table `seed_invite` (M19, LOCALE,
    revocable, modele `invite.rs`) + (optionnel) colonne `archive_ticket` a
    `keep_online` ou table `seed_pin` cote seeder. **Finding NF-3.**
  - `crates/nexus-shell-daemon/src/http.rs` + `web/` — route mint/revoke invite +
    file d'approbation + libelle faux-vert "vu de ton noeud" (D4 vol.1, FRONT).
- Deps/APIs: aucune nouvelle dep. Compose ed25519-dalek 2.2.0 (via `crypto.rs`),
  serde_jcs 0.2.0 (via `canonical.rs`), iroh 0.98.2 Router/Connection, iroh-blobs
  0.100.0 tags/Downloader, rand (nonce). **S1b clean.**
- Security/protocol surfaces: NEW ALPN `sbfb/seed/0` (wire cross-noeud) ; NEW
  domain constant(s) ; NEW table locale invite ; faux-vert NAT relabel.
- Tests attendus (plan E.3 + amendement §13): `seed_request_signature_verified`,
  `seed_request_nonce_anti_replay`, `seeder_fetches_tags_pins_blob`,
  `seed_requires_invite_and_approval`, `seeded_app_keeps_author_provenance_intact`,
  `seed_e2e_two_nodes_peer_keeps_app_reachable` (§P57),
  `voluntary_seed_distant_public_app_no_approval`,
  `voluntary_seeder_serves_author_provenance_intact`.

## S1a OSS Prior Art
- Domain: P2P content replication par invitation/approbation authentifiee +
  registre de seeders best-effort.
- Sources (kickoff §Sources, dates 2024-2025) re-verifiees pertinentes :
  - **Radicle Heartwood** (radicle.dev/guides/seeder + /protocol) : seeding policy
    par noeud ; **delegates != seeders** (le seeder replique sans pouvoir signer la
    version canonique). = invariant SBFB "seeder != co-auteur" cable dans le
    protocole. APPROACH-ALIGNED.
  - **Tailscale share** (kb/1084, kb/1388) : invite link single-use OU reusable,
    expire 30j, revocable, quarantine par defaut. = modele invite `seed_invite`.
    APPROACH-ALIGNED.
  - **Syncthing** (docs.syncthing.net/users/introducer) : approbation EXPLICITE de
    pair avant de partager un dossier. = approbation cote pair AVANT fetch+pin.
    APPROACH-ALIGNED.
  - **IPFS / iroh-blobs content-addressing** : un blob est adresse par son hash
    (blake3) verifie au fetch -> un seeder malveillant ne peut PAS servir du contenu
    altere (hash mismatch -> rejet). = fondation de la securite du seed VOLONTAIRE
    (amendement §13). APPROACH-ALIGNED.
- Finding: **APPROACH-ALIGNED** sur les 4 axes (req/resp auth point-a-point +
  invite revocable + approbation pair + content-addressing). Aucun LIB-EXISTS qui
  remplacerait le protocole (Radicle/IPFS-Cluster sont des systemes entiers, pas
  des libs composables dans le daemon SBFB ; iroh est deja la lib de transport).
- Impact: aucune adaptation requise (pas de PLAN-ADAPT).

## S1b Dependencies, CVEs, Release Notes
- Scanned: iroh 0.98.2, iroh-blobs 0.100.0, ed25519-dalek 2.2.0, serde_jcs 0.2.0,
  rand 0.8.6/0.9.4, serde_json.
- Commands/sources: `Cargo.lock` (versions exactes ci-dessus) ; `cargo tree -d`
  (0 dup introduit) ; iroh 0.98.2 registre source (API ProtocolHandler/Router/
  Connection lue directement, pas inventee).
- Finding: **clean**. Phase E n'ajoute AUCUNE dep (composition pure). Les versions
  sont pinnees et gelees (Day 0 #3 iroh 0.98). Pas de CVE crypto/wire/sandbox sur
  la surface composee. Le carry P2-A-1 (rand upstream non publie) et P2-AUDIT-2
  (iroh pre-release transitives) restent des exemptions externes inchangees
  (kickoff §6).
- **Note stale-doc (non-bloquant, route Phase G)** : THREAT_MODEL §5.4:183 dit
  encore "Version pinnee **0.97**" alors que le pin reel est iroh 0.98.2. A
  recadrer Phase G (deja dans le lot dette ; pas un blocker E).

## S2 Historical Decisions
- Commands: `git log --oneline -- blobs.rs node.rs` ; `git show 4c1acc5
  --format=%B` ; `grep SeedRequest|sbfb/seed` (0 hit).
- Decisions crossed:
  - `fetch_ticket` ne tag PAS le blob (`blobs.rs:170-193`) : ce n'est pas une
    decision deliberee "ne jamais tagger" — c'etait le flux curator-list (le blob
    curator est re-fetchable a volonte, GC tolerable). Phase D a deja ajoute
    `set_tag`/`delete_tag` (`blobs.rs:113-134`, commit `4c1acc5`) avec la doc
    explicite "pin a blob under a stable, removable name (e.g. the Sprint 74
    keep-online pin)". **Conclusion : tagger un blob fetche cote seeder est un
    AJOUT coherent (R3), pas un reversal.** Aucun commit n'interdit de tagger un
    blob telecharge. Reverse-commit check : aucun.
  - Router 3 ALPN (`node.rs:341-345`, commit `f3ea1c3` S66) : le commentaire dit
    "the full SBFB protocol stack every node carries". Ajouter un 4e ALPN est
    explicitement anticipe par le kickoff (`node.rs:341-344` cite dans D1). Pas de
    decision "exactement 3 ALPN, jamais plus". Reverse-commit check : aucun.
  - `keep_online` M18 (`db.rs:261-273`, commit `4c1acc5`) : table LOCALE, "never on
    the wire", absent-row = enabled-by-default. La spec E REUTILISE cette table cote
    seeder. Coherent. Reverse-commit check : aucun.
- Finding: **clean** (aucun reversal de decision figee ; tous les hooks E sont des
  ajouts anticipes par le kickoff/Phase D).

## S3 Local Patterns And Threat Model
- Threats/contracts checked: T-FEED-FORGERY, T-FEED-INTEGRITY (§10), iroh stack
  §5.4, Key storage §5.7, Operator/NetworkProvider §14. Threat model FULL (nouveau
  composant securite cross-noeud).
- Pattern de signature reutilise a l'identique de `TaskEntry`/`ClaimEntry`
  (`task.rs:319-341, 508-542`) : payload + `author_pubkey:[u8;32]` +
  `signature:[u8;64]` (BigArray) + `canonical_bytes(payload, DOMAIN)` + verify
  contre `author_pubkey`. **Zero nouvelle primitive** (G1 D1 ⚠️ adresse).
- THREAT-MODEL seed protocole (voir section dediee plus bas). Couvre : SeedRequest
  forge (sig + `conn.remote_id()` cross-check), replay (nonce + ts window),
  invite vole/non-revoque (ledger + verif), seeder malveillant (content-addressing
  blake3), DoS amplification fetch (approbation gate + invite obligatoire +
  rate-limit), re-attribution auteur R5 (provenance content-addressed, seeder
  re-annonce l'archive_hash signe par l'AUTEUR, ne signe AUCUNE provenance).
- HARDENING_ROADMAP status: pas de pre-requirement S74 manquant. Le seed
  cross-noeud est le pull-forward LT-5 (kickoff §1.2), pas une dette de hardening
  en retard.
- Finding: **clean (1 non-bloquant)**. Non-bloquant : la surface ALPN seed doit
  etre ajoutee a THREAT_MODEL (nouvelle section §16 "Seed surface") — route Phase
  G (doc lot). Aucune regression d'un T0-T5 deja couvert : la mitigation Ed25519 +
  content-addressing est exactement celle de §5.4 (rows S/T deja a residuel L).

## S4 Protocol And Wire Invariants
- Wire/security files checked: `canonical.rs` (entier, domain constants), `task.rs`
  (TASK_FORMAT_VERSION=1, sign/verify), `public_feed.rs` (FEED_FORMAT_VERSION=1,
  raw-op), `db.rs` (M18 LOCAL), `node.rs` (ALPN registry).
- VERSION/domain/canonical status:
  - `FEED_FORMAT_VERSION` reste **1** (E n'ajoute PAS d'op feed ; `SeedAnnounced`
    est Phase F, raw-op, 0 bump — confirme §F.1 et CLAUDE.md:354-366).
  - `TASK_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` **inchanges**.
  - Le **wire NEW** de E = le message ALPN `sbfb/seed/0`. Sa versioning est
    **portee par la string ALPN** (`/0` = version 0 du protocole seed), pas par un
    champ `*_VERSION` global. C'est le pattern iroh natif (blobs/gossip/docs ont
    chacun leur ALPN versionne). **Le `SeedRequest`/`SeedResponse` portent un champ
    `version: u16 = 1` interne** (parallele a `Task.version`) pour un grain fin
    sous l'ALPN — aligne pre-launch (reste 1 jusqu'au tag v1.0).
  - NEW domain constant(s) `DOMAIN_SEED_REQUEST_V1` (et `_RESPONSE_V1` si signe) :
    AJOUT, pas un bump des domaines existants. Coherent avec l'historique (chaque
    famille signee a son domaine : §canonical.rs:71-199).
- Day 0 status: **preserved**. publier=identite locale signee (E ne touche pas le
  publish) ; heberger!=publier (seeder re-annonce archive_hash de l'AUTEUR, signe
  AUCUNE provenance) ; iroh 0.98 ; raw-op pre-launch (F) ; M18/M19 local.
- **Wire trace producteur->consommateur (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH)** : voir
  la section "## Wire Contract Trace" plus bas (chaque champ SeedRequest/Response
  trace cote requester (producteur) -> cote seeder (consommateur)).
- Finding: **clean**. Aucun bump pre-launch. Le nouveau wire est versionne par ALPN
  + champ interne, conforme aux invariants.

---

## NF-1 (non-bloquant, architectural — load-bearing pour l'impl)

**Le Router iroh 0.98.2 n'a AUCUN enregistrement de protocole post-spawn.**
Verifie source : `iroh-0.98.2/src/protocol.rs` `impl Router` (405-448) n'expose que
`builder`/`endpoint`/`is_shutdown`/`shutdown` — pas d'`add_protocol`. Tous les ALPN
DOIVENT etre passes a `RouterBuilder::accept(alpn, handler)` AVANT `.spawn()`
(`node.rs:341-345`).

**Consequence** : le handler `sbfb/seed/0` doit etre cree DANS
`create_node_with_config` (la ou le Router est construit), MAIS il a besoin de :
(a) le `Store` blobs (pour fetch+tag) — CREE dans `create_node_with_config`
(`node.rs:313-325`), `Store` EST `Clone` (preuve : `node.rs:337` `(*blobs_store)
.clone()`, `worker-core/.../runtime.rs:580`) ; (b) la DB `keep_online` + invite
(`Arc<Mutex<CoordinatorDb>>`) — CREE plus tard dans `runtime.rs` (apres node) ;
(c) le `KeyPair` du daemon — disponible AVANT node (`runtime.rs:308/323`).

**Resolution figee (a implementer)** : `NodeConfig` gagne un champ
```rust
pub extra_protocols: Vec<(Vec<u8>, ProtocolFactory)>,
// ou, plus simple si une seule entree :
pub seed_protocol_factory: Option<Box<dyn FnOnce(Store) -> Box<dyn DynProtocolHandler> + Send>>,
```
Le daemon construit la closure AVANT `create_node_with_config`, en capturant
`Arc<Mutex<CoordinatorDb>>` + `Arc<KeyPair>` (DB ouverte avant node — verifier
l'ordre dans `runtime.rs` et reordonner si besoin : la DB n'a pas de dependance au
node, donc l'ouvrir avant le node est sans risque). `create_node_with_config`
appelle la closure avec un `blobs_store.clone()` juste avant `Router::builder(...)`
et fait `.accept(b"sbfb/seed/0", handler)`. **Pas de OnceLock/chicken-and-egg** : le
Store existe au moment de l'appel de la closure.

Pourquoi non-bloquant : c'est une extension additive de `NodeConfig` (champ
`Default`), elle ne change aucun wire ni aucune Day 0 ; les 3 ALPN existants restent
intacts. C'est du cablage Rust, pas un conflit de design.

## NF-2 (non-bloquant) — slice de repli D5/R1 (amendement §13 deja attere D+F)

L'amendement PO §13 (lu integralement) change le sequencage : **le chemin
always-on PRINCIPAL = seed VOLONTAIRE communautaire**, deja livrable par D (pin
d'un blob DISTANT fetche) + F (`SeedAnnounced`), **SANS la crypto E**. Le segment
authentifie E (`SeedRequest` + invite + approbation) est le COMPLEMENT "designer MA
machine (VPS) / un pair specifique".

**Recommandation d'ampleur (arbitrage Checkpoint Q1, a confirmer PO)** : prioriser
le chemin volontaire (fonction `fetch_and_pin` + reutilise D2 cote app distante +
F) qui est le moins risque et couvre l'essentiel produit, PUIS le `SeedRequest`
authentifie E. Si E deborde, le slice de repli borne = `SeedRequest` signe +
verif + `fetch_and_pin` + approbation auto-sur-invite-valide + E2E 2-noeuds, en
laissant la revocation-fine d'invite + la file d'approbation UI en "Bientot"
inerte (jamais un faux bouton actif, D5/D-DISPO §8). **Plancher garanti A-D + chemin
volontaire ; E authentifie = full si la fenetre tient, slice borne sinon.** Aucun
cran non livre ne devient un bouton actif.

## NF-3 (non-bloquant) — `keep_online` cote seeder ne porte pas le ticket

La table M18 `keep_online` (`db.rs:267-272`) stocke `project_id, enabled,
archive_hash, pinned_at` — **pas** l'`archive_ticket` (l'adresse de dial). Pour le
seed VOLONTAIRE/E cote seeder, le blob est fetche via `fetch_and_pin` puis le
`Store` (FsStore redb) le PERSISTE across reboot (preuve : `node.rs:486`
`persistent_fsstore_survives_reboot`, `node.rs:504-508`). Donc apres reboot le
seeder n'a PAS besoin de re-fetcher : le blob + le tag survivent ; F ne fait que
RE-ANNONCER (re-broadcast l'annonce `SeedAnnounced`/project) depuis l'etat persiste.
**Aucune colonne ticket requise pour le seeder.** (Le ticket n'est necessaire qu'au
PREMIER fetch, fourni dans le `SeedRequest` / dans le `BrowseEntry.archive_ticket`
deja ingere pour les apps distantes, `runtime.rs:1732`.) Confirme : pas de nouvelle
colonne ; la persistance seed cote pair = `keep_online` (enabled) + blob FsStore +
re-annonce F. A documenter dans la spec F (hors-scope E mais trace ici pour eviter
un faux gap "il faut stocker le ticket").

---

## [DETER] Crypto Spec (load-bearing)

> Tout ci-dessous est FIGE. Le main thread l'implemente verbatim. Noms exacts,
> champs exacts, ordre, exclusions.

### 1. Domain constant(s) — `canonical.rs`

Ajouter apres `DOMAIN_FEED_V1` (`canonical.rs:199`) :
```rust
/// Domain separation tag for cross-node seed REQUEST canonical bytes.
///
/// Sprint 74 Phase E — authenticated cross-node seed protocol
/// (ALPN `sbfb/seed/0`). A node (or an invited peer) signs a
/// SeedRequest with its node key, proving it is the dialer that
/// asked CETTE app to be seeded by CE pair. The domain tag keeps a
/// seed-request signature from being replayed as a task / result /
/// claim / invite / kudos / curator-list / provenance / canary / PoW
/// / duress-ack / age-witness / contributor / key-rotation /
/// delegation / feed signature — the pre-image spaces are disjoint.
pub const DOMAIN_SEED_REQUEST_V1: &[u8] = b"nexus-seed-request-v1";

/// Domain separation tag for cross-node seed RESPONSE canonical bytes.
/// The seeder signs its accept/reject decision so the requester gets a
/// non-repudiable ack (Sprint 74 Phase E).
pub const DOMAIN_SEED_RESPONSE_V1: &[u8] = b"nexus-seed-response-v1";
```
Note nommage : suit le pattern `b"nexus-<famille>-v1"` (cf. lignes 71-199). Le
prefixe est `nexus-`, PAS `sbfb-` (coherence avec TOUS les domaines existants —
ne PAS utiliser `b"sbfb-seed-v1"` comme suggere par le prompt, ce serait le SEUL
domaine non-`nexus-`). Le `v1` est la version du DOMAINE (independante du champ
`version` de struct), comme documente `canonical.rs:62-64`.

### 2. `SeedRequest` struct — `nexus-core-rs/src/seed.rs` (NEW)

```rust
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use crate::canonical::{DOMAIN_SEED_REQUEST_V1, canonical_bytes};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::Result;

/// Current on-wire version for the seed protocol payloads.
/// Pre-launch policy: stays at 1 until the tagged v1.0 freeze
/// (cf. CLAUDE.md Pre-launch protocol policy). The ALPN string
/// `sbfb/seed/0` carries the protocol generation; this field is the
/// fine-grained payload version under that ALPN.
pub const SEED_FORMAT_VERSION: u16 = 1;

/// A request to a specific peer asking it to seed (fetch+pin+re-announce)
/// a specific public app archive. Sent over ALPN `sbfb/seed/0`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedRequest {
    /// Must equal SEED_FORMAT_VERSION to be accepted by this build.
    pub version: u16,
    /// blake3(name)-derived per-app id of the app to seed.
    pub project_id: String,
    /// Hex-encoded blake3 archive hash (the content the seeder will fetch+pin).
    /// 64 hex chars. The seeder fetches THIS hash; content-addressing guarantees
    /// it can only end up with the exact bytes (R5/§13).
    pub archive_hash: String,
    /// BlobTicket string (provider EndpointAddr + hash + format) so the seeder can
    /// dial the source without pkarr. Carries the address the seeder fetches from
    /// (usually the requester itself or the author). NON-EMPTY.
    pub archive_ticket: String,
    /// Ed25519 public key of the node that built+signed this request (32 bytes).
    /// Cross-checked against the QUIC-authenticated dialer id (conn.remote_id()).
    pub requester_node_id: [u8; PUBLIC_KEY_LENGTH],
    /// 32-byte random anti-replay nonce (OsRng). Hex or raw bytes — see canonical
    /// note. Stored as Vec<u8> len 32.
    pub nonce: Vec<u8>,
    /// Unix seconds when the request was minted. Freshness-gated (+/- window).
    pub ts: u64,
    /// Revocable invite token authorizing a non-self peer to request seeding of
    /// the author's app (D4 vol.2). Empty string when the requester IS the app's
    /// own node (self-designation of its own VPS using the same node key, no token
    /// needed) OR for the voluntary path (which does NOT use SeedRequest at all).
    pub invite_token: String,
}

/// A signed SeedRequest, ready to send on the wire.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedRequestEnvelope {
    pub request: SeedRequest,
    pub author_pubkey: [u8; PUBLIC_KEY_LENGTH], // == request.requester_node_id
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}
```

**Canonical bytes du SeedRequest** : signe sur la struct ENTIERE `SeedRequest`
(tous les champs ENTRENT, y compris `nonce`, `ts`, `invite_token`,
`requester_node_id`, `archive_ticket`). **EXCLU : `signature` et `author_pubkey`**
(ils sont sur l'enveloppe `SeedRequestEnvelope`, jamais dans les canonical bytes —
exactement comme `TaskEntry` : `task.rs:302-341`). Aucun champ "dispatch-only" a
exclure (contrairement a `Task.redundancy_factor`).

```rust
impl SeedRequestEnvelope {
    pub fn sign(request: SeedRequest, keypair: &KeyPair) -> Result<Self> {
        let bytes = canonical_bytes(&request, DOMAIN_SEED_REQUEST_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(SeedRequestEnvelope {
            request,
            author_pubkey: keypair.public_bytes(),
            signature,
        })
    }

    /// Verify the signature AND the attribution consistency
    /// (request.requester_node_id == author_pubkey). Mirrors
    /// ClaimEntry::verify_signature (task.rs:533-541).
    pub fn verify_signature(&self) -> Result<()> {
        if self.request.requester_node_id != self.author_pubkey {
            return Err(crate::error::NexusError::Crypto(
                "requester_node_id does not match author_pubkey".into(),
            ));
        }
        let bytes = canonical_bytes(&self.request, DOMAIN_SEED_REQUEST_V1)?;
        crate::crypto::verify(&self.author_pubkey, &bytes, &self.signature)
    }
}
```

**Fonction signing-bytes explicite** (le prompt la demande) : il n'y a PAS besoin
d'un helper sur-mesure type `task_canonical_bytes` (qui n'existe que pour exclure
`redundancy_factor`). On appelle directement `canonical_bytes(&request,
DOMAIN_SEED_REQUEST_V1)` (`canonical.rs:220-228`) :
```
seed_request_signing_bytes(req) = DOMAIN_SEED_REQUEST_V1 || 0x00 || serde_jcs::to_vec(req)
```
Ordre des cles : JCS lexicographique a chaque niveau (RFC 8785, garanti par
`serde_jcs`). Encodage `nonce` : `Vec<u8>` -> JCS le serialise en array JSON de
nombres `[12,34,...]` (deterministe). `ts` : `u64` -> nombre JSON. `requester_node_id`
`[u8;32]` -> array JSON de 32 nombres. **Tout est deterministe et cross-language**
(meme garantie que `Task`).

### 3. `SeedResponse` struct — `seed.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SeedDecision {
    /// The seeder accepted and has fetched+pinned the blob.
    Accepted,
    /// The seeder rejected; `reason` is a short machine code.
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedResponse {
    pub version: u16,
    /// Echo the project_id so the requester correlates without a separate id.
    pub project_id: String,
    /// Echo the nonce from the request (anti-confusion / correlation).
    pub nonce: Vec<u8>,
    pub decision: SeedDecision,
    /// Short reason code on Rejected: "bad-sig" | "stale-ts" | "replay" |
    /// "no-invite" | "invite-revoked" | "invite-expired" | "not-approved" |
    /// "fetch-failed" | "unknown-app". Empty on Accepted.
    pub reason: String,
    pub ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedResponseEnvelope {
    pub response: SeedResponse,
    pub author_pubkey: [u8; PUBLIC_KEY_LENGTH], // the SEEDER's node key
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}
```
**La reponse EST signee** (par le seeder, domain `DOMAIN_SEED_RESPONSE_V1`) : donne
au requester un ack non-repudiable "CE pair a accepte de seeder". `sign`/`verify`
identiques au pattern ci-dessus. Faible cout, ferme la boucle proprement (le
requester verifie que la reponse vient bien du pair dialé : cross-check
`response_env.author_pubkey == conn.remote_id()` cote requester).

### 4. Nonce anti-replay + freshness `ts`

- **Nonce** : 32 octets aleatoires via `rand::rngs::OsRng` + `RngCore::fill_bytes`
  (exactement comme `KeyPair::generate`, `crypto.rs:66-70`). Stocke `Vec<u8>` len 32.
- **Fenetre `ts`** : le seeder rejette si `|now - request.ts| > 120 s` (2 minutes).
  Rationale : un req/resp live n'est pas un log append-only ; 2 min couvre le skew
  d'horloge raisonnable tout en bornant la fenetre de replay. (Le feed utilise un
  gate 30j futur car c'est un log historique — NE PAS reutiliser 30j ici.)
- **Cache des nonces vus** : `Arc<Mutex<HashMap<[u8;32], Instant>>>` EN MEMOIRE
  cote seeder (PAS de table DB — un nonce vu n'a de valeur que dans sa fenetre de
  fraicheur). TTL = la fenetre `ts` (120 s) ; purge paresseuse (drop les entrees
  `> 120s` a chaque insert). Politique : `nonce deja vu DANS la fenetre -> Rejected
  "replay"`. Apres 120 s un nonce peut "sortir" du cache car le gate `ts` rejette
  deja toute requete > 120 s -> impossible de rejouer (la requete serait stale).
  **La combinaison nonce-cache (anti-replay intra-fenetre) + ts-gate (anti-replay
  hors-fenetre) couvre 100% du replay** sans persistance.
- Test `seed_request_nonce_anti_replay` : signer 1 requete, la passer 2x au
  verifier+gate seeder -> 1er Accepted, 2e Rejected "replay".

### 5. Invite token revocable (D4 vol.2) — `seed_invite` (M19 LOCAL)

**Modele = le ledger invite existant** (`coordinator-rs/src/invite.rs`, lu
integralement : `mint`/`revoke`/`get`/`list`, champs `expires_at`, `max_uses`,
`uses_count`, `revoked_at`). NE PAS reinventer — ajouter une table `seed_invite`
(ou reutiliser `invites` avec `scope="seed"`).

**Forme exacte du token** : le token est un IDENTIFIANT OPAQUE aleatoire (pas un
JWT, pas une signature self-contained) qui INDEXE une ligne du ledger LOCAL de
l'auteur. C'est le modele Tailscale (l'invite est une reference revocable cote
serveur de l'auteur, PAS un capability auto-portant). Pourquoi : un token
auto-portant signe ne peut PAS etre revoque (une fois emis il vaut jusqu'a
expiration) ; l'invite Tailscale est revocable EN TEMPS REEL parce qu'elle est
verifiee contre le ledger. **Donc : le seeder n'a PAS a verifier une signature sur
le token** — il transmet `invite_token` a l'auteur (ou l'auteur EST le seeder ?
non : l'auteur invite un pair a seeder POUR l'auteur).

> **Point d'architecture a trancher (NF-4, arbitrage PO)** : QUI detient le ledger
> d'invite et QUI le verifie ? Deux topologies :
> - **(T-a) Auteur invite un pair A seeder l'app de l'auteur** : l'auteur mint le
>   token (ledger cote AUTEUR). Le pair invite envoie le `SeedRequest` AU... pair
>   qui doit verifier le token. Mais le verificateur du token doit etre celui qui
>   detient le ledger = l'AUTEUR. Or dans le flux "fetch+pin", c'est le PAIR INVITE
>   qui fetch+pin (il devient seeder). Donc le token autorise le pair invite a
>   AGIR ; sa validite est verifiee... par qui ? -> incoherence si le ledger est
>   cote auteur et l'acte cote pair.
> - **(T-b, RETENU)** : le `SeedRequest` va de l'AUTEUR (requester) VERS le PAIR
>   destinataire (seeder). L'auteur dit "CE pair (toi), garde MON app". Le PAIR
>   approuve (Syncthing). PAS d'invite token necessaire dans CE sens (l'auteur
>   prouve son identite par sa signature node-key + content-addressing). L'invite
>   token sert au cas SYMETRIQUE : un pair de confiance veut s'enroller comme
>   seeder d'une app dont il n'est PAS l'auteur, via un lien que l'auteur lui a
>   donne -> le pair envoie un SeedRequest a l'AUTEUR portant le token ; l'auteur
>   verifie le token contre SON ledger et repond avec le ticket. **C'est le seul
>   chemin ou invite_token est non-vide et verifie cote receveur=auteur.**
>
> **Recommandation figee** : implementer T-b. Dans T-b, `invite_token` est verifie
> par le NODE QUI RECOIT le SeedRequest contre SA propre table `seed_invite`
> (l'auteur). Self-designation (l'auteur designe sa propre 2e machine avec la MEME
> node key) = `invite_token` vide + `requester_node_id == app's author node_id`
> (verifiable car l'app distante porte `BrowseEntry.node_id`, `runtime.rs:1723`).
> **Pour le PILOTE FERME, le chemin invite-token complet (T-b) PEUT rester
> "Bientot" inerte si E deborde (NF-2)** ; le plancher = seed VOLONTAIRE (§13, pas
> de token) + self-designation (token vide).

**Verification du token (cote node qui recoit le SeedRequest, si T-b cable)** :
1. `get(invite_token)` -> None => Rejected "no-invite".
2. `revoked_at IS NOT NULL` => Rejected "invite-revoked".
3. `now > expires_at` => Rejected "invite-expired".
4. `max_uses.is_some() && uses_count >= max_uses` => Rejected "invite-expired"
   (reuse "no more uses").
5. sinon OK + `uses_count += 1` (single-use ou reusable+expiry, modele Tailscale).
**Revocation** : `UPDATE seed_invite SET revoked_at=now WHERE id=?` (`invite.rs:111`).
**Stockage** : table LOCALE M19 (jamais sur le wire) ; seul `invite_token` (l'id
opaque) circule, jamais la ligne.

### 6. Handler ALPN — `nexus-shell-daemon/src/seed_protocol.rs` (NEW)

```rust
#[derive(Debug, Clone)]
pub struct SeedProtocol {
    store: iroh_blobs::api::Store,                 // clone, fetch+tag
    endpoint: iroh::Endpoint,                       // dial pour fetch_and_pin
    memory_lookup: iroh::address_lookup::memory::MemoryLookup,
    db: Arc<Mutex<CoordinatorDb>>,                  // keep_online + seed_invite
    keypair: Arc<KeyPair>,                          // pour signer SeedResponse
    seen_nonces: Arc<Mutex<HashMap<[u8;32], Instant>>>,
    approval: Arc<...>,                             // file d'approbation (voir §8)
}

impl iroh::protocol::ProtocolHandler for SeedProtocol {
    async fn accept(&self, conn: iroh::endpoint::Connection)
        -> Result<(), iroh::protocol::AcceptError>
    {
        let dialer = conn.remote_id();                  // EndpointId AUTHENTIFIE par QUIC
        let (mut send, mut recv) = conn.accept_bi().await?;
        let req_bytes = recv.read_to_end(64 * 1024).await?;  // borne anti-DoS (64 KB)
        let env: SeedRequestEnvelope = serde_json::from_slice(&req_bytes)?;
        // 1. sig + attribution
        // 2. dialer cross-check : env.author_pubkey (== requester_node_id) doit
        //    egaler dialer (le node qui a ouvert la connexion QUIC). Empeche un
        //    SeedRequest signe mais relaye par un tiers (le transport prouve QUI dial).
        // 3. ts window 120s + nonce cache
        // 4. invite_token (T-b) OU self-designation OU rejet
        // 5. approbation (voir §8)
        // 6. fetch_and_pin(archive_ticket, archive_hash) -> set_tag(keep-online/<pid>)
        // 7. db.set_keep_online(project_id, true, Some(archive_hash))
        // 8. signer SeedResponse{Accepted} et l'ecrire sur `send`
        send.finish()?;
        conn.closed().await;
        Ok(())
    }
}
```
- **Branchement** : via `NodeConfig.seed_protocol_factory` (NF-1). Le daemon
  construit la closure capturant `db`+`keypair`+`seen_nonces`+`approval`, la passe a
  `create_node_with_config`, qui l'invoque avec `store.clone()`+`endpoint`+
  `memory_lookup` au moment du `Router::builder(...).accept(b"sbfb/seed/0",
  handler)` (`node.rs:341-345`).
- **Acces a l'etat** : tout via les `Arc` captures. `conn.remote_id()`
  (`connection.rs:1077`) = identite QUIC authentifiee du dialer.
- **Cote requester (client)** : `endpoint.connect(EndpointAddr, b"sbfb/seed/0")`
  (`endpoint.rs:981`) -> `conn.open_bi()` (`connection.rs:834`) -> write
  `serde_json::to_vec(&envelope)` -> `send.finish()` -> read `SeedResponseEnvelope`
  -> verifier sig + `author_pubkey == conn.remote_id()`. L'`EndpointAddr` du seeder
  vient de l'invite/de la fiche.

### 7. Le gap `blobs.rs` (R3) — `fetch_and_pin`

`fetch_ticket` (`blobs.rs:170-193`) fetch mais ne tag PAS -> blob GC-eligible.
Phase D a deja livre `set_tag`/`delete_tag` (`blobs.rs:113-134`). **Ajouter une
methode dediee** (ne PAS modifier `fetch_ticket` qui sert le flux curator-list ou
le GC est tolere) :
```rust
/// Fetch a blob by ticket AND immediately pin it under a removable tag so the
/// seeder's store does not GC it (Sprint 74 Phase E, R3). Composes fetch_ticket
/// + set_tag. The seeder calls this on an approved SeedRequest.
pub async fn fetch_and_pin(
    &self,
    endpoint: &Endpoint,
    memory_lookup: &MemoryLookup,
    ticket_str: &str,
    tag_name: &str,
) -> Result<[u8; 32]> {
    let hash = self.fetch_ticket(endpoint, memory_lookup, ticket_str).await?;
    self.set_tag(tag_name, hash).await?;
    Ok(hash)
}
```
- **Nom de tag** : REUTILISER `keep_online_tag(project_id)` =
  `format!("keep-online/{project_id}")` (`deploy.rs:712`). **TRANCHE : pas de tag
  seed dedie `seed/<project_id>`.** Rationale : le seeder traite la chose comme un
  "keep_online" cote pair (exactement la semantique D2 : "cette app, je la garde en
  ligne"). Reutiliser le tag keep-online unifie le cycle ON/OFF (le toggle "Garder
  en ligne" cote pair = `delete_tag(keep-online/<pid>)`) et la re-annonce boot F
  (qui lit deja `keep_online`). Un tag separe `seed/<pid>` dupliquerait la machine
  d'etat sans gain. `HashAndFormat::raw(h)` (deja dans `set_tag`, `blobs.rs:117`).
- **Idempotence** : `set_tag` est INSERT-OR-REPLACE cote iroh-blobs ; re-pin le meme
  hash est sans effet de bord. Test `seeder_fetches_tags_pins_blob` : fetch+pin,
  puis verifier `has(hash)` + tag present (helper `has_tag` existe deja, `http.rs`).

### 8. Approbation cote pair (Syncthing) — modele le plus simple SUR pilote

**Deux niveaux, trancher par PO (NF-2) :**
- **(Niveau 1, RETENU plancher pilote)** : approbation AUTO si l'authentification
  passe (sig OK + dialer cross-check OK + ts/nonce OK + (invite valide OU
  self-designation OU app PUBLIQUE pour le chemin volontaire)). Rationale : le
  contenu est DEJA public + content-addressed ; accepter de seeder une app publique
  ne cree aucun risque pour le pair (il stocke des octets publics, il ne sert aucune
  provenance). C'est l'esprit de l'amendement §13 (le chemin volontaire n'a AUCUNE
  approbation). Pour un PILOTE FERME (2-3 personnes de confiance) c'est le modele
  le plus simple et SUR.
- **(Niveau 2, "Bientot" inerte si E deborde)** : file d'attente + action UI ("CE
  noeud te demande de garder X ; accepter ?"). Materialisee par une table locale
  `seed_pending` + une route `GET/POST /api/daemon/seed-approvals`. Le handler
  ALPN, sur SeedRequest valide mais non-pre-approuve, ecrit une ligne pending +
  repond `Rejected "not-approved"` (le requester re-essaie) OU garde la connexion
  (trop couteux) -> **recommander le pattern pending+retry**, pas la connexion
  longue.

**Recommandation figee** : Niveau 1 (auto-approve sur auth valide) pour le plancher
pilote ; Niveau 2 (file UI) = "Bientot" jusqu'a ce que la fenetre le permette. Le
test `seed_requires_invite_and_approval` asserte alors : sans invite valide (chemin
T-b) ET sans self-designation ET app non-publique-volontaire -> Rejected. (Pour le
chemin volontaire §13, l'app EST publique -> auto-approve, c'est le test
`voluntary_seed_distant_public_app_no_approval`.)

---

## Threat Model — Seed Protocol (S3 FULL)

**Assets** : disponibilite des apps (blobs repliques) ; integrite de la provenance
auteur (R5) ; ressources disque/bande-passante du seeder ; le ledger invite local.

**Actors** : auteur (designe un pair) ; pair invite/volontaire (seede) ; attaquant
reseau (forge/replay/relais) ; seeder malveillant (sert du faux contenu) ;
spammeur (DoS amplification fetch).

| Vecteur | Mitigation (file:line) | Residuel |
|---|---|---|
| SeedRequest forge (mauvaise sig) | `verify_signature` Ed25519 contre `author_pubkey` (`crypto.rs:164`, pattern `task.rs:337`) + domain separation `DOMAIN_SEED_REQUEST_V1` (`canonical.rs:220`) | Nil (crypto) |
| Sig valide mais relayee par un tiers | **cross-check `author_pubkey == conn.remote_id()`** — le transport QUIC authentifie le dialer (`connection.rs:1077`). Un tiers ne peut pas re-dial sous l'identite d'autrui (il n'a pas la cle TLS). | Nil |
| Replay d'un SeedRequest | nonce 32B + cache TTL en memoire + gate `ts` +/-120s (§4) | Nil intra+hors fenetre |
| Invite vole / non-revoque | ledger LOCAL revocable temps-reel (`invite.rs:111` revoke ; verif get+revoked_at+expires+max_uses, §5) | L (token compromis avant revocation = fenetre courte ; pilote ferme) |
| **Seeder malveillant sert un contenu altere** | **content-addressing blake3 : le fetch verifie le hash, mismatch -> rejet** (THREAT_MODEL §5.4:179 "BLAKE3 hash + iroh-blobs verify on retrieval") | **Nil (le coeur de §13 : un seeder ne PEUT pas servir autre chose que l'archive_hash demande)** |
| Re-attribution auteur (R5) | le seeder re-annonce `archive_hash` SIGNE PAR L'AUTEUR (provenance content-addressed inchangee) ; il ne signe AUCUNE provenance ; `SeedAnnounced` (F) porte `seeder_node_id` DISTINCT de l'auteur (Radicle: seeder != delegate) | Nil (invariant cable) |
| DoS amplification fetch (spam SeedRequest -> fetch couteux) | (a) invite OBLIGATOIRE pour non-self non-volontaire ; (b) `read_to_end(64KB)` borne le payload ; (c) ts/nonce gate ; (d) self ne re-fetch pas ; (e) chemin volontaire = acte LOCAL unilateral (le pair decide de fetch, pas force par un tiers) | M (un pair invite legitime peut demander N apps ; borne par la confiance pilote ; rate-limit per-dialer = raffinement post-pilote, scope cut) |
| Connexion ALPN flood | borne `read_to_end` + handler court + Router shutdown propre (`node.rs:208`) | M (rate-limit iroh builtin §5.4:182) |

**Regression check** : aucune. La surface seed n'affaiblit aucun T0-T5 existant ;
elle reutilise les mitigations Ed25519 + content-addressing deja a residuel L
(§5.4). A AJOUTER comme nouvelle section THREAT_MODEL §16 "Seed surface" (Phase G,
doc lot, non-bloquant).

## Wire Contract Trace (producteur -> consommateur)

ALPN `sbfb/seed/0`, message `SeedRequestEnvelope` (JSON via `serde_json::to_vec`) :

| Champ | Producteur (requester) | Consommateur (seeder) | Forme exacte |
|---|---|---|---|
| `request.version` | `SEED_FORMAT_VERSION=1` | rejette si != 1 | `u16` toujours-present |
| `request.project_id` | blake3(name) per-app | cle `keep_online`/tag | `String` non-vide |
| `request.archive_hash` | hex 64 du blob | hash a fetch+verifier | `String` hex-64 |
| `request.archive_ticket` | BlobTicket source | parse via `BlobTicket::from_str` + seed memory_lookup (`blobs.rs:176-184`) | `String` non-vide |
| `request.requester_node_id` | `keypair.public_bytes()` | == author_pubkey ET == conn.remote_id() | `[u8;32]` array JSON |
| `request.nonce` | OsRng 32B | cache anti-replay | `Vec<u8>` len 32 -> array JSON |
| `request.ts` | `now` secs | gate +/-120s | `u64` nombre |
| `request.invite_token` | "" (self/volontaire) ou opaque id | verif ledger (T-b) ; "" autorise self/volontaire | `String` (vide-vs-rempli porteur de semantique) |
| `author_pubkey` | `keypair.public_bytes()` | verify cible | `[u8;32]` (sur l'enveloppe, HORS canonical) |
| `signature` | `keypair.sign(canonical)` | `crypto::verify` | `[u8;64]` BigArray (HORS canonical) |

Reponse `SeedResponseEnvelope` (seeder -> requester) : `decision` (`Accepted`/
`Rejected` enum tag serde externe-default), `reason` (String code, vide si
Accepted), `nonce` echo, signe par le seeder (`author_pubkey == conn.remote_id()`
verifie cote requester). Pas d'enveloppe `{results,...}` — c'est un message nu
JSON-serialise sur le bi-stream (pattern req/resp, pas une route HTTP).

**Frontend (D4 vol.1 faux-vert NAT)** : `deploy.rs:679` ecrit
`BrowseStatus::Reachable, last_probed_at:None` pour le self. Le relabel "En ligne
(vu de ton noeud)" est une MAPPING FRONT (le statut backend reste `Reachable`) —
PAS un changement wire. Cote `web/` : la pastille self -> "En ligne (vu de ton
noeud)" (strings §6 : "En ligne / Hors ligne / Verification..."). Aucun champ
serialise touche.

## Risks And Scope Cuts
- Blocking risks: **aucun** (verdict SCOPE-CUT-CONSISTENT).
- Non-blocking risks / findings:
  - NF-1 : NodeConfig gagne un hook de protocole (additif, Default-safe).
  - NF-2 : ampleur E = arbitrage PO Checkpoint Q1 (plancher = volontaire §13 +
    self-designation ; E authentifie full si fenetre, slice borne sinon ; crans non
    livres = "Bientot" inerte, jamais faux bouton actif).
  - NF-3 : pas de colonne ticket dans keep_online (FsStore persiste le blob ;
    re-annonce F suffit) — trace pour eviter un faux gap.
  - NF-4 : topologie invite T-a vs T-b -> T-b retenu (verif token cote node
    receveur=auteur) ; self-designation = token vide + node_id match.
  - THREAT_MODEL §5.4:183 stale "iroh 0.97" -> Phase G doc lot.
  - Nouvelle section THREAT_MODEL §16 "Seed surface" -> Phase G doc lot.
- Scope cuts honores (kickoff §7) : #2 quorum cross-machine -> S75 ; #4
  re-allocation/failover -> post-launch ; #5 timer 22h -> post-launch ; #6 probe
  externe NAT -> S75 (E = libelle "vu de ton noeud") ; rate-limit per-dialer seed =
  raffinement post-pilote (coherent avec #14 rate-limit search Phase G).
- Day 0 preservees : publier=identite signee (E ne touche pas publish) ;
  heberger!=publier (seeder re-annonce archive_hash auteur, signe 0 provenance) ;
  iroh 0.98 ; raw-op/M-local pre-launch (F/M19) ; FEED_FORMAT_VERSION=1.

## Test-count entry correction (load-bearing pour le body de commit)
Le plan §1 et le kickoff annoncent 1570/1566 Rust + 294 Vitest. **STALE** : le body
de Phase D (`4c1acc5`) mesure **Rust 1639 Windows / 1643 Docker Linux**, **Vitest
web 313**, factory-operator 7, size 6/6. Le main thread DOIT re-mesurer au demarrage
reel sur `4c1acc5` et baser le delta E sur 1639/1643/313, PAS sur 1570/294.

## Action
- **SCOPE-CUT-CONSISTENT** : proceder a Phase E avec la spec [DETER] ci-dessus.
- Le commit body Phase E DOIT citer ce fichier (G8 traceability) + porter les
  findings NF-1..NF-4 + l'arbitrage d'ampleur PO (Checkpoint Q1) retenu.
- Tracker carry-over : THREAT_MODEL §5.4 stale 0.97 + §16 seed surface -> Phase G.
- Aucun bump wire ; nouveau wire (ALPN `sbfb/seed/0` + domaines) versionne par ALPN
  + champ `version=1` interne (pre-launch).
