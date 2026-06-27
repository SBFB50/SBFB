# Factory Operator — Catalogue des fonctionnalités front (S80 + vision)

> Outil de dev LOCAL SOLO bi-focal (PAS un IDE, PAS un SaaS) : rail d'orientation permanent + scène mono-focale STEER (intention → observer/relancer l'agent) ⇄ VERIFY (diff → gates → preuve). Front greenfield S80 (React 19 + Tailwind v4 oklch + Base UI + Motion, hors CSP scellée, AGPL-3.0).
> **Légende statut** : `codé` = backend livré (Phase A auth, F diff) · `à-coder S80` = front à construire · `backend-neuf S80` = route Rust restante (G, I) · `S81` / `future` = différé.
> **Ancrages** : `crates/sbfb-factory/src/operator_server.rs` (routes), `sprint_history.rs` (modèle de procédé), `process.rs`, `csp.rs:33`, `scripts/check-frontier-contracts.sh`.

---

## 0. Résumé exécutif

Le front S80 est greenfield : *rien* n'est encore consommé par la nouvelle UI, mais le backend `sbfb-factory` sert **déjà** un gisement considérable — surtout l'arbre de procédé sprint→phase→commit→artefact (`sprint_history.rs`, entièrement calculé, jamais surfacé richement), le diff en hunks JSON (working-tree + commits passés, même format), le rejeu asciinema des runs de gates, le journal d'actions et les rejets du MUR, les hashes blake3 du context-pack. **Un seul ajout backend reste vraiment nécessaire pour le cœur bi-focal S80 : `GET /api/gates` (Phase G)** ; tout le reste se câble sur des routes existantes. Les **top opportunités quasi-gratuites doctrine-safe** : surfacer l'arbre de procédé (preuve de procédé unique au monde), le diff-viewer bi-usage WT/commit, le panneau gates à états distincts non-aplatis, le journal du MUR, l'inspecteur de context-pack hashé. Garde-fou structurant : tout verdict est **restitué** (gravé au commit par le Rust), jamais **calculé** par l'UI ; Aperçu scellé + Proof Card + Viewer = S81.

---

## 1. Fonctionnalités front PLANIFIÉES S80 (par surface)

### Surface 0 — Backend habilitant (BLOQUANT, non-UI)
| Capacité | Route / ancre | Statut |
|---|---|---|
| Auth cookie HttpOnly + bootstrap 303 | `GET /?token=` `handle_bootstrap` `:302/:152-157` | **codé (A)** |
| ServeDir SPA + CSP `default-src 'self'` | `:163-164`, `operator_csp_middleware :348` | **codé (A)** |
| Diff working-tree en hunks JSON (vérité Rust) | `GET /api/git/diff` `handle_git_diff :1316` | **codé (F)** |
| État gate-live `GateResult{passed,name,issues}` + `run@<rev>` | `GET /api/gates` | **backend-neuf S80 (G)** |
| Cible `ExecutionTarget` echo/fixture (SSE single-Done déterministe T1) | `provider_router.rs` | **backend-neuf S80 (I)** |

### Surface 1 — RAIL d'orientation ambiant (altitude-0, n'anime jamais) — Phase C
| Feature | Route | Statut |
|---|---|---|
| Barre d'orientation haute mono (breadcrumb `Sprint · Phase ▸ branche ▸ N modifiés ▸ pouls gates`) | `GET /api/status :171` + `GET /api/gates` | à-coder S80 (C), pouls après G |
| Provider = attribut discret (`agent: claude-code ▾`, jamais CTA) | `GET /api/providers :177` | à-coder S80 (C) |
| Indicateur loopback + token ✓ (`● loopback :7878`) | `GET /api/status` | à-coder S80 (C) |
| Sélecteur de MODE focal STEER ⇄ VERIFY (bascule manuelle, badge `1 diff`) | état d'altitude (store) | à-coder S80 (C ; câblé H) |
| Entrées sous-surfaces (Terminal ⌥T, Sessions, Historique, Knowledge `consult.`) | terminal / chat / sprint-history / context | à-coder S80 (C nav ; contenus D) |

### Surface 2 — STEER (variante B : atelier dominant, composeur en dock) — Phase C
| Feature | Route | Statut |
|---|---|---|
| Composeur d'intention en dock (grand à l'état-vide pour découvrabilité) | — | à-coder S80 (C) |
| Presets d'intention (chips « Préparer la phase » / « Vérifier avant validation » / « Transmettre à un autre agent ») | — | à-coder S80 (C) |
| CTA « Lancer l'intention » → crée la session | `POST /api/chat/session :181` + `/send :183` | à-coder S80 (C) ; T1(2) |
| Repli « ▸ détails techniques » (kind · provider · preflight · hash) | `GET /api/prompt/{kind} :174`, `/context :175`, `/context-pack :176` | à-coder S80 (C) |
| Sélecteur agent/provider inline | `GET /api/providers` | à-coder S80 (C) |
| Atelier observable (transcript SSE direct, tool-calls/prose) | `GET /api/chat/{id}/stream :184` (cookie) | à-coder S80 (C) ; T1(3) single-Done PO-14 |
| Cartes tool-call / édition (`+N −M` + « voir le diff → » route VERIFY) | flux SSE / `/chat/{id}/log :185` | à-coder S80 (C) |
| Contrôles de stream ⏸ Interrompre · ↻ Relancer (jamais auto-bascule) | client / re-stream idempotent | à-coder S80 (C) |
| Indicateur d'état session neutre (`travaille` / `streaming` / `2 éditions · terminé`) | — | à-coder S80 (C) |

