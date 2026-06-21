# Étude — Éditeur / IDE intégré dans Factory (post-S77)

> **Statut** : recherche figée, hors-cycle (exploratoire). Aucune décision Day-0 prise — document d'aide à un futur kickoff Cas C.
> **Date** : 2026-06-20.
> **Contexte de production** : 4 tours d'analyse en lecture/recherche pure (aucun code applicatif modifié), via 3 Workflows ultracode :
> - `wf_3f72e2d7-33f` — inventaire de l'existant Factory (7 lecteurs Explore).
> - `wf_90c925bb-0bc` — recherche éditeurs open source (6 chercheurs web, licences vérifiées).
> - `wf_e0eabc7c-a34` — étude ultra-deep de la solution la plus poussée (7 études + synthèse-architecte).
> **Mémoires liées** : `factory_ide_design_study.md`, `idea_hub_post_s77.md`, `nexus_grid_pivot.md`.
> **Timing** : chantier **post-S77** (sharding pipeline en cours d'abord).

---

## 1. Contexte & objectif

Le PO veut, après S77, faire évoluer Factory en un vrai poste de travail « idée → app » : un **éditeur de code / IDE intégré** pour éditer le workspace atelier directement dans Factory, sans sortir vers un éditeur externe. Cette étude établit (a) ce qui existe déjà, (b) les briques open source réutilisables, (c) la solution la plus poussée viable, (d) la posture de sécurité (sandbox ou non).

**Invariant produit** : l'éditeur est le **chaînon manquant** du flux idée → app. Une idée devient une app éditée puis publiée ; la **provenance Ed25519 du publish fait office de « champion signé »** de la vision idea-hub — pas de crypto neuve à inventer.

---

## 2. Analyse de l'existant Factory (inventaire)

Factory = **outil client local du nœud** (hors daemon, décision gelée D2), composé de briques qui se réutilisent toutes :

| Brique | Rôle | Où |
|---|---|---|
| `sbfb-factory` — CLI | 10 commandes + sous-commandes `process`/`operator` | `crates/sbfb-factory/src/main.rs:137-336` |
| Pipeline création | scaffold template → `SBFB.json` → lock+provenance → gates → deploy | `template_engine.rs`, `pipeline.rs`, `template_lock.rs`, `provenance.rs` |
| Atelier / fork / publish | workspace local, fork, redeploy re-signé | `atelier.rs`, `fork.rs`, `publish.rs`, `gates.rs`, `diff.rs`, `secret_scanner.rs` |
| Operator server | API JSON loopback gated : statut sprint, lint, audit, prompts, context-pack, **chat LLM multi-provider**, artifacts draft, **terminal WS** | `operator_server.rs`, `process.rs`, `sprint_history.rs`, `llm_bridge.rs`, `provider_router.rs`, `terminal.rs`, `auth.rs` |
| `factory-ui` — Viewer readonly | composants React lecture-pure (timeline, ProofCard, verdict chips, changelog) | `tools/factory-ui/src/readonly/` |
| `factory-ui` — Operator front | client API + extensions privilégiées (chat, ActionCenter) | `tools/factory-ui/src/operator/` |
| Apps SBFB exemples | Ideas Hub, Explorer, compute-tester | `examples/` |
| Bridge SDK | `postMessage` iframe↔host, 16 méthodes whitelist | `web/src/bridge/protocol.ts`, `crates/sbfb-manifest/src/lib.rs:67-91` |

**Modèle de données** : Factory ne possède pas de DB — il **lit** les conventions `.planning/` (kickoff/plan/phase/verification). Hiérarchie d'autorité gelée : **`process > RRV > Factory`** (les composants downstream affichent/indexent, ne décident pas).

**Conclusion** : ~80 % du socle d'un IDE existe déjà (FS atelier, terminal, preview, publish, provenance, chat LLM). Le travail neuf est surtout l'**assemblage** d'un éditeur + le branchement.

---

## 3. Recherche éditeurs open source (matrice, licences vérifiées sur LICENSE réels)

| Candidat | Licence | Modèle | +Node ? | LSP/Extensions | Fit SBFB |
|---|---|---|---|---|---|
| **Monaco Editor** | **MIT** | composant navigateur (client-only) | non | IntelliSense TS/JS natif ; LSP via `monaco-languageclient` | **4.5/5** |
| **CodeMirror 6** | **MIT** | composant navigateur (client-only) | non | LSP via `@codemirror/lsp-client` (first-party) | **4/5** |
| code-server | MIT | serveur Node (~1 Go RAM) | **oui** | IDE complet (Open VSX, terminal, debug) | 2.5/5 |
| openvscode-server | MIT | serveur Node | **oui** | IDE complet | 2/5 (maintenance ralentie, pivot Gitpod→Ona) |
| Eclipse Theia | EPL-2.0 / GPL-2.0-CPE | framework Node | **oui** | IDE complet, branding libre | 2/5 (**licence ≠ AGPL**) |
| Zed / Lapce / Helix | GPL/Apache/MPL | **desktop natif (Rust)** | — | — | **0/5** (non embarquables navigateur) |

**Pièges licence/marque** : le build « Visual Studio Code » est proprio (marque + télémétrie + marketplace) ; seuls **Code-OSS / Monaco / VSCodium / code-server (MIT)** sont réutilisables. **Marketplace Microsoft interdit aux forks** → Open VSX uniquement. **Incident supply-chain GlassWorm** (Open VSX, oct 2025 → mars 2026, 7 → 72+ extensions, vol de tokens) : argument fort contre toute marketplace d'extensions.

---

## 4. Réutilisation vs réinvention (point clé : on ne recrée pas d'éditeur)

- **Des éditeurs en Rust existent** (Zed v1.0 avril 2026, Lapce, Helix) mais ce sont des **applications de bureau** — **aucun n'est embarquable dans un navigateur** (vérifié : les discussions Zed confirment qu'une version web exigerait WASM + bindings + FS simulé, non mûr). La décision gelée « browser = client, pas Tauri/Electron » les exclut.
- **Aucun éditeur de code généraliste, web, écrit en Rust** n'existe à un niveau utilisable (l'écosystème Rust+WASM = moteur JS Boa, studios spécialisés, pas un éditeur de code).
- **On ne réinvente donc PAS d'éditeur.** Le cœur d'édition (coloration, autocomplétion, gros fichiers) = **Monaco ou CodeMirror**, réutilisé **tel quel** (MIT). Ce qu'on construit = l'**assemblage** (file-tree, onglets, branchement au backend Rust), qui est **court**.
- **Pourquoi l'éditeur visible n'est pas en Rust** : tout ce qui s'affiche dans un navigateur est en langage web par nature. Le Rust est le **backend** (FS, terminal, git, sécurité) — là où il a de la valeur.

