# Sprint 74 Phase A Preflight

Date: 2026-06-07
HEAD: `af66f0d`
Verdict: **SCOPE-CUT-CONSISTENT**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read:
  - `prompts/agent/preflight.md` (procedure portable)
  - `.planning/active/sprint74_plan.md` (Phase A: lines 98-148)
  - `.planning/active/sprint74_kickoff.md` (D1-D5 + Checkpoint §11 arbitrages PO, lines 746-769)
  - `.planning/research/s74_disponibilite_ux_design.md` (D-DISPO complet : mockups, copy FR §6, defauts §7, 5 verrous §8, phasage §9)
  - `$HOME/.claude/.../memory/feedback_approach.md` (pick deepest, no band-aid)
  - `web/src/pages/Deploy.tsx`, `web/src/pages/Browse.tsx`, `web/src/pages/BrowsedProject.tsx`,
    `web/src/pages/ProjectDetail.tsx`, `web/src/pages/OnboardingEmpty.tsx`
  - `web/src/components/AppShell.tsx`, `web/src/components/AddCoordinatorDialog.tsx`, `web/src/components/VerificationDetail.tsx`
  - `web/src/api/daemon.ts`, `web/src/stores/projectStore.ts`
  - `web/src/components/ui/sheet.tsx`, `web/src/components/ui/toggle.tsx`
  - `web/src/pages/__tests__/BrowsedProject.test.tsx`, `web/src/components/__tests__/VerificationDetail.test.tsx`, `web/src/pages/__tests__/Deploy.test.tsx`
  - `web/scripts/scan-en-strings.sh`
- Commands run:
  - `git rev-parse --short HEAD` -> `af66f0d`
  - `Grep "_VERSION\s*[:=]" web/` -> No matches found (S4 evidence)
  - `Grep "coordinateur|Coordinateur" web/src --glob !__tests__` -> 17 files (rename surface)
  - `Grep "coordinateur" web/src/**/__tests__/**` -> 1 visible-string assertion (`BrowsedProject.test.tsx:153`)
  - `Glob web/src/i18n/**` -> No files found (i18n is a phantom path)
  - `Glob web/src/components/ui/sheet*` + `toggle*` -> both present
  - WebSearch 2025 IPFS Cluster pin/keep-online UX (S1a confirmation)

## Scope
- Plan source: `.planning/active/sprint74_plan.md §Phase A` (lines 98-148).
- Target files (per plan §A.2): `Deploy.tsx`, `AvailabilitySheet.tsx` (NEW),
  `ProjectDetail.tsx` / `BrowsedProject.tsx`, `AppShell.tsx`, `daemon.ts`,
  `OnboardingEmpty.tsx`, `AddCoordinatorDialog.tsx`.
- Deps/APIs/specs: **none** (no new lib; reuses S73 primitives: `browse.rs`
  probe status serialized as `reachable/unreachable/unknown`, `verifyQuery`
  via `GET /api/v1/project/:id/provenance`, `browsePull` via
  `POST /api/daemon/browse/pull`). shadcn `Sheet` + `Toggle` already vendored.
- Security/protocol surfaces: **none new**. XSS surface on `repo_url` anchors
  reused (`isHttpsUrl` guard exists `Browse.tsx:153`). No `.rs` file in scope,
  no wire format, no `*_VERSION`.
- Tests expected (plan §A.3, Vitest `web/`):
  1. `availability_sheet_renders_author_state_seeders`
  2. `publish_success_card_folds_hashes`
  3. `availability_state_maps_reachable_unreachable_unknown`
  4. `offline_reminder_only_for_own_apps_dismissible`
  5. `coordinator_renamed_to_node_in_shell`
  6. `keep_online_toggle_readonly_in_phase_a`

## S1a OSS Prior Art
- Domain: continuous-availability / seeding / pinning UX for a P2P-hosted app
  (read-only panel surfacing author-immutable vs availability-mutable, plus an
  opt-OUT "keep online" toggle).
