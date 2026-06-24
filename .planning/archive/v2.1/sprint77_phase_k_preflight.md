# Preflight G8 — Sprint 77 Phase K (wrap-up + gate produit)

> Synthèse des 5 scans factuels (S1 harness / S2 carries / S3 THREAT §16 /
> S4 wire 0-bump / S5 baseline + invariant clôture). Phase K = wrap-up SANS
> code fonctionnel net : harness d'acceptance + docs longue-vie + clôture
> carries + verification.md + audit_plan S78. Le main-thread écrit le
> harness/docs à partir de ce document.

## Verdict: PLAN-ADAPT

Le plan §14 tient dans sa structure (harness T2 artefact-JSON, docs §16/PATTERNS,
verification fail-fast, audit_plan S78, 0-bump wire confirmé). MAIS trois
corrections concrètes fondées sur le code réel sont nécessaires — aucune ne
touche une décision Day-0 figée :

1. **Harness T2 — statut attendu honnête = RIG-ABSENT (jamais faux PASS).**
   La route shard daemon est un stub `None` (`live_shard_session` → `None`,
   `http.rs:2146-2148`) ET il n'existe AUCUN orchestrateur de session
   prod qui pilote une génération token-par-token cross-shard, mesure
   TTFT/tok-s, ou émet un `RunProof` in-vivo (`RunProof::new` seulement sous
   `#[cfg(test)]` à `validator.rs:986`). Donc `ttft_s`/`toks_per_s`/`run_proof`
   ne sont peuplés par AUCUN chemin prod → un `pass()` (qui exige `tok/s ≥ 1`
   au §14.4) est **structurellement inatteignable aujourd'hui**. Le harness
   DOIT produire RIG-ABSENT (matériel 2-machines absent OU pipeline non
   montable end-to-end) — c'est PRÉVU par §14.4 (PROVISIONAL + carry P1 S78).
   PLAN-ADAPT vs un §14.2 lu naïvement comme « PASS génératif attendu ».

2. **verification.md §6 — row 30 `shard_backend_primitive` est un placeholder
   non-matérialisé** (`grep` = 0 fn). Le filtre `test(shard_backend_primitive)`
   matche 0 test → faux-vert silencieux. À remplacer par des noms hermétiques
   CI réels (`shard_window` / `top_k` / `hidden_token_count` / `toploc_commitment`).

3. **THREAT_MODEL §16 — relabel F5 + bump v13→v14 + SI-9 carry ouvert + phrase
   route shard-session + STRIDE/LINDDUN formels.** Lignes réelles confirmées
   (918/1228/1376), aucun drift depuis la review J.

Tout le reste = EXECUTE (0-bump wire prouvé par diff git, baseline 1949 Win
cohérente, carries traçables).

---

## S1 — Contrat harness (réutilisation + champs productibles vs RIG-ABSENT)

### Squelette à copier verbatim depuis `scripts/acceptance/b3_live_pc_vps.sh`
Le nouveau `scripts/acceptance/b3_shard_pipeline.sh` reprend tel quel :
- **`SCRIPT_DIR` resolve** (l.115) `cd "$(dirname "${BASH_SOURCE[0]}")"` → artefact/log/rig.env à côté du script quel que soit le cwd.
- **`RIG_ENV` data-config** (l.117-122) sourcé si présent, env vars explicites gagnent. Shard ajoute `MODEL_20GB` + `MAC_SSH` (2e machine Metal).
- **`emit_artifact()` python3 + fallback `_json_safe()`** (l.152-193) : `json.dumps` bullet-proof, fallback `tr -d` lossy mais JSON-valide pour `last_response` multi-lignes.
- **`num()` helper** (l.168-172) : champs numériques = `int(v)` sinon `null`, jamais une chaîne (`ttft_s`/`toks_per_s`/`rtt_frontier_ms`/`n_shards`).
- **3 verdicts / 3 exit codes** (l.195-214) : `rig_absent()` exit **3** (stage `preflight`), `block()` exit **1** (`$1`=stage `$2`=diag), `pass()` exit **0**. Chacun `emit_artifact` AVANT de sortir. `status` ∈ {PASS, BLOCK{diag}, RIG-ABSENT} (§14.4).
- **Préflight = RIG-ABSENT jamais BLOCK** (l.238-284) : SSH down / Ollama|MODEL absent / binaire absent / **réconciliation PROJECT_ID** (`GET /api/daemon/project-info` mismatch) → exit 3. Seul un timeout POST-submit est un product BLOCK. Shard ajoute gardes RIG-ABSENT : « modèle ~20 Go ne tient sur aucune machine seule » + « 2e machine Metal joignable ».
- **`vps()` wrapper SSH** (l.236) + token loopback `/auth/token` (l.267) + `AUTH="-H 'x-sbfb-token: $TOKEN'"` (l.269) réutilisables tels quels.
- **Auto-diagnostic depuis worker log** (l.388-399) : adapté shard = « frame jamais reçue à la frontière » (claim/delivery) vs « frontière reçue mais pas de boundary state » (forward/inférence).
- **Garde anti-faux-vert préservée** : `set -uo pipefail` (l.110) + `|| true` sur curl/ssh → commande absente retombe sur préflight RIG-ABSENT (bon défaut).

