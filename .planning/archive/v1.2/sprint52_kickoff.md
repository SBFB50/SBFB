# Sprint 52 — Kickoff (dette pair + docs legacy cleanup + release validation)

**Ecrit** : 2026-05-02 (post-audit gate S51 PASS `749a333`).
**Type** : **sprint pair** — phase dette obligatoire
(§6.2.1 Regle 1). P2-REVIEW-A-1-S50 dispatch join order a 2/3
(approche 3/3 MANDATORY S53).
**Tip master d'entree** : `54cf0d0`.
**Phase 0 audit Sprint 51** : **DEJA JOUE** — `749a333` PASS
(0 P0, 0 P1, 1 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-01 (1j). 5 fichiers
  security + PATTERNS avec triggers_revalidate. 0 trigger actif
  (iroh reste 0.98, arti-client reste 0.41, frost-ed25519 3.0
  stable, wasmtime absent, aucun evenement externe en 1j). Pas
  de pre-research.

- **Technologies S52** : aucune nouvelle dep externe. Sprint
  purement soustractif (docs legacy DELETE) + micro-fix code
  (dispatch join order) + validation workflow existant. Aucune
  lib a consulter via context7.

- **ROADMAP_COMMITMENTS check** :
  - LT-1 Kudos-v2 : reclassifie pre-v1.0, sprint cible S50.
    S50-S51 ont fait la suppression Python. LT-1 pas encore
    adresse. **Reporte S53+** (justification : S52 est deja
    charge avec dette pair obligatoire + docs cleanup + release
    validation ; Kudos-v2 est un changement de formule
    significatif qui merite un sprint dedie).
  - LT-2..LT-5 latents (tag v1.0 non pose). LT-6 RESOLVED S32.
  - 0 condition declenchee.

- **HARDENING_ROADMAP §3** : pas de ligne S52 prescrite.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 51 CLOSED + audit PASS (`749a333`). Le projet est
Rust+Frontend pur depuis S50-S51 (0 LOC Python dans le
workspace, -72k LOC legacy supprimes). CI simplifiee a 2 blocs
(Rust + Frontend).

**Etat technique (tip `54cf0d0`)** :
- Workspace clean (`git status --short` = 0 apres chore
  .gitignore `54cf0d0`)
- Release workflow `release.yml` existant : 3 OS (linux-x86_64,
  macos-arm64, windows-x86_64) × 3 binaires (nexus-worker,
  nexus-shell-daemon, nexus-launcher) + SLSA provenance cosign +
  GitHub Release draft. Jamais teste en CI (uniquement tag v* ou
  workflow_dispatch)
- 21 fichiers docs/ legacy orphelins (~561 KB) : BENCHMARK.md,
  ARCHITECTURE.md, DATABASE_SCHEMA.md, API-REFERENCE.md,
  API_REFERENCE.md, CONFIGURATION.md, GUIDE-INSTALLATION.md,
  GUIDE-UTILISATION.md, README_FULL.md, PIPELINE.md,
  TESTING.md, TOOLS_MATRIX.md, API_COMPUTE.md,
  ARCHITECTURE_GPU.md, SECURITY_GPU.md, BENCHMARK_COMPUTE.md,
  COMPUTE_STATUS.md, FRONTEND_NETWORK.md, GUIDE_WORKER.md,
  WORKER.md, VISION_USE_CASES.md. Tous referent le monolithe
  Python supprime.

**Carries entrants S52** :
| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-REVIEW-A-1-S50 dispatch join order | 2/3 | S50 review |
| P2-REVIEW-B-1-S51 unsafe set_var futur | 1/3 | S51 review |
| P2-REVIEW-A-2-S51 docs legacy orphelines | 1/3 | S51 review |
| P2-D-1-AUDIT CLAUDE.md stale carry | 1/3 | S51 audit |

1 test flaky pre-existant confirme :
`browse::tests::probe_and_cache_with_quorum_majority_continues_to_dial`
(timing reseau, passe en re-run). Pas un P0.

### §1.2 Ancrage roadmap

S51 scope cuts §7 items 1-2 : "Binaires release cross-platform
— S52" et "VPS deployment + smoke test — S52". Le workflow
release.yml existe deja (S18 Phase B). S52 valide ce workflow ;
VPS deployment S53.

### §1.3 Compteurs tests entree (tip `54cf0d0`)

| Suite | Count |
|---|---|
| Rust nextest | 1199 |
| Rust doctests | 6 passed, 1 ignored |
| Vitest | 250 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 6/6 |
| **Total** | **~1455** |

**Post-S52 attendu** : ~1455 (sprint soustractif docs +
micro-fix dispatch ~0-2 tests, pas de regression).

### §1.4 Pre-launch protocol policy (rappel)

Aucun wire format touche. Aucun `*_FORMAT_VERSION` impacte.
Dispatch join order fix est une refactoring interne shutdown.

---

## §2 Goal

Resoudre la dette pair obligatoire (dispatch join order 2/3 +
docs legacy 21 fichiers + CLAUDE.md stale carry), puis valider
le workflow de release cross-platform existant par un dry-run
workflow_dispatch.
**Critere SMART : 15+ rows fail-fast verts au verification.md,
mesure binaire au Phase C wrap-up. 0 fichier docs/ legacy
orphelin. dispatch join order CLOSED. release.yml workflow_dispatch
lance au moins 1 fois avec succes (ou issues documentees).**

---

## §3 Phase 0 — Audit gate S51

**DEJA JOUE** : commit `749a333` PASS
(0 P0, 0 P1, 1 P2, 2 P3).
Audit findings dans `.planning/archive/v1.2/sprint51_audit_findings.md`.
6 carries documentes pour S52 (cf. §1.1 ci-dessus).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Dispatch join order : shutdown signal explicite

**Retenu** : ajouter un `tokio::sync::oneshot` shutdown signal
au dispatch_loop dans `runtime.rs`, symetrique au pattern deja
utilise pour `http_shutdown` (ligne 738). Le dispatch_loop
`select!` entre le channel recv et le shutdown signal. Plus de
dependance implicite sur l'ordre de drop de l'Arc DaemonHttpState.

**Rejete** :
- Documenter l'invariant sans fix code : la fragilite subsiste,
  un futur refactoring pourrait casser le shutdown silencieusement.
  Un commentaire n'est pas un test.
- Utiliser `CancellationToken` (tokio-util) : ajoute une dep
  pour un cas aussi simple qu'un oneshot. Surdimensionne.
- Inverser l'ordre join (dispatch avant HTTP) : creerait un
  deadlock si le dispatch_loop attend des messages du HTTP handler.

**Implications code** : `runtime.rs` (DaemonRuntime struct +
start() + shutdown), `dispatch_loop.rs` (select! avec shutdown).
~10-15 LOC.

### D2 — Docs legacy : DELETE complet des 21 fichiers

**Retenu** : `git rm` les 21 fichiers docs/ orphelins. L'historique
Git conserve le contenu pour reference. Aucun fichier n'a de
consommateur actif dans le codebase SBFB (grep confirme). Les
docs actives du projet sont dans `docs/claude/`, `docs/security/`,
`docs/rust/`, `docs/shell/`, `docs/release/`.

**Rejete** :
- Garder un sous-ensemble (ARCHITECTURE.md, API-REFERENCE.md) :
  ces docs decrivent l'architecture Python FastAPI/SQLite du
  monolithe pre-pivot. Aucune pertinence pour l'architecture
  Rust actuelle. Les futurs contributeurs trouveraient deux
  architectures contradictoires.
- Deplacer dans un dossier `docs/archive/` : ajoute du dead
  weight au checkout sans valeur. Git log suffit.
- Convertir en docs SBFB : effort demesure pour 21 fichiers
  decrivant un systeme totalement different.

**Implications code** : `git rm docs/{21 fichiers}` + nettoyage
.gitignore si certains sont listes + CLAUDE.md si docs/ refs.
~0 LOC code, ~561 KB supprimes.

### D3 — Release workflow validation : workflow_dispatch dry-run

**Retenu** : lancer le workflow `release.yml` via
`gh workflow run release.yml` (workflow_dispatch). Verifier que
les 9 jobs (3 binaires × 3 OS) completent. Telecharger les
artifacts et verifier les checksums SHA256. Documenter les
issues trouvees. Si des fixes sont necessaires, les appliquer
dans Phase B. Pas de creation de tag v* (pre-v1.0).

**Rejete** :
- Pousser un tag v0.x-test : cree un vrai release draft sur
  GitHub, pollue le namespace tags, confus pour les futurs
  utilisateurs.
- Ne pas tester du tout : le workflow existe depuis S18 mais n'a
  jamais ete execute. Les deps (cosign, SLSA, in-toto) et les
  paths (dist/, release-attest.sh) pourraient etre casses apres
  les suppressions S50-S51. Risque reel.
- Tester uniquement en local (`cargo build --release`) : ne
  valide pas le workflow GHA, les permissions, les artifacts
  upload, le release draft.

**Implications code** : `release.yml` + `scripts/release-attest.sh`
(fixes si necessaire). Pas de nouveau code — validation + fixes.

### D4 — CLAUDE.md stale carry : fix trivial

**Retenu** : supprimer la ligne `P2-REVIEW-A-1-S51 release-attest.sh
dead code 1/3` de CLAUDE.md (ligne 127). Cet item a ete CLOSE en
S51 Phase C mais la ligne n'a pas ete retiree de la section carries.
Finding P2-D-1-AUDIT de l'audit S51.

**Rejete** : aucune alternative — c'est un fix factuel d'une
ligne stale.

**Implications code** : CLAUDE.md ligne 127 DELETE. 1 ligne.

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 4, actionnable Phase B).

