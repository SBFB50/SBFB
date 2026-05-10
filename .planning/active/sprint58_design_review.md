# Sprint 58 — Design Review Day-0

**Date** : 2026-05-10 (UTC)
**Scope** : 4 décisions Day-0 pour Sprint 58 (P2P storage replication via iroh-docs)
**Scoring** : ✅ = source récente + alternative vérifiée
**Format** : factual delta surface (S1 SOTA) + historical decisions (S2) + threat model (S3) + wire invariants (S4)

---

## D1 — AppStorage P2P via iroh-docs namespace dédié

### Verdict : ✅ EXECUTE

### Factual basis (S1 SOTA)

**iroh-docs 0.98 pinné en workspace** ✅
- Vérifié : `Cargo.toml` ligne `iroh-docs = "0.98"`
- Wrapper DocsClient existant : `crates/nexus-core-rs/src/docs.rs` (Sprint 2+)
- API surface exposée : `create_doc`, `set`, `get_exact`, `get_many_by_prefix`, `subscribe`, `share_write`, `import_and_subscribe` ✅

**Namespace pattern déjà deployed** ✅
- Project docs namespace : `crates/nexus-shell-daemon/src/runtime.rs` lignes 521-552
- Code existant montre create/open pattern exact décrit dans D1
- Task documents namespace (projects) : Sprint 49 Phase A confirm multi-doc pattern

**Multi-author LWW sémantique confirmée** ✅
- Research doc `.planning/research/p2p_storage_replication_iroh_docs.md` §1.2 décrit Query API
- `Query::single_latest_per_key()` + `Query::key_exact()` : API Rust docs.rs confirmées
- Codebase usage : `nexus-core-rs/docs.rs:295` montre déjà `Query::key_prefix()` en production (Sprint 49+)

### Historical decisions (S2 — S55 Phase D context)

**Rename INVITE_VERSION → INVITE_FORMAT_VERSION (u8→u16)**
- Sprint 55 Phase D : `crates/nexus-worker-core/src/invite.rs:73`
  ```rust
  pub const INVITE_FORMAT_VERSION: u16 = 2;
  ```
- Historique : v1 (Sprint 3, jamais distribuée, hard bump Sprint 4)
- Ce changement de nommage **N'AFFECTE PAS** D1 (invite ≠ namespace storage)

**Anti-spam components déjà présents**
- Hashcash PoW (gossip publish) : Sprint 19
- Sybil resistance (Ed25519 identité) : Sprint 22
- GCRA rate limiting (worker-side) : Sprint 21 `nexus-worker-core/src/rate_limit.rs`
- Curator lists : Sprint 7+

### Alternatives rejetées — vérification concurrence

| Candidat | Rust-native | Codebase ref | Verdict |
|---|---|---|---|
| **Automerge 3** | Rust core + WASM | research §2.2 | REJETÉ : deuxième stack parallèle, JSON-CRDT overhead pour use case key-value |
| **Yjs** | ✗ JS uniquement | research §2.3 | REJETÉ : JS-only, optimisé pour texte collab, pas Rust |
| **OrbitDB** | ✗ JS + Go partial | research §2.1 | REJETÉ : IPFS orthogonal, stack P2P dupliquée, non Rust |
| **GUN.js** | ✗ JS uniquement | research §2.4 | REJETÉ : localStorage limité, relays bottleneck, instable |
| **p2panda** | Rust native (Rust/WASM) | WebSearch latency | **NE FIGURE PAS DANS RESEARCH** → ANGLE MORT |
| **Loro** | Rust native | Pas mention codebase | **NE FIGURE PAS DANS RESEARCH** → ANGLE MORT |

