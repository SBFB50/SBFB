# Sprint 81 Phase A2 — Review (Workflow ultracode)

> Workflow `wf_2391ec3b-ce0` (2026-07-02) : 6 dimensions + 6 vérifications adversariales +
> synthèse, 13 agents Opus 4.8 1M, ~1.01M tokens, 195 tool uses. La synthèse a contre-vérifié
> le diff de première main (git diff HEAD) avant verdict.

## Verdict: PASS

> Émis PASS-PENDING par le Workflow review (0 P0/P1, 0 P2 structurel, 8 P3
> cosmétiques/carries) ; **promu PASS après réconciliation Codex** (rapport CLEAN
> au sens gate : 0 P0/P1/P2, 1 P3 — cf. §Codex reconciliation).

## Résumé par dimension

1. **Diff intégral** : PASS — discrimination NotFound/corruption correcte aux 2 sites
   (`runtime.rs:2500` / `:2595`), ordre des bras **load-bearing** (garde NotFound spécifique
   AVANT catch-all fail-fast — sans cette antéposition le self-heal du hotfix `6ca9702`
   régresserait) ; M8 row intacte garantie structurellement (le `return Err` précède tout
   `set_storage_namespace`). Seul P3 : asymétrie cosmétique des messages remède storage/feed.
2. **Tests sémantique** : PASS — aucun test-mensonge ; le discriminateur est prouvé
   non-trivialement par la PAIRE recreate+fail-fast (le marqueur « refusing to silently
   recreate » n'existe que dans le bras fail-fast). P3 : propagation `start()` → exit≠0
   prouvée par inspection/type, pas par un test E2E start() sur M8 corrompue ; branche
   défensive `Ok(None)` intestable en 0.98 ; le « loud » (warn ns=) non asserté (pas de
   capture tracing en test).
3. **Scope + grounding** : PASS — chaque hunk mappe une ligne du préflight §3 ; 4 interdits
   tenus (outer None first-boot, caller project_doc, mécanisme recreate byte-identique sauf
   warn, deps) ; matcher `.contains` aligné sur l'évidence upstream (« open failed: Replica
   not found ») ; 4 carries §10 hors diff ; 0 scope creep (2 fichiers seulement).
4. **Sécurité deep** : PASS — boot duress neutre ; 0 fuite de secret (ns_id public, pas une
   clé) ; matcher non injectable à distance (le contenu d'erreur d'open_doc n'est pas
   attacker-controlled par un pair). P3 : DoS local délibéré (crash-loop systemd, NET-POSITIF
   car le crash survient AVANT le bind HTTP — supprime l'ancienne surface HTTP dégradée) ;
   chemin FS local possible dans les logs (journalctl local uniquement) ; fragilité substring
   tracée carry Phase B.
5. **Frontières docs-contrat §6.12** : PASS — 0 frontière impactée (fn privées de boot,
   signature `open_doc` inchangée, réponses API inchangées) ; doc-comment corrigé
   factuellement exact et explicitement daté 0.98 (important pour le bump Phase B). P3
   optionnel : PATTERNS.md non amendé (l'érasure RPC du variant typé ne vit que dans les
   commentaires inline + préflight §7/§10.2 — envisager une entrée courte).
6. **Suites §7.4 + honnêteté** : PASS — delta +4 exact (0 test retiré) ; compteurs
   arithmétiquement cohérents (Win 2022→2026 0-skip ; Docker 2026→2030 = 2024 passed +
   6 env-blocked) ; les 6 fails Docker tous dans `nexus-test-harness` (non touché), classe
   daemon-spawn documentée ; 0 zombie (aucun test ne vérifiait l'ancien warn-only).

## P3 consolidés (aucun bloquant, aucun P2 structurel)

- **D2-1** : call-sites (`runtime.rs:732`/`:771`) non exercés via `DaemonRuntime::start()`
  E2E — couvert au niveau fn privées, propagation prouvée par type. Documenté body.
- **SEC-P3-1** : régression de disponibilité DÉLIBÉRÉE (corruption 1 app → abort boot
  complet) — résidu maîtrisé pré-launch, carries §10.1 (THREAT_MODEL → Phase G) + §10.4
  (granularité per-app).
- **SEC-P3-3/D6** : fragilité substring + branche `Ok(None)` défensive intestable —
  re-calibrage IMPÉRATIF au bump 1.0.1 (carry §10.2, Phase B).
- **D5-patterns** : PATTERNS.md non amendé (optionnel, couverture inline dense).
- **D2-3** : les tests recreate n'assertent pas le warn « loud » (pas de subscriber tracing).
- **SEC-P3-2** : `{e}` interpolé peut embarquer un chemin FS local (journalctl local
  seulement, jamais exposé à un pair) — durcissement optionnel post-launch.
- **D1-nit-1** : message storage ne nomme pas explicitement la clé M8 (app_name EST la clé).
- **Missed adv (trous de couverture au bump)** : fail_fast simule une erreur transport (proxy
  de corruption) ; l'hypothèse « octets aléatoires → NotFound » à re-vérifier en 1.0.x ;
  la ré-écriture du ticket M8 en branche recreate non assertée.

## Notes pour le commit body

- Delta tests : **+4 exact** — nextest Win **2022→2026 0-skip** ; Docker canonique
  **2026→2030** (= 2024 passed + 6 env-blocked daemon-spawn, crate non touché, verts Win
  natif + CI Linux). web 411/411 + coverage seuils (re-run solo, flakys de charge, web non
  touché) ; operator 201/201 + gates 6/6 + E2E 10/10 ; fmt/clippy/doctests/release verts
  les 2 plateformes.
- Fichiers : 2 (runtime.rs control-flow/logs/tests + docs.rs doc-comment). 0-bump, 0 dep,
  0 migration. 3e call-site `project_doc` déjà fail-fast (hors scope).
- Couverture honnête : propagation start() par inspection/type (cf. P3 D2-1).
- Carries : §10.1 THREAT_MODEL → Phase G ; §10.2 re-calibrage matcher → Phase B ; §10.4
  granularité per-app.

## Codex reconciliation

Rapport Codex GPT 5.5 lu (`sprint81_phase_a2_codex_review.md`, output brut `codex exec -o`,
non réécrit) — **0 P0/P1/P2, 1 P3** ; critère d'arrêt boucle atteint au 1er round (CLEAN ou
P2/P3 documentés).

- **Livrables** : 1 helpers internes OK (`runtime.rs:2498`/`:2593`, « pas de bras générique
  avant le guard ») ; 2 call-sites OK (`:708`/`:750`, propagation `main.rs:203`, systemd
  `Restart=on-failure`+5s vérifiés) ; 3 commentaires OK (`docs.rs:151`, `runtime.rs:2487`/
  `:2590`) ; 4 tests PARTIEL (P3 ci-dessous).
- **GAP P3 (unique)** : les tests `recreates_loud_*` n'assertent pas le `warn!`/champ `ns`
  (le « loud ») — convergent avec le P3 D2-3 de la review Workflow. **Documenté** (pas de
  fix in-phase : capturer tracing en test = harness subscriber dédié, disproportionné pour
  un log ; le comportement est dans le code, vérifié par lecture croisée Claude + Codex).
- **Invariants contre-vérifiés par Codex** : 0-bump (pins iroh intacts `Cargo.toml:38`,
  lock 0.98.0, schéma M8 inchangé `db.rs:149`, upsert local `db.rs:1023`) ; 3e call-site
  `project_doc` (`runtime.rs:650`) déjà fail-fast via `?`, hors scope confirmé.
- Aucune correction de code requise → pas de re-boucle suites/review/Codex.
