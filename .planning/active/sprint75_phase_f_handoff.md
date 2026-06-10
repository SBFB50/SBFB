# Handoff — Sprint 75 reprise en Phase F (à coller dans une session fraîche)

> Sprint 75 (pivot découverte PULL node-centrique + ancre VPS) est OUVERT et
> avance. **Phases A + B + C + D + E DONE.** Cette session fraîche reprend en
> **Phase F (FRONT node-Browse)**. Ne ré-invente pas le sprint : tout est déjà
> décidé et écrit. Suis le workflow.

## 0. Protocole de session fraîche (obligatoire)

1. **Lis intégralement** : `docs/claude/README.md` (workflow, cycle, audit gate,
   G8 preflights, gate Codex, commit 9 sections, dual-platform, bootstrap §7.1
   routing A/B/C/D) + `CLAUDE.md` + memory `MEMORY.md` + `nexus_grid_pivot.md`
   (le **Tip « 2026-06-10 suite 6 »** a tout l'état Phase E) +
   `feedback_wsl_before_push` + `feedback_dual_platform` +
   `feedback_codex_gate_strict` + `feedback_codex_raw_output` +
   `feedback_full_failfast` + `feedback_background_checks` + `feedback_cd_web_trap`
   + `nested_git_web_trap` (CRUCIAL : phase front).
2. **VÉRITÉ TERRAIN = git, pas la mémoire** : `git log --oneline -6` +
   `git status -sb` AVANT toute décision. Au moment d'écrire ce handoff :
   HEAD = `1486fc9` (Phase E), **13 ahead local (+1 avec ce handoff), RIEN
   POUSSÉ**. Si le tip mémoire diverge, git fait foi.
3. **Routing** : main thread = ROUTEUR. `.planning/active/` contient
   `sprint75_kickoff.md` + `sprint75_plan.md` + les preflight/review/codex_review
   des phases A-E, **pas** de `sprint75_verification.md` → **Cas B (sprint en
   cours)**. Phases A-E committées → phase suivante = **F**.
4. **Règle modèle** : jamais le param `model` dans `Agent()`.
   `nexus-phase-preflight-deep` est ENREGISTRÉ ; `nexus-phase-review-deep` ne
   l'est PAS → fallback review par Workflow multi-agent (pattern C/D/E :
   5 dimensions adversariales → skeptics refute-by-default sur P0/P1 →
   synthèse ; en E il a sorti 2 P1 réels [systemd SBFB_HOME + duress driver],
   ça marche très bien).

## 1. Où on en est (commit stack S75)

```
1486fc9 feat(daemon): Sprint 75 Phase E — headless VPS anchor (config-driven seed driver + signed authoring)
41b13e3 docs(planning): handoff prompt for the next session (S75 Phase E)
0010450 feat(core+daemon): Sprint 75 Phase D — multi-provider pull + node identity exposure
9f7de7f docs(planning): handoff prompt for the next session (S75 Phase D)
821aa8c feat(daemon): Sprint 75 Phase C — node-directory ingest + remote-catalog durability (boot re-pull)
f6637d3 feat(core+daemon): Sprint 75 Phase B — NodeDirectoryEntry + DOMAIN_NODE_DIRECTORY_V1 + generic ingest gate + authoring route
479a87c feat(core+daemon): Sprint 75 Phase A — re-mint PoW + endpoint address on outbox replay
0e2fb6b chore(planning): Sprint 75 Phase 0 — S74 audit findings (PASS)
```

- **Phase E** = DONE `1486fc9` (l'ancre VPS headless) :
  - Config `[seed] keep_online_projects` dans **nexus-shell-daemon-core**
    (défaut VIDE verrou 3, clamp lowercase-64hex). **Q3 : PAS de section
    `[directory]`** — l'abonnement ancre passe par `default_curators` (UN
    attention set, DQ3).
  - Driver boot one-shot duress-gaté (résolution direct > row M18 > annuaires,
    first-applicable-only, 120 s/app, prédicat `seed_already_announced`).
    Handle RETENU + abort+join au shutdown (GAP Codex R1).
  - Re-annonce PRODUCTEUR boot state-driven (revision > 0) via le cœur partagé
    `build_sign_announce_directory` (carry P2 review C CLOSED).
  - Route `POST /api/daemon/seed/request` = 1er appelant prod de `request_seed`
    (REQUESTER, invite M19 TOUJOURS requise, self-guard parsé, mint
    gate-détention 409, écho nonce).
  - `deploy/nexus-shell-daemon.service` durcie (SBFB_HOME + NEXUS_GRID_ROOT
    épinglés, AF_NETLINK, @system-service) + `config.toml.example` [seed].
  - Gates : preflight SCOPE-CUT-CONSISTENT (5 deltas plan) ; review Workflow
    9 agents (2 P1 skeptics-confirmés + 17 fixes in-phase, 5 déférés scopés) ;
    **Codex R1 28 CONFIRMED + 1 GAP → fix → R2 19 CONFIRMED OVERALL PASS**.
  - Compteurs : nextest --workspace **1748** 0-fail 0-skip (1735→+13) ; web
    Vitest **334** INCHANGÉ (0 fichier web en E), coverage
    86.94/78.73/85.82/88.25, size 6/6.

