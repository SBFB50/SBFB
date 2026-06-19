# Sprint 76 Phase H Review

## Verdict: PASS

Review pilote profonde (driver-side, agent independant `nexus-phase-review-deep`,
1M tokens). Diff complet lu ligne par ligne (12 fichiers tracked + 4 untracked).
0 P0 / 0 P1. 1 P2 + 3 P3 (rigueur G4 satisfaite — la phase a des trade-offs
discutables documentes ci-dessous). Promu PASS apres le gate Codex (ci-dessous).

## Codex reconciliation

Codex GPT-5.5 (sortie brute `sprint76_phase_h_codex_review.md`) = **PARTIAL, not
reject** : 8 CONFIRMED (project-info additive+loopback ; injection project_id
non-spoofable ; submit daemon-level + task_id imbrique ; parite allowlist 16/16 ;
0 wire P2P ; 0 fuite secret ; SDK getTaskResult present + app button/poll ; scope
cuts honnetes) + 4 PARTIAL (aucun GAP/reject). Traitement :
- **PARTIAL #3 (4 templates Factory `sbfb-bridge.js` sans getTaskResult)** —
  **CORRIGE** : `getTaskResult` ajoute aux 4 templates (static/react/pyodide/
  static-reader) qui exposaient deja `submitTask` ; cohérence compute submit+lecture
  (le starter reste un sous-ensemble minimal qui pointe vers le full SDK). nextest
  sbfb-factory+manifest 175/175.
- **nit doc `lib.rs:191` « 15 methods »** — **CORRIGE** -> 16.
- **PARTIAL #1 (404 confond pending vs task introuvable)** — DOCUMENTE (intentionnel) :
  le task_id provient toujours d'un submit reel dans ce flux, donc « introuvable »
  ne se produit pas en pratique ; le timeout app 120s + message « worker a-t-il
  claime ? » borne le cas degenere. Distinguer exigerait un GET /tasks/{id} en plus.
- **PARTIAL #2 (TabView `ButtonBlock` appelle encore `submitAppTask` -> route morte)**
  — PRE-EXISTANT, HORS scope Phase H (chemin iframe). Surface legacy TabView, non
  introduite ici. Routee carry **TABVIEW-APP-SUBMIT-DEAD** S77.
- **PARTIAL #4 (parite allowlist declarative, pas d'autorisation par-app au dispatch)**
  — PRE-EXISTANT (design `sbfb-manifest` documente) : le bridge valide source +
  `BridgeRequestSchema` mais ne croise pas `SBFB.json bridge.methods` au dispatch.
  Inchange par cette phase ; note pour un durcissement futur.

Suites re-jouees apres les 2 corrections : sbfb-factory+manifest nextest 175/175,
clippy 0, fmt 0. Aucune correction Codex n'a touche le chemin compute teste LIVE.

## Scope And Staging

Phase = adaptation PLAN-ADAPT (cf. `sprint76_phase_h_preflight.md` verdict
PLAN-ADAPT) : cabler le pont compute iframe avant l'app, parce que STEP 0 a
prouve que le chemin etait mort sur les 3 segments.

Fichiers du commit de phase (atomique, coherent) :
- `crates/nexus-shell-daemon/src/http.rs` — route `GET /api/daemon/project-info` + test.
- `crates/sbfb-manifest/src/lib.rs` — `task_result` dans `BRIDGE_METHOD_ALLOWLIST` + test miroir.
- `web/src/api/coordinator.ts` — `getDaemonProjectInfo`, `submitComputeTask`, `getTaskResult`.
- `web/src/bridge/protocol.ts` — `task_result` dans `BridgeMethodSchema`.
- `web/src/bridge/useBridge.ts` — dispatch `task_submit` re-pointe + `task_result`.
- `web/src/bridge/__tests__/{useBridge,protocol}.test.ts` — 4 + 1 tests.
- 4 copies SDK `sbfb-bridge.js` (web/public + 3 examples).
- `examples/compute-tester/` (SBFB.json, index.html, app.js, sbfb-bridge.js).

Note staging (non bloquant) : `.planning/active/sprint76_verification.md`,
`sprint76_phase_h_preflight.md` et `phase_h_live_local.sh` sont des artefacts de
planning/outillage. `phase_h_live_local.sh` (untracked) est un helper d'acceptance
local — il ne devrait PAS entrer dans le commit de code de phase (split planning vs
code). Verification : `git diff --cached -- '*.rs' | rg '^\+pub mod '` = 0 nouveau
module. Pas de fichier accidentel, pas de cache/build.

## Three-Block Verification

