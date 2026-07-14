# Sprint 82 Phase H — Preflight (G8)

Date : 2026-07-14. Phase H ferme la dette patterns S81 Track C :
re-ancrage T20 relay-cert-pinning (S81-C-3), résolution/requalification
C-1/C-2, vérification §P73, balayage prose historique H-3 (~:862,
routage Phase E), et factorisation/tripwire du magic-string suffixe
backup redb-v2-tuples dupliqué 2 crates (F-D5-01).
Preflight ultracode = Workflow 10 agents (5 scans S2a/S2b/S1a/S3/S4+S1b
+ 5 vérifications adversariales par scan, pipeline sans barrière,
opus-4-8[1m]). Toutes les ancres ci-dessous re-vérifiées au disque le
2026-07-14 par DEUX passes indépendantes (scan + vérif adversariale ;
arbre propre, tip `d2705b7`). Deux réfutations adversariales matérielles
intégrées (candidat M1 déjà clos ; citation d'import test fabriquée).

## Verdict: PLAN-ADAPT

Le plan est exécutable, aucune décision Day-0/PO n'est contredite
(aucun DESIGN-CONFLICT : « résolus/requalifiés » du plan couvre la
disposition C-1/C-2 ci-dessous, et le defer T20 INTANGIBLE est
strictement respecté — 0 câblage). Mais **six faits du plan sont
incomplets ou inexacts** et imposent une exécution corrigée :

1. **La prose exacte de S81-C-1/C-2 est INTROUVABLE — la consigne
   « ré-extraire depuis les phase-reviews » repose sur une prémisse
   fausse.** Les deux IDs n'existent que comme identifiants sans corps :
   `sprint81_audit_findings.md:100` (« Findings : S81-C-1 (P2),
   S81-C-2 (P2), … ») + table :204 ; grep `S81-C-` sur les 17
   phase-reviews archivées (`.planning/archive/v2.1/`) = **0 hit**
   (les reviews utilisent leurs propres labels : M1, M2, D6-N,
   OBSERVATION-N). Track C est produit « diff-first anti-anchoring »
   (`sprint81_audit_plan.md:12`) : findings PROPRES de l'auditeur du
   gate, sortie brute jamais persistée. Il n'existe AUCUN crosswalk
   phase-review→S81-C-N. Corroboration négative : le seul P2 patterns
   des reviews (Phase F D6-1, `sprint81_phase_f_review.md:224-228`) est
   « CONFIRMÉ, APPLIQUÉ » (fermé, rustdoc in-code) ; le candidat M1
   (drift iroh 0.97/0.98, `sprint81_phase_e_review.md:321-330`) est
   **RÉFUTÉ adversarialement comme dette vivante** : `tls_pinning.rs:33`
   ET `PATTERNS.md:1004` disent tous deux « iroh 0.97 » (récit
   historique harmonisé in-phase — 0 occurrence « 0.98 »).
   → **Exécution : consigner INTROUVABLE (avec les preuves ci-dessus),
   puis RE-DÉRIVER une passe de fidélité PATTERNS.md↔code actuelle
   (méthode d'audit Track C), étiquetée honnêtement « ré-audités
   Phase H » — jamais « ré-extraits ». Si la passe ne révèle pas 2 P2
   vivants, REQUALIFIER C-1/C-2 (disposition motivée, compteur §6.2.1
   soldé) — le plan autorise explicitement « résolus/requalifiés ».
   La règle « ne pas fabriquer » est respectée par construction.**

2. **T20 (S81-C-3) : le pointeur concrètement faux est à
   `PATTERNS.md:1025-1026`, pas :974.** Le « :974 » du plan désigne le
   header du bloc (`### T20 — relay cert-pinning wire: upstream hook
   LANDED in iroh 1.0`). Le bloc contient DEUX pointeurs SBFB-side
   contradictoires : le blockquote status-update S81-E (:976-998) pointe
   CORRECTEMENT `crates/nexus-core-rs/src/node.rs` (:988) ; le corps
   historique S19 conservé verbatim (:997-998, :1000-1042) pointe encore
   « likely in `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` »
   (:1025-1026) — FAUX : `iroh_runtime.rs` ne contient plus AUCUN
   `Endpoint::builder`/`ca_tls_config`/`relay` (grep = 0 hit). Ancre
   valide vérifiée : `node.rs` (chokepoint `Endpoint::builder` :324,
   `relay_mode(RelayMode::Custom)` :510, démo test-only
   `.ca_tls_config(CaTlsConfig::insecure_skip_verify())` :858). Les
   références AMONT du blockquote sont VÉRIFIÉES EXACTES contre le
   registry (=1.0.1 : `endpoint.rs:713` `ca_tls_config`, iroh-relay
   `tls.rs:141` `custom_server_cert_verifier`, re-export `iroh::tls`
   `tls.rs:23`) — ne PAS les « corriger ». Origine du finding
   ré-extraite conformément : E-review M2 (`sprint81_phase_e_review.md:365-371`,
   disposition :400). Option de fidélité (non requise) : harmoniser le
   namespace du blockquote `iroh_relay::tls::CaTlsConfig` →
   `iroh::tls::CaTlsConfig` (l'import réel : `node.rs:834`,
   `pkarr_resolver.rs:41`).
   → **Exécution : annotation datée [re-anchored S82 Phase H (S81-C-3)]
   au point :1025-1026 sans réécrire la prose S19, redirigeant vers
   node.rs + le blockquote autoritaire :988. Defer INTANGIBLE tenu
   (ledger Phase E §5.2 :187-197) : 0 câblage TLS, le carry sécurité
   T20 reste OPEN (slot hardening dédié). Prose au présent-vrai.**

3. **F-D5-01 : trois faits du plan corrigés.** (i) Le littéral daemon
   est à `crates/nexus-shell-daemon/src/runtime.rs:2751` (fn
   `pub(crate) docs_migration_backup_path`, déclarée :2750) — le
   « `runtime.rs:2580` » du plan §218 est un pointeur DÉRIVÉ (chaîne
   2580 [review F :136] → 2557 [codex F] → 2558 [review H] → 2751
   [courant]) ; et c'est le crate BINAIRE `nexus-shell-daemon`, PAS
   `-core` (grep sur -core = 0 hit). (ii) `pub(crate)` dans un binaire
   = INATTEIGNABLE cross-crate → la factorisation DOIT passer par une
   `pub const` dans la LIB `nexus-core-rs` (le daemon dépend déjà de la
   lib — `runtime.rs:189` `nexus_core_rs::IdentityMode` ; le test
   `crates/nexus-core-rs/tests/store_migration.rs` l'atteint par
   construction — NOTE adversariale : le test n'importe PAS
   `nexus_core_rs` aujourd'hui [la citation « :37 use nexus_core_rs »
   du scan était FABRIQUÉE, réfutée] — l'import sera AJOUTÉ).
   (iii) Les 2 littéraux ne sont PAS byte-identiques : test = suffixe
   seul `.backup-redb-v2-tuples` (`store_migration.rs:76`), daemon =
   nom COMPLET `docs.redb.backup-redb-v2-tuples` (:2751) → le tripwire
   compare suffixe-à-suffixe (`ends_with`), la base `docs.redb` reste
   un choix propre au daemon. Owner upstream vérifié :
   `iroh-docs-0.101.0/src/store/fs/migrate_redb_v2_tuples.rs:160-166`
   (`p.push(".backup-redb-v2-tuples")` :162 — littéral privé inline,
   NON exporté : la const SBFB ne peut que refléter ; le tripwire
   détecte le drift upstream en exécutant la migration RÉELLE).
   Texte exact F-D5-01 (P3, D5, « Carry K ») ré-extrait verbatim :
   `sprint81_phase_f_review.md:136`.
   → **Exécution : (a) `pub const MIGRATION_BACKUP_SUFFIX` dans la lib
   nexus-core-rs, doc-commentée provenance upstream ; (b) daemon
   consomme la const (nom complet = base + const) ; (c) le test importe
   la const (const locale supprimée) ; (d) +1 test tripwire dédié
   (migration réelle → sibling produit `ends_with(const)`). Delta
   nextest attendu +1 (Win 2099→2100, Docker 2103→2104). SPDX si
   nouveau fichier .rs.**

4. **Le balayage prose historique « H-3 » ~:862 EST en Phase H, et
   c'est un HOMONYME de S81-H-3 (Phase I).** `PATTERNS.md:850`
   (« CLOSED Sprint 9 Phase A (H-3) », entrée wheel editable install
   drift, prose jusque ~:862) parle au PRÉSENT de `setup.sh` +
   `.githooks/post-merge` — SUPPRIMÉS en Phase E S82 (commit `f727f8c`,
   « DELETED ») — et référence « README §4.3 » de façon STALE (la
   section existe [`docs/claude/README.md:868`, vérification avant
   commit] mais ne documente pas ces scripts). Routage Phase H =
   `sprint82_phase_e_review.md:58-62` + :315-316. S81-H-3 (Track H
   hardening, P3, `sprint81_audit_findings.md:160-161`) reste routé
   Phase I (`sprint82_plan.md:225` + ledger :121) — ne PAS le traiter
   en Phase H.
   → **Exécution : réécrire l'entrée :850-:862 au récit passé immuable
   (retirer le présent sur scripts supprimés + corriger la réf README)
   et ANNOTER la désambiguïsation des deux « H-3 » (finding audit
   Sprint 9 wheel-drift ≠ S81-H-3 Track H hardening).**

5. **§P73 FIDÈLE re-confirmé — 0 édition requise.** Les 4 claims
   re-vérifiés indépendamment par 2 passes : (1) attestation interceptée
   dans `ProtocolHandler::accept` AVANT le forwarder
   (`shard.rs:323-345`), `deny_unknown_fields` (:425/:470/:573/:626),
   discriminants `kind` (:578/:608/:631/:696), consts payload dans 5
   fichiers ; (2) two-node DIRECT strip (`blobs.rs:428-449`) + groupe
   nextest `two-node-convergence = { max-threads = 2 }`
   (`.config/nextest.toml:27`) ; (3) seam env→plan pure
   (`discovery_override.rs:131`) ; (4) `GossipCmd::JoinPeers` commande
   runtime (`runtime.rs:1522`, producteur unique `http.rs:934`, test
   :7247).
   → **Exécution : consignation « §P73 re-vérifié FIDÈLE Phase H »
   (preflight + commit body), texte non touché.**

6. **Wire/deps/gates : 0 bump, 0 dep — avec contraintes de prose et de
   placement.** Census `DOMAIN_CENSUS_FROZEN=25` intact
   (`MIGRATION_BACKUP_SUFFIX` ne matche pas `DOMAIN_[A-Z0-9_]+_V[0-9]+`,
   `check-frontier-contracts.sh:190-192`) ; le grep committé §P70
   « = 25 »/« 22 » (`PATTERNS.md:3953-3957`) ne doit PAS être touché.
   PROMISE_RE ne scanne QUE `crates/` + `web/src/` (find :127-129 ;
   docs/ hors scope par construction) — la prose T20/§P73 n'est pas
   gatée MAIS tout commentaire in-code du tripwire l'EST : formuler le
   defer T20 et les commentaires au PRÉSENT-VRAI (classe
   documentée-hors-gate §P70:3932-3934 — « deferred / OPEN / routed to
   a dedicated hardening slot », jamais « will be wired when Sprint N »).
   Placement const : PAS dans `canonical.rs` ni `schemas/`, pas de nom
   `*_VERSION` (Check 4 hook `grep canonical\.rs|schemas/|_VERSION`
   :218 + census). Hook lightcheck : Checks 0 (whitespace) / 7 (codex
   brut) / 8 (ce preflight) / 9 (9 sections body) STRICT ; Check 1 si
   `+pub mod` ajouté (fichier stagé).

## Plan d'exécution adapté (résumé opérationnel)

1. `docs/rust/PATTERNS.md` T20 : annotation re-ancrage datée à
   :1025-1026 (corps S19 verbatim préservé) + option harmonisation
   namespace blockquote (`iroh::tls::CaTlsConfig`). Présent-vrai.
2. `docs/rust/PATTERNS.md` ~:850-862 : réécriture passé immuable +
   note de désambiguïsation H-3 (Sprint 9) vs S81-H-3 (Phase I).
3. Passe de fidélité Track C (C-1/C-2) : re-dérivation documentée +
   disposition (résolution des écarts trouvés OU requalification
   motivée) — consignée dans ce preflight complété + commit body.
4. §P73 : consignation FIDÈLE (0 édition).
5. Code : `pub const MIGRATION_BACKUP_SUFFIX` (lib nexus-core-rs) ;
   daemon `runtime.rs:2751` consomme la const ; test
   `store_migration.rs` importe la const ; +1 test tripwire
   upstream-drift. Delta nextest +1.
6. Suites §7.4 + gates docs + review Workflow + Codex + commit
   (body 9 sections ; critère machine du plan ré-ancré :2750-2751
   consigné au body — le plan lui-même n'est pas édité, PLAN-ADAPT).

## Passe de fidélité re-dérivée (exécutée in-phase — disposition C-1/C-2)

Workflow 13 agents (8 tranches : rust/PATTERNS.md ×6 + shell/PATTERNS.md
×2, chaque tranche vérifiée adversarialement ; opus-4-8[1m] ;
1.22M tokens, 265 tool calls). Règles de qualification strictes
(récit passé immuable = non-finding ; classe présent-vrai §P70 =
non-finding ; T20/H-3/§P73/grep §P70 exclus car traités par ailleurs).
Résultat : **21 findings vérifiés — 14 P2 + 7 P3 après re-cotation
adversariale (4 UPGRADE-P2 : frost, sybil-tail, perf-map seam, zip ;
0 REFUTED parmi les reportés ; 1 rider de
scan réfuté en amont [M1 déjà harmonisé] + 1 citation fabriquée
détectée et écartée [import test inexistant])**. Tranches rust-1,
rust-2, rust-5 : 0 finding (sections vérifiées symbole par symbole).

Les 21 findings, TOUS corrigés in-phase (doc-only) :
- **rust-3** : champs `worker.toml` fantômes §P35
  (`max_tasks_before_restart`→`max_tasks`,
  `cuda_wipe_enabled`→`vram_wipe` — une clé mal épelée est
  silencieusement ignorée par serde) [2 P2] ; symbole `EphemeralState`
  →`LifecycleState`/`EphemeralLifecycle` + graphe d'états réel
  `Ready→Running→WipePending→(RestartPending|Ready)→Exiting` [2 P3] ;
  `frost-ed25519` v2.x→v3.x (3.0.0 au lock) [P2 upgradé].
- **rust-4** : 5 sections décrivant le coordinateur Python supprimé
  S45/S50 au présent — §P36 module `redundancy.py` (re-ancré
  `validator.rs` quorum exact-equality `result_text`), §P37 détecteur
  watermark (re-ancré au PORT RUST `watermark_detector.rs` S40 Phase C
  — le scan proposait la mise au passé, la vérité est meilleure : le
  port existe), §P40 case-sensitivity (passé immuable), §P41 constantes
  canary (`canary_registry.py` retiré), §P43 rôle 3 keypair [5 P2] ;
  `TaskCanonical`/`#[serde(skip)]` fictifs → `task_canonical_bytes`
  [P3] ; 3 annotations de lignes dérivées (public_feed, node.rs,
  runtime.rs) → remplacées par des ancres SYMBOLE (root-cause de la
  classe « pointeur qui pourrit ») [3 P3].
- **rust-6** : §P68 « closes SYBIL-SEEDER-TAIL (S75) » sur-vendu — le
  sampling blake3 ferme la tail WORKER-PLACEMENT seulement,
  `seeders_recent` trie encore lexicographiquement (carry S75 reste
  OPEN, dit explicitement) [P2 upgradé] ; §P69 « the glue doc.set lives
  in the daemon » faux — seam non câblée, daemon passe un
  `PerfMap::new()` vide [P2 upgradé].
- **shell-1** : P9 réécrit — l'architecture proxy coordinator décrite
  au présent est l'INVERSE du code depuis S50-S51 (appels daemon
  DIRECTS same-origin via `daemon.ts`, enveloppe `DaemonResult<T>`
  construite côté client) [P2] ; exception plain-`fetch` AppsTab
  fermée (helper typé `getAppTabDescriptor`) [P2] ;
  `loopback_cors_layer`→`cors_layer` [P3].
- **shell-2** : P39 stub `live_shard_session` SUPPRIMÉ S81 Phase I
  (lookup = `ShardSessionRegistry` réel) [P2] ; crate `zip` 2.6→8.6.0
  au lock, contrainte déclarée 8.5 [scan P3, requalifié P2 par le
  vérificateur — l'un des 4 UPGRADE-P2 ; le « 8.5 au lock » du scan
  était lui-même inexact, corrigé au réel 8.6.0 par la review de
  phase (finding DC-H-ZIP-1/H-PROC-1)].

**Disposition finale : S81-C-1/C-2 SOLDÉS par re-dérivation (voie (a)
du verdict PLAN-ADAPT #1) ; S81-C-4/C-5 (P3, routés Phase H ledger E
:116) subsumés par la même passe ; S81-C-3 re-ancré.** Note datée
posée dans `sprint81_audit_findings.md` Track C.

## Détail des scans (verdicts adversariaux)

- **S2a-C1C2** (extraction C-1/C-2) : C1-TEXT CONFIRMED, C2-TEXT
  CONFIRMED, C2-STATUS CONFIRMED, C1-STATUS UNCERTAIN (cœur confirmé —
  OPEN + non-ré-extractible ; rider « M1 vivant » RÉFUTÉ :
  `tls_pinning.rs:33` lit « iroh 0.97 », pas « 0.98 » — M1 harmonisé
  in-phase, CLOS, non actionnable).
- **S2b-T20** : 4/4 CONFIRMED (texte courant :974-1042 ; pointeur faux
  :1025-1026 ; ancre valide node.rs + amont registry exact ; defer
  intact ledger §5.2). Nuances de wrap de ligne non matérielles.
- **S1a-FD501** : FD501-TEXT / DUP-SITE-1 / DUP-SITE-2 / UPSTREAM-OWNER
  CONFIRMED (verbatim) ; TRIPWIRE-DESIGN UNCERTAIN — citation
  d'atteignabilité côté test FABRIQUÉE (« store_migration.rs:37
  use nexus_core_rs » n'existe pas ; le test ne référence jamais
  nexus_core_rs aujourd'hui). Faisabilité maintenue par construction
  (tests/ de core-rs compilent contre la lib) ; l'import est un AJOUT
  de la phase. 3 conflits plan-vs-réalité CONFIRMED (pointeur dérivé
  :2751 ; crate binaire pas -core ; non byte-identiques).
- **S3-P73-H3** : 4/4 CONFIRMED (P73 fidèle symbole par symbole ;
  homonymie H-3 réelle + routage Phase H prouvé git `f727f8c` ;
  cohérence THREAT/HARDENING — WebPKI-only + carry S82+, re-ancrage =
  navigation, pas posture). Réserve non-refutante : « README §4.3 =
  0 ref » sur-affirmé (la section existe, mais ne documente pas les
  scripts — la réf PATTERNS reste STALE/trompeuse, à corriger).
- **S4-S1b-wire** : 4/4 CONFIRMED (zéro-wire par construction ; zéro
  dep ; PROMISE_RE scope + classe présent-vrai §P70:3932-3934 ; hook
  Checks 0/1/4/7/8/9 re-vérifiés fichier:ligne). Nuance : le grep
  `_VERSION\s*[:=]` est un instrument faible, conclusion indépendante.

## G8 traceability

- S1a (OSS prior-art) : upstream iroh-docs 0.101.0 lu au registry
  (littéral privé, non exporté — miroir + tripwire empirique justifiés).
- S1b (deps) : 0 nouvelle dep ; iroh-docs déjà présent
  (`store_migration.rs:33`).
- S2 (décisions historiques) : defer T20 INTANGIBLE réaffirmé (ledger
  Phase E §5.2) respecté ; disposition C-1/C-2 conforme « ne pas
  fabriquer » ; F-D5-01 ré-extrait verbatim (review F :136).
- S3 (threat model) : 0 changement de posture (WebPKI-only inchangé,
  THREAT_MODEL.md:1169-1174 concordant).
- S4 (wire) : 0 bump ; census 25 figé intact ; tripwire hors-wire par
  construction (nom de fichier local, jamais sérialisé réseau).
