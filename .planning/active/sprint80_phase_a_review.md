# Sprint 80 Phase A Review

> Driver-side deep review (agent `nexus-phase-review-deep`, Opus 4.8 1M).
> Independent process: I do not know the execution session history. This
> verdict is pre-Codex. The Codex auditor must replace `PASS-PENDING`
> with exact `## Verdict: PASS` (plus reconciliation) before commit, or
> downgrade to CONCERN/FAIL.

## Verdict: PASS

0 P0, 0 P1. 1 P2 + 4 P3 (rigor signal G4 satisfied: real branch-coverage
trade-offs found, not a 0-finding rubber-stamp). Code faithfully
implements the 11 frozen design decisions of the PLAN-ADAPT preflight.
Codex done (8/8 CONFIRME, 0 GAP, 0 PARTIEL) + P2-1/P3-2 closed in-phase —
see `## Codex reconciliation` below. Committable.

## Scope And Staging

Working-tree diff (uncommitted), 4 files, all coherent with Phase A
(backend auth cookie + bootstrap + ServeDir + Operator CSP, crate
`sbfb-factory` only):

- `crates/sbfb-factory/src/auth.rs` (+~95) — `AuthState.session_secret`,
  `OPERATOR_COOKIE`, `SEC_FETCH_SITE_HEADER`, `cookie_value`,
  `token_matches`, `session_secret`, cookie fallback in `auth_required`,
  2 unit tests.
- `crates/sbfb-factory/src/operator_server.rs` (+~190) — router split
  `bootstrap`/`authed`, `handle_bootstrap`, `operator_csp_middleware`,
  `OPERATOR_BUNDLE_SUBDIR`, `query_param`, `serve_bootstrap_index`,
  `build_router` gains `bundle: PathBuf`, `run_server` resolves bundle.
- `crates/sbfb-factory/tests/operator_server.rs` (+157) — 7 integration
  tests.
- `docs/security/THREAT_MODEL.md` (+41) — T-OPERATOR-CSRF amended.

No planning/cache/build artifacts mixed in. `?? .planning/active/sprint80_phase_A_preflight.md`
is the (untracked) preflight artifact — separate from the phase code
commit; should be staged with its own planning concern, not folded into
the feature commit. No `+pub mod` added. Atomic.

Red-line DEEP triggered: diff touches `docs/security/THREAT_MODEL.md` and
loopback HTTP auth → full audit performed (no "acknowledge preflight"
shortcut). No `unsafe`, no new `#[allow(dead_code)]`, no crypto primitive
changed (re-uses existing `generate_token`/`constant_time_eq`).

## Three-Block Verification

Reported by executor as ALL GREEN on Windows native; I did not re-run
(independent agent, suites are heavy — trusting the reported state per
§7.4, Codex will re-verify):

- `cargo fmt --all --check` clean; `clippy --workspace --all-targets
  --locked -D warnings` rc=0; `nextest --workspace --locked` = 2004
  passed / 0 skipped (+9: 2 unit + 7 integration); `cargo test --doc`
  rc=0; `cargo build -p nexus-shell-daemon --release` rc=0.
- Frontend untouched (lint/tsc/unit/build/size/scan green).

Note for Codex: the dual-platform Docker (rust:1.94) re-run before push
is the canonical gate (memory `feedback_wsl_before_push`); diff is
platform-agnostic (axum/std only) so low drift risk, but confirm fmt
under 1.94.

## Delta Tests

