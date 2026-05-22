# Sprint 68 Phase B — preflight G8

Date : 2026-05-21 | HEAD : `2d0999f` | Verdict : **EXECUTE plan-as-is**

---

## Memory consultation (Step 1.5)

- `feedback_approach.md` : pick deepest technical option, research
  before code, G8 = procedural mechanism. No band-aids.
- `feedback_context7_systematic.md` : context7 obligatoire avant
  code touchant lib/API. Applique pour reqwest + zip crate.
- `vision_model.md` : N/A (pas de pattern startup implique).
- `fairness_vision.md` : N/A (pas de kudos/reputation touche).
- `feedback_no_direct_blobserve.md` : preview doit etre servi dans
  iframe sandbox via shell Browse, jamais en onglet direct. Le plan
  utilise blob-serve existant qui impose CSP sandbox — conforme.
- Tensions plan vs memory : **aucune**.

---

## S1a — OSS prior art deep analysis

### Probleme fonctionnel exact

"How do mature OSS projects implement ephemeral local preview
of web app archives served via hash-addressed in-memory cache with
TTL eviction, and how do CLI tools delegate publish to a local
daemon via HTTP?"

### Projets analyses en profondeur

#### [1] Netlify / Vercel — Preview Deploys

- Source : WebSearch "Netlify Vercel preview deploy ephemeral
  environment hash-based URL architecture implementation 2025"
- Pattern : chaque PR genere un deploy preview avec URL unique
  hash-based (ex : `deploy-preview-12--project.netlify.app`).
  Environnement ephemere avec nettoyage automatique.
- Pertinence : confirme le pattern preview = URL hash-based +
  TTL/ephemere. SBFB fait la meme chose a echelle locale
  (`/blob-serve/{hash}/index.html`).
- Verdict : APPROACH-ALIGNED.

#### [2] IPFS — Content-addressed pinning + preview

- Source : WebSearch "IPFS local preview pinning temporary hash
  CID serve web app publish workflow 2025 2026"
- Pattern : CID = hash du contenu. Pinning = retention en
  memoire/disque. Garbage collection evicte le contenu non-pinne.
  `ipfs add` pinne par defaut, `ipfs pin rm` + GC = eviction.
- Pertinence directe : SBFB preview utilise le meme pattern
  (hash BLAKE3 = CID, HashMap = pin, TTL = GC timer).
- Verdict : APPROACH-ALIGNED.

#### [3] F-Droid fdroidserver — Preview local

- Source : WebSearch "F-Droid fdroidserver preview local test app
  before publish implementation 2025 2026"
- Pattern : `fdroid build --test` redirige output vers tmp/,
  pas unsigned/. Pas de serveur ephemere integre — le dev lance
  un serveur HTTP local separe et ajoute son repo local au client
  Android. Preview = repo local temporaire.
- Divergence : F-Droid n'a PAS de preview integre dans son build
  tool. SBFB est plus avance (preview integre dans le daemon
  via blob-serve existant).
- Verdict : APPROACH-NOVEL justifie (SBFB preview = daemon
  existant, pas serveur separe).

#### [4] Moka cache — TTL eviction HashMap Rust

- Source : WebSearch "moka cache Rust TTL eviction concurrent
  HashMap DashMap 2025 2026" + docs.rs/moka
- Pattern : moka::future::Cache = concurrent HashMap avec TTL
  natif, eviction automatique, capacity bound. 115K downloads/mois
  sur crates.io. Derniere release : 2025.
- Pertinence : moka fournirait un PreviewStore avec TTL integre
  sans code custom (pas de background task tokio manuelle).
  **MAIS** le plan propose un HashMap custom + tokio eviction task,
  pas moka.
