# Sprint 81 Phase A4 — Préflight G8 (POINTEUR, pas un re-jeu)

> **Verdict hérité : PLAN-ADAPT** — la Phase A4 est la moitié « fix code » du
> SPLIT décidé par le préflight G8 de la Phase A3
> (`sprint81_phase_a3_preflight.md`, Workflow `wf_7ffb4c95-8b6`, 11 agents,
> 5 scans + 5 vérifications adversariales). Ce fichier est un POINTEUR de
> conformité process (le hook lightcheck Check 8 exige un artefact préflight
> par phase committée) : le G8 d'A4 N'A PAS été re-joué — il vit
> intégralement dans l'artefact A3, sections :
>
> - **§2** — root-cause re-établie au code (open_doc/create_doc n'entrent pas
>   dans le sync-set iroh-docs 0.98 ; broadcast gaté `is_syncing`
>   `live.rs:714` ; accept rejeté `NotFound` `state.rs:97` ; seul
>   `start_sync` insère `live.rs:414`).
> - **§3 « A3b »** — l'approche corrigée que A4 implémente : boot
>   `start_sync(vec![])` après open/create du project doc + test hermétique
>   red→green mode-restart + re-run b3 différentiel. (Nommage : « A3b » du
>   préflight = « A4 » au canon `Phase [A-Z]+[0-9]?` README §4 — le regex du
>   hook tronquerait « A3b » en « A3 » et bypasserait les gates.)
> - **§6** — plan de tests (+1..2 ; livré +2 : CONTROL + GREEN).
> - **§7** — risques (ne pas re-coder le keepalive worker ; suffisance live ;
>   re-calibrage au bump).
> - **§10** — carries (Phase B/C re-calibrage `is_syncing`/`start_sync` vs
>   iroh-docs 0.101 ; Phase G THREAT_MODEL/§15.3 ; Phase K libellé T1).
>
> **Affinements post-A3 intégrés par A4** (issus de l'observation live A3,
> consignés dans `sprint81_t2_baseline_098.json` clé
> `a3b_differential_contract`) :
> 1. Le critère différentiel n'est PAS un PASS-live (déjà PASS via le
>    side-effect `share_write` du submit-path) mais la **disparition de la
>    fenêtre boot→premier-submit** (accept sans `NotFound` AVANT tout
>    submit) — prouvée LIVE le 2026-07-03 (`sprint81_t2_a4_differential_098.json`).
> 2. Carry A3 additionnel absorbé par A4 : hermétisme des 6 tests
>    `consent_*` (`mk_state_with_sbfb_home`, pollution rig observée A3).
