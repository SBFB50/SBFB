# Sprint 76 — Phase H (post-audit) : projet « Compute Tester » + acceptance compute LIVE

**But** : prouver EN VRAI la feature phare de S76 (compute GPU cross-machine) avec un
projet SBFB dédié qui soumet une tâche IA via le bridge `task_submit` et affiche le
résultat. S76 est **audité-clos** (CONDITIONAL PASS levé) ; ceci est une phase
d'**acceptance de test**, pas du nouveau scope produit. Handoff écrit pour un contexte frais.

---

## 0. PROMPT À COLLER (contexte frais)

> Reprends `.planning/active/sprint76_phase_h_compute_tester_plan.md`. Objectif :
> construire et déployer un petit projet SBFB « Compute Tester » (HTML/JS) qui, via le
> bridge `submitTask({prompt, model})`, soumet une tâche IA et affiche le résultat, pour
> tester la feature compute de S76 — **LOCAL d'abord** (worker co-localisé sur le nœud
> Windows, ce qui évite le bug de convergence WAN), puis cross-machine si le local passe.
> Lis d'abord §3 (setup déjà en place), §4 (ce qu'il faut construire), §5 (étapes
> d'acceptance), et §6 (blockers connus de la session du 2026-06-19). Confirme STEP 0
> (chemin de livraison du résultat à l'iframe) AVANT de coder l'app.

---

## 1. Pourquoi (constat de la session 2026-06-19)

Aujourd'hui on a prouvé LIVE la **distribution d'app cross-machine** (publish vérifié
depuis source → un nœud frais Mac découvre + fetch + rend over LAN). MAIS la **feature de
S76 (compute GPU)** n'a PAS pu être prouvée en vrai :
- le harness `b3_live_pc_vps.sh` avait un bug d'auth (corrigé `fef781d`, `x-sbfb-token`) ;
- les tâches soumises au coordinateur **ne se propagent pas** au worker over WAN
  (convergence delivery = carry S77) ;
- la synchro d'état mutable (storage app) est opt-in + ne converge pas cross-version.

**Clivage produit confirmé sur 3 surfaces** : la **distribution de contenu immuable** est
robuste ; l'**état mutable / sync temps-réel cross-nœud** (compute live, storage, quorum)
est la frontière. → un test compute doit commencer **en LOCAL** (app + worker même machine,
0 sync réseau) pour valider la chaîne, avant le cross-machine.

## 2. Définition de « fait »
- Single-machine : app Compute Tester déployée sur le nœud Windows, un prompt soumis via
  `task_submit` est exécuté par un worker GPU local (Ollama) et le **résultat s'affiche
  dans l'iframe**. Chaîne app→bridge→coordinateur→worker→résultat→app prouvée bout-en-bout.
- Cross-machine (stretch) : worker homogène sur le Mac, `redundancy=2`+`verifiable=true`,
  quorum byte-identique. (Bute probablement sur la convergence WAN = à diagnostiquer, pas
  à gonfler.)

## 3. Setup DÉJÀ EN PLACE (ne pas reconstruire)
- **Nœud Windows** : `nexus-shell-daemon.exe start --web-root web/dist`, HTTP
  `127.0.0.1:7654`, node_id `fe7a4898…`, 4 apps déployées (sbfb-explorer/ideas/
  factory-viewer/ideas-demo). Auth daemon = header `x-sbfb-token` (token via
  `GET /auth/token`). (Le daemon a peut-être été arrêté depuis — relancer si besoin.)
- **Worker S76 construit** : `target/release/nexus-worker.exe` (rebuild S76, identité
  `pc-rtx5080-s76b`), config `AppData\Roaming\FlowUP\nexus-grid\config\worker.toml`.
  Ollama a `llama3.1:8b` (+ gemma-26B). Voir [[live-acceptance-setup]].
- **Mac** : daemon `711c228d` port 7655, **build S75** (à rebuild en S76 pour le quorum),
  abonné au nœud Windows.
- SSH : alias `vps` / `mac` dans `~/.ssh/config`. VPS désormais sur S76 (backup `.s75.bak`).

## 4. Ce qu'il faut construire

### STEP 0 (À CONFIRMER AVANT DE CODER) — comment le RÉSULTAT revient à l'iframe
Le SDK (`examples/sbfb-ideas/sbfb-bridge.js`) : `submitTask(payload)` →
`_call("task_submit", payload)` → le shell (`web/src/bridge/useBridge.ts:234`) appelle
`submitAppTask(coordUrl, appName, taskBody)` et retourne la réponse coordinateur
(**task_id**). Le RÉSULTAT (texte généré) arrive PLUS TARD — vérifier le canal exact :
- `bridge.onEvent("task_result_ready", cb)` (push event mentionné dans le SDK), OU
- polling d'une route résultat (`/api/v1/tasks/{id}/result`), OU
- une méthode bridge dédiée.
**Tracer dans `web/src/bridge/` (useBridge + protocol.ts + submitAppTask) le chemin
result→iframe AVANT d'écrire app.js.** C'est le point qui détermine la boucle UI.

### L'app (projet « Compute Tester »)
- `SBFB.json` : `{schema_version:2, name:"compute-tester", category:"tools",
  description:"...", bridge:{methods:["task_submit"]}}`.
- `index.html` : `<script src="sbfb-bridge.js">` + textarea prompt + bouton Soumettre +
  zone résultat + indicateur d'état. **Pas de form submit** (sandbox sans allow-forms →
  div+button+click, cf. [[feedback-iframe-sandbox-forms]]). Strings FR.
- `app.js` : `const b=new SBFBBridge(); const {task_id}=await b.submitTask({prompt, model:"llama3.1:8b", task_type:"analysis"});`
  puis abonner le résultat selon STEP 0, afficher quand prêt. Afficher task_id + état
  (soumis / en cours / done / timeout).
- `sbfb-bridge.js` : copier `examples/sbfb-ideas/sbfb-bridge.js`.

### Déploiement (le plus simple = LOCAL)
- **Option A (rapide, local)** : zipper le dossier app (index.html à la RACINE) + POST
  `/api/v1/deploy-workspace?project_name=compute-tester&category=tools&description=...`
  (corps = le zip). `is_open_source=false` (auto-attesté). Pas besoin de repo public.
- **Option B (open-source vérifié)** : pousser l'app dans un repo public (index.html
  racine, comme `github.com/SBFB50/sbfb-ideas-demo`) + `POST /api/v1/deploy-from-repo`
  `{repo_url, project_name, category, description, apps}`.

