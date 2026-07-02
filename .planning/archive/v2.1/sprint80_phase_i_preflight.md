# Sprint 80 — Phase I — Preflight (deep, 4 scans + vérification adversariale par finding, finalisé G8)

**Date** : 2026-07-02
**HEAD** : `03ab6f9` (master, arbre propre)
**Sprint / Phase** : 80 / I — Testabilité T1/T2 + re-couverture SSE single-Done + comptabilité honnête du delta. Test-infra + **un** ajout backend (cible SSE déterministe côté `sbfb-factory`). 0 dep runtime nouvelle (Rust comme JS).
**Verdict** : **PLAN-ADAPT**

> Le plan littéral de la Phase I reste exécutable et **compatible Day-0** (aucun invariant contredit → pas de DESIGN-CONFLICT), mais il a été écrit AVANT les phases C/H et AVANT l'arc front off-sprint `76a99d6..03ab6f9` (documents surface, a11y WCAG, i18n Lingui 51 locales). Résultat : **une grande partie de son périmètre est DÉJÀ livrée** et sa comptabilité de tests est périmée. Le preflight corrige l'approche par **9 adaptations étayées code** (§6), tranche la **question ouverte echo-vs-stub** (§3), et re-cible le vrai travail restant : le SEUL sous-test T1 (3) full-stack SSE + le gating CI Vitest + le scan anti-score statique + l'artefact T2 committé + la fermeture réelle de l'isolation hermétique (`repo_root` override + `SBFB_HOME` per-run). Les dimensions « déjà livré/caduc » (re-couverture `useTokenStream` faite Phase C ; picker modèle client coupé ; sous-tests T1 1/2/4/5 faits) sont **SCOPE-CUT-CONSISTENT** et englobées dans le PLAN-ADAPT global.

> Finalisation : 4 scans reçus (S1b deps/CVE+graphe transitif, S2 décisions+drift, S3 threat-model, S4 wire/invariants placeholder), chaque finding passé au crible adversarial. **Aucun finding pleinement RÉFUTÉ** ; 5 findings ADJUSTED (recalibrés en sévérité, evidence intacte). Faits load-bearing re-vérifiés en main-thread : `ExecutionTarget` enum 3 bras Claude/Ollama/Network (`provider_router.rs:63-72`), `from_provider` inconnu→Claude (`:89-92`), overrides `SBFB_OLLAMA_ENDPOINT` (`:155`) + `SBFB_DAEMON_ENDPOINT` (`:282-286`) ; harness E2E lance le VRAI binaire (`serve-operator.mjs:35-36`, `cwd: repoRoot`) ; `useTokenStream` = fetch+ReadableStream+AbortController, JAMAIS EventSource (`useTokenStream.ts:134`) ; 9 cas de test déjà présents (`useTokenStream.test.ts:46-165`) ; MUR `SENSITIVE_ACTIONS` appliqué AVANT dispatch (`operator_server.rs:1644-1658` puis `:1684-1685`) ; baseline RÉELLE = **200 Vitest / 35 fichiers / 8 e2e** (mesurée, pas le « −7/−8 » du plan).

---

## 1. Synthèse des scans

| Scan | Objet | Verdict | Apport load-bearing (evidence) |
|---|---|---|---|
| **S1a** OSS prior-art / SOTA | — | **N/A (non lancé)** | Phase test-infra : aucun nouveau composant/lib à sourcer. La cible SSE déterministe est du Rust-interne (§3). Rien à évaluer côté prior-art OSS. |
| **S1b** Deps / CVE / graphe transitif | Coût runtime + audit | **EXECUTE** | **0 dep ajoutée**, Rust ET JS. Bras SSE déterministe = Rust pur (`async-stream`+`futures`+`tokio` déjà là `Cargo.toml:29-32` ; `StreamChunk::Delta/Done` déjà définis). `npm audit` propre (`--omit=dev` ET complet). msw **pas installé** (peer OPTIONNEL de `@vitest/mocker`, `package-lock.json:3311-3317`) → le stub JS est la mauvaise couche. Dérive versions installées (vitest 4.1.9, @playwright/test 1.61.1) vs plan (4.1.4/1.59.1) = ranges `^`, 0 CVE. |
| **S2** Décisions historiques + drift plan↔code | Conformité + périmètre réel | **PLAN-ADAPT** | Re-couverture SSE single-Done **déjà livrée Phase C** (`6991d51`). Sous-tests T1 (1)(2)(4)(5) déjà livrés ; **seul (3) manque**. **Vitest PAS gaté en CI** (P1). Aucun `ExecutionTarget` echo n'existe (P1, question à figer). Scan anti-score non fait (P1). Aucun artefact T2 committé (P1). Baseline réelle 200/35/8. Aucune Day-0 contredite. |
| **S3** Threat model | Bypass MUR / isolation / fuite / scan | **PLAN-ADAPT** | MUR structurellement sûr pour tout bras déterministe (gate provider-indépendant AVANT dispatch). Recommande de **réutiliser les mock-endpoints env existants (0 code prod)** ; sinon feature Cargo compile-time ; JAMAIS `#[cfg(test)]` ni provider-string nu (§3). 2 vrais bloqueurs d'isolation : `repo_root()` sans override + bundle couplé (P1) ; `SBFB_HOME` chemin fixe (P2). Artefact T2 = allowlist de champs. |
| **S4** Wire / invariants | Shape wire | **placeholder (aucun finding substantiel)** | Scan reçu à l'état de stub (`X1` info « t »). Cohérent avec la phase : `provider_router`/`operator_server` sont des **API loopback internes** (pré-launch éditables), pas du wire P2P ; aucune enveloppe `Task/ProjectAnnouncement/FeedEntry` touchée → 0 invariant wire en jeu. Absence de finding = normal, pas un angle mort. |

