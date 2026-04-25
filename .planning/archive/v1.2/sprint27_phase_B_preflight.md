# Sprint 27 Phase B — preflight G8

Date : 2026-04-25 | HEAD : `f8b8e2d` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : OSS prior art obligatoire (G10), pick deepest,
  context7 systematic avant code lib externe, planning adaptatif
- feedback_context7_systematic.md : context7 MCP non disponible cette
  session — compensé par WebSearch sur synthid-text, MarkLLM, hmac crate,
  RustSec advisory database. Pas de lib interne SBFB à query (module
  watermark est nouveau).

## Scans (all clean)

- S1a OSS prior art : 3 projets recherchés (google-deepmind/synthid-text
  Nature 2024, THU-BPM/MarkLLM EMNLP 2024, jwkirchenbauer/lm-watermarking),
  APPROACH-ALIGNED — le plan implémente une approche KGW-family avec
  améliorations BIRA-resistance (multi-token context window + secret
  rotatif par tâche) documentées dans la littérature. Tournament Sampling
  (SynthID complet) scope-cut S28+ justifié. Aucune lib production-grade
  compatible architecture split Rust-injector/Python-detector (synthid-text
  = JAX research, MarkLLM = HuggingFace-dependent). Custom implémentation
  justifiée — clean.
- S1b deps : 2 libs scannées. `sha2 0.10` déjà workspace dep. `hmac`
  crate à ajouter (RustCrypto standard, 0 CVE RustSec). Python `hmac`
  stdlib, 0 nouvelle dep. 0 delta — clean.
- S2 historiques : 5 fichiers scannés (llama_cpp.rs, canary_input.py,
  watermark_detector.py, archive S22 design review, COMPUTE_THREATS),
  12 commits scannés. S22 D4 Kirchenbauer rejeté pour INPUT watermark =
  cohérent avec Phase B rejet KGW pour OUTPUT. S20 Phase D llama_cpp.rs
  = extension sans conflit. COMPUTE_THREATS §4.5 label "Kirchenbauer
  2023" vs plan "SynthID-inspired" = doc drift description, plan
  autoritaire — clean.
- S3 threat model : FULL SCAN (nouveau composant sécurité WatermarkDetector
  + WatermarkInjector). C-ComputeTheft §4 couvert. HARDENING_ROADMAP S27
  aligné. Vecteur watermark_seed confidentiality mitigé par iroh-docs
  access control + rotation per-task. Plan §4.3 adresse conflit
  watermark+grammar (fallback). 0 régression existante — clean.
- S4 wire format : FULL SCAN (Task canonical touché — ajout champ
  `watermark_seed`). TASK_FORMAT_VERSION = 1 préservé. Redéfinition
  canonical v1 pre-launch légitime. `#[serde(default)]` runtime tolerance
  pour workers sans watermark. DOMAIN_TASK_V1 inchangé. Day 0 préservées.
  0 tolerant decoder multi-version — clean.

## Télémétrie preflight

- Durée totale : ~4m
- S1a : ~2m / 3 projets OSS consultés / finding : APPROACH-ALIGNED (clean)
- S1b : ~30s / 2 libs scannées / finding : clean
- S2 : ~30s / 12 commits scannés / finding : clean
- S3 : FULL / ~30s / C-ComputeTheft §4 mapped
- S4 : FULL / ~30s / TASK_FORMAT_VERSION=1 preserved

## Action

Procéder code Phase B.
