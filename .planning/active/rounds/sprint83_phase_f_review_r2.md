# Sprint 83 Phase F — Review round 2 (post-correctifs, verification adversariale)

**Sprint** : 83 — chantier « Workflow Agents ». **Phase** : F — Format portable et signé
(`SBFBPromptAgentPack.v1`). **Date du round** : 2026-07-21. **HEAD** : `6f95cd2`, arbre
sale (phase non committée).
**Méthode** : round 1 = 8 dimensions de review, 79 findings bruts dont 19 P0/P1,
consignés dans `rounds/sprint83_phase_f_review_r1.md` (passe adversariale tronquée par
limite de modèle). Une passe de CORRECTIFS a ensuite été appliquée (23 correctifs + 4
écarts assumés). Round 2 = **9 agents** : 6 vérificateurs de disposition (V1
correctness-sécurité, V2 sémantique des tests, V3 preflight-livrables, V4 patterns-docs,
V5 honnêteté, V6 corpus-gates) + 2 critiques de complétude balayant l'arbre corrigé au-delà
des findings r1 + 1 synthétiseur-writer. Chaque verdict porte une évidence RE-MESURÉE
(fichier:ligne relu, commande relancée) — jamais la parole de la passe de correctifs.

Verdicts par groupe : V1 **PASS**, V2 **CONCERN**, V3 **CONCERN**, V4 **CONCERN**,
V5 **PASS**, V6 **PASS**.

## Bilan de disposition

87 dispositions rendues (les recouvrements entre groupes comptent séparément — plusieurs
vérificateurs ont re-jugé le même finding r1 sous des angles différents) :
**69 FIXED**, **10 DOCUMENTED-WITH-REASON**, **8 STILL-OPEN** (7 findings distincts,
`pack.rs:426` jugé deux fois), **0 WAS-WRONG**. Aucun P0/P1 de r1 ne reste ouvert :
les 19 P0/P1 de r1 sont tous FIXED ou DOCUMENTED-WITH-REASON avec évidence re-mesurée.
Les STILL-OPEN résiduels sont des P2 re-évalués à la baisse ou des P3.

## V1 — Correctness et sécurité (PASS)

- **pack.rs:37 [P1] — test d'ordre hors du gate import_hostile** — FIXED. Re-mesuré :
  `cargo nextest run -p sbfb-workflow-core --offline import_hostile` sélectionne 5 tests
  dont `pack::import_hostile::signature_before_parse` (5/5 PASS). Le test vit à
  pack.rs:1696 dans `mod import_hostile` ; la référence du module-doc (:43-44) est devenue
  vraie. Discriminance relue : payload à champ inconnu + signature `[0;64]` → l'ordre
  inverse donnerait `SchemaViolation`, le test exige `SignatureInvalid`.
- **pack.rs:815 [P1] — PRESENTATION_KEYS filtre par nom, doc « never meaning » fausse** —
  DOCUMENTED-WITH-REASON. Doc réécrite en RÉSERVATION de noms (pack.rs:1027-1037, « a
  reservation, not an observation ») ; coût démontré par
  `tests::presentation_names_are_reserved_by_contract` (:1617-1636, PASS) : un `width`
  sémantique est strippé ET resurgit dans `presentation_view`. Écart assumé #26 : « ids
  explicites + tri total » routé vers le schéma de graphe (Phase G). La collision persiste
  par contrat, plus par accident.
- **pack.rs:650 [P1] — 8/22 noms DOS seulement** — FIXED. RESERVED = 22 noms
  (pack.rs:833-836 : con/prn/aux/nul + com1-9 + lpt1-9, compte 4+9+9=22 vérifié). Stem via
  `split('.').next()` + lowercase → `lpt3.txt` et `com3.txt` REFUSÉS, à CHAQUE segment.
  CONIN$/superscripts fermés par l'alphabet ASCII. Cas corpus `reserved-device-name` PASS.
- **pack.rs:426 [P2 dim1 + P2 sécurité] — borne I-JSON contournable par syntaxe
  flottante** — STILL-OPEN, re-évalué **P3**. `enforce_bounds` (:487-505) couvre i64
  (unsigned_abs) ET u64 : tout entier exact > 2^53 est refusé (corpus deux signes PASS).
  MAIS un littéral > u64::MAX ou une forme exponentielle (1e16) est coercé en f64 par
  serde_json et tombe dans `else { Ok(()) }`. Conséquence sécurité r1 ÉTEINTE : plus aucun
  différentiel Rust↔TS (les deux bouts lisent le même double, parité 13/13 re-mesurée).
  Résidu : asymétrie d'orthographe acceptée + `digest_for`/`canonical_bytes` sans borne
  pour un appelant direct. À router au body (doc ou borne f64 non-entier, phase future).
- **pack.rs:852 [P2] — presentation_view « lossless »** — FIXED. Doc réécrite
  (:1072-1076) : « NOT an inverse mapping » ; « lossless » n'apparaît plus dans le fichier.
- **pack.rs:231 [P2] — digests hex majuscule acceptés** — FIXED. `is_lower_hex_digest`
  (:950-953) appliqué aux 6 digests, aux subjects (:745) et aux entrées lock (:774). Cas
  corpus `valid-signature-over-uppercase-digest` PASS. (Casse de la SIGNATURE d'enveloppe
  encore libre — hors payload signé, cf. NF-1.)
- **pack.rs:619 [P2] — unicité chemins lock sur chaîne brute, %2e%2e** — FIXED. Dedup
  case-insensitive (:762-773, suffisant car alphabet ASCII-only) ; segment finissant par
  `.` refusé (:876-880) ; `%` et espace HORS alphabet `[A-Za-z0-9._/-]` (:857-864). Cas
  corpus case-colliding + trailing-dot + confusable (Cyrillic U+0430) tous PASS.
