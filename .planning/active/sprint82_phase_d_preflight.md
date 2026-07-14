# Sprint 82 Phase D — Preflight (G8)

Phase D répare ou requalifie les 10 tests du set `binary(multi_daemon)`
(9 dans `crates/nexus-test-harness/tests/multi_daemon.rs`, 1 dans
`crates/nexus-coordinator-rs/tests/multi_daemon.rs`), en distinguant la
pourriture de test env/infra d'un vrai signal produit. Preflight ultracode
= Workflow 12 agents (5 scans factuels S1a/S1b/S2/S3/S4 + 6 réfutations
adversariales + synthèse), **complété avec les résultats d'un run frais
`SBFB_INTEGRATION=1` sur HEAD `2931b82`** (produit en main-thread ; les
agents raisonnaient sur l'artefact A3 iroh 0.98 et n'avaient pas cette
donnée). Toutes les réparations restent test-only, **0 wire bump**,
**0 dep ajoutée** (`zip` déjà dep du harness), et chacune CONFIRME une
invariante gelée (blob-serve zip-only S12, feed internal-tier S65, pin
iroh `=1.0.1`, raw-op extensible) plutôt que de la re-débattre.

## Verdict: PLAN-ADAPT

Trois adaptations concrètes, appuyées par du code file:line et les runs,
imposent que le code diffère du minimal du plan : (1) la réparation feed
exige un test unitaire négatif ADDITIONNEL (l'arbre n'a aucune couverture
par défaut du garde 403 hors des tests integration-gated) ; (2)
blob_transfer doit être dé-gaté hermétique (chemin 100 % local
déterministe) et pas seulement « requalifié » ; (3) le partage 5/1 est une
hypothèse de baseline 0.98 à RE-DÉRIVER du run frais. Aucune ne touche une
décision gelée → PLAN-ADAPT, pas DESIGN-CONFLICT.

## Réconciliation run frais (main-thread, arbitre du split)

Run `SBFB_INTEGRATION=1 cargo nextest -E 'binary(multi_daemon)'` répété
**4×** sur HEAD `2931b82` (Windows natif, profile default) — résultat
STABLE **5 PASS / 5 FAIL** :

- **`test_cross_daemon_gossip_exchange` PASSE 4/4** (2.3–3.4 s), contre
  timeout 33 s sous iroh 0.98 (A3). Le seul « product-signal » de la
  baseline S81 est **résorbé**. Attribution : NON Phase A (19b92e6 touche
  le task-gossip / worker keepalive, pas le chemin curator-announce, cf.
  S1a/S2) mais le **delta transport S81** — iroh `=1.0.1` + E3 hot-join
  gossip + Topologie B. Par la logique même de la synthèse (« si gossip
  converge → 6 repairs / 0 signal »), le split final est **6 repairs /
  0 product-signal**.
- **Caveat honnête (S1a/S3)** : le PASS est sur le chemin **loopback**
  (2 process, même machine → discovery direct rapide). Il prouve la
  LOGIQUE announce/discover curator end-to-end, PAS le **SLO 30 s de
  convergence WAN-relais**, qui reste une propriété DISTINCTE couverte
  par le harness T2 live (`b3_live`), jamais élargie ici.
- Les **5 FAIL restants sont déterministes et identiques à chaque run**
  (dérive de contrat, 0 flake env) : blob_transfer (400), feed_sync (403),
  feed_replay_idempotent (403), feed_offline_catchup (403),
  new_node_full_sync_and_verify (403). Ce sont les 5 test-rots à réparer.

Reste ouvert (mesuré en Phase D après fix) : après ajout du header, les
4 tests feed passent-ils le poll de convergence iroh-docs ? Attendu OUI
(`test_cross_daemon_storage_sync` PASS 2.5 s même run = même mécanisme
ticket→join→poll iroh-docs). Si un test feed timeout au poll alors que
storage_sync passe → carry product-signal iroh-docs feed-replication
DISTINCT, jamais assoupli.

## Scans

### S1a — OSS / SOTA delta (relay-gated multi_daemon) — `PLAN-ADAPT`, med
iroh 1.0.1 (GA 2026-06-15, pin `=1.0.1`) n'a PAS changé la sémantique
discovery/gossip d'une façon qui invalide ces E2E : gossip fane toujours
sur des connexions résolues par AddressLookup/pkarr, le relais coordonne
le hole-punch NAT. Finding structurant : le repo embarque DÉJÀ le pattern
SOTA hermétique (S81 Phase K) — `crates/nexus-core-rs/src/docs.rs:613-695`
`two_nodes_sync_via_share_import` : 2 nœuds in-process + réécriture des
dial-addrs du ticket en loopback DIRECT (strip relay), non
SBFB_INTEGRATION-gated, immunisé à l'EOL n0 2026-09-30. Le dé-gating
direct-addr des 4 tests feed+new_node est POSSIBLE mais **optionnel** et
hors-scope Phase D (repair, pas ré-architecture ; mutation ticket-sur-HTTP
= changement plus large, tracké comme option). On garde le self-skip relay
pour gossip_exchange (pas de ticket à réécrire).

