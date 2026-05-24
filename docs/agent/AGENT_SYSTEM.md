# Agent System

Carte du systeme agent nexus-grid. Ce document structure les roles,
providers, modes, gates et prompts du process portable. Il ne
duplique pas `PROCESS.md` (le workflow sprint complet) ni
`TOOLING.md` (les commandes). Il les indexe et les relie.

Autorite : `repo files > .planning/active/ > commits > prompts > chat`.

---

## 1. Truth Stack

L'autorite descend du repo vers le chat. Quand deux sources
divergent, la plus haute gagne.

| Rang | Source | Exemple | Mutabilite |
|------|--------|---------|------------|
| 1 | Repo files | `PROCESS.md`, `TOOLING.md`, `CLAUDE.md`, code, tests | Commit seul |
| 2 | Planning artifacts | `.planning/active/sprint{N}_*.md` | Commit phase ou chore |
| 3 | Commit history | Bodies, titles, `git log` | Immutable |
| 4 | Prompts | `prompts/agent/*.md`, `.claude/agents/*.md` | Commit seul |
| 5 | Chat / model memory | Historique de session, memoire privee | Ephemere, non-autoritaire |

Un fait non present dans les rangs 1-4 doit etre marque
`Not evidenced` par l'agent recepteur. La memoire de chat privee
et la memoire modele ne sont jamais autoritaires.

---

## 2. Role Registry

9 roles du process portable. Tout provider peut remplir tout role
s'il peut lire le repo, executer des commandes et ecrire
l'artefact requis.

| Role | Droits | Obligations | Limites |
|------|--------|-------------|---------|
| `driver` | Editer fichiers, lancer tests, ecrire evidence phase | Suivre le plan, respecter scope cuts, produire les artefacts de phase | Ne signe pas le verdict final (PASS-PENDING seulement) |
| `reviewer` | Lire diff complet, lancer suites verification | Couvrir 11 dimensions review, produire verdict PASS-PENDING/CONCERN/FAIL | Independant du driver ; ne code pas |
| `auditor` | Lire diff, artefacts, planning | Couvrir 7 dimensions audit, produire verdict | Independant du driver et du reviewer |
| `codex-verifier` | Lire diff et artefacts review | Verifier chaque livrable, produire evidence fichier:ligne | Output brut non-reecrit ; zero exemption |
| `researcher` | Lire repo, web, registres, papers | Produire references, tradeoffs, evidence factuelle | Ne code pas ; ne produit pas de verdict |
| `preflight-checker` | Lire repo, web, historique | Executer 5 scans S1-S4, produire verdict EXECUTE/PLAN-ADAPT/SCOPE-CUT-CONSISTENT/DESIGN-CONFLICT | Ne code pas ; ne tranche pas DESIGN-CONFLICT |
| `kickoff-author` | Lire roadmap, recherche, planning | Produire kickoff + plan + design review (D1-D5, G1-G2-G7-G9) | Ne code pas ; decisions D1-D5 gelee apres kickoff |
| `audit-gate-runner` | Lire diff sprint complet, artefacts | Jouer 9 tracks audit, classifier P0-P3, produire verdict PASS/CONDITIONAL/FAIL | Ne code que les fix P0/P1 |
| `process-supervisor` | Lire plan, artefacts, repo state | Surveiller gates, envoyer BLOCK si deviation | Ne code jamais ; ne modifie jamais de fichier ; optionnel (D17) |

### Combinaisons typiques

- Sprint standard : `driver` + `reviewer` + `codex-verifier` + `preflight-checker`
- Kickoff : `kickoff-author` + `reviewer` (design review independant)
- Audit gate : `audit-gate-runner`
- Supervision : `process-supervisor` (optionnel, hooks = backstop)

---

## 3. Provider Mapping

Tout provider qui lit le repo et ecrit des fichiers peut jouer
tout role. La profondeur varie selon les capacites.

| Provider | Contexte max | Outils specifiques | Roles naturels | Limites |
|----------|-------------|-------------------|----------------|---------|
| Claude (Opus/Sonnet) | 1M tokens | WebSearch, context7, Read 1M, Agent spawn | Tous | Pas d'execution shell autonome sans session |
| Codex (GPT 5.5) | Variable | `codex exec`, shell sandbox | `codex-verifier`, `driver` | Output brut ; pas de web search |
| GPT (4o/o1/5) | 128K-1M | WebSearch, code interpreter | `driver`, `reviewer`, `researcher` | Pas de context7 ; adapter references |
| Local (Ollama, llama.cpp) | 8K-128K | Aucun externe | `driver` (bounded), `reviewer` (bounded) | Pas de web ; prompt court ; taches bornees |
| Humain | Illimite | Tous les outils manuels | Tous | Lenteur ; fatigue ; autorite finale |

