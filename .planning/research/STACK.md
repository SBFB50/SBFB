# Technology Stack

**Project:** S73-S75 Code Factory + Babel Dogfood
**Researched:** 2026-05-18

## Recommended Stack

### S73 — Template Engine

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust (native generator) | 1.94+ | Template substitution + file generation | Coherent avec stack existant, pas de dependance Python |
| clap | 4.x | CLI `sbfb create` sous-commande | Deja en place dans le daemon |
| blake3 | 1.x | Template content hash verification | Deja en place dans le workspace |
| serde_json | 1.x | Parse SBFB.json v2, template.json | Deja en place |

### S74 — Broker

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| axum | 0.8.x | Routes HTTP /api/v1/factory/* | Deja en place dans le daemon |
| similar | 2.x | Diff computation (fichiers workspace) | Crate Rust mature pour diff text |
| serde_json | 1.x | Audit log JSONL | Simple, auditable |
| std::fs::canonicalize | stdlib | Path traversal prevention | Pas de dependance externe |

### S75 — Babel

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Vanilla JS + SBFBBridge | - | App reader Babel | Meme pattern qu'Explorer/Ideas |
| NLLB-200 via worker | - | Traduction (stretch goal) | Coherent avec compute distribue |
| ctranslate2 ou Ollama | TBD | Runtime NLLB-200 cote worker | A determiner selon maturite |
| JSON fixtures | - | Textes domaine public pre-charges | MVP sans dependance backend |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| giget | 2.x (npm) | Telecharger templates depuis repos Git | Si templates externes |
| Copier | 9.x (Python) | Template generation complexe | Seulement si >10 templates |
| Sandpack | 2.x (npm) | Browser code preview component | Si preview interactive requise |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Template engine | Rust natif | Copier (Python) | Ajoute dependance Python, overkill pour 3-5 templates |
| Template download | giget | degit | degit non maintenu, giget a 6x plus d'adoption |
| Diff engine | similar (Rust) | diff (npm) | Le broker est Rust, rester dans le meme ecosysteme |
| Preview | blob-serve existant | WebContainers | 20+ MB WASM, overkill, SBFB a deja blob-serve |
| Code edit | CSS diff viewer | Sandpack/Monaco | Trop lourd pour review de diff, pas d'edition live au MVP |
| Babel NLP | NLLB-200 via worker | Transformers.js browser | Iframe sandbox bloque IndexedDB et connect-src |

## Installation

```bash
# Rust workspace — nouvelle dependance pour S74
cargo add similar -p nexus-shell-daemon-core

# Frontend — pas de nouvelle dependance npm pour S73-S74
# S74 Phase C ajoute un composant DiffViewer en React pur (pas de lib)

# Babel (S75) — app standalone
# Pas de build system, vanilla JS comme Explorer/Ideas
```

## Sources

- giget: https://github.com/unjs/giget (3M DL/semaine, UnJS)
- similar: https://docs.rs/similar/ (Rust diff library)
- Copier: https://copier.readthedocs.io/ (alternative si templates complexes)
- Transformers.js NLLB-200: https://huggingface.co/Xenova/nllb-200-distilled-600M
