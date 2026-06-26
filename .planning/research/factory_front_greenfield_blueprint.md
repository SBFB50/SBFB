# Factory — Blueprint front greenfield (page blanche totale)

> **[MISE À JOUR — DÉCISION PO 2026-06-26 — base retenue pour S80]**
> Ce blueprint est la **base du corps de S80** (refonte **Operator greenfield**).
> Deltas PO à appliquer par-dessus le corps du doc, à **trancher définitivement au
> kickoff Cas C** :
> 1. **shadcn N'EST PAS exclu.** §6.3 le purgeait par décret — **corrigé**. Le PO n'a
>    jamais dit « no shadcn » ; le « pour ne pas utiliser shadcn » qualifiait le
>    *design system SBFB (un test)*, pas une interdiction. shadcn redevient **candidat
>    de la couche composants**, face à Base UI headless et daisyUI.
> 2. **Une vraie lib de motion EST voulue** — « **motion (framer) ou anime.js** pour un
>    UI extrêmement poussé ». Override du §5.3 « 0 lib motion par défaut » : CSS / View
>    Transitions / WAAPI restent le **socle**, la lib s'ajoute pour le registre poussé.
>    Arbitrage **Motion vs anime.js** au kickoff.
> 3. **« Design system SBFB » = un test, à ÉCARTER** (ne pas réutiliser). Le greenfield
>    le jette de toute façon avec le front actuel ; identification précise de l'artefact
>    = D-decision kickoff.
> 4. **Prérequis BLOQUANT confirmé** : auth cookie HttpOnly same-origin (§6.5 / §8.1) —
>    sinon SSE/WS = 401 en prod sous `ServeDir`. 1er geste backend.
> 5. **Phase 0 = audit gate S79** avant toute Phase A ; il arbitre aussi le P1 sharding
>    (selon dispo rig 2-machines).
> Docs frères : `factory_interface_paradigm_rnd.md` (paradigme « pas IDE/plugin », tenu) ;
> `factory_front_best_approach_research.md` (Viewer scellé, **piste reportée** S81+).

> **Méta.** Équipe ultradeep « page-blanche », 2026-06-26. Méthode Workflow : Reframe (jobs-to-be-done + cube libre/contraint) → Re-dérivations parallèles (framework, UX/IA, styling-motion, stack-packages) → passe Adversariale → Synthèse Directeur. Web-grounded juin 2026 (Context7 + WebSearch), licences vérifiées au manifeste, claims ré-ancrés au code réel (`file:line`).
> **Mandat.** On JETTE le front actuel (`tools/factory-operator`) sans contrainte d'héritage (ni React, ni structure, ni pages). **daisyUI et anime.js sont RETIRÉS par le PO** — re-débattus depuis zéro, pas réintroduits par défaut.
> **Statut.** Décision-grade, mais explicitement non-rubber-stamp : on confirme la R&D antérieure là où elle tient, on la **dépasse** là où la page blanche change la donne (framework ré-ouvert ; tri-zone → bi-focal ; auth-transport ; séquencement du mode VERIFY).

---

## 1. Réponse courte

Le meilleur front greenfield de Factory est un **établi bi-focal agent-native** — pas un IDE, pas un tri-zone à trois panneaux coéquaux. Une fenêtre : un **rail d'orientation ambiant permanent** (la colonne vertébrale du procédé : sprint · phase · branche · dirty/staged · pouls des gates) et **une scène mono-focale** qui ne porte qu'**UN de deux MODES à la fois — STEER (intention → steering observable) ou VERIFY (diff → gates → aperçu scellé → preuve)** — la bascule étant pilotée par l'état du travail. La pile : **React 19** (retenu par incumbence + écosystème headless le plus mûr, la fluence-agent en appui et non comme victoire de premiers principes), **Tailwind v4 CSS-first** avec tokens **oklch maison** étendant le thème `sbfb-reflect` (**daisyUI retiré**, réservé aux apps scellées), **Base UI 1.0** headless pour les primitives à comportement-d'interaction coûteux, et **motion natif** (CSS + View Transitions API + WAAPI, **anime.js et GSAP écartés**). La décision d'architecture **#1**, à trancher AVANT le framework et **forcée par le terminal WebSocket** : basculer l'auth en **cookie HttpOnly same-origin**, sinon le steering (SSE, J3) et le terminal (WS, J12) renvoient **401 en prod** sous `ServeDir`. Honnêteté de séquencement : le mode VERIFY plein est **aspirationnel** tant que les routes manquent (pas de GET gates, pas de `git diff` working-tree) — le MVP livre **STEER câblé + rail + terminal PTY élevé comme surface de vérification**.

---

## 2. Jobs-to-be-done (la matière, irréductible)

Dérivés du backend Operator réel (axum loopback `127.0.0.1:3001`, token + Host + Origin) — rien d'autre n'existe. Poids décroissant.

**Primaire (intention / vérification — le cœur agent-native) :**

