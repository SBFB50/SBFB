# Roadmap lisible

Source principale :
`.planning/roadmap_v1_migration_rust.md` + `docs/claude/SPRINT_LOG.md`.

## Etat au 2026-05-02 (post-Sprint 51)

Le projet est **100% Rust+Frontend**. Zero Python, zero legacy.
Le daemon Rust est le seul coordinator. 1199 tests Rust, 250 Vitest.
51 sprints livres. ~1455 tests total.

## Ce qui est fait

| Zone | Sprint | Etat |
|---|---|---|
| Pivot P2P + iroh | S0-S13 | DONE |
| Deploy verifie Keyoxide | S14 | DONE |
| Loopback hardening | S16-S20 | DONE |
| Sybil-resistance 3 couches | S21-S22 | DONE |
| Ephemeral workers + guardrails | S23-S24 | DONE |
| Key rotation + capabilities | S25-S26 | DONE |
| SynthID watermark + Couche 3 | S27 | DONE |
| FROST warrant canary | S30 | DONE |
| iroh 0.98 upgrade | S32 | DONE |
| Multi-node readiness | S33 | DONE |
| Launcher UX cross-platform | S34 | DONE |
| Coordinator Rust (foundation) | S35-S36 | DONE |
| Hash-chain KudosLedger | S37 | DONE |
| Migration Python→Rust Tier 1-4 | S38-S41 | DONE |
| Integration tests 36+ routes | S42-S47 | DONE |
| Carries batch + dette pair | S48 | DONE |
| Coordinator lifecycle + CLI | S49 | DONE |
| Suppression Python | S50 | DONE |
| Suppression legacy + CI cleanup | S51 | DONE |

## Ce qui reste avant v1.0

| Sprint | Theme | Statut |
|---|---|---|
| S52 | Binaires release cross-platform + VPS deploy | A OUVRIR |
| S53 | Smoke test E2E reseau + app hello-world | PREVU |
| S54 | Polish UX + docs user-facing + installer | PREVU |
| **v1.0** | **Tag + go-live** | **CIBLE** |

## Themes post-v1.0

1. Babel : corpus + traduction + liseuse
2. App store / trust / review / vote
3. Capabilities apps avancees
4. Privacy modes
5. Contributions family Sybil v2
6. Radicle (quand mature)

## Decisions structurantes

SBFB doit rester :

- local-first
- P2P
- signe
- verifiable
- anti-capture
- sans dependance GAFAM comme strategie principale

## ADR a formaliser (post-v1.0)

- `ADR-001-pas-de-gafam.md`
- `ADR-002-babel-shelf-offline.md`
- `ADR-003-liseuse-libre-prioritaire.md`
- `ADR-004-corpus-domaine-public-juridictions.md`
- `ADR-005-sync-liseuse-sans-compte-proprietaire.md`
