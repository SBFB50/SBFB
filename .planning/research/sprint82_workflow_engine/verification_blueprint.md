# Sprint « moteur de workflows IA » — vérification ultracode + blueprint (staging S82)

> Statut : note de recherche / STAGING hors sprint (2026-07-02). Aucun code, wire, ni gate
> engagé par ce document. Produit par le Workflow ultracode `wf_23e03df4-eff`
> (11 agents, ~1,02M tokens, 149 outils : 7 vérifications parallèles ancrées code+web,
> 2 architectes concurrents, 1 attaque adversariale, 1 synthèse). Résultat brut complet
> (ancres fichier:ligne, schémas, plans A/B intégraux) : `raw_workflow_verification_wf_23e03df4.json`.
> Contexte amont : réflexion Flowise/Factory (session 2026-07-02) + note packs
> `prompt_agent_packs_protocol_research.md` (2026-06-28) + étude idéation
> `idea_development_science_ultradeep_2026-06-30.md`.
> Activation : préparer le kickoff dans CE dossier, `git mv` vers `.planning/active/`
> quand le slot s'ouvre (précédent exact : `sprint81_iroh_upgrade/`).

## 0. Question du PO

« Que peut-on avoir concrètement si on met TOUT dans un sprint (phases illimitées) :
Canvas éditeur visuel + Viewer de graphe read-only dans l'Operator + les 3 autres
briques (format de graphe workflow/ du pack, interpréteur Rust+JS embarquable,
base d'exécution câblée) ? » — plus l'import Flowise flow-JSON.

## 1. Verdict final (synthèse post-attaques)

Oui, c'est UN sprint au sens strict du process (README §4 : le nombre de phases est
une sortie, jamais un plafond), mais ce serait le plus long de l'histoire du projet —
**12 phases séquentielles** à cérémonie complète (~1,5-2× le record S77), bâties sur le
squelette **risque-d'abord** (Plan B corrigé des attaques), avec un **point de coupe
propre après le Viewer (Phase H)** où 4,5 briques sur 5 sont livrées et démontrables.
Il ne peut PAS s'ouvrir maintenant : réalistement c'est **S82** (cf. §6 prérequis).
La seule chose qui n'y tient pas, même en phases illimitées : le **canvas éditeur
COMPLET** (re-câblage libre d'arêtes + création d'arête au clavier WCAG 2.1.1) —
l'éditeur in-sprint est un **MVP dégradé assumé** (propriétés + insertion linéaire) ;
tout plan qui prétend le contraire ment sur la volumétrie.

## 2. Table des claims vérifiées

