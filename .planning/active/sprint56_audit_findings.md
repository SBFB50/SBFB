# Sprint 56 — Audit Findings

**Auditeur** : session fraiche S57
**Date** : 2026-05-09
**Tip audite** : `852c71b` (Phase D fix — wrap-up docs `bd2a62e`)
**Audit plan** : `.planning/active/sprint57_audit_plan.md`
**Verdict** : **PASS** (0 P0, 0 P1, 1 P2, 2 P3)

---

## Track A — Outbox persistence integrity

| Check | Resultat |
|---|---|
| table `gossip_outbox` dans migration M6 | OK — db.rs:130 |
| `load_outbox()` retourne `Vec<Vec<u8>>` | OK — db.rs:407 |
| `insert_outbox()` insere envelope BLOB | OK — db.rs:419 |
| `clear_outbox()` DELETE all | OK — db.rs:431 |
| `load_outbox()` appele au boot dans runtime.rs | OK — runtime.rs:698 |
| `insert_outbox()` appele dans publish path | OK — runtime.rs:1126 |
| test_outbox_survives_reopen | OK — db.rs:700 (passe) |
| Pre-launch policy : pas de nouvelle VERSION | OK — table interne |

**Verdict Track A** : PASS. Outbox persistent conforme a D1.

---

## Track B — Rate-limit browse_request effectiveness

| Check | Resultat |
|---|---|
| `BrowseRequestLimiter` struct present | OK — browse_limiter.rs:15 |
| Quota 10 req/min documente | OK — browse_limiter.rs:5,13 |
| Injection dans gossip loop | OK — runtime.rs:1027,1069 |
| test_rejects_over_quota | OK — browse_limiter.rs:62 (passe) |
| test_independent_peers | OK — browse_limiter.rs:78 (passe) |
| test_quota_recovers | OK — browse_limiter.rs:96 (passe) |

**Verdict Track B** : PASS. Rate-limit conforme a D2.

---

## Track C — Bridge security + completeness

| Check | Resultat |
|---|---|
| 5 methodes dans BridgeMethodSchema | OK — protocol.ts:29-33 |
| 5 cases dans dispatch useBridge.ts | OK — useBridge.ts:258-312 |
| 5 fonctions SDK sbfb-bridge.js | OK — sbfb-bridge.js:213-247 |
| Endpoints daemon (storage_list, storage_delete) | OK — http.rs:305,310 via storage_api.rs |
| correlationId pour chaque methode | OK — SDK `_call()` genere un uuid par requete |
| identity_pubkey ne retourne pas de cle privee | OK — useBridge.ts:286 retourne `info.node_id` (NodeId public), pas de secret_key |
| URL-encoding des parametres | OK — `encodeURIComponent()` sur key et prefix (useBridge.ts:260,273) |

**Verdict Track C** : PASS. Bridge secure et complet.

---

## Track D — Dette pair resolution completeness

| Check | Resultat |
|---|---|
| P44 forbid-deny-doc PATTERNS.md | OK — PATTERNS.md:2282 |
| P45 rustfmt-drift PATTERNS.md | OK — PATTERNS.md:2317 |
| lightcheck edition fix | OK — phase-precommit-lightcheck.sh:164 (filtre whitespace-only edition 2024) |
| BUILD-TIMEOUT Duration param + try_wait | OK — build_executor.rs:91-110 (wait_child_with_timeout + DEFAULT_BUILD_TIMEOUT) |
| REMAP-PATH --remap-path-prefix | OK — build_executor.rs:118-119 (remap_path_flag helper) |

**Verdict Track D** : PASS. 5 items P2 dette FERMES.

---

## Track E — Scope cuts compliance

| Check | Resultat |
|---|---|
| Diff stat e5d6242..852c71b | 46 fichiers, tous dans scope (crates/, web/, .planning/, docs/, docker/) |
| Aucun fichier dans zones scope-cut | OK — grep negatif sur Protocol Explorer, Ideas Hub, outbox rotation, hot-reload TOML, batch operations, podman, build log streaming |
| verification.md documente 13/13 scope cuts | OK — kickoff §7 liste 13 items, tous respectes |

**Verdict Track E** : PASS. Aucune violation scope cut.

---

## Track F — Test delta verification

| Check | Resultat |
|---|---|
| cargo nextest | 1227 passed, 0 fail (confirme) |
| Vitest | 256 passed (confirme) |
| Delta Rust | +11 (1216→1227) : Phase A +3, B +4, C +2, D +2 — conforme |
| Delta Vitest | +6 (250→256) : Phase C +5, C-fix +1 — conforme |

**Verdict Track F** : PASS. Compteurs conformes.

---

## Track G — Carry-over accountability

| Check | Resultat |
|---|---|
| P2-S53-outbox FERME | OK — verification.md:128 |
| P2-S53-browse_request FERME | OK — verification.md:129 |
| P2-S54-windows-test 3/3 MANDATORY S57 | OK — verification.md:98 |
| P2-S54-test-E2E-multi-noeuds 3/3 MANDATORY S57 | OK — verification.md:99 |
| P2-JITTER-SCOPE 2/3 | OK — verification.md:105 |
| P2-INVITE-U16-WIRE 2/3 | OK — verification.md:106 |
| 7 items CLOSED documentes | OK — verification.md:127-134 |
| 5 items dette CLOSED | OK — verification.md:131-134 |

**Verdict Track G** : PASS. Carries correctement comptabilises.

---

## Findings

### P2

**P2-RETAIN-RECENT-CARRY** : `retain_recent()` est expose dans
`BrowseRequestLimiter` (browse_limiter.rs:34) mais n'est jamais
appele dans le gossip loop. La review Phase B
(sprint56_phase_B_review.md:62) avait identifie ce point et
documente "Carry S57+ quand traffic reel". Cependant,
verification.md §4 et CLAUDE.md §Carry S57 ne listent pas ce
carry. Risque : oubli en kickoff S57 → croissance memoire DashMap
sans housekeeping sous trafic reel. Impact pre-v1.0 negligeable
(< 10 peers distincts). **Recommandation** : ajouter
`P2-RETAIN-RECENT 1/3` au kickoff S57.

### P3

**P3-CHECK-PEER-ALLOC** : `check_peer()` (browse_limiter.rs:31)
fait `.to_string()` sur un `&str` a chaque appel, creant une
allocation heap par requete. L'API governor `check_key` prend
`&K` (ici `&String`), donc la conversion est techniquement
necessaire pour le type `DefaultKeyedRateLimiter<String>`. Impact
negligeable au volume pre-v1.0.

**P3-BRIDGE-INLINE-VALIDATION** : les 5 nouvelles methodes bridge
n'ont pas de Zod payload schemas individuels (contrairement a
`PiiRedactPayloadSchema`). Validation inline dans le switch/case
est fonctionnellement correcte. Pour la coherence avec le pattern
existant, des schemas dedies pourraient etre ajoutes post-v1.0.

---

## Resume

| Severite | Count | Items |
|---|---|---|
| P0 | 0 | — |
| P1 | 0 | — |
| P2 | 1 | P2-RETAIN-RECENT-CARRY |
| P3 | 2 | P3-CHECK-PEER-ALLOC, P3-BRIDGE-INLINE-VALIDATION |

**Verdict final : PASS** — 0 P0/P1, 1 P2 documente, 2 P3
documentes. G4 rigor signal satisfait (>=1 P2+ trouve).
Sprint 57 peut proceder.
