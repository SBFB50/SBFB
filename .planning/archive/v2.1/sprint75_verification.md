# Sprint 75 — Verification (self-report fail-fast)

> Self-report ecrit par l'agent livreur en Phase G (2026-06-11). Valeur
> limitee par construction — la verification independante est l'audit gate
> S76 Phase 0 (`sprint76_audit_plan.md`, 13 tracks).

## §1 HEAD entree / HEAD sortie

- **HEAD entree** : `9b034c1` (handoff kickoff pivot, post-S74 ferme
  `bede850` + hotfixes #1-#8 + handoffs).
- **HEAD sortie** : le commit `feat(daemon): Sprint 75 Phase G — wrap-up +
  survives-VPS-death acceptance + S74 hygiene carries` (ce commit ; tip code
  Phase F = `4f52bea`).

## §2 Commit stack

`git log --oneline master ^9b034c1` (17 commits avec ce wrap-up) :

```
<G>      feat(daemon): Sprint 75 Phase G — wrap-up + survives-VPS-death acceptance + S74 hygiene carries
035a4f7  docs(planning): handoff prompt for the next session (S75 Phase G)
4f52bea  feat(shell): Sprint 75 Phase F — node-centric Browse (nodes list + node catalog + add-anchor)
491b3c8  docs(planning): handoff prompt for the next session (S75 Phase F)
1486fc9  feat(daemon): Sprint 75 Phase E — headless VPS anchor (config-driven seed driver + signed authoring)
41b13e3  docs(planning): handoff prompt for the next session (S75 Phase E)
0010450  feat(core+daemon): Sprint 75 Phase D — multi-provider pull + node identity exposure
9f7de7f  docs(planning): handoff prompt for the next session (S75 Phase D)
821aa8c  feat(daemon): Sprint 75 Phase C — node-directory ingest + remote-catalog durability (boot re-pull)
<handoff C>
f6637d3  feat(core+daemon): Sprint 75 Phase B — NodeDirectoryEntry + DOMAIN_NODE_DIRECTORY_V1 + generic ingest gate + authoring route
<handoff B>
479a87c  feat(core+daemon): Sprint 75 Phase A — re-mint PoW + endpoint address on outbox replay
e3c3fb6  chore(planning): Sprint 75 Phase A preflight (SCOPE-CUT-CONSISTENT)
f008433  chore(planning): Sprint 75 kickoff + plan + design review + pivot proposal (Cas C)
0e2fb6b  chore(planning): Sprint 75 Phase 0 — S74 audit findings (PASS)
```

## §3 How to re-run

```bash
# Rust — Windows natif (PowerShell, racine repo)
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release

# Rust — Docker Linux canonique (image re-pinnee S75-G : rust:1.94 + libgtk-3-dev)
docker build -t sbfb-ci docker/ci
docker run --rm -v "C:\Users\FlowUP\Documents\Code\nexus:/workspace" \
  -v "sbfb-ci-target:/workspace/target" -w /workspace sbfb-ci \
  cargo nextest run --workspace --locked

# Frontend
(cd web && npx tsc --noEmit -p tsconfig.app.json && npm run lint && \
  npm run test:unit && npm run test:coverage && npm run build && \
  npm run size && bash scripts/scan-en-strings.sh)

# Acceptance live (checklist horodatee complete : .git/S75_PHASE_G_ACCEPTANCE.md,
# resumee row 22 ci-dessous) — requiert SSH mac + vps et le binaire VPS
# builde via rust:1.94-bookworm (glibc 2.36 <= VPS Ubuntu 24.04 2.39).
```

## §4 Checklist fail-fast (plan §5, Observed rempli)

| # | Check | Critère | Observed |
|---|---|---|---|
| 1 | fmt | exit 0 | ✅ exit 0 (apres `cargo fmt --all` sur les fixes G) |
| 2 | clippy | 0 warn | ✅ `Finished` 0 warning, exit 0 |
| 3 | nextest workspace | 0 fail | ✅ **1755/1755 passed, 0 skipped** (Windows natif ; S74 sortie 1674 + 81 = 1755 ; le +1 vs la 1re mesure 1754 = le test quorum review COMPLETE-2) |
| 4 | doctests | 0 fail | ✅ `cargo test --doc` ok (6 passed, 0 failed) |
| 5 | release | OK | ✅ `cargo build -p nexus-shell-daemon --release` exit 0 |
| 6 | Docker Linux canonique | 0 fail | ✅ **1759/1759 passed, 0 skipped** (run final seul, sans charge cargo parallele ; les runs precedents [16 timeouts `operator_server` + flake `sigint`] = non-fidelite bind-mount 9p sous contention, PAS des regressions ; +4 vs Windows = `#[cfg(unix)]` structurel ; image re-pinnee rust:1.94+libgtk-3-dev apres derive locale trixie) |
| 7 | FIX-A bug live | PASS | ✅ tests reels : `replay_remints_own_ticket_to_current_address` + `replay_does_not_remint_a_third_party_address` + `replay_keeps_stale_ticket_when_blob_is_gone` (runtime.rs ; le nom du plan `stale_announcement_accepted_by_fresh_receiver` etait un placeholder kickoff jamais cree — divergence consignee) **+ E2E live row 22 : pair frais Mac decouvre 4 apps `direct` publiees plusieurs JOURS avant** |
| 8 | DOMAIN disjoint | unique | ✅ `DOMAIIN_NODE_DIRECTORY_V1 = b"nexus-node-directory-v1"` unique parmi les 18 domaines de canonical.rs |
| 9 | cross-domain replay rejet | PASS | ✅ `node_directory_cross_domain_signature_rejected` + `node_directory_cross_domain_bytes_differ` (node_directory.rs:549/566) |
| 10 | durabilité boot re-pull | PASS | ✅ `boot_repull_restores_remote_catalogs` (iroh_runtime.rs:1969) **+ live : locator `anchors.json` persiste sur le VPS, 2e boot re-pull → driver pin** |
| 11 | multi-provider fallback | PASS | ✅ `fetch_falls_back_to_seeder_when_anchor_offline` (blobs.rs:423) |
| 12 | VPS seed headless | PASS | ✅ `boot_seed_driver_pins_configured_projects` (http.rs:5150) **+ live : journal VPS « app pinned + kept online … held_locally=false »** |
| 13 | lock-3 tripwire | PASS | ✅ grep `135.181.42.188` + node_id VPS dans crates/ = 0 ; `default_curators` `#[serde(default)]` vide compile ; tests `default_curator_empty_when_section_absent` + tripwire `[seed]` (config.rs:634) + `default_curators_returns_empty_when_unconfigured` |
| 14 | 0 bump wire | PASS | ✅ tous les `*_FORMAT_VERSION` = 1 (FEED/CURATOR_LIST/POW/TASK/KEY_ROTATION/NODE_DIRECTORY/SEED) ; `INVITE_FORMAT_VERSION=2` pre-existant (pas un bump S75) |
| 15 | WIRE-1 searchable | PASS | ✅ `release_published_searchable_by_name` (search.rs:1015) |
| 16 | web tsc | 0 | ✅ exit 0 |
| 17 | web lint | 0 err | ✅ 0 errors (5 warnings pre-existants) |
| 18 | web Vitest | pass | ✅ **367/367** (run propre post-Docker ; les runs SOUS CHARGE cargo×3 avaient 4-6 timeouts AddAnchorDialog/GpuConsentDialog, re-PASS 19/19 isoles puis suite complete verte — classe `vitest_env_variance`) |
| 19 | web coverage | ≥ seuils | ✅ 87.17/79.01/85.92/88.5 ≥ 85/85/78/85 |
| 20 | web build+size | 6/6 | ✅ build ok + size-limit 6/6 |
| 21 | scan FR | clean | ✅ « src/ is French-only, clean » |
| 22 | survives-VPS-death | démontré | ✅ **PASS LIVE 2026-06-11** (checklist horodatee `.git/S75_PHASE_G_ACCEPTANCE.md`) : VPS Hetzner Ubuntu 24.04 install stock unit Phase E → 1er boot 0 crash-loop, `systemd-analyze security` **1.7 OK**, QUIC v4+v6 sous seccomp, SBFB_HOME resolu ; chaine PULL E2E WAN : annuaire Windows rev1-4 → ingest gossip VPS → locator persiste → 2e boot re-pull → boot driver fetch WAN `held_locally=false` → pin sbfb-explorer + keep_online ; pair FRAIS Mac (state vierge) → 7 entrees (4 direct re-mintees **= C6**, 3 nodedirectory) → **VPS `systemctl stop` → render `sbfb-explorer/index.html` HTTP 200 19 926 o 0,27 s + Browse intact** ; verrous (a) grep 0 + test-pins, (b) l'ancre du flux = Windows (pas le VPS) |
| 23 | 5 verrous | PASS | ✅ review-deep Phase G (dimension garde-fous) + lock-3 row 13 + lock-4 tests F (`source==="direct"` exact) + verrou-5 accept-list vide = borne (http.rs:5264) |
| 24 | carries closed | CLOSED | ✅ WIRE-1/WIRE-2/DBQ-1 (C), PULL-2+SEED-1/SEED-2 (D), WEB-1 (F), CARRY-2/CARRY-5/PULL-1/FORK-1 (G : `reject_result_on_guardrail_trip` 2 chemins + clamp offset/q + `strip_zip_member` + `MAX_ARCHIVE_ENTRIES`) |

## §5 Métriques sprint

| Suite | Avant (S74 sortie) | Après | Delta |
|---|---|---|---|
| Rust nextest Windows natif | 1674 (S74 sortie `bede850`) | **1755** (0 fail, 0 skip) | +81 (A +8 [→1682], B +32, C +10, D +11, E +13, F +2, G +5 ; somme = 81) |
| Rust nextest Docker Linux canonique | 1678 (S74 sortie) | **1759** (0 fail, 0 skip) | +81 (meme breakdown A-G ; +4 absolu vs Win a chaque point = `#[cfg(unix)]` structurel) |
| Vitest `web/` | 331 | **367** | +36 (C +3, F +33) |
| Vitest factory-operator | 7 | 7 | 0 (non touche) |
| size-limit | 6/6 | 6/6 | 0 (263.13 kB main, css 128.68/130) |
| coverage web | 86.91/78.63/85.82/88.23 | 87.17/79.01/85.92/88.5 | tous ≥ seuils 85/85/78/85 |

## §6 Surface nouvelle livrée

- `crates/nexus-core-rs/src/node_directory.rs` (NEW) — `NodeDirectoryEntry`
  signe + caps + `is_valid_archive_hash` ; `canonical.rs` +`DOMAIN_NODE_DIRECTORY_V1`.
- `crates/nexus-core-rs/src/blobs.rs` — `fetch_hash_multi` / `fetch_and_pin_multi`
  (multi-provider bare-hash, `MAX_FETCH_PROVIDERS=16` in-primitive).
- `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` — ingest directory
  subscription-gated + `repull_directories` + locator `anchors.json` ;
  `config.rs` +`[seed]` ; `browse.rs` +`BrowseSource::NodeDirectory`.
- `crates/nexus-shell-daemon/src/http.rs` — routes `directory/publish`,
  `/nodes`, `seed/request`, boot seed driver duress-gate ; `runtime.rs`
  re-mint replay + `boot_driver_handle` ; `seed_registry.rs` prod
  (SEED-1/SEED-2 + lowercase).
- `deploy/nexus-shell-daemon.service` + `deploy/config.toml.example` (NEW) —
  unit systemd durcie (1.7 OK live).
- `web/src/pages/{Nodes,NodeCatalog}.tsx` (NEW) + `AddAnchorDialog.tsx` (NEW)
  + AvailabilitySheet/VerificationDetail (verrou-4, WEB-1, badge Q7).
- Phase G : `validator.rs` `reject_result_on_guardrail_trip` ; `deploy.rs`
  `strip_zip_member` ; `fork.rs` `MAX_ARCHIVE_ENTRIES` ; `http.rs` clamps
  search ; THREAT_MODEL §15.1 (v8) ; PATTERNS §P59 (rust) + P37 (shell) ;
  META-1 ; LT-2 ARME + dry-run Radicle prive ; `docker/ci/Dockerfile` re-pin.

## §7 Ce que le sprint n'a PAS livré (scope cuts respectés, kickoff §9 — 12/12)

1. ❌ SearchManifest (digest Bloom, agrégation, query fédérée) — DIFFÉRÉ tient
   (D3/s73 ; l'annuaire n'est PAS le SearchManifest, pivot_proposal).
2. ❌ Tantivy — gelé (gate post-S75 >50K docs), FTS5 reste l'engine.
3. ❌ GC reaper / budget disque enforced — déféré post-launch (policy config
   seule borne).
4. ❌ Recherche cross-nœud fédérée — hors scope.
5. ❌ Approbation pair pour seed distant — inchangé (volontaire/invite S74).
6. ❌ Mobile/Electron — non.
7. ❌ Migration wire post-tag — 0 bump (`*_FORMAT_VERSION` tous inchangés).
8. ❌ GPU partagé cross-machine — S76 (amendement roadmap v5).
9. ❌ Sharding pipeline — S77.
10. ❌ Kudos-threshold tuning empirique — post-launch.
11. ❌ Multi-ancre UX avancée (priorité/fallback chains) — différé (S75 livre
    l'abonnement à N ancres seulement).
12. ❌ Bloom/Merkle digest — non posé.

## §8 Findings carry-over for memory (G6, max 5)

1. **`SeedAnnounced` ne converge pas cross-nœud** (acceptance live :
   `peer_count:0` Windows ET Mac ~10 min après le pin VPS) — best-effort par
   design mais un registre toujours-vide affaiblit le dial-set multi-provider ;
   lié à PULL-3 → audit S76.
2. **L'annuaire d'un seeder n'annonce pas ce qu'il seede** (`catalog_len:0`
   live) — conforme verrou-4 mais un pair frais dont la seule ancre est le
   seeder ne peut pas DÉCOUVRIR l'app servie → question design PO, audit S76.
3. **Image `sbfb-ci` dérivable silencieusement** : la locale avait été
   rebuildée trixie/rustc 1.95/glibc 2.41 (≠ Dockerfile rust:1.94) — binaire
   VPS incompatible + suite atk-manquante ; Dockerfile re-pinné +libgtk-3-dev,
   builds binaires VPS = `rust:1.94-bookworm`.
4. **PowerShell 5.1 `2>&1` sur exe natif = faux échec** (`$?` false sur exit
   0) — utiliser `$LASTEXITCODE`, jamais `$?` après une redirection native.
5. **Fenêtre morte premier-boot du driver OBSERVÉE live** (journal « not
   resolvable yet — skipped » puis pin au boot suivant) — le remède opérateur
   documenté fonctionne ; re-drive-on-ingest reste le vrai fix → S76.

## §9 Checkpoint de clôture (plan §9)

- [x] 24/24 fail-fast verts — rows 1-24 ✅ ci-dessus (dont Docker Linux
  1759/1759 et survives-VPS-death live).
- [x] 7 commits feat A-G (A `479a87c` B `f6637d3` C `821aa8c` D `0010450`
  E `1486fc9` F `4f52bea` G ce commit).
- [x] `sprint75_verification.md` (ce fichier) + `sprint76_audit_plan.md`
  (13 tracks, 6/6 phase reviews parsées, tous P2/P3 routés).
- [x] PATTERNS rust (§P59 + META-1) + shell (P37) à jour.
- [x] roadmap_v5 amendée (S75=découverte LIVRÉ, GPU→S76, sharding→S77).
- [x] 11 carries S74 CLOSED (row 24) + 2 doc/process (META-1, CARRY-1/LT-2
  ARME + dry-run privé fait).
- [x] memory + SPRINT_LOG row S75 + CLAUDE.md à jour.
- [x] D3 sign-off PO consigné (kickoff §13 + pivot_proposal).
