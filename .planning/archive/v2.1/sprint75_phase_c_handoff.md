# Handoff — Sprint 75 reprise en Phase C (à coller dans une session fraîche)

> Sprint 75 (pivot découverte PULL node-centrique + ancre VPS) est OUVERT et
> avance. **Phases A + B DONE.** Cette session fraîche reprend en **Phase C**. Ne
> ré-invente pas le sprint : tout est déjà décidé et écrit. Suis le workflow.

## 0. Protocole de session fraîche (obligatoire)

1. **Lis intégralement** : `docs/claude/README.md` (workflow, cycle, audit gate,
   G8 preflights, gate Codex, commit 9 sections, dual-platform, bootstrap §7.1
   routing A/B/C/D) + `CLAUDE.md` + memory `MEMORY.md` + `nexus_grid_pivot.md`
   (le **Tip « 2026-06-09 suite 3 »** a tout l'état Phase B) + `feedback_wsl_before_push`
   + `feedback_dual_platform` + `feedback_codex_gate_strict` + `feedback_codex_raw_output`
   + `feedback_full_failfast` + `feedback_background_checks`.
2. **VÉRITÉ TERRAIN = git, pas la mémoire** : `git log --oneline -6` + `git status -sb`
   AVANT toute décision. Au moment d'écrire ce handoff : HEAD = `f6637d3` (Phase B),
   **7 ahead local, RIEN POUSSÉ**. Si le tip mémoire diverge, git fait foi.
3. **Routing** : main thread = ROUTEUR. `.planning/active/` contient
   `sprint75_kickoff.md` + `sprint75_plan.md` + les preflight/review/codex_review
   des phases A/B, **pas** de `sprint75_verification.md` → **Cas B (sprint en cours)**.
   Phases A+B committées → phase suivante = **C**.
4. **Règle modèle** : jamais le param `model` dans `Agent()`. Toujours
   `claude-opus-4-8[1m]`. `nexus-phase-preflight-deep` est ENREGISTRÉ ;
   `nexus-phase-review-deep` ne l'est PAS → fallback workflow review.

## 1. Où on en est (commit stack S75)

```
f6637d3 feat(core+daemon): Sprint 75 Phase B — NodeDirectoryEntry + DOMAIN_NODE_DIRECTORY_V1 + generic ingest gate + authoring route
96943b7 docs(planning): handoff prompt for the next session (S75 continuation)
479a87c feat(core+daemon): Sprint 75 Phase A — re-mint PoW + endpoint address on outbox replay
e3c3fb6 chore(planning): Sprint 75 Phase A preflight (SCOPE-CUT-CONSISTENT)
a9a2ea7 chore(planning): Sprint 75 kickoff/plan — provenance-visibility requirement
f008433 chore(planning): Sprint 75 kickoff + plan + design review + pivot proposal (Cas C)
0e2fb6b chore(planning): Sprint 75 Phase 0 — S74 audit findings (PASS)
```

- **Phase A** (FIX-A re-mint) = DONE. Helper `mint_ticket_for_hash` (runtime.rs)
  **réutilisé par le pull Phase C/D**.
- **Phase B** (NodeDirectoryEntry + gate ingest + authoring) = DONE `f6637d3` :
  type signé sibling sous `DOMAIN_NODE_DIRECTORY_V1` (machinerie CuratorList
  verbatim) ; **trait `SignedList`** + **helper générique
  `verify_signed_list_ingest<T: SignedList>`** (bras curator `process_announcement_bytes`
  refactoré dessus, behavior-preserving gardé par `two_nodes_reject_*`) ;
  `own_entries` (browse.rs) ; route `POST /api/daemon/directory/publish`.
  Sécurité durcie via review+Codex : `static REVISION_LOCK` ; `is_valid_archive_hash`
  (64-hex minuscule au sign ET verify) ; discriminateur `is_node_directory_announcement`
  = directory ET PAS curator ; **garde anti-spoof gossip `announcement_claims_own_node_id`**
  (drop des annonces LIVE forgeant notre node_id, boot-restore non affecté) ;
  cap MAX_ENTRIES ; garde blob-presence `blobs.has`. **+32 tests, nextest
  --workspace 1682→1714 0-fail.** Gates : preflight EXECUTE + review 9 agents
  (3 P1 fermés) + **Codex 4 rounds OVERALL PASS**.

## 2. Docs à lire (tout est décidé)

- `.planning/active/sprint75_plan.md` — **§Phase C détaillée** (C.1-C.5 :
  fichiers/tests/acceptance/commit) + dépendances inter-phases + fail-fast 24 rows.
- `.planning/active/sprint75_kickoff.md` — D1-D5 gelées (§5), 5 verrous
  anti-recentralisation (§4), inventaire R4 (§6), carries 15 P2 (§8), scope cuts (§9).
- `.planning/active/sprint75_phase_b_{preflight,review,codex_review}.md` — exemple
  du process (à imiter pour C). Le `codex_review.md` = round 4 brut (OVERALL PASS).
- `.planning/research/s73_searchmanifest_index_node_design.md` — design D3 différé.

## 3. Phase C — scope (depuis le plan §Phase C, kickoff D4)

**Livrer le seul vrai trou archi (R4 load-bearing)** : ingest annuaire + durabilité
catalogue distant.
- **Sibling ingest arm** pour `NodeDirectoryEntry` via le **helper générique B**
  (`verify_signed_list_ingest`), subscription-gated (réutilise le gate
  attention-set/cap/revision). NE PAS dupliquer le bras curator — c'est tout
  l'intérêt du helper livré en B (mitigation drift R1).
- **`BrowseSource::NodeDirectory`** + branche aggregator settant `node_id` depuis
  l'entrée (aujourd'hui forcé `None` `browse.rs:632-634` ; un-skip ou view node_id).
