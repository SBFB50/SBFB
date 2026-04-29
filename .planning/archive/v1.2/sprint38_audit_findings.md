# Sprint 38 — Audit findings

**Auditeur** : session fraiche (pas la session qui a code S38).
**Tip audite** : `16ad15e` (S38 Phase C, dernier feat commit).
**Tip d'entree** : `4e842d5` (S38 Phase D wrap-up).
**Audit plan** : `sprint39_audit_plan.md` (6 tracks, 17 items).

---

## Verdict : **PASS** (1 P2 fixe + 1 P3 carry)

G4 rigor signal satisfait : 1 P2 + 1 P3 documentes (>=1 P2+
requis pour PASS, cf. §6.1.1).

---

## Track A — Securite / output filter

### A-1 : invisible text scanner Unicode ranges — **PASS**

Ranges Rust (`output_filter.rs:26-35`) identiques au Python
(`output_filter.py`) :
- Zero-width : U+200B..200F, U+2060, U+FEFF — match exact
- PUA : U+E000..F8FF, Planes 15-16 — identique
- Tags : U+E0020..E007F — identique
- Whitelist bidi : U+202A..202E + U+2066..2069 — identique

### A-2 : prompt echo EED + substring — **P2** (off-by-one FIXE)

**EED** : `strsim::normalized_levenshtein` et
`rapidfuzz.Levenshtein.normalized_similarity` utilisent la meme
formule (`1 - distance / max(len1, len2)`) avec seuil 0.85 →
correct.

**Bug substring (P2-AUDIT-A-2-S38)** : `output_filter.rs:70`
utilisait `0..prompt_lower_chars.len().saturating_sub(min_len)`.
Quand `len == min_len` (40 chars), le range `0..0` est vide →
aucune slice testee. Python utilise
`range(0, len(sp) - min_len + 1)` qui teste 1 slice.
**Fix** : `..` → `..=` (range inclusive). Test de regression
`prompt_echo_substring_exact_min_len` ajoute.

**Note P3** : le Rust fait `to_lowercase()` avant substring
match, le Python est case-sensitive. C'est un durcissement
(detection plus large), pas une regression de securite.
Carry P3-AUDIT-A-2b-S38 : documenter la divergence
comportementale dans le code si necessaire.

### A-3 : guardrail wire submit_result — **PASS**

`http.rs:1346-1371` : sequence correcte
`validate_result()` → `GuardrailContext` +
`default_output_chain().run()` → si `!passed` : return 400
rejected, pas de credit → sinon : `kudos_ledger::credit()`.
Le tripwire bloque effectivement le credit kudos.

---

## Track B — Architecture / validator_loop

### B-1 : broadcast capacity 64 + Lagged — **PASS**

- Capacity : `CHANNEL_CAPACITY = 64` (`validator_loop.rs:29`).
- Lagged : `validator_loop.rs:42-44` — `RecvError::Lagged(n)`
  match avec `tracing::warn!` + continue. Pas de panic.
- Closed : L45-48, break propre quand sender droppe.

### B-2 : idempotence set_task_result — **PASS**

`db.rs:153-155` — clause SQL
`WHERE task_id = ?4 AND status IN ('pending', 'dispatched')`.
Un task deja `completed` → `changed = 0` → `Ok(false)` → pas
de double credit. Test couvert :
`validator_loop_idempotent_double_submit` (42 kudos, pas 84).

### B-3 : result_event_tx dead code — **PASS**

`http.rs:157-158` — `#[allow(dead_code)]` sur
`result_event_tx: ResultEventSender`. Sender stocke pour
maintenir le channel ouvert (Receiver passe au validator_loop).
Droppe avec `Arc<DaemonHttpState>` au shutdown → Receiver recoit
Closed → break propre. P2 carry existant
`P2-REVIEW-A-1-S38` (1/3) reconnait ce dead code path.

---

## Track C — Tests / coverage

### C-1 : delta tests 946→967 (+21) — **PASS**

