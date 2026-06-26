# Brief Claude Design — Wireframes du front Factory Operator (SBFB, S80)

> **But.** Prompt prêt-à-coller dans **Claude Design** pour produire les **wireframes**
> (low-fi : IA, layouts, flux, états — pas la hi-fi finale) de la refonte **greenfield**
> du front Factory Operator. Grounded dans `factory_front_greenfield_blueprint.md` (base
> retenue S80) + corrections PO 2026-06-26. Niveau **wireframe = stack-agnostique** : les
> détails stack (shadcn vs Base UI, lib motion) et la hi-fi se tranchent au kickoff S80.
> Handoff attendu en retour : export/maquettes **repo-visible** avant la 1re phase front.

---

## À COLLER DANS CLAUDE DESIGN (à partir d'ici)

Tu vas produire les **wireframes low-fidelity** d'un outil de dev local. Lis tout le
cadrage avant de dessiner. Ce n'est PAS une app grand public ni un SaaS — c'est un
établi souverain, sobre, lignée OpenBSD / F-Droid / Tor. « Poussé » veut dire
**raffiné et dense**, jamais flashy/décoratif.

### Ce qu'est l'outil
**Factory Operator** = outil de dev **local privilégié** tournant sur le nœud d'un
mainteneur solo. Il parle à une API loopback Rust (token) et pilote un **agent de
codage** (Claude Code / Ollama / réseau) qui fabrique des apps web « à source
vérifiable ». L'humain **ne tape quasiment pas de code** : il donne des **intentions**,
l'agent fait le travail, l'humain **vérifie**. Mono-utilisateur (le user = le dev).

### Le paradigme (à respecter strictement) : « établi bi-focal », PAS un IDE
Une seule fenêtre = **un rail d'orientation ambiant permanent** + **une scène
mono-focale** qui ne montre qu'**UN de deux MODES à la fois** :
- **STEER** — exprimer une intention → observer/relancer l'agent.
- **VERIFY** — examiner le diff → lire les gates → aperçu scellé → preuve.

La bascule suit le travail (par défaut STEER ; on passe en VERIFY quand un diff/gate est
prêt). **Jamais d'auto-bascule en plein stream.** Exactement une chose focale, étiquetée.
Pas de tri-panneaux co-égaux, pas de board multi-agents, pas d'éditeur de code au centre.

### Écrans à wireframer (par priorité)
1. **Shell + rail d'orientation** (toujours visible, n'entre jamais dans une transition) :
   `Sprint 80 · Phase X ▸ branche ▸ ● N modifiés · M indexés ▸ ◑ gates 4✓ 1•`. + le
   sélecteur de MODE (STEER / VERIFY).