Le rapporteur fournit (echantillon verifie, pas re-run integral par moi) :
Win fmt 0 / clippy workspace 0 / nextest 1805/1805 (+1) / doctests 0 ; web tsc 0 /
lint 0 err / Vitest 402/402 (+4) / coverage 87.2/79.01/85.92/88.52 / build / size /
scan FR clean. Docker canonique 1.94 fmt+nextest (touched crates) EN COURS.

Pas de bloc Python (pas de `packages/` dans ce repo Rust+Frontend pur depuis S50).
Le bloc release `cargo build -p nexus-shell-daemon --release` n'est pas cite
explicitement par le rapporteur — la route touche `http.rs` (compile dans le binary
release) → **a confirmer avant push** (process gate, pas review-blocker : le code
compile sous nextest + clippy `--all-targets`).

## Delta Tests

- Rust nextest +1 (1804→1805) = `project_info_field_present_and_null_without_doc`.
  Le test miroir allowlist `allowlist_mirrors_host_dispatch_schema` est MODIFIE (set
  etendu), pas neuf → 0 delta attendu, correct.
- Vitest +4 = les 4 tests `compute bridge Sprint 76 Phase H` (398→402). Le test B10
  `protocol.test.ts` est MODIFIE (15→16), pas neuf → cohérent.
- 5 copies SDK `sbfb-bridge.js` : pas de test JS dedie (parite par md5 ci-dessous),
  acceptable — ce sont des copies pures, pas de logique nouvelle a tester unitairement.

Tous les deltas s'expliquent. Aucun 0-delta suspect sur de la logique neuve.

## Modified-File Branch Coverage

Couverture semantique (Read des tests en entier, pas grep) :
- `task_submit` (useBridge) — branche succes : `task_submit injects the local
  project_id and returns the task id` appelle reellement le dispatch (MessageEvent),
  asserte `task_id === "t-1"` ET injection `submitBody.project_id === "doc-abc"` ET
  ordre des fetch (project-info en 1er). Branche erreur `!project_doc_id` :
  `task_submit errors when the node has no project doc` asserte `success:false` +
  message. **Les 2 cotes de la branche testes, inputs realistes, assertions
  specifiques.**
- `task_result` (useBridge) — branche 404→pending : `maps a 404 to a pending poll`
  asserte `{ready:false, status:"pending"}`. Branche 200→done : `surfaces the
  completed result text` asserte `ready:true` + `result_text:"pong"`. **2 cotes
  testes.**
- `project_info` (http.rs) — test couvre le cas `null` (pas de doc monte). Le cas
  `Some(id)` n'a pas de test unitaire Rust dedie (le harness `mk_state` ne monte pas
  de doc) MAIS est couvert LIVE (`§5.2` : `project_doc_id 6552cbdd…` reel) + cote
  bridge le mock `project_doc_id: "doc-abc"` exerce le chemin non-null. P3 ci-dessous.
- `getTaskResult` (coordinator.ts) — branche 500 (`!res.ok` non-404) → `ApiHttpError`
  et branche parse-fail → `ApiProtocolError` ne sont pas testees unitairement.
  Defensives, chemin principal (404/200) couvert → concern P3, pas bloquant.

## Security And Protocol

Surface touchee = pont SBFB + route loopback. Verifie :
1. **Tier d'auth** : `/api/daemon/project-info` est dans `authed_routes`
   (`http.rs:277`) = X-SBFB-Token + Host loopback + Origin. Meme tier que
   `/api/daemon/info`. Correct. Read-only `GET`.
2. **Secret ?** Non. `project_doc.id()` est deja partage au worker local via
   write-ticket (provisioning `local_worker.rs`). Le commentaire du handler le
   documente honnetement. Pas un secret cryptographique, pas une cle.
3. **Spoof project_id** : `{ ...req.payload, project_id: info.project_doc_id }`
   (`useBridge.ts:248-251`) — l'injection host VIENT APRES le spread, donc un
   `project_id` fourni par l'app est ECRASE. L'app ne peut pas cibler le worker d'un
   autre projet. **Invariant de securite tenu** (criterion 2 du brief : PASS).
4. **Pas de nouveau wire P2P** : route HTTP loopback additive. 0 bump `*_VERSION`
   (verifie : aucun `_VERSION` dans le diff). `TaskSubmission` inchange. Conforme
   pre-launch protocol.
5. **Payload mismatch resolu** : `TaskSubmission` (types.rs:72) requis =
   `project_id/task_type/prompt/model` ; `SubmitComputeTaskBodySchema` requiert
   `project_id/prompt/model` et defaut `task_type:"inference"` + `system_prompt:""`
   (lui aussi `#[serde(default)]` cote Rust). Tous les requis Rust toujours presents
   → 0 risque 422.
