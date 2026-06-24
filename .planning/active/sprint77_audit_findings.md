# Sprint 77 — Audit findings (audit gate de S77, joue en Phase 0 du slot suivant)

Pattern Sprint 6/7. Verdict d'entree avant ouverture du sprint Factory (S79).

---

## 1. Auditeur

- **Session** : session fraiche, Cas A audit gate, joue `sprint78_audit_plan.md`
  (qui cible le diff S77 → produit ce fichier).
- **Methode** : orchestration **Workflow ultracode multi-agents anti-anchoring**
  (`wf_6bb553d6-b02` ; **12 agents, ~1.26M tokens, 347 tool-uses, ~10,7 min**).
  Fan-out de **11 tracks** (1 agent independant par track, lecture STATIQUE
  no-compile) : core-algo / security / patterns / scope-cuts / test-delta /
  review-files / **carry-overs** / hardening-meta / testability / vendor-fork /
  doc-layer. Puis **verification adversariale** des candidats P0/P1 (skeptic
  mandate de REFUTER), puis **critique de completude**. Opinion formee depuis le
  **code livre + `git diff`/`log`** (plage `a1bbf00~1..a795700`) AVANT toute
  lecture des self-reports (`sprint77_verification.md`,
  `sprint77_phase_*_review.md`, `*_codex_review.md`), conformement au §0 du plan
  d'audit (ordre de lecture impose, README §9.6).
- **Suite autoritaire** : re-jouee EN PARALLELE par l'orchestrateur (main thread),
  hors des agents statiques. Resultats reels ci-dessous (§4 Track Suites) — pas
  une relecture du self-report.
- **Adjudication adversariale du main thread** : les 6 gaps du critique de
  completude ont ete arbitres faits-en-main par l'orchestrateur (cf. §9). Le plus
  net : le critique a pretendu « vendor fork sans pin upstream » — **REFUTE**, la
  provenance est complete (`THIRD-PARTY-NOTICES.md` + 2× `.cargo_vcs_info.json`
  sha1 identiques + `patches/`). Verification des faits des agents = directive PO.

---

## 2. Tip audite

- **Base** : `a1bbf00~1` = `d9c93ff` (S76 audit findings = dernier commit pre-S77).
- **Tip code S77** : `a795700` (Phase N, dernier livrable S77).
- **HEAD au demarrage** : `19e1827` (chore planning : arbitrage PO Factory-first +
  handoff) — **planning-only, 0 code** ; `c367047` (directive PO S79 + research)
  idem. Le diff code auditable s'arrete a `a795700`.
- **Plage d'audit** : `a1bbf00~1..a795700` (14 phases A-N + chores intercales).
- **Commits feat/fix** : kickoff `a1bbf00`, A `36cf1cc`, B `81d667c`, C `ebe6779`,
  D `81c8f64`, E `8ab8f97`, F1 `14fa313`, F2 `a93d8bb`, G `ce2f6a7`, H `fdc65a2`,
  I `99ba7b8`, J `66259c6`, K `0f597cf`, **avenant** L `744f84a` (feat core),
  M `91be0e4` (docs), N `a795700` (docs). chore(worker) `6e07182` (gate
  shard_node example). Chores planning intercales (research, handoffs) non-code.
- **Ampleur** : 1091 fichiers / ~532K insertions, **dont 913 sous `vendor/`**
  (fork llama.cpp Phase F1, audite en provenance/patch-scope, pas ligne-a-ligne).
  Surface code reelle ≈ 40 crates + 7 web + 5 scripts + 14 docs.

---

## 3. Verdict global

# PASS

- **0 P0, 0 P1 (defaut neuf).**
- **5 P2** + **5 P3** documentes (rigueur G4 satisfaite : PASS exige ≥1 P2+).
- **SHARD-PROVISIONAL** = carry P1 **pre-declare** (S77 Phase K), **honnetement
  tracke** dans le code (data-plane `sbfb/shard/1` enregistre uniquement en
  `#[cfg(test)]`, `RunProof::sign` uniquement `#[cfg(test)]`, route
  `live_shard_session` = stub `None`, `b3_shard_pipeline.sh` = RIG-ABSENT JSON,
  honesty-gate CI armes sur les 2 pipelines). **Ce n'est pas un defaut bloquant :
  c'est la frontiere honnete du sprint, deja routee S78** (README §4 : une feature
  cross-machine sans convergence verte + `b3 PASS` reste PROVISIONAL + carry P1).

