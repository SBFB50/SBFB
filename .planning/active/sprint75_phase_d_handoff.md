# Handoff — Sprint 75 reprise en Phase D (à coller dans une session fraîche)

> Sprint 75 (pivot découverte PULL node-centrique + ancre VPS) est OUVERT et
> avance. **Phases A + B + C DONE.** Cette session fraîche reprend en **Phase D**.
> Ne ré-invente pas le sprint : tout est déjà décidé et écrit. Suis le workflow.

## 0. Protocole de session fraîche (obligatoire)

1. **Lis intégralement** : `docs/claude/README.md` (workflow, cycle, audit gate,
   G8 preflights, gate Codex, commit 9 sections, dual-platform, bootstrap §7.1
   routing A/B/C/D) + `CLAUDE.md` + memory `MEMORY.md` + `nexus_grid_pivot.md`
   (le **Tip « 2026-06-10 suite 4 »** a tout l'état Phase C) + `feedback_wsl_before_push`
   + `feedback_dual_platform` + `feedback_codex_gate_strict` + `feedback_codex_raw_output`
   + `feedback_full_failfast` + `feedback_background_checks`.
2. **VÉRITÉ TERRAIN = git, pas la mémoire** : `git log --oneline -6` + `git status -sb`
   AVANT toute décision. Au moment d'écrire ce handoff : HEAD = `821aa8c` (Phase C),
   **9 ahead local, RIEN POUSSÉ**. Si le tip mémoire diverge, git fait foi.
3. **Routing** : main thread = ROUTEUR. `.planning/active/` contient
   `sprint75_kickoff.md` + `sprint75_plan.md` + les preflight/review/codex_review
   des phases A/B/C, **pas** de `sprint75_verification.md` → **Cas B (sprint en
   cours)**. Phases A+B+C committées → phase suivante = **D**.
4. **Règle modèle** : jamais le param `model` dans `Agent()`.
   `nexus-phase-preflight-deep` est ENREGISTRÉ ; `nexus-phase-review-deep` ne l'est
   PAS → fallback review par Workflow multi-agent (le pattern Phase C : 5 dimensions
   adversariales → skeptics → synthèse, a très bien marché).

## 1. Où on en est (commit stack S75)

```
821aa8c feat(daemon): Sprint 75 Phase C — node-directory ingest + remote-catalog durability (boot re-pull)
cc8e329 docs(planning): handoff prompt for the next session (S75 Phase C)
f6637d3 feat(core+daemon): Sprint 75 Phase B — NodeDirectoryEntry + DOMAIN_NODE_DIRECTORY_V1 + generic ingest gate + authoring route
96943b7 docs(planning): handoff prompt for the next session (S75 continuation)
479a87c feat(core+daemon): Sprint 75 Phase A — re-mint PoW + endpoint address on outbox replay
e3c3fb6 chore(planning): Sprint 75 Phase A preflight (SCOPE-CUT-CONSISTENT)
a9a2ea7 chore(planning): Sprint 75 kickoff/plan — provenance-visibility requirement
f008433 chore(planning): Sprint 75 kickoff + plan + design review + pivot proposal (Cas C)
0e2fb6b chore(planning): Sprint 75 Phase 0 — S74 audit findings (PASS)
```

- **Phase A** (FIX-A re-mint) = DONE. Helper `mint_ticket_for_hash`
  (`runtime.rs:~1812`, `pub(crate)`) — **c'est LE helper que Phase D réutilise**
  pour re-minter un ticket dialable depuis un hash détenu.
- **Phase B** = DONE `f6637d3` : `NodeDirectoryEntry` + `DOMAIN_NODE_DIRECTORY_V1`
  + trait `SignedList` + helper générique `verify_signed_list_ingest` + route
  `POST /api/daemon/directory/publish` + garde anti-spoof
  `announcement_claims_own_node_id`.
- **Phase C** = DONE `821aa8c` (la primitive load-bearing) :
  - **Ingest annuaire** : `CuratorRuntime` + `directories` DashMap RAM-only +
    `process_directory_announcement_bytes(_throttled)` (miroir du bras curator via
    le gate partagé, subscription-gated sur la MÊME attention-set — DQ3 : une
    souscription `default_curators` couvre curators ET annuaires). Le dispatch
    gossip (`runtime.rs`) appelle `handle_directory_announcement`.
  - **Durabilité (PLAN-ADAPT préflight, honore D4)** : `anchors.json` persiste un
    LOCATOR `{pubkey, ticket, revision}` par ancre (jamais le contenu) ;
    `repull_directories` au boot re-fetch + re-valide (signature + floor revision)
    chaque ancre ABONNÉE, timeout 15s/ancre. Floor anti-rollback REBOOT-DURABLE :
    RAM présente → dedup strict `>` ; RAM vide (re-pull échoué) → floor persisté
    `>= P` (same-revision RESTAURE le catalogue, rollback rejeté).
  - **Browse** : `BrowseSource::NodeDirectory` (sérialise `"nodedirectory"`, Zod
    frontend additif) + 3ᵉ boucle `aggregate()` : node_id = ANCRE dialable
    (probée), archive_hash = BLAKE3 AUTEUR, `archive_ticket: None`, repo_url None
    (provenance dérivée au pull, affichée Phase F). `known_entry_count` additif.
  - **Carries CLOSED** : WIRE-1 (`ReleasePublishedPayload` + `project_name`/
    `category` Option additif 0-bump, producteur deploy câblé,
    `extract_index_fields` lit category) ; WIRE-2 (SeedRegistry re-keyé
    `(project_id, archive_hash)`, `count_recent(pid, Option<hash>, now)`
    Some=version-exacte / None=agnostic-STRICT, route `?archive_hash`, front
    `AvailabilitySheet` passe `entry.archive_hash`, `self_seeding` honnête par
    version) ; DBQ-1 (`set_keep_online` UPSERT COALESCE — None ne NULL plus le
    hash M18).
  - **Gates** : préflight PLAN-ADAPT (0 DESIGN-CONFLICT) ; review-deep Workflow
    5-dim + skeptics 0 P0/P1 ; **Codex 7 ROUNDS → OVERALL PASS** (8 GAPs réels
    corrigés ; détail au §5 pièges + `sprint75_phase_c_review.md` §Reconciliation).
  - **Compteurs** : nextest --workspace **1724** 0-fail (1714→+10) ; web Vitest
    **334** (331→+3), coverage 86.94/78.73/85.82/88.25, size 6/6, scan FR clean.
  - **C6** : acceptance E2E cross-machine différée à Phase G (consigné préflight).

## 2. Docs à lire (tout est décidé)

- `.planning/active/sprint75_plan.md` — **§Phase D détaillée** (D.1-D.5 :
  fichiers/tests/acceptance/commit) + dépendances inter-phases + fail-fast 24 rows.
- `.planning/active/sprint75_kickoff.md` — D1-D5 gelées (§5), 5 verrous
  anti-recentralisation (§4), inventaire R4 (§6), carries (§8), Q5/Q7 (§10).
- `.planning/active/sprint75_phase_c_{preflight,review,codex_review}.md` — le
  process à imiter. Le `codex_review.md` = round 7 brut (OVERALL PASS).
- `.git/CODEX_SPRINT75_PHASE_C.txt` — RÉUTILISE sa structure pour le prompt Codex
  Phase D : le bloc « Output contract » (tokens littéraux GAP/CONFIRMED +
  `OVERALL: PASS/FAIL`), le bloc « SCOPE — review CODE ONLY » (exclut `.planning/`
  et `.git/`) et le bloc « PHASE BOUNDARY » (à réécrire pour D : juger D, pas E/F).

## 3. Phase D — scope (plan §Phase D, kickoff D5, carry PULL-2 + GAPs R5)

**Pull multi-provider + node identity.** Rendre les apps découvertes via annuaire
réellement TÉLÉCHARGEABLES/rendables, et le fetch résilient à la mort de l'ancre.

- **Multi-provider `download()`** (carry PULL-2, D5) : `fetch_ticket` dial UN seul
  endpoint aujourd'hui (`blobs.rs:170-193`) alors que `Downloader.download(hash,
  providers)` accepte déjà un Vec. Plumber un vecteur de providers :
  node_id de l'annuaire d'abord, puis les `seeder_node_id` du `SeedRegistry`
  (ordering Q5 ; budget timeout). Nouveau helper genre
  `fetch_hash_multi(endpoint, lookup, hash, Vec<endpoint_id>)`.
- **FERMER LES 2 GAPs Codex R5 (hors-phase en C, in-scope D)** : une app
  directory-only (BrowseEntry `source=nodedirectory`, `archive_hash` Some,
  `archive_ticket` None) doit devenir (a) **rendable** — blob-serve résout
  aujourd'hui les tickets via `direct_entries`/`find_archive_ticket_by_hash`
  (`browse.rs:613`, `http.rs` blob-serve) ; le pull doit fetch le blob depuis
  `(node_id, archive_hash)` (multi-provider ci-dessus) puis le servir ; (b)
  **seedable volontairement** — `seed_voluntary` (`http.rs:~1413`) regarde
  seulement `direct_entries` ; il doit aussi résoudre une app annuaire. Le
  commentaire SCOPE au site aggregator (`browse.rs` 3ᵉ boucle) documente ce
  report — le retirer/ajuster quand D livre.
- **SeedRegistry prod-ready** : exposer `seeders_recent` en prod (aujourd'hui
  `#[cfg(test)]`) pour le vecteur providers ; **SEED-1** clamp
  `seen_at = min(feed_ts, recv_clock)` (anti future-ts) ; **SEED-2** cap taille
  registre (anti-bloat). NOTE : le registre est déjà keyé `(pid, hash)` depuis C —
  `seeders_recent(pid, hash, now)` test-only existe, le promouvoir.
