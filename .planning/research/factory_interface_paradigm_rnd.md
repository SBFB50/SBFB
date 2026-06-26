# R&D ultradeep — La meilleure interface pour Factory

> **[MISE À JOUR — DÉCISION PO 2026-06-26]**
> Surface confirmée = **Operator** (outil local privilégié). La conclusion « **PAS un
> IDE / fork / plugin** » est **tenue** et fonde le choix. La **base stack retenue pour
> S80** est le greenfield `factory_front_greenfield_blueprint.md` (doc 3), avec deux
> corrections PO : **shadcn N'EST PAS exclu** (candidat de la couche composants) ;
> **motion ou anime.js EST voulu** pour un UI extrêmement poussé. Le **Viewer scellé**
> (`factory_front_best_approach_research.md`) est **reporté** (candidat S81+).
> Phase 0 = audit gate S79 d'abord.

> **Méta.** Document décision-grade. Date : 2026-06-26. Méthode :
> orchestration Workflow en 5 étages — **Ground** (cube de contraintes
> vérifié in-code + nature agent-native du flux réel), **Technologies**
> (5 familles techno analysées en profondeur, licences vérifiées à la
> source), **Crosscut** (souveraineté / futur agentique 2026→2028 /
> passe adversariale), **Verify** (fact-check indépendant de 4 claims
> décisifs), **Synthèse** (ce rapport). Web-grounded (paysage réel
> juin 2026 : éditeurs, agent-IDEs, frameworks web-IDE, supply-chain
> Open VSX/GlassWorm, licences FLOSS). Tous les faits-socles sont
> ancrés à un fichier réel du repo (`CLAUDE.md`, `crates/sbfb-factory`,
> `crates/nexus-core-rs`, `tools/factory-operator`) ou à une source
> web datée. Ce rapport **ne rubber-stampe PAS** l'étude IDE existante
> (`factory_embedded_ide_study.md`) ni la tentation « reprendre VS
> Code » : il les confirme là où elles ont raison et les **dépasse**
> là où elles sur-scopent.

---

## 1. Réponse courte

La meilleure interface pour Factory **n'est pas un IDE**, ni un fork
d'IDE, ni un plugin d'IDE. C'est une **app web sur-mesure servie en
loopback par le backend Rust déjà existant**, organisée selon un
**paradigme agent-natif « control-center »** : l'humain donne des
**intentions**, l'agent (Claude Code / Ollama / réseau via
`provider_router`) fait le travail, et l'humain **vérifie** via une
surface de diff-review de premier rang + les gates Rust + le preview
scellé. L'éditeur de code (**CodeMirror 6**, MIT, **pas Monaco**) est
une surface **secondaire** d'inspection/patch, jamais le centre.

**Cette recommandation ne rompt AUCUNE Day-0 gelée.** Elle franchit
les 17 contraintes absolues + les 2 révisables du cube intact. Le
**seul gap technique réel** est l'absence d'un service de fichiers
statiques Rust dans l'Operator (`operator_server.rs:17` n'importe que
`tower_http::cors`) — à combler par un `tower_http::ServeDir`, jamais
par un runtime Node persistant. Les familles « reprendre un IDE »,
« plugin d'IDE » et « desktop natif » sont **éliminées** (Electron/
Tauri violent une Day-0 littérale ; serveurs Node + marketplace Open
VSX réintroduisent le vecteur GlassWorm que toute l'architecture
existe pour combattre).

---

## 2. Le cube de contraintes (ce qui élimine d'emblée)

Toute interface candidate doit franchir un « cube » de contraintes
tracées à une source réelle et classées par provenance. Une option
qui viole un **Tier-0 littéral** (gelé, `CLAUDE.md` « à ne PAS
re-débattre ») ou un **Tier-1 code-enforced** (invariant runtime déjà
câblé) est **éliminée d'office**. Une option qui ne heurte qu'un
révisable (Tier-2) est pénalisée et doit justifier son coût.

### 2.1 Contraintes ABSOLUES — Tier-0 littéral (gelées)

| # | Contrainte | Ancre | Ce que ça élimine |
|---|---|---|---|
| **C1** | browser = client, **PAS Tauri/Electron** | `CLAUDE.md:472` « Launcher Rust minimal (pas Tauri, browser = client) » | Tout webview embarqué dans une app native. VS Code/VSCodium desktop = Electron → éliminé. **Tauri est nommé littéralement.** |
| **C2** | rendu des apps scellées = archive zip → daemon blob-serve → iframe sandbox | `CLAUDE.md:468` | Toute interface qui prévisualiserait une app SBFB autrement que via l'iframe blob-serve. |
| **C3** | postMessage bridge = seul canal iframe ↔ réseau (3 méthodes) | `CLAUDE.md:470` | Un éditeur qui donnerait à l'app scellée un accès réseau direct. |
| **C4** | AGPL-3.0 maintenue | `CLAUDE.md:467` + en-têtes SPDX sur chaque source | Toute brique de licence incompatible (proprio JetBrains ; EPL-2.0/GPL-2.0-only de Theia). |
| **C5** | Factory = outil client externe, crate Rust `sbfb-factory`, hors daemon | `CLAUDE.md:477` | Une interface où l'autorité vivrait dans un host d'extension Node tiers. |
| **C6** | sandbox d'exécution = OS sandbox, **pas wasmtime** | `CLAUDE.md:474` | Un pari interface supposant un runtime wasm hôte. |
| **C7** | UX = **intentions**, jamais jargon | `CLAUDE.md:507-510` | Une UI exposant CLI/kinds/providers en surface primaire (≈ « ouvre un terminal et tape »). |
| **C8** | connaissance **consommée, jamais autoritaire** ; 0 verdict PASS auto ; verdict final = session agent + gates + preuves | `CLAUDE.md:502-503` | Une interface qui clôt un verdict elle-même. |
| **C9** | contexte agent depuis context-pack **repo-visible**, pas mémoire de chat implicite | `CLAUDE.md:504-506` | Un paradigme conversationnel pur sans ancrage repo. |

### 2.2 Contraintes ABSOLUES — Tier-1 code-enforced (déjà la « forme » de l'interface)