**Rationale** : le scope LIVRE (cœur sharding C-K : primitives wire, placement
Parallax, routing DAG + churn, verification N0-N3, data-plane, fork layer-block,
claim ; + avenant doc L-N) est **correct, deterministe, integer-exact, bien teste,
honnetement documente**. Le scope NON-LIVRE (orchestrateur de session in-vivo +
benchmark cross-machine) est **honnetement carry S78** avec marqueurs PROVISIONAL
mecaniquement enforces. Aligne sur le precedent **S74 (PASS propre)**, pas
S75/S76 (CONDITIONAL PASS qui exigeaient un fix duress pour clore).

**Pas de commit `fix(sprint77)` requis** (0 P0/P1). Les P2 sont logges en tech-debt
(PATTERNS.md) ; les carries sont routes S78.

---

## 4. Tracks (verdict + findings)

### Track Suites (dual-platform, autoritaire — main thread)
**Verdict : PASS.**
- **Rust Win nextest `--workspace --locked`** : **1956 / 1956 / 0-skip**. fmt OK,
  clippy `--all-targets -D warnings` OK, doctests OK, release build OK.
- **Web** : Vitest **411 passed** (38 files) ; coverage **87.27 / 79.01 / 86.02 /
  88.59** (≥ seuils 85/85/78/85) ; tsc OK ; build OK ; size OK (css 129.02/130 kB) ;
  scan-en clean ; lint 5 warnings (0 errors, react-refresh, pre-existant).
- **T1 E2E `compute-shard.spec.ts`** (hermetique, daemon reel pinne) : **2 passed +
  1 `@shard` skipped** (RIG-ABSENT correctement skip).
- **Docker canonique** (`sbfb-ci` rust:1.94, `target-linux`, `SBFB_HOME`=tmp) :
  **1820 passed / 6 failed / 134 not-run** (fail-fast). Les 6 echecs = le jeu
  **iroh-networked cross-daemon documente** (`test_two_daemons_boot_and_respond`,
  `test_cross_daemon_discovery`, `test_cross_daemon_blob_transfer`,
  `test_cross_daemon_task_stub`, `cross_daemon_publish_and_serve_blob`,
  `blob_serve_coep_headers_on_real_zip`), root-cause `timeout waiting for
  auth_token` = **env-block Docker-on-Windows** (reseau hote degrade `create_node`
  + carry TEST-ISOLATION-SBFB-HOME §8). **Pas une regression S77** (ces 6 tests
  sont des tests daemon-boot pre-sharding, 0 delta S77) ; verts sur Win natif
  (1956/1956) + le CI Linux Woodpecker/GHA.
- **Reconciliation** : `verification.md §6` annonce 1949 (snapshot Phase K) ; tip
  reel 1956 (**+7 = Phase L** : 5 tests schema `schemas/shard.rs` + 2
  `shard_sign_verify` liftes VERBATIM via `include!`). git-count `#[test]` ajoutes
  S77 = **+149 / -0** (145 CI-run + 4 GGUF `#[ignore]`+feature). **Pas de
  faux-vert** ; l'ecart 1949→1956 est un UNDERcount conservateur, pas une
  inflation.

### Track core-algo (cœur sharding C/D/E/G/H/I) — CLEAN (0 finding)
Placement : water-filling largest-remainder integer borne a la VRAM par worker,
somme exacte = total_layers ; k-medoids PAM BUILD+SWAP deterministe (0 random,
tie-break pubkey, `KMEDOITS_MAX_ITER=64`) ; saturating arithmetic ; gates
fail-closed (candidats vides / 0 layer / 0 VRAM / couverture VRAM agregee) +
`covers_full_model` + `MIN_SHARD_WORKERS`. Routing : DP topo-order single-pass
shortest-path sur DAG, tie-break (cost,pubkey), saturating ; churn O(R) borne (pas
de re-route loop). N0-N3 : TOPLOC pre-image hashe all-integer (bf16 = seule
frontiere float, juste avant encodage) seuils strict-< compares integer-only ; VRF
seuil u128 sans overflow ; N2 = **clique maximum** (pas pivot-count) sur graphe
d'accord bidirectionnel symetrique (gere la non-transitivite de l'accord tolerant —
un straddler ne gonfle pas le quorum) ; N3 commit-reveal binding-first (BLAKE3
sketch||nonce) non-grindable, session_id + frontier_index lies au pre-image signe
(anti-replay) ; SENTINEL EMA basis-point integer avec rejet d'outlier. **no-float
tenu sur tous les chemins consensus** ; magic numbers = consts nommees.

