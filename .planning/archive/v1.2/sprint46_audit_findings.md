# Sprint 46 — Audit findings (Sprint 47 Phase 0)

**Auditeur** : session fraiche (pas la session qui a code S46).
**Tip d'entree** : `706968b` (HEAD, Phase D wrap-up).
**Tip feat** : `812f3ba` (Phase C, dernier feat commit).
**Diff audite** : `d1dd4bd..812f3ba` (3 fichiers Rust, 7 fichiers
frontend, 9 fichiers planning + 9 archives S45).
**Timebox** : 30 min. **Methode** : 32 check items, 6 tracks,
lecture code + grep evidence.

---

## Verdict : PASS

0 P0, 0 P1, 1 P2, 2 P3. S47 Phase A peut demarrer directement.

---

## Track A — Integration tests MANDATORY (Phase A)

| Item | Verdict | Evidence |
|---|---|---|
| A-1 mk_state() canary_input | ✅ | `http.rs:1724` `canary_input: Some(Arc::new(...))` + `coordinator_db`, `canary_registry` ajoutes |
| A-2 19 tests 12 routes | ✅ | consent 6 + files 5 + canary 4 + contributor 4 = 19 |
| A-3 consent tests | ✅ | GET default level=1 (3823), POST level 0 (3827), level 5 (3845), invalid hex (3863), missing id (3881, 3899). Happy path = carry P2-REVIEW-A-1-S46 |
| A-4 files tests | ✅ | invalid sha 400 (3919, 3952), not found 404 (3935, 3968), too large 413 (3985). Upload = carry P2-REVIEW-A-2-S46 |
| A-5 canary tests | ✅ | freshness 200 (4005, 4022), inject-rate (4038), divergence empty (4061). canary_input=Some (1724) |
| A-6 contributor tests | ✅ | project empty (4083), invalid hex (4104), envelope 404 (4120), invalid hex (4140) |

## Track B — Dette pair S44 (Phase B)

| Item | Verdict | Evidence |
|---|---|---|
| B-1 as_str/serde | ✅ | `grep -rn "as_str.*match" crates/nexus-shell-daemon/src/` = 0 matches |
| B-2 kudos pagination | ✅ | `kudos_api.rs:21-27` KudosListQuery limit default 100, cap 500 (line 61), skip/offset (line 63-65). Count post-skip/take (line 77) |
| B-3 shell discover self | ✅ | `http.rs:4378-4398` test valide count=1 + node_id == state.node_id |
| B-4 pagination tests | ✅ | kudos_entries_with_limit_offset (4446), tasks_list_with_limit (4488) |
| B-5 diagnostic fallback | ✅ | `grep -n "unwrap_or_default" diagnostic_api.rs` = 0. Erreurs propagees 500 (lines 26-68) |

## Track C — Integration tests routes recentes (Phase B)

| Item | Verdict | Evidence |
|---|---|---|
| C-1 17 tests 14 routes | ✅ | invite 3 + quarantine 3 + tasks 2 + kudos 2 + health 1 + shell 1 + diagnostic 1 + worker_state 1 = 14 routes + 3 dette (pagination + error prop) = 17 tests |
| C-2 invite_create 201+id | ✅ | `http.rs:4177` assert 201, `http.rs:4180` assert id.as_str().is_some() |
| C-3 quarantine 404 | ✅ | flush_not_found (4254), drop_not_found (4270) |
| C-4 diagnostic error prop | ✅ | `http.rs:4518` poison mutex → 500 |

## Track D — Frontend direct-daemon (Phase C)

