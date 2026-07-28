# Nexus V1 — La fabrique distribuée des communs

**Sous-titre de travail :** Comment des idées deviennent des outils
vérifiables, accessibles et souverains.

**Statut :** édition zéro en cours de rédaction.

**Lecture continue :** [MANUSCRIT.md](MANUSCRIT.md)

Ce dossier contient le livre de référence de la **V1 accomplie** de Nexus. Il
ne décrit ni un sprint, ni l’état instantané du dépôt, ni une promesse de
livraison publique déjà tenue. Il raconte de façon cohérente ce que forme
l’ensemble lorsque les briques imaginées pour la V1 fonctionnent ensemble :
le protocole SBFB, le nœud local, les applications pair-à-pair, RRV, l’Atelier
des communs (Ideas Hub), Conception, Workflow Hub, Factory, les agents, le
calcul partagé, les preuves, l’accessibilité et la gouvernance locale.

Le texte emploie volontairement le présent. C’est le présent d’une
**description d’aboutissement** : il permet au lecteur d’habiter la V1 au lieu
de traverser une roadmap. La note au lecteur maintient la séparation entre
cette vision et la maturité réelle du logiciel au moment de la lecture.

## Ligne éditoriale

Le livre s’adresse d’abord à une personne non spécialiste qui veut comprendre
ce que Nexus change concrètement. Il doit rester assez précis pour qu’un
développeur, un auditeur, une association ou une communauté puisse reconnaître
les vraies frontières du système.

Chaque chapitre suit autant que possible le même mouvement :

1. une situation humaine concrète ;
2. le problème de pouvoir ou de dépendance qu’elle révèle ;
3. la réponse proposée par Nexus ;
4. le fonctionnement nécessaire pour rendre cette réponse crédible ;
5. les limites qui empêchent de transformer la promesse en mythe.

Le livre est humaniste sans être partisan. Il peut décrire l’arrivée massive
de personnes engagées, notamment issues de mouvements politiques, mais le
protocole n’appartient à aucun parti et ne connaît pas l’étiquette politique de
ses utilisateurs. Il accueille des individus, des groupes de projet et des
communautés de pratique ; il ne fabrique pas un « nœud de parti » doté d’une
autorité spéciale.

## Règles de vérité

- Une signature prouve une attribution ou l’intégrité d’un objet, pas la
  vérité de son contenu.
- Une Proof Card explique des signaux de preuve ; elle n’est ni un certificat
  absolu ni une note morale.
- RRV distingue ce qui est lu, déduit, vérifié ou non vérifié ; il ne devient
  pas un oracle.
- Workflow Hub conçoit, versionne et observe. Le shell de confiance présente
  les demandes, résout les secrets et recueille les confirmations. Le
  daemon/runtime Rust recalcule les capacités effectives, compile, exécute,
  journalise et signe les reçus faisant autorité.
- Un modèle peut proposer ou composer ; il ne s’accorde pas de capacités et ne
  s’attribue pas un verdict.
- Factory réalise un travail, orchestre ses vérifications et prépare sa
  fermeture ; les gates, les revues indépendantes et les humains habilités
  gardent le verdict final. Conception spécifie ce qui doit être construit.
  L’Atelier des communs fait mûrir ce qui mérite éventuellement de le devenir.
- Un workflow peut évoluer entre deux exécutions. Le plan compilé d’une
  exécution reste figé pendant celle-ci.
- Les données privées ne deviennent jamais publiques par simple proximité avec
  le réseau. Toute lecture, réplication ou publication dépend d’une capacité et
  d’une intention explicites.
- Il n’existe ni classement mondial de vérité, ni administrateur universel, ni
  droit politique acheté avec du matériel ou des kudos.
- L’accessibilité est un mode premier du produit, pas une couche ajoutée après
  coup.

## Architecture du livre

### Partie I — Reprendre prise

- [Note au lecteur](00_note_au_lecteur.md)
- [Chapitre 1 — Le monde numérique sous location](01_le_monde_sous_location.md)
- [Chapitre 2 — La thèse Nexus](02_la_these_nexus.md)
- [Chapitre 3 — Une journée dans la V1](03_une_journee_dans_nexus.md)

### Partie II — Faire mûrir le possible

- [Chapitre 4 — Ideas Hub, l’Atelier des communs](04_atelier_des_communs.md)
- [Chapitre 5 — RRV, chercher sans fabriquer la vérité](05_rrv.md)
- [Chapitre 6 — Conception, décider avant de construire](06_conception.md)

### Partie III — Transformer une intention en action

- [Chapitre 7 — Workflow Hub, rendre l’organisation programmable](07_workflow_hub.md)
- [Chapitre 8 — Factory, fabriquer avec des preuves](08_factory.md)
- [Chapitre 9 — Le nœud, les applications et le calcul partagé](09_reseau_apps_compute.md)

### Partie IV — La confiance sans trône

- [Chapitre 10 — Prouver, autoriser, consentir](10_confiance_et_capacites.md)
- [Chapitre 11 — Gouverner sans capturer](11_gouvernance_sans_capture.md)
- [Chapitre 12 — L’accessibilité comme architecture](12_accessibilite.md)
- [Chapitre 13 — Ce que Nexus refuse](13_ce_que_nexus_refuse.md)

### Partie V — Une infrastructure humaniste en société

- [Chapitre 14 — Des projets qui augmentent la capacité d’agir](14_projets_humanistes.md)
- [Chapitre 15 — L’épreuve du nombre : accueillir une masse engagée sans livrer le protocole](15_masse_engagee.md)
- [Chapitre 16 — De la difficulté locale au commun réutilisable](16_scenario_de_commun.md)

### Conclusion et annexes

- [Épilogue — Une puissance d’agir partagée](17_epilogue.md)
- [Glossaire](90_glossaire.md)
- [Carte de la V1](91_carte_de_la_v1.md)
- [Carte des sources internes](SOURCES_INTERNES.md)

## Choix de vocabulaire

Dans ce livre, **Nexus** désigne l’écosystème complet vécu par les personnes.
**SBFB** désigne le protocole et les primitives qui rendent possibles la
distribution, la réplication, les capacités, les preuves et le calcul partagé.
Cette distinction évite de réduire Nexus à un protocole réseau ou, à l’inverse,
de confondre une application particulière avec l’infrastructure commune.

Le nom **Ideas Hub** reste utile pour l’application d’entrée. Le livre emploie
surtout **Atelier des communs** pour désigner le processus plus large qui
clarifie, documente, critique et fait mûrir une idée.

## Vers une édition publiée

Cette édition est modulaire afin que chaque chapitre puisse être relu,
réorganisé et sourcé sans fragiliser l’ensemble. Une fois le fond stabilisé,
les fichiers pourront être assemblés, puis mis en page en DOCX, PDF ou EPUB.

`MANUSCRIT.md` est généré depuis les chapitres sources. Après une modification,
le reconstruire depuis PowerShell avec :

```powershell
.\assemble.ps1
```

Ne pas corriger directement le manuscrit assemblé : la prochaine génération
écraserait ces changements.

La future édition publiée devra ajouter au minimum :

- un nom d’auteur et une courte préface personnelle ;
- une révision de style complète ;
- des schémas illustrés ;
- des notes et références externes vérifiées ;
- une relecture technique, une relecture accessibilité et une relecture
  contradictoire ;
- une mention datée séparant la vision V1 de l’état public réellement
  démontré.
