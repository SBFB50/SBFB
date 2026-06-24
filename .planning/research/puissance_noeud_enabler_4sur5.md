# Puissance objective de SBFB à l'échelle « mon nœud + mes bâtisseurs » — 4/5

> **Statut** : recherche hors-sprint / découverte (2026-06-24). Lecture seule sur `.planning/`
> sauf ce doc, figé sur demande PO.
> **Source** : workflow multi-agent Opus 4.8 1M `sbfb-power-my-node-and-builders`
> (run `wf_16dcc31a-901`) — 3 lentilles web-grounded (nœud souverain / enabler-bâtisseur /
> ce-qui-reste-vrai) + synthèse. Vérifié adversarialement.
> **Cadre** : commun numérique **humaniste non-monétaire** (lignée Wikipedia/Tor/Linux), **jamais
> startup** (cf. `feedback_humanist_commons_not_startup` en mémoire). Le « non-monétaire » n'est
> pas un plafond, c'est la valeur.
> **Voir aussi** : `doctrine_contrat_pour_llm.md`, `trajectoire_taille_capacite_llm.md`,
> `vision_communs_idees_factory_builders.md`.

## Recadrage d'échelle (décisif)

On **n'évalue PAS** l'adoption de masse de SBFB (« atteint-il la grand-mère / conquiert-il le
monde / gouverne-t-il un commun planétaire »). **Le protocole est déjà open source, libre, et
n'importe qui peut être dessus — c'est un acquis, pas un objectif.** L'unité d'analyse est **une
personne** : un **opérateur de nœud souverain** qui, en plus, **aide quelques personnes/projets à
bâtir** sur le protocole (un enabler / artisan / intendant local). On mesure la puissance de cette
posture, à échelle humaine et relationnelle.

## Verdict : 4 / 5 (élevée, réelle, déjà dans le code)

À l'échelle d'un opérateur de nœud souverain qui aide 2-5 bâtisseurs, SBFB confère un pouvoir
**réel et déjà codé** — identité inrévocable, apps indéplateformables, données non-exfiltrables,
GPU mutualisé patient, Factory qui capitalise le savoir-faire. Il plafonne **non par le marché,
mais par le temps de l'opérateur** et quelques verrous techniques en cours de levée.

## Schéma — où est la puissance

```
   MOI = opérateur souverain + enabler  (≈ Tor-relay + IndieWeb + CHATONS + homelab,
                                          mais natifs au MÊME protocole, composables par 1 personne)
   ┌──────────────────────────────────────────────────────────────────────┐
   │  POUVOIR DU NŒUD (ce que ça donne, dès N=1, sans personne)      4/5    │
   │   • identité Ed25519 ≠ état civil — personne ne l'attribue,            │
   │     donc personne ne la retire                                         │
   │   • apps INDÉPLATEFORMABLES (BLAKE3 + provenance) qui survivent         │
   │     même à la mort du nœud dès qu'un pair les garde                     │
   │   • données qui ne quittent jamais le hardware (connect-src 'none'      │
   │     = anti-exfiltration par conception : aucune app ne fait d'analytics)│
   │   • IA locale sans GAFAM (Ollama) + mutualisation GPU cross-machine     │
   └───────────────────────────────┬──────────────────────────────────────┘
                                   │  ×  effet multiplicateur
   ┌───────────────────────────────▼──────────────────────────────────────┐
   │  POUVOIR D'ENABLER (lifter 3-10 bâtisseurs)                     4/5    │
   │   le multiplicateur n'est PAS l'adoption, c'est la CAPITALISATION :     │
   │   distiller son expertise UNE fois → les bâtisseurs en héritent à       │
   │   chaque app, SANS l'opérateur :                                        │
   │   • Factory + templates no-build → « monter build+hosting+CI »          │
   │       devient « édite index.html, validate, publish » en 1 session      │
   │   • knowledge-pack + prompt-kind (même vers LLM LOCAL) = le savoir       │
   │       devient un contrat machine ; le build casse si le savoir manque   │
   │   • curator-list Ed25519 = LIFTER leur travail sans être leur admin     │
   │   → 3-10 personnes passent de « GAFAM-dépendantes » à « auteures d'apps  │
   │     insaisissables, machine-fiables, avec compute de frontière différé »│
   └────────────────────────────────────────────────────────────────────────┘
```