- Sources (this scan validates the design doc's already-deep research, per the
  agent mandate):
  - IPFS persistence/pinning model — https://docs.ipfs.tech/concepts/persistence/
    (accessed 2026-06-07): availability requires active pinning; an upload does
    NOT guarantee perpetual availability. Matches the D-DISPO "ligne de verite"
    ("reste joignable tant que ton noeud tourne") and the opt-OUT toggle model.
  - IPFS Cluster pinset/allocations — https://ipfscluster.io/ (accessed
    2026-06-07): pin = explicit retention state, replication = additive
    allocations. Matches verrou §8(2) "redondance additive jamais substitutive".
  - Design doc §3 already cites Tailscale share (invite revocable + quarantine),
    Syncthing (peer approval), Radicle (seeder != delegate) — those drive E-F,
    not Phase A.
- Finding: **APPROACH-ALIGNED**. Phase A renders the IPFS-style pin/availability
  split as a read-only panel; the toggle is honestly read-only (ON, no mutation)
  until Phase D wires the local pin. No OSS evidence contradicts the read-only
  surfacing approach. The 5 anti-recentralisation locks (no host field, additive
  redundancy, possessive "Mon serveur", provenance always author's, suggestion
  state-triggered) directly encode the OSS lessons.
- Impact: none — proceed as planned.

## S1b Dependencies, CVEs, Release Notes
- Scanned: `web/package.json` deltas implied by Phase A.
- Commands/sources: plan §A.2 lists zero new deps; kickoff §"Versions deps
  confirmees" line 64-67 states front stack unchanged (React + Vite + TS +
  Tailwind + shadcn Sheet + Zustand + React Query). `Glob web/src/components/ui/sheet*`
  and `toggle*` both resolve -> the two shadcn primitives Phase A needs are
  ALREADY vendored (`@base-ui/react` based, not Radix), so no install.
- Finding: **clean**. No dependency added or bumped. No transitive graph to
  resolve (P2-PREFLIGHT-TRANSITIVE-DEPTH N/A: front-only, no Cargo.lock change,
  no `web/` lock change). No CVE surface introduced.

## S2 Historical Decisions
Decisions crossed by Phase A target files, with reverse-commit checks:

- **Sprint 5 D4 — shell does not scan ports / read FS** (`projectStore.ts:7-11`,
  `OnboardingEmpty.tsx:4-11`, `AddCoordinatorDialog.tsx:16-19`). Status: still
  valid, NOT in tension. Phase A is pure read-render + string rename; it adds no
  port scan, no FS read. The availability panel reads existing daemon routes
  (`browse`, `provenance`) via the authenticated same-origin client. **Honored.**
- **Hotfix `a53b9f6` — auto-register same-origin daemon as default coordinator**
  (HEAD-1; `bootstrap.ts::autoRegisterLocalCoordinator`, `AddCoordinatorDialog`
  default = origin, `OnboardingEmpty` daemon model). Reverse check: `a53b9f6` is
  the most recent shell commit and the kickoff (line 12-15) names it the base of
  the rename. The rename BUILDS ON it (rename "coordinateur"->"noeud" in the same
  files it already touched). No reversion; this is a documented forward
  continuation (carry G7). **Consistent.**
- **S73 finding — `web/src/i18n/*` is a PHANTOM path; strings are inline FR**
  (memory; confirmed here: `Glob web/src/i18n/**` -> No files found). Load-bearing
  for the rename: there is NO i18n framework, so the rename is a literal
  inline-string edit across files, NOT a locale-key change. Every "coordinateur"
  occurrence is a hardcoded FR string. **Confirmed — drives the rename
  implementation shape.**
