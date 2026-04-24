---
written: 2026-04-20  # S22 hors-sprint post Phase B `e9530c2`
last_validated: 2026-04-20
status: implemented (S24 Phase B `feat(sprint24)` — ABC + GuardrailChain + 4 adapters + dispatcher integration)
triggers_revalidate:
  - "openai-agents-python release > 0.7.0 (GuardrailFunctionOutput API breaking)"
  - "Nouveau checker PII/output/canary introduit hors pattern (drift ad-hoc)"
  - "B1 refactor S23 landing complet (post-sprint verification)"
---

# Guardrails architecture — pipeline déclaratif composable

## 1. Scope et motivation

Ce document spécifie l'architecture **guardrails pipeline unifié**,
pattern adopté de `openai/openai-agents-python` (`@input_guardrail /
@output_guardrail` decorators + `GuardrailFunctionOutput(output_info,
tripwire_triggered: bool)` + exceptions typées). Feature B1 du
cluster B dans `.planning/research/S23_to_S29_agents_sudo_
integration_matrix.md`.

### 1.1 État actuel post-S22 Phase B

La chaîne de checkers actuelle dans `packages/nexus-coordinator/src/
nexus_coordinator/dispatcher.py` empile **6 primitives ad-hoc**
livrées entre S21 et S22 :

1. **PiiRedactor coord-side** (S21 Phase C `23abb11`) —
   `packages/nexus-coordinator/src/nexus_coordinator/pii_redactor.py`
   Presidio + GLiNERRecognizer + local InvisibleText scanner +
   EED echo Levenshtein 0.85.
2. **Output filter** (S21 Phase D `f830579`) — `validator.py:30-247`
   scanner InvisibleText + EED echo post-task.
3. **Quarantine queue** (S21 Phase D `f830579`) — `quarantine_
   queue.py` SQLite WAL queue + Typer CLI.
4. **Rate-limit engine** (S22 Phase A `0bc499f`) — `crates/nexus-
   worker-core/src/rate_limit.rs` GCRA `governor 0.10.2` wire engine
   + hot-reload.
5. **PII redactor iframe-side** (S21 Phase B `d5b0035` + S22
   Phase B `e9530c2` decoder) — `web/src/sdk/pii/` GLiNER span-logits
   decoder + fallback regex.
6. **Watermark canari-input** (S22 Phase E à venir) — `canary_
   input.py` primitive consumer 1/N Ed25519-signed rotatable.

Chaque primitive est **appelée en séquence** dans `dispatcher.py
scan_and_execute_tasks()` avec logique `if result.X: alert` /
`if result.Y: quarantine` éparse. Pas de contrat unifié, pas de
composition déclarative, pas d'exception typée par type de tripwire.

### 1.2 Objectifs du refactor B1

- **Contrat unifié** : chaque checker implémente un trait/ABC
  `Guardrail` avec `check(ctx, input_or_output) -> GuardrailOutcome`.
- **Pipeline déclaratif** : `GuardrailChain([guardrail_1,
  guardrail_2, ...])` avec ordre stable + short-circuit sur
  tripwire.
- **Exceptions typées** : `InputTripwire(guardrail_name, evidence)`
  + `OutputTripwire(guardrail_name, evidence)` = 1 point de
  branching dans dispatcher vs N if/else actuels.
- **Testabilité contract** : chaque primitive testée contre contrat
  `Guardrail` (test reuse cross-primitive).
- **Iframe bridge exposition** : SDK expose `bridge.guardrailsCheck()`
  via P24 whitelist extend → app iframe peut enchaîner ses propres
  guardrails custom avant de soumettre à coord.
- **Observability native** : chaque `Guardrail.check()` émet hook
  (A1 consumer S24) + trace span (A2 consumer S29) = visibilité
  end-to-end auditeur externe.

### 1.3 Comparative analysis — alternatives considérées