| # | Job | Capacité backend | Surface |
|---|---|---|---|
| **J4** | **Vérifier un diff** — lire ce que l'agent a changé (working-tree dirty/staged, diff de commit) | `/api/context` (dirty/staged = **liste seulement**), `/api/sprint-history/diff/{sha}` (commits passés) | revue de diff par-fichier/hunk — **le bottleneck 2026** |
| **J3** | **Observer & steerer l'agent** — flux token, raisonnement, relance, multi-tour | `/api/chat/{id}/stream` (SSE), `/send` | transcript live + relance |
| **J2** | **Exprimer une intention** — décrire, choisir provider+rôle+kind, assembler le pack | `/api/context-pack`, `/api/chat/session`, `/api/providers`, `/api/prompt/{kind}` | composeur d'intention |
| **J5** | **Lire gates / verdict** — FG4/5/6/CSP/7/8 + lint + audit-commit | pipeline + `/api/lint` + `/api/audit/{rev}` (**aucune route GET « gates live »**) | panneau gates rouge/vert + issues |
| **J1** | **S'orienter** — sprint/phase/branche/head/dirty/staged/commits | `/api/status`, `/api/context`, `/api/sprint-history` | rail ambiant persistant |
| **J6** | **Prévisualiser l'app scellée** — rendu réel iframe blob-serve sous `BLOB_SERVE_CSP` | FG7-preview + blob-serve | iframe sandbox (CSP prod exacte) |
| **J7** | **Lire la Proof Card / provenance** — hash blake3, signataire Ed25519, commit→archive | FG8-provenance, `provenance.rs` | carte de preuve (l'ancre de confiance) |
| **J8** | **Publier / déployer vérifié** — gates → deploy-from-repo → provenance | `pipeline::run_publish_pipeline` (**côté CLI, pas une route Operator**) | flux à mur de gate |
| **J10** | **Transmettre à un autre agent** — context-pack/handoff repo-visible (refs hashées) | `/api/context-pack`, `/api/prompt/handoff` | intention « Transmettre » |

**Secondaire (édition / assist / escape — sur demande, jamais focus par défaut) :**

| # | Job | Capacité | Surface |
|---|---|---|---|
| J9 | Action allowlistée + journal | `/api/actions/run` (4 cmds : `status-sprint, lint-planning, audit-commit, prompt`), `/api/actions/log` | palette + journal |
| J11 | Historique (sprints, diffs passés) | `/api/sprint-history{,/all,/{n},/diff}` | timeline + détail commit |
| J12 | Terminal PTY privilégié + resume | `/api/terminal/ws` (**WebSocket**), `/api/terminal/sessions` | volet terminal |
| J13 | Brouillon non-autoritaire (PASS bloqué) | `/api/artifacts/draft` | éditeur markdown « non-autoritaire » |
| J14 | Inspecter/patcher du code | terminal + FS (**aucune route d'édition de code**) | lecteur/patch mono-fichier |
| J15 | Inspecter ce que l'agent sait | `/api/prompt/{kind}`, `authoring_knowledge` (refs blake3) | inspecteur « knowledge » consultatif |

**Pondération finale :** `J4 ≈ J3 > J2 > J5 > J1 > J6 > J7/J8 > J10 > J9/J11 > J12 > J13/J14/J15`. **L'éditeur de code (J14) est confirmé secondaire par le backend lui-même** : aucune route d'édition de fichier de code n'existe (seul `artifacts/draft`, allowlisté planning + PASS-bloqué). Les agents écrivent le code via SSE+terminal ; l'humain **vérifie le diff**. La gravité du front tombe donc sur l'altitude-2 (vérification), pas sur un IDE.

### Invariants backend-imposés (non négociables, à refléter)

1. **Mur `requires_gate` / `requires_external_agent`** : `SENSITIVE_ACTIONS = ["shell","commit","push","PASS"]` (`operator_server.rs:35`) gate les 3 chemins chat — l'Operator ne spawn JAMAIS d'action sensible. Le front présente un **MUR**, jamais un bouton « faire ».
2. **Zéro PASS auto** : `handle_artifact_draft` refuse tout verdict PASS au path **et** au contenu. Le front n'agrège **jamais** un « PASS ✓ ».
3. **Knowledge consommé-pas-autoritaire** : refs blake3, `chat_history_authoritative:false` (`operator_server.rs:437`). Provenance hash + marqueur « consultatif » visibles.
4. **Flux provider-agnostique** : `StreamChunk` identique pour Claude/Ollama/Network → **une** surface de steering + un sélecteur, pas trois UIs.
5. **Allowlist + journal** : tout geste privilégié passe par `ACTION_ALLOWLIST` (4 cmds) et est journalisé.
6. **Preview scellée réelle uniquement** : iframe blob-serve + `BLOB_SERVE_CSP` (`csp.rs:33`, `connect-src 'none'`), jamais d'onglet blob-serve brut.

---

## 3. Le framework : décision

**Verdict : React 19** (SPA/SSG statique, **React Compiler activé**). **Dauphin : SolidJS** (ré-ouvrir si Solid 2.0 GA). **Rejetés : Rust-WASM, Svelte 5 (3e), Lit (pari-longévité).**

### Classement pondéré (axes dérivés du brief)

| Framework | Fluence-agent | a11y headless | Interop JS (CM6/xterm) | Coût solo | Longévité | Stream | **Total /500** |
|---|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| **React 19** | 5 | 5 | 5 | 4 | 3,5 | 4 | **459 — 1er** |
| **SolidJS** | 3,5 | 4 | 4,5 | 3,5 | 3 | 5 | **381 — 2e** |
| **Svelte 5** | 3 | 3,5 | 4,5 | 4 | 3,5 | 5 | **370 — 3e** |
| **Lit / WC** | 3,5 | 2 | 4 | 3 | 5 | 4 | **342 — 4e** |
| **Rust-WASM** | 1,5 | 1,5 | 1,5 | 2,5 | 4,5 | 4 | **221 — 5e** |

### Pourquoi React gagne — la **vraie** raison, honnêtement recadrée

La re-dérivation initiale faisait gagner React sur la fluence-agent pondérée à 30/100, comme une victoire de premiers principes. **La passe adversariale a raison de contester ce cadrage**, et le Directeur l'intègre :

- **La fluence-agent est un coût _front-loaded_, pas récurrent.** Un control-center interne (lignée OpenBSD, stabilité > churn) s'écrit une fois puis se maintient des années. Après le build, le coût solo dominant est **lire/réviser** (= J4, le job #1), pas **régénérer**. Pondérer l'écriture-agent 3,75× au-dessus du streaming, sans axe propre pour la **lisibilité-diff**, **contredit** notre propre déclaration que J4 est le bottleneck. On rend donc ~10 pts de la fluence vers la lisibilité-diff + le modèle-de-stream : **l'écart React/Solid se resserre fortement**.
- **Le facteur réellement décisif est l'INCUMBENCE + l'écosystème headless mûr**, pas une supériorité fluence intrinsèque : Base UI + xterm sont déjà au `package.json`, 411 tests React vivent dans `web/`, le mainteneur connaît React, et l'interop JS↔JS (CM6 `@codemirror/merge` pour J4, xterm pour J12) est la plus propre. **On assume l'incumbence comme arbitrage légitime et suffisant** — sans la déguiser en slam-dunk de premiers principes.
- **La fluence-agent reste un appui réel et non nul** : corpus React dominant → les agents émettent du React avec le moins d'hallucination de structure, et **React 19 ne casse pas cette fluence** (continuité hooks 18→19 ; le React Compiler *pardonne* le code naïf non-mémoïsé que les agents écrivent spontanément).

### La 2e place et quand elle gagnerait

**SolidJS** sert mieux les **deux** jobs co-dominants : JSX (surface fluent-agent) **+** fine-grained (modèle mental de stream J3 plus simple que la danse `useRef`/`useEffect`/anti-reconnect que `executionChat.ts`/`ExecutionChat.tsx:96-110` montre déjà fiddly en React). a11y via Kobalte/Ark/corvu, MIT. **Solid bascule en tête SI** Solid 2.0 passe **GA avant le sprint** (sinon `@solidjs/signals` beta = cible mouvante = risque de churn sur un front solo neuf). À ré-évaluer au kickoff.

**Svelte 5 (3e)** : le moins de boilerplate à *lire* aiderait directement J4 — mais le **gap de corpus runes** (LLM entraînés sur Svelte 4 pré-runes, mitigation = injecter `llms.txt` à chaque session) est une taxe côté-écriture-agent qui domine le gain côté-lecture. **Lit (4e)** : meilleure longévité-standards, **mais a11y = 2** (aucune primitive headless niveau Base-UI ; on réécrit focus-trap/listbox/positionnement à la main = coût solo prohibitif). **Rust-WASM (5e, rejeté)** : le plus séduisant en souveraineté-langage, mais **les deux libs load-bearing sont JS** (CM6+xterm) → colle wasm-bindgen sur les surfaces les plus chaudes ; corpus minuscule + churn macro `view!`/`rsx!` → code souvent non-compilant. Rejeté **pour ce front précis**, pas dans l'absolu.

> **Ne pas vendre React comme une victoire de premiers principes.** C'est le bon choix pour la **bonne** raison : incumbence + écosystème headless mûr + interop, avec la fluence-agent en appui. Ré-ouvrir Solid si 2.0 GA.

---

## 4. L'information architecture — l'établi bi-focal

### 4.1 Ce qu'on dépasse

La R&D antérieure (`factory_interface_paradigm_rnd.md`) concluait « tri-zone : Intentions à gauche / Atelier au centre / **Dock de vérification à DROITE** ». On la **dépasse** : J4 (diff) ≈ J3 (steering) sont **co-dominants** et J4 est le **bottleneck 2026** (Sonar : 96 % ne font pas confiance au code IA, 48 % seulement le vérifient ; la revue d'une suggestion IA = 4,3 min vs 1,2 min humain). **Un dock permanent à droite sous-dimensionne structurellement la seule surface où il faut le plus investir.** Et un « verification-centric pur » par défaut produirait un **mur vide** en début de phase (on ne vérifie pas ce qui n'existe pas).

**D'où le bi-focal :** la scène suit le **centre de gravité du travail**, qui oscille entre produire (STEER) et contrôler (VERIFY) — exactement le rythme Plan-Mode ⇄ Execution de Claude Code. Bénéfice cardinal solo : **exactement une chose focale à la fois** (leçon Zed « agent threads vs text threads »). Et c'est **moins cher** qu'un tri-zone (un seul gros chantier — le diff-viewer — au lieu de trois panneaux à équilibrer en permanence).

### 4.2 Élimination adversariale des 4 autres cadres

| Cadre candidat | Pedigree 2026 | Verdict | Récupéré sous forme réduite |
|---|---|---|---|
| **Multi-agent board** | Cursor 3, Antigravity Mission Control, Zed parallel | **Rejeté** : pattern d'échelle/équipe ; Factory = solo, single-PTY, process séquentiel (Phase 0→A→B avec gate ENTRE) | **liste de sessions** (tiroir), jamais un board live concurrent |
| **Command-palette-first** | Raycast/Linear | **Rejeté comme cadre** : heurte « intentions pas jargon » | **⌘K transversal** d'intentions en clair, accélérateur pas cadre |
| **Timeline-de-procédé** | spec-driven / Antigravity | **Rejeté comme cadre** : coût graphe + drift canvas↔repo | **rail read-only** (altitude 0), projection du repo, 0 moteur de graphe |
| **Verification-centric pur** | recherche bottleneck | **Rejeté comme défaut** : mur vide en début de phase | **devient le MODE VERIFY**, plein cadre quand il y a un diff/gate |

### 4.3 Les 3 altitudes (2 seulement prennent la scène)

- **Altitude 0 — Rail d'orientation** (ambiant, permanent, jamais le cadre). Répond J1 en continu. Timeline-de-procédé **réduite à un ruban read-only** (zéro drift, zéro graphe). **N'entre jamais dans une transition de mode** = ancre de continuité.
- **Altitude 1 — MODE STEER** (J2 → J3). Défaut quand une session produit ou qu'aucun diff n'est pendant.
- **Altitude 2 — MODE VERIFY** (J4 + J5 + J6 + J7). Plein cadre quand il y a quelque chose à contrôler.

### 4.4 Règle de bascule (l'innovation vs tri-zone statique)

1. **Défaut = STEER** tant que la session stream ou qu'aucun diff n'est pendant.
2. **Jamais d'auto-bascule en plein stream** (ne pas arracher l'humain au steering — leçon Zed). Exactement **un** mode focal, **étiqueté**.
3. Fin de tour agent **ET** diff/gate frais pendant → le **pouls de gate** s'allume dans le rail + affordance « Diff prêt » → raccourci (ou auto, configurable) pivote vers VERIFY. Candidat **View Transitions** (mouvement = changement d'état).

