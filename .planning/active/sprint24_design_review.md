# Sprint 24 — Design Review Board Report

**Reviewer** : agent Explore independant (session fraiche 2026-04-21)
**Scoring** : D1 ⚠️, D2 ⚠️, D3 ⚠️, D4 ⚠️, D5 ✅
**Verdict** : 0 ❌ + 4 ⚠️ + 1 ✅. Proceder Phase A.

---

## D1 — Guardrails refactor (⚠️)

Source verifiee recente (openai-agents-python v0.14.3, 2026-04-21)
mais alternatives competitives non documentees :
- **LangChain** : middleware hooks `beforeAgent/afterAgent` avec
  state-based jump-to semantics. Architecture fondamentalement
  differente (LanggraphStateGraph vs decorateurs).
- **NVIDIA NeMo Guardrails v0.20.0** (jan 2026, Apache-2.0) :
  Colang DSL-based rails. Plus riche pour compliance conversationnelle.
- **Guardrails AI** (PyPI avril 2026) : Guard composable validators
  depuis Hub registry. `Guard` structurellement similaire a notre
  `GuardrailChain`.

Checklist [DETER] crypto/spec : pas de cite >=1 alternative <6 mois.

---

## D2 — TaskDispatchHooks (⚠️)

Pattern lifecycle hooks standard (Kubernetes postStart/preStop, AWS
CodeDeploy, Ansible AWX, Symfony EventDispatcher). Mais set de 5
events non justifie contre standards task dispatcher :
- Pas de comparaison Celery/APScheduler/RQ lifecycle.
- Events potentiellement manquants : `on_task_assigned`,
  `on_worker_timeout`, `on_retry`.
- Fire-and-forget OK pour observabilite mais pas de discussion sur
  extensions futures veto/cancellation.

---

## D3 — Re-run sampling (⚠️)

Taux 1-5% pragmatique mais non source :
- BOINC utilise replication + quorum voting, pas spot-check %.
- Folding@Home utilise adaptive sampling (conformations), pas re-run.
- Aucune reference academique BFT qui recommande un taux spot-check.
- Pas de justification statistique (intervalles de confiance absents).
- Complementarite avec majority voting S23 documentee (point positif).

---

## D4 — DNS fallback hickory-resolver (⚠️)

Crate mature et correct mais alternatives non documentees :
- **doh_dns** : queries DoH Cloudflare/Google, API plus simple.
- **reqwest custom resolver** : trait `Resolve` + DoH custom, integre
  stack HTTP existante.
- **rust-doh-proxy** : implementation production RFC 8484.

Checklist [DETER] Rust-first : pas de cite >=1 alternative Rust <6 mois.

---

## D5 — Scope cut (✅)

Deferral justifie : budget ~2400 LOC, key rotation independant B1/A1,
C3 handoffs depend fondations non testees. Cap G7 2/2 documente.
