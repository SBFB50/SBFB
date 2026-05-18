# Audit des Gates Factory G0-G10

Date : 2026-05-18
Source : `.planning/research/s65_s75_factory_babel_canary_research.md` §11
Auditeur : Claude (research agent, session fraiche)

---

## 0. Rappel des 10 gates

| Gate | Nom | Criteres cles |
|------|-----|---------------|
| G0 | Classification app | domaine, risque donnees, bridge methods, network needs, compute needs |
| G1 | Scope | MVP borne, non-goals explicites, no Babel complet |
| G2 | Template | template id/version, hash, lockfile |
| G3 | Manifest | schema v2 valide, no node_id, bridge allowlist |
| G4 | Diff | preview obligatoire, approbation utilisateur |
| G5 | Sandbox | canonicalize, prefix check, symlink deny, no shell depuis iframe |
| G6 | Secrets/deps | scan secrets, lockfile, SBOM si publish |
| G7 | Preview | iframe sandbox, CSP, no external fetch par defaut |
| G8 | Provenance | factory.provenance.json, generator version, template hash, variables hash, source commit |
| G9 | Publish | repo HTTPS, commit 40 hex, artifact hash, provenance, Browse, feed |
| G10 | Review | sprint review, verdict PASS, evidence pack |

---

## 1. Testabilite automatique — gate par gate

### G0 — Classification app

**Binaire ou subjectif ?** SUBJECTIF. La classification du domaine
("language", "utility", "data") est un choix humain. Le risque donnees
est une evaluation humaine.

**Automatisable ?** Partiellement. On peut verifier que le champ
`category` est dans un enum connu, que `bridge.methods` est un
sous-ensemble du `BridgeMethodSchema` (code existant dans
`web/src/bridge/protocol.ts` ligne 19-39), que `network_needs` et
`compute_needs` sont dans un enum. Mais la pertinence du choix
(`category: "language"` est-il correct ?) est humaine.

**Test envisageable :**
- Rust : struct `AppClassification` avec validation Serde + enum
  contraints. Test `validate_classification_rejects_unknown_category`.
- Vitest : import `BridgeMethodSchema` et verifier que chaque methode
  declaree est dans le schema.

**Code existant reutilisable :**
- `BridgeMethodSchema` dans `web/src/bridge/protocol.ts` — enum Zod
  avec liste exhaustive des methodes autorisees.
- `bridge/__tests__/protocol.test.ts` — test "rejects unknown method"
  (ligne 39-47).

**LOC estime :** ~80 LOC (struct + validation + 4 tests).

**Verdict : Semi-automatique.** Le format est testable, la semantique
ne l'est pas. Accepter comme gate checklist hybride (struct validee +
checkbox humaine pour le domaine).

---

### G1 — Scope

**Binaire ou subjectif ?** SUBJECTIF. "MVP borne" et "non-goals
explicites" sont des jugements humains. "No Babel complet" est
specifique a Babel et ne se generalise pas.

**Automatisable ?** Non pour le fond. On peut verifier l'existence
d'un fichier `SCOPE.md` ou d'une section "Non-goals" dans le
manifeste, mais pas la qualite du contenu.

**Test envisageable :**
- Shell/CI : verifier que le repo genere contient un fichier
  `.planning/active/sprint01_plan.md` (artefact sprint, cf. §6 du
  research).
- Rust : verifier que `factory.audit.jsonl` contient une entree
  `scope_validated`.

**Code existant reutilisable :** Aucun code existant pour ce gate.
Le pattern kickoff/plan du workflow SBFB (`docs/claude/README.md`
§2.1-§2.2) est un precedent conceptuel mais pas du code.

**LOC estime :** ~30 LOC (existence check + JSONL entry parse).

**Verdict : Gate humain avec trace automatique.** Factory peut logger
dans `factory.audit.jsonl` que l'utilisateur a approuve le scope, mais
la decision reste humaine.

---

### G2 — Template

**Binaire ou subjectif ?** BINAIRE. Template id/version est un string
fixe. Hash est un BLAKE3 calculable. Lockfile est un fichier presence.

**Automatisable ?** Oui, integralement.

**Test envisageable :**
- Rust : `validate_template_lock(lock_path)` verifie :
  - `template_id` non vide
  - `template_version` semver valide
  - `template_hash` == BLAKE3(contenu template)
  - fichier `factory.template.lock` present
- 5 tests : happy path, hash mismatch, missing lock, invalid version,
  empty template_id.

**Code existant reutilisable :**
- `blake3_hash()` dans `nexus-core-rs/src/crypto.rs` — la meme
  fonction utilisee pour artifact_hash.
- Pattern provenance hash dans `nexus-coordinator-rs/src/provenance.rs`
  (`provenance_blake3_hex`).

**LOC estime :** ~120 LOC (struct TemplateLock + validate + 5 tests).