### Champ par champ §14.2 — source productible RÉELLE
Cible : `{status, stage, model, n_shards, ttft_s, toks_per_s, rtt_frontier_ms, run_proof, diagnosis, last_response}`.

| Champ | Productible ? | Source / raison |
|---|---|---|
| `status` | OUI | verdict des 3 fonctions, jamais prose |
| `stage` | OUI | chaîne libre (`preflight`/`claim`/`frontier-forward`) |
| `model` | OUI | env `MODEL_20GB`, pré-checkable Ollama/GGUF → sinon RIG-ABSENT |
| `n_shards` | **config-only** | `2` (cible 5080↔Mac) ; placement water-filling Phase D PAS exécuté in-vivo → honnête seulement si session montée → **RIG-ABSENT sans rig** |
| `ttft_s` | **NON** | `RunMetrics.ttft_ms` (`shard_plan.rs:381`) jamais peuplé en prod (pas de boucle decode pilotée) → **RIG-ABSENT** |
| `toks_per_s` | **NON** | `decode_milli_tokens_per_sec` (`shard_plan.rs:386`) jamais peuplé ; §14.4 exige `≥1` → **PASS impossible aujourd'hui → RIG-ABSENT** |
| `rtt_frontier_ms` | conditionnel | `shard_rtt`/`conn.rtt(PathId::ZERO)` (`shard.rs:180`) mesurable SI connexion `sbfb/shard/1` réelle 2-machines ouverte → **RIG-ABSENT sans rig** ; gate `BLOCK{rtt>80ms}` exerçable connexion réelle seulement |
| `run_proof` | **NON** | `RunProof::new` seulement sous `#[cfg(test)]` (`validator.rs:986`) ; aucun worker n'en signe in-vivo → **RIG-ABSENT** |
| `diagnosis` | OUI | chaîne, peuplée par block/rig_absent |
| `last_response` | OUI | réponse daemon brute (ex. `{found:false}` du stub J, ou erreur) |

### CONSTAT CENTRAL (vérifié au code)
**Aucun chemin de prod ne monte une session shard, pilote une génération
cross-shard, ou émet un RunProof in-vivo.** Câblé end-to-end aujourd'hui =
forward d'UNE frame layer-block avec admission (`ShardProtocol::accept`
`shard.rs:299-327` ; `ShardBackendForwarder::forward` `worker-core/.../shard.rs:535-556`).
Le « caller orchestrateur » que `open_shard_connection` documente
(`shard.rs:191-201`) **n'existe pas en prod**. La route session daemon
`live_shard_session()` renvoie `None` en dur (`http.rs:2138-2148`), store
HTTP-lisible « lands in Phase K » mais Phase K = wrap-up 0-code-net.

→ **L'acceptance T2 sur rig réel 5080↔Mac M2 n'est PAS exécutable end-to-end
aujourd'hui.** Le plus loin atteignable (matériel présent) = stage
`claim`/`frontier-forward` d'une frame montée à la main, jamais `toks_per_s ≥ 1`
génératif. Le plan §15.129 le confirme : « `b3_shard_pipeline.sh` (Phase K)
doit atteindre AU MINIMUM le stage `claim` » — aveu que le PASS génératif
complet n'est pas l'attendu réaliste.

