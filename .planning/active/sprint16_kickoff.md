# Sprint 16 — Kickoff (Security hardening : loopback auth + GPU opt-in + roadmap VM)

**Ecrit** : 2026-04-14
**Tip master d'entree** : `4da0043` (Sprint 15 Phase E docs landed)
**Phase 0 audit** : **A JOUER**. Sprint 15 audit plan dans
`.planning/archive/v1.1/sprint15_audit_plan.md` doit produire
`.planning/active/sprint15_audit_findings.md` avant le premier
commit Phase A.
Verdict attendu (pre-audit) : PASS (les 4 commits Phase A-D sont
serres, delta tests +26, 0 clippy, 0 flake).

---

## 1. Constat d'entree

### 1.1 Etat du repo

- Sprints 0-15 **CLOSED**. v1.0.0 released. Tip `4da0043`.
- Sprint 15 a livre le **bridge push bidirectionnel** + **CPU
  watchdog** + **CLI `sbfb init`** + **E2E Playwright iframe**.
  La plateforme est maintenant **developer-ready** : un dev
  cree + publie une app en 3 commandes (`sbfb init react x`,
  `npm run build`, `curl POST /project/deploy-from-repo`).
- La **securite par iframe** est excellente (browser-enforced :
  sandbox sans same-origin + CSP `connect-src 'none'`). Les 3
  methodes du bridge (`task_submit`, `storage_get`, `storage_set`)
  + le nouveau canal `onEvent` sont le seul chemin, par design.
- Le **deploy verifie Sprint 14** (Keyoxide + SLSA L1 provenance)
  garantit que le code sur le reseau = le code du repo.

### 1.2 Compteurs de tests a l'entree (tip `4da0043`)

| Suite | Count |
|---|---|
| Rust workspace | 373 |
| Python SDK | 182 + 1 flaky Windows |
| Python coordinator | 153 + 1 skipped |
| Python app-gov | 46 |
| Vitest unit | 214 |
| Playwright | 33 |
| size-limit | 7/7 |
| SPDX | 224/224 |

Total : ~934 tests.

### 1.3 Le probleme — couche locale non-authentifiee

L'analyse de securite post-Sprint 15 a identifie que la couche
**loopback** est la plus faible du systeme :

| Endpoint | Port | Auth |
|---|---|---|
| Coordinator FastAPI | `127.0.0.1:8080` | **aucune** |
| Shell daemon HTTP | `127.0.0.1:7777` | **aucune** |
| Blob-serve | `127.0.0.1:7000` | **aucune** |

**Consequences concretes** :

1. **Extensions browser** avec `host_permissions: "http://localhost/*"`
   peuvent hitter `POST /project/deploy-from-repo` et publier sous
   l'identite de l'utilisateur.
2. **Autres apps desktop** sur la meme machine peuvent lire/ecrire
   le storage SBFB, soumettre des tasks, voire exfiltrer via
   endpoints debug.
3. **DNS rebinding** theoriquement possible si les endpoints ne
   verifient pas strictement le header `Host:`.
4. **Keypair Ed25519** probablement stockee plaintext dans
   `~/nexus-grid/shell-daemon/` — tout process user-mode y accede.

Le modele de menace implicite "la machine locale est de confiance"
ne tient plus face aux extensions navigateur modernes et aux
malwares user-mode. La couche iframe est **A**, la couche loopback
est **D**.

### 1.4 Le manque produit — consentement GPU explicite

Le worker (`crates/nexus-worker`) execute deja les tasks Ollama,
mais **aucun flag utilisateur** ne distingue "je contribue mon GPU
a tout le reseau" de "je contribue uniquement a mes propres
projets". Le modele P2P implique que les workers acceptent de
servir des tasks publiques sans opt-in explicite. Ce n'est pas
conforme aux attentes RGPD/energie (un utilisateur doit
explicitement consentir a partager son CPU/GPU) et freine
l'adoption "je ne sais pas combien ca me coute".

Il manque egalement un flag `is_open_source` sur les projets qui
permettrait aux curators et aux workers de filtrer
("je contribue uniquement aux projets marques open source").

### 1.5 La direction long-terme — isolation runtime (VM invisible)

Lors de la discussion pre-kickoff, l'utilisateur a propose que
le daemon + coordinator + keypair vivent dans un **runtime isole**
(WSL2 sur Windows, Virtualization.framework sur Mac, namespaces
sur Linux) installe automatiquement au premier lancement, sans
friction visible pour l'utilisateur.