**Recommandations par scan** : S1b EXECUTE, S2 PLAN-ADAPT, S3 PLAN-ADAPT, S4 (aucune). **Global = PLAN-ADAPT** (deux scans PLAN-ADAPT avec P1 CONFIRMÉS à adresser ; S1b EXECUTE subsumé ; aucune Day-0 réfutée).

---

## 2. Verdicts adversariaux intégrés (par finding)

Chaque finding a été vérifié contre la source. **Aucun RÉFUTÉ.** Les 5 ADJUSTED gardent leur evidence (réelle) mais voient leur sévérité recalibrée — ils **n'influencent le verdict qu'à leur sévérité corrigée**.

| Finding | Titre court | Sévérité déclarée → **retenue** | Verdict adversarial |
|---|---|---|---|
| **S1b-F1** | Bras echo = 0 nouvelle dep Rust | info → **info** | contexte (confirme EXECUTE deps) |
| **S1b-F2** | `npm audit` = 0 vuln (prod+complet) | info → **info** | contexte |
| **S1b-F3** | Stub JS = mauvaise couche + non installé → echo Rust seul choix 0-dep déterministe | P2 → **P2** | **CONFIRMED** |
| **S1b-F4** | Dérive version plan↔installé (honnêteté delta) | P3 → **info** (ADJUSTED) | evidence réelle ; lockfile déjà correct, drift dans les ranges `^` |
| **S1b-F5** | Paquets WASM `extraneous` | P3 → **info** (ADJUSTED) | evidence réelle ; `npm ci`/`npm ls` verts, remédiation proposée fausse |
| **S1b-F6** | Re-couverture `useTokenStream` ~90% en place ; gap = « fetch appelé une seule fois après Done » | info → **info** | contexte (0 dep pour combler) |
| **S1b-F7** | Doublons crates pré-existants (iroh), echo n'en crée aucun | info → **info** | contexte |
| **S2-F1** | Re-couverture SSE single-Done DÉJÀ livrée Phase C | P2 → **P2** | **CONFIRMED** |
| **S2-F2** | **Vitest PAS exécuté en CI** (196 cas gatés nulle part) | P1 → **P1** | **CONFIRMED** |
| **S2-F3** | Aucun `ExecutionTarget` echo n'existe — figer la question | P1 → **P1** | **CONFIRMED** |
| **S2-F4** | Extension anti-score/jauge du scan front NON faite | P1 → **P1** | **CONFIRMED** |
| **S2-F5** | Aucun artefact T2 JSON committé (P3-6 ouvert) | P1 → **P1** | **CONFIRMED** |
| **S2-F6** | Baseline delta STALE (réel 200/35, pas −7/−8) | P2 → **P2** | **CONFIRMED** (chiffres corrigés 200/35, cf. note) |
| **S2-F7** | E2E (1)(2)(4)(5) livrés ; SEUL (3) reste | P2 → **P2** | **CONFIRMED** |
| **S2-F8** | `SBFB_HOME` chemin fixe + specs git sur CE repo | P2 → **P2** | **CONFIRMED** |
| **S2-F9** | Picker modèle client (P2-OLLAMA-MODEL-PICKER) perdu au jettison | P3 → **P3** | **CONFIRMED** (picker client coupé ; garde vit côté Rust) |
| **S3-F1** | Gating echo : réutiliser mock-endpoints env (0 code prod) ; sinon feature Cargo ; jamais `#[cfg(test)]`/provider-string nu | P1 → **P1** | **CONFIRMED** |
| **S3-F2** | `repo_root()` sans override + bundle couplé → diff/gates/sprint-history sur le VRAI dépôt (T2 non-déterministe) | P1 → **P1** | **CONFIRMED** |
| **S3-F3** | `SBFB_HOME` fixe partagé → carry pas vraiment fermé ; « empoisonnement » du bras réseau | P1 → **P2** (ADJUSTED) | evidence réelle ; mécanisme d'empoisonnement RÉFUTÉ (endpoint lu depuis `NEXUS_GRID_ROOT`, pas `SBFB_HOME` ; bras réseau non exercé par l'E2E). Reste = hygiène/isolation P2 |
| **S3-F4** | Artefact T2 doit exclure secret session/cookie + HOME absolu | P2 → **P3** (ADJUSTED) | evidence réelle ; patron allowlist canonique existe déjà (`app_authoring_capability.sh:90-98`), secrets éphémères/inertes |
| **S3-F5** | Scan anti-score doit couvrir toutes surfaces (incl. i18n `.po`) | P2 → **P3** (ADJUSTED) | evidence réelle ; anti-PASS/verdict `.po` DÉJÀ couvert par `check-i18n-verdict-cross-locale.sh` ; seul l'axe anti-**score** est neuf |
| **S3-F6** | T-OPERATOR-CSRF/SPAWN inchangés ; 0 nouvelle surface | info → **info** | contexte (gate substring : éviter « passenger »/« marshall » dans les prompts de test) |
| **S4-X1** | (placeholder « t ») | info → **info** | stub, non substantiel |

