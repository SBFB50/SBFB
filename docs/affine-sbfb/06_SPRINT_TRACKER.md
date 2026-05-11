# Sprint Tracker

Vue kanban pour le suivi des sprints. A importer dans AFFiNE comme
database/board.

## Sprint actuel

**Sprint 52** (A OUVRIR)
- Theme : binaires release cross-platform + VPS deployment
- Type : pair (phase dette obligatoire)
- Carries entrants : 5 (rand exemption, iroh transitives,
  dispatch join 2/3, unsafe set_var 1/3, docs legacy 1/3)

## Phases S52 (previsionnel)

| Phase | Scope | Statut |
|---|---|---|
| A | Phase dette (dispatch join 2/3 + docs legacy) | TODO |
| B | GitHub Actions release binaries Win/Mac/Linux | TODO |
| C | VPS bootstrap 3 noeuds + smoke test | TODO |
| D | Wrap-up + verification + audit plan S53 | TODO |

## Carries board

| ID | Description | Compteur | Urgence |
|---|---|---|---|
| P2-A-1 | rand blocker upstream | 12+/3 | Exemption |
| P2-AUDIT-2 | iroh pre-release transitives | herite | Pin 0.98 |
| P2-REVIEW-A-1-S50 | dispatch join order | 2/3 | MANDATORY si non adresse S52 |
| P2-REVIEW-B-1-S51 | unsafe set_var futur | 1/3 | Informationnel |
| P2-REVIEW-A-2-S51 | docs legacy orphelines | 1/3 | 21 fichiers |

## Roadmap v1.0

```
S52 binaires+deploy ──> S53 smoke test ──> S54 polish ──> TAG v1.0
     [pair/dette]        [impair]           [pair/dette]
```

## Post-v1.0 apps

| App | Description | Dependance v1.0 |
|---|---|---|
| Babel | Traduction P2P / corpus libre | package app + provenance + trust |
| Wiki P2P | Knowledge base collaborative CRDT | BlockSuite + y-octo + bridge |
| Forensique | Cold-case collaboration signee | provenance + CRDT + chain custody |
| Gouvernance | Propositions/votes blocs CRDT | trust + capabilities + bridge |
