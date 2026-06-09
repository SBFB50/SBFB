# Sprint 75 — Plan d'exécution : Découverte PULL node-centrique + ancre VPS

> Feuille de route ligne-par-ligne. Phases A-G, chacune = 1 commit atomique avec
> G8 preflight → code → suites vertes (dual-platform fail-fast) → review-deep →
> Codex → reconciliation PASS → commit body 9 sections → memory. **Gate : D3
> sign-off PO (`pivot_proposal.md`) AVANT Phase A.**

## §1 État vérifié à l'entrée

- Tip `0e2fb6b` (audit gate S74 PASS). 1 ahead local (audit findings), rien
  poussé. Env récupéré (Docker up).
- Compteurs (re-vérifiés cette session) : Rust Windows `nextest --workspace` 0
  échec (~1675, iroh-networked inclus) ; clippy `--all-targets` 0 ; doctests 0 ;
  web Vitest **331**, coverage 86.91/78.63/85.82/88.23, size 6/6, scan FR clean.
- Source of truth mesurable de sortie = §5 Fail-fast (Observed rempli en
  `sprint75_verification.md`).

## §2 Décisions Day 0 (rappel — détail kickoff §5)

- **D1** `NodeDirectoryEntry` sibling sous `DOMAIN_NODE_DIRECTORY_V1`, machinerie
  CuratorList réutilisée. **D2** FIX-A re-mint adresse+PoW au replay, d'abord,
  indépendant ; `MAX_PROOF_AGE_SECS=1800` inchangé. **D3** ⚠️ VPS 2 rôles bornés
  (directory+seed), headless config-driven ; sign-off PO requis. **D4**
  durabilité = persister ancres + re-pull boot (RAM-only entries). **D5** wire
  additif 0-bump ; liveness sonde+BLAKE3 ; multi-provider IN-SCOPE.

## §3 Research consulté

Workflow `wdeedndsh` (5 agents recherche + panel 3-substrats) + doc D3 différé
`s73_searchmanifest_index_node_design.md` + prior art OSS (kickoff §0). Le
substrat D1 a survécu à un panel adversarial (avocats A/B/C + juge code-grounded).

---

## Dépendances inter-phases

```
A (FIX-A re-mint) ──┬─> C (durabilité, réutilise le helper re-mint)
                    └─> [GATE C6 : A E2E cross-machine AVANT pull gated]
B (type+authoring) ──> C (ingest annuaire) ──> D (multi-provider+identity)
                                                └─> F (front node-Browse)
B,C ──> E (VPS headless : authoring signé + driver seed)
A..F ──> G (acceptance survives-VPS-death + carries + wrap)
```

---

## Phase A — FIX-A re-mint-on-replay (le bug live, D2)

### A.1 Scope
Corriger la racine du bug de découverte : le replay outbox re-broadcaste le PoW
**et l'adresse** périmés. Stocker le payload `ProjectAnnouncement` **non-wrappé**
dans l'outbox ; à chaque replay re-wrapper avec un PoW frais (`PowSolveCache`) ET
re-minter l'`EndpointAddr`/`BlobTicket` depuis `my_endpoint_addr()`. La fenêtre
`MAX_PROOF_AGE_SECS=1800` est **inchangée** (le re-mint la rend correcte). Extraire
un **helper re-mint-adresse** réutilisé par le path pull (Phase C). Indépendant
du pivot — landé en premier.

### A.2 Fichiers touchés
| Fichier | Rôle |
|---|---|
| `crates/nexus-shell-daemon/src/runtime.rs` | outbox : stocker payload non-wrappé ; 3 sites replay (`:1513/:1544/:1615`) re-wrap PoW frais + re-mint addr ; restore boot (`:1876-1897`) idem |
| `crates/nexus-shell-daemon/src/deploy.rs` | `publish_announcement` (`:661-687`) : persist payload non-wrappé ; re-mint au broadcast |
| `crates/nexus-shell-daemon/src/http.rs` | helper `remint_blob_ticket`/`remint_endpoint_addr` près `mint_blob_ticket` (`:1639-1662`), `pub(crate)` pour réutilisation Phase C |
| `crates/nexus-core-rs/src/pow.rs` | (lecture) `PowSolveCache` ; aucun changement de la fenêtre |
| `crates/nexus-shell-daemon/src/runtime.rs` (tests) | E2E re-mint + cross-receiver accept |

