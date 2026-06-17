# Sprint 76 Phase E — Preflight G8 (dashboard contributeur, D4)

## Verdict: PLAN-ADAPT

L'approche du plan §8 (2e vue d'agregation keyee `worker_node_id` sur le
ledger kudos existant, reutilisant `effective_score()`, 3 metriques honnetes)
est saine et ne viole AUCUNE decision Day-0 gelee. Des details concrets
changent : (1) l'index `idx_kudos_worker` est PRE-EXISTANT (db.rs:50) — pas de
M20 ; (2) D4-Q est tranche ci-dessous avec une portee REALISTE fondee sur la
carte fichiers reelle ; (3) plusieurs locations du plan ont derive. Aucune de
ces adaptations ne touche un arbitrage gele : kudos reste non-monetaire/0-token,
le quorum exact-match reste la frontiere de confiance INCHANGEE, le livrable
reste une self-view per-node (pas un ranking global EigenTrust).

## Decision D4-Q (anti-gaming tokens_generated): HARDEN_MEDIAN_SANITY

**Portee retenue : DURCIR via le SANITY-BOUND per-entry
(`tokens_generated <= TOKENS_PER_MS_CEILING * generation_time_ms` avant
`log_utility`). Le "median du groupe d'accord quorum" est RE-CADRE en
SCOPE-CUT/DOC-P2** — non par confort, mais parce que (a) il n'est PAS faisable
sans casser un contrat gele, (b) il n'est PAS ce que fait BOINC, (c) il est
inerte sur le chemin dominant.

### Rationale technique (faisabilite reelle, verifiee par Read/Grep)

**Le groupe d'accord N'EST PAS threadable a `credit()` sans migration schema +
modif du hot path quorum verrouille.** Verifie :
- `task_results` ne stocke que 4 colonnes : `INSERT INTO task_results
  (task_id, worker_id, sha256, created_at)` (db.rs:549) — **aucun
  `tokens_generated`**. `TaskResultRow` = `{task_id, worker_id, sha256,
  created_at}` (types.rs:64-69). `get_task_results()` SELECT ces 4 colonnes
  seules (db.rs:558).
- `validate_quorum_pre_guardrail(db, task, worker_id, sha256, now)`
  (validator.rs:219) ne recoit QUE le `result_text` (param `sha256`) du
  resultat courant ; le groupe d'accord est calcule en interne
  (`counts` HashMap sur `r.sha256`, validator.rs:241-244) puis **JETE** :
  la fonction ne retourne que `ValidationOutcome` + `Option<TaskRecord>` +
  `Option<PendingResultPersist>` — jamais le groupe ni ses tokens.
- Les **2 seuls sites prod** qui appellent `credit()` le font sur l'outcome
  `Accepted`, avec en main UNIQUEMENT `entry.payload` du resultat courant :
  `validator_loop.rs:109-115` (chemin gossip quorum) et `http.rs:3457-3463`
  (chemin HTTP). `credit()` lui-meme prend un scalaire
  `tokens_generated: u64` (kudos_ledger.rs:51-56), jamais un groupe.

Pour implementer "median(tokens du groupe d'accord)" il faudrait : **M20**
(ajout colonnes `tokens_generated` + `generation_time_ms` a `task_results`) +
change signature `insert_task_result` (db.rs:541) + change `TaskResultRow`
(types.rs:64) + propager les tokens aux 2 sites quorum + faire remonter le
groupe hors de `validate_quorum_pre_guardrail` + change signature `credit()`.
C'est une modif **schema + signature sur le HOT PATH quorum**, dont
l'invariant "validator INCHANGE" est verrouille par le test
`validator_quorum_unchanged` (S76-C/D). Le sanity-bound, lui, n'exige
**aucune** de ces modifs.

