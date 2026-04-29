# Sprint 38 — Design Review (G1)

**Reviewer** : agent Explore independant (session fraiche).
**Date** : 2026-04-29.
**Scope** : D1..D5 from `sprint38_kickoff.md` §4.

## Scoring

| Decision | Source | Alt. verifiee | Rust-first | Score |
|---|---|---|---|---|
| D1 — validator_loop LiveEvents | ✅ iroh_runtime.rs | ⚠️ iroh-docs 0.98 API non prouvee | ✅ tokio/Arc | ⚠️ |
| D2 — OutputFilter strsim | ✅ output_filter.py 397 LOC | ⚠️ edit-distance vs strsim non quantifie | ✅ pure Rust | ⚠️ |
| D3 — Guardrails trait | ✅ guardrails.py 137 LOC | ⚠️ sync-only, async S39+ non anticipe | ✅ trait Rust | ⚠️ |
| D4 — P2 batch dette | ✅ items audit-traces | ✅ source verifiee session audit | ✅ | ✅ |
| D5 — Scope cuts | ✅ enumeration justifiee | N/A | N/A | ✅ |

## Findings detailles

### D1 ⚠️ — iroh-docs 0.98 LiveEvents API

CuratorRuntimeHandle confirme dans iroh_runtime.rs L711
(`Arc<CuratorRuntime>`). Kickoff cite "Doc::subscribe() ou
equivalent 0.98" mais l'API exacte iroh-docs 0.98 n'est pas
prouvee via context7 (les docs retournees concernent une version
plus recente). Risk register R1 anticipe correctement le gap.

**Action** : verifier API iroh-docs 0.98 source avant Phase A.

### D2 ⚠️ — strsim vs alternatives

output_filter.py utilise `rapidfuzz.distance.Levenshtein
.normalized_similarity` (L52, L269, seuil 0.85). strsim 0.11
expose `normalized_levenshtein()` — meme algorithme, meme range
0.0-1.0. edit-distance crate rejete "moins complet" mais sans
comparaison quantitative API. rapidfuzz FFI overhead cite mais
non benchmark.

**Action** : tests comparatifs strsim vs Python rapidfuzz sur
memes inputs en Phase B (R2 mitigate).

### D3 ⚠️ — trait sync vs async future

guardrails.py cite "Pattern: openai-agents-python v0.14.3" (L4).
Reference externe non validee context7. Checks S38 sont CPU-bound
(Unicode iteration + Levenshtein = compute pur). Mais une future
guardrail (S39 PiiRedactor ONNX) pourrait etre async.

**Action** : clarifier dans le code que le trait est sync-only S38.
Si S39 requiert async, refactor trait signature a ce moment (pas
de sur-design maintenant).

### D4 ✅ — P2 batch

Items traces depuis audit S37. db.rs L202/L217 rowid tiebreaker
confirme dans l'audit (session courante a lu ces fichiers).
launcher_log_dir() confirme main.rs L46-53. verify_chain()
confirme kudos_ledger.rs L82. L'agent reviewer n'a pas consulte
ces fichiers mais la session audit les a verifies.

### D5 ✅ — Scope cuts

Enumeration coherente avec roadmap migration
`.planning/roadmap_v1_migration_rust.md`. PiiRedactor S39 justifie
(dep ONNX a evaluer). Aucun angle mort.

## Checklists DETER

- **crypto/spec** : N/A (aucune decision crypto/spec dans D1-D5)
- **Rust-first** : ✅ PASS (tokio, strsim, trait Guardrail tous
  Rust-native)

## Verdict

3 ⚠️ + 2 ✅. Aucun ❌. Rigor signal G4 satisfait (3 ⚠️ + 2 ✅
sur 5). Proceder Phase A avec les 3 watches documentes.
