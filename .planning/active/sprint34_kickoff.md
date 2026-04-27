# Sprint 34 — Kickoff (UX launcher cross-platform + dette pair)

**Ecrit** : 2026-04-27 (session fraiche post-audit gate S33 `093541c`).
**Type** : **sprint pair dette** — phase dette obligatoire
(§6.2.1 Regle 1 : S34 pair). 2 MANDATORY 3/3 a resoudre +
frost-ed25519 3.0 trigger G2 + launcher UX feature.
**Tip master d'entree** : `093541c` (chore(planning): sprint 33
audit findings — verdict PASS, 0 P0/P1, 1 P2, 1 P3).
**Phase 0 audit Sprint 33** : **DEJA JOUE** — findings dans
`.planning/active/sprint33_audit_findings.md` (verdict **PASS**,
0 P0/P1, 1 P2 fixe inline, 1 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-27) : HARDENING_ROADMAP last_validated
  `2026-04-27` (S33 Phase D).

  Triggers verifies :
  - iroh > 0.98 : pas de release 0.99 (0.98.1 patch dans range)
  - arti-client > 0.41 : stable 0.41.0 (inchange)
  - **frost-ed25519 > 2.1 : 3.0.0 released 2026-04-23 — TRIGGER FIRED**
    Impact : CanarySigner FROST primitive (S20 E.2), DKG code
    (S30 Phase C). Evaluation delta API requise.
  - frost-ed25519 2.2.0 intermediaire existe (2025-08-27)
  - wasmtime 44.0.0 latest, LTS 36.x (informationnel)
  - openai-agents-python 0.14.6 (informationnel, pas de dep directe)

- **Launcher cross-platform research** (2026-04-27) :
  3 agents paralleles (deploy-readiness, packaging research,
  crates.io scan) :
  - **winresource 0.1.31** (fork maintenu de winres abandonne)
    pour icon/manifest Windows dans build.rs
  - `#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
    avec redirection logging fichier (stdout/stderr invalides
    sous subsystem=windows depuis Rust 1.56)
  - macOS `.app` bundle = structure dossier + Info.plist + script
    shell 10 lignes, pas de code signing pre-v1.0
  - Linux `.desktop` freedesktop + icon XDG, integreable dans
    install-node.sh existant
  - cargo-bundle / cargo-packager evalues et rejetes (cf. D1)

- **ROADMAP_COMMITMENTS check** (G7 Regle 3) :
  Tous les triggers (LT-1 Gini, LT-2 Radicle, LT-3 app ecosystem)
  requierent tag v1.0 → aucun declenche.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 33 CLOSED. 3 phases A-C livrees + Phase D wrap-up :
- Phase A : CORS opt-in `--cors-origin` daemon Rust + coordinator
  Python, LOC guard hook check 6, 9 stale iroh comments fixes
- Phase B : deploy infra systemd 3 templates + install-node.sh
  multi-OS Linux/macOS
- Phase C : nexus-test-harness crate DaemonCluster + 4 tests
  integration multi-daemon + smoke test script
- Phase D : wrap-up + verification + audit plan S34 + migration

Audit gate S33 : **PASS** (0 P0/P1, 1 P2 fixe, 1 P3).
P2 C.1 (CORS scheme tests) fixe inline (+3 tests Rust).
E2 tor-rtcompat RESOLU (0 code residuel).

### §1.2 Ancrage HARDENING_ROADMAP

last_validated : 2026-04-27 (S33 Phase D). **1 trigger ACTIF** :
frost-ed25519 3.0.0 (2026-04-23). Action : evaluer delta API
en Phase A dette, decider upgrade ou carry.

### §1.3 Compteurs tests entree (tip `093541c`)

| Suite | Count |
|---|---|
| Rust nextest | 901 |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~1904** |

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1.
Pas de tolerant decoder multi-version. Cf. CLAUDE.md.

---

## §2 Goal en une phrase

Le sprint rend le lanceur SBFB **double-clickable sur Windows,
macOS et Linux** (icone, pas de console, menu apps) tout en
fermant les 2 items dette MANDATORY 3/3 (rand unification +
COEP E2E reel) et en evaluant le trigger frost-ed25519 3.0.
**Critere SMART : 30+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 33

**DONE** — `093541c`. Verdict PASS (0 P0/P1, 1 P2 fixe, 1 P3).
Cf. `.planning/active/sprint33_audit_findings.md`.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Packaging manuel per-platform (pas cargo-bundle/packager)

**Retenu** : 3 scripts/configs legers per-platform :
- Windows : `build.rs` + winresource pour icon/manifest dans l'exe
- macOS : `scripts/bundle-macos.sh` (mkdir + cp + Info.plist)
- Linux : `configs/desktop/nexus-launcher.desktop` + bloc dans
  install-node.sh pour copier .desktop + icon XDG

**Rejete** :
- cargo-bundle 0.10.0 : scope etroit (principalement .app macOS),
  ne gere pas l'icon embedding Windows dans l'exe, necessite
  install global.
- cargo-packager 0.11.8 (CrabNebula/Tauri team) : "public preview",
  lourd (NSIS, WiX, DMG), overkill pour "double-clickable binary",
  instabilite API.
- Tauri : Day 0 figee — "launcher Rust minimal, pas Tauri".

**Implications code** : `crates/nexus-launcher/build.rs` (NEW),
`crates/nexus-launcher/Cargo.toml` (build-dep winresource),
`scripts/bundle-macos.sh` (NEW), `configs/desktop/` (NEW),
`scripts/install-node.sh` (extension).

### D2 — winresource 0.1.31 pour Windows

**Retenu** : crate `winresource` 0.1 — fork maintenu de winres,
meme API, activement mis a jour (dernier release 2026-03-16).
Embarque .ico + manifest version dans l'exe via build.rs.

**Rejete** :
- winres 0.1.12 : abandonne depuis 2021, casse sur Rust 1.61+.
- embed-resource 3.0.9 : API differente, moins d'exemples, pas
  de gain clair.

**Implications code** : `crates/nexus-launcher/Cargo.toml`
`[build-dependencies]`, `crates/nexus-launcher/build.rs`,
`assets/nexus-launcher.ico` (NEW).

### D3 — windows_subsystem conditionnel + logging fichier

**Retenu** :
`#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
en tete de main.rs. Sous subsystem=windows, stdout/stderr sont
invalides (EBADF depuis Rust 1.56) — tout output redirige vers
`~/.sbfb/launcher.log` (rotation ou trunc au demarrage).
Custom panic hook ecrit dans le meme fichier log.

