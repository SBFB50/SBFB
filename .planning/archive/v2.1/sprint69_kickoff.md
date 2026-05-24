# Sprint 69 — Kickoff (Babel dogfood via Factory + pilote ferme + Gate 1)

**Ecrit** : 2026-05-22 (post-audit gate S68 PASS `0c2c2a8`).
**Type** : **sprint impair** — pas de phase dette obligatoire.
Un item 3/3 (Regle 2) a traiter : P2-I-2 delta body template.
**Tip master d'entree** : `0c2c2a8` (audit findings S68 PASS
0 P0, 0 P1, 3 P2, 2 P3).
**Phase 0 audit Sprint 68** : **DEJA JOUE** — `0c2c2a8` PASS.
Aucun fix requis.
**Version archive** : v2.1 — Protocole Neutre + Factory/RRV.
**Roadmap source** : `.planning/roadmap_v4_neutral_protocol_factory_rrv.md`.
Sprint 3 sur 3 (Arc 2 Factory + RRV @protocole + Canari).
**Dernier sprint Arc 2.** S70 = consolidation Gate 1 (D17).

---

## Sources context7 + WebSearch consultees (pre-gel)

| # | Source | Type | Date | Finding cle |
|---|--------|------|------|-------------|
| 1 | WebSearch "Ed25519 provenance verification CLI publish gate Rust 2025 2026" | WebSearch | 2026-05-22 | ed25519-dalek 2.2.0 stable, `VerifyingKey::verify_strict()` recommande pour rejeter weak keys. Pas de CLI publish-gate standard — chaque projet construit le sien. |
| 2 | WebSearch "F-Droid reproducible build provenance verification app publishing 2025 2026" | WebSearch | 2026-05-22 | F-Droid 2025 : 21% des 4061 apps construites de maniere reproductible et signees par les devs. Nouveau systeme rebuilder pour que des tiers verifient independamment. Post mai 2025 : "Making reproducible builds visible" — badges visuels par app. |
| 3 | WebSearch "SLSA provenance generation CLI tool verification attestation 2025 2026" | WebSearch | 2026-05-22 | SLSA v1.0 spec distributing-provenance : in-toto format + Sigstore signing. slsa-verifier CLI pour verification. GitHub attest-build-provenance action. Notre modele SBFB est SLSA L1 auto-atteste (pas L2+ car pas de builder trusted tiers). |
| 4 | context7 `/dalek-cryptography/ed25519-dalek` "verify signature VerifyingKey" | context7 | 2026-05-22 | `VerifyingKey::verify()` permet weak keys. `verify_strict()` ajoute un check weak key. `is_weak()` pre-validation. Recommandation : `verify_strict` pour provenance verification. |
| 5 | WebSearch "Rust CLI audit log JSONL structured logging publish pipeline 2025 2026" | WebSearch | 2026-05-22 | `audit-logging` crate (crates.io). `tracing` + `tracing-subscriber` JSON layer pour structured JSONL. `klp` CLI viewer JSONL. Pattern retenu : `serde_json::to_writer` + `\n` append fichier, pas framework lourd. |
| 6 | WebSearch "translation app P2P offline decentralized open source 2025 2026" | WebSearch | 2026-05-22 | LibreTranslate (self-hosted, offline). Argos Translate (Python, offline). RTranslator (Android, beta 2026). Aucun equivalent P2P distribue — SBFB Babel serait unique. Babel = reader app, pas moteur de traduction. |
| 7 | WebSearch "F-Droid in 2025 foundations" | WebSearch | 2026-05-22 | F-Droid jan 2026 rapport : Buildbot, repo tooling, metadata cleanup, reproducible builds infra. Le rebuilder pattern = tiers verifient independamment ce que le builder officiel produit. Inspire notre FG8 : le daemon verifie ce que Factory produit. |
| 8 | WebSearch "closed beta pilot program desktop app P2P acceptance test 2025 2026" | WebSearch | 2026-05-22 | Patterns pilote ferme : 2-3 testeurs reels, feedback structure, criteres go/no-go objectifs. LaunchDarkly/Centercode/HeadSpin guides. Pertinent pour Gate 1 criteria. |
| 9 | Code local `crates/nexus-coordinator-rs/src/provenance.rs` | Code local | 2026-05-22 | `verify_provenance()` existe : deserialise JSON, extrait signature hex, reconstruit canonical bytes (DOMAIN_PROVENANCE_V1), verifie Ed25519 via `nexus_core_rs::crypto::verify`. FG8 peut reutiliser cette fonction directement. |
| 10 | Code local `crates/sbfb-factory/src/publish.rs` | Code local | 2026-05-22 | Publish actuel : pre-valide manifest, POST deploy-from-repo. Ne verifie PAS la provenance retournee par le daemon. FG8 = ajouter verification Ed25519 post-publish. |
| 11 | Code local `crates/sbfb-factory/src/gates.rs` | Code local | 2026-05-22 | 3 fonctions `#[allow(dead_code)]` : fg5_sandbox, check_path_containment, fg7_preview. P3-I-2 nit S68. FG8 s'ajoute dans ce module. |
| 12 | Code local `crates/nexus-shell-daemon-core/src/preview.rs` | Code local | 2026-05-22 | Pas de `MAX_PREVIEW_ENTRIES` — P2-B-1 audit S68. A corriger S69. |
| 13 | Code local `crates/sbfb-factory/src/main.rs` | Code local | 2026-05-22 | 6 subcommands CLI actuels : create, validate, preview, publish, diff, scan-secrets. |
| 14 | SYNTHESIS `.planning/research/SYNTHESIS_factory_rrv_protocol.md` §3.4 | Code local | 2026-05-19 | Gates FG8 (Provenance Ed25519) + FG9 (Publish gate complete) specifiees. FG8 = verification post-publish que le daemon a signe correctement. FG9 = orchestration pipeline complet. |
| 15 | Roadmap v4 §Gate 1 conditions | Artefact local | 2026-05-19 | 9 criteres go/no-go : installation, connexion P2P, deploy app, Babel via Factory, feed sync, restart, stabilite 24h, RRV trouve Babel, Proof Card. |
| 16 | Audit findings S68 P2-B-1 | Artefact local | 2026-05-22 | Preview store sans `MAX_PREVIEW_ENTRIES` cap. Vecteur DoS local. Recommandation : cap 10 entries + THREAT_MODEL §preview. |

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 68 a livre le circuit de preuve complet : ProofCard struct avec
formule score 0-100 deterministe (7 risk factors, formula_version 1),
endpoint GET daemon, bridge method, composant UI dans Browse. La
preview ephemere (POST + TTL 30min + blob-serve) et le publish path
(Factory→daemon deploy-from-repo) ferment la boucle create→preview→
publish. Les gates FG4-FG7 (diff, sandbox dunce, lockfile, secrets)
sont implantees.