### A.3 Tests plan
1. `outbox_stores_unwrapped_payload` — l'outbox contient le payload, pas
   l'envelope figée.
2. `replay_rewraps_with_fresh_pow` — un replay >30 min produit un PoW
   `issued_at` frais (passe `verify_at`).
3. `replay_remints_endpoint_addr` — l'adresse rejouée == `my_endpoint_addr()`
   courant, pas le snapshot d'annonce.
4. `stale_announcement_accepted_by_fresh_receiver` — un récepteur frais accepte
   une app publiée il y a >30 min après replay (le bug live).
5. `remint_helper_reused_shape` — le helper produit un `BlobTicket` dialable.
6. E2E `cross_machine_discovery_after_30min` (gate cross-machine, exécuté via SSH
   mac en G ; unit-simulé ici).

### A.4 Critère d'acceptation
```
cargo nextest run -p nexus-shell-daemon -p nexus-core-rs --locked
# + dual-platform fail-fast complet avant commit
```
Le test #4 PASSE (récepteur frais voit une app vieille). `MAX_PROOF_AGE_SECS`
inchangé (grep confirme 1800).

### A.5 Commit cible
`feat(daemon): Sprint 75 Phase A — re-mint PoW + endpoint address on outbox replay`
Body 9 sections : Contexte (bug live + racine adresse+PoW), Fichiers, Delta tests
(+6), Verification §7.4, Scope cuts (kickoff §9), G8 traceability, Pre-launch
(0 bump ; seul le QUAND du mint change), Codex, Carry closure (débloque pull
Phase C).

---

## Phase B — `NodeDirectoryEntry` + `DOMAIN_NODE_DIRECTORY_V1` + authoring (D1)

### B.1 Scope
Le type signé sibling + son domaine + le write-path d'authoring (primitive 1,
critical path). Réutilise la machinerie CuratorList. Helper générique
`ingest<SignedList>` (mitigation drift C1/Q2).

### B.2 Fichiers touchés
| Fichier | Rôle |
|---|---|
| `crates/nexus-core-rs/src/node_directory.rs` (NEW) | `NodeDirectoryEntry { node_id, revision, catalog: Vec<CatalogApp{project_id, archive_hash, name, category, description}> }` + sign/verify (réutil. `canonical_bytes`) + caps |
| `crates/nexus-core-rs/src/canonical.rs` | `DOMAIN_NODE_DIRECTORY_V1` (copier précédent `DOMAIN_SEED_REQUEST_V1` `:201-219`) ; ajouter à l'énumération des familles disjointes |
| `crates/nexus-core-rs/src/lib.rs` | re-export |
| `crates/nexus-shell-daemon/src/http.rs` | `POST /api/daemon/directory/publish` (build+sign+blob-store+gossip-announce le catalogue OWN du nœud) ; auth loopback |
| `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` | helper générique `ingest_signed_list<T: SignedList>` factorisant le gate subscription/cap/revision/cross-replay (`:518-582`) |
| tests | sign/verify round-trip, cross-domain replay rejet, caps, authoring route |

### B.3 Tests plan
1. `node_directory_sign_verify_roundtrip`.
2. `node_directory_cross_domain_replay_rejected` (miroir `curator.rs:589-602` :
   une sig directory ne passe pas comme CuratorList et vice-versa).
3. `node_directory_caps_enforced` (256 entries + caps par-champ).
4. `node_directory_revision_monotone_rollback`.
5. `publish_directory_route_signs_and_announces` (authoring : build+sign+blob+
   gossip ; provenance = node keypair).