**Verdict : Entierement automatisable.** Aucun blocage.

---

### G3 — Manifest

**Binaire ou subjectif ?** BINAIRE. Schema v2 validable par struct
Serde. Absence de `node_id` = champ absent ou deprecie. Bridge
allowlist = sous-ensemble de l'enum connu.

**Automatisable ?** Oui, integralement.

**Test envisageable :**
- Rust : `validate_manifest_v2(json_str)` verifie :
  - `schema_version == 2`
  - `node_id` absent (ou present avec warning de deprecation)
  - `bridge.methods` est un sous-ensemble de `ALLOWED_BRIDGE_METHODS`
  - `name`, `version`, `license` non vides
  - `tech.entry_point` existe et est `index.html`
- 8 tests : happy path, unknown bridge method, present node_id,
  invalid schema_version, empty name, missing entry_point, non-AGPL
  license (warning), unknown category.

**Code existant reutilisable :**
- `SbfbJson` dans `deploy.rs` ligne 494-498 — struct Serde pour
  SBFB.json (actuellement minimal : `node_id` + `version`).
  **A etendre** en `SbfbJsonV2` avec tous les champs §7 du research.
- `BridgeMethodSchema` Zod dans `protocol.ts` — la source de verite
  pour la liste des methodes. Il faudra un mirror Rust (un enum ou
  un `&[&str]`).
- `read_sbfb_json()` dans `deploy.rs` ligne 501-507 — parsing + erreur.

**Point d'attention :** Le manifest v2 du research §7 ajoute des champs
(`display_name`, `description`, `category`, `license`, `bridge`,
`tech`, `requirements`). L'actuel `SbfbJson` ne connait que `node_id`
et `version`. Il faut migrer `deploy.rs` pour accepter v2 ET rester
compatible avec les exemples existants (`sbfb-explorer`, `sbfb-ideas`
qui ont `node_id: "PLACEHOLDER"`).

**LOC estime :** ~200 LOC (struct v2 + validation + 8 tests + compat
v1).

**Verdict : Entierement automatisable.** C'est le gate le plus
critique — il protege le wire format.

---

### G4 — Diff

**Binaire ou subjectif ?** HYBRIDE. "Preview obligatoire" est binaire
(le diff a ete affiche). "Approbation utilisateur" est une action
humaine (click/Enter).

**Automatisable ?** La generation du diff est automatisable. L'acte
d'approbation ne l'est pas — c'est une interaction UI.

**Test envisageable :**
- Rust : `generate_diff(template_files, variables) -> Vec<DiffEntry>`
  + serde pour le JSON.
- Vitest : composant React `/factory` affiche le diff, mock du click
  "Approve".
- Integration : verifier que `factory.audit.jsonl` contient une entree
  `diff_approved` avec timestamp et hash du diff.

**Code existant reutilisable :** Aucun diff engine dans le codebase
actuel. Le pattern le plus proche est le `zip_directory` +
`add_dir_to_zip` dans `deploy.rs` qui itere les fichiers — meme
pattern d'enumeration pour generer un diff.

**LOC estime :** ~150 LOC (diff struct + generation + JSONL trace +
3 tests + UI component est hors scope gate).

**Verdict : Semi-automatique.** La trace est automatique, l'acte
humain ne l'est pas. Accepter comme gate checklist hybride.

---

### G5 — Sandbox

**Binaire ou subjectif ?** BINAIRE. Path canonicalize, prefix check,
symlink deny sont tous des conditions booleennes. "No shell depuis
iframe" est une propriete de l'architecture (CSP + sandbox attribute).

**Automatisable ?** Oui, integralement.

