# Sprint 77 — Preflight Phase E (G8)

## Verdict: PLAN-ADAPT

> Codable, 0 Day-0 touchée, 0 escalade PO. Le coeur algorithmique du plan §8.2
> (routing DAG sweep DP G→D + churn `replace_failed_server` O(t) + heap fallback
> + cache activations) est implémentable **pur en mémoire**, exactement comme le
> précédent Phase D (`placement.rs`). PLAN-ADAPT car **trois points du libellé
> du plan §8.2 doivent être corrigés** : (a) la **perf-map est un raw-op NON
> signé** (`serde_json::Value`), JAMAIS un nouveau `DOMAIN_*` (le budget §19 est
> clos à 4) ; (b) `rho`/`tau` sont des **micros entiers** (miroir `RttMatrix`/
> `RunMetrics`, no-float) pour `Eq`/déterminisme/tests ; (c) le test
> `perf_map_republished_to_doc` tourne **in-crate en round-trip
> serialize/deserialize** car `nexus-coordinator-rs` **n'a aucun accès iroh-docs
> write** — le `doc.set()` réel est de la glue daemon. Aucune de ces corrections
> ne contredit une décision figée.

---

## Résumé exécutif

Phase E = **Phase 2 du scheduler Parallax (ROUTING)** + **churn actif Petals**,
construite sur la sortie Phase D. Le routing DAG est un **DP de plus court chemin
sur un DAG indexé par couche** : un seul balayage gauche→droite (G→D)
`dp2(l+1,g')=min(..., dp2(l,g)+rho<g,g'>+tau<g',l+1>)`, O(L·R²) à 3-5 peers,
qui sélectionne la chaîne pipeline de latence minimale. Le churn = modèle
**Petals ACTIF** (`replace_failed_server` O(t) + heap fallback ordonné par
latence + cache d'activations client-side), **PAS** le modèle Parallax
« clé DHT expire » (rejeté D3 comme faille churn). La perf-map `(rho,tau)`
republiée 1-2s à iroh-docs est la **seule nouvelle surface live** — un raw-op
**non signé** (LWW), `0` bump wire, `0` nouveau `DOMAIN_*`.

Les 5 scans convergent :

| Scan | Axe | Contribution |
|---|---|---|
| **S1a** | SOTA / OSS | PLAN-ADAPT (Parallax Phase-2 routing + Petals churn vérifiés verbatim ; 3 corrections wiring/sémantique) |
| **S1b** | deps / CVE | EXECUTE (0 nouveau dep, hand-roll BinaryHeap+DP ; 1 CONCERN code-shape déterminisme + 1 CONCERN locational) |
| **S2** | décisions figées | PLAN-ADAPT (0 DESIGN-CONFLICT ; 2 BLOCKER = contraintes de forme [crate-boundary + perf-map-reste-raw-op], pas des conflits) |
| **S3** | threat model | PLAN-ADAPT (1re surface live ; posture d'auth perf-map + tau-bias SI-3/SI-4 à prescrire) |
| **S4** | wire format | PLAN-ADAPT (0-bump confirmé ; perf-map = struct coordinator + raw-op, doc.set glue différé ; rho/tau micros entiers) |

**Point META tranché (3 PLAN-ADAPT A/B/C même cause no-float, Phase D a rompu le
motif) :** Phase E **n'est PAS la 4e occurrence** du pattern float-R&D-vs-repo,
et n'est pas non plus une reprise du motif. La perf-map est un **raw-op NON
signé** (`FeedEntry.op = serde_json::Value`) — le verrou no-float ne lie QUE les
payloads JCS signés (`shard_plan.rs:46-53`, `placement.rs:28-38`). Un `f64`
serait *techniquement* toléré dans le raw-op non signé, **mais** `rho`/`tau` sont
**imposés micros entiers** (`u64`) pour des raisons de déterminisme/`Eq`/tests —
exactement le choix que Phase D a déjà fait dans le MÊME crate (`RttMatrix`
`set()` → `as_micros()` `placement.rs:104-146,129-131` ; `RunMetrics`
all-integer avec milli-tokens/sec `shard_plan.rs:373-399`). La cause du
PLAN-ADAPT est **DIFFÉRENTE** d'A/B/C : c'est le **split crate-boundary**
(coordinator sans iroh-docs) + la **prescription perf-map-reste-raw-op** + la
**posture d'auth/anti-bias** — pas la divergence float-canonical. Le motif
« 3x même cause » reste rompu (Phase D), **aucun signal-meta « 4e même cause »**.

**Pourquoi PLAN-ADAPT et non EXECUTE pur :**
1. La perf-map iroh-docs **n'a aucun home dans le coordinator** :
   `nexus-coordinator-rs/Cargo.toml:11-29` n'a **aucune dep iroh/iroh-docs**
   (seulement `nexus-core-rs` path + rusqlite/serde/blake3/rand) ; `lib.rs:11-38`
   n'a **aucun module node/docs** (DB+dispatcher+validator+placement). Tous les
   `doc.set()`/`set_bytes()` vivent dans `nexus-shell-daemon`
   (`dispatch_loop.rs:49,457,497` ; `feed_sync.rs:63`) ; le wrapper
   `DocHandle::set` est `nexus-core-rs/docs.rs:248`. **Le livrable « republish
   perf-map » DOIT migrer vers le daemon.**
2. Le plan documente `rho` comme « RTT one-way/paire » (`addendum §2:72`,
   `plan §8.2`) mais la primitive Phase B `conn_rtt` (`shard.rs:160`) rend le
   **round-trip** (`Connection::rtt(PathId::ZERO)`, `shard.rs:152-159`). Même
   classe que le fix `conn.stats()`→`conn_rtt` de Phase D : correction de
   doc/unité, pas de conflit.
3. La perf-map raw-op et le cache d'activations sont du calcul/état pur en
   mémoire comme `placement.rs` (dont `plan_placement` n'est appelé que par ses
   tests aujourd'hui, `placement.rs:648+`) ; `f64` en mémoire serait licite mais
   les coûts micros entiers sont préférés pour `Eq`/déterminisme — la cause est
   DIFFÉRENTE d'A/B/C.

**Pourquoi PAS DESIGN-CONFLICT :** aucune Day-0 n'est contredite. D3
(pipeline-parallel + k-medoids RTT empirique, géoIP REJETÉ, **churn Petals actif
PAS Parallax DHT-expiry**), D5 (admission ComputeGroup privée intacte), scope cut
#4 (activations en clair, inchangé), scope cut #5 (KV-cache distribué post-S77 ;
cache churn BORNÉ in-scope), scope cut #7 (0 pompe VRAM runtime, 0 `consent.rs`),
§19 (budget 4 `DOMAIN_*` clos), 0-bump wire pre-launch, iroh 0.98 pinné — **tous
respectables par le plan tel qu'écrit, sous réserve des 3 corrections de
libellé**.

---

## S1a — SOTA / OSS (contribution PLAN-ADAPT)

L'algorithme est **SOTA-fidèle et codable** avec le précédent Phase D (calcul
mémoire déterministe, sortie entière, 0 bump wire). Parallax (arXiv 2509.26182)
Phase-2 routing est vérifié verbatim : *« Phase 2 decides request-time GPU
pipeline chain selection via a DAG dynamic program over a DHT of live per-layer
latencies and inter-GPU RTTs »* = exactement la récurrence
`dp2(l+1,g')=min(..., dp2(l,g)+rho<g,g'>+tau<g',l+1>)` (`plan §8.2`,
`addendum §2:71-73`). Petals (arXiv 2209.01188/2312.08361) churn vérifié
verbatim : *« client-side cache holds past inputs ... if a server disconnects, a
client finds another server with that stage and uses client-side cache to
restore state »* + rebalancing périodique = exactement
`replace_failed_server` O(t) + heap fallback + cache activations client-side.

- **[INFO] S1A-1** — Récurrence Parallax Phase-2 SOTA-fidèle et codable 1:1 en DP
  pur. Un seul balayage G→D EST une relaxation valide : l'index de couche ordonne
  topologiquement le DAG, donc une passe forward (avec R groupes candidats par
  couche) calcule la chaîne min-coût — plus court chemin sur DAG standard, sans
  itération de point fixe. O(L·R²) négligeable à R=3-5. Le coder en fonction
  pure miroir de `placement.rs:184 plan_placement` (appelé seulement par ses
  tests, `placement.rs:648+`), dans `placement.rs` ou un module frère `routing.rs`
  de `nexus-coordinator-rs`.
- **[INFO] S1A-2** — Churn actif Petals SOTA-fidèle et conforme au plan. Le
  plan/kickoff D3 (`kickoff:404-405`) **REJETTE** correctement le « DHT key
  expires » de Parallax (« ne re-route jamais mid-inference → faille churn ») et
  prend le modèle ACTIF de Petals — le choix fidèle. `replace_failed_server` O(t)
  = prendre le serveur suivant détenant le stage `t` dans le heap ordonné par
  latence ; remplacer **un** assignment, pas re-planifier tout le pipeline.
- **[CONCERN] S1A-3** — Perf-map iroh-docs **n'a AUCUN home dans le coordinator**
  — gap de wiring STRUCTUREL, pas un conflit de design. `plan §8.2` livrable 3
  (« perf-map (rho,tau) republiée 1-2s iroh-docs raw-op ») ne peut pas vivre où
  Phase D place le scheduler : `lib.rs` n'a aucun module node/docs/gossip (DB+
  dispatcher+validator) et `Cargo.toml:12-23` dépend de `nexus-core-rs`+blake3+
  rand mais PAS iroh-docs. La primitive `doc.set(author,key,value)` vit dans
  `nexus-core-rs/src/docs.rs:248`, pilotée par le daemon. **Split : COMPUTE
  perf-map déterministe (ingest rho/tau → DP) reste fonction pure coordinator ;
  PUBLISH iroh-docs (1-2s) est daemon-side.** Conforme `addendum §2:77-78`.