**Ancrage OSS exact : BOINC credite l'INSTANCE CANONIQUE, PAS un median du
groupe.** BOINC CreditNew : "granted credit = claimed credit of the CANONICAL
instance" (= l'entree validee/acceptee, exactement ce que `credit()` fait deja
sur l'entree `Accepted`). Le discard-high/low BOINC s'applique a la
NORMALISATION cross-host des stats, pas au credit per-WU. La pratique OSS
robuste qui mappe ici est **`wu.rsc_fpops_bound`** : "if PFC exceeds
rsc_fpops_bound, the PFC is replaced and not used" = un SANITY-BOUND
plausibilite. Donc la moitie "median du groupe" de l'Option (a) initiale n'est
ni OSS-fidele ni faisable proprement ; la moitie OSS-SOLIDE **et** faisable est
le sanity-bound.

**Le median est de toute facon INERTE sur le chemin dominant.** `default_redundancy()
= 1` (types.rs:118). A redundancy=1 le "groupe d'accord" = 1 entree =>
`median(groupe)` = cette entree => la moitie median ne fait STRICTEMENT RIEN.
Elle n'agirait que sur le sous-ensemble verifiable redundancy>1 (S76-C/D). Le
sanity-bound, lui, agit sur les DEUX chemins (single + quorum) car
`generation_time_ms` est dans chaque payload.

**Le sanity-bound est faisable proprement (0 schema, 0 wire, validator
INCHANGE).** `ResultPayload.generation_time_ms: u64` (task.rs:481, doc deja
present : "detecting implausibly-fast replies that may indicate cheating") est
dans le payload signe, accessible aux 2 sites credit (`entry.payload` en main a
validator_loop.rs:109 et http.rs:3457). Implementation = ajouter UN parametre
scalaire `generation_time_ms` a `credit()` et clamper
`tokens_generated.min(TOKENS_PER_MS_CEILING * generation_time_ms)` AVANT
`log_utility`. Aucune migration, aucun change quorum, aucune violation de
"validator INCHANGE".

**Pourquoi HARDEN et pas DOCUMENT_P2 pur :** la decision PO (design_review §11)
est "durcir maintenant". Le sanity-bound HONORE cette intention avec la borne
anti-gonflage reelle (il ferme la fuite "declarer 1e9 tokens en 5ms" que
`log_utility` ne ferme PAS — `log2` compresse le gain marginal <10x mais ne
plafonne pas la valeur absolue). DOCUMENT_P2 seul laisserait cette fuite
ouverte. On DURCIT donc — mais avec la moitie de l'Option (a) qui est a la fois
OSS-ancree ET realisable sans casser un contrat fige ; le median-de-groupe est
documente P2 (cher, inerte a redundancy=1, touche le hot path verrouille).

## S1a — OSS prior-art (faisabilite option a)

- **CONFLICT (corrige en re-cadrage) :** le groupe d'accord n'a PAS de
  `tokens_generated` stockes (db.rs:549, types.rs:64) ; `median(groupe)` exige
  M20 + change signature `insert_task_result`/`credit()` + recompute dans le
  quorum = touche le contrat "validator INCHANGE" verrouille S76-C/D
  (`validator_quorum_unchanged`). NON-trivial.
- **NOTE (validee) :** BOINC CreditNew credite l'instance CANONIQUE (=
  l'entree acceptee, deja le cas), pas un median-of-all ; l'ancrage OSS solide
  est le sanity-bound (`wu.rsc_fpops_bound`), qui n'a pas besoin du groupe.
- **INFO (validee) :** sanity-bound `tokens <= f(generation_time_ms)` faisable
  per-entry sans toucher le quorum — `generation_time_ms` dans le payload signe
  (task.rs:481), accessible aux 2 sites credit (1 param scalaire, 0 schema, 0
  wire).
- **NOTE (validee) :** redundancy=1 domine (`default_redundancy()=1`,
  types.rs:118) => median inerte sur le chemin majoritaire.
