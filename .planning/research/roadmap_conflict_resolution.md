# Resolution des conflits Roadmap V2 vs Pivot Factory/Babel

**Date :** 2026-05-18
**Documents analyses :**
- **Doc A** : `.planning/roadmap_v2_public_trust_rrv_factory.md` (1667 lignes)
- **Doc B** : `.planning/research/s65_s75_factory_babel_canary_research.md` (851 lignes)
- **Ref 1** : `.planning/research/feed_version_bump_strategy.md` (Option E)
- **Ref 2** : `.planning/research/tantivy_vs_fts5_decision.md` (FTS5 recommande)
- **Ref 3** : `.planning/research/factory_deploy_constraint_research.md` (Option D node_id)

---

## Conflit 1 : Sequencage des arcs S65-S75

### Delta sprint par sprint

| Sprint | Doc A (roadmap canon) | Doc B (pivot Factory/Babel) | Delta |
|--------|----------------------|---------------------------|-------|
| **S65** | Contrat Public (taxonomie, badges, wording, carry items securite) | Contrat Public + contrat Factory/Babel (idem + definir manifest v2, artefacts sprint app, gates G0-G10) | Doc B elargit S65 avec les specs Factory |
| **S66** | Durabilite (iroh data_dir, FsStore, feed republish, RevocationCache, test restart) | Durabilite avant app canari (idem, identique sur le fond) | Quasi-identique |
| **S67** | **Gouvernance de Confiance** (CuratorVouched, CuratorDisendorsed, multi-curator, dissent, freshness, stale detection) | **Factory Foundation / Sprint OS** (template, SBFB.json v2, broker minimal, factory.template.lock, factory.provenance.json, factory.audit.jsonl) | **CONFLIT MAJEUR** : contenu completement different |
| **S68** | **Proof Pack** (SBOM, feed snapshot, canary, verify.sh, attestation CI) | **Broker, preview, publish gate** (UI /factory, diff preview, sandbox, scan secrets, path traversal, deploy integration) | **CONFLIT MAJEUR** : contenu completement different |
| **S69** | **Pilote Ferme** (install, invite, feedback, go/no-go, 2-3 testeurs) | **Babel Reader canari ferme** (domain pack, app generee par Factory, fixtures, pilote integre) | **CONFLIT PARTIEL** : Doc B absorbe le pilote dans le canari Babel |
| **S70** | RRV LocalOnly (Tantivy index, search API, bridge method, app sbfb-search) | RRV LocalOnly sur corpus reel (FTS5 d'abord, index apps Factory + Babel) | Moteur different (Tantivy vs FTS5), corpus enrichi |
| **S71** | RRV Proof Cards (score completude, risk factors, ProofCard UI) | Proof Cards (idem + Babel Proof Card + Factory artifact card) | Quasi-identique, Doc B ajoute des cibles concretes |
| **S72** | SearchManifest Opt-In (manifest signe, gossip, discovery P2P) | SearchManifest opt-in (idem) | Identique |
| **S73** | **Code Factory Templates** (SBFB.json v2, CLI sbfb create, 3-4 templates) | **Factory hardening / templates** (templates additionnels, deuxieme app, migration v1/v2, docs) | Doc A = premiere creation Factory. Doc B = hardening d'une Factory deja livree en S67-S68 |
| **S74** | **Code Factory Broker/Sandbox** (broker architecture, diff, review UI, preview, publish gate) | **Babel translation beta** (task_submit traduction, mock/worker, reviews, fallback fixtures) | **CONFLIT MAJEUR** : Doc A = broker (deja livre en S68 selon Doc B). Doc B = Babel enrichi |
| **S75** | **Babel Dogfood / Domain Packs** (Babel Reader, fixtures, storage P2P, deploy verifie, domain pack format) | **Pack produit defendable** (evidence pack final, Babel+Factory proof cards, RRV, release narrative, go/no-go public) | Doc A = premiere creation Babel. Doc B = consolidation d'un Babel deja livre en S69 |

### Analyse

Doc B a raison sur le fond : la sequence "RRV complet puis Factory puis Babel" de Doc A repousse le premier vrai dogfood applicatif trop tard. Le risque central -- "Factory sait-elle produire une app reelle ?" -- est invisible jusqu'a S73-S75 dans Doc A.

La logique de Doc B est plus solide : livrer Factory tot (S67-S68), forcer le dogfood avec Babel canari (S69), puis laisser RRV indexer du contenu reel (S70+) au lieu d'indexer un catalogue vide.

### Resolution

**Adopter la sequence Doc B pour S67-S75.** La gouvernance (CuratorVouched) de Doc A S67 est redistribuee (voir Conflit 2). Le proof pack de Doc A S68 est partiellement absorbe par Doc B S68 et partiellement reporte (voir Conflit 3).

**Sequence finale :**

| Sprint | Contenu |
|--------|---------|
| S65 | Contrat Public + specs Factory/Babel (Doc A + Doc B) |
| S66 | Durabilite (Doc A = Doc B, identique) |
| S67 | Factory Foundation (Doc B) + CuratorVouched/Disendorsed minimal (ex-Doc A S67 Phase A) |
| S68 | Broker, preview, publish gate, deploy integration (Doc B) |
| S69 | Babel Reader canari ferme = pilote (Doc B, absorbe Doc A S69) |
| S70 | RRV LocalOnly sur corpus reel, FTS5 (Doc B) |
| S71 | Proof Cards (Doc A enrichi par Doc B) |
| S72 | SearchManifest opt-in (identique) |
| S73 | Factory hardening + templates additionnels (Doc B) |
| S74 | Babel translation beta (Doc B) + proof pack allege (ex-Doc A S68 elements restants) |
| S75 | Pack produit defendable + go/no-go public (Doc B) |

---

## Conflit 2 : S67 -- Gouvernance vs Factory Foundation

### Delta

- **Doc A S67** : CuratorVouched + CuratorDisendorsed dans le feed, multi-curator trust overlay avec scope, UX confiance visible (badges, timeline, dissent), stale detection, tests adversariaux. 4 phases. Feed v1->v2 bump.
- **Doc B S67** : Factory Foundation / Sprint OS. Module broker minimal, template app statique, SBFB.json v2, retrait node_id, factory.template.lock, factory.provenance.json, factory.audit.jsonl, sprint skeleton genere, validation manifest/bridge methods.

### Question critique : La gouvernance disparait-elle ?

Non, mais elle est redimensionnee et redistribuee.

**Ce qui DOIT etre en S67 :**
- `CuratorVouched` et `CuratorDisendorsed` comme nouvelles ops feed (c'est le prerequis pour que la gouvernance soit lisible). C'est une Phase A dans le S67 Factory.
- Le feed doit gerer les nouvelles ops AVANT le canari S69 ou des curators verront Babel.

**Ce qui PEUT etre deporte a S71/S73 :**
- Multi-curator trust overlay avec scope dans le Browse (complexe, pas bloquant pour le canari)
- UX confiance visible (dissent timeline, freshness badges) -- enrichit les Proof Cards S71
- Stale detection automatique (timer coordinator) -- hardening post-pilote

**Ce qui est dans Factory Foundation S67 :**
- Tout ce que Doc B liste (module broker, template, SBFB.json v2, etc.)

### Resolution

**S67 = Factory Foundation + CuratorVouched minimal.**

```
S67 Phase A — SBFB.json v2 + retrait node_id + CuratorVouched/Disendorsed ops feed
  - Definir SBFB.json v2 (cf. factory_deploy_constraint_research.md Option D)
  - Retirer contrainte node_id dans deploy.rs
  - Ajouter CuratorVouched + CuratorDisendorsed au enum PublicFeedOperation
  - Feed version strategy : serde_json::Value (pas de bump, cf. Option E)
  - Tests : serde roundtrip, verify_chain avec 4 types d'ops, adversarial forgery

S67 Phase B — Module broker Factory minimal
  - factory_broker dans nexus-shell-daemon-core
  - Routes /api/v1/factory/templates, /api/v1/factory/create
  - Path allowlist + canonicalize
  - factory.template.lock + factory.provenance.json + factory.audit.jsonl

S67 Phase C — Template app statique + CLI
  - Template static-minimal
  - sbfb create ou route equivalente
  - Sprint skeleton genere
  - Validation manifest/bridge methods

S67 Phase D — Tests + non-regression
  - App generee deployable via deploy-from-repo
  - Explorer/Ideas restent compatibles (SBFB.json v1 parse OK)
  - Aucune ecriture hors workspace
```

**Gouvernance UX (ex-Doc A S67 Phases B-D) deplacee :**
- Multi-curator overlay, scope, dissent → S71 (enrichit Proof Cards)
- Stale detection timer → S73 (hardening Factory)
- Freshness badges UX → S71 Phase C

---

## Conflit 3 : S68 -- Proof Pack vs Broker/Preview/Publish

### Delta

- **Doc A S68** : Proof Pack (SBOM CycloneDX, feed snapshot, canary refresh, verify.sh, attestation GitHub, CLI sbfb proof-pack generate/verify). 4 phases.
- **Doc B S68** : Broker enrichi (UI /factory, diff preview, apply approuve, preview sandboxee, scan secrets, path traversal deny, publish-check, proof pack Factory, integration deploy-from-repo, insertion ReleasePublished si cable).

### Question : Le proof pack disparait-il ?

Non, mais il est scinde. Doc B S68 contient deja "proof pack Factory" comme livrable, et "integration deploy-from-repo" qui est le coeur du proof pack.

**Elements du proof pack Doc A qui sont absorbes par Doc B S68 :**
- `factory.provenance.json` (deja dans S67)
- `factory.audit.jsonl` (deja dans S67)
- Deploy roundtrip E2E (Doc B S68 "integration deploy-from-repo")
- Feed entry `ReleasePublished` automatique (Doc B S68 "insertion si cable")

**Elements du proof pack Doc A qui restent a placer :**
- SBOM CycloneDX 1.6 (cargo-sbom dans CI)
- CANARY.txt refresh (warrant canary)
- verify.sh (script verification autonome)
- Attestation GitHub (actions/attest-build-provenance)
- Feed snapshot export
- CLI `sbfb proof-pack generate/verify`

### Resolution

**Les elements SBOM/canary/verify.sh sont des artefacts de release, pas des prerequis pour le canari Babel.** Ils sont mieux places en S74 ou S75 quand on assemble le "pack produit defendable".

```
S68 = Broker, preview, publish gate (Doc B)
  - ABSORBE : P2-PROVENANCE-404-BRIDGE, P2-COVERAGE-DEPLOY-E2E (ex-Doc A S68)
  - ABSORBE : proof pack Factory leger (factory.provenance.json, audit.jsonl)
  - ABSORBE : deploy roundtrip E2E avec feed ReleasePublished

S74 Phase D (ou S75 Phase A) = Proof Pack Release allege
  - SBOM CycloneDX 1.6
  - CANARY.txt refresh
  - verify.sh
  - Attestation GitHub
  - Feed snapshot export
  - CLI sbfb proof-pack generate/verify (si pertinent)
```

---

## Conflit 4 : S69 -- Pilote Ferme vs Babel Reader Canari

### Delta

- **Doc A S69** : Pilote ferme generique. 5 phases : checklist prereqs + invite, installeur cross-platform, feedback collector integre (Ideas Hub), scenarios de test guides (8 scenarios), analyse go/no-go. Criteres : 2/3 testeurs installent, 24h sans crash, feed sync.
- **Doc B S69** : Babel Reader canari ferme. Domain pack Babel, app generee par Factory, fixtures multilingues, source manifests, storage local, reviews minimales, provenance visible, Browse visible, pilote ferme integre.

### Question : Le pilote est-il inclus dans le Babel canari ?

**Oui.** Doc B S69 est un sur-ensemble de Doc A S69. Le canari Babel EST le vehicule du pilote. Les 2-3 testeurs utilisent Babel Reader comme scenario concret au lieu de scenarios generiques.

**Ce que Doc A S69 apporte en plus et qu'il faut garder :**
- Mecanisme d'invite (endpoint HTTP + ticket feed) : necessaire meme pour Babel
- Test installeur cross-platform sur VM : necessaire
- Bouton "Exporter les logs" : necessaire
- Scenarios de test structures : adaptes a Babel au lieu de generiques
- Criteres go/no-go formels : necessaires
- Fix P2-VERIFY-LOCAL-KEY-ONLY : necessaire avant exposition externe
- Re-ecriture Playwright (P2-PLAYWRIGHT-SPECS-STALE partie 2)

### Resolution

**S69 = Babel Reader canari ferme (Doc B) qui absorbe le pilote (Doc A).**

```
S69 Phase A — Domain pack Babel + mecanisme invite
  - Domain pack Babel (fixtures 3 textes, ~5 langues)
  - App babel-reader generee par Factory (pas code a la main)
  - Endpoint invite (ex-Doc A Phase A)
  - Fix P2-VERIFY-LOCAL-KEY-ONLY (ex-Doc A Phase A)

S69 Phase B — Babel app fonctionnelle
  - UI reader : liste textes, lecteur, toggle langue
  - Storage : progression, bookmarks (bridge storage_get/set)
  - Identity : affichage pubkey (bridge identity_pubkey)
  - Provenance visible, feed cursor visible

S69 Phase C — Installeur + pilote
  - Test installeur cross-platform sur VM (ex-Doc A Phase B)
  - Guide testeur adapte a Babel
  - Bouton "Exporter les logs" (ex-Doc A Phase C)
  - Deploy Babel via Factory → deploy-from-repo → Browse → open

S69 Phase D — Scenarios test + feedback
  - 8 scenarios Babel (install, join, browse, deploy Babel, read text, 
    switch langue, storage sync, restart 24h)
  - Re-ecriture Playwright (P2-PLAYWRIGHT-SPECS-STALE partie 2)
  - Feedback via Ideas Hub

S69 Phase E — Go/no-go
  - Criteres go/no-go de Doc A preserves integralement
  - Critere additionnel : "Babel n'est pas code a la main hors Factory"
  - Decision iroh 0.98 vs 1.0 evaluee ici (ex-Gate 1 Doc A)
```

**Les 22 tests d'acceptance de Doc B section 12 COMPLETENT les criteres de Doc A.** Ils ne les remplacent pas. Le gate S69 combine :
- Les 7 criteres go/no-go de Doc A (installation, lancement, P2P, deploy, feed sync, restart, stabilite 24h)
- Les 22 tests bloquants de Doc B (deploy sans node_id, manifest invalide refuse, bridge inconnue refuse, path traversal, etc.)

---

## Conflit 5 : Feed version strategy

### Delta

- **Doc A** : Bump v1->v2 en S67 avec CuratorVouched. `#[serde(other)]` pour forward-compat. Decision gelee D-GEL-7 : "batch CuratorVouched + SearchManifestPublished dans un seul bump".
- **Doc B** : "raw op / serde_json::Value opaque avant nouvelles ops publiques"
- **Ref 1 (feed_version_bump_strategy.md)** : Option E recommandee -- `serde_json::Value` pour le champ `op` dans `FeedEntry`, pas de bump pour les nouvelles ops.

### Analyse

Les deux strategies sont **contradictoires** :

| Aspect | Doc A (#[serde(other)]) | Ref 1 (serde_json::Value) |
|--------|------------------------|---------------------------|
| Payload des ops inconnues | **PERDU** (unit variant) | **PRESERVE** (JSON opaque) |
| Verification crypto des ops inconnues | **IMPOSSIBLE** (canonical bytes non recalculables) | **POSSIBLE** (JCS sur Value) |
| Hash-chain integrite | **TROUEE** (entries inconnues = trou noir) | **INTACTE** (toutes entries verifiables) |
| Propagation des ops inconnues | **IMPOSSIBLE** (re-serialise comme "Unknown") | **CORRECTE** (blob original intact) |
| Bump necessaire pour nouvelle op | **OUI** (mais batche) | **NON** |
| Pattern protocoles hash-chain | **Contraire** (Avro seulement) | **Conforme** (SSB, Bitcoin, Ethereum) |

La recherche Ref 1 demontre de maniere exhaustive que `#[serde(other)]` est incompatible avec un append-only log cryptographiquement lie. Le payload est perdu, les canonical bytes ne sont pas recalculables, la verification est impossible.

### Resolution

**Option E (serde_json::Value) est la seule strategie correcte.**

```
FEED VERSION STRATEGY FINALE :

1. FeedEntry.op : PublicFeedOperation → serde_json::Value
   (refacto interne, pas de changement wire format)

2. PublicFeedOperation reste un enum type pour l'INTERPRETATION
   (validation semantique, materialisation), utilise via try_parse_op()

3. FEED_FORMAT_VERSION reste a 1. Ne bumpe que si la structure
   de FeedEntry change (nouveau champ obligatoire, changement hash algo)

4. Ajouter CuratorVouched en S67 : pas de bump
5. Ajouter SearchManifestPublished en S72 : pas de bump

6. Implementation : S65 Phase A ou S67 Phase A (avant le premier
   ajout d'operation)

7. D-GEL-7 est REMPLACEE par : "Les operations feed sont extensibles
   sans bump de version. Le transport utilise serde_json::Value."
```

---

## Conflit 6 : Tantivy vs FTS5

### Delta

- **Doc A S70** : "Tantivy (~0.22), pas FTS5" avec gate fallback FTS5 si Tantivy derive. Decision gelee D-GEL-5.
- **Doc B S70** : "FTS5 d'abord. Tantivy gate post-S75."
- **Ref 2 (tantivy_vs_fts5_decision.md)** : FTS5 recommande. Analyse exhaustive.

### Analyse

La recherche Ref 2 est la plus approfondie des trois sources. Points cles :

1. **FTS5 est DEJA compile dans le binaire** (rusqlite bundled active -DSQLITE_ENABLE_FTS5). Zero cout d'adoption.
2. **Le dataset est negligeable** : <500 documents pre-launch, <6K a 6 mois. La difference BM25 Tantivy vs BM25 FTS5 est imperceptible.
3. **Jointures natives** : FTS5 fait un JOIN SQL avec provenance_records en une requete. Tantivy necessite N+1 queries.
4. **Tantivy ajoute ~40-60 crates transitives**, +3-5 MB binaire, +15 MB memoire minimum par index, +2-4 min build from scratch.
5. **Le fuzzy search (seul avantage reel de Tantivy)** n'est pas un besoin mesure. Le prefix search (`deploy*`) couvre 90% des cas.

Doc A met Tantivy en D-GEL-5 avec un fallback FTS5. Doc B et Ref 2 inversent : FTS5 d'abord, Tantivy en gate conditionnel.

### Resolution

**FTS5 d'abord. Tantivy en gate conditionnel mesurable.**

```
DECISION MOTEUR RECHERCHE S70 :

- Implementer avec FTS5 dans coordinator.db
- Zero nouvelle dependance, zero impact build/binaire/memoire
- ~150-250 LOC Rust (un fichier search.rs)
- Trait SearchEngine pour abstraction (Fts5SearchEngine, future TantivySearchEngine)

GATE TANTIVY (criteres mesurables, pas intuition) :
- Dataset > 50K documents indexees
- Latence p95 > 100ms sur requete MATCH
- Fuzzy search = top 3 feature request utilisateurs
- Facettes necessaires pour UI Browse redesign

Si aucun critere atteint a S75 : Tantivy reporte indefiniment.

D-GEL-5 est REMPLACEE par : "FTS5 pour S70+. Tantivy en gate conditionnel
si dataset > 50K ou latence p95 > 100ms."
```

---

## Conflit 7 : Decisions gelees (D-GEL-1 a D-GEL-8)

### Analyse de chaque D-GEL vs Doc B

| D-GEL | Doc A | Contredit par Doc B ? | Resolution |
|-------|-------|----------------------|------------|
| **D-GEL-1** : iroh 0.98 pour arc 1 | Rester sur 0.98 pour S65-S69 | **Non.** Doc B ne mentionne pas iroh upgrade. | **MAINTENUE.** |
| **D-GEL-2** : OS sandbox pour Factory, pas wasmtime | Factory S74 utilise isolation OS | **Non.** Doc B confirme : "canonicalize, prefix check, symlink deny, no shell depuis iframe" (section 11 G5). | **MAINTENUE.** |
| **D-GEL-3** : Pilote ferme (2-3 personnes) | S69 ferme | **Non.** Doc B S69 dit "canari ferme". | **MAINTENUE.** |
| **D-GEL-4** : Sequentiel, arc 2 avant arc 3 | RRV (arc 2) precede Factory (arc 3) | **OUI, CONTREDITE.** Doc B met Factory en S67-S68 (avant RRV S70). Les "arcs" n'ont plus le meme sens. | **REMPLACEE** (voir ci-dessous). |
| **D-GEL-5** : Tantivy avec fallback FTS5 | Tantivy pour S70, FTS5 si derive | **OUI, INVERSEE.** Doc B + Ref 2 : FTS5 d'abord. | **REMPLACEE** (voir Conflit 6). |
| **D-GEL-6** : Babel MVP fixtures | S75 Babel avec fixtures pre-traduites | **Non sur le fond, oui sur le timing.** Doc B met Babel en S69 (pas S75) mais garde les fixtures. | **MAINTENUE sur le fond.** Timing ajuste : fixtures des S69. |
| **D-GEL-7** : Feed v2 batche | CuratorVouched + SearchManifestPublished dans un seul bump | **OUI, REMPLACEE.** Doc B + Ref 1 : serde_json::Value, pas de bump. | **REMPLACEE** (voir Conflit 5). |
| **D-GEL-8** : Vocabulaire "source verifiable" | Apps = "source verifiable", pas "open source" | **Non.** Doc B section 4 confirme les memes formulations. | **MAINTENUE.** |

### Resolutions pour les D-GEL contredites

**D-GEL-4 REMPLACEE par :**

```
D-GEL-4-v2 : Sequentiel, Factory avant RRV

Les sprints sont sequentiels (solo maintainer). Factory (S67-S68) precede
RRV (S70-S72). Babel canari (S69) est le point de jonction : il valide
Factory ET fournit un corpus reel pour RRV. La parallelisation n'est
possible que si un contributeur externe prend un arc.
```

**D-GEL-5 REMPLACEE par :**

```
D-GEL-5-v2 : FTS5 d'abord, Tantivy en gate conditionnel

Le moteur de recherche S70 est SQLite FTS5 (deja compile dans le binaire).
Trait SearchEngine pour abstraction future. Gate Tantivy : dataset > 50K
docs OU latence p95 > 100ms OU fuzzy search = top 3 feature request.
```

**D-GEL-7 REMPLACEE par :**

```
D-GEL-7-v2 : Feed ops extensibles sans bump

Les operations feed sont extensibles sans bump de FEED_FORMAT_VERSION.
Le transport utilise serde_json::Value pour le champ op. La verification
crypto (BLAKE3 + Ed25519) fonctionne sur le JSON opaque. Le bump est
reserve aux changements de structure de FeedEntry (nouveau champ
obligatoire, changement hash algo, changement domain tag).
```

---

## Conflit 8 : Tests d'acceptance

### Delta

- **Doc A S69** : 7 criteres go/no-go sous forme de tableau (installation, lancement, P2P, deploy, feed sync, restart, stabilite 24h) avec seuils quantitatifs (2/3 testeurs, <30s, <5 min, 24h).
- **Doc B section 12** : 22 tests bloquants techniques avant Babel canari + 5 tests post-S70/S71.

### Analyse

Les deux sets de tests sont **complementaires**, pas en conflit :

- Doc A teste le **scenario utilisateur** (un humain installe, deploie, utilise pendant 24h)
- Doc B teste le **contrat technique** (deploy refuse manifest invalide, path traversal bloque, etc.)

Les tests Doc B sont des **tests automatises** (assertions dans le code). Les criteres Doc A sont des **tests manuels** (verification pilote avec des testeurs reels).

### Resolution

**Les tests Doc B COMPLETENT les criteres Doc A pour S69. Ils ne les remplacent pas.**

```
GATE S69 — Criteres combines :

1. TESTS AUTOMATISES (Doc B section 12, 22 tests) :
   Tous doivent etre verts dans le CI avant le debut du pilote.
   Integres comme tests Rust dans nexus-shell-daemon et nexus-coordinator-rs.

2. TESTS PILOTE MANUELS (Doc A, 7 criteres go/no-go) :
   Evalues par les 2-3 testeurs pendant le pilote ferme.
   Adapter les scenarios au contexte Babel :
   - "Deploy app" → "Deploy Babel Reader via Factory"
   - "Feed sync" → "Feed sync incluant ReleasePublished pour Babel"
   
3. TESTS POST-PILOTE (Doc B, 5 tests post-S70/S71) :
   - RRV trouve Babel localement
   - Proof Card affiche source, artifact, provenance, feed
   - Proof Card indique stale si source cassable
   - Score de completude deterministe
   - FTS5 p95 acceptable sur corpus test
   
   Ces tests deviennent les criteres de validation de S70/S71.
```

---

## Conflit 9 : Gates Factory G0-G10

### Delta

- **Doc A** : Pas de gates Factory. Factory est en S73-S75. La structure de gates est celle des arcs (Gate 0, Gate 1, Gate 2).
- **Doc B section 11** : 10 gates Factory (G0-G10) detaillees, applicables des S67.

### Analyse

Si Factory monte en S67 (ce que la resolution des conflits precedents confirme), les gates Factory DOIVENT etre dans le roadmap canon. Elles definissent le contrat qualite de chaque operation Factory.

Les gates Factory sont des **gates de validation technique** (checklist automatisable), pas des **gates de decision PO** (go/no-go). Elles ne remplacent pas les gates d'arc (Gate 0 canonisation, Gate 1 go/no-go arc 2) -- elles les completent a un niveau de granularite plus fin.

### Resolution

**Les gates G0-G10 de Doc B sont integrees dans le roadmap canon, distribuees par sprint.**

```
GATES FACTORY — Distribution par sprint :

S67 (Factory Foundation) :
  G0 - Classification app (domaine, risque, bridge methods, network, compute)
  G1 - Scope (MVP borne, non-goals explicites)
  G2 - Template (template id/version, hash, lockfile)
  G3 - Manifest (schema v2 valide, no node_id, bridge allowlist)

S68 (Broker/Preview/Publish) :
  G4 - Diff (preview obligatoire, approbation utilisateur)
  G5 - Sandbox (canonicalize, prefix check, symlink deny, no shell depuis iframe)
  G6 - Secrets/deps (scan secrets, lockfile, SBOM si publish)
  G7 - Preview (iframe sandbox, CSP, no external fetch par defaut)

S69 (Babel canari) :
  G8 - Provenance (factory.provenance.json, generator version, template hash,
       variables hash, source commit)
  G9 - Publish (repo HTTPS, commit 40 hex, artifact hash, provenance,
       Browse, feed)

S69 Phase E (go/no-go) :
  G10 - Review (sprint review, ## Verdict: PASS, evidence pack)
```

---

## Conflit 10 : Manifest app v2 — quand definir SBFB.json v2 ?

### Delta

- **Doc A** : SBFB.json v2 en S73 Phase A. Le champ `node_id` est present dans le struct v2 propose (implicitement, car deploy.rs le verifie toujours).
- **Doc B** : SBFB.json v2 en S67 avec retrait `node_id`.
- **Ref 3 (factory_deploy_constraint_research.md)** : Option D recommandee -- retirer `node_id` du manifest. L'attribution est dans la provenance signee. 5 lignes de changement dans deploy.rs.

### Analyse

Si Factory est en S67, SBFB.json v2 DOIT etre defini en S67 (pas en S73). Les templates Factory generent du SBFB.json v2. Attendre S73 signifie que 6 sprints de Factory (S67-S72) generent du SBFB.json v1 avec le hack PLACEHOLDER.

La recherche Ref 3 est categorique : le `node_id` dans SBFB.json est une propriete du **deployeur**, pas de l'**app**. L'attribution est dans la provenance signee Ed25519, pas dans le manifest. Le `node_id` dans le zip n'a pas de valeur cryptographique propre (il n'est pas signe directement). Le retrait ameliore la reproductibilite des hashes (meme code = meme hash quel que soit le deployeur).

### Resolution

**SBFB.json v2 defini en S67 Phase A. node_id retire (Option D).**

```
S67 Phase A — SBFB.json v2 + retrait node_id :

1. Modifier SbfbJson :
   - node_id: String → node_id: Option<String> avec #[serde(default)]
   - Ajouter schema_version: u32 avec default 1
   - Ajouter name, version, display_name, description, category,
     license, lang, bridge, tech, requirements (tous Option avec serde(default))

2. deploy.rs :
   - Supprimer le bloc if sbfb.node_id != state.node_id
   - Ajouter debug! log si node_id present (deprecated warning)
   - Ajouter validate_sbfb_json() pour v2 (name+version obligatoires,
     bridge.methods contre allowlist)

3. Migration apps existantes :
   - examples/sbfb-explorer/SBFB.json → v2 (sans node_id)
   - examples/sbfb-ideas/SBFB.json → v2 (sans node_id)

4. Tests :
   - v1 compat (JSON ancien format, parse OK)
   - v2 parse OK, v2 rejet si name manquant
   - v2 rejet si bridge.methods invalide
   - mismatch node_id ne provoque plus de rejet (juste warning)

Effort : ~1 heure. 5 lignes effectives dans deploy.rs + struct etendu.
Retro-compatible : tous les SBFB.json existants parsent sans erreur.
```

---

## Synthese des resolutions

### Decisions tranchees

| # | Conflit | Resolution | Doc gagnant |
|---|---------|-----------|-------------|
| 1 | Sequencage arcs | Factory S67-S68, Babel canari S69, RRV S70+ | Doc B |
| 2 | S67 contenu | Factory Foundation + CuratorVouched minimal | **Hybride** (Doc B + Doc A Phase A) |
| 3 | S68 contenu | Broker/preview/publish. Proof pack release → S74/S75 | Doc B |
| 4 | S69 contenu | Babel canari = vehicule du pilote | Doc B (absorbe Doc A) |
| 5 | Feed version | Option E (serde_json::Value), pas de bump | Ref 1 |
| 6 | Tantivy vs FTS5 | FTS5 d'abord, Tantivy en gate conditionnel | Ref 2 + Doc B |
| 7 | D-GEL contredites | D-GEL-4, D-GEL-5, D-GEL-7 remplacees | Doc B + Refs |
| 8 | Tests acceptance | Doc B complete Doc A (automatises + manuels) | Hybride |
| 9 | Gates Factory | G0-G10 integrees, distribuees par sprint | Doc B |
| 10 | Manifest v2 timing | S67 Phase A, pas S73 | Doc B + Ref 3 |

### D-GEL finales (8 decisions gelees pour la roadmap corrigee)

```
D-GEL-1 : iroh 0.98 pour S65-S69 (INCHANGEE)
D-GEL-2 : OS sandbox pour Factory, pas wasmtime (INCHANGEE)
D-GEL-3 : Pilote ferme, 2-3 personnes (INCHANGEE)
D-GEL-4-v2 : Sequentiel, Factory avant RRV (MODIFIEE)
D-GEL-5-v2 : FTS5 d'abord, Tantivy en gate conditionnel (INVERSEE)
D-GEL-6 : Babel MVP fixtures des S69 (TIMING AJUSTE)
D-GEL-7-v2 : Feed ops extensibles via serde_json::Value, sans bump (REMPLACEE)
D-GEL-8 : Vocabulaire "source verifiable" (INCHANGEE)
```

### Impact sur le graphe de dependances

```
                    S65 Contrat Public + specs Factory
                   / |
                  /  |
         S66 Durabilite
           |
       S67 Factory Foundation + CuratorVouched
           |
       S68 Broker / Preview / Publish gate
           |
       S69 Babel Reader canari = pilote ferme
           |          \
      GATE 1 go/no-go  \
           |             \
       S70 RRV LocalOnly (FTS5, corpus reel Factory+Babel)
           |
       S71 Proof Cards (+ gouvernance UX enrichie)
           |
       S72 SearchManifest opt-in
           |
      GATE 2 go/no-go
           |
       S73 Factory hardening + templates additionnels
           |
       S74 Babel translation beta + proof pack release
           |
       S75 Pack produit defendable + go/no-go public
```

### Action requise

Le document `.planning/roadmap_v2_public_trust_rrv_factory.md` doit etre mis a jour pour integrer ces resolutions. Les modifications sont :

1. Remplacement du contenu S67-S75 selon la sequence finale
2. Remplacement de D-GEL-4, D-GEL-5, D-GEL-7 par leurs versions v2
3. Ajout des gates Factory G0-G10 distribuees par sprint
4. Ajout des 22 tests d'acceptance Doc B comme complement des criteres existants
5. Correction de la section feed version (Option E au lieu de #[serde(other)])
6. Correction du moteur recherche (FTS5 au lieu de Tantivy)
7. Mise a jour du graphe de dependances
8. Mise a jour du calendrier previsionnel (memes semaines, contenu different)
9. Mise a jour du tableau delta tests projete

---

*Ce document est une evidence de recherche. Il ne remplace pas le roadmap canon.
Le roadmap canon doit etre mis a jour par un commit dedie qui reference ce document.*
