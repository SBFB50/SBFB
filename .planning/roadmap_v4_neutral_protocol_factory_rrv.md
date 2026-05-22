# Roadmap v4 — Protocole Neutre + Factory/RRV

## Date

2026-05-19

## Statut

CANON — couche strategique uniquement. Les plans detailles de
chaque sprint sont generes par le kickoff au moment de l'execution.

## Source de verite detaillee

`.planning/research/SYNTHESIS_factory_rrv_protocol.md` — document
canon de 2000 lignes consolidant 14 fichiers de recherche. Contient
l'architecture, les decisions, les schemas, les tests, les briques
OSS, la securite. Le kickoff de chaque sprint le lit comme input
strategique.

---

## Vision PO

Decision PO 2026-05-13 :

> Release publiquement defendable. Credibilite, durabilite,
> reutilisabilite. Commun logiciel anti-capture AGPL-3.0.

Pivot PO 2026-05-18 (Factory-first + Babel canari) :

> Factory est l'atelier generique. Babel est le premier dogfood.
> RRV observe et prouve ce que Factory produit.

Pivot PO 2026-05-19 (protocole neutre + co-dev) :

> Le daemon est un tuyau stupide. Factory sort du daemon et
> devient un outil client externe. RRV @dev co-evolue avec
> Factory. @protocole d'abord (les donnees existent, c'est le
> differenciateur), puis @dev booste par le protocole, puis @web.

Recadrage PO 2026-05-21 (@dev non bloquant Gate 1) :

> L'Arc 2 ne depend pas de RRV @dev pour reussir. Gate 1 se valide
> sur la chaine @protocole : create/publish, feed, search, Proof Card,
> Babel dogfood, pilote ferme. @dev est un enrichissement post-pilote
> par defaut (S70+) ou un stretch strictement non bloquant. L'ingestion
> de repos OSS GitHub exige un contrat source-only separe et ne doit pas
> etre confondue avec une app SBFB verifiee.

---

## Boucle fondamentale

Arc 2 valide d'abord la boucle courte, deja supportee par les donnees
du daemon :

```
Factory cree ou valide une app
  -> App deployee sur le reseau
  -> Feed + browse + provenance enregistrent les faits
  -> RRV @protocole recherche l'app et genere une Proof Card
  -> Pilote verifie que la preuve est lisible et utile
  -> retour bugs/fixes
```

La boucle longue `@dev`/OSS reste la vision post-pilote :

```
RRV @dev indexe des sources locales ou source-only
  -> Factory reutilise les patterns et citations
  -> App ou librairie publiee avec labels de confiance separes
  -> RRV croise preuves protocole + preuves code
  -> retour
```

---

## Decisions gelees (D1-D16)

Ref detaillee : SYNTHESIS §9.1.

| # | Decision | Date |
|---|----------|------|
| D1 | FTS5 d'abord, Tantivy gate post-S75 | 2026-05-18 |
| D2 | Factory hors daemon (crate sbfb-factory) | 2026-05-19 |
| D3 | node_id retire de SBFB.json (Option D) | 2026-05-18 |
| D4 | Feed raw-op extensible (pas de bump par op) | 2026-05-18 |
| D5 | Babel = premier dogfood Factory | 2026-05-18 |
| D6 | @protocole avant @dev avant @web | 2026-05-19 |
| D7 | SBFB.json v2 (schema_version: 2) | 2026-05-18 |
| D8 | Gates Factory FG0-FG10 | 2026-05-18 |
| D9 | CuratorVouched minimal | 2026-05-18 |
| D10 | S66 OBLIGATOIRE avant pilote | 2026-05-18 |
| D11 | Score completude de preuve (pas trust score) | 2026-05-18 |
| D12 | Babel canari = reader + fixtures (pas reviews) | 2026-05-18 |
| D13 | Preview ephemere via daemon API neutre | 2026-05-19 |
| D14 | Pas de signature decomposee S67-S69 | 2026-05-19 |
| D15 | SearchManifest domain tag gele une fois deploye | 2026-05-19 |
| D16 | formula_version dans ProofCard | 2026-05-19 |
| D17 | S70 = consolidation Gate 1, pas SearchManifest. Phases elargies 2-3 × ~1200 LOC | 2026-05-22 |

