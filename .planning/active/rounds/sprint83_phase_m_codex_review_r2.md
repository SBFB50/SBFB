Verdict externe Round 2 : **FAIL — 2 P1 bloquants.**

Le re-hash ferme bien P1-1, mais son nouveau bras `DigestMismatch` réouvre P1-2. La correction Truth Stack ferme également une instance, pas la classe.

## P1 bloquants

### P1-A — le nouveau refus `DigestMismatch` peut rendre une fausse capability sans aucun test rouge

Le code staged rend correctement la capability à [artifact_acl.rs:1088](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1088), mais le test neuf ne vérifie que `is_some()` à [artifact_acl.rs:2177](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:2177).

Mutation mesurée : destruction de la capability reçue puis restitution de `[0;32]` sur ce seul bras.

```text
cargo nextest run -p sbfb-workflow-engine artifact_acl --locked
Summary: 22 tests run: 22 passed, 160 skipped
```

Cela réfute directement « les quatre bras […] restitution vérifiée FONCTIONNELLEMENT » à [sprint83_plan.md:1107](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1107).

Les trois autres bras sont bien discriminants :

- refus policy + fausse clé : **1 FAIL**, relecture refusée à [artifact_acl.rs:1801](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1801) ;
- blob absent + fausse clé : **1 FAIL**, relecture refusée à [artifact_acl.rs:2252](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:2252) ;
- refus `unseal` + fausse clé : **1 FAIL**, artefact tiers illisible à [artifact_acl.rs:2091](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:2091).

Correctif minimal : récupérer `refusal.capability`, puis exiger qu’elle rouvre `other`, comme les autres branches.

Le bras final « victime restaurée » est discriminant comme contrôle de fixture : restaurer `Vec::new()` donne **1 FAIL** à [artifact_acl.rs:2193](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:2193). Mais il ne contrôle pas la capability rendue : il utilise `victim_cap`, jamais passée à `delete`, et `victim_bytes`, sauvegardé hors de l’opération. C’est pourquoi il reste vert sous la mutation P1-A.

### P1-B — la justification circulaire n’a été corrigée qu’en §M

La correction est juste en §M à [sprint83_plan.md:1118](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1118), mais deux surfaces normatives restent périmées :

- §P affirme encore « n’obtiendra jamais » à [sprint83_plan.md:1414](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1414) ;
- le rustdoc de `from_hex` dit encore qu’un pair peut ne jamais obtenir les octets locaux à [artifact_acl.rs:274](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:274).

Cela contredit les deux chemins publics explicités à [THREAT_MODEL.md:2324](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2324).

Mesure documentaire :

```text
INDEX_PLAN_STALE=1
INDEX_RUSTDOC_STALE=1
CLASS_STALE_TOTAL=2
```

Après correction temporaire des deux instances : `CLASS_STALE_TOTAL=0`.

La liste §M.3 qui prétend décrire le livrable réel oublie également `delete_refuses_bytes_that_do_not_match_the_address` à [sprint83_plan.md:1164](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1164) :

```text
CODE_TESTS=22
PLAN_DECLARED_AFTER_BRACE_EXPANSION=21
MISSING=delete_refuses_bytes_that_do_not_match_the_address
```

C’est P2 isolément, mais cela confirme que le correctif Truth Stack n’a pas été balayé par classe.

## Ce qui est réellement fermé

- **Adresse de contenu : PASS classe.** Les seuls consommateurs de production de `get_unauthenticated` sont `read` et `delete`, avec re-hash respectivement à [artifact_acl.rs:987](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:987) et [artifact_acl.rs:1088](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1088). Retirer celui de `delete` donne **0/1**, comme attendu.
- **Contrat `unseal` : PASS borné.** Obligation écrite à [artifact_acl.rs:751](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:751), borne 11 continue à [THREAT_MODEL.md:2454](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2454), et `HostArtifactSealer` transmet bien la clé à AES-GCM à [artifact_sealer.rs:89](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/artifact_sealer.rs:89).
- **`StoredBytes` : PASS documentaire.** La distinction magasin/chiffrement/authentification est exacte à [THREAT_MODEL.md:2420](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2420).
- **Threat row : PASS.** `T-ARTIFACT-DELETION-THEATRE` porte bien adresse + possession à [THREAT_MODEL.md:2369](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2369).
- **Index : PASS mécanique.** 19 entrées staged, `r7` et le rapport Codex R1 inclus, aucun diff non staged.