Ce chemin resout **95%** des problemes loopback cites en §1.3 :
l'iframe vit dans un `localhost` isole du host, invisible aux
extensions et aux malwares user-mode. CUDA passthrough WSL2 est
native depuis 2022 donc la RTX 5080 reste exploitable.

**Decision Sprint 16** : ne PAS implementer l'auto-install VM ce
sprint (trop scope), mais :
- Documenter le threat model et la roadmap dans
  `docs/security/THREAT_MODEL.md` et
  `docs/security/RUNTIME_ISOLATION.md`
- Poser les fondations compatibles VM : bearer token + UDS que
  le binding ne change pas entre host et VM
- Laisser Sprint 17+ traiter l'auto-install WSL2/Docker

### 1.6 Vision sprint

Sprint 16 livre la **securite loopback de base** (bearer token +
UDS/named pipes quand disponible), le **consentement GPU opt-in**
avec toggle "partager avec le reseau" et flag `is_open_source`
sur les projets, et la **documentation roadmap** pour l'isolation
runtime (WSL2 / VM) en Sprint 17+.

Zero breaking change pour les apps. Zero perte d'UX pour les
utilisateurs existants (token genere automatiquement au boot,
opt-in GPU default = "mes projets uniquement" = comportement
actuel preserve).

---

## 2. Goal en une phrase

**La couche loopback passe de D a A- via defense en profondeur
(bearer token 256-bit + Host/Origin header allowlist +
SO_PEERCRED sur UDS + Named Pipes Windows avec DACL custom) ;
l'utilisateur consent explicitement a partager son GPU via un
toggle UI avec cap W/VRAM/h enforced dans worker-core ; le
threat model STRIDE + LINDDUN et la roadmap VM isolation sont
ecrits pour Sprint 17+.**

---

## 3. Phase 0 — Audit Sprint 15 (a jouer)

Session fraiche lit `.planning/archive/v1.1/sprint15_audit_plan.md`
et execute les 7 tracks A-G (bridge push protocol, watchdog state
machine, CLI scaffold, Playwright E2E, backward compat Sprint 13,
scope cuts, couverture tests). Timebox 2-3h.

Livrable : `.planning/active/sprint15_audit_findings.md` (sera
deplace avec les autres docs S15 en `archive/v1.1/` a la cloture
S16).

Verdict attendu : PASS ou CONDITIONAL PASS (1-3 P1 fixables).
Les fix eventuels doivent landed avant le commit Phase A.

---

## 4. Decisions Day 0 (D1..D5 proposees — a valider post-audit)

### D1 — Defense en profondeur loopback (bearer + Host + Origin)

**Retenu** : triple validation de chaque requete HTTP loopback,
inspire du pattern Jupyter/Syncthing + mitigation CVE-2025-49596
(Anthropic MCP Inspector, RCE via DNS rebinding) :

1. **Bearer token 256-bit** genere par le launcher Rust au 1er
   boot, stocke `~/.sbfb/auth_token` mode `0600` (Unix) ou
   ACL user-only (Windows), parent dir `~/.sbfb/` mode `0700`.
   Lu au demarrage par daemon + coordinator + shell, envoye en
   header `X-SBFB-Token: <hex>` sur chaque appel.
2. **Host header allowlist** : le middleware rejette tout
   `Host:` non inclus dans `{localhost, 127.0.0.1, [::1]}`
   (bloque DNS rebinding ou serveur utilise comme relay).
3. **Origin header check** : si present, doit correspondre au
   shell React (`http://localhost:<shell_port>` ou
   `about:blank`). Absent autorise pour CLI. Bloque les fetch
   cross-origin depuis sites malveillants ou extensions
   navigateur avec `host_permissions: "http://localhost/*"`.

Exception unique : `/health` reste public (probe du launcher).

**Rejete** :
- OAuth local : surdimensione, UX complique
- mTLS loopback : overhead cert rotation, perte UX
- Bearer seul sans Host/Origin : laisse passe rebindings DNS
  (cf. CVE-2025-49596 Anthropic, CVSS 9.4)
- Rotation auto du token : BOINC/Jupyter/Syncthing ne rotent
  pas, MVP accepte un token stable (user peut supprimer le
  fichier pour forcer regen au prochain boot)

