# Verdict ROUND 2

**Verdict global : FAIL de clôture — 0 P0, 3 P1.** Ce FAIL ne repose pas sur l’absence de reliure : DS‑1 et les bornes 1, 2 et 15 décrivent honnêtement les `0 call-site`, le témoin non cryptographique et le périmètre étroit du canari. `docs/security/THREAT_MODEL.md:1942-1953`, `:1984-1988`, `:2065-2070`.

| Livrable | Verdict | Motif |
|---|---|---|
| Coffre | **PARTIEL** | La primitive cryptographique est solide et la collision lost-update est documentée, mais la prose promet à tort que c’est le seul chemin de service post-révocation ; un `bind` déjà admis peut rendre l’ancien secret après un `delete` concurrent. `crates/nexus-core-rs/src/vault.rs:504-513`, `:632-644`, `docs/security/THREAT_MODEL.md:1975`. |
| HITL | **COMPLET dans le périmètre “primitive L1”** | Binding JCS, signature Ed25519 locale stricte, moteur keyless et affordance sont présents. La composition de production reste explicitement hors périmètre. Des P2 subsistent sur la sémantique de preuve, les diagnostics et la file front. `crates/sbfb-workflow-engine/src/hitl.rs:55-64`, `crates/nexus-core-rs/src/hitl_approval.rs:35-50`, `:87-102`. |
| Canari | **PARTIEL** | Il discrimine `Debug`, son oracle et le blob au repos, mais ne discrimine toujours pas l’ajout d’un `impl Serialize for BoundSecret` fuyant. `crates/sbfb-workflow-engine/src/secret.rs:441-457`, `:561-615`, `crates/nexus-core-rs/src/vault.rs:1219-1242`. |

## P0

Aucun P0 trouvé.

## P1

### P1-1 — Le canari est présenté comme discriminant contre `Serialize`, mais ne l’est pas

Le bras rouge copie manuellement `bound.expose()` dans une `ActivityReceipt`; les cinq surfaces vertes sont une requête, son hash, un receipt, un event et `Debug`. Aucune ne sérialise `BoundSecret`. `crates/sbfb-workflow-engine/src/secret.rs:561-600`.

Par inspection, la mutation proposée :

```rust
impl serde::Serialize for BoundSecret {
    // sérialise self.expose()
}
```

compilerait dans le crate et laisserait ces tests verts. Le test ne contient aucun appel de sérialisation sur `bound`. À l’inverse, une mutation de `Debug` imprimant `self.material` rougirait bien aux contrôles exacts. `crates/sbfb-workflow-engine/src/secret.rs:193-200`, `:441-457`, `:593-615`.

Trois textes affirment néanmoins que le bras rouge ferme cette classe : le commentaire de type, la row rank‑1 et le preflight. `crates/sbfb-workflow-engine/src/secret.rs:177-186`, `docs/security/THREAT_MODEL.md:1974`, `.planning/active/sprint83_phase_l1_preflight.md:102-104`.

Correction minimale : ajouter un test `compile_fail` tentant `serde_json::to_string(&BoundSecret)` et réécrire les claims jusqu’à ce que cette mutation fasse réellement échouer un gate.

### P1-2 — La garantie de révocation du coffre omet le bind déjà admis

Séquence possible sans collision lost-update :

1. A exécute `refuse_if_stale()` à l’epoch `n`.
2. B supprime ou tourne l’entrée et commit `n+1`.
3. A lit son ancien `body`, déchiffre et retourne le secret après le commit de B.

La fenêtre existe entre le contrôle `vault.rs:506` et l’ouverture `vault.rs:511-513`; `delete` peut commettre pendant cet intervalle via `persist`. `crates/nexus-core-rs/src/vault.rs:504-513`, `:632-644`, `:699-736`.

Cette exécution reste linéarisable si l’on définit le point d’admission du bind au contrôle d’epoch. Mais ni le rustdoc, ni `llms.txt`, ni la row rank‑1 n’énoncent cette sémantique : ils disent que la collision lost-update est « le seul état ». `crates/nexus-core-rs/src/vault.rs:497-503`, `docs/workflow/llms.txt:109-119`, `docs/security/THREAT_MODEL.md:1975`.

La borne 27 est exacte sur le mécanisme et les quatre dégradations, mais sa phrase « la cécité dure jusqu’au prochain commit » oublie que `reload()` réaligne immédiatement le handle sans commit. `docs/security/THREAT_MODEL.md:2271-2284`, `crates/nexus-core-rs/src/vault.rs:516-547`.

Deux fermetures possibles :

