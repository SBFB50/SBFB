# Brief de review de phase — orchestration Workflow

Ce fichier est le brief que le main thread donne aux agents d'une review de
phase orchestrée en Workflow. Il complète `phase-review.md` (qui décrit CE QUE
la review cherche) en décrivant **COMMENT elle a le droit de le chercher**.

## Pourquoi ce fichier existe

Jusqu'au Sprint 83 Phase M, ce brief n'était **dans aucun fichier du dépôt** :
il était réécrit dans le script Workflow de chaque session, jamais versionné,
jamais relu. Trois conséquences mesurées sur cette seule phase :

- les briefs demandaient « mesure par mutation » sans jamais dire **où** muter,
  donc les agents ont modifié le code de production ;
- cinq artefacts de review ont survécu à leur run — un nonce AES-GCM constant,
  une clé privée de son enveloppe zeroisante, une **clé dérivée du nonce public
  donc entièrement prédictible**, et deux fichiers sonde ;
- la détection reposait entièrement sur un `git status` manuel.

Un brief qui n'est pas un artefact du dépôt ne peut pas être corrigé une fois
pour toutes. Celui-ci l'est.

## Contraintes d'exécution — non négociables

Elles valent pour **tout modèle**. Le différenciateur n'est pas la puissance de
l'agent, c'est l'absence de règle : les prompts antérieurs n'en portaient
aucune.

1. **Ne jamais modifier le code de production, même temporairement.** L'arbre de
   travail appartient à la phase en cours, pas à la review.

2. **Toute mutation se fait dans un worktree isolé.** Créer un worktree jetable
   (`git worktree add <scratch>/<label> HEAD`), muter là-bas, mesurer, et le
   supprimer. Si le worktree ne peut pas être créé, la mutation n'a pas lieu :
   on RAPPORTE l'impossibilité, on ne se rabat pas sur l'arbre principal.

3. **Aucune sonde ne survit à son run.** Un fichier de test jetable est une
   MESURE, jamais un livrable. Le supprimer fait partie de la mesure. S'il
   trouve quelque chose, ce sont ses RÉSULTATS qui remontent dans le rapport,
   pas le fichier.

4. **Vérifier l'arbre en fin de run et le dire.** `git status --porcelain` et
   `git diff` doivent rendre exactement ce qu'ils rendaient au départ. Le
   rapport final l'affirme explicitement, avec la commande.

5. **Un correctif proposé se décrit, il ne s'applique pas.** La review dit ce
   qui ne va pas et ce qu'il faudrait faire ; l'écriture reste au main thread,
   séquentielle, dans un commit atomique.

Backstop mécanique — parce qu'une règle s'oublie et qu'un gate non : le hook
`phase-precommit-lightcheck.sh` (Check 0bis) BLOQUE un commit qui stage un
fichier `zz_*`/`*probe*`/`*.bak` ou une ligne portant un marqueur `MUTATION`,
et AVERTIT sur tout untracked sous `crates/` ou `web/src`. Ce contrôle est
mutation-testé dans les deux sens.

## Ce que la review doit chercher en priorité

Le fond reste celui de `phase-review.md`. Les trois angles ci-dessous sont ceux
qui ont réellement produit sur les phases L1 et M, et qu'aucune lecture de diff
ne donne :

- **La discriminance de chaque contrôle.** Pour chaque test : *que se
  passerait-il si la propriété qu'il nomme était cassée ?* Trois contrôles de la
  Phase M nommaient une garantie sans la garder — un `match` sur les éléments
  d'une liste prouvant que chaque élément est un variant et jamais que chaque
  variant est présent, une comparaison de blobs entiers censée vérifier un
  nonce, une vérification par sous-chaîne d'un `Debug` qui déversait la clé.

- **Les attentes dérivées du code sous test.** Un test qui écrit
  `if x.predicate() { assert(A) } else { assert(B) }` ne peut pas échouer quand
  `predicate` est ce qui change. La forme correcte est une table littérale
  indépendante.

- **Les affirmations des doc-comments.** Une phrase fausse dans un commentaire
  de sécurité se cote **P1** : l'implémenteur de la phase suivante code d'après
  elle. Quatorze des dix-neuf P1 de la Phase L1 étaient de cette classe.

Et une garde sur la review elle-même : **un finding portant sur un correctif du
round précédent mérite le même scepticisme qu'un finding sur le code d'origine**
— vingt-et-un défauts de la Phase L1 avaient été écrits par des correctifs.

## Ce que la review ne doit pas faire

- Ne pas signaler comme manque ce qui est un **scope cut explicite** du plan.
- Ne pas réclamer un appelant de production pour une phase de fondation qui
  assume `0 call-site` : le dire est la bonne réponse, l'exiger ne l'est pas.
- Ne pas inventer d'ancre. Les numéros de ligne dérivent — re-grep avant de
  citer, et un finding sans ancre vérifiable ne compte pas.

## Note honnête sur ce que ce fichier ne règle pas

Ces contraintes règlent la **pollution** de l'arbre, pas la **convergence** de
la boucle. Le rendement d'un round de review ne décroît pas mécaniquement : sur
la Phase L1, la trajectoire des P1 par round était 8, 9, 3, 0, 1, 5, 6, 4, 2, 0
— elle remonte, parce que chaque lot de correctifs ouvre une surface neuve.
Borner le nombre de rounds est un arbitrage PO distinct, laissé ouvert au
2026-07-28 : la vérification croisée externe (Codex) reste le seul juge qui lit
la source plutôt que le récit du diff.