**Implications** :
- `crates/nexus-launcher` +50 LOC : genere + persiste le token
  au boot, garantit perms `0600` / `0700`
- `crates/nexus-shell-daemon-core` +60 LOC : middleware axum
  qui valide bearer + Host + Origin (exception `/health`)
- `packages/nexus-coordinator` +70 LOC : middleware FastAPI
  idem (exception `/health`)
- `web/src/api/*.ts` +20 LOC : fetch interceptor qui injecte
  le token (lu une fois via endpoint `/auth/token` du launcher)
- Tests : 30+ Vitest + pytest + cargo verifiant 401 sans token,
  401 bad Host, 401 bad Origin, 200 avec triple valide

### D2 — UDS durcis (SO_PEERCRED) + Named Pipes avec DACL custom

**Retenu** : en plus du TCP bearer-authentifie (D1), le daemon
et le coordinator exposent une seconde surface via :

- **Unix Domain Sockets** Linux/Mac/FreeBSD :
  `~/.sbfb/run/daemon.sock`, `~/.sbfb/run/coordinator.sock`,
  mode `0600`, parent dir `0700`. **Validation SO_PEERCRED**
  (Tailscale `safesocket.PlatformUsesPeerCreds` pattern) : le
  serveur lit les credentials OS du peer via `getsockopt` et
  rejette si uid != uid propre. Auth native de l'OS, independante
  du token (belt-and-braces).

- **Named Pipes Windows** : `\\.\pipe\sbfb-daemon`,
  `\\.\pipe\sbfb-coordinator` **avec `SECURITY_ATTRIBUTES`
  custom** obligatoirement. Default DACL = Everyone readable
  = **vulnerable** (tout process user-mode peut hitter le
  pipe). DACL custom avec logon SID du user courant uniquement
  (pattern Tailscale `\\.\pipe\ProtectedPrefix\...`). Implemente
  via crate `windows-rs` + `CreateNamedPipeA` avec SD explicite.

Le shell React utilise exclusivement le TCP (browser sans API
UDS) protege par D1. Les binaires Rust et le CLI `sbfb` parlent
en UDS/NP **en priorite**, TCP bearer-auth en fallback si socket
absent.

**Rejete** :
- UDS seul : casse le browser qui ne parle qu'en TCP
- TCP sur `0.0.0.0` : anti-pattern
- Named Pipe avec DACL default : vulnerable (cf.
  Microsoft docs « Named Pipe Security and Access Rights »)
- Bearer sur UDS : redondant avec SO_PEERCRED (mais on garde
  le bearer end-to-end pour coherence code, trivial)

**Implications** :
- `crates/nexus-shell-daemon` +80 LOC : listener UDS (tokio
  UnixListener) + SO_PEERCRED check, config `SBFB_DAEMON_SOCKET`
- `crates/nexus-shell-daemon-core` +30 LOC : helper
  `verify_peer_creds(stream) -> Result<()>`
- `crates/nexus-shell-daemon` Windows +120 LOC : Named Pipe
  listener via `windows-rs` avec SECURITY_ATTRIBUTES (new
  dep `windows = { version = "...", features = ["..."] }`)
- `packages/nexus-coordinator` +60 LOC : UnixServer asyncio /
  Win32 named pipe via pywin32 (ou Rust side-car si plus
  simple — a trancher en Phase B research)
- `crates/nexus-launcher` +40 LOC : cree `~/.sbfb/run/` mode
  `0700` au boot, ajoute SID du user courant a DACL Windows
- Tests : 25+ cargo + pytest verifiant peer creds rejette
  uid different (simule via fork+setuid dans un docker test),
  DACL Windows rejette un autre user (skip si CI Linux-only)

### D3 — Consent screen 3 niveaux + caps enforced worker-side

**Retenu** : inspire du pattern BOINC `UserOptInConsent`
(ENROLL/STATSEXPORT) et conforme GDPR Art.7 (opt-in explicite,
granular, withdrawal aussi simple que donnee).

Nouveau composant React `web/src/components/GpuConsentDialog.tsx`
affiche au premier boot (apres creation de la keypair) un dialog :
- Explication : "SBFB est un reseau P2P de compute. Tu peux
  choisir comment ton GPU est utilise."
