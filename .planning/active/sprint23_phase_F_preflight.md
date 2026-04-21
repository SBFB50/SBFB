# Sprint 23 Phase F — preflight G8

Date : 2026-04-21
HEAD : `56816ac`
Verdict : **EXECUTE plan-as-is**

## Scans

### S1 — SOTA 2026 vs design
- libs scannées : ed25519-dalek 2.1 (workspace), serde/serde_jcs (workspace)
- Phase F n'ajoute aucune nouvelle dépendance
- DelegationCert utilise la stack crypto existante (ed25519-dalek 2.1 + canonical JCS) déjà exercée par AgeWitness et ContributorAttestation
- Verdict : **clean**

### S2 — Décisions historiques traversées
- `git log --grep DEVIATION|rejected|scope-cut|threat-model -- crates/nexus-core-rs/src/attestations/ docs/fairness/` : 0 hit direct
- Archive scan : la seule DEVIATION S18 `04c9621` concerne le warrant canary auto-publisher (clé Ed25519 accessible scheduler = menace), pas les delegation certs
- Memory feedback : aucune règle "do not / never / reject" sur delegation/contribution families
- Verdict : **clean**

### S3 — Threat model coverage
- DelegationCert = design-only Couche 3, pas de runtime code
- HARDENING_ROADMAP §S23 confirme : "delegation cert format Rust struct — ~100 LOC (design-only)"
- Scope cuts plan §12 : "Couche 3 DelegationCert implem runtime → S25-S27" + "Contribution families implem code → post-v1.0 LT-3"
- Zéro risque de régression : struct + tests uniquement, pas de wire réseau, pas de dispatch, pas de signature automatisée
- Verdict : **clean**

### S4 — Wire format / pre-launch invariants
- `DELEGATION_CERT_VERSION` pas encore défini en code (commentaire design-only dans `attestations/mod.rs`)
- Phase F le crée à version 1 : conforme pre-launch protocol (librement redéfinissable avant tag v1.0)
- Ajout `DOMAIN_DELEGATION_CERT_V1` dans canonical.rs : pattern identique aux 12 domaines existants
- Day 0 D1-D5 non touchées par Phase F
- `*_VERSION` existants inchangés
- Verdict : **clean**

## Action

Procéder code Phase F. Aucun carry-over nécessaire.