### RISQUE — anti-faux-vert (impératif rédacteur harness)
Le harness DOIT exiger `run_proof` non-vide ET `toks_per_s ≥ 1` AVANT
d'appeler `pass()`, sinon BLOCK/RIG-ABSENT diagnostiqué. Le danger inverse :
un `pass()` atteint parce qu'un `found`/`result_text` parse « à vide » sur
le stub — interdit. Conformément §14.4 : BLOCK/RIG-ABSENT → feature shard
**PROVISIONAL + carry P1 vers S78**, `status` = champ JSON machine-lisible,
jamais `DIFFERE-materiel` en prose. **Statut T2 attendu honnête : RIG-ABSENT.**

---

## S2 — Carry closure (clos-en-S77 vs route-S78 + compteurs)

### CLOS en-S77 (avec preuve code) — NE PAS reconduire
| Carry | Preuve | Statut |
|---|---|---|
| **SYBIL-SEEDER-TAIL** | `placement.rs:312-317,344-349` (tri `(capacity desc, blake3(session_id‖pubkey) asc)`) + test `sybil_seeder_tail_sampling_is_deterministic_non_lexicographic` (`placement.rs:751-787`) | CLOSED Phase D (vérifié exact) |
| Phase A 3 P2 + C 6 branches + E SI-3-CHURN + F1→F2 D1-2 + G 9 P2 + H 1+4 + I 0 | reviews A-I | FERMÉS in-phase |
| CARRY-3 / LOOPBACK-TIERS / PULL-3 / duress-frères / P3-D-4 / UX-ARRIVAL | audit-plan §3 l.326-331 | déjà landés S76, ne pas rouvrir |

### CLÔTURABLE Phase K (action wrap-up)
| Item | Action | Condition |
|---|---|---|
| **Invariant clôture noms de tests** | re-check grep tout nom `verification.md §6` → fn `#[test]` | nouvel invariant audit S76, à exécuter |
| **F5 relabel THREAT §16** | l.918/1228/1376 « Phase J/K » → « Phase K » | confirmé par grep (cf. S3) |
| **F6 `/compute` lazy-load `@/components/`→`@/pages/`** (P12) | `App.tsx:86` ShardSessionPanel | optionnel, non bloquant |
| **RE-DRIVE-ON-INGEST** | cascade-closable SI acceptance T2 prouve convergence WAN ; **sinon 3/3 carry P1 honnête** | **re-check wrap-up OBLIGATOIRE** — comme T2 = RIG-ABSENT attendu, marquer 3/3 carry P1 |

### ROUTE-S78 (`sprint78_audit_plan.md`) — compteurs report
| Carry | report-counter | Note |
|---|---|---|
| **seeder `catalog_len:0`** | **2/3 → 3/3** ⚠️ ESCALADE | arbitrage PO design requis S78 (« pas reporté ») |
| **REVISION-HOME-DURABILITY** | **2/3 → 3/3** ⚠️ ESCALADE | MANDATORY S78 sauf exemption « blocker externe » re-justifiée |
| **KNOWN-ENTRY-OVERCOUNT** | **2/3 → 3/3** ⚠️ ESCALADE | exemption « dépendance séquentielle » à re-justifier |
| **RE-DRIVE-ON-INGEST** (si non clos) | **2/3 → 3/3** ⚠️ ESCALADE | fermable cascade SI T2 prouve WAN ; sinon MANDATORY S78 |
| **T-NN+3** (canonical_bytes dup JCS) | open S70 (< 3, non chiffré) | NON absorbé (correct, anti-band-aid) ; 5e/6e copie Phase C/I |
| **MEDIAN-DE-GROUPE / SANITY-BOUND-ASYMETRIQUE** | DOC-P2 (THREAT §15.3 DEFERRED) | `validate_quorum_pre_guardrail` ABSENT diff H (TENU non-absorbé) |
| **B10-PARITE-FIXTURE** | DOC-P2 | S77 ≠ bridge postMessage, reconduit |
| **OWN-DOC-FLOOR-L2L4** (S76 Track B) | P2 | hors-zone, reconduit |
| **DIRECTORY-EAGER-HAPPY-PATH** (S76 Track C) | P2 | hors-zone, reconduit |
| **SI-9 withholding** (Sev M) | carry OUVERT | mitigation conçue, câblage timeout/fallback = post-K |
| **Recalibration seuils bf16/TOPLOC** | P3 Phase-K-actionable | écart ≤1 ULP bf16 absorbé par seuils ; `phase_g_review.md:64-65` + THREAT:1238 |
| **SI-5 padding latence side-channel** | post-K | raffinement post-benchmark |
| **P3 biais modulo temp ~3e-7** (H) | non-actionable | non-sécuritaire, vit dans la proof Ed25519 |
| **Track Testabilité standing** | T1 spec + CI + artefact JSON T2 | standing audit_plan S78 |

