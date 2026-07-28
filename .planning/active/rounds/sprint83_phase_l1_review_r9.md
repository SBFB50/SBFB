# Review — Sprint 83 Phase L1 — Coffre de secrets et HITL hote (round 9)

## Verdict: FAIL

Quatre dimensions jouees en lecture seule (borne 27 et systeme d'ordinaux ; mur
Argon2id ; gate mecanique ; balayage des jumeaux et des fichiers jamais relus),
33 soupcons leves, 8 refutes par mesure, 25 retenus : **2 P1, 10 P2, 12 P3**. Le
verdict tombe sur les deux P1 seulement, et les deux sont des editions d'UNE
LIGNE dans un artefact rank-1, sans une ligne de code a toucher. P1-1 est la
forme (b) a l'etat pur, trouvee INDEPENDAMMENT par deux dimensions (A-1 et B-1,
meme ancre, meme recompte) : le correctif r8 a ajoute la 27e borne a §19.4, a
mis a jour le renvoi de `docs/workflow/llms.txt:110` (« bound 27 of §19.4 ») et a
laisse « The honest bounds (**26 of them**, numbered) » treize lignes plus haut,
`:97`, dans le meme bloc `+` — recompte machine ce round entre `### 19.4`
(THREAT_MODEL.md:1911) et `## 20.` (:2292) : marqueurs colonne-1 = 1..27,
CONTIGUS, sans trou ni doublon, borne 27 ancree a :2262. Ironie utile et
mesuree : la jambe de gate REFUSE deliberement d'ecrire ce compte
(`scripts/check-frontier-contracts.sh:545-547`, « No count is written here on
purpose: three independent recounts in review round 8 disagreed with each
other ») — le correctif a donc re-arme dans un rank-1 le piege que le script
venait de desarmer, et rien ne peut rougir dessus. P1-2 est une instruction de
maintenance fausse que r8 a re-endossee sans la relire
(`docs/rust/PATTERNS.md:1581-1582`, « Record any such bump here AND in the blob
`flags` byte so older blobs keep unlocking ») : l'octet `flags` est l'offset 7,
bit 0 = `use_keyring_layer` et RIEN d'autre (verifie : seul `0b0000_0001` ecrit
a keystore.rs:619/:790/:885, lu a :1202-1203), les params KDF vivant aux offsets
24/28/32 et etant ecrits automatiquement par `encode_header` (:1167-1171) —
mecanisme que le MEME fichier rank-1 enonce CORRECTEMENT 387 lignes plus haut
(:1195-1196). Ce qui tient tient solidement, et je le declare : les quatre
consequences de la borne 27 sont exactes contre la trace lue, l'ajout du 27e
item n'a decale AUCUNE des 30+ citations par ordinal du depot, les deux tables
de menace jumelles sont byte-identiques sur leurs lignes de donnees, les 26
temoins d'identite sont byte-identiques des deux cotes, les huit mutations r8
sur la jambe reecrite reproduisent toutes, et la classe « substitution qui
avorte le gate en silence » est REELLEMENT fermee.

## Etat des correctifs r8

