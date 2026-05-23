**Verdict Livrable 1 : PARTIAL**

Le document couvre bien le protocole Gate 1, mais il n’est pas exécutable tel quel à cause des commandes `sbfb-factory` dans les tests 3 et 4.

**Gaps**

P1 — Commandes Factory incorrectes dans les procédures critiques.
`docs/release/GATE1_TEST_PROTOCOL.md:145` utilise `--template hello-world`, mais la CLI ne déclare que `static` et `static-reader` dans `crates/sbfb-factory/src/template_engine.rs:102-118`.
`docs/release/GATE1_TEST_PROTOCOL.md:147`, `170`, `171` utilisent `validate --path .` / `preview --path .`, alors que `validate` et `preview` prennent un `path` positionnel dans `crates/sbfb-factory/src/main.rs:42-51`.
`docs/release/GATE1_TEST_PROTOCOL.md:148`, `173` utilisent `publish --repo .`, alors que `publish` attend `publish <path> --repo-url <url>` selon `crates/sbfb-factory/src/main.rs:54-61` et `crates/sbfb-factory/src/publish.rs:10-14`.

Correction attendue, par exemple :
`sbfb-factory create --template static --name test-app --output ./test-app`
`sbfb-factory validate .`
`sbfb-factory preview .`
`sbfb-factory publish . --repo-url https://...`

P0 — Aucun.
P2 — Aucun nouveau identifié.
P3 — Items acceptés confirmés : chemin logs hypothétique à `docs/release/GATE1_TEST_PROTOCOL.md:243-250`, placeholders BLAKE3 à `docs/release/GATE1_TEST_PROTOCOL.md:64-68`.

**Checks PASS**

Les 9 critères Gate 1 de la roadmap sont bien mappés : source roadmap à `.planning/roadmap_v4_neutral_protocol_factory_rrv.md:217-225`, tests présents à `docs/release/GATE1_TEST_PROTOCOL.md:90`, `110`, `132`, `158`, `183`, `207`, `227`, `260`, `282`.

Chaque test a Go/No-Go et table étape/action/résultat attendu : exemples exhaustifs visibles sur `docs/release/GATE1_TEST_PROTOCOL.md:94-103`, `114-124`, `136-150`, `162-175`, `187-199`, `211-220`, `231-241`, `264-275`, `286-298`.

Installation tri-plateforme : Windows/macOS/Linux couverts à `docs/release/GATE1_TEST_PROTOCOL.md:30-54`.

Hash intégrité : BLAKE3 + fallback documentés à `docs/release/GATE1_TEST_PROTOCOL.md:58-81`.

Sécurité : recherche `daemon.key` = 0 match dans le livrable. Pas de télémétrie ni infra institutionnelle ; le feedback reste manuel via formulaire et rapport à `docs/release/GATE1_TEST_PROTOCOL.md:306-348`.

Formulaire UAT complet : 9 critères + verdict global à `docs/release/GATE1_TEST_PROTOCOL.md:310-322`, infos testeur à `324-328`, bug report structuré à `336-345`.

Niveau non-technique : globalement suffisant pour installation et parcours manuel, mais le P1 CLI casse précisément les tests non-techniques de deploy/Babel/publish.

Tests non lancés : revue statique docs/code, cohérente avec Phase D docs-only et 0 delta tests attendu. Le blocage est visible sans runtime.
