# Review — Sprint 83 Phase M (artefacts, ACL et confidentialite) — round 6

## Verdict

**FAIL** — 1 P1 retenu.

Le critere d'arret du depot, « CLEAN ou P2/P3 documentes uniquement », **n'est PAS atteint**, d'un
cran et d'un seul. Ce qui manque, en toutes lettres : l'en-tete de module
`crates/sbfb-workflow-engine/src/artifact_acl.rs:74-79`, section litteralement intitulee
« What "deletion" means, honestly », affirme au PRESENT et sans aucune borne que le chiffre
« becomes permanently unreadable … but nobody can read them again, which is the guarantee that
matters », alors que la **borne 10** que le meme commit vient d'ecrire en §19.5 dit l'inverse et
pretend meme que « les deux sites du code la portent ». Une phrase fausse dans un commentaire de
securite se cote P1 (`prompts/agent/phase-review-workflow.md:103-105`) : l'implementeur de la
reliure (M1/P/Y) code d'apres elle. Le correctif est **de la prose, trois sites, zero code, zero
test** — apres quoi le critere est atteint et le passage a Codex est autorise.

Tout le reste part sous le critere : **12 P2 et 9 P3 documentes**, 0 refutation au fond,
0 ancre inventee sur 31 findings bruts.

**Trajectoire des P1 par round : 5 · 6 · 2 · 4 · 1 · 1.** Elle indique une convergence reelle mais
d'une nature precise : le CODE a converge (le geste de r5 n'a ouvert aucune regression, mesure ci-dessous),
et le residuel est depuis trois rounds la MEME classe de prose — « l'absolu du crypto-shredding » —
balayee a chaque fois sur N-1 surfaces (r3 P1-B, r4 « the fields are public » + borne 15, r5 borne de
copie fermee a 1 instance sur 4). Ce n'est pas un plateau de defauts nouveaux, c'est un plateau de
**methode** : `feedback_close_class_not_instance`. Le present round ferme la famille par un census
exhaustif (commande ci-dessous, a rejouer avant de refermer), ce qui rend le round 7 evitable.

---

## Perimetre lu

16 entrees stagees, lues sur les blobs de l'INDEX (`git show :<chemin>`) : 3 fichiers de code neufs
(`artifact_acl.rs` 20 tests, `artifact_seal.rs` 11, `artifact_sealer.rs` 4), 4 fichiers de code
modifies (`lib.rs` x2, `main.rs`, `secret.rs`), `THREAT_MODEL.md` (§19.5 + A16 + changelog v29),
`WORKFLOW_AGENTS_OVERVIEW.md`, `sprint83_plan.md` §M, `sprint83_gate_matrix.md` row 26, plus les
5 rapports de round r1-r5.

Cinq dimensions : regressions du correctif r5, doc-claims, threat/plan/gate, security-mutation,
suites-counts. **25 mutations distinctes** jouees en worktrees jetables, chacune controlee par
`grep -c` AVANT le run. Les neuf suites §7.4 lancees jusqu'au bout. Trois refuteurs ont re-greppe
chaque ancre et re-lance chaque commande re-lancable ; les mutations non re-jouables ont ete
refaites par derivation fermee sur le chemin d'appel.

Verifie par moi a la redaction : les 4 sites de la classe (census `git show :<f> | grep -niE
"nobody can read|permanently unreadable|no way to keep a copy|gone for good|plus personne ne peut
les lire|aucun moyen d'en garder"` sur les 7 blobs) ; les comptes 20/11/4 ; `grep -c "return
Err(DeleteRefusal"` = 4 ; `plan|gate grep -c authorise_delete` = 0/0 ; `gate_matrix:60` « module de
lib » present ; `THREAT:2066` « 6 rendus du moteur » contre `:1975` « les CINQ PREMIERES … la 6e …
est le MODELE ».

---

## Findings retenus

31 index bruts, **~22 defauts distincts** apres dedoublonnage (la classe « absolu de copie »
apparait 4 fois, le test non nomme 4 fois, l'ordre des refus 2 fois, la row 26 2 fois, le compteur
« 19/19 » 2 fois). Un seul geste ferme chaque cluster.

### P1 — le seul bloquant

