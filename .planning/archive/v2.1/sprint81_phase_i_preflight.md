# Preflight S81 Phase I — Orchestrateur de session sharding in-vivo (ex-S78)

## Verdict: PLAN-ADAPT

Le **but** de Phase I est CORRECT et ne touche AUCUNE Day-0 (pipeline-parallel exclusif,
Parallax, ALPN `sbfb/shard/1` inchangé, N0-N4, groupe privé Ed25519) : construire l'orchestrateur
de session in-vivo dont l'absence a rendu S77/S76 RIG-ABSENT/DIFFERE, en **pure composition** des
primitives déjà livrées et threat-modelées (placement D, routing/churn E, data-plane F2, manifeste
signé F2, RunProof N0-N3 G/H/I). L'axe deps = **EXECUTE** (0 dep nouvelle, prouvé), l'axe décisions
= **EXECUTE** (0 conflit gelé), l'axe threat = **EXECUTE** (0 nouvelle classe de menace), l'axe wire
= **EXECUTE** (0 bump tenable). Mais la ligne « Livrables » du plan (§Phase I `sprint81_plan.md:368`,
« orchestrateur runnable + T1 in-process, +4..8 Rust ») **sous-liste** la surface que le plan lui-même
délègue explicitement au préflight (`:367` « le préflight de phase précise les livrables exacts depuis
le dossier S78 + l'état réel du code shard »). Trois écarts load-bearing forcent une adaptation :

1. **Surface HTTP/registre STRUCTURELLEMENT forcée en I** (S0 P1) : S78 §10-(e) impose de peupler le
   store HTTP lisible (`live_shard_session ≠ None`) et le harness `b3_shard` (Phase J) appelle
   littéralement 3 routes ABSENTES (`POST .../generate`, `GET .../result`, `POST .../drop-shard`) +
   un payload étendu (`ttft_s`/`toks_per_s`/`run_proof`/`result_text`/`rtt_frontier_ms`). Comme
   **Phase J = 0 Rust** (`plan:387`), ces routes NE PEUVENT PAS atterrir en J — elles sont imposées
   à Phase I par l'allocation delta du plan.
2. **Cycle de vie à enrichir de 4 → 6 étapes** (S1a P1) : le plan liste `annonce/placement/dispatch/
   collecte`, mais toute la prior-art OSS traite en 1re classe la **barrière de readiness** (cause
   racine du RIG-ABSENT S77 : le 1er token dispatché vers un shard qui n'a pas fini de fetch les poids
   → hang) et le **teardown**. À nommer : `annonce/placement/READINESS/dispatch/collecte/TEARDOWN`.
3. **SI-9 (withholding, Sev M, carry OUVERT)** (S1a P1) : `read_frame` n'a aucun timeout ; l'orchestrateur
   (côté dialer) est le SEUL composant où armer un deadline par-hop → `fallback_node` (champ déjà présent).
   Livrer le driver sans l'armer re-certifierait un livrable PROVISIONAL en laissant intacte sa faille
   de liveness Sev-M connue, dans le composant même qui la possède.

Toutes ces adaptations sont **0-bump wire** (composition + registre en-mémoire + deadline dialer-side)
et n'exigent aucune dep nouvelle. D'où **PLAN-ADAPT** : approche corrigée concrète, evidence code/OSS,
Day-0 intactes. Conséquence chiffrée : réviser le delta tests de **+4..8 → +8..14 Rust**.

## Rationale du verdict

Signaux des 6 scans : **S0 PLAN-ADAPT**, **S1a PLAN-ADAPT**, S1b EXECUTE, S2 EXECUTE, S3 EXECUTE,
S4 EXECUTE. Vérifications adversariales : **0 claim réfuté** sur les 6 scans (upheld 8/8, 8/8, 6/6,
10/10, 5/5, 7/7). Le verdict global n'est pas EXECUTE parce que les deux PLAN-ADAPT (S0 + S1a) portent
un **re-scoping** de ce que Phase I livre (routes+registre forcées, cycle de vie à 6 étapes, SI-9 armé),
pas un simple détail ; il n'est pas DESIGN-CONFLICT parce qu'aucune adaptation ne heurte une décision
gelée ni un état de fait — elles explicitent des livrables que le plan délègue au préflight.

**Ré-vérifications indépendantes (spot-checks du synthétiseur, tous CONFIRMÉS) :**
- `live_shard_session(_session_id) -> None` = stub verbatim `http.rs:2135` ; commentaire `:2132-2134`
  **mandate déjà** le double-gate `DOMAIN_SHARD_PLAN_V1` + `is_member` AVANT insert ; projection
  `project_shard_session` = `{session_id, member_count}` seulement (`:2119-2123`).
- `b3_shard_pipeline.sh` appelle `.../generate` (`:333`), `.../result` (`:340`), `.../drop-shard`
  (`:361`) — inexistantes ; lit `run_proof`/`result_text`/`toks_per_s`/`ttft_s`/`rtt_frontier_ms`.
- `RunProof::new`/`RunProofEntry::sign` : call-sites hors-test = **0** (rerun.rs:160/175 + validator.rs:986/995
  sous `mod tests` ; shard_plan.rs = tests ; activation_commit.rs:178 = doc-comment). Phase I = 1re émission PROD.
- `nexus-coordinator-rs` « has no iroh dependency at all » (`routing.rs:38`) → NE PEUT PAS héberger
  l'orchestrateur (besoin `open_shard_connection`) ; `SHARD_ALPN` absent de daemon-core → **HOME = binaire
  `nexus-shell-daemon`** (seul crate au-dessus de core-rs data-plane iroh ET coordinator-rs pur, + possède
  le doc-handle + héberge la route stub).

## S0 — Inventaire état réel S77 (HEAD `12e3954`, post-E/F/G/H iroh 1.0) — PLAN-ADAPT

Adversarial : **8/8 upheld, 0 réfuté**, inventaire EXACT, 0 hallucination.

**CE QUI EXISTE (livré S77, hermétique, recompilé iroh 1.0) :** primitives signées `shard_plan.rs`
(`ShardedSessionManifest(/Entry)` + `RunProof(/Entry)` + `RunMetrics` + `ShardPlan`/`ShardAssignment`,
v1, 0 bump) ; data-plane `shard.rs` (ALPN `sbfb/shard/1`, `ShardProtocol::accept` admission-first,
`open_shard_connection`, `write_frame`/`read_frame`, `conn_rtt`, trait `ShardForwarder`+`EchoForwarder`,
topologie STAR pilotée dialer `:219-222`) ; placement Parallax `placement.rs` (PUR iroh-free) ;
routing+churn `routing.rs` (PUR iroh-free `:38`) ; backend worker `worker-core/src/llm/shard.rs`
(`ShardBackendForwarder`, gaté `feature=llm_llama_cpp`, JAMAIS build CI) ; claim gate `shard_claim.rs`
(`authorize_claim` crypto+membership pur + `assess_capacity` VRAM fail-closed) ; route read-only
`GET /api/daemon/shard-session/{id}` (`http.rs:309`) → **stub `None`**.

**CE QUI MANQUE (= Phase I, défini par S78 §10 a-e) :** l'orchestrateur qui (a) monte une session,
(b) pilote une génération token-par-token cross-shard, (c) mesure TTFT/tok-s, (d) émet un RunProof
signé in-vivo, (e) peuple le store HTTP lisible. **Aucun de ces 5 chemins n'existe en prod.**

- **S0-P1 (load-bearing)** — Le plan sous-liste la surface HTTP/registre que S78 §10-(e) IMPOSE et
  que `b3_shard` appelle concrètement. Les 3 routes + le payload étendu + le registre + la bascule
  `live_shard_session ≠ None` sont **forcés en I** (Phase J = 0 Rust). Renforcement adversarial :
  « c'est imposé par l'allocation delta du plan », pas seulement « une décision de séquencement à acter ».
- **S0-P2 (crate-home)** — HOME = binaire `nexus-shell-daemon` (nouveau module `shard_session.rs`
  + registre). NE PAS mettre l'orchestrateur dans coordinator-rs (romprait l'invariant iroh-free `routing.rs:38`).
