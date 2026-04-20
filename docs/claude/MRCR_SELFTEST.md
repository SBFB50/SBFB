# MRCR self-test — Phase 0 check pour session fraîche

Document de discipline optionnelle à jouer en **Phase 0 de chaque
sprint** ou au démarrage d'une session fraîche sur un sprint non
trivial. Le but : détecter la régression MRCR (Multi-Round Context
Retrieval) mesurée par Anthropic sur Opus 4.7 (system card : -32.7pp
@256K / -46.1pp @1M vs Opus 4.6) AVANT d'engager du code critique.

## Pourquoi

Le projet charge ~40-50k tokens de contexte à chaque session fraîche
(MEMORY.md + nexus_grid_pivot.md + CLAUDE.md + docs/claude/README.md +
`.planning/active/`). Si Claude ne retrouve pas correctement des règles
plantées dans ce contexte cross-session, les phases A-E produisent du
code qui viole silencieusement les conventions (band-aid, scope cuts
rouverts, templates obsolètes).

Le test **n'est pas** une mesure scientifique — c'est un signal binaire :
« Claude retrouve-t-il ce qu'il a chargé ou invente-t-il ? »

## Protocole

L'utilisateur pose les 3 questions ci-dessous **avant** la première
action de Claude sur le sprint. Claude doit répondre **sans** relire
les fichiers — uniquement depuis le contexte déjà chargé. Si Claude
déclenche Read/Grep pour répondre, le test est invalidé : il faut
poser les questions à une session encore plus fraîche.

Baréme : 1 point par réponse juste, 0 sinon. Seuils de décision :

| Score | Verdict | Action recommandée |
|---|---|---|
| 3/3 | Mémoire contextuelle OK | Procéder sprint normalement avec Opus 4.7 |
| 2/3 | Dégradation probable | Envisager rester sur Opus 4.6 pour Phase 0 audit gate + G1 Design Review Board ; Opus 4.7 OK pour Phases A-F si confiance confirmée sur phase triviale |
| ≤ 1/3 | Régression majeure | Basculer modèle sur Opus 4.6 pour tout le sprint ; relancer le test à la prochaine session fraîche |

La bascule de modèle se fait via Config / `/model` en slash command.
Le toggle d'effort (high/xhigh/max) n'adresse PAS la régression MRCR —
effort = profondeur de raisonnement, pas fidélité à un contexte chargé.

## Les 3 questions

### Q1 — Pattern de pensée (source : `feedback_approach.md` §1)

> « Quand un sprint précédent a documenté un scope cut, cette décision
> devient-elle une contrainte technique ou une donnée de priorisation
> à réévaluer ? Cite le sprint où cette règle a été rédigée et la
> raison. »

**Réponse attendue** : donnée de priorisation à réévaluer à chaque
nouveau sprint, pas une contrainte technique. Rédigée après Sprint 5/11
parce que l'utilisateur a dû pousser 3× pour que Claude abandonne
l'hypothèse « iframe = Sprint 13+ » quand 60% du code était déjà
fonctionnel (WebAppFrame.tsx, CAS upload, GET /files/{sha256}). La
règle se lit : « lancer un agent Explore avant de déclarer quelque
chose trop gros ».

### Q2 — Règle band-aid (source : `docs/claude/README.md §6.3` + `CLAUDE.md`)

> « Un test échoue après un edit en Phase C. Tu as le choix entre
> `#[ignore]`, fix root cause, ou `--no-verify`. Quelle est la règle
> du projet et quelle en est la justification historique ? »

**Réponse attendue** : toujours root cause. Pas de `#[ignore]`,
`xfail`, `--no-verify` sauf si explicitement demandé. Un hook pré-commit
échoué signifie que le commit **n'a pas eu lieu** — donc `--amend`
modifierait le PRÉCÉDENT commit, risque de perte. Règle historique
rendue stricte après S7 singleton band-aid et S18 D-1 wire manquant
relevé à l'audit : la corrélation directe « temps research+doc avant
= temps économisé debug+rework après ».

### Q3 — Vérification finale (source : `docs/claude/README.md §7.4` + `CLAUDE.md` §Commandes clés)

> « Pour une phase qui touche du code TypeScript dans `web/`, cite
> les 3 commandes frontend obligatoires AVANT commit et l'ordre
> exact. Ne cite PAS `cargo`, `uv`, `pytest`. »

**Réponse attendue** (dans l'ordre) :
1. `npx tsc --noEmit -p tsconfig.app.json`
2. `npm run lint`
3. `npm run test:unit`

Puis `npm run build` + `npm run size` + `npx playwright test` +
`bash scripts/scan-en-strings.sh` pour la vérif finale. Une réponse
qui cite Prettier isolé, ou oublie scan-en-strings.sh (garde FR) =
-0.5 point.

## Notes

- Ne pas pré-communiquer les réponses à Claude (biaise le test).
- Si Claude hallucine une 4e commande plausible (ex: `vitest run` au
  lieu de `npm run test:unit`) → compter juste car l'intention est
  bonne, mais noter le drift comme signal faible.
- Les 3 questions couvrent 3 dimensions différentes de la mémoire
  cross-session : raisonnement meta (Q1), règle dure (Q2), détail
  opérationnel (Q3). Une régression sélective (ex: 3/3 sauf Q3) est
  un signal utile : Claude retient les principes mais perd les
  détails de commandes — sur un sprint tooling/CI, éviter 4.7.

## Historique de mesures

Tenir une table des passages quand le test est effectivement joué :

| Date | Sprint | Modèle | Q1 | Q2 | Q3 | Score | Décision |
|---|---|---|---|---|---|---|---|
| — | — | — | — | — | — | — | — |

(Ajouter 1 ligne par mesure. Document vivant.)

## Refs

- Anthropic Opus 4.7 system card (MRCR regression -32.7pp @256K / -46.1pp @1M)
- `memory/feedback_approach.md` (règles de pensée plantées)
- `docs/claude/README.md §6.3` (band-aid)
- `docs/claude/README.md §7.4` (verification)
- `CLAUDE.md` §Commandes clés (script finale)
