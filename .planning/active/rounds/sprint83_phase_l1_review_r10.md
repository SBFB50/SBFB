# Review — Sprint 83 Phase L1 — Coffre de secrets et HITL hote (round 10, court)

## Verdict: PASS-PENDING

Ce round a re-mesure les NEUF gestes du round 9 et rien d'autre : les cinq correctifs de
texte (P1-1, P1-2, P2-3, P2-5, P2-6, P2-9), les deux correctifs de code (P2-7 temoins Cc,
P2-8 test neuf `secret.rs`), la reecriture de la jambe `§19.4 BOUND ORDERING` (P2-4), et la
consequence chiffree 44 -> 45. Resultat mesure : les huit correctifs hors-gate TIENNENT, y
compris la totalite de leur partie chiffree (offsets 24/28/32, bit unique du `flags`,
arithmetique Argon2, somme 45, 27 bornes contigues, 29 temoins par liste, tuple 15 champs) ;
les trois candidats de divergence chiffree que ce round a instruits — « 19 vs 20 tests
vault », « `subscriptions.json` vs `state.sqlite` », « enumeration de la borne 27 sans
`put` » — sont TOUS REFUTES a la mesure, dont un a sens inverse du finding. Le seul geste
qui a cree des defauts est P2-4, et ils sont tous dans le COMMENTAIRE et les MESSAGES de la
jambe de gate, jamais dans son verdict sur l'arbre courant (gate vert, et vert pour la bonne
raison : 27 bornes contigues 1..27, 0 fence dans §19.4). Verdict PASS-PENDING parce que
l'adjudication laisse **0 P1** : le P1 candidat de la dimension A (DURESS §1 sans la borne
best-effort) est tombe en P3 — aucune regle normative ne s'applique a la borne 26, le residu
est un gap PRE-EXISTANT a 2 membres non touche par r9 — et le P1 candidat de la dimension B
est tombe en P2, son « trou de couverture reel » etant refute par mesure. PASS-PENDING
signifie **review Claude OK, verdict Codex r2 a reconcilier** ; cela ne signifie PAS
« committable en l'etat » : les trois P2 restants portent sur la surface qui garde les
ordinaux cites par le runbook lu sous contrainte, et la mesure Docker du diff (659) est
ANTERIEURE aux correctifs r9.

## Perimetre du round

Round **COURT**, sur recommandation explicite du round 9, qui a juge qu'un round 10 LARGE
serait du rendement decroissant et qu'un round 10 strictement scope a la re-mesure de ses
propres correctifs n'aurait de valeur qu'a la condition de juger la fermeture de CLASSE et
non d'INSTANCE. Le perimetre est donc : les neuf gestes r9 nommes ci-dessus, la classe des
claims chiffrees/enumeratives qui les entoure, et les proprietes que ces gestes revendiquent.

**N'a PAS ete re-balaye** (deliberement, et donc non couvert par ce verdict) :

- l'ensemble du diff de la phase hors des surfaces touchees par r9 — `capability.rs`,
  `signal.rs`, `echo.rs`, `unlock.rs`, `Approvals.tsx`, `approvalsQueue.ts`,
  `hitl_approval.rs`, `keystore_integration.rs`, les deux `lib.rs`, `Cargo.toml` ;
- les huit autres jambes de `check-frontier-contracts.sh` (seule la jambe BOUND ORDERING,
  l. 539-632, a ete instruite) ;
- la couche front et ses suites ;
- les suites Rust et web, deja mesurees par le thread principal (cf. §Perimetre verifie) ;
- les findings deja routes vers `sprint83_audit_plan.md` par les rounds 8 et 9, notamment le
  jumelage non-gate des deux predicats `is_forbidden_in_identity` (audit_plan:212).

Lecture seule stricte sur toutes les dimensions : aucun `Edit`/`Write` dans le depot (hors ce
fichier), aucun git mutatif, aucune commande `cargo` ni `npm`, `set -o pipefail` sur chaque
appel avec `head`/`tail`.

## Etat des correctifs r9

