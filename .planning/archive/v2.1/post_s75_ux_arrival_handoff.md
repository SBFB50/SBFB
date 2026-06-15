# Handoff — UX-ARRIVAL hybride (décision PO 2026-06-11) — PROMPT DE REPRISE

> **À coller tel quel dans une session fraîche.** Mission : implémenter
> l'écran d'arrivée HYBRIDE (décision PO tranchée, §2) en **mini-cycle
> hors-sprint MAINTENANT** (directive PO explicite : « pas dans un prochain
> sprint »). Ce n'est PAS l'audit gate S75 ni le kickoff S76 — ils restent à
> faire APRÈS. Tout le design est décidé (§3) ; ne ré-invente rien.

## §0 Protocole de session fraîche (obligatoire)

1. **Lis intégralement** : `docs/claude/README.md` (cycle, gates G8/review/
   Codex, commit discipline, §6 conventions) + `CLAUDE.md` + memory
   `MEMORY.md` + `nexus_grid_pivot.md` (Tip « 2026-06-11 suite 9 » = l'état
   exact) + `feedback_full_failfast` + `feedback_dual_platform` +
   `feedback_codex_gate_strict` + `feedback_codex_raw_output` +
   `feedback_background_checks` + `feedback_cd_web_trap` + `nested_git_web_trap`.
2. **Vérité terrain = git** : `git log --oneline -6` + `git status -sb`.
   Au moment d'écrire ce handoff : HEAD = `f02037a` (ce doc), 20 ahead,
   arbre net, RIEN POUSSÉ (Docker canonique déjà vert 1759/1759 ; on ne
   pousse pas sans demande PO).
3. **Routing** : mini-cycle hors-sprint « UX-ARRIVAL » (équivalent Cas B à
   phase unique). Le titre commit ne contient pas « Sprint N Phase X » →
   les hooks lightcheck ne s'arment PAS : la discipline des gates est
   VOLONTAIRE et OBLIGATOIRE quand même (nouvelle surface ingest) :
   **preflight G8 → code+tests → fail-fast dual-platform → review Workflow
   → Codex (bloquant, sortie brute) → commit unique → memory**.
4. Jamais le param `model` dans `Agent()`. `nexus-phase-preflight-deep` est
   ENREGISTRÉ ; review = fallback Workflow multi-agent (5 dimensions
   adversariales → skeptics refute-by-default sur P0/P1 → synthèse).

## §1 Pourquoi (contexte produit)

Session de test live PC+Mac+VPS post-S75 : le PO a constaté que la grille
`/browse` d'un pair se remplit TOUTE SEULE (annonces gossip `direct`
poussées par le swarm, abonné ou pas) — pas « objectif ». Mécanique
confirmée et ACQUISE : rien n'est pré-installé, fiches de découverte
seulement, octets fetchés au 1er « Ouvrir » puis cache local. Le chemin
cible : *j'arrive → je vois les nœuds du réseau → je m'abonne → leurs
projets apparaissent → j'ouvre (download à la demande) → « garder en
ligne »*.

## §2 La décision PO (gelée, ne pas rebattre)

**Option C HYBRIDE + limite de refresh anti-spam** :
- Grille `/browse` = MES sources (own + abonnés) ; les fiches poussées non
  sollicitées vont dans une section SÉPARÉE « Découvert sur le réseau »,
  jamais mélangées.
- Page `/nodes` enrichie : nœuds OBSERVÉS (annuaires entendus par gossip
  sans abonnement, MÉTADONNÉES seulement) avec CTA « S'abonner ».
- Rate-limit d'ingest des annonces non sollicitées (exigence PO explicite).

## §3 Design d'implémentation (chemins exacts, état `173426e`)

### 3.1 Daemon — registre « nœuds observés » (RAM-only, borné)
- Aujourd'hui le bras directory DROPPE les `NodeDirectoryAnnouncement` de
  pubkeys non-abonnées (gate partagé `verify_signed_list_ingest`,
  subscription-gated, `iroh_runtime.rs`). NE PAS ingérer leur CATALOGUE —
  retenir : `observed_directories: {node_id, revision, app_count, last_seen}`.
