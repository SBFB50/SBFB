# Sprint 82 — Phase B — Review Workflow

## Verdict: PASS

Promu de PASS-PENDING → PASS après réconciliation du gate Codex externe (voir
`## Codex reconciliation` en fin de doc). Aucun P0/P1 restant. Le diff est fmt /
clippy `--all-targets -D warnings` / nextest workspace 2098 / doctests / release build
VERT (état vérifié main-thread ; 2 tests `dispatch_loop::boot_*` env-flaky sous charge
parallèle, 3/3 solo, NON touchés par ce diff). La review Workflow a rendu PASS-PENDING
(0 P0/P1) ; ses 4 P2 + P3 ont été CORRIGÉS in-phase avant Codex ; les 4 P1 trouvés par
Codex (chemins de faux-vert du harness) ont été fermés en 3 rounds. Résiduels = P2/P3
documentés (commit body). Le détail des findings Workflow originaux (dont la réserve
honnête PASS-PENDING) est conservé ci-dessous.

## Resume (dimensions couvertes + méthode adversariale)

5 dimensions couvertes : `wire-safety-correctness`, `security-hygiene`, `tests-semantic`, `scope-research-deliverables`, `harness-artifact-shell`. Chaque finding a subi une vérification adversariale (champ `verify` : `is_real` + verdict CONFIRMED/REFUTED/DOWNGRADED + sévérité ajustée). Cette synthèse ne compte QUE les findings `is_real && verdict != REFUTED`, puis re-vérifie les items load-bearing directement contre le code réel (Read/Grep, 0 compilation) :

- **Invariants wire** : `git diff` sur `run_metrics.schema.json` / `run_proof.schema.json` (description-only), `shard_plan.rs` (comment-only), `shard.rs` (5 champs additifs), `http.rs` (projection) — CONFIRMÉ 0-bump.
- **Instrumentation host-side** : `shard_session.rs:1570` (push token_at_ms), `:1631-1637` (windows(2)→deltas + mean + percentiles nearest-rank), `:1666-1674` (ShardRunOutcome), `:509-514` (projection result_data), `:1641-1642` (RunMetrics signé inchangé = mean) — CONFIRMÉ.
- **Gap de test P2** : `decode_loop_generates_until_eos` (`:2628`) exerce le VRAI decode (3 tokens = 2 gaps) mais n'assert AUCUN des 5 nouveaux champs ; seul le lifecycle echo-path (`:2253-2262`) les assert, en Some(0) via `drive_pipeline` (code path différent) — CONFIRMÉ.
- **Honnêteté doc** : `SHARD_BENCHMARKS.md §3` (present-tense PPL /result), `§5:107` (hash corpus), `benchmarks_standards.sh:44/221/351` — CONFIRMÉ.
- **Artefact committé** : `sprint82_t2_benchmarks.json` (status BLOCK, model NAME, blake3 null, bench_params présent hors chemin BLOCK) — CONFIRMÉ.

## Findings confirmés