- **S0-P2 (delta sous-compté)** — `+4..8` couvre le T1(6) lifecycle seul ; registre + routes + payload
  + 1re émission RunProof portent chacun leurs tests → réel **+8..14** (garde anti-faux-vert : annoncé == git-count).
- **S0-P2 (seam T1 sans GPU)** — nommer explicitement `ShardForwarder`+`EchoForwarder` (pattern
  `two_node_shard_fixture`, 2 nœuds in-process) ; CI ne build JAMAIS `llm_llama_cpp` → aucune dépendance GGUF dans le test.
- **S0-INFO** — generate/result/drop-shard + payload étendu = frontières loopback → clôture docs-contrat
  Phase K (leçon S80-K-1) ; « 2-machines réelle » de I est le **gate LIVE de J** — I ne prouve que le lifecycle in-process.

## S1a — Prior-art OSS orchestration de session pipeline-parallel — PLAN-ADAPT

Adversarial : **8/8 upheld, 0 réfuté** (Petals v2 direct-s2s + chaîne-avant-1er-token VÉRIFIÉ release ;
llama.cpp RPC cache + « fragile and insecure » VÉRIFIÉ README ; exo download-status VÉRIFIÉ).

- **S1a-P1 (barrière de readiness ABSENTE — cause racine RIG-ABSENT S77)** — Le plan liste 4 verbes
  sans étape de readiness avant le 1er token. exo préfère les nœuds par « download status » (poids déjà
  chargés), Petals assemble la chaîne complète avant d'injecter le 1er token ; côté SBFB il n'existe
  AUCUN signal de readiness (claim gate = fonctions pures, RunProof = post-exécution). Sans barrière, le
  1er token part vers un shard qui n'a pas fini de fetch (GGUF multi-Go) + charger en VRAM → hang.
  **Adaptation 0-wire** : l'orchestrateur ouvre la connexion QUIC + une sonde-frame à chaque shard aval
  sur l'ALPN existant ; si un shard timeout dans un deadline → la session ne démarre pas (`BLOCK{diagnosis}`)
  au lieu de hang. T1 doit asserter qu'aucune frame de dispatch n'est émise avant que tous les shards aient ACK.
