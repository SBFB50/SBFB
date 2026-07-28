## Verdict ROUND 4

**FAIL de clôture — 0 P0, 1 P1.**

Deux correctifs R3 tiennent complètement ; le troisième reste incomplet dans `DURESS.md §5`. Je n’ai trouvé aucune régression d’exécution créée par `try_exists()`, le doctest ou les nouvelles formulations de `bind`.

| Livrable | Verdict | Motif |
|---|---|---|
| Coffre | **PARTIEL** | Primitive solide et bornes désormais honnêtes, mais la collision lost-update reste réelle et aucune révocation dure jusqu’au fournisseur n’est implémentée. `vault.rs:481-529`, `:715-753`, `THREAT_MODEL.md:2265-2307`. |
| HITL | **COMPLET dans le périmètre primitive L1** | Tuple 15 champs, vérification keyless et Ed25519 stricte présents ; DS-1 reste explicitement hors câblage. `hitl.rs:142-183`, `:361-396`, `hitl_approval.rs:87-102`, `THREAT_MODEL.md:1942-1953`. |
| Canari | **COMPLET pour `BoundSecret: !Serialize` et les surfaces G-SECRET** | Le doctest négatif est discriminant et le canari runtime conserve son rôle. Deux imprécisions P2 subsistent. `secret.rs:124-151`, `:586-640`. |

## État des correctifs

- **R3-P1-1 — OUI.** La row autoritaire énumère bien les trois fenêtres ; le rustdoc de `bind`, la borne 27 et `llms.txt` portent la même sémantique et la nécessité d’un bail jusqu’au fournisseur. `THREAT_MODEL.md:1975`, `:2295-2307`, `vault.rs:507-519`, `llms.txt:120-126`.

- **R3-P1-2 — PARTIEL, donc non fermé.** §3.3 porte bien HARD-FAIL / BEST-EFFORT / INERTE / NON RÉCLAMÉ. `DURESS.md:143-154`. Mais §5 conserve encore deux formulations de l’ancien état, détaillées dans le P1 ci-dessous.

- **R3-P1-3 — OUI.** Les deux appels blobs propagent avec `?`, puis `secure_unlink_blob` utilise `try_exists()?`; `metadata`, écriture et unlink propagent également leurs erreurs. `keystore.rs:1044-1048`, `:1082-1103`.

- **Doctest `BoundSecret` — OUI techniquement.** Il est porté par le type, utilise `compile_fail,E0277`, appelle réellement `serde_json::to_string(secret)`, et le crate réexporte le type tout en dépendant de `serde_json`. `secret.rs:130-151`, `sbfb-workflow-engine/Cargo.toml:17`, `lib.rs:223`.

## P1

### R4-P1-1 — La correction de `DURESS.md` n’a pas atteint §5

§1 dit correctement que la production reçoit `subscriptions.json`, jamais `state.sqlite`, et exige de tenir à jour **§1, §3.3 et §5**. `DURESS.md:35-38`, `:52-59`.

Mais §5 affirme encore :

