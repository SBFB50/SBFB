# Sprint 80 — Phase C — Preflight G8

**Phase** : C — STEER complet (atelier + composeur en dock) + rail d'orientation ambiant altitude-0.
**Date** : 2026-06-27.
**Cas** : B (pre-code). Agent : `nexus-phase-preflight-deep`. Format : 5 scans factuels + verif adversariale.
**Verdict** : **PLAN-ADAPT** (le plan tient ; l'approche est corrigee avec evidence code/OSS ; **aucune** Day-0 figee touchee).

---

## 0. Scope Phase C (rappel du plan + kickoff)

Phase C livre la focale **STEER variante B** entierement cablee + le **rail ambiant altitude-0**, intentions-pas-jargon.

- **Livrables** : composeur en **dock** (exception etat-vide = composeur en grand) ; **atelier dominant** ; **transcript SSE (J3)** via `fetch()` + `ReadableStream` + `AbortController` (**jamais** `EventSource`), encapsule dans une primitive testee **`useTokenStream`** (accumulateur + Done-unique PO-14 + abort honnete S6a) ; `provider` = attribut discret ; **MUR `requires_gate` inline** ; rail = bandeau sprint·phase·branche·dirty/staged·pouls gates + selecteur de MODE + secondaires (sessions, historique, knowledge) ; CTA en **intentions**, jargon `kind/provider/preflight` replie.
- **Livrables §5.1 plies** : S1 (bibliotheque d'intentions versionnee), S3/D3 (prompt inspector), S4 (provider attribut + amorce diagnostic), S5 (relancer le tour), S6a (interrompre l'ecoute).
- **Backend** : **toutes routes existantes** dans la crate `sbfb-factory` — **0 route ajoutee au daemon** (Frozen Factory-hors-daemon tenu). `useTokenStream` est **net-neuf** cote front (scaffold Phase B minimal : `App.tsx`, `index.css`, `main.tsx`, `lib/cn.ts`).
- **T1** : sous-tests (2) composeur -> session creee + (3) **SSE token->Done deterministe** (un seul `Done`, PO-14) via cible mockee.

Invariants Day-0 pertinents (verifies au code, pas crus sur parole) : bi-focal rail altitude-0 + scene mono-focale state-driven (#8) ; STEER B atelier-dominant + composeur-dock (#8) ; D6 auto-bascule **manuelle** par defaut ; Operator HORS CSP scellee mais **CSP self-origin minimale** (#7) ; intentions-pas-jargon (#11) ; MUR `SENSITIVE_ACTIONS=[shell,commit,push,PASS]` jamais un bouton (#11) ; Base UI = SEULE dep de primitives (#3) ; SSE = fetch+ReadableStream+AbortController (plan amendement, supersede les refs EventSource du blueprint/kickoff) ; `provider` = attribut, 0 route daemon (#9).

---

## 1. Les 5 scans

### S1a — Delta SOTA (transport SSE authentifie) — **OK**

`fetch()` + `response.body.getReader()` + `TextDecoder` + `AbortController` (jamais `EventSource`) **EST l'etat de l'art reel et documente** pour du SSE authentifie — pas une lubie PO.

- **[info] fetch+ReadableStream = SOTA confirme, 3 raisons.** `EventSource` ne peut PAS poser d'en-tete custom (GET-only, pas d'Authorization/X-API-Key — limite structurelle). `fetch` permet le header natif X-SBFB-Token (supprime la raison du proxy Vite — l'ancien `executionChat.ts:13-16` DEVAIT passer relatif car EventSource ne pose pas de header) ; AbortController = annulation deterministe ; absence d'auto-reconnect = neutralise le re-rejeu du dernier tour (PO-14). Encapsuler UNE fois dans `useTokenStream` = exactement le conseil SOTA. *Evidence* : `git 37daa09^:executionChat.ts:13-16,84-89` ; Azure/fetch-event-source/src/parse.ts ; web-developpeur.com SSE-fetch.
- **[concern] Contrat de frames SCINDE en 2 sources Rust.** L'enum `StreamChunk` (`#[serde(tag="type")]`) n'a QUE 5 variantes : delta, thinking, done, error, debug (`llm_bridge.rs:42-59` — **verifie ce preflight**). Il n'existe AUCUNE variante `RequiresGate`. Les frames `requires_gate` et un `error` de garde sont des **litteraux JSON FORGES a la main** par `sse_gate()`/`sse_error()` (`operator_server.rs:1052-1069` — **verifie**). La TS doit coder en dur l'UNION des **6** valeurs de `type`. Re-generer l'union depuis le seul enum = oublier `requires_gate` = **MUR muet cote front**.
- **[concern] S6a abort honnete : `reader.cancel()` requis, pas seulement `releaseLock()` en finally.** `releaseLock()` (meme en finally) NE declenche PAS l'algorithme cancel du stream (le serveur ne voit pas le disconnect — bug confirme elysiajs/elysia#1768). Appeler `reader.cancel()` depuis le listener d'abort (resout le read() pendant avec `{done:true}` ET invoque cancel) + `releaseLock()` en finally. *Evidence* : freecodecamp SSE client ; elysia#1768 ; MDN Streams_API.
- **[concern] Discriminer `AbortError` de la vraie erreur reseau.** `abort()` fait rejeter le `read()`/fetch avec une DOMException `AbortError` -> traiter comme arret PROPRE (transition 'arrete d'ecouter', PAS 'error') ; ne re-lever comme erreur QUE les vraies erreurs reseau. Sans ca, S6a produirait un etat error trompeur (viole 'ne ment jamais'). Assertion T1(3).
- **[concern] Hazard PO-14 le plus reel = double-open du GET /stream (React 19 StrictMode / re-render).** Le risque double-Done n'est PAS dans le transport (fetch sans auto-reconnect = 1 reponse), il est dans le CYCLE DE VIE React. `handle_chat_stream` re-execute `target.run()` ET append le tour assistant a chaque Done (`operator_server.rs:1157,1162-1170` — **verifie**). StrictMode double-monte les effects ; un effet d'ouverture re-declenche ouvrirait DEUX GET /stream -> deux agents bypassPermissions + deux messages assistant. D'ou 'encapsuler une fois dans useTokenStream avec open controle' = load-bearing. T1(3) doit asserter UN SEUL open + UN SEUL Done sous double-invoke.
- **[concern] GET /chat/{id}/stream N'EST PAS idempotent serveur-side** (friction avec S5 'idempotent quasi-gratuit') — cf. §2 verif adversariale (REFUTED).
- **[info] 5 pieges de parseur a coder (ou eventsource-parser v3 MIT).** (1) carry de ligne partielle (`buffer.split(/\r\n|\r|\n/)` puis `buffer = lines.pop()`) ; (2) `TextDecoder({stream:true})` + flush final ; (3) frontiere de frame = ligne vide ; (4) ignorer lignes-commentaire `:` (keep-alive, forward-compat) ; (5) `releaseLock()` en finally. Cote SBFB le data est JSON compact SINGLE-LINE (`serde_json::to_string`) -> hand-roll ~25 lignes defendable (doctrine min-deps/AGPL). *Evidence* : Azure/fetch-event-source/src/parse.ts ; rexxars/eventsource-parser ; `operator_server.rs:1160`.
- **[info] Libs agent-chat dominantes NON reutilisables sous les invariants S80** (donc bespoke-sur-Base-UI coherent, pas du NIH). assistant-ui = 'built on Radix UI' -> violerait le lint BLOQUANT 0-@radix-ui (Day-0 #3). Vercel AI SDK useChat = protocole `data: [DONE]` litteral + runtime plus lourd, wire SBFB different. *Evidence* : assistant-ui ('primitives inspired by Radix UI') vs kickoff:173 ; ai-sdk.dev stream-protocol.
- **[info] Aucune surface SSE pre-existante dans le scaffold** — `useTokenStream` net-neuf. Le test single-Done historique (`executionChat.test.ts`, MockEventSource single-open) est a RE-PORTER en mock de `Response.body`/`ReadableStream` (Phase I).

### S1b — Deps / CVE — **OK**

- **[info] Phase C n'exige AUCUNE dependance front nouvelle.** (1) `useTokenStream` = APIs Web NATIVES (0 dep, c'est la raison du choix). (2) Composeur/atelier = Base UI 1.6.0 + React 19 + lucide-react + `cn()` (presents). (3) Rail = Tailwind v4 + Base UI + lucide-react + motion 12.42 (presents). Geist vendore present. *Evidence* : `package.json:22-35` + `src/lib/cn.ts:2-7`.
- **[info] package-lock.json coherent** (lockfileVersion 3, 349 pkgs) ; toutes versions dans les plages semver ; 0 @radix-ui residuel (purge Phase B tenue) ; 0 lib SSE/state/router pre-introduite.
- **[info] CVE-2026-23864 (RSC DoS) NON applicable + deja patche** : SPA client-only sans RSC servie par ServeDir Rust (hors surface) ET react 19.2.7 >= patch 19.2.4.
- **[info] motion 12.42 / @base-ui/react 1.6.0 / lucide-react 1.21 : aucun avis de securite connu.**
- **[info] Deps tentantes a REJETER** : (1) lib SSE (reintroduit l'auto-reconnect -> casse PO-14) ; (2) store global zustand/jotai/valtio (useReducer/useContext suffisent, sobriete OpenBSD-solo) ; (3) react-router (kickoff §Out differe le router complexe). Lock = 0 occurrence de ces libs.

### S2 — Decisions historiques traversees — **OK** (aucune decision gelee violee)

- **[info] (a) PO-14** ne au S72 (commits 1803d78/110c003/89652e1). Portee native = bras RESEAU ('submit->poll, then a single Done', `provider_router.rs:69-71,314-316`). Le plan l'etend au CONSOMMATEUR SSE front (anti-auto-reconnect) = extension FIDELE du contrat un-terminal-Done, PAS un conflit. **Note implementeur** : l'accumulateur doit gerer DEUX cas — Claude/Ollama (N deltas puis Done) ET Network (0 delta, Done porte `result`).
- **[info] (b) Rejet EventSource -> fetch+ReadableStream** : rationale gelee `19da665` + `react_vs_solid2_eval_2026-06-27.md:76-77`. React 19 CONFIRME (Solid 2.0 = beta). Plan = report 1:1.
- **[info] (c) Suppression du proxy Vite** : l'ancien `executionChat.ts:13-16` (S72, jettisone 37daa09) prouve que le proxy n'existait QUE parce qu'EventSource ne pose pas d'en-tete. Claim exacte et code-prouvee.
- **[info] (d) intentions-pas-jargon** : origine gelee **S70 = `CLAUDE.md:511-513`** + memory `po_directive_factory_front_redesign.md:42-45` (PAS `AGENT_SYSTEM.md` — pointeur du prompt errone, grep vide). Plan = 1:1.
- **[info] (e) MUR + chat_history_authoritative=false** confirmes (`operator_server.rs:37,722-755,909-941,1110-1131`). Le declencheur est un substring-match LARGE (un message anodin contenant 'commit'/'push' declenche le mur) -> le front RESTITUE `requires_gate`, jamais un bouton Forcer/Override ni un pre-filtre 'plus malin'.
- **[info] (f) D6 auto-bascule manuelle** : le rail livre le 'selecteur de MODE' = le toggle MANUEL conforme. L'auto-bascule state-driven [fin de tour ET diff/gate frais] est Phase H, hors Phase C. Le rail (cadre stable altitude-0) heberge le controle ; le rail lui-meme ne transitionne pas.
- **[concern] Sequencement : le 'pouls gates' du rail precede `/api/gates` (Phase G).** `/api/gates` n'existe pas (grep=0) ; seul `/api/status` (handle_status) fournit sprint/phase/branche/dirty/staged. Le 'pouls gates' doit etre rendu en etat DEGRADE/'non cable' (placeholder), jamais une jauge/verdict UI (garde '0 verdict calcule UI'). Note de sequencement, PAS une violation de Day-0.
- **[info] Precondition Phase A (cookie fallback) en place** : `auth.rs:307` header X-SBFB-Token d'abord puis fallback cookie `sbfb_operator` gate same-origin (`:332,338-351`).

### S3 — Threat model / surface d'attaque — **CONCERN** (design threat-sound ; footguns d'implementation)

- **[info] Auth SSE entierement enforce serveur** : route dans `authed` (`operator_server.rs:184,208-211`), `auth_required` Host-loopback + Origin-si-present + bearer 2-transports (`auth.rs:307-357` — **verifie**). Cookie gate sur `Sec-Fetch-Site: same-origin` (garde CSRF cross-port, en-tete forbidden non forgeable). 0 token en query. Phase C herite sans ajout.
- **[concern] Footgun prod : l'auth SSE = cookie HttpOnly same-origin, PAS un header X-SBFB-Token pose en JS.** Le token n'est jamais lisible en JS (HttpOnly + valeur = `session_secret` per-boot, distinct du bearer maitre). `useTokenStream` doit laisser fetch attacher le cookie automatiquement (credentials defaut = same-origin) ; `credentials:'omit'` retirerait le cookie -> 401. Le chemin header X-SBFB-Token n'existe qu'en DEV (proxy Vite, server-to-server). La conclusion 'proxy inutile en prod' reste VRAIE (le cookie fait le travail), mais le MECANISME est inexact -> ne pas coder une lecture de token cote client (impossible). *Evidence* : `auth.rs:331-353` ; `operator_server.rs:319` Set-Cookie HttpOnly ; `vite.config.ts` injection DEV-only.
- **[concern] AbortController coupe l'ECOUTE, pas le tour serveur ; interdire l'auto-reconnect.** `abort()` ferme la connexion mais ne tue pas le child (kill-child = future backend). Le Done est ecrit dans `session.messages` meme apres deconnexion, et re-ouvrir /stream RE-SPAWNE un tour neuf. -> (i) libelle UI 'j'arrete d'ecouter', jamais 'arrete' ; (ii) PAS d'auto-reconnect (= second spawn silencieux). Reconnect = intention explicite S5. *Evidence* : `operator_server.rs:1156-1157,1162-1171`.
- **[info] MUR `requires_gate` enforce inline AVANT tout spawn sur les 3 chemins chat** (SSE `:1116-1130`, send `:1011-1034`, message `:909-941`). Provider-independant (aucun bypass). Le composeur front NE PEUT PAS contourner. Front = barriere pleine-largeur, intention unique 'Preparer le pack', zero Forcer/Override.
- **[concern] Derive d'evidence : les refs MUR du prompt/plan/kickoff sont PERIMEES.** Cite `:35` (const) et `:766-779` (requires_gate sans spawn) ; reel : const a **:37**, gate SSE a **:1116-1130**, send **:1022-1034**, message **:920-941** (les lignes 760-783 = allowlist artifact_draft aujourd'hui). Mecanisme intact ; T1(4)/review doivent asserter contre les VRAIES lignes.
- **[info] `provider` = attribut : 0 surface d'injection.** `ExecutionTarget::from_provider` -> ensemble FERME (Claude/Ollama/Network) ; inconnu/vide -> Claude (fallback, jamais d'erreur dure). argv/JSON/GenerationRequest, jamais interpole dans un shell.
- **[info] CSP self-origin Operator presente et compatible Phase C.** `operator_csp_middleware` pose `default-src 'self'; connect-src 'self'` + nosniff sur CHAQUE reponse, layere a l'exterieur de auth. `connect-src 'self'` autorise le fetch SSE + le ws:// terminal. Operator PAS sous `BLOB_SERVE_CSP`. Phase C : 0 inline `style=`/`<style>`/script inline ; mutations `.style` JS de Motion = hors champ `style-src` -> OK. Garder `e2e/boot.spec.ts` (0 violation CSP sur bundle build) vert.
- **[info] Transport supersede** : les refs EventSource-via-cookie du blueprint §6.5 (`:315`) + kickoff Day-0 §5 (`:50`) sont SUPERSEDEES par l'amendement PO (`plan:24-31,98-106`). Ignorer ces refs.
- **[info] Residuel accepte herite** : la gate est par MOTS-CLES (shell/commit/push/PASS), pas par capacite ; l'agent spawn tourne aux privileges user (THREAT_MODEL T-OPERATOR-SPAWN). L'UI du MUR ne doit pas surclamer etre un sandbox de capacites.

### S4 — Invariants wire / format — **OK** (0 route, 0 champ, 0 bump)

- **[info] Les 10 routes consommees existent toutes deja** (`operator_server.rs` table de routage `:171-203`, **renumerotee** vs prompt) : status, context, context-pack, providers, prompt/{kind}, chat/session, chat/{id}/send, chat/{id}/stream (SSE), actions/run, actions/log, sprint-history*. S1 §5.1 passe (en theorie) par artifacts/draft. **0 route neuve.**
- **[info] Les numeros de ligne du prompt (:123,:136…) sont perimes** (table glissee a :171-203 suite a Phase A/F) mais les routes existent — derive documentaire, pas un gap.
- **[info] provider/project_id/intent/model sont des attributs DEJA presents** sur les corps (`ChatSessionRequest:816-825`, `ChatSendRequest:964-975`, serde defaults). Phase C ne fait que PEUPLER des champs existants. 0 ajout wire. Les `#[serde(default)]` = tolerance runtime pre-launch legitime.
- **[info] Aucun _VERSION / DOMAIN_*_V1 touche** : tous hors crate sbfb-factory (`nexus-core-rs/canonical.rs:77-332`, `coordinator/public_feed.rs`, `daemon-core/publish.rs`). Phase front pure -> politique pre-launch non sollicitee.
- **[concern] Le 'single-Done' PO-14 n'est PAS garanti structurellement par le backend sur un GET — invariant CLIENT.** Bras Ollama/Network = exactement 1 Done terminal et dernier (`provider_router.rs:819` `dones.len()==1`). Bras Claude = 1 Done par ligne ndjson result MAIS Done N'EST PAS le dernier event (Debug 'exit' + Error si exit!=0 suivent, `llm_bridge.rs:295-328`). Le vrai danger multi-Done = double-open/reconnect (chaque GET re-spawn `target.run` frais + append assistant sur Done, `:1156-1170`). `useTokenStream` doit accumuler les Delta, capturer `result` du PREMIER Done, traiter error/requires_gate comme terminaux, latcher (ignorer post-terminal), fermer le body sans rouvrir. PAS un conflit wire.
- **[info] `useTokenStream` net-neuf** (scaffold `tools/factory-operator/src/` = 4 fichiers, 0 client SSE) ; ecrit frais contre le contrat existant SANS le modifier (handler/route/StreamChunk inchanges). Pas de bump ni route requis -> pas de CONFLICT.

---

## 2. Verification adversariale (5 assertions)

| # | Claim | Verdict | Substance |
|---|---|---|---|
| 1 | '0 route / 0 wire neuf' : le contrat SSE existant suffit a un transcript multi-tour avec Done-unique | **HOLDS** | Routes toutes presentes (`:171-203`) ; attributs serde-default ; transcript = session.messages (/send append user `:1016-1020` + /stream append assistant sur Done `:1162-1170` + GET /chat/{id}/log relit) ; Done-unique = invariant CLIENT (latch). 0 champ/route additionnel. |
| 2a | `useTokenStream` single-Done faisable avec le contrat ACTUEL (backend emet 1 Done propre) | **HOLDS** (avec garde-fous) | Faisable mais le front DOIT latcher : Done Claude n'est pas le dernier event (`llm_bridge.rs:295-328`) ; pas de break sur 'result'. Le front DEDUPE defensivement, n'invente rien. fetch sans auto-reconnect supprime le re-rejeu. |
| 2b | **S5** 'Relancer = re-stream idempotent quasi-gratuit' | **REFUTED** | Chaque GET /stream RE-LANCE une inference complete (`target.run` frais `:1156-1157`) — cout plein, PAS quasi-gratuit — ET APPEND un ChatMessage assistant a CHAQUE Done (`:1162-1170`) -> historique CROIT, NON idempotent. Double-Done/double-spawn reel sur reconnect/relance/double-effet StrictMode. A acter : relance = NOUVEAU tour a cout plein ; tout trim = backend. |
| 3 | intentions-pas-jargon + MUR inline deja cables backend | **HOLDS** | MUR = event terminal mono-event sans spawn sur 3 chemins (`:1063,:1116-1130,:1022-1034,:920-941`) ; `SENSITIVE_ACTIONS:37`. Donnees d'intentions : `intent` sur ChatSessionRequest (`:819-820,853,881-884`), `/api/prompt/{kind}` (`:174`), `/api/providers` (`:177`). Mapping front, 0 dep backend manquante. |
| 3b | **S1 §5.1** intentions.json cablable via POST /api/artifacts/draft + lecture front | **REFUTED** | (1) ECRITURE : `.planning/factory/` ABSENT de `ARTIFACT_DRAFT_ALLOWLIST` (`:28-35` — **verifie ce preflight**) -> rejet 'path not in allowlist' (`:768-783`). (2) LECTURE : aucune route GET ne sert le fichier + ServeDir enracine sur `OPERATOR_BUNDLE_SUBDIR='tools/factory-operator/bundle'` (`:47`, PAS le repo root) -> 404. -> S1 = asset bundle BUILD-TIME (recommande) OU allowlister `.planning/factory/`. |

**Overall** : le coeur du plan Phase C est FAISABLE TEL QU'ECRIT. Les 3 assertions porteuses HOLDS. 2 sous-claims de prose sont REFUTED par le code (S5 idempotent, S1 intentions.json) -> a corriger AU/AVANT code. Aucun DESIGN-CONFLICT.

---

## 3. CONTRAT SSE FIGE (verifie au code — a mirrorer dans `useTokenStream`)

**Endpoint** : `GET /api/chat/{id}/stream` (axum `Sse`, `operator_server.rs:1071 handle_chat_stream`). RELATIVE, same-origin, **0 query token**, GET sans corps. `model`/`provider`/`project_id` relus de la `ChatSession` persistee (`:1075-1099`), pas d'un body. Le message user vit dans la session (cree via `POST /api/chat/session`, alimente via `POST /api/chat/{id}/send` qui append le message user).

**Auth** (Phase A) :
- **PROD** = cookie HttpOnly `sbfb_operator` (= `session_secret` per-boot, **PAS** le bearer) attache AUTOMATIQUEMENT par fetch. `fetch(url)` avec **credentials defaut (`same-origin`)** ; **NE PAS** poser `credentials:'omit'` (-> 401) ; **NE PAS** tenter de lire le token/cookie en JS (HttpOnly).
- **DEV** = le proxy Vite injecte `x-sbfb-token` server-to-server. Le code front est IDENTIQUE dev/prod (il ne pose aucun header d'auth lui-meme).
- Le serveur exige `Sec-Fetch-Site: same-origin` sur le chemin cookie (pose par le navigateur sur un fetch same-origin — rien a faire en JS). Un fetch cross-port echoue : voulu (garde CSRF cross-port).
- *Evidence* : `auth.rs:299-307,331-353` ; `operator_server.rs:184,208-211,319`.

**Transport / encodage** : `Sse::new(stream)` SANS `.keep_alive` (`:1176`). Wire = suite de `data: <json-compact>\n\n`, un Event par chunk, AUCUN champ `event:`/`id:`, AUCUNE ligne keep-alive aujourd'hui (mais ignorer les lignes `:` pour forward-compat). `data` = `serde_json::to_string(&chunk)` COMPACT single-line (`:1160`) -> `JSON.parse` direct par frame, pas de multi-data-line en pratique. Fin de flux = fermeture du body HTTP (pas d'event 'close' explicite).

**Forme des evenements — UNION des 6 valeurs de `type`** (scindee en 2 sources Rust) :

Derives de l'enum `StreamChunk` (`#[serde(tag="type")]`, `llm_bridge.rs:42-59` — **5 variantes seulement**) :
- `{"type":"delta","text":"…"}` — token de texte (accumulant)
- `{"type":"thinking","text":"…"}` — token de raisonnement (accumulant)
- `{"type":"done","cost_usd":f64,"duration_ms":u64,"result":"…"}` — **TERMINAL** : porte le texte complet
- `{"type":"error","message":"…"}` — **TERMINAL**
- `{"type":"debug","label":"…","content":"…"}` — informatif (stderr/'exit'/ndjson# ; bruit diagnostic)

FORGES a la main (hors serde, `operator_server.rs:1052-1069`) :
- `{"type":"requires_gate","message":"…"}` — **TERMINAL** (MUR ; `sse_gate`, un seul event puis close, JAMAIS de spawn)
- `{"type":"error","message":"session not found"|"no user message"}` — **TERMINAL** (`sse_error`, courts-circuits AVANT spawn)

**Classement accumulateur** : terminaux = `{done, error, requires_gate}` (stoppent) ; accumulants = `{delta, thinking}` ; informatif = `{debug}`.

**Cardinalite Done / PO-14 (load-bearing)** : le backend NE garantit PAS structurellement 'exactement un Done' sur un GET. Bras Claude : Done n'est PAS le dernier event (Debug 'exit' + Error suivent, `llm_bridge.rs:295-328`). Bras Ollama/Network : exactement 1 Done terminal (`provider_router.rs:819`). Bras Network : Done porte `result` complet, **0 delta** (PO-14, `provider_router.rs:69-71`). Le vrai danger multi-Done = double-open d'effet React / reconnect (chaque GET re-spawn `target.run` `:1156` + append assistant `:1162-1170`). -> invariant CLIENT : latcher le 1er terminal, ignorer le post-terminal, fermer sans rouvrir.

**CSP** : `default-src 'self'; connect-src 'self'` + nosniff sur toutes reponses (`operator_csp_middleware`). Phase C : 0 inline style/script ; Motion via mutations `.style` JS = OK. Garder `e2e/boot.spec.ts` vert.

---

## 4. Specs de la primitive `useTokenStream` (net-neuve, Phase C)

Une primitive UNIQUE et testee, encapsulant tout le transport. Contrats :

1. **Transport** : `fetch(url, { signal })` -> `response.body.getReader()` + `TextDecoder({stream:true})`. **JAMAIS** `EventSource`. `credentials` defaut (same-origin) — ne PAS omettre (sinon cookie retire -> 401). Aucun header d'auth pose en JS (dev = Vite proxy ; prod = cookie HttpOnly).
2. **Parseur** (5 pieges) : carry de ligne partielle (`buffer.split(/\r\n|\r|\n/)` puis `buffer = lines.pop()`) ; flush final `decoder.decode()` ; frontiere de frame = ligne vide ; ignorer les lignes `:` (keep-alive forward-compat) ; emettre par frame `data:`. Data = JSON compact single-line -> `JSON.parse` par event.
3. **Union 6-valeurs CODEE EN DUR** : delta, thinking, done, error, debug, **requires_gate** (NE PAS deriver du seul enum 5-variantes — oublier `requires_gate` = MUR muet).
4. **Accumulateur** : concat `delta.text` (+`thinking` si affiche separe) ; capturer `result` du **PREMIER** `done` ; gerer le bras Network (Done sans delta, afficher `result`) ; **Done-unique PO-14** = latch du 1er terminal `{done|error|requires_gate}`, IGNORER tout event post-terminal (Debug 'exit'/Error Claude), fermer le body **sans rouvrir**.
5. **Open idempotent cle-par-tour** : guard contre le double-open React 19 StrictMode ; cleanup d'effet qui **abort le fetch precedent**. Test : UN SEUL open + UN SEUL Done sous double-invoke d'effet.
6. **Abort honnete (S6a)** : `AbortController.abort()` + `reader.cancel()` depuis le listener d'abort (PAS seulement `releaseLock()` en finally — sinon le serveur ne voit pas le disconnect, elysia#1768) ; `releaseLock()` en finally pour defaire le verrou. Discriminer `AbortError` (DOMException) = arret PROPRE (etat 'arrete d'ecouter', JAMAIS 'error') ; ne re-lever comme erreur QUE les vraies erreurs reseau. Libelle UI 'j'arrete d'ecouter', jamais 'arrete' (le tour serveur continue ; kill-child reel = future backend).
7. **PAS d'auto-reconnect** : un reconnect = second spawn serveur silencieux. Reconnect = intention explicite (S5), **un NOUVEAU tour a cout plein** (pas un no-op — cf. §2 REFUTED).
8. **MUR** : sur `requires_gate`, RESTITUER la barriere (intention unique 'Preparer le pack'), JAMAIS un retry/force/override, JAMAIS un pre-filtre cote front.

**Re-port T1(3)** (intention PO-14 d'`executionChat.test.ts`) : stub d'un `ReadableStream` qui enqueue des chunks `data: {...}\n\n` + close ; asserter un seul open + un seul Done (latch) + abort -> etat 'arrete' (pas 'error').

---

## 5. VERDICT + justification

**Verdict : PLAN-ADAPT** (README §4.5.7).

Le coeur de la Phase C est FAISABLE TEL QU'ECRIT et NE viole **aucune** decision Day-0 figee : S1a/S1b/S2/S4 = OK, S3 = CONCERN (footguns d'implementation, pas de CONFLICT). 0 route neuve, 0 champ wire neuf, 0 _VERSION/DOMAIN touche, 0 dependance front nouvelle, `useTokenStream` net-neuf consommant le contrat existant sans le modifier. La verif adversariale REFUTE **2 sous-claims de prose** du plan (jamais des Day-0), avec evidence code concrete, plus **7 corrections d'approche load-bearing**. Toutes corrigent l'APPROCHE (avec evidence OSS/code), aucune ne touche un Day-0 -> **exactement** le critere PLAN-ADAPT. Pas de DESIGN-CONFLICT : rien dans le code n'invalide le design bi-focal STEER B / fetch+ReadableStream / MUR inline / intentions-pas-jargon.

**Plan-adaptations (9)** :
1. **S5** re-qualifie : la relance N'EST PAS idempotente/quasi-gratuite (re-spawn inference pleine `:1156-1157` + append assistant `:1162-1170`) ; libelle 'nouveau tour, cout plein'.
2. **S1 §5.1** intentions.json : PAS via artifacts/draft (allowlist `:28-35` exclut `.planning/factory/` ; ServeDir hors repo root `:47`) -> asset bundle BUILD-TIME (recommande).
3. Union 6-valeurs codee en dur (requires_gate forge, hors enum 5-variantes).
4. S6a : `reader.cancel()` depuis le listener d'abort + discrimination AbortError.
5. Open idempotent cle-par-tour + cleanup-abort (anti double-open StrictMode).
6. Mecanisme auth prod = cookie HttpOnly automatique (pas un header JS) ; ne pas omettre credentials.
7. Rail 'pouls gates' = placeholder 'non cable' (avant Phase G) ; jamais un verdict UI.
8. Gerer le bras Network (Done-sans-delta, `result`) + ignorer post-terminal Claude.
9. Rafraichir les refs de ligne MUR (reel `:37`, gate `:1116-1130`) pour les asserts T1.

**Design-conflicts** : aucun.

---

## 6. Contraintes pour le code (a respecter en codant Phase C)

**Scope / Frozen** :
- **0 route ajoutee au daemon** (`nexus-shell-daemon`) ; tout vit dans la crate `sbfb-factory` — et Phase C est front-pur (consomme des routes existantes, n'ajoute aucune route Rust).
- **0 dependance front nouvelle** : `useTokenStream` = APIs Web natives ; composeur/atelier/rail = Base UI + React 19 + lucide-react + motion + Geist (presents). REJETER toute lib SSE/state-global/router.
- **Base UI = SEULE dep de primitives** ; le lint BLOQUANT 0-@radix-ui doit survivre (interdit assistant-ui en runtime).
- **CSP self-origin** : 0 inline `style=`/`<style>`/script inline ; Motion via mutations `.style` JS OK ; garder `e2e/boot.spec.ts` (0 violation CSP) vert.

**Invariant MUR** :
- Le front RESTITUE le `requires_gate` (event terminal, sans spawn) en barriere pleine-largeur, intention unique 'Preparer le pack'. **ZERO** bouton Forcer/Override/retry-force. Aucun pre-filtre 'plus malin' cote front (le declencheur backend est un substring-match large, c'est voulu). `SENSITIVE_ACTIONS` reel = `operator_server.rs:37` ; gate SSE sans spawn = `:1116-1130`.

**Invariant intentions-pas-jargon + connaissance consommee-jamais-autoritaire** :
- CTA en intentions ; jargon `kind/provider/preflight` replie sous '▸ details techniques'. `provider` = attribut discret (`/api/providers`), 0 surface d'injection. Prompt inspector = repli technique strict (`/api/prompt/{kind}`).
- **0 verdict calcule UI** : le rail 'pouls gates' est un placeholder 'non cable' tant que `/api/gates` (Phase G) n'existe pas ; le selecteur de MODE = toggle MANUEL (D6).

**Contrat SSE (cf. §3) + `useTokenStream` (cf. §4)** :
- `GET /api/chat/{id}/stream`, fetch+ReadableStream+AbortController, jamais EventSource. credentials same-origin (ne pas omettre). Aucun header d'auth pose en JS.
- Union 6-valeurs codee en dur ; latch du 1er terminal (Done-unique PO-14) ; ignorer post-terminal ; bras Network = Done-sans-delta.
- Open idempotent cle-par-tour + cleanup-abort (anti double-open StrictMode + anti double-spawn serveur).
- Abort honnete : `reader.cancel()` + discrimination AbortError (etat 'arrete', jamais 'error') ; libelle 'j'arrete d'ecouter'. PAS d'auto-reconnect.
- S5 relance = NOUVEAU tour a cout plein (UI honnete, pas un no-op). S1 intentions = asset build-time (pas artifacts/draft).

**T1 (sous-tests Phase C)** :
- (2) composeur -> `POST /api/chat/session` cree la session.
- (3) SSE token->Done DETERMINISTE via cible mockee : un seul open + un seul Done (latch PO-14) ; abort -> etat 'arrete' (pas 'error').
- (4) MUR `requires_gate` asserte SANS execution (asserts contre les VRAIES lignes : `:37`, `:1116-1130` — pas `:35`/`:766-779`).

**Discipline** : commit unique `feat(factory-operator): Sprint 80 Phase C — …`, body riche (delta tests cumule + scope cuts respectes) ; preflight G8 (ce doc) -> review -> Codex avant commit.