# Preflight G8 — Sprint 83 Phase L — Permissions et atténuation

Date : 2026-07-24. Tip au scan : `9498e3fa` (commit planning-only « vue PO »
EMPILÉ au-dessus de Phase K `90921211`) ; le dernier commit de CODE reste
Phase K. Arbre propre (`git status --porcelain` vide).
Méthode : Workflow ultracode — 8 agents Opus 4.8 1M (5 scans factuels
S1a-OSS / S1b-deps / S2-histoire / S3-threat / S4-wire + 3 lentilles
adversariales réfutation-des-faits / design-conflict / complétude). Synthèse
main-thread. Scans morts / à rejouer : **aucun** — les 8 rapports ont rendu.
Vérification adversariale : 16/18 claims load-bearing CONFIRMÉES par
ré-ouverture des fichiers ; 1 réfutée mineure non-load-bearing (le « tip »
du brief = Phase K, réalité = `9498e3fa` planning) ; 1 confirmée avec
imprécision de rédaction (2 corrections de numéro de ligne consignées §7).

> Note de provenance : le preflight a d'abord été lancé sous Fable 5 ; les 5
> scans factuels ont abouti puis les crédits Fable se sont épuisés. Reprise
> `resumeFromRunId` sous Opus 4.8 1M (ID cible frontmatter agents) : les 5
> scans ont rejoué depuis le cache, les 3 lentilles adversariales + la
> synthèse ont tourné en live. 9/9 agents `done`, 0 erreur.

## Verdict : PLAN-ADAPT

Le scope L.1/L.2 est intégral (AM-4 ne l'ampute pas — L = permissions/
atténuation, L1 = coffre + HITL, confirmé `sprint83_plan.md:930` +
`:1014-1015`) et le socle C/H/J/K le porte déjà : la couche 5 existe en
lecture seule (`capability_closure`), l'algèbre canonique est commitée
(`grant_v0.rs`), le carry label J est CLOS en K. Aucune décision gelée
n'est en conflit (les trois lentilles adversariales ont réfuté chaque
candidat de DESIGN-CONFLICT : `SENSITIVE_ACTIONS` non touché, 0-dep tenu,
census 29 intact, len==7, §19.3, EffectTarget gelé, carry J clos). Mais le
plan a été écrit AVANT C/H/J/K et sous-spécifie huit points que le code et
la recherche rendent obligatoires — dont DEUX pièges que les 5 scans ont
manqués et que seule la lentille design-conflict a levés (`depth_enforced`
sans référent ; `seventeen_classes_typed` lexicalement insatisfiable). Plus
UNE décision-à-signaler (frontière module-pur vs câblage `run_effect`) dont
le défaut in-charte suit le plan-tel-qu'écrit ; l'option lourde basculerait
en expansion de scope à arbitrer.

## 1. Socle vérifié (ce que L n'a PAS à construire)

