# Sprint 72 Phase E Preflight

Date: 2026-06-03
HEAD: `110c003`
Verdict: **PLAN-ADAPT**

## Evidence Rules
- Claim policy: every claim below cites a repo path, a command run with its
  relevant output, a URL/date, or an explicit assumption.
- Local sources read (in full or targeted):
  `prompts/agent/preflight.md`,
  `.planning/active/sprint72_plan.md` (§8 + §11 + §12),
  `.planning/active/sprint72_kickoff.md` (§4 D4/D5, §7, §9 R4/R5),
  `.planning/active/sprint72_phase_d_preflight.md` (Resolution Option A),
  `.planning/active/sprint72_phase_d_review.md`,
  `crates/sbfb-factory/src/operator_server.rs` (chat routes, handlers,
  router/auth wiring `:100-202`, `:611-944`),
  `crates/sbfb-factory/src/auth.rs` (full),
  `crates/sbfb-factory/src/llm_bridge.rs` (`StreamChunk` `:42-59`),
  `crates/sbfb-factory/src/terminal.rs` (PTY claude spawn, grep),
  `tools/factory-operator/src/pages/AgentChat.tsx` (full — WS terminal),
  `tools/factory-operator/src/pages/AgentSelector.tsx` (Radix Select pattern),
  `tools/factory-operator/src/App.tsx`, `src/hooks/useApi.ts`,
  `src/i18n/index.ts` + `locales/fr.json`, `vite.config.ts`,
  `tools/factory-ui/src/operator/api-client.ts` + `operator/index.ts`,
  `web/scripts/scan-en-strings.sh` (scope head).
- Commands run (relevant outputs):
  - `git rev-parse --short HEAD` -> `110c003`.
  - `git log --oneline -8` -> Phase A `105c054`, B `08b6cb2`, C `3c9ea1b`,
    D `110c003` all landed.
  - `grep -rn "chat/.*send|chat/.*stream|EventSource|/api/chat" tools/.../src`
    (excluding node_modules) -> the ONLY non-node_modules hit consuming a chat
    route in the front is `getChatLog` (`/api/chat/{id}/log`) in
    `factory-ui/src/operator/api-client.ts:87`; **no** `send`/`stream`
    consumer exists. `AgentChat.tsx` uses `/api/terminal/ws` (WS) +
    `/api/prompt/{kind}?provider=claude` (prompt-adapt axis), not the chat SSE.
  - `grep -rn "postChatSession|postChatMessage|getChatLog" factory-operator/src`
    -> **empty**: those exported helpers are imported by no operator component.
  - `grep "factory-ui" factory-operator/package.json` -> **absent**:
    factory-operator does not depend on factory-ui's operator api-client.
  - `git log --all -S "EventSource" --oneline -- "*.tsx"` -> `e26d9f2`
    (added EventSource chat), `c3f4813` (removed it for the WS terminal).
  - `git show c3f4813 -- .../AgentChat.tsx | grep EventSource/WebSocket`
    -> `-const eventSourceRef ...` / `+const wsRef ...`,
    `-const es = new EventSource(sseUrl)`, `-POST /chat/${id}/send` removed.

## Scope
- Plan source: `.planning/active/sprint72_plan.md` §8 (Phase E) + kickoff §4 D4
  ("UX intentions COMPLETE in-scope S72", arbitrage PO 2026-05-31) + §4 D5
  (3 orthogonal axes) + §9 R4/R5.
- Target files (plan §8.2): `tools/factory-operator/src/` (chat/selecteur
  component), i18n FR/EN keys, an api-client transmitting `provider` to
  `POST /chat/{id}/send`.
- Deps/APIs/specs: NONE new. React 19.2 / Vite 8 / TS 5.9 / `@radix-ui/
  react-select` 2.2 / `react-i18next` 17 / `i18next` 26 are all already in
  `tools/factory-operator/package.json`. Native browser `EventSource` (no new
  dep). Backend contract already shipped Phase D (`110c003`).
