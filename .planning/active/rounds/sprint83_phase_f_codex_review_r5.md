Je n’ai exécuté aucune commande. Verdict externe : **BLOQUÉ**, avec **0 P0, 2 P1 nouveaux, 5 P2 et 3 P3**.

Le bundle inline n’a pas de numéros de ligne pour les gros fichiers ; j’utilise donc des ancres symboliques exactes (`fichier:fonction/test`) plutôt que d’inventer des numéros. Quand elles sont déterminables, j’indique les lignes.

## Verdict par livrable

| # | Verdict | Évidence | Justification |
|---|---|---|---|
| 1 | **PARTIEL** | `pack.rs:SbfbPromptAgentPack`, `PackLock`, `PackDigests`, `PackProvenance`, `validate_pack` | La racine est effectivement remplie, le lock est signé, `content` est recalculé et `subject[]` est présent. Mais seuls les fichiers **déclarés** sont engagés ; l’exhaustivité du jeu matérialisé, les types de fichiers et les métadonnées ne sont pas établis. Cinq des six digests restent des claims syntaxiques. |
| 2 | **CONFIRMÉ** | `canonical.rs:300-490`, `pack.rs:PACK_SIGNING_DOMAIN`, `tests::pack_signing_domain_mirrors_canonical_rs` | Le domaine pack vaut bien `nexus-workflow-pack-v1` aux deux endroits. Le tree actuel contient aussi les trois domaines D2 réservés. Le test de miroir ferme les dérives accidentelles usuelles, mais reste lexical. |
| 3 | **CONFIRMÉ** | `pack.rs:verify_signed_pack_under_domain`, `import_hostile::signature_before_parse` | L’ordre réel est bien : taille → parse structurel/dup-keys → bornes → canonicalisation → signature Ed25519 stricte → parse riche → sémantique. Le test d’ordre est discriminant. |
| 4 | **PARTIEL** | `pack.rs:tests::jcs_cross_lang_parity`, `pack_digest.test.ts:24-80` | Le harness partagé est réel, mais la frontière de refus TS diverge encore pour les surrogates isolés dans les **clés**. De plus, `pack_jcs_digests.golden.json` est **Not in bundle**, donc les octets/digests attendus ne sont pas auditables ici. |
| 5 | **CONFIRMÉ** | `pack_hostile_corpus.json`, `pack.rs:import_hostile` | Les entrées sont énumérées, les variants sont comparés exactement, les détails discriminent les règles et les cas post-signature sont reliés mécaniquement au corpus. « 100 % » n’est vrai que pour ce corpus énuméré, pas pour l’espace hostile général. |
| 6 | **PARTIEL** | `check-frontier-contracts.sh:§2b`, `canonical.rs:DOMAIN_*` | Le tree fourni est désormais à **29**, pas 26 : les quatre rôles D2 sont déclarés. Le gate gèle les noms avec un tri `LC_ALL=C`. Mais le census live complet et `docs/rust/PATTERNS.md §P70` sont **Not in bundle**. |

## Réponse aux trois écarts prioritaires

1. **Une famille contre quatre : problème résolu dans ce tree.**  
   `canonical.rs` déclare désormais `DOMAIN_WORKFLOW_PACK_V1`, `DOMAIN_COMPILED_PLAN_V1`, `DOMAIN_RUN_EVENT_V1` et `DOMAIN_PACK_INSTALL_RECEIPT_V1`. Ce n’est donc plus un scope cut : l’implémentation actuelle suit la décision D2 telle que décrite dans le module. Le kickoff demandé est toutefois **Not in bundle**, donc je ne peux pas vérifier directement `kickoff:316-322`.

2. **Le miroir est défendable comme compromis de dépendances, mais ce n’est pas une preuve compilée.**  
   Le test échoue bien si le littéral principal change d’espacement, disparaît ou si une seconde copie exacte apparaît. En revanche, il ne parse pas Rust : un bloc commentaire contenant une ligne nue `pub const...`, une déclaration sous `#[cfg(any())]`, ou une déclaration compilée alternative formatée différemment peut tromper l’extraction. `canonical.rs` reconnaît honnêtement cette limite ; le commentaire « PROVEN » dans le test est plus fort que ce qui est obtenu. La meilleure architecture reste un micro-crate sans dépendances contenant uniquement les domaines, réexporté par `nexus-core-rs`.

3. **Porte locale : cohérente, mais justification `Task` non vérifiable.**  
   Le bundle confirme que la porte pack refuse `null` et les entiers hors plage, alors que `nexus_core_rs::canonical_bytes` ne le fait pas. En revanche, `crates/nexus-core-rs/src/task.rs` est **Not in bundle** : je ne peux donc pas confirmer moi-même que `Task::required_runtime` signe actuellement un `null`.