+9 Rust (1995→2004 in this crate's slice; workspace 2004 total reported).
- 2 unit (`auth.rs`): `cookie_value_extracts_named_cookie`,
  `session_secret_is_distinct_from_token`.
- 7 integration (`tests/operator_server.rs`):
  `bootstrap_valid_token_sets_cookie_and_303`,
  `bootstrap_invalid_token_no_cookie`,
  `cookie_auth_succeeds_with_sec_fetch_site`,
  `cookie_auth_rejected_without_sec_fetch_site`,
  `cookie_auth_rejected_with_wrong_value`, `header_wins_over_bad_cookie`,
  `operator_csp_header_present`.

Frontend delta = 0 (untouched; greenfield front lands Phase B — the
−7/−8 Vitest jettison is acted for Phase I). Acceptable.

## Modified-File Branch Coverage

Semantic walk (read each test in full, not grep-matched):

| New path / branch | Real call | Specific assert | Both sides | Verdict |
|---|---|---|---|---|
| `cookie_value` (parse, multi, empty, prefix) | yes (unit) | exact `Some/None` | yes | COVERED |
| `session_secret` distinct from token | yes (unit + integ) | `assert_ne` + len + hex | n/a | COVERED |
| `auth_required` header path (unchanged) | yes (`get`/`post_json`, `header_wins_over_bad_cookie`) | 200 | n/a | COVERED |
| `auth_required` cookie OK + Sec-Fetch-Site | yes (integ) | 200 | yes | COVERED |
| `auth_required` cookie OK, NO Sec-Fetch-Site | yes (integ) | 401 | yes | COVERED |
| `auth_required` cookie wrong value | yes (integ) | 401 | yes | COVERED |
| `auth_required` no cookie + no header | yes (`server_rejects_missing_token`) | 401 | yes | COVERED |
| header-first precedence over bad cookie | yes (integ) | 200 | yes | COVERED |
| `handle_bootstrap` valid `?token` → 303 + cookie | yes (integ) | 303, HttpOnly, SameSite=Strict, Path=/, no Secure, Location:/, no-referrer, cookie≠token, len 64 | yes | COVERED |
| `handle_bootstrap` wrong/no `?token` → neutral | yes (integ) | not-303, no Set-Cookie | yes | COVERED |
| **`handle_bootstrap` Host non-loopback → 403** | **NO** | — | **no** | **GAP (P2-1)** |
| `operator_csp_middleware` CSP present | yes (integ, 200 only) | `default-src 'self'; connect-src 'self'` | error responses untested | PARTIAL (P3-2) |
| ServeDir asset behind auth (fallback) | NO (bundle absent in tests) | — | — | GAP (P3-1) |

The one meaningful gap (P2-1) is a security control re-implemented in the
public bootstrap (which deliberately bypasses the middleware), explicitly
listed in preflight decision #10 as a test to add, and omitted.

## Security And Protocol

Each preflight-listed semantic point verified against the actual code:

1. **Chicken-and-egg — OK.** `bootstrap = Router::new().route("/",
   get(handle_bootstrap))` has NO `.layer(auth_required)`. `/api/*` +
   `.fallback_service(serve_assets)` live in `authed` with
   `.layer(from_fn_with_state(auth_state, auth_required))`. `GET /?token`
   reachable cookieless. axum matches `route("/")` ignoring query →
   single route covers `?token` hit and post-303 `/`.
2. **P1-A cross-port CSRF — OK.** `Sec-Fetch-Site: same-origin` required
   ONLY inside the `if !header_ok` block (cookie path). Header path
   returns early without it (`header_wins_over_bad_cookie` proves CLI/Vite
   unaffected). Cookie valid + no Sec-Fetch-Site → 401 (tested). A
   cross-port page on `127.0.0.1:<other>` is *same-site* not
   *same-origin*, so the browser emits `Sec-Fetch-Site: same-site` →
   rejected; the header is forbidden (JS cannot forge). Guard is sound.
3. **P1-B bearer leak — OK.** Cookie value = `session_secret =
   generate_token()` (per-boot CSPRNG), never `auth.token`. Cookie path
   compares against `session_secret` (`auth.rs:344`); bootstrap mints
   `session_secret` (`operator_server.rs:316`). Unit test asserts
   distinct + 64 hex; integration asserts emitted cookie ≠ TEST_TOKEN.
   A stolen/cross-port cookie never yields the daemon master bearer.
4. **ServeDir root — OK.** `bundle = root.join("tools/factory-operator/bundle")`,
   distinct from `repo_root_pub()` and from the legacy `dist`. `ServeDir`
   never rooted at repo. tower-http `fs` default path-traversal mitigation
   applies; dotfile exposure mitigated by dedicated dir + auth gate (D4).
5. **Bootstrap neutrality — OK.** No-token and wrong-token both fall to
   `serve_bootstrap_index` → identical response (no oracle). Cookie:
   HttpOnly + SameSite=Strict + Path=/, NO Secure (loopback http), NO
   Max-Age/Expires (session-only), no Domain (host-only). 303 +
   `Referrer-Policy: no-referrer` drops `?token` from the bar.
6. **CSP — OK.** `operator_csp_middleware` layered OUTER of the merge
   (`.layer(from_fn(operator_csp_middleware))` then `.layer(cors)`), so it
   wraps bootstrap + authed + their 401/403/404 (auth_required errors are
   inner). `default-src 'self'; connect-src 'self'`, no `unsafe-inline`/
   `unsafe-eval`, + `nosniff`. connect-src covers same-origin SSE/ws.
7. **Tests preserved — OK.** `server_rejects_missing_token` (Host only) →
   401 (cookie absent never fabricates authority). Header-first proven.
8. **Invariants — OK.** No `use nexus_shell_daemon*` (only doc-comment
   references in `fork.rs`). `Cargo.toml` unchanged → 0 dep added (cookie
   hand-rolled, mirrors existing loopback helpers). 0 wire-format/version
   touched (`TOKEN_HEX_LEN=64` intact; cookie/CSP are HTTP transport, not
   serialized envelopes). No Day-0 decision rebattu (D5 honored to the
   letter, the 3 additive refinements are P1 fixes D5 does not freeze).
   `build_router` signature change only reaches `run_server`; `main.rs`
   calls `run_server`, no other in-crate/external caller.
9. **THREAT_MODEL honesty — OK.** Amendment retracts the now-false `:813`
   ("a third-party browser does not know the token") for the cookie path,
   documents the two P1 guards, and records residuals (token-in-URL
   history/Referer, cross-port) honestly with accepted-rationale.

Axum semantics check (load-bearing, untested in-repo): `Router::layer`
runs the middleware for ALL requests INCLUDING the fallback service
(unlike `route_layer`). The code uses `.layer`, so the ServeDir/index
fallback IS behind `auth_required`. Confirmed by axum 0.8 contract; flag
that no test pins it (P3-1).

`rg unsafe|unwrap|panic|todo` on changed code: no new `unsafe`, no new
`panic!/todo!`. `unwrap_or(false)`/`unwrap_or(rest.len())` in tests only;
production parse paths use `?`/`unwrap_or(false)` fail-closed.

## Research And G8

G8 preflight present (`sprint80_phase_A_preflight.md`), verdict
PLAN-ADAPT, fully read. S1a evidence (daemon `http.rs:504-516` proves the
merge+ServeDir topology), S1b (0 dep, tower-http cors+fs already at
workspace), S2/S3/S4 consistent. The 3 additive refinements
(Sec-Fetch-Site, session_secret, THREAT_MODEL amend) are exactly what the
code shipped. No new crypto/spec without grounding (re-uses existing
primitives). Factory-hors-daemon and pre-launch wire policy honored.

## Scope Cuts

Kickoff/plan Phase A = backend only, 0 daemon route, crate sbfb-factory
only, greenfield front deferred to Phase B. Diff respects all: no front
code, no daemon route, bundle dir intentionally absent (404 until Phase
B), no `allow_credentials(true)`. No scope cut touched.

## Codex verification

Pending — this is the driver-side pre-Codex review. Codex (`codex exec`,
GPT 5.5) must independently verify and convert PASS-PENDING → `## Verdict:
PASS`. Security delta for Codex focus: (a) confirm `.layer`-wraps-fallback
so ServeDir is truly auth-gated; (b) confirm `Sec-Fetch-Site` is the
correct cross-port discriminant vs `same-site`; (c) confirm cookie value
is `session_secret` not bearer on every path.

## Commit Body Draft

```
feat(sbfb-factory): Sprint 80 Phase A — Operator cookie auth + bootstrap + CSP

## Contexte
Phase A = prerequis backend BLOQUANT du front Operator greenfield (D4/D5):
auth cookie HttpOnly, bootstrap GET /?token, ServeDir derriere auth, CSP
self-origin. Crate sbfb-factory uniquement, 0 route daemon. Preflight G8
PLAN-ADAPT (2 P1 cross-port loopback fermes + amendement T-OPERATOR-CSRF).

## Fichiers
- crates/sbfb-factory/src/auth.rs : AuthState.session_secret per-boot,
  cookie fallback (Sec-Fetch-Site same-origin gate), cookie_value, +2 tests.
- crates/sbfb-factory/src/operator_server.rs : router bootstrap/authed,
  handle_bootstrap, operator_csp_middleware, OPERATOR_BUNDLE_SUBDIR, bundle param.
- crates/sbfb-factory/tests/operator_server.rs : +7 tests cookie.
- docs/security/THREAT_MODEL.md : T-OPERATOR-CSRF amende (P1-A + P1-B).

## Delta tests
Rust +9 (2 unit + 7 integration), nextest workspace 2004 passed / 0 skipped.
Frontend 0 (untouched, front greenfield = Phase B).

## Verification
fmt clean ; clippy -D warnings rc=0 ; nextest 2004/0 ; doctests rc=0 ;
release build rc=0. Frontend lint/tsc/unit/build/size/scan verts.

## Scope cuts
Backend only ; 0 route daemon ; crate sbfb-factory seul ; front Phase B ;
pas de allow_credentials(true). Aucun scope cut touche.

## G8 traceability
PLAN-ADAPT, 11 decisions figees implementees telles quelles ; 0
DESIGN-CONFLICT, 0 dep, 0 wire bump, Factory hors daemon tenu.

## Pre-launch protocol
0 wire format versionne touche ; cookie/CSP/?token = transport HTTP, pas
enveloppe serialisee ; TOKEN_HEX_LEN=64 intact.

## Codex verification
[a remplir par Codex — output brut codex exec]. Security delta : 2 P1
cross-port (Sec-Fetch-Site + session_secret distinct) fermes ; CSP
self-origin defense-en-profondeur ; ServeDir auth-gated.

## Carry closure
P2-1 (test bootstrap Host non-loopback 403) + P3-1..P3-4 routes vers
sprint80_verification.md / sprint81_audit_plan.md.
```

## Findings

- **P0 / P1: none.** Security logic (chicken-and-egg, cross-port guard,
  bearer non-leak, ServeDir behind auth, neutral bootstrap, CSP) is
  correct and matches the frozen design.

- **P2-1 — Branch-coverage gap on a security control (bootstrap Host
  check).** `handle_bootstrap` re-implements the loopback `Host` check
  (`operator_server.rs:298-306`, anti-DNS-rebind on the public route that
  bypasses the middleware) but its `403` branch has ZERO test coverage.
  Preflight decision #10 explicitly listed "bootstrap Host non-loopback ->
  403" as a test to add; it was omitted. Owner: driver. Trigger: add a
  `raw_get("/?token=...", "Host: evil.com\r\n")` → assert 403 test. Exit:
  test present + green. Recommend adding now (1 test, cheap) rather than
  carrying.

- **P3-1 — ServeDir-behind-auth wiring untested.** No integration test
  fetches an asset through the authed fallback (bundle is absent in the
  harness). The guarantee rests on axum's `.layer`-wraps-fallback
  semantics (verified correct) but is not pinned by a test; preflight
  decision #10 also suggested a fixture `index.html` to assert 200 vs
  "not-401". Carry to verification. Trigger: tempdir bundle fixture +
  asset GET with/without cookie. Exit: test green.

- **P3-2 — CSP not asserted on error responses.** `operator_csp_header_present`
  checks only a 200. A future layer reorder placing CSP inner of auth
  would not be caught for 401/403/404. Carry; add a 401-path CSP assert.

- **P3-3 — Doc accuracy: Sec-Fetch-Site on WS.** The comment claims
  Sec-Fetch-Site is "emitted on same-origin GET/SSE/WS requests"; for
  WebSocket handshakes this is browser-version-dependent (Chromium ~94+).
  Phase B live validation should confirm the terminal WS authenticates via
  cookie; if a target browser omits it on WS, the WS path must keep the
  header transport. Non-blocking doc nuance.

- **P3-4 — Weak unit proof of P1-B.** `session_secret_is_distinct_from_token`
  asserts a random secret ≠ a fixed `"b"*64` token (probabilistic). The
  real invariant (cookie carries session_secret) is pinned by the
  integration test (cookie ≠ TEST_TOKEN). Acceptable as-is; noted.

## Residual Risk

- **Trade-off (G4):** the cookie path now requires fetch-metadata
  (`Sec-Fetch-Site`), so pre-fetch-metadata browsers can authenticate only
  via the header transport — accepted (greenfield front targets modern
  browsers). The CSP `default-src 'self'` (no `unsafe-inline`) constrains
  the Phase B/E front to ship without inline scripts/styles; Motion/Tailwind
  v4 inline-style handling must be reconciled at Phase E (style-src
  hardening) — flag forward, not a Phase A defect.
- **Accepted residuals (THREAT_MODEL):** `?token` survives browser
  history/Referer post-303 (mitigated by no-referrer; local attacker is
  already T0/AD2); cross-port cookie ambient authority blocked by
  Sec-Fetch-Site + distinct session_secret. Both documented honestly.
- **Carry routing:** P2-1 (close in-phase recommended) + P3-1..P3-4 →
  `sprint80_verification.md` and `sprint81_audit_plan.md`, each with
  owner/trigger/exit above.

---

## Codex reconciliation

Codex (`codex exec`, GPT 5.5) raw output: `.planning/active/sprint80_phase_a_codex_review.md`.
Verdict Codex : **8/8 livrables CONFIRME, 0 GAP, 0 PARTIEL**. Codex a
lui-meme relance les tests (`auth::tests` 7 passed ; `--test
operator_server` 51 passed) et confirme independamment les 5 invariants
de securite (cookie compare au `session_secret` jamais au bearer ;
`Sec-Fetch-Site` chemin cookie uniquement ; ServeDir jamais
`repo_root_pub()` ; bootstrap hors `auth_required` ; 0 `use
nexus_shell_daemon*`, 0 dep ajoutee). Aucun GAP P0/P1/P2 a corriger.

**P2-1 ferme in-phase** (recommandation review) : test
`bootstrap_rejects_non_loopback_host` ajoute
(`tests/operator_server.rs`) — Host `evil.com` sur le bootstrap -> 403,
pas de Set-Cookie, et CSP presente sur le 403 (ferme aussi P3-2 : CSP
sur reponse non-200). Suites relancees apres ajout : `fmt --check`
clean, tests cookie cibles 10/10 PASS, workspace nextest vert.

**P3-1, P3-3, P3-4** : residus documentes acceptes (ServeDir-behind-auth
= semantique axum `.layer`-wraps-fallback confirmee par review + Codex ;
Sec-Fetch-Site sur WS = validation live Phase B ; preuve unit P1-B
epinglee par le test d'integration `cookie != TEST_TOKEN`). Routes vers
`sprint80_verification.md`.

## Conclusion (post-Codex)

Review driver PASS-PENDING + Codex 8/8 CONFIRME + P2-1/P3-2 fermes
in-phase = PASS (verdict promu en tete). Committable.