**P1-1 — L'absolu « aucune copie possible » survit a l'en-tete de module, sans borne ni renvoi.**
Ancre : `crates/sbfb-workflow-engine/src/artifact_acl.rs:74-79`.
Mesure (re-greppee par moi) : `sed -n '60,110p'` sur le blob ⇒ **0 occurrence** de `borne`,
`copy`, `copie`, `expose`, `to_vec` dans toute la section — c'est le SEUL des sites de la classe
qui n'a AUCUN pointeur vers la borne 10, et la phrase se conclut par « which is the guarantee that
matters », donc elle VEND la garantie en absolu. `expose()` rend `&[u8]` (`:537`) et
`ReadCapability::new(key: Vec<u8>)` est `pub` (`:522`) : l'inference est fausse en logique et en
fait. Jamais signale en r1-r5 (`grep -rniE "nobody can read|permanently unreadable"
.planning/active/rounds/` ⇒ 0 hit) : ce n'est pas une reouverture, c'est le site que les trois
recensements concurrents ont manque parce que leur regex ne cherchait que `Clone` /
`no way to keep a copy`.
Scenario entrees→etat faux : l'implementeur de la reliure ouvre le module, lit la section
« honestly », en tire « une capability ne se copie pas », cable un rendu « donnees detruites »
inconditionnel apres `delete(Ok)` — soit `T-ARTIFACT-DELETION-THEATRE`, la row que cette phase
possede.
Correctif : `:78-79` → « … but nobody **other than a holder who copied the key out beforehand**
can read them again (cf. [`ReadCapability::expose`], **borne 10 de §19.5**) — which is the guarantee
that matters » ; retirer « permanently » de `:75` ou l'assortir de la meme incise.