2. **MODE STEER** :
   - **Composeur d'intention** : presets en clair (« Préparer la phase », « Vérifier avant
     validation », « Transmettre à un autre agent ») + champ langage naturel. Le jargon
     technique (`kind`, `provider`, `preflight`, hash du pack) est **REPLIÉ** dans un
     « ▸ détails techniques ». Le provider (Claude/Ollama/réseau) est un **attribut discret**,
     pas une UI séparée.
   - **Atelier observable** : flux de l'agent en streaming + **cartes tool-call / diff**
     (chaque édition de l'agent = une carte « fichier +X −Y », bouton « voir le diff »).
   - **Le MUR** : si l'intention est sensible (commit / push / shell / valider), afficher
     une **carte-MUR** expliquant « exige une vraie session agent + gates + preuves » avec
     **une seule** action « Préparer le pack pour la session ». **Jamais** de bouton
     « Forcer » / « Override » / « Exécuter quand même ».
3. **MODE VERIFY** (l'investissement central) — 3 artefacts co-localisés + onglets :
   - **Diff** par fichier / hunk (la vérité = `git diff`, pas un buffer).
   - **Gates** (diagnostic) : chaque gate = une ligne avec son état **distinct et visible**
     (`✓ passed` / `• en attente` / `BLOQUANT` / `2 issues` / `— non exécuté`). États
     **jamais aplatis** en vert/rouge binaire. Un slot ÉTAT global qui ne dit **JAMAIS
     “PASS”** (au mieux « En attente de session agent — 4✓ 1• · 0 verdict auto-clos »).
   - **Aperçu scellé** : un iframe (rendu réel de l'app produite). Le marquer « rendu
     scellé ».
   - **Preuve** : carte de provenance = chaîne `commit → archive_hash → signataire` en
     **monospace copiable**, décomposée en couches (base / provenance / oss / curateurs /
     licence) + risques. **Décomposition, jamais une coche “Vérifié”.**
   - Actions de hunk = **intentions** routées à la session (« Transmettre la correction »,
     « Signaler ce hunk »), **jamais** « Approve » / « Merge » / « Commit ».
4. **Terminal PTY** (xterm) : sous-surface « steering profond » pour l'expert (volet/onglet).
5. (Secondaires, à esquisser légèrement) palette d'actions + journal ; inspecteur
   « knowledge » consultatif ; brouillon non-autoritaire.

### Invariants UX NON NÉGOCIABLES (doivent transparaître dans les wireframes)
- **Intentions, pas jargon** : tous les CTA en langage clair ; le jargon est replié.
- **Consommée jamais autoritaire** : l'UI ne calcule **aucun** verdict ; elle ne peut
  **jamais** afficher « PASS ✓ ». Les surfaces de connaissance/IA sont marquées
  « consultatif — non autoritaire » (bordure pointillée, contraste réduit, **jamais de
  coche verte**).
- **Le MUR de gouvernance** = une vraie barrière visuelle, pas un bouton. (C'est l'avance
  de SBFB sur Cursor/Windsurf — on l'expose, on ne la dilue pas.)
- **Décomposition, jamais verdict** : la preuve = la chaîne + les couches + les manques.

### Langage visuel (niveau wireframe)
- **Achromatique par défaut** ; la couleur n'apparaît que pour porter un **signal d'état**
  (ok / warn / bad / info / neutre=absence). Dark.
- **Dualité typo = le langage** : **sans = intention / prose humaine** ; **mono = preuve /
  artefact machine** (tout hash, diff, sha, nom de gate, provenance, terminal en mono).
- **Calme et dense** : séparation par la **ligne**, pas par l'ombre SaaS. Composeur aéré /
  surface de vérif dense et tabulaire.
- États honnêtes = citoyens de 1re classe : `PROVISIONAL`, `Not evidenced`, `non exécuté`,
  `RIG-ABSENT` doivent avoir un traitement visuel **assumé**, pas caché.
- **Motion** (note pour la hi-fi, pas le wireframe) : motion = sens, jamais déco ;
  `prefers-reduced-motion` = état final instantané. Une vraie lib (Motion ou anime.js)
  servira le registre « poussé » plus tard.

### Contraintes stack (pour info — ne pas dessiner)
Cible technique : React 19 + Tailwind v4 ; couche composants = shadcn **ou** Base UI
(à trancher). **Ne PAS réutiliser** l'ancien « design system SBFB » (c'était un test).
L'Operator est **hors CSP** (outil local) → palette libre.

### Livrable attendu
Wireframes low-fi des **4 groupes d'écrans** ci-dessus + un **inventaire écrans/composants**
+ le **flux de bascule bi-focal** (quand/comment on passe STEER ⇄ VERIFY). Tous les CTA
**libellés en intentions françaises**. Montre les **états** (vide / en cours / diff prêt /
mur / gate en attente), pas seulement l'écran « plein ».

## (fin du bloc à coller)

---

### Pour le handoff retour (repo-visible, avant impl front)
- Déposer l'export Claude Design (lien + captures/HTML) dans `.planning/` (S80 actif) ;
- en extraire : inventaire d'écrans, inventaire de composants, machine d'états du slot
  VERIFY, et la liste des intentions/CTA — c'est ce que les phases front consomment.

---

## Cadrage des wireframes — décisions (réponses à l'intake skill, 2026-06-26)

1. **Registre de rendu** — **PAS** le croquis hand-drawn « napkin » (connote
   brouillon joueur = faux signal). **Low-fi grayscale sobre, rectilinéaire** : lignes
   nettes, vraie hiérarchie typo, **monospace** partout où vivent hash / diff / gate /
   sha. Séparation par la **ligne**, 0 ombre. Le wireframe respire déjà l'austérité sans
   figer les tokens hi-fi.
2. **Explorations vs états** — **États d'abord.** La doctrine d'honnêteté vit dans les
   états (vide / en cours / diff prêt / MUR / gate en attente / Not evidenced / non
   exécuté), pas dans une N-ième disposition. Variantes de disposition **limitées** aux
   2 écrans vraiment ouverts (STEER, VERIFY).
3. **Variantes par écran clé** — **2** pour STEER, **2** pour VERIFY. (1 = lock
   prématuré ; 4 = dilution solo + doctrine déjà contraignante.) Tout le reste =
   1 disposition résolue + ses états.
4. **Shell + sélecteur MODE** — **Orientation = barre fine permanente en HAUT** (J1 :
   sprint · phase · branche · dirty/staged · pouls gates ; n'anime jamais). **Bascule +
   nav profonde = rail vertical étroit à GAUCHE** : les 2 items STEER / VERIFY en tête
   (groupés, séparés visuellement), puis sous-surfaces secondaires (terminal, sessions,
   historique, knowledge) dessous. (Conforme aux maquettes doc 3.)
5. **MODE VERIFY** — **Colonne change-set à gauche** (fichiers + marqueur gate par
   fichier) + **panneau principal à onglets [Diff | Aperçu scellé | Preuve]** (Diff par
   défaut) + **bande GATES permanente en bas** (diagnostic, pleine largeur) avec le slot
   **ÉTAT**. **Gates + ÉTAT ne sont JAMAIS derrière un onglet** — toujours visibles
   (ils portent le « 0 verdict auto-clos »).
6. **MUR de gouvernance** — **Barrière en-flux pleine largeur** qui **interrompt** le
   composeur / stream (pas un toast, pas un modal cliquable-au-travers). Traitement
   « mur » : bordure lourde / hachurée, encart en retrait. Texte « exige une vraie
   session agent + gates + preuves repo — l'Operator ne peut pas l'exécuter seul ».
   **Une seule** action « Préparer le pack pour la session ». **Zéro** Forcer / Override /
   Bypass. Le flux s'arrête physiquement au mur.
7. **Terminal PTY** — **Tiroir bas escamotable** (replié par défaut), invocable depuis
   le rail gauche, **extensible** jusqu'à quasi-plein-écran. Il **surimpose** le mode
   courant sans remplacer le rail. Jamais un panneau co-égal permanent.
8. **Inventaire + flux** — (a) **planche inventaire** = grille de vignettes encadrées
   étiquetées (écran + job/route servie) + **légende de composants** atomiques (rail,
   composeur, carte tool-call, ligne de gate, bande MUR, carte preuve, tiroir terminal)
   montrés une fois ; (b) **diagramme d'états** de la bascule : `STEER (défaut) → [fin de
   tour agent ET diff/gate frais] → VERIFY → [nouvelle intention] → STEER`, annoté
   « jamais d'auto-bascule en plein stream ».
9. **Autre à cadrer** —
   - **Prioriser** STEER + VERIFY + le MUR (le reste en vignettes).
   - **Pièges à éviter (explicites)** : pas d'IDE (pas d'arbre de fichiers central, pas
     d'éditeur-au-centre, pas de multi-pane co-égal) ; pas de dashboard SaaS (pas de
     tuiles KPI, pas de graphes, pas d'ombres partout) ; **aucun** bouton Approve /
     Merge / Commit / PASS ; **aucune** coche verte « Vérifié » ; pas d'auto-bascule.
   - **Refs visuelles (lignée)** : OpenBSD (sobre, dense-texte, monospace, 0 ornement),
     F-Droid « making reproducible builds visible » (montrer le processus, pas un badge),
     Tor Browser (calme, achromatique), registre Sigstore / Rekor (chaîne append-only en
     mono).
   - **États honnêtes obligatoires** dans ≥1 état VERIFY : Not evidenced / non exécuté /
     PROVISIONAL / RIG-ABSENT visibles, jamais cachés.

### Variantes retenues (validées PO 2026-06-26) — wireframes importés
Wireframes importés dans `.planning/research/wireframes_factory_operator/` (`Factory
Operator - wireframes.dc.html` + `support.js`, canvas Claude Design). 2 variantes par
écran central proposées ; **arbitrage PO = B / B** :
- **STEER → Variante B** (atelier observable dominant, composeur replié en **dock
  invocable**). Raison : flux observe-lourd (compose 1×, regarde longtemps) + « une
  chose focale ». **Mitigation** : état vide (pas de session) = composeur en grand, pas
  en dock (découvrabilité du démarrage).
- **VERIFY → Variante B** (liste d'artefacts à gauche, **ÉTAT toujours visible**).
  Raison : co-localiser la décomposition (diff + gates + état + preuve) = honnêteté
  `0 verdict auto-clos` non masquable ; cohérent avec la décision Q5. **Mitigation** :
  colonne d'artefacts **repliable** (repliée = diff plein), bande gates/ÉTAT en bas
  persistante.
À **verrouiller formellement au preflight du kickoff S80** (avec le stack : shadcn vs
Base UI, lib motion Motion/anime). Ce sont des dispositions, pas des Day-0.