**4 carries atteignent 3/3 en S78** → signal PO fort à inscrire dans
`sprint78_audit_plan.md` comme phases ou exemptions re-justifiées.

### À CONFIRMER au wrap-up
- **P3-D-3** (send-failure un-mark `seen.remove`) : kickoff dit « absorbé Phase B »
  mais `phase_b_review.md` ne le cite pas comme test (Phase B = ALPN data-plane,
  pas de chemin result-sync) → **test présent OU doc-note honnête**, à trancher.

---

## S3 — THREAT §16 finalisation (inventaire SI-x, version, F5 relabel, PATTERNS)

### Inventaire SI-1..SI-11 (§16 débute L980) — COMPLET
| ID | Ligne | Sev | Source |
|---|---|---|---|
| SI-1 Activation reconstruction | L998 | High (résiduel ASSUME) | base v10 |
| SI-2 Layer gradient leakage | L999 | N/A (inference-only) | base v10 |
| SI-3 Activation fingerprinting | L1000 | Medium | base v10 |
| SI-4 Collusion inter-workers | L1001 | High (résiduel ASSUME) | base v10 |
| SI-5 Latence side-channel | L1002 | Low | base v10 |
| SI-6 collusion-dans-tolerance | L1186 | M | N2 Phase I |
| SI-7 calibration seuil tolérance | L1191 | H/L | N2 Phase I |
| SI-8 grinding du commitment | L1223 | M | N3 Phase I |
| SI-9 refus de reveal / withholding | L1227 | M | N3 Phase I |
| SI-10 replay cross-session du commit | L1231 | (neutralisé) | N3 Phase I |
| SI-11 évasion lente / poison EMA | L1233 | M | N3 Phase I |

Sous-sections N0 L1036 / N1 L1086 / incentive L1140 (sev M, note anti-paresseux L1151) /
N2 L1156 / N3 L1200. Caveat confidentialité cardinal L1004-1014. **Contrat §14.2
strict (SI-1/3/4/5 + groupe privé + caveat + incentive M) = AUCUN manque.**

### Version + bump
Version courante = **v13** (L1362-1380). **Phase K DOIT bumper v13→v14** (nouveau
bloc `- **v14 (Sprint 77 Phase K, …)**` en §17). Le bloc « Completion Phase K »
(L1252-1256) liste : STRIDE §5.x formel + LINDDUN §6 + ligne §2 Assets + §4 DFD
pour le composant sharding + mitigation SI-5 padding constant-rate (du benchmark
réel, ou carry honnête si rig absent).

### F5 relabel — lignes RÉELLES (0 drift depuis review J)
| Ligne | Texte | Action |
|---|---|---|
| **L918** (row I §15.2) | `... = Phase J/K)` ×2 | « Phase J/K » → « Phase K » |
| **L1228** (SI-9) | `cablage timeout/fallback = Phase J/data-plane` | → « Phase K » |
| **L1376** (bloc v13) | `... = Phase J/K). Verdict ACCEPT/REJECT` | → « Phase K » |
| **L1045** (N0, NON citée mémoire) | `... data-plane de session (Phase H/I/J)` | candidate, à arbitrer (Phase J réelle = control-plane stub → périmé) |

NB blocs §17 historiques (L1331/1336/1345/1359/1375) = NE PAS réécrire.

### À ajouter Phase K (au-delà du relabel)
- **SI-9 carry rester ouvert** explicitement (review L155) — câblage timeout/fallback non livré.
- **Phrase route shard-session** : `grep "shard-session"` = 0 résultat → AJOUTER (idéalement §15.3 ou §16) : `GET /api/daemon/shard-session` = surface read-only loopback agrégat (miroir `seed_count`).
- **STRIDE §5.x + LINDDUN §6 + §2 Assets + §4 DFD** formels pour le composant sharding (point §17.3, annoncé Phase K).

### Cibles PATTERNS — prochain numéro libre
**`docs/rust/PATTERNS.md`** : dernier = §P66 (L3672). **Prochain libre = §P67.**
Déjà écrits S77 : §P62 (S76 wrap), §P63 (A keepalive), §P64 (G TOPLOC),
§P65 (H VRF/Token-DiFR), §P66 (I N2 clique/N3). **Gap net = 3 à ajouter ≥ §P67** :
- **ALPN shard** (`sbfb/shard/1` data-plane frame/admission/cap, B/F2) — absent.
- **scheduler Parallax** (water-filling VRAM + k-medoids PAM, D) — absent.
- **perf-map** (PerfMap raw-op non-signé micros entiers, E routing) — absent.

