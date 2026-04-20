# Sprint 22 Phase A — preflight G8

Date : 2026-04-19
HEAD : `87b0891`
Verdict : **EXECUTE plan-as-is**

Phase cible : A — rate-limit engine wire-up (P2-S21-1 + P2-S21-2 +
P3-S21-4). Plan §4 (`sprint22_plan.md` L96-178).

Fichiers cibles Step 1.3 :
- `crates/nexus-worker-core/src/engine/runtime.rs` (modifié, wire
  `rate_limiter.check(RateKey)` avant `ClaimEntry::sign`)
- `crates/nexus-worker-core/src/rate_limit.rs` (modifié, ajout
  `swap_policy` Arc swap `DefaultKeyedRateLimiter`)
- `crates/nexus-worker-core/src/rate_limit_policy_loader.rs`
  (modifié, callback hot-reload appelle `swap_policy`)
- `crates/nexus-worker-core/configs/rate_limit_policy.toml.sample`
  (nouveau, fix P3-S21-4)
- Tests Rust +7 (5 intégration engine + 2 loader swap)

Libs / deps / wire format touché :
- Lib crypto/wire/network : **aucune** (pas de crypto, pas de wire
  format, pas de network).
- Lib interne : `governor 0.10.2` (déjà pinned workspace S21
  `5e67ce0`), `notify` (déjà pinned via S20 `PowPolicyWatcher`
  pattern), `tokio`, `tracing`, `dashmap`, `arc-swap`.
- Threat model claim : défense C-ModelExtract + C-DosFlood
  (runtime effectivité vs S21 primitive non-câblée).

---

## Scans

### S1 — SOTA 2026 vs design

Libs scannées :
- `governor 0.10.2` (Cargo.toml workspace pinned) via
  `grep -E "governor" Cargo.toml crates/*/Cargo.toml`.

Context7 queries :
- `mcp__claude_ai_Context7__resolve-library-id("governor")`
  → `/boinkor-net/governor` (Medium reputation, benchmark 75.6,
  51 code snippets), 2026-04-19.
- `mcp__claude_ai_Context7__query-docs("/boinkor-net/governor",
  "DefaultKeyedRateLimiter Arc swap hot-reload rebuild pattern
  for policy changes")`, 2026-04-19.
  - Output confirme `RateLimiter::keyed(quota)` constructeur,
    `check_key(&key)` méthode publique, `Arc<RateLimiter>` pattern
    thread-safe, `retain_recent()` + `shrink_to_fit()` housekeeping.
  - **Pas de méthode atomique de reset per-key intrinsèque** →
    reconstruction via nouveau `DefaultKeyedRateLimiter` + Arc
    swap = pattern canonique confirmé.

WebSearch CVE / changelog :
- `"rust governor crate 0.10.2 0.11 changelog 2026 breaking
  changes"`, 2026-04-19 : 0.10.4 publié Dec 2025, **pas de 0.11**.
  Bumps 0.10.2 → 0.10.4 = PATCH semver-stable, pas de breaking
  change documenté. Notre pin 0.10.2 reste valide ; bump optionnel
  opportuniste hors-scope Phase A.
- `"rustsec governor rate-limiter advisory 2026 CVE"`, 2026-04-19 :
  feed RUSTSEC 2026 couvre pingora (CVE-2026-2835), time
  (RUSTSEC-2026-0009), aws-lc (RUSTSEC-2026-0042), tar-rs
  (CVE-2026-33056). **Aucune advisory governor 2026**.

Délais analysés :
- Publication plan kickoff 2026-04-19 ↔ dernière release governor
  0.10.4 Dec 2025 = 4 mois gap. Aucun delta API critique.
- Phase A S21 (`63afe4e` 2026-04-19 même journée) a intégré
  `governor::DefaultKeyedRateLimiter` + `nonzero_ext::nonzero!`
  macro + `check_key` → API cohérente SOTA.

**Verdict S1 : clean**.

### S2 — Décisions historiques traversées

`git log --all --grep="DEVIATION|rejected|scope-cut|deliberate|
threat-model" -- <fichiers Phase A>` :
- Commit `63afe4e` S21 Phase A (2026-04-19) : **livre la primitive**
  `RateLimiter` + `RateKey` + `RateLimitPolicy` + loader + tests.
  Explicitement appelé « worker-engine gate pure Rust R1 scope-cut
  ». **Wire-up au chemin critique engine = continuation directe**,
  pas rejet.
- Commit `b4bda81` chore(planning) S21 : R1 scope-cut mid-phase
  drift documenté — HTTP middleware pattern incompatible avec
  stack nexus-grid (`/task/submit` vit Python FastAPI
  `packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py`,
  pas Rust shell-daemon). **Phase A S22 cible worker-engine
  (crates/nexus-worker-core) = EXACTEMENT la zone scope R1
  retenue**. Pas de rejet applicable à Phase A S22.

Archive scan `.planning/archive/v1.2/sprint21_*.md` :
- `sprint21_audit_findings.md` : P2-S21-1 (RateLimiter non-câblé)
  + P2-S21-2 (hot-reload incomplet) + P3-S21-4 (sample absent)
  = findings **explicitement ciblés S22 Phase A** (carry
  Track A-1/A-2 audit plan S22).
- `sprint21_phase_A_review.md` : PATTERNS.md §P33 documente
  engine integration outline (pattern Phase C S20 `16b94ba` PoW
  runtime wire S19 A-2 carry = précédent direct absorption
  wire-up debt sans consommer slot G7).
