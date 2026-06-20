No P0/P1/P2 GAP found in the reviewed Phase B diff. I did not run cargo; this is source/diff/registry verification only, as requested.

**Per Deliverable**
1. **ComputeGroup crypto correctness: CONFIRMED**
   `sign()` rejects signer/initiator mismatch, enforces caps, signs `canonical_bytes(&group, DOMAIN_COMPUTE_GROUP_V1)`: `crates/nexus-core-rs/src/compute_group.rs:189`, `:195`, `:196`. `verify_signature()` checks version, caps, attribution, then Ed25519 verify: `:220`, `:227`, `:228`, `:233`. Envelope `initiator`/`signature` are outside signed bytes because only `group` is canonicalized; envelope fields are separate at `:160-174`. `is_member()` checks `members` only: `:149`.

2. **Admission BEFORE frame: CONFIRMED**
   `accept()` reads `conn.remote_id()` at `crates/nexus-core-rs/src/shard.rs:226`, checks membership at `:227`, closes and returns at `:229-230`; `accept_bi()` is only reached later at `:235`, and frame reads start at `:236`. A non-member cannot reach frame processing.

3. **Framing correctness: CONFIRMED**
   Write cap is checked before writes: `shard.rs:88-94`, `:115-122`. Read header handles `FinishedEarly(0)` as clean EOF only: `:134-137`; all other header read errors, including `FinishedEarly(n>0)`, return `Err`: `:138-142`. Declared length is capped before allocation: `:144-145`.

4. **RTT API: CONFIRMED**
   `conn_rtt()` uses `conn.rtt(PathId::ZERO)`: `shard.rs:160-161`. Installed `iroh 0.98.2` exposes `Connection::rtt(path_id)`: `~/.cargo/registry/src/.../iroh-0.98.2/src/endpoint/connection.rs:968-971`. Installed `noq-proto 0.17.0` `ConnectionStats` fields have no `rtt`: `~/.cargo/registry/src/.../noq-proto-0.17.0/src/connection/stats.rs:254-267`, and stats explicitly skip rtt/cwnd/mtu: `:273-289`.

5. **0-bump wire + additive: CONFIRMED**
   `DOMAIN_COMPUTE_GROUP_V1` is additive: `crates/nexus-core-rs/src/canonical.rs:255`. `SHARD_ALPN` is a byte string constant: `crates/nexus-core-rs/src/node.rs:80`. `lib.rs` only adds module/re-export surface: `crates/nexus-core-rs/src/lib.rs:39`, `:61`, `:84`, `:164`. `git diff` shows no Cargo.toml/Cargo.lock changes and no existing `*_FORMAT_VERSION` mutation; current core adds only `COMPUTE_GROUP_FORMAT_VERSION` at `compute_group.rs:66`. Count note: `HEAD` grep in `crates/nexus-core-rs/src` shows 7 existing core `pub const *_FORMAT_VERSION`; broader workspace has additional constants, but none are touched.

6. **Tests semantics: CONFIRMED**
   Count matches expected 17: 10 compute-group tests at `compute_group.rs:276-404`, 7 shard tests at `shard.rs:275-443`. The six named tests assert their names: signature roundtrip signs/verifies and checks fields `compute_group.rs:277-288`; ALPN registration accepts registered and rejects vanilla `shard.rs:340-359`; multi-frame roundtrip uses one `open_bi()` before the loop `:375-387`; non-member test asserts no framed response is received `:422-438`; admitted member exchanges an echo `:397-409`; RTT test asserts `conn_rtt()` returns a sane value `:455-462`. No empty/`assert!(true)` false-green found.

7. **Production quality: CONFIRMED**
   `unwrap`/`expect`/`panic`/`unsafe`/`todo` matches in the two new files are under `#[cfg(test)]` modules (`compute_group.rs:264`, `shard.rs:263`). No `stake`/`cost`/`burn` fields found; `ComputeGroup` fields are only version, group_id, initiator, revision, members: `compute_group.rs:87-117`. Docs explicitly say admission control is not activation confidentiality and acknowledge SI-4 collusion residual: `compute_group.rs:18-22`, `shard.rs:41-45`.

8. **Scope: CONFIRMED**
   No `ShardPlan`, `RunProof`, or `ShardedSessionManifest` symbols found. Phase D is not implemented; only RTT is exposed through `conn_rtt()`: `shard.rs:160-161`. The perf-map/scheduler references are comments/docs, not consumers: `shard.rs:47-56`.

**Global Verdict**
CONFIRMED. The Phase B diff respects the frozen design constraints I checked: private allowlist admission, no public discovery/stake surface, frame cap before allocation, non-member rejection before `accept_bi`, correct iroh/noq RTT API, additive wire surface, and no Phase C/D scope creep.