- ``subscriptions.json` (`state.sqlite`) est unlink`` ; `DURESS.md:224-228` ;
- que le futur correctif doit modifier seulement « §1 ET §5 — les DEUX » ; `DURESS.md:229-235`.

Cela contredit le câblage réel, qui transmet `opts.paths.subscriptions_json`, et laisse `workflow_runs.sqlite3` séparé. `runtime.rs:441-445`, `:742-754`, `workflow_api.rs:45-49`.

Ce n’est pas seulement un renvoi cosmétique : §5 est la section de récupération opérateur et surestime encore la cible effacée tout en prescrivant exactement la future dérive qui avait laissé §3.3 faux au round 3.

## Classement exact du panic wipe

| Élément | Comportement réel |
|---|---|
| `identity.enc`, `identity_duress.enc` | **HARD-FAIL** : les erreurs des deux suppressions sont propagées. Une erreur interrompt `wipe_all`, donc les étapes suivantes ne sont pas exécutées. `keystore.rs:1044-1048`, `:1096-1103`, `panic.rs:190-204`. |
| Deux `.enc.tmp` | **BEST-EFFORT, erreur avalée** via deux `let _ =`. `keystore.rs:1050-1067`. |
| `subscriptions.json` | **BEST-EFFORT** : erreurs de probe/métadonnées/écriture sont converties en `false` ou ignorées ; seule la réussite finale de `remove_file` donne `true`. `runtime.rs:441-445`, `panic.rs:195-201`, `:229-240`. |
| `blob-cache/` | **BEST-EFFORT** : `remove_dir_all` retourne seulement un booléen. `panic.rs:203-204`, `:243-247`. |
| Entrées keyring | Suppression **best-effort**, mais effet de sécurité actuellement **INERTE** : erreurs de création et de suppression avalées, backend keyring réduit à `log + zeroize`. `keystore.rs:1069-1077`, `Cargo.lock:4393-4401`. |
| `SecretVault` | **NON RÉCLAMÉ** : aucune API de wipe du blob ou des scratchs. `vault.rs:303-388`, `:470-754`. |
| `workflow_runs.sqlite3` | **NON RÉCLAMÉ** : ouvert séparément et absent des trois paramètres du service. `runtime.rs:742-754`, `panic.rs:146-176`. |

Le classement à quatre étages de §1 est donc exact pour les objets qu’il nomme. `DURESS.md:26-55`.

En revanche, le contrat du rapport reste incohérent :

- le struct porte **trois** booléens, pas deux ; `panic.rs:126-133` ;
- `keystore_wiped=true` ne signifie pas que les scratchs ont disparu ; `panic.rs:41-45`, `:190-204` ;
- son rustdoc dit encore « what execute actually deleted » et « audit logs », alors que le handler jette le rapport avec `Ok(_)`. `panic.rs:126-127`, `blob_serve_http.rs:183-197` ;
- `DURESS.md` écrit littéralement « seulement DEUX booléens », puis nomme lui-même le troisième `keystore_wiped`. `DURESS.md:42-46`.

Je classe cette dernière incohérence **P2**, car la conclusion opérateur — purge partielle indiscernable — reste correcte.

## Sémantique de `bind`

Les trois fenêtres sont correctement distinguées :

1. collision lost-update : même epoch, un seul body périmé ;
2. contrôle d’epoch avant révocation, fin du `bind` après ;
3. `bind` terminé avant révocation, mais `VaultSecret` consommé après.

`vault.rs:499-529`, `THREAT_MODEL.md:1975`, `:2271-2307`, `llms.txt:109-126`.

Je ne trouve pas de quatrième fenêtre distincte dans cette API : toute copie ou conservation après le retour de `bind` retombe dans la troisième classe « capacité déjà rendue ». `vault.rs:192-213`, `:520-529`.

Il reste toutefois un **P2 documentaire** : `persist` affirme que les deux handles portent l’epoch disque « over a stale body ». En réalité, les deux portent le même epoch, mais seul le perdant du dernier `rename` a un body périmé. Le doc-module et la borne 27 donnent, eux, la bonne formulation. `vault.rs:99-107`, `:719-723`, `THREAT_MODEL.md:2271-2274`.

## Canari et `Serialize`

Un autre texte attribue encore implicitement au canari la fermeture de `Serialize` : après avoir cité précisément la mutation `Serialize`, le commentaire dit « Three changes fix that », puis énumère trois contrôles runtime qui ne sérialisent jamais `BoundSecret`. `secret.rs:514-529`.

La row rank-1 ne fait plus cette attribution au canari, mais elle présente le doctest `BoundSecret` comme gardant l’absence de `Serialize` de **`BoundSecret` et `VaultSecret`**. Aucun doctest négatif analogue n’existe sur `VaultSecret`. `THREAT_MODEL.md:1974`, `vault.rs:192-205`.

Ces deux écarts restent **P2** ; la propriété mécanique `BoundSecret: !Serialize` est bien fermée.

## Gate `§19.4 BOUND ORDERING`

État actuel :

- séquence source effectivement **1→27** ; `THREAT_MODEL.md:1984-2265` ;
- **zéro apostrophe ASCII** dans le programme AWK entre `check-frontier-contracts.sh:574-641`.

Mais les P2 latents restent présents :

- `awk … || true` avale encore une erreur d’exécution ou de lecture sans remplir `bound_break` ; `check-frontier-contracts.sh:573-643` ;
- le motif `19\.4([^0-9.]|$)` accepte toujours `19.4bis` ; `check-frontier-contracts.sh:594-610` ;
- la fence ne mémorise ni la longueur d’ouverture, ni la limite CommonMark de trois espaces, ni l’exigence d’un fermant sans contenu : quatre backticks peuvent être fermés par trois et une ligne indentée de quatre espaces est traitée comme une fence. `check-frontier-contracts.sh:581-593`.

P3 résiduels :

- le diagnostic « second §19.4 closes the section » est faux : la branche fait `next` sans `inside = 0`; `check-frontier-contracts.sh:604-617` ;
- une fence non terminée après la section fait encore échouer cette jambe, car l’état `fence` est global jusqu’à `END`; `check-frontier-contracts.sh:630-640` ;
- la suppression de la dernière borne reste volontairement non détectée : cette jambe prouve la contiguïté, pas la complétude. `check-frontier-contracts.sh:564-567`.

## P2 historiques

Aucun des six P2 assumés ne change de sévérité :

- témoin public : `hitl.rs:255-279` ;
- `BindingMismatch` brut : `hitl.rs:287-299`, `:375-385` ;
- erreurs textuelles conventionnelles : `secret.rs:106-121`, `activity.rs:113-147` ;
- rejet silencieux et doublons front : `approvalsQueue.ts:141-154`, `:209-241`, `Approvals.tsx:204-218` ;
- validation tardive des identités : `compile.rs:659-670`, `:1001-1011` ;
- limites zeroize/panic-abort : `keystore.rs:1121-1132`, `vault.rs:1083-1106`, `Cargo.toml:489-510`.

Le P3 mathématique « digest injectif / jamais de collision » demeure également. `hitl.rs:242-253`, `THREAT_MODEL.md:73`.

Aucun fichier n’a été écrit, aucun `cargo`/`npm` lancé, et le worktree est resté sur `9134af64` avec le même état non commité.

