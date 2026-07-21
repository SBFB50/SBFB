# Sprint 83 Phase F — Review round 3 (dispositions des 16 findings Codex r3)

**Sprint** : 83 — chantier « Workflow Agents ». **Phase** : F — Format portable et signé
(`SBFBPromptAgentPack.v1`). **Date du round** : 2026-07-21. **HEAD** : `6f95cd2`, arbre
sale (phase non committée).
**Méthode / historique** : r1 = 8 dimensions de review, 79 findings bruts, passe
adversariale tronquée par limite de modèle (`rounds/sprint83_phase_f_review_r1.md`).
r2 = 9 agents, 87 dispositions, PASS-PENDING (`rounds/sprint83_phase_f_review_r2.md`).
Codex externe r3 (GPT-5.6 Sol, audit rendu sur BUNDLE inline — son sandbox shell est mort
post-reboot) : **BLOQUÉ**, 0 P0 + 5 P1 + 8 P2 + 3 P3
(`rounds/sprint83_phase_f_codex_review_r3.md`, artefact brut jamais réécrit). Une passe de
CORRECTIFS post-Codex a été appliquée le 2026-07-21. Round 3 = **5 agents** : 4 groupes de
vérification des correctifs (R3-P1-code, R3-P1-tests, R3-P1-security, R3-prose-routes)
+ 1 synthétiseur-writer qui a lui-même re-mesuré P2-7, P3-3 et le P1 neuf de collation.
Chaque disposition porte une évidence RE-MESURÉE (fichier:ligne relu, commande relancée) —
jamais la parole de la passe de correctifs.

Verdicts par groupe : R3-P1-code **PASS**, R3-P1-tests **PASS**, R3-P1-security
**CONCERN**, R3-prose-routes **CONCERN**.

## Bilan de disposition