- **pack.rs:294 [P2] — lock vide accepté** — FIXED. Refus explicite (:753-757) + cas
  corpus `valid-signature-over-empty-lock` PASS. Doc PackLock (:320-326) honnête : la
  vérification des bytes matérialisés appartient à l'extracteur.
- **pack.rs:519/:509 [P1+P2] — domaine en paramètre libre** — FIXED. API publique à
  domaine FIXE : `verify_signed_pack(raw, verifying_key)` délègue sous
  `PACK_SIGNING_DOMAIN` (:564-569) ; la variante injectable est PRIVÉE (:574).
  `cross_domain_signature_rejected` exerce les DEUX portes (:1559-1570).
- **pack.rs:700 [P3] — scanner : AWS Key ID et JWT passent** — FIXED. Préfixes `akia` et
  `eyj` ajoutés (:909-911) ; doc honnête (:891-900 : « NOT a guarantee... 64 hex chars is
  indistinguishable from a digest »).
- **pack_jcs_vectors.json:2 [P1+P3] — exponent forms / negative zero annoncés absents** —
  FIXED. Vecteurs `number-exponent-forms` (1e21/1e-7/0.5) et `negative-zero` (-0.0)
  présents (lignes 36-50), golden régénéré, parité Rust (41/41) ET TS (13/13, vitest
  relancé par le round) vertes. RFC 8785 : -0.0 → « 0 », noté au commentaire.
- **pack.rs:765 [P3] — échec de to_value avalé** — FIXED. Branche Err →
  `ScanFinding{path:'$', detail:'pack could not be scanned: {e}'}` (:976-984).
- **pack.rs:569 [P3] — clone 1 MiB + signature non bornée** — FIXED. Parse par référence
  (`deserialize(payload_value)` :631, plus aucun clone du payload sur le chemin verify) ;
  chaîne signature couverte par le cap du pas 1 (:610-611). Nit résiduel : une copie
  bornée ≤ 1 MiB via to_value dans canonical_bytes (:425).
- **pack.rs:531 [P1 sécurité] — clés JSON dupliquées refusées nulle part (A8)** — FIXED.
  `parse_rejecting_duplicate_keys` (:645-714) : visitor custom, refus à toute profondeur,
  enveloppe ET payload, AVANT toute signature. Récursion bornée par serde_json (128,
  défaut vérifié Cargo.toml racine :60) après le cap 1 MiB. Test dédié
  `duplicate_json_keys_are_refused_structurally` (:2024-2047) + 2 entrées corpus — PASS.
- **pack.rs:652 [P2 sécurité] — aliasing Win32 (point/espace final, casse)** — FIXED.
  Trailing dot refusé par segment ; espace exclu par l'alphabet ; casse fermée par le
  dedup insensible. Trois cas corpus signés exercent chacun, PASS.
- **pack.rs:343 [P2 sécurité] — « aucun secret matérialisé » plus large que la règle** —
  FIXED. Module-doc étape 6 re-scopé sur `secret_refs` + heuristique déclarée (:37-41,
  :891-900) ; la règle enforced couvre exactement ce que la prose dit désormais.
- **THREAT_MODEL.md:1698 [P2 sécurité] — frontière pack absente** —
  DOCUMENTED-WITH-REASON. Écart assumé #27 (option (a) du preflight A16) : report
  explicite au commit body nommant menaces non couvertes + phase propriétaire. CAVEAT :
  le commit n'existe pas encore — disposition acquise SEULEMENT si le body tient la
  promesse (cf. NF-2).
- **pack.rs:1158 [P3 sécurité] — chemin miroir TS inexistant** — FIXED. Les références
  pointent `web/src/workflow/__tests__/pack_digest.test.ts`, fichier vérifié sur disque
  (SPDX ligne 1) et exécuté (13/13, lit les DEUX fixtures crates/).

## V2 — Sémantique des tests (CONCERN)

- **pack.rs:37 [P1] — gate ne couvrait pas la propriété phare** — FIXED. `nextest list
  import_hostile` = exactement 5 tests dont `signature_before_parse`. Mutation confirmée
  par lecture : inverser (4)/(5) de `verify_signed_pack_under_domain` (:614-632) rougit
  CE seul test — toutes les entrées corpus brutes meurent aux étapes 1-3, tous les cas
  `valid_signature_*` portent une signature authentique.
- **pack.rs:1354 [P2] — assertion scanner tautologique** — STILL-OPEN. La règle
  d'entropie (:930-936) exige uppercase ET lowercase ET digit ; les digests du pack sont
  minuscule-obligatoire → la règle ne peut PAS tirer sur eux, exemption ou pas. Aucun test
  ne fait passer une chaîne 64-hex à casse MIXTE par `looks_like_materialised_secret`.
  Supprimer l'exemption `is_hex_digest` (:924) laisse toujours 41/41 vert.
- **pack.rs:428 [P2] — aucune borne testée À la valeur du cap** — FIXED.
  `bounds_admit_their_exact_cap` (:1587-1609) exerce les 4 bornes au cap exact ; mutation
  `>` → `>=` sur :471/:479/:491/:508/:523 rougit chacune. Résidus hors périmètre r1 :
  `MAX_PACK_JSON_BYTES` sur l'enveloppe brute et la borne des noms de membre (:533).
- **pack.rs:1200 [P2] — collision « démontrée » que le corps ne produit pas** — FIXED.
  Test renommé `canonical_bytes_rejects_explicit_nulls_with_path` (:1452-1471) ; la
  collision est MESURÉE dans `nested_non_finite_float_is_refused_not_collapsed`
  (:1477-1499, `assert_eq!(as_value["ratio"], Value::Null)`).
- **pack.rs:1249 [P2] — exhaustivité ALL non testée en direction AJOUT** — STILL-OPEN.
  Un 7e variant + 7e champ avec ALL oublié compile et laisse le test vert. ATTÉNUATION
  réelle : PackDigests pinné dans contracts.schema.json:308-345 + stub STRICT
  (contracts.rs:200-206) → détecté une fois ; mais après régénération légitime
  (UPDATE_SNAPSHOTS=1) l'omission redevient silencieuse. Aucun assert
  `ALL.len() == nombre de variants`.
