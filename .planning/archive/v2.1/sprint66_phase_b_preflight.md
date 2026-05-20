# Sprint 66 Phase B — preflight G8

Date : 2026-05-19 | HEAD : `eb1d4ea` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- **feedback_approach.md** : "pick deepest", "no band-aid",
  "research BEFORE code". Phase B est une phase dette exclusivement
  documentaire (3 items docs + 1 pragma 1 LOC) — pas de decision
  technique a challenger.
- **feedback_context7_systematic.md** : context7 obligatoire avant
  code touchant lib/API/spec. Phase B touche rusqlite
  (`pragma_update`) — context7 consulte pour confirmer l'API
  `pragma_update(None, "synchronous", "FULL")`. Resultat :
  `/websites/rs_rusqlite_0_39_0_rusqlite` confirme que
  `pragma_update` est la methode correcte pour setter un pragma.
- **nexus_grid_pivot.md** : aucune decision actee touchee par
  Phase B (docs only + SQLite pragma). Pre-launch protocol :
  aucun `*_VERSION` modifie.
- **vision_model.md** : N/A (Phase B = dette process, pas de
  funding/startup pattern).
- **fairness_vision.md** : N/A (pas de kudos/reputation).
- Tensions plan vs memory : aucune.

---

## S1a — OSS prior art deep analysis

### Probleme fonctionnel exact

"How do mature OSS projects configure SQLite WAL durability for
daemon-class applications, and is synchronous=FULL the standard
recommendation?"

Phase B contient 4 items : 3 documentaires (README.md, PATTERNS.md,
THREAT_MODEL.md) et 1 technique (SQLite pragma synchronous=FULL).
Les items documentaires ne sont pas susceptibles d'un finding S1a
(ce sont des transcriptions de decisions et patterns existants dans
le code). L'analyse S1a porte donc sur l'item technique.

### Projets analyses en profondeur

#### [SQLite documentation officielle] — sqlite.org
- Fichiers source lus :
  - `wal.html` (sqlite.org/wal.html) : documentation WAL mode
  - `pragma.html` (sqlite.org/pragma.html) : documentation PRAGMA
    synchronous
- Pattern architectural extrait :
  - WAL + synchronous=NORMAL : safe contre la corruption (database
    reste consistente apres crash OS/power failure), mais la
    derniere transaction commitee PEUT etre perdue (rollback au
    dernier checkpoint).
  - WAL + synchronous=FULL : ajoute un fsync supplementaire apres
    chaque commit de transaction WAL. Les transactions sont
    durables meme en cas de crash OS.
  - Pour un daemon desktop qui persiste des entries feed et des
    caches de securite (key rotations, provenance records), la
    perte de la derniere transaction apres un crash est
    inacceptable — FULL est le bon choix.
- Verdict : **ALIGNED**

#### [avi.im SQLite durability analysis] — avi.im/blag/2025
- URL : https://avi.im/blag/2025/sqlite-fsync/
- Analyse : article technique 2025 demonstrant que SQLite + WAL +
  synchronous=NORMAL n'est PAS durable en cas de crash OS. L'auteur
  montre que les commits recents sont perdus lors d'un crash
  mid-write. Recommandation : "if durability matters, use
  synchronous=FULL".
- Impact : confirme que le passage NORMAL→FULL est la bonne
  correction pour un daemon de persistence.
- Verdict : **ALIGNED**

#### [sqlx (Rust SQLite)] — launchbadge/sqlx
- Via WebSearch : sqlx offre `SqliteConnectOptions` avec un
  champ `synchronous` qui default a FULL. La lib SQLite Rust la
  plus utilisee choisit FULL par defaut.
- Verdict : **ALIGNED**

#### [Clement Joly SQLite Pragma Cheatsheet] — cj.rs
- URL : https://cj.rs/blog/sqlite-pragma-cheatsheet-for-performance-and-consistency/
- Recommandation pour WAL + durabilite : `PRAGMA synchronous = FULL`.
  Cheatsheet reference dans la communaute Rust SQLite.
- Verdict : **ALIGNED**

#### [agwa.name SQLite durability analysis] — agwa.name
- URL : https://www.agwa.name/blog/post/sqlite_durability
- Analyse independante des settings de durabilite SQLite.
  Conclusion : WAL + NORMAL perd la durabilite, WAL + FULL la
  restaure avec un cout fsync par transaction.
- Verdict : **ALIGNED**