| Item | Verdict | Evidence |
|---|---|---|
| D-1 coordinator.ts paths | ✅ | Routes migrees /api/v1/*: tasks (455), kudos (473), invite (486/490/498), health (432), shell (709), worker (713). Routes gardees /app/* (504-649) + /project (444) |
| D-2 error classes | ✅ | ApiProtocolError (23), ApiHttpError (43). Aliases deprecated (58, 60) |
| D-3 daemon.ts callProxy→callDaemon | ✅ | callDaemon() (203). Paths /info (265), /curators (271), /browse (300) |
| D-4 proxy envelope supprime | ✅ | Aucun ProxyDataEnvelopeRaw/ProxyUnavailableEnvelope/ProxyErrorEnvelope. DaemonResult<T> conserve (194) |
| D-5 schemas Zod | ✅ | HealthSchema (158), TaskRowSchema (178), KudosEntrySchema (214), ShellDiscoverResponseSchema (362) |
| D-6 callers | ✅ | KudosTab.tsx import coordinator (23). TasksTab.tsx import coordinator (20). ProjectDetail.tsx worker_pubkey_hex→worker_node_id (diff) |
| D-7 tests | ✅ | daemon.test.ts reecrit sans proxy envelope (-429/+253 LOC). BrowsedProject.test.tsx adapte |
| D-8 Vitest -1 | ✅ | Test proxy envelope "503 body unreadable" retire. 503 toujours couvert par callDaemon() (daemon.ts:226) |

## Track E — Process / meta

| Item | Verdict | Evidence |
|---|---|---|
| E-1 G8 3/3 | ✅ | A EXECUTE (phase_A_preflight.md), B EXECUTE (phase_B_preflight.md), C EXECUTE (phase_C_preflight.md). 0 DESIGN-CONFLICT |
| E-2 scope cuts 13/13 | ✅ | `git diff --name-only d1dd4bd..812f3ba` : 0 fichier hors-scope. Pas d'events.py/SSE, pas de deploy.rs/apps.rs tests, pas de MCP/PyO3/coordinator suppression |
| E-3 6 carries resolus | ✅ | verification.md §3 : P2-AUDIT-A-1-S43 + 5 items S44 (as_str, kudos, shell, pagination, diagnostic) |
| E-4 design review G1 | ✅ | sprint46_design_review.md present. D1 ✅, D2 ✅, D3 ⚠️, D4 ⚠️ |
| E-5 sprint pair dette | ✅ | Phase B = phase dette obligatoire (plan.md §B.1) |
| E-6 reviews 3/3 | ✅ | phase_A_review.md + phase_B_review.md + phase_C_review.md |
| E-7 preflights 3/3 | ✅ | phase_A_preflight.md + phase_B_preflight.md + phase_C_preflight.md |

## Track F — Doc coherence

| Item | Verdict | Evidence |
|---|---|---|
| F-1 CLAUDE.md | ✅ | "Sprints 0-46 CLOSED", S46 carries listes, compteurs 1168 Rust / ~1984 total |
| F-2 SPRINT_LOG.md | ✅ | Row S46 presente (theme + 3 phases + delta tests + fichiers + carries) |
| F-3 memory tip | ✅ | nexus_grid_pivot.md Tip `812f3ba` = dernier feat commit S46 (convention tip = feat, pas chore wrap-up `706968b`) |

---

## Findings

### P2 (1)

- **P2-AUDIT-B-1-S46** : `kudos_api.rs:77` — le champ `count`
  dans la reponse paginee est la taille de la page (apres
  skip/take), pas le nombre total d'entrees. Le frontend
  `KudosTab.tsx:41` affiche `count` comme total d'entrees :
  un utilisateur avec 150 entrees voit "100 entree(s)" (limit
  default 100) au lieu de "150 entree(s)". Regression UX
  introduite par l'ajout de pagination S46 Phase B. **Etend le
  carry P2-REVIEW-B-1-S46** (SQL pagination) : la migration
  SQL devra ajouter `total_count` a la reponse, et le frontend
  adapter l'affichage. En attendant, le frontend pourrait passer
  `limit=9999` pour contourner. 1/3.

### P3 (2)

- **P3-AUDIT-A-1-S46** : test `consent_whitelist_add_invalid_
  node_id_400` a `http.rs:3863` — le nom reference "node_id"
  mais le body envoie `{"project_id": "not-valid-hex"}`. Le
  comportement du test est correct (hex invalide → 400), mais
  le nom est trompeur pour un lecteur a froid.

- **P3-AUDIT-A-2-S46** : les routes canary utilisent le prefix
  `/api/canary/...` (sans `v1`) alors que toutes les autres
  routes daemon utilisent `/api/v1/...`. Pattern pre-existant
  (pas introduit par S46), les tests S46 suivent correctement
  les paths existants. Candidat a une normalisation future.

---

## Carries confirms S47

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 10+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 Day 0 |
| P2-REVIEW-A-1-S45 diagnostic Err path non teste | 2/3 | |
| P2-REVIEW-A-2-S45 invite ID collision multi-daemon | 2/3 | |
| P2-REVIEW-B-1-S45 modules Python suppression differee | 2/3 | |
| P3-AUDIT-B-4-S45 TOCTOU canary reload | 1/3 | |
| P2-INT-1-S46 integration tests deploy.rs + apps.rs | 1/3 | |
| P2-INT-2-S46 integration test auth/token | 1/3 | |
| P2-REVIEW-A-1-S46 consent happy path | 1/3 | |
| P2-REVIEW-A-2-S46 files upload happy path | 1/3 | |
| P2-REVIEW-B-1-S46 kudos SQL pagination + UX count | 1/3 | etendu par P2-AUDIT-B-1-S46 |
| P2-REVIEW-C-1-S46 app-specific schema drift | 1/3 | |
| P2-REVIEW-C-2-S46 deprecated error class aliases | 1/3 | |

---

## Recommendation

Verdict PASS — 0 P0, 0 P1, 1 P2. Le P2 est une regression UX
mineure qui ne manifeste pas a l'echelle pre-launch (< 100
entries kudos). Pas de fix bloquant. S47 Phase A peut demarrer.
