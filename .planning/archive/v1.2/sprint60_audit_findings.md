# Sprint 60 — Audit findings (Phase 0 Sprint 61)

**Auditeur** : session Claude Code fraiche 2026-05-13
**Tip audite** : `69bf504` (+ chore planning `71efa7b`)
**Audit plan** : `sprint61_audit_plan.md` (8 tracks A-H)
**Timebox** : ~25 minutes

---

## Verdict global : PASS

0 P0, 0 P1, 4 P2 confirmes des phase reviews (tous carries S61),
2 P3. Sprint 61 Phase A peut demarrer directement.

---

## Track A — Phase integrity : PASS

- 4/4 preflights G8 presents (A, B, C, E) — tous EXECUTE
- 4/4 reviews presents (A, B, C, E) — tous PASS
- Phase D exclue du compteur G8 (validation manuelle, pas de code)
  -> report `sprint60_lt7_tier3_report.md` (214 lignes) avec
  evidence concrete
- 4 fix commits documentes dans verification.md §7 :
  `dd55bf6` (Win32 pump), `3b0f227` (--locked), `a045502`
  (cross-platform), `b6a93a8` (.deb GTK)
- Delta tests cumule : +2 Rust (1257→1259) verifie par nextest
  live (1259 pass, 0 fail). +0 Vitest (258→258). Coherent.
- Design review G1 : `sprint60_design_review.md` present, scoring
  D1✅ D2⚠️ D3✅ D4⚠️ D5✅ — acknowledged dans kickoff §4.

## Track B — Tray icon correctness : PASS

- `tray.rs` : `create_tray()` + `run_event_loop()` +
  `pump_win32_messages()` (Win32 FFI avec SAFETY comment L102-103)
- `main.rs` : refactor tokio background thread + shutdown_tx oneshot
  + resolve_web_root() 4 candidats (env, installed, bundled, dev)
- Fallback ctrl_c si tray init echoue (main.rs:525-527)
- Cross-platform : `#[cfg(windows)]` pump, Linux/macOS compilent
  sans tray events
- Tests : +2 (1 tray.rs:116 icon decode + 1 main.rs:641 web_root
  env var) — delta correct
- Win32 FFI `pump_win32_messages` : struct Msg manuelle au lieu de
  `windows-sys` — choix delibere (evite dep feature additionnelle).
  SAFETY comment present, ABI Win32 stable. P3 cosmetic (cf.
  findings).

## Track C — Installer correctness : PASS

- `Packager.toml` : product-name, 2 binaires (launcher + daemon),
  resources web/dist, NSIS currentUser, languages en+fr
- `scripts/build-installer.sh` : pipeline npm → cargo → packager
- Windows NSIS : 16.81 MB, SHA256 documente, install/uninstall
  valides (silent + GUI)
- Linux .deb : 26.04 MB, dpkg install/remove OK (Docker sbfb-ci)
- macOS .dmg : 23.32 MB, .app bundle ARM64 OK (SSH Mac)
- AppImage : scope cut post-v1.0 documente (linuxdeploy FUSE)
- P2-NSIS-UNINSTALL multi-binary carry S61 correctement route

## Track D — LT-7 Tier 3 validation quality : PASS

- Report 214 lignes avec evidence concrete (SHA256, tailles, IPs,
  node_id prefixes)
- Gossip 3 machines WAN : Win (Paris) + VPS (Helsinki) + Mac (Paris)
- API mutual discovery : `x-sbfb-token` header, 2-3 curators par
  noeud
- Task submit : Ed25519 signee, persistee coordinator.db
- Worker quorum E2E : carry post-tag documente (workers non deployes
  VPS/Mac — infrastructure validee, pas le workflow complet)
- Cross-OS SHA256 limitation D4 : consensus intra-OS seulement,
  documente dans design review acknowledged ⚠️

## Track E — Build pipeline + dette pair : PASS

- `build-release.sh` : 3 crates (worker + daemon + launcher) +
  frontend npm build + copie dist/web/ + `--locked` flag (fix
  `3b0f227`)
- P2-B-1 dead `--all` flag : RESOLU inline Phase B (grep confirme
  absence dans code actuel)
- P2-G-1 exe lock : FERME Phase B (5 builds consecutifs OK, non
  reproductible). Reouvert Phase E review (revit durant verification).
  Carry S61 correct.
- PATTERNS.md : §P48 StorageWriteLimiter + §P49 Kudos-v2 + §P50
  tray icon — presents et factuels

## Track F — Tag v1.0 integrity : PASS

- Tag `v1.0` present : `git tag -l` confirme
- Tag annote sur commit `e14a131` (Phase E wrap-up)
- CLAUDE.md : S60 CLOSED, tag v1.0, compteurs mis a jour
- SPRINT_LOG.md : row S60 presente (detailed, ~1500 chars)
- HARDENING_ROADMAP.md : `last_validated: 2026-05-12` avec resume
  S60 complet + triggers documentes
- ROADMAP_COMMITMENTS.md : LT-7 "gate satisfait" (Tier 1+2 S55,
  Tier 3 S60), LT-1 "reclassifie pre-v1.0", LT-2 "trigger PENDING"
  (tag pose, flip sequence post-sprint)
