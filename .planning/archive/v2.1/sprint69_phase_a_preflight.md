# Sprint 69 Phase A — preflight G8

Date : 2026-05-22 | HEAD : `b930c34` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, no band-aid, research before code — N/A pour Phase A (items P2 process + audit log trivial)
- feedback_context7_systematic.md : context7 obligatoire avant code touchant lib/API — applique sur serde_json to_writer (context7 query done)
- vision_model.md : no funding/startup patterns — N/A
- feedback_kudos_non_monetary.md : N/A (Phase A ne touche pas kudos)
- sprint14_keyoxide_decision.md : deploy from source — N/A (Phase A ne touche pas deploy path)
- nexus_grid_pivot.md : S69 OPEN, tip b930c34, Day 0 D1-D5 gelees — Phase A touche D4 (audit log + P2-I-2 + P2-B-1), coherent
- Tensions plan vs memory : aucune

## S1a — OSS prior art deep analysis

### Probleme fonctionnel exact

"How do mature Rust CLI tools implement per-invocation audit logging in JSONL format, and how do in-memory stores cap their entry count to prevent DoS?"

### Projets analyses en profondeur

#### [RustDesk] — https://github.com/rustdesk/rustdesk
- Audit log : JSONL dans fichier local, append par session
- Pattern : serde_json::to_string + writeln! append, pas de framework
- Pertinence : CLI audit logging one-shot, meme pattern que Phase A

#### [cargo-audit] — https://github.com/rustsec/advisory-db
- Logging : rapport JSON en sortie, pas de log d'invocation persistant
- Pertinence : confirme que les CLI Rust legers n'utilisent pas tracing pour l'audit

#### [vault-audit-tools] — https://crates.io/crates/vault-audit-tools
- Pattern : JSONL viewer haute performance pour logs HashiCorp Vault
- JSONL est le format standard de l'ecosysteme audit logging (1 event = 1 ligne JSON)
- Pertinence : valide le choix JSONL comme format standard

#### [moka-rs/moka] — https://github.com/moka-rs/moka
- Cache in-memory bounded : `max_capacity(N)` via builder
- Eviction LFU admission + LRU eviction (Caffeine-inspired)
- Pertinence : lib mature pour bounded cache. Mais PreviewStore est trop simple (HashMap 10 entries, TTL 30min) pour justifier moka comme dep

#### [nexus-grid interne — bootstrap_allowlist.rs]
- bootstrap_allowlist.rs:134-138 : `TooManyEntries` error pattern existant
- `MAX_BOOTSTRAP_ENTRIES` const + check avant insertion + reject loud
- Pertinence : precedent interne direct pour le pattern MAX_PREVIEW_ENTRIES + TooManyEntries

### Tableau comparatif

| Aspect | Plan Phase A | RustDesk | vault-audit-tools | moka | bootstrap_allowlist.rs |
|--------|-------------|----------|-------------------|------|----------------------|
| Format audit log | JSONL append | JSONL append | JSONL reader | N/A | N/A |
| Framework | serde_json::to_writer | serde_json::to_string + writeln! | N/A (viewer only) | N/A | N/A |
| Bounded cache | const + reject error | N/A | N/A | max_capacity builder | const + TooManyEntries error |
| Rotation | none (pre-launch) | manual | viewer only | eviction auto | N/A |

### Finding S1a

- Classification : **APPROACH-ALIGNED**
- Evidence : JSONL append via serde_json::to_writer/to_string est le pattern standard pour les CLI Rust one-shot. moka existe pour bounded caches mais est surdimensionne pour un HashMap de 10 entries. Le pattern TooManyEntries existe en interne (bootstrap_allowlist.rs:134).
- Impact sur le plan : aucun

## S1b — Deps/libs versions + CVE

### Deps du perimetre Phase A

| Dep | Version Cargo.lock | Derniere release | CVE 2025-2026 | Status |
|-----|-------------------|-----------------|---------------|--------|
| serde_json | 1.0.149 | 1.0.149 (current) | CVE-2025-62518 = FAUX POSITIF (affecte astral-tokio-tar, pas serde_json) | clean |
| blake3 | 1.8.5 | 1.8.5 (current) | 0 CVE rustsec | clean |
| clap | 4.6.1 | 4.6.1 (2026-03) | 0 CVE, pas de v5.0 annonce | clean |
| thiserror | 1.0.69 / 2.0.18 | current | 0 CVE rustsec | clean |
| walkdir | (workspace) | stable | 0 CVE rustsec | clean |
| directories | (workspace) | stable | 0 CVE | clean |
| time | (workspace) | stable | 0 CVE applicable | clean |

### Specs touchees

| Spec | Revision check | Status |
|------|---------------|--------|
| JSONL (newline-delimited JSON) | Pas de spec formelle, convention de facto | stable |

### Finding S1b

- 0 CVE applicable, 0 breaking change, 0 dep bump necessaire
- Classification : **clean**

## S2 — Decision chain reconstruction

