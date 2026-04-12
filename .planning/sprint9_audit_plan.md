# Sprint 9 — Audit Plan (a jouer dans une session fraiche)

**Ecrit** : 2026-04-12, en fin de Sprint 9, par l'agent qui vient
de livrer les 5 commits feat `22c6721` -> `eb81c27`.

**Pourquoi ce document** : `.planning/sprint9_verification.md` est
une checklist fail-fast **self-reportee** par l'agent qui a ecrit
le code. 37/38 rows passent (row 29 coverage echoue) — mais c'est
le meme agent qui les a ecrites et qui confirme qu'elles passent.
Ce n'est pas une verification, c'est une auto-attestation. Le
pattern `sprint_audit_gate.md` rend l'audit structurellement
obligatoire avant d'ouvrir Sprint 10 Phase A.

**Principe** : le fail-fast dit « le code compile et les tests
passent ». L'audit independant dit « le code fait ce qu'il pretend
faire, la surface testee correspond a la surface executee en prod,
et les decisions sont justifiees a la relecture ». Sprint 9 est un
sprint **infra lourd** (4 nouvelles primitives SDK : storage, events,
migrations, files) avec un **consommateur reel** pour chacune dans
`nexus-app-gov`. L'audit doit verifier que les primitives sont
robustes, pas seulement que les tests passent — un test naive qui
ne teste pas les edge cases masque des bugs structurels.

---

## 0. Mode d'emploi pour la session fraiche

**Avant de commencer**, l'auditeur (agent ou humain) doit :

1. `git log --oneline master ^477bcc5` — lire les 7 commits Sprint 9
   (1 doc kickoff + 5 feat A..E + 1 doc verification + ce doc dans
   le meme commit)
2. Lire dans cet ordre :
   - `MEMORY.md` + `nexus_grid_pivot.md` + `sprint_audit_gate.md` +
     `feedback_approach.md` (memory cross-session)
   - `docs/claude/README.md` (workflow source of truth)
   - `.planning/sprint9_kickoff.md` (kickoff + §4 D1..D6 gelees)
   - `.planning/sprint9_plan.md` §4-9 (phases A..F detaillees) et
     §10 (fail-fast 38 rows cible)
   - `.planning/sprint9_verification.md` (self-report 37/38 rows +
     notes row 29 coverage deficit)
3. **Ne pas lire** `docs/shell/PATTERNS.md` nouveaux patterns ni les
   sections « Sprint 9 » de `docs/rust/PATTERNS.md` avant d'avoir
   forme un avis sur la policy — l'objectif est de **challenger**
   les choix, pas les ratifier. Lecture autorisee seulement APRES
   avoir ecrit son verdict track-par-track.
4. Tenir un journal `.planning/sprint9_audit_findings.md` au fur et
   a mesure. Format par finding :
   `{track, severity, what, evidence, fix}`
5. Severites : **P0** (casse prod / data loss / surface attaque),
   **P1** (bloque Sprint 10 Phase A), **P2** (tech debt explicite a
   logger dans `PATTERNS.md`), **P3** (nit, optionnel)

**Timebox suggere** : 3-4 h. Sprint 9 est un sprint infra lourd
(~9300 LOC ajoutees sur 78 fichiers, 4 primitives SDK + CAS + TabView
v2 + 3 Rust closures). L'auditeur doit **prioriser** : Track E (file
upload + CAS) est le plus critique parce qu'il touche au filesystem
et a la validation de contenu. Tracks B (storage), C (events), D
(migrations) introduisent des surfaces neuves dans le SDK. Track I
(Rust closures) verifie 3 items tech debt specifiques. Tracks A, F,
G, H, J sont des relectures rapides.

**Format du delivrable final** : une section par track ci-dessous
dans `.planning/sprint9_audit_findings.md`, chacune avec son verdict
PASS / CONCERN / FAIL + la liste des findings. Puis un **verdict
global** (PASS / CONDITIONAL PASS / FAIL) avec les conditions pour
lever un CONDITIONAL. Les P0 + P1 doivent etre corriges en commits
`fix(sprint9): ...` atterissant sur master **avant** le premier
commit Sprint 10 Phase A.

---

## 1. Track A — Code splitting sanity + scripts hygiene

**Question centrale** : le refacto `createBrowserRouter` + `lazy`
de Phase A a-t-il correctement separe les chunks sans casser
la navigation, et les scripts `setup.sh` / `verify.sh` sont-ils
robustes et complets ?

### A1 — Route lazy structure

**Methode** :
1. Ouvrir `web/src/App.tsx` et verifier que :
   - `createBrowserRouter` est utilise (pas `<BrowserRouter>`)
   - Chaque route enfant utilise `lazy: () => import(...)` et
     exporte `Component` (pas `default`)
   - `AppShell` est l'`element` du parent route (pas lazy-loade)
   - `<Suspense>` n'entoure PAS `<AppShell>` (sinon sidebar
     disparait pendant navigation)
