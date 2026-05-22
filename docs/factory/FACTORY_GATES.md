# Factory Gates — Pipeline de publication d'apps SBFB

**Version** : 1.0 (spec, pas de code — implementation S67-S69).
**Auteur** : Sprint 65 Phase D.
**Dependances aval** : S67 Factory Foundation, S68 Broker/Preview,
S69 Babel Reader canari.
**Amendement 2026-05-22** : l'architecture Factory a ete recadree par la
roadmap v4. Factory est maintenant un outil/client externe (`sbfb-factory`),
pas un module metier du daemon. Les gates FG0-FG10 restent valides comme
contrat qualite, mais toute phrase ci-dessous qui parle de "module du daemon"
doit etre lue comme fossile architectural.

---

## Vue d'ensemble

Factory est le pipeline qui guide un developpeur de l'idee a la
publication d'une app sur le reseau SBFB. Il se decompose en 11
gates sequentielles. Chaque gate recoit un input, produit un output,
et doit etre franchie avant de passer a la suivante.

Factory est un outil client externe (`sbfb-factory`) qui consomme les API
neutres du daemon. Il n'y a pas de serveur central : chaque utilisateur peut
executer sa propre Factory pour les apps qu'il publie. Le daemon reste neutre
et ne doit pas embarquer de logique metier Factory.

Les gates FG0-FG7 sont locales (pas de reseau requis). FG8-FG9
impliquent le reseau P2P (signature + deploy). FG10 est post-
publication (curator review asynchrone).

---

## Gates

### FG0 — Classification

| Champ | Valeur |
|-------|--------|
| **Input** | Intention du developpeur (interactive ou SBFB.json existant) |
| **Output** | Type d'app classifie : `static-html`, `react`, `pyodide`, `wasm`, `jupyterlite` |
| **Description** | Determine le type technique de l'app pour selectionner le template, les outils de build, et les contraintes sandbox. Si un SBFB.json existant est fourni, le type est extrait de `tech.type`. Sinon, le broker pose 2-3 questions interactives. |
| **Critere de passage** | `tech.type` valide dans l'ensemble des types supportes |
| **Sprint cible** | S67 Phase A |

### FG1 — Scope

| Champ | Valeur |
|-------|--------|
| **Input** | Type classifie (FG0) + intention fonctionnelle |
| **Output** | Liste des permissions bridge requises (`bridge.methods` dans SBFB.json v2) |
| **Description** | Determine quelles methodes du bridge postMessage l'app a besoin (`task_submit`, `storage_get`, `storage_set`). Une app statique HTML n'a besoin d'aucune methode. Une app avec persistance a besoin de `storage_get` + `storage_set`. Le scope definit les capabilities sandbox. |
| **Critere de passage** | `bridge.methods` est un sous-ensemble valide des methodes whitelist |
| **Sprint cible** | S67 Phase A |

### FG2 — Template

| Champ | Valeur |
|-------|--------|
| **Input** | Type (FG0) + scope (FG1) |
| **Output** | Repertoire scaffold genere (index.html, SBFB.json v2, sbfb-bridge.js si besoin) |
| **Description** | Genere la structure de base du projet depuis un template correspondant au type. Inclut le manifest SBFB.json v2 pre-rempli, le SDK bridge si des methodes sont requises, et un index.html minimal fonctionnel. Le template est local — pas de telechargement reseau. |
| **Critere de passage** | Repertoire contient index.html + SBFB.json valide schema v2 |
| **Sprint cible** | S67 Phase B |

### FG3 — Manifest

| Champ | Valeur |
|-------|--------|
| **Input** | SBFB.json v2 (genere FG2 ou fourni par le developpeur) |
| **Output** | Manifest valide schema-checked |
| **Description** | Valide le manifest SBFB.json v2 contre le schema (cf. `SBFB_JSON_V2.md`). Verifie les champs requis (name, display_name, description), la coherence type/methods, et les contraintes de format. Accepte les manifests v1 (champs optionnels v2 a leur defaut). |
| **Critere de passage** | Validation schema reussie, 0 erreur |
| **Sprint cible** | S67 Phase A |

