# Sprint 63 — Kickoff (verification tiers + UX)

**Ecrit** : 2026-05-15 (post-audit gate S62 PASS `1405c0c`).
**Type** : **sprint impair** — pas de phase dette obligatoire (Regle 1 §6.2.1).
Deux MANDATORY 3/3 (Regle 2) : P2-IMAGE-DEP + P2-PLAYWRIGHT-REFACTOR.
**Tip master d'entree** : `1405c0c` (audit findings S62 PASS + route P2 audit plan).
**Phase 0 audit Sprint 62** : **DEJA JOUE** — `72db7e2` PASS
(0 P0, 0 P1, 5 P2, 2 P3). Aucun fix bloquant requis.
**Version archive** : v2.0 — Public Verifiable Protocol Feed.
**Roadmap source** : `.planning/research/public_verifiable_feed_roadmap.md`
Sprint 3 sur 6 (5+1 reserve).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-13 (2 jours).
  3 triggers evalues (INCHANGES depuis S62) :

  1. **iroh > 0.98 + iroh-docs > 0.98** : `cargo search iroh` retourne
     `iroh = "1.0.0-rc.0"`. `cargo search iroh-docs` retourne
     `iroh-docs = "0.99.0"`. Inchange depuis S62. **Decision** : reste
     deferred (upgrade iroh 1.0 = sprint dedie post-feed).

  2. **arti-client > 0.41** : `arti-client = "0.42.0"`. Inchange.
     **Decision** : deferred. 0 CVE entre 0.41 et 0.42.

  3. **frost-ed25519 > 3.0** : `frost-ed25519 = "3.0.0"`. Trigger
     INACTIF (on utilise 3.0.0, trigger > 3.x).

- **context7 tray-icon** : `/tauri-apps/tray-icon` consulte.
  API confirmee : `Icon::from_rgba(rgba, width, height)` prend du
  raw RGBA pixel data. Pas besoin du crate `image` si on pre-decode
  le PNG. Alternative : build.rs qui decode le PNG a la compilation
  et embed les octets RGBA bruts, ou utiliser le crate `png` (decoder
  minimal) au lieu de `image` (abstraction lourde).

- **context7 Playwright** : `/microsoft/playwright.dev` consulte.
  `webServer` config accepte n'importe quel `command`. `globalSetup`
  est un script TypeScript classique. Le fix du global-setup = remplacer
  le spawn `uv run nexus-coordinator` (Python supprime S50) par un
  spawn du daemon Rust `nexus-shell-daemon init` + `nexus-shell-daemon start`.

- **Codebase audit** (agent Explore 6 scans) :
  - `provenance.rs:56` : `verify_provenance(record_json, public_key)`
    retourne bool. `ProvenanceRecord` struct (7 champs, SLSA L1).
    Aucun endpoint HTTP existant. Provenance generee au deploy
    (`deploy.rs:159`), inseree dans le zip comme `provenance.json`,
    hash BLAKE3 propage dans l'annonce comme `provenance_hash`. Pas
    de stockage SQLite du record complet.
  - `sbfb-bridge.js` : 11 methodes (task_submit, storage_get/set/
    list/delete, identity_pubkey, node_status, browse_list,
    storage_version, onStorageUpdate, piiRedact). `useBridge.ts` :
    9 dispatch cases.
  - `BrowsedProject.tsx:276` : ShieldCheck badge conditionne par
    `entry.provenance_hash`. Aucun `VerificationDetail` existant.
    Verification = binaire (badge ou rien).
  - `nexus-launcher/src/tray.rs:14-19` : `include_bytes!()` +
    `image::load_from_memory()` + `Icon::from_rgba()`. Le crate
    `image 0.25` est utilise uniquement pour decoder un PNG embarque.
    `Cargo.toml` : `image = { version = "0.25", default-features = false,
    features = ["png"] }`.
  - `web/tests/global-setup.ts:52-129` : spawn `uv run --package
    nexus-coordinator nexus-coordinator init/start`. Python supprime
    S50-S51. C'est la cause exacte du blocage Playwright.
  - `public_feed.rs:56-59` : `PublicFeedOperation` a 2 variants
    (ReleasePublished, SourceBecameStale). CuratorVouched et
    BuildQuorumReached documentes spec §2.2 mais non implementes.
  - `db.rs` : 11 migrations (M1-M11). M12 disponible.

