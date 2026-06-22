# Sprint 78 — Audit plan (audit gate de S77, joue en S78 Phase 0)

**Ecrit** : 2026-06-22 (Phase K Sprint 77).
**Sprint audite** : **Sprint 77** (sharding pipeline LLM — modele ~20 Go eclate
cross-machine, pipeline-parallel `sbfb/shard/1`, scheduler Parallax, verification
graduee N0-N3, fork llama.cpp layer-block, front `/compute`, harness + wrap-up).
**Executeur** : session fraiche S78, Phase 0 (Cas A audit gate).
**Produit attendu** : `.planning/active/sprint77_audit_findings.md`
(verdict PASS / CONDITIONAL PASS / FAIL).
**Tip audite** : commit Phase K `feat(daemon)` wrap-up (HEAD au demarrage S78 ;
tip code phases A-J = `66259c6`).

---

## §0 Mode d'emploi pour la session fraiche S78

**Ordre de lecture impose** (forme une opinion AVANT de lire les self-reports) :

1. Ce fichier (`sprint78_audit_plan.md`) — la feuille de route.
2. Le **diff complet** S77 : `git diff a1bbf00~1..66259c6` (du kickoff au tip
   Phase J) **plus le diff Phase K**. Commits feat/fix : kickoff `a1bbf00`,
   A `36cf1cc` (WAN task delivery convergence), B `81d667c` (shard data-plane
   ALPN + private compute group), C `ebe6779` (shard wire primitives + run
   proof), D `81c8f64` (Parallax placement water-filling + k-medoids), E
   `8ab8f97` (DAG routing + active churn), F1 `14fa313` (forked layer-block
   backend CUDA/Metal), F2 `a93d8bb` (shard claim + `sbfb/shard/1` forward),
   G `ce2f6a7` (N0 TOPLOC), H `fdc65a2` (N1 VRF + reputation incentive), I
   `99ba7b8` (N2 tolerant + N3 commit-reveal + SENTINEL), J `66259c6` (front
   shard-session panel + E2E hermetique), K `<tip>` (harness + verification +
   docs longue-vie + carry closure).
3. `sprint77_kickoff.md` (D1-D5 + scope cuts) + `sprint77_design_review.md` (G1)
   + `sprint77_phase_*_preflight.md` (les verdicts G8, dont **Phase K PLAN-ADAPT**).
4. Le code livre, dans l'ordre des tracks ci-dessous.

**A NE PAS lire avant d'avoir forme une opinion** : `sprint77_verification.md`
(self-report) et les `sprint77_phase_*_review.md` (reviews du livreur). Les lire
**apres** pour comparer.

**Decision figee a respecter** : invariant **heberger != publier, seeder !=
auteur** ; pipeline-parallel exclusif ; groupe prive Ed25519 ; kudos non-monetaire
(PO-12, jamais slash/stake/burn) ; no-float coeur ; 0 bump wire pre-launch ;
named constants. Day-0 NON re-debattues.

---

## §1 Track Suites (dual-platform, anti-faux-vert)

- Rejouer **Win nextest** (`cargo nextest run --workspace --locked`) + **Docker
  canonique** (rust:1.94 `sbfb-ci`, `CARGO_TARGET_DIR=target-linux`,
  **`SBFB_HOME` pointe sur un tmp inscriptible** — cf. §8 TEST-ISOLATION) +
  fmt/clippy/doctest/release + web (vitest/coverage/build/size/scan-en) + **E2E
  `compute-shard.spec.ts`**. Baseline annoncee Phase K : **Win 1949 / Docker 1953
  (+4 `#[cfg(unix)]`) / Vitest 411 / E2E 41+1skip**.
- **Anti-faux-vert** : le git-count des `#[test]` ajoutes par S77 doit egaler le
  delta annonce au §15 du plan (invariant audit S76). Verifier que les tests
  `#[ignore]`+feature `llm_llama_cpp` (GGUF F1) sont comptes au `nextest list`
  mais NON executes en CI, et documentes runnable localement.

## §2 Track Securite (THREAT_MODEL §16 + §5.9 + LINDDUN)

- §16 sharding : inventaire SI-1..SI-11 + incentive (sev M) + N0-N3 ; caveat
  confidentialite cardinal (activations en clair, pas de TEE GPU consumer 2026,
  groupe prive). Verifier que les mitigations citees sont CABLEES et testees
  (admission `is_member`, cap-VRAM fail-closed `is_degenerate_geometry`,
  cap-frame 256 MiB, claim crypto-avant-IO).
- §5.9 STRIDE sharding + ligne §6 LINDDUN shard-frontier + asset §2 A8 + note §4
  DFD : verifier coherence avec le code (pas de claim au-dela du livre).
