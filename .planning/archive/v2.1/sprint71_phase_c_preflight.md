# Sprint 71 Phase C Preflight

Date: 2026-05-30
HEAD: `0daff81`
Verdict: **SCOPE-CUT-CONSISTENT**

> Resume executif : les 4 corrections securite Factory (G2 gate SSE, G9
> modele opus-4-8, G7 token+Host+CORS, G12 timeout subprocess) sont
> toutes **APPROACH-ALIGNED** avec l'etat de l'art OSS ET avec un pattern
> deja livre **dans ce repo** (le daemon loopback durci S16). Aucun
> finding S1b/S2/S3/S4 bloquant → pas de DESIGN-CONFLICT. Aucun S1a
> APPROACH-NAIVE/LIB-EXISTS → pas de PLAN-ADAPT. Le verdict est
> **SCOPE-CUT-CONSISTENT** : quatre findings non bloquants d'**execution**
> (pas de design) precisent le plan §7 et doivent etre traces au commit :
>
> 1. **(R6 affine — decisif)** Le front Operator reel
>    (`tools/factory-operator`, Vite :5174) parle au serveur `:3001`
>    **via un proxy Vite server-to-server**, pas en cross-origin browser.
>    Le `allow_origin(Any)` n'est donc JAMAIS exerce par le front
>    legitime → **restreindre le CORS ne casse pas le front** (R6
>    largement de-risque cote browser). La complication reelle de R6 est
>    la **livraison du token** : les requetes browser sont same-origin
>    proxifiees, le browser ne lit pas `~/.sbfb`. Le token doit etre
>    injecte cote proxy Vite (server-side) OU le serveur n'exige le token
>    QUE sur les routes mutantes/spawn, en s'appuyant sur Host+Origin
>    loopback pour les GET de lecture. Decision d'implementation a acter
>    en Phase C (options dans S3).
> 2. **(R5 affine — decisif)** Le front reel **ne consomme PAS** le SSE
>    `/chat/{id}/stream` : `AgentChat.tsx` a bascule sur un terminal
>    xterm WebSocket (`/api/terminal/ws`) + `fetch('/api/prompt/...')`.
>    Le SSE chat-stream est **orphelin cote front** (remplace par
>    l'embedded terminal `c3f4813`/`0aa06db`). Gater le SSE (D3) ne casse
>    donc aucun chemin front vivant — R5 fortement de-risque. Le gate
>    reste correct a poser (defense serveur ; un appel API direct ou un
>    futur re-cablage du SSE doit etre gate).
> 3. **(tests — concret)** Les ~24 tests `operator_server.rs` existants
>    appellent le serveur **sans token**. Rendre le token OBLIGATOIRE
>    casserait ces tests. Le harness `TestServer` doit etre mis a jour
>    dans le MEME commit (lire le token genere via stdout/fichier, l'
>    injecter). C'est l'analogue tests de R6.
> 4. **(dep — mineur)** Le pattern token/Host de reference vit dans
>    `nexus-shell-daemon-core::auth` (`is_loopback_host`,
>    `is_loopback_origin`, `AUTH_HEADER`), que `sbfb-factory` ne depend
>    PAS aujourd'hui. D5 doit choisir : (a) ajouter la dep
>    `nexus-shell-daemon-core`, (b) lifter les 2 predicats vers
>    `nexus-core-rs`, ou (c) reimplementer localement (predicats triviaux,
>    ~30 lignes). Choix d'implementation, pas un blocker.

## Evidence Rules
- Claim policy : chaque affirmation cite un chemin:ligne, une sortie de
  commande, une URL datee, ou une hypothese explicite.
- Local sources read :
  - `prompts/agent/preflight.md` (procedure portable, integrale)
  - `.planning/active/sprint71_plan.md §7 (Phase C, l.260-329)`, `§10`, `§12`, `§13`
  - `.planning/active/sprint71_kickoff.md §5 (D3/D4/D5/D6 + acknowledged D5 adjust)`, `§10 (R5/R6)`
  - `.planning/active/sprint71_phase_a_preflight.md` + `_phase_b_preflight.md` (format/profondeur, continuite tip)
  - `.planning/active/sprint70_audit_findings.md` (G2/G7/G9/G12 origine)
  - `crates/sbfb-factory/src/operator_server.rs` (integral, 908 lignes)
  - `crates/sbfb-factory/src/llm_bridge.rs` (integral, 255 lignes)
  - `crates/sbfb-factory/src/daemon_client.rs` (pattern token+Host, l.64-65)
  - `crates/sbfb-factory/src/process.rs:24-34` (PROVIDERS, distinction documentee S71 B)
  - `crates/sbfb-factory/src/main.rs` (OperatorCommand::Serve, runtime)
  - `crates/sbfb-factory/tests/operator_server.rs` (integral, 456 lignes, 24 tests sans token)
  - `crates/sbfb-factory/Cargo.toml` (deps Phase C : axum/tower-http/tokio/futures/async-stream deja au lock)
  - `crates/nexus-shell-daemon/src/http.rs:509-526` (cors_layer reference — AllowOrigin::predicate)
  - `crates/nexus-shell-daemon-core/src/auth.rs:197-285` (token + is_loopback_host/origin reference)
  - `tools/factory-operator/src/pages/AgentChat.tsx` (front reel — WS terminal, PAS SSE)
  - `tools/factory-operator/src/hooks/useApi.ts` (fetch same-origin `/api`)
  - `tools/factory-operator/vite.config.ts` (proxy :5174 → :3001)
  - `tools/factory-ui/src/operator/api-client.ts` (front orphelin G10 — BASE_URL :3001, sans token)
  - `docs/agent/RRV_FACTORY_CONTRACT.md §4` (contrat a amender)
  - `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (modele 3-tiers, origine S16 `d7c265a`)
  - `docs/security/THREAT_MODEL.md §3,§5,§7,§8 (AD1-AD5, STRIDE, mitigations, R1)`
- Commands run (extraits pertinents inline dans chaque scan).

## Scope
- Plan source : `.planning/active/sprint71_plan.md §7 (Phase C, l.260-329)`.
- Target files (plan §7 C.2) :
  - `crates/sbfb-factory/src/operator_server.rs` — G2 gate SSE (l.735-796), G9 modele opus-4-8 (l.776 + cabler `ChatSendRequest.model` l.665), G7 CorsLayer restreint + middleware token + Host guard (l.87-90, build_router l.80-123).
  - `crates/sbfb-factory/src/llm_bridge.rs` — G12 timeout subprocess + pre-spawn resolution check + diagnostic (l.64-118).
  - `crates/sbfb-factory/src/daemon_client.rs` — reference pattern token+Host (l.64-65), **lecture seule**.
  - `docs/agent/RRV_FACTORY_CONTRACT.md §4` — amendement PO-2 (pilotage agent local privilegie **gate** explicitement autorise).
  - `tools/factory-operator/` (front Operator) — **AJOUT vs plan §7** : le plan dit `web/` mais le front Operator vit dans `tools/factory-operator/`, pas `web/`. Token bootstrap proxy si token obligatoire (voir S3 R6).
  - `crates/sbfb-factory/tests/operator_server.rs` — **AJOUT vs plan §7** : le harness `TestServer` doit etre token-aware si le token devient obligatoire (meme commit).
- Deps/APIs/specs touches : **aucune nouvelle dep**. axum 0.8.9, tower-http 0.6.10 (feature `cors` deja activee, Cargo.toml workspace l.156), tokio 1.52.3 (feature `full` → `tokio::time::timeout` dispo), futures 0.3.32, async-stream 0.3.6 — **toutes au lock** (`Cargo.lock`). Les 3 deps off-sprint (`portable-pty`/`async-stream`/`futures`) ont DEJA ete passees au scan CVE en Phase B (`sprint71_phase_b_preflight.md §S1b`) — non re-scannees ici (aucune nouvelle dep Phase C).
- Security/protocol surfaces : serveur HTTP loopback `:3001` qui **ecrit des fichiers** (`/api/artifacts/draft`) et **spawn des process** (`/api/chat/{id}/stream` → `spawn_claude_stream` bypassPermissions ; `/api/terminal/ws` → PTY). CORS, Host guard, token bearer, gate d'action sensible, timeout subprocess. **PAS de canonical bytes, PAS de `*_VERSION` wire, PAS de signature** (voir S4).
- Tests expected (plan §7 C.3) : `sse_gates_sensitive_action`, `sse_allows_nonsensitive`, `chat_stream_uses_opus_model`, `server_rejects_missing_token`, `server_rejects_foreign_host`, `cors_restricts_origin`, `spawn_times_out`, `missing_claude_diagnostic`.

---

## S1a OSS Prior Art

- **Domain** : durcissement d'un serveur HTTP **loopback privilegie** (ecrit
  des fichiers + spawn des agents/process) — 4 sous-problemes :
  (a) auth bearer token + Host guard sur loopback, (b) CORS hardening
  axum/tower-http, (c) timeout/kill de subprocess tokio, (d) gating de
  messages avant spawn d'agent autonome. Familles de reference :
  microsoft/sudo (trust tiers, deja cite par le projet), MCP servers
  localhost (CVE-2025-49596 / CVE-2025-66414), OWASP CSRF cheat sheet,
  tokio process docs.

- **Sources (consultees 2026-05-30)** :
  - **tower-http `CorsLayer` / `AllowOrigin`** — context7 `/websites/rs_tower-http_tower_http`
    (docs.rs) : `AllowOrigin::exact(HeaderValue)` (origine unique),
    `AllowOrigin::predicate(|origin, parts| ...)` (predicat dynamique),
    `CorsLayer::allow_methods([Method::GET, Method::POST])` (methodes
    explicites vs `Any`). => l'API pour remplacer `allow_origin(Any)` par
    une restriction d'origine **existe et est documentee**.
  - **tokio process timeout/kill** — WebSearch + docs.rs
    `tokio::process::Child` : pattern recommande = `tokio::time::timeout`
    OU `select!` + oneshot ; apres `kill()`, **await le child** pour
    eviter le zombie Unix ; `Command::kill_on_drop(true)` comme filet ;
    SIGTERM-puis-SIGKILL pour un kill gracieux (optionnel).
    https://docs.rs/tokio/latest/tokio/process/struct.Child.html
    https://github.com/tokio-rs/tokio/discussions/7132
  - **DNS rebinding / CSRF localhost** — WebSearch (etat 2025-2026) :
    OWASP CSRF Prevention Cheat Sheet ; GitHub Security Blog "Localhost
    dangers: CORS and DNS rebinding" ; Oligo "0.0.0.0 Day" + CVE-2025-49596
    (Anthropic MCP Inspector RCE via localhost browser) ; CVE-2025-66414
    (MCP TypeScript SDK, DNS rebinding, dec 2025) ; CVE-2025-14279 (MLFlow
    DNS rebinding CSRF — absence de validation Origin). **Consensus : tout
    service sur 127.0.0.1 doit supposer un acces reseau hostile et exiger
    auth + validation Host + verification Origin.**
    https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html
    https://github.blog/security/application-security/localhost-dangers-cors-and-dns-rebinding/
    https://www.oligo.security/blog/0-0-0-0-day-exploiting-localhost-apis-from-the-browser

- **Reference INTERNE (la plus forte)** : le pattern cible est **deja
  livre dans ce repo** depuis S16 (`d7c265a`) :
  - `crates/nexus-shell-daemon/src/http.rs:511-526` `cors_layer()` :
    `CorsLayer::new().allow_origin(AllowOrigin::predicate(|origin, _| is_loopback_origin(origin)))`
    — exactement le remplacement de `Any` que D5 demande.
  - `crates/nexus-shell-daemon-core/src/auth.rs:47` `AUTH_HEADER = "x-sbfb-token"`,
    `:239` `is_loopback_host`, `:274` `is_loopback_origin`, `:197-214`
    lecture/validation token 256-bit hex, perm 0600 (Unix).
  - `crates/sbfb-factory/src/daemon_client.rs:64-65` consomme deja
    `X-SBFB-Token` + `Host: 127.0.0.1` cote client Factory.
  - `crates/sbfb-factory/src/terminal.rs:202` `child.kill()` — un kill de
    process existe deja cote PTY Factory (reference pour G12).

- **Finding** :
  - **G7 (D5) auth+Host+CORS** : **APPROACH-ALIGNED** (OSS + reference
    interne `http.rs:511` / `auth.rs`). Aucune lib externe a ajouter — le
    pattern est in-repo, reutilisable.
  - **G12 (D6) timeout subprocess** : **APPROACH-ALIGNED** (tokio
    `timeout` + await-after-kill, pattern docs.rs ; le kickoff D6 rejette
    explicitement la dep `which` au profit d'un check pre-spawn —
    coherent avec l'evidence "pas de dep necessaire").
  - **G2 (D3) gate avant spawn** : **APPROACH-ALIGNED** — le filtre
    `SENSITIVE_ACTIONS` existe deja sur `handle_chat_message` (l.606) et
    `handle_chat_send` (l.687) ; le SSE (l.735-796) est le seul chemin
    qui le court-circuite. Propager un filtre existant = aligne.
  - **G9 (D4) modele** : **APPROACH-ALIGNED** (config, pas de design).

- **Impact** : **aucune adaptation requise** (pas de PLAN-ADAPT). S1a ne
  produit ni APPROACH-NAIVE ni LIB-EXISTS — le besoin est couvert par un
  pattern interne deja eprouve, pas par une lib externe a importer.

---

## S1b Dependencies, CVEs, Release Notes

- **Scanned** : axum 0.8.9, tower-http 0.6.10 (feature `cors`), tokio
  1.52.3 (feature `full`), futures 0.3.32, async-stream 0.3.6.
- **Commande** :
  `rg 'name = "(axum|tower-http|tokio|portable-pty|async-stream|futures)"' Cargo.lock -A1`
  → axum 0.8.9 / tower-http 0.6.10 / tokio 1.52.3 / portable-pty 0.9.0 /
  async-stream 0.3.6 / futures 0.3.32. **Aucune nouvelle dep Phase C.**
- **Evaluation** :
  - **Aucune dep ajoutee** : Phase C reutilise axum/tower-http/tokio/
    futures/async-stream **deja au lock**, deja compiles, deja utilises
    par `operator_server.rs` (l.8-16) et `llm_bridge.rs` (l.6-9). Le scan
    CVE S1b est donc un **no-op de nouvelle dependance**.
  - Les 3 deps off-sprint (G13 : portable-pty 0.9.0, async-stream 0.3.6,
    futures 0.3.32) ont DEJA ete passees au scan CVE en **Phase B**
    (`sprint71_phase_b_preflight.md §S1b` : aucune CVE critique/high sur
    crypto/wire/network ; portable-pty = spawn local garde). G13
    satisfait. Pas de re-scan ici (regle du brief : ne re-scanner que si
    Phase C ajoute une dep — elle n'en ajoute pas).
  - axum 0.8 / tower-http 0.6 : versions stables, pas de breaking release
    sur l'API CORS/SSE/middleware que la phase utilise (l'API
    `AllowOrigin::predicate`/`exact` + `axum::middleware::from_fn` est
    stable dans ces lignees ; cf. context7 docs.rs).
- **Finding S1b** : **clean (non-bloquant)**. Aucune nouvelle dep, aucune
  CVE critique/high sur crypto/wire/network/sandbox/signing, aucun
  breaking major sur une API que la phase utilise.

---

## S2 Historical Decisions

- **Commandes** :
  - `git log --all --oneline -- operator_server.rs llm_bridge.rs daemon_client.rs`
    → bloc off-sprint : `e26d9f2` (wire SSE), `eb06c35` (bypassPermissions),
    `0aa06db`/`864b005`/`c3f4813` (terminal), `35ec331`/`886eed0` (spawn
    fixes), `69e3a06` (S70 Phase D Operator serve — origine CORS Any).
  - `git log --all --grep='DEVIATION|rejected|scope-cut|threat-model|bypassPermissions|CORS|token' -- crates/sbfb-factory/`
  - `git show e26d9f2/eb06c35/69e3a06 --no-patch --format=%B`
  - reverse-commit : recherche d'un re-alignement post-introduction.

- **Decisions traversees** :

  1. **CORS Any — S70 Phase D `69e3a06` (cycle complet : preflight
     EXECUTE + review PASS + Codex)**. Le body documente explicitement,
     section **`## Scope cuts`** :
     *"CORS Any (localhost) : acceptable Phase D, durcir Phase F si
     surface persiste."* → C'est un **deferral documente et anticipe par
     l'auteur lui-meme**, PAS une decision gelee de garder `Any`. La
     "surface persiste" condition est **realisee** : le bloc off-sprint a
     ajoute le SSE spawn (`e26d9f2`) et le terminal PTY (`c3f4813`) sur le
     meme serveur. D5 (durcir CORS) **execute** ce deferral, il ne le
     contredit pas. **Reversion CONFIRMEE / deferral honore → non-bloquant.**
     (Note : "Phase F" du S70 n'a jamais durci car le bloc est parti
     off-sprint sans cycle — c'est precisement la dette que S71 absorbe.)

  2. **Gate SENSITIVE_ACTIONS "preserve" — SSE `e26d9f2`**. Le body
     `e26d9f2` AFFIRME : *"Sensitive action gate preserved: messages
     containing 'shell', 'commit', 'push', 'PASS' are blocked before
     spawning any subprocess."* MAIS le code livre par ce commit + le
     suivant (`eb06c35`) montre que le SSE `handle_chat_stream`
     (l.735-796) **n'a jamais porte ce filtre** (seuls
     `handle_chat_message`/`handle_chat_send` l'ont). L'auteur **croyait**
     le gate preserve ; il ne l'etait pas sur le chemin SSE. C'est le
     **bug G2**, pas une decision deliberee de NE PAS gater le SSE.
     Aucune evidence d'une decision "le SSE doit rester non gate".
     **Reversion ambigue d'une intention declaree → D3 ferme le bug,
     non-bloquant.** Conforme au classement S2 : "no valid rationale pour
     l'etat actuel" → D3 (gater) aligne sur l'intention declaree.

  3. **bypassPermissions delibere — `eb06c35`**. Le body : *"changed
     --permission-mode to bypassPermissions so the spawned claude can use
     all tools and persist context."* C'est une **decision deliberee
     d'UX** (mode discussion agent autonome) que **D3 PRESERVE**
     explicitement (PO-2 : garder bypassPermissions derriere le gate, pas
     le retirer). D3 ne contredit donc PAS cette decision — il l'encadre.
     **Non-bloquant** (la decision est respectee : le happy-path garde
     bypassPermissions, seul le chemin sensible est gate).

  4. **Modele `sonnet` hardcode — `e26d9f2`/`eb06c35`**. Aucune decision
     documentee justifiant `sonnet` ; le body `e26d9f2` dit "only Claude
     is wired". C'est une valeur de cablage par defaut, **non confrontee a
     la regle modele gelee** (`feedback_model_46.md` : toujours
     `claude-opus-4-8[1m]`). D4 corrige une violation de regle gelee.
     **Non-bloquant** (la regle gelee PRIME ; pas de reversion d'une
     decision valide — il n'y a pas de decision valide a renverser).

  5. **Pattern loopback durci daemon — S16 `d7c265a`** (bearer 256-bit +
     Host allowlist + Origin check CVE-2025-49596 + peer creds). Documente
     `THREAT_MODEL.md §7` (LIVRE S16A) +
     `LOOPBACK_ENDPOINTS_TRUST_TIERS.md`. **Statut : valide, en vigueur.**
     D5 **aligne l'Operator sur ce pattern** (token+Host comme
     `daemon_client.rs:64-65`). **Coherence CONFIRMEE** — D5 etend le
     pattern projet-wide, ne le contredit pas. Note de coherence
     (kickoff D5 ⚠️ adjust) : le daemon utilise UDS/NP peer creds en plus ;
     l'Operator reste sur token+Host (HTTP TCP, pas UDS) — **meme modele
     de menace que le tier AUTO documente** (`THREAT_MODEL.md §9.2` : "Si
     bearer leak via AD2 : acces complet API loopback"). La surface
     "process local hostile qui lit le token" est **deja hors scope
     projet-wide** (R1 keypair plaintext, sandbox OS niveau noeud). Pas
     une regression — un alignement sur la posture existante.

- **Finding S2** : **clean (non-bloquant)**. Toutes les decisions
  traversees sont soit des **deferrals anticipes** (CORS Any → "durcir
  quand la surface persiste", realise), soit des **bugs vs intention
  declaree** (gate SSE cru preserve), soit des **violations de regle gelee**
  (sonnet), soit des **decisions deliberees que D3 preserve**
  (bypassPermissions), soit le **pattern de reference a etendre** (loopback
  S16). Aucune decision rejetee avec rationale valide n'est re-introduite,
  aucune decision gelee n'est contredite. **Pas de DESIGN-CONFLICT S2.**

---

## S3 Local Patterns And Threat Model — S3 FULL (nouveau durcissement securite)

> S3 FULL obligatoire : la phase touche un composant securite reseau-expose
> (serveur loopback qui ecrit/spawn). Threat-modele complet ci-dessous,
> croise avec `THREAT_MODEL.md` (AD1-AD5, STRIDE) et
> `LOOPBACK_ENDPOINTS_TRUST_TIERS.md`.

### Asset
Le serveur Operator `:3001` expose deux capacites a fort privilege sur
loopback :
- **A-write** : `/api/artifacts/draft` ecrit des fichiers dans
  `.planning/`, `docs/`, `prompts/`, `AGENTS.md`, `CLAUDE.md`
  (path-guarded l.416-495, gate PASS-verdict l.434-470).
- **A-spawn** : `/api/chat/{id}/stream` spawn `claude --permission-mode
  bypassPermissions` (`llm_bridge.rs:80`) = agent autonome avec **acces
  filesystem + shell + commit** dans le repo ; `/api/terminal/ws` spawn un
  PTY interactif (`terminal.rs`).

### Actors / vectors
| Actor | Vector | Aujourd'hui (HEAD `0daff81`) |
|-------|--------|------------------------------|
| **AD1 site web malveillant** (browser de l'user) | CSRF : `fetch('http://127.0.0.1:3001/api/chat/.../stream')` cross-origin. `allow_origin(Any)` (l.88) → la reponse est **lisible** par le site attaquant ; le POST `/api/chat/{id}/send` puis GET stream **spawne un agent bypassPermissions**. | **NON MITIGE** (CORS Any, zero auth, zero Host guard). |
| **AD1 DNS rebinding** | Le site resout `attacker.com` → `127.0.0.1`, contourne la same-origin policy, parle au serveur comme "same-origin". | **NON MITIGE** (pas de Host guard ; `Host: attacker.com` accepte). |
| **AD2 malware user-mode** | Lit le token si stocke en clair / appelle l'API directement (meme user). | Hors scope (= R1 projet-wide, sandbox OS niveau noeud). Aligne `THREAT_MODEL.md §9.2 AUTO`. |
| **Worker honnete (UX)** | Discussion agent autonome legitime (happy-path PO-2). | Doit rester fonctionnel (R5). |
| **Subprocess `claude` hang** | Un agent qui ne termine jamais bloque le stream et **fuit un process** (zombie). | **NON MITIGE** (`spawn_claude_stream` sans timeout, `llm_bridge.rs:64-118`). |

### Mitigations apportees par Phase C (mapping D3-D6)
| Threat | Mitigation Phase C | Decision |
|--------|--------------------|----------|
| AD1 CSRF (lecture cross-origin) | CorsLayer `AllowOrigin::predicate(is_loopback_origin)` au lieu de `Any` (pattern `http.rs:513`) + token `X-SBFB-Token` sur routes mutantes/spawn | G7 / D5 |
| AD1 DNS rebinding | Host guard `is_loopback_host` (rejette `Host: attacker.com`) — middleware `auth.rs` reference | G7 / D5 |
| Exfiltration via SSE non gate | Filtre `SENSITIVE_ACTIONS` applique AVANT `spawn_claude_stream` (l.776) → `requires_gate` si dernier msg user contient shell/commit/push/PASS | G2 / D3 |
| Subprocess zombie | `tokio::time::timeout` + `child.kill()` + `child.wait().await` (anti-zombie) ; `kill_on_drop` filet | G12 / D6 |
| Spawn opaque | Pre-spawn resolution check de `claude`/`claude.cmd` + message diagnostic clair | G12 / D6 |

### Gaps / decisions d'execution non bloquantes (les 4 carry du resume)

- **R6 (token vs proxy)** — DECISIF. Le front reel
  (`tools/factory-operator/vite.config.ts`) proxifie `/api` → `:3001`
  **server-side** ; le browser fait `fetch('/api/...')` **same-origin
  (:5174)** via `useApi.ts`. Consequences :
  - Le `allow_origin(Any)` n'est **jamais exerce** par le front legitime
    (la requete browser ne touche pas `:3001` en cross-origin). → CORS
    restreint **ne casse pas** le front. R6 cote CORS = **de-risque**.
  - Le **token** est la vraie complication : le browser ne lit pas
    `~/.sbfb/auth_token`. Trois options d'implementation (a acter Phase C) :
    - **(A) Token injecte par le proxy Vite** : le proxy ajoute
      `X-SBFB-Token` server-side (lit le fichier au boot). Front inchange.
      Le token n'expose pas le browser. **Recommandee** (preserve
      l'isolation browser, suit le modele "le proxy est le client de
      confiance").
    - **(B) Token requis seulement sur routes mutantes/spawn**, GET de
      lecture gardes par Host+Origin loopback seuls (comme `/auth/token`
      du daemon, T0 `LOOPBACK_..._TRUST_TIERS §3`). Reduit la surface de
      bootstrap token tout en gardant le spawn/write gate.
    - **(C) Endpoint `/auth/token`-like sur l'Operator** + bootstrap front
      (pattern `web/src/api/auth.ts` `fetchAuthToken`). Plus lourd, mais
      aligne sur le shell.
  - Decision : recommander **(A)+(B) combinees** — proxy injecte le token
    ET le serveur n'exige le token QUE sur les routes mutantes/spawn
    (`/api/actions/run`, `/api/artifacts/draft`, `/api/chat/{id}/send`,
    `/api/chat/{id}/stream`, `/api/terminal/ws`), les GET lecture restant
    Host+Origin. Le Host guard + CORS predicate s'appliquent a TOUT.
    Ferme R6 sans casser le front. **Non bloquant** (choix d'impl trace).
- **R5 (gate SSE casse l'autonome)** — le SSE est **orphelin cote front
  reel** (`AgentChat.tsx` utilise `/api/terminal/ws`, pas
  `/chat/{id}/stream`). Gater le SSE ne casse aucun chemin vivant. Le gate
  reste correct (defense serveur contre appel API direct / re-cablage
  futur). Le happy-path NON sensible passe (test `sse_allows_nonsensitive`).
  Limite a documenter : le **terminal PTY WebSocket** (`/api/terminal/ws`)
  est un **chemin de spawn distinct non couvert par D3** (il ouvre un shell
  interactif, pas un agent `-p`). Le gate SENSITIVE_ACTIONS ne s'y applique
  pas (il n'y a pas de "dernier message user" a inspecter — c'est un
  terminal brut). **Le PTY reste un canal privilegie protege par
  Host+token (D5), pas par le gate de contenu (D3)** — a documenter
  PATTERNS comme limite assumee (le WS est gate par auth de connexion, pas
  par filtre de contenu). Non bloquant (la protection est l'auth de
  connexion + le fait que l'user pilote lui-meme son terminal).
- **Tests sans token** — les 24 tests `operator_server.rs` (`TestServer`,
  l.16-58) appellent sans token. Si le token devient obligatoire sur les
  routes testees, ils cassent. Le harness doit lire le token (stdout
  `READY` etendu, ou fichier) et l'injecter — **meme commit**. (Si option
  (B), seuls les tests des routes mutantes ont besoin du token ; les GET
  lecture restent verts.) Non bloquant (mecanique, anticipe).
- **Dep du predicat loopback** — `is_loopback_host`/`is_loopback_origin`/
  `AUTH_HEADER` vivent dans `nexus-shell-daemon-core` (pas une dep de
  `sbfb-factory`). Choix : (a) dep `nexus-shell-daemon-core` (lourd —
  tire tout le daemon-core), (b) lifter les 2 predicats + AUTH_HEADER vers
  `nexus-core-rs` (deja dep de sbfb-factory, reutilisable par les deux),
  (c) reimplementer localement (~30 lignes triviales). **Recommandee (b)
  ou (c)** ; (a) est dispro. Non bloquant (choix d'impl trace).

### Croisement THREAT_MODEL / HARDENING
- `THREAT_MODEL.md §7` documente bearer+Host+Origin comme LIVRE S16A pour
  le daemon ; l'Operator est un **second serveur loopback** introduit
  off-sprint **sans** ce gate. Phase C **comble une regression de posture**
  (le projet a un standard loopback durci ; l'Operator ne le respectait
  pas). **Ce n'est PAS une regression d'une menace deja couverte par
  l'Operator** (l'Operator n'a jamais ete durci) — c'est l'alignement d'un
  composant neuf sur le standard projet. Aucune pre-requirement
  HARDENING_ROADMAP S71 manquante.
- `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` : l'Operator `:3001` n'est pas dans
  l'inventaire §3 (cree apres). Phase C le place de facto au tier **T0
  (AUTO : bearer + Host + Origin)** — le standard minimal. Les routes
  spawn/write meriteraient T1 (CONFIRM_PROMPT) a terme ; le gate
  SENSITIVE_ACTIONS (D3) est un **proto-T1 cote contenu** (confirmation
  externe requise). A documenter comme carry (T1 plein post-S71).

- **Finding S3** : **clean (non-bloquant) avec 4 carry d'execution
  traces**. Phase C **durcit** la posture (comble l'ecart de l'Operator vs
  le standard loopback S16), n'introduit aucune regression d'un threat
  couvert, preserve le happy-path PO-2. Limite assumee : le PTY WebSocket
  est protege par auth de connexion (D5), pas par le gate de contenu (D3)
  — documenter. **Pas de DESIGN-CONFLICT S3.**

---

## S4 Protocol And Wire Invariants

- **Wire/security files checked** : `crates/sbfb-factory/src/operator_server.rs`
  (structs `ChatSendRequest`, `ChatMessageRequest`, `ContextPackRequest`,
  `ActionRunRequest`, `ArtifactDraftRequest`, `StreamChunk` via llm_bridge),
  `crates/sbfb-factory/src/llm_bridge.rs` (`StreamChunk` enum SSE).

### Aucun canonical / aucun *_VERSION / aucune signature
- Le serveur Operator est un **API JSON HTTP local**, PAS un protocole
  P2P signe/gossipe. Grep `_VERSION|DOMAIN_|canonical_bytes|sign|verify`
  sur `operator_server.rs` + `llm_bridge.rs` → **0 hit**. Aucune de ces
  structs ne participe aux canonical bytes (`nexus-core-rs/src/canonical.rs`
  inchange), aucune n'est signee Ed25519, aucune n'a de `*_VERSION`.
  L'escalade S4 FULL (`canonical.rs`, `schemas/`, `*_VERSION`, `DOMAIN_*`)
  **ne s'applique pas** — ce ne sont pas des wire formats de protocole.

### `ChatSendRequest.model` (G9 / D4)
- Forme actuelle (`operator_server.rs:660-667`) :
  ```rust
  struct ChatSendRequest {
      message: String,
      #[serde(default = "default_provider")] provider: String,
      #[serde(default)] model: String,   // <- present mais IGNORE (l.776 hardcode "sonnet")
  }
  ```
- **Decision S4** : cabler `req.model` dans `handle_chat_stream` (l.776)
  avec defaut `claude-opus-4-8[1m]`. **Nuance** : `ChatSendRequest` est le
  body du POST `/chat/{id}/send` (l.669), mais le modele est consomme dans
  le GET `/chat/{id}/stream` (`handle_chat_stream`, l.735) qui **ne recoit
  PAS de body** (GET). Le modele doit donc etre **persiste dans la
  `ChatSession`** au `send` puis relu au `stream`, OU passe en query param
  du GET stream. **Choix d'impl Phase C** (le plan dit "cabler
  ChatSendRequest.model l.665" — l'implementation devra ponter send→stream
  via la session, car le GET stream n'a pas le body). Non bloquant —
  precision d'execution.
- **`#[serde(default)]` sur `model`** : **runtime tolerance LEGITIME**
  (pre-launch policy). Un client minimal qui omet `model` doit obtenir le
  defaut `claude-opus-4-8[1m]`, pas un 422. Le `#[serde(default)]` actuel
  donne `String::new()` (vide) → le code doit traiter `""` comme "absent →
  defaut opus-4-8" (ou changer en `#[serde(default = "default_model")]`).
  **Recommande** : `#[serde(default = "default_model")]` avec
  `fn default_model() -> String { "claude-opus-4-8[1m]".into() }` — plus
  explicite que `""`-puis-fallback. Rationale a ecrire dans la doc du
  champ : "runtime tolerance, pas compat historique". Conforme pre-launch.

### `StreamChunk` (SSE wire local)
- `llm_bridge.rs:11-28` : enum `#[serde(tag = "type")]` Delta/Thinking/
  Done/Error/Debug. C'est le format SSE local consomme (potentiellement)
  par un front. **Pas un wire de protocole P2P, pas de version.** G2 (gate)
  ajoute un chemin qui renvoie `requires_gate` AVANT le stream — le format
  `requires_gate` doit etre coherent avec celui deja emis par
  `handle_chat_send` (`{"ok": false, "requires_gate": true}`, l.704-708) et
  `handle_chat_message` (`{"requires_gate": true, "requires_external_agent":
  true}`, l.631-636). **Recommande** : le SSE gate emet un `StreamChunk`
  d'erreur/gate coherent (p.ex. un nouveau variant ou un `Error` avec
  message gate) OU renvoie un `Sse` avec un seul event `requires_gate` —
  choix d'impl. Pas de bump de version (StreamChunk extensible localement,
  pre-launch). Non bloquant.

### Pre-launch policy
- `CLAUDE.md §Pre-launch` + `kickoff §2.3` : 17 ahead origin, rien pousse,
  aucun noeud tiers. **Edition libre** des formats locaux. Mais ici il n'y
  a **meme pas de wire de protocole** a editer — ce sont des DTO HTTP
  locaux. `#[serde(default)]` sur `model` = runtime tolerance assumee.
  Aucun decodeur multi-version, aucun `*_VERSION` a bump.

- **Day 0 status** : **preserved.** D3 (gate SSE, garder bypassPermissions),
  D4 (opus-4-8), D5 (token+Host+CORS), D6 (timeout+diagnostic) respectes ;
  aucune decision gelee contredite (cf. S2 : bypassPermissions PRESERVE,
  pattern loopback S16 ETENDU, regle modele HONOREE).
- **Finding S4** : **clean (non-bloquant)**. Aucun canonical/signature/
  `*_VERSION` touche. `ChatSendRequest.model` `#[serde(default)]` = runtime
  tolerance legitime (recommande : `default = "default_model"` opus-4-8).
  Format `requires_gate` SSE a rendre coherent avec les 2 chemins
  existants. Send→stream pontage via session (GET stream sans body). Pas de
  decodeur multi-version. **Pas de DESIGN-CONFLICT S4.**

---

## Risks And Scope Cuts

- **Blocking risks** : **aucun**. (Aucun S1b/S2/S3/S4 bloquant →
  pas de DESIGN-CONFLICT ; aucun S1a APPROACH-NAIVE/LIB-EXISTS →
  pas de PLAN-ADAPT.)
- **Non-blocking risks / carry (les 4 d'execution)** :
  - **R6 (token vs proxy)** : front proxifie → CORS restreint ne casse pas
    le front ; token injecte cote proxy Vite (recommande A) + exige sur
    routes mutantes/spawn seulement (recommande B). Decision d'impl Phase C.
  - **R5 (gate SSE)** : SSE orphelin cote front reel → gate ne casse rien.
    Limite documentee : PTY WebSocket protege par auth de connexion (D5),
    pas par le gate de contenu (D3).
  - **Tests sans token** : harness `TestServer` token-aware dans le meme
    commit (analogue tests de R6).
  - **Dep predicat loopback** : lifter vers `nexus-core-rs` (b) ou
    reimplementer local (c) ; pas la dep daemon-core (a) dispro.
  - **Send→stream pontage** (S4) : le GET stream n'a pas le body
    `ChatSendRequest` → modele persiste dans la session au send.
  - **T1 plein** (CONFIRM_PROMPT pour spawn/write) : le gate D3 est un
    proto-T1 cote contenu ; le T1 plein (nonce+TTL UI) reste post-S71
    (`LOOPBACK_..._TRUST_TIERS §6`). Carry, non-regression.
- **Scope cuts encore honores** (kickoff §8 / plan §12) — Phase C n'en
  touche aucun :
  - #1 ProviderRouter multi-LLM → S72 (D4 cable seulement le **defaut**
    opus-4-8 + le passage de `model`, PAS le router multi-provider).
  - #2 Chat Factory cable sur routage reseau → S72.
  - #16 Packaging produit Factory → S74 (token bootstrap proxy = securite,
    pas packaging onboarding).
  - L'auditeur grep : aucune ligne S71 Phase C ne touche un scope cut.
- **Note process portable** : le front Operator vit dans
  `tools/factory-operator/` (Vite :5174), PAS dans `web/` (le plan §7 C.2
  dit "`web/` (front Operator, si present)"). Le plan se trompe de
  repertoire ; la cible reelle est `tools/factory-operator/vite.config.ts`
  (proxy token) — a corriger au commit sans toucher le plan (snapshot).

---

## Action

- **SCOPE-CUT-CONSISTENT** : proceder avec Phase C telle que planifiee
  (`plan §7`), en integrant les 4 carry d'execution non bloquants ci-dessus.
  Le commit body de Phase C DOIT citer ce fichier
  (`sprint71_phase_c_preflight.md`) en section G8 et tracer les carry.
- Points fermes par ce preflight :
  - **S1a** : G2/G7/G9/G12 tous APPROACH-ALIGNED. Pattern de reference
    **in-repo** (daemon loopback S16 : `http.rs:513` CORS predicate,
    `auth.rs:239/274` Host/Origin, `daemon_client.rs:64-65` token client).
    Aucune lib externe a ajouter, aucune adaptation (pas de PLAN-ADAPT).
  - **S1b** : aucune nouvelle dep ; axum 0.8.9 / tower-http 0.6.10 / tokio
    1.52.3 deja au lock ; G13 (3 deps off-sprint) deja scannees Phase B.
    Clean.
  - **S2** : CORS Any = deferral anticipe S70 `69e3a06` ("durcir quand la
    surface persiste", realise) ; gate SSE = bug vs intention declaree
    `e26d9f2` ; bypassPermissions = decision deliberee que D3 PRESERVE ;
    sonnet = violation regle gelee ; loopback S16 = pattern a etendre.
    Aucune decision valide renversee. Clean.
  - **S3 FULL** : threat-modele complet (AD1 CSRF + DNS rebinding, exfil
    SSE, zombie subprocess) ; mitigations D3-D6 mappees ; comble l'ecart
    de l'Operator vs standard loopback S16 (pas une regression d'un threat
    couvert) ; happy-path PO-2 preserve ; PTY WS = auth de connexion, pas
    gate contenu (limite documentee). 4 carry d'execution. Clean.
  - **S4** : aucun canonical/signature/`*_VERSION`/`DOMAIN_*` touche (DTO
    HTTP locaux, pas wire P2P) ; `ChatSendRequest.model` `#[serde(default)]`
    = runtime tolerance legitime (recommande `default = "default_model"`
    opus-4-8) ; format `requires_gate` SSE a rendre coherent ; send→stream
    pontage via session. Day 0 preserved. Clean.
