# RRV / Factory Contract

Contrat entre le process portable agent (`PROCESS.md` +
`AGENT_SYSTEM.md`), le systeme RRV (Recherche Reseau Verifiable)
et la Factory (Viewer + Operator). Document cree Sprint 70 Phase G.

Autorite courte : process > RRV > Factory.
Autorite documentaire : `PROCESS.md` > `AGENT_SYSTEM.md` > ce document.
Ce document formalise les conventions d'usage, pas les regles
d'execution — celles-ci vivent dans PROCESS.md et les gates.

---

## 1. Modes @ — alias de roles portables

Les modes `@` sont des conventions d'intention, pas des commandes
executables. Ils mappent sur les 9 roles du registre
`AGENT_SYSTEM.md §2`.

| Mode | Role(s) mappe(s) | Intention | Exemples d'usage |
|------|-----------------|-----------|------------------|
| `@research` | `researcher` | Fact-finding, references, tradeoffs. Pas de code. | Recherche pre-kickoff, intake technologique, comparatif OSS |
| `@dev` | `driver` | Implementation, bounded edits, tests. | Code phase, fix P0/P1, refactor |
| `@audit` | `reviewer` + `auditor` | Quality pass, security review, scope check. | Review deep, audit gate, Codex reconciliation |
| `@security` | `reviewer` (dimension securite) | Threat model, loopback, sandbox, crypto. | Threat model update, HARDENING_ROADMAP, CVE triage |
| `@product` | `kickoff-author` (dimension intake) | Intake, decisions, scope, roadmap. | Kickoff D1-D5, recadrage PO, scope cuts |

### Regles

1. Un mode `@` est un **alias de commodite**, pas une autorite.
   L'autorite d'execution reste dans `.planning/active/` et les
   gates (`AGENT_SYSTEM.md §5`).
2. RRV **affiche** l'etat du process et les evidences. Il ne prend
   pas de decisions d'execution.
3. Les modes `@` ne creent pas de roles supplementaires. Ils
   re-exposent les roles existants du registre sous des noms
   orientes intention.
4. Un agent peut operer dans plusieurs modes `@` simultanement
   (ex: `@dev` + `@security` pour un hotfix securite).

### Sequencing post-S70

| Mode | Statut S70 | Prochain jalon |
|------|-----------|----------------|
| `@research` | Operationnel via `researcher` role | RRV @protocole corpus S71+ |
| `@dev` | Operationnel via `driver` role | @dev index tree-sitter S71+ (pas bloquant Gate 1) |
| `@audit` | Operationnel via `reviewer` + `auditor` roles | RRV @audit corpus S71+ |
| `@security` | Operationnel via `reviewer` role (securite) | Idem |
| `@product` | Operationnel via `kickoff-author` role | Idem |

---

## 2. Principe d'autorite

Principe court : **process > RRV > Factory**. RRV consomme,
affiche et recherche les preuves du process. Factory produit,
publie et expose des artefacts, mais ne remplace jamais les gates.

```
.planning/active/ + gates + commits
       ↑ autorite d'execution
       |
  PROCESS.md + AGENT_SYSTEM.md
       ↑ contrat process
       |
  RRV (affiche, indexe, recherche)
       ↑ consultation
       |
  Factory Operator (cree, publie, journalise)
       ↑ production
       |
  Factory Viewer (expose preuves, previews, statuts)
       ↑ lecture seule
```

Aucun composant downstream (RRV, Factory Viewer, Factory Operator)
ne possede l'autorite de verification finale. L'autorite descend du
Truth Stack (`AGENT_SYSTEM.md §1`) :

1. **Repo files** (PROCESS.md, code, tests) — commit seul
2. **Planning artifacts** (.planning/active/) — commit phase ou chore
3. **Commit history** — immutable
4. **Prompts** (prompts/agent/) — commit seul
5. **Chat / model memory** — ephemere, non-autoritaire

---

## 3. Factory Viewer — app SBFB sandboxee

Le Factory Viewer est une **app SBFB publiee sur le reseau**,
rendue dans un iframe sandbox (`sandbox="allow-scripts"` sans
`allow-same-origin`), identique a toute autre app SBFB.

### Capacites

- Afficher les artefacts de preuve (Proof Cards, provenance)
- Afficher les previews exportees ou publiees
- Afficher les statuts sprint (via bridge `status_sprint` si expose)
- Afficher les labels et badges de qualite

### Limites