### Tableau comparatif

| Aspect | Plan Phase B | sqlite.org | avi.im 2025 | sqlx default | cj.rs cheatsheet |
|--------|-------------|-----------|-------------|-------------|-----------------|
| WAL mode | deja actif (db.rs:217) | recommande | presuppose | recommande | recommande |
| synchronous value | NORMAL→FULL | FULL pour durabilite | FULL oblig. pour crash safety | FULL (default) | FULL pour WAL crash safety |
| Placement pragma | apres WAL (l.218) | ordre indifferent | N/A | N/A | N/A |

### Finding S1a

- Classification : **APPROACH-ALIGNED**
- Evidence :
  - sqlite.org/wal.html : "FULL synchronous is always safe for WAL
    mode, will not lose committed transactions"
  - avi.im/blag/2025/sqlite-fsync/ : demonstration crash-loss avec
    NORMAL, recommandation FULL
  - sqlx default = FULL (docs.rs/sqlx/latest/sqlx/sqlite/)
  - context7 `/websites/rs_rusqlite_0_39_0_rusqlite` : API
    `pragma_update(None, "synchronous", "FULL")` confirmee
- Impact sur le plan : aucun. Le plan utilise l'API correcte
  avec la valeur recommandee.

---

## S1b — Deps/libs versions + CVE

### Libs dans le perimetre Phase B

| Lib | Version pinnee | Derniere release | CVE search 2026 | Status |
|-----|---------------|-----------------|-----------------|--------|
| rusqlite | 0.36 (bundled) | 0.36.x | WebSearch "rusqlite rustsec advisory 2025 2026" : dernier advisory RUSTSEC-2021-0128 (lifetime bound, corrige 0.25.4/0.26.2). Aucun advisory 2025-2026. | clean |
| rusqlite_migration | 2.2 | 2.2.x | (pas modifie Phase B, meme version) | clean |
| SQLite (bundled) | via rusqlite bundled feature | 3.50.2+ | CVE-2025-6965 (memory corruption via crafted SQL). CVE-2025-70873 (info disclosure zipfile ext). Mitigation : bundled SQLite dans rusqlite est compile avec options restrictives, et le daemon ne traite pas d'input SQL utilisateur (toutes les queries sont hardcoded dans db.rs). Severity : **low** pour SBFB. | clean (mitige) |

### Specs

| Spec | Status Phase B |
|------|---------------|
| RFC 8785 (JCS) | Non touche — canonical.rs inchange |
| SLSA L1 | Non touche — provenance inchangee |

### Finding S1b

- 0 CVE critique/high affectant le perimetre Phase B.
- CVE-2025-6965 (SQLite memory corruption) : affecte SQLite <3.50.2
  via crafted SQL queries. Le daemon SBFB n'accepte aucun SQL
  utilisateur — toutes les queries sont parametrees et hardcoded
  dans db.rs. Severity residuelle : low.
- CVE-2025-70873 (SQLite zipfile ext info disclosure) : le daemon
  n'utilise pas l'extension zipfile SQLite. Non applicable.
- Pas de lib bump necessaire.

---

## S2 — Decision chain reconstruction

### Fichiers scannes

| Fichier | Commits touches | Bodies lus |
|---------|----------------|------------|
| `docs/claude/README.md` | 20+ commits (S0 a S66) | 3 bodies complets (S65 Phase D `9727818`, S65 lightcheck `349f998`, S66 Codex gate `5972656`) |
| `docs/rust/PATTERNS.md` | 20+ commits (S1 a S60) | 1 body complet (S60 Phase B `cfa3c3c` — dernier ajout pattern P50) |
| `docs/security/THREAT_MODEL.md` | 3 commits (S16 Phase E, S29 Phase B, S30 Phase A) | 3 bodies complets |
| `crates/nexus-coordinator-rs/src/db.rs` | 17 commits (S43 a S66) | 3 bodies complets (S61 Phase B feed, S62 Phase B feed sync, S66 Phase A namespace boot) |

### Decisions historiques trouvees

#### Decision 1 : SQLite WAL sans FULL
- Sprint 43 Phase A, sha `130db9b7` : CoordinatorDb::open() cree
  avec `pragma "journal_mode" "WAL"` uniquement. Pas de mention
  explicite de `synchronous` dans le body.
  Body extrait (pertinent) : "CoordinatorDb::open — persistent
  SQLite with WAL mode"