**Aucun P0/P1.** Les items ci-dessous sont P2/P3 (documentables au commit body ; les 3 P2 marqués ★ sont recommandés à corriger avant commit vu leur coût quasi-nul et l'invariant honnêteté cardinal).

### P2 (4 distincts)

1. **★ Câblage des 5 métriques fines non couvert par un test hermétique** — `crates/nexus-shell-daemon/src/shard_session.rs:2628` (`decode_loop_generates_until_eos`). Le seul test qui exerce le VRAI `drive_decode_loop` (FakeHead+FakeTail, 3 tokens = 2 gaps réels, les 3 helpers tournent sur un slice non-vide) assert `result_text`/`tokens`/`run_proof` mais aucun de `ttft_ms`/`tpot_ms`/`itl_p50_ms`/`itl_p95_ms`/`decode_milli_tokens_per_sec`. Les seules assertions de ces 5 champs (`:2253-2262`) sont sur l'echo-path (`drive_pipeline`, hard-set Some(0)), qui ne discrimine ni p50/p95 ni la dérivation windows(2)/percentile. Une régression de câblage (oubli du push `:1570`, mauvais slice, inversion p50/p95 `:1673-1674` ou projection `:510-514`) shipperait en vert ; seul le harness live T2 (actuellement BLOCK{rig}) la rattraperait. **À faire** : dans `decode_loop_generates_until_eos`, après `result_data` `:2658`, ajouter des `assert!(...is_some())` sur les 5 champs (idéalement un forwarder à délai pour discriminer p50/p95) ; corriger le commentaire `:1709-1710` qui prétend le câblage seulement live-testable.

2. **★ PPL(shardé) sur-affirmée câblée : lecture morte `ppl_sharded`, parité structurellement toujours null** — `scripts/acceptance/benchmarks_standards.sh:291,351` + `docs/protocol/SHARD_BENCHMARKS.md:68-72`. `SHARD_BENCHMARKS.md §3` décrit au PRÉSENT « calculée tail-side … émise en scalaire via la route /result existante », mais aucun producteur n'existe : `b3_shard_pipeline.sh::emit_artifact` n'émet aucune clé `ppl`, la vue Rust `/result` n'a aucun champ perplexité (grep `ppl|perplexity` sur `crates/` = 0, hors faux-positif « aPPLication »). Donc `perplexity_parity.ppl_sharded` ET `delta` restent null MÊME sur rig chaud, ce qui contredit la note `:351` « null until a sharded PPL run lands » (aucun run ne peut la faire atterrir). Asymétrie honnêteté : `§2` flague honnêtement les outils « à construire », mais `§3` présente un câblage inexistant au présent. **À faire** : reformuler `§3` + `:351` au futur/conditionnel explicite (« sera émise ; câblage /result non implémenté en Phase B »).

3. **★ f-string `else \"\"` → SyntaxError sur Python < 3.12 → faux BLOCK sur rig chaud** — `scripts/acceptance/benchmarks_standards.sh:221` (et `:250` pour le Mac). `print(f"{pp if pp is not None else \"\"} {tg if tg is not None else \"\"}")` : un backslash dans la partie EXPRESSION d'une f-string est une SyntaxError sur toute version antérieure à 3.12 (PEP 701). Le python est dans `python3 -c '...'` single-quoted bash, donc les `\"` arrivent verbatim à Python. Sur un rig avec python3 3.6-3.11 (le HEAD parse les deux blocs), la compile échoue → stdout vide → `HEAD_PP/HEAD_TG` vides → `block_rig :230-232` avec un diagnostic qui accuse à tort llama-bench, masquant le bug harness. Fail-safe (jamais un faux PASS) mais casse la promesse « harness runnable qui PASS sur rig chaud ». Latent aujourd'hui (l'artefact committé est un BLOCK cold-rig, ce chemin n'a jamais tourné). **À faire** : remplacer par des quotes simples internes `else ''` aux 2 sites, ou `print("%s %s" % (...))`.

4. **★ « hash du corpus wikitext-2 » annoncé mais jamais calculé ni épinglé** — `scripts/acceptance/benchmarks_standards.sh:44` + `docs/protocol/SHARD_BENCHMARKS.md:107`. Le header et `§5 Déterminisme` affirment que l'artefact épingle le hash du corpus. Or `blake3_of()` n'est appelé QUE sur `MODEL_20GB` (`:193`) ; `WIKITEXT2_PATH` n'est jamais hashé (utilisé seulement en `-f` `:267`) ; `perplexity_parity` (`:345-352`) ne porte que la chaîne statique `corpus:"wikitext-2-raw"` + seed. Deux runs sur des corpus wikitext différents (mauvais fichier, troncature, variante raw) seraient indistinguables → invalide silencieusement la comparaison PPL que la note revendique. Claim de déterminisme non prouvé (précisément la classe que le sprint interdit). **À faire** : soit hasher le corpus (`wikitext2_blake3` dans `perplexity_parity`), soit retirer la mention du header `:44` et de `§5:107`.

### P3 (5 distincts, doublons cross-dimension collapsés)

5. **`assert_no_fs_path` aveugle aux chemins Windows JSON-escaped** — `scripts/acceptance/benchmarks_standards.sh:167`. Le case-pattern `*C:\\Users\\*` matche UN backslash littéral, mais `json.dumps` double chaque backslash (`C:\\\\Users\\\\`) → le chemin Windows natif traverse le backstop. Non-atteignable en pratique (`redact_model` réduit déjà au basename ; la fuite historique était en forward-slash, captée par `*C:/Users/*` + `*spike_fork*`). Défaut de durcissement latent d'un contrôle defense-in-depth (P2 initial DOWNGRADED P3). **À faire** : ajouter l'alternative `*C:\\\\Users\\\\*`, ou appeler `assert_no_fs_path` sur la valeur brute pré-encodage.

6. **Label machine = `MAC_SSH` (user@host) committé en provenance** — `scripts/acceptance/benchmarks_standards.sh:337` (`B_MAC="$MAC_SSH"` `:313`). Sur un run Metal, `single_machine[].machine` porte la cible SSH (`user@mac-m2.local` par défaut, cf. `rig.local.env.example:19`) dans l'artefact committé, alors que la tête est anonymisée en `"head"` (`:330`) et que la même phase réduit le modèle au basename pour ne pas fuiter un username. Non-fuiteur aujourd'hui (artefact committé = BLOCK, `single_machine` vide) ; choix opérateur. **À faire** : étiqueter la 2e machine par un alias stable (`"mac"`/`"metal"`).

7. **Artefact BLOCK committé non reproductible par le harness (`bench_params`)** — `.planning/active/sprint82_t2_benchmarks.json:10-15` vs `benchmarks_standards.sh:138-150`. L'artefact committé (status BLOCK) porte `bench_params` que le chemin BLOCK (`emit_and_exit` fallback python) n'émet PAS (seul `build_artifact`/PASS `:375` l'émet) ; la chaîne `diagnosis` ne matche non plus aucun message `block_rig` du script → artefact assemblé à la main. Rejouer le harness à froid l'écraserait par un JSON plus pauvre. Valeurs honnêtes (BLOCK, model NAME, blake3 null, benches null, BLOCK{rig} jamais RIG-ABSENT). **À faire** : soit ajouter `bench_params` (+ quant/n_shards) au fallback BLOCK, soit régénérer l'artefact via un vrai run froid.

8. **README §4 référence un `#16bis` kickoff + une extension Track J T3 absents des fichiers d'agents** — `docs/claude/README.md:665`. Le paragraphe d'enforcement T3 cite « nexus-sprint-kickoff, invariant #16bis » et « l'audit gate étend sa Track J » ; or `nexus-sprint-kickoff.md` n'a qu'un `#16` (nomme T1/T2, jamais T3) et pas de `#16bis`, et le gabarit Track J (`prompts/agent/audit-gate-checks.md`) ne vérifie que T1/T2/convergence, sans T3/benchmark. Pointeur pendant + enforcement documentation-only (cohérent avec le scope preflight « README §4 UNIQUEMENT »). **À faire** : soit câbler réellement `#16bis` + l'extension Track J, soit reformuler `:661-665` en libellé prospectif.

9. **Log `:182` trompeur « lossy pure-bash artefact encoder »** — `scripts/acceptance/benchmarks_standards.sh:182`. Sans python3, `run_llama_bench` (`:222-223`) et `build_artifact` (`:308`) court-circuitent → block_rig : l'encodeur pure-bash (`:152`) ne sert QUE le BLOCK minimal, jamais un PASS/mesure. Qualifier l'absence de python3 de « lossy » (dégradé-mais-fonctionnel) alors qu'il est de facto obligatoire pour toute mesure est une petite malhonnêteté de wording. **À faire** : « python3 requis — ce run BLOQUERA » ou faire de python3 un prérequis dur du preflight.

## Findings refutés (faux-positifs écartés)

- **[dim wire-safety, P3] Formule débit milli-tokens/sec (`1_000_000`) dupliquée 3×** — `shard_session.rs:513`. **REFUTED.** Les faits bruts sont exacts (3 copies : `:513`, `:1251` echo, `:1641` réel ; `.max(1)` redondant), mais le finding repose sur la règle named-constants (`feedback_named_constants.md` / README §6.9) qui est MAL APPLIQUÉE : cette règle vise une valeur de domaine ÉNUMÉRÉE miroir d'un enum Rust (ex. niveaux consent 1..4), et exclut explicitement longueurs/facteurs. `1_000_000` est un facteur d'échelle d'unité arithmétique (tokens/ms → milli-tokens/sec), pas un domaine énuméré. Précédent du repo : le frère `1000` (`:488`, toks_per_s) est nu et jamais flagué, et `/ 1_000_000` (ns→ms) est nu dans du code de production partout. Résidu = pure micro-préférence DRY, 0 impact runtime/wire/honnêteté/correctness. Le `.max(1)` est un garde anti-div-par-zéro aligné sur le frère `:488`, pas un défaut. Sévérité ajustée NONE.

- **[dim tests-semantic, P3] Le test whitelist 'required-contains' omet la clé 'tokens'** — `shard.rs:423`. **REFUTED.** Fait exact et pré-existant (Phase B n'a ajouté que les 5 nouvelles clés), mais ce n'est pas un défaut : (1) l'invariant est doublement verrouillé ailleurs — le drift-test `shard_schema_snapshot_matches_struct` (`assert_eq!` sur le schema entier, `tokens` dans `required[]` du snapshot) + le second test exact-match (`:458-478`, 14 clés dont `tokens`) ; (2) le test incriminé est documenté comme « required-field SPOT checks on representative types », pas une énumération exhaustive ; (3) aucun scénario où `tokens` perd silencieusement son statut sans faire échouer un test. Sévérité ajustée NONE.

## Invariants vérifiés

| Invariant | Statut | Preuve |
|---|---|---|
| **0 bump wire SBFB** | ✅ TENU | `run_metrics.schema.json` / `run_proof.schema.json` = diff DESCRIPTION-ONLY (note honnêteté sur `p95_token_latency_ms`) ; `shard_plan.rs` = doc-comment only, struct `RunMetrics`/`RunProof` INTOUCHÉS ; les 5 métriques fines = champs ADDITIFS `Option<u64>` `#[schemars(required)]` sur la vue NON-SIGNÉE `ShardSessionResultView` (`shard.rs:155-190`), précédent `rtt_frontier_ms` S81-I ; snapshots `shard_session_result_view/response.schema.json` régénérés (+35 lignes chacun). `http.rs:2419-2424` = simple projection. Le JCS signé round-trip byte-identique. |
| **0 dep runtime** | ✅ TENU | Aucun `Cargo.toml`/`Cargo.lock` dans le diff (`serde_json` déjà présent) ; llama-bench/perplexity = build-tools rig-only (upstream llama.cpp checkout, jamais buildés en CI, `LLAMA_BUILD_TOOLS=OFF` dans le fork vendored). 0 churn Cargo.lock. |
| **Hygiène (NAME+blake3, jamais chemin FS)** | ✅ TENU (état actuel) | Artefact committé `sprint82_t2_benchmarks.json` : `model: "codellama-34b.gguf"` (NAME seul), `model_blake3: null`, aucun chemin FS. `redact_model` strippe `/` ET `\` au basename (`:105-106`) ; `assert_no_fs_path` backstop (avec le trou double-backslash noté P3 #5, non-atteignable). |
| **Whitelist vue /result** | ✅ TENU | 2 tests étendus : required-contains (`shard.rs:423-443`) + exact-match 14 clés (`:458-478`) ; forbidden-set (`worker_pubkey`/`initiator`/`members`/`participants`) toujours interdit. Aucune identité exposée. |
| **Honnêteté** | ⚠️ TENU sur le point cardinal, 2 sur-affirmations secondaires (P2 #2, #4) | La note `p95_token_latency_ms = MEAN` est EXCELLEMMENT documentée (schema desc + doc-comment `shard_plan.rs:392-400` + `SHARD_BENCHMARKS.md §4`). MAIS 2 claims secondaires non tenus : câblage PPL(shardé)/route `/result` (P2 #2) et hash corpus wikitext-2 (P2 #4). À corriger. |
| **Rig-gated : BLOCK{rig} jamais RIG-ABSENT** | ✅ TENU | Artefact committé `status: BLOCK`, `diagnosis` explicite « rig: cold … never RIG-ABSENT (the rig is engaged for Phase A boot-SEED) » ; `block_rig` `:174` émet toujours BLOCK exit 1. |
| **Day-0 non touchées, refacto=0** | ✅ TENU | Diff = additive (5 champs vue) + doc-honnêteté + harness/scripts + tests. Aucune décision architecturale gelée modifiée, aucune refactorisation. |

## Delta tests (Win 2097 → 2098, +1)

- **+1 net** : `itl_percentile_and_tpot_are_deterministic` (fonction PURE, `shard_session.rs:1712-1756` — vérifie `token_latency_percentile_ms` nearest-rank + `mean_inter_token_ms`, gaps excluent le TTFT prefill).
- **`session_shard_in_process_full_lifecycle` ÉTENDU** (non-neuf) : asserts ajoutés sur les 5 champs via l'echo-path (`ttft_ms.is_some()`, `tpot_ms == Some(0)`, `itl_p50_ms == Some(0)`, `itl_p95_ms == Some(0)`, `decode_milli_tokens_per_sec.is_some()`).
- **2 tests whitelist ÉTENDUS** (non-neufs) : `shard.rs` required-contains + exact-match, +5 clés benchmark.
- **Non couvert (cf. P2 #1)** : le chemin decode RÉEL (`decode_loop_generates_until_eos`) n'assert aucun des 5 champs → gap de test hermétique.
- **Flaky env, hors-diff** : `dispatch_loop::boot_*_reenters_sync_set` FAIL sous charge parallèle / 3/3 solo PASS — territoire Phase A (`dispatch_loop.rs`), non touchés par ce diff (classe env-instable documentée CLAUDE.md tests iroh-networked).

État vérification main-thread (non-recompilé ici) : fmt clean, clippy `--workspace --all-targets -D warnings` VERT, nextest workspace VERT (2098 Win), doctests VERTS, release build VERT.

---

### P2/P3 documentables au commit body

**P2** : (1) gap test câblage métriques fines `shard_session.rs:2628` ; (2) sur-affirmation PPL(shardé) `SHARD_BENCHMARKS.md §3` + `benchmarks_standards.sh:291,351` ; (3) f-string `else \"\"` Python<3.12 `benchmarks_standards.sh:221,250` ; (4) claim hash wikitext-2 `benchmarks_standards.sh:44` + `SHARD_BENCHMARKS.md:107`.

**P3** : (5) `assert_no_fs_path` double-backslash `:167` ; (6) label MAC_SSH `:337` ; (7) artefact BLOCK non reproductible `sprint82_t2_benchmarks.json:10` ; (8) README `#16bis`/Track J T3 pendant `README.md:665` ; (9) log `:182` trompeur.

**Recommandation** : les 3 P2 marqués ★ (#2, #3, #4) + un correctif du gap de test (#1) sont quasi-gratuits et touchent l'invariant honnêteté cardinal — les corriger avant le commit atomique. Le gate review→Codex reste PASS-PENDING sans eux (aucun P0/P1).
## Codex reconciliation

Gate croisé externe Codex GPT-5.6 Sol (reasoning max), boucle complète — output
brut par round dans `sprint82_phase_b_codex_review.md` (R3 final, PASS) +
`_r1.md` (R1) + `_r2.md` (R2), non réécrits.

- **Fixes review appliqués AVANT Codex** (4 P2 + P3, doctrine honnêteté) :
  P2-1 couverture hermétique du câblage des 5 métriques (`decode_loop_generates_until_eos`
  + `PacedTailForwarder` gaps non-nuls) ; P2-2 PPL-shardée honnêtement documentée
  NON câblée en Phase B ; P2-3 f-string Python<3.12 réécrite `%`-style ; P2-4 hash
  corpus wikitext-2 réellement calculé ; P3 `assert_no_fs_path` robuste + label Mac
  non-identifiant + `bench_params` dans le chemin BLOCK.
- **R1 = GAP** (4 P1 faux-vert harness) → fixes : PASS gate (baselines + métriques
  shardées + pins), validation b3 (status/model/métriques), écriture échouée = FATAL
  exit 2, pins provenance requis.
- **R2 = GAP** (P1-3 fermé ; P1-A/P1-B résiduels) → hardening strict : validation b3
  exhaustive (status=PASS + NAME + **blake3** + n_shards + **5 métriques entières** +
  **freshness mtime**), `b3` émet `model_blake3`, pins format-validés (`is_blake3`,
  sha hex). Fermetures vérifiées empiriquement (matrice de rejets).
- **R3 = PASS au gate P0/P1** : P1-A CLOSED, P1-B CLOSED, 0 P0/P1, invariants durs
  tenus (0 wire, 0 dep, refacto=0, hygiène, jamais RIG-ABSENT, cold-rig jamais faux-PASS).
- **Résiduels P2/P3 documentés (non bloquants, batchés au commit body)** : commit
  operator-asserted (non lié crypto à l'exécutable) ; freshness = proxy mtime
  contournable ; ranges sémantiques via producteur de confiance Rust/b3 ; `assert_no_fs_path`
  détecteur ciblé (redaction = enforcement principal) ; pas de re-parse JSON strict
  post-write ; PPL-shardée/delta + automation kickoff/audit T3 = travail futur ;
  schema Option<u64> sans `null` = **pré-existant S81-I** (hors-scope, consommateur = harness bash).

Suites §7.4 relancées après les fixes Rust (round 0) : identiques (les rounds Codex
1-3 n'ont touché que shell/docs). Verdict final committable : **PASS**.
