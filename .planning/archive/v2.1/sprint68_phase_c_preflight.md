# Sprint 68 Phase C — preflight G8

Date : 2026-05-21 | HEAD : `6a21293` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, no band-aid, research before code, G8 obligatoire, OSS prior art avant chaque phase, planning adaptatif pas figé. Aucune tension avec le plan Phase C.
- feedback_context7_systematic.md : context7 obligatoire avant code/décision touchant lib/API/spec. Applicable pour dunce (nouvelle dep). context7 n'a pas d'entrée pour dunce (crate trop petit/niche), complété par WebSearch.
- vision_model.md : solo maintainer OpenBSD, pas de suggestion institutionnelle. N/A pour Phase C.
- nexus_grid_pivot.md : Factory = outil client externe (crate sbfb-factory), hors daemon (v4 D2). Phase C respecte ce contrat. Day 0 D4 gelée : Factory gates FG4-FG7 + P2-C-2 fix.

## Scans (all clean)

- S1a OSS prior art : 7 projets recherchés, APPROACH-ALIGNED — clean
- S1b deps : 6 libs scannées, 0 delta bloquant — clean
- S2 historiques : 3 fichiers cibles, 6 commits bodies lus — clean
- S3 threat model : FULL, 7 vectors analysés — clean
- S4 wire format : FULL / VERSION=1, Day 0 préservées — clean

---

## S1a — OSS prior art deep analysis

### Problème fonctionnel

"How do mature OSS projects implement path traversal prevention in
CLI tooling via canonicalization + prefix check, workspace-vs-template
diff comparison, and secret scanning integration?"

### Projets analysés en profondeur

