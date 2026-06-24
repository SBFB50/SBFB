# Sprint 77 Phase A Review — WAN task delivery convergence

> Review profonde pré-Codex (agent `nexus-phase-review-deep`, fallback du Workflow
> review car ultracode toggle off au resume). Transcrite par l'exécuteur (contrainte
> d'indépendance G4 : l'agent de review n'écrit pas l'artefact lui-même).

## Verdict: PASS

0 P0/P1. 3 P2 + 2 P3 sur exploration exhaustive (rigor signal G4 satisfait, chaque
finding cite file:line vérifié). **Les 3 P2 ont été traités IN-PHASE** (voir §Résolution) —
il ne reste que 2 P3 documentés (déviation honnête + test de saturation absent, tous deux
acceptables). Codex 5/5 CONFIRMED (0 PARTIAL, 0 GAP) → promu PASS (voir §Codex
reconciliation).

## Scope & staging

Diff de phase atomique = 6 fichiers cohérents :
- `crates/nexus-core-rs/src/doc_sync.rs` (NOUVEAU) + `lib.rs` (`pub mod` + re-export)
- `crates/nexus-worker-core/src/engine/runtime.rs` (câblage engine + test capture)
- `crates/nexus-shell-daemon/src/dispatch_loop.rs` (3 tests convergence)
- `docs/rust/PATTERNS.md` §P63 + `docs/security/THREAT_MODEL.md` §15.3

Artefacts planning à inclure : `sprint77_phase_a_preflight.md` (G8), ce `review.md`,
le `codex_review.md` (à venir). **Exclu du commit de phase** :
`.planning/research/factory_embedded_ide_study.md` (recherche hors-Phase A, untracked).
T1 = `N-A-no-frontend-change` respecté (0 fichier `web/`).

## Three-block verification (driver)

fmt ✅ / clippy `--all-targets -D warnings` ✅ / nextest workspace **1811/1811**
(baseline 1805 + 6 Phase A, exact) / doctests ✅ / release build ✅. Code de prod : 0
`unwrap()`/`panic!`/`unsafe`/`todo!`/`#[allow(dead_code)]` (seul `unwrap_or_else(
Instant::now)` = fallback safe sur `checked_sub`). Frontend N/A (0 web change).

## Delta tests

+6 Rust (3 convergence dispatch_loop #1/#2/#3 + 2 keepalive doc_sync + 1 capture engine
ajouté en résolution P2-1). 0 Vitest (cohérent, daemon-interne). Plan §4.3 (3 tests
nommés) couvert + 3 tests d'intégrité supplémentaires.

## Security & protocol (2 surfaces rouge-ligne → audit complet)

- **0 bump wire CONFIRMÉ** : aucun `*_VERSION`, aucun `DOMAIN_*`, clé `task:` = clé
  doc hors canonical (`dispatch_loop.rs:41`, `canonical.rs:74-80`) intouchée.