### Worker (servir les tâches)
- Le plus simple : panneau « offrir ma puissance » du shell (S76 Phase A, worker
  co-localisé) — l'activer met un worker GPU local en service. OU enrôler le standalone :
  `nexus-worker.exe register` (déjà fait) → mint invite worker via
  `POST /api/v1/invite/create {"scope":"worker"}` (header `x-sbfb-token`) → `join` →
  `start --headless`. Le worker doit accepter le projet (consent / claim-gate). En LOCAL
  il n'y a pas de convergence réseau → la tâche doit être claimée et exécutée.

## 5. Étapes d'acceptance
1. **STEP 0** confirmé (chemin résultat→iframe).
2. App construite + déployée sur Windows (Option A). Visible dans Browse, rend dans l'iframe.
3. Worker local en service (panneau offrir-ma-puissance OU standalone enrôlé).
4. Ouvrir l'app via le shell (jamais blob-serve direct, cf. [[feedback-no-direct-blobserve]]),
   taper un prompt court déterministe, Soumettre. **Attendu : task_id puis le texte généré
   par le GPU local s'affiche.** Mesurer le délai.
5. Si timeout : diagnostiquer (worker claime-t-il ? grep le log worker pour le task_id ;
   consent/claim-gate ; Ollama up). En LOCAL le chemin est prouvé in-process (audit) ;
   un échec local = bug de câblage à corriger, pas la convergence WAN.
6. **Stretch cross-machine** : rebuild daemon+worker Mac en S76, worker Mac homogène
   (même `llama3.1:8b`), soumettre `redundancy=2 verifiable=true`, observer le quorum.
   (Probable BLOCK convergence WAN → consigner le diagnostic, ne pas gonfler.)

## 6. Blockers / apprentissages connus (session 2026-06-19) — NE PAS re-découvrir
- **Auth daemon = `x-sbfb-token`** (PAS `Authorization: Bearer`). Harness corrigé `fef781d`.
- **Convergence delivery WAN** : nouvelles tâches/entrées de doc ne se propagent pas à la
  réplique d'un nœud distant ; seul le sync initial bulk marche. Remède partiel : **restart
  du daemon distant** (re-pull au boot) ; **keep-online** pour seeder un déploiement frais.
  Carry S77. → tester compute en LOCAL d'abord.
- **Storage app = sync opt-in** (`storage/ticket`→`storage/join`), pas automatique ; ne
  converge pas entre Mac S75 et Windows S76. `version` n'est PAS un compteur d'écritures.
- **Daemon longue-durée = catalogue figé** ; restart pour re-converger l'annuaire.
- Détails infra (ports, PROJECT_IDs, chemins worker, VPS-sur-S76) : [[live-acceptance-setup]].
- Sandbox iframe : pas de form submit, pas de localStorage (origine opaque) → bridge only.

## 7. Sortie attendue
- Projet `compute-tester` déployé + rendu, **un prompt exécuté par un GPU local avec
  résultat affiché** (preuve LOCAL de la chaîne compute S76). Trace consignée dans
  `sprint76_verification.md §5.1`. Cross-machine = stretch, diagnostic honnête si BLOCK.
- Si le compute local lui-même casse (worker ne claime pas / résultat ne revient pas à
  l'iframe) → c'est un **vrai bug** à router (fix ou carry S77), distinct de la convergence WAN.
