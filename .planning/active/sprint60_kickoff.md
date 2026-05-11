# Sprint 60 — Kickoff (installer Windows + tray icon + frontend bundling → tag v1.0)

**Ecrit** : 2026-05-11 (post-audit gate S59 PASS `31bc1a7`).
**Type** : **sprint pair** — phase dette obligatoire (§6.2.1 Regle 1).
**Tip master d'entree** : `31bc1a7` (migration S59 → archive/v1.2/).
**Phase 0 audit Sprint 59** : **DEJA JOUE** — `31bc1a7` PASS
(0 P0, 0 P1, 1 P2, 5 P3). Aucun fix bloquant requis.

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-11 (0j). 5 fichiers
  security avec triggers_revalidate. 1 trigger a evaluer : "iroh
  release > 0.98". **Resultat** : `cargo search iroh` retourne
  `iroh = "1.0.0-rc.0"` (publie 2026-05-07 sur crates.io).
  iroh-docs = 0.99.0, iroh-gossip = 0.99.0, iroh-blobs = 0.101.0.
  **Trigger ACTIF** — iroh 1.0.0-rc.0 > 0.98.
  **Evaluation** : c'est un release candidate (pre-release), pas
  une version stable. Le workspace pin iroh 0.98 / iroh-docs 0.98 /
  iroh-gossip 0.98 / iroh-blobs 0.100. Upgrade de 0.98 vers 1.0-rc
  pendant le sprint tag v1.0 = risque eleve (breaking changes non
  documentes, RC non stabilise). 0 CVE iroh entre 0.98 et 1.0-rc.
  **Decision** : rester sur iroh 0.98 pour le tag v1.0. L'upgrade
  iroh 1.0 stable sera un sprint dedie post-v1.0. Le trigger est
  documente comme "evalue et defere" (pas "ignore"). Politique
  clarifiee : les triggers_revalidate INCLUENT les pre-releases
  (RC) — l'evaluation est obligatoire, l'action (upgrade) ne l'est
  pas si le risque est documente.

- **cargo-packager context7** : v0.11.8 (CrabNebula, Nov 2025).
  Supporte NSIS comme output format. Config TOML (`[nsis]` section)
  avec install-mode, languages, appdata-paths. Produit un .exe
  installer. Licence Apache-2.0 — compatible AGPL-3.0.

- **tray-icon context7** : v0.24.0 (Tauri team, crates.io latest).
  Safe API `TrayIconBuilder::new().with_icon().with_menu()
  .with_tooltip().build()`. Message pump interne Windows (pas
  besoin winit). Context menu via `muda` v0.19.1 (meme equipe).
  MIT/Apache-2.0 — compatible AGPL-3.0. `windows-sys` deja en
  transitive dep dans le workspace.

- **NSIS** : v3.12 (19 avril 2026). Licence zlib — compatible AGPL.
  CVE-2025-43715 patche en 3.11, 3.12 = current stable. Silent
  install `/S`, shortcuts, uninstaller natifs.

- **WiX Toolset** : v7.0.0 (avril 2026). cargo-wix v0.3.9 target
  WiX v3 (legacy). MSI format enterprise. Evalue comme fallback,
  pas retenu pour v1.0.

- **Inno Setup** : v6.6.0 stable (nov 2025), v7.0.0-preview.
  Pas de plugin Rust. Evalue et rejete (maintenance manuelle .iss).

- **Frontend serving** : daemon a deja `--web-root` CLI flag
  (env `SBFB_WEB_ROOT`) + `ServeDir` + SPA fallback (http.rs:404).
  Le frontend n'est PAS embed dans le binaire. L'installer bundlera
  `web/dist/` et le daemon lira depuis le disque.

- **ROADMAP_COMMITMENTS check (G7 Regle 3)** :
  - LT-1 Kudos-v2 : **CLOSED S59**. Condition satisfaite.
  - LT-2 Radicle : trigger = tag v1.0. Sera **reactive** quand
    S60 pose le tag. Mais l'activation est post-tag (flip sequence),
    pas dans le scope du sprint.
  - LT-3/4/5 : latent. 0 condition declenchee.
  - LT-6 : RESOLVED S32.
  - LT-7 : Tier 1+2 DONE (S55). **Tier 3 validation controlee
    = S60 scope** (Win+VPS+Mac, redundancy=3).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 59 CLOSED + audit PASS (`31bc1a7`). Produit **early adopter