---

## Architecture cible

```
Daemon SBFB (neutre)
  Primitives existantes : blobs, feed, gossip, deploy, provenance,
    browse, curators, storage, blob-serve
  Nouvelles primitives neutres :
    GET /api/daemon/feed/entries
    POST /api/v1/preview/load
    GET /api/daemon/search (FTS5)
    GET /api/daemon/proof-card/{id}
    CuratorVouched/CuratorDisendorsed (feed ops)
    SearchManifest (wire format + gossip)
  Crate partage :
    sbfb-manifest (validation SBFB.json v2)

sbfb-factory + RRV @dev (outil client externe)
  RRV @dev : index code/AST/symbols, proof cards enrichies,
    citations fichier/ligne/hash, scan risques
  Factory : templates, generation, diff, publish gate, audit log
  Produit : Babel, apps tierces

Apps (iframe sandbox)
  Babel Reader, sbfb-search, Protocol Explorer, Ideas Hub
  Bridge : search, proof_card_get, storage, provenance
```

Ref detaillee : SYNTHESIS §2 + §3.

---

## Arcs et gates

### Arc 1 — Fondations Defendables (S65-S66)

| Sprint | Objectif | Statut |
|--------|----------|--------|
| S65 | Contrat public : vocabulaire exact, taxonomie 6 niveaux, feed raw-op | **DONE** |
| S66 | Durabilite : persistence iroh, feed republish, provenance 3 etats | **EN COURS** |

**Gate entree Arc 2 :** Le daemon survit aux restarts. Apps, feed,
provenance persistent. E2E restart test vert. Sans ca, rien de ce
qui suit n'a de sens.

---

### Arc 2 — Factory + RRV @protocole + Canari (S67-S69)

**Objectif :** Construire Factory comme outil client externe,
livrer la chaine RRV @protocole necessaire au pilote, valider avec
Babel comme premier dogfood cree via Factory, confronter a des
testeurs reels. RRV @dev n'est pas une condition de sortie Arc 2.

**Livrables de sortie de l'arc :**
- `sbfb-factory` CLI fonctionnel (create + preview + publish)
- `sbfb-manifest` crate partage
- FTS5 daemon search operationnel
- Proof Cards computees et affichees
- CuratorVouched/Disendorsed dans le feed
- Babel Reader deploye via Factory
- Pilote ferme 2-3 personnes, daemon stable 24h
- RRV @protocole trouve Babel (`search?q=babel`)

**Themes par sprint (l'ordre exact des phases est determine au
kickoff de chaque sprint) :**

| Sprint | Theme | Entree requise |
|--------|-------|----------------|
| S67 | Primitives daemon neutres + @protocole FTS5 + sbfb-factory MVP | S66 DONE |
| S68 | Proof Cards + publish gate + UX confiance | S67 FTS5 + sbfb-manifest |
| S69 | Babel dogfood via Factory + pilote ferme + RRV @protocole prouve Babel | S68 Proof Cards + publish path |

**Ce que le kickoff de chaque sprint decide :**
- Les phases A-D/E exactes et leur contenu
- Le scope @dev eventuel (stretch non bloquant ; S70+ par defaut)
- Les livrables deplacables si le sprint deborde (scope cuts)
- Les items dette absorbes

**Scope ajustable entre S67-S69 :**

| Livrable | Obligatoire | Deplacable a S70+ si debordement |
|----------|-------------|----------------------------------|
| Primitives daemon (feed/entries, search, preview) | Oui | Non |
| sbfb-factory CLI (create + generate) | Oui | Non |
| Proof Cards | Oui | Non |
| Babel Reader + deploy | Oui | Non |
| Pilote ferme | Oui | Non |
| Page React /factory | Non | Oui — CLI suffit pour S69 |
| @dev index dans sbfb-factory | Non | Oui — @protocole suffit pour Gate 1 |
| Diff engine avance | Non | Oui — publish direct suffit |
| 3eme template (react-vite) | Non | Oui — 2 suffisent |

