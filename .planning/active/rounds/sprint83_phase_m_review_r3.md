# Review — Sprint 83 Phase M (artefacts, ACL et confidentialite) — round 3

> **SUPERSEDED sur un point** : le delta de tests prescrit plus bas (+30 / 2459) est
> perime — les rounds suivants ont ajoute des tests, et le chiffre a encore bouge apres.
> Ne recopie AUCUN compteur depuis une archive de round : recompte-le a la source au
> moment de rediger le body (`cargo nextest run --workspace --locked`, plus les trois
> commandes de gate de la row 26, une par crate). Seule la baseline est stable : **2429**,
> fin de Phase L1, jamais le 2167 perime de CLAUDE.md. Le corps de ce rapport n'est PAS
> retouche : c'est une archive exacte a sa date.

## Verdict

**FAIL** — 2 P1 distincts retenus apres refutation majoritaire (3 jures adversariaux,
40 findings bruts issus de 9 dimensions, ~22 defauts distincts apres deduplication).

- **P1-A** — mutation EN PLACE des champs `pub` d'un `ArtifactRef` authentiquement minte :
  `delete` detruit un artefact PUBLIE et rend `Ok(())`. Trouve INDEPENDAMMENT par 4 dimensions
  (doc-claims, threat-model, seam-security, r2-regressions), 0 refutation sur 3 jures.
  Deux documents de securite affirment le contraire.
- **P1-B** — « The sealing key lives in the host vault » : aucune custody n'existe, et le module
  frere du MEME commit ecrit « Nothing here stores it ». 0 refutation sur 3 jures.

4 findings initialement cotes P1 ont ete **redescendus a P2** par majorite (2/3) : GUIDE absolu
non borne, borne « chemin du vault » absente des 3 documents de synthese, `delete` acceptant une
capability etrangere, `reference_serialisation_carries_no_key` non discriminant. 3 findings sont
tombes en refutation. Le coeur crypto lui-meme (AES-256-GCM, cle+nonce frais, AAD liee au mode
avec vecteur dore, fail-closed sur troncature/falsification/mauvaise cle/mauvais label,
`Zeroizing` epingle par annotation des deux cotes) **tient** et n'a produit aucun finding.

---

## Perimetre lu

13 entrees stagees (`git status --porcelain`), lues sur les **blobs de l'index**
(`git show :<path>`), jamais sur l'arbre :

- Code neuf : `crates/sbfb-workflow-engine/src/artifact_acl.rs` (1421 l),
  `crates/nexus-core-rs/src/artifact_seal.rs` (402 l),
  `crates/nexus-shell-daemon/src/artifact_sealer.rs` (229 l)
- Code modifie : `sbfb-workflow-engine/src/{lib.rs,secret.rs}`, `nexus-core-rs/src/lib.rs`,
  `nexus-shell-daemon/src/main.rs`
- Docs : `docs/security/THREAT_MODEL.md` (§2 A16, §19.5, changelog v29),
  `docs/factory/WORKFLOW_AGENTS_OVERVIEW.md`
- Planning : `sprint83_plan.md` (§Phase M/M1/Y, carte de phases), `sprint83_gate_matrix.md`
  (rows 25/26), rounds r1/r2

**9 dimensions** : suites §7.4 · mutation-acl (22 mutations) · mutation-seal (12 mutations) ·
doc-claims-acl · threat-model · plan-gate · seam-security · wire-serde · r2-regressions.
Puis **3 jures de refutation** en lentilles distinctes (la mesure tient-elle ? / est-ce dans le
scope ? / quelle est la consequence reelle ?). Verification independante de ma part sur les
ancres douteuses (voir « Findings tombes » pour les ancres corrigees).

---

## Findings retenus

