# Sprint 16 — Plan detaille (security hardening)

**Ecrit** : 2026-04-14
**Tip d'entree** : `14ec51e` (PARA migration, post audit S15 PASS)
**Commit stack attendu** : 5 phases A-E, ~3230 LOC, +~100 tests

Phase 0 (audit Sprint 15) DONE. Verdict PASS, landed :
- `e99c06f` docs(sprint15): audit findings
- `14ec51e` chore(planning): PARA layout S0-15 archives

---

## Vue d'ensemble

| Phase | Goal | LOC | Tests nouveaux | Commit |
|---|---|---|---|---|
| A | Bearer 256-bit + Host allowlist + Origin check | ~550 | +30 | `feat(auth): Sprint 16 Phase A — loopback hardening with bearer + Host + Origin` |
| B | UDS avec SO_PEERCRED + Named Pipes DACL custom | ~600 | +25 | `feat(net): Sprint 16 Phase B — UDS peer creds + Named Pipes DACL` |
| C | Consent **4 niveaux** (+L3 whitelist) + caps W/VRAM/h worker-enforced | ~830 | +35 | `feat(consent): Sprint 16 Phase C — GPU opt-in dialog (4 levels + whitelist) + worker caps enforcement` |
| D | PA v5 + is_open_source derive auto | ~250 | +15 | `feat(p2p): Sprint 16 Phase D — ProjectAnnouncement v5 with is_open_source flag` |
| E | Docs STRIDE+LINDDUN + RUNTIME_ISOLATION + verif | ~1100 | 0 | `docs(sprint16): verification + audit plan + security roadmap` |
| **Total** | | **~3330** | **+105** | 5 commits |

Ordre justifie :
- A avant B : le bearer est la base ; UDS peer creds en defense
  additionnelle. Sans A, UDS seul ne protege pas le browser.
- B avant C : D1+D2 stabilisent la surface reseau. Consent peut
  appeler `/consent/set` avec les nouvelles auth en place.
- C avant D : les caps worker utilisent `is_open_source` du PA
  v5 pour filtrer. D fournit le flag, C le consomme.
- E en fin : threat model integre les mitigations effectivement
  livres en A-D, pas un wish-list.

---

## Phase A — Bearer 256-bit + Host + Origin

### Scope code

- **`crates/nexus-launcher`** +50 LOC :
  - `auth::generate_or_load_token() -> Result<Token>` genere
    token 256-bit via `getrandom`, hex-encode (64 chars),
    persist `~/.sbfb/auth_token` mode `0600`, parent dir `0700`.
    Idempotent : lit le fichier s'il existe, genere sinon.
  - Endpoint HTTP `GET /auth/token` sur le port launcher
    (loopback seul) : retourne `{ "token": "<hex>" }`. Le shell
    React l'appelle au boot pour injecter dans `X-SBFB-Token`.
  - Tests : unit test `generate_or_load_token` idempotent,
    mode correct, length correcte.

- **`crates/nexus-shell-daemon-core`** +60 LOC :
  - `middleware::auth_required()` tower/axum layer qui valide :
    1. `X-SBFB-Token` present et matche `SBFB_DAEMON_TOKEN`
       env (le daemon le lit au boot depuis le meme fichier).
    2. `Host: localhost | 127.0.0.1 | [::1]` (avec optionnel
       `:<port>`). Sinon 403.
    3. `Origin:` absent OR dans allowlist
       (`http://localhost:<shell_port>`). Sinon 403.
  - Exception : `/health` sans auth (probe).

- **`packages/nexus-coordinator/src/nexus_coordinator/middleware/auth.py`** +70 LOC :
  - FastAPI middleware equivalent. Meme logique token + Host +
    Origin. Exception `/health`.
  - Token lu depuis `~/.sbfb/auth_token` au startup de l'app,
    cache en memoire.

