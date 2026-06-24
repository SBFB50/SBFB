# Preflight G8 — Sprint 79 Phase B (canon cadence docs-contrat + gate `check-frontier-contracts.sh`)

> Produit par Workflow ultracode `wf_300f6383-b07` (6 agents Opus 4.8 1M : S1a OSS/prior-art,
> S1b deps/CI, S2 historique+cibles, S3 threat, S4 wire/FRONTIER + synthèse adversariale ;
> ~643K tokens, 163 tool-calls). Chaque fait ancré file:line, vérifié contre le code réel.

## Verdict

**PLAN-ADAPT.**

Le plan Phase B tient dans son intention (canoniser la cadence docs-contrat dans le process Claude Code + livrer un gate générique BLOQUANT câblé CI 3 surfaces + réparer le faux-vert `phase-review-cross-check.yml` + scrubber les commentaires STALE-PHASE-K) et **ne contredit AUCUNE décision Day-0/PO** : l'arbitrage « registre EXPLICITE `// FRONTIER:` opt-in incrémental » (plan §3bis l.481-483,501 ; doctrine §7 Q2) est confirmé cohérent — registre vide au jour 1, donc 0 blocage. Phase B est sans-risque-wire (0 struct touchée, 0 bump `*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`, seules éditions Rust = doc-comments).

MAIS le code réel impose des **adaptations chiffrées et techniques concrètes** par rapport à la prose du plan, toutes ancrées :
1. **Le fix du faux-vert CI est DOUBLEMENT insuffisant** (4 scans le confirment) : la regex morte n'est pas seulement le plafond `[A-F]`, c'est le **préfixe entier** `feat\(sprint[0-9]+\): Phase X` qui ne correspond PAS à la convention réelle `feat(scope): Sprint N Phase X`. Corriger uniquement `[A-F]→[A-Z]+[0-9]?` laisse le gate MORT. Cause-racine double : préfixe + plafond.
2. **L'inventaire STALE-PHASE-K est ~30 commentaires, pas ~6** (plan l.503). La regex anti-promesse du plan (`will (populate|expose|add|read|land)`) en RATE ≥18 (verbes `adds`/`ships`/`implements`/`reach`/`layer`/`supply`/`wire`/`hold` non couverts).
3. **Le grep anti-promesse repo-wide s'auto-FAIL** sur `vendor/` (3 hits llama.cpp) et sur `docs/{rust,shell}/PATTERNS.md` qui DÉCRIVENT l'anti-pattern verbatim (`docs/shell/PATTERNS.md:2315` contient `"lands in Phase K"`) — sans exclusion explicite, le gate bloque au jour 1, y compris sur le §P que Phase B doit elle-même ajouter.
4. **La prémisse D5/doctrine « form-action/base-uri = GAP à ajouter » est PÉRIMÉE** pour `BLOB_SERVE_CSP` : les 4 directives sont DÉJÀ présentes (`blob_serve.rs:286`). Le cross-check méta `.contains("form-action")` est un test de non-régression (déjà vert), pas une fermeture de trou.

Aucun de ces points n'exige d'arbitrage PO (le design tient) ; ce sont des corrections d'évidence à absorber AVANT de coder. D'où **PLAN-ADAPT** et non DESIGN-CONFLICT.

## Scope de la phase (rappel concis du livrable Phase B)

