# Review G4 — Sprint 77 Phase K (wrap-up + gate produit)

> Workflow ultracode 5 dimensions (Opus 4.8 1M) + synthèse adversariale.
> Phase K = wrap-up SANS code fonctionnel net : harness shell + docs longue-vie
> + verification.md + sprint78_audit_plan.md.

## Verdict: PASS

Review PASS après corrections in-phase + reconciliation Codex (5 rounds,
PASS-PARTIAL 0 P0/P1). Le bloquant initial **F1 (P1)** + les 6 P2 + 2 P3 de la
review, ET les findings additionnels de Codex (2 P1 de faux-livraison S78 + 1 P2
harness JSON + ~10 P2/P3 doc-precision), tous corrigés à la source.

## R1 — Harness
PASS. `b3_shard_pipeline.sh` honnête et non-falsifiable. `bash -n` clean. JSON =
exactement les 10 champs §14.2 (numériques `int`/`null`, jamais string). **Gate
anti-faux-vert SOLIDE** : un seul `pass()`, un seul call-site, gardes séquentielles
`run_proof` non-vide ET `toks_per_s>=1` — aucun chemin vers `pass()` sans les deux.
Exit codes distincts (3/1/0). Run sans config → RIG-ABSENT structurel. 0 P0/P1.

## R2 — THREAT docs
PASS après corrections. v14 honnête ; §5.9 STRIDE / §2 A8 / §4 DFD / §6 LINDDUN
cohérents avec le code (admission `shard.rs::accept`, claim crypto-avant-IO, cap
256 MiB, route stub `http.rs:2104-2168`). Le re-labeling S78 était **incomplet**
(F1 P1 + 3 P2) → **corrigé**.

## R3 — PATTERNS
PASS après corrections. §P67/68/69 + P39 résolvent tous au code réel ; 0 pattern
orphelin. Issues = doc-accuracy (F5 mischaracterisation `accept`, F6 relabel §P64/P65
incomplet, F10/F11 §P68) → **corrigées**.

## R4 — verification + audit_plan
HONNÊTE. Chiffres fail-fast, 24 noms de tests §6, artefact T2 RIG-ABSENT, compteurs
carries — tous re-vérifiés contre logs réels + code. **Aucun faux-vert, aucun nom
non-résolu, aucun item S78 faux-déclaré livré.** `shard_backend_primitive` = 0
occurrence (corrigé). 4 rows carry 3/3 exacts + SHARD-PROVISIONAL P1 + Track
Testabilité standing.

## R5 — scope/wire/Day-0
PASS. **0 code fonctionnel net** confirmé (`git diff HEAD` : aucun `.rs`/`.ts`/`.tsx`
/`.toml`/`.lock`). **0 bump wire** (4 `DOMAIN_*_V1` additifs). Day-0 non touchées.
État docs cohérent (1949 Win / 1953 Docker / 411 Vitest / 41+1skip uniformes).
Untracked hors-scope non référencés.

## Findings consolidés