**Deux sites residuels de la MEME classe partent dans le meme geste** (cotes P2 par le jury parce
qu'ils sont au PASSE ou precedes de leur borne dans la meme unite de lecture) :
`artifact_acl.rs:967` (« had no way to keep a copy » → « had no **SUPPORTED** way ») et
`THREAT_MODEL.md:2369` (« plus personne ne peut les lire » → « plus aucun **TIERS** » ; « aucun
moyen d'en garder copie » → « aucun moyen **SUPPORTE** »).

**Correction d'honnetete — un finding retire son 4e site.** Trois dimensions annoncaient
`sprint83_plan.md:1107` comme portant l'absolu. **C'est FAUX** : la ligne ecrit seulement
« `ReadCapability` n'etant pas `Clone` », un fait de type VRAI. Re-greppe par moi. Le geste porte
donc sur **TROIS sites de prose, pas quatre**. `artifact_acl.rs:1924` (« a consumed capability
would be gone for good », commentaire de test) est **defendable dans son scenario** : aucune copie
n'y est faite — ne pas le toucher.

### P2 (12) — a jouer avant le commit, aucun ne bloque

| # | Defaut | Ancre | Mesure | Correctif |
|---|---|---|---|---|
| P2-1 | Le bras **SEAL** du liage de mode n'est epingle par aucun des 35 tests | `artifact_seal.rs:180`, `artifact_sealer.rs:72` | **S10** (`seal_aad("local")` en dur) ⇒ 690 passed + 4 passed, **0 rouge** ; **S11** (miroir hote) ⇒ 4 passed ; la mutation SYMETRIQUE cote `unseal` (**S12**) ⇒ **1 FAIL**. Cause : aucun test ne scelle jamais sous `Published`. Claims `:163` et `THREAT:2367` (« les DEUX bras ») restent vraies du CODE | 12 lignes en fin d'`opening_under_another_mode_does_not_work` : sceller sous `Published`, asserter `unseal(Local)` = `SealerRefused` et `unseal(Published)` = canari. **Verifie discriminant : 4/4 propre, 1 FAIL sous S11** |
| P2-2 | L'**ORDRE** des refus (provenance avant mode) est affirme 3 fois et epingle par rien | `artifact_acl.rs:781`, `:788-791`, `:1643-1657` | **M3** (ordre inverse) ⇒ **20 passed, 0 failed**. Cause : les 3 references untrusted de la suite portent `Local`, que `deletable()` autorise ; le couple discriminant (untrusted + `Published`) n'existe nulle part | Un token : `ArtifactMode::Local` → `Published` au bras 3 (`:1649`). **Verifie : 20/20 propre, 1 FAIL sous M3** |
| P2-3 | Garde de longueur de cle d'`open_artifact` = doublon §P76, seul des 35 tests qu'aucune mutation a un site ne rougit | `artifact_seal.rs:204-206` contre `:212` | **S1** (retrait du bloc) ⇒ **11 passed**. `KeyInit::new_from_slice` rend le MEME variant `KeyLength`. §P76 (`PATTERNS.md:4448-4451`, commit `a67e1eb3`) n'admet la duplication que si chaque site est independamment epingle | 5 lignes dans `wrong_key_length_is_refused` : cle courte + **cadre tronque** ⇒ `KeyLength`. Epingle l'ORDRE et CONSERVE la profondeur. **Verifie : 11/11 propre, 1 FAIL sous S1** |
| P2-4 | La justification anti-oracle de `read` est refutee par l'ordre de ses propres enonces | `artifact_acl.rs:899-903` contre `:911` et `:924` | Aucune mutation necessaire : `digest_is_not_read_right:1293` (present ⇒ `MissingCapability`) et `reading_an_absent_digest_is_refused_like_a_mismatch:1582` (absent ⇒ `DigestMismatch`) sont la preuve cote a cote. La fusion reelle est **absent vs CORROMPU**, pas present vs absent | 2 phrases, zero code. Ne PAS toucher `ArtifactError::DigestMismatch:606-611`, qui reste VRAI |
| P2-5 | `addresses_same_artefact` sans aucun controle negatif | `artifact_acl.rs:477-479`, site unique `:2055` | **M17** (corps `true` inconditionnel) ⇒ **20 passed**. L'unique assertion porte sur deux references du MEME digest. Doc `:470-476` en fait le comparateur de la resolution idempotente Phase Y | 1 ligne : `assert!(!published.addresses_same_artefact(&<autre digest>))` |
| P2-6 | La decision documentee sur 11 lignes « `PartialEq` compte `origin` » reste sans garde — **reconduction r5 non appliquee** | `artifact_acl.rs:338-348`, assertion `:2052-2054` | **M21** (`PartialEq` manuel sans `origin`) ⇒ **20 passed**. `edited` (`Local`) et `published` (`Published`) different AUSSI par le mode | Le correctif deja redige ET deja mesure discriminant par r5 : une assertion identique en tout champ public, differente par la seule origine |
| P2-7 | « `delete` delegates its **WHOLE** refusal logic here » — faux a 1/4 | `artifact_acl.rs:781-783` (et `:1610-1612`) contre `:989/:993/:1003/:1009` | `grep -c "return Err(DeleteRefusal"` ⇒ **4**, un seul delegue. La version PRECISE est a `:981-982` du meme fichier. Pas P1 : la proposition principale (« the ONLY place either is checked ») est VRAIE — `is_locally_minted()` et `deletable()` ont 1 site de prod chacun | Un mot + une incise : « …delegates its whole **PROVENANCE-AND-MODE** refusal here. POSSESSION reste dans `delete` » |
| P2-8 | Le test neuf de r5 n'est nomme dans AUCUN document de planning | `sprint83_plan.md:1148-1158` (puce auto-declaree exhaustive) | Diff bidirectionnel a la source : **20 livres, 19 nommes**, seul `authorise_delete_refuses_provenance_and_mode` absent ; `grep -c authorise_delete` ⇒ **0** dans le plan ET dans le gate matrix (verifie par moi). Sens nomme→livre : **0 nom invente**. Ironie : l'ecart est cree par le correctif r5 qui avait ferme la meme classe | Ajouter le nom a la puce. Et refermer la puce en **recomptant** `grep -c '#[test]'`, jamais en recopiant |
| P2-9 | Borne 15 de §19.4 : la phase a transforme un compte EXACT en compte faux — **reconduction r5** | `THREAT_MODEL.md:2066` contre `:1975` (meme commit) | `HEAD:2065` = « 5 rendus du moteur » (exact) ; blob stage = « 6 », alors que `:1975` ecrit « les CINQ PREMIERES sont des rendus de types de PRODUCTION ; la 6e … est le MODELE ». `grep 'impl ArtifactBlobs for'` ⇒ 3, toutes `#[cfg(test)]` | Une ligne : « 5 rendus du moteur + le magasin d'artefacts (MODELE, cf. §19.5 bornes 1 et 6) + le blob au repos ». **Ne pas toucher** au 6 la ou il designe le sweep |
| P2-10 | Row 26 : « le filtre positionnel exige un **module de lib** » — falsifie par la 3e commande de la row elle-meme — **reconduction r5, correctif redige non applique** | `sprint83_gate_matrix.md:60` | **LANCEE** : `cargo nextest run -p nexus-shell-daemon artifact_sealer --locked` ⇒ 4 passed, exit 0, binary id `nexus-shell-daemon::bin/nexus-shell-daemon` ; `crates/nexus-shell-daemon/src/lib.rs` **n'existe pas**. La CONCLUSION de la row est vraie, c'est la clause de mecanisme qui est fausse | « il matche le CHEMIN DE MODULE — lib **ou binaire** ; un fichier `tests/*.rs` ne matcherait pas, son nom devenant l'ID du binaire ». **Ne PAS toucher `sprint83_plan.md:1069`** : cette occurrence porte sur `artifact_acl.rs`, dans un crate qui A une lib — elle est VRAIE |
| P2-11 | Le carry `DOCTEST-ERRCODE-NOT-ENFORCED` a une branche d'exit qui permet de le clore en laissant debout un enonce mesure faux — **reconduction r5** | `sprint83_plan.md:1108-1115` ; cible `vault.rs:195-197` | `vault.rs:195-197` (hors diff, dans l'arbre) affirme « the error code is fixed so the test cannot go green on an unrelated compilation error » ; `THREAT:1975`, ecrit par CE commit, publie la mesure inverse (rustc 1.95.0, editions 2021 ET 2024) | Supprimer la branche « ou decision ecrite » : « exit = jumeau pose sur `vault.rs` **ET** phrase corrigee ; une decision ecrite ne rend pas vrai un enonce mesure faux » |
| P2-12 | Le run workspace n'est vert qu'en `--no-fail-fast` / au re-run | binaire `nexus-shell-daemon` (classe timing/iroh-networked) | Run 1 `cargo nextest run --workspace --locked` ⇒ exit 100, 1 failed sur `dispatch_loop::tests::boot_path_reenters_sync_set_and_delivers_after_reopen` (22,2 s sous charge ; **5,8 s solo, PASS**). Run `--no-fail-fast` ⇒ **2464 passed, 7 skipped, exit 0**. Un refuteur a observe un membre DIFFERENT de la meme classe (`convergence_incremental_task_reaches_remote_replica`, 20,4 s) ; un autre n'a rien reproduit. Ni le test ni la classe ne sont du code de Phase M, mais M ajoute 4 tests a ce binaire | Aucun code. Le body decrit la **classe**, pas l'instance, et n'annonce pas un vert sec (formulation en §Delta ci-dessous). Router la stabilisation vers le carry CI existant |

### P3 (9) — documentes, partent tels quels

1. `artifact_acl.rs:329-330` attribue a `delete` un mecanisme que la delegation a deplace (`delete`
   ne consulte plus ni provenance ni mode, `:981-982` le dit) ; la CONCLUSION reste vraie.
   Deux membres de phrase. *(1 vote de refutation : l'enonce se lit comme un comportement, qui
   reste exact — retenu au benefice de la navigabilite.)*
2. « exactly one implementer » (`artifact_acl.rs:681`, §19.5 borne 7) : `git grep "impl
   ArtifactSealer for"` ⇒ **3**. Les phrases JUMELLES du meme commit (`:131`, borne 1) portent
   « de **production** ». Un mot par site.
3. Compteur nu « **19/19** » a `artifact_acl.rs:984` alors que le module en compte 20, dans le
   commit dont `artifact_seal.rs:431-433` ecrit « Test counts are deliberately not quoted anywhere
   in this module ». Situe (« An earlier revision »), donc pas faux. Aligner sur ses deux jumeaux
   sans chiffre (`:783-786`, `:1613-1616`).
4. En-tete « (pre-coupe AM-4) » `plan:1023` contre son propre encadre `:1025` et l'enumeration
   AM-4 de §R2 `:2559` (qui ne contient pas M mais compte M1 en AM-7). Patron de rechange : `I1`.
5. §M.4 (`plan:1172-1176`) lance **une** commande la ou la row 26 en nomme **trois** — donc
   `local_artefact_round_trips_end_to_end_under_real_crypto`, seul test sous crypto REELLE, n'est
   pas atteint. Deux lignes copiees depuis la row 26.
6. « deux `compile_fail` » (`plan:1167`) contre « les trois doctests du moteur » (`:1111`) :
   referentiels compatibles, mais la ligne Tests sous-liste le jumeau pose sur `secret.rs`.
7. Renvois nus « borne 25 » (`THREAT:2400`) et « borne 16 » (`:2407`) dans une liste §19.5
   numerotee **1..10 continues** ; leurs cibles sont en §19.4. Deux prefixes de section.
8. `sprint83_kickoff.md:639` decrit encore M par la doctrine REFUTEE (4 classes + audience), seule
   surface au PRESENT hors preflight. Artefact DATE hors diff : etendre la clause de supersession
   de `plan:1032-1039`, zero entree stagee neuve.
9. Bande d'estimation `plan:2474` `| L / L1 / M | +12..20 |` contre un reel de **+80** (L1 +45,
   M +35). Table etiquetee « ordre de grandeur », critere d'aucun gate. Une ligne de recalage.

