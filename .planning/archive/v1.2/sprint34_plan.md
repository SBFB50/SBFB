# Sprint 34 — Plan d'execution

**Kickoff** : `sprint34_kickoff.md`
**Tip entree** : `093541c`
**Phases** : A (dette MANDATORY) → B (Windows UX) → C (macOS/Linux) → D (wrap-up)

---

## Phase A — Dette MANDATORY (sprint pair §6.2.1 Regle 1)

**But** : fermer P2-A-1 rand triple + P2-REVIEW-C-2 COEP E2E +
evaluer frost-ed25519 3.0 trigger.

### A.1 — rand unification

1. `cargo tree -d | grep rand` pour confirmer l'etat actuel
   (rand 0.8 + 0.9 + 0.10, getrandom 0.2 + 0.3 + 0.4)
2. `cargo update --aggressive` pour tirer les derniers minors
3. `cargo tree -d | grep rand` apres update — verifier si
   unification automatique a reduit les versions
4. Si cohabitation persist (probable : frost-ed25519 2.x pin
   rand 0.8 via rand_core 0.6) : documenter la contrainte
   upstream, confirmer que les sous-arbres sont disjoints (pas
   de confusion CSPRNG), fermer l'item avec rationale "blocker
   externe frost-ed25519 upstream"
5. Si unification possible : modifier les pins Cargo.toml,
   verifier compilation + tests

### A.2 — COEP E2E test reel (D5)

1. Ajouter dep `zip = "2"` dans
   `crates/nexus-test-harness/Cargo.toml`
2. Creer `crates/nexus-test-harness/tests/blob_serve_coep.rs` :
   - Fixture : zip en memoire avec `index.html` minimal
     (DOCTYPE + `<html><body>ok</body></html>`, cf. G1 D5 caveat)
   - Spawn 1 daemon via DaemonHandle
   - Ecrire le zip dans le cache blob-serve du daemon
     (`{NEXUS_GRID_ROOT}/blob-serve-cache/{hash}`)
   - GET `http://127.0.0.1:{port}/blob-serve/{hash}/index.html`
     avec auth bearer token
   - Assert headers :
     - `cross-origin-opener-policy: same-origin`
     - `cross-origin-embedder-policy: require-corp`
     - `content-security-policy` contient `connect-src 'none'`
   - Assert body contient `ok`
3. `cargo nextest run -p nexus-test-harness -E "test(coep)"` vert

### A.3 — frost-ed25519 3.0 evaluation (D4, ajuste G1)

1. `cargo tree -i frost-ed25519` : identifier toutes les deps
   directes ET transitives qui tirent frost-ed25519
2. `grep -rn "frost" crates/nexus-shell-daemon-core/src/canary/`
   pour mesurer le surface code touche
3. Verifier si 3.0 change le format de signature (Ed25519 RFC
   8032 byte-identical vs Ed25519ctx/BIP32)
4. Decision :
   - Si delta < 100 LOC net ET signature identique ET 0 conflit
     transitive → upgrade inline dans cette phase
   - Sinon → carry S35 avec fiche `frost_3_0_impact.md` dans
     `.planning/active/`
5. Mettre a jour HARDENING_ROADMAP.md trigger frost-ed25519

### A.4 — Verification Phase A

- `cargo nextest run --workspace --locked` — 901+ verts
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo fmt --all --check`
- Python + Frontend suites (full fail-fast §7.4)

### Commit Phase A

```
feat(sprint34): Sprint 34 Phase A — dette MANDATORY rand + COEP E2E + frost eval
```

Body : delta tests, rand outcome, COEP test ajouté, frost decision.

---

## Phase B — Windows launcher UX

**But** : nexus-launcher.exe a une icone, pas de console en release,
logs dans fichier.

### B.1 — Asset icon

1. Creer `assets/nexus-launcher.ico` (256x256, 48x48, 32x32,
   16x16 multi-resolution). Source : logo SBFB existant ou
   placeholder geometrique (hexagone reseau P2P).
   Note : si pas d'outil icon sur la machine dev, utiliser un
   PNG 256x256 converti via script Python PIL ou asset statique.

### B.2 — build.rs winresource

1. Ajouter dans `crates/nexus-launcher/Cargo.toml` :
   ```toml
   [build-dependencies]
   winresource = "0.1"
   ```
2. Creer `crates/nexus-launcher/build.rs` :
   ```rust
   #[cfg(windows)]
   fn main() {
       let mut res = winresource::WindowsResource::new();
       res.set_icon("../../assets/nexus-launcher.ico");
       res.compile().unwrap();
   }

   #[cfg(not(windows))]
   fn main() {}
   ```
3. Verifier `cargo build -p nexus-launcher --release` cree un
   exe avec icon visible dans Explorer

### B.3 — windows_subsystem + file logging

1. En tete de `crates/nexus-launcher/src/main.rs` :
   ```rust
   #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
   ```
2. Setup logging fichier au tout debut de main() :
   - Creer `~/.sbfb/launcher.log` (trunc au demarrage)
   - Rediriger env_logger (ou tracing) vers ce fichier
   - Custom panic hook qui ecrit dans le meme fichier
3. Commentaire code (G1 D3 adjust) : "Launcher et daemon ont
   des fichiers log separes. Convergence = carry S35."

### B.4 — Tests

- `cargo build -p nexus-launcher --release` reussit
- `cargo nextest run -p nexus-launcher` — tests existants passent
- Verification manuelle : double-click exe → pas de console,
  daemon spawne, browser s'ouvre

### Commit Phase B

```
feat(sprint34): Sprint 34 Phase B — Windows launcher UX icon + subsystem + file logging
```

---

## Phase C — macOS .app + Linux .desktop

**But** : double-click macOS, menu apps Linux.

### C.1 — macOS .app bundle

1. Creer `configs/macos/Info.plist` :
   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
     "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
     <key>CFBundleExecutable</key>
     <string>nexus-launcher</string>
     <key>CFBundleIdentifier</key>
     <string>dev.sbfb.nexus-launcher</string>
     <key>CFBundleName</key>
     <string>SBFB Nexus Grid</string>
     <key>CFBundlePackageType</key>
     <string>APPL</string>
     <key>CFBundleVersion</key>
     <string>1.0.0</string>
     <key>CFBundleIconFile</key>
     <string>nexus-launcher</string>
     <key>LSMinimumSystemVersion</key>
     <string>11.0</string>
   </dict>
   </plist>
   ```