### Surface 3 — Le MUR de gouvernance (barrière en-flux, pleine largeur) — Phase D
| Feature | Route / ancre | Statut |
|---|---|---|
| Barrière `requires_gate` inline (scinde le flux, hachures/cadenas, **jamais un modal**, 0-spawn) | réponse `requires_gate:true :766-779`, `SENSITIVE_ACTIONS :37` | à-coder S80 (C inline / D plein) ; T1(4) |
| Décomposition « la session produira & exigera » (session · gates · preuves · signataire) | — | à-coder S80 (D) |
| Action unique « Préparer le pack pour la session » + « Retour » ; **0 Forcer/Override/Bypass** | `POST /api/artifacts/draft :700` | à-coder S80 (D) |

### Surface 4 — VERIFY (change-set + onglets + bande gates) — Phases D (bootstrap) / H (plein)
| Feature | Route | Statut |
|---|---|---|
| Colonne change-set repliable (fichiers + `+N −M` + marqueur gate par fichier) | `/api/git/diff` (F) + `/api/gates` (G) | à-coder S80 (H) |
| **Diff-viewer bespoke React** sur hunks JSON Rust (le vrai investissement) | `GET /api/git/diff` (F) | à-coder S80 (H) ; T1(5) |
| Actions de hunk = intentions (« Transmettre la correction → », « Signaler ») — **jamais Approve/Merge/Commit** | `POST /api/chat/{id}/send` | à-coder S80 (H) |
| Onglet « Diff » (défaut) | — | à-coder S80 (H) |
| Onglet « Aperçu scellé » (iframe rendu réel + `render-seal sha256`) | blob-serve / `compute_output_hash` | **différé S81** (onglet désactivé « à venir ») |
| Onglet « Preuve » / Proof Card (chaîne provenance + couches + manques) | provenance SLSA L1 | **différé S81** (onglet désactivé « à venir ») |
| Bande GATES permanente — **états distincts jamais aplatis** (`✓/•/✕/N issues/—/PROVISIONAL/Not evidenced/RIG-ABSENT`), `▾ détails` | `GET /api/gates` (G) | panneau à-coder S80 (H) |
| Slot ÉTAT (machine d'états énumérée nommée) — **ne dit JAMAIS PASS**, 0 verdict UI | constante miroir | à-coder S80 (H) ; scan anti-PASS BLOQUANT |
| Provenance de fraîcheur (`run@<rev>`, `◦ obsolète, relancer`) | `GET /api/gates` | à-coder S80 (H) |
| Bascule bi-focal STEER↔VERIFY state-driven (View-Transitions, rail exclu, jamais arrachée au stream) | état d'altitude | à-coder S80 (H) |

### Surface 5 — Terminal PTY + surfaces secondaires/consultatives — Phase D
| Feature | Route | Statut |
|---|---|---|
| Terminal xterm/PTY (tiroir) élevé en **VERIFY de bootstrap** | `GET /api/terminal/ws :198` + `/sessions :199` | à-coder S80 (D) |
| Inspecteur knowledge advisory (pointillé, contraste réduit, chip hash, « N'affecte aucun verdict ») | `GET /api/context` (`authoring_knowledge :430`) | à-coder S80 (D) |
| Brouillon de plan non-autoritaire (☐ « aucune case ne valide », chip `non autoritaire`, refuse PASS) | `POST /api/artifacts/draft :700` (refus PASS `:722-758`) | à-coder S80 (D) |
| Tiroir Sessions (liste simple + compteur ; board multi-agents coupé) | `POST /chat/session` + `GET /chat/{id}/log :1179` | à-coder S80 (D) |
| Journal d'actions allowlistées | `POST /actions/run :178` + `GET /actions/log :689` | à-coder S80 (D) |

### Surface 6 — Historique de procédé / diff de commits passés — Phase D
| Feature | Route | Statut |
|---|---|---|
| Visualiseur de diff d'un commit passé (0 ajout) | `GET /api/sprint-history/diff/{sha} :1294` | à-coder S80 (D) |
| Arborescence sprint→phase→commit→artefact (MVP sprint actif) | `GET /api/sprint-history :186` / `/all :187` / `/{sprint} :189-191` | à-coder S80 (D MVP) ; profond S81 |

### Surface 7 — Design-system oklch + motion — Phase E (transversal)
| Feature | Statut |
|---|---|
| Tokens oklch canoniques (achromatique par défaut, couleur = signal d'état VRAI) | à-coder S80 (E) |
| Dualité typo Geist sans (intention) / mono (preuve), vendorée fontsource 0-CDN, `tabular-nums` | à-coder S80 (B install / E langage) |
| 5 signatures de motion sens-porteuses (allowlist figée) + `MotionConfig reducedMotion='user'` | à-coder S80 (E) |
| États d'interaction des atomes (rest/hover/focus-visible/active/disabled) | à-coder S80 (E) |

### Surface 8 — Scaffold + gates de discipline + testabilité — Phases B / I
| Feature | Statut |
|---|---|
| Scaffold React 19 + Tailwind v4 `@theme` + Base UI + Motion `LazyMotion`+`m` + jettison `tools/factory-operator` ET `tools/factory-ui` | à-coder S80 (B) |
| 5 gates BLOQUANTS : (1) 0 `@radix-ui` runtime · (2) anti-`tailwind.config.js` v3 · (3) `size-limit` chiffré · (4) anti-`motion.*` nu · (5) scan anti-PASS (hors slot ÉTAT) | à-coder S80 (B) |
| T1 Playwright hermétique (cookie-boot / composeur→session / SSE single-Done / MUR 0-spawn / diff-viewer+gates sans PASS) + T2 JSON committé + re-couverture SSE single-Done | backend-neuf + à-coder S80 (I) |

---

## 2. Données backend DÉJÀ disponibles non surfacées — quick wins

Toutes ces routes existent et servent des données calculées **jamais affichées richement**. Aucune ne demande de backend neuf (sauf mention).

| # | Gisement | Route / ancre | Ce que le front en fait | Effort |
|---|---|---|---|---|
| QW1 | **Arbre de procédé complet** (`SprintHistoryResult` : phases[preflight_verdict/review_verdict/codex_confirmed-partial-gap/findings P0-P3/deliverables/files_changed/deltas], commits[body_sections], tests.per_phase, scope_cuts, carries, preflight_bilan, verification) | `GET /api/sprint-history[/{n}]` `sprint_history.rs:9-180` | Spine VERIFY « preuve de procédé » — restitution, 0 calcul UI | M |
| QW2 | **Diff de n'importe quel commit historique** (même format hunks que `/api/git/diff`) | `GET /api/sprint-history/diff/{sha} :1294` | Diff-viewer bi-usage gratuit (WT + commit passé) | S |
| QW3 | **Index tous-sprints** (`AllSprintsResult{sprints[],total}`) | `GET /api/sprint-history/all :187` | Ruban carrière / timeline globale v1.0→v2.1, drill-down | S |
| QW4 | **Rejeu des runs terminal/gates passés** (`.cast` asciinema) | `GET /api/terminal/sessions :1216` + `/{name} :1223` | Terminal-as-VERIFY montre l'historique, pas que le live | M |
| QW5 | **Journal d'actions + rejets du MUR avec raison** | `GET /api/actions/log :689` | Journal de bord du nœud ; rend le MUR visible | S |
| QW6 | **Rejeu d'une session STEER** (context_pack + messages, dont `requires_gate`) | `GET /api/chat/{id}/log :1179` | Tiroir Sessions rouvrable (in-process) | M |
| QW7 | **Conformité commit** (9 sections body, review PASS exact, codex_review présent → `issues[]`) | `GET /api/audit/{rev} :393` | Annotation « N manques » par commit (Rust juge, UI liste) | S |
| QW8 | **Hygiène planning** (ORPHAN_FILE, STALE_PASS_PENDING, INVALID_VERDICT_FORMAT, PLAN_WITHOUT_KICKOFF) | `GET /api/lint :375` | Hygiène du sprint courant | S |
| QW9 | **Matériau knowledge advisory hashé** (`authoring_knowledge` `{path,hash,exists}`) | `POST /api/context-pack :533` (`:498-511`) | Rendu pointillé/consultatif | S |
| QW10 | **Prompt réellement transmis** (11 kinds × 5 providers × depth, `strip_cloud_references` local) | `GET /api/prompt/{kind} :420`, `process.rs:7-23` | Prévisualisation sous « détails techniques » | S |

---

## 3. Features POUSSÉES proposées

### 3.1 Arborescence sprint/phase (navigation de procédé)
| ID | Description | Donnée / route | Backend | Doctrine | Effort | Fenêtre |
|---|---|---|---|---|---|---|
| A1 | **Arbre de procédé** sprint→phase→commit→artefact (cœur, dédupe QW1/V8/F1) | `/api/sprint-history/{n}` `:31-48` | existant | OK (restitution mono) | M | **S80 (D MVP)** → profond S81 |
| A2 | **Ruban carrière read-only** dans le rail (pastilles status tous sprints, n'anime jamais, drill-down) | `/api/sprint-history/all :165-180` | existant | OK | S | **S80 (C/D min)** |
| A3 | **Breadcrumb de procédé** sprint·phase·branche·pouls cliquable → arbre | `/api/status` + `/api/gates` | existant + neuf (G) | OK | S | **S80 (C)** |
| A4 | **Deep-link** état d'altitude (`/sprint/79/phase/F`, `/commit/<sha>`) restaurant l'arbre | consomme A1 | existant | OK (accélérateur) | S | **S80 (C min)** |
| A5 | **Diff de commit relié à l'arbre** (clic nœud → diff-viewer, dédupe V2/F7) | `/api/sprint-history/diff/{sha}` | existant | OK | M (dépend H) | **S80 (D ouverture, H rendu)** |
| A6 | Navigateur de **findings P0-P3** cross-phase (open/resolved, lien commit) | `phases[].findings[] :845-867` | existant | OK | M | S81 |
| A7 | Navigation par **carry** (ouvert/fermé inter-sprints) | `carries_open/closed :103-109` + boucle `/all` | existant | OK | M | S81 |
| A8 | **Filtre par verdict de procédé** (DESIGN-CONFLICT, codex_gap>0…) | `preflight_verdict` + `preflight_bilan :135-149` | existant | OK (ne pas agréger en score) | S/M | S81 |
| A9 | **Annotation conformité commit** sur la timeline (dédupe QW7/V10/F3) | `/api/audit/{rev}` + `body_sections` | existant | OK (« N manques » ≠ ✓) | S/M | **S80 (D)** |
| A10 | **Replay attaché au nœud** (session STEER + run terminal `.cast`) | `/chat/{id}/log` + `/terminal/sessions/{name}` | existant (lien = heuristique) | OK | M | S81 |

### 3.2 Docs / research / knowledge
> Fait load-bearing : les routes servent des **hashes/refs, jamais le CONTENU** des docs. Lire un doc = **route neuve** (`GET /api/docs/file` / `GET /api/knowledge/{pack}/{file}`).

| ID | Description | Donnée / route | Backend | Doctrine | Effort | Fenêtre |
|---|---|---|---|---|---|---|
| D1 | **Fix gap daisyui** : ajouter `docs/factory/knowledge/daisyui/MANIFEST.json` à `AUTHORING_KNOWLEDGE_MANIFESTS :521` (animejs seul aujourd'hui) | const + test miroir | neuf-trivial | OK | S | **S80** |
| D2 | **Dérive de hash des sources consommées** (`◦ dérive — relu` si hash on-disk a bougé) | `file_hash() :498` + context-pack + snapshot store | existant | OK (fraîcheur ≠ verdict) | S | **S80** (réutilise fraîcheur VERIFY) |
| D3 | **Prompt inspector** (prompt réel kind×provider×depth, met en évidence `strip_cloud_references`) | `/api/prompt/{kind} :420` | existant | OK (sous « détails ») | S-M | **S80 (C)** |
| D4 | **Lien artefact→source** dans l'arbre (deliverable/files_changed → diff `/diff/{sha}`) | sprint-history + diff/{sha} | existant | OK | S | **S80 (→diff)** ; fichier S81 |
| D5 | **Pack reader consultatif** (contenu réel animejs/daisyui en lecture-seule pointillée) | `GET /api/knowledge/{pack}/{file}` allowlist | **neuf** | OK | M | S81 |
| D6 | **TOC vivante du process** (`docs/`, `.planning/research`, `CLAUDE.md` depuis le FS) | `GET /api/docs/tree` + `/docs/file` | **neuf** | OK | M | S81 |
| D7 | **Recherche grep-gate** (ripgrep allowlisté, `file:line + snippet`, **pas FTS5**, jamais réponse synthétisée) | `GET /api/search` | **neuf** | OK (correspondances, 0 ranking) | M-L | S81 |
| D8 | **Visionneuse contrat-pour-LLM** (`llms.txt`, `WIRING_SPEC`, `HOW_TO_WIRE`, `FACTORY_GATES`) | = D6 allowlist `docs/factory/` | neuf (= D6) | OK | S-M | S81 |
| D9 | **Intégrité knowledge-packs** (MANIFEST déclaré vs blake3 recalculé, état descriptif « cohérent/dérive » jamais PASS) | `GET /api/knowledge/integrity` (`template_lock.rs:45`) | **neuf** | OK (descriptif, surveiller la frontière PASS) | M | S81 |
| D10 | **« Ce que l'agent a réellement consulté »** (corréler tool-calls SSE ↔ advisory) | `/chat/{id}/stream\|log` + context-pack | existant (parse front) | OK (descriptif) | M | S81 |

### 3.3 Profondeur VERIFY
| ID | Description | Donnée / route | Backend | Doctrine | Effort | Fenêtre |
|---|---|---|---|---|---|---|
| V1 | **Diff-viewer bi-mode** (inline ⇄ side-by-side) + word-diff intra-ligne (spans front sur texte Rust) | `/api/git/diff :1316` | existant | OK | L | **S80 (H)** ; T1(5) |
| V2 | **Diff bi-usage WT ⇄ commit** (même composant, dédupe A5/F7) | `/diff/{sha} :1294` | existant | OK | S | **S80 (H)** |
| V3 | **Nav hunk clavier + minimap de densité** `+/−`, saut au hunk « marqué gate » | `/api/git/diff` | existant | OK | M | **S80 (H)** |
| V4 | **Panneau gates riche** — états distincts jamais aplatis + `run@<rev>` + `◦ obsolète` | `GET /api/gates` | **neuf (G)** | OK (clé du thème) | M | **S80 (H)**, backend G |
| V5 | **Pouls de gate dans la gouttière** (issue → `fichier:ligne`) | F + G | **neuf** — exige `GateResult.issues{path,line?,message}` | OK si UI ne déduit rien | M | S80 si shape G le permet, sinon carry |
| V6 | **Filtre change-set « par gate »** (fichiers concernés sans masquer l'état) | F + G | **neuf** (même dépendance V5) | OK | S | S80 ou carry S81 |
| V7 | **« Qu'est-ce qui a changé depuis le dernier vert »** (diff `<sha-phase>..WT`) | sprint-history + **diff range** | **neuf-léger** (range pas servi) | OK | L | S80 si range tient en G, sinon S81 |
| V8 | **Frise des verdicts de procédé** par phase (preflight/review/Codex/deltas/findings) (= A1 vue resserrée) | `/api/sprint-history :31-48` | existant | OK (exemplaire) | M | **S80 (D MVP)** |
| V9 | **Rejeu des passages de gate** (`.cast` sortie brute clippy/nextest/fmt) (dédupe QW4/F6) | `/terminal/sessions` + `/{name}` | existant | OK | M | **S80 (D)** |
| V10 | **Annotation conformité commit** (issues `audit-commit`, jamais score) (= A9) | `/api/audit/{rev} :393` | existant | OK | S | **S80 (D)** |
| V11 | **Actions-de-hunk-en-intentions** (« Transmettre la correction → session », « Signaler ») | F + `POST /chat/{id}/send :977` | existant | OK (verbes d'intention) | M | **S80 (H)** |
| V12 | **Provenance décomposée** (commit→archive_hash→signataire + couches + manques) | provenance/`render-seal` | **neuf** (lié Viewer) | OK sur le principe | L | **S81** (onglet désactivé en S80) |

### 3.4 Puissance STEER
> Faits load-bearing : sessions **en mémoire pure** (`HashMap` `:868`) → tout « reprendre après redémarrage » = persistance neuve ; **aucun token d'annulation** dans `handle_chat_stream :1071` → un vrai « Abandonner » = neuf ; `/stream` re-spawne idempotent → « Relancer » quasi-gratuit.

| ID | Description | Donnée / route | Backend | Doctrine | Effort | Fenêtre |
|---|---|---|---|---|---|---|
| S1 | **Bibliothèque d'intentions versionnée** (presets repo-visible `.planning/factory/intentions.json`, kind/provider sous le capot) | `POST /api/artifacts/draft :700` + lecture front | existant + neuf-léger (allowlist) | OK (intentions-pas-jargon, repo-visible) | M | **S80 (C)** socle, S81 enrichi |
| S2 | **Inspecteur de context-pack pré-vol** (le pack EXACT scellé, hashé, pointillé) (dédupe QW9) | `POST /api/context-pack :533` | existant | OK (consommée-jamais-autoritaire littéral) | S/M | **S80 (D)** |
| S3 | **Aperçu du prompt assemblé par provider** (= D3, bascule provider) | `/api/prompt/{kind}` | existant | OK (repli technique strict) | S | **S80 (C)** |
| S4 | **Multi-provider + diagnostic de joignabilité** (Claude clé / Ollama up+modèle / réseau pairs) | `/api/providers :608` + `/providers/health` | liste existante + neuf-léger | OK (diagnostic = signal VRAI) | M | S80 attribut (C), diagnostic S81 |
| S5 | **Relancer le tour** (re-stream idempotent sans re-saisie) | `/chat/{id}/stream :1085-1091` | existant (quasi-gratuit) | OK (manuel state-driven) | S | **S80 (C)** |
| S6 | **Interrompre le stream** — 2 niveaux honnêtes : (a) `EventSource.close()` « j'arrête d'écouter » ; (b) abandon réel kill du child | (a) client ; (b) `:1156-1174` sans cancel-token | (a) existant / (b) **neuf** (CancellationToken+kill) | OK (ne jamais mentir « arrêté ») | S(a)/L(b) | S80 (a) ; (b) future |
| S7 | **Tiroir Sessions + replay STEER** (context_pack + messages dont rejets du mur) (dédupe QW6) | `/chat/{id}/log :1179` | existant (in-process) ; **persistance disque = neuf** | OK (liste, pas board) | M | S80 liste (D) ; persistance S81 |
| S8 | **Journal de bord du nœud** (actions + rejets allowlist/PASS/traversal avec raison) (dédupe QW5/F5) | `/api/actions/log :689` | existant | OK (renforce le MUR) | S | **S80 (D)** |
| S9 | **Brouillon non-autoritaire = action unique du MUR** (refuse `## Verdict: PASS`, 0 Forcer) | `POST /api/artifacts/draft :700` (`:722-758`) | existant | OK (cœur doctrine) | M | **S80 (D)** |
| S10 | **Reprendre/transmettre depuis un pack scellé** (regénère handoff base/universal/handoff/runtime) | `/api/context-pack` (`handoff_prompt`) + draft | existant ; re-seed nouvelle session = neuf-léger | OK (handoff repo-visible) | M | S80 génération+export (C/D) ; re-seed S81 |

### 3.5 Signatures uniques SBFB
| ID | Description | Donnée / route | Backend | Doctrine | Effort | Fenêtre |
|---|---|---|---|---|---|---|
| U1 | **Process-as-artifact** : la chaîne preflight→code→review→Codex→commit inspectable (= A1, la signature que personne d'autre n'a) | `/api/sprint-history :9-48` | existant | OK | M | **S80 (D MVP)** → S81 profond |
| U2 | **Provenance-de-verdict** (invariant transversal : tout verdict cliquable → ouvre l'artefact `.planning/` source `:798-867`) | sprint-history + `preflight_bilan.phases[].file` | existant (ajouter `file` si absent = trivial) | OK (matérialise « 0 verdict calculé UI ») | S | **S80 (transversal D/H)** |
| U3 | **Carte de conformité du commit** (9 sections body + audit + lint, « N manques » jamais ✓) (= A9/V10) | `/api/audit/{rev}` + `/api/lint` + body_sections | existant | OK | S | **S80 (D)** |
| U4 | **Frontière de contrats RRV** (FRONTIER-tags + source-ref STALE-PHASE-K + 6 directives `'none'` `csp.rs:33` + arête prompt-kind→pack) | sortie `check-frontier-contracts.sh` | **neuf-léger** (parser→JSON) | OK (consultatif) | M | S81 (ou S80-G si embarqué) |
| U5 | **Le MUR comme avancée produit** + registre des refus (dédupe S8/F5) | `SENSITIVE_ACTIONS :37` + `/actions/log` | existant | OK (registre lecture-seule, jamais « réessayer en forçant ») | S-M | **S80 (inline C, registre D)** |
| U6 | **Rejeu de runs de vérification** asciinema (dédupe V9/QW4) | `/terminal/sessions` + `/{name}` | existant | OK | M | **S80 (D)** bootstrap, S81 poli |
| U7 | **Diff bi-usage WT + commit** (dédupe V2/A5) | `/git/diff` (F) + `/diff/{sha}` | existant | OK | S | **S80 (H)** |
| U8 | **Lignée blake3 des knowledge-packs** (advisory → MANIFEST, dédupe D5/QW9) | context-pack + `*/MANIFEST.json` | existant (lignée complète = D5/D9 neuf) | OK (consultatif) | M | **S80 (D advisory)** → S81 lignée |
| U9 | **Graphe de provenance** (commit→archive_hash→signataire→curateurs, couches + manques) | `provenance.json` SLSA L1 + FeedEntry Release/Vouched | **neuf** | OK (signature réelle, bloc manques assumé) | L | **S81** (couplé Viewer/Proof Card) |
| U10 | **Pont vers le Viewer scellé** (iframe prod `BLOB_SERVE_CSP` + `render-seal sha256`) | `compute_output_hash` + self-check S79 H | **neuf-léger** | OK (status self-check = test, jamais autorité publish) | M-L | **S81** (onglet désactivé en S80) |
| U11 | **Détection de drift** doc↔code / maquette↔code / template↔généré (la maquette montre 3 onglets, le code en câble 1 → drift détectable) | `check-frontier-contracts.sh` + `check-factory-docs.sh` | **neuf** | OK (consultatif) | L | future |

---

## 4. Filtre doctrine & scope (verdict condensé)

| Feature | Garde-fou clé | Fenêtre | Verdict |
|---|---|---|---|
| Arbre de procédé (A1/U1/V8) | restitution, 0 calcul UI | S80 D | **GARDER** |
| Provenance-de-verdict (U2) | « 0 verdict calculé UI » matérialisé | S80 | **GARDER** |
| Diff-viewer bespoke + bi-usage (V1/V2/U7) | diff = vérité Rust | S80 H | **GARDER** |
| Actions de hunk = intentions (V11) | jamais Approve/Merge/Commit | S80 H | **GARDER** |
| Panneau gates états distincts (V4) | jamais aplati vert/rouge | S80 H (G) | **GARDER** |
| Slot ÉTAT énuméré | ne dit jamais PASS | S80 H | **GARDER** (scan anti-PASS BLOQUANT) |
| MUR + registre refus + brouillon (S8/S9/U5) | barrière pleine largeur, 0 Forcer | S80 D | **GARDER** |
| Journal de bord (S8) / conformité commit (U3/A9) | issues from Rust, « N manques » ≠ ✓ | S80 D | **GARDER** |
| Inspecteur context-pack (S2) / advisory (U8) | consultatif pointillé | S80 D | **GARDER** |
| Rejeu .cast (U6/V9) / replay session (S7) | vérité brute / liste pas board | S80 D | **GARDER** |
| Relancer (S5) / Interrompre-écoute (S6a) | manuel state-driven | S80 C | **GARDER** |
| Fix daisyui (D1) / dérive hash (D2) / prompt inspector (D3) | gap réel / fraîcheur ≠ verdict | S80 | **GARDER** |
| Pouls gate gouttière (V5) / filtre par gate (V6) | dépend shape `GateResult.issues` | S80 ou carry | **ADAPTER** (dégrader si shape G insuffisant) |
| Diagnostic joignabilité (S4) | signal VRAI pas verdict | S80/S81 | **ADAPTER** (attribut S80, health S81) |
| Bibliothèque intentions (S1) / handoff (S10) | repo-visible | S80 socle | **ADAPTER** (socle S80, persistance S81) |
| Filtre par verdict (A8) | ne pas agréger en score | S81 | **ADAPTER** (jamais « santé ») |
| Intégrité packs (D9) | état descriptif jamais PASS | S81 | **ADAPTER** (surveiller frontière) |
| Pack reader / TOC / search / contrat-LLM (D5-D8) | lecture-seule, correspondances pas réponse | S81 | **ADAPTER** (route lecture neuve) |
| Findings/carry/replay-nœud (A6/A7/A10) | restitution | S81 | **GARDER** (S81) |
| Frontière contrats (U4) / drift (U11) | consultatif | S81/future | **GARDER** (S81+) |
| Abandon réel du stream (S6b) | ne pas mentir « arrêté » | future | **GARDER** (backend neuf) |
| Provenance graphe (U9/V12) / Viewer scellé (U10) | signature réelle, status ≠ autorité publish | S81 | **GARDER** (onglet désactivé « à venir » en S80) |
| Aperçu scellé / Proof Card (Surface 4) | rouvrirait P1 app-authoring in-vivo | S81 | **DIFFÉRÉ** |
| Score de santé / trust-score / jauge originalité | UI calculerait un verdict | — | **REJET** (cf. §6) |
| Approve/Merge/Forcer/Override | verbe d'écriture repo / contourne le MUR | — | **REJET** (cf. §6) |
| Q&A/RAG synthétisé | réponse autoritaire vs restitution | — | **REJET** (cf. §6) |
| Multi-session board / CM6 / ⌘K-cadre | YAGNI solo / coupés kickoff | — | **REJET/COUPÉ** (cf. §6) |
| Auto-bascule STEER→VERIFY | arrachée au stream, casse déterminisme T1 | — | **INTERDITE** |

---

## 5. Recommandations

### 5.1 À intégrer DANS S80 sans élargir le cœur bi-focal
*(toutes sur routes existantes ou le seul backend-neuf prévu `GET /api/gates` ; quasi-gratuit haute valeur)*
1. **A1/U1 Arbre de procédé** (Phase D MVP) — la signature la plus différenciante, données déjà calculées.
2. **U2 Provenance-de-verdict** (transversal) — rend l'invariant doctrinal tangible et gratuit.
3. **V4 Panneau gates états distincts** (Phase H, backend G) — clé de VERIFY-plein.
4. **V1+V2/U7 Diff-viewer bespoke bi-usage** (Phase H) — le vrai investissement, bonus `/diff/{sha}` gratuit.
5. **S8/U5 Journal du nœud + registre du MUR** (Phase D) — rend le MUR visible, renforce la doctrine.
6. **S2 Inspecteur context-pack** + **D2 dérive de hash** + **D3 prompt inspector** (C/D) — transparence du steering, réutilise la mécanique de fraîcheur VERIFY.
7. **D1 Fix gap daisyui** (S, neuf-trivial) — corrige une asymétrie pack/process réelle.
8. **U3/A9/V10 Conformité commit** + **U6/V9 rejeu .cast** + **S5 Relancer** + **S7 liste Sessions** (D) — quick wins purs.

### 5.2 Top S81 (fondation readonly Viewer re-planifiée)
- **U9 Graphe de provenance** + **U10 Viewer scellé** + **V12 Preuve décomposée** (cœur Proof Card).
- **D5/D6/D7/D8 routes de lecture de contenu** (`/api/docs/file`, `/api/knowledge/{pack}/{file}`, search grep-gate) — pierre angulaire de la bibliothèque consultative.
- **A6 findings cross-phase** + **A7 carries inter-sprints** + **A10 replay-nœud** + **S7 persistance sessions**.
- **U4 Frontière de contrats RRV** + **D9 intégrité packs**.

### 5.3 Top roadmap (future)
- **S6b abandon réel du stream** (CancellationToken + kill child) — honnêteté d'état.
- **U11 détection de drift** (doc↔code, maquette↔code) — garde le contrat-pour-LLM honnête.
- **D7 index local** si le grep-gate ne tient plus le volume (jamais FTS5/Tantivy côté Operator avant nécessité prouvée).

### 5.4 Tentant mais à REJETER pour rester fidèle (solo + doctrine)
- Tout **dashboard KPI / courbes de delta tests** stylé « metrics » → toléré seulement en micro-restitution mono dans l'arbre, jamais en page dédiée.
- Toute **jauge/score** (santé sprint, trust-score provenance, originalité-vs-corpus) → l'UI calculerait un verdict.
- **Auto-bascule STEER→VERIFY** + bandeau « ✓ réussie » → interdite (déterminisme T1 + anti-PASS).
- **Multi-session board / Mission-Control** → réduit à la liste Sessions.

---

## 6. Idées REJETÉES (doctrine / scope) — pour mémoire

| Idée | Garde-fou violé | Raison |
|---|---|---|
| **Score/jauge de « santé du sprint »** | « l'UI ne calcule AUCUN verdict » | Une jauge synthétique EST un verdict calculé UI. Garder les dimensions séparées (findings/carries/verdicts), jamais aplaties. |
| **Trust-score / badge confiance provenance** (« provenance 87% ») | consommée-jamais-autoritaire | Verdict scalaire fabriqué par le front. Garder la décomposition couches + bloc manques. |
| **Jauge d'originalité nouveauté-vs-corpus** | jamais de ranking global (curation par listes signées) | Ranking global proscrit. Recevable seulement reformulé en restitution non-ordonnée, hors S80. |
| **Ranking/classement qualité des phases/sprints** | jugement global calculé UI | Le filtre par verdict factuel (A8) est l'alternative conforme. |
| **Bouton Approuver / Merger / « hunk validé »** | actions de hunk = intentions, jamais Approve/Merge/Commit | Verbe d'écriture repo. Remplacé par V11 (intentions routées). |
| **« Exécuter quand même » / « Forcer le spawn » sur le MUR** | MUR = barrière, 0 Forcer | Contournerait `requires_gate` 0-spawn backend. Seule issue : S9 (Préparer le pack) + Retour. |
| **« Demander à la base de connaissance » (Q&A/RAG synthétisé)** | consommée-jamais-autoritaire | L'UI produirait une réponse autoritaire. Garder D7 (correspondances), router la question à STEER. |
| **Score de pertinence / ranking des résultats de recherche** | jamais de ranking global | Tri neutre (ordre fichier/ligne) seulement. |
| **Score de couverture de gate auto-calculé** | 0-PASS / 0 verdict UI | Si agrégat voulu, il vient d'un champ Rust de `/api/gates`, restitué tel quel. |
| **Auto-bascule STEER→VERIFY arrachée au stream** | bascule manuelle state-driven (déterminisme T1) | INTERDITE. Indicateur neutre + badge `1 diff` + bascule manuelle. |
| **Lancer N intentions en parallèle (board multi-agents)** | YAGNI solo (single-PTY séquentiel) | Coupé kickoff. La liste Sessions suffit. |
| **Éditeur CM6 riche** | YAGNI solo | Le terminal xterm reste le cœur d'édition par l'agent. |
| **Palette ⌘K comme cadre** | intentions-pas-jargon | Accélérateur seulement, jamais le cadre principal. |
| **Timeline-canvas de procédé** | moteur de graphe + drift canvas↔repo | Différé ; ruban read-only altitude-0 (A2) au MVP. |
| **i18next + router complexe** | mono-locale FR / routing = état d'altitude | Différé ; deep-link minimal (A4). |

---

*Distinction transverse rappelée : afficher un `review_verdict:"PASS"`/`preflight_verdict:"EXECUTE"` **historique** est une RESTITUTION d'un fait gravé au commit par le Rust (rendu mono factuel cité de `sprint_history.rs`), visuellement distinct du slot ÉTAT **live** de VERIFY qui, lui, ne dit jamais PASS. Le seul backend-neuf du cœur S80 est `GET /api/gates` (Phase G) ; tout le reste se câble sur l'existant.*