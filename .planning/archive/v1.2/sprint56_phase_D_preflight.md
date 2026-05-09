# Sprint 56 Phase D — preflight G8

Date : 2026-05-09 | HEAD : `89f8a2f` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest technical option, research
  before code, no band-aid. Phase D = dette resolution, aligne.
- feedback_context7_systematic.md : context7 avant tout code
  touchant lib/API. Phase D touche tokio (timeout) et cargo
  (remap-path) — verifie ci-dessous S1b.

## Scans (all clean)

- S1a OSS prior art : Phase D est 3/5 items docs/process (forbid-
  deny doc, rustfmt drift investigation, lightcheck hook fix) +
  2/5 items code mineur (build timeout, remap-path-prefix).
  - Build timeout : pattern standard (Nix, Bazel, CI systems).
    tokio::time::timeout / std::process::Command::spawn + wait
    sont les approches idiomatiques Rust. APPROACH-ALIGNED.
  - Remap-path-prefix : standard reproducible-builds.org. Cargo
    supporte `--remap-path-prefix` nativement (rustc flag). Debian,
    NixOS, Fedora l'utilisent systematiquement. APPROACH-ALIGNED.
  - Items 1-3 : docs/process, pas de design a challenger — N/A.
- S1b deps : 0 nouvelle dep. tokio 1.40 deja workspace. rustfmt
  local 1.9.0 (rustc 1.95) vs CI 1.94 — difference documentee,
  pas de delta breaking. 0 CVE pertinent — clean.
- S2 historiques : 3 fichiers cibles scannes (PATTERNS.md,
  build_executor.rs, hooks/). git log grep DEVIATION/rejected :
  0 hit pertinent Phase D. Archive scan : 0 hit forbid/deny/
  rustfmt/lightcheck/build-timeout/remap. Memory feedback :
  0 constraint sur la zone dette — clean.
- S3 threat model : fast-path verified. Phase D ne cree aucun
  nouveau composant securite ni wire format. HARDENING_ROADMAP :
  0 ligne S56 specifique. Pas de regression T0-T5 — clean.
- S4 wire format : fast-path verified. Phase D ne touche pas
  canonical.rs, schemas/, ni *_VERSION. Day 0 D1-D4 preservees
  (D4 = selection batch dette, Phase D l'implemente). Pre-launch
  protocol non impacte — clean.

## Notes implementation

- `execute_build()` est actuellement sync (std::process::Command).
  Le plan dit `tokio::time::timeout()` mais la fonction n'est pas
  async. Approche : utiliser `Command::spawn()` + `child.wait()`
  avec thread timeout OU convertir en async. Pas de callers
  externes (MVP S55 non wire encore). Detail implementation, pas
  conflit design.
- lightcheck Check 4 (wire-format staging alert) : le faux-positif
  vient du grep `canonical\.rs|schemas/` qui matche les fichiers
  schemas/ meme pour un simple reformat import. Fix : affiner le
  grep pour detecter les changements de contenu, pas juste les
  noms de fichiers stages.
- forbid vs deny : worker-core lib.rs:50 `#[deny(unsafe_code)]`
  + lib.rs:51 `#[cfg_attr(test, allow(unsafe_code))]`. Rationale
  deja en commentaire lib.rs:45-49. Pattern a documenter §P44.
- Dernier pattern PATTERNS.md : §P43 (Sprint 44).

## Telemetrie preflight

- Duree totale : ~3m
- S1a : ~1m / 2 patterns OSS consultes (build timeout, remap-path) / finding : clean (APPROACH-ALIGNED)
- S1b : ~30s / 1 lib scannee (tokio 1.40 workspace) / finding : clean
- S2 : ~30s / 3 fichiers, 0 commits pertinents / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action

Proceder code phase D.
