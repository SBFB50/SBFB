Verdict externe : **BLOQUÉ**. L’ordre cryptographique central est correctement construit, mais plusieurs garanties de format sont plus fortes que ce que le vérificateur établit réellement. Aucun P0 prouvé ; cinq P1 bloquants.

## Verdict par livrable

| # | Verdict | Evidence | Justification |
|---|---|---|---|
| 1 | **PARTIEL** | `pack.rs:338-389`, `pack.rs:720-800` | La racine, le lock, les six champs et `subject[]` existent. Mais seul `digests.content` est confronté au lock ; les cinq autres sont des affirmations signées non vérifiées. Aucun workflow, agent ou prompt n’est requis dans le lock. `contracts.schema.json` : **Not in bundle**. |
| 2 | **GAP** | `pack.rs:75-83`, `pack.rs:1274-1300`, `canonical.rs:338-369`, `sprint83_kickoff.md:285-288`, `sprint83_plan.md:590-594` | Le domaine pack existe au bon endroit, mais trois des quatre domaines gelés manquent. Le test miroir est un scrape lexical contournable et ne prouve pas l’absence de dérive. |
| 3 | **CONFIRMÉ** | `pack.rs:579-636`, `pack.rs:1696-1720` | L’ordre statique est bien : taille → parse structurel strict → bornes payload → JCS/null → Ed25519 → parse riche → sémantique. `signature_before_parse` est réellement discriminant pour l’ordre signature/parse riche. |
| 4 | **PARTIEL** | `pack.rs:1395-1448`, `pack_digest.test.ts:31-112`, `pack_jcs_vectors.json:1-60` | Les deux langages lisent les mêmes dix vecteurs et le discriminant UTF-16 est présent. Ils comparent cependant des digests BLAKE3, pas les octets canoniques eux-mêmes ; la couverture Vitest/TSConfig/ESLint est **Not in bundle**. |
| 5 | **PARTIEL** | `pack_hostile_corpus.json:3-211`, `pack.rs:1770-2020` | Les 16 entrées pré-signature meurent bien sur le variant annoncé. Plusieurs cas post-signature restent verts si leur contrôle spécifique est supprimé, à cause du fallback `content digest does not match PACK.lock`. |
| 6 | **PARTIEL** | `check-frontier-contracts.sh`, bloc `(2b)`, `PATTERNS.md:4065-4088` | Les surfaces fournies sont cohérentes à 26. Le décompte réel ne peut pas être recompté puisque l’arbre Rust complet est **Not in bundle** ; surtout, quatre domaines gelés auraient conduit à 29, pas 26. |

## Arbitrages demandés

### Une famille DOMAIN au lieu de quatre

C’est un **scope cut**, pas un durcissement.

Le kickoff gelé prescrit explicitement quatre rôles (`sprint83_kickoff.md:285-288`), le plan les place dans F (`sprint83_plan.md:590-594`) et le préflight B1 exige de fermer la liste avant édition, avec les quatre comme défaut. Une constante de domaine est une réservation de protocole, pas du « code mort » comportemental. Elle peut être documentée sans promesse future interdite par `PROMISE_RE`.

Le body H/I/X rendrait le cut visible, mais un body de commit n’amende ni D2 ni le plan canonique. Deux voies valides :

- ajouter maintenant les trois constantes, leurs décisions D8 et rafraîchir le census 26→29 ;
- obtenir une modification formelle et autorisée du kickoff/plan/preflight. Le body seul est insuffisant.

### Emplacement et miroir

Ne pas dépendre en production du lourd `nexus-core-rs` est défendable. La duplication actuelle ne l’est pas comme « preuve ».

`pack_signing_domain_mirrors_canonical_rs` cherche le premier texte exact :

```rust
pub const DOMAIN_WORKFLOW_PACK_V1: &[u8] = b"
```

Conséquences :

- une ligne commentée contenant l’ancien littéral avant la vraie constante fait passer le test malgré une dérive réelle ;
- commenter la constante tout en laissant son texte dans le fichier peut passer ;
- un simple changement d’espacement produit un faux rouge ;
- le premier de deux littéraux gagne, sans vérifier l’unicité ni qu’il s’agit d’un item Rust actif.

