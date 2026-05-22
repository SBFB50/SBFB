# Sprint 68 — Kickoff (Proof Cards + Publish Gate)

**Ecrit** : 2026-05-21 (post-audit gate S67 PASS `5449903`).
**Type** : **sprint impair** — pas de phase dette obligatoire.
**Tip master d'entree** : `b937b03` (audit findings S67 PASS
0 P0, 0 P1, 3 P2, 2 P3).
**Phase 0 audit Sprint 67** : **DEJA JOUE** — `5449903` PASS.
Aucun fix requis.
**Version archive** : v2.1 — Protocole Neutre + Factory/RRV.
**Roadmap source** : `.planning/roadmap_v4_neutral_protocol_factory_rrv.md`.
Sprint 2 sur 3 (Arc 2 Factory + RRV @protocole + Canari).

---

## Sources context7 + WebSearch consultees (pre-gel)

| # | Source | Type | Date | Finding cle |
|---|--------|------|------|-------------|
| 1 | WebSearch "W3C Verifiable Credentials proof card computation" | WebSearch | 2026-05-21 | W3C VC 2.0 publie 2025-05-15 — modele `proof` block (Data Integrity) ou JWT/JWS. Render Method prevu sept. 2026. Pattern retenu pour inspiration : evidence factuelle, pas trust social. |
| 2 | WebSearch "OpenSSF Scorecard proof evidence computation" | WebSearch | 2026-05-21 | OpenSSF Scorecard V5 : 18+ checks, score 0-10 par check, structured results. Pattern : score de completude = combinaison de checks binaires. Confirme D11 roadmap v4. |
| 3 | WebSearch "F-Droid reproducible builds proof verification card display" | WebSearch | 2026-05-21 | F-Droid verification.f-droid.org : status cards par app, badges rebuilder. IzzyOnDroid 5-level graph. NLnet funding overhaul 2025. Pattern retenu pour Proof Card UI : badges par couche. |
| 4 | WebSearch "SLSA provenance verification display UI proof" | WebSearch | 2026-05-21 | slsa-verifier CLI, GitHub artifact attestations. SLSA v1.1/v1.2 Source Track 2025. Pas de composant UI standard — chaque projet construit le sien. Confirme : Proof Card = notre composant natif. |
| 5 | WebSearch "Sigstore cosign SLSA proof verification badge display" | WebSearch | 2026-05-21 | Cosign v2.6.0, bundles par defaut. Pas de composant UI badge. Confirme : espace design ouvert pour Proof Card composant. |
| 6 | WebSearch "Rust path traversal canonicalize Windows dunce" | WebSearch | 2026-05-21 | `dunce` crate : canonicalize sans UNC prefix. `soft-canonicalize` : ADS validation + TOCTOU + traversal clamp. `strict_path` : CVE-2025-8088 NTFS ADS, CVE-2022-21658 TOCTOU. `path-security` crate aussi. Retenu pour D5 : dunce::canonicalize + prefix check. |
| 7 | WebSearch "Rust CLI publish pipeline local preview ephemeral server" | WebSearch | 2026-05-21 | Pubky CLI (Rust, assert_cmd ephemeral testnet). Pattern : server temporaire avec cleanup. Confirme approche preview via daemon existant. |
| 8 | context7 `/tokio-rs/axum` "serve static files directory ServeDir" | context7 | 2026-05-21 | `tower_http::services::ServeFile` / `ServeDir` integrable via `route_service()`. Pattern utilise pour preview endpoint. Blob-serve existant couvre deja 100% du besoin. |
| 9 | context7 `/websites/serde_rs` resolu pour serde patterns | context7 | 2026-05-21 | serde `#[serde(default)]` + flatten + adjacently tagged enums. Pattern applicable pour ProofCard struct. |
| 10 | SYNTHESIS `.planning/research/SYNTHESIS_factory_rrv_protocol.md` §4.6 | Code local | 2026-05-19 | ProofCard data model complet (12 champs, formule score 0-100, 7 risk factors). formula_version gelee D16. |
| 11 | SYNTHESIS §3.5 | Code local | 2026-05-19 | Publish path 16 etapes (preview → local draft → verified release). POST /api/v1/preview/load = primitive manquante P0. |
| 12 | SYNTHESIS §3.4 | Code local | 2026-05-19 | Gates FG4-FG7 = scope S68. FG4 diff + FG5 sandbox + FG6 secrets + FG7 preview. |
| 13 | Audit findings S67 Track B | Artefact local | 2026-05-21 | P2-C-2 path traversal : `contains("..")` string-level, pas canonicalize. 1/3. |
| 14 | `crates/sbfb-factory/src/main.rs` | Code local | 2026-05-21 | CLI actuel : 2 subcommands (create, validate). Manquent : preview, publish, diff, scan-secrets CLI. |
| 15 | `crates/sbfb-manifest/src/lib.rs` | Code local | 2026-05-21 | SbfbManifest : 8 champs, BridgeConfig.methods, allowlist 9 methodes. Manque : `proof_card_get` dans allowlist. |
| 16 | `crates/nexus-shell-daemon/src/deploy.rs` | Code local | 2026-05-21 | deploy_from_repo : clone+zip+hash+sign. publish_announcement cree browse entry + feed ReleasePublished. Besoin : wiring Factory→daemon via cette API. |

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 67 (Factory Foundation, Arc 2 1/3) a livre les primitives daemon neutres et les outils Factory de base. Le daemon a maintenant FTS5 search operationnel, le crate `sbfb-manifest` partage entre daemon et factory, les feed operations CuratorVouched/CuratorDisendorsed, et le endpoint pagine feed/entries. Le crate `sbfb-factory` existe avec `create` et `validate`, un template static embarque, un secret scanner regex, et une provenance locale BLAKE3.