| # | Invariant déjà câblé | Ancre vérifiée | Conséquence |
|---|---|---|---|
| **C10** | bind loopback strict 127.0.0.1 | `operator_server.rs` (`TcpListener::bind 127.0.0.1`) | L'interface est un client local d'un service local. |
| **C11** | Host + Origin + token bearer sur chaque route | `auth.rs` (`auth_required`, token CSPRNG 64 hex) | Une UI navigateur ne lit pas `~/.sbfb` → besoin d'un proxy serveur-à-serveur injectant le token. |
| **C12** | gate `SENSITIVE_ACTIONS` : `shell`/`commit`/`push`/`PASS` ne spawnent jamais d'agent autonome | `operator_server.rs:35` const ; appliqué `:752`, `:854`, `:952` | Pas de bouton « commit/push/PASS » qui s'exécute sans session agent externe. **Câblé, vérifié.** |
| **C13** | `/api/artifacts/draft` refuse un verdict PASS + allowlist de chemins | `operator_server.rs:602`, `ARTIFACT_DRAFT_ALLOWLIST:26` | Aucune surface ne matérialise un verdict final via l'Operator. |
| **C14** | `ACTION_ALLOWLIST` fermée : seules `status-sprint`/`lint-planning`/`audit-commit`/`prompt` | `operator_server.rs:24` + enforcement `:469` | Les actions privilégiées sont énumérées côté Rust ; l'UI ne fait que les déclencher. |
| **C15** | CSP/COOP/COEP source unique injectée sur chaque réponse blob-serve | `nexus_core_rs::csp::BLOB_SERVE_CSP` (`csp.rs:33`) : `connect-src 'none'`, `worker-src 'none'`, `frame-src 'none'`, `object-src 'none'`, `base-uri 'none'`, `form-action 'none'`, `sandbox allow-scripts` ; COOP `same-origin` (`:36`), COEP `require-corp` (`:39`) | Toute prévisualisation passe sous ce CSP, immuable depuis l'interface. |
| **C16** | gate CSP authoring déterministe Rust non-délégable | `gates.rs` `run_gate_csp_authoring` + test anti-drift cross-crate | L'interface ne remplace pas ce gate ; au mieux elle l'expose en diagnostic. |
| **C17** | anti git-option-injection / path-traversal en entrée | `operator_server.rs` (`is_safe_git_rev`, rejets `..`/drive) | Toute UI passant rev/sha/noms respecte ces validateurs serveur. |

### 2.3 Contraintes RÉVISABLES (avec coût)

| # | Contrainte | Statut réel | Coût de révision |
|---|---|---|---|
| **R1** | « pas de runtime Node persistant » | **Non littéral** : cohérence d'architecture (daemon/Operator/launcher tous Rust). Réalité : Node n'apparaît qu'en outillage **dev** (Vite). L'Operator ne sert aucun fichier statique → en prod le bundle `dist/` n'a pas de serveur Rust. **Gap ouvert.** | **Faible** via `tower_http::ServeDir`. **Élevé** si on réintroduit un serveur Node persistant (code-server/Theia). |
| **R2** | « pas de marketplace d'extensions » | **Non littéral** ; dérivé de la posture supply-chain + AGPL + solo + l'incident GlassWorm Open VSX. | **Quasi-interdit** : adopter VS Code/Open VSX réintroduit le vol-de-token GlassWorm que loopback+token+gates combattent. |
| **R4** | loopback-only (C10 conceptuellement révisable) | Tier-1 code. | **Élevé** : exposer hors loopback casse le threat-model DNS-rebinding. |
| **R5** | front actuel Base UI/Radix React 19 vs directive PO daisyUI + anime.js | Réalité : stack Radix ; directive PO `po_directive_factory_front_redesign`. | **Moyen** : refonte front, mais C7/C8/C9 inchangés ; l'Operator est outil **local** donc **hors CSP scellée** (palette libre). |

### 2.4 La membrane CSP — les 2 régimes (déterminant)

Toute interface doit distinguer deux mondes aux libertés opposées :

- **Régime 1 — l'OUTIL Factory privilégié (l'Operator lui-même)** :
  servi à un origin loopback, **hors `BLOB_SERVE_CSP`**. Peut donc :
  `fetch`/WebSocket vers l'API loopback, `localStorage`, xterm.js,
  **anime.js + daisyUI sans contrainte CSP** (cohérent directive PO),
  PTY, SSE. Doit : porter le token (proxy), respecter C12-C14,
  n'afficher que des **intentions**, ne jamais auto-clore un verdict.
- **Régime 2 — le CONTENU untrusted (app scellée)** : rendu
  **uniquement** via l'iframe blob-serve sous `BLOB_SERVE_CSP`. Aucun
  réseau, aucun worker, aucune iframe imbriquée, ES modules interdits
  (COEP `require-corp`). Seul canal = postMessage bridge.

**Conséquence cardinale** : une interface qui voudrait
« éditer-et-prévisualiser dans le même contexte privilégié » est
**structurellement impossible** — le preview DOIT traverser l'iframe
blob-serve. C'est précisément ce que rejoue le self-check viewer
(S79 Phase H).

---

## 3. Le flux réel de Factory est AGENT-NATIVE

Le point le plus important du dossier, et celui qui **change le centre
de gravité** de l'interface. Investigation lecture-seule des 12 pages
+ CLI :

**Dans le Factory tel qu'il existe, la frappe de code applicatif par
l'humain est ~0 %, et le geste « humain donne une intention / un agent
fait le travail » est ~100 %.**

- `tools/factory-operator/src/App.tsx` = **12 routes, aucune route
  éditeur**. 4 des 12 pages sont littéralement « générer un texte →
  bouton Copier → coller à un agent » (PhaseAssistant, AgentTransfer,
  ContextPackBuilder). L'humain est chef d'orchestre + relecteur, pas
  dactylo.
- L'« éditeur » de Factory **EST l'agent** : `AgentChat.tsx` (`/chat`)
  est un **terminal xterm.js plein écran** branché en WebSocket sur
  `/api/terminal/ws` ; `terminal.rs` ouvre un PTY portable et y spawn
  `claude`/`claude.cmd`. Le code est lu et écrit par les outils
  internes de Claude Code (Read/Edit/Write de l'agent). **Il n'existe
  ni CodeMirror, ni Monaco, ni arbre de fichiers, ni éditeur de diff
  dans tout `factory-operator`.**
- La fabrication d'app (idée → app → publiée vérifiable) n'est **pas
  dans l'UI** : ce sont des verbes CLI (`create`/`fork`/`redeploy`/
  `validate`/`preview`/`publish`) tapés **par l'agent dans le PTY**,
  pas par l'humain.
- La doctrine agent-native est déjà **gravée dans le code** : context-
  pack `chat_history_authoritative:false` ; prompt app-authoring
  « consumed and displayed, never authoritative » ; `CAPABILITY_BLOCK`
  « Do not assert a PASS yourself » ; `SENSITIVE_ACTIONS → requires_gate`.

### Ce que ça change : éditeur PRIMAIRE vs SECONDAIRE

1. **L'éditeur de code est SECONDAIRE.** Une interface dont le centre
   est un buffer de frappe (Monaco/CM6 + chrome IDE complet) optimise
   un geste qui n'existe quasiment pas. L'éditeur n'est nécessaire que
   comme surface d'**inspection / patch d'appoint** (corriger 3 lignes
   quand l'agent dérape).
2. **Le geste primaire = intention → steering → vérification.** Trois
   surfaces par poids : (a) capture d'intention, (b) observation/
   steering de l'agent (le PTY aujourd'hui), (c) **surface de
   vérification** (diff des éditions agent + résultats de gates +
   Proof Cards).