2. Verifier que la palette (`CommandPalette`) est montee dans
   `AppShell` hors `<Outlet>`, pas dans une route enfant — sinon
   elle se demonte a chaque navigation

**Signal d'audit** :
- P1 si une route n'utilise pas `lazy` mais importe directement
  (fuite de chunk dans main)
- P2 si une page exporte `default` au lieu de `Component` (fonctionne
  mais ne suit pas la convention React Router v6.9+)

### A2 — `manualChunks` guards + dead chunk cleanup

**Methode** :
1. Ouvrir `web/vite.config.ts`, section `manualChunks`
2. Verifier que le guard `if (!id.includes('node_modules')) return;`
   est en tete (empeche les fichiers `src/` de tomber dans un
   chunk vendor)
3. Verifier qu'il n'y a **pas** de chunks morts `vendor-graph`,
   `vendor-charts`, `vendor-map` (heritage pre-pivot Sprint 5)
4. Lister les chunks definis : vendor-react, vendor-ui, vendor-query,
   tabview, palette, projectStore. Cross-check avec les entrees
   `.size-limit.json`

**Signal d'audit** :
- P1 si le guard `node_modules` est absent (un fichier `src/` dans
  un chunk vendor = chunk trop gros + confusion)
- P2 si un chunk mort survive

### A3 — `verify.sh` completude vs plan D5

**Methode** :
1. Lire `scripts/verify.sh` et comparer avec la liste D5 du kickoff
2. Le verification.md note que `test:coverage` est **absent** de
   verify.sh (step 12 = build, pas coverage). Statuer :
   - Est-ce un oubli (P1 si oui — la suite de reference ne couvre
     pas la couverture) ?
   - Ou un choix delibere non documente (P2 — l'absence devrait
     etre justifiee dans le script ou dans PATTERNS.md) ?
3. Verifier que `setup.sh` est idempotent (2 runs consecutifs, le
   2e skip le rebuild wheel)

**Signal d'audit** :
- P1 si verify.sh omet un check que le plan listait comme critique
- P2 si l'absence est explicable mais non documentee

### A4 — T8/T10/T11/T12 reellement fermes

**Methode** :
1. `grep -n 'T8\|T10\|T11\|T12' docs/shell/PATTERNS.md` — chaque
   item doit porter la mention « CLOSED Sprint 9 Phase A SHA 22c6721 »
2. Spot-check T11 (palette error swallow) : ouvrir
   `CommandPalette.tsx` et verifier que les erreurs async (fetch
   commands) sont visibles a l'utilisateur, pas un `catch () {}` vide
3. Spot-check T12 (commands ordering) : ouvrir `registry.py` et
   verifier que `commands()` retourne une liste triee par `name` asc

**Verdict track** : PASS / CONCERN / FAIL.

---

## 2. Track B — `AppContext.storage` primitive + gov consumer

**Question centrale** : la primitive storage est-elle robuste face
aux race conditions, aux shutdowns brutaux, et aux donnees invalides ?
Le consommateur gov (filter persist) l'utilise-t-il correctement ?

### B1 — Atomic rename safety

**Methode** :
1. Ouvrir `packages/nexus-sdk/src/nexus_sdk/storage.py`
2. Verifier que le write utilise `os.replace(tmpfile, target)` et
   **pas** `shutil.move` ni `os.rename` (qui ne sont pas atomiques
   cross-filesystem sur certaines plateformes)
3. Verifier que le tmpfile est cree dans le **meme repertoire** que
   le target (sinon `os.replace` echoue cross-device)
4. Tester : que se passe-t-il si le process crash entre le write du
   tmpfile et le rename ? Le fichier cible doit soit garder l'ancien
   contenu, soit etre absent (pas de corruption partielle)

**Signal d'audit** :
- **P0** si le write n'est pas atomique (corruption possible au crash)
- P2 si tmpfile n'est pas dans le meme dossier

### B2 — Write coalescing + flush-on-shutdown