Ce qui manque pour la chaine complete Arc 2 est le **lien entre Factory et daemon** (publish path), la **preuve visible** que le protocole fonctionne (Proof Cards), et le **preview ephemere** qui permet de tester une app avant publication. S68 comble ces gaps pour que S69 puisse faire le dogfood Babel de bout en bout.

### §1.2 Ancrage roadmap v4

Arc 2 (Factory + RRV @protocole + Canari), sprint 2 sur 3. Dependances amont : S67 FTS5 + sbfb-manifest (satisfaites). Dependances aval : S69 attend Proof Cards + publish path pour le dogfood Babel et le pilote ferme.

Gate 1 (post-S69) exige : Proof Card Babel affichee, search retourne Babel, publish operationnel, daemon stable 24h.

### §1.3 Compteurs tests entree (tip `b937b03`)

| Suite | Count |
|---|---|
| Rust nextest | 1384 |
| Vitest | 270 |
| size-limit | 6/6 |
| **Total** | **~1660** |

### §1.4 Pre-launch protocol policy (rappel)

- `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` restent a 1 jusqu'au
  go-live public. Un sprint qui change le canonical redefinit la v1
  courante, ne bump PAS la version.
- Feed extensible via raw-op : ajouter une operation ne bump PAS
  `FEED_FORMAT_VERSION`. Ajout ProofCardComputed comme feed op
  candidat S70+ seulement.
- `#[serde(default)]` reste legitime pour robustesse runtime.
- ProofCard est un artefact **local compute** (pas un wire format
  protocolaire). Sa formula_version est locale au daemon. Pas de
  bump feed.

---

## §2 Goal

Livrer le **circuit de preuve visible** : Proof Cards computees depuis
les donnees daemon existantes (browse, feed, provenance, curators) et
affichables dans le shell Browse, plus le **publish path** Factory→daemon
et le **preview ephemere** qui ferment la boucle create→preview→publish.
Les Factory gates FG4-FG7 sont implantees. La path traversal Windows
(P2-C-2) est corrigee.

**Critere SMART : toutes les rows fail-fast vertes au verification.md,
mesure binaire au Phase E wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 67

Audit S67 execute en session fraiche `5449903`.
Verdict : **PASS** (0 P0, 0 P1, 3 P2, 2 P3).
Aucun fix bloquant. Sprint 68 autorise.

P2 documentes :
- P2-I-1 body Phase E docs-only 7/9 sections (1/3)
- P2-I-2 delta repartition body vs verification.md (1/3)
- P2-C-2 path traversal Windows (1/3, confirme)

P3 documentes :
- P3-A-1 sanitize_query MAX_QUERY_LENGTH (nit)
- P3-I-1 volume commits inter-phases (documentaire)

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — ProofCard struct Rust + formule de score deterministe