- **3 options radio** (pre-coche interdit par GDPR) :
  - "Uniquement mes projets" (default a l'ouverture, zero
    partage — GDPR-safe)
  - "Projets open source verifies" (accepte uniquement les apps
    avec `is_open_source: true` dans leur annonce P2P)
  - "Tous les projets publics" (opt-in complet)
- **Caps configurables** : W max, VRAM max MB, heures/jour max
- Bouton "Enregistrer" persiste dans `~/.sbfb/consent.json`

**Enforcement** : le worker `crates/nexus-worker-core::allowlist`
lit `consent.json` au boot ET a chaque claim de task :
1. Filtre par niveau (reject si task.is_open_source=false et
   niveau=2, reject toute task non-local si niveau=1)
2. **Enforce caps actifs** : si une task demande plus que W max
   ou VRAM max, reject. Si cumul journalier atteint h max, reject
   toute nouvelle task jusqu'au reset minuit-local.
3. Log chaque rejection avec raison (observability).

Les caps ne sont PAS juste des valeurs cosmetique UI : elles sont
la source de verite pour `allowlist.should_accept_task(&task)`.
Withdrawal via menu Network > "Modifier consentement", meme
dialog, re-ecrit le fichier.

**Rejete** :
- Default "tout le reseau" : viol GDPR + anti F-Droid ethique
- Consent implicite "si tu installes c'est que tu acceptes" :
  idem
- Caps UI-only (pas enforced) : trompeur pour l'utilisateur

**Implications** :
- `web/src/components/GpuConsentDialog.tsx` +220 LOC (3 radios
  + caps sliders + validation)