#### [1] dunce — crates.io/crates/dunce
- Version actuelle : 1.0.5 (dernière release 2024-04-15)
- Downloads : 6M+ total, ~650K/mois
- Fonctionnalité : `canonicalize()` wrapper autour de `std::fs::canonicalize()` qui strip le préfixe UNC `\\?\` sur Windows quand le path n'a pas besoin de l'extended-length syntax.
- Pattern : drop-in replacement 1 ligne — `dunce::canonicalize(path)` au lieu de `std::fs::canonicalize(path)`.
- 0 CVE connu (RustSec advisory DB clean). 0 dépendance transitive.
- Verdict : APPROACH-ALIGNED. Le plan utilise dunce exactement comme conçu.

#### [2] soft-canonicalize — crates.io/crates/soft-canonicalize
- Version 0.1.x (pre-1.0), inspiré de Python 3.6+ `pathlib.Path.resolve(strict=False)`.
- Features additionnelles : ADS validation (NTFS Alternate Data Streams), TOCTOU race resistance, symlink cycle detection with bounded depth.
- Plus complet que dunce mais plus lourd et pre-1.0.
- Verdict : N/A. Le plan a déjà rejeté soft-canonicalize au kickoff D4 (pre-1.0, surdimensionné pour un outil CLI local). Confirme le rejet.

#### [3] path-security — crates.io/crates/path-security
- Bibliothèque de validation/sanitization complète : bloque `..`, rejette paths absolus, prévient null byte injection, bloque expansion variables.
- Pattern différent : lib de validation pure, pas de canonicalization. Complémentaire à dunce.
- Verdict : APPROACH-ALIGNED comme alternative. Le plan utilise canonicalize + prefix check manuellement, ce qui couvre le même cas sans ajouter une dep.

#### [4] cargo-generate (GitHub)
- Outil de scaffolding Rust qui génère des projets depuis des templates Git.
- Utilise `walkdir` + substitution de placeholders (similaire au `{{name}}` de sbfb-factory).
- Ne fait PAS de path traversal canonicalization dans sa validation — délègue au contexte d'exécution.
- Verdict : APPROACH-ALIGNED. sbfb-factory ajoute de la validation en plus (canonicalize + symlink + secrets), ce qui est plus rigoureux que cargo-generate.

#### [5] Nosey Parker — praetorian-inc/noseyparker (Rust, Apache 2.0)
- Secret scanner Rust : 150+ patterns regex, vitesse ~100s MB/s, supporte Git history scanning.
- Architecture : regex compilée, walkdir récursif, déduplication findings.
- Pattern similaire à sbfb-factory/secret_scanner.rs mais beaucoup plus large scope (scanner de repos entiers vs validation pre-publish).
- Verdict : APPROACH-ALIGNED. Le secret scanner de sbfb-factory (3 patterns) est délibérément minimal pour un outil CLI pre-publish. L'approche regex + walkdir est la même que Nosey Parker.

#### [6] Kingfisher — mongodb/kingfisher (Rust, Apache 2.0)
- MongoDB 2025 : fork Nosey Parker + Hyperscan SIMD + live API validation + 942 detection rules.
- Architecture : tree-sitter pour context-aware scanning, détecteurs custom, blast radius mapping.
- Verdict : N/A. Surdimensionné pour sbfb-factory (outil CLI local, 3 patterns suffisent pre-launch). Confirme APPROACH-NOVEL justifié : un scanner minimal embarqué dans le CLI est plus approprié qu'une dep externe lourde.

#### [7] dir-diff — crates.io/crates/dir-diff
- 6.7M downloads. Compare deux répertoires et retourne si différents.
- Pattern : walkdir + hash comparison, simple boolean diff.
- sbfb-factory diff.rs a besoin de lister ajoutés/modifiés/supprimés (pas juste un bool), donc impl custom justifiée.
- Verdict : APPROACH-ALIGNED. Le plan implémente un diff basique sans dep externe, ce qui est correct car dir-diff ne fournit pas le détail (quels fichiers ajoutés/modifiés/supprimés).

### Tableau comparatif

| Aspect | Plan Phase C | dunce | soft-canonicalize | cargo-generate | Nosey Parker |
|--------|-------------|-------|-------------------|----------------|--------------|
| Canonicalization | dunce + prefix check | drop-in replace | ADS + TOCTOU | non fourni | N/A |
| Symlink check | walkdir follow_links(false) | non fourni | cycle detection | non fourni | N/A |
| Secret scanning | 3 regex patterns | N/A | N/A | N/A | 150+ patterns |
| Diff workspace | custom walkdir (add/mod/del) | N/A | N/A | N/A | N/A |
| Template lockfile | BLAKE3 hash match | N/A | N/A | hash Git | N/A |

### Finding S1a

- Classification : **APPROACH-ALIGNED**
- Evidence : dunce est le standard de facto Rust pour canonicalize Windows-friendly (6M+ downloads, 0 CVE). Le pattern canonicalize + prefix check est documenté comme best practice (StackHawk, path-security crate docs). Le scanner minimal 3 patterns est cohérent avec l'usage CLI local (les projets OSS comme Nosey Parker/Kingfisher visent un scope beaucoup plus large). Le diff custom est justifié car dir-diff ne fournit pas le détail requis.
- Impact sur le plan : aucun

---

## S1b — Deps/libs versions + CVE

### Deps directes Phase C

| Dep | Version actuelle | Dernière release | CVE check | Status |
|-----|-----------------|------------------|-----------|--------|
| dunce | (NEW, sera ajoutée) | 1.0.5 (2024-04-15) | 0 CVE rustsec | clean |
| walkdir | 2.5.0 | 2.5.0 | 0 CVE rustsec | clean |
| blake3 | workspace | — | 0 CVE | clean |
| regex | workspace | — | 0 CVE | clean |
| serde_json | workspace | — | 0 CVE | clean |

### Deps indirectes pertinentes

| Dep | Version | CVE check | Status |
|-----|---------|-----------|--------|
| zip | 8.6.0 | CVE-2025-29787 affecte 1.3.0-2.2.x — 8.6.0 NON affecté | clean |
| reqwest | 0.12.28 / 0.13.3 | 0 CVE critique | clean |

### Specs touchées

Aucune spec externe (RFC, SLSA) touchée par Phase C. Les gates FG4-FG7 sont des validations locales.

### Finding S1b

- 0 finding bloquant. dunce 1.0.5 est stable, 0 transitive, 0 CVE. zip 8.6.0 post-CVE-2025-29787. Toutes les deps dans la plage verte.

---

## S2 — Decision chain reconstruction

### Fichiers scannés

- `template_engine.rs` : 3 commits bodies lus (49d6bcd0, a4cc0aef, 1d53f18c)
- `main.rs` : 3 commits bodies lus (idem — mêmes commits touchent les deux)
- `Cargo.toml` : 3 commits bodies lus (idem)

### Décisions historiques trouvées

#### Décision 1 : Path traversal via string check (P2-C-2)

- Sprint 67, sha `49d6bcd0` : validation `path.contains("..")` (string-level) implémentée dans `template_engine.rs:120`.
  Body extrait : "Codex GPT 5.5 : 9 CONFIRMES, 0 GAP, 1 PARTIEL [...] P2-C-2 : path traversal via path.contains("..") -- renforcer avec Path::components() pour le publish path S68"
- Sprint 67 audit `5449903` : carry P2-C-2 confirmé 1/3.
  Body extrait : "P2-C-2 path traversal Windows (1/3, confirme)"
- Sprint 68 kickoff `3ca563f` : D4 gèle "Factory gates FG4-FG7 + P2-C-2 fix". carry absorbe Phase C.
- Sprint 68 Phase B `1d53f18c` : aucune modification de template_engine.rs (preview/publish ajoutés sans toucher validate).
- Reverse-commit check : 3 commandes exécutées — 0 reversion trouvée. La décision de corriger via canonicalize est active.
- Status : **active** — correction planifiée Phase C exactement comme prévu.
- Impact phase : **aucun** — le plan Phase C implémente exactement cette correction.

#### Décision 2 : Factory = outil client externe, pas daemon

- Sprint 67, sha `49d6bcd0` : "Decision D2 v4 : Factory hors daemon, crate independant."
- Reverse-commit check : 0 reversion. Décision v4 D2 figée.
- Status : active.
- Impact phase : aucun — Phase C code dans crates/sbfb-factory/, hors daemon.

#### Décision 3 : dunce retenu pour canonicalize Windows

- Sprint 68 kickoff `3ca563f` : D4 retient dunce, rejette soft-canonicalize (pre-1.0) et std::fs::canonicalize seul (UNC paths Windows).
  Body extrait dans kickoff §4 D4 : "`dunce` crate : canonicalize sans UNC prefix. `soft-canonicalize` : plus complet mais pre-1.0. `dunce` est mature (6M downloads, dernière release 2024)."
- Reverse-commit check : 0 reversion.
- Status : active.
- Impact phase : aucun — le plan utilise dunce conformément à D4.

### Memory constraints

- feedback_approach.md : "Toujours aller au plus poussé" — dunce vs canonicalize seul est le choix le plus poussé praticable (soft-canonicalize est pre-1.0, path-security est une dep supplémentaire non justifiée). Pas de tension.
- feedback_context7_systematic.md : context7 query tentée pour dunce, pas d'entrée (crate trop niche). Complété par WebSearch crates.io/dunce + docs.rs/dunce. Obligation satisfaite.

---

## S3 — Threat model analysis

### Primitive analysée : Factory gates FG4-FG7

### Assets en jeu

- A1 Filesystem local (criticité : Medium) — les gates valident des paths et fichiers sur le FS de l'utilisateur
- A2 App archive intégrité (criticité : High) — le diff et le lockfile check garantissent que l'app est conforme au template
- A3 Secrets développeur (criticité : High) — le scanner empêche la publication accidentelle de credentials

### Threat actors

- TA1 Développeur inattentif (capacité : écriture FS, motivation : erreur humaine) — publie une app avec un secret embarqué ou un path traversal
- TA2 Repo squat attaquant (capacité : contrôle repo source, motivation : supply chain) — craft un workspace avec des symlinks ou des paths traversal pour escape le sandbox

### Attack vectors identifiés

1. **V1 Path traversal via input CLI** : l'utilisateur passe un path `../../etc/passwd` comme argument à `validate`/`diff`.
   - Couverture : FG5 (canonicalize + prefix check). Post Phase C, le path est canonicalisé et vérifié sous le workspace root.
   - Residual : L (canonicalize résout les `..` et les symlinks)

2. **V2 Symlink escape** : le workspace contient un symlink pointant hors du workspace.
   - Couverture : `walkdir::WalkDir::follow_links(false)` + détection symlink dans validate().
   - Residual : L (symlinks détectés et rejetés)

3. **V3 Secret leak via publish** : un fichier dans le workspace contient un secret (AWS key, GitHub token, PEM key).
   - Couverture : FG6 secret scanner (3 patterns regex). Les findings bloquent la gate.
   - Residual : M (patterns limités à 3 — patterns moins courants non détectés, ex: Slack webhook, npm token)

4. **V4 Template lockfile tampering** : un attaquant modifie le lockfile pour masquer une divergence template/workspace.
   - Couverture : FG6 lockfile hash check (factory.template.lock hash == factory.provenance.json template_hash). Le hash BLAKE3 est recalculé.
   - Residual : L (BLAKE3 collision infeasible)

5. **V5 DoS via large workspace** : un workspace avec des millions de fichiers bloque le diff/scan.
   - Couverture : aucune mitigation explicite.
   - Residual : L (outil CLI local, pas de surface réseau, l'utilisateur contrôle ses fichiers)

6. **V6 Race condition canonicalize (TOCTOU)** : entre canonicalize() et l'opération sur le fichier, un symlink est créé.
   - Couverture : outil CLI single-threaded, single-user. Le risque TOCTOU est théorique sur un outil CLI local.
   - Residual : L (pre-launch, CLI local, pas de concurrent access)

7. **V7 Windows NTFS ADS** : un fichier avec Alternate Data Stream pourrait contenir un secret non scanné.
   - Couverture : aucune (dunce ne protège pas contre ADS). soft-canonicalize le fait mais est rejeté (pre-1.0).
   - Residual : L (ADS est un vecteur exotique, sbfb-factory est un outil développeur local, pas un service exposé)

### Mitigations existantes (T0-T5)

- Les gates FG4-FG7 sont des validations **locales** dans l'outil CLI. Elles ne touchent pas la surface réseau (THREAT_MODEL §5.1-5.7).
- Le deploy vérifié (T5.3) est en aval : même si un secret passe le scanner local, le deploy-from-repo fait un clone fresh (pas d'historique) et le provenance.json signe le contenu.

### Gaps identifiés

- GAP1 V3 (secret scanner limité à 3 patterns) : severity Low, acceptable pre-launch. Le scanner est un filet de sécurité minimaliste, pas un remplacement pour des outils dédiés (Gitleaks/TruffleHog/Nosey Parker). Recommandation : enrichir les patterns S69+ (Slack webhook, npm token, generic high-entropy strings). Scope cut S68 §11 #11.
- GAP2 V7 (NTFS ADS non couvert) : severity Low. Documenter dans le code comme limitation connue.

### Regression check

- La primitive (canonicalize + prefix check) **améliore** la mitigation existante (string `contains("..")` → canonicalize réel). Aucune régression T0-T5.
- Aucun nouveau vecteur créé : FG4-FG7 sont des validations additionnelles, pas des surfaces exposées.

### Verdict S3

clean — 0 régression, 2 gaps Low acceptables pre-launch.

---

## S4 — Wire format deep audit

### canonical.rs lu intégralement : oui

296 lignes. 14 constantes DOMAIN_*_V1. Fonction `canonical_bytes()` unique.

### Structs vérifiées

Phase C ne touche **aucune struct dans canonical.rs**. Les fichiers cibles (gates.rs NEW, diff.rs NEW, template_engine.rs refactor, main.rs update, Cargo.toml dep) sont tous dans le crate sbfb-factory qui est un **outil client externe** sans interaction avec le wire format protocolaire.

Vérifications :
- [x] Aucune struct canonical touchée par Phase C
- [x] Aucune constante DOMAIN_*_V1 touchée
- [x] Aucune *_VERSION touchée
- [x] Aucun `serde_json::to_string` dans le périmètre wire (déjà vérifié S67 audit Track B : provenance.rs:25 est du hashing local)

### Day 0 check

- D1..D5 sprint courant : D4 (Factory gates FG4-FG7 + P2-C-2 fix) est exactement ce que Phase C implémente. Aucune D contredite.
- Décisions actées pivot.md : Factory = outil client externe (D2 v4) respecté. Gates FG4-FG7 (D8 v4) implémentées.

### Pre-launch policy

- *_VERSION = 1 : non touchées (sbfb-factory n'a pas de version protocolaire)
- Pas de tolerant decoder multi-version : N/A
- Pas de tests "legacy decode" zombie : N/A
- Feed extensible raw-op : N/A (Phase C ne touche pas le feed)

---

## Telemetrie preflight (agent deep)

- S1a : 7 projets OSS analysés / ~15 WebSearch queries / 0 context7 queries utiles (dunce trop niche) / finding : APPROACH-ALIGNED
- S1b : 6 libs scannées / 4 CVE searches / finding : clean
- S2 : 6 commits bodies lus / 1 archive file (sprint67 audit) / 5 memory files / finding : clean
- S3 : FULL / 7 vectors analysés / 2 gaps Low
- S4 : FULL / 0 structs vérifiées (aucune touchée) / canonical.rs lu intégralement : oui

## Action

Procéder code phase C.
