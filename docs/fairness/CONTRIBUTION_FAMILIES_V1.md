# Contribution Families v1 — Option F 3 couches asymetriques

Sprint 23 Phase F — design doc (LT-3 post-v1.0 implementation).

- **Status** : design-only. No code lands S23 under this spec.
- **Depends on** : Sprint 22 Sybil-resistance 3-layer composition
  (Couche 1 AgeWitness, Couche 2 ContributorAttestation, Couche 3
  DelegationCert S23 format primitive).
- **Consumed by** : LT-1 Kudos-v2 fairness reform (post-v1.0).

---

## 1. Motivation

The current kudos ledger tracks a single dimension (compute tasks
completed). This creates a Matthew effect where early adopters with
fast GPUs accumulate disproportionate kudos, making it harder for
new contributors to reach governance thresholds. Empirical trigger:
Gini coefficient > 0.70 on the compute ledger (cf.
`docs/release/ROADMAP_COMMITMENTS.md §LT-1`).

Option F (arbitrated 2026-04-19, kickoff §4 D1 fairness discussion)
decomposes contributions into 3 asymmetric families with independent
weight vectors and decay functions. Each family captures a
structurally different contribution type with distinct abuse vectors
and measurement challenges.

## 2. The 3 families

### Family 1 — Compute

- **What** : GPU/CPU task execution (current ledger dimension).
- **Measurement** : task_count x redundancy_factor x quality_score.
  Quality_score = 1.0 for majority-vote matches, 0.0 for quarantined
  outliers (S23 Phase D redundancy voting provides the signal).
- **Decay** : exponential half-life 90 days. Stale compute does not
  permanently anchor reputation.
- **Abuse vector** : Sybil farms, GPU rental burst, result spoofing.
  Mitigated by PoW escalation (S23 Phase C) + redundancy voting +
  ephemeral worker restart (S23 Phase B).
- **Weight** : dominates early network (pre-storage, pre-relay).
  Target long-term share: 40-60% of total kudos.

### Family 2 — Storage

- **What** : hosting and serving blob archives for projects that the
  node did not publish. Altruistic replication increases availability.
- **Measurement** : bytes_served x uptime_ratio x unique_blobs_count.
  uptime_ratio = fraction of time the node was reachable in the last
  30 days (sampled via pkarr availability probes). unique_blobs_count
  prevents gaming by self-replicating a single tiny blob.
- **Decay** : linear decay 180 days. Storage is a sustained
  contribution (unlike compute which is burst).
- **Abuse vector** : self-publishing trivial blobs, spoofed uptime.
  Mitigated by: min blob size threshold (>1 KiB), pkarr probe-based
  uptime (not self-reported), unique_blobs_count diversity requirement.
- **Weight** : grows as network scales. Target long-term share: 20-30%.
- **Prerequisite** : iroh-blobs replication tracking (not yet wired,
  deferred to storage-incentive sprint post-v1.0).

### Family 3 — Relay

- **What** : forwarding gossip messages, relay availability, NAT
  traversal assistance for peers behind restrictive firewalls.
- **Measurement** : messages_relayed x peer_diversity x session_duration.
  peer_diversity = unique peers served / total peers in neighborhood
  (prevents relay nodes from only serving their own Sybil cluster).
- **Decay** : exponential half-life 60 days. Relay is lightweight and
  should reward sustained presence, not burst.
- **Abuse vector** : self-relaying between controlled nodes, message
  amplification. Mitigated by: peer_diversity ratio requirement,
  gossip message deduplication (already in iroh-gossip), relay
  bandwidth cap per-peer.
- **Weight** : smallest family. Target long-term share: 10-20%.
- **Prerequisite** : relay-level telemetry (deferred — iroh 0.97 does
  not expose relay-hop stats at the SDK level).

## 3. Weight vectors

The total kudos for a node is:

```
kudos_total = w_compute * f_compute(decay) +
              w_storage * f_storage(decay) +
              w_relay   * f_relay(decay)
```

