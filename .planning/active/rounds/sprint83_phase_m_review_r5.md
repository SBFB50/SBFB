# Review — Sprint 83 Phase M (artefacts, ACL et confidentialite) — round 5

## Verdict

**FAIL** — 1 P1 retenu, d'un seul cran.

Le critere d'arret du depot (« CLEAN ou P2/P3 documentes uniquement ») **n'est PAS atteint a ce
round**, et il ne manque qu'une chose, nommement : la **borne de COPIE** absente du document
canonique de securite (`docs/security/THREAT_MODEL.md:2369`, sa liste de bornes §19.5, et l'incise
A16 `:74`), alors que le code du MEME commit l'ecrit deux fois. Tant que cette row affirme le
crypto-shredding en absolu, un enonce de securite du depot est faux a la lettre — la faute cardinale.

Les 18 autres defauts sont des P2/P3 documentables et **ne bloquent pas** le passage a Codex.
Le geste P1 est une incise + une 10e borne + une reprise d'une ligne : **une seule row de
`THREAT_MODEL.md` et deux lignes voisines**. Une fois joue (avec le lot P2 recommande ci-dessous),
le round 6 doit etre CLEAN et le passage a Codex autorise.

Delta de tests pour le body du commit : **+34 (2429 -> 2463)**.

## Perimetre lu

Six dimensions en parallele + trois refuteurs adversariaux, toutes lectures sur les **blobs de
l'INDEX** (`git show :<chemin>`), jamais sur l'arbre.

- **r4-regressions** — les quatre correctifs du round 4 ont-ils cree de nouveaux defauts ?
  (6 mutations en worktree jetable)
- **doc-claims** — chaque affirmation des doc-comments et du threat model, re-mesuree
  (mutation des codes d'erreur rustdoc, renommage de module, retrait de re-export)
- **threat-plan-gate** — coherence documentaire : THREAT §2 A16 / §19.4 / §19.5 / changelog v29,
  `sprint83_plan.md` §M.*/§Y.2/carte/carry, `gate_matrix` row 26, `WORKFLOW_AGENTS_OVERVIEW.md`
- **security-mutation** — 12 mutations + 1 sonde jetable, surface publique enumeree (32 `pub` +
  11 re-exports racine), chemins d'erreur, oracles, double `ScriptedSealer`
- **suites-counts** — les neuf suites §7.4, recomptage a la source de toutes les claims chiffrees
- **never-exhausted** — ce que r1..r4 n'ont jamais lu : lignes de registration, re-exports de
  `lib.rs`, items « reste pour Codex » de r3/r4 non joues

Fichiers : `crates/sbfb-workflow-engine/src/artifact_acl.rs` (2001 l),
`crates/nexus-core-rs/src/artifact_seal.rs`, `crates/nexus-shell-daemon/src/artifact_sealer.rs`,
`crates/sbfb-workflow-engine/src/secret.rs`, les 3 lignes de registration,
`docs/security/THREAT_MODEL.md`, `.planning/active/sprint83_plan.md`,
`.planning/active/sprint83_gate_matrix.md`, `.planning/active/sprint83_kickoff.md`.

**28 findings bruts, 6 clusters de doublons fusionnes => 19 defauts distincts : 1 P1, 8 P2, 10 P3.**
Aucune ancre inventee ; toutes re-greppees a la redaction (voir §Findings tombes).

## Findings retenus

### P1

| Sev | Titre | Ancre | Pourquoi (mesure) | Correctif |
|---|---|---|---|---|
| **P1** | La row `T-ARTIFACT-DELETION-THEATRE` enonce le crypto-shredding en ABSOLU alors que le code du meme commit le borne deux fois — le balayage r4 a atterri sur 2 des 3 surfaces qu'il nommait *(fusion des index #1/#4/#17, trois dimensions independantes)* | `docs/security/THREAT_MODEL.md:2369` + les 9 bornes §19.5 (`:2376-2439`) + A16 `:74` — contre `crates/sbfb-workflow-engine/src/artifact_acl.rs:930-933` et `:531-536` | `sed -n '2369p'` : « supprimer detruit la CLE : le chiffre devient **definitivement illisible** … **plus personne ne peut les lire** … (`ReadCapability` n'etant pas `Clone`, l'appelant n'avait **aucun moyen d'en garder copie**) ». Le MEME commit ecrit l'inverse : `:930-933` « for a holder who **did not copy the key out of the capability first** » et `:531-536` « `.to_vec()` is one keystroke away, and a caller who did that **can rebuild a working capability** ». `grep -niE "copi\|to_vec\|keystroke"` sur `:2376-2439` (les NEUF « Bornes honnetes — ce que M ne ferme PAS ») ⇒ **exit 1, zero hit** — re-mesure a la redaction. Le correctif r4 nommait TROIS surfaces : les deux de code ont atterri verbatim, la troisieme non. Precedent interne qui tranche la severite : le depot A DEJA ecrit cette borne pour le type jumeau — A14 `:74` porte « la route COPIE reste une discipline de la reliure, cf. §19.4 borne 15 » pour `BoundSecret` ; A16, ou la garantie est vendue PLUS fort, ne la porte pas. Scenario entrees→etat faux : la reliure Phase P/Y lit §19.5 pour enumerer le risque residuel, n'y trouve pas la copie, cable un adaptateur qui `.to_vec()` une capability, `delete` rend `Ok(())` et le produit affiche « donnees detruites » sur un artefact toujours lisible — exactement le theatre que la row existe pour interdire. | UN geste, trois lignes du meme fichier. (a) Row `:2369` : « … le chiffre devient definitivement illisible **pour un detenteur qui n'a pas recopie la cle hors de la capability** » ; remplacer « plus personne ne peut les lire » par « plus aucun **tiers** ne peut les lire » ; requalifier la parenthese en « aucun moyen **SUPPORTE** d'en garder copie (`ReadCapability` n'est pas `Clone`) ». (b) **10e borne §19.5**, au patron exact de la borne 15 de §19.4 deja ecrite pour `BoundSecret` : « **Borne de COPIE** — `ReadCapability::expose` rend un emprunt que `.to_vec()` transforme en copie possedee, et `ReadCapability::new` en rebatit une qui fonctionne. Le type ferme la route de SERIALISATION par construction, jamais la copie ; ne pas copier est la discipline de l'appelant. » (c) Reprendre l'incise dans A16 `:74`. |

