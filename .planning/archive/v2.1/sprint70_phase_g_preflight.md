# Sprint 70 Phase G — preflight G8

Date : 2026-05-25 | HEAD : `6201f11` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- `feedback_approach.md` : pick deepest, no band-aid, research before code, G8 = mecanisme procedural. Phase G docs-only — pas de choix technique a deepener, le contrat RRV/Factory est la formalisation de decisions deja actees (D5, D6, D18, H7).
- `vision_model.md` : OpenBSD solo maintainer, pas startup. Phase G ne propose aucun partnership, funding, ou institutionnalisation. N/A.
- `feedback_context7_systematic.md` : context7 obligatoire avant code touchant lib/API. Phase G ne touche aucune lib, aucune API, aucun code. N/A.
- `fairness_vision.md` : kudos non-monetaire. Phase G ne touche pas les kudos. N/A.
- `feedback_kudos_non_monetary.md` : interdit cost/deposit/stake dans code+docs. Phase G ne mentionne pas les kudos. N/A.
- Tensions plan vs memory : aucune.

## Scans (all clean)

- S1a OSS prior art : 4 WebSearch executees (RBAC agent mapping, Factory/Operator separation, sprint verification checklist, RRV P2P protocol). Aucun projet OSS n'implemente un systeme RRV specifique — design maison SBFB coherent avec RBAC standard. L'approche table mapping modes → roles portables est APPROACH-ALIGNED avec les patterns RBAC 2026 (agent-as-principal, per-tool scoping). La separation Viewer (sandbox read-only) / Operator (local privileged) est APPROACH-ALIGNED avec separation of concerns classique et le pattern Operator (CNCF/agentic workflows) — clean.
- S1b deps : 0 lib ajoutee, 0 dep bumpee. Phase G est docs-only. Aucun Cargo.toml/package.json modifie — clean.
- S2 historiques : 6 commits pertinents lus (78e4413, 7b96abc, 9e8deb5, c4494a6, fa7ce72, e415034). Day 0 D5 "Contrat RRV/Factory (modes @ = alias roles portables)" inchangee. D6 "@protocole avant @dev avant @web" inchangee. D18 "Process Portable Complete avant RRV total" inchangee. H7 "RRV lit le process ; Factory le package. Aucun des deux ne devient autorite process." inchangee. CLAUDE.md recadrage PO 2026-05-22 confirme. Aucune reversion trouvee — clean.
- S3 threat model : FULL. Phase G docs-only ne cree aucune primitive, endpoint, wire format, ou surface exposee. 0 asset en jeu, 0 vecteur d'attaque, 0 gap, 0 regression T0-T5 — clean.
- S4 wire format : FULL. canonical.rs header lu + grep exhaustif. 7 constantes VERSION verifiees (toutes = 1, aucune touchee). Day 0 D1-D5 preservees. Pre-launch policy respectee — clean.

## S1a — OSS prior art deep analysis

### Probleme fonctionnel

"How do mature OSS projects document the contract between an agent orchestration tool (Factory) and a role-based verification/research system (RRV), with mode-to-role mapping?"

### Projets analyses

Phase G est docs-only — pas de primitive technique a comparer avec l'OSS. Le sujet est la documentation d'un contrat interne entre deux composants SBFB (Factory et RRV). Les recherches confirment que :