Ref detaillee : SYNTHESIS §3, §4, §5, §11.

---

### Gate 1 — Go/no-go Arc 3

**Quand :** Apres S69.

**Conditions go :**

| Critere | Go | No-Go |
|---------|-----|--------|
| Installation | 2/3 testeurs installent sans aide | 0/3 reussit |
| Connexion P2P | 2 noeuds se voient en < 5 min | Aucune connexion apres 15 min |
| Deploy app | 1 testeur deploie depuis source | Deploy echoue |
| Babel via Factory | Babel creee avec Factory, deployee, visible Browse | Factory echoue |
| Feed sync | Feed synchronise entre 2+ noeuds | Divergence ou corruption |
| Restart | Daemon redemarrage propre | State corrompu |
| Stabilite 24h | Daemon tourne 24h sans crash | Crash, OOM, freeze |
| RRV trouve Babel | `search?q=babel` retourne Babel | Search vide |
| Proof Card | Proof Card Babel affichee | Proof Card absente |

**Decision iroh 0.98 vs 1.0 :** Evaluee ici. Si le pilote revele
des bugs fixes uniquement en iroh 1.0, l'upgrade devient prioritaire.

Si > 5 bugs P0/P1 : sprint fix dedie avant S70.

---

### Sprint consolidation S70 (D17, amendement 2026-05-22)

**Objectif :** Rendre l'Arc 2 défendable avant d'ajouter du réseau
P2P. Aucune feature nouvelle — uniquement stabilisation.

**Axes (chaque item doit référencer un carry, bug pilote, ou test
manquant — zéro item spéculatif) :**

1. **Audit Gate 1 réel** — rejouer install, publish, Babel,
   ProofCard, restart, feed sync, browse, search. Pas seulement
   tests unitaires.
2. **Refacto coutures** — Factory/daemon API, bridge method policy,
   ProofCard data validation, publish path, preview TTL, Browse/
   Proof UI. Pas de refacto esthétique.
3. **Dette sécurité** — fermer P2-D-1 wiring, P2-D-2 Zod runtime,
   P2-D-3 XSS ProofCard, manifest vs bridge allowlist.
4. **Dette produit** — clarifier états : draft, preview, published,
   verified, stale, source-only.
5. **Tests E2E** — reload, restart, deux noeuds, app generated-by-
   Factory, proof visible, storage cohérent.
6. **Docs/roadmap sync** — roadmap v4, audit_plan, verification,
   threat model, publish model.