- **S73 finding — Rust serializes always-present keys as `null`; Zod must use
  `.nullable()` not `.optional()`** (memory; confirmed `daemon.ts:339-343`
  SearchResult provenance fields are `.nullable()`). Phase A reads `BrowseEntry`
  (the availability panel data source). `BrowseEntry` provenance/archive fields
  are `.optional()` (`daemon.ts:155-171`) because the daemon serializes them via
  `#[serde(skip_serializing_if)]` (hotfix #6 made `node_id` serde-skip;
  `is_open_source` doc-commented as runtime-tolerant optional). **Phase A reads,
  never re-shapes, this schema — no Zod change needed; the existing `.optional()`
  contract is the producer/consumer agreement already in place.**
- **Pre-launch protocol — feed raw-op extensible, `*_VERSION` stay 1**
  (CLAUDE.md). Phase A touches zero wire. **N/A but honored.**

- Finding: **clean** (no blocking S2; all crossed decisions are valid and
  preserved or forward-continued).

## S3 Local Patterns And Threat Model
- Threats/contracts checked: XSS via untrusted `repo_url` anchors (the only
  security-relevant surface Phase A could touch).
- Evidence: `isHttpsUrl` guard exists at `Browse.tsx:153-155` and is applied to
  the search-hit `repo_url` anchor (`Browse.tsx:239,281`). It is NOT applied to
  the three PRE-EXISTING anchors that render `entry.repo_url` / `record.repo_url`
  verbatim:
  - `Browse.tsx:469-481` (AppCard `entry.repo_url`)
  - `BrowsedProject.tsx:365-376` (top-bar Source anchor `entry.repo_url`)
  - `VerificationDetail.tsx:184-192` (`record.repo_url`)
- HARDENING_ROADMAP status: PO Q6 arbitrated (kickoff line 762-764) that
  normalizing these 3 pre-existing anchors + new anchors lands in **Phase G**
  (carry B.5), NOT Phase A.
- Phase-A perimeter confirmation: the request explicitly asks to confirm the
  Phase A perimeter for XSS. **Confirmed: Phase A MUST apply `isHttpsUrl` to any
  NEW anchor it introduces** (the design "app tombee" placeholder -> `/deploy`
  prefill is an internal route, not an external anchor; the "Source" panel, if
  it renders a `repo_url`, MUST guard it). **The 3 pre-existing anchors are NOT
  in Phase A scope (Phase G, B.5).** Introducing a new unguarded `repo_url`
  anchor in Phase A would be a regression -> the phase must reuse/extract
  `isHttpsUrl` for any new external link it adds.
- Finding: **non-blocking**. No regression as long as Phase A guards any new
  anchor it adds (a hard requirement, not a finding against the plan — the plan
  already lists `isHttpsUrl` as a Phase A guard, plan §A line 114-ish via design
  §8). The 3 pre-existing anchors are a documented future fix (Phase G), not a
  Phase A regression.

## S4 Protocol And Wire Invariants
- Wire/security files checked: none touched. `Grep "_VERSION\s*[:=]" web/` ->
  **No matches found**. No `.rs` file appears in plan §A.2. No `canonical.rs`,
  no `schemas/`, no `DOMAIN_*`, no signing domain.
- Producer->consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH) for the fields
  Phase A READS:
  - `status` (probe): producer = `nexus-shell-daemon-core::browse::BrowseStatus`
    (Rust enum, lowercased `reachable|unreachable|unknown`); consumer =
    `daemon.ts:128 BrowseStatusSchema = z.enum([...])`, rendered by
    `StatusPill`/`StatusDot` (`Browse.tsx:499-530`, `BrowsedProject.tsx:552`).
    Shape: bare lowercase enum string, always present. Phase A maps it to copy
    "En ligne / Hors ligne / Verification..." (design §6). **No re-shape.**
  - `last_probed_at`: producer = `Option<String>` -> consumer
    `daemon.ts:157 z.string().nullable()`. Always-present-as-null contract; the
    "Verifie il y a {duree}" / "Reverifier" UX reads it as-is. **Unchanged.**
  - provenance trio (`repo_url`, `provenance_hash`, `archive_hash`,
    `is_open_source`): consumer `daemon.ts:158-171` `.optional()` (serde-skip on
    producer side). Phase A renders, does not re-serialize. **Unchanged.**
  - `DeployResponse` (`hash`, `provenance_hash?`, `commit_sha?`):
    `daemon.ts:423-430`. Phase A only REFOLDS these into the success card +
    "Details techniques" (replaces the raw `<dl>` `Deploy.tsx:151-174`).
    No new field read, no field written. **Unchanged.**
