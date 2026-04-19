---
sprint: 21
phase: A
date: 2026-04-18
head: f5ad2e1
verdict: DESIGN-CONFLICT
gate: G8 (nexus-phase-preflight)
scan_trigger: S1 SOTA delta (tower-governor 0.8 / axum 0.7 version clash)
arbitrage_required: user A/B/C
arbitrated_option: C (deep-evolution — axum 0.7 → 0.8 workspace bump + tower-governor 0.8 natif)
arbitrated_by: user (FlowUP)
arbitrated_at: 2026-04-18
post_audit_scope_note: axum usage audit revisited — scope réel ~5 sites (3 routes path syntax + Cargo.toml + middleware signature compile-driven), tower/tower-http/hyper workspace déjà compat 0.8, pas d'Option<Extractor> / PathParamsRejection / hyper::Body / axum::async_trait. Risque S16 régression réduit versus estimation initiale proposal §3 Option C.
post_bump_drift_R1: après commit chore axum bump `5e67ce0`, pre-code Phase A grep a révélé un 2e drift plan §4.1 `/task/submit` n'existe pas côté Rust shell-daemon (endpoint vit `packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py` FastAPI depuis Sprint 4 Phase A). `tower-governor` axum ne peut pas middleware FastAPI. User arbitre 2026-04-19 **R1 worker-engine gate pure Rust** (drop HTTP middleware, primitive worker-core native consommée par engine pre-task-execution). Scope-cut : tests §4.2 row 7-8 différés S22+ sprint API sécurité Python dédié. D1 core `governor 0.10.2 GCRA` préservé. 10 tests Rust Phase A au lieu de 15 initial. Chore(planning) inline avant feat commit Phase A.
---

# Sprint 21 Phase A — pivot proposal G8

**Verdict** : DESIGN-CONFLICT. STOP code, attendre arbitrage utilisateur.

## 1. Le conflit

Plan §Phase A §4.1 propose :

> `Cargo.toml` workspace (modifié) : deps `governor = "0.10.2"`,
> `tower-governor = "0.8"`, `notify = "*"` (déjà workspace),
> `toml = "*"` (déjà).

Kickoff §D1 cite :

> Retenu : `governor 0.10.2` (GCRA) + `tower-governor 0.8` axum
> middleware (compat axum 0.8).

Conflit factuel : **`tower-governor 0.8` exige `axum = "0.8"`**
(pinning strict dans son `Cargo.toml`), mais le workspace Nexus
pinne **`axum = "0.7"`** (ligne 139 `Cargo.toml` racine) utilisé
par `crates/nexus-shell-daemon` (HTTP loopback S16) + `crates/
nexus-launcher` (auth middleware S16).

Plan-as-is ne compile pas : clash transitif axum 0.7 (workspace) vs
axum 0.8 (dep tower-governor 0.8). Soit on bumpe axum workspace-
wide (MAJOR breaking, rework S16 loopback primitives), soit on
change de crate middleware, soit on abandonne la glue tower-
governor.

## 2. Evidence factuelle

Toutes sources primaires, vérifiées 2026-04-18 session fraîche
pré-code.

**E1 — `tower-governor 0.8.0` Cargo.toml main branch** (GitHub
`benwis/tower-governor`, WebFetch 2026-04-18) :

```toml
[package]
version = "0.8.0"

[dependencies]
axum = { version = "0.8", optional = true }
governor = "0.10.0"
```

Confirmé aussi par WebSearch crates.io : "tower-governor 0.8.0
specifies `axum = { version = '0.8', optional = true }`, which
means tower-governor 0.8 is compatible with axum 0.8, not axum
0.7". `optional = true` signifie feature-gated (tonic/hyper-only
possible), pas multi-version axum : quand feature axum enabled,
le pin est strict 0.8.

**E2 — Workspace Nexus Cargo.toml ligne 139** (état actuel
`f5ad2e1`, `grep -nE "^axum" Cargo.toml`) :