**Note S2-F6 (chiffres corrigés en main-thread)** : le finding annonce 196 Vitest / 34 fichiers ; la mesure réelle à `03ab6f9` = **200 Vitest / 35 fichiers** (+ 8 cas e2e). Le « −7/−8 » du plan était le delta de **jettison** au Phase B (`37daa09`, ~7-8 tests perdus dont `executionChat.test.ts`), PAS un total de baseline ; la suite neuve a été rebâtie depuis 0 (C 52 → D 77 → E 92 → H 137 → **200**). Ancrer le plancher anti-régression sur **200/35/8**.

---

## 3. QUESTION TRANCHÉE — echo vs stub (design retenu + gating prod)

> Question ouverte du plan (`sprint80_plan.md:249-250` « cible `ExecutionTarget` echo/fixture … ou stub HTTP/SSE — à figer au preflight »). **Tranchée ici. Aucun conflit Day-0.**

### 3.1 Design RETENU

**Fixture sur le VRAI dispatch via les overrides d'environnement DÉJÀ existants** — de préférence le bras **Network** (`SBFB_DAEMON_ENDPOINT`, `provider_router.rs:282-286`) pointé sur un fixture loopback minimal renvoyant **un seul `Done`, zéro `Delta`** (= exactement l'invariant PO-14, miroir du mock `spawn_mock_daemon` `provider_router.rs:731-784`) ; OU le bras **Ollama** (`SBFB_OLLAMA_ENDPOINT`, `:155`) sur un fixture NDJSON `delta+delta+Done` (miroir `:630-693`) si l'on veut aussi exercer le streaming token-par-token.

Le SSE déterministe traverse **le vrai `handle_chat_stream → ExecutionTarget::from_provider → run`**, donc le vrai parsing + la vraie émission SSE que le front consomme. **ZÉRO nouveau code prod, ZÉRO nouveau provider, ZÉRO nouvelle dep.**

### 3.2 Options rejetées (justification threat-model)

- **Stub HTTP/SSE JS (msw/nock)** — REJETÉ. (a) **Mauvaise couche** : l'E2E lance le VRAI serveur Rust (`serve-operator.mjs:35-36`) ; un mock `fetch` JS n'intercepte que le fetch du navigateur, jamais le SSE émis par `operator_server` Rust → ne résout PAS le déterminisme E2E (S1b-F3 CONFIRMED). (b) **Non installé** : msw = peer OPTIONNEL, absent du tree (`package-lock.json:3311-3317`) ; l'adopter tirerait ~19 deps directes → interdit (D2/sobriété).
- **`#[cfg(test)]`** — REJETÉ : invisible au binaire réel spawné en cross-process (`provider_router.rs:502` inaccessible au `cargo run`/`SBFB_FACTORY_BIN`).
- **Nouveau provider-string nu `"echo"` dans `from_provider`** — REJETÉ : embarqué en prod ET atteignable par tout appelant loopback qui envoie `provider:"echo"`. **Footgun aggravant** : `from_provider` inconnu→Claude (`:89-92`), donc un « echo » désactivé **spawnerait le vrai CLI claude** (non-déterministe / hang e2e).

### 3.3 Gating prod (justifié threat-model — S3-F1 CONFIRMED)

Le design retenu (§3.1) **n'ajoute rien au binaire livré** : les `SBFB_*_ENDPOINT` existent déjà, défaut = loopback ; le déterminisme vient **entièrement de l'injection d'env au test** (`serve-operator.mjs` / `playwright.config.ts`). C'est strictement plus sûr que tout chemin echo embarqué.

**Si** l'implémentation exige malgré tout un bras dédié (`ExecutionTarget::Echo`) pour une clarté single-`Done` sans écrire de fixture-serveur : le gater par une **feature Cargo `operator-test-echo`** (exclusion **compile-time** du binaire prod ; la harness build `--features operator-test-echo`, le prebuild CI `SBFB_FACTORY_BIN` idem). Repli le plus faible = env-var runtime (embarque du code mort) — à éviter.