- `web/src/pages/Network.tsx` +30 LOC (badge + bouton "Modifier
  consentement")
- `crates/nexus-worker-core::allowlist` +160 LOC : charge
  consent.json, filtre tasks par is_open_source + visibility,
  enforce caps W/VRAM/h (daily counter persisted to
  `~/.sbfb/usage.json`)
- `packages/nexus-coordinator/src/nexus_coordinator/consent.py`
  +60 LOC : API `/consent/get` et `/consent/set`
- Tests : 30+ Vitest + cargo + pytest (inclut cap enforcement :
  task >maxW rejected, cumul >max_h rejected, reset quotidien)

### D4 — Flag `is_open_source` sur ProjectAnnouncement v5

**Retenu** : extension du schema `ProjectAnnouncement` (v4 deja
Sprint 14) avec un nouveau champ booleen `is_open_source`. Deja
implicite : tout projet deploye via `deploy-from-repo` (repo
public) est open source ; tout projet deploye via `deploy`
(zip prive) ne l'est pas. Le champ est donc **derive
automatiquement** par le coordinator au moment du publish, pas
entre a la main.

**Rejete** :
- Champ user-settable : risque de flag "open source" pour
  un zip non-verifiable
- Ne rien ajouter et laisser le worker re-deriver du repo_url :
  force le worker a re-verifier chaque annonce, cout n^2

**Implications** :
- `web/src/bridge/project_announcement.ts` (ou equivalent Zod
  schema) : version bump 4 → 5, ajout champ
- `crates/nexus-core-rs::project_announcement` : ajout champ +
  test backward compat v4 (v5 peut decoder v4 avec
  is_open_source=false par defaut)
- `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py`
  +5 LOC : set le flag lors de deploy-from-repo
- Tests : +8 cargo + +4 pytest + +3 Vitest

### D5 — Threat model STRIDE + LINDDUN + runtime isolation roadmap

**Retenu** : 3 documents dans `docs/security/`, combinant STRIDE
(security, pattern classique Microsoft) et LINDDUN (privacy,
pertinent pour un reseau P2P qui collecte stats workers) via
pattern OWASP Threat Dragon.

1. `docs/security/README.md` — index + matrice de severite +
   pointeur vers les 2 autres docs + instructions pour
   contributeurs (comment etendre le threat model).

2. `docs/security/THREAT_MODEL.md` ~500 LOC — modele formel :
   - **Assets** : keypair Ed25519, zip artifacts, provenance
     signatures, user consent.json, usage.json, project archives,
     task results, kudos ledger
   - **Adversaires** : extension navigateur malveillante,
     malware user-mode local, node byzantin P2P, repo git
     squatte, fornisseur d'app malveillant
   - **STRIDE** par composant (iframe A, deploy B+, iroh B+,
     loopback post-S16 A-, key storage C?, supply chain C) :
     Spoofing / Tampering / Repudiation / Info disclosure / DoS
     / Elevation of privilege
   - **LINDDUN** par flux de donnees :
     Linkability (CPID-like cross-project identifier worker ?) /
     Identifiability (GPU/CPU stats fingerprinting ?) /
     Non-repudiation (provenance signee = trace non-niable) /
     Detectability (un peer peut savoir qui run quoi ?) /
     Disclosure (consent.json contient prefs sensibles) /
     Unawareness (user informe ?) / Non-compliance (GDPR)
   - Tableaux mitigations + severite (CVSS 3.1-like) + residual
     risk par item
   - Diagramme DFD ASCII des flux (iframe ↔ bridge ↔ coord ↔
     iroh ↔ peers)

3. `docs/security/RUNTIME_ISOLATION.md` ~350 LOC — roadmap VM :
   - Pourquoi (loopback → VM elimine 95% des risques locaux)
   - Tech cible : WSL2 (Windows) / Virtualization.framework
     (Mac) / systemd-nspawn (Linux)
   - CUDA passthrough WSL2 (natif depuis 2022)
   - Phasage propose pour Sprint 17+ :
     * S17 Phase A : detection environnement (probe WSL2)
     * S17 Phase B : bootstrap WSL2 image signee
     * S17 Phase C : migration daemon + coord dans VM
     * S17 Phase D : fallback machines sans virtualisation

**Rejete** :
- Implementer l'auto-install WSL2 ce sprint : scope creep
  massif, touche launcher + CI + packaging
- Ne documenter qu'a Sprint 17 : perd l'alignement
  architectural que D1-D2 posent
- STRIDE uniquement : insuffisant pour un reseau P2P qui
  collecte des stats workers (LINDDUN privacy requis pour
  conformite GDPR)

**Implications** :
- `docs/security/` : dossier cree
- `docs/security/README.md` ~60 LOC (index + severity matrix)
- `docs/security/THREAT_MODEL.md` ~500 LOC (STRIDE + LINDDUN)
- `docs/security/RUNTIME_ISOLATION.md` ~350 LOC
- Lien depuis `README.md` section "Security" (deja pose en
  v1.1 cleanup, update avec pointeur vers threat model)
- Lien depuis `CLAUDE.md` section "Etat actuel"
- Lien depuis `docs/claude/README.md` §10 table

---

## 5. Plan Phase outline

### Phase 0 — Audit Sprint 15

Session fraiche joue `sprint15_audit_plan.md`, produit
`sprint15_audit_findings.md`, landed eventuels
`fix(sprint15): ...`. Verdict attendu : PASS.

**Commit** : 1 commit par fix (si necessaire), pas de commit
dedie au findings doc (fait partie du Phase A si PASS).

### Phase A — Bearer token loopback

**Scope** :
- `crates/nexus-launcher` : genere + persiste token, expose via
  endpoint `/auth/token` (loopback seul, lit depuis `~/.sbfb/auth_token`)
- `crates/nexus-shell-daemon-core` : middleware axum
- `packages/nexus-coordinator` : middleware FastAPI
- `web/src/api/*.ts` : interceptor fetch + hook `useAuthToken`
- Tests : 401 sans header, 200 avec, token rotation

**Critere** : requests loopback sans `X-SBFB-Token` -> 401.
Avec token valide -> 200. Rotation du fichier
`~/.sbfb/auth_token` → tous les composants se reconnectent sans
redemarrer le daemon.

**Commit** : `feat(auth): Sprint 16 Phase A — bearer token loopback auth`

### Phase B — Unix Domain Sockets + Named Pipes

**Scope** :
- `crates/nexus-shell-daemon` : listener UDS + NP
- `packages/nexus-coordinator` : idem
- `crates/nexus-launcher` : cree `~/.sbfb/run/` perm 0700
- Config discovery : env `SBFB_DAEMON_SOCKET` ou defaut
- Tests : cargo + pytest

**Critere** : le CLI `sbfb` parle au daemon via UDS sur Unix,
Named Pipe sur Windows, TCP fallback si socket absent. Tests
verifient les 3 chemins.

**Commit** : `feat(net): Sprint 16 Phase B — UDS + Named Pipes loopback`

### Phase C — Consent screen + worker filtering

**Scope** :
- `web/src/components/GpuConsentDialog.tsx` + hook
- `web/src/pages/Network.tsx` badge
- `packages/nexus-coordinator` API `/consent/*`
- `crates/nexus-worker-core::allowlist` : lecture consent.json
- Tests : Vitest + cargo + pytest

**Critere** : premier boot affiche le dialog. Consent persiste.
Worker refuse les tasks hors du scope consenti (testable en
injectant une task flag `is_open_source=false` quand user a
choisi "open source uniquement").

**Commit** : `feat(consent): Sprint 16 Phase C — GPU sharing opt-in dialog + worker filtering`

### Phase D — ProjectAnnouncement v5 + is_open_source flag

**Scope** :
- `crates/nexus-core-rs::project_announcement` bump v4 → v5
- `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py`
  set le flag
- `web/src/bridge/project_announcement.ts` (ou Zod schema)
- Backward compat : decoder v4 avec default `is_open_source=false`
- Tests : decode v4 works, encode v5 works, round-trip

**Critere** : un projet deploye via `deploy-from-repo` a
`is_open_source: true` dans son annonce. Un noeud sur l'ancienne
version decode l'annonce v5 sans crasher.

**Commit** : `feat(p2p): Sprint 16 Phase D — ProjectAnnouncement v5 with is_open_source flag`

### Phase E — Documentation security + roadmap

**Scope** :
- `docs/security/THREAT_MODEL.md` (nouveau)
- `docs/security/RUNTIME_ISOLATION.md` (nouveau)
- `README.md` : nouvelle section "Security" avec liens
- `CLAUDE.md` : update "Etat actuel" + lien docs/security/
- `docs/claude/README.md` : row Sprint 16 dans §10 table
- `docs/shell/PATTERNS.md` : nouveau pattern "bearer token loopback"
- `.planning/active/sprint16_verification.md` : fail-fast
- `.planning/active/sprint16_audit_plan.md` : plan audit Sprint 17 Phase 0

**Commit** : `docs(sprint16): verification + audit plan + security roadmap`

---

## 6. Scope cuts (PAS dans ce sprint)

- **Auto-install WSL2 / VM au premier boot** → Sprint 17+.
  Ce sprint ecrit la roadmap, Sprint 17 implemente.
- **Encryption at rest de la keypair** (Keychain / DPAPI /
  libsecret) → Sprint 17+. Perm `0600` est le MVP ce sprint.
- **cargo-audit / pip-audit / npm audit en CI** → Sprint 17+
  (infra CI hors scope)
- **Rate limiting** sur `/project/deploy-from-repo` → Sprint 17+
  (quota/abuse layer)
- **CSP report-uri** pour surveillance attaques bloquees →
  Sprint 17+ (endpoint + telemetrie)
- **Audit externe** (Trail of Bits / Cure53) → post-v1.1, budget
  hors scope projet solo
- **Bug bounty** → post-v1.1
- **Revocation de node_id** (CRL Ed25519) → v2.0+
- **MIME scan** dans le zip deploy (P2 Sprint 14 T47) → Sprint 17+
  (la sandbox iframe suffit comme MVP)
- **Multi-level consent** (par-projet plutot que global) →
  Sprint 17+ si feedback utilisateur le demande
- **Bytecode signing** des wheels PyO3 → v2.0+

---

## 7. Tracabilite scope (items differes des sprints precedents)

| Item | Origine | Sprint 16 |
|---|---|---|
| Bearer token loopback | Analyse securite post-S15 | **Phase A** |
| UDS / Named Pipes | Analyse securite post-S15 | **Phase B** |
| Consent screen GPU | Feedback utilisateur pre-kickoff | **Phase C** |
| `is_open_source` flag | Alignement vision F-Droid | **Phase D** |
| Threat model documentation | Sprint 13/14/15 implicite | **Phase E** |
| Runtime isolation roadmap | Discussion pre-kickoff | **Phase E** |
| Re-publish auto (scope cut S14/S15) | S12-S15 | Differe Sprint 17+ |
| Branding SBFB (scope cut S10-S15) | S10-S15 | Differe Sprint 17+ |
| Origin subdomain (scope cut S12-S15) | S12-S15 | Differe Sprint 17+ |
| VPS US/Asia (scope cut S12-S15) | S12-S15 | Differe Sprint 17+ |
| MIME scan (scope cut S14/S15) | S14-S15 | Differe Sprint 17+ |
| Templates Vue/Svelte/Jupyter | S15 scope cut | Differe Sprint 17+ |
| `sbfb publish` integre au CLI | S15 scope cut | Differe Sprint 17+ |
| Dispatcher server-side events | S15 scope cut | Differe Sprint 17+ |

---

## 8. Audit gate pattern — rappel

Phase 0 Sprint 15 a jouer. Phase E de ce sprint produira
`sprint16_audit_plan.md` pour que Sprint 17 Phase 0 audite
independamment. Pattern permanent depuis Sprint 7.

---

## 9. Estimations LOC (renforcees post-recherche 2026-04-14)

| Phase | LOC estimee | Repartition |
|---|---|---|
| 0 — Audit S15 | ~330 (findings + PARA cleanup) | DONE : findings `e99c06f` + PARA `14ec51e` |
| A — Bearer + Host + Origin | ~550 | 50 launcher + 60 daemon + 70 coord + 20 web + 350 tests |
| B — UDS/NP avec peer creds + DACL | ~600 | 80 daemon UDS + 120 daemon NP Windows + 30 core + 60 coord + 40 launcher + 270 tests |
| C — Consent 3 levels + caps enforced | ~680 | 220 dialog + 30 Network + 60 consent.py + 160 allowlist (caps!) + 210 tests |
| D — PA v5 is_open_source | ~250 | 30 core-rs + 5 deploy + 15 web + 200 tests |
| E — Docs STRIDE+LINDDUN+VM | ~1100 | 60 README + 500 threat model + 350 runtime-isolation + 100 verif + 100 audit_plan |
| **Total** | **~3230** | |

Delta par rapport a l'estimation initiale (2900) : +330 LOC en
grande partie dues a :
- Named Pipes Windows SECURITY_ATTRIBUTES custom (+120)
- Cap enforcement worker-core avec usage.json daily counter (+80)
- LINDDUN privacy section dans threat model (+100)
- Host/Origin header allowlist (+100 tests)

---

## 10. Checkpoint de validation

**Status** : D1..D5 valides post-recherche le 2026-04-14 par
l'utilisateur. Recherche a couvert : Tailscale safesocket,
Syncthing GUI API, Jupyter server token, BOINC UserOptInConsent,
SLSA v1 + cosign self-managed keys, CVE-2025-49596 (Anthropic
MCP Inspector RCE via DNS rebinding), Windows Named Pipe DACL,
OWASP Threat Dragon STRIDE+LINDDUN.

Decisions confirmees :

1. **D1** — Bearer 256-bit launcher-generated + **Host header
   allowlist** + **Origin header check** (triple validation,
   defense en profondeur, mitigation CVE-2025-49596)
2. **D2** — UDS `0600` avec **SO_PEERCRED** validation
   (Tailscale pattern) + Named Pipes Windows **avec
   SECURITY_ATTRIBUTES custom** (DACL user-only, critique car
   default DACL Windows est permissif)
3. **D3** — Consent 3 niveaux (default "mes projets", GDPR-safe,
   pattern BOINC) + **caps W/VRAM/h enforced dans worker-core**
   avec daily counter `~/.sbfb/usage.json` (pas juste UI)
4. **D4** — ProjectAnnouncement v5 avec `is_open_source` derive
   automatiquement par le coordinator, backward compat v4
5. **D5** — `docs/security/` avec **STRIDE + LINDDUN** (privacy
   requis GDPR) + runtime isolation roadmap WSL2/VM Sprint 17+

Ordre des phases valide : **A** bearer+Host+Origin → **B** UDS/NP
peer-auth → **C** consent+caps → **D** PA v5 → **E** docs.

Scope cuts confirmes differes Sprint 17+ : auto-install WSL2,
encryption at rest keypair, CI security audit (cargo-audit/pip-
audit/npm audit), rate limiting deploy-from-repo, CSP report-uri,
audit externe, bug bounty, revocation node_id, MIME scan (P2 S14
T47), multi-level consent per-project, bytecode signing PyO3.

Backward compat : PA v4 reste decodable (default
`is_open_source=false`), pas de migration forcee. Redemarrer
daemon + coordinator apres upgrade suffit pour que le launcher
genere le token et que les middlewares appliquent le mode strict.