| Framework | Pattern | Adapté SBFB ? | Raison |
|---|---|---|---|
| openai-agents-python v0.14.3 | `@input_guardrail` / tripwire decorator | **Oui (retenu)** | ABC simple, outcome typé, short-circuit explicite, pipeline ordonné |
| LangChain callback hooks | State graph middleware (`next()` cascade) | Non | Architecture state-graph, inadaptée à notre dispatcher linéaire ; ordre implicite, error handling complexe |
| NeMo Guardrails (Colang DSL) | Domain-specific language | Non | Over-engineered pour 4 primitives connues ; runtime DSL lourd, courbe d'apprentissage |
| Guardrails AI (Guard) | Pipeline structurellement similaire | Structurellement confirmé | Confirme le pattern GuardrailChain, mais wrapper OpenAI-spécifique ; notre implem. reste agnostique |

(G1 review finding D1-G1-1 acknowledged — cf. `sprint24_kickoff.md §4.5`)

## 2. Contrat `Guardrail`

### 2.1 Python ABC (coord-side primary)

```python
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Generic, TypeVar

T = TypeVar("T")  # Input ou Output type

@dataclass(frozen=True)
class GuardrailOutcome(Generic[T]):
    """Résultat de check.
    - `passed` True = rien à signaler, pipeline continue.
    - `passed` False + `tripwire` True = abort pipeline,
      exception typée levée.
    - `passed` False + `tripwire` False = warning log seul,
      pipeline continue.
    - `mutated_value` optionnel = si le checker a transformé
      l'input/output (ex: PiiRedactor qui remplace PII par
      [REDACTED]), cette valeur remplace l'original downstream.
    """
    passed: bool
    tripwire: bool
    guardrail_name: str
    evidence: dict         # Structured data for audit (pas user-facing)
    mutated_value: T | None = None
    latency_ms: float = 0.0


class Guardrail(ABC, Generic[T]):
    """Contrat unifié pour tout checker du pipeline."""

    @property
    @abstractmethod
    def name(self) -> str:
        ...

    @property
    def direction(self) -> str:
        """'input' | 'output' — contrainte check direction."""
        return "input"

    @abstractmethod
    async def check(self, ctx: "GuardrailContext", value: T) -> GuardrailOutcome[T]:
        ...

    async def on_tripwire(self, ctx: "GuardrailContext", outcome: GuardrailOutcome[T]) -> None:
        """Hook optionnel. Default = no-op. Exemples d'implem :
        - QuarantineGuard : enqueue in SQLite pour validator humain
        - RateLimitGuard : increment counter per-(consumer, worker)
        - CanaryGuard : log divergence + emit trace event Ed25519-signed
        """
        ...
```

### 2.2 Rust trait (worker-side secondary — rate_limit Gate)

Le rate-limit gate engine `crates/nexus-worker-core/src/rate_limit.rs`
reste côté Rust (performance critical path). Abstraction
symétrique :

```rust
pub trait Guardrail: Send + Sync {
    fn name(&self) -> &'static str;
    fn direction(&self) -> GuardrailDirection;

    async fn check(
        &self,
        ctx: &GuardrailContext,
        value: GuardrailValue,
    ) -> Result<GuardrailOutcome, GuardrailError>;

    async fn on_tripwire(&self, ctx: &GuardrailContext, outcome: &GuardrailOutcome) {
        // default no-op
    }
}
```

PyO3 binding dans `nexus-core-py` expose le trait aux hooks Python
(pattern S22 Phase C `verify_contributor_attestation` PyO3 binding).

## 3. Pipeline `GuardrailChain`

```python
from typing import Sequence

class GuardrailChain:
    def __init__(self, guardrails: Sequence[Guardrail]):
        self._guardrails = list(guardrails)

    async def check_input(self, ctx: GuardrailContext, input: T) -> T:
        """Exécute chain input-direction. Retourne input final
        (éventuellement muté par redactors). Lève `InputTripwire`
        si un guardrail tripwire."""
        value = input
        for g in self._guardrails:
            if g.direction != "input":
                continue
            outcome = await g.check(ctx, value)
            if outcome.tripwire:
                await g.on_tripwire(ctx, outcome)
                raise InputTripwire(
                    guardrail_name=g.name,
                    evidence=outcome.evidence,
                )
            if outcome.mutated_value is not None:
                value = outcome.mutated_value
        return value

    async def check_output(self, ctx: GuardrailContext, output: T) -> T:
        """Symétrique check_input pour direction output."""
        ...
```