- **ROADMAP_COMMITMENTS check (G7 Regle 3)** :
  - LT-1 Kudos-v2 : **CLOSED S59**.
  - LT-2 Radicle : **trigger PENDING** — tag v1.0 pose localement,
    pas pousse vers origin. Push prevu Sprint 65 (go-live). Pas
    encore actif.
  - LT-3/LT-4/LT-5 : latent. 0 condition declenchee.
  - LT-6 : RESOLVED S32.
  - LT-7 : **gate satisfait** (Tier 1+2 S55 + Tier 3 S60).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 62 CLOSED + audit PASS (`1405c0c`). Deuxieme sprint du roadmap
post-v1.0 livre : feed sync P2P via iroh-docs (FeedSyncState,
boot_feed_namespace, spawn_feed_subscribe, endpoints feed/ticket +
feed/join), catch-up offline multi-daemon E2E (import_and_subscribe
atomique, blob read retry backoff), anti-spam minimal (FeedRateLimiter
GCRA 5/min/auteur, PoW 16-bit BLAKE3). 4 phases A-D + 8 fix.

Le reseau SBFB peut desormais synchroniser le feed public entre
noeuds, y compris apres une periode offline. Les operations sont
verifiees (Ed25519 + hash-chain per-auteur) et protegees (PoW +
rate-limit). Ce qui manque : un utilisateur ou developpeur ne peut
pas encore **voir** pourquoi un projet est verifie — le badge
"Verifie" existe mais ne detaille rien.

### §1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP last_validated 2026-05-13. 3 triggers evalues
(voir §Sources ci-dessus) : tous inchanges depuis S62. Aucune
action requise pour S63.

### §1.3 Compteurs tests entree (tip `1405c0c`)

| Suite | Compte |
|---|---|
| Rust nextest | 1299 |
| Rust doctests | 0 pass, 1 ignored |
| Vitest | 258 |
| Playwright | 0 (global-setup fail pre-existant S50) |
| size-limit | 6/6 |
| **Total** | **~1563** |

### §1.4 Post-launch protocol policy

Tag v1.0 pose localement. Politique post-v1.0 :

- Chaque break du format bump `*_FORMAT_VERSION`
- Chaque decoder accepte un range de versions
- Ajouts de champs portent `#[serde(default)]`
- S63 ne touche PAS le wire format feed (pas de nouveau variant
  PublicFeedOperation, pas de bump FEED_FORMAT_VERSION)
- La nouvelle table `provenance_records` est purement locale
  (pas de wire format inter-noeuds)

---

## §2 Goal en une phrase

Un utilisateur non-technique voit "pourquoi ce projet est verifie"
dans le shell (modal detail provenance) ; un developpeur verifie
une release via endpoint HTTP ; une app iframe interroge la
provenance via le bridge — pendant que les 2 carries MANDATORY 3/3
sont resolus et que Playwright redemarre.

**Critere SMART : toutes les rows fail-fast du verification.md
vertes, mesure binaire au Phase D wrap-up.** Le verification.md
§Fail-fast checklist est le critere mesurable du sprint.

---

## §3 Phase 0 — Audit gate du sprint precedent

Sprint 62 audit PASS (`72db7e2`). 0 P0, 0 P1, 5 P2, 2 P3.
2 nouveaux P2 identifies (F1 P2-FEED-PUBLISH-ORPHAN, F2
P2-SUBSCRIBE-STREAM-BREAK). Carries routes dans sprint63_audit_plan.md.
Sprint 63 demarre directement.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Provenance endpoint : stockage SQLite + HTTP

**Retenu** : nouvelle migration M12 `provenance_records` dans
`coordinator.db`. Chaque deploy via `deploy_from_repo` insere le
`ProvenanceRecord` complet dans la table (actuellement seul le
hash BLAKE3 est propage dans l'annonce, le record JSON est enterre
dans le zip). Endpoint `GET /api/v1/project/{id}/provenance`
retourne le record JSON + resultat de verification live
(`verify_provenance()` appele cote serveur avec la cle publique
du noeud coordinateur).

