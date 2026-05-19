# Sprint 65 — Verification (Contrat Public)

**Ecrit** : 2026-05-18.
**Tip Phase D** : a remplir post-commit.
**Theme** : Contrat public — vocabulaire confiance, raw-op feed,
Factory gates spec, SBFB.json v2 spec, dette pair.

---

## §1 Commandes lancees

```bash
# Rust
cargo fmt --all --check                                  # 0 diff
cargo clippy --workspace --all-targets --locked -- -D warnings  # 0 warnings
cargo nextest run --workspace --locked                   # 1333 passed
cargo test --workspace --locked --doc                    # ok (1 ignored)
cargo build -p nexus-shell-daemon --release              # ok

# Frontend
cd web
npx tsc --noEmit -p tsconfig.app.json                    # 0 errors
npm run lint                                             # 0 errors (5 warnings shadcn)
npm run test:unit                                        # 268 passed
npm run build                                            # ok
npm run size                                             # 6/6
bash scripts/scan-en-strings.sh                          # clean
cd ..

# Trust wording
bash scripts/scan-trust-wording.sh                       # clean
```

---

## §2 Checklist fail-fast (plan §9)

| # | Check | Commande | Critere | Resultat |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | OK |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | OK |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1333 | 1333 |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | OK |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | OK |
| 6 | npm lint | `(cd web && npm run lint)` | 0 errors | OK (5 warnings shadcn) |
| 7 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors | OK |
| 8 | Vitest | `(cd web && npm run test:unit)` | >= 268 | 268 |
| 9 | npm build | `(cd web && npm run build)` | ok | OK |
| 10 | size-limit | `(cd web && npm run size)` | 6/6 | 6/6 |
| 11 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean | OK |
| 12 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean | OK |
| 13 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical | OK |
| 14 | auth tier reject | `cargo nextest run -E 'test(feed_insert_rejects)' -p nexus-shell-daemon` | PASS | Phase A |
| 15 | raw-op roundtrip | `cargo nextest run -E 'test(unknown_op)' -p nexus-coordinator-rs` | PASS | Phase A |
| 16 | deploy→feed wire | `cargo nextest run -E 'test(deploy_inserts_release)' -p nexus-shell-daemon` | PASS | Phase A |
| 17 | TRUST_TAXONOMY.md | `test -f docs/trust/TRUST_TAXONOMY.md` | exists | OK |
| 18 | COMMONS.md | `test -f COMMONS.md` | exists | OK |
| 19 | FACTORY_GATES.md | `test -f docs/factory/FACTORY_GATES.md` | exists | OK |
| 20 | SBFB_JSON_V2.md | `test -f docs/protocol/SBFB_JSON_V2.md` | exists | OK |
| 21 | 0 "Verifie" sans qualif | `scan-trust-wording.sh` row | clean | OK |
| 22 | Badge dynamique | Vitest badge states | PASS | Phase C |
| 23 | Playwright zombies | `! test -f web/playwright.config.ts` | supprime | OK |

---

## §3 Script reproductible

```bash
cargo fmt --all --check && \
  cargo clippy --workspace --all-targets --locked -- -D warnings && \
  cargo nextest run --workspace --locked && \
  cargo test --workspace --locked --doc && \
  cargo build -p nexus-shell-daemon --release && \
  (cd web && npx tsc --noEmit -p tsconfig.app.json && \
   npm run lint && npm run test:unit && \
   npm run build && npm run size && \
   bash scripts/scan-en-strings.sh) && \
  bash scripts/scan-trust-wording.sh
```

---

## §4 Metriques sprint

| Suite | Avant (S64) | Apres (S65) | Delta |
|-------|-------------|-------------|-------|
| Rust nextest | 1326 | 1333 | +7 (Phase A) |
| Vitest | 265 | 268 | +3 (Phase C) |
| size-limit | 6/6 | 6/6 | +0 |
| **Total** | **~1597** | **~1607** | **+10** |

Decomposition delta Rust Phase A : +7 (auth tier reject +1,
version guard +1, unknown op roundtrip +1, canonical bytes +1,
deploy inserts release +1, deploy rejects http +1, deploy failure
no feed +1).