S69 est le **dernier sprint Arc 2** avant consolidation S70. Son
objectif est de valider la chaine complete en conditions reelles :
creer Babel avec Factory, le deployer sur le reseau, verifier que
RRV @protocole le trouve (`search?q=babel`), afficher sa Proof Card,
et confronter le tout a 2-3 testeurs reels dans un pilote ferme.

Les carries P2 s'accumulent (8 items reconduits). P2-I-2 (delta body)
atteint 3/3 MANDATORY. P2-B-1 (preview entries cap) et P2-I-3 (body
docs minimaliste) sont a 1/3.

### §1.2 Ancrage roadmap v4

Arc 2 (Factory + RRV @protocole + Canari), sprint 3 sur 3. Dependances
amont : S68 Proof Cards + publish path (satisfaites). Dependances
aval : Gate 1 go/no-go post-S69, puis S70 consolidation.

Gate 1 exige : Babel creee via Factory, deployee, visible Browse,
Proof Card affichee, `search?q=babel` retourne Babel, feed sync
entre 2+ noeuds, daemon stable 24h, restart propre, installation
par 2/3 testeurs sans aide.

### §1.3 Compteurs tests entree (tip `0c2c2a8`)

| Suite | Count |
|---|---|
| Rust nextest | 1419 |
| Vitest | 279 |
| size-limit | 6/6 |
| **Total** | **~1704** |

### §1.4 Pre-launch protocol policy (rappel)

- `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` restent a 1 jusqu'au
  go-live public. Un sprint qui change le canonical redefinit la v1
  courante, ne bump PAS la version.
- Feed extensible via raw-op : ajouter une operation ne bump PAS
  `FEED_FORMAT_VERSION`. Pas de nouvelle feed op dans S69.
- `#[serde(default)]` reste legitime pour robustesse runtime.
- ProofCard est un artefact local compute. Pas de bump feed.
- Factory gates (FG8) sont locales au CLI — pas de wire format
  protocolaire impacte.

---

## §2 Goal

Livrer le **dogfood Babel de bout en bout** : creer l'app Babel Reader
avec `sbfb-factory create`, la deployer via `sbfb-factory publish`, et
valider que le daemon la sert, la recherche, et affiche sa Proof Card.
Completer le publish gate (FG8 provenance Ed25519, FG9 pipeline
complet). Resoudre P2-I-2 3/3 MANDATORY (template body standardise).
Corriger P2-B-1 (MAX_PREVIEW_ENTRIES). Preparer la Gate 1 go/no-go
avec le test-protocol pilote ferme 2-3 personnes et les livrables
documentaires (verification.md, audit_plan S70).

**Critere SMART : toutes les rows fail-fast vertes au verification.md,
mesure binaire au Phase E wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 68