Schema M12 :
```sql
CREATE TABLE IF NOT EXISTS provenance_records (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      TEXT NOT NULL,
    repo_url        TEXT NOT NULL,
    commit_sha      TEXT NOT NULL,
    artifact_hash   TEXT NOT NULL,
    node_id         TEXT NOT NULL,
    signature       TEXT NOT NULL,
    timestamp       TEXT NOT NULL,
    schema_version  INTEGER NOT NULL DEFAULT 1,
    created_at      INTEGER NOT NULL,
    UNIQUE (project_id, artifact_hash)
);
CREATE INDEX IF NOT EXISTS idx_prov_project ON provenance_records(project_id);
```

Le `project_id` est derive du `ProjectAnnouncement` (blake3 hash
du contenu d'annonce — pattern existant `BrowseAggregator`).

**Rejete** :
- Extraction a la volee depuis le zip : necessite acces blob a
  chaque requete, lent, pas cache-friendly, le blob peut ne plus
  etre local si le noeud fait du garbage-collection.
- Table separee par projet : 1 table par projet = complexite
  migration. Une table indexee suffit pour des milliers de projets.
- Pas de stockage (hash only) : un hash BLAKE3 sans le record
  associe ne permet pas la verification tiers — le but du sprint.

**Implications code** :
- `crates/nexus-coordinator-rs/src/db.rs` (M12 + insert/query)
- `crates/nexus-shell-daemon/src/deploy.rs` (insert au deploy)
- `crates/nexus-shell-daemon/src/http.rs` (GET endpoint)

### D2 — Bridge verification : 3 methodes postMessage

**Retenu** : 3 nouvelles methodes dans `sbfb-bridge.js`, pattern
identique aux 11 methodes existantes (postMessage + correlation
ID + reponse async) :

1. `getProvenanceRecord(projectId)` → `provenance_get` : retourne
   le ProvenanceRecord JSON complet pour un projet donne.
2. `verifyRelease(projectId)` → `provenance_verify` : retourne
   `{ verified: bool, record: ProvenanceRecord | null, error?: string }`.
3. `getPublicFeedCursor()` → `feed_cursor_get` : retourne
   `{ last_seq: number, last_entry_hash: string }` (utilise le
   cursor materializer existant `feed_cursor` M10).

Cote daemon, 3 handlers HTTP correspondants dans `http.rs` qui
delegent a `provenance.rs` et `db.rs`.

**Rejete** :
- WebSocket push : pattern non etabli dans SBFB (tout est
  request/response via postMessage). Over-engineering pour du
  read-only.
- REST direct depuis l'iframe : bloque par `sandbox="allow-scripts"`
  sans `allow-same-origin` + CSP `connect-src 'none'` — seul le
  bridge postMessage peut communiquer avec le reseau.
- Methode unique `getVerificationInfo()` bundled : 3 methodes
  distinctes permettent l'usage a la carte (feed cursor sans
  provenance, provenance sans verification active).

**Implications code** :
- `web/public/sbfb-bridge.js` (+3 methodes)
- `web/src/hooks/useBridge.ts` (+3 dispatch cases)
- `crates/nexus-shell-daemon/src/http.rs` (+3 handlers)

### D3 — UI proof-chain : modal VerificationDetail

**Retenu** : composant React `VerificationDetail` en modal/drawer,
declenche par clic sur le badge ShieldCheck existant
(`BrowsedProject.tsx:276`). Progressive disclosure :

- **Niveau 1** (toujours visible) : badge "Verifie" + icone
  ShieldCheck (existant).
- **Niveau 2** (modal au clic) : repo URL cliquable, commit SHA
  (tronque + copie), artifact hash, signature Ed25519 (tronque),
  noeud coordinateur (node_id tronque), timestamp. Bouton
  "Verifier" qui appelle `verifyRelease()` via le daemon et
  affiche le resultat live.
- **Niveau 3** (futur S64+) : placeholders pour curator vouches,
  build quorum, historique feed operations. Non implemente S63.

Composant : `web/src/components/VerificationDetail.tsx` (nouveau).
Donnees : le composant fetch la provenance via
`GET /api/v1/project/{id}/provenance` au moment du clic (pas au
mount de la page browse — lazy loading).

**Rejete** :
- Page full-page dediee : trop lourd pour une info de detail.
  Modal = pattern UX coherent avec shadcn/ui (Dialog component).
- Tooltip seulement : insuffisant pour 7+ champs de provenance.
  Un tooltip ne permet pas la verification interactive.
- Inline dans la card : pollue visuellement la page Browse qui
  liste potentiellement des dizaines de projets.