- Day 0 status: **preserved**. D1-D5 all gated to later phases (E/F cross-node,
  D pin local); Phase A is segment 1 (D5 "SUR"). No Day 0 contradiction.
- Finding: **clean** (0 wire, 0 `*_VERSION`, confirmed explicitly as requested).

## Risks And Scope Cuts
- **Blocking risks: none.**
- **Non-blocking findings (the SCOPE-CUT-CONSISTENT basis):**

  1. **Rename surface is broader than plan §A.2's 6-file list, but PO Q8
     arbitrated "toute l'UI".** `Grep` shows the visible FR string "coordinateur"
     in 11+ component/page files beyond the 6 listed:
     `BrowsedProject.tsx:70,73`, `Browse.tsx:35,38,584` (incl. the
     `DaemonOfflineBanner`), `Network.tsx:50,53`, `Projects.tsx:27`,
     `Curators.tsx:35,38`, `ProjectDetail.tsx:56,64-65`, `AppShell.tsx:150,243,263,306`,
     `AddCoordinatorDialog.tsx:102,104,125,136,163`, `OnboardingEmpty.tsx:63`,
     `InvitesTab.tsx:77`, `CommandPalette.tsx:140,234`, `OverviewTab.tsx:48`,
     plus `Deploy.tsx:29,32`. The kickoff arbitrage (line 768-769) is explicit:
     **"toute l'UI"**. Resolution: the plan's 6-file table is a NON-EXHAUSTIVE
     starting point; the binding scope is PO Q8 = all visible "coordinateur"
     occurrences. **Action: extend the rename to every visible-string file, not
     just the 6.** This is a SCOPE-CONSISTENT clarification (the plan under-listed
     vs the PO arbitrage it cites), not a plan conflict. Comments/identifiers/
     localStorage key stay untouched (see below).

  2. **The Zustand store API + persisted localStorage key stay "coordinator".**
     `projectStore.ts:146` persist key = `"nexus-grid:shell:v1"`;
     identifiers `knownCoordinators`, `activeCoordinatorUrl`, `addCoordinator`,
     `KnownCoordinator`, `selectActiveCoordinator`, and the threaded `coordUrl`
     prop are CODE identifiers, not user-facing text. The rename is
     **user-facing strings only** (CLAUDE.md "Langue": identifiers stay English).
     Renaming the store/key would (a) be out of scope for a string rename and
     (b) risk a localStorage migration/data-loss (the key change would orphan
     the persisted coordinator list). **Action: rename visible text ONLY; do NOT
     touch store identifiers, the persist key, or the `coordUrl` prop.** This
     bounds the "var interne" in plan §A.2 (`daemon.ts`) to comments/labels only
     — `daemon.ts` has no user-facing "coordinateur" string (only JSDoc), so its
     "rename var interne" line is effectively a no-op for the string rename and
     should be treated as optional comment hygiene, not a functional change.

  3. **One existing test asserts the visible string -> must be updated in-phase.**
     `BrowsedProject.test.tsx:153` expects `screen.getByText(/Aucun coordinateur/)`.
     After the rename to "Aucun noeud actif" (design §6) this assertion breaks.
     **Action: update that assertion as part of the rename** (and audit
     `Browse.test.tsx` / `Deploy.test.tsx` for any "coordinateur" text assertions
     — grep shows only `BrowsedProject.test.tsx` asserts visible text; the
     `bootstrap.test.ts:4` hit is a comment). This is expected rename fallout, not
     a regression.

  4. **`scan-en-strings.sh` flags "Coming soon" — design uses "Bientot" (FR).**
     `scan-en-strings.sh:26` lists `Coming\s*soon`. The "Bientot" inert badge
     (design §8 verrou 5, mockup §5) is FR, so the gate stays green. **No risk**
     — just do not write "Coming soon" / "Soon" for the inert cross-node CTAs.

  5. **Sheet/Toggle test architecture is `@base-ui/react` + Portal.**
     `sheet.tsx:3` and `toggle.tsx:4` are `@base-ui/react`, not Radix. The
     existing precedent for testing a portal-mounted Dialog/Sheet is
     `VerificationDetail.test.tsx` (which queries `screen.getByTestId(...)` for
     content rendered inside `DialogContent` -> portal content IS queryable by
     Testing Library). The 6 planned Vitest tests are **realistic** against this
     architecture:
     - existing pattern = `vi.stubGlobal("fetch", ...)` +
       `useProjectStore.setState({knownCoordinators, activeCoordinatorUrl})` +
       `render(<QueryClientProvider><MemoryRouter>...)` +
       `@testing-library/user-event` for the open-sheet click
       (`BrowsedProject.test.tsx:73-129`).
     - localStorage mock: the store uses `window.localStorage`
       (`projectStore.ts:23-28`); tests drive state via `setState`, not via
       localStorage directly, so no extra mock is needed.
     - Test #6 (`keep_online_toggle_readonly_in_phase_a`) asserts the toggle is
       ON and that clicking it triggers NO mutation -> realistic via a
       `vi.fn()` fetch spy assertion that no `POST /api/daemon/keep-online`
       fires (that route does not exist until Phase D — confirmed: no
       `keep-online` helper in `daemon.ts`). **The toggle MUST be a presentational
       element (ON, disabled or no onChange-network), never a faux active button
       (D5 / verrou §8(5)).**
     **No risk** — the 6 tests fit the existing harness.