## 4. Retrofit plan — 6 primitives

Chaque primitive existante devient une instance `Guardrail`. Le
refactor est **non-breaking wire** (zero `*_VERSION` bump).

| Primitive actuelle | Class refactor | Direction | Mutation | Tripwire triggers |
|---|---|---|---|---|
| PiiRedactor coord (S21C) | `PiiRedactGuardrail` | input | oui (remplace PII → [REDACTED]) | non (jamais fatal, mute always) |
| Output filter (S21D) | `OutputFilterGuardrail` | output | non | oui (InvisibleText detected OR EED echo > 0.85) |
| Quarantine queue (S21D) | `QuarantineSinkGuardrail` | output | non | oui (consumer de tripwires upstream, enqueue via on_tripwire) |
| Rate-limit engine (S22A) | `RateLimitGuardrail` (Rust) | input | non | oui (tuple saturé) |
| PII iframe (S21B+S22B) | (reste iframe-side, exposé via bridge P24) | input iframe | oui | non |
| Canary input watermark (S22E) | `CanaryInputGuardrail` | input | oui (injecte prompt canari 1/N) | non (observational only) |

### 4.1 Pipeline coord-side déclaratif cible

```python
dispatcher_guardrails = GuardrailChain([
    RateLimitGuardrail(rate_limiter),            # fast-fail tuple saturé
    PiiRedactGuardrail(presidio_analyzer),       # mute PII input
    CanaryInputGuardrail(canary_signer, rate=0.01),  # 1% injection
    # ... task dispatched to worker ...
    OutputFilterGuardrail(invisible_text_scanner, eed_threshold=0.85),
    QuarantineSinkGuardrail(quarantine_queue),   # consume tripwires
])
```

### 4.2 Ordre d'exécution

- **Input direction** (ordre déterministe) : rate_limit → pii_redact
  → canary_input.
- **Output direction** : output_filter → quarantine_sink.
- Rate_limit **en premier** = fast-fail avant coût Presidio (économie
  CPU).
- Quarantine_sink **en dernier** = consume les tripwires des
  guardrails précédents (quarantine = destination, pas checker
  propre).

## 5. Exposition iframe bridge (SDK wrapper)

Extension P24 whitelist (pattern additif per `PATTERNS.md §P24`) :

Méthode bridge actuelle (S13) : `task_submit`, `storage_get`,
`storage_set`, `pii_redact` (S21 Phase B).

Méthode bridge ajoutée S23 : **`guardrails_check`**.

Payload :

```typescript
interface GuardrailsCheckRequest {
  direction: "input" | "output";
  value: string;
  policy?: {
    // Override policy optionnel.
    // Défaut: DEFAULT_POLICY app iframe (pii_redact strict).
    pii_entities?: string[];
    threshold?: number;
    skip_guardrails?: string[];  // whitelist disable
  };
}

interface GuardrailsCheckResponse {
  passed: boolean;
  tripwire: boolean;
  guardrail_name?: string;       // si tripwire, quel checker
  evidence?: Record<string, unknown>;
  mutated_value?: string;        // si mutation, nouveau value
  findings: PiiFinding[];        // pattern S21 Phase B reuse
}
```

Permet apps iframe chain leurs propres guardrails **avant** de
soumettre à coord (défense en profondeur supplémentaire côté client).

## 6. Observability native

Chaque `Guardrail.check()` émet automatiquement :

1. **Hook A1** `TaskDispatchHooks.on_guardrail_check_start / on_
   guardrail_check_end` (S24 consumer). Permet dashboards per-guardrail
   metrics (latency, tripwire rate, mutation rate).