| Sev | Titre | Ancre | Pourquoi (mesure) | Correctif propose |
|---|---|---|---|---|
| **P1** | Muter un champ `pub` d'une reference **authentiquement mintee** contourne la garde de provenance : `delete` de-tague un artefact PUBLIE et rend `Ok(())` pendant que le magasin sert toujours le clair | `artifact_acl.rs:282-290` (`pub digest`/`pub mode`/`pub retention`, seul `origin` prive) · `:296-299` (« lets `delete` tell an authentic reference from a forged one ») · `:707` (unique garde) · `:1290` (le seul test, n'emprunte QUE `from_untrusted_wire`) · `THREAT_MODEL.md:2370` (statut **fermee (provenance non falsifiable)**) · `:2379` (borne 2) · `:2764` (v29) | Sondes d'integration en worktree jetable, 4 dimensions independantes, resultats concordants. `let (mut r,_) = vault.publish(Published, Keep, X); r.mode = Local;` — `origin` survit (champ prive empeche la CONSTRUCTION, jamais l'assignation a un champ `pub` d'une valeur possedee), `is_locally_minted()` reste vrai, `delete` rend `Ok(())`, `blob_untagged=true`, `store_still_serves_plaintext=true`. **Consequence neuve (PROBE-B)** : `r.digest = victim.digest` sur une ref LOCALE mintee de-tague un blob sans rapport — `delete` fait `untag(&reference.digest)` (`:721`) apres n'avoir controle que `origin` — donc primitif de de-tag ARBITRAIRE. Meme famille sur `retention` (flip `Keep` sur une ref locale expiree ⇒ `Ok("PLAINTEXT")` la ou l'authentique rend `Err(Expired{2000,5000})`) : celle-la EST ecrite en borne 2, la destructrice ne l'est pas. C'est mot pour mot le scenario que `:2370` decrit a l'imparfait comme corrige. Le controle honnete voisin EST discriminant (ref `Published` authentique ⇒ `DeletionRefused`), donc la garde arrete une reference RECONSTRUITE et pas une reference EDITEE. | Structurel, cout mesure nul : `digest`/`mode`/`retention` **prives** + accesseurs `&` — dans tout le diff ils ne sont LUS hors module qu'a `artifact_sealer.rs:212` et `:215` ; `from_untrusted_wire` reste le seul constructeur public, donc muter un champ oblige a y repasser et perd l'origine. Idiome maison deja present : `DecidedRef` (`capability.rs:622-628`), `BoundSecret` (`secret.rs:151-153`). Variante : `RefOrigin::ThisVault { digest, mode, retention }` + egalite exigee dans `delete`. Test rouge-avant/vert-apres `delete_refuses_a_mode_mutated_in_place` (+ variante `digest`). Tant que ce n'est pas fait : `:2370` → **bornee**, `:2764` aligne, borne 2 nommant la mutation in-place, et reecrire `:296-299` / `:320-323`. |
| **P1** | « The sealing key lives in the host vault » : aucune custody n'existe, le module frere du MEME commit ecrit l'inverse | `artifact_acl.rs:88` (section « At-rest bounds ») · contredit par `artifact_seal.rs:16-17` · consequence sur `artifact_acl.rs:200-201` (`Keep` = « Kept until explicitly deleted ») · `THREAT_MODEL.md:2389` (borne 4 « At-rest = Argon2id(PIN) seul ») | `grep -rn "artifact" crates/nexus-core-rs/src/{vault.rs,keystore.rs}` ⇒ 3 hits, tous le mot anglais sans rapport (`vault.rs:20`, `:90`, `:1617`) — **aucune API de custody**. `grep -rn "seal_artifact\|artifact_seal::" crates/ --include=*.rs` hors le fichier primitif ⇒ un seul consommateur, `artifact_sealer.rs:29/:72`, qui **rend** la cle (`ReadCapability::new(sealed.key.to_vec())`, `:79`) sans rien persister. Le frere l'ecrit noir sur blanc : « minted at seal time from the OS CSPRNG and handed back to the caller as the read capability. **Nothing here stores it.** ». Deux consequences derivables : (1) la borne 4 de §19.5 decrit une protection qui ne protege AUCUNE cle d'artefact ; (2) `Keep` n'a aucun support de stockage — la capability meurt avec le processus, donc tout artefact `local` est de facto illisible apres redemarrage. Contradiction interne, meme commit, sur une affirmation de securite : classe R5 (14 des 19 P1 de la Phase L1). | Remplacer `:88` par la position DS-1 deja juste chez le voisin : « Rien ici ne detient la cle : `seal_artifact` la frappe et la rend a l'appelant comme capability ; sa custody — et donc toute borne at-rest la concernant — appartient au call-site que M ne livre pas ; l'enonce Argon2id(PIN)/`KEYSTORE-LAYER2-INERT` est ce que `nexus_core_rs::vault` garantit POUR SES PROPRES entrees, cite comme patron. » Ajouter a `ArtifactRetention::Keep` que la persistance de la capability n'existe pas encore ; aligner la borne 4 de §19.5. |
| **P2** | `delete` accepte **n'importe quelle** capability : la garantie de destruction est affirmee sans reserve, et le site ou r2 demandait la borne a ete SUPPRIME au lieu d'etre annote | `artifact_acl.rs:715-719` (la garde est `capability.is_none()`) · `:686` (« That destruction is the whole guarantee ») · `THREAT_MODEL.md:2368` (**fermee (vocabulaire et refus)**, « supprimer detruit la CLE : le chiffre devient definitivement illisible ») · bornes 1-6 `:2378-2400` (aucune ne nomme ce cas) | Sonde : `PROBE-C delete(local, foreign capability) = Ok(())` puis `still readable with the surviving real key = Ok("SECRET-PAYLOAD")`. La possession n'est jamais verifiee ; `ReadCapability::new` est `pub` (`:379`). Le round 2 avait cote ce point P2 en demandant nommement « nommer l'instance dans le doc de `Shredded` ET dans la borne 2 de §19.5 » : le correctif a **supprime** `DeletionOutcome` (le doc porteur), n'a ajoute aucune borne, et a ecrit a la place un absolu dans le threat model. Redescendu de P1 par majorite : le scenario n'a pas d'adversaire (la partie qui fournit la capability est celle que la garantie protege) ; la defaillance realiste est une mis-reliure de la Phase Y. | Root cause ~4 lignes, les deux seams sont deja detenus par le vault : avant `untag`, `let stored = self.blobs.get_unauthenticated(&reference.digest).ok_or(DigestMismatch)?; self.sealer.unseal(reference.mode, &stored, cap)?;` puis `drop`. Test `deleting_with_a_foreign_capability_is_refused`. Si refus PO : 7e borne a §19.5 (« `delete` verifie qu'une capability est FOURNIE, jamais qu'elle ouvre l'artefact »), qualifier `:686`, retrograder `:2368`. |
| **P2** | La borne « le long du chemin du vault » est ecrite QUATRE fois dans ce commit et absente des TROIS documents de synthese contre lesquels la gate ferme | `gate_matrix.md:60` (row 26) · `plan.md:1117` (§M.4) · `WORKFLOW_AGENTS_OVERVIEW.md:48` — contre `THREAT_MODEL.md:74` (A16), `:2366`, borne 6 `:2394`, `artifact_acl.rs:21-24` | Verifie verbatim : A16 dit « … n'entre jamais dans le magasin **LE LONG DU CHEMIN DU VAULT** (`ArtifactBlobs::put` reste publique — borne 6) » ; `:2366` « a travers ce type … **Borne (borne 6 ci-dessous)** » ; borne 6 « La porte du magasin n'est fermee qu'au niveau du VAULT, pas du crate » ; module doc « along the vault's publishing path, which is the only one this crate owns ». Les trois documents de synthese la suppriment : row 26 « connaitre le digest ne permet pas de lire un artefact `local` (y compris via `/blob-serve` et `BLOBS_ALPN`) » ; §M.4 « ACL prouvee (digest ≠ lecture, y compris par blob-serve) » ; GUIDE:48 « L'ACL est STRUCTURELLE et non une facade … — cf. THREAT §19.5 **DS-1** » (le renvoi pointe la borne 1, pas la borne 6). Aggravant mesure : `grep -rn "impl ArtifactBlobs" crates/` ⇒ **0 implementeur de production**, `ArtifactBlobs` est un `pub trait` re-exporte racine (`lib.rs`), et **aucun test ne touche `/blob-serve` ni `BLOBS_ALPN`** — le bras 1 de `digest_is_not_read_right` interroge `MemoryBlobs`, ce que §M.3:1094-1095 dit honnetement (« le modele de `/blob-serve` ») et que la row et §M.4 ne disent pas. Fermeture de l'INSTANCE (r2), pas de la CLASSE. | Reporter la borne aux trois surfaces avec les mots deja ecrits dans le meme commit. Row 26 et §M.4 : « … LE LONG DU CHEMIN DU VAULT (les deux chemins publics ne sont pas exerces : le bras 1 interroge le modele de magasin non authentifie — cf. THREAT §19.5 borne 6) ». GUIDE:48 : « — garantie portee par le chemin du vault, pas par le crate : `ArtifactBlobs::put` reste une couture hote publique », et renvoi « §19.5 bornes 1 **et 6** ». |
| **P2** | `reference_serialisation_carries_no_key` reste VERT alors que la reference serialisee porte la cle entiere : filtre hex-only, aucun controle positif | `artifact_acl.rs:1393` (test), assertion `:1410-1418` (`key_hex` en `{b:02x}`) · claims adossees `:249-250`, `:25` | Mutation en worktree : champ `pub key_material: Vec<u8>` ajoute a `ArtifactRef`, rempli depuis `capability.expose()` ⇒ **15/15 passed**, ce test compris. Sonde d'impression du meme run : `{"digest":"bd5b…","mode":"local","retention":{"kind":"keep"},"key_material":[1,1,…,1]}` — les 32 octets sont DANS le JSON. Cause : `serde_json` rend un `Vec<u8>` en tableau **decimal**, le filtre est en hexa, et le canari `!contains(CANARY)` ne voit pas une cle. Redescendu de P1 : la propriete est tenue STRUCTURELLEMENT (aucun champ de cle dans le type, et l'absence de `Serialize` sur `ReadCapability` est epinglee par un doctest `compile_fail,E0277` mesure discriminant) — la fuite exigerait qu'on ajoute deliberement du materiel de cle. Classe deja fermee deux fois a P1 dans ce depot. | Le patron existe DANS CE COMMIT : `secret.rs:385` `find_material` couvre raw / json decimal / debug decimal / JSON-echappe, avec un RED ARM `secret.rs:616-619`. Le passer `pub(crate)` et asserter `find_material(&json, capability.expose()).is_none()`, **plus un RED ARM** serialisant une struct qui porte la cle et attendant `Some("json decimal array")`. Sans le RED ARM le nouveau controle retombe dans la meme classe. |
| **P2** | Le contrat de `ArtifactSealer::seal` n'interdit NI par le type NI par ecrit qu'un implementeur hote rende le plaintext tel quel comme `stored`, et `publish` ne re-verifie pas | `artifact_acl.rs:479-493` (doc + signature, aucune obligation) · `:635` (`let digest = self.blobs.put(&stored);`) · claims `:22`, `:584-585` | Sonde : `IdentitySealer` STRICTEMENT conforme au trait branche sur `ArtifactVault::new`, puis `publish(Local, Keep, CANARY)` ⇒ `capability_minted=true`, `plaintext_in_unauthenticated_store=true`, `publish_returned_ok=true`, `locally_minted=true`. Le clair d'un artefact `Local` est servi par `get_unauthenticated` — le modele exact de `/blob-serve` — avec une reference authentique. `grep -niE "implementer\|must return\|must not\|obligation\|post-condition"` sur le blob ⇒ aucun hit sur `seal`. Contraste interne au crate : `secret.rs:260-264`, `SecretResolver::resolve` ECRIT son obligation fail-closed. Redescendu de P1 : il existe UN implementeur (`HostArtifactSealer`, AES-256-GCM reel, teste bout-en-bout), aucun autre n'est prevu, et le cas symetrique (`put` acceptant des octets arbitraires) est deja traite par le projet comme une **borne honnete** (borne 6) et non un bloqueur. | (1) ECRIRE l'obligation sur `seal` : « Implementer obligation, load-bearing for G-ARTIFACT-ACL: for a mode whose `requires_sealing()` is true, the returned bytes MUST NOT permit recovering `plaintext` without the returned capability. `publish` does not re-check. » (2) Borner `:22` et `:584-585`, borne 7 a §19.5 en miroir. (3) Fermeture structurelle preferee — le newtype `StoredBytes` **deja nomme a `:586-587`** pour la borne 6 ferme LES DEUX portes d'un geste : `seal(...) -> Result<(StoredBytes, ReadCapability), _>` avec constructeur atteignable seulement depuis le chemin de scellement, et `put(&self, stored: &StoredBytes)`. **Ne PAS** se contenter d'un `if stored == plaintext { Err }` : `plaintext ‖ padding` le franchit. |
| **P2** | Le seam `ArtifactBlobs` est **synchrone et infaillible** au-dessus d'un implementeur NOMME qui est async et faillible : les erreurs d'ecriture ne sont pas exprimables | `artifact_acl.rs:516-525` (`put -> ArtifactDigest`, `get_unauthenticated -> Option`, `untag`) · `:117-119` (nomme `BlobsClient` comme implementeur prevu) · `:409-466` (`ArtifactError` sans variante de defaillance du magasin) · `nexus-core-rs/src/blobs.rs:85`, `:103` | `grep -n "pub async fn" crates/nexus-core-rs/src/blobs.rs` ⇒ `add_bytes(...) -> Result<[u8;32]>`, `get_bytes(...) -> Result<Vec<u8>>` et 7 autres : les 10 methodes du client sont `async fn … -> Result`. Les 3 du seam sont sync, 2/3 sans canal d'erreur ; `grep -niE "async\|block\|fallib"` sur le blob ⇒ **0 hit**. **Refutation honnete de l'hypothese du mandat** : `get_unauthenticated -> Option` n'est PAS la classe `Path::exists()` de L1 — un `None` ressort en `DigestMismatch` (`:663-667`), fail-CLOSED. La forme fail-OPEN est cote ECRITURE : `put` infaillible ⇒ `publish` rend `Ok((ref, cap))` pour des octets que le magasin a pu refuser ; `untag` infaillible ⇒ `delete` rend `Ok(())`. Un implementeur en contrat n'a que deux issues : paniquer, ou rapporter un succes qui n'a pas eu lieu. | Gratuit maintenant, fige au premier cablage : `put -> Result<ArtifactDigest, ArtifactError>`, `get_unauthenticated -> Result<Option<Vec<u8>>, _>`, `untag -> Result<(), _>`, + variante `ArtifactError::BlobStoreRefused { reason }` (le doc de l'enum `:407-408` dit « Everything that can refuse an artefact operation »). Si le seam reste infaillible pour M : ECRIRE les trois faits — (i) une ecriture qui echoue doit paniquer plutot que rendre un digest fabrique ; (ii) `untag` est best-effort et le `Ok(())` de `delete` porte la destruction de la CLE ; (iii) la reliure devra bloquer sur un client `async` via `spawn_blocking`/canal, pas un `block_on` naif. Borne 7 a §19.5. |
| **P2** | L'argument d'exhaustivite par CARDINALITE de `ArtifactMode::ALL` n'est porte par rien — un 3e variant traverse TOUS les controles runtime, **MESURE 15/15 verts** | `artifact_acl.rs:149-155` (« Exhaustiveness is proved by CARDINALITY ») · `:894-897` · `:908` (`assert_eq!(ALL.len(), 2)`) · `:864` (« the table must cover every mode ») · filet hors-crate `artifact_sealer.rs:154-159` | MUTATION E en worktree : variant `Ephemeral` ajoute, les 3 `match` exhaustifs etendus (le compilateur l'exige), `ALL` (`:155`) et `POLICY` (`:747`) INTOUCHES ⇒ **15 tests run: 15 passed**. `assert_eq!(ArtifactMode::ALL.len(), 2)` porte sur un `[T; 2]` : verite de TYPE, structurellement irrougissable ; le dedup ne voit que les 2 elements listes ; six tests iterent `POLICY`/`ALL` et un 3e mode en sortirait. **Le filet suppose n'existe pas** : `mode_labels_are_distinct_and_stable` compare `ALL.len()` a `EXPECTED.len()` — 2 == 2 des deux cotes — donc il attrape « ajoute a `ALL` sans libelle » mais PAS « variant hors de `ALL` ». Net compilateur mesure : sans l'arm daemon, `cargo check -p nexus-shell-daemon` ⇒ `E0004 non-exhaustive patterns` a `artifact_sealer.rs:46` ; **une fois l'arm ecrit** (ce que le compilateur force), `cargo check --bins --tests` ⇒ exit 0. Le compilateur force a TRAITER, jamais a INSCRIRE. P2 n°5 du round 2 **non ferme** : la re-coupe 4→2 a reecrit la prose sans ajouter le controle, et l'ensemble de modes a deja change UNE FOIS dans cette phase. | Rendre l'ajout rouge DANS le test : un `match` exhaustif **ecrit dans le test** mappant chaque variant a un ordinal, puis `let mut covered = [false; ArtifactMode::ALL.len()]; for mode in ArtifactMode::ALL { covered[ordinal(mode)] = true; } assert!(covered.iter().all(\|c\| *c));` — un 3e variant recoit un slot, un slot hors borne fait paniquer. Idiome maison : destructuration sans `..` de `mutation_invalidates_approval`. Corriger en parallele `:149-152`, `:894-897` et le message `:864` (« every VARIANT »). Le tautologique `:908` peut rester mais ne doit plus etre presente comme le controle. **Correctif non mesure** — a rejouer sous MUTATION E (attendu : rouge ou non-compilation). |
| **P2** | Le cadrage at-rest `nonce ‖ ciphertext+tag` n'est epingle par RIEN — MESURE 30/30 verts sous inversion coherente | `artifact_seal.rs:103` (le contrat publie) · construction `:165-167` · lecture `:184` · hypothese du test `:246`, `:254-264` | Mutation h coherente sur les deux moities (`stored = ciphertext ‖ nonce` + `split_at(len - NONCE_LEN)`) ⇒ **30 tests run: 30 passed, 1152 skipped**, aucun rouge dans AUCUN des trois crates. C'est le diagnostic que la phase a ecrit elle-meme pour l'AAD (`:294-296` : « seal and open derive it from the same helper, so the round trip is blind by construction ») — elle a ferme la moitie AAD par `aad_matches_its_golden_vector` (verifie discriminant 3 fois : domaine retire, `SEP` 0x1f→0x1e, ordre inverse) et laisse la moitie CADRAGE ouverte, alors que le correctif r2 l'avait nommee. Aggravant : `each_seal_mints_a_distinct_key_and_nonce` lit `first.stored[..NONCE_LEN]` et asserte la fraicheur du nonce (`:254-259`) — sous la mutation il asserte la fraicheur d'octets de CHIFFRE en restant vert. `grep -n 'ciphertext+tag\|nonce' docs/security/THREAT_MODEL.md` ⇒ §19.5 ne publie que l'AAD : le contrat de cadrage ne vit que dans ce rustdoc. | ~6 lignes dans `each_seal_mints_a_distinct_key_and_nonce`, en **re-derivant** depuis les primitives au lieu de repasser par `open_artifact` (c'est la re-derivation independante qui rend le controle discriminant, comme le vecteur dore pour l'AAD) : `Aes256Gcm::new_from_slice(&first.key)` puis `decrypt(Nonce::from_slice(&first.stored[..NONCE_LEN]), Payload { msg: &first.stored[NONCE_LEN..], aad: &seal_aad("local") })`. 0 dependance (`Aes256Gcm`, `Nonce`, `Aead`, `KeyInit`, `Payload` importes `:40-41`). Rouge sous la mutation h par construction. Ajouter a `:103` que le cadrage est un contrat at-rest immuable, sur le gabarit de `:297-299`. |
| **P2** | Deux comptes mesures perimes dans `artifact_seal.rs` : la classe soulevee au round 2 a ete fermee dans `artifact_acl.rs` et NON balayee ici | `artifact_seal.rs:115` (« 9/9 tests passed ») et `:384` (« 10/10 green against exactly that ») | `git show :crates/nexus-core-rs/src/artifact_seal.rs \| grep -c '#\[test\]'` ⇒ **11**. Reproductions fideles : (a) `:115` — champ+accesseur+frappe en `Vec<u8>` nu, annotation `:277` retiree ⇒ **11 passed**, pas 9/9 ; (b) `:384` — assertion ramenee a `contains` + filtre hex-only, `Debug` deversant `{:?}` ⇒ **11 passed**, pas 10/10. Contraste qui etablit instance-vs-classe, dans le MEME commit : `:296` « 14/14 across both crates » est **EXACT** (10 hors le golden + 4 daemon, re-mesure) ; `artifact_sealer.rs:167-168` DATE la sienne (« Measured on the previous revision ») ; les jumeaux `artifact_acl.rs:362-364` et `:390-394` ont vu leur chiffre RETIRE. Seul `artifact_seal.rs` garde des nombres nus. Memoire `feedback_close_class_not_instance`. | Aligner sur le remede deja applique aux jumeaux : retirer les deux chiffres (`:115` → « every other test in this module stayed green » ; `:384` → « the whole module stayed green against exactly that ») OU les dater comme `artifact_sealer.rs:167`. **Ne PAS reecrire 11/11** : le prochain test re-perime le nombre, c'est la classe qu'il faut fermer. |
| **P2** | « This accessor makes the wrapper part of the compiled contract » : c'est l'ANNOTATION du call-site qui le fait — texte-source du P1 r2, corrige cote moteur et pas ici | `artifact_seal.rs:110-118` (doc `key_is_zeroizing`) · test annote `:277` · version CORRIGEE du jumeau `artifact_acl.rs:390-394` + test `:1386` | Le round 2 a mesure sur le jumeau qu'une mutation COHERENTE (champ + constructeur + **signature de l'accesseur**) compile et laisse la suite verte : `.len()` existe sur les deux types, l'accesseur seul ne tient rien. Le correctif r3 l'a enonce cote moteur (`:394` « calling this accessor and reading a length **proves nothing** ») et annote le test `:1386`. Cote `nexus-core-rs`, le test `:277` est bien annote mais la doc affirme toujours « **This accessor** makes the wrapper part of the compiled contract, which is **the only thing** that can hold a claim ». La propriete TIENT ici — seule l'explication designe le mauvais porteur, dans le fichier que le prochain jumeau recopiera : litteralement l'enchainement qui a produit le P1 du round 2. | Aligner `:116-118` sur le jumeau : « L'ANNOTATION de type au call-site (`key_is_held_in_a_zeroising_wrapper`, l. 277) est ce qui tient : une mutation coherente qui change AUSSI la signature de cet accesseur compile, donc l'accesseur seul ne prouve rien. » |
| **P2** | Vestiges du vocabulaire a quatre classes : une recette de mutation publiee est **injouable**, et l'invariant de la gate est enonce sous un nom supprime | `THREAT_MODEL.md:1975` · `artifact_acl.rs:7` · `secret.rs:628` | `:1975` (edite par ce commit) : « alimente en publiant le canari sous une **classe** qui DOIT sceller — mutation-testee, publier la meme charge en **classe `public`** fait rougir le canari » — or `ArtifactMode` n'a que `Published`/`Local` (`:136-160`), donc la recette nomme un variant **inexistant**. `artifact_acl.rs:7` enonce l'invariant de la gate : « knowing the digest of a **private** artefact does not permit reading it », ~50 lignes au-dessus de `:56-59` qui declare `private` supprime, et la row 26 dit `local` — le module qui EST le livrable nomme sa propre propriete sous un mode inexistant. `secret.rs:628` : « under the **class** that MUST seal it ». Les 10 autres occurrences de « class » du diff sont explicitement historiques et datees ; ces trois sont au **present**. | `THREAT_MODEL:1975` → « sous un MODE qui DOIT sceller … publier la meme charge en mode `published` » ; `artifact_acl.rs:7` → « of a **local** artefact » ; `secret.rs:628` → « under the mode that MUST seal it ». |
| **P2** | `from_untrusted_wire` documente une reconstruction « from a wire payload » qu'aucun crate externe ne peut effectuer | `artifact_acl.rs:312-313` (doc), constructeur `:333` · `:228` (`pub struct ArtifactDigest(String);`, champ prive) · unique constructeur `:232` `of_stored_bytes(&[u8])` · frontiere `:108-110` (TEST-ACTOR 1 « Another node (wire) ») | Test d'integration jetable exercant les six routes de decodage depuis le JSON reel ⇒ **6 erreurs de compilation**, une par route : E0599 pas de `from_str` ; E0277 `!FromStr` ; E0308 pas de `From<String>` ; E0423 champ prive ; E0277 `!DeserializeOwned` ; E0277 `ArtifactRef: !Deserialize`. Le seul chemin restant est `of_stored_bytes(&octets)` — impossible pour un artefact `local` dont les octets ne quittent jamais le noeud (`:176-180`). Le type documente « The reference that TRAVELS » (`:248-249`) ne peut etre reconstruit par aucune des trois sources que son unique constructeur public nomme. | (a) Livrer le maillon, 4-6 lignes : `ArtifactDigest::from_hex(&str) -> Result<Self, ArtifactError>` validant 64 hex minuscules — c'est exactement l'« invariant de constructeur » que `:252-254` invoque pour refuser `Deserialize` et qui n'est applique NULLE PART. (b) Sinon, remplacer `:312-313` par l'enonce honnete et **nommer la Phase P**. Note : la raison citee pour refuser `Deserialize` ne tient pas pour `ArtifactRef` lui-meme — son unique champ prive est `#[serde(skip)]` et `RefOrigin` derive `Default` sur `UntrustedWire`, donc un derive le reconstruirait dans l'etat SUR ; l'invariant reel manquant est sur `ArtifactDigest`. |
| **P2** | La forme serialisee d'`ArtifactRef` n'est epinglee par rien : un changement wire cassant passe la totalite du crate | `artifact_acl.rs:279` (derive `Serialize`) · `:197-198` (`ArtifactRetention` interne-taggee `tag = "kind"`) · `:134-135` (snake_case) · unique assertion de forme `:1419` | Forme mesuree : `{"digest":"…","mode":"local","retention":{"kind":"keep"}}`. Mutation : `#[serde(rename = "hash")]` sur `digest` + `tag="kind"` → `rename_all="camelCase"` ⇒ **168 tests run: 168 passed, 7 skipped** (crate ENTIER), forme derivee `{"hash":"…","retention":{"expiresAtMs":{"at_ms":42}}}`. Seule la chaine `"mode":"local"` est assertee : ni le nom de champ, ni la representation d'`ArtifactRetention`, ni `"mode":"published"`. `plan.md:1312` programme `ArtifactRef` sur le wire compute de la Phase P comme « additif 0-bump (AM-12) » — « 0 bump » n'est pas verifiable contre une forme que personne n'a ecrite. | 4 lignes dans le test qui construit deja les deux variantes : `assert_eq!(json, "…")` litteral pour `Local`/`Keep`, plus un vecteur `Published` et un `ExpiresAtMs`. Idiome maison : `crates/sbfb-workflow-engine/tests/orchestrator_goldens.rs`. |
| **P2** | Deux des 15 tests de la gate — et les deux refus qu'ils gardent — n'existent dans **aucune** section du plan, alors que §19.5 leur credite deux fermetures de menace | `artifact_acl.rs:1290` (`delete_refuses_a_reference_this_vault_did_not_mint`), `:1336` (`retention_on_a_published_artefact_is_refused`), `:289-290` (champ `origin`), `:439` (`UntrustedReference`), `:448` (`RetentionOnPublished`) vs `plan.md:1058-1090` (§M.2), `:1092-1110` (§M.3), `:1117-1120` (§M.4) | Diff bidirectionnel. Nommes→livres : les **9/9** noms de §M.3 existent sous leur nom exact. Livres→nommes : **21** tests livres ne sont nommes nulle part, dont ces deux, que §19.5 cite comme la fermeture de **T-ARTIFACT-FORGED-REF-DESTRUCTIVE** (`:2370`) et de la moitie « combinaison impossible » de **T-ARTIFACT-RETENTION-BYPASS** (`:2369`). Ils ajoutent un 4e champ (prive, `#[serde(skip)]`) et deux variantes d'erreur, alors que §M.2 decrit un `ArtifactRef` v1 sans provenance et §M.4 ne mentionne ni l'un ni l'autre refus. Miroir du P1 n°6 de r2 : la, la gate fermait sur un critere qu'aucun test ne satisfait ; ici, du code ferme des menaces qu'aucune ligne de plan ne prevoit. Aggravant croise : l'un des deux est celui dont le P1-A montre qu'il ne garde pas ce que §19.5 lui credite. | 3 lignes en §M.2 (« `ArtifactRef` porte une origine PRIVEE `RefOrigin` non serialisee : `delete` refuse une reference non mintee localement ; une retention sur un artefact publie est refusee a la publication »), les deux noms de tests en §M.3, les deux refus dans la condition binaire §M.4. |
| **P2** | Le scope cut `ArtifactOutput.v1` est declare mais sa destination ne le porte pas, et sa consequence n'est pas ecrite | `plan.md:1064-1069` (§M.2) vs `:1834-1838` (§Y.2) et `:1840-1844` (§Y.3) | `grep -n 'ArtifactOutput' plan.md` ⇒ **un seul hit**, `:1064` (la coupe elle-meme). §Y.2/§Y.3 ne parlent que de « resolution `ArtifactRef` sous ACL (Phase M) » et `handoff_artifactref_acl.rs` : carry **unilateral**, invisible a l'audit gate (README §11 : tout carry a owner, trigger, exit). Et la CONSEQUENCE que r2 exigeait nommement n'est pas ecrite : une sortie declaree dans le pack **SIGNE** ne porte pas son mode, donc la confidentialite est choisie a l'execution par l'appelant de `publish` et un auteur de pack ne peut pas la declarer. | La consequence en une phrase dans §M.2 + une puce miroir en §Y.2 (« relier `ArtifactOutput.v1` (graphe signe) a l'`ArtifactRef` produit a l'execution — carry de la Phase M »). |
| **P2** | La carte de phases du plan assigne toujours `G-PROVENANCE` a **M**, contre la row 25 et §Phase M1 | `plan.md:166` vs `:1136` (§Phase M1) et `gate_matrix.md:59` (row 25) | `sed -n '166p'` ⇒ « M  artefacts / ACL / provenance ── G-ARTIFACT-ACL,G-PROVENANCE ». Aucun noeud M1 dans la carte, alors que les deux autres pre-coupes AM-4 y figurent (`L1` l.163, `P1` l.178) et que §Phase M1 existe (l.1136, ajoutee par `0fdbca91`). La row 25 attribue `G-PROVENANCE` a « **M1** (re-coupe de M, 2026-07-28) ». L'audit gate S84 lit **deux proprietaires pour une meme gate** — et une Phase M qui parait n'avoir livre qu'une des deux gates qu'elle porte. Defaut herite de la commit de re-coupe, non ferme par les +84 l de ce diff. | Une ligne, gabarit exact des noeuds L1/P1 : « M  artefacts / ACL ── G-ARTIFACT-ACL » + « M1 canonicalisation signee + provenance (re-coupe de M) ── G-PROVENANCE ». |
| **P2** | Le changelog v29 annonce « 5 rows neuves » ; §19.5 en contient **6** — et la promesse « statuts recopies VERBATIM » est elle-meme fausse | `THREAT_MODEL.md:2748` vs table `:2366-2371` ; la 6e enumeree hors ordre a `:2764` | `grep -c '^\| \*\*T-ARTIFACT'` ⇒ **6**. `grep -n 'rows neuves'` ⇒ `2748: 5 rows neuves`, `2894: 6`, `2971: 6` — le compte est une convention tenue par les entrees precedentes. Sur-defaut releve par un jure : la meme entree promet « statuts recopies **VERBATIM** de la table », or T-ARTIFACT-RETENTION-BYPASS y est cote « *bornee (politique, pas revocation)* » contre « **fermee (combinaison impossible) / bornee (politique, pas revocation)** » dans la table — la moitie FERMEE est perdue dans le resume que v28 designe comme « ce qu'un auditeur grep en premier ». | « **6** rows neuves », re-copier le statut complet de RETENTION-BYPASS, et remonter T-ARTIFACT-FORGED-REF-DESTRUCTIVE dans l'ordre de la table. Recompter a la source (`grep -c '^\| \*\*T-ARTIFACT'`) plutot que recopier. |
| **P2** | Delta de tests annonce **+27 / 2456** : le reel MESURE est **+30 / 2459** | `sprint83_phase_m_review_r2.md:86`, `:90`, **`:98`** (l'ancre `:107` du finding d'origine est FAUSSE — re-greppee, le « = **27** » est a `:98`) | `cargo nextest run --workspace --locked` (run propre, exit 0) ⇒ `Summary [ 62.284s] **2459 tests run: 2459 passed, 7 skipped**` (reproduit independamment par un jure : `[55.716s] 2459 passed`, vert au 1er run). Decompte par module a la source sur les blobs de l'index : `artifact_acl` **15** + `artifact_seal` **11** + `artifact_sealer` **4** = **30** ; 2429 (baseline L1, memory) + 30 = **2459**, coherent a l'unite. Les correctifs du round 3 ont ajoute 3 tests que r2 ne pouvait pas connaitre : r2 est une archive **exacte a sa date**, et la row 26 du gate (15+11+4) est deja exacte. Ce qui reste actionnable est la redaction du body. | Le body du commit doit porter **+30 (2429 → 2459)**, jamais +27/2456, et la baseline valide est **2429** (memory), jamais le 2167 perime de CLAUDE.md. Optionnel : une ligne « SUPERSEDED par r3 : 15+11+4 = 30, total 2459 » en tete de la section Verification de r2. |
| **P3** | « Swapping `Zeroizing<Vec<u8>>` for a bare `Vec<u8>` compiles » est au PRESENT et l'annotation deux paragraphes plus bas le rend faux | `artifact_seal.rs:112` contre `:277` | MESURE (mutation g coherente, annotation `:277` CONSERVEE) : `cargo nextest run -p nexus-core-rs -E 'test(artifact_seal)'` echoue a la compilation, `E0308 expected &Zeroizing<Vec<u8>>, found &Vec<u8>` — la propriete est reellement portee par le compilateur, le correctif r2 tient. Attenuant verifie : `:112-113` ecrit « every **other** test in this module stays green », donc l'enonce se scope deja hors du test annote. Risque residuel : un implementeur presse conclut que le controle est encore ouvert. | Deux mots : « … **WOULD** compile … were it not for the annotated binding in `key_is_held_in_a_zeroising_wrapper` », ou reprendre la formulation d'`artifact_acl.rs:392-394`, deja dans ce commit. |
| **P3** | La these de §19.5 enonce l'absolu sans qualificatif, la borne 6 arrivant ~60 lignes plus bas sans renvoi | `THREAT_MODEL.md:2335-2336` (« Il est tenu **STRUCTURELLEMENT** : pour le mode `local`, le CLAIR n'entre jamais dans le magasin ») ; borne 6 verifiee a **`:2394`** (les ancres `:2396-2398` des dimensions sont decalees de 2) | Lecture du blob. La row cardinale `:2366` renvoie explicitement (« Borne (borne 6 ci-dessous) ») et A16 `:74` porte le qualificatif en ligne ; la these de section est le seul des quatre enonces du THREAT_MODEL a rester nue. P3 et non P2 : la qualification vit dans la meme section auto-portante. | « … le CLAIR n'entre jamais dans le magasin **le long du chemin du vault (borne 6)** ». Aligne les quatre sites sur une seule formulation. |
| **P3** | §M.4 ferme sur `add_bytes`, une fonction que la phase n'atteint jamais ; la row 26 ferme sur `put` | `plan.md:1118` (§M.4) et `:1053` (§M.1) vs `gate_matrix.md:60` | `add_bytes` appartient a `nexus_core_rs::blobs::BlobsClient` (`blobs.rs:85`) ; la couture livree est `ArtifactBlobs::put` (`artifact_acl.rs:516`), sans implementeur de production. Le document de GATE (row 26) est deja correct et §M.1 nomme `add_bytes` comme enonce de la **decision PO**, pas comme critere. Imprecision de vocabulaire dans un critere d'acceptation. | §M.4 → « chiffrement obligatoire avant `ArtifactBlobs::put` (la couture ; `add_bytes` est la primitive hote visee, non atteinte en M — 0 implementeur) ». §M.1 peut garder `add_bytes`. |
| **P3** | La row 26 cite `sprint83_phase_m_review_r3`, qui n'existe pas au moment de la lecture | `gate_matrix.md:60` ; `.planning/active/rounds/` | `ls .planning/active/rounds/ \| grep phase_m` ⇒ `_r1.md`, `_r2.md` seulement ; `git status --porcelain` (13 entrees) sans `_r3.md`. Reference en avant, benigne SI r3 est stage au commit. Precedent verifie dans le meme repertoire : la Phase L1 est allee jusqu'a `_r10.md` — la citation figee `r1/r2/r3` rancira des qu'un round 4 aura lieu. | Verifier au `git commit` que `.planning/active/rounds/sprint83_phase_m_review_r3.md` est dans l'index. Formulation qui ne rancit pas : « cf. les rounds de review de la Phase M sous `.planning/active/rounds/` ». |
| **P3** | Le preflight garde M-d/M-e tels qu'ecrits (`read_audience`, quatre classes) sans marque de supersession | `sprint83_phase_m_preflight.md:416-417` ; encadre de re-coupe `plan.md:1022-1029` | `grep -rn 'read_audience' .planning/active/*.md` ⇒ 3 hits : `plan.md:1073` et `:1097` (explicitement historiques, « retire » / « SUPPRIME du plan ») et le preflight `:417`, qui exige encore « Classes public/prive/secret/reseau + `read_audience` » et nomme le test `artifact_acl::tests::read_audience_enforced` — un test qui ne doit pas exister. L'encadre cite le preflight en bloc sans dire quels items sont amendes. Un preflight est un artefact **date** qu'on ne reecrit pas : ce qui manque est le renvoi. | Une incise dans l'encadre : « les items M-d/M-e du preflight sont amendes par les deux arbitrages PO du 2026-07-28 (retrait de `read_audience`, quatre classes → deux modes) ; M-h et M-a/M-b/M-c/M-g partent en M1 ». |
| **P3** | Nouvelle racine de domaine crypto versionnee invisible au census DOMAIN, sans la ligne de mise au point que son propre precedent de sprint ecrit | `artifact_seal.rs:60-62` (`ARTIFACT_SEAL_AAD_DOMAIN = b"sbfb.artifact.seal.v1"`) ; regex du census `scripts/check-frontier-contracts.sh:259-260` ; precedent `hitl_approval.rs:29-30` | Le census ne matche que `const DOMAIN_[A-Z0-9_]+_V[0-9]+` : la constante lui echappe **par son nom** seul, alors que sa prose compte `DOMAIN_KEYSTORE_V1`, lui-meme un separateur AEAD (`keystore.rs:147-151`). `hitl_approval.rs:27-30` (Phase L1, meme crate, meme sprint, meme suffixe `-v1`) porte verbatim « NOT a `DOMAIN_*_V1` signing family » ; `artifact_seal.rs:60-62` n'en porte aucune. Les deux claims formulees (« census stays frozen at 29 », « 0 famille `DOMAIN_*` neuve ») sont **mecaniquement VRAIES** (gate relance : exit 0, `DOMAIN census [29 frozen]`) et la valeur est epinglee octet par octet par `aad_matches_its_golden_vector`. | Une ligne de doc a `:60`, patron exact de `hitl_approval.rs:29`. **Ne PAS renommer** en `DOMAIN_ARTIFACT_SEAL_V1` sans decision D8 : le census passerait a 30 et contredirait les deux claims. |
| **P3** | §20 item 3 : trois des quatre jambes (§4 DFD, §5.x STRIDE, §6 LINDDUN) restent non honorees — dette de sprint, pas defaut de M | `THREAT_MODEL.md:2410-2411` ; §2 A16 `:74` ; §6 LINDDUN `:294-308` | `sed -n '94,333p' \| grep -c "A1[3-6]"` ⇒ **0** : aucun des quatre actifs de l'arc Workflow n'a de ligne en §4/§5.x/§6. La pratique etablie du sprint (une §19.x dediee portant sa propre table STRIDE + la row §2) est honoree **integralement** par M. Nuance qui justifie de le tracer : A16 est le premier des quatre dont la confidentialite **Haute** est justifiee par la donnee personnelle, et §6 porte deja une row pour l'autre actif PII (« Projection accessibilite host→app (A9) », `:307`) — le declencheur LINDDUN existe ici et pas sur A13-A15. | Ne rien bloquer sur M. Carry nomme : owner = phase de cloture docs-contrat AJ1 ; trigger = audit gate S84 rejouant §20 ; exit = une row §6 LINDDUN « publication d'artefact » (Disclosure/Linkability) ou une decision ecrite que les §19.x tiennent lieu des trois jambes pour l'arc Workflow. |
| **P3** | Un test iroh-networked **hors perimetre** a echoue au 1er run complet (flake env, non reproductible) | `crates/nexus-shell-daemon/src/dispatch_loop.rs:361` — `boot_path_reenters_sync_set_and_delivers_after_reopen` | 1er run : `FAIL [21.700s] (1267/2459)`, panic `a gossip neighbor must form within the deadline: Elapsed(())`, exit 100, fail-fast. Isole ⇒ `1 test run: 1 passed`. Run suivant ⇒ `2459 passed`. Un jure a obtenu **vert au premier run**. Fichier NON touche : `git diff --cached --stat -- .../dispatch_loop.rs` vide, dernier commit `ce99c974` (Phase A1). Classe deja documentee CLAUDE.md (« tests iroh-networked env-INSTABLES »), ici sur Windows natif. | Pas de correctif de code. Mentionner en « Verification §7.4 » du body que la suite est verte (2459/2459) et qu'un flake reseau hors perimetre a ete observe, avec la commande d'isolement. Si la classe se repete : carry `SEEDER/GOSSIP-DEADLINE` dans `sprint84_audit_plan.md`. |
| **P3** | **Process — hygiene de worktrees** : 7 worktrees d'une session ANTERIEURE non nettoyes | `git worktree list` ⇒ `…/bf77b335-…/scratchpad/{wt-d2crypto,wt-d4,wt-dim1-logic,wt-dim5,wt-dim6,wt-dim7,wt-mrev3}`, tous sur `de3e9570` | Verifie moi-meme. **Aucun** worktree de CE round ne survit (mut-acl, mut-seal, aclprobe, wire3, regress, seam, plangate, tm3 ont tous ete retires par leurs dimensions) — donc **pas de P1 de process** : aucun artefact de mutation ne touche l'arbre principal (`git diff` VIDE, 13 entrees stagees identiques). Les 7 restants viennent de la session `bf77b335` sur un commit anterieur. | `git worktree prune` (ou `remove --force` ×7) avant le commit, sinon `.git/worktrees` accumule des HEAD detaches a chaque round. |

