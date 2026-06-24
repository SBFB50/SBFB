# Sprint 77 — Audit plan (audit gate de S76, joue en S77 Phase 0)

**Ecrit** : 2026-06-17 (Phase G Sprint 76).
**Sprint audite** : **Sprint 76** (GPU partage cross-machine — panneau « offrir
ma puissance » + enrolement worker co-localise, dette reservee, E2E cross-machine
compute B-3 + cohorte homogene, quorum redundancy>1 deterministe + fix bridge
result-sync, dashboard contributeur + anti-gaming sanity-bound, quantization 4-bit
doc-only, wrap-up).
**Executeur** : session fraiche S77, Phase 0 (Cas A audit gate).
**Produit attendu** : `.planning/active/sprint76_audit_findings.md`
(verdict PASS / CONDITIONAL PASS / FAIL).
**Tip audite** : commit Phase G `feat(daemon)` wrap-up (HEAD au demarrage S77 ;
tip code phases A-F = `a547de6`).

---

## §0 Mode d'emploi pour la session fraiche S77

**Ordre de lecture impose** (forme une opinion AVANT de lire les self-reports) :

1. Ce fichier (`sprint77_audit_plan.md`) — la feuille de route.
2. Le **diff complet** S76 : `git diff 73831c0~1..<tip Phase G>` (du dernier
   commit pre-S76 au tip). Les commits feat/fix/docs : Phase 0 audit gate S75
   (`73831c0` findings + `23a08c9` fix duress), kickoff `3faee6e`, A `ce43894`
   (offer-my-power panel + co-located worker), B `6904cdd` (duress siblings +
   2-report carries + test/doc debt), C `1cc28e7` (cross-machine compute B-3 +
   homogeneous cohort), D `d75ae77` (redundancy>1 quorum over the bridge, PO
   Option A), E `768e235` (contributor dashboard + D4-Q sanity-bound), F
   `a547de6` (quantization doc-only), G `<tip>` (+ chores intercales : verif
   self-reports `5b07472`/`1de6f8a`/`24bda54`/`df86bdc`, supervisor-supprime
   `42c7448`, README-bootstrap `a21aaad`, agents-enregistres `d6dea45`).
3. `sprint76_kickoff.md` §11 (Checkpoint — D1-D5 arbitres PO geles) + §7 (11
   scope cuts) + le `sprint76_design_review.md` (G1, 3 ajustements D1/D2/D3).
4. Le code livre, dans l'ordre des tracks ci-dessous.

**A NE PAS lire avant d'avoir forme une opinion** :
`sprint76_verification.md` (self-report — l'agent livreur a ecrit le code ET la
verification) et les `sprint76_phase_*_review.md` (reviews du livreur). Les lire
**apres** pour comparer, pas pour se faire une opinion.

**Format du livrable** : `sprint76_audit_findings.md` (§7 ci-dessous).

