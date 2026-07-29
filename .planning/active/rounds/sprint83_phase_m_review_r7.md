# Review — Sprint 83 Phase M (artefacts, ACL et confidentialite) — round 7

## Verdict

**PASS-PENDING** — 0 P1.

Le critere d'arret du depot, en toutes lettres : **« CLEAN ou P2/P3 documentes
uniquement »**. Il est **ATTEINT**. Aucun des 25 findings retenus ne porte un P1 ;
aucun ne decrit une propriete de securite absente, une affirmation fausse dans un
document de gate qui fasse prendre une decision erronee, ni un controle promis et
non livre. La boucle de review s'arrete ici — non par plafond de rounds (il n'y en
a pas), mais parce que le corpus a cesse de produire des defauts bloquants et que
les residus sont ecrits, ancres et chiffres.

**Trajectoire des P1 : 5 → 6 → 2 → 4 → 1 → 1 → 0.** Ce que la courbe indique, lu
honnetement : la bosse r2 (6) est une bosse de COUVERTURE — le round elargissait les
dimensions, pas le defaut ; la remontee r4 (4) est une bosse de METHODE — r3 avait
corrige des instances et r4 a decouvert la CLASSE (« absolus de crypto-shredding
enonces sans leur borne de copie »), ce qui est exactement le geste
« fermer la classe, pas l'instance ». Les deux plateaux a 1 (r5, r6) sont le
symptome inverse : un correctif documentaire joue instance par instance qui laissait
a chaque fois un membre residuel. Le round 7 tranche ce plateau par une MESURE et non
par un jugement : j'ai rejoue le balayage de classe independamment, d'abord avec la
regex de r6 puis avec un regime elargi de 12 motifs (`guarantee that matters`,
`la garantie qui compte`, `irrecuperable`, `unrecoverable`, `crypto-shred by
dropping`, absents du census r6) sur les 17 blobs de l'INDEX. Resultat : **aucun
membre supplementaire de la famille n'existe**. Les residus nommes par les findings
#7, #8 et #11 sont l'inventaire complet. Le plateau n'etait donc pas un plateau de
decouverte — c'etait deux P2 deja arbitres dont le correctif n'a pas ete joue, plus
un site (`:2069-2076`) que personne n'avait nomme et dont la formulation
parenthetique « (destroying the key) » le borne d'elle-meme. Le code, lui, a
converge : `from_hex` est le seul livrable de code du round 6 et il a passe quatre
mutations independantes discriminantes.

**PASS-PENDING** et non PASS sec : les 21 defauts distincts sont de la prose (plus
une troncature d'une ligne, #1) ; ils partent avec le commit ou juste avant, au
choix du correcteur, mais ils ne sont pas encore appliques.

---

## Perimetre lu

INDEX de la phase : **17 entrees stagees**, `git diff` VIDE (controle par marqueur
`[END diff]`, sortie `wc -c` = 0). Six rapports de round `A ` (r1..r6), trois
fichiers de code `A ` (`artifact_seal.rs`, `artifact_sealer.rs`, `artifact_acl.rs`),
huit `M ` (`lib.rs` ×2, `main.rs`, `secret.rs`, `sprint83_plan.md`,
`sprint83_gate_matrix.md`, `WORKFLOW_AGENTS_OVERVIEW.md`, `THREAT_MODEL.md`).

Quatre dimensions ont couvert le round :

1. **from-hex** — audit du seul code NEUF du round 6 (`ArtifactDigest::from_hex`,
   `ArtifactError::MalformedDigest`, le test `a_digest_round_trips_through_its_hex_form`,
   le renvoi ajoute au doc de `from_untrusted_wire`), audite comme du code d'origine.
2. **class-sweep** — exhaustivite du balayage des trois familles traitees par
   balayage en Phase M : absolus de destruction (A), compteurs de tests nus (B),
   affirmations sur les codes d'erreur des doctests `compile_fail` (C).
3. **docs-coherence** — coherence des 11 blobs non-rounds entre eux et avec le code.
4. **suites-counts** — dimension mecanique : suites §7.4, delta, deps, wire.

Verifications que j'ai relancees moi-meme a la redaction, sur les blobs de l'INDEX
(aucune mutation, aucun worktree cree par moi) :

- `git status --porcelain` ⇒ 17 entrees, 0 untracked ; `git diff` ⇒ 0 octet.
- Comptes d'attributs de test sur les trois blobs neufs : **21 + 11 + 4 = 36**.
- `git show :docs/security/THREAT_MODEL.md | grep -n from_hex` ⇒ **0 hit**.
- `git grep -n "impl ArtifactSealer for" -- crates/` ⇒ **3**
  (`artifact_sealer.rs:66` production, `artifact_acl.rs:1122` `ScriptedSealer`,
  `secret.rs:655` `XorSealer`).
- `grep -cE "^## Phase " sprint83_plan.md` ⇒ **46**, moins Phase 0 = **45**
  (= 41 plancher + 4 re-coupes A1/I1/L1/M1) — le compte est EXACT.
- `artifact_acl.rs:561` « The host sealer is the only producer. » surplombe
  `pub fn new(key: Vec<u8>)` a `:562` — verifie.
- `THREAT_MODEL.md:2066` « 6 rendus du moteur » et `:2447` « Les deux sites du code
  la portent » — verifies verbatim.

---

## Findings retenus

**25 findings, 0 P1** — 13 P2 et 12 P3 au libelle retenu. Quatre paires portent le
MEME geste ; apres fusion, le lot reel est de **21 defauts distincts (11 P2 +
10 P3)**. Le correcteur doit jouer 21 gestes, pas 25, et surtout ne pas ecrire deux
fois la meme incise dans la borne 2 de §19.5.

Doublons a fusionner : **#4 ≡ #12** (attribution de l'invariant de constructeur) ·
**#6 ≡ #16** (`from_hex` absent de `THREAT_MODEL`) · **#9 ≡ #19** (« les deux sites »
de la borne 10) · **#10 ≡ #18** (branche d'exit du carry `DOCTEST-ERRCODE-NOT-ENFORCED`).

Quatre findings ont ete cotes en desaccord par le jury (P2 chez deux jures, P3 chez
un) : #1, #2 vs #4, #16 et #9. Je retiens le libelle majoritaire et je le signale
en ligne. Aucun de ces desaccords ne franchit le seuil P1.

### Code neuf (`from_hex`)

