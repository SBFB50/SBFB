# Sprint 16 — Audit plan pour Sprint 17 Phase 0

**Ecrit** : 2026-04-14 (Phase E)
**Tip a auditer** : `10bbc63` (Phase D PA v5) + commit Phase E
docs
**Commit stack** : Phase 0 gate (`e99c06f` + `14ec51e`) + A-D
(`d7c265a`, `1cfde89`, `3247e88`, `10bbc63`) + Phase E docs

---

## Mode d'emploi pour la session fraiche

1. Lire dans l'ordre :
   - memory (`MEMORY.md`, `nexus_grid_pivot.md`,
     `sprint_audit_gate.md`)
   - `git log --oneline 4da0043..HEAD` (le range Sprint 16)
   - `.planning/active/sprint16_kickoff.md` (D1..D5 gelees)
   - `.planning/active/sprint16_plan.md`
   - `.planning/active/sprint16_verification.md`
   - **ce document**
2. **NE PAS lire** `docs/shell/PATTERNS.md` ni
   `docs/security/THREAT_MODEL.md` avant d'avoir forme une
   opinion track par track. Ces docs captent la narration
   livreur — l'audit doit challenger, pas confirmer.
3. Timebox suggere : **2-3h**.
4. Delivrable : `.planning/active/sprint16_audit_findings.md`
   (session fraiche qui audite produit le doc, meme layout
   que `sprint15_audit_findings.md`). Au demarrage Sprint 17,
   les 5 docs S16 + `sprint16_audit_findings.md` seront migres
   en `archive/v1.2/` via `git mv`.
5. Les commits fix eventuels (P0/P1) doivent atterrir avant
   le premier commit Sprint 17 Phase A. Format
   `fix(sprint16): <track>-P<n> — <short>`.

---

## Scope auditable

Sprint 16 livre **4 surfaces nouvelles + 1 bump format** :

| Surface | Phase | Fichiers principaux |
|---|---|---|
| Loopback auth middleware (bearer + Host + Origin) | A `d7c265a` | `launcher/src/auth.rs`, `shell-daemon-core/src/auth.rs`, `coord/auth.py`, `web/src/api/auth.ts` |
| Peer creds bypass (UDS Unix + Named Pipes Windows) | B `1cfde89` | `shell-daemon/src/uds_server.rs`, `shell-daemon/src/named_pipe_server.rs`, `coord/peer_creds.py` |
| Consent 4 niveaux + caps + watcher | C `3247e88` | `worker-core/src/consent.rs`, `coord/api/consent.py`, `web/GpuConsentDialog.tsx` |
| ProjectAnnouncement v5 + is_open_source | D `10bbc63` | `shell-daemon-core/src/publish.rs`, `coord/api/deploy.py`, `web/src/api/daemon.ts` |
| Docs security + roadmap VM | E | `docs/security/README.md`, `THREAT_MODEL.md`, `RUNTIME_ISOLATION.md` + updates CLAUDE.md, README.md, SPRINT_LOG.md, PATTERNS.md |

Chacune a son track dans ce plan (A-E + F, G) plus un track
meta (tests / scope cuts / docs coherence).

---

## Track A — Bearer + Host + Origin middleware (Phase A)

**Question centrale** : la triple validation est-elle reellement
appliquee a chaque requete non-/health, ou existe-t-il un
chemin qui court-circuite l'un des trois checks ?

**Methodes** :

1. Grep `crates/nexus-shell-daemon-core/src/auth.rs::auth_required`
   : verifier que les 3 checks (token, host, origin) s'executent
   en sequence et que **chaque** miss retourne une erreur 401/403.
2. Lire `crates/nexus-shell-daemon/src/http.rs` : identifier la
   liste des routes **exemptes** (normalement seulement
   `/health` + `/blob-serve/*`). Cross-checker avec la liste des
   routes mentionnees dans le kickoff §D1.
3. `grep -rn "Router::new\|\.route(" crates/nexus-shell-daemon*/src/`
   et verifier qu'aucune route business (/publish, /browse, etc.)
   n'est hors du scope middleware par oubli.