### FG4 — Diff

| Champ | Valeur |
|-------|--------|
| **Input** | Repertoire app apres modifications developpeur |
| **Output** | Rapport diff resume (fichiers modifies, lignes ajoutees/supprimees) |
| **Description** | Presente un resume des changements depuis le template initial (ou depuis la derniere version publiee en cas de mise a jour). Permet au developpeur de verifier ce qu'il s'apprete a publier. Pas de review automatique a ce stade — c'est une preview humaine. |
| **Critere de passage** | Developpeur confirme les changements |
| **Sprint cible** | S68 Phase A |

### FG5 — Sandbox

| Champ | Valeur |
|-------|--------|
| **Input** | Archive app (zip) |
| **Output** | Rapport de test sandbox (CSP violations, tentatives `allow-same-origin`, scripts externes) |
| **Description** | Charge l'app dans un iframe sandbox local (`sandbox="allow-scripts"` sans `allow-same-origin`, CSP `connect-src 'none'`). Detecte les violations : tentative d'acces au DOM parent, scripts charges depuis un CDN externe, form submit (bloque par sandbox), appels fetch/XHR hors bridge. Le test est local — le daemon charge l'archive via blob-serve en loopback. |
| **Critere de passage** | 0 violation CSP, 0 tentative same-origin, toutes les communications passent par le bridge |
| **Sprint cible** | S68 Phase B |

### FG6 — Secrets/deps

| Champ | Valeur |
|-------|--------|
| **Input** | Repertoire app |
| **Output** | Rapport de scan (secrets detectes, deps avec CVE connues) |
| **Description** | Scan les fichiers de l'app pour detecter des patterns de secrets (API keys, tokens, credentials) et des deps JavaScript avec des CVE connues. Les deps sont scannees si un package.json existe. Le scan est heuristique (regex patterns) — pas de connexion a une base de vulnerabilites en ligne. La base CVE est embarquee dans le daemon et mise a jour avec les releases. |
| **Critere de passage** | 0 secret detecte. Deps avec CVE : warning affiche, pas bloquant (le developpeur decide) |
| **Sprint cible** | S68 Phase B |

### FG7 — Preview

| Champ | Valeur |
|-------|--------|
| **Input** | Archive app validee (FG5 + FG6) |
| **Output** | URL loopback temporaire pour preview live |
| **Description** | Ouvre l'app dans le navigateur via blob-serve en mode preview (URL ephemere, pas encore publiee sur le reseau). Le developpeur peut tester l'app dans les conditions reelles (iframe sandbox, bridge actif si methodes declarees). La preview expire apres fermeture ou timeout (30 min). |
| **Critere de passage** | Developpeur confirme que l'app fonctionne comme attendu |
| **Sprint cible** | S68 Phase C |

### FG8 — Provenance

| Champ | Valeur |
|-------|--------|
| **Input** | Repertoire app + depot git source (si app publique) |
| **Output** | provenance.json SLSA L1 (Ed25519 signature, commit SHA, artifact hash BLAKE3) |
| **Description** | Genere la provenance auto-attestee SLSA L1 : clone le depot source, verifie que le commit est celui declare, construit l'archive zip, signe le hash BLAKE3 de l'archive avec la cle Ed25519 du noeud local. La provenance lie un commit source au hash de l'archive. Pour les apps N0 (upload direct sans depot), cette gate est skippee et l'app est publiee sans provenance. |
| **Critere de passage** | provenance.json valide, signature Ed25519 verifiable, artifact hash correspond a l'archive |
| **Sprint cible** | S69 Phase A |

### FG9 — Publish

