# Sprint 22 Phase E — preflight G8

Date : 2026-04-20
HEAD : `9c8805b`
Verdict : **EXECUTE plan-as-is**

Phase ciblée : **Watermark canari-input primitive** (spot-check
consumer 1/N task, distinct watermark-output Kirchenbauer
vulnérable BIRA 2025). Plan §8 / kickoff §4 D4.

---

## Scans

### S1 — SOTA 2026 vs design

**Libs scannées (coord-side Python only, aucune nouvelle dep)** :
- `rapidfuzz>=3.0` : déjà présent `pyproject.toml` (S21 Phase C
  EED echo 0.85). Pattern Levenshtein coord-side `output_filter.py`
  à réutiliser identique pour `CanaryInputObserver.similarity`.
- `typer>=0.12` : déjà présent (CLI pattern `quarantine` S21 Phase D
  + `canary ack` S20 Phase E). Pas de bump.
- `pydantic` : déjà présent (BaseModel coord schemas). Pas de bump.
- `nexus_core` (PyO3 binding Rust) : expose `sign_canary` /
  `verify_canary` / Ed25519 primitives depuis S14 (Keyoxide) +
  S21 Phase E wrap `canary_wire_bytes` JCS canonical. Pattern
  reuse pour signer `CanaryInputSet` (local integrity, pas wire).

**CVE / advisory** :
- `rapidfuzz` — aucun CVE 2026 open (context7 + WebSearch RustSec /
  PyPI advisories N/A — lib pure Python + Cython, pas wire-exposed).
- `typer` — aucun CVE 2026 open.
- `nexus_core` (interne) — pas d'advisory tierce applicable.

**Spec / recherche externe** :
- **BIRA attack arXiv 2509.23019** (sept 2025) : Bidirectional
  Iterative Rewriting Attack sur watermark-**output** style
  Kirchenbauer ICML 2023 (green-list logit biasing). Le plan §8.1
  D4 kickoff §4 rejette explicitement ce design : **Phase E
  implémente watermark-INPUT** (prompt-probe known-answer)
  sémantiquement distinct. BIRA non-applicable.
- Aucune nouvelle publication distributed-canary-prompt 2026
  trouvée. Gap académique = opportunité primitive nexus-grid
  (plan §8.5 body commit attendu mentionne ce gap).

**Verdict S1** : **clean**. Aucune lib nouvelle, aucun CVE ou
advisory touchant le scope, BIRA confirmé non-applicable au
design input-side.

---

### S2 — Decisions historiques traversées

**Commits scannés `--grep="DEVIATION|rejected|scope-cut|threat-model"`
sur `packages/nexus-coordinator/`** : 5 commits trouvés (S4 Phase C
invite v2, S7 Phase E /daemon proxy, S9 Phase D migration runner,
S18 Phase D TaskEntry wire, S21 Phase C PII coord). **Aucun ne
rejette un pattern input-canary / prompt-probe / spot-check**.

**Précédents historiques canary** (scan général `--grep="canary"`) :

| Sprint | Commit | Nature | Impact Phase E S22 |
|---|---|---|---|
| S18 Phase E2 | `04c9621` | warrant canary monthly auto-publish scheduler | **distinct** — warrant = anti-NSL external publish ; canary-input = runtime dispatch hook |
| S20 Phase E | `6a3f199` | federation foundations (CanarySigner trait + observational-only) | **distinct** — `CanaryRegistry` observe gossip warrant ; canary-input vit coord-side dispatcher |
| S20 Phase E | `bd16e64` pivot G8 | **rejet auto-publish** scheduler S18 E2 → "clef Ed25519 jamais exposée scheduler + CLI manuel uniquement" | **NON APPLICABLE** — Phase E S22 = CLI manuel rotate set + hook dispatch-time injection ≠ publish externe |
| S21 Phase E | `49f0d32` | `canary_wire_bytes` JCS canonical + `verify_canary` at ingest | **reuse direct** — signature CanaryInputSet = primitive à réutiliser via nexus_core PyO3 |
| S21 Phase C | `23abb11` | EED echo Levenshtein rapidfuzz 0.85 `output_filter.py` | **reuse direct** — pattern exact Observer similarity check |