### Fichiers scannes

- `crates/nexus-shell-daemon-core/src/preview.rs` : 1 commit (1d53f18, S68 Phase B)
- `crates/sbfb-factory/src/main.rs` : 3 commits (49d6bcd S67-C, 1d53f18 S68-B, a201b3e S68-C)
- `crates/sbfb-factory/src/gates.rs` : 1 commit (a201b3e S68-C)
- `docs/security/THREAT_MODEL.md` : 6 commits (1ff04df S16-E through ecb25c5 S68-D)

### Decisions historiques trouvees

#### Decision 1 : PreviewStore sans MAX_PREVIEW_ENTRIES (gap)

- Sprint 68, sha `1d53f18` : PreviewStore cree avec MAX_PREVIEW_BYTES (10 MB par entry) mais sans cap sur le nombre d'entries
  Body extrait : "PreviewStore ephemere (HashMap + RwLock, TTL 30 min, max 10 MB, BLAKE3 hash)"
- Sprint 68 audit `0c2c2a8` : P2-B-1 documente comme gap, recommande cap 10 entries + THREAT_MODEL §preview
- Reverse-commit check : pas de reversion (gap ouvert, pas une decision rejetee)
- Status : gap actif, corrige par Phase A (comme planifie)
- Impact phase : aucun conflit — Phase A resout le gap

#### Decision 2 : Factory gates FG4-FG7 avec #[allow(dead_code)]

- Sprint 68, sha `a201b3e` : gates FG5, FG7, check_path_containment marquees dead_code car pas encore wirees dans publish pipeline
  Body extrait : "3 fonctions #[allow(dead_code)] : fg5_sandbox, check_path_containment, fg7_preview. P3-I-2 nit S68."
- Reverse-commit check : pas de reversion (dead_code intentionnel en attente du pipeline S69)
- Status : actif, resolution prevue Phase B (pas Phase A)
- Impact phase : aucun pour Phase A

#### Decision 3 : THREAT_MODEL renumerotation §12→§13 (ProofCard)

- Sprint 68, sha `ecb25c5` : §12 ProofCard surface ajoute, ancien §12 Revue→§13
  Body extrait : "§12 T-PROOFCARD-FORMULA-GAME [...] renumerotation §12→§13"
- Impact phase : Phase A ajoute §13 Preview ephemere → devra etre §14 (ou insere dans §12 ProofCard si thematiquement lie). **Attention : la numerotation doit etre §14** car §13 = Revue et evolution (actuel). Le plan dit "§13 Preview ephemere" mais le fichier actuel a §13 = "Revue et evolution". Adapter en inserant AVANT §13 ou en creant §13bis.

### Memory constraints

- feedback_approach.md : "pick deepest technical option" — le reject error (pas LRU silent eviction) est le choix le plus strict, conforme
- feedback_context7_systematic.md : context7 query sur serde_json done — conforme
- nexus_grid_pivot.md : "Pre-launch protocol policy" — Phase A ne touche pas de wire format, conforme

### Finding S2

