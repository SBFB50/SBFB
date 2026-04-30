# Sprint 43 Phase B — preflight G8

Date : 2026-04-30 | HEAD : `130db9b` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — consent port = 1:1 pattern Python, files port = CAS simplifie (per-app Python SDK concept → daemon-level CAS)

## Scans (all clean)
- S1a OSS prior art : CAS file storage = pattern standard (IPFS, blob stores). Consent JSON file = trivial I/O. Pattern S42 deploy.rs/apps.rs etabli.
- S1b deps : axum multipart feature peut etre requis pour files upload. axum 0.8 deja workspace dep, feature ajout = mineur. 0 nouvelle dep externe.
- S2 historiques : 0 commit match sur files.py et consent.py. 0 conflit.
- S3 threat model : fast-path verified. Consent routes lisent/ecrivent un fichier local. Files CAS = stockage local. Pas de nouveau composant securite.
- S4 wire format : fast-path verified. 0 canonical.rs/schemas touche. ConsentConfig wire-compatible avec Rust worker existant.

## Action
Proceder code phase B.
