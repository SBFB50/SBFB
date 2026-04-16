# Sprint 20 — Carry-over summary (cap G7)

**Source** : `sprint19_audit_findings.md §Track A finding A-2` (P2)
qui identifie **5 carry-overs reels** S19 → S20 vs cap G7 = 2 documents
par `docs/claude/README.md §6.2.1`.

**Decision Day-0 D5** (kickoff S20) : reclassification explicite pour
ramener le nombre de carries reels a 2/2. Les 3 items ecartes ne
disparaissent pas — ils sont relocalises dans leur bon registre
(scope integre, tech debt long-terme, roadmap post-Gate-2).

---

## Carries confirmes Sprint 20 (cap G7 = 2/2)

| ID | Description | Owner | Deadline | Runbook / ref |
|---|---|---|---|---|
| **Meta-1** | Radicle-v1.0 activation tracking (re-carry S18→S19→S20) | FlowUP | Jour du tag v1.0 (probablement sprint release v1.0) | `docs/release/MIRROR_FALLBACK.md §3.1-3.8` (8 sous-sections self-contained, 5 secrets GHA, action `gsaslis/mirror-to-radicle@514707f3` v0.2.0) |
| **P2-2** | `.gitignore` NOISE coverage (untracked root pre-launch) | S20 | Chore open S20 (ce commit) | Pattern `*.exe`, `*.pdb`, `cc.json`, `/node_modules/` racine, `/site/`, `/docs/apps/` (listes via `sprint19_audit_findings.md §Track F`) |

---

## Items reclassifies (NON-carry)

### PoW runtime wire gossip subscribe

**Ancienne classification** (audit S19 A-2) : carry implicit S20 Phase 1.

**Nouvelle classification** : **scope S20 Phase C** (integre directement
au plan, pas carry-over).

**Rationale** : le commit Phase B S19 `edfc51b` body annonce deja
« integration intentionnellement differee Sprint 20+ pour (a) eviter
risk breakage flows gossip existants, (b) permettre rollout selectif
per-topic, (c) laisser S20 Phase 0 auditer la primitive + envelope +
caches isolement ». Le S20 Phase C est l'execution de cette promesse
— pas un scope creep mais l'incrementalite annoncee du pattern
primitive/wire/enforcement (`docs/rust/PATTERNS.md §Sprint 19.1`).

Consequence : Phase C S20 livre `subscribe_with_pow` wire au path
gossip subscribe runtime `crates/nexus-shell-daemon-core/src/iroh_
runtime.rs`, debloque definitivement S21 rate-limit
per-(consumer, worker, model).

### TLS pinning wire iroh T20

**Ancienne classification** (audit S19 A-2) : carry implicit S20+.

**Nouvelle classification** : **tech debt long-terme** documente dans
`docs/rust/PATTERNS.md §T20` + suivre issue upstream iroh.

**Rationale** : iroh 0.97 n'expose **pas** `relay::client::Client
Builder::custom_cert_verifier` publiquement (feature `#[cfg(test)]`
only, verifie context7 `/websites/rs_iroh` 2026-04-16). Fix path
= soit upstream PR (S20-S22 suivre — pas solo-implementable
rapide), soit fork connect path ~150 LOC (burden re-sync chaque
upgrade iroh). Les deux scenarios ne sont pas bloquants pour S20
big rock encryption at rest.

Impact : le `PinValidator` primitive S19 `540bb51` reste un
**defense-en-profondeur** pret a cabler des que iroh 0.98+ land.
Le runtime S20 reste WebPKI-only.

### DHT canary → enforcement strict

**Ancienne classification** (audit S19 A-2) : carry implicit.

**Nouvelle classification** : **post-Gate-2 design decision** reportee
dans `docs/security/HARDENING_ROADMAP.md §3` (section post-S22 a
etoffer en kickoff S23+).

**Rationale** : le canary opt-in via `SBFB_PKARR_RELAYS` env var est
le **bon niveau de maturite** pour pre-launch + Gate 2 beta publique.
Basculer enforcement strict par defaut requiert (1) un federation
Pkarr consolide (3+ relays stables operes non-SBFB, pas juste n0),
(2) runbook user-facing si pannes DHT, (3) telemetrie sur false-
positive rate canary. Ces prerequis sont post-Gate-2. Si les 3 sont
reunis au kickoff S23+, on ouvre la discussion de-bascule ; sinon le
canary opt-in reste le default long-terme (acceptable).

Impact : aucun work S20. Le wording `docs/claude/SPRINT_LOG.md +
CLAUDE.md + sprint18/19_verification.md` a deja ete fix dans le
commit `1af90b3` audit-P2 batch (claim-drift A-1 resolu).

---

## Memory carry-over (G6)

Items a fusionner manuellement depuis `sprint19_verification.md §5
Findings carry-over for memory` dans `nexus_grid_pivot.md` frontmatter
description a la fin du Sprint 20 wrap-up (pattern S18→S19 applique) :

- Sprint 19 CLOSED + audit gate leve (commits `1af90b3..3a7f0a3`).
- Eclipse-by-DHT defense **runtime-active sous config opt-in**
  `SBFB_PKARR_RELAYS` (canary, pas enforcement strict).
- PoW Hashcash primitive livree S19, runtime wire gossip subscribe
  Phase C S20.
- TLS SPKI cert pinning primitive livree S19, runtime wire iroh
  reporte T20 tech debt long-terme (iroh 0.97 limitation upstream).
- Delayed upload queue SQLite WAL, plaintext payload caveat en haut
  PATTERNS.md §P29.
- pkarr relay self-hosted docker image + ops doc §1-§7 publiable.
- Pattern Design Review Board G1 renforce : D-decisions crypto/spec
  doivent enumerer ≥1 alternative concurrente recente (≤6 mois)
  dans §Rejete, sinon ⚠️ automatique reviewer.

Pas d'autres zones rouges nouvelles. R-wasmtime-cve / R-iroh-audit /
R-libcrux-hax / R-pyodide-escape inchangees.

---

## Cap G7 status apres reclassification

- **Carries confirmes** : 2/2 (Meta-1 + gitignore)
- **Integres au scope S20** : 1 (PoW wire)
- **Tech debt long-terme** : 1 (TLS wire T20)
- **Reportes post-Gate-2** : 1 (DHT canary strict)

**Cap G7 respecte** strictement. Finding A-2 S19 audit = resolu via
decision Day-0 D5 kickoff S20.

---

**Maintenance** : ce fichier est archive `archive/v1.2/` a la cloture
S20 (meme commit que `sprint20_verification.md` + `sprint20_audit_
plan.md`). Le Meta-1 Radicle est re-carry S21 tant que le tag v1.0
n'est pas tire (pattern permanent jusqu'au go-live).