Decomposition delta Vitest Phase C : +3 (badge dynamique succes +1,
badge echec +1, badge etat transitoire +1).

---

## §5 Findings carry-over pour memory (G6)

1. **P2-PROVENANCE-404-BRIDGE** passe 3/3 MANDATORY S66. Le badge
   dynamique Phase C appelle provenance_verify mais l'endpoint
   retourne 404 si aucune provenance n'a ete stockee. L'UX affiche
   "Verification echouee" ce qui est techniquement correct mais
   potentiellement confus pour un utilisateur. S66 doit enrichir
   le handler pour distinguer "pas de provenance" de "verification
   echouee".

2. **P2-VERIFY-LOCAL-KEY-ONLY** passe 3/3 MANDATORY S66. La
   verification provenance utilise la cle Ed25519 du noeud local.
   Cross-node verification (verifier une provenance signee par un
   autre noeud) n'est pas encore implementee.

3. **Systeme agents orchestration** deploye S65 (4 agents ultra-deep
   + 2 skills fallback). Process valide sur 3 phases code (A/B/C).
   Le gain en profondeur de review est mesurable (branch coverage
   semantique, research grounding, scope cuts semantiques).

4. **Playwright** supprime (30 fichiers). Re-ecriture planifiee S69
   post-Factory. La dep `@playwright/test` reste dans package.json
   pour la re-ecriture.

5. **Python** definitvement absent. Toutes les references process
   nettoyees (README.md §4.3, §7.4, skill review). Le bloc Python
   dans le skill review est commente et marque OBSOLETE.

---

## §6 Scope cuts respectes (kickoff §7)

| # | Item | Sprint cible | Respecte |
|---|---|---|---|
| 1 | CuratorVouched/CuratorDisendorsed implementation | S67 | NON livre |
| 2 | BuildQuorumReached feed implementation | S67+ | NON livre |
| 3 | Quarantine feed hot path | S67+ | NON livre |
| 4 | Age witness gate feed admission | S67+ | NON livre |
| 5 | T1 CONFIRM_PROMPT complet (UI nonce) | post-pilote S69 | NON livre |
| 6 | SBFB.json v2 code implementation | S67 Phase A | NON livre (spec seulement) |
| 7 | node_id deprecation dans deploy.rs | S67 Phase A | NON livre |
| 8 | Factory template scaffold | S67 Phase B+ | NON livre |
| 9 | Fuzzing cargo-fuzz/proptest | post-audit | NON livre |
| 10 | CLI verify-release | S66+ | NON livre |
| 11 | VerificationDetail niveau 3 | S66+ | NON livre |
| 12 | Playwright E2E tests re-ecriture | S69 | NON livre (supprime S65) |
| 13 | THREAT_MODEL.md section feed | S66 | NON livre |
| 14 | Feed format version bump | post-launch | NON livre |

14/14 scope cuts respectes.

---

## §7 Items CLOSED S65

| Item | Phase | Exit condition |
|---|---|---|
| P2-FEED-INSERT-NO-AUTH-TIER (3/3 MANDATORY) | A | feed_insert rejette sans header interne |
| P2-VERIFY-ENTRY-VERSION-GUARD (1/3) | A | verify_entry rejette version != 1 |
| P2-COVERAGE-DEPLOY-E2E (2/3) | A | test deploy→feed roundtrip |
| P2-BADGE-WORDING-PREMATURE (pre-S14) | B | 0 badge "Verifie" sans qualification |
| P2-COMMIT-TITLE-FORMAT (2/3) | D | README.md §4.1 types valides documentes |
| P2-REVIEW-ORDER (2/3) | D | README.md §4.3 ordre review → Codex → commit |
| P2-PYTHON-BLOCK-EXEMPTION (2/3) | D | reclassifie resolved (pivot S50 supprime Python) |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE (2/3) | D | escapeAttr single quote |
| P2-PLAYWRIGHT-SPECS-STALE (2/3) | D | 30 fichiers supprimes |

9 items CLOSED (1 MANDATORY + 8 P2).