### 4.5 Maquette — MODE STEER

```
┌─ ÉTABLI FACTORY ─────────────────────────────────────────────────────────────────────┐
│ Sprint 80 · Phase D  ▸  master  ▸  ● 3 modifiés · 1 indexé  ▸  ◑ gates 4✓ 1•           │ ← rail (J1), toujours là
├──────┬────────────────────────────────────────────────────────────────────────────────┤
│ ▸STEER│  INTENTION                                              Claude ▾   · profond     │ ← provider = attribut (J2)
│  VERIF│  ┌──────────────────────────────────────────────────────────────────────────┐  │
│  ─────│  │ « Génère la vue de preuve additive du Viewer depuis le pack S80-D. »      │  │
│  ⌘term│  └──────────────────────────────────────────────────────────────────────────┘  │
│  ⌘sess│   Presets ▸ [Préparer la phase]  [Vérifier avant validation]  [Transmettre…]   │ ← intentions, pas jargon
│  ⌘hist│   ▸ détails techniques (kind=app-authoring · pack blake3:9f3c… · 0 PASS)   ⌄    │ ← jargon REPLIÉ
│  ⌘know├────────────────────────────────────────────────────────────────────────────────┤
│       │  ATELIER OBSERVABLE  ·  session #7  ·  ● en cours                            ⏸ ⟳ │ ← steering SSE (J3)
│       │  │ ◆ outil  Edit  app/proof-view.js     +38 −4        [voir le diff ▸ VERIF] │  │ ← carte tool-call → pivote VERIF
│       │  │ ⊹ « j'assemble le score par couches additives ; N3 jamais dérivé »         │  │
│       │  ┌─ relance ────────────────────────────────────────────────────────────────┐  │
│       │  │ ⟳ « garde N3 grisé tant qu'aucun artefact signé »             ⌘↵ envoyer  │  │
│       │ ⚑ Intention sensible détectée : « commit ». ──────────────────────────────────  │ ← le MUR requires_gate
│       │    Exige une vraie session agent + gates + preuves repo.                         │
│       │    L'Operator ne peut pas l'exécuter seul.    [Préparer le pack pour la session] │ ← seule action ; jamais "Forcer"
└──────┴────────────────────────────────────────────────────────────────────────────────┘
```

### 4.6 Maquette — MODE VERIFY (le vrai investissement, **séquencé** — cf. §4.8)

```
┌─ ÉTABLI FACTORY ─────────────────────────────────────────────────────────────────────┐
│ Sprint 80 · Phase D  ▸  master  ▸  ● 3 modifiés · 1 indexé  ▸  ◑ gates 4✓ 1•           │
├──────┬───────────────────────────┬────────────────────────────────────────────────────┤
│  STEER│ CHANGE-SET  (session #7)  │  [ Diff ]   Aperçu scellé   Preuve                  │ ← onglets ; Diff par défaut
│ ▸VERIF│ ▸ app/proof-view.js +38−4 │  app/proof-view.js                       hunk 1 / 3 │
│  ─────│    FG6 secrets ✓          │  │  − verdict = "✓ Vérifié"                     │  │ ← l'anti-pattern banni
│  ⌘term│   factory-data.js   +51   │  │  + layers.forEach(l => addSegment(l))        │  │
│  ⌘hist│  Vérité = git diff (Rust),│  │  + // N3 jamais dérivé : reste grisé          │  │
│  ⌘know│  pas un buffer périmé      │  Ce hunk →  [Transmettre la correction à #7]       │ ← INTENTION, jamais "Approve"
│       ├───────────────────────────┴────────────────────────────────────────────────────┤
│       │ GATES   (diagnostic · 1:1 artefact · l'Operator ne clôt aucun verdict)          │ ← J5, honnête
│       │  FG-CSP authoring  ✓ passed   run@b3f1   · BLOQUANT · hors skip                 │
│       │  FG5 sandbox ✓    FG6 secrets ✓    FG4 diff ◦ informatif (n'avorte pas)         │
│       │  FG7 preview •  2 issues ▸     FG8 provenance — non exécuté (pas de publish)     │ ← « non exécuté » ≠ « PASS »
│       │ ÉTAT :  ⧗ En attente de session agent — gates 4✓ 1• · 0 verdict auto-clos       │ ← le slot ÉTAT ne dit JAMAIS « PASS »
└──────┴────────────────────────────────────────────────────────────────────────────────┘
```