**Methode** :
1. Verifier le pattern write coalescing :
   - `asyncio.call_later(0.5, flush)` ou equivalent
   - Si un 2e write arrive avant le flush, le timer est reset (pas
     d'accumulation de timers orphelins)
2. Verifier que `flush_on_shutdown` est appele dans le lifespan
   FastAPI du coordinator (test `test_lifespan_flushes_app_storage`
   dans `test_apps.py`)
3. Tester : que se passe-t-il si 100 writes arrivent en 100 ms ?
   Le coalescing doit produire UN seul write disk, pas 100

**Signal d'audit** :
- P1 si le timer n'est pas annule/reset au 2e write (timer leak)
- P1 si flush-on-shutdown n'est pas appele (perte de donnees au stop)

### B3 — `asyncio.Lock` per-app

**Methode** :
1. Verifier que chaque `AppStorage` instance porte son propre
   `asyncio.Lock`, **pas** un lock global partage entre apps
2. Tester : 2 apps concurrentes qui ecrivent dans leur storage
   respectif ne se bloquent pas mutuellement
3. Tester : 2 handlers concurrents de la **meme** app sont serialises
   par le lock (pas de race condition sur le dict en memoire)

**Signal d'audit** :
- P1 si le lock est global (bottleneck pour multi-app)
- P1 si le lock est absent (race condition possible)

### B4 — `TypedNamespace` validation stricte

**Methode** :
1. Ouvrir la methode `namespace("key", Schema)` dans `storage.py`
2. Verifier que `Schema.model_validate()` est appele au get (pas
   seulement au set) — sinon un fichier corrompu sur disk passe
   silencieusement
3. Tester : un fichier storage manuellement corrompu (bad JSON ou
   schema mismatch) doit lever `StorageSchemaError`, pas crasher
   le coordinator

**Signal d'audit** :
- P1 si la validation est skip au get
- P2 si l'erreur est swallowed au lieu d'etre logguee

### B5 — Gov consumer : filter persist roundtrip

**Methode** :
1. Lire le test Playwright `gov-politicians-filter-persist.spec.ts`
2. Verifier que le test :
   - Set un filtre (chambre, date, search)
   - Reload la page
   - Verifie que le filtre est conserve
3. Verifier que le namespace key est deterministe et ne fuite pas
   entre projets

**Verdict track** : PASS / CONCERN / FAIL.

---

## 3. Track C — `AppContext.events` primitive + SSE + gov consumer

**Question centrale** : le bus d'events in-process est-il robuste
face aux subscribers lents, aux disconnects, et aux pics de
publication ? Le SSE endpoint nettoie-t-il les subscribers morts ?

### C1 — Subscriber leak sur disconnect

**Methode** :
1. Ouvrir `packages/nexus-sdk/src/nexus_sdk/events.py`
2. Verifier que le context manager `subscribe(pattern)` fait un
   **cleanup explicite** dans son `__aexit__` :
   - Retire le subscriber de la liste interne
   - Ferme le `MemoryObjectSendStream`
3. Tester : un subscriber qui crash (exception dans le handler) doit
   quand meme unregister — verifier que le `finally:` est dans le
   context manager, pas dans le handler appelant

**Signal d'audit** :
- **P0** si un subscriber crash fuite un stream (memory leak cumulative)
- P1 si le cleanup est dans le handler mais pas dans le CM

### C2 — Overflow policy `drop_oldest`

**Methode** :
1. Verifier le mecanisme `drop_oldest` :
   - `send_nowait()` -> `WouldBlock` -> drain 1 item du receive ->
     retry `send_nowait()`
   - Le warning est throttle 1/min (pas un warning par message)
2. Tester avec un subscriber qui ne consomme jamais : le bus ne doit
   PAS bloquer, les messages doivent etre droppes avec warning

**Signal d'audit** :
- P1 si `drop_oldest` bloque au lieu de dropper
- P2 si le warning flood la console

### C3 — SSE endpoint lifecycle

**Methode** :
1. Ouvrir `packages/nexus-coordinator/src/nexus_coordinator/api/events.py`
2. Verifier que :
   - Le subscriber est cree dans un `async with` ou `try/finally`
   - Le heartbeat est envoye toutes les 30s (detect dead clients)
   - Si le client disconnect brutalement, le `finally:` unsubscribe
3. Lire le test `test_events_sse.py` — verifier qu'au moins un test
   couvre la deconnexion brutale (pas seulement le happy path)

**Signal d'audit** :
- P1 si le cleanup est absent sur disconnect
- P2 si le heartbeat est absent (subscribers morts accumulent)

### C4 — Gov consumer : party.refreshed flow

**Methode** :
1. Retracer le flow complet :
   - Worker `gov.refresh_party_cache` publish `party.refreshed`
   - SSE endpoint propage l'event au shell
   - React Query invalidation via `useEventSource` hook dans
     `AppTabPage.tsx`
2. Lire le Playwright `gov-party-refresh-event.spec.ts` — verifier
   que le test trigger le worker, attend l'event SSE, et verifie
   la mise a jour du grid sans reload
3. Verifier que le pattern `useEventSource` n'ouvre PAS un EventSource
   par tab (1 connexion SSE globale, pas N connexions)

**Signal d'audit** :
- P1 si un EventSource est ouvert par tab render (N connexions au lieu
  d'une)
- P2 si le test Playwright est flaky (race condition SSE timing)

**Verdict track** : PASS / CONCERN / FAIL.

---

## 4. Track D — Migration runner + SHA256 tamper + gov 001_documents.sql

**Question centrale** : le runner est-il robuste face aux fichiers
corrompus, aux migrations concurrentes, aux re-boots ? Le SHA256
tamper detection detecte-t-il reellement un changement de contenu ?

### D1 — SHA256 tamper detection

**Methode** :
1. Ouvrir `packages/nexus-sdk/src/nexus_sdk/migrations.py`
2. Retracer le flow :
   - Au boot : scan `migrations/` lexicographique
   - Pour chaque fichier : calcul `sha256(file_bytes)`, compare a
     `_nexus_migrations.sha256` stocke
   - Si divergence : `MigrationTamperedError` avec message explicite
3. Tester manuellement : modifier un fichier `.sql` deja applique,
   rouler le boot → doit lever l'erreur avec les deux hashes
4. Cross-check : le test `test_migrations.py` couvre-t-il ce scenario
   (pas seulement le happy path) ?

**Signal d'audit** :
- **P0** si la tamper detection est bypassable (ex: supprimer la
  row de `_nexus_migrations` suffit a re-appliquer une migration
  modifiee sans erreur)
- P1 si la detection leve bien l'erreur mais le coordinator continue
  a booter (doit etre bloquant)

### D2 — Transaction `BEGIN IMMEDIATE` + rollback

**Methode** :
1. Verifier que chaque migration est wrappee dans
   `BEGIN IMMEDIATE ... COMMIT`, avec `ROLLBACK` sur exception
2. Tester : une migration SQL avec une erreur syntaxique doit
   rollback sans corrompre la table `_nexus_migrations`
3. Verifier que le runner ne laisse PAS une transaction ouverte
   en cas d'exception (verrouillage DB post-crash)

**Signal d'audit** :
- P1 si le rollback est absent
- P1 si `BEGIN IMMEDIATE` est en fait `BEGIN` (pas de protection
  contre les concurrent writers, meme si notre singleton D1 Sprint 7
  les rend improbables)

### D3 — CLI `migrate --plan` vs `--apply`

**Methode** :
1. Lire le test `test_cli_migrate.py`
2. Verifier que `--plan` est un dry-run qui ne modifie rien
3. Verifier que `--apply` applique et ecrit dans `_nexus_migrations`
4. Verifier que `--apply` sur une base deja a jour est un no-op
   (idempotent)

**Signal d'audit** :
- P1 si `--plan` modifie la DB
- P2 si `--apply` n'est pas idempotent

### D4 — Gov consumer : 001_documents.sql

**Methode** :
1. Lire `packages/nexus-app-gov/src/nexus_app_gov/migrations/001_documents.sql`
2. Verifier la structure : `gov_documents(sha256 TEXT PRIMARY KEY,
   politician_id INT NULL, uploaded_at TEXT NOT NULL, title TEXT)`
3. Verifier que le test `test_gov_migrations.py` :
   - Applique la migration
   - Verifie le schema resultant
   - Verifie que la tamper detection fonctionne sur ce fichier
4. Verifier que le tab Documents lit bien cette table via
   `ctx.db_app.fetchall()`

**Verdict track** : PASS / CONCERN / FAIL.

---

## 5. Track E — File upload + CAS + magic bytes + v2 evolution

**C'est le track le plus critique du sprint.** Le CAS touche au
filesystem avec des operations atomiques, la validation magic bytes
est une surface de securite, et le bump v2 TabView est un contrat
d'evolution qui doit etre forward/backward compatible.

### E1 — CAS layout + sharding

**Methode** :
1. Ouvrir `packages/nexus-sdk/src/nexus_sdk/files.py`
2. Verifier le layout CAS :
   - `projects/<p>/apps/<a>/uploads/<sha256[:2]>/<sha256>` (fichier)
   - `projects/<p>/apps/<a>/uploads/<sha256[:2]>/<sha256>.json`
     (manifest)
3. Verifier que le sharding utilise `sha256[:2]` (256 buckets),
   pas `sha256[:1]` (16 buckets, trop peu pour de gros volumes)
4. Verifier que le repertoire `uploads/` est cree lazy au premier
   upload, pas au boot

**Signal d'audit** :
- P1 si le layout est different du plan D3 du kickoff
- P2 si le sharding est insuffisant

### E2 — Dedup pre-write

**Methode** :
1. Verifier le flow dedup :
   - Calcul SHA256 incremental pendant le chunked read
   - Check `sha_path.exists()` avant le rename
   - Si le fichier existe : skip l'ecriture, retourner le handle
     existant
2. Tester : upload du meme fichier 2 fois → le dedup evite la
   2e ecriture, le manifest est identique
3. Verifier que le manifest est quand meme mis a jour (uploaded_at,
   original_name peuvent differer entre les uploads)

**Signal d'audit** :
- P1 si le dedup n'est pas effectif (2 copies du meme fichier)
- P2 si le manifest n'est pas mis a jour au dedup

### E3 — Magic bytes validation

**Methode** :
1. Lire la whitelist hardcoded :
   - PNG `89 50 4e 47`
   - JPEG `ff d8 ff`
   - WEBP `52 49 46 46 ... 57 45 42 50`
   - PDF `25 50 44 46 2d`
   - SVG `3c 3f 78 6d 6c` ou `3c 73 76 67`
2. Verifier que la validation est faite **apres** l'ecriture
   (les premiers octets du fichier ecrit sont lus, pas le header
   multipart client-controlled)
3. Tester : upload d'un fichier avec extension `.png` mais contenu
   `Hello World` → doit etre rejete avec 415 Unsupported Media Type
4. Tester : upload d'un fichier SVG avec BOM UTF-8 `ef bb bf` avant
   `3c 3f` → est-il accepte ? Si non, c'est un faux negatif P2
5. Verifier que le fichier rejete est **supprime du disk** (pas de
   fichier orphelin dans le CAS)

**Signal d'audit** :
- **P0** si la validation est faite sur le header multipart seulement
  (client-controlled, bypassable)
- P1 si le fichier rejete n'est pas supprime (orphelin CAS)
- P2 si un SVG avec BOM est un faux negatif

### E4 — Manifest integrity + soft delete

**Methode** :
1. Verifier le format manifest JSON :
   `{sha256, size, content_type, original_name, uploaded_at,
   uploaded_by, app_name}`
2. Verifier que `delete(sha256)` supprime le manifest mais **pas**
   le fichier CAS (soft delete pour preservation dedup)
3. Tester : apres delete, `manifest(sha256)` retourne `None` mais
   `open(sha256)` retourne toujours le contenu (le fichier CAS
   survit)
4. Verifier que le manifest est ecrit de maniere atomique (meme
   pattern que storage: tmpfile + rename)

**Signal d'audit** :
- P1 si le delete supprime aussi le fichier CAS (casse la dedup)
- P2 si le manifest n'est pas atomique

### E5 — `@nexus_app_files(accept=[...])` opt-in

**Methode** :
1. Verifier que l'upload endpoint retourne **404** si l'app n'a pas
   le decorateur `@nexus_app_files`
2. Verifier que `accept=["image/*", "application/pdf"]` est verifie
   au moment du type detection, pas au moment du header multipart
3. Tester : une app sans decorateur → `POST /app/{name}/files/upload`
   → 404 « app does not accept file uploads »

**Signal d'audit** :
- P1 si l'upload route est accessible sans decorateur (toute app
  accepte des fichiers par defaut = surface non consentie)

### E6 — Chunked read + SHA256 incremental

**Methode** :
1. Verifier que le read utilise `while chunk := await file.read(8192)`
   et **pas** `await file.read()` (qui charge tout en memoire)
2. Verifier que le SHA256 est mis a jour incrementalement
   (`hashlib.sha256().update(chunk)` dans la boucle)
3. Verifier que le `max_part_size=50 * 1024 * 1024` est passe
   explicitement au endpoint FastAPI (defaut est 1 MB)

**Signal d'audit** :
- **P0** si le read charge tout en memoire (memory exhaustion DoS)
- P1 si max_part_size n'est pas overridde (uploads > 1 MB echouent
  silencieusement)