- borner explicitement : « un bind admis avant la révocation peut terminer après elle » ;
- pour une révocation dure, porter un verrou/lease jusqu’au point d’usage provider. Une simple seconde relecture réduit la fenêtre sans la fermer.

### P1-3 — `DURESS.md` promet une destruction irréversible que le code traite en best-effort

La vue d’ensemble inclut les deux scratchs, subscriptions et blob cache dans une « destruction irréversible ». `docs/security/DURESS.md:20-30`.

Le code réel est :

- blobs normal/duress : zero-overwrite + unlink, erreur propagée ; `crates/nexus-core-rs/src/keystore.rs:1044-1048`, `:1082-1092` ;
- deux scratchs : erreurs avalées par `let _ =`; `crates/nexus-core-rs/src/keystore.rs:1058-1067` ;
- keyring : best-effort et actuellement inerte ; `crates/nexus-core-rs/src/keystore.rs:1069-1078` ;
- subscriptions et blob cache : best-effort, échec seulement enregistré dans le rapport ; `crates/nexus-shell-daemon/src/panic.rs:177-197` ;
- le handler ignore le détail du rapport et répond `{"wiped": true}` sur tout `Ok`; `crates/nexus-shell-daemon/src/blob_serve_http.rs:183-196` ;
- le coffre de secrets n’est pas réclamé. `docs/security/DURESS.md:187-193`.

La section 5 reconnaît correctement la survie silencieuse d’un scratch, mais pas le caractère best-effort de subscriptions/cache. `docs/security/DURESS.md:171-190`. Le résumé rank‑1 doit distinguer « hard-fail », « best-effort » et « non réclamé ».

## P2

### P2-1 — Autorité cryptographique : le témoin est public et la rustdoc attribue trop à la signature

`HostVerifiedApproval` est public, son champ est privé, mais son unique constructeur ex nihilo est lui aussi public et n’effectue aucune crypto. `Clone::clone` est également public, mais exige un témoin existant. Il n’existe ni `Default`, ni `Deserialize`, ni `From`. `crates/sbfb-workflow-engine/src/hitl.rs:263-285`, `crates/sbfb-workflow-engine/src/lib.rs:216-219`.

Donc, depuis un autre crate :

```rust
HostVerifiedApproval::attest_verified_signature("n'importe quoi")
```

est valide. Ce n’est pas un P1 de périmètre : le module et la borne 2 le disent honnêtement. `crates/sbfb-workflow-engine/src/hitl.rs:33-45`, `:465-479`, `docs/security/THREAT_MODEL.md:1987-1988`.

La chaîne cryptographique exacte est :

1. JCS du tuple, puis BLAKE3 sur `HITL_BINDING_CONTEXT || 0x1f || JCS`; `crates/sbfb-workflow-engine/src/hash.rs:330-347`, `crates/sbfb-workflow-engine/src/hitl.rs:249-252`.
2. Ed25519 sur `sbfb-hitl-approval-v1 || 0x1f || digest_hex`; `crates/nexus-core-rs/src/hitl_approval.rs:27-50`.
3. Vérification stricte contre la clé publique fournie, avec refus des clés faibles. `crates/nexus-core-rs/src/hitl_approval.rs:87-102`.

La préimage est canonique et non ambiguë : les contextes sont fixes et les séparateurs ont une position déterminée. Le marqueur de signature est correctement déclaré **LOCAL**, pas famille wire. `crates/nexus-core-rs/src/hitl_approval.rs:13-20`, `:27-37`.

En revanche, la fonction accepte n’importe quel `KeyPair` et le vérificateur n’importe quelle clé publique. Elle prouve seulement : « la clé privée correspondant à cette clé publique a signé ces octets ». Elle ne prouve pas cryptographiquement que la clé est l’identité déverrouillée du nœud, ni que le chrome ou un humain a approuvé. `crates/nexus-core-rs/src/crypto.rs:60-97`, `crates/nexus-core-rs/src/hitl_approval.rs:47-50`, `:87-102`. La rustdoc « proving the approval came from the trusted chrome host » doit être qualifiée comme invariant futur de routage. `crates/nexus-core-rs/src/hitl_approval.rs:4-11`.

### P2-2 — `BindingMismatch` conserve l’injection de ligne

`approved` est interpolé verbatim par `thiserror`, et `verify_binding` y recopie directement le contenu du témoin. `crates/sbfb-workflow-engine/src/hitl.rs:289-299`, `:380-385`.

Ainsi, `attest_verified_signature("00\nWARN forged")` produit bien une ligne de diagnostic forgée en cas de mismatch. `sanitise_untrusted` ne protège que les références de capacité. `crates/sbfb-workflow-engine/src/hitl.rs:334-359`.