4. Cote FastAPI : `grep -rn "LoopbackAuthMiddleware\|@router"
   packages/nexus-coordinator/src/` : verifier que le middleware
   est applique globalement et non par-router. Verifier que
   l'ordre middleware dans `app.py` place CORS en outer (pour
   OPTIONS preflight) et auth en inner.
5. Tester manuellement :
   - `curl http://localhost:8080/app/gov/tabs` → **doit**
     retourner 401 (pas 200, pas 500)
   - `curl -H "X-SBFB-Token: INVALID" ...` → 401
   - `curl -H "X-SBFB-Token: $TOKEN" -H "Host: attacker.com" ...` → 403
   - `curl -H "X-SBFB-Token: $TOKEN" -H "Origin: evil.com" ...` → 403
   - `curl http://localhost:8080/health` → 200 **sans token**
6. Verifier la **timing side-channel** : le compare du bearer est-il
   constant-time (`constant_time_eq`, `subtle::ConstantTimeEq`) ou
   bien `==`-string ? Un `==` permet de faire du timing attack bit-
   par-bit.
7. Verifier que le `auth_token` file est lu **une fois** au boot
   (pas re-lu a chaque requete — trop de IO) mais que ce cache
   invalide correctement si le fichier change (via le launcher
   qui regenerate). Regression : l'utilisateur ne peut plus se
   connecter apres rotation manuelle.

**Signal** :

- **P0** : une route business repond 200 sans bearer, ou un
  header malforme casse le serveur (panic/500)
- **P0** : comparaison bearer non-constant-time (timing leak
  exploitable en local avec clock precision)
- **P1** : une route **autre** que `/health` / `/blob-serve/*`
  est exemptee sans commentaire justificatif
- **P1** : l'ordre middleware FastAPI est inverse (auth avant
  CORS → preflight casse)
- **P2** : pas de test dedie pour le cas "header X-SBFB-Token
  present mais vide" (tronque)
- **P3** : nit sur le wording du 401 ("missing or invalid")

---

## Track B — UDS + Named Pipes peer creds (Phase B)

**Question centrale** : sur **Windows**, le DACL custom SDDL
bloque-t-il effectivement un autre user, et le `PeerCredsVerified`
marker est-il rigoureusement non-spoofable cross-OS ?

**Methodes** :

1. **UDS Unix** : lire `crates/nexus-shell-daemon/src/uds_server.rs` :
   - `getsockopt(SOL_SOCKET, SO_PEERCRED)` sur Linux → uid
     retourne dans `ucred.uid`
   - `getpeereid` sur macOS / *BSD → idem
   - Verifier que l'accept loop rejette **en fermant le socket**
     (pas en loggant puis servant) si uid != geteuid()
2. **Named Pipe Windows** : lire
   `crates/nexus-shell-daemon/src/named_pipe_server.rs` :
   - `ConvertSidToStringSidW` + SDDL construction
   - **CONSUME** un autre SID → le pipe doit refuser le
     `CreateFile` coteclient (ACCESS_DENIED 5)
   - Verifier qu'il n'y a PAS de retry transparent sur accept
     fail (un attacker peut log-flooder)
3. `PeerCredsVerified` bypass : relire
   `shell-daemon-core/src/auth.rs:293` :
   - C'est un **type privé** (`pub struct PeerCredsVerified;`
     dans un module qui expose *pas* de constructeur public ? Ou
     bien la struct est pub mais le champ de construction
     uniquement interne) — verifier le module boundary
   - Dans `auth_required`, le check du marker se fait via
     `request.extensions().get::<PeerCredsVerified>()`. Un
     attacker ne peut pas injecter une extension par header.
   - Verifier les tests qui essaient justement ce spoof (header
     `X-SBFB-PeerCreds: true` → doit rester 401).
4. **Cross-OS test** : tourner `cargo test -p nexus-shell-daemon
   --target x86_64-pc-windows-msvc` si la machine est Windows ;
   inversement sur Linux. Les tests `cfg(unix)` / `cfg(windows)`
   doivent pas etre skipped en silence.
5. Coord UDS : verifier que le second uvicorn UDS (Unix) n'a PAS
   encore le bypass ASGI (documenter comme scope cut, check que
   le TODO comment cite Sprint 17+).

