# Sprint 39 — Audit findings (gate S40)

**Auditeur** : session fraiche (pas la session qui a code S39).
**Tip d'entree** : `c1afec6` (HEAD master, S39 Phase D docs).
**Audit plan** : `sprint40_audit_plan.md` (6 tracks, 18 checks).
**Date** : 2026-04-29.

## Verdict : PASS

0 P0, 0 P1 — gate ouverte pour Sprint 40.
1 P2 carry confirme + 1 P3 nouveau.
G4 rigor signal satisfait (>=1 P2+ documente avec evidence).

---

## Track A — Securite / PII redactor

### A-1 : Regex patterns parity — PASS

Les 7 patterns (email, phone, credit card, IBAN, SSN, IP, URL) sont
structurellement identiques entre `pii_redactor.py:68-110` et
`pii_redactor.rs:24-64`. Comparaison ligne par ligne effectuee.

**P3-AUDIT-A-1-S39** : URL regex divergence mineure — Python
exclut `"` mais pas `'` dans le character class. Rust exclut les
deux (`[^\s<>"']+`). Le Rust est marginalement plus correct (single
quote peut terminer des attributs HTML). Impact negligeable.

### A-2 : Luhn validation parity — PASS

Algorithme identique pas a pas entre `pii_redactor.py:113-125` et
`pii_redactor.rs:68-91` : extraction digits, length check 13-19,
reversed enumeration, double odd-indexed, subtract 9 if >9, sum
mod 10. Tests confirment parite (Visa 4111... valid, 1234... invalid).

### A-3 : PiiInputGuardrail wire — PASS

Wire correct dans `http.rs:1295-1315` : `default_input_chain().run()`
appele sur `submission.prompt`, retourne 400 si `!passed`. La
`default_input_chain()` (`guardrails.rs:136-138`) inclut
`PiiInputGuardrail::default()`.

**P2-REVIEW-C-1-S39 confirme** (carry 1/3 → 2/3) : pas de test
integration HTTP exercant le path de rejet PII. Les tests unitaires
couvrent la logique guardrail (`pii_redactor.rs:341-365`) mais le
wire HTTP est non-teste end-to-end.

---

## Track B — Architecture / CanaryRegistry

### B-1 : Persistence atomique — PASS

`persist()` Rust (`canary_registry.rs:136-148`) ecrit dans
`.json.tmp` puis `std::fs::rename()`. Pattern identique au Python
(`canary_registry.py:223-234`). Atomicite correcte.

### B-2 : Freshness classification — PASS

Seuils identiques : canary 30/45 jours (`canary_registry.rs:15-16`
vs `canary_registry.py:73-79`), duress ack 2/7 jours. Classification
`classify_canary_age()` identique (fresh / warn / stale aux memes
bornes). Note : le plan d'audit citait "seuils 7/14/30j" qui
refere a une version anterieure du Python — les seuils actuels
30/45 sont corrects dans les deux impls.

### B-3 : Coerce canary payload — PASS

Rename `v→version` identique (`canary_registry.rs:257-268` vs
`canary_registry.py:338-358`). Payload sans "v" ni "version" echoue
correctement via serde deserialization (champ `version: u32`
obligatoire sans `#[serde(default)]`). Test `coerce_canary_payload_missing_field`
(`canary_registry.rs:418-421`) confirme.

---

## Track C — Tests / coverage

### C-1 : Delta tests 968→991 (+23) — PASS

14 tests PII + 9 tests CanaryRegistry. Tous testent du comportement
reel (pas de stubs, pas de mocks). PII utilise `PiiRedactor::default()`,
CanaryRegistry utilise `TempDir` avec vrai I/O fichier.

### C-2 : PII tests 14/14, 7 patterns — PASS

| Pattern | Test(s) | Lignes |
|---|---|---|
| email | redact_email, has_pii_true | L265, L329 |
| phone | redact_phone | L274 |
| SSN | redact_ssn | L282 |
| credit card | redact_credit_card + luhn_valid/invalid | L288, L255, L260 |
| IP | redact_ipv4 | L304 |
| URL | redact_url | L312 |
| IBAN | redact_iban | L320 |

7/7 patterns couverts, chacun >=1 test.

### C-3 : CanaryRegistry tests 9/9 — PASS