ready** : deploy → browse → run fonctionne E2E, scoring equitable
(LT-1 ferme), launcher communique les erreurs, storage valide et
rate-limited. 2 apps SBFB fonctionnelles (Protocol Explorer + Ideas
Hub). CI operationnel (Woodpecker + GHA).

**Etat technique (tip `31bc1a7`)** :
- Workspace clean, edition 2024, Rust 1.94 CI / 1.95 local
- Launcher : spawn daemon + browser open + identity init/unlock +
  token rotation + NVIDIA check + MessageBox FFI erreurs fatales.
  `#![windows_subsystem = "windows"]` en release. Attend Ctrl+C
  (pas de tray). Pas d'installer.
- Frontend : React + Vite + Tailwind + shadcn/ui. Servi via
  `--web-root` / `SBFB_WEB_ROOT` + `ServeDir`. Pas embed dans
  le binaire. Pas distribue P2P. 6 pages (Browse, Curators,
  Network, OnboardingEmpty, ProjectDetail, Projects, Deploy).
- build-release.sh : copie binaires dans dist/ mais pas le launcher
  ni le frontend. Pas d'installer Windows.
- Assets : nexus-launcher.ico + .png (S34). Desktop file Linux +
  macOS .app bundle script.

**Carries entrants S60** :

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 20+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-G-1 exe lock release build | 1/3 | audit S59 |

### §1.2 Ancrage roadmap

S59 a ferme les 3 derniers gaps feature pre-v1.0 (LT-1 + deploy
E2E + storage carries). S60 ferme le gap "end user ready" :

Roadmap pre-v1.0 (mise a jour 2026-05-11) :
- **S56** : gossip resilience + bridge extensions ✓
- **S57** : Protocol Explorer + Ideas Hub MVPs ✓
- **S58** : AppStorage P2P replication ✓
- **S59** : LT-1 Kudos-v2 + verified deploy E2E + launcher
  readiness + storage carries ✓ (early adopter ready)
- **S60** : installer Windows + tray icon + frontend bundling +
  LT-7 Tier 3 → tag v1.0 ← **ici** (end user ready)

### §1.3 Compteurs tests entree (tip `31bc1a7`)

| Suite | Count |
|---|---|
| Rust nextest | 1257 |
| Rust doctests | 6 (1 ignored) |
| Vitest | 258 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 6/6 |
| **Total** | **~1521** |

**Post-S60 attendu** : ~1530+ (tray tests + installer smoke).

### §1.4 Pre-launch protocol policy (rappel)

S60 ne touche PAS les wire formats (TaskEntry, ProjectAnnouncement,
CuratorList, etc.). Les changements sont sur le launcher (tray icon)
et le tooling (installer). Aucun `*_FORMAT_VERSION` ne sera bumpe.
Le pre-launch protocol s'applique normalement.

---

## §2 Goal

Sprint 60 rend le produit **end user ready** : un utilisateur non-
technique telecharge un installer Windows, installe en 2 clics,
lance l'app depuis le Start Menu, voit un tray icon, et accede au
shell React dans le navigateur. LT-7 Tier 3 valide le build
self-hosted sur 3 machines reelles. Le sprint pose le tag v1.0.
**Critere SMART : 24+ rows fail-fast verts au verification.md,
mesure binaire au Phase E wrap-up. Installer produit et teste.
Tray icon fonctionnel. LT-7 Tier 3 quorum SHA256 consensus 2/3.**

---

## §3 Phase 0 — Audit gate S59

**DEJA JOUE** : commit `31bc1a7` PASS (0 P0, 0 P1, 1 P2, 5 P3).
Aucun fix requis. Audit findings dans
`.planning/archive/v1.2/sprint59_audit_findings.md`. 2 carries
exemption + 1 carry P2 dev-env pour S60 (cf. §1.1).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Installer Windows : cargo-packager + NSIS

**Retenu** : utiliser cargo-packager v0.11.8 (CrabNebula) avec
output NSIS pour produire un .exe installer Windows.

L'installer bundle :
- `nexus-launcher.exe` (crate nexus-launcher, release build)
- `nexus-shell-daemon.exe` (crate nexus-shell-daemon, release build)
- `web/dist/` (React build du shell — servi par le daemon via
  `--web-root`)