**Sources consultees** :
- WebSearch "W3C Verifiable Credentials 2.0" (2026-05-21) : W3C VC 2.0 publie mai 2025. Modele `proof` block pour evidence cryptographique. Render Method prevu sept. 2026. Inspiration pour la structure evidence-based.
- WebSearch "OpenSSF Scorecard V5" (2026-05-21) : 18+ checks, score structuré par check. Pattern : score = combinaison de checks binaires mesurables. Confirme l'approche "evidence score" pas "trust score".
- WebSearch "F-Droid verification status cards" (2026-05-21) : badges par app, 5 niveaux de verification. Pattern retenu pour les couches de preuve (provenance, license, curation, freshness).
- SYNTHESIS §4.6 (2026-05-19) : data model complet ProofCard 12 champs, formule 0-100, 7 risk factors, formula_version gelee D16.
- Code local `crates/nexus-coordinator-rs/src/provenance.rs` : verify_provenance() existe deja, prend `record_json` + `public_key`.
- Code local `crates/nexus-shell-daemon-core/src/search.rs` : search() retourne `SearchResult` avec project_id, score bm25, snippet.

**Retenu** : ProofCard comme struct Rust dans `nexus-coordinator-rs` (pas dans sbfb-manifest — c'est un artefact daemon, pas un artefact Factory). La formule est celle de la SYNTHESIS §4.6 : base 30 + evidence layers (provenance +20, open_source +10, freshness +10, curation +10/+10, license +5, archive_hash +5) - risk factors (no_provenance -15, stale_source -10, unverified_deploy -10, old_release -5). `formula_version: 1` (D16 gelee). Le compute est local — le daemon rassemble les donnees qu'il possede deja (browse entry, feed, provenance record, curator lists) et produit la ProofCard. Pas de nouveau wire format protocolaire.

**Rejete** :
- **W3C VC 2.0 complet** : surdimensionne pour un compute local. W3C VC vise l'interoperabilite multi-emetteur avec proof cryptographique — SBFB fait un self-report local. La spec Render Method (sept. 2026) n'est pas encore publiee. (Source : W3C press release 2025-05-15)
- **OpenSSF Scorecard comme framework** : couple a GitHub API, Go-only, 18 checks specifiques CI/CD. Notre ProofCard evalue les couches protocolaires SBFB, pas les pratiques GitHub. (Source : github.com/ossf/scorecard docs/checks.md)
- **Score pondere configurable** : introduit une complexite inutile pre-launch. La formule additive fixe est deterministe et auditable. Un formula_version permet de la faire evoluer post-launch sans casser la comparabilite. (Rationale SYNTHESIS §4.6)

**Implications code** : `crates/nexus-coordinator-rs/src/proof_card.rs` (NEW), `crates/nexus-shell-daemon/src/http.rs` (endpoint GET), `web/src/api/protocol.ts` (schema Zod).

### D2 — Preview ephemere via blob-serve existant

**Sources consultees** :
- SYNTHESIS §3.5 (2026-05-19) : preview = POST /api/v1/preview/load, charge zip dans cache blob-serve sans persister dans iroh-blobs, TTL ~30 min. Primitive daemon manquante P0.
- context7 `/tokio-rs/axum` (2026-05-21) : `route_service()` + `ServeFile` pattern. Blob-serve existant couvre deja la decompression zip + LRU cache + CSP.
- WebSearch "Rust CLI publish pipeline local preview ephemeral server" (2026-05-21) : Pubky CLI ephemeral testnet pattern. Confirme : utiliser l'infra existante avec nettoyage automatique.
- Code local `crates/nexus-shell-daemon/src/http.rs` : blob-serve route existante `GET /blob-serve/{hash}/{path}`. Le hash est l'identifiant du zip dans le BlobStore.
- Code local `crates/nexus-shell-daemon/src/runtime.rs` : BlobStore enum (Mem/Fs) avec Deref pattern P52.

**Retenu** : Le preview est un zip charge en memoire dans le BlobStore existant avec un TTL de 30 minutes et un nettoyage automatique. L'endpoint `POST /api/v1/preview/load` recoit le zip (multipart ou raw bytes), le stocke dans le BlobStore memoire avec un hash BLAKE3, et retourne le hash. Le blob-serve existant sert le contenu dans l'iframe sandbox. Un tokio background task evicte les previews expires. Pas de persistence iroh-blobs — c'est ephemere par design.

**Rejete** :
- **Serveur HTTP temporaire separe** : duplique l'infra blob-serve existante (CSP, COOP/COEP, sandbox headers). Pas de raison d'avoir un second serveur quand le daemon sait deja servir des zips. (Source : code blob_serve.rs existant)
- **Preview via iroh-blobs persistant** : pollue le store permanent avec des archives de test. Le preview est ephemere — il ne devrait pas survivre un restart. (Rationale SYNTHESIS §3.5)
- **Preview cote Factory (serveur local Factory)** : brise la neutralite du daemon. Le preview doit passer par le meme chemin blob-serve que les apps deployees pour que l'experience soit fidele. (Source : SYNTHESIS §2.4 daemon = tuyau stupide)

**Implications code** : `crates/nexus-shell-daemon/src/http.rs` (POST handler), `crates/nexus-shell-daemon-core/src/preview.rs` (NEW, preview store + TTL eviction).

### D3 — Publish path Factory→daemon via deploy-from-repo existant

**Sources consultees** :
- SYNTHESIS §3.5-3.6 (2026-05-19) : publish path 16 etapes, `sbfb-factory publish` appelle POST /api/v1/deploy-from-repo. Le daemon fait le clone+verify+sign.
- Code local `crates/nexus-shell-daemon/src/deploy.rs` : deploy_from_repo() complet — clone, validate manifest, zip, BLAKE3 hash, Ed25519 sign, blob store, announce, feed entry.
- Code local `crates/sbfb-factory/src/main.rs` : CLI actuel 2 subcommands. Manque : `publish`, `preview`, `diff`, `scan-secrets`.
- SYNTHESIS §3.4 (2026-05-19) : Gates FG4-FG7 implantees ici, FG8-FG9 S69.

**Retenu** : La commande `sbfb-factory publish` lit `running.json` (port + auth token du daemon), pre-valide le projet localement (manifest, secrets, path traversal), puis appelle POST /api/v1/deploy-from-repo avec le repo_url et commit_sha. Le daemon fait tout le travail serveur (clone, zip, hash, sign, announce). Factory est un **client mince** qui pre-filtre et delegue. Les subcommands `preview` et `diff` sont ajoutees aussi pour completer la boucle SYNTHESIS §3.6.

**Rejete** :
- **Upload direct du zip** : bypass la verification serveur (provenance, Ed25519). Le deploy-from-repo est la seule voie qui produit une provenance verifiable. (Source : deploy.rs, THREAT_MODEL §verification)
- **API publish custom** : duplique deploy-from-repo. Le path existant est complet et teste. Pas de raison d'en creer un second. (Source : deploy.rs 363 lignes, 4 tests)
- **Factory fait la signature Ed25519** : brise la separation des responsabilites. Le daemon est le seul depositaire de la cle node — Factory ne doit pas avoir acces a la cle privee. (Rationale SYNTHESIS §2.4, decision D2 roadmap v4)

**Implications code** : `crates/sbfb-factory/src/publish.rs` (NEW), `crates/sbfb-factory/src/preview_cmd.rs` (NEW), `crates/sbfb-factory/src/diff.rs` (NEW), `crates/sbfb-factory/src/main.rs` (3 subcommands ajoutes).

### D4 — Factory gates FG4-FG7 dans sbfb-factory

**Sources consultees** :
- SYNTHESIS §3.4 (2026-05-19) : gates FG4 (Diff), FG5 (Sandbox), FG6 (Secrets/deps), FG7 (Preview). FG0-FG3 deja implicites dans create/validate S67.
- Code local `crates/sbfb-factory/src/secret_scanner.rs` : scan_directory() avec regex AKIA/ghp_/gho_/PEM. FG6 secrets = deja fait. FG6 deps = lockfile check a ajouter.
- Code local `crates/sbfb-factory/src/template_engine.rs` : validate() avec path traversal `contains("..")`. FG5 = ameliorer avec canonicalize.
- WebSearch "Rust path traversal canonicalize Windows dunce" (2026-05-21) : `dunce` crate pour canonicalize Windows-friendly. `soft-canonicalize` pour ADS + TOCTOU. `strict_path` pour CVE coverage.

**Retenu** : Les gates FG4-FG7 sont implantees comme un pipeline de validation dans `sbfb-factory`. FG4 (Diff) : `sbfb-factory diff` compare le workspace vs le template et affiche les modifications. FG5 (Sandbox) : canonicalize via `dunce` + prefix check (le path canonique doit rester sous le workspace root) + symlink deny. Corrige P2-C-2 (path traversal Windows 1/3→resolved). FG6 (Secrets) : scan existant + verification lockfile (factory.template.lock hash = factory.provenance.json template_hash). FG7 (Preview) : wiring vers POST /api/v1/preview/load + ouverture blob-serve URL.

**Rejete** :
- **`std::fs::canonicalize` seul** : retourne des paths UNC sur Windows (`\\?\C:\...`), incompatibles avec certaines libs. (Source : dunce crate documentation, crates.io/crates/dunce)
- **`soft-canonicalize`** : plus complet (ADS, TOCTOU) mais plus lourd et pre-1.0 (0.1.x). `dunce` est mature (6M downloads, derniere release 2024). Pour un outil CLI local pre-launch, dunce suffit. (Source : crates.io stats dunce vs soft-canonicalize)
- **Gates comme crate separe** : overhead d'un crate pour ~300 LOC de validation. Les gates sont internes a sbfb-factory et testees unitairement dans le meme crate. (Rationale : simplicite, pas de dep externe supplementaire pour la structure)

**Implications code** : `crates/sbfb-factory/src/gates.rs` (NEW), `crates/sbfb-factory/src/diff.rs` (NEW), `crates/sbfb-factory/Cargo.toml` (dep dunce).

### D5 — Proof Card UI composant dans le shell Browse

**Sources consultees** :
- WebSearch "F-Droid verification status cards" (2026-05-21) : badges par app avec niveaux de verification (verified/not verified). IzzyOnDroid 5-level graph. Pattern retenu : carte compacte avec niveaux de preuve.
- WebSearch "Sigstore cosign badge display UI" (2026-05-21) : pas de composant UI standard dans l'ecosysteme SLSA/Sigstore. Chaque projet construit son propre affichage. Confirme que SBFB peut definir son propre composant.
- SYNTHESIS §4.6 (2026-05-19) : ProofCard data model avec confidence 0-100, risk factors, source layers. Composant HTML dans une future app sbfb-search. Pour S68 : composant React dans le shell Browse.
- Code local `web/src/pages/BrowsedProject.tsx` : affiche deja les badges provenance (3 etats) et le statut verification. Le ProofCard s'integre naturellement.

**Retenu** : Un composant React `ProofCard.tsx` dans le shell affiche le score de completude (0-100) avec les couches de preuve sous forme de checklist visuelle (provenance, license, freshness, curation, archive hash). Le composant fait un appel bridge `proof_card_get(project_id)` qui interroge le daemon. Le daemon compute la ProofCard a la volee (pas de cache — le compute est rapide, ~1ms, car les donnees sont locales). L'affichage est une carte expandable dans `BrowsedProject.tsx` avec le score en gros et les details en accordeon.

**Rejete** :
- **App sbfb-search separee** : prevue S70+ (SYNTHESIS §4.5). Pour S68, le composant est dans le shell Browse car c'est l'endroit ou l'utilisateur voit les projets. L'app search sera un client supplementaire.
- **Score en badge (juste le chiffre)** : insuffisant pour la transparence. F-Droid montre les couches individuelles, pas juste un badge OK/NOK. Le score 0-100 seul est opaque — les couches detaillees montrent pourquoi le score est ce qu'il est. (Source : F-Droid verification.f-droid.org pattern)
- **Cache ProofCard persistant** : complexite prematuree. Le compute est local et rapide (lookup SQLite + browse cache). Un cache invaliderait mal (quelle TTL ? quel trigger de refresh ?). Compute a la volee = toujours frais. (Rationale : simplicite pre-launch)

**Implications code** : `web/src/components/ProofCard.tsx` (NEW), `web/src/pages/BrowsedProject.tsx` (integration), `web/src/api/protocol.ts` (schema ProofCard Zod), `web/public/sbfb-bridge.js` (methode proof_card_get), `web/src/hooks/useBridge.ts` (dispatch proof_card_get).

---

**Acknowledged review findings (G1)** :

Scoring : D1 ok, D2 ok, D3 ok, D4 warning, D5 ok.
Rigor signal G4 satisfait (1 warning sur 5).

D4 warning : la dep `dunce` est mature (6M downloads) mais sa derniere release date de 2024 — pas de source < 90 jours specifique a `dunce`. La decision reste valide : `dunce` est stable, le probleme qu'il resout (UNC paths Windows) est inherent a l'OS, et le crate n'a pas de CVE. Le warning est acknowledge, pas adjust.

---

## §5 Plan Phase outline A..E

### Phase A — ProofCard computation + daemon endpoint

Scope : struct ProofCard Rust dans nexus-coordinator-rs, formule de score deterministe, endpoint GET /api/daemon/proof-card/{project_id}, bridge method proof_card_get, schema Zod frontend.

### Phase B — Preview ephemere + Factory publish path

Scope : POST /api/v1/preview/load dans le daemon, preview store avec TTL 30 min, subcommands `sbfb-factory preview` et `sbfb-factory publish` (via deploy-from-repo), lecture running.json.

### Phase C — Factory gates FG4-FG7 + P2-C-2 path traversal fix

Scope : gates FG4 (diff), FG5 (sandbox avec dunce::canonicalize), FG6 (secrets + lockfile check), FG7 (preview wiring). Correction P2-C-2 (path traversal Windows). Subcommand `sbfb-factory diff`. Subcommand `sbfb-factory scan-secrets` CLI expose.

### Phase D — Proof Card UI composant + Browse integration

Scope : ProofCard.tsx composant React, integration dans BrowsedProject.tsx, bridge wiring proof_card_get, tests Vitest composant.

### Phase E — Verification + wrap-up

Scope : verification.md fail-fast, audit_plan S69, CLAUDE.md, SPRINT_LOG.md, memory update.

---

## §6 Items carry/dette

### Items 3/3 (traitement Sprint 68)

Aucun item n'atteint 3/3 MANDATORY au S68.

### Carry absorbes S68

| Item | Reports | Phase S68 | Exit condition |
|---|---|---|---|
| P2-C-2 path traversal Windows | 1/3→resolved | Phase C | `dunce::canonicalize` + prefix check + test Windows backslash path |
| P2-I-1 body Phase E docs-only | 1/3 | Phase E | 9/9 sections dans le commit body Phase E |

### Carries reconduits S69

| Item | Reports | Justification |
|---|---|---|
| P2-A-1 rand blocker upstream | exemption→exemption | upstream rand 0.9 non publie. Dep transitive iroh 0.98. Aucune action possible tant que rand 0.9 ne sort pas. |
| P2-AUDIT-2 iroh transitives | exemption→exemption | herite du pin iroh 0.98. Evaluate a Gate 1 post-S69. Aucune action possible sans upgrade iroh. |
| P2-G-1 exe lock intermittent | monitoring→monitoring | non reproductible depuis S62 (6 sprints). Monitoring passif. Si reproduit : priorite immediate. |
| T-NN+2 iframe Rust-wasm | bloque→bloque | toolchain gaps wasm32 inchanges. Hors scope Arc 2. |
| P2-I-2 delta repartition body | 1/3→2/3 | gap formel dans les commit bodies. Absorption planifiee S69 via template body standardise. |
| LT-2 Radicle sortie cap G7 | trigger PENDING | tag v1.0 pose localement, pas pousse origin. Trigger = push tag + GitHub Release. |
| LT-5 redundancy persistence | hors-sprint | reclassifie S26. Condition : premier deploiement multi-worker OU tag v1.0 go-live. |

### Attention 3/3 S69

| Item | Reports apres S68 | Raison |
|---|---|---|
| P2-I-2 delta repartition body | 2/3 | Passera 3/3 au S69. Devra etre resolu dans le plan S69. |

---

## §7 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | SearchManifest wire format + gossip | S70 | Protocole reseau. Hors scope S68 (couche service locale seulement). Prereq Arc 3. |
| 2 | Page React /factory | S69+ | CLI suffit pour S68. L'UI viendra quand la boucle CLI est validee par le dogfood. |
| 3 | Babel dogfood via Factory | S69 | Attend publish path complet + Proof Cards (livrables S68). |
| 4 | @dev index tree-sitter | S70+ | Decision PO 2026-05-21 : @dev non bloquant Gate 1. |
| 5 | Template react-vite | S69+ | 2 templates suffisent (static + static-storage). react-vite si demande pilote. |
| 6 | Factory audit log JSONL | S69+ | Tracabilite avancee. Les gates FG4-FG7 tracent par stdout pour S68. |
| 7 | CuratorVouched UI shell | S70+ | Le vouch est dans le feed (S67). L'UI de curation dans le shell est post-pilote. |
| 8 | FG8 Provenance Ed25519 | S69 | Depend du publish path complet. FG8 = provenance daemon-side, pas factory-side. |
| 9 | FG9 Publish gate complete | S69 | S68 livre le publish basic (via deploy-from-repo). FG9 = garde-fou complet pre-publish. |
| 10 | FG10 Review gate | S69 | Sprint review automatise. Depend de FG8+FG9. |
| 11 | Fuzzing cargo-fuzz/proptest | post-audit | Hors scope fonctionnel. Utile post-Gate 1. |
| 12 | Feed format version bump | post-launch | Pre-launch policy. |
| 13 | ProofCard comme feed op | S70+ | Candidat feed op SearchManifest. S68 = compute local seulement. |
| 14 | Diff engine avance | S69+ | S68 livre un diff basique (fichiers ajoutes/modifies/supprimes). Diff semantique = post-pilote. |

---

## §8 Tracabilite scope

| Item S67 "What's NOT" | Sprint + Phase S68 |
|---|---|
| Preview ephemere | Phase B S68 (D2 retenu) |
| Diff engine avance | Phase C S68 basique / Reconduit S69+ avance |
| Page React /factory | Reconduit S69+ |
| Proof Cards computation | Phase A S68 (D1 retenu) |
| SearchManifest wire format | Reconduit S70+ |
| Babel dogfood via Factory | Reconduit S69 |
| @dev index tree-sitter | Reconduit S70+ |
| Bridge method proof_card_get | Phase A S68 (inclus dans D1) |
| Template react-vite | Reconduit S69+ |
| Factory audit log JSONL | Reconduit S69+ |
| CuratorVouched UI shell | Reconduit S70+ |
| Publish path factory→daemon | Phase B S68 (D3 retenu) |
| Feed format version bump | Reconduit post-launch |
| Fuzzing cargo-fuzz/proptest | Reconduit post-audit |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Preview ephemere fuite memoire | Medium | Medium | TTL 30 min + eviction background task + test load/evict cycle |
| R2 | `dunce` ne couvre pas tous les edge cases Windows | Low | Medium | Tests avec paths UNC, junction points, ADS. Fallback : `std::fs::canonicalize` + strip UNC prefix manuellement. |
| R3 | ProofCard formule produit des scores confus pour l'utilisateur | Medium | Low | formula_version permet d'iterer post-launch. Score accompagne des couches detaillees (pas juste le chiffre). |
| R4 | publish subcommand echoue sur running.json absent | Low | Medium | Message d'erreur clair "daemon not running". Documentation dans --help. |
| R5 | bridge method proof_card_get non couvert par le sandbox iframe | Low | High | proof_card_get est read-only (GET). Ajout a BRIDGE_METHOD_ALLOWLIST dans sbfb-manifest. Test allowlist. |
| R6 | Carry P2-I-2 (delta body) atteint 3/3 S69 | High | Low | Impact formel seulement. Template body standardise planifie S69. |
| R7 | Preview + blob-serve interaction avec le BlobStore Fs (persistent) | Low | Medium | Preview utilise un HashMap separe (pas le BlobStore iroh). Namespace distinct. Test isolation. |

---

## §10 Audit gate pattern — rappel

Phase 0 S67 jouee (`5449903` PASS). Phase E du sprint devra produire :
- `sprint68_verification.md` (self-report fail-fast)
- `sprint69_audit_plan.md` (plan pour Phase 0 S69)
- Mise a jour `docs/rust/PATTERNS.md` si nouveaux patterns
- Mise a jour `docs/shell/PATTERNS.md` si nouveaux patterns

---

## §11 Checkpoint de validation

1. **D1** — La formule additive fixe (base 30 + layers - risks) est-elle assez expressive, ou faut-il un systeme pondere configurable des maintenant ?
2. **D2** — Le preview ephemere en memoire (HashMap, pas BlobStore iroh) avec TTL 30 min est-il acceptable, ou faut-il un mecanisme plus robuste (fichier temporaire, LRU borne) ?
3. **D3** — Le publish via deploy-from-repo existant (repo_url + commit_sha) est-il suffisant pour le pilote, ou faut-il un chemin upload-direct du zip pour les projets sans repo public ?
4. **D4** — `dunce` seul pour le path traversal Windows est-il suffisant, ou faut-il `strict_path` / `soft-canonicalize` pour la couverture ADS et TOCTOU ?
5. **D5** — Le composant ProofCard dans le shell Browse (pas une app separee) est-il le bon endroit pour S68, ou faut-il une app sbfb-proof-card des maintenant ?
