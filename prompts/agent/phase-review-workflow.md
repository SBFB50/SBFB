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

2. **Toute mutation se fait dans un worktree isolé.** Créer un worktree jetable,
   **y porter le code de la phase**, muter là-bas, mesurer, puis le supprimer :

   ```
   git worktree add <scratch>/<label> HEAD
   git diff HEAD -- <chemins de la phase> | git -C <scratch>/<label> apply
   git -C <scratch>/<label> status --porcelain   # doit égaler `git status --porcelain`
   ```

   **L'étape 2 n'est pas optionnelle et manquait à la première rédaction de ce
   brief** (mesuré le 2026-07-29) : un worktree ouvert sur `HEAD` ne contient
   aucun fichier neuf d'une phase non committée — `git cat-file -e HEAD:<fichier
   neuf>` échoue — donc la mutation y porte sur du vide et la mesure est
   vacueuse tout en paraissant réussir. Le contrôle `status --porcelain` de la
   troisième ligne est ce qui rend ce défaut visible.

   Si le worktree ne peut pas être créé, la mutation n'a pas lieu : on RAPPORTE
   l'impossibilité, on ne se rabat pas sur l'arbre principal.

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

6. **La longueur n'est pas une mesure de qualité** (amendement 2026-07-29,
   Opus 5). Un finding = une ancre + une mesure + un correctif. Ne re-narre pas
   le diff : l'auteur l'a écrit, il n'a pas besoin qu'on le lui raconte. Ce qui
   se coupe est la paraphrase — jamais la substance, jamais une mesure, jamais
   un finding. Pourquoi cette contrainte existe : Opus 5 écrit des fichiers plus
   longs que 4.8 et **l'effort ne les raccourcit pas** — seule une instruction
   le fait. Repère mesuré : les rapports de la Phase L1 atteignaient 124 Ko, où
   la part portante tenait dans un tableau de findings.

Note d'outillage — `isolation: 'worktree'` **ne remplace pas** la recette de la
contrainte 2 : le worktree natif part d'un commit, or le code d'une phase en
cours ne l'est pas. Il est le bon outil pour muter du code déjà committé (audit
gate, recherche sur l'historique), pas pour une review pre-commit. Cf. README
§7.1.2.

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

## Pour le CORRECTEUR — le gate de correctif (amendement 2026-07-29)

Cette section ne s'adresse pas aux agents de review mais au **main thread**, qui
écrit les correctifs entre deux rounds. Elle existe parce que l'asymétrie
suivante est la cause dominante des rounds supplémentaires :

> le LIVRABLE passe préflight + review + Codex ; le CORRECTIF n'passe rien,
> jusqu'au round suivant. Il entre dans l'arbre sans contrôle propre.

Trois règles, dans l'ordre de leur rendement mesuré sur S83 Phase M.

**1. Un correctif n'entre pas dans l'arbre sans sa mutation.** Pour chaque
correctif, produire la mutation qui le fait ROUGIR, avec la commande et la
sortie réelle. « Les tests passent » ne prouve rien : ils passaient déjà avant
le finding. Pour un correctif documentaire, la mutation est le `grep` qui prouve
que l'affirmation est vérifiable — s'il n'existe pas, l'affirmation est une
décoration. C'est ce contrôle qui a attrapé la garde redondante décrite au §P76
de `docs/rust/PATTERNS.md`, qu'aucune relecture n'aurait montrée.

Piège d'outillage mesuré le 2026-07-29 : `python` sous Windows ne résout PAS un
chemin MSYS `/c/...`. Une mutation « appliquée » par python sur un tel chemin
échoue silencieusement et laisse les tests verts **sur du code intact** —
mesure vacueuse qui paraît réussir, exactement le défaut que la recette worktree
de la contrainte 2 combat. Utiliser `perl -0777 -pi -e`, et VÉRIFIER par
`grep -c` que la mutation est dans le fichier avant de lancer quoi que ce soit.

**2. Fermer la famille, pas l'instance — et l'écrire.** Un finding « X manque
dans Y » oblige à énumérer tous les Y de la famille AVANT de corriger, et à
poser l'inventaire dans le rapport ou le commit. Sans cette trace, le round
suivant retrouve la même classe ailleurs et le compteur de P1 remonte sans que
rien n'ait régressé. Mémoire : `feedback_close_class_not_instance`.

**3. Ne jamais ajouter une garde redondante.** Cf. §P76 : une condition, un
endroit. La « défense en profondeur » n'est valide qu'entre couches aux modes de
défaillance INDÉPENDANTS.

## Note honnête sur ce que ce fichier ne règle pas

Ces contraintes règlent la **pollution** de l'arbre et le **contrôle du
correctif**, pas la convergence en elle-même. Le rendement d'un round ne décroît
pas mécaniquement : sur la Phase L1 la trajectoire des P1 par round était
8, 9, 3, 0, 1, 5, 6, 4, 2, 0 ; sur S83 Phase M, 5, 6, 2, 4, 1. Elle remonte,
et le diagnostic affiné le 2026-07-29 distingue TROIS causes qu'il ne faut pas
confondre — les confondre donne le pessimisme « chaque round casse quelque
chose », qui est faux :

- **vraie régression** (le correctif écrit un défaut neuf) — 2 sur les 18 P1 de
  S83 Phase M. Remède : règle 1 ci-dessus ;
- **balayage de famille incomplet** (le correctif ferme l'instance montrée et
  laisse ses sœurs) — cause DOMINANTE. Remède : règle 2 ;
- **défaut pré-existant révélé tard** (les rounds creusent plus profond). Aucun
  remède : c'est le process qui fonctionne.

Borner le nombre de rounds reste un arbitrage PO distinct. Un critère d'arrêt
plus honnête que « 0 P1 » : **0 P1 ET tous les correctifs du dernier round
mesurés discriminants** — sinon le round suivant re-trouve ce que le dernier lot
a introduit. La vérification croisée externe (Codex) reste le seul juge qui lit
la source plutôt que le récit du diff.
