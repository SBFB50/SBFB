Périmètre respecté : working tree courant uniquement. Tests non exécutés par moi.

### Livrable 1 : B1 duress frères LOCAL-ONLY
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:1543`, `crates/nexus-shell-daemon/src/http.rs:2097`, `crates/nexus-shell-daemon/src/noop_identity.rs:83`, `crates/nexus-shell-daemon/src/http.rs:5986`, `crates/nexus-shell-daemon/src/http.rs:6029`
- Evidence :
```rust
1543: if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
1544:     == crate::noop_identity::PublishOutcome::Noop
1546:     return (StatusCode::OK,
1548:         Json(serde_json::json!({"ok": true, "enabled": req.enabled})))
```
`seed_voluntary` a le même early-return avant résolution/fetch/pin/DB/emit à `http.rs:2097-2104`, et le succès normal renvoie le même JSON à `http.rs:2282-2285`. Les tests vérifient zéro row `keep_online` et zéro tag blob à `http.rs:6016-6024` et `http.rs:6056-6064`.

### Livrable 2 : B1 observed availability-only
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon-core/src/iroh_runtime.rs:2352`
- Evidence :
```rust
2354: // registry is AVAILABILITY-ONLY, not publisher-authenticated.
2358: // verified envelope author is not plumbed to this layer
2371: assert!(runtime.record_observed_directory(observed_pk(101), t0));
2372: assert!(runtime.record_observed_directory(observed_pk(202), t0));
2374: runtime.observed_count(),
```
Le diff de ce fichier ajoute seulement ce test. Il ne rajoute pas de code prod de binding PoW, et la doc du test ne sur-promet pas ce binding.

### Livrable 3 : B2 CARRY-3 downgrade ingress `is_open_source`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:2299`, `crates/nexus-shell-daemon/src/runtime.rs:2310`, `crates/nexus-shell-daemon/src/runtime.rs:2335`, `crates/nexus-shell-daemon/src/http.rs:2773`, `crates/nexus-shell-daemon/src/runtime.rs:2686`
- Evidence :
```rust
2299: let is_open_source = crate::http::trustworthy_open_source(
2300:     ann.is_open_source,
2301:     ann.provenance_hash.as_deref(),
2302:     ann.repo_url.as_deref(),
2327:     is_open_source,
```
Le downgrade est appliqué avant `browse_aggregator.add_direct_entry(entry)` à `runtime.rs:2335`, donc bien à l’ingress `/browse`. Le helper reste un bool local : `http.rs:2773-2779`.