- **[CONCERN] S1A-4** — `rho` spécifié « one-way RTT » mais la primitive Phase B
  rend le round-trip — fidélité de nommage/sémantique, même classe que le fix
  Phase D. `addendum §2:72` + `plan §8.2:265` écrivent « rho = RTT one-way/paire »
  mais `conn_rtt` (`shard.rs:160`) = `Connection::rtt(PathId::ZERO)` = le
  round-trip du path (`shard.rs:152-159` doc). Le `RttMatrix` Phase D stocke déjà
  ce round-trip en micros entiers (`placement.rs:104-146`). Résoudre en
  documentant `rho` comme round-trip (le DP est self-consistant : les deux termes
  `rho` utilisent la même métrique, seul l'ordre relatif des chaînes compte) OU
  en halvant pour approximer le one-way. Doc/naming, pas un changement d'algo.
  **Réutiliser le `RttMatrix` micros-entier verbatim — ne PAS ré-introduire un
  type RTT flottant.**
- **[INFO] S1A-5** — META tranché : la sérialisation perf-map ne ré-ouvre PAS le
  conflit float-vs-canonical A/B/C, et Phase E n'est PAS la 4e occurrence. (a) Le
  DP routing et le coeur churn sont du calcul PUR en mémoire comme `placement.rs`
  (doc module `placement.rs:28-38` : la règle no-float ne lie QUE les payloads
  JCS signés ; la sortie est 100% entière). `rho`/`tau` en micros entiers
  (miroir `RttMatrix` `placement.rs:104` et `RunMetrics` all-integer
  `shard_plan.rs:378-399` dont `decode_milli_tokens_per_sec`) gardent `Eq` exact
  et la chaîne reproductible en test. (b) La perf-map iroh-docs est un raw-op
  `serde_json::Value` NON signé (`FeedEntry.op`), PAS un payload canonical signé
  — §19 ne budgète AUCUN `DOMAIN_*` perf-map. Float toléré dans le JSON mais
  micros entiers fortement préférés pour déterminisme/`Eq`/fixtures. Comme Phase
  D, la cause PLAN-ADAPT est DIFFÉRENTE d'A/B/C — streak « 3x même cause » reste
  rompu, **pas de « 4e même cause »**.
- **[INFO] S1A-6** — `fallback_node` prêt et explicitement différé Phase E par
  Phase D. `ShardAssignment.fallback_node:Option<[u8;32]>`
  (`shard_plan.rs:169-174`, `#[serde(default)]`) est documenté « assigned by
  Phase E churn handling, not at placement time » et `placement.rs:269-271` le
  met `None` à chaque assignment. Phase E le peuple depuis le heap fallback
  ordonné par latence. **Pas de changement wire** : le champ existe et round-trip
  (`shard_assignment_serde_roundtrip shard_plan.rs:657`, omis⇒`None` `:666-673`).
  `churn_replaces_failed_server_oturn` asserte qu'un drop re-route via le heap et
  l'inférence continue depuis le cache d'activations. 0 bump wire, additif seul.
- **[INFO] S1A-7** — Aucun crate prior-art requis — hand-roll continue le pattern
  Phase D. Le DP routing est une boucle plate sur couches×groupes sur matrices de
  coût `Vec`/slice (pas de graph crate ; petgraph/pathfinding absents et inutiles
  à O(L·R²), R≤5). Le churn est un `BinaryHeap` (std) de serveurs fallback clé
  micros-entiers, tie-break déterministe par `worker_pubkey` (miroir PAM
  `placement.rs:499/514/550/563`). `rand` est dans `Cargo.toml:23` mais le chemin
  routing/churn n'a besoin d'AUCUN aléa (DP + heap déterministes) — **ne PAS
  introduire `thread_rng`** (caution Phase D S1b-2). Grep confirme aucun code
  `routing_dag/replace_failed/perf_map/dp2` existant — Phase E est net-new.

## S1b — deps / CVE / déterminisme (contribution EXECUTE)

S1b = EXECUTE. 0 nouveau crate, 0 churn advisory, et le code-shape déterminisme
est déjà établi par le précédent Phase D dans le fichier même que Phase E étend.

- **[INFO] S1b-1** — Zéro nouveau crate. Routing DAG sweep DP = boucles `Vec`/2D
  (O(L·R²), 3-5 peers) ; churn fallback = `std::collections::BinaryHeap`. Aucun
  crate graph/matrix/clustering requis ni atteignable : grep `Cargo.lock`
  kmeans/kmedoids/linfa/ndarray/nalgebra/smartcore/petgraph = 0 ;
  `priority-queue 2.7.0` existe (`Cargo.lock:6527`) seulement en dep transitive
  iroh-side, PAS importé par le coordinator. serde/serde_json/blake3/rand déjà
  présents (`Cargo.toml:16-23`). 0 churn `Cargo.toml`/`Cargo.lock`.
- **[INFO] S1b-2** — Zéro churn advisory. Le seul ignore `deny.toml:55-65` =
  `RUSTSEC-2026-0097`, scopé `ThreadRng` reseed sous `log::Log` custom — non
  utilisé par SBFB, à ne PAS introduire. Tant que Phase E reste OsRng/thread_rng-
  libre (impératif pour le déterminisme), `cargo deny check` reste vert sans
  nouvel ignore. `rand` reste workspace 0.8.
- **[CONCERN] S1b-3** — Code-shape déterminisme (le vrai watch-item S1b). Le heap
  fallback churn et le DP doivent être byte-reproductibles pour des
  `ShardPlan`/résultats routing `Eq`-stables en test. Phase D a établi le pattern
  canonique dans le MÊME fichier : tie-break par `worker_pubkey`
  (`cluster_order_by_rtt placement.rs:497-514`, `pam_build:550`, `pam_swap:592`),
  sampling `blake3(session_id||pubkey)` (`sampling_key:313-318`), `saturating_add`
  contre RTT adversarial (`total_cost:531`, `pam_build:548-549`). Phase E DOIT
  miroir : `BinaryHeap` ordonné par `(latency_micros, pubkey)` ; relaxation DP en
  ordre fixe ; **AUCUN `thread_rng()`/`rand::random()`** copié des modules voisins
  non-déterministes (`dispatcher.rs:81`, `canary_input.rs:230`,
  `upload_queue.rs:107`). Prescription de code-shape, pas un blocker — le
  précédent le rend mécanique.
- **[INFO] S1b-4** — La sérialisation perf-map (rho,tau) ne ré-ouvre PAS le
  streak META float A/B/C. Le repo mandate déjà la latence entière par convention :
  `RunMetrics` all-integer encode un ratio fractionnaire en milli-tokens/sec pour
  garder les floats hors des signed bytes (`shard_plan.rs:383-386`), et
  `RttMatrix` est déjà micros entiers (`placement.rs:104-146`,
  reproductibilité-exacte `:28-38`). `rho`/`tau` DOIVENT donc être micros entiers
  (`u64`), que la perf-map soit signée ou un raw-op `serde_json::Value`. Aucun
  nouveau `DOMAIN_*` budgété (§19 = compute_group/shard_plan/run_proof/
  activation_commit seulement) et le raw-op ne bump aucune feed version
  (CLAUDE.md pre-launch + `plan §8.2`). Micros entiers = house style existant,
  pas un PLAN-ADAPT forcé — confirme que le streak-break Phase D tient pour E.
- **[CONCERN] S1b-5** — Wiring locational (hors coeur dep/CVE/déterminisme mais
  déterminisme-adjacent) : le coordinator n'a aucune capacité iroh-docs write —
  `lib.rs:11-39` expose seulement des modules DB-level (db, search, public_feed,
  feed_materializer lisent/matérialisent les feed entries ; pas de module
  node/docs). Donc le republish iroh-docs de la perf-map (`plan §8.2`, 1-2s) ne
  peut PAS être émis depuis `placement.rs` et doit transiter par la couche
  daemon/core node qui détient le doc handle. Le DP routing lui-même reste une
  fonction pure coordinator (miroir `placement.rs`), consommant un snapshot
  perf-map fourni par le daemon. Note de wiring impl, pas une nouvelle dep ni une
  violation de déterminisme.

## S2 — décisions figées (contribution PLAN-ADAPT)

**Aucun DESIGN-CONFLICT** : chaque Day-0 (D3 pipeline-parallel exclusif,
k-medoids RTT EMPIRIQUE seul, géoIP REJETÉ, churn Petals actif PAS Parallax
DHT-expiry) et chaque scope cut touchable (#5 cache churn borné, #7 pas-de-pompe-
VRAM/pas-de-`consent.rs`) est respectable tel quel. La seule contrainte
load-bearing qui rend ce PLAN-ADAPT plutôt qu'EXECUTE est ARCHITECTURALE : la
perf-map « republiée iroh-docs » ne peut PAS être implémentée dans
`nexus-coordinator-rs` (0 dep iroh).

- **[BLOCKER] S2-F1** — CRATE BOUNDARY (contrainte, pas conflit) :
  `nexus-coordinator-rs` n'a AUCUNE dep iroh (`Cargo.toml:11-29` = nexus-core-rs/
  rusqlite/tokio/serde/blake3/rand ; `lib.rs:11-39` aucun module node/docs/iroh).
  Donc le republish perf-map iroh-docs (`plan §8.2` livrable 3 + test
  `perf_map_republished_to_doc` + fail-fast row 28) ne peut PAS vivre à côté de
  `placement.rs`. Phase E split : routing-DAG sweep DP + churn heap/
  `replace_failed_server` restent in-memory dans coordinator (miroir
  `placement.rs`, acceptance `-p nexus-coordinator-rs`), et le publish/read
  perf-map traverse vers le daemon (`nexus-shell-daemon` détient les writes
  feed/doc raw-op). Si l'implémenteur tente un write iroh-docs depuis coordinator,
  **ça ne compile pas**. Honorer en gardant l'algo routing/churn crate-pur et en
  câblant la perf-map au bord daemon.