21 tests repartis : 3 validator_loop + 10 output_filter +
6 guardrails + 1 verify_chain + 1 launcher log_dir.
Tous avec assertions substantielles (assert_eq!, comparaisons
de valeurs reelles). 0 stub `assert!(true)`.

### C-2 : output_filter tests 10/10 — **PASS**

Layer 1 (invisible) : 4 tests (zero-width, bidi, PUA, tags).
Layer 2 (prompt echo) : 4 tests (exact, substring, EED detect,
EED below threshold).
Layer 3 (integration) : 2 tests (clean pass, invisible detect).
Couverture correcte des 3 layers.

### C-3 : guardrails tests 6/6 — **PASS**

4 tests chain (empty, pass-through, flag accumulate, tripwire
short-circuit). 2 tests integration OutputSafetyGuardrail
(clean pass, invisible tripwire). Mocks AlwaysPass/AlwaysFlag/
AlwaysTripwire implementent le trait complet.

---

## Track D — Process / meta

### D-1 : G8 preflights coherence — **PASS**

3 preflights (A+B+C) tous verdict EXECUTE plan-as-is.
Scans S1a/S1b/S2/S3/S4 documentes. Pas de drift plan→code
non documente.

### D-2 : scope cuts 12/12 — **PASS**

Grep PiiRedactor/CanaryRegistry dans le diff S38 : seules
occurrences = commentaires scope-cut. Aucune implementation.

### D-3 : MANDATORY 3/3 ferme — **PASS**

`runtime.rs:576` — `tokio::spawn(validator_loop::run(...))`.
Le loop est reellement spawne au boot avec broadcast channel
cree L477. Module `validator_loop.rs` complet (3 tests).

---

## Track E — Dependencies

### E-1 : strsim Cargo.lock — **PASS**

strsim 0.11.1 present dans Cargo.lock. strsim 0.10.0 aussi
present (dep transitive clap) — coexistence normale.

### E-2 : strsim RustSec — **PASS**

strsim 0.11.1 = pure Rust, MIT, 0 dep transitive.
Aucun advisory RustSec connu. Risque residuel negligeable.

---

## Track F — Doc coherence

### F-1 : HARDENING_ROADMAP compteurs — **PASS**

`last_validated: 2026-04-29` — 967 Rust / ~1970 total.
Coherent avec verification.md.

### F-2 : CLAUDE.md etat actuel — **PASS**

S38 CLOSED, 967 Rust, compteurs alignes.

### F-3 : phase review files 3/3 — **PASS**

A_review + B_review + C_review presents dans active/.

### F-4 : phase preflight files 3/3 — **PASS**

A_preflight + B_preflight + C_preflight presents dans active/.

### F-5 : PATTERNS.md P33 rowid — **PASS**

`docs/shell/PATTERNS.md` L2089 : P33 — rowid tiebreaker in
kudos ORDER BY queries. Section documentee avec invariant.

---

## Resume des findings

| # | Track | Severite | Description | Action |
|---|---|---|---|---|
| P2-AUDIT-A-2-S38 | A-2 | P2 | Off-by-one substring loop `output_filter.rs:70` : range exclusive quand `len == min_len` → slice manquee | **FIXE** dans ce commit (`.=` inclusive + test regression) |
| P3-AUDIT-A-2b-S38 | A-2 | P3 | Divergence comportementale : Rust lowercase vs Python case-sensitive (durcissement, pas regression) | Carry S39 (1/3) |

---

## Carries S39 (mis a jour)

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 6+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P3-grammar executor | 3/3+ | defer Rust pipeline S40 |
| P3-watermark executor | 3/3+ | defer Rust pipeline S40 |
| P2-REVIEW-A-1-S38 result_event_tx dead code | 1/3 | wire gossip S39+ |
| P2-REVIEW-B-1-S38 substring O(n*m) | 1/3 | perf post-v1.0 |
| P2-REVIEW-C-1-S38 chain Arc singleton | 1/3 | perf post-v1.0 |
| P2-REVIEW-A-1-S37 launcher logging test | 2/3 | Phase A partial |
| P3-AUDIT-A-2b-S38 lowercase divergence | 1/3 | doc post-v1.0 |