Initial weight vector (tunable post-launch via governance vote
through the curator system):

| Family | Weight | Decay function | Half-life |
|---|---|---|---|
| Compute | 0.50 | exponential | 90 days |
| Storage | 0.30 | linear | 180 days |
| Relay | 0.20 | exponential | 60 days |

Weights sum to 1.0. Governance can adjust within the constraint that
no single family exceeds 0.70 (anti-capture rule).

## 4. Decay functions

### Exponential decay (Compute, Relay)

```
f(t) = raw_score * 2^(-t / half_life)
```

Where `t` = days since contribution. Each task/relay session
contributes `raw_score` at time of completion, decaying thereafter.
The ledger stores `(timestamp, raw_score)` pairs and recomputes
on-demand (no background sweep).

### Linear decay (Storage)

```
f(t) = raw_score * max(0, 1 - t / lifetime)
```

Where `lifetime` = 180 days. Storage contributions vanish completely
after 180 days of non-renewal. A node that continues hosting renews
the contribution daily (rolling window).

## 5. Gini trigger (LT-1)

The `/diagnostic/fairness` endpoint (S23 Phase E) already computes
the Gini coefficient over the compute-only ledger. The trigger
extends to multi-family:

1. Compute Gini per-family independently.
2. Compute composite Gini over `kudos_total`.
3. If composite Gini > 0.70 sustained for 30 consecutive days:
   - Emit a `fairness-alert` via the diagnostic endpoint.
   - Curator lists may advertise a `fairness-degraded` flag.
   - No automatic intervention (preserves "no admin" charter).

The 0.70 threshold and 30-day sustain window are governance-tunable
parameters stored in the coordinator's config (not hardcoded).

## 6. Integration with Couche 3 (DelegationCert)

Couche 3 multi-forge cross-validation (S24-S27) provides independent
evidence of contribution beyond the SBFB network's own measurements.
When wired:

- Verified git commits attributed via DelegationCert count as a
  bonus multiplier (1.1x-1.3x) on the Compute family for the
  project the contributor deployed. Rationale: a contributor who
  both writes code AND runs compute for the same project is more
  valuable than a pure compute node.
- The multiplier is capped to prevent gaming (contributor publishes
  trivial commits to boost their compute kudos). Cap: max 1.3x,
  requires >= 5 verified commits in the last 90 days.

## 7. Non-goals (explicit exclusions)

- **Monetary valuation** : kudos are reputation scores, never
  tokens, never tradeable, never purchasable. Cf.
  `feedback_kudos_non_monetary.md`.
- **Automated redistribution** : no "progressive tax" on high-kudos
  nodes. Fairness is achieved through decay + diversity, not
  confiscation.
- **Real-time rebalancing** : weight vector changes are governance
  decisions, not algorithmic responses. The Gini trigger signals,
  it does not act.
- **Cross-network portability** : kudos do not transfer between
  SBFB deployments. Each network instance has its own ledger.

## 8. Open questions (post-v1.0 research)

- Storage measurement without iroh-blobs replication API — requires
  upstream iroh feature or proxy metric (disk usage self-report +
  spot-check probe).
- Relay measurement without relay-hop telemetry — requires iroh
  relay instrumentation or protocol-level relay receipt.
- Weight governance UX — how do curators propose and vote on weight
  changes? CLI-only initially, web UI post-Gate 4.
- Sybil cross-family attack — a single operator running compute +
  storage + relay on the same machine scores triple. Mitigation:
  per-IP diversity cap? Needs research.

## 9. References

- Research: `fairness_vision.md` (memory, 2026-04-19 arbitrage)
- Diagnostic: `packages/nexus-coordinator/src/nexus_coordinator/fairness.py`
- Wire: `docs/fairness/KUDOS_V2_WIRE.md` (companion spec)
- Ledger: `packages/nexus-coordinator/src/nexus_coordinator/kudos/`
- Roadmap: `docs/release/ROADMAP_COMMITMENTS.md §LT-1`