- **pack_digest.test.ts:1 [P2] — parité sans garde de non-vacuité** — FIXED. Check (2c)
  de check-frontier-contracts.sh:186-198 : échec si le miroir est absent OU ne lit plus
  les 2 fixtures. Re-exécuté : EXIT=0 « pack-parity mirror ».
- **pack.rs:1119 [P2] — direction défavorable de la collision jamais couverte** — FIXED.
  `presentation_names_are_reserved_by_contract` démontre la collision (même digest avec et
  sans `width` porteur de sens) et la PINE comme contrat + résurgence dans
  `presentation_view`. Écart assumé #26.
- **pack.rs:1158 [P3] — fichier TS inexistant cité** — FIXED (cf. V1 ; gate (2c)
  verrouille l'existence).
- **pack.rs:1494 [P3] — hostile_corpus_is_not_vacuous doublon** —
  DOCUMENTED-WITH-REASON. Doc-comment (:1826-1838) : duplication INTENTIONNELLE — le
  contrôle d'acceptation doit tourner DANS le filtre du gate. La limite résiduelle est
  couverte par les `expect_detail_contains` discriminants (mauvaise variante → échec).
- **pack_hostile_corpus.json:84 [P3] — « before serde sees a byte » indistinguable** —
  FIXED. « why » honnête (:110) ; l'ordre reste porté par le code (check taille = première
  instruction, :579-585).
- **pack.rs:1595 [P3] — .filter sans justification** — FIXED. Exception NOMMÉE
  (`cross-domain-signature`, :1984-1995) + corpus l.2 ; `declared==covered` reste fermé
  pour toute autre entrée.