## Pouvoir du nœud (4/5) — ce que ça donne à l'opérateur, dès N=1

1. **Identité auto-portée** : clé Ed25519 ≠ identité civile, que personne ne m'attribue donc
   personne ne me retire, vérifiable sans annuaire ni DNS payant.
2. **Apps indéplateformables** : archive content-adressée BLAKE3, iframe sandboxé, provenance
   Ed25519/SLSA L1 — elles survivent même à la mort de mon nœud dès qu'un pair les garde (gain net
   vs Nextcloud/YunoHost dont le service meurt avec le serveur).
3. **Données qui ne quittent jamais le hardware** : iroh-docs CRDT offline-first, loopback-only,
   sandbox `connect-src 'none'` origine opaque = anti-exfiltration **par conception** (aucune app
   hébergée ne peut faire d'analytics).
4. **IA locale sans cloud GAFAM** via ProviderRouter (Ollama local + réseau), avec en plus la
   mutualisation GPU cross-machine.

C'est **strictement plus** que la somme {self-hoster + IndieWeb + CHATONS + relais Tor}, car ces
4 postures sont **natives au même protocole** et **composables par un seul opérateur**.

## Pouvoir d'enabler (4/5) — le multiplicateur

Le multiplicateur n'est **pas** l'adoption mais la **capitalisation** du savoir-faire en assets
réutilisables sans l'opérateur :

- **Factory** (crate `sbfb-factory`, déjà dogfoodé — le projet se construit avec) + 4 templates
  no-build/no-CDN font passer une personne aidée de « monter build+hosting+CI » à « édite
  index.html, validate, publish » en une session.
- **knowledge-pack + prompt-kind** (`--kind app-authoring --provider claude|gpt|local`, avec
  `strip_cloud_references` vers **Ollama local**) transforme l'expertise (ex. 93 primitives
  anime.js annotées CSP-usable) en **contrat machine-consommable**, couplé au disque par un test
  qui casse le build si le savoir manque : on distille une fois, N bâtisseurs héritent du filtre
  dur CSP à chaque app.
- **curator-list Ed25519** = je LIFTE leur travail (découvrabilité + crédibilité par provenance
  attestée) **sans devenir leur admin ni eux le mien**.

