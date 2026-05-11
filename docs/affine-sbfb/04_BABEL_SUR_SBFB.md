# Babel sur SBFB

## Positionnement

Babel doit etre une app vitrine SBFB, mais pas seulement une app de lecture.
Le vrai objectif est une infrastructure libre de lecture:

- corpus;
- bibliotheque;
- traduction;
- provenance contributive;
- validation humaine benevole;
- lecture offline;
- synchronisation;
- Babel Shelf;
- liseuses libres ou reconditionnees.

## Flux Babel v1

```text
Sources domaine public ou licence compatible
  -> source-policy gate
     (domaine public/licence, juridiction, redistribution P2P,
      traduisibilite, attribution, takedown/opt-out)
  -> manifest source signe
  -> archive/app Babel
  -> distribution SBFB
  -> lecture dans le shell
  -> cache offline local
```

## Flux Babel compute

```text
Texte source
  -> task traduction/indexation
  -> workers consentants (GPU / NLLB / pipeline traduction)
  -> drafts machine signes
  -> validateurs automatiques
  -> corrections et revues humaines benevoles
  -> consensus signe par les pairs/noeuds participants
  -> corpus Babel enrichi
```

## Registre Babel

Chaque texte Babel doit garder un graphe de provenance complet:

```text
source
  -> manifest droits/provenance
  -> chunks
  -> draft LLM
  -> validations automatiques
  -> corrections humaines
  -> revues native-speaker
  -> consensus
  -> traduction publiee
```

Les noeuds ne contribuent pas seulement de la puissance LLM. Ils peuvent etre
workers de traduction, validateurs automatiques, traducteurs/correcteurs
humains, reviewers native-speaker, temoins de consensus ou replicateurs de
corpus. Kudos doit pouvoir distinguer ces roles.

Gutenberg reste le corpus de demarrage, pas une exception au droit. BnF/Gallica,
Wikisource ou Internet Archive deviennent possibles si leur source-policy prouve
la redistribuabilite et la traduisibilite du document precis.

## Flux Babel Shelf

```text
Babel Shelf
  -> bibliotheque locale
  -> selection de textes
  -> export liseuse
  -> lecture offline
```

## Liseuses

Strategie coherente avec SBFB:

1. court terme: export libre vers appareils existants sans GAFAM quand possible;
2. moyen terme: Babel Shelf local + sync USB/Wi-Fi local;
3. long terme: firmware ou liseuse ouverte Babel.

Ne pas faire de Kindle/Amazon la strategie centrale. Ces chemins peuvent servir
de comparaison, pas de dependance.

## Questions a garder visibles

- Comment recevoir un texte Babel sans compte proprietaire?
- Comment lire offline sans DRM?
- Comment verifier qu'un texte est legalement distribuable?
- Comment prouver qu'une traduction est redistribuable et traduisible?
- Comment rendre visible le travail humain de validation/correction?
- Comment synchroniser une liseuse simple avec un noeud local?
- Comment garder la gouvernance non capturable?