**#1 [P2] — `MalformedDigest` recopie sans borne le texte fourni par l'appelant.**
Ancre : `crates/sbfb-workflow-engine/src/artifact_acl.rs:297` (`digest: hex.to_string()`,
verifie par moi) et `:646-654` (`#[error("... got {digest:?}")]`).
Mesure : sonde en worktree jetable, entree de 200 000 octets ⇒ `e.to_string()` de
200 059 octets, aucune troncature. `{digest:?}` echappe les sauts de ligne
(`injected_has_raw_newline=false`) : l'injection de log est FERMEE, seule la
LONGUEUR est libre. C'est le seul des 5 payloads `String` d'`ArtifactError` a
recopier du texte non valide — les 4 autres (`Expired`, `MissingCapability`,
`UntrustedReference`, `DigestMismatch`) tirent tous de
`reference.digest.as_str().to_string()`, borne a 64 par l'invariant de type. Le plan
§P.2 met ce constructeur sur le wire compute : l'entree deviendra attaquant-controlee.
Le module porte deja la doctrine inverse a trois lignes (`SealerRefused` :642-644,
« Plaintext-free by contract »).
Correctif : `digest: hex.chars().take(72).collect()` + une phrase au doc de la
variante. **NE PAS ecrire `&hex[..72]`** — le decoupage par index d'octet panique sur
une frontiere de `char`, et la propriete auditee de la phase est « 0 panique
atteignable ». (Jure dissident : P3, echo lineaire, 0 call-site aujourd'hui.)

**#2 [P2] — le chemin de decodage n'est complet que pour le digest.**
Ancre : `:420-423` (« that is the whole decoding path ») ; types `:157-159` et `:230-232`.
Mesure : `ArtifactMode` et `ArtifactRetention` derivent `serde::Serialize` SEUL —
`grep -nE "Deserialize|FromStr|TryFrom|from_label|from_str"` sur le blob ne rend que
des doc-comments, **0 impl**. Un pair tenant le JSON doit ecrire a la main
`match parsed["mode"].as_str()`. Le test neuf le montre malgre lui : il prend `digest`
du payload parse (:1958) mais **re-fournit `ArtifactMode::Local` et
`ArtifactRetention::Keep` en litteraux** (:1964-1965) sous un commentaire
« The path a peer actually walks » (:1949-1950). Et ce chemin echoue OUVERT : un
decodeur avec `_ => ArtifactMode::Published` fait dire oui a `authorise_egress`
(:859-865 via `may_leave_node` :209-213). Le module interdit deja le bras `_` pour ses
propres predicats (:183-186) et ne dit rien pour celui du pair.
Attenuation qui empeche le P1 : le mode d'une reference untrusted est DEJA une claim
attaquant-controlee (borne 2 de §19.5) — un mis-decodage n'ouvre aucune porte
qu'ecrire `"published"` dans le JSON n'ouvrait pas.
Correctif : trois lignes sur `from_untrusted_wire` — `from_hex` decode le DIGEST ;
`mode`/`retention` se rebatissent en matchant les etiquettes epinglees (renvoi
`reference_wire_form_is_pinned`) ; un decodeur d'etiquette doit REFUSER une etiquette
inconnue plutot que defauter. Plus une phrase au commentaire du test :1949-1950.

**#3 [P3] — « refused rather than normalised » est justifie par une raison qui ne
discrimine pas.** Ancre : `:286-288`, `:647-649`, `sprint83_plan.md:1109-1110`.
Mesure : le but enonce (« two references … cannot differ by case and compare
unequal ») est atteint AUSSI BIEN par `hex.to_ascii_lowercase()`. L'enonce est VRAI,
il n'est pas discriminant. Le depot normalise deja cette classe ailleurs
(`seed_registry.rs:146`, `contributor_api.rs:45`). La raison discriminante existe et
est plus forte : le refus rend octets-recus == octets-re-emis, donc une signature
posee sur le JSON recu reste valide sur le JSON re-serialise — et la Phase M1 est
« canonicalisation signee + provenance ».
Correctif : remplacer la clause « so … » aux 3 sites par cette raison. Le choix est
bon, seul son motif est a changer.

**#4 [P2] — mesattribution : `ArtifactDigest` n'invoque aucun argument « pas de
`Deserialize` », et `from_hex` n'est pas « le seul endroit » ou un invariant de
constructeur est applique.** Ancre : `:283-286`, `:1939-1940`,
`sprint83_plan.md:1112-1114` et `:1398-1400`. **Fusionne avec #12** (`:285-287`,
`:1943-1945`).
Mesure : le doc de type d'`ArtifactDigest` est `:263-264`, deux lignes, **zero**
mention de `Deserialize` ni d'invariant de constructeur. Les types qui portent
l'argument sont `ArtifactRef` (:319-321) et `crate::secret::SecretRef`
(`secret.rs:79`). Le plan ecrit « le SEUL endroit ou l'invariant … est reellement
APPLIQUE » : faux d'au moins un site du meme fichier —
`ArtifactRef::from_untrusted_wire` applique le sien (`origin: RefOrigin::UntrustedWire`,
:485) depuis le debut de la phase. Le FOND est vrai et vaut d'etre ecrit tel quel :
`from_hex` est le premier constructeur d'`ArtifactDigest` qui ait quelque chose a
VALIDER, `of_stored_bytes` ne pouvant pas produire un digest invalide.
Correctif : nommer les types aux 4 sites ; supprimer « le seul endroit » du plan.
(Jures dissidents : 2/3 cotent P3 — imprecision d'attribution, pas propriete fausse.)

**#5 [P3] — l'idiome etabli du depot pour « minuscule seulement » n'est pas reutilise,
et la divergence avec les freres qui NORMALISENT n'est pas nommee.** Ancre : `:291-295`
et `:264` (« Lowercase hex, blake3, as everywhere else in the tree »).
Mesure : trois postures coexistent dans le depot. REFUSER —
`nexus-core-rs/src/attestations/contributor.rs:360` et `delegation.rs:274`, tous deux
`.all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())`, exactement la regle de
`from_hex` ecrite autrement. ACCEPTER LES DEUX CASSES — `files.rs:38`, `consent.rs:148`,
`dns_fallback.rs:243`, `deploy.rs:756`, `public_feed.rs:276`, `auth.rs:202`.
NORMALISER — `seed_registry.rs:146`, `contributor_api.rs:45`, `browse_api.rs:105-106`.
`from_hex` choisit la plus stricte, ce qui est le bon appel ; `:264` se lit comme si
l'arbre etait uniforme.
Correctif : reutiliser l'idiome frere, ou remplacer « as everywhere else in the tree »
par un renvoi aux deux sites `attestations/*`. (Un jure refute : `:264` enonce le
FORMAT, pas une uniformite de posture. Retenu a P3 par majorite.)

**#6 [P3] — §19.5 gagne un constructeur public et ne le dit pas.** **Fusionne avec
#16**, qui porte le meme geste a P2. Voir #16.

### Balayage de classe

**#7 [P2] — le balayage de la famille « absolu de crypto-shredding » atterrit encore
a N-1 : les DEUX sites que r6 nommait pour le MEME geste sont restes.**
Ancre : `artifact_acl.rs:1017` · `docs/security/THREAT_MODEL.md:2369`.
Mesure : census rejoue independamment sur les 17 blobs. `:1017` porte verbatim
« `ReadCapability` is deliberately not `Clone`, so the caller had no way to keep a
copy » — l'inference que toute la famille existe pour tuer. Sur `:2369`, SEUL le
sous-correctif (a) est parti (« definitivement illisible » → « illisible pour un
detenteur qui n'a pas recopie la cle … borne 10 ») ; (b) « mais plus personne ne peut
les lire » et (c) « aucun moyen d'en garder copie » sont intacts sur la MEME ligne.
Pourquoi P2 et non P1 : ce sont des reconductions de P2 DEJA arbitrees par le jury r6
sur le critere « borne presente dans la meme unite de lecture » (`:1008-1009` porte
« under the COPY bound … bound 10 of §19.5 » neuf lignes plus haut ; la cellule
`:2369` OUVRE sur la borne 10 avec renvoi explicite). Un P2 non joue n'est pas un
balayage incomplet, et un P2 arbitre ne devient pas P1 parce qu'un round a passe.
Correctif : `:1017` → « had no SUPPORTED way to keep a copy » ; `:2369` → « plus aucun
TIERS ne peut les lire » et « aucun moyen SUPPORTE d'en garder copie ». Trois mots,
zero code, zero test.

**#8 [P2] — un membre de la famille que personne n'a jamais nomme, le seul dont
l'unite de lecture ne porte ni borne ni renvoi.** Ancre : `artifact_acl.rs:2069-2076`
(doc-comment de `deleting_an_artefact_whose_blob_vanished_returns_the_capability`).
Mesure : `:2070` « so the caller can still crypto-shred by dropping it » et `:2076`
« the guarantee that matters (destroying the key) stays available to the caller, only
the tag survives ». Grep des six marqueurs de borne (`to_vec`, `COPY bound`,
`copied the key out`, `borne 10`, `§19.5`, `expose`) sur `:2060-2085` ⇒ **0 hit**.
Jamais signale : `grep -rn "blob vanished|crypto-shred by dropping"
.planning/active/rounds/` ⇒ 0. Son jumeau francais `THREAT:2436-2437` (borne 9) est,
lui, immediatement suivi de la borne 10.
Pourquoi P2 et non P1, dit explicitement pour que le main thread puisse trancher a la
hausse : c'est un doc-comment de TEST sur le chemin de REFUS ; la garantie y est
parenthetiquement redefinie comme « destroying the key », ce qui est VRAI sans borne
(dropper SA capability detruit bien SA cle) ; et le module porte desormais la borne
sur TROIS sites de production. Ce n'est pas la qu'un implementeur apprend le contrat
de suppression.
Correctif : `:2076` → « … stays available to the caller — sous la borne 10 : pas
contre une copie prise a l'avance —, only the tag survives ».

**#9 [P2] — le correctif de ce round a rendu FAUX un compte de `THREAT_MODEL`, de sa
propre main.** Ancre : `docs/security/THREAT_MODEL.md:2447` (verifie verbatim par moi).
**Fusionne avec #19.**
Mesure : la borne 10 ecrit « Les deux sites du code la portent (`artifact_acl.rs`, doc
de `delete` et de `expose`) ». Le correctif de r6 a ajoute un TROISIEME site — l'en-tete
de module. Mesure : `grep -nE "to_vec|COPY bound|copied the key out"` sur le blob ⇒
`:75`+`:83-85` (module), `:573` (`expose`), `:985` et `:1009` (`delete`) = **3 docs**.
C'est exactement le defaut vise par la regle 1 du § « Pour le CORRECTEUR » : le
correctif entre sans sa mesure, et le grep qui aurait prouve la phrase la refute.
Direction conservatrice (sous-compte) ⇒ pas P1 ; precedent interne a la phase : r6 P2-9,
compte 5→6, cote P2.
Correctif : « Les TROIS sites du code la portent (`artifact_acl.rs` : en-tete de module,
doc de `delete`, doc de `expose`) ». Un mot.

**#10 [P2] — le carry `DOCTEST-ERRCODE-NOT-ENFORCED` garde une branche d'exit qui
permet de le clore sur un enonce mesure faux.** Ancre :
`.planning/active/sprint83_plan.md:1121-1122` ; cible `crates/nexus-core-rs/src/vault.rs:195-197`.
**Fusionne avec #18.**
Mesure : le plan ecrit « exit = jumeau pose sur `vault.rs` **ou decision ecrite** que
le doctest seul suffit la-bas ». Or `vault.rs:195-197` affirme « the error code is
fixed so the test cannot go green on an unrelated compilation error » — l'inverse exact
de ce que le MEME commit publie a `THREAT:1975` : « le code d'erreur annote DOCUMENTE
la raison mais ne l'enforce PAS — mesure Phase M sur rustc 1.95.0, editions 2021 ET
2024 ». Une decision ecrite ne rend pas vrai un enonce mesure faux. Owner/trigger/exit
sont tous les trois presents (README §11 satisfait) : c'est la seconde BRANCHE qui est
le defaut. Reconduction r5/r6 jamais appliquee ; `vault.rs` est hors du diff de phase.
Correctif : « exit = jumeau pose sur `vault.rs` **ET** phrase de `vault.rs:195-197`
corrigee ». Un mot : « ou » → « ET ».

**#11 [P3] — l'absolu nu survit entre ses deux bornes, au site meme que ce round a
corrige.** Ancre : `artifact_acl.rs:79-80` et jumeau `:1022-1023`.
Mesure : le correctif a pose la borne a `:75` (« unreadable TO ANYONE WHO DID NOT COPY
THE KEY OUT OF ITS CAPABILITY ») et le paragraphe qualificatif a `:82-88`, mais a
laisse intacte, ENTRE les deux, la clause « but nobody can read them again, which is
the guarantee that matters ». Unite de lecture bornee DEUX fois (avant et apres) ⇒ P3.
Meme cran a `:1022-1023`, sous la borne de `:1008-1009`.
Correctif : « but nobody **else** can read them again » ; « which is the guarantee that
matters, **sous la meme borne** ». A jouer dans le geste de #7 et #8.

**#12 [P3] — `from_hex` attribue a `ArtifactDigest` un invariant que ce type n'invoque
nulle part.** **Doublon de #4** — meme geste, deux ancres de plus (`:285-287`,
`:1943-1945`). Voir #4.

**#13 [P3] — compte de `compile_fail` faux d'une unite dans un rapport de round stage.**
Ancre : `.planning/active/rounds/sprint83_phase_m_review_r6.md:299` (« Workspace :
6 `compile_fail`, 0 echec »).
Mesure : `grep -rn '```compile_fail' crates/ --include="*.rs"` ⇒ **5** blocs
(`nexus-core-rs/src/vault.rs:199`, `sbfb-workflow-core/src/accessibility_profile.rs:333`,
`sbfb-workflow-engine/src/artifact_acl.rs:453` et `:541`,
`sbfb-workflow-engine/src/secret.rs:147`). Le « +5 doctests » de la meme puce est,
lui, EXACT. Rapport de round = trace de mesure, pas document de gate ⇒ P3.
Correctif : « 5 », ou retirer le chiffre — ce que la regle de `artifact_seal.rs:431-433`
recommanderait.

### Coherence documentaire

**#14 [P2] — row 26 : « le filtre positionnel exige un module de lib » est falsifie par
la 3e commande de la row elle-meme. 4e reconduction.** Ancre :
`.planning/active/sprint83_gate_matrix.md:60` (seule ligne modifiee du fichier,
`git diff --cached -U0` ⇒ `@@ -60 +60 @@`).
Mesure LANCEE : `cargo nextest run -p nexus-shell-daemon artifact_sealer --locked` ⇒
4 passed, exit 0, binary id `nexus-shell-daemon::bin/nexus-shell-daemon` — un
BINAIRE ; `crates/nexus-shell-daemon/src/lib.rs` n'existe pas. La clause est fausse
pour un tiers exact du livrable que la row certifie. Le RESTE de la row est vrai et
re-mesure : 21 + 11 + 4 = 36, exit 0 chacune ; la claim d'exit 4 est exacte aussi
(`-p sbfb-workflow-engine artifact_seal` ⇒ `0 tests run: 0 passed, 181 skipped`,
`error: no tests to run`, exit 4).
C'est le finding ou le calibrage litteral (« affirmation fausse dans un document de
gate » = P1) pourrait mordre. Je ne l'escalade PAS : r4, r5 et r6 ont arbitre le meme
fait a P2 trois fois, sur le motif tenu que la CONCLUSION de la row est VRAIE ; la
clause fautive est une note de mecanisme nextest, qui ne fait prendre aucune decision
de gate erronee. Rejuger a la hausse un cran arbitre trois fois serait exactement
l'escalade que ce round doit refuser.
Correctif : « il matche le CHEMIN DE MODULE — lib **ou binaire** (mesure :
`nexus-shell-daemon::bin/nexus-shell-daemon`, ce crate n'a pas de `lib.rs`) ; un
fichier `tests/*.rs` ne matcherait pas, son nom devenant l'ID du binaire ».
**NE PAS toucher `sprint83_plan.md:1069`**, ou l'occurrence porte sur `artifact_acl.rs`
dans un crate qui A une lib — elle est VRAIE.

**#15 [P2] — « The host sealer is the only producer » survit sur `pub fn new` — la
premisse que la borne 10 de §19.5, ecrite par ce commit, contredit.**
Ancre : `crates/sbfb-workflow-engine/src/artifact_acl.rs:561`, verifie par moi, juste
au-dessus de `pub fn new(key: Vec<u8>)` (:562), type re-exporte racine
(`lib.rs:203`). *Note d'ancre : une section de r7 ecrivait `:566` ; l'ancre exacte est
`:561`, re-greppee.*
Mesure : census de la famille « unicite de producteur/implementeur » sur les 11 blobs
non-rounds ⇒ `:561` est le SEUL enonce FAUX du lot. Le meme fichier ecrit l'inverse
trois fois : `:571-576` (« a caller who did that **can rebuild a working capability
with `ReadCapability::new`** »), `:983-986`, module-doc `:82-88`. Et `THREAT:2439-2441`
(borne 10) repose dessus : « `ReadCapability::new` **est publique** ».
Pourquoi P2 et non P1 : aucun controle ne casse — `delete` unseal-verifie la
capability, donc une capability forgee ne detruit rien ; et « producer » se lit en
contexte comme producteur de MATERIEL DE CLE fraiche, lecture sous laquelle la phrase
n'est pas fausse. Arbitrages r5 et r6 identiques.
Correctif : « Wrap freshly-minted key material. The host sealer is the INTENDED
producer; this constructor is `pub`, so a caller who copied key bytes out of
[`ReadCapability::expose`] can rebuild one — bound 10 of §19.5. » Une ligne.
**Si le correcteur la saute une quatrieme fois, le plateau cesse d'etre imputable aux
reviewers.**

**#16 [P2] — `ArtifactDigest::from_hex`, seul livrable de CODE ajoute au round 6,
n'apparait nulle part dans `THREAT_MODEL.md`.** Ancre : `artifact_acl.rs:289`
(`pub fn from_hex`) ; decrit au plan `:1108-1114` (§M.2) et `:1399-1404` (§P.2) ;
absent de `docs/security/THREAT_MODEL.md`. **Absorbe #6.**
Mesure : `git show :docs/security/THREAT_MODEL.md | grep -c from_hex` ⇒ **0**
(relance par moi). Or §19.5 nomme tous les autres constructeurs de la famille :
`ArtifactRef::from_untrusted_wire` (row `T-ARTIFACT-FORGED-REF-DESTRUCTIVE`, `:2371`)
et `ReadCapability::new` (borne 10). La borne 2 (`:2380-2385`) est exactement
l'endroit qui enumere ce qui reste ouvert sur la CONSTRUCTION d'une reference — et
elle ignore le maillon qui la rend praticable pour un pair ne tenant que le JSON.
Rien ne devient FAUX (ni la borne 2 ni A16 ne qualifient l'origine du digest ;
`from_untrusted_wire` reste le seul constructeur public d'`ArtifactRef`, verifie) ⇒
borne manquante, P2. Le contrat du plan est exact au mot pres : refus hors-64, hors
`[0-9a-f]`, MAJUSCULE refusee sans normalisation (`:291-298`), `MalformedDigest`
(`:295`), test = chemin reel du pair + **6** bras de refus (`""`, `"abc"`, 63, 65,
upper, non-hex).
Correctif : (a) une clause dans la borne 2 de §19.5 ; (b) une ligne dans le changelog
v29, qui se presente lui-meme comme ce qu'un auditeur grep en premier.
(Un jure cote P3 : omission de completude sans consequence de lecture.)

**#17 [P2] — borne 15 de §19.4 : « 6 rendus du moteur » la ou la row `T-SECRET-REF-EXFIL`
du MEME fichier ecrit « les CINQ PREMIERES … la 6e est le MODELE ».**
Ancre : `docs/security/THREAT_MODEL.md:2066` contre `:1975` (les deux dans le diff
staged, `@@ -1974 +1975 @@` et `@@ -2065 +2066,3 @@`). Les deux lignes verifiees
verbatim par moi.
Mesure : `git show HEAD:` ⇒ « Le canari couvre **5** rendus du moteur + le blob au
repos. » ; blob de l'INDEX ⇒ « **6** rendus du moteur + le blob au repos (la 6e
surface, `artifact_public_store`, est ajoutee par la Phase M — cf. §19.5) ». Source :
`secret.rs:721-727`, tableau `surfaces` a 6 entrees dont la 6e est
`artifact_public_store` — un MODELE de magasin de blobs, pas un rendu du moteur
(`git grep "impl ArtifactBlobs for"` ⇒ 3, toutes sous `mod tests`). Le COMPTE est bon,
le NOM generique est faux pour la 6e — et l'ecart va dans le sens qui SURVEND la
couverture. La phase a transforme un enonce exact de HEAD en enonce inexact. La
parenthese ajoutee situe la 6e sans corriger le substantif.
Correctif : « Le canari couvre **5 rendus du moteur + le magasin d'artefacts**
(MODELE, cf. §19.5 bornes 1 et 6) **+ le blob au repos**. » Ne pas toucher au 6 de
`:1975`, qui designe le sweep et est exact.

**#18 [P2] — branche d'exit du carry `DOCTEST-ERRCODE-NOT-ENFORCED`.** **Doublon de
#10.** Voir #10.

**#19 [P2] — borne 10 de §19.5 enumere « les deux sites » alors qu'il y en a trois.**
**Doublon de #9.** Voir #9.

**#20 [P3] — « exactly one implementer » / borne 7 « un seul implementeur » — il y en
a trois.** Ancre : `crates/sbfb-workflow-engine/src/artifact_acl.rs:730` (EXACTE,
verifiee : « is exactly one implementer (`nexus-shell-daemon`, real AES-256-GCM, ») et
`docs/security/THREAT_MODEL.md` borne 7. **Ancre THREAT RECALEE par moi : le finding
citait `:2437`, c'est FAUX — la borne 7 est a `:2411-2419` (« Rien n'oblige un
implementeur de `ArtifactSealer::seal` a sceller » … « implementeur, reel
(AES-256-GCM) et teste bout-en-bout. »).** Je ne retire pas le finding pour ce seul
decalage : la citation verbatim et l'etiquette « borne 7 » la rendent localisable au
grep, et le fait tient.
Mesure : `git grep -n "impl ArtifactSealer for" -- crates/` ⇒ **3** —
`nexus-shell-daemon/src/artifact_sealer.rs:66` (production), `artifact_acl.rs:1122`
(`ScriptedSealer`, test), `secret.rs:655` (`XorSealer`, test). Les phrases JUMELLES du
meme commit portent le mot juste (« de **production** », borne 1 `:2431-2433`).
Severite basse justifiee : les deux enonces sont immediatement qualifies par la
parenthese qui NOMME l'implementeur de production. Reconduction r6 P3 n°2.
Correctif : un mot par site — « exactly one PRODUCTION implementer » / « un seul
implementeur DE PRODUCTION ».

**#21 [P3] — en-tete de la Phase M etiquetee « (pre-coupe AM-4) » alors que §R2 range
M1 en AM-7 et n'enumere pas M dans AM-4.** Ancre : `sprint83_plan.md:1023` contre
`:2578`.
Mesure : §R2 ecrit « AM-4 pre-coupes (**AI×3, AJ×2, P, L**) + AM-7 (re-coupes
nominales) » — M n'y figure pas ; M1 est dans AM-7. Le compte lui-meme est EXACT et
re-mesure par moi : `grep -cE "^## Phase "` ⇒ **46**, moins Phase 0 = **45** = 41
plancher + 4 re-coupes. Seule l'etiquette derive. Reconduction r6 P3 n°4.
Correctif : aligner sur le patron `I1` — « ## Phase M — Artefacts, ACL et
confidentialite (re-coupe AM-7, preflight PLAN-ADAPT) ».

**#22 [P3] — §M.4 ne lance qu'une des trois commandes que la row 26 declare
indispensables.** Ancre : `sprint83_plan.md:1183-1188` contre
`sprint83_gate_matrix.md:60`.
Mesure : §M.4 nomme `cargo nextest run -p sbfb-workflow-engine artifact_acl --locked`,
le run crate entier, et `check-frontier-contracts.sh`. Les deux autres filtres sont
absents — donc `local_artefact_round_trips_end_to_end_under_real_crypto`, que la row 26
designe comme LE test sous crypto REELLE, n'est le critere d'acceptation d'aucune ligne
du plan (il vit dans `nexus-shell-daemon::bin/nexus-shell-daemon`, vu dans ma sortie
nextest). La claim de §M.4 sur l'avant-phase est EXACTE (`0 tests run, 160 skipped` —
recoupe : 181 dans le crate, 181 − 21 = 160). Aucun scenario ou le livrable passe non
verifie : le run `--workspace` de §7.4 le couvre. Reconduction r6 P3 n°5.
Correctif : deux lignes copiees de la row 26.