3. **Le vrai trou n'est pas « pas d'éditeur » — c'est « pas de surface
   de diff-review ».** Le bottleneck 2026 est la **vérification**, pas
   la frappe. C'est l'investissement d'interface le plus rentable.

### Honnêteté requise (passe adversariale)

Le keystone « ~0 % frappe humaine » comporte un **biais du survivant** :
on mesure l'absence de frappe **sur un outil dépourvu d'éditeur**. On
ne peut donc pas conclure « l'humain n'a pas besoin d'éditer » de
manière falsifiable. Conséquence pratique : la décision « éditeur
secondaire / CM6 minimal » est correcte **comme point de départ**,
mais la demande réelle de frappe manuelle **doit être mesurée** sur un
éditeur minimal, pas décrétée nulle a priori. C'est exactement
pourquoi le MVP livre un diff-viewer + patch d'appoint et **diffère**
l'IDE complet — voir §10.

---

## 4. Les 3 familles passées au crible

### 4.1 Reprendre un IDE complet (Code-OSS / VSCodium / Theia / code-server / openvscode-server) — ÉLIMINÉE

**Distinction préalable** : « source MIT » ≠ « produit distribuable ».
Le dépôt `microsoft/vscode` est MIT, mais le **binaire** VS Code est
propriétaire (télémétrie, branding, marketplace). « Reprendre un IDE »
= reprendre Code-OSS (MIT) **et reconstruire soi-même un produit** —
on hérite de toute la charge de build/rebranding, pas d'un produit
clé-en-main.

| Candidat | Licence | Node persistant | Violation décisive | Verdict |
|---|---|---|---|---|
| Code-OSS | MIT | Electron (host Node) | **C1 Tier-0** (Electron) | ÉLIMINÉ d'office |
| VSCodium | MIT | Electron | C1 + R2 (Open VSX/GlassWorm) | ÉLIMINÉ |
| Eclipse Theia | **EPL-2.0 / GPL-2.0-only** | **Oui (backend Node)** | **C4** (incompat. AGPL) + R1 + C5 + R2 | ÉLIMINÉ |
| code-server | MIT | **Oui (~1 Go RAM)** | R1 (élevé) + C5 + R2 | ÉLIMINÉ |
| openvscode-server | MIT | **Oui** | R1 + C5 + R2 + gouvernance OpenAI (Ona) | ÉLIMINÉ |
| Gitpod / Ona | proprio/SaaS | Cloud | C10 (pas loopback) + C5 + posture | ÉLIMINÉ |

