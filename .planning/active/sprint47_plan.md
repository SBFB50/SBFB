# Sprint 47 — Plan

**Tip d'entree** : `d1ef20d`.
**Referentiel** : `sprint47_kickoff.md` D1..D4 gelees.

---

## Phase A — S45 carries resolution (3 items 2/3)

### §A.1 P2-REVIEW-A-1-S45 : diagnostic Err path tests

**Fichier** : `crates/nexus-shell-daemon/src/diagnostic_api.rs`
(105 LOC) + tests dans `crates/nexus-shell-daemon/src/http.rs`.

**4 error paths identifies** :
1. L25 — `coordinator_db.lock()` Mutex poisoned → 500
2. L36 — `worker_contributions()` query DB error → 500
3. L52 — `active_workers_since()` current window error → 500
4. L62 — `active_workers_since()` previous window error → 500

**Pattern existant** : le test
`diagnostic_fairness_returns_500_on_poisoned_mutex` (S46 Phase B)
couvre le path 1. Il faut ajouter les paths 2-4 en injectant des
conditions d'erreur DB.

**Tests a ajouter** :
- `diagnostic_worker_contributions_error_returns_500` (path 2)
- `diagnostic_active_workers_current_error_returns_500` (path 3)
- `diagnostic_active_workers_previous_error_returns_500` (path 4)

**Approche** : si les error paths 2-4 sont atteignables uniquement
par DB corruption/schema mismatch, verifier si un
`CoordinatorDb` vide (sans tables) suffit a triggerer l'erreur.
Sinon, documenter pourquoi le path est unreachable et clore le
carry avec evidence.

### §A.2 P2-REVIEW-A-2-S45 : invite ID collision fix

**Fichier** : `crates/nexus-shell-daemon/src/invite_api.rs`
(223 LOC).

**Code actuel** (L17, L80) :
```rust
static INVITE_COUNTER: AtomicU64 = AtomicU64::new(0);
let seq = INVITE_COUNTER.fetch_add(1, Ordering::Relaxed);
let id = format!("inv-{now}-{seq}");
```

**Collision** : 2 daemons debutent meme seconde → `inv-{ts}-0`
identiques.

**Fix** : ajouter le node_id (8 chars hex) dans le format.
Le `state.node_id` est un `String` hex 64 chars dans
`DaemonHttpState`. Prendre `&state.node_id[..8]`.

**Nouveau format** : `inv-{node_id_8}-{now}-{seq}`

**Tests a ajouter** :
- `invite_create_success` existant — verifier que l'ID contient
  le prefixe node_id
- Pas de test cross-daemon (necessite 2 Nodes, hors scope)

### §A.3 P2-REVIEW-B-1-S45 : Python modules audit + suppression

**Methode** : grep `coordinator.py` + `app.py` +
`packages/nexus-coordinator/src/nexus_coordinator/api/*.py` pour
chaque module candidat.

**Modules a auditer** (21 avec equivalents Rust) :

| Module Python | Equivalent Rust | Import coordinator.py ? |
|---|---|---|
| canary_input.py | canary_input.rs | OUI |
| canary_registry.py | canary_registry.rs | OUI |
| capability_store.py | capability_store.rs | a verifier |
| contributor_registry.py | contributor_registry.rs | OUI |
| dispatcher.py | dispatcher.rs | OUI |
| fairness.py | fairness.rs | a verifier |
| forge.py | forge.rs | a verifier |
| guardrails.py | guardrails.rs | a verifier |
| honeypot.py | honeypot.rs | a verifier |
| hooks.py | hooks.rs | a verifier |
| invite.py | invite.rs | OUI |
| kudos.py | kudos_ledger.rs | OUI |
| output_filter.py | output_filter.rs | OUI |
| pii_redactor.py | pii_redactor.rs | a verifier |
| pow_counter.py | pow_counter.rs | a verifier |
| provenance.py | provenance.rs | a verifier |
| quarantine_queue.py | quarantine_queue.rs | OUI |
| redundancy.py | redundancy.rs | a verifier |
| rerun.py | rerun.rs | a verifier |
| upload_queue.py | upload_queue.rs | OUI |
| validator.py | validator.rs | OUI |
| watermark_detector.py | watermark_detector.rs | a verifier |

