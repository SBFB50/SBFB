# Sprint 71 — Audit Plan (pour Phase 0 S70)

**Ecrit** : 2026-05-25 (Phase G Sprint 70).
**Sprint audite** : Sprint 70 (Process Portable Complete + Gate 1 dogfood).
**Tip attendu** : commit Phase G docs(sprint70).
**Source intake** : roadmap v4 §Arc 3 + audit S70 findings.

---

## S71 Objective — Reseau Verifiable + Industrialisation (Arc 3)

S71 ouvre l'Arc 3. Le contenu exact (SearchManifest opt-in vs RRV
Core vs dette pair) sera decide par les findings de cet audit et
le kickoff S71. La roadmap v4 positionne S71 comme "SearchManifest
opt-in ou RRV Core selon audit S70".

S70 a livre le process portable complet (AGENT_SYSTEM.md, prompts
executables, sbfb-factory process CLI, Factory Viewer/Operator,
hooks dynamiques, provider config, contrat RRV/Factory). S71
consomme ce process pour construire la couche reseau verifiable.

---

## Audit Tracks A-I

### Track A — AGENT_SYSTEM.md canon

Verifier que AGENT_SYSTEM.md (Phase A) :
- Couvre les 9 roles du registre sans dupliquer PROCESS.md
- Le provider mapping (Claude, Codex, GPT, local, humain) est coherent
- Le gate contract (§5) formalise les verdicts attendus
- Le prompt registry (§6) liste les 8 kinds executables
- Le Truth Stack (§1) est respecte dans la pratique S70

### Track B — Prompt portability

Verifier que les 8 prompt kinds (Phase C) :
- Sont tous executables via `sbfb-factory process prompt --kind {kind}`
- Le handoff genere suffit a reprendre le travail sans chat history
- Le provider flag (--provider local/codex/gpt/human) filtre correctement
- Les tests Rust couvrent chaque kind + cas d'erreur

### Track C — Observabilite process Rust

Verifier que sbfb-factory process (Phase D) :
- status-sprint, lint-planning, audit-commit fonctionnent en JSON
- operator serve expose les endpoints attendus
- Les tests Rust couvrent les commandes et les guards
- La sortie JSON est coherente avec les artefacts planning

### Track D — Factory Viewer / Operator

Verifier que (Phase E) :
- Factory Viewer est une app SBFB sandboxee (pas d'import Operator)
- Factory Operator est un outil local Rust (pas une app SBFB)
- Le socle factory-ui/readonly est partage en lecture seule
- Les actions Operator sont gated (confirmation/guard)
- Build + lint + tsc passent pour les deux surfaces

### Track E — Hooks et bypasses

Verifier que (Phase F) :
- Les hooks sont dynamiques (pas de sprint hardcode)
- Les bypasses chore/verdict espace sont fermes
- Le verdict exact `## Verdict: PASS` est enforce
- PROVIDER_CONFIG.md definit la matrice driver/verificateur
- Les agents Claude sont des wrappers legers

### Track F — RRV/Factory contract

Verifier que (Phase G) :
- Les 5 modes @ sont documentes comme alias de roles
- Le principe d'autorite est clair (process > RRV > Factory)
- Factory Viewer et Operator sont correctement separes
- Le sequencing post-S70 est coherent avec la roadmap v4
- Babel est defini comme app, pas comme process

### Track G — Delta tests

Verifier les compteurs :
- Entree S70 : 1433 Rust, 279 Vitest
- Sortie S70 attendue : ~1486 Rust, 279 Vitest
- Delta reel vs estime (plan: +45 Rust)
- Aucune regression introduite

### Track H — Carries et dette

Verifier le routage des carries S70 → S71 :
- P2-I-3 3/3 MANDATORY CLOSED (Phase B)
- P2-G-1 CLOSED 8 sprints non-repro (Phase B)
- P2-C-1, P2-C-2, P2-I-1 documentes PATTERNS/README (Phase B)
- 7 carries reconduits S71 (rand, iroh, iframe, Radicle,
  redundancy, quorum, prompt coupling)
- P2-F-3 prompt file coupling 1/3 (nouveau Phase F)

### Track I — Meta-process

Verifier que S70 respecte le meta-process :
- 7/7 phases G8 preflight (cinquantieme sprint systematique)
- 7/7 phases review + Codex
- Commit body 9 sections pour chaque phase
- Scope cuts 14/14 respectes
- Sprint pair phase dette Phase B (§6.2.1 Regle 1)
- Design review G1 respecte (sprint70_design_review.md)

---

## RRV/Factory Consumer Contract (post-S70)

Apres S70, RRV et Factory consomment le process portable :

- RRV peut lire `sbfb-factory process status-sprint`,
  `lint-planning`, `audit-commit` et `AGENT_SYSTEM.md` comme premier
  corpus process-aware.
- Factory Viewer peut afficher les preuves/previews publiees.
- Factory Operator peut packager les templates/contrats/prompts.
- Les modes `@` sont des alias de roles, pas un systeme parallele.
- Factory ne possede pas l'autorite de verification.