2. Creer `scripts/bundle-macos.sh` :
   - Arg : chemin vers le binaire compile
   - Cree `NexusGrid.app/Contents/{MacOS,Resources}/`
   - Copie binary + Info.plist + icon .icns
   - `chmod +x` le binary dans le bundle
3. Creer `assets/nexus-launcher.icns` (ou documenter la
   conversion depuis PNG via `iconutil` sur macOS)

### C.2 — Linux .desktop

1. Creer `configs/desktop/nexus-launcher.desktop` :
   ```ini
   [Desktop Entry]
   Type=Application
   Name=SBFB Nexus Grid
   Comment=Decentralized P2P compute network
   Exec=/opt/nexus-grid/target/release/nexus-launcher
   Icon=nexus-launcher
   Terminal=false
   Categories=Network;P2P;
   ```
2. Creer `assets/nexus-launcher.png` (256x256 icon)

### C.3 — install-node.sh integration

1. Apres le bloc build dans install-node.sh, ajouter :
   - Linux : copie .desktop vers `$HOME/.local/share/applications/`
     + icon vers `$HOME/.local/share/icons/hicolor/256x256/apps/`
     + `gtk-update-icon-cache` si disponible
   - macOS : appeler `scripts/bundle-macos.sh` + copier le .app
     dans `/Applications/` (ou `$HOME/Applications/`)

### C.4 — Tests

- `shellcheck scripts/bundle-macos.sh` (si shellcheck dispo)
- Assert structure .app valide (ls -R NexusGrid.app/)
- Assert .desktop parseable (`desktop-file-validate` si dispo,
  sinon grep sections obligatoires)

### Commit Phase C

```
feat(sprint34): Sprint 34 Phase C — macOS .app bundle + Linux .desktop integration
```

---

## Phase D — Wrap-up

### D.1 — verification.md

Fail-fast checklist 30+ rows :
- Rows 1-18 : standard (Rust compile/test/clippy/fmt/release,
  Python 3 suites, Frontend lint/tsc/vitest/build/size/PW/en-strings)
- Row 19 : FORMAT_VERSION v1
- Row 20 : HARDENING compteurs updated
- Row 21 : Planning docs complets
- Row 22 : rand unification outcome documented
- Row 23 : COEP E2E test pass (real zip)
- Row 24 : frost-ed25519 evaluation documented
- Row 25 : Windows exe has icon (resource check)
- Row 26 : Windows exe no console (subsystem check)
- Row 27 : Launcher log file created
- Row 28 : macOS .app bundle structure valid
- Row 29 : Linux .desktop file valid
- Row 30 : install-node.sh .desktop integration

### D.2 — Docs updates

- SPRINT_LOG.md : row S34
- CLAUDE.md : etat actuel S34 CLOSED
- HARDENING_ROADMAP.md : last_validated S34, frost trigger update
- sprint35_audit_plan.md : 6 tracks standard

### D.3 — Migration

`git mv .planning/active/sprint34_*.md .planning/archive/v1.2/`
+ `git mv .planning/active/sprint33_audit_findings.md .planning/archive/v1.2/`

### Commit Phase D

```
chore(sprint34): Phase D — wrap-up + verification + audit plan S35 + migration
```

---

## §5 Verification avant commit (rappel §7.4)

Avant chaque commit phase :
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
uv run ruff format --check packages/ && uv run ruff check packages/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q
cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json
npm run test:unit && npm run build && npm run size
npx playwright test && bash scripts/scan-en-strings.sh
```
