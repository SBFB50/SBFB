# Cockpit SBFB / Babel

But de cet espace AFFiNE : se reperer dans SBFB, Nexus et Babel sans relire
toute la documentation technique.

## Etat du projet (2026-05-02)

**Sprint 51 CLOSED** — workspace 100% Rust+Frontend, 0 Python/legacy.
~1455 tests (1199 Rust / 250 Vitest). ~3 sprints avant v1.0.

| Indicateur | Valeur |
|---|---|
| Sprints livres | 51 |
| Version | v1.2 (en cours) |
| Tests Rust | 1199 |
| Tests Frontend | 250 Vitest + 42 Playwright |
| LOC supprimes S50-S51 | ~102k (Python + legacy) |
| Carries actifs S52 | 5 (2 exemptions, 1 a 2/3, 2 NEW) |

## Carte mentale principale

- **SBFB** = protocole local + P2P pour apps, compute, provenance, confiance
- **Nexus** = implementation actuelle (daemon Rust, shell React, worker, bridge, 10 crates)
- **Babel** = future app vitrine SBFB (lecture libre, traduction, corpus, liseuse)

## Navigation

1. `01_ARCHITECTURE_SBFB.md` — les grands blocs et crates
2. `02_PROTOCOLE_DATA_FLOW.md` — comment une app circule jusqu'au worker
3. `03_SECURITE_STRIDE.md` — surfaces d'attaque et defenses
4. `04_BABEL_SUR_SBFB.md` — ou placer Babel dans le protocole
5. `05_ROADMAP.md` — prochaine logique produit et migration

## Fichiers sources repo

- `docs/architecture/WHITEBOARD_SBFB.md`
- `docs/security/THREAT_MODEL.md`
- `docs/security/PROCESS_ARCHITECTURE.md`
- `.planning/active/` (sprint en cours)
- `.planning/research/babel_translation_protocol.md`
- `docs/claude/SPRINT_LOG.md` (historique complet)

## Regle de travail

AFFiNE sert a penser et organiser. Le repo Git reste la source de verite.

Idee floue -> AFFiNE / whiteboard.
Decision stabilisee -> Markdown dans `docs/` ou `.planning/`.
Contrat technique -> types Rust, tests, threat model.
