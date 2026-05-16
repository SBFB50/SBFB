# Sprint 63 — Plan d'execution (verification tiers + UX)

**Ecrit** : 2026-05-15.
**Kickoff** : `sprint63_kickoff.md` (meme date).
**Tip d'entree** : `1405c0c`.
**Phases** : 4 (A-D).
**Estimation delta tests** : +8 Rust, +6 Vitest (conservateur).

---

## §1 Phase A — MANDATORY 3/3 carries (IMAGE-DEP + PLAYWRIGHT-REFACTOR)

### §1.1 P2-IMAGE-DEP : image → png dans nexus-launcher

**Objectif** : eliminer le crate `image 0.25` (~15 transitives) au
profit du crate `png` (~3 transitives) pour le decodage du PNG
tray icon embarque.

**Etapes** :

1. Dans `crates/nexus-launcher/Cargo.toml` :
   - Remplacer `image = { version = "0.25", default-features = false, features = ["png"] }`
     par `png = "0.17"`
   - Supprimer la dep `image`

2. Dans `crates/nexus-launcher/src/tray.rs` :
   - Remplacer `image::load_from_memory(png_bytes)` par l'API `png::Decoder` :
     ```rust
     let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
     let mut reader = decoder.read_info()?;
     let mut buf = vec![0u8; reader.output_buffer_size()];
     let info = reader.next_frame(&mut buf)?;
     let (w, h) = (info.width, info.height);
     buf.truncate(info.buffer_size());
     ```
   - Le `Icon::from_rgba(buf, w, h)` reste identique
   - Adapter le test `embedded_icon_decodes_to_valid_rgba` de la
     meme facon

**Critere d'acceptation** :
- `cargo build -p nexus-launcher` compile
- `cargo nextest run -p nexus-launcher` passe (test icon decode)
- `cargo tree -p nexus-launcher -d` confirme absence de `image`

### §1.2 P2-PLAYWRIGHT-REFACTOR : global-setup.ts → daemon Rust

**Objectif** : reecrire `web/tests/global-setup.ts` pour spawner le
daemon Rust au lieu du coordinateur Python (supprime S50-S51).

**Etapes** :

1. Localiser le binaire daemon Rust :
   - Chemin : `target/release/nexus-shell-daemon` (ou `target/debug/`)
   - Utiliser une variable d'env `SBFB_DAEMON_BIN` avec fallback vers
     le chemin debug standard

2. Reecrire `initProject()` dans `global-setup.ts` :
   - Remplacer `spawn("uv", ["run", "--package", "nexus-coordinator", ...])` par
     `spawn(daemonBin, ["init", testName])` avec `NEXUS_GRID_ROOT`
   - Supprimer les refs Python (`PYTHONIOENCODING`, `--package`, etc.)

3. Reecrire le spawn start :
   - Remplacer `spawn("uv", ["run", "--package", "nexus-coordinator", "nexus-coordinator", "start", ...])` par
     `spawn(daemonBin, ["start", testName, "--port", "18765"])`
   - Conserver `NEXUS_GRID_ROOT` et `SBFB_AUTH_TOKEN`

4. `global-teardown.ts` : verifier que le kill fonctionne avec le
   PID du daemon Rust (pattern identique, pas de changement necessaire
   si le PID est ecrit dans `.playwright-state.json`)

5. Adapter le `webServer.command` si necessaire (devrait rester
   `npm run dev` inchange)

**Critere d'acceptation** :
- `npx playwright test --list` execute le global-setup sans erreur
  (le daemon demarre et /health repond 200)
- Si des tests E2E echouent pour des raisons non liees au setup
  (ex: donnees de test manquantes post-migration Python→Rust),
  documenter comme P2 carry S64

### §1.3 Delta tests Phase A

| Suite | Delta | Detail |
|---|---|---|
| Rust | +0 | test existant adapte, pas de nouveau test |
| Vitest | +0 | pas de nouveau composant |
| Playwright | N/A → operationnel | setup fonctionnel, tests existants re-executebles |

