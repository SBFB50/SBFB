# Sprint 56 — Verification

**Date** : 2026-05-09
**Tip cloture** : `852c71b` (Phase D fix) → Phase E wrap-up
**Phases livrees** : A + B + C + D (4 commits feat + 2 fix)

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1227, 0 fail | 1227 passed, 0 fail |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ok (6p 1i) |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ok |
| 6 | npm lint | `npm run lint` (web/) | 0 error | 0 error (5 warnings pre-existants) |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | 0 error |
| 8 | Vitest | `npm run test:unit` (web/) | >= 255 | 256 |
| 9 | npm build | `npm run build` (web/) | ok | ok |
| 10 | size-limit | `npm run size` (web/) | 6/6 | 6/6 |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | clean |
| 12 | Phase A preflight G8 | verdict | EXECUTE | EXECUTE |
| 13 | Phase A review | verdict | PASS | PASS |
| 14 | Phase B preflight G8 | verdict | EXECUTE | EXECUTE |
| 15 | Phase B review | verdict | PASS | PASS |
| 16 | Phase C preflight G8 | verdict | EXECUTE | EXECUTE |
| 17 | Phase C review | verdict | PASS | PASS |
| 18 | Phase D preflight G8 | verdict | EXECUTE | EXECUTE |
| 19 | Phase D review | verdict | PASS | PASS |
| 20 | outbox survives restart | test_outbox_survives_reopen | pass | pass |
| 21 | rate-limit rejects spam | test_rejects_over_quota | pass | pass |
| 22 | bridge storage_list | test_bridge_storage_list_dispatch | pass | pass |
| 23 | bridge identity_pubkey | test_bridge_identity_pubkey | pass | pass |
| 24 | Scope cuts | 13/13 respectes | all checked | all checked |
| 25 | Delta tests | cumule documente | documented | documented |

---

## §2 Delta tests cumule

| Suite | Entree (e5d6242) | Sortie (852c71b) | Delta | Phases |
|---|---|---|---|---|
| Rust nextest | 1216 | 1227 | +11 | A +3, B +4, C +2, D +2 |
| Rust doctests | 6p 1i | 6p 1i | +0 | — |
| Vitest | 250 | 256 | +6 | C +5, C-fix +1 |
| size-limit | 6/6 | 6/6 | +0 | — |

**Total Rust nextest** : 1216 → 1227 (+11).
**Total Vitest** : 250 → 256 (+6).

---

## §3 Phases livrees

### Phase A — Outbox gossip persistent SQLite
- Migration M6 : table gossip_outbox dans coordinator.db
- Helpers DB : load_outbox() + insert_outbox() + clear_outbox()
- Runtime boot : pre-remplir Vec depuis DB
- Runtime publish : insert DB en plus du Vec push
- +3 tests Rust (1216→1219)
- CLOSE P2-S53-outbox non-persistant (3/3 MANDATORY)

### Phase B — Browse_request rate-limit governor per-peer
- BrowseRequestLimiter : governor GCRA keyed par NodeId hex
- Quota 10 req/min/peer, drop silencieux + log debug
- Injection runtime.rs avant replay outbox sur browse_request
- +4 tests Rust (1219→1223)
- CLOSE P2-S53-browse_request rate-limit (3/3 MANDATORY)

### Phase C — Bridge extensions 5 methodes
- storage_list : endpoint GET + handler + SDK
- storage_delete : endpoint DELETE + handler + SDK
- identity_pubkey : handler + SDK (pas de nouvel endpoint)
- node_status : handler + SDK (proxy health enrichi)
- browse_list : handler + SDK (proxy browse existant)
- +2 tests Rust (1223→1225) + +5 Vitest (250→255)
- Fix commit 89f8a2f : +1 Vitest auth header test (255→256)

### Phase D — Dette pair P2 batch
- forbid-deny-doc : PATTERNS.md §P44 convention documentee
- rustfmt-drift-sessions : investigation + documentation PATTERNS.md §P45
- lightcheck-edition-faux-positif : Dockerfile lightcheck fix
- BUILD-TIMEOUT : Duration param + tokio::time::timeout 30min
- REMAP-PATH : --remap-path-prefix dans build_executor
- +2 tests Rust (1225→1227)
- CLOSE 5 items P2 dette pair

---

## §4 Carries sortants S57

### 3/3 MANDATORY (passent de 2/3 a 3/3)

| Item | Source | Compteur S57 |
|---|---|---|
| P2-S54-windows-test-cfg-unix | S54 Phase B | **3/3 MANDATORY** |
| P2-S54-test-E2E-multi-noeuds | S54 Phase C | **3/3 MANDATORY** |

### P2 (compteur incremente)

| Item | Source | Compteur S57 |
|---|---|---|
| P2-JITTER-SCOPE | S55 Phase D | 2/3 |
| P2-INVITE-U16-WIRE | S55 Phase D | 2/3 |

### Exemptions / heritage

| Item | Statut |
|---|---|
| P2-A-1 rand blocker upstream | exemption externe (15+/3, inchange) |
| P2-AUDIT-2 iroh transitives | herite pin 0.98 (inchange) |

### Long-term

| Item | Statut |
|---|---|
| LT-7 self-hosted build | Tier 1+2 DONE (S55). Tier 3 reste (N builders, auto-deploy). S57+ |
| LT-1 Kudos-v2 fairness | trigger Gini > 0.70. Latent. |
| LT-2 Radicle | trigger tag v1.0. Latent. |
| LT-5 redundancy persistence | reclassifie S26. Latent. |

### CLOSED S56

| Item | Resolution |
|---|---|
| P2-S53-outbox non-persistant | Phase A — 3/3 MANDATORY FERME |
| P2-S53-browse_request rate-limit | Phase B — 3/3 MANDATORY FERME |
| P2-S54-forbid-deny-doc | Phase D — 2/3 → CLOSE |
| P2-S54-rustfmt-drift-sessions | Phase D — 2/3 → CLOSE |
| P2-S54-lightcheck-edition-faux-positif | Phase D — 2/3 → CLOSE |
| P2-BUILD-TIMEOUT | Phase D — 1/3 → CLOSE |
| P2-REMAP-PATH | Phase D — 1/3 → CLOSE |

---

## §5 Findings carry-over for memory

- Outbox gossip persistent SQLite (coordinator.db M6). Le daemon
  charge l'outbox au boot via load_outbox() et insere chaque
  enveloppe via insert_outbox(). Le replay reste en memoire.
- Browse_request rate-limit governor GCRA per-peer (10 req/min).
  Drop silencieux + log debug. Module browse_limiter.rs dans
  nexus-shell-daemon-core.
- Bridge postMessage : 9 methodes (4 existantes + 5 nouvelles).
  storage_list, storage_delete, identity_pubkey, node_status,
  browse_list. SDK sbfb-bridge.js mis a jour.
- Dette pair S56 : 5 items P2 FERMES (forbid-deny-doc, rustfmt-drift,
  lightcheck-edition, build-timeout, remap-path).
- 2 items 3/3 MANDATORY S56 FERMES (outbox + browse_request).
- 2 items passent 3/3 MANDATORY S57 (windows-test + E2E multi-noeuds).
  S57 DOIT les inclure dans le plan obligatoire.
- Compteurs tests : 1227 Rust / 256 Vitest / 6/6 size.
- S57 impair : pas de phase dette obligatoire, mais 2 items
  MANDATORY a resoudre.
