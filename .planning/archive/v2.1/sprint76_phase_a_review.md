# Sprint 76 Phase A — Deep Adversarial Review

Date: 2026-06-15
Reviewer: deep-review fallback (`nexus-phase-review-deep` not registered)
Diff: uncommitted working tree (10 files, +594/-128)
HEAD: `3faee6e`

## Verdict: PASS

(PASS-PENDING promu à PASS après suites vertes dual-platform + Codex 8/8 —
cf. `## Codex reconciliation` en fin de fichier. Les 3 findings non-bloquants
ont été traités : P2 affichage L3 corrigé, P3 disarm testé, P3 string
pré-existant laissé hors-scope.)

No P0/P1 found after line-by-line scrutiny of all 10 axes. The change is a clean,
contained exposition+wiring of the mature consent engine, faithful to D1. Three
non-blocking items (1 P2 semantic-copy mismatch, 2 P3 test/cosmetic gaps) are
listed below; none blocks the commit. PASS-PENDING (not PASS) only because the
heavy suites (Rust nextest dual-platform + Vitest) are still running in parallel
and must land green before commit — code-level review is clean.

## Findings

| Sev | File:line | Description | Fix proposé |
|---|---|---|---|
| P2 | `web/src/pages/Network.tsx:943` | `OfferPowerCard` computes `sharing = level >= 2`, so a **Whitelist (L3)** user sees the copy "Ta machine contribue au calcul du réseau public". But D1 classes L3 (`Whitelist`) as **least-priv OFF** for the co-located worker (`user_public_consent` returns `None` for `Whitelist`, `local_worker.rs:154` — verified). The enrollment is correct; only the *display copy* mis-labels L3 as public sharing. Cosmetic, not behavioral. | Gate the "sharing public" copy on `level === 2 || level === 4` (the D1 public set), not `level >= 2`. L3 should read "tes projets choisis" rather than "réseau public". |
| P3 | `web/src/components/__tests__/GpuConsentDialog.test.tsx` | The disarm-on-level-change branch (`onValueChange` → `setConfirmingAll(false)`, `GpuConsentDialog.tsx:191-194`) has no test. Axis 7 asks this be proven. The arm + direct-save + confirm-save paths ARE covered. | Add a test: render L4, click save (arms `consent-confirm-all`), change level to 2 via the radio, assert `consent-confirm-all` is gone and a save POSTs directly. |
| P3 | `web/src/pages/Network.tsx:442` | Pre-existing string "la allowlist du worker" (should be "l'allowlist"). NOT introduced by this diff (untouched line). | Opportunistic fix while on the page, or leave; out of scope. |

## Axis-by-axis scrutiny

1. **SCHEMA_VERSION=1 additive** — PASS. `#[serde(default, skip_serializing_if = "Option::is_none")] consent: Option<ConsentSnapshot>` (`state_writer.rs:352`). The module doc-comment (`state_writer.rs:22-29`) explicitly authorizes additive optional fields staying on the same version. `consent_snapshot_serializes_additively` proves: (a) absent → key omitted (not `null`), (b) `schema_version == 1`, (c) a legacy `state.json` with no `consent` key deserializes to `None`. SCHEMA_VERSION literal unchanged (`= 1`, line 54). The legacy-decode test is legitimate runtime-tolerance (not a zombie redefinition test) and is labeled as such.

2. **Enrollment semantics D1** — PASS. `user_public_consent` (`local_worker.rs:145-156`) returns `Some((level, caps))` ONLY for `OpenSource`/`All`, `None` for `OwnProjects`/`Whitelist` (exhaustive match). The provision path copies **only `level` + `caps`** (`local_worker.rs:123-126`); `own_node_id` stays `"local-worker"` and the own-doc whitelist floor (`allowed_project_ids.insert(project_id)`) is inserted BEFORE the override and survives it. `colocated_worker_honors_user_consent_when_public` asserts `own_node_id == "local-worker"` AND the doc id is still whitelisted after an `All` override. `colocated_worker_least_privilege_when_off` asserts a foreign L3 whitelist entry is NOT inherited. Both branches proven.