**Contexte non-standard a connaitre** :
- **S76 a herite de la roadmap v5 amendee (S75-G)** : la decouverte PULL est
  passee AVANT le GPU. GPU = S76 (ce sprint), **sharding = S77** (prochain).
  S76 prouve d'abord le **task-routing du modele ENTIER cross-machine** ; le
  modele eclate (70B sur 2+ machines x 1 GPU) est S77. Ne pas compter ce
  sequencement comme un drift — il est documente et arbitre PO (« personne n'a
  2 GPU » → mono-machine 2-GPU ENTERRE, scope cut #2).
- **2 preflights PLAN-ADAPT (C, F) + 1 (G)** — pas des derives agent, des
  corrections factuelles du plan contre le code reel : **C** `model_digest`
  durci nom→GGUF irrealisable en stock (Ollama n'expose pas le hash GGUF) →
  doc-note honnete + `RuntimeTuple` cohorte sur `required_runtime` ; **F** lien
  doc cliquable irrealisable (la SPA ne sert aucun markdown → 404) → pointeur
  texte non-cliquable ; **G** acceptance LIVE traitee differe-materiel-operateur
  (pas un faux-vert 38/38) + harness palier 2 rendu runnable (parametre
  `REDUNDANCY`). L'audit verifie que chaque PLAN-ADAPT porte une evidence
  ground-truth, pas qu'il aurait fallu suivre le plan d'origine.
- **Phase D a touche un fix PROD en cours de phase (arbitrage PO Option A)** :
  `forward_result_entry` dedupliquait par `task_id` SEUL → pour redundancy=2 les
  2 workers ecrivent la meme cle `result:{task_id}` sous auteurs iroh-docs
  distincts → 2e jete avant validator → **quorum cross-machine JAMAIS forme**.
  Fix = dedup `(worker_pubkey, task_id)` miroir exact du validator (~5 lignes
  prod, 0 wire/dep). Prouve rouge-avant-vert par revert. L'audit verifie le fix
  et sa non-regression, ne rebat pas l'Option A.
- **Phase G a corrige un VRAI gate fmt mal-diagnostique** : `http.rs:8531` (test
  `#[cfg(test)]` du dashboard contributeur, ajoute Phase E `768e235`) contenait
  un appel `credit(...)` non-formate. Les suites D/E/F avaient DIFFERE Docker, et
  le diff etait faussement attribue a une « derive toolchain 1.95 vs 1.94 ». Le
  re-run Docker canonique 1.94 de Phase G a montre le **MEME diff byte-identique**
  → ce n'etait pas un drift mais une vraie violation fmt latente. Phase G l'a
  reformatee (wrapping que les deux toolchains produisent a l'identique) →
  fmt vert sous Win 1.95 ET Docker 1.94. L'audit re-confirme fmt 0 sous les deux.
- **PROCESS S76** : le superviseur long-lived (`nexus-process-supervisor`) a ete
  SUPPRIME en cours de sprint (Phase D, demande PO) ; l'orchestration est passee
  aux Workflows ultracode par etape de phase (preflight/review en fan-out) ; le
  SEUL backstop mecanique restant = hooks lightcheck + Codex au commit. Ne pas
  compter cette suppression comme une regression de discipline.
- **Environnement** : `wsl --shutdown` + restart Docker Desktop = recovery du
  wedge moteur (S76-C : 2 builds Rust lourds CONCURRENTS → OOM linker MSVC).
  Lancer les suites SEQUENTIELLEMENT (Windows seul, puis Docker seul). Le compte
  canonique = Docker Linux sbfb-ci (`rust:1.94`), gate AVANT PUSH
  (`feedback_wsl_before_push`). JAMAIS git/codex depuis `web/` (`.git` imbrique
  perime).
- **Phase G env-sensible** : l'acceptance LIVE cross-machine (B-3 palier 1 +
  quorum palier 2) exige le materiel operateur (PC RTX 5080 + VPS Hetzner + Mac
  + SSH/WAN) ABSENT de l'environnement de session. Differe-trace-user (precedent
  S74 dual-platform). L'audit verifie la posture honnete (DIFFERE marque, aucune
  assertion LIVE non executee) et le harness runnable (`b3_live_pc_vps.sh` avec
  `REDUNDANCY`), PAS la trace live (sauf si l'operateur l'a entre-temps executee
  et consignee).

---

## §1 Critere verdict audit S76

| Verdict | Condition |
|---------|-----------|
| **PASS** | 0 P0, 0 P1, >= 1 P2+ documente |
| **CONDITIONAL PASS** | 0 P0, 0 P1 mais conditions a surveiller S77 |
| **FAIL** | >= 1 P0 ou >= 1 P1 non resoluble dans l'audit |

Rigor signal G4 : PASS exige >= 1 P2+ documente. 0 P0/P1 **et** 0 P2+ = CONCERN
(audit trop superficiel). S76 expose au minimum ~10 P2+ candidats (carries §3 +
findings routes des 6 phase reviews) — un audit qui n'en confirme aucun est
suspect.

---

## §2 Tracks audit S76 (ce que Phase 0 S77 doit verifier)

> **Note de provenance** : les items routes ci-dessous proviennent du parse des
> **6 phase reviews A-F (ratio present : 6/6, toutes `## Verdict: PASS`)** + la
> review G + Codex (rounds cumules : A 8/8, B 12/12, C 3 rounds 8/0/0, D PARTIAL
> 0-GAP, E 14 confirme, F 5/5, G a venir) + les 6 preflights. Ce sont des
> **questions a verifier**, pas des findings confirmes. Chaque item porte une
> severite « si faux ».