**Signal** :

- **P0** : PeerCredsVerified peut etre injecte depuis un header
  ou un cookie (reproducible via integration test)
- **P0** : Named Pipe SDDL oublie de restreindre `Everyone` et
  autorise tous les processes connectes (test sur Windows :
  `sudo -u other` → 200)
- **P1** : UDS rejette mais ne ferme pas le fd (leak)
- **P1** : le chmod 0600 post-bind UDS est absent cote coord
- **P2** : manque un test `PeerCredsVerified::new()` public
- **P3** : wording des logs d'erreur

---

## Track C — Consent dialog + caps + watcher (Phase C)

**Question centrale** : le worker enforce-t-il reellement les
caps **avant** d'appeler Ollama, et le watcher est-il robuste
aux editeurs qui font write+rename ?

**Methodes** :

1. Lire `crates/nexus-worker-core/src/consent.rs` :
   - `should_accept_task` — verifier l'ordre des checks :
     level **avant** caps (sinon caps seuls = level-agnostic)
   - `UsageTracker::hours_used_today` — verifier que le reset
     sur midnight-local utilise bien `chrono::Local` et pas
     `Utc`
   - `ConsentWatcher::spawn` — verifier que le debounce 50 ms
     tient contre un `write+rename` (rename emit 2 events
     `notify`)
   - Check que le `RwLock` est en read lock pendant
     `should_accept_task` et en write lock uniquement pendant
     le reload
2. Lire `crates/nexus-worker-core/src/engine/runtime.rs` :
   - Ou est place l'appel `should_accept_task` dans le claim
     loop ? **Avant** le `verify_signature` ou **apres** ?
     Doit etre apres signature verify (sinon on perd le drop
     sur task forgee).
   - Apres verify + consent accept, est-ce que le worker
     enregistre le `reserve_hours(estimate)` avant de lancer
     Ollama (pour tenir le cumul en cas de crash Ollama) ?
3. Lire `packages/nexus-coordinator/src/nexus_coordinator/api/consent.py` :
   - Atomic write via `tmp + rename` — verifier que le fd
     `tmp` est bien fsynced avant rename (sinon crash kernel =
     perte)
   - `POST /consent/whitelist/add` : le resolver repo_url →
     node_id retourne 422 quand non-resolu. Verifier que le
     test couvre ce chemin.
4. Lire `web/src/components/GpuConsentDialog.tsx` :
   - Aucune radio pre-cochee au 1er ouvre (GDPR Art.25)
   - Le slider heures/jour = 0 doit signifier "pas de task
     aujourd'hui" (pas "no cap" / illimite)
   - Input node_id whitelist valide hex 64 chars (grep
     `/^[0-9a-f]{64}$/i` ou similaire)
5. Tester manuellement :
   - Start worker avec consent L1 + submit task projet tiers
     → **rejected** avec raison `NotOwnProject`
   - Consent L2 + PA v5 `is_open_source=false` → **rejected**
     avec `NotOpenSource`
   - Consent L3 + whitelist vide + task → **rejected**
   - Consent L4 + cap_watts=20 + task.watts_estimate=50 →
     **rejected** `CapWatts`
   - Reecrire `consent.json` direct sur disque → worker reload
     en <100 ms (sans redemarrer)

**Signal** :

- **P0** : `should_accept_task` accepte une task quand le level
  devrait refuser (bug logic bypass)
- **P0** : watcher perd l'update si l'ecriture est write+rename
  (reproducible avec `vim`)
- **P1** : caps UI uniquement, pas enforce worker (test
  manquant)
- **P1** : midnight reset utilise Utc au lieu de Local (DST
  bugs)
- **P2** : whitelist input accepte un node_id non-hex 64
- **P3** : wording des rejection reasons dans les logs

---

## Track D — ProjectAnnouncement v5 (Phase D)

**Question centrale** : le flag `is_open_source` est-il
rigoureusement **derive** (et pas user-settable) cote coord, et
le decoder accepte-t-il les v4 legacy sans crash ?

**Methodes** :

