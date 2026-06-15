# Sprint 75 Phase A — Review (FIX-A re-mint-on-replay)

## Verdict: PASS

(Promu depuis PASS-PENDING après réconciliation Codex — cf. `## Codex reconciliation`.)

## Process

Review adversariale par workflow `w0qyfivn2` (agent `nexus-phase-review-deep` non
enregistré → fallback workflow 5 dimensions anti-anchoring + vérification
adversariale par finding, 10 agents, ~0.75M tok). Dimensions : correctness,
security, tests, wire-scope, patterns-docs. Toutes les suites lourdes vertes
(fail-fast dual-platform, cf. `verification`).

## Verdict du workflow : CONCERN → résolu

Le workflow a jugé la **logique de PRODUCTION FIX-A correcte sur les 5 dimensions**
(security PASS, 0 P0/P1/P2) : outbox stocke le payload non-wrappé ; les 4 sites de
replay re-mintent l'adresse + re-stampent un PoW frais ; `MAX_PROOF_AGE_SECS`
inchangé ; 0 diff nexus-core-rs (avant T2) → 0 break wire ; garde anti-hijack
`ann.node_id == node.node_id()` présent et correct ; outbox OWN-only structurel.

Il a soulevé **2 P1 (test-only)** — TOUS RÉSOLUS :

### P1-1 (CORR-1/WS-1/PD-1) — test outbox non migré `http.rs:7548` → suite ROUGE
- **Réel** : la review a tourné sur le diff AVANT ma correction. `publish_and_gossip_
  use_per_app_project_id` décodait l'entrée outbox comme `PowEnvelope`
  (`ProofLenOverrun` → panic) car elle est maintenant non-wrappée.
- **FERMÉ** : migré comme son jumeau `http.rs:2897` (bind `payload`, `is_project_
  announcement` + `from_gossip_bytes(&payload)`). Test PASS confirmé.

### P1-2 (T1) — test hijack-guard FAUX-VERT (couverture nulle du garde)
- **Réel et important** : `replay_does_not_remint_a_third_party_address` mintait le
  ticket « étranger » depuis NOTRE adresse, donc supprimer le garde `node_id`
  (`runtime.rs:1849`) produirait le même ticket déterministe → le test passerait
  quand même. Le garde anti-recentralisation avait 0 couverture effective.
- **FERMÉ** : réécrit avec 2 nœuds — ticket étranger depuis l'adresse d'`other`,
  sous `other.node_id()`, **blob détenu aussi par nous** (donc un re-mint non-gardé
  RÉUSSIRAIT et changerait le ticket). Assert ticket inchangé → SENSIBLE (échoue si
  le garde est retiré). + contrôle positif `replay_remints_own_ticket_to_current_
  address` (ticket stale depuis l'adresse d'`other`, own → re-mint vers NOTRE
  adresse, `assert_ne!` ticket changé + hash préservé). Les 2 PASS.

## P2 — traités (4/6) + carries (2)

- **T2** (invariant SESSION_WINDOW < MAX_PROOF_AGE non pinné + doc-comment FAUX
  « comfortably exceed ») → **FERMÉ** : commentaire corrigé (« stay BELOW ») +
  **const-assert compile-time** `const _: () = assert!(SESSION_WINDOW.as_secs() <
  crate::pow::MAX_PROOF_AGE_SECS)` (pow_gossip.rs). Un futur bump réintroduisant le
  bug ne compile plus.
- **T3** (mint-failure fallback non testé) → **FERMÉ** : `replay_keeps_stale_ticket_
  when_blob_is_gone` (blob absent → garde le ticket stale + PoW frais ; `expect`
  échoue si remint renvoyait None).
- **T4** (keep_online testé seulement sur shape legacy wrappée) → **FERMÉ** :
  `keep_online_gate_handles_unwrapped_payload` (suppress/replay/fast-path sur la
  shape non-wrappée du hot path S75).
