# Sprint 57 — Audit Plan (pour session fraiche S57)

**Sprint audite** : S56 (gossip resilience + bridge extensions + dette pair).
**Tip a auditer** : `852c71b` (HEAD post-Phase D fixes).
**Phases** : A (outbox persistent), B (browse rate-limit), C (bridge extensions), D (dette pair).
**Compteurs S56 sortie** : 1227 Rust / 256 Vitest / 42+2f PW / 6/6 size / ~1489 total.

---

## Track A — Outbox persistence integrity

**Objectif** : verifier que l'outbox gossip est bien persistent en DB.

1. `grep -n gossip_outbox crates/nexus-coordinator-rs/src/db.rs`
   → table creee dans migration M6
2. `grep -n load_outbox crates/nexus-coordinator-rs/src/db.rs`
   → fonction presente, retourne Vec<Vec<u8>>
3. `grep -n insert_outbox crates/nexus-coordinator-rs/src/db.rs`
   → fonction presente, insere envelope BLOB
4. `grep -n clear_outbox crates/nexus-coordinator-rs/src/db.rs`
   → fonction presente, DELETE all
5. `grep -n load_outbox crates/nexus-shell-daemon/src/runtime.rs`
   → appel au boot dans start()
6. `grep -n insert_outbox crates/nexus-shell-daemon/src/runtime.rs`
   → appel dans publish path
7. Verifier test_outbox_survives_reopen passe (close DB + reopen)
8. Pre-launch policy : pas de nouvelle VERSION (outbox = table interne,
   pas de wire format)

## Track B — Rate-limit browse_request effectiveness

**Objectif** : verifier que le rate-limit per-peer fonctionne.

1. `grep -n BrowseRequestLimiter crates/nexus-shell-daemon-core/src/browse_limiter.rs`
   → struct present avec governor GCRA
2. `grep -n "10\|BROWSE_RATE" crates/nexus-shell-daemon-core/src/browse_limiter.rs`
   → quota 10 req/min documente
3. `grep -n browse_limiter crates/nexus-shell-daemon/src/runtime.rs`
   → injection dans gossip loop, check avant replay
4. Verifier test_rejects_over_quota passe (15 requests → some rejected)
5. Verifier test_independent_peers passe (2 peers, chacun sous quota)

## Track C — Bridge security + completeness

**Objectif** : verifier que les 5 nouvelles methodes bridge sont
securisees et fonctionnelles.

1. `grep -n "storage_list\|storage_delete\|identity_pubkey\|node_status\|browse_list" web/src/bridge/protocol.ts`
   → 5 methodes dans BridgeMethodSchema
2. `grep -n "storage_list\|storage_delete\|identity_pubkey\|node_status\|browse_list" web/src/bridge/useBridge.ts`
   → 5 cases dans dispatch
3. `grep -n "storage_list\|storage_delete\|identity_pubkey\|node_status\|browse_list" web/public/sbfb-bridge.js`
   → 5 fonctions SDK
4. Verifier les endpoints daemon existent :
   `grep -n "storage.*list\|storage.*delete" crates/nexus-shell-daemon/src/http.rs`
5. Verifier que les methodes bridge ont des payload schemas Zod
6. Verifier que le SDK utilise des correlationId pour chaque methode
7. Spot-check : identity_pubkey ne retourne pas de cle privee
   `grep -n pubkey crates/nexus-shell-daemon/src/http.rs`

## Track D — Dette pair resolution completeness

**Objectif** : verifier que les 5 items P2 sont reellement resolus.

1. `grep -n "P44\|forbid.*deny" docs/rust/PATTERNS.md` → section
   documentant la convention deny vs forbid
2. `grep -n "P45\|rustfmt" docs/rust/PATTERNS.md` → section
   documentant le drift rustfmt et la solution
3. Verifier lightcheck fix :
   `grep -n "edition\|2024" .claude/hooks/` ou Dockerfile
   → faux positif corrige
4. `grep -n timeout crates/nexus-worker-core/src/build_executor.rs`
   → Duration param + tokio::time::timeout
5. `grep -n remap_path crates/nexus-worker-core/src/build_executor.rs`
   → --remap-path-prefix present

## Track E — Scope cuts compliance

**Objectif** : verifier qu'aucun scope cut n'a ete viole.

1. `git diff e5d6242..852c71b --stat` → pas de fichiers dans les
   zones scope-cut (LT-7 Tier 3, Protocol Explorer, Ideas Hub,
   outbox rotation, hot-reload TOML, batch operations, podman,
   build log streaming, etc.)
2. Verifier que verification.md documente 13/13 scope cuts

## Track F — Test delta verification

**Objectif** : verifier le delta cumule annonce vs reel.

1. `cargo nextest run --workspace --locked 2>&1 | tail -5` → 1227
2. `cd web && npm run test:unit 2>&1 | tail -5` → 256
3. `git log --oneline e5d6242..852c71b --grep="test\|Test"` →
   identifier les commits ajoutant des tests
4. Comparer avec le delta annonce (+11 Rust, +6 Vitest)

## Track G — Carry-over accountability

**Objectif** : verifier les compteurs carries et les escalades.

1. Verifier que P2-S53-outbox est documente FERME (3/3 MANDATORY)
   dans verification.md + CLAUDE.md
2. Verifier que P2-S53-browse_request est documente FERME (3/3 MANDATORY)
3. Verifier que P2-S54-windows-test et P2-S54-test-E2E-multi-noeuds
   sont a 3/3 MANDATORY pour S57
4. Verifier que P2-JITTER-SCOPE et P2-INVITE-U16-WIRE sont a 2/3
5. Verifier que les 7 items CLOSED ont bien ete resolus (cross-ref
   Tracks A-D ci-dessus)
6. Verifier que les 5 items dette pair (D4) sont documentes CLOSED