| # | Action r8 | Etat | Preuve fichier:ligne (lue ce round) |
|---|---|---|---|
| 1 | Borne 27 neuve en §19.4 + renvois re-pointes | **A CREE UN DEFAUT** | Borne exacte et complete (THREAT_MODEL.md:2262-2284, 4 consequences verifiees contre la trace vault.rs:501-511/:552-580/:591-604/:629-642 + :699-711) ; renvois OK (vault.rs:496, :585, :621-622 ; llms.txt:110 ; gate_matrix:44). MAIS le compte de llms.txt:97 reste a « 26 of them » (P1-1) et l'enumeration normative :2281-2284 nomme trois rustdocs en oubliant `persist` (A-2) |
| 2 | Mur Argon2id — 3 sites supplementaires, CIBLE vs MESURE | **PARTIEL** | Tables jumelles byte-identiques sur les 4 rows (keystore.rs:85-87 == PATTERNS.md:1438-1440, « ~82 ms/attempt measured ») ; etiquetage CIBLE pose a keystore.rs:50/:55, Cargo.toml:272/:275/:280-282, PATTERNS.md:1175/:1560, benches:5. MAIS 6 sites vivants disent encore « 3-second » comme un FAIT, dont 2 ECRITS PAR CETTE PHASE (vault.rs:306, :1152 — B-3), et le doc-module a gagne un locatif « below » vers du vide (B-4) |
| 3 | `|| true` porte de 4 a 6 substitutions | **TIENT** | Les six presentes et terminees par `|| true` : :382, :385, :430, :487, :490-492, :521. Classe fermee, mesuree : les 9 substitutions sans `|| true` sont soit sans pipeline (:75 x2, :76), soit des ARGUMENTS d'`echo` (:440/:441/:531/:532, ou errexit ne s'applique pas), soit des filtres purs rendant 0 sur entree vide (:143, :259). Residu : le chiffre « ALL SIX » du commentaire :374 (C-R9-2) |
| 4 | Jambe §19.4 BOUND ORDERING reecrite | **A CREE UN DEFAUT** | Les 8 mutations r8 reproduisent toutes ; scan instrumente = 27 bornes contigues, `scanned=27`. MAIS deux fail-OPEN neufs mesures dans le meme awk de 10 lignes : la regle de titre :573 precede la bascule de fence :579 (une ligne `# ` en colonne 1 dans une fence tue le scan) et `fence` n'est jamais audite dans END :593-599 (C-R9-1) ; la claim « no section seam can reset it » :584-586 est fausse (C-R9-4) ; le membre « sub-heading » de :556 est faux tel qu'il se rejoue (C-R9-3) |
| 5 | Borne 25 — « en-tete, probe ET commentaire in-corps », les TROIS | **TIENT** | Les trois sites enumerees sont bien traites (keystore_integration.rs:12-23, :58-67, :76-82) et la borne se pre-immunise textuellement contre l'exhaustivite (THREAT_MODEL.md:2216-2220). Residu : deux commentaires non requalifies 24 lignes plus bas (:94, :129 — D-02) |
| 6 | DURESS §5 — deux puces + « les paragraphes §1 ET §5 » | **PARTIEL** | §5 corrige (DURESS.md:184-190, « §1 ET §5 — les DEUX »). MAIS §1 porte toujours la forme mono-paragraphe (:29-31, « devra etendre ce paragraphe ») : geste sur UN SEUL des deux jumeaux, sur la consigne dont l'objet EST d'empecher qu'un jumeau soit oublie (D-04) ; et §1 ne porte pas l'ancre « borne 12 » que son jumeau :185 porte (D-03) |
| 7 | panic.rs etape 1 requalifiee + inscrite dans les DEUX enumerations | **PARTIEL** | Inscription faite (borne 25 + audit_plan) ; test de reprise des scratchs non vacueux (chemins `identity.enc.tmp`/`identity_duress.enc.tmp` = ceux que `write_atomic_with_tmp_extension` cree, keystore.rs:1288). MAIS la reprise est BEST-EFFORT a echec silencieux (:1066-1067 `let _ =`) et les TROIS textes la presentent comme inconditionnelle (D-07) ; l'etape 2 nomme encore `state.sqlite` la ou la prod passe `subscriptions_json` (D-08) |
| 8 | llms.txt — 4e membre `put` COMMIT avec son verbe propre | **TIENT** | llms.txt:114-115 « `put` COMMITS the stale body, silently undoing the other handle's write » |
| 9 | gate_matrix row 10 — « borne 27 du §19.4 » + 4 consequences | **TIENT** | gate_matrix.md:44 ; « **20 tests `vault::`** » recompte a la source ce round : `grep -c '#\[test\]' vault.rs` = **20**, exact ; la qualification « pas apres la collision » est exacte contre le test cite (:1806-1857, scenario d'epoch EN RETARD) |
| 10 | 4 temoins ajoutes aux DEUX jumeaux (C0/C1 refus, non-ASCII accept) | **TIENT** | 26 litteraux byte-identiques des deux cotes (vault.rs:1726-1779 == hitl.rs:917-974, `diff` vide) ; les 4 classements verifies contre les deux predicats (vault.rs:837-857, hash.rs:162-187). Residu de CLASSE : `c.is_control()` couvre TROIS morceaux, et ni DEL `U+007F` ni les bornes C1 `U+0080`/`U+009F` n'ont de temoin — mesure ce round, grep = ZERO hit (D-05) |
| — | ROUTAGE des 4 chantiers en P2 | **PARTIEL** | audit_plan:206-218 : (a) et (b) sont retrouvables et nomment leur cible ; (c) :216 designe `_bp_extract_rs` et `_hp_extract_ts`, deux symboles absents du depot (contradiction inter-dimensions arbitree ci-dessous) ; (d) est mince mais rattrape par son voisinage. La liaison ordinal→contenu (A-3) n'y est PAS routee |

## Perimetre verifie

Lecture seule stricte : aucun Edit/Write hors ce fichier, aucun git mutatif,
**aucune commande cargo ni npm**. Toutes les mutations ont ete jouees sur des
COPIES dans le scratchpad, jamais sur le depot. `set -o pipefail` sur tous les
pipes.

- **Recompte des bornes** : `awk` fence-aware borne a la section entre
  `docs/security/THREAT_MODEL.md:1911` (`### 19.4`) et `:2292` (`## 20.`) →
  COUNT=27, sequence 1..27 contigue, ancres 1@:1984 … 25@:2203, 26@:2249,
  27@:2262 ; les « 1. 2. 3. » de :2297-2300 sont dans §20, hors section.
- **Inventaire des citations par ordinal** : ripgrep `(borne|bound)s? \(?[0-9]+`
  hors `rounds/` et hors archive, chaque cible ouverte et appariee au texte de
  sa borne — 30+ renvois, tous corrects apres l'ajout du 27e item (qui est
  APPENDU, donc 1..26 intact). Aucun « bound 2 of §19.4 » perime : les deux
  occurrences restantes sont l'une une citation historique (:2267), l'autre un
  renvoi CORRECT (hitl.rs:478 → temoin `HostVerifiedApproval` = borne 2 reelle).
- **Format du blob keystore** : keystore.rs:104-115 (layout), :619/:790/:885
  (ecriture du seul bit 0), :1167-1171 (`encode_header`), :1202-1203 (lecture du
  bit 0), :1211-1217 (relecture des params), :686 et :997 (les DEUX chemins de
  lecture derivent avec les params du HEADER) — preuve de P1-2.
- **Jambe BOUND ORDERING** : extraite VERBATIM (`sed -n '558,608p'`) et pilotee
  dans un faux repo. 8 mutations r8 rejouees (5 rouges, 3 muettes correctement)
  + 12 mutations neuves. Fail-OPEN mesures : fence colonne-1 + ligne `# ` +
  inversion → VERT (le meme fichier avec `echo` au lieu de `#` → ROUGE, donc
  discriminant a un token) ; fence non fermee + inversion → VERT
  (`scanned=19 prev=19 fence=1`, 8 bornes perdues muettes) ; titre §19.4
  duplique + renumerotation de 22 ordinaux → VERT (sans le titre duplique →
  ROUGE). Plancher `scanned` : 25/27 bornes perdues → VERT, 26/27 → ROUGE.