- **INFO (validee) :** `log_utility` compresse l'incitatif <10x (test
  `log_utility_compression`, kudos_ledger.rs:290-300) mais ne BORNE pas la
  valeur absolue ; le sanity-bound ferme la vraie fuite.

**Conclusion faisabilite Option (a) :** la moitie "median du groupe" n'est ni
OSS-fidele ni faisable sans casser un contrat gele et est inerte a
redundancy=1 => RE-CADREE en P2. La moitie "sanity-bound" est OSS-solide
(BOINC fpops_bound) + faisable proprement => RETENUE comme le coeur du DURCIR.

## S1b — Deps/CVE (0 nouvelle dep confirme)

- coordinator-rs : la 2e agregation keyee `worker_node_id` (GROUP BY + fold EMA
  Rust) reutilise `rusqlite`/`rusqlite_migration`/`serde`/`serde_json` (deja
  dans Cargo.toml). 0 crate.
- nexus-shell-daemon : la nouvelle route axum (handler appelant coordinator-rs)
  miroir de `leaderboard()` (kudos_api.rs:98) reutilise axum/serde + dep path
  `nexus-coordinator-rs` deja presente. 0 crate.
- `log` = `std f64::log2` deja (kudos_ledger.rs:21) ; un median eventuel =
  `Vec<u64>::sort_unstable()` + index = std pur ; sanity-bound = multiplication
  + `.min()` = std pur. Aucune tentation statrs/num/ndarray.
- Front : carte contributeur + hook fetch reutilisent
  `@tanstack/react-query`/`zod`/`lucide-react` (deja dans package.json) ;
  Network.tsx heberge deja GpuCard + ProjectsServedCard sur cette stack. 0 npm.
- 0 nouveau crate transitif => 0 nouvelle surface RustSec ; posture CVE
  (R-iroh-audit/R-wasmtime/R-libcrux) inchangee.

**=> 0 nouvelle dep, sous l'une OU l'autre issue de D4-Q.**

## S2 — Decisions historiques (DESIGN-CONFLICT check)

- **Kudos non-monetaire : NON viole.** Grep `cost|deposit|stake|burn|refund|
  escrow|currency|monetary|wallet|->GRC` sur kudos_ledger.rs = 0 hit (seul
  match = le mot "non-monetary" dans le header doc L4). Les 3 metriques = kudos
  effectifs (EMA `effective_score`), taches servies (COUNT lignes quorum), GPU-
  heures LOCALES (`usage.json`, jamais repliquees). Gridcoin RAC->GRC reste
  REJETE (decision gelee). 0 token crypto.
- **Rejet EigenTrust : NON viole.** Le livrable = self-view per-node
  (`contributor_dashboard(Path(node_id))`), miroir de `leaderboard()`
  (kudos_api.rs:98, Path(project_id)) — PAS un ranking global normalise
  power-iteration. Le leaderboard per-PROJET existant (`get_project_kudos`,
  kudos_ledger.rs:134, tri descendant :156) reste per-projet. La reconnaissance
  contributeur publique reseau-wide est un scope cut post-launch explicite
  (plan §8.8). **Garde-fou review : NE PAS ajouter de route classement
  reseau-wide tous-nodes — ce serait le CONFLICT.**
- **Granularite per-task native : preservee.** `credit()` insere 1 ligne par
  appel (`db.insert_kudos(&entry)`, kudos_ledger.rs:81) ; le sanity-bound (ou
  un median) ne touche QUE le calcul d'`amount`, pas la cardinalite ni le
  chainage `prev_hash` (kudos_ledger.rs:64).
- **Fairness LT-1 (Gini>0.70) : pas declenche.** La vue reutilise
  `effective_score()` (EMA alpha=0.97, kudos_ledger.rs:124) + `log_utility`
  (deja livres S59). Affichage seul, aucune gouvernance kudos-weighted.