**Critere de suppression** : module N'est PAS importe par
coordinator.py, app.py, ni aucun api/*.py restant, ni par un
autre module Python encore vivant. Si critere rempli : `git rm`
module + tests associes.

**Si 0 module supprimable** : carry clos avec evidence "tous les
modules candidats sont encore utilises par coordinator.py, la
suppression depend de App Runtime Migration Rust (scope cut
S48+)". La resolution par evidence documentee est valide.

### §A.4 Scope cuts Phase A

- Tests E2E diagnostic multi-daemon → hors scope (teste
  unitairement)
- Cross-daemon invite collision test reel → hors scope
  (2 Nodes necessaires)

### §A.5 Commit

```
feat(sprint47): Sprint 47 Phase A — S45 carries resolution
  diagnostic Err tests + invite ID fix + Python modules audit
```

---

## Phase B — Integration tests 5 routes restantes

### §B.1 deploy.rs integration tests

**Routes** :
- `POST /api/v1/deploy` (deploy_private) — zip upload CAS
- `POST /api/v1/deploy-from-repo` (deploy_from_repo) — clone +
  verify + zip

**Approach deploy_private** :
Le handler extrait le body `Bytes`, verifie la taille, valide le
format zip, calcule le BLAKE3 hash, stocke via `BlobsClient`.
Le test doit fournir un zip valide ET un `DaemonHttpState` avec
un Node dont le `blobs_store()` est fonctionnel.

Si `mk_state()` ne peut pas fournir un BlobStore fonctionnel
(l'iroh Node test n'expose pas de store simple), fallback :
tester seulement les **error paths** (zip invalide → 400, trop
grand → 413, body vide → 400).

**Approach deploy_from_repo** :
Le handler fait un `git clone` reel — ne peut pas etre teste
via Router::oneshot() sans infra git. Tester seulement les
**error paths** (repo_url invalide → 400/500, payload invalide
→ 400).

**Tests prevus** :
- `deploy_private_invalid_zip_returns_400`
- `deploy_private_empty_body_returns_400`
- `deploy_private_too_large_returns_413` (si testable sans
  envoyer 100MB — sinon skip)
- `deploy_from_repo_invalid_url_returns_400`
- `deploy_from_repo_missing_project_name_returns_400`

### §B.2 apps.rs integration tests

**Routes** :
- `GET /api/v1/apps` (list_apps) — from browse aggregator
- `GET /api/v1/apps/{project_id}` (get_app) — single entry

**Approach** : peupler le `BrowseAggregator` dans
`DaemonHttpState` avec des `BrowseEntry` mock (pattern existant
dans les 11 tests unitaires de apps.rs via `make_entry()`).
Enrichir `mk_state()` avec browse entries ou creer un
`mk_state_with_browse()` variant.

**Tests prevus** :
- `apps_list_returns_empty_when_no_entries`
- `apps_list_returns_entries_with_filters`
- `apps_get_returns_entry_by_id`
- `apps_get_unknown_id_returns_404`

### §B.3 auth/token integration test

**Route** : `GET /auth/token` (auth_token_public) — endpoint
public, pas d'auth required.

**Approach** : appeler via Router::oneshot() sans bearer token.
Le handler doit retourner un token valide.

**Tests prevus** :
- `auth_token_returns_200_with_valid_token`
- `auth_token_is_public_no_bearer_needed`

### §B.4 Scope cuts Phase B

- deploy_private happy path (zip + BlobStore) → scope cut si
  mk_state() ne peut pas fournir de BlobStore fonctionnel
- deploy_from_repo happy path → scope cut (git clone reel)

### §B.5 Commit

```
feat(sprint47): Sprint 47 Phase B — integration tests 5 routes
  deploy+apps+auth completion
```

---

## Phase C — Happy path tests + deprecated aliases cleanup

### §C.1 Consent happy path tests

**Routes** :
- `POST /api/v1/consent/set` (level 2) → 200
- `GET /api/v1/consent` → returns persisted level 2
- `POST /api/v1/consent/whitelist/add` (valid project_id) → 200
- `POST /api/v1/consent/whitelist/remove` (added project_id) → 200

**Approach** : les handlers consent.rs utilisent `sbfb_home()`
(env var `SBFB_HOME` ou `~/.sbfb/`), pas un champ
DaemonHttpState. Setter `SBFB_HOME` vers un tmpdir dans chaque
test (ou fixture shared). Le consent handler charge/sauvegarde
dans un fichier JSON sous `$SBFB_HOME/consent.json`.

**Tests prevus** :
- `consent_set_level_2_returns_200`
- `consent_get_returns_persisted_level`
- `consent_whitelist_add_returns_200`
- `consent_whitelist_remove_returns_200`

### §C.2 Files happy path tests

**Routes** :
- `POST /api/v1/files/upload` (petit fichier) → 200 + SHA
- `GET /api/v1/files/{sha256}/manifest` → manifest JSON
- `GET /api/v1/files/{sha256}` → stream blob

**Approach** : les handlers files.rs utilisent aussi `sbfb_home()`
pour le stockage CAS SHA-256. Meme technique que consent : setter
`SBFB_HOME` vers un tmpdir. Le CAS ecrit sous `$SBFB_HOME/files/`.

**Tests prevus** :
- `files_upload_small_returns_200_with_sha`
- `files_manifest_after_upload_returns_200`
- `files_stream_after_upload_returns_content`

### §C.3 Deprecated aliases cleanup

**Fichiers** :
- `web/src/api/coordinator.ts` L57-60, L71 : 3 alias exports
  (`CoordinatorProtocolError`, `CoordinatorHttpError`,
  `normalizeCoordinatorUrl`)
- `web/src/components/AddCoordinatorDialog.tsx` : 6 refs
- `web/src/stores/projectStore.ts` : 5 refs
- `web/src/api/__tests__/coordinator.test.ts` : 1 ref
- `web/src/components/command-palette/__tests__/CommandPalette.test.tsx` : 1 ref

**Approach** : remplacer les refs par les nouveaux noms
(`ApiProtocolError`, `ApiHttpError`, `normalizeApiUrl`), puis
supprimer les 3 alias exports. Verifier avec `npm run lint` +
`tsc --noEmit` + Vitest.

### §C.4 Scope cuts Phase C

- Tests E2E Playwright consent/files → hors scope (Router-level
  suffisant)

### §C.5 Commit

```
feat(sprint47): Sprint 47 Phase C — happy path tests
  consent+files + deprecated aliases cleanup
```

---

## Phase D — Wrap-up

### §D.1 verification.md

Fail-fast checklist 28+ rows, delta tests cumule, carries S48.

### §D.2 sprint48_audit_plan.md

Tracks A-F + carrys documentes pour session fraiche S48.

### §D.3 HARDENING_ROADMAP update

last_validated S47 + compteurs.

### §D.4 Commit

```
chore(sprint47): Phase D — wrap-up + verification + audit plan
  S48 + counters
```