Concrètement : 3 à ~10 personnes/projets passent de « incapables ou GAFAM-dépendants » à
« auteurs d'apps insaisissables, machine-fiables, avec compute de frontière différé ». C'est
explicitement **le seul pari de la vision « sans précédent négatif »** (un humain augmenté porte
une app de l'idée à la maintenance) — cadre **solo-augmenté, pas communautaire**.

## Ce qui s'évapore (le bear-case d'adoption de masse, hors-sujet à cette échelle)

- **Syndrome Scuttlebutt / friction grand-public** : sans objet — j'onboarde 2-5 personnes que
  *je* choisis ; la friction devient un coût relationnel ponctuel, pas une barrière structurelle.
- **Paradoxe de la modération globale** : dissous (pas résolu) — curator-lists = je ne curate que
  *ma* vue, je ne sers que les hash que *je* choisis (invariant local-only, cf. Mastodon « un admin
  d'un serveur ne peut rien sur un user d'un autre »). Le problème planétaire n'existe pas à mon
  échelle.
- **Gouvernance d'un commun mondial** : hors-périmètre — aucun DAO, aucun vote planétaire ; je ne
  gouverne que mon nœud.
- **Bus-factor du projet entier** : pas mon fardeau — le protocole est un acquis libre ; si l'amont
  ralentit, mon nœud continue de servir mes blobs et mon GPU.
- **« Il faut une masse critique pour être utile »** : faux côté hébergement (utile dès N=1) et
  Factory+Ollama local ; *partiel* seulement pour la mutualisation GPU/seeding.

## Ce qui reste vrai (contraintes honnêtes — aucune n'est le marché)

| Contrainte | Nature |
|---|---|
| **Temps et énergie de l'opérateur** | Le **vrai goulot**. Être ancre/seeder/mentor pour 3-10 bâtisseurs est un travail soutenu, récurrent, non-délégable. Tous les archétypes (Tor, CHATONS, homelab) convergent : la ressource rare est l'opérateur, pas la techno |
| **Coût d'amorce côté bâtisseur** | Irréductible (CSP `connect-src 'none'`, signer sa provenance, accepter le compute patient) — abaissé par les packs, pas annulé |
| **Maturité technique** | Borne ce qu'on peut *promettre* : sharding/session LIVE = **PROVISIONAL** (route `/shard-session` stub None, `RunProof` sans caller prod hors `#[cfg(test)]`, benchmark cross-machine RIG-ABSENT, carry P1 S78) ; WAN ~2-3 tok/s = batch/async **patient, jamais chat réactif** ; plafond VRAM inchangé |
| **Transport cross-nœud bogué** | `storage_get/set` local au daemon ; convergence `SeedAnnounced` observée à `peer_count:0 ~10 min`, annuaire d'un seeder qui n'annonce pas son catalogue — dette technique réelle |
| **Besoin de ≥2-3 nœuds amis vivants** | Un nœud seul ne mutualise rien (GPU, sharding, redondance seeding, re-pull cross-tier supposent des pairs joignables) |
| **Gate audit P0 (R-iroh-audit)** | Décision gelée « aucun usage réel avant audit externe » → pilote fermé entre nœuds de confiance |
| **Le sandbox est aussi un mur** | `connect-src 'none'` interdit par conception tout canal sortant ; pas encore de méthode bridge « signe ce payload » (une app affiche une pubkey mais ne peut pas la *prouver* — manque N1) |

## La question politique se résout d'elle-même

Au niveau **projet**, un endossement politique capturerait tout le monde (insoluble, source de
schismes façon forks Fediverse). À l'échelle d'**un nœud sur un protocole neutre par
construction**, la question s'évapore : l'engagement est strictement **local**. Ma curator-list,
mes apps hébergées, mes alliances de seeding ne touchent que **ma** vue (même invariant que
Mastodon) : zéro conséquence sur les autres nœuds, **zéro capture du protocole**. La neutralité est
préservée **mécaniquement** (5 verrous anti-recentralisation gravés dans le code, pas une discipline
associative à tenir par charte comme CHATONS), et la liberté d'expression personnelle est **totale
et sans externalité**. « Comment rester neutre tout en ayant des valeurs » devient trivial : le
protocole est neutre, l'opérateur non, et les deux coexistent car son pouvoir s'arrête à son nœud.
Posture identique à l'opérateur Tor / steward IndieWeb.

## Plafond réaliste

Un enabler solo lifte de l'ordre de **3 à ~10 personnes/projets simultanément** (cohérent avec le
pilote fermé actuel). Le goulot est **l'attention par bâtisseur, pas un TAM**. Les capitalisations
(knowledge-packs réutilisables sans l'opérateur, templates, assets hashés/signés) **repoussent** ce
plafond car elles se consomment en son absence ; mais l'amorce de chaque bâtisseur reste un coût
humain, et l'entretien des snapshots datés (re-extraction manuelle, car `connect-src 'none'`
interdit l'auto-fetch) est une charge récurrente.

## Verdict

À l'échelle **opérateur souverain + enabler de 2-5 bâtisseurs**, le pouvoir est **élevé, réel et
déjà cohérent dans le code** — pas une promesse de masse. Le **4/5 et non 5** tient **uniquement**
parce que le bras multiplicateur (mutualisation GPU/sharding live + convergence cross-nœud) n'est
**pas encore garanti par la maturité technique**, et parce que le temps de l'opérateur reste le
goulot irréductible. Tout le reste — souveraineté, insaisissabilité, capitalisation du savoir-faire,
neutralité politique — est **acquis et cohérent dès maintenant**. Ce n'est pas un pari d'adoption :
c'est **l'augmentation d'un artisan-souverain qui en augmente d'autres, sur une infrastructure que
personne ne peut lui retirer**.