- **Aucun rejet historique d'une vue contributeur per-node.** `list_entries`
  a deja un filtre `worker_node_id` (kudos_api.rs, route /api/v1/kudos/entries
  http.rs:445). La 2e vue est un prolongement de surfaces deja acceptees.

**=> Pas de DESIGN-CONFLICT.** Point clarifie : D4-Q est cadre par l'arbitrage
PO "durcir" ; le preflight ne re-ouvre PAS vers DOCUMENT_P2 pur — il DURCIT via
le sanity-bound (HARDEN) et documente P2 la SEULE sous-partie (median-de-groupe)
non realisable sans casser le contrat quorum verrouille.

## S3 — Threat model (residuel chiffre, GPU-heures honnetes, surface route)

- **Residuel anti-gaming chiffre.** `tokens_generated` est self-declare HORS
  quorum (le validator ne compare QUE `result_text`, validator.rs:122/204).
  THREAT_MODEL §15.2 acte deja le Sybil multi-keypair en residual M. Apres
  sanity-bound : un worker SOLO ne peut plus declarer un volume de tokens
  incoherent avec son `generation_time_ms` (fuite "1e9 tokens en 5ms" fermee) ;
  `log_utility` compresse en plus le gain a <10x (kudos_ledger.rs:290-300). Le
  residuel restant = adversaire qui forge les DEUX champs coherents (meme
  payload signe) — c'est le residual M pre-existant §15.2, PAS un nouveau
  residuel cree par Phase E (qui ne fait qu'AGREGER en lecture un ledger deja
  credite).
- **Sanity-bound = plausibility-check, PAS defense anti-Sybil.**
  `generation_time_ms` est dans le MEME payload signe que `tokens_generated`
  (task.rs:476,481) ; un adversaire qui controle le payload peut satisfaire la
  borne en forgeant les deux coherents. A DOCUMENTER comme borne ASYMETRIQUE
  (attrape le bug/exageration grossiere + worker naif), pas comme attestation.
- **GPU-heures LOCALES : libelle honnete OBLIGATOIRE (sinon WARN sur-promesse
  bloquant).** `hours_today` vit dans `consent.rs` (`UsageState`, `usage.json`
  local), JAMAIS dans la coordinator DB ni repliquee (grep
  `usage.json|hours_today` dans nexus-shell-daemon/src = 0 match). Le front
  existant Network.tsx libelle deja "heures donnees par cette machine". Le
  livrable DOIT : (1) libeller "heures donnees par CETTE machine (non
  attestees)", (2) ne JAMAIS agreger cross-node ni classer reseau, (3)
  reutiliser le ton honnete de Network.tsx.