6. **404=pending ne masque pas d'erreur** : backend `get_task_result`
   (tasks_api.rs:160) renvoie 500 sur db-lock-poison, 404 sur "no result yet" ET
   "task not found". Le bridge mappe 404→pending et `!res.ok` non-404→`ApiHttpError`.
   Une vraie erreur (500) remonte, pas masquee. PASS. (Effet de bord : un task_id
   inexistant poll jusqu'au timeout 120s → P3.)
7. **Parite cross-langage** : `task_result` ajoute aux 2 cotes (Rust
   `BRIDGE_METHOD_ALLOWLIST` 16 + TS `BridgeMethodSchema` 16), 2 tests de parite
   verrouillent. La validation manifeste live a accepte un manifeste declarant
   `task_result` (§5.2 step 4) — preuve cross-langage reelle.
8. **Parite 5 copies SDK** : md5sum des 5 `sbfb-bridge.js` = IDENTIQUE
   (`a85ce625f6f0a8462429995c3f9a3d79`). Byte-identiques (criterion 3 : PASS).
9. **App iframe** : `index.html` utilise `<button type="button">` + click handler,
   PAS de `<form>` — conforme a la contrainte sandbox `allow-scripts` sans
   `allow-forms` (memory iframe_sandbox_forms). PASS.
10. Pas de `unsafe`, `unwrap()` prod, `panic!`, `todo!`, secret, path traversal dans
    le diff. `encodeURIComponent(taskId)` sur le chemin result → pas d'injection.

Aucune rouge-ligne DEEP declenchee (pas THREAT_MODEL/canonical/crypto/unsafe/zip).

## Research And G8

Preflight present et explicite : `sprint76_phase_h_preflight.md` verdict
**PLAN-ADAPT** avec STEP 0 trace code file:line (les 4 constats : route morte,
payload mismatch, aucun canal resultat, push impossible). Decision PO tracee :
option A (poll) maintenant, option B (SSE daemon adosse iroh-docs subscribe) S77.
iroh-docs subscribe LiveEvent confirme via Context7 (cite dans preflight). G8
satisfait — adaptation documentee AVANT le code, pas de derive.

## Scope Cuts

- **Option B (push SSE/iroh-docs)** : DIFFEREE S77, **non amorcee**. Verifie :
  `task_result_ready` n'existe QUE dans les tests (`useBridge.test.ts:628`,
  `protocol.test.ts:79`), JAMAIS emis en prod. Aucun SSE daemon pour tasks ajoute.
  Commentaires honnetes (protocol.ts:44-46, app.js:12-14). PASS.
- **Attribution compute par-app** : explicitement routee produit→S77 (preflight +
  verification §5.2). La tache locale s'attribue a `project_doc.id()` du noeud, pas a
  l'app — documente, assume.
- Pas de propagation de scope-cut comme verite : l'adaptation EST le re-examen
  code-first du gap "compute cable" suppose par le plan.

## Codex verification

NON FAIT (driver-side review). A executer : `codex exec` sur le diff. Security delta
a confirmer cote Codex : (a) injection project_id post-spread non-spoofable, (b) tier
auth loopback identique, (c) 0 bump wire. Aucune surface crypto/canonical touchee →
audit Codex attendu CONFIRM sur les 3 points.

## Commit Body Draft