**`docs/shell/PATTERNS.md`** : dernier = P38 (L2274). **Prochain libre = P39.**
Aucun pattern S77 shell écrit. Candidat P39 : front `/compute` ShardSessionPanel
+ route daemon read-only `GET /api/daemon/shard-session` (whitelist
ShardSessionView, intentions FR, miroir seed_count) — à arbitrer P39 vs carry P3.

---

## S4 — Wire 0-bump (preuve)

**VERDICT scan : EXECUTE — invariant 0-bump wire CONFIRMÉ sur tout S77.**
Aucun DESIGN-CONFLICT.

- **Versions wire = 1** : 21 constants à leur valeur historique ; 3 net-new
  S77 nés à `1` (`SHARD_PLAN_FORMAT_VERSION` `shard_plan.rs:76`,
  `RUN_PROOF_FORMAT_VERSION` `shard_plan.rs:80`, `ACTIVATION_COMMIT_FORMAT_VERSION`
  `activation_commit.rs:69`). Seul `INVITE_FORMAT_VERSION=2` (`invite.rs:73`) ≠ 1
  = pré-S77 (S30), non-touché.
- **4 nouveaux `DOMAIN_*_V1` ADDITIFS** (canonical.rs, re-exportés lib.rs:88-93) :
  `DOMAIN_SHARD_PLAN_V1` (:276, C), `DOMAIN_RUN_PROOF_V1` (:290, C),
  `DOMAIN_VRF_DRAW_V1` (:310, H), `DOMAIN_ACTIVATION_COMMIT_V1` (:332, I).
  Tous `_V1`, pattern S74 `DOMAIN_SEED_REQUEST_V1`. Total repo = 25, tous `_V1`.
- **Aucun `_V2+`** : `grep DOMAIN_[A-Z_]+_V[2-9]` crates/ = No matches.
- **Preuve git** (range Phase C `ebe6779` → tip `66259c6`) : `*_FORMAT_VERSION`
  touchés = 3 en ligne `+` valeur `=1` ; `DOMAIN_*_V1` touchés = 4 en ligne `+` ;
  sanity adversarial `grep '^-pub const (DOMAIN_…|…FORMAT_VERSION|…ANNOUNCEMENT_VERSION|SCHEMA_VERSION)'`
  = **0 ligne de suppression/modification**.
- **Enveloppe `FeedEntry` raw-op INCHANGÉE** (`public_feed.rs:147-163`, `op:Value`
  l.151) ; enum `PublicFeedOperation` figé à 5 (`SeedAnnounced` S74) — le sharding
  ne passe PAS par le feed → 0 surface FeedEntry. Conforme Pre-launch policy.

→ **Row §16 #38 du plan (« 0 bump wire ») = PROUVÉE** par code + diff git complet.

---

## S5 — Baseline + invariant clôture (noms tests → fns réelles)

### Baseline anti-faux-vert
- **1949 Win nextest 0-skip = COHÉRENT.** Trajectoire grep-traçable :
  1828 (C) → 1852 → 1863 → 1883 → 1900 → 1916 → 1927 → 1947 → **1949 (J, tip `66259c6`)**.
- Le « ~2209 total » CLAUDE.md = baseline clôture **S76**, pas S77. Total S77
  actuel ≈ 1949 Rust Win + 411 Vitest + 7 factory-operator + 6 size-limit ≈ **2373**.
- Empreinte 12 fichiers net-new S77 = **130 attributs `#[test]`** (cohérent +121 Rust).