- `assets/nexus-launcher.ico` (icone Start Menu + tray)
- Shortcut Start Menu → `nexus-launcher.exe`
- Uninstaller enregistre dans Add/Remove Programs
- Optionnel : auto-start via registry `HKCU\Software\Microsoft\
  Windows\CurrentVersion\Run` (checkbox dans l'installer)

**Configuration** : `Packager.toml` a la racine (format standard
cargo-packager). Sections `[package]` (binaries, resources, icon)
et `[nsis]` (install-mode, languages, app-data-paths).

**Build pipeline** :
```bash
npm --prefix web run build
cargo build --release -p nexus-launcher -p nexus-shell-daemon
cargo packager --release
```

**Rejete** :
- WiX Toolset (cargo-wix v0.3.9) : produit .msi enterprise-grade,
  cargo-wix target WiX v3 legacy (v7 sorti avril 2026, gap
  maintenance). MSI utile pour deploiement GPO mais overkill pour
  un produit P2P. Le daemon n'est pas un Windows Service —
  `ServiceInstall` MSI inutile. Defer post-v1.0 si demande
  enterprise.
- Inno Setup v6.6.0/7.0-preview : pas de plugin Rust. Maintenance
  manuelle fichier .iss separe. Effort superieur sans avantage.
- Raw NSIS script (.nsi) : verbose (centaines de lignes), pas de
  tooling Rust. cargo-packager abstrait la complexite.

**Implications code** : `Packager.toml` (NEW), `scripts/
build-installer.sh` (NEW), modifications build-release.sh.

### D5 — Scope change explicite : frontend bundling remplace frontend P2P distribution

**Retenu** : CLAUDE.md §Etat actuel et les roadmaps precedentes
mentionnent "frontend P2P distribution" comme objectif S60. Apres
evaluation (cf. §Sources pre-gel, agent research frontend P2P), le
frontend sera **bundle dans l'installer** et servi depuis le disque
via `--web-root`. La "P2P distribution" (mise a jour du frontend
via iroh-blobs) est **scope cut post-v1.0** (§7.1).

**Rationale** : pour le premier install v1.0, l'utilisateur
telecharge l'installer qui inclut tout. Il n'y a pas d'update a
distribuer en P2P puisque c'est la premiere version. La P2P
distribution devient utile uniquement pour les **updates post-
v1.0** — c'est un mecanisme d'auto-update, pas un prerequis v1.0.
Le blob-serve existant sert les **apps** (contenu tiers sandboxe),
pas le **shell** (qui a besoin d'appeler l'API daemon sans CSP
restrictif). Un chemin de serving dedie serait necessaire.

**Rejete** :
- Garder "frontend P2P" dans le scope S60 : ajoute 200+ LOC pour
  un mecanisme d'update inutile au premier lancement. Risque
  d'allonger le sprint au-dela du budget.
- Ne pas documenter le changement : cree un drift silencieux entre
  CLAUDE.md et le kickoff (violation prompt universel §gate).

**Action** : CLAUDE.md sera mis a jour en Phase E wrap-up pour
refleter "frontend bundling" au lieu de "frontend P2P distribution"
dans la section roadmap S60.

### D2 — Tray icon : crate tray-icon (Tauri team)

**Retenu** : ajouter `tray-icon` v0.24 + `muda` v0.19 comme deps
du launcher. Le launcher passe de "spawn → browser → Ctrl+C → exit"
a "spawn → browser → tray icon → message loop → menu Quit → exit".

**Fonctionnalites** :
- Icone dans la zone de notification Windows (notification area)
- Tooltip : "SBFB Nexus Grid — Connecte" / "Deconnecte"
- Context menu clic droit :
  - "Ouvrir le navigateur" → `open::that(url)`
  - "Quitter" → shutdown daemon + exit
- Double-clic → ouvrir le navigateur
- Icone dynamique (vert = connecte, gris = deconnecte) — stretch
  goal, pas bloquant v1.0

**Integration** : remplacer `tokio::signal::ctrl_c().await`
(main.rs:548) par une boucle `MenuEvent::receiver().try_recv()` +
`TrayIconEvent::receiver().try_recv()`. Le tray icon doit etre cree
sur le main thread (exigence Win32). Le runtime tokio tourne en
background thread via `tokio::runtime::Builder::new_multi_thread()`.

**Cross-platform** : `tray-icon` supporte macOS + Linux/GTK mais
S60 cible Windows primary. macOS/Linux → scope cut post-v1.0
(structure en place pour extension future).

**Rejete** :
- systray / systray2 (Actyx) : abandonne depuis 2021-2022. Utilise
  `winapi 0.3` legacy (pas `windows-sys`). Pas de tooltip/icon
  dynamiques.
- Raw Win32 `Shell_NotifyIconW` FFI : ~300 LOC unsafe. Le launcher
  a le precedent MessageBoxW mais un tray icon complet (hidden
  window + message pump + menu) est trop complexe pour du raw FFI.
  Risque UB sur struct layout.
- `windows-sys` direct avec features tray : ~150-200 LOC unsafe.
  Meilleur que raw FFI mais toujours du boilerplate message loop.
  tray-icon encapsule exactement ca en safe API.

**Implications code** : `crates/nexus-launcher/Cargo.toml` (deps
tray-icon + muda), `crates/nexus-launcher/src/main.rs` (refactor
main loop), `crates/nexus-launcher/src/tray.rs` (NEW module tray).

### D3 — Frontend bundling : installer + web_root disk

**Retenu** : l'installer (D1) inclut `web/dist/` comme ressource.
Le daemon le sert via `--web-root` pointant sur le repertoire
d'installation. Le launcher passe `--web-root` au daemon au spawn.

**Mecanisme existant** : le daemon a deja `--web-root` CLI flag
(env `SBFB_WEB_ROOT`) et `ServeDir` + SPA fallback (http.rs:404).
Zero code backend a ecrire — juste le wiring launcher → daemon.

**Wiring** : le launcher connait le repertoire d'installation
(via `std::env::current_exe().parent()`). Il passe
`--web-root <install_dir>/web` comme argument au spawn du daemon.
Fallback : `SBFB_WEB_ROOT` env var.

**Rejete** :
- `rust-embed` (embed frontend dans le binaire daemon) : augmente
  la taille du binaire daemon de ~5-10 MB (React build). Le
  frontend est mis a jour independamment du daemon. Embed force
  un rebuild daemon pour chaque change CSS/UI. Separation binaire
  + fichiers = plus flexible.
- Frontend P2P distribution (fetch blob iroh au premier lancement) :
  conceptuellement attractif pour l'update P2P mais ajoute un cold
  start de plusieurs secondes au premier lancement (fetch blob).
  L'installer inclut deja le frontend — pas de UX degradee. P2P
  distribution utile pour les **updates** (pas le premier install).
  Defer post-v1.0 avec un mecanisme auto-update.
- Dev server Vite en production : non-sense, jamais considere.

**Implications code** : `crates/nexus-launcher/src/main.rs`
(passer `--web-root` au daemon spawn).

### D4 — LT-7 Tier 3 validation controlee

**Retenu** : executer un build task reel sur 3 machines (Win dev +
VPS Helsinki + Mac) avec `redundancy_factor=3`. Le validator doit
passer AwaitingQuorum → Completed sur consensus SHA256 2/3.

**Procedure** :
1. Sur chaque machine : `nexus-shell-daemon start`
2. Machine A soumet un build task (target = un petit crate Rust)
3. Les 3 machines executent le build (build_executor.rs)
4. Les 3 soumettent leur SHA256
5. Le validator verifie le quorum (au moins 2/3 identiques)
6. Le task passe AwaitingQuorum → Completed

**Pre-requis** : les 3 machines doivent etre sur le meme reseau
gossip (decouverte pkarr + relay). WAN cross-machine valide depuis
S53 (Win↔Mac LAN + dev↔VPS Helsinki WAN).

**Rejete** :
- Validation sur une seule machine (localhost) : ne prouve pas le
  quorum cross-machine.
- Redundancy=5 ou plus : overkill avec 3 machines controlees.
  redundancy=3 est le minimum pour un quorum 2/3.
- Build task sur un projet complexe (le workspace SBFB lui-meme) :
  le build dure 15-30min+ sur des machines variees. Un petit crate
  (< 1min build) suffit pour prouver le chemin E2E.

**Implications code** : pas de code a ecrire — infrastructure
existante (build_executor.rs + quorum validator). Validation
manuelle + script de test.

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ⚠️, D3 ✅, D4 ⚠️, D5 ✅.
Rigor signal G4 satisfait (2 ⚠️ sur 5, 0 ❌).
G1 Verdict : **PASS** (cf. sprint60_design_review.md).

**D2 ⚠️ (tray-icon message pump standalone sans winit)** : le
reviewer note que la documentation ne detaille pas le mode message
pump Windows standalone (sans winit). Decision : le README tray-icon
montre un exemple "no winit" avec `TrayIconEvent::receiver()` +
`MenuEvent::receiver()` en boucle polling. Le crate cree son propre
hidden HWND en interne sur Windows. Si le pattern echoue au
runtime, fallback vers `windows-sys` direct (~150 LOC) — R2 dans
le risk register couvre ce scenario.

**D4 ⚠️ (Rust reproducible builds cross-OS)** : le reviewer
confirme que rust-lang/rust#129080 (MSVC random seed, chroot
indisponible Windows) n'est pas resolu en 2026. Decision :
LT-7 Tier 3 accepte un consensus intra-OS comme MVP (3 machines
Linux identiques OU 3 machines Windows identiques). Cross-OS
(Win vs Linux vs Mac) producira des SHA256 differents par design
(linker, libc differents). Le quorum est significatif uniquement
entre machines du meme triplet OS+toolchain. R3 dans le risk
register documente cette limitation.

