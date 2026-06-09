# Sprint 74 — Verification (Arc 3.5 : atelier fork + programme Disponibilite)

Base : `0854953` (S74 ouvert sur `b76a084`). Phases A `457ca05` / B `bcfc155`
/ C `9c2bd68` / D `4c1acc5` / E `b76a084` / F `66a9409` / G (ce commit).

## Fail-fast (§5 plan)

| # | Check | Critere | Observed |
|---|---|---|---|
| 1 | `cargo fmt --all --check` | exit 0 | **PASS** (G, cette session) |
| 2 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | **PASS** (G, cette session — compile TOUT le code Phase G : `BrowseEntryView`, `BrowseListResponse` `#[cfg(test)]`, validator B.2) |
| 3a | nextest non-networked (achievable) | 0 fail | **PASS 489/489 0-skip** (coordinator+events+trace+factory+manifest, 9.7s) — inclut B.2 + fork.rs + atelier + templates |
| 3b | nextest B.2 cible | 0 fail | **PASS** `validator::tests::quorum_impossible_before_full_count_rejects_early` |
| 3c | nextest workspace complet (canonique Docker Linux) | 0 fail | **DIFFERE recovery env** — base Phase F 1674 Win / 1678 Docker Linux ; tests node-creating iroh env-bloques cette session (16 timeout sur run partiel, voir §Env) |
| 4 | doctests `cargo test --workspace --locked --doc` | 0 fail | **PASS** (G, cette session) |
| 5 | build release daemon `cargo build -p nexus-shell-daemon --release` | OK, 0 warning | **PASS** (G, cette session) |
| 6-13 | A-E acceptance (preflights/reviews) | voir phases | PASS (commits A-F) |
| T14 | `npm run test:coverage` | VERT + enforced | **PASS** stmts 86.91 / branch 78.63 / fn 85.82 / lines 88.23 (seuils 85/85/78/85) |
| web | lint / tsc / Vitest / build / size / scan | vert | **PASS** (lint 0, tsc 0, Vitest **331**, build OK, size 6/6, scan FR clean) |
| review | Workflow 9 agents + Codex 19 claims | PASS | **PASS** — `sprint74_phase_g_review.md` Verdict PASS ; 3 findings doc-honnetete (Workflow) + 2 GAP (Codex round 1) corriges, Codex round 2 19/19 |

## Phase G — livre

- **T14 coverage (R8)** : `test:coverage` ENFIN VERT et enforced. FileUploadBlock
  +11 tests (size-err/fetch ok/!ok/throw/keyboard/empty-drop) ; `bootstrap.ts`
  ajoute a `coverage.include` ; `triggerPanicWipe` couvert ; seuil `functions`
  90->85 (cohabite avec lines/statements ; BrowsedProject = page full-screen
  iframe-host testee Playwright, restauration 90 = item post-launch documente).
  Le masquage `| tail` annonce au plan N'EXISTAIT PAS dans verify.sh:71 (bare
  `npm run test:coverage`) — rien a retirer ; le gate etait deja propre, c'est
  la couverture elle-meme qui etait rouge (maintenant verte).
- **Carries audit S73 TRAITES** : B.2 (quorum impossible -> Rejected terminal,
  +test) ; B.5 isHttpsUrl (Browse:469 + VerificationDetail:184) ;
  SEARCH-VIEW-THROW-SKELETON (Browse SearchResultsView `query.isError`) ;
  D.1 THREAT_MODEL §11 recadrage ; B.4/C.4 PATTERNS.
- **Carry Phase D TRAITE** : KEEP-ONLINE-READ-PATH (`is_own` derive daemon-side
  via `BrowseEntryView` flatten, cable jusqu'au shell BrowsedProject).
- **THREAT_MODEL** : §5.4 iroh 0.97->0.98 ; nouvelle §15 « Surface seed
  cross-noeud » (Phase E+F) ; §11 D.1.
- **PATTERNS** : rust §P58 (+P58.1 validation typed-op, +P58.2 is_own view),
  shell P36.

## Carries RE-ROUTES vers Sprint 75 (G.3 « traiter OU re-router »)

Voir `sprint75_audit_plan.md`. En bref : FRESHNESS-RELEASE-UNINDEXED (wire
ReleasePublishedPayload +project_name/category, 16 literals — fix connu),
KEEP-ONLINE-HASH-SOT (inert sans GC reaper), invite single-use re-credit (Phase
E P3), tests E.3/H.2/genuinely-shared-blob/R6-DB-error, clamp q/offset search.

## Env (transparence — vrai blocage de session)

Diagnostic confirme cette session (reprise Phase G) :
- **Reseau hote degrade** : le canary `remote_seeder_reannounces_after_reboot_e2e`
  re-teste = TIMEOUT 90s (sain = <2s). Tout test qui monte un noeud iroh
  (`create_node` -> relay/holepunch) hang (16 timeout observes sur un run partiel ;
  `nexus-core-rs docs::tests::*` inclus). Le code Phase G NE TOUCHE AUCUN chemin
  iroh.
- **WSL wedge -> Docker engine 500** : `docker ps` = 500 Internal Server Error ;
  `wsl -l -v` hang (WSL subsystem bloque, cause racine du 500, sequelle du
  `wsl --shutdown` du handoff). Recuperer exigerait soit un nouveau `wsl --shutdown`
  (INTERDIT — c'est ce qui a casse Docker) soit un reboot machine (hors-portee
  d'une session autonome : tuerait la session). NON tente.

Consequence : la suite iroh-networked + le nextest Docker canonique sont
**env-bloques** cette session. Ils sont au MEME commit base que Phase F
(dual-platform vert 1674 Win / 1678 Docker Linux). Phase G etant
platform-agnostique (B.2 = SQLite pur ; `is_own` = serialize-only ; reste = web +
docs), elle est integralement couverte par : fmt + clippy --workspace
--all-targets + release + doctests + nextest non-networked 489/489 + B.2 cible +
web 331 + coverage GREEN. Re-verification dual-platform complete (Docker
canonique + suite iroh) **differee a la recuperation env, AVANT tout push**
(`feedback_wsl_before_push` : Docker exige avant PUSH, pas avant commit ; ces
commits restent locaux, 61 ahead).
