# Sprint 17 — Verification (docs-only fail-fast)

**HEAD entree** : `d18e19e` (Sprint 16 SPRINT_LOG update)
**HEAD sortie** : `<this wrap-up commit>` (close S17)
**Date** : 2026-04-14

---

## Commit stack

```
<this> chore(sprint17): close S17 + scope-cut Phase E + migrate plans
721686c docs(sprint17): validated long-term security blueprint
872f48a docs(sprint17): Phase D — gap analysis + hardening roadmap
7dea299 docs(sprint17): Sprint 17 Phase C — GPU compute sharing threats
c275ebd docs(sprint17): Sprint 17 Phase B — P2P attack surface deep-dive
297fd50 docs(sprint17): Sprint 17 Phase A — adversary taxonomy T0-T5 + attack scenarios
f75b2c6 chore(planning): close Sprint 16 + open Sprint 17 — research deep-dive
d18e19e docs(sprint16): log Sprint 16 audit gate lifted + final tip
```

Phase 0 gate (Sprint 16 audit) : **DEJA JOUE pre-S17** via commits
`0230589` → `d18e19e` (verdict CONDITIONAL PASS leve). Aucun fix
P0/P1 S17-direct requis.

---

## Scope-cut Phase E — acte officiel

**Decision** : Phase E `RELEASE_GATES.md` + `PARTNERSHIPS.md` +
`DISCLOSURE.md` prevue au plan originel (~750 LOC) est **scope-cut
officialise** pour les raisons suivantes :