---

## §5 Plan Phase outline A..E

### Phase A — Tray icon + launcher refactor

**But** : le launcher montre un tray icon au lieu de juste attendre
Ctrl+C.

- Ajouter deps `tray-icon` + `muda` au launcher
- Creer module `tray.rs` : init tray icon, menu (Ouvrir, Quitter),
  tooltip, event handler
- Refactorer `main()` : remplacer `ctrl_c().await` par message
  loop tray
- Passer `--web-root` au daemon spawn (D3 wiring)
- Tests : tray module unit tests (menu creation, event dispatch)
- Commit : `feat(sprint60): Sprint 60 Phase A — Tray icon +
  launcher message loop + web-root wiring`

### Phase B — Dette pair (sprint pair §6.2.1 Regle 1)

**But** : resoudre les carries P2 et dette technique.

- P2-G-1 exe lock : investiguer le processus verrouillant
  `target/release/nexus-shell-daemon.exe` (handle.exe Sysinternals)
- build-release.sh : ajouter le launcher + frontend build au
  pipeline
- Cleanup : sync PATTERNS.md avec les nouveaux patterns S59-S60
- Tests : delta minimal (investigations + docs)
- Commit : `feat(sprint60): Sprint 60 Phase B — Dette pair exe
  lock investigation + build pipeline update`

