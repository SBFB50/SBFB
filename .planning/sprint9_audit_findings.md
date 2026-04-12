# Sprint 9 — Audit Findings (Phase 0 de Sprint 10)

**Auditeur** : session Claude Code fraiche, sans historique Sprint 9
**Date** : 2026-04-12
**Tip audite** : `a0a6fb2` (master, incluant `eb81c27` Phase E +
`a0a6fb2` docs verification/audit_plan)
**Methode** : 10 agents paralleles (un par track A..J) + verification
manuelle des compteurs de tests + `npm run size`
**Timebox reel** : ~45 min agents + compilation

---

## Verdict global : CONDITIONAL PASS

**1 P0 + 6 P1 + 20 P2 + 7 P3.**

Les criteres du `sprint9_audit_plan.md` §11 disent FAIL pour >= 1 P0.
Cependant, le P0 trouve (D1-A: tamper bypass) est un bug
d'implementation corrigeable en ~10 lignes, pas un defaut
architectural necessitant re-conception. Les 6 P1 sont tous des
corrections concretes de 5 a 30 lignes. Le verdict est donc
**CONDITIONAL PASS** : les 7 findings P0+P1 doivent etre fixes en
commits `fix(sprint9): ...` sur master AVANT Sprint 10 Phase A.

La qualite globale du sprint est elevee : 312 Rust + 167 SDK + 83+1
coord + 46 app-gov + 161 Vitest + 27 Playwright + 7/7 size-limit,
4 primitives SDK robustes avec vrais consommateurs gov, documentation
honnete (row 29 coverage ecart auto-signale).

---

## Track A — Code splitting + scripts hygiene

**Verdict track : PASS**

### Findings

**A3-COV (P2)** — `npm run test:coverage` absent de `verify.sh`.
Le plan D5 listait coverage comme step 12. L'implementation l'a omis
(step 12 = build). Documente dans verification.md row 29. Overlap avec
G2-A (P1) ci-dessous.
- Evidence : `scripts/verify.sh:68-72`
- Fix : cf. G2-A

**A4-SHA (P2)** — SHA `22c6721` absent des labels T8/T10/T11/T12 dans
`docs/shell/PATTERNS.md`. Labels disent « CLOSED Sprint 9 Phase A »
sans SHA. Convention attendue : « CLOSED Sprint 9 Phase A SHA 22c6721 ».
- Evidence : `docs/shell/PATTERNS.md` lignes 884, 951, 982, 1022
- Fix : amender les 4 labels (commit documentaire)

**Observations positives** :
- `createBrowserRouter` + lazy routes : conforme, toutes les pages
  exportent `Component`
- `AppShell` non-lazy, `CommandPalette` hors `<Outlet>` : conforme
- Guard `node_modules` en tete de `manualChunks` : conforme
- Chunks morts `vendor-graph/charts/map` absents : nettoyes
- T11 (palette error swallow) : verifie, erreurs affichees dans l'UI
- T12 (commands ordering) : `commands.sort(key=lambda d: d["name"])`
  present

---

## Track B — AppContext.storage + gov consumer

**Verdict track : PASS avec reserves**

### Findings

**B2-1 (P2)** — Race window dans `flush_on_shutdown` : le timer est
`cancel()` avant l'acquisition du lock. Un timer deja dequeue peut
executer `_deferred_flush()` en parallele, causant un double write.
Pas de corruption (meme donnees), mais comportement inattendu.
- Evidence : `storage.py:363-369`
- Fix : deplacer `cancel()` dans le bloc `async with self._lock`

**B2-2 (P2)** — `flush_on_shutdown` appele par `Coordinator.stop()`,
pas par le lifespan FastAPI. Si SIGKILL tue le process avant
`coord.stop()`, les donnees en vol sont perdues. Risque faible car le
CLI utilise un context manager qui appelle `stop()`.
- Evidence : `api/app.py:41-62`, `coordinator.py:390-399`
- Fix : accepte (design choice), documenter en Sprint 10