2. **Trace span A2** `TraceProvider` avec attributes `guardrail.name`,
   `guardrail.direction`, `guardrail.tripwire`, `guardrail.latency_ms`
   (S29 consumer). W3C Trace Context propagation cross-process.
3. **Audit event A3** `nexus-events-core` sur tripwire event type
   `guardrail_tripwire` (S25 consumer). Visible SIEM entreprise.

## 7. Sprint integration

**S22 Phase F** (maintenant, ce commit) : création de ce document
+ amendement HARDENING §3 S23 (item net-new "guardrails refactor").

**S23 chore hors-sprint** (optionnel, pattern `88eee23`) : si
arbitrage user favorable, draft initial trait `Guardrail` + squelette
`GuardrailChain` sans migration primitives (poser les types).

**S23 Phase dédiée** (item net-new HARDENING §3 S23) :

- Phase A : trait/ABC `Guardrail` + `GuardrailChain` Python + PyO3
  Rust binding. Tests contract.
- Phase B : retrofit PiiRedactor (coord-side). Tests regression
  S21 Phase C préservés.
- Phase C : retrofit OutputFilter + Quarantine. Tests S21 Phase D
  préservés.
- Phase D : retrofit RateLimit (Rust-side trait impl). Tests S22
  Phase A préservés.
- Phase E : wire dispatcher.py utiliser `GuardrailChain` vs
  primitives directes. Tests integration.
- Phase F : wrap + verification + `CanaryInputGuardrail` wire
  (provenant S22 Phase E à venir).

**S23 estimation retrospective** : ~800 LOC refactor + ~200 LOC
tests contract = ~1000 LOC. Arbitrage scope au kickoff S23.

**S24 consumer** (A1 `TaskDispatchHooks` landing) : refactor B1
devient testable observability hook-by-hook.

**S29 consumer** (A2 TraceProvider + B4 residual risk doc) :
guardrails chain attestable audit externe Cure53/ToB + residual
risk par guardrail disabled documenté dans `THREAT_MODEL §9`.

## 8. Contre-indications

- **Performance rate-limit hot-path** : RateLimitGuardrail trait Rust
  doit rester zero-cost vs impl actuelle S22 Phase A. Bench
  obligatoire pre-commit S23 Phase D (garder GCRA tuple saturé check
  < 1µs).
- **Backward compat non-wire** : un test obligatoire `contract_test_
  pii_redact_guardrail_matches_legacy()` vérifie que
  `PiiRedactGuardrail.check(text)` produit byte-identique output à
  l'ancienne `PiiRedactor.redact(text)`. Aucune régression behavior.
- **Exposition iframe bridge méthode P24 whitelist extend** : nouveau
  vecteur attaque potentiel si le guardrails_check endpoint n'est pas
  correctement rate-limité. Même rate-limit wire S22 Phase A
  applicable (already testé).

## 9. Références

- `.planning/research/S23_to_S29_agents_sudo_integration_matrix.md §1 Cluster B`
- `docs/security/HARDENING_ROADMAP.md §3 S23` (amendement item net-new)
- `packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py`
  (callsite refactor)
- `packages/nexus-coordinator/src/nexus_coordinator/pii_redactor.py`
  (primitive actuelle S21 Phase C)
- `packages/nexus-coordinator/src/nexus_coordinator/validator.py`
  (output filter S21 Phase D)
- `packages/nexus-coordinator/src/nexus_coordinator/quarantine_queue.py`
  (S21 Phase D)
- `crates/nexus-worker-core/src/rate_limit.rs` (S22 Phase A)
- `docs/shell/PATTERNS.md §P24` (bridge whitelist extend pattern)
- Source externe : [openai-agents-python guardrails.md](
  https://github.com/openai/openai-agents-python/blob/main/docs/guardrails.md)
  (pattern `@input_guardrail / @output_guardrail` + tripwire)
