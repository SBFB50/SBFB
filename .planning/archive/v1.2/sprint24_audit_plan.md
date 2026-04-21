# Sprint 24 — Audit Plan pour Sprint 25

**Redige** : 2026-04-21
**Sprint audite** : Sprint 24 (Guardrails pipeline refactor +
TaskDispatchHooks lifecycle + re-run sampling divergence + DNS fallback DHT)
**Commits** : de `6cb6e72` (chore open S24) a Phase F inclus.

---

## 1. Checklist d'audit (Phase 0 Sprint 25)

L'auditeur independant (nouvel agent, session fraiche) doit :

1. Lire ce document + `sprint24_kickoff.md` D1-D5 + `sprint24_plan.md`
2. Verifier chaque commit Phase A-F contre le plan (scope, fichiers,
   tests annonces vs reels)
3. Scanner pour regressions, dead code, security smells
4. Emettre `sprint24_audit_findings.md` avec verdict PASS / CONDITIONAL
   PASS / FAIL + items P0-P3

---

## 2. Dimensions a auditer

### Track A — Guardrails pipeline (Phase B)

- [ ] ABC `Guardrail.check()` return type strictement `GuardrailOutcome`
- [ ] `GuardrailChain` short-circuit : premier tripwire stop l'execution
- [ ] Ordering test : chain execute les guardrails dans l'ordre insere
- [ ] `InputTripwire` / `OutputTripwire` exceptions heritent correctement
- [ ] 4 adapters wrappent les primitives existantes sans modifier leur
  logique interne (PiiRedactor, OutputFilter, QuarantineQueue,
  CanaryInputInjector)
- [ ] `dispatcher.py` : `input_chain.run()` remplace tous les if/else
  PII precedents, pas de logique dupliquee restante
- [ ] Backward compat : dispatcher sans chain fonctionne comme avant

### Track B — TaskDispatchHooks (Phase C)

- [ ] ABC `DispatchHook` non-instanciable directement
- [ ] `HookRunner` fire-and-forget : exception hook → log, pas crash
- [ ] 5 events types tous fires aux bons points dans dispatcher.py +
  validator.py (verifier chaque point d'injection)
- [ ] `HookContext` contient task_id + timestamp + metadata dict
- [ ] Trait Rust `DispatchHook` dyn-safe (test compile present)
- [ ] Pas de PyO3 binding Rust→Python S24 (scope cut respecte)

### Track C — Re-run sampling (Phase D)

- [ ] `RerunSampler` rate boundaries : 0.0 → aucun re-run, 1.0 →
  tous re-run, >1.0 → clamp + warning
- [ ] `DivergenceScorer` hash comparison BLAKE3 deterministic
- [ ] Re-run task a un task_id distinct de l'original (pas de collision)
- [ ] Mismatch → `QuarantineQueue.enqueue()` appele (pas silencieux)
- [ ] Config TOML sample parse valide
- [ ] `DivergenceScorer` registered comme hook `on_result_received` (pas
  d'autre event)

### Track D — DNS fallback (Phase E)

- [ ] `DnsFallbackResolver` n'intervient QUE si pkarr quorum echoue
  (pas de concurrent resolution par defaut)
- [ ] DoH endpoint config : TLS name per-endpoint ou per-group coherent
  (cf. P2-E-1 carry finding)
- [ ] DoT endpoint config : port 853 par defaut, TLS certificate
  validation active
- [ ] TXT record parsing pkarr format → `PkarrSignedPacket` roundtrip
- [ ] `browse_aggregator` : fallback declenche sur `AllFailed` uniquement
  (pas sur timeout partiel avec quorum reussi)
- [ ] `hickory-resolver` dependency : features activees minimales, pas
  de feature inutile (tokio-runtime + dns-over-https-rustls +
  dns-over-rustls)
- [ ] DOMAIN_FRONTING_DESIGN.md : design-only, aucun code implementation
  present (scope cut respecte)

### Track E — P2 cleanup batch (Phase A)

- [ ] `pow.rs` exponent saturation : `exponent.min(i32::MAX as u64)`
  avant cast (pas de wrap around)
- [ ] KudosLedger public API : `get_total_kudos()` retourne somme
  correcte, `get_top_contributors(n)` retourne top n tries
- [ ] pynacl dep floor `>= 1.6.2` present dans pyproject.toml
- [ ] PATTERNS.md §P35 + §P36 presents et complets
- [ ] docs/shell/PATTERNS.md §PyO3 rebuild procedure documentee

### Track F — Process / meta

- [ ] G8 preflight systematique 6/6 phases (documents presents)
- [ ] Phase reviews A-E presentes (review documents)
- [ ] Commit bodies contiennent delta tests cumule + scope cuts
- [ ] Pas de dead code introduit (unused imports, unreachable branches)
- [ ] Pre-launch protocol respecte (VERSION = 1 partout, 0 tolerant
  decoder multi-version)
- [ ] SPDX / ruff F401 clean

---

## 3. Items connus (carry-over Sprint 24)

| ID | Severite | Description | Source |
|---|---|---|---|
| P2-E-1 | P2 | `build_resolver` per-endpoint TLS name support | Phase E review |
| P2-E-2 | P2 | Concurrent DoH+DoT fallback strategy | Phase E review |
| P2-D-1 | P2 | Redundancy persistence SQLite (in-memory pre-v1.0) | S23 carry |
| P2-D-2 | P2 | Quarantine curator alerting (queue-only) | S23 carry |
| P2-E-1-iroh | P2 | iroh neighborhood enrichment | S23 carry |
| T-NN+2 | P3 | iframe Rust-wasm (PATTERNS §P34, triggers inactive) | S22 carry |

---

## 4. Zones a risque (recommandation d'attention supplementaire)

1. **GuardrailChain ordering semantics** : verifier que l'ordre
   d'insertion = ordre d'execution, et que le short-circuit sur
   tripwire est bien teste
2. **DivergenceScorer + QuarantineQueue integration** : verifier
   que le scorer appelle bien la queue existante (pas un mock ou
   stub en prod)
3. **DNS fallback trigger condition** : verifier que le fallback
   ne se declenche PAS si le quorum pkarr reussit (meme avec
   latence elevee)
4. **hickory-resolver feature surface** : verifier que les features
   TLS activees n'incluent pas de backend OpenSSL (rustls uniquement)

---

## 5. G8 pivot retrospective

0 DESIGN-CONFLICT S24. 6/6 phases EXECUTE (5 EXECUTE + 1 EXECUTE
Phase F wrap-up). Quatrieme sprint consecutif (S21-S24) sans
DESIGN-CONFLICT — G1 Design Review Board pre-gel suffisant depuis
S21 pour eliminer les conflits en amont.

S1a OSS prior art systematique depuis S24 (5 scans S1a documentes).
Aucun APPROACH-NAIVE detecte — plans alignes SOTA.
