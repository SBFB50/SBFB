Mon verdict est **FAIL** : la Phase L1 ne peut pas être clôturée ni committée comme phase achevée. Les primitives cryptographiques et de stockage sont sérieuses, les contraintes Day‑0 sont respectées, mais les trois promesses autoritaires restent incomplètes. Le coffre n’est relié à aucun adaptateur d’effet réel, l’approbation HITL peut être auto-attestée sans vérification de signature, et le canari ne traverse aucune surface de production. Les gates `G-SECRET-VAULT`, `G-SECRET` et `G-HITL` doivent rester ouvertes.

## Statut des trois livrables

| Livrable | Statut | Verdict factuel |
|---|---|---|
| Coffre hôte générique | **PARTIEL** | Le coffre AES-GCM/Argon2, le `SecretRef` opaque et le résolveur existent, mais il n’existe aucun appelant de production réalisant le binding juste avant l’effet. Audit, concurrence et révocation en vol restent incomplets. [`secret.rs:23`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:23), [`secret.rs:202`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:202), [`vault.rs:78`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:78), [`vault.rs:655`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:655) |
| HITL rendu et signé par l’hôte | **PARTIEL** | Le tuple de 15 champs, son digest et la signature Ed25519 existent. Leur composition daemon→engine n’existe pas. Le type témoin peut être construit à partir d’un digest auto-calculé, sans preuve cryptographique. La file front est vide en production et sans producteur. [`hitl.rs:250`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:250), [`hitl.rs:356`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:356), [`hitl_approval.rs:47`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/hitl_approval.rs:47), [`approvalsQueue.ts:221`](/C:/Users/FlowUP/Documents/Code/nexus/web/src/pages/approvalsQueue.ts:221) |
| Canari absent partout | **GAP** | Le test construit manuellement cinq représentations qui ne contiennent pas le secret. Il ne passe ni par un adaptateur, ni par les stores réels, ni par les exports, erreurs, prompts, traces ou datasets. [`secret.rs:458`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:458), [`THREAT_MODEL.md:2065`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2065) |

## Constats bloquants

### P1 — L’autorité HITL est contournable dans le type public

Le mécanisme exact est :

```rust
let witness =
    HostVerifiedApproval::attest_verified_signature(effect.digest()?);
decide_ref(&effect, &witness, ...)?;
```

`attest_verified_signature` est public, accepte n’importe quelle chaîne et n’appelle aucune primitive cryptographique. `decide_ref` vérifie ensuite uniquement l’égalité avec `effect.digest()`. Aucun type ne prouve que `verify_hitl_approval` a été exécuté. [`hitl.rs:250-280`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:250), [`hitl.rs:394-421`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:394).

Le helper de test `host_attests` affirme en commentaire « verify signature then attest », mais son corps calcule directement le digest et l’atteste. Le test valide donc précisément le chemin non autorisé. [`hitl.rs:457-461`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:457).

Conséquence réelle :

- ce n’est pas encore une vulnérabilité exploitable dans le daemon, car il n’existe aucun câblage de production ;
- c’est néanmoins un **blocage P1 de conception d’autorité** : le futur appelant est encouragé à considérer le témoin comme une preuve alors qu’il est auto-constructible ;
- la phrase « caller cannot skip signature verification » de [`hitl.rs:33-45`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:33) est fausse ;
- la signature de [`hitl_approval.rs:47-73`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/hitl_approval.rs:47) prouve que la clé d’identité du nœud a signé le digest, pas qu’un humain ou le chrome hôte a effectivement approuvé.

Le type devrait être créé exclusivement par une fonction qui prend le digest recomputé, la signature et la clé publique, vérifie cryptographiquement puis retourne le témoin. À défaut, la documentation doit parler de « primitive non reliée », jamais d’autorité fermée.

Un second bord de rendu reste ouvert : `BindingMismatch` réaffiche verbatim le digest `approved`. Un appelant peut attester `"00\nWARN forged"` puis provoquer un mismatch et injecter une ligne de log. `sanitise_untrusted` ne protège que le mismatch de capacité. [`hitl.rs:283-293`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:283), [`hitl.rs:329-354`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:329).

### P1 — Le test canari est non discriminant

Le test :

1. crée lui-même un `ActivityRequest`, un reçu et un événement ;
2. n’insère jamais le matériel dans ces objets ;
3. « empoisonne » ensuite manuellement un champ pour démontrer que `contains` fonctionne.

Cela ne teste ni la non-sérialisation de `BoundSecret`, ni un trajet de production. La mutation suivante laisserait le test vert tout en introduisant une fuite directe :