- Signature Ed25519 TOUJOURS vérifiée avant rétention (PoW gossip = 1er
  filtre). Un nœud ABONNÉ ne va pas dans observed (il est dans nodes).
- **Bornes anti-spam** (pattern SeedRegistry S75-D, SEED-1/SEED-2) : cap
  ~256 nœuds observés, eviction stalest ; TTL 48h purge paresseuse ;
  **rate-limit par node_id** (1 update acceptée / ~60 s — base : le
  throttle existant `process_directory_announcement_bytes_throttled`
  Phase C) ; clamp `last_seen = min(now, claimed)` DANS la primitive
  (§P59.2, jamais convention d'appelant) ; hex lowercase write+read
  (§P59.3).

### 3.2 Route `/api/daemon/nodes` — clé additive `observed`
- Enveloppe actuelle `{nodes:[…]}`, Zod front `.strict()` sur l'ENVELOPPE :
  ajouter `observed:[{node_id, revision, app_count, last_seen}]` = MAJ du
  schéma front DANS LE MÊME COMMIT (loopback-local, pas un wire P2P, 0 bump ;
  précédent : `self_pin_enabled` en F).

### 3.3 Daemon — flag `from_subscribed` sur /browse (serialize-only)
- Le front ne distingue pas un `direct` d'un nœud abonné d'un `direct`
  d'inconnu (`node_id` est `#[serde(skip)]`). Exposer
  `from_subscribed: bool` via le flatten `BrowseEntryView` (pattern
  `is_own`, §P58.2 — ZÉRO churn des ~26 sites de construction), calculé à
  la sérialisation : `is_own || entry.node_id ∈ attention set`.

### 3.4 Front
- `/browse` : grille principale = `is_own || source ∈ {curator,
  nodedirectory} || (direct && from_subscribed)` ; le reste → section
  « Découvert sur le réseau » séparée, cappée à l'affichage (~24 fiches,
  plus récentes d'abord), copy honnête « annoncé sur le réseau — non
  sollicité ». La dédup `dedupeBrowseEntries` (`fdb4fb1`) s'applique aux
  deux groupes. Si la section est vide → ne pas la rendre.
- `/nodes` : section « Nœuds découverts sur le réseau » (observed), CTA
  S'abonner = `addAnchor` EXISTANT ; copy « s'annonce sur le réseau —
  abonne-toi pour voir son catalogue ». Lignes en-attente existantes
  inchangées. Zod : rows observed TOLÉRANTES (pas `.strict()` sur les rows,
  règle P37).

### 3.5 Garde-fous (NON négociables)
- Verrous anti-recentralisation 1-5 inchangés (rien de pré-rempli/hard-codé ;
  additive jamais substitutive : la grille reste le superset de MES sources,
  la section découverte est l'ambiant séparé). 0 bump `*_FORMAT_VERSION`,
  0 nouveau DOMAIN, 0 dep. Pre-launch policy (CLAUDE.md).
- Tests Rust : registre (cap, TTL, rate-limit, eviction stalest, clamp,
  lowercase, abonné-exclu-d'observed) + `from_subscribed` (own / abonné /
  inconnu) + `/nodes` enveloppe+observed pinnés producteur. Tests Vitest :
  split grille/section (direct inconnu → section ; direct abonné → grille),
  cap d'affichage, section vide non rendue, /nodes observed + CTA ouvre
  AddAnchorDialog, Zod tolérant sur row additive.

## §4 Cycle d'exécution

1. **Preflight G8** : `Agent(subagent_type: "nexus-phase-preflight-deep")`
   → `.planning/active/post_s75_ux_arrival_preflight.md`. Lui passer §3 +
   les chemins (iroh_runtime.rs gate, seed_registry.rs pattern, http.rs
   BrowseEntryView/list_nodes, Nodes.tsx/Browse.tsx/daemon.ts). Attendre le
   verdict (EXECUTE/PLAN-ADAPT attendu ; DESIGN-CONFLICT → STOP arbitrage PO).
2. Code + tests conformément au verdict.
3. **Fail-fast COMPLET dual-bloc** (Rust touché → les DEUX) en
   `run_in_background` : fmt/clippy/nextest workspace/doctests/release
   Windows + Docker Linux canonique (image `sbfb-ci` re-pinnée OK ; run
   SEUL, le bind-mount sous contention fabrique des timeouts
   `operator_server` — pas des régressions) + bloc web complet. Compteurs
   attendus : Rust ≥ 1755 Win / 1759 Docker (+ tes ajouts), Vitest ≥ 370.
4. **Review** : Workflow 5 dimensions → skeptics → synthèse →
   `post_s75_ux_arrival_review.md`. Corriger P1 + cheap P2/NIT.
5. **Codex** (bloquant) : prompt `.git/CODEX_UX_ARRIVAL.txt` sur le modèle
   `.git/CODEX_SPRINT75_PHASE_G.txt` (Output contract : tokens littéraux
   `GAP`/`CONFIRMED` + `OVERALL: PASS|FAIL` ; SCOPE diff-only ; decisions
   do-NOT-re-litigate = §2/§3.5). `Get-Content -Raw | codex exec
   --dangerously-bypass-approvals-and-sandbox -o
   .planning/active/post_s75_ux_arrival_codex_review.md`. Sortie BRUTE.
   Boucler jusqu'à PASS, réconcilier review → `## Verdict: PASS`.
6. **Commit unique** : `feat(daemon+shell): post-S75 UX-ARRIVAL — hybrid
   arrival surface (observed nodes + subscribed-first grid + ingest
   rate-limit)` — body riche (sections du template, sans l'armement hooks).
   Staging explicite, `git commit -F`.
7. **Acceptance live courte** (assets dispo) : PC daemon
   `--web-root web/dist` port 7654 ; Mac `~/sbfb-test` + `/tmp/sbfb-dist`
   (re-scp `web/dist` après rebuild — ServeDir lit le FS à la volée côté
   PC) ; VPS ancre active. Démontrer : Mac avec abonnements → grille =
   sources abonnées, AUCUN inconnu mélangé ; un annuaire non-abonné entendu
   → apparaît dans /nodes observed + section découverte ; spam de
   re-publications → rate-limité (1/fenêtre).
8. **Memory** : `nexus_grid_pivot.md` Tip + `MEMORY.md` AVANT de rendre.

## §5 Pièges connus (appris S75 + session test)

- **UN SEUL cargo natif à la fois ; jamais de cargo pendant Codex.**
- **JAMAIS git/codex depuis `web/`** (`.git` imbriqué périmé) — racine,
  `git -C`, subshell `(cd web && ...)`.
- PowerShell 5.1 : `2>&1` sur exe natif = faux échec → `$LASTEXITCODE`.
- **Gossip bootstrap figé au boot** (`runtime.rs:1080`) : un subscribe
  post-boot ne re-joint pas le swarm — pour les tests live, config
  `default_curators` AVANT boot ou restart.
- Hook stop process-supervisor : messages d'attente arbre-sale sans les
  sous-chaînes interdites (fr fai-t/corri-g/termin-é/committ/propr-e/prê-t/
  liv-r ; en done/completed/fixed/committed/clean/ready/final/finished) ;
  ne jamais citer la regex ; signaler explicitement l'état non-enregistré.
- Vitest sous charge cargo = timeouts classe `vitest_env_variance` →
  re-run propre avant de conclure.
- Lint React : setState synchrone dans un effect = erreur → pattern
  « adjust state during render ».
- `seedCount`/seed-count : clé React Query partagée
  `["seed-count", coordUrl, pid, hash]` — réutiliser, pas dupliquer.

## §6 Après ce mini-cycle

L'ordre redevient : **S76 Phase 0 = audit gate S75** (`sprint76_audit_plan.md`,
13 tracks — y ajouter une note : vérifier le mini-cycle UX-ARRIVAL comme
surface supplémentaire) puis kickoff S76 GPU partagé cross-machine
(migration `sprint75_*` → archive/v2.1 à ce moment-là). Push origin =
décision PO uniquement.

## DÉMARRAGE

1. §0 lectures + vérité terrain git. 2. Preflight G8 (§4.1). 3. Exécute le
cycle §4 jusqu'au commit + memory. Rien n'est poussé sans demande PO.
