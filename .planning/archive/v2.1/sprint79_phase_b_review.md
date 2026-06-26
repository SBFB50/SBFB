# Review — Sprint 79 Phase B (canon cadence docs-contrat + gate `check-frontier-contracts.sh`)

> Produit par Workflow ultracode `review-s79-phase-b` (6 agents Opus 4.8 1M : R1 honnêteté
> scrub, R2 gate, R3 CI, R4 docs, R5 complétude + synthèse adversariale). **3 rounds** :
> round 1 FAIL (3 P1), round 2 FAIL (1 P1), round 3 **PASS-PENDING (0 P0 / 0 P1)**.
> Chaque round a attrapé de vrais défauts dans le scrub — la vérification adversariale a
> fonctionné. Les findings ont été corrigés in-phase puis re-vérifiés.

## Verdict: PASS

0 P0 / 0 P1 (review interne, round 3) ET Codex GPT 5.5 réconcilié (4 rounds,
cf. ## Codex reconciliation). 0 P0 behaviour sur tout le diff. Reste 1 GAP Codex
DISCLOSÉ + ROUTÉ (carry `task_response`, wire-schema-couplé) — commit autorisé par
la règle META-1 (GAP nommé + routé + réconcilié ici).

## Historique des rounds (la review a fait son travail)

- **Round 1 — FAIL (3 P1)** :
  1. `shell-daemon-core/lib.rs:33` — rewrite inventait un module `curator_runtime` inexistant
     (le type réel = `iroh_runtime::CuratorRuntime`). Faux « fait ». → corrigé.
  2. `shell/PATTERNS.md` — affirmait à tort que `daemon.ts`/`ShardSessionPanel.tsx` « carry no
     such promise » alors qu'ils portaient des promesses Phase-K (FR, gate aveugle). → fichiers
     web scrubbés + note corrigée + gate élargi au FR (`arrive en Phase`).
  3. `tui.rs:170` — promesse `W9.1 will introduce` non scrubbée (jumeau de `:452` scrubbé) ;
     gate aveugle aux waves décimales + `introduce`. → scrubbé + gate durci
     (`W[0-9]+(\.[0-9]+)? (will|adds|ships|introduce)`).
- **Round 2 — FAIL (1 P1 + P2)** :
  1. **P1** `phase-review-cross-check.yml` — le fix scope `[a-z0-9_-]+` excluait `+` (scopes
     composites `feat(bridge+daemon)`, `feat(coordinator+shell+worker)`) → 132 match au lieu de
     147 ground-truth, 15 commits skippés silencieusement (auto-référentiel : `chore(process+ci):
     Sprint 79 Phase B` aurait été skippé). → `[a-z0-9_+-]+` aux 3 sites ; re-vérifié = **147**.
  2. P2 — 5 promesses non-adjacentes résiduelles (paths.rs:93, registry.rs:38, e2e.rs:18,
     gpu/profile.rs:371+554, keystore.rs:397) + 1 rewrite faux registry.rs:265 (read_running
     « exposé via /daemon/info » — faux). → toutes scrubbées/corrigées ; grep résiduel = 0.
- **Round 3 — PASS-PENDING (0 P0 / 0 P1)** : 2 corrections d'honnêteté MAJEURES validées exactes
  (`consent.rs` ex-« L2 inert » FAUX → `runtime.rs:1026` passe la vraie valeur ; `registry.rs`
  read_running ex-« Phase E expose » FAUX → non câblé). 3 P2 doc + 8 P3 cosmétiques.
  **P2 repliés dans le commit B** (post-round-3) : `keystore.rs:314` `(cf. Phase B)` retiré ;
  en-tête gate section (3) reformulé (asserte CHAQUE directive, pas « META only ») ;
  README §6.12 « ~22 familles » → « 22 des 25 familles DOMAIN_*_V1 sans schéma généré ».

## Dimensions (round 3)

- **R1 — honnêteté du scrub — PASS.** ~30 rewrites vérifiés adversarialement contre le code.
  Aucun « faux fait ». Claims présent fondés (modules docs/gossip/blobs/discovery, CuratorRuntime,
  boucle async, duress livré). « Not yet wired » confirmés (Tor handle droppé, stubs cli
  print_stub, rpassword absent, slowapi/FastAPI retiré pivot S50-S51, build_executor dormant).
  2 corrections où l'ancien commentaire MENTAIT (consent.rs, registry.rs read_running).
- **R2 — gate — PASS.** `check-frontier-contracts.sh` BusyBox-safe, shellcheck clean, exécuté Win
  Git Bash + Docker `bash:5` pinné → exit 0 `[1 tagged]`. 3 volets à vraies dents (injection →
  FAIL → restore → exit 0). Anti-promesse ANCRÉE (0 faux-positif), scope `crates`+`web/src`
  (docs/ exclus par construction). FRONTIER opt-in incrémental (ShardPlan, consts+schema résolus).
  BLOB_SERVE_CSP 6 directives vs `blob_serve.rs:286`.
- **R3 — CI — PASS.** 3 surfaces câblées (ci.yml step [15], woodpecker même digest, verify.sh
  step 20). Fix `phase-review-cross-check.yml` validé 3 axes (regex 147 commits dont scopes
  composites, sed F2/E1/AA, lowercasing path).
- **R4 — docs — PASS.** §P70 / README §6.12 / AGENT_SYSTEM §7 (Non-Goals→§8) / shell note
  cohérents avec le code (8 sharding + TaskResponse, 6 CSP, 22 of 25 DOMAIN). Truth-Stack
  canonique byte-identique (AGENT_SYSTEM.md:8 == check-sharding-docs.sh:217 == README == §P70).
- **R5 — complétude/scope/0-behavior — PASS.** grep résiduel VIDE ; 0 ligne non-commentaire sur
  28 .rs + 3 .ts/.tsx ; `// FRONTIER:` = ligne-commentaire ; scope tenu (0 prompt-kind Phase C,
  0 CSP runtime Phase E) ; delta tests = 0 ; 0 bump `*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`.

## P0/P1 à corriger

Aucun. **0 P0, 0 P1.**

## P2/P3 documentés

- **P2 (repliés dans commit B)** : keystore.rs:314, en-tête gate section (3), README §6.12 — FAITS.
- **P2 (carry, hors-livrable)** : `.planning/research/doctrine_contrat_pour_llm.md:208/218/219/287`
  garde les chiffres périmés (23/21) vs canonique « 22 des 25 » — research doc hors-gate ; le
  preflight (WI-10) a délibérément choisi de ne pas corriger la doctrine. Carry note.
- **P3 (cosmétiques, aucune action en B)** : bannières stub `main.rs:82`/`e2e.rs:145` (mécanisme
  stub actif, exemption) ; `sprint80_audit_plan.md` référencé mais absent (créé à l'ouverture S80,
  y matérialiser le carry « 22 des 25 DOMAIN_*_V1 sans schéma ») ; casse cosmétique
  `phase-review-cross-check.yml:92-93` (`${phase}` dans le message d'erreur seul, lookup correct
  via `${phase_lc}`) ; `docs/shell/PATTERNS.md:395/672` anciennes promesses (hors-scope, docs/ hors
  gate) ; `sprint79_plan.md:484/490` « ~21 familles » périmé (plan non-livrable, carry S80).
- **Note process** : 2 agents de review ont fait des teeth-tests du gate sur le vrai `pow.rs:887`
  au lieu d'une copie scratchpad (pollution intermittente de l'arbre partagé). Le gate a attrapé
  chaque injection ; arbre nettoyé, `git diff pow.rs` = uniquement la réécriture doc-comment
  S22/S26 (vérifié). Discipline : teeth-test sur copie isolée.