### Livrable 4 : B3 PULL-3 failover cross-tier
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:2143`, `crates/nexus-shell-daemon/src/http.rs:2186`, `crates/nexus-shell-daemon/src/http.rs:2218`, `crates/nexus-shell-daemon/src/http.rs:2315`, `crates/nexus-shell-daemon/src/http.rs:6069`
- Evidence :
```rust
2186: let chain = build_seed_fetch_chain(direct_plan, directory_plan);
2224: for (hash_hex, plan) in chain {
2294: Err(e) => {
2295:     debug!(error = %e, "voluntary seed: tier fetch failed (trying next tier if any)");
2296:     last_error = (StatusCode::BAD_GATEWAY, "could not fetch the app archive");
```
Ce n’est pas une simple sélection : le handler boucle sur la chaîne et ne retourne pas sur échec du ticket. Le mismatch supprime le tag avant le tier suivant à `http.rs:2288-2292`. Les codes locaux 400/404/502 sans tier sont préservés à `http.rs:2187-2215`. Le test `pull_falls_back_across_tiers_when_ticket_dead` vérifie l’ordre ticket puis directory à `http.rs:6076-6091`.

### Livrable 5 : B4 T6-OUTBOX-DIRECT
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:1800`, `crates/nexus-shell-daemon/src/runtime.rs:1807`, `crates/nexus-shell-daemon/src/runtime.rs:2911`
- Evidence :
```rust
1800: Some(GossipCmd::Outbox(payload)) => {
1807:     if let Ok(guard) = coordinator_db.lock() {
1808:         if let Err(e) = guard.insert_outbox(&payload) {
1836:     outbox.push(payload);
```
Le test démarre la vraie tâche gossip 1 nœud et assert la persistance DB de l’annonce non wrappée à `runtime.rs:2930-2949` puis `runtime.rs:2983-2990`.

### Livrable 6 : B5 hoisting `my_endpoint_addr()`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:1678`, `crates/nexus-shell-daemon/src/runtime.rs:1757`, `crates/nexus-shell-daemon/src/runtime.rs:1818`, `crates/nexus-shell-daemon/src/runtime.rs:1866`, `crates/nexus-shell-daemon/src/runtime.rs:2056`, `crates/nexus-shell-daemon/src/runtime.rs:2074`, `crates/nexus-shell-daemon/src/runtime.rs:2194`
- Evidence :
```rust
1678: if let Some(addr) = current_replay_addr(&node).await {
1683:     let Some(fresh) = remint_and_wrap_for_replay(
1689:         &addr,
1690:         stored,
```
Les quatre sites passent une adresse préfetchée : browse request, NeighborUp, Outbox cmd, periodic republish. Si l’adresse échoue, le `if let Some(addr)` saute toute la passe, donc zéro broadcast comme avant. Le test prouve que l’adresse passée est utilisée, pas refetchée : `runtime.rs:3302-3304` et `runtime.rs:3324-3331`.

### Livrable 7 : B6 discriminateur curateur/ancre `/nodes`
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/Nodes.tsx:170`, `web/src/pages/Nodes.tsx:221`, `web/src/pages/Nodes.tsx:362`, `web/src/pages/Nodes.tsx:408`, `web/src/pages/__tests__/Nodes.test.tsx:143`
- Evidence :
```tsx
170: const curatingHexes = new Set(
172:   ? curatorsResult.body.entries.map((e) => bytesToHex(e.curator_pubkey))
221: <WaitingRow
224:   isCurator={curatingHexes.has(hex)}
373: data-kind={isCurator ? "curator" : "anchor"}
```
Le test B6 crée un abonné présent dans `entries` et un absent, puis assert `data-kind="curator"` et `data-kind="anchor"` à `Nodes.test.tsx:173-187`.

### Livrable 8 : B7 LOOPBACK-TIERS
- Statut : CONFIRME
- Fichier(s) : `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:82`, `.planning/active/sprint76_audit_plan.md:380`
- Evidence :
```md
82: | `POST /api/daemon/keep-online` ...
83: | `POST /api/daemon/seed` ...
84: | `GET /api/daemon/seed-count/{project_id}` ...
85: | `GET /api/daemon/nodes` ...
86: | `POST /api/daemon/directory/publish` ...
```
Les 7 routes attendues sont inscrites à `LOOPBACK...:82-88`. La phrase fausse du plan est corrigée explicitement à `.planning/active/sprint76_audit_plan.md:380-386`.

### Livrable 9 : B8 THREAT-BLOBSERVE-BEARER
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:876`, `crates/nexus-shell-daemon/src/http.rs:248`, `crates/nexus-shell-daemon/src/http.rs:489`
- Evidence :
```rust
248: let blob_serve_routes = Router::new()
249:     .route("/{hash}/{*path}", get(blob_serve))
252: // Public routes: no bearer, no Host check, no Origin check.
253: let public_routes = Router::new()
255:     .nest("/blob-serve", blob_serve_routes);
```
La doc §15.1 dit bien que `/blob-serve` est publique par construction et que le bornage vient de subscribed-only + cap + timeout, pas d’un bearer (`THREAT_MODEL.md:876`). Le routeur merge `public_routes` séparément avant les routes authentifiées à `http.rs:489-492`.

### Livrable 10 : B10 BRIDGE-ALLOWLIST-DRIFT
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-manifest/src/lib.rs:54`, `crates/sbfb-manifest/src/lib.rs:67`, `crates/sbfb-manifest/src/lib.rs:186`, `web/src/bridge/protocol.ts:20`, `web/src/bridge/__tests__/protocol.test.ts:142`
- Evidence :
```rust
61: /// this list MUST mirror the host dispatch schema `BridgeMethodSchema`
63: /// (15 methods). Pre-B10 it carried only 10
67: const BRIDGE_METHOD_ALLOWLIST: &[&str] = &[
72:     "pii_redact",
80:     "storage_version",
```
Les 15 méthodes sont présentes côté TS à `protocol.ts:20-44`. Le test Rust vérifie le miroir et les 5 méthodes ajoutées à `lib.rs:196-240`; le test TS verrouille aussi les 15 méthodes à `protocol.test.ts:167-177`. La doc distingue bien manifest déclaratif et dispatch/sandbox à `lib.rs:54-59`.

### Livrable 11 : B9 FRONTEND-COVERAGE-GAP + CI-PLAYWRIGHT-NOOP
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/__tests__/Curators.test.tsx:53`, `web/src/pages/__tests__/OnboardingEmpty.test.tsx:28`, `web/src/pages/__tests__/ProjectDetail.test.tsx:51`, `web/src/pages/__tests__/Projects.test.tsx:49`, `.github/workflows/ci.yml:81`, `web/package.json:8`, `web/vitest.config.ts:15`
- Evidence :
```tsx
57: expect(screen.getByText("Aucun noeud actif")).toBeInTheDocument();
31: expect(screen.getByText("Bienvenue sur nexus-grid")).toBeInTheDocument();
55: expect(screen.getByText("Projet introuvable")).toBeInTheDocument();
62: expect(screen.getByText("Mes projets")).toBeInTheDocument();
```
Les 4 smoke tests ont des assertions utiles. L’étape Playwright CI est retirée comme vacuous à `.github/workflows/ci.yml:81-86`; `test:e2e` n’est plus dans les scripts `package.json:8-13`. Inventaire fichier : aucun `playwright.config.*` ni `*.spec.*` sous `web/`. `vitest.config.ts:15-25` documente que ces pages restent hors `coverage.include`.

### Livrable 12 : B11 UX-ARRIVAL track couvert
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint76_audit_plan.md:431`, `crates/nexus-shell-daemon/src/http.rs:904`, `crates/nexus-shell-daemon/src/http.rs:1959`, `crates/nexus-shell-daemon/src/http.rs:4366`
- Evidence :
```md
431: | **Surface UX-ARRIVAL (B11, COUVERTE S76-B)** ...
431: **Couverte** : observed borne+rate-limite ... `from_subscribed` CATALOG-BACKED ...
431: discriminateur curateur/ancre (B6, `Nodes.test.tsx`).
```
Le claim est recoupé par le code : `from_subscribed` est catalog-backed à `http.rs:904-921`, `/nodes` expose `observed` à `http.rs:1959-1965`, et le test anti-spoof `browse_views_derives_from_subscribed` est à `http.rs:4366-4468`.

### Invariant pre-launch
- Statut : CONFIRME
- Evidence : `git diff --name-only` ne contient aucun des fichiers wire/canonical listés (`canonical.rs`, `seed.rs`, `task.rs`, `node_directory.rs`, `curator.rs`). Le grep du diff sur `FORMAT_VERSION` ne retourne aucun changement de version. Les changements constatés restent dans `http.rs`, `runtime.rs`, tests, UI, docs, CI et manifest allowlist.

## Resume final
- Total livrables : 12
- Confirmes : 12
- Gaps : 0
- Partiels : 0