Les 16 findings Codex r3 sont tous disposés : **14 FIXED**, **2 DOCUMENTED-WITH-REASON**
(P2-3 : moitié corrigée en code, moitié documentée honnêtement ; P3-3 : refus confirmé
mais avec rationale CORRIGÉE par le writer), **0 STILL-OPEN**, **0 WAS-WRONG**. Les
**5 P1 bloquants de Codex sont tous FIXED** avec évidence re-mesurée. S'y ajoute r2 NF-2
(déclarations obligatoires du commit body) : STILL-OPEN par construction (le commit
n'existe pas encore), liste REFORMULÉE en section dédiée — les routes DOMAIN de r2 sont
CADUQUES depuis que les 4 constantes existent in-tree.

MAIS la vérification adversariale des correctifs a produit **11 findings neufs : 1 P1
+ 2 P2 + 8 P3**. Le P1 (NF-r3-1) est un défaut INTRODUIT par le correctif P1-4/P2-6 :
la liste nominale gelée du census est triée en collation en_US alors que le pipeline live
trie en byte-lex sous les locales CI Linux — gate frontier rouge DÉTERMINISTE en CI
pendant qu'il est vert en local. Re-mesuré par le writer : `LC_ALL=C bash
scripts/check-frontier-contracts.sh` → « census drift », diff `12d11` / `13a13`
(`DOMAIN_KEY_ROTATION_V1` / `DOMAIN_KEYSTORE_V1`), tandis que le run en locale par défaut
rend EXIT=0. Correctif à 2 lignes AVANT Codex r4.

| Finding Codex r3 | Sévérité | Groupe | Disposition |
|---|---|---|---|
| P1-1 racine « carries » plus forte que le type + PackDigests | P1 | prose-routes | FIXED |
| P1-2 collision réelle `presentation_view` | P1 | code | FIXED (résidu prose → NF-r3-5 P3) |
| P1-3 tests post-signature verts via fallback content-mismatch | P1 | tests | FIXED |
| P1-4 cut des 3 domaines contredit D2 gelé | P1 | code + prose-routes | FIXED (mais NF-r3-1 P1 introduit) |
| P1-5 garanties secrets/rendu fausses + fuite dans l'erreur | P1 | security | FIXED (résidus → NF-r3-2/NF-r3-3 P2) |
| P2-1 miroir « ne peut pas dériver » = fausse preuve | P2 | security | FIXED |
| P2-2 helpers publics sans `enforce_bounds` (collision 2^53) | P2 | code | FIXED |
| P2-3 bornes après matérialisation du DOM + signature non bornée | P2 | security | DOCUMENTED-WITH-REASON |
| P2-4 chemins non matérialisables, tailles sans plafond | P2 | security | FIXED (couverture → NF-r3-7 P3) |
| P2-5 parité ne compare que BLAKE3 ; pas de lone-surrogate | P2 | tests | FIXED |
| P2-6 census gèle un nombre, pas le registre | P2 | prose-routes | FIXED (mais NF-r3-1 P1 introduit) |
| P2-7 enveloppe signée hors registre de schémas | P2 | writer | FIXED |
| P2-8 fausse attribution `canonical.rs` | P2 | security | FIXED (miroir → NF-r3-8 P3) |
| P3-1 ordre du lock documenté mais jamais vérifié | P3 | tests | FIXED |
| P3-2 unicité corpus non vérifiée ; expect décoratif | P3 | tests | FIXED |
| P3-3 `digest_kinds` ne détecte pas un variant oublié dans `ALL` | P3 | writer | DOCUMENTED-WITH-REASON (rationale corrigée) |

## R3-P1-code (PASS)

- **Codex-r3-P1-2 — collision `presentation_view`** — FIXED. pack.rs:1221-1255 relu :
  chemins encodés en TABLEAUX JSON de segments (clés objet = String, indices = entiers)
  via `serde_json::to_string` — injectif car la sérialisation JSON est inversible par
  parsing (deux séquences distinctes ne peuvent produire le même texte ; `["a","b"]` !=
  `["a.b"]`, `["0"]` != `[0]`) et les chemins d'un même document sont uniques donc aucun
  écrasement `Map::insert`. Test `presentation_view_is_injective_on_ambiguous_keys`
  pack.rs:1859-1877 pinne la paire EXACTE de Codex (`{"a":{"b":{"x":1}},"a.b":{"x":2}}`
  vs `999`) : vue != vue, 2 membres survivants assertés, digest != digest.
  `cargo nextest -p sbfb-workflow-core` re-lancé : 44/44 PASS dont ce test. Recherche
  adversariale d'un nouveau contre-exemple : aucun trouvé (reconstruction unique du doc
  depuis le couple vue-sémantique/vue-présentation ; `collect_presentation` ne traverse
  que des clés non-présentation, les arrays gardent leur longueur dans les deux vues).
  Résidu prose du point secondaire de Codex (« by stable id » suggérant une invariance à
  l'ordre) → NF-r3-5 (P3).
- **Codex-r3-P1-4 — cut des 3 domaines D2 (volet code)** — FIXED. canonical.rs:390
  `DOMAIN_COMPILED_PLAN_V1`, :399 `DOMAIN_RUN_EVENT_V1`, :408
  `DOMAIN_PACK_INSTALL_RECEIPT_V1` relus — pattern `DOMAIN_KUDOS_V1` (tag réservé,
  payload ailleurs), doc-comments au présent (« Nothing signs under this tag yet »),
  aucune promesse ancrée phase/sprint. Gate re-lancé : `bash
  scripts/check-frontier-contracts.sh` EXIT=0, sortie « DOMAIN census [29 frozen] » —
  PROMISE_RE self-test + scan crates/ (canonical.rs inclus) propres. Census gelé par
  LISTE NOMINALE triée de 29 noms (check-frontier-contracts.sh:218-246), comparaison
  live par égalité de chaîne + diff sur drift (ferme P2-6). Compte re-mesuré : 27
  `pub const DOMAIN_` dans canonical.rs = prose « 27 + KEYSTORE + TRACE_EVENT ».
  Surfaces relues cohérentes : PATTERNS.md:4068-4083 (29 = 27+2, 4 schématisées,
  25 = 22+3, « 25 → 29 »), contracts.rs:25 (« 25 to 29 »), workflow-core lib.rs:55,
  engine lib.rs:106 (« 29 since S83 Phase F »), THREAT_MODEL.md:1695-1696 + 1961
  (« 25 à l'époque S83-B ; 29 depuis S83-F »), README.md:2178 (« 22 sur 25 à l'époque
  S79 ; 25 sur 29 depuis S83-F » — historique daté, honnête). Grep résiduel
  « 26 »/« 25 to 26 » sur crates+docs : uniquement dans `.planning/active/rounds` et
  `sprint83_phase_f_review.md` (artefacts de rounds historiques jamais écrasés, canon
  §4.5.8) — aucune surface vivante. Une inexactitude INTRODUITE au roster module
  canonical.rs:77-82 (identité install-receipt inexistante dans contracts.rs) →
  NF-r3-4 (P3).
- **Codex-r3-P2-2 — helpers publics sans `enforce_bounds`** — FIXED. `reject_nulls`
  pack.rs:489-522 relu : le bras Number refuse |i| > 9_007_199_254_740_991 via
  `unsigned_abs` (pas de panic `i64::MIN`) et u64 > borne ; floats non concernés — le
  collapse flottant a lieu au PARSE (une seule valeur parsée), caveat documenté
  honnêtement pack.rs:452-455. Test `canonical_door_refuses_the_measured_integer_collapse`
  pack.rs:1884-1911 : DÉMONTRE d'abord le collapse mesuré
  (`serde_jcs::to_string(2^53+1) == to_string(2^53)`) puis asserte le refus des DEUX
  valeurs avec message nommant « I-JSON », et l'admission de la borne exacte — PASS
  re-mesuré dans les 44/44. Aucun chemin de digest ne contourne la porte : grep
  `serde_jcs::` dans pack.rs = ligne 468 (dans `canonical_bytes`) + 2 démonstrations de
  test volontairement « unguarded » (1626, 1891-2) ; `digest_for`:1300-1306 →
  `canonical_bytes`, `content_digest_of_lock`:910-914 → `digest_for`, pré-image
  signature :678 → `canonical_bytes` ; `blake3_hex`/`hex_encode` = octets bruts jamais
  JSON (doc module :57-59) ; `web/src/workflow/` ne contient que le miroir de test
  `pack_digest.test.ts`, aucun chemin TS de production. Cohérence pipeline conservée :
  `enforce_bounds` (étape 3) fire `Unbounded` pour les entiers d'enveloppe AVANT la
  porte — routage corpus intact (2 entrées entiers → `Unbounded`). serde_json 1.0.149
  sans `arbitrary_precision` (grep Cargo.toml/lock vide) donc aucun grand entier ne peut
  atteindre le bras `(None,None)`.

## R3-P1-tests (PASS)

- **Codex-r3-P1-3 — tests post-signature verts via fallback content-mismatch** — FIXED.
  pack.rs:2145-2369 relu en entier. `rebind_content` (2148-2152) recalcule
  `digests.content` via `content_digest_of_lock` après CHAQUE mutation de lock : 7 cas
  de chemin (2215), duplicate-lock-path (2225), case-colliding (2239), unsorted (2249),
  oversized-file (2258), empty-lock (2267). SEUL le cas mismatched-content-digest ne
  rebinde pas (2274-2281, le mismatch EST le sujet). Les cas syntaxe-digest mutent
  `graph_semantic`, pas `content` (2314-2330). Chaque cas asserte un fragment
  DISCRIMINANT vérifié contre les messages réels : 'backslash' == pack.rs:950, 'not
  sorted byte-lexicographically' == 872, 'per-file cap' == 852, 'duplicate lock path
  (case-insensitive)' == 840, 'pins no file' == 822, 'opaque logical reference (value
  withheld)' == 894, 'relative segment' == 984, 'ends with `.`' == 989, 'outside the
  lock-path alphabet' == 963, 'reserved device name' == 995, 'drive or stream' == 955,
  'graph_semantic' == 289-291. Discriminance re-raisonnée : retirer la règle backslash
  route vers le message alphabet (963) qui ne contient pas 'backslash' → assert fragment
  ROUGE ; retirer le tri (869-875) rend le cas unsorted rebindé ACCEPTÉ → panic 2354.
  Le set declared==covered (2337-2349) force chaque ligne `post_signature_entries` à
  être exercée PAR NOM. Re-mesuré : `cargo nextest run -p sbfb-workflow-core --locked`
  = 44/44 PASS.
- **Codex-r3-P3-1 — ordre du lock jamais vérifié** — FIXED. pack.rs:869-875 : tri
  byte-lexicographique STRICT enforce dans `validate_pack` (comparaison `as_bytes()`
  avec `>=`, message 'lock entries are not sorted byte-lexicographically'). Fixture
  `valid_pack` triée : 'prompts/reviewer.md' < 'workflow/workflow.json'
  (pack.rs:1334-1343, 0x70 < 0x77). Cas unsorted-lock présent (2245-2252, `swap(0,1)` +
  `rebind_content` pour que seul le tri puisse refuser). Ordre dup-avant-tri vérifié :
  le check duplicate est DANS la boucle per-entry (835-843), le check tri est APRÈS la
  boucle (869), commenté explicitement (865-868 'Checked AFTER per-entry rules so a
  duplicate dies as a duplicate, not as an ordering error') et pinné par le cas
  case-colliding (2230-2242) dont l'entrée ajoutée est aussi mal triée mais meurt en
  'duplicate lock path (case-insensitive)'.
- **Codex-r3-P3-2 — unicité corpus + expect décoratif** — FIXED. pack.rs:2067-2076 :
  unicité assertée via UN SEUL `BTreeSet` couvrant les DEUX sections (entries +
  post_signature_entries), donc les doublons intra- ET cross-section sont refusés
  ('duplicate corpus name'). Le floor `entries.len() >= 17` (2063) ne peut plus être
  rembourré. Expect cross-domain lu MÉCANIQUEMENT : `cross_domain_signature_rejected`
  (pack.rs:1719-1733) charge `pack_hostile_corpus.json`, trouve la ligne
  'cross-domain-signature', extrait son champ `expect` et l'asserte ==
  'SignatureInvalid', puis les deux portes (injectable 1746-1752 et publique 1755-1757)
  sont assertées `SignatureInvalid`. Toute dérive de la ligne corpus rend le test rouge.
  Les 20 autres lignes post_signature sont liées par l'égalité de sets declared==covered
  (2337-2349). Re-mesuré 44/44.
- **Codex-r3-P2-5 — parité BLAKE3-seule + lone-surrogate absent** — FIXED. Golden
  `pack_jcs_digests.golden.json` relu : 10 vecteurs portant CHACUN 'canonical' (texte
  JCS) ET 'digest'. Rust (pack.rs:1578-1612) extrait le texte JCS de la pré-image
  (`bytes[domain.len()+1..]`, 1585-1587), construit {digest, canonical} par vecteur et
  asserte le rendu COMPLET contre le snapshot (1608-1612) — donc les deux champs. TS
  (pack_digest.test.ts:97-105) asserte le texte canonique D'ABORD (ligne 101) puis le
  digest (102-103), par vecteur, plus la complétude bidirectionnelle vectors==golden
  (91-95) et le floor >=10. Lone-surrogate : réellement GÉNÉRÉ (pack.rs:2017-2022, bytes
  bruts contenant l'escape `\ud800` — le fichier corpus reste parseable, `corpus()` l'a
  parsé), ligne corpus 'lone-surrogate-escape' avec raw_generator 'lone_surrogate 0' et
  expect `MalformedEnvelope`, exercée par `hostile_corpus_refused`. Le refus tombe au
  parser JSON (un lone surrogate ne peut pas devenir une `serde_json::Value` donc
  n'atteint jamais la couche I-JSON) — couche honnête, documentée dans le 'why' de la
  ligne. Re-mesuré : nextest 44/44 + vitest pack_digest 14/14 PASS. Note conforme au
  r3 : NFC/NFD restent non-normalisés (distincts), aucune claim contraire n'a été
  ajoutée.

## R3-P1-security (CONCERN)

- **Codex-r3-P1-5 — secrets/rendu + fuite dans l'erreur** — FIXED. pack.rs:888-897 —
  l'erreur `secret_refs` porte l'index + « value withheld », jamais la valeur.
  `redact_userinfo` (pack.rs:1144-1161) appliqué au finding endpoint (pack.rs:1123) ;
  vérifié empiriquement (copie verbatim compilée, 10 cas) :
  'https://user:pass@h/p'→'https://REDACTED@h/p', 'http://h/a@b' inchangé (@ de chemin,
  `authority_end` borné au premier /?#), multi-@, port, query, multibyte OK, aucun panic
  de slicing. Prose honnête : « Never a secret »/« nothing to render » absents du code
  (grep repo — seules citations dans la review r3) ; doc `secret_refs` pack.rs:410-416
  admet 64-hex/split/champs non scannés ; section Media pack.rs:61-70 = métadonnées
  hostiles, échappement à l'UI ; doc scanner pack.rs:1078-1087 « Not authoritative, and
  not a gate ». Résidus = NF-r3-2 et NF-r3-3 (P2).
- **Codex-r3-P2-3 — bornes post-DOM + signature non bornée** — DOCUMENTED-WITH-REASON.
  Moitié FIXED : `signature_hex.len()!=128` vérifié AVANT `hex_decode`
  (pack.rs:679-686, re-lu). Moitié documentée : les bornes
  profondeur/cardinalité/longueur restent post-matérialisation du DOM, mais
  `MAX_PACK_JSON_BYTES` se déclare désormais explicitement borne honnête d'amplification
  mémoire du parse structurel (pack.rs:97-103) et `MAX_STRING_LEN` restreint honnêtement
  sa claim au PAYLOAD, signature et membres refusés bornés par le cap 1 MiB seul
  (pack.rs:108-110, écho au step-3 pack.rs:671-672). Amplification bornée 1 MiB ×
  constante — pas de DoS non borné, raison recevable.
- **Codex-r3-P2-4 — plafonds de chemins et de tailles** — FIXED. 5 caps déclarés
  pack.rs:113-125 (`MAX_LOCK_PATH_BYTES`=512, `MAX_PATH_SEGMENTS`=32,
  `MAX_SEGMENT_BYTES`=255, `MAX_FILE_SIZE_BYTES`=256 MiB, `MAX_TOTAL_SIZE_BYTES`=2 GiB) ;
  enforcement pack.rs:940/966/977 (chemin) + :850 (per-file) + :856-863 (total via
  `checked_add().filter()` — l'overflow u64 devient une erreur Semantic, pas un wrap).
  Cas corpus valid-signature-over-oversized-file présent (corpus json:183) et exercé PAR
  NOM (pack.rs:2254-2261 + garde d'exhaustivité :2336-2349). Un pack légitime de 512
  fichiers passe encore : `enforce_bounds` admet exactement `MAX_COLLECTION_LEN`=512
  (refus à cap+1, pack.rs:569 ; admission au cap prouvée par
  `bounds_admit_their_exact_cap` pack.rs:1774-1796) et `validate_pack` n'impose aucune
  cardinalité — seule contrainte le total ≤ 2 GiB. Suite 44/44 re-mesurée. Résidu
  test-coverage = NF-r3-7 (P3).
- **Codex-r3-P2-1 — test miroir durci** — FIXED.
  `pack_signing_domain_mirrors_canonical_rs` (pack.rs:1422-1457, re-lu + PASS au run
  re-mesuré) : occurrence EXACTEMENT unique du needle
  (`source.matches().count()==1` — une copie commentée ferait 2 → rouge), ligne de
  déclaration vérifiée non commentée (`trim_start().starts_with("pub const")`,
  :1441-1445), comparaison sur les BYTES du littéral. Prose honnête : pack.rs:90-94 « A
  lexical check, not a compiler link » ; PATTERNS.md:4082-4090 documente la borne
  honnête (le grep du census compte encore une const commentée — classe laissée à la
  review adversariale). Cosmétique : run d'espaces parasite dans le message d'assertion
  pack.rs:1435 (NF-r3-6).
- **Codex-r3-P2-8 — fausse attribution `canonical.rs`** — FIXED. canonical.rs:372-377
  (re-lu) : « **Honesty:** a valid signature proves that the holder of one Ed25519 key
  signed exactly these canonical bytes under this domain — not an identity, not a
  publication event, and nothing about the trustworthiness of the prompts inside ». La
  formulation « who published » a disparu ; le texte va plus loin que le module pack
  (« signed », « not a publication event »). Micro-incohérence résiduelle : pack.rs:13
  garde le verbe « published » = NF-r3-8 (P3).

## R3-prose-routes (CONCERN)

- **Codex-r3-P1-1 — volet prose racine + PackDigests** — FIXED. Relu pack.rs:363-370 :
  la doc du struct racine dit désormais « NAMES AND PINS — by digest, never by
  embedding » suivie IMMÉDIATEMENT de « Documented v1 limitation: the root does not yet
  REQUIRE any particular artefact set in its lock (no role→path rule says "a workflow
  graph must be pinned") ... a syntactically valid pack may pin any file set » —
  l'ancien « Carries the workflow graph... » de contracts.rs est remplacé (diff
  contracts.rs:40-47 relu). La doc module (pack.rs:2-9) dit « names a workflow, pins
  every one of its files by BLAKE3 digest ». La limitation ne nomme AUCUN token
  phase/sprint (routing « belong to the graph/agent contracts », pas de promesse
  datée) : anti-promise gate relancé, `bash scripts/check-frontier-contracts.sh`
  EXIT=0. Le split PackDigests est présent (pack.rs:235-244) : content RECOMPUTED par le
  vérificateur, graph_semantic/presentation « recomputable » via les vues,
  compiled_plan/evaluation/publication « SHAPE only ». Résidu P3 sur la tri-partition
  (NF-r3-10).
- **Codex-r3-P1-4 + P2-6 — domaines D2 + census nominal (volet prose/gate)** — FIXED.
  Relu canonical.rs : `DOMAIN_WORKFLOW_PACK_V1` :378, `DOMAIN_COMPILED_PLAN_V1` :390,
  `DOMAIN_RUN_EVENT_V1` :399, `DOMAIN_PACK_INSTALL_RECEIPT_V1` :408 — pattern
  `DOMAIN_KUDOS_V1` explicite, doc-comments « RESERVED tag... Nothing signs under this
  tag yet », routage sans lettre de phase (compatible PROMISE_RE, gate EXIT=0
  re-mesuré). Census 25→29 gelé par LISTE NOMINALE triée
  (check-frontier-contracts.sh:227-245) — voie 1 de l'arbitrage Codex jouée, plus
  aucune contradiction avec le kickoff/plan/preflight B1. Roster doc :57-82 + prose
  rafraîchie vérifiée sur les 6 surfaces : PATTERNS.md §P70 (arithmétique 29=27+2, 4
  schématisées, 25=22+3 cohérente), workflow-core lib.rs:44-60, contracts.rs:14-26,
  engine lib.rs:105-107 (« 29 since S83 Phase F »), THREAT_MODEL:1692-1696 /
  :1958-1962 / :1996-1999, README.md:2178-2180. Gate relancé : « DOMAIN census
  [29 frozen] » EXIT=0. MAIS le correctif introduit un défaut P1 de collation locale
  dans la liste gelée — voir NF-r3-1 (à corriger avant commit, sinon gate rouge en CI
  Linux).
- **r2 NF-2 [P2] — les déclarations du commit body** — STILL-OPEN par construction : le
  commit n'existe pas encore (git status = arbre sale, HEAD 6f95cd2). État re-mesuré de
  chaque déclaration r2 : (1) CADUQUE SOUS SA FORME r2 — les 4 domaines sont DÉCLARÉS
  in-tree (canonical.rs:378/390/399/408), donc plus de « route H/I/X pour les
  constantes » ; le body doit être REFORMULÉ : déclarer census 25→29 (liste nominale) +
  router le REMPLISSAGE des contrats réservés (CompiledPlan.v1→Phase H,
  RunEvent.v1→Phase I, PackInstallReceipt.v1→Phase X). Les doc-comments in-tree ne
  nomment volontairement AUCUNE phase (PROMISE_RE) — le mapping H/I/X ne peut donc
  vivre QUE dans le body/planning : cette déclaration reste DUE. (2)
  mapped/approximated/refused→AG : grep re-joué = 0 hit dans crates/ et docs/ — 100%
  body, DUE. (3) A14 §19.2 : la clause (1) longueur-de-chaînes est désormais couverte
  en code (`MAX_STRING_LEN` pack.rs:108-110), mais (7) schémas $ref pinnés et (9)
  import-brouillon-jusqu'à-revue = 0 trace in-tree (grep re-joué) — DUES au body
  (explicitées ou routées AG). (4) A16 : le diff THREAT_MODEL ne contient QUE les
  rafraîchissements census — aucune section §5.x/§6/§2/§4, pas de §19 ; l'option (a) du
  preflight :796-804 tient, le report (4 menaces : substitution de pack, confusion de
  domaine, downgrade stateful, digest partiel + phases propriétaires) reste DÛ au body.
  (5) import_hostile re-mesuré : `cargo nextest run -p sbfb-workflow-core
  import_hostile` = 5 tests 5/5 PASS (39 skipped ⇒ crate 44, cohérent avec 44/44
  annoncé ; delta body attendu : crate 38→44).

## Dispositions du writer (P2-7 et P3-3, non couverts par les 4 groupes)

- **Codex-r3-P2-7 — enveloppe signée hors registre de schémas** — FIXED (vérifié par le
  writer). `signed_pack.schema.json` présent sur disque (9,7 Ko, untracked, à committer
  avec la phase). Test `signed_pack_schema_snapshot` pack.rs:1914-1936 : génère
  `schema_for!(SignedPack)`, panique si le golden n'est pas committé (« must be
  committed... UPDATE_SNAPSHOTS=1 », :1931) et asserte l'égalité STRICTE (« SignedPack
  envelope schema drift », :1936). La doc du test (:1914-1916) le place explicitement
  « outside the 7-identity registry (it is transport, not a [contract]) » — exactement
  le correctif compagnon demandé par Codex, sans toucher au registre `len == 7`. Inclus
  dans les 44/44 re-mesurés.
- **Codex-r3-P3-3 — variant oublié dans `ALL`** — DOCUMENTED-WITH-REASON, avec
  rationale CORRIGÉE. La raison transmise par la passe de correctifs (« le compilateur
  force ALL[6] ») est IMPRÉCISE : le type `[Self; 6]` (pack.rs:213) ne force PAS un
  nouveau variant à entrer dans `ALL` — le tableau à 6 éléments compilerait inchangé.
  Ce qui casse réellement à la compilation, mesuré sur pièce : les TROIS matches
  exhaustifs sans wildcard — `field()` :224-232, `PackDigests::get` :265-272, et le
  match de mutation DU TEST LUI-MÊME :1691-1698 — un variant neuf fait donc atterrir
  une erreur de compilation À L'INTÉRIEUR du test de complétude. Résidu réel (celui de
  Codex) : un dev qui corrige les 3 matches mais oublie `ALL` laisse
  `names.len()==6` (:1677) vert pendant que le digest neuf échappe à
  `PackDigests::validate` (itère `ALL`, :286) ; le champ struct neuf fait toutefois
  dériver `contracts.schema.json` (snapshot STRICT) → rouge, forçant une relecture
  humaine sans garantir la correction. Refus du fix in-phase confirmé au niveau P3 ;
  fermeture 1 ligne possible plus tard (asserter que `to_value(digests)` a exactement
  `ALL.len()` clés) — à tracer au body avec les STILL-OPEN routés.

## Findings neufs du round 3

- **NF-r3-1 [P1]** `scripts/check-frontier-contracts.sh:229` — la liste nominale gelée
  du census est dans l'ordre de collation en_US (`DOMAIN_KEY_ROTATION_V1` :229 AVANT
  `DOMAIN_KEYSTORE_V1` :230) alors que le pipeline live utilise `sort -u` sans LC_ALL
  (:248) : sur les DEUX surfaces CI Linux (GHA ubuntu-latest LANG=C.UTF-8,
  ci.yml:28/:136 ; Woodpecker rust:1.94 Debian locale POSIX, ci-linux.yml:92) le tri
  est byte-lex et rend KEYSTORE AVANT KEY_ROTATION → mismatch déterministe → gate FAIL
  « census drift » en CI pendant qu'il est vert en local. Défaut INTRODUIT par le
  correctif P1-4/P2-6. RE-MESURÉ par le writer : `LC_ALL=C bash
  scripts/check-frontier-contracts.sh` → « DOMAIN_*_V1 census drift » avec diff
  `12d11 < DOMAIN_KEY_ROTATION_V1` / `13a13 > DOMAIN_KEY_ROTATION_V1` ; run locale par
  défaut → EXIT=0 ; `grep -n LC_ALL` sur le script = 0 hit ; '_'=0x5F > 'S'=0x53 en
  byte-lex, en_US ignore l'underscore ; seule paire affectée sur les 29 (vérification
  exhaustive des adjacences). Fix : `LC_ALL=C sort -u` à la ligne 248 + permuter les
  lignes 229/230, puis re-prouver EXIT=0 sous les DEUX locales. Le check bridge-parity
  n'est PAS affecté (il compare deux sorties du MÊME sort runtime, jamais une liste
  committée).
- **NF-r3-2 [P2]** `pack.rs:1119` — la troncature à `MAX_REPORTED_ENDPOINT_CHARS`
  s'applique AVANT `redact_userinfo` : un endpoint dont l'userinfo dépasse ~240 chars
  perd son '@' à la coupe et le credential partiel est cité verbatim dans le
  `ScanFinding` — le résidu exact de la fuite que P1-5 visait. Reproduit empiriquement
  (copie verbatim de `collect_findings`/`redact_userinfo`, scratchpad) : 'see
  https://user:<password JWT-shaped 300 chars>@host/path' → endpoint tronqué à 256
  chars sans '@' → `rfind('@')=None` → retour verbatim, le finding contient
  'https://user:eyJZ0eyJZ0…' (~243 chars du credential). Scénario plausible : JWT ou
  long token collé comme mot de passe d'URL. Correctif : rédiger sur la run non-blanche
  COMPLÈTE puis tronquer (redact avant `take(256)`).
- **NF-r3-3 [P2]** `pack.rs:2303` — aucun test ne pinne les deux propriétés de
  non-fuite livrées par le fix P1-5 : deux mutations qui régressent exactement le
  défaut signalé par Codex laissent la suite 44/44 verte. Mutation A : supprimer
  l'appel `redact_userinfo` (pack.rs:1123 → quoter endpoint brut) —
  `scanner_is_advisory_not_a_gate` (pack.rs:1828) n'utilise qu'une URL SANS
  credentials, aucun autre test n'appelle le scanner sur une URL credentialisée →
  verte. Mutation B : ré-ajouter `{secret_ref}` au message d'erreur (pack.rs:893) — le
  cas corpus valid-signature-over-materialised-secret (pack.rs:2302-2309) n'asserte que
  la PRÉSENCE du fragment 'opaque logical reference', jamais l'ABSENCE de
  'sk-abcdef0123456789' dans `err.to_string()` → verte. Doctrine projet
  (feedback_mutation_before_trust) : un fix sécurité fraîchement audité doit être pinné
  par un contrôle discriminant.
- **NF-r3-4 [P3]** `crates/nexus-core-rs/src/canonical.rs:80` — le roster module
  (bullet des 3 tags réservés, :77-82) affirme « their payload contracts are reserved
  identities in sbfb-workflow-core::contracts, still stubs » pour les TROIS tags — vrai
  pour CompiledPlan.v1 et RunEvent.v1, FAUX pour l'install receipt : grep
  'InstallReceipt|PackInstall' dans contracts.rs = 0 hit ; `CONTRACT_NAMES`
  (contracts.rs:136-144) n'a pas d'identité receipt. La doc per-constante
  (canonical.rs:401-408) est elle honnête (« the receipt's contract is authored by the
  phase that installs »). Fix : reformuler le bullet (2 identités réservées + 1 contrat
  à autoriser par la phase d'installation).
- **NF-r3-5 [P3]** `pack.rs:198` — résidu du point secondaire de Codex P1-2 : la doc
  `PackDigestKind::GraphSemantic` dit « The meaning of the workflow graph: nodes and
  edges by stable id » alors que `graph_semantic_view` (pack.rs:1191-1206) mappe les
  arrays élément par élément sans tri par id ; JCS préserve l'ordre des arrays —
  réordonner deux nodes change le digest. Fix : reformuler (« ids stables pinnés, ordre
  des tableaux significatif jusqu'au schéma typé de graphe et son ordre total ») ou
  trier quand le contrat de graphe typé existera.
- **NF-r3-6 [P3]** `pack.rs:1435` — le message de panic du test miroir durci contient
  un run d'environ 14 espaces au milieu de la phrase ('a              commented copy') —
  cosmétique introduit par la passe de correctifs (rustfmt ne reformate pas les
  littéraux de chaîne). Aucun impact fonctionnel.
- **NF-r3-7 [P3]** `pack.rs:856` — sur les 5 caps P2-4, seul le per-file a un cas
  hostile : le cap total, la branche overflow du `checked_add` et les 3 caps de chemin
  (512 bytes / 32 segments / 255 bytes-segment) n'ont aucun test — muter `checked_add`
  en `wrapping_add` ou supprimer un des 3 caps de chemin laisse la suite verte. Grep
  tests : `MAX_FILE_SIZE_BYTES` exercé (pack.rs:2255) ; aucune occurrence de
  `MAX_TOTAL_SIZE_BYTES`, `MAX_LOCK_PATH_BYTES`, `MAX_PATH_SEGMENTS` ni
  `MAX_SEGMENT_BYTES` dans le mod tests.
- **NF-r3-8 [P3]** `pack.rs:13` — micro-incohérence entre les deux moitiés du miroir :
  pack.rs dit « the holder of one Ed25519 key published exactly these canonical bytes »
  alors que canonical.rs:373-374 dit désormais « signed... not a publication event ».
  Dérive dans le sens de PLUS d'honnêteté côté canonical.rs — aligner pack.rs:13 sur
  'signed'.
- **NF-r3-9 [P3]** `docs/claude/README.md:2178` — phrase census rafraîchie ambiguë :
  « 25 sur 29 depuis S83-F, dont 4 schématisées et 3 tags réservés » a des antécédents
  mixtes (les 4 schématisées appartiennent aux 29, les 3 réservés aux 25 sans schéma) ;
  et le sujet « carry routé vers l'audit-plan S80 » couvre désormais les 3 tags mintés
  en S83-F, qui n'en font PAS partie. Fix une phrase : « 22 sur 25 à l'époque S79
  (carry routé audit-plan S80) ; 25 sur 29 depuis S83-F (22 carry + 3 tags réservés
  S83-F, 4 schématisées) ». Arithmétique correcte ailleurs (PATTERNS §P70).
- **NF-r3-10 [P3]** `pack.rs:241` — la tri-partition de la doc PackDigests suggère que
  graph_semantic/presentation reçoivent plus qu'une validation de forme au verify :
  « compiled_plan, evaluation and publication are validated for SHAPE only here »
  implique par contraste un tier intermédiaire qui n'existe pas — dans `validate_pack`,
  graph_semantic et presentation sont EXACTEMENT aussi shape-only que les trois autres
  (seul content est recalculé ; les 5 autres passent par `is_lower_hex_digest` seul,
  :285-295). Fix une clause : « the OTHER FIVE are validated for SHAPE only here;
  graph_semantic and presentation are additionally recomputable by a caller holding
  the graph document via graph_semantic_view / presentation_view ».
- **NF-r3-11 [P3]** `scripts/check-frontier-contracts.sh:8` — en-tête « Five
  deterministic checks ... BusyBox-safe » doublement dérivé : le script porte désormais
  7 vérifications restituées par l'echo final (ajout (2c) pack-parity en F), et le
  correctif P2-6 a introduit une process substitution bash-only `diff <(...)` (:252)
  que l'énumération BusyBox ne couvre pas — sous un sh BusyBox le bloc if mourrait en
  erreur de syntaxe. Latent seulement : les 3 sites d'invocation utilisent
  explicitement bash (ci.yml:136, ci-linux.yml:92, verify.sh:98-99). La staleness du
  compte pré-existe aux correctifs (git show HEAD), la process substitution non. Fix :
  dé-chiffrer l'en-tête + noter que le script requiert bash (le shebang :1 le dit
  déjà).

## Suites §7.4 — état OBSERVÉ (jamais « vert » sans observation)

Mesuré par les groupes de ce round ou par la passe de correctifs et RE-MESURÉ ici :
- crate `sbfb-workflow-core` : **44/44 PASS** (41→44, +3 :
  `presentation_view_is_injective_on_ambiguous_keys`,
  `canonical_door_refuses_the_measured_integer_collapse`,
  `signed_pack_schema_snapshot`) ;
- gate `import_hostile` : **5 tests, 5/5 PASS** (commande d'enforcement exacte) ;
- miroir TS `pack_digest.test.ts` : **14/14 PASS** (13→14, lone-surrogate) ; suite web
  attendue 529→543 ;
- `check-frontier-contracts.sh` **EXIT=0 en locale locale** (« DOMAIN census
  [29 frozen] ») — MAIS **rouge sous LC_ALL=C** (NF-r3-1, re-mesuré par le writer) ;
  `check-spdx.sh` **EXIT=0** ;
- fmt : **0 sous les deux toolchains** (canonique ET local) ;
- mutations re-jouées post-correctifs : A (retirer le refus backslash) → cas rouge par
  fragment ; B (retirer le tri) → cas rouge par acceptation détectée — les deux
  mutations qui restaient VERTES avant le fix P1-3 sont TUÉES.

**EN COURS au moment de ce round, NON observés verts — à attendre avant commit** :
- `cargo nextest run --workspace --locked` Windows complet
  (fmt/clippy/doctests/release) ;
- pipeline Docker canonique `sbfb-ci` (rust:1.94, `SBFB_TEST_HTTP_TIMEOUT_SECS=120`) ;
- suite web complète (`npm run test:unit` 543 attendus, coverage, build, size).

## Déclarations obligatoires du commit body (liste REFORMULÉE, remplace r2 NF-2)

Les routes DOMAIN de r2 (« COMPILED_PLAN→H / RUN_EVENT→I / PACK_INSTALL_RECEIPT→X pour
les constantes ») sont CADUQUES : les 4 constantes existent in-tree. Restent DUES :

1. Census 25→29 gelé par liste nominale ; routage du REMPLISSAGE des contrats
   réservés : CompiledPlan.v1→Phase H, RunEvent.v1→Phase I,
   PackInstallReceipt.v1→Phase X (mapping impossible in-tree à cause de PROMISE_RE — il
   ne peut vivre que dans le body/planning).
2. D8 par famille : WORKFLOW_PACK = schéma généré (pinné `contracts.schema.json` +
   compagnon `signed_pack.schema.json` STRICT hors registre 7) ; les 3 réservées =
   no-schema motivé jusqu'au remplissage.
3. Scope cut daté mapped/approximated/refused→AG (0 trace in-tree, grep re-joué).
4. A14 §19.2 : (7) schémas $ref pinnés + (9) import-brouillon-jusqu'à-revue —
   explicitées ou routées AG (la clause (1) longueurs est couverte en code).
5. Report THREAT_MODEL A16 : 4 menaces (substitution de pack, confusion de domaine,
   downgrade stateful, digest partiel) + phases propriétaires.
6. Deltas tests mesurés : crate 38→44 (import_hostile = 5), miroir TS 14, web 529→543
   attendus, workspace Windows/Docker à reporter une fois OBSERVÉS.
7. STILL-OPEN/P3 routés : NF-r3-2/3 s'ils ne sont pas fixés avant commit,
   NF-r3-4/5/7/8/9/10/11, résidu P3-3 (assertion structurelle `ALL.len()` 1 ligne),
   scanner/exemption, sorted-by-producer (fermé), lock-depth (fermé),
   EXTERNAL_AUDIT_SCOPE (CG-2 r2).

## Verdict: CONCERN

Les 16 findings Codex r3 sont disposés (14 FIXED + 2 DOCUMENTED-WITH-REASON, 5/5 P1
fermés, évidence re-mesurée partout), les mutations A et B sont tuées, la suite crate
est 44/44 — mais ce round ouvre **1 P1 neuf** (NF-r3-1, collation de la liste census :
gate frontier rouge déterministe sur les deux CI Linux, introduit par le correctif
P1-4/P2-6 et re-mesuré sur place) et 2 P2 neufs (NF-r3-2 troncature avant rédaction ;
NF-r3-3 fix sécurité P1-5 non pinné par mutation). PAS de PASS-PENDING tant que le P1
est ouvert. Codex r4 n'a pas encore tourné sur cet arbre.

Ordre des actions avant commit :

1. **Fixer NF-r3-1** (2 lignes : `LC_ALL=C sort -u` à
   check-frontier-contracts.sh:248 + permutation des lignes 229/230) puis re-prouver le
   gate EXIT=0 sous la locale par défaut ET sous `LC_ALL=C`.
2. **Fixer NF-r3-2 et NF-r3-3** (redact avant troncature + 2 assertions de non-fuite
   discriminantes — doctrine feedback_mutation_before_trust) ; quick-fixes P3
   recommandés dans la foulée (NF-r3-4/5/6/8/10/11, chacun ≤ 3 lignes).
3. **Attendre et OBSERVER les 3 suites en cours** (workspace Windows complet, Docker
   canonique, web complet) — aucun commit sans les trois verts constatés.
4. **Codex r4 sur BUNDLE RAFRAÎCHI** (l'arbre corrigé, jamais un bundle périmé — leçon
   Phase C/E), artefact `rounds/sprint83_phase_f_codex_review_r4.md` brut, boucle
   jusqu'à « CLEAN ou P2/P3 documentés ».
5. **Commit body 9 sections** portant les 7 déclarations reformulées ci-dessus.
6. **Copie canonique maintenue** : `.planning/active/sprint83_phase_f_review.md` reste
   la copie byte-identique du DERNIER round (canon §4.5.8), condition du hook
   phase-auditor-gate et du CI cross-check.