- **[BLOCKER] S2-F2** — PERF-MAP RESTE RAW-OP NON SIGNÉ — PAS de 5e `DOMAIN_*`
  (budget wire §19). §19 budgète EXACTEMENT 4 `DOMAIN_*` : compute_group/
  shard_plan/run_proof/activation_commit. `canonical.rs:214-286` confirme
  compute_group/shard_plan/run_proof définis ; `DOMAIN_ACTIVATION_COMMIT_V1` pas
  encore présent (Phase I). AUCUN perf-map `DOMAIN_*` budgété. Le design
  (`addendum §2:77-78` + `plan §8.2`) appelle la perf-map un raw-op republié
  1-2s ; CLAUDE.md pre-launch + `kickoff §1.4` disent qu'un raw-op ne bump PAS
  `FEED_FORMAT_VERSION`. Donc perf-map = `serde_json::Value` raw-op non signé,
  0 bump. Une famille perf-map SIGNÉE serait un DESIGN-CONFLICT vs le budget wire
  explicite. Honorer en rendant la perf-map non signée (pas d'Ed25519, pas de
  `canonical_bytes`, pas de `DOMAIN_*`) — intégrité best-effort/advisory comme le
  seed registry, jamais une attestation signée.
- **[CONCERN] S2-F3** — META résolu — la perf-map n'est PAS la 4e occurrence
  float-même-cause. La cause récurrente A/B/C (R&D floats/String-blake3/
  schema_version-à-plat vs repo entiers/[u8;32]/version+Entry) ne récurre pas ici
  car la perf-map est un raw-op non signé, où la règle no-float NE lie PAS (elle
  lie seulement les payloads JCS signés — `shard_plan.rs:46-53`,
  `placement.rs:28-38`). `rho`/`tau` sont des latences : float TECHNIQUEMENT
  permis dans un raw-op non signé, mais micros entiers FORTEMENT préférés pour
  déterminisme/`Eq`/reproductibilité (miroir `RttMatrix` `placement.rs:104-146`
  `set` = `rtt.as_micros()` `u64` ; `RunMetrics` all-integer
  `shard_plan.rs:378-399`). La cause PLAN-ADAPT Phase E = split crate-boundary
  (S2-F1) + perf-map-reste-raw-op (S2-F2), DIFFÉRENTE d'A/B/C — D a rompu le 3x
  streak, E le garde rompu.