Correction : newtype de digest validant exactement 64 caractères hexadécimaux minuscules, ou échappement systématique avant rendu.

### P2-3 — Les erreurs secret/activity restent conventionnelles

`BoundSecret` ferme bien les routes directes suivantes :

- `Debug` : matériel masqué, longueur seulement ; `crates/sbfb-workflow-engine/src/secret.rs:193-200` ;
- `Display`, `Serialize`, `Deserialize`, `Clone` : absents du type actuel ; `crates/sbfb-workflow-engine/src/secret.rs:124-140` ;
- un crate externe ne peut pas ajouter `Serialize` ou `Clone` au type étranger.

Mais `expose()` permet explicitement une copie, donc un `panic!`, un `format!` sur le slice ou une erreur construite depuis cette copie peut fuir. `crates/sbfb-workflow-engine/src/secret.rs:142-153`.

`SecretError::Refused.reason` est rendu brut et « material-free » n’est qu’un contrat textuel. `ActivityError::Refused(String)` est également brut. `Io` n’est plus `Io(String)` : c’est désormais `Io(std::io::Error)`, mais son message interne est toujours rendu verbatim. `crates/sbfb-workflow-engine/src/secret.rs:106-121`, `crates/sbfb-workflow-engine/src/activity.rs:138-147`.

Correction : codes d’erreur typés sans texte arbitraire à la frontière secret, avec cause interne non affichable.

### P2-4 — Le front jette silencieusement les payloads Rust directs et autorise les doublons

Rust porte trois entiers : `max_cost: u64`, `seq: u64`, `expires_at_ms: i64`. `crates/sbfb-workflow-engine/src/hitl.rs:168-180`. Le miroir TS les modélise volontairement en chaînes pour préserver les valeurs dépassant `2^53`. `web/src/pages/approvalsQueue.ts:47-76`.

Le choix est défendable et le futur mapper est explicitement exigé. `web/src/pages/approvalsQueue.ts:29-36`. Mais aujourd’hui :

- `isValidRequest` exige 14 chaînes ; `web/src/pages/approvalsQueue.ts:113-145` ;
- `sanitiseApprovalQueue` ne retourne ni erreur ni nombre d’items rejetés ; `web/src/pages/approvalsQueue.ts:195-212` ;
- si toutes les entrées portent les nombres JSON Rust, elles disparaissent et l’écran affirme « Aucune approbation en attente ». `web/src/pages/Approvals.tsx:204-211`.

Le rejet n’est donc **pas observable**. Le mapper de reliure devra convertir noms snake_case→camelCase et nombres→chaînes avant cette porte, et tout rejet devra déclencher une erreur visible plutôt qu’un faux état vide.

Les doublons `requestId` ne sont pas interdits : le garde exige seulement une chaîne non vide, le sanitizer ne déduplique pas, et une décision supprime toutes les entrées partageant l’identifiant. `web/src/pages/approvalsQueue.ts:147-155`, `:209-212`, `:236-241`. React reçoit aussi des clés et des IDs DOM identiques. `web/src/pages/Approvals.tsx:215-218`, `:235-247`.

### P2-5 — Domaine d’identité : valider à la compilation, ne pas le restreindre à HITL

Le fait est confirmé : `U+200D` appartient à la plage refusée `U+200B..U+200F`. `crates/sbfb-workflow-engine/src/hash.rs:162-170`. Le compilateur accepte néanmoins les clés arbitraires de `nodes`, ne scanne que les valeurs JSON et ne contrôle sur les clés que les noms de présentation réservés. `crates/sbfb-workflow-core/src/graph.rs:625-640`, `crates/sbfb-workflow-core/src/compile.rs:752-789`, `:1001-1011`.

`instance_id` n’échappe que `~` et `/`; le moteur refuse ensuite le node ID lors de `RunSpec::digest`. `crates/sbfb-workflow-core/src/compile.rs:659-670`, `crates/sbfb-workflow-engine/src/engine.rs:59-69`, `:278-285`.

Recommandation : **valider au bord de compilation**, avec erreur portant le document et le node ID, puis conserver le contrôle moteur en défense en profondeur. Restreindre le prédicat à HITL rouvrirait ses usages contre les séparateurs de digest, tabs et retours ligne des clés/side-outputs. `crates/sbfb-workflow-engine/src/hash.rs:82-104`.

Coût : faible à moyen pour le code et les tests, mais toute graph existante concernée doit être renommée, recompilée et re-signée — jamais normalisée silencieusement, puisque le graphe est épinglé par digest. `crates/sbfb-workflow-core/src/graph.rs:610-618`.

