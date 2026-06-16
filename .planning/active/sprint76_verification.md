# Sprint 76 — Verification (self-report fail-fast, living)

> Doc VIVANT mis a jour a la cloture de chaque phase ; finalise en Phase G.
> Valeur limitee par construction (self-report de l'agent livreur) — la
> verification independante est l'audit gate S77 Phase 0. NE consigne PAS
> d'assertion non executee : ce qui est differe est marque DIFFERE.

## §1 HEAD progression

- **HEAD entree S76** : handoff kickoff `3faee6e` (docs(sprint76), POUSSE
  origin/master apres gate dual-platform vert).
- **HEAD courant** : `1cc28e7` (Phase C). NON push (ahead 4 vs origin :
  Phase A `ce43894` + chore agents `d6dea45` + Phase B `6904cdd` + Phase C
  `1cc28e7`). Push differe post-phases C-G + recovery Docker dual-platform.

## §2 §7.4 par phase (fail-fast)

| Phase | Commit | Rust Windows nextest | Rust Docker (canonique) | Frontend | fmt/clippy/doctests/release |
|---|---|---|---|---|---|
| A | `ce43894` | 1763 -> 1767 (+4) 0-skip | subset 582 (crates touches) | Vitest 379 -> 386 (+7) | 0 / 0 / 0 / 0 |
| B | `6904cdd` | 1767 -> 1775 (+8) 0-skip | 3 crates 675/675 (+3 cfg(unix)) | Vitest 386 -> 396 (+10) | 0 / 0 / 0 / 0 |
| C | `1cc28e7` | 1775 -> **1785** (+10) 0-skip | **1789/1789** 0-skip (code fonctionnel, round 1) | 0 (aucun changement web) | 0 / 0 / 0 / 0 |

Delta tests Phase C = **+10 Rust** (6 task.rs + 1 dispatcher.rs + 3
engine/runtime.rs). Detail des suites Phase C :
- Windows : `cargo fmt --all --check` 0 ; `cargo clippy --workspace
  --all-targets --locked -- -D warnings` 0 ; `cargo nextest run --workspace
  --locked` 1785/1785 0-skip ; `cargo test --workspace --locked --doc` 0 ;
  `cargo build -p nexus-shell-daemon --release` 0. Re-confirme APRES les
  fixes doc PARTIEL (run sequentiel Windows-seul `bg2vhto0q`).
- Docker Linux sbfb-ci (canonique) : fmt + clippy + nextest 1789/1789
  0-skip + doctests = `DOCKER_ALL_GREEN` sur le code FONCTIONNEL (run
  `bq6jj6fkz`, target dir isole en volume nomme). Le re-run Docker APRES
  les fixes doc-only (commentaires verification.rs + 2 maps `.md` + script
  `.sh`) est **env-bloque** (cf. §4) — delta doc-only platform-agnostique,
  Windows-confirme.
- Gate anti-regression existant `e2e_network_execute_gate_real_http_no_
  frontier_mock` (nexus-shell-daemon) : hors-diff, vert.

## §3 Acceptance LIVE B-3 (palier 1, D2) — DIFFERE materiel operateur

- **Livre** : harness scripte `scripts/acceptance/b3_live_pc_vps.sh`
  (endpoints reels verifies contre le code : `POST /api/v1/invite/create`
  scope:worker -> token sous cle `wire` ; `nexus-worker join` +
  `nexus-worker start --headless` ; `POST /api/v1/tasks/submit` ;
  `GET /api/v1/tasks/{id}/result` -> `result_text`).
- **Critere falsifiable (D2 adjust)** : `DELAY` = delai submit (VPS) ->
  result_text visible (VPS), END-TO-END = borne SUP de la convergence WAN
  `result:` (claim + inference GPU + replication). Pour le petit prompt
  deterministe du harness, claim+inference est de quelques secondes, donc
  un delai proche/au-dela de 30s implique la convergence WAN. **> 30s =
  BLOCK a DIAGNOSTIQUER** (root-cause : inference ou replication ? ref S75
  `SeedAnnounced peer_count:0 ~10 min`), **PAS un timeout a rallonger**.
- **Statut** : run WAN reel PC RTX 5080 <-> VPS **DIFFERE** — l'environnement
  de dev de cette session n'a pas la paire PC+VPS+WAN. A executer sur le
  materiel operateur ; la trace (submit -> claim PC GPU -> result_text WAN
  rendu + delai mesure) sera consignee ici a ce moment. PAS d'assertion
  LIVE non executee (precedent S74 : re-run dual differe a recovery). Le
  chemin compute (dispatch/pompe/result-sync/validator/sign-verify) est
  INCHANGE et couvert in-process par le gate anti-regression (§2).

## §4 Note env — Docker Desktop Linux engine wedge (S76-C)

Le re-run Docker du delta doc-only est env-bloque : lancer Windows nextest
et un build Docker LOURDS en CONCURRENCE a sature la RAM hote ->
(1) le linker MSVC a crashe en boucle `STATUS_STACK_BUFFER_OVERRUN`
(0xc0000409), (2) le conteneur Docker a ete tue (`unexpected EOF`), puis le
moteur Docker Desktop Linux s'est wedge (`500 Internal Server Error` sur
`/_ping`, le WSL-wedge documente en memoire S74). Le code FONCTIONNEL avait
deja passe Docker canonique 1789/1789 (round 1, avant les doc-fixes) ; le
delta doc-only (commentaires + `.md` + `.sh`) est platform-agnostique et
Windows-confirme. **Lecon : relancer les suites SEQUENTIELLEMENT (Windows
seul, puis Docker seul), jamais simultanement.** Le re-run Docker sur le
doc-delta est differe a la recovery du moteur (`wsl --shutdown` + restart
Docker Desktop) AVANT push.

## §5 Carries / Phase G TODO

- THREAT_MODEL : ajouter la ligne « compute-cohort homogeneity = advisory
  routing, pas une frontiere de confiance » (la vraie defense reste le
  quorum exact-match qui rejette un resultat divergent comme outlier).
  Route par le preflight Phase C (P3).
- Consigner la trace de l'acceptance LIVE B-3 une fois executee (§3).
- Re-run Docker dual-platform sur l'arbre final apres recovery du moteur
  (§4), AVANT push.
- model_digest : durcissement en hash du fichier GGUF = S77 (chemin
  `llm_llama_cpp` C-API, reserve etage-2 de D3) — doc-note honnete livre
  Phase C (task.rs + verification.rs + 2 maps).