**Contrainte cardinale, quelle que soit l'option** : la cible n'est atteinte QUE via `handle_chat_stream → from_provider → run`, **après** le gate MUR (`operator_server.rs:1644-1658`, dispatch `:1684-1685`) → aucun bypass MUR ; **jamais** de route SSE parallèle ; **jamais** exposée par `handle_providers`/`PROVIDERS` (`process.rs:41` ; axe prompt-adaptation distinct). API loopback interne = pré-launch éditable → **pas de DESIGN-CONFLICT**.

**Recommandation par défaut** : commencer par l'option §3.1 (fixture sur bras Network via `SBFB_DAEMON_ENDPOINT`) — c'est le chemin 0-code-prod, single-`Done`-natif (PO-14), et il ferme du même geste le vecteur « `running.json` résiduel » puisqu'on n'utilise plus la découverte disque (cf. S3-F3).

---

## 4. Cartographie « déjà couvert vs à écrire »

### 4.1 DÉJÀ COUVERT — NE PAS ré-écrire (SCOPE-CUT-CONSISTENT)

| Élément | Où | Preuve |
|---|---|---|
| Re-couverture SSE single-Done (`useTokenStream`) — 9 cas | `src/lib/useTokenStream.test.ts:46-165` | single-Done delta (46-60), Network 0-delta PO-14 (62-69), latch post-Done (71-85), requires_gate (87-93), abort honnête (95-106), supersede one-live (108-128), transport 502 (130-136), error-first-terminal (138-150), ended-EOF (152-165). Commit `6991d51` (Phase C). Fetch+ReadableStream, 0 EventSource. |
| Sous-test T1 (1) boot cookie HttpOnly | `e2e/boot.spec.ts:27-69` | cookie boot + CSP header + reload cookie-only + 303/HttpOnly |
| Sous-test T1 (2) composeur→session | `e2e/steer.spec.ts:18-33` | `POST /api/chat/session` |
| Sous-test T1 (4) MUR `requires_gate` sans spawn | `e2e/steer.spec.ts:35-57` | `requires_gate:true`, mur visible, atelier count 0, `streamOpened false` |
| Sous-test T1 (5) diff-viewer + gates 1:1 + ÉTAT jamais PASS | `e2e/verify.spec.ts:17-43` (+ `:45-72` procédé verdict restitué + diff bi-usage V2) | `\d+%` count 0, gates 1:1, slot ÉTAT ≠ PASS |
| T1-étendu : inspecteur context-pack (hash) | `src/**/ContextPackInspector.test.tsx` présent | à VÉRIFIER l'assertion hash |
| T1-étendu : refus MUR + raison | `src/**/Mur.test.tsx` présent | à VÉRIFIER l'assertion raison |
| 6 gates front | `tools/factory-operator/scripts/` | no-radix, no-tw-config, scan-front-discipline, i18n-verdict, i18n-parity, accessibility-system (chaînés `package.json:24`) |
| Garde « jamais l'id Claude » + trim modèle | **côté Rust** `operator_server.rs:1867-1894` | picker modèle client coupé (`useOperator.ts` n'envoie plus de `model`) ; l'invariant vit et est testé serveur → rien à re-couvrir côté client |

### 4.2 À ÉCRIRE — le vrai travail Phase I