**Rejete** :
- windows_subsystem inconditionnel : casse `cargo test` output
  + debug impossible.
- Garder la console : UX non-dev inacceptable.
- Message box Win32 pour panics : dep lourde (windows-sys),
  overkill.

**Implications code** : `crates/nexus-launcher/src/main.rs`
(attribut + logging setup + panic hook).

### D4 — frost-ed25519 3.0.0 : evaluation-only, pas upgrade

**Retenu** : Phase A evalue le delta API 2.1→3.0 (grep impact
CanarySigner, FrostCanarySigner, DKG ceremony.rs, dkg.rs).
Si delta < 100 LOC net et 0 breaking sur wire format
CanarySigned v1 : inclure dans dette Phase A. Sinon carry S35
avec fiche impact documentee.

**Rejete** :
- Upgrade immediat sans evaluation : risque regression wire
  format warrant canary (DOMAIN_WARRANT_CANARY_V1 doit rester
  Ed25519 RFC 8032 byte-identical).
- Ignorer le trigger : viole G2.
- Rester sur 2.1 indefiniment : 2.1 est 1+ an derriere, les
  fixes securite sont sur 3.x.

**Implications code** : potentiellement
`crates/nexus-shell-daemon-core/src/canary/dkg.rs`,
`ceremony.rs`, `Cargo.toml`. Decision finale en Phase A
apres scan.

### D5 — COEP E2E : test Rust harness avec zip reel

**Retenu** : test d'integration dans `nexus-test-harness` ou
`nexus-shell-daemon` qui :
1. Cree un zip minimal en memoire (crate `zip`) avec index.html
2. L'ecrit dans le blob-serve cache directory du daemon
3. Fait un GET `/blob-serve/{hash}/index.html`
4. Assert headers COEP/COOP/CORP/CSP

Approche Rust-side (pas Playwright) car le harness a deja les
daemons spawnes et l'auth token.

**Rejete** :
- Playwright real daemon : necessite publication blob via
  coordinator (trop de plomberie, coord n'est pas dans le
  harness).
- Mock-only (statu quo) : 3/3 MANDATORY, doit etre ferme.
- Publication via iroh-blobs API dans le test : trop couple
  au protocol layer.

**Implications code** :
`crates/nexus-test-harness/Cargo.toml` (dep zip),
`crates/nexus-test-harness/tests/blob_serve_coep.rs` (NEW).

---

**Acknowledged review findings (G1)** :

Scoring : D1 ⚠️, D2 ✅, D3 ⚠️, D4 ⚠️, D5 ✅.
Rigor signal G4 satisfait (3 ⚠️ sur 5, 0 ❌).