### Track security (THREAT_MODEL §16 + §5.9 + LINDDUN) — CLEAN (0 finding)
Toutes les mitigations citees PRESENTES sont cablees+testees : admission
`is_member` Ed25519 sur `remote_id()` AVANT `accept_bi` (shard.rs:303-308) ET avant
I/O sur le claim (shard_claim.rs:276-294) ; **crypto-before-IO reel** (signature→
membership→in-plan pure no-IO, puis GGUF header + 1 snapshot GPU) ;
`MAX_SHARD_FRAME_BYTES=256MiB` rejete a la lecture avant alloc ; fail-closed VRAM
`is_degenerate_geometry`+`assess_capacity` ; N2 vote uniquement sur RunProofs
signes dont le sketch ouvre le commitment N0 signe. **PO-12 non-monetaire HONORE**
(0 slash/stake/burn ; seules occurrences = commentaires d'interdiction).
**Forward-refs honnetes** : SI-9/SI-5/SI-11, dispute arbitration, emission signee
in-vivo, transport sketch = carry S78, **jamais faux-declares livres**. Caveat
cardinal confidentialite (activations en clair, pas de TEE GPU consumer 2026,
groupe prive = admission≠confidentialite, SI-1/SI-4 residuel High) honnete.

### Track patterns (rust §P67-69 + shell P39) — CONCERN → **PAT-1 (P2)**
Les 4 patterns grep-resolvent au code reel. §P67/§P68/P39 fideles (dont disclaimer
explicite « orchestrateur n'existe pas encore (S78 carry) » §P67 point 3).
**PAT-1 (P2)** : §P69 narre au present que la glue perf-map republish/ingest
« `doc.set` lives in the daemon » / « the daemon owns the doc handle » alors que ce
cablage **n'existe nulle part** (0 caller daemon de `routing.rs`/`PerfMap`). Pas de
faux-vert (le test `perf_map_republished_to_doc` se disclaime in-body, et l'etat
non-cable est inferable du contexte SHARD-PROVISIONAL) — mais **manque le
disclaimer explicite « not-yet-wired (S78) »** que §P67 applique correctement.
Recommandation : aligner §P69 sur §P67 (marquer la glue daemon-side comme seam S78).
0 code change.

### Track scope-cuts (plan §17, 14 items) — CLEAN (0 finding)
Les 14 cuts restent enterres, documentes honnetement, 0 code half-shipped.
**Pipeline-parallel EXCLUSIF** verifie cote worker (`with_shard_range` +
`with_n_gpu_layers` ; APIs tensor-split `with_split_mode`/`with_devices`
test-asserted ABSENTES, `quantization_doc.rs:103`) ET cote fork
(`patches/llama-cpp-shard.patch` ajoute seulement la fenetre pipeline, 0
all-reduce/NCCL). **mode-public reste cut** (`compute_group.rs` = allowlist privee
Ed25519). `KvCachePolicy` enum CLOSED gelee `LocalEphemeral`. Aucun cut « differe »
n'est silencieusement load-bearing (VRAM-live admission cut comme pompe continue,
mais le claim-gate lit toujours un snapshot point-in-time fail-closed).

### Track test-delta (anti-faux-vert) — CLEAN (0 finding)
Les 27 noms de tests de `verification.md §6` grep-resolvent tous a une fn reelle.
Placeholder `shard_backend_primitive` (=0 fn) **ABSENT** (fix Phase K tenu). Les 4
T1 hermetiques (`shard_window_validates_contiguous_range`,
`top_k_extracts_largest_by_magnitude_deterministically`,
`hidden_token_count_validates_shape`, `toploc_commitment_is_deterministic_and_swap_sensitive`)
= plain `#[test]` CI-run. Les 4 `shard_backend_*` = `#[ignore]`+feature
`llm_llama_cpp` (jamais-CI, documentes runnable local). Acceptance « 41+1skip »
reconcilie (43 test() web/e2e − 1 @compute grep-invert − 1 @shard test.skip).

### Track review-files (coherence livreur) — CLEAN (0 finding)
Les 14 phases A-N portent un review PASS + un `codex_review.md` brut (format
CONFIRME/PARTIEL/GAP file:line, pas prose Claude). Chaque P1 review corrige
in-phase (verifie code pour les cas porteurs : F1 middle-shard NULL-deref test,
E SEC-SI3 rho-only, K SI-5 relabel S78, N faux « 3-method » entierement retire).
GAPs Codex routes/clos (META-1) ; Phase N 4 rounds (round1/2/3 bruts conserves +
round4 CLEAN). **Incident honnetete Phase N** (flip hors-bande llms.txt →
« shipped ») documente + **CI-locke** (`check-sharding-docs.sh:225`). Note (pas
defaut) : reviews A/B honnetement transcrites inline (B = panne Anthropic 529) ;
independance portee par le gate Codex externe.

### Track carry-overs (audit_plan §7 — la priorite) — CONCERN → **CARRY-1 (P2)**
Les 4 escalades ⚠️ 3/3 + le P1 SHARD-PROVISIONAL sont **REELS dans le code et
HONNETEMENT trackes** — 0 fausse cloture. `live_shard_session` = stub `None`
(http.rs:2115) ; tous les `RunProof::new`/`sign` en `#[cfg(test)]`. seeder
`catalog_len:0` toujours present (annuaire bati sur `own_entries`).
RE-DRIVE-ON-INGEST = boot driver one-shot avec dead-window 1er boot documentee
(http.rs:1772-1777). REVISION-HOME-DURABILITY + KNOWN-ENTRY-OVERCOUNT = carries
exemption honnetes. **CARRY-1 (P2)** : §P68/audit_plan §3 declarent
« SYBIL-SEEDER-TAIL clos », mais la fermeture (sampling blake3 anti-crowding) a ete
appliquee au **tail WORKER-placement** (`placement.rs:309-350`), PAS au **dial-set
SEEDER** que le carry visait a l'origine (`seeders_recent` fait toujours
`ids.sort()` lexicographique ; http.rs:1709-1714 dit lui-meme « carried to the S76
audit »). Residuel **availability-only** (BLAKE3 = integrite preservee), pas
P0/P1 — mais **carry credite clos pendant que le chemin nomme est inchange et
absent de la table §7** (risque silent-drop G7/META-1). Recommandation : tracker
SEEDER-DIAL-TAIL explicitement S78 OU re-scoper §P68 (ferme le tail worker
seulement).

