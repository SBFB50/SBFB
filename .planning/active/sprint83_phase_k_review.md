# Review — Sprint 83 Phase K — Effets, retries et idempotence

## Verdict: PASS

Canonique = promotion du round r5 (`rounds/sprint83_phase_k_review_r5.md`)
après le round Codex r9 (PARTIEL, critère d'arrêt ATTEINT : 0 P0, 0 P1
technique, 0 P1 process, 0 P2, 1 P3 documentaire — « aucun correctif code
ni r10 requis »). Historique complet de la boucle : rounds review r1..r5 +
rounds Codex r1..r9, tous conservés dans `rounds/`, jamais réécrits
(canon §4.5.8). Le canonique Codex `sprint83_phase_k_codex_review.md` est
la copie byte-identique du r9.

## La boucle en une ligne

Une seule classe a porté 7 des 9 rounds : « un ingrédient qui influence
l'effet mais manque dans l'identité durable (`dedup_scope`) permet à un
resume de refaire "le même travail" en visant autre chose ». Fermée par
inventaire EXHAUSTIF : état worktree attendu (r2), racine FS (r3, figée
r4), cwd process (r4), program (r5, ABSOLU-only r6, suffixe `.exe`
Windows r7), log fenced (r5, feuille nue r6). Chaque fermeture porte un
test discriminant + une mutation ACTIVE jouée (15 mutations de boucle
tuées, annotées dans les tests ; effects.rs en documente ~25 au total).
Le solde : echo.rs = preuve GELÉE Phase E (byte-intact, documenté),
chemins store hors classe (le fichier + lease EST l'autorité durable).

## Synthèse des rounds review (détail dans rounds/)

- r1..r3 : boucle initiale (voir `rounds/sprint83_phase_k_review_r3.md`
  — traitement des 5 P1 Codex r1, 4 P2 fermés code, rétrogradations
  honnêtes receipt/Unix/fencing).
- r4 : round consolidé post-r3 (Codex r4-r6 + 3 Workflow reviews +
  fermeture de classe). Round HISTORIQUE aux compteurs de son époque.
- r5 : review post-correctif r7→r8 exigée par Codex r8 (P1 processuel
  K-ROUND-REVIEW-AFTER-FIX-08, FERMÉ) — delta-r8 AUCUN FINDING (gate
  suffixe vérifié contre la SOURCE std 1.95 indépendamment),
  final-proofs CLEAN + 1 P3 fermé (annotations mutations).

## Vérification §7.4 (arbre final)

- Windows : fmt 0 ; clippy workspace -D warnings OK ; nextest
  2364 tests (2363 verts + dispatch_loop convergence = classe flake
  STANDING carry 7, solo PASS 1.8s) + 7 skip ; doctests OK ; release
  OK ; rustdoc engine OK.
- Docker sbfb-ci : fmt OK ; nextest 2369 tests (2367 verts + sigint +
  worker start_headless = classes STANDING carry 7, solo Docker 2/2
  PASS) + 7 skip ; le run r7 quasi-identique côté Linux (delta r8+
  cfg(windows)-inerte) avait donné 2369/2369 0-ROUGE.
- Engine+core : 227 run + 7 skip. Web 548 + operator 201 verts (blocs
  complets). check-frontier clean, census 29, goldens byte-identiques
  (hashes confirmés indépendamment par review r5 ET Codex r8/r9),
  0 nœud lock neuf (992 avant/après).

## Codex reconciliation

GPT-5.6 Sol effort max, 9 rounds numérotés
(`rounds/sprint83_phase_k_codex_review_r1..r9.md`), canonique = copie
byte-identique du r9. r1 GAP 5 P1 ; r2 GAP 2 P1 ; r3 GAP 1 P1 ; r4 GAP
1 P1 + 2 P2 + 1 P3 ; r5 GAP 2 P1 + 1 P2 + 1 P3 ; r6 GAP 1 P1 + 1 P2 ;
r7 GAP 1 P1 ; r8 CLEAN technique + 1 P1 processuel (review manquante →
review r5) ; r9 PARTIEL = scellement (0 P0/P1/P2, 1 P3 documentaire).
Tous les P0/P1/P2 fermés en code ou rétrogradés honnêtement en bornes
documentées ; chaque fix re-vérifié par le round suivant.

**P3 résiduel r9 (K-MUTATION-PROVENANCE-R9), documenté — provenance
corrigée ici** : la review r5 citait « documentées review r3 » pour les
mutations r2 « parse-laxiste »/« contains_key » ; la source exacte est
le PROMPT Codex r3 (`.git/CODEX_SPRINT83_PHASE_K_R3.txt`, qui décrit
« mutation « parse laxiste » jouée → test rouge » et « mutation
« contains_key » → rouge »), review_r3.md ne les nomme pas. La
DISCRIMINANCE des deux tests est CONFIRMÉE par Codex r9 contre le code
vivant (orchestrator.rs:440, reducer.rs:475/490) ; seul le pointeur de
provenance était inexact. r3/r5/r9 restent gelés ; la correction vit
ici et au commit body.

## Décision

Boucle arrêtée au critère canonique « CLEAN ou uniquement P2/P3
documentés ». Phase K committable : canoniques promus, staging
inventorié (review r5 §final-proofs), body 9 sections prêt.
`sprint83_plan_po.md` reste HORS du commit de phase (chore(planning)
séparé).
