## Verdict ROUND 3

**FAIL de clôture — 0 P0, 3 P1.** Deux P1 sont des corrections incomplètes ; le troisième est un défaut préexistant révélé par la confrontation exacte du panic wipe.

| Livrable | Verdict | Motif |
|---|---|---|
| Coffre | **PARTIEL** | Primitive solide, mais la row rank‑1 contredit encore la nouvelle sémantique de révocation et le point d’usage provider reste non atomique. `crates/nexus-core-rs/src/vault.rs:507-525`, `docs/security/THREAT_MODEL.md:1975`, `:2295-2303`. |
| HITL | **COMPLET dans le périmètre “primitive L1”** | Binding, décision typée et vérification Ed25519 stricte présents ; DS‑1 reste explicitement hors câblage. `crates/sbfb-workflow-engine/src/hitl.rs:361-410`, `crates/nexus-core-rs/src/hitl_approval.rs:87-102`, `docs/security/THREAT_MODEL.md:1942-1953`. |
| Canari | **COMPLET pour la propriété corrigée `BoundSecret: !Serialize`** | Le doctest négatif est discriminant et le canari conserve son rôle runtime. Des imprécisions P2 subsistent sur `VaultSecret` et dans un commentaire. `crates/sbfb-workflow-engine/src/secret.rs:124-151`, `:531-640`. |

## P1

### R3-P1-1 — La correction du bind n’a pas atteint la row autoritaire

Le code et trois textes décrivent correctement la fenêtre ordinaire : contrôle d’epoch à `n`, révocation à `n+1`, puis lecture/déchiffrement de l’ancien body. `vault.rs:507-525`, `docs/workflow/llms.txt:119-125`, `THREAT_MODEL.md:2295-2303`.

Mais la row `T-SECRET-VAULT-LIFECYCLE` affirme toujours que la collision lost-update est « le seul état » où la moitié lecture tombe. `docs/security/THREAT_MODEL.md:1975`. C’est la contradiction exacte avec la borne 27. Le P1-2 du round 2 n’est donc pas fermé au niveau rank‑1.

Il existe aussi une troisième fenêtre au niveau système : `bind` peut **terminer avant** la révocation, puis le `VaultSecret` déjà rendu être consommé par le provider **après** celle-ci. Ce n’est pas une troisième fenêtre interne à `bind`, mais c’est distinct de « bind se termine après révocation ». La phrase sur le bail jusqu’au point d’usage l’admet indirectement, sans l’énoncer comme sémantique. `vault.rs:513-525`, `THREAT_MODEL.md:2300-2303`.

### R3-P1-2 — Les quatre étages du runbook ne sont pas présents à §3.3

§1 distingue correctement HARD-FAIL, BEST-EFFORT, INERTE et NON RÉCLAMÉ. `docs/security/DURESS.md:20-45`. §5 les développe également. `DURESS.md:185-217`.

En revanche, §3.3 dit encore : « seuls les deux blobs […] sont hard-fail, les autres étages sont best-effort ». Cela classe faussement le keyring inerte et surtout le coffre non réclamé comme des purges tentées. `DURESS.md:129-136`.

Les deux instructions futures « modifier §1 ET §5 — les DEUX » oublient désormais précisément ce troisième site. `DURESS.md:41-45`, `:211-217`.

L’énumération des fichiers est également inexacte : en production, `PanicWipeService` reçoit `subscriptions_json`, pas `state.sqlite`. `crates/nexus-shell-daemon/src/runtime.rs:429-445`, `crates/nexus-shell-daemon-core/src/config.rs:107-110`, `:148-152`. Le vrai run store est `workflow_runs.sqlite3` et n’est pas réclamé. `crates/nexus-shell-daemon/src/workflow_api.rs:45-49`, `crates/nexus-shell-daemon/src/runtime.rs:752-754`.

### R3-P1-3 — Le HARD-FAIL des blobs peut rendre `Ok` sans les effacer

`wipe_all` propage bien les erreurs produites par `secure_unlink_blob` avec `?`. `crates/nexus-core-rs/src/keystore.rs:1044-1048`. Mais `secure_unlink_blob` commence par `Path::exists()` et retourne directement `Ok(())` lorsque celui-ci vaut faux. `keystore.rs:1082-1092`.

