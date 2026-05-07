# Sprint 54 — Verification

**Tip d'entree** : `2f5d76c` (post-audit S53 PASS).
**Tip de sortie** : `5e12d14` (post-Phase D fixes).
**Date** : 2026-05-07.

---

## §1 Goal recall

Sprint 54 stabilise le workspace pour la production : edition Rust
2024 (MANDATORY 3/3), resorbe la dette pair S53, cable le E2E task
flow (prerequis LT-7), et consolide l'infra CI.
**Critere SMART : 24+ rows fail-fast verts au verification.md, mesure
binaire au Phase E wrap-up. Edition 2024 compile + tests verts.
tasks_doc_ticket present dans le wire invite.**

---

## §2 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff ✅ |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings ✅ |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1206, 0 fail | 1207/1207, 0 fail ✅ |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | 6 pass, 1 ignored ✅ |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ok ✅ |
| 6 | npm lint | `npm run lint` (web/) | 0 error | 0 errors (7 warnings pre-existant) ✅ |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | 0 errors ✅ |
| 8 | Vitest | `npm run test:unit` (web/) | >= 250 | 250/250 ✅ |
| 9 | npm build | `npm run build` (web/) | ok | ok ✅ |
| 10 | size-limit | `npm run size` (web/) | 6/6 | 6/6 ✅ |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | clean ✅ |
| 12 | Playwright | `npx playwright test` (web/) | >= 42 | 42 + 2 fail env pre-existant ✅ connu |
| 13 | edition | `grep 'edition' Cargo.toml` | "2024" | edition = "2024" ✅ |
| 14 | Phase A preflight G8 | verdict | EXECUTE | EXECUTE ✅ |
| 15 | Phase A review | verdict | PASS | PASS (2 P2) ✅ |
| 16 | Phase B preflight G8 | verdict | EXECUTE | EXECUTE ✅ |
| 17 | Phase B review | verdict | PASS | PASS (2 P2) ✅ |
| 18 | Phase C preflight G8 | verdict | EXECUTE | EXECUTE ✅ |
| 19 | Phase C review | verdict | PASS | PASS (2 P2) ✅ |
| 20 | Phase D preflight G8 | verdict | EXECUTE | EXECUTE ✅ |
| 21 | Phase D review | verdict | PASS | PASS (3 P2) ✅ |
| 22 | tasks_doc_ticket | `grep tasks_doc_ticket crates/nexus-coordinator-rs/src/invite.rs` | present | present ✅ |
| 23 | node_key perms | `grep set_permissions crates/nexus-shell-daemon/src/runtime.rs` | present | present ✅ |
| 24 | GossipTaskConfig | `grep GossipTaskConfig crates/nexus-shell-daemon/src/runtime.rs` | present | present ✅ |
| 25 | periodic republish | `grep -E 'Duration::from_secs\|republish' crates/nexus-shell-daemon/src/runtime.rs` | present | present ✅ |
| 26 | CI images pin | `grep sha256 .woodpecker/ci-linux.yml` | present | 3 images SHA256 ✅ |
| 27 | Rust CI fix | `grep nexus-core-py .github/workflows/rust-ci.yml` | absent | absent (supprime) ✅ |
| 28 | Scope cuts | 12/12 respectes | all checked | 12/12 ✅ |
| 29 | Delta tests | cumule documente | documented | +1 Rust, +0 frontend ✅ |

**29/29 checks verts.** Critere SMART satisfait.

---

## §3 Phases livrees

| Phase | Commit | Titre | Review |
|---|---|---|---|
| A | `1d010b0` | Rust edition 2024 upgrade + unsafe set_var wrapping | PASS (2 P2) |
| B | `ed5bbdc` | Dette pair quick P2 batch (node_key perms + gossip refactor + periodic republish) | PASS (2 P2) |
| C | `0d17660` | E2E wire tasks_doc_ticket in invite format | PASS (2 P2) |
| D | `be633c3` | CI infra images pin + Rust CI fix + nextest profiling | PASS (3 P2) |

Fixes supplementaires (Phase D fallout) :
- `649d67f` fix(sprint54): collapsible_match clippy lint
- `60fc462` fix(sprint54): unnecessary_sort_by clippy lint
- `5e12d14` fix(sprint54): add libc dep to nexus-test-harness

---

## §4 Delta tests cumule

| Suite | Entree (S53 CLOSED) | Sortie (S54 CLOSED) | Delta |
|---|---|---|---|
| Rust nextest | 1206 | 1207 | +1 (Phase C invite_worker_requires_project_doc) |
| Rust doctests | 6 pass, 1 ignored | 6 pass, 1 ignored | +0 |
| Vitest | 250 | 250 | +0 |
| Playwright | 42 + 2 fail env | 42 + 2 fail env | +0 |
| size-limit | 6/6 | 6/6 | +0 |
| **Total** | **~1462** | **~1463** | **+1** |

