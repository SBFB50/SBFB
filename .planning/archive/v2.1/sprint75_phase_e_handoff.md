# Handoff — Sprint 75 reprise en Phase E (à coller dans une session fraîche)

> Sprint 75 (pivot découverte PULL node-centrique + ancre VPS) est OUVERT et
> avance. **Phases A + B + C + D DONE.** Cette session fraîche reprend en
> **Phase E**. Ne ré-invente pas le sprint : tout est déjà décidé et écrit.
> Suis le workflow.

## 0. Protocole de session fraîche (obligatoire)

1. **Lis intégralement** : `docs/claude/README.md` (workflow, cycle, audit gate,
   G8 preflights, gate Codex, commit 9 sections, dual-platform, bootstrap §7.1
   routing A/B/C/D) + `CLAUDE.md` + memory `MEMORY.md` + `nexus_grid_pivot.md`
   (le **Tip « 2026-06-10 suite 5 »** a tout l'état Phase D) + `feedback_wsl_before_push`
   + `feedback_dual_platform` + `feedback_codex_gate_strict` + `feedback_codex_raw_output`
   + `feedback_full_failfast` + `feedback_background_checks`.
2. **VÉRITÉ TERRAIN = git, pas la mémoire** : `git log --oneline -6` + `git status -sb`
   AVANT toute décision. Au moment d'écrire ce handoff : HEAD = `0010450` (Phase D),
   **11 ahead local (+1 avec ce handoff), RIEN POUSSÉ**. Si le tip mémoire diverge,
   git fait foi.
3. **Routing** : main thread = ROUTEUR. `.planning/active/` contient
   `sprint75_kickoff.md` + `sprint75_plan.md` + les preflight/review/codex_review
   des phases A/B/C/D, **pas** de `sprint75_verification.md` → **Cas B (sprint en
   cours)**. Phases A+B+C+D committées → phase suivante = **E**.
4. **Règle modèle** : jamais le param `model` dans `Agent()`.
   `nexus-phase-preflight-deep` est ENREGISTRÉ ; `nexus-phase-review-deep` ne l'est
   PAS → fallback review par Workflow multi-agent (pattern C/D : 5 dimensions
   adversariales → skeptics refute-by-default sur P0/P1 → synthèse ; en D il a
   sorti le finding hex-case, ça marche très bien).

## 1. Où on en est (commit stack S75)

```
0010450 feat(core+daemon): Sprint 75 Phase D — multi-provider pull + node identity exposure
9f7de7f docs(planning): handoff prompt for the next session (S75 Phase D)
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

- **Phase A** (FIX-A re-mint) = DONE. `mint_ticket_for_hash` = helper PRODUCTEUR
  (bail si blob absent). **Phase B** = DONE (type signé + DOMAIN + authoring
  `POST /api/daemon/directory/publish`). **Phase C** = DONE (ingest annuaire +
  re-pull boot + locator `anchors.json` + carries WIRE-1/2, DBQ-1).
- **Phase D** = DONE `0010450` (le pull) :
  - **PLAN-ADAPT préflight** : le « re-mint ticket » du plan pour le pull
    consommateur était irréalisable — la forme correcte est
    `BlobsClient::fetch_hash_multi(endpoint, hash, Vec<EndpointId>)` (download
    bare-hash via `Downloader.download`, vec ORDONNÉ ancre-d'abord Q5, pkarr
    résout les ids nus, AUCUN ticket) + `fetch_and_pin_multi` (+ tag keep-online).
    `MAX_FETCH_PROVIDERS=16` enforced DANS la primitive.
  - **GAPs R5 fermés** : blob_serve 4e tier directory-only (resolve sur annuaires
    ABONNÉS → fetch multi → read-back par hash, timeout 120 s) ; `seed_voluntary`
    fallback `SeedFetchPlan Ticket|Multi`. Helpers daemon réutilisables :
    `find_directory_app_by_hash/by_project` + `directory_pull_providers`
    (cap 8, dedup, self exclu) dans `http.rs`.
  - **SeedRegistry prod** : SEED-1 clamp `min(seen_at, now)` in-registry, SEED-2
    double cap (1024/64) éviction stalest, **normalisation hex lowercase
    write+read**, `seeders_recent` prod. Signatures : `record(..., seen_at, now)`.
  - **Node identity** : route additive `GET /api/daemon/nodes` enveloppe
    `{nodes:[{node_id,revision,app_count,catalog}]}` ; `/browse` BYTE-IDENTIQUE
    (node_id reste `#[serde(skip)]`). Badge Q7 visible → Phase F (signal backend
    livré : row unreachable + seed-count version-exact).
  - **Gates** : review Workflow 5-dim + skeptics 0 P0/P1 (12 fixes in-phase dont
    hex-case + 7 déférés scopés) ; **Codex 1 SEUL ROUND → OVERALL PASS 12/12**
    (vs 7 rounds en C — voir §5 pièges, leçon prompt).
  - **Compteurs** : nextest --workspace **1735** 0-fail (1724→+11) ; web Vitest
    **334** INCHANGÉ (0 fichier web), coverage 86.94/78.73/85.82/88.25, size 6/6.