- **[INFO] S2-F4** — D3 churn/routing RESPECTÉ. Churn actif = Petals
  `replace_failed_server` O(t) + heap fallback ordonné latence + cache
  activations client-side (`plan §8.2`, `addendum §2:74-78`), explicitement PAS
  Parallax « DHT key expires » (`addendum §2` le qualifie de faille churn,
  `plan §8.2` « PAS le clé DHT expire »). Routing DAG = indexé couche + un seul
  sweep G→D `dp2(l+1,g')=min(...,dp2(l,g)+rho<g,g'>+tau<g',l+1>)` O(L·R²)
  (`addendum §2:70-73`). k-medoids déjà EMPIRICAL-RTT-only, géoIP REJETÉ
  (`placement.rs:40-45` « no geo-IP table, ASN lookup or central region
  authority » ; D3 `kickoff:399-403`). Routing réutilise le même `RttMatrix`
  mesuré ; aucune nouvelle autorité centrale. Aucune Day-0 contredite.
- **[CONCERN] S2-F5** — SCOPE CUT #5 — le « cache d'activations client-side »
  DOIT être le cache churn BORNÉ, PAS le KV-cache distribué non-borné post-S77.
  §17 #5 : « KV-cache distribué / activation cache O(t) gros contexte → post-S77
  ... KV-cache local + cache churn BORNÉ ». `KvCachePolicy` est FIGÉ à
  `LocalEphemeral` (`shard_plan.rs:128-133`, doc : « distributed KV cache is
  post-S77 (scope cut #5) »). Le cache Phase E = PETIT buffer borné d'activations
  de frontière récentes pour rejouer un stage sur drop worker (Petals), PAS un KV
  cache full-context O(t) ni distribué. RISQUE : la prose « cache activations
  client-side » pourrait être sur-lue comme l'item post-S77. Honorer en bornant
  le cache avec une CONSTANTE NOMMÉE (ex `ACTIVATION_REPLAY_CACHE_MAX`,
  README §6.9), en le gardant local (pas de distribution iroh-docs des
  activations), et en doc-notant que le KV cache distribué/O(t) reste scope cut #5.
- **[INFO] S2-F6** — SCOPE CUT #7 RESPECTÉ — `fallback_node` est runtime-only,
  0-wire, ne touche pas `consent.rs`/pompe-VRAM. `placement.rs:269-271` documente
  déjà `fallback_node` « assigned by Phase E churn handling, not at placement
  time » ; le champ est `#[serde(default)] Option<[u8;32]>`
  (`shard_plan.rs:169-174`) — vraiment optionnel, pas identité. Le re-routing
  churn est une décision RUNTIME sur un `ShardedSessionManifest` déjà signé ; ne
  requiert PAS de re-sign. Si l'initiateur re-PLANIFIE toute la session, le
  `revision:u64` + `ShardedSessionManifestEntry::sign` existants
  (`shard_plan.rs:256-259,330-344`) couvrent additivement — aucune branche
  n'ajoute de wire. Scope cut #7 (pas de pompe VRAM runtime, pas d'édition
  `consent.rs`, placement read-only) intact : Phase E lit RTT (`conn_rtt`
  `shard.rs:160`) pas VRAM, et n'entre pas dans `consent.rs`.
- **[INFO] S2-F7** — `rho` = ONE-WAY RTT vs `conn_rtt` round-trip — nuance
  naming/unité, pas un conflit. `addendum §2:72-73` + `plan §8.2` définissent
  `rho` « RTT one-way/paire » mais la source `conn_rtt` (`shard.rs:160`) rend le
  round-trip `Duration` (`Connection::rtt(PathId::ZERO)`). Phase E soit halve vers
  one-way, soit documente que `rho` porte les micros round-trip de façon cohérente
  (le DP n'a besoin que d'un coût-lien monotone cohérent). Honorer par une
  doc-note one-line sur l'unité `rho` ; **ne PAS inventer un nouveau chemin de
  mesure** (pas de `conn.stats().rtt` — inexistant sur le fork noq,
  `shard.rs:158`). Micros entiers partout (miroir `RttMatrix`).
- **[INFO] S2-F8** — CONSTANTES NOMMÉES (README §6.9) pour les magic numbers
  Phase E, comme le précédent D. `placement.rs` nomme déjà `KMEDOIDS_DEFAULT_K`,
  `KMEDOIDS_MAX_ITER`, `MISSING_RTT_PENALTY_MICROS`, `MIN_SHARD_WORKERS`. Phase E
  doit nommer : l'intervalle de republish perf-map (1-2s →
  `PERF_MAP_REPUBLISH_INTERVAL`), la borne du heap fallback, le cap du cache
  d'activations, tout cap du DP. Pas de magic number nu, vérifié G-REVIEW.

## S3 — threat model (contribution PLAN-ADAPT)

Phase E est le PREMIER item Phase-E à toucher la **delivery LIVE** : la perf-map
`(rho,tau)` republiée iroh-docs 1-2s (`plan §8.2`, design figé l.77-78/108-109)
est une nouvelle surface lisible/inscriptible, contrairement à Phase D
(placement = calcul pur en mémoire, 0 signature, 0 wire, `placement.rs:1-45`). Le
deferral Phase D (S3-F1) des SI-1/SI-3/SI-4/SI-5 vers le data-plane (F+) et
THREAT_MODEL §16 à Phase K (`plan §14.2`, fail-fast row 41) reste VALIDE pour les
SI-* data-plane, MAIS la perf-map raw-op force UNE adaptation threat-model
MAINTENANT : un membre admis mais malveillant peut publier une perf-map forgée
pour biaiser le routing vers lui (SI-3), et le churn fallback peut re-router vers
un complice (SI-4). Contribution = PLAN-ADAPT (pas DESIGN-CONFLICT : aucune Day-0
contredite, D5 gate la surface ; pas EXECUTE-pur : la posture d'auth + la
mitigation tau-bias doivent être prescrites avant code). Cause DIFFÉRENTE d'A/B/C
(no-float) ET de Phase D (stale-API) → **pas de « 4e même cause »**.