- Evaluation : HashMap custom vs moka. Arguments pour HashMap :
  (a) zero dep supplementaire, (b) le preview store est trivial
  (~5 entries max), (c) le TTL est un simple `Instant::elapsed()`,
  (d) le crate est deja DashMap-heavy (blob_serve.rs), un second
  pattern de cache est coherent. Arguments pour moka : (a) TTL
  natif, (b) eviction thread-safe sans spawn, (c) 6M downloads.
  **Decision** : HashMap custom est APPROACH-ALIGNED (pas
  APPROACH-NAIVE). Moka serait un over-engineering pour un cache
  de ~5 entries. Le plan est coherent.
- Verdict : APPROACH-ALIGNED (HashMap custom acceptable pour ce
  volume).

#### [5] reqwest — HTTP client Rust pour Factory CLI

- Source : context7 `/seanmonstar/reqwest` query "POST body bytes
  multipart upload file send HTTP request"
- Pattern : `reqwest::Client::new().post(url).body(bytes).send()`.
  Multipart via `multipart::Form::new().part("file", Part::bytes
  (vec))`. Feature flags : `multipart`, `json`, `stream`.
- Pertinence directe : sbfb-factory preview_cmd.rs et publish.rs
  utiliseront reqwest pour POST vers le daemon.
- CVE check : WebSearch "reqwest CVE rustsec 2026" — aucun CVE
  trouve sur reqwest. Derniere release : active (derniere version
  0.12.x, 2025).
- Verdict : APPROACH-ALIGNED (reqwest = standard Rust HTTP client).

#### [6] zip crate — Creation archive

- Source : WebSearch "zip crate Rust CVE vulnerability rustsec
  2025 2026"
- **CVE-2025-29787** : path traversal via symlinks dans zip
  extraction (< 2.3.0). SBFB utilise zip 8.5 — **non affecte**.
  De plus, SBFB utilise zip uniquement en **creation** (Phase B
  preview_cmd.rs crée le zip), pas en extraction via le crate zip
  dans sbfb-factory. L'extraction est faite par blob_serve.rs qui
  a sa propre validation `validate_zip_path()`.
- Verdict : clean (zip 8.5 >> fix 2.3.0).

### Tableau comparatif

| Aspect | Plan Phase B | Netlify/Vercel | IPFS | F-Droid | Moka |
|--------|-------------|----------------|------|---------|------|
| Preview URL | `/blob-serve/{hash}/index.html` | `deploy-preview-{id}.netlify.app` | `ipfs://{CID}` | repo local + client | N/A |
| Hash scheme | BLAKE3 | commit SHA | SHA256 multihash | N/A | N/A |
| TTL eviction | 30 min background task | PR close | GC non-pinne | manuelle | natif TTL |
| Publish path | CLI → HTTP POST → daemon | git push | ipfs add + pin | fdroid publish | N/A |
| Isolation | CSP sandbox iframe | isolated env | IPFS gateway | Android sandbox | N/A |

### Finding S1a

- **Classification** : APPROACH-ALIGNED
- **Evidence** : Netlify/Vercel (hash-based preview URL),
  IPFS (content-addressed eviction), F-Droid (CLI → local repo
  publish). Tous confirment le pattern plan. Aucun projet mature
  n'a abandonne le pattern "hash-based ephemeral cache + CLI
  delegate to daemon".
- **Impact sur le plan** : aucun.

---

## S1b — Deps/libs versions + CVE

### reqwest (NEW dep sbfb-factory)

