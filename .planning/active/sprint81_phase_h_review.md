# Review S81 Phase H — Migration LIVE ancre VPS (livrables agent-executables)

## Verdict: PASS

0 P0/P1 CONFIRMED. Tous les findings survivants sont P2/P3/nit documentables au
commit body — et les actionnables ont ete APPLIQUES in-phase (cf. §Fixes
in-phase). Review 6 dimensions OK + gate Codex joue et reconcilie (cf. §Codex
reconciliation). Promu de PASS-PENDING a PASS le 2026-07-09 apres Codex.

## Synthese

Phase H **operationnelle** : `git status` = 2 fichiers modifies
(`docs/release/STORE_MIGRATION_OPS.md`, `docs/security/THREAT_MODEL.md`) + 3
nouveaux (`docs/release/LIVE_FLIP_RUNBOOK.md`,
`scripts/acceptance/flip_convergence_check.sh`,
`.planning/active/sprint81_phase_h_preflight.md`). **Aucun `.rs/.ts/.tsx`
touche, 0 code runtime, 0 dep, delta tests 0** — coherent avec le pattern E2/E3
(le flip live lui-meme reste operator-gated, futur `chore(acceptance)`). Le seul
artefact executable neuf est un harness bash d'acceptance, homologue de
`b3_live_pc_vps.sh`, non compte dans les suites nextest/Vitest.

Le grounding factuel est **excellent** : chaque claim technique des 5 livrables a
ete verifie contre le code REEL et **aucun claim faux** n'a survecu (routes
`/health`, `/auth/token`->`{"token"}`, `/api/daemon/info`->`node_id` 64-hex,
`/api/daemon/browse`->`"status":"reachable"` + `archive_hash`, `/blob-serve`
toutes existent ; regeneration `node_key` warn-only reelle a
`runtime.rs:139` ; divergence `deploy.sh` reelle ; DEUX roots
`NEXUS_GRID_ROOT`+`SBFB_HOME` niche confirmes ; backup
`docs.redb.backup-redb-v2-tuples` byte-exact ; pins `iroh =1.0.1 / docs =0.101.0
/ gossip =0.101.0 / blobs =0.103.0` exacts).

Les 5 adaptations exigees par le preflight PLAN-ADAPT sont TOUTES presentes et
fideles (rollback 2 gestes / R2-REFUTED, portee snapshot 2 roots + checklist
survivants, tar non-skippable, convergence scindee LOCAL/CROSS, rationale
VPS-dernier sans sur-vendre). Scope/Day-0/PO tenus : re-decision Topologie
A-vs-B **jamais figee**, re-install stock interdit respecte, in-place tranche,
pins exacts, gel=discipline sans verrou code invente. §15.5 respecte la
convention ASCII-sans-accents du fichier, ne contredit aucune row existante, et
son changelog v16 est fidele au diff.

Aucun defaut bloquant. Les findings survivants sont : un fail-open du seul
backstop mecanique d'identite (P2 CONFIRMED, doc-only cheap), et une grappe de
nits de traçabilite/hygiene (staleness plan H downgrade P3, gitignore downgrade
nit, ponts de nommage FLIP_ARTIFACT/T2). Rien n'exige de code runtime.

## Dimension 1 — Diff complet ligne par ligne (harness bash + docs)

Diff = 2 docs modifies + 3 nouveaux. **Correctness bash solide** : `set -uo
pipefail` (pas de `set -e`, delibere — erreurs gerees explicitement), quoting
correct, `mktemp` nettoye sur tous les chemins (`flip_convergence_check.sh:177-186`),
sha256 portable `sha256sum`/`shasum` (`:160-169`), `--capture-baseline` teste
AVANT que `ARCHIVE_HASH`/`BASELINE` soient requis (`:212` avant `:226`),
`BODY_SHA` capture sur echec via `var=$(cmd) || rig_absent` (semantique
exit-status de l'assignation). Le contrat JSON (PASS=0 / BLOCK=1 / RIG-ABSENT=3,
python3 + fallback lossy-mais-valide) est coherent avec `b3_live_pc_vps.sh`.