Livrable Phase B = **canon process docs-contrat (DOGFOOD)**, 5 chantiers :
1. **Canonisation dans le process Claude Code** de la cadence docs-contrat (la RÈGLE, pas que la Factory) : éditions de `docs/claude/README.md`, `docs/agent/AGENT_SYSTEM.md`, `docs/rust/PATTERNS.md` (nouveau §P70), `docs/shell/PATTERNS.md` (nouveau T## — prochain ≈ T52).
2. **`scripts/check-frontier-contracts.sh`** (net-new, SEUL fichier créé) : gate générique BLOQUANT, cloné du gabarit BusyBox-safe `scripts/check-sharding-docs.sh`. Volets : (a) anti-promesse STALE source-ref générique ; (b) couverture-étiquette `// FRONTIER:` opt-in incrémentale ; (c) cross-check méta `BLOB_SERVE_CSP.contains("form-action")` (non-régression).
3. **Câblage CI 3 surfaces** : `.github/workflows/ci.yml`, `.woodpecker/ci-linux.yml`, `scripts/verify.sh` (à l'identique de `check-sharding-docs.sh`).
4. **Fix du faux-vert CI** `phase-review-cross-check.yml` (regex doublement morte).
5. **Scrub des commentaires STALE-PHASE-K** existants (doc-comments seuls, 0 ligne exécutable).

Hors-scope Phase B (route ailleurs) : prompt-kind `app-authoring` = Phase C (D2) ; gate CSP runtime/vitrine = Phase E ; couverture-étiquette exhaustive des ~22 familles wire = carry `sprint80_audit_plan.md`.

## Confirmations factuelles (chaque fait avec file:line)

- **0 dépendance Rust/web** : Phase B ne touche aucun `Cargo.toml` (membre) ni `package.json` ; les seules éditions Rust sont des doc-comments STALE. `scripts/check-frontier-contracts.sh` est le seul fichier net-new. (S1b)
- **0 risque wire** : tous les fichiers Rust touchés sont des `///`/`//!` (vérifié `keystore.rs:323`, `pow.rs:46`, `relay_pow_policy.rs:35`, `state.rs:81`, `http.rs:2107-2114`). 0 `*_FORMAT_VERSION`, 0 `*_ANNOUNCEMENT_VERSION` modifié — politique pre-launch respectée trivialement. (S4)
- **Gabarit à cloner vérifié EN ENTIER** `scripts/check-sharding-docs.sh:1-241` : `set -euo pipefail` (l.23), BusyBox-safe documenté (l.4-7 : no `-P`, no `--include`, no `\b`), `anchor_present()` (l.69-74), `require_marker()` (l.88-99), source-ref-check `path:Symbol` rank-1 `crates|docs|web|scripts` (l.156-194), required-anchor allowlist (l.196-212). AUCUN `mapfile`/`readarray`. (S1a/S1b)
- **3 surfaces CI confirmées au file:line** : `ci.yml:117-118` (step `[14] sharding docs check`, `run: bash scripts/check-sharding-docs.sh`, juste après `[13] SPDX check` l.113-114) ; `.woodpecker/ci-linux.yml:74-77` (step `sharding-docs-check`, image `bash:5@sha256:2003051c5eb5154cbd44fd4b1a2b8f1be886517b383813c998c72cb15840357f`) ; `scripts/verify.sh:108-109` (step 19, avant l'echo final l.111-112). (vérifié directement)
- **Faux-vert CI confirmé ET doublement mort** : `phase-review-cross-check.yml:49` `grep -E 'feat\(sprint[0-9]+\): Phase [A-F]'`, et `:74`/`:75` `sed -nE 's/feat\(sprint([0-9]+)\): Phase ([A-F]).*/\1|\2/p'`. Le commentaire de tête (l.5-6) décrit lui-même `feat(sprintNN): Phase X`. Convention réelle git log = `feat(scope): Sprint N Phase X` (ex `feat(factory): Sprint 79 Phase A`, `feat(worker): Sprint 77 Phase F2`). git log -200 : OLD regex = 0 récents / NEW convention = 68-93. (vérifié + S1b/S2/S3/S4)
- **STALE-PHASE-K LIVE confirmé** `http.rs:2107-2114` : docstring `/// ... lands in Phase K, so this always misses and returns None`, au-dessus de `fn live_shard_session(_) -> Option<...> { None }` (l.2115-2117). Phase J/K de S77 sont CLOSES, le live store est un carry S78 — promesse mensongère. **Seam intentionnel** (le `None` est le comportement honnête voulu). `http.rs:2109` (`Phase J ships...`) et `:2120` (`no live store (Phase J)`) sont 2 promesses sœurs sharding S77 closes. (vérifié)
- **`BLOB_SERVE_CSP` contient DÉJÀ les 4 directives** `blob_serve.rs:286` : chaîne complète inclut `frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'`. Consommée à `http.rs:556`. Le cross-check méta `.contains("form-action")` PASSE au jour 1. Le vrai « GAP » D5 est dans `check-csp.mjs` (vitrine) / le commentaire périmé, PAS dans la const Rust. (vérifié + 4 scans)
- **Registre `// FRONTIER:` vide au jour 1** : grep `FRONTIER` sur `crates/`+`web/src/` = 0 hit → couverture-étiquette opt-in incrémentale ne bloque RIEN au démarrage. Confirme la décision PO « registre EXPLICITE opt-in ». (4 scans)
- **`PROMPT_KINDS` = 8 kinds** (`process.rs:7-16`), `app-authoring` ABSENT → ajout = Phase C (D2), pas B. Cohérent avec le graphe §3. (S1a)
- **Truth-Stack a 2 formulations divergentes** : `AGENT_SYSTEM.md §1` (l.19-23, table `Repo files > Planning artifacts > ...`) vs `check-sharding-docs.sh:217` (chaîne exacte `repo files > .planning/active/ > commits > prompts > chat`). Risque de drift si Phase B asserte une forme. (S1a)
- **Prochain numéro pattern** : Rust `§P70` (dernier = `§P69`, `docs/rust/PATTERNS.md`) ; shell prochain `T##` (≈ T52). Pas de collision. (S1a)
- **4e surface CI orthogonale** : `.github/workflows/shellcheck.yml:20` lint TOUS `scripts/**/*.sh` avec `--severity=warning` → `check-frontier-contracts.sh` DOIT être shellcheck warning-clean ou casse cette CI. (S1a)
- **`vendor/` est tracké (913 fichiers)** : `vendor/llama-cpp-sys-2/llama.cpp/{chat-peg-parser.cpp:345, sampling.cpp:271, sampling.cpp:350}` contiennent `will add` → faux-FAIL si grep repo-wide. (S3)
- **docs auto-FAIL confirmé** : `docs/shell/PATTERNS.md:2315` (`"lands in Phase K" — STALE`), `docs/shell/PATTERNS.md:1913` (`Sprint 22+ will add`), `docs/rust/PATTERNS.md:1387` (`Phase B will add`), `docs/rust/PATTERNS.md:1935` (`will land with Phase B`). (vérifié + S1a/S1b/S3)

## Conflits / adaptations

### CONFLIT-1 (BLOQUANT — adaptation obligatoire) — Fix CI `phase-review-cross-check.yml` doublement mort
- **Plan** (l.486-487) : « regex obsolète `[A-F]` → regex `Phase [A-Z]+[0-9]?` ».
- **Réalité** : la regex échoue sur DEUX axes. (a) plafond `[A-F]` rate G+/phases composées ; (b) **préfixe** `feat\(sprint[0-9]+\): Phase X` n'a JAMAIS existé — convention réelle = `feat(scope): Sprint N Phase X` (scope sémantique + « Sprint N » APRÈS le `:`). Corriger seulement `[A-F]→[A-Z]+` laisse le gate MORT (0 match). Les 3 sites (l.49 grep, l.74+l.75 sed) doivent être réécrits.
- **Evidence** : `phase-review-cross-check.yml:49,74,75` vs `feat(factory): Sprint 79 Phase A`.
- **Action** : voir watch-item #4.

### CONFLIT-2 (adaptation obligatoire) — Inventaire STALE ~30, pas ~6 ; regex anti-promesse trop étroite
- **Plan** (l.503, §3bis) : « ~6 commentaires STALE » + regex `lands in Phase [A-Z]|will (populate|expose|add|read|land)`.
- **Réalité** : ~30 commentaires (S2 exhaustif, recoupé par grep élargi). La regex du plan RATE ≥18 (verbes `adds/ships/implements/reach/layer/supply/wire/hold` non couverts). Point dur : le corpus est majoritairement **letter-only** (`Phase C will populate` sans n° de sprint) → un grep ne peut PAS décider close/ouvert mécaniquement sans contexte sprint.
- **Action** : élargir la regex (watch-item #3) + décider sémantique « nettoyer l'existant PUIS interdire les nouvelles promesses » (watch-item #5).

### CONFLIT-3 (adaptation obligatoire) — Grep repo-wide s'auto-FAIL (vendor + docs descriptives)
- **Plan** (l.478-480) : « anti STALE-PHASE-K source-ref GÉNÉRIQUE repo-wide ».
- **Réalité** : repo-wide naïf faux-positive sur `vendor/` (3 hits llama.cpp, tracké 913 fichiers), `docs/{rust,shell}/PATTERNS.md` (4 hits dont `2315 "lands in Phase K"` qui DOCUMENTE l'incident), le §P que Phase B ajoutera, et `local_worker.rs:453` (`will read are correct`, prose runtime). Sans exclusion, auto-FAIL jour 1.
- **Action** : scoper aux fichiers de CODE + exclure `vendor/`/`target/`/`node_modules/` + ne pas grep `docs/` descriptives (watch-items #6/#7).

### ADAPTATION-4 (mineure) — Cross-check CSP est non-régression, pas fermeture de GAP
- **Plan** (l.488-489, D5) : présente `BLOB_SERVE_CSP.contains("form-action")` comme fermeture d'un trou.
- **Réalité** : `blob_serve.rs:286` contient DÉJÀ les 4 directives → le cross-check passe au jour 1. Le vrai durcissement (asserter CHAQUE directive vs substring) relève des tests Rust Phase E/H (`http.rs:7268` ne teste que `connect-src 'none'` substring ; `http.rs:7532` ne teste que la présence-header). En B = méta-assertion non-régression seulement.
- **Action** : formuler honnêtement dans le commit B (watch-item #8).

### ADAPTATION-5 (mineure) — Re-chiffrer le carry « familles wire »
- **Plan/doctrine** : « ~21 familles wire non-schématisées » + « 23 DOMAIN_*_V1 ».
- **Réalité** : 25 DOMAIN_*_V1 réels (24 dans `nexus-core-rs/src` + `DOMAIN_TRACE_EVENT_V1` dans `nexus-trace-core/lib.rs:29`) ; 9 types ont `schema_for!`+snapshot (8 sharding + TaskResponse) ; 22 DOMAIN sans schema (3 chevauchent : ComputeGroup/ShardPlan/RunProof). Aucune métrique ne donne 21.
- **Action** : nommer la métrique exacte dans le carry `sprint80_audit_plan.md` (watch-item #10).

### NOTES (traçabilité, pas conflits de design)
- `verify.sh` n'est PAS un miroir propre de la CI : steps Python 4-8 (`uv run ruff/pytest packages/`) sont morts post-pivot S50-S51 mais subsistent. Câbler quand même par cohérence, mais **valider le nouveau script en l'exécutant DIRECTEMENT** (`bash scripts/check-frontier-contracts.sh`), pas via `verify.sh` complet (échoue step 4).
- `docs/shell/PATTERNS.md:2316` route le scrub STALE comme carry S78 — le ramener en S79 Phase B est cohérent (le gate live l'exige) mais **mettre à jour cette ligne dans le même commit** sinon la doc se contredit.
- Décision FRONTIER opt-in absente du kickoff Day-0 (vient des amendements `ed3d4cb`/`7d0225d`) : ancrer la canonisation à la doctrine §7 Q2 + amendements, pas au kickoff (100% Factory app-authoring).

## Watch-items pour l'implementation

**[WI-1] STALE-PHASE-K — liste COMPLÈTE à corriger (~30, pas ~6). CLASSE-1 (phase/sprint clos → réécrire au passé/présent) :**
- `crates/nexus-shell-daemon/src/http.rs:2109` (`Phase J ships...`), `:2111` (`lands in Phase K`), `:2120` (`no live store (Phase J)`) — sharding S77 J/K clos ; **reformuler en gardant le seam** (`store = carry S78`, PAS supprimer le `None`).
- `crates/nexus-shell-daemon/src/main.rs:23,26,27` — shell S5-S7 clos.
- `crates/nexus-shell-daemon-core/src/state.rs:17,18,81,88,94,215` — shell S5 clos.
- `crates/nexus-shell-daemon-core/src/registry.rs:266` — shell S7 clos.
- `crates/nexus-shell-daemon/src/cli.rs:64,65,94,102,111` — shell S7 clos.
- `crates/nexus-shell-daemon-core/src/lib.rs:33` — shell S5 clos.
- `crates/nexus-shell-daemon/src/runtime.rs:211` — shell S5 clos.
- `crates/nexus-shell-daemon/Cargo.toml:27` — shell S5 clos.
- `crates/nexus-core-rs/src/keystore.rs:323,332` — duress LIVRÉ S74/S75.
- `crates/nexus-core-rs/src/lib.rs:17` (`Sprint 2 will add`), `discovery.rs:21` (`Sprint 4 will add`), `node.rs:198,207` (`Sprint 2 will layer`), `Cargo.toml:15` (`When Sprint 2 adds PyO3` — pivot a SUPPRIMÉ PyO3, jamais).
- `crates/nexus-worker/src/main.rs:186` (`Phase D adds a data_dir` — S20 clos, l.190 dit déjà « now »).
- `crates/nexus-worker-core/src/llm/mod.rs:327`, `schema_bridge.rs:45`, `llama_cpp.rs:43` (S20 descriptifs), `gpu/mod.rs:6` (S3).
- `crates/nexus-launcher/src/token_rotation.rs:26` (S19 carry), `unlock.rs:37` (`Phase B adds rpassword` — statut à vérifier).

**[WI-2] STALE-PHASE-K — CLASSE-2 (sprints pré-pivot jamais joués sous ces numéros → décider exemption-documentée OU réécriture, trancher explicitement) :**
- `crates/nexus-core-rs/src/pow.rs:46` (`S22 ... kudos_score`), `:49` (`S26 post-quantum`).
- `crates/nexus-core-rs/src/relay_pow_policy.rs:35` (`S22 kudos_threshold`).
- `crates/nexus-worker-core/src/gpu/profile.rs:11` (`Sprint 24 Phase D`).
- `crates/nexus-worker-core/src/consent.rs:347` (`inert until Phase D ships`).
- `crates/nexus-worker-core/configs/rate_limit_policy.toml.sample:33` (`S22+ sprint`).
- `crates/nexus-worker/src/tui.rs:452` (`W9.1` — notation worker-sprint, PAS Phase [A-Z], faux-positif si grep large).
- `crates/nexus-core-rs/src/tor_transport.rs:6,124` (`Phase 1/Phase 2`).

**[WI-3] ÉLARGIR la regex anti-promesse** au-delà de `will (populate|expose|add|read|land)` : ajouter `adds|ships|implements|reach(es)?|layer(s)?|supply|supplies|wire(s)?|hold(s)?|will land`. Sinon ≥18 commentaires échappent (token_rotation.rs:26, worker/main.rs:186, consent.rs:347, cli.rs:111, daemon/Cargo.toml:27, gpu/profile.rs:11, gpu/mod.rs:6, llm/mod.rs:327, schema_bridge.rs:45, llama_cpp.rs:43, state.rs:215, runtime.rs:211, daemon/main.rs:26,27, keystore.rs:332, core/Cargo.toml:15, node.rs:198,207, tor_transport.rs:6,124). **Ancrer à `Phase [A-Z]+[0-9]? (will|adds|...)|lands in Phase [A-Z]`** (pas `will add` nu) pour éviter les faux-positifs prose (`local_worker.rs:453`, `tui.rs:452`). BusyBox-safe : pas de `\b`, pas de `-P`, pas de `\s`.

**[WI-4] BLOQUANT — Fix `phase-review-cross-check.yml` aux 3 sites (l.49 grep, l.74+l.75 sed) sur les DEUX axes :**
- Regex grep cible : `feat\([a-z-]+\): Sprint [0-9]+ Phase [A-Z]+[0-9]?` (PAS seulement remplacer `[A-F]→[A-Z]+`).
- sed d'extraction : sprint via `Sprint ([0-9]+)` (espace, pas `sprint([0-9]+)` collé), phase via `Phase ([A-Z]+[0-9]?)`.
- Mettre à jour le commentaire de tête l.5-6 (décrit l'ancien format `feat(sprintNN): Phase X`).
- **Valider** : `git log -200 | grep -cE '<nouvelle regex>'` doit retrouver les ~68 commits (pas 0). Vérifier que `sprint{N}_phase_{X}_review.md` (l.82) gère X multi-lettres (AA/AB) + composées (E1/F2).

**[WI-5] Décider la SÉMANTIQUE du source-ref-check** : le corpus letter-only ne permet pas de décider close/ouvert mécaniquement. Voie réaliste à figer dans `docs/claude/README.md` + le §P70 : (1) NETTOYER les ~30 commentaires (réécriture passé/présent ou suppression de la clause future) ; (2) le gate INTERDIT de NOUVELLES promesses (motif futur ancré). Trancher le sort Classe-2 (WI-2). Trancher avec le PO si le recompte (~30 vs plan ~6) change le périmètre.

**[WI-6] EXCLURE du grep** : `vendor/` (3 hits llama.cpp `sampling.cpp:271,350`, `chat-peg-parser.cpp:345`), `target/`, `node_modules/`. Énumérer via `git ls-files 'crates/**/*.rs' 'web/src/**/*.ts*'` OU `find ... -not -path '*/vendor/*' -not -path '*/target/*' -not -path '*/node_modules/*'`.

**[WI-7] CADRER pour ne PAS s'auto-FAIL sur la doc descriptive** : exclure `docs/{rust,shell}/PATTERNS.md` (`2315 "lands in Phase K"`, `1913`, `1387`, `1935`), la doctrine, et le script lui-même + le §P70 que Phase B ajoute (il citera l'anti-pattern verbatim). Ajouter une allowlist d'exclusion explicite documentée. **Tester le script SUR lui-même + sur le commit B avant de le déclarer vert.**

**[WI-8] Cross-check méta CSP = non-régression, formuler honnêtement** : `BLOB_SERVE_CSP.contains("form-action")` passe DÉJÀ (`blob_serve.rs:286` contient les 4 directives). NE PAS le présenter comme fermeture de GAP dans le commit B. Le vrai durcissement (asserter chaque directive vs substring `http.rs:7268`) appartient à Phase E/H.

**[WI-9] Structure du script clonée de `check-sharding-docs.sh`** : `set -euo pipefail` + `SCRIPT_DIR`/`REPO_ROOT`/`cd` (l.23-27) ; helpers `anchor_present()` (l.69-74, `grep -qF`), `require_marker()` (l.88-93), source-ref-check rank-1 (l.156-194), required-anchor allowlist boucle `for req` + `grep -qx` (l.202-212). `grep -oE`/`-qF` UNIQUEMENT (pas `-P`, pas `--include`, pas `\b`/`\s`), process substitution `< <(...)`, `while IFS= read -r` (pas `mapfile`/`readarray`). Doit tourner sous Windows Git Bash (verify.sh local) ET image Woodpecker `bash:5`. **Shellcheck `--severity=warning` clean** (4e surface `.github/workflows/shellcheck.yml:20` le lint auto). Pas besoin de header SPDX (`check-spdx.sh` ne scanne pas `scripts/*.sh`).

**[WI-10] Re-chiffrer le carry avec métrique nommée** : 25 DOMAIN_*_V1 (24 `nexus-core-rs/src` + `DOMAIN_TRACE_EVENT_V1` `trace-core/lib.rs:29`), 9 types `schema_for!`+snapshot, **22 DOMAIN sans schema** (3 chevauchent ComputeGroup/ShardPlan/RunProof). Écrire « 22 des 25 DOMAIN_*_V1 sans schema généré » dans `sprint80_audit_plan.md`, JAMAIS « ~21 familles » ni « 23 DOMAIN ».

**[WI-11] Câblage CI 3 surfaces — points d'insertion EXACTS (clone du modèle `[14]`/step 19/`sharding-docs-check`) :**
- **(A) `.github/workflows/ci.yml`** : ajouter step `[15]` APRÈS la l.118 (fin du bloc sharding), bloc identique à l.116-118 : `# ── frontier contracts check (Sprint 79 Phase B) ──` / `- name: "[15] frontier contracts check"` / `  run: bash scripts/check-frontier-contracts.sh`.
- **(B) `.woodpecker/ci-linux.yml`** : ajouter step APRÈS la l.77, bloc identique à l.74-77, MÊME digest pinné : `- name: frontier-contracts-check` / `  image: bash:5@sha256:2003051c5eb5154cbd44fd4b1a2b8f1be886517b383813c998c72cb15840357f` / `  commands:` / `    - bash scripts/check-frontier-contracts.sh`. **Ne pas inventer un autre tag/digest.**
- **(C) `scripts/verify.sh`** : insérer ENTRE la l.109 et l'echo final l.111 : `step 20 "bash scripts/check-frontier-contracts.sh"` / `bash scripts/check-frontier-contracts.sh`.
- NB : la Phase I prendra `[16]`/step 21 — laisser Phase B prendre `[15]`/step 20.

**[WI-12] Registre `// FRONTIER:` opt-in incrémental** : 0 annotation existante. La branche couverture-étiquette itère SEULEMENT sur les types ANNOTÉS `// FRONTIER:` — JAMAIS un grep large des struct wire (sinon FAIL massif sur les 22 DOMAIN). FAIL uniquement sur type annoté sans snapshot ni exemption `// FRONTIER-NO-SCHEMA: <raison>`. Documenter le format exact `// FRONTIER: <name> domain=DOMAIN_X_V1 version=X_FORMAT_VERSION` dans le §P70. **Annoter au moins 1 primitive S79** (ex. manifeste CSP) pour que la branche ne soit pas no-op permanente — OU documenter franchement le PROVISIONAL (registre vide aujourd'hui = `grep FRONTIER: = 0`).

**[WI-13] Choisir UNE forme canonique du Truth-Stack** (table `AGENT_SYSTEM.md §1` vs chaîne `check-sharding-docs.sh:217 "repo files > .planning/active/ > commits > prompts > chat"`) avant d'en asserter une dans README/PATTERNS/script — sinon drift doc auto-infligé.

**[WI-14] Discipline patterns + commit** : `§P70` Rust + nouveau `T##` (≈ T52) shell pour « cadence docs-contrat » ; inscrire au registre `§P30..§P69→§P70` ; ajouter l'ancre correspondante à l'allowlist si un doc la cite. Mettre à jour `docs/shell/PATTERNS.md:2316` (route le scrub vers S78) dans le MÊME commit. **Gate dual-platform requis** (Win nextest + Docker `sbfb-ci` rust:1.94, fmt 0 sous les 2 toolchains) car le commit B touche du Rust (scrub doc-comments). Confirmer au review que `git diff` ne montre que `scripts/*.sh` + `.github/workflows/*.yml` + `.woodpecker/*.yml` + `docs/**/*.md` + le scrub `.rs`.

## Carry honnête (~22 familles wire non-schématisées + autres reports assumés)

- **Couverture-étiquette des familles wire non-schématisées = NON faite en B** (registre `// FRONTIER:` vide au jour 1, opt-in incrémental). Chiffre exact : **22 des 25 DOMAIN_*_V1 sans schema généré** (le « ~21 » du plan et le « 23 » de la doctrine sont tous deux FAUX — métrique à nommer). → **carry `sprint80_audit_plan.md`** (route Phase I). NE PAS prétendre « tout est gaté ».
- **Classe-2 STALE (sprints pré-pivot jamais joués)** : si exemption documentée plutôt que réécriture (WI-2), tracer la décision dans le §P70 + carry honnête.
- **Durcissement CSP réel (asserter CHAQUE directive vs substring)** : `http.rs:7268` (`connect-src 'none'` substring) + `http.rs:7532` (présence-header) restent des tests substring → vrai durcissement = Phase E/H, pas B. Carry vers Phase E.
- **Drift Truth-Stack** (2 formulations) : si non unifié en B, carry doc à suivre.
- **`verify.sh` cassé en amont** (steps Python 4-8 morts post-pivot S50-S51) : non purgé en B (hors-scope) ; le câblage step 20 est nominal mais le script complet ne tourne pas E2E localement — dette pré-existante, à noter sans la résoudre en B.
- **Décision FRONTIER opt-in non portée par le kickoff Day-0** : ancrée aux amendements `ed3d4cb`/`7d0225d` + doctrine §7 Q2 — traçabilité assumée, pas un gap de design.