> Analogie : on n'invente pas le moteur, on monte la carrosserie autour d'un moteur open source éprouvé.

---

## 5. Étude ultra-deep — la solution la plus poussée

### 5.1 Verdict

**La solution la plus poussée ET alignée = un IDE web sur-mesure dans Factory, éditeur client-only (CodeMirror 6 recommandé, ou Monaco), FS/terminal/git/preview/publish 100 % en Rust (Operator axum), LSP optionnel via un pont WebSocket↔stdio spawnant des language servers.** ~80 % d'un VS Code sur l'atelier, **sans runtime Node persistant, sans marketplace, Day-0-clean, AGPL-clean.**

**Theia et code-server écartés** : serveur Node permanent (~1 Go RAM) = conflit frontal avec « backend = Rust » ; Theia = licence EPL/GPL ≠ AGPL ; marketplace = vecteur GlassWorm.

### 5.2 Le fait nouveau décisif : tsgo

Le dogme « le typage TS cross-fichier exige Node » a basculé en 2026 : **`tsgo`** (`@typescript/native-preview`, TypeScript 7 réécrit en Go, `tsgo --lsp`, **zéro Node**) est adopté par Zed/Helix. **Nuance honnête** : LSP « in progress », stable visé ~fin juin 2026 — **pari à 2-3 mois**, pas une certitude. Python : `ruff` (natif) fait lint/format mais pas le typage profond ; `pyright` reste Node.

### 5.3 Le pivot Node (arbitrage central pour le PO)

