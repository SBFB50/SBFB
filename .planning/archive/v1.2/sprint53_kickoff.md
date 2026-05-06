# Sprint 53 — Kickoff (P2P smoke test multi-plateforme + VPS bootstrap)

**Ecrit** : 2026-05-03 (post-audit gate S52 PASS `b85a3a1`).
**Type** : **sprint impair** — pas de phase dette obligatoire.
P2-REVIEW-B-1-S51 unsafe set_var a 2/3 (approche 3/3 MANDATORY S54).
**Tip master d'entree** : `b85a3a1`.
**Phase 0 audit Sprint 52** : **DEJA JOUE** — `b85a3a1` PASS
(0 P0, 0 P1, 1 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-02 (1j). 5 fichiers
  security + PATTERNS avec triggers_revalidate. 0 trigger actif.
  Pas de pre-research.

- **iroh relay/discovery** (context7 `/websites/rs_iroh`) :
  iroh fournit 3 relay servers par defaut (NA `use1-1.relay.n0.iroh.iroh.link`,
  EU `euc1-1.relay.n0.iroh.iroh.link`, AP `aps1-1.relay.n0.iroh.iroh.link`).
  NAT traversal automatique via hole-punching + fallback relay.
  Port QUIC relay par defaut : 7842. Le VPS avec IP publique
  (`135.181.42.188`) sera joignable directement en QUIC sans relay.
  Aucun port specifique a ouvrir cote client (bind `0.0.0.0:0`).
  Cote VPS, UDP entrant necessaire pour QUIC direct.

- **ROADMAP_COMMITMENTS check** :
  - LT-1 Kudos-v2 : sprint dedie requis, pas S53.
  - LT-7 self-hosted build : S54-S55 (pre-v1.0). S53 VPS bootstrap
    est le prerequis infra pour LT-7.
  - LT-2..LT-5 latents (tag v1.0 non pose). LT-6 RESOLVED S32.
  - 0 condition declenchee.

- **HARDENING_ROADMAP §3** : pas de ligne S53 prescrite.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 52 CLOSED + audit PASS (`b85a3a1`). Le projet est
Rust+Frontend pur. Le P2P iroh est integre dans le daemon
depuis S7 mais n'a **jamais ete teste entre deux machines
distinctes**. Tout le developpement s'est fait en localhost.

**Infrastructure disponible** :

| Machine | OS | Target Rust | Role |
|---|---|---|---|
| Dev FlowUP | Windows 11, RTX 5080 | `x86_64-pc-windows-msvc` | Daemon instance 1 |
| MacBook Air 15 | macOS ARM (Apple Silicon) | `aarch64-apple-darwin` | Daemon instance 2 |
| sbfb-eu VPS | Ubuntu 24.04 LTS, Hetzner CX33, 8GB RAM | `x86_64-unknown-linux-gnu` | Daemon instance 3 |

IP VPS : `135.181.42.188` (Helsinki, eu-central).
SSH : `ssh -i ~/.ssh/sbfb_hetzner root@135.181.42.188` (cle Ed25519).

**Etat technique (tip `b85a3a1`)** :
- Workspace clean
- `nexus-shell-daemon start` : boote iroh endpoint, ecrit
  `running.json`, bind HTTP loopback, curator subscribe, pkarr
  browse aggregation. Fonctionne sur Windows (dev). Jamais teste
  macOS ni Linux runtime.
- `nexus-shell-daemon init` : cree directories + coordinator.db
- CLI cross-platform : `#[cfg(unix)]` UDS server,
  `#[cfg(windows)]` Named Pipe server. Code compile sur les 3 OS
  (CI GHA ubuntu + release.yml 3 OS). Jamais RUN sur Linux/macOS.
- iroh 0.98 pinne (Day 0 #3). Relay servers n0 par defaut.
- Deploy scripts `deploy/` datent de S10 (architecture Python).
  **Obsoletes** — ne pas utiliser. Build from source.

**Carries entrants S53** :
| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-REVIEW-B-1-S51 unsafe set_var futur | 2/3 | S51 review |
| P2-REVIEW-A-1-S52 nextest timeout profiling | 1/3 | S52 review |
| P2-REVIEW-B-1-S52 Woodpecker E2E validation | 1/3 | S52 review |
| P2-REVIEW-B-2-S52 GHA 9/9 re-run confirm | 1/3 | S52 review |
| P2-AUDIT-1-S52 images CI Woodpecker non pinnees | 1/3 | S52 audit |

### §1.2 Ancrage roadmap

S52 scope cut §1 : "VPS deployment + smoke test — S53".
Reporté depuis S51 → S52 → S53. C'est le prerequis concret
pour LT-7 (self-hosted build, S54-S55).

roadmap_v1_migration_rust.md §S52 : "Premier noeud live
(VPS Hetzner/OVH), Smoke test P2P multi-noeuds, Monitoring baseline."
roadmap_v1.0_alexandria.md §5 : 3 VPS Hetzner provision S10.
Etat reel : 1 VPS CX33 `sbfb-eu` (les 2 autres soit supprimes
soit jamais crees).

### §1.3 Compteurs tests entree (tip `b85a3a1`)

| Suite | Count |
|---|---|
| Rust nextest | 1199 |
| Rust doctests | 6 passed, 1 ignored |
| Vitest | 250 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 6/6 |
| **Total** | **~1455** |

**Post-S53 attendu** : ~1457+ (unsafe set_var fix ~1-2 tests,
smoke test scripts eventuellement).

### §1.4 Pre-launch protocol policy (rappel)

Aucun wire format touche. Le smoke test valide le runtime, pas
les schemas. Si un bug de serialisation est decouvert lors du
test P2P, le canonical est redefini (pas de bump version).

---

## §2 Goal

Premier test reel du protocole SBFB entre machines distinctes :
LAN (Windows ↔ Mac) puis WAN (dev ↔ VPS Helsinki). Valide que
le P2P iroh fonctionne hors localhost, que les binaires tournent
sur 3 OS, et que le VPS bootstrap est operationnel.
**Critere SMART : 20+ rows fail-fast verts au verification.md,
mesure binaire au Phase C wrap-up. Daemon demarre et communique
sur au moins 2 des 3 machines. unsafe set_var CLOSED.**

---

## §3 Phase 0 — Audit gate S52

**DEJA JOUE** : commit `b85a3a1` PASS
(0 P0, 0 P1, 1 P2, 2 P3).
Audit findings dans `.planning/active/sprint52_audit_findings.md`.
7 carries documentes pour S53 (cf. §1.1 ci-dessus).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Build from source sur chaque cible

**Retenu** : installer Rust via `rustup` sur chaque machine
(Mac + VPS) et builder directement `cargo build --release
-p nexus-shell-daemon -p nexus-launcher`. Pas de cross-compilation.
Le code est deja cross-platform (`cfg(unix)` / `cfg(windows)`).

**Rejete** :
- GHA release artifacts : ajoute une dependance GHA pour un
  smoke test. Les artifacts sont signes/attestes (SLSA), overkill
  pour tester le runtime. De plus le run 9/9 n'est pas encore
  confirme (carry P2-REVIEW-B-2-S52).
- Cross-compilation depuis Windows : macOS requiert le SDK Apple
  (XCode headers), non disponible legalement sur Windows. Linux
  cross-compile est possible (via `cross`) mais ajoute Docker.
  Builder sur la cible est plus simple et valide l'environnement.
- Binaires pre-buildes SCP : pas de pipeline de build Linux
  verifie. Builder from source est le chemin le plus fiable.

**Implications code** : 0 modification code. Setup toolchain
sur Mac + VPS uniquement.

### D2 — Smoke test pass criteria

**Retenu** : 3 niveaux de validation, chacun suffisant pour
considerer le sprint reussi. Le test le plus avance atteint
est le resultat du sprint.

**Niveau 1 (minimum)** : daemon demarre sur les 3 OS, ecrit
`running.json`, bind HTTP port, shutdown propre (ctrl+c /
SIGTERM). Valide : binaires cross-platform.

**Niveau 2 (cible)** : 2 daemons sur LAN (Windows + Mac)
se decouvrent mutuellement via iroh relay EU. Les logs montrent
`peer discovered` ou equivalent. Valide : P2P fonctionne.

**Niveau 3 (stretch)** : daemon VPS rejoint le reseau, le
frontend Browse sur la machine dev affiche un projet publie
depuis le Mac (ou inversement). Valide : protocole SBFB
end-to-end.

**Rejete** :
- Test automatise CI : le smoke test est par nature multi-machine,
  pas automatable en CI sans infrastructure. Les resultats sont
  documentes dans le commit body et verification.md.
- Load testing / benchmark : premature — le protocole n'a jamais
  fonctionne entre 2 machines. Valider d'abord, optimiser apres.

**Implications code** : 0 modification code. Documentation des
resultats dans verification.md.

### D3 — VPS setup minimal

**Retenu** : installation manuelle SSH. Rust toolchain + git
clone + cargo build + execution manuelle (`./nexus-shell-daemon start`).
Pas de systemd, pas de nginx, pas de TLS, pas de monitoring.
Le VPS sert exclusivement au smoke test P2P. Le firewall (ufw)
ouvre UDP pour QUIC entrant (iroh direct connectivity).

**Rejete** :
- systemd service : premature pour un smoke test. Le daemon sera
  lance manuellement et tue apres le test. systemd viendra quand
  le VPS sera un noeud permanent (LT-7 scope).
- Docker : ajoute une couche d'abstraction qui masque les
  problemes runtime reels (filesystem, ports, permissions).
  Builder et run natif est plus instructif pour un premier test.
- Reprovisioning complet (provision.sh S10) : le script est
  pour l'architecture Python. Il faudrait le reecrire — hors
  scope smoke test.

**Implications code** : 0 modification code. Commandes SSH
documentees dans le plan.

### D4 — unsafe set_var resolution (carry 2/3)

**Retenu** : wrapper les appels `std::env::set_var` existants
dans des blocs `unsafe {}` avec commentaire SAFETY documentant
que l'appel est fait avant le spawn de threads (single-threaded
context au point d'appel). Rust 1.94 a rendu `set_var` unsafe
car la mutation de l'environnement n'est pas thread-safe.

**Rejete** :
- Supprimer les appels `set_var` : certains configurent le
  runtime (RUST_LOG, OLLAMA_HOST) et sont necessaires.
- Refactorer vers un config struct explicite : changement
  d'architecture significatif hors scope smoke test. Le pattern
  `unsafe set_var` avant spawn est valide et documente dans
  le Rust ecosystem.
- Reporter a S54 (3/3 MANDATORY) : risque de bloquer S54
  avec du travail mecanique. Mieux de resoudre maintenant.

**Implications code** : 2-5 fichiers touches (grep `set_var`),
~10-20 LOC de blocs unsafe + commentaires SAFETY.

---

## §5 Plan Phase outline A..C

### Phase A — Build cross-platform + smoke test LAN (Windows ↔ Mac)

**But** : builder le daemon sur macOS ARM et valider le P2P
sur le reseau local entre la machine dev et le MacBook.

- Guider l'utilisateur : install Rust sur Mac, clone repo, build
- Demarrer daemon sur Windows et Mac
- Verifier Niveau 1 (daemon start/stop sur les 2 OS)
- Tester Niveau 2 (decouverte P2P via iroh relay LAN)
- Documenter les resultats et bugs trouves
- Fixer les bugs bloquants trouves (si runtime, pas wire format)
- Commit : `feat(sprint53): Sprint 53 Phase A — cross-platform
  build + P2P smoke test LAN Windows-macOS`

### Phase B — VPS deployment + smoke test WAN (dev ↔ VPS)

**But** : builder le daemon sur le VPS Linux et valider le P2P
a travers internet.

- Install Rust sur VPS, clone repo, build
- Configurer firewall UDP pour QUIC
- Demarrer daemon sur VPS
- Verifier Niveau 1 (daemon Linux runtime)
- Tester Niveau 2/3 (decouverte P2P cross-network)
- Documenter les resultats
- Commit : `feat(sprint53): Sprint 53 Phase B — VPS bootstrap
  + P2P smoke test WAN dev-Helsinki`

### Phase C — unsafe set_var + verification + wrap-up

**But** : resoudre le carry 2/3, executer la verification
fail-fast, rediger l'audit plan S54.

- Fix unsafe set_var (D4)
- CLAUDE.md : compteurs, carries S54
- HARDENING_ROADMAP : update last_validated S53
- Verification fail-fast 20+ checks
- sprint54_audit_plan.md
- Commit : `chore(sprint53): Phase C — unsafe set_var fix +
  wrap-up + verification + audit plan S54`

---

## §6 Items carry/dette

### Carries confirmes S53

- [carry] **P2-A-1** rand blocker upstream 12+/3 : exemption
  externe (pas de release rand 0.9 ni fix getrandom).
- [carry] **P2-AUDIT-2** iroh transitives : herite pin 0.98
  (Day 0 #3).
- [dette] **P2-REVIEW-B-1-S51** unsafe set_var futur 2/3 :
  **ADRESSE Phase C** → CLOSE attendu.
- [carry] **P2-REVIEW-A-1-S52** nextest timeout profiling 1/3 :
  pression ressources Windows, non-regression (monitorer S53).
- [carry] **P2-REVIEW-B-1-S52** Woodpecker E2E validation 1/3 :
  agent VPS hors scope S53 smoke test, reporte S54.
- [carry] **P2-REVIEW-B-2-S52** GHA 9/9 re-run confirm 1/3 :
  pas de trigger GHA ce sprint. Reporte S54.
- [carry] **P2-AUDIT-1-S52** images CI Woodpecker non pinnees
  1/3 : pas de deploiement agent S53. Reporte S54.

### S53 impair — pas de phase dette obligatoire

unsafe set_var a 2/3 adresse en Phase C (prevention MANDATORY S54).

### Carries residuels post-S53

| Item | Compteur S54 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-REVIEW-A-1-S52 nextest timeout | 2/3 | S52 review |
| P2-REVIEW-B-1-S52 Woodpecker E2E | 2/3 | S52 review |
| P2-REVIEW-B-2-S52 GHA 9/9 | 2/3 | S52 review |
| P2-AUDIT-1-S52 images CI pin | 2/3 | S52 audit |

**Attention S54 pair** : 4 items passent a 2/3 — si non adresses
S54, ils deviennent 3/3 MANDATORY S55 (surcharge).

---

## §7 Scope cuts

1. **Woodpecker agent VPS** — S54 (avec LT-7)
2. **systemd service VPS** — S54 (deploiement permanent)
3. **VPS TLS + nginx** — S54 (production readiness)
4. **VPS monitoring + alerting** — S54+
5. **LT-1 Kudos-v2 fairness reform** — sprint dedie (S55+)
6. **LT-7 self-hosted build** — S54-S55 (prerequis VPS valide S53)
7. **Events SSE daemon-native** — post-v1.0
8. **MCP server Rust** — post-v1.0
9. **Pagination SQL-side LIMIT/OFFSET** — S54+
10. **Test infra mk_state() refactoring** — S54+
11. **Deploy scripts rewrite (provision.sh)** — S54 (apres smoke test)
12. **Load testing / benchmark P2P** — post smoke test

---

## §8 Tracabilite scope (S52 → S53)

| S52 scope cut | S53 disposition |
|---|---|
| VPS deployment + smoke test — S53 | **Phase A + B** (theme principal) |
| Pagination SQL-side — S53+ | Scope cut reporte S54+ |
| Test infra mk_state() — S53+ | Scope cut reporte S54+ |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Build echoue sur macOS ARM (dep systeme manquante, linker) | Medium | High | Builder incrementalement, fixer les erreurs une par une |
| R2 | Build echoue sur VPS Linux (8GB RAM, linking) | Low | Medium | VPS CX33 a 8GB, suffisant. Swap si besoin |
| R3 | iroh P2P ne traverse pas le NAT (relay down, QUIC bloque) | Low | High | Relay EU n0 par defaut. VPS a IP publique (pas de NAT). LAN = pas de NAT |
| R4 | Daemon crash au demarrage sur Linux (runtime bug cfg(unix)) | Medium | Medium | Les tests CI passent sur ubuntu. Debug SSH si crash |
| R5 | Flaky test browse probe_and_cache timing | Low | Low | Pre-existant, non imputable S53, monitorer |
| R6 | unsafe set_var touch des chemins complexes | Low | Low | Grep exhaustif, wrapping mecanique |

---

## §10 Audit gate pattern — rappel

Phase 0 S52 jouee (PASS `b85a3a1`). Phase C produira
sprint54_audit_plan.md pour la session fraiche S54.

---

## §11 Checkpoint de validation

1. **D1** : build from source sur chaque machine ?
   → oui (pas de cross-compilation, pas de GHA dependency)
2. **D2** : smoke test 3 niveaux (start / P2P LAN / P2P WAN) ?
   → oui (niveau atteint = resultat du sprint)
3. **D3** : VPS setup minimal (pas systemd) ?
   → oui (smoke test, pas production)
4. **D4** : unsafe set_var wrap + SAFETY comment ?
   → oui (prevention MANDATORY S54)