### Track A — Suites verification

Rejouer la fail-fast `sprint76_verification.md` (38 rows, plan §Fail-fast).
Attendu :

- **Windows natif** : nextest `--workspace` **1804**, 0 fail 0 skip.
  Progression par phase depuis l'entree S76 **1763** (= S75 close 1755 + fix
  audit `23a08c9` +8) : A 1767 (+4) → B 1775 (+8) → C 1785 (+10) → D 1789 (+4)
  → E 1799 (+10) → F 1804 (+5) → G +0 (wrap-up, fmt-fix whitespace seul).
  Verifier la decomposition au `nextest list` ; le git-count des `#[test]`
  ajoutes par commit doit egaler chaque delta (anti faux-vert).
- **Docker Linux canonique** (row #6) : **1808**, 0 fail 0 skip (run final seul,
  sans charge cargo parallele). L'ecart +4 vs Win = tests `#[cfg(unix)]`
  structurels. **fmt canonique 1.94 = 0** (verifie le fix `http.rs:8531` Phase G,
  byte-identique a Win 1.95).
- **Web** : Vitest **398** (367 entree → 386 Phase A → 396 B → 397 E → 398 F),
  coverage 87.2/79.01/85.92/88.52 >= seuils, size-limit OK, tsc 0, lint
  0-err, `scan-en-strings.sh` clean.
- **Rows cles a re-executer soi-meme** (pas juste lire Observed) : #13/#14/#15
  (snapshot additif 0-bump + enrolement worker public + least-priv OFF), #19
  (downgrade open-source sans provenance), #20 (failover multi-tier), #24
  (routing cohorte homogene), #27/#28 (quorum 2 byte-identique + divergence
  rejetee), #29 (validator inchange — `git diff` quorum vide), #31/#32
  (agregation contributeur EMA + route dashboard), #35 (backend doc-only
  inchange — grep `with_split_mode`/`with_devices` absent), #36 (0 bump wire).
- **0 delta dependances sur tout le sprint** : `git diff 73831c0~1..<tip> --
  Cargo.lock` vide. Toute ligne non-vide = drift non documente (P2).

### Track B — Phase A : panneau « offrir ma puissance » + enrolement co-localise (D1)

Question centrale : le worker co-localise lit-il VRAIMENT le consentement user
sans escalade de privilege, et le champ snapshot est-il additif 0-bump ?

- `ConsentSnapshot` additif : `SCHEMA_VERSION: u32 = 1` INCHANGE (le worker
  co-localise copie `level`+`caps` SEULEMENT si OpenSource/All, jamais
  `own_node_id`). Re-executer `consent_snapshot_serializes_additively`.
- **Fix prod prefixe route** `/api/v1/consent*` (trou prod : le front POST-ait
  une route inexistante) — re-executer `consent_route_reaches_daemon_prefix` +
  verifier `vite.config.ts` proxy.
- `CONSENT_LEVEL` named-constant (refactor PO, magic numbers elimines) : grep
  qu'aucune comparaison de niveau de consent n'utilise un litteral nu.
- Double-confirm L4 (front) + jauge caps : verifier le pattern front Phase A.

### Track C — Phase B : dette reservee (duress + carries 2-reports + test/doc)

Question centrale : les 3 carries « 2-reports » sont-ils REELLEMENT fermes
(anti-escalade G7), et les no-op duress ne mutent-ils vraiment rien ?

- **CARRY-3** (B2) : downgrade `trustworthy_open_source` re-applique a l'INGRESS
  `handle_project_announcement` AVANT `add_direct_entry` (pas seulement a l'index
  FTS5). Re-executer `aggregator_downgrades_open_source_without_provenance`.
- **LOOPBACK-TIERS** (B7) : 7 routes S74/S75 inscrites
  `LOOPBACK_ENDPOINTS_TRUST_TIERS.md §3` + phrase fausse audit corrigee.
- **PULL-3** (B3) : `build_seed_fetch_chain` chaine ordonnee ticket-mort →
  directory → multi-provider, vraie boucle (pas selection), `delete_tag` avant
  tier suivant, codes 400/404/502 preserves. Re-executer
  `pull_falls_back_across_tiers_when_ticket_dead`.
