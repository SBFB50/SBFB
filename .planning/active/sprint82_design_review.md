# Sprint 82 — Design Review Board (G1)

> **STATUT : ACTIVÉ le 2026-07-12 — les deux conditions du board sont jouées.**
> (1) Phase 0 = audit gate S81 **DÉJÀ JOUÉ** : **FAIL (0 P0, 4 P1, 16 P2, 14 P3)
> → GATE LEVÉ voie A** (`ad53940` + `95ff46c` ; ne PAS re-jouer ; carry P1
> S81-G-ESC-1 routé S82 par construction) ; (2) les 9 arbitrages PO-1..PO-9 sont
> **tranchés à l'ouverture**. **UNE décision CONTRE la reco du board : PO-2 —
> benchmarks INCLUS en S82** (Phase B + amendement canon T3) alors que la synthèse
> G1 (D5) **recommandait de les différer**. Le scoring D5 conserve cette tension
> (évalue la reco « défer », note l'override PO comme autorité). Les autres
> divergences spec↔board (D3 granularité, D6 lettre de phase, ordre A→T) sont des
> **résolutions** du critic, pas des overrides.

> **Méthode (ultracode).** Kickoff S82 orchestré en Workflow : census 7 axes →
> G1 design board (perspective indépendante, scoring 0-5) → completeness critic
> adversarial (`s82_critic.json`, **NEEDS-ADJUSTMENT**) → arbitrage PO. **Pas de
> rubber-stamp** : les 4 ajustements durs sont portés au PO et tranchés ; les
> faits load-bearing (http.rs 12460 l, runtime.rs 5096 l, driver ONE-SHOT, ancres
> task_response.rs, 11 modules `*_api.rs`) re-vérifiés au disque ce jour
> (restitués §Réconciliation). **Sprint** : S82 — **dette docs-contrat +
> refactorisation** (C9 tranché PO 2026-07-11 ; workflow-engine + Viewer DÉCALÉS).

## Verdict board : GO CONDITIONNEL → LEVÉ à l'ouverture

**7/12 PASS** (D1/D4/D6/D7/D8/D9/D10) + **5 CONDITIONAL** (D2/D3/D5/D11/D12) +
**0 CONFLICT**. Aucune décision ne contredit une Day-0 gelée ni la pre-launch
policy (0 bump wire SBFB ; iroh reste `=1.0.1`). Le board (`s82_synth.json`)
émettait un **GO CONDITIONNEL** subordonné à deux réserves dures (PO-1 mode de
fermeture boot-SEED ; PO-3/PO-4 périmètre sharding-debt + condition push) et le
completeness critic le requalifiait **NEEDS-ADJUSTMENT** sur **4 ajustements
durs**. **Les 4 sont RÉSOLUS à l'ouverture** par les décisions PO tranchées :

1. **Granularité commit du split http.rs** (Phase M synth = 4 domaines/4 commits,
   violation 1-commit-par-phase) → **D3 : 1 domaine = 1 commit = 1 phase**
   (Phases N→S, 6 domaines). Le sprint grandit à **20 phases** (README §4 ne
   plafonne pas). RÉSOLU sans déviation canon.
2. **Migration stores worker S81-G-1** (findings : « thème S82 obligatoire »,
   relégué à un scope-cut sans slot) → **D12 : rattachée Phase T** avec artefact
   `sprint82_t2_store_migration.json` (vérif 3 nœuds au push). RÉSOLU, plus « en
   marge ».
3. **hickory sans phase provisionnée** si PO-7=A → **PO-7=A ⇒ Phase K dédiée**
   supply-chain bornée. RÉSOLU.
4. **Fermeture §6.2.1 boot-SEED quand (c) live=BLOCK{rig}** → **PO-1=B tranche la
   voie STRICTE** : (c) live PASS EST exigée ; rig indispo ⇒ escalade PO
   explicite, JAMAIS un 4e report sec silencieux. RÉSOLU.

Sous ces résolutions, le sprint **FERME ses items par construction** (critères
machine-checkables par type de phase), respecte « sprint dette nommé, jamais
bundlé » (doc-dette éclatée E→J), et est BORNÉ (scope cuts explicites). **Réserve
résiduelle assumée par le PO** : la CHARGE (20 phases, 4 axes ambitieux, deux
workstreams high-risk) — mitigée par la fermabilité per-phase + le clustering rig
A+B + le golden refacto. Aucun obstacle de conception n'empêche d'ouvrir Phase A.

## Scoring par décision

| Décision | Score | Verdict | Note |
|---|:--:|---|---|
| **D1** Ordre : boot-SEED Phase A / bench Phase B / CI Phase C avant refacto+push | 4/5 | PASS | Phase A **CI-INDÉPENDANTE** (rust-ci 3-OS vert, 0 GTK) — lève la seule objection à boot-SEED-first malgré la CI cassée. §6.2.1 (OVERDUE 3/3) interdit un report sec. |
| **D2** Design unifié boot-SEED (broadcast=HINT, état durable=VÉRITÉ) | 3/5 | CONDITIONAL | **RISQUE PRINCIPAL (high).** Design SAIN (critic confirme les 3 exit conditions + red revert-proof), driver ONE-SHOT confirmé `http.rs:1819-1826`. Réserve dure : clôture pleine contingente à (c) LIVE + dépendance rig (PO-1=B). |
| **D3** Split http.rs incrémental, **1 domaine = 1 phase** (N→S) | 4/5 | CONDITIONAL | Granularité critic RÉSOLUE (fini le Phase M à 4 commits). Risque **high** rebase (arc front `provider_router.rs` + axe sharding) ; mitigé golden Phase M + count invariant + test module ~7915 l co-déplacé. |
| **D4** Gate machine d'invariance pour TOUT refacto | 5/5 | PASS | fmt --check + clippy --all-targets -D warnings + nextest count >= baseline (Win 2095/Docker 2099) + 0 route path + 0 bump wire + golden splits. Le count constant EST la preuve de comportement préservé. |
| **D5** Benchmarks **IN S82** (Phase B) + amendement canon T3 | 3/5 | CONDITIONAL | **Override PO (PO-2).** Le board **recommandait de différer** (ni docs-contrat ni refacto, rig-dépendant, amendement canon lourd). PO tranche IN. Score reflète la tension : rig-gated `BLOCK{rig}`, jamais RIG-ABSENT. |
| **D6** Sharding-debt = doc-contrat SEUL (SCHEMAS-SHARD-REQ Phase G) | 4/5 | PASS | Cœur sharding LIVE-prouvé ; seule asymétrie de contrat (requêtes non schématisées vs réponses schématisées) relève du thème. R-J-6/F2/SI-12/N3-reveal fermement différés. |
| **D7** LOOPBACK §3 = tier-target représentatif verrouillé | 4/5 | PASS | Front-matter representative-lock + trigger incrémental. Exhaustif = volume mécanique faible signal ; risque réel (frontière neuve échappe) couvert par D8. |
| **D8** Registre FRONTIER = accept-and-close incrémental + métrique figée | 4/5 | PASS | Grep déterministe committé (fin du flottement 21/22/23) + tag `// FRONTIER:` exigé pour toute frontière NEUVE (dont les 3 request-bodies Phase G). Ferme la boucle de reports (S80-G-1, 3 reports). |
| **D9** Ledgers PATTERNS = re-audit COMPLET (~80 tickets) | 5/5 | PASS | Bon marché (docs pure, 0 risque code), cœur du thème. Principe sessions-fraîches : re-vérifier chaque item différé. Purge zombies Python (`git ls-files packages/`=0) + collision T15/T16. |
| **D10** Hors-thème NON codé (statué au ledger seulement) | 5/5 | PASS | T20-wire/T21/T23/T25/T26/T27/nginx-DRY/firewall + veilles supply-chain = features/durcissements trigger-driven. EXCEPTION assumée : hickory (PO-7=A). |
| **D11** hickory-resolver 0.24→0.26 **IN S82** (Phase K) | 4/5 | CONDITIONAL | Risque **med** (churn API resolver 0.25 dans `dns_fallback.rs`). Slot refacto = bon moment (blocage S81 « iroh SEUL » levé). Critère : `cargo deny check advisories` vert (4 ignores retirés, 4 RUSTSEC clos). |
| **D12** Migration stores worker S81-G-1 → Phase T + artefact T2 | 3/5 | CONDITIONAL | Résout le critic gap #2 (plus « en marge »). Risque : vérif **live-ops one-way** dépendante du push 3 nœuds ; artefact `sprint82_t2_store_migration.json` PASS obligatoire. |

## Détail par décision (perspective indépendante)

- **D1 — Ordre des phases (4/5 PASS).** boot-SEED en Phase A (première substantielle),
  benchmarks Phase B clusterés rig-chaud, CI Phase C avant refacto et push. Le point
  décisif : **Phase A ne dépend PAS de la CI cassée** — son T1 hermétique 2-nœuds tourne
  sur `rust-ci.yml` (3-OS, VERT, 0 dépendance GTK). C'est ce qui lève la seule objection
  (Axe4/Axe7 suggéraient « CI d'abord ») à placer boot-SEED avant la réparation CI, sans
  violer la contrainte dure OVERDUE 3/3. La CI (Phase C) précède le refacto et le push
  groupé parce que le volet « + CI chaque push » du gate testabilité doit être opérant
  pour que les phases suivantes soient vérifiables. *Alternative écartée : CI d'abord
  (rejeté — viole boot-SEED-first sans nécessité, T1 CI-indépendant).*
- **D2 — Design unifié boot-SEED (3/5 CONDITIONAL — RISQUE PRINCIPAL).** 2 livrables conçus
  ensemble sous l'invariant « broadcast gossip = HINT non fiable ; état durable synchronisé =
  VÉRITÉ ; tout cold-boot RECONCILIE une fois le neighborhood formé » : ANCRE
  (re-drive-on-ingest de `run_boot_seed_driver`, idempotent par set pinné, duress-gate hérité,
  borne 1/batch) + WORKER (réconciliation cold-boot par `start_sync(peers)`/keepalive forcé,
  scan `get_many_by_prefix` inchangé). 0 wire/dep/bump. Le driver est **ONE-SHOT confirmé**
  (`http.rs:1819-1826`) et l'évidence NEUVE worker (run-2 : booté 3s avant submit, 0 réception
  en 30s, +2m08 pour converger) prouve une classe cold-boot **BI-FACE**, pas un edge ancre
  isolé. Concevoir les 2 faces séparément ré-introduirait la même dette. *Alternative écartée :
  restart-only ratifié PO — ferme au canon mais laisse un défaut produit observé LIVE sur le
  chemin `b3_p2_quorum` PASS, à réserver au cas où le fix hermétique est infaisable.*
- **D3 — Split http.rs, 1 domaine = 1 phase (4/5 CONDITIONAL).** Extraire les 6 plus gros
  domaines inline (shard-session http, seed, frost, coordinator, curators, publish) un domaine
  = un commit atomique = **une phase** (N→S), réutilisant le pattern `*_api.rs` précédent,
  co-déplacer handler+DTO+tests, route inchangée dans `build_router`. Cible : région prod
  http.rs (~4545 prod / 12460 total) sous ~2500 l ; long tail (feed/search/preview/canary/kudos/
  apps) DÉFÉRÉE ; golden Phase M AVANT. Le big-bang XL = risque high de rebase (arc front parqué
  + axe sharding) ; l'incrémental borné capture ~70% du levier avec le count nextest comme gate.
  Test module ~7915 l (region 4546-12460) co-déplacé, jamais orphelin. *Alternatives écartées :
  split complet une passe (XL, high) ; ne pas toucher http.rs (levier-phare du thème).*
- **D4 — Gate machine d'invariance refacto (5/5 PASS).** Pour chaque phase refacto : `cargo
  fmt --all --check` (0) + `cargo clippy --workspace --all-targets --locked -D warnings`
  (0) + `cargo nextest run --workspace --locked` GREEN avec total >= baseline (Win 2095 /
  Docker 2099, 0 baisse) + web vitest 412 + operator 201 + 0 route path + 0 bump wire, et
  un golden de caractérisation pour les splits structurels. Un refacto (0 changement de
  comportement) a pour T1 la non-régression : le count constant EST la preuve. *Alternative
  écartée : non-régression seule sans golden — acceptable pour refactos triviaux densément
  couverts, rejeté pour http.rs (fort risque de drift).*
- **D5 — Benchmarks IN S82 (3/5 CONDITIONAL — OVERRIDE PO).** Le board **recommandait de
  différer** (ni docs-contrat ni refacto ; exige un rig chaud 5080-CUDA+M2-Metal, contraire à
  une hygiène déterministe ; ouvre un amendement canon T3 lourd que la memory `po_benchmarks`
  situe à la ratification, jamais mid-phase). **PO-2 tranche IN S82** : Phase B dédiée
  (llama-bench + perplexity-parity + TTFT/TPOT/ITL versionnés), clusterée rig-chaud avec
  l'exit-condition (c) de boot-SEED. Le score reflète la tension assumée : Phase B **rig-gated**
  (`BLOCK{rig}`, jamais RIG-ABSENT — le rig est engagé pour A) + amendement canon T3 ratifié EN
  S82. *Alternative board : différer (D5 synth), superseded par PO-2.*
- **D6 — Sharding-debt doc-contrat SEUL (4/5 PASS).** Folder UNIQUEMENT SCHEMAS-SHARD-REQ
  (fusionné avec l'item axe1 en Phase G, seul vrai item docs-contrat). DIFFÉRER fermement
  au slot sharding rig-chaud : R-J-6 (RunProof per-worker), F2 (KV-cache), SI-12 (TOCTOU),
  SHARD-TRUST-RECALIB (N3-reveal/SI-5/SI-7/SI-11), métriques-honnêteté cluster. Le cœur
  sharding S81 est LIVE-prouvé ; sa dette résiduelle est majoritairement feature/hardening
  rig-dépendante. *Alternatives écartées : + les 4 fixes robustesse bon-marché (acceptable
  si slack, hors-thème étiqueté) ; tout différer y compris SCHEMAS (rejeté — seul item
  docs-contrat).*
- **D7 — LOOPBACK §3 représentatif (4/5 PASS).** Déclarer §3 périmètre tier-target
  représentatif verrouillé dans le front-matter ; le trigger nouvel endpoint reste le
  garde-fou incrémental. `authed_routes` contient de nombreuses routes non listées : les
  toutes indexer est du volume mécanique à faible signal. *Alternatives écartées : rendre
  §3 exhaustif (scope blow-up) ; le laisser ambigu (fermabilité §6.12 exige une décision).*
- **D8 — Registre FRONTIER accept-and-close (4/5 PASS).** Figer le décompte exact des
  familles `DOMAIN_*_V1` sans `schema_for!` (grep déterministe committé, fin du flottement
  21/22/23). Acter formellement l'opt-in incrémental comme choix écrit, PAS un 4e
  re-confirm. Exiger `// FRONTIER:` (ou `FRONTIER-NO-SCHEMA` motivé) pour toute frontière
  NEUVE — dont les 3 request-bodies Phase G. Trancher aussi S80-G-1 doc-lint accept-and-close
  (3 reports, §6.2.1). *Alternative écartée : taguer les 22 familles (volume mécanique).*
- **D9 — Ledgers PATTERNS re-audit COMPLET (5/5 PASS).** Statuer chaque T*
  CLOSED/ZOMBIE/OPEN-avec-ancre-grep-résolvable (Phase E). Purge zombies Python (T44-T51 —
  `git ls-files packages/`=0 confirmé) + collision T15/T16. Statuer sans coder les tickets
  hors-thème et les router à leurs owners. Cœur du thème docs-contrat ; produit un ledger
  honnête débloquant la comptabilité de dette. *Alternative écartée : purge ciblée des
  zombies évidents (laisse le ledger partiellement stale).*
- **D10 — Hors-thème NON codé (5/5 PASS).** Ne PAS coder : T20-wire, T21, T23 Docker@sha256,
  T25 FIPS, T26 Argon2id, T27 rpassword, nginx-DRY, firewall, veilles supply-chain standing.
  Features/durcissements/veilles trigger-driven, pas du refacto ni de la dette docs-contrat.
  *Alternative : élargissement PO explicite (hickory PO-7 accordé, le reste rejeté par
  défaut).*
- **D11 — hickory bump IN S82 (4/5 CONDITIONAL).** Mettre à jour hickory-resolver
  (churn API réel du resolver 0.25 dans `dns_fallback.rs`), retirer les 4 ignores
  `deny.toml`, clore 4 RUSTSEC vivants (Phase K bornée). Un sprint refacto est le bon slot
  pour absorber le churn 0.25 ; le blocage S81 « iroh STRICTEMENT SEUL » est levé. Critère
  machine : `cargo deny check advisories` vert + nextest >= baseline. *Alternative écartée :
  garder ignore+carry re-daté (PO-7=B) — ne PAS laisser le carry se re-router muet une 4e
  fois.*
- **D12 — Migration stores worker Phase T + artefact (3/5 CONDITIONAL).** Rattacher la
  vérif live-ops redb2→4 sur 3 nœuds (S81-G-1) à Phase T avec artefact T2 committé
  `sprint82_t2_store_migration.json` (vérif au push), plus « en marge du push ». Résout le
  critic gap #2 (in_theme conflit axe4 false vs axe6 true, findings « obligatoire »). Le
  score reflète la dépendance **live-ops one-way** au push 3 nœuds (submitter+worker sur
  VPS+PC+Mac, sibling backup présent ou recréation fail-loud). *Alternative écartée :
  simple constatation en marge (laisse un carry audit-obligatoire sans propriétaire de
  phase — exactement le gap critic).*

## Lentille adversariale — constats et résolution

> Le completeness critic (`s82_critic.json`) émet **NEEDS-ADJUSTMENT** : les BONES du
> plan sont solides (« ce n'est pas FLAWED ») mais **4 ajustements durs** sont requis
> avant d'ouvrir Phase A. Ci-dessous les **7 coverage_gaps** (avec severity),
> boot_seed_soundness, refactor_sizing, frontier_completeness et missing_arbitrages —
> chacun avec sa **résolution** au spec verrouillé. Les réserves ne sont PAS masquées.

- **[important] Store migration S81-G-1 sans slot ni artefact.** Conflit `in_theme` non
  résolu (axe4 false vs axe6 true + findings « thème S82 obligatoire »), relégué à un
  scope-cut « en marge du push ». → **RÉSOLU (D12)** : rattaché **Phase T** avec artefact
  T2 committé `sprint82_t2_store_migration.json` (vérif 3 nœuds au push), plus un carry sans
  propriétaire.
- **[important] HICKORY sans phase provisionnée si PO-7=A.** `in_theme=true`, laissé à
  l'arbitrage, mais aucune phase, aucun critère de fermeture rattaché à une lettre. → **RÉSOLU
  (PO-7=A / D11)** : **Phase K** supply-chain bornée, critère `cargo deny check advisories`
  vert (4 ignores retirés, 4 RUSTSEC clos). Plus d'insertion « à chaud ».
- **[important] Granularité commit split http.rs (L=2, M=4 domaines).** Phase M = 4
  commits dans une phase = violation 1-commit-par-phase ; D3 l'évoquait « en parenthèse »
  sans trancher. → **RÉSOLU (D3)** : **1 domaine = 1 commit = 1 phase** (N→S, 6 phases). Le
  sprint passe à **20 phases** (README §4 ne plafonne pas) ; l'atomicité canon est
  respectée SANS déviation.
- **[important] Fiabilité de la reconcile « 8 P2/11 P3 S80 → réel 4 P2/10 P3 ».** Le plan
  ferme les items S80 sur la foi du décompte census, sans re-vérification indépendante du
  statut LIVE de chaque finding — risque d'orphelin silencieux (P1 au prochain audit). →
  **RÉSOLU (Phase E, D9)** : re-audit COMPLET (pas confiance au décompte) ; critère machine
  = tout T* OPEN pointe un fichier existant (grep résout), 0 collision d'ID ; le décompte
  stale est corrigé au réel dans le planning.
- **[minor] S82-TEST-E2E-ISOLATION-HYGIENE (préserver l'E2E web GREEN 44/2skip).** Garde
  standing réel (défaut S81-A3-1 fraîchement résolu) mais dans le `covers` d'aucune phase.
  → **RÉSOLU (test gate, garde standing)** : « tout nouveau spec semeur ⇒ projet
  chromium-authoring / cleanup (ne pas re-casser browse-search empty-state) » inscrit au
  gate de testabilité, s'applique à toute phase touchant `web/src`.
- **[minor] REFACTO-MAGIC-NUMBER-SWEEP `in_theme=true` sans statut explicite.** Traité en
  note générique, disparaît du tableau. → **RÉSOLU (scope-cut nommé)** : formalisé comme
  scope-cut explicite (« aucun résiduel concret S81 ; au mieux un gate grep léger, pas une
  phase »), plus de flottement.
- **[minor] Couplage Phase A ↔ refacto runtime.rs.** Phase A modifie
  `handle_directory_announcement` (retour `Option<NodeDirectoryEntry>`) + threade state
  dans la boucle gossip ; la phase de décomposition `DaemonRuntime::start()` (Phase **L**
  au spec) regroupe précisément `handle_announcement/directory/project`. Risque de refacto
  qui écrase le hook re-drive. → **RÉSOLU (note Phase L)** : le goal Phase L porte
  explicitement « **L absorbe le hook re-drive-on-ingest ajouté Phase A** — ne pas
  l'écraser » ; séquencement A-avant-L correct + coordination rendue explicite.

**boot_seed_soundness (réserve dure §6.2.1 + ambiguïté rig).** Le critic CONFIRME que
Phase A adresse RÉELLEMENT les 3 exit conditions machine-checkables — (a) test ancre
2-nœuds re-drive, (b) test worker 2-nœuds catch-up, (c) re-jeu live `BOOT_AFTER_SUBMIT`
artefact JSON PASS<30s — chacun avec contrôle red revert-proof (patron
`keepalive_rejoins_doc_after_neighbor_loss` / `dispatch_loop.rs:396`) ; le code-fix (a)+(b)
est **hermétique et platform-agnostique**, ce qui autorise légitimement boot-SEED-first
malgré la CI cassée. **Réserve DURE non masquée** : pour une escalade OVERDUE 3/3, §6.2.1
interdit un 4e report sec, or la fermeture PLEINE reste contingente à (c) LIVE, et le
census se **contredit** sur la dépendance rig ((c) « dépend du Mac M2 » vs harness
`b3_live_pc_vps.sh` = PC+VPS ⇒ probablement Mac-indépendant). → **RÉSOLU (PO-1=B, voie
stricte)** : (c) live PASS EST exigée pour la clôture ; l'engagement rig **Mac+PC+VPS** est
pris ; si le rig est indisponible ⇒ **escalade PO explicite**, JAMAIS un fallback
silencieux ni un 4e report sec. Le spec ferme ainsi l'ambiguïté par le haut (voie stricte),
et non par le « code-fix hermétique = clôture même si (c)=BLOCK » que le critic proposait
en repli.

**refactor_sizing.** Le critic juge le SCOPE refacto correctement borné et faisable sans
casser nextest (11 modules `*_api.rs` de précédent confirmés au disque — pattern PROUVÉ, pas
hypothétique ; gate d'invariance = vrai garde machine ; golden Phase M AVANT ; co-déplacement
test module prévu). **Deux problèmes de dimensionnement** : (1) granularité commit → **RÉSOLU
D3** (1 domaine/phase) ; (2) **DOUBLE CHARGE high-risk** — la refacto XL (rebase-conflit
attendu) COHABITE avec l'escalade boot-SEED high-risk dans le même sprint ; si boot-SEED
consomme le budget, la refacto est starvée. → **MITIGÉ (assumé, non nié)** : chaque phase =
commit indépendant DONE-par-construction ; le split est fermable **vague par vague** (N→S,
long tail différée) ; le rig est clusterisé A+B ; le golden Phase M borne le drift. La charge
reste la **réserve résiduelle assumée par le PO**.

**frontier_completeness.** Le critic la juge **COMPLÈTE pour les frontières neuves** : la
seule du corps S82 = les **3 corps de requête loopback shard-session** (`ShardGroupMintRequest`
/ `MountSessionRequest` / `ShardGenerateRequest`, `#[derive(Deserialize)]` seuls, réponses
déjà schématisées) — contractés Phase G, INDEXÉS Phase T (GUIDE + llms.txt + WIRING_SPEC +
SHARD_PROTOCOL_SPEC §6). La refacto est 0 route path + 0 bump wire ⇒ aucune frontière neuve
(garde-fou « si signature DTO lue par `web/src` touchée ⇒ index Phase T »). **Réserve
mineure** : si Phase G choisit l'option (b) `FRONTIER-NO-SCHEMA` plutôt que `JsonSchema`,
l'indexation Phase T reste requise → **le plan le couvre**. Les 3 gates docs exit 0 = critère
machine de fermabilité.

**missing_arbitrages (5) — tous absorbés.** (1) Granularité commit → **D3**. (2) Statut +
livrable d'acceptance store migration → **D12 (Phase T + artefact T2)**. (3) Provisionnement
phase hickory → **PO-7=A / D11 (Phase K)**. (4) Fermeture §6.2.1 quand (c)=BLOCK{rig} →
**PO-1=B (voie stricte, escalade PO, pas de fallback silencieux)**. (5)
ARB-CONSENT-L4-REDESCENTE (redescendre les consents L4 sur PC+Mac post-quorum) → **note
opérationnelle mineure Phase T** (choix opérationnel mineur, consigné, pas perdu).

## Arbitrages PO load-bearing (PO-1..PO-9 — tranchés à l'ouverture)

> S81 clos + audit gate joué → ces arbitrages **sont tranchés** (pas des recos). L'ordre
> suit le dossier (PO-1 = le plus structurant, fermeture de l'escalade OVERDUE 3/3).

1. **PO-1 — Fermeture escalade boot-SEED (OVERDUE 3/3).** *Options : A fix complet ancre+worker
   ; B restart-only ratifié.* → **Choix PO = B au sens strict** : fix complet ancre+worker + 2
   tests hermétiques BLOQUANTS **ET** clôture §6.2.1 conditionnée au re-jeu live (c) PASS<30s
   (engagement rig Mac+PC+VPS) ; rig indispo ⇒ escalade PO explicite, jamais un 4e report sec.
   *Impact : Phase A high-risk gate plein ; la clôture NE ferme pas sur (a)+(b) seuls.*
2. **PO-2 — Benchmarks standards (ex-Phase L).** *Options : in-S82 / différer / capture
   opportuniste.* → **Choix PO = IN S82** (CONTRE la reco board « différer ») : Phase B dédiée
   (llama-bench + perplexity-parity + TTFT/TPOT/ITL versionnés) + amendement canon T3 ratifié en
   S82. *Impact : +1 phase rig-dépendante `BLOCK{rig}` + amendement README §4 + G8 design dédié.*
3. **PO-3 — Périmètre sharding-debt.** *Options : doc-contrat seul / + 4 fixes robustesse / +
   hardening lourd.* → **Choix PO = doc-contrat SEUL** (SCHEMAS-SHARD-REQ, Phase G). Feature/hardening
   (R-J-6, F2, SI-12, N3-reveal) DIFFÉRÉ slot rig-chaud. *Impact : Phase G bornée, thème pur.*
4. **PO-4 — Condition push groupé.** *Options : A GHA vert exigé / B gate 3 verts best-effort GHA
   / C réparer CI puis pousser.* → **Choix PO = C** : réparer CI (Phase C) puis pousser sur 3 verts
   (Woodpecker ci-linux + rust-ci 3-OS + run `workflow_dispatch integration-nightly` lisible) en
   Phase T. *Impact : Phase C en amont, Phase T déclenche le push.*
5. **PO-5 — Topologie A-vs-B zéro-n0.** *Options : garder B / basculer A / hybride.* → **Choix PO =
   garder B** (déployée, éprouvée flip 3 nœuds identités conservées). Re-décision calendaire
   HORS-S82, due avant 25/08 ; croise gate n0 15/09 (EOL relais 30/09). *Impact : 0 travail S82.*
6. **PO-6 — Reprise arc front parqué.** *Options : dans S82 / après.* → **Choix PO = après S82**
   (memory `rapid_front_add_session`). *Impact : scope-cut S82, reste borné ; rebase conflit
   `provider_router.rs` évité.*
7. **PO-7 — Bump hickory-resolver 0.24→0.26.** *Options : A absorber / B ignore+carry re-daté.* →
   **Choix PO = A** (Phase K supply-chain bornée) : réécrire construction resolver + retirer 4
   ignores `deny.toml` + clore 4 RUSTSEC vivants. *Impact : +1 phase med-risk, critère cargo deny
   vert.*
8. **PO-8 — catalog_len=0 seeder (S81-G-3).** *Options : corriger design / accept-and-document.* →
   **Choix PO = accept-and-document** (consigné Phase I sécurité-docs ; report répété depuis S75 ⇒
   décision fermante). *Impact : 1 note docs Phase I, sort des carries.*
9. **PO-9 — Ratifier décalage workflow-engine/Viewer.** *Options : ratifier / ré-ouvrir.* → **Choix
   PO = ratifier** : S82=dette ; workflow-engine + Viewer = futurs slots ; supersede D6 actée ;
   staging `.planning/research/sprint82_workflow_engine/` marqué SUPERSEDED (Phase E) + audit_plan
   §6 corrigé. *Impact : Phase E ferme le staging-stale, thème préservé.*

## Réconciliation main-thread (vérification indépendante des faits load-bearing)

Avant de figer ce board, les faits porteurs ont été re-vérifiés au disque (2026-07-12) :
- **`http.rs` = 12460 l** (confirmé) ; région production ~4545 l, test module ~7915 l
  (region 4546-12460). CONFIRMÉ → co-déplacement du test module obligatoire aux splits N→S
  (jamais orphelin).
- **`runtime.rs` = 5096 l** ; `DaemonRuntime::start()` ~950 l monolithiques (l.276-1224).
  CONFIRMÉ → décomposition Phase L en sous-fonctions boot <~150 l faisable (extract-method
  behavior-preserving).
- **Driver boot-SEED ONE-SHOT** : `http.rs:1819-1826` intact, carry S76 non fermé. CONFIRMÉ →
  le re-drive-on-ingest (Phase A) est un vrai gap, pas un edge fantôme.
- **3 commentaires-promesse `task_response.rs`** présents verbatim : `:14` (« empty at S20 —
  S22+ sandbox activates »), `:84-85`, `:95` (« does not bump when S22 lands »). CONFIRMÉ →
  Phase F les réécrit au passé immuable + élargit `PROMISE_RE` (aveugle vérifié).
- **11 modules `*_api.rs`** de précédent existent (canary/contributor/diagnostic/health/invite/
  kudos/quarantine/shell/storage/tasks/worker_state). CONFIRMÉ → pattern d'extraction PROUVÉ,
  pas hypothétique (étaye le sizing refacto).
- **Points de vigilance kickoff à ACTER (pas des faits à corriger, des dettes à fermer)** :
  (a) `sprint82_audit_plan §6` présente ENCORE workflow-engine comme scope S82 alors que C9 le
  décale → corriger Phase E (LEDGER-STAGING-STALE, PO-9), + marquer le staging SUPERSEDED ;
  (b) `sprint82_audit_plan §3` **conflate** app-authoring in-vivo (STANDING OUVERT, carry P1
  distinct) et le quorum sharding (CLOSED par `b3_p2_quorum` C10) → clarifier, **NE PAS**
  déclarer app-authoring éteint ; (c) la fusion axe1+axe5 (SCHEMAS-SHARD-REQ) évite un doublon
  → UN seul item consolidé en Phase G.

## Statut

**APPROUVÉ pour kickoff**, sous ratification PO des 9 arbitrages — **faite à l'ouverture**.
Le lot de décisions est solide (**7 PASS forts + 5 CONDITIONAL aux conditions intégrées, 0
CONFLICT**) et aucune décision ne contredit une Day-0 gelée ni la pre-launch policy (0 bump
wire ; iroh `=1.0.1` ; 0 dep runtime hors hickory PO-7=A). Le completeness critic
(NEEDS-ADJUSTMENT) est **levé** : ses 4 ajustements durs sont résolus par D3 (granularité
1-domaine/phase, 20 phases), D12 (store-migration Phase T + artefact), PO-7=A (hickory Phase
K) et PO-1=B (fermeture §6.2.1 voie stricte, escalade PO si rig indispo) ; ses 3 gaps mineurs
(A↔L couplage, E2E isolation hygiene, magic-number sweep) sont nommés et traités. Le canon
« sprint dette nommé, jamais bundlé » est respecté (doc-dette éclatée E→J par domaine, pas
fourre-tout). Le sprint FERME ses items **par construction** (critères machine-checkables par
type de phase), pas par « amélioration ». **Réserve résiduelle assumée = la CHARGE** (20
phases, 4 axes ambitieux — boot-SEED high + benchmarks rig + hickory med + full split 6 phases
high ; deux+ workstreams high-risk) — **mitigée** par la fermabilité per-phase (chaque phase =
commit indépendant DONE-par-construction), le clustering rig A+B, et le golden refacto (Phase
M AVANT les splits). Les conditions BLOQUANTES reportées au plan de phases (T1 hermétique
2-nœuds BLOQUANT-vert boot-SEED ; (c) live PASS<30s ou escalade PO ; golden Phase M avant
N→S ; 3 gates docs exit 0 ; count nextest >= baseline ; push gaté sur 3 verts) sont **non
négociables**.
