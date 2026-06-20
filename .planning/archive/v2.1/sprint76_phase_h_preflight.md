# Sprint 76 — Phase H préflight (compute bridge wiring + Compute Tester)

## Verdict : PLAN-ADAPT

Le plan Phase H suppose que la chaîne app→bridge→coordinateur→worker→résultat→app
existe et qu'il suffit de construire l'app. **STEP 0 (trace code) prouve le contraire** :
le chemin compute depuis une iframe sandboxée n'est câblé sur **aucun** des 3 segments,
et le claim local exige une donnée non exposée. L'app seule ne peut donc rien afficher.
Adaptation = câbler le pont (1 route daemon read-only + bridge + SDK) AVANT l'app.

## STEP 0 — constats (file:line)

1. **Route submit app-scoped morte.** `submitTask()` → `task_submit` →
   `useBridge.ts:236` `submitAppTask()` → `POST /app/{name}/tasks/submit`. Le daemon Rust
   n'a que `/app/{name}/state*` (`http.rs:358-364`) — pas de route tasks app-scoped (supprimée
   avec le coordinator Python au pivot S50). Seul existe le daemon-level
   `POST /api/v1/tasks/submit` → `coordinator_submit_task` (`http.rs:3306`), chemin prouvé
   par l'E2E in-process B-3 (`runtime.rs:4055-4102`).
2. **Payload mismatch.** Le host parse `task_submit` avec `SubmitAppTaskBodySchema`
   (`coordinator.ts:593`, exige `worker`). Le SDK envoie `{prompt,model,task_type}` → parse Zod
   échoue avant tout HTTP. Aucune app n'appelle réellement `submitTask` (grep examples : que des
   commentaires SDK + un `<code>` doc `sbfb-explorer/index.html:198`) → re-pointer ne casse rien.
3. **Aucun canal résultat.** Pas de méthode bridge `task_result` (switch `dispatch`
   `useBridge.ts:233-387`, enum `protocol.ts:20-44`). L'event push `task_result_ready` n'est
   **jamais émis en prod** (`pushEvent("task_result_ready")` uniquement dans le test
   `useBridge.test.ts:474`). Back-end pourtant prêt : `GET /api/v1/tasks/{id}/result`
   (`tasks_api.rs:160`, 404 pending → `{result_text,status,result_hash}` sinon).
4. **Push end-to-end impossible sans nouvelle infra.** iroh-docs `subscribe` (LiveEvent
   LocalInsert/RemoteInsert, confirmé Context7) vit sous la frontière HTTP (niveau daemon Rust),
   n'atteint pas le navigateur. Aucun SSE daemon pour les tâches (seul SSE du dépôt =
   `sbfb-factory/operator_server.rs`, outil local distinct). → décision PO : **A (poll) maintenant,
   B (SSE daemon adossé iroh-docs subscribe) planifié S77** avec la convergence WAN.

## Le point dur — project_id du claim local

- `TaskSubmission` (`types.rs:72`) : requis = `project_id`, `task_type`, `prompt`, `model` ;
  reste `#[serde(default)]` (redundancy=1, verifiable=false par défaut → pas de cohorte/runtime gate).
- `coordinator_submit_task` (`http.rs:3306`) : guardrail input → `dispatcher::submit_task` →
  **auto-spawn worker local on-demand** `local_worker.ensure_spawned(project_doc, sbfb_home)`
  (Hotfix #5). Réponse = `TaskEntry` sérialisé, task_id **imbriqué** `body.task.task_id`
  (`runtime.rs:4072`).
- Worker local provisionné `ConsentLevel::Whitelist`, `allowed_project_ids = {project_doc.id()}`
  (`local_worker.rs:305,334-335`). `should_accept_task` Whitelist (`consent.rs:408-412`) rejette
  `NotInWhitelist` si `project_id ∉` whitelist ; n'exige PAS `is_open_source`.
  → une tâche dont `project_id == project_doc.id()` est claimée + exécutée ; tout autre
  project_id (dont le node_id, ou le blake3(name) d'une app déployée `deploy.rs:141`) →
  jamais claimée → `/result` 404 éternel.
- **`project_doc.id()` n'est exposé par aucune route HTTP.** `GET /api/daemon/info` →
  `state.snapshot()` ne le contient pas ; le node_id ≠ doc id (warning live-smoke
  `local_worker.rs:331-333`).

## Adaptation à implémenter

1. **Daemon (Rust, additif, read-only)** : route authentifiée `GET /api/daemon/project-info`
   → `{ "project_doc_id": Option<String> }` lisant `state.project_doc.as_ref().map(|d| d.id())`.
   + test unitaire. Pas de bump wire (réponse HTTP loopback, pas un format P2P).
2. **Shell `coordinator.ts`** : `getDaemonProjectInfo`, `submitComputeTask` (body avec
   `project_id` injecté ; réponse lit `task.task_id`), `getTaskResult` (404 → `{status:"pending"}`).
3. **Shell `protocol.ts`** : ajouter `task_result` à `BridgeMethodSchema`.
4. **Shell `useBridge.ts` dispatch** :
   - `task_submit` : `getDaemonProjectInfo` → injecte project_id → `submitComputeTask` →
     retourne `{task_id}`. L'app reste node-agnostique (n'envoie que prompt/model/task_type).
   - `task_result` : `getTaskResult(taskId)`.
   - tests `useBridge.test.ts` : succès task_submit (mock project-info + submit) + task_result
     (404 pending / 200 done).
5. **SDK `sbfb-bridge.js`** : `getTaskResult(taskId)` → `_call("task_result",{task_id})` ;
   recopie identique web/public + examples/{sbfb-ideas,sbfb-explorer,sbfb-factory-viewer}.
6. **App** `examples/compute-tester/` (SBFB.json, index.html div+button sans form, app.js
   submit→poll, sbfb-bridge.js).

## Invariants / Day-0

- Pré-launch : pas de bump `*_VERSION` (route HTTP additive, pas wire P2P). OK.
- `heberger != publier, seeder != auteur` non touché.
- Le project_id d'attribution de la tâche locale = `project_doc.id()` du nœud (pas l'app) ;
  l'attribution compute par-app est une question produit → S77 (avec B).
- Aucune décision Day-0 figée touchée. Additif pur côté wire + shell + SDK.
