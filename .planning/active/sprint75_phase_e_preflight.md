# Sprint 75 Phase E Preflight

Date: 2026-06-10
HEAD: `41b13e3`
Verdict: **SCOPE-CUT-CONSISTENT**

## Evidence Rules
- Claim policy: every claim below cites a path:line, a command + output, a
  URL/date, or an explicit assumption.
- Local sources read in full or in the cited regions:
  - `prompts/agent/preflight.md` (full portable procedure, source of truth).
  - `.planning/active/sprint75_plan.md` (§Phase E E.1-E.5, dep graph, fail-fast 24
    rows, scope cuts §7).
  - `.planning/active/sprint75_kickoff.md` (§4 5 verrous, §5 D1-D5/D3 frozen, §9
    scope cuts, §10 Q3/Q4, §6 substrate inventory).
  - `.planning/active/sprint75_pivot_proposal.md` (D3 sign-off; §6 anchor lives in
    `default_curators`).
  - `.planning/active/sprint75_phase_d_preflight.md` (the consumer-fetch
    PLAN-ADAPT: `fetch_hash_multi`/`fetch_and_pin_multi`, NOT ticket re-mint).
  - `.planning/active/sprint75_phase_d_review.md` (7 deferred, what is Phase F/G).
  - `.planning/active/sprint75_phase_c_review.md` (the PRODUCER re-announce-at-boot
    P2 explicitly routed to Phase E).
  - `.planning/active/sprint75_phase_e_handoff.md` (§3 detailed scope).
  - `crates/nexus-shell-daemon-core/src/config.rs` (full — `ShellDaemonConfig`,
    `CuratorConfig.default_curators`, `clamped()`).
  - `crates/nexus-shell-daemon/src/runtime.rs` (`DaemonStartOptions:158-178`,
    boot driver sites `:408` auto-subscribe, `:839` reannounce_seeds_at_boot,
    `:1467` repull_directories, `mint_ticket_for_hash:1868-1882`,
    `remint_and_wrap_for_replay:1924-1950`).
  - `crates/nexus-shell-daemon/src/main.rs` (`:84/:124` config load, `:178-185`
    `DaemonStartOptions` build, `:183` `curator: cfg.curator.clone()`).
  - `crates/nexus-shell-daemon/src/feed_sync.rs` (`reannounce_seeds_at_boot:160-200`,
    `emit_seed_announced:121-148`, `:942-1047` e2e re-announce test).
  - `crates/nexus-shell-daemon/src/seed_protocol.rs` (`:1-58` module doc,
    `request_seed:298-337` `#[allow(dead_code)]`).
  - `crates/nexus-core-rs/src/seed.rs` (`:18-23` requester semantics,
    `SeedRequest:82-118` invite_token, `SeedRequestEnvelope::sign:135`).
  - `crates/nexus-core-rs/src/blobs.rs` (`fetch_hash_multi:231-252`,
    `fetch_and_pin_multi:260-270`, `MAX_FETCH_PROVIDERS:55`).
  - `crates/nexus-shell-daemon/src/http.rs` (`find_directory_app_by_project:1434`,
    `directory_pull_providers:1466-1494`, `seed_voluntary:1586-1700` directory-only
    acquisition, `publish_directory:1068-1227` authoring + the §E producer
    re-announce comment `:1216-1226`).
  - `crates/nexus-shell-daemon-core/src/iroh_runtime.rs`
    (`subscribe:608-622` single attention set, `is_subscribed:681-683`,
    `repull_directories:1055`, `NodeDirectoryAnnouncement:155-198` "same attention
    set as curator lists" `:162-163`, subscription-gate test `:1927-1949` "DQ3: one
    attention set covers both", `subscriptions.json` restore `:1222-1231`).
  - `deploy/config.toml.example` (existing `[logging]/[network]/[curator]`).
  - `docs/security/THREAT_MODEL.md` (§15 seed cross-node surface, full).
  - memory `MEMORY.md`, `nexus_grid_pivot.md` (Tip), `feedback_approach.md`,
    `feedback_context7_systematic.md`, `feedback_wsl_before_push`.
- Commands run (relevant output):
  - `git rev-parse --short HEAD` -> `41b13e3`; `git status -sb` -> `ahead 12`,
    working tree clean.
  - `grep -A1 '^name = "toml"' Cargo.lock` -> two entries: `toml 0.8.2` (runtime
    config) and `toml 1.1.2+spec-1.1.0`. `cargo tree -i toml@1.1.2+spec-1.1.0
    --workspace` -> sole parent is `winresource v0.1.31` (build-dep of
    `nexus-launcher`), UNRELATED to config parsing.
  - `grep '^toml' Cargo.toml` -> workspace pin `toml = "0.8"`. Daemon-core
    `Cargo.toml:48 toml = { workspace = true }`. Phase E adds NO dependency.
  - `grep reannounce_seeds_at_boot|fetch_and_pin_multi|repull_directories|...` ->
    boot sites and shipped primitives located (see scope).

## Scope
- Plan source: `.planning/active/sprint75_plan.md` §Phase E (E.1-E.5), lines
  212-245.
- Target files (CORRECTED — see S2/S4 findings; the plan §E.2 mis-locates the
  config crate):
  - `crates/nexus-shell-daemon-core/src/config.rs` (NOT
    `nexus-shell-daemon/src/config.rs` as plan §E.2 says — that file does not
    exist; `glob crates/nexus-shell-daemon*/src/config*.rs` -> only the `-core`
    one): NEW `[seed]`/`[directory]` config sections, EMPTY defaults (verrou 3).
  - `crates/nexus-shell-daemon/src/runtime.rs`: `DaemonStartOptions` gains the new
    config fields (`:158-178`); boot driver in `DaemonRuntime::start` reads them
    -> acquire-then-pin configured project_ids + set keep_online + re-emit
    `SeedAnnounced` + producer directory re-announce. Hooks alongside the existing
    `reannounce_seeds_at_boot` call (`:839`) and `repull_directories` (`:1467`).
  - `crates/nexus-shell-daemon/src/main.rs`: plumb the new config sections into
    `DaemonStartOptions` (mirrors `:183 curator: cfg.curator.clone()`).
  - `crates/nexus-shell-daemon/src/feed_sync.rs`: the boot seed driver reuses /
    extends `reannounce_seeds_at_boot` and `emit_seed_announced`.
  - `crates/nexus-shell-daemon/src/seed_protocol.rs`: drop `#[allow(dead_code)]`
    on `request_seed` (`:298`) ONLY if a real prod caller is wired (see S3 / Risks
    — the VPS SEED-acquisition driver is the voluntary path, NOT `request_seed`;
    `request_seed` is the REQUESTER/author-designates-a-peer path; clarify before
    coding to avoid a semantic mismatch).
  - `crates/nexus-shell-daemon/src/http.rs` OR a runtime builder: headless
    authoring at boot — reuse the `publish_directory` build+sign+announce path
    (`:1068-1227`) so the VPS publishes its catalog without a browser.
  - `deploy/`: NEW systemd `.service` unit (none exist today;
    `ls deploy/*.service` -> none) + extend `config.toml.example` with
    `[seed]`/`[directory]` sections.
- Deps/APIs/specs: NONE added or bumped. `toml 0.8` (config), `serde` are
  already workspace deps used by `ShellDaemonConfig`. `fetch_and_pin_multi`,
  `directory_pull_providers`, `find_directory_app_by_project`,
  `reannounce_seeds_at_boot`, `publish_directory` are ALL already shipped (Phase
  C/D + S74). Phase E is wiring, not new primitives.
- Security/protocol surfaces: a NEW boot network-fetch path (the seed driver
  dials providers at boot) gated on EXPLICIT config (verrou 5 nuance); the
  producer directory re-announce (gossip emit at boot); headless authoring (who
  can trigger publication). NO wire-schema change, NO new `DOMAIN_*`, NO
  `*_VERSION` bump.
- Tests expected (plan §E.3): `boot_seed_driver_pins_configured_projects`,
  `boot_repins_keep_online_blobs`, `request_seed_prod_caller` (see Risks —
  re-target), `vps_authoring_signs_own_directory`, `config_seed_section_parsed`,
  + a producer-reannounce-at-boot test (carry C).

## S1a OSS Prior Art
- Domain: config-driven, headless "seed-these-and-publish-my-catalogue at boot"
  for an always-on anchor node in a content-addressed P2P network (no UI session),
  with a BOUNDED (non-universal-mirror) seeding policy and a hardened service unit.
- Sources (accessed 2026-06-10):
  - **Radicle `radicle-node` seeding policy** (`radicle.dev/guides/seeder`,
    `radicle.dev/guides/protocol`, HackMD `@radicle/r1Zejbx5a`): seeding policy is
    config-driven in `~/.radicle/config.json` under `node.seedingPolicy`. A
    BOUNDED anchor sets `node.seedingPolicy.default = "block"` + per-repo `allow`
    via `rad seed`; `scope: "followed"` subscribes only to delegates + explicitly
    followed peers, NOT all peers. Headless = `radicle-node` runs as a systemd
    background process. This is EXACTLY the kickoff Q4 design: a per-project
    accept-list, bounded scope, NO universal mirror, no numeric knob exposed.
  - **IPFS Cluster `ipfs-cluster-follow`** (`ipfscluster.io/documentation/
    reference/configuration`, `.../collaborative/setup`): a follower peer runs from
    a config file (`service.json`) and replicates a CONFIGURED pinset at boot;
    `follower mode` means it cannot mutate others' pinset, only its own view. Same
    "config-driven pin-list, headless daemon, boot-time acquisition" shape as the
    Phase E seed driver.
  - **SSB rooms vs pubs** (kickoff §0, `manyver.se/blog/announcing-ssb-rooms`):
    pubs that mirror EVERYONE's data became overloaded; rooms store NOTHING
    (tunnel/meeting-point). The kickoff's "seed only MINE + invites, never a
    universal mirror" (verrou, scope cut #3) is the directly-cited mitigation of
    this failure mode. APPROACH-ALIGNED with the lesson.
  - **systemd hardening for network daemons** (`freedesktop.org/.../systemd.exec`,
    `docs.rockylinux.org/10/guides/security/systemd_hardening`,
    `wiki.archlinux.org/title/Systemd/Sandboxing`, 2025): the canonical hardened
    unit pattern is `NoNewPrivileges=yes ProtectSystem=strict ProtectHome=read-only
    PrivateTmp=yes RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
    RestrictNamespaces=yes MemoryDenyWriteExecute=yes LockPersonality=yes`. NOTE:
    `ProtectSystem=strict` mounts the FS read-only EXCEPT explicit `StateDirectory`/
    `ReadWritePaths`; the daemon WRITES its persistent state (`config.toml`, blobs,
    `anchors.json`, `subscriptions.json`, coordinator DB) under
    `~/.nexus-grid/shell-daemon/` (`config.rs:127-153`), so the unit MUST grant a
    `StateDirectory`/`ReadWritePaths` for that tree or the daemon cannot boot.
- Finding: **APPROACH-ALIGNED**. The config-driven headless seed driver, the
  bounded per-project accept-list (Q4), and the systemd unit all match mature OSS
  practice (Radicle seeding policy, IPFS Cluster follow, SSB rooms lesson). No
  `LIB-EXISTS` (the substrate is the project's own already-shipped primitives) and
  no `APPROACH-NAIVE`.
- Impact: none on the verdict. One CONCRETE deploy detail to honor: the systemd
  unit must allow write access to the state dir (documented under Risks, not
  blocking).

## S1b Dependencies, CVEs, Release Notes
- Scanned: `toml` (config parse), `serde`, and the iroh-blobs `Downloader` path
  the seed driver reuses transitively via `fetch_and_pin_multi`.
- Commands/sources: Phase E adds and bumps NOTHING (`Cargo.toml` untouched in
  scope; new config = new FIELDS on existing serde structs using the existing
  `toml 0.8` workspace pin). The P2-PREFLIGHT-TRANSITIVE-DEPTH gate is satisfied:
  `cargo tree -i toml@1.1.2+spec-1.1.0 --workspace` shows the second `toml` major
  is a BUILD-DEP of `winresource` under `nexus-launcher` ONLY — it never reaches
  the runtime config-parse path (workspace `toml 0.8.2`), so it is a pre-existing,
  benign duplicate, NOT a collision Phase E introduces (contrast S72 Phase C/D
  where a `Cargo.toml` BUMP pulled `schemars 1.2`; Phase E bumps nothing).
  `iroh`/`iroh-blobs` pins are unchanged (the consumer fetch primitives shipped in
  Phase D against the locked `iroh-blobs 0.100.0`). No advisory surface change
  because the dependency set is byte-identical.
- Finding: **clean** (no dependency surface change, no new transitive major).

## S2 Historical Decisions
- Commands:
  - `git log --oneline -- crates/nexus-shell-daemon-core/src/config.rs` ->
    `default_curators` added Sprint 11 Phase B (auto-subscribe at boot), last
    touched S75 (T31 hex validation in `clamped`).
  - `git log --oneline -- crates/nexus-shell-daemon/src/feed_sync.rs` ->
    `reannounce_seeds_at_boot` added S74 Phase F (`66a9409`).
  - reverse-commit check on: verrou-5 "no silent boot network fetch", the
    `default_curators`-is-the-anchor-home decision, and the producer-reannounce
    deferral.
- Decisions crossed:
  - **Verrou 5 / confidentiality default-OFF (kickoff §4:155-156, §5 D3)**: "les
    requetes utilisateur ne quittent JAMAIS la machine ; le pull d'une ancre est un
    choix explicite, jamais un appel reseau silencieux au boot." The Phase E boot
    seed driver IS a boot network fetch. **Is this a reversal?** NO. The kickoff
    EXPLICITLY nuances it (handoff §3, plan §E.1, kickoff R3): the driver is
    config-driven EXPLICIT — the operator WROTE the project_ids into MY
    `config.toml`, so it is not a silent default. **The empty-default invariant is
    the proof of non-reversal**: `config.rs` ships `default_curators` empty
    (`:248-250`), the new `[seed]`/`[directory]` sections MUST ship empty too
    (verrou 3 tripwire, fail-fast row 13). With empty config = ZERO boot network
    call (the precedent is `repull_directories` `:1467`, which already does a
    subscribed-ONLY boot fetch — `iroh_runtime.rs:1061 .filter(is_subscribed)` —
    and is a no-op on a fresh install). The boot seed driver is the SAME shape:
    config-gated, no-op when empty. **No reversion** — it is the planned realization
    of D3 with the verrou-5 nuance the kickoff already argued and the PO signed off
    (`pivot_proposal.md`, sign-off OBTENU per kickoff §5 / handoff).
  - **Anchor home = `default_curators` (kickoff §4:130, §6:86-87,
    pivot_proposal §6:86-88)**: the kickoff REPEATEDLY frames the VPS anchor
    node_id as living in MON `config.toml` `default_curators` (vide par defaut),
    "JAMAIS hard-code dans un `default_curators` compile livre a tous." This is a
    FORWARD decision and it DIRECTLY answers Q3 (see below): the design intent is to
    reuse `default_curators`, because D1 reuses the CuratorList machinery verbatim
    and `iroh_runtime.rs:162-163` + `:1931` confirm node-directory ingest is gated
    on "the SAME attention set as curator lists (DQ3: one attention set covers
    both)." **No reversion**; a separate `default_anchors` would be a new field with
    no functional difference (same `subscribe()` -> same `attention` set).
  - **Producer re-announce-at-boot deferred TO Phase E (Phase C review §"P2/NIT
    deferes":59-61; `http.rs:1216-1226` in-code comment)**: `publish_directory`'s
    gossip-announce is LIVE-ONLY, does NOT persist to the outbox, so "this PRODUCER
    does not itself re-announce on NeighborUp / boot ... A PRODUCER re-announce
    timer ... is the VPS headless driver's job (Phase E)." Phase E is the PLANNED
    closure of exactly this deferral. **No reversion**; the deferral marker is
    consumed as designed.
  - **`mint_ticket_for_hash` = PRODUCER-only (`runtime.rs:1868-1882`, S75 Phase A;
    Phase D PLAN-ADAPT)**: it bails if `!blobs.has(hash)` (`:1875`), so it CANNOT
    drive the consumer acquisition of an app the VPS never deployed. The Phase D
    preflight already adapted the consumer leg to `fetch_hash_multi`/
    `fetch_and_pin_multi`. Phase E MUST inherit that adaptation: the plan §E.2 line
    "blobs.rs: `fetch_and_pin` headless boot driver" must read `fetch_and_pin_multi`
    (already shipped), NOT a ticket re-mint. **No reversion** — this is the Phase D
    decision propagated forward (see S4 + Risks; flagged so Phase E does not
    re-introduce the impossible ticket detour the handoff §5 warns about).
  - **Seed VOLUNTARY vs `request_seed` REQUESTER (`seed_protocol.rs:1-38`,
    `seed.rs:18-23`)**: `request_seed` is the AUTHOR-designates-a-peer flow ("please
    keep MY app online" + invite_token M19); the SEEDER side (acquire a public app)
    is the VOLUNTARY path (`http::seed_voluntary` -> `fetch_and_pin_multi`, NO
    `SeedRequest`). The plan §E.1/§E.3 frames the VPS seed driver as the
    "1er appelant prod de `request_seed`", but the seed-acquisition DRIVER is the
    voluntary path. This is a plan NUANCE, not a reversal (both roles fit D3's "2
    roles bornes"), documented under Risks so Phase E targets the right caller.
- Finding: **clean** (no DESIGN-CONFLICT). Every crossed decision is a
  forward-planned continuation; the verrou-5 boot-fetch is explicitly nuanced and
  PO-signed; the empty-default invariant is the structural proof of non-reversal.

## S3 Local Patterns And Threat Model
- Full scan performed (Phase E adds a NEW boot network-fetch path that dials an
  UNTRUSTED provider set + a headless authoring trigger = new security surface, per
  preflight Step 4 escalation). Threat mapping anchored to THREAT_MODEL §15:
  - **Assets**: the local blob store (a fetched configured app becomes seedable);
    the published `NodeDirectoryEntry` (the VPS's signed catalogue); the
    `keep_online` rows; the systemd-run process identity.
  - **Actors**: a malicious provider advertised in the directory/SeedRegistry; a
    Sybil swarm padding the provider `Vec`; an attacker with loopback access
    triggering authoring; a config-file tamperer.
  - **Vectors + mitigations**:
    - **Boot driver dials providers for configured apps (amplification at boot?)**
      -> the driver reuses `directory_pull_providers` (`http.rs:1466-1494`), which
      is CAPPED (`PULL_PROVIDER_CAP`, self-excluded, deduped) and the primitive
      `fetch_hash_multi` enforces `MAX_FETCH_PROVIDERS=16` (`blobs.rs:55,244`). The
      provider `Vec` per app is bounded; the number of apps is the operator's own
      config list (small, explicit). Each fetch is integrity-gated by BLAKE3
      (`blobs.rs:220-230`): a lying provider costs one failed dial, never a poisoned
      store (verrou 4 / §15 row I = Nil holds structurally). The boot driver should
      bound concurrency / total (the C carry already notes "re-pull boot sequentiel
      N x 15s" as bounded pilote-ferme) — no NEW amplification class beyond the
      existing directory pull (which Phase D review already routed to THREAT_MODEL
      §15 rows in Phase G).
    - **Headless authoring trigger (who can publish?)** -> reuse
      `publish_directory` (`http.rs:1081`), which is loopback-authenticated
      (`authed_routes`), duress-gated (`:1086`), advertises ONLY OWN apps whose
      blob this node ACTUALLY HOLDS (`:1130-1147` content-addressing ownership
      guard, verrou 4), signs with the LOCAL node keypair (provenance = author),
      embeds no peer node_id (verrou 1/3). A boot-time builder that calls the SAME
      build+sign+announce path inherits ALL these guards. A loopback-scriptable
      endpoint is already gated; a pure boot builder needs no new surface. **No new
      trust boundary** as long as authoring reuses `publish_directory`'s guards and
      never signs a catalogue of apps it does not hold.
    - **`request_seed` prod caller + invite M19 (§15 row T = Nil)** -> IF Phase E
      wires a real `request_seed` caller, it is the REQUESTER (author asks a chosen
      seeder) and is invite-gated: the seeder checks the invite bound to
      `(project_id, archive_hash)` (M19, `seed.rs:111-116`, §15 row T). This surface
      is already fully mitigated (Ed25519 + nonce + ts window + invite ledger,
      `seed_protocol.rs:20-32`). The dead-code removal only adds a caller; it
      changes no wire and no gate. **But** see Risks: the VPS seed-acquisition
      driver is the VOLUNTARY path, not `request_seed` — re-target the test.
    - **Config tamper (operator's own machine)** -> outside the threat model
      boundary (loopback trust + the operator owns the VPS); `clamped()`
      (`config.rs:303-319`) already drops malformed hex `default_curators` entries,
      and the new sections should clamp/validate identically (drop invalid
      project_ids / node_ids at load).
  - **Verrou 5 / confidentiality default-OFF**: PRESERVED. Empty `[seed]`/
    `[directory]` config = ZERO boot network call (same as the empty
    `default_curators` -> `repull_directories` no-op). The fetch fires only for
    project_ids the operator EXPLICITLY configured.
  - **Anti-Sybil triad (kickoff §4:151-154)**: leg 1 (Ed25519 on the directory +
    feed) + leg 3 (subscription gate) present; leg 2 (kudos threshold) remains
    satisfied-by-subscription for S75 (documented scope cut §9.10). Phase E does not
    regress this — the seed driver pulls apps the operator named, from a subscribed
    anchor + signature-gated seeders.
- HARDENING_ROADMAP status: no S75/Phase-E pre-requirement is listed as a blocker.
  The §15 seed surface is already shipped (S74); the directory-pull / seed-driver
  THREAT_MODEL §15 rows are sequenced to Phase G (Phase D review, conforming to the
  S74 precedent of writing §15 at wrap-up). Phase E ships the producer-side seed
  driver; Phase G documents the §15 rows for the whole pull surface.
- Finding: **clean / non-blocking**. No regression of a covered T0-T5 / §15 threat.
  The boot driver inherits BLAKE3 integrity + provider caps; authoring inherits
  `publish_directory`'s ownership/provenance guards; `request_seed` (if wired) is
  already invite-gated. Non-blocking carry: bound the boot driver's per-app and
  total concurrency (the C carry "re-pull boot sequentiel N x 15s" pattern); add the
  directory-pull / seed-driver §15 rows in Phase G (sequenced, not a Phase E gap).

## S4 Protocol And Wire Invariants
- Wire/security files checked: `config.rs` (TOML on-disk config, NOT a network
  wire format); `feed_sync.rs` (`SeedAnnounced` feed op — UNCHANGED, the boot
  driver re-emits the EXISTING op via `emit_seed_announced`); `http.rs`
  `publish_directory` (`NodeDirectoryAnnouncement` gossip — UNCHANGED, the boot
  authoring re-uses the existing announce); `seed_protocol.rs` /`seed.rs`
  (`SeedRequestEnvelope` — UNCHANGED, only dead-code removed); `blobs.rs`
  (`fetch_and_pin_multi` — transport, not wire).
- VERSION/domain/canonical status: Phase E bumps NO `*_FORMAT_VERSION` /
  `*_ANNOUNCEMENT_VERSION` / `*_SCHEMA_VERSION`; adds NO `DOMAIN_*`. The kickoff
  §1.4 + D5 confirm 0-bump for the whole pivot; Phase E is pure wiring of shipped
  types. `MAX_PROOF_AGE_SECS=1800` untouched.
- Day 0 status: **preserved** (D1-D5; verrous 1-5; D3 PO-signed; see S2).
- Producer -> consumer trace (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH) for every field
  Phase E reads or writes:
  - **New `[seed]`/`[directory]` TOML config fields**: PRODUCER = a human writing
    `config.toml` (and `deploy/config.toml.example`); PARSER = `toml::from_str`
    into the NEW `ShellDaemonConfig` fields (`config.rs:265-279`), then `clamped()`
    (`:303-319`); CONSUMER = `main.rs` (`:124` load, `:178` build
    `DaemonStartOptions`) -> `runtime.rs` boot driver. **Contract to confirm at
    code time**: (a) the field shape in `deploy/config.toml.example` MUST match the
    parser exactly (section name, key name, list-of-string vs list-of-table) — the
    S72-D / S73-E lesson applied to a TOML producer/consumer pair, asserted by
    `config_seed_section_parsed`; (b) defaults EMPTY (verrou 3); (c) `clamped()`
    drops malformed entries like it does for `default_curators` (`:308-317`). This
    is a NEW config contract internal to the daemon (no cross-process / cross-
    language consumer), so the risk is the example-vs-parser drift, not a Zod
    boundary. A round-trip serde test (`config_seed_section_parsed`) pins it.
  - **`SeedAnnounced` feed op (boot re-emit)**: PRODUCER = the boot seed driver via
    `emit_seed_announced` (`feed_sync.rs:121-148`) — the SAME op
    `reannounce_seeds_at_boot` already emits (`:160-200`); CONSUMER = a remote
    peer's `ingest_doc_entry` -> `record_announced` (`feed_sync.rs:388-394`). The
    op fields (`project_id`, `seeder_node_id`, `archive_hash`) are UNCHANGED.
    **Zero wire impact** — Phase E re-uses the S74 op verbatim. The e2e shape is
    already covered by `remote_seeder_reannounces_after_reboot_e2e`
    (`feed_sync.rs:942-1047`).
  - **`NodeDirectoryAnnouncement` (boot producer re-announce)**: PRODUCER = the
    boot authoring builder via `publish_directory`'s announce path
    (`http.rs:1215-1227`); CONSUMER = a subscriber's
    `process_directory_announcement_bytes` (`iroh_runtime.rs:950`). The
    announcement struct (`v`, `node`, `ticket`, `iroh_runtime.rs:165-181`) is
    UNCHANGED. **Zero wire impact** — Phase E re-emits the existing B/C type.
  - **`SeedRequestEnvelope` (if `request_seed` wired)**: PRODUCER = the prod caller
    via `SeedRequestEnvelope::sign` (`seed.rs:135`); CONSUMER = the seeder's
    `SeedProtocol` handler (`seed_protocol.rs`). The envelope is UNCHANGED; only the
    `#[allow(dead_code)]` is removed. **Zero wire impact.**
  - **`fetch_and_pin_multi` (boot acquisition)**: transport helper, NOT a wire
    schema; already shipped Phase D (`blobs.rs:260-270`). No serialization change.
- Finding: **clean**. No `*_VERSION` bump, no new domain, no tolerant
  multi-version decoder, no `serde(default)` wire drift (the `#[serde(default)]` on
  config sections is legitimate runtime tolerance — a partial `config.toml` loads,
  the established `config.rs:186-191` pattern). The ONLY new contract is the
  internal TOML example-vs-parser pair, pinned by `config_seed_section_parsed`. No
  Day-0 contradiction.

## Q3 Decision (config anchor: `default_curators` vs new `default_anchors`)
- **Decision: REUSE `default_curators`. Do NOT add a `default_anchors` field.**
- Evidence:
  - There is structurally ONE attention set and ONE `subscribe()` entry point
    (`iroh_runtime.rs:608-622` inserts into `self.attention`); `is_subscribed`
    (`:681-683`) checks that single set; `default_curators` is consumed by
    `curator_runtime.subscribe(hex_key)` at boot (`runtime.rs:408-417`), landing in
    `attention`.
  - Node-directory ingest is gated on the EXACT SAME attention set: the
    `NodeDirectoryAnnouncement` doc states "subscription-gated on the SAME attention
    set as curator lists" (`iroh_runtime.rs:162-163`); the subscription-gate test
    is explicit — "Mirrors the curator subscription gate (DQ3: one attention set
    covers both)" (`iroh_runtime.rs:1931`); `repull_directories` filters on
    `is_subscribed` (`:1061`); `subscriptions.json` persists everything under a
    single `curators` field (`:1222`).
  - The kickoff itself already frames the anchor as living in `default_curators`
    (§4:130 "vit dans MON `config.toml` `default_curators`", §6:86-87,
    `pivot_proposal.md` §6:86-88). D1 reuses the CuratorList machinery VERBATIM.
  - A `default_anchors` field would have to call the SAME `subscribe()` -> the SAME
    `attention` set: it is a cosmetic/semantic split with ZERO functional
    difference, and would add new boot-loop wiring (`runtime.rs:408`) + a new
    `DaemonStartOptions` field for no behavioral gain.
- Consequence for Phase E: the `[directory] catalog` config concept collapses —
  SUBSCRIBING to an anchor (so its directory ingests + re-pulls) is already done by
  putting the anchor node_id in `default_curators`. The NEW config Phase E needs is
  ONLY the **`[seed]` section** (the list of project_ids this VPS acquires-and-pins
  at boot — the seed driver input). A separate `[directory]` section for "which
  anchors to subscribe to" is REDUNDANT with `default_curators`; if a distinct
  section is still wanted for operator clarity, it must funnel into the SAME
  `subscribe()` and be documented as an alias of curator-subscription, never a
  parallel attention set (DQ3). RECOMMEND: ship only `[seed] keep_online_projects`
  as the genuinely-new section; subscribe anchors via the existing
  `default_curators` (rename-friendly doc note that a "node anchor" is a
  curator-attention pubkey). This is a delta vs plan §E.2 (which lists a
  `[directory] catalog` section) — see Plan delta.

## Q4 Decision (bounded SEED policy form)
- **Decision: a per-project ACCEPT-LIST in `[seed]`, NO numeric quota knob.**
- Evidence: the kickoff §10 Q4 + scope cut §9.3 mandate "budget disque +
  accept-list par-projet, abstraite (pas de knob numerique pour user
  non-technique)" and DEFER the GC reaper / enforced disk budget post-launch.
  Radicle's `node.seedingPolicy` (S1a: `default: block` + per-repo `allow`) is the
  prior-art shape: an explicit allow-list, bounded scope, no numeric tuning exposed.
- Concrete form: `[seed] keep_online_projects = ["<project_id>", ...]` — the VPS
  acquires + pins ONLY these project_ids at boot (the accept-list IS the bound).
  There is NO `max_disk_gb` / `max_apps` numeric field (scope cut #3: the GC
  reaper / enforced budget is deferred; the list length is the de-facto bound). The
  driver pins each listed app skip-GC (via `fetch_and_pin_multi`'s tag) and sets
  `keep_online`. This honors "seed borne MES apps + invites, JAMAIS miroir
  universel" — there is no wildcard/"seed everything" option by construction.
- Non-goal (scope cut #3, ack): enforced disk-budget eviction / LRU reaper. The
  accept-list is the only bound S75 ships; the GC reaper is a post-launch follow-up.

## Plan delta (vs plan §Phase E)
1. **Config crate path**: plan §E.2 + handoff §3 say
   `crates/nexus-shell-daemon/src/config.rs`. That file DOES NOT EXIST. The config
   lives in `crates/nexus-shell-daemon-core/src/config.rs` (`glob` confirms; the
   `ShellDaemonConfig` struct is there `:184-192`). Phase E edits the `-core` crate
   + plumbs through `DaemonStartOptions` (`runtime.rs:158`) + `main.rs:183`.
2. **`[directory]` section is redundant (Q3)**: plan §E.2 lists a
   `[directory] catalog` config section. Per Q3, subscribing to an anchor is
   already `default_curators`; a parallel `[directory]` attention concept would
   violate DQ3 (one attention set). Ship `[seed] keep_online_projects` as the only
   genuinely-new section; subscribe anchors via `default_curators`. If a
   `[directory]` label is kept for operator ergonomics, it MUST funnel into the
   existing `subscribe()`, documented as an alias.
3. **`fetch_and_pin` -> `fetch_and_pin_multi` (inherit Phase D PLAN-ADAPT)**: plan
   §E.2 line "blobs.rs: `fetch_and_pin` headless boot driver" must read
   `fetch_and_pin_multi` (already shipped, `blobs.rs:260`). The boot driver's
   acquisition leg is the SAME chain `seed_voluntary` uses
   (`find_directory_app_by_project` -> `directory_pull_providers` ->
   `fetch_and_pin_multi`, `http.rs:1610-1664`), NOT a ticket re-mint
   (`mint_ticket_for_hash` is PRODUCER-only, bails on absent blob). NO new
   `blobs.rs` primitive is required — `blobs.rs` is likely UNTOUCHED in Phase E.
4. **`request_seed` prod caller is the REQUESTER, not the seed driver**: the VPS
   seed-acquisition driver is the VOLUNTARY path (`fetch_and_pin_multi`), NOT
   `request_seed` (which is the author-designates-a-peer/invite path,
   `seed.rs:18-23`). Re-target the test `request_seed_prod_caller`: either (a) wire
   a genuine REQUESTER caller (e.g. a boot/loopback path where an operator's node
   asks a designated peer with an invite to also seed its app), and test THAT; or
   (b) if no real requester caller exists in the headless VPS model, the
   dead-code removal is unjustified — keep `#[allow(dead_code)]` and DEFER the prod
   caller to the front peer-designation UI (Phase F / "Bientot", as
   `seed_protocol.rs:292-297` already documents). RECOMMEND clarifying with the
   plan intent before removing the `#[allow]`; do not conflate the two seed roles.
5. **systemd unit**: `deploy/` has NO `.service` file today; Phase E adds one. It
   MUST grant write access to the daemon state dir
   (`StateDirectory=nexus-grid/shell-daemon` or explicit `ReadWritePaths`) because
   `ProtectSystem=strict` mounts the FS read-only and the daemon writes config /
   blobs / `anchors.json` / `subscriptions.json` / DB there.

## Risks And Scope Cuts
- Blocking risks: **none**.
- Non-blocking risks / carry-over:
  - **`request_seed` role mismatch** (delta #4): clarify requester-vs-seeder before
    removing `#[allow(dead_code)]`; the seed driver itself does not call it.
  - **`[directory]` redundancy** (Q3 / delta #2): collapse into `default_curators`
    to avoid a second attention concept (DQ3).
  - **Boot driver concurrency bound**: pin per-app provider caps (already in
    `directory_pull_providers` / `MAX_FETCH_PROVIDERS`) and a sane total
    sequential/timeout bound (the C carry "N x 15s sequentiel" pattern) so a long
    `[seed]` list cannot stall boot; non-blocking, pilote-ferme.
  - **systemd state-dir write** (delta #5): the hardened unit must allow writing the
    daemon state tree or the daemon cannot boot.
  - **THREAT_MODEL §15 rows for the pull/seed-driver surface**: sequenced to Phase G
    (Phase D review decision; S74 precedent of §15-at-wrap-up). Phase E ships the
    driver; G documents the rows. Non-blocking, not a Phase E gap.
  - **Empty-default tripwire (verrou 3)**: the new `[seed]` (and any `[directory]`)
    defaults MUST be empty in the compiled binary (fail-fast row 13); a non-empty
    compiled default = DESIGN-CONFLICT. Guard with a test asserting
    `ShellDaemonConfig::default()` yields empty seed/anchor lists.
- Scope cuts still honored (kickoff §9 / plan §7): SearchManifest DEFER (#1) — the
  VPS is a catalogue-publisher, never an aggregator (pivot_proposal §4-5); GC
  reaper / enforced disk budget deferred (#3) — Q4 ships only the accept-list bound;
  peer-approval-for-seed unchanged (#5) — voluntary/invite (S74); multi-anchor
  advanced UX deferred (#11); 0-bump wire (#7) — E bumps nothing; front node-Browse
  deferred to Phase F (#F boundary).

## Action
- **SCOPE-CUT-CONSISTENT**: proceed with Phase E. All primitives are already
  shipped (config plumbing pattern, `fetch_and_pin_multi`, `directory_pull_providers`,
  `find_directory_app_by_project`, `reannounce_seeds_at_boot`, `emit_seed_announced`,
  `publish_directory`); Phase E WIRES them into a config-driven headless boot driver
  + producer re-announce + systemd unit. Apply the plan deltas: edit the `-core`
  config crate (not the non-existent `-shell-daemon/src/config.rs`); ship `[seed]`
  as the only genuinely-new section (Q3: subscribe anchors via `default_curators`,
  one attention set); use `fetch_and_pin_multi` (inherit the Phase D PLAN-ADAPT, no
  ticket re-mint); clarify the `request_seed` requester-vs-seeder role before
  removing `#[allow(dead_code)]`; grant the systemd unit write access to the state
  dir. EMPTY defaults (verrou 3); verrou-5 boot-fetch is config-gated EXPLICIT
  (PO-signed D3); 0 wire bump; provenance = author (authoring reuses
  `publish_directory` guards); THREAT_MODEL §15 rows sequenced to Phase G. The
  commit body should cite this preflight and document the config-path correction +
  the Q3/Q4 decisions + the `request_seed` clarification.