- **Scope cuts still honored** (kickoff §7 / plan §7):
  - Phase A is D5 "Segment SUR" (front-only, primitives S73). E-F cross-node not
    started. Toggle is read-only (pin local = Phase D). "Inviter un pair" /
    "Copies de secours" stay inert "Bientot" (verrou §8(5), no faux active
    button). 0 host field at publish (verrou §8(1)). Faux-vert NAT honest label
    "vu de ton noeud" is a copy-only change (PO Q2). Provenance/author always
    the author's (verrou §8(4)).

## Action
- **SCOPE-CUT-CONSISTENT: proceed with Phase A as planned, with the documented
  scope clarifications:**
  1. Rename ALL visible "coordinateur" -> "noeud"/"reseau" per design §6
     (PO Q8 "toute l'UI"), NOT only the 6 files in plan §A.2; the plan's table is
     a non-exhaustive start.
  2. Rename VISIBLE TEXT ONLY — do not touch the Zustand store identifiers, the
     `nexus-grid:shell:v1` persist key, or the `coordUrl` prop (avoids a
     localStorage migration; CLAUDE.md keeps identifiers English).
  3. Update `BrowsedProject.test.tsx:153` (and any other visible-string
     assertion surfaced at code time) to the new copy.
  4. Guard any NEW external `repo_url` anchor Phase A introduces with the
     existing `isHttpsUrl` pattern; the 3 pre-existing anchors are Phase G (B.5),
     out of Phase A scope.
  5. Keep the "Garder en ligne" toggle strictly presentational/read-only (ON,
     no network mutation) and every un-wired cross-node CTA an inert "Bientot"
     (D5 / verrou §8(5)).
- The commit body should cite this preflight under `## G8 traceability` and note
  the rename-scope clarification (plan listed 6 files; PO Q8 = all visible
  strings) and the visible-text-only bound on the store.