## 2. Docs à lire (tout est décidé)

- `.planning/active/sprint75_plan.md` — **§Phase E détaillée** (E.1-E.5 :
  fichiers/tests/acceptance/commit) + dépendances + fail-fast 24 rows.
- `.planning/active/sprint75_kickoff.md` — **D3 gelée (§5)** : VPS = 2 rôles
  bornés, sign-off PO OBTENU ; 5 verrous (§4) ; Q3/Q4 (§10) à trancher au
  preflight ; scope cut 3 (GC reaper déféré, ack budget disque).
- `.planning/active/sprint75_phase_d_{preflight,review,codex_review}.md` — le
  process à imiter (et l'inventaire des helpers D réutilisables par E).
- `.git/CODEX_SPRINT75_PHASE_D.txt` — RÉUTILISE sa structure pour le prompt
  Codex Phase E : Output contract (tokens littéraux GAP/CONFIRMED + `OVERALL:
  PASS/FAIL`), SCOPE code-only (exclut `.planning/` et `.git/`), PHASE BOUNDARY
  (à réécrire pour E : juger E, pas F/G) + le bloc PLAN-ADAPT si le preflight E
  en produit un. C'est ce prompt-là qui a donné un PASS en 1 round.

## 3. Phase E — scope (plan §Phase E, kickoff D3, carry re-annonce producteur)

**Ancre VPS headless.** Le modèle opérationnel VPS sans session UI : un nœud
config-driven qui seede MES apps + invitées au boot et publie son annuaire signé.

- **Config** (`crates/nexus-shell-daemon/src/config.rs`) : sections
  `[seed] keep_online_projects` / `[directory] catalog` lues au boot. **Défauts
  VIDES** (verrou 3 tripwire : tout default non-vide compilé = DESIGN-CONFLICT).
  Q3 du kickoff (réutiliser `default_curators` vs nouveau `default_anchors`) à
  trancher au preflight.
- **Driver boot** (`crates/nexus-shell-daemon/src/runtime.rs`) : lire config →
  `fetch_and_pin` les project_ids configurés (apps JAMAIS déployées localement)
  + set keep_online + re-emit + re-mint. Étend `reannounce_seeds_at_boot`
  (`feed_sync.rs:160-199`) en acquire-then-pin. **Phase D a livré
  `fetch_and_pin_multi` + `directory_pull_providers` + les résolveurs — le
  driver doit les RÉUTILISER** (une app configurée se résout par les annuaires
  abonnés/le SeedRegistry, pas par un ticket frais).
- **`request_seed` 1er appelant prod** (`crates/nexus-shell-daemon/src/
  seed_protocol.rs:298`, retirer `#[allow(dead_code)]`) : le client du protocole
  authentifié `sbfb/seed/0` (S74 Phase E) gagne un appelant réel.
- **Re-annonce PRODUCTEUR au boot** (P2 review C déféré → livrable E) : un
  publisher d'annuaire ne re-annonce pas son `NodeDirectoryEntry` après reboot —
  le re-pull consommateur (C) couvre les ABONNÉS, pas la re-émission du
  producteur. À câbler dans le boot path.
- **Authoring VPS signé** : builder boot ou endpoint loopback scriptable
  (`CuratorListEntry::sign`/`NodeDirectoryEntry::sign` avec node keypair) — le
  VPS publie son catalogue sans navigateur.
- **`deploy/`** : unit systemd + `config.toml.example` sections seed/directory.
- **Q4** : policy SEED bornée abstraite (budget disque + accept-list par-projet,
  pas de knob numérique user) — ack scope cut 3 (pas de reaper).

Tests plan §E.3 : `boot_seed_driver_pins_configured_projects` (LE test : le VPS
seede une app qu'il n'a PAS déployée), `boot_repins_keep_online_blobs` (re-pin,
pas seulement re-announce), `request_seed_prod_caller`,
`vps_authoring_signs_own_directory`, `config_seed_section_parsed`. + un test de
la re-annonce producteur boot (le carry C).
Commit cible : `feat(daemon): Sprint 75 Phase E — headless VPS anchor
(config-driven seed driver + signed authoring)`. Body : Contexte (D3 +
sign-off PO réf), Scope cuts (GC reaper déféré), Carry closure (re-annonce
producteur C CLOSED).

**Garde-fous (kickoff §4, inchangés)** : verrou 3 = l'ancre vit dans MON
config.toml, défauts compilés VIDES, jamais hard-codée ; verrou 5 nuancé = le
driver boot EST un fetch au boot mais il est config-driven EXPLICITE (l'opérateur
l'a écrit), jamais un défaut universel ; seed borné MES apps + invites, JAMAIS
miroir universel (leçon SSB pubs) ; provenance = AUTEUR toujours (le VPS seede,
ne re-signe rien).

## 4. Le cycle de phase (à respecter strictement)

1. **G8 preflight** : `Agent(subagent_type: "nexus-phase-preflight-deep")` →
   `sprint75_phase_e_preflight.md`. Lui passer le scope §3 + plan §Phase E + D3
   gelée + Q3/Q4 + le carry re-annonce producteur + les helpers D réutilisables
   (`fetch_and_pin_multi`, `directory_pull_providers`, résolveurs http.rs,
   `seeders_recent` prod) + les régions code (`config.rs:245-251
   default_curators`, `feed_sync.rs:160-199 reannounce_seeds_at_boot`,
   `seed_protocol.rs:298`, `runtime.rs` boot path + `mint_ticket_for_hash`).
2. **Code** conformément au plan/preflight.
3. **Fail-fast Windows COMPLET** en `run_in_background` : fmt + clippy
   `--workspace --all-targets` + nextest --workspace + doctests + release + web
   COMPLET (même si 0 fichier web touché — règle full fail-fast).
4. **Review-deep** : Workflow 5 dimensions adversariales → skeptics (REFUTER,
   défaut isReal=false) → synthèse `sprint75_phase_e_review.md`. Corrige P1 ET
   les P2/NIT cheap-value (norme anti-faux-vert).
5. **Codex** (gate BLOQUANTE, GPT-5.5) : prompt `.git/CODEX_SPRINT75_PHASE_E.txt`
   (structure du D !) → `Get-Content ... -Raw | codex exec
   --dangerously-bypass-approvals-and-sandbox -o
   .planning/active/sprint75_phase_e_codex_review.md`. **Sortie BRUTE.** Si GAP →
   corrige + fail-fast re-vert + round suivant jusqu'à `OVERALL: PASS`.
6. **Réconcilie** : review.md → `## Verdict: PASS` (header EXACT, PASS même ligne).
7. **Commit** `feat(daemon): Sprint 75 Phase E — ...` body 9 sections (headers
   EXACTS de `0010450` : Contexte / Fichiers / Delta tests / Verification §7.4 /
   Scope cuts / G8 traceability / Pre-launch protocol / Codex verification /
   Carry closure / Unblock). Séquence : trim trailing-blank du codex_review →
   `git add` explicite → `git commit -F` (2 appels Bash séparés).
8. **Memory** : `nexus_grid_pivot.md` Tip + `MEMORY.md` AVANT de rendre.

## 5. Pièges process (appris A→D, IMPORTANT)

- **Le prompt Codex de D a donné OVERALL PASS en 1 SEUL round** (vs 7 en C).
  La recette : PHASE BOUNDARY explicite (ce qui est E vs F vs G), le PLAN-ADAPT
  du preflight expliqué comme design CORRECT (pas une déviation à flagger),
  SCOPE code-only (ignore `.planning/` + `.git/`), et la liste des livrables
  avec leurs evidence attendues. Copier la structure de
  `.git/CODEX_SPRINT75_PHASE_D.txt`.
- **clippy `await_holding_lock` ignore un `drop()` explicite** (analyse
  lexicale) → toujours un bloc `{ }` autour d'un MutexGuard avant un await
  (pattern prod existant).
- **`mint_ticket_for_hash` = PRODUCTEUR uniquement** (bail si `!blobs.has`).
  Le chemin consommateur = `fetch_hash_multi`/`fetch_and_pin_multi`. Ne pas
  re-introduire le détour ticket dans le driver E.
- **Codex multi-rounds reste possible** : il peut flagger SA PROPRE sortie
  précédente (scoper code-only), ne connaît PAS les frontières de phase
  (les expliciter), re-exécute la suite lui-même (~10-15 min/round — PAS de
  cargo concurrent pendant un run Codex).
- Exiger les tokens littéraux `GAP`/`CONFIRMED` + `OVERALL: PASS|FAIL` +
  evidence `file.rs:NN` (« GAPS » pluriel ne matche pas le lightcheck).
- **Stop-hook process-supervisor** : messages d'attente finality-free (pas de
  `fait`/`corrigé`/`done`/`prêt`...) tant que l'arbre est sale.
- **1er `cargo check` post-grosse-édition peut FAILLIR transitoire** (E0463
  cascade) → re-run propre. `nexus-launcher` STATUS_STACK_BUFFER_OVERRUN =
  crash compilo transitoire → re-run.
- **`phase-auditor-gate`** exige `## Verdict: PASS` EXACT dans review.md.
  Lightcheck WARN « wire-format surface » = faux-positif connu quand le body
  mentionne le 0-bump (non bloquant).
- **JAMAIS git/codex depuis `web/`** (`.git` imbriqué périmé) — racine + `git -C`.
- **Pipes masquent l'exit code** : inspecter le CONTENU des sorties background
  (markers `===X===`, « N passed »), pas l'exit de la notification.

## 6. État git + push

11 ahead local (`0e2fb6b`→`0010450`) + ce handoff, **RIEN POUSSÉ**. Le **Docker
Linux canonique** (`sbfb-ci`) est le gate **AVANT PUSH** uniquement
(`feedback_wsl_before_push`). On ne pousse pas tant que le PO ne le demande pas.
NE JAMAIS faire `wsl --shutdown`.

## 7. Carries + phases suivantes

- **Phase F** : front node-Browse (`/nodes` + `/node/:id`, AddAnchorDialog,
  cold-start, **exigence PO verrou-4 : provenance auteur AFFICHÉE + fork marqué
  « version dérivée »**, WEB-1 toggle selfSeeding, badge Q7
  « joignable-via-seeder » visible). + déférés D pour F : discriminateur
  `archive_hash` optionnel sur `SeedVoluntaryRequest` (collision multi-ancres) ;
  **contrat Zod /nodes : `.strict()` sur l'enveloppe `{nodes}`, PAS sur les rows
  catalog** (sinon le 1er ajout additif 0-bump brique la page) — ou projection
  HTTP dédiée.
- **Phase G** : wrap-up + acceptance survives-VPS-death cross-machine (SSH mac
  `192.168.1.53` + vps `135.181.42.188`) + C6 E2E + hygiène (CARRY-5 clamp,
  CARRY-2 Rejected-terminal, PULL-1 dedup, FORK-1 entry-cap) + **THREAT_MODEL
  §15 rows déférés D** (directory pull route publique blob-serve
  [oracle drive-by + amplification sans in-flight dedup], /nodes, SEED-1/2,
  fresh-flood acceptance) + LT-2 ARMÉ + verification.md + sprint76_audit_plan.md.
- **Carries P2 → audit S76** : **PULL-3 NEW** (cross-tier failover : un direct
  entry au ticket mort ne tente pas le tier directory pour le même hash —
  trou availability documenté review D) ; **sampling anti-Sybil NEW** (le seeder
  tail de `directory_pull_providers` est lexicographique → crowdable, doc
  inline posée) ; T6 (test direct `GossipCmd::Outbox`), WS-3 (hoist
  `my_endpoint_addr`), pin skip-GC du blob annuaire re-pull (quand un GC
  existera), `known_entry_count` double-compte (best-effort assumé), re-pull
  boot séquentiel N×15s (parallélisation non bloquante).

## DÉMARRAGE

1. Lis §0 docs + memory. 2. `git log --oneline -6` + `git status -sb` (git fait
foi). 3. Cas B → phase E. 4. **G8 preflight Phase E** via
`nexus-phase-preflight-deep` (scope §3 + plan §Phase E + D3 + Q3/Q4 + carry
re-annonce producteur + helpers D réutilisables). 5. Attends le verdict, code
Phase E. Rien n'est poussé sans demande PO.
