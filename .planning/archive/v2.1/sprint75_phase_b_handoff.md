# Handoff — Sprint 75 reprise en Phase B (à coller dans une session fraîche)

> Sprint 75 (pivot découverte PULL node-centrique + ancre VPS) est OUVERT et
> avance. **Phase A DONE.** Cette session fraîche reprend en **Phase B**. Ne
> ré-invente pas le sprint : tout est déjà décidé et écrit. Suis le workflow.

## 0. Protocole de session fraîche (obligatoire)

1. **Lis intégralement** : `docs/claude/README.md` (workflow, cycle, audit gate,
   G8 preflights, gate Codex, commit 9 sections, dual-platform, bootstrap §7.1
   routing Cas A/B/C/D) + `CLAUDE.md` + memory `MEMORY.md` +
   `nexus_grid_pivot.md` (le **Tip** « 2026-06-09 suite 2 » a tout l'état Phase A)
   + `feedback_wsl_before_push` + `feedback_dual_platform` +
   `feedback_codex_gate_strict` + `feedback_codex_raw_output`.
2. **VÉRITÉ TERRAIN = git, pas la mémoire** : lance `git log --oneline -6` +
   `git status -sb` AVANT toute décision. Au moment d'écrire ce handoff :
   HEAD = `479a87c` (Phase A), **5 ahead local, RIEN POUSSÉ**. Si le tip mémoire
   diverge, git fait foi.
3. **Routing** : le main thread est ROUTEUR. Détecte le Cas via bootstrap §7.1
   d'après `.planning/active/`. Ici : `sprint75_kickoff.md` + `sprint75_plan.md`
   présents, **pas** de `sprint75_verification.md` → **Cas B (sprint en cours)**.
   Phase suivante = **B** (Phase A committée `479a87c`).
4. **Règle modèle** : jamais le param `model` dans `Agent()`. Toujours
   `claude-opus-4-8[1m]`. Agent `nexus-phase-preflight-deep` est ENREGISTRÉ ;
   `nexus-phase-review-deep` ne l'est PAS → fallback workflow review.

## 1. Où on en est (commit stack S75)

```
479a87c feat(core+daemon): Sprint 75 Phase A — re-mint PoW + endpoint address on outbox replay
e3c3fb6 chore(planning): Sprint 75 Phase A preflight (SCOPE-CUT-CONSISTENT)
a9a2ea7 chore(planning): Sprint 75 kickoff/plan — provenance-visibility requirement
f008433 chore(planning): Sprint 75 kickoff + plan + design review + pivot proposal (Cas C)
0e2fb6b chore(planning): Sprint 75 Phase 0 — S74 audit findings (PASS)
```

- **Phase 0** (audit gate S74) = **PASS** (0 P0/0 P1/15 P2/10 P3). Aucun fix S74.
- **Phase A** (FIX-A re-mint-on-replay) = **DONE** : corrige le bug live (apps
  >30 min invisibles aux nouveaux pairs). Outbox stocke le payload non-wrappé ;
  chaque replay re-mint l'adresse + re-stampe un PoW frais ;
  `MAX_PROOF_AGE_SECS=1800` inchangé ; garde anti-hijack OWN-only. Helper
  `mint_ticket_for_hash` (runtime.rs) **réutilisé par le pull Phase C/D**.
  Gates : preflight SCOPE-CUT-CONSISTENT ; review-deep CONCERN→PASS (2 P1 fermés
  dont T1 hijack-test faux-vert réécrit 2-nœuds) ; Codex 7/8 + 1 GAP-nommage ;
  nextest `--workspace` **1682 passed 0 fail**.

## 2. Docs à lire (tout est décidé)

- `.planning/active/sprint75_kickoff.md` — D1-D5 gelées, 5 verrous
  anti-recentralisation, inventaire R4, phases A-G, scope cuts, risk register.
- `.planning/active/sprint75_plan.md` — **§Phase B détaillée** (fichiers/tests/
  acceptance/commit) + fail-fast 24 rows.
- `.planning/active/sprint75_design_review.md` — board G1 (D1✅D2✅D3⚠️D4✅D5✅).
- `.planning/active/sprint75_pivot_proposal.md` — frontière D3 (l'annuaire
  `NodeDirectoryEntry` ≠ SearchManifest différé). **3 sign-offs PO OBTENUS.**
- `.planning/active/sprint75_phase_a_{preflight,review,codex_review}.md` — exemple
  du process Phase A (à imiter pour B).
- `.planning/research/s73_searchmanifest_index_node_design.md` — design D3 différé.

## 3. Phase B — scope (depuis le plan §Phase B)

**Livrer** `NodeDirectoryEntry` + son domaine + le write-path d'authoring
(primitive 1, chemin critique). D1 gelée : nouveau type signé sibling sous
**`DOMAIN_NODE_DIRECTORY_V1`** réutilisant *verbatim* la machinerie `CuratorList`
(sign/verify Ed25519+JCS, revision monotone, caps 256+par-champ, attention-set,
ingest gossip 9-étapes subscription-gated, read-path BrowseAggregator). Porte
`node_id` + `Vec<{project_id, archive_hash, name, category, description}>` +
`revision`. Payload = liste **humainement affichable** (forme F-Droid), PAS un
digest Bloom.

Fichiers (plan §B.2) : NEW `crates/nexus-core-rs/src/node_directory.rs` ;
`canonical.rs` `DOMAIN_NODE_DIRECTORY_V1` (copier le précédent S74
`DOMAIN_SEED_REQUEST_V1` `:201-219`) ; `lib.rs` re-export ; `http.rs` route
`POST /api/daemon/directory/publish` (build+sign+blob-store+gossip-announce le
catalogue OWN, auth loopback) ; `iroh_runtime.rs` **helper générique
`ingest_signed_list<T: SignedList>`** factorisant le gate
subscription/cap/revision (mitigation drift C1/Q2). Tests B.3 : sign/verify
round-trip, cross-domain replay rejeté (miroir `curator.rs:589-602`), caps,
revision monotone, route authoring signe+annonce, parité helper générique.

**Garde-fous (kickoff §4)** : nouveau DOMAIN disjoint (JAMAIS réutiliser
`DOMAIN_CURATOR_LIST_V1`/`DOMAIN_SEED_REQUEST_V1`) ; 0 bump `*_FORMAT_VERSION`
(additif pre-launch, pattern SeedRequest) ; entrée node_id == signing pubkey ==
author (anti-impersonation) ; ancre VPS jamais hard-codée dans le binaire
(`default_curators` vide par défaut). Triade anti-Sybil voyage avec tout annuaire.

## 4. Le cycle de phase (à respecter strictement)

Pour Phase B (et chaque phase) :
1. **G8 preflight** : `Agent(subagent_type: "nexus-phase-preflight-deep")` pour
   Phase B → `sprint75_phase_b_preflight.md` (verdict EXECUTE/PLAN-ADAPT/
   SCOPE-CUT-CONSISTENT/DESIGN-CONFLICT). Commit-le en `chore(planning): Sprint 75
   Phase B preflight (...)` (chore n'arme PAS le gate lourd) OU bundle dans le feat.
2. **Code** la phase conformément au plan/preflight.
3. **Fail-fast Windows** : `cargo fmt --all --check` + `clippy --workspace
   --all-targets --locked -- -D warnings` + `nextest run --workspace --locked` +
   `test --workspace --doc` + `build -p nexus-shell-daemon --release` + (web si
   touché). En background (`run_in_background`).
4. **Review-deep** : workflow adversarial 5 dim (correctness/security/tests/
   wire-scope/patterns) → skeptics sur P0/P1 → synthèse. Écris
   `sprint75_phase_b_review.md` verdict PASS-PENDING. Corrige les P1.
5. **Codex** (gate BLOQUANTE, GPT-5.5) : prompt `.git/CODEX_SPRINT75_PHASE_B.txt`
   (liste les livrables) → `Get-Content ... -Raw | codex exec
   --dangerously-bypass-approvals-and-sandbox -o
   .planning/active/sprint75_phase_b_codex_review.md`. Sortie BRUTE, jamais
   réécrite. Triage GAPs (corrige réels, documente faux-positifs).
6. **Réconcilie** : promeus `review.md` à `## Verdict: PASS` + `## Codex
   reconciliation`.
7. **Commit** `feat(scope): Sprint 75 Phase B — ...` body 9 sections (template
   `.claude/templates/commit_body_phase.txt`, **header `## Scope cuts` NU**).
8. **Memory** : update `nexus_grid_pivot.md` Tip + `MEMORY.md` AVANT de rendre.

## 5. Pièges process appris cette session (IMPORTANT)

- **Stop-hook process-supervisor** : bloque toute fin de tour si l'arbre est sale
  ET le message contient un mot de finalité (`fait`/`corrig`/`propre`/`pret`/
  `livr`/`done`/`clean`/`ready`/`final`/`fixed`/`finished`/`committed`/`termine`).
  Pendant un travail en cours (arbre sale, attente background), écris des messages
  **finality-free**. NE liste PAS ces mots non plus (le quote déclenche aussi).
- **PreToolUse hook `phase-precommit-lightcheck`** : intercepte tout bloc Bash
  contenant `git commit` et inspecte l'état **staged AVANT exécution**. Donc
  sépare `trim/git add` du `git commit` en 2 appels Bash distincts.
- **`git diff --cached --check`** bloque les trailing-blank-EOF du
  `codex_review.md`. Trim la dernière ligne blanche (whitespace, PAS le contenu
  brut Codex) : `python -c "p='...';s=open(p).read();open(p,'w',newline='\n').
  write(s.rstrip()+'\n')"` puis re-`git add`.
- **Lightcheck commit body** : teste AVANT commit via `python
  scripts/agent/agentctl.py precommit-lightcheck --scope message --message-file
  <body>`. Header EXACT `## Scope cuts` (nu). Les WARN « wire-format surface » sont
  des faux-positifs si le body *mentionne* les versions (0 bump réel).
- **`chore`** dans le titre n'arme PAS le gate lourd (Check 5/7/8/9), même avec
  « Phase X » ; **`feat`/`fix`/`docs`/`test`/`refactor` + « Sprint N Phase X »**
  l'arme (exige preflight+review PASS+codex+9 sections).
- **Background obligatoire** : toutes les vérifs lourdes (clippy/nextest/build)
  en `run_in_background` (feedback_background_checks).

## 6. État git + push

5 ahead local (`0e2fb6b`→`479a87c` + ce handoff), **RIEN POUSSÉ**. Le **Docker
Linux canonique** (`rust:1.94` / `sbfb-ci:latest`) est le gate **AVANT PUSH**
uniquement (`feedback_wsl_before_push` : Docker avant PUSH, pas avant commit). On
ne pousse pas tant que le PO ne le demande pas. Env récupéré (Docker up) ; NE
JAMAIS faire `wsl --shutdown`.

## 7. Carries + phases suivantes

- **Phases C-G** (plan) : C ingest annuaire + durabilité catalogue distant
  (absorbe carries S74 WIRE-1/WIRE-2/DBQ-1) ; D pull multi-provider + node
  identity (carry PULL-2) ; E ancre VPS headless ; F front node-Browse
  (`/nodes` + `/node/:id`, **provenance visible + fork marqué** = exigence PO,
  verrou 4) ; G wrap-up + acceptance **survives-VPS-death cross-machine** (SSH
  mac `192.168.1.53` + vps `135.181.42.188`).
- **Décisions PO actées** : sprint complet A-G ; acceptance live SSH ; provenance
  visible + fork distinctement marqué (Ed25519 `provenance.json` + BLAKE3 + verrou
  4 seeder≠auteur déjà structurels, à AFFICHER en Phase F).
- **Carries P2 Phase A → audit S76** : T6 (test direct broadcast handler
  GossipCmd::Outbox), WS-3 (hoist `my_endpoint_addr()` once-per-replay-pass).
- **15 P2 audit S74** (`sprint74_audit_findings.md`, archive/v2.1/) dont 8 à
  concevoir DANS le pivot (déjà mappés aux phases C/D).

## DÉMARRAGE

1. Lis §0 docs + memory. 2. `git log --oneline -6` + `git status -sb` (git fait
foi). 3. Cas B détecté → phase B. 4. **G8 preflight Phase B** via
`nexus-phase-preflight-deep` (lui passer le scope §3 + plan §Phase B + D1 gelée +
garde-fous). 5. Attends le verdict, puis code Phase B. Rien n'est poussé sans
demande PO.
