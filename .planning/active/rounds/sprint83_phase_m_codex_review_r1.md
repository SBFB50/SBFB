Verdict externe : **FAIL — r7 est réfuté.** J’ai retenu trois classes P1 techniques, plus un P1 de Truth Stack/process. `PASS-PENDING` n’est pas défendable dans l’état actuel.

## P1 bloquants

### P1-1 — `delete` vérifie la clé contre les octets rendus, pas contre l’artefact nommé

`read` re-hashe les octets avant `unseal` ([artifact_acl.rs:960](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:960)). `delete` saute ce contrôle et appelle directement `unseal` ([artifact_acl.rs:1052](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1052)).

Sonde mesurée :

1. publication d’une victime et d’un artefact tiers ;
2. sous le digest victime, le store rend le chiffré valide du tiers ;
3. `delete(victim, other_capability)` rend `Ok(())` ;
4. après restauration d’une copie survivante du chiffré victime, la vraie capability victime le rouvre encore.

Commande :

```text
cargo nextest run -p sbfb-workflow-engine \
  external_probe_delete_accepts_a_foreign_key_for_misaddressed_bytes --locked
```

Sortie : **1 passed**.

La capability étrangère n’ouvre donc pas « l’artefact » : elle ouvre seulement les octets arbitraires rendus par la couture. C’est exactement du deletion theatre. Cela réfute :

- [THREAT_MODEL.md:2369](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2369), qui affirme que la capability doit ouvrir l’artefact ;
- [r7:707](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_m_review_r7.md:707), qui affirme faussement que `read` **et `delete`** re-hashent.

Correctif minimal : recopier avant `unseal` le contrôle de contenu de `read`, et rendre la capability avec `DigestMismatch`.

### P1-2 — « tout refus rend la capability » n’est pas gardé sur deux branches

Le code actuel est correct, mais deux contrôles restent non discriminants :

- refus de policy à [artifact_acl.rs:1038](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1038) ;
- blob absent à [artifact_acl.rs:1052](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1052).

Mutation 1 : sur le refus policy, destruction de la capability reçue et retour d’une capability `[0; 32]` fabriquée.

```text
21 tests run: 21 passed, 160 skipped
```

Mutation 2 : même substitution sur le chemin blob-absent.

```text
21 tests run: 21 passed, 160 skipped
```

Les tests ne vérifient que `capability.is_some()` à [artifact_acl.rs:1746](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1746) et [artifact_acl.rs:2104](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:2104). Seul le refus d’`unseal` vérifie réellement l’identité fonctionnelle de la clé en relisant l’artefact tiers ([artifact_acl.rs:2021](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:2021)).

Cela réfute « **Tout refus** rend… » dans [sprint83_plan.md:1104](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1104) comme garantie testée.

### P1-3 — troisième porte de couture : `unseal` peut ignorer la capability

La borne 7 ne couvre que `seal` ([THREAT_MODEL.md:2411](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2411)). Rien d’équivalent n’impose à `ArtifactSealer::unseal` d’authentifier la capability ([artifact_acl.rs:746](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:746)).

Or :

- `read` lui délègue le droit de lecture ;
- `delete` l’utilise comme unique preuve de possession ;
- `ReadCapability::new` est public.

Sonde avec un implémenteur type-correct qui ignore la capability : une capability vide inventée lit puis supprime un artefact local.

```text
cargo nextest run -p sbfb-workflow-engine \
  external_probe_unseal_may_ignore_the_capability --locked
Summary: 1 passed
```

Le dépôt contient déjà un exemple compilé d’`unseal` ignorant son argument à [secret.rs:666](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:666).

Le `HostArtifactSealer` réel, lui, est correct : il transmet la clé à AES-GCM ([artifact_sealer.rs:83](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/artifact_sealer.rs:83)). Le défaut porte sur le contrat public de fondation et sur le statut « fermé ».

Un newtype `StoredBytes` peut fermer à faible coût la porte brute `ArtifactBlobs::put`, puisqu’il n’existe aucun implémenteur de production. Il **ne peut pas** prouver qu’un implémenteur externe a chiffré, ni que `unseal` vérifie la clé. L’affirmation selon laquelle le même newtype fermerait les deux portes est donc excessive.

