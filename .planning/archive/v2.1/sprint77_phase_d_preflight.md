# Sprint 77 — Preflight Phase D (G8)

## Verdict: PLAN-ADAPT

> Codable, 0 Day-0 touchée, 0 escalade PO. Le plan §7 est implémentable
> tel quel sur le coeur algorithmique (water-filling DP + k-medoids + seuil
> sharding + ShardPlan figé Phase C) ; PLAN-ADAPT car **2 références du plan
> §7.2 sont périmées** (API RTT et nom du champ VRAM) et une **prescription
> de déterminisme** s'impose sur le sampling SYBIL absorbé. Aucune de ces
> corrections ne contredit une décision figée.

---

## Résumé exécutif

Phase D = **Phase 1 du scheduler Parallax (PLACEMENT)** : un **calcul interne**
côté initiateur qui, à partir de mesures locales (VRAM libre par worker +
matrice RTT pairwise), produit un `ShardPlan` (primitive **figée Phase C**,
`shard_plan.rs:188`). C'est du scheduling en mémoire — **aucun nouveau payload
signé, aucun bump wire, aucun DOMAIN_\* net-new** (S4 EXECUTE binaire).

Les 5 scans convergent :

| Scan | Axe | Contribution |
|---|---|---|
| **S1a** | SOTA / OSS | PLAN-ADAPT (algo SOTA-aligné Parallax + PAM BUILD ; 2 corrections wiring/doc) |
| **S1b** | deps / CVE | EXECUTE (0 nouveau dep, hand-roll ; 1 CONCERN code-shape déterminisme) |
| **S2** | décisions figées | EXECUTE (0 DESIGN-CONFLICT ; 2 BLOCKER = contraintes de forme du plan/C, pas des conflits) |
| **S3** | threat model | PLAN-ADAPT (placement = 0 surface signée ; sampling SYBIL à seeder par blake3) |
| **S4** | wire format | EXECUTE (0-bump confirmé, types d'entrée internes non-sérialisés) |

**Point META tranché (3 PLAN-ADAPT A/B/C même cause no-float) :** Phase D
**n'est PAS la 4e occurrence** du pattern float-R&D-vs-repo. Le scheduler est
du calcul mémoire, pas un payload JCS signé. Un `f64` pour les ratios VRAM et
les distances RTT **EN MÉMOIRE est LICITE** (S1A-1, S2-F3, S3-F2, S4-2 — quatre
scans concordants). La seule contrainte : **la SORTIE `ShardPlan` reste 100%
entière** (`layer_start/layer_end:u32`, `[u8;32]`) et aucun float ne fuit dans
un canonical signé ni un `Eq` dérivé sur struct wire. Donc PLAN-ADAPT ici a une
**cause DIFFÉRENTE** de A/B/C (wiring/doc + déterminisme), pas la divergence
float-canonical.

**Pourquoi PLAN-ADAPT et non EXECUTE pur :**
1. Le plan §7.2 (`sprint77_plan.md:226`) cite `conn.stats()` pour le RTT — **périmé**.
   L'API réelle établie Phase B est `conn_rtt(conn) -> Option<Duration>`
   (`shard.rs:160`) ; `conn.stats()` n'a **aucun champ rtt** sur le fork `noq`
   installé (`shard.rs:158` doc explicite).
2. Le plan §7.2 (`sprint77_plan.md:223`) + le contexte parlent de
   `GpuInfo.vram_free_bytes` — **mauvais nom** : `GpuInfo` (`gpu/mod.rs:81-95`)
   ne porte que `vram_total_bytes` ; le champ mesuré libre vit sur `GpuStats`
   (`gpu/mod.rs:107`).
3. Le sampling SYBLIL absorbé doit être **déterministe-non-lexicographique**
   (seed `blake3(session_id)`), pas un `thread_rng()`, pour résoudre la tension
   reproductibilité-des-tests / anti-crowding (S3-F4, S1b-2).

**Pourquoi PAS DESIGN-CONFLICT :** aucune Day-0 n'est contredite. D3
(pipeline-parallel + k-medoids empirique RTT, géoIP REJETÉ), D5 (ComputeGroup
admission intacte), scope cut #7 (lecture `vram_free` au placement seulement,
pompe runtime garde `estimated_*`), 0-bump wire pre-launch, iroh 0.98 pinné —
**tous respectables par le plan tel qu'écrit**.