**Le coût caché qui tue la prémisse « + ses plugins ».** Le cœur de
l'option est « on reprend l'IDE **avec tous ses plugins** ». **C'est
factuellement mort en 2026** : les forks sont bannis de la marketplace
Microsoft (retrait de l'extension C/C++ des forks, avril 2025), et la
seule alternative (Open VSX) est le vecteur **GlassWorm** — ver auto-
propageant actif depuis oct. 2025, ~35 800 machines, qui **vole tokens
npm/GitHub/Open VSX/Git** et republie via les credentials volés (vague
« sleeper » de 73 extensions en avril 2026). **Brancher Open VSX dans
Factory = inviter le vol de `<sbfb_home>/auth_token` + des creds GitHub
de provenance — exactement ce que loopback+token+gates existe pour
empêcher.** On ne récolte que la coquille **sans ses plugins** + la
dette de fork (rebase perpétuel sur 20M+ LOC).

**Le moins-pire théorique** est **Theia** (seul non-Electron, conçu
pour le white-label) — mais coulé par trois murs : (a) incompatibilité
licence **EPL-2.0/GPL-2.0-only ↔ AGPL-3.0** (FSF : EPL incompatible
GPLv3) ; (b) backend Node persistant (R1 élevé + C5, en doublon de
l'axum Rust) ; (c) Open VSX/GlassWorm (R2). **La seule brique
légitimement récupérable est Monaco/CM6 comme librairie MIT** — ce qui
relève de la famille 3, pas de « reprendre l'IDE ».

### 4.2 Plugin/extension d'IDE (VS Code / Zed / JetBrains) — ÉLIMINÉE comme interface ; ACP = passerelle additive différée

| Candidat | Verdict | Raison décisive |
|---|---|---|
| Extension VS Code/VSCodium | **ÉLIMINÉ** | Le webview impose **sa** CSP (≠ `BLOB_SERVE_CSP` source-unique) → le preview vérifiable n'y vit pas (C2/C15) + R1 Node persistant (extension host) + **R2 Open VSX/GlassWorm** + C5 (autorité diluée) + C1-esprit. |
| Extension Zed (WASM) | **ÉLIMINÉ (capacité)** | API volontairement bridée : « no support for modifying the UI to create new panels, or making arbitrary HTTP requests, or touching the file system ». On ne peut **pas** y loger l'UI Factory. |
| Plugin JetBrains | **ÉLIMINÉ** | C1 + **C4 le plus murky** (AGPL-dans-proprio, « effet viral » documenté par JetBrains) + R2 (marketplace à revue) + coût JVM/Gradle solo. |
| Extension Theia | **ÉLIMINÉ** | R1 (backend Node persistant = Family-1 déguisée) + friction EPL/AGPL. |
| **ACP interop (Zed/JetBrains)** | **CONDITIONNEL / DIFFÉRÉ post-1.0** | **N'est pas une extension** : un protocole (Apache-2.0) où l'agent externe possède son runtime/auth/tools ; l'éditeur héberge juste le thread. Le backend Rust souverain reste maître. **0 marketplace, 0 lock-in, 0 délégation d'autorité.** |

**Le piège séduisant (« rejoindre le dev dans son IDE ») se retourne.**
La marketplace **est** le vecteur GlassWorm (R2 anti-supply-chain) ;
le webview ne porte pas la membrane CSP scellée (C2/C15) ; l'autorité
fuit hors du Rust souverain (C5) ; l'utilisateur doit installer une app
desktop tierce télémétrée (C1-esprit). Le tout pour **zéro gain net**
sur un standalone. **Ironie centrale** : la seule chose que
l'écosystème extension fait *bien* en 2026 pour l'agent-native n'est
**pas une extension** — c'est **ACP** : l'industrie a abandonné le
modèle « plugin-pour-agent » au profit d'« agent externe maître de son
runtime » = exactement la forme PTY-Claude-Code + `provider_router` +
gates de Factory. Donc même l'industrie pointe vers **Family 3 +
interop ACP**, pas Family 2.

### 4.3 Tout autre logiciel — SEULE FAMILLE SURVIVANTE

**(a) App web sur-mesure CM6/Monaco + Rust — RECOMMANDABLE.** Servie
en loopback, client navigateur, backend axum existant. Satisfait
l'intégralité du cube (17/17 absolues + 2/2 révisables) sans réviser
un seul Tier-0/Tier-1. Sous-choix tranché :

- **Éditeur : CodeMirror 6, sans ambiguïté.** CM6 = MIT, ~50-150 KB
  tree-shakeable, **un seul auteur** (Marijn Haverbeke), arbre
  transitif quasi-nul, `@codemirror/merge` natif (= exactement le
  maillon faible diff-review). Monaco = MIT mais 5-10 MB, workers
  obligatoires, et son IntelliSense passe par `monaco-languageclient`
  = **134 dépendances** tirant un sous-arbre `vscode-*`. Le seul atout
  de Monaco (IntelliSense TS de classe IDE) sert un geste **~0 %** dans
  Factory. **Monaco est admissible mais dominé** (R2 dégradé). CM6
  gagne sur le vrai besoin **et** sur la surface supply-chain.
- **Pont LSP : faisable mais HORS MVP.** État 2026 mûr :
  `@codemirror/lsp-client` est **officiel** (Marijn, MIT) ; transport
  WS↔stdio réimplémentable en ~150 lignes Rust (pas de binaire tiers) ;
  serveurs permissifs zéro-Node (`tsgo --lsp` Apache-2.0, `ruff server`
  natif Rust, rust-analyzer MIT). **Mais** tant que l'agent + les gates
  + `cargo`/`tsc` portent l'autorité de correction, l'IntelliSense ne
  sert que l'escape-hatch « corriger 3 lignes ». À différer ; si activé
  un jour, **tsgo `--lsp` d'abord** (Node-free, éprouvé-éditeur), jamais
  rust-analyzer (RAM/lifecycle/crash).
- **Framework UI : React 19 incumbent**, re-skin daisyUI + anime.js
  (directive PO, régime 1 hors CSP). Pas de rewrite Svelte/Solid : gain
  marginal hors-CSP, coût élevé.

**(b) Paradigme nouveau agent-natif — RECOMMANDÉ (le bon centre de
gravité).** Trois variantes :

- **3b-α « Atelier conversationnel »** (intention → agent observable →
  dock de vérification diff/gates/preview) : **✅ recommandé plein.**
  Centre de gravité = le flux réel ; comble le bottleneck diff-review ;
  réutilise ~70 % du backend ; cube intact ; C7-C9 natifs.
- **3b-β « Canvas de procédé »** (idée→plan→phases→gates→provenance en
  graphe d'artefacts) : **⚠️ conditionnel.** Exprime littéralement le
  process SBFB + « artifact-as-trust » (Antigravity 2026) mais coût
  solo le plus élevé (moteur de graphe) + **risque de drift canvas↔repo
  anti-C9**. À récupérer comme **ruban de procédé read-only** en
  en-tête, **pas** comme cadre ; canvas plein différé.
- **3b-γ « TUI souverain élevé »** (PTY au centre + panneaux preview/
  gates + rejeu `.cast`) : **⚠️ conditionnel.** Coût solo le plus bas,
  lignée OpenBSD/Tor la plus pure, ~85 % déjà livré. Mais **tension C7**
  (terminal = jargon en surface primaire) et **ne comble pas** le
  bottleneck (scrollback ANSI = low-fidelity). À **embarquer comme
  sous-surface « steering profond »**, pas comme cadre unique.

**(c) Desktop natif — ÉLIMINÉE en bloc.** Tauri 2 **viole C1 (Tier-0
littéral, Tauri nommé)** ; Electron viole C1+R1+R2 ; les éditeurs
GPU-natifs (Zed/Lapce/Helix) **n'ont aucun moteur web** → preview
blob-serve impossible sans y greffer un webview (= re-violer C1) :
impasse architecturale, pas idéologique. Coûts solo : signature/
notarisation multi-OS à renouvellement **annuel dès le 15 fév. 2026**,
fragmentation 3-webviews à la charge de SBFB, inadéquation au **nœud
headless VPS** (ancré S75). Fit agent-native **nul** : Tauri n'améliore
ni le steering ni le diff-review. Les deux seuls bénéfices réels de
Tauri (« 1 geste → Factory », accès FS) sont **déjà couverts** par
`nexus-launcher` + un `ServeDir` axum.

---

## 5. Matrice de scoring (22 options × axes du cube + fit + futur + coût solo)

Axes souveraineté : **(a)** marketplace/GlassWorm · **(b)** Node
persistant · **(c)** Tauri/Electron (Day-0 C1) · **(d)** AGPL-compat ·
**(e)** hôte NOUS vs TIERS · **(f)** maintenabilité solo · **(g)**
rendu scellé blob-serve natif.

| # | Option | (a) | (b) | (c) | (d) | (e) | (f) | (g) | STATUT |
|---|---|:--:|:--:|:--:|:--:|:--:|:--:|:--:|---|
| 1 | Code-OSS | exposé | Electron | **OUI** | OK | TIERS | LOW | webview | **ÉLIMINÉE** (C1) |
| 2 | VSCodium | **OUI** | Electron | **OUI** | OK | TIERS | LOW | webview | **ÉLIMINÉE** (C1+R2) |
| 3 | Eclipse Theia | **OUI** | **OUI** | NON | **INCOMPAT** | TIERS | LOW | OUI | **ÉLIMINÉE** (C4+R1+R2) |
| 4 | code-server | **OUI** | **OUI (~1 Go)** | NON | OK | TIERS | LOW | webview | **ÉLIMINÉE** (R1+C5+R2) |
| 5 | openvscode-server | **OUI** | **OUI** | NON | OK | **TIERS (OpenAI)** | LOW | webview | **ÉLIMINÉE** |
| 6 | Gitpod/Ona | n/a | cloud | n/a | proprio | **TIERS (OpenAI)** | n/a | NON | **ÉLIMINÉE** (C10) |
| 7 | Ext. VS Code | **OUI** | OUI | esprit | RISK | TIERS | LOW | **NON (CSP host)** | **ÉLIMINÉE** |
| 8 | Ext. Zed (WASM) | NON | NON | esprit | OK | TIERS | LOW | **NON (API bridée)** | **ÉLIMINÉE (capacité)** |
| 9 | **ACP interop** | **NON** | tension (adaptateur Node) | NON | OK | **NOUS maître** | MED | OUI (preview côté SBFB) | **CLEAN — additif/DIFFÉRÉ, PAS l'interface** |
| 10 | Plugin JetBrains | **OUI** | OUI (JVM) | esprit | **RISK max** | TIERS | LOW | NON | **ÉLIMINÉE** |
| 11 | Ext. Theia | **OUI** | **OUI** | NON | RISK | TIERS | LOW | OUI | **ÉLIMINÉE** (R1+C4) |
| 12 | **CM6 + Rust axum** | **NON** | **NON (dev-time)** | **NON** | OK | **NOUS** | **HAUTE (~80 %)** | **OUI natif** | **✅ CLEAN — SURVIT (socle)** |
| 13 | Monaco + Rust | NON | NON | NON | OK | NOUS | MED (134-dep) | OUI | **CLEAN mais DOMINÉ** |
| 14 | **3b-α Atelier conversationnel** | **NON** | **NON** | **NON** | OK | **NOUS** | **HAUTE (~70 %)** | **OUI natif** | **✅ CLEAN — RECOMMANDÉ** |
| 15 | 3b-β Canvas de procédé | NON (+dep graphe) | NON | NON | OK | NOUS | MED-LOW (drift C9) | OUI | **CLEAN — ruban-only, canvas différé** |
| 16 | 3b-γ TUI souverain élevé | **NON** | **NON (PTY transitoire)** | **NON** | OK | **NOUS** | **HAUTE (~85 %)** | OUI | **CLEAN — sous-surface (tension C7)** |
| 17 | Tauri 2 | NON | NON | **OUI — rompt C1** | OK | NOUS (2ᵉ surface sécu) | MED-LOW | OUI | **révision-Day-0-requise → ÉLIMINÉE** |
| 18 | Electron | **OUI** | **OUI** | **OUI** | OK | NOUS | LOW | OUI | **ÉLIMINÉE (C1+R1+R2)** |
| 19 | Fork Zed | NON | NON | NON | OK | **TIERS (~500k LOC)** | LOW | **NON (0 webview)** | **ÉLIMINÉE (C2/C15)** |
| 20 | Fork Lapce | NON | NON | NON | OK | TIERS (pré-1.0) | LOW | **NON (0 webview)** | **ÉLIMINÉE (C2/C15)** |
| 21 | Helix | NON | NON | NON | OK | TIERS | n/a | **NON (TUI)** | **ÉLIMINÉE (0 surface)** |

**Synthèse transverse : 18 des 22 options éliminées sur souveraineté**
(9 par marketplace/GlassWorm, 4 par Day-0 C1 littéral, 3 par incompat.
licence/Node, 3 par impossibilité preview blob-serve). **Tauri = la
seule « révision-Day-0-requise », non justifiée.** Survivantes
Day-0-clean : #12/#14/#16 (l'interface, même famille 3 web) + #15
(emprunt ruban) + #9 (ACP passerelle post-1.0). #13 Monaco admissible
mais dominé.

---

## 6. Le futur agentique (quel paradigme vieillit le mieux)

Le marché 2026 a basculé **exactement** sur le flux Factory et converge
sur **la vérification comme surface centrale** — pas l'éditeur. Sept
vecteurs datés :

1. **Le centre passe de la frappe à l'orchestration — chiffré.**
   Cursor 3 (avril 2026) = « agent-first interface, beyond the IDE
   model ». L'inversion d'usage est la preuve dure : mars 2025 la
   tab-completion avait 2,5× plus d'utilisateurs que les agents ; **ce
   ratio s'est inversé — les agents en ont 2×**.
2. **« L'IDE devient un anachronisme »** : « less like an editor and
   more like a control center for managing autonomous engineering
   agents ».
3. **La vérification/relecture devient le travail de premier rang** —
   le goulot 2026 (PR assistées 5,3× plus longues à reprendre ; revue
   d'une suggestion IA 4,3 min vs 1,2 min code humain).
4. **Background / ambient agents montent** (Claude Code `--bg`/
   Routines, loop-engineering : « I don't prompt Claude anymore… My
   job is to write loops »).
5. **La confiance est un cadran, pas un interrupteur** — co-construite
   = **exactement `SENSITIVE_ACTIONS → requires_gate` + action-log de
   SBFB**, la couche de gouvernance que Cursor/Windsurf/Devin n'ont
   **pas**.
6. **Le backend agent se découple de la surface via protocole** (ACP,
   « LSP for AI coding agents », 25+ agents mars 2026) — mais c'est une
   **guerre de protocoles ACP↔MCP**, spec jeune.
7. **Le coding-local monte, l'agentique-autonome-repo-scale reste
   cloud ~2028-2031** → le **provider-agnosticisme** (déjà câblé :
   `provider_router` Claude/Ollama/Network) devient un trait de
   durabilité.

**Contre-courant honnête** : un backlash anti-agent-first réel
(Antigravity perf/auto-update ; Cursor 3 « abandonne l'identité IDE » ;
taxe cognitive du context-switch). **Mais ce backlash ne ramène pas
vers l'éditeur — il revalide le TUI** (« agent-native workflows need
terminals, logs, bash ; the center of gravity has shifted to the CLI
and TUI »). Donc paradigme D (terminal souverain), déjà livré.

**Classement par robustesse-au-futur :**

| Rang | Paradigme | Vieillit | Pari = | Risque dominant |
|---|---|---|---|---|
| **1** | **Control-center agent-natif** (3b-α) | **Le mieux** | la direction où TOUT le champ converge | immaturité *de surface* (stream-json), chat-fatigue, sur-scope orchestration |
| **2** | **Terminal/PTY souverain** (3b-γ) | **Bien (plancher anti-regret)** | substrat revalidé 2026 + **déjà livré** | tension C7 ; ne comble pas le diff-review |
| **3** | **Graphe de procédé / spec-driven** (3b-β) | **Moyennement** (concept oui, canvas non) | la traçabilité idée→preuve | coût graphe + drift repo (anti-C9) |
| **4** | **Éditeur-au-centre** (IDE classique) | **Le moins (pari 2015)** | optimiser la frappe humaine | construire ce que le marché abandonne ; LSP = coût pur pour geste ~0 % |
| hors-rang | **Desktop natif** (Tauri/Electron) | **Le pire** | un binaire installé lourd | mauvaise forme pour un monde background/headless ; tué par C1 |

**Réponses directes :**
- *« Un IDE classique centré-éditeur est-il un pari 2015 ? »* → **OUI,
  décisivement et chiffré.** Le *composant* éditeur (diff CM6) survit
  rétrogradé ; c'est le *paradigme éditeur-au-centre* qui est le pari.
- *« L'agent-natif est-il l'avenir mais immature ? »* → **OUI à
  l'avenir** (l'axe le mieux daté), **immature seulement dans la
  surface** (flux d'édition structurés, fidélité diff, guerre ACP/MCP),
  **pas dans la direction**. Mieux : la partie immature *chez les
  concurrents* (gouvernance) est *déjà mûre chez SBFB*. **SBFB est en
  avance sur l'axe durable (gouvernance/preuve), en retard uniquement
  sur la surface diff-review.**

**L'insight cardinal de durabilité dépasse le choix de cadre** : la
décision la plus future-proof pour un solo n'est pas « quelle surface »
mais **« découpler le backend-agent de la surface »**. Ce qui vieillit,
c'est le couplage ; ce qui survit, c'est le **contrat `intention → flux
d'édition structuré → vérification (diff repo + gates Rust)`**. SBFB en
détient déjà 70-85 %.

---

## 7. Recommandation finale

### 7.1 LE paradigme

**Construire un « Atelier conversationnel » agent-natif (3b-α) comme
cadre**, où l'agent est PRIMAIRE et l'éditeur SECONDAIRE. Surface
tri-zone :

- **Gauche — Intentions** : presets + NL (réutilise
  `ContextPackBuilder`/`PhaseAssistant`/`AgentTransfer` +
  `/api/prompt/{kind}`, `/api/context-pack`). 0 jargon en CTA (C7) ;
  le jargon `kind/provider/preflight` reste replié (pattern
  `<TechnicalDetails>` existant).
- **Centre — Fil d'atelier observable** : chat SSE provider-routé
  (`provider_router::ExecutionTarget{Claude,Ollama,Network}`) +
  **cartes diff/tool-call** des éditions agent. Bouton preview =
  **iframe blob-serve sous `BLOB_SERVE_CSP`** (C2, régime 2). Les
  boutons de relecture `[✓]/[✗]` = **intention de relecture, JAMAIS un
  verdict** (C8) ; le serveur refuse de toute façon (`:752/:854/:952`).
- **Droite — Dock de vérification (le neuf)** : gates `gates.rs`
  (FG4/FG5/FG6/FG-CSP/FG7/FG8) en diagnostic + Proof Card brouillon ;
  `commit/push/PASS` → `requires_gate` (la gouvernance que les ADE
  2026 n'ont pas).
- **Onglet « steering profond » = 3b-γ** : le PTY xterm.js + sessions
  `.cast` **déjà livrés** (`terminal.rs`), pour l'expert.
- **Ruban de procédé = emprunt 3b-β** : projection **read-only** du
  repo en en-tête (`Idée ▸ Plan ▸ Phase ▸ Publish`), **pas** de
  canvas-graphe.

### 7.2 LA stack concrète

| Couche | Choix | Justification |
|---|---|---|
| Backend | **axum Operator existant** (`operator_server.rs`/`auth.rs`/`gates.rs`/`terminal.rs`/`provider_router.rs`/`pipeline.rs`) | ~70-85 % déjà câblé, testé, loopback+token. |
| Service statique | **`tower_http::ServeDir`** (Rust) servant `dist/` | Ferme le **seul gap R1** ; sort Node du chemin prod **sans** runtime Node persistant. |
| Front framework | **React 19 incumbent** | 12 pages existantes ; refonte = re-skin, pas rewrite. |
| Look | **daisyUI 5.6 + anime.js 4.5** (MIT) | Directive PO ; régime 1 **hors CSP scellée** → palette libre. |
| Éditeur | **CodeMirror 6 (MIT)** + `@codemirror/merge`, en surface **secondaire** | Léger (~50-150 KB), un-auteur, arbre transitif quasi-nul, diff natif. **Pas Monaco** (5-10 MB, 134-dep `monaco-languageclient`, IntelliSense pour un geste ~0 %). |
| Diff | **calculé en Rust (`git diff`)**, rendu en cartes daisyUI | Le repo = vérité (C9) ; robuste, **0 dépendance** à un format de stream. |
| Enrichissement temps-réel | `stream-json` (Claude Code headless) **optionnel non-bloquant** | Subprocess transitoire (comme `git`), **pas** de serveur Node persistant (R1 préservé). Repli git-diff si le format n'est pas confirmé. |
| Steering | **PTY xterm.js + Claude Code** (déjà livré) | L'« éditeur agent » réel. |
| Preview | **iframe blob-serve** sous `BLOB_SERVE_CSP` | C2/C15 non négociables. |
| LSP / IntelliSense | **EXCLU du MVP**, différé | Si un jour : `tsgo --lsp` (Node-free, RC éprouvé) d'abord, jamais rust-analyzer. |
| Desktop natif / Tauri | **EXCLU** | Viole C1 (Tier-0). |
| Marketplace / Open VSX | **EXCLU** | R2 quasi-interdit (GlassWorm). |

### 7.3 Pourquoi elle bat les autres

1. **Cube intact** : seule famille à 17/17 absolues + 2/2 révisables,
   sans réviser un seul Tier-0/Tier-1. Familles 1/2/3c violent une
   Day-0 littérale ou réintroduisent GlassWorm.
2. **Fit agent-native maximal** : matérialise le flux réel mesuré
   (intention > steering > vérification) ; comble le **bottleneck
   2026** (diff-review) là où SBFB était en retard, et capitalise son
   avance de gouvernance (gates + 0-auto-PASS).
3. **Future-proof** : le contrat `intention → flux structuré → diff
   repo + gates` survit au churn modèle/agent 2026→2028 ; le
   provider-agnosticisme et la gouvernance-cadran sont déjà câblés.
4. **Soutenable solo** : réutilise ~70-85 % du backend ; aucun rebase
   d'IDE, aucun second serveur, aucune décision marketplace, aucune
   signature multi-OS.
5. **Souveraine** : autorité 100 % dans le crate Rust, lignée
   OpenBSD/F-Droid/Tor préservée.

### 7.4 Trancher CM6 vs Monaco

**CodeMirror 6.** Dans un flux où l'humain ne tape ~aucun code
applicatif et où la valeur est le **diff-review**, payer 5-10 MB +
134 dépendances `vscode-*` + config worker pour une IntelliSense
quasi-inutilisée est un anti-pattern. CM6 gagne sur le vrai besoin
(`@codemirror/merge`) **et** sur la surface supply-chain (un auteur,
arbre transitif nul). Et même CM6 reste **secondaire** : la vérité du
diff est calculée en Rust ; CM6 ne sort que pour l'**escape-hatch
d'édition manuelle**.

---

## 8. Angles morts & risques (assumés, depuis l'adversarial)

1. **Biais du survivant sur le keystone « ~0 % frappe »** (M2) : la
   mesure se fait sur un outil sans éditeur. **Mitigation** : livrer un
   éditeur minimal (CM6 lecture + patch d'appoint) et **mesurer** la
   demande réelle de frappe ; ne pas décréter le besoin nul. Interdire
   par scope-cut écrit : FS-watch multi-fichiers, onglets, LSP,
   find-references tant que la demande n'est pas mesurée.
2. **Le différenciateur dépend d'un format non garanti** (D1) : la
   carte tool-call/diff « riche » repose sur `claude --output-format
   stream-json`, dont le schéma exact n'est **pas vérifié**. **Sans
   stream-json, l'atelier α = un visualiseur de git-diff web + le PTY
   existant + un panneau gates** — incrémental, pas paradigm-shifting.
   **Mitigation dure** : `git diff` Rust comme **plancher robuste**
   (repo = vérité) ; stream-json en enrichissement non-bloquant à
   spiker en preflight.
3. **Gradient de scope-creep le plus raide du dossier** : « juste la
   vue diff » → sauvegarde → FS-watch → reload → conflit deux-écrivains
   → onglets → recherche → arbre = **un IDE rampant**. La synchro
   éditeur↔agent est sous-tarifée (Cursor/Zed y ont brûlé un effort
   VC). **Mitigation** : diff en Rust (pas de buffer périmé), CM6
   strictement escape-hatch mono-fichier.
4. **Pari-paradigme solo à contre-standardisation** (D2) : ossifier en
   2026 une UI agent-native bespoke pendant qu'ACP se standardise.
   **Mitigation** : ne **PAS** loger l'autorité dans ACP maintenant ;
   le traiter comme **port d'interop additif post-1.0** (guerre de
   protocoles + transport HTTP encore « proposition »).
5. **Lock-in Claude Code propriétaire vs souveraineté** (D3, l'angle
   mort le plus stratégique) : l'authoring réel est couplé à la CLI
   d'Anthropic. **Mitigation** : garder la surface model-agnostique via
   `provider_router` ; la valeur durable est le *contrat
   intention→stream→verify*, invariant au modèle qui le produit.
6. **Coût de sécurité marginal sur l'origin le plus privilégié** (D4) :
   ajouter `/api/fs/write` + `/api/git/diff` étend la surface d'attaque
   du process token-gardé (équivalent-RCE si le token fuit). **Ne pas
   ajouter légèrement de l'écriture FS** : réutiliser les validateurs
   anti-traversal de `artifact-draft` (`:546-562`), garder l'écriture
   minimale et allowlistée.
7. **Membrane CSP à ne jamais franchir** : toute tentation d'un
   « preview inline rapide » dans le contexte privilégié viole C2 ; le
   preview reste iframe blob-serve scellé.
8. **Risque UX C8** : un utilisateur peut croire que `[✓]` « valide ».
   Libeller en intentions (« transmettre à la session pour commit sous
   gate »), jamais « valider/PASS ».

**Le risque dominant du projet n'est pas de choisir la mauvaise
famille** (la famille 3 est juste). **C'est de sur-investir dans le
paradigme excitant (atelier bespoke + IDE CM6 complet) avant d'avoir
falsifié le keystone biaisé et vérifié le levier porteur (stream-json).**

---

## 9. Chemin de mise en œuvre

> **Rappel de discipline** : tout sprint commence par sa **Phase 0 =
> audit gate** du sprint précédent (P0/P1 bloquants), et chaque phase
> suit le process per-phase complet (deep preflight 5 scans → code →
> review → Codex → commit atomique → post-commit), gate de testabilité
> T1 E2E hermétique + T2 artefact JSON au wrap-up.

**MVP minimal viable (increment lean — l'hypothèse de moindre regret) :**

1. **Backend (Rust, marginal)** : ajouter `tower_http::ServeDir` au
   routeur Operator pour servir `dist/` → **Node sort du chemin prod
   (clôt R1)**. Réutiliser tel quel context-pack, chat SSE, PTY WS,
   gates, action-log.
2. **Routes neuves minimes** : `/api/fs/tree` + `/api/fs/read`
   (lecture, allowlistée au workspace, validateurs anti-traversal
   existants) ; `/api/git/diff` (diff des éditions agent, calculé en
   Rust) ; optionnel `/api/fs/write` gardé (mêmes validateurs que
   `artifact-draft`).
3. **Front (React 19 re-skiné daisyUI + anime.js)** — 3 zones :
   intention (réutilise ExecutionChat/PhaseAssistant), steering (PTY
   xterm existant + chat SSE), **vérification (le neuf)** = arbre
   read-only + vue diff (cartes daisyUI depuis `git diff` Rust ; CM6
   pour l'escape-hatch) + panneau gates/Proof Cards + preview iframe
   blob-serve.
4. **Ruban de procédé** : projection read-only du repo en en-tête.

**Différable (PROVISIONAL, assumé) :**
- Tout **LSP/IntelliSense** (commencer par `tsgo --lsp` Node-free si
  jamais activé).
- **stream-json** (enrichissement temps-réel ; le MVP tient sur
  git-diff Rust seul).
- L'**arbre de fichiers éditable complet**, multi-onglets, recherche
  façon IDE.
- Le **canvas-graphe plein** (3b-β) — garder le ruban, différer le
  graphe jusqu'à preuve empirique (idea-hub post-S77).
- **ACP interop** (post-1.0, passerelle additive, jamais la maison de
  l'autorité).

**Pré-requis adversarial à lever en preflight de Phase A :** spike du
schéma `stream-json` (confirmer le format ou acter le repli git-diff) ;
décision écrite du scope-cut anti-IDE-rampant ; mesure instrumentée de
la demande de frappe manuelle sur l'éditeur minimal.

---

## 10. Questions ouvertes PO + Day-0 à confirmer/réviser

**Day-0 : aucune révision requise.** La recommandation (web sur-mesure
loopback + Rust) franchit le cube intact. À confirmer explicitement,
pour mémoire :

1. **C1 « pas Tauri / browser = client » reste gelé ?** → La reco le
   **respecte**. Si jamais le PO voulait un binaire double-cliquable
   « non-technicien », il faudrait **rouvrir formellement la Day-0
   C1** : coût = vote PO + fit agent-native **nul** (Tauri n'améliore
   ni steering ni diff) + signatures multi-OS annuelles (Apple 99 $/an,
   Windows OV ~216 $/an ou EV 300-500 $/an, certs plafonnés à 1 an dès
   le 15/02/2026) + 2ᵉ surface sécurité + inadéquation au nœud headless
   VPS. **Recommandation : ne pas rouvrir** ; `nexus-launcher` +
   `ServeDir` couvrent l'UX « 1 geste → Factory » sans casser C1.
2. **R1 « pas de Node persistant »** (révisable, non littéral) : la reco
   ferme le gap par `ServeDir` Rust, **Node devient purement dev-time**.
   Confirmer que c'est la lecture canonique (vs réintroduire un serveur
   Node, coût élevé rejeté).
3. **R5 refonte daisyUI + anime.js** : déjà mandatée
   (`po_directive_factory_front_redesign`) → l'Operator est outil
   **local** donc **hors CSP scellée** (palette libre). Confirmer
   re-skin React vs rewrite (reco : re-skin, pas rewrite).

**Questions ouvertes PO (arbitrages produit, pas techno) :**

- **Profondeur de la surface diff au MVP** : unified diff lisible +
  accept/reject d'intention seulement, ou multi-fichiers/syntax-
  highlight d'emblée ? (Reco : MVP minimal, mesurer.)
- **Statut PROVISIONAL de l'atelier-paradigme** : acter que l'« atelier
  α » complet + l'éditeur CM6 + le levier stream-json sont **différés/
  PROVISIONAL** jusqu'à (a) schéma stream-json vérifié, (b)
  standardisation ACP décantée, (c) demande de frappe **mesurée** — ou
  pousser le paradigme complet dès le sprint front ?
- **ACP post-1.0** : ouvrir un endpoint compatible ACP comme passerelle
  interop additive après le tag (cohérent avec la décision mémoire
  « alliance PR-plugin DIFFÉRÉE post-1.0, 0 surface plugin,
  mono-auteur »), ou rester strictement sur l'interface native ?
- **Lien idea-hub post-S77 ↔ ruban/canvas de procédé** : le canvas-
  graphe complet (3b-β) ne se justifie qu'avec l'idea-hub — l'arbitrer
  à ce moment, pas maintenant.

---

### Fichiers-ancres (absolus) vérifiés in-code

- `C:\Users\FlowUP\Documents\Code\nexus\CLAUDE.md` (461-514 gelées ;
  **472 « pas Tauri, browser = client »** ; 467 AGPL ; 474 OS sandbox ;
  468-470 blob-serve/bridge ; 477 Factory hors daemon ; 502-510
  intentions/0-auto-PASS/context-pack ; 523-549 pre-launch)
- `crates\sbfb-factory\src\operator_server.rs` (**17 `tower_http::cors`
  seul → ServeDir absent = gap R1** ; 24 `ACTION_ALLOWLIST` ; 26
  `ARTIFACT_DRAFT_ALLOWLIST` ; 35 `SENSITIVE_ACTIONS` ; 469/602/752/
  854/952 gates)
- `crates\sbfb-factory\src\auth.rs` (token CSPRNG / Host / Origin)
- `crates\sbfb-factory\src\terminal.rs` (PTY portable spawn `claude`,
  asciicast `.cast`, resume, liste sessions)
- `crates\sbfb-factory\src\gates.rs` (`run_gate_csp_authoring` +
  anti-drift cross-crate) · `provider_router.rs`
  (`ExecutionTarget{Claude,Ollama,Network}`)
- `crates\nexus-core-rs\src\csp.rs` (**33 `BLOB_SERVE_CSP` source
  unique** : `connect-src 'none'`… ; 36 COOP ; 39 COEP)
- `crates\nexus-shell-daemon-core\src\blob_serve.rs` (ré-export CSP)
- `tools\factory-operator\src\App.tsx` (**12 routes, 0 éditeur**) ·
  `pages\AgentChat.tsx` (PTY workhorse) · `package.json` (**xterm v6
  présent, 0 CM6/Monaco** ; Base UI/Radix React 19) · `vite.config.ts`
  (proxy token)

### Sources web (juin 2026, vérifiées)

GlassWorm/Open VSX : [TheHackerNews 03/2026](https://thehackernews.com/2026/03/glassworm-supply-chain-attack-abuses-72.html) ·
[BleepingComputer](https://www.bleepingcomputer.com/news/security/self-spreading-glassworm-malware-hits-openvsx-vs-code-registries/) ·
[Socket.dev](https://socket.dev/blog/open-vsx-transitive-glassworm-campaign) ·
[SecurityBoulevard sleeper 04/2026](https://securityboulevard.com/2026/04/glassworm-malware-attacks-return-via-73-openvsx-sleeper-extensions/).
Marketplace/forks : [MS Marketplace ToU](https://cdn.vsassets.io/v/M253_20250303.9/_content/Microsoft-Visual-Studio-Marketplace-Terms-of-Use.pdf) ·
[TheRegister C/C++ retiré des forks](https://www.theregister.com/2025/04/24/microsoft_vs_code_subtracts_cc_extension/).
Licences : [microsoft/vscode MIT](https://raw.githubusercontent.com/microsoft/vscode/main/LICENSE.txt) ·
[VS Code proprio](https://code.visualstudio.com/license) ·
[Theia EPL/GPL](https://open-vsx.org/api/eclipse-theia/builtin-extension-pack/1.95.3/file/LICENSE.txt) ·
[FSF EPL incompatible](https://gplv3.fsf.org/wiki/index.php/Compatible_licenses) ·
[npm CodeMirror MIT](https://www.npmjs.com/package/codemirror) ·
[PkgPulse Monaco vs CM6](https://www.pkgpulse.com/guides/monaco-editor-vs-codemirror-6-vs-sandpack-in-browser-2026).
LSP 2026 : [@codemirror/lsp-client officiel](https://discuss.codemirror.net/t/codemirror-lsp-client/9309) ·
[TS 7.0 RC](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0-rc/) ·
[PkgPulse tsgo](https://www.pkgpulse.com/guides/tsgo-vs-tsc-typescript-7-go-compiler-2026) ·
[ruff server Rust](https://astral.sh/blog/ruff-v0.4.5).
Paysage agentique : [InfoQ Cursor 3 agent-first](https://www.infoq.com/news/2026/04/cursor-3-agent-first-interface/) ·
[Builder.io agentic IDE/control-center](https://www.builder.io/blog/agentic-ide) ·
[Coder — Is the IDE Dead?](https://coder.com/blog/is-the-ide-dead-the-rise-of-agentic-ai-in-software-development) ·
[Augment ADE vs Agentic-IDE](https://www.augmentcode.com/guides/agentic-ide-vs-agentic-development-environment) ·
[Zed ACP](https://zed.dev/acp) · [JetBrains ACP](https://www.jetbrains.com/acp/) ·
[ACP vs MCP protocol war](https://www.contextstudios.ai/blog/acp-vs-mcp-the-protocol-war-that-will-define-ai-coding-in-2026) ·
[RedMonk — CLI/TUI metal](https://redmonk.com/kholterhoff/2025/12/22/10-things-developers-want-from-their-agentic-ides-in-2025/) ·
[SRLabs verification bottleneck](https://srlabs.de/blog/ai-verification-bottleneck) ·
[Google Antigravity](https://developers.googleblog.com/build-with-google-antigravity-our-new-agentic-development-platform/).
Desktop : [Tauri v2](https://v2.tauri.app/blog/tauri-20/) ·
[Zed v1.0 — The Register](https://www.theregister.com/software/2026/04/30/zed-team-releases-version-10-of-rust-built-editor/) ·
[Apple Developer ID](https://developer.apple.com/developer-id/) ·
[code-signing certs 1-an dès 02/2026](https://codesigncert.com/blog/code-signing-certificate-cost).
