# Sprint 23 Phase E — preflight G8

Date : 2026-04-21
HEAD : `ab24080`
Verdict : **SCOPE-CUT-CONSISTENT**

## Scans

### S1 — SOTA 2026 vs design
- libs scannées : pynacl (Ed25519 canary peers), iroh-blobs (diagnostic endpoint)
- WebSearch CVE : `pynacl CVE 2026` → CVE-2025-69277 (libsodium `crypto_core_ed25519_is_valid_point` incomplete validation, CVSS 4.5 MEDIUM, fix pynacl >=1.6.2). Phase E utilise `nacl.signing.SigningKey()` (key generation) — code path distinct, non-overlapping attack surface.
- WebSearch RustSec : `rustsec advisory iroh 2026` → aucun advisory iroh. R-iroh-audit P0 déjà tracé (zone rouge permanente).
- RUSTSEC-2026-0075 libcrux-ed25519 all-zero key on RNG failure — non-applicable (projet utilise ed25519-dalek via iroh, pas libcrux-ed25519).
- Verdict : **1 finding non-bloquant** (CVE-2025-69277 MEDIUM, code path non-affecté, dep floor carry-over)

### S2 — Décisions historiques traversées
- git log scan : `git log --all --grep="DEVIATION|rejected|scope-cut|deliberate|threat-model" -- honeypot.py diagnostic.rs diagnostic.py fairness.py` → 0 hit (fichiers nouveaux)
- archive scan : `04c9621` S18 E2 warrant canary auto-publisher rejeté threat-model (clé Ed25519 maintainer-only). **NON-applicable** : Phase E concerne honeypot canary *peers* (dummy NodeId eclipse detection), pas warrant canary (transparency report signing). Confirmé non-applicable par 3 preflights antérieurs (S20 Phase E, S22 Phase C, S22 Phase F). Reverse-commit check : pas de reversion car décision toujours active et hors-scope.
- memory feedback scan : aucun pattern "never/avoid" applicable à honeypot eclipse ou fairness observability.
- Verdict : **clean**

### S3 — Threat model coverage
- threats mappés : Phase E couvre détection eclipse (T-eclipse/Sybil monitoring layer). Aligne HARDENING_ROADMAP mention "DHT canary enforcement strict reporté post-Gate-2" — Phase E livre **détection** (pas enforcement), cohérent avec le roadmap.
- regression flags : aucun. Phase E ajoute observabilité, ne retire aucune protection existante.
- HARDENING_ROADMAP gaps : aucun pré-requis S23 bloquant pour Phase E.
- Verdict : **clean**

### S4 — Wire format / pre-launch invariants
- `_VERSION` fields : aucun touché par Phase E. Plan confirme "No new wire format (canary peers use existing gossip publish)".
- canonical.rs : non touché.
- Day 0 D4 "Canary peer rotation + alert 80%/3rot" : Phase E implémente exactement D4.
- Pre-launch protocol : respecté (0 version bump).
- Verdict : **clean**

## Findings (SCOPE-CUT-CONSISTENT)

- **S1-pynacl-floor** : pynacl dep floor `>=1.5` permet installation de versions affectées par CVE-2025-69277. Carry-over recommandé S24 : bumper à `>=1.6.2`. Non-bloquant car code path Phase E (`SigningKey()`) distinct de la fonction vulnérable (`crypto_core_ed25519_is_valid_point`).

## Action

Procède code Phase E. Carry-over S1-pynacl-floor ajouté à sprint24_audit_plan.md track deps.