D3 ⚠️ (cosign version gap + upload-artifact) : le reviewer
signale que release.yml pin cosign v2.4.1 mais v3.0.6 est
courante (breaking change --bundle flag). Et upload-artifact v4
vs v8. Decision : Phase B dry-run exposera les issues. Si cosign
v2.4.1 fonctionne, garder le pin (stabilite). Si le dry-run
echoue sur cosign, upgrader en Phase B avec verification
release-attest.sh. upload-artifact v4 still supported — upgrade
a v8 non-critique, reporter si fonctionnel.

---

## §5 Plan Phase outline A..C

### Phase A — Dette pair obligatoire + docs legacy cleanup

**But** : resoudre les 3 items dette (dispatch join order 2/3 →
CLOSE, docs legacy 21 fichiers → DELETE, CLAUDE.md stale carry →
fix).

- Dispatch join order : ajouter shutdown oneshot au dispatch_loop
  (D1 retenu)
- `git rm` des 21 fichiers docs/ legacy (D2 retenu)
- Fix CLAUDE.md ligne 127 stale carry (D4 retenu)
- Nettoyer .gitignore si refs docs legacy
- Commit : `feat(sprint52): Sprint 52 Phase A — dette pair
  dispatch shutdown + docs legacy cleanup + CLAUDE.md fix`