**Implications code** :
- `web/src/components/VerificationDetail.tsx` (nouveau)
- `web/src/pages/BrowsedProject.tsx` (badge cliquable → modal)
- `web/src/api/` ou inline (fetch provenance)

### D4 — MANDATORY 3/3 : IMAGE-DEP + PLAYWRIGHT-REFACTOR

**Retenu** : Phase A batch resolution des 2 items 3/3.

**P2-IMAGE-DEP** : remplacer `image = "0.25"` par le crate `png`
(decoder minimal) dans `nexus-launcher/Cargo.toml`. Le code
`tray.rs:14-19` utilise `image::load_from_memory()` uniquement pour
decoder un PNG embarque en RGBA. Le crate `png` fait exactement la
meme chose avec une fraction des transitives (~3 deps au lieu de ~15).
Alternative evaluee : build.rs pre-decode (zero dep runtime) —
rejete car ajoute complexite build pour un gain marginal vs `png`.

**P2-PLAYWRIGHT-REFACTOR** : reecrire `web/tests/global-setup.ts`
pour spawner le daemon Rust au lieu du coordinateur Python supprime
S50. Le flow :
1. `global-setup.ts` spawn `nexus-shell-daemon init <name>` (cree la
   DB + config dans un repertoire hermetic).
2. `global-setup.ts` spawn `nexus-shell-daemon start <name> --port 18765`
   (lance le daemon HTTP + P2P).
3. `waitForHealth()` poll `/health` (existant, inchange).
4. `global-teardown.ts` kill le process (inchange).
Le `webServer` Playwright continue a spawner le frontend Vite
separement (config existante, inchangee).

**Rejete** :
- Defer a S64 : violation Regle 2 §6.2.1. 3/3 = plan obligatoire.
- IMAGE-DEP via build.rs : complexite supplementaire (build script,
  fichier .rgba intermediaire, CI cache invalidation) pour eliminer
  ~12 transitives de plus que `png`. Le crate `png` est le sweet spot.
- PLAYWRIGHT via mock : les tests Playwright testent le vrai flux
  E2E. Mocker le backend = tests inutiles.

**Implications code** :
- `crates/nexus-launcher/Cargo.toml` (image → png)
- `crates/nexus-launcher/src/tray.rs` (adaptation API png)
- `web/tests/global-setup.ts` (rewrite spawn Rust daemon)
- `web/tests/global-teardown.ts` (adaptation si necessaire)

### D5 — Scope : feed operations differees, Protocol Explorer absorbe

**Retenu** : les variants `CuratorVouched` et `BuildQuorumReached`
sont differes a S64 (hardening public, §Sprint 4 roadmap). S63 se
concentre sur rendre la verification **existante** visible, pas
sur enrichir le feed avec de nouvelles operations. Le Protocol
Explorer avance (section "Verification & Provenance" dans
`examples/sbfb-explorer/`) est absorbe dans Phase D si le budget
LOC le permet, sinon scope cut S64.

**Rejete** :
- Implementer CuratorVouched/BuildQuorumReached en S63 : elargit
  le scope au-dela du theme "verification tiers UX". Ces operations
  sont des enrichissements feed (Sprint 4 territory), pas de la
  mise en visibilite.
- Reporter le Protocol Explorer completement : c'est un ajout HTML
  minimaliste a une app existante. Si le budget existe en Phase D,
  l'inclure renforce la coherence du sprint.

**Implications** :
- Scope cuts explicites (§7)

### Acknowledged review findings (G1)

Scoring : D1 ⚠️, D2 ✅, D3 ✅, D4 ⚠️, D5 ✅.
Rigor signal G4 satisfait (2 ⚠️ sur 5, 0 ❌).

D1 ⚠️ : numerotation M12 non validee contre le code au moment du
draft. **Decision** : verifie — `db.rs` contient M1-M11 (11
migrations). M12 est le bon numero. Le scan codebase pre-gel
a confirme (agent Explore : "db.rs : 11 migrations M1-M11").

D4 ⚠️ : poids exact de `png` vs `image` non compare factuellement.
**Decision** : ajustement — Phase A preflight G8 (scan S1b) doit
inclure un `cargo tree -p nexus-launcher -d` avant/apres le swap
pour quantifier le delta transitives. Si `png` ne reduit pas
significativement (< 5 transitives eliminees), garder `image` avec
features minimales et reclasser le P2 comme process (calibration
estimation). Le fix reste Phase A quoi qu'il arrive — le carry 3/3
est resolu.