- **[CONCERN] S3-E1** — POSTURE D'AUTH PERF-MAP — la vraie adaptation. Le design
  figé (`addendum` l.77-78,108-109) dit que la perf-map `(rho,tau)` est republiée
  iroh-docs en raw-op (LWW). Le précédent SBFB pour TOUT raw-op iroh-docs republié
  = le payload porte sa PROPRE sig Ed25519 + `author_pubkey` sur `canonical_bytes`
  avec un domain tag dédié, et l'ingest enforce `verify_entry` + PoW +
  `author==node` (`seed.rs:30-36` SeedAnnounced ; `node_directory.rs:32-53` ;
  THREAT_MODEL §15 « SeedAnnounced forge » mitigé par `verify_entry` +
  `FEED_POW_DIFFICULTY=16`). CEPENDANT la perf-map vit DANS une ComputeGroup
  privée déjà admise (D5 allowlist Ed25519, scope cut #8 « zero worker anonyme »),
  donc le forgeur est un MEMBRE, pas un anonyme — PoW/anti-Sybil n'est pas l'outil.
  **Prescription** : la perf-map raw-op DOIT être attribuée à la clé Ed25519 de
  son publieur et l'ingest DOIT rejeter les entries dont l'author n'est pas dans
  l'allowlist ComputeGroup de la session (réutiliser `compute_group.is_member`,
  miroir `shard.rs:222-231` admission handshake). Authentification de QUI a
  publié, pas un nouveau `DOMAIN_*` ; ride l'enveloppe signée `FeedEntry`
  existante, reste additif et 0-bump (CLAUDE.md raw-op policy).
- **[CONCERN] S3-E2** — TENSION BUDGET `DOMAIN_*` (lie la question META float,
  résout CONTRE un nouveau `DOMAIN_*`). §19 budgète SEULEMENT 4 net-new
  `DOMAIN_*` : compute_group, shard_plan, run_proof, activation_commit
  (`plan §19`, fail-fast row 38). Aucun perf-map `DOMAIN_*`. Si l'implémenteur
  faisait de la perf-map un payload canonical signé avec `DOMAIN_PERF_MAP_V1`,
  ça (a) ferait sauter le budget §19 = DESIGN-CONFLICT, ET (b) ré-ouvrirait la
  tension float-vs-canonical (un pré-image canonical signé interdit `f64`, donc
  `tau` devrait être micros entiers comme `RttMatrix`/`RunMetrics`). RÉSOLUTION
  évitant les deux : NE PAS minter de perf-map `DOMAIN_*`. Porter la perf-map en
  `FeedEntry` raw-op (`serde_json::Value` op body) authentifié par l'enveloppe
  signature/PoW `FeedEntry` EXISTANTE (S3-E1) — le body per-op n'a pas besoin de
  domain tag canonical séparé, donc la règle no-float ne lie pas le body et
  `rho`/`tau` PEUVENT être serde-encodés ; néanmoins les encoder en micros entiers
  (miroir `RunMetrics`/`RttMatrix`) pour que le DP routing soit déterministe,
  `Eq`-comparable, reproductible en test. Maintient Phase E dans le budget 4
  `DOMAIN_*` → EXECUTE sur l'axe wire, PLAN-ADAPT seulement sur l'auth.
- **[CONCERN] S3-E3** — TAU SELF-REPORT = VECTEUR DE BIAIS ROUTING (SI-3). Dans la
  perf-map, `rho` (one-way RTT/paire) est MESURABLE par le nœud routing depuis ses
  paths QUIC (`conn_rtt`, `shard.rs` exposé Phase B — même posture que
  `placement.rs` qui utilise du RTT MESURÉ, jamais self-report). Mais `tau`
  (latence per-layer/GPU profilée) est INTRINSÈQUEMENT self-reported par le
  worker. Un membre qui sous-déclare son `tau` biaise le DP min-latence pour
  router la chaîne par lui — attaque SI-3 self-favouring que le placement n'a
  jamais affrontée. **MITIGATION à designer maintenant** : (a) préférer le `rho`
  mesuré au `tau` self-reported partout où le nœud routing peut observer ; (b)
  traiter `tau` comme ADVISORY, jamais une frontière de confiance — exactement le
  précédent S76 « cohorte homogène routage ADVISORY » (THREAT_MODEL §15.2 :
  « routage ADVISORY ... n'est JAMAIS une frontière de confiance ; la vraie
  défense reste le quorum/fingerprint »). Ici la vraie défense d'intégrité est la
  vérification downstream N0-N3 RunProof (TOPLOC fingerprint, `plan §10-12`) : un
  worker qui gagne le routing en mentant sur `tau` ne peut toujours pas produire
  un fingerprint valide pour des couches qu'il calcule mal. Documenter `tau`
  comme un hint PERF self-reported qui biaise SEULEMENT l'optimisation latence,
  avec la vérification (pas le routing) comme autorité d'intégrité.