```rust
impl serde::Serialize for BoundSecret {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.expose())
    }
}
```

Le commentaire affirmant que le bras rouge couvre l’absence de `Serialize` est donc incorrect. [`secret.rs:538-592`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:538).

Autre fuite non couverte : `SecretError::Refused` rend `reason` verbatim malgré la simple convention « material-free by contract ». Un provider peut mettre le canari dans son message d’erreur. [`secret.rs:106-120`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:106). `ActivityError::Refused(String)` et `Io(String)` possèdent le même risque. [`activity.rs:113-145`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/activity.rs:113).

Le remplacement minimal exact pour verrouiller la non-sérialisation est un doctest compile-fail :

```rust
/// ```compile_fail
/// use sbfb_workflow_engine::BoundSecret;
///
/// fn bound_secret_must_not_serialize(secret: &BoundSecret) {
///     let _ = serde_json::to_vec(secret).unwrap();
/// }
/// ```
```

Mais le vrai WA‑T1 doit être un test d’intégration hôte qui :

- exécute un effet réel avec uniquement un `secret_ref` en entrée ;
- confirme hors bande que le faux provider a reçu le matériel ;
- collecte ensuite les véritables DB de runs, inputs, événements, snapshots, signaux, erreurs, logs, traces, prompts, `SBFB_EFFECT_INPUT`, artifacts, exports et datasets ;
- recherche le canari dans chaque sortie.

Ce test ne peut pas être écrit contre une API de production actuelle : l’adaptateur JIT n’existe pas. C’est la preuve que `G-SECRET` ne peut pas être fermé.

### P1 — La file HITL n’est pas une file persistante et utilisable

L’état est un tableau dans le module JavaScript. Il survit au démontage d’une iframe tant que le document hôte reste vivant, mais pas au rechargement du shell, à un crash ou à un nouveau processus. Le code le reconnaît correctement. [`approvalsQueue.ts:21-24`](/C:/Users/FlowUP/Documents/Code/nexus/web/src/pages/approvalsQueue.ts:21).

Il manque cependant :

- un producteur de requêtes de production ;
- une souscription : `Approvals` prend un snapshot initial avec `useState(getHostQueue)` mais ne reçoit pas les nouvelles demandes ; [`Approvals.tsx:118-121`](/C:/Users/FlowUP/Documents/Code/nexus/web/src/pages/Approvals.tsx:118) ;
- une route de décision vers le daemon ;
- un accès fiable depuis l’app plein écran : la route existe, mais n’est pas dans la navigation et le chrome peut être masqué ;
- une persistance/reprise hôte.

Le test E2E qui constate `remote-iframe-element == 0` prouve seulement que la page `/approvals` ne contient pas d’iframe. Il ne crée pas une demande, ne tue pas l’iframe productrice et ne vérifie pas que la demande reste actionnable. [`workflow-hub-a11y.spec.ts:221-228`](/C:/Users/FlowUP/Documents/Code/nexus/web/e2e/workflow-hub-a11y.spec.ts:221).

Deux défauts supplémentaires :

- `sanitiseApprovalQueue` n’interdit pas deux entrées ayant le même `requestId`. Une décision supprime alors les deux via `filter`, tandis que React et les attributs ARIA reçoivent des clés/IDs dupliqués. [`approvalsQueue.ts:195-212`](/C:/Users/FlowUP/Documents/Code/nexus/web/src/pages/approvalsQueue.ts:195), [`approvalsQueue.ts:236-241`](/C:/Users/FlowUP/Documents/Code/nexus/web/src/pages/approvalsQueue.ts:236) ;
- le sanitizer accepte les champs inconnus et retourne l’objet original. Une propriété `decision: "granted"` ou un futur champ non validé survit donc au passage supposé « pending only ».

### P1 — Le durcissement Unicode casse un domaine d’identité partagé

Les deux nouveaux prédicats couvrent bien un surensemble strict de `output_filter::is_invisible_char` :

- `U+200B..U+200F`,
- `U+2060`,
- `U+FEFF`,
- PUA BMP et plans 15/16,
- TAG `U+E0020..U+E007F`,
- plus contrôles, soft hyphen, bidi, séparateurs et plusieurs caractères invisibles supplémentaires.

Le code satisfait donc la claim point par point. [`hash.rs:162-187`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hash.rs:162), [`vault.rs:804-835`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:804).

Mais la validation est appliquée à tout le moteur, pas seulement au bord HITL. Le compilateur accepte aujourd’hui des IDs Unicode arbitraires comme clés de graphe et ne valide pas les clés JSON avec ce prédicat. Un ID tel que `👩‍💻` contient `U+200D` : il peut être compilé, puis refusé seulement lors de l’instanciation du moteur. [`graph.rs:620-640`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/graph.rs:620), [`compile.rs:740-789`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/compile.rs:740), [`engine.rs:60-67`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:60).

Il faut soit valider le même contrat au bord de compilation avec une erreur explicite et une stratégie de migration, soit limiter ce refus aux données effectivement rendues par HITL.

La couverture « surensemble point par point » n’est pas prouvée par les tests : remplacer `'\u{200B}'..='\u{200F}'` par `'\u{200B}'` dans les deux prédicats laisserait les tests actuels verts alors que `U+200C..U+200F` repasseraient.

## Coffre : résultats adversariaux détaillés

- **PIN erroné sur coffre vide — CONFIRME corrigé.** `init` crée une entrée de contrôle et `unlock` doit toujours la ouvrir. Un mauvais PIN ne retourne plus de handle utilisable. [`vault.rs:327-400`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:327).

- **Suppression/rejeu partiel — CONFIRME corrigé.** L’empreinte authentifiée de l’ensemble des références détecte la suppression d’une seule entrée et le rejeu isolé d’un ancien ciphertext. [`vault.rs:905-1009`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:905).

- **Rollback global — GAP assumé.** Un ancien fichier cohérent, header et empreinte compris, reste rejouable. La rotation peut être annulée par rollback complet.

- **Écrivains concurrents — GAP majeur.** Deux writers lisant le même epoch peuvent chacun passer le contrôle avant le rename ; le second écrase le premier et peut ressusciter une entrée supprimée. Il n’existe pas de verrou couvrant lecture→écriture. [`vault.rs:675-707`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:675).

- **Révocation en vol — GAP nouveau.** `bind` vérifie l’epoch, puis retourne le matériel. Une rotation ou suppression peut être commitée entre cette lecture et l’appel provider ; l’ancienne valeur reste servie après la révocation. [`vault.rs:470-504`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:470), [`vault.rs:630-638`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:630).

- **Séparation de domaine — CONFIRME.** `derive_vault_kek` réutilise bien `DOMAIN_KEYSTORE_V1`, avec séparateurs/contexte et longueur de préimage distincts de `combine_keks`; aucune collision de préimage inter-formule concrète n’apparaît. [`vault.rs:403-429`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:403), [`keystore.rs:1113-1131`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/keystore.rs:1113).

- **Nonce AES-GCM — PARTIEL.** Le nonce aléatoire 96 bits est correct, mais aucune borne d’invocations par KEK, stratégie de rekey ou analyse birthday n’est documentée. [`vault.rs:1012-1060`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:1012).

- **Zeroize — PARTIEL.** Le `Vec<u8>` déchiffré est immédiatement placé sous `Zeroizing`; `body.clone()` et `serde_json::to_vec` ne copient que références, nonces et ciphertexts. En revanche restent hors protection : le PIN appartenant à l’appelant, toute copie créée par `expose().to_vec()`, les 64 MiB de mémoire de travail Argon2 et des états internes AES/GHASH dérivés de la clé. [`vault.rs:949-956`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:949), [`THREAT_MODEL.md:2019-2026`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2019).

- **Audit — GAP.** `inventory()` n’est pas un journal d’audit : pas d’identité, date, action, succès/refus ou chaîne append-only. Le livrable « rotation/suppression/zeroize/audit » n’est pas satisfait. [`vault.rs:655-673`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:655).

- **0600 Unix — PARTIEL.** Le chmod est effectué après la création/synchronisation du fichier ; il existe une fenêtre où le ciphertext final ou temporaire hérite du mode initial. La donnée reste chiffrée, mais « atomic 0600 » serait trop fort. [`vault.rs:750-773`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:750).

## HITL : exactitude du rendu

`ExpiryValue` est désormais **CONFIRME** contre les coercions objet : la branche examine le type avant toute conversion, n’appelle pas `Number()`/`String()` sur un objet et affiche toujours la préimage brute. [`Approvals.tsx:326-369`](/C:/Users/FlowUP/Documents/Code/nexus/web/src/pages/Approvals.tsx:326).

Le tuple complet n’est cependant pas encore WYSIWYS :

- Rust signe `effect_class`, `max_cost` et `sequence` comme nombres ;
- TypeScript les stocke comme chaînes ;
- le sanitizer accepte `"01"`, `"+1"` ou `"-0"`, alors que la valeur Rust/JCS correspondante serait `1` ou `0` ;
- le texte affirme que les octets affichés sont ceux signés, ce qui n’est pas établi. [`hitl.rs:163-175`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:163), [`approvalsQueue.ts:47-51`](/C:/Users/FlowUP/Documents/Code/nexus/web/src/pages/approvalsQueue.ts:47), [`Approvals.tsx:312-314`](/C:/Users/FlowUP/Documents/Code/nexus/web/src/pages/Approvals.tsx:312).

De plus, un `<dd>` HTML replie les espaces : `"admin"` et `"admin "` ont des digests différents mais peuvent être visuellement indistinguables. Le threat model admet cette borne. [`THREAT_MODEL.md:2189-2202`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2189).

## Conditions binaires L1.4

| Condition | Statut |
|---|---|
| Canari absent de toutes les surfaces d’export | **GAP** — cinq chaînes synthétiques et le blob vault ne représentent pas toutes les surfaces. |
| Toute mutation invalide l’approbation | **PARTIEL** — les 15 champs influencent le digest et la signature, mais l’origine autorisée de la signature n’est pas imposée et la représentation front n’est pas canonique. |
| File survivant à l’iframe morte et accessible | **PARTIEL** — propriété de module dans le document hôte et page minimalement accessible, mais aucun scénario réel iframe→requête→mort→décision, aucune persistance shell et aucun producteur. |

La matrice requalifie ces conditions en « cinq rendus engine », « état module » et « backends ultérieurs ». Cette adaptation est honnête comme constat de portée, mais elle ne peut pas remplacer silencieusement les critères binaires gelés du plan. [`sprint83_plan.md:1003-1005`](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1003), [`sprint83_gate_matrix.md:94-115`](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_gate_matrix.md:94).

## Six rows du threat model

| Row | Statut externe |
|---|---|
| `T-SECRET-REF-EXFIL` | **PARTIEL** — opaque ref et non-sérialisation actuelle réelles, mais canari synthétique, aucune route JIT et erreurs libres. La fermeture « définitive » est sur-affirmée. |
| `T-SECRET-VAULT-LIFECYCLE` | **PARTIEL** — wrong-PIN et tampering partiel corrigés ; rollback global, concurrence, révocation en vol, audit et wipe physique restent ouverts. |
| `T-HITL-APPROVAL-TAMPER` | **PARTIEL** — digest sensible aux 15 champs, mais témoin auto-attestable et composition absente. |
| `T-HITL-FORGE-REPLAY` | **PARTIEL** — vérification Ed25519 stricte disponible ; consommation du nonce, authentification du principal, coût cumulé et routage sont absents. |
| `T-HITL-QUEUE-CONFUSED-DEPUTY` | **PARTIEL** — l’iframe actuelle n’a aucun verbe d’approbation, mais la file utilisable et son autorité de décision n’existent pas encore. |
| `T-HITL-COERCION-A11Y` | **PARTIEL** — helper expiry et fixture clavier confirmés ; exactitude numérique, espaces rendus et test avec technologie d’assistance restent ouverts. |

Tous les noms de tests Rust cités dans §19.4 existent. Le changelog actuel ne dit plus « 17 tests » : il annonce 44, cohérent avec l’énumération actuelle. Il n’y a donc plus de nom de test fabriqué, mais ces tests ne démontrent pas le câblage absent. [`THREAT_MODEL.md:2649-2650`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2649).

La phrase ultérieure affirmant que la phase « opens+closes » les trois gates contredit les limites détaillées du même document. [`THREAT_MODEL.md:2653`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2653).

## Corrections postérieures à r4

- `sanitise_untrusted` et l’élargissement Cf/PUA/TAG : **PARTIEL**, car le set est correct mais `BindingMismatch.approved` reste rendu sans filtre et le domaine global a régressé.
- `ExpiryValue` : **CONFIRME**.
- Test direct de `sanitiseApprovalQueue` : **CONFIRME** ; il passe bien par la fonction exportée, pas par `__seedHostQueueForTest`. Sa couverture n’inclut toutefois ni doublons ni champs inconnus.
- Self-tests `BRIDGE-PARITY` et `HITL-PARITY` : **CONFIRME** non tautologiques. Relâcher l’ancre Bridge pour capturer tout `BridgeMethodSchema` injecterait `trap_token` et ferait échouer le self-test ; relâcher l’ancre HITL pour toute occurrence de `HitlBinding` capturerait `avant`. [`check-frontier-contracts.sh:361-410`](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:361), [`check-frontier-contracts.sh:449-491`](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:449).
- `HitlValidity::len()` : **CONFIRME** supprimée sans appelant ; `is_empty()` reste correct. [`capability.rs:655-675`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:655).
- `SecretRef` : **CONFIRME**, il ne dérive plus `Deserialize`; l’hypothèse de bypass du prompt n’est plus vraie. `SealedEntry` possède `deny_unknown_fields`. [`secret.rs:78-91`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:78), [`vault.rs:731-740`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:731).
- `HitlBinding` : ses champs publics contournent `new`, mais sans collision JCS ni contournement de signature. Le risque réel est une validation incohérente et un rendu trompeur, pas une égalité cryptographique forgée. [`hitl.rs:120-234`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:120).
- Le qualificatif « injective » pour BLAKE3 est mathématiquement faux sur un domaine non borné ; il faut écrire « résistant aux collisions et sensible aux champs ». [`hitl.rs:237-238`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/hitl.rs:237).

## Contraintes Day‑0

Toutes sont **CONFIRMÉES** sur l’arbre vivant :

- aucun package nouveau dans `Cargo.lock`, seulement deux arêtes vers `zeroize`, dont le package existait déjà ;
- census `DOMAIN_*_V*` = 29 ;
- aucun `DOMAIN_HITL_*`, `DOMAIN_SECRET_*` ou `DOMAIN_VAULT_*` ;
- aucun bump wire ;
- `DOMAIN_KEYSTORE_V1` réutilisé ;
- engine keyless, sans environnement, FS ambiant ou signature ;
- aucun changement daemon/factory/CSP ; la sandbox demeure scellée dans [`csp.rs:24-33`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/csp.rs:24).

L’adaptation keyring reste toutefois **PARTIELLEMENT documentée**. Le mock est correctement reconnu dans le preflight et plusieurs docs, et `DURESS.md` a bien été traité. Deux claims subsistent :

- le workspace root présente encore keyring comme seconde couche sans qualification mock ; [`Cargo.toml:314-325`](/C:/Users/FlowUP/Documents/Code/nexus/Cargo.toml:314) ;
- le commentaire « Default production keystore: blob + OS keyring » reste trop fort ; [`keystore.rs:466-474`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/keystore.rs:466) ;
- l’en-tête du test d’intégration affirme que `Entry::new` échoue en headless, alors que le mock accepte la création et échoue seulement au probe inter-handle. [`keystore_integration.rs:1-13`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/tests/keystore_integration.rs:1).

## État de preuve et décision de commit

L’état final inspecté est `HEAD 9134af64`, avec **40 entrées de worktree** : 25 fichiers suivis modifiés et 15 non suivis. L’inventaire « 17 fichiers » n’est donc plus le périmètre vivant.

Le dernier review présent est toujours `r6` avec `## Verdict: FAIL` et cinq P1 au moment de son gel. Plusieurs fichiers ont ensuite encore été corrigés, sans nouvelle revue canonique. [`sprint83_phase_l1_review_r6.md:3-12`](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_l1_review_r6.md:3).