- **Node identity exposure** : `GET /api/daemon/nodes` (grouping des entries par
  node_id) OU promotion additive du champ `node_id` (aujourd'hui `#[serde(skip)]`
  `browse.rs:195` — si promu : additif, Zod `.optional()`, et c'est un changement
  de bytes /browse → fail-fast web complet). Le choix est au préflight ; la page
  front `/nodes` complète est Phase F.
- **Statut honnête « joignable-via-seeder »** (Q7) : publisher down mais un seeder
  détient le BLAKE3 → nouveau bucket de statut (ou champ additif), jamais un
  mensonge « Reachable » sur l'ancre morte.

Tests plan §D.3 : `fetch_falls_back_to_seeder_when_anchor_offline` (LE test
load-bearing), `fetch_provider_ordering` (annuaire d'abord puis seeders, Q5),
`seed_registry_clamps_future_ts` (SEED-1), `seed_registry_size_bounded` (SEED-2),
`reachable_via_seeder_status` (Q7), `nodes_endpoint_groups_by_node_id`.
Commit cible : `feat(core+daemon): Sprint 75 Phase D — multi-provider pull + node
identity exposure`. Body : Carry closure PULL-2/SEED-1/SEED-2 CLOSED + GAPs R5
fermés.

**Garde-fous (kickoff §4, inchangés)** : zéro champ cible/hôte ; additif jamais
substitutif ; ancre jamais hard-codée ; provenance = AUTEUR jamais seeder (le
multi-provider fetch vérifie le BLAKE3 — un seeder ne peut servir que les bons
octets) ; requêtes ne quittent jamais la machine sans choix explicite.

## 4. Le cycle de phase (à respecter strictement)

1. **G8 preflight** : `Agent(subagent_type: "nexus-phase-preflight-deep")` →
   `sprint75_phase_d_preflight.md`. Lui passer le scope §3 + plan §Phase D + D5
   gelée + Q5/Q7 + le fait que les 2 GAPs R5 Codex sont à fermer ici + les régions
   code clés (`blobs.rs:170-193`, `seed_registry.rs`, `browse.rs:613` +3ᵉ boucle,
   `http.rs` seed_voluntary/blob-serve, `runtime.rs mint_ticket_for_hash`).
2. **Code** conformément au plan/preflight.
3. **Fail-fast Windows COMPLET** en `run_in_background` : fmt + clippy
   `--workspace --all-targets` + nextest --workspace + doctests + release + web
   COMPLET si touché (tsc/lint/vitest/coverage/build/size/scan).
4. **Review-deep** : Workflow 5 dimensions adversariales (correctness, security,
   wire, tests, patterns) → skeptics sur chaque P0/P1 (consigne REFUTER, défaut
   `isReal=false`) → synthèse `sprint75_phase_d_review.md`. Corrige les P1 ET les
   P2/NIT cheap-value (norme anti-faux-vert).
5. **Codex** (gate BLOQUANTE, GPT-5.5) : prompt `.git/CODEX_SPRINT75_PHASE_D.txt`
   (réutilise la structure du C : Output contract + SCOPE code-only + PHASE
   BOUNDARY adaptée) → `Get-Content ... -Raw | codex exec
   --dangerously-bypass-approvals-and-sandbox -o
   .planning/active/sprint75_phase_d_codex_review.md`. **Sortie BRUTE.** Si GAP →
   corrige + fail-fast re-vert + round suivant jusqu'à `OVERALL: PASS`.
6. **Réconcilie** : review.md → `## Verdict: PASS` (header EXACT, PASS même ligne).
7. **Commit** `feat(core+daemon): Sprint 75 Phase D — ...` body 9 sections
   (headers EXACTS du commit `821aa8c` : Contexte / Fichiers / Delta tests /
   Verification §7.4 / Scope cuts [NU] / G8 traceability / Pre-launch protocol /
   Codex verification / Carry closure / Unblock).
8. **Memory** : `nexus_grid_pivot.md` Tip + `MEMORY.md` AVANT de rendre.

## 5. Pièges process (appris A→C, IMPORTANT)

- **Codex multi-rounds** (7 rounds en C — c'est NORMAL, pas un échec) :
  - Il peut **flagger SA PROPRE sortie précédente** (le `codex_review.md` round
    N-1 contenant FAIL) comme « stale artifact » → le prompt DOIT scoper « review
    CODE ONLY, ignore `.planning/` + `.git/` ».
  - Il **ne connaît PAS les frontières de phase** → bloc PHASE BOUNDARY explicite
    (ce qui est D vs E vs F), sinon il flagge du travail séquencé comme GAP.
  - Il **re-exécute la suite de tests lui-même** (rounds longs ~10-15 min) — ne
    pas lancer de cargo concurrent pendant un run Codex (build-lock).
  - Exiger les tokens littéraux `GAP`/`CONFIRMED` + `OVERALL: PASS|FAIL` + evidence
    `file.rs:NN` (le lightcheck les exige ; « GAPS » pluriel ne matche pas).
  - L'artefact `-o` = le DERNIER round ; l'historique des GAPs vit dans review.md
    §Reconciliation + le commit body.
- **Stop-hook process-supervisor** : pendant les attentes longues (Codex), bloquer
  en **FOREGROUND par chunks** (`while ! grep -q "CODEX_EXIT=" ...; do sleep 20;
  done` avec sortie avant le timeout de l'outil, puis re-chunk). Un wait en
  background TERMINE le tour → le hook bloque si l'arbre est sale. Messages
  d'attente finality-free (pas de `fait`/`corrigé`/`done`/`prêt`...).
- **1er `cargo check` après une grosse édition multi-crates peut FAILLIR
  transitoire** (course de build parallèle, E0463 « can't find crate » en cascade,
  erreurs dans des crates non touchées) → re-run propre avant de paniquer.
- **`phase-auditor-gate`** exige `## Verdict: PASS` EXACT dans review.md.
- **Lightcheck** : exige le codex_review STAGED au moment du check → séquence =
  trim trailing-blank (`python -c "...rstrip()..."`) → `git add` → lightcheck →
  `git commit -F` (2 appels Bash séparés). WARN faux-positifs connus : il tronque
  `.tsx` en `.ts` (« missing file ») + « wire-format surface » quand le body
  mentionne le 0-bump — non bloquants.
- **`nexus-launcher` rustc STATUS_STACK_BUFFER_OVERRUN** au build test = crash
  compilo transitoire → re-run.
- **Pipes masquent l'exit code** : inspecter le CONTENU des sorties background
  (markers `===X===`, « N passed »), pas l'exit de la notification.
- **JAMAIS git/codex depuis `web/`** (`.git` imbriqué périmé) — racine + `git -C`.
- **Review-deep par Workflow** : le pattern C (5 finders schema-typés → skeptics
  adversariaux sur P0/P1 → `{confirmed}`) est efficace ; corriger aussi les P2/NIT
  cheap-value trouvés (doc-honnêteté, regression guards) — Codex re-trouve les
  mêmes sinon.

## 6. État git + push

9 ahead local (`0e2fb6b`→`821aa8c` + ce handoff), **RIEN POUSSÉ**. Le **Docker
Linux canonique** (`sbfb-ci`) est le gate **AVANT PUSH** uniquement
(`feedback_wsl_before_push`). On ne pousse pas tant que le PO ne le demande pas.
NE JAMAIS faire `wsl --shutdown`.

## 7. Carries + phases suivantes

- **Phase E** : ancre VPS headless (config `[seed]`/`[directory]` boot driver,
  `fetch_and_pin`, 1er appelant prod `request_seed` (`seed_protocol.rs:298`
  dead_code), authoring VPS signé, systemd ; sign-off PO D3 OBTENU). **La
  re-annonce PRODUCTEUR au boot** (un publisher ne re-annonce pas son annuaire
  après reboot — P2 review C déféré) = livrable E.
- **Phase F** : front node-Browse (`/nodes` + `/node/:id`, AddAnchorDialog,
  cold-start, **exigence PO verrou-4 : provenance auteur AFFICHÉE + fork marqué
  « version dérivée »**, WEB-1 toggle selfSeeding). Le repo_url/provenance des
  apps annuaire se dérive du provenance.json fetché au pull (pas du listing —
  consigné préflight C DQ4).
- **Phase G** : wrap-up + acceptance survives-VPS-death cross-machine (SSH mac
  `192.168.1.53` + vps `135.181.42.188`) + C6 E2E + hygiène (CARRY-5 clamp,
  CARRY-2 Rejected-terminal, PULL-1 dedup, FORK-1 entry-cap) + LT-2 ARMÉ +
  verification.md + sprint76_audit_plan.md.
- **Carries P2 → audit S76** : T6 (test direct `GossipCmd::Outbox`), WS-3 (hoist
  `my_endpoint_addr`), pin skip-GC du blob annuaire re-pull (quand un GC
  existera — scope cut 3), `known_entry_count` double-compte curator+annuaire
  (best-effort assumé, sur-estimation tolérée THREAT_MODEL §15), re-pull boot
  séquentiel N×15s (parallélisation pilote-ferme non bloquante).

## DÉMARRAGE

1. Lis §0 docs + memory. 2. `git log --oneline -6` + `git status -sb` (git fait
foi). 3. Cas B → phase D. 4. **G8 preflight Phase D** via
`nexus-phase-preflight-deep` (scope §3 + plan §Phase D + D5 + Q5/Q7 + les 2 GAPs
R5 à fermer + le helper `mint_ticket_for_hash` réutilisable + `Downloader.download`
accepte déjà un Vec). 5. Attends le verdict, code Phase D. Rien n'est poussé sans
demande PO.