1. **RBAC pour agents IA (2026)** — 6 patterns identifies (agent-as-principal, on-behalf-of, per-tool scoping, per-tenant isolation, request-context binding, ABAC extensions). Le mapping modes → roles portables est coherent avec "agent-as-principal" + "per-tool scoping".
2. **Operator pattern (CNCF, Claude Code agentic workflows)** — la separation "operator plans, agents execute" est un pattern valide. La separation SBFB Viewer (read-only sandbox) / Operator (local privileged) suit ce pattern.
3. **Sprint verification** — les checklists fail-fast et Definition of Done sont des pratiques standard (Scrum.org, Atlassian).
4. **RRV specifique** — aucun standard OSS "RRV" n'existe. Le kickoff S70 l'a deja note (WebSearch source #5). Design maison SBFB.

### Tableau comparatif

| Aspect | Plan Phase G | RBAC agent 2026 | CNCF Operator | Scrum DoD |
|--------|-------------|-----------------|---------------|-----------|
| Mapping modes → roles | Table 5 modes → roles portables | agent-as-principal + per-tool scoping | N/A | N/A |
| Autorite | `.planning/active/` + gates | policy engine | CRD + controller | Definition of Done |
| Verification | fail-fast checklist | audit trail per-action | reconciliation loop | sprint review checklist |
| Separation Viewer/Operator | Viewer = sandbox read, Operator = local actions | N/A | Operator = controller | N/A |

### Finding S1a

- Classification : **APPROACH-ALIGNED**
- Evidence : pattern RBAC agent standard + CNCF Operator pattern + sprint verification standard
- Impact sur le plan : aucun

## S2 — Decision chain reconstruction

### Fichiers scannes
- CLAUDE.md : 107 commits total, 6 lus en detail (recents + RRV-pertinents)
- docs/agent/ : 4 fichiers existants, 0 commit historique touchant RRV_FACTORY_CONTRACT.md (fichier NEW)
- docs/claude/SPRINT_LOG.md : commits recents lus

### Decisions historiques trouvees

#### Decision 1 : D5 — Contrat RRV/Factory modes @ = alias roles portables

- Sprint 70 kickoff, sha `78e4413` : D5 gelee
  Body extrait : "D5 — Contrat RRV/Factory (modes @ = alias roles portables)"
- Sprint 70 plan v5, sha `c4494a6` : confirmee (plan §10 Phase G)
  Body extrait : "agentctl → sbfb-factory Rust, dashboard → Factory Viewer + Factory Operator split"
- Reverse-commit check :
  1. `git log --all --oneline 78e4413..HEAD -- docs/agent/` — aucun revert/undo
  2. `git log --all --grep=78e4413 --oneline` — 0 match
  3. Pas de candidate reversion
- Status : **active** (D5 gelee S70)
- Impact phase : aucun — Phase G **implemente** cette decision

#### Decision 2 : D6 — @protocole avant @dev avant @web

- Roadmap v4, sha `9e8deb5` : D6 documentee
  Body extrait : "Roadmap v4 D18 : S70 = Process Portable Complete + Gate 1 dogfood"
- CLAUDE.md (L267) : "@protocole d'abord, puis @dev, puis @web (v4 D6)"
- Reverse-commit check : aucune reversion
- Status : **active**
- Impact phase : aucun — Phase G documente le sequencing conforme

#### Decision 3 : H7 — RRV lit le process, Factory le package

- Roadmap v4 (L375) : "RRV lit le process ; Factory le package plus tard. Aucun des deux ne devient autorite process."
- Reverse-commit check : aucune reversion
- Status : **active**
- Impact phase : aucun — Phase G documente cette frontiere

#### Decision 4 : D17 — Superviseur optionnel, hooks backstop

- sha `17394b6` : D17 documentee
- CLAUDE.md : "Superviseur process optionnel, hooks = backstop mecanique (D17)"
- Reverse-commit check : aucune reversion
- Status : **active**
- Impact phase : aucun

### Memory constraints

- `feedback_approach.md` : "G8 = mecanisme procedural pour pick-deepest" — respecte (5 scans executes)
- `vision_model.md` : "pas startup/funding/fondation" — respecte (Phase G ne propose rien de tel)
- `nexus_grid_pivot.md` : "S70 = Process Portable Complete + Gate 1 dogfood" — Phase G ferme ce sprint
- `feedback_context7_systematic.md` : "context7 obligatoire avant code" — Phase G ne code pas

## S3 — Threat model analysis

### Primitive analysee : documentation contrat RRV/Factory + verification sprint

### Assets en jeu
- Aucun. Phase G docs-only : ne cree, ne modifie et n'expose aucun asset technique.

### Threat actors
- N/A. Aucune surface exposee.

### Attack vectors identifies
1. (a) Injection/forgery inputs : N/A (pas d'input)
2. (b) Replay/reorder messages : N/A (pas de message)
3. (c) DoS/resource exhaustion : N/A (pas de runtime)
4. (d) Information leakage : N/A (docs publiques du repo)
5. (e) Privilege escalation : N/A (pas de nouvelle surface)
6. (f) Supply chain : N/A (pas de dep)
7. (g) Temporal attacks : N/A (pas de concurrence)

### Mitigations existantes
- T0-T5 inchanges. Phase G ne modifie aucun composant couvert par le threat model.

### Gaps identifies
- Aucun.

### Regression check
- Aucune regression. Phase G ne diminue l'efficacite d'aucune mitigation existante.
- Phase G ne cree aucun nouveau vecteur non couvert.

### Verdict S3 : clean

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui (header + grep exhaustif, aucune struct touchee par Phase G)

### Structs verifiees

Phase G ne touche aucune struct. Verification exhaustive des constantes VERSION :

| Constante | Fichier:ligne | Valeur | Phase G touche ? |
|-----------|--------------|--------|-----------------|
| `CURATOR_LIST_FORMAT_VERSION` | `curator.rs:61` | 1 | Non |
| `BLOB_VERSION` | `keystore.rs:108` | 0x01 | Non |
| `KEY_ROTATION_FORMAT_VERSION` | `key_rotation.rs:32` | 1 | Non |
| `POW_FORMAT_VERSION` | `pow.rs:85` | 1 | Non |
| `TASK_FORMAT_VERSION` | `task.rs:61` | 1 | Non |
| `PIN_FILE_FORMAT_VERSION` | `tls_pinning.rs:102` | 1 | Non |
| `TASK_RESPONSE_VERSION` | `task_response.rs:48` | 1 | Non |

### Day 0 check
- D1-D5 sprint courant : aucune contredite. Phase G implemente D5.
- Decisions actees pivot.md : aucune contredite. D6, D17, D18, H7 respectees.

### Pre-launch policy
- *_VERSION = 1 : OK (aucune touchee)
- Pas de tolerant decoder multi-version : OK (pas de code)
- Pas de tests "legacy decode" zombie : OK (pas de tests)

## Telemetrie preflight (agent deep)

- Duree totale : ~5min
- S1a : ~2min / 0 projets OSS analyses en profondeur code source (phase docs-only, aucune primitive technique a comparer) / 0 fichiers source lus / 0 LOC reviewees / 0 context7 queries / 4 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : ~30s / 0 libs scannees / 0 CVE searches / finding : clean (0 dep)
- S2 : ~1min30s / 6 commits bodies lus / 0 archive files / 5 memory files lus / finding : clean
- S3 : FULL / ~30s / 7 categories evaluees / 0 gaps
- S4 : FULL / ~30s / 7 constants VERSION verifiees / canonical.rs lu (header + grep) : oui

## Action

Proceder code phase G. Phase docs-only — 5 fichiers a creer/mettre a jour :
1. `docs/agent/RRV_FACTORY_CONTRACT.md` (NEW)
2. `.planning/active/sprint70_verification.md` (NEW)
3. `.planning/active/sprint71_audit_plan.md` (NEW)
4. `CLAUDE.md` (UPDATE)
5. `docs/claude/SPRINT_LOG.md` (UPDATE)
