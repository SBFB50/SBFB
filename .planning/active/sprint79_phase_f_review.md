# Sprint 79 — Phase F review : pack daisyUI knowledge + extension prompt-kind

**Méthode** : Workflow ultracode review (run `wf_d2e78d16-b28`), fan-out 6 dimensions
(correctness · CSP-security · facts-grounding · frontier-gate · tests-semantic ·
scope-livrables) + vérification adversariale. 8 agents, modèle forcé `claude-opus-4-8[1m]`.

## Verdict : PASS

(PASS-PENDING au sortir de la review → promu **PASS** après réconciliation Codex, cf. §Codex.)

**6/6 dimensions PASS, 0 P0/P1/P2 non corrigé.** Tous les findings actionnables traités
in-phase avant commit. Suites §7.4 toutes vertes.

## Dimensions

| Dim | Verdict | Findings |
|---|---|---|
| 1 Correctness (data+Rust) | PASS | 2 P3 cosmétiques (MANIFEST « 5-layer » label vs 7 hashes = parité animejs ; csp_class taxonomie ouverte) |
| 2 CSP-security (cœur) | PASS | 1 **P2 CORRIGÉ** + 1 P3 |
| 3 Facts-grounding vs OSS | PASS | 2 P3 (label 5-layer ; `@source ./` vs `../`) |
| 4 Frontier-gate + cadence | PASS | 5 P3 (tous « aucune action » — diagnostic gate vérifié empiriquement) |
| 5 Tests sémantiques + delta | PASS | 2 P3 (coverage filesystem fail-safe ; artefact temp **SUPPRIMÉ**) |
| 6 Scope + livrables + Day-0 | PASS | 1 P3 (note 5.5.22 pré-existante theming.json) |

## Findings actionnables traités

- **P2 (D2) — drift source-CSP intra-fichier CORRIGÉ** : `app-authoring.md:27` (texte hérité
  Phase A) disait que `BLOB_SERVE_CSP` « lives in `blob_serve.rs` » alors que Phase E a déplacé
  la définition vers `crates/nexus-core-rs/src/csp.rs` (`blob_serve.rs:284` n'est plus qu'un
  re-export). Le fichier se contredisait avec les éditions Phase F (section daisyUI + synthesis).
  → Corrigé : pointe désormais « defined in `csp.rs`, re-exported by `blob_serve.rs`, mirror
  `csp-contract.json` ». Aligne le fichier sur D4. (test markers re-vert.)
- **P3 (D5) — artefact temp SUPPRIMÉ** : `scripts/.gate-buggy-tmp.sh` (repro du bug gate par un
  agent de review) — vérifié absent du working tree avant commit (nettoyé). `git status` propre.

## Findings non-bloquants documentés (P3, pas d'action Phase F)

- MANIFEST `method:"5-layer"` vs 7 hashes : **parité exacte avec le pack animejs** (method=9 hashes
  / "5-layer") — convention établie, le test garde la couverture exacte des fichiers présents, pas
  la string. Ne pas diverger.
- `csp_class` (classes-bank.json) = domaine ouvert (4 valeurs) non gardé par enum/test : artefact
  dev-only lu par LLM, non parsé par un gate ; les 4 valeurs sont sémantiquement justes bloc par
  bloc. Garde optionnelle future (pattern `feedback_named_constants`).