1. Lire `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py` :
   - `deploy_from_repo` → `_publish_with_archive(..., is_open_source=True)`
   - `deploy` (zip prive) → `_publish_with_archive(..., is_open_source=False)`
   - Aucun chemin ou un client peut injecter `is_open_source`
     via body JSON et le voir propager
2. Lire `crates/nexus-shell-daemon-core/src/publish.rs` :
   - `from_gossip_bytes` → `if ann.v == 0 || ann.v >
     PROJECT_ANNOUNCEMENT_VERSION { return Err }` : verifier
     que v4 (= 4) passe, et que les versions 1..5 sont toutes
     acceptees
   - `is_open_source` serde default false — verifier via un
     test qui serialize une v4 sans le champ et decode en v5
3. `web/src/api/daemon.ts::BrowseEntrySchema` :
   - `z.boolean().optional()` : distinguer `undefined` (legacy
     v4 inconnu) de `false` (explicitement prive)
   - Grep que l'UI n'affiche **pas** un badge "Proprietary"
     sur une entry `is_open_source === undefined` (c'est une
     entry legacy, pas une info sure)
4. Test round-trip : encode v5 avec flag true → bytes → decode
   → flag true. Encode v5 avec flag false → decode → flag false.
   Legacy v4 bytes → decode v5 → flag false.
5. **Test manuel prod** : deploy-from-repo un repo public →
   recup la PA sur un autre noeud → verifier via
   `nexus-shell-daemon-core` qu'elle a bien
   `is_open_source=true`.

**Signal** :

- **P0** : un client peut faire `POST /project/deploy` avec
  `{"is_open_source": true, ...}` (body JSON) et voir la
  PA sortir avec `true` (non-user-settable viole)
- **P0** : decoder rejette v4 legacy (regression)
- **P1** : `BrowseEntry::is_open_source=None` affiche un
  badge "proprietaire" cote UI (faux positif legacy)
- **P2** : `deny_unknown_fields` active sur le schema Rust
  → impossible d'ajouter un champ sans bumper a nouveau
- **P3** : nit sur le naming

---

## Track E — Docs security + runtime isolation (Phase E)

**Question centrale** : les mitigations citees dans
`THREAT_MODEL.md` §7 correspondent-elles strictement au code
livre A-D, ou y a-t-il du wish-list ?

**Methodes** :

1. Pour chaque row de `THREAT_MODEL.md` §7 status "LIVRE S16X" :
   - Verifier que le commit SHA cite match un commit du range
     `d7c265a..10bbc63`
   - Verifier que le fichier cite existe et contient
     effectivement la logique decrite
   - Si une LOC range est donnee (ex: `auth.rs:274-383`),
     spot-check qu'elle couvre le code annonce
2. Pour chaque row status "DIFFERE S17+" : verifier que la
   description est factuelle (pas de "sera tres facile a faire"
   sans effort estime)
3. DFD §4 : verifier que chaque composant mentionne dans le
   texte (`/auth/token`, `blob-serve:7000`, `UDS`, `:7777`,
   `:8080`) est bien dans le diagramme. Pas de flux fantome.
4. STRIDE §5 : pour chaque severite "res L" (residuelle faible),
   verifier que la mitigation listee est deployee en prod, pas
   juste ecrite dans un test
5. LINDDUN §6 : le mapping GDPR est-il conforme au texte reel
   du RGPD ? Un reviewer externe ne doit pas etre capable de
   dire "Art.7(3) parle de la possibilite de retirer, pas de
   l'UX du retirement".
6. `RUNTIME_ISOLATION.md` §2.1 : `wsl --install --no-distribution`
   + `wsl --install -d Ubuntu-24.04` — verifier que la syntax
   est correcte (la doc MS a change sur WSL2 2024). Si erreur
   de commande, P1.
7. Cross-reference : chaque mention de commit hash dans ces
   docs doit matcher une ligne de `git log --oneline`. Grep
   tous les `\`[a-f0-9]{7,8}\`` et checker.

**Signal** :

- **P0** : un commit SHA cite n'existe pas dans le repo
- **P0** : une mitigation "LIVRE S16" correspond a du code
  inexistant ou a un commit d'un sprint precedent
