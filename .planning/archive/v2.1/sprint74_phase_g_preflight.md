# Sprint 74 Phase G Preflight (G8)

Date: 2026-06-08
HEAD: `66a9409` (Phase F landed)
Verdict: **EXECUTE** (wrap-up + dette ; carries pre-cartographies ; scope cuts re-routes G.3)

> Phase G is the wrap-up/dette phase (`docs(sprint74)` per plan §G), not a new
> feature surface. Its "implementation" is closing pre-identified carries +
> writing planning/doc artifacts. The factual G8 scan below was performed by a
> 6-agent mapping Workflow (read-only Explore) that produced, per carry, the
> current code state (file:line), the exact minimal fix, and the risk — i.e. the
> S1-S4 evidence is the mapping output, recorded here.

## S1 — SOTA / dependency delta
- **clean**. Zero new crate, API, or spec. Every treated carry composes existing
  primitives: validator quorum logic (B.2), the `isHttpsUrl` helper (B.5), React
  Query `isError` (SEARCH-VIEW), the existing `BrowseEntry.node_id` field
  (KEEP-ONLINE is_own, a serialize-only derived flag). T14 is test + config only
  (Vitest). No transitive graph change vs `66a9409`.

## S2 — Historical decisions traversed
- **clean (no reversal)**. The carries originate from the S73 audit
  (`archive/v2.1/sprint73_audit_findings.md`) and the Phase D/E reviews — already
  analysed, not new debate. KEEP-ONLINE is_own uses the #6 `node_id` field as
  intended (the precise "did this node publish it" signal). T14 thresholds: the
  S9 "temporary lowering" is made the honest enforced baseline (functions 90->85
  documented, BrowsedProject = Playwright-tested page).

## S3 — Threat model coverage
- **clean (covered)**. is_own adds NO surface (a derived boolean, daemon-side,
  serialize-only). B.2 makes a zombie task terminal (strictly safer). B.5 closes
  an XSS anchor (feed-sourced repo_url, React unsanitised href) — defensive.
  query.isError fixes a UX hang, not a security path. THREAT_MODEL updated this
  phase: §5.4 iroh 0.97->0.98, NEW §15 "Surface seed cross-noeud" (Phase E+F
  STRIDE table, over-count M residual), §11 D.1 reframe.

## S4 — Protocol / wire invariants
- **clean (0 bump)**. is_own is a daemon->shell JSON flag (a flatten view on the
  `/browse` response), NOT P2P wire; no `*_VERSION`/`DOMAIN_*` bump. The one
  carry that WOULD touch a wire payload — FRESHNESS-RELEASE-UNINDEXED
  (`ReleasePublishedPayload` +project_name/category) — is deliberately
  **RE-ROUTED to S75** (`sprint75_audit_plan.md`) precisely to avoid a 16-literal
  wire-payload change inside the wrap-up commit (G.3 permits re-routing).

## Risks / scope cuts
- Blocking risks: none (verdict EXECUTE).
- Scope cuts (re-routed to S75 per G.3): FRESHNESS-RELEASE-UNINDEXED,
  KEEP-ONLINE-HASH-SOT (inert without GC), invite single-use re-credit,
  E.3/H.2/genuinely-shared-blob/R6-DB-error tests, search q/offset clamp.
- Env note (non-code): the iroh-networked Rust tests hang this session on a
  degraded host network (relay UP, socket/holepunch host-side) + Docker engine
  500 after a `wsl --shutdown` recovery attempt — environmental, not a code
  regression (same base as Phase F dual-platform green; Phase G touches no iroh
  path). Full dual-platform fail-fast to be re-run after env recovery.

## Action
- EXECUTE Phase G: close the treated carries + write the doc/planning artifacts.
- Commit body MUST cite this preflight (G8 traceability) + the re-route list.
- No wire bump; is_own rides the unchanged `/browse` JSON as an additive flag.