---

## Ce qui tient (garanties CONFIRMEES par mesure)

**Suites §7.4 — toutes vertes**, sorties exactes : `cargo fmt --all --check` exit 0 ·
`cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0, 0 warning sur 9 crates ·
`cargo nextest run --workspace --locked` ⇒ **2459 tests run: 2459 passed, 7 skipped** ·
`cargo test --workspace --locked --doc` exit 0, 3 doctests dont **`artifact_acl::ReadCapability`
(l. 366) NEUF, `compile fail … ok`** — forme correcte du correctif au P1 r2 (propriete portee par
le compilateur, plus par un `assert_eq!(.len(), 32)` que `Vec<u8>` satisfaisait aussi) ·
`cargo build -p nexus-shell-daemon --release` exit 0 ·
`bash scripts/check-frontier-contracts.sh` exit 0, **DOMAIN census [29 frozen]**, bridge-parity 16,
hitl-parity 15 · **Gate row 26** `cargo nextest run -p sbfb-workflow-engine artifact_acl --locked`
⇒ **15 passed, 160 skipped**, exit 0 ; l'« avant » cite en §M.4 est reproduit sur `HEAD` pristine
(`0 tests run` + `error: no tests to run`, exit 4). Les 7 `skipped` sont le socle habituel
(6 feature-gated + le `#[ignore]` S83-E). 0 dep, 0 bump wire (aucun manifeste dans les 13 chemins ;
`git diff --cached -U0 | grep '_VERSION'` ne rend que de la prose).

