# Sprint 56 — Design Review

## Scoring
D1 ✅, D2 ⚠️, D3 ✅, D4 ⚠️
Rigor signal: 2/4

---

## D1 — Outbox persistence via SQLite coordinator.db

**Status**: ✅ Source solide + alternatives verifiees

**Sourcing**:
- 5 migrations existantes db.rs:15-128. M6 additif (nouvelle table,
  pas d'ALTER). WAL mode actif. Pattern valide.
- CoordinatorDb accessible dans runtime.rs via DaemonHttpState
  (http.rs:141 `pub coordinator_db: Arc<Mutex<CoordinatorDb>>`).
- M5 (task_results) independant : pas de foreign key, pas de conflit.

**Alternatives verification**:
1. JSONL append-only : pas de cleanup atomique. ✓ Rejete legitimement.
2. DB autonome gossip_outbox.db : fichier supplementaire, couplage
   deja existant. ✓ Rejete legitimement.
3. iroh-docs : overhead protocole P2P pour du stockage local.
   ✓ Rejete legitimement.

**Assessment**: Migration SQLite dans la DB existante est l'approche
la plus legere et la plus coherente avec les patterns du projet.

---

## D2 — Rate-limit per-peer governor GCRA

**Status**: ⚠️ Wire-format dependency verification needed

**Sourcing**:
- governor 0.10.2 confirme dans Cargo.toml L388, utilise S21 Phase A.
- DashMap 6 dans workspace, utilise iroh_runtime.rs.

**Concern identifie**:
- Le reviewer n'a pas trouve `delivered_from` dans runtime.rs. Apres
  verification : le champ EST disponible — `GossipEvent::Message {
  content, delivered_from }` (gossip.rs:220) destructure a
  runtime.rs:1012. Le reviewer a probablement grep sur un pattern
  trop etroit.
- `delivered_from` est un `String` (NodeId hex), directement
  utilisable comme cle governor.

**Alternatives verification**:
1. DashMap grace period : pas de burst support, code custom vs
   battle-tested. ✓ Rejete legitimement.
2. Token bucket custom : reinvente governor. ✓ Rejete legitimement.

**Assessment**: Decision techniquement solide. Le concern du reviewer
est resolu factuellement (delivered_from disponible). Governor GCRA
keyed est le bon choix.

---

## D3 — Bridge extensions 5 methodes

**Status**: ✅ Endpoints verifies + bridge extensible

**Sourcing**:
- GET /api/daemon/browse (http.rs:246 list_browse handler) ✓
- GET /api/v1/coordinator/health (http.rs:325) ✓
- Bridge protocol.ts : 4 methodes actuelles, schema Zod extensible
- 2 nouveaux endpoints requis : storage list (prefix query) +
  storage delete. Pattern existant (GET/POST /app/{name}/state/{key}).

**Alternatives verification**:
1. HTTP direct sans bridge : iframes sandboxed CSP connect-src none.
   ✓ Rejete legitimement.
2. WebSocket : over-engineered pour request/response. ✓ Rejete.

**Assessment**: Extension naturelle du bridge existant. Pas de risque
architectural.

---

## D4 — P2 dette batch (5 items)

**Status**: ⚠️ Task roster, pas decision architecturale

**Sourcing**:
- 5 items verifies dans les audit trails S54/S55 avec compteurs
  corrects (2/3 et 1/3).
- Scope total < 100 LOC (docs + code mecanique).

**Concern identifie**:
- Le D4 documente une selection de priorisation, pas une decision
  architecturale. Le framing "Day 0" est un abus de format kickoff
  (meme observation que S55 D4). Acknowledge : la selection est
  pertinente mais le format est le mauvais vehicule.
- Si un item balloon (ex: rustfmt drift necessite config CI multi-
  couche), Phase D overrun. Mitigation : scope cut au niveau item
  si > 50 LOC individuel.

**Assessment**: Priorisation correcte. Sprint pair dette obligatoire.
