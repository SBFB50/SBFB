# Sprint 33 — Design Review Board (G1)

**Date review** : 2026-04-27
**Scope** : 5 decisions Day-0 (D1..D5) du kickoff Sprint 33 "multi-node readiness"
**Reviewer** : Agent Explore (read-only codebase scan)
**Mandate** : Scoring impact-based (source freshness + alternative verification)

---

## Scoring par decision

### D1 — CORS : explicit opt-in pour acces externe

**Status** : ✅ **BIEN-FONDEE**, sourcing exhaustif

- Source tower-http 0.6 : ✅ Presente dans Cargo.toml, version 0.6.8 en Cargo.lock (publie 2025-12-08 = ~140j, RECENTE)
- Source FastAPI CORSMiddleware : ✅ Utilisee production (app.py:121, ACTIVE)
- Implementation existante : ✅ `loopback_cors_layer()` en place depuis S16, `is_loopback_origin()` tests complets (http.rs:293-315, 7 tests unitaires)
- Alternatives comparees : ✅ WILDCARD rejete (Bearer + SOP incompatible = SOUND), env-var-only rejete (ergonomie = VALIDE), reverse-proxy-only rejete (setup complexity = FAIR). tower-http domine ecosysteme axum → pas d'alternative credible.

**Verdict** : ✅ **APPROVE**

### D2 — Multi-node test strategy : 2-daemon localhost

**Status** : ✅ **BIEN-FONDEE**, sourcing solide

- Source iroh 0.98 : ✅ iroh 0.98.1 en Cargo.lock (publie 2026-04-20 = 7j, TRES-RECENTE), multi-instance support documente
- Pattern e2e.rs : ✅ Existant (nexus-shell-daemon/tests/e2e.rs, nexus-worker/tests/e2e.rs), transposable 2 daemons
- Research documentee : ✅ sprint33_multinode_research.md §5.1 (rows 30-33)
- Alternatives comparees : ✅ Docker Compose rejete (CI dep), VPS rejete (flaky), mocking rejete (perd valeur)

**Verdict** : ✅ **APPROVE**

### D3 — Deploy packaging : systemd + install script Linux

**Status** : ⚠️ **SOURCING-INCOMPLET**, rationale acceptable

- Source Docker pattern : ✅ Existant (docker/pkarr-relay/Dockerfile)
- Source systemd : ⚠️ AUCUNE source codebase — aucun `.service` avant S33. Zero preuve experience projet avec templates systemd.
- Install script : ⚠️ Aucun script install-node existant
- Alternatives comparees : ✅ Docker rejete (overhead), Snap/Flatpak rejete (overhead packaging), Nix rejete (barrier entree)
- **Angle mort** : aucune exploration init-systems alternatifs (runit Alpine, OpenRC Gentoo, s6)

**Verdict** : ⚠️ **CONDITIONAL-APPROVE** — rationale valide (Ubuntu/Debian enonce), sourcing zero systemd codebase + zero exploration init-systems.

### D4 — P2-REVIEW-A-1 MANDATORY : hook LOC guard

**Status** : ⚠️ **SOURCING-INCOMPLET, SEMANTIC GAP**

- Documentation §6.7 : ✅ Trouvee (README.md:1022+), bien motivee, 3 cas empiriques (S14/S17/S18 vs S7/S18)
- Hook lightcheck.sh : ✅ Existe, 5 checks implementes
  - Check 3 (ligne 114-136) : LOC deviation WARN — grep `~[0-9]+ LOC`, calcul ACTUAL_LOC, threshold 2.5×. **NON-BLOQUANT** (WARN, continue exit 0).
- **SEMANTIC GAP** : D4 demande « bloquer le commit si patterns LOC detectes ». Check 3 existant est WARN-only (reactif, post-code). D4 = preventif (BLOCK exit 2, pre-code).

**Verdict** : ⚠️ **CONDITIONAL-APPROVE avec MUST-CLARIFY** — D4 implementable en ajoutant check 6 preventif (bloquer patterns `~NNN LOC` dans *_plan.md), mais kickoff ne clarifie pas check 3 (renforcer) vs check 6 (ajouter).

### D5 — Fail-fast checklist extension : rows multi-noeuds

**Status** : ✅ **BIEN-FONDEE**, scoping correct

- 4 rows proposees : ✅ Documentees sprint33_multinode_research.md §5.1
- Alternatives comparees : ✅ rows conditionnelles rejetees (risque oubli), scope creep rejetees (minimum viable)

**Verdict** : ✅ **APPROVE**

---

## Synthese scoring global

| D | Titre | Freshness | Alternatives | Sourcing | Verdict |
|---|---|---|---|---|---|
| D1 | CORS opt-in | ✅ 140j | ✅ 3 rejetees | ✅ Complet | **APPROVE** |
| D2 | 2-daemon localhost | ✅ 7j iroh 0.98 | ✅ 3 rejetees | ✅ Solide | **APPROVE** |
| D3 | systemd + install | ⚠️ 0 codebase | ⚠️ 0 init-alt | ⚠️ Incomplet | **COND** |
| D4 | LOC guard hook | ✅ Doc exist | ⚠️ Semantic gap | ⚠️ Incomplet | **COND** |
| D5 | Fail-fast rows | ✅ Doc exist | ✅ 2 rejetees | ✅ Clair | **APPROVE** |

---

## G4 Rigor signals

### G4-SIGNAL-1 : D4 — Semantic gap check 3 vs D4 intent (MEDIUM)

Hook check 3 (LOC deviation) est WARN non-bloquant. D4 decrit un BLOCK exit-2.
Phase A doit clarifier : renforcer check 3 (deviation → ERROR) ou ajouter
check 6 dedie (presence → BLOCK) ? Comment distinguer `~300 LOC` post-recherche
(retrospectif, §6.7 admis) vs `~300 LOC` amont (estimation, §6.7 interdit) ?

### G4-SIGNAL-2 : D3 — Zero systemd proof-of-concept (LOW-MEDIUM)

Aucune trace systemd/launchd/init-system codebase. D3 suppose expertise sans
pattern reference. Phase B doit inclure spike 30 min validation Ubuntu VM.

### G4-SIGNAL-3 : D3 — No init-system exploration (LOW)

Scope enonce "Ubuntu/Debian" mais aucune exploration runit/OpenRC/s6.
Acceptable si scope locked. Phase B commit body doit documenter :
"systemd templates Ubuntu/Debian 22.04+, autres init-systems = PR welcome post-v1.0."

---

## Verdict final G1

**Status** : ✅ **DESIGN-APPROVED-WITH-RESERVATIONS**

- 3/5 fully-green (D1, D2, D5)
- 2/5 conditional (D3, D4) — sourcing incomplet mais rationale acceptable
- No blocking findings ; 3 G4 signals (1 MEDIUM, 2 LOW) mitiges Phase A/B
- Research-first evident (sprint33_multinode_research.md)

**Cleared for Phase A** si Phase A plan integre les 3 G4 reservations.