Findings retenus :

- **D1-1 (P2 — CONFIRMED)** — assert `EXPECT_NODE_ID` **fail-OPEN**.
  `flip_convergence_check.sh:230` garde le bloc identite derriere
  `if [ -n "$EXPECT_NODE_ID" ]` : var non exportee = no-op silencieux, le check
  peut sortir PASS malgre un `node_id` regenere. C'est le SEUL backstop
  automatique de la regeneration warn-only de `node_key` (`runtime.rs:139`
  regenere sans erreur si `len()!=32`). Or `THREAT_MODEL §15.5` row S/D et
  `LIVE_FLIP_RUNBOOK.md` GO/NO-GO declarent `EXPECT_NODE_ID` **OBLIGATOIRE sur
  le VPS -> BLOCK + STOP** : la doc dit MUST, l'outil ne l'enforce pas. La
  mitigation que §15.5 presente comme un gate committe se reduit a la discipline
  checklist. Detail : `ARCHIVE_HASH`/`BASELINE_SHA256` sont, eux, fail-closed
  (`:226`) ; seul l'assert identite est fail-open (`:230`). Fix doc-only cheap :
  mode fail-closed `REQUIRE_NODE_ID` (rig_absent/block si vide sur un run
  non-`--capture-baseline`), ou rejeter d'entree un run avec `BASELINE` fourni
  mais `EXPECT_NODE_ID` vide.

- **D1-2 (P3)** — faux BLOCK possible sur `archive_hash` duplique.
  `flip_convergence_check.sh:242`
  (`... | tr '}' '\n' | grep "$ARCHIVE_HASH" | head -1`) fige le verdict sur la
  PREMIERE entree portant le hash ; si le meme content-hash apparait sur
  plusieurs lignes browse (entree `is_own` + ligne decouverte directory/gossip,
  §P59 catalog-backed) et que la premiere ordonnee n'est pas encore `reachable`
  alors qu'une autre l'est, le check emet un BLOCK a tort. Faible probabilite
  sur le rig 3-noeuds, reel dans le cas cross-noeud own+distant. Fix : ne pas
  figer sur `head -1` — tester la presence de `"status":"reachable"` sur AU
  MOINS une ligne filtree par `archive_hash`.

- **D1-3 (nit)** — `--capture-baseline` ecrit un artefact PASS (`stage="baseline"`)
  sur le meme `FLIP_ARTIFACT` par defaut (`.flip_last_result.json`) que le check
  de convergence (`:221-223`). Les runs sequentiels s'ecrasent proprement et
  `stage` distingue, mais un lecteur qui ne cle que sur `status:"PASS"` peut
  confondre. Fix : recommander un `FLIP_ARTIFACT` distinct pour la capture.

- **D1-4 (nit)** — avec `pipefail`, le pipeline browse (`:242`) remonte un
  non-zero sur no-match/SIGPIPE-de-`head`, silencieusement avale faute de
  `set -e` (comportement voulu : `ENTRY=""` -> poll continue). Correct mais
  implicite. Fix optionnel : commentaire d'une ligne.

## Dimension 2 — Conformite au preflight PLAN-ADAPT