### Adaptation provider

Quand `--provider local` : retirer les references WebSearch et
context7 des instructions. Quand `--provider codex` : formater
pour `codex exec -o`. La politique fine d'adaptation est
documentee dans `PROVIDER_CONFIG.md` (Phase F).

---

## 4. Lifecycle Modes

10 modes du cycle de vie sprint. Chaque mode pointe vers le(s)
prompt(s) portable(s) a utiliser.

| # | Mode | Prompt(s) | Role(s) actif(s) | Artefact produit |
|---|------|-----------|-------------------|------------------|
| 1 | `kickoff` | `universal.md` | `kickoff-author` | `sprint{N}_kickoff.md`, `sprint{N}_plan.md`, `sprint{N}_design_review.md` |
| 2 | `plan` | `universal.md` | `kickoff-author` | `sprint{N}_plan.md` (mise a jour) |
| 3 | `preflight` | `preflight.md` | `preflight-checker` | `sprint{N}_phase_{X}_preflight.md` |
| 4 | `implement` | `base.md` + `universal.md` | `driver` | Code + tests |
| 5 | `review` | `phase-review.md` | `reviewer` | `sprint{N}_phase_{X}_review.md` (verdict PASS-PENDING/CONCERN/FAIL) |
| 6 | `codex-verify` | Per-phase prompt (cf. `codex-process/`) | `codex-verifier` | `sprint{N}_phase_{X}_codex_review.md` |
| 7 | `commit` | `commit-body.md` | `driver` | Commit `type(scope): Sprint N Phase X ...` |
| 8 | `audit-gate` | `audit-gate-checks.md` | `audit-gate-runner` | `sprint{N}_audit_findings.md` |
| 9 | `verification` | `universal.md` | `driver` | `sprint{N}_verification.md` |
| 10 | `handoff` | `handoff.md` | Tout role | Context-pack point-in-time pour le recepteur |

### Sequencement standard

```
kickoff → plan → [pour chaque phase:]
  preflight → implement → review → codex-verify → commit
→ verification → audit-gate (sprint suivant)
```

Le mode `handoff` peut intervenir a tout moment pour transferer
a un autre provider.

---

## 5. Gate Contract

Formalise les verdicts et artefacts de chaque gate.

### 5.1 Preflight (G8)

| Verdict | Condition | Action |
|---------|-----------|--------|
| `EXECUTE` | Plan aligné, aucun conflit detecte | Coder la phase |
| `PLAN-ADAPT` | Approche corrigee par evidence OSS concrete | Coder selon l'adaptation, pas le plan original |
| `SCOPE-CUT-CONSISTENT` | Scope cut prevu, pas de surprise | Coder la phase (scope reduit confirme) |
| `DESIGN-CONFLICT` | Contradiction Day-0 ou wire format | STOP : presenter pivot_proposal, attendre arbitrage utilisateur |

Artefact : `sprint{N}_phase_{X}_preflight.md`
(ou `sprint{N}_phase_{X}_pivot_proposal.md` si DESIGN-CONFLICT).

Contenu obligatoire : S1a (OSS prior art), S1b (deps/CVE),
S2 (decisions historiques), S3 (threat model), S4 (wire format).

### 5.2 Review (post-implementation)

| Verdict | Condition | Action |
|---------|-----------|--------|
| `PASS-PENDING` | Review Claude OK, Codex pas encore fait | Passer a Codex verification |
| `CONCERN` | Issues P2/P3, pas de P0/P1 | Documenter dans commit body ; continuer |
| `FAIL` | Issues P0/P1 detectees | Corriger, re-invoquer review |

Artefact : `sprint{N}_phase_{X}_review.md`

Le verdict final `## Verdict: PASS` n'est ecrit qu'apres
reconciliation Codex. `PASS-PENDING` n'est jamais committable.

11 dimensions review : staging coherence, scope-cuts semantique,
branch coverage (appel reel, assertion, cas limites, inputs
realistes), research grounding, security OWASP 9 patterns,
patterns drift, horizon long-terme, livrables check, body format
9/9, codex reconciliation, carry routing.

### 5.3 Codex verification

| Verdict | Condition | Action |
|---------|-----------|--------|
| `CLEAN` | Tous livrables verifies, aucun gap | Promouvoir review a PASS |
| `GAP-P0` | Gap critique (securite, invariant casse) | Corriger → boucle review+codex |
| `GAP-P1` | Gap important (test manquant, scope non-couvert) | Corriger → boucle review+codex |
| `GAP-P2-P3` | Gap mineur (style, optimisation) | Documenter dans commit body |

Artefact : `sprint{N}_phase_{X}_codex_review.md`

Regles : output brut `codex exec -o`, jamais reecrit par Claude.
Zero exemption : toutes les phases y compris docs-only.