Audit S68 execute en session fraiche `0c2c2a8`.
Verdict : **PASS** (0 P0, 0 P1, 3 P2, 2 P3).
Aucun fix bloquant. Sprint 69 autorise.

P2 documentes :
- P2-I-1 → P2-I-2 carry 3/3 MANDATORY S69 : template body standardise
- P2-I-3 body docs(research) minimaliste (1/3)
- P2-B-1 `MAX_PREVIEW_ENTRIES` cap + THREAT_MODEL §preview (1/3)

P3 documentes :
- P3-I-1 deux fix(planning) groupables (nit)
- P3-I-2 `#[allow(dead_code)]` gates FG5/FG7/check_path_containment (nit)

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — FG8 Provenance Ed25519 verification dans Factory publish

**Sources consultees** :
- context7 `/dalek-cryptography/ed25519-dalek` (2026-05-22) : `VerifyingKey::verify_strict()` recommande, rejette weak keys. `is_weak()` pre-validation. ed25519-dalek 2.2.0 stable (crates.io, derniere release dec 2024, 0 CVE).
- WebSearch "Ed25519 provenance verification CLI publish gate Rust" (2026-05-22) : pas de CLI publish-gate standard dans l'ecosysteme. Chaque projet (F-Droid, SLSA verifier, cosign) construit son propre pipeline.
- WebSearch "SLSA provenance generation CLI verification" (2026-05-22) : SLSA v1.0 distributing-provenance spec. slsa-verifier Go CLI. Notre modele est SLSA L1 auto-atteste.
- Code local `crates/nexus-coordinator-rs/src/provenance.rs` (2026-05-22) : `verify_provenance()` deja implante avec canonical bytes + domain separation DOMAIN_PROVENANCE_V1. Reutilisable directement depuis sbfb-factory.
- Code local `crates/sbfb-factory/src/publish.rs` (2026-05-22) : publish actuel POST deploy-from-repo, ne verifie pas la provenance retournee. Gap a combler.
- WebSearch "F-Droid reproducible builds visible 2025" (2026-05-22) : F-Droid rebuilder pattern — un tiers verifie ce que le builder produit. FG8 = Factory verifie ce que le daemon produit.

**Retenu** : Apres le POST deploy-from-repo, Factory recupere la reponse (hash + provenance_hash), puis interroge `GET /api/v1/project/{id}/provenance` pour obtenir le record complet. Factory verifie la signature Ed25519 en utilisant le `node_id` du daemon (disponible dans `running.json` ou via `GET /api/daemon/status`). La verification utilise `verify_provenance()` de `nexus-coordinator-rs` (code partage entre daemon et factory via dep sur le crate). Si la verification echoue, Factory affiche une erreur et retourne un code non-zero. C'est le pattern F-Droid rebuilder adapte : Factory (le client) verifie ce que le daemon (le builder) a signe.

Le `node_id` du daemon est la cle publique Ed25519 (hex 64 chars). Factory le recoit soit de `running.json` (si present), soit via `GET /api/daemon/status` (endpoint existant). La verification est locale (pas de reseau P2P necessaire) — Factory compare la signature du record contre la cle publique connue du daemon.

**Rejete** :
- **Verification via slsa-verifier CLI externe** : ajoute une dep Go CLI, incompatible avec le modele mono-binaire Rust. Notre `verify_provenance()` fait le meme travail en Rust natif. (Source : slsa-verifier est Go-only, github.com/slsa-framework/slsa-verifier)
- **Verification par un tiers distant** : hors scope pre-launch. Il n'y a pas de rebuilder tiers. Le daemon auto-atteste (SLSA L1). Factory verifie cette auto-attestation. Un rebuilder P2P est post-Gate 1. (Source : SLSA spec v1.0 L1 vs L2)
- **Skip la verification** : le publish path sans FG8 est un tube aveugle — Factory envoie et espere. La verification post-publish garantit que le daemon n'a pas corrompu l'archive ou la provenance pendant le process. (Source : SYNTHESIS §3.4 FG8 rationale)

**Implications code** : `crates/sbfb-factory/src/gates.rs` (fn `run_gate_fg8_provenance`), `crates/sbfb-factory/src/publish.rs` (appel FG8 post-publish), `crates/sbfb-factory/Cargo.toml` (dep `nexus-coordinator-rs` pour `verify_provenance`).

### D2 — Babel Reader app creee via Factory dogfood

