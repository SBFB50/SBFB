# Sprint 83 Phase I — Review r5 (réconciliation Codex r5, arrêt de boucle)

## Verdict: PASS-PENDING

Codex r5 = **0 P0/P1, 2 P2** (H1 index partiel CONFIRMÉ fermé, H2 PK
exacte CONFIRMÉ fermé, H3/H4 partiels) — le critère d'arrêt canonique
« CLEAN ou P2/P3 documentés » est ATTEINT. Les 2 P2 sont néanmoins fermés :

1. **P2 H3 — FK composite nullable** : la vérification suit exactement la
   recette Codex — `foreign_key_list` groupé par id de contrainte, le
   groupe doit être MONO-ligne (`seq == 0`, `run_id → runs(run_id)`) ; une
   FK composite à composante enfant NULLable est refusée. Résiduel P3
   documenté : pas de bras de test dédié au forgeage composite (recréer la
   table sur une copie = lourd ; le fix suit la recette r5 ligne à ligne
   et la borne vit dans §19).
2. **P2 H4 — docs** : STORE_MIGRATION_OPS ajoute les violations FK
   préexistantes + colonnes surnuméraires aux bornes non comparées ; §19
   reformule la claim FK (mono-colonne exacte) et énumère les bornes.

Suites : crates 146+2skip verts, clippy workspace -D warnings vert,
11/11 mutations tuées (M1..M11). Workspace Win 2273/2273 + Docker
canonique 146/146 + fmt 1.94 (post-r4 ; re-run final au commit).

## Bilan de boucle (5 rounds Codex, 4 rounds review internes)

r1 GAP (3 P1 : fenêtre effet-externe sur-affirmée → rescopée couche
STORE + carry Phase K ; TOCTOU statut GC → relecture in-txn +
interleaving testé ; validation backup → inventaire) ; r2 GAP (2 P1 :
états clos ABSORBANTS `InvalidTransition` + sémantique divergent-sur-
completed tracée, golden terminal régénéré ; colonnes backup) ; r3 GAP
(1 P1 : sémantique index/PK/FK) ; r4 GAP (1 P1 : index partiel, PK
élargie, FK mal ciblée) ; r5 PARTIEL 0 P0/P1 → fermetures + arrêt.