### 5.4 Audit gate (Phase 0)

| Verdict | Condition | Action |
|---------|-----------|--------|
| `PASS` | 0 P0/P1, au moins 1 P2+ documente ou evidence negative exhaustive | Sprint suivant peut commencer |
| `CONDITIONAL PASS` | 0 P0/P1, conditions a respecter | Commencer avec conditions |
| `FAIL` | P0 ou P1 presents, ou >=3 P1 | Fix requis avant de continuer |

Artefact : `sprint{N}_audit_findings.md`

9 tracks : A suites, B security, C patterns, D scope, E tests
delta, F review files, G carry-overs, H HARDENING, I meta-process.

Classification :
- P0 : securite ou invariant protocolaire casse
- P1 : regression fonctionnelle ou process gate manquant
- P2 : dette technique, style, documentation
- P3 : amelioration optionnelle

### 5.5 Artefact contract

| Gate | Fichier requis | Staged au commit | Verdict final |
|------|---------------|-----------------|---------------|
| Preflight | `sprint{N}_phase_{X}_preflight.md` | Non (pre-code) | EXECUTE ou equivalent |
| Review | `sprint{N}_phase_{X}_review.md` | Oui | `## Verdict: PASS` |
| Codex | `sprint{N}_phase_{X}_codex_review.md` | Oui | CLEAN ou GAP-P2-P3 documentes |
| Audit | `sprint{N}_audit_findings.md` | Oui (commit propre) | PASS ou CONDITIONAL PASS |

Sequence stricte pre-commit :
preflight → implement → review PASS-PENDING → codex → reconciliation
→ review promue PASS → commit.

---

## 6. Prompt Registry

Table des prompts portables executables. Chaque prompt vit dans
`prompts/agent/` et peut etre assemble par
`agentctl prompt --kind {kind}` (Python legacy) ou
`sbfb-factory process prompt --kind {kind}` (Rust, Phase C+).

| Kind | Fichier | Purpose | Providers compatibles | Depth |
|------|---------|---------|----------------------|-------|
| `base` | `prompts/agent/base.md` | Orientation courte, invariants projet | Tous | Toujours court |
| `universal` | `prompts/agent/universal.md` | Process sprint complet vendor-neutral | Tous | standard / deep |
| `preflight` | `prompts/agent/preflight.md` | G8 pre-code : 5 scans S1-S4 | Tous (profondeur reduite sans web) | standard / deep |
| `phase-review` | `prompts/agent/phase-review.md` | Post-code : 11 dimensions review | Tous | standard / deep |
| `phase-auditor` | `prompts/agent/phase-auditor.md` | Audit independant pre-commit : 7 dimensions | Tous | standard |
| `commit-body` | `prompts/agent/commit-body.md` | Template 9 sections commit body | Tous | standard |
| `handoff` | `prompts/agent/handoff.md` | Transfert inter-provider 9 sections | Tous | standard / deep |
| `audit-gate` | `prompts/agent/audit-gate-checks.md` | 9 tracks audit sprint | Tous | standard / deep |

Les agents Claude (`.claude/agents/*.md`) sont des wrappers qui
ajoutent les outils Claude-specifiques (WebSearch, context7, Read
1M tokens) au-dessus des prompts portables. Un provider sans ces
outils execute le meme workflow mais avec moins de profondeur.

### Bootstrap session fraiche

1. `base.md` → orientation invariante et regles evidence
2. `universal.md` → lifecycle sprint complet et gates
3. `context` (runtime) → faits repo live (HEAD, dirty files, sprint, phase)
4. `handoff.md` → etat point-in-time (phase, verdict state, carries, next actions)
5. Prompt specialise → prochaine action de gate

La memoire de chat privee n'est jamais autoritaire. Si un fait
n'est pas dans les fichiers repo, le runtime context ou le
handoff, l'agent recepteur ecrit `Not evidenced`.

---

## 7. Non-Goals

Ce que AGENT_SYSTEM.md ne fait pas :

- **Pas un process complet** : le workflow sprint detaille vit dans
  `PROCESS.md`. Ce document l'indexe.
- **Pas un runbook provider** : les instructions specifiques Claude
  vivent dans `.claude/agents/` et `docs/claude/README.md`. Les
  instructions Codex vivent dans `docs/agent/codex-process/`.
- **Pas un outil** : les commandes vivent dans `TOOLING.md` et
  `scripts/agent/agentctl.py` (legacy) / `crates/sbfb-factory` (Rust).
- **Pas une autorite de verdict** : les verdicts sont produits par
  les gates executees, pas par ce document.
- **Pas un substitut au repo** : si ce document diverge du code ou
  de PROCESS.md, le code et PROCESS.md font autorite (Truth Stack
  rang 1).