- **Duress no-op** (B1) : `seed_voluntary` + `set_keep_online` early-return AVANT
  mutation/emit ; bytes leurre == succes. Re-executer
  `seed_voluntary_noop_in_duress` + `set_keep_online_noop_in_duress` (0 row,
  0 tag).
- T6 GossipCmd::Outbox (B4), WS-3/PD-5 hoisting (B5), discriminateur
  curator/ancre (B6), 5 pages front smoke (B9), bridge allowlist parite (B10).

### Track D — Phase C : E2E cross-machine compute B-3 + cohorte homogene (D2/D3)

Question centrale : la cohorte au CLAIM-GATE est-elle bien advisory (le worker ne
claim pas si tuple mismatch, la tache reste live), et `model_digest` durci est-il
honnetement documente comme champ mort ?

- `RuntimeTuple{model, quant, runtime_family}` sur `Task.required_runtime`,
  `#[serde(default)]`, 0 bump `TASK_FORMAT_VERSION`/`DOMAIN`/dep ; `matches()`
  wildcard-sur-vide. Re-executer `submit_sets_required_runtime_only_for_verifiable_redundant`
  (dispatcher pose le tag) + `cohort_gate_admits_homogeneous_worker`/`cohort_gate_blocks_non_homogeneous_worker` (worker enforce).
- Cohorte au CLAIM-GATE worker (PULL : ne claim PAS si tuple mismatch → continue,
  tache reste live) ; dispatcher pose `required_runtime` SEULEMENT si
  `verifiable && redundancy>1`.
- **`model_digest` = blake3(name)** doc-note (Verifier : 0 appelant prod = champ
  mort ; vrai chemin = `validate_quorum_pre_guardrail` sur `result_text` ; GGUF
  hash = S77). Verifier que la doc-note est honnete et que rien ne traite ce
  digest comme une frontiere de confiance.
- Gate anti-regression `e2e_network_execute_gate_real_http_no_frontier_mock` :
  hors-diff, vert.