### P2

| Sev | Titre | Ancre | Pourquoi (mesure) | Correctif |
|---|---|---|---|---|
| P2 | La garde de provenance ajoutee a `ArtifactPolicy::authorise_delete` par r4 est **inatteignable et mutation-silencieuse**, et elle a rendu la garde du vault non-discriminante : la phrase « Mutation-teste » de `:2371` est fausse a la lettre pour chaque garde prise seule *(fusion #0/#3/#18/#23, deux dimensions)* | `crates/sbfb-workflow-engine/src/artifact_acl.rs:789-793` (branche neuve) et `:977-982` (garde du vault) ; unique appelant `:984` ; `docs/security/THREAT_MODEL.md:2371` | Worktree jetable, baseline 172 passed / 7 skipped. M1 (garde vault seule retiree) ⇒ **172 passed** ; M2 (branche `authorise_delete` seule) ⇒ **172 passed** ; M5 (`panic!` en tete de la branche) ⇒ **172 passed** — la branche n'est JAMAIS atteinte ; M6 (les DEUX) ⇒ **2 failed**. Verifie a la redaction sans muter : `delete` teste `is_locally_minted` a `:977` et rappelle le MEME test via `authorise_delete` a `:984` → `:789` ; `grep -rn authorise_delete crates/ --include=*.rs` ⇒ 4 hits, 1 seul appel, **0 appelant de test** (contraste dans le meme fichier : `authorise_egress` EST appelee directement par un test a `:1508`). La propriete de securite **tient** (M6 rougit) : ce qui est perdu est la discriminance de chaque couche et la recette de falsification en prose. Pas P1 : aucun scenario entrees→etat faux — ni retirer l'une ni retirer l'autre ne produit un `delete` qui de-tague une reference forgee. | 3 lignes + une demi-phrase. Dans `an_edited_reference_loses_its_provenance` (`:1955`), ou `edited` existe deja : `assert!(matches!(ArtifactPolicy::authorise_delete(&edited), Err(ArtifactError::UntrustedReference { .. })))` — discriminant par construction (garde retiree, `edited.mode == Local` ⇒ `deletable()` ⇒ `Ok(())` ⇒ rouge). Puis reecrire `:2371` en « retirer les **DEUX** gardes fait rougir … » (mesure M6). **NE PAS** supprimer la garde de `delete` : elle vit ou vit l'effet destructeur. **Piege du round** : apres correction, RE-MESURER que la mutation d'une seule garde rougit, sinon r4 se rejoue a l'identique. |
| P2 | « The host sealer is the only producer » sur `ReadCapability::new` — le meme fichier dit deux fois l'inverse | `crates/sbfb-workflow-engine/src/artifact_acl.rs:521` (re-greppe a la redaction, verbatim confirme) — contre `:534` et `:942` | `:521` « The host sealer is the only producer. » immediatement suivi de `pub fn new(key: Vec<u8>)` (`:522`), type re-exporte racine (`lib.rs:203-206`). `:534` : « a caller who did that **can rebuild a working capability with `ReadCapability::new`** » ; `:942` : « **`ReadCapability::new` being public**, `delete(local, Some(any 32 bytes))` untagged the blob ». Aggravant de forme : `:367` emploie la formule identique la ou elle est structurellement VRAIE (`RefOrigin::ThisVault`, variante privee). Pas P1 : l'objet de la seconde phrase est elide et se lit comme « producteur de materiel FRAICHEMENT frappe » (vrai du CSPRNG), et la conclusion operationnelle correcte est ecrite DEUX FOIS dans le meme type ; un implementeur devrait ignorer les deux pour se tromper. Ambiguite reelle sur une phrase de securite d'un type `pub`, dans le fichier qui porte la trace du bug qu'elle a inspire. | Une phrase : « Wrap key material. **PUBLIC, donc JAMAIS une preuve d'origine** : n'importe quel appelant peut en frapper une — c'est precisement pourquoi `ArtifactVault::delete` verifie la POSSESSION par un `unseal` et non par `Option::is_some`. Le scelleur hote est le seul producteur **ATTENDU**. » |
| P2 | La decision « `PartialEq` compte `origin` » n'a **aucun controle discriminant** : l'unique assertion compare deux references qui different AUSSI par `mode` | `crates/sbfb-workflow-engine/src/artifact_acl.rs:1996-1998` (« `==` counts provenance » + `assert_ne!(edited, published)`) — contre la decision documentee `:338-348` | Re-greppe a la redaction : `edited` est construit `:1972-1976` en `ArtifactMode::Local`, `published` `:1961-1967` en `ArtifactMode::Published` — l'`assert_ne!` est satisfait par le seul ecart de `mode`, il ne peut pas echouer quand `origin` est ce qui change. MUTATION (`impl PartialEq` manuel sur `digest`+`mode`+`retention`, `origin` ignore) ⇒ `artifact_acl` **19 passed**, workspace crate **172 passed** : rien ne rougit. Une decision documentee sur 11 lignes (« Decided rather than inherited from the derive ») n'a aucune garde. Pas P1 : `grep -rn ArtifactRef crates/ --include=*.rs` hors module ⇒ le seul `pub use` de `lib.rs` — **0 consommateur de `==`**, et aucune garde de securite n'en depend (`delete` decide sur `is_locally_minted()`). | Une assertion apres `:1998`, identique en TOUT champ public et differente par la seule origine : `assert_ne!(ArtifactRef::from_untrusted_wire(published.digest().clone(), published.mode(), published.retention()), published, "identical in every public field, different only in origin");`. **Verifie discriminant dans le meme worktree** : code propre ⇒ 19/19 ; sous la mutation ⇒ **18 passed, 1 failed**. |
| P2 | Le bras `DeletionRefused` de `delete` peut re-zeroiser la capability sans qu'aucun test ne rougisse — 4e instance d'une classe fermee a 3/4 par r4 | `crates/sbfb-workflow-engine/src/artifact_acl.rs:984-986` ; claim `sprint83_plan.md:1104-1107` | MUTATION sur `:985` (`drop(capability); capability: None`) ⇒ **172 passed** — aucun controle. La meme mutation sur les trois bras freres est DISCRIMINANTE (`:980`, `:1001`, `:1007` ⇒ 1 rouge chacune). Derive sans muter : `grep -n '\.delete('` ⇒ 11 sites, le SEUL atteignant ce bras est `published_artefact_refuses_deletion` (`:1447`), qui passe `none` — le `None` rendu par `publish(Published, …)`. Sonde jetable : `delete(&published_minte_ici, Some(cap_d_un_autre_artefact_local))` ⇒ `Err(DeletionRefused)`, capability rendue, `other` toujours lisible ⇒ PASS sur code propre, 1 rouge sous mutation. Pas P1 : le **code est correct** et la claim du plan est VRAIE aujourd'hui ; c'est un risque de regression, pas un etat faux. `feedback_close_class_not_instance`. | 4 lignes dans `published_artefact_refuses_deletion` (`:1432`), au patron deja ecrit a `:1858-1874` : publier un second artefact `Local`, `vault.delete(&published_ref, Some(other_cap)).expect_err(..)`, asserter `DeletionRefused`, puis recuperer `refusal.capability` et asserter que `vault.read(&other, Some(&back))` rend encore le clair — le **DROIT DE LECTURE**, pas le tag. |
| P2 | §19.4 borne 15 : « 6 rendus du moteur » est un compte faux, et `:1975` du meme commit dit l'inverse *(fusion #5/#8)* | `docs/security/THREAT_MODEL.md:2066` — contre `:1975` | `sed -n '2066p'` (re-greppe) ⇒ « 15. **Le canari couvre 6 rendus du moteur + le blob au repos** (la 6e surface, `artifact_public_store`, … ». `:1975`, ecrit par le correctif r4 ⇒ « les **CINQ PREMIERES** sont des rendus de types de PRODUCTION ; la 6e, `artifact_public_store`, … est le **MODELE** du magasin ». Table du sweep `secret.rs:721-728` : 5 rendus + 1 contenu de magasin ; `grep -rn "ArtifactBlobs for" crates/` ⇒ 3 impls, **toutes sous `#[cfg(test)]`**. Le chiffre 6 reste EXACT partout ou il designe le SWEEP (`assert_eq!(surfaces.len(), 6)`). C'est l'amendement r4 de `:1975` qui cree la contradiction — jumeau edite, celui-ci non balaye. Consequence bornee : la borne 15 NOMME la 6e surface et renvoie a §19.5, dont la borne 1 ecrit deja « aucun implementeur de production ». | Une ligne : « 15. **Le canari couvre 5 rendus du moteur + le magasin d'artefacts (Phase M — MODELE du magasin, `ArtifactBlobs` n'a aucun implementeur de production, cf. §19.5 bornes 1 et 6) + le blob de coffre au repos** (la 6e surface du sweep, `artifact_public_store`). » Ne PAS toucher au 6 la ou il designe le sweep (`:1975`, `:2372`, `:2814`, `secret.rs:735`). |
| P2 | Row 26 : « le filtre positionnel exige un module de lib » est falsifie par la troisieme commande de la row elle-meme | `.planning/active/sprint83_gate_matrix.md:60`, clause finale ; miroir `.planning/active/sprint83_plan.md:1069-1071` | `cargo nextest run -p nexus-shell-daemon artifact_sealer --locked` ⇒ **4 passed, 486 skipped**, exit 0, binary id `nexus-shell-daemon::bin/nexus-shell-daemon` — un module de **BINAIRE**. Fait structurel decisif : `crates/nexus-shell-daemon/src/lib.rs` **n'existe pas**, `Cargo.toml` ne declare qu'un `[[bin]]`, `artifact_sealer` est enregistre par `mod artifact_sealer;` dans `main.rs`. La CONCLUSION de la row reste vraie, mesuree separement : `-p nexus-shell-daemon loopback_token` (fichier `tests/loopback_token.rs`) ⇒ **exit 4, 0 test**. Severite calibree sur la decision r4 pour CETTE row (P1→P2 : « le critere ferme bien sur des tests qui existent ; c'est la clause de MECANISME qui est fausse »). | Row 26 : « Filtre positionnel : il matche le **CHEMIN DE MODULE** du test — module de lib **ou de binaire** (`artifact_sealer` vit dans le bin du daemon, mesure 4/4) ; un fichier `tests/artifact_acl.rs` ne matcherait pas, son nom devenant l'ID du binaire (mesure sur `tests/loopback_token.rs` ⇒ exit 4). » Meme amendement au plan `:1070`. |
| P2 | Carry `DOCTEST-ERRCODE-NOT-ENFORCED` : sa branche de sortie n°2 laisse debout une phrase FAUSSE | `.planning/active/sprint83_plan.md:1108-1114` ; cible `crates/nexus-core-rs/src/vault.rs:195-197` | `sed -n '195,197p' vault.rs` (hors diff, lu dans l'arbre a dessein) ⇒ « The absence of `Serialize` is PINNED here … **the error code is fixed so the test cannot go green on an unrelated compilation error** » — l'exact contraire de la mesure que le carry publie lui-meme (rustc 1.95.0, editions 2021 ET 2024). Le carry ne nomme pas cette phrase, et son exit propose « OU decision ecrite que le doctest seul suffit la-bas » : branche qui ferme le carry SANS corriger l'enonce. Le CODE est hors phase M ; la **redaction du carry** est un livrable de M. | Completer le carry : « … `vault.rs:195-197` ne l'est pas — **et y AFFIRME encore le contraire**. Exit = jumeau qui COMPILE pose sur `vault.rs` **ET** phrase corrigee ; la seconde n'est pas optionnelle, une decision ecrite ne peut pas rendre vrai un enonce mesure faux. » |
| P2 | `ArtifactDigest` n'a **aucun** constructeur depuis sa forme serialisee : `from_untrusted_wire` n'est pas alimentable par un payload de fil, et « additive wire (AM-12) » n'est vrai qu'en EMISSION *(fusion #19/#24)* | `crates/sbfb-workflow-engine/src/artifact_acl.rs:256-269` (seul constructeur `of_stored_bytes(&[u8])`) ; claims `:115`, `:122-124`, `:382-384`, `:1794-1796` ; premisse `THREAT_MODEL.md:2367` | `grep -rn ArtifactDigest crates/ --include=*.rs | grep -E "impl|Deserialize|FromStr|TryFrom"` ⇒ **2 hits**, `impl ArtifactDigest` (`:259`) et `impl Display` (`:271`). Sondes de compilation en worktree jetable : `serde_json::from_str::<ArtifactRef>(payload_gele_de_:1812)` ⇒ **E0277 `!Deserialize`** ; `"aaaabbbb".parse::<ArtifactDigest>()` ⇒ **E0277 `!FromStr`** ; `ArtifactDigest::try_from("aaaabbbb")` ⇒ **E0277 `!TryFrom<&str>` + `!From<&str>`**. Consequences : la forme de fil que `reference_wire_form_is_pinned` GELE n'est parsable par rien ; l'acteur-test n°1 (« Another node (wire) ») ne peut pas appeler la fonction dont le nom lui est destine ; la premisse de `T-ARTIFACT-DIGEST-AS-READ-RIGHT` (« un tiers qui a VU PASSER le digest ») n'est pas exprimable dans l'API. Direction du gap : plus **FERME** que le document ne le dit — aucune propriete affaiblie, donc pas de P1. Item 1 de « Ce qui reste pour Codex » de r4, jamais traite. | **Deux phrases, zero code.** Sur `:382-383` : l'entree est un digest que l'appelant a RECALCULE sur des octets qu'il detient (ou clone d'une reference), jamais la chaine hex qui voyage. Une borne §19.5 : « Un digest observe sur le fil n'est pas RE-ADMISSIBLE ; ajouter un `TryFrom<&str>` ou un `Deserialize` a la reliure rendrait ATTEIGNABLES d'un geste les forgeries bornees en 2 — gratuit a ecrire aujourd'hui, fige au premier cablage. » Preciser `:115`/`:1794` en « additive wire dans le sens **EMISSION** ». **CONSERVER** la decision `Serialize`-seul (`:281-283`), qui est juste ; ne PAS ajouter de constructeur maintenant. |

### P3

| Sev | Titre | Ancre | Pourquoi (mesure) | Correctif |
|---|---|---|---|---|
| P3 | Le plan compte « deux `compile_fail` » 56 lignes apres « les trois doctests du moteur » | `.planning/active/sprint83_plan.md:1167` — contre `:1111` | `cargo test -p sbfb-workflow-engine --locked --doc` ⇒ 3 blocs `compile_fail` (`artifact_acl.rs:413`, `:501`, `secret.rs:147`) + 3 jumeaux + 1 `ignore`. Les DEUX `E0277` sont « absence de `Serialize` » (`ReadCapability` et `BoundSecret`) : le parenthetique ne les distingue pas. Les deux enonces sont compatibles dans leurs referentiels respectifs ; le residu reel est que la ligne Tests sous-liste le jumeau pose sur `secret.rs`. | « Doctests : trois `compile_fail` — `E0616` (champs prives d'`ArtifactRef`) et `E0277` (`ReadCapability` sans `Serialize`), tous deux NEUFS en M, plus l'`E0277` de `BoundSecret` herite de L1 —, **chacun apparie a un jumeau qui COMPILE** ; rustdoc n'enforce pas le code annote. » |
| P3 | « its class argument » nomme un parametre qui s'appelle `mode` (r4 P3 #44, non corrige) | `crates/nexus-shell-daemon/src/artifact_sealer.rs:167` et `:169` | `grep -n class` ⇒ 5 hits ; `:54-57` sont historiques et dates, `:167`/`:169` decrivent une mesure DATEE mais nomment un parametre du code ACTUEL, `fn seal(&self, mode: ArtifactMode, …)` (`:69`). | Deux mots : `:169` → « ignores its **mode** argument » ; `:167` → « replacing the reference's **mode** ». |
| P3 | Deux renvois de §19.5 (« borne 25 », « borne 16 ») pointent §19.4 sans le dire ; bornes 7/8/9 sans renvoi entrant *(fusion #7/#14, r4 P3 #38/#40 non corriges)* | `docs/security/THREAT_MODEL.md:2400` et `:2407` | Re-greppe a la redaction : les deux vivent DANS la liste de §19.5, qui numerote **1..9 continues** ; cibles reelles en §19.4 (item 25 = keystore couche 2 INERTE, item 16 = oracle de taille). Convention du document ailleurs : un « borne N » nu designe la section courante. Navigabilite, aucun etat faux. | « (**§19.4** borne 25, carry `KEYSTORE-LAYER2-INERT`) » et « meme classe residuelle que la **§19.4** borne 16 ». Ajouter « (borne 9 ci-dessous) » dans la row `T-ARTIFACT-DELETION-THEATRE` (`:2369`) — **meme row que le P1**, a jouer d'un seul geste. |
| P3 | Le kickoff decrit encore la Phase M par la doctrine REFUTEE (classes/audience) — 4e surface non balayee | `.planning/active/sprint83_kickoff.md:639` | Toutes les autres occurrences de `read_audience`/`public/prive` sont dans un contexte de supersession explicite (plan §M.1, encadre M-d/M-e, THREAT `:2351-2358`, changelog v29, row 26). Le kickoff est la seule surface au PRESENT. Le patron existe dans le fichier (`L … PRE-COUPEE au plan avec L1 — AM-4`, idem `P`). **Contre-precedents qui coupent la severite** : `sprint83_kickoff.md` n'est PAS dans le diff stage ; le commit ECRIT sa propre doctrine (`plan:1033-1039` « Le preflight est un artefact DATE qu'on ne reecrit pas ») ; et I1 (re-coupe B6) n'a rien recu. **1 vote de refutation sur 3** — l'objet est un scope cut arbitre PO. | Variante la moins chere : une clause dans l'encadre de re-coupe du plan (`:1033-1039`) nommant le kickoff comme second artefact date superseded. Eviter d'ouvrir une 16e entree stagee. |
| P3 | En-tete de la Phase M : « (pre-coupe AM-4) » contredit son propre encadre et l'enumeration AM-4 de §R2 | `.planning/active/sprint83_plan.md:1023` — contre `:1025` et `:2559` | `:1023` = « (pre-coupe AM-4, preflight PLAN-ADAPT) », `:1025` juste dessous = « **Re-coupe mid-sprint du 2026-07-28.** », §R2 `:2559` enumere AM-4 = AI×3, AJ×2, P, L (**sans M**). Ecrit par `0fdbca91`, laisse par ce commit qui reecrit la section juste en dessous. | « ## Phase M — Artefacts, ACL et confidentialite (**re-coupe AM-7 du 2026-07-28**, preflight PLAN-ADAPT) », au patron de la Phase I1. |
| P3 | §M.4 n'a pas suivi la row 26 : une seule commande d'acceptation la ou la row en nomme trois | `.planning/active/sprint83_plan.md:1172-1176` — contre `.planning/active/sprint83_gate_matrix.md:60` | §M.4 ne lance que `-p sbfb-workflow-engine artifact_acl` et `-p sbfb-workflow-engine`. Ni `artifact_seal` (11 tests, nexus-core-rs) ni `artifact_sealer` (4 tests, daemon) n'est atteint — dont `local_artefact_round_trips_end_to_end_under_real_crypto`, le seul test sous crypto REELLE. §M.4 n'est pas FAUX, il est plus etroit que la row que l'audit gate S84 rejoue. | Ajouter a §M.4, mot pour mot depuis la row 26 : `cargo nextest run -p nexus-core-rs artifact_seal --locked` et `cargo nextest run -p nexus-shell-daemon artifact_sealer --locked`, exit 0 chacun. |
| P3 | `DeleteRefusal` imprime son message DEUX FOIS dans toute chaine d'erreur *(fusion #20/#25)* | `crates/sbfb-workflow-engine/src/artifact_acl.rs:638-642` (`Display` → `self.error.fmt(f)`) et `:644-648` (`source()` → `Some(&self.error)`) | Sonde : marche de la chaine standard ⇒ `"a published artefact cannot be deleted: … : a published artefact cannot be deleted: …"`. `grep -rn "From<DeleteRefusal>" crates/` ⇒ 0. La convention `Error` veut qu'une `source` designe la cause SUIVANTE. Aucun materiel divulgue (voir §Ce qui tient). | Une ligne : soit `write!(f, "deletion refused")` dans `Display` en gardant `source()`, soit retirer l'override de `source`. |
| P3 | La reduction naturelle vers `ArtifactError` re-detruit la capability que `DeleteRefusal` vient de rendre — non signale | `crates/sbfb-workflow-engine/src/artifact_acl.rs:953-963` (§ « A refusal gives the capability back ») | Fait de type : aucun `From<DeleteRefusal> for ArtifactError` n'existe, donc `.map_err(|r| r.error)?` est la seule facon d'obtenir un `Result<_, ArtifactError>` homogene avec `read`/`publish` — et elle droppe donc zeroise `refusal.capability`. Idem `.ok()`, `let _ =`. La PROPAGATION (`?` vers `anyhow`/`Box<dyn Error>`) la GARDE : c'est la reduction, pas la propagation, qui detruit. La doc ne ment pas ; il manque une phrase de garde. | Une phrase en fin de section : « Le retour n'aide que l'appelant qui la RECUPERE : reduire ce type a son `error` (`.map_err(|r| r.error)`, `.ok()`, `let _ =`) re-detruit la capability. Propager le `DeleteRefusal` entier la garde vivante. » |
| P3 | `ArtifactRef` derive `Clone` — zero site d'appel, et c'est le seul derive que la doc du type ne justifie pas | `crates/sbfb-workflow-engine/src/artifact_acl.rs:349` — contre `:338-348`, `:281-283`, `:357-360` | `grep -n '\.clone()'` ⇒ 5 occurrences, **toutes** sur `ArtifactDigest`, aucune sur `ArtifactRef` ; hors module, seul le `pub use` de `lib.rs`. **1 vote de refutation sur 3** : un clone EST la meme reference, il conserve `ThisVault` a dessein et ne fabrique aucune provenance (r4 l'avait deja porte au credit du design). Residu purement redactionnel. | Une phrase a cote de « # Why `PartialEq` counts `origin` ». **NE PAS retirer le derive** : le remettre plus tard est additif, mais son absence forcerait des `from_untrusted_wire` de convenance dans les tests, ce qui degraderait la provenance. |
| P3 | Le « controle positif » de `stored_bytes_never_contain_plaintext` s'execute sur le litteral du canari, pas sur la botte de foin qu'il pretend controler | `crates/nexus-core-rs/src/artifact_seal.rs:256-263` — contre l'assertion gardee `:250-254` | Le controle est `CANARY.as_bytes().windows(CANARY.len()).any(|w| w == CANARY.as_bytes())` : une tautologie sur un litteral. Ce n'est PAS « the same search » — la botte de foin differe (`sealed.stored` vs le clair). Il attrape une taille de fenetre mutee, pas une botte VIDE ou tronquee — fermee **ailleurs**, par `round_trip_recovers_plaintext` (`:241-246`), donc la propriete est gardee et seule la PHRASE sur-decrit. Contraste dans le meme lot ou le controle positif est REEL : bras 4 de `digest_is_not_read_right`, et le bras rouge neuf de `secret.rs`. | Faire porter le controle sur une botte de meme nature (le clair rendu par `open_artifact` sur ce meme `sealed`), ou dire honnetement ce qu'il pin : la RECHERCHE. **Ne pas laisser « the same search »** — c'est la formulation qui a produit la vacuite de la 6e surface au round 4. |

### Trois angles jamais couverts en cinq rounds — a trancher AVEC le lot P2 (non cotes, decisions)

1. **Visibilite des lignes de registration** (item 3 « reste pour Codex » de r4, deux rounds sans
   reponse). Mesure : `crates/nexus-core-rs/src/lib.rs` ajoute `pub mod artifact_seal;` dans un crate
   consomme par le daemon, le worker ET le launcher ; le daemon ajoute `mod artifact_sealer;`
   (**prive**, module de BIN — `crates/nexus-shell-daemon/src/lib.rs` n'existe pas). **Reponse etablie
   ce round** : le `pub` est OBLIGATOIRE et non retractable a la baisse — `artifact_sealer.rs`
   consomme `nexus_core_rs::artifact_seal` depuis un AUTRE crate, `pub(crate)` ne compilerait pas ; et
   les 41 modules de `nexus-core-rs` sont **tous** `pub mod`, `vault`/`keystore`/`crypto` compris,
   sans aucun feature-gate crypto a imiter. Il n'y a donc **pas de defaut**, mais la question merite
   **une ligne de decision ecrite** (§M.2 ou borne §19.5) : une surface publique ne se retracte pas
   apres la reliure. Residuel cosmetique non cote : `artifact_seal` et `hooks` sont les 2 seuls
   modules sur 41 sans ligne `pub use` racine.
2. **Non-regression des 5 surfaces PRE-EXISTANTES du canari** (item 2 de r4). Deux rounds l'ont
   laissee ouverte. **Fermee ici par mesure du diff** : `git diff --cached -U0 -- secret.rs` montre
   les 5 entrees d'origine et `find_material` (7 bras) en lignes de **CONTEXTE pures** ; le bloc neuf
   est insere d'un seul tenant (`let artefact_surface = { … };`). Aucune construction n'a bouge,
   aucune surface existante n'a ete editee. **Il n'y a pas de zone de regression silencieuse.**
3. **Chemin de PANIQUE de `delete`** — le brief le demandait, aucun finding ne l'a couvert. `capability`
   est prise PAR VALEUR ; une panique dans `get_unauthenticated` (`:998`), `unseal` (`:1005`) ou
   `untag` (`:1012`) — trois methodes de traits implementes par l'HOTE — deroule la pile, droppe la
   capability et la zeroise sur un appel qui n'a rien supprime. Direction **SURE** (cle detruite,
   jamais une cle survivante sur un succes annonce), non corrigeable a bon compte (`catch_unwind` sur
   du code hote ou `ManuallyDrop` sont pires que le mal). **A NOMMER en borne §19.5**, au meme titre
   que la borne « blob absent » que r4 a eu l'honnetete d'ecrire (`:948-953`) — a joindre au geste P1,
   qui edite deja cette section.

## Ce qui tient

**Trois des quatre correctifs r4 tiennent sous mutation, et deux ferment bien la CLASSE.**

- **`DeleteRefusal` — aucun chemin ne perd plus la capability.** Les cinq bras de refus (`:978`,
  `:985`, `:989` [`None` legitime et declare honnetement a `:633-635`], `:999`, `:1005`) rendent tous
  la capability ; **0 `?` faillible ne subsiste** apres le move ; l'unique `drop(capability)` est a
  `:1012`, APRES `untag`. La claim `plan:1104-1107` « Tout refus rend `DeleteRefusal { error,
  capability }` » est VRAIE a la lettre.
- **Le type neuf n'ouvre rien cote materiel** — reponse mesuree a la question explicite du brief :
  `format!("{:?}", refusal)` ⇒ `DeleteRefusal { error: DeletionRefused, capability:
  Some(ReadCapability(<redacted 32 bytes>)) }`. Le `Debug` MANUEL redacteur de `ReadCapability`
  (`:552-556`, epingle en egalite EXACTE par `capability_redacts_and_zeroises` `:1709`) porte a
  travers le `derive(Debug)`. `Display` forwarde a `ArtifactError`, dont le seul champ libre est
  `SealerRefused { reason }` alimente par trois messages constants plaintext-free
  (`artifact_sealer.rs:73-76`, `:88-92`). Construire un `DeleteRefusal` a la main ne donne rien qu'un
  appelant n'ait deja.
- **6e surface du canari — bras rouge DISCRIMINANT dans les DEUX directions.** Collecteur neutralise
  ⇒ `canary_absent_everywhere` **FAIL** (`secret.rs:709`, `left: None / right: Some("raw text")`) ;
  double `XorSealer` rendu identite (« un scelleur qui ne scelle pas ») ⇒ « canary material leaked
  into artifact_public_store » **FAIL**. La classe de vacuite cotee P2 par r4 est fermee, et pour la
  bonne raison (le bras publie en `Published`, donc la surface collectee est l'ASCII du canari,
  `from_utf8_lossy` y est l'identite).
- **Jumeaux de doctests — discriminants INDIVIDUELLEMENT.** Renommage du module `artifact_acl`
  (`#[path]` + alias, consommateurs recables) ⇒ les 2 jumeaux `artifact_acl` **FAIL** (`E0432`)
  pendant que leurs `compile_fail` restent verts ; `BoundSecret` retire de la re-export racine ⇒ le
  jumeau `secret.rs:155` FAIL, son `compile_fail:147` vert ; remettre `pub mode` ⇒ le
  `compile_fail,E0616` FAIL. Et la claim rustdoc est **REPRODUITE, pas un finding** : muter
  `E0616`→`E0308` et `E0277`→`E0001` ⇒ **3 passed, 0 failed** — rustdoc n'enforce PAS le code annote,
  exactement ce que `:1975` et les 3 sites ecrivent.
- **Tests des rounds 3-5, mutes un par un, tous discriminants** : retrait du gate `unseal` de `delete`
  (`:1004`) ⇒ `deleting_with_a_foreign_capability_is_refused` FAIL (question laissee ouverte au point
  4 de r4 : **fermee**) ; `:980` ⇒ `delete_refuses_a_reference_this_vault_did_not_mint` FAIL ;
  `:1001` ⇒ `deleting_an_artefact_whose_blob_vanished_returns_the_capability` FAIL.
- **Surface publique enumeree** (32 `pub` + 11 re-exports racine, EXACTEMENT les 11 items publics
  d'`artifact_acl`, `RefOrigin` reste prive) : aucun `&mut`, aucun `Deserialize`, aucun `impl From<>`,
  aucun `set_`. **Chemins d'erreur** : 0 `unwrap` / `expect(` / `panic!` / `unsafe` / `todo!` / cast
  tronquant / indexation / soustraction unsigned dans la moitie PRODUCTION des trois fichiers neufs.
  `open_artifact` garde la longueur de cle PUIS `stored.len() < NONCE_LEN` AVANT `split_at`.
- **Double `ScriptedSealer` audite bras par bras** : `next_key` demarre a 1, les cles valent 1 et 2,
  les capabilities inventees (`0u8`, `1u8`, `7u8`) tombent sur des tags distincts — **aucun test n'est
  faussement vert**. Fragilite latente sans consequence (cle 0 rendrait le double identite et ferait
  ROUGIR `digest_is_not_read_right` ; il faudrait 256 publications dans un meme test).
- **Diff bidirectionnel des tests — CLEAN.** 19 + 11 + 4 = **34** `#[test]`. Sens nomme→livre : 0 nom
  invente. Sens livre→nomme : **0 test non nomme** ; les deux puces de FAMILLE de §M.3 sont
  **bijectives** (11 items ↔ 11 tests, 4 ↔ 4). Le trou de r4 (13/33 non nommes) est ferme.
- **Chiffres recomptes a la source, exacts** : §19.5 = bornes **1..9 continues, 0 doublon** ; « 6 rows
  neuves » ⇒ `grep -c '^| \*\*T-ARTIFACT'` = 6 ; « **neuf** methodes de transfert (la dixieme, `new`,
  est un wrapper) » ⇒ `grep -c "pub async fn" blobs.rs` = **9**, `new` sync `:73` (borne 8 et
  `artifact_acl.rs:721-723` corrigees, r4-P2 ferme) ; « 6 surfaces » du canari ⇒ table
  `secret.rs:721-728` = 6 + pin de cardinalite `:733-737` ; « 45 phases » ⇒ `grep -cE '^## Phase '`
  = 46 moins Phase 0 ; `DOMAIN census [29 frozen]`.
- **Ordre du changelog v29 aligne** sur la table §19.5 (DIGEST → MODE-RELABEL → DELETION-THEATRE →
  RETENTION-BYPASS → FORGED-REF-DESTRUCTIVE → LOCAL-EGRESS) : sa self-claim est VRAIE. v29 en tete de
  la liste descendante.
- **Ancres fichier:ligne toutes exactes** : `http.rs:272`, `node.rs:442`, `http.rs:246-256`,
  `blobs.rs:130-133`, `graph.rs:599`, `vault.rs:22-31`, `csp.rs:33`, `artifact_sealer.rs` (**0
  reference hors de son fichier** ⇒ « 0 call-site » exact).
- **Carte de phases** : rail mesure caractere par caractere (perl `-CSD`) ⇒ **colonne 66** sur les 34
  lignes qui le portent, `┘` a 66 ; `M1` (74) deborde comme `L1` (83), `I` (74), `P` (74) — pattern
  homogene. La seule ligne a 66 sans rail (`N`, `:172`) est ANTERIEURE au commit.
- **Contradiction at-rest du P1-B (round 3) FERMEE** : `artifact_seal.rs:26-39` et son jumeau
  `artifact_acl.rs:86-106` disent la meme chose (« no custody, therefore no at-rest bound », `vault`
  cite comme PATRON) ; premisse re-mesuree : `grep -rniE "artifact|artefact" vault.rs keystore.rs` ⇒
  3 hits, tous le mot anglais generique, **0 API de custody** ; keyring declare **sans feature
  backend** (`nexus-core-rs/Cargo.toml:105-113`).
- **Compteurs nus** : `artifact_seal.rs` ⇒ **0**. Les survivants (`artifact_acl.rs:1034-1035`,
  `:1798`, `artifact_sealer.rs:167-168`) sont tous **DATES et SITUES**, au patron reserve par r4.
  « the fields are public » ⇒ **0 hit**.
- **Les neuf suites §7.4 passent, exit 0 chacune** : fmt vide ; clippy 0 warning ;
  `cargo nextest run --workspace --locked` = **2463 passed, 7 skipped** (61,052 s) ; doctests 0 echec ;
  release build 0 ; `check-frontier-contracts.sh` clean (DOMAIN census 29 frozen, bridge-parity 16,
  hitl-parity 15, frontier-tag 2).
- **0 dep neuve** (`git diff --cached --stat -- '*Cargo.toml' '*Cargo.lock' 'web/package.json'` =
  vide) ; **0 bump wire** (le seul hit `_VERSION` est de la prose ajoutee).
- **`addresses_same_artefact` exerce et correct** (`:1999`, corps `self.digest == other.digest`,
  conforme a sa doc `:470-476` et a l'usage Phase Y annonce).

## Findings tombes en refutation

**Aucun finding retenu n'est tombe** au fond : les 28 ancres brutes ont toutes ete re-greppees sur les
blobs de l'INDEX, aucune n'est inventee, et les 6 mutations non reproductibles telles quelles sont
toutes DERIVABLES par lecture du chemin d'appel. **0 refutation au fond.**

Ce qui a change au jury, ce sont des **severites** et des **doublons** :

- **Six P1 bruts retrogrades en P2** apres application du bareme : le lot `authorise_delete`
  (#0/#3/#18/#23), `ReadCapability::new` « only producer » (#16), `PartialEq` sans controle (#21),
  « 6 rendus du moteur » (#8). Motif commun : **propriete tenue par le CODE mais non epinglee par un
  test** ≠ propriete non gardee — le premier est un trou de couverture, le second un P1. Sur ce round,
  TOUS les « la mutation laisse vert » tombent dans le premier cas.
- **Un P2 brut escalade en P1** : la borne de COPIE (#1/#17), remontee au niveau de #4. Motif : le
  document canonique qui affirme un ABSOLU que le code du meme commit borne deux fois, dans une liste
  qui s'intitule litteralement « Bornes honnetes (ce que M ne ferme PAS) ».
- **Deux P3 avec 1 vote de refutation sur 3** : la row M du kickoff (artefact DATE hors diff, objet =
  scope cut arbitre PO) et le derive `Clone` (r4 l'avait deja porte au credit du design). Conserves en
  P3 par majorite, avec le vote dissident trace dans leur ligne.
- **Douze des 28 index sont des doublons** repartis en 6 clusters (garde `authorise_delete` ×4, borne
  de COPIE ×3, borne 15 ×2, `Display` de `DeleteRefusal` ×2, digest non reconstructible ×2, renvois
  §19.4 ×2). **19 defauts distincts**, pas 28.
- **Deux points de r4 CLOS ici sans defaut** : la premisse de l'item 2 (« non-regression des 5
  surfaces ») est **fausse** — le diff montre les 5 entrees en contexte pur, rien n'a ete edite ; et
  l'item 3 (visibilite des registrations) n'est pas un defaut — `pub` est obligatoire (consommation
  cross-crate) et conforme aux 41 `pub mod` du crate.
- **Non rouvert** (refute ou traite en r1-r4) : borne honnete d'`ArtifactMode::ALL`,
  `from_utf8_lossy`, `authorise_egress` sur etiquette forgee, absence d'appelant de production (DS-1),
  scope cuts `ArtifactOutput.v1`/lineage/partage selectif, `pub mod artifact_seal` conforme aux pairs
  crypto.

## Etat de l'arbre

**PROPRE — verifie par moi a la redaction, pas rapporte.**

- `git -C C:/Users/FlowUP/Documents/Code/nexus status --porcelain` ⇒ **exactement les 15 entrees
  stagees** attendues (4 `A ` rounds r1-r4, 3 `A ` fichiers de code neufs, 8 `M `), plus ce fichier r5.
- `git diff` ⇒ **VIDE**.
- `git worktree list` ⇒ le depot principal (`17a2b21b [master]`), les 2 worktrees **permanents**
  `.claude/worktrees/agent-a97fde30` et `dazzling-cannon`, et `nexus-098-flip` (detached).
  **Aucun worktree de mesure ne survit** — `dc5`, `ne5wt`, `r5reg`, `r5sec`, `r5sec2` ont tous ete
  retires par leurs dimensions respectives. Le P1 de PROCESS ne se declenche pas.
- **Aucun code de production modifie, aucun correctif applique** : les 19 defauts sont DECRITS. Toutes
  les mutations ont vecu dans des worktrees jetables (recette 3 lignes, `status --porcelain` du
  worktree prouvant les 7 fichiers de phase, `grep -c` de controle avant chaque run, restauration par
  `git show :<chemin>` entre mutations). Les 2 sondes d'integration jetables et le repertoire de blobs
  extraits du scratchpad sont supprimes. Aucun `zz_*` / `*probe*` / `*.bak`, aucun untracked sous
  `crates/` ou `web/src`.

**Delta de tests EXACT pour le body du commit : `+34 (2429 -> 2463)`.** Recompte a la source sur les
blobs de l'INDEX (`git show :<f> | grep -c '#\[test\]'`), jamais recopie : `artifact_acl` **19** +
`artifact_seal` **11** + `artifact_sealer` **4** = **34**. Les trois commandes de gate de la row 26
rendent exactement ces trois nombres (19/19, 11/11, 4/4 ; skipped 160, 506, 486), exit 0 chacune.
L'egalite STRICTE somme-modules == delta-workspace (34 == 34) corrobore independamment et exclut tout
test ajoute hors des trois modules. Le bras rouge du canari ajoute par r4 dans `secret.rs` n'ajoute
**aucune** fonction de test (`git diff --cached -- secret.rs | grep -c '^+.*#\[test\]'` = 0) : il
enrichit `canary_absent_everywhere`. **Doctests** : 3 `compile_fail` (`artifact_acl.rs:413`, `:501`,
`secret.rs:147`) + 3 jumeaux qui COMPILENT (`:429`, `:509`, `secret.rs:155`) + 1 `ignore` legitime
(`artifact_acl.rs:293`, prose illustrant le bug ferme). **0 dep neuve, 0 bump wire, census DOMAIN 29.**

## Ce qui reste pour Codex

Le critere d'arret **n'est pas atteint**. Codex ne doit PAS etre lance avant que le geste P1 soit joue.

**BLOQUANT — a jouer avant Codex (1 P1, une seule row + deux lignes voisines) :**

1. Borne de **COPIE** dans `THREAT_MODEL.md` : incise dans la row `:2369`, **10e borne** en §19.5,
   reprise dans A16 `:74`. C'est le seul geste qui debloque le round.

**RECOMMANDE dans le meme commit (le lot P2, ~30 lignes au total, et il rend le round 6 CLEAN) :**

2. 3 lignes de test direct sur `ArtifactPolicy::authorise_delete` dans
   `an_edited_reference_loses_its_provenance` + reecriture de `THREAT_MODEL.md:2371` en « retirer les
   **DEUX** gardes ». **RE-MESURER apres**, sinon r4 se rejoue.
3. 4 lignes dans `published_artefact_refuses_deletion` fermant le 4e bras (`DeletionRefused`).
4. Une assertion `PartialEq` differente par la seule origine (correctif deja verifie discriminant).
5. Une phrase sur `artifact_acl.rs:521` (`ReadCapability::new` n'est pas une preuve d'origine).
6. Les 4 corrections documentaires : borne 15 `:2066`, clause « module de lib » de la row 26 + son
   miroir `plan:1070`, branche de sortie du carry `DOCTEST-ERRCODE-NOT-ENFORCED`, deux phrases sur
   `ArtifactDigest` non reconstructible + « additive wire en EMISSION ».

**A trancher par une ligne de decision ecrite (pas des defauts, mais irreversibles apres la reliure) :**

7. La borne de **PANIQUE** de `delete` (§19.5) — meme section que le geste P1, cout marginal nul.
8. La visibilite de `pub mod artifact_seal;` — reponse etablie (obligatoire, conforme aux 41 `pub mod`
   du crate), a **ecrire** en §M.2 ou en borne.
9. `DeleteRefusal` a ses deux champs `pub` et n'est pas `#[non_exhaustive]` (`:628-636`) : constructible
   de l'exterieur. Sans consequence aujourd'hui (le type ne circule que sortant, il n'est pas une
   preuve d'origine) — mais c'est **exactement la forme d'ouverture qui a coute les rounds 3 et 4 sur
   `ArtifactRef`**. Le moment le moins cher pour decider est maintenant. Une phrase, mais **prise**,
   pas heritee du derive.

**Angles a NE PAS rejouer** (fermes ce round, mesures) : non-regression des 5 surfaces du canari
(premisse fausse — rien n'a ete edite) ; `publish`/`read` apres le passage des champs en prive
(fermeture etablie par les accesseurs `:450-477` et par les 3 `compile_fail` + jumeaux, tous mesures
discriminants) ; visibilite des registrations (pas un defaut, une decision a ecrire).

**Le doctest `ignore` de `:293-296` reste a verifier** (r4 P3 #37, non corrige, non re-detecte ce
round) : il lie `let (mut reference, _cap) = vault.publish(Published, Keep, bytes)?;` puis passe
`Some(cap)` — le nom `cap` n'existe pas, et `publish(Published, …)` rend `None` en seconde position.
La fence `ignore` l'empeche de rancir seul. P3, mais c'est l'exemple qui DOCUMENTE le bug historique
le plus grave du module.

**Les P3 restants sont documentes ci-dessus et peuvent partir tels quels** — ils sont tous
cosmetiques (deux mots, un renvoi de section, une etiquette d'amendement) et aucun n'affirme un etat
faux.