**Sources consultees** :
- WebSearch "translation app P2P offline open source 2026" (2026-05-22) : LibreTranslate (self-hosted), Argos Translate (Python offline), RTranslator (Android beta). Aucun equivalent P2P distribue. Babel = reader statique, pas moteur de traduction.
- Code local `crates/sbfb-factory/src/templates/` (2026-05-22) : template `static` existant (index.html + sbfb-bridge.js + README.md + .gitignore + SBFB.json). Template suffisant pour Babel Reader.
- Code local `examples/sbfb-explorer/` (2026-05-22) : Protocol Explorer = premiere app SBFB, template statique, ~12KB. Pattern reference pour Babel.
- Roadmap v4 D5 + D12 (2026-05-18) : "Babel = premier dogfood Factory" + "Babel canari = reader + fixtures (pas reviews)". Babel affiche des textes traduits pre-charges (fixtures), pas de traduction live.
- Recadrage PO 2026-05-21 : "Babel est cree avec Factory par le dogfood utilisateur, pas code comme livrable agent". L'agent ne code PAS Babel — l'agent prepare les outils (Factory + template), l'utilisateur cree Babel lui-meme.

**Retenu** : Babel Reader est une app statique HTML/CSS/JS creee via `sbfb-factory create --template static --name babel-reader`. Le contenu (textes traduits, UI de lecture) est ajoute par l'utilisateur. L'agent livre un template `static-reader` enrichi avec un squelette de lecteur (navigation entre "pages" de texte, dark theme, responsive) que l'utilisateur remplit. Les fixtures de traduction sont du contenu utilisateur, pas du code agent.

Le template `static-reader` est un enrichissement du template `static` existant : meme structure (index.html + SBFB.json + sbfb-bridge.js), avec un squelette UI minimal de type "book reader" (prev/next navigation, section titles). Le contenu reel est un placeholder remplace par l'utilisateur.

L'agent NE code PAS les traductions, NE connecte PAS de moteur LLM, NE fait PAS de NLP. Babel est un reader de contenu statique — le "dogfood" est la chaine Factory, pas le contenu.

**Rejete** :
- **Coder Babel complet avec traduction live** : hors scope. Le recadrage PO 2026-05-21 est explicite : "Babel est cree avec Factory par le dogfood utilisateur, pas code comme livrable agent". L'agent ne code pas le contenu de l'app. (Source : PO directive)
- **Template react-vite pour Babel** : surdimensionne. Babel est un reader statique. Le template `static` suffit. react-vite = scope cut S69+ (roadmap v4 §Scope ajustable). (Source : roadmap v4 tableau "Deplacable")
- **Pas de template enrichi (juste static brut)** : trop minimal pour le dogfood. Un squelette reader avec navigation aide l'utilisateur a demarrer. La valeur du template est le scaffolding, pas le contenu. (Source : feedback_approach.md "pick deepest", mais cadre par PO directive)

**Implications code** : `crates/sbfb-factory/src/templates/static-reader/` (NEW directory, template enrichi), `crates/sbfb-factory/src/template_engine.rs` (support template name "static-reader").

### D3 — FG9 Publish gate complete (pipeline orchestrant FG4-FG8)

**Sources consultees** :
- SYNTHESIS §3.4 (2026-05-19) : FG9 = "Publish gate complete" — orchestration des gates FG4-FG8 en pipeline avant/apres publish. Pre-publish : FG4 diff + FG5 sandbox + FG6 secrets. Post-publish : FG8 provenance.
- Code local `crates/sbfb-factory/src/publish.rs` (2026-05-22) : publish actuel fait `load_and_validate_manifest` puis POST. Pas d'integration des gates FG4-FG7 existantes.
- Code local `crates/sbfb-factory/src/gates.rs` (2026-05-22) : FG4 diff, FG5 sandbox, FG6 secrets existent mais ne sont pas wireees dans publish (3 `#[allow(dead_code)]`). FG7 preview check existe.
- WebSearch "F-Droid app publishing workflow 2025" (2026-05-22) : F-Droid : metadata → build → sign → verify → publish. Pipeline sequentiel avec abort si un step echoue.
- Code local `crates/sbfb-factory/src/main.rs` (2026-05-22) : subcommands independants. Pas de pipeline integre.

**Retenu** : `sbfb-factory publish` est enrichi pour executer le pipeline FG4→FG5→FG6→(publish)→FG8 automatiquement. Avant le POST deploy-from-repo : FG4 diff (informatif, pas bloquant — juste affiche les changements), FG5 sandbox (bloquant — refuse si path traversal), FG6 secrets (bloquant — refuse si secret detecte). Apres le POST : FG8 provenance Ed25519 (bloquant — refuse si signature invalide). Un flag `--skip-gates` permet de bypasser les pre-publish gates pour le debugging (pas la post-publish FG8 qui reste obligatoire).

Le pipeline est une fonction `run_publish_pipeline()` dans un nouveau module `pipeline.rs` qui appelle les gates dans l'ordre et s'arrete au premier FAIL bloquant. Les resultats sont affiches en console (pas encore JSONL — scope D4). FG10 (review gate automatise) reste scope cut S70+.