**Verdict track** : PASS / CONCERN / FAIL.

---

## 6. Track F — Schema v2 forward/backward compat

**Question centrale** : le bump `schema_version: 1 → 2` respecte-t-il
les contrats de compatibilite bidirectionnelle, et le renderer React
gere-t-il gracieusement les deux versions ?

### F1 — Forward compat (v1 valide sous v2)

**Methode** :
1. Ouvrir `packages/nexus-sdk/src/nexus_sdk/view.py`
2. Verifier que `AnyTabView.model_validate(v1_descriptor)` produit
   un `TabViewV1` valide (pas un rejet)
3. Lire le test `test_v1_descriptor_validates_under_v2` (ou
   equivalent) dans `test_view_v2.py`
4. Cross-check : le test lit-il la fixture
   `tabview_v1_schema.json` existante ou une fixture inline ?

**Signal d'audit** :
- P1 si un descriptor v1 est rejete par le schema v2
- P2 si le test utilise une fixture inline differente de la fixture
  shared cross-lang

### F2 — Backward compat (v1 recoit un v2)

**Methode** :
1. Verifier que `TabViewV1.model_validate(v2_descriptor)` leve une
   erreur structuree (pas un crash, pas un silent drop)
2. Le message d'erreur doit citer `schema_version` (pour guider le
   debuggage)
