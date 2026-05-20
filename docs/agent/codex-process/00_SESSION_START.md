# 00 - Session Start

But : demarrer une session Codex sans charger le repo entier et sans se baser
sur une memoire stale.

## Commandes obligatoires

Executer avant toute lecture large :

```bash
git log --oneline -10
git status --short --branch
ls .planning/active
python scripts/agent/agentctl.py context
```

Si la session porte sur un sprint phase :

```bash
rg -n "Sprint|Phase|Scope cuts|Commit cible|Tests plan|Research|G8|Verdict|PASS-PENDING|Codex verification" .planning/active
git log --oneline -20
```

Si la session porte sur le process :

```bash
rg -n "G[0-9]|phase|review|precommit|auditor|scope|commit|gate|staging|agentctl" docs/agent prompts/agent scripts/agent .githooks
```

## Classification du cas

Classer en un seul cas avant d'agir :

- `Audit gate` : audit independant, pas d'implementation.
- `Sprint phase` : implementation atomique apres preflight.
- `New sprint` : planning/design, pas de code.
- `Hotfix` : correction ciblee hors sprint, pas de modification planning sauf
  demande explicite.
- `Process hardening` : docs/prompts/hooks/agentctl, verification du contrat
  avant modification.

Si deux cas semblent valides, le cas le plus restrictif gagne.

## Resume utilisateur obligatoire — juste apres classification

Avant toute lecture large ou code, donner 5 a 10 lignes :

- cas detecte ;
- HEAD actuel ;
- etat working tree ;
- phase/sprint vise ;
- identite phase stricte si plusieurs phases apparaissent ;
- fichiers vraiment pertinents ;
- risques de scope ;
- tests qui seront utilises ;
- stop condition connue.

Si le cas est determine par le process, executer sans demander confirmation.

## Dirty tree policy

Ne jamais ignorer `git status`.

- Changements connus de la session courante : continuer seulement si le nouveau
  travail les concerne.
- Changements non lies : ne pas les modifier, ne pas les formatter, ne pas les
  committer.
- Changements interrompus par un tour precedent : les nommer explicitement dans
  le final et eviter de les melanger.
- Untracked ambigu : STOP et demander.
- Cache/build evident : ajouter au `.gitignore` seulement si le process courant
  l'autorise.

## Lecture ciblee

Lire le minimum utile :

- `docs/agent/PROCESS.md` et `docs/agent/TOOLING.md` pour le process.
- `docs/claude/README.md` §6-§7 pour le workflow sprint complet (lifecycle
  kickoff, plan, phases, verification, audit_plan, commit discipline, delta
  tests cumule). Ce fichier est la source de verite du cycle sprint — le
  runbook Codex le complete, il ne le remplace pas.
- `.planning/active/sprint{N}_kickoff.md` et `sprint{N}_plan.md` pour une phase.
- `sprint{N}_phase_{X}_preflight.md` pour G8.
- `sprint{N}_phase_{X}_review.md` pour le gate.
- fichiers touches par le diff.

Pour une phase, `PASS-PENDING` n'est qu'un etat de transition avant Codex.
Ne pas committer tant que le review final ne contient pas exactement
`## Verdict: PASS` et que le body n'a pas ses 9 sections avec
`## Codex verification`.

Ne pas lire toute `.planning/archive/` sans raison. Utiliser `rg` pour trouver
les decisions historiques.
