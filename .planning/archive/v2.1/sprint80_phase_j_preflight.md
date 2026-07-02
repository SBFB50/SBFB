# Sprint 80 — Phase J — Preflight (deep, 4 scans + vérification adversariale par finding, finalisé G8)

**Date** : 2026-07-02
**HEAD** : `a6b4ca4` (master, arbre propre)
**Sprint / Phase** : 80 / J — Wrap-up + **CLÔTURE DOCS-CONTRAT** (dernière phase, ferme le sprint). Phase docs-pure : `sprint80_verification.md` (9 sections canon), `sprint81_audit_plan.md` (11 tracks canon), indexation des 4 frontières S80 dans `docs/factory/`, SPRINT_LOG row 80 + CLAUDE.md/`nexus_grid_pivot.md`/MEMORY.md. **0 dep, 0 code** (aucun `.rs`/`.ts`/manifeste touché).
**Verdict** : **PLAN-ADAPT**

> Le plan littéral de la Phase J (`sprint80_plan.md:253-262`) reste exécutable et **compatible Day-0** (aucun invariant contredit → pas de DESIGN-CONFLICT), mais il doit être corrigé sur **quatre points étayés code** : (1) l'**ajout de la clôture docs-contrat au scope** — actée par l'amendement canon `a6b4ca4` (README §3.3 livrable 3 + DoD (d) §4 + §6.12 porteurs) ; Phase J est la **PREMIÈRE application réelle** du canon amendé ; (2) le run **Docker dual-platform `sbfb-phase-j` a réellement ÉCHOUÉ** (exit 100, 2 timeouts) — la prémisse « la phase consigne les résultats, ne relance pas » est factuellement fausse ; verification.md ne peut PAS écrire « Docker 2018 GREEN » ; (3) `sprint80_verification.md` doit suivre les **9 sections canon README §2.3**, PAS le gabarit **6-sections** de `sprint79_verification.md` (piège de recopie) ; (4) `sprint81_audit_plan.md` doit refléter **11 tracks** (ajout Track J testabilité + Track K docs-contract, `a6b4ca4`), pas 9. Verdict **PLAN-ADAPT** (aucune Day-0 réfutée, aucun DESIGN-CONFLICT).

> Finalisation : 4 scans reçus (S1a prior-art llms.txt/Diátaxis + S1b deps/état + S2 formats canon + drift + carries + S3 threat/honnêteté + S4 contrats wire des 4 frontières), chaque finding passé au crible adversarial. **Aucun finding pleinement RÉFUTÉ** ; 2 ADJUSTED (S1a-03 P2→P3, S1a-05 recalibrage remède à sévérité inchangée P1). Faits load-bearing re-vérifiés en main-thread : **15/15 source-refs des 4 frontières grep-résolvent** (auth.rs / operator_server.rs / sprint_history.rs / gates.rs / llm_bridge.rs, cf. §4) ; `check-factory-docs.sh` lu intégralement (source-ref-check sur AGENT_DOCS = WIRING_SPEC + llms.txt seulement, honesty-gate, REQUIRED_ANCHORS) ; REFERENCE.md porte déjà `PROVISIONAL` (:7) + `Not evidenced` (:10) et est english-body-exempt (:8) ; llms.txt porte déjà les 3 marqueurs d'honnêteté (:8,:14) ; HEAD `a6b4ca4`, arbre propre, cibles J absentes (verification.md / sprint81_audit_plan.md à créer).

---

## 1. Synthèse des scans