3. Lire le test `test_v2_file_upload_block_rejected_under_v1` ou
   equivalent
4. Cote React : verifier que le `TabViewRenderer` gere les deux
   versions — un descriptor v2 avec `file_upload_block` doit render
   le composant, un descriptor v1 ne doit jamais voir ce block

**Signal d'audit** :
- P1 si un v2 descriptor passe silencieusement sous v1 (le block
  `file_upload_block` est droppe sans erreur)
- P1 si le shell crash au lieu de retourner une erreur structuree

### F3 — Cross-lang fixture v2

**Methode** :
1. Verifier que `packages/nexus-sdk/tests/snapshots/tabview_v2_canonical.json`
   existe et est lu par :
   - Python `test_view_v2.py::test_cross_lang_fixture_v2_roundtrip_python_side`
   - Vitest `schema.test.ts` (ou equivalent) pour le cross-lang v2
2. Verifier que la fixture contient un `file_upload_block` (le seul
   block v2-only a date)
3. Verifier que `extra="forbid"` est preserve sur les deux versions
   (`TabViewV1` et `TabViewV2`) — c'est le seul mecanisme qui
   empeche un silent drop

**Signal d'audit** :
- P1 si la fixture cross-lang n'existe pas ou ne couvre pas le v2 block
- P1 si `extra="forbid"` est retire sur l'une des versions