Observe (canary + duress), freshness (fresh + stale + unknown),
health, persist + reload, coerce (ok + error). 5 domaines couverts
par 9 tests.

---

## Track D — Process / meta

### D-1 : G8 preflights 3/3 — PASS

- `sprint39_phase_A_preflight.md` — HEAD `9a2cebd`, EXECUTE
- `sprint39_phase_B_preflight.md` — HEAD `ff919b4`, EXECUTE
- `sprint39_phase_C_preflight.md` — HEAD `905e3f5`, EXECUTE

Coherence preflight → code verifiee : chaque preflight precede
chronologiquement le commit feat correspondant.

### D-2 : Scope cuts 12/12 — PASS

Fichiers modifies S39 (`ff919b4^..09d490f`) : `pii_redactor.rs`,
`canary_registry.rs`, `guardrails.rs`, `lib.rs`, `http.rs`,
`runtime.rs` + planning + Cargo. Grep pour scope cuts (onnx, ort,
gossip_sync, canary_input, redundancy, re-run, honeypot, coordinator
suppression, kudos debit/stake, sliding_window, distributed) :
0 violation.

### D-3 : P2-REVIEW-A-1-S37 launcher logging — PASS

Test `launcher_log_dir_matches_daemon_log_dir` dans
`crates/nexus-launcher/src/main.rs:594-612` couvre l'invariant
complet (tempdir + env var override). Resolution complete, carry
ferme.

---

## Track E — Dependencies

### E-1 : regex crate — PASS

Workspace dep `regex = "1"` (`Cargo.toml:133`). Resolue regex
1.12.3 (`Cargo.lock:6589-6592`). Pas d'advisory RustSec connue
(derniere RUSTSEC-2022-0013 fixee en 1.5.5).

### E-2 : Pas de dep transitive inattendue — PASS

Phase A : +1 dep directe (`regex = { workspace = true }`) dans
`crates/nexus-coordinator-rs/Cargo.toml`, exactement comme prevu D1.
Phases B et C : 0 dep nouvelle. `regex` existait deja comme dep
transitive, maintenant pinne comme directe.

---

## Track F — Doc coherence

### F-1 : HARDENING_ROADMAP compteurs — PASS

`docs/security/HARDENING_ROADMAP.md:3` : 991 Rust / ~1994 total.
Conforme verification.md §3.

### F-2 : CLAUDE.md etat actuel — PASS

S39 CLOSED, compteurs 991 Rust / ~1994 total, carries S40 listes.
Coherent avec verification.md et audit plan.

### F-3 : Phase review files 3/3 — PASS

4 review files presents (A + B + C + bonus D). Tous verdict PASS.

### F-4 : Phase preflight files 3/3 — PASS

3 preflight files presents (A + B + C). Tous verdict EXECUTE.

---

## Synthese findings

| # | Severity | Track | Description | Action |
|---|---|---|---|---|
| P2-REVIEW-C-1-S39 | P2 | A-3 | HTTP integration test PII wire absent | carry 2/3 |
| P3-AUDIT-A-1-S39 | P3 | A-1 | URL regex single-quote divergence Rust vs Python | cosmetic |

### Carries mis a jour S40

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 6+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P3-grammar executor | 3/3+ | defer Rust pipeline S40 |
| P3-watermark executor | 3/3+ | defer Rust pipeline S40 |
| P2-REVIEW-A-1-S38 result_event_tx dead code | 2/3 | wire gossip S40+ |
| P2-REVIEW-B-1-S38 substring O(n*m) | 2/3 | perf post-v1.0 |
| P2-REVIEW-C-1-S38 chain Arc singleton | 2/3 | perf post-v1.0 |
| P3-AUDIT-A-2b-S38 lowercase divergence | 2/3 | doc post-v1.0 |
| P2-REVIEW-A-1-S39 Tripwire vs Mutation | 1/3 | trait extension post-v1.0 |
| P2-REVIEW-B-1-S39 warn threshold | 1/3 | seuil cadence post-v1.0 |
| P2-REVIEW-C-1-S39 HTTP integration tests | 2/3 | confirm audit A-3 |
| P3-REVIEW-A-2-S39 LOC kickoff | 1/3 | cosmetic |
| P3-REVIEW-B-2-S39 persist error silent | 1/3 | robustness post-v1.0 |
| P3-AUDIT-A-1-S39 URL single-quote | 1/3 | cosmetic |