| Geste r9 | Etat | Preuve (fichier:ligne, lue ou recomptee ce round) |
|---|---|---|
| **P1-1** llms.txt : parenthese chiffree supprimee | **TIENT** | `docs/workflow/llms.txt:97-100` — « The honest bounds (numbered) are §19.4 … No count is given here on purpose ». Factuellement vrai : recompte a la source, `§19.4` porte 27 items NUMEROTES, contigus 1..27 (`awk` sur 1911-2300 : `1.@1984 … 26.@2249 27.@2265`). Classe fermee : aucun autre compte de bornes §19.4 ailleurs dans le depot (dimension A, grep sur docs/ + crates/ + scripts/ + web/ + .planning/active/). |
| **P1-2** PATTERNS.md : params KDF dans le HEADER | **TIENT** | `docs/rust/PATTERNS.md:1581-1587` — « the KDF params live in the blob HEADER (`m_cost`/`t_cost`/`parallelism`, offsets 24/28/32) and `encode_header` writes them from `KdfParams` — … nothing to record in the `flags` byte, which carries a single bit (`use_keyring_layer`) ». Verifie a trois endroits du code : `keystore.rs:1169-1171` (`encode_header`), `:1211-1213` (`parse_blob`), `:104-114` (table du module doc) ; `flags` : `:108`, ecrit `0b0000_0001` seul (`:619`, `:790`, `:885`), relu `& 0b0000_0001` (`:1203`). Reserve de scope non bloquante : cf. §Proprietes non gatees, point 4. |
| **P2-3** borne 27 nomme `persist` et le test | **TIENT** | `docs/security/THREAT_MODEL.md:2285-2290` — « Enumeration NORMATIVE des sites qui la portent : `vault.rs` rustdocs de `bind`, `rotate`, `delete`, **de `persist`** … et **du test `delete_and_rotate_refuse_a_stale_handle_instead_of_answering_from_the_snapshot`** ». Les 5 sites `vault.rs` portent effectivement `bound 27 of §19.4` : 499, 588, 625, 707, 1825 — recompte, l'inventaire est EXACT, ni incomplet ni sur-large. |
| **P2-4** jambe BOUND ORDERING reecrite | **A CREE UN DEFAUT** | `scripts/check-frontier-contracts.sh:539-632`. Structure correcte (fences l. 573-575 AVANT le titre l. 582, entree gatee `!seen` l. 582, regle second titre l. 591-594, `END` fence l. 620-622) et les 8 mutations annoncees par r9 donnent bien les 8 resultats annonces. Mais **3 P2 + 2 P3** : etiquettes de mutation sur-larges (B-1), classe fence non fermee (B-2), deux messages neufs qui affirment plus que le code n'etablit (B-3), deux faux positifs fail-closed (B-4), provenance incomplete (B-5). |
| **P2-5** DURESS §1 aligne sur §5 | **TIENT** (residu P3) | `DURESS.md:24-31` (§1) et `:187-193` (§5) portent tous deux les DEUX exceptions d'ENSEMBLE (keyring inerte, coffre non reclame) et l'obligation croisee « §1 ET §5 — les DEUX ». C'est la classe que P2-5 visait, et elle est fermee. Le mode d'ECHEC n'est pas porte par §1 — residu re-cote P3, cf. finding A-P1-1. |
| **P2-6** « 3-second » -> « ~82 ms measured » | **TIENT** | `vault.rs:1161-1162` — « (~82 ms measured, not the "3 s" S20 design target) », et `vault.rs:306-309` pour le second site. `grep -rniE '3-second\|3 s/att\|3s/attempt\|3 secondes'` sur les 6 fichiers de la phase = 0 (dimension A). Les 2 seules lignes « 3 s » ajoutees dans le diff tracke (`Cargo.toml:272`, `keystore.rs:50`) portent « DESIGN TARGET » en majuscules avec la mesure citee dans le meme bloc. |
| **P2-7** 3 temoins aux DEUX listes | **TIENT** | `hitl.rs:974-980` et `vault.rs:1789-1795` — commentaire et trois litteraux byte-identiques, memes positions 27/28/29. Extraction mecanique des deux listes (dimension C) : **29 entrees / 29 entrees, LITERALS IDENTICAL IN ORDER**. Les trois sont bien refuses : `char::is_control()` = General_Category Cc = `0000..001F ∪ 007F..009F`, et les deux predicats commencent litteralement par `c.is_control()` (`hash.rs:162-163`, `vault.rs:846-847`). |
| **P2-8** test neuf `secret.rs` | **TIENT** | `secret.rs:317-338`. Discriminant : `SecretRef::new` (`:88-92`) contient exactement une expression faillible, `validate_identity_component("secret_ref", &name)?` — la retirer fait rougir 3 bras sur 4. Les trois bras refusants frappent les TROIS branches distinctes du validateur (`hash.rs:107` vide, `:112` trop long, `:119` caractere hors domaine) ; bras acceptant present (`:337`). |
| **P2-9** best-effort aux TROIS sites | **TIENT** | `THREAT_MODEL.md:2254-2257` (borne 26, « reclamation **BEST-EFFORT a echec avale** … rien ne le signale »), `DURESS.md:174-180` (« en BEST-EFFORT a echec avale … mais il SURVIT alors, silencieusement »), `panic.rs:16-19` (« BEST-EFFORT with a swallowed error … but it then SURVIVES, with no signal »). Les trois textes disent la meme chose. |
| **Consequence chiffree 44 -> 45** | **TIENT** | Recompte a la source ce round : `grep -c '^\s*#\[test\]'` = vault 20, hitl 11, secret 6, hitl_approval 6 (les quatre fichiers sont UNTRACKED, compte fait sur le CONTENU) ; keystore `git diff \| grep -c '^+.*#\[test\]'` = 2. Somme 20+11+6+6+2 = **45**, valeur du changelog v28 (`THREAT_MODEL.md:2696-2697`). Coherent avec la mesure du thread principal : 2429 − 2384 (fin Phase L) = +45. |