- **P1** : une commande shell dans `RUNTIME_ISOLATION.md` est
  syntaxiquement fausse
- **P1** : le mapping GDPR oublie un article applicable
  evident (ex: Art.33 breach notification)
- **P2** : une row de matrice severite est inconsistante
  (brute=H, res=L, mais la mitigation est juste "config par
  default 0600")
- **P3** : typos, wording, ordering alphabetique ou pas

---

## Track F — Backward compat consent + PA (cross-sprint)

**Question centrale** : un noeud v1.1 (avant Sprint 16) peut-il
continuer a fonctionner sur le reseau avec des noeuds v1.2, et
inversement ?

**Methodes** :

1. PA v5 backward compat :
   - Un noeud v1.1 recoit une PA v5 : decoder v4 rejette v5
     (T49 scope legacy) OU le bump v4→v5 est purement additif
     serde ? Verifier.
   - Un noeud v1.2 recoit une PA v4 : decoder accepte +
     default false.
2. Bearer token backward compat :
   - Un daemon v1.1 (sans bearer) tourne vers un coord v1.2 ?
     Non applicable, les deux composants sont du meme install.
   - Un shell React v1.1 (sans `authFetch`) vers coord v1.2 ?
     Les fetch sans bearer → 401 → shell casse. Le kickoff
     dit "redemarrer daemon + coord apres upgrade" : verifier
     que cette instruction est dans le CHANGELOG / README
     upgrade section.
3. Consent backward compat :
   - Un worker v1.1 (sans consent module) tourne vers une task
     assignee v1.2 ? Pas de probleme : le worker v1.1 claim
     tout ce qu'il peut, il n'a simplement pas le filter.
     Mais si l'utilisateur a set consent L1 sur son noeud v1.2,
     il ne peut pas downgrade sans perdre la config.
4. UDS / Named Pipes : TCP reste le chemin principal, donc
   compat.

**Signal** :

- **P0** : noeud v1.1 crash sur PA v5 (regression T49)
- **P1** : instructions d'upgrade absentes ou fausses
- **P2** : downgrade v1.2 → v1.1 perte de consent.json silent

---

## Track G — Tests coverage + scope cuts

**Question centrale** : les tests livres couvrent-ils les
scenarios **critiques** du kickoff, et les scope cuts sont-ils
tous respectes ?

**Methodes** :

1. Compteur tests attendu : 421 Rust / 187 coord / 240 vitest /
   38 Playwright. Re-runner :
   - `cargo test --workspace --locked | grep "test result" |
     awk '{sum+=$4} END{print sum}'` — doit etre >= 421 + 5
     doc-tests = 426
   - `uv run pytest packages/nexus-coordinator/tests/ -q`
     → 187 passed + 1 skipped
   - `cd web && npm run test:unit` → 240 passed
   - `npx playwright test` → 38 passed
2. Scope cuts kickoff §6 : une par une, verifier qu'aucune n'a
   ete livre "par erreur". Grep les fichiers nouveaux :
   - `cargo-audit` / `pip-audit` / `npm audit` workflows →
     aucun `.github/workflows/audit*.yml`
   - Rate limiting → pas de crate `governor` ou
     `slowapi` ajoute
   - CSP report-uri → aucun endpoint `/security/csp-report`
   - Token rotation → pas de scheduled task / cron
   - MIME scan → pas de `libmagic` / `python-magic`
3. Verifier que les LOC estimees kickoff §9 matchent la realite
   a +/- 20% (pas un bug si delta ; juste un signal si le plan
   a sous-estime/sur-estime massivement).
4. Tests critiques MUST-HAVE :
   - test bearer 401 sans header
   - test Host attacker.com 403
   - test Origin evil.com 403
   - test `/health` sans auth 200
   - test `PeerCredsVerified` non-spoofable
   - test `should_accept_task` L1/L2/L3/L4 + caps
   - test watcher reload
   - test PA v4 legacy decode
   - test PA v5 encode + decode flag true / false
5. Verifier `.github/workflows/` CI : les runs ont-ils le meme
   scope que le kickoff fail-fast ? Si le kickoff dit "cargo
   test --locked" et la CI run "cargo test --all", il peut y
   avoir un gap sur les features.

**Signal** :

- **P0** : un scope cut a ete livre (= le kickoff est menti)
- **P0** : un test MUST-HAVE est absent
- **P1** : compteur tests total bas de > 5% par rapport a
  l'annonce
- **P1** : CI run differente du fail-fast local (risque de
  green local / red prod)
- **P2** : tests nominaux sans tests negatifs
- **P3** : commentaires tests incomplets

---

## Verdict global attendu

Les Sprints 14 et 15 ont donne PASS et PASS-a-gate-leve. Sprint
16 a livre beaucoup de surface nouvelle (~3200 LOC + tests) ; un
CONDITIONAL PASS avec 2-3 P1 sur des points subtils (timing-safe
compare, DACL edge case, watcher TOCTOU) est **plausible**.

| Scenario | Condition | Action |
|---|---|---|
| **PASS** | 0 P0, 0 P1 | Sprint 17 Phase A demarre direct |
| **CONDITIONAL PASS** | 1-3 P1 fixables | Sprint 17 Phase A bloque tant que `fix(sprint16): ...` pas landed |
| **FAIL** | >= 1 P0 OU >= 3 P1 critiques | Re-conception partielle, possiblement re-opener Sprint 16 avec un commit stack de fix, ou ouvrir Sprint 17 autour du rework |

---

## Out of scope pour l'audit

L'auditeur ne doit **pas** rebattre :

- **D1..D5** du kickoff Sprint 16 (deja validees post-
  recherche le 2026-04-14 : Tailscale safesocket pattern,
  Syncthing/Jupyter token pattern, BOINC UserOptInConsent,
  CVE-2025-49596, Windows Named Pipe DACL SDDL)
- Les scope cuts (§6 kickoff). Si un item est differe, il est
  differe — pas de "pourquoi pas maintenant"
- Le choix PyO3 bindings Python (decision Sprint 4+)
- Le choix iroh 0.97 pin (decision Sprint 3)
- Le choix PARA layout (decision Sprint 16 Phase 0,
  non-rebattable)
- Le pattern audit gate lui-meme (pattern permanent depuis
  Sprint 7)

Si l'auditeur **pense** qu'une D doit bouger, il l'enregistre
comme P3 "discussion Sprint 17 kickoff" mais ne bloque pas le
gate.

---

## Livrable final attendu

`sprint16_audit_findings.md` doit contenir :

1. **En-tete** : date audit, tip auditee, timebox consomme
2. **Resume executif** : verdict global + top 3 findings
3. **Track A..G** : pour chaque track :
   - Methode effectivement roulee (commandes)
   - Findings P0/P1/P2/P3
   - Pour chaque P0/P1 : reproducer (commande shell) + fix
     suggere + LOC estime
4. **Scope cuts audit** : table verifiee, tous respectes
   (ou liste des items leaked)
5. **Tests coverage** : counts observes vs attendus
6. **Verdict final** : PASS / CONDITIONAL PASS / FAIL
7. **Commits fix proposes** : liste ordonnee avec titles
   `fix(sprint16): <track>-P<n> — <short>`

Format identique a `sprint15_audit_findings.md`
(→ `archive/v1.1/sprint15_audit_findings.md`) pour le parsing
cross-sprint et les reports de debt dans `PATTERNS.md`.

---

## Criteres de cloture Phase 0 Sprint 17

- [ ] `sprint16_audit_findings.md` ecrit et commite
      (`docs(sprint16): audit findings from Sprint 17 Phase 0 gate`)
- [ ] Verdict global clair (PASS / CONDITIONAL / FAIL)
- [ ] Si CONDITIONAL : commits `fix(sprint16): ...` landed
      pour tous les P1 avant Phase A
- [ ] Les 6 docs Sprint 16 (kickoff + plan + verification +
      audit_plan + audit_findings + security) restent dans
      `active/` jusqu'au premier commit Sprint 17 Phase A,
      puis migration `git mv` vers `archive/v1.2/`
- [ ] `docs/claude/SPRINT_LOG.md` row S16 mise a jour avec le
      tip post-gate si CONDITIONAL PASS leve