- **Pas d'execution locale** : le Viewer ne lance pas de commandes,
  ne modifie pas de fichiers, ne commit pas.
- **Pas d'import Operator** : le Viewer n'importe jamais
  `factory-ui/operator` ni aucune extension privilegiee.
- **Pas d'autorite** : le Viewer affiche ce qui est publie, il ne
  valide pas la qualite.
- **Lecture seule** : socle partage `tools/factory-ui/src/readonly`
  pour les modeles, labels, previews et cartes de preuve.

---

## 4. Factory Operator — outil local privilegie

Le Factory Operator est un **outil local Rust** du noeud, servi
par `sbfb-factory operator serve`. Il n'est pas une app SBFB — il
tourne en local avec acces au filesystem et au daemon.

### Capacites

- Creer des projets/apps via templates (`sbfb-factory create`)
- Valider des manifestes (`sbfb-factory validate`)
- Publier des apps sur le reseau (`sbfb-factory publish`)
- Journaliser les actions dans `audit_log.jsonl`
- Exposer le statut sprint, le lint planning, l'audit commit
  via l'API JSON (`sbfb-factory operator serve`)
- Afficher les previews locales (`sbfb-factory preview`)
- Executer les gates Factory (FG0-FG10)
- Generer des context-packs et des handoffs

### Limites

- **Pas d'autorite de verification finale** : l'Operator pilote
  et journalise, mais le shell, le commit, le push et le verdict
  final passent par une vraie session agent, les gates et les
  preuves repo.
- **Actions allowlistees** : chaque action Operator est gated
  (confirmation ou guard Rust).
- **Socle partage en lecture** : reutilise `tools/factory-ui/src/readonly`
  comme le Viewer, plus les extensions `factory-ui/operator` pour
  les actions privilegiees.

### UX Operator

L'utilisateur voit des **intentions** lisibles :
- "Preparer la phase"
- "Verifier avant validation"
- "Transmettre a un autre agent"

Pas des commandes `sbfb-factory` ni du jargon `kind/provider/preflight`
en CTA principal. Le mode actuel "prompt de base + discussion agent
autonome" est preserve.

---

## 5. Babel — une app, pas le process

Babel est la premiere app SBFB candidate post-v1.0. Elle est
**creee avec Factory** (templates, publish pipeline, Proof Cards),
mais elle n'est pas le process lui-meme.

Distinction :
- **Babel = app** : contenu, traduction, collaboration.
- **Factory = outil** : creation, publication, gates.
- **Process = workflow** : preflight, review, Codex, commit.

Le dogfood Babel valide que Factory et le process portable
fonctionnent de bout en bout. Babel ne dicte pas le process —
elle le consomme.

---

## 6. Sequencing post-S70

S70 livre le process portable complet. Les surfaces suivantes
sont planifiees post-S70 :

| Surface | Sprint cible | Prerequis |
|---------|-------------|-----------|
| SearchManifest opt-in | S71 | Process portable S70 |
| RRV Core @protocole corpus | S71 | Process portable S70 |
| @dev index tree-sitter | S71+ | Pas bloquant Gate 1 |
| Gouvernance + Factory hardening | S72 | S71 |
| @web frontend RRV | S73+ | Post-pilote ferme |
| Provider router multi-LLM | S74+ | Post-S75 |
| Ingestion OSS generique | Post-S75 | Mode source-only/source-index distinct d'app SBFB |

### Decisions gelees qui s'appliquent

- Factory = crate externe (`sbfb-factory`), hors daemon (v4 D2)
- @protocole d'abord, puis @dev, puis @web (v4 D6)
- Ingestion OSS = futur mode `source-only`/`source-index` (CLAUDE.md)
- Gate 1 valide sur @protocole + Proof Cards + publish + Babel (D5 S70)
- OS sandbox pour Factory, pas wasmtime (12 CVE avril 2026)
- Superviseur process optionnel, hooks = backstop mecanique (D17)

---

## 7. Non-Goals

Ce que ce document ne fait pas :

- **Pas un registre de roles** : le registre vit dans
  `AGENT_SYSTEM.md §2`.
- **Pas un workflow** : le workflow sprint vit dans `PROCESS.md`.
- **Pas une spec Factory** : les gates FG0-FG10 vivent dans
  `docs/release/FACTORY_GATES.md`.
- **Pas une spec RRV** : RRV total est post-S70 (S71+).
- **Pas une autorite de verdict** : les verdicts sont produits par
  les gates, pas par ce document.