### Invariant clôture §16 (rows 14-37) — 23/24 résolvent à une vraie fn `#[test]`
Filtre nextest `test(<nom>)` = substring match (un nom-plan plus court résout).
Résolus : convergence_incremental, convergence_boot_catchup, shard_alpn_registered,
shard_frame_roundtrip, compute_group_signature, shard_handshake_rejects,
shard_plan_signature, run_proof_signature, placement_water_fills_vram,
placement_refuses_when_model_fits, kmedoids_groups_low_rtt,
placement_handles_5_workers_70b, sybil_seeder_tail, routing_dag_sweep,
churn_replaces_failed_server, perf_map_republished,
shard_backend_loads_layer_subset (#[ignore]+feature `llm_llama_cpp`),
toploc_detects_model_swap, n1_vrf_selects_deterministic_verifier,
n1_spot_check_randomizes, incentive_credits_reputation, n2_tolerant_quorum (×2),
n3_sentinel_localizes. (Row 36 = `git diff --stat validator.rs`, pas un test.)

### SEUL ÉCART — Row 30 `shard_backend_primitive` = PLACEHOLDER non-matérialisé
`grep -rn "shard_backend_primitive"` = **AUCUNE fn**. Glob a-priori
(`sprint77_plan.md:338,616` + `phase_f1_preflight.md:55`) jamais matérialisé.
`test(shard_backend_primitive)` → **0 test → faux-vert silencieux** si recopié
dans verification.md §6.

Les 4 fns `shard_backend_*` réelles (`worker-core/llm/shard.rs:706/725/762/805`)
sont TOUTES dans `mod gguf_tests` (`#[cfg(all(test, feature="llm_llama_cpp"))]`
l.682, chacune `#[ignore]`) → **NON hermétiques CI**.

**Tests hermétiques CI réels** couvrant la primitive shard (`mod tests`
`worker-core/llm/shard.rs:560-677`, NON-ignore NON-feature) :
`shard_window_validates_contiguous_range` (:564),
`shard_window_end_zero_means_n_layer` (:583), `shard_window_rejects_invalid` (:597),
`top_k_extracts_largest_by_magnitude_deterministically` (:619),
`top_k_clamps_k_and_handles_nan` (:631), `hidden_token_count_validates_shape` (:645),
`toploc_commitment_is_deterministic_and_swap_sensitive` (:658).

**ACTION verification.md §6 (row 30)** : remplacer le filtre par
`test(shard_window) + test(top_k) + test(hidden_token_count) + test(toploc_commitment)`
(ou citer `shard_window_validates_contiguous_range` comme représentant).
**NE PAS citer `shard_backend_primitive` ni `shard_backend_*`** (ce dernier
`#[ignore]`+feature = jamais-CI).

NB : `sprint77_verification.md` = **PAS ENCORE ÉCRIT** (à créer Phase K).

---

## Plan d'exécution Phase K (fichiers à écrire/éditer + statut T2 attendu)

Ordre recommandé (wrap-up, 0-code-fonctionnel-net hors harness shell) :

1. **`scripts/acceptance/b3_shard_pipeline.sh`** (NET-NEW) — copier le squelette
   `b3_live_pc_vps.sh` (SCRIPT_DIR, RIG_ENV+`MODEL_20GB`+`MAC_SSH`, emit_artifact
   python3+fallback, num(), 3 verdicts/3 exit codes, préflight=RIG-ABSENT,
   vps()/token loopback, auto-diagnostic frame-frontière). Gardes RIG-ABSENT
   shard : modèle ~20 Go ne tient sur aucune machine seule + 2e machine Metal
   joignable + PROJECT_ID reconcilié + stub session `None`. **Exiger `run_proof`
   non-vide ET `toks_per_s ≥ 1` AVANT `pass()`.** Harness palier 2 runnable
   (`REDUNDANCY`). Artefact JSON T2 = source de vérité.

2. **`scripts/acceptance/rig.local.env.example`** (NET-NEW si absent) — template
   gitignored documentant `VPS_SSH`/`MAC_SSH`/`PROJECT_ID`/`MODEL_20GB`/
   `WORKER_BIN`/`VPS_DAEMON`.

3. **`docs/security/THREAT_MODEL.md`** (ÉDIT) — bump v13→v14 (§17 nouveau bloc) ;
   relabel L918/L1228/L1376 « Phase J/K|J/data-plane » → « Phase K » (arbitrer
   L1045) ; SI-9 carry rester ouvert ; AJOUTER phrase route `/api/daemon/shard-session`
   (§15.3 ou §16) ; STRIDE §5.x + LINDDUN §6 + §2 Assets + §4 DFD sharding ;
   mitigation SI-5 padding (du benchmark OU carry honnête si rig absent).

4. **`docs/rust/PATTERNS.md`** (ÉDIT) — §P67 ALPN shard `sbfb/shard/1`, §P68
   scheduler Parallax (water-filling + k-medoids PAM), §P69 perf-map raw-op
   non-signé. (N0-N3/TOPLOC déjà §P64-66.)

5. **`docs/shell/PATTERNS.md`** (ÉDIT) — P39 front `/compute` ShardSessionPanel
   + route daemon read-only `GET /api/daemon/shard-session` (whitelist
   ShardSessionView, intentions FR, miroir seed_count) — OU laisser carry P3.

6. **`.planning/active/sprint77_verification.md`** (NET-NEW) — §6 invariant
   clôture : **row 30 corrigé** (`shard_window`/`top_k`/`hidden_token_count`/
   `toploc_commitment`, jamais `shard_backend_primitive`) ; baseline 1949 Win
   nextest + 411 Vitest ; fail-fast 3 blocs + release ; statut T2
   **PROVISIONAL + carry P1** (RIG-ABSENT honnête).

7. **`.planning/active/sprint78_audit_plan.md`** (NET-NEW) — router : 4 carries
   3/3 (seeder `catalog_len:0`, REVISION-HOME-DURABILITY, KNOWN-ENTRY-OVERCOUNT,
   RE-DRIVE-ON-INGEST si non clos) + T-NN+3 + MEDIAN-DE-GROUPE/SANITY-BOUND +
   B10-PARITE-FIXTURE + OWN-DOC-FLOOR-L2L4 + DIRECTORY-EAGER-HAPPY-PATH +
   SI-9 withholding + recalibration bf16/TOPLOC + SI-5 padding + **Track
   Testabilité standing** (T1 E2E hermétique BLOQUANT + CI chaque push +
   artefact JSON T2).

8. **`docs/claude/SPRINT_LOG.md`** (ÉDIT) — row S77 (Arc 3.5 6/6, sharding
   pipeline, 1949 Win/411 Vitest, T2 PROVISIONAL).

9. **`CLAUDE.md`** (ÉDIT) — état S77 DONE, S78 à ouvrir.

10. **`.planning/roadmap_v5_factory_complete_vision.md`** (ÉDIT) — S77 clos.

11. **Mémoire** : `nexus_grid_pivot.md` + `MEMORY.md` (post-commit, feedback_memory_update).

### Statut T2 attendu : **RIG-ABSENT** (honnête)
Aucun chemin prod ne peuple `ttft_s`/`toks_per_s`/`run_proof` ; route session
= stub `None` ; pas d'orchestrateur de session cross-shard. Avec matériel
2-machines : plafond atteignable = stage `claim`/`frontier-forward`, jamais
`toks_per_s ≥ 1`. **→ feature shard PROVISIONAL + carry P1 S78** (PRÉVU §14.4,
PAS un échec). `status` = champ JSON machine-lisible, jamais `DIFFERE-materiel`
en prose. RE-DRIVE-ON-INGEST en cascade → 3/3 carry P1 (pas de preuve WAN).

---

## Décisions Day-0 — non touchées (confirmation)

PLAN-ADAPT ne modifie AUCUNE décision figée :
- **0-bump wire** sur tout S77 = CONFIRMÉ (4 `DOMAIN_*_V1` additifs, 0 `_V2+`,
  enveloppe FeedEntry byte-stable) — invariant Pre-launch policy intact.
- **Pipeline-parallel exclusif**, ALPN `sbfb/shard/1`, vérif graduée N0-N4,
  groupe privé Ed25519, 1-3 tok/s WAN (`sharding_design_frozen`) — le kickoff
  exécute, ne re-conçoit pas.
- **Amendement PO 70B→20Go rig 5080+MacM2** + 2-machines (`sprint77_phase_f_fork_spike`,
  2026-06-21) — le harness cible 2 machines, pas 5.
- **Gate testabilité par-sprint** (README §4) : T1 E2E hermétique BLOQUANT +
  artefact JSON T2 {PASS/BLOCK{diag}/RIG-ABSENT}, jamais `DIFFERE-materiel` prose
  — Phase K l'honore via `b3_shard_pipeline.sh`.
- **PO ultra-complets** : T2 RIG-ABSENT n'est PAS un defer du cœur — le cœur
  sharding (primitives wire C, placement D, routing E, fork F, claim F2, N0-N3
  G-I, front J) est LIVRÉ ; seul le benchmark live cross-machine reste gated au
  matériel, ce qui est la nature même de T2 (carry P1 honnête).
- **kudos non-monétaire** (PO-12), **no-float coeur**, **named constants** —
  intacts.
