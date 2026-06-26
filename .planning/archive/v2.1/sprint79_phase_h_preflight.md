# Sprint 79 — Phase H preflight (G8)

**Phase** : H — Self-check runtime viewer + confirmation CSP daemon + testabilité (T1/T2)
**Méthode** : Workflow ultracode `wf_ca421f86-7ca` — fan-out 5 scans (S1a OSS / S1b deps / S2 historique / S3 threat / S4 wire) + 5 vérifications adversariales + synthèse. 11 agents `claude-opus-4-8[1m]`, 1.1M tokens.

## Verdict: PLAN-ADAPT

**Rationale** : consensus 4/5 scans PLAN-ADAPT ; le seul EXECUTE (S1a) est rétrogradé par les 5 vérifications
adversariales convergentes. Le plan H est **sous-spécifié sur ~6 mécanismes load-bearing** dont un choix naïf
produirait un **test menteur (faux-vert)** ou une **violation Day-0**. AUCUNE décision Day-0 gelée n'est violée par
le plan tel qu'écrit — tous les « design_conflicts » des scans sont des **risques-à-éviter** (résolus, étayés par le
code), pas des conflits durs. Le tier runtime est un net-new légitime, explicitement forward-référencé dans le code
(`gates.rs:374-376` + `THREAT_MODEL §13.1` nomment « Phase H »). Pas SCOPE-CUT-CONSISTENT global : le delta runtime
est réel.

**Réfutation centrale (vérifiée à la main)** : le template **daisyui** — le template phare de ce sprint
app-authoring, livré Phase G — n'a **AUCUN `sbfb-bridge.js`** (seuls `static/static-reader/react/pyodide`
l'embarquent). Donc la voie « étendre `sbfb-bridge.js` comme reporter » NE couvre PAS le type d'app que ce sprint
existe pour activer. ⇒ le canal d'observation load-bearing doit être **browser-level Playwright**, pas un shim in-app.

## Adaptations vs plan original (toutes étayées, 0 Day-0 touchée)

1. **Canal d'observation T1 = capture browser-level Playwright** (`page.on('console')` / CDP `Log.entryAdded` à
   travers l'iframe opaque), PAS un shim in-app. Une violation CSP runtime émet une erreur console browser-level que
   Playwright capture sans injecter de code ni coopération de l'app. `computeFrame` (`web/e2e/fixtures.ts:48-51`
   frameLocator) perce déjà l'opaque. Dissout le double dilemme drift-d'octets / bon-vouloir.
2. **Test byte-exact NEUF** pour `blob_serve_csp_equals_contract` : nextest dans `http.rs` assertant
   `served CSP header == nexus_core_rs::csp::BLOB_SERVE_CSP` sur **200 ET 404**. Trou réel : `http.rs:7269` (200)
   n'asserte que `.contains("connect-src 'none'")` [substring] ; `http.rs:7533-7536` (T37, 404) n'asserte que
   `.is_some()` [présence]. Le pattern `==` exact existe déjà pour COOP/COEP (`http.rs:7272-7297`) → l'appliquer à
   la string CSP. C'est exactement le « BLOB_SERVE_CSP testé par substring » noté dans `doctrine_contrat_pour_llm`.
3. **Seed hermétique T1 via `/blob-serve/{hash}`, PAS `--web-root`**. Piège false-green : le middleware CSP est
   scopé au nest `/blob-serve` (`http.rs:551-577`), `--web-root` passe par `ServeDir` SANS aucune CSP. Une fixture
   en `web/dist` testerait ZÉRO CSP. `global-setup.ts` sert le shell via `--web-root` avec ZÉRO entrée browse ⇒ T1
   doit SEEDER via `publish-blob` (ZIP→hash) → `publish` (archive_hash→entrée browse) puis piloter Browse →
   BrowsedProject → iframe.
4. **Hôte = contexte first-party** (`web/` shell ou factory-operator), **JAMAIS le Viewer SBFB sandboxé**.
   `BLOB_SERVE_CSP` (`csp.rs:33`) contient `frame-src 'none'` (doc `csp.rs:28-29` « blocks nested iframes ») → une app
   servie sous cette CSP ne peut pas imbriquer l'iframe candidate. Le harnais + T1 + T2 vivent en `web/` (vitest +
   Playwright) car `tools/factory-ui/package.json` n'a AUCUNE infra de test (0 script, 0 devDeps) et n'est importé
   par aucun hôte ; `readonly` ne fournit que la restitution du verdict (`VerdictChip` existe déjà).