### Phase B — Release workflow validation

**But** : valider le workflow release.yml par un dry-run
workflow_dispatch. Fixer les issues trouvees.

- Lancer `gh workflow run release.yml`
- Monitorer les 9 jobs (3 × 3)
- Telecharger artifacts et verifier checksums
- Fixer release.yml et/ou release-attest.sh si issues
- Documenter resultats dans le commit body
- Commit : `feat(sprint52): Sprint 52 Phase B — release workflow
  validation + fixes`

### Phase C — Docs + verification + wrap-up

**But** : mettre a jour la documentation, executer la verification
fail-fast, rediger l'audit plan S53.

- CLAUDE.md : mettre a jour compteurs, carries S53
- HARDENING_ROADMAP.md : update last_validated S52
- Verification fail-fast 15+ checks (2 blocs Rust + Frontend)
- sprint53_audit_plan.md
- Compteurs tests post-sprint
- Commit : `chore(sprint52): Phase C — wrap-up + verification +
  audit plan S53 + counters`

---

## §6 Items carry/dette

### Carries confirmes S52

- [dette] **P2-REVIEW-A-1-S50** dispatch join order 2/3 :
  **ADRESSE Phase A** → CLOSE attendu.
- [dette] **P2-REVIEW-A-2-S51** docs legacy orphelines 1/3 :
  **ADRESSE Phase A** → CLOSE attendu (21 fichiers DELETE).