| Claim (réflexion 2026-07-02) | Verdict | Preuve (résumé) |
|---|---|---|
| La base d'exécution existe déjà (effort ~0) | **NUANCÉ** | Vraie pour la primitive MONO-NŒUD (`ExecutionTarget` 3 bras HEAD + 4e bras ChatGpt dans l'arbre sale ; `StreamChunk` llm_bridge.rs:42-59 ; gate SENSITIVE_ACTIONS avant dispatch). MAIS 9 gaps réels pour un GRAPHE : 0 persistance (tout en `Arc<Mutex<HashMap>>`, 0 DB dans le crate), 0 pause/reprise HITL, 0 multiplexage node-scopé, 0 annulation, 0 historique de runs, 0 scheduler DAG, 0 schema-out, sémantique gate inter-nœuds à concevoir, bridge hors-crate. |
| Format de graphe = des jours | **CONFIRMÉ** | Preuve constructive produite pendant la vérif : JSON Schema des 8 types + le workflow d'idéation 10-nœuds exprimé intégralement dedans. Layout pack réserve déjà `workflow/`. |
| Interpréteur noyau Rust+JS = ~1 sprint | **NUANCÉ** | Faisable DANS le sprint mais seulement risque-d'abord : la reprise HITL durable (kill→restart→`waiting_human` intact→reprise sans ré-exécuter) est l'hypothèse LA plus structurante, 0 précédent code → spike dédié AVANT de geler le format. |
| Viewer read-only = ~1 phase | **NUANCÉ** | Oui : 5e inspecteur du rail (~15 lignes de glue SurfaceHost). Mais phase PLEINE à 3 conditions : spike xyflow séparé avant ; SSE node-scopé livré par le moteur ; vocabulaire d'états non-verdict vérifié sur 51 locales. |
| Canvas éditeur = 1-2 sprints, compressible | **NUANCÉ** | L'éditeur COMPLET (drag-connect + validation connexions + arête AU CLAVIER que xyflow ne fournit pas + liste éditable parallèle) = 2-3 phases à lui seul → in-sprint = MVP dégradé (propriétés + insertion linéaire), le complet = S+1. |
| xyflow MIT + React 19 + CSP self | **CONFIRMÉ** | MIT vérifié 2× (package.json 12.11.1 + LICENSE webkid GmbH), peerDeps react>=17, CSP propre en build prod SANS unsafe-inline. Taille : Bundlephobia ~183 KB (budget size-limit à initialiser à MESURE CONSTATÉE +10 %, jamais à l'estimation). |
| Canvas Flowise = React Flow | **CONFIRMÉ** | flowise-ui 3.1.3 : `"reactflow": "^11.5.6"` + imports dans `views/canvas/index.jsx` (vérifié raw GitHub). Eux = v11 legacy, nous = @xyflow/react 12. |
| flow-JSON transpilable (subset sûr) | **CONFIRMÉ** | 2 dialectes vérifiés sur la marketplace (chatflow + agentflowv2 : wrapper ReactFlow, vrai type dans `data.name`, template = donnée pure). ~90 % de la topologie du workflow 10-nœuds importable. Champs exécutables (func/customCode/mcpServerConfig/http*) = REFUS DUR du flow entier (CVE-2026-40933 se déclenche au RENDU → zéro rendu avant validation). |
| task_submit suffit en MVP in-app | **CONFIRMÉ** | Prouvé 2× dans le code vivant : `submitTask→getTaskResult` 404→pending (sbfb-bridge.js:169-183, S76-H) + l'Operator lui-même poll le réseau (bras Network PO-14). 0 méthode bridge nouvelle en v1. |
| ~8 types de nœuds suffisent | **CONFIRMÉ** | Preuve constructive sur 3 workflows cibles : idéation 10-nœuds (4/8 types suffisent), Factory Intake (5), preflight fan-out 5 scans (AND-join). Vocabulaire figé proposé : input, llm, branch, loop(max_iterations), human-gate, tool, schema-gate, output. |

## 3. Blueprint fusionné (12 phases, squelette risque-d'abord)

- **0** — Audit gate S81 + vérifs d'ouverture (echo-target ExecutionTarget : 0 occurrence aujourd'hui → sinon 1er livrable moteur ; baseline figée SUR LE TIP à l'activation).
- **A** — SPIKE xyflow sous CSP 'self' + budget RAW mesuré (tueur front). Chunk lazy `vendor-xyflow`, GO/NO-GO consigné. Plan B : SVG maison + dagre (MIT, ~38 KB).
- **B** — SPIKE pause/reprise HITL durable sur run-store persistant (tueur moteur). rusqlite via spawn_blocking (leçon P2-SYNC-FS-ASYNC S73), tables runs/node_states/history append-only, kill-restart-resume prouvé.
- **C** — SPIKE runner in-app via task_submit polling + storage 1-clé (tueur sandbox). État `workflow/{run_id}` debouncé (GCRA 10 writes/min), human-gate in-app, cap 1 MB.
- **D** — FORMAT SBFBWorkflow.v1 gelé : crate contrat PUR `sbfb-workflow-core` (serde+schemars, PAS de tokio, le daemon n'en dépend JAMAIS), 8 types deny_unknown_fields, schémas FRONTIER drift-gatés, JCS/BLAKE3, layout/positions HORS-HASH (sidecar).
- **E** — MOTEUR Rust dans sbfb-factory : walk DAG (AND-join, arêtes skipped, loop borné, budget global), nœud llm = enveloppe ExecutionTarget::run (cancel-token, prompt_ref+vars, output_schema_ref, réparation bornée N retries), GATE par CLASSE sur nœuds tool (D9) + statut `gated`, SSE node-scopé {run_id,node_id,chunk}.
- **F** — INTERPRÉTEUR JS embarquable : AST-walker déclaratif SANS eval, équivalence golden-trace déterministe Rust↔JS en CI, app `examples/workflow-idea-hub`, tool→refused_by_host par défaut.
- **G** — IMPORT Flowise donnée-hostile : parseur offline pur dans sbfb-workflow-core, allowlist deny-by-default `data.name` (2 dialectes), refus dur champs exécutables, strip credentials jamais loggés, rapport mappé/approximé/refusé, corpus fixtures vendoré.
- **H** — VIEWER read-only : 5e inspecteur SurfaceHost, états PAR NŒUD strictement RESTITUÉS (« terminé/atteint/en attente/écarté/refusé par l'hôte » — jamais un mot des word-lists verdict 51 locales), live SSE + catch-up re-GET, a11y liste parallèle. **← POINT DE COUPE : 4,5 briques sur 5 livrées.**
- **I** — CANVAS ÉDITEUR MVP DÉGRADÉ ASSUMÉ : host atelier distinct (option b — le 3e mode focal COMPOSE amenderait la décision gelée S80-D6, seulement sur ratification PO écrite), inspecteur formulaire typé par nodeType (accessible par construction), insertion/suppression linéaire, re-lier un nœud refusé d'import, save→hash stable.
- **J** — DOCS-CONTRACT closure §6.12 : `docs/protocol/SBFB_WORKFLOW_V1.md` (R1), GUIDE Diataxis FR + llms.txt BLOQUANTS, THREAT_MODEL corrigé (§5.2 dit encore « 3 méthodes whitelist » pour 15-16 réelles) + surfaces runner in-app et import hostile.
- **K** — TESTABILITÉ T1/T2 + wrap-up : T1 Playwright hermétique BLOQUANT (viewer+éditeur+boot CSP ; in-app en vitest/jsdom), T2 = artefact JSON PASS sur fixture 3-5 nœuds schéma tolérant solo Ollama (le flagship 10 nœuds réel = dogfood tracé PROVISIONAL + carry P1 assumé), carries routés.

## 4. Décisions Day-0 à trancher au kickoff (D1..D11)

1. **D1** xyflow @xyflow/react 12.11.1 : OUI (MIT 2×, React 19 OK) avec rationale écrit vs culture 0-dep ; budget = mesure constatée +10 %.
2. **D2** Bridge : ZÉRO méthode nouvelle en v1 (task_submit+storage_* prouvés suffisants) ; streaming/HITL-chrome/session différés avec design note.
3. **D3** Découpage crates : `sbfb-workflow-core` = format+lint+import PURS ; moteur+run-store+routes dans sbfb-factory ; le daemon ne dépend JAMAIS du crate (v4-D2).
4. **D4** JSON-JCS seul pour l'artefact signé/hashé ; YAML rejeté.
5. **D5** Import Flowise in-sprint = parseur offline pur ; sortie = pack local NON-SIGNÉ tant que l'utilisateur n'a pas relu/signé.
6. **D6** ⚠️ SUPERSEDE doctrinal à ratifier PAR LE PO : le non-objectif « pas d'agent qui tourne dans l'iframe app » (note packs l.619) vs l'interpréteur JS in-app. Formulation : interpréteur mince DÉTERMINISTE ≠ agent autonome, inférence UNIQUEMENT via bridge, tools UNIQUEMENT allowlistés. Si refus PO → runner in-app dégradé/coupé.
7. **D7** 8 types figés + statut runState `gated` + équivalence Rust/JS par golden traces DÉTERMINISTES uniquement.
8. **D8** T1/T2 NOMMÉS au kickoff.
9. **D9** [P0 design] GATE des workflows par CLASSE sur les nœuds tool (tools.allowlist.json) + confirmation humaine à l'exécution ; le scan substring SENSITIVE_ACTIONS reste réservé au texte libre HUMAIN (jamais sur prompts assemblés — faux positifs quasi-certains sur « pass »/« push »/« shell »).
10. **D10** [P1 format] Layout/positions HORS-HASH — trancher AVANT la phase format.
11. **D11** [P1 doctrine] Décision HITL Operator = ACTION gated/journalisée ; viewer read-only strict ; placement éditeur = host atelier par défaut.

## 5. Ce qu'on a concrètement à la fin

**GARANTI** : le FORMAT signé (8 types nés avec leurs schémas drift-gatés, JCS+BLAKE3+Ed25519, lint 6 règles dont decision_writes réservé human-gate = porte NR8/NR9) · la DURABILITÉ (un run SURVIT à la mort de l'Operator, prouvé kill-restart-resume) · la SÛRETÉ D'IMPORT (flow-JSON jamais évalué ni rendu, Custom Tools/MCP/http = rejet total diagnostiqué, CVE-2026-40933 hors-sujet par construction) · le DOUBLE INTERPRÉTEUR (même pack dans l'Operator et dans une app sandboxée, équivalence golden CI, 0 méthode bridge nouvelle, 0 changement CSP) · le VIEWER (live nœud-par-nœud, 8 états nommés jamais un verdict, 51 locales, clavier+SR) · les PREUVES MACHINE (T1 verts CI + t2_acceptance.json PASS + parité golden + doc protocole utilisable par un LLM frais).

**PROBABLE** : l'éditeur MVP dégradé (glisse S83 sans casser le sprint si le calendrier craque après H) · le run live 10 nœuds sur Ollama (sinon dogfood tracé PROVISIONAL) · la démo import sur flow marketplace réel.

**PROVISIONAL (avec critère de preuve)** : exécution multi-provider réelle du graphe · runner in-app avec VRAI worker · équivalence Rust↔JS en live · import hors corpus officiel · EFFICACITÉ GÉNÉRATIVE du pipeline d'idéation (même statut que S79 : Not-evidenced sans session dogfood tracée) · gate inter-nœuds face à données amont adversariales (prompt-injection inter-nœuds → catalogue de menaces).

## 6. Prérequis AVANT d'ouvrir (bloquants, hors-sprint)

1. **Fermer l'arc parallèle par SA session** : 88 fichiers sales, Codex groupé EN PAUSE, extraction i18n + switcher inachevés — dont `provider_router.rs` (+51, 4e bras ChatGpt) et `llm_bridge.rs` (+359) qui SONT la brique 1, et ~+10 800 lignes de .po.
2. **Livrer S80 Phases I/J** (dont l'echo-target ExecutionTarget déterministe dont ce sprint dépend — 0 occurrence dans le code aujourd'hui). Dual-platform Docker avant push (tip non pushé).
3. **Arbitrage slot** : S81 = iroh 0.98→1.0 (staging présent, deadline DURE relais N0 EOL 2026-09-30, sprint dédié SEUL par décision GuardianDB) → ce sprint = **S82**, Phase 0 = audit gate S81.
4. **Kickoff en staging dès maintenant** (ce dossier) ; ne graver AUCUN chiffre de baseline (les figer à l'activation sur le tip réel).
5. **Dédup carries S80** : la « fondation partagée Viewer/Operator » (S81 planifiée) glisse S83+ ; WorkflowSurface assume le SurfaceHost actuel.

## 7. Dette S+1 explicite (ne tient pas, même phases illimitées)

Canvas éditeur COMPLET (drag-connect + arête au clavier WCAG 2.1.1 + validation connexions : 2-3 phases seul) · streaming token-par-token vers les apps (chantier PROTOCOLE : route SSE daemon + deltas contraires à PO-14 + doctrine guardrail S73 sur texte partiel) · rejeu SSE Last-Event-ID (catch-up re-GET suffit) · annulation FORTE du bras Network (pas de cancel protocole réseau) · publication réseau des packs (raw-op 0-bump, précédent SeedAnnounced) + réplication P2P de l'état des runs · format v2 (rag, sub-workflow, map dynamique, attente événement réseau) · fuzzing différentiel Rust/JS + cargo-fuzz du parseur · mini-viewer graphique in-app (v1 = UI d'état textuelle, pas 185 KB de xyflow par archive) · template Factory workflow-runner (machinerie S79 complète) · anti-drift des 9 copies de sbfb-bridge.js · traductions effectives ~70-100 clés × 51 locales · AAA/COGA complet du canvas · fondation partagée Viewer/Operator (S83+) · éditeur de prompts du pack.

## 8. Attaques P0 retenues (ce qui a corrigé les plans)

1. Les deux plans minimisaient le séquençage externe (« prérequis de Phase 0 ») alors que c'est des semaines hors-sprint (arbre sale + S80 I/J + S81 iroh).
2. Plan A = 2 sprints déguisés en 1 (sa phase canvas empilait 3e mode focal + éditeur complet + i18n/a11y ; +200-280 nextest ≈ 2× le record S77).
3. Étendre le scan substring SENSITIVE_ACTIONS aux prompts assemblés des nœuds = tempête de faux positifs (« pass »/« push »/« shell » dans du texte technique) → gate par CLASSE de nœud (D9).

Plan survivant : **B (risque-d'abord) en fusion corrigée** — spikes tueurs A/B/C d'abord, format APRÈS les spikes, gate par classe, layout hors-hash, viewer strictement read-only, équivalence sur fixtures déterministes seulement, T2 recalibré.