Je n’ai relancé aucune suite Cargo. Aucun conteneur `sbfb-ci` actif ni preuve verte finale du worktree post-r6 n’était observable ; cela signifie **non vérifié**, pas « Docker échoué ».

Avant un commit de clôture L1, il faut au minimum :

1. relier le résolveur secret à un véritable adaptateur JIT et fermer les erreurs libres ;
2. rendre impossible la création d’un témoin HITL sans vérification cryptographique daemon ;
3. remplacer le chemin `HumanReview`/signal actuel par la composition signée ;
4. ajouter producteur, souscription, décision et validation canonique à la file hôte ;
5. résoudre le contrat Unicode partagé ;
6. ajouter le test canari sur les stores/exports réels ;
7. verrouiller les écrivains du coffre et définir la sémantique de révocation en vol ;
8. corriger les affirmations de fermeture, puis obtenir un run final stable — notamment Unix — et un nouveau verdict indépendant `PASS`.

Le rollback global, le wipe physique, la mémoire de travail Argon2, la borne nonce/rekey, le mock keyring et l’absence de validation AT complète peuvent être documentés comme bornes honnêtes seulement si le plan autoritaire accepte explicitement cette réduction de portée. La matrice de gates seule ne peut pas modifier une condition binaire gelée.

Aucun fichier du dépôt n’a été modifié durant cette vérification.

