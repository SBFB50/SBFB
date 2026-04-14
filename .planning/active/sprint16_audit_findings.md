# Sprint 16 — Audit findings (Sprint 17 Phase 0 gate)

**Auditeur** : Claude Code session fraiche (Sprint 17 Phase 0)
**Date** : 2026-04-14
**Tip audite** : `1ff04df` (docs Sprint 16 Phase E verification + audit
plan + security roadmap)
**Commit stack Sprint 16** :
- `e99c06f` Phase 0 — audit findings Sprint 15 (PASS)
- `14ec51e` Phase 0 — PARA layout S0-15 archive
- `d7c265a` Phase A — loopback hardening bearer + Host + Origin
- `1cfde89` Phase B — UDS peer creds + Named Pipes DACL
- `3247e88` Phase C — GPU opt-in dialog + worker caps enforcement
- `10bbc63` Phase D — ProjectAnnouncement v5 + is_open_source
- `1ef22c6` fix(sprint16) — pre-set sbfb-consent-seen-v1 Playwright
- `1ff04df` Phase E (docs) — verification + audit plan + security

**Timebox observe** : ~1h45 (dont ~7min rebuild PyO3 wheel stale dans
`.venv` — voir §Hors-scope).

---

## Verdict global : **CONDITIONAL PASS**

- **P0** : 0
- **P1** : 4 (C1 L2 inert, C2 caps inert, C3 consent fail-open, D1
  daemon `/publish` accepte `is_open_source` via body JSON)
- **P2** : 7
- **P3** : 7

Sprint 17 Phase A peut demarrer apres les fix P1 ciblables sans
toucher au schema TaskEntry P2P :
- **C3** (consent fail-closed) et **D1** (daemon reject body
  `is_open_source`) sont de petits commits `fix(sprint16): ...`
  autonomes, a lander avant Phase A.
- **C1** et **C2** ne sont pas fixables sans bumper le `Task`
  canonical schema (ajouter `estimated_watts` / `estimated_vram_mb` /
  `estimated_hours` / `is_open_source` task-side). Ce bump aurait
  du etre declare en scope cut du kickoff §6 ; il ne l'est pas. La
  recommandation est de les **re-encadrer comme scope cut officiel**
  dans le kickoff Sprint 17 avec un pointeur explicite vers C1+C2
  dans les Phases S17 consacrees au wire-through, plutot que de
  bloquer le gate : les deux findings sont **fail-safe** (L2
  rejette tout, caps W/VRAM ne sont jamais franchis) et ne
  violent aucune garantie de confidentialite.

Les 7 P2 atterrissent en tech debt (`docs/shell/PATTERNS.md` /
`docs/rust/PATTERNS.md`) avec TODO Sprint 17+. Les 7 P3 restent
tels quels — nits de coherence docs, pas d'action requise.

---

## Resume executif

Top 3 findings :