### Trois reconductions qu'AUCUNE des cinq dimensions n'a re-detectees

Signalees par les refuteurs, meme famille que le P1 (un balayage qui atterrit a N-1) :

- **`artifact_acl.rs:521`** porte toujours « Wrap freshly-minted key material. **The host sealer is
  the only producer.** » juste avant `pub fn new(key: Vec<u8>)`, type re-exporte racine — alors que
  le meme fichier ecrit **trois** fois l'inverse (`:534`, `:937`, `:946`). C'etait le P2 n°5 de r5,
  et c'est la **premisse de la borne 10** que ce round vient d'ecrire : le laisser est incoherent
  avec le geste P1. **P2, a joindre au lot.**
- **`published_artefact_refuses_deletion` (`:1436-1458`)** passe toujours `none` : le 4e bras de
  `delete` (`DeletionRefused`) peut encore re-zeroiser une capability confiee sans qu'aucun test ne
  rougisse. r5 avait mesure la mutation et redige le correctif de 4 lignes. **4e instance d'une
  classe fermee a 3/4. P2.**
- **`artifact_sealer.rs:167` / `:169`** disent encore « the reference's **class** » / « ignores its
  **class** argument » pour un parametre nomme `mode`. P3 depuis r4. Deux mots. (`:54-57` emploient
  « class » legitimement : recit historique date.)