- **T5** (pas d'e2e DB sur la NOUVELLE shape non-wrappée) → **FERMÉ** : `browse_boot_
  restore_from_unwrapped_outbox_e2e` (insert_outbox payload non-wrappé → load →
  restore → carte Reachable + search-by-name).
- **T6** (handler GossipCmd::Outbox broadcast non testé en direct) → **CARRY P2**
  (couvert indirectement par l'assertion de shape http.rs ; un test direct exige 2
  nœuds connectés). Route audit S76.
- **WS-3/PD-5** (double parse normalize + `my_endpoint_addr()` par-entrée par-passe)
  → **CARRY P2** (efficience pure, bornée par la petite cardinalité outbox + le
  fast-path `disabled.is_empty()`). Optim possible : hoister `my_endpoint_addr()`
  once-per-pass. Route audit S76.

## P3 — traités

- Doc comments périmés (CORR-2/PD-2/PD-4) « PoW-wrapped envelope » sur
  `restore_browse_from_outbox` + header `keep_online_allows_rebroadcast` + inline
  boot-restore → **reformulés** (entrées = payloads non-wrappés, legacy normalisé).
- SEC-1/SEC-2/T7 : notes hors-scope ou propriétés pré-existantes (anti-spam stamp ≠
  signature payload ; node_id self-asserted mais outbox OWN-only) — pas d'action.

## Dimensions

| Dimension | Verdict | Note |
|---|---|---|
| correctness | PASS | no-ticket/mint-failure/persist-always/4-sites/round-trip OK |
| security | PASS | hijack guard correct + désormais COUVERT ; MAX_PROOF_AGE inchangé ; OWN-only structurel |
| tests | PASS (post-fix) | 2 faux-verts/manquants fermés ; sensibilité prouvée |
| wire-scope | PASS | 0 bump ; stockage LOCAL ; byte-shape identique ; pas de zombie |
| patterns-docs | PASS (post-fix) | doc comments alignés ; 1 helper unique ; pas de dead code |

## Suites

Fail-fast dual-platform : voir `verification` (Windows natif + Docker canonique avant
push). Tests ciblés post-fix : **13/13 PASS** (T1 sensibilisés + T3/T4/T5 + touchés).

## Codex reconciliation

Codex GPT-5.5 (`sprint75_phase_a_codex_review.md`, sortie brute `codex exec -o`) :
**7/8 livrables CONFIRMÉ, 1 GAP mineur de NOMMAGE (non-défaut)**.

- Livrables 1-7 CONFIRMÉ avec evidence fichier:ligne : outbox non-wrappée
  (`deploy.rs:673/676-680/692-695`), helper re-mint + garde OWN-only
  (`runtime.rs:1847-1864`), 4 sites couplés (browse_request/NeighborUp/republish +
  restore + keep_online via normalize, aucun rebroadcast verbatim), fenêtre PoW
  inchangée (`MAX_PROOF_AGE_SECS=1800`, `verify_at` intact ; la seule modif core =
  l'assert T2, pas un affaiblissement), normalize transition, mint partagé, 0 bump
  wire (4 `*_VERSION` restent 1).
- **GAP #8 = artefact de nommage du prompt** : le prompt Codex listait l'ancien nom
  de test `replay_refreshes_own_ticket_preserving_hash` ; il a été RENOMMÉ en
  `replay_remints_own_ticket_to_current_address` lors du fix P1-2/T1 (sensibilisation
  2-nœuds). Codex confirme lui-même que la couverture est présente sous le nouveau
  nom, et plus forte. Aucune correction code. Documenté `## Codex verification` du
  commit body.
- Nuance Codex livrable 1 (`let _ =` sur `gossip_cmd_tx.send`) = best-effort
  intentionnel (pattern pré-existant ; channel fermé = daemon en arrêt). Non-défaut.

Suites relancées post-fix : fail-fast Rust complet VERT (fmt/clippy --workspace
--all-targets/nextest --workspace **1682 passed 0 fail**/doctests/release) + Codex
tests ciblés passés. **Verdict final : PASS.**