- **Surface route : sous `authed_routes`.** Toutes les routes
  /api/v1/kudos/* sont dans `authed_routes` (http.rs:340/445/447), merge :492
  avec le layer `auth_required` :487 = X-SBFB-Token + loopback Host + Origin.
  `public_routes` (:253) = seulement /health + /blob-serve.
  `contributor_dashboard(Path(node_id))` DOIT etre ajoutee a `authed_routes`.
  Aucune fuite nouvelle : ledger deja local, `worker_node_id` = cle publique
  Ed25519 deja en clair (kudos_ledger.rs:25-32).
- **Cellule §15.3 recommandee (ajout DANS la phase, autorise par la regle
  d'evolution §16).** Documenter : tokens-gaming -> sanity-bound +
  log_utility<10x -> residual M (Sybil majoritaire, pre-existant §15.2) ;
  GPU-heures self-mesure -> libelle honnete -> residual L (jamais repliquee) ;
  sanity-bound = plausibility-check self-consistant -> residual M.

## S4 — Wire format / migration (0 bump ; index PRE-EXISTANT)

- **0 nouveau champ wire signe.** `tokens_generated` (task.rs:476) et
  `generation_time_ms` (task.rs:481) sont des champs EXISTANTS du payload signe.
  La vue ne fait que LIRE le ledger ; les GPU-heures viennent de `usage.json`
  (local, non-attestee, jamais serialisee dans une struct gossipee).
- **`credit()` ecrit une ligne SQLite LOCALE.** `db.insert_kudos(&entry)`
  (kudos_ledger.rs:81) = INSERT local, pas de gossip. `DOMAIN_KUDOS_V1`
  (kudos_ledger.rs:45) sert UNIQUEMENT au hash-chain local
  (`compute_entry_hash`), aucun broadcast. **=> la decision D4-Q (sanity-bound
  ou median) est un changement PURE-LOCAL de `credit()`, 0 wire/canonical.**
- **`idx_kudos_worker ON kudos(worker_node_id)` EXISTE DEJA — db.rs:50** (dans
  la 1re migration M0, a cote du CREATE TABLE kudos db.rs:39). `idx_kudos_project`
  a db.rs:51. **=> PAS de M20 pour l'index.** Le test plan #3
  (`contributor_kudos_query_uses_worker_index`) doit verifier l'USAGE de
  l'index via `EXPLAIN QUERY PLAN` (`SEARCH ... USING INDEX idx_kudos_worker`),
  PAS sa creation.
- **0 nouvelle colonne.** CREATE TABLE kudos (db.rs:39-48) a deja toutes les
  colonnes requises par la requete cross-project (`worker_node_id`, `task_id`,
  `project_id`, `amount`, `created_at`). La requete `WHERE worker_node_id=?1
  GROUP BY project_id` lit des colonnes existantes.
- **`SCHEMA_VERSION` (WorkerStateSnapshot) inchange.** Phase E est read-side ;
  n'ecrit pas dans WorkerStateSnapshot.
- **Plafond migration = M19** (db.rs, dernier M::up = seed_invite S74).
  Phase E n'ajoute AUCUNE migration => plafond reste M19.

## Adaptations au plan (PLAN-ADAPT)

1. **Index `idx_kudos_worker` PRE-EXISTANT (db.rs:50).** Le plan §8 E.2
   annonce "+ index SQLite sur worker_node_id" — l'index existe deja (M0). NE
   PAS creer de M20 ni de `CREATE INDEX` duplicate. Le test #3 doit asserter
   l'USAGE via `EXPLAIN QUERY PLAN`, pas la creation. (SCOPE-CUT-CONSISTENT sur
   le sous-item "ajouter index".)
2. **D4-Q tranche = HARDEN via sanity-bound, median-de-groupe -> P2.** Le
   DURCIR retenu = `tokens_generated.min(TOKENS_PER_MS_CEILING *
   generation_time_ms)` avant `log_utility`, via ajout d'1 param scalaire
   `generation_time_ms` a `credit()` (kudos_ledger.rs:51) cable aux 2 sites
   (validator_loop.rs:109, http.rs:3457). Le "median du groupe d'accord
   quorum" est DOCUMENTE P2 (non realisable sans M20 + casser "validator
   INCHANGE" ; inerte a redundancy=1 ; non OSS-fidele a BOINC qui credite le
   canonique). Definir `TOKENS_PER_MS_CEILING` comme constante nommee
   (regle README §6.9 named-constants).
3. **Site credit() — signature.** `credit(db, project_id, worker_node_id,
   task_id, tokens_generated)` (kudos_ledger.rs:51) devient
   `credit(..., tokens_generated, generation_time_ms)`. Les 2 sites prod ont
   `entry.payload.generation_time_ms` en main. Mettre a jour les ~13 appels de
   test internes (kudos_ledger.rs tests + http.rs tests) en consequence.
4. **Route `contributor_dashboard(Path(node_id))` dans `authed_routes`**
   (http.rs:275-487, PAS `public_routes` :253), pour heriter du gate loopback
   bearer+Host+Origin. Handler miroir de `leaderboard()` (kudos_api.rs:98).
5. **Nouvelle agregation coordinator-rs keyee `worker_node_id` cross-project.**
   Ajouter une fonction (p.ex. `get_contributor_summary(db, node_id, now)`)
   qui SELECT `WHERE worker_node_id=?1`, GROUP BY project_id, et applique
   `effective_score()` (EMA alpha=0.97) — EXACTEMENT la primitive existante,
   pas une nouvelle formule. Metrique "taches servies" = COUNT des lignes
   kudos du node (= resultats credites/valides-quorum).
6. **GPU-heures : libelle honnete + non-agregation.** Lire `usage.json`
   `hours_today` (consent.rs) cote worker local, libeller "heures donnees par
   CETTE machine (non attestees)", JAMAIS agreger cross-node ni classer reseau.
   Reutiliser le ton de Network.tsx (GpuCard).
7. **Front : carte contributeur dans Network.tsx**, reutiliser GpuCard +
   ProjectsServedCard, hook React Query + enveloppe Zod `.strict()` (pattern
   `searchBrowse()`/leaderboard existant).
8. **THREAT_MODEL : ajouter cellule §15.3** (surface reputation gaming +
   GPU-heures locales + sanity-bound) — autorise par la regle d'evolution §16.

## Scope confirme (ce qui suit le plan tel quel)

- 2e vue d'agregation keyee `worker_node_id` sur le ledger kudos EXISTANT,
  reutilisant `effective_score()` (EMA alpha=0.97) — primitive INCHANGEE.
- 3 metriques honnetes : kudos effectifs (EMA) / taches servies (lignes
  validees) / GPU-heures LOCALES (`usage.json`, non-attestees, jamais
  repliquees).
- Kudos NON-MONETAIRE, 0 token crypto — decision GELEE respectee (0 vocabulaire
  monetaire introduit).
- Self-view per-node + leaderboard per-projet pre-existant ; PAS de ranking
  global reseau-wide (rejet EigenTrust tenu ; reconnaissance publique reseau =
  post-launch).
- 0 nouvelle dependance (Rust + npm), 0 bump wire, 0 nouvelle migration.
- `validator` (`validate_quorum_pre_guardrail`) INCHANGE — verrou S76-C/D tenu.

## Risques residuels / a surveiller en review

- **R1 (BLOQUANT si non corrige) :** GPU-heures mal libellees (presentees comme
  metrique reseau verifiable / agregees cross-node / classees) => sur-promesse.
  Verifier le libelle "CETTE machine (non attestees)" et l'absence de toute
  agregation cross-node.
- **R2 (BLOQUANT si non corrige) :** route ajoutee a `public_routes` au lieu de
  `authed_routes` => fuite du gate loopback. Verifier la presence sous
  `auth_required`.
- **R3 (CONFLICT si fait) :** ajout d'une route de classement reseau-wide
  tous-nodes confondus (ranking global) => viole le rejet EigenTrust. La
  self-view per-node et le leaderboard per-projet ne le sont pas.
- **R4 :** sanity-bound sur-vendu comme defense anti-Sybil. C'est un
  plausibility-check self-consistant (les deux champs sont dans le meme payload
  signe). Documenter la limite (§15.3) pour eviter une fausse-promesse.
- **R5 :** `TOKENS_PER_MS_CEILING` mal calibre => clampe des resultats honnetes
  (faux positif). Choisir un plafond genereux (borne anti-absurdite, pas une
  mesure de debit reelle) et documenter le choix ; ajouter un test sur une
  valeur plausible non-clampee + une valeur absurde clampee.
- **R6 :** test #3 ecrit comme "creer l'index" au lieu de "verifier l'USAGE"
  (`EXPLAIN QUERY PLAN`). L'index existe deja (db.rs:50).
- **R7 :** oubli de mettre a jour les ~13 appels de test de `credit()` apres
  l'ajout du param `generation_time_ms` (compile-break sinon — detecte au
  build, mais a anticiper).