### F4 — Zod mirror v2

**Methode** :
1. Ouvrir `web/src/components/app/tabview/schema.ts`
2. Verifier que la union Zod discrimine sur `schema_version`
3. Verifier que `FileUploadBlockSchema` est present et valide
   seulement sous v2
4. Cross-check : le `TabViewRenderer` handle-t-il le switch
   `schema_version` au render time ? Ou est-ce que le coordinator
   normalise ?

**Verdict track** : PASS / CONCERN / FAIL.

---

## 7. Track G — Scripts hygiene + H-3 closure

**Question centrale** : les scripts `setup.sh` et `verify.sh` sont-ils
robustes, portables (Git Bash Windows + Linux + macOS), et H-3 est-il
reellement ferme ?

### G1 — `setup.sh` portabilite

**Methode** :
1. Lire `scripts/setup.sh`
2. Verifier les POSIX-isms :
   - Shebang `#!/usr/bin/env bash`
   - Pas de `[[` (bash-ism OK si shebang bash)
   - `sha256sum` vs `shasum -a 256` (Git Bash a `sha256sum`,
     macOS a `shasum`)
3. Tester sur Git Bash Windows (`MSYS2`) : les paths Windows
   (`C:\Users\...`) sont-ils geres ?

**Signal d'audit** :
- P2 si le script crashe sur macOS (`sha256sum` absent)
- P3 nit si les messages d'erreur sont en anglais mais les docs
  sont en francais

### G2 — `verify.sh` coverage step absent

**Methode** :
1. Comparer verify.sh avec le plan D5 du kickoff
2. Le plan listait `npm run test:coverage` comme step 12 — le
   script a step 12 = build
3. Le verification.md row 29 echoue sur les seuils de couverture
   mais verify.sh passe car il ne les roule pas
4. Statuer : ajouter coverage comme step (P1 si la couverture est
   consideree comme gate), ou documenter l'absence (P2)

### G3 — H-3 hash mechanism

**Methode** :
1. Verifier que le hash dans `.venv/.nexus-core-hash` est base sur :
   - `Cargo.lock`
   - `crates/nexus-core-rs/src/**`
   - `crates/nexus-core-py/src/**`
2. Verifier que si on modifie un fichier Rust dans `nexus-core-py`,
   le hash change et le rebuild est declenche
3. Tester : modifier `crates/nexus-core-py/src/lib.rs` → rouler
   setup.sh → doit rebuild le wheel

**Verdict track** : PASS / CONCERN / FAIL.

---

## 8. Track H — Bundle headroom + chunks

**Question centrale** : les budgets size-limit sont-ils bien calibres
apres le code splitting massif de Phase A, et les chunks sont-ils
correctement separes ?

### H1 — Budget calibration

**Methode** :
1. `cd web && npm run size` — capturer les 7 budgets
2. Comparer chaque budget a sa valeur reelle et calculer le headroom
   en pourcentage
3. Verifier que le headroom est entre 15% et 50% — trop peu (< 15%)
   et le moindre ajout fail le budget, trop (> 50%) et le budget ne
   sert a rien
4. **Cas vendor-react** : Sprint 9 l'a mis a 290 KB (observe 274.69 KB,
   5.3% headroom) — est-ce trop serre ?
5. **Cas css** : 95.16 / 100 KB (4.8% headroom) — meme question

**Signal d'audit** :
- P2 si un budget est a < 10% headroom (fragile, le prochain sprint
  fait fail)
- P3 si un budget est a > 60% headroom (inutile)

### H2 — Chunk isolation

**Methode** :
1. `ANALYZE_MODE=true npm run build` (si `rollup-plugin-visualizer`
   est configure) → inspecter le treemap `dist/stats.html`
2. Verifier que :
   - `vendor-react` contient uniquement `react`, `react-dom`,
     `react-router-dom`
   - `vendor-ui` contient uniquement `@radix-ui/*`
   - `vendor-query` contient `@tanstack/react-query*` + `zustand`
   - `tabview` contient `src/components/app/tabview/**`
   - `palette` contient `src/components/command-palette/**`
