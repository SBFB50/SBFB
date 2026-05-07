# Sprint 53 — Audit Findings (S54 Phase 0)

**Auditeur** : session Claude Code fraiche (independante de l'executeur S53).
**Date** : 2026-05-06
**HEAD audite** : `17dc535` (tip master post-S53 wrap-up)
**Base** : `b85a3a1` (tip master post-S52 audit PASS)
**Methode** : audit plan `sprint54_audit_plan.md` (7 tracks A-G)

## Verdict : PASS (0 P0, 0 P1, 7 P2, 2 P3)

Rigor signal G4 satisfait (7 P2 documentes, exploration exhaustive des
7 tracks avec evidence inline citee).

---

## Track A — P2P smoke test resultats

**Question** : resultats P2P documentes credibles et complets ?

**Evidence** :
- Phase A review : LAN Win-Mac bidirectionnel (node_ids documentes)
- Phase B review : WAN dev-VPS Helsinki bidirectionnel (3 OS, build times)
- Niveau 1 atteint : daemon demarre sur Windows, macOS ARM, Linux VPS
- Niveau 2 atteint : peer discovery via iroh relays EU
- Niveau 3 initialement NON atteint (gossip deadlock) → trace dans
  Phase D fix → resolu Phases F/G
- `running.json` : 18 references dans shell-daemon (cli.rs, main.rs,
  runtime.rs), write + cleanup + tests

**Croisement verification.md** : rows 15-21 (macOS build, macOS daemon,
Linux build, Linux daemon, Niveau 1/2/3) coherents avec les reviews.

**Bugs traces** :
- gossip deadlock (Phase B P1) → Phase D (bootstrap from attention set)
- ephemeral identity (Phase B/D discovery) → Phase E (file-backed node key)
- blocking subscribe (Phase B/D analysis) → Phase F (non-blocking + outbox)
- missing pull mechanism (Phase F observation) → Phase G (browse_request)

**Signal** : 0 finding. Resultats credibles et tracables.

---

## Track B — Gossip pipeline correctness

**Question** : chaine gossip coherente end-to-end ?

**Evidence** (grep runtime.rs) :
- `subscribe_topic()` non-bloquant via iroh subscribe() — line ~951
- outbox `Vec<Vec<u8>>` — line 991
- `GossipCmd::Outbox(envelope)` — line 1077
- `GossipCmd::RequestBrowse` — line 1085
- `GossipEvent::NeighborUp` — line 1044, replay outbox
- `is_browse_request(&payload)` — line 1031, discriminant replay
- POST `/api/daemon/browse/pull` — http.rs:247

Frontend wire : Browse.tsx:82 bouton "Rafraichir" → browsePull() → daemon.ts

Tests : 3 tests browse_request dans publish.rs
(is_browse_request_accepts_valid, rejects_project, rejects_garbage).
0 test filtre par pattern "gossip" — infrastructure testee indirectement
via runtime start/shutdown tests.

**Findings** :
- **P2** : browse_request sans rate-limit per-peer (runtime.rs:1085).
  PoW mitige le flooding trivial mais un peer motive pourrait forcer
  des replays repetes. Carry S54 (1/3) — P2-S53-browse_request.
- **P2** : outbox in-memory (runtime.rs:991). Crash daemon = perte des
  annonces. Carry S54 (1/3) — P2-S53-outbox.

---

## Track C — Node identity persistence

**Question** : node key persistent correctement implemente et securise ?

**Evidence** (runtime.rs:122-142) :
- `load_or_generate_node_key(root: &Path) -> Result<[u8; 32]>` : lit
  `<root>/node_key` (32 bytes raw Ed25519), ou genere + persiste si
  absent/malforme.
- Appel runtime.rs:291 dans `DaemonRuntime::start` chemin None.
- `std::fs::write(&path, secret)` sans appel `set_permissions`.
- grep `permissions|0600|set_permissions` dans runtime.rs → 0 resultat.

**Findings** :
- **P2** : node_key permissions non restreintes (runtime.rs:139).
  Sur Unix le umask donne 0644 (world-readable). Un
  `std::fs::set_permissions(..., Permissions::from_mode(0o600))` manque.
  Carry S54 (1/3) — P2-S53-node_key perms.
- **P3** : `secret_bytes` ([u8; 32] stack) non zeroise apres usage
  (runtime.rs:291-297). Le chemin launcher (runtime.rs:118) appelle
  `.zeroize()` mais `load_or_generate_node_key` ne le fait pas.
  Asymetrie mineure (secret deja sur disque en clair).

---

## Track D — Route collision fix (Phase A)

**Question** : fix de collision route correct sans regression ?

**Evidence** (http.rs:1-382) :
- Module doc http.rs:1-32 : routes daemon sous `/api/daemon/*`.
- 40+ routes enregistrees sous `/api/daemon/*`, `/api/canary/*`,
  `/api/v1/*`, `/health`, `/auth/token`, `/blob-serve/{hash}/{*path}`.
- SPA fallback : http.rs:376-378
  `ServeDir::new(root).fallback(ServeFile::new(root.join("index.html")))`
  — applique comme `fallback_service()` apres toutes les routes API.
- Test `unknown_route_returns_404` : PASS.
- Tests SPA fallback : `spa_fallback_serves_browse_as_html_document`,
  `spa_fallback_serves_curators_as_html_document`.

**Signal** : aucune collision. Routes API en premier, SPA fallback en
dernier. Architecture correcte.

**Findings** :
- **P3** : docs securite (LOOPBACK_ENDPOINTS, LAUNCHER) referencent
  les anciens noms courts. Cosmetique — P2-S53-route collision doc.

---

## Track E — Edition 2024 / unsafe set_var scope cut

**Question** : re-scoping justifie et correctement documente ?

**Evidence** :
- `Cargo.toml` : `edition = "2021"` confirme.
- 70+ appels `set_var`/`remove_var` dans le workspace (grep : 17
  fichiers dont dns_fallback.rs, relay_config.rs, relay_pow_policy.rs,
  tls_pinning.rs, auth.rs, unlock.rs, paths.rs, config.rs).
- Phase C preflight (SCOPE-CUT-CONSISTENT) : "wrapping in edition 2021
  is incorrect; the fix is edition 2024 upgrade".
- Design review D4 : confirme que set_var est safe en edition 2021.
- CLAUDE.md : "RE-SCOPED : edition 2024 upgrade requise".
- verification.md row 22 : "RE-SCOPED (edition 2024 requis)".

**Signal** : re-scoping entierement justifie. Code compile sans erreur.

**Findings** :
- **P2** : P2-REVIEW-B-1-S51 re-scoped edition 2024 upgrade. 70+
  call sites a migrer. 3/3 MANDATORY S55. Carry documente a 3 endroits
  (CLAUDE.md, preflight, verification.md).

---

## Track F — Process meta

**Question** : process sprint respecte ?

**Evidence** :
- 15 commits (git log b85a3a1..HEAD) : 4 feat/fix + 10 chore(planning)
  + 1 chore(planning) kickoff. Atomicite respectee.
- Format commit titles : tous `feat(sprint53): Sprint 53 Phase X —
  titre` ou `chore(planning):` ou `fix(sprint53):`.
- Bodies : riches (detail changements, fichiers touches, delta tests
  cumule, Co-Authored-By).
- Reviews : 7/7 PASS (A, B, D, E, F, G, C).
- Preflights : 4/7 (A EXECUTE, B EXECUTE, D EXECUTE, C SCOPE-CUT-
  CONSISTENT). Manquants : E, F, G.
- Design review G1 : present (sprint53_design_review.md), D1-D4 scores.

**Phases E/F/G sans preflight** :
- E (node identity) : ajoutee ad hoc apres discovery Phase B/D
- F (gossip non-blocking) : ajoutee ad hoc, fix du P1 Phase B
- G (browse pull) : ajoutee ad hoc, completing gossip pipeline
- Justification documentee dans Phase C review (P2 process)
- Plan initial A/B/C ; phases D-G = reponses reactives aux bugs
  runtime decouverts pendant le smoke test

**Findings** :
- **P2** : Phases E, F, G sans artefact G8 preflight. Justifiable
  (phases reactives post-plan) mais affaiblit la tracabilite G8.
  Recommandation : documenter des criteres d'exemption pour les
  phases post-plan dans README.md §6.9. Carry process (1/3) —
  P2-S53-preflight process gap.

---

## Track G — G1 Design Review Board

**Question** : design review existe et couvre D1..D4 ?

**Evidence** (sprint53_design_review.md) :
- Fichier present (77 lignes, substantiel).
- D1 ✅ (build from source) : verifie Cargo.toml, cfg gates, CI.
- D2 ✅ (smoke test 3 niveaux) : verifie DaemonRuntime::start().
- D3 ⚠️ (VPS setup minimal) : finding localhost auth gate, analyse et
  ack'd (P2P iroh ≠ HTTP loopback).
- D4 ✅ (unsafe set_var) : finding corrige (edition 2021 = safe).
- Scoring G4 : 1 ⚠️ sur 4 = rigor signal satisfait.

**Signal** : 0 finding. Design review complete et rigoureuse.

---

## Compteurs tests verifies independamment

| Suite | Self-report | Audit independant | Match |
|---|---|---|---|
| Rust nextest | 1206/1206, 0 fail | 1206 (1 flaky pre-existant R5) | oui |
| Vitest | 250/250 | 250/250 | oui |

Le test flaky `probe_and_cache_with_quorum_majority_continues_to_dial`
est documente dans le kickoff R5 et la Phase D review comme
pre-existant. Non imputable S53.

---

## Synthese findings

| # | Severity | Track | Description | Carry |
|---|---|---|---|---|
| 1 | P2 | B | browse_request sans rate-limit (runtime.rs:1085) | P2-S53-browse_request 1/3 |
| 2 | P2 | B | outbox in-memory volatil (runtime.rs:991) | P2-S53-outbox 1/3 |
| 3 | P2 | C | node_key permissions 0600 manquantes (runtime.rs:139) | P2-S53-node_key 1/3 |
| 4 | P2 | E | edition 2024 upgrade re-scoped (70+ call sites) | P2-REVIEW-B-1-S51 3/3 MANDATORY |
| 5 | P2 | F | Phases E/F/G sans preflight G8 (process gap) | P2-S53-preflight-process 1/3 |
| 6 | P2 | D/A | gossip params struct 9 args (runtime.rs) | P2-S53-gossip-params 1/3 |
| 7 | P2 | A | periodic republish manquant (NeighborUp only) | P2-S53-periodic-republish 1/3 |
| 8 | P3 | C | secret_bytes non zeroise (runtime.rs:291) | mineur |
| 9 | P3 | D | docs securite old endpoint names | P2-S53-route-collision-doc 1/3 |

---

## Carries resume post-audit (pour S54 kickoff)

### Carries entrants confirmes (inchanges)

| Item | Compteur S54 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-REVIEW-B-1-S51 edition 2024 upgrade | **3/3 MANDATORY** | re-scoped S53 |
| P2-REVIEW-A-1-S52 nextest timeout | 2/3 | S52 review |
| P2-REVIEW-B-1-S52 Woodpecker E2E | 2/3 | S52 review |
| P2-REVIEW-B-2-S52 GHA 9/9 | 2/3 | S52 review |
| P2-AUDIT-1-S52 images CI pin | 2/3 | S52 audit |

### Carries S53 NEW confirmes par audit

| Item | Compteur S54 | Source |
|---|---|---|
| P2-S53-outbox non-persistant | 1/3 | S53 Phase F review + audit |
| P2-S53-browse_request rate-limit | 1/3 | S53 Phase G review + audit |
| P2-S53-gossip params struct | 1/3 | S53 Phase D review + audit |
| P2-S53-node_key perms 0600 | 1/3 | S53 Phase E review + audit |
| P2-S53-route collision doc | 1/3 | S53 Phase A review + audit |
| P2-S53-periodic republish | 1/3 | S53 Phase F review + audit |
| P2-S53-preflight E/F/G process gap | 1/3 | S53 Phase C review + audit |

### Alerte S54 pair

4 items a 2/3 (nextest timeout, Woodpecker E2E, GHA 9/9, CI image
pinning) deviennent 3/3 MANDATORY S55 si non adresses S54.
P2-REVIEW-B-1-S51 est deja 3/3 MANDATORY.
Sprint 54 pair → phase dette obligatoire (§6.2.1 Regle 1).

---

## Recommandation

Sprint 53 **PASS**. L'audit gate S53 est leve. Sprint 54 peut demarrer.

Les 7 phases ont ete livrees avec discipline (commits atomiques, reviews
independantes, delta tests documentes). Les resultats P2P sont credibles
et verifiables (cross-reference logs/reviews/verification). Le re-scoping
unsafe set_var est techniquement justifie. La chaine gossip est coherente
end-to-end.

Attention S54 : 5 items MANDATORY a 3/3 en vue (1 edition 2024 deja la +
4 items a 2/3 qui basculeraient). Sprint pair = phase dette obligatoire.
