# Projection — l'écosystème SBFB entièrement câblé + apps interconnectées

> **Statut** : recherche hors-sprint / découverte (2026-06-24). Figé sur demande PO.
> **Source** : workflow multi-agent Opus 4.8 1M `sbfb-fully-wired-interconnected-ecosystem`
> (run `wf_623f603e-e58`) — 5 lentilles (flywheel créatif / stack authoring / interconnexion
> app-à-app / organisme entier / sceptique de composabilité) + synthèse. Substrat = inventaire
> des ~2 650 sous-features (`inventaire_features_commits_atomiques.md`).
> **Cadre** : commun numérique **humaniste non-monétaire**, échelle opérateur + quelques bâtisseurs,
> hypothèse « tout câblé, apps interconnectées » prise pour acquise (sans raisonner sur le temps).
> **Voir aussi** : `puissance_noeud_enabler_4sur5.md`, `doctrine_contrat_pour_llm.md`,
> `trajectoire_taille_capacite_llm.md`, `inventaire_features_commits_atomiques.md`,
> `sbfb-wired-vs-open-source-landscape` (run `wf_80c45357-3a6`).

## Verdict : puissance émergente 4 / 5

**Un atelier-souverain vivant** : ni un catalogue d'apps, ni un réseau qui scale — un organisme où
un opérateur distille son goût et sa rigueur dans des **contrats-machine**, et fait émerger des apps
belles-vérifiables-indéplateformables **dont la lignée, la confiance et la nouveauté circulent
attachées au hash, pas à une plateforme.** (Optimisme 4 lentilles à 4/5 pondéré par l'adversariale
à 2/5 : c'est de l'**outillage souverain qui augmente un artisan**, pas un organisme auto-croissant
à effet de réseau.)

## Ce que l'infra *devient* (pas la somme des briques)

```
   ATELIER-MÉMOIRE (le jugement de l'artisan, gravé une fois, exécuté sans lui)
   ┌──────────────────────────────────────────────────────────────────────┐
   │ 1. MÉMOIRE DE MÉTIER  — les erreurs CSP de l'app N deviennent les       │
   │    garde-fous mécaniques de l'app N+1 (le savoir ne fuit pas en têtes)  │
   │ 2. GREFFOIR DE LIGNÉES — chaque app content-adressée = forkable +       │
   │    attribuée + endossable, un graphe de remix signé Ed25519 bout-en-bout│
   │ 3. CLIQUET ANTI-CLICHÉ — chaque app publiée ré-alimente le juge de      │
   │    nouveauté → le seuil pour être jugé "neuf" monte tout seul           │
   │ 4. SERVICE QUI SURVIT — à la mort de l'auteur ET du nœud, dès qu'1 pair │
   │    seede, sans jamais pouvoir exfiltrer (connect-src 'none')            │
   └──────────────────────────────────────────────────────────────────────┘
   = un commun de PROCÉDÉS vérifiables, PAS une collection d'objets.
     Analogie : un atelier d'ébénisterie (gabarits + recettes + carnet d'échecs
     transmissibles, chaque pièce portant sa provenance gravée), pas un app-store.
```

## Les capacités qui *émergent* (et seulement de l'interconnexion)

| Capacité émergente | Émerge de | Mord à petite échelle ? |
|---|---|---|
| **Confiance composable sans autorité** (réputation sur le hash) | provenance Ed25519 + BLAKE3 + RRV + curator-list + Kudos anti-Matthew + Sybil multi-forge ; forker re-signe (« dérivé de X » attestable, « je suis X » non) | **Oui dès N=1** (transitive locale type Tor/Debian-signing) ; partielle (N1-N3 non câblés) |
| **Création souveraine auto-amorcée** (idée→app belle+vérifiable, zéro cloud dans la boucle) | Factory + knowledge-packs (anime.js 93 / daisyUI 68) + novelty-engine + **LLM LOCAL** + gates tracés à `BLOB_SERVE_CSP` | **Oui** (capitalisation, pas effet réseau) ; curation humaine finale non-délégable |
| **Nouveauté mesurée contre un corpus qui grossit** (fonction de coût de l'originalité) | juge 5-dim contre `examples-bank` + `dejavu_corpus` ; chaque app ajoute son fingerprint → cliquet | **Non** (exige volume) ; prouvé sur 2 tours / 1 domaine ; faible hors-distribution |
| **Qualité générative reproductible et *signable*** (SEED = pont créatif↔vérifiable) | RNG seedé + iframe inerte → animation déterministe rejouable bit-pour-bit → entre dans la chaîne provenance | Oui, mais plafonne au domaine web-animé-CSP |
| **Indéplateformabilité vivante** (survit sans pouvoir exfiltrer) | BLAKE3 + CSP `connect-src 'none'` + provenance + iroh-docs + seeding cross-pair | **La seule émergence vraiment solide** (zéro dépendance au volume) |

## Le flywheel — deux bras, pas un

- **Bras SAVOIR (coût marginal décroissant)** : chaque app distille pièges/snippets dans le
  knowledge-pack → l'app N+1 coûte moins cher. **Compose à TOUTE échelle** (capitalisation, pas
  effet réseau).
- **Bras NOUVEAUTÉ (qualité croissante)** : l'app publiée fait monter le seuil de déjà-vu (gen-2
  périme gen-1). **EXIGE volume + curation** — sans injection externe, le corpus raffine *le même
  moteur* (auto-critique projet : 5/12 candidats = « costumes du même moteur »).