## Conséquences non traitées

| Conséquence | Verdict |
|---|---|
| Oracle absent=`DigestMismatch`, mauvaise clé=`SealerRefused` | **Non bloquant, accepté.** La distinction est explicitement gardée : la fusion vers `DigestMismatch` fait échouer `deleting_with_a_foreign_capability_is_refused`. Elle ne révèle rien qu’un détenteur du digest ne puisse déjà demander aux chemins publics, et `delete` exige en plus une référence localement mintée à [artifact_acl.rs:862](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:862). |
| Coût O(n), stored + plaintext | **Carry légitime, non bloquant.** `get` matérialise un `Vec` à [artifact_acl.rs:1082](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1082), puis `unseal` rend un `Zeroizing<Vec>` à [artifact_acl.rs:1094](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1094). Quotas/backpressure sont déjà routés vers AI2 à [sprint83_plan.md:1208](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1208). |
| Panic/unwind après prise de capability | **Carry P3, non bloquant M.** Sonde `catch_unwind` : le panic traverse, aucun `DeleteRefusal`, tag encore présent. En production workspace, le profil release est `panic = "abort"` à [Cargo.toml:489](/C:/Users/FlowUP/Documents/Code/nexus/Cargo.toml:489), et il n’existe aucun call-site. À borner avant un consommateur susceptible de catcher un unwind. |
| `untag(digest)` vs `delete_tag(nom)` | **Carry P2 légitime, mais à router explicitement vers Y/premier câblage.** `add_bytes` abandonne le nom auto-généré à [blobs.rs:77](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/blobs.rs:77), tandis que `delete_tag` exige le nom à [blobs.rs:130](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/blobs.rs:130). Sonde réelle : supprimer le tag sous le hex du digest laisse inchangé le nombre de tags portant ce hash, **1 passed**. Ce n’est pas un blocage de confidentialité M — la clé est toujours détruite — mais Owner/Trigger/Exit doivent être ajoutés conformément à [PROCESS.md:112](/C:/Users/FlowUP/Documents/Code/nexus/docs/agent/PROCESS.md:112). |

Autre résidu mesuré de r7, non causé par ces correctifs : `MalformedDigest` recopie toujours l’entrée entière à [artifact_acl.rs:296](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:296). Sonde : `INPUT_BYTES=200000`, `ERROR_BYTES=200059`. Cela reste **P2 carry avant Phase P**, pas un bloqueur M sous DS-1.

## Verdict par livrable

| Livrable | Verdict R2 |
|---|---|
| `artifact_acl.rs` | **FAIL** — re-hash correct, mais nouveau bras de restitution non discriminant + rustdoc `from_hex` périmé. |
| `artifact_seal.rs` | **PASS — 11/11**. |
| `artifact_sealer.rs` | **PASS — 4/4**. |
| `secret.rs` canari | **PASS borné** ; le double `XorSealer` non authentifiant reste test-only et explicitement cité comme limite du contrat. |
| Trois registrations / re-exports | **PASS**. |
| `ArtifactRef` / sérialisation | **PASS borné**, avec les P2 r7 conservés. |
| `THREAT_MODEL.md` | **PASS sur les quatre corrections R1** ; les carrys ci-dessus restent à router/documenter. |
| `sprint83_plan.md` | **FAIL** — quatre restitutions fonctionnelles faux, deux justifications périmées, test neuf absent de §M.3. |
| `sprint83_gate_matrix.md` | **PASS mécanique**. |
| Rapports / index | **PASS mécanique**, mais le dernier verdict externe reste FAIL. |
| `WORKFLOW_AGENTS_OVERVIEW.md` | **PASS**. |

Mesure finale sur l’export exact de l’index : **22 + 11 + 4 = 37**, toutes vertes sans mutation. Je n’ai pas rejoué le workspace complet : la baseline fournie **2466 / 7 skipped** reste non réfutée.

Toutes les copies, probes et cibles temporaires ont été supprimées. État final : **19 staged, diff non staged vide, aucun processus Cargo/Rustc**.