- **`web/src/api/auth.ts`** +20 LOC :
  - `useAuthToken()` hook React Query qui fetch
    `http://localhost:<launcher_port>/auth/token` une fois au
    boot, cache le token.
  - `authFetch(url, init)` wrapper qui injecte
    `X-SBFB-Token: <token>` + `Origin: http://localhost:<shell>`.
  - Tous les appels aux APIs (coord, daemon) passent par
    `authFetch`.

### Tests (+30)

- cargo (`nexus-shell-daemon-core`) 10 : auth middleware unit :
  - token valide + Host + Origin → 200
  - token absent → 401
  - token mismatch → 401
  - Host = `attacker.com` → 403
  - Origin = `https://attacker.com` → 403
  - path `/health` sans token → 200
- pytest (`nexus-coordinator`) 10 : middleware equivalent
- vitest (`web/src/api`) 5 : `authFetch` inject headers, fetch
  failures propagate, token caching
- Playwright 5 : scenario end-to-end shell → coord via authFetch

### Critere Phase A

- Requests loopback sans `X-SBFB-Token` → 401
- Requests avec `Host: attacker.com` → 403
- Requests avec `Origin:` non-whitelisted → 403
- Avec token + Host + Origin valides → 200
- `/health` sans auth → 200 (probe uniquement)

### Commit A

```
feat(auth): Sprint 16 Phase A — loopback hardening with bearer + Host + Origin

Triple validation des requests HTTP loopback :
- X-SBFB-Token 256-bit genere par le launcher au boot
- Host allowlist (localhost/127.0.0.1/[::1])
- Origin check (shell React uniquement, absent autorise CLI)

Mitigation CVE-2025-49596 (Anthropic MCP Inspector RCE via
DNS rebinding) + pattern Jupyter/Syncthing. Defense en profondeur
compatible avec D2 (SO_PEERCRED sur UDS, Phase B).

- crates/nexus-launcher : generate_or_load_token + /auth/token
- crates/nexus-shell-daemon-core : middleware axum
- packages/nexus-coordinator : middleware FastAPI
- web/src/api/auth : authFetch wrapper + useAuthToken hook

Tests : 373 + 30 = 403 cargo | 153 + 10 = 163 coord | 214 + 5 =
219 vitest | 33 + 5 = 38 Playwright
```

---

## Phase B — UDS + Named Pipes avec SO_PEERCRED + DACL

### Scope code

- **`crates/nexus-shell-daemon`** +80 LOC (Unix) :
  - `listen_uds() -> Result<UnixListener>` sur
    `~/.sbfb/run/daemon.sock`, mode `0600` apres bind, parent
    dir `0700`.
  - `verify_peer_creds(stream)` via
    `getsockopt(SOL_SOCKET, SO_PEERCRED)` : compare uid peer
    vs uid propre (`geteuid()`). Reject si different.
  - Feature flag `uds` activate par defaut sur Unix, no-op
    sur Windows.
  - Axum `Router::into_make_service()` avec UnixListener
    (axum 0.7 supporte tokio::net::UnixListener depuis 0.7.4).

- **`crates/nexus-shell-daemon`** +120 LOC (Windows) :
  - Dep `windows = "0.57"` avec features `["Win32_System_Pipes",
    "Win32_Security"]`.
  - `listen_named_pipe() -> Result<NamedPipeServer>` :
    - `GetTokenInformation` pour recuperer le SID du user
      courant
    - Build `SECURITY_ATTRIBUTES` + `SECURITY_DESCRIPTOR` +
      DACL avec 1 seule ACE : Allow + user SID + full control
    - `CreateNamedPipeA(r"\\.\pipe\sbfb-daemon",
      PIPE_ACCESS_DUPLEX, ..., lpSecurityAttributes)`
  - Loop accept → handle request avec le meme axum Router.

- **`crates/nexus-shell-daemon-core`** +30 LOC :
  - Helper commun `auth::verify_peer_or_token(req, creds)` :
    - Si connection UDS/NP avec peer_creds OK → bypass bearer
    - Sinon → valider bearer + Host + Origin (Phase A)