## Perimetre verifie

**Lu et re-verifie en synthese** (ancres relues une par une avant d'etre ecrites dans ce
rapport) : `scripts/check-frontier-contracts.sh:535-639` ; `docs/security/DURESS.md:15-39` et
`:165-199` ; `docs/security/THREAT_MODEL.md:2245-2299` et `:2694-2700` ;
`docs/workflow/llms.txt:94-129` ; `docs/rust/PATTERNS.md:1574-1591` ;
`crates/nexus-shell-daemon/src/panic.rs:10-34` ; `crates/nexus-core-rs/src/vault.rs:544-571`,
`:645-660`, `:1155-1170`, `:1780-1799` ; `crates/sbfb-workflow-engine/src/hitl.rs:968-985` ;
`crates/sbfb-workflow-engine/src/secret.rs:312-341`.

**Mesures rejouees en synthese** (lecture seule, `set -o pipefail`) :

- ordinaux de §19.4 recomptes par `awk` sur 1911-2300 : **27 bornes, contigues 1..27**,
  premiere l. 1984, derniere l. 2265 ;
- `grep -c '^[[:space:]]*```' docs/security/THREAT_MODEL.md` = **2** (parite paire) ;
  `grep -c '^[[:space:]]*~~~'` = **0** ;
- `grep -c '^\s*#\[test\]'` sur les 4 fichiers untracked = 20 / 11 / 6 / 6 ; keystore via
  `git diff` = 2 ; somme **45** ;
- `grep -n 'BEST-EFFORT' docs/security/DURESS.md` = **une seule ligne, la 174** ;
- `git diff --stat -- scripts/check-frontier-contracts.sh` = **269 insertions / 13
  suppressions** ; `git show HEAD:scripts/check-frontier-contracts.sh | grep -c 'BOUND
  ORDERING'` = **0** — la jambe est en insertion PURE, aucune version anterieure ne la porte ;
- `git status --porcelain` confirme `vault.rs`, `hitl.rs`, `secret.rs`, `hitl_approval.rs`
  **UNTRACKED** : tout compte les concernant a ete fait sur le CONTENU, jamais sur `git diff`.

**Vient du thread principal, non rejoue** (interdiction cargo/npm, et consigne explicite) :
Windows `cargo nextest run --workspace --locked --no-fail-fast` = **2429 passed / 0 failed /
7 skipped** ; `fmt` 0 ; `clippy` 0 ; doctests 10 blocs ; `bash scripts/check-frontier-contracts.sh`
exit 0 sous Windows ET sous Linux ; `bash -n` OK ; Docker `sbfb-ci` surface du diff = 659,
**mesure ANTERIEURE aux correctifs r9** ; web inchange (vitest 565/565, coverage
86.94/78.94/86.13/88.35, e2e 59 passed / 2 skipped).

**Vient de la dimension B, non rejoue en synthese** : les batteries de mutation sur la jambe
extraite verbatim (`sed -n '566,631p' scripts/check-frontier-contracts.sh`), 33 fixtures dans
`C:/Users/FlowUP/AppData/Local/Temp/claude/C--Users-FlowUP-Documents-Code-nexus/3c8a3c0a-0b2a-4a80-9597-5d803377106d/scratchpad/r10b/`
(`leg.sh`, `battery.sh`, `battery2.sh`, `fx/`, `fx2/`, `fx3/`) plus le harnais de contre-mesure
`r10ref/leg.sh`. Aucune ecriture dans le depot.

## Findings

### P1

**Aucun.** Les deux P1 candidats du materiel brut sont tombes a l'adjudication, tous deux par
mesure et non par arbitrage : le P1 de la dimension A (DURESS §1) en P3, le P1 de la
dimension B (troncature de queue) en P2. Les motifs de degradation sont ecrits dans les deux
findings correspondants ci-dessous.

### P2

