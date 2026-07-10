Verdict global : **PARTIEL**, pas de P0/P1 trouvé. Le P1 interne SI-9 semble corrigé : le timeout couvre bien `open_bi + write_frame + read_frame`. Le diff est cohérent avec Phase I, mais il reste un vrai gap P2 sur la borne de readiness et quelques résidus P3 de documentation/worktree.

**Livrables**
1. `shard_session.rs` : **PARTIEL**  
   OK sur mint head-as-member (`crates/nexus-shell-daemon/src/shard_session.rs:491`), gate registre signature/binding/contiguite/membership/adresses/duplicate (`:337`, `:393`), Debug count-only (`:323`), manifest signé et rejet federation (`:513`, `:532`, `:571`), mount gate avant reseau + teardown readiness (`:648`, `:656`, `:679`), RunProof prod + verify (`:986`, `:989`), HUB + fallback replay (`:848`, `:938`).  
   Gap P2 : `probe_shard_readiness` borne `open_shard_connection` par `deadline`, puis redémarre un budget RTT après l’ouverture (`:595`, `:602`, `:607`), donc le couple handshake+RTT peut durer presque 2x deadline.

2. `http.rs` routes shard-session : **OK**  
   Routes dans `authed_routes`, avant middleware auth/Host/Origin (`crates/nexus-shell-daemon/src/http.rs:282`, `:323`, `:529`). Stub remplacé par lecture registre (`:2154`). Duress avant travail pour group/mount/generate (`:2219`, `:2268`, `:2326`). Generate 202 fire-and-forget + mismatch path/body (`:2335`, `:2357`). Result envelope `{found,result}` avec champs harness (`:2381`, `:2388`). Empty-envelope et privacy testés (`:5503`, `:5541`, `:6536`).

3. Schémas core : **OK**  
   `ShardSessionView.rtt_frontier_ms` nullable requis (`crates/nexus-core-rs/src/schemas/shard.rs:79`). Nouveaux DTO result (`:108`, `:158`), schemas (`:205`), snapshots (`:255`), whitelist view/result sans identités (`:398`, `:433`). `lib.rs` et `schemas/mod.rs` ré-exportent (`crates/nexus-core-rs/src/lib.rs:182`, `crates/nexus-core-rs/src/schemas/mod.rs:45`). `check-frontier-contracts` est vert.

4. CLI opérateur : **OK**  
   Sous-commande complète (`crates/nexus-shell-daemon/src/cli.rs:155`, `:178`). Clef persistante avec refus taille invalide (`crates/nexus-shell-daemon/src/main.rs:128`). Auto-discovery `running.json` + token bootstrap (`:171`, `:189`). `serve` vérifie group signé + `is_member`, expose `SHARD_ALPN` + EchoForwarder, imprime JSON (`:250`, `:257`, `:265`, `:274`). Aucun changement de dépendances détecté.

5. Web API : **OK**  
   Zod accepte `rtt_frontier_ms` nullable optional pour version-skew (`web/src/api/daemon.ts:545`, `:553`). Envelope reste `.strict()` (`:569`). Test fixture renforcé avec RTT (`web/src/api/__tests__/daemon.test.ts:1017`) et nouveau test old-daemon tolerance (`:1036`).

6. `sprint81_plan.md` : **PARTIEL**  
   L’amendement Phase J pose bien baseline HUB, pas Petals direct-s2s, churn et RunProof driver (`.planning/active/sprint81_plan.md:381`). Mais le bloc Phase I garde une formulation ambiguë “session shard 2-machines réelle” (`:368`), contraire au cadrage “Phase J = live 2-machines”.

7. Artefacts process : **OK**  
   Preflight en `PLAN-ADAPT` (`.planning/active/sprint81_phase_i_preflight.md:3`). Review avec header unique `PASS-PENDING` et boucle post-FAIL documentée (`.planning/active/sprint81_phase_i_review.md:3`, `:5`).

**Invariants**
- 0 bump wire : **OK**. `compute_group.rs`, `shard_plan.rs`, data-plane `shard.rs` non modifiés; `sbfb/shard/1`, plan/run/feed v1 inchangés.
- 0 dépendance nouvelle : **OK**. Aucun `Cargo.toml`, `Cargo.lock`, `package.json` dans le diff.
- Privacy SI-3/SI-4 : **OK fonctionnel**. Projections/test JSON excluent identités complètes; registry Debug count-only.
- Duress : **OK** pour group/mount/generate.
- Delta tests annoncé : **OK**. Vérifié 15 tests in-module shard session, 3 tests HTTP shard-session dont 1 net-new duress, 6 tests schemas shard dont whitelist result, 51 tests Vitest daemon.

**Vérifications lancées**
- `cargo test -p nexus-shell-daemon shard_session::tests --locked` : 15 passed.
- `cargo test -p nexus-shell-daemon shard_session_ --locked` : 3 passed.
- `cargo test -p nexus-core-rs schemas::shard::tests --locked` : 6 passed.
- `cd web && npm run test:unit -- daemon.test.ts` : 51 passed.
- `bash scripts/check-frontier-contracts.sh` : clean.

**GAPs**
- P0 : aucun.
- P1 : aucun.
- P2 : readiness deadline non holistique dans `probe_shard_readiness` (`shard_session.rs:595`, `:602`, `:607`).
- P3 : teardown fallback non explicite si le fallback `drive_hop` échoue après readiness (`shard_session.rs:943`), résidus prose “pas de live store / Phase J” dans front/tests/harness, et fichier untracked hors scope `.planning/research/psyche_nous_analysis_2026-07.md`.