```
feat(daemon): Sprint 76 Phase H — cablage pont compute iframe + Compute Tester

## Contexte
Le plan Phase H supposait la chaine app->bridge->coordinateur->worker->resultat
cablee. STEP 0 (preflight PLAN-ADAPT) prouve le contraire : route submit
app-scoped morte (supprimee S50), payload task_submit mismatch, aucun canal
resultat (push task_result_ready jamais emis en prod). Le worker local on-demand
ne claime que project_id == project_doc.id(), non expose par aucune route.
Adaptation : cabler le pont (1 route daemon read-only + bridge + SDK) avant l'app.

## Fichiers
- crates/nexus-shell-daemon/src/http.rs : route GET /api/daemon/project-info
  -> {project_doc_id} (auth loopback, read-only) + test.
- crates/sbfb-manifest/src/lib.rs : task_result -> BRIDGE_METHOD_ALLOWLIST (16) + test miroir.
- web/src/api/coordinator.ts : getDaemonProjectInfo + submitComputeTask
  (task_id imbrique) + getTaskResult (404=pending).
- web/src/bridge/protocol.ts : task_result -> BridgeMethodSchema (16).
- web/src/bridge/useBridge.ts : task_submit re-pointe daemon-level + injection
  project_id host + case task_result.
- web/{public,src} + examples/{sbfb-ideas,sbfb-explorer,sbfb-factory-viewer,
  compute-tester}/sbfb-bridge.js : getTaskResult (5 copies byte-identiques).
- examples/compute-tester/ : app de test (div+button, submit->poll).

## Delta tests
Rust nextest 1804->1805 (+1 project_info). Vitest 398->402 (+4 compute bridge).
Tests miroir allowlist (Rust + protocol.test.ts B10) etendus 15->16.

## Verification
Win fmt 0 / clippy workspace 0 / nextest 1805 / doctests 0 ; web tsc 0 / lint 0 /
Vitest 402 / coverage 87.2/79.01/85.92/88.52 / build / size / scan FR clean.
Docker canonique 1.94 + release daemon a confirmer avant push.
Acceptance LIVE LOCAL PASS (12s, llama3.1:8b, result_text reel, app deployee +
render path blob-serve OK) — sprint76_verification.md §5.2.

## Scope cuts
Option B (push SSE/iroh-docs subscribe) differee S77 (non amorcee : task_result_ready
tests-only). Attribution compute par-app = produit, S77. Poll-only maintenant.

## G8 traceability
Preflight PLAN-ADAPT (sprint76_phase_h_preflight.md) STEP 0 trace code file:line.
iroh-docs subscribe confirme Context7. Decision PO A/B tracee.

## Pre-launch protocol
0 bump *_VERSION (route HTTP loopback additive, pas wire P2P). TaskSubmission
inchange. project_doc_id non-secret (deja au worker via write-ticket).

## Codex verification
(a remplir : raw output codex exec)
Security delta : injection project_id post-spread non-spoofable ; tier auth
loopback identique a /api/daemon/info ; 0 nouvelle surface P2P.

## Carry closure
Aucun carry ferme. P3 routes verification.md (cas Some(id) Rust untested,
branches defensives getTaskResult, task inexistant poll-jusqu-timeout).
```

(Header `## Scope cuts` EXACT, pas de suffixe — validateur strict agentctl.)

## Findings

- **P2 — `phase_h_live_local.sh` hors atomicite du commit code.**
  `.planning/active/phase_h_live_local.sh` (untracked) est un helper d'acceptance.
  Il ne doit pas entrer dans le commit de code de phase. Le committer separement
  (planning) ou l'exclure. Non bloquant pour la review du code, signal d'hygiene
  staging.

- **P3 — `#[allow(dead_code)]` desormais stale sur `project_doc`.**
  `http.rs:154` — le champ `project_doc` est maintenant LU par `project_info`. Le
  `#[allow(dead_code)]` (commentaire "Read by future endpoints") est obsolete : le
  futur est arrive. Retirer l'attribut (ou actualiser le commentaire). Sur champ
  `pub`, l'allow est probablement deja un no-op, mais le commentaire ment.

- **P3 — cas `Some(project_doc_id)` non couvert par test unitaire Rust.**
  `project_info_field_present_and_null_without_doc` ne teste que la branche `null`
  (harness `mk_state` ne monte pas de doc). Le chemin non-null est couvert LIVE
  (§5.2) + cote bridge (mock "doc-abc"), donc pas bloquant, mais un test Rust avec
  doc monte fermerait la branche cote daemon.

- **P3 — `getTaskResult` : branches d'erreur (`!res.ok` non-404, parse-fail) non
  testees ; task_id inexistant poll jusqu'au timeout 120s.**
  `coordinator.ts` getTaskResult — les chemins `ApiHttpError`/`ApiProtocolError` sont
  defensifs et non couverts. Par ailleurs un task_id inexistant renvoie 404 (=pending)
  cote backend → l'app poll 120s avant timeout au lieu d'echouer vite. Acceptable
  (timeout existe, UX message clair) ; a noter pour S77.

## Residual Risk

Faible. L'adaptation est additive, sans bump wire, sans surface crypto. L'invariant
de securite cle (project_id injecte par le host, non-spoofable par l'app) est tenu et
teste. Acceptance LIVE LOCAL PASS prouve la chaine HTTP bout-en-bout ; honnetement le
clic in-browser literal n'est pas drive (meme code bridge teste unitairement, meme
render path blob-serve que 4 apps deployees). Le push (option B) est correctement
differe S77 et non amorce. Risque residuel = la branche `Some(doc_id)` daemon repose
sur le live + le mock bridge plutot qu'un test Rust dedie (P3). Aucun blocage P0/P1.

A confirmer avant push (process, pas review) : Docker canonique 1.94 (en cours) +
`cargo build -p nexus-shell-daemon --release` + Codex gate.