- **Re-pull boot** des `NodeDirectoryEntry` des ancres abonnées (itérer les pubkeys
  d'ancre, re-fetch leurs blobs — réutilise path curator gossip+blob + helper
  re-mint A). **Le gap load-bearing** : `direct_entries` in-memory + restore
  OWN-only ne survivent pas au reboot pour les catalogues distants.
- **Absorbe 3 carries S74 dans le schéma** : WIRE-1 (indexer `ReleasePublished`
  par nom — `public_feed.rs` + `search.rs extract_index_fields` lit `category`),
  WIRE-2 (seed-count keyé (project_id, archive_hash) — `seed_registry.rs`), DBQ-1
  (`set_keep_online` coalesce l'archive_hash, lit M18 pas l'aggregator volatile).
- **Gate C6** : Phase A E2E cross-machine validée AVANT de gater le pull dessus.

Fichiers (plan §C.2) : `iroh_runtime.rs` (sibling ingest arm via helper B) ;
`browse.rs` (`BrowseSource::NodeDirectory` + aggregator node_id) ; `runtime.rs`
(re-pull boot) ; `public_feed.rs` + `search.rs` (WIRE-1) ; `seed_registry.rs`
(WIRE-2) ; `db.rs` (DBQ-1). Tests §C.3 : ingest subscription-gated, aggregator
node_id depuis directory, **boot_repull_restores_remote_catalogs** (le gap),
release_published_searchable_by_name, seed_count keyé, set_keep_online coalesce.

**Réutilise depuis B** : le côté RÉCEPTION de `NodeDirectoryAnnouncement` existe
déjà comme **drop-at-debug** dans le dispatch gossip (`runtime.rs`,
`is_node_directory_announcement`) — Phase C remplace ce drop par le **vrai ingest
arm** (fetch blob + `verify_signed_list_ingest` + store + aggregator). C'est le
point d'entrée naturel.

