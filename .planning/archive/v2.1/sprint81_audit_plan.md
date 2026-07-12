# Sprint 81 — Audit plan (à jouer en Phase 0 de S81)

> Écrit à la clôture de S80 (Phase J). Une session fraîche S81 joue ce plan
> AVANT toute Phase A, produit `.planning/active/sprint80_audit_findings.md`
> avec verdict PASS / CONDITIONAL PASS / FAIL, et écrit les commits
> `fix(sprint80)` pour les P0/P1. Canon des tracks :
> `prompts/agent/audit-gate-checks.md` (11 tracks A..K depuis `a6b4ca4`).

## 0. Mode d'emploi (session fraîche)

Lire ce plan + `prompts/agent/audit-gate-checks.md` EN ENTIER d'abord.
NE PAS lire `docs/{rust,shell}/PATTERNS.md` avant la Track C — former une
opinion indépendante depuis le diff, puis confronter aux patterns (règle
anti-anchoring du canon audit).

## 1. Périmètre

- **Diff audité** : `f4b4600..<tip S80>` (49 commits : Phase 0 audit S79 +
  kickoff + 10 phases A-J + arc off-sprint a11y/i18n/lots-rapides committé
  sur master + fixes process `a6b4ca4` + hooks `d1864dc`).
- **Particularité 1** : l'arc off-sprint (19 commits hors cadre
  `Sprint N Phase X`, de `8fa715a` à `94eb030`, feat + chore) a été committé
  SANS le process per-phase (mode rapid-add, PO 2026-06-28/29) — la review
  groupée + le Codex groupé de CET arc restent DUS (memory
  `rapid_front_add_session`) ; l'audit S80 vérifie qu'ils n'ont pas cassé
  d'invariant mais leur review complète appartient à la reprise post-S82.
- **Particularité 2** : S80 = sprint FRONT-dominant (greenfield Operator) ;
  le seul Rust neuf = routes `sbfb-factory` (auth cookie, git/diff, gates)
  + 0 route daemon (invariant Factory-hors-daemon à re-vérifier).
- **Baseline suites à l'entrée d'audit** : Rust nextest Win 2014 / Docker
  Linux 2018 (+4 `#[cfg(unix)]`) ; Vitest operator 201 (35 fichiers) ;
  E2E Playwright 10 ; Vitest web 411 ; T2 `sprint80_t2_acceptance.json`
  PASS committé.

## 2. Les 11 tracks (canon `prompts/agent/audit-gate-checks.md`)

| Track | Focus S80 spécifique |
|---|---|
| A suites | 3 blocs fail-fast + dual-platform Docker (voir verification.md §Checklist — statut Docker consigné honnêtement, échappatoire `SBFB_TEST_HTTP_TIMEOUT_SECS`) |
| B security | auth cookie bootstrap (T-OPERATOR-CSRF §14), MUR jamais bouton, artefact T2 allowlist (0 secret), fixture daemon 127.0.0.1, scan anti-score |
| C patterns | §P72 (cwd-fixture + 2 footguns) exact vs code ; §P70/§P71 non régressés |
| D scope | 8 cuts §Out kickoff respectés (Aperçu scellé/publish Operator/CM6/⌘K/multi-session/timeline/i18next†/auto-bascule) ; † Lingui livré off-sprint sur OVERRIDE PO ≠ violation du cut i18next |
| E tests delta | 201/35/10 operator + 411 web + 2014 nextest — trajectoire honnête (C 52→I 201), CI vitest gaté 2 surfaces |
| F review files | 10 phases × 3 artefacts (preflight/review/codex) présents + review = UN SEUL header `## Verdict` (leçon Phase I) |
| G carry-overs | §3 ci-dessous — chaque item CLOSED ou re-routé avec rationale |
| H HARDENING | drift HARDENING_ROADMAP vs état réel |
| I meta-process | bodies 9 sections, Codex bruts non réécrits, G8 verdicts ; 1re application du canon amendé `a6b4ca4` (DoD (d)) |
| J testabilité (standing) | T1 BLOQUANT-vert + T2 JSON machine-lisible ; 0 prose DIFFERE-* |
| K docs-contract (standing, NEUVE) | clôture S80 livrée Phase J (llms.txt H2 Operator + REFERENCE §Operator + pointeur EXPLANATION) — vérifier 15 source-refs résolvent toujours + honnêteté ; frontière S81 neuve non indexée = P1 |

Track G1 : `sprint80_design_review.md` existe dans `active/` (vérifier avant
archive). Verdict tree : PASS (0 P0/P1, ≥1 P2+ documenté) / CONDITIONAL
PASS / FAIL (P0 ou P1) ; 0 P0/P1 ET 0 P2+ = CONCERN (rigor signal G4).

## 3. Carries à escalader (inventaire nommé, zombies filtrés)

### Fermés S80 — NE PAS re-router (vérifier le statut LIVE)
- TEST-ISOLATION-SBFB-HOME — CLOSED Phase I `782796c`.
- Gating CI Vitest operator (S2-F2) — CLOSED Phase I `782796c`.
- P2-1 bootstrap Host non-loopback — CLOSED in-phase A.
- P3-6 artefact T2 gitignoré — CLOSED Phase I (JSON committé).