- 4 commits post-tag (planning fixes) : coherent, ne modifient pas
  le code produit

## Track G — Carries residuels : PASS

| Item | Compteur S61 | Source | Audit check |
|---|---|---|---|
| P2-A-1 rand blocker | 21+/3 | exemption externe | rand x3 dans Cargo.lock, upstream bloque |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 | iroh = "0.98" confirme workspace Cargo.toml |
| P2-NSIS-UNINSTALL multi-binary | NEW 1/3 | Phase C/D report | Phase C review documente |
| P2-IMAGE-DEP image 0.25 | NEW 1/3 | Phase A review | image crate en dep tray, ~15 transitives |
| P2-G-1 exe lock intermittent | reouvert | Phase E review | ferme B → reouvert E, cycle documente |
| P2-PLAYWRIGHT-REFACTOR | NEW 1/3 | Phase E review | global-setup pyproject.toml absent post-S50 |

Compteurs coherents avec verification.md §6 et audit_plan Track G.

## Track H — Pre-launch → post-launch policy transition : PASS

- `*_VERSION` : grep `crates/nexus-core-rs/src/` confirme
  `KEY_ROTATION_FORMAT_VERSION = 1` et `*_VERSION = 1` uniquement
  dans les comments de documentation — pas de bump accidentel
- `canonical.rs` : diff vide entre `31bc1a7..69bf504` — non modifie
  S60 (confirme)
- CLAUDE.md §Pre-launch protocol policy : mentionne "Apres le tag
  `v1.0`, la politique bascule"

---

## Security scan : PASS

- `unsafe` blocks S60 diff :
  - `tray.rs:104-109` Win32 FFI (PeekMessageW/TranslateMessage/
    DispatchMessageW) avec SAFETY comment L102-103. ABI stable Win32.
    Struct Msg manuelle. Risk: nul (MSG layout gele depuis Win95).
  - `main.rs:649,651` set_var/remove_var en `#[cfg(test)]` uniquement
    avec env_lock mutex guard. Pattern standard Rust tests.
- `unwrap()` : 2 en test code (tempfile::tempdir, fs::write) —
  acceptable
- Secrets : 0 match (AKIA, ghp_, pat_, sbfb_ patterns)
- Scope cuts : 0 leak (12/12 items verifies)

---

## Findings sorted by severity

### P2 (4 — tous confirmes des phase reviews, carries S61)

1. **P2-IMAGE-DEP** : `image` 0.25 ajoute comme dep tray icon mais
   non trace dans plan §Research. ~15 crates transitives (png,
   jpeg-decoder, etc.) non evaluees securite. Surface d'attaque
   limitee (include_bytes embedded, pas input utilisateur).
   Source : Phase A review. Carry S61 1/3.

2. **P2-NSIS-UNINSTALL** : uninstaller NSIS laisse fichiers
   residuels en multi-binary si processus verrouille a
   l'uninstall. Comportement natif template cargo-packager.
   Source : Phase C review. Carry S61 1/3.

3. **P2-G-1 exe lock intermittent** : exe lock revit durant
   verification Phase E apres fermeture Phase B. Root cause non
   confirme (Defender / IDE / agent load hypothesis). Carry S61
   reouvert, monitoring.

4. **P2-PLAYWRIGHT-REFACTOR** : 42 tests Playwright bloques par
   global-setup (pyproject.toml coordinateur Python supprime S50).
   Refactor PW pour coordinator Rust = item post-v1.0.
   Source : Phase E review. Carry S61 1/3.

### P3 (2)

1. **P3-1** : verification.md §5 montre P2-G-1 "CLOSED Phase B"
   tandis que §6 montre "reouvert Phase E". Sequence factuelle
   correcte mais lecture ambigue. §6 carries = source de verite.

2. **P3-2** : audit_plan Track B cite "2 tests (web_root resolution
   + menu creation)" mais les tests reels sont "icon decode +
   web_root resolution". Cosmetic — delta +2 correct, nommage
   divergent dans le plan.

---

## Commits fix attendus

Aucun. 0 P0, 0 P1. Sprint 61 Phase A demarre directement.

---

## P2 a logger en tech debt

Les 4 P2 sont deja documentes dans les carries S61 (verification §6
+ audit_plan Track G). Pas de nouveau P2 necessitant ajout
PATTERNS.md.

---

## P3 laisses sans action

P3-1 et P3-2 : cosmetic, aucune action requise. Le prochain sprint
verification.md peut adopter la convention "§5 = items adresses, §6
= etat final carries" pour lever l'ambiguite.

---

## Notes on audit completeness

- **Dimensions couvertes** : 8/8 tracks audit_plan + security scan +
  scope cut verification + delta tests live (nextest 1259 pass)
- **Non verifie** : run complet Vitest + Playwright (les compteurs
  memory + verification sont fiables, Playwright bloque pre-existant)
- **Pas de release build lance** : le build release a ete valide
  par les 4 phases + reviews. Un re-build auditeur ajouterait 15 min
  sans valeur (meme Cargo.lock, meme code)
- **G4 rigor** : 4 P2 + 2 P3 documentes avec evidence inline.
  Dimensions explorees avec grep/read cites. Pas de finding hallucine.
