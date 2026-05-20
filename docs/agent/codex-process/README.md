# Codex Process Runbook

Ce dossier est le runbook Codex pour travailler dans `nexus-grid` avec un niveau
de rigueur comparable au process Claude deja en place.

Il ne remplace pas la source de verite. Les contrats canoniques restent :

- `docs/agent/PROCESS.md`
- `docs/agent/TOOLING.md`
- `prompts/agent/*.md`
- `scripts/agent/agentctl.py`
- `.planning/active/sprint{N}_*.md`
- `.githooks/pre-commit` et `.githooks/commit-msg`

Le role de ce dossier est different : il transforme ces contrats en protocole
d'execution Codex, avec des checklists operationnelles, des arrets obligatoires
et des templates a remplir avant de toucher au commit.

## Probleme vise

Le point faible observe sur Sprint 53 n'etait pas la direction technique. Le
fix `/api/daemon/*` etait correct. Le probleme etait le process :

- scope creep dans un commit annonce comme phase/fix cible ;
- changement securite non documente explicitement ;
- smoke utilisateur trop etroit avant commit ;
- second commit correctif qui aurait du etre attrape avant le premier commit.

Le runbook Codex doit rendre ces erreurs difficiles, pas seulement les decrire
apres coup.

## Regle racine

Un agent ne peut pas compenser un process faible par de la bonne volonte. Si un
risque doit etre pris au serieux, il doit etre visible dans l'un de ces
endroits :

- une commande executee ;
- un artefact `.planning/active/` ;
- une section obligatoire du review ;
- une section obligatoire du commit body ;
- un check `agentctl.py` ou hook.

Tout le reste est de la discipline non enforcee.

## Ordre d'utilisation

1. Lire `00_SESSION_START.md` au debut de session.
2. Lire `01_PHASE_DRIVER.md` avant implementation.
3. Lire `02_REVIEWER_AUDIT.md` avant de declarer la phase bonne.
4. Lire `03_COMMIT_GATE.md` avant staging/commit.
5. Lire `04_DOMAIN_SMOKE_MATRICES.md` pour choisir le smoke adapte au domaine.
6. Lire `05_AUTOMATION_BACKLOG.md` quand un probleme doit devenir un check.
7. Utiliser `templates/` pour produire les artefacts repo-visibles.

## Stop conditions

Codex doit stopper et produire un diagnostic au lieu de coder quand :

- le plan et le diff divergent sans section `Deviation accepted` ;
- le diff touche une decision Day 0 gelee ;
- un changement securite/protocole n'a pas de `Security delta` ;
- le review trouve un P0/P1 non corrige ;
- le review est encore `PASS-PENDING` ou ne contient pas exactement
  `## Verdict: PASS` pour un commit de phase ;
- le commit body de phase n'a pas exactement 9 sections avec
  `## Codex verification` ;
- trois P1 ou plus pointent vers une mauvaise conception ;
- les tests obligatoires sont rouges ;
- le working tree contient des changements non lies et non compris ;
- un fichier untracked est ambigu.

## Non-goals

- Ne pas dupliquer tout `docs/agent/PROCESS.md`.
- Ne pas deplacer `.planning/active/`.
- Ne pas creer un process Codex cache dans la memoire du modele.
- Ne pas rendre Claude obligatoire : le process doit fonctionner avec Codex,
  Claude, GPT, local LLM ou humain.

## Definition de "niveau Claude"

Le niveau attendu n'est pas une marque de modele. C'est une qualite observable :

- les critiques sont evidence-based ;
- le reviewer cherche activement les P1/P2 ;
- un PASS sans findings est suspect et doit etre justifie ;
- les changements de securite sont nommes ;
- les commits sont atomiques ;
- les smoke tests couvrent le chemin utilisateur reel ;
- les carry-over sont visibles dans `.planning/active/` ou archive ;
- les hooks peuvent bloquer ce qui ne doit pas passer.