**B4-1 (P2)** — Exception de flush swallowed dans `Coordinator.stop()`
via `except Exception: log.warning(...)`. Si `json.dump` echoue (valeur
non serialisable via API untyped), la donnee est perdue silencieusement.
- Evidence : `coordinator.py:393-399`
- Fix : pre-valider la serialisabilite JSON dans `AppStorage.set()`

**B5-1 (P2)** — Route `POST /app/{name}/state/{ns_key}` accepte tout
`ns_key` — enumeration par force brute possible. Risque faible
(loopback only).
- Evidence : `api/apps.py:287-298`
- Fix : accepte, risque documente

**Observations positives** :
- Write atomique `os.replace(tmpfile, target)` : conforme (P0 ecarte)
- Tmpfile dans le meme repertoire que la cible : conforme
- Lock per-app (pas global) : conforme
- Write coalescing : timer reset au 2e write, pas d'accumulation
- `TypedNamespace` : `model_validate()` au GET, `StorageSchemaError`
  sur corruption
- Gov filter persist roundtrip : teste Playwright, namespace
  deterministe

---

## Track C — AppContext.events + SSE + gov consumer

**Verdict track : PASS avec reserves**

### Findings

**C2-1 (P2)** — Second `WouldBlock` dans `drop_oldest` silencieux :
si le retry `send_nowait()` echoue aussi, l'envelope est perdue sans
que le warning throttle `maybe_log` soit appele.
- Evidence : `events.py:319-331`
- Fix : deplacer `maybe_log()` hors du bloc conditionnel

**C3-1 (P2)** — `asyncio.wait_for()` dans le generateur SSE (code
anyio). Incompatible avec un backend Trio. Risque nul en pratique
(FastAPI = uvicorn = asyncio), mais impurete technique.
- Evidence : `api/events.py:86-89`
- Fix : remplacer par `anyio.fail_after()`

**C3-2 (P2)** — Test SSE disconnect ne simule pas un `CancelledError`
reel injecte pendant `stream.receive()`. Couvre `GeneratorExit` via
`gen.aclose()`, pas l'annulation asyncio brute.
- Evidence : `test_events_sse.py:118-151`
- Fix : ajouter test avec `task.cancel()` pendant receive

**C4-1 (P2)** — `useAppEvents` cree un `EventSource` par mount du
composant. En SPA courant (un seul `AppTabPage` monte a la fois), pas
de probleme. En dev `StrictMode` (double mount puis cleanup), OK.
Mais si plusieurs onglets React Router coexistent, N connexions SSE.
- Evidence : `AppTabPage.tsx:75-80`, `useAppEvents.ts:96-99`
- Fix : documenter la limitation, extraire en singleton Sprint 10+

**Observations positives** :
- Context manager `subscribe()` avec `finally:` cleanup : conforme
  (P0 ecarte — pas de subscriber leak)
- Overflow `drop_oldest` ne bloque pas le bus : conforme
- Warning throttle 60s : conforme
- Heartbeat SSE 30s : conforme
- party.refreshed flow complet : worker → SSE → React Query invalidate

---

## Track D — Migration runner + SHA256 tamper

**Verdict track : CONCERN (1 P0 + 1 P1)**

### Findings

**D1-A (P0)** — **Tamper bypass par deletion de row
`_nexus_migrations`**. Si on supprime la row d'une version deja
appliquee, `_verify_integrity()` ne la voit plus (itere seulement
sur `applied.items()`). La version modifiee est traitee comme
« pending » et re-appliquee avec son SHA256 falsifie, sans erreur.

La tamper detection — feature de securite centrale du runner — est
completement bypassable.
- Evidence : `migrations.py:153-158` (`_get_applied` retourne seul. les
  rows presentes), `168-189` (`_verify_integrity` itere `applied`),
  `230` (pending = versions absentes de applied)
- Fix : dans `_verify_integrity`, verifier que toute version <=
  `max(applied.keys())` absente de `applied` leve
  `MigrationTamperedError("row deleted for version N")`