6. `generic_ingest_helper_parity` (le helper produit le même verdict que l'arm
   curator d'origine sur les mêmes inputs).

### B.4 Critère d'acceptation
`cargo nextest run -p nexus-core-rs -p nexus-shell-daemon-core -p nexus-shell-daemon --locked` + fail-fast. Le grep confirme `DOMAIN_NODE_DIRECTORY_V1` ≠ tout
domaine existant.

### B.5 Commit cible
`feat(core+daemon): Sprint 75 Phase B — NodeDirectoryEntry signed type + DOMAIN_NODE_DIRECTORY_V1 + authoring route`
Body : Contexte (D1, substrat panel adversarial), Pre-launch (nouveau DOMAIN,
0-bump `*_FORMAT_VERSION`), Carry (authoring débloque durabilité Phase C).

---

## Phase C — Ingest annuaire + durabilité catalogue distant (D4, primitive 5)

### C.1 Scope
Le **seul vrai trou archi** (R4 load-bearing). Sibling ingest arm
subscription-gated, `BrowseSource::NodeDirectory`, aggregator settant `node_id`
depuis l'entrée (plus None), **re-pull actif au boot** des `NodeDirectoryEntry`
des ancres abonnées. Absorbe WIRE-1 (indexer ReleasePublished par nom) + WIRE-2
(seed-count keyé (pid,hash)) + DBQ-1 (keep_online hash-SOT) dans le schéma.
**Gate C6** : Phase A E2E cross-machine validée AVANT de gater le pull dessus.

### C.2 Fichiers touchés
| Fichier | Rôle |
|---|---|
| `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` | sibling ingest arm `NodeDirectoryEntry` via le helper générique B ; subscription-gated |
| `crates/nexus-shell-daemon-core/src/browse.rs` | `BrowseSource::NodeDirectory` ; aggregator branche settant `node_id` (plus `None` `:632-634`) ; un-skip ou view node_id |
| `crates/nexus-shell-daemon/src/runtime.rs` | re-pull boot : itérer les ancres abonnées, re-fetch leurs `NodeDirectoryEntry` blobs (réutilise path curator gossip+blob + helper re-mint A) |
| `crates/nexus-coordinator-rs/src/public_feed.rs` | WIRE-1 : `ReleasePublishedPayload` + `project_name`/`category` Option `#[serde(default, skip_serializing_if)]` (0-bump) |
| `crates/nexus-coordinator-rs/src/search.rs` | WIRE-1 : `extract_index_fields` lit `field("category")` (déjà `project_name`) |
| `crates/nexus-shell-daemon/src/seed_registry.rs` | WIRE-2 : keyer le compteur sur (project_id, archive_hash) |
| `crates/nexus-coordinator-rs/src/db.rs` | DBQ-1 : `set_keep_online` coalesce l'archive_hash (lit M18 pas l'aggregator volatile) |

### C.3 Tests plan
1. `node_directory_ingest_subscription_gated` (pas d'ingest d'une ancre non
   abonnée).
2. `browse_aggregator_sets_node_id_from_directory`.
3. `boot_repull_restores_remote_catalogs` (le gap load-bearing : un catalogue
   distant survit au reboot via re-pull).
4. `release_published_searchable_by_name` (WIRE-1).
5. `seed_count_keyed_by_project_and_hash` (WIRE-2).
6. `set_keep_online_coalesces_known_hash` (DBQ-1).

### C.4 Critère d'acceptation
Test #3 PASSE (catalogue distant re-pull au boot). `cargo nextest run -p nexus-shell-daemon -p nexus-shell-daemon-core -p nexus-coordinator-rs --locked` + fail-fast.
**Pré-requis** : Phase A E2E vert (C6).

### C.5 Commit cible
`feat(daemon): Sprint 75 Phase C — node-directory ingest + remote-catalog durability (boot re-pull)`
Body : Contexte (primitive 5 load-bearing + 3 carries conçus dedans), Carry
closure (WIRE-1/WIRE-2/DBQ-1 CLOSED).

---

## Phase D — Pull multi-provider + node identity (D5, carry PULL-2)

### D.1 Scope
Plumber les `seeder_node_id` de `SeedRegistry` dans le vecteur providers de
`download()` (multi-provider, carry PULL-2 + SEED-1/SEED-2 bornes). Exposer
node_id (un-skip additif ou `GET /api/daemon/nodes`). Statut honnête
« joignable-via-seeder » (Q7).