D1 ⚠️ : maintenance 3 build chains separees. Decision : accept —
re-evaluer cargo-packager en S35 si une plateforme ajoute de la
complexite (signing, notarization). Le cout actuel est ~30 lignes
de scripts, acceptable pour v1.0.

D3 ⚠️ : deux systemes de log (launcher.log vs daemon log).
Decision : adjust — le launcher ecrit dans `~/.sbfb/launcher.log`
avec rotation trunc au demarrage. Le panic hook ecrit dans le
meme fichier. Convergence des logs launcher+daemon = carry S35
(necessite design partage de log directory). Phase B documente
le choix dans un commentaire code.

D4 ⚠️ : critere evaluation sous-specifie + risque transitive.
Decision : adjust — Phase A fait un `cargo tree -i frost-ed25519`
complet (pas juste grep direct) pour detecter toute dep transitive
qui pourrait tirer 3.x. Le critere d'upgrade est : (a) delta
< 100 LOC net, (b) signature CanarySigned v1 reste Ed25519 RFC 8032
byte-identical (pas Ed25519ctx/BIP32), (c) 0 dep transitive en
conflit. Si (b) echoue → carry S35 obligatoire avec fiche impact.

D5 ✅ : caveat HTML fixture. Decision : le test utilise un
`index.html` minimal (DOCTYPE + `<html><body>ok</body></html>`,
pas de scripts) pour eviter que CSP rejection masque un bug COEP.

---

## §5 Plan Phase outline A..D

### Phase A — Dette MANDATORY (sprint pair §6.2.1 Regle 1)

**But** : fermer les 2 items 3/3 MANDATORY + evaluer frost trigger.
- P2-A-1 rand : `cargo update --aggressive` + audit tree, supprimer
  pins redondants si unification possible
- P2-REVIEW-C-2 COEP E2E : test Rust zip reel (cf. D5)
- frost-ed25519 3.0 evaluation : scan delta API, decision inline
  (upgrade si < 100 LOC, sinon carry S35)
- Commit : `feat(sprint34): Sprint 34 Phase A — dette MANDATORY
  rand + COEP E2E + frost eval`

### Phase B — Windows launcher UX

**But** : le .exe a une icone, pas de console, logs dans fichier.
- build.rs + winresource : icon .ico + manifest version
- main.rs : `cfg_attr(not(debug_assertions), windows_subsystem)`
- File logging : `~/.sbfb/launcher.log` setup au boot
- Panic hook : ecrit dans le log au lieu de stderr
- Tests : assert que le log file est cree, assert icon dans exe
  (resource check)
- Commit : `feat(sprint34): Sprint 34 Phase B — Windows launcher
  UX icon + subsystem + file logging`

### Phase C — macOS .app + Linux .desktop

**But** : double-click sur macOS et menu apps Linux.
- `scripts/bundle-macos.sh` : cree `NexusGrid.app/` bundle
  (Info.plist + binary copy + icon .icns)
- `configs/desktop/nexus-launcher.desktop` : fichier freedesktop
- `assets/nexus-launcher.png` : icon 256x256 pour Linux
- install-node.sh : bloc .desktop + icon XDG post-install
- Tests : assert .app structure valide, assert .desktop
  valide (desktop-file-validate si dispo)
- Commit : `feat(sprint34): Sprint 34 Phase C — macOS .app
  bundle + Linux .desktop integration`

### Phase D — Wrap-up

- verification.md fail-fast 30+ rows
- sprint35_audit_plan.md
- SPRINT_LOG.md row S34
- CLAUDE.md etat actuel
- HARDENING_ROADMAP.md last_validated S34 + frost trigger
- Migration active/ → archive/v1.2/
- Commit : `chore(sprint34): Phase D — wrap-up + verification
  + audit plan S35 + migration`

---

## §6 Items carry/dette

### Resolus

- [x] P2-B-1 tor-rtcompat 3/3 : **FERME** par audit S33
  (0 code residuel, Phase 1 Tor design correct, tor-rtcompat
  sera ajoute si/quand Phase 2 Tor procede)

### MANDATORY — integres dans plan Phase A

- [x] P2-A-1 rand triple 3/3 : Phase A (unification rand workspace)
- [x] P2-REVIEW-C-2 COEP E2E 3/3 : Phase A (test Rust zip reel)

### Carries confirmes S34 (non-MANDATORY)

- [carry] P2-B-1-S33 shellcheck CI 2/3 : report S35 (pas de CI
  Linux encore, pre-requis CI pipeline). Justification : depend
  du setup CI qui n'est pas dans le scope S34.
- [carry] P2-B-2-S33 REPO_URL 2/3 : report S35 (placeholder OK
  pre-v1.0, URL reelle = decision release). Justification :
  blocker externe (pas de repo public encore).