**B-1 — les deux etiquettes de mutation « deleted bound » et « marker-to-bullet » sont
sur-larges** (`scripts/check-frontier-contracts.sh:556-557`).
Le commentaire revendique que « inversion, gap, deleted bound, renamed heading,
sub-heading-PLUS-inversion, and marker-to-bullet all turn this leg red ». Mesure : les deux
etiquettes ne tiennent qu'en instance MILIEU-DE-LISTE. Borne 27 entiere supprimee
(l. 2265-2292) → exit=0 ; bornes 26+27 → puces → exit=0 ; borne 1 supprimee (l. 1984-1986) →
exit=0 ; 23 bornes → puces (4 restantes) → exit=0. Temoins discriminants : borne 25 du MILIEU
supprimee → exit=1 « bound 24 is followed by bound 26 at line 2203 » ; 27 renumerotee 99 →
exit=1. r8 n'avait mesure ces deux etiquettes qu'en instance milieu et en instance masse ; le
kill a bien eu lieu, c'est la generalisation qui est fausse. C'est la classe (b) de la lecon,
jouee dans un commentaire de script.
*Ce qui est REFUTE dans le finding d'origine, et pourquoi il n'est pas P1* : le « trou de
couverture reel » n'existe pas. L'invariant que la jambe ENONCE (l. 541-543, « Only the FIRST
marker counts: a renderer renumbers the rest sequentially ») n'est viole par AUCUNE des
mutations vertes : ordinaux survivants recomptes 1..26 (queue supprimee), 2..27 (tete
supprimee), 1..25 (queue bulletisee) — tous contigus depuis le premier marqueur, donc
CommonMark rend chaque borne survivante sous SON ordinal et toute citation qui survit resout
encore juste. Le second anchor est refute aussi : `scanned < 2` (l. 615) n'est pas un
« garde-fou de volume » mais la garde de VACUITE prescrite mot pour mot par r8, et son message
(l. 616) est vrai sur son domaine N∈{0,1}. Aucune propagation rank-1 (grep `deleted
bound|marker-to-bullet|BOUND ORDERING` = 0 hit hors `.planning/active/rounds/`).
*Fermeture de CLASSE* : scoper l'etiquette (« middle-of-list deletion / bulletisation ») ET
ecrire la borne assumee — troncature de tete, troncature de queue et renumerotation CONTIGUE
ne sont pas couvertes, par construction. Le vrai angle mort, que B-1 ne nommait pas : borne 25
supprimee PUIS 26→25 et 27→26 → exit=0, ordinaux 1..26, et les 7 citations vivantes de
« bound 27 » (`vault.rs:499/:588/:625/:707/:1825`, `docs/workflow/llms.txt:112`,
`.planning/active/sprint83_gate_matrix.md:44`) pointent alors sur la borne scratch-keystore.
La jambe delegue explicitement ce cas a l'humain (l. 629, « move the block, do not renumber »).

**B-2 — le geste « fences en premier » ferme l'INSTANCE backtick, pas la CLASSE fence
CommonMark** (`scripts/check-frontier-contracts.sh:569-575`).
Les trois regles (l. 573, 574, 575) ne reconnaissent que ```` ``` ````, alors que le
commentaire l. 569-570 affirme « inside a fenced block, NOTHING else is markup » sans
qualifier — et `~~~` est une fence CommonMark de plein droit, dont le depot ecrit deja
(`docs/archive/BENCHMARK-SCENARIO.md:392/:414/:439`). Mesure : fence `~~~` contenant `##
Titre` inseree l. 2203 PLUS inversion 26↔27 apres → **exit=0, sortie vide** (l'inversion
masquee) ; jumeau strictement identique en fence backtick → exit=1 avec les deux ruptures
nommees. Symptomes fail-closed du meme defaut : `~~~` contenant `1.` → faux positif ; `~~~`
contenant `### 19.4` → faux positif « SECOND heading ». C'est la classe (a) dans sa forme
canonique, et c'est le finding le plus solide du lot parce qu'il touche l'invariant ENONCE.
Non rehausse en P1 : `grep -c '^[[:space:]]*~~~' docs/security/THREAT_MODEL.md` = **0**
recompte ce round, donc aucun faux vert sur l'arbre courant — le fail-open exige une edition
future.
*Fermeture de CLASSE* : elargir a `/^[[:space:]]*(```|~~~)/` ne suffit PAS. Une fermeture
CommonMark doit matcher le caractere OUVRANT et etre au moins aussi longue : il faut memoriser
le delimiteur, sans quoi une fence `~~~` se fermerait sur des backticks.

**B-3 — les deux diagnostics NEUFS affirment une consequence que le code n'etablit pas**
(`scripts/check-frontier-contracts.sh:586-594` et `:618-622`).
(a) La regle l. 591-594 fait `next` SANS poser `inside = 0` : elle PREEMPTE la fermeture
l. 596-600, donc le scan CONTINUE. Mesure : doublon `### 19.4` insere l. 2203 PLUS inversion
apres → la MEME sortie imprime « a SECOND §19.4 heading at line 2203 truncates the scan — the
bounds after it were never examined » PUIS « bound 25 is followed by bound 27 at line 2250 »
et « bound 27 is followed by bound 26 at line 2266 ». Le diagnostic est refute par les deux
lignes qu'il precede, dans le meme run ; le commentaire l. 586-588 decrit l'etat
PRE-correctif, jamais recontrole apres. Aggravant : quand le doublon tombe APRES la fin de
section (`inside==0`), la regle rougit quand meme — faux positif net. (b) Fence non terminee
ouverte l. 2400, hors §19.4 (section 1911→2299) → exit=1 « part of §19.4 was never examined »
alors que l'instrumentation du meme awk donne `scanned=27`, derniere borne lue 27 : les 27
bornes ONT toutes ete examinees, l'imputation est fausse.
Non rehausse en P1 : les deux sont fail-CLOSED (gate rouge, jamais un vert silencieux) ; le
prejudice est l'operateur envoye au mauvais endroit sur un echec reel — classe que r8 cotait
deja P2. Deux correctifs d'une ligne : « a SECOND §19.4 heading at line N — the scan
continues, the ordinals below are still checked » ; et n'imputer §19.4 que si la fence s'est
ouverte alors que `inside` valait 1 (memoriser `fence_line = NR` et comparer aux bornes de
section).