- [dette] **P2-D-1-AUDIT** CLAUDE.md stale carry 1/3 :
  **ADRESSE Phase A** → CLOSE attendu.
- [carry] **P2-A-1** rand blocker upstream 12+/3 : exemption
  blocker externe. Justification renouvelee : pas de release rand
  0.9 ni fix getrandom upstream.
- [carry] **P2-AUDIT-2** pre-release transitives iroh : herite
  pin 0.98 (Day 0 #3).
- [carry] **P2-REVIEW-B-1-S51** unsafe set_var futur 1/3 :
  Rust 1.94 `unsafe` env ops proposal pas encore stabilise.
  Carry S53 (incremente a 2/3).

### Sprint pair — phase dette obligatoire

S52 pair → Phase A reservee dette (§6.2.1 Regle 1). 3 items
adresses : dispatch join order (2/3), docs legacy (1/3), CLAUDE.md
stale carry (1/3).

### Carries residuels post-S52

| Item | Compteur S53 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-REVIEW-B-1-S51 unsafe set_var futur | 2/3 | S51 review |

---

## §7 Scope cuts

1. **VPS deployment + smoke test** — S53 (binaries d'abord)
2. **LT-1 Kudos-v2 fairness reform** — S53+ (sprint dedie requis)
3. **Events SSE daemon-native** — post-v1.0
4. **MCP server Rust** — post-v1.0
5. **app-gov recreation** — post-v1.0
6. **Kudos debit/stake** — interdit (Day 0 #7)
7. **Pagination SQL-side LIMIT/OFFSET** — S53+
8. **Test infra mk_state() refactoring** — S53+

---

## §8 Tracabilite scope (S51 → S52)

| S51 scope cut | S52 disposition |
|---|---|
| Binaires release cross-platform — S52 | **Phase B** validation workflow (workflow existe depuis S18) |
| VPS deployment + smoke test — S52 | Scope cut reporte S53 (valider binaries d'abord) |
| Pagination SQL-side — S52+ | Scope cut reporte S53+ |
| Test infra mk_state() — S52+ | Scope cut reporte S53+ |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | release.yml workflow_dispatch echoue (deps cosign, paths dist/) | Medium | Low | Fix inline Phase B, documenter issues |
| R2 | Dispatch shutdown signal cree un regression tokio select! | Low | Medium | Test exhaustif du shutdown path |
| R3 | Docs legacy deletion casse des liens internes docs/ | Low | Low | Grep systematique `docs/{fichier}` avant delete |
| R4 | Flaky test browse probe_and_cache timing | Low | Low | Pre-existant, non imputable S52, monitorer |

---

## §10 Audit gate pattern — rappel

Phase 0 S51 jouee (PASS `749a333`). Phase C produira
sprint53_audit_plan.md pour la session fraiche S53.

---

## §11 Checkpoint de validation

1. **D1** : shutdown signal explicite dispatch_loop ?
   → oui (oneshot symétrique a http_shutdown)
2. **D2** : docs legacy DELETE complet ?
   → oui (21 fichiers, 0 consommateur actif, Git conserve)
3. **D3** : release workflow validation ?
   → oui (workflow_dispatch, pas de tag v*)
4. **D4** : CLAUDE.md stale carry ?
   → oui (1 ligne DELETE, finding audit P2)