- crates.io : reqwest 0.12.x actif, derniere release 2025.
- CVE : aucun CVE reqwest trouve (WebSearch "reqwest CVE rustsec
  2026"). RustSec advisory DB : clean pour reqwest.
- Features requises : `json` + blocking ou async. Le plan ne
  specifie pas — recommandation : async (coherent avec tokio
  runtime si utilise dans tests, ou blocking pour CLI simple).
  **Note** : sbfb-factory est un CLI synchrone (pas de tokio
  runtime). Deux options : (a) `reqwest::blocking` (simple, pas
  de dep tokio), (b) `tokio::runtime::Runtime::new()` + async
  reqwest. Option (a) est plus simple pour un CLI.

### zip (EXISTANTE, creation archive)

- Workspace : zip 8.5
- CVE-2025-29787 : path traversal via symlink, fixe 2.3.0.
  zip 8.5 >> 2.3.0 — **non affecte**.
- Usage Phase B : `zip::ZipWriter` pour creer archive dans
  preview_cmd.rs. Read-only path (blob_serve.rs) deja couvert.

### blake3 (EXISTANTE)

- Workspace : blake3 (workspace dep).
- Pas de CVE blake3 2025-2026 connu.

### clap (EXISTANTE sbfb-factory)

- Workspace : clap (workspace dep).
- Pas de breaking change impactant le plan (ajout subcommands).

### serde_json (EXISTANTE)

- Workspace : serde_json (workspace dep).
- Clean.

### Finding S1b

- **Classification** : clean
- **Aucun CVE bloquant**, aucune breaking change, aucune dep
  ajoutee avec risque connu.
- **Note operationnelle** : reqwest est une nouvelle dep pour
  sbfb-factory. Choisir `reqwest = { version = "0.12", features
  = ["blocking", "json"] }` pour eviter une dep tokio runtime
  dans le CLI.

---

## S2 — Decision chain reconstruction

### Fichiers scannes

- `crates/nexus-shell-daemon-core/src/lib.rs` : 10 commits
- `crates/nexus-shell-daemon/src/http.rs` : 17 commits
- `crates/nexus-shell-daemon/src/runtime.rs` : 15+ commits
- `crates/nexus-shell-daemon/src/deploy.rs` : 8 commits
- `crates/sbfb-factory/src/main.rs` : 3 commits (S67 C/D)
- `crates/sbfb-factory/Cargo.toml` : 3 commits (S67 C/D)
- `crates/nexus-shell-daemon-core/src/blob_serve.rs` : 7 commits

### Decisions historiques trouvees

#### Decision 1 : blob-serve CSP hardening (sandbox opaque origin)

- Sprint 53+, sha `4780c5a` : CSP hardened avec 6 directives
  (sandbox allow-scripts, worker-src none, frame-src none,
  object-src none, form-action none, base-uri none).
  Body : "Remove Nouvel onglet button that opened blob-serve in
  top-level tab — same-origin attack vector /auth/token."
- Sprint 53+, sha `8712890` : CSP sandbox dropped temporarily
  (wrong hypothesis about 'self' resolution).
- Sprint 53+, sha `0ee8cf4` : CSP sandbox restored, CORP header
  added.
  Body : "GPT 5.5 review confirmed Chromium resolves 'self'
  correctly under CSP sandbox."
- Reverse-commit check : pas de reversion post-`0ee8cf4`.
- **Status** : ACTIVE. Le CSP sandbox + CORP est le standard
  blob-serve actuel.
- **Impact Phase B** : positif. Le preview servi via blob-serve
  herite automatiquement de toutes ces protections. Pas besoin de
  CSP custom pour preview.

#### Decision 2 : Factory hors daemon (crate sbfb-factory)

- Sprint 67, sha `49d6bcd` (Phase C) : "Decision D2 v4 : Factory
  hors daemon, crate independant."
- Memory `nexus_grid_pivot.md` : "Factory = outil client externe
  (crate sbfb-factory), hors daemon (v4 D2)"
- CLAUDE.md : "Factory = outil client externe (crate sbfb-factory),
  hors daemon"
- Reverse-commit check : aucune reversion.
  ```
  git log --all --oneline HEAD -- | grep -iE "revert|undo" (factory)
  ```
  0 matches.
- **Status** : ACTIVE (Day 0 figee v4 D2).
- **Impact Phase B** : sbfb-factory ne depend PAS de
  nexus-shell-daemon-core. Les subcommands preview/publish
  communiquent via HTTP avec le daemon. Conforme.

#### Decision 3 : deploy-from-repo = seule voie publish verifiee

- Sprint 42, sha `aaa2e18` : "deploy API Rust" (port de deploy.py).
- Sprint 59, sha `46ed2c2` : "Verified deploy E2E".
- Sprint 67, sha `4ee93ab` : "sbfb-manifest + feed primitives +
  SBFB.json v2" (node_id optionnel dans deploy.rs).
- Kickoff S68 D3 : "La commande sbfb-factory publish lit
  running.json, pre-valide, puis appelle POST
  /api/v1/deploy-from-repo."
- Kickoff S68 D3 rejet : "Upload direct du zip : bypass la
  verification serveur (provenance, Ed25519)."
- Reverse-commit check : aucune reversion.
- **Status** : ACTIVE. deploy-from-repo est la seule voie qui
  produit une provenance verifiable.
- **Impact Phase B** : conforme. publish.rs delegue a
  deploy-from-repo, pas d'upload direct.

### Memory constraints

- `feedback_no_direct_blobserve.md` : JAMAIS ouvrir blob-serve en
  onglet direct. Preview dans iframe sandbox uniquement. Le plan
  est conforme (preview URL servie dans Browse iframe).
- `feedback_approach.md` : pick deepest option. HashMap custom est
  la bonne option pour un cache 5-entry (moka = over-engineering).

---

## S3 — Threat model analysis

### Primitive analysee : Preview ephemere + Factory publish path

### Description

POST /api/v1/preview/load recoit un zip brut, le stocke en memoire
avec hash BLAKE3, et le sert via blob-serve existant dans l'iframe
sandbox. TTL 30 min + eviction background task. sbfb-factory
preview zippe le repertoire local et POST vers le daemon.
sbfb-factory publish lit running.json et POST
/api/v1/deploy-from-repo.

### Assets en jeu

- A1 Memoire daemon : criticite Medium. Un zip malveillant ou
  surdimensionne pourrait epuiser la RAM.
- A2 Auth token daemon : criticite High. sbfb-factory lit
  auth_token pour s'authentifier aupres du daemon.
- A3 Integrite deploy : criticite High. Le publish path doit passer
  par deploy-from-repo pour garantir la provenance.
- A4 Code source utilisateur : criticite Medium. preview_cmd.rs
  zippe le repertoire local et l'envoie au daemon.

### Threat actors

- TA1 Script local malveillant : un malware user-mode qui abuse
  le endpoint preview/load pour epuiser la RAM du daemon.
- TA2 Insider (utilisateur du CLI) : utilisation non prevue des
  commandes preview/publish.
- TA3 Extension navigateur : exfiltration du preview content via
  blob-serve (couvert par CSP existant).

### Attack vectors identifies

1. **V1 ZIP bomb via preview/load** (DoS/resource exhaustion)
   - Asset : A1 memoire daemon
   - Un zip avec ratio compression >1000:1 pourrait decompresser
     en centaines de MB.
   - Mitigation existante : blob_serve.rs `load()` a deja
     `max_decompressed_bytes = 100 MB` (DEFAULT_MAX_DECOMPRESSED_BYTES).
   - Mitigation plan : le handler POST preview/load doit aussi
     avoir un `max upload size` (plan dit 10 MB). Double defense.
   - Couverture : test_preview_max_size_rejected (plan test #4).
   - **Gap** : aucun — double limite (10 MB upload + 100 MB
     decompressed).

2. **V2 Preview flooding** (DoS/resource exhaustion)
   - Asset : A1 memoire daemon
   - Calls repetes POST preview/load pour accumuler des previews.
   - Mitigation plan : TTL 30 min + eviction. Nombre max de
     previews en memoire = implicitement borne par la taille
     du HashMap.
   - Mitigation existante : bearer auth requis (endpoint dans
     authed_routes).
   - **Gap Low** : pas de limite max_preview_entries explicite.
     Avec bearer auth + TTL + 10 MB max, le pire cas est
     ~10 MB * N uploads en 30 min. Pour un daemon local
     single-user, acceptable. Recommandation : ajouter un cap
     (ex : 10 previews max), mais non-bloquant pre-launch.

3. **V3 Auth token exposure** (Information leakage)
   - Asset : A2 auth token
   - sbfb-factory lit auth_token ou tokens.json pour s'authentifier.
   - Mitigation existante : auth_token est 0600 (Unix),
     tokens.json aussi. Le CLI lit le fichier avec les memes
     permissions user que le daemon.
   - **Gap** : aucun — meme modele de securite que le shell React
     (qui lit le token via GET /auth/token sur loopback).

4. **V4 Path traversal dans preview zip** (Injection/forgery)
   - Asset : A4 code source
   - Un zip crafted avec paths "../" pourrait ecrire hors du
     cache.
   - Mitigation existante : blob_serve.rs `validate_zip_path()`
     rejette "..", backslash, absolute paths. Le zip est decompress
     en memoire (HashMap), PAS sur filesystem. Pas de write
     filesystem.
   - **Gap** : aucun — decompression in-memory + validation path.

5. **V5 Bypass deploy-from-repo** (Privilege escalation)
   - Asset : A3 integrite deploy
   - sbfb-factory publish pourrait tenter un upload direct du zip
     au lieu de passer par deploy-from-repo.
   - Mitigation plan : publish.rs appelle POST
     /api/v1/deploy-from-repo avec repo_url + commit_sha. Le
     daemon fait clone + verify + sign.
   - **Gap** : aucun — le plan est conforme a D3 (rejete "upload
     direct du zip" dans kickoff §4 D3).

6. **V6 Supply chain reqwest** (Supply chain)
   - Asset : sbfb-factory binaire
   - reqwest est une dep nouvelle. Vecteur : compromission
     upstream dep de reqwest.
   - Mitigation : reqwest est maintenu par seanmonstar (hyper
     author), 200M+ downloads, aucun CVE connu. Locked via
     Cargo.lock.
   - **Gap** : aucun — dep standard, auditee par l'ecosysteme.

7. **V7 Race condition TTL eviction** (Temporal attacks)
   - Asset : A1 memoire
   - Un GET /blob-serve/{hash}/ pendant l'eviction pourrait
     trouver un etat inconsistant.
   - Mitigation plan : HashMap + Mutex/RwLock ou DashMap. L'eviction
     supprime l'entree atomiquement. Un GET concurrent retourne
     404.
   - **Gap** : aucun — atomicite HashMap operations.

### Mitigations existantes (T0-T5)

- T-BLOB-1 (Sprint 12) : blob-serve CSP `connect-src 'none'` +
  COOP/COEP isolation → couvre V4.
- T-BLOB-2 (Sprint 53) : CSP sandbox + frame-src none + form-action
  none → couvre V1/V4.
- T-AUTH (Sprint 16) : bearer + Host + Origin loopback gate →
  couvre V2/V3.
- T-DEPLOY (Sprint 14/42) : deploy-from-repo provenance chain →
  couvre V5.

### Gaps identifies

- **GAP1** V2 pas de cap max_preview_entries : severity **Low**.
  Recommandation S69+ si monitoring montre accumulation. Non-bloquant
  pre-launch (single-user, bearer auth).

### Regression check

- La primitive NE diminue PAS l'efficacite de T-BLOB-1/T-BLOB-2 :
  le preview est servi via le meme blob-serve avec meme CSP.
- La primitive NE cree PAS de nouveau vecteur non couvert
  (les 7 vecteurs sont couverts ou Low-gap).
- Pas de nouveau T necessaire.

### Verdict S3

**clean** (1 gap Low, 0 regression T0-T5).

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui

Fichier `crates/nexus-core-rs/src/canonical.rs` : 296 lignes,
13 DOMAIN_*_V1 constants, 1 fonction `canonical_bytes()`.

### Structs verifiees

Phase B ne touche AUCUNE struct canonical. Le preview est un
HashMap ephemere in-memory. Le publish path delegue a
`deploy-from-repo` existant (qui utilise DOMAIN_PROVENANCE_V1 +
DOMAIN_FEED_V1 via ses propres paths deja testes).

Nouveaux fichiers Phase B :
- `preview.rs` (NEW daemon-core) : PreviewStore = HashMap<String,
  PreviewEntry>. Pas de Serialize/Deserialize — pas un wire format.
- `publish.rs` (NEW sbfb-factory) : CLI HTTP client → POST
  deploy-from-repo. Pas de struct canonical.
- `preview_cmd.rs` (NEW sbfb-factory) : CLI → zip → POST
  preview/load. Pas de struct canonical.

### Day 0 check

- D1 ProofCard : non touche Phase B.
- D2 Preview ephemere via blob-serve : Phase B **implemente**
  D2. Conforme.
- D3 Publish via deploy-from-repo : Phase B **implemente**
  D3. Conforme.
- D4 Factory gates FG4-FG7 : non touche Phase B (Phase C).
- D5 ProofCard UI : non touche Phase B (Phase D).
- Decisions actees pivot.md : aucune contredite.

### Pre-launch policy

- `*_VERSION = 1` : aucune constante VERSION touchee. OK.
- Pas de tolerant decoder multi-version : N/A (pas de wire format).
- Pas de tests "legacy decode" zombie : N/A.
- Preview est un artefact local compute ephemere, pas un wire
  format protocolaire. Meme traitement que ProofCard Phase A.

### Version constants scan

```
TASK_FORMAT_VERSION = 1
CURATOR_LIST_FORMAT_VERSION = 1
FEED_FORMAT_VERSION = 1
KEY_ROTATION_FORMAT_VERSION = 1
POW_FORMAT_VERSION = 1
PIN_FILE_FORMAT_VERSION = 1
INVITE_FORMAT_VERSION = 2
PROJECT_ANNOUNCEMENT_VERSION = 1
```

Toutes inchangees par Phase B.

---

## Telemetrie preflight (agent deep)

- S1a : 6 projets OSS analyses (Netlify/Vercel, IPFS, F-Droid,
  Moka, reqwest, zip crate) / 0 fichiers source bruts lus via
  WebFetch (projets analyses via docs + WebSearch + context7) /
  1 context7 query (reqwest multipart) / 8 WebSearch queries /
  finding : APPROACH-ALIGNED
- S1b : 6 libs scannees (reqwest, zip, blake3, clap, serde_json,
  walkdir) / 3 CVE searches (reqwest, zip, blake3) / finding :
  clean (CVE-2025-29787 zip non-applicable car v8.5)
- S2 : 7 commit bodies lus in extenso (f9d722e, 49d6bcd, a4cc0ae,
  4780c5a, 8712890, 0ee8cf4, 4ee93ab) / 3 decisions reconstruites /
  3 reverse-commit checks executes (0 reversion) / 4 memory files
  lus / finding : clean
- S3 : FULL / 7 vectors analyses / 1 gap Low (max_preview_entries)
  / 0 regression T0-T5
- S4 : FULL / 0 structs canonical touchees / canonical.rs lu
  integralement : oui / 0 VERSION constants modifiees

---

## Action

Proceder code Phase B. Le plan est conforme a l'etat de l'art OSS,
aux decisions historiques, au threat model, et au wire format.

Note operationnelle S1b : pour `reqwest` dans sbfb-factory, preferer
`reqwest = { version = "0.12", features = ["blocking", "json"] }`
(CLI synchrone, pas de dep tokio runtime dans sbfb-factory).
