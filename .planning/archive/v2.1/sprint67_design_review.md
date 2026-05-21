# Sprint 67 — Design Review Board (G1)

**Date** : 2026-05-20
**Sprint** : 67 — Factory Foundation
**Reviewer** : self-review profond (auto-challenge systematique)

---

## Scoring

| D# | Titre | Source recente | Alternative | [DETER] Crypto | [DETER] Rust | Code verifie | Verdict |
|---|---|---|---|---|---|---|---|
| D1 | FTS5 search @protocole | ok (sqlite.org/fts5.html, rusqlite build.rs 2026-05-20) | ok (Tantivy + MeiliSearch rejetes avec sources) | N/A | ok (rusqlite Rust-native, FTS5 via bundled) | ok (db.rs M14, public_feed.rs, http.rs routes lus) | ok |
| D2 | sbfb-manifest + SBFB.json v2 | ok (Backstage docs 2026-05-20, serde docs) | ok (inline validation + YAML rejetes) | N/A | ok (crate Rust pur) | ok (deploy.rs l.119-128 + l.543-557 lus) | ok |
| D3 | CuratorVouched feed ops | ok (SYNTHESIS 2026-05-19, SSB/AT Proto patterns) | ok (DashMap only + wire format dedie rejetes) | N/A | ok (Rust enum extension) | ok (public_feed.rs l.52-60 commentaire lu) | ok |
| D4 | Feed entries read paginee | ok (SYNTHESIS 2026-05-19, AT Proto/SSB pagination) | ok (SSE + GraphQL + status quo rejetes) | N/A | ok (axum handler Rust) | ok (db.rs l.780-826 get_feed_entries_after_seq lu) | ok |
| D5 | sbfb-factory CLI create + validate | warning | ok (Factory daemon + Copier + Tera rejetes) | N/A | ok (crate Rust CLI) | ok (deploy.rs, Cargo.toml workspace lus) | warning |

**Resume** : D1 ok, D2 ok, D3 ok, D4 ok, D5 warning.
Rigor signal G4 satisfait (1 warning sur 5).

---

## Findings

### D5 warning — Source Copier non verifiee en profondeur

**Detail** : la source Copier (copier.readthedocs.io) est citee
comme alternative rejetee mais la comparaison est au niveau du
pattern conceptuel (copie de fichiers + substitution de variables),
pas au niveau du code source Copier. Le moteur interne sbfb-factory
est ecrit from scratch et ne reproduit pas la logique Copier.
La source la plus recente pour le pattern template est
SYNTHESIS §3.3 (2026-05-19), qui recommande explicitement la
copie+substitution simple sans moteur tiers.

**Decision** : acknowledge — la comparaison est intentionnellement
au niveau du pattern, pas de l'implementation. Le code Copier est
Python et n'est pas transposable en Rust tel quel. Le moteur
interne est plus simple (< 200 lignes estimees). Le warning est
documente et n'impacte pas la decision retenue.

---

## Checklist [DETER] (si applicable)

### Crypto/spec

- [x] D-choices ne touchent PAS de crypto nouvelle. CuratorVouched
  utilise la signature Ed25519 existante du feed (pas de nouvelle
  primitive crypto). Factory provenance hash utilise BLAKE3
  existant. Pas de trigger [DETER] crypto.

### Rust-first

- [x] D1 cite rusqlite (Rust-native binding SQLite). FTS5 est
  une extension C compilee via bundled mais consommee via API Rust.
- [x] D2 sbfb-manifest est un crate Rust pur.
- [x] D3 extension enum Rust existant.
- [x] D4 handler axum Rust.
- [x] D5 sbfb-factory est un crate Rust CLI (clap derive).
- Exemptions : bridge search (TypeScript/Zod dans web/).
