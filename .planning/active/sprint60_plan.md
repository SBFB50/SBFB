# Sprint 60 — Plan

**Ecrit** : 2026-05-11
**Kickoff** : sprint60_kickoff.md
**Theme** : installer Windows + tray icon + frontend bundling → tag v1.0

---

## §1 Phase outline

| Phase | Titre | But |
|-------|-------|-----|
| A | Tray icon + launcher refactor | Le launcher montre un tray icon avec menu |
| B | Dette pair + build pipeline | P2-G-1 exe lock + build-release update |
| C | Installer Windows | cargo-packager + NSIS produit un .exe installer |
| D | LT-7 Tier 3 validation | Build quorum cross-machine 3 machines |
| E | Wrap-up + tag v1.0 | Verification + audit plan S61 + git tag |

---

## §2 Phase A — Tray icon + launcher refactor

### But
Le launcher affiche un tray icon dans la zone de notification
Windows au lieu de simplement attendre Ctrl+C. L'utilisateur peut
ouvrir le navigateur et quitter via le context menu.

### Fichiers touches
| Fichier | Role |
|---------|------|
| `crates/nexus-launcher/Cargo.toml` | Ajouter deps tray-icon + muda |
| `crates/nexus-launcher/src/tray.rs` | NEW — module tray icon |
| `crates/nexus-launcher/src/main.rs` | Refactor main loop, passer --web-root |

### Detail technique

1. **Deps** : ajouter `tray-icon = "0.21"` et `muda = "0.17"`
   sous `[dependencies]`. Pas de cfg gate Windows-only — tray-icon
   est cross-platform (compile en no-op sur les OS non supportes).

2. **Module tray.rs** :
   - `pub fn create_tray(icon_bytes: &[u8]) -> TrayIcon` : charge
     l'icone depuis les bytes (include_bytes! sur assets/nexus-
     launcher.ico ou .png), cree le menu (Ouvrir, Quitter), cree
     le TrayIcon.
   - `pub fn run_event_loop(url: String, shutdown_tx: oneshot::Sender<()>)` :
     boucle polling `MenuEvent::receiver().try_recv()` +
     `TrayIconEvent::receiver().try_recv()`. Sur "Ouvrir" →
     `open::that(&url)`. Sur "Quitter" → `shutdown_tx.send(())`.
     Sur double-clic → `open::that(&url)`.

3. **Refactor main.rs** :
   - Apres spawn daemon + browser open, au lieu de
     `tokio::signal::ctrl_c().await`, appeler
     `tray::run_event_loop(url, shutdown_tx)`.
   - Le runtime tokio continue en arriere-plan (token rotation,
     NVIDIA check, auth server).
   - Le shutdown_rx dans le runtime tokio declenche le cleanup.
   - Passer `--web-root <install_dir>/web` au daemon spawn si le
     dossier existe a cote du launcher.

4. **Fallback non-Windows** : sur Linux/macOS, tray-icon compile
   mais necessite GTK/AppKit. Si l'init echoue, fallback vers
   `ctrl_c().await` (comportement actuel). Log warning.

### Critere d'acceptation
- Le launcher affiche un tray icon sur Windows
- Clic droit → menu "Ouvrir le navigateur" / "Quitter"
- "Ouvrir" ouvre le navigateur
- "Quitter" arrete le daemon et ferme le launcher
- Le daemon recoit `--web-root` si le dossier existe

### Delta tests attendu
- +2-3 Rust (tray module unit tests : menu creation, web_root
  resolution)

### Commit
```
feat(sprint60): Sprint 60 Phase A — Tray icon + launcher message loop + web-root wiring
```

---

## §3 Phase B — Dette pair + build pipeline

### But
Sprint pair § 6.2.1 Regle 1. Resoudre P2-G-1 (exe lock) et mettre
a jour le pipeline de build.

### Fichiers touches
| Fichier | Role |
|---------|------|
| `scripts/build-release.sh` | Ajouter launcher + frontend build |
| `docs/rust/PATTERNS.md` | Update patterns S60 |

### Detail technique

1. **P2-G-1 exe lock investigation** :
   - Sur Windows dev, lancer `cargo build -p nexus-shell-daemon
     --release` et observer si le lock se reproduit.
   - Si oui : identifier le processus avec
     `handle.exe nexus-shell-daemon.exe` (Sysinternals).
   - Documenter le root cause ou fermer comme non-reproductible.

2. **build-release.sh update** :
   - Ajouter `cargo build --release -p nexus-launcher` au pipeline.
   - Ajouter `npm --prefix web run build` pour le frontend.
   - Copier `nexus-launcher.exe` + `web/dist/` dans `dist/`.

3. **PATTERNS.md** : sync nouveaux patterns S59-S60 (StorageWrite
   Limiter pattern, tray icon pattern, cargo-packager pattern).

### Critere d'acceptation
- P2-G-1 documente (root cause ou non-reproductible)
- build-release.sh produit launcher + daemon + frontend dans dist/
- PATTERNS.md a jour

### Delta tests attendu
- +0 (investigation + docs + script)

### Commit
```
feat(sprint60): Sprint 60 Phase B — Dette pair exe lock + build pipeline + PATTERNS
```

---

## §4 Phase C — Installer Windows (cargo-packager + NSIS)

### But
Un utilisateur peut telecharger un .exe installer, l'executer, et
avoir SBFB installe sur sa machine.

### Fichiers touches
| Fichier | Role |
|---------|------|
| `Packager.toml` | NEW — config cargo-packager |
| `scripts/build-installer.sh` | NEW — pipeline complet |

### Detail technique

