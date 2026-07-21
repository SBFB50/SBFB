Verdict externe Round 4 : **BLOQUÉ**. Aucun P0 démontré, mais un P1 cryptographique inédit invalide la formulation centrale « le détenteur de la clé a signé ces octets ». Le reste montre plusieurs garanties encore plus larges que les contrôles réels.

Aucune commande n’a été exécutée.

## Verdict par livrable

| # | Verdict | Evidence | Justification |
|---|---|---|---|
| 1 | **PARTIEL** | `pack.rs:190-205`, `pack.rs:235-295`, `pack.rs:323-374`, `pack.rs:802-914` | La racine, le lock, les six champs et `subject[]` existent. Seul `content` est recalculé. Les cinq autres digests restent des claims signées de forme valide, sans liaison rôle→fichier ni recalcul. La prose admet désormais qu’un pack peut pinner n’importe quel jeu de fichiers, mais continue ailleurs à parler de six préoccupations « couvertes » et disjointes. `contracts.schema.json` : **Not in bundle**. |
| 2 | **PARTIEL** | `canonical.rs:378-408`, `pack.rs:84-95`, `pack.rs:1422-1457` | Les quatre domaines D2 sont maintenant présents au bon endroit. Le cut précédent est fermé. Le miroir reste toutefois une vérification lexicale contournable, pas une preuve que les constantes compilées ne peuvent jamais diverger. |
| 3 | **CONFIRMÉ** pour l’ordre, pas pour la force cryptographique | `pack.rs:632-708`, `pack.rs:1945-1970` | L’ordre est bien taille → parse strict → bornes → JCS/null/I-JSON → signature → parse riche → sémantique. Le test d’ordre est discriminant. Mais l’étape Ed25519 utilise `Verifier::verify`, non la porte stricte, ce qui ouvre le P1 ci-dessous. |
| 4 | **PARTIEL** | `pack.rs:1578-1612`, `pack_digest.test.ts:35-112`, `pack_jcs_vectors.json:2` | Les deux côtés vérifient désormais le texte canonique et le digest sur dix vecteurs. Mais le miroir TS n’a pas la même frontière d’acceptation que Rust : il canonicalise `null`, les entiers non sûrs et les surrogates isolés que la porte Rust refuse. |
| 5 | **PARTIEL** | `pack.rs:2058-2105`, `pack.rs:2145-2369`, `pack_hostile_corpus.json:2` | Les entrées pré-signature lisent réellement leur `expect`. Pour 20 des 21 entrées post-signature, seule l’identité du nom est liée au JSON : le variant attendu demeure recopié en dur dans Rust. Modifier le `expect` du corpus laisse donc le gate vert. |
| 6 | **PARTIEL** | `check-frontier-contracts.sh:218-258`, `PATTERNS.md:4065-4092` | La correction vers **29**, la liste nominale et `LC_ALL=C sort -u` sont cohérentes sur les pièces fournies. L’ancien objectif 25→26 est bien supersédé par D2 : 25→29. Le recomptage réel de toutes les déclarations reste impossible, car `canonical.rs` complet et l’ensemble des crates ne sont pas dans le bundle. |

## Arbitrages demandés

- **Quatre domaines D2** : le défaut de scope du round 3 est fermé. Réserver les trois tags sans producteur est ici défendable, car D2 les gelait comme identifiants de protocole. Cela n’autorise cependant pas à prétendre qu’ils sont automatiquement « refusés partout ».

- **Emplacement et miroir** : garder `sbfb-workflow-core` indépendant du lourd `nexus-core-rs` est une bonne frontière. La duplication reste une nouvelle classe de dérive. Le test actuel détecte les changements ordinaires, pas tous les états compilés. La conception robuste est un petit crate feuille pur contenant uniquement les domaines, réexporté par `nexus-core-rs` et importé par `sbfb-workflow-core`.

- **Garde `null` locale** : la décision est cohérente. `canonical.rs` montre que la porte générique existante ne refuse pas `null`, tandis que `pack.rs:441-522` le fait localement. En revanche, `task.rs` et le chemin `Task → signature` sont **Not in bundle** : l’affirmation précise concernant `required_runtime` ne peut pas être revérifiée ici.

## Findings

### P0

Aucun P0 prouvé sur le bundle.

### P1 — bloquant

1. **La vérification Ed25519 n’est pas stricte : une clé publique de petit ordre peut rendre la preuve de possession fausse.**

   `verify_signed_pack_under_domain` appelle `verifying_key.verify(...)` (`pack.rs:688-697`) avec `ed25519-dalek 2.2.0` (`Cargo.lock:7515-7524`), et non `verify_strict`.

   Pour la clé publique encodant le point identité, la signature `R = identité, S = 0` satisfait l’équation de vérification pour n’importe quel message : aucune clé privée normale n’est nécessaire. C’est précisément la classe rejetée par la vérification stricte des points de petit ordre. `wrong_key_rejected` n’exerce que deux clés produites par `SigningKey`, donc jamais cette frontière.

   Cela contredit directement :

   - « the holder of one Ed25519 key … signed exactly these canonical bytes » (`pack.rs:12-18`) ;
   - la même garantie dans `canonical.rs:372-377`.

   Correctif précis :

   - remplacer `.verify(...)` par `.verify_strict(...)` ;
   - éventuellement refuser explicitement `verifying_key.is_weak()` ;
   - ajouter un test avec clé publique compressée `01 00…00` et signature `R=01 00…00, S=00…00`, qui doit produire `SignatureInvalid` sur un payload autrement valide.

   La sélection de clés de confiance reste externe, mais la validité mathématique de la clé ne peut pas être déléguée silencieusement au caller.

