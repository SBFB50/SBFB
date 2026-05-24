# Sprint 70 - Design Review Board (G1)

**Date** : 2026-05-24
**Sprint** : 70 - Option ambitieuse : Process Portable Complete + Factory Viewer/Operator Rust + Gate 1 dogfood
**Reviewer** : self-review profond (auto-challenge systematique)

---

## Scoring

| D# | Titre | Source recente | Alternative | [DETER] Crypto | [DETER] Rust | Code verifie | Verdict |
|---|---|---|---|---|---|---|---|
| D1 | AGENT_SYSTEM.md carte derivee 7 sections | ok (AGENTS.md ecosysteme, arxiv 2605.11032, PROCESS.md local) | ok (fusion PROCESS, prompt-only, status quo) | N/A | N/A | ok (`PROCESS.md`, `AGENTS.md`, `TOOLING.md`) | ok |
| D2 | Prompt portability full + handoff | ok (handoff protocols, OpenAI Agents SDK, GCC arxiv 2508.00031) | ok (handoff seul, universal etendu, pas de migration) | N/A | N/A | ok (`PROMPT_KINDS`, `prompts/agent/*`) | ok |
| D3 | Observabilite process Rust + Operator serve JSON | ok (agent/process observability, local JSON control planes) | ok (CLI only, sqlite state, prototype non retenu) | N/A | ok (nouvelle surface dans `crates/sbfb-factory`) | ok (`sbfb-factory`, roadmap v4) | ok |
| D4 | Hooks dynamiques + provider config + dogfood | warning (prior art externe limite pour hooks Claude dynamiques) | ok (pre-receive, CI-only, supprimer hooks Claude) | N/A | N/A | ok (`process-task-gate.sh`, `process-supervisor-stop.sh`, `auditor-gate`) | warning |
| D5 | Factory Viewer + Operator boundary | ok avec contrainte action-gated | ok (Viewer plus tard, Operator CLI only, route web shell S71+) | N/A | ok (Operator Rust local, Viewer app SBFB statique) | ok (`sbfb-factory`, `web/public/sbfb-bridge.js`, roadmap v4) | ok |

**Resume** : D1 ok, D2 ok, D3 ok, D4 warning, D5 ok sous contrainte action-gated.
Rigor signal G4 satisfait : 1 warning documente, 0 P0/P1 design.

---

## Findings

### D4 warning - Source recente limitee pour hooks dynamiques

Les sources recentes couvrent les hooks pre-commit, leurs bypasses et la
defense en couches, mais pas un pattern public specifique "hook Claude avec
sprint hardcode -> detection dynamique". Le gap est prouve par le code local :
`process-task-gate.sh` et `process-supervisor-stop.sh` hardcodent S67.

Decision : accepter. Le fix est un pattern interne simple (detecter le sprint
courant depuis `.planning/active/`, construire les artefacts attendus, tester
sprint pair/impair). Risque suivi par S70 R2.

### D5 contrainte PO - Viewer protocole oui, Operator Rust local oui

Le plan initial excluait une "Factory process UI". L'option ambitieuse
accepte deux surfaces distinctes : Factory Viewer comme app SBFB
sandboxee de consultation/preuve, et Factory Operator comme outil local
privilegie du noeud. L'Operator est Rust (`crates/sbfb-factory`), pas
backend Python. Il peut declencher des actions allowlistees, preparer
des brouillons d'artefacts et ouvrir une discussion agent comme le chat
actuel. Le Viewer ne lit pas le workspace prive et n'appelle pas
l'API Operator. Les operations sensibles (shell/commit/push/verdict
final) peuvent etre pilotees par une vraie session agent si le provider
et l'environnement l'autorisent, mais elles ne sont valides que via
gates, preuves repo-visibles et journalisation.

La bonne architecture n'est pas deux fronts dupliques : S70 doit creer
un socle `tools/factory-ui/src/readonly` partage par Viewer et Operator
(modeles, labels FR, previews, Proof Cards, timeline, statuts). Les
extensions `tools/factory-ui/src/operator` restent reservees a l'Operator et
ne doivent jamais entrer dans le bundle/source Viewer.

Decision : accepter si les criteres suivants sont dans le plan et la verification :
- endpoints `sbfb-factory operator serve` action-gated ;
- prompt UX Claude Design ecrit dans
  `.planning/active/sprint70_factory_ux_design_prompt.md` avant code
  front ;
- lien/export Claude Design reference dans
  `.planning/active/sprint70_factory_ux_design_handoff.md` avant
  integration repo ;
- Factory Viewer app SBFB sandbox-compatible ;
- socle UI lecture partage entre Viewer et Operator, avec import
  `factory-ui/operator` interdit cote Viewer ;
- Factory Operator compile/lint/tsc/build ;
- smoke `operator serve --once-smoke` ;
- preview diff + confirmation avant tout draft repo/docs sur allowlist ;
- journal JSONL des actions Operator ;
- libelles utilisateur en langage produit ("Preparer la phase",
  "Verifier avant validation", "Transmettre a un autre agent") ;
- commandes `sbfb-factory` et termes `kind/provider/preflight` caches dans
  un panneau details techniques, jamais comme CTA principal ;
- flux "Nouveau contexte / Transmettre a un autre agent" via
  ContextPackBuilder : base/universal/context/handoff/prompt
  specialise, sans chat history authoritative ;
- Agent Chat preserve le mode actuel : discussion libre, agent
  autonome, mise a jour du repo possible via le meme contrat
  repo-visible que le flux chat ;
- Agent Selector mappe "Qui code ?" / "Qui verifie ?" vers
  driver/reviewer/auditor + provider/depth/kind, sans modifier
  l'autorite des gates ;
- ActionCenter et DraftArtifactDialog rendent visibles resultat,
  action id, preview diff et log avant toute ecriture ;
- interdiction de creer/promouvoir un `## Verdict: PASS` final par
  simple UI ; le PASS doit venir du flow review/gate ;
- contrat RRV/Factory rappelant que le Viewer expose, l'Operator pilote
  localement, et l'autorite reste planning + commits + gates.

---

## Checklist [DETER]

### Crypto/spec
- [x] Pas de decision crypto/spec dans ce sprint.
- [x] Le Viewer/Operator ne modifie aucun format protocolaire ni signature.

### Rust-first
- [x] Nouvelle surface privilegiee en Rust : `crates/sbfb-factory`.
- [x] Exemption justifiee : docs Markdown et UI React/TS ; pas de nouveau backend Python.

### Product boundary
- [x] Factory Viewer = app SBFB de consultation/preuve.
- [x] Factory Operator = outil local Rust action-gated.
- [x] UX utilisateur = intentions humaines, pas commandes `sbfb-factory`.
- [x] Route shell `web/ /factory`, RRV total, SearchManifest, @dev index restent hors scope S70.
- [x] Factory/RRV consomment les preuves ; ils ne deviennent pas autorite de verification.

## Verdict: PASS