Correctif préféré : petit crate pur d’autorité pour les domaines. À défaut, test par dépendance de développement si aucun cycle, ou parsing AST avec `syn`, item actif unique et valeur littérale évaluée.

### Garde `null` locale

La décision locale est cohérente. `task.rs:315-316` montre un `Option<RuntimeTuple>` sans `skip_serializing_if`, et `canonical.rs` ne rejette pas `null`.

Cependant, le chemin exact `Task → préimage signée` et le derive de `Task` sont **Not in bundle**. Je peux confirmer que `None` serait sérialisé en `null` par le chemin Serde normal, mais pas vérifier sur pièces que le chemin actuel de signature n’utilise aucune projection intermédiaire. Il ne faut donc pas écrire que cette affirmation a été intégralement revérifiée ici.

## Findings

### P0

Aucun P0 prouvé sur le bundle.

### P1 — bloquants

1. **Cinq “digests sémantiques” ne vérifient aucune sémantique.**

   `validate_pack` ne recalcule que `content` (`pack.rs:782-790`). `graph_semantic`, `presentation`, `compiled_plan`, `evaluation` et `publication` peuvent être n’importe quelles chaînes hexadécimales de 64 caractères.

   Un pack signé peut aussi ne contenir que `README.txt` : aucune règle n’exige `workflow/workflow.json`, un agent, un prompt, un schéma ou une évaluation. Le texte « Carries the workflow graph, agent definitions, prompts… » est donc plus fort que le type accepté.

   Correctif : définir des références typées rôle→chemin de lock, exiger les artefacts obligatoires et fournir une porte pure recevant `path → bytes`. Elle doit vérifier l’égalité exacte du jeu de fichiers, tailles, BLAKE3 et six digests avant de produire un type distinct `MaterializedVerifiedPack`. Tant que cela n’existe pas, nommer les cinq valeurs « signed digest claims », pas « verified semantic digests ».

2. **Collision réelle dans `presentation_view`.**

   `pack.rs:1072-1102` affirme qu’un chemin textualisé ambigu est « fine for a digest input ». C’est faux : `Map::insert` écrase une entrée antérieure.

   Ces deux documents distincts ont la même vue sémantique et la même vue de présentation :

   ```json
   {"a":{"b":{"x":1}},"a.b":{"x":2}}
   ```

   ```json
   {"a":{"b":{"x":999}},"a.b":{"x":2}}
   ```

   Dans les deux cas, `graph_semantic_view` retire les deux `x`. Dans `presentation_view`, les deux chemins deviennent `$.a.b.x` et la valeur sous la clé littérale `a.b` écrase l’autre.

   De plus, la fonction ne trie ni nodes ni edges par identifiant : réordonner deux nodes change le digest alors que la documentation parle de « meaning … by stable id ».

   Correctif : conserver une projection arborescente, ou encoder les chemins comme tableaux de segments typés/JSON Pointer correctement échappé. Ajouter exactement la paire ci-dessus comme régression. Retirer toute claim de canonicalisation sémantique avant le schéma typé de graphe et son ordre total.

3. **Les tests post-signature restent verts quand les protections de chemins disparaissent.**

   Dans `valid_signature_over_hostile_body_is_terminal`, toute mutation du lock laisse `digests.content` calculé sur l’ancien lock. Si `validate_lock_path`, l’unicité, la casse, le trailing-dot ou le refus du lock vide sont supprimés, le contrôle suivant produit encore `PackError::Semantic` via le mismatch de content. Le test ne vérifie que le variant `Semantic`.

   Sont concernés : zip-slip, backslash, absolu, drive letter, device DOS, doublons exacts/de casse, trailing-dot, confusable et lock vide. Les cas digest tronqué/majuscule ont le même problème puisqu’ils mutent précisément `content`.

   Correctif : après chaque mutation de lock, recalculer `digests.content`, sauf dans le cas dédié au mismatch. Exiger ensuite un fragment d’erreur propre à chaque règle. Pour tester la syntaxe d’un digest, muter `graph_semantic`, pas `content`.