- **S1a-P1 (SI-9 withholding Sev M non scopé)** — `read_frame` sans timeout (`shard.rs:151-169`) : un
  membre admis qui se tait bloque tout le pipeline. L'orchestrateur (côté dialer) est le SEUL endroit où
  armer un deadline par-hop → `fallback_node` (`shard_plan.rs:174-175`). Le bucket delta-tests « erreurs »
  DOIT inclure le test `hop bloqué → timeout → fallback`. Fait avancer/fermer le carry SI-9 dans le composant qui le possède.
- **S1a-P2 (topologie HUB vs direct-s2s)** — L'`accept()`-retour-au-dialer figé S77 force un HUB : chaque
  frontière fait ~2 traversées WAN (shard→orchestrateur→shard suivant). **NE PAS changer le data-plane**
  (figé), mais (1) stater que l'orchestrateur I est un HUB, et (2) juger le verdict Phase J contre une
  **baseline HUB**, PAS l'enveloppe Petals direct-s2s §6 (sinon faux-BLOCK). Réaliser le direct-s2s = évolution data-plane hors I.
- **S1a-P2 (churn)** — Trancher **resume-from-cache** (le design a figé l'actif Petals : cacher la dernière
  frontière réussie + rejouer sur re-route) OU **coupe explicite** (échec propre + `worker_drop_count` =
  « observés non récupérés »). Documenter le choix ; le test « erreurs » T1 couvre un drop mid-session.
- **S1a-INFO (teardown)** — nommer la libération KV `LocalEphemeral` + close QUIC gracieux comme étape de
  fin (2e session sur le même groupe privé sans fuite VRAM).

## S1b — Deps / CVE — EXECUTE

Adversarial : **6/6 upheld, 0 réfuté** ; 2 vérifs live (`cargo tree -d`, `cargo deny check advisories` = OK).

- Phase I réalisable à **0 dépendance nouvelle** : clap 4.5 déjà workspace (`Cargo.toml:112`, 4 consommateurs)
  OU `std::env::args` (précédent `examples/shard_node.rs:38`) ; toutes les briques (tokio full, iroh =1.0.1
  + ALPN codé, ed25519-dalek 2.1 + serde_jcs pour RunProof, serde_json, `std::time::Instant` pour TTFT/tok-s
  précédent S76 `generation_time_ms`, placement/routing coordinator-rs) sont existantes ou code S77.