### P1 standing (dette dominante, in-vivo)
1. **Sharding S77 in-vivo RIG-ABSENT** — orchestrateur de session + benchmark
   live 2-machines absents (S78 différé Factory-first, `sprint78_audit_plan.md §7/§10`).
2. **app-authoring in-vivo `Not evidenced`** — parcours auteur réel →
   publish → rendu cross-pair jamais exercé (`docs/factory/llms.txt`).

### P2/P3 ouverts (S79 + phases S80 + process)
3. S79 audit findings 8 P2 / 11 P3 backend/docs (`archive/v2.1/sprint79_audit_findings.md`) — re-vérifier lesquels restent ouverts.
4. Couverture étiquette : ~21 familles `DOMAIN_*_V1` sans schéma généré (registre `// FRONTIER:` opt-in incrémental).
5. Doc-lint sémantique limite : `check-factory-docs.sh` vérifie l'existence, pas le support des claims.
6. **Fix process 5 (`a6b4ca4`)** : parité Rust↔TS + élargir le scan des gates frontier/docs à `tools/factory-operator` (`check-frontier-contracts.sh` ne couvre que `crates`+`web/src` — une forward-promise y a échappé en S80).
7. `sse_gate` forge `format!` brut (`operator_server.rs`) — durcir `serde_json::to_string` si message dynamique.
8. Asymétrie blake3 daisyui (`exists`) vs animejs (recompute) — Phase D.
9. `GET /api/git/diff` branche `truncated==true` non testée hermétiquement — Phase F.
10. Docs périmées `tools/factory-ui` dans `docs/agent/RRV_FACTORY_CONTRACT.md:109,142`.
11. `GateIssueView.line=null` hardcodé + refactor `GateResult.issues` ligne fine — Phase G (carry P1 nommé au commit `ed00b4a`).
12. V5/V6 + marqueur-gate-par-fichier + onglets Aperçu scellé/Preuve DÉGRADÉS — Phase H (fondation Viewer).
13. Fraîcheur head-live figée au mount (ment après 1er commit) — Phase H.
14. P3-e surface prompt-injection `onHunkIntent` (`VerifyScene.tsx`) — Phase H.
15. HEAD-50-YOUNG-REPO : `collect_sprint_commits` fallback `HEAD~50..HEAD` invalide sur repo <51 commits (`sprint_history.rs`, contourné fixture ; §P72).
16. PO-MULTILINE-SCAN : continuations `msgstr` multi-lignes hors axe anti-score (`scan-front-discipline.sh`).
17. CALLS-ORDERING : `/__calls` compteurs cumulés, ordonnancement wall-clock implicite (`steer.spec.ts`).
18. RR-1 : harness t2 — un test SUPPRIMÉ resterait PASS si N<attendu ET ids inchangés (garde ids-attendus posée post-Codex, garde count-total absente).
19. **Env-block Docker-on-Windows loopback HTTP** : 2 tests operator_server (`operator_context_pack_schema_complete`, `operator_git_diff_endpoint_returns_envelope`) TimedOut 30s reproductibles en Docker-on-Windows, verts Windows natif — même classe que `multi_daemon` iroh ; statut du run officiel avec `SBFB_TEST_HTTP_TIMEOUT_SECS=120` consigné dans verification.md.

### Standing + décision
20. Track J testabilité + Track K docs-contract = standing chaque sprint.
21. **Fondation Viewer/Operator S81** (socle `tools/factory-ui` jeté, kickoff S80 Arbitrage PO #2) + décision PO séquencement confirmée : S81 = iroh 0.98→1.0 (relais N0 EOL 2026-09-30, kickoff DRAFT en staging `.planning/research/sprint81_iroh_upgrade/`) — la fondation Viewer se re-planifie à son slot.

### Externes inchangés
P2-A-1 rand (exemption), P2-AUDIT-2 iroh (pin 0.98 → objet même de S81), T-NN+2 iframe Rust-wasm (§P34), P3-OS-1, LT-2 Radicle ARMÉ (flip = PO).

## 4. Out-of-scope de l'audit

Les décisions Day-0 S80 (D1..D11 kickoff) et les arbitrages PO tracés
(greenfield, React 19, Base UI seule, cookie HttpOnly, bi-focal manuel,
Lingui OVERRIDE) sont GELÉS — l'audit vérifie leur respect, ne les re-débat
pas. Le contenu de l'arc parqué `wip/factory-front-arc-post-s82` est HORS
périmètre (non mergé).

## 5. Format du livrable

`.planning/active/sprint80_audit_findings.md` : un finding par item
(sévérité P0-P3, evidence fichier:ligne/commit, action), verdict final en
dernière ligne unique `## Verdict: <PASS|CONDITIONAL PASS|FAIL>`, commits
`fix(sprint80): ...` pour chaque P0/P1 avant fermeture de gate.

## 6. Note

S80 ferme avec la PREMIÈRE application du canon docs-contrat amendé
(`a6b4ca4`) : la Track K est jouable dès cet audit (la clôture S80 est le
cas de référence). L'audit S80 est aussi le premier à auditer un sprint
contenant un arc off-sprint committé mode rapid-add — statuer sur la dette
de review groupée (due à la reprise post-S82, pas à cet audit).
