## Verdict ROUND 5

**PASS de clôture — 0 P0, 0 P1, 4 P2, 2 P3. Aucun défaut bloquant ni nouvelle régression d’exécution détectée.**

| Livrable | Verdict | Motif |
|---|---|---|
| Coffre | **PARTIEL** | Primitive solide, mais collision lost-update et révocation dure jusqu’au fournisseur restent ouvertes. [`vault.rs:94`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:94), [`vault.rs:499`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:499), [`vault.rs:514`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:514) |
| HITL | **COMPLET dans le périmètre primitive L1** | Tuple 15 champs, vérification keyless et signature Ed25519 stricte présents ; DS-1 reste explicitement hors reliure. [`hitl.rs:143`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:143), [`hitl.rs:375`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:375), [`hitl_approval.rs:87`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/hitl_approval.rs:87), [`THREAT_MODEL.md:1942`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1942) |
| Canari | **COMPLET pour `BoundSecret: !Serialize` et les cinq surfaces déclarées** | Le doctest négatif est discriminant ; les bras rouges et les cinq surfaces sont présents. Les deux imprécisions documentaires ci-dessous n’annulent pas la propriété mécanique. [`secret.rs:130`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:130), [`secret.rs:586`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:586), [`secret.rs:618`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:618) |

## Les trois corrections

- **Canari / `Serialize` : correction mécanique tenue.** Le doctest est attaché directement à `BoundSecret`, utilise bien `compile_fail,E0277` et appelle réellement `serde_json::to_string(secret)`. `serde_json` est une dépendance normale et `BoundSecret` est réexporté. [`secret.rs:124`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:124), [`Cargo.toml:17`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/Cargo.toml:17), [`lib.rs:223`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:223)

- **Bind : correction rank-1 tenue.** Collision, bind admis avant révocation et secret déjà rendu puis utilisé après révocation sont maintenant tous explicités dans le rustdoc, la borne 27 et `llms.txt`. [`vault.rs:499`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:499), [`THREAT_MODEL.md:2295`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2295), [`llms.txt:120`](/C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/llms.txt:120)

- **DURESS : correction de cible tenue.** §5 nomme bien `subscriptions.json`, jamais `state.sqlite` autrement que par négation ; `workflow_runs.sqlite3` est déclaré non réclamé ; les deux consignes futures nomment désormais §1, §3.3 et §5. Le runtime transmet effectivement `opts.paths.subscriptions_json`, tandis que le run store est ouvert séparément. [`DURESS.md:224`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/DURESS.md:224), [`DURESS.md:240`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/DURESS.md:240), [`DURESS.md:243`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/DURESS.md:243), [`runtime.rs:441`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:441), [`runtime.rs:752`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:752)

## Findings P2

### P2-1 — La classe textuelle `Serialize` n’est toujours pas entièrement fermée

Oui, il reste l’« autre texte » demandé : après avoir pris `Serialize` comme exemple de défaut non détecté, le commentaire affirme encore « Three changes fix that », puis énumère trois contrôles runtime qui ne sérialisent aucun `BoundSecret`. [`secret.rs:514`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:514), [`secret.rs:521`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:521)

La row autoritaire dit correctement que le canari ne garde pas cette propriété, mais rattache grammaticalement le doctest `BoundSecret` à l’absence de `Serialize` de **`BoundSecret` et `VaultSecret`**. Aucun doctest négatif analogue n’existe sur `VaultSecret`. [`THREAT_MODEL.md:1974`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1974), [`vault.rs:192`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:192)

### P2-2 — Les nouvelles formulations d’observabilité du wipe sont contradictoires

Trois affirmations restent trop larges :

- le rapport « ne porte que deux booléens », alors que `PanicWipeReport` en porte trois ;
- §3.3 dit que « la réponse est la même dans tous ces cas » ;
- §5 dit que « ce qui a résisté n’est signalé nulle part ».

[`DURESS.md:42`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/DURESS.md:42), [`DURESS.md:146`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/DURESS.md:146), [`DURESS.md:203`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/DURESS.md:203), [`DURESS.md:229`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/DURESS.md:229)

Or un hard-fail de blob produit une erreur HTTP 500 et ne programme pas l’arrêt ; seuls les `Ok` obtiennent `{"wiped":true}` puis `exit_only`. Le rapport contient bien `keystore_wiped`, `state_db_deleted` et `blob_cache_deleted`, même si le premier ne sait pas exprimer un scratch survivant. [`panic.rs:126`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/panic.rs:126), [`blob_serve_http.rs:186`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/blob_serve_http.rs:186)

Le rustdoc du rapport reste également faux en disant qu’il décrit ce qui a « actually deleted » et alimente les audit logs : le handler jette la valeur via `Ok(_)`. [`panic.rs:126`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/panic.rs:126), [`blob_serve_http.rs:187`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/blob_serve_http.rs:187)

### P2-3 — `persist` dit encore que les deux corps sont périmés