- `cargo tree -d` = duplications héritées de la migration iroh 1.0 (redb 3.1.3/4.1.0 les DEUX via stack iroh
  — `redb@3` via iroh-docs 0.101, `redb@4` via iroh-blobs 0.103 ; rand/base64/http/rustls/ed25519-dalek/hickory
  doubles) ; Phase I n'ajoute aucune dep → **0 duplication nouvelle**. Flip `[bans] multiple-versions` reste
  `warn` (bloqué par pin iroh ed25519-dalek =3.0.0-rc.0), carry S82 — NE PAS rouvrir le gate Phase G.
- RustSEC (deny.toml requalifié S81 Phase G, 2026-07-08) : 6 ignores bornés carry S82 (hickory-0.24 opt-in
  default-off + quick-xml via iroh input local), `yanked=deny`, aucun advisory sur redb 4 / iroh-blobs 0.103 / la stack shard.

## S2 — Décisions historiques (cohérence Day-0) — EXECUTE

Adversarial : **10/10 upheld, 0 réfuté** (2 imprécisions non-load-bearing notées : claim 7 « same-backend »
= en fait CPU-head+Metal-tail ; pointeur `:160-166` → « Track test-delta » et non le gate b3-PASS `:62-64`).

- L'orchestrateur est une **RE-CERT d'un livrable S77 déjà conçu et figé**, pas une feature neuve ; périmètre
  = S78 §10 a-e verbatim. Design GELÉ (addendum SOTA 2026-05-30 + memory `sharding_design_frozen`) :
  pipeline-parallel EXCLUSIF, `open_bi` long-vécu (1 conn QUIC persistante/paire), admission crypto-avant-IO,
  N0-N4, groupe privé Ed25519. Gap honnête confirmé (ALPN test-only, RunProof `cfg(test)`, stub `None`).
- **S2-P2** — « OUTIL opérateur, pas feature produit » sous-estime le câblage daemon-side réel : (a) monter
  `SHARD_ALPN` dans le nœud en exécution (aujourd'hui test-only ; le boot prod ne monte que `SEED_ALPN`),
  (e) peupler un registre gaté `DOMAIN_SHARD_PLAN_V1` + `is_member`. Ce n'est PAS une contradiction (route
  existe, ALPN string inchangé, FORMAT_VERSION=1) mais un périmètre daemon à expliciter.
- **S2-P2 (Phase J)** — Tolérance N0 TOPLOC vs dérive inter-backend : cosine **0.978** CUDA(5080)+Metal(M2)
  < seuil same-backend >0.999 (dérive numérique caractérisée, PAS un bug de coupe) → risque `BLOCK{n0-false-reject}`
  sur un split correct. Fence N0 émise in-vivo à calibrer (Q4_K_S ~19Go OU fence élargie documentée) au préflight J.