### P1-4 — Truth Stack et index

L’affirmation « tout est dans l’index » est fausse :

- **17** entrées sont stagées ;
- `r7` est `??`, non suivi ;
- le dernier rapport présent dans l’index est donc **r6**, dont le verdict est `FAIL — 1 P1`.

De plus, r7 affirme successivement « aucune entrée untracked » puis reconnaît que son propre fichier restera untracked ([r7:645](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_m_review_r7.md:645), [r7:653](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_m_review_r7.md:653)).

Autre phrase fausse dans le plan actif : un pair tenant le JSON « n’obtiendra jamais » les octets d’un artefact local ([sprint83_plan.md:1111](C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1111)). Le modèle répète pourtant que tout pair tenant le digest peut récupérer le chiffré par les deux chemins publics ([THREAT_MODEL.md:2324](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2324)). La bonne justification de `from_hex` est circulaire : il faut décoder le digest **avant** de pouvoir demander ces octets.

## Conséquences supplémentaires de `delete`

La borne 9 n’est pas exhaustive :

- oracle maintenu : absent ⇒ `DigestMismatch`, présent mais mauvaise clé ⇒ `SealerRefused` ;
- `delete` charge le blob entier puis produit un plaintext entier uniquement pour le jeter : coût O(n), avec au moins deux buffers de taille artefact ;
- panic/unwind dans la couture après prise de la capability peut la détruire sans `DeleteRefusal` ;
- `ArtifactBlobs::untag(digest)` ne correspond pas encore au `BlobsClient` visé : `add_bytes` jette le nom auto-généré du tag ([blobs.rs:77](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/blobs.rs:77)), tandis que `delete_tag` exige ce nom ([blobs.rs:130](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/blobs.rs:130)). Ce n’est pas une demande de reliure, mais un carry de couture manquant.

## Verdict par livrable

| Livrable | Verdict externe |
|---|---|
| `artifact_acl.rs` | **FAIL** — P1 delete/adresse, P1 contrôles de restitution, P1 contrat `unseal`. |
| `artifact_seal.rs` | **PASS** — AES-256-GCM, clé/nonce frais, AAD, cadrage et erreurs tiennent. |
| `artifact_sealer.rs` | **PASS pour l’implémentation concrète** — la clé et le mode atteignent bien AES-GCM. |
| `secret.rs` canari | **PASS** — mutation `Local → Published` rejouée : test rouge, `Some("raw text")`. |
| Trois lignes de registration | **PASS**. |
| `ArtifactRef` / sérialisation | **PASS borné** — forme émise épinglée, origine volontairement hors wire, `PartialEq` incluant l’origine cohérent avec `addresses_same_artefact`. Le décodage mode/rétention reste incomplet, déjà P2 de r7. |
| `THREAT_MODEL.md` | **FAIL** — fermeture de `delete` fausse et borne `unseal` absente. |
| `sprint83_plan.md` / row 26 | **FAIL sémantique** malgré commandes correctes — deux contrôles P1 restent verts sous perte de clé. |
| Rapports de rounds / index | **FAIL process** — r7 absent de l’index, dernier verdict stagé = r6 FAIL. |

## Mesures qui tiennent

Sur l’export exact de l’index :

- `artifact_acl` : **21/21** ;
- `artifact_seal` : **11/11** ;
- `artifact_sealer` : **4/4** ;
- somme : **36** ;
- ajouts `#[test]` dans le diff : **36**, `secret.rs` +0 ;
- aucun manifeste/dépendance modifié ;
- aucun bump `*_VERSION`, aucun `DOMAIN_*` neuf ;
- `check-frontier-contracts.sh`, lancé sous Git Bash explicite : **clean, DOMAIN census 29 frozen**.

Je n’ai pas rejoué le workspace complet à 2465 tests : je ne réfute pas cette baseline, mais elle ne détecte aucun des P1 ci-dessus. Le dépôt n’a pas été modifié et toutes les copies/mutations temporaires ont été supprimées.

