# Contribuer a NEXUS GOV

Merci de votre interet pour NEXUS GOV. Ce projet est ouvert a toutes les contributions.

## Comment contribuer

### Signaler un bug
1. Verifiez qu'il n'existe pas deja dans les [Issues](https://github.com/FlowUP/nexus-gov/issues)
2. Creez une issue avec : description, etapes pour reproduire, comportement attendu vs observe
3. Ajoutez les logs pertinents si possible

### Proposer une fonctionnalite
1. Ouvrez une [Discussion](https://github.com/FlowUP/nexus-gov/discussions)
2. Decrivez le besoin, l'approche proposee, et l'impact attendu
3. Attendez un retour avant de coder

### Soumettre du code
1. Forkez le repo
2. Creez une branche depuis `main` : `git checkout -b feature/ma-feature`
3. Codez, testez, commitez
4. Ouvrez une Pull Request avec description claire
5. Review sous 48h

## Standards de code

### Python
- Formatter : black (ligne max 120)
- Imports : isort
- Types : annotations pour les fonctions publiques
- Tests : pytest, meme repertoire `tests/`

### TypeScript/React
- ESLint config du projet
- Composants dans `web/src/components/gov/`
- Types dans `types.ts`
- Hooks dans `web/src/hooks/`

### Commits
- Format : `type(scope): description`
- Types : feat, fix, docs, refactor, test, chore
- Exemple : `feat(gov): add thematic classification worker`

## Bonnes premieres contributions

Cherchez les issues taggees `good-first-issue` :
- Ajouter des tests pour un worker existant
- Ameliorer la documentation d'une source de donnees
- Corriger un bug d'affichage frontend
- Ajouter une traduction

## Code de conduite

Ce projet suit le [Contributor Covenant](CODE_OF_CONDUCT.md).
Respect, bienveillance, et constructivite sont attendus de tous les participants.