- `@source "./index.html"` (prompt) vs `"../index.html"` (showcase `src/input.css`) : les deux
  valides (relatif à l'emplacement de input.css) ; le prompt décrit un template lean générique.
  À aligner quand les templates daisyui seront écrits (**Phase G**).
- `@plugin "daisyui";` (recette) vs `@plugin "daisyui" {}` (lean) : deux formes valides v4,
  contextes distincts. Non bloquant.
- `theming.json:2` note « properties.css 5.5.22 / pack 5.5.23 » : **pré-existante** (hors diff F),
  note de provenance honnête ; le pin canonique 5.5.23 est testé + cohérent partout.

## Vérifications adversariales (CONFIRMED)

- **Fix gate `grep -hoE` = root-cause, toujours mordant** : reproduction live de la version pré-fix
  (échoue avec 2 packs, flagge même les hashes animejs valides) ; le `-h` restaure la sémantique
  union cross-pack voulue à l'origine (97f7720), ne l'affaiblit pas ; bite-test (hash bidon
  `deadbeefdeadbeef` injecté → gate FAILED exit 1) ; seul `find -exec grep` multi-fichiers du
  script ; `-h` portable GNU+BSD. La chaîne prompt→MANIFEST→octets est intègre (test hermétique).
- **backdrop-filter + @apply = CSP-safe CONFIRMED** : aucun contre-exemple de fetch réseau sous
  `connect-src 'none'` ; backdrop-filter = composite GPU ne traversant pas l'origine opaque ;
  @apply = compile-time absent au runtime.
- **Faits corrigés grounded** : « 35 thèmes » prouvé (theming.json `builtin_themes` len=35 + 35
  fichiers `node_modules/daisyui/theme/`) ; « `fill-*`/`stroke-*` se compilent en CSS statique »
  prouvé par **compilation réelle** (`.fill-primary{fill:var(--color-primary)}`) ; aucun « 8 »
  résiduel dans le pack/prompt (seulement dans `.planning/` = la source du faux que F corrige).

## Suites §7.4 (toutes vertes, AVANT commit)

- Rust : `fmt --all --check` ✓ · `clippy --workspace --all-targets -D warnings` ✓ ·
  `nextest --workspace` **1983** (1981→1983 = **+2** exact) ✓ · doctest ✓ ·
  `build -p nexus-shell-daemon --release` ✓
- Frontend (non-régression, `web/` intact) : lint ✓ (0 error) · tsc ✓ · unit **411** ✓ · build ✓
  · size 6/6 ✓ · scan-en-strings ✓
- Gate `check-frontier-contracts.sh` ✓ (clean, après fix root-cause)
- Tests ciblés : `daisyui_manifest` ✓ · `app_authoring_prompt_surfaces_daisyui_markers` ✓ ·
  `animejs_manifest` ✓ (non-régression pack Phase A)

## Day-0 (tenu)

D1 (`provenance.rs` INCHANGÉ, knowledge dev-only) · D4 (source CSP unique `BLOB_SERVE_CSP`,
re-ancrée) · D7 (snapshot 5.5.23/4.3.1/4.5.0, sbfb-reflect, lean) · D9 (0 bump wire, 0 dép
runtime). Aucune rouverte. PLAN-ADAPT du preflight honoré.

## Codex reconciliation

**Codex GPT 5.5 (`codex exec`, output brut `sprint79_phase_f_codex_review.md`).**

- **Run 1** : 8 livrables → **6 CONFIRME, 0 GAP, 2 PARTIEL**.
  - PARTIEL L5 (corrections in-pack incomplètes) : `COMPONENTS.md` (mirror markdown, jamais édité au
    git mv) gardait « ne compilent pas » en 7 endroits ; `components.json:229` (champ `reason`) +
    `synthesis.json:66/76` idem. La correction A2 n'était appliquée qu'à `components.json` (5 `risks`)
    + README + `synthesis` csp_ruleset[0].
  - PARTIEL L6 (extension prompt) : `app-authoring.md:68-70` (pitfall #3, section anime) disait encore
    « do not compile inside the sealed iframe » → se contredisait avec la nouvelle section daisyUI.
- **Fix de complétude (root-cause, in-phase)** : alignement de TOUTES les occurrences sur la formulation
  correcte (« se compilent en CSS statique mais purgeables si non vus par `@source` + pas de build
  Tailwind au runtime ») — `COMPONENTS.md` ×7, `components.json` ×2 (reason + 1762), `synthesis.json`
  ×2 (66/76), `app-authoring.md` pitfall #3. Cascade hash : `components.json`/`COMPONENTS.md`/
  `synthesis.json` étant des couches hashées, recompute blake3[..16] → MANIFEST mis à jour (3 hashes)
  → citations du prompt mises à jour (gate 4a prompt↔MANIFEST re-vert).
- **Run 2 (BOUCLE COMPLETE)** : re-suites (`fmt`/`clippy`/`nextest` **1983**) + re-Codex →
  **8 CONFIRME, 0 GAP, 0 PARTIEL**. Codex re-run a relancé `cargo test` daisyui_manifest +
  app_authoring markers + animejs_manifest (non-régression) = verts.

Aucun GAP P0/P1 résiduel. Verdict final **PASS**.