### Phase C — Installer Windows (cargo-packager + NSIS)

**But** : un utilisateur peut telecharger et installer SBFB.

- Creer `Packager.toml` : config cargo-packager avec output NSIS
- Creer `scripts/build-installer.sh` : pipeline complet
  (npm build → cargo build release → cargo packager)
- Tester l'installer : install → launch → tray visible → browser
  → naviguer Browse → uninstall propre
- Shortcut Start Menu, uninstaller, optionnel auto-start registry
- Commit : `feat(sprint60): Sprint 60 Phase C — Windows installer
  cargo-packager NSIS`

### Phase D — LT-7 Tier 3 validation controlee

**But** : prouver le build self-hosted sur 3 machines reelles.

- Preparer un petit crate de test (ou utiliser hello-world-app)
- Executer le build task sur Win + VPS + Mac avec redundancy=3
- Documenter les resultats (SHA256, consensus, timing)
- Script de validation reproductible
- Commit : `feat(sprint60): Sprint 60 Phase D — LT-7 Tier 3
  cross-platform build quorum validation`

### Phase E — Wrap-up + tag v1.0 + verification + audit plan S61

**But** : cloturer le sprint et poser le tag.

- CLAUDE.md : update S60 CLOSED, tag v1.0
- HARDENING_ROADMAP : update last_validated S60
- SPRINT_LOG : row S60
- verification.md : 24+ fail-fast rows
- sprint61_audit_plan.md : tracks pour S61
- Memory nexus_grid_pivot.md : update tip + tag v1.0
- **git tag v1.0** sur master
- ROADMAP_COMMITMENTS : LT-7 → RESOLVED. LT-1 → RESOLVED.
  LT-2 → trigger note (tag pose, flip sequence post-sprint).
- Commit : `chore(sprint60): Phase E — wrap-up + verification +
  audit plan S61 + tag v1.0`

---

## §6 Items carry/dette

### Carries confirmes S60

- [Phase B] **P2-G-1** exe lock release build 1/3 :
  **ADRESSE Phase B** → investigation root cause.
- [carry] **P2-A-1** rand blocker upstream 20+/3 : exemption
  externe. Justification : dep `rand 0.8` upstream bloque version
  compatible iroh 0.98. Aucun changement depuis S59.
