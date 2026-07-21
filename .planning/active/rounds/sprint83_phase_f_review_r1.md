# Sprint 83 Phase F — Review round 1 (Workflow ultracode, INTERROMPUE puis consignée)

**Sprint** : 83 — chantier « Workflow Agents ». **Phase** : F — Format portable et signé
(`SBFBPromptAgentPack.v1`). **Date du round** : 2026-07-20 (consigné 2026-07-21).
**HEAD** : `6f95cd2`, arbre sale (phase non committée). **Workflow** : `wf_bc8858f4-119`.

## Méthode et incident de run

Fan-out de **8 dimensions de review** (correction ligne à ligne, sécurité profonde,
sémantique des tests, conformité preflight, patterns dépôt, livrables du plan,
docs-contrat, honnêteté des garanties), chacune re-mesurant ses claims sur le diff réel.
Les 8 dimensions ont TERMINÉ et produit **79 findings bruts** (19 déclarés P0/P1).

**Incident** : la passe adversariale (1 réfuteur par finding) et la synthèse sont mortes
sur la limite hebdomadaire du modèle des sous-agents (80 agents en échec « weekly
limit ») — 4 réfuteurs seulement ont terminé (2 claims survivent, 2 réfutées). Ce round
est donc consigné SANS réfutation systématique : les findings ci-dessous sont les
sorties brutes des dimensions, à traiter comme des claims re-mesurées par leur
dimension mais non contre-vérifiées par un adversaire indépendant. Le round 2
(`sprint83_phase_f_review_r2.md`) porte la vérification post-correctifs sur l'arbre
corrigé, modèle de session (Fable 5) hérité — la directive « forcer opus-4-8 » étant
inapplicable pendant la fenêtre de limite.

Valeur d'audit conservée : les 4 verdicts de réfutation partiels vivent dans le journal
du Workflow (`wf_bc8858f4-119/journal.jsonl`), dont « aucun blake3 côté TS » RÉFUTÉ
(déjà au lock via eciesjs) et le chemin miroir TS re-confirmé.

## Verdicts par dimension et findings bruts

### Dimension 8 — Honnêteté des garanties (doc-comments, module-docs, champs `comment` JSON)

Verdict de dimension : **FAIL**

Vérifié et trouvé correct : 10 points (détail dans le journal du Workflow wf_bc8858f4-119).

Findings :

- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:34` — Le module-doc résume l'étape 6 par « Semantic validation — all six digests present, subjects well formed, **no materialised secret** ». Littéralement : un pack qui sort de `verify_signed_pack` ne contient aucun secret matérialisé. Ce que le code garantit : aucune chaîne de `secret_refs` (et de RIEN d'autre) ne matche une heuristique de 6 préfixes + une règle d'entropie. VERDICT : SUR-AFFIRMÉE.
- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:294` — Le doc-comment de `PackLock` affirme : « This is what makes a pack rug-pull resistant: the signature covers the lock, and the lock covers the bytes, so **swapping a prompt after publication breaks verification**. » Littéralement : substituer un fichier du pack après publication fait échouer la vérification. Ce que le code garantit : `verify_signed_pack` ne lit AUCUN fichier et ne compare AUCUN digest à un contenu — il vérifie seulement que les 64 caractères sont hex. VERDICT : FAUSSE au périmètre de ce crate.
- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:519` — `verify_signed_pack(raw, domain, verifying_key)` prend le domaine en PARAMÈTRE et ne le contraint jamais à `PACK_SIGNING_DOMAIN`, alors que les doc-comments parlent comme s'il était fixe : pack.rs:13-14 « the holder of one Ed25519 key published exactly these canonical bytes **under the pack domain** » ; pack.rs:509 « were these bytes signed by THAT key **under the pack domain** ». Ce que le code garantit : « signées par cette clé sous le domaine que L'APPELANT a passé ». VERDICT : SUR-AFFIRMÉE (+ danger d'API latent).
- **[P1]** `crates/sbfb-workflow-core/src/pack_jcs_vectors.json:2` — Le champ `comment` affirme : « Adverse cases on purpose: key ordering by UTF-16 code unit, non-BMP keys, escapes, **exponent forms, negative zero**. » Littéralement : le corpus de parité contient des formes exponentielles et un zéro négatif. VERDICT : FAUSSE — ni l'un ni l'autre n'est présent ; il n'y a AUCUN nombre non entier dans tout le fichier.
- **[P1]** `crates/nexus-core-rs/src/canonical.rs:4` — Le module-doc de `canonical.rs` — fichier édité par cette phase — ouvre sur « **All signed payloads in the nexus-grid system go through** [`canonical_bytes`], which produces RFC 8785 JCS output with a type-specific domain separation prefix. » La Phase F rend cette phrase FAUSSE : le pack est un payload signé dont la pré-image est produite par une AUTRE fonction, dans un autre crate.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:27` — Étape 3 du module-doc : « **Shape bounds** — depth, collection cardinality and string length are enforced **on that `Value`** ». L'antécédent de « that Value » est l'enveloppe décrite à l'étape 2 (« the envelope is read as a `serde_json::Value` »). Le code ne borne que `$.payload`. SUR-AFFIRMÉE.
- **[P2]** `crates/nexus-core-rs/src/canonical.rs:347` — Le doc-comment neuf de `DOMAIN_WORKFLOW_PACK_V1` justifie son emplacement par « the constant is reserved *here* so **every SBFB signing domain stays enumerable in one place** ». Dans le même commit, `pack.rs` fabrique six tags de domaine par concaténation qui ne sont ni dans `canonical.rs`, ni visibles du census gelé.
- **[P2]** `crates/sbfb-workflow-engine/src/lib.rs:106` — Troisième site de prose recensé par le préflight (§6 : « Trois blocs de prose affirment aujourd'hui que le census reste intact ») — non rafraîchi par la phase. Il affirme encore « the frozen `DOMAIN_*_V1` census stays untouched (cf. `CLAUDE.md §Pre-launch protocol policy`) », alors que le census est passé de 25 à 26 dans ce même commit.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:1158` — Le chemin du miroir TS est FAUX dans les deux endroits qui le citent : `web/src/workflow/pack.digest.test.ts`. Le fichier livré est `web/src/workflow/__tests__/pack_digest.test.ts` (répertoire ET nom différents).
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:44` — Deux petites sur-affirmations résiduelles. (a) « [`canonical_bytes`] is the single canonicalisation door of this module … **Everything that digests, signs or compares goes through it** » — `pub fn blake3_hex` (:900-911) est une porte de digest publique qui ne passe par rien. (b) `PackLock.entries` : « Pinned files, paths unique and **sorted by the producer** » (:301) — le tri n'est vérifié nulle part, et un lock VIDE est accepté alors que le module-doc dit que le pack « pins every one of its files » (:5).

### Dimension 3 — Valeur sémantique des tests (pack.rs mod tests + mod import_hostile + web/src/workflow/__tests__/pack_digest.test.ts)

Verdict de dimension : **CONCERN**

Vérifié et trouvé correct : 15 points (détail dans le journal du Workflow wf_bc8858f4-119).

Findings :

- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:37` — Le module-doc affirme que le test d'ORDRE vit dans `import_hostile`, ce qui est FAUX, et cette fausseté fait que la garantie phare de la phase n'est PAS couverte par la gate `G-IMPORT-HOSTILE` que F ouvre.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:1354` — L'assertion « no false positive on the pack's own digests » de `scanner_is_advisory_not_a_gate` est TAUTOLOGIQUE : elle est vraie par construction indépendamment de l'exemption `is_hex_digest` qu'elle prétend couvrir. Supprimer cette exemption du code de prod laisse la suite 38/38 verte.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:428` — Aucun test n'exerce la borne À la valeur du cap : les quatre bornes (`MAX_JSON_DEPTH`, `MAX_STRING_LEN`, `MAX_COLLECTION_LEN`, `MAX_SAFE_INTEGER`) n'ont que leur côté « au-delà ». Une mutation `>` → `>=` sur n'importe laquelle survit à toute la suite.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:1200` — Le doc-comment de `canonical_bytes_rejects_the_null_collision` affirme que le test « DÉMONTRE la collision au lieu de la supposer ». Le corps ne démontre AUCUNE collision : il ne produit jamais deux valeurs distinctes partageant une forme canonique.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:1249` — `digest_kinds_are_exhaustive_and_named` ne teste l'exhaustivité que dans le sens SUPPRESSION (déjà garanti par le compilateur). Un 7e digest ajouté à l'enum et à `PackDigests` sans être ajouté à `ALL` passerait inaperçu : le test reste vert.
- **[P2]** `web/src/workflow/__tests__/pack_digest.test.ts:1` — La « parité cross-langage » n'a aucun garde de non-vacuité : supprimer ce fichier TS laisse le Rust 38/38 vert et la revendication devient creuse, alors que le précédent maison (bridge-parity, Phase C) implémente exactement ce self-test.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:1119` — `graph_semantic_ignores_presentation_but_not_meaning` ne teste que la direction favorable. La collision structurelle du filtre par NOM de membre — un champ porteur de sens nommé `width`/`color`/`x`/`viewport` disparaît du digest sémantique — n'est couverte par aucun test.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:1158` — Le doc-comment de `jcs_cross_lang_parity` désigne un fichier qui n'existe pas.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:1494` — `hostile_corpus_is_not_vacuous` est un doublon strict de `valid_pack_round_trips` et n'exclut qu'un vérificateur qui refuse LITTÉRALEMENT tout, pas un vérificateur trop strict sur une dimension attaquée.
- **[P3]** `crates/sbfb-workflow-core/src/pack_hostile_corpus.json:84` — L'entrée `oversized-envelope` prétend prouver « must die on the size bound, before serde sees a byte » ; le test ne peut pas distinguer cette propriété.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:1595` — Le commentaire « Every declared post-signature entry is exercised, BY NAME — the corpus file cannot grow a case that silently goes untested » est contredit par le `.filter` de la ligne suivante : une entrée déclarée est exclue sans justification écrite.
- **[P3]** `web/src/workflow/__tests__/pack_digest.test.ts:107` — Le test TS « is sensitive to a change in the value, not only to its shape » n'exerce pas le canonicaliseur : il asserte essentiellement la résistance aux collisions de BLAKE3.

### Dimension 5 — Conventions et patterns du depot (SPDX, PROMISE_RE, etiquette FRONTIER, census DOMAIN, nommage, module-doc, style/langue)

Verdict de dimension : **FAIL**

Vérifié et trouvé correct : 12 points (détail dans le journal du Workflow wf_bc8858f4-119).

Findings :

- **[P1]** `web/src/workflow/__tests__/pack_digest.test.ts:1` — Le fichier TS neuf n'a PAS d'en-tete SPDX : `bash scripts/check-spdx.sh` passe de vert a EXIT=1. L'invariant du preflight §8.4 (« SPDX | bash scripts/check-spdx.sh | exit 0 ») est VIOLE, et la liste de mesures de l'implementeur ne mentionne jamais check-spdx.
- **[P1]** `crates/sbfb-workflow-engine/src/lib.rs:106` — La 7e surface census que le preflight B3 exige explicitement de corriger n'a PAS ete touchee : le module-doc de `sbfb-workflow-engine` affirme encore « the frozen `DOMAIN_*_V1` census stays untouched », proposition devenue globalement fausse (25 -> 26). Le fichier n'apparait pas dans `git status`.
- **[P3]** `crates/sbfb-workflow-core/src/lib.rs:83` — La nouvelle section de module-doc « ## Sprint 83 Phase F » est separee de la precedente par une ligne VIDE brute (pas `//!`), rompant le style de toutes les autres sections du meme module-doc et scindant le doc interne en deux groupes d'attributs.
- **[P3]** `crates/nexus-core-rs/src/canonical.rs:343` — Le doc-comment de la nouvelle constante justifie son emplacement par analogie avec `DOMAIN_KUDOS_V1`, « whose payload is likewise minted outside this crate » — or le precedent invoque est mort : la charge kudos etait mintee par le coordinateur PYTHON, supprime aux S50-S51 (projet Rust+Frontend pur depuis).
- **[P3]** `docs/security/THREAT_MODEL.md:1695` — Le bloc autonome « **Wire / census** » de la surface a11y affirme « census DOMAIN inchange (25) » sans marqueur temporel ; le nombre est desormais perime (26). Deux occurrences soeurs (:1960, :1997) sont, elles, clairement scopees a leur phase et restent des enonces historiques vrais.

### 6 — Livrables du plan (Sprint 83 Phase F : F.2 livrables, F.3 tests, F.4 critère d'acceptation)

Verdict de dimension : **CONCERN**

Vérifié et trouvé correct : 10 points (détail dans le journal du Workflow wf_bc8858f4-119).

Findings :

- **[P1]** `crates/sbfb-workflow-core/src/pack_hostile_corpus.json:3` — Quatre des sept classes de corpus hostile NOMMÉES en F.2 sont absentes du corpus committé, sans scope cut écrit : casse Windows, clés JSON dupliquées, SVG, Markdown (plus les confusables Unicode exigés par A13).
- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:922` — « Les 6 fonctions de digest » de F.2 sont livrées comme 1 fonction paramétrée + 2 projections : `content`, `compiled_plan`, `evaluation` et `publication` n'ont AUCUNE projection définie, et rien ne relie `PACK.lock` à `digests.content` (test d'exhaustivité A10(i) absent).
- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:72` — Une seule famille `DOMAIN_*` sur les 4 nommées par F.2 est livrée, alors que `DOMAIN_RUN_EVENT_V1` avait été explicitement ROUTÉ vers F par le préflight de la phase E ; aucune trace écrite de ce report dans un artefact committé.
- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:763` — Le rapport `mapped/approximated/refused`, nommé explicitement en F.2 au même titre que le scanner, est totalement absent — aucune structure, aucune fonction, aucun mot.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:37` — Le module-doc désigne le test gardien de l'ordre par un chemin qui n'existe pas : `import_hostile::signature_before_parse`. Le test est dans `tests::`, et le gate `import_hostile` ne le sélectionne PAS.
- **[P2]** `crates/sbfb-workflow-core/src/pack_jcs_vectors.json:2` — Le commentaire du fichier de vecteurs revendique une couverture qu'il n'a pas : « exponent forms, negative zero » — aucun des 7 vecteurs ne contient de forme exponentielle ni de `-0` (l'adaptation A4 du préflight les exigeait nommément).
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:354` — `SignedPack` est un type public déclaré, documenté et `deny_unknown_fields`, mais jamais construit, jamais désérialisé, jamais testé, et absent du golden de schéma : rien ne garantit qu'il décrit l'enveloppe que le vérificateur accepte réellement.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:307` — La phrase « `layout/` stays outside the semantic digest » n'est adossée à rien : le pack ne porte aucun champ `layout`, et `graph_semantic_view` filtre un MEMBRE JSON nommé `layout`, pas un répertoire `layout/` du pack.
- **[P2]** `crates/sbfb-workflow-engine/src/lib.rs:105` — Surface B3(6bis) du préflight non traitée : la prose de `sbfb-workflow-engine` affirme toujours que « the frozen `DOMAIN_*_V1` census stays untouched », phrase que la phase F rend partiellement périmée, et le gate ne la voit pas.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:1158` — Le chemin du miroir TS est faux dans deux fichiers committés : `web/src/workflow/pack.digest.test.ts` n'existe pas, le fichier livré est `web/src/workflow/__tests__/pack_digest.test.ts`.

### 7 — Cadence docs-contrat (§6.12 / §P70 / AGENT_SYSTEM §7) : étiquette générée par phase, GUIDE en clôture, arêtes de provenance rang-1, test-acteur

Verdict de dimension : **CONCERN**

Vérifié et trouvé correct : 8 points (détail dans le journal du Workflow wf_bc8858f4-119).

Findings :

- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:37` — Le module-doc de la primitive de sécurité désigne le test qui garde l'ordre verify/parse dans le MAUVAIS module : il écrit `import_hostile::signature_before_parse`, or le test s'appelle `pack::tests::signature_before_parse`. Conséquence non triviale : la commande d'enforcement de G-IMPORT-HOSTILE ne le sélectionne PAS.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:1158` — Les DEUX seules références au miroir TypeScript pointent vers un chemin qui n'existe pas (`web/src/workflow/pack.digest.test.ts`) ; le fichier réel est `web/src/workflow/__tests__/pack_digest.test.ts` (point vs underscore, et répertoire `__tests__/` omis).
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:1196` — L'étiquette cross-langage n'a AUCUN gate d'existence côté TS, alors que la Phase C du MÊME sprint a établi le précédent dans le MÊME script pour la primitive de parité Rust↔TS précédente. Supprimer `web/src/workflow/__tests__/pack_digest.test.ts` laisse `cargo nextest`, `npm run test:unit`, `check-frontier-contracts.sh` et `check-factory-docs.sh` tous verts.
- **[P2]** `crates/sbfb-workflow-core/src/contracts.rs:192` — La décision D8 écrite dans les DEUX surfaces de prose que la phase édite désigne `contracts.schema.json` comme le pin de l'étiquette ; or `schema_snapshot_stub` est FAIL-OPEN à la suppression, alors que les deux goldens NEUFS de la même phase sont STRICT avec un commentaire anti-suppression explicite. L'arbitrage que le préflight A1 demandait de RENDRE n'est rendu nulle part.
- **[P2]** `crates/sbfb-workflow-engine/src/lib.rs:106` — La 6e surface de prose census listée par le préflight B3 n'a pas été rafraîchie : `sbfb-workflow-engine/src/lib.rs` affirme toujours « the frozen `DOMAIN_*_V1` census stays untouched », alors que la phase l'a fait passer de 25 à 26 — et que les DEUX phrases jumelles de `sbfb-workflow-core` ont, elles, été réécrites dans le même commit.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:56` — L'étiquette `// FRONTIER:` est détachée de son type (262 lignes plus haut) et ne porte aucune arête de provenance, contrairement à l'unique précédent du dépôt.
- **[P3]** `scripts/check-frontier-contracts.sh:190` — Le registre `// FRONTIER:` est opt-in DANS LES DEUX SENS : le nombre d'étiquettes n'est pas gelé, donc supprimer l'étiquette que F vient de poser ramène le gate à `[1 tagged]` et EXIT 0 — alors que le census, lui, est gelé par un entier dans ce même fichier.
- **[P3]** `docs/factory/WORKFLOW_AGENTS_OVERVIEW.md:6` — Le GUIDE amorcé du chantier promet « il grandit à chaque phase » et n'a été touché par AUCUNE phase depuis A ; il n'est en outre indexé par aucun `llms.txt` et gaté par aucun script. F reconduit l'écart sans le déclarer.

### 1 — Correction du code, ligne par ligne (pack.rs + diff canonical.rs)

Verdict de dimension : **CONCERN**

Vérifié et trouvé correct : 18 points (détail dans le journal du Workflow wf_bc8858f4-119).

Findings :

- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:37` — Le module-doc localise LE test d'ordre dans `import_hostile`, alors qu'il vit dans `mod tests` — consequence mesurable : la commande de gate `G-IMPORT-HOSTILE` ouverte par la phase NE selectionne PAS `signature_before_parse`.
- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:815` — `PRESENTATION_KEYS` filtre par NOM de membre a toute profondeur : un membre semantiquement porteur nomme `width`/`height`/`x`/`y`/`color`/`zoom` est silencieusement retire du digest `graph_semantic`. Le doc-comment « Member names that carry PRESENTATION, never meaning » est faux, et mesurablement.
- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:650` — Le doc-comment affirme que « the reserved DOS device names are refused » ; la liste n'en contient que 8 sur les 22 reserves par Windows. `lpt3.txt`, `com3.txt`… sont ACCEPTES.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:426` — La borne I-JSON de `enforce_bounds` est contournable par la SYNTAXE flottante, et elle est absente de `canonical_bytes`/`digest_for` — la porte de canonicalisation elle-meme ne l'applique nulle part.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:852` — `presentation_view` est documentee « lossless » ; la projection par chemin-textuel n'est pas injective et perd silencieusement une valeur.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:231` — Les digests sont acceptes en hex MAJUSCULE, alors que `blake3_hex` n'emet que du minuscule — deux encodages du meme digest passent la validation et produisent deux packs canoniquement distincts.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:619` — L'unicite des chemins de lock est verifiee sur la chaine brute : elle ne survit pas a la materialisation Windows (casse, point/espace final), et `%2e%2e` traverse.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:294` — Un pack dont `lock.entries` est VIDE est accepte, alors que le doc-comment de `PackLock` fonde sur le lock la resistance au rug-pull.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:519` — `domain` est un parametre libre de `verify_signed_pack`, non documente et sans valeur sure par defaut : la separation de domaine repose entierement sur la discipline de l'appelant.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:700` — Le doc de `looks_like_materialised_secret` sur-promet (« anything that carries entropy or a known credential prefix ») ; l'heuristique laisse passer une AWS Access Key ID et un JWT, motifs que le scanner deja present dans le depot couvre.
- **[P3]** `crates/sbfb-workflow-core/src/pack_jcs_vectors.json:2` — Le commentaire du corpus de parite revendique des cas adverses (« exponent forms, negative zero ») qui n'y figurent pas.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:765` — `scan_for_disclosures` avale l'echec de `to_value` et renvoie « aucune trouvaille » — indiscernable de « rien trouve ».
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:569` — `payload_value.clone()` copie integralement un payload pouvant atteindre 1 MiB, et la chaine `signature` n'est bornee par aucune des bornes de forme.

### Sécurité profonde — frontière de confiance « pack signé hostile » (Sprint 83 Phase F)

Verdict de dimension : **CONCERN**

Vérifié et trouvé correct : 18 points (détail dans le journal du Workflow wf_bc8858f4-119).

Findings :

- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:531` — Malléabilité de signature / différentiel de parseur : les clés JSON dupliquées ne sont refusées NULLE PART, ni au niveau enveloppe ni au niveau payload. Des octets bruts distincts — dont un porteur d'un `name` attaquant — vérifient et rendent le même pack. L'adaptation A8 du préflight (obligatoire) a disparu sans test, sans entrée de corpus et sans mention.
- **[P1]** `crates/sbfb-workflow-core/src/pack_jcs_vectors.json:2` — Le commentaire du corpus de parité affirme couvrir des cas adverses qui n'y sont PAS : « exponent forms » et « negative zero » sont annoncés, aucun vecteur ne les contient. C'est exactement l'adaptation A4 du préflight et le risque R5.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:509` — `verify_signed_pack` prend le domaine en PARAMÈTRE alors que sa doc affirme répondre « signed by THAT key under the pack domain » : la séparation de domaine n'est pas une propriété de la fonction, elle est déléguée à chaque appelant — et il n'existe aucun appelant de production pour la tenir.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:652` — `validate_lock_path` traite les noms de périphériques Windows mais laisse passer les deux règles d'aliasing Win32 qui produisent EXACTEMENT l'ambiguïté que le refus des chemins dupliqués existe pour fermer : point final, espace final, et collision par la casse.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:343` — La règle « aucun secret matérialisé » promise par la doc est plus large que la règle appliquée : elle ne couvre qu'un champ (`secret_refs`) et rate les credentials du propre scanner du dépôt. Deux mesures le montrent.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:426` — La garde d'entiers I-JSON est annoncée en termes de « valeur au-delà de la plage exacte IEEE-754 » mais n'est appliquée qu'aux nombres que serde_json a stockés en i64/u64 ; les entiers plus grands que u64 et les formes exponentielles passent sans `Unbounded`.
- **[P2]** `docs/security/THREAT_MODEL.md:1698` — Aucune trace de la nouvelle frontière de confiance dans le THREAT_MODEL, et la phase introduit désormais des surfaces qui n'étaient pas au catalogue du préflight (différentiel de parseur, domaine fourni par l'appelant, refus de secret partiel).
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:1158` — Le doc-comment du test de parité pointe un fichier TypeScript qui n'existe pas à ce chemin.

### D4 — Conformité au preflight G8 (`.planning/active/sprint83_phase_f_preflight.md`, verdict PLAN-ADAPT) : B1..B6, A1..A20, interdits §8.3, invariants §8.4

Verdict de dimension : **CONCERN**

Vérifié et trouvé correct : 14 points (détail dans le journal du Workflow wf_bc8858f4-119).

Findings :

- **[P1]** `web/src/workflow/__tests__/pack_digest.test.ts:1` — A2/§8.4 « SPDX → exit 0 » VIOLÉ : le seul fichier neuf non-Rust de la phase n'a pas d'en-tête SPDX, et le gate SPDX est CI-BLOQUANT. L'invariant §8.4 est donc rouge à HEAD+dirty.
- **[P1]** `crates/sbfb-workflow-core/src/pack.rs:817` — A12 NON APPLIQUÉ tel que prescrit et remplacé par la solution que le preflight interdisait : au lieu d'« ids explicites + ordre total », `graph_semantic` est calculé par une LISTE NOIRE de NOMS DE MEMBRES appliquée à toute profondeur — ce qui efface silencieusement des membres PORTEURS DE SENS et produit une collision de digest sémantique.
- **[P2]** `crates/sbfb-workflow-engine/src/lib.rs:106` — B3 PARTIEL — 6 des 7 surfaces de census rafraîchies. La surface que le preflight nomme explicitement comme devant être corrigée MÊME SI F ne touche pas le crate engine est restée intacte et est maintenant fausse.
- **[P2]** `crates/nexus-core-rs/src/canonical.rs:338` — B1 — la LISTE close est bien écrite (1 famille), et l'argument « pas de constante sans consommateur » est DÉFENDABLE, mais l'écart n'est nulle part déclaré ni re-routé : `DOMAIN_RUN_EVENT_V1` avait été explicitement routé vers F par le preflight de la Phase E, et il disparaît en silence.
- **[P2]** `crates/sbfb-workflow-core/src/pack_jcs_vectors.json:2` — A4 PARTIEL et prose FAUSSE : le commentaire du corpus de parité annonce des cas adverses (« exponent forms, negative zero ») que le fichier NE CONTIENT PAS. Les deux vecteurs RFC 8785 les plus discriminants nommés par le preflight (`1e+21`, `-0`) sont absents.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:147` — A10 PARTIEL / A11 sur-affirmé : l'exhaustivité au sens du preflight (« un fichier hors de tout digest fait échouer la vérification ») n'est pas livrée, et le doc affirme que « la vérification est une CONJONCTION — chaque digest contraint » alors qu'AUCUN chemin de code ne confronte un digest à quoi que ce soit.
- **[P2]** `crates/sbfb-workflow-core/src/pack.rs:2` — A14 NON APPLIQUÉ : les clauses §19.2 (7) « schémas `$ref` locales ou pinnées » et (9) « un import reste brouillon non signé jusqu'à revue et republication » ne sont ni explicitées ni routées nulle part dans le livrable — alors que (9) est un critère LITTÉRAL du gate que F ouvre.
- **[P2]** `crates/sbfb-workflow-core/src/pack_hostile_corpus.json:3` — A8 NON APPLIQUÉ et A13 PARTIEL : le corpus n'a AUCUNE entrée « duplicate JSON key » (pourtant exigée nommément par le plan F.2 et par A8), ni casse Windows, ni confusables Unicode, ni ratio de compression. La non-injectivité octets-reçus → octets-signés est réelle et non documentée.
- **[P2]** `crates/sbfb-workflow-core/src/contracts.rs:193` — A1 — l'arbitrage explicitement demandé par le preflight (durcir `schema_snapshot_stub` en STRICT) n'a pas été rendu : le golden reste FAIL-OPEN, alors même que la prose de census invoque désormais « pinned by contracts.schema.json » comme la décision D8 qui justifie le passage 25→26.
- **[P2]** `.planning/active/sprint83_plan.md:595` — Périmètre F.2 réduit sans mandat : le preflight conclut explicitement « rien n'est retiré : le périmètre est conservé et précisé », or trois livrables F.2 sont absents et non routés — le layout §9.2 / licences / limites, et le rapport `mapped/approximated/refused`.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:1158` — Deux références de chemin FAUSSES vers le miroir TS, dans le Rust et dans le corpus JSON.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:1467` — `hostile_corpus_refused` n'assert que la VARIANTE d'erreur, jamais quelle borne a mordu : quatre entrées distinctes (array/objet/chaîne/profondeur) partagent l'attendu `Unbounded`, alors que `PackError::Unbounded` porte `path` et `detail` exploitables.
- **[P3]** `crates/sbfb-workflow-core/src/pack.rs:724` — A18/A15 — la règle « pas de secret matérialisé », présentée comme ENFORCÉE par le format, comporte une exemption qui laisse passer la forme de secret la plus courante : 64 caractères hexadécimaux.

## Verdict

**FAIL** (au sens round-1 : correctifs requis avant commit). Deux dimensions FAIL
(honnêteté des garanties, patterns dépôt), six CONCERN. Les P1 dominants : SPDX absent
du fichier TS (gate CI rouge mesuré EXIT=1), 7e surface census non rafraîchie
(`sbfb-workflow-engine/src/lib.rs`), doc-comment localisant le test d'ordre dans le
mauvais module (gate `import_hostile` ne couvrait pas la propriété phare), classes de
corpus hostile nommées par F.2 absentes (clés JSON dupliquées, casse Windows,
confusables), digests non liés au lock (A10), domaine en paramètre libre de l'API
publique, prose sur-affirmée en plusieurs points (« exponent forms/negative zero »
annoncés non présents, « lossless », « rug-pull », « all signed payloads »).

Chaque finding est adressé (corrigé, durci ou documenté-avec-raison) dans la passe de
correctifs du 2026-07-21 ; la disposition finding-par-finding et la re-vérification
adversariale vivent dans le round 2.