### 4.7 Le pattern de vérification — « Décomposition, jamais verdict » (le livrable central)

1. **Trois artefacts co-localisés** (Antigravity « verify with Artifacts, not logs ») : DIFF (ce qui a changé) · GATES (ce que disent les checks Rust déterministes, **1:1**) · APERÇU SCELLÉ (ce que ça rend, sous CSP prod) ; + la **chaîne de preuve** au publish. Jamais un scrollback de logs comme surface de contrôle.
2. **Slot ÉTAT = machine d'états énumérée nommée** (constante miroir backend, façon `feedback_named_constants`), **jamais une chaîne libre « PASS »**. États : `Aucun changement` · `En cours (agent)` · `Diff en attente de revue` · `Gates: N✓ M• K✗` · `Brouillon non-autoritaire` · `Transmis à la session sous gate`. Les chaînes `PASS`/`Vérifié`/`Approuvé` sont **interdites** dans ce slot — **gardées par un scan front** (jumeau de `scan-en-strings.sh`) en plus du refus serveur.
3. **Zéro verdict calculé par l'UI** : chaque badge mappe 1:1 un champ `GateResult{passed,name,issues}`. États `non exécuté`/`informatif`/`BLOQUANT`/`2 issues` **distincts et visibles**, jamais aplatis en vert/rouge binaire.
4. **Actions de hunk = INTENTIONS routées à la session agent** (qui ré-applique **sous gate**), jamais des mutations directes. Libellés conformes au refus serveur : `[Transmettre la correction à #7]` / `[Signaler ce hunk à revoir]` — **jamais** `[Approve]`/`[Merge]`/`[Commit]`.
5. **Provenance de fraîcheur du gate** : chaque badge porte `run@b3f1` ; un gate dont le diff a bougé depuis affiche `◦ obsolète, relancer`.
6. **Le mur `requires_gate` est un MUR** : seule action « Préparer le pack » — **aucun** « Forcer »/« Override »/« bypassPermissions ». C'est l'avance gouvernance de SBFB sur Cursor/Windsurf/Devin ; on l'expose, on ne la dilue pas.
7. **Aperçu = unique chemin de rendu scellé** (iframe blob-serve + `BLOB_SERVE_CSP`), jamais un preview privilégié inline.

### 4.8 Honnêteté de séquencement (intégrée depuis l'adversarial) — le mode VERIFY est partiellement **aspirationnel**

La surface VERIFY présentée comme « le gros investissement page-1 » s'appuie sur des capacités backend qui **n'existent pas encore** (vérifié dans le code) :

- **J5 gates : aucune route GET dédiée.** La donnée gate arrive via `run_publish_pipeline` (publish, CLI) et `audit-commit`/`lint` (actions). Le dock l'assume : gate non exécuté = `— non exécuté` (≠ échoué, ≠ PASS), avec `[Lancer audit ▸]` (action allowlistée) pour le peupler.
- **J4 diff working-tree : aucune route.** `git diff` n'existe que pour `sprint-history/diff/{sha}` (commits passés, `sprint_history.rs`) ; `dirty_files`/`staged_files` (`operator_server.rs:419-420`) sont une **liste de fichiers**, pas le contenu du diff. Le diff-viewer bespoke exige une **nouvelle route Rust** `GET /api/git/diff` (working-tree, calculé en Rust = vérité repo).
- **J8 publish : pas une route Operator** ; `run_publish_pipeline` est CLI (cohérent avec le mur).

**Conséquence (correction adversariale §3 actée) :** on **séquence**. Le coin le plus mûr backend (diff/gates/publish) est précisément celui que **le terminal/CLI sert déjà nativement**. **MVP = STEER (entièrement câblé) + rail + terminal PTY élevé comme surface VERIFY de bootstrap.** Le diff-viewer bespoke + panneau gates ne se construisent **qu'après** l'ajout des routes manquantes. Ne pas présenter « VERIFY-mode-avec-diff-viewer » comme livrable page-1.

---

## 5. Styling + composants + motion (sans daisyUI/anime)

**Fait cardinal :** les agents écrivent le front → la **fluence-LLM du couple (framework + style)** est un axe de maintenabilité de 1er rang. Données 2026 : *« AI is most fluent in Tailwind today because that is where the training data is densest »* (~90 % utilisable au 1er jet vs CSS-modules « naming varies wildly »). **L'Operator est régime-1 (outil local) hors `BLOB_SERVE_CSP`** → styling/motion/fonts **non bridés** (la CSP scellée ne s'applique qu'aux apps produites).

### 5.1 Styling — **Tailwind CSS v4 CSS-first + tokens oklch maison**, zéro kit de composants-CSS

| Approche | Licence | Fluence-agent | Runtime prod | Verdict |
|---|---|---|---|---|
| **Tailwind v4** (Oxide/Rust) | MIT | **#1 (corpus le + dense)** | **0** (CSS statique) | **✅ RETENU** |
| UnoCSS | MIT | moyenne | 0 | dominé (gain bundle non décisif hors-CSP, perte fluence) |
| PandaCSS | MIT | basse-moyenne | 0 | sur-outillé pour solo |
| vanilla CSS / CSS-modules | — | basse | 0 | rejeté : tue l'axe fluence-agent |
| CSS-in-JS | MIT | moyenne | **runtime** | rejeté (runtime + en déclin 2026) |

**daisyUI RETIRÉ côté Operator** : kit opinioné (`.btn .card .badge`) qui **possède le markup** au registre SaaS, ajoute un plugin, verrouille le thème. Pour un outil local sobre, on veut le contrôle total **utilitaires + tokens maison**. **daisyUI reste légitime UNIQUEMENT pour les apps scellées que la Factory _produit_** (template `sbfb-reflect`, knowledge-pack CSP-safe S79) — public/contexte différent. **Ne pas confondre le style des apps-produites avec le style de l'outil-Operator.**

