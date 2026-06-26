# Quel est le meilleur front pour Factory ? — et la thèse « Viewer seul à travers Factory »

> **[MISE À JOUR — DÉCISION PO 2026-06-26 — piste reportée]**
> Le **corps de S80 = Operator greenfield** (refonte de `tools/factory-operator`),
> **PAS le Viewer**. Ce document reste l'**analyse de référence du Viewer scellé**, mais
> le Viewer est **reporté** (candidat S81+ ; il rouvrirait le P1 `app-authoring in-vivo`,
> ~70 % fermé). La base stack S80 = `factory_front_greenfield_blueprint.md` (doc 3) — voir
> son banner pour les corrections PO (**shadcn NON exclu** ; **une vraie lib motion
> [motion/anime.js] EST voulue**). Phase 0 = audit gate S79 d'abord.

**Recherche ultracode — 2026-06-26**
**Méthode** : Workflow Establish + Explore + Verify + Synthèse. 4 enquêtes factuelles (G1 surfaces & état, G2 data-flow scellé, G3 stack app scellée, G4 réalité Operator), 3 explorations (X1 design/UX viewer-preuve, X2 thèse Viewer-à-travers-Factory, X3 alternatives adversariales), 1 fact-check adversarial (V1, re-vérification des 3 claims décisifs sur le code réel).
**Question** : objectivement, quel est le meilleur front pour Factory, et faut-il « créer le Viewer SEUL, à travers Factory » (produire le Factory Viewer comme vraie app SBFB scellée, authored via le pipeline Factory — le dogfood ultime) ?
**Périmètre** : lecture seule du code réel (cwd = racine repo). Tous les chemins de fichiers sont absolus depuis la racine. Aucun fichier modifié.

---

## Réponse courte

**Oui au « Viewer à travers Factory » — mais comme corps de S80 APRÈS la Phase 0 (audit gate S79), et RECADRÉ honnêtement.** C'est objectivement le meilleur prochain move substantiel : il referme *partiellement* le carry P1 `app-authoring in-vivo Not evidenced`, produit la **1re vraie app Factory-authored** (l'actuel `examples/sbfb-factory-viewer` est du vanilla hand-authored, sans `factory.template.lock` ni `provenance.json` → ce n'est PAS un produit Factory), et fabrique la **surface-preuve publique** sur laquelle repose toute la thèse « source vérifiable ». Il bat décisivement la refonte Operator (qui n'ajoute aucune capacité et ne touche aucun P1).

**Mais l'adversarial révèle un angle mort sérieux qu'il faut nommer : un Viewer scellé est data-affamé pour le SEUL contenu qui le rendrait spécifiquement « Factory ».** La CSP `connect-src 'none'` interdit tout fetch ; aucune méthode bridge n'expose les artefacts de processus Factory. Donc un Viewer scellé ne peut afficher le processus Factory qu'en **snapshot figé au publish**, jamais en live. Le « tableau de bord live de processus » reste architecturalement le rôle de l'**Operator** (outil local, hors CSP). Correction : ancrer le Viewer sur **(1) vitrine réseau LIVE via bridge + (2) preuve-de-processus SNAPSHOT signée embarquée**, authorer via le **prompt-kind `app-authoring` + copilote Ollama** (sinon la moitié « efficacité générative » du carry reste ouverte), et cadrer le segment **cross-pair en T2 honnête** (`RIG-ABSENT` si pas de 2e machine).

---

## 1. Les deux surfaces Factory (à ne jamais confondre)

Le débat « front de Factory » porte en réalité sur **deux surfaces de nature opposée**, plus un socle partagé qui est aujourd'hui du code mort.

### 1.1 Factory OPERATOR — outil de dev local privilégié (NON scellé)

- **Localisation** : `tools/factory-operator/` (app React/Vite, front :5174) + crate `sbfb-factory operator serve` (`crates/sbfb-factory/src/operator_server.rs`, API loopback :3001 + token).
- **Nature** : outil de dev **local privilégié, qu'on fait confiance**. PAS d'iframe, PAS de CSP, PAS d'origine opaque. Peut lancer des actions gated, spawner des agents, exposer un terminal `xterm` (PTY WebSocket `/api/terminal/ws`).
- **Stack réelle** : React 19.2 + Vite 8 + TypeScript 5.9 + `react-router-dom` 7 (12 routes) + i18next (FR/EN) + Tailwind v4. **Le comportement a11y-critique passe par Base UI** (`@base-ui/react`, le successeur de Radix par la même équipe) via 11 shims maison `src/components/ui/*` (dialog/menu/select/tabs/tooltip/scroll-area…), PAS par les `@radix-ui/*` du `package.json` qui sont **du code mort** (grep `radix` sur `src/` = 0 hit). Thème = CSS-vars maison GitHub-dark dans `src/index.css`. **daisyUI absent, anime.js absent.**
- **Densité** : 12 pages routées (`SprintOverview, AgentSelector, PhaseAssistant, LintOperator, CommitAuditor, AgentTransfer, ContextPackBuilder, ActionCenter, ExecutionChat, AgentChat, ActionLog, SprintHistory`) + Sidebar + StatusBar ; `ExecutionChat` = SSE 3-intentions, `AgentChat` = dashboard live + xterm. **C'est un IDE-tool dense, stateful, temps-réel.**
- **Gating réel** : actions allowlistées = `["status-sprint","lint-planning","audit-commit","prompt"]` (`operator_server.rs:24`) ; artifact-draft borné à `.planning/active/` (`:26-27`). **Aucune action `create`/`publish`/`validate`** : l'authoring d'app se pilote par le terminal / agent-chat, pas par un bouton dédié.