1. **Packager.toml** :
   ```toml
   [package]
   product-name = "SBFB Nexus Grid"
   version = "1.0.0"
   description = "Decentralized P2P compute network for apps"
   homepage = "https://sbfb.world"
   icons = ["assets/nexus-launcher.ico"]
   
   [[package.binaries]]
   path = "nexus-launcher"
   main = true
   
   [[package.binaries]]
   path = "nexus-shell-daemon"
   
   [package.resources]
   "web/dist" = "web"
   
   [nsis]
   install-mode = "currentUser"
   languages = ["English", "French"]
   ```

2. **scripts/build-installer.sh** :
   - `npm --prefix web run build`
   - `cargo build --release -p nexus-launcher -p nexus-shell-daemon`
   - `cargo packager --release`
   - Output : `dist/SBFB Nexus Grid_1.0.0_x64-setup.exe`

3. **Test manuel** : installer → launch → tray visible → browser →
   naviguer Browse → desinstaller via Add/Remove Programs.

### Critere d'acceptation
- L'installer .exe est genere sans erreur
- Installation dans Program Files (ou AppData current user)
- Shortcut Start Menu "SBFB Nexus Grid"
- Le launcher ouvre le navigateur + tray icon
- Desinstallation propre via Add/Remove Programs

### Delta tests attendu
- +0-1 Rust (smoke test installer existence optionnel)

### Commit
```
feat(sprint60): Sprint 60 Phase C — Windows installer cargo-packager NSIS
```

---

## §5 Phase D — LT-7 Tier 3 validation controlee

### But
Prouver que le build self-hosted fonctionne sur 3 machines reelles
avec quorum SHA256.

### Fichiers touches
| Fichier | Role |
|---------|------|
| `scripts/test-lt7-tier3.sh` | NEW — script validation |
| `.planning/active/sprint60_lt7_tier3_report.md` | NEW — resultats |

### Detail technique

1. **Pre-requis** : les 3 machines (Win dev + VPS Helsinki + Mac)
   doivent executer `nexus-shell-daemon start` et etre sur le meme
   reseau gossip.

2. **Scenario** :
   - Machine A soumet un build task (target = examples/hello-world-app
     ou un petit crate Rust avec Cargo.toml minimal)
   - Les 3 machines executent le build via build_executor.rs
   - Chaque machine soumet son SHA256
   - Le validator verifie le quorum (consensus 2/3)
   - Le task passe AwaitingQuorum → Completed

3. **Report** : documenter les SHA256, timing, consensus dans
   sprint60_lt7_tier3_report.md.

4. **Limitation documentee (G1 D4 ⚠️)** : le consensus est
   significatif uniquement entre machines du meme triplet
   OS+toolchain (Rust reproducible builds cross-OS non garanti
   en 2026). MVP = 3 machines Linux (VPS × 3) OU 3 machines
   Windows. Cross-OS = stretch goal.

### Critere d'acceptation
- Au moins 1 build task complete avec consensus SHA256 2/3
- Resultats documentes dans le report
- Task status AwaitingQuorum → Completed observe

### Delta tests attendu
- +0 (validation manuelle, pas de code nouveau)

### Commit
```
feat(sprint60): Sprint 60 Phase D — LT-7 Tier 3 cross-platform build quorum validation
```

---

## §6 Phase E — Wrap-up + tag v1.0 + verification + audit plan S61

### But
Cloturer le sprint, poser le tag v1.0, produire les artefacts de
sortie.

### Fichiers touches
| Fichier | Role |
|---------|------|
| `CLAUDE.md` | Update S60 CLOSED, tag v1.0 |
| `docs/security/HARDENING_ROADMAP.md` | Update last_validated |
| `docs/claude/SPRINT_LOG.md` | Row S60 |
| `docs/release/ROADMAP_COMMITMENTS.md` | LT-7 RESOLVED |
| `.planning/active/sprint60_verification.md` | NEW |
| `.planning/active/sprint61_audit_plan.md` | NEW |

### Detail technique

1. **Verification** : 24+ fail-fast rows (cargo fmt, clippy,
   nextest, doctests, release build, npm lint, tsc, Vitest, npm
   build, size-limit, scan-en-strings, installer generation, tray
   fonctionnel, LT-7 Tier 3 resultat).

2. **Audit plan S61** : tracks pour auditer S60 (installer
   correctness, tray icon UX, build pipeline, LT-7 Tier 3
   resultats, tag v1.0 integrite).

3. **Tag v1.0** : `git tag -a v1.0 -m "v1.0 — end user ready"`.

4. **Memory** : update nexus_grid_pivot.md avec tag, compteurs,
   carries.

5. **ROADMAP_COMMITMENTS** :
   - LT-1 : deja RESOLVED (S59)
   - LT-7 : Tier 3 RESOLVED (S60). Note : Tier 3 diversite
     publique = post-launch.
   - LT-2 : trigger note "tag v1.0 pose, flip sequence post-sprint"

### Commit
```
chore(sprint60): Phase E — wrap-up + verification + audit plan S61 + tag v1.0
```

---

## §7 Research consulte

| Source | Date | Pertinence |
|--------|------|------------|
| cargo-packager crates.io | 2026-05-11 | D1 version + features |
| NSIS 3.11 release notes | 2026-04 | D1 securite CVE-2025-43715 |
| tray-icon crates.io + GitHub | 2026-05-11 | D2 version + API |
| muda crates.io | 2026-05-11 | D2 context menu companion |
| iroh crates.io | 2026-05-11 | G2 trigger check (0.98.2 latest) |
| Rust reproducible builds | 2026-04 | D4 blocker cross-OS |
| WiX Toolset v7 | 2026-04 | D1 rejete (cargo-wix gap) |
| Inno Setup 7.0-preview | 2025-11 | D1 rejete (pas de plugin Rust) |
| systray2 GitHub | 2021-2022 | D2 rejete (abandonne) |
