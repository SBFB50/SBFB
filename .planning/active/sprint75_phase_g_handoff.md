# Handoff — Sprint 75 reprise en Phase G (à coller dans une session fraîche)

> Sprint 75 (pivot découverte PULL node-centrique + ancre VPS) est OUVERT et
> avance. **Phases A + B + C + D + E + F DONE.** Cette session fraîche reprend
> en **Phase G (wrap-up + acceptance survives-VPS-death — DERNIÈRE phase)**.
> Ne ré-invente pas le sprint : tout est déjà décidé et écrit. Suis le workflow.

## 0. Protocole de session fraîche (obligatoire)

1. **Lis intégralement** : `docs/claude/README.md` (workflow, cycle, audit gate,
   G8 preflights, gate Codex, commit 9 sections, dual-platform, bootstrap §7.1
   routing A/B/C/D — et SURTOUT **§2.3 + §2.4 + §4.4** : les specs canoniques de
   `verification.md` / `audit_plan.md` / le routage des findings de reviews,
   c'est le cœur de G) + `CLAUDE.md` + memory `MEMORY.md` + `nexus_grid_pivot.md`
   (le **Tip « 2026-06-10 suite 7 »** a tout l'état Phase F) +
   `feedback_wsl_before_push` + `feedback_dual_platform` +
   `feedback_codex_gate_strict` + `feedback_codex_raw_output` +
   `feedback_full_failfast` + `feedback_background_checks` + `feedback_cd_web_trap`
   + `nested_git_web_trap` + `feedback_radicle_private` (LT-2 dry-run privé).
2. **VÉRITÉ TERRAIN = git, pas la mémoire** : `git log --oneline -6` +
   `git status -sb` AVANT toute décision. Au moment d'écrire ce handoff :
   HEAD = `4f52bea` (Phase F), **15 ahead local (+1 avec ce handoff), RIEN
   POUSSÉ**. Si le tip mémoire diverge, git fait foi.
3. **Routing** : main thread = ROUTEUR. `.planning/active/` contient
   kickoff + plan + les preflight/review/codex_review des phases A-F, **pas**
   de `sprint75_verification.md` → **Cas B (sprint en cours)**. Phases A-F
   committées → phase suivante = **G (dernière)**.
4. **Règle modèle** : jamais le param `model` dans `Agent()`.
   `nexus-phase-preflight-deep` est ENREGISTRÉ ; `nexus-phase-review-deep` ne
   l'est PAS → fallback review par Workflow multi-agent (pattern C/D/E/F :
   5 dimensions adversariales → skeptics refute-by-default sur P0/P1 →
   synthèse ; en F il a sorti 1 P1 réel [test faux-vert lock-4b], ça marche).

## 1. Où on en est (commit stack S75)

```
4f52bea feat(shell): Sprint 75 Phase F — node-centric Browse (nodes list + node catalog + add-anchor)
491b3c8 docs(planning): handoff prompt for the next session (S75 Phase F)
1486fc9 feat(daemon): Sprint 75 Phase E — headless VPS anchor (config-driven seed driver + signed authoring)
41b13e3 docs(planning): handoff prompt for the next session (S75 Phase E)
0010450 feat(core+daemon): Sprint 75 Phase D — multi-provider pull + node identity exposure
9f7de7f docs(planning): handoff prompt for the next session (S75 Phase D)
821aa8c feat(daemon): Sprint 75 Phase C — node-directory ingest + remote-catalog durability (boot re-pull)
f6637d3 feat(core+daemon): Sprint 75 Phase B — NodeDirectoryEntry + DOMAIN_NODE_DIRECTORY_V1 + generic ingest gate + authoring route
479a87c feat(core+daemon): Sprint 75 Phase A — re-mint PoW + endpoint address on outbox replay
0e2fb6b chore(planning): Sprint 75 Phase 0 — S74 audit findings (PASS)
```

- **Phase F** = DONE `4f52bea` (front node-Browse) :
  - Pages lazy `/nodes` + `/node/:nodeId` ; AddAnchorDialog = la route
    subscribe EXISTANTE (ancre = subscription, Q3/DQ3) ; lignes « en attente »
    copy honnête ; cold-start gated sur subscriptions CONNUES-vides.
  - **Verrou 4 PO livré + testé** : provenance = `VerificationDetail` par
    projectId + prop additive `expectedArtifactHash` (avertissement « autre
    version que celle affichée ») ; marqueur « Version dérivée » UNIQUEMENT
    depuis l'annonce ÉDITEUR `source==="direct"` match exact (pid,hash) —
    les boucles curator/nodedirectory hardcodent `is_open_source:false`
    (browse.rs:684/803), les lire = faux marquage ; row sans annonce éditeur
    = AUCUN claim.
  - **Badge Q7 = COMPOSÉ FRONT-SIDE** (PLAN-ADAPT preflight F : le variant
    wire `reachableviaseeder` du handoff F n'a JAMAIS existé ; Phase D livre
    la paire unreachable + peer_count>0 version-exact ; `/browse`
    BYTE-IDENTIQUE) — gate `!!archive_hash`.
  - **WEB-1 CLOSED** : clé loopback `self_pin_enabled` 3-états sur /seed-count
    (null = row M18 absente = diffusée par défaut ; précédence écho POST >
    intent > défaut ON) ; échos in-session reset sur la PAIRE pid:hash.
  - `archive_hash` discriminateur `#[serde(default)]` sur POST /seed
    (fallback annuaire narrowé, lowercase, 400 pré-F préservé).
  - Gates : preflight PLAN-ADAPT ; review Workflow 7 agents (1 P1 corrigé +
    20 P2/P3/NIT in-phase, 1 P2 duress déféré S76) ; **Codex 3 rounds → R3
    21 CONFIRMED, 0 GAP, OVERALL: PASS**.
  - Compteurs : nextest --workspace **1750** 0-fail 0-skip (Windows) ; web
    Vitest **367**, coverage 87.17/79.01/85.92/88.5, size 6/6, scan FR.

## 2. Docs à lire (tout est décidé)

- `.planning/active/sprint75_plan.md` — **§Phase G** (G.1-G.5) + **§5
  fail-fast 24 rows** (c'est LA checklist que verification.md remplit) + §9
  checkpoint de clôture.
- `.planning/active/sprint75_kickoff.md` — §2 goal (critère SMART =
  fail-fast vert + survives-VPS-death démontré), §4 verrous + test cardinal,
  §8 carries, §9 scope cuts, §13 checkpoint PO (le #5 valide l'usage des
  assets SSH pour l'acceptance).
- **Les 6 phase reviews** `sprint75_phase_{a..f}_review.md` — §4.4 README :
  G doit les PARSER et router chaque P2/P3 dans les tracks de
  `sprint76_audit_plan.md` (ratio reviews présents 6/6 à inscrire).
- `.git/CODEX_SPRINT75_PHASE_F.txt` — RÉUTILISE sa structure pour le prompt
  Codex Phase G : Output contract (tokens littéraux GAP/CONFIRMED + `OVERALL:
  PASS/FAIL`), SCOPE explicite (G = docs planning + 4 fixes hygiène Rust +
  THREAT_MODEL + PATTERNS — scoper `git diff` en conséquence), PHASE BOUNDARY
  (juger G, pas S76), décisions à ne pas re-litiger. F a pris 3 rounds (3 GAP
  réels) ; D avait passé en 1.

## 3. Phase G — scope (plan §G, kickoff, déférés routés G)

**Wrap-up + acceptance.** Commit cible : `feat(daemon): Sprint 75 Phase G —
wrap-up + survives-VPS-death acceptance + S74 hygiene carries` (titre AVEC
« Phase G » → hooks lightcheck ARMÉS → preflight + review + Codex
obligatoires, précédent S74-G `bede850`, PAS le précédent S71/S73 docs-only).

1. **Acceptance « survives-VPS-death » cross-machine** (le critère SMART du
   sprint, kickoff §4 test cardinal) : assets SSH mac `192.168.1.53` + vps
   `135.181.42.188` (PO checkpoint #5 OK). Démontrer : (a) aucune découverte
   hard-câblée sur le node_id du VPS (`default_curators=[]` compilé — déjà
   test-pinné) ; (b) une autre ancre est première-classe ; (c) les apps
   seedées restent joignables tant qu'un détenteur du BLAKE3 répond, VPS
   éteint. + **Valider l'unit systemd Phase E EN LIVE** (déféré review E →
   G) : boot stock Debian/Ubuntu, `systemd-analyze security`, bind QUIC sous
   seccomp (`deploy/nexus-shell-daemon.service` + `config.toml.example`).
2. **C6 E2E cross-machine** (gate Phase A, différé C→G) : la découverte
   re-mint marche au-delà de 30 min entre machines réelles
   (`stale_announcement_accepted_by_fresh_receiver` est le unit-simulé ;
   l'E2E = Win↔Mac/VPS réel).
3. **Hygiène carries S74** (4 fixes Rust + tests plan §G.3) :
   - CARRY-5 : clamp `offset`/`q` de la route search (http.rs) →
     `search_clamps_offset_and_query` ;
   - CARRY-2 : guardrail trip ⇒ `Rejected` TERMINAL (validator.rs +
     validator_loop.rs) → `guardrail_trip_sets_rejected_terminal` ;
   - PULL-1 : dedup provenance au deploy (deploy.rs) →
     `deploy_strips_existing_provenance` ;
   - FORK-1 : entry-cap au fork (fork.rs) → `fork_entry_count_capped`.
4. **THREAT_MODEL §15 — rows déférés D/E/F** : directory pull route publique
   blob-serve (oracle drive-by + amplification dials), /nodes, SEED-1/SEED-2,
   fresh-flood displacement, boot seed driver + requester route (E), surfaces
   front F (exposition accrue de seed_voluntary/set_keep_online sans duress
   gate — le P2 déféré).
5. **CARRY-1 / LT-2 ARMÉ** : flipper les docs + dry-run Radicle PRIVÉ
   (memory `feedback_radicle_private` : la réplication sélective permet un
   repo privé d'abord). + doc META-1 (règle PATTERNS GAP-carry).
6. **`sprint75_verification.md`** — lire README **§2.3** d'abord (9 sections
   canoniques : HEAD entrée/sortie, commit stack, how to re-run, checklist
   24 rows Observed remplies, métriques, surface nouvelle, scope cuts ❌
   exhaustifs 12/12, findings carry-over G6 max 5, checkpoint).
7. **`sprint76_audit_plan.md`** — lire README **§2.4 + §4.4** d'abord
   (tracks, G1 presence, verdict attendu, out-of-scope ; router TOUS les
   P2/P3 des 6 phase reviews + les déférés consolidés, cf. §7 ci-dessous).
8. **Docs vivantes** : `docs/rust/PATTERNS.md` + `docs/shell/PATTERNS.md`
   (nouveaux patterns S75 : locator anchors.json, paire Q7, self_pin_enabled
   intent-vs-truth, adjust-state-during-render…) + `docs/claude/SPRINT_LOG.md`
   row S75 + `CLAUDE.md` (état actuel S75) + `roadmap_v5` amendé
   (S75=découverte, GPU→S76, sharding→S77 — kickoff §12).
9. **Fail-fast 24 rows COMPLET** dont row 6 = **Docker Linux canonique**
   (`sbfb-ci`, volume nommé `sbfb-ci-target`, chemin absolu pas `${PWD}` —
   memory `feedback_dual_platform`) : pas re-joué depuis S74 (Windows est à
   1750, le compte canonique Linux sera ±4 tests `#[cfg(unix)]`). C'est
   AUSSI la gate avant push.

**Garde-fous** : verrous 1-5 kickoff §4 inchangés ; pre-launch policy
(0 bump) ; pas d'emoji ; français docs/commits-body, anglais code.

## 4. Le cycle de phase (à respecter strictement)

1. **G8 preflight** : `Agent(subagent_type: "nexus-phase-preflight-deep")` →
   `sprint75_phase_g_preflight.md`. Lui passer le scope §3 + plan §G + les
   chemins réels des 4 carries S74 (vérifier validator/validator_loop/deploy/
   fork — l'état du code a pu bouger depuis S74) + la liste des rows §15 à
   écrire + l'état des 24 rows fail-fast.
2. **Code/docs** conformément au plan/preflight. L'acceptance SSH se script
   et se CONSIGNE (checklist manuelle horodatée dans verification.md).
3. **Fail-fast COMPLET dual-bloc** en `run_in_background` (les 4 fixes
   touchent du Rust → les DEUX blocs) + **Docker Linux** (row 6).
4. **Review-deep** : Workflow 5 dimensions → skeptics → synthèse
   `sprint75_phase_g_review.md`. Corrige P1 ET P2/NIT cheap-value.
5. **Codex** (gate BLOQUANTE) : `.git/CODEX_SPRINT75_PHASE_G.txt` →
   `Get-Content ... -Raw | codex exec --dangerously-bypass-approvals-and-sandbox
   -o .planning/active/sprint75_phase_g_codex_review.md`. Sortie BRUTE.
   Boucle jusqu'à `OVERALL: PASS`.
6. **Réconcilie** : review.md → `## Verdict: PASS` (header EXACT même ligne).
7. **Commit** body 9 sections (headers EXACTS de `4f52bea` : Contexte /
   Fichiers / Delta tests / Verification §7.4 / Scope cuts / G8 traceability /
   Pre-launch protocol / Codex verification / Carry closure / Unblock).
   Trim trailing-blank du codex_review → `git add` explicite →
   `git commit -F` (2 appels séparés).
8. **Memory** : `nexus_grid_pivot.md` Tip + `MEMORY.md` AVANT de rendre. Le
   sprint est alors FERMÉ (la migration des docs vers archive/v2.1 = au
   kickoff S76, pas en G). Proposer la vision sprint suivant
   (`feedback_sprint_guidance`).

## 5. Pièges process (appris A→F, IMPORTANT)

- **JAMAIS git/codex depuis `web/`** (`.git` imbriqué périmé) — racine +
  `git -C`. JAMAIS `cd web &&` chaîné : subshell `(cd web && ...)`.
- **UN SEUL cargo à la fois** ; PAS de cargo pendant un run Codex (~10-15
  min/round, il ré-exécute les suites lui-même).
- **Env réseau hôte** : `create_node` hang 90 s en masse = classe S74.
  PREUVE par stash/pop sur HEAD. Remède : reboot machine (JAMAIS
  `wsl --shutdown`). L'acceptance SSH/Docker de G y est très exposée.
- **Hook stop process-supervisor** : arbre sale ⇒ messages d'attente SANS
  les sous-chaînes fr fai-t / corri-g / termin-e / committ / propr-e /
  pre-t / liv-r (collées) ni les mots anglais done/completed/fixed/
  committed/clean/ready/final/finished. Ne JAMAIS citer la regex.
- **Lightcheck WARN connus (non bloquants)** : « missing file …ts » =
  troncature `.tsx` dans les tables du body ; « wire-format surface » se
  déclenche quand le body documente le 0-bump.
- **PIÈGE lint React** (appris F) : setState synchrone dans un effect =
  ERREUR (cascading renders) → pattern « adjust state during render ».
- **Coverage Vitest flake** (classe `vitest_env_variance`) : 1 fail
  child-process isolé au 1er run → re-run propre avant de conclure.
- **Recette prompt Codex** : PHASE BOUNDARY explicite (G vs S76), décisions
  « do NOT re-litigate » listées, SCOPE diff-only, evidence `file:NN`,
  tokens littéraux `GAP`/`CONFIRMED` + `OVERALL: PASS|FAIL` (« GAPS »
  pluriel ne matche pas le lightcheck).
- **`phase-auditor-gate`** exige `## Verdict: PASS` EXACT dans review.md.
- **Docker** : bind-mount Windows NON fidèle pour `operator_server`
  HTTP-spawn-git (16 timeouts) — canonique = le compte global, pas ces
  tests-là ; volume target nommé pour le cache.

## 6. État git + push

15 ahead local (`0e2fb6b`→`4f52bea`) + ce handoff, **RIEN POUSSÉ**. Le
**Docker Linux canonique** est la gate **AVANT PUSH** (et c'est la row 6 du
fail-fast G, donc elle tombe dans la phase). On ne pousse pas tant que le PO
ne le demande pas. NE JAMAIS faire `wsl --shutdown`.

## 7. Déférés consolidés à router dans `sprint76_audit_plan.md`

Depuis les reviews D + E + F (chaque item avec sa source) :
- **Duress gates des frères PRÉEXISTANTS** : `seed_voluntary`,
  `set_keep_online`, `reannounce_seeds_at_boot` (gap S74 ; exposition UX
  accrue par F — le P2 review F) → lot dette duress S76.
- **PULL-3** cross-tier failover (ticket mort → pas de bascule
  multi-provider ; + le call-site driver E).
- **Sampling anti-Sybil** du seeder tail (lexicographic crowding ; doc
  inline D livrée).
- **Re-drive-on-ingest** du driver one-shot (fenêtre morte premier boot ;
  remède opérateur documenté E).
- **Discriminateur curator-vs-ancre des lignes en-attente** /nodes (review
  F : `listCurators().entries` permettrait de distinguer sans wire).
- Doc `seed.rs:111-116` self-designation à réaligner si l'exemption
  same-key est un jour voulue (E).
- NIT laissés F : 404 version pour pid inconnu, asymétrie 400/404 arête
  hash-sans-ticket, truncateHex dupliqué ×4, test addAnchor mal rangé.
- **Externes inchangés** : P2-A-1 rand (exemption), P2-AUDIT-2 iroh (pin
  0.98), T-NN+2 wasm, P3-OS-1, LT-3/4/7.
- + TOUT P2/P3 des reviews A/B/C (déjà routés en partie — re-parser, §4.4).

## DÉMARRAGE

1. Lis §0 docs + memory. 2. `git log --oneline -6` + `git status -sb` (git
fait foi). 3. Cas B → phase G (dernière). 4. **G8 preflight Phase G** via
`nexus-phase-preflight-deep` (scope §3 + plan §G + chemins réels des 4
carries S74 + rows §15 + état fail-fast). 5. Attends le verdict, exécute la
Phase G. Rien n'est poussé sans demande PO.