**Reverse-commit check S20 Phase E rejet auto-publish** : aucun
revert (non-applicable — rejet toujours actif en memory
`nexus_grid_pivot.md §Sprint 20`). Mais **non-applicable à Phase E
S22** car design sémantiquement différent :
- Warrant canary (S18/S20) = publie message signé périodique vers
  gossip → un scheduler auto serait dangereux (coord kidnappé ne
  peut plus arrêter la publication sans violer warrant).
- Canary-input (S22) = INJECTE prompt vers worker en pre-dispatch
  hook runtime task → fonction tâche-dispatch locale, pas
  publication externe. La clef Ed25519 signe LE SET de prompts
  (rotate CLI manuel), pas chaque injection (qui est un
  dispatcher-side ref vers set pré-signé).

**Memory feedback scan** (`feedback_*.md`) : aucun pattern
"jamais faire canary-input" trouvé. Décisions approach-pick-
deepest + context7-systematic toutes respectées (rapidfuzz /
typer / nexus_core déjà cited stack éprouvée).

**Verdict S2** : **clean**. Aucune décision historique bloquante.
Tous les précédents canary sont soit distincts (warrant vs input)
soit patterns à réutiliser (JCS verify + rapidfuzz EED).

---

### S3 — Threat model coverage

**HARDENING_ROADMAP §3 S22 ligne 294-296** (vérifié textuellement) :

> « Spot-check watermark canari-input (consumer glisse 1/N prompt
> known-answer Ed25519-signed rotatable, distinct watermark-output
> Kirchenbauer vulnérable BIRA 2025) — ~300 Python »

→ Phase E S22 **implémente exactement cette primitive**. Plan §8.2
~250 LOC vs roadmap ~300 LOC : marge estimation, cohérent (bornes
±20%).

**Threat mapping** :
- **C-ComputeTheft / silent-model-swap** (worker utilise modèle
  moins cher, réponse plausible mais non-exacte) → couvert par
  canary-input spot-check : worker qui swap Qwen2.5-7B vers
  Qwen2.5-0.5B échoue les known-answer probes → divergence
  Levenshtein < tolerance → alerte Observer.
- **C-SilentOutputTamper** (worker tampers output) → partiellement
  couvert si tamper pattern change l'answer distance > tolerance.
- **Défense distincte** du threat couvert par rate-limit Phase A
  (resource exhaustion) + PII Phase B/C (data leak) + Sybil
  Phase C (identity flood) + NVML Phase D (compute profile
  observability S24). Pas de chevauchement.

**Regression flags** : aucun threat actuellement couvert n'est
régressé. Le hook `pre-dispatch` ajoute un step sans toucher
rate-limit / PII / dispatcher-state-machine.

**HARDENING gaps S22** : ligne 311-312 mentionne
`canary_input S22E` comme futur consumer pour B1 Guardrails
refactor S23 (`GUARDRAILS_ARCHITECTURE.md` écrit S22 hors-sprint).
Phase E alimente cette architecture. Cohérent.

**Verdict S3** : **clean**. Primitive couvre gap explicitement
attendu par roadmap, aucune régression, alimente S23 B1.

---

### S4 — Wire format / pre-launch invariants

**`*_VERSION` scanning** : Phase E ne touche aucun fichier
`crates/nexus-core-rs/src/*`. `BLOB_VERSION=0x01`,
`TASK_VERSION=1`, `CANARY_VERSION=1` tous **unchanged**.

**`canonical.rs`** : non-touché. `schemas/` non-touché.

**Phase E est 100% coord-side Python** :
- `packages/nexus-coordinator/src/nexus_coordinator/canary_input.py`
  (nouveau ~250 LOC) — local set + observer.
- `api/canary.py` (modifié) — ajout 2 endpoints loopback.
- `cli/` (modifié) — Typer commands.
- `~/.sbfb/canary_input_policy.toml` (nouveau) — config locale
  file-watched (pattern S18 TokenRotator).

