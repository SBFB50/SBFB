# Carte de la V1

Cette annexe résume les responsabilités. Elle ne remplace pas les chapitres :
son but est d’empêcher qu’une fonction soit attribuée à la mauvaise couche.

## Les composants

| Composant | Rôle | Reçoit principalement | Produit principalement | Ne possède pas |
|---|---|---|---|---|
| Nœud local | Centre de gravité de la personne | préférences, clés, apps, données et intentions | vue locale, stockage, décisions et connexions autorisées | autorité sur les autres nœuds |
| Shell de confiance | Interface privilégiée | demandes de l’app et du runtime | présentation des effets, résolution de secrets, grants et confirmations humaines recueillis | autorité finale d’exécution ou logique métier des packs |
| Protocole SBFB | Distribution et échanges entre pairs | blobs, annonces, événements et tâches signés | disponibilité, réplication choisie et transport | vérité globale, classement unique ou propriété des apps |
| Atelier des communs | Maturation des besoins | graine, contexte, sources, objections et portage | `ResearchPack` puis `DecisionPack` | décision automatique de construire |
| RRV | Recherche et explication proof-first | artefacts publics, projections, sources et espaces autorisés | résultats sourcés, Proof Cards et actions bornées | accès implicite aux bases privées ou verdict final |
| Conception | Décision produit et technique | `DecisionPack`, recherche et contraintes | `ConceptionPack`, ADR, critères et risques | fabrication silencieuse du code |
| Workflow Hub | Auteur et opérateur visible | intentions, modèles, agents et artefacts | workflows, versions, évaluations et demandes de run | secrets, clé privée ou autorité d’exécution |
| Daemon et runtime Rust | Autorité d’un run | identité et version authentifiées, workflow validé, demandes, politiques, grants et signaux | capacités effectives recalculées, `CompiledPlan`, événements, checkpoints, artefacts et reçus signés | droit d’étendre seul son plan |
| Factory | Fabrication et préparation de la fermeture | brief, `DecisionPack`, `ConceptionPack`, repo et composants | code, tests, revues, diff, release candidate et preuves | verdict final ou droit de publier seule |
| Workers | Calcul volontaire borné | tâches et entrées minimales autorisées | résultats signés et reçus | accès au plan privé complet ou à la gouvernance |
| Curateurs | Médiation choisie | objets, chartes, preuves et expérience | recommandations, mises en garde et collections | capacité de supprimer un objet pour tous |
| Babel | Traduction distribuée vérifiable | corpus autorisés, chunks, modèles et revues | traductions, attestations et graphe de provenance | dispense de vérifier les droits ou la qualité humaine |

## La chaîne d’artefacts

```text
Graine de besoin
  ↓ clarification, incubation, recherche et contradiction
ResearchPack
  ↓ arbitrage humain
DecisionPack
  ↓ analyses parallèles et choix d’architecture
ConceptionPack
  ↓ auteur de workflow et validation
Workflow + agents + évaluations
  ↓ compilation Rust
CompiledPlan
  ↓ exécution durable
RunEvent + ArtifactRef + receipts
  ↓ Factory, gates et décision humaine
release + provenance + Proof Card
  ↓ projection publique
RRV, forks, usages et nouveaux besoins
```

Chaque flèche constitue une frontière. Un texte libre peut accompagner
l’artefact, mais la machine ne doit pas deviner silencieusement l’autorité
qu’il transporte.

## Les quatre espaces de données

| Espace | Exemples | Accès | Usage RRV |
|---|---|---|---|
| Privé local | secrets, préférences, brouillons, historique personnel | personne ou politique locale | aucun par défaut |
| Partagé borné | corpus de groupe, état applicatif, artefacts internes | capacité explicite de lecture ou d’écriture | seulement si une capacité dédiée l’autorise |
| Publié | apps, packs, documents, manifests, releases | adresse de contenu et règles de distribution | indexable selon les données publiées |
| Projection publique | version, licence, digest, provenance, résultat expurgé | lecture publique explicitement prévue | surface principale de découverte |

Une donnée ne passe pas automatiquement de gauche à droite. La réplication
pair-à-pair d’un espace partagé ne le transforme pas en espace public.

## L’autorité en une phrase par couche

- Le modèle **propose**.
- Workflow Hub **exprime et versionne**.
- Le compilateur Rust **valide les invariants et fige le plan**.
- Le shell **présente les demandes, résout les secrets et recueille grants et
  confirmations humaines**.
- Le daemon et le runtime **authentifient, recalculent les capacités
  effectives, exécutent, journalisent et signent les reçus faisant autorité**.
- L’humain **arbitre et engage les effets sensibles**.
- RRV **retrouve, qualifie et explique**.
- Factory **réalise, orchestre les vérifications et prépare la fermeture**.
- Les gates, les revues indépendantes et les humains habilités **décident du
  verdict final**.
- SBFB **transporte et distribue**.
- Les curateurs **recommandent sans gouverner tous les autres**.

## Ce que la V1 ne centralise pas

La V1 n’introduit pas :

- de classement mondial de vérité ;
- de marketplace obligatoire ;
- de mise à jour silencieuse ;
- de gouvernance par token ou par volume de GPU ;
- de base universelle contenant toutes les données ;
- de scheduler mondial capable de déplacer librement un run privé ;
- de mémoire personnelle répliquée publiquement par défaut ;
- de plugin importé autorisé à exécuter du code arbitraire ;
- d’agent qui s’accorde des outils, un budget ou un verdict ;
- de publication sensible sans consentement final.

Ces absences ne sont pas toutes des retards. Plusieurs sont les conditions qui
permettent aux composants puissants de rester compatibles avec la souveraineté
locale.