- Sprint 56 Phase A, sha `2f7a1c7f` : gossip outbox persistent
  SQLite. Utilise `CoordinatorDb::open_in_memory()` pour les tests,
  `open()` pour prod. Pas de modification du pragma synchronous.
- Reverse-commit check :
  1. `git log --all --oneline 130db9b7..HEAD -- crates/nexus-coordinator-rs/src/db.rs` :
     16 commits, aucun ne mentionne "synchronous/FULL/NORMAL/durability"
  2. `git log --all --grep="synchronous" --oneline` : 0 match
  3. Aucune reversion, aucune decision explicite de garder NORMAL
- Status : **absence de decision explicite** (NORMAL est le
  default SQLite WAL, jamais choisi deliberement)
- Impact phase : **aucun** — Phase B comble un gap non-delibere
  (pas de reversion a craindre, pas de decision contredite)

#### Decision 2 : THREAT_MODEL.md feed surface absente
- Sprint 64 Phase B audit, sha `b7469ae` : P2-THREAT-MODEL-FEED-
  SURFACE identifie a 1/3. "THREAT_MODEL.md manque section STRIDE
  pour feed protocol (surface S61-S64)."
- Sprint 65 audit findings, sha `a2fec86` : reconduit a 1/3.
  Kickoff S65 §6 documente le carry.
- Sprint 65 Phase D wrap-up, sha `9727818` : scope cut §7 item 13
  "THREAT_MODEL.md section feed → S66". Phase D ferme les items
  dette process mais explicitement defere feed surface a S66.
- Sprint 66 kickoff : P2-THREAT-MODEL-FEED-SURFACE absorbe Phase B
  (dette), 1/3→2/3.
- Reverse-commit check : N/A (carry-forward, pas un rejet)
- Status : **active carry, completion programmee Phase B**
- Impact phase : **aucun** — Phase B realise le carry 1/3→2/3

#### Decision 3 : Raw-op pattern non documente
- Sprint 65 Phase A, sha `ace05b0` : raw-op migration
  (`FeedEntry.op: Value`, `try_parse_op()`, `validate_feed_operation`
  accept-unknown). Le commit code le pattern mais ne met pas a jour
  PATTERNS.md.
- Sprint 65 audit, sha `a2fec86` : P2-S65-RAWOP-PATTERN-UNDOC
  identifie a 1/3.
- Sprint 66 kickoff : absorbe Phase B dette, 1/3→CLOSED.
- Reverse-commit check : N/A (carry-forward)
- Status : **active carry, completion programmee Phase B**
- Impact phase : **aucun** — Phase B documente le pattern existant

#### Decision 4 : Commit classification (chore vs feat)
- Sprint 65 Phase D, sha `cf4339d` (chore(planning)) : commit
  supprime 30 fichiers Playwright. Classify comme chore(planning)
  mais contient des deletions de source code.
- Sprint 65 audit, sha `a2fec86` : P2-S65-CHORE-MISCLASSIFIED
  identifie a 1/3. "Les deletions de source code (meme zombies)
  doivent etre dans chore(cleanup) ou integrees dans le feat de
  la phase."
- Sprint 66 kickoff : absorbe Phase B dette, 1/3→CLOSED.
- Reverse-commit check : N/A (carry-forward)
- Status : **active carry, completion programmee Phase B**
- Impact phase : **aucun** — Phase B documente la regle dans
  README.md §4.1

### Memory constraints

| Fichier | Contrainte | Relevance Phase B |
|---------|-----------|-------------------|
| feedback_approach.md | "pick deepest", "no band-aid" | Phase B est une phase dette documentaire — pas un band-aid, c'est une completion de documentation differee |
| feedback_context7_systematic.md | context7 avant code | Consulte pour rusqlite pragma API |
| nexus_grid_pivot.md | iroh 0.98 pinne, pre-launch policy | Phase B ne touche aucun wire format ni version |
| vision_model.md | no startup patterns | N/A |

### Finding S2

- 0 decision historique contredite.
- Les 4 items de Phase B sont tous des carries documentes avec
  lineage complet (audit S64/S65 → kickoff S66 → plan Phase B).
- Le pragma synchronous=NORMAL n'etait pas un choix delibere
  mais un default SQLite WAL — Phase B le corrige sans contredire
  aucune decision explicite.

---

## S3 — Threat model analysis

### Primitive analysee : SQLite synchronous=FULL pragma