**Commit** : `feat(launcher+web): Sprint 63 Phase A — MANDATORY IMAGE-DEP + PLAYWRIGHT-REFACTOR`

---

## §2 Phase B — Provenance endpoint HTTP + stockage SQLite

### §2.1 Migration M12 provenance_records

**Etapes** :

1. Ajouter M12 dans `db.rs` MIGRATIONS array :
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

2. Ajouter les fonctions DB dans `db.rs` :
   - `insert_provenance_record(project_id, record: &ProvenanceRecord)`
   - `get_provenance_by_project(project_id) -> Option<ProvenanceRecord>`

### §2.2 Insert au deploy

Dans `deploy.rs`, apres la generation du `ProvenanceRecord` (ligne 159)
et avant le publish de l'annonce :
- Appeler `db.insert_provenance_record(project_id, &prov)`
- Le `project_id` est derive du hash de l'annonce (meme pattern que
  `BrowseAggregator`)

Note : le project_id au moment du deploy doit correspondre a celui
du BrowseEntry. Verifier le pattern dans `browse.rs` pour la
derivation du project_id.

### §2.3 Endpoint GET /api/v1/project/{id}/provenance

Dans `http.rs` :
- Route : `GET /api/v1/project/:project_id/provenance`
- Handler :
  1. `db.get_provenance_by_project(project_id)`
  2. Si absent → 404
  3. Si present → `verify_provenance(record_json, &public_key)` pour
     verification live
  4. Retourner `{ record: ProvenanceRecord, verified: bool }`

### §2.4 Tests

| Test | Description |
|---|---|
| `test_provenance_insert_and_retrieve` | M12 insert + get roundtrip |
| `test_provenance_endpoint_found` | GET 200 avec record + verified=true |
| `test_provenance_endpoint_not_found` | GET 404 sans provenance |
| `test_provenance_endpoint_verified` | verification live retourne true |

### §2.5 Delta tests Phase B

| Suite | Delta | Detail |
|---|---|---|
| Rust | +4 | DB roundtrip + 3 handler tests |
| Vitest | +0 | pas de frontend touche |

**Commit** : `feat(feed): Sprint 63 Phase B — provenance endpoint HTTP + SQLite M12`

---

## §3 Phase C — Bridge verification + UI proof-chain

### §3.1 Bridge methodes (sbfb-bridge.js + useBridge.ts)

3 nouvelles methodes dans `sbfb-bridge.js` :

1. `getProvenanceRecord(projectId)` :
   - Type message : `provenance_get`
   - Request : `{ project_id: string }`
   - Response : `{ record: ProvenanceRecord | null }`

2. `verifyRelease(projectId)` :
   - Type message : `provenance_verify`
   - Request : `{ project_id: string }`
   - Response : `{ verified: bool, record: ProvenanceRecord | null, error?: string }`

3. `getPublicFeedCursor()` :
   - Type message : `feed_cursor_get`
   - Request : `{}`
   - Response : `{ last_seq: number, last_entry_hash: string }`

Handlers HTTP correspondants dans `http.rs` :
- `GET /api/v1/bridge/provenance/:project_id` (reutilise le handler Phase B)
- `GET /api/v1/bridge/provenance/:project_id/verify` (verification)
- `GET /api/v1/bridge/feed/cursor` (lecture cursor M10)

Dispatch dans `useBridge.ts` : 3 nouveaux cases dans le switch.

### §3.2 Composant VerificationDetail

Nouveau fichier `web/src/components/VerificationDetail.tsx` :

- Props : `projectId: string`, `provenanceHash: string | null`,
  `open: boolean`, `onClose: () => void`