Les `#[allow(dead_code)]` de FG5/FG7/check_path_containment sont retires naturellement par le wiring dans le pipeline. Resolution du P3-I-2 nit S68.

**Rejete** :
- **Gates toutes bloquantes** : FG4 diff ne devrait pas bloquer — c'est informatif (montre les changements avant publish). Forcer un "0 diff" empecherait tout publish reel. (Source : SYNTHESIS §3.4 "FG4 = review aid, not blocker")
- **Pipeline externe (shell script orchestrant les subcommands)** : fragile, pas testable unitairement, pas Windows-friendly. Un module Rust `pipeline.rs` est testable et type-safe. (Source : code architecture sbfb-factory mono-binaire)
- **FG10 review gate ici** : trop ambitieux. FG10 requiert un reviewer automatise (lint, analyse statique). Scope cut S70+ (roadmap v4 §Scope ajustable). (Source : SYNTHESIS §3.4 "FG10 = automated review, post-pilot")

**Implications code** : `crates/sbfb-factory/src/pipeline.rs` (NEW), `crates/sbfb-factory/src/publish.rs` (refactor pour appeler pipeline), `crates/sbfb-factory/src/gates.rs` (retrait `#[allow(dead_code)]`).

### D4 — Factory audit log JSONL + P2-I-2 template body + P2-B-1 MAX_PREVIEW_ENTRIES

**Sources consultees** :
- WebSearch "Rust CLI audit log JSONL structured logging" (2026-05-22) : `serde_json::to_writer` + `\n` pour JSONL append. `tracing` + JSON layer pour structured logging. `audit-logging` crate (petit, BSS/OSS). Pattern retenu : JSONL simple sans framework.
- Audit findings S68 P2-I-1 → P2-I-2 3/3 MANDATORY (2026-05-22) : delta body diverge de 1 test. Recommandation : template body standardise avec compteur reel verifie par nextest AVANT redaction body.
- Audit findings S68 P2-B-1 (2026-05-22) : `MAX_PREVIEW_ENTRIES` cap absent. Un script local peut saturer la memoire daemon via previews 10MB en boucle.
- Code local `crates/nexus-shell-daemon-core/src/preview.rs` (2026-05-22) : `PreviewStore` HashMap sans borne. `MAX_PREVIEW_BYTES = 10MB` par entry mais pas de cap sur le nombre d'entries.

**Retenu** : Trois items dans une meme decision car ils sont petits et thematiquement lies (discipline/hygiene process).

**(a) Factory audit log JSONL** : Chaque invocation de `sbfb-factory` (create, validate, preview, publish, diff, scan-secrets) ecrit une ligne JSONL dans `~/.sbfb/factory-audit.log` avec timestamp, command, arguments, resultat (success/failure), et les resultats des gates. Format simple `serde_json::to_writer` + `\n`, pas de framework. Le fichier est en append-only. Pas de rotation (pre-launch, taille negligeable). La lecture se fait avec `cat` ou un viewer JSONL.

**(b) P2-I-2 3/3 MANDATORY — template body** : Pour chaque phase, le commit body section "## Delta tests" est redige APRES avoir lance `cargo nextest run --workspace --locked` et lu le compteur reel dans la sortie. Le process est : code → tests → nextest output → compteur reel → redaction body. Ce n'est pas un changement de code mais de **process** : documenter la procedure exacte dans le plan §Phase de chaque phase pour que l'executeur la suive. L'agent cree un script `scripts/count-tests.sh` (ou PowerShell equivalent) qui parse la sortie nextest et affiche les compteurs structures.

**(c) P2-B-1 MAX_PREVIEW_ENTRIES** : Ajouter `const MAX_PREVIEW_ENTRIES: usize = 10;` dans `preview.rs`. `load()` verifie `guard.len() >= MAX_PREVIEW_ENTRIES` avant insertion et retourne `PreviewError::TooManyEntries` si depasse. Documenter le vecteur dans THREAT_MODEL.md §preview (nouveau paragraphe dans §12 ou nouvelle section §13).

**Rejete** :
- **Framework audit logging (tracing + JSON layer)** : surdimensionne pour un CLI qui s'execute ponctuellement. `tracing` est pour les services long-running. Factory est un one-shot CLI. (Source : tracing docs "designed for programs with long-lived operations")
- **Ne pas resoudre P2-I-2** : MANDATORY 3/3. Pas d'option. (Source : README §6.2.1 Regle 2)
- **MAX_PREVIEW_ENTRIES via LRU eviction au lieu de reject** : LRU evicterait le preview le plus ancien silencieusement. Pour un outil dev, mieux vaut une erreur explicite "too many previews, close one first" que la surprise de perdre son preview. (Source : principe de moindre surprise)