| Champ | Valeur |
|-------|--------|
| **Input** | Archive zip + provenance.json (si applicable) + SBFB.json v2 |
| **Output** | ProjectAnnouncement broadcast sur gossip + FeedEntry ReleasePublished insere |
| **Description** | Publie l'app sur le reseau SBFB. L'archive est inseree dans iroh-blobs, l'annonce est broadcastee via gossip, et une operation ReleasePublished est inseree dans le feed public (cf. deploy→feed wiring S65 Phase A). Le hash BLAKE3 de l'archive est l'identifiant permanent de cette version. |
| **Critere de passage** | Annonce recue par au moins 1 peer (ou self si noeud isole). FeedEntry inseree avec hash-chain valide. |
| **Sprint cible** | S69 Phase A |

### FG10 — Review

| Champ | Valeur |
|-------|--------|
| **Input** | App publiee sur le reseau |
| **Output** | CuratorVouched ou CuratorDisendorsed (operation feed asynchrone) |
| **Description** | Apres publication, les curators du reseau peuvent examiner l'app et emettre un vouch (endorsement positif) ou un disendorsement. Ce processus est asynchrone et volontaire — il n'y a pas de review obligatoire, pas de validation centralisee. Les curators sont des noeuds qui maintiennent des listes de curation signees Ed25519. Un vouch curator eleve le niveau de confiance de l'app dans la taxonomie (cf. TRUST_TAXONOMY.md dimension transversale "Curator vouch"). |
| **Critere de passage** | N/A — gate asynchrone, pas de blocage. La publication est effective des FG9. |
| **Sprint cible** | S69 Phase B (CuratorVouched/CuratorDisendorsed feed operations) |

---

## Flux global

```
Developpeur
    |
    v
[FG0 Classification] --> type
    |
    v
[FG1 Scope] ----------> bridge.methods
    |
    v
[FG2 Template] --------> scaffold projet
    |
    v
  (developpeur code l'app)
    |
    v
[FG3 Manifest] --------> SBFB.json v2 valide
    |
    v
[FG4 Diff] ------------> confirmation humaine
    |
    v
[FG5 Sandbox] ---------> test iframe local
    |
    v
[FG6 Secrets/deps] ----> scan securite
    |
    v
[FG7 Preview] ---------> test live loopback
    |
    v
[FG8 Provenance] ------> provenance.json SLSA L1
    |
    v
[FG9 Publish] ---------> app live sur le reseau
    |
    v
[FG10 Review] ---------> curator vouch (asynchrone)
```

---

## Principes de design

1. **Sequentiel, pas configurable.** Les gates sont parcourues dans
   l'ordre. Pas de skip (sauf FG8 pour apps N0 sans depot).
   La simplicite du flux lineaire est preferee a la flexibilite
   d'un DAG configurable.

2. **Local-first.** FG0-FG7 fonctionnent sans connexion reseau.
   Un developpeur peut preparer son app entierement hors-ligne.

3. **Pas de moderation centralisee.** Aucune gate ne requiert
   l'approbation d'une autorite. FG10 (curator review) est
   volontaire et post-publication.

4. **Deterministe.** Pour un meme input, une gate produit le meme
   output. Pas de composant ML, pas de scoring opaque.

5. **Client externe Rust.** Factory est un outil/client externe
   (`sbfb-factory`) qui consomme les API neutres du daemon, pas une
   app iframe et pas un module metier du daemon. L'interface future
   pilote le pipeline gate par gate sans deplacer l'autorite dans
   Factory.

---

## Retro-compatibilite

Les apps existantes (publiees avant Factory, S0-S64) ont ete
deployees manuellement via `deploy.rs`. Factory ne les invalide pas.
Le deploy existant reste fonctionnel — Factory est un chemin
additionnel, pas un remplacement obligatoire.

Les apps avec SBFB.json v1 sont acceptees par FG3 (les champs v2
sont optionnels avec defauts sensibles). Le parser de manifest
accepte `schema_version: 1` et `schema_version: 2`.