### P3

**A-P1-1 (re-cote de P1 a P3) — DURESS §1 ne porte pas le mode d'echec best-effort**
(`docs/security/DURESS.md:20-31`).
Le constat de fait est vrai et re-verifie : §1 ecrit « destruction irreversible des secrets
daemon ENUMERES ci-apres — blobs identite normal + duress **et leurs deux scratchs
`.enc.tmp`**, subscriptions, blob cache », et `grep -n 'BEST-EFFORT' docs/security/DURESS.md`
ne rend QU'UNE ligne, la 174 (§5). Ce qui detruit le P1 : (1) aucune regle normative
n'existe pour la borne 26 — la clause « Les textes qui decrivent X doivent la qualifier par
CETTE borne » + « Enumeration NORMATIVE des sites » n'apparait QUE dans la borne 27
(`THREAT_MODEL.md:2284-2290`) ; la borne 26 (`:2249-2264`) n'en a aucune, et §1 ne DECRIT pas
la reclamation, il liste des cibles. (2) La regle editoriale de §1 est ecrite dans §1 :
« Deux precisions que le mot « tous » cachait » = appartenance a l'ENSEMBLE, et les deux
exceptions d'ensemble SONT mirrorees §1/§5 — la classe visee par P2-5 est fermee. (3) Le
« 3 des 4 jumeaux » ne tient pas : §1 ET §5 laissent tous les deux `subscriptions` et
`blob-cache/` non qualifies alors qu'ils sont best-effort (`panic.rs:231` `.is_ok()`, `:238`
`.is_ok()`) — gap PRE-EXISTANT a 2 membres, intouche par r9. (4) Le residu est delibere,
documente en §5 + `panic.rs:16-19` + borne 26, ET teste
(`keystore.rs:1760 an_unremovable_scratch_does_not_abort_the_wipe`). (5) Un fait du finding
est faux : il nomme `state.sqlite` parmi les cibles de §1 ; §1 dit « subscriptions ».
(6) Aucune conduite operateur alternative n'existe sous contrainte.
*Fermeture de CLASSE si on y touche* : une incise en §1 **ET** §5 couvrant les TROIS
best-effort (scratchs + subscriptions + blob-cache), pas les seuls scratchs — sinon on rejoue
l'instance.

**B-4 — deux faux positifs de la regle de marqueur** (`scripts/check-frontier-contracts.sh:601`).
Borne indentee de 3 espaces → exit=1 « bound 24 is followed by bound 26 » ; ligne de prose
« 2026. Le texte continue… » → exit=1 avec deux ruptures fantomes. Les deux sont fail-CLOSED
(bruit, jamais un vert). **Motivation corrigee, et c'est le point important** : le defaut tel
qu'enonce par le finding (« asymetrie d'indentation ») est faux comme diagnostic, et le geste
qu'il suggerait creerait un defaut. La colonne 0 de la l. 601 est PORTANTE : un bloc de code
indente de 4 espaces contenant « 1. exemple » insere dans §19.4 laisse la jambe MUETTE
precisement grace a elle ; relacher a `^[[:space:]]*` ferait de ce bloc — et de toute
sous-liste ordonnee imbriquee — des ordinaux fantomes. Le residu reel se reduit a la bande
1-3 espaces et au cas « N. » en prose, que CommonMark ne rend meme pas en item.

**B-5 — provenance incomplete et liste de kills incomplete**
(`scripts/check-frontier-contracts.sh:552-560`).
La l. 552 dit « REWRITTEN in round 8 » alors que le corps cite deux fois le round 9 (l. 572,
l. 588) et que la jambe vient d'etre reecrite une DEUXIEME fois ; et la liste ne contient
aucune des deux mutations que les regles neuves ferment (fence non terminee, second titre
§19.4), toutes deux rouges et rejouees. Degrade a P3 pour deux raisons : la phrase n'est pas
FAUSSE, seulement incomplete (la jambe A bien ete reecrite au round 8, et le lecteur apprend
l'existence du round 9 deux fois dans le corps qu'il lit) ; et elle n'est pas actionnable
separement — le site de correctif est le MEME bloc que B-1.

## Findings refutes

Huit soupcons ont ete fermes par mesure, plus les deux re-cotations ci-dessus.