**Garde-fous (kickoff §4)** : ingest subscription-gated (jamais une ancre non
abonnée) ; triade anti-Sybil ; provenance = auteur jamais seeder ; ancre jamais
hard-codée (`default_curators`/`default_anchors` vides). RAM-only + re-pull
(persister les node_ids d'ancre, PAS les entrées distantes — invite over-count).

## 4. Le cycle de phase (à respecter strictement)

1. **G8 preflight** : `Agent(subagent_type: "nexus-phase-preflight-deep")` →
   `sprint75_phase_c_preflight.md` (verdict EXECUTE/PLAN-ADAPT/SCOPE-CUT-CONSISTENT/
   DESIGN-CONFLICT). Lui passer le scope §3 + plan §Phase C + D4 gelée + garde-fous.
2. **Code** conformément au plan/preflight.
3. **Fail-fast Windows COMPLET** (feedback_full_failfast, en `run_in_background`) :
   `cargo fmt --all --check` + `clippy --workspace --all-targets --locked -- -D warnings`
   + `nextest run --workspace --locked` + `test --workspace --doc` + `build -p
   nexus-shell-daemon --release` + web SI touché (Phase C est probablement pur Rust).
4. **Review-deep** : workflow adversarial 5 dim → skeptics P0/P1 → synthèse
   `sprint75_phase_c_review.md`. Corrige les P1.
5. **Codex** (gate BLOQUANTE, GPT-5.5) : prompt `.git/CODEX_SPRINT75_PHASE_C.txt`
   → `Get-Content ... -Raw | codex exec --dangerously-bypass-approvals-and-sandbox
   -o .planning/active/sprint75_phase_c_codex_review.md`. **Sortie BRUTE, jamais
   réécrite.** Si GAP → corrige + **re-run round suivant** jusqu'à `OVERALL: PASS`.
6. **Réconcilie** : review.md → `## Verdict: PASS` (header EXACT, PASS même ligne).
7. **Commit** `feat(scope): Sprint 75 Phase C — ...` body 9 sections (template
   `.claude/templates/commit_body_phase.txt`, header `## Scope cuts` NU).
8. **Memory** : `nexus_grid_pivot.md` Tip + `MEMORY.md` AVANT de rendre.

## 5. Pièges process appris en Phase B (IMPORTANT)

- **`phase-auditor-gate`** (hook commit) exige le verdict review EXACTEMENT
  `## Verdict: PASS` (PASS sur la MÊME ligne que le header — pas `## Verdict\n\n**PASS**`).
- **Lightcheck Codex** (`agentctl.py precommit-lightcheck`) exige dans le
  `codex_review.md` un marker `\bGAP\b`/`CONFIRMED`/`CONFIRME`/`PARTIEL` ET une
  evidence `file.rs:NN`. **« GAPS » (pluriel) NE matche PAS `\bGAP\b`** → demander
  à Codex d'émettre les tokens littéraux `GAP`/`CONFIRMED` dans le prompt, ou
  re-run un round avec consigne de markers. Teste AVANT commit :
  `python scripts/agent/agentctl.py precommit-lightcheck --scope message
  --message-file <body>` (EXIT 0 = OK ; les WARN « wire-format surface » sont des
  faux-positifs si le body mentionne le 0-bump).
- **Codex multi-rounds** : après un fix, re-run Codex (round N+1) sur le diff
  corrigé jusqu'à `OVERALL: PASS` ; l'artefact `codex_review.md` = le DERNIER round
  (PASS). L'historique des GAPs vit dans le commit body + `review.md`.
- **`nexus-launcher` rustc `STATUS_STACK_BUFFER_OVERRUN` (0xc0000409)** au build du
  binaire de test = crash COMPILO transitoire (deps GUI tray_icon/muda/png +
  resource.res), crate non touchée — **re-run** `nextest --workspace`, ça passe.
- **Pipes masquent l'exit code** : `cargo ... | tail` rapporte l'exit de `tail`
  (0), pas du cargo. **Inspecter le CONTENU** (markers `===*_DONE===`, « N passed »),
  pas le « exit code 0 » de la notification background.
- **Stop-hook process-supervisor** : bloque la fin de tour si l'arbre est sale ET
  le message contient un mot de finalité (`fait`/`corrig`/`propre`/`final`/`done`/
  `clean`/`ready`/`fixed`/`committed`/`termine`…). Pendant le travail, messages
  **finality-free** (ne liste pas ces mots non plus).
- **`git add` séparé du `git commit`** en 2 appels Bash (le hook lightcheck
  inspecte le staged AVANT exécution). Trim le trailing-blank du `codex_review.md`
  (`git diff --cached --check` le bloque) :
  `python -c "p='...';s=open(p,encoding='utf-8').read();open(p,'w',encoding='utf-8',newline='\n').write(s.rstrip()+'\n')"`.
- **Toutes les vérifs lourdes en `run_in_background`** (feedback_background_checks) ;
  Codex tient le build-lock cargo → ne pas lancer un nextest concurrent (sérialise).

## 6. État git + push

7 ahead local (`0e2fb6b`→`f6637d3` + ce handoff), **RIEN POUSSÉ**. Le **Docker
Linux canonique** (`rust:1.94` / `sbfb-ci:latest`) est le gate **AVANT PUSH**
uniquement (`feedback_wsl_before_push` : Docker avant PUSH, pas avant commit). On
ne pousse pas tant que le PO ne le demande pas. NE JAMAIS faire `wsl --shutdown`.

## 7. Carries + phases suivantes

- **Phases D-G** (plan) : D pull multi-provider + node identity (carry PULL-2 :
  plumber `seeder_node_id` de `SeedRegistry` dans `download()` ; SEED-1/SEED-2) ;
  E ancre VPS headless (driver seed config-driven, 1er appelant prod de
  `request_seed`, **sign-off PO D3 obtenu**) ; F front node-Browse (`/nodes` +
  `/node/:id`, **provenance visible + fork marqué** = exigence PO verrou 4) ;
  G wrap-up + acceptance **survives-VPS-death** cross-machine (SSH mac
  `192.168.1.53` + vps `135.181.42.188`).
- **Carries P2 Phase B → audit S76** : T6 (test direct broadcast handler
  `GossipCmd::Outbox`), WS-3 (hoist `my_endpoint_addr()`), + le verrou-4
  « provenance auteur affichée » est une exigence d'AFFICHAGE Phase F (structurel
  déjà câblé : signature auteur + BLAKE3 + seeder≠auteur).
- **15 P2 audit S74** (`sprint74_audit_findings.md`, archive/v2.1/) — 8 mappés aux
  phases C/D (WIRE-1/2/3, SEED-1/2, PULL-2→D5, CARRY-3, DBQ-1), 5 hygiène → Phase G.

## DÉMARRAGE

1. Lis §0 docs + memory. 2. `git log --oneline -6` + `git status -sb` (git fait
foi). 3. Cas B → phase C. 4. **G8 preflight Phase C** via
`nexus-phase-preflight-deep` (scope §3 + plan §Phase C + D4 + garde-fous + le fait
que le helper générique `verify_signed_list_ingest` et le drop-at-debug existent
déjà depuis B). 5. Attends le verdict, code Phase C. Rien n'est poussé sans
demande PO.