---

## Ce qui tient

**La de-duplication du round 5 est VALIDEE — c'est la question centrale du brief, et la reponse est
NON-bloquante.** Mesuree independamment par trois dimensions en worktrees jetables (baseline
`-p sbfb-workflow-engine` = 173 passed / 7 skipped) :

- **M1** — retrait de la garde unique de provenance ⇒ **3 FAIL**
  (`authorise_delete_refuses_provenance_and_mode`, `delete_refuses_a_reference_this_vault_did_not_mint`,
  `an_edited_reference_loses_its_provenance`). **Le chiffre annonce par r5 est exact.**
- **M2** — neutralisation de la garde de mode ⇒ **2 FAIL**. Chaque garde est donc visible
  **separement** : la propriete que la suppression du doublon devait obtenir est TENUE.
- **Semantique strictement inchangee** : `grep -c "return Err(DeleteRefusal"` = 4, memes quatre
  refus dans le meme ordre (provenance → mode → capability absente → blob absent → `unseal`),
  capability rendue sur les quatre bras, **0 `?` faillible** entre le move et la fin.
- **Effet de bord positif** : la fusion a **FERME le P2 n°4 de r5** — il n'existe plus de site de
  retour propre a `DeletionRefused`, il partage `:989` avec `UntrustedReference`, qui est asserte
  (**M4** ⇒ 1 FAIL). Le correctif r5 a resorbe un P2 en plus de celui qu'il visait.
- **`THREAT_MODEL.md:2371` est devenue VRAIE sans etre editee** : « retirer la garde fait rougir
  `delete_refuses_a_reference_this_vault_did_not_mint` » etait faux a la lettre tant qu'il y avait
  deux gardes (P2 de r5) ; M1 le confirme exact aujourd'hui.

**Census de discriminance : 34 des 35 tests rougissent sous une mutation a UN site** (25 mutations
distinctes). Seule exception : `wrong_key_length_is_refused` (P2-3). Detail : `artifact_acl` 20/20,
`artifact_seal` 10/11, `artifact_sealer` 4/4.