**Signature Ed25519 CanaryInputSet** : usage **local integrity**
(verify set au reload file-watcher), **pas wire P2P**. Pattern
S18 Phase D TokenRotator : sign + store + file-watch + verify
reload. Pas de publication gossip, pas de serialization JCS
cross-node, pas d'invariant wire introduit.

**Day 0 kickoff §4 D4** : « primitive only (pas de backend ML,
pas de signature complexe), CLI manuel rotate » — Phase E plan
§8.2 respecte 100% : pas de ML, Ed25519 seul (existant depuis
S14), CLI `canary-rotate` + `canary-status` (pattern Typer
S19+). **Aucune D1..D5 contredite**.

**`nexus_grid_pivot.md §Decisions actées`** : pré-launch protocol
policy — aucun bump de version, aucun tolerant-decoder multi-
version, `#[serde(default)]` non-applicable (pas de Rust struct
touchée). Respecte 100%.

**Verdict S4** : **clean**. Aucun wire format modifié, aucune
version bumpée, Day 0 préservé, pre-launch protocol respectée.

---

## Synthèse

| Scan | Résultat |
|---|---|
| S1 SOTA 2026 | clean (stack existant, BIRA non-applicable input-side) |
| S2 Historiques | clean (précédents distincts ou reuse direct, rejet S20 auto-publish non-applicable) |
| S3 Threat model | clean (HARDENING §3 S22 ligne 294-296 explicite la primitive, alimente S23 B1) |
| S4 Wire / pre-launch | clean (coord-side Python only, zéro _VERSION, Day 0 D4 conforme) |

**Verdict G8** : **EXECUTE plan-as-is**.

---

## Action

Procéder implémentation Phase E selon plan §8 sans déviation :
1. `canary_input.py` — `CanaryInputSet` (pydantic + Ed25519 sign
   via `nexus_core.sign_canary` pattern reuse) +
   `CanaryInputInjector` (hook `dispatcher.pre_dispatch` 1/N
   sampling) + `CanaryInputObserver` (rapidfuzz.ratio ≥ tolerance
   reuse pattern S21 Phase C output_filter).
2. `api/canary.py` — 2 endpoints loopback `POST /canary/inject-rate`
   + `GET /canary/observed-divergence` (pattern S20 Phase E
   `/canary ack` router reuse).
3. CLI Typer — `canary-rotate` (rotate set + resign) +
   `canary-status` (list current set + recent divergences).
4. `~/.sbfb/canary_input_policy.toml` — `inject_rate` (default 1/100),
   `tolerance` (default 0.85), `rotation_frequency_days`.
5. 5 tests Python coord attendus (delta Phase E = +5) selon plan
   §8.3 : inject_rate_1_per_100 + signature_rotation +
   observer_alert_low + observer_pass_high + api_endpoints_smoke.

Body commit cible plan §8.5 respecté : « feat(sprint22): Phase E —
watermark canari-input spot-check consumer 1/N primitive », corps
riche avec delta tests +5 Python coord + distinction BIRA + gap
prior art + working tree audit G5 + G8 EXECUTE.

**Pas de carry-over S23 à ajouter** (verdict EXECUTE = 0 finding
non-bloquant). Meta-track G8 traceability (kickoff §4.5) :
5/6 phases A-E avec preflight.md émis (B-E = 4/6 + A déjà).
Phase F reste doc-only preflight.

---

## Garde-fous §6.9 (vérification formelle)

| Garde-fou | Check | Status |
|---|---|---|
| 1. Evidence-based pivot | N/A (pas de pivot) | ✅ (EXECUTE direct) |
| 2. Day 0 respect | D4 kickoff §4 primitive-only préservé | ✅ |
| 3. Wire format | Aucun `*_VERSION` bumpé pre-launch | ✅ |
| 4. Test budget cap | +5 tests = 2.5% delta coord (marge < 2.5x) | ✅ |
| 5. Thème sprint | Sybil + PII + canary theme (kickoff §1) | ✅ |
| 6. Pas YAGNI | Consommé par S23 B1 Guardrails explicite | ✅ |
| 7. Retrospective trackée | N/A (pas de pivot, juste EXECUTE log) | ✅ |

Tous les garde-fous passés. Implémentation autorisée.