```toml
axum = "0.7"
```

Utilisé par `crates/nexus-shell-daemon/Cargo.toml:36` et
`crates/nexus-launcher/Cargo.toml:25` via `axum = { workspace =
true }`.

**E3 — Portée axum dans le code (grep `use axum` sur `crates/`)** :

- `crates/nexus-shell-daemon/src/http.rs` : ~1040 LOC, Router +
  middleware from_fn + HeaderName from_static + extract::Request +
  body::Body — S16 Phase A bearer token + Host + Origin gate
  (audit gate S17 leveraged, S19 audit PASS).
- `crates/nexus-launcher/src/auth.rs` : ~230 LOC, Router +
  middleware::from_fn + middleware::Next + response::Response —
  S16 Phase A launcher loopback auth path.

**E4 — axum 0.7 → 0.8 breaking changes** (WebSearch 2026-04-18,
tokio.rs/blog/2025-01-01-announcing-axum-0-8-0 + CHANGELOG) :

1. Path syntax `/:id` + `/*many` → `/{id}` + `/{*many}`. Toutes les
   routes shell-daemon + launcher à réécrire.
2. `axum::async_trait` re-export supprimé (remplacer par
   `async_trait::async_trait`).
3. `Option<T>` extractor nécessite nouveau trait
   `OptionalFromRequestParts`.
4. `Handler<B, T>` type params swappés → `Handler<T, B = Body>`.
5. `IntoResponse` : Body et BodyError associated types supprimés.
6. `Router::merge`/`Router::nest` panic si fallback double
   (previously silently discarded).
7. `PathParamsRejection` renommé `PathRejection`, variants
   renommés.
8. hyper::Body re-export supprimé.

Impact sur S16 loopback primitives audited : le rework réécrit
tout `middleware::from_fn(host_origin_gate)` path + `Router::new()
.route()` syntax + `into_make_service_with_connect_info` glue +
`axum::extract::Request` → possibles régressions subtiles sur
bearer token gate + Host + Origin check.

**E5 — `governor 0.10.x` API stable** (context7 `/boinkor-net/
governor` queried 2026-04-18) :

`DefaultKeyedRateLimiter<K>`, `RateLimiter::keyed(Quota::per_
second(...))`, `check_key(&key)` → `Result<(), NotUntil>`,
`retain_recent()`, `shrink_to_fit()`, `len()`, `until_key_ready`
async — tous présents et documentés. `governor 0.10.2` (plan
version) est semver-compat avec `governor 0.10.0` (utilisé par
tower-governor 0.8). Backend pur natif sans dep axum.

**E6 — RUSTSEC / CVE 2026 sur governor** (WebSearch 2026-04-18) :
aucun advisory publié. ✓

**E7 — tower-governor 0.7.x axum compat** : non confirmé
factuellement (WebFetch `v0.7.0` tag GitHub → 404, crates.io
versions endpoint vide WebFetch). lib.rs mentionne "17 releases
(8 breaking)". Inférence : tower-governor 0.8 released Aug 14
2025 sync avec axum 0.8 Jan 2025 → pre-0.8 (0.6, 0.7) probablement
axum 0.6/0.7 mais non verified. Option B ci-dessous requiert
vérification pre-Phase-A.

## 3. Options

### Option A — Scope-cut conforme (governor direct + custom middleware)

**Description** : Drop `tower-governor` du plan entièrement. Phase
A livre `crates/nexus-worker-core/src/rate_limit.rs` avec
`RateLimiter` wrapping `governor::DefaultKeyedRateLimiter<(Consumer
Id, WorkerId, ModelId)>` natif + un adapteur middleware custom
axum 0.7 ~30-50 LOC dans `crates/nexus-shell-daemon/src/http.rs`
(petit `tower::Service` ou `middleware::from_fn` qui extrait la
tuple clé du body Task et appelle `limiter.check_key()`).