> **Honnêteté centrale** : chaque flèche est un **acte humain non-délégable** (claim signé, curation
> finale, re-extraction des packs car `connect-src 'none'` interdit l'auto-fetch). **Ce n'est pas une
> turbine auto-tournante — c'est un artisan qui tourne une manivelle de mieux en mieux huilée.** Le
> nombre de tours/an est borné par son attention.

## Ce que SEUL cet écosystème donne (vs toute composition OSS)

La conjonction, dans **une seule boucle fermée** : reproductible-souverain (F-Droid/Nix) **+**
juge-de-nouveauté adversarial ancré dans un corpus (MusicBrainz/Wikidata rendu *fonction de coût*)
**+** attribution de fork re-signée Ed25519 (Git/IPFS pin mais sans paternité re-signée) **+**
savoir-faire créatif porté à un **LLM local** (pas de point de capture) **+** vérification N0-N3 qui
rend le partage GPU possible entre quasi-inconnus. Wikipedia maille mais ne *note pas* la nouveauté ;
F-Droid attribue mais ne *mesure pas* l'originalité ; Sandstorm sandboxe sans provenance composable
ni stack créative ; v0/bolt.diy génèrent du plausible sans pression de nouveauté ni souveraineté.
**C'est l'intégration *verticale* humaniste — du SEED créatif à la permanence du graphe de pairs —
qui n'existe nulle part comme un seul organisme.** Et **beauté et preuve ne se sabotent jamais** (gate
mécanique déterministe FAIL=blocage *séparé* du goût humain ; autorité process > RRV > Factory).

## Plafond réaliste (honnête)

À l'échelle posée (opérateur + 3-10 bâtisseurs), le flywheel mord **partiellement** :
- **Mord dès N=1** : capitalisation du savoir-machine + indéplateformabilité composée (indépendants
  du volume).
- **Ne mord PAS à petite échelle** : pression de nouveauté auto-entretenue, curation plurielle, tout
  effet de réseau (exigent un volume qui *contredit* l'échelle).
- **Bras mutualisation-calcul = PROVISIONAL** : sharding live RIG-ABSENT, route stub None, WAN
  2-3 tok/s (batch patient, jamais chat réactif), gate audit P0 R-iroh-audit ferme l'usage cross-nœud
  réel ; sans 2-3 pairs vivants, un nœud seul ne mutualise rien ; tissu cross-nœud buggy
  (`SeedAnnounced` peer_count:0 ~10 min, annuaire d'un seeder qui n'annonce pas son catalogue).
- **Goulot irréductible = HUMAIN** : le temps de l'opérateur par bâtisseur, que les capitalisations
  *repoussent* sans annuler.

→ **« augmentation d'un artisan-souverain qui en augmente quelques autres », pas commun planétaire.
Et c'est la valeur assumée, pas le défaut.**

## Caveats durs (le sceptique, retenu)

- **Composabilité-mythe** : storage partitionné par `app_name` (`REPLICATED_APPS = ['sbfb-ideas']`
  hardcodé), bridge à 16 méthodes **toutes node-scoped, aucune n'adresse une app tierce**. La
  composition réelle est **asynchrone et médiée par artefacts hashés/forkés** (graphe d'artefacts
  recomposables), **pas un réseau d'apps qui se téléphonent.**
- **CSP bride par conception** : `connect-src 'none'` + origine opaque = le mur même qui interdit
  l'orchestration live multi-apps et empêche une app de *prouver* une pubkey. **On ne peut avoir la
  sécurité de Tor ET la composabilité de Zapier — SBFB a choisi la sécurité.**
- **Joli ≠ utile** : la pression de nouveauté optimise la *surprise mécanique*, pas la valeur/adoption
  humaine ; juge faible hors-distribution.
- **Couplage-fardeau** : 17 domaines couplés sur un mainteneur = surface ingérable (**risque
  Sandstorm, avertissement direct**).
- **Curation-ne-scale-pas** : généré-puis-curé = travail humain récurrent non-délégable à chaque
  maillon.
- **Substrat** : phare sharding PROVISIONAL, N1-N3 non câblés in-vivo (carry S78), pré-launch
  (R-iroh-audit + R-wasmtime-cve P0 ouverts, pilote fermé). « Tout câblé » résout l'intégration
  technique, jamais le fait sociologique que les gens utilisent des apps isolées.

## Caractère unique

Lignée **Tor / F-Droid / Linux-distro-signing** (souveraineté prouvée à l'exécution, signature-par-
mainteneur, zéro modération centrale) **× Nix / IPFS / Git** (immuabilité content-adressée, fork =
dérivé d'un commit pin) **greffée d'un organe absent de tous** : un moteur de nouveauté reproductible
+ un atelier créatif local-LLM. Sa nature : **un atelier-mémoire, pattern OpenBSD-solo-maintainer** —
neutre par construction (verrous gravés, curation local-only façon Mastodon : un admin ne peut rien
sur le user d'un autre nœud) *tout en* laissant l'humain qui l'opère porter des valeurs fortes, **car
son pouvoir s'arrête à son nœud**. Trait dominant rare : **honnête avec lui-même** (il mesure sa
propre redondance et le dit, il marque sa phare PROVISIONAL *dans le code*) — propriété d'un organisme
de gouvernance, pas d'une démo.

## Conclusion (transversale à la session)

Breadth réelle et profonde (4/5), nouveauté vs OSS sur points précis (3/5), organisme
câblé+interconnecté = **atelier-souverain qui augmente un artisan et son cercle (4/5)** : un **commun
de procédés vérifiables**. La même conclusion revient sous tous les angles — **le seul geste qui
transforme ce potentiel en preuve n'est pas plus de features, c'est une traversée end-to-end éprouvée**
(la boucle qui tourne en vrai, la phare live, N0-N3 câblé). C'est exactement ce qui distinguerait
SBFB de Sandstorm.
