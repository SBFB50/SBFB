# Sprint 55 — Design Review

## Scoring
D1 ✅, D2 ⚠️, D3 ✅, D4 ⚠️  
Rigor signal: 2/4

---

## D1 — Woodpecker serveur Docker Compose + Caddy auto-TLS

**Status**: ✅ Source récente + alternatives vérifiées

**Sourcing**:
- Context7 woodpecker-ci/woodpecker 2026-05-07 : v3.14.0 stable, Docker Compose deployment standard, Caddy reverse proxy recommandé (auto Let's Encrypt TLS).
- Codebase state: .woodpecker/ci-linux.yml exists (commit be633c3 Phase D), VPS infra ready (Docker 29.4.3, 8GB RAM, images pinned SHA256, deploy-key GitHub present).

**Alternatives verification**:
1. **Nginx + Certbot** → WebSearch 2026 confirms Caddy eliminates certbot cron (operational failure point causing ~2 hours/year downtime). For single-service reverse proxy, unnecessary complexity. ✓ Rejected fairly.
2. **Woodpecker binaire natif** → Docker Compose is the community standard; native binary complicates updates and rollback. ✓ Rejected fairly.
3. **Traefik au lieu de Caddy** → WebSearch 2026 confirms Caddy Caddyfile (3 lines for basic reverse proxy) vs Traefik YAML/TOML (15+ lines, labels-based Docker integration not needed here). Caddy automatic HTTPS is native, Traefik requires extra config. ✓ Rejected fairly.
4. **Forgejo Actions** → Would require full Forgejo instance; Woodpecker already configured (S52 pipeline, S54 images). ✓ Rejected fairly.

**Assessment**: Caddy + Docker Compose is state-of-the-art 2026 for this scope. All alternatives properly evaluated against current docs.

---

## D2 — GHA validation via push + run documentation

**Status**: ⚠️ Source non vérifiable + alternative workflow_dispatch non comparée techniquement

**Sourcing issues**:
- Kickoff cites: "fix nexus-core-py est deja commite (S54 Phase D)". Commit be633c3 (2026-05-07) lists "CI infra images pin + Rust CI fix" but grep --grep="nexus-core-py" yields no visible commits in last 20. Either rebase/squash occurred post-kickoff, or reference is stale.
- Wire format claim: "documenter le run ID GHA dans le commit body" — no existing convention found in codebase (phase A artifact).

**Alternative workflow_dispatch comparison**:
- Kickoff dismisses as "complexité supplementaire sans bénéfice".
- WebSearch 2026 finds: push and workflow_dispatch both consume GHA minutes identically. No cost or performance delta. Manual trigger via workflow_dispatch doesn't alter the validation correctness.
- **Gap**: Decision lacks technical grounding. The rationale "push est automatique" is process preference, not architectural. workflow_dispatch would be equally valid, with explicit triggering control.

**Assessment**: Decision is pragmatic (auto-trigger is operationally simpler) but lacks rigorous alternative comparison. The nexus-core-py fix status cannot be verified from current HEAD.

---

## D3 — LT-7 build executor tmpdir MVP + dispatcher routing + quorum SHA256

**Status**: ✅ Source design doc solide + alternatives techniquement vérifiées

**Sourcing**:
- Design doc SELF_HOSTED_BUILD.md §8 MVP scope is detailed (origin S52 Phase B design).
- Kickoff enriches: metadata BTreeMap wire format (task_type="build", keys build.repo/commit/binary/target/source_date_epoch), zero wire format bump (pre-launch policy respected).
- Codebase ready: dispatcher.rs and validator.rs exist in nexus-coordinator-rs; task_type field already accepts arbitrary strings (confirmed in kickoff).

**Rust reproducible builds verification**:
- WebSearch: rust-lang/rust#129080 tracks reproducibility; 62/81 subtasks complete as of latest.
- SOURCE_DATE_EPOCH + remap-path-prefix are recognized reproducible-builds.org standards.
- x86_64-linux homogeneous coverage ~95% (design doc says "MVP accepts 5% false negatives").
- Cross-platform (Linux/macOS/Windows) explicitly called out as unsolved upstream → MVP x86_64-linux scope cut is justified.

**Alternatives verification**:
1. **Nix sandbox** → WebSearch 2026 confirms hermeticity (naersk, rust.nix). However, learning curve is real. MVP tmpdir pragmatic. ✓ Rejected fairly (Tier 2 post-MVP acceptable).
2. **Podman rootless** → WebSearch 2026 confirms mature 2026 (user namespaces, pasta/slirp4netns networking, seccomp support). Design doc places this in Tier 2 post-MVP. ✓ Deferred justifiably.
3. **Binaire séparé nexus-builder** → Design doc (§4) argues overkill for MVP; module within nexus-worker-core is pragmatic. ✓ Rejected fairly.
4. **Log streaming from build** → Design doc acknowledges complexity (websocket/SSE). Final result suffices for MVP. ✓ Deferred justifiably.
5. **Cross-platform builds** → rust#129080 shows upstream non-determinism. MVP x86_64-linux only. ✓ Scope cut justified.

**Assessment**: Design well-grounded in upstream Rust reproducibility state. Alternatives evaluated against current 2026 tooling maturity. Pre-launch policy (no TASK_FORMAT_VERSION bump) is respected.

---

## D4 — P2 batch 4 items quick (jitter-republish, project_name hardcode, SAFETY FFI, invite naming)

**Status**: ⚠️ Items are S54 carries; scope is documented but none implemented; verification incomplete

**Sourcing issues**:
- Kickoff: All 4 items listed as carries ("P2-S54-*") with counter "1/3" (Phase B/C/D prior reviews).
- Kickoff also says: Each item will be addressed in Phase D (coming). None are yet implemented (code review shows no commits touching these items).

**Item-by-item status**:

1. **P2-S54-jitter-republish** (±15s jitter on 45s timer, runtime.rs)
   - Scope is clear (1 line). No verification possible until Phase D implementation.

2. **P2-S54-project-name-hardcode** (extract "sbfb" hardcode from invite_api.rs to configurable const)
   - Grep finds project_name: String in coordinator invite.rs, but hardcode location in invite_api.rs not yet verified (artifact of Phase D work).
   - Scope estimate: <10 LOC (consistent with kickoff).

3. **P2-S54-AUDIT-2 SAFETY FFI convention** (add // SAFETY: comments to unsafe blocks)
   - Grep finds unsafe blocks in launcher/main.rs (set_var, remove_var without SAFETY comment) and named_pipe_server.rs (Win32 FFI GetTokenInformation without SAFETY comment).
   - **Verification gap**: These unsafe blocks exist today *without* SAFETY comments. They are pre-existing from S54 audit findings. Adding comments is a cosmetic fix, not a correctness issue, but the fact that they remain uncommented post-S54 audit suggests the decision was filed but not yet actioned.

4. **P2-S54-AUDIT-3 invite version naming** (INVITE_VERSION → INVITE_FORMAT_VERSION, u8 → u16)
   - Grep in worktree agent shows INVITE_VERSION: u8 = 2 (not yet renamed).
   - No rename to INVITE_FORMAT_VERSION, no type change to u16 in current code.
   - Scope estimate: ~5 files (invite.rs coordinator + invite.rs worker + test imports). <50 LOC total.

**Assessment**: 
- All 4 items are documented carries from prior sprint reviews, not new decisions.
- Scope estimates are consistent with "quick" characterization (<100 LOC total).
- **Risk**: None of the 4 have been touched. Phase D is the scheduled implementation window. Design review cannot validate execution quality or scope creep until actual PRs are available.
- The decision to batch these 4 together is sound operationally (prevent accumulation), but their inclusion in "Day-0 decisions" is misleading — they are task roster items, not architectural decisions.

---

## Summary notes

**D1** is well-grounded. Caddy + Docker Compose is the 2026 standard for this use case; alternatives exhaustively evaluated against current tooling.

**D2** lacks technical rigor. The choice of push-triggered GHA over workflow_dispatch is a process preference, not an architectural constraint. The nexus-core-py fix reference cannot be verified.

**D3** is solid. Rust reproducibility state is current (rust#129080 tracked), MVP scope cut to x86_64-linux is justified by upstream limitations. Alternatives (Nix, podman, cross-platform) are reasonably deferred to Tier 2-3.

**D4** is more a "task roster" than a "decision". All 4 items are S54 carries; none have been implemented. Their scope is documented and lightweight, but pre-implementation verification of Phase D work is not possible at design review stage.

**Recommended follow-up**: 
- D2: Clarify nexus-core-py fix commit reference (verify it is merged at ee0e54c or identify stale reference).
- D4: Schedule Phase D implementation with explicit pre-commit quality gates (unsafe comments present, naming conventions applied, test coverage added).