- **Fixtures de parite** : 6 mutations de `_bp_extract_ts` et 8 de
  `_hp_extract_rs` rejouees — les 5 kills BRIDGE et les 3 kills HITL revendiques
  reproduisent, les 3 non-kills declares sont vrais, et DEUX pieges HITL
  effectifs ne sont pas documentes (`sort -u`, ordre).
- **Jumeaux d'identite** : extraction mecanique des litteraux des deux listes +
  `diff` → vide, 26 de chaque cote. `grep -rnE 'u\{7[fF]\}|u\{007[fF]\}|u\{0080\}|u\{009[fF]\}'`
  sur vault.rs + hitl.rs + hash.rs → **exit 1, zero hit**.
- **Gate reel** : `bash scripts/check-frontier-contracts.sh` → EXIT=0, sortie
  `clean (… + hitl-parity [15 fields])`, huit membres, aucun n'est la jambe des
  bornes.
- **Lectures integrales** : `hitl_approval.rs` (229 l), `secret.rs` (663 l),
  `DURESS.md` (226 l), plus les sections ciblees de vault.rs, keystore.rs,
  panic.rs, keystore_integration.rs, PATTERNS.md, llms.txt, audit_plan,
  gate_matrix.

**Les suites viennent du thread principal, pas de ce round** : Windows nextest
2428 passed / 0 failed / 7 skipped, fmt 0, clippy 0, doctests ok ; Docker
`sbfb-ci` 659 passed / 7 skipped ; vitest 565/565 ; coverage
86.94/78.94/86.13/88.35 ; e2e 59 passed / 2 skipped ; build release exit 0. Je
n'en ai rejoue aucune et je ne m'en sers pour refuter aucun finding : les deux
P1 portent sur des proprietes qu'AUCUNE des deux surfaces ne teste.

**Incident de concurrence, a remonter.** `scripts/check-frontier-contracts.sh`
a ete MODIFIE PENDANT ce round : etat courant LF, 616 lignes, md5
`745aff7099a8f6e772f0227d87f6c24d`, mtime 2026-07-28 13:13:08 — le fichier est
passe CRLF→LF et a gagne :136-143 (correctif CRLF du word-split FRONTIER, dont
le commentaire se date lui-meme « S83 L1 round 9 »). Toutes les mesures de ce
rapport ont ete REJOUEES apres detection et les resultats sont identiques dans
les deux etats ; **les ancres de ce rapport sont post-edition**. Une mesure
prise avant 13:13 porte des ancres decalees de −8 lignes a partir de :143.

**Note de methode heritee du round 8, respectee** : `git diff` ne prouve rien
sur `vault.rs`, `hitl.rs`, `secret.rs`, `hitl_approval.rs` (UNTRACKED, `??`).
L'attribution de B-3 a cette phase repose sur `git status --porcelain`, pas sur
un diff. Le diff n'a servi de preuve que sur `llms.txt`, `THREAT_MODEL.md` et
`PATTERNS.md`, qui sont TRACKES.

## Findings

### P1

| ID | Ancre | Defaut | Scenario | Correctif |
|---|---|---|---|---|
| **P1-1** (A-1 = B-1) | `docs/workflow/llms.txt:97` | Claim ENUMERATIVE fausse dans un rank-1, auto-invalidee par le correctif r8 : « The honest bounds (**26 of them**, numbered) are §19.4 » alors que §19.4 en porte **27** depuis que r8 y a ajoute la borne 27 — et le MEME fichier cite « **bound 27 of §19.4** » treize lignes plus bas (:110), les deux lignes etant dans le MEME bloc `+` du diff | Un lecteur applique la Truth Stack de llms.txt (index MACHINE, rank-1), lit « 26 bornes numerotees », et conclut que la borne 27 citee 13 lignes plus bas n'existe pas — exactement le mode de panne que la borne 27 a ete creee pour reparer (r8 P1-1 : un renvoi qui ne resout pas). Aucune barriere mecanique : la jambe §19.4 refuse deliberement d'ecrire un compte (`check-frontier-contracts.sh:545-547`), donc le gate est vert et le restera | **SUPPRIMER la parenthese chiffree** : « The honest bounds (numbered) are §19.4 ». C'est la doctrine que le depot s'est donnee a :545-547. Ecrire « 27 » ré-arme le piege sans gate pour le tenir ; si le compte est conserve, il faut une jambe qui le compare a la source |
| **P1-2** (B-2) | `docs/rust/PATTERNS.md:1581-1582` | Claim technique FAUSSE dans un rank-1, attachee a l'action que r8 vient de re-armer : « Record any such bump here AND in the blob `flags` byte so older blobs keep unlocking. » L'octet `flags` est l'offset 7, bit 0 = `use_keyring_layer` et SEUL bit existant ; il ne porte aucun parametre KDF. Les params vivent aux offsets 24/28/32 et sont ecrits AUTOMATIQUEMENT — rien a « enregistrer » a la main — et la clause causale est servie par ces champs. Le MEME fichier l'enonce correctement a :1195-1196 | Le paragraphe prescrit precisement le bump que r8 vient de re-armer (:1578-1581). Le mainteneur qui l'execute cherche ou « enregistrer le bump » dans `flags` : soit il n'y trouve qu'un bit de keyring et conclut que la compat ascendante n'est pas geree (re-mint inutile de toutes les identites), soit il consomme un bit que le decodeur ignore (:1202-1203 ne lit que le bit 0) — un champ mort silencieux dans un format wire pre-launch | Remplacer « in the blob `flags` byte » par « in the blob header's `m_cost`/`t_cost`/`parallelism` fields (offsets 24/28/32), that `encode_header` already writes from `KdfParams` — no manual step ». Aligner sur :1195-1196, deja correct |

Les deux P1 sont des editions de texte. **Zero ligne de code, zero suite a
re-jouer.**

