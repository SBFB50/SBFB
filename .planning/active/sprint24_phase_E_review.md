# Phase Review — Sprint 24 Phase E

## Verdict : PASS

Rigor signal : 2 findings P2 documentes (>=1 requis pour PASS rigoureux)

## Memory consultation (Step 1.5)
- feedback_approach.md : "pick deepest" → hickory-resolver (most mature Rust DNS resolver, 4.4k stars, Apache-2.0, DoH+DoT+DNSSEC native). Conforme.
- feedback_context7_systematic.md : context7 `/websites/rs_hickory-resolver` queried pre-code. Conforme.
- feedback_kudos_non_monetary.md : N/A (phase transport, pas kudos)
- vision_model.md : N/A

## Staging check (Step 1bis)
- Phase fichiers : 7 (5 modified + 2 new)
- Planning/docs split : chore(planning) commit `9b2686c` fait AVANT phase
- Untracked accidentels : 0

## Suites (Step 2)
- Rust fmt : clean
- Rust clippy : 0 warnings
- Rust nextest : 757 pass (+12 vs 745 pre-Phase E)
- Rust doctests : pass
- Python ruff : clean
- Python SDK : 185 pass
- Python coord : 315 pass + 32 stale + 3 skip
- Python gov : 46 pass
- TSC : clean
- Web lint : 0 errors
- Web unit : 264 pass
- Web build : OK
- Size-limit : 7/7
- Playwright : 43 pass
- Release build : OK

## Modified-file branch coverage (Step 2bis, G9)
- browse.rs : `with_dns_fallback()` (4 LOC) → tested by `probe_dns_fallback_resolves_on_quorum_all_failed` + `_fails_marks_unreachable` PASS
- browse.rs : `if let Some(dns) = self.dns_fallback.as_ref()` Some branch → tested by 2 tests above PASS
- browse.rs : `if let Some(dns)` None branch → tested by existing `probe_and_cache_skips_dial_when_all_quorum_resolvers_fail` (no dns_fallback configured) PASS
- lib.rs : `pub mod dns_fallback` + re-exports → compilation-checked PASS

## Delta tests (Step 3)
- Rust nextest : 745 → 757 (+12 Phase E : 10 dns_fallback + 2 browse integration)
- Python coord : 315 (unchanged)
- Vitest : 264 (unchanged)
- Playwright : 43 (unchanged)
- Total : ~1609 → ~1621 (+12)

## Commit body validation (Step 4)
- Format titre : `feat(sprint24): Sprint 24 Phase E — ...` PASS
- Contexte present : oui (DnsFallbackResolver DoH+DoT description) PASS
- Fichiers touches listes : oui (6 fichiers + rationale) PASS
- Delta tests cumule : +12 coherent avec Step 3 PASS
- Scope cuts honoured : 10 items listes PASS
- Co-Authored-By : present PASS

## Research grounding (Step 4bis)
- 4bis-A OSS prior art (G10) : preflight S1a documente 5 projets OSS (libp2p/IPFS, pubky/pkdns, pubky/pkarr, KadNode, AdguardTeam/dnsproxy). APPROACH-ALIGNED. PASS
- 4bis-B deps context7 : hickory-resolver queried via context7 (`/websites/rs_hickory-resolver`), API DoH/DoT confirmed, TxtLookup type verified. Plan §3 Research consulte lists hickory-resolver. PASS

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc present : DOMAIN_FRONTING_DESIGN.md (outline, S25+ implementation) PASS
- D1..D5 Day 0 citent alternatives + rationale : D4 kickoff §4 documents DNSCrypt (rejected), DNS plain (rejected), custom bootstrap (rejected). PASS
- Solution la plus poussee : hickory-resolver = most mature Rust DNS resolver (vs doh_dns DoH-only, vs reqwest custom reimplementation). PASS
- Aucune LOC estimee au plan : verifie, 0 match PASS

## Scope cuts verification (Step 5)
- Key rotation : 0 fichiers diff PASS
- C3 handoffs : 0 fichiers diff PASS
- GuardrailChain cross-process : 0 fichiers diff PASS
- Domain fronting implementation : design doc only, no implementation code PASS
- All 10 scope cuts honoured PASS

## Findings (rigor signal)
- **P2-E-1** : `DnsFallbackResolver::build_resolver` uses `endpoints[0].tls_name` for all IPs in the group — if DoH endpoints have different TLS names (e.g. mixed Cloudflare + Google), only the first endpoint's TLS name is applied to all. Current defaults have distinct TLS names per endpoint but the API allows misconfiguration. Carry-over S25: per-endpoint TLS name support in `NameServerConfig` construction.
- **P2-E-2** : `resolve_node` tries DoH sequentially then DoT — no concurrent attempt. For latency-sensitive fallback, concurrent DoH+DoT with first-wins would reduce worst-case latency from 2x timeout to 1x. Carry-over S25: concurrent fallback strategy.

## Recommendation
- Ready to commit : OUI (committed `e9d69db`)
- Carry-overs S25 : P2-E-1 (per-endpoint TLS name) + P2-E-2 (concurrent DoH+DoT)