**CONCERN G1 levee** : les 2 ⚠️ sont acknowledges avec actions
concretes (verification M12 factuelle + delta transitives mesure
en preflight). Verdict G1 final : **PASS** (2 ⚠️ acknowledges,
0 ❌, CONCERN levee).

---

## §5 Plan Phase outline A..D

### Phase A — MANDATORY 3/3 carries (IMAGE-DEP + PLAYWRIGHT-REFACTOR)

Resolution des 2 items a 3 reports consecutifs.
P2-IMAGE-DEP : remplacer `image` par `png` dans nexus-launcher,
adapter `tray.rs` pour l'API `png::Decoder`. P2-PLAYWRIGHT-REFACTOR :
reecrire `global-setup.ts` pour spawner le daemon Rust. Verification :
`cargo build -p nexus-launcher` compile, `npx playwright test`
passe le setup (meme si certains tests echouent sur d'autres
aspects, le global-setup ne bloque plus).

**Commit cible** : `feat(launcher+web): Sprint 63 Phase A — MANDATORY IMAGE-DEP + PLAYWRIGHT-REFACTOR`

### Phase B — Provenance endpoint HTTP + stockage SQLite

Migration M12 `provenance_records`. Insert au deploy. GET endpoint
`/api/v1/project/{id}/provenance` avec verification live. Tests
handler : provenance presente → 200 + record JSON, provenance
absente → 404, verification result inclus.

**Commit cible** : `feat(feed): Sprint 63 Phase B — provenance endpoint HTTP + SQLite M12`

### Phase C — Bridge verification + UI proof-chain

3 methodes bridge (provenance_get, provenance_verify, feed_cursor_get).
Handlers HTTP correspondants. SDK sbfb-bridge.js mis a jour.
Composant VerificationDetail (modal shadcn Dialog, lazy fetch
provenance). Badge ShieldCheck cliquable. Tests Vitest composant +
integration bridge.

**Commit cible** : `feat(web+bridge): Sprint 63 Phase C — bridge verification + UI VerificationDetail`

### Phase D — Protocol Explorer verification + wrap-up

Section "Verification & Provenance" dans `examples/sbfb-explorer/`
(HTML pur, demo verification interactive via bridge). Si scope
coupe, documenter dans verification.md. Verification.md + audit_plan
S64.

**Commit cible** : `feat(examples): Sprint 63 Phase D — Protocol Explorer verification + wrap-up`

---

## §6 Items carry/dette

| Item | Compteur S63 | Classification | Justification |
|---|---|---|---|
| P2-A-1 rand blocker upstream | 24+/3 | exemption externe renouvelee | blocker upstream rand 0.9 crate. Pas de resolution possible cote SBFB. Exemption permanente. |
| P2-AUDIT-2 iroh transitives pre-release | herite | exemption externe renouvelee | herite du pin iroh 0.98. Upgrade iroh 1.0 = sprint dedie. |
| P2-IMAGE-DEP image 0.25 footprint | 3/3 → **RESOLU Phase A** | MANDATORY (Regle 2) | Remplace image par png (D4). |
| P2-PLAYWRIGHT-REFACTOR global-setup | 3/3 → **RESOLU Phase A** | MANDATORY (Regle 2) | Rewrite spawn Rust daemon (D4). |
| P2-G-1 exe lock intermittent | reouvert | carry confirme S63 | dev-env intermittent. Monitoring continu. |
| P2-FEED-INSERT-NO-AUTH-TIER | 1/3 → 2/3 | carry S64+ | auth tier feed insert. Pas bloquant pour verification tiers. |
| P2-FEED-SUBSCRIBE-JOINHANDLE | 1/3 → 2/3 | carry S64 | subscribe JoinHandle non trackee. Pas de symptome observe. |
| P2-BACKFILL-6PLUS-TEST | 1/3 → 2/3 | carry S64 | P1 code ferme (5d52b6c), preuve test integration manquante. |
| P2-FEED-PUBLISH-ORPHAN | 1/3 → 2/3 | carry S64 | feed_insert split DB/iroh-docs. Retry/rollback S64 hardening. |
| P2-SUBSCRIBE-STREAM-BREAK | 1/3 → 2/3 | carry S64 | subscribe reconnexion manquante. Resilience S64 hardening. |
| F1 P2-VERSION-NOT-STORED | 2/3 → 3/3 | carry confirme S64 MANDATORY | version non stockee en DB. Devient 3/3 S64. |
| F5 P2-IROH-INFRA-TIMEOUT | 2/3 → 3/3 | carry confirme S64 MANDATORY | iroh infra tests timeout intermittent. Gate SBFB_INTEGRATION. Devient 3/3 S64. |

