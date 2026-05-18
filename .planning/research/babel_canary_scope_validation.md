# Validation de scope : Babel Reader canari S69

**Date :** 2026-05-18
**Confiance globale :** HIGH (code source verifie exhaustivement)
**Mode :** Feasibility + Ecosystem

---

## Table des matieres

1. [Scope MVP Babel — realisme pour 1 sprint](#1-scope-mvp-babel)
2. [Bridge methods requises — audit complet](#2-bridge-methods)
3. [Storage structure Babel — compatibilite bridge](#3-storage-structure)
4. [Source manifest minimal — mode de stockage](#4-source-manifest)
5. [Publish path canari — faisabilite par etape](#5-publish-path)
6. [Tests d'acceptance — ecrivabilite](#6-tests-dacceptance)
7. [Dependances S67-S68 — risque cascade](#7-dependances-s67-s68)
8. [Comparaison avec S69 original — scope creep ou synergie](#8-comparaison-s69-original)
9. [Verdict et recommandations](#9-verdict)

---

## 1. Scope MVP Babel — realisme pour 1 sprint {#1-scope-mvp-babel}

### Decomposition effort par item

| Item | Effort estime | Pre-requis | Notes |
|------|---------------|------------|-------|
| App `babel-reader` generee par Factory | 0j (si Factory S67-S68 livree) | Factory existe et fonctionne | L'app EST le livrable Factory, pas du code a la main |
| 3 textes domaine public, ~5 langues fixtures | 1j | Aucun (travail editorial) | Gutenberg/Wikisource. 3 textes courts (~500 mots). 5 langues = FR/EN/ES/DE/PT. Fixtures JSON statiques |
| Liste de textes | 0.5j | Factory template | HTML/CSS pur dans l'app. `storage_list` prefix `texts/` |
| Vue lecture | 0.5j | Factory template | HTML pur. Affichage du texte selectionne |
| Toggle langue | 0.5j | Fixtures chargees | Dropdown/bouton qui re-fetch la traduction |
| Progression/bookmarks (bridge storage) | 1j | `storage_get`, `storage_set` fonctionnels | `identity_pubkey` pour scoper par utilisateur. JSON `{position, last_read}` |
| Reviews minimales | 1.5j | Storage + identite | Formulaire simple (note 1-5 + commentaire libre). `storage_set` avec cle `reviews/{translation_id}/{pubkey}` |
| Source manifest visible | 0.5j | Fixtures JSON | Affichage d'un panneau "Source" avec les champs du manifest |
| Provenance app visible | 0.5j | `provenance_get` bridge | Appel bridge → affichage du record |
| Feed cursor visible | 0.5j | `feed_cursor_get` bridge | Affichage last_seq + last_entry_hash |
| Verification provenance depuis le bridge | 0.5j | `provenance_verify` bridge | Bouton "Verifier" → resultat {verified: true/false} |
| Task traduction mock si backend absent | 1j | `task_submit` bridge | Mock = renvoie la fixture existante avec un delai simule |

### Total estime : ~8 jours de travail net

Pour un sprint de 2 semaines (~10 jours effectifs) avec un solo maintainer :

**Verdict : FAISABLE mais serre.**

L'estimation de 8j laisse 2j de marge pour les imprevu (bugs Factory, fixes d'integration, commit discipline). C'est tenable SI et SEULEMENT SI :

1. **Factory S67-S68 est livree et fonctionnelle.** Si Factory ne sait pas generer un repo app propre, Babel ne peut pas demarrer. Ce n'est pas 8j d'effort Babel, c'est 8j + le delta de rattrapage Factory.

2. **Le domain pack Babel est prepare en S68.** Les fixtures (textes, traductions, manifests sources) doivent etre pretes avant S69. Sinon, le travail editorial consomme 2-3 jours supplementaires.

3. **Le pilote ferme est simplifie.** Le doc original de S69 prevoyait install VM + ticket/join UI + invite flow + logs/support + go/no-go. Avec Babel ajoute, il faut choisir : soit le pilote complet, soit Babel canari. Pas les deux dans le meme sprint.

### Recommandation de scope cut

Retirer de S69 canari :
- **Reviews minimales** (reporter a S74). Raison : c'est une feature sociale qui complique le stockage et l'UX sans prouver la viabilite Factory.
- **Task traduction mock** (reporter a S74). Raison : le mock ne prouve rien que les fixtures ne prouvent deja. Le vrai test de task_submit vient avec un worker reel.

Scope reduit = ~5.5j, marge confortable.

---

## 2. Bridge methods requises — audit complet {#2-bridge-methods}

### Protocole bridge : etat du code

**Fichier :** `web/src/bridge/protocol.ts` (lignes 20-40)

Le `BridgeMethodSchema` (Zod enum) contient exactement ces methodes :

```
task_submit, storage_get, storage_set, pii_redact,
storage_list, storage_delete, identity_pubkey, node_status,
browse_list, storage_version, provenance_get, provenance_verify,
feed_cursor_get
```

### Verification methode par methode

| Methode requise par Babel | Presente dans protocol.ts | Dispatch dans useBridge.ts | SDK sbfb-bridge.js | Backend Rust | Verdict |
|--------------------------|--------------------------|---------------------------|-------------------|--------------|---------|
| `storage_get` | OUI (L28) | OUI (L239-248) | `getStorage(key)` | storage_api.rs:214 (HashMap + SQLite + iroh-docs replicated) | **COMPLET** |
| `storage_set` | OUI (L29) | OUI (L250-256) | `setStorage(key, value)` | storage_api.rs:293 (rate-limited, DB write-through) | **COMPLET** |
| `storage_list` | OUI (L33) | OUI (L258-267) | `listStorage(prefix)` | storage_api.rs:117 (prefix filter, replicated support) | **COMPLET** |
| `storage_delete` | OUI (L34) | OUI (L269-278) | `deleteStorage(key)` | storage_api.rs:381 (tombstone for replicated) | **COMPLET** |
| `identity_pubkey` | OUI (L35) | OUI (L280-287) | `getIdentityPubkey()` | Via `/api/daemon/info` → `node_id` | **COMPLET** |
| `node_status` | OUI (L36) | OUI (L289-304) | `getNodeStatus()` | health + daemon/info | **COMPLET** |
| `browse_list` | OUI (L37) | OUI (L306-310) | `getBrowseList()` | `/api/daemon/browse` | **COMPLET** |
| `task_submit` | OUI (L22) | OUI (L233-236) | `submitTask(payload)` | coordinator task pipeline | **COMPLET** |
| `provenance_get` | OUI (L39) | OUI (L324-335) | `getProvenanceRecord(pid)` | `/api/v1/project/{pid}/provenance` | **COMPLET** |
| `provenance_verify` | OUI (L40) | OUI (L337-348) | `verifyRelease(pid)` | `/api/v1/project/{pid}/provenance` + Ed25519 check | **COMPLET** |
| `feed_cursor_get` | OUI (L41) | OUI (L350-356) | `getPublicFeedCursor()` | `/api/daemon/feed/cursor` | **COMPLET** |

### Verdict bridge : 11/11 methodes COMPLETES

Toutes les methodes listees dans le doc §5 existent, sont implementees dans les 3 couches (schema Zod, dispatch host-side, SDK client), et ont un backend Rust fonctionnel.

**Aucun effort supplementaire n'est requis pour les methodes bridge.**

### Methodes bonus disponibles mais non listees

- `pii_redact` : disponible si Babel veut offrir une redaction PII avant `task_submit`. Non necessaire pour le MVP.
- `storage_version` : disponible pour detecter les updates distants en temps reel. Utile pour la future replication P2P des reviews.
- `onStorageUpdate(appName, cb)` : polling SDK-side pour reactiver. Non necessaire pour MVP fixtures.

---

## 3. Storage structure Babel — compatibilite bridge {#3-storage-structure}

### Namespaces proposes (doc §8)

```
texts/{text_id}
translations/{lang}/{text_id}
bookmarks/{pubkey}/{text_id}
reviews/{translation_id}/{pubkey}
manifests/sources/{source_id}
app/state/{pubkey}
```

### Le bridge est-il namespace-aware ?

**OUI, via le mecanisme de prefix dans `storage_list`.**

Le code `storage_api.rs` (lignes 117-147, 149-208) supporte :

- `storage_list` avec parametre `prefix` : filtre les cles par prefix.
  - HashMap path : `app_map.iter().filter(|(k, _)| k.starts_with(p.as_str()))`
  - Replicated path : `ns_state.doc.get_many_latest_per_key_prefix(prefix_bytes)`

Le bridge `useBridge.ts` (ligne 259) passe `req.payload.prefix` correctement.

Le SDK `sbfb-bridge.js` : `listStorage(prefix)` envoie `{ prefix: prefix || "" }`.

### Comment ca marche en pratique pour Babel

```javascript
const bridge = new SBFBBridge();

// Lister tous les textes
const texts = await bridge.listStorage("texts/");
// → { entries: [{key: "texts/001", value: {...}}, ...], count: 3 }

// Lire une traduction specifique
const fr = await bridge.getStorage("translations/fr/001");
// → { key: "translations/fr/001", value: { title: "...", body: "..." } }

// Sauvegarder un bookmark
const pubkey = await bridge.getIdentityPubkey();
await bridge.setStorage(`bookmarks/${pubkey.pubkey}/001`, { position: 42, last_read: Date.now() });

// Lire mes bookmarks
const myBookmarks = await bridge.listStorage(`bookmarks/${pubkey.pubkey}/`);
```

### Limitation critique : pas de replication automatique pour les nouvelles apps

Le code `storage_api.rs` (ligne 29) contient un hardcode :

```rust
const REPLICATED_APPS: &[&str] = &["sbfb-ideas"];
```

**Seule l'app `sbfb-ideas` route vers iroh-docs pour la replication P2P.** Toutes les autres apps (dont `babel-reader`) utilisent le stockage local HashMap + SQLite.

**Impact pour Babel canari :** Le stockage est LOCAL au noeud. Les bookmarks, reviews et progression ne sont pas repliques entre les testeurs du pilote. C'est acceptable pour un canari ferme de 2-3 personnes (chacun a ses propres bookmarks), mais cela signifie :

1. Les reviews d'un testeur ne sont PAS visibles par les autres (pas de consensus humain sur les traductions).
2. Un texte ajoute par un testeur n'apparait PAS chez les autres.

**Recommandation :** Pour le canari S69, le stockage local suffit. Les fixtures sont pre-chargees dans l'app (pas via storage bridge). Le storage bridge sert uniquement aux bookmarks/progression/reviews locaux. La replication Babel sera un item S74 qui requiert l'ajout de `"babel-reader"` a `REPLICATED_APPS` ou, mieux, un mecanisme generique d'enregistrement d'apps replicables.

---

## 4. Source manifest minimal — mode de stockage {#4-source-manifest}

### Schema propose (doc §8)

12 champs : `source_id`, `source_url`, `source_hash`, droits, juridictions, redistribution autorisee, traduction autorisee, licence/politique de sortie, attribution, takedown policy, date d'import, signataire.

### C'est un schema applicatif, pas un schema protocole

**Correct.** Le manifest source est purement metier Babel. Le protocole SBFB ne doit pas connaitre `babel_*` (invariant doc §1). Le stockage se fait via le bridge `storage_set/get` comme n'importe quelle donnee applicative.

### Mode de stockage recommande : JSON dans le bridge storage

```javascript
// Ecrire un manifest source
await bridge.setStorage("manifests/sources/gutenberg-001", {
  source_id: "gutenberg-001",
  source_url: "https://www.gutenberg.org/ebooks/11",
  source_hash: "abc123...", // BLAKE3 du texte brut
  rights: {
    redistribution: true,
    translation: true,
    license: "PD-US", // Public Domain (US)
    jurisdictions: ["US", "EU"],
    attribution: "Lewis Carroll, via Project Gutenberg",
    takedown_policy: "N/A (public domain)"
  },
  imported_at: "2026-05-18T12:00:00Z",
  manifest_author: "<pubkey hex du mainteneur>"
});
```

### Pourquoi PAS une structure formelle protocole

1. **Le manifest source est subjectif.** Les droits d'un texte dependent de la juridiction. Le protocole ne peut pas valider "redistribution autorisee en France" -- c'est un jugement humain.
2. **Le schema va evoluer.** Babel canari n'a que 3 textes fixtures. Le schema V1 sera simpliste. Figer un schema protocole maintenant serait premature.
3. **Le bridge storage suffit.** JSON dans `storage_set` est exactement le bon niveau d'abstraction. Le SDK lit/ecrit, l'app interprete.

### Recommandation pour le canari

Pour les 3 textes fixtures, les manifests sont PRE-INCLUS dans le zip de l'app (fichiers JSON statiques). L'app les lit localement sans bridge. Le bridge `storage_set` entre en jeu uniquement si un testeur ajoute un texte (use case hors scope MVP canari).

---

## 5. Publish path canari — faisabilite par etape {#5-publish-path}

### Etape 1 : Factory genere Babel en Local Draft

**Factory n'existe pas dans le codebase.** Aucun module `factory` n'est present dans `crates/`. Les seules references a "Factory" dans le code sont `LlmClientFactory` (worker LLM, sans rapport).

**Prerequis S67-S68 :** Factory doit etre livree comme module Rust daemon + UI shell `/factory`. Sans Factory, l'etape 1 est un bloqueur absolu.

**Effort si Factory n'est pas prete :** L'alternative est de generer Babel a la main (mkdir, ecrire index.html, SBFB.json, copier sbfb-bridge.js). Mais cela viole le contrat du doc §9 : "Babel n'est pas code a la main hors Factory". Si cette regle est assouplie pour le canari, l'effort est de ~2j (ecrire l'app HTML/JS statique manuellement).

### Etape 2 : Broker produit un diff

**Broker n'existe pas.** Meme blocage que l'etape 1. Prerequis S68.

### Etape 3 : Utilisateur approuve

**UI `/factory` n'existe pas.** Prerequis S68. Si Factory est livree, cette etape est triviale (bouton "Approve").

### Etape 4 : Preview locale sandboxee

**blob-serve PEUT servir un draft.** Le blob-serve (`blob_serve.rs`) fonctionne avec n'importe quel hash de blob. Si le draft est stocke dans iroh-blobs (meme localement), blob-serve le decompresse et le sert dans un iframe sandbox.

**Mais :** Il faut un mecanisme pour passer de "fichiers sur disque" a "blob dans iroh-blobs" sans passer par `deploy-from-repo`. L'endpoint `POST /api/v1/deploy` (deploy prive, `deploy.rs:264`) accepte un upload raw zip et le stocke dans iroh-blobs. C'est utilisable pour la preview.

**Effort :** 0j si Factory utilise `POST /api/v1/deploy` pour la preview. Le mecanisme existe.

### Etape 5 : Tests Babel passent

**Comment tester une app iframe ?** Deux approches :

1. **Playwright** : ouvre le browser, navigue vers Browse, ouvre l'app dans l'iframe, interagit via l'API Playwright. Les tests existants de Protocol Explorer et Ideas Hub utilisent cette approche. Effort : 1-2j pour ecrire les tests.

2. **Tests unitaires JS dans l'app** : si l'app est generee avec un `test.html` ou des tests Vitest/Jest inclus. Effort : 0.5j si Factory genere les tests.

**Recommandation :** Playwright pour le smoke test E2E (app s'ouvre, textes visibles, toggle langue fonctionne). Pas de tests unitaires JS pour le canari.

### Etape 6 : Commit + push repo public

**Manuel.** Le mainteneur fait `git add && git commit && git push`. Factory peut generer le repo localement, mais le push est toujours manuel (pas d'acces git credential depuis le daemon).

**Effort :** 0j (workflow humain).

### Etape 7 : `POST /api/v1/deploy-from-repo`

**EXISTE et FONCTIONNE.** `deploy.rs:65` implete le handler complet. Verifie dans les tests S64 (753 lignes, 13 tests).

**Blocage `node_id` :** Le deploy actuel exige `sbfb.node_id == state.node_id` (deploy.rs:119). Pour un template portable genere par Factory, le `node_id` ne sera pas connu a la generation. Le doc §7 recommande de rendre `node_id` optionnel/deprecie. C'est un prerequis S67.

**Effort :** Ce fix est prevu en S67 (deprecation `node_id`). Si fait, etape 7 fonctionne. Si pas fait, l'app Babel doit etre generee avec le `node_id` du daemon, ce qui rend le template non-portable mais fonctionnel pour le canari.

### Etape 8 : Daemon clone, valide, zippe, hash, signe

**EXISTE.** Flow complet dans `deploy_from_repo()`. Clone → SBFB.json → index.html → zip → BLAKE3 → provenance Ed25519 → inject provenance.json dans zip.

**Effort :** 0j.

### Etape 9 : Blob archive publie/persiste

**EXISTE mais VOLATIL.** Le blob est stocke dans iroh-blobs (MemStore). Au restart du daemon, le blob disparait.

**Prerequis S66 :** La durabilite des blobs est un livrable S66. Sans S66, le canari perd l'app au restart.

**Effort :** 0j si S66 livre. Bloqueur sinon.

### Etape 10 : Browse entry creee

**EXISTE.** `publish_announcement()` dans `deploy.rs:316-379` cree un `BrowseEntry` et le broadcast via gossip.

**Effort :** 0j.

### Etape 11 : Public feed `ReleasePublished` cree automatiquement

**N'EXISTE PAS.**

J'ai grep exhaustivement : `deploy.rs` ne contient AUCUNE reference a `ReleasePublished`, `insert_feed`, ou `public_feed`. La fonction `deploy_from_repo()` s'arrete apres `publish_announcement()` (gossip + Browse). Il n'y a PAS d'insertion automatique dans le feed append-only.

Le `feed_insert` handler existe dans `feed_sync.rs:445` mais c'est un endpoint HTTP (`POST /api/daemon/feed/insert`) appele manuellement.

**Gap confirme :** Le doc §5 dit "deploy-from-repo ne cree pas encore automatiquement l'entree public feed ReleasePublished". C'est exact.

**Effort pour corriger :** MEDIUM (~0.5-1j). Il faut ajouter dans `deploy_from_repo()`, apres le `publish_announcement()` :

```rust
// Insert ReleasePublished into public feed
if let Some(feed_state) = &state.feed_sync_state {
    let op = PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
        project_id: state.node_id.clone(),
        repo_url: repo_url.clone(),
        commit_sha: commit_sha.clone(),
        artifact_hash: artifact_hash_hex.clone(),
        provenance_hash: Some(prov_hash.clone()),
        is_open_source: true,
    });
    // ... sign and insert
}
```

**Attention :** Le feed exige `project_id` hex-64 et `repo_url` HTTPS. Actuellement, `project_id` est `state.node_id` (qui est hex-64 Ed25519). Et `deploy_from_repo` accepte `http` dans le code (la validation est `starts_with("http")`, pas `starts_with("https")`), tandis que le feed exige `https://`. Il faut d'abord corriger la validation deploy OU ajouter une verification specifique avant l'insertion feed.

**Ce correctif devrait etre fait en S67 ou S68, pas en S69.** C'est une plomberie protocole, pas du travail Babel.

### Etape 12 : Evidence pack capture

**NOUVEAU. Aucun code existant.**

L'evidence pack est un dossier avec provenance record, feed entry, Browse entry, screenshots, resultats de tests. C'est essentiellement un `proof-pack` au sens du doc S68.

**Effort :** Si S68 livre le proof-pack CLI, l'evidence pack Babel est un appel de commande. Si S68 ne livre pas, il faut collecter manuellement (~0.5j).

### Synthese publish path

| Etape | Existe | Blocage | Sprint requis |
|-------|--------|---------|---------------|
| 1. Factory genere | NON | BLOQUANT | S67 |
| 2. Broker diff | NON | BLOQUANT | S68 |
| 3. Utilisateur approuve | NON | BLOQUANT | S68 |
| 4. Preview sandboxee | OUI (via deploy prive) | Non | - |
| 5. Tests Babel | Outil existe (Playwright) | Non | - |
| 6. Commit + push | Workflow humain | Non | - |
| 7. deploy-from-repo | OUI (node_id fix requis) | BLOQUANT SI pas fix | S67 |
| 8. Clone/valide/zippe/hash/signe | OUI | Non | - |
| 9. Blob persiste | OUI SI S66 livre | BLOQUANT | S66 |
| 10. Browse entry | OUI | Non | - |
| 11. Feed ReleasePublished auto | NON | Gap a combler | S67-S68 |
| 12. Evidence pack | NON | Gap a combler | S68 |

**3 bloqueurs durs (Factory), 2 gaps a combler (feed auto + evidence), 1 prerequis infra (durabilite blobs).**

---

## 6. Tests d'acceptance — ecrivabilite {#6-tests-dacceptance}

### Analyse des 22 tests bloquants (doc §12)

| # | Test | Code necessaire existe | Type test | Effort | Sprint |
|---|------|----------------------|-----------|--------|--------|
| 1 | deploy-from-repo accepte app sans node_id | NON (node_id requis actuellement) | Rust integration | 0.5j (modifier deploy + test) | S67 |
| 2 | deploy-from-repo refuse manifest invalide | PARTIEL (valide node_id + index.html, pas "manifest invalide" au sens SBFB.json v2) | Rust integration | 0.5j | S67 |
| 3 | deploy-from-repo refuse methode bridge inconnue | NON (deploy ne lit pas les bridge methods) | Rust integration | 1j (lire SBFB.json v2, valider methods) | S67-S68 |
| 4 | deploy-from-repo refuse repo non HTTPS pour feed | PARTIEL (deploy accepte HTTP, feed exige HTTPS) | Rust integration | 0.5j | S67-S68 |
| 5 | App generee contient index.html, SBFB.json, SDK bridge | NON (Factory n'existe pas) | Test Factory | 0.5j | S67 |
| 6 | App generee contient planning sprint | NON (Factory n'existe pas) | Test Factory | 0.5j | S67 |
| 7 | App generee contient factory.template.lock | NON (Factory n'existe pas) | Test Factory | 0.5j | S67 |
| 8 | App generee contient factory.provenance.json | NON (Factory n'existe pas) | Test Factory | 0.5j | S67 |
| 9 | Path traversal refuse | OUI (deploy.rs:547 `name.contains("..")`) + blob_serve.rs validate_zip_path | Rust unit (existe deja implicitement) | 0j | - |
| 10 | Symlink refuse | OUI (deploy.rs:544 `path.is_symlink()`) | Rust unit (existe deja implicitement) | 0j | - |
| 11 | Secret fixture refuse | NON (pas de scan secrets dans deploy) | Rust integration | 1j | S68 |
| 12 | Preview iframe smoke test | Playwright possible | Playwright | 0.5j | S68-S69 |
| 13 | Babel affiche textes fixtures | NON (Babel n'existe pas) | Playwright | 0.5j | S69 |
| 14 | Babel lit/ecrit progression storage | NON | Playwright | 0.5j | S69 |
| 15 | Babel appelle identity_pubkey | NON | Playwright | 0.5j | S69 |
| 16 | Babel affiche provenance | NON | Playwright | 0.5j | S69 |
| 17 | Babel lit feed cursor | NON | Playwright | 0.5j | S69 |
| 18 | Babel deploy -> Browse -> open -> provenance verify | NON | Playwright E2E | 1j | S69 |
| 19 | Feed ReleasePublished cree si chemin cable | NON (pas d'auto-insert) | Rust integration | 0.5j | S67-S68 |
| 20 | Test negatif : aucune methode babel_*/factory_*/shell_* | Verification manuelle | Grep + Rust test | 0.5j | S69 |
| 21 | RRV trouve Babel localement | NON (RRV n'existe pas) | Post-S70 | - | S70 |
| 22 | Proof Card affiche source/artifact/provenance/feed | NON (Proof Cards n'existent pas) | Post-S71 | - | S71 |

### Repartition par sprint

- **Tests S67 (Factory foundation) :** #1, #2, #3, #4, #5, #6, #7, #8 = ~4j
- **Tests S68 (Broker/preview/publish gate) :** #3, #4, #11, #12, #19 = ~3j
- **Tests S69 (Babel canari) :** #13, #14, #15, #16, #17, #18, #20 = ~4j
- **Tests post-S69 :** #21, #22 = post-S70/S71
- **Tests deja couverts :** #9, #10 = 0j

### Verdict tests

Les 20 tests applicables a S69 sont ecrivables. Les tests Factory (#1-#8, #11) sont du ressort de S67-S68 et ne doivent PAS etre portes par S69. Les tests Babel (#13-#18, #20) totalisent ~4j, ce qui est coherent avec le scope sprint si les reviews et le task mock sont retires.

---

## 7. Dependances S67-S68 — risque cascade {#7-dependances-s67-s68}

### Chaine de dependances

```
S65 (contrat public)
 └→ S66 (durabilite blobs)
     └→ S67 (Factory foundation)
         └→ S68 (broker/preview/publish gate)
             └→ S69 (Babel canari)
```

### Analyse de risque

| Si... glisse | Impact sur S69 | Mitigation |
|-------------|---------------|------------|
| S65 glisse | FAIBLE. S65 = contrat public, docs, wording. Babel ne depend pas des docs mais du code. | Babel peut demarrer en S69 meme si S65 a un carry cosmetic |
| S66 glisse | MOYEN. Les blobs disparaissent au restart. Le canari fonctionne en session unique mais ne survit pas au redemarrage. | Accepter le risque : le canari est un test de quelques heures, pas de 24h. OU re-deploy apres restart |
| S67 glisse | **CRITIQUE.** Factory n'existe pas, Babel ne peut pas etre "generee par Factory". | **Fallback :** generer Babel a la main (violant le contrat doc §9) OU reporter S69 a S70 |
| S68 glisse | **HAUT.** Pas de broker diff, pas de preview gate, pas de publish-check, pas de feed auto. | **Fallback partiel :** deployer Babel via `deploy-from-repo` direct (sans preview Factory ni evidence pack) |
| S67+S68 glissent | **BLOQUANT.** Babel canari perd tout son sens de "premiere app Factory". | **Decision PO :** Reporter Babel a apres Factory OU accepter une Babel "a la main" comme proto-canari |

### Probabilite de glissement

Factory (S67+S68) est un projet de 2 sprints complets pour un systeme qui n'a AUCUN code existant. La probabilite de retard est **HAUTE**.

Arguments :
- Factory doit generer des repos apps (scaffolding, templates, variables)
- Factory doit avoir un broker local (operations privilegiees, git, disk)
- Factory doit avoir une UI shell `/factory` (React, routes, composants)
- Factory doit valider les manifests, les bridge methods, les secrets
- Factory doit integrer le deploy (preview, publish-check)
- Tout cela est du code neuf, pas du refactoring

**Recommandation :** Prevoir un plan B explicite pour S69 :

- **Plan A :** Factory S67+S68 livrees → Babel canari Factory-generated.
- **Plan B :** Factory S67 livree, S68 partiel → Babel canari avec scaffold Factory + deploy-from-repo direct (sans broker diff ni publish gate).
- **Plan C :** Factory retardee → Babel canari "a la main" comme proto-canari, labellise comme tel (pas "premiere app Factory" mais "prototype Babel pour valider le bridge"). Factory rejoint en S70.

---

## 8. Comparaison avec S69 original — scope creep ou synergie {#8-comparaison-s69-original}

### S69 original (doc S68-S69 research, section 8.2)

5 phases :
- Phase A : Checklist prerequisites + invite mechanism (endpoint HTTP invite, onboarding update)
- Phase B : Installeur cross-platform teste (VM propre, NSIS/.deb/.dmg)
- Phase C : Feedback collector integre (Ideas Hub deploye, export logs, rapport de bug)
- Phase D : Scenarios de test guides (document testeur, checklist parcours)
- Phase E : Analyse go/no-go (collecte retours, decision)

### S69 revise (doc Factory/Babel, section 10)

Livrables :
- Domain pack Babel
- App babel-reader generee par Factory
- Fixtures multilingues
- Source manifests
- Storage local
- Reviews minimales
- Provenance visible
- Browse visible
- Pilote ferme

### Verdict : c'est du SCOPE CREEP, mais avec une bonne raison

Le S69 original avait un scope clair et auto-suffisant : tester l'infrastructure (install, join, browse, restart, stabilite) avec des testeurs reels. C'est un sprint d'integration/qualification.

Le S69 revise ajoute Babel canari, qui est un sprint de DEVELOPPEMENT applicatif. Ce sont deux types d'activites differentes :

| Activite | S69 original | S69 revise |
|----------|-------------|------------|
| Code applicatif nouveau | 0j | ~5-8j |
| Tests d'infra (VM, install) | ~3j | ~2j (reduit) |
| Invite/onboarding | ~2j | ~1j (reduit) |
| Feedback/guides | ~2j | ~1j (reduit) |
| Go/no-go | ~1j | ~1j |

Le S69 revise ne fait pas les deux -- il fait Babel ET un pilote reduit. L'infrastructure pilote (installeur VM, invite flow, export logs, scenarios guides) est compressee.

### La bonne raison

Babel canari donne un OBJET REEL a tester pendant le pilote. Sans Babel, les testeurs voient Protocol Explorer (un outil dev) et Ideas Hub (un formulaire). Avec Babel, ils voient une app "utilisateur" qui lit des textes multilingues, affiche une provenance, utilise le bridge. C'est un meilleur test de la plateforme.

### Recommandation

**Separer les livrables en 2 phases distinctes au sein de S69 :**

- **Phases A-C (6j) :** Babel canari = app generee/deployee + tests d'acceptance
- **Phases D-E (4j) :** Pilote ferme = invite 2-3 testeurs, scenarios, go/no-go

Si le temps manque, le pilote ferme peut glisser a S70 Phase A. Babel canari seul reste un deliverable valide (prouve que Factory fonctionne et que le bridge est fonctionnel).

---

## 9. Verdict et recommandations {#9-verdict}

### Realisme global

| Dimension | Verdict | Confidence |
|-----------|---------|------------|
| Bridge methods | **COMPLET**, 11/11 existent | HIGH |
| Storage pour Babel | **FONCTIONNEL** en local, pas replique | HIGH |
| Source manifests | **JSON dans storage**, schema applicatif | HIGH |
| Publish path | **3 bloqueurs Factory**, 2 gaps protocole | HIGH |
| Tests d'acceptance | **ECRIVABLES** mais 11/22 dependent de S67-S68 | HIGH |
| Dependances S67-S68 | **RISQUE HAUT** de glissement | MEDIUM |
| Scope S69 | **SCOPE CREEP** gerable si scope cuts appliques | HIGH |

### Bloqueurs durs

1. **Factory n'existe pas.** Tout le publish path (etapes 1-3) depend de Factory S67-S68.
2. **Feed ReleasePublished n'est pas auto-insere dans deploy-from-repo.** Gap a combler en S67-S68.
3. **`node_id` obligatoire dans SBFB.json.** Bloque les templates portables. Fix prevu S67.

### Scope cuts recommandes pour S69

1. **Retirer reviews minimales.** Reporter a S74. Economise ~1.5j et simplifie le schema storage.
2. **Retirer task traduction mock.** Reporter a S74. Le mock ne prouve rien que les fixtures ne prouvent deja.
3. **Garder le pilote ferme mais l'alleger.** 2 testeurs au lieu de 3. Scenarios reduits a 4 (install, browse Babel, provenance verify, restart). Pas de test 24h pour le canari.
4. **Pre-generer les fixtures editoriales en S68.** Les 3 textes + 5 langues sont prepares en amont, pas en S69.

### Plan de contingence

- **Si S67 livre mais S68 pas complete :** Babel est generee par Factory scaffold (S67) et deployee via `deploy-from-repo` direct. Pas de broker diff ni preview gate. Pas de feed auto. Canari Browse + provenance uniquement.
- **Si S67 retardee :** Babel est codee a la main comme proto-canari. Le label est "Babel proto-canari bridge validation" et non "premiere app Factory". Factory revient en S70.
- **Si S66 retardee (blobs volatils) :** Le canari fonctionne en session unique. Re-deploy apres chaque restart du daemon. Acceptable pour 2-3 heures de test.

### Feuille de route des gaps a combler AVANT S69

| Gap | Sprint cible | Effort | Priorite |
|-----|-------------|--------|----------|
| Factory module minimal (scaffold + SBFB.json v2) | S67 | Sprint complet | P0 |
| `node_id` optionnel dans SBFB.json / deploy | S67 | 0.5j | P0 |
| Feed `ReleasePublished` auto dans deploy-from-repo | S67-S68 | 1j | P1 |
| Broker diff + preview + publish gate | S68 | Sprint complet | P1 |
| Durabilite blobs (MemStore → persist) | S66 | Sprint complet | P0 |
| Domain pack Babel (fixtures editoriales) | S68 | 1j | P2 |
| Evidence pack / proof-pack CLI | S68 | Prevu dans S68 research | P1 |

### Conclusion

Le scope MVP Babel canari est **techniquement faisable en 1 sprint** si les prerequis S66-S68 sont livres. Le bridge est complet (11/11 methodes). Le storage local suffit pour le canari. Le publish path a 3 bloqueurs durs (Factory) qui ne sont pas du ressort de S69.

Le risque principal n'est pas technique mais sequentiel : Factory est un projet de 2 sprints neuf, et tout retard cascade directement sur Babel. Le plan B (Babel a la main) doit etre explicitement accepte par le PO comme fallback credible.

Avec les scope cuts recommandes (pas de reviews, pas de task mock, fixtures pre-generees), le scope est de ~5.5j sur 10j disponibles, ce qui laisse une marge saine pour l'integration et la discipline de commit.
