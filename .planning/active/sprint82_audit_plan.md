# Sprint 82 — Audit plan (à jouer en Phase 0 de S82)

> Écrit à la clôture de S81 (Phase K). Une session fraîche S82 joue ce plan
> AVANT toute Phase A, produit `.planning/active/sprint81_audit_findings.md`
> avec verdict PASS / CONDITIONAL PASS / FAIL, et écrit les commits
> `fix(sprint81)` pour les P0/P1. Canon des tracks :
> `prompts/agent/audit-gate-checks.md` (11 tracks A..K depuis `a6b4ca4`).

## 0. Mode d'emploi (session fraîche)

Lire ce plan + `prompts/agent/audit-gate-checks.md` EN ENTIER d'abord.
NE PAS lire `docs/{rust,shell}/PATTERNS.md` avant la Track C — former une
opinion indépendante depuis le diff, puis confronter aux patterns (règle
anti-anchoring du canon audit).

## 1. Périmètre

- **Diff audité** : `61412bb..<tip S81>` (24+ commits : kickoff/activation +
  vérification ultracode `b1f174e` + 15 phases 0/A/A2/A3/A4/B/C/D/E/E2/E3/
  F/G/H/I/J/K + 4 chores acceptance `8872596`/`a085853`/`bd5d680` + palier J
  + 2 chores process/recherche `e7ff73c` [bascule Codex 5.5→5.6 Sol
  MI-SPRINT — la moitié des phases a été gatée 5.5, l'autre 5.6] +
  `9c52cb7` Psyche/Nous).
- **Particularité 1** : sprint de MAINTENANCE d'infrastructure (iroh
  0.98→=1.0.1) au DONE BI-AXE (C1) — l'audit vérifie les DEUX axes :
  transport (baseline→flip live) ET sharding (ex-S78 absorbé I/J).
- **Particularité 2** : migration on-disk ONE-WAY jouée sur la flotte live
  (3 nœuds) — vérifier qu'aucun claim de réversibilité ne subsiste
  (rollback = DEUX gestes, restore tar + redeploy 0.98).