**mutation-acl — 22 mutations, 21 DISCRIMINANTES.** `publish` court-circuitant le scellement ⇒
4 rouges ; copie en clair `put` en plus du scelle ⇒ `digest_is_not_read_right` (via l'assertion
`public_surface()`, donc non redondante) ; les trois predicats mentis a UN SEUL bras rougissent
tous, table littérale rouge dans les trois cas ⇒ **le P1 n°2 du round 1 est CLOS et verifie
discriminant** ; ligne `Local` de `POLICY` dupliquee ⇒ rouge par l'assertion QUI NOMME la
propriete (dedup `:871-878`) ⇒ **le P2 n°6 de r2 est CLOS** ; `from_untrusted_wire` frappant
`ThisVault` ⇒ `delete_refuses_a_reference_this_vault_did_not_mint` (la garde de provenance EST
reelle sur son chemin) ; `authorise_egress`/`authorise_delete` toujours `Ok` ⇒ 1 rouge chacun ;
sealer double qui MENT ⇒ 4 rouges (les assertions observent le MAGASIN) ; borne de retention
`>=`→`>` ⇒ rouge (les deux bords 1999/2000 epingles) ; `rename_all` retire d'`ArtifactMode` ⇒
rouge. Mutation `Zeroizing`→`Vec<u8>` coherente ⇒ **NE COMPILE PLUS** (`E0308` a `:1384`) : le
correctif r2 est lui-meme discriminant. La claim d'auto-non-discriminance du doc `:1236-1242`
se reproduit **au test pres** (`unwrap_or_default` ⇒ 15/15 ; les deux gardes retirees ⇒ 2 rouges
exactement nommes).

