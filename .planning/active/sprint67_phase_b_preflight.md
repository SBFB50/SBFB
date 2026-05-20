# Sprint 67 Phase B — preflight G8

Date : 2026-05-20 | HEAD : `4ee93ab` | Verdict : **EXECUTE plan-as-is**

---

## Memory consultation (Step 1.5)

- **feedback_approach.md** : pick deepest technical option, research before code, G8 procedural gate, OSS prior art obligatoire. Phase B conforme — FTS5 est la decision la plus pragmatique validee par SYNTHESIS §4.3 avec Tantivy en gate post-S75.
- **feedback_context7_systematic.md** : context7 obligatoire pour toute lib touchee. Kickoff a deja fait 3 queries context7 (sqlite, rusqlite, rusqlite vtab). Preflight deep a ajoute 2 queries supplementaires (prepare_cached pattern, bundled FTS5 flags).
- **vision_model.md** : solo maintainer, zero dep externe si possible. FTS5 via rusqlite bundled = zero dep ajoutee. Conforme.
- **nexus_grid_pivot.md** : decisions actees incluent D1 roadmap v4 (FTS5 d'abord, Tantivy gate post-S75). Aucune tension.
- Tensions plan vs memory : aucune.

---

## S1a — OSS prior art deep analysis

### Probleme fonctionnel exact

"How do mature local-first/P2P applications implement full-text search over a local SQLite database with FTS5, including query sanitization and incremental indexing?"

### Projets analyses en profondeur

#### [Projet 1] — AIngram (https://github.com/bozbuilds/AIngram)
- Fichiers source identifies : SQLite FTS5 + sqlite-vec hybrid search, local HTTP server (localhost:7750)
- Pattern architectural : FTS5 virtual table linked to content table, incremental indexation on insert, Ed25519-signed entries, MCP server bridge
- Edge cases geres : hybrid retrieval (FTS5 + vector), auto-indexing, health checks
- Verdict : ALIGNED — same pattern daemon local + FTS5 + incremental index

#### [Projet 2] — PocketBase FTS5 (https://github.com/pocketbase/pocketbase/issues/5225)
- Pattern architectural : triggers pour update FTS index, hooks alternatifs, FTS5 virtual table dynamically updatable
- Discussion communautaire montre que FTS5 triggers sont le pattern recommande pour indexation incrementale dans un contexte SQLite embarque
- Cautionary note : PocketBase author warned triggers could conflict with framework internals — hooks preferred
- Verdict : ALIGNED — SBFB plan utilise des appels explicites (index_browse_entry, index_feed_entry) pas des triggers SQLite, ce qui est le pattern plus propre

#### [Projet 3] — Obsidian Hybrid Search (https://github.com/flowing-abyss/obsidian-hybrid-search)
- Pattern architectural : FTS5 BM25 keyword search with column weights (title 10x, aliases 5x, content 1x), fuzzy trigram search, incremental updates < 10s
- Performance reference : 49,746 chunks from 16,894 files in 83 MB, full reindex 4 minutes
- Verdict : ALIGNED — validates FTS5 performance adequate for SBFB's < 500 entries pre-launch

#### [Projet 4] — SQLite FTS5 security patterns (codestudy.net/blog/how-to-escape-string-for-sqlite-fts-query)
- Pattern architectural : wrap user input in double-quotes + escape internal double-quotes to force literal phrase matching
- Security : FTS5 query syntax injection is a real concern distinct from SQL injection — special characters (`"`, `*`, `:`, `-`, `+`, `~`, `(`, `)`) and keywords (AND, OR, NOT) can alter query logic
- Recommended sanitization : `fn escape_fts(input: &str) -> String { format!("\"{}\"", input.replace('"', "\"\"")) }`
- Verdict : ALIGNED with plan's sanitizer approach, but plan should specify FTS5 query syntax escaping (not just HTML strip + NUL bytes)

#### [Projet 5] — SQLite FTS5 official reference (sqlite.org/fts5.html + deepwiki.com/sqlite/sqlite)
- Architecture : LSM-tree inverted index in shadow tables, unicode61 tokenizer (case-insensitive + diacritics removal), bm25() ranking, snippet() excerpts
- Performance : O(log n) lookups, adequate for < 50K documents
- Shadow tables : %_data, %_idx, %_content, %_docsize, %_config — auto-managed by FTS5
- Verdict : ALIGNED — confirms plan's CREATE VIRTUAL TABLE + MATCH + bm25() approach

### Tableau comparatif

| Aspect | Plan Phase B | AIngram | PocketBase FTS5 | Obsidian Hybrid |
|--------|-------------|---------|-----------------|-----------------|
| Index engine | FTS5 virtual table | FTS5 + sqlite-vec | FTS5 triggers | FTS5 + column weights |
| Indexation | Explicit calls (boot + on-insert) | Auto-indexing daemon | Triggers on content table | Incremental file scan |
| Query sanitization | HTML strip + NUL bytes | N/A (local only) | N/A (framework-level) | N/A (local UI) |
| Ranking | bm25() | BM25 + vector RRF | BM25 | BM25 weighted columns |
| Tokenizer | unicode61 | unicode61 | Default | unicode61 |

### Finding S1a

- **Classification** : APPROACH-ALIGNED
- **Evidence** : 5 projets analyses confirment le pattern FTS5 virtual table + MATCH + bm25() + explicit indexation. Aucun projet de reference equivalent n'a abandonne cette approche pour un moteur externe (Tantivy/MeiliSearch) tant que le corpus est < 50K docs.
- **Observation non-bloquante** : le plan specifie "sanitizer strip HTML, reject NUL bytes" pour T-SEARCH-INJECTION. L'analyse OSS (codestudy.net) montre qu'il faut aussi echapper la syntaxe FTS5 elle-meme (double-quote wrapping). Le code devrait wrapper les queries utilisateur dans des double-quotes echappees (`"user input"`) en plus du strip HTML/NUL. Ceci est une amelioration de l'implementation dans le scope existant, pas un changement d'approche — le code peut implementer les deux sanitizations.
- **Impact sur le plan** : aucun changement structurel. Note technique pour l'agent codeur : `search.rs` devrait implementer `fn sanitize_query(input: &str) -> String` avec (1) strip NUL bytes, (2) strip HTML tags, (3) double-quote escape + wrapping pour forcer le mode phrase literal FTS5.

---

## S1b — Deps/libs versions + CVE

### Libs scannees

| Lib | Version pinned | Derniere dispo | Delta | CVE | Status |
|-----|---------------|----------------|-------|-----|--------|
| rusqlite | 0.36 | 0.39.0 | 3 minor | RUSTSEC-2021-0128, RUSTSEC-2020-0014 (anciens, fixes) | clean — pas de breaking change affectant FTS5 API entre 0.36 et 0.39 |
| rusqlite_migration | 2.2 | 2.2 | 0 | aucun | clean |
| libsqlite3-sys (transitive) | 0.34.0 (SQLite 3.49.2) | 0.37.0 (SQLite 3.51.3) | 3 minor | **CVE-2025-6965** (CVSS 9.8) | voir analyse ci-dessous |
| serde_json | workspace | - | 0 | aucun recent | clean |
| axum | workspace | - | 0 | aucun recent | clean |

### CVE-2025-6965 — analyse de contexte

- **Vulnerability** : memory corruption in SQLite < 3.50.2 when aggregate terms exceed available columns (CWE-197 Numeric Truncation)
- **CVSS** : 9.8 Critical (NVD), 7.2 High (Google)
- **Fix** : SQLite 3.50.2+ (libsqlite3-sys 0.35+)
- **Exploitabilite dans le contexte SBFB** : **LOW**. Le daemon controle toutes les requetes SQL. L'endpoint search prend un parametre `q` (string) qui est passe a FTS5 MATCH via parametre bind (`?1`). L'utilisateur ne peut PAS injecter de SQL arbitraire ni de requete aggregate. Le CVE requiert des requetes aggregate crafted qui depassent le nombre de colonnes — cette surface n'est pas exposee par le search endpoint ni par aucune autre route HTTP du daemon.
- **Herite** : ce CVE existait avant Phase B. Phase B ne change pas la version SQLite.
- **Classification** : **Non-bloquant** — CVE high/critical mais mitigation alternative solide (toutes les requetes SQL sont ecrites dans le code, parameterisees, pas de SQL dynamique). La version rusqlite 0.36 est maintenue volontairement (iroh 0.98 compatibility — carry P2-AUDIT-2). Un upgrade rusqlite independant vers 0.39 pourrait etre considere en sprint pair dette, mais ne bloque pas Phase B.

### Specs verifiees

- SQLite FTS5 spec (sqlite.org/fts5.html) : pas de revision majeure depuis la version lue au kickoff (2026-05-20). `unicode61` tokenizer, `rank` column, `bm25()`, `snippet()` confirmes.
- rusqlite `bundled` feature : build.rs confirme `-DSQLITE_ENABLE_FTS5` active (kickoff WebSearch github.com/rusqlite/rusqlite/blob/master/libsqlite3-sys/build.rs).

### Finding S1b

Aucun finding bloquant. 1 finding non-bloquant : CVE-2025-6965 affecte SQLite 3.49.2 bundled mais non-exploitable dans le contexte SBFB (pas de SQL dynamique). Carry recommande pour sprint pair dette (upgrade rusqlite 0.36→0.39 avec libsqlite3-sys 0.37.0 = SQLite 3.51.3, post-CVE).

---

## S2 — Decision chain reconstruction

### Fichiers scannes

- `crates/nexus-coordinator-rs/src/db.rs` : 18 commits, bodies complets des 5 plus recents lus
- `crates/nexus-shell-daemon/src/http.rs` : 18 commits, bodies complets des 5 plus recents lus
- `crates/nexus-shell-daemon/src/runtime.rs` : 18 commits, bodies complets des 3 plus recents lus
- `web/src/bridge/protocol.ts` : 6 commits, bodies complets des 3 plus recents lus
- `docs/security/THREAT_MODEL.md` : 3 commits, bodies complets des 3 lus

### Decisions historiques trouvees

#### Decision 1 : FTS5 vs Tantivy — choix FTS5 d'abord

- Sprint 67 kickoff, sha `d477d81` : D1 FTS5 search @protocole gelee
  Body extrait : "D1 FTS5 search @protocole (M15, search.rs, endpoint)"
- SYNTHESIS (sha `276173ac`) : "Decision moteur : FTS5 d'abord, pas Tantivy"
  Rationale : "Zero dep ajoutee, tables SQLite existantes, volume < 50K pre-launch"
- Reverse-commit check : aucune reversion trouvee
- Status : **active**
- Impact phase : aucun — Phase B suit exactement cette decision

#### Decision 2 : Feed raw-op extensible (pattern P51)

- Sprint 65 Phase A, sha `ff5e3497` : raw-op migration, `FeedEntry.op` = `serde_json::Value`
  Body extrait : "raw-op migration S65 preserves the same JCS output"
- Sprint 67 Phase A, sha `4ee93ab` : CuratorVouched/CuratorDisendorsed ajoutees sans bump FEED_FORMAT_VERSION
  Body extrait : "FEED_FORMAT_VERSION = 1 preserve. CuratorVouched/CuratorDisendorsed = nouvelles variantes PublicFeedOperation, PAS de bump (raw-op P51)"
- Reverse-commit check : aucune reversion
- Status : **active**
- Impact phase : Phase B indexe les feed entries via le champ `payload` (text JSON). Le raw-op pattern signifie que les operations inconnues sont stockees mais seront indexees comme JSON brut dans FTS5. Coherent et non-problematique.

#### Decision 3 : THREAT_MODEL feed surface — carry 3/3 MANDATORY

- Sprint 66 Phase B, sha `ea87547` : T-FEED-1..T-FEED-4 ajoutees a THREAT_MODEL.md
  Body extrait : "THREAT_MODEL.md feed section (T-FEED-1..4)"
- Carry P2-THREAT-MODEL-FEED-SURFACE 2/3→3/3 MANDATORY Sprint 67
- Reverse-commit check : N/A (carry en cours)
- Status : **active — Phase B doit fermer le 3/3**
- Impact phase : Phase B ajoute T-SEARCH-INJECTION, T-CURATOR-VOUCH, T-SEARCH-DOS pour fermer P2-THREAT-MODEL-FEED-SURFACE 3/3. Direct compliance.

### Memory constraints

- **feedback_approach.md** : "G8 = mecanisme procedural pour le principe pick-deepest" — Phase B suit le protocole
- **feedback_context7_systematic.md** : "context7 obligatoire pour toute lib touchee" — 5 queries context7 effectuees (kickoff 3 + preflight 2)
- **nexus_grid_pivot.md** : D1 roadmap v4 "FTS5 d'abord, Tantivy gate post-S75" — Phase B conforme
- **vision_model.md** : zero dep externe quand possible — FTS5 via bundled = zero dep. Conforme.

---

## S3 — Threat model analysis

### Primitive analysee : FTS5 search daemon-local + bridge method search

### Assets en jeu

- A8 Search index (FTS5 virtual table) : criticite **LOW** (derive des donnees publiques deja dans coordinator.db, pas de donnee nouvelle)
- A6 Project archives : criticite **HIGH** (integrite) — le search ne modifie pas les archives, seulement les indexe
- A2 Bearer token loopback : criticite **HIGH** — le search endpoint est protege par le meme middleware auth_required

### Threat actors

- TA1 Extension navigateur malveillante (AD1) : si bearer leak, peut appeler GET /api/daemon/search
- TA2 App iframe malveillante (AD5) : peut appeler `search` via bridge method
- TA3 Noeud byzantin P2P (AD3) : influence le contenu indexe via feed entries propagees

### Attack vectors identifies

1. **V1 FTS5 query syntax injection** (injection/forgery)
   - Description : un utilisateur passe une query FTS5 crafted (`*`, `OR`, `NEAR`, column filters) qui retourne plus de resultats que prevu ou cause un crash
   - Assets vises : A8 (index integrity), information leakage
   - Couverture : plan propose T-SEARCH-INJECTION (strip HTML, reject NUL bytes). L'analyse S1a ajoute : wrapper en double-quotes echappees pour forcer le mode phrase literal FTS5

2. **V2 Search DoS via queries repetees** (DoS/resource exhaustion)
   - Description : un caller flood GET /api/daemon/search pour saturer le CPU (chaque query scanne l'index FTS5)
   - Assets vises : disponibilite daemon
   - Couverture : plan propose T-SEARCH-DOS (rate limit query). Le bearer token limite l'acces au loopback. Le bridge method est dans un iframe sandbox sans network. Rate limit existant GCRA per-peer (S56) ne couvre pas le search specifiquement mais le loopback est deja limite par design (1 seul client browser).

3. **V3 Endorsement spam via CuratorVouched feed indexation** (injection)
   - Description : un attaquant publie de nombreux CuratorVouched pour poluer l'index search avec des entries spam
   - Assets vises : qualite des resultats search
   - Couverture : plan propose T-CURATOR-VOUCH. Le feed rate limiter GCRA (5 ops/min per author) et le PoW 16-bit limitent le debit. Les CuratorVouched sont validates (hex-64 pubkey + project_id).

4. **V4 Information leakage via search results** (information disclosure)
   - Description : le search endpoint retourne des snippets qui exposent du contenu pas encore visible via browse
   - Assets vises : confidentialite (low — toutes les donnees indexees sont publiques par design)
   - Couverture : les BrowseEntries et FeedEntries sont deja publiques. Le search ne revele rien de nouveau. Risque residuel nil.

5. **V5 Supply chain** (nouvelle dep)
   - Phase B n'ajoute aucune nouvelle dep (FTS5 via rusqlite bundled existant). Risque nil.

6. **V6 Temporal attacks** (race condition boot indexation)
   - Description : entre le boot et la fin de l'indexation initiale, le search retourne des resultats incomplets
   - Couverture : acceptable pour un daemon local pre-launch (< 500 entries, indexation < 100ms). Pas de garantie temps-reel requise.

### Mitigations existantes

- T-FEED-SPAM couvre V3 partiellement : GCRA 5 ops/min + 64KB limit + PoW
- Loopback auth (bearer + Host + Origin) couvre V2 : acces limite au client browser local
- Bridge whitelist couvre V2/V4 : 1 methode additionnelle dans le schema Zod, validee source iframe

### Gaps identifies

- GAP1 V1 FTS5 query syntax injection : severity **MEDIUM** — la sanitization HTML/NUL est necessaire mais insuffisante. Il faut aussi echapper la syntaxe FTS5 (double-quote wrapping). Non-bloquant car le plan prevoit deja un sanitizer — la recommandation est d'enrichir l'implementation du sanitizer, pas de changer l'architecture.

### Regression check

- La primitive FTS5 search ne diminue l'efficacite d'aucune mitigation T0-T5 existante
- La primitive ne cree pas de nouveau vecteur non couvert critique (les vecteurs identifies sont couverts par le plan + l'enrichissement sanitizer)
- Aucun nouveau T necessaire au-dela de T-SEARCH-INJECTION, T-CURATOR-VOUCH, T-SEARCH-DOS deja prevus dans le plan

### Verdict S3 : clean (1 gap severity M — FTS5 syntax escaping, couvert par note technique pour l'implementation)

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui (296 lignes)

Phase B ne touche PAS canonical.rs. FTS5 est un index interne daemon, pas un wire format. Aucune struct signee n'est modifiee.

### Structs verifiees

Phase B ne modifie aucune struct dans canonical.rs ni dans public_feed.rs. Les structs existantes restent intactes :

#### FeedEntry (public_feed.rs:102-118)
- version = 1 : OK (FEED_FORMAT_VERSION = 1, l.20)
- serde derives : OK (Serialize, Deserialize, Debug, Clone, PartialEq)
- serde(default) : present sur `pow_nonce: Option<u64>` (l.116) — rationale "runtime tolerance: local entries omit it, remote sync enforces it" documentee inline
- DOMAIN signature : OK (DOMAIN_FEED_V1, canonical.rs:199)
- JCS serialization : OK (via compute_feed_canonical_bytes, l.171)
- Phase B impact : aucune modification de FeedEntry

#### FeedEntryCanonical (public_feed.rs:126-133)
- Pas de serde(default) : correct (canonical struct, pas de champs optionnels)
- Phase B impact : aucune modification

#### PublicFeedOperation (public_feed.rs:80-87)
- serde(tag = "op_type") : OK
- 4 variantes (ReleasePublished, SourceBecameStale, CuratorVouched, CuratorDisendorsed)
- Phase B impact : aucune modification (variantes ajoutees en Phase A, pas Phase B)

### Nouvelle surface Phase B

Phase B ajoute :
1. **Migration M15** : CREATE VIRTUAL TABLE search_index USING fts5(...) — c'est une table interne daemon, pas un wire format. Pas de `*_VERSION` a bumper.
2. **search.rs module** : nouveau module dans nexus-coordinator-rs. Pas de struct serialisee/signee. Fonctions pures (index + query).
3. **GET /api/daemon/search** : endpoint HTTP local. Response JSON `{results, total, took_ms}`. Pas de wire format gossip/iroh.
4. **Bridge method search** : ajout dans BridgeMethodSchema (protocol.ts). Pas de wire format — c'est un schema Zod local.
5. **THREAT_MODEL.md** : documentation securite. Pas de wire format.

Aucune nouvelle surface wire/protocole. Le search est exclusivement un service daemon-local.

### Day 0 check

- D1 FTS5 search @protocole : Phase B = implementation directe de D1. Conforme.
- D2 sbfb-manifest : non touche par Phase B.
- D3 CuratorVouched : non touche (Phase A livre). Phase B indexe ces entries.
- D4 Feed entries read : non touche (Phase A livre). Phase B utilise les entries pour l'indexation.
- D5 sbfb-factory : non touche par Phase B.
- Aucune Day 0 contredite.

### Decisions actees pivot.md

- "FTS5 d'abord, Tantivy gate post-S75" (D1 v4) : Phase B conforme
- "Feed raw-op extensible" (D4 v4) : Phase B indexe le payload JSON des raw-ops, conforme
- "@protocole d'abord, puis @dev, puis @web" (D6 v4) : Phase B = @protocole. Conforme
- Aucune decision actee contredite.

### Pre-launch policy

- `FEED_FORMAT_VERSION` = 1 : preserve (Phase B ne touche pas)
- `*_ANNOUNCEMENT_VERSION` = 1 : preserve (Phase B ne touche pas)
- Pas de nouvelle version a bumper : Phase B ajoute un index FTS5 (migration M15), pas un wire format
- Pas de tolerant decoder multi-version : N/A
- Pas de tests "legacy decode" : N/A

### Version constants grep

```
FEED_FORMAT_VERSION: u16 = 1          (public_feed.rs:20)
CURATOR_LIST_FORMAT_VERSION: u16 = 1  (curator.rs:61)
KEY_ROTATION_FORMAT_VERSION: u16 = 1  (key_rotation.rs:32)
ANNOUNCEMENT_VERSION: u16 = 1         (iroh_runtime.rs:92)
BLOB_VERSION: u8 = 0x01               (keystore.rs:108)
```

Tous a 1. Aucun bump par Phase B.

---

## Scans (all clean)

- S1a OSS prior art : 5 projets recherches (AIngram, PocketBase FTS5, Obsidian Hybrid Search, codestudy.net FTS5 security, SQLite FTS5 official), APPROACH-ALIGNED — clean
- S1b deps : 5 libs scannees, CVE-2025-6965 non-bloquant (SQLite 3.49.2, non-exploitable en contexte SBFB) — clean
- S2 historiques : 5 fichiers, ~15 commits bodies lus, 3 decisions reconstruites, 4 memory files lus — clean
- S3 threat model : FULL, 6 vectors analyses, 1 gap severity M (FTS5 syntax escaping — enrichissement implementation) — clean
- S4 wire format : FULL / VERSION=1 preserves, 0 struct modifiee, 0 Day 0 contredite — clean

---

## Telemetrie preflight (agent deep)

- Duree totale : ~25 minutes
- S1a : 5 projets OSS analyses / 5 sources web lues en profondeur / ~3000 LOC reviewees (code + docs) / 2 context7 queries / 7 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : 5 libs scannees / 3 CVE searches (rusqlite RustSec, SQLite NVD, libsqlite3-sys) / finding : non-bloquant CVE-2025-6965
- S2 : ~15 commits bodies lus / 0 archive files (mentions superficielles seulement) / 4 memory files lus integralement / finding : clean
- S3 : FULL / 6 vectors analyses / 1 gap severity M (FTS5 syntax escaping)
- S4 : FULL / 5 structs verifiees / canonical.rs lu integralement : oui (296 lignes)

---

## Action

Proceder code phase B.

Notes techniques pour l'agent codeur :
1. Le sanitizer `search.rs` devrait implementer un double-quote wrapping FTS5 en plus du strip HTML/NUL : `fn sanitize_query(q: &str) -> String` avec (a) reject NUL bytes, (b) strip HTML tags, (c) escape internal double-quotes (`"` → `""`), (d) wrap dans double-quotes (`"..."`) pour forcer le mode phrase literal.
2. Utiliser `prepare_cached()` pour les queries FTS5 frequentes (recommandation context7 rusqlite).
3. Le search endpoint est ajoute dans `authed_routes` (meme trust tier que tous les endpoints daemon).
4. CVE-2025-6965 (SQLite < 3.50.2) : non-bloquant en contexte SBFB (pas de SQL dynamique). Carry recommande pour sprint pair dette (upgrade rusqlite 0.36→0.39).
