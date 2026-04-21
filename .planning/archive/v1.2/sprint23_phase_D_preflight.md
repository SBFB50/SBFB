# Sprint 23 Phase D — preflight G8

Date : 2026-04-20
HEAD : `6102dc2`
Verdict : EXECUTE plan-as-is

## Scans

### S1 — SOTA 2026 vs design

- libs scannees : blake3 Rust 1.8.4 (workspace dep nexus-core-rs)
- WebSearch RustSec + CVE blake3 2025-2026 : aucun advisory
- hashlib Python 3.13 : blake3 absent (blake2b/blake2s seulement).
  Pour le hash comparison coordinator-side, hashlib.sha256 suffit
  (egalite intra-coordinator, pas d'interop cross-language requis)
- Verdict : clean

### S2 — Decisions historiques traversees

- git log DEVIATION/rejected/scope-cut sur task.rs : commit `1aa6fed`
  (S16 C-1/C-2 wire is_open_source + estimates) — ajout de champs,
  pas de rejet du concept redundancy
- git log sur dispatcher.py : commit `23abb11` (S21 Phase C PII
  redactor), commit `94cccb2` (S18 Phase D wire-through) — aucun
  lien avec redundancy voting
- archive scan : redundancy voting explicitement DIFFERE S22→S23
  (HARDENING_ROADMAP : "DEFERRÉ S23, mitigue C-ResultSpoof tier T5,
  BOINC/F@H ont opéré 1-worker prod 20 ans, Gate 3 track"). Carry
  confirme dans sprint23_carry_summary.md §4
- memory feedback scan : aucun pattern "avoid"/"reject" sur
  redundancy/voting/majority
- Verdict : clean

### S3 — Threat model coverage

- threats mappes : C-ResultSpoof (T5 tier, Gate 3 scope) —
  redundancy voting ajoute une couche de detection, pas de
  regression sur defenses existantes (PoW S20/S23, Sybil S22,
  watermark S22)
- HARDENING_ROADMAP §S23 : "Redundancy voting Task.redundancy_factor
  (3 workers majority) — ~400 LOC (carry S22)" — explicitement prevu
- S24 dep confirmee : re-run sampling depend de S23 redundancy voting
  pour seuil detection
- regression flags : aucune
- Verdict : clean

### S4 — Wire format / pre-launch invariants

- `TASK_FORMAT_VERSION = 1` : reste a 1 (redefine v1 pre-launch)
- `redundancy_factor: u8` ajoute avec `#[serde(default)]` →
  deserialise a 0 quand omis (runtime tolerance, client minimal
  Python/JSON sans le champ = pas de parse error)
- canonical.rs : non modifie (canonical_bytes generique sur Task)
- Day 0 preservees : D1-D5 S23 non rebattues
- Decisions nexus_grid_pivot.md : aucune contradiction
- Verdict : clean

## Action

Procede code phase D. Aucun carry-over requis.