**Items a 3/3 MANDATORY S63** : P2-IMAGE-DEP + P2-PLAYWRIGHT-REFACTOR
→ resolus Phase A.

**Items qui passent 3/3 S64** : F1 P2-VERSION-NOT-STORED +
F5 P2-IROH-INFRA-TIMEOUT. Deviennent MANDATORY Sprint 64.

---

## §7 Scope cuts (10 items)

| # | Item | Sprint cible |
|---|---|---|
| 1 | CuratorVouched operation implementation | Sprint 64 (hardening) |
| 2 | BuildQuorumReached operation implementation | Sprint 64 |
| 3 | Quarantine feed (gate d'admission) | Sprint 64 |
| 4 | Age witness gate feed | Sprint 64 |
| 5 | Multi-forge feed sync (>3 noeuds) | Sprint 64+ |
| 6 | Feed format version bump | Sprint 64+ |
| 7 | Go-live public + tag push + pilote externe | Sprint 65 (roadmap S5) |
| 8 | CLI verify-release (HTTP endpoint suffit S63) | Sprint 64 |
| 9 | Protocol Explorer verification (si scope coupe Phase D) | Sprint 64 |
| 10 | VerificationDetail niveau 3 (curator vouches, build quorum) | Sprint 64+ |

---

## §8 Tracabilite scope

Troisieme sprint du roadmap post-v1.0. Le scope S63 vient
directement du roadmap `.planning/research/public_verifiable_feed_roadmap.md`
Sprint 3 ("Verification tiers + UX"), avec ajout de la Phase A
MANDATORY 3/3 (Regle 2 §6.2.1). Les scope cuts S62 resolus
ici : items 3, 4, 5 du kickoff S62 (endpoint, bridge, UI).
Les scope cuts S62 items 1, 2 (CuratorVouched, BuildQuorumReached)
sont re-differes a S64 (not UX, enrichissement feed).

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Playwright global-setup ne demarre pas le daemon Rust (binaire pas trouve, port conflit) | Medium | Medium | Tests locaux Phase A. PATH explicite vers le binaire release. Port hermetic 18765. |
| R2 | Crate `png` API incompatible avec le pattern tray.rs actuel | Low | Low | L'API `png::Decoder` est documentee, pattern connu. Fallback : garder image avec features minimales si echec. |
| R3 | Provenance endpoint lent (join SQLite + verification Ed25519) | Low | Low | verify_provenance() = pure function ~1ms. Table indexee par project_id. |
| R4 | Bridge postMessage timing (iframe pas encore prete quand VerificationDetail appelle) | Low | Medium | Pattern etabli (11 methodes existantes avec retry + timeout). Lazy fetch au clic, pas au mount. |
| R5 | Tests Playwright pre-existants echouent apres PLAYWRIGHT-REFACTOR | Medium | Low | Le daemon Rust expose la meme API HTTP que le coordinateur Python (S46 frontend direct-daemon migration). Les tests doivent fonctionner. Adapter si besoin (Phase A scope). |

---

## §10 Audit gate pattern — rappel

Phase 0 jouee (Sprint 62 audit PASS). Phase D devra produire
`sprint64_audit_plan.md` pour le prochain sprint. L'audit gate
reste actif a chaque transition de sprint.

---

## §11 Checkpoint de validation

1. Le stockage provenance SQLite M12 (D1) est-il coherent avec
   le pattern M9-M11 du feed store ?
2. Les 3 methodes bridge (D2) suivent-elles exactement le pattern
   des 11 methodes existantes ?
3. Le modal VerificationDetail (D3) est-il coherent avec le
   design system shadcn/ui utilise dans le shell ?
4. Le remplacement image→png (D4) elimine-t-il effectivement les
   ~12 transitives superflues ?
5. Les scope cuts (D5) sont-ils coherents avec le roadmap Sprint 4
   (hardening public) ?