- **A-P2-1** (« l'enumeration normative de la borne 27 rate `put` »). Les deux enumerations
  mesurent des grandeurs differentes : `THREAT_MODEL.md:2286-2290` enumere les SITES QUI
  PORTENT la borne, pas les points d'entree degrades — et l'inventaire est EXACT, `bound 27 of
  §19.4` apparait 5 fois dans `vault.rs` (499, 588, 625, 707, 1825), exactement les 5 sites
  listes. La regle est conditionnelle (« les textes qui DECRIVENT la garde ») ; le rustdoc de
  `put` (`vault.rs:550-554`, relu en synthese) ne decrit pas la garde mais la zeroisation du
  buffer appelant, et `put` n'appelle jamais `refuse_if_stale` lui-meme : il delegue a
  `self.persist()` (`vault.rs:566`), dont le rustdoc porte la borne et la declare NEE la
  (`:703-707`). Les deux rank-1 invoques contre l'enumeration portent deja `put` explicitement
  (`llms.txt:109-118`, `gate_matrix:44`).
- **A-P2-2** (« `refuse_if_stale` et `epoch()` echappent a la regle »). Le contrat de
  `refuse_if_stale` (`vault.rs:647`) est exactement vrai sans exception : apres la collision
  de la borne 27, l'epoch du disque n'a justement PAS depasse celui du handle. Le finding
  tronque la citation — « One helper for the whole class » est qualifie dans la MEME phrase
  (`:649-652`) et designe la classe des sites d'appel MANQUANTS. `refuse_if_stale` est PRIVE
  (`vault.rs:654`, `fn` et non `pub fn`), donc le « nouvel appelant » du scenario ne peut
  naitre que dans le meme fichier ; et `llms.txt:118-119` pre-empte mot pour mot le mesusage :
  « Read the guarantee as "re-reads the epoch", never as "never answers from a stale snapshot" ».
- **A-P3-1** (« 19 vs 20 tests vault »). Les trois textes ne comptent pas la meme grandeur :
  `vault.rs:1155-1162` compte les tests restes VERTS sous une mutation, `gate_matrix:44` et
  `THREAT_MODEL.md:2697` comptent les tests du module aujourd'hui. Le 20e test EST
  `production_constructor_uses_production_kdf_params` (`vault.rs:1164`), cree PARCE QUE la
  mutation avait survecu : au moment de la mutation il n'existait pas. La phrase est vraie
  sous les deux lectures, et l'edition redoutee 19 → 20 la rendrait fausse.
- **A-P3-2** (« §5 nomme `subscriptions.json`, le code supprime `state.sqlite` »). Finding
  INVERSE : `DURESS.md:185` a raison et `panic.rs:26-27` porte le nom trompeur. En production
  `PanicWipeService::new` recoit `opts.paths.subscriptions_json.clone()` comme `state_db_path`
  (`runtime.rs:441-443`, avec commentaire explicite `:434-437`) ; le chemin resolu est
  `<root>/shell-daemon/subscriptions.json` (`config.rs:140`, `:152`). `state.sqlite` n'apparait
  que dans deux cablages de TEST et dans le module-doc. Le residu reel est un P3
  pre-existant sur `panic.rs:26-27`, de sens oppose au finding, hors perimetre.
- **C-1** (« la justification des trois temoins nomme une mutation deja tuee »). Refute sur
  deux axes. Le texte relu en synthese (`hitl.rs:974-977` == `vault.rs:1789-1792`) dit « …
  passait SUR DEL ET SUR TOUT C1 » : la phrase enonce le COMPORTEMENT de la reecriture sur
  DEL/C1, pas un verdict de suite, et elle est vraie. Et la premisse est fausse pour la
  famille qui a motive le geste : la mutation que r9 a effectivement mesuree est
  `matches!(c, '\0'..='\u{1F}' | '\u{80}'..='\u{9F}')`, sous laquelle le temoin pre-existant
  `\u{0085}` reste VERT (0x85 ∈ 0x80..=0x9F) tandis que DEL devient acceptable —
  `with\u{7F}delete` en est le SEUL tueur. Second contre-exemple mesure :
  `c.is_whitespace() || c < ' '` laisse 0085 muet (U+0085 a White_Space=Yes) et n'est rouge que
  par les trois neufs.
- **C-2** (« "both sides pin the set" est faux »). Hors perimetre — `vault.rs:838` n'est aucun
  des neuf gestes r9, et r9 a LU cette zone et l'a validee. La propriete est une borne DEJA
  declaree et DEJA routee (`sprint83_audit_plan.md:212`, remede structurel
  `const IDENTITY_FORBIDDEN_RANGES` propose par r8). Et la direction non couverte est
  fail-CLOSED : ajouter une plage a UN seul jumeau le rend plus STRICT (mismatch de
  disponibilite), jamais un contournement d'autorite ; la direction dangereuse — PERDRE une
  couverture — est couverte des deux cotes, chacun des 11 patrons portant au moins un temoin
  et les 9 plages multi-points leurs deux bornes.
- **C-3** (« "007F DEL ferme C0" est terminologiquement faux »). Convention-dependant : la
  table de codes Unicode intitule le bloc U+0000..U+007F « C0 Controls and Basic Latin », et
  `char::is_ascii_control()` en Rust vaut `'\0'..='\x1F' | '\x7F'`, rangeant DEL du cote C0.
  La claim operante est vraie (couverture resultante : 0000/001F et 007F/009F). Les deux
  branches du scenario sont mortes : `0080` est deja litteralement deux lignes plus bas, et
  supprimer `007F` supprimerait le seul tueur de la mutation r9.
- **C-4** (« le test neuf n'asserte pas le `kind` »). `is_err()` est pleinement discriminant
  ici : `SecretRef::new` n'a qu'une expression faillible. Le second scenario est un no-op
  semantique — intervertir les branches 1 et 2 de `validate_identity_component` ne change rien
  (`is_empty()` et `len() > 512` sont mutuellement exclusifs). Et « aucun test du depot
  n'epingle le `kind` » est faux : `hitl.rs:815-876` boucle sur les ONZE positions et asserte
  `assert_eq!(&reported, kind, …)`, avec le rationale ecrit in situ ; l'asymetrie est justifiee
  par la forme (11 composantes = il faut prouver laquelle a refuse ; 1 composante = rien a
  desambiguer). `kind` n'alimente que le `#[error("invalid {kind}: {reason}")]`.

### Contradictions entre dimensions

Ecrites explicitement, comme demande :

1. **Dimension A contre l'adjudication.** Les notes de la dimension A concluent « un seul P1, a
   correction triviale » et « deux P2 » ; l'adjudication a degrade le P1 en P3 et refute les
   deux P2. La dimension A a donc livre **0 P1 et 0 P2** apres mesure. Le motif est le meme
   pour les trois : la dimension a traite comme normative une regle qui ne l'est que pour la
   borne 27, et a lu comme une seule enumeration deux enumerations de grandeurs differentes.
2. **Contradiction factuelle directe.** La dimension A ecrit, en preuve de A-P3-2, que
   `grep -rn subscriptions.json crates/` rend **0 resultat** ; l'adjudication mesure de
   nombreux hits (`config.rs:110/140/152`, `runtime.rs:357/434/443/4488`,
   `blob_serve_http.rs:177`, `iroh_runtime.rs`, `paths.rs`). C'est la seule preuve
   demonstrativement fausse produite par une dimension ce round, et elle inversait le sens du
   finding.
3. **Dimension C contre l'adjudication.** Les notes de la dimension C declarent C-1 « le seul
   defaut cree par r9 dans cette dimension » ; C-1 est refute. La dimension C a donc livre
   **0 defaut** : les deux correctifs de code r9 sont fonctionnellement corrects et
   discriminants, et ses quatre findings sont des nits de redaction, tous fermes.
4. **Dimension B contre elle-meme.** B-1 revendique un « trou de couverture REEL » ; la mesure
   montre que l'invariant enonce par la jambe n'est viole par aucune mutation verte, et exhibe
   un angle mort DIFFERENT (renumerotation contigue) que B-1 ne nomme pas. B-4 enonce un
   defaut (« asymetrie d'indentation ») dont le correctif propose creerait le fail-open que la
   colonne 0 empeche aujourd'hui.
5. **Dimension C nuance dimension A.** La dimension A cloture P1-2 comme exact (il l'est) ; la
   dimension C ajoute que §P29 n'est scope au blob keystore que par la commande de bench en fin
   de section, alors qu'un second `encode_header` existe pour le blob vault avec des offsets
   differents. Les deux affirmations sont compatibles : cf. §Proprietes non gatees, point 4.

## Proprietes non gatees

1. **Le jumelage des deux predicats `is_forbidden_in_identity`** (`hash.rs:162-186` ↔
   `vault.rs:846-868`) : revendique par trois textes, tenu par aucun controle machine. Etat
   courant sain — comparaison mecanique des deux ensembles de code points ce round :
   **PREDICATE SETS IDENTICAL** (20 points chacun, meme ordre). Deja route
   (`sprint83_audit_plan.md:212`, remede `const IDENTITY_FORBIDDEN_RANGES` propose par r8).
2. **Le jumelage des deux LISTES de temoins** (`hitl.rs` ↔ `vault.rs`) : 29 entrees identiques
   en ordre aujourd'hui, verifie mecaniquement, mais aucune jambe de gate ne le recompte.
3. **La completude de la liste des bornes §19.4** : la jambe BOUND ORDERING ne prouve que la
   contiguite de ce qu'elle VOIT. Troncature de tete, troncature de queue et renumerotation
   contigue ne sont pas couvertes, par construction, et sont deleguees a l'humain
   (`check-frontier-contracts.sh:629`). Le patron auto-verifiant existe pourtant dans le meme
   fichier : `DOMAIN_CENSUS_FROZEN` (l. 230), qui `diff` une liste gelee contre la liste vivante.
4. **Le scope de §P29 de `PATTERNS.md`** : les offsets 24/28/32 valent pour le blob KEYSTORE ;
   le blob VAULT a son propre `encode_header` (`vault.rs:868`) avec des params KDF aux offsets
   7/11/15. §P29 n'est scope au keystore que par la commande de bench en fin de section — un
   lecteur qui transpose obtient de mauvais offsets. Non bloquant (la phrase corrigee par P1-2
   est exacte dans son scope), mais aucun gate ne l'empeche.
5. **L'etiquette de domaine `"secret_ref"`** (`secret.rs:90`) n'est epinglee par aucun test.
   Sans consequence d'autorite (le `kind` n'alimente que le message d'erreur), a la difference
   des ONZE positions de `hitl.rs` ou l'assertion existe et est portante.
6. **Les comptes de la phase** : 45 tests, 29 temoins, 27 bornes, 15 champs du tuple HITL, 5
   sites portant la borne 27. Aucun gate ne les recompte ; ils sont recomptes a la main a
   chaque round, et ce round les a tous retrouves exacts.

## Fermeture du dossier

Trajectoire : **r7 = 6 P1, r8 = 4 P1, r9 = 2 P1, r10 = 0 P1** (3 P2, 3 P3, 8 refutations sur
14 findings bruts). Sur les trois classes de la lecon dominante : **(b) claim chiffree ou
enumerative fausse dans un rank-1 est FERMEE** — les trois candidats instruits ce round sont
tous refutes, l'un a sens inverse, et la totalite des chiffres verifiables tient ; elle
survit uniquement HORS rank-1, dans le commentaire d'un script (B-1). **(a) geste applique a
un seul des deux jumeaux est FERMEE sur les jumeaux de code** (les deux listes sont
byte-identiques, mesure) mais **a reparu sous l'axe instance-vs-classe** dans le gate (B-2 :
fence backtick fermee, fence CommonMark non). **(c) branche d'echec inatteignable est FERMEE
dans le code** (les bras `Ok(false)`/`Err(UnknownRef)` de `vault.rs:1850/:1863` sont des
detecteurs de mutation deliberes, avec catch-all) mais **a produit une variante dans le gate**
(B-3(a) : un message dont la consequence enoncee est refutee par le run qui l'imprime).
Autrement dit, les trois classes ont migre du livrable vers l'outil qui le garde. **Un round 11
n'aurait PAS de valeur, franchement** : les trois P2 sont sur UN SEUL fichier non-Rust,
tous fail-closed ou latents-a-0-occurrence, et deux d'entre eux plus les deux P3 de gate
touchent le MEME bloc `:552-560` et le MEME `END` — c'est un geste, pas un round. Rejouer une
review complete pour re-mesurer ce geste, c'est exactement la boucle auto-alimentee que 9
rounds ont documentee. La sortie est : appliquer les correctifs en UN passage, rejouer les
8 mutations connues plus les 4 nommees ici (fence tilde, doublon hors-section, fence
non-terminee hors-section, renumerotation contigue), et passer la main a Codex.

## Actions avant commit

**Aucun P1. Rien ne bloque le commit du point de vue de cette review.**

Optionnel, par ordre de rendement — a jouer en UN passage sur DEUX surfaces, pour ne pas
rejouer la boucle :

1. *Surface 1, un seul bloc* (`check-frontier-contracts.sh:552-560` + `:586-594` + `:618-622`) :
   scoper les deux etiquettes de mutation en « middle-of-list deletion / bulletisation » et
   ecrire la borne assumee (B-1) ; corriger les deux messages neufs pour dire ce que le code
   fait (B-3) ; mettre la provenance au round reel et ajouter les deux kills manquants (B-5).
   Idealement, ancrer la FIN de liste avec le patron `DOMAIN_CENSUS_FROZEN` (l. 230) — un
   compte ecrit DANS le gate est auto-verifiant, ce que le compte en prose retire par P1-1
   n'etait pas.
2. *Surface 2, isolee* (`check-frontier-contracts.sh:573-575`) : reconnaitre la fence `~~~` en
   memorisant le caractere OUVRANT, pas seulement en elargissant l'alternative de motif (B-2).
3. *Facultatif* : incise best-effort en `DURESS.md` §1 **ET** §5 couvrant les TROIS cibles
   best-effort — scratchs, subscriptions, blob-cache — jamais les seuls scratchs (A-P1-1).
4. *Facultatif, fail-closed pur* : borne de plausibilite sur l'ordinal (l. 601) contre le cas
   « 2026. » en prose. **Ne PAS relacher la colonne 0** — elle est portante (B-4).
5. *Non-finding, controle de forme* : la mesure Docker `sbfb-ci` du diff (659) est ANTERIEURE
   aux correctifs r9. Le seul correctif de code r9 est +1 test dans `secret.rs` et +3 litteraux
   dans deux listes de temoins, sans `cfg` ni I/O — l'attendu derive est 660, mais il n'a pas
   ete mesure et ce round ne peut pas le mesurer (interdiction cargo).