**Implications code** : `crates/sbfb-factory/src/audit_log.rs` (NEW), `crates/sbfb-factory/src/main.rs` (appel audit log), `crates/nexus-shell-daemon-core/src/preview.rs` (MAX_PREVIEW_ENTRIES), `docs/security/THREAT_MODEL.md` (§preview), `scripts/count-tests.sh` (NEW).

### D5 — Gate 1 test-protocol + pilote ferme prep

**Sources consultees** :
- Roadmap v4 §Gate 1 (2026-05-19) : 9 criteres go/no-go (installation, connexion P2P, deploy, Babel Factory, feed sync, restart, stabilite 24h, search, Proof Card).
- WebSearch "closed beta pilot program desktop app acceptance test 2025" (2026-05-22) : patterns pilote ferme. Feedback structure, criteres binaires, formulaire standardise.
- Recadrage PO 2026-05-21 : Gate 1 se valide sur @protocole + Proof Cards + publish + Babel dogfood.
- Code local `crates/nexus-launcher/src/main.rs` (2026-05-22) : launcher operationnel (tray icon + spawn daemon + open browser).

**Retenu** : L'agent produit un document `docs/release/GATE1_TEST_PROTOCOL.md` qui formalise les 9 criteres Gate 1 en un protocole de test executable par 2-3 personnes. Chaque critere est une **procedure pas-a-pas** (pas juste "testez l'installation" mais "1. Telecharger l'installeur NSIS, 2. Double-cliquer, 3. Accepter UAC, 4. Verifier raccourci Start Menu, 5. Lancer, 6. Verifier que le navigateur ouvre Browse"). Le document inclut un formulaire de feedback (table a remplir : critere | resultat | notes | bloqueur).