**Coût** :
- +30-50 LOC middleware custom (au lieu d'utiliser tower-governor).
- Aucun bump axum, aucun rework shell-daemon/launcher S16.
- Aucun changement workspace Cargo.toml pour axum.
- Tests +15 attendus plan §4.2 restent applicables 1:1.
- Delta plan §4.1 : retirer ligne `tower-governor = "0.8"` + ajouter
  commentaire inline « custom axum 0.7 middleware, tower-governor
  droppée car requiert axum 0.8 ».

**Bénéfice** :
- Core D1 préservé : `governor 0.10.2` GCRA est la decision Day 0
  racine, tower-governor est citée comme moyen middleware. Option
  A garde le core, change le moyen.
- Zéro risque régression S16 loopback primitives (audited
  multi-sprint).
- Phase A lande en un commit atomique comme prévu.
- Pattern custom-wrapper cohérent avec S20 Phase C `pow_policy_
  loader.rs` + S18 D-1 `TokenRotator` qui utilisent aussi des
  adapteurs custom autour de primitives (notify file-watcher)
  plutôt que des libs tierces "clé en main".

**Invariants preserves** :
- Wire format : OK (aucun touché).
- Threat model : OK (C-ModelExtract + C-DosFlood toujours
  mitigés par rate-limit per-tuple).
- Day 0 : OK (D1 core governor 0.10.2 GCRA + DashMap keyed +
  policy hot-reload tous livrés).
- Pre-launch protocol : OK.
- S16 loopback audited primitives : **PRESERVÉES** (pas de
  touche).
- Cap G7 carry-overs : OK (2/2 inchangé).
- Test budget : OK (+15 tests cible plan §4.2 inchangé).

**Recommandation** : **default**.

### Option B — Adapt minimal (tower-governor 0.7.x compat axum 0.7)

**Description** : Remplacer `tower-governor = "0.8"` par la
dernière version de tower-governor compatible axum 0.7 (0.7.x,
0.6.x ou antérieur). Garder le reste du plan as-is.

**Coût** :
- Vérification pre-code obligatoire : quelle version exacte
  tower-governor marche avec axum 0.7 ? API stable (GovernorLayer
  + GovernorConfigBuilder) identique à 0.8 ?
- Si tower-governor 0.7.x a moins de features (ex: pas de
  `SmartIpKeyExtractor` ou pas de `key_extractor` custom), re-
  adjust.
- Delta plan §4.1 : changer `tower-governor = "0.8"` → `"0.7"`
  (ou autre version).

**Bénéfice** :
- Réutilise la glue tower-governor existante.
- Moins de LOC custom vs Option A.

**Invariants preserves** :
- Wire format : OK.
- Threat model : OK.
- Day 0 : OK (D1 inchangé littéralement, juste version tower-
  governor ajustée).
- S16 loopback primitives : PRÉSERVÉES.
- Pre-launch protocol : OK.

**Risques** :
- tower-governor 0.7.x n'existe peut-être pas dans une forme
  compat axum 0.7 stable + maintenue. Si la version requise est
  0.5 ou plus ancienne (axum 0.6), on prend un crate obsolète +
  écart de features vs 0.8.
- tower-governor n'est pas audité public, pas plus 0.7 que 0.8.
  Écart maintenabilité.
- lib.rs flags "17 releases (8 breaking)" = instabilité API
  historique.

**Recommandation** : alternative si Option A jugée trop verbose.

### Option C — Deep-evolution (bump axum workspace 0.7 → 0.8)

**Description** : Commit chore pre-Phase-A `chore(sprint21):
bump axum 0.7 → 0.8 workspace-wide` qui réécrit :
- `crates/nexus-shell-daemon/src/http.rs` : toutes les routes
  (syntax `/{id}`), `middleware::from_fn` + `Router::new()` +
  `extract::Request` + bearer/Host/Origin gate en pattern axum
  0.8.
- `crates/nexus-launcher/src/auth.rs` : idem pour loopback
  launcher.
- Tous les tests axum (multi-crates) qui touchent Router /
  middleware.
- Dépendances transitives tower-http / hyper / etc. (tower 0.5.1+
  requis axum 0.8).
Ensuite Phase A normale avec `tower-governor = "0.8"` natif.

**Coût** :
- chore pre-Phase-A : ~300-500 LOC réécrits répartis 2 crates
  critiques.
- Tests existants shell-daemon + launcher à re-valider (suite
  S16 loopback + multi-transport).
- Possibles effets transversaux tower 0.5 / hyper 1.0 upgrade si
  pas déjà.
- Timeline : chore pre-Phase-A = 1 commit séparé, Phase A = 1
  commit. Total 2 commits pour livrer rate-limit.

**Bénéfice** :
- Utilise tower-governor 0.8 moderne + maintenu.
- Aligne workspace sur axum 0.8 (récent, avec SSE + stream
  improvements + Option extractor nouveau trait).
- Long-term : débloque futures deps qui pinnent axum 0.8.

**Invariants preserves** :
- Wire format : OK.
- Threat model : OK **si** le rework S16 loopback est fait sans
  régression (charge de preuve sur l'executeur : tests
  `loopback_*_test.rs` tous verts + re-run audit gate S16
  checklists sur shell-daemon/launcher).
- Day 0 : OK.
- Pre-launch protocol : OK.

**Risques (majeurs)** :
- **Régression S16** : middleware::from_fn signature change entre
  axum 0.7 et 0.8 (trait `OptionalFromRequestParts` nouveau). Le
  bearer + Host + Origin gate actuel (`crates/nexus-shell-daemon/
  src/http.rs:913-1033`) utilise ce pattern. Rework = risque
  régression silencieuse (ex: ordre des middlewares, fallback
  Router::nest panic, etc.).
- **Scope drift Phase A** : plan original +15 Rust tests → chore
  axum bump est ~0 tests nouveaux, +beaucoup LOC modifiés hors
  scope D1 rate-limit.
- **Test budget cap garde-fou 4** : plan §4.2 estimait +15 tests
  pour Phase A. Option C ajoute un chore préalable hors-phase
  qui ne compte pas formellement mais consomme du temps sprint.
  Cap risque dépassement.
- **Breaking change transverse** : dépendances downstream possibles
  (ex: `tower-http` pour CORS / compression) peuvent aussi
  casser.

**Recommandation** : **à éviter Phase A**. Meilleur fit sprint
dédié ops/maintenance ou bundling avec un autre chore axum-
ecosystem.

## 4. Recommandation default

**Option A — Scope-cut conforme (governor direct + custom
middleware axum 0.7)**.

Rationale technique chiffrée :
- +30-50 LOC custom middleware pur wrapping `limiter.check_key
  (tuple)` — petit scope, isolé dans Phase A, zéro transversal.
- Zéro risque régression S16 audited primitives (gate S17+S19
  PASS) : aucune touche au path bearer + Host + Origin + UDS peer
  creds.
- D1 core `governor 0.10.2 GCRA + DashMap keyed + policy hot-
  reload + burst GCRA multiplier` **entièrement livré natif**
  (context7 confirme API `DefaultKeyedRateLimiter<K>` + `check_
  key` + `retain_recent` + quota builder).
- Pattern cohérent projet : S18 D-1 `TokenRotator` + S20 Phase C
  `pow_policy_loader.rs` utilisent tous des adapteurs custom
  `notify` + file-watcher + hot-reload sans lib middleware glue
  tierce.
- Tests cible plan §4.2 inchangés : saturation, per-tuple
  independence, eviction, hot-reload malformed + deletion guard,
  429 retour + Retry-After header, middleware ordre avant PoW
  gate — tous ré-applicables avec custom middleware.
- Plan delta minimal : supprimer ligne `tower-governor = "0.8"`
  du §4.1 + ajouter note inline custom middleware rationale.

## 5. Garde-fous (cf. README §6.9)

- [x] Pivot evidence-based (>=1 source externe) ✅ — E1 à E7 ci-
  dessus, 7 sources primaires (context7, WebFetch GitHub Cargo.toml,
  WebSearch axum announcement, grep workspace).
- [x] Pivot ne rebat pas Day 0 sans escalation ✅ — D1 core
  `governor 0.10.2` GCRA préservé (tower-governor est cité comme
  moyen middleware, pas core). Option A = ajustement du moyen,
  pas du core. Option C touche workspace axum → escalation user
  sur coût et risque S16.
- [x] Pivot ne casse pas pre-launch wire ✅ — aucun `*_VERSION`
  touché dans toutes options.
- [x] Test budget cap respecté (<= 2.5x plan) ✅ — Option A +15
  tests inchangé. Option B idem. Option C chore pre-Phase-A
  hors-budget tests strict.
- [x] Pivot dans theme sprint (kickoff §1 rate-limit multi-tier) ✅ —
  toutes options livrent rate-limit per-(consumer, worker, model).
- [x] Pivot ferme gap clair (pas YAGNI) ✅ — Phase A originale
  HARDENING_ROADMAP §3 S21 C-ModelExtract + C-DosFlood.
- [x] Pivot retrospective trackée dans audit_plan S21 ✅ — à
  ajouter Phase F `sprint21_audit_plan.md §meta-track G8
  traceability Phase A pivot Option X`.

**Aucun garde-fou échoué**. Le pivot est valide.

## 6. Suite

Arbitrage user requis : A, B, ou C ?

**Si user choisit A** (default recommandé) :
1. Commit chore inline `chore(planning): sprint21 Phase A — pivot
  G8 Option A adopt governor direct + custom axum 0.7 middleware
  (tower-governor 0.8 requires axum 0.8 incompat workspace)` qui
  update `sprint21_plan.md §4.1` (retirer `tower-governor = "0.8"`
  + ajouter note rationale) + archive ce pivot_proposal.md vers
  `archive/v1.2/sprint21_phase_A_pivot_proposal.md` (§6.11 pattern
  research outputs).
2. Implémenter Phase A normalement avec custom middleware ~30-50
  LOC.
3. Commit feat `feat(sprint21): Phase A — rate-limit sliding-
  window multi-tier per-(consumer, worker, model) via governor
  GCRA + custom axum 0.7 middleware` avec body riche + working
  tree audit G5 + mention pivot G8 Option A verdict.
4. Phase F audit_plan S22 trackera le pivot en dimension G8
  traceability.

**Si user choisit B** :
1. D'abord vérification factuelle : quelle version tower-governor
  compat axum 0.7 existe + API similaire (GovernorLayer). Si
  confirmation → commit chore inline update plan + procéder.
  Sinon → fall-back Option A.

**Si user choisit C** :
1. Commit chore pre-Phase-A `chore(sprint21): bump axum 0.7 →
  0.8 workspace-wide` avec tests verts complets sur `nexus-shell-
  daemon` + `nexus-launcher` (re-run suites §7.4).
2. Puis Phase A normale avec tower-governor 0.8.
3. **Attention** : risque régression S16 loopback. Re-review
  manuel path bearer/Host/Origin gate obligatoire + tests S16 à
  re-valider un-par-un.

**Si user propose Option D (rejeu contraint)** :
1. Agent construit Option D evidence-grounded depuis feedback
  user.
2. Emit `sprint21_phase_A_pivot_proposal.v2.md` avec Option D +
  A/B/C conservés.
3. Max 1 rejeu : si user rejette v2, fallback Option A par
  défaut + log carry-over explicite S22 audit_plan.
