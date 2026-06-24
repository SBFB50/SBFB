# Sprint 79 — Fix machinery (phase detection) — Review

**Méthode** : Workflow ultracode `wvpp20quq` (5 agents Opus 4.8 1M, 430 K tokens, 96
tool-calls) — fan-out 4 dimensions (équivalence sémantique / correction algorithmique /
complétude / adéquation des tests) + synthèse adversariale. Verdict initial **CONCERN**
(0 P0/P1, 2 P2 « manques de garde ») → **2 P2 corrigés in-phase** → PASS-PENDING.

## Verdict: PASS

Le fix est **algorithmiquement correct et vert** (vérifié in-vivo : nextest -p sbfb-factory,
+9 tests, 0 diff Cargo.lock). **0 P0/P1.** Le verdict initial CONCERN portait sur 2 manques
de garde de test (pas une régression) — tous deux corrigés ci-dessous.

### Dimensions (toutes PASS sauf tests→CONCERN résolu)
- **Équivalence** : comportement A-G préservé ; `done` basculé `Some('G')`→`verification.md`
  (VOULU, débloque les sprints >G comme S77=N ; cohérent avec build_sprint_summary) ;
  champ public MAJUSCULE intact via `display_label` ; comptage par union dédoublonné ;
  front jamais indexé par slot fixe. PASS.
- **Correction** : `next_phase_label` base-26 bijectif sans bug `'z'+1` (le test `if
  chars[i]==b'z'` précède tout incrément → `String::from_utf8` réussit toujours, 0 panic) ;
  `phase_order_key` (len,lexico) défait le tri naïf `aa<b` ; `find_phase_artifact`
  case-insensible des 2 côtés, vrai chemin disque ; `read_dir` absent → vide/None sans
  panic. PASS.
- **Complétude** : les 3 hits du grep de cap sont TOUS en commentaire (documentent l'ancien
  pattern) ; 0 cap en code exécutable ; 5 sites array + 2 regex tous via `crate::phase`.
  PASS.
- **Tests** : CONCERN initial → résolu (voir ci-dessous). PASS.

## P0/P1
Aucun.

## P2 corrigés in-phase
- **P2-1 (CORRIGÉ)** — la regex phare `[A-G][0-9]?`→`[A-Z]+[0-9]?` (PHASE_TITLE_RE +
  PHASE_RE) n'avait AUCUN test multi-lettres : un typo de retour à `[A-G]` passerait les
  178 tests (toutes fixtures mono-lettre « Phase A »). **Ajout** :
  `process::tests::phase_title_re_accepts_unbounded_multi_letter` +
  `sprint_history::tests::phase_re_accepts_unbounded_multi_letter` (asserttent N / AA / F1).
- **P2-2 (CORRIGÉ)** — les fixtures audit-commit testaient la casse dans le sens INVERSE du
  bug (UPPERCASE on-disk). **Ajout** : `audit_commit_resolves_lowercase_active_artifacts`
  (titre « Phase A » MAJUSCULE + fichiers `sprint1_phase_a_*` minuscules → `ok==true` ;
  échouerait Linux pré-fix avec un faux « missing review file »).

## P3 documentés (commit body / carry)
- **P3** — sémantique `done` : un sprint à la dernière phase PASS mais SANS verification.md
  affiche « phase suivante » jusqu'à l'écriture de verification.md (décision PO, commentée
  `process.rs`). Doc body.
- **P3** — cardinalité `status_sprint` : renvoyait toujours 7 entrées A-G ; renvoie
  maintenant les phases réelles. Aucun test n'assertait `length==7` ; strictement plus correct.
- **P3 (carry)** — edge latent (erreur opérateur seulement) : si `phase_a` ET `phase_A`
  coexistent sur ext4, `discover_phase_artifacts` (pas de dédup per-kind) pousse 2 entrées
  même label ; `discover_phase_labels` dédoublonne (status sûr), seuls les consommateurs
  directs voient une redondance inoffensive. Hardening optionnel → carry P3 (non requis).
- **P3** — preuve Docker/Linux DÉCISIVE (le bug ne se manifeste que sur FS case-sensible ;
  Windows natif ne peut pas distinguer l'ancien du nouveau). Faite : 14/14 hermétiques
  Linux verts. Garder `sprint79_machinery_fix_preflight.md` staged.

## Notes adversariales (réfutées par la synthèse)
- FAIL réfuté : le finding regex est un MANQUE DE GARDE sur un code correct-et-vert (les 2
  regex live SONT `[A-Z]+[0-9]?`), pas une régression. La règle réserve FAIL au P0/P1 réel.
- CONFIRMÉ : l'ancien `detect_current_phase` (['A'..'G'] + next-letter) échouerait le nouveau
  test (ne voit pas h/i au-delà de G) → vrai garde-fou cross-platform, pas un no-op Windows.
- CONFIRMÉ : `next_phase_label` ne peut pas paniquer (tout octet écrit ∈ b'a'..b'z').
- RÉFUTÉ comme couverture : `operator_sprint_history_endpoint` n'asserte que `is_array()` →
  ne garde pas la regex (renforce P2-1, corrigé).

## Codex reconciliation
Codex GPT 5.5 (`codex exec`, output brut `sprint79_machinery_fix_codex_review.md`) :
**7/7 livrables CONFIRMÉ, 0 GAP, 0 PARTIEL.** Codex a indépendamment :
- vérifié chaque livrable avec evidence file:line (module `phase.rs`, detect_current_phase,
  status_sprint_data, les 2 regex `[A-Z]+[0-9]?`, audit_commit_data case-insensible, les 3
  `build_*`, les tests) ;
- ré-exécuté les tests (`cargo test -p sbfb-factory phase` + `sprint_summary_is_unbounded…`
  → pass) ;
- passe adversariale : les anciens caps trouvés par `rg` sont **uniquement en
  commentaires/tests de régression**, 0 cap en code ; **0 changement Cargo.lock/Cargo.toml**,
  0 dépendance ajoutée.
Aucun GAP → aucune correction requise après les 2 P2 déjà corrigés. Review promue PASS.
Le fichier Codex brut n'est ni réécrit ni résumé.