- [carry] P2-C-1-S33 cross-daemon E2E 2/3 : report S35 (tests
  HTTP-level suffisants, full iroh-blobs cross-fetch = integration
  lourde). Justification : dependance sequentielle interne
  (harness blob publication).

### P3 a 3/3 — evalues

- P3 grammar executor 3/3 : **DEFER** — advisory P3, llguidance
  wiring dans l'executor est post-feature (executor task_runner
  est encore stub-to-Ollama, pas de grammaire a appliquer avant
  que le pipeline inference soit complet). Pas de suppression,
  reste carry.
- P3 watermark executor 3/3 : **DEFER** — advisory P3, SynthID
  wiring depend du pipeline inference complet (meme raison que
  grammar). Pas de suppression, reste carry.

### Long-terme inchanges

- T-NN+2 iframe Rust-wasm (PATTERNS §P34)
- LT-2 Radicle sortie cap G7 (trigger tag v1.0)
- LT-3/LT-4 hors-sprint (post-v1.0)
- LT-5 redundancy persistence (reclassifie S26)

---

## §7 Scope cuts

1. **VPS deployment effectif** — S35+ (harness multi-daemon
   suffit pour tester, pas de VPS encore)
2. **Code signing macOS** — post-v1.0 (Apple Developer ID
   $99/year, right-click bypass OK pre-v1.0)
3. **MSI/NSIS installer Windows** — post-v1.0 (exe standalone
   suffit, pas de distribution store)
4. **.deb/.rpm packages Linux** — post-v1.0 (manual install OK)
5. **Auto-update mechanism** — post-v1.0 (pas de release channel)
6. **Tray icon / notification area** — post-v1.0 (necessite dep
   lourde type tray-icon, hors scope minimal)
7. **frost-ed25519 3.0 upgrade** — S35 si delta > 100 LOC
   (evaluation Phase A)
8. **CI pipeline Linux** — S35+ (bloque shellcheck, Docker tests)
9. **stop/status CLI** — S35+ (inchange depuis S33)
10. **Cross-node task Ollama reel** — S35+ (stub)
11. **Docker daemon/worker** — S35+ (systemd suffit)
12. **P3 grammar/watermark wiring** — post-pipeline inference

---

## §8 Tracabilite scope (S33 → S34)

| Item S33 NOT | Ou dans S34 |
|---|---|
| VPS deployment | §7.1 scope cut S35+ |
| Mobile browser | §7 implicite (hors-scope depuis S33) |
| iroh relay over Tor | §7 implicite |
| Nym mixnet | §7 implicite |
| TEE H100 attestation | §7 implicite |
| DKG distribue FROST | §7 implicite |
| CI multi-node VPS | §7.8 scope cut S35+ |
| Docker daemon/worker | §7.11 scope cut S35+ |
| stop/status CLI | §7.9 scope cut S35+ |
| Build CI merge | §7.8 scope cut S35+ |
| Cross-node task Ollama reel | §7.10 scope cut S35+ |
| Output filter client-side | §7 implicite |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | winresource ne compile pas sur CI Linux/macOS | Medium | Low | cfg(windows) dans build.rs, 0 impact autres plateformes |
| R2 | rand unification impossible (frost-ed25519 2.x pin rand 0.8) | Medium | Medium | Accepter cohabitation si upstream bloque, documenter |
| R3 | COEP E2E test flaky (zip en memoire, path deps) | Low | Medium | Test deterministe, hash fixe, cleanup strict |
| R4 | frost-ed25519 3.0 casse wire format CanarySigned | Low | High | Evaluation Phase A, abort si signature change |
| R5 | .app bundle Gatekeeper bloque sans signing | Certain | Low | Documente (right-click bypass), code signing post-v1.0 |
| R6 | windows_subsystem=windows masque les panics | Medium | Medium | Panic hook fichier log + message box fallback optionnel |
| R7 | Icon .ico/.icns creation tooling absent | Low | Low | ImageMagick convert ou online tool pre-build, asset static |

---

## §10 Audit gate pattern — rappel

Phase 0 jouee (§3). Phase D produira `sprint35_audit_plan.md`.
Cf. §3 `docs/claude/README.md`.

---

## §11 Checkpoint de validation

5 questions avant d'attaquer le plan detaille :

1. **D1** : packaging manuel per-platform OK ? (vs cargo-packager)
2. **D2** : winresource 0.1.31 OK ? (vs embed-resource)
3. **D3** : logging fichier suffit ? (vs message box Win32 pour panics)
4. **D4** : frost-ed25519 evaluation-only OK ? (vs upgrade force)
5. **D5** : test Rust harness pour COEP E2E OK ? (vs Playwright real daemon)
