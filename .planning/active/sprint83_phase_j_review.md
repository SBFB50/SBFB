# Sprint 83 Phase J — Review round r4 (post-Codex r2/r3, promotion)

Date : 2026-07-23.

## Verdict: PASS

## Codex reconciliation

3 rounds Codex GPT-5.6 Sol effort max (rounds/…_codex_review_r1..r3.md,
canonique = copie byte-identique de r3). Arrêt au critère canonique
« CLEAN ou P2/P3 documentés » — sévérité jugée par la review, pas déléguée.

- r1 GAP (0 P0, 5 P1) → TOUS traités : cancel n'émet plus jamais
  (drain supprimé, pending conservé, side-output == 0 asserté) ; join par
  LIVRAISON d'edge (reducer) ; timer schedulé chaque execute (première
  deadline gagne) ; repeat/nested liveness (enfants DIRECTS) ; intégrité
  causale requalifiée honnêtement (écrivain unique lease-fencé, validation
  causale = K).
- r2 GAP (0 P0, 4 P1) → join-payload par livraison d'edge (orchestrateur,
  symétrique du reducer — CONFIRMÉ r3) ; watermark AVANT le schedule
  initial ; borne label charset RETIRÉE (plan signé valide reste
  reprenable ; borne de stockage seule) ; test nested discriminant AJOUTÉ
  (repeat contenant map, doubles suffixes ~2N — CONFIRMÉ r3).
- r3 GAP (0 P0, 0 P1 NOUVEAU) → TOCTOU horloge FIXÉ (une seule lecture
  `judged_now`, jugée puis réutilisée) ; 2 résiduels ASSUMÉS P2/P3
  documentés avec filets fail-closed, portés au THREAT :
  (1) cohorte join élargie par un perdant tardif → l'input aval est figé
  par son request-hash memoisé, une re-résolution divergente = terminal
  StepHashMismatch (jamais une réponse stale) ; JoinFired{sources}
  journalisé = durcissement K ; (2) borne de stockage label (256 KiB) plus
  étroite que le domaine String illimité du compilateur — alignement
  contractuel K/L. Le fichier Codex brut n'est PAS modifié.

## Suites finales sur l'arbre commité

Crates core+engine 184/184+3skip ; fmt dual + clippy -D warnings verts ;
workspace Win 2321/2321+3skip et Docker --no-fail-fast 2325/2325+3skip
re-mesurés après le fix TOCTOU (compteurs inchangés, contenus re-runs).