**#23 [P3] — renvois nus « borne 25 » et « borne 16 » a l'interieur d'une liste §19.5
numerotee 1..10 continue.** Ancre : `docs/security/THREAT_MODEL.md:2400` (borne 4) et
`:2407` (borne 5).
Mesure : numerotation de §19.5 re-comptee a la source — 1..10, **continue, sans trou ni
doublon**, chacune verifiee vraie du code stage (borne 8 :
`grep -c "pub async fn" crates/nexus-core-rs/src/blobs.rs` ⇒ **9**, `new` etant la 10e
et non-async ✓ ; borne 4 : aucune API artefact dans `vault.rs`/`keystore.rs` ✓). Les
deux renvois designent des bornes de §19.4 ; un lecteur cherche un item 25 dans une
liste qui s'arrete a 10. Reconduction r6 P3 n°7.
Correctif : deux prefixes — « borne **25 de §19.4** », « borne **16 de §19.4** ».

**#24 [P3] — bande d'estimation de tests L/L1/M a `+12..20` contre un reel de +81.**
Ancre : `sprint83_plan.md:2493`.
Mesure : `| L / L1 / M | +12..20 | 0 | 0 | …` ; reel L1 = +45, M = +36 (recompte par
moi : 21 + 11 + 4 attributs de test sur les trois blobs de l'INDEX, corrobore par les
trois commandes de la row 26). La table est etiquetee « ordre de grandeur » et n'est
le critere d'aucun gate ; le compte qui ENGAGE (+36) est exact et mesure.
Reconduction r6 P3 n°9. (Un jure refute : une estimation ex ante fausse a posteriori
n'est pas un defaut. Retenu a P3 par majorite, une ligne de recalage suffit.)
Correctif : recaler, ou noter « bande perimee, cf. §Delta des commits L1/M ».

**#25 [P3] — le helper ignore `effects_child_writes_cwd_marker` salit l'arbre quand la
suite est relancee avec `--run-ignored`.** Ancre :
`crates/sbfb-workflow-engine/tests/effects.rs:2744`
(`std::fs::write("cwd_marker.txt", b"here")` dans le corps de l'ENFANT, tandis que le
parent lit sous un `TempDir`, `:2788`) ; residu observe :
`crates/sbfb-workflow-engine/cwd_marker.txt`.
Mesure : `cargo nextest run --workspace --locked --run-ignored ignored-only` (lance
pour identifier les 7 skips) laisse le fichier non suivi — `git status --porcelain`
passe de 17 a 18 entrees avec une ligne `??`. Le run nominal ne le declenche PAS.
Pre-existant Phase K, hors Phase M, sans effet sur le +36 (fichier supprime, arbre
re-verifie a 17 / 0 untracked, `git diff` vide).
Correctif : ecrire le marqueur dans un `TempDir` passe en variable d'environnement,
ou `.gitignore`. **A router vers la phase proprietaire du harness d'effets (K), pas
vers M.**

### Un defaut qu'aucun des 25 findings ne porte, releve a la redaction

**[P3, non compte dans les 25] — le bras « majuscule » du test neuf n'est
discriminant que sous une premisse non assertee.** `artifact_acl.rs:1969` fait
`let upper = digest.as_str().to_uppercase();`. Ce bras n'est discriminant que SI le
digest concret de la charge de test contient au moins un caractere `a`-`f` : sur un
digest hypothetiquement tout-numerique, `to_uppercase()` rendrait la chaine inchangee
et le bras deviendrait un doublon vacueux du cas nominal — vert quelle que soit la
regle de casse. Les bras 63/65 (`"a".repeat(..)`) ne souffrent pas de ce defaut.
Meme classe que « muter avant de croire un test vert » : un bras present n'est pas un
bras discriminant. Correctif d'une ligne : batir le cas depuis une constante hex
litterale contenant explicitement une lettre, ou asserter `upper != digest.as_str()`
avant. **La mutation M-C du round (acceptation de la majuscule) a bien fait ROUGIR ce
test**, donc la premisse est vraie AUJOURD'HUI — ce qui n'est pas asserte, c'est
qu'elle le reste.

---

## Ce qui tient

**Le code neuf `from_hex` — VALIDATION CORRECTE ET COMPLETE, discriminante sur CHAQUE
bras.** Worktree jetable, code de phase porte par `git diff HEAD -- crates/ |
git apply --index` (controle `status --porcelain` = les 7 fichiers de phase, donc
mesure NON vacueuse), baseline `cargo nextest run -p sbfb-workflow-engine --locked`
= **174 passed, 7 skipped** (= 173 de r6 + le test neuf). **Quatre mutations
independantes**, chacune controlee par `grep -c` AVANT le run, chacune ⇒ **1 FAIL sur
`a_digest_round_trips_through_its_hex_form`** : M-A retrait de la clause de longueur
(`hex.len() != BLAKE3_HEX_LEN`) ; M-B neutralisation de l'alphabet (`.all(|_b| true)`) ;
M-C acceptation de la majuscule (`b.is_ascii_hexdigit()`) ; M-E elargissement
`b'a'..=b'f'` → `b'a'..=b'z'`. Longueur, alphabet, casse et borne haute de l'alphabet
sont pinnes **SEPAREMENT**.

**Octets vs caracteres — pas de faille.** `hex.len()` est en OCTETS et `hex.bytes()`
itere des OCTETS : le couplage est SUR parce que tout octet ≥ 0x80 echoue
`(b'a'..=b'f')`. Mesure par sonde : 32×U+00E9 (64 octets, 32 chars) refuse ; 64×U+FF10
(192 octets) refuse ; 63×'a'+NUL refuse ; 62×'a'+U+00E9 (64 octets, 63 chars) refuse ;
64 octets espace-paddes refuse. Aucun `char` multi-octet ne passe. Aucune indexation,
aucun `unwrap`/`expect`/tranche ⇒ **0 panique atteignable ajoutee**.

**Pas de route de contournement — tranche.** `from_hex` rend la forgerie strictement
PLUS FACILE (une chaine hex suffit) mais n'accorde AUCUN acces neuf :
`ArtifactBlobs::get_unauthenticated` « takes no principal and no capability, on
purpose » (:764-766, :797-798) — quiconque tient le hex obtient deja le chiffre SANS
aucune `ArtifactRef`. Les quatre portes traitent la reference reconstruite comme
`UntrustedWire` : `delete` (:1038) delegue a `authorise_delete` qui refuse la
provenance en PREMIER (:842-846) ; `read` (:953-979) exige une capability qui OUVRE ;
`authorise_read` n'applique que la retention ; `authorise_egress` decide sur
l'etiquette — la seule porte ouverte, et la borne 2 de §19.5 la declare deja. Aucune
claim du threat model falsifiee. **Ce n'est PAS une regression de securite.**

**Invariant de type tenu a l'echelle du crate.** `grep -rn "ArtifactDigest("` sur
`crates/` ⇒ la declaration seule ; deux sites de construction dans le module, `:271`
`of_stored_bytes` (toujours minuscule via `blake3::Hash::to_hex`) et `:300` `from_hex`
(valide) ; champ tuple prive, aucune ecriture `.0`. « 64 hex minuscules » vaut donc
pour toute valeur du type. `MalformedDigest` est une VARIANTE, pas un item : les 11
re-exports racine (`lib.rs:203-206`) restent exactement les 11 items publics du module.

**L'item ouvert de r5/r6 est CLOS.** « `ArtifactDigest` n'a aucun constructeur depuis
sa forme serialisee » (statut « INCONNU a la cloture » en r6) est ferme par ce code. Le
plan NOMME le test neuf (`sprint83_plan.md:1160`) — la classe de r6 P2-8 ne se rouvre pas.

**Balayage de classe — MATERIELLEMENT COMPLET, verifie par deux regimes de grep
independants.** Regime r6 verbatim ⇒ **4 hits, exactement les 4 de r6** :
`artifact_acl.rs:79`, `:1017`, `:2033`, `THREAT:2369` — aucun site NEUF n'est apparu
apres le correctif. Regime elargi (12 motifs) ⇒ 7 sites de production :
`:79-80`, `:1017`, `:1022-1023`, `:2033`, `:2070`, `:2075`, `THREAT:2369`. `:2033`
est celui que r6 a explicitement defendu (« a consumed capability would be gone for
good » — aucune copie dans son scenario) et **ne doit pas etre rouvert**. Les six
autres sont couverts par #7, #8 et #11. **Aucun membre supplementaire n'existe.**
Le P1 de r6 (en-tete de module) est ferme et VERIFIE ferme : `:73-75` porte
« unreadable TO ANYONE WHO DID NOT COPY THE KEY OUT OF ITS CAPABILITY », `:82-88`
l'explicite (« That qualifier is not pedantry ») avec renvoi nomme a
`ReadCapability::expose`, a `ReadCapability::new` et a la borne 10.

**Classe B (compteurs de tests nus) — CLOSE.** Census sur les 7 blobs de code : 4
compteurs, tous attaches a une revision explicitement perimee (`artifact_acl.rs:1034`
« An earlier revision … left 19/19 green », `:1088-1089` « the four-class version »,
`artifact_sealer.rs:168` « Measured on the previous revision »). **ZERO compteur au
present.** La regle qui les gouverne (`artifact_seal.rs:431-433`, « Test counts are
deliberately not quoted anywhere in this module ») est VRAIE de son module (0 hit hors
la regle). Aucun document stage ne recopie « 35 », « 36 » ni « +35 » (grep sur les 11
fichiers non-rounds ⇒ 0), et la row 26 dit explicitement « recomptes a la source ».

**Classe C (codes d'erreur des `compile_fail`) — CLOSE cote code, et MESUREE.**
`artifact_acl.rs:462-464` affirme « a `compile_fail,E0616` block whose real error is
`E0308` or `E0432` still passes » — couple que ni `THREAT:1975` ni le plan ne
mesuraient. Mutation en worktree jetable (corps du bloc E0616 remplace par
`let _x: u8 = "MUTATION-E0308";`, `grep -c` = 1 avant le run, perl jamais python) ⇒
`cargo test -p sbfb-workflow-engine --locked --doc` rend
`…from_untrusted_wire (line 453) - compile fail ... ok` et `test result: ok. 3 passed;
0 failed`. **L'affirmation est VRAIE et desormais mesuree, pas seulement citee.** Le
meme run prouve l'appariement 3 `compile_fail` ↔ 3 jumeaux qui COMPILENT
(`:453`↔`:469`, `:541`↔`:549`, `secret.rs:147`↔`:155`).

**THREAT_MODEL §19.5** : bornes **1..10 continues, 0 trou, 0 doublon**, chacune
re-verifiee contre le code stage (borne 8 : `pub async fn` dans `blobs.rs` = 9 + `new`
= la 10e ✓ ; borne 4 : 0 API artefact dans `vault.rs`/`keystore.rs` ✓ ; bornes 1/6 :
`impl ArtifactBlobs for` = 3, toutes sous `mod tests` ✓). Ancres externes toutes
exactes AUJOURD'HUI : `http.rs:272`, `node.rs:442` (`BlobsProtocol::new(&blobs_store,
None)`), `http.rs:246-256`, `blobs.rs:130-133`. Changelog v29 : « 6 rows neuves »
recompte a la source (`grep -c '^| \*\*T-ARTIFACT'` ⇒ **6**) ✓, **6/6 statuts VERBATIM**
compares un a un dans le MEME ordre ✓, « Bornes portees de 6 a 10 » ✓ avec les quatre
ajouts nommes. A16 (`:74`) coherent sur les cinq points porteurs.

**PLAN §Phase M** — diff BIDIRECTIONNEL recompte a la source, **0 ecart** : 21 noms
cites en §M.3 ↔ 21 tests livres dans `artifact_acl.rs`, **0 nom invente**, 0 test livre
non nomme. Les deux FAMILLES decrites en prose sont exhaustives 1:1 : 11 ↔ 11
`artifact_seal`, 4 ↔ 4 `artifact_sealer`. Total 21+11+4 = **36** = le delta annonce.
Compte de phases recalcule et non recopie : 46 − 1 = 45.

**GATE** — les TROIS commandes de la row 26 LANCEES : 21 / 11 / 4 passed, **exit 0
chacune**, somme 36 ; claim d'exit 4 exacte.
`bash scripts/check-frontier-contracts.sh` ⇒ exit 0, **DOMAIN census 29 frozen**, et
ce gate est **DISCRIMINANT** — mesure par mutation en worktree jetable : ajout d'un
`DOMAIN_R7PROBE_V1` ⇒ `check-frontier-contracts: FAILED`, exit 1 ; retrait ⇒ exit 0.

**CARTE ASCII** re-mesuree octet par octet (`cat -A`) : rail en colonne 66, la ligne M
porte son `│` en colonne 66 et s'aligne avec L et P1 ; la ligne M1 deborde SANS rail,
exactement comme L1, N, O, P et Q. Motif preserve.

**`docs/factory/WORKFLOW_AGENTS_OVERVIEW.md`** : une seule ligne modifiee, juste au mot
pres (deux traits nommes, `HostArtifactSealer` reel a 0 call-site, `ArtifactBlobs` sans
implementeur de production, ACL portee par le CHEMIN DU VAULT, renvoi bornes 1 et 6) —
aucun defaut. Aucun membre de la famille « absolu de destruction » dedans (grep ⇒ 1 hit,
`:48`, hors sujet).

### Suites §7.4 et delta de tests — LE BLOC POUR LE BODY DU COMMIT

- `cargo fmt --all --check` ⇒ exit 0, sortie vide.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` ⇒ exit 0, 0 warning.
- `cargo nextest run --workspace --locked` ⇒ **`2465 tests run: 2465 passed,
  7 skipped`**, exit 0.
- `cargo test --workspace --locked --doc` ⇒ **14 passed / 0 failed / 2 ignored**, exit 0.
- `cargo build -p nexus-shell-daemon --release` ⇒ exit 0.
- `bash scripts/check-frontier-contracts.sh` ⇒ exit 0, DOMAIN census **[29 frozen]**.

**DELTA DE TESTS EXACT POUR LE BODY : Rust nextest Windows 2429 → 2465 = +36**
(baseline L1, **jamais** le 2167 perime de `CLAUDE.md`). Vitest web 529 / operator 201 /
E2E inchanges. **Egalite stricte VRAIE et recomptee a la source** :
`artifact_acl` **21** + `artifact_seal` **11** + `artifact_sealer` **4** = **36 == 36** ;
aucun test hors-module n'a bouge. Doctests par categorie : `nexus_core_rs` 6 + 1
`compile_fail`, `sbfb_workflow_core` 1, `sbfb_workflow_engine` 3 + 3 `compile_fail` ;
les 3 blocs `compile_fail` du moteur sont chacun apparies a un jumeau qui COMPILE
(453/469, 541/549, 147/155).

**0 dep neuve** : `git diff --cached --stat` sur `Cargo.toml` / `Cargo.lock` /
`crates/*/Cargo.toml` est **VIDE** (`aes_gcm`, `rand`, `zeroize` etaient deja au
manifeste de `nexus-core-rs`). **0 bump wire** : les deux seules occurrences de
`*_VERSION` dans le diff `crates/` sont des lignes de DOC qui affirment le non-bump.

**SKIPS** : les 7 sont TOUS pre-existants (Phases E/I/K —
`store_single_writer_child_dies_holding_lease`, `effects_*` ×4,
`store_child_writer_exits_mid_run`, `orchestrator_child_dies_parked`). **Phase M
ajoute 0 skip** : ses 21/11/4 sont tous executes.

**FLAKE — aucun observe ce round, et je le dis avec sa nuance plutot qu'en vert sec.**
Les 12 tests de la classe iroh-networked (`nexus-test-harness` `multi_daemon` ×8 +
`cross_daemon_blob` + `blob_serve_coep`, `nexus-coordinator-rs` `multi_daemon`) etaient
bien DANS ce run et sont **tous verts sur Windows natif**. Le body doit l'annoncer
ainsi : « inclus et verts Win natif, classe env-instable sous Docker-on-Windows », avec
la commande d'isolement `cargo nextest run -p nexus-test-harness multi_daemon --locked`
(et `SBFB_TEST_HTTP_TIMEOUT_SECS=120` pour la classe voisine `operator_server` sous
Docker). Une seule trace non-flake a ete produite par une sonde : le run
`--run-ignored ignored-only`, lance pour identifier les 7 skips, fait ecrire
`crates/sbfb-workflow-engine/cwd_marker.txt` (finding #25) — supprime, arbre re-verifie.

---

## Findings tombes en refutation

**1 finding tombe (2/3 des jures refutent).**

**[P3, from-hex] « Raison perimee : "No serialisation round-trip can observe the
difference, because this type has no `Deserialize`" ».**
Ancre proposee : `artifact_acl.rs:385-386`.
**Refute pour deux raisons independantes, dont l'une est une mesure fausse.**
(a) La mesure de soutien affirmait que le test « doit utiliser `addresses_same_artefact`
(:1967) et non `==`, parce que `==` echouerait ». Relecture du blob : `sent` est bati
par `ArtifactRef::from_untrusted_wire` (:1951-1955) et la reconstruction aussi
(:1962-1966) ; le constructeur estampille `RefOrigin::UntrustedWire` dans les DEUX cas
(:476-487), `ArtifactRef` derive `PartialEq`, et digest/mode/retention sont egaux —
donc `==` **REUSSIRAIT**. Claim non reproductible.
(b) `from_hex` est un constructeur d'`ArtifactDigest`, pas d'`ArtifactRef` : le type
dont parle `:385-386` n'a TOUJOURS pas de `Deserialize`, donc la raison n'est ni
perimee ni fausse. Et la conclusion tient au sens ou elle est ecrite : deux references
qui ne different que par `origin` serialisent en JSON identique et se rebatissent
toutes deux en `UntrustedWire` — la difference reste inobservable par serialisation.
Le chemin de rebuild manuel est deja decrit trois paragraphes plus haut (:420-423).
Reste au mieux une preference de redaction adossee a une mesure erronee ⇒ **DROP**.

**Ancres corrigees plutot que findings retires** (honnetete de mesure, aucun finding
n'a du etre retire pour ancre fausse) :
- #20 : `THREAT_MODEL.md:2437` est **FAUX** — la borne 7 est a `:2411-2419`. Fait tenu,
  ancre recalee par moi.
- #15 : une section de brief ecrivait `artifact_acl.rs:566` — l'ancre exacte est
  **`:561`**, re-greppee (`pub fn new(key: Vec<u8>)` a `:562`).
- #4 : le plan porte la clause a `:1112-1115` selon le jure, `:1112-1114` selon le
  reporter ; texte retrouve verbatim dans les deux cas.

---

## Etat de l'arbre

**PROPRE — verifie par moi a la redaction, pas rapporte.**

`git status --porcelain` rend **exactement les 17 memes entrees stagees** qu'a
l'entree du round : 6 `A ` rapports r1-r6, 3 `A ` fichiers de code neufs
(`artifact_seal.rs`, `artifact_sealer.rs`, `artifact_acl.rs`), 8 `M `
(`nexus-core-rs/src/lib.rs`, `nexus-shell-daemon/src/main.rs`,
`sbfb-workflow-engine/src/lib.rs`, `secret.rs`, `sprint83_gate_matrix.md`,
`sprint83_plan.md`, `WORKFLOW_AGENTS_OVERVIEW.md`, `THREAT_MODEL.md`). **Aucune entree
untracked. `git diff` ⇒ 0 octet** (controle `wc -c` + marqueur `[END diff]`).

**Aucun code de production modifie, aucun correctif applique : les 25 defauts sont
DECRITS.** Ce fichier `sprint83_phase_m_review_r7.md` est le SEUL geste d'ecriture du
round — il apparaitra en 18e entree, untracked, jusqu'a son `git add`.

Les mesures destructrices ont vecu dans des worktrees jetables crees sur HEAD puis
alimentes par `git diff HEAD -- crates/ | git apply --index`, avec controle
`status --porcelain` du worktree = les 7 fichiers de phase (donc **mesures non
vacueuses**). Chaque mutation a ete posee par `perl -0777 -pi -e` (jamais python) et
confirmee par `grep -c` = 1 **AVANT** le run. Discipline observee et notee : **deux
mutations ont ete rejetees par ce controle avant mesure** (perl a mange un `\u` dans
une sonde ; un quoting `b'f'` n'a pas matche — `grep -c` a rendu 0, la mesure n'a
jamais ete lancee a vide) puis rejouees. Tous les worktrees de mesure ont ete retires
(`git worktree remove --force` + `prune`).

`git worktree list` ⇒ depot principal seul plus les worktrees PERMANENTS
(`.claude/worktrees/agent-*`, `nexus-098-flip` detached). **Aucun worktree de mesure ne
survit.** Aucun `zz_*` / `*probe*` / `*.bak`, aucun fichier untracked sous `crates/` ou
`web/src`. Le `cwd_marker.txt` produit par la sonde `--run-ignored` a ete supprime et
l'arbre re-verifie a 17 / 0 untracked / diff vide.

Consigne : `HEAD` est a `a67e1eb3`, un commit de PROCESS seul (gate de correctif +
§P76), **non audite comme Phase M**.

---

## Ce qui reste pour Codex

Le round de review s'arrete ici : **0 P1, critere « CLEAN ou P2/P3 documentes »
ATTEINT**. Ce qui suit est ce que la gate Codex doit chercher, pas ce que la review a
laisse tomber.

1. **Le geste de correctif documentaire lui-meme, et sa trace.** Le pattern qui a
   produit #9 est plus large que #9 : sur les trois sites bornes ce round, DEUX ont
   engendre un enonce derive qui n'a pas ete re-greppe (le compte de sites en borne 10,
   et le residu #11 laisse entre ses deux bornes au site meme qu'on editait). La regle 1
   du § « Pour le CORRECTEUR » — « pour un correctif documentaire, la mutation est le
   grep qui prouve l'affirmation » — n'a pas ete appliquee AUX ENONCES QUE LE CORRECTIF
   CREE. Controle d'une commande, a lancer apres edition et **avant** de stager :
   `git show :<f> | grep -nE "les (deux|trois) sites|exactly one|only producer|module de lib|rendus du moteur"`
   puis verifier chaque compte. C'est le seul controle qui aurait attrape #9, #14, #15,
   #17 et #20 d'un coup.

2. **Le seul finding a troncature de code (#1).** Verifier que le correctif ecrit bien
   `hex.chars().take(72).collect()` et **jamais** `&hex[..72]` : le decoupage par index
   d'octet panique sur une frontiere de `char`, et la propriete auditee de la phase est
   « 0 panique atteignable dans la moitie production ». Si le correcteur applique #1,
   c'est le seul geste du lot qui puisse INTRODUIRE un defaut.

3. **La borne 9 de §19.5 (`THREAT:2436-2437`), jumeau francais de #8.** Elle porte
   elle-meme « la garantie qui compte reste disponible … l'appelant crypto-shredde en la
   droppant » sans borne dans sa propre phrase — elle n'est sauvee que par la borne 10
   qui la suit immediatement. Aucun des 25 findings ne relie les deux : si le correcteur
   touche `:2069-2076` (#8), il doit relire `THREAT:2436-2437`.

4. **`from_hex` fabrique un digest BIEN FORME qui n'adresse rien.** Le doc du type
   promet « The content address of what is STORED » ; pour un digest reconstruit depuis
   le JSON, c'est desormais une CLAIM et non un fait. C'est rattrape a l'usage (`read`
   et `delete` re-hachent via `of_stored_bytes` et rendent `DigestMismatch`), mais aucune
   ligne de `from_hex` ne le dit — une phrase a folder dans le geste de #2, qui touche
   deja ce doc.

5. **Ce que la Phase M ne doit PAS porter, et que P doit epingler.**
   `reference_wire_form_is_pinned` epingle la forme EMISE, jamais une forme RECUE. Le
   plan §P.2 le nomme correctement (« P doit epingler la forme recue par un golden, pas
   seulement la forme emise ») : c'est deja trace vers P, ce n'est pas une dette de M.
   Codex ne doit pas le rouvrir ici.

6. **Ne pas rouvrir `artifact_acl.rs:2033`** (« a consumed capability would be gone for
   good ») : r6 l'a explicitement defendu — vrai dans son scenario, aucune copie
   possible. Meme traitement pour `THREAT_MODEL.md:2371` (« ce qui sort est du chiffre
   que personne ne peut ouvrir ») : ce qui sort part vers une partie sans cle, et le
   scope cut 10 dit que rien ne distribue les cles.

7. **Le carry a router vers `sprint84_audit_plan.md`** : #25
   (`effects_child_writes_cwd_marker` salit le cwd du crate sous `--run-ignored`),
   proprietaire = phase K du harness d'effets, **pas M**. Plus le carry existant
   `DOCTEST-ERRCODE-NOT-ENFORCED`, dont la branche d'exit doit passer de « ou » a « ET »
   (#10) avant de pouvoir etre clos honnetement.