## Findings

### P0

Aucun.

### P1 — bloquants

#### P1-1 — La frontière TS accepte un surrogate isolé dans une clé JSON

Évidence : `web/src/workflow/__tests__/pack_digest.test.ts:40-47,56-59,66-79`.

Le contrôle des surrogates n’est exécuté que lorsque la **valeur** est une chaîne. Dans la branche objet :

```ts
const keys = Object.keys(record).sort();
return `{${keys.map((k) => `${JSON.stringify(k)}:${canonicalize(record[k])}`).join(",")}}`;
```

`k` n’est jamais validé. Contre-exemple exact :

```ts
const value = JSON.parse('{"\\ud800":1}');
canonicalize(value); // ne jette pas
```

`JSON.stringify(k)` échappe le surrogate et produit une sortie, tandis que le parseur Rust refuse ce texte comme `MalformedEnvelope`. La phrase « lone UTF-16 surrogates all throw here just as they are rejected there » est donc fausse, et le test vert ne couvre que les surrogates placés dans des valeurs.

Correctif :

- extraire `assertUnicodeScalarString`;
- l’appeler sur chaque clé avant le tri et sur chaque valeur chaîne ;
- ajouter les cas clé high-surrogate, clé low-surrogate et clé imbriquée ;
- ajouter un générateur Rust `lone_surrogate_key` au corpus hostile.

#### P1-2 — Le scanner peut encore recopier un secret placé dans le hostname

Évidence : `pack.rs:collect_findings`, `pack.rs:scheme_and_host`, `tests::scanner_never_echoes_url_secrets`.

Contre-exemple accepté :

```text
https://sk-prod-ABC123.attacker.example/path
```

`scheme_and_host` retourne :

```text
https://sk-prod-ABC123.attacker.example
```

Le secret est donc reproduit dans `ScanFinding.detail`. Le test actuel ne couvre que userinfo, query, fragment et port. Or les tokens placés dans un sous-domaine sont courants, notamment pour l’exfiltration DNS et les URLs par locataire.

Cela réfute littéralement « NEVER reproduce a secret carried anywhere in a URL ».

Correctif précis : ne recopier **aucune partie de l’autorité contrôlée par le pack**. Retourner seulement un détail statique, par exemple `declares an https endpoint`, ou un hash non réversible du host. Ajouter des tests avec secret dans hostname, hostname Unicode/punycode et IPv6.

### P2

#### P2-1 — `validate_lock_path` ne garantit pas le confinement à la matérialisation

Évidence : `pack.rs:PackLockEntry`, `pack.rs:validate_lock_path`.

La validation ferme correctement `..`, `%`, Unicode, backslashes, ADS et noms DOS. Mais un chemin lexicalement valide comme `workflow/workflow.json` sort de la racine si `workflow` est un symlink, une junction ou un reparse point préexistant. Le lock ne signe pas non plus le type de l’entrée, les permissions, xattrs ou liens durs.

La phrase « Reject a lock path that could escape the pack root when it is materialised » est donc trop forte.

Correctif :

- écrire « refuse les échappements lexicaux uniquement » ;
- imposer normativement « fichiers réguliers seulement » ;
- matérialiser dans un répertoire neuf ;
- refuser symlinks, hardlinks, devices et reparse points ;
- ouvrir les composants en no-follow/resolve-beneath ;
- vérifier digest et taille avant renommage atomique ;
- refuser toute entrée absente ou supplémentaire par rapport au lock.

#### P2-2 — La borne de forme intervient après l’amplification mémoire

Évidence : `pack.rs:parse_rejecting_duplicate_keys`, `verify_signed_pack_under_domain`, `enforce_bounds`.

Le visiteur matérialise tout le `Value` avant que la cardinalité 512 soit examinée. Une enveloppe de moins de 1 MiB peut contenir plusieurs centaines de milliers de scalaires courts et allouer bien davantage que « times a small constant », surtout avec les allocations de map.

Correctif : faire respecter profondeur, longueur des chaînes, cardinalité et idéalement nombre total de nœuds directement dans le visiteur structurel, avant `push`/`insert`.

#### P2-3 — Les nouvelles bornes lock ne sont pas protégées par des tests discriminants

Évidence : `pack.rs:MAX_LOCK_PATH_BYTES`, `MAX_PATH_SEGMENTS`, `MAX_SEGMENT_BYTES`, `MAX_TOTAL_SIZE_BYTES` et module `import_hostile`.

Aucun cas fourni ne cible :

- chemin à 513 octets ;
- 33 segments ;
- segment à 256 octets ;
- total déclaré supérieur à 2 GiB avec chaque fichier sous son plafond ;
- acceptation aux plafonds exacts correspondants.