3. Verifier qu'aucun fichier `src/` ne tombe dans un chunk vendor
   (le guard `node_modules` doit l'empecher)

### H3 — No chunk upload separate

**Methode** :
1. Le plan D6 prevoyait un chunk `upload <= 30 KB` separe
2. L'implementation bundle `FileUploadBlock` dans `TabViewRenderer`
   (13.02 KB total)
3. Statuer : est-ce acceptable (P3 nit) ou problematique si
   `TabViewRenderer` grandit au-dela de 20 KB (P2 si previsible) ?

**Verdict track** : PASS / CONCERN / FAIL.

---

## 9. Track I — Sprint 7 Rust closures (E-1 / C-4 / D-3)

**Question centrale** : les 3 items tech debt Rust Sprint 7 sont-ils
**reellement** codes et testes, ou sont-ils seulement marques CLOSED
dans PATTERNS.md sans implementation ?

### I1 — E-1 : probe_timeout env override

**Methode** :
1. `grep -rn 'NEXUS_PROBE_TIMEOUT\|probe_timeout' crates/nexus-shell-daemon-core/` —
   trouver le code qui lit la variable d'environnement
2. Verifier que la valeur par defaut est raisonnable (2s selon le
   kickoff Sprint 8)
3. Lire le test `probe_timeout_env_override_parses_valid_ms` dans
   `shell-daemon-core` — verifier qu'il set la env var, appelle la
   fonction, et verifie le resultat
4. Verifier que la env var est documentee (PATTERNS.md E-1 ou README)

**Signal d'audit** :
- P1 si le code lit la env var mais ne l'utilise pas (dead code)
- P2 si le test ne set pas reellement la env var (mock qui ne prouve
  rien)

### I2 — C-4 : gossip semaphore permits

**Methode** :
1. `grep -rn 'Semaphore\|semaphore' crates/nexus-shell-daemon-core/src/iroh_runtime.rs` —
   trouver le semaphore qui limite les announcements in-flight
2. Verifier le nombre de permits (doit etre documente dans le code
   ou dans PATTERNS.md)
3. Lire le test `gossip_semaphore_limits_inflight_announcements` —
   verifier qu'il depasse le nombre de permits et observe le
   blocage/backpressure
4. Verifier que le semaphore est cree dans le bon scope (per-curator
   vs global) et qu'il ne fuite pas (release sur drop)

**Signal d'audit** :
- P1 si le semaphore n'a pas de test de backpressure (juste un test
  d'acquisition)
- P2 si le nombre de permits est hardcode sans configuration

### I3 — D-3 : subscribe persist-first rollback

**Methode** :
1. `grep -rn 'persist_first\|persist.*rollback' crates/nexus-shell-daemon-core/src/iroh_runtime.rs` —
   trouver le code qui persiste sur disk AVANT de modifier l'etat
   en memoire
2. Verifier l'invariant : si le disk write echoue, l'etat en memoire
   ne doit PAS etre modifie (rollback)
3. Lire le test `subscribe_persist_first_rollback_on_disk_failure` —
   verifier qu'il injecte un disk failure et observe que l'etat
   memoire reste intact
4. Cross-check avec `unsubscribe` : le meme pattern persist-first
   doit s'appliquer (si le disk write d'un unsubscribe echoue, le
   subscriber reste actif en memoire)

**Signal d'audit** :
- P1 si le rollback n'est pas teste avec un vrai disk failure
  (pas un mock qui return false)
- P2 si unsubscribe ne suit pas le meme pattern

**Verdict track** : PASS / CONCERN / FAIL.

---

## 10. Track J — Doc consistency + PATTERNS updates

**Question centrale** : les fichiers PATTERNS.md sont-ils a jour
avec les fermetures Sprint 9, et les docs sont-ils coherents avec
le code ?

### J1 — `docs/rust/PATTERNS.md` Sprint 9 entries

**Methode** :
1. Verifier que H-3, E-1, C-4, D-3 sont marques CLOSED avec le
   SHA correct
2. Verifier qu'aucun item tech debt Sprint 9 n'a ete oublie
   (cross-check avec les commit bodies des 5 phases)

### J2 — `docs/shell/PATTERNS.md` Sprint 9 entries

**Methode** :
1. Verifier que T8, T10, T11, T12 sont marques CLOSED avec SHA
   `22c6721`
2. Verifier que les nouveaux patterns ajoutes (P12 code splitting,
   P13+ storage/events/files/migrations si documentes) sont
   coherents avec l'implementation

### J3 — Cross-reference kickoff → verification

**Methode** :
1. Verifier que chaque item de `sprint9_kickoff.md` §6 (scope cuts)
   est repris dans `sprint9_verification.md` §scope cuts
2. Verifier que les compteurs dans verification.md correspondent
   aux compteurs reels (relancer `cargo test --workspace --locked
   2>&1 | grep 'test result'` et `uv run pytest ... -q | tail -1`
   et comparer)

