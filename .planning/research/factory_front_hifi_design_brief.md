# Brief Claude Design — Passe HI-FI du front Factory Operator (SBFB, S80)

> **But.** Prompt prêt-à-coller dans **Claude Design** pour faire passer les wireframes
> **low-fi** existants (`Factory Operator - wireframes.dc.html`, projet « Factory Operator
> dev local ») au niveau **HI-FI dark, prêt-à-coder**. Ce n'est PAS une refonte : la
> largeur (écrans, états, variantes, planche, atomes, CTA-intentions) est acquise et bonne.
> Cette passe (1) **réconcilie 3 contradictions** avec des décisions déjà verrouillées,
> (2) **complète les variantes B/B retenues** (sous-développées), (3) **monte en hi-fi**
> (thème dark, tokens oklch, typo, densité, états d'interaction, motion).
> Grounded : `factory_front_wireframe_finalization_audit_2026-06-27.md` (l'audit qui motive
> cette passe) + `factory_front_operator_wireframe_brief.md` (le brief low-fi d'origine) +
> `factory_front_greenfield_blueprint.md` + kickoff S80 Day-0.
> **Handoff attendu** : `.dc.html` mis à jour (hi-fi dark, Q-réconcilié) + une **feuille de
> tokens** (valeurs oklch + typo + spacing + specs motion) déposés repo-visible dans
> `.planning/research/` AVANT que les phases front fidélité (C/D/E/H) ne codent.

---

## À COLLER DANS CLAUDE DESIGN (à partir d'ici)

Tu vas faire une **passe hi-fi** sur des wireframes low-fi existants d'un outil de dev
**local solo**, pas un SaaS. « Poussé » = **raffiné, dense, sobre**, lignée OpenBSD /
F-Droid / Tor / Sigstore — **jamais flashy/décoratif**. Conserve toute la structure et tout
le contenu déjà dessinés ; tu **élèves** le rendu et tu **corriges** les points ci-dessous.

### Rappel de l'outil (inchangé)
**Factory Operator** = établi **bi-focal** (PAS un IDE) : un **rail d'orientation ambiant
permanent** + **une scène mono-focale** qui ne montre qu'**UN** de deux MODES — **STEER**
(intention → observer/relancer l'agent) / **VERIFY** (diff → gates → preuve). Bascule
suit le travail, **jamais d'auto-bascule en plein stream**. Mono-utilisateur. Intentions,
pas jargon. Connaissance **consommée jamais autoritaire** (0 « PASS », 0 coche verte, 0
bypass). Le **MUR** de gouvernance est une **barrière**, pas un bouton.

### A. CORRECTIONS BLOQUANTES (réconcilier le dessin avec les décisions déjà prises)
Le low-fi actuel **contredit** 4 décisions verrouillées. Corrige-les — ce sont les
**surfaces porteuses** :

1. **Shell — barre d'orientation fine en HAUT (pas tout dans le rail gauche).**
   Bandeau horizontal mince permanent en haut : `Sprint 80 · Phase X ▸ branche ▸ ● N
   modifiés · M indexés ▸ ◑ gates 4✓ 1•`. Il **n'anime jamais**, ne transitionne jamais.
   Le **rail vertical gauche étroit** garde la **bascule MODE** (STEER / VERIFY en tête,
   groupés) + les sous-surfaces secondaires (terminal, sessions, historique, knowledge)
   dessous. (Aujourd'hui tout est empilé dans un rail gauche de 212px sans barre haute.)

2. **VERIFY — disposition variante B canonique (l'investissement central).**
   - **Colonne change-set à gauche** = liste de **fichiers**, **repliable** (repliée → diff
     plein), avec un **marqueur de gate par fichier** (aujourd'hui absent : la liste ne
     montre que `+/−`).
   - **Panneau principal à 3 onglets** `[Diff | Aperçu scellé | Preuve]` (Diff par défaut).
     **Gates n'est PAS un onglet** (le dessin actuel en fait un 4e onglet — interdit).
   - **Bande GATES + slot ÉTAT permanente en BAS, pleine largeur**, toujours visible (jamais
     derrière un onglet) : elle porte le « **0 verdict auto-clos** » non masquable. Le slot
     ÉTAT ne dit **JAMAIS « PASS »** (au mieux « En attente de session agent — 4✓ 1• · 0
     verdict auto-clos »). (Aujourd'hui le seul bandeau permanent est en HAUT.)

3. **MUR — barrière en-flux pleine largeur, PAS un modal.** Le MUR **interrompt
   physiquement** le composeur / le stream sur **toute la largeur** (encart en retrait,
   bordure lourde/hachurée), il ne flotte pas en carte centrée sur un scrim. **Une seule**
   action « Préparer le pack pour la session ». **Zéro** Forcer / Override / Bypass.

4. **STEER variante B — dessiner l'état vide manquant.** STEER-B = atelier dominant +
   composeur replié en **dock invocable** ; MAIS l'**état vide (pas de session)** doit
   montrer le **composeur en grand** (découvrabilité du démarrage). Cet état manque.

> Dessine les variantes **B/B retenues** au **même niveau de détail** que les A (le détail
> riche actuel vit dans les variantes A écartées — rééquilibre vers B).

### B. MONTÉE EN HI-FI (ce qui n'existe pas encore — à produire)
Le low-fi est en thème **clair** sans système. Produis le **hi-fi dark** + une **feuille de
tokens** :

- **Thème DARK, achromatique par défaut.** La **couleur ne porte QUE le signal d'état** :
  `ok` / `warn` / `bad` / `info` / `neutre = absence`. Aucune couleur décorative.
  Définis une **palette oklch** (pas hex) : 3-4 **couches de surface** sombres (fond le plus
  profond → panneaux → cartes), 2-3 niveaux de **texte** (primaire/secondaire/atténué), une
  couleur de **bordure** (séparation par la **ligne**, pas l'ombre), et les **5 teintes
  d'état** oklch (ok/warn/bad/info/neutre) calibrées pour le dark. Donne les **valeurs
  oklch** dans la feuille de tokens.
- **Typographie = le langage de preuve.** **Sans** (Geist Sans) = intention / prose humaine ;
  **Mono** (Geist Mono) = **toute** preuve/artefact machine (hash, diff, sha, nom de gate,
  provenance, terminal, états énumérés). `tabular-nums` sur les compteurs. Donne une **rampe
  typo** (tailles/poids/line-height) + les rôles.
- **Densité & espacement.** Composeur = **aéré/calme** ; surface VERIFY = **dense/tabulaire**.
  Séparation par la **ligne** (0 ombre SaaS). Donne une **échelle d'espacement**.
- **États d'interaction** pour chaque atome (bouton, ligne de gate, carte tool-call, onglet,
  item de change-set) : `hover` / `focus-visible` / `active` / `disabled` — sobres, dark.
- **5 signatures de motion (sens, jamais déco)** — spécifie chacune (durée/easing/déclencheur),
  à implémenter plus tard avec Motion : (1) **token settle** (une valeur qui se fixe),
  (2) **gate flip** (un gate qui change d'état), (3) **verification reveal** (la preuve qui se
  décompose), (4) **altitude shift** (bascule STEER⇄VERIFY via View-Transitions — **le rail
  est EXCLU**, il ne transitionne jamais), (5) **confirmation gravity** (le MUR qui s'impose).
  `prefers-reduced-motion` → **état final instantané** (anti-déco).

### Invariants UX NON NÉGOCIABLES (doivent rester vrais en hi-fi)
- **Intentions, pas jargon** : CTA en français clair ; `kind`/`provider`/`preflight`/hash
  repliés dans « ▸ détails techniques ». Provider = attribut discret.
- **Consommée jamais autoritaire** : l'UI ne calcule **aucun** verdict, n'affiche **jamais**
  « PASS ✓ ». Surfaces IA/knowledge marquées « consultatif — non autoritaire » (bordure
  pointillée, contraste réduit, **jamais de coche verte**).
- **MUR = vraie barrière** (l'avance de SBFB sur Cursor/Windsurf — on l'expose).
- **Décomposition, jamais verdict** : la preuve = chaîne `commit → archive_hash → signataire`
  en mono copiable, décomposée en couches (base/provenance/oss/curateurs/licence) + risques.
- **Diff = vérité** (`git diff`, pas un buffer). Actions de hunk = **intentions** routées à
  la session (« Transmettre la correction », « Signaler ce hunk »), **jamais** Approve /
  Merge / Commit / PASS.
- **États honnêtes = citoyens de 1re classe** : `PROVISIONAL` / `Not evidenced` / `non
  exécuté` / `RIG-ABSENT` ont un traitement visuel **assumé** dans ≥1 état VERIFY.

### Contraintes stack (pour info — influence le rendu mais ne le dessine pas)
Cible : **React 19 + Tailwind v4 + tokens oklch maison + Base UI** (primitives) + **Motion**.
shadcn = outil build-time **seulement**, **re-thémé oklch avant tout commit** (ne livre
JAMAIS un preset de tokens shadcn/GitHub-dark). L'Operator est **hors CSP scellée** → palette
libre. **Ne PAS** réutiliser l'ancien « design system SBFB » (un test).

### Livrable attendu (handoff repo-visible AVANT le code de fidélité)
1. **`.dc.html` mis à jour** : hi-fi dark, les 4 corrections A appliquées, variantes B/B au
   même niveau de détail, les états honnêtes visibles.
2. **Feuille de tokens** (markdown ou fichier) : **valeurs oklch** (surfaces/texte/bordure/
   5 états), rampe typo (familles/tailles/poids/rôles sans-vs-mono), échelle d'espacement,
   états d'interaction, specs des 5 signatures de motion (durée/easing/déclencheur). C'est
   ce que la **Phase E** (design-system) encodera en `@theme` oklch, et ce dont **C/D/H**
   ont besoin pour coder fidèlement.

## (fin du bloc à coller)

---

### Pour le handoff retour (repo-visible)
- Déposer l'export Claude Design (HTML màj + feuille de tokens) dans `.planning/research/`.
- En extraire pour les phases front : machine d'états du slot VERIFY (libellés exacts), liste
  des intentions/CTA, inventaire d'atomes hi-fi, valeurs de tokens. C'est l'**input de spec**
  que Claude Code re-thème en oklch, câble aux routes réelles, et teste (T1/T2). La maquette
  n'est **jamais** un livrable de build (verrou D2/D5 : ni câblée, ni testée, ni oklch-finale).