3. **Fail-closed / path consistency** — PASS. Write path (`set_consent` → `save_consent(state.sbfb_home.as_deref())` → `consent_path = override.or_else(auth::sbfb_home).join("consent.json")`) and read path (`user_public_consent` → `user_sbfb_home.or_else(auth::sbfb_home).join("consent.json")`, with `user_sbfb_home = state.sbfb_home.clone()` from `http.rs:3243`) resolve to the **identical file**. Absent file → `ConsentConfig::load_or_default` returns default L1 `OwnProjects` (`consent.rs:203`) → `user_public_consent` matches `OwnProjects` → `None` → least-priv floor kept. A malformed/out-of-range file → `from_slice` Err → `.ok()?` → `None` → least-priv. No accidental opening. **Cross-type compat verified**: daemon writes `shell-daemon::consent::ConsentConfig` (`level: u8`, `caps.max_vram_mb: Option<u32>`, extra `level_threat_note`/`residual_threats_acknowledged`); worker-core reads `nexus_worker_core::consent::ConsentConfig` (`level: ConsentLevel` via `try_from="u8"`, `max_vram_mb: Option<u64>`, no `deny_unknown_fields`). Bare-int level, u32→u64 widening, unknown-field tolerance, and Vec→HashSet all deserialize cleanly. Byte-compatible.

4. **Concurrency `consent_snapshot()` `try_lock`** — PASS, safety argued. `flush_state_snapshot()` and the claim pump (`tick()` → `usage.lock().await`) run on the **same single task** inside one `tokio::select!` loop (`runtime.rs:680-714`), serialized — `flush` runs only after `tick().await` returns and has released the usage lock. `try_lock` therefore virtually always succeeds; it can **never deadlock** (non-blocking, same-task). The `unwrap_or(0.0)` fallback is honest (rare contention miss → 0h for one tick, next flush recovers). The in-code comment (`runtime.rs:312-316`) states this accurately. `current()` returns `ConsentResult<ConsentConfig>`; `.ok()?` correctly leaves the snapshot field `None` if the watcher lock is poisoned.

5. **Wire-contract front** — PASS. `WorkerStateV1Schema` is a plain `z.object()` (non-strict; the `.strict()` at coordinator.ts:303 belongs to a different schema) → unknown keys stripped, additive `consent` parsed. `consent: ConsentSnapshotSchema.nullable().optional()` tolerates both absent-key (Rust `skip_serializing_if` omits) AND a defensive `null`. `ConsentSnapshotSchema` field shapes match Rust: `level z.number().int().min(1).max(4)` (Rust `u8` 1..=4), cap fields `.nullable().optional()` (Rust `Option` + `skip_serializing_if`), `hours_used_today z.number().nonnegative()` (Rust `f64`, always present). No drift.

6. **Route reconciliation** — PASS. Daemon mounts `/api/v1/consent` (GET, `http.rs:423`), `/api/v1/consent/set` (`:424`), `/api/v1/consent/whitelist/add|remove` (`:426-431`). Front `consent.ts` now posts to all four prefixed paths (GET = `/api/v1/consent`, NOT `/consent/get`). `consent.test.ts` pins all four URLs. THREAT_MODEL cell updated to the prefixed paths. The pre-existing latent bug (un-prefixed → SPA GET fallback → inert) is correctly fixed in-phase, in `consent.ts` (not by widening the Vite proxy, per preflight guidance).

7. **Double-confirm L4** — PASS. First click at `level === 4 && !confirmingAll` arms `confirmingAll` and returns with NO POST (`GpuConsentDialog.tsx:169-172`); second click calls `doSave()`. `GpuConsentDialog.test.tsx` "L4 exige une double confirmation" asserts first click → `consent-confirm-all` present + `fetchMock` NOT called, then second click → exactly one POST to the prefixed route. "un niveau < 4 enregistre directement" asserts L2 POSTs on the first click with no confirm banner. Disarm-on-level-change branch present but untested (P3 above). On save failure, `confirmingAll` stays armed (dialog open, retry goes straight to `doSave`); on success the dialog closes and re-mounts via `key` → state resets. No stuck-state bug.

8. **Strings FR (scan-en-strings)** — PASS. `scan-en-strings.sh` only flags a narrow EN word list (Welcome/Dashboard/…) — none present in the new strings. No stray CJK (grep `[\x4e00-\x9fff…]` over all four changed front files → NO_CJK_FOUND; the "直接" coquille is gone). Accents in new strings ("réseau", "données aujourd'hui", "privée") are proper UTF-8, no mojibake.