## Invariants confirmés

- **0 changement de comportement Rust** : `git diff '*.rs'` filtré hors commentaires = VIDE
  (28 .rs + 3 .ts/.tsx). Seule addition non-doc = `// FRONTIER:` (`shard_plan.rs:188`,
  ligne-commentaire, struct ShardPlan intact).
- **0 bump wire, 0 dépendance** : aucun `*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`/`SCHEMA_VERSION`
  de code modifié ; aucun Cargo.toml dep touché (éditions = doc-comments).
- **Carry honnête** : « 22 des 25 DOMAIN_*_V1 sans schéma généré » dans §P70/README/script ;
  carry routé `sprint80_audit_plan.md`.
- **Gate vert** : `check-frontier-contracts.sh` exit 0 ; `check-sharding-docs.sh` exit 0
  (§P64-§P69 intactes + §P70 ajouté).
- **Dual-platform** : gate shell exit 0 Win Git Bash + Docker `bash:5` + shellcheck ;
  fmt 0 Win + Docker 1.94 ; Win nextest 1957 + clippy + doctest + release ; web 411 + lint +
  tsc + scan + build + size. Câblé CI 3 surfaces.

## Codex reconciliation

Codex GPT 5.5 (`codex exec`, cross-model) exécuté **4 rounds** sur l'état final
(artefact brut : `sprint79_phase_b_codex_review.md`, round 4, NON réécrit).
Deliverables 1/3/4/5 CONFIRMED ; 2 & 6 PARTIAL (résidu = le carry ci-dessous).
0 P0 behaviour (seul edit non-commentaire = 1 message d'assert `http.rs:5292`,
0 effet sur l'assertion — invariant « 0-behaviour »).

Findings Codex corrigés in-phase (rounds 1→4) :
- Faux module `curator_runtime` → `iroh_runtime::CuratorRuntime`.
- 4 promesses Phase-K web (ShardSessionPanel/daemon.ts) → S78 seam.
- `tui.rs` W9.1 + gate aveugle décimal/`introduce` → scrub + gate durci.
- Scope composite `+` `phase-review-cross-check.yml` → `[a-z0-9_+-]+` (147 commits).
- `runtime.rs` « not yet wired » FAUX → /browse consomme `curator_runtime`.
- `cli.rs` « return error » FAUX → print stub + Ok.
- `registry.rs` read_running « /daemon/info » FAUX → non câblé.
- core `lib.rs` PyO3/nexus-core-py périmé → post-pivot Rust-only.
- Gate `schema_for!` toothless → reword annotation + gate robuste `| grep -qvF FRONTIER`.
- 8 « Phase K » résiduels (http.rs/sentinel.rs/toploc.rs/daemon.test.ts) → S78 carry.
- FRONTIER silent-removal → required-registry assertion ShardPlan.
- 2 P2 web future-claims → current-state.
- keystore.rs:891 / frost.rs:105 sprint-anchored → scrub.
- MANIFEST « self-hash » inexact → « registre de hash par fichier ».
- codex_review.md « stale » (round 4 P2) → moot : round 4 a écrasé l'artefact.

GAP DISCLOSÉ + ROUTÉ (règle META-1, commit autorisé) :
- **task_response.rs S20/S22 + task_response.schema.json** (P1 round 4) : 4 doc-comments
  + 2 descriptions de schéma GÉNÉRÉES ancrant le champ wire `tool_calls` (déclaratif S20,
  activation sandbox différée). **WIRE-SCHEMA-COUPLÉ** : scrubber les doc-comments
  régénère la description `schema_for!` → exige une régén byte-exacte du snapshot
  `task_response.schema.json` (tâche focalisée, risque snapshot sous contrainte de contexte).
  Hors de la classe STALE-PHASE-K (Phase-letter) ciblée par Phase B ; classe sprint-ancrée
  pré-existante. → **CARRY routé `sprint80_audit_plan.md`** (sweep provenance sprint-ancrée
  + régén snapshot). Le gate anti-promesse est ANCRÉ (formes adjacentes) ; cette classe
  non-adjacente est attrapée par review/Codex, pas le gate (limitation documentée §P70).