- **[CONCERN] S3-E4** — COLLUSION CHURN FALLBACK (SI-4) + DÉTERMINISME. Le heap de
  serveurs fallback + `replace_failed_server` O(t) (`plan §8.2` ;
  `fallback_node` documenté `placement.rs:269-271`) re-route sur drop worker.
  Menace : si la sélection fallback est influencée par `tau`/availability
  self-reported, une paire en collusion pourrait provoquer un drop pour forcer le
  re-route sur le complice (SI-4 — confidentialité tient seulement si ≥1 worker
  honnête, `SPLIT_INFERENCE_DESIGN.md:201`). **Mitigation** : le heap fallback
  DOIT tirer SEULEMENT de la même allowlist ComputeGroup (aucune relaxation
  d'admission au churn — miroir `shard.rs:227 is_member` ; le remplaçant d'un
  membre droppé est toujours un membre Ed25519-admis, jamais un pair opportuniste
  anonyme, scope cut #8). ET l'ordre fallback doit être déterministe et non
  gameable par self-report : ordonner par RTT MESURÉ (`rho`) avec tie-break
  déterministe sur `worker_pubkey` (miroir PAM `placement.rs:40-45`), PAS par
  `tau` self-reported. Garde `churn_replaces_failed_server_oturn` reproductible et
  ferme le re-route self-favouring.
- **[CONCERN] S3-E5** — CACHE D'ACTIVATIONS = MEMORY-DoS, DOIT ÊTRE UNE CONST
  NOMMÉE BORNÉE (lie scope cut #5). Phase E ajoute un cache d'activations
  client-side pour continuer l'inférence à travers un churn replace (`plan §8.2`,
  `kickoff:361,379`). Scope cut #5 dit que le cache in-scope est le cache churn
  BORNÉ (« KV-cache local + cache churn borné »). Un cache non-borné est un
  memory-DoS (un pair qui déclenche des drops répétés force des activations
  retenues non-bornées). **Prescription** : cache churn = ring/LRU borné avec une
  CONSTANTE NOMMÉE de capacité (README §6.9, miroir `NODE_DIRECTORY_MAX_ENTRIES`/
  `MAX_FETCH_PROVIDERS=16` `node_directory.rs` + posture eviction §15.1).
  Eviction = oldest-frontier-first. Garder O(borné) honore aussi scope cut #5
  (le cache gros-contexte NON-borné reste différé) — donc
  SCOPE-CUT-CONSISTENT, pas un creep.
- **[INFO] S3-E6** — TIMING EXTENSION THREAT_MODEL — le deferral Phase D TIENT
  pour les SI-* data-plane, mais une note perf-map est due au commit Phase E. Le
  preflight Phase D S3-F1 a différé SI-1/SI-3/SI-4/SI-5 à « Phase F+ », et le §16
  canonical (SI-* + private-group + confidentiality caveat + reputational-
  incentive sev M) est programmé Phase K (`plan §14.2`, rows 41/§19). Phase E n'a
  PAS besoin d'écrire le §16 complet (data-plane activation = F+, caveat
  confidentialité scope cut #4 inchangé). MAIS comme Phase E est la 1re à publier
  une perf-map influençant le routing, le COMMIT BODY Phase E (`## G8
  traceability` / `## Pre-launch protocol`) doit enregistrer la posture SI-3/SI-4
  (tau advisory, fallback allowlist-gated, vérification-comme-autorité-intégrité)
  pour que le §16 Phase K ait la source. Pas d'édition fichier threat-model en E ;
  une doc-note dans le commit body + une forward reference est le move cohérent.
- **[INFO] S3-E7** — L'AMPLIFICATION DE LECTURE perf-map est bornée par la
  posture subscribed-only existante. Une perf-map sur iroh-docs est lisible par
  qui a le doc ticket. Dans une ComputeGroup privée le doc est partagé seulement
  entre membres admis (D5), donc la surface de lecture = le groupe lui-même — PAS
  la surface publique Browse/anchor que §15.1 a dû défendre. La perf-map fuite une
  topologie RTT-pairwise grossière du groupe à ses propres membres (un hint
  fingerprinting SI-3), mais c'est une info que les membres obtiennent déjà en
  mesurant leur propre `conn_rtt` ; elle n'ajoute AUCUNE disclosure nouvelle à un
  non-membre (rejeté au handshake `sbfb/shard/1`, `shard.rs:222-231`, et ne détient
  jamais le doc groupe). Résiduel = visibilité topologie intra-groupe, sev Low,
  assume-and-document.

## S4 — wire format (contribution PLAN-ADAPT)

S4 tranche la question décisive Phase E avec une forme binaire : la perf-map est
un raw-op NON SIGNÉ, PAS un nouveau payload canonical signé — mais le libellé du
plan (`§8.2` « perf-map ... republiée 1-2s iroh-docs », test #3
`perf_map_republished_to_doc`, fail-fast row 28, acceptance §8.4) doit être
corrigé sur trois points précis, d'où PLAN-ADAPT.

- **[BLOCKER] S4-F1** — Perf-map DOIT être un raw-op NON SIGNÉ
  (`serde_json::Value`), PAS un nouveau payload canonical signé. §19
  (`sprint77_plan.md:648-649`) budgète exactement 4 `DOMAIN_*` (compute_group,
  shard_plan, run_proof, activation_commit) et AUCUN pour perf-map ; fail-fast
  row 38 (l.585) exige tous les `*_FORMAT_VERSION = 1` + seulement ces `DOMAIN_*`
  additifs. La perf-map ne porte aucune autorisation/non-répudiation (contre
  manifest « I AUTHORISE » / RunProof « I EXECUTED », `shard_plan.rs:16-20`) ;
  c'est de la donnée control-plane LWW (`addendum:77-78,108`). Créer un perf-map
  `DOMAIN_*` violerait la liste close §19. Ride la policy raw-op additive
  pre-launch (CLAUDE.md « Feed extensible via raw-op » ; `public_feed.rs:116-123`
  « Adding a typed variant does NOT bump FEED_FORMAT_VERSION »). 0-bump confirmé.
- **[BLOCKER] S4-F2** — `nexus-coordinator-rs` n'a AUCUN accès iroh-docs WRITE —
  le test #3 `perf_map_republished_to_doc` (l.273) + acceptance « perf-map
  propagée sur le doc » (l.280) + fail-fast row 28 (l.575) ne peuvent PAS lancer
  un vrai `doc.set()` in-crate. PREUVE : coordinator `Cargo.toml:11-29` aucune dep
  iroh/iroh-docs ; `lib.rs:11-38` aucun module node/docs ; tous les
  `doc.set()`/`set_bytes()` vivent dans `nexus-shell-daemon`
  (`dispatch_loop.rs:49,457,497` ; `feed_sync.rs:63` ; `storage_api.rs:362,459`) ;
  le wrapper `DocHandle::set` est `nexus-core-rs/docs.rs:248`. CORRECTION PLAN :
  Phase E livre la STRUCTURE PerfMap + le raw-op (de)serialize testé in-crate en
  round-trip bytes↔`serde_json::Value`↔bytes avec `Eq` stable (miroir
  `placement.rs` crate-local, acceptance `-p nexus-core-rs -p nexus-coordinator-rs`
  l.279/355). Le republish iroh-docs littéral est de la glue daemon mince (hors
  scope pure-compute ou wiring daemon-side séparé), exactement comme
  `placement.rs:269` laisse `fallback_node` « assigned by Phase E ». **Reformuler
  le test #3 en round-trip serialize, pas un doc write networké.**
- **[CONCERN] S4-F3** — Résolution META float (binaire) : `rho`/`tau` DOIVENT être
  micros entiers (`u64`), no-float, même si un raw-op non signé tolère
  techniquement les floats. Phase D a déjà choisi micros entiers dans le MÊME
  crate pour les MÊMES raisons : `RttMatrix` stocke `as_micros()` `u64`
  (`placement.rs:104-146,129-131`) ; doc module (`placement.rs:28-38`) garde
  l'arithmétique entière-exacte pour une sortie déterministe/`Eq`-comparable en
  test ; `RunMetrics` all-integer avec milli-tokens/sec (`shard_plan.rs:373-399`).
  Un `f64` perf-map casserait `Eq` sur PerfMap (requis pour test #4
  `routing_recomputed_on_perf_map_update` et test #3 round-trip) et ne
  round-trip pas bit-identique cross-platform (footgun `canonical.rs:11-21` /
  `shard_plan.rs:47-53`) — minant le sweep `dp2` déterministe. Ce n'est PAS la
  cause META A/B/C revivée : celles-là étaient des payloads JCS SIGNÉS ; la
  perf-map E est non signée et la tension float est PRÉ-RÉSOLUE par le précédent
  `placement.rs` micros entiers. Phase D a rompu le 3x streak ; Phase E miroir le
  choix de Phase D — guidance, pas un signal « 4e même cause ».
- **[CONCERN] S4-F4** — Population `fallback_node` = 0-bump, aucun
  `FORMAT_VERSION`, avec un caveat tamper. `fallback_node` est
  `#[serde(default)] Option<[u8;32]>` (`shard_plan.rs:173-174`) et fait partie des
  canonical bytes du manifest signé (chaque champ `ShardAssignment` contribue via
  `ShardPlan` dans `ShardedSessionManifest`, `shard_plan.rs:142-179,337`). Phase E
  doit calculer les cibles fallback au temps PLAN et laisser l'initiateur re-signer
  un NOUVEAU manifest avec `revision++` (`shard_plan.rs:255-259` existe précisément
  pour le re-planning) — il ne doit PAS muter un manifest déjà signé in-place
  (after-sign tamper, attrapé par `manifest_verify_rejects_tampered_payload`
  `shard_plan.rs:692-705`). Un plan all-`None` (Phase D, `placement.rs:271`) et un
  plan avec `Some` fallbacks sont tous deux du wire v1 valide : additif, 0 nouveau
  `DOMAIN_*`, aucun bump `FORMAT_VERSION` — seulement `revision++` au re-plan.
- **[INFO] S4-F5** — DAG routing sweep + churn (heap fallback + cache activations
  + `replace_failed_server` O(t)) sont du CALCUL PUR EN MÉMOIRE sur la sortie
  placement, exactement comme le water-filling/k-medoids Phase D — pas de type
  signé, pas de doc write, pas de nouveau wire. Le `dp2 = min(...,dp2(l,g)+
  rho<g,g'>+tau<g',l+1>)` (`plan §8.2:261`, `addendum:70-73`) opère sur la PerfMap
  micros entiers ; O(L·R²) négligeable à 3-5 peers. Précédent `placement.rs`
  étendu : déterministe, `Eq`-comparable, fixture-testé in-crate. Confirme un
  coeur EXECUTE-shaped sous les corrections de libellé PLAN-ADAPT (S4-F1/F2/F3).

---

## Spec concrète Phase E

### Module
**Pur-compute (coordinator)** : `crates/nexus-coordinator-rs/src/placement.rs`
(étendre le module Phase D) **ou** un module frère `routing.rs` net-new
(`pub mod routing;` dans `lib.rs:11-39`). Calcul interne côté initiateur. Importe
`nexus_core_rs::shard_plan::{ShardPlan, ShardAssignment, ShardRole,
KvCachePolicy}` (dep path déjà présente, `Cargo.toml:12`). Réutilise le
`RttMatrix` micros-entier Phase D (`placement.rs:104-146`). **0 import iroh,
0 import worker-core, 0 nouveau dep.**

**Glue iroh-docs (daemon)** : le republish perf-map 1-2s + l'ingest authentifié
vivent dans `nexus-shell-daemon` (détenteur du doc handle ;
`dispatch_loop.rs:49` `doc.set(author,key,value)` ; `feed_sync.rs:63`). Le
coordinator ne fait que PRODUIRE/CONSOMMER une `PerfMap` (struct + raw-op
serde). **Split BLOCKER S2-F1/S4-F2 : si on tente un write iroh-docs depuis
coordinator, ça ne compile pas.**

### Types d'entrée / d'état (INTERNES, non-wire-signés)
```text
PerfMap {
    // rho : RTT round-trip MESURÉ par paire, micros entiers (miroir RttMatrix).
    //       (le plan dit "one-way" ; conn_rtt rend le round-trip — doc-note S2-F7/S1A-4)
    rho_micros: BTreeMap<([u8;32],[u8;32]), u64>,
    // tau : latence per-layer/GPU SELF-REPORTED (advisory, S3-E3), micros entiers.
    tau_micros: BTreeMap<([u8;32], u32 /*layer*/), u64>,
}                          // derive PartialEq/Eq (test #3 round-trip, test #4 change-detect)
                           // (de)serialize en serde_json::Value raw-op — NON signé, 0 DOMAIN_*
RoutingChain               // sortie du DP : Vec<group_id> par couche, latence totale micros
FallbackHeap               // BinaryHeap<(latency_micros, worker_pubkey, group)>, déterministe
ActivationReplayCache      // ring/LRU borné par ACTIVATION_REPLAY_CACHE_MAX (S3-E5/S2-F5)
```
- `rho`/`tau` = **micros entiers** `u64` (S4-F3/S1b-4 ; PAS de `f64`).
- La `PerfMap` (de)serialize en `serde_json::Value` (raw-op) ; **PAS** de sig
  Ed25519, **PAS** de `canonical_bytes`, **PAS** de `DOMAIN_*` (S4-F1/S2-F2).
- L'authentification = `author_pubkey` de l'enveloppe `FeedEntry` EXISTANTE +
  ingest gate `compute_group.is_member` (S3-E1), wiring daemon-side.

### Algo (a) routing DAG sweep DP (déterministe)
1. Indexer le DAG par couche `l ∈ [0, L)` ; R groupes candidats par couche.
2. Relaxation forward un seul balayage G→D :
   `dp2(l+1,g') = min over g of (dp2(l,g) + rho<g,g'> + tau<g',l+1>)`
   — plus court chemin sur DAG topo-ordonné par `l` (S1A-1). O(L·R²), R≤5.
3. Coûts = micros entiers depuis la `PerfMap` ; tie-break déterministe par
   `worker_pubkey`/`group_id` (miroir PAM `placement.rs:499/514/550/563`) pour
   une `RoutingChain` stable (`routing_dag_sweep_selects_min_latency_chain`).
4. **AUCUN rand** (S1b-3/S1A-7). `tau` ADVISORY, jamais frontière de confiance
   (S3-E3) ; préférer `rho` mesuré.

### Algo (b) churn actif Petals (déterministe)
- `replace_failed_server(t)` O(t) : prendre le serveur suivant détenant le stage
  `t` dans le `FallbackHeap` ordonné par `(latency_micros, worker_pubkey)`
  (S1A-2/S1b-3). Remplacer UN assignment, pas re-planifier tout le pipeline.
- Le heap tire SEULEMENT de l'allowlist ComputeGroup (aucune relaxation
  d'admission au churn — `shard.rs:227 is_member`, scope cut #8, S3-E4).
- Le cache d'activations client-side rejoue le stage sur le remplaçant ; **borné**
  par `ACTIVATION_REPLAY_CACHE_MAX`, eviction oldest-frontier-first (S3-E5).
- Ordre fallback par RTT MESURÉ (`rho`), PAS par `tau` self-reported (anti-SI-4,
  S3-E4). `churn_replaces_failed_server_oturn` reproductible.

### Perf-map shape (BLOCKER S4-F1/S2-F2)
- **Raw-op NON signé** `serde_json::Value` (`FeedEntry.op`), LWW, **0 bump
  `FEED_FORMAT_VERSION`**, **AUCUN `DOMAIN_*`** (§19 clos à 4).
- `rho`/`tau` micros entiers `u64` (S4-F3).
- Auth = enveloppe `FeedEntry` signée existante + ingest
  `compute_group.is_member` (S3-E1, daemon-side).
- COMPUTE (ingest rho/tau → DP) = coordinator pur ; PUBLISH (1-2s) = daemon glue.

### fallback_node (S2-F6/S4-F4 — runtime routing, PAS mutation in-place)
- `placement.rs:269-271` le met `None` ; Phase E le peuple depuis le
  `FallbackHeap` au temps PLAN. `#[serde(default)] Option<[u8;32]>`
  (`shard_plan.rs:173-174`) — 0 bump wire.
- Si re-plan : `ShardedSessionManifest::new(... revision+1 ...)` → re-sign
  (`shard_plan.rs:255-259,330-344`). **NE JAMAIS** muter un manifest déjà signé
  in-place (tamper, `manifest_verify_rejects_tampered_payload`
  `shard_plan.rs:692-705`).

### Named constants (S2-F8, README §6.9)
- `PERF_MAP_REPUBLISH_INTERVAL` (1-2s).
- `ACTIVATION_REPLAY_CACHE_MAX` (borne cache churn, S3-E5/S2-F5).
- Borne du `FallbackHeap` + tout cap du DP. **Pas de magic number nu** (G-REVIEW).

### Tests (plan §8.3 ; acceptance `-p nexus-core-rs -p nexus-coordinator-rs`)
1. `routing_dag_sweep_selects_min_latency_chain` — fixture `PerfMap`
   micros-entiers → `RoutingChain` min-latence déterministe (`Eq` stable).
   **In-crate coordinator** (calcul pur).
2. `churn_replaces_failed_server_oturn` — drop worker → re-route via
   `FallbackHeap`, inférence continue depuis le cache d'activations borné.
   **In-crate coordinator**.
3. `perf_map_republished_to_doc` — **REFORMULÉ** (S4-F2) : round-trip
   bytes↔`serde_json::Value`↔bytes de la `PerfMap` avec `Eq` stable. **In-crate**
   (le coordinator n'a pas de Doc handle). Le `doc.set()` networké réel = glue
   daemon, hors pure-compute.
4. `routing_recomputed_on_perf_map_update` — une `PerfMap` mise à jour
   (`Eq`-différente) déclenche un re-calcul de la `RoutingChain`. **In-crate**.

### Acceptance / vérification
- `cargo nextest run -p nexus-core-rs -p nexus-coordinator-rs --locked` vert.
- `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --locked
  -- -D warnings` + `cargo test --workspace --locked --doc`.
- **Dual-platform** : Windows PowerShell + Docker `sbfb-ci` (rust:1.94) AVANT
  push (mémoire `feedback_dual_platform` / `feedback_wsl_before_push`).
- T1 E2E : `N-A-no-frontend-change` (pas de surface front en E).

### Wire / commit body
- **Pre-launch protocol : 0 bump wire** (perf-map = raw-op non signé ;
  `fallback_node` runtime additif ; tous `*_FORMAT_VERSION = 1`, S4).
- **AUCUN nouveau `DOMAIN_*`** (§19 clos à 4 ; S2-F2/S4-F1).
- Commit cible : `feat(core): Sprint 77 Phase E — Parallax routing DAG + Petals
  active churn`.
- Le commit body (`## G8 traceability` / `## Pre-launch protocol`) enregistre la
  posture SI-3/SI-4 (`tau` advisory, fallback allowlist-gated, vérification-comme-
  autorité-intégrité) pour la source du §16 Phase K (S3-E6). Pas d'édition
  fichier THREAT_MODEL en E.

---

## Risques résiduels

1. **Tentative de write iroh-docs depuis coordinator** (BLOCKER S2-F1/S4-F2) : le
   crate n'a aucune dep iroh → ne compile pas. **Mitigation** : split COMPUTE
   (coordinator pur) / PUBLISH (daemon glue) ; test #3 reformulé en round-trip
   serialize in-crate.
2. **Perf-map mintée en `DOMAIN_*` signé** (BLOCKER S2-F2/S4-F1) : fait sauter le
   budget §19 (4 clos) ET ré-ouvre la tension float-canonical = DESIGN-CONFLICT.
   **Mitigation** : perf-map raw-op `serde_json::Value` non signé, auth par
   l'enveloppe `FeedEntry` existante + `compute_group.is_member`.
3. **Fuite de float dans la `PerfMap`/`Eq`** : un `f64` casserait `Eq`
   (tests #3/#4) et le round-trip cross-platform. **Mitigation** : `rho`/`tau`
   micros entiers `u64` (miroir `RttMatrix`/`RunMetrics`, S4-F3/S1b-4).
4. **`tau` self-reported traité comme trust boundary** (SI-3, S3-E3) : un worker
   mentant biaise le routing vers lui. **Mitigation** : `tau` ADVISORY, préférer
   `rho` mesuré, vérification N0-N3 RunProof = autorité d'intégrité.
5. **Re-route churn vers complice** (SI-4, S3-E4) : **Mitigation** : heap fallback
   tire seulement de l'allowlist ComputeGroup, ordonné par `rho` mesuré +
   tie-break `worker_pubkey`, jamais par `tau`.
6. **Cache d'activations non-borné** (memory-DoS, S3-E5) + sur-lecture scope cut
   #5 : **Mitigation** : ring/LRU borné `ACTIVATION_REPLAY_CACHE_MAX`, local
   (pas de distribution iroh-docs), doc-note KV distribué/O(t) = post-S77.
7. **Mutation in-place d'un manifest signé** (S4-F4) : after-sign tamper.
   **Mitigation** : re-plan → `revision++` + re-sign d'un NOUVEAU manifest, jamais
   d'édition d'un manifest déjà signé.
8. **Copie de `thread_rng()` des modules voisins** (`dispatcher.rs:81`,
   `canary_input.rs:230`, `upload_queue.rs:107`) : casse le déterminisme DP/heap.
   **Mitigation** : DP + `BinaryHeap` déterministes, 0 rand (S1b-3/S1A-7).

**META** : 3 PLAN-ADAPT consécutifs A/B/C avaient la même cause (float-R&D vs
no-float-canonical) ; **Phase D a rompu ce motif** (cause stale-API RTT/VRAM +
déterminisme). **Phase E ne le ravive PAS et n'est pas la 4e occurrence** : la
perf-map est un raw-op NON signé où le no-float ne lie pas, donc le float y est
*techniquement* toléré — mais `rho`/`tau` sont imposés micros entiers pour le
déterminisme/`Eq`/tests, exactement le choix DÉJÀ fait par Phase D dans le même
crate pour les mêmes raisons. La cause du PLAN-ADAPT Phase E est **DIFFÉRENTE**
d'A/B/C ET de D : c'est le **split crate-boundary** (coordinator sans iroh-docs)
+ la **prescription perf-map-reste-raw-op** + la **posture d'auth/anti-bias
SI-3/SI-4**. Le motif « 3x même cause » reste rompu ; **aucun signal-meta « 4e
même cause » à noter**.