9. **Scope cuts respected** — PASS. No BOINC idle/day-of-week scheduler. No separate `worker_enabled: bool` (the level IS the state — `OfferPowerCard` derives sharing from `level`). No HTTP endpoint added to the worker binary (Sprint 5 D3 preserved — the snapshot field feeds the existing file contract; the panel reads `consent` from `/api/v1/worker/state`). No self-test/benchmark of enrollment. No `model_digest`/`logprobs_hash`/validator/quorum touched (those are C/D). 0 dep added/bumped. 0 wire `*_VERSION` bump.

10. **Tests assert behavior** — PASS (with the one P3 disarm gap). Rust: additive serialization (key omitted + legacy decode), level+usage carried through flush, public-honor vs least-priv-off enrollment branches (both with own-doc-floor and own-identity assertions). Vitest: 4 route URLs pinned, L4 arm-then-confirm, L2 direct-save, caps-gauge render ("3.5 h / 12 h"), intention-CTA (no `consent/set|kind|provider` jargon). Semantic branch coverage is real, not "it compiles".

## Branch coverage sémantique

- **Additive vs legacy** (state_writer): covered — absent-key omit + legacy `state.json` decode + populated round-trip.
- **Public vs least-priv enrollment** (local_worker): covered — `All`→adopt level+caps+keep floor; `Whitelist`→stay own-doc, reject foreign whitelist. `OpenSource`/`OwnProjects` share the same code arms (matched exhaustively) but are not each separately tested — acceptable (the two tested arms exercise both the `Some` and `None` sides of `user_public_consent`).
- **Snapshot concurrency** (runtime): same-task serialization makes `try_lock` deterministic in practice; the `unwrap_or(0.0)` miss path is unreachable in the single-task loop and is defensive only — no test needed.
- **L4 confirm vs direct** (dialog): covered both sides. **Disarm-on-level-change: NOT covered** (P3).
- **Route prefix** (consent client): all 4 endpoints pinned.
- **Caps-gauge presence**: gauge renders only when `consent && hoursCap !== null`; the `hoursCap === null` (no-cap) hidden path and the `level === undefined` loading path are exercised implicitly but not asserted — acceptable for a display card.

## Scope cuts

Honored in full (kickoff §4 D1 "Rejeté"): no idle/day-of-week scheduler, no
orthogonal `enabled` flag, no worker HTTP endpoint, no enrollment self-test.
No primitive rewritten — `should_accept_task` and the caps/fail-closed gate are
untouched (0 logic change). The phase stays exactly within the D1 envelope:
front exposition + additive snapshot field + co-located enrollment bascule +
route reconciliation.

## Recommendation

Commit-ready pending green suites. Address the P2 copy mismatch (`level >= 2` →
`level === 2 || level === 4`) before commit for D1 display fidelity — it is a
one-line change with no behavioral risk. The two P3s can be folded in or carried.

## Codex reconciliation

Codex GPT 5.5 (`codex exec`, cross-model) sur le code final :
`.planning/active/sprint76_phase_A_codex_review.md` (output BRUT, non
réécrit). **8/8 livrables CONFIRME, 0 GAP, 0 PARTIEL.** Codex a vérifié
indépendamment : rétro-compat additive (`SCHEMA_VERSION`=1, legacy-decode),
enrôlement `level`+`caps` only (`own_node_id` user non copié, floor own-doc
survit, consent absent → least-priv), routes `/api/v1/consent*` match exact,
double-confirm L4 (arme sans POST au 1er clic + désarmement), tests à
assertions utiles. Codex a lu le code POST-refactor (cite
`level === CONSENT_LEVEL.ALL`).

Findings review traités avant commit :
- **P2 affichage L3** : corrigé — `sharing` ne couvre plus que les niveaux
  publics réels (`PUBLIC_SHARING_LEVELS = [OpenSource, All]`), pas L3.
- **P3 disarm** : test ajouté (`désarme la confirmation L4 quand on change
  de niveau`).
- **P3 string pré-existant** (`la allowlist`) : laissé hors-scope (ligne non
  touchée par la phase).

Suites relancées vertes après les corrections : Vitest 386, Rust nextest
Windows 1767 / Docker subset 582, coverage + size + scan OK. **Verdict final
PASS, committable.**