**Test envisageable :**
- Rust : `validate_workspace_safety(workspace_path, output_files)`
  verifie :
  - Chaque fichier de sortie est sous `workspace_path` apres
    canonicalize
  - Aucun symlink dans l'arbre
  - Pas de `..` dans les chemins
  - Pas de path contenant des caracteres dangereux (`\`, null byte)
- 7 tests : happy path, path traversal `../`, symlink present, absolute
  path `/etc`, backslash path, null byte, path outside workspace.

**Code existant reutilisable — TRES RICHE :**
- `validate_zip_path()` dans `blob_serve.rs` ligne 181-195 — **meme
  logique exacte** (rejet `..`, `/`, `\`, vide). Reutilisable tel quel
  pour valider les fichiers de sortie Factory.
- `add_dir_to_zip()` dans `deploy.rs` ligne 522-560 — contient deja
  le check symlink (`path.is_symlink()` → skip) et le check `..`
  (`name.contains("..")` → skip). Pattern reutilisable.
- Tests existants dans `blob_serve.rs` lignes 310-338 : path traversal,
  absolute path, backslash, empty, normal paths — **couvrent deja les
  scenarios**.
- `BLOB_SERVE_CSP` dans `blob_serve.rs` ligne 277 — CSP complet
  incluant `sandbox allow-scripts`, `connect-src 'none'`,
  `worker-src 'none'`, `frame-src 'none'`, `form-action 'none'`.
  C'est la reference pour G7 aussi.

**LOC estime :** ~60 LOC (facade `validate_factory_output_safety` +
3 tests supplementaires, le reste existe deja).

**Verdict : Entierement automatisable.** Code existant couvre 80%
du besoin. Factory peut reutiliser `validate_zip_path` directement.

---

### G6 — Secrets/deps

**Binaire ou subjectif ?** BINAIRE pour le scan secrets (pattern
matching). BINAIRE pour la presence de lockfile. SEMI-BINAIRE pour
SBOM (presence d'un fichier, contenu non verifie semantiquement).

**Automatisable ?** Oui pour le scan basique.

**Test envisageable :**
- Rust : `scan_secrets(files: &[(String, Vec<u8>)])` avec patterns
  regex pour :
  - `PRIVATE KEY` (RSA/EC/Ed25519)
  - `sk_live_`, `pk_live_` (Stripe)
  - `ghp_`, `gho_`, `github_pat_` (GitHub tokens)
  - `AKIA` (AWS access key)
  - `-----BEGIN.*PRIVATE` (generic PEM)
  - `.env` file avec `API_KEY=`, `SECRET=`, `TOKEN=`, `PASSWORD=`
- Test lockfile presence : `package-lock.json` ou `Cargo.lock` ou
  `pnpm-lock.yaml`.
- Test SBOM si publish : fichier `sbom.json` ou `sbom.spdx` present.
- 6 tests : happy path clean, secret detected, .env detected, missing
  lockfile, SBOM present, SBOM absent sur publish.

**Code existant reutilisable :**
- Supply chain CI dans `.github/workflows/supply-chain.yml` — pattern
  conceptuel (cargo-deny, pip-audit, npm audit).
- Aucun scan secrets Rust dans le codebase actuel. C'est du code
  nouveau.

**LOC estime :** ~250 LOC (secret patterns + scanner + lockfile check +
SBOM check + 6 tests). Plus si on veut un scanner configurable.

**Verdict : Automatisable mais effort significant.** Le scan secrets
basique (regex) est faisable. Un scan semantique (AST-aware) est hors
scope — utiliser un outil externe comme gitleaks/trufflehog en CI
serait plus robuste pour le long terme.

---

### G7 — Preview

**Binaire ou subjectif ?** BINAIRE pour les headers (CSP, sandbox).
SEMI-BINAIRE pour "no external fetch" (le CSP `connect-src 'none'`
l'impose deja, mais l'app pourrait contenir des `<img src=...>`
externes que le CSP ne bloque pas si `default-src 'self'`).

**Automatisable ?** Oui pour la verification des headers.

**Test envisageable :**
- Rust : `validate_preview_headers(response_headers)` verifie :
  - CSP contient `connect-src 'none'`
  - CSP contient `sandbox allow-scripts`
  - COOP `same-origin`
  - COEP `require-corp`
  - X-Content-Type-Options `nosniff`
- Vitest/Playwright : ouvrir l'app dans un iframe, verifier que les
  headers sont presents.
- 5 tests : headers present, CSP missing, COOP missing, COEP missing,
  sandbox attribute present.

**Code existant reutilisable — COMPLET :**
- `BLOB_SERVE_CSP` dans `blob_serve.rs` — **la constante existe deja**.
- `BLOB_SERVE_COOP` et `BLOB_SERVE_COEP` dans `blob_serve.rs` —
  **les constantes existent deja**.
- HTTP handler dans `http.rs` ligne 473-482 — les headers sont deja
  injectes sur chaque reponse blob-serve.
- Test `blob_serve_rejects_path_traversal` dans `http.rs` ligne 2935 —
  verifie deja le comportement de rejet.
- Tests CSP dans `http.rs` ligne 2794-2797 — verifient deja la
  presence du header.

**LOC estime :** ~40 LOC (facade de validation + 2 tests
supplementaires, le reste existe).

**Verdict : Entierement automatisable.** Le code existant couvre 90%
du besoin. Factory doit simplement reutiliser le meme chemin blob-serve
que le deploy normal.

---

### G8 — Provenance

**Binaire ou subjectif ?** BINAIRE. Chaque champ est verifiable :
generator version est un string, template hash est BLAKE3,
variables hash est BLAKE3, source commit est hex-40.

**Automatisable ?** Oui, integralement.

**Test envisageable :**
- Rust : `validate_factory_provenance(json_str)` verifie :
  - Fichier `factory.provenance.json` present
  - `generator` non vide, format `factory/X.Y.Z`
  - `template_hash` == BLAKE3 64-hex
  - `variables_hash` == BLAKE3 64-hex
  - `source_commit` hex-40 (si repo public)
  - Schema coherent (memes champs que le template lock)
- 6 tests : happy path, missing generator, invalid template_hash,
  missing variables_hash, commit_sha non-hex, missing file.

**Code existant reutilisable :**
- `ProvenanceRecord` dans `provenance.rs` — struct + generation +
  verification. **Pattern a suivre** pour `FactoryProvenanceRecord`.
- `generate_provenance()` dans `provenance.rs` ligne 30-57 — meme
  pattern (canonical bytes + signature + hash).
- `verify_provenance()` dans `provenance.rs` ligne 59-89 — meme
  pattern de verification.
- `is_hex_exact()` dans `public_feed.rs` ligne 186-188 — utilitaire
  reutilisable.
- `provenance_blake3_hex()` dans `provenance.rs` ligne 95-99 — hash
  du record.

**LOC estime :** ~180 LOC (struct FactoryProvenance + validate +
generation + 6 tests).

**Verdict : Entierement automatisable.** Le pattern provenance est
bien rode dans le codebase.

---

### G9 — Publish

**Binaire ou subjectif ?** BINAIRE. Chaque critere est une condition
booleenne : repo HTTPS, commit hex-40, artifact hash present,
provenance present, Browse entry creee, feed entry creee.

**Automatisable ?** Oui, integralement.

**Test envisageable :**
- Rust integration : `test_publish_gate_roundtrip()` verifie :
  - `repo_url.starts_with("https://")` (existe dans
    `validate_feed_operation`)
  - `is_valid_sha(commit)` (existe dans `deploy.rs`)
  - `artifact_hash` hex-64 (existe dans `validate_feed_operation`)
  - `provenance_hash` present et hex-64
  - Browse entry indexee apres deploy
  - Feed entry `ReleasePublished` creee
- 7 tests : happy path, HTTP repo rejected, invalid SHA, missing
  provenance, Browse absent, feed absent, artifact hash invalid.

**Code existant reutilisable — QUASI-COMPLET :**
- `deploy_from_repo()` dans `deploy.rs` — **c'est exactement le flow
  de publish**. Lignes 64-262 : clone, validate, zip, hash, provenance,
  blob store, Browse, gossip.
- `is_valid_sha()` dans `deploy.rs` ligne 391-393 — SHA validation.
- `is_repo_public()` dans `deploy.rs` ligne 395-404 — repo public
  check.
- `validate_feed_operation()` dans `public_feed.rs` lignes 204-249 —
  **verifie deja** HTTPS, hex-40, hex-64, provenance_hash si
  is_open_source.
- `publish_announcement()` dans `deploy.rs` lignes 316-378 — Browse
  entry creation.
- `feed_insert()` dans `feed_sync.rs` lignes 445-500 — feed entry
  creation.

**LOC estime :** ~60 LOC (test d'integration + validation facade).
Le code de publish lui-meme existe deja.

**Verdict : Entierement automatisable.** La quasi-totalite du code
existe. Le seul gap est l'appel automatique `feed_insert` apres
`deploy_from_repo` (actuellement deux operations separees — cf.
research §5 "Ce qui manque").

---

### G10 — Review

**Binaire ou subjectif ?** SUBJECTIF. "Sprint review" est un processus
humain. "Verdict PASS" est un jugement. "Evidence pack" est une
collection de fichiers dont la completude peut etre verifiee mais pas
la qualite.

**Automatisable ?** Partiellement.

**Test envisageable :**
- Shell/CI : verifier que le dossier `evidence/` ou `proof-pack/`
  contient au minimum :
  - `factory.provenance.json`
  - `factory.template.lock`
  - `factory.audit.jsonl`
  - `verification.md` (ou `sprint01_verification.md`)
  - screenshot ou capture
- Rust : parser `factory.audit.jsonl` et verifier qu'il contient les
  10 events attendus (un par gate G0-G9).

**Code existant reutilisable :**
- Le pattern `verification.md` du workflow sprint (`docs/claude/README.md`
  §2.3) — precedent conceptuel.
- Le pattern `audit_plan.md` + `audit_findings.md` — meme pattern pour
  l'evidence app.

**LOC estime :** ~80 LOC (existence checks + JSONL completeness +
3 tests).

**Verdict : Semi-automatique.** La completude est testable, la qualite
ne l'est pas.

---

## 2. Code existant reutilisable — synthese

| Gate | Code existant | Reutilisation | Gap |
|------|--------------|---------------|-----|
| G0 | `BridgeMethodSchema` (Zod) | Validation methodes bridge | Mirror Rust de l'enum methodes |
| G1 | Aucun | — | Gate humain pur |
| G2 | `blake3_hash()`, `provenance_blake3_hex()` | Hash template | Struct TemplateLock |
| G3 | `SbfbJson`, `BridgeMethodSchema` | Parsing + validation | Extension v2 du manifest |
| G4 | Pattern `add_dir_to_zip` | Enumeration fichiers | Diff engine |
| G5 | `validate_zip_path()`, `add_dir_to_zip` symlink check | **Direct** | Facade workspace |
| G6 | Supply chain CI patterns | Conceptuel | Scanner secrets Rust |
| G7 | `BLOB_SERVE_CSP/COOP/COEP`, tests HTTP | **Direct** | Quasi-rien |
| G8 | `ProvenanceRecord`, `generate/verify_provenance` | Pattern | Struct FactoryProvenance |
| G9 | `deploy_from_repo`, `validate_feed_operation`, `feed_insert` | **Direct** | Feed auto apres deploy |
| G10 | Workflow sprint pattern | Conceptuel | Evidence pack check |

**Proportion de code reutilisable :** ~60-70% du code necessaire
existe deja dans le codebase. Les gates G5, G7, G9 sont quasiment
couvertes. Les gaps principaux sont G0 (mirror Rust du BridgeMethodSchema),
G3 (manifest v2), G6 (secret scanner), G8 (struct factory provenance).

---

## 3. Ordre des gates — analyse des dependances

### Graphe de dependances

```
G0 (Classification) → G1 (Scope) → G2 (Template) → G3 (Manifest)
                                         ↓
                                    G4 (Diff) → G5 (Sandbox) → G6 (Secrets)
                                                                     ↓
                                                               G7 (Preview) → G8 (Provenance) → G9 (Publish) → G10 (Review)
```

### L'ordre actuel est-il correct ?

**G0 → G1 : OUI.** La classification precede le scope — on ne peut
pas borner le MVP sans savoir dans quel domaine on est.

**G1 → G2 : OUI.** Le scope guide le choix de template.

**G2 → G3 : OUI.** Le template genere le manifest. G3 valide ce que
G2 a produit.

**G3 → G4 : OUI.** Le manifest doit etre valide avant de montrer le
diff a l'utilisateur — sinon il approuve un artefact invalide.

**G4 → G5 : DISCUTABLE.** L'ordre actuel montre le diff (G4) puis
verifie la sandbox (G5). C'est logique si on considere que G5 valide
les fichiers generes apres approbation. Mais il y a un argument pour
inverser : verifier la securite AVANT de montrer quoi que ce soit
a l'utilisateur.

**Recommandation : garder G4 avant G5.** Raison : G4 est la preview
du diff source (texte), G5 est la validation des fichiers ecrits sur
disque. Le diff peut contenir un path traversal sans que ca soit un
risque tant que les fichiers ne sont pas materialises. L'ordre est :
voir → approuver → materialiser → verifier securite. C'est le bon
flux.

**G5 → G6 : OUI.** Sandbox d'abord (pas d'ecriture hors workspace),
puis scan du contenu.

**G6 → G7 : OUI.** Verifier les secrets avant de lancer la preview
iframe — un secret dans un fichier servi via blob-serve serait
expose.

**G7 → G8 : OUI.** La preview confirme que l'app fonctionne. Ensuite
on genere la provenance de ce qui a ete previewed.

**G8 → G9 : OUI.** La provenance precede le publish — on ne peut pas
publier sans provenance.

**G9 → G10 : OUI.** Le publish precede la review — la review inspecte
le resultat du publish.

### Verdict sur l'ordre

**L'ordre G0-G10 est correct.** Le seul point de discussion est G4/G5
mais l'ordre propose est le bon flux (voir avant materialiser).

### Dependance implicite non documentee

G9 (Publish) depend de G3 (Manifest) ET de G8 (Provenance) :
- `deploy_from_repo` lit `SBFB.json` (G3)
- `deploy_from_repo` genere la provenance (G8)
- `deploy_from_repo` fait le publish (G9)

C'est coherent — G9 est le dernier gate automatique et il rassemble
tout. Mais il serait utile de documenter cette dependance transitive.

---

## 4. Gates manquantes

### 4.1 Taille maximale de l'archive

**Manque : OUI.**

Le code existant a deja deux limites :
- `MAX_DEPLOY_BYTES = 100 MB` (deploy.rs ligne 27) pour le zip upload
- `MAX_CLONE_BYTES = 500 MB` (deploy.rs ligne 28) pour le git clone
- `DEFAULT_MAX_DECOMPRESSED_BYTES = 100 MB` (blob_serve.rs ligne 38)
  pour le decompresse

Mais Factory n'a pas de gate explicite sur la taille de l'archive
generee. Un template mal ecrit pourrait generer un zip de 90 MB qui
passe le deploy mais est impraticable pour la distribution P2P.

**Recommandation :** Ajouter dans G5 ou G6 :
- Taille maximale du zip genere : 10 MB par defaut (configurable par
  domain pack).
- Nombre maximal de fichiers dans le zip : 500 (eviter les zip bombs
  par nombre d'entrees).

### 4.2 Temps de build maximum

**Manque : NON (pour le MVP).**

Factory MVP genere des apps statiques (pas de build step). Le template
produit directement les fichiers. Il n'y a pas de `npm run build` ou
`cargo build` dans le chemin Factory MVP.

Si Factory evolue vers des templates avec build step (React, WASM),
alors un timeout sera necessaire. C'est un gate post-MVP.

### 4.3 Nombre maximal de fichiers

**Manque : OUI.** Voir §4.1 ci-dessus.

### 4.4 Validation des types MIME

**Manque : NON.**

`detect_content_type()` dans `blob_serve.rs` gere deja la detection
MIME pour les fichiers servis. Factory n'a pas besoin d'un gate MIME
supplementaire — le blob-serve s'en charge au serving time.

Cependant, il serait utile de **rejeter les fichiers binaires
inattendus** dans G5 (un .exe ou .dll dans une app web est suspect).
C'est un sous-cas de G6 (secrets/deps).

### 4.5 Verification des imports bridge (runtime compliance)

**Manque : OUI, c'est une lacune significative.**

G3 valide la declaration des methodes bridge dans le manifest
(`bridge.methods: ["storage_get", ...]`). Mais rien ne verifie que
l'app n'appelle pas des methodes non declarees a runtime.

Le code existant dans `useBridge.ts` fait la validation du schema Zod
au moment de la reception du message (`BridgeRequestSchema.safeParse`),
ce qui rejette les methodes inconnues. Mais il ne verifie pas que la
methode appelee est dans l'allowlist du manifest de l'app.

**Recommandation :** Ajouter un gate G3.5 ou enrichir G3 :
- Static : scanner le code source genere (grep des appels
  `bridge.request(method, ...)`) et verifier que chaque `method` est
  dans `bridge.methods` du manifest.
- Runtime : le host shell charge `SBFB.json`, extrait
  `bridge.methods`, et rejette tout `BridgeRequest` dont la methode
  n'est pas dans la liste.

Le runtime check est plus robuste (l'app peut construire les noms de
methodes dynamiquement). Le static check est un filet supplementaire.

**LOC estime :** ~100 LOC pour le runtime check dans `useBridge.ts`
(lire manifest, comparer methode).

### 4.6 Validation de l'entrypoint

**Manque : PARTIELLEMENT.**

G3 verifie `tech.entry_point: "index.html"`. Mais le deploy existant
ne verifie que la presence de `index.html` dans le zip
(`validate_zip` dans `deploy.rs` ligne 602-609).

Il faudrait verifier que l'`index.html` est du HTML valide (au minimum
un doctype et un `<html>` tag) et qu'il inclut le SDK bridge
(`sbfb-bridge.js` ou equivalent).

**Recommandation :** Ajouter dans G3 :
- `index.html` present ET contient `<html` (pas un fichier vide)
- Presence de `sbfb-bridge.js` dans l'archive (ou import equivalent)

### 4.7 Deduplication / idempotence

**Manque : OUI.**

Si Factory regenere la meme app deux fois avec les memes variables,
le resultat devrait etre identique (determinisme, §6 du research).
Aucun gate ne verifie cela.

**Recommandation :** Ajouter dans G8 :
- `variables_hash` est deterministe (memes variables → meme hash)
- Si `variables_hash` + `template_hash` identiques a un publish
  precedent, Factory avertit l'utilisateur (pas un blocage, un
  warning).

---

## 5. Gates redondantes

### 5.1 G5 (Sandbox) vs G7 (Preview)

**Question :** G5 et G7 sont-elles distinctes ?

**Reponse : OUI, elles sont distinctes et complementaires.**

- **G5** valide les fichiers generes **sur disque** : pas de path
  traversal, pas de symlink, pas d'ecriture hors workspace. C'est
  une validation **statique** du filesystem.
- **G7** valide le **rendu HTTP** : les headers CSP/COOP/COEP sont
  presents quand l'app est servie dans un iframe. C'est une validation
  **runtime** du serving.

Un fichier peut passer G5 (chemin safe, pas de symlink) mais echouer
G7 (le serveur blob-serve oublie le header CSP). Et inversement, un
path traversal ne serait pas detecte par G7 (les headers sont
corrects sur la reponse 404).

**Verdict : Pas de redondance.** Les deux gates couvrent des couches
differentes (FS vs HTTP).

### 5.2 G2 (Template) vs G8 (Provenance)

**Question :** G2 et G8 se recoupent-elles sur le hash ?

**Reponse : OUI, partiellement, mais c'est intentionnel.**

- **G2** verifie que le template utilise est correctement identifie
  (id, version, hash dans le lockfile). C'est un gate **build-time**.
- **G8** enregistre le template hash dans la provenance Factory. C'est
  un gate **attestation-time**.

Le template hash apparait dans les deux :
- G2 : `factory.template.lock.template_hash == BLAKE3(template)`
- G8 : `factory.provenance.json.template_hash == ...`

Ce n'est pas de la redondance — c'est de la **tracabilite**. G2
verifie que le hash est correct au moment de la generation. G8
enregistre le hash dans un document signe pour que des tiers puissent
le verifier plus tard.

**Recommandation :** Ajouter un check de coherence dans G8 :
`factory.provenance.json.template_hash == factory.template.lock.template_hash`.
Si les deux divergent, c'est un P0.

**Verdict : Recoupement intentionnel, pas de redondance.** Ajouter
un invariant de coherence.

---

## 6. Effort d'implementation

### 6.1 Estimation LOC par gate

| Gate | LOC nouveau | LOC reutilise | Total effectif |
|------|-------------|---------------|----------------|
| G0 | 80 | 20 (BridgeMethodSchema mirror) | 100 |
| G1 | 30 | 0 | 30 |
| G2 | 120 | 30 (blake3_hash) | 150 |
| G3 | 200 | 50 (SbfbJson, protocol.ts) | 250 |
| G4 | 150 | 20 (file enumeration) | 170 |
| G5 | 60 | 80 (validate_zip_path, symlink check) | 140 |
| G6 | 250 | 10 (CI patterns) | 260 |
| G7 | 40 | 100 (CSP/COOP/COEP, tests HTTP) | 140 |
| G8 | 180 | 60 (ProvenanceRecord pattern) | 240 |
| G9 | 60 | 200 (deploy_from_repo, feed_insert) | 260 |
| G10 | 80 | 20 (workflow pattern) | 100 |
| **Total** | **~1250** | **~590** | **~1840** |

### 6.2 Decoupe en phases

Les gates se groupent naturellement en 4 lots :

**Lot 1 — Fondations (S67)** : G2 + G3 + G5
- Template lock, manifest v2, sandbox validation.
- ~540 LOC effectif.
- 1 phase (~Phase B ou C de S67).

**Lot 2 — Broker (S68)** : G0 + G4 + G6 + G7
- Classification, diff preview, secrets scan, preview iframe.
- ~670 LOC effectif.
- 2 phases (~Phase A-B de S68).

**Lot 3 — Provenance + Publish (S68)** : G8 + G9
- Factory provenance, publish gate.
- ~500 LOC effectif.
- 1 phase (~Phase C de S68).

**Lot 4 — Review (S69+)** : G1 + G10
- Scope validation (humain), evidence pack.
- ~130 LOC effectif.
- 1 phase (integree a S69 canari).

### 6.3 Crates touches

| Crate | Gates | Changements |
|-------|-------|-------------|
| `nexus-coordinator-rs` | G2, G3, G8 | Structs Template/Manifest/FactoryProvenance + validation |
| `nexus-shell-daemon` | G5, G9 | Extension deploy.rs pour manifest v2, feed auto |
| `nexus-shell-daemon-core` | G5, G7 | Reutilisation blob_serve, possible facade factory |
| Nouveau crate `nexus-factory` (ou module) | G0, G1, G4, G6, G10 | Broker, diff, secrets, evidence |
| `web/` (React) | G0, G4, G7 | UI /factory, diff preview, classification form |

### 6.4 Implementation incrementale

**OUI, les gates sont implementables incrementalement.**

Chaque gate est independant dans sa validation (bien que l'ordre
d'execution en pipeline soit fixe). On peut implementer :

- S67 : G2 + G3 + G5 (les gates fondation sans UI)
- S68 : G0 + G4 + G6 + G7 + G8 + G9 (le gros du broker)
- S69 : G1 + G10 (gates humains avec le canari Babel)

C'est coherent avec la roadmap §10 du research :
- S67 = Factory Foundation → G2, G3, G5
- S68 = Broker, preview, publish gate → G0, G4, G6, G7, G8, G9
- S69 = Babel canari → G1, G10

---

## 7. Comparaison avec les gates workflow G1-G9

### 7.1 Les deux systemes de gates

Le workflow SBFB (`docs/claude/README.md` §6.9-§6.10) definit des
gates **G1 a G9** (G5 supprime) :

| Gate workflow | Quoi | Quand |
|---------------|------|-------|
| G1 | Design Review Board | Kickoff |
| G2 | Fraicheur artefacts long-life | Session-start |
| G3 | Goal SMART → fail-fast | Kickoff |
| G4 | Rigor signal (audit gate) | Phase review |
| ~~G5~~ | ~~(supprime S24)~~ | — |
| G6 | Memory update | Post-commit |
| G7 | Carry-over discipline | Phase F |
| G8 | Pre-flight factual check | Pre-implementation |
| G9 | Factual research gate | Avant D-decisions |

Les gates **Factory G0-G10** definies dans le research §11 :

| Gate Factory | Quoi | Quand |
|--------------|------|-------|
| G0 | Classification app | Creation |
| G1 | Scope | Creation |
| G2 | Template | Generation |
| G3 | Manifest | Generation |
| G4 | Diff | Pre-apply |
| G5 | Sandbox | Post-generation |
| G6 | Secrets/deps | Post-generation |
| G7 | Preview | Pre-publish |
| G8 | Provenance | Pre-publish |
| G9 | Publish | Publication |
| G10 | Review | Post-publish |

### 7.2 Confusion de nommage

**PROBLEME MAJEUR : les deux systemes utilisent le meme namespace G0-G10
pour des choses completement differentes.**

- "G1" workflow = Design Review Board (scoring D-decisions)
- "G1" Factory = Scope (MVP borne)
- "G3" workflow = Goal SMART → fail-fast checklist
- "G3" Factory = Manifest schema v2
- "G8" workflow = Pre-flight factual evolution check
- "G8" Factory = Provenance record

Ce n'est pas un probleme technique mais un probleme de communication.
Quand quelqu'un dit "G3 a echoue", on ne sait pas s'il parle du
workflow sprint ou du pipeline Factory.

### 7.3 Recommandation de renommage

**Renommer les gates Factory avec un prefixe `FG` (Factory Gate) :**

| Avant | Apres |
|-------|-------|
| G0 | FG0 — Classification |
| G1 | FG1 — Scope |
| G2 | FG2 — Template |
| G3 | FG3 — Manifest |
| G4 | FG4 — Diff |
| G5 | FG5 — Sandbox |
| G6 | FG6 — Secrets |
| G7 | FG7 — Preview |
| G8 | FG8 — Provenance |
| G9 | FG9 — Publish |
| FG10 | FG10 — Review |

Alternative : `F0-F10`. Mais `F` est deja utilise pour "Phase F" dans
le workflow sprint.

**Recommandation : `FG0-FG10`.** C'est sans ambiguite et searchable.

### 7.4 Compatibilite

Les deux systemes de gates sont **orthogonaux** :
- Les gates workflow (G1-G9) regissent le **processus de developpement
  de SBFB** (sprints, commits, audits).
- Les gates Factory (FG0-FG10) regissent la **generation et
  publication d'apps** par le module Factory.

Ils ne se contredisent pas. Le sprint qui implemente FG3 suivra le
workflow G1-G9 habituel. Et les apps generees par Factory (qui passent
FG0-FG10) peuvent elles-memes avoir un workflow sprint interne (si
Factory genere des artefacts `.planning/`).

La seule interaction est FG10 (Review) qui ressemble
conceptuellement a G4 (Rigor signal). FG10 est la review de l'app
generee ; G4 est la review du code SBFB. Ce sont des objets differents
revises par des mecanismes similaires.

---

## 8. Synthese des recommandations

### 8.1 Corrections prioritaires

1. **Renommer G0-G10 en FG0-FG10** pour eviter la collision de
   namespace avec les gates workflow G1-G9. (P0 — confusion
   communication.)

2. **Ajouter une limite de taille archive** dans FG5 ou FG6 : max
   10 MB par defaut, max 500 fichiers. (P1 — zip bomb/DDoS.)

3. **Ajouter la verification bridge runtime** : enrichir FG3 ou creer
   FG3.5 pour que le host shell verifie que chaque methode appelee
   par l'iframe est dans l'allowlist du manifest. (P1 — privilege
   escalation app.)

4. **Ajouter l'invariant de coherence G2/G8** : le template_hash dans
   `factory.provenance.json` doit matcher celui dans
   `factory.template.lock`. (P2 — tracabilite.)

5. **Documenter la dependance transitive G9 → G3 + G8** : le publish
   gate rassemble les produits des gates amont. Le rendre explicite
   dans le pipeline. (P2 — documentation.)

### 8.2 Gates inchangees

Les 10 gates sont toutes pertinentes. Aucune n'est a supprimer. Les
recoupements (G2/G8 sur le hash, G5/G7 sur la securite) sont
intentionnels et couvrent des couches differentes.

### 8.3 Implementation

- ~1840 LOC effectif total (dont ~590 reutilises)
- ~1250 LOC nouveau
- 3 sprints (S67-S69) en 4-5 lots
- Implementation incrementale possible et recommandee
- Le codebase existant couvre 60-70% du besoin grace au deploy path,
  blob-serve, provenance, et bridge protocol deja en place

### 8.4 Tests d'acceptance supplementaires (recommandation)

Au-dela des tests §12 du research, ajouter :
- `factory_rejects_archive_over_10mb`
- `factory_rejects_archive_over_500_files`
- `bridge_host_rejects_method_not_in_manifest_allowlist`
- `factory_provenance_template_hash_matches_lockfile`
- `factory_rejects_binary_exe_in_output`
- `factory_deterministic_same_inputs_same_hash`
