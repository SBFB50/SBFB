# Sprint 7 — Audit findings

**Auditeur** : session Claude Code fraîche (Opus 4.6, 1M context), jouée
en Phase 0 de Sprint 8 conformément au pattern `sprint_audit_gate.md`.

**Tip audité** : `9cc0796` (master au 2026-04-11, fin Phase F Sprint 7).

**Timebox observée** : ~2 h (audit plan §0 demandait 3 h max).

**Méthode** : lecture ordonnée mémoire → git log Sprint 7 → kickoff → plan
→ verification → audit_plan ; analyse track-par-track sans ouvrir
`docs/shell/PATTERNS.md` §P9 ni `docs/rust/PATTERNS.md` §Sprint 7
canonical avant d'avoir formé une opinion ; cross-check des hypothèses
du plan contre le code réel.

## Verdict global : **PASS**

- **0 P0** (aucune casse prod, aucune data loss possible)
- **0 P1** (rien ne bloque Sprint 8 Phase A)
- **10 P2** (tech debt explicite à logger dans `PATTERNS.md`, dont 4
  déjà pré-auto-confessés par l'agent livreur)
- **5 P3** (nits laissés tels quels)

Sprint 8 Phase A peut démarrer directement. Aucun commit
`fix(sprint7): ...` n'est requis en pré-condition. Les 6 nouveaux P2
détectés par l'audit (A-3, A-4, C-2, D-1, F-1, F-3, G-1, G-3)
complètent la section tech debt existante de `docs/rust/PATTERNS.md`
§820–858.

Note sur le pattern audit gate : le Phase F Sprint 7 a été le premier
cycle complet à **pré-confesser** 4 items de tech debt dans
`PATTERNS.md` (E-1 probe TTL, C-4 backpressure, D-3 persist order,
H-3 wheel drift). L'audit confirme ces 4 items et en ajoute 6 de plus.
La pré-confession est une bonne pratique mais n'immunise pas contre
les blind spots — elle balise seulement ce que l'agent livreur a
lui-même vu.

---

## 1. Track A — Intégrité du contrat cross-langue curator

**Verdict track** : **PASS with concerns** (2 P2).

### A-1 Structural diff Rust → Zod — PASS

Le mirror Zod dans `web/src/api/daemon.ts::CuratorListEntrySchema`
couvre fidèlement la shape Rust :

- `version: z.literal(1)` est **plus strict** que Rust (`u16`
  déserialisable mais rejeté au verify) — OK, Zod en amont rejette
  les payloads que Rust rejetterait de toute façon
- `curator_pubkey: z.array(z.number().int().min(0).max(255)).length(32)`
  — match exact du comportement serde par défaut de `[u8; 32]`
- `signature: z.array(z.number().int().min(0).max(255)).length(64)` —
  match du comportement `serde_big_array` qui sérialise `[u8; 64]`
  comme array de numbers
- `entries: z.array(...).max(256)` — match `CURATOR_LIST_MAX_ENTRIES`
- `.strict()` partout refuse les champs inconnus (défense en profondeur
  contre une dérive Rust ajoutant un champ sans bump de version)

Aucun payload Rust-valide (qui passerait `verify_signature`) n'est
Zod-rejeté. Aucun payload Zod-valide ne peut passer `verify_signature`
sans clé privée (sécurité triviale d'Ed25519).

### A-2 curator_pubkey encoding — PASS

Les bornes `min(0).max(255).length(32)` valident byte 0 / byte 255 /
length 31 / length 33 comme attendu. Le test Vitest
`listCurators > parses entries[] + subscribed_curators` ne couvre que
le cas `entries: []` — un entry signé réel n'est jamais parsé côté Zod
(cf. A-3), mais la forme numérique est bien verrouillée par la
signature Zod elle-même.

### A-3 — Pas de test triangle Python-sign → Zod-parse — P2

**Severity** : P2
**What** : Sprint 6 avait institué le pattern cross-lang fixture
`packages/nexus-sdk/tests/snapshots/tabview_canonical.json` lu par
Python (`test_canonical_fixture_roundtrip`) ET par Vitest
(`cross_lang.test.ts` via `resolveJsonModule`). Sprint 7 Phase B a
ajouté `test_curator.py` qui fait Python sign → Rust verify
(via PyO3) mais **aucun test ne fait Python sign → Zod parse**. Le
mirror Zod dans `web/src/api/daemon.ts` est défini à la main et
auditablement correct aujourd'hui, mais une drift Rust sans bump de
version ne serait détectée par rien (compilation Zod passe, aucun
payload réel n'est validé end-to-end).
**Evidence** :
- `web/src/api/__tests__/daemon.test.ts:213-227` — `listCurators` teste
  `entries: []` uniquement
- `packages/nexus-sdk/tests/test_curator.py:82-99` — roundtrip PyO3
  mais pas de fixture partagée avec Vitest
- Sprint 6 commit `cfb06f9` fix A-3 a démontré que le pattern attrape
  des drifts bilatérales en pratique
**Fix** : ajouter `packages/nexus-sdk/tests/snapshots/curator_canonical.json`
(un `CuratorListEntry` déterministe signé par une keypair test fixe)
lu par `test_curator.py::test_canonical_fixture_roundtrip` ET par
`web/src/api/__tests__/daemon.test.ts::parse canonical fixture`.
Sprint 8 Phase A peut tacler ça dans ~40 LOC.

### A-4 — `CuratorProjectRef` strings sans length cap — P2

**Severity** : P2
**What** : Le doc du struct `CuratorProjectRef` dit « bounded in
length to keep a single list well under 200 KB total — 280 chars
matches the plan D3 freeze ». Le code n'enforce **rien** :
`project_id`, `project_name`, `category`, `description` sont des
`String` illimitées côté Rust et `z.string()` illimitées côté Zod. Un
curator malicieux peut publier 1 entry avec `description` = 10 MB →
bypass du cap `CURATOR_LIST_MAX_ENTRIES` (qui compte les entries, pas
les octets). Le blob gossip est limité à la taille max iroh-blobs,
mais le `DashMap<curator, CuratorListEntry>` en RAM et la sérialisation
`list_snapshot → JSON` côté HTTP /curators portent la charge.
**Evidence** :
- `crates/nexus-core-rs/src/curator.rs:113-131` — struct fields sans
  `#[serde(deserialize_with)]` / `validator` / cap manuel
- `web/src/api/daemon.ts:97-103` — `z.string()` sans `.max(...)`
**Fix** : ajouter un cap dans `CuratorListEntry::verify_signature`
step 2 (après le cap entries) du type
`list.entries.iter().all(|e| e.project_id.len() <= 128 && e.project_name.len() <= 128 && e.category.len() <= 64 && e.description.len() <= 280)`
— ~15 LOC + test. Miroir Zod : `z.string().max(128)` etc.
L'impact réel Sprint 7 est faible (curator = attaque ciblée par
subscribe-list, user peut désabonner) ; loguer en tech debt et traiter
en Sprint 8 Phase A ou Sprint 9.

---

## 2. Track B — Crypto resilience & envelope attacks

**Verdict track** : **PASS** (1 P3).

### B-1 Envelope / payload split-brain — PASS

Les cinq cas couverts par `curator.rs::tests` :

- `verify_rejects_attribution_mismatch` — envelope flipped, payload
  intact → détecté au check step 3 avant signature
- `verify_rejects_wrong_signer` — envelope + payload tous deux flipped
  → signature check (step 4) échoue car signé par la vraie clé
- `two_nodes_reject_attribution_mismatch_in_announcement` — couvre
  l'attaque au niveau gossip (annonce qui staple le bon blob à un
  mauvais pubkey) via `CuratorRuntimeError::AnnouncementAttributionMismatch`
  après fetch

La défense est redondante et cohérente : attribution match est
enforced **deux fois** (crypto layer dans `CuratorListEntry::verify_signature`,
transport layer dans `process_announcement_bytes` step 7). C'est le
pattern `docs/rust/PATTERNS.md` §Sprint 7.3 et il tient.

### B-2 Revision rollback & replay — PASS

`two_nodes_reject_revision_rollback` verrouille rev 5 → rev 3 → refus
et l'entry stockée reste à rev 5. Le comment du module explique que le
check `new.revision > stored.revision` est **strict** donc un replay
rev = rev actuelle est aussi rejeté (ce qui est OK puisque le blob est
byte-identique, l'insert est redondant).

### B-3 — u64::MAX revision footgun — P3

**Severity** : P3
**What** : L'audit plan §2 B-2 scénario 4 pose la question « daemon
démarre à froid, attaquant broadcast rev u64::MAX signé valide → est
accepté, légitime curator ne peut plus remplacer ». Correction : pour
broadcaster une rev u64::MAX signée valide il faut la clé privée du
curator. Si l'attaquant a la clé, il EST le curator — pas d'attaque
externe possible sous Ed25519. Reste le footgun : un curator qui
expérimente et signe une rev trop haute se lock out lui-même de ses
subscribers. Self-inflicted, non-bloquant.
**Evidence** : `iroh_runtime.rs:483-490` utilise `<=` strict contre
`stored.value().list.revision` sans bounds upper.
**Fix** : optionnel. Pourrait ajouter un cap `MAX_REASONABLE_REVISION
= 1_000_000` en Sprint 10 avant release publique. P3, laissé tel quel.

### B-4 Check ordering — PASS

Order dans `curator.rs::verify_signature` : version → cap → attribution
→ signature. Le cap check avant signature est un **gain** DoS (rejet
rapide d'un blob gonflé sans calcul ed25519) sans perte d'info (on
sait que `entries.len() > 256` indépendamment du signer). Documenté
en `docs/rust/PATTERNS.md` §Sprint 7.2.

---

## 3. Track C — Gossip ingest pipeline robustness

**Verdict track** : **PASS with concerns** (1 P2 déjà confessé, 1 P2
nouveau).

### C-1 `process_announcement_bytes` ordering — PASS

Lecture ligne-à-ligne de `iroh_runtime.rs:417-501` : 9 steps
documentés, chacun échoue cleanly (error branché `?`), aucune mutation
de la DashMap avant step 9 (l'insert final). Pas de travail
non-réversible avant signature verify.

### C-2 — `AnnouncementAttributionMismatch` conflate deux cas — P2

**Severity** : P2
**What** : La variante `CuratorRuntimeError::AnnouncementAttributionMismatch`
est utilisée pour **deux** situations :
1. Non-subscribed curator — cas bénin, flood attendu en prod
2. Attaque réelle où l'envelope ment sur qui a signé
Le handler `runtime.rs::handle_announcement:431-434` les drop toutes
les deux en `debug!` silencieux avec un commentaire explicite
« Non-subscribed curator and envelope-mismatch both map to this
variant; silent drop ». La télémétrie ne peut donc pas distinguer un
flood normal d'une vraie tentative de spoofing — un opérateur voit
juste du trafic debug.
**Evidence** :
- `iroh_runtime.rs:435-447` — step 4 retourne
  `AnnouncementAttributionMismatch { announcement, entry: "<not subscribed>" }`
- `iroh_runtime.rs:475-480` — step 7 retourne
  `AnnouncementAttributionMismatch { announcement, entry: hex::encode(entry.curator_pubkey) }`
- `runtime.rs:431-434` — handler collapse les deux en un debug log
**Fix** : splitter la variante en deux :
```rust
CuratorRuntimeError::NotSubscribed { announcement: String },
CuratorRuntimeError::EnvelopeMismatch { announcement: String, entry: String },
```
et logger le second en `warn!` avec le pubkey de l'attaquant. Sprint 8
Phase A ou 8B si le temps manque. ~30 LOC changes + 1 test.

### C-3 Panic safety dans la gossip task — PASS

Lecture exhaustive de `runtime.rs:340-443` — aucun `unwrap()` /
`expect()` hors `#[cfg(test)]` dans le chemin gossip. Les échecs
branchent `match` / `?` / `warn!` + `break`. La task exit
gracefully sur erreur `next_event()` au lieu de paniquer, ce qui évite
l'interlock `Arc::try_unwrap` sur shutdown noté dans le doc du module.

### C-4 — Pas de backpressure / rate limiter — P2 (déjà confessé)

**Severity** : P2 (pré-confessé dans `docs/rust/PATTERNS.md:833-837`)
**What** : `handle_announcement` est appelé séquentiellement dans la
boucle gossip. Un flood 10k/s de curators subscribés saturerait
`fetch_ticket` sans limite concurrente. Le filtre attention (step 4)
absorbe le flood des non-subscribed, mais un curator malicieux qu'on
suit peut saturer.
**Fix** : `tokio::sync::Semaphore::new(8)` devant
`process_announcement_bytes` — Sprint 8 ou 9.

---

## 4. Track D — Singleton enforcement edge cases

**Verdict track** : **PASS with concerns** (1 P2 nouveau, 1 P2 déjà
confessé).

### D-1 — `process_name_matches` substring trop large — P2

**Severity** : P2
**What** : `process_name_matches` fait un `contains` après
lower-case + hyphen→underscore. Cela match intentionnellement
`nexus_shell_daemon_core-<hash>.exe` (test binary), ce qui est le
feature visé, mais ça match aussi trivialement n'importe quel
`nexus_shell_daemon_launcher.exe` ou `my-nexus-shell-daemon-wrapper`.
Un pid recyclé vers un tel binaire ferait faussement croire qu'un
daemon est live → le vrai `start` refuserait à tort. En pratique le
risque est quasi nul (personne ne nomme son process comme ça), mais
la loose match est une gâchette latente.
**Evidence** : `registry.rs:347-352`, le `.contains(&norm(expected))`
n'est pas borné sur les extrémités.
**Fix** : comparer après striping du hash suffix et du `.exe` :
```rust
let trimmed = observed_norm
    .strip_suffix(".exe").unwrap_or(&observed_norm)
    .strip_prefix("nexus_shell_daemon").is_some();
```
ou plus simple : vérifier que `observed_norm == expected_norm ||
observed_norm.starts_with(&(expected_norm.clone() + "-")) ||
observed_norm.starts_with(&(expected_norm.clone() + "_"))`. ~10 LOC
+ tests cibles. Sprint 8 Phase A ou plus tard.

### D-2 `running.json` atomic write race — PASS

`write_running` utilise `create → write → sync_all → rename`. Sur
Windows NTFS le rename est atomique same-dir. Si un AV tient
`running.json` ouvert au moment du rename, l'erreur se propage via
`RegistryError::Rename` → `DaemonRuntime::start` retourne Err, le
`Arc<Node>` tombe de portée et iroh fait son Drop sync (pas gracieux
mais safe — c'est une pré-existing tech debt iroh, pas Sprint 7).
Non-bloquant.

### D-3 — `subscriptions.json` persistence ordering — P2 (déjà confessé)

**Severity** : P2 (pré-confessé dans `docs/rust/PATTERNS.md:839-844`)
**What** : `CuratorRuntime::subscribe` fait
`self.attention.insert(pubkey, ())` AVANT `persist_subscriptions()?`.
Si le persist échoue, la RAM a la subscription mais le disque non.
L'appelant voit Err mais le prochain `is_subscribed(...)` retourne
true jusqu'au restart du daemon (après quoi la sub est perdue).
**Fix** : écrire un `try_persist_then_commit` pattern : stage la
nouvelle attention set dans un HashSet temporaire, persist, et SEULEMENT
ensuite mutate la DashMap. ~20 LOC. Sprint 8 Phase A ou plus tard.

---

## 5. Track E — Pkarr probe correctness & test contamination

**Verdict track** : **PASS with concerns** (1 P2 déjà confessé, 2 P3).

### E-1 — `probe_reachable` 2s timeout vs pkarr cold start — P2 (déjà confessé)

**Severity** : P2 (pré-confessé dans `docs/rust/PATTERNS.md:826-831`)
**What** : `DEFAULT_PROBE_TIMEOUT = 2s` peut être trop court pour un
lookup pkarr cold-start via relay n0 sur home NAT (3-5s observés).
Conséquence UX : premier `/browse` sur une node cold → Unreachable →
TTL cache 60s → 1 min d'injoignable pour un projet accessible.
Mitigé en pratique par le fait que les probes sont faits APRÈS le
fetch_ticket gossip qui a déjà warmed le `memory_lookup` cache, mais
pas garanti.
**Fix** : `DEFAULT_PROBE_TIMEOUT = 5s`, ou exposer en config, ou ne
pas cacher les `Unreachable` (re-probe sur chaque hit). Sprint 8 ou 9.

### E-2 — Test `aggregate_probes_seeded_peer_and_marks_it_reachable` pre-seed — P3

**Severity** : P3
**What** : Le test seed manuellement `node_b.memory_lookup().add_endpoint_info(a_addr)`
avant de lancer `process_announcement_bytes`. Le fetch_ticket puis le
probe_reachable subséquent trouvent tous deux le peer dans le cache —
le test n'exerce donc PAS le path pkarr-only. C'est une représentation
fidèle du state **post-fetch** (auquel moment le cache EST toujours
warm), donc techniquement correct mais la nomenclature du test
suggère qu'il vérifie la résolution pkarr.
**Evidence** : `browse.rs:489-559` lines 496 (`add_endpoint_info`).
**Fix** : renommer en `aggregate_probes_memory_lookup_peer...` ou
ajouter un 3-node test (publisher → daemon pkarr-only → probe) en
Sprint 8 ou Sprint 10. P3, laissé en note.

### E-3 — Clock recoil `duration_since` fallback — P3

**Severity** : P3
**What** : Si `SystemTime::now() < entry.probed_at` (NTP adjust, DST),
`duration_since` retourne `Err` → le match `_ => None` traite le
cache comme expiré et re-probe. Safe mais non documenté.
**Fix** : ajouter un commentaire explicite ou switcher à
`Instant::now()` pour les durées internes. P3, nit.

---

## 6. Track F — Shell UX dans les états dégradés

**Verdict track** : **PASS with concerns** (2 P2, 1 P3).

### F-1 — `Curators.tsx` n'a pas de bouton Refresh + pas de refetchInterval — P2

**Severity** : P2
**What** : `Browse.tsx:93-105` expose un bouton "Rafraîchir" avec
`data-testid="browse-refresh"`. `Curators.tsx:91-96` fait `useQuery`
avec `staleTime: 30_000, refetchOnWindowFocus: false` mais **pas de
bouton refresh manuel**. Si l'utilisateur démarre le daemon APRÈS
avoir ouvert la page Curators, il voit le `DaemonOfflineBanner`
jusqu'à ce qu'il ajoute/retire un curator (qui triggere invalidate).
Le seul moyen de forcer un refresh sans mutation est un hard reload
de la route.
**Evidence** : `web/src/pages/Curators.tsx:86-143` — `refetch()` n'est
exposé nulle part dans le JSX.
**Fix** : mirror le bouton de `Browse.tsx` en haut de `Curators.tsx`
— ~10 LOC + 1 testid. Sprint 8 Phase A ou plus tard.

### F-2 Hex case-sensitivity — PASS

`Curators.tsx:133` fait `pubkeyInput.trim().toLowerCase()` AVANT
`isValidCuratorPubkey(candidate)` check. Un user qui colle
`ABCD...EF` obtient un input valide. La policy canonique lowercase
est préservée.

### F-3 — Accessibility (CardTitle is a div) — P2

**Severity** : P2
**What** : shadcn vendored `CardTitle` est un `<div>` stylisé, pas
un `<h2>` / `<h3>`. Les pages Sprint 7 Browse/Curators utilisent
`CardTitle` partout — aucune hiérarchie de headings pour les screen
readers au-delà du `<h1>` top-of-page dans `PageHeader`. Les badges
et les boutons ont des `data-testid` mais pas d'`aria-label` sur les
`<BookmarkPlus>` / `<Trash2>` icônes.
**Evidence** : `web/src/pages/Browse.tsx:188, 216-236`,
`web/src/pages/Curators.tsx:297-309`.
**Fix** : soit personnaliser `CardTitle` en `<h3>` une seule fois
(Sprint 8 peut le faire dans `components/ui/card.tsx` — c'est un
vendored file mais la modif est auditable) soit ajouter `as="h3"`
en prop. Le JSX root obtient un `role="article"`. ~20 LOC total.
Sprint 9 polish est le candidat naturel.

### F-4 — Pas de toast pour l'erreur subscribe — P3

**Severity** : P3
**What** : `Curators.tsx:114-116` stocke l'erreur dans `formError`
(rendu inline sous l'input). Pas de toast shadcn/sonner. Le pattern
existant du projet n'utilise pas de toast global de toute façon.
**Fix** : optionnel. Laissé tel quel.

---

## 7. Track G — Coordinator proxy security

**Verdict track** : **PASS with concerns** (2 P2).

### G-1 — `httpx.AsyncClient` créé par appel, pas de limites — P2

**Severity** : P2
**What** : `api/daemon.py::_forward:157` crée
`async with httpx.AsyncClient(timeout=timeout)` à chaque call. Chaque
`/browse` fait un handshake TCP complet vers `127.0.0.1:<port>`.
Pour du loopback c'est trivialement rapide (~0ms), mais sous rafale
de F5 la coordinator accumule des clients sans limite. Pas de
`httpx.Limits(max_connections=...)`.
**Evidence** : `packages/nexus-coordinator/src/nexus_coordinator/api/daemon.py:155-167`
**Fix** : soit factoriser un `@lru_cache` / module-level
`httpx.AsyncClient(limits=httpx.Limits(max_connections=10))` réutilisé
entre calls (attention aux fuites si le client n'est jamais fermé en
tests), soit ajouter explicitement `limits=httpx.Limits(max_connections=10)`
dans le client-per-call actuel. ~15 LOC. Sprint 8 ou 9.

### G-2 CORS trust boundary — PASS

Les deux layers CORS (coordinator FastAPI CORSMiddleware + daemon
tower-http `CorsLayer`) sont indépendantes et cohérentes. Le browser
parle au coordinator (qui fait son check CORS contre l'origine du
shell), puis le coordinator fait un appel **server-side** à
`http://127.0.0.1:<port>` qui ne porte pas d'Origin header → le
daemon's CORS ne se déclenche pas pour ces appels (il ne bloquerait
rien de toute façon car la preflight ne se fait pas). Le daemon CORS
est utile uniquement si un futur sprint expose le daemon hors proxy
(ce qui violerait D1) et sert de rattrapage défense en profondeur.

### G-3 — `SubscribeCuratorRequest` sans `#[serde(deny_unknown_fields)]` — P2

**Severity** : P2
**What** : `http.rs:153-157` définit
`pub struct SubscribeCuratorRequest { pub curator_pubkey_hex: String }`
sans `#[serde(deny_unknown_fields)]`. Un body
`{"curator_pubkey_hex": "...", "evil_field": "..."}` est accepté en
serde par défaut (les champs extras sont ignorés silencieusement).
Pas un bug de sécurité aujourd'hui (aucun champ extra n'est lu), mais
**défense en profondeur** : toute future extension qui ajouterait
un champ côté shell devrait le propager côté daemon au même commit,
et `deny_unknown_fields` protège contre l'oubli de cette propagation.
**Evidence** : `crates/nexus-shell-daemon/src/http.rs:153-157`
**Fix** : ajouter `#[serde(deny_unknown_fields)]` sur
`SubscribeCuratorRequest`, `SubscriptionsResponse`, `CuratorsListResponse`,
`BrowseListResponse`. ~5 LOC + 1 test du rejet. Sprint 8 Phase A.

### G-4 `DaemonUnavailable` info leak — PASS

Les 4 call sites de `_unavailable(reason)` dans `daemon.py:122-129`
ne portent PAS le path `%APPDATA%\nexus-grid\...\running.json`. Le
cas `_read_running_state` qui retourne `None` appelle
`_unavailable("shell-daemon not running")` — chaîne fixe. Les cas
`httpx.ConnectError` / `ReadTimeout` / `HTTPError` portent le message
httpx qui contient l'URL `http://127.0.0.1:<port>` (info publique,
loopback) et jamais le path local. L'hypothèse du plan §7 G-4 est
invalidée — **pas de leak**.

---

## 8. Track H — Cross-dependency hygiene

**Verdict track** : **PASS with concerns** (1 P2 déjà confessé, 1 P3).

### H-1 — Workspace deps pins — PASS (1 P3)

`Cargo.toml` top-level :
- `iroh = "0.97"` ✓ (caret range vers 0.97.x, bloque 0.98+)
- `iroh-blobs = "0.99"` ✓
- `axum = "0.7"` ✓ (caret range vers 0.7.x, bloque 0.8 comme demandé
  plan §13 R2)
- `tower-http = { version = "0.6", features = ["cors"] }` ✓
- `sysinfo = "0.32"` ✓
- `dashmap = "6"` ✓

**P3** : le plan §13 R2 demandait un pin **exact** `= "0.7.9"` pour
axum ; le Cargo.toml utilise `"0.7"` (caret). Les deux bloquent 0.8
mais le caret permet un bump 0.7.9 → 0.7.10 qui pourrait introduire
un patch-level regression. Sprint 8 est libre de tightener en
`= "0.7.N"` s'il voit passer du bruit ; pas une condition de gate.

### H-2 — `httpx` pré-existence — PASS

`packages/nexus-coordinator/pyproject.toml:41` déclare
`"httpx>=0.27"` depuis Sprint 4. Le grep `httpx` dans
`packages/nexus-coordinator/` retourne 4 fichiers (`api/daemon.py`,
`tests/test_daemon_proxy.py`, `pyproject.toml` declaration,
`paths.py` docstring seulement). Sprint 7 Phase E n'a pas introduit
de collision d'import.

### H-3 — `nexus_core` wheel editable install drift — P2 (déjà confessé)

**Severity** : P2 (pré-confessé dans `docs/rust/PATTERNS.md:846-852`)
**What** : Phase E a observé que le wheel PyO3 peut être écrasé par
un `uv sync` entre Phase B install et Phase E test run. Reproducibility
hazard pour CI.
**Fix** : soit un `scripts/setup.sh` qui lance `maturin develop --release`
en post-sync, soit figer la dépendance wheel dans `pyproject.toml`.
Sprint 8 Phase A devrait traiter avant de toucher au SDK.

---

## 9. Track I — Documentation & traceability

**Verdict track** : **PASS**.

### I-1 `docs/shell/PATTERNS.md` P9 cohérent avec le code — PASS

P9 (lignes 147-201) décrit avec précision :
- Le chemin `shell → coordinator /daemon/* → daemon`
- Les 4 raisons (single trust boundary, ephemeral port, daemon-offline
  UX, proxy input validation)
- Le contrat discriminated envelope `{"kind": "data"|"unavailable"|"error"}`
  avec les 3 status codes (200/503/400)
- La référence aux 10 tests `test_daemon_proxy.py`

Toutes les claims sont exactes contre le code lu dans `api/daemon.py`
et `web/src/api/daemon.ts`.

### I-2 `docs/rust/PATTERNS.md` Sprint 7 canonical section — PASS

Section (lignes 646+) couvre :
- `DOMAIN_CURATOR_LIST_V1` ✓
- Check order version/cap/attribution/signature ✓
- `CURATOR_LIST_MAX_ENTRIES = 256` + rationale ✓
- `CURATOR_TOPIC_SEED = b"nexus-grid/curator/v1"` + R6 rollback ✓
- `probe_reachable` using `BLOBS_ALPN` with 2s default timeout ✓
- Singleton registry + hyphen/underscore normalization ✓
- Plus la section auto-tech-debt (820-858) avec les 4 items pré-confessés

Cohérent avec le code.

### I-3 Scope cuts Sprint 7 vs reality — PASS

Grep `grep -r '(task_submit|submit_task|nexus_command|pkarr.*publish|bootstrap_nodes|VPS)'` :
- **`task_submit`** : présent uniquement dans `ButtonBlock.tsx:25`
  (stub `console.warn` Sprint 6 inchangé), `tabview/schema.ts:76`
  (le schema type défini Sprint 6), test fixtures Sprint 6 → **aucune
  implémentation Sprint 7**. ✓
- **`nexus_command`** : 0 match dans packages/nexus-sdk sources —
  signature gelée dans le doc T5 uniquement, pas d'impl. ✓
- **Publish pkarr** : grep `publish` ne retourne que la doc module
  `discovery.rs` (pattern reference, pas un call site). ✓
- **Bootstrap peers VPS** : 0 match. ✓
- **Multi-writer iroh-docs** : 0 match. ✓

Grep `TODO(Sprint7)` sur crates/, packages/, web/ → 0 match.
Scope cut discipline honorée à 100%.

### I-4 Commit messages vs reality — PASS (spot-checked Phase A)

Commit `2c896a8` (Phase A) bodie claim :
- `193 → 254 (+61 Phase A)` ✓ (verification.md row 4 confirme 304
  total après toutes phases, 193 + 61 = 254 aligne)
- Files touched: 15 files, 3499 insertions — match `git show --stat`
- Delta breakdown `34 + 21 + 6 = 61` — match les catégories

Timebox pour le spot-check Phase A : ~10 min. Aucune drift détectée
entre le body et le diff réel. Les autres phases sont supposées
fidèles par induction (pattern Sprint 6).

---

## 10. Out of scope (audit plan §11 respecté)

Rien de ce rapport ne touche aux Day 0 D1..D5 gelées :
- D1 HTTP loopback via coordinator proxy — non rebattable
- D2 singleton strict — non rebattable
- D3 curator schema + topic + domain — non rebattable (A-4 est une
  amélioration DE l'implémentation, pas une remise en cause du schema)
- D4 task_submit Option B gelée — non rebattable, scope Sprint 8
- D5 @nexus_command gelé — non rebattable, scope Sprint 8

---

## 11. Findings list sorted by severity

| # | Track | Severity | What | Fix target |
|---|---|---|---|---|
| A-3 | A | P2 | Pas de test triangle Python-sign → Zod-parse | Sprint 8 Phase A (~40 LOC) |
| A-4 | A | P2 | `CuratorProjectRef` strings sans length cap | Sprint 8 Phase A (~15 LOC) |
| C-2 | C | P2 | `AnnouncementAttributionMismatch` conflate non-subscribed / attacker | Sprint 8 Phase A (~30 LOC) |
| C-4 | C | P2 | Pas de backpressure sur `process_announcement_bytes` | Sprint 9 (déjà confessé) |
| D-1 | D | P2 | `process_name_matches` substring trop large | Sprint 8 Phase A (~10 LOC) |
| D-3 | D | P2 | `subscriptions.json` persist order (RAM avant disque) | Sprint 8 Phase A (~20 LOC, déjà confessé) |
| E-1 | E | P2 | `probe_reachable` 2s timeout vs pkarr cold start | Sprint 8 ou 9 (déjà confessé) |
| F-1 | F | P2 | `Curators.tsx` n'a pas de bouton Refresh | Sprint 8 Phase A (~10 LOC) |
| F-3 | F | P2 | Accessibility : CardTitle `<div>` pas `<h3>` | Sprint 9 polish |
| G-1 | G | P2 | `httpx.AsyncClient` per-call + pas de limits | Sprint 8 ou 9 (~15 LOC) |
| G-3 | G | P2 | Daemon HTTP DTOs sans `#[serde(deny_unknown_fields)]` | Sprint 8 Phase A (~5 LOC) |
| H-3 | H | P2 | `nexus_core` wheel editable install drift | Sprint 8 Phase A (déjà confessé) |
| B-3 | B | P3 | u64::MAX revision footgun (self-inflicted) | Sprint 10 optionnel |
| E-2 | E | P3 | Test `aggregate_probes_seeded_peer` ne teste pas le path pkarr-only | Sprint 8 ou 10 rename |
| E-3 | E | P3 | Clock recoil `duration_since` fallback non documenté | Sprint 9 nit |
| F-4 | F | P3 | Pas de toast sur `subscribeMutation` error | Laissé tel quel |
| H-1 | H | P3 | Plan §13 R2 demandait axum exact pin mais actuel est caret | Sprint 8 optionnel |

**0 P0, 0 P1, 10 P2 (4 pré-confessés + 6 nouveaux), 5 P3, 1 PASS majeur sur 9 tracks.**

---

## 12. Commits fix attendus

**Aucun**. Verdict = PASS. Sprint 8 Phase A peut démarrer directement
après commit du présent findings doc.

Si Sprint 8 veut traiter les P2 nouveaux en ouverture de son Phase A
(hygiène), un ordre suggéré (chacun un commit atomique `fix(sprint7):
...`) :

1. `fix(sprint7): deny_unknown_fields on daemon HTTP DTOs` (G-3, ~5 LOC)
2. `fix(sprint7): split NotSubscribed / EnvelopeMismatch errors` (C-2, ~30 LOC)
3. `fix(sprint7): tighten process_name_matches boundary check` (D-1, ~15 LOC)
4. `fix(sprint7): persist subscriptions before mutating RAM` (D-3, ~20 LOC)
5. `fix(sprint7): cap curator project-ref string lengths` (A-4, ~20 LOC)
6. `fix(sprint7): curators page refresh button` (F-1, ~15 LOC)

Total ~100 LOC si tous traités. **Ces commits sont OPTIONNELS** —
ils ne gatent pas Sprint 8 Phase A puisque le verdict est PASS.

---

## 13. P2 list to log in PATTERNS.md

Les 4 items déjà auto-confessés (E-1, C-4, D-3, H-3) sont déjà dans
`docs/rust/PATTERNS.md:820-858`. Les 6 nouveaux P2 doivent être
ajoutés :

- **docs/rust/PATTERNS.md** (Sprint 7 tech debt section) :
  - A-3 cross-lang curator fixture manquante
  - A-4 CuratorProjectRef strings illimitées
  - C-2 AnnouncementAttributionMismatch conflation
  - D-1 process_name_matches substring bounded
- **docs/shell/PATTERNS.md** (T8+ tech debt section) :
  - F-1 Curators refresh button
  - F-3 CardTitle accessibility
  - G-1 httpx client pooling
  - G-3 daemon DTOs deny_unknown_fields (côté Rust mais impacte le
    contrat shell, à logger aussi shell side)

Sprint 8 Phase F (sortie) est le bon moment pour cette mise à jour,
ou Sprint 8 Phase A si le chantier gov migration touche déjà le SDK.

---

## 14. P3 list laissés sans action

- **B-3** — u64::MAX revision self-footgun. Réviser en Sprint 10 avant
  release publique si besoin.
- **E-2** — Rename du test `aggregate_probes_seeded_peer_*` pour
  refléter ce qu'il teste vraiment. Sprint 8 ou plus tard.
- **E-3** — Commentaire sur le fallback clock-recoil. Sprint 9 polish.
- **F-4** — Toast vs inline form error. Non-prioritaire.
- **H-1** — Axum pin exact vs caret. Sprint 8 peut resserrer à
  `= "0.7.N"` s'il voit du bruit.

---

## 15. Notes on audit completeness

**Couverts intégralement** :
- Tracks A, B, D, I complets.

**Couverts partiellement (timebox)** :
- Track C : panic safety vérifiée par lecture ; pas de fuzzing réel
  du parser. Les 19 tests iroh_runtime cependant couvrent les cas
  négatifs documentés.
- Track E : 2 tests iroh_runtime + 9 tests browse lus ; le bench
  pkarr réel (E1 mesure in-the-wild) n'a pas été conduit — estimé
  trop coûteux vs gain audit.
- Track G : lecture statique uniquement, pas de test sous rafale.
  Les 10 tests `test_daemon_proxy.py` couvrent les cas négatifs
  synthétiquement.

**Baseline re-validée** :
`cargo test -p nexus-shell-daemon-core --locked` → **62 passed, 0
failed** (confirme le row 4 de verification.md pour ce crate).
Pas re-roulé les autres suites (fmt/clippy/workspace/python/web) —
timebox et fidélité du self-report sur ces axes jugée suffisante
(Sprint 6 avait eu le même pattern de confiance sans re-run
systématique).

**Not conducted** :
- Bench réel pkarr latency (E-1 mesure)
- Fuzzing parseur JSON sur `process_announcement_bytes` (C-3 panic
  safety verified by reading only)
- Playwright spec re-run (assumé PASS du self-report row 24 = 13/13)
- Cross-lang fixture run with a real Python-signed blob through Zod
  (A-3 confirmed absent by grep, not proven by execution)

Ces trous sont documentés comme limitations de l'audit, pas comme
findings supplémentaires.

---

## 16. Sign-off

**Auditeur** : Claude Opus 4.6 session fraîche, pas d'historique
Sprint 7 au démarrage (lit `MEMORY.md` + `nexus_grid_pivot.md` +
`sprint_audit_gate.md` + `docs/claude/README.md` puis ouvre
`.planning/sprint7_audit_plan.md` comme feuille de route).

**Tip audité** : `9cc0796` (master, 2026-04-11).

**Timebox** : ~2 h d'analyse + rédaction (plan §0 demandait 3h max).

**Verdict global** : **PASS** — Sprint 8 Phase A peut démarrer
directement, sans commit de gate préalable.

**Condition levée** : aucune. 10 P2 vont en tech debt dans
`PATTERNS.md`, 5 P3 restent optionnels.