Le constat est documenté dans la review Codex, mais pas réellement routé par le lot canonique de l’audit plan : celui-ci ne nomme que la parité des deux prédicats, leur table et les extracteurs, pas la divergence compilation→moteur. `.planning/active/sprint83_phase_l1_codex_review.md:103-118`, `.planning/active/sprint83_audit_plan.md:202-218`.

### P2-6 — Zeroize : oui sur les retours ordinaires, non comme garantie universelle erreurs/panics

Les buffers reçus par `put` et `rotate` sont enveloppés avant leurs premiers chemins faillibles. `crates/nexus-core-rs/src/vault.rs:550-564`, `:594-606`. Les plaintexts ouverts et le check plaintext sont placés dans `Zeroizing`. `crates/nexus-core-rs/src/vault.rs:967-977`, `:1067-1090`.

Mais la garantie n’est pas absolue :

- `derive_kek1` utilise un `Box<[u8;32]>` nu pendant un appel faillible, puis ne construit le `SecretBox` qu’après succès ; `crates/nexus-core-rs/src/keystore.rs:1110-1122` ;
- `open_entry` n’enveloppe le buffer qu’après le retour de `decrypt`; `crates/nexus-core-rs/src/vault.rs:1080-1090` ;
- le profil release utilise `panic = "abort"`, donc les `Drop` de `Zeroizing`/`SecretBox` ne s’exécutent pas lors d’un panic release. `Cargo.toml:489-510` ;
- les 64 MiB de travail Argon2 ne sont pas essuyés, limite déjà honnêtement documentée. `docs/security/THREAT_MODEL.md:2019-2026`.

Je ne classe pas cela P1 : les claims parlent surtout de drop/retours et la mémoire Argon2 est bornée. La réponse stricte à « tous les chemins, paniques inclus » reste toutefois **non**.

## P3

La formulation « digest injectif » et « deux bindings différents ne collisionnent jamais » est mathématiquement fausse pour un digest BLAKE3 de 256 bits. La propriété réelle est la canonicalité de la préimage et une résistance cryptographique aux collisions. `crates/sbfb-workflow-engine/src/hitl.rs:242-253`, `docs/security/THREAT_MODEL.md:73`.

## Recomptes rank‑1

- **Mur Argon2id : PASS arithmétique.** `10^6 × 82 ms = 82 000 s = 22,78 h`; espérance uniforme ≈ `41 000 s = 11,39 h`. Les trois sites sont désormais alignés. La mesure de 82 ms elle-même n’a pas été rejouée conformément à la contrainte. `docs/security/DURESS.md:48-54`, `docs/rust/PATTERNS.md:1172-1181`, `:1554-1569`.
- **Borne 25 : PASS sur les sites et l’ordre.** Les onze sites traités sont dans le même ordre dans le threat model et l’audit plan ; le plan ajoute seulement `(12) borne 25` comme auto-référence. Les deux textes nomment ensuite le code et les doc-items `LocalFileKeyStore` comme non traités. `docs/security/THREAT_MODEL.md:2220-2247`, `.planning/active/sprint83_audit_plan.md:140-167`.
- **Liste CommonMark : PASS.** Les marqueurs source vont strictement de 1 à 27 : début `1/2/3`, puis `24`, `25`, `26`, `27`, sans inversion ni doublon. `docs/security/THREAT_MODEL.md:1984-1989`, `:2170`, `:2203`, `:2249`, `:2265`.
- **Gate coffre : PASS sur le compte.** J’ai recompté 20 marqueurs `#[test]` dans `vault.rs`, du premier à `:1139` au dernier à `:1829`; la matrice annonce bien 20. `.planning/active/sprint83_gate_matrix.md:44`.
- **Tuple HITL : PASS.** Le type Rust porte 15 champs ; la demande front en affiche 14, `decision` étant produit par le bouton. `crates/sbfb-workflow-engine/src/hitl.rs:143-182`, `web/src/pages/approvalsQueue.ts:80-128`.
- **Correction de mutation Unicode : PASS par inspection.** Les neuf plages multi-points ont désormais leurs témoins bas/haut dans les deux jumeaux, notamment `200B` et `200F`. `crates/sbfb-workflow-engine/src/hitl.rs:917-968`, `crates/nexus-core-rs/src/vault.rs:1736-1783`.
- **`llms.txt` : PASS sur le compte volatil.** Le document refuse désormais explicitement d’annoncer un nombre de bornes non recalculé par un gate. `docs/workflow/llms.txt:91-100`.

Aucune suite `cargo`/`npm` n’a été relancée et aucun fichier du dépôt n’a été écrit ou modifié pendant cette vérification.