| # | Sev | Finding | Statut |
|---|-----|---------|--------|
| F1 | **P1** | `THREAT_MODEL.md:1031` SI-5 « (Phase K) » alors que Completion + v14 disent S78 (sur-claim de correction) | **CORRIGÉ** → « (carry S78) » |
| F2 | P2 | `THREAT_MODEL.md:947` row I « ride le data-plane (Phase H/I/J) » résiduel | **CORRIGÉ** → « (carry S78) » |
| F3 | P2 | `THREAT_MODEL.md:1142` §16 N1 prose « (Phase I/K) » | **CORRIGÉ** → « (S78) » |
| F4 | P2 | v14 + verification « `RunProof::new` reste `#[cfg(test)]` » faux littéral (`pub fn`, seuls callers test-gated) | **CORRIGÉ** → « aucun caller prod ; appels sous `#[cfg(test)]` » |
| F5 | P2 | §P67 item 3 « ONE frame, not a session / NO decode loop / forward_hidden » contredit `shard.rs:309-323` (boucle `while let Some(frame)` bi-stream, appelle `forward`) | **CORRIGÉ** → « serves a long-lived bi-stream ; no autoregressive decode loop / no orchestrator » (+ harness + THREAT §4/§5.9/N0 + verification reformulés) |
| F6 | P2 | relabel §P64 (`:3415/3592/3599`) + §P65 (`:3659`) « Phase K / I/K / H/I/J » stale | **CORRIGÉ** → S78 |
| F7 | P2 | P39 dit « S78 seam » mais commentaires CODE (`http.rs`/`daemon.ts`/`ShardSessionPanel.tsx`) disent encore « Phase K » | **DOCUMENTÉ carry S78** (STALE-PHASE-K-COMMENTS, audit_plan §8 ; scrub-code hors wrap-up 0-code) |
| F8 | P3 | truncation décimale `toks_per_s`/`ttft_s`/`rtt_frontier_ms` (faux-vert impossible, conservateur) | DOCUMENTÉ (limite connue, centi-unités S78) |
| F9 | P3 | `MODEL_BASE` grep non-ancré (faux-OK présence, jamais faux-PASS) | DOCUMENTÉ |
| F10 | P3 | §P68 nom test tronqué | **CORRIGÉ** → `..._single_worker` |
| F11 | P3 | §P68 `covers_full_model` localisation | **CORRIGÉ** → `placement.rs` |
| F12 | P3 | « Phase J/K » dans changelogs historiques v10/v12/v13 | LAISSÉ (records datés, défendable) |
| F13/F14 | P3 | verification row 6 lecture-rapide ; SPRINT_LOG ref `d9c93ff` hors-portée | DOCUMENTÉ |

**Récap : 0 P0 · 1 P1 (CORRIGÉ) · 6 P2 (5 corrigés + 1 documenté carry) · 7 P3.**

## Corrections in-phase (re-vérifiées)
Après corrections review : harness `bash -n` clean + re-run RIG-ABSENT (artefact
resync verification.md). La review (grep) avait sous-estimé quelques résiduels que
Codex a ensuite attrapés (cf. ## Codex reconciliation) — tous fermés.
Phase K reste **0 code fonctionnel net** (corrections = docs + harness shell + planning).

## Codex reconciliation
Codex GPT 5.5 (`codex exec`, brut dans `sprint77_phase_k_codex_review.md`) — **5
rounds** de convergence, verdict final **PASS-PARTIAL, 0 P0/P1**. Codex, plus
exhaustif que la review sur la propagation, a trouvé des findings RÉELS au-delà de
la review, tous corrigés à la source (no band-aid), puis re-vérifiés :
- **2 P1 faux-livraison S78** : row SI-5 `(Phase K)` (round 1) + §16 N0 prose
  « transport sketch y est aussi câblé » L1102 (round 3) → re-ciblés « carry S78 ».
- **P2 harness JSON** (round 4) : fallback bash émettait `n_shards:bad` invalide
  pour un `N_SHARDS` non-numérique → coercion int-or-null, edge re-testé vert.
- **~10 P2/P3 doc-précision** : relabels §P64/§P65/§P67 incomplets, `RunProof::new`
  est `pub fn` (callers seuls cfg-test), §P67 « ONE frame » → boucle bi-stream,
  §P67 « never links iroh » trop large, compte DOMAIN 4→5 (COMPUTE_GROUP[B]),
  inventaire STALE-PHASE-K-COMMENTS complété, snippet T2 marqué abrégé.
- **P3 résiduel** (SCOPE-WIRE-PRECISION) : claim abstrait « tous _VERSION=1 / 4
  domaines » imprécis repo-wide ; verification.md row 38 porte déjà le caveat
  correct (5 domaines S77 + `INVITE_FORMAT_VERSION=2` hors-S77). Accepté.
Aucun re-run de suite nécessaire (corrections = docs + harness shell, 0 code Rust/web).

## Synthèse
Le wrap-up est structurellement honnête sur les axes haut-risque (harness
non-falsifiable, 24 noms résolus, chiffres exacts, 0 code/0 bump/0 Day-0). Les
findings étaient des forward-refs S78 incomplets + 2 preuves citées imprécises +
1 mischaracterisation du data-plane — tous corrigés à la source (no band-aid).
Prêt pour Codex.