1. **Sous-test T1 (3) full-stack SSE déterministe** (`e2e/steer.spec.ts`, carry explicite `:5-9`) : token→`Done`, **un seul `Done`** (PO-14), via le mécanisme §3 (fixture sur bras Network/Ollama). NEUF.
2. **Mécanisme SSE déterministe** (§3) : fixture loopback + wiring `serve-operator.mjs`/`playwright.config.ts` (défaut) ou bras `Echo` sous feature Cargo (si exigé). NEUF.
3. **Scan anti-score/jauge STATIQUE BLOQUANT** : étendre `scan-front-discipline.sh` (aujourd'hui `FORBIDDEN` = verdicts seulement, `:23` ; extension anti-score auto-déclarée « in Phase I » `:14-15`) ou nouveau gate ; interdire « % santé »/trust-score/jauge sur `.tsx`/`.ts` **et** catalogues `.po` (défense en profondeur) ; preuve anti-vacuous (inject+restore). **Note** : l'axe anti-PASS/verdict sur `.po` est DÉJÀ couvert par `check-i18n-verdict-cross-locale.sh:35` → seul l'axe **score** est neuf. Câbler dans `npm run gates` (déjà en CI `ci.yml:176`). NEUF.
4. **Artefact T2 JSON COMMITTÉ** (aucun n'existe ; P3-6 routé S80 `kickoff:224`) : harness reproductible, `PASS` déterministe / `BLOCK{diagnosis}`, `RIG-ABSENT` illégitime (Operator 100 % loopback 127.0.0.1). Suivre le patron canonique `scripts/acceptance/app_authoring_capability.sh:90-98` (allowlist `json.dumps({status, checks…})`). **Allowlist stricte** : jamais l'environnement, les en-têtes de réponse (cookie `sbfb_operator` = secret de session per-boot), ni `SBFB_HOME` absolu (porte le username). NEUF.
5. **Gating CI Vitest** : AJOUTER une étape `npm run test:unit` au job `factory-operator` de `.github/workflows/ci.yml:169-188` **ET** `.woodpecker/ci-linux.yml` (steps 93-121) — absente aujourd'hui ; retirer/justifier `--passWithNoTests` (`package.json:11`). Sinon la re-couverture PO-14 n'est enforced nulle part. NEUF (P1).
6. **Isolation hermétique** (ferme `TEST-ISOLATION-SBFB-HOME`) :
   - (a) **Découpler root-de-données et root-de-bundle** : `repo_root()` (`process.rs:56-67`) n'a AUCUN override et le bundle se résout sur ce même root (`operator_server.rs:232,236`). Ajouter un override explicite (ex. `SBFB_FACTORY_ROOT` lu par `repo_root_pub`) **OU** une fixture qui embarque/symlink le bundle bâti. **Attention 2 chemins de résolution** : `handle_git_diff`/`handle_gates` lisent `state.root`, mais `sprint-history` (`commit_diff_data`/`get_sprint_history`) tourne sur le **cwd** du process — les deux doivent viser la fixture. Sans ça diff/gates/sprint-history tournent sur le VRAI dépôt (incl. le travail Phase I en cours) → T1 non-hermétique, T2 non-déterministe (S3-F2 P1). NEUF.
   - (b) **`SBFB_HOME` per-run** : `playwright.config.ts:36` = chemin FIXE `os.tmpdir()/sbfb-operator-e2e` → `mkdtemp` unique + cleanup `afterAll` (0700 Unix) (S2-F8/S3-F3 P2). NEUF.
7. **Comptabilité delta honnête** : acter la baseline RÉELLE **200 Vitest / 35 fichiers / 8 e2e** comme plancher anti-régression (pas le « −7/−8 » du plan) ; filet = gating CI (#5). Documenter la perte assumée des Vitest ancien factory-operator + factory-ui comme delta de jettison Phase B (`37daa09`).
8. **(optionnel, 0 dep)** combler le gap S1b-F6 : assertion explicite « `fetch` appelé une seule fois après un `Done` » (0 auto-reconnect) dans `useTokenStream.test.ts` via `vi.fn().mock.calls.length`. NEUF (mineur).

---

## 5. Baseline delta tests — honnête

| Mesure | Valeur à `03ab6f9` | Source |
|---|---|---|
| Vitest (cas) | **200** (passing) | `npm run test:unit` mesuré |
| Vitest (fichiers) | **35** `*.test.*` | idem |
| Playwright e2e (cas) | **8** (boot 2, steer 2, verify 2, documents 1, motion 1) | `e2e/*.spec.ts` |
| `npm audit` | 0 vuln (prod + complet) | S1b-F2 |

- **Trajectoire** : suite neuve rebâtie depuis 0 au jettison Phase B (C 52 → D 77 → E 92 → H 137 → **200**). Le « −7/−8 » du plan = delta de jettison (`executionChat.test.ts` + 1 autre), déjà absorbé.
- **Plancher anti-régression Phase I** = 200/35/8. La Phase I AJOUTE : (3) e2e full-stack SSE (+1 e2e), scan anti-score (+1 gate + son test de preuve), T2 harness, éventuel test « fetch-once-après-Done ». Le total ne doit PAS descendre → filet CI Vitest (#5, aujourd'hui absent).
- **Versions réelles à ancrer au wrap-up J** (pas celles du plan) : vitest `4.1.9`, `@playwright/test` `1.61.1`, `@vitest/coverage-v8` `4.1.9`, `size-limit` `12.1.0`, `jsdom` `29.1.1` (montées via ranges `^`, lockfile committé déjà cohérent — S1b-F4).

---

## 6. Adaptations PLAN-ADAPT (numérotées, evidence)

> Aucune ne touche une Day-0. Les P1/P2 CONFIRMÉS sont tous adressés.

1. **Ne PAS ré-écrire la re-couverture SSE single-Done** — elle est LIVRÉE Phase C (`useTokenStream.test.ts`, 9 cas, `6991d51`). Reframe : **auditer** sa complétude contre l'intention de l'ancien `executionChat.test.ts` et ne combler que le trou résiduel (assertion « fetch appelé une seule fois après Done » = 0 auto-reconnect, `vi.fn().mock.calls.length`, 0 dep). *Evidence : S2-F1 CONFIRMED, S1b-F6, `useTokenStream.ts:134` (fetch, jamais EventSource).*
2. **T1 « 5 sous-tests » ~80 % déjà livré** (1/2/4/5 via `boot`/`steer`/`verify` specs). Phase I = ajouter le **seul** sous-test (3) full-stack SSE + VÉRIFIER les assertions T1-étendu (hash context-pack dans `ContextPackInspector.test.tsx` ; refus MUR + raison dans `Mur.test.tsx`). *Evidence : S2-F7 CONFIRMED, `verify.spec.ts:17-72`, `steer.spec.ts:5-9,18-57`, `boot.spec.ts:27-69`.*
3. **Echo-vs-stub TRANCHÉ (§3)** : fixture sur le VRAI dispatch via `SBFB_DAEMON_ENDPOINT`/`SBFB_OLLAMA_ENDPOINT` existants (0 code prod, single-`Done` PO-14). PAS de stub JS (mauvaise couche + non installé), PAS de provider-string nu (footgun inconnu→Claude), PAS de `#[cfg(test)]` (invisible au binaire réel) ; feature Cargo `operator-test-echo` seulement si un bras dédié est exigé. *Evidence : S1b-F3, S2-F3, S3-F1 (tous CONFIRMED) ; `provider_router.rs:63-72,89-92,155,282-286,630-693,731-784` ; `serve-operator.mjs:35-36`.*
4. **Isolation hermétique = 2 vrais bloqueurs à corriger** : (a) `repo_root()` sans override + bundle couplé au même root → découpler (`SBFB_FACTORY_ROOT` ou fixture embarquant le bundle) + workspace git FIXTURE pour diff/gates/sprint-history ; attention aux 2 chemins de résolution (`state.root` vs cwd pour sprint-history). (b) `SBFB_HOME` chemin fixe → `mkdtemp` per-run + cleanup. *Evidence : S3-F2 CONFIRMED (P1) ; S2-F8/S3-F3 (P2) ; `process.rs:56-67`, `operator_server.rs:232,236,1844-1857`, `playwright.config.ts:36`.*
5. **CI : AJOUTER `npm run test:unit`** au job `factory-operator` (GHA `ci.yml:169-188` **et** Woodpecker `ci-linux.yml:93-121`) — absent aujourd'hui, donc la re-couverture PO-14 et les 200 cas ne sont gatés à aucun push ; retirer/justifier `--passWithNoTests`. *Evidence : S2-F2 CONFIRMED (P1), `package.json:11`.*
6. **Scan anti-score/jauge STATIQUE BLOQUANT** = livrable réel : étendre `scan-front-discipline.sh` (verdicts seulement `:23` ; anti-score déclaré « in Phase I » `:14-15`) ou nouveau gate ; couvrir `.tsx`/`.ts` **et** `.po` ; anti-vacuous inject+restore. L'axe anti-PASS/verdict `.po` étant déjà couvert (`check-i18n-verdict-cross-locale.sh:35`), le NEUF = l'axe **score** (% santé/trust-score/jauge). *Evidence : S2-F4 CONFIRMED (P1), S3-F5 (P3 ADJUSTED).*
7. **Artefact T2 JSON COMMITTÉ** = livrable réel (aucun n'existe ; P3-6 routé S80) : harness reproductible `PASS`/`BLOCK{diagnosis}`, `RIG-ABSENT` illégitime, **allowlist de champs** (statut + noms/compteurs de checks), JAMAIS env/en-têtes/cookie `sbfb_operator`/`SBFB_HOME` absolu ; suivre `app_authoring_capability.sh:90-98`. *Evidence : S2-F5 CONFIRMED (P1), S3-F4 (P3 ADJUSTED), `kickoff:224`.*
8. **Comptabilité delta ancrée sur le RÉEL** : 200 Vitest / 35 fichiers / 8 e2e (pas −7/−8) ; plancher anti-régression = ce compte, filet = CI Vitest (#5). *Evidence : S2-F6 CONFIRMED (chiffres corrigés 200/35 en main-thread).*
9. **Picker modèle client (P2-OLLAMA-MODEL-PICKER)** : perte ASSUMÉE — le picker client a été coupé (`useOperator.ts` n'envoie plus de `model`), l'invariant « jamais l'id Claude » + trim vit et est testé côté Rust (`operator_server.rs:1867-1894`). Documenter comme réduction de périmètre assumée dans le delta honnête ; rien à re-couvrir côté client. *Evidence : S2-F9 CONFIRMED (P3).*

**Hygiène (info, non-bloquant, wrap-up J)** : lockfile committé reflète déjà les versions installées (S1b-F4) ; paquets WASM `extraneous` bénins, `npm ci`/`npm ls` verts (S1b-F5) — aucune remédiation requise.

---

## 7. Scope (confirmation des cuts)

**DANS le scope S80 Phase I** :
- Sous-test T1 (3) full-stack SSE déterministe (un seul `Done`, PO-14) + mécanisme fixture §3.
- Scan anti-score/jauge statique BLOQUANT (`.tsx`/`.ts`/`.po`, axe score, anti-vacuous).
- Artefact T2 JSON committé (`PASS`/`BLOCK{diagnosis}`, allowlist, `RIG-ABSENT` illégitime).
- Gating CI Vitest (GHA + Woodpecker) + retrait/justif `--passWithNoTests`.
- Isolation hermétique : `repo_root` override + workspace git fixture ; `SBFB_HOME` per-run.
- Vérification des assertions T1-étendu déjà présentes (context-pack hash, MUR refus+raison) + éventuel gap « fetch-once-après-Done ».
- Comptabilité delta honnête ancrée 200/35/8.

**HORS scope (cut cohérent / déjà livré / caduc)** :
- Re-écriture de `useTokenStream.test.ts` → **déjà livré Phase C** (audit seulement).
- Sous-tests T1 (1)(2)(4)(5) → **déjà livrés** (boot/steer/verify specs).
- Re-couverture client du picker modèle → **caduc** (picker client coupé ; garde côté Rust).
- Stub JS SSE (msw/nock) → **rejeté** (mauvaise couche + non installé, §3).
- Onglets « Aperçu scellé » / « Preuve » → restent **désactivés S81** (hérité Phase H, hors Phase I).
- Toute modif du wire P2P → **N/A** (API loopback interne, pré-launch éditable).

---

## 8. Risques résiduels / cibles à re-vérifier EN Phase I

1. **Determinisme fixture** : vérifier que le fixture `SBFB_DAEMON_ENDPOINT`/`SBFB_OLLAMA_ENDPOINT` émet bien **un seul `Done`** et traverse le vrai `from_provider→run` (pas un court-circuit) ; asserter `Done` unique côté e2e (miroir de l'invariant `useTokenStream.test.ts:62-69`).
2. **Gate substring MUR** : le prompt BÉNIN du sous-test (3) doit éviter les sous-chaînes `shell`/`commit`/`push`/`pass` (ex. « passenger », « marshall ») sous peine d'être faussement gaté (`SENSITIVE_ACTIONS` = match substring, `operator_server.rs:37,1644-1658`). Le sous-test (4) repose au contraire sur ce même substring.
3. **Découplage root** : après ajout de `SBFB_FACTORY_ROOT` (ou fixture-bundle), re-vérifier que **les 3** routes (diff, gates, sprint-history) visent la fixture — sprint-history passe par le cwd, pas `state.root` (S3-F2). Re-builder + re-lancer le T1 pour confirmer 0 fuite du working-tree réel.
4. **Anti-vacuous du scan anti-score** : prouver que le gate échoue si l'on injecte « 87 % santé » dans un `.tsx` ET dans un `.po`, puis restore (comme les gates i18n existants).
5. **T2 allowlist** : re-lire le JSON généré pour confirmer 0 secret/chemin absolu/en-tête ; le committer non-gitignored (P3-6).
6. **CI vert** : après ajout `test:unit`, confirmer GHA **et** Woodpecker verts (pas seulement GHA) ; le total Vitest gaté = 200+.
7. **`SBFB_HOME` mkdtemp** : confirmer cleanup `afterAll` (pas de dir résiduel) et absence de collision en re-run/concurrent.

---

## 9. Questions ouvertes PO

**Aucune décision PO requise pour débloquer la Phase I** (pas de DESIGN-CONFLICT ; aucune Day-0 contredite). La seule question ouverte du plan (echo-vs-stub) est **tranchée** en §3 (fixture sur le VRAI dispatch via env overrides existants ; feature Cargo si bras dédié exigé ; jamais stub JS/`#[cfg(test)]`/provider-string nu). Si le PO souhaite explicitement un bras `ExecutionTarget::Echo` visible plutôt que la réutilisation des mock-endpoints, le défaut recommandé reste **§3.1 (fixture Network 0-code-prod)**, avec le repli feature Cargo `operator-test-echo` (compile-time) documenté.

---

## 10. Addendum 2026-07-02 — comblement S1a/S4 (process complet 5 scans)

> Le Workflow initial a livré 4 scans : S1a a échoué techniquement (retry cap
> StructuredOutput, 5 tentatives) et S4 est arrivé en stub (`X1` info « t »).
> Conformément à la directive process-uniforme (5 scans par phase, jamais
> trimmés), les deux ont été rejoués en fan-out avant-plan (2 agents).
> **Verdict inchangé : PLAN-ADAPT** — aucun finding ne modifie les
> adaptations §6 ; les compléments les PRÉCISENT.

### 10.1 S4-bis — trace SSE bout-en-bout + spec fixture (rejoué, complet)

- **Root-cause du stub S4 initial trouvée (S4b-F8)** : la regex de mission
  `_VERSION\s*[:=]\s*[0-9]+` ne matche pas les annotations de type Rust
  (`: u16 = 1`) → 0 hit → agent sans matière. Regex corrigée → **15 constantes
  wire toutes à 1**, aucune touchée par la Phase I. Wire P2P : zéro impact
  confirmé (0 hit `FeedEntry|Announcement|canonical` dans `provider_router.rs`
  et `operator_server.rs`).
- **Contrat SSE** : `StreamChunk` serde = 5 variantes (`llm_bridge.rs:42-59`) +
  `requires_gate` forgé main (`operator_server.rs:1591-1597`) ; frames
  `data: <json compact>\n\n` SANS `event:`/`id:`/heartbeat/sentinelle
  (`:1687-1704`, pas de `.keep_alive`) ; **EOF = signal de fin**. Oracle E2E
  secondaire : `GET /api/chat/{id}/log` → exactement **+1** message assistant
  après le stream (side-effect push à chaque `Done`, `:1690-1699` ; l'invariant
  UN-Done est porté par le provider, S4b-F9).
- **Spec fixture Network** (miroir `spawn_mock_daemon` `provider_router.rs:731-784`) :
  3 routes 200 JSON — `POST */api/v1/tasks/submit` → `{"task":{"task_id":…}}`
  (chemin imbriqué obligatoire) ; `GET */api/v1/tasks/{id}` → `{"status":
  "dispatched"|"completed"}` (`rejected`/`timed_out` → Error) ; `GET */result`
  → `{"result_text":…}`. `Connection: close` par réponse. Env :
  `SBFB_DAEMON_ENDPOINT` + `SBFB_NETWORK_POLL_INTERVAL_MS=20` +
  `SBFB_NETWORK_TIMEOUT_SECS` court (mêmes valeurs que les tests Rust `:795-796`).
  Ne pas valider `Host` (client envoie `127.0.0.1` sans port, S4b-F4) ; ne pas
  exiger `X-SBFB-Token` non vide (défaut `""`, S4b-F5).
- **Spec fixture Ollama** (`POST /api/generate`, NDJSON) — 2 P1 d'écriture :
  **S4b-F1** ollama-rs 0.3.4 parse par chunk HTTP + `filter_map(Result::ok)` →
  un objet fragmenté entre 2 chunks TCP est droppé en silence → **1 write
  flushé = N objets complets** ; **S4b-F2** les 4 champs `model`/`created_at`/
  `response`/`done` sont NON-optionnels sur CHAQUE ligne. Frame finale
  `response:""` sinon Delta parasite avant Done (S4b-F7) ; `total_duration` en
  ns → `duration_ms = ns/1_000_000` ; endpoint = origin nu sans path (S4b-F3) ;
  fin sans `done:true` → Done fallback quand même (un Done toujours émis).
- **Union front précisée** : 6 types d'événements wire (`streamChunk.ts:12-18`)
  vs **7 statuts** `StreamStatus` (`useTokenStream.ts:29`, + `ended`).
  Commentaire front `streamChunk.ts:7` cite `:1063` (décalé, réel `:1591`) —
  P3 doc-drift, à corriger au passage.

### 10.2 S1a-bis — prior-art OSS (rejoué, complet)

- **LibreChat** (PR #13472/#13589, live sur main) : fixture `http.createServer`
  pur Node, port fixe via env avec défaut, readiness `GET /` → 200,
  **`webServer` en TABLEAU** (app + fixture, supporté par notre 1.59) —
  topologie exacte du besoin. Scénarios sélectionnés par **marqueur dans le
  prompt** (`E2E_REPLY:` etc.), jamais par état mutable partagé. Teardown =
  tree-kill Playwright (pas de SIGTERM maison — non fiable win32, S1a2-F5).
- **Playwright interne `TestServer`** : si fixture in-process un jour →
  `closeAllConnections()` avant `close()` (connexions longues).
- **`route.fulfill` inadapté au streaming** (playwright#15353) → le choix
  « vraie fixture loopback + vrai binaire Rust » est le pattern éprouvé,
  pas un pis-aller.
- **Artefacts committés** (Node WPT `test/wpt/status/*.json`, Deno
  `tests/wpt/runner/expectations`, CNCF k8s-conformance) — règles : (a) clés
  stables/triées ; (b) **zéro** timestamp/durée/chemin absolu/pid/port ;
  (c) allowlist de champs (verdict enum + détails nominatifs) ; (d) l'artefact
  committé = **projection** du rapport brut (jamais le rapport runner).
- **Vitest 0-reconnect** : `expect(globalThis.fetch).toHaveBeenCalledTimes(1)`
  après le `waitFor` terminal + flush microtasks ; timers réels (le hook ne
  programme AUCUN timer — pas de fake timers). Dépasse la baseline OSS
  (Vercel AI SDK n'a pas ce test).

### 10.3 Précisions d'implémentation retenues (n'altèrent pas §6)

1. Fixture = `e2e/serve-fixture-daemon.mjs`, port fixe constante
   `FIXTURE_DAEMON_PORT` dans `e2e/fixtures.ts`, `GET /` → 200 readiness,
   `Connection: close`, compteur `GET /__calls` (traduction E2E de PO-14 :
   exactement 1 submit).
2. Câblage = 2e entrée du tableau `webServer` de `playwright.config.ts` ;
   env injectée sur l'entrée Operator (`SBFB_DAEMON_ENDPOINT`,
   `SBFB_OLLAMA_ENDPOINT`, poll/timeout courts) ; ordre de boot indifférent
   (env lue à la requête, `provider_router.rs:282`).
3. Les 2 P1 S4b-F1/S4b-F2 sont des contraintes d'ÉCRITURE de la fixture
   (absorbées dans l'adaptation §6.3), pas de nouveaux livrables.

## Verdict: PLAN-ADAPT