### P2

1. **Les cinq digests non-content ne “couvrent” encore rien que le signataire n’affirme lui-même.**

   `PackDigests::validate` ne contrôle que la syntaxe (`pack.rs:285-295`), puis `validate_pack` ne recalcule que `content` (`pack.rs:877-885`). `graph_semantic`, `presentation`, `compiled_plan`, `evaluation` et `publication` peuvent tous valoir `"00" × 32`.

   De plus, `content` est un digest parapluie du lock : toute modification d’un fichier de présentation modifie aussi le lock et donc `content`. La phrase « each covering a disjoint concern » (`pack.rs:190-205`) est littéralement fausse.

   Correctif : parler de « six signed digest claims », préciser que `content` recouvre tous les fichiers, et que les cinq autres ne sont vérifiés qu’après fourniture des artefacts concernés. Idéalement, introduire une seconde porte retournant un `MaterializedVerifiedPack`, avec égalité exacte du jeu de fichiers, rejet des extras/liens, vérification taille+BLAKE3 et recalcul des digests applicables.

2. **La parité TS ne reproduit pas la frontière canonique Rust.**

   `canonicalize` retourne `"null"` pour `null` et pour plusieurs valeurs JS non JSON (`pack_digest.test.ts:41-55`). Il ne rejette ni un surrogate UTF-16 isolé ni un entier non sûr. Rust refuse ces classes avant signature (`pack.rs:441-522`).

   Ainsi :

   - Rust refuse `{"x":"\ud800"}`, tandis que JS peut le parser puis le sérialiser en escape ;
   - Rust refuse un entier `2^53`, tandis que TS le canonicalise ;
   - Rust refuse `null`, TS produit `null`.

   Le cas lone-surrogate ajouté au corpus ne teste que Rust.

   Correctif : ajouter au miroir TS les mêmes gardes `null`/safe-integer/Unicode-scalar et un corpus partagé d’acceptation **et de refus**. Ne pas présenter cette utility de test comme une future porte navigateur complète tant que doublons JSON, bornes et signature ne sont pas également traités.

3. **Les `expect` post-signature du corpus sont presque décoratifs.**

   `valid_signature_over_hostile_body_is_terminal` construit ses propres tuples `(name, value, expect, fragment)`. L’égalité avec le corpus ne compare que les noms (`pack.rs:2337-2349`). Seul `cross-domain-signature` lit réellement son `expect`.

   Correctif : construire une map `name → {expect, expect_detail_contains}` depuis le JSON et utiliser ces valeurs pendant chaque assertion. Ajouter un roster ou un plancher explicite de 21 entrées post-signature pour empêcher une suppression simultanée dans le JSON et dans le test.

4. **Le binding `content ↔ lock` est correct en production mais insuffisamment prouvé.**

   `valid_pack` calcule `content` avec le même helper que le vérificateur. Une mutation de production remplaçant `content_digest_of_lock(lock)` par le digest du seul nombre d’entrées — voire une constante — laisserait vraisemblablement les 44 tests verts : baseline et vérification partageraient la même erreur, et le cas mismatch resterait différent.

   Correctif : test indépendant avec golden littéral, puis mutations séparées de `path`, `blake3`, `size_bytes`, ajout/suppression et ordre ; chaque mutation doit changer `content_digest_of_lock`.

5. **Le scanner peut encore recopier un secret présent dans le path ou la query d’une URL.**

   `redact_userinfo` ne masque que `user:pass@host` (`pack.rs:1107-1161`). Une description telle que :

   ```text
   see https://api.example/callback?token=sk-secret
   ```

   produit un finding contenant le token. `looks_like_materialised_secret` ne mord pas, car la chaîne entière ne commence pas par `sk-`. Aucun test ne pinne même la rédaction userinfo actuelle (`scanner_is_advisory_not_a_gate`, `pack.rs:1810-1845`).

   Correctif : ne jamais recopier l’endpoint hostile ; retourner seulement le chemin du champ et « endpoint http(s) détecté ». À défaut, conserver uniquement scheme+host et supprimer systématiquement userinfo, path, query et fragment. Ajouter des tests pour userinfo >256 caractères, query token et fragment token.