### J4 — Commit bodies structuredness

**Methode** :
1. Lire les 5 commit bodies (`git log --format="%B" 477bcc5..eb81c27`)
2. Verifier que chaque body contient :
   - Delta de tests cumule
   - Scope cuts honores
   - Fichiers touches avec rationale
   - Co-authored-by
3. Verifier qu'aucun body ne mentionne un fichier qui n'est pas
   dans le diff du commit correspondant

**Verdict track** : PASS / CONCERN / FAIL.

---

## 11. Verdict global attendu

Trois scenarios :

### PASS

0 P0, 0 P1. Sprint 10 Phase A demarre directement.

### CONDITIONAL PASS

1-3 P1 fixables. Sprint 10 Phase A est bloque tant que les commits
`fix(sprint9): ...` ne sont pas landed sur master. Les P1 les plus
probables vu le contenu de ce sprint :

- Coverage deficit (row 29) : si l'auditeur juge que le deficit
  `FileUploadBlock` est structurel (pas de tests Vitest unitaires,
  la couverture Playwright ne compense pas), c'est un P1 qui
  demande des tests supplementaires
- Subscriber leak events : si le cleanup context manager n'est pas
  dans le `__aexit__` mais dans le handler (P1 memory leak)
- Magic bytes faux negatif : si un SVG avec BOM UTF-8 est rejete
  a tort (P1 fonctionnel)

### FAIL

>= 1 P0 ou >= 3 P1. Re-conception partielle necessaire. Les
scenarios P0 les plus probables :

- `AppStorage` write non-atomique (corruption au crash)
- File upload read en memoire (DoS)
- Magic bytes validation sur header multipart seulement (bypassable)
- Subscriber leak sans cleanup (memory leak cumulative)

---

## 12. Out of scope pour l'audit

L'auditeur ne doit **PAS rebattre** les items suivants — ils sont
geles par les decisions Day 0 Sprint 9 (`sprint9_kickoff.md` §4)
ou par les decisions multi-sprint (`nexus_grid_pivot.md` §Decisions
actees) :

### Decisions Day 0 Sprint 9 (D1..D6 gelees)

- **D1** : `AppContext.storage` JSON file KV + typed namespaces
  (pas SQLite, pas iroh-docs, pas Redis)
- **D2** : `AppContext.events` anyio memory streams + fnmatch glob
  (pas asyncio.Queue maison, pas blinker, pas MQTT)
- **D3** : File upload CAS filesystem local + magic bytes whitelist
  hardcoded (pas python-magic, pas cloud storage)
- **D4** : Migration runner forward-only + SHA256 tamper (pas
  Alembic, pas downgrade, pas repair)
- **D5** : Scripts bash `setup.sh` + `verify.sh` (pas nox, pas
  cargo xtask)
- **D6** : createBrowserRouter + lazy + manualChunks (pas RSC,
  pas module federe, pas Rolldown advancedChunks)

### Decisions multi-sprint gelees

- Pivot P2P integral (pas de master central)
- Option G hybride Rust+Python
- iroh 0.97 pinne
- Visibilite 2 etats public/prive
- Zero moderation centrale, curator lists Ed25519+gossip+blobs
- Kudos per-project
- Worker binaire Rust single-file
- PyO3 via maturin
- License AGPL-3.0

### Scope cuts Sprint 9

L'auditeur ne doit PAS reprocher l'absence de :
- Cross-app events / cross-node events
- Storage cross-app
- Downgrade migration runner
- Cloud storage / S3
- python-magic
- Toast lib (sonner)
- Rolldown advancedChunks
- RSC / SSR / route loaders
- Les P3 Sprint 8 audit (A-FX-1, B-FX-1..3, C-FX-3..5, D-FX-2..3,
  E-FX-1..3, V-FX-2)
- F-2 CommandPalette loading state (P3 Sprint 7)

---

## 13. Livrable final attendu

`.planning/sprint9_audit_findings.md` avec :

1. **Auditeur** : id de session, timebox reellement observe
2. **Tip audite** : SHA master pris comme base
3. **Verdict global** : PASS / CONDITIONAL PASS / FAIL
4. **Une section par track** (A..J) avec verdict et findings
5. **Findings list sorted by severity** : table recapitulative
   P0 → P3
6. **Commits fix attendus** : si CONDITIONAL, liste des
   `fix(sprint9): ...` a lander avant Sprint 10 Phase A
7. **P2 a logger en tech debt** : items qui vont dans PATTERNS.md
8. **P3 laisses sans action** : nits explicitement ignores
9. **Notes on audit completeness** : tracks skippees et pourquoi

Le sprint est **CLOSED** quand :
- audit_findings.md est commite
- Tous les P0 et P1 sont fixes en commits `fix(sprint9): ...`
- Sprint 10 Phase A peut demarrer