**mutation-seal — les six angles imposes par le mandat sont TOUS fermes.** (a) AAD : domaine
retire / `SEP` 0x1f→0x1e / ordre inverse ⇒ **1 rouge, meme test** dans les trois cas — le P2 r2
est CLOSED et l'egalite litterale attrape les trois classes. (b) `Debug` deversant la cle en
decimal ⇒ `debug_redacts_key_material` rouge — **P1 r2 CLOSED**. (c) Nonce fige ⇒ 1 rouge ; cle
figee ⇒ 2 rouges : les deux moities de la fraicheur sont gardees separement. (d) Label constant
⇒ 2 rouges ; labels **swappes** (bijection conservee) ⇒ 1 rouge grace a la table litterale
`EXPECTED`. (e) Implementeur hote rendant le plaintext ⇒ 2 rouges, dont le bout-en-bout sous
crypto reelle. (f) Mode code en dur au site d'unseal (moteur) ⇒ 1 rouge ; mode ignore dans le
pont ⇒ 1 rouge — **le P2 r2 sur le bras unseal (18/18 verts) est CLOSED**.

**Fail-closed et surface publique.** 0 `unwrap()`/`panic!`/`expect(`/`unsafe`/`todo!` en code de
production des 3 fichiers neufs. `open_artifact` garde la longueur de cle **puis** `stored.len()
< NONCE_LEN` AVANT `split_at` : troncature (boucle `0..NONCE_LEN+17`, qui depasse volontairement
la garde locale pour couvrir celle de `aead`), falsification (nonce/corps/tag nommes), mauvaise
cle, mauvaise longueur, re-etiquetage ⇒ tous une erreur, aucun `Ok`. `ArtifactSealError` est
deliberement grossier (pas d'oracle bad-key/bad-tag). Les 10 types re-exportes racine ont ete
passes en revue : aucune route de contournement neuve au-dela de celles **deja ecrites**.
`ReadCapability::new` n'a qu'UN producteur de production, sans `Serialize` (doctest
`compile_fail,E0277`), `Debug` redacteur epingle en **egalite exacte** des deux cotes, wrapper
`Zeroizing` epingle par **annotation de type** des deux cotes, `unseal` rend `Zeroizing` de bout
en bout. `#[serde(skip)]` sur `origin` est epingle par le COMPILATEUR (mutation retirant
l'attribut ⇒ `E0277 RefOrigin: !Serialize`).

**Claims verifiees exactes** : « no `rand`, no `getrandom`, no AEAD crate » (`cargo tree` ⇒ 0) ·
« the ONLY crate that depends on `sbfb-workflow-engine` » ⇒ `nexus-shell-daemon` seul · DS-1 reel
(0 hit hors les deux fichiers) · l'AAD publie a §19.5 correspond **octet pour octet** au code ·
`P_MAX = 2^36 - 31` = valeur RFC 5116 · les citations `blobs.rs:130-133`, `http.rs:272`,
`node.rs:442`, `graph.rs:599`, `vault.rs:22-31` sont exactes au mot et a la ligne — **zero ancre
inventee dans le plan** · les **9/9** tests nommes en §M.3 existent · les **6/6** tests cites par
§19.5 existent · les comptes de la row 26 (15/11/4) sont exacts, recomptes a la source · le
filtre positionnel de la row 26 est vrai, **mesure par mutation** (un `tests/artifact_acl.rs`
ajoute ⇒ `15 passed, 161 skipped` : le +1 est SKIPPE).

**Correctifs r1/r2 verifies solides** : R1-P1-1 (classe traversant la couture) tenu et renforce ·
R1-P1-2 (table derivee) tenu et renforce · R1-P1-4 (TEST-ACTOR) tenu · R1-P1-5 (§19.5 + canari
5→6 surfaces, `assert_eq!(surfaces.len(), 6)`) tenu · R2-P1-1 ferme par suppression propre
(`authorise_render`/`may_be_rendered` disparus avec la taxonomie) · R2-P1-2 (« no second door »)
tenu, borne 6 existante · R2-P1-4 (egalite exacte) tenu · R2-P1-6 (row 26) tenu, `read_audience`
n'apparait plus dans aucun critere de gate. §20 mecanique : les deux manques de r2 sont **FERMES**
(A16 versee en §2 `:74`, entree v29 presente au gabarit de v28, numerotation sans trou).
Effets de la re-coupe 4→2 juges **sains** : le libelle `"published"` epingle sans chemin de
production est du future-proofing **declare** (`artifact_sealer.rs:25-27`), pas une garantie
sur-vendue.

---

## Findings tombes en refutation

1. **Baseline de tests de CLAUDE.md perimee (2167 vs 2459)** — DROP 2/3 : hors des 13 chemins de
   la phase, et le finding conclut lui-meme « rien a faire en M » ; la reconciliation est la
   **Condition 3 explicite du wrap-up S83** (`plan.md:2504`). La consigne operante survit dans le
   P2 « delta +30 » : la seule baseline valide est **2429**, jamais 2167.
2. **« The plaintext never enters the store, so it never leaves the node » serait un non-sequitur**
   (`artifact_acl.rs:141-142`) — DROP 2/3 : l'antecedent naturel de « it » est *the plaintext*, et
   sous cette lecture la phrase est litteralement VRAIE. Les deux etais du finding sont **deja
   ecrits** dans le meme fichier (`:137-139` « serves any stored blob to any dialer » ; `:560-564`
   « a door the wiring must walk through, not a barrier standing in a path ») et la classe etait
   deja tombee en refutation au round 2 sur preuve. Residu non cote : la cellule « leaves the node
   \| no » de `:67` n'ecrit pas que le CHIFFRE, lui, est servi a tout dialeur.
3. **« Trois mesures de discriminance non reproductibles dans `artifact_seal.rs` »** — DROP 2/3 :
   deux de ses trois jambes sont fausses. `:296` « 14/14 across both crates » est **EXACT**
   (11 − 1 golden = 10 + 4 daemon) et la phrase est explicitement conditionnelle (« **Without it**
   the AAD is pinned by nothing … measured ») ; `:384` decrit deja l'assertion AFFAIBLIE remplacee.
   Seule la jambe « 9/9 » survit — c'est le P2 canonique ci-dessus.

**Redescendus de P1 a P2 par majorite (2/3)** : GUIDE:48 non borne (fusionne dans le P2 « borne
chemin du vault », superset) · `delete` acceptant une capability etrangere (pas d'adversaire :
la partie qui fournit la capability est celle que la garantie protege) ·
`reference_serialisation_carries_no_key` (propriete tenue structurellement par le type + doctest
`compile_fail`) · obligation d'implementeur de `seal` (un seul implementeur, reel et teste ; le
cas symetrique `put` est deja traite comme une borne honnete). **Redescendus de P2 a P3** :
flake `dispatch_loop`, §M.4 `add_bytes`, tournure au present de `artifact_seal.rs:112`.

**Deduplication** : 40 findings bruts ⇒ ~22 defauts distincts. 8 groupes de doublons
inter-dimensions, dont le P1-A trouve **quatre fois independamment** (#8 doc-claims, #14
threat-model, #28 seam-security, #34 r2-regressions) — le meilleur signal de fiabilite du dossier.
**Corriger UNE fois par defaut, pas une fois par index.** P1-A et le P2 `delete`/capability se
ferment par UN SEUL geste structurel (champs prives + accesseurs, plus `unseal` avant `untag`).

**Ancres corrigees par moi avant redaction** (regle d'honnetete) : le « = 27 » de r2 est a
**`:98`**, pas `:107` ; la borne 6 de §19.5 commence a **`:2394`**, pas `:2396-2398` ni
`:2394-2396`. Aucun finding n'a ete retire de ce fait, les mesures restant valides.

---

## Etat de l'arbre

**PROPRE — verifie par moi en fin de round.**
`git -C C:/Users/FlowUP/Documents/Code/nexus status --porcelain` rend **exactement les 13 entrees
stagees de la phase**, inchangees : 5 `A ` (rounds r1/r2, `artifact_seal.rs`, `artifact_sealer.rs`,
`artifact_acl.rs`) et 8 `M ` (`sprint83_gate_matrix.md`, `sprint83_plan.md`,
`nexus-core-rs/src/lib.rs`, `nexus-shell-daemon/src/main.rs`,
`sbfb-workflow-engine/src/{lib.rs,secret.rs}`, `WORKFLOW_AGENTS_OVERVIEW.md`, `THREAT_MODEL.md`).
`git diff` est **VIDE** (`--stat` sans sortie). Ce rapport (`r3.md`) est le seul ajout, untracked
jusqu'a son `git add`.

**Aucun artefact de mutation ne survit dans l'arbre principal.** Les 9 dimensions ont travaille en
worktrees jetables (recette a 3 lignes, avec le controle `status --porcelain` du worktree prouvant
que le diff de phase y etait applique — sans quoi les mesures auraient ete vacueuses) et les ont
tous retires : `mut-acl`, `mut-seal`, `aclprobe`, `wire3`, `regress`, `seam`, `plangate`, `tm3`
sont absents de `git worktree list`. Aucun code de production n'a ete modifie ; **aucun correctif
n'a ete applique** — ils sont decrits, pas joues.

**Une reserve d'hygiene, hors de ce round** (cotee P3 ci-dessus, pas P1) : `git worktree list`
montre 7 worktrees d'une session anterieure (`bf77b335`), tous epingles sur `de3e9570` —
`wt-d2crypto`, `wt-d4`, `wt-dim1-logic`, `wt-dim5`, `wt-dim6`, `wt-dim7`, `wt-mrev3`. Ils ne
touchent pas l'arbre principal. A purger avant le commit. Les worktrees permanents du depot
(`.claude/worktrees/*`, `nexus-098-flip`) sont hors sujet.

---

## Ce qui reste pour Codex

Les angles que ce round n'a **pas** epuises, par ordre de valeur :

1. **La 6e surface du canari n'a ete rejouee par PERSONNE.** `secret.rs:621-690` ajoute
   `artifact_public_store` au sweep et `THREAT_MODEL:1975` affirme « **mutation-testee**, publier
   la meme charge fait rougir le canari en le nommant ». C'est la **seule affirmation de
   mutation-test du commit qui n'a ete ni reproduite ni refutee**. Aggravant : la surface est
   collectee via `String::from_utf8_lossy(&blobs.stored)` sur des octets XOR-es — conversion
   **lossy**, qui peut mangler des octets et rendre la recherche non discriminante. A jouer :
   publier sous `ArtifactMode::Published` dans ce bloc doit rendre le canari ROUGE **en le nommant**.
2. **`ArtifactRef` derive `PartialEq, Eq` en INCLUANT le champ prive `origin`** (`:279`, verifie).
   Consequence non examinee : deux references decrivant le MEME artefact comparent **inegales** si
   l'une vient de `publish` et l'autre de `from_untrusted_wire` — et comme `origin` est
   `#[serde(skip)]`, un aller-retour de serialisation change silencieusement le resultat d'un `==`.
   §Y.2 (`plan.md:1836`) prevoit explicitement une resolution `ArtifactRef` avec deduplication
   idempotente. A trancher : `PartialEq` doit-il ignorer `origin` ? Aujourd'hui c'est decide par
   accident du `derive`.
3. **La famille « mutation en place » n'a ete inventoriee que sur `delete`.** `read` (`:675`)
   choisit d'unsceller selon `reference.mode`, champ `pub` : forcer `mode = Published` sur un
   artefact `Local` rend le CHIFFRE en `Ok(...)` la ou l'appelant attend du clair — confusion de
   type de retour silencieuse. `authorise_read` et `authorise_egress` n'ont pas ete balayes.
   **Fermer la classe suppose de les enumerer toutes, pas de patcher `delete`**
   (memoire `feedback_close_class_not_instance`).
4. **Le doctest `compile_fail,E0277` n'a pas ete interroge sur sa raison d'echec.** `:366-372` est
   presente comme le SEUL garde de l'absence de `Serialize` sur `ReadCapability` : passe-t-il pour
   la BONNE raison ? (si `artifact_acl` n'etait pas `pub`, l'echec serait `E0603` et le
   `compile_fail` verdirait sur un `E0277` jamais atteint). `cargo test --doc` a ete relance et est
   vert, mais la modalite « bonne erreur » reste non mesuree.
5. **La discriminance du filtre positionnel de la row 26 n'a pas ete mutee dans l'autre sens** :
   un **renommage du module** `artifact_acl` fait-il bien echouer la commande de gate ? C'est le
   seul controle mecanique de la row. (Le sens « un fichier `tests/` ne matcherait pas » est,
   lui, mesure et vrai.)
6. **Les correctifs proposes ci-dessus n'ont ete mesures par personne** — en particulier le
   `match`-dans-le-test de l'exhaustivite (attendu : rouge ou non-compilation sous MUTATION E) et
   la re-derivation independante du cadrage `nonce ‖ ct` (attendu : rouge sous mutation h). Ils
   doivent etre rejoues **sous leur mutation** apres application, pas seulement compiles.
7. **Reserve d'honnetete a re-lire** : le critere de gate M.4 n'est pas discriminant pour le bras
   unseal — sous la mutation (f) cote moteur, `cargo nextest run -p sbfb-workflow-engine
   artifact_acl` rend **15/15 passed**, le seul rouge vivant dans `nexus-shell-daemon`. La
   propriete EST tenue par la §7.4 complete, qui est ce que le commit execute ; c'est un repere de
   lecture du gate, pas un defaut.