- **Honnetete des forward-refs** : Phase K a re-cible « Phase J/K » → **S78** pour
  l'emission signee in-vivo, le transport sketch, l'arbitrage litige, SI-9
  timeout/fallback, SI-11 re-calibration, SI-5 padding. Confirmer qu'AUCUN de ces
  items n'est faux-declare « livre » dans le code.

## §3 Track Patterns (rust §P67-69 + shell P39)

- §P67 (data-plane `sbfb/shard/1` SEAM trait + admission crypto-avant-IO + boucle
  bi-stream sans orchestrateur + integrite hors-bande), §P68 (placement water-filling entier +
  k-medoids PAM deterministe + SYBIL-SEEDER-TAIL clos), §P69 (routing DP +
  churn actif borne + perf-map raw-op non-signe). Shell P39 (route read-only
  whitelist + stub None + intentions FR + DTO 2-champs).
- Verifier que chaque pattern grep-resout au code cite (pas de pattern orphelin).

## §4 Track Scope cuts (§17 plan, 14 items)

- Confirmer que les scope cuts ENTERRES restent enterres (N4 zkML, tensor-parallel
  2-GPU, streaming token-live, confidentialite-workers, KV-cache distribue,
  quant blockwise, VRAM-live admission, mode public, AWQ/GPTQ/EXL2). Pour chaque
  item « post-S77 » : verifier dans le code actuel s'il est devenu un vrai gap
  (principe sessions fraiches), ne PAS propager un scope cut comme verite technique.

## §5 Track Tests delta (invariant clôture)

- Tout nom de test cite dans `sprint77_verification.md §6` grep-resout a une fn
  `#[test]` reelle. **Specialement** : la row T1 cite des noms HERMETIQUES CI
  (`shard_window`/`top_k`/`hidden_token_count`/`toploc_commitment`), JAMAIS le
  placeholder `shard_backend_primitive` (= 0 fn, corrige Phase K — verifier qu'il
  ne revient pas) ni les `shard_backend_*` (`#[ignore]`+feature = jamais-CI).

## §6 Track Review files (coherence livreur)

- Les `sprint77_phase_*_review.md` PASS + Codex `*_codex_review.md` bruts
  (non-reecrits). Verifier que chaque P1 review a ete corrige in-phase et que les
  GAP Codex sont routes (META-1).

## §7 Track Carry-overs (la priorite — 4 escalades 3/3)

Tous les carries route ici DOIVENT etre traites en S78 (phase ou exemption
re-justifiee). **Les 4 marques ⚠️ atteignent 3/3 reports — signal PO fort.**

| Carry | report | Action S78 attendue |
|---|:---:|---|
| **seeder `catalog_len:0`** (l'annuaire d'un seeder n'annonce pas ce qu'il seede) | **3/3** ⚠️ | **arbitrage PO design requis** (« pas reporte ») — phase ou decision figee |
| **REVISION-HOME-DURABILITY** | **3/3** ⚠️ | MANDATORY sauf exemption « blocker externe » re-justifiee |
| **KNOWN-ENTRY-OVERCOUNT** | **3/3** ⚠️ | exemption « dependance sequentielle » a re-justifier ou fermer |
| **RE-DRIVE-ON-INGEST** (boot driver one-shot, fenetre morte 1er boot) | **3/3** ⚠️ | T2 RIG-ABSENT n'a pas prouve la convergence WAN → MANDATORY S78 |
| **SHARD-PROVISIONAL** (orchestrateur de session in-vivo + benchmark live cross-machine) | **P1 nouveau** | **le gros carry** : aucun chemin prod ne monte/pilote une generation cross-shard ni n'emet de `RunProof` in-vivo ; route `/shard-session` = stub None. Le coeur (placement/routing/N0-N3/data-plane/fork/claim) est LIVRE+teste ; il manque le DRIVER de session + le benchmark T2. Cf. §10 |
| **SI-9 withholding** (Sev M) | ouvert | cablage timeout/fallback (re-route shard `fallback_node`) |
| **SI-11 / recalibration bf16/TOPLOC** | P3 | fence IQR adaptative + signal magnitude absolue |
| **SI-5 padding latence side-channel** | post-K | derive du benchmark REEL (gated T2) |
| **TEST-ISOLATION-SBFB-HOME** (nouveau, §8) | P2 | hermeticiser les e2e daemon-spawn (SBFB_HOME par TempDir) |
| **T-NN+3** (factorisation JCS sign/verify) | open S70 (<3) | 5e/6e copie ; non-absorbe (anti-band-aid correct) |
| **MEDIAN-DE-GROUPE / SANITY-BOUND-ASYMETRIQUE** | DOC-P2 | THREAT §15.3 DEFERRED, non-absorbe (tenu) |
| **B10-PARITE-FIXTURE** | DOC-P2 | hors-zone S77 (bridge postMessage), reconduit |
| **OWN-DOC-FLOOR-L2L4** (S76 Track B) | P2 | hors-zone, reconduit |
| **DIRECTORY-EAGER-HAPPY-PATH** (S76 Track C) | P2 | hors-zone, reconduit |
| **P3-D-3** (send-failure un-mark `seen.remove`) | a confirmer | test present OU doc-note honnete (S77 ≠ result-sync) |