**Phases élargies** : 2-3 phases à ~1200-1500 LOC code au lieu de
4-5 à ~600 LOC. Le coût process par phase est quasi-fixe (~1000
lignes d'artefacts preflight+review+codex). Moins de phases = ratio
code/process de ~45-50% au lieu de ~29%.

**Gate de sortie S70** : "un utilisateur externe installe, crée via
Factory, publie, cherche, vérifie une Proof Card, redémarre le
daemon — tout fonctionne."

---

### Arc 3 — Reseau Verifiable + Industrialisation (S71-S73)

**Objectif :** Etendre au reseau P2P (SearchManifest opt-in),
formaliser la gouvernance, durcir Factory.

**Livrables de sortie de l'arc :**
- SearchManifest wire format + gossip + discovery
- Gouvernance formelle (CuratorVouched UI, timeline, dissent)
- Factory hardening (tests adversariaux, securite broker)
- Optionnel : Babel translation beta si sprint reserve disponible

**Themes par sprint :**

| Sprint | Theme | Entree requise |
|--------|-------|----------------|
| S70 | **Consolidation Gate 1** — dette Arc 2, refacto coutures, tests E2E, docs sync. Phases elargies (2-3 × ~1200 LOC). Zero feature nouvelle. | Gate 1 PASS |
| S71 | SearchManifest opt-in + discovery P2P | S70 consolidation DONE |
| S72 | Gouvernance complete + Factory hardening | S71 SearchManifest |
| S73 | Sprint reserve (fixes pilote / Babel translation / dette) | S72 |

---

### Gate 2 — Go/no-go Arc 4

**Quand :** Apres S73.

**Conditions go :**
- SearchManifest fonctionne opt-in entre 3 noeuds
- Gouvernance documentee et verifiable dans le feed
- Aucun bug P0 ouvert
- Babel trouvable via RRV avec Proof Card complete

---

### Arc 4 — Pack Produit (S73-S75)

**Objectif :** Assembler SBFB + Factory + Babel + RRV en
demonstration defendable. Proof pack verifiable hors connexion.
Decision go/no-go release publique.

**Themes par sprint :**

| Sprint | Theme | Entree requise |
|--------|-------|----------------|
| S73 | Bridge avance + domain packs + 2eme app Factory | Gate 2 PASS |
| S74 | Proof pack structure + CI attestations + SBOM | S73 |
| S75 | Release narrative + decision go/no-go public | S74 |

---

## Graphe de dependances

```
S65 Contrat Public (DONE)
  |---> S67 (vocabulaire + raw-op prerequis)

S66 Durabilite (EN COURS)
  |---> S67 (persistence prerequis FTS5 + pilote)

S67 Primitives + Factory + @protocole
  |---> S68 (FTS5 + sbfb-manifest prerequis Proof Cards)
  |---> S69 (Factory prerequis Babel)

S68 Proof Cards + publish gate
  |---> S69 (proof card + publish path prerequis pilote)

S69 Babel + pilote (Gate 1)
  |---> S70 (consolidation dette Arc 2 avant reseau)

S70 Consolidation Gate 1
  |---> S71 (base stable pour SearchManifest)

S71 SearchManifest
  |---> S72 (manifests enrichis par gouvernance)

S72 Gouvernance + hardening
  |---> S73 reserve

S73 reserve (Gate 2)
  |---> S74-S76 (pack produit)
```

**Dependances cachees :**

| ID | Dependance | Impact |
|----|-----------|--------|
| H1 | S65 auth tier -> S66 persistence | Ops non-autorisees persistees si inversees |
| H2 | S66 -> S71 | SearchManifests doivent survivre aux restarts |
| H3 | iroh 1.0-rc -> Gate 1 | Decision point upgrade |
| H4 | sbfb-manifest -> deploy.rs + sbfb-factory | Crate partage, schema v2 affecte les deux |
| H5 | S67 FTS5 -> S68 proof-card | Proof cards utilisent l'index FTS5 |
| H6 | sbfb-factory independant daemon | JAMAIS importer nexus-shell-daemon-core |

---

## Gates Factory FG0-FG10

Criteres de qualite cumulatifs dans `sbfb-factory`, implantes
progressivement. Ref detaillee : SYNTHESIS §7.2.

| Gate | Nom | Implante en |
|------|-----|-------------|
| FG0 | Classification app | S67 |
| FG1 | Scope | S67 |
| FG2 | Template | S67 |
| FG3 | Manifest | S67 |
| FG4 | Diff | S68+ |
| FG5 | Sandbox | S68+ |
| FG6 | Secrets/deps | S68+ |
| FG7 | Preview | S68+ |
| FG8 | Provenance | S69 |
| FG9 | Publish | S69 |
| FG10 | Review | S69 |

---

## Ordonnancement RRV

| Scope | Quand | Pourquoi |
|-------|-------|----------|
| **@protocole** | S67 | Les donnees existent. Differenciateur SBFB. Besoin pilote. |
| **@dev** (booste par protocole) | S70+ par defaut ; stretch S68-S69 seulement si zero impact Gate 1 | Depend de sbfb-factory + @protocole. N'est pas requis pour le pilote ferme. |
| **@web** | Post-pilote S73+ | Depend de Factory fonctionnelle pour consommer les resultats. |

Ref detaillee : SYNTHESIS §4.2 + rrv_scope_ordering_analysis.md.

---

## Decision 2026-05-21 — @dev, Babel et seed OSS

1. **Gate 1 ne teste pas @dev.** Les criteres decisifs sont
   l'installation, la connexion P2P, le deploy, le feed sync, le
   restart, la stabilite, la recherche `@protocole`, et la Proof Card.
2. **Babel est un dogfood utilisateur.** FlowUP cree Babel avec
   `sbfb-factory` et le protocole ; Claude/Codex maintiennent les
   primitives, templates, publish path, preuves, et bugs remontes.
3. **Les gros repos OSS GitHub ne sont pas des apps SBFB.** Le
   `deploy-from-repo` actuel exige une app avec `SBFB.json` et
   `index.html`. Un repo source generique requiert un mode
   `source-only`/`source-index` separe.
4. **Le seed OSS est S70+ experimental.** Il doit etre curatee
   (petit corpus pertinent), borne (taille/fichiers/langages), et
   etiquete `external OSS source index`, jamais `verified SBFB app`.
5. **Les labels de confiance restent separes.** GitHub sert a la
   decouverte et au commit hash ; la verification SBFB reste reservee
   aux artefacts publies via le protocole.

---

## Risques strategiques

| ID | Risque | Impact | Mitigation |
|----|--------|--------|------------|
| R1 | S66 prend plus de temps que prevu | Retarde tout | Pas de raccourci. S66 est non-negociable. |
| R2 | sbfb-factory deborde sur 2 sprints | Pilote retarde | Scope cuts : /factory UI et @dev deplacables |
| R3 | iroh 0.98 bugs en pilote | Pilote instable | Decision upgrade a Gate 1 |
| R4 | Testeurs pilote pas disponibles | Pas de feedback | Backup : FlowUP + 1 machine secondaire |
| R5 | FTS5 insuffisant a l'echelle | Recherche mediocre | Tantivy en gate post-S75 |
| R6 | Factory couplage daemon | Neutralite violee | Test acceptance #20 : sbfb-factory ne depend PAS de nexus-shell-daemon-core |

---

## Comment le process s'adapte

Le cycle sprint existant gere deja l'adaptation :

```
Audit gate (sprint N-1)
  -> Kickoff sprint N (lit: etat code + carries + SYNTHESIS + cette roadmap)
  -> Plan detaille (phases A-D, genere par le kickoff)
  -> G8 preflight (verifie chaque phase vs code reel)
  -> Execution (phases + review)
  -> Verification + audit_plan
  -> Audit gate (sprint N)
  -> Kickoff sprint N+1 (s'adapte)
```

**Ce que le kickoff lit comme input strategique :**
1. Cette roadmap (arcs, gates, dependances, decisions gelees)
2. La synthese SYNTHESIS_factory_rrv_protocol.md (architecture, schemas, tests)
3. L'etat reel du code (git log, tests, carries)
4. La memory nexus_grid_pivot.md (compteurs, tip)

**Ce que le kickoff decide tactiquement :**
- Les D1..D5 specifiques au sprint
- Les phases A-D/E et leur contenu exact
- Les scope cuts du sprint
- Le delta tests estime

Pas de phases detaillees pre-planifiees au-dela du sprint en cours.
La roadmap fournit la direction. Le kickoff fournit le plan.

---

## Calendrier indicatif (pas un engagement)

| Sprint | Estimation | Theme |
|--------|-----------|-------|
| S66 | 1-2 semaines | Durabilite (en cours) |
| S67 | 1-2 semaines | Primitives + @protocole + Factory CLI |
| S68 | 1-2 semaines | Proof Cards + publish gate |
| S69 | 2-3 semaines | Babel + pilote ferme |
| Gate 1 | — | Go/no-go |
| S70 | 1-2 semaines | Consolidation Gate 1 (dette + refacto + E2E) |
| S71-S73 | 3-6 semaines | SearchManifest + gouvernance + reserve |
| Gate 2 | — | Go/no-go |
| S74-S76 | 3-6 semaines | Pack produit defendable |

Horizon total : ~15-25 semaines (ajustable). Chaque sprint
s'adapte a la velocite reelle observee.
