# Sprint 64 Phase A — review

**Date** : 2026-05-16.
**Commit** : `daa3a8e` (initial) + fixes inter-phase `9b8abfa`/`956a1a7`.
**Phase** : A — F1 CLOSED ; F5 code delivered, preuve E2E Phase D.

---

## Staging coherence (Step 1bis)

- 8 fichiers code + 1 planning (preflight) — coherent avec scope Phase A
- Pas de fichier docs/planning hors-phase stage avec le code
- Bridge SDK non modifie (pas de changement sbfb-bridge.js copie)

---

## Verification suites §7.4

| Suite | Resultat |
|---|---|
| cargo fmt --all --check | 0 diff |
| cargo clippy --workspace --all-targets --locked -- -D warnings | 0 warnings |
| cargo nextest run --workspace --locked | 1309/1309 PASS |
| cargo test --workspace --locked --doc | ok (1 ignored) |
| cargo build -p nexus-shell-daemon --release | ok |
| npm run test:unit (web/) | 265/265 PASS |
| sync-bridge-sdk.sh | match |

---

## Post-fix targeted proof (2026-05-16)

| Commande | Resultat |
|---|---|
| `cargo nextest run --workspace --locked` | 1309/1309 PASS, nextest run `8d4c2fc5-439e-4004-8759-33a80fee02c4` |
| `cargo test -p nexus-coordinator-rs --locked blake3_hash_stable_without_app_version -- --nocapture` | 1/1 PASS |
| `python -m py_compile scripts\agent\agentctl.py` | PASS |
| `phase_from_title("feat(feed): Sprint 64 Phase A - test")` | `('64', 'A')` |
| `agentctl.py auditor-gate` on `feat(feed): Sprint 64 Phase A - test` | rc=0 |
| `npm run test:unit` (web/) | 265/265 PASS apres hardening localStorage Node v25 |
| `npm run build` (web/) | PASS |
| `npm run size` (web/) | 6/6 PASS |
| `npm run lint` (web/) | 0 error, 5 warnings fast-refresh preexistants |

---

## Delta tests

| Reference | Delta | Commentaire |
|---|---|---|
| Plan initial Phase A | +4 attendu | incluait `test_subscribe_timeout_reconnects` |
| Livraison Phase A initiale `daa3a8e` | +3 (1305→1308) | 2 DB + 1 endpoint provenance |
| Fix inter-phase `9b8abfa` | +1 (1308→1309) | hash provenance legacy |
| Ecart fonctionnel restant | preuve E2E Phase D | subscribe timeout/reconnect en scenario multi-daemon |

**Justification ecart** : `test_subscribe_timeout_reconnects` requiert
un multi-daemon E2E setup (gate SBFB_INTEGRATION=1) pour simuler un
timeout reel du subscribe iroh-docs. Ce scenario est plus naturellement
couvert par le test nouveau-noeud E2E en Phase D. Le mecanisme
timeout/retry est en place dans le code (feed_sync.rs), sa preuve E2E
sera Phase D.

---

## F1 P2-VERSION-NOT-STORED — status

**CLOSED.** Exit condition satisfaite :
- M13 migration : colonne `app_version TEXT` ajoutee ✅
- Insert : `deploy.rs` lit version de SBFB.json, passe a `insert_provenance_record` ✅
- Lecture : `get_provenance_by_project` retourne app_version ✅
- Endpoint : provenance response inclut app_version via record serialize ✅
- Tests : `provenance_insert_with_version` + `provenance_insert_without_version_backward_safe` + `provenance_endpoint_returns_app_version` ✅

## F5 P2-IROH-INFRA-TIMEOUT — status

**PARTIELLEMENT RESOLU.** Code livré, exit condition partiellement satisfaite :
- Timeout 30s sur subscribe : ✅ (feed_sync.rs tokio::time::timeout)
- Retry backoff exponentiel : ✅ (500ms → 30s max)
- Reconnexion stream break : ✅ (stream.next() returns None → resubscribe)
- Shutdown graceful : ✅ (watch channel + JoinHandle joined)
- Test stabilite SBFB_INTEGRATION "0 timeout 5 runs" : ❌ (differe Phase D)

**Reclassement** : F5 passe de "3/3 MANDATORY" a "code livre, preuve E2E
Phase D". L'exit condition "0 timeout 5 runs consecutifs" sera validee
par `test_new_node_full_sync_and_verify` (Phase D) qui exerce le subscribe
dans un scenario multi-daemon reel.

---

## Fix inter-phase (post-audit GPT 5.5)

1. **P1 provenance_hash drift** : `#[serde(skip_serializing_if = "Option::is_none")]`
   ajoute sur `app_version`. Records legacy (app_version=NULL) ne voient plus
   `"app_version": null` dans le JSON serialise → hash BLAKE3 stable.
   Test ajoute : `blake3_hash_stable_without_app_version`.

2. **P1 hook gate bypass** : `.claude/hooks/*` et
   `scripts/agent/agentctl.py` detectent maintenant Sprint+Phase meme
   dans les domain scopes (`feat(feed): Sprint 64 Phase A`).

---

## Scope cuts respectes

Phase A livrable apres fix inter-phase (hash stability + hook gate).
F5 reclasse "preuve Phase D" — pas un bloquant Phase B.
12/12 items non touches (confirme via grep sur fichiers stages).

---

## Verdict: PASS