- **Algèbre canonique d'atténuation, déjà commitée** : `grant_v0.rs:8`
  `effective = declared ∩ grant ∩ local-policy (∩ consent, inert v0)` ;
  `None` = élément d'identité de l'atténuation monotone (`:78-79`) ; expiry
  fail-closed À LA BORNE `now_ms >= exp` (`:64`, test `expiry_refused`
  refuse `now==1000` `:162`) ; `revoked` re-vérifié à CHAQUE appel, jamais
  caché (`:49`, test `:168`) ; borne domaine des deux côtés (`:99`) ; test
  `policy_layer_attenuates_only` (`:231`). L MIROITE ces invariants (voir
  PA-1, la fonction elle-même n'est PAS réutilisable — mauvais domaine).
- **Couche 5 « fermeture du plan compilé » livrée par H, en LECTURE SEULE** :
  `capability_closure: BTreeMap<String, Vec<String>>` (`compile.rs:600-604`,
  doc « membership is DATA for the runtime, per-effect enforcement binding
  is Phases J/K/L »), pinnée goldens + snapshot STRICT. L la CONSOMME, ne
  la recalcule pas. Refus statiques `CapabilityNotRequested` /
  `CapabilityNotInManifest` déjà côté compilateur.
- **Borne de profondeur EXISTANTE réutilisable** : `MAX_PLAN_DEPTH = 16`
  (`compile.rs:104`) = le seul objet de « profondeur » in-scope L (voir
  PA-3 ; la profondeur de DÉLÉGATION est hors-MVP).
- **Doctrine double-sémantique déjà écrite** : `grant_v0.rs:14-17` « no
  inert stubs: absence of a layer imposes no constraint = identity
  element » ; `:73-75` `Policy = None` inert permissif DISTINCT de
  `Some(∅)` qui refuserait tout. C'est le patron exact de la couche 7 HITL
  sans HITL (PA-2).
- **Point de branchement couche 6 propre, aucun refactor de surface gelée
  requis** : `EffectContext` (`activity.rs:68-84`, **5 champs** :
  `key`/`attempt`/`fencing_epoch`/`deadline_ms`/`input`) ne porte **aucun**
  champ capability ; `orchestrator.rs:17` « per-effect capability binding is
  Phases K/L », `:59` « Capability attenuation is L » ; choke point net
  `run_effect` (`orchestrator.rs:1097` via `:984 dispatch`). La couche 6 se
  DÉRIVE du `capability_ref` déclaré du nœud dans le plan compilé (lecture
  seule), PAS d'un champ `EffectContext` neuf (couche 6 = donnée, GAP-6).
- **Carry label J DÉJÀ CLOS en K** : `MAX_DECISION_LABEL_BYTES = 256*1024`
  (`compile.rs:125`, doc « closing the J carry ») == `MAX_SIGNAL_PAYLOAD_
  BYTES` (`store.rs:314`), assert engine `effects.rs:2131`. L n'a RIEN à y
  fermer.
- **17 classes = terrain vierge** : `grep read_public|write_draft|…` = 0
  partout, enum créé de zéro, 0 collision. `grep "mod capability"` engine
  = 0 → le module et le préfixe de test conditionnent la non-vacuité du
  gate (PA-5).
- **Baseline VERTE** : `cargo nextest run -p sbfb-workflow-engine --locked`
  = 116 passed / 7 skipped (1.5 s, reproduit adversarialement à
  l'identique). Les 7 skips = enfants `#[ignore]` out-of-process,
  classe consignée depuis Phase E.
- **Prior-art externe (support, non re-vérifié en repo — conclusion ancrée
  LOCALEMENT par grant_v0)** : la composition conjonctive/intersective est
  le mécanisme canonique d'atténuation monotone (Biscuit blocs+checks,
  Macaroons caveats, UCAN chaînes) ; `min` des expirations = règle UCAN ;
  audience = égalité fail-closed par maillon (pas un set) ; la mort du
  rights-system WASI preview1 conforte la granularité 17 classes sémantiques
  vs bitflags. Ces citations ne gouvernent PAS le gate ; elles cadrent
  l'implémentation.

## 2. Adaptations PLAN-ADAPT (le code suit CES corrections)

**PA-1 — Deux algèbres évitées par MIROIR, jamais par appel de fonction ni
généralisation (le « vrai danger » désigné).** L'engine ne dépend PAS de
`sbfb-manifest` (`crates/sbfb-workflow-engine/Cargo.toml:11-30` : deps =
`sbfb-workflow-core` + serde/serde_jcs/blake3/rusqlite/thiserror). Or
`grant_v0` vit dans `sbfb-manifest` et son résultat est borné à
`bridge_method_allowlist()` = **16 verbes bridge** (`grant_v0.rs:99`),
mauvais domaine pour 17 classes d'effet. Donc `effective_capabilities`
engine-side (L.2) EST forcément une 2e algèbre. Discipline : (i) reproduire
VERBATIM les invariants documentés de grant_v0 (`None`=identité, `now_ms >=
exp`, `revoked` per-call, borne domaine des deux côtés) avec un test
épinglant les MÊMES conventions de frontière + cross-ref doc vers
`grant_v0` ; (ii) NE PAS appeler `effective_methods` (domaine bridge) ;
(iii) discipline partagée = tests miroirs, PAS code partagé (domaines
réellement distincts). **Écarter « généraliser grant_v0 »** : cela touche
la paire GELÉE C `grant_v0.rs`↔`web/src/bridge/authority.ts` dont la parité
d'ALGÈBRE n'est PAS machine-gatée (le drift-gate ne couvre que la parité
allowlist, `check-frontier-contracts.sh:334-376`) → refactor = drift TS
silencieux. Consommer `grant_v0` en référence exige au plus
`sbfb-manifest = { path }` (intra-workspace, 0 nœud lock) ; par défaut,
miroir pur sans dep.

**PA-2 — Couche 7 « décision HITL valide » sans HITL (AM-4) : DOUBLE
sémantique d'absence, à écrire explicitement.** L1 possède tout le rendu/
signature/file HITL (`plan:967-1015`) et la visibilité UI est Phase U. Le
piège : `None = identité` (doctrine `grant_v0.rs:14-17`) est correct pour
une classe SANS exigence d'approbation, mais une classe AVEC exigence
(le « niveau d'approbation » est une donnée de la capability ref,
research `:1261`) + décision absente/expirée doit donner l'ensemble VIDE
(`Some(∅)`, fail-closed), JAMAIS identité (leçon absence-de-couche ≠
absence-dans-couche). Représentable dès L sans producteur HITL : couche 7 =
ensemble des décisions valides (VIDE en L) + exigence lue dans la ref +
`Option<HitlDecisionRef>` opaque ; L1 fournira le producteur réel. Sans
cette écriture, un `None=identité` naïf fail-open sur les classes à
approbation = P1 assuré.

**PA-3 — `depth_enforced` re-défini contre un référent IN-SCOPE (piège
manqué par les 5 scans).** La profondeur de DÉLÉGATION est hors-MVP :
research §14.5 (`:1295-1305`) fait de `DelegationGrant.v1` « un saut au
MVP » (inexistant ; seul `DelegationCert` existe, `delegation.rs:1-35` =
bridging clé-de-forge Couche 3, PAS un grant d'activité atténuable) ; la
garde `depth/cycles/budgets` des chaînes app.call est **Phase Y**
(`plan:1657` + `:1668`, gate `G-APPCALL`). `depth_enforced` n'a donc AUCUN
objet de délégation à borner en L. Le lier à un référent EXISTANT : soit la
borne de nesting de la couche 5 (`MAX_PLAN_DEPTH=16`, `compile.rs:104`),
soit reformuler « profondeur » comme la propriété de chaîne monotone
(empiler N couches ne peut que rétrécir — généralisation de
`intersection_only_attenuates`). INTERDIT : tester une delegation depth
(§14.5 différée) ou préempter la garde app.call de Y.

**PA-4 — `seventeen_classes_typed` teste le MÉCANISME, pas la lexie (piège
lexical manqué par les 5 scans).** Les classes `commit` et `push` sont
littéralement 2 des 17 (research §14.2:1257-1258) et SONT 2 des 4
`SENSITIVE_ACTIONS` (`operator_server.rs:37` = `["shell","commit","push",
"PASS"]`). Un test naïf « le module capability ne contient aucune des 4
chaînes » est INSATISFIABLE — l'enum DOIT nommer ses variants `commit`/
`push`. Assertion correcte : le moteur capability **n'importe/ne référence
pas** `operator_server::SENSITIVE_ACTIONS` et ne fait AUCUNE autorisation
par substring (mécanisme), PAS un grep de source dépourvue de « commit »/
« push ». Sans cette précision, soit le test est mal conçu, soit Codex/
review rejette une classe correctement nommée.

**PA-5 — Property tests hand-rolled 0-dep + module nommé `capability` + lois
meet-semilattice + meet PAR CHAMP.** `proptest`/`quickcheck`/`arbitrary`
ABSENTS de `Cargo.lock` (0 match) → « proptest-style » (`plan:940`) = le
style, pas la crate : générateurs seedés déterministes hand-rolled
(précédents : énumération sur domaine clos `grant_v0.rs:186-193` ;
échantillonnage blake3-seedé SEEDER-DIAL-TAIL). Lois à couvrir :
atténuation `eff(L∪{l}) ⊆ eff(L)`, idempotence, commutativité/associativité
(ordre indifférent), identité de l'absence, absorbant `∅`. **Meet par champ,
direction PAR CHAMP** : §14.2 refs portent schemas/destinations/quotas/
timeout/approbation → l'intersection est un MEET défini par champ dont la
direction s'inverse : `min(quota)`, `min(timeout)`, `∩(destinations)`, mais
**`max(niveau d'approbation)`** (plus strict = plus haut) ; un `min`
copié-collé sur l'approbation = union accidentelle invisible à un test
class-set-only. Module MUST be named `capability` : gate `G-CAPABILITY` =
`cargo nextest run -p sbfb-workflow-engine capability` (`gate_matrix:42`),
un filtre nextest sans match = EXIT 4 (précédent Phase E).

**PA-6 — WA-T5.2 a DEUX jambes : test nommé `undeclared_capability_refused`
distinct de `expiry_enforced`.** Le palier T2 WA-T5.2 (`plan:955`) =
« refus grant expiré **ET** capability non déclarée ». L.3 nomme
`expiry_enforced` (jambe 1) mais AUCUN test nommé ne couvre « capability non
déclarée refusée » (jambe 2) au niveau de l'intersection (couche 5 présente
+ ref absente → ∅ fail-closed) — le refus statique existe côté COMPILATEUR
(H) mais WA-T5.2 est un palier RUNTIME. Sans test explicitement nommé, AI2
ne pourra citer d'evidence L pour la moitié de WA-T5.2. Ajouter
`capability::tests::undeclared_capability_refused`. **17 classes = données
host/engine keyed by `capability_ref`, 0 champ pack/graph neuf** : le
descripteur riche §14.2 NE doit PAS entrer dans le pack SIGNÉ (refs =
`Vec<String>`, `pack.rs:420`, tag FRONTIER `pack.rs:391`, format F gelé).

**PA-7 — Réconciliation `sprint83_gate_matrix.md` L→L1 : livrable de
fermeture OBLIGATOIRE (P1 manqué par les 5 scans).** La matrice, figée au
cadrage Phase A, est ANTÉRIEURE à la re-coupe AM-4 : rows 9/10/11
(`gate_matrix:43-45`, `G-SECRET`/`G-SECRET-VAULT`/`G-HITL`) et row 44
(`:78`, `G-A11Y-HOST`) déclarent owner **`L`** alors que le plan L1.5
(`:1014-1015`) fait fermer G-SECRET-VAULT/G-SECRET/G-HITL par **L1**
(`grep -c "L1" gate_matrix` = 0, confirmant la staleness). Si le commit L
ferme `G-CAPABILITY` seul en laissant la matrice, L apparaîtra comme
laissant **4 gates orphelins** — divergence plan↔matrice qu'un auditeur/
Codex signalera (invariant « 0 orphelin » §4.1 lu par l'audit gate S84). Le
commit L (ou un micro-lot planning-hygiène juste avant) DOIT reassigner
rows 9/10/11 `L→L1` et row 44 `L/Q→L1/Q`, sous l'invariant §4.4 « statut
re-vérifié ». Ce n'est PAS une question PO (AM-4 est déjà pris) — une
réconciliation d'artefact.

**PA-8 — Flips doc-honnêteté au commit L (le gate anti-promise ne les
attrape pas — formes verb-free).** Basculer vers le passé immuable les
anchors « Phase L » que L satisfait, en conservant EXPLICITEMENT les moitiés
L1 : code — `orchestrator.rs:17/:59`, `compile.rs:27`, `agent_definition.rs`
« sensitive-effect classes are Phase L/L1 » ; docs — `docs/workflow/
llms.txt:15-16` (déplacer « Permissions (L) … later phases / Not evidenced »
vers les livrées, le fichier grandit PAR phase), `WORKFLOW_AGENTS_
OVERVIEW.md:44` (statut du trait `policy`), `THREAT_MODEL.md:1725/:1736`.
**Requalification PARTIELLE HONNÊTE** des trois promesses « Phase L rendra
la reconnaissance précise » (`compile.rs:138`/`:171-176`, `THREAT:1727`) :
elles vivent côté COMPILATEUR (core) et le format pack est gelé ; L livre
engine-side → écrire « le raffinement runtime engine-side ferme le gap en
défense-en-profondeur ; la borne nominale STATIQUE du compilateur reste
documentée telle quelle », JAMAIS « fermé statique » (sur-claim = classe
d'erreur CI-1 S82).

## 3. Décision-à-signaler (défaut in-charte pris ; l'option lourde = expansion de scope à arbitrer)

**DS-1 — L = module autoritaire PUR (défaut, plan-consistant) ; le câblage
`run_effect` + l'application du P0 « intersection AVANT dispatch » = phase
ultérieure TRACÉE.** Le plan-tel-qu'écrit scope L en module : L.2 =
moteur + trait `policy` + « câblage des grants release/pack/plan/effet
distincts » (= câbler les 4 SOURCES de grant EN ENTRÉE de l'intersection,
pas l'intersection dans la boucle de dispatch) ; L.3 = property tests
`capability::*` ; L.4 acceptance = property tests ; **T1 = N-A**, **T2 =
NOT-RUN jusqu'à AI2**. `EffectContext` (`activity.rs:68`) n'a aucun champ
capability et « per-effect capability binding is Phases K/L » reste au
conditionnel.
- **DÉFAUT PRIS (in-charte, suit le plan)** : L livre le module pur + les
  property tests + les 4 grants distincts en entrée. Les tests d'autorité×
  retry/replay que S1a/S3 réclament (`revoked_between_retries_refused`,
  `replay_never_reauthorizes`) N'ONT PAS de call-site en L → ils deviennent
  des **notes de conception pour la phase de wiring**, pas des tests L
  (graver : intersection recalculée à l'émission, par-attempt, `now_ms`
  watermarké ; refus d'autorité JOURNALISÉ comme décision ; le replay lit
  le journal, jamais la policy live — sinon réouverture de StepHashMismatch/
  TOCTOU-retry fermés v25/v26). Conséquence à TRACER explicitement, sinon
  carry P1 « invariant ratifié sans point d'application » : le point
  d'enforcement du P0 « intersection avant verbes/dispatch » (D4,
  `kickoff:883`) = la phase qui shippe la surface opérateur/runtime
  (V « Centre des Runs » / U), à inscrire au plan + row THREAT.
- **OPTION LOURDE (à arbitrer AVANT tout Edit si le PO veut l'enforcement
  DANS L)** : L câble l'intersection dans `run_effect` et threade la
  couche 6 via une EXTENSION de `EffectContext` — cela touche la surface
  effet K (pattern PA-1-de-K « trait/champ NEUF à côté du gelé », jamais
  muter `EffectTarget` gelé E), et déplace T1/T2 (l'acceptance ne serait
  plus « property tests seuls »). = expansion de scope, à ratifier.

## 4. Obligations héritées & livrables docs-contrat — traitement dans L (aucun defer silencieux)

| Carry / obligation (provenance) | Traitement L |
|---|---|
| Borne label vs String compilateur « K/L » (J §3.2) | **CLOS en K** (`compile.rs:117-125`, assert `effects.rs:2131`) — L vérifie seulement qu'aucun doc ne le rouvre |
| Couche 5 fermeture plan compilé (H `compile.rs:600-604`) | CONSOMMÉE en lecture seule (une couche de l'intersection), jamais recalculée |
| 3 promesses « Phase L rendra précis » (`compile.rs:138/:171-176`, `THREAT:1727`) | PA-8 — requalification PARTIELLE honnête (raffinement runtime engine-side ; borne statique compilateur documentée telle quelle) |
| Gate matrix rows 9/10/11/44 owner `L` (stale pré-AM-4) | PA-7 — reassignation `L→L1` / `L/Q→L1/Q`, livrable obligatoire |
| Anchors doc « Phase L » verb-free (`llms.txt:15`, OVERVIEW:44, code, THREAT) | PA-8 — flip vers passé immuable, moitiés L1 conservées |
| Couche 1 « OS capability toggle » §14.1 | **Position écrite** : `BTreeSet` opaque INJECTÉ par le trait host (précédent `Clock`/`SignalSource` injectés J ; charte `lib.rs:1-8`, l'engine ne lit ni env ni identité) ; L n'importe rien de `CAPABILITY_TOGGLES.md` (GAP-4) |
| `docs/security/CAPABILITY_TOGGLES.md` Python-era périmé (S22, `packages/nexus-coordinator` supprimé S50-51) | **Doc-dette signalée, non-bloquante L** : bandeau « superseded / Rust-era à ré-écrire » = phase docs ou question PO de routage, HORS scope L |
| Couche 6 « capacité de l'effet courant » | Dérivée du `capability_ref` déclaré du nœud dans le plan compilé (lecture seule) ; PAS un champ `EffectContext` neuf (surface K gelée) — GAP-6 |
| Persistance des grants (`grant_v0.rs:35-38`) | Si L persiste : serde defaults OBLIGATOIREMENT fail-closed (jamais `#[serde(default)]` permissif sur `revoked`/`expires_at_ms`). Si L ne persiste pas : documenté comme borne honnête de la sous-section |
| Dep engine→manifest | À noter : miroir d'algèbre par défaut (0 dep) ; `sbfb-manifest = { path }` si référence directe (0 nœud lock) — PA-1 |
| LOOPBACK / T1 posture | Aucune row L attendue (rows mutantes re-routées Phase V) ; L engine-only, `T1=N-A` ; à confirmer d'une ligne au commit (miroir posture K) |
| Principal HITL signé (T-RUN-SIGNAL `THREAT:1808`) | Reste **L1** (pas L) — L représente la décision comme donnée opaque, ne la signe pas |

## 5. Menaces — rows à ajouter (THREAT v26, nouvelle §19.3 « Surface permissions et atténuation »)

Suivre le pattern §19.1=J / §19.2=K (pas de renumérotation top-level).
Rows candidates : **T-CAP-WIDEN-BYPASS** (une couche élargit / un chemin
d'émission saute le choke point — type opaque, constructeur = intersection
UNIQUEMENT, aucun `union`/`extend`/`insert` public) ;
**T-CAP-EXPIRY-TOCTOU-RETRY** (une réémission K est une nouvelle émission →
recompute par-attempt, `now_ms` watermarké ; contrainte gravée pour la
phase de wiring, DS-1) ; **T-CAP-REPLAY-REAUTHORIZE** (l'autorité gate
l'ÉMISSION côté non-rejoué ; un refus = décision journalisée ; le replay lit
le journal, jamais la policy live — sinon StepHashMismatch/faux positif) ;
**T-CAP-GRANT-AUDIENCE-CONFUSION** (grant release appliqué à la version pack
ou inverse = élargissement silencieux ; égalité fail-closed par couche,
property test audience) ; **T-CAP-CLASS-UNKNOWN-FAILOPEN** (17 classes =
ensemble FERMÉ ; ref sans classe mappée ou `destructive` mal-mappée →
refus, jamais défaut-à-`read_public`) ; **T-CAP-DEPTH-AMPLIFICATION**
(chaîne monotone / `MAX_PLAN_DEPTH` — PA-3, la delegation depth reste §14.5
hors-MVP). **Flips** : `THREAT:1725` (« l'intersection 7 couches = Phase L »
→ LIVRÉ §19.3) ; `:1736-1737` (scinder part L livrée / part L1 restante) ;
`:1727` et `:190` requalification PARTIELLE (PA-8, R1/R2). **Asset A13 —
grants et autorité effective** versé §2 (Intégrité Critique : grant élargi/
forgé = EoP directe ; Disponibilité moyenne : fail-closed, absence ⇒ refus,
jamais déni d'un run clos) + entrée historique **v27**. Contraintes suivies
(pas de rows neuves) : duress — AUCUN gate dans l'engine (doctrine I1,
`THREAT:1824-1827`, trait policy INJECTÉ) ; journal digest-only par TYPE ;
**NE PAS fermer** le residual Operator `THREAT:933` (SENSITIVE_ACTIONS non
étendu, L n'importe pas `operator_server.rs`).

## 6. Impacts wire / census / goldens / frontière

- **Wire-touching : NON.** Toutes les surfaces sont locales non signées
  (`CompiledPlan` dérivé, journal `RunEvent` local), non sérialisées
  (`Grant` sans serde), ou lues sans modification (couche 5). L.3 ne liste
  que des tests `capability::*` — rester HORS journal (pas d'entrée
  `RunEvent` neuve).
- **0 bump wire** : `PACK_FORMAT_VERSION` (`pack.rs:84`), `RUN_EVENT_FORMAT_
  VERSION` (`run_event.rs:65`), `SNAPSHOT_FORMAT_VERSION` (`store.rs:321`),
  `FEED_FORMAT_VERSION` (`public_feed.rs:20`) tous encore `= 1`.
- **Census DOMAIN 29 INTACT** : les grants L ne sont pas signés (HITL signé
  = L1) → 0 `const DOMAIN_*`. `check-frontier-contracts.sh` (name-list gelée
  `:44/:203/:221/:254`) non éditée ; à re-prouver `clean` au commit.
- **Registre contrats 7/7 INTACT** : `CONTRACT_NAMES: [&str; 7]`
  (`contracts.rs:148`), test `obj.len() == 7` (`:162`), 7e =
  `WorkflowPublicProjection` stub `{}` (`contracts.rs:103`, correction de
  ligne §7). Le moteur capability = structures host in-memory, PAS une
  identité de contrat.
- **Goldens byte-intacts attendus** : `contracts.schema.json`,
  `babel/conception.{workflow,compiled}.golden.json`,
  `run_event{,_orchestrated,_terminal}.golden.json` — vérifiable
  `git diff --stat` vide sur ces chemins.
- **Parité TS intacte** : `web/src/bridge/authority.ts` + fixtures
  `pack_digest.test.ts` non modifiés (sinon vitest web + review manuelle de
  la claim miroir — la parité d'algèbre n'est PAS machine-gatée, PA-1).
- **Frontière docs-contrat (test-acteur §6.12) : AUCUNE nouvelle** — le
  trait `policy` est un trait Rust intra-workspace, pas une primitive
  sérialisée lue par un runtime distinct. Commit body : Pre-launch protocol
  = « 0 wire », aucune étiquette FRONTIER requise. `llms.txt`/OVERVIEW
  grandissent PAR phase (PA-8), la clôture chantier-wide reste AJ1.
- **Charte** : core reste PUR ; engine reste synchrone/sans-tokio (set
  algebra pure, `now_ms` injecté, aucune lecture ambiante).

## 7. Rectificatifs mineurs (adversarial-facts) & signal méta

- **Rectificatif R-1** : le stub `WorkflowPublicProjection` est à
  `contracts.rs:103`, PAS `:26-28` (S2 F7 imprécis ; `:26-28` = prose
  census). Substance (7e identité non remplie, len==7) intacte.
- **Rectificatif R-2** : `EffectContext` a **5 champs** (aucun capability),
  pas « `fencing_epoch` SEUL » (S2 F4 mal formulé). Partie load-bearing
  (pas de binding capability, couche 6 part de zéro) intacte.
- **Rectificatif R-3** : le brief nomme `90921211` « tip » ; réalité = tip
  `9498e3fa` (planning-only) au-dessus de Phase K `90921211`. Impact nul
  (aucun code touché).
- **Signal méta** : PLAN-ADAPT consécutifs (I1, J, K, L) — attendu et
  déclaré : le plan 43 phases a été écrit avant C/H/J/K ; chaque adaptation
  de L découle d'un fait créé par une phase postérieure au plan (couche 5
  livrée H, absorbance/journal K, algèbre commitée C) OU d'une staleness
  d'artefact (gate_matrix pré-AM-4). Aucune ne touche une décision Day-0 ni
  un invariant de sprint. Les deux pièges manqués par les 5 scans (PA-3
  depth, PA-4 lexie) et le P1 gate_matrix (PA-7) confirment la valeur de la
  triple lentille adversariale. Signal surveillé.
