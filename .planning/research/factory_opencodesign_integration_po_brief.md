# Open CoDesign × Factory — Note de décision (Product Owner)

> **Pour qui** : décision produit (PO) — pas besoin de lire du code pour comprendre.
> **Statut** : recherche / aide à la décision. Rien n'est construit, rien n'est engagé.
> **Détail technique** : voir `factory_opencodesign_design_integration_study.md` (mêmes conclusions, niveau code).
> **Date** : 2026-06-22

---

## En une phrase

**On ne « reprend » pas Open CoDesign. On en récupère 3 petites pièces (gratuites, sous licence permissive), on refait nous-mêmes ses bonnes idées, et on garde 100 % de la sécurité chez nous.** Le partenariat « plugin » avec eux est une bonne intention, mais **pas faisable aujourd'hui** — on le garde en option pour plus tard.

---

## 1. Le problème produit

SBFB veut que **n'importe qui puisse créer une app et la publier** sur le réseau. Pour ça, il manque un **étage « design / maquette »** dans Factory : un endroit où l'on passe d'une idée à une première interface visible, avant d'écrire le code.

Open CoDesign est un outil open-source qui fait exactement ça : **« décris ton app → tu obtiens une maquette en quelques secondes »**, avec l'IA de ton choix (y compris une IA locale). La question : **qu'est-ce qu'on lui prend, et comment ?**

---

## 2. Ce qu'Open CoDesign nous offre vraiment

Open CoDesign n'est pas « une seule chose ». C'est **deux choses de natures très différentes** :

```
 ┌────────────────────────────────────┐     ┌────────────────────────────────────┐
 │  LE MOTEUR  (l'application entière) │     │  DES PIÈCES DÉTACHÉES  (utilitaires)│
 │                                     │     │                                     │
 │  • une grosse app de bureau         │     │  • le « trieur » (HTML ou React ?)  │
 │  • dépend d'un kit IA d'un seul     │     │  • « l'emballeur » (rend une page   │
 │    auteur + d'un navigateur caché   │     │     autonome, sans internet)        │
 │  • tire des polices/scripts du      │     │  • le « contrôleur » (détecte les   │
 │    réseau (contraire à nos règles)  │     │     erreurs de base)                │
 │                                     │     │                                     │
 │   →  ON NE PREND PAS                │     │   →  ON PREND  (petites, stables,   │
 │      (lourd, pas souverain, fragile)│     │      licence permissive MIT)        │
 └────────────────────────────────────┘     └────────────────────────────────────┘
```

> **Analogie.** C'est comme un architecte qui te livre (a) son **logiciel de conception** complet, et (b) quelques **outils à main** (un gabarit, un mètre, un niveau). On ne rachète pas son logiciel — on prend les outils à main et on les range dans **notre** atelier.

Point capital : ce qui **évolue vite** chez eux (la qualité de génération, les styles) est dans **le moteur** — qu'on ne prend pas. Les 3 pièces qu'on prend sont **stables** : elles ne « bougent » quasiment jamais, donc on ne rate aucune mise à jour qui compte.

---

## 3. Notre choix : prendre 3 pièces, refaire les idées

Il y avait 4 façons d'intégrer Open CoDesign, du plus risqué au plus sain :

```
   À ÉVITER ◄──────────────────────────────────────────────────► RECOMMANDÉ

   A. ADOPTER         B. PRENDRE          C. REFAIRE LES        D. FORKER
      l'app entière      3 pièces            IDÉES nativement      leur dépôt
                         (MIT, patchées)     (sur notre socle)

   ❌  importe toute   ✅  utilitaires     ✅  presque gratuit:  ❌  pire des
       la dette            stables, sous       nos briques font      deux mondes:
       (app lourde,        licence libre,      déjà 80% du travail   leur dette +
       pas souverain)      petit périmètre                           un fork à
                                                                     maintenir seul
```

**Notre stratégie = C (socle) + B (2 pièces ciblées, corrigées).** A et D rapportent peu et coûtent cher.

---

## 4. Comment ça s'intègre — vu produit

La bonne nouvelle : Factory a **déjà** presque tout. L'étage design vient se **brancher** dessus, pas créer un tuyau parallèle.

```
  1.  L'utilisateur décrit / ajuste son app          ┌─ assisté par une IA 100% LOCALE
      (boutons d'intention, pas de jargon)  ─────────┤  (Ollama, gratuit, sans cloud,
                                                      └─  jamais le kit IA d'Open CoDesign)
                          │
                          ▼
  2.  Le design est écrit DANS le dossier source de l'app
      (sa "charte" + sa maquette de référence vivent avec le code)
                          │
                          ▼
  3.  PREUVE AUTOMATIQUE  ◄── tout ce qui est dans le source est haché + signé.
      Le design hérite GRATUITEMENT de la "preuve d'origine" de l'app.
                          │
                          ▼
  4.  AUTO-VÉRIFICATION : on rejoue l'app dans le bac à sable RÉEL de production.
      Si elle dépend d'internet ou casse → ça échoue ICI, pas chez l'utilisateur.
                          │
                          ▼
  5.  PUBLICATION : passe les contrôles de sécurité déjà existants → diffusée sur le réseau.
```

> **L'aubaine la plus importante (point 3)** : chez SBFB, *« ce qui est dans le dossier source est automatiquement scellé et signé »*. Donc **le design devient vérifiable sans effort** — c'est le cœur de notre promesse « source vérifiable », appliqué gratuitement au design.

---

## 5. Le design system à deux couches