---

## S1a — SOTA / OSS (contribution PLAN-ADAPT)

L'algorithme est **aligné SOTA et codable** : Parallax (arXiv 2509.26182)
Phase 1 = model allocation DP + heuristiques latency-dominant + water-filling
sous contraintes mémoire/lien (= notre placement) ; Phase 2 DAG DP = notre
Phase E. k-medoids **PAM BUILD** est déterministe par construction (arXiv
1810.05691 / 2008.05171 : *"PAM does not use random elements during
initialization"*, BUILD O(n²k)) — trivial à 3-5 peers.

- **[INFO] S1A-1** — Float interne LICITE, pas la 4e occurrence no-float.
  La sortie `ShardPlan` est all-u32 (`shard_plan.rs:150-156`), jamais signée
  seule ; `canonical.rs` no-float ne couvre QUE les payloads JCS signés ;
  l'addendum §2 décrit le placement comme un calcul de montage de session
  non-wire. → L'implémenteur PEUT utiliser `f64` pour les ratios VRAM et les
  distances RTT EN MÉMOIRE. **Ne PAS forcer une arithmétique entière
  artificielle par mimétisme A/B/C.** Préférer néanmoins une matrice RTT
  **entière** (`Duration` → ms/ns `u64`) pour la reproductibilité exacte des
  tests k-medoids.
- **[CONCERN] S1A-2** — Source RTT du plan PÉRIMÉE. Phase B (correction A1) a
  établi que le fork `noq` n'a pas de champ `rtt` dans `ConnectionStats`
  (`shard.rs:51,158`) ; l'API réelle est `conn_rtt(conn) -> Option<Duration>`
  (`shard.rs:160`), vs `sprint77_plan.md:226` + `kickoff` l.374 (`conn.stats()`).
  → Le scheduler ingère la matrice RTT depuis `conn_rtt()`, **pas** `conn.stats()`.
  Type d'entrée = `Duration` (entiers), `Option` pour le cas "pas encore
  d'échantillon" (connexion fraîche) ; le placement doit traiter le `None`
  (exclure du dial-set ou pénaliser).
- **[INFO] S1A-3** — k-medoids DÉTERMINISTE pour des `ShardPlan` reproductibles.
  PAM init BUILD est déterministe (SOTA) ; `ShardPlan` dérive `Eq`
  (`shard_plan.rs:187`) → comparaison exacte en test ; le test
  `kmedoids_groups_low_rtt_consecutive_layers` exige un résultat stable. →
  Hand-roll k-medoids avec **init BUILD** (medoïde 1 = point minimisant la
  somme des distances ; suivants = celui réduisant le plus le coût total) puis
  swap-phase jusqu'à convergence. **AUCUN rand requis.** Si random-restart
  retenu : `StdRng` graine FIXE (miroir quorum-déterministe-seed-fixe S71-B-2).
- **[INFO] S1A-4** — Le scheduler définit ses PROPRES types d'entrée :
  `nexus-worker-core` n'est PAS dep du coordinator (`Cargo.toml:11-29`). →
  Nouveau module avec un type local (ex `WorkerVramProfile { vram_free_bytes:u64 }`)
  construit côté appelant depuis `GpuStats`/`conn_rtt`. Sortie = la primitive
  figée `nexus-core-rs::ShardPlan`. 0 nouveau dep, 0 import worker-core.
- **[INFO] S1A-5** — Seuil sharding figé et codable : shard SSI
  `VRAM_modèle_quantifié > VRAM_max_d_un_worker`, sinon endpoint federation
  (addendum §5 l.135-140 ; `kickoff` l.387-388 ; test #2). → Retour =
  variante `EndpointFederation`/`None` (pas un `ShardPlan` dégénéré à
  1 assignment couvrant tout).
- **[INFO] S1A-6** — SYBIL-SEEDER-TAIL : sampling du tail seeder dans la
  sélection des candidats du dial-set ; plan autorise **test OU doc-note**
  (`kickoff` l.667). → Si randomisation : graine FIXE ; sinon doc-note honnête
  (sélection déterministe par RTT/availability sans tirage). **Ne PAS**
  introduire de rand non-graine qui rendrait le `ShardPlan` non reproductible.

## S1b — deps / CVE / déterminisme (contribution EXECUTE)

0 nouveau dep (`rand`/`blake3`/`serde` déjà présents ; **aucun crate
clustering** dans `Cargo.lock` — kmeans/kmedoids/linfa/ndarray/nalgebra/
smartcore = 0 occurrence → hand-roll). `nexus-worker-core` confirmé non-dep.
0 CVE nouveau (seul `RUSTSEC-2026-0097` exempté, scope `ThreadRng` hors
périmètre si on évite `thread_rng`).

- **[INFO] S1b-1** — DP water-filling + k-medoids hand-roll avec les deps
  existantes ; 0 churn `Cargo.toml`, 0 advisory nouveau (boucles sur
  `Vec`/slices, pas de crate matricielle).
- **[CONCERN] S1b-2** — **Déterminisme k-medoids/sampling** : `StdRng::seed_from_u64`
  (ou init déterministe sans rand), **JAMAIS `thread_rng()`**. `deny.toml:64`
  exempte UNIQUEMENT la voie OsRng-direct ; or `dispatcher.rs:81` /
  `canary_input.rs:230` / `upload_queue.rs:107` utilisent `random()`/`thread_rng()`
  (non-déterministe + `ThreadRng`). → Recommandé : init **déterministe** (medoids
  triés/dérivés, **pas de rand du tout**) ; sinon `StdRng::seed_from_u64(seed)`.
  **Ne PAS copier le pattern `thread_rng()` des modules voisins.**
- **[INFO] S1b-3** — `nexus-worker-core` n'est et ne doit pas devenir dep du
  coordinator ; le scheduler crée ses structs d'entrée locales. La valeur
  `vram_free_bytes` transite via la couche capability/consent, pas par un
  import direct.
- **[INFO] S1b-4** — 0 CVE/advisory actif au-delà de l'exemption en place et
  hors périmètre ; `cargo deny check` reste vert sans nouvel ignore tant qu'on
  reste sur la voie déterministe/OsRng-libre.
- **[INFO] S1b-5** — Le carry SYBIL existant vit dans `nexus-shell-daemon`
  (`http.rs:1704-1709`, dial-set seeder), pas dans coordinator ; l'item (c)
  Phase D est un sampling DANS le scheduler sur la liste de workers candidats.
  Même contrainte de seed-fixe (test `sybil_seeder_tail_sampling_*`).

## S2 — décisions figées (contribution EXECUTE)

**Aucun DESIGN-CONFLICT** : `git log` 0 décision antérieure contraignant le
placement ; les 4 verrous pertinents (D3 anti-recentralisation/géoIP rejeté,
scope cut #7 pas-de-pompe-VRAM-runtime, no-float-payloads-signés,
[0..L)-délégué-au-scheduler) sont tous RESPECTABLES.

- **[INFO] S2-F1** — Seul verrou = D3 (géoIP REJETÉ, `kickoff` l.399-403). →
  k-medoids opère **uniquement** sur la matrice RTT pairwise mesurée
  (`conn_rtt`/`PathId::ZERO`). **Interdiction** de toute table géo, ASN-lookup
  ou autorité de région centrale.
- **[CONCERN] S2-F2** — Nom de champ : le contexte/plan disent
  `GpuInfo.vram_free_bytes` mais le champ vit sur `GpuStats` (`gpu/mod.rs:107`),
  pas `GpuInfo` (`gpu/mod.rs:81-95` = `vram_total_bytes` seul). Doc/naming, PAS
  un conflit. → Phase D nomme le champ `vram_free_bytes` (vérité mesurée) ;
  doc-note = "mesure = `GpuStats.vram_free_bytes` côté worker", pas `GpuInfo`.
- **[INFO] S2-F3** — Floats LICITES dans l'algo interne ; le no-float lock ne
  s'applique QU'aux payloads signés (`shard_plan.rs:46-53`). Sortie
  `ShardAssignment` reste 100% entière. C'est le point clé tranché : **PAS** un
  4e PLAN-ADAPT "même cause".
- **[BLOCKER] S2-F4** — **Scope cut #7** : ne PAS câbler de pompe VRAM-live
  runtime ni modifier le check consent `estimated_vram_mb` (`consent.rs:422-426`).
  Phase D lit `vram_free_bytes` (mesure) au **placement seulement**.
  **INTERDIT** : toucher `consent.rs`, câbler `gpu.snapshot()`/`GpuStats` live
  dans la pompe runtime, faire dépendre l'admission worker de la VRAM-live.
  Violation → review BLOCK. *(Contrainte de forme portée par le plan, pas un
  conflit à arbitrer.)*
- **[BLOCKER] S2-F5** — Le `ShardPlan` produit DOIT satisfaire
  `is_pipeline_contiguous()` **ET** couvrir exactement `[0..L)` (check stateful
  délégué Phase D, `shard_plan.rs:209`). `is_pipeline_contiguous` vérifie
  `layer_start<layer_end` + `next.layer_start==prev.layer_end` mais **PAS**
  `first==0`/`last==total`. → Phase D ajoute le check de couverture totale :
  `assignments.first().layer_start==0 && assignments.last().layer_end==total_layers`.
  `placement_handles_5_workers_70b` asserte les deux. *(Contrat explicite délégué
  par Phase C, pas un choix.)*
- **[CONCERN] S2-F6** — `WorkerCapability` n'existe pas en code
  (`grep crates/` = No files found ; R&D §10.3 seulement). → Type d'entrée
  LOCAL non-signé (ex `WorkerPlacementProfile { worker_pubkey:[u8;32],
  vram_free_bytes:u64 }`) + matrice RTT. **Éviter de baptiser ce type
  `WorkerCapability`** pour ne pas suggérer un payload wire. Aucun DOMAIN_\*
  net-new en D (§19 ne liste que compute_group/shard_plan/run_proof/activation_commit).
- **[INFO] S2-F7** — **Named constants** (README §6.9 l.1576-1597) : seuil
  sharding, nb min/max de shards, `k` de k-medoids, ratios water-filling =
  constantes nommées uniques. Pas de magic number nu (vérifié en review).
- **[INFO] S2-F8** — Seuil sharding figé : ne sharder QUE si
  `VRAM_modèle > VRAM_max_worker` (comparaison sur `vram_free_bytes` mesuré =
  max des workers, pas déclaré), sinon variante federation/no-shard. Test #2.

## S3 — threat model (contribution PLAN-ADAPT)

EXECUTE sur le coeur (placement = calcul interne, 0 nouveau payload signé ;
admission ComputeGroup intacte `shard.rs:227` ; menaces SI-\* du SPLIT runtime
hors scope D ; floats internes licites car `conn_rtt` rend `Duration` entière).
UNE adaptation concrète sur le carry absorbé.

- **[INFO] S3-F1** — Placement = 0 nouvelle surface signée. Les menaces SPLIT
  (SI-1 activation reconstruction High, SI-3 fingerprinting, SI-4 collusion,
  SI-5 latence side-channel, `SPLIT_INFERENCE_DESIGN.md:196-213`) sont du
  RUNTIME data-plane, écrites quand le data-plane arrive (Phase F+). Aucune
  extension THREAT_MODEL requise pour le calcul lui-même.
- **[INFO] S3-F2** — Float interne LICITE : matrice RTT vient de
  `conn_rtt -> Option<Duration>` (entiers ns), water-filling/k-medoids en `f64`
  interne sans toucher canonical signé ni `Eq` wire. Si un float fuit dans le
  `ShardPlan`/`Eq` wire → PLAN-ADAPT même-cause ; **sinon EXECUTE** (le cas
  attendu, évitable par construction).
- **[CONCERN] S3-F3** — SYBIL-SEEDER-TAIL concret : `seeders_recent`
  (`seed_registry.rs:331`) trie **lexicographiquement** (`ids.sort()`) ;
  `directory_pull_providers` prend `PULL_PROVIDER_CAP-1=7` premiers après
  l'ancre (`http.rs:1731-1737`). Un Sybil mintant des pubkeys à préfixe hex bas
  occupe déterministiquement les slots. **NB** : la sélection des WORKERS du
  shard passe par l'admission ComputeGroup (S3-F5), PAS par `seeders_recent` ;
  ce dernier est le dial-set du PULL d'archive. → Le sampling Phase D (c)
  s'applique au tail seeder de `directory_pull_providers` (carry littéral), à
  câbler précisément OU doc-noter honnêtement (exit condition binaire,
  `plan:239`/`kickoff:667`).
- **[CONCERN] S3-F4** — Résolution déterminisme-vs-sampling : sampling
  **déterministe-MAIS-NON-LEXICOGRAPHIQUE** seedé par `blake3(session_id)`
  (`shard_plan.rs:246`) ou `group_id` (`:253`). Clé de tri =
  `blake3(seed || seeder_pubkey)` → reproductible (même `session_id` → même
  ordre) ET non-crowdable (préfixe bas n'aide pas). Reste 100% testable
  (fixture `session_id` fixe → ordre attendu), satisfait
  `sybil_seeder_tail_sampling_*` SANS rng non-déterministe. `blake3` déjà dep
  coordinator. Si doc-note plutôt que test (availability-only) → exit condition
  acceptable explicitement permise.
- **[INFO] S3-F5** — Admission NON relâchée : ComputeGroup allowlist Ed25519
  vérifiée SERVER-SIDE au handshake `sbfb/shard/1` (`shard.rs:227` `is_member`
  → rejet non-membre AVANT toute frame). Le placement optimise sur un set
  **fermé** de membres déjà admis ; jamais une porte d'admission. Le sampling
  SYBIL est purement availability (quel seeder dialer pour TÉLÉCHARGER
  l'archive) : intégrité garantie par BLAKE3 (un seeder menteur = 1 dial
  échoué, jamais d'octets faux). Aucune Day-0 (D5, anti-recentralisation)
  touchée.

## S4 — wire format (contribution EXECUTE)

0-bump confirmé et binaire. Phase D consomme les primitives figées Phase C et
produit un `ShardPlan` en mémoire ; **aucun nouveau DOMAIN_\*, aucun
\*_FORMAT_VERSION, aucun type signé**.

- **[INFO] S4-1** — 0 nouveau DOMAIN_\*/FORMAT_VERSION/type signé. Le scheduler
  REMPLIT/VALIDE un plan, pas crée un type (`shard_plan.rs:209` doc Phase D).
  0 édition de `shard_plan.rs`/`canonical.rs` côté wire.
- **[INFO] S4-2** — Types d'entrée INTERNES non sérialisés : RTT via `conn_rtt`
  (`Duration` in-memory), VRAM via `vram_free_bytes:u64`. `nexus-worker-core`
  non-dep → structs scheduler privées au crate, ne dérivent PAS
  `Serialize`-pour-le-wire. C'est l'angle où le float META s'applique : `f64`
  RTT/water-filling EN MÉMOIRE licite, à distinguer du float-dans-canonical
  interdit.
- **[INFO] S4-3** — Le `ShardPlan` sera wrappé plus tard dans
  `ShardedSessionManifest` signé (`DOMAIN_SHARD_PLAN_V1`) — déjà couvert
  additivement Phase C (`shard_plan.rs:234-372`). Phase D ne signe rien.
- **[INFO] S4-4** — Carry **T-NN+3** (factorisation JCS sign/verify) reste
  P2 NON absorbé en D (le scheduler ne signe rien, ne touche pas `canonical.rs`).
  Le commit body Phase D ne réclame PAS T-NN+3 ; il porte SYBIL-SEEDER-TAIL.
- **[INFO] S4-5** — Vigilance scope-creep : la perf-map `(rho, tau)` republiée
  iroh-docs raw-op appartient à **Phase E** (`plan §8.2`), PAS D. Garder D pur
  calcul (pas d'écriture iroh-docs, pas de raw-op, pas de wrap manifest).

---

## Spec concrète Phase D

### Module
**`crates/nexus-coordinator-rs/src/placement.rs`** (nouveau module ; déclarer
`pub mod placement;` dans `lib.rs`). Calcul interne côté initiateur. Importe
`nexus_core_rs::shard_plan::{ShardPlan, ShardAssignment, ShardRole,
KvCachePolicy}` (dep path déjà présente, `Cargo.toml:12`). **0 import
`nexus-worker-core`.** 0 nouveau dep.

### Types d'entrée (INTERNES, non-wire, non-signés)
```text
WorkerPlacementProfile {
    worker_pubkey: [u8; 32],
    vram_free_bytes: u64,        // mesure = GpuStats.vram_free_bytes côté worker
    shard_hashes: Vec<[u8; 32]>, // pin BLAKE3 des poids du bloc
    launch_profile_hash: [u8; 32],
}
RttMatrix                       // RTT pairwise ; entrées = Duration (Phase B conn_rtt)
                                // représentation interne entière (ms ou ns u64) ;
                                // Option pour "pas encore d'échantillon" (None)
ModelSpec { total_layers: u32, quantized_vram_bytes: u64 }
PlacementOutcome (enum) {
    Sharded(ShardPlan),         // VRAM_modèle > VRAM_max_worker
    EndpointFederation,         // VRAM_modèle <= VRAM_max_worker (pas de shard)
}
```
- Ne dérivent **PAS** `Serialize`-pour-le-wire. RTT converti `Duration -> u64`
  en entrée ; la sortie n'expose que des entiers.
- **Ne PAS nommer `WorkerCapability`** (suggère un payload wire — S2-F6).

### Algo (a) water-filling DP + contrainte VRAM + seuil sharding
1. **Seuil sharding (test #2)** : si `model.quantized_vram_bytes <=
   max(workers.vram_free_bytes)` → retourner `PlacementOutcome::EndpointFederation`
   (PAS un `ShardPlan` dégénéré). Comparaison sur `vram_free_bytes` **mesuré**
   (S2-F8, S1A-5).
2. **Water-filling** : répartir `total_layers` entre workers
   **proportionnellement** à `vram_free_bytes` (les blocs sont des plages de
   couches contiguës) sous contrainte que le bloc d'un worker tienne dans sa
   VRAM libre. Calcul interne `f64` LICITE (S1A-1/S2-F3/S4-2) ; les bornes de
   couches converties en `u32`. Contrainte de lien intégrée via le coût RTT (b).
3. **Sortie** : assemblage en `Vec<ShardAssignment>` ordonné pipeline, puis
   `ShardPlan::new(...)`.

### Algo (b) k-medoids RTT (déterministe)
- **Init PAM BUILD** (déterministe, S1A-3) : medoïde 1 = peer minimisant la
  somme des RTT aux autres ; suivants = celui réduisant le plus le coût total.
- **Swap-phase** jusqu'à convergence (échange medoïde↔non-medoïde si réduit le
  coût). **AUCUN rand.**
- Groupe les couches **consécutives** entre peers à **faible RTT mutuel**
  (matrice mesurée `conn_rtt`, PAS géoIP — D3). Tie-break déterministe par
  `worker_pubkey` pour stabilité de `ShardPlan` (`Eq` exact en test).
- Traiter `RttMatrix` `None` : exclure le peer du dial-set ou le pénaliser
  (S1A-2).

### Sortie ShardPlan — couverture [0..L) (BLOCKER S2-F5)
Après assemblage, valider **les deux** :
- `plan.is_pipeline_contiguous()` (gap-free / non-overlap, `shard_plan.rs:211`).
- **Couverture totale** (check délégué Phase D) :
  `plan.assignments.first().layer_start == 0
   && plan.assignments.last().layer_end == model.total_layers`.
Chaque `ShardAssignment` : `role = ShardRole::LayerWorker`,
`kv_cache_policy = KvCachePolicy::LocalEphemeral`, `shard_hashes`/
`launch_profile_hash` repris du `WorkerPlacementProfile`, `fallback_node`
optionnel (Phase E re-balancing — laisser `None` en D).

### Absorption (c) SYBIL-SEEDER-TAIL
Sampling **déterministe-non-lexicographique** (S3-F4) du tail seeder dans la
sélection des candidats du dial-set : clé de tri = `blake3(session_id ||
seeder_pubkey)`, l'ancre reste slot 0 non-crowdable, tri du tail par cette clé.
Reproductible (même `session_id` → même ordre) ET non-crowdable. `blake3` déjà
dep. **Exit condition binaire** : soit le test ci-dessous, soit une **doc-note
honnête** si Phase D ne touche pas le dial-set seeder (availability-only,
permis `plan:239`/`kickoff:667`). **Ne PAS** `thread_rng()` (S1b-2/S1b-5).

### Tests (plan §7.3 ; acceptance `-p nexus-core-rs -p nexus-coordinator-rs`)
1. `placement_water_fills_vram_free` — 3-5 workers de VRAM libre distincte →
   blocs proportionnels.
2. `placement_refuses_when_model_fits_single_worker` — `VRAM_modèle <= VRAM_max`
   → `PlacementOutcome::EndpointFederation` (pas de shard).
3. `kmedoids_groups_low_rtt_consecutive_layers` — groupement déterministe des
   couches consécutives à faible RTT pairwise (fixture matrice RTT).
4. `placement_handles_5_workers_70b` — placement valide d'un 70B Q4 sur 5
   shards ; **asserte `is_pipeline_contiguous()` ET couverture `[0..L)`**.
5. `sybil_seeder_tail_sampling_*` — sampling déterministe (fixture `session_id`
   fixe → ordre attendu) OU doc-note honnête.

### Named constants (S2-F7, README §6.9)
- `SHARD_THRESHOLD` (sémantique `VRAM_modèle > VRAM_max_worker`).
- `KMEDOIDS_K` (ou dérivation du nombre de clusters) + bornes min/max de shards.
- Tout ratio/seuil water-filling réutilisé. **Pas de magic number nu.**

### Acceptance / vérification
- `cargo nextest run -p nexus-core-rs -p nexus-coordinator-rs --locked` vert.
- `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --locked
  -- -D warnings` + `cargo test --workspace --locked --doc`.
- **Dual-platform** : Windows PowerShell + Docker `sbfb-ci` (rust:1.94) AVANT
  push (mémoire `feedback_dual_platform` / `feedback_wsl_before_push`).
- T1 E2E : `N-A-no-frontend-change` (pas de surface front en D).

### Wire / commit body
- **Pre-launch protocol : 0 bump wire** (scheduler = calcul interne, S4).
- Commit cible : `feat(core): Sprint 77 Phase D — Parallax placement scheduler
  (water-filling + k-medoids)`. Carry closure : SYBIL-SEEDER-TAIL (absorbé).
- **NE PAS** réclamer T-NN+3 (reste P2, S4-4).

---

## Risques résiduels

1. **Doc périmée plan §7.2** (`sprint77_plan.md:223,226`) : si l'implémenteur
   suit littéralement `GpuInfo.vram_free_bytes`/`conn.stats()`, le code ne
   compilera pas (`GpuInfo` n'a pas `vram_free_bytes` ; `conn.stats()` n'a pas
   `rtt`). **Mitigation** : la spec ci-dessus pointe `GpuStats.vram_free_bytes`
   + `conn_rtt()` ; corriger en passant le commentaire de code/doc-note.
2. **Fuite de float dans la sortie** : un `f64` qui atteindrait `ShardPlan`/un
   `Eq` wire transformerait PLAN-ADAPT en cause-A/B/C. **Mitigation** :
   conversion explicite `f64 -> u32` aux bornes de couches ; revue G-REVIEW
   vérifie que `ShardAssignment` reste all-integer.
3. **Non-déterminisme du sampling/k-medoids** : un `thread_rng()` (copié des
   modules voisins) casserait la reproductibilité des tests ET toucherait la
   voie advisory exemptée. **Mitigation** : init PAM BUILD sans rand + sampling
   `blake3(session_id)`-seedé (S1b-2/S3-F4).
4. **Débordement scope Phase E** : toute persistance iroh-docs / raw-op /
   perf-map `(rho,tau)` / wrap manifest serait un empiètement E (`plan §8.2`).
   **Mitigation** : D reste pur calcul en mémoire sur fixtures.
5. **Scope cut #7 (BLOCKER S2-F4)** : toute modification de `consent.rs` ou
   câblage VRAM-live runtime = scope cut cassé → review BLOCK. **Mitigation** :
   D ne lit `vram_free_bytes` qu'au placement, sur profils d'entrée fournis par
   l'appelant.

**META** : 3 PLAN-ADAPT consécutifs A/B/C avaient la même cause (float-R&D vs
no-float-canonical). **Phase D rompt ce motif** : son PLAN-ADAPT a une cause
DIFFÉRENTE (wiring API RTT/VRAM périmé + prescription de déterminisme du
sampling). Le float interne au scheduler est tranché LICITE par quatre scans
concordants. Pas de signal-meta "4e même cause" à noter.
