# Sprint 27 Phase C — preflight G8

Date : 2026-04-25 | HEAD : `7bb656b` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, research before code, OSS prior art obligatoire (G10), context7 systematique pour deps/specs
- feedback_context7_systematic.md : context7 avant toute dep tierce — rusqlite 0.32 deja workspace, pas de nouvelle dep externe
- sprint14_keyoxide_decision.md : deploy from source, multi-forge zero OAuth — Phase C multi-forge cross-validate est alignee avec cette decision
- Tensions plan vs memory : aucune

## Scans (all clean)

- S1a OSS prior art : 3 recherches (web-of-trust P2P identity, git signature parser Rust, multi-forge Sybil resistance). Sequoia-WoT (Rust, OpenPGP) confirme le modele delegation chain + trust decay per hop comme pattern WoT standard (flow network / max-flow authentication). Aucune lib Rust existante pour parser `git log --show-signature` (git2-rs n'expose pas la verification de signature nativement). TIWD (Rust P2P) utilise "Proof of Presence" (temps+participation) = alignee Couche 1 AgeWitness. Recherches Sybil-resistance academiques (Hades ACSAC'23, ZK-PoI, MPC-TLS Semaphore) visent des contextes blockchain/DeFi plus complexes que le modele SBFB. APPROACH-ALIGNED — clean
- S1b deps : rusqlite 0.32 (bundled) deja workspace, 0 CVE 2026 sur rustsec.org. chrono 0.4 deja workspace. serde-big-array 0.5, serde_jcs 0.2 deja workspace. 0 nouvelle dep ajoutee — clean
- S2 historiques : 4 groupes de fichiers scannes (attestations/delegation.rs, attestations/, shell-daemon-core/src/, CONTRIBUTOR_ATTESTATION_RFC.md). 1 hit non pertinent (S25 Phase B key rotation sur shell-daemon-core — sujet different). Archive scan : mentions trust-web/multi-forge dans planning docs (D-5 S17 = dep ONG timing, pas rejection d'approche). Memory feedback : 0 "do not" sur trust-web/multi-forge/delegation — clean
- S3 threat model : FULL SCAN (3 nouveaux composants securite : ForgeParser, TrustCache, TrustWebManager). B-Sybil = threat principal adresse (score 4.0, T2+ pre-S19). Vecteurs nouveaux : (1) ForgeParser command injection via repo_path — mitigue par local-only path validation (clone coordinator-side S14, pas user-supplied URLs) ; (2) TrustCache stale data — mitigue par TTL 7j + invalidation API ; (3) Trust-web bootstrap centralisation FlowUP-only — acknowledge plan (ONG S28). (4) Gossip DelegationCert spam — mitigue par Ed25519 signature verification existante + PoW admission S19/S22. 0 regression sur threats existants. HARDENING_ROADMAP §3 S27 "Sybil mature" = aligne — clean
- S4 wire format : DOMAIN_DELEGATION_CERT_V1 (`canonical.rs:189`) preserved. DelegationCert struct etendu avec trust_level + scope (pre-launch redefinition v1, pas de bump). `expires_at_ts` deja present (le plan mentionne `valid_until` mais le champ existe deja — seuls `trust_level: u8` et `scope: DelegationScope` sont des ajouts nets). `#[serde(default)]` sur nouveaux champs = legitime runtime tolerance (anciens certs JSON sans ces champs deserializent). Day 0 preservees. Pre-launch protocol respectee — clean

## Note implementation

Le plan §C.5 mentionne `valid_until: Option<DateTime<Utc>>` mais DelegationCert possede deja `expires_at_ts: Option<i64>` (meme semantique). Pas de duplication — garder `expires_at_ts` existant, ajouter uniquement `trust_level` et `scope` comme nouveaux champs.

## Telemetrie preflight

- Duree totale : ~4m
- S1a : ~2m / 3 recherches WebSearch / finding : APPROACH-ALIGNED (Sequoia-WoT, TIWD, academique)
- S1b : ~30s / 3 libs scannees (rusqlite, chrono, serde-big-array) / finding : clean
- S2 : ~30s / 4 groupes fichiers + archive + memory / finding : clean
- S3 : full / ~30s / HARDENING_ROADMAP aligne, 0 regression, 4 vecteurs nouveaux mitiges
- S4 : full (DelegationCert wire format touche) / ~30s / VERSION=v1 preserved, Day 0 OK

## Action

Proceder code phase C.