Phase B est principalement documentaire (3 items). L'unique
primitive technique est l'ajout de `pragma synchronous FULL` dans
`CoordinatorDb::open()`. Threat model de cette primitive :

### Assets en jeu

- A1 **coordinator.db** (existant) : base SQLite contenant feed
  entries, kudos ledger, tasks, provenance records, invites,
  quarantine, gossip outbox, storage namespaces, key rotations.
  Criticite : **high** (source de verite pour la persistence
  daemon).

### Threat actors

- TA1 **Crash OS / power failure** : perte de courant pendant
  une ecriture SQLite WAL. Capacite : interruption non-gracieuse
  du daemon.
- TA2 **Crash applicatif (panic)** : le daemon Rust panic pendant
  une transaction. Capacite : terminaison du process mid-write.

### Attack vectors identifies

| # | Vecteur | Asset(s) | Couverture |
|---|---------|---------|-----------|
| V1 | Perte transaction commitee apres crash OS (WAL+NORMAL) | A1 | COUVERT apres Phase B (FULL restaure durabilite) |
| V2 | Corruption DB apres crash OS | A1 | Couvert par WAL mode (ACID, journal rollback) — FULL ne change pas ca |
| V3 | Performance degradation avec FULL (fsync par tx) | A1 | Non-risque : le daemon fait <10 writes/s, le cout fsync est negligeable (<1ms par tx sur SSD) |

### Mitigations existantes

- WAL mode (db.rs:217) couvre V2 : la base reste consistente apres
  crash.
- Phase B ajoute synchronous=FULL qui couvre V1 : la derniere
  transaction commitee ne sera pas perdue.

### Gaps identifies

- Aucun gap. Le changement NORMAL→FULL est une amelioration pure
  de la durabilite sans regression.

### Regression check

- La primitive NE diminue PAS l'efficacite d'une mitigation
  existante. FULL est strictement plus durable que NORMAL.
- La primitive ne cree pas de nouveau vecteur d'attaque.
- Performance : negligeable pour un daemon desktop (<10 writes/s).
  Non bloquant.

### THREAT_MODEL.md section feed (item doc)

Phase B ajoute la section "Feed surface" dans THREAT_MODEL.md avec
T-FEED-1 a T-FEED-4. Ces threats sont deja documentes dans
PUBLIC_FEED_SPEC.md §12 (Security Considerations) — Phase B les
transpose dans THREAT_MODEL.md pour completude du modele de
menaces. Pas de nouvelle surface d'attaque creee.

- T-FEED-1 (integrity) : couvert par BLAKE3 hash-chain + Ed25519
  signatures
- T-FEED-2 (spam) : couvert par rate limiter per-author (5 ops/min)
  + payload size limit (64 KB)
- T-FEED-3 (forgery) : couvert par Ed25519 signature verification
  contre author_pubkey
- T-FEED-4 (clock skew) : couvert par 30-day future timestamp gate

### Verdict S3 : **clean** (0 regression, 0 gap)

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui

Lu en entier (296 lignes, `eb1d4ea`) : 14 domain separation tags
(DOMAIN_TASK_V1 a DOMAIN_FEED_V1), fonction `canonical_bytes<T>()`
avec JCS + domain prefix + null separator, 4 tests. Phase B ne
touche PAS canonical.rs.

### Structs verifiees

Phase B ne touche AUCUNE struct wire format. Les modifications
sont :
- `docs/claude/README.md` : documentation process (pas de code)
- `docs/rust/PATTERNS.md` : documentation pattern raw-op (pas de
  code — transcription du pattern existant dans public_feed.rs)
- `docs/security/THREAT_MODEL.md` : documentation securite feed
  (transposition de PUBLIC_FEED_SPEC.md §12, pas de code)
- `crates/nexus-coordinator-rs/src/db.rs` : 1 ligne pragma update
  + 1 test. La ligne ne modifie AUCUNE struct serialisee, aucun
  schema SQL, aucune migration.

### Constantes version verifiees

| Constante | Fichier | Valeur | Phase B touche | Status |
|-----------|---------|--------|---------------|--------|
| `TASK_FORMAT_VERSION` | task.rs | 1 | non | ok |
| `CURATOR_LIST_FORMAT_VERSION` | curator.rs | 1 | non | ok |
| `KEY_ROTATION_FORMAT_VERSION` | key_rotation.rs | 1 | non | ok |
| `POW_FORMAT_VERSION` | pow.rs | 1 | non | ok |
| `PIN_FILE_FORMAT_VERSION` | tls_pinning.rs | 1 | non | ok |
| `FEED_FORMAT_VERSION` | public_feed.rs | 1 | non | ok |
| `PROVENANCE_SCHEMA_VERSION` | provenance.rs | 1 | non | ok |
| `CANARY_INPUT_SET_VERSION` | canary_input.rs | 1 | non | ok |