Le pilote ferme n'est PAS dans le code — c'est un processus humain. L'agent ne deploie pas de serveur, ne collecte pas de telemetrie, ne fait pas de beta enrollment. L'agent livre :
1. Le test protocol document (procedures pas-a-pas)
2. La checklist Gate 1 remplie par le daemon E2E (l'agent valide les 9 criteres localement)
3. Les instructions d'installation pour les testeurs

Le verdict Gate 1 est pris par l'utilisateur (FlowUP) apres le retour des testeurs. L'agent documente, ne decide pas.

**Rejete** :
- **Telemetrie automatisee** : pas de collecte de donnees. Pattern OpenBSD solo maintainer (vision_model.md). Les testeurs remplissent un formulaire texte. (Source : vision_model.md "pas de partnership/infrastructure institutionnelle")
- **Infrastructure pilote (serveur de distribution, update channel)** : overkill pour 2-3 personnes. Le pilote se fait par envoi direct du binaire/installeur. (Source : vision_model.md + pilote ferme = 2-3 personnes max)
- **Skip Gate 1 et passer a S70** : les criteres go/no-go sont contractuels (roadmap v4). Sans validation pilote, S70 consolide dans le vide. (Source : roadmap v4 §Gate 1 "Si > 5 bugs P0/P1 : sprint fix dedie avant S70")

**Implications code** : `docs/release/GATE1_TEST_PROTOCOL.md` (NEW), `.planning/active/sprint69_verification.md` §Gate 1 checklist.

---

**Acknowledged review findings (G1)** :

Scoring : D1 ok, D2 ok, D3 ok, D4 warning, D5 ok.
Rigor signal G4 satisfait (1 warning sur 5).

D4 warning : le JSONL audit log n'a pas de source < 90 jours specifique — les patterns (serde_json append, JSONL format) sont stables depuis des annees et ne changent pas. Le warning est acknowledge, pas adjust. Le risque est nul car le format est trivial (1 struct serialisee par ligne).

---

## §5 Plan Phase outline A..E

### Phase A — P2-I-2 template body + P2-B-1 preview cap + audit log

Scope : resoudre les 2 items carry/audit (P2-I-2 3/3 MANDATORY
template body, P2-B-1 MAX_PREVIEW_ENTRIES). Creer le module audit log
JSONL pour Factory. Script `count-tests.sh`. THREAT_MODEL §preview.

### Phase B — FG8 Provenance Ed25519 + FG9 Publish pipeline

Scope : implanter FG8 (verification provenance Ed25519 post-publish),
FG9 (pipeline integre FG4→FG5→FG6→publish→FG8 dans un module
pipeline.rs), retirer les `#[allow(dead_code)]` gates. Wiring dans
`sbfb-factory publish`.

### Phase C — Babel Reader template + Factory dogfood E2E

Scope : template `static-reader` pour Babel, test du flow complet
create→validate→preview→publish→browse→search→proof-card. Test E2E
documentaire (pas de test automatise multi-daemon, mais execution
manuelle documentee du flow).

### Phase D — Gate 1 test protocol + docs + P2 absorbables

Scope : GATE1_TEST_PROTOCOL.md avec 9 procedures pas-a-pas,
instructions installeur pour testeurs, expose les subcommands FG5
sandbox et FG7 preview-check en CLI (resolution P3-I-2).

### Phase E — Verification + wrap-up + audit_plan S70

Scope : verification.md fail-fast, audit_plan S70, CLAUDE.md,
SPRINT_LOG.md, memory update, Gate 1 self-checklist.

---

## §6 Items carry/dette

### Items 3/3 (traitement Sprint 69)

| Item | Reports | Phase S69 | Exit condition |
|---|---|---|---|
| P2-I-2 delta body | 3/3 | Phase A | Script count-tests + procedure documentee dans chaque §Phase du plan. Verification : chaque commit body §Delta tests = compteur reel nextest. |

### Carry absorbes S69

| Item | Reports | Phase S69 | Exit condition |
|---|---|---|---|
| P2-B-1 MAX_PREVIEW_ENTRIES | 1/3 | Phase A | `const MAX_PREVIEW_ENTRIES: usize = 10` dans preview.rs + test + THREAT_MODEL §preview |
| P3-I-2 dead_code gates | nit | Phase D | `#[allow(dead_code)]` retires de gates.rs via wiring pipeline + CLI subcommands |

### Carries reconduits S70

| Item | Reports | Justification |
|---|---|---|
| P2-A-1 rand blocker upstream | exemption→exemption | upstream rand 0.9 non publie (derniere verif crates.io 2026-05-22). Dep transitive iroh 0.98. Aucune action possible. |
| P2-AUDIT-2 iroh transitives | exemption→exemption | herite du pin iroh 0.98. Evaluate a Gate 1 post-S69 — si pilote ne revele pas de bug iroh, report S70. Si bug iroh, sprint fix dedie. |
| P2-G-1 exe lock intermittent | monitoring→monitoring | non reproductible depuis S62 (7 sprints). Monitoring passif. Si reproduit : priorite immediate. |
| T-NN+2 iframe Rust-wasm | bloque→bloque | toolchain gaps wasm32 inchanges (tract opset 19, ort wasm32 browser target). Trigger : ort stable wasm32 ou tract opset 19. |
| P2-I-3 docs body minimaliste | 1/3→2/3 | gap process formel. Les commits docs(research) significatifs >100 lignes doivent avoir 3-5 lignes body. A surveiller S70. |
| LT-2 Radicle sortie | trigger PENDING | tag v1.0 pose localement, pas pousse origin. La condition "push tag + GitHub Release" n'est pas remplie. Le push est une decision utilisateur, pas agent. |
| LT-5 redundancy persistence | hors-sprint | reclassifie S26. Condition : premier deploiement multi-worker OU tag v1.0 go-live. |
| LT-7 worker quorum E2E | post-tag | Tier 1+2 DONE (S55). Tier 3 P2P infra validee (S60). Worker quorum E2E carry post-tag : workers non deployes sur VPS/Mac. |

### Attention 3/3 S70

| Item | Reports apres S69 | Raison |
|---|---|---|
| P2-I-3 docs body minimaliste | 2/3 | Passera 3/3 au S70. Devra etre resolu dans le plan S70. |

---

## §7 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | SearchManifest wire format + gossip | S71 | Protocole reseau. Hors scope Arc 2. Prereq Arc 3 (D17 : S70 = consolidation, pas SearchManifest). |
| 2 | Page React /factory | S70+ | CLI suffit pour S69 et le pilote. L'UI viendra quand la boucle CLI est validee par le dogfood. |
| 3 | @dev index tree-sitter | S70+ | Decision PO 2026-05-21 : @dev non bloquant Gate 1. |
| 4 | Template react-vite | S70+ | 3 templates suffisent (static + static-storage + static-reader). react-vite si demande pilote. |
| 5 | CuratorVouched UI shell | S70+ | Le vouch est dans le feed (S67). L'UI de curation dans le shell est post-pilote. |
| 6 | FG10 Review gate | S70+ | Lint/analyse statique automatise. Depend de l'outillage post-Gate 1. |
| 7 | Fuzzing cargo-fuzz/proptest | post-Gate 1 | Hors scope fonctionnel. Utile apres stabilisation. |
| 8 | Feed format version bump | post-launch | Pre-launch policy. |
| 9 | ProofCard comme feed op | S71+ | Candidat SearchManifest. S69 = compute local seulement. |
| 10 | Diff engine avance (semantique) | S70+ | S68 livre diff basique (fichiers). Diff semantique = post-pilote. |
| 11 | Multi-template switching UI | S70+ | CLI template choice suffit pour S69. |
| 12 | Factory update-check automatique | post-launch | Pas de telemetrie, pas d'auto-update (vision_model.md). |
| 13 | Babel traduction live (moteur LLM) | post-launch | Babel S69 = reader statique. Le moteur de traduction est l'app future, pas le dogfood. |
| 14 | iroh 1.0 upgrade | Gate 1 decision | Evalue post-S69 (roadmap v4 §Gate 1 : "Si pilote revele bugs fixes en iroh 1.0, upgrade prioritaire"). |

---

## §8 Tracabilite scope

| Item S68 "What's NOT" | Sprint + Phase S69 |
|---|---|
| SearchManifest wire format + gossip | Reconduit S71 (D17 : S70 = consolidation) |
| Page React /factory | Reconduit S70+ |
| Babel dogfood via Factory | **Phase C S69** (D2 retenu) |
| @dev index tree-sitter | Reconduit S70+ |
| Template react-vite | Reconduit S70+ |
| Factory audit log JSONL | **Phase A S69** (D4 retenu) |
| CuratorVouched UI shell | Reconduit S70+ |
| FG8 Provenance Ed25519 | **Phase B S69** (D1 retenu) |
| FG9 Publish gate complete | **Phase B S69** (D3 retenu) |
| FG10 Review gate | Reconduit S70+ |
| Fuzzing cargo-fuzz/proptest | Reconduit post-Gate 1 |
| Feed format version bump | Reconduit post-launch |
| ProofCard comme feed op | Reconduit S71+ |
| Diff engine avance | Reconduit S70+ |

---

## §9 Risk register

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | sbfb-factory dep sur nexus-coordinator-rs pour FG8 cree un couplage | Medium | Medium | Dep read-only sur le crate (verify_provenance est pur, pas de state). Si le couplage gene, extraire verify dans sbfb-manifest. |
| R2 | Babel dogfood echoue car le publish path a un bug non couvert par les tests unitaires | Medium | High | Test E2E documentaire Phase C : execution manuelle du flow complet avec screenshots. Si bug, fix immediat. |
| R3 | Pilote ferme revele > 5 bugs P0/P1 → sprint fix dedie avant S70 | Medium | High | Gate 1 test protocol avec criteres binaires. Si > 5 P0/P1 : sprint S69.5 fix (roadmap v4 §Gate 1). |
| R4 | P2-I-2 template body non resolvable par process seul (erreur humaine) | Low | Low | Script count-tests parse nextest output. L'executeur copie les compteurs du script, pas de sa memoire. Verification au commit. |
| R5 | MAX_PREVIEW_ENTRIES cap trop bas (10) bloque le workflow dev | Low | Low | La valeur 10 est un defaut raisonnable (10 × 10MB = 100MB max). Augmentable post-launch si besoin. |
| R6 | Template static-reader trop minimal pour le dogfood | Medium | Low | Le template est un squelette. L'utilisateur ajoute le contenu. Si le squelette manque de structure, l'utilisateur edite l'HTML directement. |
| R7 | Gate 1 critere "installation par 2/3 testeurs sans aide" echoue sur macOS | Medium | Medium | Le .dmg macOS existe (S60). Si probleme macOS, documenter le workaround. Le pilote Windows est prioritaire. |

---

## §10 Audit gate pattern — rappel

Phase 0 S68 jouee (`0c2c2a8` PASS). Phase E du sprint devra produire :
- `sprint69_verification.md` (self-report fail-fast)
- `sprint70_audit_plan.md` (plan pour Phase 0 S70)
- Mise a jour `docs/rust/PATTERNS.md` si nouveaux patterns
- Mise a jour `docs/shell/PATTERNS.md` si nouveaux patterns

---

## §11 Checkpoint de validation

1. **D1 (FG8)** — Factory verifie la provenance Ed25519 post-publish en dependant de `nexus-coordinator-rs` pour `verify_provenance()`. Ce couplage crate est-il acceptable, ou faut-il extraire la verification dans `sbfb-manifest` (crate partage) ?

2. **D2 (Babel)** — L'agent livre un template `static-reader` mais ne code PAS le contenu Babel. L'utilisateur cree Babel lui-meme avec Factory. Est-ce suffisant pour le dogfood, ou faut-il que l'agent cree une app Babel minimale fonctionnelle (avec fixtures) ?

3. **D3 (FG9)** — Le pipeline publish integre FG4-FG6 en pre-publish et FG8 en post-publish, avec `--skip-gates` pour les pre-publish. FG4 diff est informatif (pas bloquant). Est-ce la bonne politique, ou FG4 diff devrait-il aussi bloquer si des fichiers inattendus sont detectes ?

4. **D4 (audit log + P2)** — Le JSONL est un append simple dans `~/.sbfb/factory-audit.log` sans rotation. Pour le pre-launch c'est suffisant, mais faut-il prevoir un `--no-audit-log` flag pour les CI/tests ?

5. **D5 (Gate 1)** — Le test protocol est un document statique que les testeurs remplissent manuellement. Faut-il plutot un script interactif qui guide le testeur etape par etape et collecte les resultats ?
