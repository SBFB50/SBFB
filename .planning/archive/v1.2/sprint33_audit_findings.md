# Sprint 33 — Audit findings (Phase 0 S34 gate)

**Date** : 2026-04-27
**Scope** : commits Phase A `a103696` → Phase C `3d3bd96`
(3 feat commits Sprint 33 — multi-node readiness)
**Audit plan** : `sprint34_audit_plan.md`

## Verdict : PASS

0 P0 / 0 P1 / 1 P2 / 1 P3. Rigor signal G4 satisfait
(>=1 P2 documenté). Aucun bloquant pour S34.

---

## Track A — Phase-level correctness

### A.1 CORS daemon opt-in preserves loopback default — OK

`http.rs` `cors_layer()` : sans `extra_origins`, retourne un
CORS layer qui n'autorise que les origines validées par
`is_loopback_origin()` (HTTP uniquement, host 127.0.0.1 ou
localhost, port optionnel). Avec `--cors-origin`, la logique
combine `is_loopback_origin(origin) || allowed.contains(origin)`
— loopback toujours préservé. 6 tests unitaires couvrent les
scénarios (default rejette externe, custom préserve loopback,
suffix trick rejeté).

### A.2 CORS coordinator opt-in preserves localhost default — OK

`app.py` : `allow_origin_regex` défini en base, `allow_origins`
ajouté conditionnellement quand `cors_origins` est fourni.
Starlette CORSMiddleware évalue les deux critères en OR — une
origine est acceptée si elle matche la regex OU la liste.
Confirmé par test `test_cors_custom_preserves_localhost` qui
vérifie explicitement que `http://127.0.0.1:5173` reste accepté
quand un custom origin externe est configuré. 3 tests CORS
dans `test_cors.py`, tous verts.

### A.3 LOC guard hook — OK

`phase-precommit-lightcheck.sh` Check 6 : patterns grep couvrent
`~NNN LOC`, `environ NNN LOC`, `budget LOC`, `LOC total` en
case-insensitive. Le hook bloque (exit 2) si un match est trouvé
dans un `sprint*_plan.md` staged. Filtre `/archive/` ajouté
correctement pour éviter les faux positifs lors de migrations.

### A.4 systemd units ExecStart — OK

3 service files vérifés :
- `nexus-daemon.service` : `ExecStart=/opt/nexus-grid/target/release/nexus-shell-daemon start --headless`
- `nexus-coordinator.service` : `ExecStart=/opt/nexus-grid/.venv/bin/nexus-coordinator start my-project --host 0.0.0.0 --port 8765`
- `nexus-worker.service` : `ExecStart=/opt/nexus-grid/target/release/nexus-worker start --headless`

Flags `--headless`, `--host`, `--port` existent dans les CLI
respectifs. Override example avec `--cors-origin` documenté.

### A.5 install-node.sh idempotent — OK

Second run safe : `git pull --ff-only`, `mkdir -p`, `id -u nexus`
guard avant `useradd`, `cargo build --release` idempotent,
`uv sync --frozen` idempotent, `systemctl enable --now` idempotent.

### A.6 Test harness isolation — OK

`DaemonHandle::spawn()` crée 2 `TempDir::new()` distincts par
daemon (NEXUS_GRID_ROOT + SBFB_HOME). Aucune collision possible
entre instances.

### A.7 Test harness cleanup — OK

`kill_on_drop(true)` sur le `Command` tokio + shutdown explicite
via SIGINT (Unix) / kill (Windows). `TempDir` drop nettoie les
répertoires. Pas de zombie processes ni de fuites.

---

## Track B — Cross-phase integration

### B.1 CORS flag documented in systemd — OK

`nexus-daemon.service` inclut un exemple override avec
`--cors-origin http://192.168.1.10:8080` en commentaire.

### B.2 Binary path consistency — OK

Le harness utilise `current_exe → parent → parent → push(binary)`
ce qui suit la convention standard Cargo : le test et le daemon
sont dans le même répertoire `target/{debug,release}/`. Le profil
est implicitement le même que celui du test runner. Le smoke
script a une logique fallback différente car c'est un script
standalone hors Cargo — contexte différent, pas un bug.

### B.3 Env isolation consistency — OK

Smoke script et harness utilisent les mêmes patterns d'isolation :
`NEXUS_GRID_ROOT` + `SBFB_HOME` en TempDir par daemon,
`RUST_LOG=warn`, args `start --headless`. Parfaitement cohérent.

---

## Track C — Security & hardening

### C.1 CORS origin validation schemes dangereux — P2

`is_valid_origin()` dans `http.rs` valide strictement les schémas
via `strip_prefix("http://")` / `strip_prefix("https://")`. Les
schemes `javascript:`, `data:`, `file:`, `blob:` sont correctement
rejetés par cette logique (pas de match prefix → retourne false).

**Cependant**, aucun test explicite ne documente le rejet de ces
schemes dangereux. Un refactoring futur pourrait affaiblir cette
protection sans casser de test. Recommandation : ajouter 3 tests
négatifs (`javascript:alert('xss')`, `data:text/html,...`,
`file:///etc/passwd`) dans les tests `is_valid_origin` de `http.rs`.