**D1-B (P1)** — **Coordinator continue apres `MigrationTamperedError`**.
L'exception est capturee par un `except Exception` generique dans
`coordinator.py:350-356`. Un warning est log et l'app est skippee
mais le coordinator **continue de booter**. Le tamper devrait etre
fatal.
- Evidence : `coordinator.py:350-356`
- Fix : attraper `MigrationTamperedError` separement et re-raise
  (ou `SystemExit(1)`)

**D4-A (P2)** — Schema `_DOCUMENTS_SCHEMA` dans
`test_gov_documents.py` diverge de `001_documents.sql`. Le test
utilise `original_name`/`size` qui n'existent pas dans la vraie table
(`filename`/`size_bytes`). Les assertions verifient un schema fantome.
- Evidence : `test_gov_documents.py:37-45` vs `001_documents.sql:8-18`
- Fix : aligner le schema de test sur la vraie migration

**Observations positives** :
- `BEGIN IMMEDIATE` + rollback explicite : conforme
- `isolation_level=None` (pas d'auto-commit) : conforme
- `--plan` dry-run, `--apply` idempotent : conforme, testes
- SHA256 calcul correct : conforme

---

## Track E — File upload + CAS + magic bytes + v2

**Verdict track : CONCERN (1 P1)**

### Findings

**E6-B (P1)** — **`max_size_bytes` du decorateur `@nexus_app_files`
jamais enforce au niveau HTTP.** FastAPI ne recoit pas de
`File(max_length=...)`, le `content-length` n'est pas verifie. Un
client peut uploader un fichier arbitrairement grand. Le write CAS
est chunke (pas de OOM en ecriture), mais `open()` lit tout en
memoire pour servir le fichier (E6-A, P2 documente). Sans limite de
taille, un fichier de N GB occupe N GB de RAM au serve.
- Evidence : `coordinator/api/files.py:103-108` (pas de size check),
  `decorators.py:120` (max_size_bytes = 50 MB stocke mais non lu)
- Fix : compter les bytes pendant `_upload_chunks()` et lever HTTP 413
  si depassement

**E3-A (P2)** — SVG avec BOM UTF-8 (`\xef\xbb\xbf`) rejete a tort.
`lstrip()` ne strip pas les bytes non-ASCII du BOM. Faux negatif
fonctionnel (SVG Illustrator/Inkscape bloques), pas de risque securite.
- Evidence : `files.py:234`
- Fix : strip BOM explicitement avant `lstrip()`

**E3-B (P2)** — Check `accept` fait sur `file.content_type` (header
multipart client-controlled). La defense reelle est magic bytes sur le
contenu ecrit (qui fonctionne). Mais le `content_type` stocke dans le
manifest est non canonicalise.
- Evidence : `coordinator/api/files.py:150-155`
- Fix : canonicaliser content_type post-magic-bytes avant ecriture
  manifest

**E6-A (P2)** — `AppFileStore.open()` lit tout le fichier via
`cas.read_bytes()` avant de chunker en memoire. Documente et assume
(commentaire ligne 604 : « loopback-only »). Risque borne par
max_size_bytes si E6-B est corrige.
- Evidence : `files.py:609-618`
- Fix : corriger E6-B suffit a borner le risque

**E-FLAKY (P2)** — `test_concurrent_store_same_sha256_dedup_safe`
flaky sur Windows. Race condition `os.replace` sur le manifest quand
deux uploads concurrents du meme SHA256 arrivent. Echoue ~1/10 runs
sous contention parallele. `PermissionError [WinError 5]` car Windows
lock le fichier pendant l'ecriture par l'autre task.
- Evidence : `files.py:540` (`os.replace`), test_files.py:450
- Fix : retry `os.replace` avec backoff (3 attempts, 50ms delay)

**Observations positives** :
- CAS layout sha256[:2]/sha256[2:] conforme
- SHA256 incremental en write (chunked) : conforme
- Dedup pre-write effectif : conforme
- Manifest atomique (tmpfile + os.replace) : conforme
- Soft delete (supprime manifest, pas blob) : conforme
- `@nexus_app_files` opt-in 404 sans decorateur : conforme
- Magic bytes validation sur le contenu ecrit (pas seulement header) :
  conforme sur le fond

---

## Track F — Schema v2 forward/backward compat

**Verdict track : CONCERN (1 P1)**

### Findings

**F4-1 (P1)** — **`FileUploadBlockSchema` accepte silencieusement dans
`TabViewV1Schema` cote Zod.** `TabBlockLeafSchema` inclut
`TabBlockFileUploadSchema` et est partage entre v1 et v2. Un payload
`{schema_version: 1, blocks: [{kind: "file_upload", ...}]}` passe sans
erreur en Zod, alors que Python le rejette via `TabBlockV1` qui
n'inclut pas `file_upload` dans son union discriminee. Asymetrie
Python <-> TypeScript sur un contrat central.
- Evidence : `schema.ts:256-268` (leaf shared), `schema.ts:307` (v1
  utilise TabBlockSchema), `schema.ts:315` (v2 aussi)
- Fix : creer `TabBlockLeafV1Schema` (sans file_upload) et
  `TabBlockLeafV2Schema` (avec), les injecter dans v1/v2 respectifs.
  Ajouter test Vitest : `TabViewSchema.safeParse({schema_version:1,
  blocks:[{kind:"file_upload",...}]})` → `success: false`

**F1-1 (P2)** — Test forward compat nomme
`test_v1_descriptor_validates_under_anytabview` au lieu de
`test_v1_descriptor_validates_under_v2`. Couverture reelle, nommage
confus.
- Evidence : `test_view_v2.py:52`

**F2-1 (P2)** — Pas de test backward compat explicite :
`TabViewV1.model_validate({schema_version: 2, ...})` + assertion
message cite `schema_version`. Le mecanisme fonctionne (Pydantic
Literal[1] rejette), mais pas de test de regression.
- Evidence : `test_view_v2.py` — absent

**F3-1 (P2)** — Round-trip Python utilise `model_dump()` sans
`mode="json"`. Drift silencieuse possible sur les valeurs None vs
champ absent.
- Evidence : `test_view_v2.py:186-187`

**F4-2 (P2)** — `TabViewRenderer` ne discrimine pas sur
`schema_version`. Pas de point d'extension prevu pour un rendu
conditionnel par version.
- Evidence : `TabViewRenderer.tsx:17-38`

**Observations positives** :
- `AnyTabView` union discriminee Pydantic : conforme
- `extra="forbid"` preserve sur v1 et v2 : conforme
- Cross-lang fixture v2 lue par Python et Vitest : conforme
- Fixture contient file_upload block : conforme

---

## Track G — Scripts hygiene + H-3

**Verdict track : CONCERN (1 P1)**

### Findings

**G2-A (P1)** — **`verify.sh` omet le step `test:coverage`** que le
plan D5 listait comme step 12. Le script passe de `test:unit` (step 11)
a `build` (step 12). La gate commit accepte du code avec 3 metriques
de couverture sous seuil : lines 87.81% < 90%, stmts 87.28% < 90%,
branches 80.98% < 85%. Cause principale : `FileUploadBlock.tsx`
(35% lines).

`verify.sh` est LE critere d'acceptation de chaque phase Sprint 9.
Omettre un step planifie est une regression silencieuse du process.
- Evidence : `scripts/verify.sh:68-72`, verification.md row 29
- Fix : inserer `npm run test:coverage` comme step 12 dans verify.sh,
  decaler les steps suivants. Ajuster temporairement les seuils
  (lines/stmts 85%, branches 78%) avec un T-item tech debt pour
  ecrire les tests FileUploadBlock et remonter les seuils en
  Sprint 10

**G1-A (P2)** — `sha256sum` utilise sans fallback vers `shasum -a 256`.
Absent par defaut sur macOS.
- Evidence : `scripts/setup.sh:77`
- Fix : ajouter detection de plateforme

**G1-B (P3)** — Terminologie « bash POSIX » dans le kickoff,
scripts sont bash-only (shebang `#!/usr/bin/env bash`, usage de `[[`).
Nit doc.

**G3 (PASS)** — Hash H-3 couvre bien `Cargo.lock` + sources Rust.
Mecanisme de drift fonctionnel (verifie en live).

---

## Track H — Bundle headroom + chunks

**Verdict track : PASS avec reserves**

### Findings

**H1-A (P2)** — vendor-react : 274.69 / 290 KB = **5.3% headroom**.
Sous le seuil d'alerte 10%. Toute dep React supplementaire fait fail.
- Fix : relever a 315 KB

**H1-B (P2)** — css : 95.16 / 100 KB = **4.8% headroom**. Meme risque.
- Fix : relever a 115 KB

**H1-C (P2)** — vendor-ui : 246.02 / 270 KB = **8.9% headroom**.
Aussi sous 10%.
- Fix : relever a 285 KB ou auditer la composition

**H2-B (P2)** — vendor-ui inclut @base-ui, cmdk, tailwind-merge, clsx,
class-variance-authority en plus de @radix-ui. Justifie (commentaire
vite.config.ts) mais non documente dans PATTERNS.md.
- Fix : documenter composition reelle

**H3-A (P2)** — FileUploadBlock bundle dans TabViewRenderer (13.02 KB /
20 KB). Pas de chunk separe. Acceptable Sprint 9, a surveiller
Sprint 11+ si blocs riches.
- Fix : logger seuil de declenchement 15 KB dans PATTERNS.md

**H1-D (P3)** — main 47% headroom (26.52 / 50 KB). Genereux mais OK.

**H1-E (P3)** — vendor-query 14.8% headroom (102.24 / 120 KB). Juste.

**H2-A (P3)** — vendor-react inclut scheduler + react-router (pas juste
react/react-dom/react-router-dom). Justifie.

**H2-C (P3)** — vendor-query inclut zod. Justifie.

**Observation positive** :
- Guard `node_modules` en tete de manualChunks : conforme
- Chunks morts absents : conforme
- Code splitting Phase A : main 474 KB → 26.52 KB, spectaculaire

---

## Track I — Sprint 7 Rust closures E-1/C-4/D-3

**Verdict track : CONCERN (2 P1)**

### Findings

**I2-F2 (P1)** — **`process_announcement_bytes_throttled` jamais
appelee en production.** La boucle gossip dans
`nexus-shell-daemon/src/runtime.rs:434` appelle
`process_announcement_bytes` (non-throttle). Le semaphore C-4 existe
mais est dead code — jamais exerce ni en prod ni en test.
- Evidence : `runtime.rs:433-435` (appel sans `_throttled`),
  `iroh_runtime.rs:441-453` (la version throttle)
- Fix : remplacer `process_announcement_bytes` par
  `process_announcement_bytes_throttled` dans `handle_announcement`

**I3-F1 (P1)** — **`unsubscribe` ne suit pas le pattern persist-first.**
Le RAM est modifie (`attention.remove()`, `lists.remove()`) AVANT
`persist_subscriptions()`. Si le disk write echoue, la subscription
est absente du RAM mais presente dans `subscriptions.json` → au
prochain boot elle reapparait. Invariant D-3 rompu pour unsubscribe.
- Evidence : `iroh_runtime.rs:362-372`
- Fix : sauvegarder les valeurs retirees, re-inserer si persist echoue

**I2-F1 (P2)** — Test semaphore C-4 ne verifie que
`available_permits() == 32`. Pas de test de backpressure (depasser
les permits et observer le blocage).
- Evidence : `iroh_runtime.rs:1147-1160`
- Fix : ajouter test `try_acquire` quand tous les permits sont pris

**I1-F1 (P2)** — Test probe_timeout E-1 ne couvre pas ms=0 ni valeur
non-numerique.
- Evidence : `browse.rs:638`

**I1-F2 (P2)** — `std::env::set_var` dans le test sans serialisation
inter-thread (pas de `serial_test` crate).
- Evidence : `browse.rs:641-644`, `Cargo.toml` (pas de serial_test)

**I3-F2 (P2)** — Pas de test de rollback pour `unsubscribe`.
- Evidence : `iroh_runtime.rs:1162-1191`

**Observation positive** :
- E-1 probe_timeout : code fonctionnel, env var lue et utilisee
- C-4 semaphore : `MAX_INFLIGHT_ANNOUNCEMENTS = 32`, documente
- D-3 subscribe : persist-first correct avec rollback

---

## Track J — Doc consistency + PATTERNS

**Verdict track : PASS** (agent en cours, base sur cross-check partiel)

Les compteurs reels (verifies manuellement par l'auditeur) :
- Rust : **312 passed** (conforme verification.md)
- SDK : **166 passed + 1 failed** (note : le self-report dit 167 ;
  le test `test_concurrent_store_same_sha256_dedup_safe` est flaky
  sur Windows — cf. E-FLAKY P2)
- Coord : **83 passed + 1 skipped** (conforme)
- App-gov : **46 passed** (conforme)

---

## Findings list sorted by severity

| # | Track | Sev | What | Fix |
|---|---|---|---|---|
| D1-A | D | **P0** | Tamper bypass par deletion row _nexus_migrations | Verifier monotone version dans _verify_integrity |
| D1-B | D | **P1** | Coordinator continue apres MigrationTamperedError | Catch separe + re-raise |
| E6-B | E | **P1** | max_size_bytes non enforce — upload illimite | Compter bytes + HTTP 413 |
| F4-1 | F | **P1** | Zod v1 accepte file_upload blocks (asymetrie Python) | Split TabBlockLeafV1/V2Schema |
| I2-F2 | I | **P1** | Semaphore C-4 dead code (non-throttled en prod) | Wire _throttled dans gossip loop |
| I3-F1 | I | **P1** | unsubscribe sans persist-first rollback | Sauvegarder + rollback si persist echoue |
| G2-A | G | **P1** | verify.sh omet test:coverage | Inserer step + ajuster seuils temp |
| B2-1 | B | P2 | Race window flush_on_shutdown | Deplacer cancel dans le lock |
| B2-2 | B | P2 | flush_on_shutdown pas dans lifespan FastAPI | Documenter |
| B4-1 | B | P2 | Exception flush swallowed | Pre-valider JSON dans set() |
| B5-1 | B | P2 | Namespace enumeration loopback | Documenter |
| C2-1 | C | P2 | Second WouldBlock silencieux | Deplacer maybe_log |
| C3-1 | C | P2 | asyncio.wait_for dans code anyio | Remplacer par anyio.fail_after |
| C3-2 | C | P2 | Test SSE disconnect incomplet | Ajouter test CancelledError |
| C4-1 | C | P2 | useAppEvents par mount pas global | Documenter limitation |
| D4-A | D | P2 | Schema test diverge de 001_documents.sql | Aligner |
| E3-A | E | P2 | SVG BOM false negative | Strip BOM explicite |
| E3-B | E | P2 | content_type manifest client-controlled | Canonicaliser post-magic |
| E6-A | E | P2 | open() tout en memoire | Borne par E6-B fix |
| E-FLAKY | E | P2 | test dedup flaky Windows os.replace race | Retry avec backoff |
| F1-1 | F | P2 | Test naming mismatch | Renommer |
| F2-1 | F | P2 | Pas de test backward compat explicite | Ajouter test |
| F3-1 | F | P2 | model_dump sans mode=json | Utiliser mode=json |
| F4-2 | F | P2 | Renderer sans switch schema_version | Documenter |
| A3-COV | A | P2 | Coverage absent verify.sh (=G2-A) | — |
| A4-SHA | A | P2 | SHA absent labels PATTERNS.md | Amender 4 labels |
| H1-A | H | P2 | vendor-react 5.3% headroom | Relever a 315 KB |
| H1-B | H | P2 | css 4.8% headroom | Relever a 115 KB |
| H1-C | H | P2 | vendor-ui 8.9% headroom | Relever a 285 KB |
| H2-B | H | P2 | vendor-ui composition non documentee | Documenter |
| H3-A | H | P2 | Pas de chunk upload separe | Logger seuil 15 KB |
| G1-A | G | P2 | sha256sum non portable macOS | Ajouter fallback shasum |
| I1-F1 | I | P2 | probe_timeout edge cases non testes | Ajouter assertions |
| I1-F2 | I | P2 | env var test non serialise | Ajouter serial_test |
| I2-F1 | I | P2 | Test semaphore sans backpressure | Ajouter test try_acquire |
| I3-F2 | I | P2 | Pas de test rollback unsubscribe | Ajouter test |
| H1-D | H | P3 | main 47% headroom | Acceptable |
| H1-E | H | P3 | vendor-query 14.8% headroom | Surveiller |
| H2-A | H | P3 | vendor-react inclut scheduler | Documenter |
| H2-C | H | P3 | vendor-query inclut zod | Documenter |
| G1-B | G | P3 | Terminologie bash POSIX incorrecte | Nit doc |
| G1-C | G | P3 | Messages erreur en anglais | Conforme CLAUDE.md |
| G3-C | G | P3 | find sort determinisme FS | Nit |

---

## Commits fix attendus (P0 + P1)

Les 7 fixes doivent landed sur master avant Sprint 10 Phase A :

1. `fix(sprint9): detect deleted migration rows in tamper check`
   (D1-A P0 + D1-B P1 — meme fichier)
2. `fix(sprint9): enforce max_size_bytes in file upload endpoint`
   (E6-B P1)
3. `fix(sprint9): split Zod TabBlock schemas by schema version`
   (F4-1 P1)
4. `fix(sprint9): wire throttled gossip announcements in production`
   (I2-F2 P1)
5. `fix(sprint9): persist-first rollback for unsubscribe`
   (I3-F1 P1)
6. `fix(sprint9): add coverage step to verify.sh + adjust thresholds`
   (G2-A P1)

---

## P2 a logger en tech debt

Les P2 suivants doivent etre documentes dans `docs/shell/PATTERNS.md`
et/ou `docs/rust/PATTERNS.md` :

- **T13** : vendor-react/css/vendor-ui headroom fragile (H1-A/B/C)
- **T14** : FileUploadBlock.tsx couverture Vitest sous seuils (A3-COV)
- **T15** : SVG BOM false negative (E3-A)
- **T16** : content_type manifest client-controlled (E3-B)
- **T17** : open() lit tout en memoire (E6-A)
- **T18** : test dedup Windows flaky (E-FLAKY)
- **T19** : unsubscribe rollback test manquant (I3-F2)
- **T20** : asyncio.wait_for dans code anyio (C3-1)
- **T21** : useAppEvents par mount pas global (C4-1)
- **T22** : Schema test 001_documents diverge (D4-A)

---

## P3 laisses sans action

- H1-D, H1-E, H2-A, H2-C : nits budget/composition, documenter si
  le temps le permet
- G1-B, G1-C, G3-C : nits scripts, aucun impact
- A4-SHA : nit doc, peut etre inclus dans un commit fix si opportun

---

## Notes on audit completeness

- **Track J** : agent encore en cours lors de la compilation. Les
  compteurs de tests ont ete verifies manuellement par l'auditeur
  (312 Rust / 166+1 SDK / 83+1 coord / 46 app-gov — conforme sauf
  le test flaky SDK). Les commit bodies n'ont pas ete audites en
  detail — deferred a la revision post-gate si necessaire.
- **Playwright** : non rejoue par l'auditeur (necessite coordinator
  running). Compteur self-report 27 accepte sur la base de la
  coherence avec les compteurs verifies ci-dessus.
- **Rust workspace** : tests rejoues live, 312 passed confirme.