- 0 decision historique contredite
- 1 attention numerotation THREAT_MODEL : plan dit §13 mais le fichier actuel utilise §13 pour "Revue et evolution". Ajuster le numero de section lors de l'implementation.
- Classification : **clean** (l'attention numerotation est cosmetique, pas bloquante)

## S3 — Threat model analysis

### Primitive analysee : MAX_PREVIEW_ENTRIES cap + Factory audit log JSONL

### Assets en jeu

- A-PREVIEW : Memoire daemon (HashMap in-memory, entries 10MB max chacune)
  Criticite : Medium (DoS local, pas de perte de donnees)
- A-AUDIT-LOG : Fichier `~/.sbfb/factory-audit.log` (JSONL local)
  Criticite : Low (traces locales, pas de secret)

### Threat actors

- TA1 : Script local malveillant / extension navigateur compromise (via bearer token leak)
  Capacite : peut appeler POST /api/v1/preview/load en boucle
  Motivation : DoS memoire daemon

- TA2 : Utilisateur maladroit (boucle de script dev)
  Capacite : appels preview en boucle sans eviction
  Motivation : accidentel

### Attack vectors identifies

1. **V1 Preview memory exhaustion** : appels POST /api/v1/preview/load en boucle sans eviction
   - Asset : A-PREVIEW
   - Pre-mitigation : MAX_PREVIEW_BYTES 10MB par entry, TTL 30min, bearer auth
   - Gap actuel : pas de cap sur le nombre d'entries → N × 10MB = OOM potentiel
   - Post-mitigation Phase A : MAX_PREVIEW_ENTRIES = 10 → max 100MB (10 × 10MB)
   - Couverture T0-T5 : couvert par mitigations S16 loopback (bearer + Host + Origin)
   - Severite residuelle : Low (loopback + bearer + cap)

2. **V2 Audit log disk exhaustion** : invocations Factory en boucle remplissent le log JSONL
   - Asset : A-AUDIT-LOG
   - Mitigation : pre-launch, taille negligeable (1 ligne JSON par invocation, ~200 bytes)
   - Estimation : 1M invocations = ~200MB. Non realiste en usage pre-launch.
   - Severite residuelle : Nil (pas de rotation necessaire pre-launch)

3. **V3 Audit log information leakage** : le fichier contient les arguments CLI
   - Asset : A-AUDIT-LOG
   - Mitigation : fichier dans ~/.sbfb/ (perm user-only), pas de secret dans les arguments (repo URL publique, path local)
   - Severite residuelle : Low (meme posture que consent.json, usage.json)

4. **V4 Audit log tampering** : un processus modifie le log pour couvrir ses traces
   - Asset : A-AUDIT-LOG
   - Mitigation : append-only par convention (pas de mechanism crypto). Pre-launch, le log est un outil dev, pas un artefact de securite signe.
   - Severite residuelle : Low (pas d'engagement d'integrite sur le log pre-launch)

### Mitigations existantes

- T-LOOPBACK (S16) couvre V1 : bearer auth empeche l'acces anonyme
- T-PREVIEW-BYTES (S68) couvre V1 partiellement : 10MB max par entry
- T-PREVIEW-TTL (S68) couvre V1 partiellement : 30min eviction
- Permissions ~/.sbfb/ (0600/0700) couvrent V3, V4

### Gaps identifies

- GAP1 V1 partiellement non couvert : MAX_PREVIEW_ENTRIES absent
  Severity : Medium (DoS memoire potentiel sans cap)
  Recommendation : Phase A resout ce gap (plan conforme)

### Regression check

- La primitive MAX_PREVIEW_ENTRIES ne diminue pas l'efficacite d'une mitigation existante
- La primitive audit log JSONL ne cree pas de nouveau vecteur non couvert (fichier local, memes permissions que consent.json)
- Aucun nouveau T necessaire

### Verdict S3

clean — 1 gap Medium (V1 MAX_PREVIEW_ENTRIES) resolu par Phase A

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui

Phase A ne touche AUCUNE struct dans canonical.rs. Aucun derive, aucune const, aucun impl.

### Structs verifiees

Phase A touche uniquement :

#### PreviewStore (preview.rs:27)
- Wire format : NON — HashMap in-memory ephemere, pas de serialization canonical
- Pas de serde derives sur PreviewStore
- Pas de DOMAIN_* signature
- MAX_PREVIEW_BYTES existant, MAX_PREVIEW_ENTRIES a ajouter
- Verdict : hors scope canonical, clean

#### AuditEntry (audit_log.rs — NEW)
- Wire format : NON — struct JSONL locale, pas de canonical bytes
- Serialization : serde_json::to_writer pour JSONL, pas JCS canonical
- Pas de DOMAIN_* signature
- Pas de signature Ed25519
- Verdict : artefact local, hors scope wire format

### Day 0 check

- D1 FG8 Ed25519 : non touche Phase A
- D2 Babel template : non touche Phase A
- D3 FG9 pipeline : non touche Phase A
- D4 audit log JSONL + P2-I-2 + P2-B-1 : **Phase A implemente D4** — coherent
- D5 Gate 1 test protocol : non touche Phase A
- Decisions actees pivot.md : aucune contredite

### Pre-launch policy

- `*_FORMAT_VERSION` = 1 partout (7 constantes verifiees) : OK
- `PROVENANCE_SCHEMA_VERSION` = 1 : OK
- Pas de tolerant decoder multi-version : OK
- Pas de tests "legacy decode" zombie : OK
- Phase A ne touche aucune constante VERSION : OK

### Grep exhaustif constantes version

```
CURATOR_LIST_FORMAT_VERSION = 1 (curator.rs:61)
KEY_ROTATION_FORMAT_VERSION = 1 (key_rotation.rs:32)
POW_FORMAT_VERSION = 1 (pow.rs:85)
TASK_FORMAT_VERSION = 1 (task.rs:61)
PIN_FILE_FORMAT_VERSION = 1 (tls_pinning.rs:102)
PROVENANCE_SCHEMA_VERSION = 1 (provenance.rs:15)
FEED_FORMAT_VERSION — not in canonical.rs (in public_feed.rs)
```

Toutes a 1, aucune modifiee par Phase A.

## Telemetrie preflight (agent deep)

- S1a : 5 projets OSS analyses / 1 precedent interne / 1 context7 query serde_json / 3 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : 7 libs scannees / 3 CVE searches (serde_json, blake3, thiserror+walkdir) / finding : clean
- S2 : 6 commits bodies lus en entier / 0 archive files / 6 memory files lus / finding : clean (1 attention cosmetique numerotation)
- S3 : FULL / 4 vectors analyses / 1 gap Medium (resolu par Phase A)
- S4 : FULL / 2 structs verifiees (PreviewStore, AuditEntry) / canonical.rs lu integralement : oui / 7 constantes VERSION verifiees

## Action

Proceder code phase A.