| | Option | Capacités | Day-0 |
|---|---|---|---|
| **A** | Zéro Node strict — LS natifs (`rust-analyzer`/`biome`/`ruff`/`taplo`/`tsgo`) | ~80 % (typage cross-fichier dégradé tant que tsgo immature) | ✅ parfaite |
| **B** | Node **éphémère** — `typescript-language-server`/`pyright` spawnés on-demand, tués au close | IDE plein aujourd'hui | ⚠️ à trancher |
| **C** | Node persistant — code-server/Theia | VS Code complet | ❌ rejeté |

Un LS Node spawné-puis-tué **n'est pas** un runtime persistant (≠ code-server) — c'est un outil externe éphémère, comme l'Operator spawn déjà `git`/`claude`/`codex`. **Reco : A par défaut, B opt-in par-atelier** (le pont Rust est identique ; seule la table `langue → commande` change).

### 5.4 Architecture cible recommandée

```
NAVIGATEUR : factory-ui (NOUVELLE app Vite) — file-tree + éditeur (CM6|Monaco) + xterm + preview iframe + SCM + copilote
   │ HTTP /api/atelier/*        │ WS /api/terminal/ws    │ WS .../lsp/ws?lang=   │ HTTP /api/git/* · preview · publish
OPERATOR RUST (axum, loopback) — .layer(auth_required) sur CHAQUE route (Host + Origin + X-SBFB-Token)
   NOUVEAU  FS atelier (tree/file/scaffold/search) → FG5 check_path_containment sur chaque accès
   REUSE    terminal.rs (PTY portable-pty) + preview_cmd.rs + pipeline.rs (FG4/5/6/8) + atelier::redeploy + provider_router (copilote)
   NOUVEAU  lsp_bridge.rs (WS↔stdio, allowlist langue→cmd, kill-on-close) + git.rs (shell durci, commit/push GATÉS)
DAEMON RUST — preview/load · deploy-from-repo · blob-serve   (wire INCHANGÉ, 0 bump)
```

### 5.5 Capacités par priorité

- **P0** : édition multi-fichiers + onglets + diff, file-tree, coloration, **preview iframe sandbox** (`preview_cmd.rs` existe).
- **P1** : **terminal PTY** (`terminal.rs` existe), recherche projet (`grep`/`ignore` Rust), command palette, **git** (commit/push **gatés**), **publish-from-IDE** (`pipeline`/`redeploy` existent).
- **P2** : **LSP cross-fichier** (le pivot Node), **copilote LLM** (`provider_router` existe — Ollama = offline gratuit).
- **Différé (peut-être jamais)** : **debug DAP** — DevTools Chrome couvre 90 % des apps iframe.

### 5.6 Threat model & consentement

- **RCE-by-design légitime** (éditer + terminal + LS + build). ⚠️ Le gate actuel `SENSITIVE_ACTIONS` est keyword-based et **ne couvre pas le canal terminal** → terminal IDE = RCE non gatée.
- **Reco : Workspace Trust** — un atelier forké = `untrusted` par défaut (édition + LS read-only + preview) ; build/terminal/commit = consentement explicite par-atelier ; commit/push/deploy = gate + session agent réelle. Le contenu de l'atelier est **données, jamais instruction de confiance**.
- **Pas de marketplace = réponse à GlassWorm.** LS = binaires **pinnés par hash, lancés par l'Operator**, jamais auto-téléchargés.
- ⚠️ **`rust-analyzer` exécute du code** (`build.rs` + proc-macros) → **uniquement sur le repo SBFB de confiance**, jamais sur un fork tiers (sinon `buildScripts`/`procMacro` OFF).
- **2 régimes CSP stricts jamais confondus** : apps SBFB (`sandbox` sans `allow-same-origin`, `connect-src 'none'`) vs outil Factory (`script-src 'self'`, `worker-src 'self'`, `connect-src 'self' ws://127.0.0.1:*`).
- **Auth loopback** (Host + Origin + token) sur chaque nouvelle route ; token WS en `Sec-WebSocket-Protocol`. **Scope FS borné à l'atelier** via FG5 (`check_path_containment`), denylist `.git`/`node_key`/`auth_token`/`~/.sbfb`.