Delta modeste : S54 est principalement une migration mecanique
(edition 2024), du refactoring (gossip struct), de l'infra CI
(docs/YAML), et un seul ajout fonctionnel (tasks_doc_ticket wire).

---

## §5 Items CLOSED ce sprint

| Item | Phase | Disposition |
|---|---|---|
| P2-REVIEW-B-1-S51 edition 2024 upgrade | A | **3/3 MANDATORY CLOSE** |
| P2-S53-node_key perms 0600 | B | CLOSE |
| P2-S53-gossip params struct | B | CLOSE |
| P2-S53-route collision doc | B | CLOSE |
| P2-S53-periodic republish | B | CLOSE |
| P2-S53-preflight process gap | B | CLOSE (doc README.md §6.9) |
| P2-AUDIT-1-S52 images CI pin | D | CLOSE (SHA256 3 images) |
| P2-REVIEW-A-1-S52 nextest timeout | D | CLOSE (profiled + documented) |

**8 items CLOSED** (1 MANDATORY 3/3 + 7 P2).

---

## §6 Carries residuels S55

### Items herites

| Item | Compteur S55 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 13+/3 | exemption externe permanente |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 (Day 0 #3) |
| P2-S53-outbox non-persistant | 2/3 | S53 Phase F review |
| P2-S53-browse_request rate-limit | 2/3 | S53 Phase G review |

### Items escalades 3/3 MANDATORY S55

| Item | Compteur S55 | Raison |
|---|---|---|
| P2-REVIEW-B-1-S52 Woodpecker serveur | **3/3 MANDATORY** | Infra VPS prete (Docker, deploy-key, cli), serveur + webhooks TLS requis |
| P2-REVIEW-B-2-S52 GHA validation post-push | **3/3 MANDATORY** | Rust CI fix committe, run ID a documenter post-push master |

### Nouveaux P2 S54

| Item | Compteur | Source |
|---|---|---|
| P2-S54-forbid-deny-doc | 1/3 | Phase A review — documenter dans PATTERNS.md |
| P2-S54-lightcheck-edition-faux-positif | 1/3 | Phase A review — process improvement |
| P2-S54-jitter-republish | 1/3 | Phase B review — jitter ±15s thundering-herd |
| P2-S54-windows-test-cfg-unix | 1/3 | Phase B review — cfg(unix) gap CI Windows |
| P2-S54-test-E2E-multi-noeuds | 1/3 | Phase C review — test iroh multi-node |
| P2-S54-project-name-hardcode | 1/3 | Phase C review — "sbfb" hardcode invite_api |
| P2-S54-rustfmt-drift-sessions | 1/3 | Phase D review — drift rustfmt 1.94→1.95 |

### Long-terme (inchanges)

- T-NN+2 iframe Rust-wasm (PATTERNS §P34)
- LT-2 Radicle sortie cap G7 (trigger tag v1.0)
- LT-3/LT-4 hors-sprint (post-v1.0)
- LT-5 redundancy persistence (reclassifie S26)
- LT-7 self-hosted build — **PRE-V1.0 OBLIGATOIRE** (S55)

---

## §7 Scope cuts verification (12/12)

1. LT-7 self-hosted build foundation — S55 ✅ non touche
2. Test E2E multi-noeuds automatise — S55 ✅ non touche
3. Outbox persistant fichier — S55 ✅ non touche
4. Browse_request rate-limit per-peer — S55 ✅ non touche
5. VPS TLS + nginx — S55 ✅ non touche
6. VPS monitoring + alerting — S55+ ✅ non touche
7. systemd service VPS — S55 ✅ non touche
8. LT-1 Kudos-v2 fairness reform — S56+ ✅ non touche
9. Events SSE daemon-native — post-v1.0 ✅ non touche
10. MCP server Rust — post-v1.0 ✅ non touche
11. Pagination SQL-side LIMIT/OFFSET — S55+ ✅ non touche
12. Test infra mk_state() refactoring — S55+ ✅ non touche

---

## §8 Findings carry-over for memory

- **Edition 2024 done** : workspace en edition 2024, Rust 1.94+.
  set_var/remove_var wrappees unsafe dans 17 fichiers. 3 crates
  downgrades forbid→deny unsafe_code (deny production, allow test).
- **GAP E2E CLOSED** : tasks_doc_ticket cable dans MintRequest +
  InviteRecord + invite_api (export) + worker invite (parse).
  Prerequis LT-7 satisfait.
- **CI infra** : images Woodpecker pinnees SHA256 (3 images).
  Rust CI GHA corrige (nexus-core-py fantome supprime). VPS
  Docker 29.4.3 + deploy-key + woodpecker-cli 3.14.0.
  Serveur Woodpecker + webhooks TLS = S55 3/3 MANDATORY.
- **Dette pair** : 5 P2 S53 CLOSED. Gossip refactore
  (GossipTaskConfig struct + periodic republish timer).
- **2 items escalades 3/3 MANDATORY S55** : Woodpecker serveur
  + GHA validation post-push. A traiter en priorite S55.