- `sprint21_phase_F_review.md` L21 : « rate_limit_policy.toml.
  sample absent : Phase A R1 scope-cut (defaults runtime
  suffisent). Carry S22 Track A-2. » = livrable Phase A S22 P3-S21-4.

Memory feedback scan :
- `feedback_approach.md` : « No band-aids ». Wire engine ne rebat
  pas Day 0, livre exactement ce que la primitive S21 prépare.
- `feedback_context7_systematic.md` : context7 fresh governor
  exécuté ci-dessus, compliant.

Reverse-commit check :
- Aucune commit « rejected » à retourner n'existe pour la zone
  rate-limit worker-engine. R1 scope-cut (`b4bda81`) est un narrow
  scope-cut latéral (dropped HTTP middleware, retained engine
  gate) — pas un rejet à inverser.

Note mineure (non-finding, documentation drift interne au plan) :
- Plan §4.2 dit « `runtime.rs` ~ligne 150 » selon audit agent 8.
  Inspection actuelle : `ClaimEntry::sign(claim, &self.keypair)`
  se trouve ligne **833** (`fn run_until_shutdown` après
  `claim.sign()` L828). Ligne 150 correspond au setup
  `author` / metadata, pas à l'emission de claim. Intent plan
  (« avant `ClaimEntry` broadcast ») reste sans ambiguïté → wire
  insertion autour de L820-833 avant `let claim = Claim::new(...)`.

**Verdict S2 : clean** (note de drift plan consignée, ne change
pas le verdict ni le wire-point décidé).

### S3 — Threat model coverage

Threat mapping Phase A :
- **C-ModelExtract** (`HARDENING §3 S21 §4 model extraction
  paper-flood`) : primitive GCRA sliding-window déjà livrée.
  Phase A **wire engine** = rend la défense effective runtime
  (sinon primitive dormante = vulnérabilité latente).
- **C-DosFlood** (`HARDENING §3 S21 §7 DoS flood`) : même couverture
  runtime-effective.

Regression flags :
- Pas de régression introduite : le wire ajoute un `rate_limiter.
  check()` gate avant `ClaimEntry::sign`. En cas de saturation,
  task deferred (pas détruite) → pattern « reject-preserve »
  identique au plan P2-S21-1. Les paths non-rate-limited (si
  tuple non couvert par policy) → fallback default quota
  `RateLimitPolicy.default`.

HARDENING_ROADMAP §3 S22 ligne 250-271 : items 1-5 couverts par
S22 plan §4-8. Phase A cible spécifiquement items 1 (couche 1/2
Sybil) indirectement via rate-limit per-consumer.

**Verdict S3 : clean**.

### S4 — Wire format / pre-launch invariants

`_VERSION` fields touchés :
- Aucun. Rate-limit est un gate runtime local au worker.
- `TaskEntry`, `ClaimEntry`, `ResultEntry`, `ProjectAnnouncement`,
  `CuratorListEntry`, `ProvenanceRecord`, `CanarySigned` :
  intouchés.
- `TASK_FORMAT_VERSION`, `BLOB_VERSION`, `TASK_RESPONSE_VERSION`,
  `CANARY_VERSION`, `ANNOUNCEMENT_VERSION`, `CURATOR_LIST_VERSION`,
  `PROVENANCE_VERSION` : inchangés.

`canonical.rs` touché : **non**. Pas de nouveau `DOMAIN_*`, pas de
réordonnancement de champs canonical JCS.

`#[serde(default)]` ajoutés : **non** (config TOML locale lue via
`toml::from_str` ; pas de wire format décodé cross-noeud).

Day 0 S22 D1..D5 préservées :
- **D2 (Scope γ hybride 6 phases)** : Phase A = première phase du
  scope, livrée conforme plan.
- **D1, D3, D4, D5** : non touchés par Phase A.
- Décisions actées `nexus_grid_pivot.md §Décisions actées` :
  Rust-first, Option G hybride, iroh 0.97, HTTP loopback via
  coordinator proxy — aucune contredite.

Pre-launch protocol policy (CLAUDE.md) : respectée strictement.
Aucun bump version pré-v1.0 sans CVE bloquant. Aucun tolerant
decoder multi-version introduit.

**Verdict S4 : clean**.

---

## Action

Procéder code Phase A selon plan §4. Verdict G8 **EXECUTE
plan-as-is** confirmé sur les 4 dimensions.

Ajustement micro à acter pendant l'implémentation (documentation
drift interne au plan, pas finding) : wire `rate_limiter.check()`
autour de `runtime.rs` L820-833 (site réel `ClaimEntry::sign`),
pas L150 (site audit agent 8 `author` setup). Pas de pivot
nécessaire ; intent plan préservé (« avant claim broadcast »).

Pas de carry-over S+1 requis (aucun finding non-bloquant détecté).
Pas de pivot_proposal.md nécessaire.

---

## Refs

- `docs/claude/README.md §6.9` (G8 source-of-truth)
- `.claude/skills/nexus-phase-preflight/SKILL.md` (procédure exécutée)
- `sprint22_plan.md §4` (Phase A scope)
- `sprint22_kickoff.md §4 D2` (scope gelé)
- Commits traversés : `63afe4e`, `b4bda81` (S21 Phase A +
  scope-cut record)
- Context7 `/boinkor-net/governor` queried 2026-04-19
- WebSearch RUSTSEC 2026 feed 2026-04-19