### S1b — Supply-chain delta — `EXECUTE`, low
**0 ajout de dep, 0 changement de pin**. Le bump hickory-resolver
0.24→0.26 est Phase K (plan:266, PO-7=A), pas Phase D. iroh reste `=1.0.1`
(`Cargo.toml:48`). La fixture zip réelle pour blob_transfer ne nécessite
AUCUNE dep : `zip` est déjà `[workspace.dependencies]` ET `[dependencies]`
direct du harness (`crates/nexus-test-harness/Cargo.toml:17`).

### S2 — Chaînes de décision historiques — `PLAN-ADAPT`, med
Split structurel CONFIRMÉ dans le code : (A) 5 test-rots DÉTERMINISTES,
relay-indépendants, iroh-version-INVARIANTS (blob_transfer 400 intentionnel
S12 ; 4 feed 403 `feed_sync.rs:596-608` avant tout gossip/sync) ; (B) 1
signal relais gossip_exchange — **désormais vert sous 1.0.1** (cf.
réconciliation). PLAN-ADAPT à honorer : (1) partage 5/1 = hypothèse 0.98,
re-dérivé du run frais → 6/0 ; (2) la réparation feed doit AJOUTER un test
négatif dédié (POST sans header → 403) ; (3) aucune production ne set
`x-sbfb-feed-internal` (grep : seul `feed_sync.rs:597` le lit ; écritures
internes via `insert_feed_operation` in-process, jamais HTTP) → un test qui
le set émule le caller interne sanctionné, PAS une déviation S65.
Requalifier un test-rot RÉPARABLE serait un scope-cut déguisé, INTERDIT.

### S3 — Couverture threat-model — `PLAN-ADAPT`, test-only 0-bump
Trois risques d'érosion à ne pas suprimer silencieusement : (1) GARDE FEED
`x-sbfb-feed-internal` = invariante sécurité (defense-in-depth vs client
externe/browser forgeant des écritures feed sur loopback ; le handler fait
signer l'op par le keypair DU nœud `feed_sync.rs:621-638` ; classe AD2,
`LOOPBACK_ENDPOINTS_TRUST_TIERS.md`). Zéro test négatif hors integration →
ajouter le header sans test négatif hermétique érode la seule tripwire →
**AJOUT obligatoire** d'un test négatif par-défaut-CI. (2) FRONTIÈRE BLOB
zip-only + CSP (`THREAT_MODEL.md`, source CSP unique
`nexus_core_rs::csp::BLOB_SERVE_CSP`) : réparer via vrai zip touche le
happy-path decompress+serve+CSP-inject ; JAMAIS de raw-serve ni assert 200
sur non-zip. (3) GOSSIP : convergence discovery = propriété disponibilité ;
sortie machine-lisible, jamais un early-return muet masquant une régression.

### S4 — Invariants wire-format — `EXECUTE`, very low
Les trois vecteurs (zip réel, header HTTP, timeout gossip) sont test-side
only. `FeedInsertRequest{op: Value}` (`feed_sync.rs:586-589`) inchangé ; le
header est un header HTTP de requête (`headers.get()`), pas un champ de
struct signée/canonique, référencé par aucun `*_VERSION`. Tous les
`*_FORMAT_VERSION`/`*_VERSION` restent =1. **0 wire bump** définitif.

## Vérification adversariale