### Track hardening-meta (audit_plan §8) — CONCERN → **HARD-1/2 (P2), HARD-3/4 (P3)**
Meta-process propre : chaque phase A-N a preflight + review PASS + Codex brut +
commit atomique ; Phase K = vrai PLAN-ADAPT avec T2 RIG-ABSENT HONNETE (`pass()`
structurellement inatteignable sans RunProof signe in-vivo — confirme : tous les
call-sites `RunProof::sign` sont `#[cfg(test)]`, donc pas de faux PASS possible) ;
avenant L/M/N = reopen legitimement PO-documente (plan §20) ; Phase M docs-only ;
Phase L = feat(core) correctement scope (DTO http.rs = MOVE byte-identique).
- **HARD-1 (P2)** : `scripts/verify.sh` STALE — steps 4-8 invoquent un toolchain
  Python `packages/` supprime (0 fichier tracke) → abort step 4 sous `set -euo
  pipefail` sur un checkout frais. PAS sur le chemin du gate (ci.yml est la version
  propre), mais script mort que **S77 Phase M a touche** (ajout step 19) sans
  nettoyer. Fix : retirer steps 4-8 + preambule venv.
- **HARD-2 (P2)** : TEST-ISOLATION-SBFB-HOME confirme **pre-existant** (e2e.rs
  dernier touche S10, 0 delta S77 ; fixe `NEXUS_GRID_ROOT` mais jamais
  `SBFB_HOME`). Root-cause = ajouter `.env("SBFB_HOME", tmp)` aux e2e
  daemon-spawn. (C'est aussi la cause des 6 fails Docker §Suites.)
- **HARD-3 (P3)** : STALE-PHASE-K-COMMENTS confirmes cosmetiques (forward-refs
  « lands in Phase K » alors que le store atterrit S78 ; comportement honnete
  `None`/200 empty). Scrub differe S78.
- **HARD-4 (P3)** : Phase N (`docs(sharding):`) inclut `tests/shard_sign_verify.rs`
  (+19 l., `include!` de l'exemple doc = drift-gate). Borderline mais defendable
  (test-only doc-integrite, 0 comportement prod). Note hygiene README §9.5.

### Track testability (audit_plan §9 — STANDING) — CLEAN → **TEST-1 (P2), TEST-2 (P3)**
T1 + T2 honorent le gate README §4. T1 : `compute-shard.spec.ts` (2 hermetiques +
1 `@shard` skip par `test.skip`, **pas** grep-invert → couverture jamais
silencieusement perdue), wire BLOQUANT GHA ci.yml `[10c]` chaque push, GREEN. T2 :
`b3_shard_pipeline.sh` emet le contrat JSON `{PASS/BLOCK{diag}/RIG-ABSENT}` a
chaque sortie (python3 + fallback bash pur), jamais `DIFFERE-materiel`. RIG-ABSENT
structurellement honnete + machine-emis. honesty-gate `check-sharding-docs.sh`
arme sur **les 2 CI** (GHA + Woodpecker).
- **TEST-1 (P2)** : l'E2E hermetique tourne **seulement sur GHA**, pas dans le
  miroir d'independance Woodpecker `.woodpecker/ci-linux.yml` (qui porte
  fmt/clippy/test/build/sharding-docs mais 0 Playwright). T1 satisfait (bloquant
  GHA chaque push), mais asymetrie : si GHA disparait, le gate E2E s'evapore. Fix :
  ajouter un step Playwright a Woodpecker.
- **TEST-2 (P3)** : `verification.md` row 39 attribue le total suite (41) au seul
  fichier `compute-shard.spec.ts` (qui a 2+1skip). Imprecision de label, pas de
  faux-vert. Reformuler.

### Track vendor-fork (Phase F1, 913 fichiers) — CLEAN → **VENDOR-1 (P3)**
Provenance forte : `THIRD-PARTY-NOTICES.md` enregistre les 2 composants, licences
MIT / MIT-OR-Apache-2.0 (AGPL-compat), URLs upstream, **pin upstream
`4afdaf0782ef7f3254a186a7ff67a1c7491c6dce`** (= 2× `.cargo_vcs_info.json`) ; delta
SBFB dans `patches/llama-cpp-shard.patch` (236 l.) + miroir in-tree marque
`SBFB S77 fork`. Feature gate complet (`default=[]` ; `llm_llama_cpp` opt-in ;
backend `#[cfg(feature)]` ; vendor crates NON membres du workspace, tires via
`[patch.crates-io]` ; exemple `shard_node.rs required-features` chore `6e07182`).
**Le fork 913-fichiers ne compile JAMAIS en CI/build par defaut.** `.gitattributes`
`vendor/** linguist-vendored`. Supply-chain propre (build.rs `LLAMA_CURL OFF`, 0
fetch reseau, seul blob = fixture vocab upstream 627KB) ; `check-spdx.sh` ne scanne
pas vendor/ (MIT vendored non false-flagge).
- **VENDOR-1 (P3)** : label `THROWAWAY` perime sur des champs
  `llama_context_params` shard committed-et-utilises (llama.h:390 ; le bloc
  jumeau model_params:323 utilise le label propre `// SBFB S77 fork:`). Cosmetique,
  pourrait induire un mainteneur a croire du code mort. Relabel.

### Track doc-layer (avenant L/M/N + honesty gate) — CLEAN → **DOC-1 (P3)**
Avenant honnete et bien ancre. Schemas Phase L **generes** (drift-canary
`shard_schema_snapshot_matches_struct` compare chaque `*.schema.json` a
`schema_for!(T)` ; `spec_consts_exist` lie chaque DOMAIN tag + cap a la const Rust
= build-break si rename) ; round-trip `include!` reel
(`tests/shard_sign_verify.rs` include `examples/sign_verify.rs` = 2 `#[test]`
nextest). Gate `check-sharding-docs.sh` wire CI **sans bypass** (verifie : strip
PROVISIONAL → EXIT 1 + restore → byte-identique). WIRING_SPEC : 51 source_refs
rank-1, 11 REQUIRED_ANCHORS resolvent (dont `accept_bi` flag Codex round-3).
**PO#4** : `BridgeMethodSchema` a exactement **16 methodes** ; bridge_gap.md +
WIRING_SPEC + llms.txt disent tous 16 (le faux « 3-method » P1 review corrige dans
les docs sharding). Banner cardinal correct (orchestrateur PROVISIONAL / carry S78,
jamais « shipped »).
- **DOC-1 (P3)** : le faux « 3-method bridge » survit dans **4 docs HORS avenant**
  (LAUNCHER.md:252/392, ATTACK_SCENARIOS.md:283, EXTERNAL_AUDIT_SCOPE.md:104 ;
  THREAT_MODEL.md:172 = ligne STRIDE pre-existante, pas le §16 S77). **Pas un
  defaut de l'avenant** (interne honnete) ; dette doc repo-wide. Balayer vers 16.

---

## 5. Findings — recap trie par severite

| ID | Sev | Track | Titre | Disposition |
|---|---|---|---|---|
| — | **P0** | — | (aucun) | — |
| — | **P1** | — | (aucun defaut neuf) | — |
| SHARD-PROVISIONAL | P1-carry | carry | data-plane/orchestrateur in-vivo absent (stub None, ALPN test-only) | **carry S78** (deja routee audit_plan §7/§10) — honnete, non-bloquant |
| PAT-1 | P2 | patterns | §P69 surdeclare la glue perf-map daemon comme livree | PATTERNS rust + S78 |
| CARRY-1 | P2 | carry | SYBIL-SEEDER-TAIL credite clos mais seul le tail worker corrige | PATTERNS rust + S78 (SEEDER-DIAL-TAIL) |
| HARD-1 | P2 | hardening | `verify.sh` stale (steps Python supprimes) | PATTERNS shell + S78 |
| HARD-2 | P2 | hardening | TEST-ISOLATION-SBFB-HOME (pre-existant, e2e sans SBFB_HOME) | PATTERNS shell + S78 (root-cause `.env`) |
| TEST-1 | P2 | testability | E2E hermetique GHA-only, absent de Woodpecker | PATTERNS shell + S78 |
| HARD-3 | P3 | hardening | commentaires forward-ref « Phase K » stale | optionnel (scrub S78) |
| HARD-4 | P3 | hardening | `.rs` drift-gate dans un commit `docs(sharding)` | optionnel (note convention) |
| TEST-2 | P3 | testability | `verification.md` attribue 41 au seul fichier shard | optionnel (reformuler) |
| VENDOR-1 | P3 | vendor | label `THROWAWAY` sur champs fork utilises | optionnel (relabel) |
| DOC-1 | P3 | doc | « 3-method bridge » stale dans 4 docs hors avenant | optionnel (balayage repo-wide vers 16) |

---

## 6. Commits fix attendus

**AUCUN.** 0 P0, 0 P1 (defaut neuf). Le gate ferme sans commit `fix(sprint77)`.
SHARD-PROVISIONAL est un carry pre-declare (non un defaut a corriger pour clore) :
sa resolution = le cœur naturel de S78 (orchestrateur de session in-vivo +
benchmark live), cf. audit_plan §10.

---

## 7. P2 a logger en tech debt (PATTERNS.md)

- **rust/PATTERNS.md** : PAT-1 (§P69 disclaimer not-yet-wired), CARRY-1
  (SEEDER-DIAL-TAIL residuel availability-only BLAKE3-borne), HARD-2
  (TEST-ISOLATION-SBFB-HOME root-cause `.env("SBFB_HOME", tmp)`).
- **shell/PATTERNS.md** : HARD-1 (`verify.sh` steps 4-8 morts), TEST-1 (E2E hors
  Woodpecker).

Tous reconduits dans le ledger S78 (le sprint suivant les route en phase ou
exemption re-justifiee, jamais silencieusement).

---

## 8. P3 laisses sans action

HARD-3 (scrub commentaires Phase K → S78 quand le store atterrit), HARD-4 (note
convention include!-doc-test acceptable en `docs()`), TEST-2 (reformuler
verification row 39), VENDOR-1 (relabel `THROWAWAY` → `SBFB S77 fork:`), DOC-1
(balayage repo-wide « 3-method » → 16). Tous non-bloquants, traçables.

---

## 9. Notes sur la completude de l'audit

**Critique de completude (6 gaps) — adjudication adversariale du main thread :**

- **G1 — data-plane jamais monte en prod** : **CONFIRME**. `SHARD_ALPN` /
  `shard_protocol_factory` referencees seulement en `#[cfg(test)]` (shard.rs:439) ;
  le boot prod (runtime.rs:358/381, seed_protocol.rs:417, http.rs:5976) ne monte
  **que `SEED_ALPN`**. Le data-plane `sbfb/shard/1` n'est monte dans AUCUN nœud
  en exécution. **Pas un defaut neuf** — c'est l'etat attendu d'une feature
  PROVISIONAL dont le montage = carry S78. **Renforce le verdict** : le gate ne
  doit PAS se lire « feature live mais non-benchmarkee » ; elle est **non-montee**
  en prod, honnetement.
- **G2 — backend GPU layer-block jamais compile en CI** : **CONFIRME mais
  by-design**. `default=[]` ; les 1956 verts prouvent le control-plane + les algos
  integer purs, **pas** la compute GPU (CUDA/Metal/GGUF). La bit-exactness a ete
  prouvee par le **spike hors-repo** (`sprint77_phase_f_fork_spike`, verdict GO) ;
  les tests GGUF sont `#[ignore]`+feature, **documentes runnable local** (audit_plan
  §1 documente deja cette frontiere). **Renforce PROVISIONAL**, pas un defaut neuf.
- **G3 — vendor fork sans pin upstream** : **REFUTE**. Le critique a grepe des
  tokens specifiques et **rate `THIRD-PARTY-NOTICES.md`** qui enregistre pin
  `4afdaf078...` (= 2× `.cargo_vcs_info.json` verifies) + licences + `patches/`.
  Le track vendor (qui a lu le fichier) avait raison. **Non-finding.**
- **G4 — disposition individuelle des 4 carries 3/3** : valable. Dispositions :
  **seeder catalog_len:0** → arbitrage PO design S78 ; **REVISION-HOME-DURABILITY**
  → MANDATORY S78 sauf exemption re-justifiee ; **KNOWN-ENTRY-OVERCOUNT** →
  exemption « dependance sequentielle » a re-justifier ou fermer ;
  **RE-DRIVE-ON-INGEST** → MANDATORY S78 (T2 RIG-ABSENT n'a pas prouve la
  convergence). Tous deja dans audit_plan §7 ; routes S78, jamais reconduits
  silencieusement.
- **G5 — parite honnetete Docker/iroh** : resolu par le run reel de l'orchestrateur
  (cf. §4 Track Suites) — 1820 passed / 6 iroh env-bloques (jeu documente, pas
  regression S77).
- **G6 — security CLEAN trop large** : framing valable. Le verdict security
  s'applique aux **mitigations d'un protocole NON-MONTE** (cf. G1) : les
  mitigations cablees (admission, cap-frame, fallback field) sont reelles+testees,
  et les items aspirationnels (SI-9/SI-5/SI-11, dispute) sont honnetement carry
  S78. Le CLEAN ne signifie PAS « data-plane live et defendu », mais « 0 surclaim,
  0 mitigation faux-declaree, frontiere honnete ».

**Couvert** : 11 tracks statiques + verification adversariale + critique + suite
dual-platform reelle + adjudication main-thread des faits contestes.
**Non couvert (assume)** : la compute GPU reelle cross-machine (rig 5080+Mac M2
absent — c'est precisement l'objet de S78) ; la review ligne-a-ligne des 505K
lignes vendor (auditees en provenance/patch, conforme a la nature d'un fork
vendored pinne).

**Conclusion** : S77 a livre un cœur sharding correct, deterministe, securise au
niveau des primitives, et une couche doc honnete drift-gated. La feature reste
**PROVISIONAL** (carry P1 → S78) de maniere mecaniquement enforce. **Gate PASS.**
