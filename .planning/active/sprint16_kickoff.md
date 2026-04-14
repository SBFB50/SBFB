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

**La couche loopback passe de D a A- via bearer token + UDS ;
l'utilisateur consent explicitement a partager son GPU via un
toggle UI ; la roadmap VM isolation est ecrite pour Sprint 17+.**

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

### D1 — Bearer token loopback genere au boot par le launcher

**Retenu** : le launcher Rust (`crates/nexus-launcher`) genere un
token 256-bit random au premier boot, ecrit dans
`~/.sbfb/auth_token` avec permissions `0600` (Unix) ou ACL
user-only (Windows). Le daemon, le coordinator et le shell lisent
ce fichier au demarrage et l'envoient en header
`X-SBFB-Token: <hex>` sur chaque appel HTTP loopback.

**Rejete** :
- OAuth local : surdimensione, PR-compliqu
- mTLS loopback : overhead cert rotation, perte UX
- Unix sockets seuls : ne resolvent pas Windows proprement

**Implications** :
- `crates/nexus-launcher` +50 LOC : genere + persiste le token au
  boot, garantit perm `0600`
- `crates/nexus-shell-daemon-core` +30 LOC : middleware axum qui
  rejette les requests sans header valide (excepte `/health`)
- `packages/nexus-coordinator` +40 LOC : middleware FastAPI idem,
  exception `/health` + `/api/public/*` si besoin
- `web/src/api/*.ts` +20 LOC : fetch interceptor qui injecte le
  token (lu une fois au boot via endpoint `/auth/token` du
  launcher, qui lui lit le fichier)
- Tests : 20+ Vitest + pytest + cargo verifiant 401 sans token,
  200 avec token, rotation du token

### D2 — Unix Domain Sockets (Linux/Mac) + Named Pipes (Windows)

**Retenu** : en plus du TCP loopback avec bearer token, le daemon
et le coordinator exposent une seconde surface via UDS
(`~/.sbfb/run/daemon.sock`, `~/.sbfb/run/coordinator.sock`) sur
Unix et Named Pipes `\\.\pipe\sbfb-daemon` / `\\.\pipe\sbfb-coord`
sur Windows. Le shell React utilise le TCP (pas d'API UDS dans
un browser), mais les binaires Rust et le CLI `sbfb` utilisent
l'UDS quand dispo (plus strict).

**Rejete** :
- UDS uniquement : casse le browser qui ne parle qu'en TCP
- TCP sur `0.0.0.0` : anti-pattern securite

**Implications** :
- `crates/nexus-shell-daemon` +60 LOC : listener UDS en plus du
  TCP (config via env `SBFB_DAEMON_SOCKET`)
- `packages/nexus-coordinator` +40 LOC : idem
- `crates/nexus-launcher` +30 LOC : cree le repertoire
  `~/.sbfb/run/` avec perm `0700` au boot
- Feature flag `uds` dans `crates/nexus-shell-daemon-core`
  (no-op sur Windows si non-supporte)

### D3 — Consent screen + toggle "Partager GPU avec le reseau"

**Retenu** : nouveau composant React
`web/src/components/GpuConsentDialog.tsx` affiche au premier boot
(apres creation de la keypair) un dialog :
- Explication : "SBFB est un reseau P2P de compute. Tu peux
  choisir comment ton GPU est utilise."
- 3 options radio :
  - "Uniquement mes projets" (default, zero partage)
  - "Projets open source verifies" (accepte uniquement les apps
    avec `is_open_source: true` dans leur annonce P2P)
  - "Tous les projets publics" (opt-in complet)
- Cap configurable : W max, VRAM max, heures/jour
- Bouton "Enregistrer" persiste dans `~/.sbfb/consent.json`

Le worker (`crates/nexus-worker-core`) lit ce fichier au boot et
filtre les tasks entrantes selon le niveau de consentement. Le
badge "contribution" dans le shell indique le niveau actuel.

**Rejete** :
- Default "tout le reseau" : viol RGPD + anti F-Droid ethique
- Consent implicite "si tu installes c'est que tu acceptes" :
  idem

**Implications** :
- `web/src/components/GpuConsentDialog.tsx` +200 LOC
- `web/src/pages/Network.tsx` +30 LOC (badge + bouton "Modifier
  consentement")
- `crates/nexus-worker-core::allowlist` +80 LOC : charge
  consent.json, filtre tasks par flag is_open_source et
  visibility
- `packages/nexus-coordinator/src/nexus_coordinator/consent.py`
  +60 LOC : API `/consent/get` et `/consent/set`
- Tests : 15+ Vitest + cargo + pytest

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

### D5 — Documentation threat model + runtime isolation roadmap

**Retenu** : 2 nouveaux documents dans `docs/security/` :

1. `docs/security/THREAT_MODEL.md` — modele de menace formel
   (assets, adversaires, vecteurs, mitigations). Reprend
   l'analyse pre-kickoff : iframe sandbox A, deploy verifie B+,
   reseau iroh B+, loopback post-Sprint 16 A-, stockage cles C?,
   supply chain C.

2. `docs/security/RUNTIME_ISOLATION.md` — roadmap VM/namespace :
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
  architectural que D1-D2 posent (bearer token + UDS sont
  compatibles VM by design)

**Implications** :
- `docs/security/` : dossier cree
- `docs/security/THREAT_MODEL.md` ~400 LOC markdown
- `docs/security/RUNTIME_ISOLATION.md` ~300 LOC markdown
- Lien depuis `README.md` section nouvelle "Security"
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

## 9. Estimations LOC

| Phase | LOC estimee | Repartition |
|---|---|---|
| 0 — Audit S15 | ~300 (findings + fix eventuels) | findings doc + 0-3 fix commits |
| A — Bearer token | ~400 | 50 launcher + 30 daemon + 40 coord + 20 web + 260 tests |
| B — UDS/NP | ~450 | 60 daemon + 40 coord + 30 launcher + 320 tests |
| C — Consent + worker filter | ~600 | 200 dialog + 30 Network + 60 consent.py + 80 allowlist + 230 tests |
| D — PA v5 | ~250 | 30 core-rs + 5 deploy + 15 web + 200 tests |
| E — Docs | ~900 | 400 threat model + 300 runtime-isolation + 100 verif + 100 audit_plan |
| **Total** | **~2900** | |

---

## 10. Checkpoint de validation

Avant de passer au plan detaille, confirmer :

1. D1 (bearer token au boot par le launcher, perm 0600, header
   `X-SBFB-Token`) est valide
2. D2 (UDS Unix + Named Pipes Windows en supplement du TCP
   authentifie) est valide
3. D3 (consent screen 3 niveaux + worker filtering par
   `is_open_source`) est valide
4. D4 (ProjectAnnouncement v5 avec flag derive
   automatiquement par le coordinator, pas user-settable) est
   valide
5. D5 (`docs/security/THREAT_MODEL.md` + `RUNTIME_ISOLATION.md`
   ecrits ce sprint, auto-install VM differe Sprint 17+) est
   valide
6. L'ordre des phases (A bearer token → B UDS/NP → C consent
   → D PA v5 → E docs) est OK
7. Les scope cuts (auto-install WSL2, encryption keypair,
   CI security audit, rate limiting, audit externe → Sprint
   17+) sont acceptes
8. La decision de **ne pas** casser la backward compat (PA v4
   reste decodable, pas de migration forcee des daemons) est
   valide