4. **Le cut des trois domaines contredit D2 gelé.**

   Evidence : `sprint83_kickoff.md:285-288`, `sprint83_plan.md:590-594`, préflight §8.1 B1. Les routes inscrites dans le body documenteraient la dette, mais ne rendraient pas le livrable conforme.

   Correctif : ajouter les trois domaines et passer le census à 29, ou obtenir une modification formelle de l’autorité gelée avant PASS.

5. **Les garanties sur secrets et rendu sont fausses, avec fuite dans l’erreur.**

   Le champ `secret_refs` promet « Never a secret », alors que la règle reconnaît explicitement qu’un secret hexadécimal ou fractionné passe. Surtout, `validate_pack` n’analyse que `secret_refs`. Un secret ou une URL credentialisée dans `description`, `name`, `builder_id` ou un subject est accepté. `scanner_is_advisory_not_a_gate` prouve volontairement qu’une description contenant une URL passe la vérification.

   La section Media dit également « nothing to render », alors que `name`, `description`, `builder_id` et les sujets sont précisément des métadonnées susceptibles d’être rendues.

   Enfin, le refus d’un `secret_ref` réinjecte intégralement la valeur hostile dans l’erreur (`pack.rs:793-800`), donc potentiellement dans les logs.

   Correctif :

   - retirer « Never a secret » et « nothing to render » ;
   - dire que toutes les métadonnées restent hostiles et exigent un rendu échappé ;
   - ne jamais recopier un secret détecté dans l’erreur : index/chemin + message générique ou empreinte tronquée ;
   - si l’interdit D2 doit réellement être dur, supprimer les champs texte libres de cette frontière ou formaliser que l’interdit porte seulement sur les références exécutables. Un scanner heuristique ne peut pas garantir l’absence d’un secret dans une chaîne arbitraire.

### P2

1. **Le miroir “ne peut pas dériver” est une fausse preuve.**

   Evidence : `pack.rs:75-83`, `pack.rs:1274-1300`, `PATTERNS.md:4078-4083`. Corriger l’architecture ou, au minimum, le test et la prose.

2. **Les helpers publics gardent des collisions numériques que `verify_signed_pack` refuse.**

   `canonical_bytes`, `digest_for` et `content_digest_of_lock` n’appellent pas `enforce_bounds` (`pack.rs:421-435`, `pack.rs:1153-1159`). Deux locks construits en Rust avec `size_bytes = 2^53` et `2^53+1` peuvent donc recevoir la même forme JCS arrondie. De même, JCS canonise `-0.0` et `0` en `0`.

   Correctif : faire de la porte canonique locale la porte I-JSON complète, y compris pour les appels directs. Soit rejeter `-0`, soit documenter explicitement que les digests travaillent sur le quotient JCS où `-0 == 0`. Ajouter les deux collisions comme tests directs.

3. **Les bornes de forme arrivent après la matérialisation complète du DOM.**

   Un tableau proche de 1 MiB contenant environ 500 000 petites valeurs est entièrement alloué par `parse_rejecting_duplicate_keys`, puis seulement refusé à 512. Un membre d’enveloppe inconnu subit aussi ce parse complet sans passer dans `enforce_bounds`.

   C’est borné par 1 MiB, donc pas un DoS sans limite, mais l’amplification mémoire est réelle. `MAX_STRING_LEN` est en outre documenté pour « any string in the envelope » alors qu’il ne couvre ni la signature ni les membres inconnus.

   Correctif : appliquer profondeur, cardinalité et longueur directement dans le visitor. Vérifier `signature.len() == 128` avant `hex_decode`.

4. **Le format dit “portable” mais accepte des chemins non matérialisables.**

   `validate_lock_path` accepte un segment de presque 8 KiB ou plusieurs milliers de segments (`pack.rs:832-889`). Aucun plafond par segment, nombre de segments ou longueur portable n’existe. Les `size_bytes` peuvent aussi totaliser plusieurs exaoctets.

   Correctif : plafond par segment, nombre de segments, longueur totale portable, taille par fichier et taille totale via `checked_add`.

