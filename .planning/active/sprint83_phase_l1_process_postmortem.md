# Post-mortem de process — Sprint 83 Phase L1

Objet : le SYSTÈME DE TRAVAIL, pas ses exécutants. Aucune affirmation de ce
document n'impute un défaut à une personne : les six défauts hérités qui dominent
le dossier datent de deux commits de 2026-04, dans un process qui n'avait alors
ni hook pre-commit, ni gate Codex, ni gate frontière (dates au §2).

Convention de preuve : **[M]** = mesuré de première main le 2026-07-28 sur ce
dépôt (commande rejouable citée) ; **[I]** = inféré à partir de faits mesurés,
avec le raisonnement explicite ; **[B]** = repris du brief de mission sans
re-dérivation nominative.

Note de fraîcheur, importante pour relire les ancres : les quatre dimensions de
mesure ont travaillé sur un arbre NON COMMITÉ. La phase a été commitée depuis,
`8199c50a` du 2026-07-28 16:19:52 +0200 [M : `git log -1 --format="%h %ad"`].
`git status --porcelain` ne rend plus que `?? assets/factory/` et `?? docs/book/`
[M]. Toutes les ancres décrites comme « arbre de travail » par les dimensions
sont donc désormais des ancres HEAD.

---

## Ce qui s'est passé, en chiffres

- **15 rounds** : 10 review interne + 5 Codex [M : `ls .planning/active/rounds/`
  → `..._review_r1..r10.md` + `..._codex_review_r1..r5.md`], du 2026-07-25 12:38
  (r1) au commit `8199c50a` du 2026-07-28 16:19:52 — **3 j 3 h 42 min**.
- **P1 bruts : 48** = 38 en review interne (trajectoire 8, 9, 3, 0, 1, 5, 6, 4,
  2, 0 [M : en-têtes `### P1` des rounds 1-6 ; comptage de table pour 7-10]) +
  10 côté Codex (3, 3, 3, 1, 0 [M : body du commit `8199c50a`, § « Codex
  verification »]). **19** est le compte APRÈS déduplication/adjudication
  inter-rounds [B] — ce n'est pas le même dénominateur, les deux sont exacts.
- **Répartition des 19** [B] : 14 affirmations fausses dans des documents de
  référence, 3 défaillances du garde-fou automatique, 1 propriété de sécurité
  qu'aucun contrôle ne gardait, 1 vrai bug de production.
- **21 défauts créés par un correctif du round précédent** [M : occurrences de
  « A CREE UN DEFAUT » par round = 0, 0, 0, 4, 5, 3, 3, 3, 2, 1].
- **Ancienneté du seul bug de production** : introduit le 2026-04-17
  (`c32ecb35`), trouvé le 2026-07-28 — **102 jours**, ~63 sprints ouverts [M].

---

## Quand chaque défaut a été écrit