Or `Path::exists()` convertit notamment une erreur d’accès aux métadonnées en `false`, au lieu de la propager. [Documentation officielle Rust](https://doc.rust-lang.org/stable/std/path/struct.Path.html#method.exists). Un blob existant mais inaccessible peut donc survivre, `wipe_all` rendre `Ok`, `keystore_wiped` devenir `true`, puis le handler répondre `{"wiped":true}` et programmer l’arrêt. `crates/nexus-shell-daemon/src/panic.rs:183-197`, `crates/nexus-shell-daemon/src/blob_serve_http.rs:183-207`.

La qualification HARD-FAIL de `DURESS.md:26-28` est donc fausse sur cette branche fail-open.

## Classement réel du panic wipe

| Étage | Confrontation au code |
|---|---|
| Blobs d’identité | `?` propage écriture/unlink, mais le probe `exists()` masque certaines erreurs : **pas strictement hard-fail**. `keystore.rs:1044-1048`, `:1085-1092`. |
| Deux scratchs | Bien **best-effort, erreur avalée** par deux `let _ =`. `keystore.rs:1050-1067`. |
| Subscriptions + blob cache | Bien best-effort ; seules deux valeurs booléennes sont conservées. `panic.rs:188-197`, `:220-239`. |
| Keyring | Suppressions avalées ; actuellement **inerte** avec le backend mock. `keystore.rs:1069-1078`. |
| `SecretVault` | Bien **non réclamé** : aucune API `wipe`. `THREAT_MODEL.md:2042-2049`, `DURESS.md:211-217`. |

Le rapport ne sait pas enregistrer l’échec d’un scratch : `wipe_all()` l’avale, puis `keystore_wiped` est posé à `true`. Les affirmations « le rapport enregistre ce qui a réellement disparu » sont donc trop fortes. `DURESS.md:29-37`, `panic.rs:119-126`, `:183-197`.

## P2 nouveaux ou résiduels

- **Attribution `Serialize` pas entièrement nettoyée.** Le doctest est au bon endroit, `serde_json` est une dépendance réelle, et `E0277` est bien l’erreur de trait attendue. `secret.rs:130-151`, `crates/sbfb-workflow-engine/Cargo.toml:11-17`. Mais le commentaire « Three changes fix that » garde pour antécédent l’exemple `Serialize`, alors que les trois bras runtime ne le détectent toujours pas. `secret.rs:512-529`. De plus, `THREAT_MODEL.md:1974` présente le doctest `BoundSecret` comme gardant l’absence de `Serialize` de **`BoundSecret` et `VaultSecret`**, tandis que `VaultSecret` n’a aucun doctest négatif. `vault.rs:192-205`.

- **Rustdoc de `persist` faux sur le compte.** Après une collision, les deux handles ont le même epoch, mais un seul body est périmé : celui du writer dont le rename a perdu. `vault.rs:715-720` dit que les **deux** portent l’epoch sur un body périmé, contrairement à la description correcte de `vault.rs:99-107` et `THREAT_MODEL.md:2271-2279`.

- **Gate AWK encore fail-open sur ses propres erreurs.** `awk … || true` avale un échec d’exécution ou de lecture ; stderr peut contenir l’erreur tandis que `bound_break` reste vide et que la jambe passe. `scripts/check-frontier-contracts.sh:573-648`. Le motif de titre accepte aussi `19.4bis`, car il exige seulement un caractère « ni chiffre ni point » après `19.4`. `check-frontier-contracts.sh:594-610`.

- **Classe fence incomplète.** Le caractère ouvrant est désormais mémorisé, mais pas la longueur ; quatre backticks peuvent être fermés par trois, et une indentation supérieure à trois espaces est traitée comme une fence. CommonMark exige au plus trois espaces d’indentation et un fermant au moins aussi long que l’ouvrant. `check-frontier-contracts.sh:581-593`, [spécification CommonMark](https://spec.commonmark.org/spec#fenced-code-blocks).

Les six P2 du round 2 restent au même niveau, sans aggravation constatée : témoin public (`hitl.rs:255-279`), `BindingMismatch` (`hitl.rs:287-299`, `:375-385`), erreurs textuelles (`secret.rs:106-121`, `activity.rs:113-144`), front et doublons (`approvalsQueue.ts:141-154`, `:209-241`, `Approvals.tsx:214-244`), validation tardive des identités (`compile.rs:662-670`, `:1001-1011`) et limites de zeroize/panic-abort (`keystore.rs:1110-1121`, `vault.rs:1079-1102`, `Cargo.toml:489-510`).

## P3

- Le diagnostic « second §19.4 ferme la section » est faux : la branche fait `next` sans mettre `inside = 0`, donc le scan continue. `check-frontier-contracts.sh:604-617`.
- Une fence non terminée ouverte après §19.4 provoque également un rouge imputé à §19.4 alors que ses 27 bornes ont déjà été lues. `check-frontier-contracts.sh:630-640`.
- « Digest injectif » / « deux bindings différents ne collisionnent jamais » reste mathématiquement trop fort pour BLAKE3-256. `crates/sbfb-workflow-engine/src/hitl.rs:242-253`, `docs/security/THREAT_MODEL.md:73`.

La séquence actuelle de §19.4 est bien `1…27`, de `THREAT_MODEL.md:1984` à `:2265`. Le programme AWK contient **zéro apostrophe ASCII** entre ses délimiteurs `check-frontier-contracts.sh:574-641`.

Aucun `cargo`/`npm` n’a été lancé et aucun fichier du dépôt n’a été écrit ; le worktree final est resté dans l’état non commité initial.