> **Dette corpus à nommer (correction adversariale §4) :** le « ~90 % » mesure le corpus utilitaires (v3-dominant). Tailwind **v4 est CSS-first** (`@theme`/`@custom-variant`, config dans le CSS) — surface 2025-récente sous-représentée à l'entraînement : les agents émettent parfois un `tailwind.config.js` v3 **qui n'existe plus en v4**. Mitigation = petite surface posée une fois + un fichier de tokens canonique + un lint. On **nomme** cette dette au lieu de la masquer (la même pénalité de récence-corpus appliquée aux runes Svelte doit l'être ici).

### 5.2 Composants & a11y — **Base UI 1.0 headless**, recadré « comportement d'interaction » (pas a11y-lecteur-d'écran)

| Lib | Licence | Maintenance 2026 | Verdict |
|---|---|---|---|
| **Base UI 1.0** (`@base-ui/react`) | MIT (équipe MUI/ex-Radix/Floating-UI) | **GA 11 déc. 2025**, 35 composants, latest 1.6.x ; **jeune (churn 1.0→1.6 en ~6 mois)** | **✅ RETENU** |
| Radix UI | MIT | éprouvé mais updates ralenties (WorkOS) | dominé — mais **stabilité = atout OpenBSD** (cf. risque ci-dessous) |
| React Aria | Apache-2.0 | active (Adobe) | hedge si a11y-SR devient priorité absolue |
| Ark UI | MIT | active (Zag, **multi-framework**) | **hedge si le framework bascule** |

**Doctrine d'usage minimaliste.** On importe Base UI **uniquement** pour les ~8 primitives à **comportement-d'interaction coûteux et dangereux à coder seul** : **Dialog** (focus-trap du mur `SENSITIVE_ACTIONS`), Popover/Menu, Tabs (altitudes), Select/Combobox (provider), Tooltip, Toast, Collapsible (replier le jargon — pattern intentions-pas-jargon), ScrollArea. **Recadrage adversarial acté :** Base UI sert le **comportement correct** (focus-trap, navigation clavier, positionnement collision-aware), **pas** une revendication a11y-lecteur-d'écran — pour un outil **mono-utilisateur** (le user = le dev), l'argument SR est faible ; on ne le sur-vend pas. Tout le **vocabulaire métier reste tout-maison** (HTML sémantique + Tailwind, 0 dépendance) : cartes diff/tool-call, lignes de gate, Proof Card, chrome terminal, rail, composeur — c'est là que vit l'identité visuelle.

> **Risque assumé (adversarial) :** Base UI est la lib **la plus jeune** du stack. On le retient par **incumbence** (déjà au `package.json`, consolide 6 `@radix-ui/*` en 1 paquet) + maturité des auteurs, **en assumant le churn**. Alternative conservatrice cohérente avec la lignée OpenBSD : **garder Radix** (battle-tested, « updates ralenties » = stabilité). Décision PO (cf. §10). Hedge framework : Svelte → Bits/Melt ; Solid → Kobalte ; incertitude → **Ark UI** (multi).

### 5.3 Motion — **CSS natif + View Transitions API + WAAPI**, 0 lib par défaut

| Approche | Licence | Verdict |
|---|---|---|
| **CSS transitions/keyframes + View Transitions API** | natif (0 dep) | **✅ RETENU (défaut)** — Baseline same-document 2026 (Chrome 111+, Safari 18+, Firefox 144+), respecte `prefers-reduced-motion` nativement |
| **WAAPI** (`element.animate()`) | natif (0 dep) | **✅ RETENU (helper ~30 lignes maison, dette explicite)** pour les coordinations JS (reveal de stream, flip de gate) |
| Motion (ex-framer) `motion/react` | MIT | **escape-hatch sanctionné** (~4,6 kb tree-shaké) si spring/layout/shared-element non exprimable en VT+CSS |
| **GSAP** | **non-OSI** (« no charge » Webflow, IP propriétaire) | **❌ REJETÉ** (hygiène AGPL/lignée libre + sur-dimensionné) |
| anime.js | MIT | **❌ RETIRÉ PO**, non réintroduit |

**Doctrine :** *motion = sens, jamais décoration ; `prefers-reduced-motion` = état final instantané.* L'Operator étant local (navigateur récent contrôlé), on s'appuie sur le Baseline VT sans polyfill, dégradation propre (VT absent → l'état change sans transition, jamais cassé). **Honnêteté :** le « helper WAAPI maison » est une **dette explicite** (discipline maison vs API déclarative type framer) — assumée au nom de la sobriété/souveraineté.

### 5.4 Design system concret — thème oklch dark étendant `sbfb-reflect`

Discipline cardinale : **achromatique par défaut ; la couleur n'apparaît que quand quelque chose est VRAI d'un état** (gate, diff, provider, provenance). Traduction visuelle de « calme/anti-SaaS » + « consommée jamais autoritaire ».

```css
@import "tailwindcss";
@custom-variant dark (&:where([data-theme="dark"], [data-theme="dark"] *)); /* défaut = dark */

:root {
  /* Surfaces achromatiques (alignées sbfb-reflect base-100/200/300) */
  --bg-void: oklch(15% 0 0); --bg-surface: oklch(18% 0 0); --bg-card: oklch(22.5% 0 0);
  --bg-raised: oklch(27% 0 0); --line: oklch(32% 0 0); --line-strong: oklch(42% 0 0);
  /* Encre */
  --ink-hi: oklch(95% 0 0); --ink: oklch(86% 0 0); --ink-mid: oklch(68% 0 0); --ink-lo: oklch(55% 0 0);
  /* LA seule teinte interactive (focus/lien/nav active) */
  --accent: oklch(72% 0.09 230); --accent-ink: oklch(18% 0 0);
  /* État sémantique (hues MIROIR de sbfb-reflect) */
  --ok: oklch(80% 0.18 152); --warn: oklch(85% 0.19 92); --bad: oklch(70% 0.19 22); --info: oklch(75% 0.04 230);
  /* Diff calmes (pas néon) */
  --diff-add-line: oklch(28% 0.05 152); --diff-add-ink: oklch(82% 0.10 152);
  --diff-del-line: oklch(28% 0.05 22);  --diff-del-ink: oklch(78% 0.12 22);
  /* Rayons / élévation sobre (séparation par la ligne, pas l'ombre) */
  --r-field: 0.5rem; --r-box: 1rem; --shadow-pop: 0 1px 2px oklch(0% 0 0 / .4), 0 8px 24px oklch(0% 0 0 / .35);
  /* Motion */
  --ease-out: cubic-bezier(0.2,0,0,1); --ease-fluid: cubic-bezier(0.3,0,0,1);
  --dur-press: 80ms; --dur-control: 160ms; --dur-surface: 240ms; --dur-route: 400ms;
}
@theme inline { /* expose les tokens comme utilitaires Tailwind */
  --color-bg-void: var(--bg-void); --color-bg-card: var(--bg-card); --color-line: var(--line);
  --color-ink: var(--ink); --color-ink-mid: var(--ink-mid); --color-ink-lo: var(--ink-lo);
  --color-accent: var(--accent); --color-ok: var(--ok); --color-warn: var(--warn); --color-bad: var(--bad);
  --radius-field: var(--r-field); --radius-box: var(--r-box);
  --font-sans: 'Geist Variable', ui-sans-serif, system-ui, sans-serif;
  --font-mono: 'Geist Mono Variable', ui-monospace, 'JetBrains Mono', monospace;
}
@media (prefers-reduced-motion: reduce) {
  :root { --dur-press:1ms; --dur-control:1ms; --dur-surface:1ms; --dur-route:1ms; }
  ::view-transition-group(*),::view-transition-old(*),::view-transition-new(*){ animation: none !important; }
}
```

**Typographie — la dualité _est_ le langage « consommée jamais autoritaire » :** **sans = intention/prose humaine** (Geist Variable, déjà vendorée `@fontsource-variable/geist`, 0 Google Fonts) ; **mono = preuve/artefact machine** (Geist/JetBrains Mono OFL, `font-variant-numeric: tabular-nums` sur tous les compteurs). Tout ce qui est ancré-au-hash (hash blake3, diff, sha, noms de gate, provenance Ed25519, terminal) est en **mono**.

**Densité :** deux régimes — composeur d'intention **aéré** (invite au langage naturel) ; surface de vérification **dense, tabulaire, mono** (J4 = bottleneck → max d'info sans bruit). Séparation par `--line`, pas par l'ombre SaaS.

**Tenue « consommée jamais autoritaire » (backend-imposée) :** surfaces advisory (knowledge/prompt/pack) = fond `--bg-card`, **bordure gauche pointillée `--ink-lo`**, **chip hash mono**, label « consultatif — non-autoritaire », contraste réduit — **jamais de vert, jamais de coche** (miroir `chat_history_authoritative:false`). Zone verdict = slot non-actionnable « verdict = session agent + gates + preuves repo » (le front **ne peut pas** rendre un « PASS ✓ »).

**Les 5 signatures de motion (sobres, sens-porteuses, reduced-motion = état final) :** (1) *token settle* — chunk `opacity 0→1` sur 80 ms, **aucun déplacement** ; (2) *gate flip* — `pending→ok/bad` cross-fade 160 ms **sans rebond**, issues en `grid-template-rows 0fr→1fr` ; (3) *verification reveal* — carte diff `translateY(4px)+opacity` 240 ms, hunks staggerés 30 ms ; (4) *altitude shift* — View Transitions cross-fade, **rail exclu** (`view-transition-name: none`) ; (5) *confirmation gravity* — modal `scale 0.98→1` + dim backdrop ~200 ms (le seul motion qui ajoute du **sérieux**, pas du plaisir).

---

## 6. La stack de packages (manifeste concret)

Versions vérifiées registre npm / releases datées juin 2026. Toutes les retenues sont **MIT / ISC / Apache-2.0 / OFL** → **compatibles AGPL-3.0** (permissif → copyleft = OK). 0 GPL/EPL/propriétaire, **0 runtime Node prod**, 0 marketplace.

### 6.1 Décisions de couche (re-débattues depuis zéro)

| Axe | Décision | Raison |
|---|---|---|
| **Framework** | **React 19.2.x** (SPA/SSG statique) | incumbence + écosystème headless mûr + interop CM6/xterm ; fluence-agent en appui |
| **Styling** | **Tailwind v4.3.x SEUL + tokens maison `@theme`** | fluence-agent #1, moteur Rust, 1 dep, 0 runtime ; **daisyUI retiré** |
| **Composants** | **`@base-ui/react` 1.6.x** headless | successeur Radix, 1 paquet vs 6 ; consolide les `@radix-ui/*` |
| **Motion** | **CSS + View Transitions + WAAPI, 0 lib** | sobre/souverain ; **anime.js retiré, GSAP écarté** |
| **State client** | **Zustand 5.0.x** | minimal, agent-fluent, standard SBFB |
| **Server-state REST** | **`@tanstack/react-query` 5.x** | poll/cache/invalidation pour J1/J5 (`/status`,`/context`,`/providers`,`/lint`,`/audit`,`/actions/log`) |
| **Transport SSE/WS** | **cookie HttpOnly → `EventSource` + `WebSocket` natifs** | **cf. §6.5 — change décisive vs la re-dérivation stack** |

### 6.2 `package.json` exemple (cible greenfield)

```jsonc
{
  "name": "factory-operator",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite", "build": "tsc -b && vite build", "lint": "eslint .",
    "preview": "vite preview", "test:unit": "vitest run",
    "test:coverage": "vitest run --coverage", "test:e2e": "playwright test"
  },
  "dependencies": {
    "react": "^19.2.7", "react-dom": "^19.2.7",
    "@base-ui/react": "^1.6.0",
    "tailwindcss": "^4.3.1", "@tailwindcss/vite": "^4.3.1",
    "clsx": "^2.1.1", "tailwind-merge": "^3.6.0", "class-variance-authority": "^0.7.1",
    "zustand": "^5.0.14", "@tanstack/react-query": "^5.101.1",
    "@codemirror/state": "^6.7.0", "@codemirror/view": "^6.38.0",
    "@codemirror/commands": "^6.8.0", "@codemirror/language": "^6.11.0",
    "@codemirror/search": "^6.5.0", "@codemirror/merge": "^6.12.2",
    "@codemirror/lang-javascript": "^6.2.0", "@codemirror/lang-html": "^6.4.0",
    "@codemirror/lang-css": "^6.3.0", "@codemirror/lang-json": "^6.0.0",
    "@xterm/xterm": "^6.0.0", "@xterm/addon-fit": "^0.11.0", "@xterm/addon-web-links": "^0.12.0",
    "lucide-react": "^1.21.0", "@fontsource-variable/geist": "^5.2.9"
  },
  "devDependencies": {
    "vite": "^8.0.1", "@vitejs/plugin-react": "^6.0.1", "typescript": "~5.9.3",
    "@types/react": "^19.2.7", "@types/react-dom": "^19.2.3",
    "eslint": "^9.39.4", "@eslint/js": "^9.39.4", "typescript-eslint": "^8.57.0",
    "eslint-plugin-react-hooks": "^7.0.1", "eslint-plugin-react-refresh": "^0.5.2", "globals": "^17.4.0",
    "vitest": "^4.1.9", "@vitest/coverage-v8": "^4.1.9",
    "@testing-library/react": "^16.3.2", "@testing-library/jest-dom": "^6.9.1",
    "@testing-library/user-event": "^14.6.1", "jsdom": "^29.0.2",
    "@playwright/test": "^1.61.1"
  }
}
```

Licences : MIT (React, Tailwind, Base UI, Zustand, TanStack Query, CodeMirror×N, xterm×N, Vite, Vitest, eslint…), **ISC** (lucide-react), **Apache-2.0** (TypeScript, cva, Playwright [dev]), **OFL-1.1** (Geist). **Aucune incompatibilité AGPL.**

### 6.3 Ce qu'on NE prend PAS (et pourquoi)

| Écarté | Raison |
|---|---|
| **daisyUI** | retiré PO ; classes SaaS qui possèdent le markup ; gardé pour apps scellées seulement |
| **anime.js** | retiré PO ; remplacé par CSS + View Transitions + WAAPI |
| **GSAP** | **non-OSI** (« no charge » Webflow ≠ libre) + sur-dimensionné |
| **Motion / framer-motion** | pas par défaut (spring = registre flashy + dep) ; escape-hatch uniquement |
| **Monaco + `monaco-languageclient`** | 5-10 MB, workers, ~134 deps `vscode-*`, IntelliSense pour geste ~0 % ; CM6 gagne sur le vrai besoin (diff) |
| **6× `@radix-ui/react-*`** (présents au repo) | consolidés dans `@base-ui/react` (1 dep vs 6+) ; **à purger** |
| **`shadcn ^4.8.0`** (présent en **dependency** runtime, `package.json:36`) | un CLI dans les deps de prod = wart ; **à purger** |
| **`diff`/`jsdiff`/`diff-match-patch`** | le diff fait autorité est **calculé en Rust** ; un diff JS divergerait (anti-vérité-repo) |
| **PandaCSS / UnoCSS / Emotion / styled-components** | corpus-agent + petit ou runtime CSS-in-JS → moins fluent + moins sobre |
| **`@microsoft/fetch-event-source`** | rendu **inutile** par la voie cookie (§6.5) ; peu maintenu |
| **Tauri / Electron / code-server / Open VSX** | Day-0 (browser=client) + Node persistant + anti-marketplace (GlassWorm) |

### 6.4 Deps « incumbence vs minimal » — à trancher honnêtement (correction adversariale §5)

La re-dérivation stack listait des paquets qu'elle admettait elle-même sur-dimensionnés. Le Directeur les **étiquette honnêtement** et les sort du défaut :

- **`i18next` + `react-i18next`** : l'Operator est **mono-locale FR local** → un **catalogue TS typé 0-dépendance** suffit. Garder i18next **seulement** si multi-locale est visé (décision PO). **Hors MVP par défaut.**
- **`react-router-dom@7`** : data-router lourd pour une app « 3 altitudes, PAS des pages ». Routing peu profond → **état d'altitude en store Zustand + deep-link minimal** ; si une dep est voulue, `wouter` (~2 kb). **Hors MVP par défaut.**
- **`tw-animate-css`** : redondant avec « motion = 0 lib ». **Retiré.**
- **CodeMirror — trim aux grammaires app-authoring** (js/html/css/json) ; **pas** `lang-rust`/`lang-python`/`lang-markdown` (spéculatifs pour un éditeur secondaire d'apps HTML/JS).
- **Double-source diff — résolu** : `@codemirror/merge` (`unifiedMergeView`) **uniquement** pour l'édition inline à la demande (escape-hatch J14) ; la vue diff par défaut (J4) = composant React maison sur les hunks **JSON de `git diff` Rust** (vérité repo unique). Un seul chemin par geste.

### 6.5 Arbitrages d'intégration (load-bearing)

1. **[#1 — la décision avant le framework] Auth-transport = cookie HttpOnly same-origin.** Vérifié au code : `auth_required` (`auth.rs:252-259`) lit le token **uniquement** dans le header `X-SBFB-Token` — zéro query, zéro cookie ; le middleware garde **aussi** `/api/terminal/ws` (`operator_server.rs:145`). Or **ni `EventSource` ni `WebSocket` ne peuvent poser un header custom** (limite DOM) ; le front actuel ne marche qu'en **dev via le proxy Vite** (`executionChat.ts:8-14,86` l'admet ; `AgentChat.tsx:150` idem pour le WS). En prod sous `ServeDir` same-origin `:3001`, **`/api/chat/{id}/stream` (J3) et `/api/terminal/ws` (J12) renvoient 401** — angle mort total des re-dérivations UX/stack. **Correction :** ajouter un **fallback `Cookie` dans `auth_required`** + un **handler Rust `GET /` qui template `index.html` et pose un cookie HttpOnly same-origin** (le navigateur l'envoie automatiquement sur REST + SSE + WS ; `Host`/`Origin` loopback passent). Effet : **REST + SSE + WS marchent nativement en prod**, **`EventSource` est conservé** (le « EventSource inutilisable » de la re-dérivation stack est un **faux-positif**, vrai seulement sous header-token — modèle de toute façon intenable à cause du WS), et le **parseur SSE `fetch`+`ReadableStream` maison sort du scope**.
2. **`ServeDir` à câbler** : `tower_http::ServeDir` (0 occurrence aujourd'hui dans `sbfb-factory`) sert les assets statiques Vite à l'origin loopback ; tous les `fetch` deviennent **relatifs** (`/api/...`).
3. **Base UI + Tailwind** : styler via `className`/`render` des parts Base UI, variants via **cva**, fusion via **clsx + tailwind-merge** ; palette/rayons définis **une seule fois** en `@theme`.
4. **Motion = View Transitions pour les altitudes** : `document.startViewTransition()` (bascule orientation↔intention↔vérif) ; WAAPI pour le flip de gate ; `@keyframes` Tailwind pour l'apparition des tokens. Aucune lib.
5. **CM6 secondaire, diff-Rust primaire** : évite le double-source-of-truth (cf. §6.4).
6. **Compat Vite 8 (Rolldown)** : `@vitejs/plugin-react ≥6`, `@tailwindcss/vite` aligné sur `tailwindcss` 4.3.x ; vérifier que Vitest 4.x + plugin React tournent sous Rolldown au preflight.

---

## 7. Angles morts & risques (assumés)

1. **[BLOQUANT] Auth-transport SSE/WS en prod.** Le plus grave : sans la voie cookie (§6.5 #1), J3 + J12 = 401 en prod. À trancher **en premier**, **forcé par le terminal WS**. Risque résiduel : un fallback cookie élargit la surface CSRF — mitigé par `Origin`/`Host` loopback déjà enforce + `SameSite=Strict` + cookie limité au loopback.
2. **Mode VERIFY aspirationnel.** Diff-viewer + panneau gates dépendent de routes inexistantes (`GET` gates, `git diff` working-tree). Risque : sur-ingénier une UI pour des données non-exposées. **Mitigation = séquencement** (§4.8) + terminal-as-VERIFY au MVP.
3. **Framework par incumbence, pas premiers principes.** On l'assume explicitement ; risque = se priver du meilleur modèle de stream (Solid). **Mitigation = ré-ouvrir Solid si 2.0 GA** au kickoff.
4. **Base UI jeune (churn 1.0→1.6 en ~6 mois)** vs lignée OpenBSD-stabilité. Risque de breaking minor. **Mitigation = épingler + alternative Radix conservatrice** (décision PO §10) + hedge Ark/Kobalte/Bits si pivot framework.
5. **Dette corpus Tailwind v4 CSS-first** (agents hallucinant `tailwind.config.js` v3). **Mitigation = fichier tokens canonique + lint + petite surface posée une fois.**
6. **Helper WAAPI maison = dette explicite** (vs API déclarative). Assumé au nom de la souveraineté/sobriété.
7. **YAGNI mono-utilisateur** : ⌘K, multi-session, knowledge-inspector, timeline = **différables** (tous secondaires). Risque de scope-creep si construits page-1. **Mitigation = MVP 2-modes + rail.**
8. **`/api/chat/message` = placeholder** (« Agent integration pending ») : le seul vrai chemin agent est la **SSE** — ne pas câbler `/message`.

---

## 8. Chemin de mise en œuvre

### 8.1 Préalable backend (petit, non négociable, le 1er geste)
- **Auth cookie** : fallback `Cookie` dans `auth_required` + handler `GET /` (template `index.html` + pose cookie HttpOnly `SameSite=Strict` loopback). **Débloque SSE + WS en prod.**
- **`ServeDir`** : `tower_http::ServeDir` sert le build Vite statique à `:3001`.

### 8.2 MVP greenfield (ce qui est **câblé** aujourd'hui)
1. **Rail d'orientation** (J1) — `/api/status` + `/api/context` + `/api/sprint-history` (TanStack Query, poll).
2. **MODE STEER** (J2 → J3) — composeur d'intention (`/api/context-pack` + `/api/chat/session` + `/api/providers` + `/api/prompt/{kind}`) → transcript SSE (`/api/chat/{id}/stream`, **EventSource via cookie**), provider = attribut, **mur `requires_gate` inline**.
3. **Terminal élevé comme surface VERIFY de bootstrap** (J12) — `/api/terminal/ws` (**WebSocket via cookie**) + `/api/terminal/sessions` (xterm déjà câblé) : c'est là que diff/gates/publish vivent nativement en CLI **avant** les routes dédiées.
4. **Palette d'actions + journal** (J9) — `ACTION_ALLOWLIST` (4 cmds) + `/api/actions/log`.
5. **Inspecteur knowledge advisory** (J15) + **brouillon non-autoritaire** (J13, PASS-bloqué).
6. **Design system** : `@theme` oklch + Base UI (Dialog focus-trap, Tabs, Select, Collapsible) + motion natif.

### 8.3 Différé (après ajout des routes backend)
- **Diff-viewer bespoke** (J4) — nécessite `GET /api/git/diff` (working-tree, calculé en Rust) → hunks JSON → composant React maison.
- **Panneau gates** (J5) — nécessite une route GET « gates » (ou consommer `audit-commit`/`lint` + pipeline).
- **Aperçu scellé** (J6, iframe blob-serve sous `BLOB_SERVE_CSP`) + **Proof Card** (J7) + **flux publish** (J8, reste CLI/terminal).
- **⌘K intentions**, **multi-session**, **timeline historique** — accélérateurs secondaires.
- **Éditeur CM6 inline** (J14, `@codemirror/merge` à la demande).
- **i18next / router dédié** — seulement si multi-locale / deep-link riche est voulu (§6.4).

### 8.4 Rappel process (non négociable, README §4)
- **Phase 0 = audit gate** du sprint précédent **avant** tout code.
- **Gate de testabilité par-sprint** : **T1 E2E Playwright hermétique BLOQUANT-vert** au wrap-up (+ CI chaque push) ; **T2 acceptance = artefact JSON machine-lisible** (`PASS`/`BLOCK{diagnosis}`/`RIG-ABSENT`). Le T1 doit couvrir au minimum : ouverture loopback authentifiée (cookie), composeur → session, **stream SSE token→Done**, et le **mur `requires_gate`** (intention sensible → carte mur, jamais exécution).
- Process per-phase complet (deep preflight 5 scans + review + Codex) sur **chaque** phase ; commit atomique par phase, body riche (delta tests cumulé + scope cuts).

---

## 9. Questions ouvertes PO

1. **Auth-transport (BLOQUANT) :** valider la voie **cookie HttpOnly same-origin** (vs garder header-token + accepter de perdre SSE/WS en prod, ce qui est intenable). C'est la 1re décision d'architecture.
2. **Framework — fenêtre Solid :** ré-ouvrir **SolidJS** si Solid 2.0 passe **GA avant le sprint** ? Sinon React 19 ferme la question.
3. **Base UI jeune vs Radix éprouvé :** assumer le churn Base UI (consolide les `@radix-ui/*`, déjà au repo) **ou** rester sur Radix pour la stabilité lignée-OpenBSD ?
4. **Greenfield total vs migration :** jeter `tools/factory-operator` intégralement (le brief le dit) **ou** re-skin in-place en purgeant `@radix-ui/*` + `shadcn` + `tw-animate-css` ? (Le greenfield est plus propre ; la migration sauve les tests existants.)
5. **i18n & router :** garder `i18next`/`react-router` (incumbence) ou passer au catalogue TS 0-dep + état d'altitude en store (minimal) ?
6. **Routes backend à prioriser :** ordonner `GET /api/git/diff` (working-tree) et la route GET gates — ce sont les **prérequis du mode VERIFY plein**. Lesquelles dans le 1er sprint front ?
7. **Périmètre éditeur CM6 :** confirmer J14 « inspection/patch secondaire » (grammaires js/html/css/json) ou besoin d'un éditeur plus complet (rouvrirait Monaco, déconseillé) ?

---

### Fichiers-ancres (absolus)
- `C:\Users\FlowUP\Documents\Code\nexus\crates\sbfb-factory\src\auth.rs` (`:229-262` header-only, **fallback cookie à ajouter**)
- `C:\Users\FlowUP\Documents\Code\nexus\crates\sbfb-factory\src\operator_server.rs` (`:35` SENSITIVE_ACTIONS, `:123-147` routes, `:419-420` dirty/staged liste, **`ServeDir`+`GET /` à ajouter**)
- `C:\Users\FlowUP\Documents\Code\nexus\crates\sbfb-factory\src\gates.rs` (`run_gate_csp_authoring`), `pipeline.rs` (gates→publish, CLI), `provider_router.rs` (`ExecutionTarget` StreamChunk unique)
- `C:\Users\FlowUP\Documents\Code\nexus\crates\nexus-core-rs\src\csp.rs:33` (`BLOB_SERVE_CSP`, **ne s'applique PAS à l'Operator**)
- `C:\Users\FlowUP\Documents\Code\nexus\tools\factory-operator\package.json` (`@base-ui/react ^1.5` + 6 `@radix-ui/*` + `shadcn ^4.8` runtime + `tw-animate-css` + `i18next` à purger/trancher) ; `src\lib\executionChat.ts:86` + `src\pages\AgentChat.tsx:150` (EventSource/WebSocket dev-proxy-only) ; `vite.config.ts:34-55` (token dev-proxy)
- R&D dépassée : `.planning\research\factory_interface_paradigm_rnd.md` (tri-zone → bi-focal ; React-reskin/daisyUI+anime ré-ouverts), `factory_front_best_approach_research.md`, `factory_embedded_ide_study.md`, `factory_front_redesign_daisy_anime_design_study.md`

### Sources web (juin 2026)
Tailwind v4 (releases tailwindlabs) · Base UI 1.0 (npm `@base-ui/react`, base-ui.com releases v1-0-0, InfoQ) · GSAP licensing (gsap.com, Webflow blog — non-OSI) · Motion (motion.dev, MIT) · View Transitions API + caniuse + `prefers-reduced-motion` (MDN) · Vite 8 (vite.dev) · TanStack Query v5 · `@codemirror/merge` (npm) · Cursor 3 (InfoQ/cursor.com) · Google Antigravity Agent Manager « verify with Artifacts, not logs » · Zed Agent Panel/ACP · Claude Code Plan Mode · Sonar/SRLabs/Aviator (verification bottleneck).