**Absolus falsifies par mutation, tous TENUS** : (1) `:489-494` « pinned HERE by a compile-fail
doctest PAIR, and nowhere else » — injection d'un `impl Serialize` qui serialise la cle ⇒ 173/173
runtime verts + doctest `compile_fail` **rouge**, vrai mot pour mot ; (2) `artifact_seal.rs:140-144`
« what stops it is the type ANNOTATION at the call site » — swap `Zeroizing<Vec<u8>>` → `Vec<u8>` ⇒
**E0308 exactement a la ligne d'annotation** `:323:44` ; (3) « the ONLY place either is checked » ⇒
1 site de prod chacun ; (4) `artifact_sealer.rs:6-7` « the ONLY crate that depends on
sbfb-workflow-engine » ⇒ vrai, inversion de couche impossible ; (5) `:77-79` la `Zeroizing`
originale est bien essuyee au drop de `sealed` ; (6) `ArtifactBlobs:739-743` fail-CLOSED sur les
deux chemins.

**Chemins d'erreur** : **0** `unwrap` / `expect` / `panic!` / `unreachable!` / `todo!` / `unsafe` /
cast tronquant / indexation / soustraction unsigned dans la moitie PRODUCTION des trois fichiers
neufs. Aucune panique atteignable. Chaque bras de la primitive crypto est fail-CLOSED.

**Surface publique** : 11 re-exports racine = exactement les 11 items publics d'`artifact_acl` ;
`RefOrigin` prive et sans constructeur public ; `publish` seul producteur de `ThisVault` ;
`DeleteRefusal` purement sortant (`grep "From<DeleteRefusal>"` ⇒ 0).

**Documents canoniques** : §19.5 = **bornes 1..10 continues, 0 doublon**, chacune verifiee vraie du
code stage ; borne 10 verifiee sur ses quatre sous-affirmations ; changelog v29 = 6 rows,
**6/6 statuts VERBATIM** compares mecaniquement, « Bornes portees de 6 a 10 » exact ; A16 `:74`
coherent ; census DOMAIN **29 frozen** ; carte de phases mesuree caractere par caractere (rail
colonne 66, anomalie `N` **byte-identique a HEAD**, anterieure au commit) ; `grep -cE '^## Phase '`
= 46, −1 = **45 phases**, conforme a §R2. **Aucune ancre inventee, aucune perimee** sur les 31
findings.

**`docs/factory/WORKFLOW_AGENTS_OVERVIEW.md`**, seule entree stagee que personne n'avait auditee,
est **JUSTE au mot pres** (2 traits nommes, `HostArtifactSealer` reel mais 0 call-site,
`ArtifactBlobs` sans implementeur de production, ACL portee par le CHEMIN DU VAULT, renvoi bornes
1 et 6). Aucun defaut.

**Suites §7.4, lancees jusqu'au bout depuis la racine** : `cargo fmt --all --check` exit 0 ·
`clippy --workspace --all-targets --locked -D warnings` exit 0 (0 warning) · `nextest run
--workspace --locked --no-fail-fast` **2464 passed, 7 skipped**, exit 0 · `cargo test --workspace
--locked --doc` exit 0 · `cargo build -p nexus-shell-daemon --release` exit 0 ·
`bash scripts/check-frontier-contracts.sh` exit 0 (anti-promise + frontier-tag 2 + DOMAIN 29 frozen
+ BLOB_SERVE_CSP + prompt-kind + bridge-parity 16 + pack-parity + hitl-parity 15).

---

## Findings tombes en refutation

**Aucun sur le fond.** 31 findings soumis, **0 ancre inventee, 0 mesure fausse, 0 refutation**.
Ce qui a bouge est la SEVERITE et le DEDOUBLONNAGE :

- **1 sous-ancre retiree** — `sprint83_plan.md:1107` presente par trois dimensions comme portant
  l'absolu « aucun moyen d'en garder copie ». **Re-greppee par moi : FAUX.** La ligne ecrit
  « `ReadCapability` n'etant pas `Clone` », un fait de type VRAI. Le geste P1 porte sur 3 sites de
  prose, pas 4.
