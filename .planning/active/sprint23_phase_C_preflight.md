# Sprint 23 Phase C — preflight G8

Date : 2026-04-20
HEAD : efe6ab3
Verdict : EXECUTE plan-as-is

## Scans

### S1 — SOTA 2026 vs design
- libs scannées : sha2, blake3, rand (workspace deps pow.rs)
- Pas de nouvelle dep externe requise (EscalatingPolicy = extension interne)
- relay_pow_policy.rs (S19) lit déjà TOML — Phase C étend le pattern
- CVE scan : aucun advisory sha2/blake3/rand 2026 critique
- Verdict : clean

### S2 — Décisions historiques traversées
- git log scan : 0 DEVIATION/rejected sur pow.rs, pow_gossip.rs, gossip.rs, relay_pow_policy.rs
- Archive scan : S19 D2 "Hashcash daté vs Equi-X 2023" = audited finding, décision RETENUE (audit clarity SHA256). Phase C ne change PAS la primitive, ajoute un policy layer au-dessus.
- S23 kickoff D2 bénit explicitement : "difficulté PoW augmente géométriquement (×2 par tranche de K tasks, configurable) par tuple (consumer_id, model_id)"
- Memory feedback : aucun pattern interdit sur PoW/escalation
- Verdict : clean

### S3 — Threat model coverage
- Threats mappés : T2 (resource exhaustion) + T3 (Sybil) renforcés par escalation
- AgeWitness (S22 C) = admission gate orthogonale, pas de régression
- HARDENING_ROADMAP §3 : pas de pre-requirement S23 bloquant pour PoW escalation
- Regression flags : aucun
- Verdict : clean

### S4 — Wire format / pre-launch invariants
- `POW_FORMAT_VERSION: u16 = 1` (pow.rs:85) — inchangé par Phase C
- `HashcashChallenge.difficulty: u32` existe déjà — Phase C compute dynamiquement au lieu de const `DEFAULT_DIFFICULTY_BITS`
- `EscalatingPolicy` = struct config interne (pas sérialisée sur le wire)
- `PowCounter` (Python) = SQLite locale coordinator, pas wire format
- Day 0 D2 S23 préservé : implémentation conforme au kickoff
- Pre-launch protocol : aucune violation
- Verdict : clean

## Action

Procède code Phase C. Aucun carry-over requis.
