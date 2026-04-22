# Sprint 25 — Design Review Board (G1)

**Date** : 2026-04-22
**Reviewer** : agent Explore independant (session fraiche)
**Kickoff review** : sprint25_kickoff.md §4

---

## Scoring

| Decision | Verdict | Detail |
|---|---|---|
| D1 — Key rotation | ✅ | Sources validees (SSH key rotation 2025, Keybase sigchain, Matrix cross-signing). Alternatives CA/WoT/timer comparees et rejetees avec justification factuelle. ed25519-dalek 2.1 stable, gossip iroh natif. |
| D2 — C3 handoffs | ✅ | Sources recentes (OpenAI agents-python 0.14.3, ASP.NET Core .NET 10 2026, Kong 2026). Input/output distinction confirmee par pattern industrie. Alternatives single-chain/per-guardrail/AOP rejetees. |
| D3 — D5 capabilities | ✅ | Windows MIL confirme actif 2026 (Google Project Zero Feb 2026). microsoft/sudo pattern valide. SHA-256 pre-quantum acceptable S25. Semgrep rule testing disponible. Alternatives config.toml/env var/feature flags rejetees avec threat analysis. |
| D4 — P2 cleanup batch | ✅ | tokio::select! valide pour concurrent DNS (cancel-safe read-only futures). TLS per-endpoint justifie. Pattern quarantine alerting existant S21. |
| D5 — Scope management | ✅ | arti-client 0.37.0 pre-1.0 = defer Tor S26 justifie. MCP vuln avril 2026 confirmee (OX Security, The Register 2026-04-16, ~200k serveurs RCE STDIO). D5 prereq B2 = correct. |

---

## Detail par decision

### D1 — Key rotation ceremony

- **Sources validees** : SSH key rotation (old key signs new key,
  best practices 2025), Keybase device revocation (sigchain signed
  revocation statement), Matrix cross-signing (master key rotation).
- **Alternatives** : CA model (centralise, contredit Day 0 #1),
  Web of Trust PGP (N^2 non-scalable, confirme par sources 2026),
  timer rotation (inutile pre-v1.0).
- **DETER crypto** : ed25519-dalek 2.1, pas de rotation native =
  correct (rotation = protocole applicatif). Gossip iroh 0.97 natif.
- **DETER Rust-first** : KeyRotationAnnouncement struct Rust,
  gossip subscribe Rust, PyO3 binding coord-side = OK.

### D2 — C3 handoffs StageGuardrailMap

- **Sources validees** : openai-agents-python v0.14.3 (input/output
  guardrail distinction native), ASP.NET Core 10 (request/response
  bidirectionnel), Kong (request-transformer + response-transformer
  plugins separes).
- **Alternatives** : single global chain (PII ≠ output safety, no-ops
  forces), per-guardrail stage annotation (couplage), AOP (S24 D2
  deja rejete).
- **Note** : migration backward-compat input_chain → stage_guards
  requiert tests regression.

### D3 — D5 capabilities gate-off-by-default

- **Sources validees** : microsoft/sudo pattern (OFF par defaut,
  admin-only), Windows MIL confirme Google Project Zero Feb 2026,
  Semgrep custom rules testing 2026.
- **Alternatives** : config.toml (user-mode editable, pas de gate
  admin), env var (ephemere, heritage), feature flags compile-time
  (rebuild requis).
- **Security** : integrity_hash SHA-256 anti-tamper, fallback all-OFF,
  admin check double (IsUserAnAdmin + MIL High Windows).

### D4 — P2 cleanup batch

- **tokio::select!** : cancel-safe pour DNS read-only futures.
  Pattern "first-responder" ideal pour concurrent DoH+DoT.
  Alternative tokio::spawn+JoinSet = overkill pour 2 queries fixes.
- **TLS per-endpoint** : refactor dns_fallback.rs, chaque endpoint
  utilise son propre tls_name.
- **Quarantine alerting** : structlog + hook pattern existant S21.

### D5 — Scope management

- **Tor defer S26** : arti-client 0.37.0 pre-1.0, API instable.
- **B2 MCP defer S26** : D5 prereq livre ce sprint. MCP vuln avril
  2026 confirmee (OX Security, ~200k serveurs, STDIO RCE, Anthropic
  decline fix architectural). S26 B2 integrera mitigations.
- **Autres defers** : A3/C2/C5/RAG/rate budget → S26-S27, dependencies
  et scope clairement documentes.

---

## Verdict global

**0 ❌ + 0 ⚠️ + 5 ✅**. Toutes decisions etayees par sources 2025-2026
verifiables, alternatives concurrentes analysees, rationale de rejet
documente. Proceder Phase A.

**Recommandations pour implem** (non-bloquantes) :
1. D1 : monitoring ed25519-dalek 2.2+ si breaking changes
2. D2 : tests regression input_chain → stage_guards migration
3. D3 : Windows MIL testing sur Windows 11 25H2
4. D4 : timeout logging DNS concurrent pour debugging
5. D5 : monitoring MCP spec proposals post-sprint
