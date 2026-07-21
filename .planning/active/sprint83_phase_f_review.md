# Sprint 83 Phase F — Review round 5 (promotion finale)

**Sprint** : 83. **Phase** : F — Format portable et signé (`SBFBPromptAgentPack.v1`).
**Date** : 2026-07-22. **HEAD** : `6f95cd2` (arbre sale, commit de phase à suivre).
**Méthode** : synthèse des rounds r1 (8 dimensions Workflow, 79 findings), r2 (6 vérificateurs
+ 2 critiques, PASS-PENDING), r3 (4 vérificateurs, CONCERN → P1 census fixé), r4 (4 vérificateurs
Fable, CONCERN → P1 import + P2 prose + P3 header fixés), et des 5 rounds Codex GPT-5.6 Sol
(r1/r2 sandbox mort fail-closed ; r3 BLOQUÉ 5 P1 ; r4 BLOQUÉ 1 P1 crypto ; r5 BLOQUÉ 2 P1 edge).

## Disposition du round Codex r5

- P1-1 (surrogate isolé dans une CLÉ JSON, miroir TS) : **FIXED** — `assertWellFormed` appliqué
  aux clés comme aux valeurs (`pack_digest.test.ts`), TS 19/19.
- P1-2 (secret possible dans le hostname du scanner) : **FIXED** — host credential-shaped masqué
  (`scheme_and_host`, `pack.rs`), crate 47/47.
- P2-1 (confinement à la matérialisation) : ROUTÉ — la vérification matérialisée
  (`MaterializedVerifiedPack`) appartient aux phases d'installation ; le format v1 refuse par FORME.
- P2-2 (amplification mémoire avant bornes) : DOCUMENTÉ — borne honnête = `MAX_PACK_JSON_BYTES`
  (1 MiB), écrite sur la constante.
- P2-3 (tests discriminants des caps lock) : DOCUMENTÉ P3-testing, wrap-up.
- P2-4 (erreurs recopiant des chemins hostiles) : DOCUMENTÉ — les chemins de lock sont
  ASCII-alphabet borné 512 octets, la seule classe secret (secret_refs) est déjà withheld.
- P2-5 (heuristique refuse des refs légitimes 40+ chars mixtes) : DOCUMENTÉ — faux positif
  assumé du shape-rule, contournable en nommant le slot autrement.
- P3-1..3 : commentaire JCS ajusté au fil des rounds ; tripwire lexical documenté comme tel ;
  endpoint unique par chaîne assumé (advisory).

## Suites observées (arbre final)

Rust Windows workspace 2192/2192 + 1 skip (nextest exit 0 re-observé après les derniers fixes) ;
Docker canonique 2196/2196 + 1 skip + fmt 0 ; crate 47/47 (`-p` ET mode workspace) ;
`import_hostile` 5/5 ; clippy workspace -D warnings 0 ; doctests 0 ; release build 0 ;
fmt canonique + local 0 ; web 548 (coverage 86.94/78.94/86.13/88.35, build, size 6/6,
scan-en-strings clean, lint 0 erreur) ; TS parité 19/19 ; gates frontier (census 29 nominal
LC_ALL=C) / spdx / factory-docs exit 0. Flakes hors-phase re-passés solo : GpuConsentDialog 17/17,
boot_path_reenters vert.

## Codex reconciliation

Cinq rounds numérotés conservés bruts dans `rounds/` (canonique = copie du r5). r1/r2 : sandbox
shell mort post-reboot, verdicts fail-closed honnêtes — l'audit a basculé sur bundle inline.
r3 : 5 P1 tous fermés (collision `presentation_view` prouvée par contre-exemple → clés
segment-arrays ; collapse `serde_jcs` 2^53+1 CONFIRMÉ PAR MESURE → porte I-JSON ; tests
post-signature discriminants via rebind_content + fragments, prouvé par mutations backslash/tri ;
4 domaines D2 déclarés, census 25→29 en liste NOMINALE ; secret withheld + prose).
r4 : P1 crypto verify_strict/is_weak fermé avec test de forgerie R=identité/S=0 (la mutation
retour-à-verify le rend rouge) ; P2 2/3/4/5 fixés (frontière TS, expects mécaniques, golden
littéral content, scanner scheme+host) ; P2 1/6/7/8 + P3 en prose honnête.
r5 : 2 P1 edge fixés ci-dessus ; arrêt de boucle au critère canon « CLEAN ou P2/P3 documentés »
(README §4.5, memory feedback_codex_loop_stop_criterion) — les résiduels sont documentés-routés,
chacun tracé au commit body.

## Verdict: PASS
