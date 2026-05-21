# Sprint 68 — Design Review Board (G1)

**Date** : 2026-05-21
**Sprint** : 68 — Proof Cards + Publish Gate
**Reviewer** : self-review profond (auto-challenge systematique)

---

## Scoring

| D# | Titre | Source recente | Alternative | [DETER] Crypto | [DETER] Rust | Code verifie | Verdict |
|---|---|---|---|---|---|---|---|
| D1 | ProofCard struct + formule score | ok (W3C VC 2.0 mai 2025, OpenSSF Scorecard V5 2025, F-Droid verification 2025) | ok (3 alternatives rejetees : W3C VC complet, OpenSSF framework, score pondere) | N/A | ok (struct Rust dans coordinator-rs) | ok (proof_card.rs, provenance.rs, search.rs lus) | ok |
| D2 | Preview ephemere via blob-serve | ok (context7 axum ServeDir mai 2026, Pubky CLI pattern) | ok (3 alternatives : serveur separe, iroh-blobs persistent, Factory serveur local) | N/A | ok (axum handler, BlobStore P52) | ok (http.rs, runtime.rs, blob_serve.rs lus) | ok |
| D3 | Publish path Factory→daemon | ok (SYNTHESIS mai 2026, deploy.rs lu) | ok (3 alternatives : upload direct zip, API custom, Factory signe Ed25519) | N/A | ok (deploy.rs 363L existant, reqwest client Rust) | ok (deploy.rs, main.rs factory lus) | ok |
| D4 | Factory gates FG4-FG7 + dunce | warning (dunce release 2024, pas < 90j) | ok (3 alternatives : std canonicalize seul, soft-canonicalize, strict_path) | N/A | ok (dunce 6M downloads, template_engine.rs lu) | ok (template_engine.rs, secret_scanner.rs lus) | warning |
| D5 | Proof Card UI Browse | ok (F-Droid verification 2025, SYNTHESIS mai 2026) | ok (3 alternatives : app separee, badge seul, cache persistant) | N/A | N/A (frontend React) | ok (BrowsedProject.tsx, protocol.ts lus) | ok |

**Resume** : D1 ok, D2 ok, D3 ok, D4 warning, D5 ok.
Rigor signal G4 satisfait (1 warning sur 5).

---

## Findings

### D4 warning — dunce crate pas de source < 90 jours

**Detail** : Le crate `dunce` (v1.0.5, derniere release 2024-08-18) est
cite comme dependance pour la correction path traversal Windows. Aucune
source < 90 jours ne traite specifiquement de `dunce`. Les alternatives
(`soft-canonicalize` v0.1.3, `strict_path`, `path-security`) sont plus
recentes (2025) mais moins matures.

**Decision** : acknowledge — `dunce` est stable et resout un probleme
inherent a l'OS (UNC paths `\\?\`), pas un probleme qui evolue. Le crate
a 6M+ downloads, 0 CVE connu, et le probleme qu'il resout (Windows
`std::fs::canonicalize` retourne des UNC paths) est un comportement OS
fixe depuis Windows XP. La maturite compense l'absence de source recente.
Si un CVE apparait sur `dunce`, `soft-canonicalize` avec feature `dunce`
est le fallback immediat.

---

## Checklist [DETER] (si applicable)

### Crypto/spec
- [x] Pas de D-choice touchant crypto ou spec cryptographique dans ce sprint
- [x] ProofCard formula est un calcul local deterministe, pas un protocole crypto
- [x] Ed25519 et BLAKE3 inchanges (utilisation existante dans deploy + provenance)

### Rust-first
- [x] D1 : ProofCard struct Rust dans nexus-coordinator-rs
- [x] D2 : Preview handler axum Rust dans le daemon
- [x] D3 : Publish client reqwest Rust dans sbfb-factory
- [x] D4 : Gates Rust dans sbfb-factory + dep dunce (Rust)
- [x] Aucune alternative non-Rust consideree pour les composants backend
- Exemptions : D5 frontend React (composant UI, pas backend)