### 1.2 Factory VIEWER — vraie app SBFB SCELLÉE (read-only, consultation/preuve)

- **Localisation** : `examples/sbfb-factory-viewer/` (`index.html` + `app.js` + `style.css` + `sbfb-bridge.js` + `SBFB.json`).
- **Nature** : app SBFB **scellée**. Rendue par le shell Browse (`web/src/pages/BrowsedProject.tsx:605`) dans un iframe `sandbox="allow-scripts"` **SANS** `allow-same-origin` (origine opaque), CSP `connect-src 'none'`. Lecture seule stricte. Communique uniquement par le bridge postMessage.
- **État réel** : vanilla JS hand-authored (daté mai-juin), **PAS React**, **0 daisyUI / 0 anime.js**, **AUCUN `factory.template.lock` ni `factory.provenance.json`** → **ce Viewer n'a JAMAIS été produit par le pipeline Factory**. C'est un exemple écrit à la main, et — point décisif de la section 4 — **il est cassé** contre le daemon d'aujourd'hui.

### 1.3 Le socle partagé `tools/factory-ui/src/readonly` — aspirationnel, code MORT

- `@sbfb/factory-ui` exporte `./readonly` (`ProofCard`, `SprintTimeline`, `StatusBadge`, `VerdictChip`, `PreviewList`, `ChangelogPanel`, `labels.ts`, `types.ts`) et `./operator` (api-client :3001). Classes Tailwind à hex codés en dur, **0 daisyUI**.
- **Constat factuel fort** : grep `factory-ui` hors `tools/factory-ui/` → **aucun import de code** (les 26 hits sont tous des docs/planning, jamais un `import` dans `web/`, `examples/` ou `tools/factory-operator/`). **Personne ne consomme ce socle.** Ni le Viewer (vanilla JS), ni l'Operator (qui a ses propres `components/ui` Base UI). Le « socle partagé réutilisé par les deux surfaces » de `CLAUDE.md` est aujourd'hui **purement aspirationnel** — c'est le G10/P1 de l'audit S70 jamais refermé. **Ce n'est pas un actif sur lequel s'appuyer ; c'est de la dette à trancher.**

---

## 2. La vérité du data-flow scellé — LE pivot technique

C'est ici que se joue tout : ce qu'un Viewer scellé **peut** être est entièrement dicté par la CSP. À trancher AVANT de coder.

### 2.1 La CSP réelle (source unique)

`crates/nexus-core-rs/src/csp.rs:33`, `BLOB_SERVE_CSP`, injectée sur chaque réponse blob-serve :

```
default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:; connect-src 'none';
worker-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none';
form-action 'none'; frame-ancestors *; sandbox allow-scripts
```

+ iframe `sandbox="allow-scripts"` sans `allow-same-origin` → **origine opaque**.

**Point cardinal souvent mal compris** : `connect-src 'none'` bloque **TOUT** `fetch`/`XHR`/`WebSocket`/`EventSource`/`sendBeacon`, **y compris vers sa propre origine**. Un Viewer scellé ne peut donc **PAS** faire `fetch('./data.json')` sur un fichier de sa propre archive.

### 2.2 Les deux seules portes d'entrée de données

La donnée n'entre dans l'iframe que par **exactement deux portes** :

1. **Le bridge postMessage** vers le host (canal CSP-immune). Le host (shell React, origine daemon, porteur du bearer token) `authFetch` POUR l'iframe et répond par postMessage corrélé. Whitelist d'enum `BridgeMethodSchema` (`web/src/bridge/protocol.ts:20-49`), dispatch host-side `web/src/bridge/useBridge.ts:226-416`. **16 méthodes** : `task_submit, storage_get, storage_set, pii_redact, storage_list, storage_delete, identity_pubkey, node_status, browse_list, storage_version, provenance_get, provenance_verify, feed_cursor_get, search, proof_card_get, task_result`.
2. **Des fichiers JS embarqués dans l'archive au publish** (gouvernés par `default-src 'self'` → un `<script src="…">` local est chargeable ; `'unsafe-inline'` autorise l'inline). Figés au publish, hashés par content-addressing.

### 2.3 Le constat qui décide tout : aucune méthode bridge n'expose le PROCESSUS Factory

Vérifié exhaustivement sur les 16 méthodes (V1, re-confirmé) :

- **Donnée RÉSEAU / PROVENANCE = LIVE et disponible aujourd'hui** : `browse_list` (`/api/daemon/browse`), `search` (FTS5), `proof_card_get` (provenance réseau SLSA L1 / curateurs), `provenance_get/verify`, `node_status`, `identity_pubkey`. Reflète l'état courant du daemon local **au moment du rendu** (pas au publish). Zéro changement protocole.
- **Donnée de PROCESSUS Factory = INATTEIGNABLE en live** : sprint timeline, phases, verdicts de phase, résultat Codex, kickoff/plan, audit findings. **Aucune** des 16 méthodes ne l'expose.
  - `storage_get` (`useBridge.ts:264`) → `/app/{appName}/state/{key}` : stockage **app-privé**, le Viewer ne lit que ce qu'il a lui-même écrit.
  - `proof_card_get` (`:398`) → provenance **réseau** d'une app publiée, **pas** l'historique d'un run Factory.
  - L'API `sbfb-factory operator serve` (:3001) est un **crate séparé, hors daemon HTTP** → inatteignable depuis l'iframe (ni fetch via CSP, ni méthode bridge).
- **Le canal `pushEvent` host→iframe** (`useBridge.ts:198-205`, `createEvent` free-form) **existe mais n'est pas câblé** dans le shell : `BrowsedProject.tsx` n'appelle jamais `pushEvent` avec de la donnée Factory, et le host est un navigateur réseau qui ne **possède** pas les artefacts de processus. Mécanisme théorique, mort en pratique.