Les 5 adaptations exigees (`sprint81_phase_h_preflight.md:24-37`) sont TOUTES
presentes et fideles : (1) rollback 2 gestes / R2-REFUTED (`STORE_MIGRATION_OPS`
regle 2 + runbook L121-124 + §15.5 row D/I, rationale « restore seul RE-MIGRE »
reproduit) ; (2) portee snapshot 2 roots + checklist survivants
(`STORE_MIGRATION_OPS` regle 1 reecrite, plus complete que le minimum) ; (3) tar
load-bearing non-skippable (titre Phase 1 + modele-de-fenetre + §15.5 + regle 1
« daemon ARRETE ») ; (4) convergence scindee LOCAL/CROSS (runbook 10c/10d +
en-tete harness + §15.5 row I) ; (5) rationale VPS-dernier sans sur-vendre
(runbook « l'ordre ne borne PAS la partition » + §15.5 non-menaces). Le GO/NO-GO
du runbook reproduit fidelement les 5 conditions STOP du preflight.

Finding retenu :

- **D2-1 (P3 — DOWNGRADED de P2)** — staleness du plan H.
  `sprint81_plan.md:335-339` conserve VERBATIM la procedure refutee par le
  preflight : L336 snapshot `docs.redb + blobs/` **omettant node_key** +
  rollback un-seul-geste ; L339 `convergence apres CHAQUE noeud`. Le plan n'est
  PAS dans le diff. Le preflight nomme explicitement le plan comme la SOURCE des
  divergences (L28-31 « c'est le plan H qui diverge », item#2 P1) et liste
  « enrichissement plan H » (L203) comme item agent-preparable non livre. Le gap
  est **reel** (un lecteur consultant le plan plutot que le runbook obtient la
  procedure refutee). Downgrade P2->P3 justifie : (a) le preflight L201 cadre ces
  items comme « L'agent PEUT preparer » (frontiere capacite, pas checklist
  livrables obligatoires) ; (b) toutes les surfaces operateur canoniques
  (`LIVE_FLIP_RUNBOOK.md` + `STORE_MIGRATION_OPS` regle 1/2, header pointant deja
  le runbook) sont pleinement corrigees ; (c) risque operationnel nul — flip
  operator-gated, plan Phase H bloque deja sur F-PASS + D3 node_key non-regen ;
  (d) `plan.md` est un snapshot kickoff-time par convention projet. Fix le moins
  cher : pointeur canonique d'une ligne depuis le bloc Phase H du plan vers
  `LIVE_FLIP_RUNBOOK.md` + `STORE_MIGRATION_OPS` regle 1/2 (discipline
  doc-honnetete : ne pas laisser vivre un texte P1-refute dans le repo).

## Dimension 3 — Securite + Threat Model

`§15.5` coherente avec le style STRIDE-lite de §15.4 (meme en-tete
Menace/Exemple/Sev.brute/Mitigation/res), ne contredit aucune row existante
(§15.4 = MECANISME, §15.5 = OPERATION, residuels mutuellement consistants),
respecte la convention ASCII-sans-accents (verifie 0 lettre accentuee
`THREAT_MODEL.md:1166-1200` + changelog), changelog v16 fidele (5 rows, toutes
res L, 0 bump wire / 0 dep / 0 code runtime). Toutes les severites residuelles L
sous C4/C5 defendables (aucun noeud tiers -> menaces self-inflicted bornees ;
rollback 2-gestes corrige R2 REFUTED ; plan B zero-n0 deja LIVE-prouve
`a085853` borne le risque EOL). **Surface d'attaque du harness : AUCUNE fuite** —
l'artefact JSON n'ecrit jamais le token (shape = donnees PUBLIQUES
node_id/hash/sha256) ; le token loopback n'est ni logge ni persiste ;
`/auth/token` est loopback-gated par Host (`http.rs:530`) donc un BASE distant
renverrait 403 -> RIG-ABSENT (zero token-over-WAN). Le runbook n'expose aucune
valeur secrete (il nomme des variables/chemins a sauvegarder, jamais leurs
contenus).

Finding retenu :

- **SEC-H-1 (P3)** — meme cause racine que D1-1, formulee cote residuel §15.5.
  Le residuel L de la row 3 §15.5 (`THREAT_MODEL.md:1174`, « regression
  d'identite silencieuse ») repose sur `EXPECT_NODE_ID` traite comme OPTIONNEL
  (`flip_convergence_check.sh:57` « optional but STRONGLY recommended », compare
  seulement `:230`), et la sante LOCALE seule est structurellement incapable de
  detecter un node_key tronque/regenere : blob-serve est content-addressed
  (sert par hash, independant de l'identite) et les apps self-published
  court-circuitent a Reachable (`browse.rs:713-717`). Donc sur Win/Mac, si
  l'operateur omet `EXPECT_NODE_ID`, un node_key tronque passe la sante locale en
  PASS et ne surface qu'en cross-noeud sous un diagnostic trompeur
  (« docs-sync/gossip convergence not reached », `:254`). Le residuel L n'est
  non-contingent QUE sur le VPS. Fix doc-only : (a) durcir le harness pour
  RIG-ABSENT si `EXPECT_NODE_ID` vide sur un run non-baseline (option preferable,
  aligne D1-1) ; ou (b) preciser en une ligne dans §15.5 row 3 que la garantie L
  est mandatee-VPS et operateur-disciplinee sur Win/Mac.

## Dimension 4 — Scope + Day-0 + Decisions PO

Phase H reste STRICTEMENT dans son scope : `git status` = 2 M docs + 3 nouveaux,
AUCUN `.rs/.ts/.tsx`, 0 code runtime, 0 dep. Toutes les contraintes Day-0/PO
tiennent : (1) re-decision Topologie A-vs-B **JAMAIS figee** — ni runbook ni
§15.5 ne la mentionnent ; l'ordre Win->Mac->VPS est du sequençage d'upgrade,
orthogonal a la topologie ancre/curateur (preflight S2-6 le confirme) ; (2)
re-install stock INTERDIT respecte (runbook + §15.5 row S/D « D3 cond.5/R1 ») ;
(3) in-place tranche respecte ; (4) pins exacts (`Cargo.toml:48-51`) ; (5)
gel=discipline sans verrou code invente (runbook « aucun verrou code n'existe »).
Le preflight S2-3 (interdiction de re-planifier A2 self-heal + garde F committes)
est respecte : le runbook REFERENCE la garde
`refuse_recreate_on_interrupted_migration` (`runtime.rs:2574`) et l'etape 14
supprime le backup = action operateur re-armant le self-heal existant. Franglais
conforme : docs/release en français accentue, §15.5 en ASCII, harness en anglais.

Findings retenus (nits, deja cadres par la phase) :

- **D4-1 (nit)** — §15.5 attribue res=L a la row « Partition totale intra-fenetre »
  sur une propriete UPSTREAM (iroh) que la phase declare non verifiable depuis
  SBFB avant le flip. Deja cadre honnetement (§15.5 « Non-menaces » : « a
  confirmer empiriquement au flip et logger dans l'artefact T2 » + runbook Phase 3
  « verifier que les residuels observes collent »). Aucune action — comportement
  correct exige par le preflight (residuel provisoire a confirmer au flip).

- **D4-2 (nit)** — `flip_convergence_check.sh` est un artefact executable committe
  sous une phase « delta tests 0 ». Coherent avec le pattern (`b3_live_pc_vps.sh`
  n'incremente aucun compteur nextest/Vitest). Aucune action ; le commit body
  peut expliciter « nouveau harness acceptance, hors compteurs suites ».

## Dimension 5 — Research grounding + exactitude factuelle

**ZERO claim faux — aucun P0/P1/P2.** Chaque surface citee existe avec la forme
annoncee : (a) routes `/health` + `/blob-serve` publics (`http.rs:255-256`),
`/auth/token` public loopback Host+Origin (`:265`, handler `:523-547`
->`{"token":...}`), `/api/daemon/info` + `/api/daemon/browse` authed (`:277,282`) ;
(b) `DaemonStateSnapshot.node_id` String serialise `"node_id"`, test asserte
`len==64` (`state.rs:49,190`) -> le sed 64-hex du harness matche ; (c)
`BrowseStatus #[serde(rename_all="lowercase")] Reachable->"reachable"`
(`browse.rs:147,162`), `archive_hash` Option skip_if_none (`:237`),
`BrowseEntryView #[serde(flatten)]` (`http.rs:988`), own_entries
court-circuitent Reachable sans dial quand `node_id==me` (`browse.rs:713-717`)
-> le mode sante LOCALE du harness est **fonde**, pas une hypothese ; (d)
`load_or_generate_node_key` regenere warn-only si `len!=32` (`runtime.rs:139`) ;
(e) divergence deploy correcte (`deploy.sh:70-92` /opt/nexus-grid/bin +
`restart nexus-daemon` vs unite S75 `/usr/local/bin/nexus-shell-daemon` +
`nexus-shell-daemon.service:30`) ; (f) backup sibling
`docs.redb.backup-redb-v2-tuples` (`runtime.rs:2558`), DEUX roots niches
(`service:48-49`) ; (g) `sbfb-ci=rust:1.94` ; (h) pins iroh exacts
(`Cargo.toml:48-51`).

Findings retenus (nits) :

- **D5-1 (nit)** — pont de nommage artefact non epele au step 13. Le runbook
  Phase 3 nomme l'artefact T2 committe `sprint81_t2_h_live_flip.json`, mais le
  harness ecrit par defaut `.flip_last_result.json` (`:63,88`) ; le nom T2 n'est
  produit que si l'operateur exporte `FLIP_ARTIFACT` (documente `:63` du script,
  pas rappele au step 13). Fix : ajouter la commande explicite au runbook step 13.

- **D5-2 (nit)** — caveat boot driver one-shot non nomme. Le runbook step 12
  s'appuie sur la re-annonce du boot driver (reelle : `runtime.rs:888,1139-1148`)
  mais ne nomme pas le carry S75 « re-drive-on-ingest, fenetre morte 1er boot
  OBSERVEE live ». Le runbook mitige deja (verifier sur un pair + T2), d'ou nit.
  Fix : une phrase « si l'annuaire n'est pas re-vu, forcer un pull/re-boot plutot
  qu'attendre une re-drive ».

## Dimension 6 — Livrables + Patterns + Frontieres (test-acteur §6.12)

PASS. La phase livre TOUT son perimetre agent-executable promis par le preflight
H : runbook central neuf, harness convergence committe, corrections
`STORE_MIGRATION_OPS` (rollback 2 gestes + snapshot 2 roots + TAR-pas-rename
VPS), §15.5 (5 rows + v16), artefact preflight. Le flip live est correctement
operator-gated. Le harness suit fidelement les patterns d'acceptance : SPDX
AGPL-3.0-or-later, doc-in-head detaillee, contrat JSON ferme PASS/BLOCK/RIG-ABSENT
exit 0/1/3, `--capture-baseline` — miroir de `b3_live_pc_vps.sh`. Toutes les APIs
loopback consommees existent, shape JSON correct.
`check-frontier-contracts.sh` clean : le harness est CONSOMMATEUR d'une frontiere
loopback existante deja exercee par b3 -> **aucune frontiere NEUVE** (N-A pour H,
a tracer au wrap-up K). `preflight.md` bien forme (un seul `## Verdict:
PLAN-ADAPT`).

Findings retenus :

- **H6-1 (nit — DOWNGRADED de P2)** — l'artefact par defaut
  `scripts/acceptance/.flip_last_result.json` n'est PAS dans `.gitignore` alors
  que ses 5 freres d'acceptance y sont (`.gitignore:151-157` :
  `.b3_last_result.json`, `.b3_shard_last_result.json`, `.b3_worker.log`,
  `.app_authoring_last_result.json`, `.app_authoring_pw.json/.log`). Le harness
  ecrit inconditionnellement (`emit_artifact` -> `>"$FLIP_ARTIFACT"`, defaut
  `$SCRIPT_DIR/.flip_last_result.json`) ; a chaque run pendant le flip, un
  untracked machine-specifique apparait en `git status` -> heurte la discipline
  arbre-propre + risque de commit accidentel. Rupture de pattern REELLE, fix
  d'une ligne. Downgrade P2->nit : impact purement cosmetique (aucun effet
  runtime/correctness/wire ; simple dotfile errant, risque de commit faible).
  Fix : ajouter `scripts/acceptance/.flip_last_result.json` a `.gitignore` a cote
  de `:152-156`. **Recommande d'appliquer avant commit** (0 code, coherent
  delta-tests-0, evite un untracked en fin de tour).

- **H6-2 (P3)** — meme pont de nommage que D5-1 : rien dans le runbook n'indique
  de passer `FLIP_ARTIFACT=...` vers le chemin/nom T2 committe
  (`sprint81_t2_h_live_flip.json`) lors du palier d'acceptance. Fix : commande
  explicite au runbook Phase 3 #13.

## Findings P0/P1 (bloquants)

Aucun. 0 P0, 0 P1.

## Findings P2/P3 retenus (a documenter dans le commit body)

- **D1-1 (P2, CONFIRMED)** — assert `EXPECT_NODE_ID` fail-open
  (`flip_convergence_check.sh:230`) : seul backstop mecanique de la regeneration
  warn-only `node_key` (`runtime.rs:139`), la mitigation §15.5 que la doc declare
  gate committe se reduit a la discipline checklist. Fix doc-only cheap
  (`REQUIRE_NODE_ID` fail-closed) **recommande dans ce commit ou le suivant**.
- **SEC-H-1 (P3)** — corollaire securite de D1-1 : residuel L §15.5 row 3
  non-contingent seulement sur le VPS ; sante LOCALE structurellement aveugle a
  un node_id regenere (blob content-addressed + own court-circuit
  `browse.rs:713-717`).
- **D2-1 (P3, DOWNGRADED de P2)** — `sprint81_plan.md:335-339` conserve verbatim
  la procedure refutee par le preflight (snapshot omettant node_key + rollback
  1-geste + convergence-apres-chaque-noeud) ; corrige partout ailleurs. Fix :
  pointeur canonique d'une ligne du plan vers le runbook.
- **D1-2 (P3)** — `head -1` sur `archive_hash` (`:242`) peut emettre un faux BLOCK
  en cas de hash duplique own+distant. Fix : `grep -q '"status":"reachable"'` sur
  l'ensemble filtre.
- **H6-2 (P3)** — pont de nommage `FLIP_ARTIFACT`->T2 non epele au runbook step 13.
- **D5-1 (nit)** — meme pont, cote step 13.
- **D5-2 (nit)** — caveat boot driver one-shot (carry S75) non nomme au step 12.
- **D1-3 / D1-4 (nits)** — baseline sur meme artefact par defaut ; pipefail avale
  implicitement le non-zero du pipeline browse.
- **D4-1 / D4-2 (nits)** — residuel partition UPSTREAM a confirmer au flip (deja
  cadre) ; harness executable sous delta-tests-0 (coherent pattern).
- **H6-1 (nit, DOWNGRADED de P2)** — `.flip_last_result.json` absent de
  `.gitignore` ; **applique de preference avant commit** (une ligne).

## Refutes/downgrades (transparence adversariale)

- **D2-1 : P2 -> P3 (DOWNGRADED).** Fait exact et verifiable, gap reel, MAIS le
  preflight L201 cadre l'« enrichissement plan H » comme item agent-*capable*
  (« L'agent PEUT preparer »), pas checklist livrables obligatoires ; toutes les
  surfaces operateur canoniques sont corrigees (runbook + STORE_MIGRATION_OPS,
  header pointant le runbook) ; risque operationnel nul (flip operator-gated, plan
  bloque deja sur F-PASS + D3) ; `plan.md` = snapshot kickoff par convention.
  Legitime nit doc-honnetete, n'atteint pas P2.
- **H6-1 : P2 -> nit (DOWNGRADED).** Rupture de pattern confirmee contre
  `.gitignore:151-157` et fix phase-approprie, MAIS impact purement cosmetique
  (aucun effet runtime/correctness/wire, simple untracked dotfile, risque commit
  faible) — omission d'hygiene triviale d'une ligne, mieux classee nit.
- **D1-1 : P2 CONFIRMED (maintenu).** Tous les faits verifies ; fail-open reel
  (`:230`), warn-only reel (`runtime.rs:139`), §15.5+runbook declarent OBLIGATOIRE
  sans enforcement outil. Non sur-severise : flip operator-gated, residuel borne L
  sous C4/C5, header documente deja la var — mais l'objectif explicite de la phase
  (prose manuelle -> gate machine-lisible committe) est mine precisement pour le
  check sans backstop alternatif. Reste P2.

**Prochaine etape** : gate Codex BLOQUANTE (review->commit) avant le commit de la
phase. Fixes doc-only recommandes avant commit : **H6-1** (une ligne `.gitignore`)
et de preference **D1-1** (`REQUIRE_NODE_ID` fail-closed) — 0 code runtime, delta
tests 0 conserve.

## Fixes in-phase (post-review, avant Codex)

Les findings survivants ont ete APPLIQUES avant le gate Codex (tous doc/script,
0 code runtime, delta tests 0 conserve) :

- **D1-1 (P2) APPLIQUE** — mode fail-closed `REQUIRE_NODE_ID=1` ajoute au
  harness : `EXPECT_NODE_ID` vide -> `RIG-ABSENT` (jamais un skip silencieux) ;
  runbook (step 11 + GO/NO-GO) mandate `REQUIRE_NODE_ID=1` sur le VPS.
- **SEC-H-1 (P3) APPLIQUE** — §15.5 row 3 precise que le residuel L est
  CONTINGENT au fail-closed sur le VPS + que la sante LOCALE seule ne detecte
  pas une regeneration (blob content-addressed).
- **D2-1 (P3) APPLIQUE** — note PLAN-ADAPT ajoutee sous §Phase H du plan
  (pointeur canonique vers LIVE_FLIP_RUNBOOK.md + STORE_MIGRATION_OPS.md ;
  le rollback 1-geste et le perimetre snapshot verbatim sont marques refutes).
- **D1-2 (P3) APPLIQUE** — le poll browse teste `"status":"reachable"` sur
  TOUTES les lignes matchant `ARCHIVE_HASH` (plus de `head -1` — un hash
  duplique own+distant ne peut plus produire un faux BLOCK).
- **H6-2 (P3) APPLIQUE** — runbook step 13 epelle le pont `FLIP_ARTIFACT` ->
  artefact T2 committe `.planning/active/sprint81_t2_h_live_flip.json`.
- **H6-1 (nit) APPLIQUE** — `.gitignore` + `scripts/acceptance/.flip_last_result.json`.
- Nits restants (D1-3, D1-4, D4-1, D4-2, D5-1, D5-2) : documentes, non-appliques
  (cosmetiques, sans effet operationnel).

`bash -n` re-verifie apres fixes : SYNTAX OK.

## Codex reconciliation

Rapport Codex GPT 5.5 lu (`sprint81_phase_h_codex_review.md`, output brut
`codex exec -o`, non reecrit) : **verdict global OK, 0 gap P0/P1**. Les 6
livrables (harness + runbook + STORE_MIGRATION_OPS + §15.5/v16 + preflight +
fixes post-review) verdict **OK** chacun, avec evidence fichier:ligne verifiee
par Codex contre le code reel (routes http.rs:253-282/519-544, state.rs:42-49,
browse.rs:178-237, runtime.rs:129-150, deploy.sh:69-101 vs service:30-49).

- **P2 Codex (review.md PASS-PENDING non suivi)** : c'etait l'etape suivante
  normale de la sequence stricte review → Codex → promote → commit ; le present
  fichier est promu `## Verdict: PASS` et STAGE avec la phase. Resolu.
- **P3 Codex (synthese perimetree "2 M + 3 nouveaux" vs etat post-fixes)** :
  exact — la synthese decrivait l'etat AVANT les fixes in-phase ; le perimetre
  FINAL du commit est 4 M (`.gitignore`, `sprint81_plan.md`,
  `STORE_MIGRATION_OPS.md`, `THREAT_MODEL.md`) + 4 nouveaux (`LIVE_FLIP_RUNBOOK.md`,
  `flip_convergence_check.sh`, `sprint81_phase_h_preflight.md`, ce review) +
  l'artefact Codex. Le delta est entierement couvert par §Fixes in-phase. Note
  ici plutot que reecrire la synthese (historicite de la review).

Aucune correction de code requise par Codex → pas de boucle re-suites/re-review.
Suites §7.4 re-verifiees sur le diff final par ailleurs (cf. commit body).