## §8 Track HARDENING + meta-process

- **TEST-ISOLATION-SBFB-HOME (P2, decouvert Phase K)** : les e2e daemon-spawn
  (`crates/nexus-shell-daemon/tests/e2e.rs`, ex.
  `start_writes_running_json_and_responds_to_health`) fixent `NEXUS_GRID_ROOT`
  sur un TempDir mais PAS `SBFB_HOME` → ils partagent `$HOME/.sbfb` global. Dans
  un conteneur Docker frais lance en parallele (nextest), une course sur
  `/root/.sbfb/auth_token` fait echouer un des spawns (« No such file or
  directory »). Le vrai CI + le dev ont `~/.sbfb` → masque. **Pre-existant
  (0 delta Rust S77), pas une regression** ; root-cause = ajouter
  `.env("SBFB_HOME", tmp)` aux e2e daemon-spawn. Le harness Phase K documente
  l'exigence (`SBFB_HOME` au Docker canonique) comme contournement honnete.
- **STALE-PHASE-K-COMMENTS (P3, decouvert Phase K)** : des commentaires source
  ecrits en Phase J anticipaient le store de session live « lands in Phase K »
  (`crates/nexus-shell-daemon/src/http.rs:2100/2142/2151/2173/2175/5242/5322`,
  `web/src/api/daemon.ts:542`, `web/src/components/ShardSessionPanel.tsx:11/136/181`,
  `web/e2e/compute-shard.spec.ts:11`).
  Phase K = wrap-up 0-code → le store atterrit en realite S78. Les commentaires
  sont des forward-refs stale ; le scrub (commentaires seuls, 0 comportement) est
  differe S78 pour preserver la frontiere wrap-up 0-code-net (la doc THREAT/PATTERNS
  est deja re-ciblee S78 ; seuls les commentaires code restent). Cf. shell P39.
- **Externes inchanges** : P2-A-1 `rand` (exemption), P2-AUDIT-2 iroh (pin 0.98),
  T-NN+2 iframe Rust-wasm (§P34), P3-OS-1. LT-2 Radicle ARME (flip publie = PO).
- **Meta-process** : verifier que toutes les phases A-K ont preflight (Workflow) +
  review (Workflow) PASS + Codex brut + commit atomique, et que Phase K
  (PLAN-ADAPT) a suivi l'approche corrigee (T2 RIG-ABSENT honnete, pas faux PASS).

## §9 Track Testabilite (STANDING — gate par-sprint, README §4)

**Permanent a chaque audit gate** :
1. **T1** : `web/e2e/compute-shard.spec.ts` existe, est BLOQUANT au wrap-up + CI
   chaque push, et est GREEN (41+1skip@shard observe Phase K).
2. **T2** : `scripts/acceptance/b3_shard_pipeline.sh` produit un artefact JSON
   machine-lisible `{status ∈ PASS/BLOCK{diag}/RIG-ABSENT}`, jamais
   `DIFFERE-materiel` en prose. Phase K = **RIG-ABSENT** (orchestrateur + rig
   absents) → feature shard **PROVISIONAL + carry P1** (cf. §7 SHARD-PROVISIONAL).
3. **S78 doit verifier** : si l'orchestrateur de session atterrit en S78, le
   harness re-tourne et DOIT atteindre PASS (`run_proof` non-vide + `toks_per_s
   >= 1`) ou BLOCK diagnostique — la feature ne sort de PROVISIONAL qu'avec un
   `b3 PASS` + test de convergence cross-machine vert.

## §10 Pre-requis S78 (le sprint suivant naturel)

Le gate produit S77 (benchmark live) est **RIG-ABSENT structurel** : le coeur
sharding est livre, mais l'**orchestrateur de session** qui (a) monte une session
shard, (b) pilote une generation token-par-token cross-shard, (c) mesure
TTFT/tok-s, (d) emet un `RunProof` signe in-vivo, et (e) peuple le store HTTP
lisible (`live_shard_session` n'est plus un stub), n'existe pas. C'est le coeur
naturel de S78 — apres quoi `b3_shard_pipeline.sh` devient executable end-to-end
sur le rig 5080+Mac M2 et la feature shard sort de PROVISIONAL.

---

> Colonne report-counter : un carry a 3/3 est une ESCALADE (3 sprints porte sans
> resolution) — l'audit gate S78 le traite en phase dediee ou en exemption
> re-justifiee, JAMAIS en simple reconduction silencieuse (regle G7 META-1).