5. **La parité affirme des octets identiques mais ne compare que BLAKE3.**

   Pour un test portant littéralement sur la préimage de signature, stocker aussi `canonical_hex` ou `canonical_utf8` dans le golden et l’asserter dans les deux langages. Ajouter un cas de surrogate isolé attendu en refus I-JSON. NFC et NFD ne sont actuellement pas normalisés : ils restent distincts, donc aucune collision de normalisation n’est prouvée.

6. **Le census gèle un nombre, pas le registre.**

   Un retrait et un ajout simultanés passent à compteur constant ; une fausse constante commentée peut aussi être comptée par le grep. Le script le reconnaît partiellement.

   Correctif : geler la liste triée exacte des noms dans un golden et extraire uniquement des déclarations Rust actives.

7. **L’enveloppe signée n’est pas dans le registre de schémas.**

   `all_contract_schemas` enregistre `SbfbPromptAgentPack`, pas `SignedPack`. La forme réellement reçue par `verify_signed_pack` n’a donc pas de schéma contractuel gelé.

   Correctif : produire et geler aussi le schéma de l’enveloppe sans modifier le registre des sept identités, par exemple comme schéma compagnon ou `$defs`.

8. **Fausse attribution dans `canonical.rs`.**

   « A valid signature proves who published » est trop fort. Elle prouve qu’une clé privée correspondant à la clé fournie a signé la préimage ; ni identité, ni publication, ni acceptabilité de la clé.

   Correctif : reprendre exactement la formulation honnête du module pack.

### P3

1. `PackLock.entries` est documenté « sorted by the producer », mais aucun ordre n’est vérifié. Enforcer un ordre byte-lexicographique strict ou retirer la claim.

2. Le corpus ne vérifie pas l’unicité de ses noms ; des doublons pourraient rembourrer les planchers. `cross-domain-signature.expect` reste décoratif. Ajouter une assertion d’unicité et faire lire mécaniquement le `expect` cross-domain.

3. `digest_kinds_are_exhaustive_and_named` ne détecte pas un nouveau variant oublié dans `ALL`. Ajouter une assertion structurelle indépendante ou générer l’énumération depuis une source unique.

## Routage réel des 16 entrées pré-signature

Aucune ne tombe accidentellement sur `SignatureInvalid` :

| Entrée | Premier contrôle effectif |
|---|---|
| `not-json` | parser strict → `MalformedEnvelope` |
| `envelope-is-array` | `as_object` → `MalformedEnvelope` |
| `envelope-extra-member` | allowlist enveloppe → `MalformedEnvelope` |
| `envelope-no-signature` | extraction signature → `MalformedEnvelope` |
| `envelope-signature-not-string` | `as_str` → `MalformedEnvelope` |
| deux duplicate-key | visitor strict → `MalformedEnvelope` |
| deux null | `canonical_bytes/reject_nulls` → `NonRepresentable` |
| deux entiers hors 2^53 | `enforce_bounds` → `Unbounded` |
| array/object/string/depth | `enforce_bounds` → `Unbounded` |
| oversized envelope | `raw.len()` → `TooLarge` |

Cette partie du corpus est solide. Le défaut de discriminance concerne les cas sémantiques signés décrits en P1-3.

## Not in bundle

Je ne peux pas confirmer sur pièces :

- `contracts.schema.json`, explicitement annoncé mais absent ;
- le `Cargo.lock` complet, seulement son diff ;
- `canonical.rs` complet, remplacé par deux extraits avec `[...]` ;
- `web/package.json` et `package-lock.json` complets ;
- les configurations Vitest, ESLint, TypeScript et coverage ;
- les autres consommateurs de `contracts.schema.json` ou des fixtures ;
- le sweep complet des anciennes proses « 25 familles » ;
- `rounds/sprint83_phase_f_review_r1.md` ;
- le chemin complet de signature de `Task` autour de `task_canonical_bytes`.

Les cinq déclarations prévues dans le body sont utiles et le nombre statique de tests `import_hostile` est bien cinq. Elles ne corrigent toutefois ni les P1 de code, ni la contradiction d’autorité D2.

**Verdict global : BLOQUÉ / FAIL. Phase F établit une bonne enveloppe signée et un bon ordre de vérification, mais pas encore un `SBFBPromptAgentPack.v1` dont les six digests, le contenu workflow et les garanties de sécurité correspondent à la prose publiée.**