- **`packages/nexus-coordinator`** +60 LOC :
  - UnixServer via `uvicorn --uds ~/.sbfb/run/coordinator.sock`
    (uvicorn 0.30+ supporte `--uds` depuis 0.19)
  - Named Pipe Windows : option deleguer a un Rust side-car
    qui forward vers FastAPI TCP, OU via `pywin32` direct.
    **Decision Phase B jour 1** : si pywin32 propre, rester
    Python ; sinon side-car Rust proxy.
  - Helper Python `peer_creds.py` lit `SO_PEERCRED` via
    `socket.getsockopt(SOL_SOCKET, SO_PEERCRED, ...)` + struct.

- **`crates/nexus-launcher`** +40 LOC :
  - Cree `~/.sbfb/run/` mode `0700` au boot (Unix)
  - Sur Windows, pas de dir needed (pipes = namespace kernel)
  - Expose via `/runtime/sockets` endpoint les paths pour les
    clients (CLI `sbfb`, daemon)

### Tests (+25)

- cargo (`nexus-shell-daemon`) 12 : UDS peer creds :
  - connect meme user → 200
  - connect different user (mock fork+setuid, Linux-only) → 401
  - UDS mode `0600` verification
  - parent dir `0700` verification
  - NP Windows DACL : ACE present avec user SID
  - NP Windows : autre user rejected (skip non-Windows)
- pytest 8 : coord uvicorn UDS, peer creds Python
- Integration 5 : CLI `sbfb` talks to daemon via UDS
  (fallback TCP si UDS absent)

### Critere Phase B

- Sur Linux/Mac : UDS existe avec mode `0600`. Connection UDS
  accepte sans bearer (peer creds OK). Connection avec autre uid
  rejected.
- Sur Windows : Named Pipe `\\.\pipe\sbfb-daemon` accessible au
  user courant, rejected pour autre user (DACL enforce).
- TCP reste disponible pour browser (bearer + Host + Origin).

### Commit B