6. **Le miroir lexical peut encore passer sur une constante active divergente.**

   Exemple passant son intention actuelle :

   ```rust
   /*
   pub const DOMAIN_WORKFLOW_PACK_V1: &[u8] = b"nexus-workflow-pack-v1";
   */
   pub const DOMAIN_WORKFLOW_PACK_V1 : &[u8] = b"drifted";
   ```

   Le needle exact n’apparaît qu’une fois, la ligne du bloc commence par `pub const`, et l’ancien texte est comparé au miroir. Une déclaration sous `#[cfg(any())]` pose le même problème.

   Le caveat « lexical check » est honnête (`pack.rs:84-95`), mais « proves the two never drift apart » dans `canonical.rs` reste faux.

   Correctif : petit crate d’autorité commun. À défaut, parser l’AST Rust et ajouter une référence compilée au symbole dans les tests de `nexus-core-rs`.

7. **Une collision canonique reste volontairement admise : `-0` et `0`.**

   Le golden prouve lui-même que `-0.0` devient `0`. `reject_nulls` ne refuse pas le zéro négatif. Deux valeurs que Rust ou JavaScript peuvent distinguer par le signe produisent donc la même préimage JCS.

   Ce n’est pas une collision BLAKE3 : c’est le quotient numérique imposé par JCS. Elle devient dangereuse si le futur graphe traite le signe de zéro ou distingue entier/flottant.

   Correctif : soit refuser explicitement le zéro négatif dans cette porte, soit geler maintenant la règle normative « les consommateurs ne peuvent pas distinguer les représentations numériques que JCS identifie ». NFC/NFD restent distincts ; aucune collision Unicode de normalisation n’est visible ici.

8. **Les domaines réservés portent une garantie de refus que les constantes ne peuvent pas assurer.**

   La documentation de `DOMAIN_COMPILED_PLAN_V1` affirme qu’une signature sous ce domaine « is refused wherever it shows up » (`canonical.rs:384-390`). Un tag ne refuse rien : n’importe quel code peut signer sous ces octets, et aucun consommateur complet n’est fourni pour prouver le refus universel.

   Correctif : écrire seulement « aucun producteur ni vérificateur typé in-tree n’est défini dans ce bundle ; cette constante seule ne confère aucune acceptation ».

### P3

1. `pack_jcs_vectors.json:2` affirme encore que `digest_for` ne borne pas les entiers. C’est devenu faux depuis que la règle I-JSON a été hissée dans `canonical_bytes`. Corriger le commentaire.

2. `MAX_JSON_DEPTH` et `MAX_COLLECTION_LEN` prétendent couvrir « l’enveloppe » (`pack.rs:104-129`), mais `enforce_bounds` ne reçoit que `payload_value` (`pack.rs:670-678`). Les membres inconnus sont entièrement parsés puis meurent sur `MalformedEnvelope`. Remplacer « envelope » par « payload », ou borner dans le visitor structurel.

3. Le roster de `canonical.rs:77-82` dit que les trois payload contracts réservés existent dans `contracts.rs`. C’est faux pour `PackInstallReceipt.v1`. Corriger en « deux identités réservées + un contrat non encore défini ».

4. `PackDigestKind::GraphSemantic` parle toujours de nodes/edges « by stable id » (`pack.rs:196-201`), alors que les tableaux ne sont pas triés par id. Reformuler : l’ordre des tableaux reste significatif jusqu’au contrat typé du graphe.

5. Le module pack emploie encore « published exactly these bytes » (`pack.rs:12-18`), alors que `canonical.rs` a correctement remplacé cela par « signed » et « not a publication event ». Aligner les deux.

6. Les caps total/path/segments ne possèdent toujours pas de tests discriminants, et `PackDigestKind::ALL` peut omettre un nouveau variant si les matches sont simultanément adaptés. Ajouter les mutations ciblées décrites par la review interne.

7. `scripts/check-frontier-contracts.sh:8` annonce toujours « Five deterministic checks » alors que l’echo final en énumère sept. Corriger l’en-tête et indiquer simplement « Bash + utilitaires BusyBox compatibles ».

## Not in bundle

Impossible de confirmer sur pièces :

- `contracts.schema.json` ;
- `task.rs` et le chemin réel `Task::required_runtime → préimage signée` ;
- `canonical.rs` complet, l’extrait contenant `[...]` ;
- le kickoff, le plan et le préflight originaux ;
- les reviews r1/r2 ;
- `Cargo.lock`, `web/package.json` et `package-lock.json` complets ;
- les configurations Vitest, ESLint, TypeScript et coverage ;
- les autres consommateurs des schémas/fixtures ;
- le sweep repo-wide des anciennes proses 25/26 ;
- les résultats de commandes annoncés et le futur commit body.

## Verdict global

**BLOQUÉ / FAIL.**

Les quatre domaines et l’ordre de vérification sont désormais correctement structurés, et les corrections de collision de présentation, d’I-JSON et de census sont réelles sur les pièces. Mais l’usage de la vérification Ed25519 non stricte laisse une signature universelle possible sous une clé de petit ordre et rend fausse la garantie cryptographique centrale. Cette P1 doit être fermée et pinnée par un test avant tout PASS.

Les P2 sur la portée réelle des cinq digests, le miroir TS, le corpus post-signature et le contrat de matérialisation doivent au minimum recevoir une correction ou une disposition explicite avant que 30 phases puissent traiter `Ok(pack)` comme une frontière de confiance.