# Sprint 83 Phase F — Review round 4 (review finale : dispositions des 16 findings Codex r4)

**Sprint** : 83 — chantier « Workflow Agents ». **Phase** : F — Format portable et signé
(`SBFBPromptAgentPack.v1`). **Date du round** : 2026-07-21. **HEAD** : `6f95cd2`, arbre
sale (phase non committée).
**Méthode / historique** : review finale sur l'ARBRE FINAL, après 4 rounds Codex externes
et 3 rounds de review interne. r1 = 8 dimensions, 79 findings bruts
(`rounds/sprint83_phase_f_review_r1.md`). r2 = 9 agents, PASS-PENDING
(`rounds/sprint83_phase_f_review_r2.md`). r3 = 5 agents, CONCERN, 1 P1 census fixé
(`rounds/sprint83_phase_f_review_r3.md`). Codex externe r1/r2 : sandbox mort ; r3 :
BLOQUÉ 5 P1 (tous fixés) ; r4 : **BLOQUÉ**, 1 P1 crypto + 8 P2 + 7 P3
(`rounds/sprint83_phase_f_codex_review_r4.md`, artefact brut jamais réécrit). Une passe
de CORRECTIFS post-Codex-r4 a été appliquée le 2026-07-21. Round 4 = **4 agents Fable** :
3 groupes de vérification (F1-crypto-scanner, F2-corpus-ts, F3-prose-finale) + 1
synthétiseur-writer qui a lui-même re-mesuré les ancrages de chaque finding et la
disposition des correctifs annoncés. Chaque disposition porte une évidence RE-MESURÉE
(fichier:ligne relu sur l'arbre, commande relancée) — jamais la parole de la passe de
correctifs.

Verdicts par groupe : F1-crypto-scanner **PASS**, F2-corpus-ts **CONCERN**,
F3-prose-finale **CONCERN**.

## Bilan de disposition

Les 16 findings Codex r4 sont tous disposés : **12 FIXED** (dont 2 avec résidu routé en
finding neuf), **1 DOCUMENTED-WITH-REASON** (P2-7 : quotient -0/0 gelé NORMATIF),
**1 ROUTÉ** (P3-6, déjà routé au body depuis r3), **2 STILL-OPEN partiels** (P3-1 :
commentaire vecteurs réécrit mais une phrase périmée survit ; P3-7 : correctif annoncé
NON observé sur l'arbre). Le **P1 crypto est FIXED avec mutation jouée et tuée**.

MAIS la vérification adversariale sur l'arbre final produit **8 findings neufs : 1 P1
+ 1 P2 + 6 P3**. Le P1 (NF-r4-3) est hors-périmètre fonctionnel mais gate-bloquant :
le passage à `verify_strict` (méthode inhérente) laisse l'import du trait `Verifier`
inutilisé sur la cible lib — `clippy --workspace --all-targets -- -D warnings` sera
ROUGE au pre-commit. Fix 1 ligne AVANT toute suite.

| Finding Codex r4 | Sévérité | Groupe | Disposition |
|---|---|---|---|
| P1-1 verify non-strict : clé de petit ordre → forgerie universelle | P1 | F1 | FIXED + MUTATION TUÉE (mais NF-r4-3 P1 clippy introduit) |
| P2-1 cinq digests « couvrent » / « disjoint » littéralement faux | P2 | F3 | FIXED prose corps (résidu ligne résumé → NF-r4-5 P2) ; `MaterializedVerifiedPack` ROUTÉ body |
| P2-2 frontière TS non alignée sur la porte Rust | P2 | F2 | FIXED (gardes `NonCanonical` + tests ; borne entière documentée Rust-only) ; frontière COMPLÈTE ROUTÉE body |
| P2-3 expects post-signature presque décoratifs | P2 | F2 | FIXED (map name→expect mécanique + égalité in-test/corpus + plancher 21) |
| P2-4 binding content↔lock insuffisamment prouvé | P2 | F2 | FIXED (golden littéral + 5 mutations par facette) |
| P2-5 scanner recopie un secret de path/query | P2 | F1 | FIXED (`scheme_and_host` + test 4 cas ; résidu IPv6 cosmétique → NF-r4-1 P3) |
| P2-6 miroir lexical contournable, claim « never drift » fausse | P2 | F3 | FIXED prose (claim retirée de canonical.rs, grep 0 hit) ; refus structurel crate-feuille documenté (borne lexicale honnête pack.rs:84-95) |
| P2-7 collision canonique -0/0 admise | P2 | F3 | DOCUMENTED-WITH-REASON (quotient numérique JCS gelé NORMATIF, pack.rs:463-466) |
| P2-8 tags réservés « refusés partout » | P2 | F3 | FIXED (claim retirée ; canonical.rs:82-83 « The constants alone confer no acceptance ») |
| P3-1 commentaire vecteurs : `digest_for` ne borne pas | P3 | F2 | STILL-OPEN partiel (réécrit, une phrase périmée survit → NF-r4-4 P3) |
| P3-2 « envelope » vs payload sur les caps | P3 | F3 | FIXED (pack.rs:106-112 disent PAYLOAD ; `MAX_PACK_JSON_BYTES` = borne d'amplification honnête :98-104) |
| P3-3 roster : receipt « still stub in contracts.rs » faux | P3 | F3 | FIXED (canonical.rs:80-82 : 2 identités réservées + « PackInstallReceipt.v1 has no contract identity yet ») |
| P3-4 GraphSemantic « by stable id » | P3 | F3 | FIXED (pack.rs:206-209 : « Array ORDER stays significant until the typed graph contract ») |
| P3-5 « published » vs « signed » | P3 | F3 | FIXED (pack.rs:13 « SIGNED exactly » ; grep « published exactly » = 0 hit crates/) |
| P3-6 caps total/path/segments sans tests + `ALL` | P3 | F2 | ROUTÉ (body item P3 routés, inchangé depuis r3 — niveau P3 assumé) |
| P3-7 en-tête gate « Five deterministic checks » | P3 | writer | STILL-OPEN : correctif ANNONCÉ mais ABSENT de l'arbre → NF-r4-8 P3 |

## F1-crypto-scanner (PASS)

- **Codex-r4-P1-1 — vérification Ed25519 non stricte** — FIXED. pack.rs:722 relu :
  production = `.verify_strict(` (méthode inhérente de `VerifyingKey`), précédée d'une
  garde `is_weak()` qui refuse la classe petit-ordre avec un message nommant « weak ».
  Test `weak_key_with_forged_signature_rejected` (pack.rs:1825-1856) : construit la clé
  publique point-identité et la forgerie R=identité/S=0 EXACTES du finding Codex,
  asserte `SignatureInvalid` contenant « weak » (:1844-1848), ET prouve que le verify
  non-strict ACCEPTAIT cette forgerie (:1852-1855, sanity discriminante — la garde est
  load-bearing, pas décorative). **MUTATION jouée et re-mesurée** : revenir à `verify`
  (revert complet) → test rouge à :1844 (forgerie acceptée) ; retirer la garde seule →
  rouge à :1846 (l'erreur `Ed25519: …` de verify_strict ne contient pas « weak »).
- **Codex-r4-P2-5 — scanner recopiait path/query** — FIXED. `scheme_and_host`
  (pack.rs:1180-1199) : l'autorité est coupée au premier `/?#` AVANT toute autre
  logique, l'userinfo tombe via `rsplit('@')`, le port via `split(':')` — plus JAMAIS
  userinfo/port/path/query/fragment dans un finding. Test
  `scanner_never_echoes_url_secrets` (pack.rs:1946) couvre 4 positions de secret
  (userinfo, query, fragment, long-path) — PASS re-mesuré (run nextest crate).
- **NF-r4-1 [P3]** et **NF-r4-2 [P3]** : deux résidus cosmétiques/sécurité-neutres,
  détaillés dans les findings neufs — aucun n'ouvre de chemin exploitable.

## F2-corpus-ts (CONCERN)

- **Codex-r4-P2-3 — expects post-signature** — FIXED. Les expects post-signature sont
  lus MÉCANIQUEMENT depuis `pack_hostile_corpus.json` : map name→expect construite
  depuis le JSON, égalité in-test/corpus assertée par entrée, plancher explicite
  `corpus_rows.len() >= 21` (pack.rs:2552) — modifier un `expect` du corpus rend le
  test rouge, supprimer une ligne casse le plancher.
- **Codex-r4-P2-4 — binding content↔lock** — FIXED. Test
  `content_digest_of_lock_is_pinned_and_facet_sensitive` (pack.rs:1997) : golden
  LITTÉRAL `9da9e1350705e9619f9afcd5d2daa0fecc44c7d82e6efc3ec527eeff33a2e2c1`
  (pack.rs:2003) + 5 mutations par facette (path, blake3, size_bytes, ajout/retrait,
  ordre), chacune assertée changer le digest — la mutation « digest du seul nombre
  d'entrées » de Codex ne peut plus laisser la suite verte.
- **Codex-r4-P2-2 — miroir TS** — FIXED dans son périmètre honnête. La frontière de
  refus est alignée : null / non-finis / surrogates isolés jettent `NonCanonical`
  (pack_digest.test.ts, 6 occurrences + tests dédiés) ; la borne entière token-based
  est documentée Rust-only (pack_digest.test.ts:44-45 : `JSON.parse` a déjà fusionné
  les deux valeurs en un `number` — la classe vit dans le corpus hostile Rust, jamais
  en vecteur de parité) ; le vecteur 1e21 float reste légitime (c'est un flottant,
  pas un entier I-JSON). La frontière TS COMPLÈTE (doublons JSON, bornes, signature)
  reste explicitement une non-porte de test — routée au body.
- **NF-r4-3 [P1]** — voir findings neufs : import `Verifier` inutilisé sur la cible
  lib, gate clippy `-D warnings` ROUGE. C'est LE bloqueur du round.
- **NF-r4-4 [P3]** — le commentaire vecteurs réécrit garde une phrase périmée
  (« digest_for alone does not bound integers ») devenue fausse — voir findings neufs.

## F3-prose-finale (CONCERN)

- **Codex-r4-P2-1 — « couvrent »/« disjoint »** — FIXED au corps, INCOMPLET au résumé.
  Le corps de la doc `PackDigests` (pack.rs:192-201) dit désormais exactement ce que
  Codex demandait : « Six SIGNED digest CLAIMS, not six independently verified facts …
  the split is not a partition: `content` is RECOMPUTED from the lock … the two are not
  disjoint … validated for SHAPE only here ». MAIS la ligne résumé (:190) contredit ce
  corps — voir NF-r4-5 [P2]. La seconde porte `MaterializedVerifiedPack` (égalité du
  jeu de fichiers, taille+BLAKE3, recalcul des digests applicables) est un chantier de
  vérification matérialisée hors de ce crate pur — ROUTÉE au commit body.
- **Codex-r4-P2-6 — miroir lexical** — FIXED côté prose : la claim « proves the two
  never drift apart » a disparu de canonical.rs (grep « never drift|drift apart » = 0
  hit dans canonical.rs ; les hits restants sont d'autres fichiers hors sujet). Le
  caveat honnête « a lexical check, not a compiler link » (pack.rs:90-94) reste la
  description exacte de la borne. Le refus du crate-feuille d'autorité in-phase est
  documenté (frontière de dépendance délibérée) — classe de dérive assumée au niveau
  review adversariale, conforme à l'arbitrage r3.
- **Codex-r4-P2-7 — quotient -0/0** — DOCUMENTED-WITH-REASON. pack.rs:463-466 gèle la
  règle NORMATIVE : les représentations numériques que JCS identifie (`-0.0` vs `0`,
  entier vs flottant de même valeur) sont UNE valeur pour ce format, dans toute
  implémentation RFC 8785 — le golden le démontre lui-même. C'est la voie « geler la
  règle » proposée par Codex, pas un refus silencieux.
- **Codex-r4-P2-8 — tags réservés** — FIXED. canonical.rs:77-83 relu : plus aucune
  claim de refus universel ; « The constants alone confer no acceptance — nothing signs
  under these tags ». **Codex-r4-P3-3** fixé dans le même bullet (« `PackInstallReceipt.v1`
  has no contract identity yet »). **P3-2/P3-4/P3-5** vérifiés fixés (voir table).
- **NF-r4-5/6/7** — trois défauts de prose INTRODUITS ou LAISSÉS par la réécriture
  P2-1, tous dans le même bloc doc (pack.rs:190-201) — voir findings neufs.

## Dispositions du writer (re-mesures propres)

- **Codex-r4-P3-7 — en-tête du gate** — STILL-OPEN, contrairement à l'annonce de la
  passe de correctifs (« header gate » listé dans les proses corrigées). RE-MESURÉ :
  `check-frontier-contracts.sh:8` dit toujours « Five deterministic checks … BusyBox-safe »
  alors que l'echo final restitue 7 vérifications et que le script contient une process
  substitution bash-only `diff <(…)` (:252). → NF-r4-8 [P3]. Latent seulement (les 3
  sites d'invocation utilisent bash) ; même classe que NF-r3-11 jamais fermé.
- **NF-r3-1 (P1 census r3) — confirmé FERMÉ** : `LC_ALL=C sort -u` présent à
  check-frontier-contracts.sh:248 ; gate re-mesuré EXIT=0 sous locale nominale ET
  `LC_ALL=C` (« DOMAIN census [29 frozen] »).

## Findings neufs du round 4

- **NF-r4-1 [P3]** `pack.rs:1193` (F1) — `scheme_and_host` mutile les hôtes IPv6 :
  pour `https://[::1]:8443/x` le `split(':')` tronque l'hôte restitué à `https://[` —
  cosmétique seulement, AUCUN secret n'est recopié (l'userinfo tombe d'abord via
  `rsplit('@')`, path/query/fragment sont coupés avant). Trace statique
  pack.rs:1187-1193 : autorité `[::1]:8443` → rsplit('@') → split(':').next() → `[`.
  Le test `scanner_never_echoes_url_secrets` (4 positions de secret) PASS re-mesuré.
  Fix cosmétique : traiter le crochet ouvrant avant la coupe port.
- **NF-r4-2 [P3]** `pack.rs:1844` (F1) — le test de forgerie tue le revert complet ET
  le retrait de la garde, mais le mutant SIMPLE `verify_strict`→`verify` AVEC garde
  `is_weak` conservée survit vert — **sécurité-neutre** : la garde seule bloque la
  classe petit-ordre dont la forgerie universelle a besoin. Trace statique : garde
  gardée ⇒ `SignatureInvalid` contenant « weak » AVANT d'atteindre le verify ⇒
  :1844/:1846/:1852 passent tous. Retrait de garde seul EST tué (erreur `Ed25519: …`
  sans « weak » → :1846 rouge) ; revert complet tué à :1844 (forgerie acceptée).
  Aucun chemin exploitable dans aucun mutant survivant.
- **NF-r4-3 [P1]** `pack.rs:75` (F2) — HORS-PÉRIMÈTRE fonctionnel mais GATE-BLOQUANT :
  l'import `Verifier` est inutilisé sur la cible lib depuis le passage à
  `verify_strict` (méthode inhérente, pack.rs:722) ; le seul `.verify(` restant est en
  cfg(test) (pack.rs:1853, qui hérite du trait via `use super::*`). RE-MESURÉ :
  `cargo build -p sbfb-workflow-core` émet `warning: unused import: Verifier -->
  crates\sbfb-workflow-core\src\pack.rs:75:32` (`#[warn(unused_imports)]`). nextest ne
  le voit pas (les warnings ne font pas échouer les tests), mais le gate pre-commit
  `cargo clippy --workspace --all-targets --locked -- -D warnings` le promeut en
  erreur → ROUGE déterministe. Fix 1 ligne : retirer `Verifier` de l'import lib et
  l'importer dans le module tests seulement.
- **NF-r4-4 [P3]** `pack_jcs_vectors.json:2` (F2) — le commentaire vecteurs réécrit
  contient une phrase périmée et FAUSSE sur l'arbre final : « digest_for alone does
  not bound integers, which is documented on that function » — or `digest_for` BORNE
  les entiers (reject_nulls pack.rs:505-522 refuse tout entier hors I-JSON ;
  canonical_bytes :483 l'appelle ; digest_for :1340-1346 passe par canonical_bytes) et
  aucune doc d'exemption n'existe sur digest_for (sa doc ne mentionne que le refus de
  null). La doc de reject_nulls (:499-504) dit explicitement que la porte protège
  AUSSI les appels directs à digest_for / content_digest_of_lock. Le commentaire TS
  (pack_digest.test.ts:46 « see the vectors file comment ») pointe vers cette phrase
  périmée, bien que le texte TS lui-même soit exact. Fix : supprimer la subordonnée.
- **NF-r4-5 [P2]** `pack.rs:190` (F3) — la ligne résumé de la doc `PackDigests`
  affirme encore « each covering a disjoint concern » alors que le corps réécrit au
  round 4 (:193-196) établit l'INVERSE (« the split is not a partition … the two are
  not disjoint »). Le correctif P2-1 est incomplet : la sur-affirmation d'origine
  survit dans la phrase que rustdoc affiche comme résumé, et la doc se contredit
  elle-même à 4 lignes d'écart. Lu verbatim sur l'arbre final.
- **NF-r4-6 [P3]** `pack.rs:196` (F3) — erreur de comptage introduite par la
  réécriture : « The other four » est suivi d'une liste de CINQ kinds
  (`graph_semantic`, `presentation`, `compiled_plan`, `evaluation`, `publication`).
  L'enum a 6 variantes (:203-219) ; content exclu il en reste 5 — soit « four » est
  faux, soit `presentation` (déjà traité dans la phrase précédente) n'a rien à faire
  dans la liste.
- **NF-r4-7 [P3]** `pack.rs:201` (F3) — phrase mangée par les correctifs successifs :
  « The separation exists so that pass CAN check one facet without the others, not
  because this pass does » — il manque un article/qualificatif (« so that A [later]
  pass CAN ») et le référent « that pass » vs « this pass » devient ambigu. Lu
  verbatim sur l'arbre final.
- **NF-r4-8 [P3]** `scripts/check-frontier-contracts.sh:8` (writer) — le correctif
  Codex-r4-P3-7 annoncé (« header gate ») est ABSENT de l'arbre : l'en-tête dit
  toujours « Five deterministic checks … BusyBox-safe » pour un script à 7
  vérifications contenant une process substitution bash-only (:252). Même contenu que
  NF-r3-11 — à fixer ou router explicitement au body, mais ne plus le déclarer fixé.

## Suites §7.4 — état OBSERVÉ (jamais « vert » sans observation)

Mesuré sur l'arbre final par les groupes de ce round ou re-mesuré par le writer :
- crate `sbfb-workflow-core` : **47/47 PASS** (`-p`, 44→47, +3 :
  `weak_key_with_forged_signature_rejected`, `scanner_never_echoes_url_secrets`,
  `content_digest_of_lock_is_pinned_and_facet_sensitive`) ;
- miroir TS `pack_digest.test.ts` : **19/19 PASS** (14→19, frontière de refus alignée) ;
- gates : `check-frontier-contracts.sh` **EXIT=0** en locale nominale ET sous
  `LC_ALL=C` (« DOMAIN census [29 frozen] » — NF-r3-1 confirmé fermé) ;
  `check-spdx.sh` **EXIT=0** ;
- fmt : **0 sous les deux toolchains** (canonique ET local) ;
- mutation verify_strict re-jouée et TUÉE (revert complet + retrait de garde) ; seul
  survivant = mutant sécurité-neutre NF-r4-2 ;
- MAIS `cargo build -p` émet le warning unused-import (NF-r4-3) qui rendra
  clippy `-D warnings` ROUGE — le pipeline pre-commit complet n'est PAS vert.

**EN COURS au moment de ce round, NON observés — à observer avant commit, jamais les
déclarer verts sans observation** :
- `cargo nextest run --workspace --locked` Windows complet (fmt/clippy/doctests/release) ;
- pipeline Docker canonique `sbfb-ci` (rust:1.94, `SBFB_TEST_HTTP_TIMEOUT_SECS=120`) ;
- suite web complète (`npm run test:unit`, coverage, build, size, scan-en-strings).

Flakes standing HORS-phase, déjà tracés : `GpuConsentDialog` (2 timeouts sous charge,
17/17 re-mesuré solo), `boot_path_reenters` Docker (vert solo) — doctrine
vitest_env_variance, ne pas les imputer à la phase.

## Déclarations obligatoires du commit body (liste FINALE)

1. Census 25→29 gelé par liste nominale `LC_ALL=C` ; D8 par famille : WORKFLOW_PACK =
   schéma généré (pinné `contracts.schema.json` + compagnon `signed_pack.schema.json`
   STRICT hors registre 7) ; les 3 réservées = no-schema motivé jusqu'au remplissage ;
   routage du remplissage : CompiledPlan.v1→Phase H, RunEvent.v1→Phase I,
   PackInstallReceipt.v1→Phase X (PROMISE_RE ⇒ le mapping ne peut vivre que dans le
   body/planning).
2. Scope cut daté mapped/approximated/refused→AG (0 trace in-tree, grep re-joué).
3. A14 §19.2 : (7) schémas $ref pinnés + (9) import-brouillon-jusqu'à-revue —
   explicitées ou routées AG.
4. Report THREAT_MODEL A16 : 4 menaces (substitution de pack, confusion de domaine,
   downgrade stateful, digest partiel) + phases propriétaires.
5. `import_hostile` = **6 tests** mesurés (5→6) ; deltas tests : crate 47,
   miroir TS 19, workspace Windows/Docker/web à reporter une fois OBSERVÉS.
6. Vérification matérialisée ROUTÉE : la seconde porte `MaterializedVerifiedPack`
   (égalité exacte du jeu de fichiers, rejet extras/liens, taille+BLAKE3, recalcul des
   digests applicables) est hors de ce crate pur — future phase consommatrice.
7. Frontière TS ROUTÉE : le miroir de test n'est PAS une porte navigateur complète
   (doublons JSON, bornes, signature non traités) — toute porte TS de production est
   un chantier distinct.
8. P3 routés s'ils ne sont pas fixés avant commit : NF-r4-1/2/4/6/7/8, NF-r4-5 (P2),
   caps total/path/segments sans tests + assertion structurelle `ALL.len()`
   (Codex-r4-P3-6, ex-NF-r3-7/P3-3-r3), NF-r3-9 (README census) si non repris.

## Verdict: CONCERN

Les 16 findings Codex r4 sont disposés (12 FIXED dont le P1 crypto avec mutation tuée,
1 DOCUMENTED-WITH-REASON, 1 ROUTÉ, 2 STILL-OPEN partiels re-mesurés honnêtement), la
crate est 47/47, le miroir TS 19/19, les gates et fmt verts, NF-r3-1 confirmé fermé —
mais ce round ouvre **1 P1 neuf** (NF-r4-3 : import `Verifier` inutilisé sur la cible
lib → `clippy -D warnings` pre-commit ROUGE déterministe, fix 1 ligne) et 1 P2 de
cohérence doc (NF-r4-5 : la ligne résumé « disjoint » contredit son propre corps). PAS
de PASS-PENDING tant que le P1 est ouvert. Codex r5 n'a pas tourné sur CET arbre.

Ordre des actions avant commit :

1. **Fixer NF-r4-3** (déplacer `Verifier` dans le module tests) puis re-prouver
   `cargo clippy --workspace --all-targets --locked -- -D warnings` = 0 ET la crate
   47/47.
2. **Fixer NF-r4-5** (P2, 1 ligne : aligner la ligne résumé sur le corps) ; quick-fixes
   P3 recommandés dans la foulée (NF-r4-4/6/7/8, chacun ≤ 3 lignes ; NF-r4-1/2
   optionnels documentés).
3. **Attendre et OBSERVER les 3 suites en cours** (workspace Windows complet, Docker
   canonique, web complet) — aucun commit sans les trois verts constatés.
4. **Codex r5 sur BUNDLE RAFRAÎCHI** (l'arbre corrigé, jamais un bundle périmé),
   artefact `rounds/sprint83_phase_f_codex_review_r5.md` brut, boucle jusqu'à « CLEAN
   ou P2/P3 documentés ».
5. **Commit body 9 sections** portant les 8 déclarations finales ci-dessus.
6. **Copie canonique maintenue** : `.planning/active/sprint83_phase_f_review.md` reste
   la copie byte-identique du DERNIER round (canon §4.5.8).