Une contrainte de sécurité forte façonne tout : **une app publiée n'a pas le droit d'aller chercher quoi que ce soit sur internet** (0 CDN, 0 police Google, 0 fetch). Conséquence directe :

```
   COUCHE 1 — L'INTERFACE SBFB (le "shell")        COUCHE 2 — CHAQUE APP PUBLIÉE
   (notre application, chez nous)                  (dans un bac à sable scellé, isolé)

   peut POINTER vers une charte partagée           doit EMBARQUER sa charte (une COPIE)
   (couleurs, typo) — c'est gratuit                car elle n'a pas le droit d'aller
                                                   chercher la charte ailleurs

        ╲                                                       ╱
         ╲────────── UNE SEULE SOURCE de couleurs/typo ────────╱
                    (les "tokens"), compilée vers les deux
                  →  cohérence garantie, zéro dépendance externe
```

En clair : **on partage une source de design à la fabrication, jamais à l'exécution.** Chaque app emporte sa propre copie. Pas de point de panne, pas de dépendance réseau.

---

## 6. Ce qu'on gagne / ce que ça coûte / ce qu'on ne délègue jamais

| | |
|---|---|
| **On gagne** | Un étage « idée → maquette → app » crédible, **accessible aux non-développeurs** ; une IA locale gratuite ; le design **vérifiable par construction** ; cohérence visuelle des apps. |
| **Ça coûte** | Récupérer + **corriger** 2 petites pièces (retirer leurs polices Google et leur dépendance CDN) ; refaire quelques idées en natif. Petit périmètre, sous notre contrôle. |
| **On ne délègue JAMAIS** | La **sécurité** : le bac à sable, la signature, les contrôles de publication restent **100 % chez nous**. Aucun artefact n'est dispensé de vérification « parce qu'il vient d'Open CoDesign ». |

---

## 7. L'ordre de marche (séquencement)

Chaque brique ne s'ouvre que quand la précédente est verte. **Ce n'est pas un planning daté**, c'est l'ordre logique.

```
  ①  Design dans le source         →  ②  Copilote IA local      →  ③  Modèle d'app
      (+ preuve gratuite)               (Ollama)                     "design"
                                                                        │
  ⑦  Affichage + preuve   ◄──  ⑥  Auto-vérification  ◄──  ⑤  Rendu  ◄──  ④  Récupérer les
      (galerie de maquettes)       (dans le bac à sable)      en bac      2 pièces (corrigées)
                                                              à sable
```

Le point d'attention : **④ avant ⑤** — il faut corriger les pièces (supprimer le réseau) *avant* d'afficher, sinon une page qui dépend d'internet casse à l'écran au lieu d'échouer proprement à la fabrication.

---

## 8. Les décisions à prendre (PO)

| Décision | Le choix, en clair | Recommandation |
|---|---|---|
| **Outil visuel embarqué ?** | Mettre un éditeur "façon Figma" dans Factory maintenant ? | **Non / plus tard** — ça apporte juste une interface, pas une capacité ; coût de maintenance pour un solo. |
| **Polices** | Garder leurs polices Google (réseau) ou les embarquer ? | **Embarquer** — règle « 0 internet dans les apps ». |
| **Ressource externe oubliée** | Que faire si une maquette pointe vers internet ? | **Refuser à la fabrication** (échouer tôt et lisiblement, pas à l'écran de l'utilisateur). |
| **Vérification de l'app** | Comment rejouer l'app pour la tester ? | **Dans notre bac à sable réel** (pas leur navigateur caché) — on teste sous les vraies règles de prod. |
| **Copilote IA** | Quelle IA pour assister le design ? | **Ollama local** (gratuit, hors-ligne) — jamais leur kit IA. |
| **Maquette obligatoire avant code ?** | Bloquer la publication si pas de maquette ? | **Garde-fou de processus**, pas un blocage technique de plus (un seuil esthétique n'est pas une règle de sécurité). |
| **Premier geste vers eux** | Comment être "allié" sans risque ? | **Publier une fiche de style "goût SBFB"** (voir §10) — zéro engagement, annulable. |

---

## 9. Risques & incertitudes (honnêtes)

- **Pari sur un projet jeune** : Open CoDesign est en v0.2, mené par **une seule personne**, pas encore signé. → On ne dépend de rien chez eux : on **copie le comportement sous revue**, on ne les ajoute pas à nos dépendances. **Bouton d'arrêt** garanti.
- **Pas de "prise plugin"** : ils n'ont **aucun système de plugin** aujourd'hui, et il n'arrivera **pas avant leur v1.0**. Un partenariat technique dépendrait de leur calendrier, qu'on ne maîtrise pas.
- **Ce qu'on n'a pas encore confirmé** (à vérifier avant de coder) : le coût exact de séparer leur IA de leur agent ; le détail de leur vérificateur ; si certaines maquettes embarquent des appels réseau cachés. → Le bac à sable réel reste notre filet de sécurité.

---

## 10. Le prochain pas le moins risqué

**Publier une fiche « goût SBFB »** (un simple fichier d'instructions de style : « 0 internet, polices embarquées, structure compatible »).

- Coût : quasi nul. Risque : nul (aucun engagement, annulable).
- Effet : un premier signe de coopération concret avec Open CoDesign, **sans** dépendre d'eux ni toucher à notre sécurité.

Si tu valides, je peux **rédiger ce fichier** ou détailler n'importe laquelle des 7 briques du §7.

---

*Note de décision PO — dérivée de l'étude technique `factory_opencodesign_design_integration_study.md`. Read-only ; rien n'est engagé ; sprint 77 non touché. 2026-06-22.*