### 5.7 Intégration au flux Factory

atelier → edit (`PUT /api/atelier/{id}/file`, FG5) → diff (`diff::diff_workspace`) → preview (`preview_cmd` → iframe sandbox, **jamais blob-serve brut**) → validate (afficher FG4/5/6) → **publish gaté, deux chemins honnêtes** : *vérifié open-source* (`pipeline::run_publish_pipeline` → `deploy-from-repo`, clone+Ed25519+SLSA L1) si repo clean+pushed, ou *redeploy self-attestation* (`atelier::redeploy`, `is_open_source=false`, pas de `commit_sha`) si dirty. **Invariant : « héberger ≠ publier » — publish reste un acte d'auteur signé.** Widgets métier = réutiliser `readonly/{ProofCard,SprintTimeline,...}`. **Dette à router : `TABVIEW-APP-SUBMIT-DEAD`** (bloquant si l'idea-hub génère des apps TabView).

### 5.8 Roadmap d'implémentation (mappable sprints/phases SBFB)

| Jalon | Contenu | Différable ? |
|---|---|---|
| **M0a** | **Shell host Operator** (app Vite — prérequis dur, absent aujourd'hui) | Non |
| **M0b** | FS atelier confiné (tree/file/scaffold) + FG5 sur chaque accès | Non |
| **M1** | Éditeur multi-fichiers + diff + palette + **preview** | Non |
| **M2** | Terminal (généraliser `terminal.rs` `?shell`+`?cwd`) + consentement par-atelier | Non |
| **M3** | Git (shell durci, commit/push gatés) + publish-from-IDE | Partiel |
| **M4** | **LSP bridge** (natifs d'abord) — **le pivot Node, découplé** | Oui |
| **M5** | Copilote LLM (`provider_router`) | Oui |
| **M6** | Debug DAP (MVP via terminal) | Oui — peut-être jamais |

**M0-M3 = MVP « idée → app éditée → publiée », ~1-2 sprints solo, zéro conflit Day-0.** M4 découplé : ne jamais coupler la valeur (éditer/preview/publish) au risque (proxifier un LS privilégié). **0 bump wire daemon.**

---

## 6. Décision d'architecture — iframe / sandbox

Question : Factory doit-il vivre dans un iframe sandbox comme les apps utilisateurs ? **Réponse : non pour la partie qui agit, oui pour la partie qui ne fait que regarder.** Principe directeur : **la confiance détermine le confinement.**

| « Factory » | Rôle | Sandbox ? | Pourquoi |
|---|---|---|---|
| **IDE / Operator** (édition, terminal, git, publish) | agit sur la machine | **NON** — privilégié-local, même origine que le shell | Le sandbox coupe FS **et** réseau → l'IDE ne pourrait même plus parler à son Operator Rust. C'est l'outil **de confiance** du propriétaire. |
| **Viewer** (statut sprints, preuves, changelog) | lit & affiche | **OUI possible** — app SBFB sandboxée | Aucun pouvoir requis → peut être livré comme n'importe quelle app du réseau (bon dogfooding). |
| **Preview** de l'app éditée dans l'IDE | exécute du code untrusted | **OUI toujours** — iframe sandbox enfant | L'app en cours d'édition (surtout un fork tiers) n'est **pas encore de confiance**. On ne rend **jamais** une app dans le document privilégié de l'éditeur. |

C'est **déjà la direction figée** (S70) : un *Viewer* consultable (sandbox possible) distinct d'un *Operator* privilégié-local (jamais sandbox). L'IDE est une **extension de l'Operator**. À inscrire au THREAT_MODEL comme exception privilégiée distincte (l'iframe IDE est l'opposé des apps sandboxées : même origine + WS).

---

## 7. Découvertes dans le code réel (corrigent la cartographie initiale)

- ✅ **`terminal.rs` est déjà un VRAI PTY interactif** (`portable-pty 0.9`, resize/kill testés, l.48-204) **+ journal asciicast** — pas un simple replay. Réutilisable ; gap = paramétrer `?shell` + `?cwd=atelier` (spawn `claude` hard-codé l.69).
- ✅ **`gates.rs` FG5** (`check_path_containment` l.117, `dunce::canonicalize` + `starts_with`) et **FG6** (`secret_scanner` l.127) directement réutilisables.
- ⚠️ **`tools/factory-ui` n'est PAS une app** — c'est une **librairie** (`exports ./readonly` + `./operator`, pas de `main.tsx`/Vite/router). **Aucun shell host Operator monté** → **M0a = prérequis dur**, avant l'éditeur.
- `atelier.rs` redeploy R5 `is_open_source=false`, pas de `commit_sha` (l.104).
- `SENSITIVE_ACTIONS` gate keyword-based (`operator_server.rs:35`), ne couvre pas le terminal.

---

## 8. Décisions à arbitrer par le PO (au futur kickoff)

1. **Curseur Node** : un LS Node spawné-on-demand-puis-tué (`typescript-language-server`/`pyright`) est-il toléré (comme `git`/`claude`) ou interdit ? → A strict vs A+B.
2. **Éditeur** : **CodeMirror 6** (surface npm minuscule — reco sous threat model anti-supply-chain) ou **Monaco** (IntelliSense TS single-file natif + rendu VS Code, mais `monaco-languageclient` v10 tire 100+ paquets / +10 Mo) ?
3. **Ambition** : IDE de confort (~80 %, voie maison Rust) suffit, ou IDE de marque complet (debug + `.vsix`, voie Node) est un objectif produit central ?
4. **`rust-analyzer` sur ateliers** : interdit par défaut (reco), ou opt-in gaté `buildScripts`/`procMacro` OFF ?
5. **tsgo** : attendre sa maturité (~fin juin 2026) pour le typage TS cross-fichier, ou tolérer un `typescript-language-server` Node éphémère d'ici là ?
6. **Debug DAP** : différé/jamais (DevTools Chrome suffit), ou objectif futur explicite ?
7. **Périmètre** : M0a + FS + éditeur + terminal + preview = **un sprint Cas C dédié** (kickoff appliquant le gate de testabilité T1/T2), ou éclaté ?
8. **Viewer sandboxé** : livre-t-on la face consultation comme app SBFB dogfoodée (sandbox), distincte de l'Operator privilégié ?

---

## 9. Sources

**Éditeurs / licences** (vérifiées sur repos / LICENSE réels, 2025-2026) :
- Monaco MIT : https://github.com/microsoft/monaco-editor · CodeMirror lsp-client : https://github.com/codemirror/lsp-client
- code-server MIT : https://github.com/coder/code-server/blob/main/LICENSE · openvscode-server : https://github.com/gitpod-io/openvscode-server
- Theia EPL-2.0/GPL-CPE : https://github.com/eclipse-theia/theia · marketplace MS interdit aux forks : https://github.com/microsoft/vscode/issues/141340
- Open VSX + GlassWorm : https://thehackernews.com/2025/10/self-spreading-glassworm-infects-vs.html · https://thehackernews.com/2026/03/glassworm-supply-chain-attack-abuses-72.html

**tsgo / TypeScript natif** :
- https://devblogs.microsoft.com/typescript/announcing-typescript-native-previews/ · https://github.com/zed-industries/zed/discussions/31541 · https://github.com/helix-editor/helix/issues/15134

**Éditeurs Rust desktop (non embarquables web)** :
- Zed : https://en.wikipedia.org/wiki/Zed_(text_editor) · version web (discussion) : https://github.com/zed-industries/zed/discussions/22953 · pure-Rust editor on wasm32 : https://users.rust-lang.org/t/any-pure-rust-minimal-editor-that-runs-on-wasm32/98935

**Briques IDE** : monaco-languageclient v10 https://www.typefox.io/blog/monaco-languageclient-v10/ · monaco-vscode-api (100+ pkgs) https://github.com/CodinGame/monaco-vscode-api/issues/383 · xterm.js https://github.com/xtermjs/xterm.js/ · portable-pty https://docs.rs/portable-pty · rust-analyzer RCE https://github.com/rust-lang/rust-analyzer/issues/14375

**Fichiers SBFB load-bearing** : `crates/sbfb-factory/src/{terminal.rs,operator_server.rs,gates.rs,atelier.rs}` · `tools/factory-ui/package.json`