- **1 site explicitement exclu du correctif** — `artifact_acl.rs:1923-1924` (« a consumed
  capability would be gone for good ») : commentaire de test, **defendable dans son scenario**,
  aucune copie n'y est faite. Ne pas le toucher.
- **1 finding refute par un juge sur trois** (P3 n°1, doc `ArtifactRef:329-330`) : l'enonce decrit
  un COMPORTEMENT qui reste exact apres delegation. Retenu P3 a la majorite, au benefice de la
  navigabilite.
- **12 doublons intra-cluster** fusionnes : la classe « absolu de copie » (4 rendus), le test non
  nomme au plan (4), l'ordre des refus (2), la row 26 (2), le compteur « 19/19 » (2). Chacun compte
  **une** fois ; la tete de cluster porte la severite.
- **1 escalade refusee** — la row 26 (« module de lib ») a deja ete cotee P2 par r4 puis r5 sur le
  motif que sa CONCLUSION est vraie et re-mesuree ; un round de cloture ne rejuge pas a la hausse un
  cran arbitre deux fois.
- **1 finding retrograde P2→P3 par deux juges sur trois** (le flake workspace) : ni le test ni la
  classe ne relevent de M, et un juge n'a pas reproduit. Conserve P2 par le troisieme parce qu'il a
  observe un **autre** membre de la classe sur son propre run — donc consigne comme classe dans le
  body plutot que tu.

---

## Etat de l'arbre

**VERIFIE PAR MOI a la redaction, pas rapporte.**

- `git status --porcelain` ⇒ **exactement les 16 entrees stagees** attendues (5 `A ` rounds r1-r5,
  3 `A ` fichiers de code neufs, 8 `M `), plus ce fichier r6 untracked.
- `git diff` ⇒ **VIDE** (controle par marqueur `[END diff]`).
- `git worktree list` ⇒ depot principal (`a67e1eb3 [master]`), les 2 worktrees PERMANENTS
  `.claude/worktrees/agent-a97fde30` et `dazzling-cannon`, et `nexus-098-flip` (detached).
  **Aucun worktree de mesure ne survit** : `r6reg`, `r6dc`, `r6tpg`, `r6sec` ont tous ete retires
  par leurs proprietaires. Pas de P1 de process.
- **Aucun code de production modifie, aucun correctif applique.** Les 22 defauts sont DECRITS.
  Les 25 mutations ont vecu en worktrees jetables, chacune precedee d'un `grep -c` de controle
  (trois mutations rejetees par ce controle avant mesure et rejouees — jamais mesurees a vide).
  Aucun `zz_*` / `*probe*` / `*.bak`, aucun untracked sous `crates/` ou `web/src`.

**Fait a consigner** : `HEAD` a bouge pendant le round, de `17a2b21b` a `a67e1eb3`
(« chore(process): donner un gate au correctif, et interdire la garde redondante »), commit de
PROCESS seul touchant `docs/rust/PATTERNS.md` (+§P76) et `prompts/agent/phase-review-workflow.md`.
Il ne touche aucun fichier de la phase, l'index des 16 entrees est intact, et **§P76 est cite comme
ancre du finding P2-3** — la regle fraiche s'applique donc a son premier cas reel.

---

## Delta de tests exact pour le body du commit

```
+35 (2429 -> 2464)
```

Baseline fin L1 = **2429** (jamais le 2167 perime de CLAUDE.md).
Repartition **recomptee a la source sur les blobs de l'INDEX** et corroboree par les trois commandes
de la row 26 (exit 0 chacune) :

| Module | `#[test]` | nextest |
|---|---|---|
| `sbfb-workflow-engine::artifact_acl` | 20 | 20 passed / 160 skipped |
| `nexus-core-rs::artifact_seal` | 11 | 11 passed / 506 skipped |
| `nexus-shell-daemon::artifact_sealer` | 4 | 4 passed / 486 skipped |
| **Total** | **35** | **= 2464 − 2429, egalite stricte VERIFIEE** |

- Le diff stage de `crates/` ajoute **exactement 35** attributs `#[test]`/`#[tokio::test]` au total :
  **aucun test ajoute ailleurs**. `secret.rs` (+107/−7) n'en ajoute ni n'en retire.
- **Skipped : 7, tous PRE-EXISTANTS** (bras out-of-process de `sbfb-workflow-engine` pilotes par un
  parent). **Phase M ajoute 0 skip.**