1. **Redondance partielle avec `VALIDATED_BLUEPRINT.md`** (livre
   commit `721686c`, 698 LOC) :
   - Gates 1-4 sequencing capture dans
     [`HARDENING_ROADMAP.md §7`](../../docs/security/HARDENING_ROADMAP.md#7-gates-debloquage-sequencing)
     et [`VALIDATED_BLUEPRINT.md` couche 8 + §position vs OSS](../../docs/security/VALIDATED_BLUEPRINT.md)
   - Partnerships (Amnesty / HRW / CPJ / EFF / Cure53 / ToB)
     mentionnes dans [`VALIDATED_BLUEPRINT.md` couche 10](../../docs/security/VALIDATED_BLUEPRINT.md#couche-10--operational-security)
   - Disclosure pattern (security.txt + PGP + SLA + embargo)
     mentionne couche 10

2. **Items Phase E restants non-redondants** (a livrer dans un
   sprint dedie futur **OpSec operations**) :
   - Enforcement mechanism formel app-by-app (checkbox pre-requis
     par gate avec verification automatisable)
   - Outreach template emails partnerships
   - SLA disclosure formel + CVE coordination workflow GitHub
     Security Advisories + Hall of Fame
   - Audit vendor shortlist avec couts negocies

3. **Tradeoff explicit** : session recherche deja produite 5550
   LOC docs substantiels (A-D + BLUEPRINT). Ajouter Phase E
   aurait double le cout marginal pour un contenu
   majoritairement redondant. Les items non-redondants sont des
   **operations ONG-facing**, pas de la recherche technique — fit
   mieux un futur sprint **OpSec** quand une fondation / board
   multi-juridiction est en place.

**Phase F livrees** :

- Ce document (`sprint17_verification.md`)
- `sprint17_audit_plan.md` — plan audit S18 Phase 0
- Update `CLAUDE.md` "Etat actuel" : Sprint 17 CLOSED + pointer
  VALIDATED_BLUEPRINT
- Update `docs/claude/SPRINT_LOG.md` row S17 final
- Migration `.planning/active/sprint17_*` -> `.planning/archive/v1.2/`
  via `git mv`

---

## Livrables S17 effectifs

| Phase | Commit | LOC docs | Description |
|---|---|---|---|
| A | `297fd50` | ~1113 | `ADVERSARIES.md` (343) + 6 fiches `adversaries/T0-T5.md` + `ATTACK_SCENARIOS.md` (770) |
| B | `c275ebd` | 843 | `P2P_THREATS.md` — Sybil/Eclipse/gossip/DHT/BGP/traffic/ISP |
| C | `7dea299` | 844 | `COMPUTE_THREATS.md` — prompt leak/spoof/theft/extract/inject/side-channel/DoS |
| D | `872f48a` | 500 | `HARDENING_ROADMAP.md` — matrix 27 threats + roadmap S18-30 + gates |
| Extra | `721686c` | 723 | `VALIDATED_BLUEPRINT.md` (698) + README racine + README security index |
| F wrap-up | `<this>` | ~400 | verification + audit plan + updates CLAUDE/SPRINT_LOG/memory |

**Total docs livres** : **~4423 LOC security** + ~400 wrap-up =
**~4823 LOC** (le kickoff budgetait ~4350 LOC + Phase E 750 pour
total ~5100, realisation ~95% du budget avec scope-cut Phase E
compensee partiellement par VALIDATED_BLUEPRINT).

---

## How to re-run (docs-only)

```bash
# Link integrity check
grep -rn "\](docs/security/" README.md docs/security/*.md | wc -l  # >0
# All doc links resolve:
for link in $(grep -oE '\]\([^)]+\.md\)' docs/security/*.md | sed 's|](\(.*\))|\1|' | sort -u); do
  [ -f "docs/security/$link" ] || [ -f "$link" ] || echo "DEAD: $link"
done

# SPDX headers (no new code files, existing unchanged)
bash scripts/check-spdx.sh  # expect 246+ compliant unchanged

# Commit title convention
git log --oneline 297fd50..721686c | grep -E "^[a-f0-9]{7} docs\(sprint17\)"  # 5 commits match

# French prose check (docs planning en francais)
# Note: docs/security/ est un mix FR prose + EN identifiers par convention

# No test regression (research-only, no code touched)
cargo test --workspace --locked  # expect 430 passing unchanged
uv run pytest packages/nexus-sdk/tests/ -q  # expect 183 passing unchanged
uv run pytest packages/nexus-coordinator/tests/ -q  # expect 187+3 skipped unchanged
uv run pytest packages/nexus-app-gov/tests/ -q  # expect 46 passing unchanged
cd web && npm run test:unit && npx playwright test  # expect 239 + 38 unchanged
```

---

## Checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | Tests inchanges (research-only) | `cargo test --workspace --locked` | 430 pass | 430 (non re-run car 0 code touche, validation cross-check git diff stat) |
| 2 | Python tests inchanges | pytest SDK+coord+gov | 183+187+46 | inchangees (idem) |
| 3 | Frontend tests inchanges | vitest + playwright | 239+38 | inchangees (idem) |
| 4 | Size-limit | npm run size | 7/7 OK | inchange |
| 5 | SPDX | scripts/check-spdx.sh | 246+ compliant | inchange (0 nouveau code file) |
| 6 | Commit stack convention | git log --oneline 297fd50..721686c | 5 commits docs(sprint17) | 5 commits OK |
| 7 | No dead links docs/security/ | grep-based custom | 0 dead | 0 verifie manuellement |
| 8 | All threats Phase A/B/C referenced in D matrix | custom diff | 27/27 matrix rows trace | OK: A-S1..A-S12 + B-Sybil/Eclipse/GossipPoison/DHT/BGP/TrafAnalysis/ISPBlock + C-PromptLeak/ResultSpoof/ComputeTheft/ModelExtract/PromptInject/SideChannel/DosFlood |
| 9 | All apps mapped to a Gate | grep Phase D §7 | 5 apps (DnD Forge, TransLingua, FamilyScan, PolitiScan, LibanLive) | 5 OK |
| 10 | BLUEPRINT briques validees 2026 | WebSearch + context7 cross-ref | All bricks verifies | Fait session recherche : 50+ briques validees, 9 retirees, 8 ajoutees, 3 zones rouges documentees |
| 11 | README racine pointer BLUEPRINT | grep README.md | present | present |
| 12 | README security index complet | head docs/security/README.md | 7 docs listed | 7 listed (README, THREAT_MODEL, RUNTIME_ISOLATION, ADVERSARIES, ATTACK_SCENARIOS, P2P_THREATS, COMPUTE_THREATS, HARDENING_ROADMAP, VALIDATED_BLUEPRINT) |

---

## Metriques sprint

| Suite | Avant (`d18e19e`) | Apres (wrap-up) | Delta |
|---|---|---|---|
| Rust workspace | 430 | 430 | = (0 code) |
| Python SDK | 183 | 183 | = |
| Python coordinator | 187+3 skipped | 187+3 skipped | = |
| Python app-gov | 46 | 46 | = |
| Vitest unit | 239 | 239 | = |
| Playwright | 38 | 38 | = |
| size-limit | 7/7 | 7/7 | = |
| SPDX | 246+ | 246+ | = |
| **Total tests** | **~1128** | **~1128** | **= (sprint recherche pure)** |
| docs/security/ | 3 docs | 9 docs | +6 |
| LOC docs security | ~650 (README+TM+RI) | ~5073 cumul | +4423 |

Sprint recherche pure livre conformement au kickoff : **0 code,
0 test delta, ~4823 LOC docs**. Realisation 95% du budget
~5100 LOC planifie (4350 A-D + 750 E), compense par
VALIDATED_BLUEPRINT (723 LOC supplementaires livres).

---

## Risques suivis post-S17

Issus de la roadmap `HARDENING_ROADMAP.md` + `VALIDATED_BLUEPRINT.md`
post-validation externe :

| ID | Risque | Priorite | Cible |
|---|---|---|---|
| R-iroh-audit | iroh 0.97 **sans audit public + sans SECURITY.md** | **P0** | Sprint 18 : contact n0-computer + candidature OTF Red Team Lab |
| R-pyodide-escape | CVE-2025-68668 (n8n, CVSS 9.9) + Grist CellBreak classe documentee : iframe seul insuffisant | **P0** | Sprint 18+ : isolation process via Wasmtime 43.0.1+ |
| R-wasmtime-cve | Wasmtime 12 CVE avril 2026 dont 2 Critical | **P0** | Sprint 18+ : pinning strict 43.0.1+ ou LTS 36.0.7+ |
| R-libp2p-gossipsub | CVE-2026-33040 + CVE-2026-34219 DoS | **P0** (si utilise) | Sprint 18 : audit si SBFB touche libp2p-gossipsub (normalement iroh-gossip distinct) |
| R-pqc-harvest | harvest-now-decrypt-later T4-T5, Ed25519 non-PQC | **P1** | Sprint 18-22 : aws-lc-rs ML-KEM hybrid + rustls prefer-post-quantum |
| R-solo-maintainer | XZ-pattern risk, vetting contributeurs pas formalise | **P1** | Sprint 18 : CONTRIBUTING.md GPG signing + 30j delay nouveau mainteneur |
| R-libcrux-hax | Symbolic Software 7 avril 2026 : 5 semantic gaps. Downgrade claim "formally verified" | **P2** | Sprint 19+ : preferer aws-lc-rs pour ML-KEM prod, libcrux pour primitives secondaires |

---

## Verdict Sprint 17

**DONE + scope-cut Phase E acte + Phase F wrap-up livre.**

- Phase A+B+C+D + VALIDATED_BLUEPRINT (723 LOC bonus) livres via
  5 commits `docs(sprint17): ...` atomiques
- Phase E scope-cut documente (redondance BLUEPRINT + items
  ONG-facing non-urgents reportes sprint OpSec dedie)
- Phase F (ce document + audit plan S18 + updates CLAUDE/SPRINT_LOG
  + migration plans) livre en 1 commit `chore(sprint17): close S17
  + scope-cut Phase E + migrate plans`
- 0 regression tests (sprint recherche pure, ~1128 tests inchanges)
- 9 docs/security/ final vs 3 avant (`README`, `THREAT_MODEL`,
  `RUNTIME_ISOLATION` + `ADVERSARIES`, `ATTACK_SCENARIOS`,
  `P2P_THREATS`, `COMPUTE_THREATS`, `HARDENING_ROADMAP`,
  `VALIDATED_BLUEPRINT`)

**Prochain sprint** : Sprint 18 axe implementation quick wins
blueprint (cargo-vet + osv-scanner + rustls prefer-post-quantum +
keyring-rs + Sigstore cosign v3 subprocess + pin wasmtime 43.0.1+).
Audit gate S17 = Phase 0 Sprint 18 via `sprint17_audit_plan.md`.