Après la collision, les deux handles portent l’epoch gagnant, mais **un seul** possède un body périmé : celui dont le `rename` a perdu. Le module et la borne 27 le disent correctement ; le rustdoc de `persist` écrit encore que les deux portent « the disk’s epoch over a stale body ». [`vault.rs:94`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:94), [`vault.rs:719`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:719), [`THREAT_MODEL.md:2271`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2271)

### P2-4 — La jambe `BOUND ORDERING` conserve des fail-open

- `awk … || true` absorbe une erreur awk ou de lecture ; si stdout reste vide, `bound_break` est vide et la jambe passe. [`check-frontier-contracts.sh:642`](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:642)
- Le titre `19\.4([^0-9.]|$)` accepte `19.4bis` : une telle « nouvelle » rubrique serait encore reconnue comme §19.4 malgré la revendication « renamed heading → RED ». [`check-frontier-contracts.sh:558`](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:558), [`check-frontier-contracts.sh:600`](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:600)
- Les fences mémorisent désormais le caractère, mais ni la longueur ni la grammaire de fermeture CommonMark : quatre backticks peuvent être fermés par trois, et une ligne indentée de quatre espaces est traitée comme une fence. Cela peut rendre visibles des nombres qui sont encore dans le bloc de code et masquer un trou réel. [`check-frontier-contracts.sh:586`](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:586)

L’arbre courant reste sain : titre exact à §19.4 et séquence source 1→27. [`THREAT_MODEL.md:1911`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1911), [`THREAT_MODEL.md:1984`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1984), [`THREAT_MODEL.md:2265`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2265)

## Panic wipe, étage par étage

| Étage | Résultat exact |
|---|---|
| Blobs d’identité | **HARD-FAIL réel.** Les deux appels portent `?`; `try_exists`, métadonnées, écriture et unlink propagent. Une erreur arrête `execute` avant subscriptions/cache et aucun exit n’est programmé. [`keystore.rs:1044`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/keystore.rs:1044), [`keystore.rs:1096`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/keystore.rs:1096), [`panic.rs:190`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/panic.rs:190) |
| Deux scratchs | **BEST-EFFORT, erreur avalée** par deux `let _ =`; `keystore_wiped=true` ensuite. [`keystore.rs:1050`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/keystore.rs:1050), [`panic.rs:192`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/panic.rs:192) |
| Keyring | Suppressions ignorées et actuellement **INERTES**, car la dépendance utilise le builder mock. [`keystore.rs:1069`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/keystore.rs:1069), [`Cargo.toml:322`](/C:/Users/FlowUP/Documents/Code/nexus/Cargo.toml:322) |
| `subscriptions.json` | **BEST-EFFORT.** Probe, métadonnées ou écriture peuvent échouer sans propagation ; seul `remove_file().is_ok()` alimente le booléen. [`panic.rs:195`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/panic.rs:195), [`panic.rs:229`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/panic.rs:229) |
| `blob-cache/` | **BEST-EFFORT**, résultat réduit à un booléen. [`panic.rs:203`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/panic.rs:203), [`panic.rs:243`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/panic.rs:243) |
| Coffre + run store | **NON RÉCLAMÉS.** Le service ne reçoit que keystore, subscriptions et cache ; le run store est ouvert séparément. [`panic.rs:146`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/panic.rs:146), [`runtime.rs:752`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/runtime.rs:752) |

Le classement à quatre étages est donc exact ; c’est son résumé d’observabilité qui ne l’est pas complètement.

## Bind et troisième fenêtre

La troisième fenêtre existe bien et elle est maintenant explicitement documentée :

1. collision lost-update : même epoch, un body périmé ;
2. bind admis avant révocation, terminé après ;
3. bind terminé avant révocation, `VaultSecret` utilisé par le fournisseur après.

[`vault.rs:499`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:499), [`vault.rs:507`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:507), [`vault.rs:514`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:514)

La borne 27 et `llms.txt` portent la même sémantique et la nécessité d’un bail jusqu’au point d’usage. Je ne trouve pas de quatrième classe distincte dans cette API : conservation, copie ou réutilisation ultérieure du `VaultSecret` retombent dans la troisième. [`THREAT_MODEL.md:2295`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2295), [`llms.txt:120`](/C:/Users/FlowUP/Documents/Code/nexus/docs/workflow/llms.txt:120)

## P3

- Le diagnostic du second titre dit qu’il ferme la section, mais la branche fait seulement `next`, sans `inside = 0`; inversement, une fence non terminée après §19.4 peut encore faire rougir cette jambe. Fail-closed seulement. [`check-frontier-contracts.sh:609`](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:609), [`check-frontier-contracts.sh:636`](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:636)
- §5 affirme que le coffre survivant est scellé sous « un PIN à 6 chiffres », alors que `SecretVault::init` accepte un `&str` sans contrainte de longueur ou de chiffres. [`DURESS.md:235`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/DURESS.md:235), [`vault.rs:324`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:324)

Enfin, le programme awk contient **zéro apostrophe ASCII** entre ses délimiteurs. [`check-frontier-contracts.sh:573`](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:573)

Aucun fichier n’a été écrit et aucun `cargo`/`npm` n’a été relancé.