## 2. Docs à lire (tout est décidé)

- `.planning/active/sprint75_plan.md` — **§Phase F détaillée** (F.1-F.5 :
  fichiers/tests/acceptance/commit) + fail-fast 24 rows.
- `.planning/active/sprint75_kickoff.md` — §4 verrous (le 4 = exigence PO
  provenance), §10 Q6/Q7, §9 scope cuts.
- `.planning/active/sprint75_phase_d_review.md` + `sprint75_phase_e_review.md`
  — les **déférés routés vers F** (cf. §3) et vers S76/G.
- `.git/CODEX_SPRINT75_PHASE_E.txt` — RÉUTILISE sa structure pour le prompt
  Codex Phase F : Output contract (tokens littéraux GAP/CONFIRMED + `OVERALL:
  PASS/FAIL`), SCOPE explicite (cette fois `web/` EST le diff principal —
  scoper `git diff -- web crates` et exclure `.planning/`/`.git/`), PHASE
  BOUNDARY (juger F, pas G), et les décisions design à ne pas re-litiger.
  E a pris 2 rounds (1 GAP réel) ; D avait passé en 1 round.

## 3. Phase F — scope (plan §F, kickoff, déférés D/E routés vers F)

**Browse node-centrique (front).** Le shell rend la découverte PULL : liste de
nœuds → catalogue d'un nœud → download/seed.

- **Pages** : `/nodes` (liste des catalogue-publishers découverts) +
  `/node/:nodeId` (catalogue d'un nœud → pull). `web/src/App.tsx` routes lazy ;
  `web/src/pages/Nodes.tsx` (NEW) ; `web/src/pages/NodeCatalog.tsx` (NEW).
- **API client** (`web/src/api/daemon.ts`) : `listNodes` sur
  `GET /api/daemon/nodes` — enveloppe `{nodes:[{node_id,revision,app_count,
  catalog}]}`. **Contrat Zod (déféré D, IMPÉRATIF) : `.strict()` sur
  l'ENVELOPPE `{nodes}`, PAS sur les rows catalog** (sinon le premier ajout
  additif 0-bump côté Rust brique la page) — ou projection HTTP dédiée.