- UI : shadcn `Dialog` modal
- Contenu :
  - Titre "Details de verification"
  - 7 champs provenance (repo_url lien cliquable, commit_sha copie,
    artifact_hash, signature tronquee, node_id tronque, timestamp
    formate, schema_version)
  - Bouton "Verifier maintenant" → fetch verify endpoint → badge
    resultat (vert/rouge)
  - Etat loading pendant le fetch
  - Etat empty si pas de provenance (texte explicatif)

### §3.3 Integration BrowsedProject.tsx

Dans `BrowsedProject.tsx` :
- Le badge ShieldCheck existant (ligne ~276) devient cliquable
  (onClick → setState open VerificationDetail)
- Ajout `<VerificationDetail>` avec `projectId` et `provenanceHash`
  de l'entry

### §3.4 Tests

| Test | Description |
|---|---|
| `VerificationDetail.test.tsx` | render modal, affiche champs, handle close |
| `VerificationDetail.test.tsx` | etat loading, etat empty |
| `VerificationDetail.test.tsx` | bouton verify trigger fetch |
| `useBridge provenance_get` | dispatch + mock response |
| `useBridge provenance_verify` | dispatch + mock response |
| `useBridge feed_cursor_get` | dispatch + mock response |

### §3.5 Delta tests Phase C

| Suite | Delta | Detail |
|---|---|---|
| Rust | +2 | 2 handlers bridge HTTP (cursor + verify relay) |
| Vitest | +6 | 3 VerificationDetail + 3 bridge dispatch |

**Commit** : `feat(web+bridge): Sprint 63 Phase C — bridge verification + UI VerificationDetail`

---

## §4 Phase D — Protocol Explorer verification + wrap-up

### §4.1 Protocol Explorer (si budget le permet)

Nouvelle section dans `examples/sbfb-explorer/index.html` :
- Section "Verification & Provenance" (6eme section)
- Contenu : explication du flow deploy verifie, bouton demo
  `verifyRelease()` via bridge, affichage resultat live
- Meme pattern HTML/CSS/JS que les 5 sections existantes

### §4.2 Wrap-up planning

- Rediger `sprint63_verification.md` (fail-fast checklist)
- Rediger `sprint64_audit_plan.md`
- Mettre a jour compteurs dans CLAUDE.md, SPRINT_LOG.md
- Mettre a jour memory nexus_grid_pivot.md

### §4.3 Delta tests Phase D

| Suite | Delta | Detail |
|---|---|---|
| Rust | +2 | tests supplementaires si Protocol Explorer wire |
| Vitest | +0 | HTML pur, pas de React |

**Commit** : `feat(examples): Sprint 63 Phase D — Protocol Explorer verification + wrap-up`

---

## §5 Fail-fast checklist (preview)

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1307, 0 fail |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | npm lint | `npm run lint` (web/) | 0 error |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error |
| 8 | Vitest | `npm run test:unit` (web/) | >= 264 |
| 9 | npm build | `npm run build` (web/) | ok |
| 10 | size-limit | `npm run size` (web/) | 6/6 |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean |
| 12 | sync-bridge-sdk | `bash scripts/sync-bridge-sdk.sh` | exit 0 |
| 13 | Playwright setup | `npx playwright test --list` | global-setup OK |
| 14 | Phase A-D preflights G8 | sprint63_phase_{A..D}_preflight.md | EXECUTE |
| 15 | Phase A-D reviews | sprint63_phase_{A..D}_review.md | PASS |

---

## §6 Estimation LOC par phase

| Phase | Rust | TypeScript | HTML/CSS | Total estime |
|---|---|---|---|---|
| A — MANDATORY | ~40 (tray.rs) | ~80 (global-setup) | 0 | ~120 |
| B — Provenance endpoint | ~150 (db+deploy+http) | 0 | 0 | ~150 |
| C — Bridge + UI | ~60 (handlers) | ~250 (bridge+component) | 0 | ~310 |
| D — Explorer + wrap-up | 0 | ~30 (si explorer) | ~100 | ~130 |
| **Total** | **~250** | **~360** | **~100** | **~710** |