### 2.4 LIVE vs SNAPSHOT — la réponse nette

| Donnée | Disponibilité | Mécanisme |
|---|---|---|
| Réseau (apps publiées, proof cards de provenance, search, browse, statut nœud) | **LIVE** (état courant du daemon au rendu) | bridge whitelist — **0 changement daemon** |
| Processus Factory (timeline, phases, verdicts, Codex) | **SNAPSHOT figé au publish, OBLIGATOIRE** | embarqué en `<script src="factory-data.js">` posant `window.__FACTORY__` ; jamais `fetch` |
| Processus Factory **LIVE** | **IMPOSSIBLE** sans nouvelle surface protocole | route daemon + nouvelle méthode bridge (ex. `factory_proof_get`) — **hors périmètre d'une app** |

**Conséquence stratégique** (l'angle mort #1 de l'adversarial, confirmé sur le code) : la composante LIVE d'un Viewer (réseau) **chevauche la page Browse native du shell** qui montre déjà browse_list + proof cards. Un Viewer scellé qui ne fait que re-rendre le réseau est **partiellement redondant**. **Sa valeur UNIQUE = la preuve-de-processus SNAPSHOT** — précisément ce que le shell ne montre pas et qu'aucun bridge n'expose. **Le Viewer doit s'ancrer là-dessus, ou c'est un dogfood creux.** Snapshot hash-pinné, anti-PASS, consommé-jamais-autoritaire : c'est aussi le data-flow le plus **aligné aux invariants** (preuve épinglée au hash, pas verdict calculé).

---

## 3. Stack recommandé pour le Viewer scellé

### Recommandation : template Factory `daisyui` lean (CSS AOT) + JS vanilla, anime.js retiré par défaut

C'est le stack le plus CSP-propre, le plus maintenable solo, et celui qui fait coup double : il **exerce le pipeline S79 livré** (knowledge packs `docs/factory/knowledge/{daisyui,animejs}`, prompt-kind `app-authoring`, gate Rust `run_gate_csp_authoring`, self-check runtime) sur un artefact réel scellé rendu cross-pair.

### Faisabilité prouvée par le code et poids réels mesurés

