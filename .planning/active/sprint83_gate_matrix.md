# Sprint 83 — Matrice de gates (cadrage Phase A)

**But** (AM-2, plan §Phase A l.219-222) : matérialiser `objectif → acceptance →
gate → phase propriétaire → commande → artefact` pour **les 47 gates §23** du
Research Pack Workflow Agents + les **6 acceptances produit WA-T1..T6** (§22.5)
+ les **10 acceptances Hub HUB-T1..T10** (§18). Chaque ligne a **une phase
propriétaire de fermeture** (0 orphelin), ses fermetures multi-phases, un
**mécanisme d'enforcement** (script / hook / test / artefact vs **review
humaine**) et un **statut** EXISTANT / ÉTENDU / NEUF.

Sources autoritaires : `research/workflow_agents_app_conception_ultradeep_2026-07-15.md`
§23 (gates) + §22.5 (WA-T) ; `research/workflow_hub_product_conception_2026-07-15.md`
§18 (HUB-T) + §19 (traçabilité) ; `sprint83_kickoff.md` §5 (outline phase→gate) +
§4.5 AM-2 (mécanisme/statut). Vocabulaire de statut d'acceptance : voir
`sprint83_t2_spec.md`.

Légende **Mécanisme** : `test` = suite automatisée (Rust/Vitest/Playwright) ;
`script` = gate shell/CI déterministe ; `hook` = hook Git de finalité ;
`artefact` = preuve committée (matrice, JSON, Proof Card) ; `humain` = revue /
panel rémunéré (jamais un verdict calculé par l'UI). Légende **Statut** :
`EXISTANT` = infra déjà en place ; `ÉTENDU` = infra existante élargie au
chantier ; `NEUF` = enforcement créé par S83.

> **Ouverture / fermeture de gate** : « ouvre » = le gate devient actif et
> bloquant à partir de cette phase ; « ferme » = la preuve d'acceptance est
> complète. `G-SCOPE-FLOOR` ouvre en A et ne ferme qu'en AJ1 (AM-5) ; sa
> pré-fermeture (WA-T1..T5 + pré-cutover T6) est en AI2.

---

## 1. Gates globaux §23 (47)

| # | Gate | Acceptance (preuve attendue, §23) | Phase propriétaire | Fermetures multi-phases | Mécanisme | Statut | Commande d'enforcement | Artefact |
|---|---|---|---|---|---|---|---|---|
| 1 | `G-SCOPE-FLOOR` | 11 piliers livrés ; WA-T1..T6 complets ; aucune coupe centrale masquée en carry | **A** (ouvre) → **AJ1** (ferme, AM-5) | AI2 (pré-fermeture WA-T1..T5 + pré-cutover T6) | artefact + humain | NEUF | revue kickoff §7 + `sprint83_t2_acceptance.json` | `sprint83_gate_matrix.md`, `sprint83_verification.md` |
| 2 | `G-PHASE-GRAMMAR` | labels A..AJ reconnus par preflight/review/Codex/auditor ; phase numérique refusée | **A** | — | test + hook | ÉTENDU (grammaire `[A-Z]+[0-9]?` existante, exercice bi-lettre neuf) | `python scripts/agent/test_agentctl.py` | `scripts/agent/test_agentctl.py` |
| 3 | `G-AUTHORITY` | replay déterministe ; tuer l'iframe ne tue pas le run | **E** (ouvre) | **J** (ferme avec E) | test | NEUF | `cargo nextest run -p sbfb-workflow-engine` | spike E + tests reducer J |
| 4 | `G-BRIDGE-GRANT` | méthode non déclarée / non accordée refusée malgré message iframe valide | **C** | L / V / AI (révocation en run réel) | test | NEUF | `npm run test:e2e -- bridge-grant-intersection` | `web/e2e/bridge-grant-intersection.spec.ts` |
| 5 | `G-EFFECT` | doubles commandes / résultats sans double transition ni effet | **K** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine effect` | tests idempotence K |
| 6 | `G-UNKNOWN` | effet non idempotent perdu devient `outcome_unknown` | **K** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine outcome_unknown` | tests K |
| 7 | `G-FS-PROCESS-EFFECT` | HEAD/index/worktree préconditions, racine FS et descendants processus vérifiés en K/Q/AA (§23) | **K** (ouvre) | **Q** (companion IPC) + **AA** (ferme, dogfood Factory) | test | NEUF | `cargo nextest run -p sbfb-workflow-engine fs_process` | tests K + AA (Windows inclus) |
| 8 | `G-CAPABILITY` | atténuation, expiration, audience, profondeur testées par propriétés | **L** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine capability` | tests propriété L |
| 9 | `G-SECRET` | canari absent des surfaces de sérialisation du moteur (step-request, receipt, run-event, Debug) + du blob au repos — **critère resserré** aux surfaces réellement prouvées : app KV / blobs / exports appartiennent aux phases qui les produisent | **L1** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine secret::` + `cargo nextest run -p nexus-core-rs vault::` | seam secret engine (bras ROUGE sur surface de prod + **6 encodages**, chacun avec son bras rouge) + pin de type du `Zeroizing` |
| 10 | `G-SECRET-VAULT` | credentials providers liés/rotés/supprimés dans un coffre hôte, jamais dans le run/app | **L1** | — | test | NEUF | `cargo nextest run -p nexus-core-rs vault::` | coffre L1 crate HÔTE (PLAN-ADAPT preflight : engine PUR/keyless ⇒ crypto vault côté `nexus-core-rs`). **20 tests `vault::`** — la commande couvre les propriétés qui ferment les ex-P1, pas un seul test : PIN prouvé sur coffre VIDE, entrée-vérificateur inatteignable des 4 opérations, rejeu de ct refusé, delete derrière le coffre détecté, handle périmé refusé en LECTURE **et** en ÉCRITURE, downgrade de params refusé, sel par coffre, champ inconnu refusé, KAT de dérivation du KEK, nonce malformé sans panic, init échoué non bloquant. handle périmé refusé aussi par `delete` et `rotate`, au lieu de répondre `Ok(false)`/`UnknownRef` depuis le snapshot (`delete_and_rotate_refuse_a_stale_handle_instead_of_answering_from_the_snapshot`) — refus qui vaut pour un handle dont l'epoch est EN RETARD, et **pas** après la collision lost-update de la **borne 27 du §19.4**, où le handle porte l'epoch du disque sur un BODY périmé : la garde passe et les quatre points d'entrée répondent depuis ce body (`bind` sert une entrée révoquée, `delete` rend `Ok(false)`, `rotate` rend `UnknownRef`, `put` COMMIT le body périmé) — résidu assumé, fermeture = verrou relecture→`rename`. Un filtre sur le seul `vault_bind_rotate_zeroize` (1/20) laissait toute régression de l'entrée-vérificateur ou de l'epoch VERTE. **Critère resserré** (même annotation que la row 9) : les surfaces « run/app » de l'intitulé ne sont PAS parcourues par ces tests — cf. borne 15 du §19.4, propriété des phases qui les produisent (I/K pour le run store, N/O pour l'appel provider et `SBFB_EFFECT_INPUT`, V pour les routes, X pour l'export, AI2 pour l'agrégat) |
| 11 | `G-HITL` | iframe incapable d'approuver ; mutation des paramètres invalide la décision | **L1** | — | test | NEUF | `npm run test:e2e -- workflow-hub-a11y` + `npm run test:unit -- authority` + `npm run test:unit -- Approvals` + `cargo nextest run -p sbfb-workflow-engine hitl` + `cargo nextest run -p sbfb-workflow-engine human_approval_requires_valid_decision` + `cargo nextest run -p nexus-core-rs hitl_approval` | binding keyless (mutation invalide ; exhaustivité par DEUX contrôles — compilateur pour un champ AJOUTÉ, compte pour un mutateur RETIRÉ — avec le résidu ASSUMÉ `champ: _,` qu'aucun des deux n'attrape) + signature hôte (small-order refusée nommément, préimage épinglée par un vecteur doré côté hôte) + affordance host-chrome. La moitié « iframe incapable d'approuver » est tenue par la **LISTE LITTÉRALE** de 16 verbes + `ALL_DISPATCH_METHODS.size === 16` de `authority.test.ts` — l'assertion nommée voisine est un **fil-piège de nommage**, pas la fermeture (un verbe hors radicaux la laisserait verte, la liste littérale non) — la spec e2e prouve « le rendu est hôte », pas « l'allowlist est fermée » ; et `human_approval_requires_valid_decision` est le test qui tue la mutation `contains(k)` → `!is_empty()`, invisible au filtre `hitl` |
| 12 | `G-STORE` | fault injection journal/outbox sans perte ni double émission | **E** (ouvre) / **I** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine store` | run store SQLite I |
| 13 | `G-UPGRADE-REPLAY` | ancien run rejoué après upgrade ; events/upcasters, runtime, adapters pinning prouvés | **I** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine upgrade_replay` | upcasters + snapshots I |
| 14 | `G-SINGLE-WRITER` | lease/fencing empêche deux runtimes d'émettre la même outbox | **I** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine single_writer` | fencing I |
| 15 | `G-NETWORK` | Task partagée public-only sans envelope chiffré ; aucun grant/secret ; ensemble réel des lecteurs prouvé | **P** | — | test | NEUF | `cargo nextest run -p nexus-shell-daemon network_projection` | wire tests P |
| 16 | `G-WIRE-COMPAT` | Task/Result/ArtifactRef vNext interopère ou refuse proprement les versions mixtes | **P** | — | test | ÉTENDU (re-scope pre-launch AM-12) | `cargo nextest run -p nexus-core-rs wire_compat` | tests additifs P |
| 17 | `G-PLACEMENT` | même contrat Ollama réel, cloud réel autorisé, SBFB ; `local_only` ne sort jamais | **N** (ouvre) / **P** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine placement` | doubles hermétiques N |
| 18 | `G-MCP-SSRF` | schémas/outils pinnés, DNS/IP/redirect revalidés, destinations privées bloquées, output borné | **P** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine mcp_ssrf` | adaptateur MCP P |
| 19 | `G-CLAUDE-CODEX-RECEIPT` | provider/modèle/CLI/version/digests/exit/output brut et indépendance prouvés | **O** | AB (dogfood) | test + artefact | NEUF | `cargo nextest run -p sbfb-workflow-engine receipt` | receipts O |
| 20 | `G-APPCALL` | child-run dédupliqué, schémas et cycles/depth refusés | **Y** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine appcall` | opérations inter-apps Y |
| 21 | `G-LIFECYCLE` | publish/install/activate/fork/update/re-consent/rollback/deprecate/revoke sans auto-update | **X** | — | test | NEUF | `npm run test:e2e -- lifecycle` | cycle X |
| 22 | `G-PUBLISH-RECOVERY` | publication partielle réparable et republish idempotent après fault injection | **X** | — | test | NEUF | `cargo nextest run -p nexus-shell-daemon publish_recovery` | tests X |
| 23 | `G-RRV-PROJECTION` | projection publique allowlistée, Proof Card, tombstone/reindex ; RRV ne lance ni ne modifie | **X** | — | test + artefact | NEUF | `cargo nextest run -p nexus-shell-daemon rrv_projection` | `WorkflowPublicProjection.v1` + Proof Card |
| 24 | `G-VERTICALS` | E2E complet Ideas, Conception, Factory, Babel ; pas de fixtures compilables seules | **Z** (ouvre, Factory 1er) | **Z..AC** (Factory) + **AD/AE/AF** (Ideas/Conception/Babel) + **AI2** (agrégée 4/4) | test | NEUF | `npm run test:e2e -- verticals` + `cargo nextest run verticals` | 4 goldens + dogfood |
| 25 | `G-PROVENANCE` | mutation de chaque digest rejetée, rôles non interchangeables | **M** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-core provenance` | 6 digests M |
| 26 | `G-ARTIFACT-ACL` | connaître le digest ne permet pas de lire un artefact privé ; audience/chiffrement/rétention testés | **M** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine artifact_acl` | ArtifactRef M |
| 27 | `G-EVAL` | routes, tools, budgets, sorties vérifiés sur goldens | **W** | — | test | NEUF | `cargo nextest run -p sbfb-workflow-engine eval` | datasets/graders W |
| 28 | `G-FACTORY-CONTRACT` | 9 rôles, 10 modes, G1-G10, FG0-FG10, hooks séparés ; enforcement réel marqué ; drifts de verdict résolus | **D** | — | artefact + hook | NEUF | `cargo nextest run -p sbfb-workflow-core factory_profile` (PROFILE_STALE) | `FactoryProcessProfile.v1` |
| 29 | `G-REVIEW-INDEPENDENCE` | driver, reviewer, Codex distincts ; sortie Codex brute ; aucun auto-PASS | **D** (contrat candidat) | **AB** (ferme runtime) + AI (consolidé) | test + process | NEUF | dogfood AB + hook lightcheck Check 7 | `operator_server`/receipts AB |
| 30 | `G-HOOK-FINALITY` | hooks refusent PROFILE_STALE/source mismatch, PASS-PENDING, Codex absent, mismatch phase/HEAD, body incomplet | **AC** | — | hook | ÉTENDU (2 hooks canon ex-D17 `lightcheck.sh`+`phase-auditor-gate.sh`) | `.claude/hooks/phase-precommit-lightcheck.sh` | hooks + tests hook |
| 31 | `G-FACTORY-CUTOVER` | shadow diff vert, source de vérité explicite, kill switch et rollback testés avant dépréciation | **AJ** | — | test | NEUF | `npm run test:e2e -- cutover` + oracle différentiel | shadow AJ |
| 32 | `G-HOST-ISOLATION` | companion séparé, IPC authentifié/capability-scoped, crash sans fuite prouvés | **Q** | — | test | NEUF | `cargo nextest run -p sbfb-factory host_isolation` | companion IPC Q |
| 33 | `G-WORKFLOW-CSP` | profil CSP Hub strict ; toute exception compensée ; XSS incapable d'un effet bridge | **Q** | — | script + test | ÉTENDU (source CSP UNIQUE `crates/nexus-core-rs/src/csp.rs:33`) | `bash scripts/check-frontier-contracts.sh` (check 3) | profil CSP Hub |
| 34 | `G-AUTHORING-AI-DIFF` | diff déclaratif source-only, provenance, inspection, acceptation ; aucune action autonome | **AG** | — | test | NEUF | `npm run test:e2e -- ai-diff` | assistance IA AG |
| 35 | `G-IMPORT-HOSTILE` | parser borné avant rendu ; code/secrets/outils arbitraires refusés ; brouillon non signé | **F** (ouvre) | **AG** (ferme, import Flowise) | test | NEUF | `cargo nextest run -p sbfb-workflow-core import_hostile` | corpus hostile F |
| 36 | `G-SUPPLY-CHAIN` | locks/licences/advisories/SBOM/build offline et corpus hostile vérifiés | **AH** | — | script | ÉTENDU (`deny.toml` existant) | `cargo deny check` + SBOM | `deny.toml` + SBOM AH |
| 37 | `G-A11Y-WCAG` | zéro échec connu A/AA sur toute surface/état/processus ; matrice AAA archivée | **B** (harness) + phases UI | **AI1** (ferme) | test + artefact | NEUF | `npm run test:e2e -- a11y` (axe) | matrice WCAG |
| 38 | `G-A11Y-EN301549` | version applicable figée ; matrice clause par clause sans écart P0/P1 non résolu | **B** (réserve) | **AI1** (ferme) | artefact + humain | NEUF | revue matrice EN 301 549 | matrice EN 301 549 |
| 39 | `G-A11Y-AUTHORING` | canvas, outline, mode guide partagent commandes/digest ; templates/réparations ATAG | **T** | AI1 (consolidation, G-A11Y-* ferment) | test | NEUF | `npm run test:e2e -- authoring-a11y` | parité auteur T |
| 40 | `G-A11Y-PARITY` | 1 000 mutations générées à digest identique ; chaque action disponible sans drag | **T** (ouvre) | **AI1** (ferme) | test | NEUF | `npm run test:e2e -- a11y-parity` | générateur de mutations T |
| 41 | `G-A11Y-AT` | matrice Windows/macOS/iOS/Android/Linux, lecteurs d'écran, braille, voice, switch archivée | **B** (réserve matrice) | **AI1** (ferme, multi-AT) | artefact + humain | NEUF | revue matrice AT (5 OS) | matrice multi-AT |
| 42 | `G-A11Y-COGA` | compréhension, mémoire, attention, erreurs, reprise testées avec utilisateurs concernés | **B** (harness) + surfaces | AI1 | test + humain | NEUF | harness COGA + panel | matrice COGA |
| 43 | `G-A11Y-PREFS` | axes fonctionnels empilables preboot ; aucun diagnostic/need dans DOM, app, trace, réseau | **B** (ouvre) | AI1 (consolidation, G-A11Y-* ferment) | test | NEUF | `npm run test:e2e -- prefs-preboot` | préférences privées B |
| 44 | `G-A11Y-HOST` | grants, secrets, HITL, publication, erreurs du chrome hôte au même niveau | **L1** / **Q** (chrome hôte) | AI1 | test | NEUF | `npm run test:e2e -- workflow-hub-a11y` + `npm run test:unit -- Approvals` | chrome hôte L1/Q (la jambe L1 est la spec `workflow-hub-a11y` : aucun fichier `host-a11y` n'existe, le filtre ne matchait rien). Le scan axe de `/approvals` est **scopé** à `[data-testid="hitl-host-chrome"]` : page-wide, `color-contrast` remonte 123 violations appartenant toutes au chrome PARTAGÉ (déclencheur de palette à 3,82:1), dette pré-existante de toute route et propriété d'AI1 ; le scope est mutation-testé (contraste cassé dans la carte ⇒ ROUGE). `Approvals.test.tsx` est la seule jambe qui falsifie *host-owned* (démontage/remontage) et le repli d'expiration |
| 45 | `G-A11Y-USERS` | deux rounds / panel rémunéré ; zéro barrière P0/P1 sur parcours critiques | **W1** (round 1, ouvre) | **AI1** (round 2, ferme) | artefact + humain | NEUF | panel rémunéré (checkpoints AM-10) | rapports panel round 1/2 |
| 46 | `G-I18N` | nouveaux messages extraits et parité locale gatée | **B** (ouvre) + phases UI | — | script | ÉTENDU (3 scripts Lingui existants) | `npm run lint` + gates Lingui (`web/scripts/scan-en-strings.sh`) | catalogues locale |
| 47 | `G-PERF` | budgets chiffrés graphe/timeline/Babel et size-limit verrouillés | **AI** | — | script | ÉTENDU (`size-limit` existant) | `npm run size` + budgets chiffrés | budgets AI |

**Comptes** : 47 gates §23, 0 orphelin. Statut : **40 NEUF + 7 ÉTENDU**
(G-PHASE-GRAMMAR, G-WIRE-COMPAT, G-HOOK-FINALITY, G-WORKFLOW-CSP,
G-SUPPLY-CHAIN, G-I18N, G-PERF) ; 0 EXISTANT pur (toutes touchent au chantier).

> **Réconciliation AM-4 (Phase L, 2026-07-24, §4.4 statut re-vérifié)** : la
> matrice a été figée au cadrage Phase A, AVANT la re-coupe AM-4 qui scinde
> L (permissions/atténuation) et **L1** (coffre `secret_ref` + HITL signé).
> Les rows 9 `G-SECRET`, 10 `G-SECRET-VAULT`, 11 `G-HITL` et 44 `G-A11Y-HOST`
> ont donc migré leur phase propriétaire **`L → L1`** (plan §Phase L1 l.1009-1015).
> Phase L ne ferme que **row 8 `G-CAPABILITY`**. 0 orphelin conservé.

> **Réconciliation Phase L1 (2026-07-25)** : quatre rows ont été éditées PAR L1,
> et la nature de chaque édition est consignée ici pour qu'un audit puisse
> l'attribuer sans relire les rounds de review.
> - **Row 9 `G-SECRET`** — critère RESSERRÉ et annoté : le canari couvre les 5
>   rendus du moteur + le blob au repos, pas les surfaces `app KV` / blobs /
>   exports, qui appartiennent aux phases qui les PRODUISENT (I/K run store,
>   N/O appel provider et `SBFB_EFFECT_INPUT`, V routes, X export, AI2 agrégat).
> - **Row 10 `G-SECRET-VAULT`** — commande re-pointée sur la famille `vault::`
>   entière (**20 tests**, pas le seul `vault_bind_rotate_zeroize`), énumération
>   des propriétés complétée par le refus du handle périmé dans `delete`/`rotate`,
>   et même annotation « critère resserré » que la row 9 (la moitié « run/app »
>   de l'intitulé n'a jamais eu de preuve — borne 15 du §19.4).
> - **Row 11 `G-HITL`** — commande étendue à `npm run test:unit -- Approvals`
>   (seul falsificateur du « host-owned », critère BINAIRE de L1.4) et
>   requalification honnête du pouvoir des deux contrôles d'exhaustivité.
> - **Row 44 `G-A11Y-HOST`** — commande étendue de même ; portée du scan axe
>   divulguée (scope `include` sur la région HITL, 123 `color-contrast`
>   page-wide qui sont une dette PARTAGÉE de toute route, propriété d'AI1).
>
> Contributrices d'acceptance : **WA-T1** gagne **L1** (items WA-T1.8 /
> WA-T1.11 — approbation chrome hôte accessible, canari `secret_ref`), et L1
> est l'une des « phases UI » de **WA-T4**.

### Gates process repo (hors 47, EXISTANT — rappel de complétude)
`G1` Design Review Board (`sprint83_design_review.md`, hook Check 5) et `G9`
factual research sont des gates de kickoff/phase déjà en place (EXISTANT). Ils
encadrent Phase A mais ne comptent pas parmi les 47 gates du chantier.

---

## 2. Acceptances produit WA-T1..T6 (§22.5)

Ces acceptances flagship sont jouées agrégées en **AI2** (WA-T1..T5 + pré-cutover
T6) puis **finalisées AJ/AJ1** (palier T6, règle bi-temporelle AM-1). Statut de
palier : `ACTED{evidence}` / `MIXED` / `NOT-RUN` (cf. `sprint83_t2_spec.md`).

| Acceptance | Résumé (§22.5) | Phase propriétaire (jeu) | Phases contributrices | Mécanisme | Artefact |
|---|---|---|---|---|---|
| **WA-T1** Auteur + runtime durable local | auteur sans JSON, parité canvas/outline (digest), Ollama/hermétique, tool idempotent, `waiting_human`, kill/restart runtime, reprise frontier, 0 double effet, compare versions, 0 canari | **AI2** | R/S/T/U (auteur) + E/I/J/K (runtime) | test (Playwright + engine) | `sprint83_t2_acceptance.json` (palier WA-T1) |
| **WA-T2** Protocole multi-nœuds | signer/publier pack A, search/verify/install-désactivé B, diff capabilities + activer grants minimaux, Ollama+cloud+Task SBFB (`local_only` ne sort pas), provenance source→output, publish sans auto-update, blocage si droits ↑, fork/deprecate/revoke, rollback | **AI2** | F/N/O/P/X | test + `RIG-ABSENT` (pas de 2e worker/cloud en CI) | `sprint83_t2_acceptance.json` (WA-T2) |
| **WA-T3** Chaîne métier cross-app | Ideas→Research/Decision, Conception→ConceptionPack, Factory sprint complet, chaîne headless+child-runs dédup, Babel ≥100 chunks 2 workers crash/reassembly, packs forkables | **AI2** | Z..AF (verticals) | test + `RIG-ABSENT` (Babel 2 workers) | `sprint83_t2_acceptance.json` (WA-T3) |
| **WA-T4** Accessibilité maximale E2E | sans souris/drag, lecteur d'écran, single-pointer, voix, 400%/forced-colors/RTL/reduced-motion, prefs empilées preboot, approbation hôte, `needs[]` ne quitte pas le nœud, parité canvas/outline/guide, panel 2 rounds, EN 301 549 | **AI2** | B + phases UI + W1 + AI1 | test + humain (panel) | `sprint83_t2_acceptance.json` (WA-T4) + matrices a11y |
| **WA-T5** Attaque et exploitation | pack malveillant non exécuté, refus usurpation/grant expiré/capability non déclarée/privé→réseau, crash journal/outbox 0 double effet, timeout/dup/tardif/`outcome_unknown`, quotas/rétention, mutation digest rejetée, budgets, export sans secret/a11y-data, opérable en erreur | **AI2** | AH (+ E/K/L/M) | test | `sprint83_t2_acceptance.json` (WA-T5) |
| **WA-T6** Parité, migration, bascule Factory | extraire `FactoryProcessProfile.v1`, baseline legacy vs nouveau 2 worktrees, ≥1 phase + sprint complet shadow, indépendance driver/reviewer/Codex + Codex brut, `commit.intent`/`push.intent` humains, hooks bloquent, kill switch/rollback, dépréciation post-sprint-vert | **AJ** (jeu) / **AJ1** (finalise) | D + Z..AC + AJ | test + hook + humain | `sprint83_t2_acceptance.json` (WA-T6, MIXED→final AM-1) |

---

## 3. Acceptances Hub HUB-T1..T10 (§18 / §19)

Traçabilité vers le plan global (§19) — chaque HUB-T a une phase propriétaire OU
une subsomption documentée par un gate/WA-T (AM-2).

| Acceptance | Résumé (§18) | Phases propriétaires (§19) | Gates associés | Mécanisme | Artefact |
|---|---|---|---|---|---|
| **HUB-T1** Auteur sans JSON | canvas/outline/mode-guide → même digest sémantique ; undo/autosave/recovery | **S**, T, U | `G-SCOPE-FLOOR`, `G-A11Y-AUTHORING` | test (Playwright) | `web/e2e/workflow-hub-author.spec.ts` |
| **HUB-T2** Confiance avant run | agents/providers/outils/coûts/grants/HITL visibles ; mutation invalide la confirmation | **U** (C, L, L1) | `G-BRIDGE-GRANT`, `G-CAPABILITY`, `G-HITL` | test | inspecteurs U + spec preflight |
| **HUB-T3** Run durable | kill/restart runtime, reprise frontier, `outcome_unknown` sans double effet | **V** (E, I, J, K) | `G-STORE`, `G-UPGRADE-REPLAY`, `G-EFFECT` | test | `web/e2e/workflow-hub-run.spec.ts` |
| **HUB-T4** Catalogue et cycle complet | découvrir/installer-désactivé/activer/forker/update-reconsent/rollback/deprecate/revoke | **X** | `G-PUBLISH-RECOVERY`, `G-RRV-PROJECTION`, `G-LIFECYCLE` | test | `web/e2e/workflow-hub-lifecycle.spec.ts` |
| **HUB-T5** Cross-app | packs externes exécutables ouverts/forkés/réévalués/invoqués ; Ideas→Conception→Factory + Babel ; logique ne migre pas dans le Hub | **Y** (D, Z..AF, AJ) | `G-APPCALL`, `G-FACTORY-CONTRACT`, `G-FACTORY-CUTOVER` | test | verticals Z..AF |
| **HUB-T6** Évaluations et provenance | comparer 2 versions, régression, export preuve rédigée, refus mutation digest/receipt | **W** (M) | `G-PROVENANCE`, `G-EVAL` | test | eval W + provenance M |
| **HUB-T7** Accessibilité complète | tout le parcours app+shell WCAG 2.2 AA + matrices AAA/ATAG/COGA/EN 301 549 ; prefs preboot ; multi-OS/AT ; 2 rounds | **B** + chaque phase UI + **AI1** (consolidation) | `G-A11Y-*`, `HUB-A11Y-*` | test + humain | matrices a11y + panel (subsomé par G-A11Y-*) |
| **HUB-T8** Résilience produit | offline/bridge-refusé/provider-absent/run-partiel/storage-plein/pack-invalide : orientation préservée, 0 exécution inventée ; recherche/nouveau/notifs/profil sans vol de focus | **R**, **V**, **AI** | `G-AUTHORITY`, `G-PERF`, `HUB-BRIDGE` | test | `HUB-DRAFT`→R + états vides + fault AI |
| **HUB-T9** Import data-only | fixture Flowise valide → brouillon + rapport ; fichier hostile refusé sans exec/secret/grant/effet | **AG**, **AH** | `HUB-IMPORT`, `G-SUPPLY-CHAIN` | test | import AG + corpus hostile AH |
| **HUB-T10** Vérité d'intégration | parcours finaux via Workflow Runtime + shell hôte + fixtures multi-nœuds réels ; mocks CI insuffisants pour fermer le pilote | **C..AJ** (pré-cutover AI, fermeture AJ) | `G-SCOPE-FLOOR`, `HUB-INTEGRATION` | test + humain | `sprint83_t2_acceptance.json` (agrégat) |

**HUB-DRAFT** (autosave/undo/redo/recovery/conflits) est joué en **Phase R**
avec le scaffold app-authoring (kickoff §6.1). Les gates Hub `HUB-*` de §19
(HUB-SCOPE, HUB-MODEL-PARITY, HUB-SHELL, HUB-BRIDGE, HUB-TRUST, HUB-CAPACITIES,
HUB-RUN, HUB-EVAL, HUB-LIFECYCLE, HUB-APPCALL, HUB-IMPORT, HUB-AI-DIFF,
HUB-MOBILE-E2E, HUB-INTEGRATION, HUB-A11Y-*) sont les projections **produit** des
gates globaux ci-dessus sur les surfaces Hub R..Y ; ils se ferment avec leur
HUB-T propriétaire et n'ajoutent pas de fermeture orpheline.

---

## 4. Invariants de la matrice (à tenir au fil du sprint)

1. **0 orphelin** : chaque gate §23 (47) + WA-T (6) + HUB-T (10) a une phase
   propriétaire ci-dessus. Un gate neuf apparu en cours de sprint s'ajoute ici.
2. **Ouverture ≠ fermeture** : un gate « ouvre » tôt (C/E/F) mais ne « ferme »
   qu'avec sa preuve complète ; `G-SCOPE-FLOOR` ferme en **AJ1** (AM-5).
3. **Mécanisme honnête** : un gate `humain`/`artefact` (panel, matrice AT) n'est
   jamais un verdict calculé par l'UI ; l'autorité reste la preuve committée +
   l'audit gate S84 (Track J testabilité + Track K docs-contrat).
4. **Statut re-vérifié** : `ÉTENDU`/`EXISTANT` cité contre le code au HEAD ; ne
   pas requalifier un `NEUF` en `ÉTENDU` sans ancre fichier:ligne.