- [carry] **P2-AUDIT-2** iroh transitives : herite pin 0.98.
  Justification : iroh 0.98 pinne (Day 0 #3), transitives
  non controlables. iroh 1.0.0-rc.0 publie (trigger G2 ACTIF,
  evalue et defere — cf. §Sources pre-gel).

### Carries residuels post-S60

| Item | Compteur S61 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 21+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |

---

## §7 Scope cuts

1. **Frontend P2P distribution** (update via iroh-blobs) — post-v1.0
2. **macOS tray icon** (tray-icon cross-platform) — post-v1.0
3. **Linux tray icon** (tray-icon GTK) — post-v1.0
4. **MSI installer** (WiX enterprise) — post-v1.0
5. **Windows Service registration** (daemon as service) — post-v1.0
6. **Auto-update mechanism** (P2P ou HTTP) — post-v1.0
7. **Tray icon dynamique** (vert/gris connecte/deconnecte) — stretch
   S60, scope cut post-v1.0 si non faisable dans le budget
8. **LT-7 Tier 3 diversite publique** — post-launch (nœuds tiers)
9. **LT-2 Radicle flip sequence** — post-tag (trigger actif mais
   execution post-sprint)
10. **DRF Couche B** (Kudos multi-ressource) — post-v1.0
11. **AppStorage Phase 2** (manifest per app) — post-v1.0
12. **Keyoxide identity verification** — post-v1.0

---

## §8 Tracabilite scope (S59 → S60)

| S59 scope cut | S60 disposition |
|---|---|
| NSIS/WiX installer | **Phase C** |
| Tray icon | **Phase A** |
| Frontend P2P distribution | Scope cut post-v1.0 (§7.1) |
| Protocol Explorer F3 (gossip stats) | Scope cut post-v1.0 |
| Protocol Explorer F4 (tutoriel) | Scope cut post-v1.0 |
| Ideas Hub F3 (lier repos Git) | Scope cut post-v1.0 |
| Ideas Hub F4-F5 (groupes) | Scope cut post-v1.0 |
| Kudos-weighted voting | Scope cut post-v1.0 |
| AppStorage Phase 2 (manifest) | Scope cut post-v1.0 |
| AppStorage Phase 3 (optimisations) | Scope cut post-v1.0 |
| Ticket Write rotation dynamique | Scope cut post-v1.0 |
| LT-7 Tier 3 validation controlee | **Phase D** |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | cargo-packager ne supporte pas le layout multi-binaire | Low | High | cargo-packager documente le support multi-binaires. Fallback : raw NSIS script .nsi. |
| R2 | tray-icon message loop bloque le runtime tokio | Medium | Medium | Creer le tray sur le main thread, tokio en background thread. Pattern documente par tray-icon. |
| R3 | LT-7 Tier 3 echoue (SHA256 diverge cross-platform) | Medium | Medium | Rust reproducible builds pas garanti cross-OS. Mitigation : tester d'abord Linux-only (VPS×3), puis cross-OS. Accepter un consensus intra-OS comme Tier 3 MVP. |
| R4 | Installer Windows bloque par Windows Defender SmartScreen | Medium | Low | Le binaire n'est pas signe. SmartScreen affiche un warning "publisher inconnu". Pre-v1.0 acceptable. Signature code (Authenticode) = post-v1.0 si budget certificat. |
| R5 | web/dist/ trop gros pour l'installer | Low | Low | React build ~5MB compresse. NSIS gere la compression nativement. |
| R6 | exe lock (P2-G-1) non root-cause meme apres investigation | Medium | Low | Si non reproductible, fermer comme flaky dev-env. Si reproductible, documenter le contournement (rename avant build). |

---

## §10 Audit gate pattern — rappel

Phase 0 S59 jouee (PASS `31bc1a7`). Phase E produira
sprint61_audit_plan.md pour la session fraiche S61.

---

## §11 Checkpoint de validation

1. **D1** : installer = cargo-packager + NSIS (.exe) ?
   → oui (meilleure integration Rust, .exe familier, TOML config)
2. **D2** : tray icon = crate tray-icon (Tauri) ?
   → oui (safe API, message pump interne, MIT, active maintenance)
3. **D3** : frontend bundling = installer + web_root disk ?
   → oui (mecanisme existant --web-root, zero code backend)
4. **D4** : LT-7 Tier 3 = 3 machines, redundancy=3, SHA256 ?
   → oui (infra existante build_executor + quorum validator)