Retirer ces règles laisserait les 47 tests présentés verts.

Correctif : ajouter quatre entrées post-signature au corpus, recalculer `digests.content`, vérifier le fragment exact, puis ajouter les quatre contrôles `cap`/`cap+1`.

#### P2-4 — Plusieurs erreurs recopient des données hostiles non échappées

Évidence : `pack.rs:verify_signed_pack_under_domain`, `parse_rejecting_duplicate_keys`, `validate_lock_path`, `validate_pack`.

Les messages recopient notamment le nom d’un membre d’enveloppe, une clé dupliquée, un chemin ou un nom de subject. Une clé comme `"\u001b[31mFAUX\nPASS"` peut injecter des contrôles dans les logs avant même la signature.

Correctif : erreurs structurées avec raison statique et chemin échappé/cappé ; ne jamais inclure directement une valeur hostile. Ajouter des tests négatifs pour newline, ESC et clé proche de la limite brute.

#### P2-5 — Le détecteur de secret refuse des références logiques légitimes

Évidence : `pack.rs:looks_like_materialised_secret`.

La prose affirme que les deux formes détectées ne sont « never slot names », mais aucune grammaire des slots ne l’établit. Par exemple :

```text
ProdTenantCredentialReferenceSlot2026Alpha9
```

est une référence logique plausible, mais elle est longue, alphanumérique et mélange majuscules/minuscules/chiffres : elle est refusée terminalement.

Correctif : soit définir une vraie grammaire normative des références et l’exprimer aussi dans le schema, soit rendre cette heuristique uniquement consultative comme le scanner.

### P3

#### P3-1 — Commentaire JCS devenu faux après le correctif entier

Évidence : `pack_jcs_vectors.json:2`.

Le commentaire dit encore :

> `digest_for alone does not bound integers`

C’est faux : `digest_for` appelle `canonical_bytes`, qui appelle maintenant `reject_nulls`, lequel refuse également les entiers hors I-JSON.

Correctif : écrire que la porte Rust entière — vérification **et** `digest_for` — refuse les tokens entiers hors `±(2^53−1)`, tandis que JS ne peut plus retrouver cette distinction après `JSON.parse`.

#### P3-2 — Le test miroir reste un tripwire lexical, pas une non-dérive absolue

Évidence : `pack.rs:tests::pack_signing_domain_mirrors_canonical_rs`, `canonical.rs:300-490`.

Un bloc commentaire ou une déclaration `cfg` peut contourner l’intention. La documentation de `canonical.rs` l’admet, mais « PROVEN » et « uncommented » restent sur-affirmés ailleurs.

Correctif : micro-crate commun sans Tokio/iroh, ou parsing syntaxique d’item Rust avec contrôle des attributs `cfg`.

#### P3-3 — Le scanner ne recense qu’un endpoint et tronque les IPv6

Évidence : `pack.rs:collect_findings`, `scheme_and_host`.

`find("http://").or_else(find("https://"))` préfère n’importe quel HTTP ultérieur à un HTTPS antérieur et ne rapporte qu’une URL par chaîne. `host.split(':')` transforme `[2001:db8::1]` en `[2001`.

Le correctif P1-2 — ne plus recopier le host — supprime le risque de fuite ; un vrai parseur URL reste nécessaire si la détection fonctionnelle doit être exacte.

## Éléments Not in bundle

Les éléments suivants étaient annoncés mais ne figurent pas dans l’arbre inline :

- `contracts.schema.json` ;
- `pack_jcs_digests.golden.json` ;
- `pack_digests.golden.json` ;
- `signed_pack.schema.json`, pourtant lu par un test ;
- `Cargo.toml` et le diff `Cargo.lock` ;
- `web/package.json` et les configurations Vitest/ESLint/TypeScript/coverage ;
- `docs/rust/PATTERNS.md §P70` ;
- preflight, plan, kickoff, review canonique et rounds r1/r2/r3 ;
- `task.rs` pour `required_runtime` ;
- le reste de `canonical.rs` et du dépôt nécessaire au balayage des consommateurs.

Je ne peux donc pas confirmer les goldens, les features Cargo unifiées, l’inclusion réelle du test web, le census live complet, l’absence d’anciennes mentions « 25 », ni les cinq déclarations NF-2 du futur commit body.

## Verdict global

**BLOQUÉ.**

La vérification Rust principale est désormais nettement solide : ordre discriminant, duplicate keys, liaison content↔lock, Ed25519 strict et corpus post-signature ne présentent pas de défaut P0 visible. Mais la frontière TS accepte encore une classe Unicode que Rust refuse, et le scanner viole sa nouvelle garantie de non-recopie des secrets via le hostname. Ces deux P1 sont nouveaux, précis et reproductibles sur le code fourni.