- Security/protocol surfaces touched by the front: Operator loopback `:3001`
  (dev proxied from `:5174`); `X-SBFB-Token` auth on every route incl.
  `/api/chat/{id}/stream` (`auth.rs:229-262`); `SENSITIVE_ACTIONS` gate runs
  server-side BEFORE dispatch (`operator_server.rs:896-910`). The front cannot
  weaken either — both are server-enforced. No wire format (loopback SSE is a
  local contract, kickoff §1.4).
- Tests expected (plan §8.3): NONE (front Operator has no test runner —
  `package.json` has `build`=`tsc -b && vite build`, `lint`=`eslint .`, no
  Vitest). Gates: `(cd tools/factory-operator && npx tsc -b --noEmit)` exit 0
  + `npx eslint .` exit 0 + FR user strings.

## S1a OSS Prior Art
- Domain: a React execution-target selector ("run on cloud / locally / on the
  network") wired to a backend chat that streams an incremental assistant
  reply over SSE, with a non-streaming "in progress on the network" state for
  the async (poll) target.
- Sources (dated 2026):
  - Open WebUI (github.com/open-webui/open-webui) + AnythingLLM — mature OSS
    UIs that expose a provider/runner selector spanning local Ollama vs cloud
    vs remote API; "run locally vs cloud is a privacy/latency/cost trade-off"
    (WebSearch 2026-06-03; also kickoff §4 D4 AnythingLLM/Open WebUI note).
  - SSE-in-React patterns (Upstash "SSE to stream LLM responses",
    oneuptime.com 2026-01-15 "Implement SSE in React", Medium/DEV 2025-2026):
    native `EventSource` + incremental append of `data:` chunks to the
    assistant message is the current standard for one-way LLM token streaming;
    "SSE is a simpler alternative to WebSocket for one-way chat responses".
  - In-tree precedent: the SSE chat consumer ALREADY EXISTED in this repo
    (`e26d9f2` "wire chat to Claude CLI subprocess with SSE streaming" —
    `AgentChat.tsx` used `new EventSource('/api/chat/{id}/stream')`,
    `POST /chat/{id}/send`, rendered Delta/Done). It compiled and shipped.
  - React 19 StrictMode cleanup (context7 `/reactjs/react.dev`,
    StrictMode.md): an effect opening a connection MUST return a cleanup that
    closes it; StrictMode double-mounts effects in dev. `main.tsx:10` wraps the
    app in `<StrictMode>`, so this is load-bearing.
- Finding: **APPROACH-ALIGNED**. The intent-selector + SSE-consumer pattern is
  mature OSS practice AND has a working in-repo precedent. The PO-blessed scope
  ("UX intentions COMPLETE") is exactly what mature OSS does. No LIB-EXISTS
  (this is glue UI, not a library gap), no APPROACH-NAIVE.
- Impact: none on the chosen mechanism. The adaptation below is about the
  *ampleur* the plan under-described (a "dropdown" is actually a from-scratch
  SSE chat consumer), not about a flawed approach — which is why this is
  PLAN-ADAPT (S1a-grounded), not DESIGN-CONFLICT.

## S1b Dependencies, CVEs, Release Notes
- Scanned: `react@^19.2`, `@radix-ui/react-select@^2.2`, `react-i18next@^17`,
  `i18next@^26`, `vite@^8`, `typescript@~5.9` (all in
  `tools/factory-operator/package.json`); native `EventSource` (web platform).
- Commands/sources: `package.json` deps read directly; no `Cargo.toml` /
  `pyproject.toml` touched by Phase E (front-only).
- Finding: **clean**. No new dependency is added. `EventSource` is a stable web
  API (the in-repo `eventsource` npm package under node_modules is a transitive
  MCP-SDK dep, irrelevant to the browser front). No CVE on the front toolchain
  in scope; no major breaking release introduced (versions are unchanged from
  Phase D). React 19 StrictMode double-mount is a known dev-only behavior,
  mitigated by the mandatory effect-cleanup (S3 below), not a CVE.

## S2 Historical Decisions
- Commands:
  - `git log --all -S "EventSource" --oneline -- "*.tsx"` ->
    `e26d9f2`, `c3f4813`.
  - `git show e26d9f2 --no-patch` (full body) + `git show c3f4813 --no-patch`
    (full body) + `git show c3f4813 -- AgentChat.tsx` (diff grep).
  - Reverse-commit check: `git log e26d9f2..HEAD --oneline -- AgentChat.tsx`
    confirms `c3f4813` is AFTER `e26d9f2` and is the replacement commit.
- Decisions crossed:
  - **Chat-SSE-EventSource front consumer was DELIBERATELY REMOVED**
    (`c3f4813` "embedded Claude Code terminal + live project dashboard":
    *"Replace custom chat with real Claude Code terminal via PTY WebSocket"*).
    The diff removes `eventSourceRef`/`new EventSource`/`POST /chat/{id}/send`
    and adds `wsRef`/`new WebSocket('/api/terminal/ws')`. The current
    `AgentChat.tsx` (HEAD) is the xterm.js terminal — it spawns Claude Code
    interactively via a raw PTY (`terminal.rs:70-74` `claude.cmd`), which
    **bypasses `ExecutionTarget`/`provider_router` entirely** (grep on
    `terminal.rs` shows no `provider_router`/`ExecutionTarget` reference).
    Reversion status: **confirmed reversion of the OLD chat UI** — NON-blocking.
    The old removal was a product choice (full Claude Code terminal > custom
    token chat) for the *Claude-only, single-target* era; it predates the S72
    multi-target ExecutionTarget. Removing the old EventSource chat did NOT
    decide "never build a provider-routed chat UI"; it decided "for plain
    Claude, use the real terminal". S72's NetworkProvider/Ollama targets cannot
    run in a Claude-Code PTY, so a provider-routed SSE surface is the only way
    to exercise D4. No frozen decision forbids Phase E.
  - **UX intentions COMPLETE in-scope S72** (kickoff §4 D4, plan §8, scope cut
    #1 NOTE): the PO explicitly arbitrated (2026-05-31) that the full UX
    intentions screen is S72, only *packaging/onboarding* defers to S74. This
    legitimizes building the SSE consumer this sprint. Not reverted; valid.
  - **Two `provider` axes must not be conflated** (D5 / `PATTERNS §P53`/`§P55`):
    the existing front `AgentSelector.tsx` (5 values claude/codex/gpt/local/
    human) + `/api/prompt?provider=` (`AgentChat.tsx:214`) is the PROMPT-ADAPT
    axis. Phase E targets the EXECUTION axis (`ChatSendRequest.provider` ->
    `ExecutionTarget`, 3 values claude/ollama/network). Valid, must be honored
    (see Plan Adaptation — do NOT reuse `/api/prompt?provider=`).
  - **UX obligatoire: intentions, never `provider`/`kind` jargon in CTA**
    (`CLAUDE.md` "Decisions architecturales gelees"). Not reverted; binding on
    the i18n labels.
- Finding: **clean** (no blocking conflict). The one crossed decision (removal
  of the old chat UI) is a confirmed, context-resolved reversion, not a
  prohibition. PO scope + D5 + UX-intentions rules are all consistent with the
  adapted plan.

## S3 Local Patterns And Threat Model
- Threats/contracts checked: T-OPERATOR-CSRF / T-OPERATOR-SPAWN
  (THREAT_MODEL §14, catalogued Phase A `105c054`); daemon/Operator T0 loopback
  (`auth.rs`); `SENSITIVE_ACTIONS` gate-before-dispatch (S71 D3, preserved
  Phase D).
- **CRITICAL question resolved — EventSource auth.** `auth_required`
  (`auth.rs:229-262`) enforces `X-SBFB-Token` on EVERY route, incl.
  `/api/chat/{id}/stream` (`operator_server.rs:136` is inside the middleware
  layer `:162-165`). The native browser `EventSource` constructor **cannot set
  a custom request header** (web spec: only `withCredentials` cookies). A naive
  `new EventSource('http://127.0.0.1:3001/api/chat/{id}/stream')` from the
  browser would be **401**. RESOLUTION: the operator front is served by Vite
  dev (`npm run dev`, `:5174`); `vite.config.ts:46-52` proxies `/api` ->
  `http://127.0.0.1:3001` and **injects `x-sbfb-token` server-side on every
  proxied request** (`proxy.on("proxyReq", ... setHeader("x-sbfb-token", ...))`).
  Therefore a **same-origin, RELATIVE-path** `new EventSource('/api/chat/{id}/
  stream')` (no scheme/host) is proxied through Vite, which adds the token —
  the browser never needs the header. This is the same mechanism the existing
  `useApi.ts:37` `fetch('/api...')` and the WS terminal
  (`AgentChat.tsx:149` relative `/api/terminal/ws`) already rely on. **No
  custom fetch-based SSE reader, no query-param token, no cookie is needed.**
  The Phase D preflight's R3 concern about EventSource headers is answered:
  it is a non-issue *because the front is proxy-fronted*. (Production note: no
  static serving of the built operator front exists today — `operator_server.rs`
  has no `ServeDir`/fallback; the launcher does not bundle it. The operator UI
  is a dev tool reached via the Vite proxy. So the proxy path is the operative
  one; if a future sprint static-serves the built front same-origin on `:3001`,
  the relative EventSource still works because Host+Origin are loopback and the
  request carries no token requirement change — but that is S74 packaging, out
  of scope, flagged below.)
- Gate preservation: the front MUST NOT attempt to bypass the gate. The
  `SENSITIVE_ACTIONS` check runs server-side at `:896-910` BEFORE dispatch and
  emits a `requires_gate` SSE event (`sse_gate`, `:843-849`,
  `{"type":"requires_gate","message":...}`). The front must render that event
  as a gate notice, never silently retry. No new attack surface: Phase E adds a
  *consumer* of an already-shipped, already-gated, already-auth'd endpoint; it
  introduces zero new inbound route.
- HARDENING/Phase A status: Phase A (`105c054`) already catalogued the Operator
  `:3001` surface and the NetworkProvider client in THREAT_MODEL §14 +
  LOOPBACK §3. Phase E adds no inbound endpoint, so no further catalogue delta.
- Finding: **clean (non-blocking)**. No regression of a covered T0-T5 threat;
  the auth + gate invariants hold and are server-enforced. The EventSource
  header limitation is fully mitigated by the existing Vite proxy.

## S4 Protocol And Wire Invariants
- Wire/security files checked: none changed by Phase E. The SSE payload shape
  is `StreamChunk` (`llm_bridge.rs:42-59`, `#[serde(tag="type")]`:
  `delta`/`thinking`/`done`/`error`/`debug`) — a LOCAL loopback SSE contract,
  NOT a P2P wire format (kickoff §1.4 confirms `StreamChunk` is the
  Operator<->front local contract, free to evolve, no `*_VERSION`).
- VERSION/domain/canonical status: no `*_VERSION`, no `DOMAIN_*`, no
  `canonical.rs`, no schema touched. Pre-launch wire policy unaffected
  (front-only, loopback).
- Day 0 status: **preserved**. PO-14 (single `Done`, no WAN token-by-token) is
  honored by rendering the network path as Debug/progress + one final Done
  (the backend already emits exactly that — Phase D review confirms
  `dones==1`, `deltas==0`). Gate-before-dispatch preserved (server-side). D5
  (3 axes, do not reuse the prompt-adapt `provider`) preserved by wiring to
  `/chat/{id}/send`+`/stream`, NOT `/api/prompt?provider=`.
- Finding: **clean**. No wire bump, no canonical change, no Day 0 contradiction.

## Plan Adaptation
- Original plan (§8.1): "Implementation COMPLETE du selecteur d'intentions dans
  `tools/factory-operator/`" wording presumes an EXISTING chat to which a
  selector is added (§8.2 lists "composant chat/selecteur" + "api-client (ou
  equivalent)").
- Evidence requiring adaptation:
  1. `git show c3f4813` — the chat-SSE front consumer was REMOVED; the current
     `/chat` route (`App.tsx:40`) mounts a WebSocket xterm terminal that does
     NOT use `ExecutionTarget` (S2). So there is **no chat UI to extend** —
     Phase E must BUILD the provider-routed SSE chat consumer.
  2. `grep` — `factory-operator` does not import `postChatSession`/
     `postChatMessage`/`getChatLog` and does not depend on `factory-ui`; there
     is no operator api-client for chat. The send/stream client is also new.
  3. `auth.rs` + `vite.config.ts` — the auth/EventSource mechanism is settled
     (relative-path EventSource via Vite proxy; no header gymnastics).
  4. `web/scripts/scan-en-strings.sh` (head) is scoped to `web/src/` ONLY — it
     does NOT scan `tools/factory-operator`. The plan §8.4 "scan-en-strings if
     applicable" is **N/A here**; the FR-string gate is manual review + the
     fact that all user strings live in `i18n/locales/fr.json` (lng default
     `fr`, `i18n/index.ts:11`).
- Corrected approach (this supersedes plan §8 for Phase E ampleur; front-only,
  no Day 0 touched). Build a dedicated provider-routed execution chat surface:

  A. **New page** `tools/factory-operator/src/pages/ExecutionChat.tsx`
     (do NOT graft onto the terminal `AgentChat.tsx` — keep the Claude-Code
     terminal intact; they are different products). Mount on a NEW route, e.g.
     `App.tsx` `<Route path="/execute" .../>` + a Sidebar entry
     (`components/Sidebar.tsx` nav array, new `{ to:"/execute", key:"execute",
     icon:<lucide> }`) + `nav.execute` i18n key. (Alternatively reuse `/chat`
     by replacing the terminal — but that loses the interactive Claude Code
     terminal users have; NEW route is the lower-risk, additive choice.)
  B. **Intent selector** (the 3-target EXECUTION axis, D5 — distinct from the
     5-value prompt-adapt `AgentSelector`). Reuse the proven Radix Select
     pattern from `AgentSelector.tsx:79-102` OR a 3-card/segmented control.
     Closed set mapping to `ChatSendRequest.provider`:
     - intent "Executer sur Claude" -> `provider:"claude"`
     - intent "Executer en local"   -> `provider:"ollama"`
     - intent "Executer sur le reseau" -> `provider:"network"`
     CTA labels are INTENTIONS, never `provider`/`ollama`/`network` jargon
     (CLAUDE.md UX rule). Keep the literal `provider` value only in the wire
     body, not in any visible label.
  C. **api-client** (local to factory-operator, mirroring `useApi.postApi`
     pattern, relative `/api` paths so the Vite proxy injects the token):
     - `createSession()` -> `POST /api/chat/session` (body
       `{provider, project_id?}` — `ChatSessionRequest` accepts both,
       `operator_server.rs:611-621`; `project_id` defaults server-side to
       `operator-chat`, leave it default unless/until product defines it —
       the Phase D review flagged `project_id` as a Phase E UX decision; for
       this quick win, send no `project_id` and let the server default, OR
       expose it only when intent=network if product wants it — recommend
       default for S72, note as deliberate).
     - `sendMessage(id, {message, provider, model?})` ->
       `POST /api/chat/{id}/send` (THIS is where the selected intent's
       `provider` is transmitted — plan §8.2 item 3). Returns
       `{ok, requires_gate?}`; if `requires_gate` true, show the gate notice
       and do NOT open the stream.
     - `openStream(id)` -> `new EventSource('/api/chat/' + id + '/stream')`
       (relative path -> Vite proxy adds `x-sbfb-token`). Parse `event.data`
       as JSON `StreamChunk` by `type`:
       - `delta` -> append `text` to the in-flight assistant message (Claude/
         Ollama token streaming).
       - `thinking` -> optional muted "reflexion" indicator.
       - `debug` (label `"network-poll"`, content = status) -> render the
         **"en cours sur le reseau"** progress state (the NetworkProvider
         emits one Debug per poll tick — NO Delta tokens, PO-14). Map status
         dispatched/awaiting_quorum/completed to a FR progress label.
       - `done` -> finalize the assistant message with `result`
         (cost_usd/duration_ms available for an optional footer).
       - `error` / `requires_gate` -> render notice, close the stream.
       Close the EventSource in the `useEffect` cleanup (StrictMode dev
       double-mount — context7 React 19; `main.tsx:10` is StrictMode) AND on
       receiving `done`/`error`/`requires_gate`.
  D. **Network UX (R5)**: while `provider:"network"` and only Debug events
     arrive, show a persistent "en cours sur le reseau" state with the poll
     status, NOT a fake typing cursor. Make the async/batch nature explicit
     (no false live promise) — kickoff §9 R5, scope cut #12.
  E. **i18n**: add a new block to `fr.json` + `en.json`, e.g.
     `"execute": { "title", "intentClaude", "intentLocal", "intentNetwork",
     "intentClaudeDesc", "intentLocalDesc", "intentNetworkDesc",
     "send", "placeholder", "networkInProgress", "networkStatus.dispatched",
     "networkStatus.awaitingQuorum", "gateRequired", "streamError",
     "thinking" }` + `nav.execute`. Reuse existing `chat.gateRequired`/
     `chat.thinking` strings if semantically identical (already present
     `fr.json` chat block) to avoid duplication. ALL user-visible strings in
     French (default lng `fr`); `en.json` mirrors for completeness.
- File/test delta vs original plan:
  - NEW `src/pages/ExecutionChat.tsx` (the SSE consumer + selector).
  - NEW small api-client (inline in the page or `src/lib/operatorChat.ts`).
  - EDIT `src/App.tsx` (+route), `src/components/Sidebar.tsx` (+nav entry),
    `src/i18n/locales/fr.json` + `en.json` (+`execute` block, +`nav.execute`).
  - Tests: still NONE (no runner). Gates unchanged: `tsc -b --noEmit` exit 0 +
    `eslint .` exit 0 + manual FR-string + manual proof that the selected
    intent's `provider` reaches `POST /chat/{id}/send` (the load-bearing wire).
  - The plan's "scan-en-strings.sh" gate is replaced by "all user strings in
    `i18n/fr.json`, lng=fr" (scan-en-strings does not cover this package).

## Risks And Scope Cuts
- Blocking risks: **none**. (No S1b CVE, no S2 reversion conflict, no S3 threat
  regression, no S4 wire/Day 0 contradiction.)
- Non-blocking risks / carry-over:
  - R5 (false-live promise on network): mitigated by the explicit
    "en cours sur le reseau" Debug-driven progress state (no Delta WAN).
  - StrictMode dev double-mount of the EventSource: mitigated by the mandatory
    `useEffect` cleanup `es.close()` (context7 React 19). Load-bearing because
    `main.tsx:10` is StrictMode — call it out in review.
  - `project_id` product semantics (Phase D review P2): default server-side
    `operator-chat` for S72; richer per-project selection is deferred (Phase D
    review tracked it as a Phase E decision — recommend: keep default this
    sprint, document as deliberate, do NOT invent a project picker = that edges
    toward the S74 atelier).
  - Production static-serving of the built operator front: does not exist
    today; the EventSource auth relies on the Vite dev proxy. If S74 packaging
    static-serves it same-origin on `:3001`, re-verify the relative-path
    EventSource still satisfies Host/Origin/token (it should — same origin,
    loopback). Flag for S74, NOT this sprint.
- Scope cuts still honored (kickoff §7): #1 packaging/onboarding -> S74 (Phase E
  builds the functional selection screen ONLY, no installer/launcher/onboarding
  doc); #6 search/open/fork -> S74; #9/#10 cross-machine GPU/quorum -> S75;
  #12 token-by-token WAN -> never (one Done, Debug progress). The NEW `/execute`
  page is the "ecran de selection d'execution fonctionnel" explicitly scoped
  IN by §7 #1 NOTE — not a scope-cut violation.

## Action
- PLAN-ADAPT: proceed with the corrected approach above (build a dedicated
  provider-routed SSE execution-chat surface + 3-intent selector, NOT a
  selector grafted onto a pre-existing chat that does not exist). The Phase E
  commit body MUST cite this preflight: "Plan §8 proposed extending an existing
  chat; preflight S1a/S2 found the chat-SSE front consumer was removed in
  `c3f4813` and the WS terminal bypasses ExecutionTarget; adapted to build the
  SSE consumer + intent selector on a new `/execute` surface, wired to
  `/chat/{id}/send`+`/stream` (execution axis, D5), auth via the existing Vite
  proxy token injection." The plan file stays a snapshot (unchanged).
- Codex gate, a final `## Verdict: PASS` review line, and the 9-section commit
  body with `## Codex verification` remain required before the Phase E commit
  (this preflight authorizes no commit by itself).

## Verdict: PLAN-ADAPT