- **THREAT_MODEL §15.3 vérifiée exacte** : keepalive re-dial le MÊME coordinateur via
  `ticket.nodes` (EndpointAddr porte l'id/clé publique) ; re-résolution pkarr ne peut
  substituer un node id (paquet signé) ; canary Eclipse-by-DHT inchangé. Aucune
  frontière d'admission nouvelle.
- **API iroh-docs 0.98 relue ligne-par-ligne** : `start_sync(Vec<EndpointAddr>)`
  (api.rs:437), `leave()` = `state.remove`→`set_sync(false)`→`gossip.quit`
  (live.rs:444-462) ⇒ le test control est un VRAI rouge STRUCTUREL (B hors swarm, aucun
  path ne peut livrer k2), pas temporel. `import()` appelle `start_sync(nodes)`
  (api.rs:223) ⇒ H4 réfutée confirmée dans le code.

## Research & G8

Code suit fidèlement PLAN-ADAPT 6/6 : (1) subscribe = observabilité-seule, claim
poll-based (pas le faux-levier H4) ; (2) levier = maintenir le voisinage gossip via
re-`start_sync` + pkarr ; (3) fix côté worker (pas de faux-vert coordinateur H2) ;
(4) band-aids D1 rejetés (pas de poll-sub, HTTP push, N0 hot-path) ; (5) `get_sync_peers`
(store persistant) NON gaté → ne masque pas un drop réel ; (6) drain best-effort, pas de
backpressure P54.

## Findings & résolution

- **P2-1 — Wiring engine non couvert sémantiquement** → **FERMÉ IN-PHASE.** Test ajouté
  `runtime.rs::engine_captures_coordinator_peers_from_imported_ticket` : boote un
  coordinateur, mint un write ticket, enrôle un projet avec ce ticket, boote l'engine,
  et asserte que `task_docs` contient le projet ET que `task_doc_peers` a capturé ≥1
  EndpointAddr (la branche import → `ticket.nodes` capture, l'input du keepalive).
- **P2-2 — Backoff jamais reset** (écart vs `result_sync.rs:217`) → **FERMÉ IN-PHASE.**
  `doc_sync.rs` : `backoff = 500ms` après tout subscribe sain, donc un drop transitoire
  ultérieur repart du délai court, pas du plafond 30s.
- **P2-3 — `min_rejoin_interval` borne aussi le backstop périodique** → **FERMÉ
  IN-PHASE (doc).** Note ajoutée sur `KeepaliveConfig::check_interval` : garder
  `check_interval >= min_rejoin_interval` (cadence plus courte = harmless, ticks
  absorbés par le cooldown). Comportement correct ; aux défauts (15s/5s) et en config
  test (500ms/200ms) aucun tick n'est skippé (check > cooldown).
- **P3-1 — Déviation plan "rouge→vert par revert" → `leave()`-based** : documenté
  (limite hermétique : in-process le neighbor se forme trivialement, donc
  `convergence_incremental_*` est un GREEN guard ; le red→green vit sur
  `keepalive_rejoins_doc_after_neighbor_loss` via `leave()` ; preuve WAN = `b3` T2,
  Phase K). Non bloquant ; explicité au commit `## Contexte` + PATTERNS §P63.
- **P3-2 — Pas de test de saturation du buffer subscription** : drain best-effort
  (`Pulled::Event(_)`), buffer 64 ne sature pas en pratique ; hot-path couvert par P54.
  Acceptable (défensif).

## Residual risk

Faible. Levier de fix correct (worker-side, pas de faux-vert coordinateur), 0 bump wire
confirmé, API 0.98 conforme à la source. Risque résiduel principal NON réductible en
hermétique : la convergence WAN réelle (drop NAT/relay) n'est prouvable que par `b3`
cross-machine (T2, Phase K) — reconnu et routé, jamais revendiqué vert depuis l'in-process.

## Codex reconciliation

Codex GPT 5.5 (effort xhigh, `sprint77_phase_a_codex_review.md`, output brut) : **verdict
global CONFIRMED, 5/5 livrables CONFIRMED, 0 PARTIAL, 0 GAP.** Codex a relu la source
iroh-docs 0.98 installée (buffer 64 api.rs:459-471) ET relancé les tests ciblés lui-même
(`doc_sync::tests::`, `convergence_`, `engine_captures_coordinator_peers` — tous PASS).
Confirmations clés : (1) subscribe observabilité-seule, claim poll-based `runtime.rs:911-923` ;
(2) capture-before-consume `runtime.rs:360-368` + teardown avant node.shutdown `runtime.rs:760-770` ;
(3) test control = vrai rouge structurel ; (4) 0 bump wire (TASK_FORMAT_VERSION=1, DOMAIN_*
inchangés, iroh pin 0.98) ; (5) band-aids tous absents (pas de HTTP push, fork iroh, upgrade
1.0, relais N0 hot-path). **Aucun GAP → aucune correction requise** ; review promue PASS.
Note Codex : il n'a pas relancé les gates workspace (limite sandbox) — couverts côté driver
(fmt/clippy/nextest 1811/release/doctests verts).