- **S2-P2 (Phase J)** — RTT multipath live encore UNVERIFIED (`conn_rtt` lit `PathId::ZERO` single-path ;
  Phase E l'a laissé UNVERIFIED-high-risk). Le T1 in-process single-path loopback n'exerce PAS ce risque.
- **S2-P1 (Phase J bloqué, Phase I non)** — Convergence WAN cross-machine = **prérequis DUR** du sharding
  (bug live S76) ; carry `RE-DRIVE-ON-INGEST` 3/3 MANDATORY + escalade S75 boot-SEED OVERDUE à l'audit gate S81.
  Phase I loopback NON bloquée ; Phase J peut hériter `RIG-ABSENT/BLOCK{convergence}` plutôt qu'un rig-absence.
- **S2-INFO** — Contraintes Phase J : arch-llama uniquement (gemma4/MoE non patché dans le fork), ~20Go →
  chargement partiel obligatoire, backend feature-gated (T1 in-process SANS GPU), incentive réputationnel
  non-monétaire (PO-12, 0 slash/stake/burn), no-float sur consensus + named constants.

## S3 — Threat model — EXECUTE

Adversarial : **5/5 upheld, 0 réfuté** (chaque emplacement relu verbatim vs HEAD `12e3954`).

- La surface sharding est DÉJÀ couverte exhaustivement (§5.9 STRIDE `sbfb/shard/1`, §16 SI-1..SI-11 + N0-N3,
  §6 LINDDUN, §4 DFD, §2 asset A8). L'orchestrateur y est nommé comme nœud-TÊTE/DIALER (trust-A membre)
  marqué « carry S78 non-câblé » à 4 endroits. Admission Ed25519 QUIC (`remote_id` vs `is_member` AVANT
  `accept_bi`, close `SHARD_REJECT_NOT_MEMBER`) : un non-membre est fermé au handshake, ne peut empoisonner
  le placement (manifeste signé vérifié AVANT toute I/O). **Aucune nouvelle classe de menace, 0 nouveau `DOMAIN_*`, 0 nouvel invite.**
- **S3-P2 (si registre live)** — Si Phase I peuple `live_shard_session`, **préserver** le double-gate déjà
  mandaté (`is_member` + `DOMAIN_SHARD_PLAN_V1` AVANT insert, `http.rs:2132-2134`) + la whitelist
  `ShardSessionView` (`member_count`-only, JAMAIS `worker_pubkey`/`initiator` ; SI-3/SI-4). Contrat déjà
  écrit dans le code — le review de phase DOIT le confirmer sur le code réel.
- **S3-INFO** — Aucune nouvelle credential (réutilise node key Ed25519 + bearer loopback durci). Delta de
  STATUT threat-model à acter en Phase K (les textes « carry S78 » deviennent périmés — MAJ §16 + §5.9 + note DFD).
- Résiduels ASSUMÉS INCHANGÉS par ce sprint : SI-1 (reconstruction activations, High), SI-4 (collusion, High)
  — limite physique, pas de TEE GPU grand-public 2026.

## S4 — Invariants wire format / store on-disk — EXECUTE

Adversarial : **7/7 upheld, 0 réfuté** (1 correction factuelle renforçante, cf. ci-dessous).

- `SHARD_ALPN = b"sbfb/shard/1"` = SEUL palier shard, inchangé ; `SHARD_PLAN_FORMAT_VERSION=1` /
  `RUN_PROOF_FORMAT_VERSION=1`. Le manifeste signé porte DÉJÀ tous les champs de lifecycle (`session_id`,
  `revision` monotone, `group_id` ; `RunProof.session_id` anti-rejeu) → **composition pure, aucun champ additif requis**.
- **Teardown = 0 op wire** : KV `LocalEphemeral` (jetée en fin) + FIN QUIC (`read_frame → Ok(None)`) / `conn.close`.
- Aucun store on-disk shard-session déployé (`live_shard_session` stub) → câbler un **registre LOCAL non-wire**
  est 0-bump et hors invariant store-migration. Statut de session (pending/active/complete/failed) = registre
  local OU projection serialize-only, **JAMAIS** un champ sur le canonical signé.
- **Chemin LIVE (Phase J)** : distribuer le manifeste via transport EXISTANT (feed raw-op additif 0-bump —
  précédent `SeedAnnounced`/`CuratorVouched` — OU iroh-docs), **JAMAIS** un nouvel ALPN de contrôle (design
  figé `shard.rs:4` : le control-plane ride sur docs/blobs/gossip). Résidu adversarial (A) : ce transport
  manifeste initiateur→workers **n'existe pas encore en code** — I/J DOIT le rider 0-bump, à vérifier au review.
- **S4-P2 (docs-contrat §6.12)** — Si `ShardSessionView` est enrichi : regen schema committé + `check-frontier-contracts.sh`
  + MAJ Zod front DANS LE MÊME COMMIT + étiquette docs-contrat (Track K). Correction adversarial (B) : le
  `.strict()` front est sur l'ENVELOPPE seule ; le sous-schéma `ShardSessionView` n'a pas `.strict()` (Zod strip
  silencieusement une clé inconnue) → un enrichissement additif du champ INTERNE ne casserait PAS le parse front,
  ce qui rend le gate 0-bump **encore plus permissif** (renforce EXECUTE). L'étiquetage §6.12 reste bonne pratique.

## Livrables exacts de Phase I

1. **Module orchestrateur `crates/nexus-shell-daemon/src/shard_session.rs`** (NOUVEAU ; HOME = binaire
   daemon, garde coordinator-rs iroh-free). Fonction de cycle de vie **6 étapes** composée :
   *(a) placement* `plan_placement(candidates, rtt, model, session_id)` (coordinator-rs) → `ShardPlan` ;
   *(b) manifeste* `ShardedSessionManifest::new` + `ShardedSessionManifestEntry::sign` (initiateur) ;
   *(c) READINESS-BARRIER* sonde transport 0-wire par shard aval sur l'ALPN existant, ACK requis avant le
   1er token, sinon `BLOCK{diagnosis}` (pas de hang) ;
   *(d) claim/dispatch* `authorize_claim` (crypto-avant-IO) + `assess_capacity` + `open_shard_connection`
   + `write_frame`/`read_frame` (HUB piloté dialer) **avec deadline par-hop (SI-9)** → `fallback_node` → re-readiness ;
   *(e) mesure* `RunMetrics` (`ttft_ms`, `decode_milli_tokens_per_sec` via `Instant`) ;
   *(f) collecte* `RunProofEntry::sign` **(1re émission PROD — aujourd'hui 0 hors `cfg(test)`)** ;
   *(g) TEARDOWN* discard KV `LocalEphemeral` + FIN QUIC gracieux.
2. **Registre de sessions en-mémoire** (daemon). Insert **gaté `DOMAIN_SHARD_PLAN_V1` + `is_member` AVANT
   insert** (mandate `http.rs:2132-2134`). `live_shard_session` lit ce registre (fin du stub `None` →
   `found:true`). **Projection privacy INCHANGÉE** (`member_count`-only, jamais `worker_pubkey`/`initiator` ; SI-3/SI-4).
3. **Surface HTTP dans `authed_routes`** (STRUCTURELLEMENT forcée en I car Phase J = 0 Rust) :
   `POST /api/daemon/shard-session/{id}/generate`, `GET /api/daemon/shard-session/{id}/result` (payload
   étendu `ttft_s`/`toks_per_s`/`run_proof`/`result_text`/`rtt_frontier_ms`), `POST /api/daemon/shard-session/{id}/drop-shard`.
   Frontières loopback (harness `b3_shard`, runtime distinct) → **tracer pour clôture docs-contrat Phase K**.
4. **CLI/harness opérateur** (sous-commande du binaire ; clap 4.5 déjà présent OU `std::env::args` précédent
   `shard_node.rs`) qui monte une session shard 2-machines réelle — le RUN LIVE = Phase J ; **Phase I livre l'OUTIL**.
5. **T1 sous-test (6) `session_shard_in_process`** — 2 nœuds loopback in-process via le seam
   `ShardForwarder`+`EchoForwarder` (pattern `two_node_shard_fixture`), SANS GPU/GGUF : asserte la plomberie
   complète (readiness ACK avant 1er frame → mount → manifest sign → claim authorize → drive frames → RunProof
   sign → registre → `live_shard_session` `found:true`) **+ cas d'erreur** (hop bloqué → timeout → fallback OU
   échec propre diagnostiqué). Runs **Win-natif + CI Linux, JAMAIS Docker-on-Windows** (`create_node` hang).
6. **Delta tests RÉVISÉ +8..14 Rust** (au lieu de +4..8) : lifecycle in-process, placement, dispatch,
   readiness-barrier, timeout→fallback, émission+verify RunProof, pin route(s), round-trip payload, projection
   privacy `member_count`-only. Garde anti-faux-vert : **annoncé == git-count** au commit.
7. **Enrichissements plan/doc (0 code)** : stater la topologie **HUB** (Phase J jugé sur baseline HUB, pas
   l'enveloppe Petals direct-s2s §6) ; nommer le cycle de vie 6 étapes ; trancher+documenter la sémantique
   churn (resume-from-cache OU coupe explicite + `worker_drop_count`) ; tracer les 3 frontières loopback pour Phase K.

## Risques résiduels

- **R-I-1 (P1, code-time)** — Faisabilité 0-wire de la readiness-barrier : une sonde-frame peut exiger un
  **discriminateur de type-de-frame** qui toucherait le layout wire de `sbfb/shard/1`. Mitigation : readiness
  via signaux **transport** (`open_bi` établi + 1er layer-block round-trip via le chemin frame EXISTANT /
  `EchoForwarder`), PAS un nouveau type de frame. Le **timeout SI-9 est sans ambiguïté 0-wire** (deadline
  dialer-side autour de `read_frame`). Si un discriminateur s'avère nécessaire → décision code-time préservant
  0-bump ; un nouvel ALPN de contrôle est INTERDIT (design gelé). À trancher au review de phase.
- **R-I-2 (P2, Phase J)** — Fence N0 TOPLOC vs dérive inter-backend (cosine 0.978 CUDA+Metal) → risque
  `BLOCK{n0-false-reject}`. Calibrer (Q4_K_S ~19Go OU fence élargie documentée) au préflight J.
- **R-I-3 (P1, Phase J bloqué / Phase I non)** — Convergence WAN = prérequis DUR de J (carry `RE-DRIVE-ON-INGEST`
  3/3 MANDATORY + escalade S75 boot-SEED OVERDUE à l'audit gate S81). Vérifier au préflight J que A2/A4 +
  convergence `PublicRegistryView` (Phase K) couvrent — sinon J hérite `RIG-ABSENT/BLOCK{convergence}`.
- **R-I-4 (P2, Phase J)** — RTT multipath live UNVERIFIED (`conn_rtt` = `PathId::ZERO` single-path). Le T1
  in-process ne l'exerce pas ; Phase J mesure la fiabilité RTT par-connexion sur liens résidentiels réels.
- **R-I-5 (P2, env)** — Le T1(6) utilise `create_node` (famille iroh-networked, env-bloquée Docker-on-Windows)
  → Win-natif + CI Linux, jamais Docker-on-Windows (déjà acté Phase K).
- **R-I-6 (INFO, docs-contrat K)** — generate/result/drop-shard + payload étendu = frontières loopback → DoD (d)
  indexation Phase K (llms.txt/REFERENCE/check-frontier-contracts), jamais l'excuse « 0 wire bump ».
- **R-I-7 (INFO, transport manifeste)** — le chemin initiateur→workers du manifeste n'existe pas encore en code ;
  I/J DOIT le rider sur un transport existant (feed raw-op additif OU docs), conformité à vérifier au review.

## Verdict final motivé + conséquences sur le code

**PLAN-ADAPT.** Le but (orchestrateur de session in-vivo, re-cert d'un livrable S77 PROVISIONAL, T1
in-process, 0 bump SBFB) est juste et ne touche AUCUNE Day-0. L'adaptation = **expliciter la surface que le
plan délègue au préflight** (`:367`) et **enrichir le cycle de vie** de 4 → 6 étapes, sur evidence code+OSS,
en restant 0-bump et 0-dep.

**Conséquences directes sur le code à écrire :**
1. HOME = nouveau module `crates/nexus-shell-daemon/src/shard_session.rs` + registre en-mémoire ; NE PAS
   toucher l'invariant iroh-free de coordinator-rs.
2. Câbler daemon-side : monter `SHARD_ALPN` dans le nœud en exécution (aujourd'hui test-only) + registre
   gaté `DOMAIN_SHARD_PLAN_V1`+`is_member` avant insert + `live_shard_session` lit le registre.
3. Ajouter les 3 routes (`generate`/`result`/`drop-shard`) + payload étendu — imposées à I par Phase J = 0 Rust.
4. Cycle de vie 6 étapes avec **readiness-barrier** (0-wire transport, cf. R-I-1) + **deadline par-hop SI-9**
   (0-wire dialer-side) → `fallback_node` ; 1re émission RunProof PROD ; teardown déterministe.
5. T1(6) in-process via `EchoForwarder` (SANS GPU), incluant le cas d'erreur `hop bloqué → timeout → fallback`.
6. Réviser le delta annoncé **+4..8 → +8..14 Rust** (garde annoncé == git-count).
7. Préserver strictement la whitelist `member_count`-only (SI-3/SI-4) ; 0 bump wire (manifest/RunProof/PerfMap
   v1, `sbfb/shard/1` inchangé) ; 0 dep runtime nouvelle ; heberger≠publier intact (outil opérateur du groupe privé).

Ce qui reste **hors Phase I** (Phase J, operator-gated) : le run LIVE 2-machines (RTX 5080 + Mac M2), le
verdict T2 `PASS/BLOCK{diagnosis}/RIG-ABSENT`, la calibration N0 finale, la mesure RTT multipath réelle, et
la dépendance à la convergence WAN (prérequis DUR de J, non de I).