### P2

| ID | Ancre | Defaut | Scenario | Correctif |
|---|---|---|---|---|
| A-2 | `THREAT_MODEL.md:2281-2284` + `vault.rs:696-698` et `:1469-1470` | L'enumeration normative de la borne 27 nomme trois rustdocs (`bind`, `rotate`, `delete` — les trois portent bien la qualification, verifie :496/:585/:621-622) et manque les DEUX sites restants qui decrivent la garde sans la qualifier : `persist`, la fonction qui l'IMPLEMENTE, et le rustdoc du test, qui porte la claim de fermeture la plus forte du fichier | `persist` declare fermee la classe ex-P1-4 (« a stale handle rewriting its whole snapshot resurrected deleted secrets ») que la borne 27 declare ROUVERTE : trace verifiee, `refuse_if_stale` passe (:651-660) puis :711 commet le body perime COMPLET. Attenuation reelle : les deux sites sont prives ou `#[cfg(test)]`, et le commentaire in-corps :712-718 avertit deja le lecteur | Qualifier `persist` ET le rustdoc de test, puis corriger l'enumeration :2281-2284. **Fermer la CLASSE, pas l'instance citee**, sinon le round 10 la rouvre |
| A-3 | `check-frontier-contracts.sh:563-600` | Le seul garde du systeme d'ordinaux ne lie pas ORDINAL → CONTENU : une INSERTION suivie de renumerotation preserve la contiguite et laisse la jambe MUETTE pendant que chaque citation `bound N` change de referent | Mesure : insertion apres la borne 12 + renumerotation 13..27→14..28 → jambe MUETTE, et `bound 25` glisse de « La couche 2 du keystore (OS-keyring) est INERTE » a « Le spoof d'AFFICHAGE ». `bound 25` est cite de **13 endroits vivants** (recomptes), dont deux rank-1 et le runbook de duresse | A ROUTER, pas a corriger ce round : (a) ancrer les citations sur un SLUG stable, l'ordinal restant un confort ; ou (b) apparier ordinal↔motif de texte pour un petit set de bornes load-bearing. **N'est PAS dans les quatre chantiers deja routes** (audit_plan:206-218) |
| B-3 | `vault.rs:306` et `:1152` | Deux sites NEUFS de cette phase enoncent « 3-second production KDF » comme un FAIT — faux d'un facteur 36 contre la mesure canonique 82 ms — et contredisent la regle que la MEME phase vient d'ecrire 846 lignes plus haut dans le fichier jumeau (`keystore.rs:60` : « Cite the measurement, never the target ») | Le sprint MINTE de nouvelles instances de la classe qu'il pretend fermer, pendant les rounds qui la corrigent ailleurs. Quatre autres sites (keystore.rs:389/:515, benches:77/:80) sont de la dette S20 pre-existante, en P3 | Porte reduite aux DEUX sites neufs : citer la mesure ou, mieux, le fait qui motive vraiment `fast_for_tests` (« 64 MiB per derivation »). Controle de fermeture : `grep -rnE "3[- ]?second|~3 ?s" crates/ \| grep -iE "argon\|kdf\|keystore\|vault"` |
| C-R9-1 | `check-frontier-contracts.sh:573-577`, `:579-580`, `:593-599` | La comptabilite de fence du correctif 4 est a moitie posee, dans les DEUX sens : la regle de titre est evaluee AVANT les regles de fence et n'est pas gatee par `fence` ; et `fence` n'est jamais verifie dans END | Mesure sur les octets courants : (a) fence ```bash dont la 1re ligne est `# recompute the epoch` + inversion 25/26 → **VERT**, le meme fichier avec `echo` a la place de `#` → ROUGE ; (b) fence non fermee + inversion → **VERT**, END instrumente `scanned=19 fence=1` (8 bornes perdues muettes). Latent : 0 fence dans §19.4, 2 dans les 2880 lignes, aucune prefixee `#` | Deux lignes, zero chiffre : deplacer :579-580 AVANT :573 (ou `&& !fence`), et ajouter `if (fence) print "a code fence opened in §19.4 was never closed"` dans END. Les huit controles r8 restent inchanges |
| C-R9-3 | `check-frontier-contracts.sh:556-557` | Un des six membres de l'enumeration de mutations est faux tel qu'il se rejoue : `#### 19.4.0` SEUL laisse la jambe VERTE (et c'est le comportement CORRECT). Ce qui rougit, c'est le sous-titre COMBINE a une inversion | Fausse RELATIVEMENT A SON PROPRE REFERENT : la phrase :552-555 designe les quatre fail-open que r8 exigeait de rendre rouges ; deux le sont, deux ne le sont pas — correctement, le correctif ayant transforme le sous-titre en NON-mutation. L'implementeur a diverge a bon droit mais revendique la conformite. Cout : un round de re-arbitrage, paye huit fois deja | Une clause : « sub-heading **COMBINED WITH an inversion** ». Et ajouter la ligne de bornes negatives que la mesure fournit (le sous-titre seul et une fence close restent MUETS, correctement) |
| C-R9-4 | `check-frontier-contracts.sh:584-586` | « `prev` is seeded ONCE … and no section seam can reset it mid-scan » est faux : la regle d'entree :569-571 n'est pas gatee par `inside` et re-arme `first=1` a chaque titre matchant — `prev` est seme une fois PAR ENTREE | Mesure plus forte que celle du finding : titre `### 19.4 (suite)` duplique + bornes 6..27 renumerotees 1..22 → **VERT**, alors que la MEME renumerotation sans le titre duplique → ROUGE. Une seule ligne de titre neutralise la jambe sur un decalage de 22 ordinaux. §19.4 fait 381 lignes et le geste anticipe par r8 est de la scinder | Gater la regle d'entree sur `!seen`, ou reformuler honnetement (« `prev` est seme une fois par entree ; un titre §19.4 duplique re-seme et laisse son 1er marqueur non compare ») |
| D-04 | `DURESS.md:29-31` (§1) ↔ `:188-190` (§5) | Le correctif 6 a pose « les paragraphes §1 ET §5 — les DEUX » sur §5 seulement ; §1 porte toujours « devra etendre **ce paragraphe** ». Forme (a), sur la consigne dont l'unique objet EST d'empecher qu'un jumeau soit oublie | Le commit qui cable `SecretVault` est ecrit par quelqu'un qui entre par §1 (la « Vue d'ensemble ») : il etend §1, le trouve honnete, committe — et §5, la section lue APRES le wipe, devient une promesse fausse. Attenuation mesuree : les DEUX puces contiennent le litteral `SecretVault` (:27 et :184), donc un balayage par grep atteint les deux. Rien de ce qui est ecrit aujourd'hui n'est faux : §1 est plus ETROIT que son jumeau, pas errone — d'ou P2 et non P1 | Aligner §1 sur la forme « §1 ET §5 — les DEUX » |
| D-05 | `vault.rs:1774-1778` ↔ `hitl.rs:969-973` | La discipline « une plage porte son point BAS et son point HAUT », appliquee aux NEUF plages multi-points, n'a pas ete appliquee a `c.is_control()`, qui est lui-meme une union de TROIS morceaux (C0, DEL `U+007F`, C1). Les temoins ajoutes sont `\t` (interieur de C0) et `U+0085` (MILIEU de C1) | Mesure ce round : `grep` des six formes de DEL/`U+0080`/`U+009F` sur les trois fichiers → **exit 1, zero hit**. Une reecriture plausible en `matches!(c, '\0'..='\u{1F}' \| '\u{80}'..='\u{9F}')` rend DEL acceptable dans un `secret_ref`, un `capability_ref` et les 11 composantes du tuple HITL — les DEUX jumeaux restent verts (26 temoins de refus, 0 sur DEL). C'est le defaut que le commentaire voisin decrit pour `E0020..E007F` | Ajouter aux DEUX listes, position identique : `"with\u{7F}delete"`, `"with\u{0080}c1-start"`, `"with\u{009F}c1-end"`. Appartient au chantier deja route `const IDENTITY_FORBIDDEN_RANGES` (audit_plan:212-214) |
| D-06 | `secret.rs:90` | Le garde `validate_identity_component("secret_ref", &name)?` de `SecretRef::new` n'a AUCUN test negatif : mesure ce round, `grep -rn 'SecretRef::new' crates/ --include=*.rs` = deux lignes seulement, la rustdoc (:73) et le helper de test (:314, `.expect("valid ref")`) | Une simplification retire la ligne 90 (« le validateur est deja applique en amont ») : le workspace entier reste vert. La rustdoc :73-76 declare pourtant ce garde load-bearing (« cannot smuggle a separator or a newline into a line-oriented log »). Correction du raisonnement : les DEUX jumeaux epinglent le VALIDATEUR (vault.rs:1719, hitl.rs:906) ; l'asymetrie reelle est le SITE D'APPEL — le coffre a `check_entry_is_unreachable_from_user_operations` (vault.rs:1333), `SecretRef::new` n'a pas d'equivalent | Trois lignes dans `secret.rs::tests` : `is_err()` sur `""`, sur `"with\nnewline"`, sur `"a".repeat(IDENTITY_MAX_LEN + 1)` |
| D-07 | `keystore.rs:1066-1067` ↔ `THREAT_MODEL.md:2253-2254`, `DURESS.md:173-174`, `panic.rs:11-16` | La reprise des deux scratchs par `wipe_all` est BEST-EFFORT a echec silencieux (`let _ =`, contre `?` aux :1047-1048) — choix delibere, argumente sur place et couvert par son test (:1759-1771). Les TROIS documents la presentent comme inconditionnelle, et panic.rs:31 (« Steps 2 and 3 are best-effort ») affirme par contraste que l'etape 1 ne l'est pas | Un AV, un `rotate_pin` en vol ou une ACL verrouille `identity.enc.tmp` : les deux blobs partent, `wipe_all` rend `Ok(())`, `report.keystore_wiped = true` INCONDITIONNELLEMENT (panic.rs:181-182), et il subsiste une copie COMPLETE et scellee de l'identite sous l'ancien PIN. La branche avalee n'est pas benigne : `secure_unlink_blob` retourne tot sur `!exists()`, donc l'erreur avalee est un ECHEC REEL. Aucun rapport, aucun log, aucune borne | Une phrase dans la borne 26 et une incise dans DURESS §5 (reprise tentee, echec avale, arbitrage assume) ; dans panic.rs, distinguer dans l'etape 1 les deux gestes hard-fail des quatre best-effort |

### P3

| ID | Ancre | Defaut | Correctif |
|---|---|---|---|
| A-4 | `check-frontier-contracts.sh:573` vs `:579` | Meme mecanisme que C-R9-1, mais le SCENARIO invoque est refute : un bloc de code attache a un item de liste est INDENTE en CommonMark, et la regle de fence accepte l'indentation — mutation avec fence indentee → la jambe SURVIT (ROUGE). Le fail-open exige la conjonction fence-colonne-1 ET ligne `# `-colonne-1 | Couvert par le correctif de C-R9-1 |
| A-5 | `vault.rs:468-471` | « which the four writing and binding entry points run FIRST » : `refuse_if_stale` tourne dans `persist`, apres `validate_ref`, `seal_entry` et l'insert. Nuance : il n'est litteralement premier pour AUCUN des quatre (les trois autres l'appellent en deuxieme geste) | « run before answering / before touching the disk » — redevient vrai des quatre a la fois |
| B-4 | `keystore.rs:52-53` | Locatif « below » vers du vide (aucun chiffre GPU sous :54 ; mesure : occurrences aux 52/53/54/62 seulement). Fortement attenue : les deux valeurs sont reproduites DANS la meme phrase et la retractation y est explicite. Residu reel : les chiffres ont perdu leur referent « 6-digit PIN brute force », donc leur unite | « quoted here » au lieu de « below », et retablir l'unite |
| C-R9-2 | `check-frontier-contracts.sh:374` | « ALL SIX command substitutions » : recompte exact = 26 substitutions, 17 avec `|| true`. Vrai sous la lecture etroite, faux sous la lecture large. Mesure : AUCUN trou reel sous la lecture large. La redaction est en outre VERBATIM celle prescrite par r8 | Supprimer le chiffre, garder l'enumeration nominative |
| C-R9-5 | `check-frontier-contracts.sh:500-505` vs `:512-520` | La fixture HITL porte CINQ pieges effectifs, le commentaire en enumere TROIS (`sort -u` et ordre non documentes, tous deux mesures KILLED). Le jumeau BRIDGE documente les siens (:416-418). Troisieme asymetrie du meme motif : la fixture HITL n'a pas non plus de leurre a majuscule, la ou BRIDGE a `"Zulu-Cap"` | Deux puces calquees sur :416-418 |
| C-R9-6 | `check-frontier-contracts.sh:143` vs `:381` | La regle « Any NEW extraction added below must carry it too » est positionnelle et ne couvre pas :143, ajoute au-dessus le jour meme. Consequence refutee : la seule extraction future planifiee (jambe `IDENTITY-DOMAIN-PARITY`) atterrirait apres :537, donc dans la portee | « anywhere in this file » |
| C-R9-7 | `audit_plan:216` | Les deux symboles cites sont ingrepables (0 occurrence dans le depot). Voir l'arbitrage de contradiction ci-dessous | Citer les ancres `:385` et `:490-492` en plus des noms de convention |
| C-R9-8 | `check-frontier-contracts.sh:615` | Le message de succes enumere 8 couvertures et n'inclut pas la jambe §19.4 (sortie reelle mesuree). Corroboration neuve : la jambe n'a pas non plus d'en-tete `# ── (N) … ──`, alors que les huit autres en portent un — indiscernable d'une jambe supprimee sur les deux plans. Ouvert depuis r8 et non route | Ajouter `+ §19.4 bound ordering`, sans chiffre |
| C-R9-9 | `check-frontier-contracts.sh:579`, `:581` | Quatre classes de faux positifs mesurees (fence `~~~`, seconde liste numerotee, ligne `2026. ` colonne 1, marqueur re-indente 1-3 espaces), toutes fail-CLOSED. Asymetrie : les jambes BRIDGE (:355-364) et HITL (:457-469) declarent leur portee, celle-ci non | Trois lignes de commentaire declarant ces bornes |
| D-02 | `keystore_integration.rs:94` et `:129` | Deux commentaires portent encore la mis-attribution « flaky on some Windows Credential Manager backends » que le probe refute nommement 25 lignes plus haut. Le P1 propose est refute : la borne 25 se pre-immunise contre l'exhaustivite (:2216-2220), les trois sites enumerees SONT traites, et le scenario est detruit par la doc du probe (:65-67) | Requalifier les deux commentaires (le blocage vient du builder MOCK) |
| D-03 | `DURESS.md:25` | §1 ne porte pas l'ancre « borne 12 » que son jumeau §5 porte (:185). Le P1 propose est refute : le parenthetique annote l'EDITION de la puce entiere, la clause precedente EST la borne 26 verbatim, et le fait substantiel est enonce IN SITU (:26-29) | Distribuer les citations : « (borne 25) … (borne 12) » |
| D-08 | `panic.rs:22` et `:184` | L'etape 2 nomme `state.sqlite` la ou la prod passe `subscriptions_json` (runtime.rs:443). Fortement attenue : la divergence est deja divulguee au site de cablage (runtime.rs:434-437), la phrase identifie le fichier par son ROLE, et DURESS §5:182 — le seul texte que l'operateur lit — est correct | Nommer `subscriptions.json` avec le renvoi a runtime.rs:434-437 |

## Findings refutes

Huit soupcons fermes par mesure. Le taux de refutation (8/33 = 24 %) est le plus
haut du dossier et il est lui-meme un signal de convergence.

| ID | Soupcon | Mesure qui innocente |
|---|---|---|
| B-5 | Les deux tables de menace ne sont pas litteralement identiques (en-tete « Needs » vs « Needs to decrypt ») | L'action r8 relue a la source (`review_r8.md:46`) portait sur la **CELLULE**, pas la table : « harmoniser « ~80 ms » sur « ~82 ms » pour que les deux CELLULES soient litteralement identiques ». Atteinte : `diff` mecanique → 4 rows + separateur byte-identiques. La divergence d'en-tete est PRE-EXISTANTE (ligne de contexte S20 au diff) et aucune jambe du gate ne compare ces tables |
| B-6 | L'explication ARM/Android du facteur 36x n'est pas soutenue par le dossier S20 | La source PRIMAIRE dit l'inverse : `sprint20_kickoff.md:33-35` source le chiffre NOMMEMENT du blog Signal SVR (« Calibre ~3s/attempt », pattern enveloppant une cle « Shared Preferences », API Android). Le passage cite comme contradicteur (`design_review.md:81-82`) ecrit « plausible » — c'est une CORROBORATION, pas une origine. Et le correctif reclame est deja prescrit a PATTERNS.md:1193-1195 |
| B-7 | `SBFB.txt:159` porte la claim doublement fausse et doit etre route | `git check-ignore -v SBFB.txt` → `.gitignore:166:/SBFB.txt`. Le fichier est **EXCLU du depot**, absent de `git status`, non commitable sans `git add -f`. Ni defaut de code ni artefact rank-1 : hors juridiction. L'inscrire dans une enumeration du threat model la rendrait invalidable par un fichier que le depot ne verra jamais |
| D-01 | Le routage (c) inverse les suffixes et nomme des symboles inexistants (**P1**) | Voir l'arbitrage de contradiction ci-dessous : la convention du script est `_{jambe}_extract_{langage}`, donc les deux pipelines INLINE s'appellent bien `_bp_extract_rs` (:385) et `_hp_extract_ts` (:490-492) sous cette convention — et ce sont exactement les deux non self-testes. Le predicat de la phrase EST « n'ont ni fonction nommee », donc un grep a 0 la CONFIRME. Le scenario « il cible les deja-self-testes » est inatteignable : ceux-la ont fonction ET fixture |
| D-09 | L'etape 1 tait que les `delete_credential` sont sous `if self.use_keyring` | Les DEUX constructeurs de production posent `use_keyring: true` (keystore.rs:503-508, :517-522) et le cablage du panic wipe utilise `LocalFileKeyStore::new` (runtime.rs:440) : sur TOUT chemin de production le garde est vrai. La nuance est deja portee par keystore.rs:28. Ajouter une condition toujours vraie ajoute du bruit |
| D-10 | `hitl_approval.rs:118` dit « all five tests » alors qu'il y en a six | La phrase est une MESURE AU PASSE, faite avant l'existence du sixieme test — celui dans la rustdoc duquel elle vit. Verifie en outre que la mesure est SAINE. Le finding s'auto-annule : sa seconde branche decrit le dommage que sa premiere provoquerait |
| D-11 | Le « NEVER carries material » de `SecretError::Refused` enonce une discipline comme une propriete du type | Le « NEVER » est QUALIFIE quatre lignes plus bas, sur le champ meme : `secret.rs:121` « Why the host refused. **Material-free by contract.** ». La distinction est tenue ici comme partout dans le module (:19-21, :28-30) |
| D-12 | La surface `step_request_hash` du canari ne peut structurellement jamais rougir | « JAMAIS » est faux : elle rougit des que `step_request_hash` cesse de rendre un digest, classe qu'aucun autre membre ne couvre. Et le pin :580-582 enonce son objet — garder contre le RETRECISSEMENT de la table — il n'affirme pas que chaque membre est independamment discriminant |

**Contradiction inter-dimensions, arbitree.** Les dimensions C et D ont analyse
le MEME texte (`audit_plan:216`) et rendu des verdicts opposes : C-R9-7 le donne
P3 (notation ingrepable), D-01 le donne **P1** (suffixes inverses, routage qui
defait sa propre raison d'etre). Mesure d'arbitrage faite ce round :
`grep -nE '^\s*_[a-z_]+\(\)' scripts/check-frontier-contracts.sh` ne rend que
`369: _bp_extract_ts()` et `480: _hp_extract_rs()` ; les deux noms cites par
l'audit_plan n'existent nulle part ailleurs dans le depot (7 hits au total, 6
dans le script pour les deux fonctions reelles, 1 pour audit_plan:216). **Les
deux dimensions sont d'accord sur la SUBSTANCE** — les deux extracteurs non
self-testes sont les awk INLINE de `:385` (Rust/BRIDGE) et `:490-492`
(TS/HITL) ; elles divergent seulement sur la qualification. D-01 a raison sur le
fond : sous la convention `_{jambe}_extract_{langage}` du script lui-meme, les
noms cites designent correctement les deux pipelines anonymes, et la phrase dit
explicitement qu'ils « n'ont ni fonction nommee », donc elle n'est pas fausse.
C-R9-7 a raison sur le residu : deux transpositions a un caractere de symboles
EXISTANTS restent un piege de greppabilite pour l'auditeur S84. **Verdict
retenu : P3**, correctif = ajouter les ancres `:385` et `:490-492`. Le P1 de
D-01 est ecarte.

## Proprietes non gatees

Vraies par convention aujourd'hui, tenues par aucun controle mecanique. Aucune
n'est un P1 ; plusieurs sont des bornes assumees de §19.4 et le restent.

1. **Le compte des bornes de §19.4.** La jambe refuse deliberement d'en ecrire
   un (`:545-547`). C'est exactement par la que P1-1 est passe : rien dans le
   depot ne peut rougir sur « 26 » ecrit dans un rank-1.
2. **La liaison ordinal → contenu** (A-3). La jambe verifie la contiguite, pas
   le referent : insertion + renumerotation = silence, pendant que 13 citations
   de `bound 25` glissent, dont le runbook de duresse.
3. **Le jumelage des deux predicats `is_forbidden_in_identity`** (hash.rs ↔
   vault.rs) : revendique par trois textes, tenu par aucun controle. Deja route
   (audit_plan:206-211).
4. **Le jumelage des deux listes de 26 temoins** : byte-identiques aujourd'hui
   (mesure `diff` vide), rien ne l'empeche de diverger demain.
5. **DEL `U+007F` et les bornes C1 `U+0080`/`U+009F`** (D-05) : aucun temoin,
   donc une reecriture de `c.is_control()` en `matches!` explicite passe verte
   des DEUX cotes.
6. **Le garde de `SecretRef::new`** (D-06) : aucun rouge ne le tient.
7. **L'identite des deux tables de menace Argon2id** : aucune jambe ne les
   compare (verifie : `grep "Adversary reads"` sur le script = vide).
8. **La jambe §19.4 elle-meme n'est pas nommee dans le message de succes**
   (C-R9-8) : un gate vert ne distingue pas « la jambe a valide 27 bornes » de
   « la jambe a ete supprimee ».
9. **Bornes assumees de §19.4, non gatees par nature et non re-facturees ici** :
   la fenetre relecture→`rename` non verrouillee (borne 27, dette assumee,
   fermeture = verrou couvrant) ; la couche keyring INERTE (borne 25) ; l'absence
   d'API de destruction du coffre (borne 12) ; les orphelins de scratch du
   coffre (borne 26) ; la reprise best-effort des scratchs du keystore — dont le
   RESIDU n'est declare nulle part, ce qui est le P2 D-07, pas la borne.

## Trajectoire du dossier

r7 = 6 P1, r8 = 4 P1, **r9 = 2 P1**. Le dossier converge, et pas seulement en
comptage : les deux P1 restants sont des editions d'une ligne dans un rank-1,
sans code, sans suite a rejouer, la ou r7 corrigeait encore des predicats et r8
des jambes de gate. Le second signal est le taux de refutation, 8 soupcons sur
33 fermes par mesure (24 %, contre une poignee aux rounds precedents) et 12 des
25 findings retenus classes P3 : la couture est en train d'etre epuisee. **Une
CLASSE reste ouverte**, et c'est la meme depuis r7 : « claim chiffree ou
enumerative fausse dans un rank-1 », dont P1-1 est la forme la plus pure — le
depot s'est pourtant donne le remede a `:545-547` (ne pas ecrire de compte qu'aucun
gate ne recompte) sans l'appliquer a `llms.txt:97`. Les autres classes se
referment : la classe (a) « un seul des deux jumeaux » ne produit plus que du
P2/P3 documentaire (D-04, C-R9-5) apres avoir produit des divergences de code ;
la classe (c) « branche d'echec inatteignable » n'a AUCUNE instance ce round —
toutes les branches d'echec du gate ont ete atteintes par mutation, les deux
fail-open restants (C-R9-1) etant des trous de scan et non des branches mortes.
**Un round 10 large serait du rendement decroissant** ; un round 10 COURT et
strictement scope a re-mesurer les deux editions P1 plus le lot P2 batche a de
la valeur, a une condition : que les gestes ferment LA CLASSE et non l'instance
citee (A-2 doit toucher `persist` **et** le rustdoc de test ; D-05 doit poser
DEL **et** les deux bornes C1), faute de quoi le round 10 rouvrira mecaniquement
les memes lignes.

## Actions avant commit

**Bloquantes (2, texte seul, zero code) :**

1. `docs/workflow/llms.txt:97` — SUPPRIMER la parenthese chiffree : « The honest
   bounds (numbered) are §19.4 of THREAT_MODEL.md ». Ne pas ecrire « 27 » : le
   depot a deja tranche contre les comptes non gates a
   `check-frontier-contracts.sh:545-547`.
2. `docs/rust/PATTERNS.md:1581-1582` — remplacer « in the blob `flags` byte » par
   « in the blob header's `m_cost`/`t_cost`/`parallelism` fields (offsets
   24/28/32), that `encode_header` already writes from `KdfParams` — no manual
   step », en alignement sur :1195-1196.

**Fortement recommandees dans la meme passe (P2 batchables, ~20 lignes de texte
+ 2 lignes d'awk + 3 lignes de test) :**

3. `vault.rs` — qualifier `persist` (:696-698) **et** le rustdoc de test
   (:1469-1470) par la borne 27, puis corriger l'enumeration normative
   `THREAT_MODEL.md:2281-2284` pour qu'elle les nomme (A-2, fermer la classe).
4. `check-frontier-contracts.sh` — deplacer les regles de fence :579-580 AVANT
   la regle de titre :573, et ajouter `if (fence) print …` dans END :593-599
   (C-R9-1) ; gater la regle d'entree :569 sur `!seen` ou reformuler :584-586
   (C-R9-4) ; qualifier « sub-heading » en « sub-heading + inversion » a :556
   (C-R9-3).
5. `DURESS.md:29-31` — aligner §1 sur « les paragraphes §1 ET §5 — les DEUX »
   (D-04).
6. `vault.rs:306` et `:1152` — retirer les deux « 3-second » NEUFS de cette
   phase (B-3, portee reduite aux deux sites neufs ; les quatre sites S20 sont
   de la dette P3).
7. `hitl.rs` + `vault.rs` — ajouter aux DEUX listes, position identique, les
   temoins `"with\u{7F}delete"`, `"with\u{0080}c1-start"`, `"with\u{009F}c1-end"`
   (D-05).
8. `secret.rs::tests` — trois `is_err()` sur `SecretRef::new` (D-06).
9. `THREAT_MODEL.md` borne 26 + `DURESS.md` §5 + `panic.rs` etape 1 — declarer
   que la reprise des scratchs est BEST-EFFORT a echec avale (D-07).

**A router, pas a corriger ce round :** la liaison ordinal → contenu (A-3), qui
n'est PAS dans les quatre chantiers deja poses a `audit_plan:206-218` et doit y
etre ajoutee comme cinquieme.

**Optionnel (P3, batchable en fin de passe ou reporte) :** A-4, A-5, B-4,
C-R9-2, C-R9-5, C-R9-6, C-R9-7, C-R9-8, C-R9-9, D-02, D-03, D-08.

**Rappel de sequencement :** ce verdict couvre la review Claude uniquement. Le
round Codex de reference (`rounds/sprint83_phase_l1_codex_review_r1.md`) est
**PERIME** — il porte sur un arbre anterieur a huit rounds de correctifs. Un
Codex r2 frais est DU apres application des actions 1 et 2, et avant le commit :
le dossier a deja montre en Phase L que Codex trouve des P1 reels que les
reviews multi-agents manquent.