5. **Fixture NEGATIVE obligatoire (anti faux-vert)** : T1/T2 doivent inclure une app **DIRTY** qui assemble au
   runtime une violation (`fetch` via `atob` / `url()` / `@font-face` dynamique vs `connect-src`/`default-src 'none'`)
   prouvant que le filet DÉTECTE ce que le lint statique manque, **plus** une app **CLEAN** qui passe.
   `gates.rs:373-378` énumère exactement ces vecteurs runtime hors-portée statique. Un self-check clean-only prouve
   que le harnais tourne, pas qu'il détecte ⇒ gate README §4 vide de sens.
6. **Reporter in-app (si voie Operator/advisory en complément)** : script CLASSIQUE same-origin (les modules ES
   échouent en CORS sous COEP require-corp à origine opaque) chargé EN PREMIER, enregistrant
   `securitypolicyviolation` + `window.onerror` + `unhandledrejection`. Canal = type de message postMessage
   **DISTINCT** (à la `BridgeHeartbeat`), **JAMAIS** une valeur de `BridgeMethodSchema` ; auth nonce-par-rejeu +
   `event.source===iframe` (`event.origin==='null'` sous origine opaque, inexploitable). Reporter servi sur une
   **COPIE**, ne JAMAIS muter le deliverable scellé (`compute_output_hash` / FG6 lock==prov).
7. **Production ZIP des fixtures** : `publish-blob` exige un corps `.zip` (daemon décompresse via crate `zip`),
   Node/Playwright n'a pas d'écrivain zip natif ⇒ **COMMITTER `clean.zip` + `dirty.zip` pré-builds (0-dep)**. NE PAS
   ajouter `jszip`/`fflate` (briserait le verdict 0-dep front).
8. **T2 modèle = `b3_shard_pipeline.sh`** (JSON `{status,stage,diagnosis,...}` + exit 0/1/3 PASS/BLOCK/RIG-ABSENT +
   encoder python3/fallback bash), PAS `phase_h_compute_local.sh` (prose, 0 JSON). PASS hermétique requis (plan:391),
   ATTEIGNABLE (aucun rig multi-machine) ; RIG-ABSENT honnête UNIQUEMENT si pas de Chromium.

## Scope confirmé (execute_scope)

1. Self-check runtime CSP rejouant l'app dans l'iframe-host first-party (`BrowsedProject.tsx:599-609`
   `sandbox=allow-scripts` SANS `allow-same-origin`) sous la CSP RÉELLE du daemon, via `/blob-serve/{hash}`, JAMAIS
   un onglet direct (Day-0 `no_direct_blobserve` respectée).
2. Champ T2 `blob_serve_csp_equals_contract` adossé à un VRAI test byte-exact (servi == const, 200 ET 404).
3. T1 E2E hermétique Playwright BLOQUANT (`web/e2e/app-authoring.spec.ts`, untagged ⇒ `test:e2e --grep-invert
   @compute` + GHA `ci.yml` step 10c + `verify.sh` step 15) avec fixtures CLEAN ET DIRTY.
4. T2 `scripts/acceptance/app_authoring_capability.sh` (net-new) modèle `b3_shard_pipeline.sh`, vise PASS hermétique.
5. Docs gate-spécifiques SEULEMENT : `docs/rust/PATTERNS.md` (§ gate CSP-authoring + source CSP unique + note
   self-check runtime) et `docs/factory/FACTORY_GATES.md` (addendum runtime FG-CSP-authoring). Le reste = Phase I.
6. `blob_serve.rs` reste CONSOMMÉ/confirmé : 0 modif wire, 0 bump `*_ANNOUNCEMENT_VERSION` (pre-launch conforme).

## Scope cuts cohérents (ne PAS faire)

1. NE PAS re-asserter l'égalité de CONST gate↔daemon : déjà garantie compile-time (source unique → re-export →
   import). Le delta de H = confirmation RUNTIME de l'en-tête servie.
2. NE PAS dupliquer les tests Rust de PRÉSENCE existants (`http.rs:7533-7548` T37 404 + `7281-7297` COOP/COEP) ; H
   ajoute l'égalité byte-EXACT de la string CSP, qui manque.
3. `FACTORY_GATES.md` porte DÉJÀ FG-CSP-authoring depuis Phase E (`:136-137` forward-ref « Phase H ») ; H AJOUTE la
   note runtime, ne recrée pas la section.
4. Wrap-up complet (Diataxis, llms.txt, WIRING_SPEC, check-factory-docs.sh, SPRINT_LOG, CLAUDE.md, MEMORY,
   audit_plan) → Phase I. H borné à self-check + T1/T2 + 2 docs gate.
5. Boucle d'auto-repair = COPILOTE (Phase G livré) ; H = capture + verdict + champ T2 uniquement.
6. Le self-check ne devient PAS une autorité de publish : `status=PASS` T2 = verdict de TEST, pas approbation d'app
   (Day-0 « connaissance CONSOMMÉE jamais autoritaire, 0 verdict PASS auto »).