- **Particularité 3** : Phase J a subi 3 blocages exécuteur Codex (parade
  prompt INLINE documentée) — vérifier l'authenticité brute des artefacts
  Codex de J et K (Check 7 lightcheck déjà passé, l'audit re-juge le fond).
- **Baseline suites à l'entrée d'audit** : Rust nextest Win 2084 / Docker
  2088 à la fin de J (le body K porte le delta final) ; Vitest web 412 ;
  Vitest operator 201 ; E2E Playwright operator 10 ; T2 agrégat bi-axe
  `sprint81_t2_acceptance.json` committé.

## 2. Les 11 tracks (canon `prompts/agent/audit-gate-checks.md`)

| Track | Focus S81 spécifique |
|---|---|
| A suites | 3 blocs fail-fast + dual-platform Docker `sbfb-ci` rust:1.94 ; `multi_daemon` env-bloqué Docker-on-Windows (jamais compté) ; le pipeline K doit avoir joué `cargo deny check` COMPLET (4 catégories) |
| B security | THREAT_MODEL v17 (sweep S78 ×15 — grep `S78` : 0 ref vivante non requalifiée hors historiques v12-v14) ; attestation loaded-stage = self-claim (pas de sur-claim byzantin) ; sanitize_diagnostic note N/A GUARDRAILS motivée ; §15.4/15.5 zero-n0 + flip |
| C patterns | §P73 (si posé) vs code ; §P70/P71/P72 non régressés ; pattern payload applicatif in-frame (SHARD_STEP/ATTEST) cohérent aux 2 usages |
| D scope | iroh STRICTEMENT SEUL tenu (0 feature produit hors iroh/sharding re-cert) ; bump toolchain 1.95 ABSENT ; materializer commit séparé AVANT bump (bisectabilité A→B) |
| E tests delta | 2014→2084→<final K> Win : delta par phase = somme des bodies, 0 baisse silencieuse ; zombies legacy-decode actés au body C ; les réparations test-rot ne comptent pas net-new |
| F review files | 15 phases × 3 artefacts (preflight/review/codex) + review = UN SEUL header `## Verdict` ; Phase J : boucle Codex 2 P1 (fingerprint corrigé + binding CARRY → vérifier K l'a fermé) |
| G carry-overs | §3 ci-dessous — chaque item CLOSED ou re-routé avec rationale |
| H HARDENING | trigger iroh FIRED consigné (G) + last_validated bumpés (LOOPBACK 2026-07-11) ; drift vs état réel |
| I meta-process | bodies 9 sections ; Codex bruts non réécrits ; G8 verdicts (I/K = PLAN-ADAPT ; **J = DESIGN-CONFLICT résolu arbitrage PO Option B**, tracé `sprint81_phase_j_preflight.md §Arbitrage PO` + body `43623a5` — corrigé audit S81-I-1) ; bascule Codex mi-sprint tracée `e7ff73c` |
| J testabilité (standing) | T1 6 sous-tests mappés BLOQUANT-vert (mapping committé K) + T2 agrégat bi-axe vocabulaire fermé ; 0 prose DIFFERE-* ; le job `integration-nightly.yml` existe ET a tourné ≥1 fois (vérifier un run réel, pas juste le fichier) |
| K docs-contract (standing) | clôture K : LOOPBACK §3 +6 lignes shard-session ; SHARD_PROTOCOL_SPEC §5.1/5.2/§6 ; lot doc-stale docs/sharding requalifié (llms.txt, WIRING_SPEC ×4 dont réf pendante :147) ; `spec_consts_exist` étendu types J+K ; frontière S82 neuve non indexée = P1 |

Track G1 : `sprint81_design_review.md` existe dans `active/` (vérifier avant
archive). Verdict tree : PASS (0 P0/P1, ≥1 P2+ documenté) / CONDITIONAL
PASS / FAIL (P0 ou P1) ; 0 P0/P1 ET 0 P2+ = CONCERN (rigor signal G4).

## 3. Carries à escalader (inventaire nommé, zombies filtrés)

### ESCALADE BLOQUANTE Phase 0 (compteur de reports atteint)
- **S75 re-drive-on-ingest boot-SEED-driver — OVERDUE 3/3** (routé par
  `e05338f` + `50f05c1` + `8872596` ; fenêtre morte 1er boot OBSERVÉE live
  S75-G et S81-E2). Règle §6.2.1 : fermer dans S82 ou re-conception —
  plus jamais de report sec. Évidence NEUVE S81-K (même famille
  cold-boot catch-up, côté WORKER cette fois) : au run 2 du palier
  quorum, un worker démarré 3s avant le submit n'a JAMAIS reçu l'entrée
  `task:` incrémentale en 30s (gossip neighborhood pas formé au moment
  du broadcast) ; la même tâche a convergé une fois le worker stable
  (+2m08 : exécution + quorum + result visible). Provenance : timings et
  attributions per-worker operator-corroborated depuis des logs rig NON
  committés (le raw run 2 a été écrasé par le run 3) — contrat de
  provenance dans la note de l'agrégat T2. Un worker frais devrait
  rattraper les tâches pending à l'ingest/au boot — instruire ENSEMBLE
  avec l'escalade re-drive-on-ingest.

### Fermés S81 — NE PAS re-router (vérifier le statut LIVE)
- Carry P1 sharding S77 RIG-ABSENT — **CLOSED** (`43623a5` +
  `sprint81_t2_j_shard_inference.json` PASS). NE PAS re-router.
- Binding loaded-stage↔manifeste (carry P1 Phase J) — fermé Phase K
  (attestation fail-closed aux stage-links ; vérifier tests + THREAT v17).
- WAN task-delivery S77 (C10) — fermé A3/A4 (boot sync-set).
- Hot-join gossip du curateur souscrit — fermé E3 (+ palier live PASS).
- P2-SIBLING-SYNC-SET — fermé Phase C (storage+feed sync-set au boot).
- Dépendance relais n0 des 2 tests two_nodes core — fermée K (strip-relay
  direct-only) ; vérifier qu'ils ne re-flakent pas en CI 3-OS.

### Supply-chain (Track B/H)
- **P2-AUDIT-2-RESIDUEL** (lock non convergent : ed25519-dalek 2.2.0 +
  3.0.0-rc.0 interne iroh ; `deny.toml` multiple-versions = warn).
  **FAIT NEUF à instruire : ed25519-dalek 3.0.0 STABLE publiée 2026-07-06**
  (rc.0 non yanké) — vérifier si iroh 1.0.2+/futur relève le pin interne ;
  si oui le déblocage devient possible (flip warn→deny).
- **HICKORY-024-RUSTSEC** (6 ignore-with-reason racines hickory-0.24/
  quick-xml — re-vérifier les advisories à la date d'audit).
- **RUSTSEC-2026-0185 quinn-proto <0.11.15 (HIGH, DoS OOM)** — NON-trackée
  dans le repo (0 ignore, deny vert LÉGITIME graph-aware : tirée uniquement
  par reqwest→quinn http3 optionnelle hors graphe résolu ; iroh 1.0 = fork
  `noq`). Résiduel borné VÉRIFIÉ au préflight K ; à re-vérifier (un futur
  `cargo update` ou feature http3 la rendrait réelle). Note d'honnêteté K :
  `cargo audit` (lock-based) N'EST PAS installé — le claim G « cargo-deny /
  cargo-audit verts » était inexact sur la moitié cargo-audit.
- **`deny.toml` `yanked = "deny"` = mode de casse CI MÉCANIQUE** : un yank
  d'ed25519-dalek 3.0.0-rc.0 (pin exact iroh =1.0.1) rendrait advisories
  ROUGE sans aucun commit SBFB. Parade documentée : ignore temporaire
  motivé + escalade bump iroh — décision à jouer LE JOUR OÙ ça casse,
  pas préventivement.
- G-D5-1 : `VALIDATED_BLUEPRINT` cite « iroh 0.97 » (stale mineur).

### Sharding (post-bi-axe)
- RunProofs PER-WORKER + binding N0-N3 in-vivo (canal de retour
  control-plane feed raw-op / docs, JAMAIS un ALPN neuf) — R-J-6.
- Arbitrage de litige N3-reveal in-vivo + transport du sketch complet
  hors slot 32B ; SI-5 padding ; SI-7/SI-11 re-calibration sur rig
  (baseline = benchmark J).
- J-D5-1 : assertion machine `conn_type == direct` au readiness-barrier
  (K = label honnête dans l'agrégat, l'assertion reste due).
- Schémas JSON des corps de requête shard-session
  (`ShardGroupMintRequest`/`MountSessionRequest`/`ShardGenerateRequest`)
  — les réponses sont schématisées, les requêtes non (dette doc-contrat).
- F2 KV-cache cross-step (stateless recompute = coût quadratique assumé) ;
  p95=moyenne + ttft_ms résolution 1s dans les métriques harness ;
  cold/warm ttft non distingués ; fallback windows partiels ;
  churn 16-hex dans les logs live (R-J-7 verifié sur artefacts committés).
- **J1b-3** (P3, review J + K-R-4) : `participants` du chemin decode
  (`drive_decode_loop`) non borné à `RUN_PROOF_MAX_PARTICIPANTS` — edge
  churn au plan très large ; routé K « Phase K/audit gate » par le body
  `43623a5` mais NON livré en K (robustesse, pas sécurité).
- **D3-2** (P3, review J + K-R-4) : `reply.piece` (bytes
  attaquant-contrôlés) concaténé dans `result_text` sans normalisation
  (le cap `MAX_RESULT_TEXT_BYTES` borne la taille, pas le charset) —
  robustesse extraction harness.
- **SI-12 TOCTOU load↔hash** (P2, Codex K) : le `model_digest` attesté
  est le blake3 du FICHIER au chemin GGUF, hashé APRÈS le load (mmap par
  défaut) ; un remplacement atomique entre load et hash servirait l'ancien
  inode mmapé sous le nouveau digest. Surface hôte-local/trusted (L), le
  binding n'est pas atomique. Durcissement = hash de la région mmapée OU
  hash-avant-load + re-vérif (THREAT v17 §16 SI-12).

### Transport / réseau
- Constats flip H (`bd5d680`) : seeder VPS `catalog_len=0` one-shot
  (question design PO S75 standing) ; stores local-worker encore redb2
  (migrent à leur prochain boot — vérifier qu'ils ont bien migré).
- Test-rot multi_daemon : baseline A3 4/10 verts (5 test-rot + 1 signal
  produit gossip discovery) — le job `integration-nightly.yml` (K) les
  expose désormais ; réparer ou requalifier CHAQUE test rouge.
- Topologie A-vs-B zero-n0 : re-décision PO AVANT 25/08 (B déployée live
  `a085853` ; re-décision explicitement OUVERTE `50f05c1` §15.4).
- Gates calendaires C8 : 25/08 (Phase F pas PASS → plan B flotte) déjà
  moot (F PASS) ; **15/09** (garde-fou EOL 30/09) — vérifier l'état n0.

### Tous P2/P3 des 15 phase-reviews S81 (règle kickoff : chaque review route)
- A : author→project_id (fold), MAX_FEED_ENTRIES, BinaryHeap topo-sort.
- A2 : import_ticket au recreate, per-app fail-fast.
- A3/A4 : P2-PROJECT-DOC-SELECTOR (`list_docs().first()`), keepalive
  NeighborDown.
- C : duress 45s fenêtre, WS-3/PD-5 hoisting (héritage S75).
- D : BlobTicket call-sites multiples, outbox stale, troncature-16.
- E : doc-stale age_witness, MANQUE-3. E2 : D2-2. E3 : `from_subscribed`
  à chaud, asymétrie unsubscribe (pas d'API leave iroh-gossip).
- I : TOCTOU 202/202, resume mid-pipeline mono-étage, mappings HTTP fins,
  D4-1 RunProof projection.
- K : voir la review K elle-même (`sprint81_phase_k_review.md`).

### Standing (hors-S81, ne pas perdre)
- 1 carry P1 in-vivo : app-authoring S79 `Not evidenced` (standing).
  b3_p2_quorum est PASS au wrap-up K (2026-07-11, 1er de l'histoire,
  C10, 6s end-to-end) — carry ÉTEINT ; vérifier seulement que l'agrégat
  T2 committé le reflète (top-level PASS bi-axe).
- Viewer fondation + Aperçu scellé/Proof Card — candidats S82 (arbitrage
  C9 BLOQUANT : slot S82 = workflow-engine vs Viewer vs dette docs-contract,
  **à RE-CONFIRMER PO au wrap-up S81 — pas encore tranché**).
- Dette docs-contrat : audit S79 = 8 P2/11 P3, audit S80 = 4 P2/10 P3 →
  sprint dette nommé (jamais bundlé). *(Correction S82 Phase E, 2026-07-14 :
  la ligne d'origine « 8 P2/11 P3 docs-contract S80 » était un mislabel — 8/11 est
  le tally de l'audit S79, joué en Phase 0 de S80. Re-audit par item :
  `sprint82_phase_e_ledger_reconciliation.md`.)*
- Arc front parqué `wip/factory-front-arc-post-s82` (review + Codex
  groupés DUS à la reprise ; rebase conflit attendu provider_router.rs).
- S79-P2-1 ancres task_response.rs → sprint dette.
- Benchmarks standards LLM/sharding (llama-bench + perplexity-parity +
  TTFT/TPOT/ITL versionnés) — arbitrage PO Phase L S81 vs S82 (memory
  `po_benchmarks_standards_llm_sharding`) ; si non joué en L, S82 candidat.
- Externes : P2-A-1 rand (exemption), T-NN+2 iframe Rust-wasm (§P34),
  P3-OS-1, LT-2 Radicle ARMÉ (flip = décision PO), R-iroh-audit P0
  INCHANGÉ (upgrade ≠ Gate 1/3, pilote reste fermé).

## 4. Out-of-scope de l'audit

- La review groupée + Codex groupé de l'arc front parqué (appartient à la
  reprise post-S82, memory `rapid_front_add_session`).
- Le flip publiée/privée Radicle (LT-2) — décision PO hors-sprint.
- Toute re-litigation des Day-0 S81 (D1..D8) et arbitrages C1..C10 —
  l'audit vérifie leur RESPECT, pas leur bien-fondé.

## 5. Format du livrable

`sprint81_audit_findings.md` : verdict + findings P0..P3 nommés
(`S81-<track>-<n>`), chaque P0/P1 avec commit fix ; carries §3 chacun
CLOSED / re-routé-avec-rationale / escaladé. Baseline tests re-mesurée
à l'entrée (Win + Docker + web + operator + E2E).

## 6. Note

L'audit gate S81 est la Phase 0 de S82 — il BLOQUE toute Phase A de S82.
*(Corrigé S82 Phase E, 2026-07-14, PO-9 : la note d'origine présentait le staging
workflow-engine comme kickoff bloquant de S82 avec la supersede D6
pendante. Réel : S82 = sprint dette docs-contrat + refactorisation
(kickoff = `sprint82_kickoff.md` en `active/`) ; workflow-engine + Viewer
sont DÉCALÉS vers de futurs slots tracés ; la supersede D6 est RATIFIÉE ;
le staging `.planning/research/sprint82_workflow_engine/` est marqué
SUPERSEDED.)*