**Sévérité** : P2 — implémentation correcte, couverture test
insuffisante pour prévenir régression. Pas de risque runtime actuel.

### C.2 Auth token per-daemon-instance — OK

Le harness lit un token distinct par `SBFB_HOME` (TempDir unique
par daemon). `load_or_generate_token()` crée le token dans le
répertoire spécifié avec permissions 0o700/0o600. Pas de partage
entre instances.

### C.3 systemd User=nexus — OK

Les 3 services spécifient `User=nexus` / `Group=nexus`. Aucun
ne tourne en root.

---

## Track D — Meta-process

### D.1 G8 preflight 3/3 — OK

3 fichiers preflight présents dans archive/v1.2/ avec verdict
EXECUTE plan-as-is. Scans S1a/S1b/S2/S3/S4 documentés.

### D.2 Phase review 3/3 — OK

3 fichiers review présents avec verdict PASS :
- Phase A : 2 P2 + 1 P3
- Phase B : 2 P2 + 1 P3
- Phase C : 2 P2 + 1 P3

### D.3 Commit bodies structurés — OK

Les 3 feat commits contiennent delta tests, scope cuts, et
Co-Authored-By. Bodies riches conformes à §4.1.

### D.4 Pas de LOC estimates — OK

Grep `~[0-9]+ LOC` dans plan et kickoff : 0 match hors
description du mécanisme LOC guard lui-même.

### D.5 Carry counters corrects — OK

carry_summary.md counters cohérents avec les findings des
3 phase reviews. 3 items MANDATORY 3/3 correctement identifiés.

---

## Track E — MANDATORY carries S34

### E.1 P2-A-1 rand triple (3/3 MANDATORY) — confirmé, faisable

Cargo tree confirme la triple cohabitation :
- rand 0.8.6 (workspace direct via frost-ed25519)
- rand 0.9.4 (transitive intermédiaire)
- rand 0.10.1 (iroh 0.98 stack)
- 3 versions getrandom (0.2, 0.3, 0.4)

Impact runtime nul (sous-arbres disjoints). Unification faisable
si frost-ed25519 ou iroh convergent sur une version commune.
Action S34 : `cargo update --aggressive` + audit transitive,
consolider si < 2h sinon documenter la contrainte upstream.

### E.2 P2-B-1 tor-rtcompat (3/3 MANDATORY) — RÉSOLU

**Aucun code résiduel tor-rtcompat détecté** dans le workspace.
Grep `rtcompat|runtime.compat|tokio.*compat` = 0 matches.
`tor_transport.rs` utilise le runtime inference interne
d'arti-client (PreferredRuntime), pas de shim compat nécessaire
en Phase 1 Tor. Le carry était une anticipation de Phase 2
(long-lived TorClient handle) — pas un dette technique actuel.

**Recommandation** : fermer ce carry. Si Phase 2 Tor procède,
tor-rtcompat sera ajouté à ce moment comme item planifié.

### E.3 P2-REVIEW-C-2 COEP E2E (3/3 MANDATORY) — faisable

Headers COOP/COEP implémentés dans blob_serve.rs (constantes +
middleware). Test Playwright existe en mock-only. Le real E2E
nécessite : création zip avec index.html + publication via
blob-serve + assertion headers réels. Le test harness S33 peut
spawner des daemons réels mais ne publie pas encore de blobs.

Effort estimé : 2-4h si le harness expose blob-serve, sinon
refactoring plus conséquent. Action S34 : planifier comme phase
dédiée.

---

## Track F — Sprint 33 specifics

### F.1 Multi-daemon tests reliable — OK

Le harness utilise des timeouts explicites (30s deadline, 250ms
polling). Pas de sleeps hardcodés dans les tests, pas de patterns
race-prone. Timing défensif avec deadlines, pas de risque flaky
identifié.

### F.2 Smoke test portabilité — P3

`scripts/test-multi-node.sh` utilise des bash-isms (`[[`, arrays,
`(( ))`). Le shebang `#!/usr/bin/env bash` est correct. Acceptable
pour CI Linux (Ubuntu 20.04+ avec bash 4+). Non portable vers
POSIX sh strict, mais ce n'est pas le target.

### F.3 Phase review files 3/3 — OK

3 fichiers review présents et complets dans archive/v1.2/.

---

## Résumé findings

| # | Track | Finding | Sévérité | Action |
|---|---|---|---|---|
| 1 | C.1 | Tests manquants pour rejection schemes dangereux (javascript:/data:/file:) dans CORS daemon | P2 | Ajouter 3 tests négatifs `is_valid_origin` dans http.rs |
| 2 | F.2 | Bash-isms dans smoke test (non-POSIX) | P3 | Acceptable, aucune action requise |

## MANDATORY carries S34 — état vérifié

| Item | Compteur | État audit | Action S34 |
|---|---|---|---|
| P2-A-1 rand triple | 3/3 | Confirmé, faisable | Phase dédiée unification |
| P2-B-1 tor-rtcompat | 3/3 | **RÉSOLU** — aucun résidu | **Fermer** |
| P2-REVIEW-C-2 COEP E2E | 3/3 | Faisable, 2-4h | Phase dédiée real zip test |