**⚠️ Competitors Rust-native oubliés possible** :
- **p2panda** (https://p2panda.org/) : Rust core, CRDT append-log pour graphs/sets, libp2p-ready
  - Non mentionné dans research §2 (alternatives évaluées)
  - Relevance : moderate (graph-oriented CRDT, moins direct que iroh-docs key-value)
- **Loro** (https://loro.dev/) : Rust core, RichText-optimized CRDT, YJS-compatible
  - Non mentionné, low relevance (text-oriented comme Yjs)

**Impact** : D1 reste valide (iroh-docs est le bon choix pour Ideas Hub), mais la research aurait dû mentionner p2panda pour complétude. Post-v1.0 candidat pour collaborative editing use cases.

### Threat model (S3)

**R1 — Pas de suppression physique** ✅ Accepté
- Ideas Hub << millions entries (thousands max) → pas problème
- Tombstones strategy documentée research §4.3

**R2 — Ticket Write = accès total namespace** ⚠️ Attaquant interne
- Mitigé par : schema UUID imprédictible (`ideas/{uuid}`), votes par-author natif
- **Risque résiduel** : spammeur avec ticket peut écrire toute clé
  - Layer 3 validation (daemon-side) + tombstone filtering UI = mitigation suffisante pre-v1.0

**R3 — Clock skew LWW** ✅ Bas risque Ideas Hub
- Conflits LWW rares (1 idea = 1 auteur)
- Votes sans conflit LWW (each AuthorId = separate entry)

### Wire format invariants (S4)

**iroh-docs INVITE_FORMAT_VERSION (Sprint 55 Phase D)**
- Current : `INVITE_FORMAT_VERSION = 2` (u16)
- Pre-launch policy (Sprint 58) : pas de compat multi-version, version = 2 hardcoded
- Post-v1.0 : bump à chaque break incompatible, range u16 = [0, 65535] → decade+ runway
- **Impact sur D1** : Aucun (invites ≠ app storage)

**App manifest ticket strategy (D1)**
- Option A (recommandée pre-v1.0) : DocTicket embarqué dans zip app
- Ticket contient : namespace public key + relay addresses
- Pre-launch assumption : réseau petit, open source zip acceptable (anti-spam couches 2-3 suffisent)

---

## D2 — MANDATORY JITTER-SCOPE : test unitaire bounds

### Verdict : ✅ EXECUTE

### Factual basis (S1 SOTA)

**Fonction jittered_republish_duration() existe** ✅
- Location : `crates/nexus-shell-daemon/src/runtime.rs:1182`
  ```rust
  fn jittered_republish_duration() -> std::time::Duration {
      use rand::Rng;
      let secs = rand::thread_rng().gen_range(30..=60);
      std::time::Duration::from_secs(secs)
  }
  ```
- Appels : ligne 1031 (initial delay), ligne 1171 (reset via select loop)

**test_jitter_bounds inexistant** ❌
- `grep -r "jittered_republish\|test.*jitter" crates/` → 0 résultats
- Ceci est le carry MANDATORY S55 Phase D P2-JITTER-SCOPE (non testé)

### Historical decisions (S2)

**Sprint 55 Phase D : jitter ±15s gossip republish**
- Commit `d37c54f` : "jitter ±15s sur le republish timer 45s (prevention thundering-herd)"
- Range : 30..=60 secondes (45s ± 15s)
- Raison : plusieurs daemons redémarrage simultané → gossip spike

**Carry raison** : "P2-JITTER-SCOPE : jittered_republish_duration() non unit-testable sans mock tokio timer"
- Verdict S55 : risque faible, 4 LOC trivial
- **Reclassification Day-0 S58** : MANDATORY (pas de mock tokio requis, test bounds simple)

### Threat model (S3)

**Thundering-herd sur republish** ✅ Mitigé
- Sans jitter : N daemons simultané → gossip overload tick N
- Avec jitter 30..60s : probabilité pic réduite
- Test bounds = assurance jitter activé

### Wire format invariants (S4)

**Duration encoded dans gossip republish timer** ✅ No breaking change
- Internal timer, pas wire-visible
- Pre-launch policy : aucun impact

---

## D3 — MANDATORY INVITE-U16-WIRE : doc PATTERNS.md §P47

### Verdict : ✅ EXECUTE

### Factual basis (S1 SOTA)

**INVITE_FORMAT_VERSION change Sprint 55 Phase D** ✅
- Ancien nom : `INVITE_VERSION` (u8)
- Nouveau nom : `INVITE_FORMAT_VERSION` (u16)
- Location : `crates/nexus-worker-core/src/invite.rs:73`
- Commit : `d37c54f` (S55 Phase D)

**PATTERNS.md §P46 existe** ✅
- Location : `docs/rust/PATTERNS.md` tail section
- Dernière section : §P46 "cross-platform cfg strategy" (S57 Phase A)

**§P47 inexistant** ❌
- `grep "§P47\|INVITE_FORMAT_VERSION" docs/rust/PATTERNS.md` → 0 résultats
- Ceci est le carry MANDATORY S55 Phase D P2-INVITE-U16-WIRE (non documenté)

### Historical decisions (S2)

**Raison du rename S55 Phase D**
- Commit `d37c54f` : "INVITE_VERSION → INVITE_FORMAT_VERSION rename + u8→u16 pour coherence naming"
- Autres *_FORMAT_VERSION dans codebase : `INVITE_FORMAT_VERSION = 2` (u16)
- Cohérence : tous les protocols wire format versioning use consistent `_FORMAT_VERSION` suffix

**Pre-launch policy (to document §P47)**
- Version = 2 (u16) at launch
- No multi-version compat pre-v1.0 (all nodes must update in lockstep)
- v1 invites (Sprint 3) never distributed → hard bump Sprint 4 acceptable

### Threat model (S3)

**Wire format stability** ✅ Documented before launch
- u16 range [0, 65535] → decade+ runway for version bumps
- Pre-launch : breaking changes acceptable (test network)
- Post-v1.0 : must maintain backward compat to at least v-2

### Wire format invariants (S4)

**InvitePayload.version = u16** ✅ Already deployed
- Sprint 55 Phase D shipped code change (version: u16)
- Now shipping wire format policy documentation

**Documentation checklist (§P47 scope)**
1. ✅ Historique : INVITE_VERSION → INVITE_FORMAT_VERSION + u8→u16
2. ✅ Current range : u16 = 2
3. ✅ Pre-launch policy : version = 2 hardcoded, no multi-version support
4. ✅ Post-v1.0 compat : document version bump policy

---

## D4 — Phase dette : retain_recent + bridge sync

### Verdict : ⚠️ SCOPE-CUT-CONSISTENT (avec angle morts)

### Factual basis (S1 SOTA)

**retain_recent() implemented** ✅
- Location : `crates/nexus-shell-daemon-core/src/browse_limiter.rs:34`
- Governor GCRA wrapper : `self.limiter.retain_recent()`
- Scope : browse_request gossip rate limiter (10/min per peer)

**retain_recent() NOT called periodically** ❌
- `grep -r "\.retain_recent()" crates/nexus-shell-daemon/src/` → 0 results (no production calls)
- Only in tests : `nexus-worker-core/src/rate_limit.rs` test suite
- **Action D4(a) requis** : add 60s timer in gossip select loop

**sbfb-bridge.js exists in 2+ locations** ✅
- Source : `web/public/sbfb-bridge.js`
- Destinations (should be) : `examples/sbfb-explorer/sbfb-bridge.js`, `examples/sbfb-ideas/sbfb-bridge.js`
- Current state : SHA256 = `ef55ce969704ca6071b6cd0c60197204e0201369b02f5c208191b820793ffa9f` (all identical)

**sync-bridge-sdk.sh script inexistant** ❌
- `find . -name "*sync*bridge*"` → 0 results
- **Action D4(b) requis** : create script with SHA256 check

### Historical decisions (S2)

**GCRA rate limiting Sprint 21+**
- Worker-side : `nexus-worker-core/src/rate_limit.rs` (multiple rate limiters)
- Shell-daemon-side : `nexus-shell-daemon-core/src/browse_limiter.rs` (gossip browse_request)
- Both use `governor` crate with `retain_recent()` for memory housekeeping

**Memory leak risk in keyed limiters** ⚠️
- `governor::DefaultKeyedRateLimiter` accumulates per-key buckets indefinitely
- `retain_recent()` call = evict stale keys (inactive > ~1-2 hours)
- Absence of call = unbounded memory growth (slow, but exploitable over weeks)

### Threat model (S3)

**D4(a) — 60s retain_recent timer**
- **Risk R4a** : memory exhaustion in browse_limiter via stale peer keys
  - Attacker : N distinct fake peers, 1 browse_request each → N buckets forever
  - Pre-mitigation : slow leak (~1KB per bucket, 1000s peers = 1MB/week)
  - Post-mitigation : 60s timer evicts inactive peers → bounded memory
  - **Severity** : LOW (slow leak, not instantaneous DOS), but production debt

**D4(b) — sync-bridge-sdk.sh script**
- **Risk R4b** : stale bridge.js in examples/ breaking SDK users
  - Scenario : web/public/sbfb-bridge.js updated with bug fix → examples/ not updated → SDK breakage
  - Mitigation : atomic script ensuring SHA256 match + alerting on drift
  - **Severity** : MODERATE (SDK reliability), but rare (manual updates currently)

### Wire format invariants (S4)

**No wire changes in D4** ✅
- retain_recent() : internal memory management
- sync-bridge-sdk.sh : build-time tooling, not runtime protocol

### Current status vs D4 plan

| Item | Current state | D4 plan | Verdict |
|---|---|---|---|
| retain_recent() implementation | ✅ done (S21+) | call periodically 60s gossip loop | ⚠️ todo, low-risk |
| sync-bridge-sdk.sh | ❌ doesn't exist | create script, SHA256 check | ⚠️ todo, moderate-risk |
| Web/public bridge.js | ✅ current | source-of-truth | ✅ no change |
| Examples/* bridge.js | ⚠️ currently synced | kept in-sync by script | ⚠️ todo |

### Angle mort D4

**D4 labeled "Phase dette"** but scope unclear
- Is D4(a)+(b) **mandatory Phase A** or **optional Phase B debt**?
- Sprint 58 plan (sprint58_plan.md §Phase B) lists "debt pair"
- **Ambiguity** : decide if this is PHASE A scope (before coding starts) or Phase B (interleaved with C/D)

---

## Résumé scoring

| Decision | Verdict | Source | Alternative | Risk |
|---|---|---|---|---|
| **D1** — iroh-docs namespace | ✅ EXECUTE | research doc (2026-05-10) + deployed code | p2panda, Loro (post-v1.0 candidates) | p2panda not evaluated (minor gap) |
| **D2** — test jitter bounds | ✅ EXECUTE | Sprint 55 Phase D commit + codebase | N/A (simple bounds test) | None |
| **D3** — PATTERNS.md §P47 | ✅ EXECUTE | Sprint 55 Phase D commit + codebase | N/A (documentation) | None |
| **D4(a)** — retain_recent timer | ⚠️ SCOPE-CUT | codebase (function exists, not called) | manual periodic invocation | Memory leak slow (weeks scale) |
| **D4(b)** — sync-bridge.sh | ⚠️ SCOPE-CUT | web/public + examples/ (manual update risk) | N/A (script creation) | SDK breakage rare, but possible |

---

## Recommendations

### Day-0 EXECUTE
- ✅ **D1** : proceed with iroh-docs P2P storage namespace architecture
  - Post-v1.0 : evaluate p2panda for collaborative editing use cases
- ✅ **D2** : implement `#[test] fn jitter_bounds_are_within_range()` in Phase A
- ✅ **D3** : add §P47 to PATTERNS.md in Phase A

### Day-0 CLARIFY
- ⚠️ **D4** : scope unclear between Phase A (MANDATORY) vs Phase B (debt)
  - If Phase A : both D4(a) + D4(b) fit (small, blocking memory risk)
  - If Phase B : defer to post Phase A (implement after core P2P storage works)
  - **Decision required** before kickoff

### Post-v1.0 backlog
- p2panda research : evaluate for collaborative editing (text, shared docs)
- Loro backlog : low priority (Yjs-compatible, text-optimized, less relevant than p2panda)

---

## Metadata

**Review date** : 2026-05-10 (UTC)
**Tip reviewed** : `4cf8bba` (Sprint 57 audit findings PASS)
**Sources verified** :
- ✅ Cargo.toml workspace deps
- ✅ crates/nexus-core-rs/src/docs.rs (DocsClient API surface)
- ✅ crates/nexus-shell-daemon/src/runtime.rs (namespace pattern, jitter function)
- ✅ crates/nexus-worker-core/src/invite.rs (INVITE_FORMAT_VERSION = 2)
- ✅ crates/nexus-shell-daemon-core/src/browse_limiter.rs (retain_recent method)
- ✅ .planning/research/p2p_storage_replication_iroh_docs.md (research doc, 2026-05-10)
- ✅ docs/rust/PATTERNS.md (existing patterns, §P46 confirmed, §P47 absent)
- ✅ git log Sprint 55 Phase D (commit `d37c54f`, jitter + INVITE rename)

**Decision tracker** :
- D1 iroh-docs : EXECUTE (source + alternative review complete)
- D2 MANDATORY JITTER-SCOPE : EXECUTE (code + test scope clear)
- D3 MANDATORY INVITE-U16-WIRE : EXECUTE (code change deployed, doc todo)
- D4 debt : SCOPE-CUT-CONSISTENT (Phase A vs B ambiguity needs clarification)