- **pack_digest.test.ts:107 [P3] — test TS assertait BLAKE3, pas le canonicaliseur** —
  FIXED. Nouveau test digère `{a,b,c}` vs `{c,a,b}` (ordres d'insertion distincts) et
  exige l'égalité — retirer le `.sort()` (ts:43) le rougit. + ts:100-106 chaîne canonique
  exacte pour une clé non-BMP. 13/13 re-mesuré.

## V3 — Preflight et livrables (CONCERN)

- **pack_digest.test.ts:1 [D4 P1] — SPDX absent** — FIXED. Ligne 1 =
  `// SPDX-License-Identifier: AGPL-3.0-or-later` ; `bash scripts/check-spdx.sh` relancé
  → « 388 files compliant », EXIT=0.
- **pack.rs:817 [D4 P1] — A12 non appliqué** — DOCUMENTED-WITH-REASON. Réservation de
  noms documentée + testée (cf. V1/V2) ; la prescription « ids explicites + tri total »
  vise `SBFBWorkflowGraph.v1` (stub jusqu'à Phase G) — impossible de pinner des node ids
  dans un schéma inexistant. Divergence assumée #26, défendable. RÉSIDU : Phase G devra
  respecter la réservation — à rappeler au body.
- **engine lib.rs:106 [D4 P2 + Dim6 P2 + B3(7)] — 7e surface census** — FIXED. Phrase
  réécrite « 26 since S83 Phase F's DOMAIN_WORKFLOW_PACK_V1 » ; les 7 surfaces B3 sont
  toutes vraies ; census verbatim re-mesuré = 26 == gel.
- **canonical.rs:338 [D4 P2 + Dim6 P1] — 1 famille DOMAIN au lieu de 4** —
  DOCUMENTED-WITH-REASON. Fond défendable (écart #24 : constantes sans consommateur =
  code mort + promesse future ; D8 rendu pour la famille livrée). MAIS re-mesuré :
  `grep RUN_EVENT|COMPILED_PLAN|PACK_INSTALL_RECEIPT` → 0 hit — le re-routage H/I/X
  n'existe NULLE PART dans l'arbre, il repose à 100% sur le commit body (cf. NF-2).
- **pack_jcs_vectors.json:2 [D4 P2 + Dim6 P2] — A4 partiel** — FIXED (cf. V1).
- **pack.rs:147 [D4 P2 + Dim6 P1 :922] — A10/A11 : aucun digest confronté** — FIXED.
  `validate_pack` (:782-790) RECOMPUTE `content` via `content_digest_of_lock` et refuse le
  mismatch (cas corpus PASS) ; doc PackDigests honnête (content RECOMPUTED,
  compiled_plan/evaluation/publication « SHAPE only », exhaustivité archive transférée à
  l'extracteur).
- **pack.rs:2 [D4 P2] — A14 : clauses §19.2 (7) et (9) ni explicitées ni routées** —
  STILL-OPEN. `grep republication|draft|\$ref` → seul hit = le « why » de
  envelope-no-signature (versant refus seulement). A14 absent des 23 correctifs ET des 4
  écarts assumés. À router au commit body (cf. NF-2).
- **pack_hostile_corpus.json:3 [D4 P2 + Dim6 P1] — A8/A13 : classes absentes** — FIXED.
  Dup keys structurel + 2 entrées corpus ; casse Windows + confusables + 22 DOS + trailing
  dot couverts ; SVG/Markdown ABSENTS PAR CONSTRUCTION et l'écart est ÉCRIT dans le corpus
  + module-doc §Media content (:59-66), conforme à A13. Ratio compression : N/A (aucun
  parseur zip dans le crate). import_hostile 5/5 PASS.
- **contracts.rs:193 [D4 P2] — A1 : stub fail-open** — FIXED. STRICT : panic si absent,
  assert si vide, écriture UNIQUEMENT sous UPDATE_SNAPSHOTS ; arbitrage A1 RENDU dans le
  doc-comment. PASS dans 41/41.
- **plan.md:595 [D4 P2] — périmètre F.2 réduit sans mandat** — DOCUMENTED-WITH-REASON.
  Limites LIVRÉES (5 bornes + tests au cap) ; layout réalisé en réservation (écart #26) ;
  rapport mapped/approximated/refused = écart #25 défendable (pour un pack natif seul
  « refused » existe, PackError EST ce rapport ; mapped/approximated présuppose un format
  ÉTRANGER = Phase AG). RÉSIDU non couvert : « licences » → NF-3.
- **pack.rs:1158 [D4 P3 + Dim6 P3] — chemins miroir TS** — FIXED (+ gate 2c).
- **pack.rs:1467 [D4 P3] — 4 entrées Unbounded interchangeables** — FIXED. Needles
  discriminants (« array of 513 entries » / « object of 513 members » / « string of 8193
  bytes » / « nesting deeper »), vérifiés :1801-1807.
- **pack.rs:724 [D4 P3] — exemption 64-hex** — DOCUMENTED-WITH-REASON. Borne écrite noir
  sur blanc, exemption motivée, préfixes ajoutés, scanner « Not authoritative, and not a
  gate » ; la règle enforced (secret_refs) reste vivante (cas corpus PASS).
- **pack.rs:72 [Dim6 P1] — 1 famille sur 4, aucun report écrit** —
  DOCUMENTED-WITH-REASON, conditionnel au body (cf. canonical.rs:338 et NF-2).
- **pack.rs:763 [Dim6 P1] — rapport mapped/approximated/refused absent** —
  DOCUMENTED-WITH-REASON (écart #25 ; date du scope cut DUE au body, cf. NF-2).
- **pack.rs:37 [Dim6 P2] — module-doc / gate** — FIXED (cf. V1/V2).
- **pack.rs:354 [Dim6 P2] — SignedPack type mort** — FIXED. `valid_envelope()`
  (:1241-1255) construit l'enveloppe acceptée À TRAVERS le type ; `valid_pack_round_trips`
  + `hostile_corpus_is_not_vacuous` passent par lui.
- **pack.rs:307 [Dim6 P2] — « layout/ stays outside » sans support** — FIXED. Phrase
  disparue de contracts.rs ; reformulation honnête au root doc (:338-340 : intention
  kickoff « realised as reserved member names »).

## V4 — Patterns et docs-contrat (CONCERN)

- **pack_digest.test.ts:1 [P1 dim5] — SPDX** — FIXED (re-mesuré EXIT=0).
- **engine lib.rs:106 [P1 dim5] — census 25→26** — FIXED. Fichier désormais en `M` dans
  git status ; phrase vraie, census re-grep = 26.
- **workflow-core lib.rs:83 [P3 dim5] — ligne vide brute dans le module-doc** — FIXED.
  Sections séparées par `//!` pleins ; première ligne vide = frontière légitime :93.
- **canonical.rs:343 [P3 dim5] — analogie kudos sur précédent mort** — FIXED sur la
  constante (:351-353, « payload type also lives outside this crate » — vrai :
  kudos_ledger.rs:70 en Rust vivant) + claim « one place » corrigée. Résiduel neuf : la
  ligne roster :60-61 « Python coordinator » reste morte → NF-8.
- **THREAT_MODEL:1695 [P3 dim5] — census sans marqueur temporel** — FIXED. Les 3
  occurrences ancrées « 25 à l'époque S83-B/C ; 26 depuis S83-F » — énoncés historiques
  vrais.
- **pack.rs:37 [P2 dim7] — module-doc / gate** — FIXED (5 `#[test]` dans le module,
  filtre nextest par sous-chaîne du nom qualifié).
- **pack.rs:1158 [P2/P3 dim7] — chemins TS** — FIXED (ancien chemin éradiqué, grep 0 hit).
- **pack.rs:1196 [P2 dim7] — aucun gate d'existence du miroir** — FIXED (check 2c,
  EXIT=0 re-mesuré). Résiduel neuf : commentaire « 38 crate tests » périmé → NF-7.
- **contracts.rs:192 [P2 dim7] — stub fail-open, arbitrage A1 non rendu** — FIXED
  (cf. V3).
- **pack.rs:56 [P3 dim7] — étiquette FRONTIER sans arête de provenance** — STILL-OPEN
  (moitié corrigée : tag désormais ADJACENT au type :347 ; moitié ouverte : aucune ligne
  de provenance façon ShardPlan `shard_plan.rs:189-190` ; grep `doctrine|gated by` sur
  pack.rs = 0 hit. Fix = 2 lignes de commentaire, non bloquant).
- **check-frontier-contracts.sh:190 [P3 dim7] — registre FRONTIER opt-in** — FIXED. Pin
  anti-suppression `// FRONTIER: SbfbPromptAgentPack` requis (miroir du pin ShardPlan) ;
  gate relancé EXIT=0 « [2 tagged] ».
- **WORKFLOW_AGENTS_OVERVIEW.md:6 [P3 dim7] — GUIDE « grandit à chaque phase »** —
  STILL-OPEN. git log re-mesuré : seul `f4b257c` (Phase A) a touché le fichier. La
  frontière pack ne DOIT PAS d'entrée GUIDE dès F (clôture = AJ1), mais le body de F DOIT
  déclarer le report (formule Phase D) — déclaration absente des écarts #24-27 ; la
  bannière est falsifiée par 5 phases consécutives. Route : 1 ligne au body + 1 mot à la
  bannière.

## V5 — Honnêteté des garanties (PASS)

- **pack.rs:34 [P1] — « no materialised secret » sur-affirmée** — FIXED (étape 6
  re-scopée « SHAPE of opaque references », :37-41 + :891-900).
- **pack.rs:294 [P1] — claim rug-pull fausse au périmètre du crate** — FIXED (doc
  :320-326 « ends at the lock » + recompute content réel, cas hostile PASS).
- **pack.rs:519 [P1] — domaine paramètre vs doc** — FIXED (« The domain is NOT a
  parameter » désormais vraie du code).
- **pack_jcs_vectors.json:2 [P1] — exponent/negative-zero** — FIXED (parité re-mesurée
  TS 13/13 + Rust).
- **canonical.rs:4 [P1] — « All signed payloads... » rendue fausse** — FIXED. Tête
  réécrite, 2 familles rebâties localement nommées (nexus-trace-core, sbfb-workflow-core
  packs) ; contre-vérifié : trace-core construit DOMAIN+0x00+json localement
  (signed.rs:61-62), hash.rs de l'engine est hashing-only, kudos passe bien par
  `canonical_bytes`.
- **pack.rs:27 [P2] — « on that Value » (enveloppe vs payload)** — FIXED (« enforced on
  the payload value », colle à :612).
- **canonical.rs:347 [P2] — « enumerable in one place »** — FIXED. Remplacé par « a
  reference list, not an exhaustive one » (keystore.rs:102, trace lib.rs:29 nommés) ; les
  6 préfixes `sbfb-pack-digest-*-v1` ne sont pas des domaines de SIGNATURE.
- **engine lib.rs:106 [P2] — census périmé** — FIXED (DOMAIN_CENSUS_FROZEN=26, gate
  EXIT=0, PATTERNS.md:4084 aligné).
- **pack.rs:1158 [P3] — chemin TS** — FIXED (lu ET exécuté, 13/13).
- **pack.rs:44 [P3] — (a) « everything that digests » ; (b) « sorted by the producer » +
  lock vide** — STILL-OPEN partiel. (a) FIXED (:51-57 re-scopé + parenthèse honnête sur
  blake3_hex) ; (b-lock-vide) FIXED (:753 + cas hostile). RÉSIDU : « paths unique and
  sorted by the producer » (:331) — `validate_pack` relu en entier : AUCUN check d'ordre,
  un lock non trié vérifie sans erreur. P3 : une ligne de doc ou un check à ajouter.

## V6 — Corpus et gates (PASS)

- **Miroir TS sans gate d'existence** — FIXED. Check (2c) : `[ -f ]` + grep des DEUX
  fixtures ; relancé EXIT=0 ; mutation simulée sur copie scratchpad (lectures supprimées)
  → grep FAIL, attrapé. Miroir re-mesuré 13/13.
- **SPDX** — FIXED (EXIT=0 re-mesuré).
- **4 classes F.2 absentes du corpus** — FIXED. Dup keys (2 entrées + visitor + test dédié
  avec contrôle d'acceptation) ; casse/trailing-dot/confusable joués BY NAME
  (:1938-1963) ; SVG/Markdown absents BY CONSTRUCTION, documenté au header corpus ; ratio
  compression sans objet (cap 1 MiB sur raw AVANT parse).
- **4 Unbounded interchangeables** — FIXED (needles distincts + 2 NonRepresentable à
  chemins distincts).
- **hostile_corpus_is_not_vacuous doublon** — DOCUMENTED-WITH-REASON (répétition
  intentionnelle DANS le filtre du gate ; `valid_pack_round_trips` n'est PAS sélectionné
  par le gate, vérifié au run).
- **« before serde sees a byte »** — FIXED (« why » honnête ; check taille à :580 avant
  :591).
- **.filter sans justification** — FIXED (exception nommée, jouée réellement par
  `cross_domain_signature_rejected`, égalité declared==covered bidirectionnelle pour les
  18 autres).
- **Registre FRONTIER opt-in** — FIXED (pin anti-suppression :179-184, « [2 tagged] »).
- **Module-doc / gate G-IMPORT-HOSTILE** — FIXED (5/5, commande d'enforcement EXACTE de
  gate_matrix.md:69 rejouée).

## Les 4 écarts assumés — jugement du round 2

1. **#24 — UNE famille DOMAIN (WORKFLOW_PACK) au lieu des 4 du plan.** DÉFENDABLE sur le
   fond : les 3 familles sans producteur/consommateur seraient du code mort + promesse
   future (la classe que PROMISE_RE combat) ; B1 exigeait une décision D8 par famille et
   la famille livrée a la sienne. MAIS le re-routage (COMPILED_PLAN→H, RUN_EVENT→I —
   routé vers F par le preflight E:144 —, PACK_INSTALL_RECEIPT→X) n'a AUCUNE trace dans
   l'arbre (grep = 0 hit) : il repose entièrement sur le commit body pas encore écrit.
   ACCEPTÉ CONDITIONNEL au body (NF-2).
2. **#25 — rapport mapped/approximated/refused non livré.** DÉFENDABLE : la taxonomie n'a
   de sémantique que pour l'import d'un format ÉTRANGER (Flowise/n8n = Phase AG) ; pour un
   pack natif la seule sortie est « refused » et `PackError` EST ce rapport (8 variantes
   terminales, path+detail). Scope cut À DATER au body (NF-2).
3. **#26 — A12 réalisé en RÉSERVATION de noms de présentation.** ACCEPTÉ : la
   prescription « ids explicites + tri total » vise le SCHÉMA de graphe (Phase G, stub) ;
   la v1 livre la réservation documentée comme contrat ET testée (collision démontrée et
   pinnée, résurgence en presentation_view). Rappel Phase G dû au body.
4. **#27 — THREAT_MODEL sans section nouvelle (option (a) du preflight A16).** ACCEPTÉ
   CONDITIONNEL : le report explicite (menaces non couvertes + phase propriétaire) doit
   figurer au body — il n'existe encore nulle part (NF-2).

## Findings neufs du round 2

**0 P0/P1. 2 P2. 11 P3 distincts** (doublons inter-groupes fusionnés).

- **NF-2 [P2]** `pack.rs:79` — **Cinq déclarations dont dépendent les écarts #24/#25/#27
  + A14 n'ont AUCUNE trace dans l'arbre** : elles reposent à 100% sur un commit body pas
  encore écrit. Si le body les omet, trois DOCUMENTED-WITH-REASON de ce round dégradent
  en scope cuts silencieux (la classe que le preflight interdisait). Le body DOIT porter :
  (1) routes DOMAIN_COMPILED_PLAN_V1→H, DOMAIN_RUN_EVENT_V1→I, DOMAIN_PACK_INSTALL_RECEIPT_V1→X ;
  (2) scope cut daté mapped/approximated/refused → AG ; (3) A14 clauses §19.2 (7) et (9)
  explicitées ou routées AG ; (4) report THREAT_MODEL A16 avec menaces + phases
  propriétaires ; (5) nombre de tests import_hostile mesuré (5) exigé par B4.
- **CG-4 [P2]** `pack_jcs_vectors.json:14` + `pack_digest.test.ts:100` — **Aucun vecteur
  de parité ne DISCRIMINE le tri UTF-16 du tri par code-point.** Le seul vecteur non-BMP
  est non-discriminant (l'emoji a le plus grand code-point ET la plus grande unité de
  tête). Le cas discriminant U+FFFF vs U+1F600 a été MESURÉ par le round : serde_jcs et
  la TS rendent tous deux `{"a":3,"😀":2,"￿":1}` — concordance UTF-16-correcte, donc PAS
  exploitable aujourd'hui, mais aucun vecteur committé ne la prouve. Blind spot du gate
  sur sa propriété TITRE. Fix : 1 vecteur + golden (régénération UPDATE_SNAPSHOTS).
- **NF-1 [P3]** `pack.rs:31` — nouvelle prose dup-key SUR-AFFIRMÉE : « two distinct byte
  streams must never verify as the same pack » est universellement fausse (whitespace,
  ordre payload/signature, hex_decode :1109-1121 accepte 2 casses de signature). La
  propriété fermée est la direction cross-parseur. Reformuler.
- **NF-4 [P3]** `pack.rs:920` — le commentaire justifiant l'exemption is_hex_digest est
  mesurablement FAUX (« fires on every BLAKE3 hex string — nine of them ») : la règle
  d'entropie exige un uppercase, un digest minuscule ne peut pas la déclencher. Rejoint
  le STILL-OPEN V2 :1354.
- **NF-5 [P3]** `accessibility_profile.rs:806` — le durcissement STRICT du stub (#14)
  rend périmée la prose sœur « a stricter variant than... schema_snapshot_stub » : les
  deux tests ont désormais la même strictesse. Même classe que la 7e surface census r1.
- **NF-6 [P3]** `pack.rs:1783` (+ :1425-1428, ts:90) — **planchers anti-shrink non
  rehaussés après les correctifs** : corpus assert >=14 vs 16 entrées ; parité >=7 (Rust
  ET TS) vs 9 vecteurs. Supprimer exactement les entrées ajoutées par les correctifs
  (2 dup-key, exponent-forms, negative-zero) laisserait les gardes vertes. Fix 1 ligne ×3
  (16/9/9).
- **NF-7 [P3]** `check-frontier-contracts.sh:188` — commentaire du check (2c) écrit par
  la passe de correctifs elle-même : « 38 crate tests » alors que la suite est à 41.
  Compteur périmé à la livraison. Fix : dé-chiffrer la phrase.
- **NF-8 [P3]** `canonical.rs:60` — ligne roster « DOMAIN_KUDOS_V1 — ... Python
  coordinator » morte depuis S50-51, et la passe de correctifs a AJOUTÉ 4 lignes plus bas
  une entrée qui invoque ce précédent sans rafraîchir la description (le consommateur
  vivant est kudos_ledger.rs:70, Rust, qui DÉPEND de nexus-core-rs).
- **NF-9 [P3]** `docs/claude/README.md:2178` — 8e surface census HORS liste B3 : « les 22
  des 25 familles DOMAIN_*_V1 sans schéma généré » au présent de vérité générale — « 25 »
  périmé (26, 4 schématisées), ni ancré ni rafraîchi, dans la source de vérité workflow.
  Fix : même traitement que THREAT_MODEL.
- **NF-10 [P3]** `pack_jcs_vectors.json:2` — « the pack pipeline refuses them (see
  enforce_bounds) » sur-affirmée pour la porte `digest_for` : digest_for → canonical_bytes
  n'applique QUE reject_nulls, jamais enforce_bounds — un document de graphe portant
  9007199254740993 se digère sans refus côté Rust (u64 exact) pendant qu'un consommateur
  JS lit ...992. Le bord relève du schéma de graphe Phase G — à router ou reformuler
  (« the ENVELOPE pipeline refuses them ») au body.
- **NF-11 [P3]** `check-frontier-contracts.sh:196` — le check (2c) « two fixture reads »
  est satisfiable par un commentaire mentionnant les noms (démontré sur copie
  scratchpad). Borne standard des gates grep du dépôt ; résiduel borné, pas de fix requis.
- **NF-12 [P3]** `pack_hostile_corpus.json:209` — le champ `expect: SignatureInvalid` de
  l'entrée cross-domain-signature est décoratif (filtrée de l'assertion, jamais
  machine-lue ; la propriété EST testée en dur ailleurs).
- **NF-13 [P3]** `pack_hostile_corpus.json:5` — résidu de discriminance : `not-json` sans
  pin (6 entrées MalformedEnvelope sans needle) ; `envelope-no-signature` et
  `envelope-signature-not-string` partagent le même needle ET le même chemin de code
  (:605-608 fusionne absence et mauvais type).

## Gaps de complétude (2 critiques dédiés)

- **CG-1 [P3]** = NF-9 (README.md:2178, surface census hors B3) — voir ci-dessus.
- **CG-2 [P3]** `docs/security/EXTERNAL_AUDIT_SCOPE.md:28-62` — le scope d'audit externe
  borne Ed25519+JCS à `nexus-core-rs` (§2.1) et « 6 canonical structures » (§2.2). La
  Phase F crée un DEUXIÈME site de vérification Ed25519 + une DEUXIÈME implémentation de
  pré-image JCS (`sbfb-workflow-core::pack`, serde_jcs + ed25519-dalek) — exactement la
  matière « domain separation correctness » du doc. Dérive PRÉ-EXISTANTE (doc daté
  2026-04-26, jamais maintenu) que F étend d'un cran qualitatif. À ROUTER (audit_plan ou
  body), pas à corriger in-phase.
- **CG-3 [P3]** — rappels writer/commit : (a) `phase-review-cross-check.yml:86` et le hook
  `phase-auditor-gate` exigent le fichier CANONIQUE
  `.planning/active/sprint83_phase_f_review.md` (copie byte-identique du dernier round,
  canon §4.5.8) — les fichiers rounds/ seuls font échouer le CI ; (b) deltas au body :
  Rust crate 38→41 (+3), workspace 2167→+3 attendu, Vitest web 529→+13 ; (c) les 4
  écarts assumés promettent tous une trace au body (cf. NF-2).
- **CG-4 [P2]** — vecteur de parité UTF-16 discriminant absent — voir section findings
  neufs.
- **CG-5 [P3]** `pack.rs:832-889` — `validate_lock_path` borne la longueur (8 Ko) mais
  PAS le nombre de segments : probe mesurée via `verify_signed_pack` → chemin de 3000
  segments `a/a/.../a` ACCEPT (dépasse MAX_PATH Windows ~260 à la matérialisation).
  Combiné à 512 entrées max, un lock accepté peut imposer 512 chemins pathologiquement
  profonds. La doc dit « narrow admissible shape » — pas vrai sur l'axe profondeur.
- **CG-6 [P3]** `pack.rs:811-815/:1153-1159/:421-435` — les helpers PUBLICS
  `content_digest_of_lock`/`digest_for`/`canonical_bytes` n'appliquent aucune borne
  d'entier : probe mesurée `canonical_bytes(json!({"n": 9007199254740993u64}))` produit
  `{"n":9007199254740992}` (arrondi silencieux). Le chemin verify est protégé par
  enforce_bounds ; le risque résiduel est limité aux appels directs. Rejoint NF-10.

## Attaques tentées qui ont ÉCHOUÉ (preuve de couverture)

- Gate frontier : `bash scripts/check-frontier-contracts.sh` → EXIT=0, « anti-promise +
  frontier-tag coverage [2 tagged] + DOMAIN census [26 frozen] + ... + pack-parity
  mirror » ; grep census rejoué indépendamment = 26 exactement ; arithmétique 26/4/22
  cohérente gate ↔ PATTERNS.md §P70.
- SPDX : `bash scripts/check-spdx.sh` → EXIT=0 (fichier TS neuf conforme ligne 1).
- Miroir TS réellement exécuté : `npx vitest run src/workflow/__tests__/pack_digest.test.ts`
  → 13/13 PASS. Glob vitest `src/**/*.{test,spec}` le ramasse sans édition de config ;
  les 2 CI lancent depuis `web/` donc le chemin fixture `../crates/...` résout partout.
- `npx tsc --noEmit -p tsconfig.app.json` → EXIT=0 ; `npx eslint` sur le fichier neuf →
  EXIT=0 (override test globals).
- Phantom-dep @noble/hashes : SEUL le fichier de test l'importe ; devDependency dans
  package.json ET lock ; hors graphe d'import de l'app → build/size-limit non impactés ;
  playwright testDir=./e2e ne le ramasse pas.
- Supply-chain : Cargo.lock diff = +2 arêtes (ed25519-dalek 2.2.0, serde_jcs) sur des
  nœuds DÉJÀ présents — zéro nouveau nœud de licence.
- Suite crate : `cargo nextest run -p sbfb-workflow-core --offline` → 41/41 PASS 0 skip.
- Consommateur réel du gate : la commande EXACTE de gate_matrix.md:69
  (`cargo nextest run -p sbfb-workflow-core import_hostile`) rejouée → 5 tests dont
  `signature_before_parse` (plus EXIT=4).
- Invariant registre `len == 7` chez ses 6 citeurs : contracts.rs remplit l'identité par
  `pub use crate::pack::SbfbPromptAgentPack` — toutes les proses « len == 7 » restent
  VRAIES ; assert contracts.rs:149 inchangé.
- Proses census transitives : 7 surfaces B3 rafraîchies relues ; AGENT_SYSTEM.md ne cite
  aucun compte (grep vide) ; WORKFLOW_AGENTS_OVERVIEW.md:70-73 reste vrai ;
  CLAUDE.md/SPRINT_LOG = énoncés historiques ancrés.
- Hook lightcheck Check 0 : 0 trailing-whitespace + newline final sur les 6 fichiers
  neufs ; `.gitattributes` couvre les rounds (`*_review_r*.md -whitespace`, posé par
  6f95cd2).
- Mécanique rounds/ : lightcheck Check 7, phase-auditor-gate.sh, agentctl, lint-planning
  et phase-review-cross-check.yml construisent tous le chemin canonique par concaténation
  (jamais par glob) — précédent vivant : rounds Phase E committés.
- Lecteurs de contracts.schema.json : le SEUL consommateur est `schema_snapshot_stub`,
  désormais STRICT (fail-open r1 fermé).
- `bash scripts/scan-en-strings.sh` → « src/ is French-only, clean » EXIT=0.
- Câblage CI du gate modifié : ci.yml:136, .woodpecker/ci-linux.yml:92,
  scripts/verify.sh:98-99 ; le check (2c) n'utilise que `[ -f ]` + `grep -qF`
  (BusyBox-safe).
- Test miroir cross-crate `pack_signing_domain_mirrors_canonical_rs` (:1274-1300) : lit
  canonical.rs via CARGO_MANIFEST_DIR, échoue bruyamment si la constante bouge.
- ATTAQUE récursion-avant-bornes : payloads imbriqués 200..100000 de profondeur →
  tous `MalformedEnvelope("recursion limit exceeded")` — serde_json (128) mord AU PARSE,
  avant tout stack overflow. Aucun crash.
- ATTAQUE lock réordonné : `content_digest_of_lock({A,B})` ≠ `({B,A})` mesuré (JCS
  préserve l'ordre des tableaux, l'ordre est signé). Pas de collision d'ordre.
- ATTAQUE dup-key en profondeur (dans un tableau du payload) →
  `MalformedEnvelope("duplicate object key 'path'")` : le visitor refuse à TOUTE
  profondeur.
- ATTAQUE différentiel de parseur 2^53+1 : serde_jcs ET la TS rendent tous deux
  `9007199254740992` — aucun différentiel exploitable à la couche JCS ; le chemin verify
  rejette de toute façon (Unbounded).
- ATTAQUE suppression du miroir TS : le gate (2c) rougit désormais (branche `[ ! -f ]`).
- ATTAQUE chemins passant l'alphabet : fullwidth-solidus U+FF0F, espace, NUL, `$`
  (CONIN$) → tous rejetés par l'alphabet ; `nul.txt`/`COM1`/`aux.h` → rejetés device ;
  `workflow./a.json` → rejeté trailing-dot. Les chemins acceptés (`-rf`, `.git/config`,
  `com0/x`) sont légitimes ou shape-only documentés — pas de traversée ni d'aliasing
  Win32 exploitable à ce crate.

## Suites §7.4 — état OBSERVÉ (jamais « vert » sans observation)

Mesuré par ce round ou par la passe de correctifs et RE-MESURÉ ici :
- crate `sbfb-workflow-core` : **41/41 PASS** (38→41, +3 : bounds-cap, presentation
  reservation, dup-key) ;
- gate `import_hostile` : **5 tests, 5/5 PASS** (commande d'enforcement exacte) ;
- `check-frontier-contracts.sh` **EXIT=0** ; `check-spdx.sh` **EXIT=0** (388 fichiers) ;
- miroir TS **13/13 PASS** (vitest run) ; tsc + eslint EXIT=0 sur le fichier neuf ;
- mutations re-jouées post-correctifs : inversion signature/parse → SEUL
  `import_hostile::signature_before_parse` rouge ; garde dup-key neutralisée → 2 rouges ;
  binding content retiré → 1 rouge ; arbre restauré 41/41.

**EN COURS au lancement de ce round, NON observés verts — à attendre avant commit** :
- `cargo nextest run --workspace --locked` Windows complet (fmt/clippy/doctests/release) ;
- pipeline Docker canonique `sbfb-ci` (rust:1.94, avec `SBFB_TEST_HTTP_TIMEOUT_SECS=120`) ;
- suite web complète (`npm run test:unit` 529+13 attendus, coverage, build, size).

## Verdict: PASS-PENDING

0 P0/P1 de r1 encore ouvert (19/19 disposés avec évidence re-mesurée) ; 0 P0/P1 neuf.
Restent 7 findings r1 STILL-OPEN (tous P2 re-évalués P3 ou P3) et 2 P2 neufs (NF-2 corps
de commit porteur de 5 déclarations obligatoires ; CG-4 vecteur UTF-16 discriminant),
plus 11 P3 neufs — aucun ne bloque le format livré, tous exigent une trace. Codex n'a
pas encore tourné.

Ordre des actions avant commit :

1. **Attendre et OBSERVER les 3 suites en cours** (workspace Windows complet, Docker
   canonique, web complet) — aucun commit sans les trois verts constatés.
2. **Quick-fixes recommandés avant Codex** (chacun ≤ 3 lignes, ferment 5 P3 neufs) :
   planchers 16/9/9 (NF-6) ; « 38 crate tests » → sans nombre (NF-7) ; ligne roster
   kudos/Python (NF-8) ; prose « two distinct byte streams » (NF-1) ; commentaire
   exemption is_hex_digest (NF-4) + prose « stricter variant » (NF-5). Optionnel mais
   fort : vecteur UTF-16 discriminant (CG-4, 1 vecteur + golden) et arête de provenance
   FRONTIER (2 lignes).
3. **Codex rounds numérotés** (`codex exec -m gpt-5.6-sol -c model_reasoning_effort=max`),
   artefacts `rounds/sprint83_phase_f_codex_r{N}.md` bruts jamais réécrits, boucle jusqu'à
   « CLEAN ou P2/P3 documentés ».
4. **Commit body 9 sections** portant OBLIGATOIREMENT (sinon 3 DOCUMENTED-WITH-REASON de
   ce round dégradent en scope cuts silencieux, NF-2) : routes DOMAIN COMPILED_PLAN→H /
   RUN_EVENT→I / PACK_INSTALL_RECEIPT→X ; scope cut daté mapped/approximated/refused→AG ;
   A14 §19.2 (7)+(9) explicitées ou routées AG ; report THREAT_MODEL A16 (menaces +
   phases propriétaires) ; rappel réservation présentation → contrainte Phase G ; report
   GUIDE (formule Phase D) ; scope cut « licences » (NF-3) ; deltas tests mesurés (crate
   38→41, import_hostile 5, TS 13, web +13) ; STILL-OPEN routés (scanner/exemption,
   ALL.len, sorted-by-producer, digest_for sans borne, lock-depth, EXTERNAL_AUDIT_SCOPE).
5. **Mettre à jour la copie canonique** si un round ultérieur (Codex) modifie l'état —
   `.planning/active/sprint83_phase_f_review.md` reste la copie byte-identique du DERNIER
   round (canon §4.5.8), condition du hook phase-auditor-gate et du CI cross-check.