- **Doctests : +5** — `compile_fail,E0616` (`:413`) et `compile_fail,E0277` (`:501`), tous deux a
  code d'erreur EPINGLE donc discriminants, plus leurs deux jumeaux qui COMPILENT (`:429`, `:509`),
  plus un bloc ```ignore (`:293`) illustrant un contre-factuel d'une revision anterieure — etiquette
  correcte, ce n'est pas un controle deguise. Workspace : 6 `compile_fail`, 0 echec.
- **Invariants** : **0 dep neuve** (`git diff --cached --stat` sur `Cargo.lock` / `Cargo.toml` /
  `crates/*/Cargo.toml` ⇒ vide) · **0 bump wire** (le seul `*_VERSION` du diff est dans deux
  doc-comments) · **census DOMAIN 29 frozen**.

**Formulation honnete a reprendre dans le body** — ne pas annoncer un vert sec :

> Workspace vert : 2464/2464, 7 skipped (`--no-fail-fast`). Un flake timing observe au 1er run
> complet sur la classe iroh-networked du binaire daemon (`dispatch_loop`, deux membres distincts
> observes sur deux runs : `boot_path_reenters_sync_set_and_delivers_after_reopen` 22,2 s sous
> charge / 5,8 s solo PASS, et `convergence_incremental_task_reaches_remote_replica` 20,4 s) —
> classe env connue (CLAUDE.md), hors perimetre M, routee vers le carry CI.

---

## Ce qui reste pour Codex

**Prerequis** : jouer le **P1** (3 sites de prose) — sans quoi le critere « CLEAN ou P2/P3
documentes uniquement » n'est pas atteint et Codex n'est pas autorise. Le lot P2 devrait partir dans
le meme commit : les correctifs sont **mesures et rediges**, moins de 60 lignes au total, et deux
d'entre eux (P2-1, P2-2) sont deja verifies discriminants dans les deux sens.

**Controle a rejouer AVANT de refermer** (le seul qui aurait attrape `:74-79` au round 5) :

```bash
for f in <les 7 blobs de l'INDEX>; do
  git show ":$f" | grep -niE "nobody can read|permanently unreadable|no way to keep a copy|gone for good|plus personne ne peut les lire|aucun moyen d'en garder"
done
# exiger 0 hit NON BORNE (artifact_acl.rs:1924 excepte, vrai dans son scenario)
```

**A confier a Codex** :

1. **La 6e surface du canari** (`artifact_public_store`, `secret.rs:655` `XorSealer`) n'a ete mutee
   par **personne** ce round. r5 a ferme la non-regression des 5 surfaces d'origine par mesure du
   diff, mais la **discriminance du bras rouge de la surface NEUVE** — publier le canari sous un
   mode qui DOIT sceller et verifier que le collecteur n'est pas vide — n'a jamais ete mesuree
   depuis son ajout. C'est la surface qui rend vraie la borne 15 amendee (P2-9).
2. **Statut INCONNU a la cloture** : le P2 de r5 « `ArtifactDigest` n'a aucun constructeur depuis sa
   forme serialisee » n'a ete ni re-mesure ni re-signale ce round. `grep -n "additive wire"` ⇒
   `:115` et `:1850` inchanges, aucun « EMISSION », aucune borne §19.5 sur l'absence de
   `TryFrom`/`Deserialize`. Ouvert depuis r4 : le verifier ou le router explicitement, pas le
   laisser tomber en silence.
3. **`artifact_sealer.rs:167`/`:169`** (« class » pour un parametre `mode`) et
   **`artifact_acl.rs:521`** (« the host sealer is the only producer ») si le lot P2 ne les touche
   pas — deux reconductions qu'aucune dimension n'a re-detectees.
4. Adversarial sur la **surface `pub` d'`authorise_delete`** : elle est passee de 0 a 4 appelants de
   test directs et reste fail-CLOSED (toute reference du fil rend `UntrustedReference`, jamais un
   faux « oui »), mais c'est une surface de decision neuve exportee sans call-site de production.
5. La **discriminance des 3 `compile_fail`** : deux ont un code d'erreur epingle, le troisieme est
   herite de L1 ; verifier qu'aucun ne peut passer au vert sur une erreur de compilation sans
   rapport (c'est precisement la classe du carry `DOCTEST-ERRCODE-NOT-ENFORCED`, P2-11).