- **Acceptance LIVE B-3 (row #26)** : DIFFERE-materiel-operateur. Auditer la
  TRACE si l'operateur l'a executee ; sinon verifier la posture honnete (harness
  `b3_live_pc_vps.sh`, critere <30s = BLOCK-a-diagnostiquer).

### Track E — Phase D : quorum redundancy>1 deterministe + fix bridge (D3)

Question centrale : le fix bridge `(worker_pubkey, task_id)` ferme-t-il VRAIMENT
le trou quorum cross-machine, et le validator reste-t-il INCHANGE ?

- Fix `forward_result_entry` (result_sync.rs) : dedup `(worker_pubkey, task_id)`
  miroir EXACT du validator (avant : `task_id` seul → 2e worker jete). Prouve
  rouge-avant-vert par revert. Re-executer
  `quorum_redundancy_two_workers_reach_validator` +
  `quorum_redundancy_diverging_outputs_rejected`.
- `verifiable_seed` = u32 LITTLE-ENDIAN de `blake3(task_id)[..4]` (PAS le digest).
  Re-executer `verifiable_seed_is_cross_worker_stable`.
- **Validator INCHANGE** : `git diff --stat validator.rs` sur le quorum = 0 ligne ;
  `validate_quorum_pre_guardrail` diff-vide. Re-executer `validator_quorum_unchanged`.
- **3-noeuds E2E RETIRE** (P2 reconciliation Codex : timeout 124s sous `cargo
  test` shared-process contention pleine-crate). Quorum prouve PAR COMPOSITION
  (2 hermetiques vrai bridge + `worker_result_syncs` redundancy=1 existant +
  Phase G LIVE). Verifier que la composition couvre, ou inscrire le gap.
- **Acceptance LIVE quorum (row #30)** : DIFFERE-materiel-operateur ; harness
  rendu runnable Phase G (`REDUNDANCY=2` + 2e worker homogene). Auditer la trace
  si executee, sinon la posture + le harness.

### Track F — Phase E : dashboard contributeur + anti-gaming sanity-bound (D4)

Question centrale : l'agregation reutilise-t-elle l'EMA EXACT du leaderboard, et
le P1 `generation_time_ms: 0` hardcode est-il fixe a la ROOT (worker mesure
reel) et pas band-aide ?

- `get_contributor_summary` reutilise `effective_score()` EMA EXACT (alpha=0.97) ;
  index `idx_kudos_worker` PRE-EXISTANT 0-M20 (EXPLAIN QUERY PLAN). Re-executer
  `get_contributor_summary_aggregates_ema` + `counts_tasks_served` + `_empty`.
- **P1 fix ROOT-CAUSE** : worker mesure `Instant` reelle autour de `generate()`
  (avant : `generation_time_ms: 0` hardcode latent, 1er consommateur = le
  sanity-bound). `StubBackend::with_delay_ms` + E2E assert `>=1` + test
  non-regression. Verifier que le fix est a la source, pas un clamp cosmetique.
- `sanity_bounded_tokens` clamp AVANT `log_utility` ; raw_kudos RETIRE (champ
  mort) ; GPU-heures `usage.json` LOCALES (jamais agregees reseau). Route
  `/api/v1/contributor/{node_id}` dans `authed_routes`.
- **MEDIAN-DE-GROUPE** (DOC-P2, D4-Q option a non implementee) : verifier que la
  doc l'enregistre honnetement (THREAT_MODEL §15.3 DEFERRED) et que le
  sanity-bound per-entry livre suffit pre-launch.

### Track G — Phase F : quantization 4-bit doc-only (D5)

Question centrale : la doc est-elle honnete sur les caps VRAM (lit
`estimated_vram_mb` DECLARE, pas la taille GGUF), et le backend est-il VRAIMENT
inchange (anti scope-creep) ?

- `docs/operators/QUANTIZATION.md` : reco GGUF par carte + table empreintes VRAM
  + cible single-GPU <=14B modele entier + 70B=sharding S77 + pre-condition
  quorum redundancy>1 = MEME GGUF (exactitude, PAS anti-Sybil, renvoi §15.2).
- **Backend inchange** : `git diff llama_cpp.rs` vide ; grep `with_split_mode`/
  `with_devices` = 0 (tensor-split = S77 rejete). Re-executer
  `llama_cpp_unchanged_doc_only`. 5 tests integration lecture texte.
- Pointeur front non-cliquable `GpuConsentDialog.tsx` (Option B PLAN-ADAPT : SPA
  ne sert aucun markdown → href relatif=404). Verifier que ce n'est pas un lien
  mort presente comme cliquable.

### Track H — Phase G : wrap-up + fmt-fix root-cause + acceptance + Arc 3.5 close

Question centrale : le fmt-fix `http.rs` est-il bien la VRAIE cause (pas un
band-aid masquant un drift), et l'acceptance LIVE est-elle honnete (DIFFERE, pas
faux-vert) ?

- **fmt-fix** `http.rs:8531` : le wrapping applique est-il byte-identique a la
  sortie rustfmt 1.94 ET 1.95 ? Re-executer `cargo fmt --all --check` sous les
  deux toolchains = 0. Verifier que le diagnostic « faux-positif toolchain » des
  suites 13-19 est corrige dans verification.md + memory (honnetete meta).
- **Harness palier 2** : `b3_live_pc_vps.sh` parametre `REDUNDANCY` (defaut 1)
  cable a `redundancy_factor` + section enrolement 2e worker homogene. `bash -n`
  clean. Verifier que le row #30 est REELLEMENT runnable (pas juste documente).
- **Acceptance LIVE** : verification.md trace 36/38 verts session + #26/#30
  DIFFERE-trace-user + #6 Docker recovery. Verifier qu'AUCUNE assertion LIVE
  non-executee n'est claimee comme verte.
- **Docs longue-vie** : THREAT_MODEL v9, PATTERNS §P62 + shell P38, SPRINT_LOG row
  S76, CLAUDE.md 0-76 CLOSED + Arc 3.5 6/6, roadmap_v5 livraison. Verifier
  coherence (pas de row STRIDE dupliquee, §P60.x non recree).

### Track I — Wire 0-bump + pre-launch policy (transverse)

- `*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`/`SCHEMA_VERSION` tous a 1 sur
  `73831c0~1..<tip>`. `RuntimeTuple` + `ConsentSnapshot` strictement additifs
  `#[serde(default)]`. `canonical.rs` 0 changement structurel. 0 nouveau
  `DOMAIN_*` non additif. Re-executer row #36.
- Pre-launch policy respectee : aucun test « legacy decode » zombie ajoute ;
  `#[serde(default)]` documente runtime-tolerance vs historical-compat.

### Track J — Meta-process

- **6/6 phase reviews `## Verdict: PASS`** + 6 codex_review.md bruts (non
  reecrits par Claude) + 6 preflights. Verifier la presence et l'authenticite
  (lightcheck Check 7).
- **Superviseur supprime** (`42c7448`) : verifier que `nexus-process-supervisor`
  n'est plus enregistre (settings.json) et que les hooks lightcheck restent le
  backstop. Pas un gap — un changement de process documente.
- **README bootstrap** (`a21aaad`) : marqueurs `<!-- BOOTSTRAP:BEGIN/END -->`
  greppables. Verifier que CLAUDE.md pointe la bonne plage.
- Commit bodies 9 sections + delta tests cumule + scope cuts exhaustifs sur les
  7 phases.

---

## §3 Carries re-routes vers S77 (a traiter ou re-router au kickoff S77)

| Carry | Source | Compteur | Pourquoi differe / exemption |
|---|---|---|---|
| **SYBIL-SEEDER-TAIL** | S75 audit P2 (track D/E) | **2/3 → 3/3 si non fait** | **EXEMPTION NOMMEE « dependance interne sharding »** : S77 touche le dial-set/topology, le sampling se regroupe naturellement avec le sharding (pas un fix isole) ; residuel availability-only non-securitaire, ancre slot-0 non-crowdable, verrou tient. SANS exemption = MANDATORY S77. Confirme non-traite Phase B (grep code = NONE). **Seul 2-report reconduit.** |
| **REVISION-HOME-DURABILITY** | S75 audit P2 (track C) | 1/3 → 2/3 | pas d'exemption ; mitige systemd `SBFB_HOME` epingle ; surveiller si un mode deploiement sans home pinne apparait ; pas exploitable pre-launch |
| **KNOWN-ENTRY-OVERCOUNT** | S75 audit P2 (track D) | 1/3 → 2/3 | pas d'exemption ; superset HONNETE (curator-list + annuaire) ; dedup `(pid,hash)` requis SEULEMENT si une UX future affiche « N apps decouvrables » ; pas de consommateur UI = pas de bug aujourd'hui |
| **seeder `catalog_len:0`** | S75-G constat acceptance live | 1/3 → 2/3 | pas d'exemption ; bloque sur **arbitrage PO design** (pas code) : section « seeded » distincte non-autoritaire dans `NodeDirectoryEntry` vs verrou-4 (seeder != editeur) + modele F-Droid |
| **RE-DRIVE-ON-INGEST** | S75 audit P3 (track F) | 1/3 → 2/3 | pas d'exemption ; remede operateur documente (restart) ; lie SeedAnnounced/PULL-3 — si la convergence se resout, peut se fermer en cascade |
| **T-NN+3** (canonical_bytes dup JCS) | carry ouvert S70 | open S70 (non chiffre) | pas d'exemption ; absorbable au prochain sprint touchant JCS crypto ; pas force par scope GPU/sharding |
| **P3-D-3** (send-failure un-mark `seen.remove` sans test dedie) | sprint76_phase_d_review.md:303 (route EXPLICITE) | **1/3 NOUVEAU** | branche defensive 1-ligne, cle composite verifiee par lecture, chemin principal couvert ; **ABSENT du plan §10 G.1 — ajoute ici explicitement** |
| **MEDIAN-DE-GROUPE** anti-gaming (D4-Q option a) | sprint76_phase_e_review (PLAN-ADAPT #2) | **DOC-P2 NOUVEAU** | sanity-bound per-entry livre (clamp tokens<=f(gen_ms)) ; option « durcir via median du groupe d'accord » deferee ; THREAT_MODEL §15.3 DEFERRED ; **ABSENT du plan §10 G.1 — ajoute ici** |
| **B10-PARITE-FIXTURE** (allowlist bridge 2 miroirs hand-maintained) | sprint76_phase_b_review.md:28/:268 (P2 route) | DOC-P2 | la parite Rust allowlist (`allowlist_mirrors_host_dispatch_schema`) ↔ TS dispatch (`protocol.test.ts`) est test-locked des 2 cotes mais PAS depuis une fixture partagee ; un 3e champ ajoute a une seule liste passe les 2 tests si l'autre est mis a jour separement. Maintenabilite : fixture partagee S77+ si une phase touche le bridge ; pas un bug aujourd'hui (2 cotes verrouilles) |

**Residuel surveille (PAS un carry actionnable seul)** : SYBIL-FORGE-COHERENTE
(Sybil multi-keypair, assume M) — documente §15.2/§15.3, coût Sybil pre-existant
(PoW/AgeWitness + pilote ferme). A mentionner, pas a fixer isolement.

**NE PAS reconduire (deja landes / fermes)** : les **3 carries 2-reports**
CARRY-3 (B2) / LOOPBACK-TIERS (B7) / PULL-3 (B3) FERMES Phase B (`6904cdd`,
Codex 12/12) ; P3-THREAT-MODEL-COHORT-ROW (§15.2 LANDEE THREAT_MODEL.md:895-916) ;
duress-freres B1 (FERME §15.1:884) ; P3-D-4 (log slice cosmetique « No action
required ») ; P3 editoriaux doc QUANTIZATION Phase F (corriges en-phase avant Codex) ;
UX-ARRIVAL (couverte S76-B).

**Externes inchanges (a reporter tels quels, escalade G7 < 3 reports)** : P2-A-1
rand (exemption upstream), P2-AUDIT-2 iroh pre-release transitives (pin 0.98),
T-NN+2 iframe Rust-wasm (PATTERNS §P34), P3-OS-1 `operator_server` OR duplique,
LT-3/LT-4/LT-7 hors-sprint. **LT-2** : trigger ARME + dry-run prive FAIT — flip
publie = decision PO hors-sprint. **LT-5** : RESORBE (quorum DB-Rust, NE PAS
re-coder dispatcher Python). Verifier qu'aucun n'atteint 3 reports sans exemption.

---

## §4 S77 Objective — sharding pipeline (contexte, hors audit)

Apres l'audit S76, S77 ouvre (roadmap v5, scope cut #1) le **sharding pipeline** :
un modele trop gros pour une seule carte 16GB (70B = 42.5 GB) eclate sur **2+
machines x 1 GPU** (cross-machine, jamais mono-machine 2-GPU — arbitrage PO
« personne n'a 2 GPU », scope cut #2). S76 a livre le prerequis : le task-routing
du modele ENTIER cross-machine (B-3 + quorum redundancy=2 + cohorte homogene).
**Pre-requis a inscrire au plan S77** (carries §3) : SYBIL-SEEDER-TAIL (le
sampling se regroupe avec le dial-set/topology du sharding), P3-D-3 (si le
pipeline ajoute des chemins result-sync), et l'etage-2 TOPLOC (`logprobs_hash`,
slot pose S76, requis pour le quorum cross-GPU heterogene que le sharding peut
imposer).

---

## §5 Out of scope pour l'audit (NE PAS rebattre)

L'audit S76 **audite**, il ne re-concoit pas. Ne pas rebattre :
- **D1-D5 gelees** (kickoff §11, arbitrage PO Checkpoint) : D1 OpenSource+All
  ouvrent le partage (least-priv OwnProjects/Whitelist ; worker co-localise lit
  consent.json) ; D2 E2E cross-machine + convergence `result:` WAN 1er critere
  falsifiable ; D3 quorum redundancy>1 cohorte homogene + durcir model_digest
  doc-note ; D4 kudos per-task + durcir anti-gaming sanity-bound ; D5 doc-only
  + **mono-machine 2-GPU ENTERRE**.
- **Les 11 scope cuts** (kickoff §7) : sharding (#1, S77), tensor-split
  mono-machine (#2, rejete), VRAM-live admission (#3, S77), median-de-groupe
  (#4, P2), TOPLOC etage 2 (#5, post-S77), quorum cross-GPU heterogene (#6,
  post-S77 TOPLOC), `execute_build` LT-7 (#7, S77), reconnaissance contributeur
  publique (#8, post-launch), self-test enrolement (#9, rejete), scheduler idle
  BOINC (#10, post-launch), AWQ/GPTQ/EXL2 (#11, rejete).
- **Les 3 PLAN-ADAPT (C/F/G)** sont des adaptations d'implementation DANS le cadre
  D1-D5, deja arbitrees (evidence ground-truth). L'arbitrage PO Option A Phase D
  (fix bridge dans la phase) ne se rebat pas.
- **Pre-launch policy** : pas de bump `*_VERSION` tant que rien n'est pousse ;
  canonical librement editable ; ne PAS exiger de migration wire.
- **Le pin iroh 0.98** et les arbitrages PO (amendement roadmap, sign-off D3,
  superviseur supprime).
- Re-corriger un P2/P3 deja documente (router vers S77+ phases, pas le
  re-implementer en Phase 0).

---

## §6 Verdict global attendu

- **PASS** : 0 P0, 0 P1 → S77 Phase A demarre direct. **Scenario attendu** — les
  ~10 P2 candidats sont des conceptions deferees (SYBIL-SEEDER-TAIL avec exemption
  sharding, MEDIAN-DE-GROUPE, P3-D-3) ou des residus documentes (model_digest
  champ mort, 3-noeuds E2E retire par composition, acceptance LIVE differe-user) ;
  les invariants headline tiennent (snapshot 0-bump ; fix bridge quorum
  rouge-avant-vert ; validator inchange ; P1 gen_time_ms fixe a la root ; backend
  quant inchange ; fmt vert sous les 2 toolchains ; 0 bump wire) ; G1 present.
- **CONDITIONAL PASS** : 1-3 P1 fixables → S77 Phase A bloque tant que les
  `fix(sprint76): ...` ne sont pas landed. **Candidats P1 a trancher** :
  (1) Track E — le quorum cross-machine n'est prouve QUE par composition
  (2 hermetiques + LIVE differe) : si la composition ne couvre pas un chemin
  reel, P1 ; (2) Track F — le fix `generation_time_ms` reel ne se propage pas a
  tous les chemins de credit → kudos toujours gonflables ; (3) Track D — un
  chemin traite `model_digest`/cohorte comme une frontiere de confiance (pas
  advisory) → faux sentiment de securite ; (4) Track A — un delta test annonce ne
  correspond pas au git-count (faux-vert).
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle.

---

## §7 Livrable final attendu

`sprint76_audit_findings.md` (pattern Sprint 6/7), sections :
1. **Auditeur** — id session, duree.
2. **Tip audite** — SHA master pris comme base (tip code A-F `a547de6` + G).
3. **Verdict global** — PASS / CONDITIONAL PASS / FAIL.
4. **Une section par track (A-J)** avec verdict (PASS / CONCERN / FAIL) + findings.
5. **Findings list sorted by severity** — table P0 → P3.
6. **Commits fix attendus** — si CONDITIONAL PASS, liste `fix(sprint76): ...`
   prealable au kickoff S77.
7. **P2 a logger en tech debt** — items vers `PATTERNS.md` sans code change.
8. **P3 laisses sans action** — nits ignores.
9. **Notes on audit completeness** — ce qui n'a pas ete couvert et pourquoi
   (notamment l'acceptance LIVE cross-machine si l'env bloque : consigner,
   auditer la posture + le harness runnable a la place).

**Critere SMART** : la fail-fast `verification.md` rejoue verte (Windows nextest
1804 0 fail + Docker Linux canonique 1808 0 fail + fmt 0 sous les 2 toolchains +
web 398/coverage/size) + 0 bump wire + 0 P0/P1 non resolu = S77 kickoff debloque
(sharding pipeline).

**Exit Gate** : l'audit S76 est complet quand `sprint76_audit_findings.md` porte
un verdict avec >= 1 P2+ (G4), couvre toutes les tracks A-J, ingere le diff
complet S76 (Phase 0 + A-G), confirme les invariants headline (fix bridge ferme
le trou quorum ; validator inchange ; P1 gen_time_ms root-fixe ; fmt root-fixe ;
backend quant inchange ; 0 bump wire), et tranche les 4 candidats P1 du §6.