| # | Défaut | Date + sprint | Origine | Étape qui aurait dû l'attraper | Pourquoi elle ne l'a pas fait |
|---|---|---|---|---|---|
| 1 | `PATTERNS.md` « ~83 s pour un PIN à 6 chiffres » (faux d'un facteur 988), écrit dans DEUX paragraphes | 2026-04-16, S20 A, `05271fa4` [M : `git log -S"83 s" -- docs/rust/PATTERNS.md`] | HÉRITÉ | Audit gate, Track K (seul contrôle dont le périmètre nomme `PATTERNS.md`) | Track K constate la PRÉSENCE d'une entrée d'index, jamais la VÉRITÉ d'un chiffre déjà présent (README:2154 « frontière neuve non indexée = P1 ») |
| 2 | Jumeaux Argon2id contradictoires : `keystore.rs` « 3 s/attempt » vs `PATTERNS.md` « ~80 ms/attempt », facteur 37,5 | 2026-04-16, S20 A, `05271fa4` — les DEUX cellules, NÉES divergentes | HÉRITÉ | Review, cohérence des jumeaux ; à défaut le gate frontière (il sait faire du lockstep) | Aucun registre des paires à maintenir en lockstep ; le gate ne compare que les paires codées en dur |
| 3 | `secure_unlink_blob` sonde avec `Path::exists()` (erreur d'accès → « absent ») : blob présent-mais-illisible sauté, `wipe_all` rend `Ok` | 2026-04-17, S20 B, `c32ecb35` [M : `git blame -L 1018 <keystore.rs>` à HEAD~1] | HÉRITÉ | Review S20 B (sûreté du chemin de destruction), puis chaque audit gate | Faux-négatif SILENCIEUX (suite verte) + le mensonge est un cran SOUS la garantie annoncée : le call-site propage bien avec `?` |
| 4 | Deux appels supplémentaires à `secure_unlink_blob` (scratchs `.enc.tmp`), qui triplent l'exposition au défaut n°3 | S83 L1, correctif du round 5 | CRÉÉ PAR UN CORRECTIF | Review adversariale du correctif (mandat explicite du round 8 : correctifs « traités comme SUSPECTS ») | La review lit le DIFF ; le diff contient les deux lignes neuves, pas le corps de la primitive réutilisée |
| 5 | `DURESS.md` §1 « destruction irréversible de tous les secrets daemon » — fausse sur trois étages | 2026-04-17, S20 B, `c32ecb35` ; fichier à UN SEUL commit avant L1 [M : `git log --follow -- docs/security/DURESS.md`] | HÉRITÉ | Audit gate (dérive HARDENING) et préflight L1 S3 | Dérive PASSIVE : le document devient faux parce qu'on ajoute du code qu'il ne mentionne pas — aucun déclencheur du process ne réagit à cet événement |
| 6 | `DURESS.md` ligne NEUVE « ~82 ms → ~83 s » : le chiffre faux recopié dans le document qui porte désormais toute la défense at-rest | S83 L1, correctif documentant la borne 25 | CRÉÉ PAR UN CORRECTIF | Review adversariale du correctif, dimension arithmétique (elle l'a fait, au round 7) | Le process valorise la propagation cohérente d'un chiffre et ne distingue nulle part « propager » de « re-dériver » |
| 7 | Commentaire de `os_keyring_available()` imputant l'échec à « certain Windows user profile configurations » — la vraie cause est `keyring` déclaré sans feature de backend (builder MOCK) | 2026-04-16, S20 A, `05271fa4` | HÉRITÉ | Préflight S1b (graphe transitif `Cargo.lock` + features) | Falsifiable en une commande, mais protégé par la VRAISEMBLANCE : version exacte citée (`3.6.3`) + symptôme environnemental plausible, dans un dépôt qui documente de vraies instabilités d'env |
| 8 | `Cargo.toml` racine : « Used as the *second* layer » à 9 lignes de `keyring = "3.6"` (sans backend) | 2026-04-16, S20 A, `05271fa4` | HÉRITÉ | Review dimension 6bis (« qui LIT cette primitive ? ») ; préflight S1b | Aucun contrôle ne considère un manifeste de build comme une surface documentaire : le gate scanne `find crates web/src`, `docs/` est exclu « par construction » |
| 9 | Self-test de non-vacuité TAUTOLOGIQUE de la jambe de parité (branche d'échec inatteignable), recopié ensuite par L1 et cité comme preuve dans le threat model | 2026-07-19, S83 C, `8252a452` | CRÉÉ PAR UN CORRECTIF (de la Phase C) | Review de Phase C (qui a prescrit le geste) ET Codex de Phase C (qui a coté « HOLDS ») | Un contrôle ajouté POUR prouver la discrimination n'est soumis à aucune obligation de discrimination : la récursion n'est pas fermée |
| 10 | 3 P1 fail-open dans `check-frontier-contracts.sh` (dont une jambe NEUVE verte sur quatre mutations) | S83 L1, rounds 7-8 | CRÉÉ PAR UN CORRECTIF | Review dimension 5 (couverture de branches) ; hook | Un script de gate n'est traité comme du code livrable par aucun canon : ni mutation exigée, ni suite de tests pour `scripts/` |
| 11 | `BoundSecret` sans `Serialize` : propriété de sécurité qu'aucun contrôle ne gardait (le canari restait vert avec un `impl Serialize` fuyant) | S83 L1 | NÉ EN L1 | Le canari de la phase lui-même | Le canari couvrait 5 surfaces d'exécution, dont aucune ne sérialise le type ; la propriété est portée par le TYPE, elle exigeait un `compile_fail` |
| 12 | Canon d'audit : `## 10 Tracks` pour 11 tracks A..K | 2026-07-02, `a6b4ca41` [M : `grep -c "^### Track"` = 11 ; `grep -n "^## .* Tracks"` = `39:## 10 Tracks`] | CRÉÉ PAR UN CORRECTIF | L'audit gate lui-même (Track I meta-process) | Exactement la classe étudiée ici, appliquée au process : un nombre affirmé que rien ne recompte. Encore faux au 2026-07-28, 26 jours après |
| 13 | Synthèse de review r1 : « la Phase L, trois semaines plus tôt » — écart réel ~19 h | 2026-07-25, S83 L1 | NÉ EN L1 | Aucune | Le process gate les LIVRABLES, jamais ses propres organes de vérification : aucun contrôle factuel sur un artefact de review |

---

## Ce que le process prévoyait

**PRÉFLIGHT (G8, 5 scans).** Les cinq scans sont PROSPECTIFS. Le tableau du canon
donne comme sortie attendue de S3 une « Matrix : primitive proposée → threats
T0-T5 couverts vs non-couverts » et de S4 une « Liste invariants » — deux sorties
sur l'objet À VENIR (README §6.9, l. 1823-1831). Le skill le dit sans détour :
« S3 est un gate check de contrainte, pas un scan de decouverte »
(`.claude/skills/nexus-phase-preflight/SKILL.md:337-343`), avec fast-path par
défaut justifié par « 0 finding en 21 runs S22-S25 ». Application mesurée en L1 :
le §S3 conclut « Aucune ligne `T-SECRET-*` ni `T-HITL-*` n'existe (grep exhaustif
= 0 hit) → travail L1 purement ADDITIF ». Le préflight L1 contient bien une
section « Vérification adversariale », mais son propre périmètre est écrit :
elle re-mesure les claims du PLAN et du CODE nécessaires à la conception — jamais
les lignes existantes de `THREAT_MODEL.md`, `DURESS.md`, `PATTERNS.md`.

**REVIEW.** Elle porte sur le DIFF, sans ambiguïté : `prompts/agent/phase-review.md`
invoque `git diff` **11 fois** [M : `grep -c "git diff"` = 11] et formule ses
dimensions sur le delta (« Compare `git diff HEAD` to the phase section », « For
every existing MODIFIED code file », « present in the diff »). Les seules mentions
de `docs/` du fichier sont **3**, toutes sur `PATTERNS.md` et toutes orientées
code→doc [M : `grep -niE "docs/"` → l. 141, 146, 147]. Une phrase fausse au repos
n'est ni du code neuf, ni une branche neuve, ni un livrable manquant : elle n'a
aucune porte d'entrée.

**AUDIT GATE.** Même ancrage, granularité plus grosse : « This audit covers the
complete diff of Sprint {N-1} » (`audit-gate-checks.md:12`). Track K pose deux
questions de PRÉSENCE (indexé dans `llms.txt` + le GUIDE ; verdict explicite dans
`verification.md`) — un document intégralement faux mais correctement indexé
passe. Track C lit `PATTERNS.md` APRÈS s'être fait une opinion du code, pour
détecter du pattern neuf non documenté. Track B ne cite `THREAT_MODEL.md` qu'une
fois, pour classer les findings du diff contre T0-T5.

**GATES MÉCANIQUES.** `check-frontier-contracts.sh` est un vérificateur de FORME
et il le documente lui-même : « the semantic "the cited line still supports the
claim" check stays the adversarial LLM review's job (companion); this gate guards
the path + hash drift only » (l. 66-69) [M]. Couverture mesurée à HEAD : **2
types annotés `// FRONTIER:`** (`ShardPlan`, `SbfbPromptAgentPack`) pour **29
familles `DOMAIN_*_V1`** [M : sortie du gate « frontier-tag coverage [2 tagged] +
DOMAIN census [29 frozen] », exit 0]. Le hook pre-commit fait 10 contrôles
intégralement structurels et n'ouvre jamais un document de référence ; le seul qui
frôle la véracité est « le chemin cité dans le body existe-t-il », en WARN.

**LE CONSTAT NÉGATIF, MESURÉ.** Il n'existe, dans tout le corpus process
(`docs/claude/README.md` + `prompts/agent/` + `.claude/skills/` + `.claude/agents/`),
AUCUNE étape dont le mandat est « recompter / re-vérifier les affirmations
vérifiables des documents de référence ». Le seul mécanisme périodique, G2 §6.8,
est OPT-IN par frontmatter et déclenché par un ÉVÉNEMENT UPSTREAM (release, CVE,
S+2) — or aucun des documents porteurs des 14 claims fausses n'a de
`triggers_revalidate`, et une claim devient fausse par une évolution INTERNE du
dépôt, qui n'est aucun de ces événements.

---

## Les trous structurels

**1. Le périmètre de lecture de toute étape automatique est le DIFF.** Préflight
S2 grep `git log` sur les fichiers TOUCHÉS ; review lit `git diff HEAD` ; audit
gate lit `git diff <sprint_start_sha>..HEAD`. Un défaut sort du champ de vision
du système dès qu'on cesse de le toucher. **Preuve [M]** : `git log -S
'secure_unlink_blob' -- keystore.rs` → UN SEUL commit (`c32ecb35`) ; les 3 commits
qui ont retouché le fichier en 102 jours ne portaient pas la fonction. Corollaire :
l'ancienneté d'un défaut le rend PLUS invisible, pas moins.

**2. La sixième étape est indexée sur le NEUF, pas sur le VRAI.** Track K et la
cadence §6.12 parlent de « frontière NEUVE ». Ajouter `SecretVault` ne déclenche
rien du côté du document qui promet de détruire « TOUS les secrets daemon ».
**Preuve [M]** : `DURESS.md` a UN commit en 102 jours, alors que le nombre de
secrets à détruire a augmenté (S54, S79, S81, S83 I/L1).

**3. Le registre de vérification est la PRÉSENCE et la FORME, jamais le
RECALCUL.** Le seul contrôle du dépôt qui recompte vraiment quelque chose
(`DOMAIN_CENSUS_FROZEN`) compare une liste gelée à la liste vivante DANS LE CODE
— il ne lit aucun document [M : liste vivante recomptée à l'instant = 29, gelée =
29]. Le gate le dit de lui-même sur la jambe §19.4 : « Nothing here counts the
bounds — a count no gate recomputes is the class of claim this repo keeps getting
wrong » (l. 565-567) [M]. Volumétrie [B, dimension C] : 93 lignes porteuses de
comptes vérifiables dans les 5 docs rank-1, dont 0 recomptée par une machine.

**4. Le canon EXIGE la duplication des claims et n'exige aucun mécanisme de
liaison.** La clôture docs-contrat impose qu'une frontière soit décrite sur 3-4
surfaces ; aucun gate ne compare deux énoncés d'une même propriété. « Rater un
jumeau » est donc le mode d'échec par DÉFAUT. **Preuve [M]** : `vault.rs` déclare
« TWIN of `sbfb_workflow_engine::hash::is_forbidden_in_identity` — Any change MUST
be mirrored », et `grep "is_forbidden_in_identity" scripts/check-frontier-contracts.sh`
→ 0 hit ; le jumelage est routé à l'audit du sprint suivant.

**5. Le lot de correctifs d'un round n'est soumis à aucune obligation de review
propre — le rendement suit la TAILLE DU LOT, pas le numéro de round.** Trajectoire
P1 : 8, 9, 3, 0, 1, 5, 6, 4, 2, 0 — le rendement REMONTE de 0 à 6 entre r4 et r7
[M]. 21 défauts créés par correctif [M]. Le round 10 ne rend 0 qu'après un r9
réduit à « deux éditions de texte, zéro ligne de code ». **[I]** : un numéro de
round élevé n'indique pas une vigilance croissante mais une RÉCURRENCE — les
défauts trouvés tard le sont parce qu'un correctif les a rendus load-bearing.

**6. Le garde-fou n'est soumis à aucun garde-fou.** L1 a ajouté **+287 lignes** à
`check-frontier-contracts.sh` (384 → 658 [M : `git show HEAD~1:… | wc -l` = 384 ;
`wc -l` = 658]) et 3 des 19 P1 sont des défauts fail-open de ce script. Un gate
est le seul artefact dont l'échec est SILENCIEUX par nature : un `|| true` mal
placé sort `rc=0` et zéro ligne, indiscernable de « clean ». Cas le plus net : le
self-test tautologique de la Phase C, ajouté POUR satisfaire la règle « un gate
doit être discriminant », validé par la review ET par Codex, puis recopié par L1.

**7. L'instrument canonique est aveugle aux fichiers untracked, et une sortie vide
est indiscernable de « aucun changement ».** Au moment des reviews, 4 840 lignes
sur 6 836 (71 %) étaient hors du champ de `git diff HEAD`, dont les 3 870 lignes
de Rust qui SONT le livrable [B, dimension C]. Le round 8 a dû réfuter
explicitement un raisonnement fondé sur cette sortie vide (« ce n'est pas une
preuve d'absence de changement de code, c'est une absence de baseline »). Le canon
prescrit une COMMANDE (`git diff HEAD`) au lieu d'une PROPRIÉTÉ (« lire
l'intégralité de ce que la phase produit »).

**8. Les leçons de process atterrissent dans une mémoire privée que les agents
exécutants ne lisent pas.** `grep -rn "fermer la classe|class not instance|
inventaire exhaustif" docs/claude/README.md prompts/agent/ .claude/agents/` → 0
hit [B, dimension C]. La règle a été écrite le 2026-07-24 ; elle a été violée 3
fois dans un même diff correctif le 2026-07-26, et le préflight L1 (2026-07-25,
27 h après) n'en cite aucune [B]. **[I]** : une règle sans point de lecture dans
le séquencement d'une phase ne peut mordre qu'en aval, dans la colonne
« Correctif » d'un finding déjà trouvé.

**9. Le seul scan pointé sur le threat model a été délibérément dégradé, sur un
argument auto-référentiel.** README:1859 : « S3+S4 sont des gate checks : 0
finding en 17 runs → fast-path grep only ». **[I]** : la mesure « 0 finding » a
été lue comme « ce scan ne sert à rien » plutôt que comme « ce scan pose la
mauvaise question ». Un scan qui ne demande jamais « le texte déjà écrit est-il
encore vrai ? » ne peut évidemment rien trouver ; son taux de découverte nul a
servi à justifier son allègement, ce qui verrouille le trou.

**10. La cotation de sévérité est indexée sur l'effet MACHINE, pas sur le
LECTEUR.** La même forme de défaut (itemisation fausse dans un document de
sécurité committé) a été cotée **P3** à l'audit S82 (finding S82-AG-E-2) et **P1**
en L1 [B, dimension D]. **[I]** : un doc rank-1 lu par l'implémenteur de la phase
suivante a le pouvoir de nuisance d'un bug, mais aucune règle de cotation ne
l'exprime — donc une claim fausse descend mécaniquement en P3, et une classe cotée
P3 ne déclenche aucune contre-mesure.

**Observation résiduelle, à traiter comme telle.** Le correctif L1 a converti la
sonde de `secure_unlink_blob` en `try_exists()?` [M : `keystore.rs:1097` + le
doc-comment l. 1086-1093 qui explique le choix], mais **deux `blob_path.exists()`
subsistent** à `keystore.rs:582` (`init`) et `:852` (`init_duress`) [M]. **[M]** :
les deux sites existent. **[I, non établi comme défaut]** : leur direction
d'échec diffère — un blob illisible y rend `false` et fait PROCÉDER l'init, où
l'écriture elle-même devrait échouer, au lieu de faire déclarer un wipe réussi.
Ce point n'a pas été instruit ici ; il est cité comme illustration du trou n°4
(fermer l'instance plutôt que la classe) et comme candidat pour l'audit gate.

---

## Ce qui a fonctionné

Il faut le dire avec la même précision : **les 19 P1 ont été fermés AVANT le
commit**. Aucun n'est parti dans l'historique. Parmi eux, un bug de sécurité de
102 jours sur le chemin de DURESSE — celui où l'utilisateur est sous contrainte.
Le dispositif a donc rattrapé, en une phase, ce que 62 audit gates et 338 phase
reviews n'avaient pas vu [B, dimension C]. Ce qui l'a permis, mesuré :

1. **La gate EXTERNE, et son mode de lecture.** Codex a trouvé les quatre choses
   que 10 rounds internes avaient manquées, et le body du commit les nomme :
   l'absence de garde sur `Serialize`, le bug `Path::exists()`, deux fenêtres de
   révocation, et le classement du panic wipe [M : `git show -s 8199c50a`, §
   « Codex verification »]. Le levier n'est pas « un round de plus » : c'est un
   modèle qui LIT LA SOURCE (documentation std de `Path::exists`) au lieu de
   raisonner sur le récit du diff.
2. **Le canon §4.5.8 (rounds numérotés, jamais écrasés), posé le 2026-07-20** —
   5 jours avant L1. Sans lui, la récurrence serait invisible : c'est lui qui
   rend mesurables les 21 défauts créés par correctif et la trajectoire non
   décroissante. Corollaire honnête : on ne peut PAS affirmer que ce motif est
   propre à S83 ; avant cette date la trace n'existait pas (absence de MESURE,
   pas absence de fait).
3. **La méthode de CONFRONTATION doc↔code.** Ranger les étages du panic wipe dans
   quatre catégories exclusives (HARD-FAIL / BEST-EFFORT / INERTE / NON RÉCLAMÉ)
   oblige à décider, pour chaque étage, si le code tient la catégorie. C'est ce
   geste qui a rendu `Path::exists()` atteignable. Une prose descriptive
   (« le panic wipe est irréversible ») avait cohabité 102 jours avec le code qui
   la contredisait. Précédent unique : S82 Phase H `32a23f6a` (« passe de fidélité
   PATTERNS↔code »), qui avait déjà produit un correctif de code réel.
4. **L'exigence de discriminance appliquée aux contrôles neufs.** Les deux
   doctests `compile_fail,E0277` ont été mesurés discriminants en injectant l'impl
   fuyant, aux DEUX jumeaux [M : body §« Delta tests »]. C'est la bonne forme —
   celle qui manquait au self-test de la Phase C.
5. **Le recompte à la source plutôt que la reprise.** Le body déclare PÉRIMÉS les
   comptes des rounds r1/r2/r3 et recompte fichier par fichier [M]. C'est la
   contre-mesure du trou n°3, appliquée manuellement.
6. **La jambe Docker.** Deux défauts que Windows ne pouvait pas voir (le script de
   gate en CRLF ne s'exécutait pas sous Linux ; une fois exécutable, sa jambe
   `frontier-tag` rougissait à tort) [M : body §« Verification »]. Une
   plateforme unique n'est pas une preuve.
7. **Le refus de fermer sur un correctif.** Les P2 non traités sont ÉNONCÉS
   (fail-open résiduels de la jambe §19.4), et le carry `KEYSTORE-LAYER2-INERT`
   est routé avec sa conséquence FONCTIONNELLE mesurée plutôt que maquillé.

---

## Amendements proposés

Séparation nette : un **GATE** est fiable parce qu'il est exécuté par la machine
et qu'il rougit ; une **RÈGLE** dépend d'une lecture humaine ou agentique et sera
oubliée — l'ordre de grandeur du taux d'oubli est donné par le dossier lui-même
(règle écrite le 2026-07-24, violée 3 fois le 2026-07-26).

### Gates mécaniques (fiables)

**G1 — Généraliser l'étiquette générée drift-gatée aux comptes des docs rank-1.**
Tout compte ou énumération affirmé dans un document rank-1 devient une jambe de
gate qui le RECALCULE à la source. Le patron existe et est vert
(`DOMAIN_CENSUS_FROZEN`, 29 == 29 [M]).
*Coût* : un recensement initial (~93 lignes porteuses à trier) + ~20-40 lignes de
shell par famille de compte.
*Aurait attrapé dans ce dossier* : les 27 bornes §19.4, les 45 tests, le tuple à
15 champs, et le « ~83 s » — mais SEULEMENT sous la forme qu'il a prise à la
correction, c'est-à-dire la FORMULE (`10^6 × 82 ms`) à côté du résultat.
*Limite dite* : un gate ne recalcule que ce qui a une source machine. « ~82
ms/tentative » est une MESURE de banc, pas un calcul : le gate peut vérifier
l'arithmétique, jamais la valeur mesurée. Ce résidu est irréductible.

**G2 — Registre de jumeaux.** Un fichier de déclaration (source, jumeau,
extracteur) + une jambe générique qui extrait des deux côtés et rougit sur
divergence.
*Coût* : ~60 lignes de shell une fois, puis une ligne par paire.
*Aurait attrapé* : la divergence 3 s vs ~80 ms née le 2026-04-16 et jamais fermée
avant le round 8 ; et `is_forbidden_in_identity`, aujourd'hui « revendiqué par
trois textes et tenu par aucun contrôle machine ».
*Prérequis dur, non négociable* : toute jambe neuve livre sa MUTATION (la preuve
qu'elle rougit). Sans cette clause, on reproduit exactement le self-test
tautologique de la Phase C — le dossier montre qu'il a passé review ET Codex.

**G3 — Élargir le périmètre de scan aux manifestes et aux docs rank-1.** Le gate
scanne `find crates web/src` et exclut `docs/` « par construction ».
*Coût* : une ligne de périmètre + le bruit initial à absorber.
*Aurait attrapé* : le bloc `keyring` du `Cargo.toml` racine, qui a survécu à la
PREMIÈRE passe de correction L1 elle-même parce que le correctif a cherché dans
les surfaces que le process désigne comme documentaires.
*Limite dite* : le gate y verra des chaînes, pas du sens — il attrape la promesse
littérale (« second layer ») en regard de la déclaration, pas un raisonnement.

**G4 — Détecteur d'angle mort de l'instrument.** Rougir quand `git status` montre
des `??` sous `crates/` ou `web/src` pendant qu'une review s'appuie sur
`git diff HEAD`.
*Coût* : ~10 lignes dans le hook.
*Aurait attrapé* : les 71 % du volume de la phase invisibles à l'instrument
canonique, et le raisonnement fondé sur une sortie vide que le round 8 a dû
réfuter.

**G5 — Traiter `scripts/*.sh` comme du code livrable.** Toute jambe neuve ou
modifiée exige une mutation committée et citée dans le body.
*Coût* : ~1 h par jambe.
*Aurait attrapé* : les 3 P1 fail-open de L1 et la tautologie de la Phase C —
c'est-à-dire 4 des 19, sans compter les fail-open assumés en P2.

### Règles (oubliables — à poser en sachant qu'elles rateront)

**R1 — Quand un diff ajoute un call-site à une fonction existante, relire le corps
de cette fonction.** *Aurait attrapé* le défaut n°4, et rendu `Path::exists()`
visible 3 jours plus tôt. *Mécanisable à moitié* : le hook peut LISTER les
fonctions dont le nombre de call-sites augmente dans le diff — c'est un signal,
jamais un verdict.

**R2 — Un chiffre recopié n'hérite pas du statut de vérité de sa source : écrire
la formule à côté du résultat.** *Aurait attrapé* la ligne neuve de `DURESS.md`.
*Absorbable par G1* dès que la formule est présente : elle devient recalculable.

**R3 — Un symptôme environnemental non reproductible dans un commentaire est un
signal de suspicion, pas une explication.** *Aurait attrapé* le commentaire de
`os_keyring_available()`. **Non mécanisable** : c'est un jugement, et le dépôt
documente de vraies instabilités d'environnement qui ont la même forme.

**R4 — Déplacer les leçons de process de la mémoire privée vers le canon portable
lu par les agents.** *Coût* : ~20 lignes dans `prompts/agent/phase-review.md`.
*Aurait attrapé* les récidives de « fermer la classe » le lendemain de sa
consignation. *Mécanisable à moitié* : le hook peut vérifier que le fichier de
review CITE la check-list, jamais qu'il l'a appliquée.

**R5 — Coter P1 une affirmation fausse dans un document rank-1 de sécurité.**
Alignerait l'audit gate sur ce que L1 a fait de fait, et sortirait la classe du
statut P3 qui ne déclenche aucune contre-mesure. **Non mécanisable** : c'est une
grille de jugement.

**R6 — Borner le lot de correctifs et le re-reviewer comme un diff neuf avant de
relancer un round complet.** *Justification mesurée* : 21 défauts créés par
correctif, trajectoire non décroissante, r10 = 0 seulement après un r9 réduit à
deux éditions de texte. *Mécanisable à moitié* : on peut mesurer la taille du lot,
pas décider qu'il est trop gros.

---

## Ce qu'il ne faut PAS faire

**1. Ajouter des rounds.** La trajectoire P1 n'est pas décroissante (8, 9, 3, 0,
1, 5, 6, 4, 2, 0) : le rendement suit la taille du lot correctif, pas le numéro de
round. Un 11e round interne aurait produit ce que produit un lot correctif — de
nouveaux défauts à trouver — pas ce que produirait une vigilance accrue. 15 rounds
ont déjà coûté 3 j 3 h pour une phase.

**2. Alourdir le préflight.** S3/S4 sont en fast-path parce qu'ils rendaient « 0
finding en 21 runs ». Ce n'est pas leur PROFONDEUR qui manque, c'est leur
QUESTION : ils demandent « la primitive proposée est-elle couverte ? » au lieu de
« le texte déjà écrit est-il encore vrai ? ». Un préflight plus lourd sur la même
question rendra 0 finding, plus lentement.

**3. Ajouter des règles-mémoire.** Le dossier contient le contre-exemple :
`feedback_close_class_not_instance` écrite le 2026-07-24, citée en section « À NE
PAS FAIRE » du round 5, violée 3 fois dans le diff correctif du round 6. Une règle
sans point de lecture dans le séquencement d'une phase ne mord qu'après coup.

**4. Interdire les chiffres dans les documents.** `docs/workflow/llms.txt` le fait
localement et l'assume (« No count is given here on purpose »). C'est tenable
ponctuellement, pas comme politique : un threat model sans compte est un threat
model non falsifiable. La bonne réponse est de RECALCULER (G1), pas d'omettre.

**5. Faire d'un round Codex supplémentaire la réponse au bug de production.**
Codex l'a trouvé parce qu'il lit la SOURCE, pas parce qu'il y avait un round de
plus. La conversion à faire est une DIMENSION de review (« re-dériver la
sémantique de chaque primitive std invoquée sur un chemin de destruction »), pas
un round de plus dans la même boucle.

**6. Réécrire en bloc les documents fautifs « pour repartir propre ».** Chaque
passe de correction de ce dossier a créé des défauts (21 mesurés) ; deux des
défauts les plus graves de la phase ont été ÉCRITS par des correctifs. Une
réécriture massive sans gate qui recompte reproduit la classe à plus grande
échelle.

**7. Traiter le sujet comme un problème de rigueur individuelle.** Aucun des
garde-fous invoqués n'existait quand les claims héritées ont été commitées : hook
pre-commit `34dacdcc` le 2026-04-20 (3 jours après), gate Codex mi-mai (1 mois
après), `check-frontier-contracts.sh` `b27079c2` le 2026-06-24 (2 mois après)
[M : `git log -1` sur chaque SHA]. Le process de 2026-04 n'a pas « raté » ces
défauts : il ne les regardait pas. Ce qui est un échec de SYSTÈME, ce sont les 103
jours suivants, où les gates existaient et ne les ont pas vus — parce que leur
périmètre est le diff.

---

## Contradictions entre dimensions, dites comme telles

1. **Nombre de rounds Codex.** La dimension D a mesuré 4 rounds sur disque à 15:52
   ; le brief en annonçait 5. **Résolu [M]** : `r5` a été écrit à 16:09 et le
   commit à 16:19. Ce n'était pas un désaccord de fait mais un décalage de fenêtre
   de mesure — l'analyse tournait pendant que la phase se terminait.
2. **« 19 P1 » vs les sommes par round.** 38 (interne) + 10 (Codex) = 48 P1
   étiquetés ; 19 après déduplication. Les deux chiffres sont exacts sur des
   dénominateurs différents. Ce document utilise 19 pour la répartition par nature
   [B] et 48 pour le coût de la boucle [M].
3. **État de l'arbre.** Les quatre dimensions décrivent un arbre non commité et
   quatre fichiers cœur UNTRACKED. C'est vrai au moment de leur mesure et faux
   maintenant (`8199c50a`). Aucun fait ne change ; seules les ancres migrent de
   « arbre de travail » vers HEAD.

---

## Recommandation unique

Si on ne fait qu'un seul amendement : **G1 — tout compte ou énumération affirmé
dans un document rank-1 doit être RECALCULÉ par une jambe de gate, et toute jambe
neuve doit livrer la mutation qui prouve qu'elle rougit.**

Pourquoi celui-là et pas un autre, en quatre points mesurés :

- Il attaque la classe **majoritaire** : 14 des 19 P1 sont des affirmations
  fausses dans des documents, et le trou de périmètre qui les abrite est explicite
  dans le canon (le gate scanne `crates/` et `web/src`, `docs/` est exclu « par
  construction »).
- Il est **déjà prouvé dans ce dépôt** : `DOMAIN_CENSUS_FROZEN` recompte 29 et
  rougit à la divergence [M, exécuté à l'instant]. On ne demande pas d'inventer un
  remède, on demande de généraliser celui qui marche.
- Il porte le **seul contre-exemple** du dossier : la seule claim fausse héritée
  qu'une étape a fait tomber par mandat propre est « census 25 », tombée parce que
  le scan S4 devait DE TOUTE FAÇON recompter 29. Une affirmation ne tombe que
  quand une étape a une raison indépendante de la re-mesurer. G1 fabrique cette
  raison, systématiquement.
- Il est **mécanique, donc non oubliable** — et le dossier établit que les règles
  le sont : celle qui aurait évité 4 des récidives avait été écrite la veille.

La clause de mutation n'est pas décorative : sans elle, ce dossier montre qu'on
produit des contrôles verts qui ne peuvent pas rougir, validés par la review et
par la gate externe, puis recopiés ailleurs et cités comme preuve.
