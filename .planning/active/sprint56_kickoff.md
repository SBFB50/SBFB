# Sprint 56 — Kickoff (Gossip resilience + bridge extensions)

**Ecrit** : 2026-05-09 (post-audit gate S55 PASS `e5d6242`).
**Type** : **sprint pair** — phase dette obligatoire (§6.2.1
Regle 1). 2 items 3/3 MANDATORY a traiter Phases A-B.
**Tip master d'entree** : `e5d6242`.
**Phase 0 audit Sprint 55** : **DEJA JOUE** — `e5d6242` PASS
(0 P0, 0 P1, 2 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-08 (1j). 5 fichiers
  security avec triggers_revalidate. 0 trigger actif pertinent pour
  le theme S56. HARDENING_ROADMAP frais (S55). Pas de pre-research
  supplementaire.

- **governor 0.10.2** (deja dans workspace S21) : GCRA rate-limiter
  per-key via `DefaultKeyedRateLimiter<K>`. DashMap backend pour
  cardinalite dynamique. Deja utilise dans nexus-worker-core pour
  rate-limit per-tuple. Aucune mise a jour necessaire.

- **rusqlite / rusqlite_migration** (deja dans workspace) :
  5 migrations existantes dans coordinator.db. WAL mode actif.
  Pattern migration M{N} valide.

- **ROADMAP_COMMITMENTS check** :
  - LT-7 self-hosted build : Tier 1+2 DONE (S55). Tier 3 S57+.
  - LT-1 Kudos-v2 : trigger Gini > 0.70. Pas de donnees. Latent.
  - LT-2..LT-5 latents. LT-6 RESOLVED S32.
  - 0 condition declenchee.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 55 CLOSED + audit PASS (`e5d6242`). Le workspace est en
edition 2024. Le P2P iroh est valide cross-machine (LAN Win-Mac,
WAN dev-VPS Helsinki). CI operationnel (Woodpecker ci.sbfb.world +
GHA). LT-7 Tier 1+2 livres (build executor + quorum SHA256).

**Etat technique (tip `e5d6242`)** :
- Workspace clean, edition 2024, Rust 1.94
- Outbox gossip = Vec en memoire, perdu au restart daemon
- Browse_request = 0 rate-limit per-peer
- Bridge postMessage = 4 methodes (task_submit, storage_get,
  storage_set, pii_redact), SDK sbfb-bridge.js
- CoordinatorDb = 5 migrations SQLite WAL, table task_results (S55)
- governor 0.10.2 rate-limiter keyed GCRA dans workspace
- dashmap 6 dans workspace

**Carries entrants S56** :

| Item | Compteur | Source |
|---|---|---|
| P2-S53-outbox non-persistant | **3/3 MANDATORY** | S53 Phase F |
| P2-S53-browse_request rate-limit | **3/3 MANDATORY** | S53 Phase G |
| P2-A-1 rand blocker upstream | 14+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-S54-forbid-deny-doc | 2/3 | S54 Phase A |
| P2-S54-lightcheck-edition-faux-positif | 2/3 | S54 Phase A |
| P2-S54-windows-test-cfg-unix | 2/3 | S54 Phase B |
| P2-S54-test-E2E-multi-noeuds | 2/3 | S54 Phase C |
| P2-S54-rustfmt-drift-sessions | 2/3 | S54 Phase D |
| P2-BUILD-TIMEOUT | 1/3 | S55 Phase B |
| P2-REMAP-PATH | 1/3 | S55 Phase B |
| P2-JITTER-SCOPE | 1/3 | S55 Phase D |
| P2-INVITE-U16-WIRE | 1/3 | S55 Phase D |

### §1.2 Ancrage roadmap

S55 a livre LT-7 Tier 1+2 (CI + build executor + quorum). S56
est le sprint qui rend le gossip resilient (outbox persistant +
rate-limit) et pose le bridge pour les apps pre-v1.0.

Roadmap pre-v1.0 (decision utilisateur 2026-05-07) :
- **S56** : gossip resilience + bridge extensions + dette pair
- **S57** : Protocol Explorer MVP + Ideas Hub MVP
- **S58** : stabilisation + tag v1.0

### §1.3 Compteurs tests entree (tip `e5d6242`)

| Suite | Count |
|---|---|
| Rust nextest | 1216 |
| Rust doctests | 6 passed, 1 ignored |
| Vitest | 250 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 6/6 |
| **Total** | **~1472** |

**Post-S56 attendu** : ~1490+ (outbox persistence tests, rate-limit
tests, bridge extensions tests).

### §1.4 Pre-launch protocol policy (rappel)

Phases A-B touchent la DB (nouvelle table gossip_outbox) et le
gossip loop (rate-limit). Pas de changement wire format — les
enveloppes gossip sont deja serializees en bytes opaques, la
persistence est transparente. Phase C ajoute des methodes bridge
(protocole interne postMessage, pas de wire format P2P).

---

## §2 Goal

Sprint 56 rend le noeud SBFB resilient au restart (outbox persistent)
et resistant au spam (rate-limit per-peer), puis ouvre le bridge a
5 nouvelles methodes pour supporter les apps pre-v1.0.
**Critere SMART : 24+ rows fail-fast verts au verification.md, mesure
binaire au Phase E wrap-up. Outbox survit un restart daemon dans
test. Rate-limit rejette un peer > quota dans test. 5 methodes
bridge fonctionnelles dans test.**

---

## §3 Phase 0 — Audit gate S55

**DEJA JOUE** : commit `e5d6242` PASS
(0 P0, 0 P1, 2 P2, 2 P3).
Audit findings dans `.planning/archive/v1.2/sprint55_audit_findings.md`.
13 carries documentes pour S56 (cf. §1.1 ci-dessus).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Outbox persistence via table SQLite dans coordinator.db

**Retenu** : migration M6 ajoute une table `gossip_outbox(id INTEGER
PK AUTOINCREMENT, envelope BLOB NOT NULL, added_at INTEGER NOT NULL)`
dans coordinator.db. Le daemon charge l'outbox au demarrage via
`load_outbox()` et insere chaque nouvelle enveloppe via
`insert_outbox(envelope)`. Le replay existant (NeighborUp, browse,
periodic) reste identique — il opere sur le Vec en memoire, qui est
pre-rempli depuis la DB au boot.

**Rejete** :
- JSONL append-only (`~/.sbfb/outbox.jsonl`) : pas de cleanup
  atomique sans reecrire le fichier. Performance degradee avec
  volume (parse JSON par ligne). Pas de transactions.
- DB autonome `gossip_outbox.db` : fichier supplementaire. Le
  daemon accede deja a coordinator.db (validator_loop). Zero
  benefice de separation.
- iroh-docs pour persistence : overhead protocole P2P pour de la
  persistence locale. iroh-docs est pour la replication, pas le
  stockage local.

**Implications code** : `crates/nexus-coordinator-rs/src/db.rs`
(migration M6 + helpers), `crates/nexus-shell-daemon/src/runtime.rs`
(load at boot + insert on publish). Tests unitaires DB + test
integration restart.

### D2 — Rate-limit per-peer browse_request via governor GCRA

**Retenu** : nouveau struct `BrowseRequestLimiter` dans
`crates/nexus-shell-daemon-core/src/browse_limiter.rs`. Wrap un
`DefaultKeyedRateLimiter<String>` keyed par NodeId hex. Quota
hardcode 10 req/min/peer. Injection a la reception browse_request
dans runtime.rs : si `limiter.check(peer_id).is_err()` → drop
silencieux + log debug. Drop = pas de replay outbox pour ce peer
cette fois.

**Rejete** :
- DashMap<NodeId, Instant> grace period : pas de burst support,
  code custom a maintenir vs governor battle-tested. governor a
  deja expire les entrees automatiquement.
- Token bucket custom : reinvente governor sans la maturite
  (0.10.2, 3 ans de stabilite, deja utilise S21).
- Rate-limit cote emetteur (pas recepteur) : impossible — on ne
  controle pas le code des peers distants.
- Policy hot-reload TOML : overkill pour le MVP. Quota hardcode
  suffisant pre-v1.0. Hot-reload scope S57+.

**Implications code** : `crates/nexus-shell-daemon-core/src/browse_limiter.rs`
(NEW), `crates/nexus-shell-daemon/src/runtime.rs` (injection 1 ligne).
Tests unitaires limiter + test integration rejection.

### D3 — Bridge extensions 5 methodes postMessage

**Retenu** : etendre le bridge postMessage avec 5 methodes pour
supporter les apps pre-v1.0 (Protocol Explorer, Ideas Hub) :
1. `storage_list` — lister les cles par prefixe (pagination)
2. `storage_delete` — supprimer une cle du storage local
3. `identity_pubkey` — obtenir la cle publique Ed25519 du noeud
4. `node_status` — obtenir l'etat du daemon (peers, uptime, version)
5. `browse_list` — lister les apps disponibles sur le reseau

Chaque methode suit le pattern existant : handler React shell
(postMessage listener) → HTTP GET/DELETE daemon → reponse JSON →
postMessage callback avec correlationId.

**Rejete** :
- Differer a S57 : les apps pre-v1.0 dependent du bridge etendu.
  Reporter bloque le planning (S57 = apps, pas infra bridge).
- Endpoints REST sans bridge : les iframes sont sandboxed
  (`sandbox="allow-scripts"` sans `allow-same-origin`, CSP
  `connect-src 'none'`). Elles n'ont PAS d'acces HTTP direct au
  daemon. Le bridge postMessage est le seul canal.
- WebSocket au lieu de postMessage : complexite significative
  (handshake, reconnexion, heartbeat). postMessage est synchrone
  frame-to-frame, pas de connexion a gerer.

**Implications code** : `web/src/` bridge handler (extend switch),
`crates/nexus-shell-daemon/src/http.rs` (3-5 nouveaux endpoints),
`web/public/sbfb-bridge.js` (5 nouvelles fonctions SDK).

### D4 — P2 dette batch selection (5 items)

**Retenu** : resoudre 5 items P2 en phase dette pair :
- `P2-S54-forbid-deny-doc` (2/3) : documenter dans PATTERNS.md
  pourquoi deny au lieu de forbid pour certains lints (cfg_attr
  test allow incompatible avec forbid). Doc-only.
- `P2-S54-rustfmt-drift-sessions` (2/3) : investiguer le drift
  rustfmt entre versions (1.94 vs 1.95) et documenter la solution
  (pin version dans CI ou configurer rustfmt.toml).
- `P2-S54-lightcheck-edition-faux-positif` (2/3) : corriger le
  faux-positif du hook lightcheck lie a l'edition 2024.
- `P2-BUILD-TIMEOUT` (1/3) : ajouter un timeout configurable a
  `execute_build()` (Duration param + default 30min).
- `P2-REMAP-PATH` (1/3) : ajouter `--remap-path-prefix` au
  cargo build dans le build executor.

**Rejete** :
- Inclure E2E multi-noeuds (2/3) : test d'integration multi-node
  iroh. > 500 LOC, necessite infra VPS ou Docker multi-daemon.
  Carry S57 (3/3 MANDATORY).
- Inclure windows-test-cfg-unix (2/3) : investigation CI
  cross-platform significative. Carry S57 (3/3 MANDATORY).
- Inclure JITTER-SCOPE (1/3) : test integration gossip timing,
  faible priorite, risque faible.
- Inclure INVITE-U16-WIRE (1/3) : documentation post-v1.0, pas
  urgent pre-launch.

**Implications code** : `docs/rust/PATTERNS.md` (forbid-deny doc),
`.claude/hooks/` ou `.claude/skills/` (lightcheck fix),
`build_executor.rs` (timeout + remap-path).

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ⚠️, D3 ✅, D4 ⚠️.
Rigor signal G4 satisfait (2 ⚠️ sur 4).

D2 ⚠️ (delivered_from availability) : acknowledge — le reviewer
n'a pas trouve `delivered_from` dans runtime.rs. Verification
factuelle : le champ EST disponible dans `GossipEvent::Message {
content, delivered_from }` (gossip.rs:220), destructure a
runtime.rs:1012. Le concern est resolu. governor GCRA keyed par
`delivered_from` est viable.

D4 ⚠️ (task roster vs decision architecturale) : acknowledge —
meme observation que S55 D4. Le §D4 documente la **selection**
(quels items inclure dans le batch dette et lesquels reporter) ce
qui est une decision de priorisation. Le framing "Day 0" est un
abus de format kickoff, pas un abus de substance.

---

## §5 Plan Phase outline A..E

### Phase A — Outbox persistent (3/3 MANDATORY)

**But** : rendre l'outbox gossip persistent entre restarts daemon.
CLOSE P2-S53-outbox non-persistant (3/3 MANDATORY).

- Migration M6 : table gossip_outbox dans coordinator.db
- Helpers DB : load_outbox() + insert_outbox() + clear_outbox()
- Runtime boot : pre-remplir Vec depuis DB
- Runtime publish : insert DB en plus du Vec push
- Test : outbox survit un restart (mock open DB → insert → reload)
- Commit : `feat(sprint56): Sprint 56 Phase A — outbox gossip
  persistent SQLite`

### Phase B — Browse_request rate-limit (3/3 MANDATORY)

**But** : proteger le daemon contre le spam browse_request per-peer.
CLOSE P2-S53-browse_request rate-limit (3/3 MANDATORY).

- BrowseRequestLimiter : governor GCRA keyed par NodeId
- Injection runtime.rs : check avant replay outbox
- Log debug sur rejection, pas de reponse erreur (drop silencieux)
- Test : peer > 10 req/min → rejection
- Test : peer sous quota → replay OK
- Commit : `feat(sprint56): Sprint 56 Phase B — browse_request
  rate-limit governor per-peer`

### Phase C — Bridge extensions 5 methodes

**But** : etendre le bridge postMessage pour les apps pre-v1.0.

- storage_list : endpoint + handler + SDK
- storage_delete : endpoint + handler + SDK
- identity_pubkey : endpoint + handler + SDK
- node_status : endpoint + handler + SDK
- browse_list : endpoint + handler + SDK
- Tests : Vitest pour chaque handler bridge
- Commit : `feat(sprint56): Sprint 56 Phase C — bridge extensions
  5 methodes (storage_list + storage_delete + identity + status +
  browse)`

### Phase D — Dette pair (P2 batch)

**But** : resoudre 5 items P2 pour prevenir l'accumulation.
Sprint pair → phase dette obligatoire (§6.2.1 Regle 1).

- forbid-deny-doc : PATTERNS.md §P-NN documentation
- rustfmt-drift : investigation + fix/documentation
- lightcheck-edition : hook fix ou exemption documentee
- BUILD-TIMEOUT : Duration param + default 30min build_executor.rs
- REMAP-PATH : --remap-path-prefix env build_executor.rs
- Commit : `feat(sprint56): Sprint 56 Phase D — dette pair P2
  batch (forbid-deny + rustfmt + lightcheck + build-timeout +
  remap-path)`

### Phase E — Wrap-up + verification + audit plan S57

**But** : cloturer le sprint.

- CLAUDE.md : update S56 CLOSED, carries S57
- HARDENING_ROADMAP : update last_validated S56
- verification.md : 24+ fail-fast rows
- sprint57_audit_plan.md : 7+ tracks
- Commit : `chore(sprint56): Phase E — wrap-up + verification +
  audit plan S57`

---

## §6 Items carry/dette

### Carries confirmes S56

- [phase A] **P2-S53-outbox non-persistant** 3/3 MANDATORY :
  **ADRESSE Phase A** → CLOSE attendu.
- [phase B] **P2-S53-browse_request rate-limit** 3/3 MANDATORY :
  **ADRESSE Phase B** → CLOSE attendu.
- [phase D] **P2-S54-forbid-deny-doc** 2/3 :
  **ADRESSE Phase D** → CLOSE attendu.
- [phase D] **P2-S54-rustfmt-drift-sessions** 2/3 :
  **ADRESSE Phase D** → CLOSE attendu.
- [phase D] **P2-S54-lightcheck-edition-faux-positif** 2/3 :
  **ADRESSE Phase D** → CLOSE attendu.
- [phase D] **P2-BUILD-TIMEOUT** 1/3 :
  **ADRESSE Phase D** → CLOSE attendu.
- [phase D] **P2-REMAP-PATH** 1/3 :
  **ADRESSE Phase D** → CLOSE attendu.
- [carry] **P2-A-1** rand blocker upstream 14+/3 : exemption externe.
- [carry] **P2-AUDIT-2** iroh transitives : herite pin 0.98.
- [carry] **P2-S54-windows-test-cfg-unix** 2/3 : carry S57
  (investigation CI cross-platform). **Attention : 3/3 MANDATORY S57.**
- [carry] **P2-S54-test-E2E-multi-noeuds** 2/3 : carry S57
  (> 500 LOC, infra multi-daemon). **Attention : 3/3 MANDATORY S57.**
- [carry] **P2-JITTER-SCOPE** 1/3 : carry S57.
- [carry] **P2-INVITE-U16-WIRE** 1/3 : carry S57.

### Carries residuels post-S56

| Item | Compteur S57 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 15+/3 | exemption |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-S54-windows-test-cfg-unix | 3/3 **MANDATORY** | S54 Phase B |
| P2-S54-test-E2E-multi-noeuds | 3/3 **MANDATORY** | S54 Phase C |
| P2-JITTER-SCOPE | 2/3 | S55 Phase D |
| P2-INVITE-U16-WIRE | 2/3 | S55 Phase D |

**Attention S57 impair** : windows-test et E2E multi-noeuds passent
a 3/3 MANDATORY. S57 DOIT les inclure dans le plan obligatoire
(pas de phase dette impaire, mais items obligatoires).

---

## §7 Scope cuts

1. **LT-7 Tier 3** (N builders, auto-deploy) — S57+
2. **E2E multi-noeuds automatise** — S57 (2/3→3/3 MANDATORY)
3. **windows-test-cfg-unix CI** — S57 (2/3→3/3 MANDATORY)
4. **Protocol Explorer MVP** — S57
5. **Ideas Hub MVP** — S57
6. **Outbox rotation/compaction TTL** — S57+ (MVP = simple table)
7. **Rate-limit policy hot-reload TOML** — S57+ (quota hardcode)
8. **Bridge batch operations** — S57+
9. **Podman rootless build sandbox** — S57+
10. **Build log streaming** — S57+
11. **P2-JITTER-SCOPE test integration** — S57
12. **P2-INVITE-U16-WIRE doc post-v1.0** — post-v1.0
13. **LT-1 Kudos-v2 fairness reform** — S58+

---

## §8 Tracabilite scope (S55 → S56)

| S55 scope cut | S56 disposition |
|---|---|
| Outbox persistant fichier | **Phase A** (SQLite au lieu de fichier) |
| Browse_request rate-limit | **Phase B** |
| Test E2E multi-noeuds automatise | Scope cut reporte S57 (3/3 MANDATORY) |
| Windows test cfg(unix) CI | Scope cut reporte S57 (3/3 MANDATORY) |
| forbid-deny-doc PATTERNS | **Phase D** dette |
| Lightcheck edition faux-positif | **Phase D** dette |
| rustfmt drift sessions | **Phase D** dette |
| Pre-v1.0 apps Protocol Explorer + Ideas Hub | S57 (bridge prerequis = **Phase C**) |
| LT-1 Kudos-v2 fairness reform | Scope cut S58+ |
| LT-7 cross-platform builds | S57+ |
| LT-7 podman rootless sandbox | S57+ |
| LT-7 build log streaming | S57+ |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Migration M6 gossip_outbox conflit avec quorum M5 | Low | Low | Tables independantes, migration additives. rusqlite_migration gere l'ordre. |
| R2 | Governor per-peer accumule entrees peers deconnectes | Low | Low | GCRA expire automatiquement les entrees inactives (no-op clock-based). |
| R3 | Bridge extensions nécessitent endpoints daemon HTTP inexistants | Medium | Medium | Verifier existence avant Phase C. browse_list peut necessiter un nouvel endpoint si le BrowseAggregator n'expose pas de route. |
| R4 | Rate-limit trop agressif degrade browse UX | Low | Medium | Quota conservateur 10/min. Un refresh utilisateur normal < 1/min. |
| R5 | 2 items 2/3 deviennent 3/3 MANDATORY S57 | Certain | Medium | Documente, S57 plan obligatoire. S57 est impair (pas de phase dette) mais les 2 items < 500 LOC individuel. |

---

## §10 Audit gate pattern — rappel

Phase 0 S55 jouee (PASS `e5d6242`). Phase E produira
sprint57_audit_plan.md pour la session fraiche S57.

---

## §11 Checkpoint de validation

1. **D1** : Outbox persistence via SQLite coordinator.db ?
   → oui (DB deja ouverte dans runtime, migration pattern valide)
2. **D2** : Rate-limit via governor GCRA keyed ?
   → oui (governor 0.10.2 dans workspace, pattern S21 valide)
3. **D3** : Bridge extensions 5 methodes postMessage ?
   → oui (bridge existant extensible, pattern handler clair)
4. **D4** : P2 dette batch 5 items ?
   → oui (2 code quick + 3 docs/process, sprint pair dette obligatoire)