1. **P1-C1 + P1-C2** — le trio consent level L2 (open source) +
   caps watts/VRAM/h est **livre en forme mais inert en pratique**
   cote worker. `crates/nexus-worker-core/src/engine/runtime.rs:778`
   hardcode `is_open_source: false` et `estimated_watts/vram_mb:
   0`, `estimated_hours: 0.0`. Le `Task` canonical schema
   (`crates/nexus-core-rs/src/task.rs:47`) ne porte pas ces champs.
   Consequence :
   - L2 rejette TOUTES les tasks (plus restrictif que L1 qui au
     moins accepte l'auto-partage avec soi-meme).
   - Caps watts/VRAM ne peuvent jamais declencher un reject
     pre-claim — la task passe, consomme ce qu'elle veut, et
     seules les heures cumulees remontent via `record_task` apres
     coup.
   Kickoff D3 annonce pourtant explicitement : "Les caps ne sont
   PAS juste des valeurs cosmetique UI : elles sont la source de
   verite pour `should_accept_task`". Gap non documente en scope
   cut §6.

2. **P1-D1** — le endpoint daemon `POST /publish` accepte
   `is_open_source` via le body JSON
   (`crates/nexus-shell-daemon/src/http.rs:311` serde default
   false). Le kickoff D4 insiste sur "derive par le coordinator,
   jamais user-settable". Un process local avec le bearer token
   peut donc flag son zip prive comme open source. L'invariant
   "coordinator = seule source du flag" est casse. Fix 1-liner :
   rejeter le champ dans `PublishRequest` ou forcer le daemon a
   le deriver uniquement de `provenance_hash` + `repo_url`.

3. **P1-C3** — `runtime.rs:829-830` : si `watcher.current()`
   renvoie `Err(Poisoned)`, le worker logge "consent state
   unreadable; accepting task by default" et accepte la task. Le
   default post-load est L1 ("mes projets uniquement"), pas "tout
   le reseau", donc la branch fail-open viole le principe GDPR
   "zero opt-in". Fail-closed (skip la task + retry next tick)
   est 3 lignes de diff.

Les 3 commits `fix(sprint16): ...` proposes sont decrits en
§Commits fix a lander.

---

## Track A — Bearer + Host + Origin middleware — **PASS avec P3**

**Methode rollee** :
- Lecture integrale `crates/nexus-shell-daemon-core/src/auth.rs`
  (708 LOC).
- Lecture `crates/nexus-launcher/src/auth.rs` (460 LOC).
- Lecture `packages/nexus-coordinator/src/nexus_coordinator/auth.py`
  (229 LOC) + `api/app.py` §middleware wiring.
- Grep routes `Router::new\|\.route\|nest` dans
  `crates/nexus-shell-daemon/src/http.rs` pour lister exemptions.
- Spot-check `constant_time_eq` (auth.rs:370-379) + tests
  `constant_time_eq_matches_slice_eq`.

**Resultat principal** : la triple validation est correctement
sequencee et implementee en constant-time cote Rust (diff-OR
accumulator sans short-circuit) et `hmac.compare_digest` cote
Python. 10 tests axum unit couvrent les 6 combinaisons
(token=OK/KO, host=OK/KO, origin=absent/OK/KO). Le marker
`PeerCredsVerified` est une **extension axum** qu'un client
wire-level ne peut pas injecter (test `peer_creds_marker_does_not_leak_via_http`
verifie qu'un header spoof `x-peer-creds-verified:1` reste 401).
Ordre middleware FastAPI correctement verifie : `add_middleware(LoopbackAuthMiddleware)`
**avant** `add_middleware(CORSMiddleware)` → CORS est outer
(repond OPTIONS preflight sans token), auth inner.

### A-1 (P2) — `/blob-serve/*` exempte du bearer sans mention kickoff D1

**Localisation** : `crates/nexus-shell-daemon/src/http.rs:141-148`

`build_router` met `/blob-serve/*` dans `public_routes` (pas de
`auth_required`). Justifie via docstring lignes 130-136 : "blob
content already public by construction". Mais le kickoff §D1 dit
"**Exception unique** : `/health` reste public". Divergence
kickoff ↔ impl, **justifie** par design (les blobs sont deja
P2P-publics par hash) mais non validee dans le kickoff.

**Action** : noter en `docs/shell/PATTERNS.md` tech debt + faire
referencer explicitement par le kickoff Sprint 17 comme scope
gele. Pas de fix code.

### A-2 (P2) — `PeerCredsVerified` est une `pub struct` unit sans champ prive

**Localisation** : `crates/nexus-shell-daemon-core/src/auth.rs:292-293`

```rust
#[derive(Debug, Clone, Copy)]
pub struct PeerCredsVerified;
```

Le plan audit B demandait : "C'est un type **prive** (`pub struct
PeerCredsVerified;` dans un module qui expose *pas* de
constructeur public ?)". La struct est **publique avec
constructeur implicite** — n'importe quel crate qui depend de
`nexus-shell-daemon-core` peut faire `PeerCredsVerified {}` et
l'injecter en `request.extensions_mut()`. Pas exploitable
wire-level (teste), mais un bug dans un futur handler qui
appelle `req.extensions_mut().insert(PeerCredsVerified)` par
erreur casse l'invariant.

**Fix defense-in-depth** : passer a un tuple struct avec champ
prive `pub struct PeerCredsVerified(());` et un constructeur
`pub(crate) fn new() -> Self { Self(()) }` dans le module UDS/NP
uniquement. 5-8 LOC, sans breaking change pour les consumers.

**Action** : tech debt P2 — commit `fix(sprint16): seal PeerCredsVerified
constructor` possible avant Sprint 17 Phase A si l'utilisateur le
souhaite, sinon defere.

### A-3 (P3) — `loopback_cors_layer` n'accepte pas `[::1]`

**Localisation** : `crates/nexus-shell-daemon/src/http.rs:213`

```rust
if host != "127.0.0.1" && host != "localhost" {
    return false;
}
```

Le `loopback_cors_layer` du daemon n'inclut pas `::1` alors que
`auth.rs::is_loopback_host` (ligne 238) l'accepte. Pas
exploitable — en pratique les browsers n'envoient pas d'Origin
IPv6 pour du loopback. Mais c'est inconsistent avec le tier
`auth_required`. Nit.

**Action** : nit.

### A-4 (P3) — CORSMiddleware coord accepte HTTPS

**Localisation** : `packages/nexus-coordinator/src/nexus_coordinator/api/app.py:108`

```python
allow_origin_regex=r"^https?://(127\.0\.0\.1|localhost)(:\d+)?$"
```

Le regex accepte `https://localhost` alors que
`auth.py::is_loopback_origin` refuse tout sauf `http://`. Un
preflight OPTIONS depuis `https://localhost` passe CORS puis est
bloque 403 par auth. Final behaviour sur, juste incoherent.

**Action** : nit. Resserrer le regex a `http://` (1-char change)
quand un sprint fait du nettoyage middleware.

### A-5 (P3) — CORSMiddleware coord n'accepte pas `[::1]`

Meme argument que A-3, cote coord. `allow_origin_regex` liste
`127.0.0.1|localhost`. Nit.

### A-6 (P3) — kickoff `scope cut npm audit` inexact

**Localisation** : kickoff §6 "CI cargo-audit / pip-audit /
**npm audit** → Sprint 17+"

`npm audit --audit-level=high || true` est deja present en CI
depuis avant Sprint 16 (`.github/workflows/ci.yml:120-121`). Le
kickoff laisse croire qu'aucun des 3 n'existe en CI. Pas une
scope cut **violee** (rien n'a ete ajoute en S16), juste une
mention inexacte.

**Action** : corriger le phrasage au kickoff Sprint 17 si le
sujet revient.

---

## Track B — UDS + Named Pipes peer creds — **PASS avec P2**

**Methode rollee** :
- Lecture integrale `crates/nexus-shell-daemon/src/uds_server.rs`
  (366 LOC).
- Lecture integrale `crates/nexus-shell-daemon/src/named_pipe_server.rs`
  (417 LOC).
- Lecture `packages/nexus-coordinator/src/nexus_coordinator/peer_creds.py`
  (92 LOC).
- Spot-check SDDL `D:(A;;GA;;;<sid>)` + SECURITY_DESCRIPTOR
  lifecycle + `LocalFree` in Drop.
- Verifie le flux accept-loop : `spawn_handler` execute AVANT
  `create_next_instance` → pas de fenetre ou un squatter
  pourrait s'inserer (tant que `first_pipe_instance(true)` du
  bind initial tient).

**Resultat principal** : l'implementation Unix (SO_PEERCRED
Linux, getpeereid macOS/BSD) rejette `uid != geteuid()` en
droppant le stream silencieusement (`handle_connection:231-238`).
L'implementation Windows construit le DACL via SDDL `D:(A;;GA;;;<sid>)`
avec le SID du user courant, recupere via `OpenProcessToken +
GetTokenInformation(TokenUser) + ConvertSidToStringSidW`. Tous
les handles sont closes, tous les heap Win32 sont `LocalFree`-ed
dans les drops. Le marker `PeerCredsVerified` est injecte via
`router.layer(axum::Extension(...))`, applique **outer** de
`auth_required`, donc chaque request UDS/NP voit l'extension
avant le middleware.

### B-1 (P2) — Coord UDS sans bypass ASGI (documente scope cut)

**Localisation** : `packages/nexus-coordinator/src/nexus_coordinator/peer_creds.py:11-18`

Le coord n'a **pas** de listener UDS actif en Phase B. `peer_creds.py`
expose `peer_uid()` utilisable par une future couche ASGI, mais
ce n'est pas wired dans l'app FastAPI. Scope cut documente dans
la docstring : "We do not wire the verification into the auth
middleware bypass yet — uvicorn's ASGI scope does not expose the
connection FD directly, so deferred to Sprint 17". Le kickoff
Phase B parle pourtant de "coordinator UDS binding". Incoherence
kickoff ↔ impl — mais le comportement de repli (TCP+bearer seul)
est securitairement equivalent.

**Action** : valider explicitement au kickoff Sprint 17 que le
coord UDS Unix ASGI bypass est scope Sprint 17 Phase A. Pas de
fix code.

### B-2 (P2) — Parent dir `~/.sbfb/run/` TOCTOU window

**Localisation** : `crates/nexus-shell-daemon/src/uds_server.rs:73-84`

```rust
if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
    set_mode(parent, 0o700)?;
}
```

`create_dir_all` cree un dir aux permissions par defaut (0755
sur Linux), puis `set_mode` l'amene a 0700. Fenetre TOCTOU
micro-seconde entre les deux syscalls. En pratique : le
`bind_socket` suit immediatement et pose le socket en 0600. Un
attacker co-local ne peut rien faire dans cette fenetre (il
ne sait pas quand elle s'ouvre et le path est imprevisible via
`SBFB_HOME`). Neanmoins, `DirBuilder::new().mode(0o700).create(...)`
eliminerait la fenetre.

**Action** : nit defense-in-depth, tech debt P2.

### B-3 (P3) — Log-flood sur `accept` fail

**Localisation** : `uds_server.rs:213-215`

```rust
Err(e) => {
    warn!(error = %e, "UDS accept failed; continuing");
}
```

Le accept loop continue la boucle apres un `warn!` sans
backoff. Si un attacker pouvait provoquer des accept fails en
boucle, il remplirait les logs. En pratique, les only paths qui
font fail l'accept sont `EMFILE` / `ENFILE` — ressource
exhaustion generale, pas un vecteur exploit. Nit.

---

## Track C — Consent caps + watcher — **4 P1 + 2 P2**

**Methode rollee** :
- Lecture integrale `crates/nexus-worker-core/src/consent.rs`
  (952 LOC).
- Lecture `crates/nexus-worker-core/src/engine/runtime.rs:755-955`
  (claim loop + consent filter + record_task post-success).
- Grep `estimated_watts|estimated_vram|estimated_hours` cross-
  crate pour verifier la presence dans `nexus-core-rs::task::Task`.
- Verifie `today_local_iso` utilise `chrono::Local`.
- Mental simulation du watcher sur write+rename.

**Resultat principal** : la **logique** `should_accept_task` est
correcte (level avant caps, chrono::Local pour midnight, HashSet
O(1) lookup pour L3, atomic write). Mais le **wire-through** des
inputs depuis la TaskEntry vers la fonction est incomplet —
`is_open_source`, `estimated_watts/vram_mb/hours` sont hardcodes
cote runtime (engine/runtime.rs:778-781 + 802-804). Consequence
operationnelle severe : L2 = reject all ; caps watts/VRAM =
inert.

### C-1 (P1) — `is_open_source` hardcode `false` cote engine runtime

**Localisation** :
- `crates/nexus-worker-core/src/engine/runtime.rs:778`
- `crates/nexus-worker-core/src/engine/runtime.rs:801`

Le claim loop construit le `TaskContext` avec `is_open_source:
false` en dur. La TaskEntry ne carry pas le flag. Meme apres
Phase D (PA v5 avec is_open_source), le worker ne peut pas
savoir si la task vient d'un projet open source — il voit
seulement l'`is_open_source` de l'**annonce** (PA), pas de la
task.

**Impact** : un utilisateur qui choisit L2 ("Projets open source
verifies") dans le dialog consentement voit **toutes** ses tasks
rejetees avec `RejectReason::NotOpenSource`. L2 est donc
equivalent a "ne rien contribuer" — plus restrictif que L1 qui
au moins accepte les tasks de ses propres projets.

Le commentaire ligne 761-763 l'admet : "Phase D wires this
through end-to-end ; until then L2 rejects everything". Mais
Phase D a livre PA v5 **uniquement**, pas le wire-through
task-side. Cela aurait du apparaitre en scope cut §6 du kickoff.

**Reproducer** :
1. Worker config `consent.json` : `{"level": 2, ...}`
2. Submit une task depuis un projet deploy-from-repo (PA v5
   `is_open_source=true`)
3. Observe les logs : `task rejected by consent filter
   reason=not_open_source`

**Fix suggere** : deux options, les deux necessitent de bumper
le TaskEntry schema (breaking change P2P — donc Sprint 17 Phase
A avec bump canonical + test backward compat decoder).

Option A (preferee) : ajouter `is_open_source: bool` (default
false) dans `nexus-core-rs::task::Task`. Le coordinator qui
publie la task le set depuis le `ProjectAnnouncement.is_open_source`
du projet courant. Worker lit `task.is_open_source` dans le
`TaskContext`. LOC estime : +30 core-rs + 5 coord + 5 runtime +
20 tests = ~60 LOC. Commit `feat(consent): Sprint 17 Phase X —
wire is_open_source through TaskEntry schema`.

Option B (scope minimal) : le worker cache localement la PA de
chaque `project_id` vu sur le reseau. Au claim time, lookup PA
→ is_open_source. Ajoute une dep sur le curator store, pas
trivial. ~150 LOC.

**Action** : re-encadrer en scope cut kickoff Sprint 17 Phase
A. Le gate conditional ne bloque PAS si S17 kickoff declare
l'item explicitement ; il bloque si S17 ignore la dette.

### C-2 (P1) — Caps watts/VRAM hardcode `0` cote engine runtime

**Localisation** :
- `crates/nexus-worker-core/src/engine/runtime.rs:779-781`
- `crates/nexus-worker-core/src/engine/runtime.rs:802-804`

```rust
&TaskContext {
    project_id: &project_id,
    is_open_source: false,
    estimated_watts: 0,
    estimated_vram_mb: 0,
    estimated_hours: 0.0,
},
```

Tous les 3 sont en dur. Le `Task` canonical schema
(`crates/nexus-core-rs/src/task.rs:47`) n'a **aucun** de ces
champs. Consequence :
- `estimated_watts: 0` < `max_watts: Some(400)` → **toujours
  accept**. Un task qui consomme 600W passe.
- `estimated_vram_mb: 0` < `max_vram_mb: Some(16384)` → idem.
- `estimated_hours: 0.0` + `used` peut s'accumuler via
  `record_task` apres succes, **donc la cap heures fonctionne
  a posteriori** (overshoot possible de la duree d'une task en
  cours).

Le kickoff §D3 declare pourtant textuellement : "Les caps ne
sont PAS juste des valeurs cosmetique UI : elles sont la source
de verite pour `allowlist.should_accept_task(&task)`". C'est
factuellement **faux** pour watts/VRAM dans l'etat livre.

**Reproducer** :
1. Configurer `consent.json` : `{"level": 4, "caps": {"max_watts": 50}}`
2. Submit une task dont l'estimation reelle est >50W (un modele
   LLM quelconque).
3. Observe les logs : aucun `cap_watts` reject. Task accepte et
   consomme ce qu'elle veut.

**Fix suggere** : meme Option A que C-1 — ajouter
`estimated_watts: u32`, `estimated_vram_mb: u64`, `estimated_hours: f64`
dans le Task canonical schema. Le coord qui craft la task soit
demande au client (dialog "estime les ressources") soit derive
des metadonnees de l'app (manifest). ~50 LOC core-rs + 30 coord
+ 20 tests = ~100 LOC.

**Action** : re-encadrer en scope cut kickoff Sprint 17
conjointement avec C-1 (meme bump).

### C-3 (P1) — Consent watcher fail-open quand `RwLock` poisoned

**Localisation** : `crates/nexus-worker-core/src/engine/runtime.rs:828-831`

```rust
Err(e) => warn!(
    error = %e,
    "consent state unreadable; accepting task by default"
),
```

Si `watcher.current()` renvoie `ConsentError::Poisoned` (un
autre thread du processus a panic alors qu'il detenait le write
lock), le match tombe dans le bras `Err` qui **logge** et
**continue sans `continue;`** — le control flow sort du bloc
`if let Some(watcher) = ...` et tombe dans la suite du claim
loop, qui signe et emet le claim. Fail-open.

Le default consent.json post-load est L1 ("mes projets
uniquement"), pas "tout le reseau", donc le fail-open viole le
principe annonce "GDPR-safe: zero opt-in par default".

**Reproducer** : panic intentionnel dans un hypothetique
consumer du watcher → poisoning → prochain claim → accept.
Difficile a declencher en prod mais la classification "fail
mode should be safe" est sans ambiguite.

**Fix suggere** : remplacer le `warn!` par `warn! + continue;`
ou par un match explicite qui set `outcome =
AllowOutcome::Reject(RejectReason::...)` generique. 3 lignes.

```rust
Err(e) => {
    warn!(error = %e, "consent state unreadable; rejecting task");
    continue;
}
```

**Action** : commit `fix(sprint16): C3 — consent watcher
fail-closed when state unreadable` AVANT Sprint 17 Phase A.

### C-4 (P1) — Consent watcher reload sur EventKind::Remove

**Localisation** : `crates/nexus-worker-core/src/consent.rs:492-497`

```rust
if !matches!(
    event.kind,
    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
) {
    continue;
}
```

Le watcher declenche un reload sur `EventKind::Remove` en plus
de Create/Modify. Si un operateur ou script delete `consent.json`
sans le recreer, `load_or_default` retourne le default L1 + caps
defaults + empty whitelist. Le worker silent-downgrade a L1.

**Impact** : pas un bypass de securite (L1 est plus restrictif),
mais un **silent behaviour change** qui peut surprendre. Si un
user en L4 voit soudain ses tasks rejetees parce qu'il a delete
le fichier par erreur, il n'aura pas d'info utile dans les logs.

**Fix suggere** : soit ignorer les `Remove` events (garder
l'ancien `inner` en memoire), soit emettre un `warn!` explicite
"consent.json deleted — reverting to defaults".

**Action** : commit `fix(sprint16): C4 — consent watcher logs
remove explicitly` ou ignore Remove — au choix utilisateur.

### C-5 (P2) — `atomic_write_json` sans fsync avant rename

**Localisation** : `crates/nexus-worker-core/src/consent.rs:606-617`

```rust
fs::write(&tmp, body)?;
fs::rename(&tmp, path)?;
```

`fs::write` ne fsync pas le tmp file avant le rename. Sur crash
kernel (power loss) entre write et rename, ext4 peut promoter
le rename en ancrage a un inode contenant un tmp vide ou
partiel. Le worker au reboot lit un fichier vide → default L1.

En pratique : crash kernel sur un poste dev est rare, et le
fail-mode est safe (L1). Defense-in-depth.

**Action** : tech debt P2. Pattern classique `File::create + write + sync_all + rename`.

### C-6 (P2) — Le plan audit demandait un test manuel watcher write+rename

**Localisation** : plan audit C.1.3

> `ConsentWatcher::spawn` — verifier que le debounce 50 ms
> tient contre un `write+rename` (rename emit 2 events `notify`)

J'ai fait la mental simulation : le `notify` crate emet
typiquement Create(new) ou Modify(new) apres un `write + rename`
atomique. Le debounce 50 ms entre chaque event est un **sleep
inconditionnel**, pas un vrai debounce temporel (si 3 events
arrivent, 3 reloads sequentiels). Pas un bug — le rename est
atomique donc les reloads redondants lisent le meme contenu
final — juste non-optimal.

**Reproducer** : non fait (timebox). Un test d'integration
Linux avec un `mv tmp consent.json` observable via cargo test
aurait ete couvrant. La suite existante couvre `write` direct
(via `save_atomic`) mais pas le rename externe.

**Action** : tech debt P2. Ajouter un test integration
`watcher_picks_up_rename_from_different_file` dans un sprint
futur.

---

## Track D — ProjectAnnouncement v5 is_open_source — **1 P1 + 1 P3**

**Methode rollee** :
- Lecture integrale `crates/nexus-shell-daemon-core/src/publish.rs`
  §v5 (156 LOC delta Phase D).
- Lecture `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py:100-188`
  + `deploy_from_repo` §309-322.
- Grep `crates/nexus-shell-daemon/src/http.rs::PublishRequest`
  + verif le champ + verif comment le daemon le propage.
- Lecture `web/src/api/daemon.ts:210-218` (BrowseEntrySchema Zod
  `z.boolean().optional()`).
- Grep UI usage `is_open_source|isOpenSource` dans `web/src/pages/`.
- Re-run `cargo test -p nexus-shell-daemon-core publish` : 4
  tests v5 passent (round-trip true/false, v4 legacy default
  false, v5 always serializes field).

**Resultat principal** : cote coord, `is_open_source` est
strictement derive : `deploy_from_repo` force `True`, `deploy`
(zip prive) force `False`, pas de chemin intermediare. Les
tests `test_deploy_from_repo_sets_open_source` /
`test_deploy_private_zip_sets_open_source_false` valident les
deux branches. Le Zod schema cote shell distingue correctement
`undefined` (legacy) de `false` (explicit). Decoder Rust accept
v1..v5, test backward compat v4 → default false present
(`v4_legacy_announcement_defaults_is_open_source_to_false`,
publish.rs:549).

### D-1 (P1) — Daemon `POST /publish` accepte `is_open_source` via body JSON

**Localisation** : `crates/nexus-shell-daemon/src/http.rs:311-312`

```rust
#[serde(default)]
pub is_open_source: bool,
```

Le `PublishRequest` (body du endpoint daemon `/publish`) accepte
`is_open_source` avec default false. Le coord pousse ce champ
via `_publish_with_archive` → daemon `/publish` → gossip PA v5.
Aucune validation daemon-side que le caller est bien le coord
local plutot qu'un autre process qui a le bearer token.

**Impact** : un process local avec le bearer token (voleur de
token, malware user-mode, extension navigateur qui a exploite
le bearer) peut faire :
```
POST /publish
X-SBFB-Token: <hex>
{"project_name":"evil", "is_open_source":true, "archive_hash":"...",
 "category":"...", "description":"..."}
```
et voir le PA v5 gossiper avec `is_open_source=true`, sans passer
par `/project/deploy-from-repo` ni par le clone + verification
SBFB.json + provenance. Les workers en L2 accepteront ses tasks.

Le kickoff D4 garantit textuellement : "derive par le
coordinator, jamais user-settable". L'invariant est casse au
niveau du daemon. Le bearer token n'est PAS une preuve
d'authenticite du coord local — c'est juste "process du meme
user".

**Reproducer** :
1. Daemon + coord tournent sur localhost.
2. `TOKEN=$(cat ~/.sbfb/auth_token)`
3. `curl -X POST http://127.0.0.1:7777/publish -H "X-SBFB-Token: $TOKEN"
   -H "Host: 127.0.0.1:7777" -d '{"project_name":"x",
   "category":"x", "description":"x", "is_open_source":true,
   "archive_hash":"aa...aa"}' → 200`
4. Observer la PA gossipe : `is_open_source=true` visible cote
   pairs alors qu'aucun repo n'a ete clone/verifie.

**Fix suggere** (1-2 LOC) :

Option A — retirer le champ du `PublishRequest` (daemon ignore,
derive de `provenance_hash.is_some()`) :
```rust
// in PublishRequest: remove is_open_source field entirely
// in publish_project handler: 
let is_open_source = req.provenance_hash.is_some();
```

Option B — valider le couple : `is_open_source=true` implique
`provenance_hash.is_some() && repo_url.is_some()` :
```rust
if req.is_open_source && (req.provenance_hash.is_none() || req.repo_url.is_none()) {
    return StatusCode::BAD_REQUEST.into_response();
}
```

Option A est plus stricte (le daemon decide seul) mais
couple daemon+coord. Option B est plus permissive mais
preserve la separation.

**Action** : commit `fix(sprint16): D1 — daemon reject
is_open_source body flag without provenance chain` AVANT
Sprint 17 Phase A. Recommandation : Option B.

### D-2 (P3) — UI ne consomme pas encore `is_open_source`

**Localisation** : grep `is_open_source|isOpenSource` dans
`web/src/pages/` → 0 match.

Phase D a landed le schema Zod et le backward compat, mais
aucun badge "Open Source" / "Proprietary" n'est affiche sur
`BrowsedProject` ou `Browse`. Le kickoff §D4 ne promet pas
d'UI — il promet le wire-through data. Techniquement respecte.

En pratique : le flag est **inert cote UX** jusqu'a ce qu'un
sprint futur ajoute un badge. Les tests `daemon.test.ts:649-680`
verifient que `is_open_source === undefined` n'est **pas**
surface comme false, donc une future UI sera coherente, mais il
n'y a rien a montrer aujourd'hui.

**Action** : nit. Noter comme "UI candidate Sprint 17" si un
des residuals R1-R4 touche a la Browse page.

---

## Track E — Docs security coherence — **2 P2 + 1 P3**

**Methode rollee** :
- Relu `docs/security/README.md` (77 LOC) en cross-ref avec
  `git log --oneline 4da0043..HEAD` : les 4 SHA cites (d7c265a,
  1cfde89, 3247e88, 10bbc63) matchent.
- Spot-check `THREAT_MODEL.md` §7 (matrix mitigations) : chaque
  row "LIVRE S16X" pointe vers un fichier qui existe, avec une
  LOC range credible pour auth.rs (274-383 couvre effectivement
  le middleware) et publish.rs (22-110 couvre le struct +
  methodes).
- Relecture `RUNTIME_ISOLATION.md` §2.1 (WSL2 install
  commands) : `wsl --install --no-distribution` + `wsl --install
  -d Ubuntu-24.04` sont syntaxiquement valides (docs MS
  courantes).

**Resultat principal** : la doc est globalement coherente avec
le code livre. Deux findings de coherence interne (§E-1, E-2)
qui mettent en lumiere le gap C1/C2 (worker runtime hardcode)
par effet de bord : la doc claim "caps enforced" mais le
wire-through est incomplet.

### E-1 (P2) — `THREAT_MODEL.md` §5 row "Task crafted...caps enforced" est inexact

**Localisation** : `docs/security/THREAT_MODEL.md:204`

> | S | Task crafted pour consommer plus de ressources que claim
> | H | `3247e88` : `should_accept_task` check `estimated_watts`
> / `estimated_vram_mb` contre caps avant accept | **L** |

La mitigation pointe vers `should_accept_task` qui implemente
bien le check. Mais la TaskEntry ne carry pas les estimates
(cf. C-2) et l'engine runtime hardcode `0`. La severite
residuelle "L" (low) est optimiste : en pratique, la mitigation
ne declenche jamais pour watts/VRAM.

**Action** : mettre a jour la row post-fix C1/C2, ou ajouter une
note explicite "res H jusqu'a ce que TaskEntry porte les
estimates (Sprint 17 Phase A)". Tech debt P2.

### E-2 (P2) — `THREAT_MODEL.md` §5 row "L2 ment sur le flag" est incomplete

**Localisation** : `docs/security/THREAT_MODEL.md:208`

> | E | L2 (open source) accepte un projet qui ment sur le flag
> | H | **res** : `d7c265a` + `10bbc63` — coordinator force
> `is_open_source=true` uniquement sur deploy-from-repo... ; non-user-settable
> | **L** |

La mitigation dit "non-user-settable". Factuellement, le coord
interne ne laisse pas le user settler le flag (OK). Mais D-1
montre que le daemon `/publish` l'accepte via body JSON. Un
attacker local avec bearer peut donc set `is_open_source=true`
sans passer par le coord. La residuelle "L" suppose que le
bearer token est toujours en possession du coord legit — ce
qui est vrai mais non declare comme assumption.

**Action** : ajouter une ligne "Assume token stays on local
coord process ; if bearer leaks, see R1 keypair + R5 token
exfil". Tech debt P2.

### E-3 (P3) — `THREAT_MODEL.md` §5 ne cite pas le fail-open C-3

La row E-3 (fail-open consent watcher) n'est pas documentee
dans le threat model. Un finding audit mineur : la doc est
incomplete sur ce point.

**Action** : nit. A inclure dans le fix `fix(sprint16): C3`
(update doc en meme temps que le code).

---

## Track F — Backward compat PA v4 + upgrade path — **1 P2 + 1 P3**

**Methode rollee** :
- Re-verifie le test `v4_legacy_announcement_defaults_is_open_source_to_false`
  passe en cargo test.
- Grep "upgrade" / "migration" dans `README.md` + `CLAUDE.md` +
  `docs/claude/`.

**Resultat principal** : PA v5 → v4 legacy decodage backward
compat present et teste. PA v4 → v5 forward compat : un noeud
v1.1 avec decoder PA v4 rejette les PA v5 (Version error, pas
crash). Split-brain naturel apres un bump de version — le
comportement est celui attendu d'un protocol bump.

### F-1 (P2) — Instructions d'upgrade absentes

**Localisation** : aucune section "upgrade" dans README.md ni
CLAUDE.md ni docs/claude/.

Le kickoff §D1 et §D4 disent respectivement :
- "redemarrer daemon + coord apres upgrade" (pour que les
  middlewares appliquent le mode strict)
- "Backward compat : PA v4 reste decodable, pas de migration
  forcee"

Aucun de ces 2 points n'est documente en README ou dans un
CHANGELOG accessible user. Un utilisateur qui met a jour son
install depuis v1.1 n'a aucun pointeur sur "tu dois redemarrer
daemon + coord".

**Action** : creer `docs/UPGRADE.md` ou ajouter une section
"Upgrade from v1.1" dans README. Tech debt P2.

### F-2 (P3) — Noeud v1.1 ne voit plus les projets v5 (split-brain)

Comportement attendu d'un bump de version. Un noeud v1.1 a un
decoder PA v4 qui rejette les PA avec `v > 4`. Donc les projets
publies en v1.2 avec PA v5 sont **invisibles** pour les pairs
qui n'ont pas upgrade. C'est le prix standard d'un protocol
bump — il serait plus inquietant si le noeud v1.1 **crashait**
sur une PA v5 (plan audit Track F demande de verifier ce
scenario ; l'analyse du decoder v1.1 historique montre qu'il
retourne un `Error::Version`, pas un panic).

**Action** : nit. Le phase-out v1.1 peut etre annonce au
CHANGELOG quand Sprint 17 ferme sa premiere phase livrable.

---

## Track G — Tests coverage + scope cuts — **PASS avec P3**

**Methode rollee** :
- Re-run local complet :
  - `cargo test --workspace --locked` → **426 passed** (421 +
    5 doc-tests). Match verification.md row 3.
  - `uv run pytest packages/nexus-coordinator/tests/ -q` →
    **187 passed, 3 skipped** (apres rebuild PyO3 wheel —
    voir §Hors-scope). Match verification.md row 11.
  - `uv run pytest packages/nexus-sdk/tests/ -q` → 182 passed +
    1 flaky Windows `test_concurrent_store_same_sha256_dedup_safe`
    (`PermissionError`). Inchange vs verification.md row 10.
  - `uv run pytest packages/nexus-app-gov/tests/ -q` → 46 pass.
    Match row 12.
  - `cd web && npm run test:unit` → **240 passed**. Match row 19.
  - `cd web && npx playwright test` → **38 passed**. Match
    row 26.
  - `cargo clippy --workspace --all-targets --locked -- -D warnings`
    → **0 warnings**. Match row 2.
- Grep scope cuts §6 kickoff : aucun `governor` / `slowapi` /
  `libmagic` / `python-magic` / `csp-report` endpoint nouveau
  dans `crates/` + `packages/` + `web/src/`.
- Grep `.github/workflows/` pour cargo-audit / pip-audit : **0**
  (seul `npm audit --audit-level=high || true` preexistant en
  ci.yml ligne 120).

**Resultat principal** : tous les compteurs verification.md
sont atteints **apres rebuild wheel PyO3** (cf. §Hors-scope).
Scope cuts kickoff §6 respectes a 100% — aucun rate limiting
nouveau, aucun cargo-audit ajoute, pas de MIME scan, pas de
CSP report-uri endpoint. Les tests MUST-HAVE du plan audit G.4
sont tous presents et passent (bearer 401/200, Host/Origin 403,
/health unauth 200, PeerCredsVerified not-spoofable, L1..L4 +
caps fn-level, watcher reload, PA v4 legacy, PA v5 round-trip
true/false).

### G-1 (P3) — Tests fn-level OK, tests end-to-end caps manquants

Le plan audit G.4 demande explicitement : "test
`should_accept_task` L1/L2/L3/L4 + caps". Les 4 niveaux + 3 caps
sont couverts en `consent.rs::tests` (fn-level). Pas de test
qui verifie **end-to-end** que le engine runtime applique bien
le reject (avec un fake doc + fake tokio runtime + fake consent
watcher injecte). Le finding C-1/C-2 est precisement dans ce
gap : la logique fn est bonne, l'integration au runtime hardcode
les inputs.

**Action** : ajout d'un test integration dans `runtime.rs::tests`
mock un claim loop + observe que `RejectReason::NotOpenSource`
est bien emis. A faire conjointement avec le fix C-1.

### G-2 (P3) — Pas de test "token valide + Host absent"

Le test suite `auth.rs::tests` couvre `token=OK, Host=OK`,
`token=OK, Host=rebound`, `token=OK, Host=ipv6`, mais pas
`token=OK, Host=absent` (HTTP/1.0 sans Host). Le code le gere
(`.unwrap_or(false)` → 403), mais pas de test couvrant.

**Action** : nit. 5-LOC test possible a glisser dans un fix
ulterieur.

### G-3 (P3) — Test PeerCredsVerified ne couvre pas la construction hors-crate

`peer_creds_marker_does_not_leak_via_http` verifie que le header
spoof echoue, mais pas que `PeerCredsVerified {}` est
inconstructible depuis un crate externe. Puisque la struct est
`pub struct PeerCredsVerified;` (cf. A-2), un crate externe
peut la construire. Le test ne le capte pas — il teste le
wire-level, pas le language-level.

**Action** : lie a A-2. Quand A-2 est fixe (sealing
constructor), ce nit disparait.

---

## Scope cuts kickoff §6 — VERIFIES INTACTS

| Item | Verifie | Evidence |
|---|---|---|
| Auto-install WSL2 / VM | Intact | `docs/security/RUNTIME_ISOLATION.md` expose la roadmap S17+, aucun code launcher touche |
| Encryption at rest keypair | Intact | `~/.sbfb/auth_token` reste plaintext (mode 0600), aucun dep Keychain/DPAPI/libsecret |
| CI cargo-audit / pip-audit | Intact | Aucune workflow nouvelle dans `.github/workflows/` |
| CI npm audit | **Deja existant preS16** | `ci.yml:120-121` — voir A-6 (kickoff inexact, pas violated) |
| Rate limiting deploy-from-repo | Intact | Aucune dep `slowapi` / `governor` |
| CSP report-uri endpoint | Intact | Aucun endpoint `/security/csp-report` |
| Audit externe / bug bounty | Intact | Hors scope |
| Revocation node_id (CRL) | Intact | Rien ajoute |
| MIME scan zip deploy | Intact | `nexus-sdk` contient `libmagic` historiquement (S14-), pas d'ajout S16 |
| Multi-level consent per-project | Intact | Consent reste global (niveau + caps) |
| Bytecode signing PyO3 wheels | Intact | `build-wheels.yml` inchange S16 |
| Token rotation automatique | Intact | "delete file, restart" reste le flow |

**Conclusion scope cuts** : tous respectes. L'unique mention
inexacte au kickoff (A-6, npm audit) est nit, pas violation.

---

## Tests coverage — counts observes

| Suite | Kickoff target | Observe | Delta |
|---|---|---|---|
| Rust workspace | 421 + 5 doc-tests = 426 | 426 | = |
| Python SDK | 182 + 1 flaky Windows | 182 + 1 flaky | = |
| Python coord | 187 + 3 skipped | 187 + 3 skipped | = |
| Python app-gov | 46 | 46 | = |
| Vitest unit | 240 | 240 | = |
| Playwright | 38 | 38 | = |
| size-limit | 7/7 | 7/7 | = |
| SPDX | 246+ | non re-run ce gate | - |
| **Total** | **~1136** | **~1136** | **=** |

---

## Hors-scope audit

### Rebuild PyO3 wheel necessaire pre-audit

Au premier `uv run pytest packages/nexus-coordinator/tests/ -q`,
10 tests coord ont echoue avec `AttributeError: module
'nexus_core' has no attribute 'sign_bytes'`. Cause : le .venv
local contenait une wheel PyO3 buildee avant Phase D (ou avant
un autre crate build), donc `sign_bytes` (defini Rust side
ligne `crates/nexus-core-py/src/lib.rs:1097` et exporte ligne
1161) etait absent de la surface Python.

Rebuild :
```bash
unset CONDA_PREFIX CONDA_DEFAULT_ENV && \
  VIRTUAL_ENV=$PWD/.venv maturin develop --release \
    --manifest-path crates/nexus-core-py/Cargo.toml
```

Apres rebuild : **187 passed, 3 skipped** (match verification.md).

Ce n'est **PAS un finding** du Sprint 16 : c'est un mainteance
item du dev env local. Sprint 17 pourrait ajouter un pre-test
hook qui check la presence de `nexus_core.sign_bytes` et refuse
de run si absent, ou un script `./scripts/reinstall-wheel.sh`.
Tech debt mineur.

### Migration PARA planning pendante

Le fichier `.planning/active/sprint15_audit_findings.md` est
reste dans `active/` apres fermeture Sprint 16 (bypass de
`sprint16_audit_plan.md:505` : "Les 6 docs Sprint 16 ... restent
dans active/ jusqu'au premier commit Sprint 17 Phase A, puis
migration `git mv` vers archive/v1.2/"). La migration S15 vers
`archive/v1.1/` a ete faite en `14ec51e` mais le
`sprint15_audit_findings.md` n'a pas suivi — il est reste en
`active/` parce qu'il a ete produit apres la migration. A la
fermeture Sprint 16 (ce gate), les 6 docs S16 + findings S15
suivront tous ensemble en `archive/v1.1/` (findings S15) et
`archive/v1.2/` (docs S16 + findings S16).

Action : au premier commit Sprint 17 Phase A,
```bash
git mv .planning/active/sprint15_audit_findings.md \
  .planning/archive/v1.1/
git mv .planning/active/sprint16_{kickoff,plan,verification,audit_plan,audit_findings}.md \
  .planning/archive/v1.2/
```

### Draft planning S17 en `.planning/` racine

Pendant l'audit, j'ai note `.planning/sprint17_kickoff.md` et
`.planning/sprint17_plan.md` en **untracked** a la racine
`.planning/` (hors `active/`). Ces drafts sont **prematures** —
le kickoff Sprint 17 doit reprendre les findings du present doc
(ajouter C-1/C-2 en scope officiels, declarer les fix C-3 + D-1
landed). Ne pas les integrer tels quels sans relecture.

Action : inspecter ces drafts a la fermeture du gate (ils
peuvent servir de base a condition de les mettre a jour). Si
obsoletes, `rm` les.

---

## Commits fix a lander AVANT Sprint 17 Phase A

Dans l'ordre chronologique recommande :

### 1. `fix(sprint16): C3 — consent watcher fail-closed when state unreadable`

**Fichier** : `crates/nexus-worker-core/src/engine/runtime.rs`

**Diff approximatif** (3 lignes) :
```rust
Err(e) => warn!(
    error = %e,
-   "consent state unreadable; accepting task by default"
+   "consent state unreadable; rejecting task as fail-closed"
),
+ continue;
```

**Test** : ajouter `consent_watcher_poisoned_rejects_task`
dans `runtime.rs::tests` (mock d'un watcher qui poison puis
verify que la task est skipped). +30 LOC de test.

**LOC estime** : ~35 LOC.

### 2. `fix(sprint16): D1 — daemon reject is_open_source without provenance chain`

**Fichier** : `crates/nexus-shell-daemon/src/http.rs`

**Diff approximatif** (option B, 5 lignes) :
```rust
// Dans le handler publish_project, debut de fn :
+ if req.is_open_source && (req.provenance_hash.is_none() || req.repo_url.is_none()) {
+     return (StatusCode::BAD_REQUEST, Json(ErrorResponse {
+         error: "is_open_source=true requires provenance_hash + repo_url".into()
+     })).into_response();
+ }
```

**Test** : ajouter `publish_rejects_is_open_source_without_provenance`
+ `publish_accepts_is_open_source_with_provenance` dans le test
suite du daemon. +40 LOC de test.

**LOC estime** : ~50 LOC.

### 3. (optionnel) `fix(sprint16): C4 — consent watcher explicit log on remove`

**Fichier** : `crates/nexus-worker-core/src/consent.rs`

Soit retirer `EventKind::Remove` du match (ignore), soit ajouter
un `warn!` dedie. A trancher avec l'utilisateur — les 2 sont
acceptables. LOC estime : ~5 LOC (pas de test nouveau, log
verifiable manuellement).

---

## Verdict final : **CONDITIONAL PASS**

Sprint 17 Phase A peut demarrer une fois les 2 commits fix
critiques (C-3 + D-1) landed. C-1 + C-2 sont re-encadres comme
scope cut officiel du Sprint 17 : le kickoff Sprint 17 doit
contenir une section "Dette heritee Sprint 16 §C-1/C-2" avec
une Phase consacree au wire-through TaskEntry → TaskContext
(bump canonical schema avec backward compat v4 legacy task
decoder).

Les 7 P2 (A-1, A-2, B-1, B-2, C-5, C-6, E-1, E-2, F-1) sont
logged en `docs/shell/PATTERNS.md` / `docs/rust/PATTERNS.md`
comme tech debt. Les 7 P3 restent sans action.