### D.2 Fichiers touchés
| Fichier | Rôle |
|---|---|
| `crates/nexus-core-rs/src/blobs.rs` | `fetch_ticket` → multi-provider : `Vec<endpoint_id>` (node_id annuaire + seeders SeedRegistry) dans `download()` (`:170-193`) |
| `crates/nexus-shell-daemon/src/seed_registry.rs` | exposer `seeders_recent` en prod (aujourd'hui `#[cfg(test)]`) ; SEED-1 clamp `seen_at=min(feed_ts, recv_clock)` ; SEED-2 cap taille |
| `crates/nexus-shell-daemon-core/src/browse.rs` | statut « joignable-via-seeder » (publisher down + seeder détient BLAKE3) |
| `crates/nexus-shell-daemon/src/http.rs` | `GET /api/daemon/nodes` (grouping) OU promotion node_id additif |

### D.3 Tests plan
1. `fetch_falls_back_to_seeder_when_anchor_offline` (multi-provider).
2. `fetch_provider_ordering` (node_id annuaire d'abord puis seeders — Q5).
3. `seed_registry_clamps_future_ts` (SEED-1).
4. `seed_registry_size_bounded` (SEED-2).
5. `reachable_via_seeder_status` (Q7).
6. `nodes_endpoint_groups_by_node_id`.

### D.4 Critère d'acceptation
Test #1 PASSE (fallback seeder). Fail-fast.

### D.5 Commit cible
`feat(core+daemon): Sprint 75 Phase D — multi-provider pull + node identity exposure`
Body : Carry closure (PULL-2/SEED-1/SEED-2 CLOSED).

---

## Phase E — Ancre VPS headless (D3)

### E.1 Scope
Le modèle opérationnel VPS sans session UI. Section config `[seed]`/`[directory]`
lue au boot → `fetch_and_pin` les project_ids configurés (apps jamais déployées
localement) + re-mint adresse + re-emit. 1er appelant prod de `request_seed`.
Authoring VPS signé (builder boot ou endpoint loopback scriptable). Unit systemd.
Ack budget disque/GC (déféré). **Gate : D3 sign-off PO.**

### E.2 Fichiers touchés
| Fichier | Rôle |
|---|---|
| `crates/nexus-shell-daemon/src/config.rs` | section `[seed] keep_online_projects` / `[directory] catalog` |
| `crates/nexus-shell-daemon/src/runtime.rs` | driver boot : lire config → `fetch_and_pin` + set keep_online + re-emit + re-mint (étend `reannounce_seeds_at_boot` `feed_sync.rs:160-199` acquire-then-pin) |
| `crates/nexus-shell-daemon/src/seed_protocol.rs` | 1er appelant prod de `request_seed` (`:298`, retirer `#[allow(dead_code)]`) |
| `crates/nexus-core-rs/src/blobs.rs` | `fetch_and_pin` headless boot driver |
| `deploy/` | unit systemd + `config.toml.example` section seed/directory |
| `crates/nexus-shell-daemon/src/http.rs` ou builder | authoring VPS signé (`CuratorListEntry::sign`/`NodeDirectoryEntry::sign` avec node keypair) |

### E.3 Tests plan
1. `boot_seed_driver_pins_configured_projects` (fetch_and_pin d'une app non
   déployée localement).
2. `boot_repins_keep_online_blobs` (re-pin, pas seulement re-announce).
3. `request_seed_prod_caller` (le client a un appelant réel).
4. `vps_authoring_signs_own_directory`.
5. `config_seed_section_parsed`.

### E.4 Critère d'acceptation
Test #1 PASSE (VPS seede une app qu'il n'a pas déployée). Fail-fast. **Sign-off
PO D3 obtenu.**

### E.5 Commit cible
`feat(daemon): Sprint 75 Phase E — headless VPS anchor (config-driven seed driver + signed authoring)`
Body : Contexte (D3 + sign-off PO réf), Scope cuts (GC reaper déféré).

---

## Phase F — Browse node-centrique (front)

### F.1 Scope
Pages `/nodes` + `/node/:nodeId` (App.tsx lazy). node-Browse cohabite/supersede
la grille (Q6). Intention « ajouter une ancre » (template `Curators.tsx`). UX
cold-start 1er-run (C4 : pas d'écran vide mort). Intégration `AvailabilitySheet`.
Strings FR (scan-en-strings).
**Exigence PO provenance visible (verrou 4)** : chaque carte du catalogue node-
Browse AFFICHE la preuve de provenance (auteur signé `provenance.json` commit→
hash + identité BLAKE3), via le composant `VerificationDetail` existant ; une app
**forkée/modifiée** (re-signée local, `is_open_source=false`, hash distinct) porte
un **marqueur visuel « version dérivée/modifiée »** non ambigu, jamais le badge
de l'original. Le badge d'autorité = signature AUTEUR, jamais le nœud seeder.

### F.2 Fichiers touchés
| Fichier | Rôle |
|---|---|
| `web/src/App.tsx` | routes lazy `/nodes` + `/node/:nodeId` |
| `web/src/pages/Nodes.tsx` (NEW) | liste de nœuds (catalogue-publishers découverts) |
| `web/src/pages/NodeCatalog.tsx` (NEW) | catalogue d'un nœud → download (pull) |
| `web/src/pages/Browse.tsx` | cohabitation/supersede + `known_browse_entries` honnête |
| `web/src/components/AddAnchorDialog.tsx` (NEW) | « ajouter une ancre » (template Curators) ; cold-start 1er-run |
| `web/src/api/daemon.ts` | `listNodes`, `nodeCatalog`, `addAnchor`, `pullApp` (Zod `.strict()`) |
| `web/src/components/AvailabilitySheet.tsx` | intégration node-centrique + WEB-1 (seed toggle depuis `selfSeeding`) |

### F.3 Tests plan
Vitest : `Nodes` rendu + empty/cold-start, `NodeCatalog` pull, `AddAnchorDialog`,
`daemon` schémas `.strict()`, WEB-1 toggle reconcilié, lock-1 (0 champ
cible/hôte au publish), **lock-4 provenance** : (a) carte catalogue affiche la
signature auteur (`VerificationDetail`), (b) app forkée/modifiée
(`is_open_source=false`, hash distinct) rend le marqueur « version dérivée », pas
le badge original ; (c) le seeder n'est jamais rendu comme autorité.

### F.4 Critère d'acceptation
`(cd web && npm run test:unit && npm run test:coverage && npm run build && npm run size)` + `bash web/scripts/scan-en-strings.sh` + tsc + lint. Coverage ≥ seuils.

### F.5 Commit cible
`feat(shell): Sprint 75 Phase F — node-centric Browse (nodes list + node catalog + add-anchor)`
Body : Carry closure (WEB-1 CLOSED), lock-1/2/4 vérifiés UI.

---

## Phase G — Wrap-up + acceptance survives-VPS-death

### G.1 Scope
`sprint75_verification.md` (fail-fast §5 rempli) + `sprint76_audit_plan.md`.
Acceptance **« survives-VPS-death »** cross-machine (Win/Mac/VPS via SSH).
Hygiène carries S74 (CARRY-5 clamp offset/q, CARRY-2 Rejected-terminal-sur-trip,
PULL-1 dedup provenance, FORK-1 entry-cap). Doc META-1 (règle PATTERNS GAP-carry)
+ **CARRY-1 LT-2 ARMÉ** (flipper docs + dry-run Radicle privé). PATTERNS rust+
shell. memory + SPRINT_LOG + CLAUDE.md.

### G.2 Fichiers touchés
`.planning/active/sprint75_verification.md` + `sprint76_audit_plan.md` ;
`crates/nexus-shell-daemon/src/http.rs` (CARRY-5 clamp) ; `validator.rs` +
`validator_loop.rs` (CARRY-2 Rejected terminal) ; `deploy.rs` (PULL-1 dedup) ;
`fork.rs` (FORK-1 entry-cap) ; `docs/rust/PATTERNS.md` + `docs/shell/PATTERNS.md`
+ `docs/claude/SPRINT_LOG.md` + `CLAUDE.md` + `roadmap_v5` (amendement).

### G.3 Tests plan
Hygiène : `search_clamps_offset_and_query`, `guardrail_trip_sets_rejected_terminal`,
`deploy_strips_existing_provenance`, `fork_entry_count_capped`. + acceptance
manuelle cross-machine (checklist).

### G.4 Critère d'acceptation
Fail-fast dual-platform complet (Win nextest + **Docker Linux canonique** —
gate avant push) + web. Acceptance survives-VPS-death démontrée (Browse
fonctionne avec VPS down + une autre ancre). LT-2 dry-run Radicle privé fait.

### G.5 Commit cible
`feat(daemon): Sprint 75 Phase G — wrap-up + survives-VPS-death acceptance + S74 hygiene carries`

---

## §5 Fail-fast checklist (Observed rempli en verification.md)

| # | Check | Commande | Critère | Observed |
|---|---|---|---|---|
| 1 | fmt | `cargo fmt --all --check` | exit 0 | |
| 2 | clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warn | |
| 3 | nextest workspace | `cargo nextest run --workspace --locked` | 0 fail | |
| 4 | doctests | `cargo test --workspace --locked --doc` | 0 fail | |
| 5 | release | `cargo build -p nexus-shell-daemon --release` | OK | |
| 6 | Docker Linux canonique | `rust:1.94` nextest workspace | 0 fail | |
| 7 | FIX-A bug live | test `stale_announcement_accepted_by_fresh_receiver` | PASS | |
| 8 | DOMAIN disjoint | grep `DOMAIN_NODE_DIRECTORY_V1` ≠ existants | unique | |
| 9 | cross-domain replay rejet | test `node_directory_cross_domain_replay_rejected` | PASS | |
| 10 | durabilité boot re-pull | test `boot_repull_restores_remote_catalogs` | PASS | |
| 11 | multi-provider fallback | test `fetch_falls_back_to_seeder_when_anchor_offline` | PASS | |
| 12 | VPS seed headless | test `boot_seed_driver_pins_configured_projects` | PASS | |
| 13 | lock-3 tripwire | grep `default_curators`/`default_anchors` default vide ; pas de node_id hard-codé | PASS | |
| 14 | 0 bump wire | grep `*_FORMAT_VERSION` inchangés | PASS | |
| 15 | WIRE-1 searchable | test `release_published_searchable_by_name` | PASS | |
| 16 | web tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 | |
| 17 | web lint | `npm run lint` | 0 err | |
| 18 | web Vitest | `npm run test:unit` | pass | |
| 19 | web coverage | `npm run test:coverage` | ≥ seuils | |
| 20 | web build+size | `npm run build && npm run size` | 6/6 | |
| 21 | scan FR | `bash web/scripts/scan-en-strings.sh` | clean | |
| 22 | survives-VPS-death | acceptance cross-machine | démontré | |
| 23 | 5 verrous | review-deep checklist garde-fous 15 items | PASS | |
| 24 | carries closed | WIRE-1/2, DBQ-1, PULL-2, SEED-1/2, WEB-1, CARRY-2/5, PULL-1, FORK-1 | CLOSED | |

## §6 Git plan

1. `feat(daemon): Sprint 75 Phase A — re-mint PoW + endpoint address on outbox replay`
2. `feat(core+daemon): Sprint 75 Phase B — NodeDirectoryEntry + DOMAIN_NODE_DIRECTORY_V1 + authoring`
3. `feat(daemon): Sprint 75 Phase C — node-directory ingest + remote-catalog durability`
4. `feat(core+daemon): Sprint 75 Phase D — multi-provider pull + node identity`
5. `feat(daemon): Sprint 75 Phase E — headless VPS anchor`
6. `feat(shell): Sprint 75 Phase F — node-centric Browse`
7. `feat(daemon): Sprint 75 Phase G — wrap-up + survives-VPS-death acceptance`
(+ `chore(planning)` kickoff/plan/design_review/pivot_proposal en ouverture.)

## §7 Scope cuts (copie kickoff §9)

1. SearchManifest (digest Bloom, agrégation, query fédérée) — **différé** (s73
   §5). 2. Tantivy — gelé. 3. GC reaper/budget disque enforced — déféré
   post-launch. 4. Recherche cross-nœud fédérée — hors scope. 5. Approbation pair
   seed — inchangé (volontaire/invite S74). 6. Mobile/Electron — non. 7.
   Migration wire post-tag — 0 bump. 8. GPU → S76. 9. Sharding → S77. 10.
   Kudos-threshold tuning empirique — post-launch. 11. Multi-ancre UX avancée
   (priorité/fallback chains) — différé. 12. Bloom/Merkle digest — non posé.

## §8 Risks (détail kickoff §11)

R1 drift ingest-arm (helper générique B) · R2 stale-catalog (BLAKE3+sonde
autorité) · R3 gap seed headless (driver E) · R4 cold-start (UX 1er-run F) · R5
tripwire lock-3 (guard/review) · R6 fenêtre-rollout (A E2E avant pull gated) · R7
D3 Day-0 (pivot_proposal + sign-off avant A).

## §9 Checkpoint de clôture

Sprint fermé quand : 24/24 fail-fast verts (dont Docker Linux + survives-VPS-death
acceptance) · 7 commits A-G · `sprint75_verification.md` + `sprint76_audit_plan.md`
écrits · PATTERNS rust+shell à jour · roadmap_v5 amendé · 11 carries S74 CLOSED ·
memory + SPRINT_LOG + CLAUDE.md à jour · D3 sign-off PO consigné.