- **AddAnchorDialog** (NEW, template `Curators.tsx`) : intention « ajouter une
  ancre » = subscribe au pubkey (l'ancre EST une subscription curator, Q3/DQ3).
  UX cold-start 1er-run (C4 : pas d'écran vide mort).
- **Browse.tsx** : cohabitation/supersede node-Browse vs grille (Q6 — design
  intention UX à trancher au preflight) + `known_browse_entries` honnête.
- **Exigence PO verrou-4 (critère d'acceptation)** : chaque carte du catalogue
  AFFICHE la preuve de provenance (signature auteur `provenance.json`
  commit→hash via `VerificationDetail` existant + identité BLAKE3) ; une app
  **forkée/modifiée** (`is_open_source=false`, hash distinct) porte un
  **marqueur « version dérivée »** non ambigu, jamais le badge de l'original ;
  le nœud seeder n'est JAMAIS rendu comme autorité.
- **Badge Q7 « joignable-via-seeder » visible** (déféré D→F) : nouveau variant
  `BrowseStatus` (`reachableviaseeder` lowercase serde) + Zod + rendu. **C'est
  LE changement wire de `/browse` assumé en F** (D l'a différé précisément pour
  que le bump de bytes arrive avec son consommateur Zod dans la MÊME phase) —
  producteur aggregate/daemon + Zod + rendu + web fail-fast COMPLET ensemble.
  Le signal backend existe : row unreachable + seed-count version-exact > 0
  (le daemon HTTP layer détient SeedRegistry, pas l'aggregator core — seam
  documenté preflight D).
- **AvailabilitySheet** : intégration node-centrique + **WEB-1** (toggle seed
  reconcilié depuis `selfSeeding`).
- **`SeedVoluntaryRequest.archive_hash` optionnel** (déféré D→F) :
  discriminateur quand le CTA front câble le seed volontaire (collision
  multi-ancres project_id first-match).
- **Strings FR** (scan-en-strings) ; lock-1 (0 champ cible/hôte au publish).

Tests plan §F.3 (Vitest) : `Nodes` rendu + empty/cold-start, `NodeCatalog`
pull, `AddAnchorDialog`, schémas `.strict()` enveloppe, WEB-1 toggle, lock-1,
**lock-4 provenance (a) signature auteur affichée (b) fork « version dérivée »
pas le badge original (c) seeder jamais autorité**, badge Q7.
Commit cible : `feat(shell): Sprint 75 Phase F — node-centric Browse (nodes
list + node catalog + add-anchor)`. Body : Carry closure (WEB-1 CLOSED,
badge Q7 CLOSED, archive_hash discriminateur CLOSED), lock-1/2/4 vérifiés UI.

**Garde-fous (kickoff §4)** : verrou 1 = l'annuaire est read-side, jamais un
sélecteur de destination ; verrou 2 = node-Browse additif/sur-ensemble, jamais
substitutif silencieux ; verrou 4 = provenance auteur, cf. exigence PO.

## 4. Le cycle de phase (à respecter strictement)

1. **G8 preflight** : `Agent(subagent_type: "nexus-phase-preflight-deep")` →
   `sprint75_phase_f_preflight.md`. Lui passer le scope §3 + plan §Phase F +
   Q6 à trancher + les déférés D/E routés F + le contrat /nodes (enveloppe
   pinnée par `nodes_response_pins_envelope_and_grouping`) + le seam
   SeedRegistry/daemon-layer pour le badge Q7 + les régions front
   (`App.tsx` lazy routes, `Curators.tsx` template, `AvailabilitySheet.tsx`,
   `Browse.tsx`, `daemon.ts` pattern Zod `.strict()` enveloppe S73-E).
2. **Code** conformément au plan/preflight.
3. **Fail-fast COMPLET** en `run_in_background` : web COMPLET (lint, tsc,
   test:unit, test:coverage, build, size, scan FR) ET Rust complet (fmt,
   clippy --workspace --all-targets, nextest --workspace, doctests, release)
   — le badge Q7 touche `browse.rs` (variant serde) donc les DEUX blocs.
4. **Review-deep** : Workflow 5 dimensions adversariales → skeptics (REFUTER,
   défaut isReal=false) → synthèse `sprint75_phase_f_review.md`. Corrige P1 ET
   les P2/NIT cheap-value (norme anti-faux-vert).
5. **Codex** (gate BLOQUANTE, GPT-5.5) : prompt `.git/CODEX_SPRINT75_PHASE_F.txt`
   (structure du E !) → `Get-Content ... -Raw | codex exec
   --dangerously-bypass-approvals-and-sandbox -o
   .planning/active/sprint75_phase_f_codex_review.md`. **Sortie BRUTE.** Si
   GAP → corrige + fail-fast re-vert + round suivant jusqu'à `OVERALL: PASS`.
6. **Réconcilie** : review.md → `## Verdict: PASS` (header EXACT, même ligne).
7. **Commit** `feat(shell): Sprint 75 Phase F — ...` body 9 sections (headers
   EXACTS de `1486fc9` : Contexte / Fichiers / Delta tests / Verification §7.4
   / Scope cuts / G8 traceability / Pre-launch protocol / Codex verification /
   Carry closure / Unblock). Séquence : trim trailing-blank du codex_review →
   `git add` explicite → `git commit -F` (2 appels séparés).
8. **Memory** : `nexus_grid_pivot.md` Tip + `MEMORY.md` AVANT de rendre.

## 5. Pièges process (appris A→E, IMPORTANT)

- **JAMAIS git/codex depuis `web/`** (`.git` imbriqué périmé pré-pivot) —
  racine + `git -C`. JAMAIS `cd web &&` chaîné : subshell `(cd web && ...)`
  ou `npm --prefix web`. Le cwd persiste après un `cd web` → piège. Phase
  FRONT = exposition maximale à ce piège.
- **UN SEUL cargo à la fois** : 2 cargo parallèles = course sur le binaire de
  test (os error 5) + faux TIMEOUT massifs (le 2e run recompile les .exe SOUS
  le nextest en cours d'exécution). Et PAS de cargo pendant un run Codex
  (~10-15 min/round, il ré-exécute la suite lui-même).
- **Env réseau hôte** : si des tests `create_node` triviaux timeout à 90 s en
  masse → classe S74. PREUVE par stash/pop sur HEAD avant de chercher dans le
  diff. Remède : reboot machine (jamais `wsl --shutdown`).
- **Hook stop process-supervisor** : tant que l'arbre est sale, les messages
  de fin de tour ne doivent contenir AUCUNE des sous-chaînes françaises
  fai-t / corri-g / termin-e / commit-t / propr-e / pre-t / liv-r (collées,
  sans tiret) ni les mots anglais done/completed/fixed/committed/clean/ready/
  final/finished. Ne JAMAIS citer la regex dans le message (auto-match).
- **Coverage Vitest flake** (classe `vitest_env_variance`) : un fail
  child-process isolé au 1er run coverage → re-run propre avant de conclure.
- **La recette prompt Codex** : PHASE BOUNDARY explicite (F vs G), décisions
  design listées « do NOT re-litigate », SCOPE diff-only, evidence attendue
  par livrable, tokens littéraux `GAP`/`CONFIRMED` + `OVERALL: PASS|FAIL` +
  `file:NN` (« GAPS » pluriel ne matche pas le lightcheck).
- **`phase-auditor-gate`** exige `## Verdict: PASS` EXACT dans review.md.
  Lightcheck WARN « wire-format surface » = faux positif connu quand le body
  mentionne le bump/0-bump (non bloquant).
- **Zod `.nullable()` vs `.optional()`** (leçon S73-E) : le Rust sérialise
  TOUJOURS la clé (null) → `.nullable()`. Enveloppes, jamais bare arrays.
- **Wire /browse en F** : le variant Q7 est un VRAI changement de bytes —
  pre-launch policy l'autorise (0 deploiement tiers), mais producteur + Zod +
  rendu + les DEUX fail-fast doivent atterrir dans le MÊME commit.

## 6. État git + push

13 ahead local (`0e2fb6b`→`1486fc9`) + ce handoff, **RIEN POUSSÉ**. Le
**Docker Linux canonique** (`sbfb-ci`) est le gate **AVANT PUSH** uniquement
(`feedback_wsl_before_push`). On ne pousse pas tant que le PO ne le demande
pas. NE JAMAIS faire `wsl --shutdown`.

## 7. Carries + phase suivante

- **Phase G (dernière)** : wrap-up + acceptance **survives-VPS-death**
  cross-machine (SSH mac `192.168.1.53` + vps `135.181.42.188`, l'unit
  systemd E à valider LIVE : boot stock Debian/Ubuntu, `systemd-analyze
  security`, bind QUIC sous seccomp) + C6 E2E + hygiène carries S74 (CARRY-5
  clamp, CARRY-2 Rejected-terminal, PULL-1 dedup, FORK-1 entry-cap) +
  **THREAT_MODEL §15 rows déférés D/E** (directory pull route publique
  blob-serve, /nodes, SEED-1/2, fresh-flood, boot driver + requester route) +
  LT-2 ARMÉ (flipper docs + dry-run Radicle privé) + `sprint75_verification.md`
  (fail-fast §5 rempli) + `sprint76_audit_plan.md` + PATTERNS rust+shell +
  SPRINT_LOG + CLAUDE.md + roadmap_v5 amendé.
- **Carries → audit S76** (consolidés D+E) : PULL-3 cross-tier failover (+ le
  call-site driver E, doc inline posée) ; sampling anti-Sybil du seeder tail
  (lexicographic crowding) ; re-drive-on-ingest du driver one-shot (fenêtre
  morte premier boot, remède opérateur documenté) ; duress gates des frères
  PRÉEXISTANTS S74 (`seed_voluntary`, `reannounce_seeds_at_boot`) ; doc
  `seed.rs:111-116` self-designation à réaligner si l'exemption same-key est
  un jour voulue ; T6/WS-3 et le reste des P2 routés (cf. reviews D/E).

## DÉMARRAGE

1. Lis §0 docs + memory. 2. `git log --oneline -6` + `git status -sb` (git
fait foi). 3. Cas B → phase F. 4. **G8 preflight Phase F** via
`nexus-phase-preflight-deep` (scope §3 + plan §Phase F + Q6 + déférés D/E
routés F + contrat /nodes + seam Q7). 5. Attends le verdict, code Phase F.
Rien n'est poussé sans demande PO.