| Test | catégorie S81 | réfutée ? | root-cause défendable | disposition | conf. |
|---|---|---|---|---|---|
| `blob_transfer` | test-rot : raw blob, zip-only S12 (400) | oui (catégorie incomplète) | 400 réel & intentionnel MAIS = la réparation la plus propre ; chemin 100 % LOCAL (`spawn(1)`), raw payload `:105` → unzip échoue `http.rs:3478-3485` ; ne masque rien (`blob_serve_returns_file_from_cached_zip` vert) | **repair + de-gate** | high |
| `feed_sync` | test-rot : S65 auth 403 | non | 403 réel `feed_sync.rs:596-608` (S65 ace05b0) ; harness omet le header `:365`. Mur env caché RÉFUTÉ : storage_sync PASS 2.5 s même run | **repair** header + AJOUT test négatif | high |
| `feed_offline_catchup` | test-rot : S65 auth 403 | oui (2e couche convergence) | 403 réel `:522` puis poll count==5 iroh-docs 30 s = classe env, PAS masquant (storage_sync converge) | **repair** header, relay-gated | high |
| `feed_replay_idempotent` | test-rot : S65 auth 403 | oui (2e porte jamais observée) | 403 à 1.96 s (`:628/:637`) avant polls `:694/:729` ; sujet réel (idempotence) jamais atteint sous 1.0.1 | **repair** header ; ne pas prétendre vert no-relay | high |
| `new_node_full_sync_and_verify` (coord) | test-rot : S65 auth 403 | non | 403 à 1.87 s (`:52`), insert `:34-57` omet le header ; poll 60 s jamais atteint. storage_sync prouve la convergence | **repair** header | high |
| `gossip_exchange` | product-signal : announce non-convergence S75 | **flippé par le run frais** | Agents (sur A3 0.98) : « survit, relay-gated ». Run frais 1.0.1 : **PASS 4/4 loopback** → fermé par transport S81 (1.0.1 + E3 hot-join + Topologie B), PAS Phase A | **repaired/documenté** (loopback) ; SLO WAN = T2 live | high |

## Plan de réparation par test

Tout test-only, **0 wire bump**, **0 dep**.

1. **blob_transfer — REPAIR + DE-GATE.** Remplacer `let payload = b"hello…"`
   (`multi_daemon.rs:105`) par un zip in-memory (crate `zip`, STORED)
   contenant `test.txt`. POST le zip à `/api/daemon/publish-blob`, GET
   `/blob-serve/{hash}/test.txt`, asserter `200` ET `body == octets`.
   RETIRER `if integration_enabled()` (`:125`) — chemin 100 % local
   déterministe. Jamais de raw-serve ni assert 200 sur non-zip.

2. **4 tests feed** (feed_sync ~:362, offline_catchup ~:502, replay ~:617 ;
   coordinator :35) **— REPAIR header.** Ajouter
   `.header("x-sbfb-feed-internal", "1")` à chaque POST
   `/api/daemon/feed/insert`. Émule le caller interne loopback sanctionné,
   NE re-débat PAS P2-FEED-INSERT-NO-AUTH-TIER. Self-skip `SBFB_INTEGRATION`
   conservé. Confirmer sur run frais que le poll de convergence passe
   (attendu OUI). Si un feed timeout au poll alors que storage_sync passe →
   carry product-signal iroh-docs feed-replication DISTINCT.

3. **AJOUT — test unitaire négatif du garde S65** (hermétique, non
   integration-gated), dans le module test de `http.rs` (pattern
   `mk_state()` + `build_test_router()`) : POST `/api/daemon/feed/insert`
   SANS `x-sbfb-feed-internal` → `403 FORBIDDEN` ; AVEC `:1` → **503**
   (`feed_sync_state: None` en test = garde passé, échec annexe attendu,
   PROUVE que le header est le gate). Tourne à chaque push, sans relais.

4. **gossip_exchange — REPAIRED/DOCUMENTÉ (run frais).** Rester relay-gated
   self-skip (`:141-144`). Run frais `2931b82` : **converge 4/4 sous 1.0.1
   hot-join + Topologie B** → note « CLOSED loopback sous 1.0.1 ». JAMAIS
   forcer vert (il l'est réellement), JAMAIS élargir le deadline 30 s, ne
   PAS prétendre couvrir le SLO WAN-relais (= T2 live). Env caveat
   CLAUDE.md:408-412 : re-run SOLO avant toute conclusion de régression.

## Invariants à préserver

- **0 wire bump / 0 dep** : test-side only ; tous `*_VERSION` restent =1 ;
  `zip` déjà dep ; aucun champ de struct canonique/signée touché.
- **Auth-tier feed conservé + tripwire hermétique** : garde 403
  `feed_sync.rs:596-608` (P2-FEED-INSERT-NO-AUTH-TIER, S65 ace05b0, GELÉE)
  non affaibli ; test négatif par-défaut-CI pour qu'une refacto future ne
  puisse pas retomber en pré-S65 no-auth silencieusement.
- **Frontière blob untrusted zip-only préservée** : réparer via vrai zip
  (happy-path), jamais raw-serve ni assert 200 sur non-zip ; garder CSP.
- **Signal gossip jamais masqué** : il est réellement vert (mesuré) ; on le
  documente, on ne le force pas ; SLO 30 s WAN jamais élargi. Requalifier un
  test-rot déterministe réparable = scope-cut déguisé, INTERDIT.
- **Aucune décision Day-0 re-débattue** : blob zip-only S12, feed
  internal-tier S65, pin iroh `=1.0.1`, raw-op extensible — CONFIRMÉES.