```
feat(net): Sprint 16 Phase B — UDS peer creds + Named Pipes DACL

UDS `~/.sbfb/run/*.sock` avec SO_PEERCRED validation (pattern
Tailscale safesocket). Named Pipes Windows avec
SECURITY_ATTRIBUTES custom + DACL user-only (prevent exploit
Named Pipe default permissive DACL).

Browser continue en TCP bearer-authentifie (Phase A). CLI Rust
et coord parlent UDS/NP en priorite, TCP fallback.

Tests : 403 + 12 = 415 cargo | 163 + 8 = 171 coord | +5
integration UDS
```

---

## Phase C — Consent 4 niveaux (+L3 whitelist) + caps worker-enforced

### Scope code

- **`web/src/components/GpuConsentDialog.tsx`** +320 LOC :
  - Dialog shadcn/ui, **4 radios** (default = L1 "mes projets",
    pas pre-selection GDPR-safe)
  - Sliders caps : W max [10, 500] W, VRAM max [1, 24] GB,
    heures/jour [0, 24] h
  - **Section whitelist L3** (visible uniquement si L3 coche) :
    - Input texte "Ajouter un projet" qui accepte :
      - node_id hex (64 chars) → ajout direct
      - URL git (https://github.com/...) → resolution via
        `POST /consent/whitelist/add` (le coord resout URL vers
        le node_id publie)
    - Liste actuelle avec bouton "Retirer" par item
    - Empty state : "Aucun projet dans ta whitelist. Utilise
      le bouton 'Contribuer mon GPU' sur la page Browse pour
      ajouter rapidement."
  - `POST /consent/set` body `{ level: 1|2|3|4, cap_watts,
    cap_vram_mb, cap_hours_day, allowed_project_ids: string[] }`
  - Validation cote client + cote API Python.

- **`web/src/pages/BrowsedProject.tsx`** +30 LOC :
  - Nouveau bouton "Contribuer mon GPU" dans la top bar
    (icone Heart), visible uniquement si consent.level === 3
  - Click → POST `/consent/whitelist/add` avec
    `{ project_id: entry.project_id }`. Toast de confirmation.
  - Si le projet est deja dans la whitelist, le bouton indique
    "Contribution active" + click = remove.

- **`web/src/pages/Network.tsx`** +40 LOC :
  - Badge coin haut droit indiquant le level actuel (1/2/3/4)
  - Bouton "Modifier consentement" → reopen dialog
  - Si L3 : mini-liste des projets whitelistes visible en preview

- **`packages/nexus-coordinator/src/nexus_coordinator/consent.py`** +80 LOC :
  - `GET /consent/get` → lit `~/.sbfb/consent.json`
  - `POST /consent/set` → valide payload + write atomique
    (tmp + rename)
  - `POST /consent/whitelist/add` body `{ project_id | repo_url }` :
    - Si project_id hex → ajout direct
    - Si repo_url → resolution best-effort (query local browse
      aggregator pour trouver un projet avec ce repo_url)
  - `POST /consent/whitelist/remove` body `{ project_id }`
  - Pydantic model `ConsentConfig` avec
    `allowed_project_ids: list[str]`

- **`crates/nexus-worker-core/src/allowlist.rs`** +180 LOC :
  - `ConsentLevel { OwnProjects = 1, OpenSource = 2,
    Whitelist = 3, All = 4 }`
  - `Caps { max_watts, max_vram_mb, max_hours_day }`
  - `ConsentConfig` inclut `allowed_project_ids: HashSet<NodeId>`
    (HashSet pour O(1) lookup)
  - `UsageTracker` : charge `~/.sbfb/usage.json`, expose
    `reserve_hours(h) -> Result<()>` qui verifie cumul + reset
    a minuit-local.
  - `should_accept_task(&task, &consent) -> AllowOutcome` :
    1. L1 → reject si `task.project_id != self.node_id`
    2. L2 → reject si `!task.is_open_source`
    3. L3 → reject si `!consent.allowed_project_ids.contains(&task.project_id)`
    4. L4 → pass
    5. caps : reject si `task.watts_estimate > max_watts`,
       `task.vram_mb > max_vram_mb`, `usage.hours_today + task.duration_h > max_hours_day`
  - Persistance usage.json a chaque task completed (atomic
    write).
  - **File watcher** : le worker re-lit consent.json quand il
    change (via `notify` crate) pour appliquer les add/remove
    whitelist sans redemarrage.

- **`crates/nexus-worker`** +20 LOC : appelle
  `allowlist.should_accept_task` dans le claim loop.

### Tests (+35)

- vitest 12 : dialog rendering (4 radios, caps validation,
  whitelist section toggle), `POST /consent/set` flow,
  `POST /consent/whitelist/add` inline, badge reflects level,
  bouton "Contribuer mon GPU" visible uniquement L3
- pytest 7 : `/consent/get`/`set` API, `/consent/whitelist/add`
  resolut repo_url → project_id, `/consent/whitelist/remove`
  idempotent, JSON validation, atomic write, error recovery
- cargo `nexus-worker-core::allowlist` 16 :
  - L1 accept own, reject other
  - L2 accept is_open_source, reject not
  - **L3 accept si project_id dans whitelist, reject sinon**
  - **L3 whitelist empty reject tout (sauf L1 conditions)**
  - L4 accept all
  - caps : reject task > max_watts
  - caps : reject task > max_vram_mb
  - caps : reject apres cumul >= max_hours_day
  - reset a minuit local : usage.json rebuild, accept retombe
  - **file watcher : consent.json rewrite → nouveau state applique
    avant le prochain claim (verifier via polling ou event)**

### Critere Phase C

- 1er boot → dialog visible, 4 radios, default L1, enregistrer
  persiste consent.json
- Worker refuse une task `is_open_source=false` quand L2
- **Worker L3 refuse une task dont project_id n'est pas dans
  la whitelist ; accepte quand ajoute**
- Bouton "Contribuer mon GPU" sur Browse ajoute le projet en
  1 clic (test en conditions reelles : start worker, click,
  verifier qu'une task de ce projet est maintenant acceptee)
- Worker refuse une task apres cumul h_day atteint
- "Modifier consentement" rouvre le dialog, sauve, effets
  immediats sur le worker (file watcher `notify` crate detecte
  le write et reload consent.json sans redemarrer le worker)

### Commit C

```
feat(consent): Sprint 16 Phase C — GPU opt-in dialog (4 levels + whitelist) + worker caps enforcement

Pattern BOINC UserOptInConsent + GDPR Art.7 (opt-in explicite,
granular, withdrawal simple).

- Dialog React 4 niveaux :
  L1 "mes projets" (default, zero partage)
  L2 "open source verifies" (filtre is_open_source flag)
  L3 "projets specifiques" (whitelist manuelle editable)
  L4 "tous les projets publics"
- Raccourci "Contribuer mon GPU" sur page Browse pour ajouter
  1 projet en 1 clic a la whitelist L3
- Caps W/VRAM/h max configurables
- consent.json persiste preferences + allowed_project_ids
- worker-core allowlist enforce niveau + caps + whitelist a
  chaque claim + daily counter usage.json reset minuit
- File watcher notify crate applique les changements sans
  redemarrage du worker

Bloquee derriere D1+D2 (/consent/* et /consent/whitelist/*
passent par loopback auth).

Tests : 415 + 16 = 431 cargo | 171 + 7 = 178 coord | 219 + 12
= 231 vitest
```

---

## Phase D — ProjectAnnouncement v5 + is_open_source

Note : les compteurs tests de la Phase D s'ajoutent au cumul
post-C : 431 → 439 cargo / 178 → 182 coord / 231 → 234 vitest.



### Scope code

- **`crates/nexus-core-rs/src/project_announcement.rs`** +30 LOC :
  - Enum interne version bump `V4 | V5`
  - `V5 { ..v4_fields.., is_open_source: bool }`
  - Decoder retro-compatible : si serialized contient pas le
    champ, default `false`.
  - `ProjectAnnouncementV5::is_open_source()` method.

- **`packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py`** +5 LOC :
  - Dans `deploy_from_repo` (public = true), set le flag
    `is_open_source=true` lors de la creation du PA.
  - Dans `deploy` (upload zip, prive), set `false`.

- **`web/src/api/project_announcement.ts`** (ou Zod schema) +15 LOC :
  - Schema Zod v5 : optional `is_open_source: z.boolean().default(false)`
  - Type export mis a jour

### Tests (+15)

- cargo `nexus-core-rs` 8 :
  - encode V5 with flag true
  - encode V5 with flag false
  - decode V4 legacy → flag defaults false
  - decode V5 → flag preserved
  - round-trip binary stability
- pytest 4 : deploy-from-repo → PA has is_open_source=true
- vitest 3 : Zod schema accept v4, accept v5

### Critere Phase D

- Un deploy-from-repo produit une PA v5 avec is_open_source=true
- Un noeud sur ancien code decode la PA v5 en ignorant
  le champ (decoder tolerant) — pas de crash
- Le consent niveau 2 (Phase C) utilise bien le flag pour
  filtrer

### Commit D

```
feat(p2p): Sprint 16 Phase D — ProjectAnnouncement v5 with is_open_source flag

Derive automatiquement par le coordinator : true pour
deploy-from-repo (repo public), false pour deploy (zip prive).
Non-user-settable (pattern npm provenance, cosign attestation).

Backward compat : decoder V4 legacy default false. Pas de
migration forcee.

Tests : 430 + 8 = 438 cargo | 176 + 4 = 180 coord | 229 + 3
= 232 vitest
```

---

## Phase E — Documentation security + roadmap

### Scope docs

- **`docs/security/README.md`** ~60 LOC : index,
  matrice severite (C/H/M/L), pointeurs, contrib guide.

- **`docs/security/THREAT_MODEL.md`** ~500 LOC :
  - §1 Scope & assumptions
  - §2 Assets table (7 items)
  - §3 Adversary model (5 personas)
  - §4 DFD ASCII flows
  - §5 STRIDE par composant (iframe, bridge, deploy-from-repo,
    iroh, loopback, worker-core, storage)
  - §6 LINDDUN par flux (7 categories, focus GDPR)
  - §7 Mitigations table (livre S16 + roadmap S17+)
  - §8 Residual risks

- **`docs/security/RUNTIME_ISOLATION.md`** ~350 LOC :
  - §1 Rationale (95% local risks eliminated)
  - §2 Technologies : WSL2 / Virtualization.framework /
    systemd-nspawn
  - §3 CUDA passthrough (WSL2 natif, VMware/QEMU limites)
  - §4 Phasage Sprint 17+ (4 phases)
  - §5 Backward compat strategy
  - §6 Alternative sans virtualisation (process isolation,
    seccomp, AppArmor, BPF)

- **`CLAUDE.md`** update : section "Etat actuel" avec Sprint 16
  CLOSED, compteurs tests mis a jour, pointeur vers
  `docs/security/`.

- **`README.md`** update : section "Security" enrichie avec
  STRIDE+LINDDUN mention + pointeur threat model.

- **`docs/claude/README.md`** §10 row Sprint 16 / v1.2.

- **`docs/shell/PATTERNS.md`** +40 LOC : nouveau pattern
  "Defense en profondeur loopback (bearer + Host + Origin +
  peer creds)".

- **`.planning/active/sprint16_verification.md`** : fail-fast
  checklist 35+ rows.

- **`.planning/active/sprint16_audit_plan.md`** : plan audit
  S17 Phase 0 (6-7 tracks : auth middleware, UDS peer creds,
  Named Pipes DACL Windows, consent+caps, PA v5 compat,
  docs completeness, scope cuts).

### Tests : 0 nouveau (docs pur)

### Critere Phase E

- `docs/security/` contient 3 fichiers avec les sections
  promises
- Threat model cite chaque mitigation livree A-D avec
  reference au commit
- Roadmap S17+ laisse des hooks explicites (les 4 phases
  d'auto-install WSL2)

### Commit E

```
docs(sprint16): verification + audit plan + security roadmap

- docs/security/README.md + THREAT_MODEL.md (STRIDE + LINDDUN)
  + RUNTIME_ISOLATION.md (WSL2 / Virtualization.framework /
  systemd-nspawn roadmap Sprint 17+)
- CLAUDE.md + README.md : pointeurs sections Security
- docs/claude/README.md §10 : Sprint 16 / v1.2 row
- docs/shell/PATTERNS.md : pattern defense en profondeur
  loopback
- .planning/active/sprint16_verification.md : fail-fast 35+
- .planning/active/sprint16_audit_plan.md : plan audit S17

Tests : unchanged (438 cargo / 180 coord / 232 vitest / 38
Playwright / 7/7 size / 228 SPDX ~1123 total)
```

---

## Fail-fast checklist globale (pre-cloture)

| # | Check | Commande | Target |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warn |
| 3 | cargo test | `cargo test --workspace --locked` | 373 → 438 (+65) |
| 4 | cargo test Windows-only | manuel on Win11 | Named Pipe DACL tests pass |
| 5 | ruff format | `uv run ruff format --check packages/` | clean |
| 6 | ruff check | `uv run ruff check packages/` | clean |
| 7 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | 182 pass |
| 8 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | 153 → 180 (+27) |
| 9 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass |
| 10 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | clean |
| 11 | eslint | `npm run lint` | 0 errors |
| 12 | vitest | `npm run test:unit` | 214 → 232 (+18) |
| 13 | build | `npm run build` | success |
| 14 | size-limit | `npm run size` | 7/7 under budget |
| 15 | scan-en-strings | `bash scripts/scan-en-strings.sh` | French-only |
| 16 | Playwright | `npx playwright test` | 33 → 38 (+5) |
| 17 | Manual : bearer 401 | `curl http://localhost:8080/app/gov/tabs` | 401 |
| 18 | Manual : bearer 200 | `curl -H "X-SBFB-Token: $(cat ~/.sbfb/auth_token)" ...` | 200 |
| 19 | Manual : Host rebind | `curl -H "Host: attacker.com" ...` | 403 |
| 20 | Manual : Origin block | `curl -H "Origin: https://x.com" ...` | 403 |
| 21 | Manual : UDS Linux | `curl --unix-socket ~/.sbfb/run/daemon.sock http://x/health` | 200 |
| 22 | Manual : UDS autre user | `sudo -u other curl --unix-socket ...` | EACCES |
| 23 | Manual : consent dialog | first boot → visible | render OK |
| 24 | Manual : worker reject | inject task > cap_watts | rejected with reason |
| 25 | Manual : PA v5 flag | deploy-from-repo → inspect announcement | is_open_source=true |
| 26 | SPDX | nouveaux fichiers | 224 → 240 (+16) |

---

## Risques R1..R7

| # | Risque | Mitigation |
|---|---|---|
| R1 | Named Pipes Windows DACL API complexe (windows-rs learning curve) | Phase B jour 1 : spike 4h pour valider l'approche. Fallback : Rust side-car proxy TCP → Named Pipe |
| R2 | axum 0.7 UnixListener : breaking change avec 0.8 ? | Pinner axum = "0.7.9" dans Cargo.toml, noter upgrade dans PATTERNS.md |
| R3 | pywin32 dependency lourde pour coord Windows | Alt : Python via subprocess → un Rust binaire qui owne le Named Pipe. Decision Phase B apres spike |
| R4 | Consent dialog UX confusion (caps sliders unclear) | User test en interne avant cloture Phase C. Tooltip explicatif sur chaque slider |
| R5 | Worker reset minuit timezone : DST bugs | Utiliser `chrono::Local::now().date_naive()` + check `today != cached_today`. Test avec `TZ=America/Chicago` + fake clock |
| R6 | PA v5 decoder tolerant casse un vieux noeud qui requiere les fields stricts | Verifier `crates/nexus-core-rs` deserde config accept extra fields. Test round-trip V4 ↔ V5 |
| R7 | Threat model trop generic, ne reflete pas le vrai systeme | Ecrit en Phase E APRES A-D implementees. Cite les commits et les LOC exactes. Fail si un reviewer externe (user lui-meme ou prompt 2e pass) identifie une mitigation listee mais absente du code |

---

## Scope cuts stricts (differes Sprint 17+)

Cf. `sprint16_kickoff.md` §6. Rappel court :
- Auto-install WSL2 / VM
- Encryption at rest keypair (Keychain/DPAPI/libsecret)
- CI security audit (cargo-audit/pip-audit/npm audit)
- Rate limiting deploy-from-repo
- CSP report-uri
- Audit externe
- Revocation node_id
- MIME scan zip
- Multi-level consent per-project
- Bytecode signing PyO3
- Token rotation automatique

---

## Compteurs tests attendus en sortie

| Suite | Entree (tip `14ec51e`) | Sortie | Delta |
|---|---|---|---|
| Rust workspace | 373 | 439 | +66 |
| Python SDK | 182 + 1 flaky | 182 + 1 flaky | = |
| Python coordinator | 153 + 1 skip | 182 + 1 skip | +29 |
| Python app-gov | 46 | 46 | = |
| Vitest unit | 214 | 234 | +20 |
| Playwright | 33 | 38 | +5 |
| size-limit | 7/7 | 7/7 | = |
| SPDX | 224 | 242 | +18 |

Total : ~934 → ~1054 tests (+120).

---

## Migration / upgrade notes

- Les users existants v1.1 doivent redemarrer daemon +
  coordinator apres upgrade v1.2 pour que le launcher genere
  le token `~/.sbfb/auth_token`.
- Sans redemarrage, les anciens processus continuent de
  fonctionner (pas de token generate = middleware en mode
  legacy warn-only pendant la fenetre d'overlap). Un warning
  STDERR informe.
- Le CLI `sbfb` v1.2 detecte UDS/NP, fallback TCP auto.
- PA v5 consommable par tous les noeuds v1.1+ (decoder
  tolerant). Les noeuds v1.2+ voient is_open_source=true
  sur deploy-from-repo, v1.1- voient la PA complete mais
  ignorent le champ.