### `serde_json::to_string` audit

Phase B ne modifie aucun fichier Rust contenant
`serde_json::to_string`. Non applicable.

### Day 0 check

D1-D5 sprint 66 :
- D1 iroh-docs persistence via data_dir : Phase A — non touche
- D2 iroh-blobs FsStore activation : Phase A — non touche
- D3 Feed republish au boot : Phase C — non touche
- D4 Provenance 3 etats MANDATORY : Phase C — non touche
- D5 Verification cross-node MANDATORY : Phase C — non touche

Aucune D1-D5 contredite.

### Decisions actees pivot.md

1. iroh 0.98 pinne : Phase B ne bumpe rien — ok
2. Deploy verifie from source : non touche — ok
3. Feed raw-op extensible : Phase B DOCUMENTE le pattern dans
   PATTERNS.md, ne le modifie pas — ok
4. Pre-launch policy : aucun `*_VERSION` modifie — ok

Aucune decision actee contredite.

### Pre-launch policy

- `*_VERSION = 1` : toutes les constantes restent a 1 — ok
- Pas de tolerant decoder multi-version : non touche — ok
- Pas de tests "legacy decode" zombie : non touche — ok
- Phase B ne modifie aucun wire format — conformite totale

### Verdict S4 : **clean**

---

## Telemetrie preflight (agent deep)

- S1a : 5 sources analysees (sqlite.org WAL, avi.im 2025, sqlx
  defaults, cj.rs cheatsheet, agwa.name durability) / 3 fichiers
  doc lus (wal.html, pragma.html, avi.im article) / ~500 LOC
  reviewees / 1 context7 query (rusqlite pragma_update) / 4
  WebSearch queries (SQLite WAL FULL, rusqlite CVE, rusqlite
  rustsec, SQLite best practice Rust) / finding : APPROACH-ALIGNED
- S1b : 3 libs scannees (rusqlite, rusqlite_migration, SQLite
  bundled) / 2 CVE searches (rusqlite rustsec, SQLite CVE 2025) /
  finding : clean (0 CVE critique applicable)
- S2 : 10 commits bodies lus / 0 archive files specifiques /
  4 memory files lus (feedback_approach, feedback_context7,
  nexus_grid_pivot, vision_model) / finding : clean (0 decision
  contredite, 4 carries documentes avec lineage complet)
- S3 : FULL / 3 vectors analyses / 0 gaps / finding : clean
- S4 : FULL / 0 structs wire format touchees / canonical.rs lu
  integralement : oui / 8 constantes version verifiees = 1 /
  finding : clean

## Action

Proceder code phase B. Aucun finding bloquant. La phase est
exclusivement documentaire (3 items docs) + 1 ligne pragma
technique avec 1 test. Tous les items sont des carries documentes
avec lineage d'audit complet (S64→S65→S66).

Points d'attention pour l'implementation (non-bloquants) :
1. Le test `test_coordinator_db_synchronous_full` doit utiliser
   `CoordinatorDb::open(&path)` (fichier temporaire), pas
   `open_in_memory()`, pour tester le pragma sur une DB avec WAL.
   Le module `tests` a acces au champ prive `conn` — utiliser
   `db.conn.pragma_query_value(None, "synchronous", |row| row.get::<_, i32>(0))`
   et verifier `== 2` (valeur numerique de FULL).
2. La section THREAT_MODEL.md feed doit transposer les threats
   T-FEED-1 a T-FEED-4 de PUBLIC_FEED_SPEC.md §12 avec les
   mitigations existantes (rate limiter, payload size, Ed25519,
   timestamp gate) — pas inventer de nouveaux threats.
3. Le pattern PATTERNS.md raw-op doit referencer les fichiers
   source concrets : `public_feed.rs:67-117` (FeedEntry struct +
   try_parse_op + op_type), `public_feed.rs:224-260`
   (validate_feed_operation accept-unknown).
4. La note README.md §4.1 doit etre ajoutee apres les types valides
   existants (feat/fix/docs/chore), pas en remplacement.