| Scan | Objet | Verdict | Apport load-bearing (evidence) |
|---|---|---|---|
| **S1a** Prior-art llms.txt + Diátaxis (4 frontières Operator) | Format de clôture | **PLAN-ADAPT** | llms.txt EST llms.txt-spec-conforme (H1 + blockquote + H2 file-lists) → **nouvelle H2**, pas un restructure. Les 4 frontières sont l'**API control-plane Operator**, categoriquement distincte de l'app-authoring de tout le quad Diátaxis → home dédiée (REFERENCE.md + index llms.txt), **PAS** WIRING_SPEC (dont l'audience + REQUIRED_ANCHORS sont 100% CSP/authoring). Piège d'honnêteté : `requires_gate` = **frame forgé main**, PAS une variante serde (5 serde + 1 forgé = 6 types wire). Consommateurs TS sous `tools/` = ignorés par le source-ref-check → citer en **liens markdown** (link-checkés), pas en backtick. Prior-art SSE : OpenAI data-only + sentinelle ; Ollama newline-JSON done:true ; Syncthing loopback X-API-Key + events. |
| **S1b** Deps / état | Trajectoire docs-pure + compteurs + run Docker | **PLAN-ADAPT** | Docs-pure CONFIRMÉE (0 dep / 0 code ; `a6b4ca4` = 7 fichiers agents/skills/prompts/README, 0 manifeste). Compteurs à consigner tous EXACTS (nextest Win **2014** / Docker Linux **2018** [+4 `#[cfg(unix)]`], Vitest operator **201**/35 fichiers, e2e Playwright **10**, Vitest web **411**, T2 **PASS**). **MAIS** run Docker détaché `sbfb-phase-j` **terminé RED** (exit 100 : 2018 tests, 2016 passed, **2 failed** = `operator_context_pack_schema_complete` + `operator_git_diff_endpoint_returns_envelope`, tous deux `reqwest TimedOut` 30s sur endpoints frontières S80). Root cause = contention hôte (7 conteneurs langfuse/open-webui up 6h) + build cold, échappatoire `SBFB_TEST_HTTP_TIMEOUT_SECS` non utilisée. |
| **S2** Formats canon + drift + carries | Conformité verification/audit_plan + inventaire | **PLAN-ADAPT** | `sprint79_verification.md` = **6 sections** là où README §2.3 en impose **9** → PIÈGE de recopie (écrire les 9). Clôture docs-contrat = 1re application §6.12 amendé : indexer les 4 frontières SANS casser les 9 REQUIRED_ANCHORS. `a6b4ca4` a porté le compte de tracks d'audit **9→11** (Track J testabilité + Track K docs-contract). Risque **zombie-carry** : `TEST-ISOLATION-SBFB-HOME` FERMÉ Phase I mais listé « ouvert » dans `sprint80_audit_plan.md:81` → filtrer avant de router. Inventaire NOMMÉ des carries réellement ouverts fourni (§7). |
| **S3** Threat / honnêteté | Fuite / anti-promesse / verdicts fermés | **PLAN-ADAPT** | Docker RED = verification.md ne peut PAS écrire « GREEN » (miroir S1b). Anti-promesse dans les nouveaux docs **PAS gatée** (check-frontier-contracts.sh exclut `docs/` par construction) → discipline rédaction pure (passé/présent, jamais « S81 will »). Auth cookie : placeholder `<token>`/`<hex>` obligatoire (jamais un 64-hex concret) + cookie = **secret de session per-boot ≠ bearer**. Routes /api/* = carte de surface honnête (loopback+auth) mais **LIER** THREAT_MODEL §14, ne pas dupliquer. SSE debug : documenter la SHAPE, ne PAS coller un dump de prompt. |
| **S4** Contrats wire des 4 frontières | Producteur→consommateur byte-exact | **EXECUTE** | Les 4 contrats tracés au HEAD, **4 résumés FR prêts-à-coller** livrés (§6). Deux gardes : (a) le P3 doc-drift `streamChunk.ts:1063` est **déjà résolu** au HEAD (ancre par nom `sse_gate`) → ancrer par SYMBOLE, pas par ligne ; (b) **WIRING_SPEC = NON concerné** — les 4 frontières sont l'API du control-center (hors `BLOB_SERVE_CSP`, `operator_server.rs:342-349`), sous-domaine DISTINCT du sealed-iframe → REFERENCE.md + llms.txt. |

**Recommandations par scan** : S1a PLAN-ADAPT, S1b PLAN-ADAPT, S2 PLAN-ADAPT, S3 PLAN-ADAPT, S4 EXECUTE. **Global = PLAN-ADAPT** (4 scans PLAN-ADAPT avec P1 CONFIRMÉS ; S4 EXECUTE subsumé ; aucune Day-0 réfutée, aucun DESIGN-CONFLICT).

---

## 2. Verdicts adversariaux intégrés (par finding)

Chaque finding a été vérifié contre la source. **Aucun RÉFUTÉ.** Les ADJUSTED gardent leur evidence (réelle) ; ils n'influencent le verdict qu'à leur sévérité corrigée.

| Finding | Titre court | Sévérité déclarée → **retenue** | Verdict adversarial |
|---|---|---|---|
| **S1a-01** | llms.txt spec-conforme → NOUVELLE H2, pas restructure | info → **info** | contexte (format retenu §5) |
| **S1a-02** | Scope : 4 frontières = API Operator ≠ app-authoring → doc dédiée, pas WIRING_SPEC | P2 → **P2** | **CONFIRMED** (concordant S4-DOMAIN-SCOPE) |
| **S1a-03** | Nouveau doc OPERATOR_API.md non-gaté sauf câblage script | P2 → **P3** (ADJUSTED) | evidence réelle ; **hors scope J** — folder dans REFERENCE.md (déjà gaté) au lieu de créer un doc non-gaté ; câblage Track K = contingent, pas mandatoire J |
| **S1a-04** | 15 source-refs exacts grep-résolvables des 4 frontières | P1 → **P1** | **CONFIRMED** (re-vérifié main-thread 15/15 OK, §4) |
| **S1a-05** | `requires_gate` = frame forgé ≠ variante serde (5+1=6 wire) | P1 → **P1** | **ADJUSTED** (thèse CONFIRMED ; remède recalibré : forge SSE = `sse_gate` `operator_server.rs:1591-1592`, PAS `handle_chat_message` qui pose un champ booléen d'une réponse POST non-SSE) |
| **S1a-06** | Consommateurs TS sous `tools/` ignorés par source-ref-check → liens markdown | P2 → **P2** | **CONFIRMED** |
| **S1a-07** | Template prior-art SSE + control-plane loopback | info → **info** | contexte (§6.4) |
| **S1a-08** | Langue : REFERENCE.md english-body exempt vs « français Diátaxis » | P3 → **P3** | **CONFIRMED** (décider explicitement, §5.4) |
| **S1b-01** | Run Docker `sbfb-phase-j` RED (exit 100, 2 timeouts) — DoD dual-platform NON satisfaite | P1 → **P1** | **CONFIRMED** (Windows-natif VERT sur ces 2 tests → env-block, pas régression) |
| **S1b-02** | Compteurs à consigner tous EXACTS + invariant count Linux 2018 | info → **info** | contexte (§8) |
| **S1b-03** | Trajectoire docs-pure confirmée (0 dep / 0 code) | info → **info** | contexte |
| **S1b-04** | 2 échecs = timeouts, échappatoire `SBFB_TEST_HTTP_TIMEOUT_SECS` non utilisée | P2 → **P2** | **CONFIRMED** |
| **S1b-05** | Gates docs-contrat + arbo docs/factory présents/exécutables | info → **info** | contexte |
| **S2-1** | `sprint79_verification.md` a 6 sections, README §2.3 en impose 9 | P1 → **P1** | **CONFIRMED** |
| **S2-2** | 4 frontières → docs gatés SANS casser REQUIRED_ANCHORS | P1 → **P1** | **CONFIRMED** (résolu §5 : llms.txt + REFERENCE.md ; WIRING_SPEC NON concerné par S4) |
| **S2-3** | Anti-promesse vacuous pour `docs/` (jamais scanné) | P2 → **P2** | **CONFIRMED** |
| **S2-4** | audit_plan : 11 tracks (Track J + K), README §2.4 dit « Tracks A..I » | P2 → **P2** | **CONFIRMED** |
| **S2-5** | Zombie-carry : `TEST-ISOLATION-SBFB-HOME` FERMÉ Phase I mais listé ouvert | P2 → **P2** | **CONFIRMED** |
| **S2-6** | Inventaire NOMMÉ des carries réellement ouverts | info → **info** | contexte (§7) |
| **S2-7** | SPRINT_LOG row 80 (insertion haut de table) + CLAUDE.md §Etat S80 DONE | info → **info** | contexte (§9) |
| **S2-8** | Format S79 docs-contract closure = gabarit direct llms.txt/REFERENCE | info → **info** | contexte (§5) |
| **S3-DOCKER-EXIT100** | Docker RED — verification.md ne peut pas consigner GREEN | P1 → **P1** | **CONFIRMED** (Windows-natif vert cross-check → classe d'env-block) |
| **S3-ANTIPROMISE-DOCS-BLINDSPOT** | Anti-promesse docs/ non gatée mécaniquement | P1 → **P1** | **CONFIRMED** |
| **S3-LEAK-TOKEN-COOKIE** | Placeholder token + cookie=secret-session≠bearer | P2 → **P2** | **CONFIRMED** |
| **S3-SURFACE-REFER-THREATMODEL** | Routes /api/* = carte honnête mais LIER §14 pas dupliquer | P2 → **P2** | **CONFIRMED** |
| **S3-VERDICT-VOCAB-DOD** | Verdicts fermés seuls + verdict docs-contrat DoD (d) | P2 → **P2** | **CONFIRMED** |
| **S3-SCOPE-FACTORY-DOCS-GATE-GREEN** | Garder check-factory-docs vert + ne pas conflater périmètres | P2 → **P2** | **CONFIRMED** |
| **S3-SSE-DEBUG-PROMPT-DUMP** | Ne pas coller de dump de prompt en exemple SSE | P3 → **P3** | **CONFIRMED** |
| **S4-AUTH-CONTRACT** | Contrat auth cookie complet vérifié | info → **info** | prêt-à-rédiger (§6.1) |
| **S4-DIFF-CONTRACT** | Contrat GET /api/git/diff vérifié | info → **info** | prêt-à-rédiger (§6.2) |
| **S4-GATES-CONTRACT** | Contrat GET /api/gates vérifié | info → **info** | prêt-à-rédiger (§6.3) |
| **S4-SSE-CONTRACT** | Contrat SSE chat re-vérifié au HEAD | info → **info** | prêt-à-rédiger (§6.4) |
| **S4-DOMAIN-SCOPE** | 4 frontières = API control-center, WIRING_SPEC NON concerné | P2 → **P2** | **CONFIRMED** |
| **S4-ANTI-PROMISE** | Provenance au passé + ancrer par symbole | P3 → **P3** | **CONFIRMED** |

**Note S1a-05 (remède recalibré, load-bearing)** : la thèse « `requires_gate` forgé main = 6e type wire hors serde » est CONFIRMED, MAIS le finding proposait de citer le forge à `handle_chat_message` (`:1452-1465`) — ces lignes posent un **champ booléen `"requires_gate": true` d'une réponse POST NON-SSE**, pas la frame SSE. La frame SSE `{"type":"requires_gate","message":"..."}` est forgée par **`fn sse_gate` `operator_server.rs:1591-1592`** (concordant S4-SSE-CONTRACT). **Documenter le contrat SSE en ancrant `sse_gate`**, jamais `handle_chat_message`, sous peine d'introduire une erreur factuelle neuve.

---

## 3. Compatibilité Day-0 / invariants (pas de DESIGN-CONFLICT)

- **API loopback interne pré-launch** : les 4 frontières sont des routes `/api/*` du control-center Operator (127.0.0.1, hors CSP scellée), pas du wire P2P (`Task`/`ProjectAnnouncement`/`FeedEntry`). Éditables librement pré-tag v1.0 (`CLAUDE.md §Pre-launch protocol policy`). **0 bump wire, 0 invariant en jeu.**
- **Factory hors daemon** (v4 D2) : `sbfb-factory` reste la crate outil-client ; documenter son API ne recentralise rien.
- **Operator hors CSP** (décision gelée) : `operator_server.rs:342-349` — « The Operator is NOT under the sealed BLOB_SERVE_CSP » ; documenter ses routes ≠ contredire le contrat sealed-iframe app-authoring. Ce sont DEUX sous-domaines distincts (S4-DOMAIN-SCOPE CONFIRMED).
- **Consommée-jamais-autoritaire** : le GUIDE/index restitue 1:1, n'émet aucun verdict PASS (invariant U1 process-as-artifact, préservé par les marqueurs d'honnêteté du gate).

**Aucune Day-0 contredite → pas de DESIGN-CONFLICT.** Le seul écart au plan littéral est de nature PLAN-ADAPT (scope étendu par amendement canon + prémisse Docker corrigée + formats canon à respecter).

---

## 4. Les 15 source-refs des 4 frontières — grep-résolvables (re-vérifiés main-thread)

> Le source-ref-check (`check-factory-docs.sh:156-194`) itère **AGENT_DOCS = WIRING_SPEC + llms.txt** ; tout backtick `` `(crates|docs|web|scripts)/…:Symbol` `` doit résoudre (fichier existe + symbole non-numérique grep-trouvé). **Copier VERBATIM ces 15 refs dans la nouvelle H2 de llms.txt** — chacun vérifié `OK` (script de vérif joué en main-thread) :

| # | Frontière | Ref rank-1 (backtick dans llms.txt) | Rôle |
|---|---|---|---|
| 1 | auth cookie | `crates/sbfb-factory/src/auth.rs:AUTH_HEADER` | `x-sbfb-token` = racine de confiance |
| 2 | auth cookie | `crates/sbfb-factory/src/auth.rs:OPERATOR_COOKIE` | `sbfb_operator` |
| 3 | auth cookie | `crates/sbfb-factory/src/auth.rs:session_secret` | secret de session per-boot ≠ bearer |
| 4 | auth cookie | `crates/sbfb-factory/src/operator_server.rs:handle_bootstrap` | GET /?token → Set-Cookie HttpOnly + 303 |
| 5 | git diff | `crates/sbfb-factory/src/operator_server.rs:handle_git_diff` | route GET /api/git/diff |
| 6 | git diff | `crates/sbfb-factory/src/sprint_history.rs:working_tree_diff_data` | calcul Rust {head,unstaged,staged,truncated} |
| 7 | gates | `crates/sbfb-factory/src/operator_server.rs:handle_gates` | route GET /api/gates |
| 8 | gates | `crates/sbfb-factory/src/gates.rs:gates_live_data` | producteur idempotent (0 scan publish) |
| 9 | gates | `crates/sbfb-factory/src/gates.rs:GateStatus` | 5 valeurs snake_case |
| 10 | gates | `crates/sbfb-factory/src/gates.rs:GatesView` | enveloppe SANS champ agrégé |
| 11 | gates | `crates/sbfb-factory/src/gates.rs:GateEntryView` | {gate,status,issues[]} clé (gate,status) |
| 12 | gates | `crates/sbfb-factory/src/gates.rs:GateIssueView` | {message,file,line} — line null en S80 |
| 13 | SSE | `crates/sbfb-factory/src/llm_bridge.rs:StreamChunk` | 5 variantes serde (tag "type") |
| 14 | SSE | `crates/sbfb-factory/src/operator_server.rs:handle_chat_stream` | route GET /api/chat/{id}/stream |
| 15 | SSE | `crates/sbfb-factory/src/operator_server.rs:sse_gate` | forge `requires_gate` hors serde |

**Consommateurs front** (à citer en **liens markdown** relatifs, link-checkés mais HORS source-ref-check car `tools/` ∉ `(crates|docs|web|scripts)`, S1a-06) : `[streamChunk.ts](../../tools/factory-operator/src/lib/streamChunk.ts)`, `[useTokenStream.ts](../../tools/factory-operator/src/lib/useTokenStream.ts)`, `[operator.ts](../../tools/factory-operator/src/lib/operator.ts)`.

---

## 5. FORMAT DE CLÔTURE RETENU (fichiers, sections, compatibilité gate PROUVÉE)

### 5.1 Décision de placement (résolution de la tension S2 ↔ S4/S3)

Tension entre scans : **S2-2** proposait d'indexer aussi dans **WIRING_SPEC.md** (« si concerné = OUI ») ; **S4-DOMAIN-SCOPE** (CONFIRMED P2) + **S3-SCOPE** (CONFIRMED P2) démontrent que **WIRING_SPEC gouverne le sealed-iframe untrusted**, catégoriquement distinct de l'API control-center (Operator explicitement hors `BLOB_SERVE_CSP`, `operator_server.rs:342-349`). **Résolution : WIRING_SPEC = NON concerné** (le « si concerné » du scope J = NON). Injecter l'API Operator dans WIRING_SPEC serait une erreur de catégorie et diluerait son contrat d'authoring.

**Home retenue = REFERENCE.md (corps) + llms.txt (index gaté source-ref).** On NE crée PAS de `docs/factory/OPERATOR_API.md` standalone (S1a-03 ADJUSTED P3 : un nouveau doc serait NON gaté = exactement le trou Track K ; le câbler dans le script serait du travail Track K contingent, hors scope J). Folder dans REFERENCE.md (déjà dans ALL_DOCS, donc link-checké + honesty-gaté) est le geste scope-approprié et sûr.

### 5.2 Fichiers à éditer (3) + non touchés

| Fichier | Action | Gate qui le couvre |
|---|---|---|
| **`docs/factory/llms.txt`** | **NOUVELLE H2** `## Operator control-plane API (loopback, hors sealed-iframe)` + les **15 backtick source-refs** (§4) + 1 phrase de scope dans le blockquote (l'index n'implique plus app-authoring-SEULEMENT) | source-ref-check (grep-verifié) + honesty markers (déjà présents :8,:14) + link-check |
| **`docs/factory/REFERENCE.md`** | **NOUVELLE section** `## Operator control-plane API (loopback)` avec les **4 résumés de contrat** (§6), corps english (précédent :8), liens markdown vers source + consommateurs TS, **1 lien** `[THREAT_MODEL.md §14](../security/THREAT_MODEL.md)` pour les mitigations | link-check + honesty (`PROVISIONAL`+`Not evidenced` déjà présents :7,:10 — inchangés) |
| **GUIDE Diátaxis FR** (`EXPLANATION.md` **ou** `HOW_TO_WIRE.md`) | **1 paragraphe** FR pointant « l'API loopback de l'Operator (control-center privilégié, hors iframe scellée) est référencée dans REFERENCE.md §Operator control-plane API » + lien | french-body (éviter EN_WORDS : Welcome/Dashboard/Sign in/Loading…) + honesty markers déjà présents |
| **`docs/factory/WIRING_SPEC.md`** | **NON TOUCHÉ** (NON concerné) — les 9 REQUIRED_ANCHORS restent intacts trivialement | inchangé |

### 5.3 Compatibilité `check-factory-docs.sh` — PROUVÉE contre les checks lus

| Volet (ligne script) | Ce qu'il exige | Preuve de passage |
|---|---|---|
| **(1) link-check** (`:60-76` ALL_DOCS ; `:138-154` AGENT_DOCS) | tout `](path)` repo-relatif résout depuis le dir du doc | REFERENCE.md ajoute des liens vers `../../crates/...` (existent), `../../tools/factory-operator/src/lib/*.ts` (existent, vérifiés disque), `../security/THREAT_MODEL.md` (existe). llms.txt liens idem. **PASS** |
| **anchors** (`:85-88`) | `FG-CSP-authoring`, `§P71`, `BLOB_SERVE_CSP`, `run_gate_csp_authoring` présents | inchangés (aucune édition de ces cibles). **PASS** |
| **(2) honesty-gate** (`:97-104`) | REFERENCE : `PROVISIONAL` (:7) + `Not evidenced` (:10) ; FR docs : caveat cardinal + `0 verdict PASS` + `PROVISIONAL` | présents avant J ; nos ajouts ne les retirent pas. **PASS** |
| **(3) french-body** (`:110-117`) | 0 EN_WORDS dans les 3 FR docs ; REFERENCE exempt | paragraphe FR du GUIDE rédigé sans EN_WORDS. **PASS** |
| **(4) source-ref-check** (`:156-194`, AGENT_DOCS) | tout backtick `crates/…:Symbol` de **llms.txt** résout | **15/15 OK** (§4, vérifié main-thread). **PASS** |
| **REQUIRED_ANCHORS** (`:200-209`, WIRING_SPEC) | 9 symboles app-authoring présents comme queue de source-ref | WIRING_SPEC NON touché → 9 intacts. **PASS** |
| **Truth-Stack + Not evidenced** (`:214-218`) | présents dans WIRING_SPEC + llms.txt | inchangés. **PASS** |
| **honesty-gate extension** (`:224-230`) | llms.txt : `PROVISIONAL` + caveat cardinal 2 clauses ; root llms.txt : `app-authoring (factory)` | présents (:8,:14) ; on n'y touche pas. **PASS** |
| **(5) fiche prompt-kind** (`:240-267`) | refs `PRIMITIVES.md:N`/`README.md:N` in-bounds | fiche non touchée. **PASS** |

> **Piège à éviter** : la nouvelle H2 de llms.txt ne doit citer QUE des backtick rank-1 qui résolvent. Un ref mal orthographié ou vers `tools/…` en backtick casserait le source-ref-check (`tools/` ∉ rank-1 ⇒ ignoré silencieusement, donc NON validé — S1a-06). Les consommateurs TS se citent en **liens markdown**.

### 5.4 Compatibilité `check-frontier-contracts.sh` (anti-promesse)

**Vacuous pour `docs/`** (S2-3 / S3-ANTIPROMISE CONFIRMED) : le volet anti-promesse scanne `find crates web/src` (`:77`) ; `docs/` est exclu « by construction » (`:20-23`) ; `.planning/` n'est scanné par aucun des deux scripts. **⇒ l'anti-promesse est une discipline de rédaction, pas un gate**, à appliquer À LA MAIN dans llms.txt + REFERENCE.md + GUIDE + verification.md + sprint81_audit_plan.md :
- Provenance au **PASSÉ immuable** : citer les commits `a5ace8d` (auth), `bb35d39` (diff), `ed00b4a` (gates), `6991d51` (SSE). **JAMAIS** « Phase X will », « S81 will add », « le Viewer arrivera en Phase B », « lands in Phase ».
- Dette : écrire « `line` est **null en S80** » (fait présent), pas « S81 will add the line anchor ».
- Ancrer par **SYMBOLE** (`handle_bootstrap`, `working_tree_diff_data`, `gates_live_data`, `sse_gate`), pas par numéro de ligne (API loopback pré-launch qui dérive — pratique déjà adoptée dans `streamChunk.ts` et llms.txt).

### 5.5 Décision de langue (S1a-08, à acter explicitement)

REFERENCE.md est **english-body par design** (`:8`, exempt du french-body gate). Le canon mission dit « docs/factory/ = français Diátaxis », mais REFERENCE est l'**exception documentée** (audience reference-for-agents). **Décision retenue : la section Operator dans REFERENCE.md suit le corps ANGLAIS** (même audience, précédent gate-béni) ; le pointeur GUIDE est en FR. Le gate passe dans les deux cas (REFERENCE non-french-gaté) — la décision est actée ici, pas laissée implicite.

---

## 6. Les 4 résumés de contrat prêts-à-rédiger (S4, ancrés par SYMBOLE)

> À coller dans REFERENCE.md §Operator control-plane API. Provenance au passé (commits). Ancrer par symbole.

### 6.1 Frontière (a) — Amorçage auth Operator (Phase A `a5ace8d`)
GET `/?token=<hex>` est **hors** `auth_required` (chicken-and-egg : joignable avant tout cookie). Il valide le bearer en temps constant (`token_matches`), puis pose `Set-Cookie: sbfb_operator=<session_secret>; HttpOnly; SameSite=Strict; Path=/` et répond **303 See Other** vers `/` avec `Referrer-Policy: no-referrer` (le token quitte la barre d'adresse). Le cookie porte un **secret de session per-boot** (`auth.rs:session_secret`), **JAMAIS le bearer** : la racine de confiance reste l'en-tête `x-sbfb-token` (`auth.rs:AUTH_HEADER`). Le cookie n'est accepté comme transport de repli navigateur (SSE/WS ne peuvent poser d'en-tête) que si `Sec-Fetch-Site: same-origin` est présent (garde CSRF cross-port ; cookies non port-scopés RFC 6265). Réponse **identique** token-absent/token-faux (aucun oracle). Le front n'ajoute jamais d'en-tête d'auth (`credentials:'same-origin'` laisse le navigateur joindre le cookie). Mitigations : `THREAT_MODEL.md §14` (T-OPERATOR-CSRF). Ancres : `operator_server.rs:handle_bootstrap`, `auth.rs:OPERATOR_COOKIE`.

### 6.2 Frontière (b) — GET /api/git/diff working-tree (Phase F `bb35d39`)
Restitue l'arbre de travail du dépôt calculé **EN RUST** — jamais un diff JS (invariant kickoff #11 : source de vérité unique). Read-only, 0 entrée utilisateur. Enveloppe `{head, unstaged, staged, truncated}` : `head` = sha court HEAD (fraîcheur `run@<rev>`) ; `unstaged` = `git diff`, `staged` = `git diff --cached` (un fichier partiellement stagé apparaît légitimement dans les DEUX tableaux, sémantique git) ; `truncated=true` au-delà de `MAX_DIFF_LINES=20000` (coupe à une frontière de ligne). Chaque `FileDiff` = `{path, insertions, deletions, hunks[]}` ; chaque hunk = `{header, lines[]}` ; chaque `DiffLine` = `{kind: "add"|"del"|"ctx", content, old_lineno, new_lineno}`, `old_lineno`/`new_lineno` sérialisés **null** quand absents (contrat Zod `.nullable()` côté front). Fichiers non-suivis absents (pas dans `git diff`). Ancre : `sprint_history.rs:working_tree_diff_data`.

### 6.3 Frontière (c) — GET /api/gates (Phase G `ed00b4a`)
Diagnostic **1:1 read-only et idempotent** : **AUCUN scan publish** déclenché sur ce GET (un effet de bord casserait l'idempotence). Enveloppe `{gates:[...]}` **SANS champ agrégé racine** — pas de `overall`/`all_passed`/`score`. Chaque `GateEntryView` = `{gate, status, issues[]}` ; `status` = enum `GateStatus` snake_case à **EXACTEMENT cinq valeurs** : `not_run` / `not_applicable` / `passed` / `informational` / `blocking`. Un même gate peut apparaître sous **plusieurs statuts** (`lint-planning` scinde erreurs→`blocking` et warnings→`informational`) : indexer par la clé **(gate, status)**, jamais par gate seul. Invariant cardinal : l'Operator ne calcule **AUCUN verdict agrégé** (0 verdict calculé UI) ; le front restitue 1:1 et ne fabrique jamais un PASS — les mots d'acceptance (`PROVISIONAL`/`Not-evidenced`/`RIG-ABSENT`) ne sont PAS dans l'enum. Chaque `GateIssueView` = `{message, file, line}` ; `line` est **null en S80** (ancre de ligne = dette S81). Ancres : `gates.rs:gates_live_data`, `gates.rs:GateStatus`.

### 6.4 Frontière (d) — Contrat SSE chat (Phase C `6991d51`)
`POST /api/chat/session` → `{id, context_pack}`, puis `POST /api/chat/{id}/send` (persiste le provider + le model du tour et applique le MUR), puis `GET /api/chat/{id}/stream` bodyless (l'auth chevauche le cookie same-origin). Le flux émet des frames `data: <json compact>\n\n` **SEULES** — aucun `event:`/`id:`/heartbeat/keep_alive : **EOF = signal de fin**. **Six types wire** : les cinq variantes serde de `StreamChunk` (`delta`, `thinking`, `done`, `error`, `debug` ; tag `"type"`) + **`requires_gate` forgé à la main hors serde par `sse_gate`** (`operator_server.rs:1591-1592`). Le MUR `SENSITIVE_ACTIONS` s'exécute **AVANT tout dispatch** : un message sensible renvoie `requires_gate` et ne spawn JAMAIS d'agent — refus **structurel** (0 spawn), jamais un bouton. Invariant PO-14 : **UN SEUL `done`** (le bras Network porte un `done` unique, zéro `delta`) ; le front latch le **PREMIER** événement terminal `{done|error|requires_gate}` et ignore la suite, via `fetch + ReadableStream + AbortController` — **JAMAIS EventSource** (qui se reconnecterait et rejouerait le tour). Sept statuts front `StreamStatus` (les cinq + `gate` + `ended`). La variante `debug` porte un `content` qui **peut contenir le prompt assemblé verbatim** — documenter la SHAPE `{type:"debug",label,content}`, **NE PAS coller un dump** (S3-SSE-DEBUG-PROMPT-DUMP). Ancres : `operator_server.rs:handle_chat_stream`, `operator_server.rs:sse_gate`, `llm_bridge.rs:StreamChunk`.

---

## 7. Inventaire NOMMÉ des carries à router dans `sprint81_audit_plan.md §3`

> Chaque item avec ancre (fichier:ligne/commit), sévérité re-jugée maison. **Filtrer les zombies AVANT de router** (S2-5).

### 7.1 NE PAS re-router (FERMÉS S80 — vérifier le statut LIVE, S2-5)
- `TEST-ISOLATION-SBFB-HOME` — **FERMÉ Phase I `782796c`** (« workspace git fixture + SBFB_HOME mkdtemp per-run »). `sprint80_audit_plan.md:81` le liste ouvert = périmé.
- P2-4 couverture Vitest / gating CI Vitest — **FERMÉ Phase I `782796c`** (S2-F2 Vitest gaté CI).
- P2-1 Phase A bootstrap Host non-loopback 403 — **FERMÉ in-phase A** (`sprint80_phase_a_review.md:309`).

### 7.2 P1 standing (dette dominante)
1. **Sharding S77 in-vivo RIG-ABSENT** — orchestrateur de session in-vivo + benchmark live 2-machines (RTX 5080 / Mac M2 absents), différé S78 Factory-first (`sprint78_audit_plan.md §7/§10`).
2. **app-authoring in-vivo `Not evidenced`** — parcours auteur réel → gate → self-check → publish → rendu cross-pair jamais exercé ; efficacité générative prompt-kind/Ollama non mesurée (`docs/factory/llms.txt:8-12`).

### 7.3 P2/P3 encore ouverts (S79 + S80 phases + Fix 5)
3. **S79 audit findings** (`archive/v2.1/sprint79_audit_findings.md`, « 8 P2 / 11 P3 backend/docs ») — re-vérifier lesquels restent ouverts, filtrer les fermés.
4. **Couverture étiquette** ~21 familles wire non-schématisées (registre `// FRONTIER:` incrémental).
5. **Doc-lint sémantique limite** — `check-factory-docs.sh` vérifie l'existence d'une ligne, pas le support de la claim (revue LLM adversariale, pas un gate).
6. **Fix process 5 (`a6b4ca4`)** — parité Rust↔TS `audit-gate-checks` + **élargir le scan des gates à `tools/factory-operator`** (`check-frontier-contracts.sh:77` ne couvre que `crates`+`web/src` ; la fausse promesse STALE-PHASE-K y a échappé, `sprint80_phase_i_review.md:73`).
7. **SSE_GATE `requires_gate` forgé par `format!` brut** `operator_server.rs:1591-1592` → durcir `serde_json::to_string` si le message devient dynamique.
8. **Asymétrie blake3** daisyui (`exists==true`) vs animejs (recompute) — Phase D.
9. **GET /api/git/diff `truncated==true`** branch non testé hermétiquement — Phase F (`sprint80_phase_f_review.md:201-215`).
10. **Docs périmées `tools/factory-ui`** encore vivantes dans `docs/agent/RRV_FACTORY_CONTRACT.md:109,142` (+arbre README).
11. **`GateIssueView.line=null` hardcodé** `gates.rs` + refactor `GateResult.issues`+`line` fine — Phase G.
12. **V5/V6 + marqueur-gate-par-fichier + Aperçu scellé/Preuve DÉGRADÉS** — Phase H (fondation Viewer S81).
13. **Fraîcheur head-live figé au mount** ment après 1er commit — Phase H (re-poll `/api/context` ou rev sur `/api/gates`).
14. **P3-e prompt-injection surface `onHunkIntent`** `VerifyScene.tsx:98` (hunk hostile → session LLM privilégiée ; atténué manuel+gate) — Phase H.
15. **HEAD-50-YOUNG-REPO** : range `HEAD~50..HEAD` invalide sur jeune repo, `fixture-workspace.mjs:102-107` + PATTERNS §P72 — Phase I.
16. **PO-MULTILINE-SCAN** : `scan-front-discipline.sh:83` anti-score `.po` matche 1re ligne `msgstr` seule — Phase I.
17. **CALLS-ORDERING** : `/__calls` compteur cumulatif partagé + dépendance wall-clock `steer.spec.ts:40` — Phase I.
18. **INFO/RR-1** : harness t2-acceptance ne rougit qu'à `length===0`, un test supprimé laisse PASS avec N<10 scénarios — Phase I.

### 7.4 Standing tracks (Track J + Track K, canon `a6b4ca4`)
19. **Track J — testabilité standing** : T1 E2E Playwright hermétique BLOQUANT-vert + T2 acceptance JSON machine-lisible à chaque sprint.
20. **Track K — docs-contract closure standing (NEUVE)** : frontière neuve non indexée GUIDE/llms.txt à la fermeture = P1 sprint suivant (miroir Track J). Canon `prompts/agent/audit-gate-checks.md`.

### 7.5 Fondation Viewer S81
21. **Fondation Viewer/Operator re-planifiée S81** (kickoff Arbitrage PO #2 : socle `tools/factory-ui` jeté). Décision PO **S81-vs-sharding** à mentionner.

---

## 8. Compteurs vérifiés à consigner (S1b — tous EXACTS)

| Suite | Valeur à consigner | Source |
|---|---|---|
| Rust nextest **Windows natif** | **2014 / 2014** INCHANGÉ (S80 front-pur) | `git show -s 782796c` §Delta |
| Rust nextest **Docker Linux** (count) | **2018** (= 2014 Win + 4 `#[cfg(unix)]`) | log Docker `sbfb-phase-j` |
| Rust nextest **Docker Linux** (statut) | ⚠️ **2 FAILED** (voir §9 honnêteté) | log Docker (exit 100) |
| Vitest factory-operator | **201** (35 fichiers) | `git show -s 782796c` |
| E2E Playwright factory-operator | **10** | idem |
| Vitest web/ | **411** INCHANGÉ | idem |
| T2 acceptance | **PASS** (10 scénarios, 9 gates) | `sprint80_t2_acceptance.json:3` (committé `782796c`) |

**Trajectoire Vitest operator** : C 52 → D 77 → E 92 → H 137 → I **201**. E2E : 8 → **10** (I +2). Le count Docker **2018** est correct ; c'est le **statut** (2 échecs) qui est en défaut, pas les compteurs.

---

## 9. Règles d'honnêteté verification.md (S3 — impératif)

### 9.1 Docker RED — pas de « GREEN » (S1b-01 / S3-DOCKER-EXIT100, P1 CONFIRMED)
Le run `sbfb-phase-j` = **exit 100, 2018 tests, 2016 passed, 2 failed** (`operator_context_pack_schema_complete` + `operator_git_diff_endpoint_returns_envelope`, `reqwest TimedOut` 30s sur `/api/context-pack` + `/api/git/diff`). Ces 2 tests sont **VERTS en Windows-natif** (cross-check main-thread S3 : 0.6s / 0.7s) ⇒ **PAS une régression** = signature de **contention** (7 conteneurs langfuse/open-webui up 6h + build cold ; frères operator_server à 24-27s au ras du plafond 30s). **verification.md §How-to-re-run + §Métriques DOIVENT** :
- Consigner le compte **RÉEL** (2016/2 fail, exit 100) — jamais « Docker 2018 GREEN ».
- Voie honnête (a) recommandée : **RE-RUN** avec atténuation `SBFB_TEST_HTTP_TIMEOUT_SECS=120` et/ou pile langfuse stoppée + cache cargo chaud (échappatoire documentée `operator_server.rs:24`, non utilisée dans le run initial), puis consigner le résultat du **RE-RUN**. Le brief « ne relance pas » est **corrigé par evidence** (le run est TERMINÉ et ROUGE, pas EN COURS).
- Voie (b) si le RE-RUN reste vert-cross-check-Win mais rouge-Docker de façon reproductible : documenter une **classe d'env-block** « loopback-HTTP-server TimedOut sous Docker-on-Windows (réseau hôte dégradé, même racine que `multi_daemon` iroh-networked) », cross-check Windows-natif vert consigné comme preuve. **NE PAS** inventer un GREEN.
- Sans l'une des deux, la DoD (c) §4 (gate testabilité + T0 verts) n'est pas honnêtement tenue.

### 9.2 Verdicts fermés machine-lisibles seuls (S3-VERDICT-VOCAB-DOD)
§Acceptance de verification.md : **T1** ∈ `{GREEN, RED, N-A}` uniquement ; **T2** ∈ `{PASS, BLOCK{diagnosis}, RIG-ABSENT, N-A}`. **Aucune prose `DIFFERE-*`** (README:646 → P1 Track J). T2 = **PASS déjà committé** (`sprint80_t2_acceptance.json`). Le T1 réel (5 specs `tools/factory-operator/e2e/*.spec.ts`) consigné **d'après le run**, pas supposé GREEN.

### 9.3 Verdict docs-contrat DoD (d) — obligatoire, neuf (`a6b4ca4`)
verification.md **DOIT porter** le verdict de clôture docs-contrat : les 4 frontières (auth-cookie / git-diff / gates / SSE) **indexées** (§5). **`N-A-no-new-frontier` NON applicable** — S80 a 4 frontières réelles (§6.12 zone grise `:2033-2040`). Frontière neuve non indexée → **consigner explicitement, jamais omettre en silence** (§3.3:545).

### 9.4 Fuite / duplication
- **Auth cookie** : n'utiliser que `<token>`/`<hex>` — jamais un 64-hex concret même fictif (fuite au sens mission). Énoncer cookie = secret de session per-boot HttpOnly+SameSite=Strict, **distinct** du bearer (S3-LEAK).
- **Routes /api/*** : carte de surface honnête (loopback+auth) mais **LIER** `THREAT_MODEL.md §14` (T-OPERATOR-CSRF/SPAWN), **ne pas dupliquer** la garde CSRF (source unique = THREAT_MODEL). Ne pas transformer le GUIDE en second modèle de menace divergent.
- **SSE debug** : documenter la SHAPE, jamais un dump de prompt (S3-SSE-DEBUG).

### 9.5 SPRINT_LOG + CLAUDE.md (S2-7)
- `SPRINT_LOG.md` : insérer **row 80 en TÊTE de table** (au-dessus de row 79 à `:19`, format 5-colonnes `| Sprint | Etat | Tip cloture | Nb commits | Docs |` ; S78 absent car différé).
- `CLAUDE.md §Etat actuel` : réécrire « Sprints 0-77 + **S79-S80 DONE** (S78 différé) » ; S80 = refonte greenfield front Operator + auth cookie + /api/git/diff + /api/gates + contrat SSE + testabilité T1/T2 + **clôture docs-contrat**, avec delta réel + carries P1 (sharding + app-authoring in-vivo + fondation Viewer S81).
- `nexus_grid_pivot.md` + `MEMORY.md` : maj tip + S80 DONE (feedback_memory_update).

---

## 10. `sprint80_verification.md` — les 9 sections canon (README §2.3, NE PAS copier S79 6-sections)

> **PIÈGE (S2-1 CONFIRMED)** : `sprint79_verification.md` n'a que 6 sections. Écrire les **9** :

1. **HEAD entrée / HEAD sortie** — entrée = tip S79 `f4b4600` ; sortie = commit Phase J (à créer).
2. **Commit stack** — `git log --oneline master ^f4b4600` (A→J).
3. **How to re-run** — bloc bash copiable : 3 blocs fail-fast (Rust dual-platform Win + Docker `sbfb-ci` + frontend lint/tsc/vitest/coverage/build/size/scan-en-strings) + 3 doc-lints (`check-factory-docs.sh`, `check-frontier-contracts.sh`) + la commande Docker RE-RUN avec `SBFB_TEST_HTTP_TIMEOUT_SECS`.
4. **Checklist** — table `plan.md §Fail-fast` avec colonne **Observed** remplie + ✅/⚠️ par row (Docker = ⚠️ voir §9.1).
5. **Métriques** — `Suite | Avant | Après | Delta` (§8).
6. **Surface LOC** par nouveau module (front greenfield, `gates.rs` GatesView, `operator_server.rs` router split, `sprint_history.rs` working_tree_diff_data) — S80 corps, pas Phase J.
7. **Ce que S80 n'a PAS livré** — reprise EXHAUSTIVE §8 kickoff S80, ❌ par item, ne pas tronquer.
8. **Findings carry-over for memory (G6)** — ≤ 5 items.
9. **Checkpoint de clôture** — N conditions cochées (dont verdict docs-contrat §9.3).

## 11. `sprint81_audit_plan.md` — 11 tracks canon (README §2.4, `a6b4ca4`)

Honorer le CONTENU README §2.4 même en forme compressée : Mode d'emploi session fraîche ; **11 tracks** (A suites, B security, C patterns, D scope, E tests delta, F review files, G carry-overs, H HARDENING drift, I meta-process, **J testabilité standing**, **K docs-contract closure standing — NEUVE**) ; Track G1 presence (vérifier `sprint80_design_review.md` existe → il est dans `active/`) ; les 3 scénarios de verdict (PASS/CONDITIONAL/FAIL) ; Out-of-scope (D1..D5 gelées) ; format livrable final `audit_findings.md`. §3 Carries = l'inventaire NOMMÉ §7 (zombies filtrés).

---

## 12. Adaptations PLAN-ADAPT (numérotées, evidence)

> Aucune ne touche une Day-0. Les P1/P2 CONFIRMÉS sont tous adressés.

1. **Ajout de la clôture docs-contrat au scope J** — actée par l'amendement canon `a6b4ca4` (README §3.3 livrable 3 + DoD (d) §4 + §6.12 porteurs). Phase J = 1re application réelle. Home = REFERENCE.md + llms.txt (§5). *Evidence : `a6b4ca4`, S2-2/S4-DOMAIN-SCOPE CONFIRMED.*
2. **WIRING_SPEC = NON concerné** — le « si concerné » du scope résout à NON (API control-center ≠ sealed-iframe, `operator_server.rs:342-349`). Ne pas y injecter l'API ; 9 REQUIRED_ANCHORS intacts. *Evidence : S4-DOMAIN-SCOPE + S3-SCOPE CONFIRMED.*
3. **Pas de nouveau doc standalone** — folder dans REFERENCE.md (déjà gaté) plutôt que créer `OPERATOR_API.md` non-gaté (= trou Track K). Un futur doc dédié = câblage script contingent, routé Track K. *Evidence : S1a-03 ADJUSTED P3.*
4. **Docker RED — RE-RUN + honnêteté** : la prémisse « consigne les résultats, ne relance pas » est corrigée (run TERMINÉ ROUGE). RE-RUN avec `SBFB_TEST_HTTP_TIMEOUT_SECS=120` / pile stoppée ; consigner le RE-RUN ; jamais « GREEN » supposé ; cross-check Windows-natif vert. *Evidence : S1b-01/S1b-04/S3-DOCKER CONFIRMED (Win-natif vert).*
5. **verification.md = 9 sections canon** (pas le gabarit S79 6-sections). *Evidence : S2-1 CONFIRMED, README §2.3:372-395.*
6. **sprint81_audit_plan = 11 tracks** (Track J + Track K NEUVE). *Evidence : S2-4 CONFIRMED, `a6b4ca4`.*
7. **Filtrer les zombie-carries** (`TEST-ISOLATION-SBFB-HOME`, P2-4 Vitest CI, P2-1 Phase A = FERMÉS) avant de router §7. *Evidence : S2-5 CONFIRMED, `782796c`.*
8. **`requires_gate` = frame forgé** (5 serde + 1 forgé = 6 wire) ancré `sse_gate` `operator_server.rs:1591-1592`, jamais `handle_chat_message`. *Evidence : S1a-05 ADJUSTED, S4-SSE-CONTRACT CONFIRMED.*
9. **Anti-promesse à la main** dans TOUS les nouveaux docs + verification + audit_plan (gate vacuous pour `docs/`+`.planning/`). Ancrer par symbole, provenance au passé (commits). *Evidence : S2-3/S3-ANTIPROMISE/S4-ANTI-PROMISE CONFIRMED.*

---

## 13. Scope (confirmation des cuts)

**DANS le scope S80 Phase J** :
- Clôture docs-contrat : nouvelle H2 llms.txt (15 source-refs) + section REFERENCE.md (4 résumés §6) + pointeur GUIDE FR ; WIRING_SPEC NON touché.
- `sprint80_verification.md` (9 sections canon, verdicts fermés, verdict docs-contrat, Docker honnête).
- `sprint81_audit_plan.md` (11 tracks, carries §7 zombies filtrés).
- SPRINT_LOG row 80 + CLAUDE.md/`nexus_grid_pivot.md`/MEMORY.md S80 DONE.
- Pipeline fail-fast 3 blocs (déjà verts Phase I) + Docker RE-RUN consigné.

**HORS scope (cut cohérent)** :
- Tout code `.rs`/`.ts` / manifeste → **0 dep, 0 code** (phase docs-pure).
- Création de `OPERATOR_API.md` standalone → **rejeté** (non-gaté ; folder dans REFERENCE.md).
- Injection de l'API Operator dans WIRING_SPEC → **rejeté** (erreur de catégorie).
- Fondation Viewer S81, V5/V6, Aperçu scellé → **carries S81** (routés §7, pas livrés J).
- Fix des 2 timeouts Docker comme régression code → **N/A** (env-block, Windows-natif vert).

---

## 14. Risques résiduels / cibles à re-vérifier EN Phase J

1. **source-ref-check llms.txt** : après ajout de la H2, jouer `bash scripts/check-factory-docs.sh` — les 15 backtick refs doivent rester `OK` (0 faute de frappe ; `tools/` uniquement en liens markdown).
2. **honesty markers non-régressés** : confirmer que l'ajout REFERENCE.md/llms.txt ne retire aucun marqueur (`PROVISIONAL`, `Not evidenced`, caveat cardinal 2 clauses).
3. **french-body GUIDE** : le paragraphe FR ajouté (EXPLANATION/HOW_TO_WIRE) ne contient aucun EN_WORDS.
4. **Docker RE-RUN** : confirmer vert (ou classe d'env-block documentée + Windows-natif vert consigné) AVANT tout push ; le count Vitest gaté = 201+.
5. **anti-promesse manuelle** : relire llms.txt + REFERENCE + GUIDE + verification + audit_plan — 0 « will/adds/ships/arrivera en Phase ».
6. **T2 committé** : `sprint80_t2_acceptance.json` reste PASS, non-gitignored, allowlist (0 secret/HOME absolu).
7. **row SPRINT_LOG** : insertion en tête (au-dessus row 79), 5 colonnes, S78 absent.

## Verdict: PLAN-ADAPT