## Risques à surveiller

1. FAUX-VERT sans fixture DIRTY (condition de validité du gate BLOQUANT T1).
2. FAUX-VERT web-root : fixture en `--web-root` sert ZÉRO CSP ; DOIT transiter par `/blob-serve/{hash}` seedé.
3. AUTH origine opaque : `event.origin==='null'` ; auth par nonce + `event.source===iframe` spécifique, jamais par
   origine ni whitelist bridge.
4. HASH/scellage daisyui : fixture/reporter DANS le deliverable change le hash (FG6 lock==prov) ; servi-sur-copie
   garde le hash mais teste des octets différents. À trancher AVANT code.
5. `blockedURI` REDACTÉ à origine opaque : diagnostic auteur plus grossier (seuls `effectiveDirective`/
   `violatedDirective` survivent). Documenter comme limite connue.
6. Trous web-platform (filet non-total) : échec COEP-pur ne fire pas `securitypolicyviolation` (mais `connect-src
   'none'` + `default-src` sans hôte distant bloque-et-émet d'abord) ; form submit sous sandbox sans `allow-forms`
   préempté par l'attribut sandbox. Documenter.
7. Modules ES échouent sous COEP require-corp à origine opaque → reporter = script classique chargé en premier.
8. Parité CI Woodpecker : `.woodpecker/ci-linux.yml` ne lance AUCUN Playwright ; T1 tagué CI seulement GHA +
   `verify.sh`. Décider explicitement (GHA-canonique ou ajout étape Woodpecker) — pré-existant, à acter H/I.
9. Surface E2E hermétique inédite : Phase H = 1er E2E hermétique à déployer une app et la rendre dans l'iframe ;
   valider end-to-end le comportement OFFLINE du daemon spawné (publish/announce gossip no-op vers 0 pair).
10. Refs de ligne périmées à corriger AVANT le lint docs-contract Phase I : plan cite `blob_serve.rs:286` (réel
    `:284`), `gates.rs:1176` (réel `:1174`) ; injection runtime à `http.rs:556`.

## Ordre d'implémentation concret

1. Corriger les source-refs périmées dans plan/docs (`blob_serve.rs:284`, `gates.rs:1174`, `http.rs:556`).
2. Ajouter le nextest byte-exact dans `crates/nexus-shell-daemon/src/http.rs` : served CSP == `BLOB_SERVE_CSP` sur
   200 ET 404 (miroir du pattern `==` exact COOP/COEP `7272-7297`). Unique preuve machine du champ T2.
3. Committer `clean.zip` + `dirty.zip` (0-dep) : dirty assemble au runtime une violation ; clean conforme.
4. Étendre `web/tests/global-setup.ts` (ou le spec) pour SEEDER les deux fixtures via `publish-blob` → `publish`
   (entrées browse avec archive_hash).
5. Écrire `web/e2e/app-authoring.spec.ts` (T1, untagged) : piloter Browse → BrowsedProject iframe (`computeFrame`)
   pour chaque fixture sous la VRAIE CSP ; capturer violations via `page.on('console')`/CDP (browser-level, sans
   shim) ; asserter clean=0 / dirty≥1 ; asserter `response.headers` CSP == contrat.
6. (Surface produit optionnelle) Restituer le verdict dans `tools/factory-ui/src/readonly` (reuse `VerdictChip`) +
   câblage advisory Operator si retenu ; tester la logique parse/verdict en factory-operator vitest UNIQUEMENT
   (jsdom n'enforce pas la CSP) ; la preuve d'enforcement reste T1 Playwright.
7. Écrire `scripts/acceptance/app_authoring_capability.sh` (T2) clone de `b3_shard_pipeline.sh` : JSON
   `{status,stage,diagnosis,blob_serve_csp_equals_contract,...}`, exit 0/1/3, encoder python3/fallback bash ; PASS
   requiert égalité byte-exact CSP (probe live daemon spawné, 404 suffit) ET détection dirty + clean-propre de T1 ;
   viser PASS hermétique.
8. Ajouter UNIQUEMENT les docs gate-spécifiques (`PATTERNS.md` § gate CSP + `FACTORY_GATES.md` addendum runtime) ;
   différer le reste à Phase I.
9. Confirmer le tag CI de T1 (GHA `ci.yml` step 10c + `verify.sh` step 15) et acter la décision de parité Woodpecker.

## Décision de routage

PLAN-ADAPT avec evidence concrète (file:line vérifiés) + 0 Day-0 touchée ⇒ **EXÉCUTER** : le code suit l'approche
corrigée ci-dessus (pas le plan original). Pas de DESIGN-CONFLICT ⇒ pas d'arbitrage user requis. 1er PLAN-ADAPT de
la phase (pas de signal méta).