| Asset runtime (dans l'archive) | Poids réel | Source |
|---|---|---|
| `app.css` (Tailwind v4 + daisyUI compilé **lean** `themes:false` + `source(none)`) | **18 750 o** | `crates/sbfb-factory/src/templates/daisyui/app.css` |
| `vendor/anime.umd.js` (anime.js v4.5.0 UMD) | **118 204 o** | `…/daisyui/vendor/anime.umd.js` |
| `index.html` | ~1,5 KB | `…/daisyui/index.html` |
| `sbfb-bridge.js` (SDK, obligatoire) | 14,6 KB | `examples/sbfb-factory-viewer/sbfb-bridge.js` |

- **Build AOT, runtime à 0 dépendance réseau** : `build:css = tailwindcss -i src/input.css -o app.css --minify` (sortie statique same-origin) ; `vendor:anime` copie le bundle UMD depuis `node_modules` (pas de CDN). Pins exacts sans caret (reproductible) : daisyUI **5.5.23**, Tailwind **4.3.1**, anime.js **4.5.0** (vérifié `test_daisyui_package_json_pins_resolved_versions`).
- **CSP-cleanliness prouvée par construction ET par le gate Rust non-délégable** : `'unsafe-inline'`+`'unsafe-eval'` autorisent CSS inline et JS classique ; le **seul** piège est `<script type=module>` (fetché en mode CORS, qu'une origine opaque sous COEP `require-corp` ne peut satisfaire). Le template charge anime en **classic `<script src>`** et daisyUI en `<link rel=stylesheet>` → 0 module, 0 réseau. Tests `test_create_daisyui_template` assertent `!type="module"` et `!src="http`.
- **daisyUI n'est PAS lourd au niveau CSS** : 18,7 KB lean pour la palette d'un Viewer (cards/badges/btn/progress/timeline/steps) vs 117 KB pour le showcase qui exerce toute la largeur. La seule lourdeur du template tel quel = anime.js (118 KB).

### Comparatif honnête (app scellée read-only, origine opaque, COEP require-corp)

| Stack | CSP-clean | Runtime | a11y / thème | Maintenance solo | Verdict |
|---|---|---|---|---|---|
| HTML/CSS pur + vanilla (état actuel) | Parfait | ~27 KB | OK mais tout re-dérivé à la main, dérive de thème | Bonne mais ad hoc | Solide mais ré-invente ce que daisyUI donne gratis |
| **daisyUI lean + vanilla, SANS anime (reco)** | Parfait (CSS AOT statique) | **~41 KB** | **Meilleur** (composants sémantiques + thème `sbfb-reflect` oklch centralisé) | **Meilleure** | **Idéal** |
| daisyUI + anime.js (template tel quel) | Parfait | **~159 KB** | id. + lib motion à gérer | id. | **Sur-dimensionné** pour read-only |
| React no-build UMD vendoré | Bon | **~141 KB** | OK | Lourd ; le seul attrait (réutiliser le socle TSX) est **NUL** : le socle est du TSX qui exige un build que le no-build n'a pas | Overkill |
| Preact + htm vendoré UMD | Bon | **~5,5 KB** | Bonne | Bonne | **Plan B** si l'interactivité grossit (peu probable read-only) |
| Lit / Web Components | **Friction** : ESM-first, `type=module` rejeté en origine opaque sous COEP | ~6-15 KB | OK | Lutte contre la CSP = mauvais signal | À éviter |
| Svelte compilé IIFE | Bon | minuscule | OK | **Ajoute une 2e toolchain** au pipeline qui n'a que le CLI Tailwind | Coût d'outillage injustifié |

### anime.js : sur-dimensionné pour une *vitrine* read-only, mais à conserver pour un *Viewer de preuve animé*

Nuance importante entre G3 et X1. Pour une **vitrine triviale** (cartes/badges statiques), les besoins de motion (reveal/stagger, pulse de badge, barre de fraîcheur) se font **en CSS** (`@keyframes`, `transition`, `prefers-reduced-motion`) sans 118 KB de JS → **retirer anime.js**. **Mais** si le Viewer adopte le registre « instrument » de preuve animée (section 7 — score additif qui s'assemble par segments, échelle N0→N3, pouls de fraîcheur TTL), alors le **mouvement EST le contenu de preuve** et anime.js se justifie. **Arbitrage à trancher au kickoff** (question ouverte §11) : vitrine sobre CSS-only (anime retiré) vs Viewer-de-preuve-animé (anime conservé). Par défaut, partir CSS-only et n'ajouter anime que si la preuve animée devient un objectif explicite.

### Décision stack

1. **Template Factory `daisyui` lean** (CSS AOT + vanilla), anime.js **retiré par défaut** (ré-introductible si preuve animée).
2. **Ré-implémenter ProofCard/StatusBadge/VerdictChip en markup daisyUI** (`badge`, `card`, `stat`, `timeline`, `steps`) — gain visuel net sur le `style.css` artisanal.
3. **NE PAS** réutiliser le socle React `tools/factory-ui/src/readonly` (TSX → build incompatible no-build/scellé). Plan B : **Preact+htm vendoré (~5,5 KB)** si l'interactivité dépasse un jour la vitrine ; **jamais** Lit ni Svelte.

---

## 4. « Viewer à travers Factory » — verdict stratégique

### 4.1 Ferme-t-il le carry P1 in-vivo ? → **OUI PARTIELLEMENT** (jamais « ferme P1 »)

Formulation exacte du carry (`.planning/active/sprint80_audit_plan.md:54-57`) — **DEUX composantes** :

> « parcours auteur réel → gate → self-check → publish → **rendu cross-pair** JAMAIS exercé in-vivo ; **efficacité générative** du prompt-kind / copilote Ollama non mesurée. »

**Parcours réel mappé sur le code** :

| Étape | Mécanisme réel | Ferme P1 ? |
|---|---|---|
| CREATE | `sbfb-factory create daisyui` → `template_engine.rs:277` matérialise fichiers + `SBFB.json` + `factory.template.lock` + `factory.provenance.json` | Oui — exerce le template S79 |
| GATE CSP | `pipeline.rs:52` `run_gate_csp_authoring` **BLOQUANT, hors `skip_gates`** (invariant Day-0 « 0 dispense CSP », bloque même `--skip-gates`) | Oui — exerce le gate non-délégable |
| SELF-CHECK | self-check runtime Phase H (filet du gate statique) | Oui |
| PUBLISH | `pipeline.rs:62` `post_deploy_from_repo` envoie **`repo_url`** à `/api/v1/deploy-from-repo` : **le daemon CLONE le repo git** (clone→Ed25519→zip→provenance, modèle verified-deploy S14). Le workspace local ne sert qu'aux gates | Oui — mais exige source **dans un repo git clonable** + daemon vivant |
| RENDER local | shell `BrowsedProject.tsx:605` iframe scellé + bridge | Oui — render **local** |
| RENDER **cross-pair** | un **2e pair frais** pull l'archive annoncée et la rend | **Gated rig 2-machines** |

**Verdict composante (a)** : la part **auteur→gate→self-check→publish→render LOCAL** devient exerçable in-vivo **aujourd'hui, sur une seule machine** — la majeure partie du carry passe de `Not evidenced` à *evidenced*. Mais le mot **« cross-pair »** exige un 2e pair joignable qui pull l'archive — exactement la convergence WAN/LAN `DIFFERE-materiel` en S74/S76 (rig RTX 5080 + Mac M2 absent, `live_acceptance_setup.md`). Sans 2e pair, ce sous-segment reste **PROVISIONAL + carry** (gate T2 `RIG-ABSENT`). Même plafond-matériel récurrent, pas un défaut du plan Viewer.

**Verdict composante (b) — le piège** : l'efficacité générative n'est mesurée **QUE si l'authoring passe réellement par le prompt-kind `app-authoring` + copilote Ollama**. **Hand-coder le Viewer prouve la mécanique du pipeline (a) mais laisse (b) INTACT.** C'est l'angle mort #2 de l'adversarial : on peut cocher « P1 fermé » sur le papier pendant que la substance générative reste `Not evidenced` — faux sentiment de clôture. **Condition impérative : générer via le chemin prompt-kind/copilote.**

→ **Fermeture nette = (a)-local TOTALE, (a)-cross-pair sous condition-rig (RIG-ABSENT probable → re-carry), (b) sous condition-génération-réelle. Donc : oui-partiellement (~70 % du carry), jamais 100 %.**

### 4.2 Valeur vs coût

**Valeur** :
- **1re vraie app Factory-authored** (l'actuel Viewer est vanilla, 0 `template.lock`/`provenance` → n'est PAS un produit Factory).
- **Surface-preuve publique cardinale** : la vitrine sur laquelle repose la thèse « source vérifiable ». L'Operator n'est qu'un établi privé que seul le mainteneur solo voit.
- **Dogfood ultime** : fait passer en LIVE les 4 surfaces livrées par S79 sur un artefact réel scellé.
- **Tranche la dette socle orphelin** au passage (le Viewer scellé n'a de toute façon pas besoin du socle TSX).

**Coût (chiffré, factuel)** :
- **Ré-authoring complet** (pas « re-rendre l'existant » — voir 4.3) : le Viewer actuel est cassé et hand-authored.
- **Build daisyUI AOT** réintroduit une dépendance Node/npm author-time (contrairement au template react no-build). L'archive publiée reste statique/CSP-safe.
- **`bridge_methods` du template daisyui = VIDE** (`template_engine.rs:266`) → re-déclarer `browse_list`/`search`/`proof_card_get` à la main dans `SBFB.json` (autorisés par la whitelist, mais pas de preset « viewer »).
- **Script de scellage du snapshot** : générer `factory-data.js` (`window.__FACTORY__`) depuis les artefacts `.planning/` au publish — **n'existe pas, à écrire**.
- **Mapping à refaire proprement** (section 4.3).
- **Repo git clonable + daemon vivant** requis par le modèle publish.

Coût total = **borné, 1 app isolée par l'iframe**, sans nouvelle surface protocole tant qu'on reste sur (1)+(2).

### 4.3 « Re-rendre l'existant » n'est PAS une option — le Viewer actuel est un STUB PÉRIMÉ

Vérifié sur le code (V1, re-confirmé) — l'exemple est **cassé** contre le daemon d'aujourd'hui :

1. **Grille toujours vide** : `app.js:149` fait `apps = (result && result.projects) || []`. Or `/api/daemon/browse` renvoie `{ "entries": [...] }` (`http.rs:1047`) — pas `projects`, pas `published/version/category`. → `result.projects = undefined` → grille vide en permanence.
2. **Panneau preuve toujours mort** : `app.js:91` passe `app.name` à `getProofCard`, qui exige un `project_id` (`useBridge.ts:399`) ; et lit `proof.commit_source`/`proof.verified`/`proof.signer_pubkey` — champs **plats inexistants** (le vrai `ProofCard` est **nesté** : `provenance.verified`, `provenance.commit_sha`, `hash.archive_hash`, `proof_card.rs:39`).
3. **Anti-pattern de verdict inventé** : `app.js:93` fabrique `verifiedText = "✓ Vérifié"` à partir du champ mort `proof.verified` — exactement le péché « consommée → autoritaire » à NE PAS reproduire.

→ Tout Viewer utile = **ré-authoring**, en **corrigeant le mapping (`entries` pas `projects` ; `project_id` pas `name` ; `ProofCard` nesté) AVANT toute esthétique**.

---

## 5. Direction design du Viewer (surface de preuve, calm-tech, consommée-jamais-autoritaire)

Le langage visuel n'est **pas à inventer** : `examples/daisyui-animejs-showcase/app.js` contient déjà, en daisyUI + anime.js CSP-safe + reduced-motion-aware, les composants exacts d'un Viewer de preuve, chacun ancrant un fait Rust réel. La direction = **extraire le registre « instrument », jeter le registre « démo joujou », re-sceller via le template `daisyui`**.

### Trois doctrines (grounded)

- **Calm technology (Amber Case)** — l'attention vit en périphérie ; le Viewer est **immobile par défaut**, le mouvement n'apparaît qu'à un changement d'état ou pour révéler une structure. Traduction directe de « motion = sens, jamais décor ».
- **Transparency-log / Sigstore-Rekor** — la valeur n'est pas un tampon, c'est un **registre append-only inspectable** rejouable. Le Viewer montre **la chaîne** (commit_source → archive_hash → signer_pubkey) en **monospace copiable**, et **qualifie** ce qui est auto-attesté : SLSA L1 (S14) est une **auto-attestation, pas un re-build indépendant** — on l'écrit, on ne laisse pas un ✓ le sous-entendre.
- **F-Droid « making reproducible builds visible » (2025)** — *« si les apps sont juste marquées d'une case à cocher, l'utilisateur doit faire confiance au fait que quelqu'un a bien agi »*. Une badge peut induire en erreur. Le Viewer ne montre **JAMAIS un « ✓ Vérifié » nu** : il montre **la décomposition** (couches additives + manques) et **le chemin pour vérifier** (voir le code source).

**Principe directeur unique** : **la preuve EST la décomposition, jamais le verdict.** Le Viewer affiche l'état que les artefacts contiennent ; il n'en calcule aucun. Le plus haut niveau de confiance (N3, PASS plein) n'est **jamais** atteignable par dérivation UI.

### Architecture de l'information

```
VIEWER
├─ A. INDEX — vitrine réseau (LIVE, bridge: browse_list / search)
│     cartes d'apps · chips d'état honnêtes · recherche FTS5 · filtres
├─ B. DÉTAIL APP = VUE DE PREUVE (LIVE, bridge: proof_card_get / provenance_get)
│     1. Chaîne provenance  commit → archive_hash → signer_pubkey [mono, copiable]
│                           qualif : « auto-attesté SLSA L1, non re-build indépendant »
│     2. Carte de preuve ADDITIVE  couches (base/prov/oss/curateurs/licence) + score
│                                   + RISQUES en pénalités
│     3. Portes franchies   gate CSP · sandbox · secrets — « ce qui a été vérifié »
│     4. Échelle N0→N3      N3 jamais dérivé (immobile, grisé)
│     5. Manques honnêtes   PROVISIONAL · Not evidenced · RIG-ABSENT — citoyens 1re classe
│     → intentions : « Voir le code source » · « Vérifier la provenance »
└─ C. PREUVE DE PROCESSUS (SNAPSHOT scellé, <script>window.__FACTORY__)
      timeline sprint · phases (steps ✓/✕/●) · verdicts · Codex
      bandeau permanent : « instantané scellé au publish · épinglé au hash · non-live »
```

Règle IA cardinale : **B est LIVE** (reflète le nœud local au rendu), **C est figé** (aucun chemin live). Le dashboard live de processus reste le boulot de l'**Operator** — pas du Viewer scellé ; on l'assume visuellement par le bandeau « instantané ».

### Système visuel

- **Thème** `sbfb-reflect` oklch dark (existe, `src/input.css:21`) : surfaces **achromatiques chroma 0**, `--depth:0 --noise:0`. **La couleur ne sert qu'à porter un signal** (success=attesté, warning=provisoire, error=risque, neutre=absence).
- **Typo crypto monospace** pour tout hash/pubkey/commit, copiable (registre Rekor).
- **Densité calme**, 1 colonne `max-w-xl`, allure page-app F-Droid pas dashboard SaaS.
- **Bordures > ombres** : pleine = attesté, pointillée = provisoire/brouillon.
- **Vocab daisyUI v5** : `timeline`, `steps` (`data-content` ✓/✕/●), `badge`+`badge-outline`/`badge-ghost`, `radial-progress`, `status`, `stat`, `alert` — a11y portée par l'élément, 0 JS comportement.

### Langage de mouvement (si preuve animée)

**Contrat** : le mouvement encode un changement d'état ou révèle une structure ; `prefers-reduced-motion` ⇒ **état final immédiat** (branche `REDUCE` du showcase = le contrat, pas une option).

- **Retenu** : reveal stagger d'entrée (one-shot, jamais en boucle) ; **score qui s'assemble par segments additifs** (le mouvement EST le calcul de `proof_card.rs`) ; transition d'échelle N0→N3 (**N3 ne s'allume jamais par dérivation**) ; pouls de fraîcheur ambient TTL (seul mouvement idle toléré car il porte le sens de l'incertitude).
- **Banni** : boutons magnétiques, tilt 3D, burst, orbites, scramble décoratif, `outBack`/overshoot en contexte de preuve (l'overshoot connote la confiance/le jeu — sémiotique fausse).
- **Mouvement d'honnêteté** : une valeur *Not evidenced* **n'anime vers aucun nombre** — elle énonce l'absence et reste immobile. **L'absence est immobile ; la présence s'assemble.**

### Tenue de l'invariant « consommée jamais autoritaire »

1. **0 verdict calculé par l'UI** — chaque badge mappe 1:1 un champ de l'artefact (inverse exact du stub actuel `app.js:93` qui invente `proof.verified`).
2. **Le plus haut état de confiance jamais dérivable** — N3 et « PASS plein » restent immobiles/grisés, atteignables seulement par un artefact signé.
3. **États honnêtes citoyens de 1re classe** : *PROVISIONAL* → `badge-warning badge-outline` (ambre) ; *Not evidenced* → `badge-ghost` pointillé **achromatique** ; *RIG-ABSENT* → ghost + tooltip ; *auto-attesté SLSA L1* → qualificatif « non re-build indépendant » collé au ✓.
4. **Intentions, pas jargon** : CTA = « Voir la preuve », « Vérifier la provenance », « Voir le code source » — jamais `proof_card_get`, `FG-CSP`, `N2`.

Cohérence philosophique : sobre/plat/achromatique-sauf-signal + auto-vérifiable + anti-tampon = lignée **Tor/F-Droid/OpenBSD**, atelier-souverain, anti-SaaS. Le Viewer ne **vend** pas la confiance ; il **expose le registre** et laisse vérifier.

---

## 6. Faut-il refondre l'Operator ?

### Verdict : **PLUS TARD** (pas maintenant), et obligatoirement **HYBRID** (jamais daisyUI-pur)

**Tension à nommer honnêtement** : la directive `po_directive_factory_front_redesign.md` confond deux dogfoods. Refondre l'Operator dogfoode le **langage visuel** daisyUI+anime ; il ne dogfoode **PAS** le **pipeline Factory**. L'Operator est un outil local privilégié, **non scellé, sans CSP, sans iframe** — le re-skiner n'exerce **aucun** étage app-authoring (ni `run_gate_csp_authoring`, ni self-check, ni publish, ni rendu cross-pair). **Le carry P1 qui compte reste intact après une refonte Operator.**

**daisyUI-pur = régression a11y/comportement** : daisyUI est une lib de **classes CSS**, elle ne livre **aucun comportement JS**. Les composants composites « CSS-hack » (dropdown via focus/`<details>`, tabs via `<input radio>`, tooltip via `::before`) ont une a11y/clavier nettement plus faibles que Base UI : pas de focus-trap robuste, pas de navigation flèches+typeahead dans une listbox/menu, pas de portal anti-clipping ni de positionnement collision-aware (Floating UI), pas d'annonces SR fiables. Or l'Operator utilise précisément ces comportements riches (Select portal+typeahead, Menu submenu+kbd nav, Tooltip, ScrollArea, Dialog focus-trap).

→ **Si/quand l'Operator est refondu, obligatoirement HYBRID** : garder **Base UI** (comportement/a11y, déjà en place) + adopter le **système de thème/couleurs daisyUI** + **anime.js** (motion). On n'empile **pas** les widgets interactifs daisyUI (`modal`, `dropdown`, `tabs`) sur les slots Base UI. Au passage : **purger les 6 `@radix-ui/*` morts** du `package.json`.

### L'option « design system partagé readonly »

**Non recommandée maintenant.** Le socle `tools/factory-ui/src/readonly` est orphelin **depuis S70** — c'est déjà la preuve que l'abstraction était prématurée une fois. Un DS partagé CSP-safe exigerait soit un build pour le Viewer (casse no-build/CSP), soit un **double rendu React+vanilla** (double maintenance) — sur-ingénierie pour un mainteneur solo. Le Viewer scellé n'a de toute façon **pas besoin** du socle TSX. **Décision forcée par le Viewer : trancher le socle (le retirer / l'assumer comme code Operator-only), pas le ressusciter.**

---

## 7. Comparatif honnête des 4 options + angles morts de la préférence PO

**Fait transverse qui recadre tout** : l'**audit gate S79 = Phase 0 de S80 est obligatoire et passe EN PREMIER**, quelle que soit l'option (`sprint80_audit_plan.md`). La vraie question n'est pas « front ou pas » mais **« que fait Phase A+ APRÈS le gate »**. Et le §3 pose **DEUX P1 ouverts** : `app-authoring in-vivo` **ET** `Sharding S77 PROVISIONAL`, avec arbitrage explicite « Décider S80 : ouvrir l'orchestrateur sharding **ou** poursuivre Factory ».

| | **A — Viewer-via-Factory** (préf. PO) | **B — Refonte Operator** | **C — Les deux (DS partagé)** | **D — Minimal (audit/sharding)** |
|---|---|---|---|---|
| Ferme un P1 ? | (a)-local oui, (a)-cross-pair sous rig, (b) sous génération-réelle | **Non** | in-vivo oui mais noyé | sharding **si rig dispo** |
| Capacité ajoutée | **1re app Factory-authored + surface-preuve publique** | **Zéro** (Base UI déjà complet/a11y) — cosmétique | les deux mais scope énorme | gate qualité / preuve compute |
| Risque/surface | Faible/borné (1 app iframe-isolée) ; **risque data-affamé** | **Élevé** (12 pages + xterm + SSE ; régression a11y) ; peut casser l'établi de prod | **Le plus gros, multi-sprint** ; double maintenance | viewer public reste un stub cassé |
| Fit thèse (vitrine publique / solo / atelier) | **Maximal** | Faible (établi **privé**) | sur-ingénierie solo | fort si sharding actionnable |

### Angles morts de la préférence PO (depuis l'adversarial, à ne PAS rubber-stamper)

1. **Confusion de catégorie** : un « Factory Viewer » scellé ne peut PAS montrer le processus Factory en LIVE (section 2). Sa donnée live (réseau) chevauche la page Browse native. → **Sa raison d'être réelle = la preuve-de-processus SNAPSHOT**, pas un doublon de Browse. **Correction : ancrer le Viewer sur le snapshot (2), vitrine réseau (1) en contexte.**
2. **Fermeture HOLLOW du P1** : le carry dit « efficacité générative non mesurée ». Authorer un Viewer **read-only** (l'app la plus triviale, qui existe déjà en vanilla) **sous-exerce** la claim générative. → **Correction : générer via prompt-kind/Ollama ; envisager une app non-triviale** pour maximiser le signal génératif.
3. **A choisit le P1 FACILE ; le P1 phare (sharding) pourrit** — sauf que sharding est **RIG-ABSENT**. **C'est le seul argument solide pro-A** : faire le dogfood Factory précisément quand on ne PEUT pas faire le sharding. À confirmer au gate (le rig est-il vraiment indisponible ce sprint ?).
4. **daisyUI réintroduit une toolchain Node/npm** pour une app déjà fonctionnelle 0-build → justification partiellement circulaire. Acceptable car le dogfood vise précisément à exercer le template S79, mais à assumer comme tel.
5. **A laisse l'Operator intact ET entérine la dette socle** ; si le Viewer part en vanilla/daisyUI, A ajoute un **3e front indépendant** (vanilla daisy) à côté de l'Operator (Base UI) et du socle mort (TSX). → **Correction : trancher explicitement le socle au passage.**

---

## 8. Recommandation finale & chemin S80

### Option retenue : **A, recadrée** — Viewer SEUL, authored à travers Factory, comme corps de S80

**Rappel non-négociable : S80 Phase 0 = audit gate S79 D'ABORD** (pattern permanent depuis Sprint 7 ; PASS exige le traitement des P0/P1). Le Viewer est le **corps** de S80, pas un remplaçant du gate. Les findings du gate arbitrent aussi A-vs-sharding (le seul juge factuel de « quel P1 est actionnable ce sprint »).

**Stack** : template Factory `daisyui` lean (CSS AOT) + JS vanilla ; anime.js retiré par défaut (ré-introductible si preuve animée tranchée) ; ProofCard/badges ré-implémentés en markup daisyUI ; socle TSX orphelin **non** réutilisé → **décider sa purge**.

**Data-flow tranché AVANT de coder** : (1) vitrine réseau **LIVE** via bridge whitelist existant (0 changement daemon) **+** (2) preuve-de-processus **SNAPSHOT** signée embarquée (`window.__FACTORY__`, script de scellage à écrire). **Pas** de process-proof live (hors périmètre).

**Design** : registre « instrument » de preuve (section 5), calm-tech, oklch `sbfb-reflect`, consommée-jamais-autoritaire, intentions-pas-jargon.

### Phases incrémentales proposées (indicatives)

- **Phase 0** — audit gate S79 (BLOQUANT ; arbitre A-vs-sharding selon rig).
- **Phase A** — fix mapping + bridge réel : ré-authoring du Viewer via `create daisyui`, `bridge_methods` déclarées (`browse_list`/`search`/`proof_card_get`), mapping corrigé (`entries`/`project_id`/ProofCard nesté). T1 Playwright hermétique (render local + bridge mock) **BLOQUANT-vert**.
- **Phase B** — vitrine réseau LIVE (index + détail app = vue de preuve additive + provenance copiable + N0→N3 immobile).
- **Phase C** — snapshot preuve-de-processus : script de scellage `factory-data.js` au publish + vue C avec bandeau « instantané scellé ».
- **Phase D** — authoring GÉNÉRATIF via prompt-kind `app-authoring` + copilote Ollama (mesure de l'efficacité générative — referme la composante (b) du P1).
- **Phase E** — publish in-vivo (gate CSP bloquant + provenance) + render local + **T2 cross-pair JSON honnête** (`PASS` / `RIG-ABSENT` si pas de 2e machine) + trancher la dette socle.
- **Phase F** — wrap-up, doc, carry résiduel.

### Lien avec le carry P1

À l'issue de S80, le carry doit être **requalifié honnêtement, pas coché** : « (a)-local *evidenced* ; (a)-cross-pair *PROVISIONAL/RIG-ABSENT* (re-carry si pas de 2e pair) ; (b) *evidenced* si génération réelle via prompt-kind/Ollama ». Le sprint **réduit P1 d'environ 70 %** mais ne le clôt à 100 % que si (i) un 2e pair est joignable au wrap-up ET (ii) l'authoring est génératif réel. À cadrer explicitement dans le kickoff, sous gate de testabilité §4 (T1 BLOQUANT + T2 JSON `PASS`/`BLOCK`/`RIG-ABSENT`).

---

## 9. Questions ouvertes PO (arbitrages)

1. **Rig 2-machines disponible pour S80 ?** Si oui, le P1 sharding redevient actionnable et l'arbitrage A-vs-sharding doit être re-posé (X3 angle mort #3). Si non (RIG-ABSENT confirmé), A est le bon move ce sprint, et le segment cross-pair du Viewer retombera lui aussi en `RIG-ABSENT`.
2. **Vitrine sobre (CSS-only, anime retiré) ou Viewer-de-preuve-animé (anime conservé) ?** Décide si on garde anime.js (118 KB) — divergence G3 (retirer) vs X1 (conserver car le mouvement EST la preuve). Recommandation par défaut : CSS-only, anime seulement si la preuve animée devient un objectif explicite.
3. **App non-triviale ou Viewer read-only ?** Le carry « efficacité générative » se ferme mieux sur une app plus riche. Accepter un Viewer read-only = accepter que la composante (b) ne se ferme que partiellement.
4. **Sort du socle `tools/factory-ui/src/readonly` orphelin** : le **retirer** (le Viewer scellé n'en a pas besoin) ou le requalifier en code **Operator-only** ? À trancher au passage, sinon A ajoute un 3e front divergent.
5. **Requalifier la directive `po_directive_factory_front_redesign.md`** : la refonte Operator n'est PAS du dogfood-pipeline (elle ne touche aucun P1). La reprogrammer comme sprint cosmétique **optionnel, borné, HYBRID, après** le Viewer ?
6. **Quel niveau de scellage pour le snapshot de processus ?** Snapshot brut embarqué vs snapshot **signé** (Ed25519) pour rester cohérent avec la doctrine « arête de provenance » — décider si `factory-data.js` porte une signature vérifiable.

---

### Fichiers de référence (tous absolus depuis la racine repo)

- `web/src/bridge/protocol.ts:20-49` (whitelist 16 méthodes, 0 méthode processus Factory)
- `web/src/bridge/useBridge.ts:226-416` (dispatch host ; `storage_get:264`, `proof_card_get:398`, `task_result:236`)
- `web/src/pages/BrowsedProject.tsx:605` (iframe scellé `sandbox=allow-scripts`)
- `crates/nexus-core-rs/src/csp.rs:33` (`BLOB_SERVE_CSP`, source unique)
- `crates/sbfb-factory/src/pipeline.rs:47-62` (gate CSP bloquant + `post_deploy_from_repo` repo-clone)
- `crates/sbfb-factory/src/template_engine.rs:266,277` (`bridge_methods` daisyui VIDE ; matérialisation)
- `crates/sbfb-factory/src/gates.rs:386` (`run_gate_csp_authoring`)
- `crates/sbfb-factory/src/templates/daisyui/{index.html,app.css,app.js,src/input.css,package.json,vendor/anime.umd.js}`
- `crates/nexus-shell-daemon/src/http.rs:1047` (browse `{entries}`)
- `crates/nexus-coordinator-rs/src/proof_card.rs:39` (ProofCard nesté)
- `examples/sbfb-factory-viewer/{app.js:91,93,149,SBFB.json}` (stub périmé, 0 `factory.template.lock`/`provenance.json`)
- `examples/daisyui-animejs-showcase/app.js` (registre composants vivants : `:423` ladder N0→N3, `:566` proof card additive, `:513` pouls TTL)
- `tools/factory-operator/{package.json,src/components/ui/*,src/index.css,src/pages/{AgentChat,ExecutionChat}.tsx}` (Operator Base UI, 0 daisyUI, 6 radix morts)
- `tools/factory-ui/src/readonly/` (socle TSX orphelin, 0 consommateur de code)
- `.planning/active/sprint80_audit_plan.md:50-57` (2 P1 concurrents)
- `po_directive_factory_front_redesign.md` (directive à requalifier) ; `live_acceptance_setup.md` (rig cross-pair récurrent-absent)

### Sources web (grounded)

- Sigstore — Introduction to Rekor (transparency log append-only inspectable)
- Amber Case — Calm Technology: Principles and Patterns for Non-Intrusive Design
- F-Droid — Making reproducible builds visible (2025) : la case à cocher induit en erreur, montrer le processus
- Trust UX — Badges, Proof, and the Research Behind Them (les experts escomptent les badges, donner la preuve brute)
- MDN COEP require-corp + web.dev COOP/COEP cross-origin isolation (`type=module` rejeté en origine opaque)
- daisyUI v5 (timeline/steps/status/stat) + anime.js v4 (UMD global, lightweight engine) via Context7
