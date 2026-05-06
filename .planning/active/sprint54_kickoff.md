# Sprint 54 — Kickoff (Edition 2024 + dette pair + E2E wire + CI infra)

**Ecrit** : 2026-05-06 (post-audit gate S53 PASS `2f5d76c`).
**Type** : **sprint pair** — phase dette obligatoire (§6.2.1 Regle 1).
P2-REVIEW-B-1-S51 edition 2024 upgrade a 3/3 MANDATORY (entre
comme phase, pas carry).
**Tip master d'entree** : `2f5d76c`.
**Phase 0 audit Sprint 53** : **DEJA JOUE** — `2f5d76c` PASS
(0 P0, 0 P1, 7 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-06 (0j). 5 fichiers
  security + PATTERNS avec triggers_revalidate. 0 trigger actif.
  Pas de pre-research.

- **Rust edition 2024** (context7 `/rust-lang/rust` + analyse codebase) :
  Edition 2024 stabilisee Rust 1.85+. Workspace utilise Rust 1.94 →
  compatible. Changements impactants audites :
  - `std::env::set_var` / `std::env::remove_var` deviennent `unsafe fn`
    → 70+ sites a wrapper dans 17 fichiers.
  - `unsafe extern` blocks requis → 0 extern block dans le codebase → ok.
  - `gen` keyword reserve → utilise seulement comme methode `.gen()` et
    string literal → ok.
  - `unsafe_op_in_unsafe_fn` lint → 0 `unsafe fn` dans le codebase → ok.
  - `-> impl IntoResponse` lifetime capture → tous des async fn top-level
    → non applicable.
  - Strategie : `cargo fix --edition 2024` puis SAFETY comments manuels.

- **ROADMAP_COMMITMENTS check** :
  - LT-1 Kudos-v2 : RECLASSIFIE pre-v1.0, pas S54.
  - LT-7 self-hosted build : PRE-V1.0 OBLIGATOIRE (S54-S55). S53 VPS
    bootstrap valide. Phase C E2E wire est le prerequis technique.
  - LT-2..LT-5 latents (tag v1.0 non pose). LT-6 RESOLVED S32.
  - 0 condition declenchee.

- **HARDENING_ROADMAP §3** : pas de ligne S54 prescrite.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 53 CLOSED + audit PASS (`2f5d76c`). Le projet est Rust+Frontend
pur depuis S50-S51. Le P2P iroh est valide cross-machine (LAN Win-Mac,
WAN dev-VPS Helsinki, Niveaux 1+2 atteints sur 3 OS). Le gossip
pipeline est complet (subscribe non-bloquant, outbox, NeighborUp
replay, browse pull). Le VPS sbfb-eu est operationnel.

**Etat technique (tip `2f5d76c`)** :
- Workspace clean, edition 2021
- `nexus-shell-daemon start` : fonctionne sur 3 OS (Windows, macOS ARM,
  Linux VPS). Gossip non-bloquant + outbox + browse pull.
- P2P valide cross-machine depuis S53
- 70+ appels `set_var`/`remove_var` non-unsafe (safe en edition 2021)
- GAP E2E : `tasks_doc_ticket` absent du wire format invite → chemin
  task→worker→result bloque
- CI : GHA fonctionnel (9 jobs, dernier run S52), Woodpecker pipeline
  ecrit (`.woodpecker/ci-linux.yml`) mais agent VPS non deploye

**Carries entrants S54** :

| Item | Compteur | Source |
|---|---|---|
| P2-REVIEW-B-1-S51 edition 2024 upgrade | **3/3 MANDATORY** | re-scoped S53 |
| P2-REVIEW-A-1-S52 nextest timeout | 2/3 | S52 review |
| P2-REVIEW-B-1-S52 Woodpecker E2E | 2/3 | S52 review |
| P2-REVIEW-B-2-S52 GHA 9/9 | 2/3 | S52 review |
| P2-AUDIT-1-S52 images CI pin | 2/3 | S52 audit |
| P2-A-1 rand blocker upstream | 12+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-S53-outbox non-persistant | 1/3 | S53 Phase F review |
| P2-S53-browse_request rate-limit | 1/3 | S53 Phase G review |
| P2-S53-gossip params struct | 1/3 | S53 Phase D review |
| P2-S53-node_key perms 0600 | 1/3 | S53 Phase E review |
| P2-S53-route collision doc | 1/3 | S53 Phase A review |
| P2-S53-periodic republish | 1/3 | S53 Phase F review |
| P2-S53-preflight E/F/G process gap | 1/3 | S53 audit |

### §1.2 Ancrage roadmap

S53 a valide le P2P cross-machine (prerequis VPS pour LT-7). S54
est le sprint qui prepare le terrain pour LT-7 (self-hosted build) :
le E2E wire (Phase C) connecte le dernier chainon task→worker→result.
L'edition 2024 upgrade (Phase A) est MANDATORY independamment du
theme — c'est de la dette a 3/3.

roadmap_v1_migration_rust.md §S54 : "Self-hosted build fondation,
CI infra, dette pair."
ROADMAP_COMMITMENTS LT-7 : "PRE-V1.0 OBLIGATOIRE. S54-S55."

### §1.3 Compteurs tests entree (tip `2f5d76c`)

| Suite | Count |
|---|---|
| Rust nextest | 1206 |
| Rust doctests | 6 passed, 1 ignored |
| Vitest | 250 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 6/6 |
| **Total** | **~1462** |

**Post-S54 attendu** : ~1470+ (E2E wire tests, dette items tests).

### §1.4 Pre-launch protocol policy (rappel)

Phase C touche le wire format invite (ajout `tasks_doc_ticket` a
`MintRequest`). Pre-launch : pas de bump version, on redefini la v1
courante. Les tests "legacy decode" existants qui simuleraient
l'ancien format sans le champ = a supprimer immediatement.

---

## §2 Goal

Sprint 54 stabilise le workspace pour la production : edition Rust
2024 (MANDATORY 3/3), resorbe la dette pair S53, cable le E2E task
flow (prerequis LT-7), et consolide l'infra CI (prevention escalation
4 items 2/3 → 3/3 MANDATORY S55).
**Critere SMART : 24+ rows fail-fast verts au verification.md, mesure
binaire au Phase E wrap-up. Edition 2024 compile + tests verts.
tasks_doc_ticket present dans le wire invite.**

---

## §3 Phase 0 — Audit gate S53

**DEJA JOUE** : commit `2f5d76c` PASS
(0 P0, 0 P1, 7 P2, 2 P3).
Audit findings dans `.planning/active/sprint53_audit_findings.md`.
14 carries documentes pour S54 (cf. §1.1 ci-dessus).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Edition 2024 via `cargo fix --edition`

**Retenu** : utiliser `cargo fix --edition 2024` pour auto-fixer la
majorite des changements (ajout `unsafe {}` autour de `set_var` /
`remove_var`), puis revue manuelle pour ajouter les commentaires
SAFETY documentes. Le commentaire standard :
```rust
// SAFETY: called in test setup before thread spawn / single-threaded context.
unsafe { std::env::set_var("KEY", "value") };
```

**Rejete** :
- Refactoring des tests pour eliminer `set_var` : chaque test qui
  utilise `set_var` devrait etre reecrit avec un struct config injecte.
  Changement d'architecture significatif hors scope — c'est la bonne
  direction long-terme mais serait du scope creep pour un sprint deja
  charge. Le wrapping mecanique est suffisant et conforme a l'ecosysteme
  Rust (pattern documente dans la migration guide edition 2024).
- Migration progressive (1 crate par sprint) : ajoute de la complexite
  de workspace mixed-edition. Rust supporte les editions mixtes mais
  c'est fragile et peu teste. Migration atomique en 1 commit est plus
  propre et plus facile a auditer.
- Reporter a S55 : l'item est a 3/3 MANDATORY, reporter n'est pas
  autorise sans exemption blocker externe (§6.2.1 Regle 2).

**Implications code** : 17 fichiers `.rs` + `Cargo.toml` (edition bump).
Aucun changement de logique metier — wrapping mecanique uniquement.

### D2 — E2E wire tasks_doc_ticket

**Retenu** : ajouter le champ `tasks_doc_ticket: String` (serialise
`DocsTicket`) dans `MintRequest` et `InviteRecord`. Le coordinateur
(daemon) exporte le `DocsTicket` de son `project_doc` iroh-docs dans
`invite_api.rs` lors de la creation d'invite. Le worker recoit le
ticket et l'utilise pour se synchroniser sur le document de taches.

**Rejete** :
- Passer le ticket hors-bande (API separee) : ajoute un round-trip
  supplementaire et un point de failure. L'invite est le bon vehicule
  car elle contient deja toutes les informations de connexion worker.
- Utiliser un champ `project_id` existant pour encoder le ticket :
  semantiquement incorrect, le project_id est un identifiant humain
  et le DocsTicket est un blob binaire/base32. Confusion de types.
- Bump INVITE_FORMAT_VERSION : pre-launch policy, pas de bump. On
  redefini la v1 courante.

**Implications code** : 3 fichiers (coordinator-rs/invite.rs,
shell-daemon/invite_api.rs, worker-core/invite.rs).

### D3 — CI infra VPS

**Retenu** : installer l'agent Woodpecker sur le VPS sbfb-eu
(`135.181.42.188`), valider le pipeline `.woodpecker/ci-linux.yml`,
confirmer le run GHA 9/9, et pinner les images CI. Le VPS est deja
operationnel (Rust 1.95, daemon teste S53).

**Rejete** :
- Nouveau VPS dedie CI : overkill pour un agent CI seul. Le VPS
  sbfb-eu a 8GB RAM, suffisant pour build + daemon + agent CI.
- CI cloud (GitHub Actions only) : ne resout pas LT-7 (self-hosted
  build). L'agent Woodpecker local est le premier pas vers
  l'autonomie CI.
- Docker-in-Docker pour l'agent : ajoute une couche d'abstraction.
  L'agent natif (binaire) est plus simple et plus fiable pour un
  premier deploiement.

**Implications code** : `.woodpecker/ci-linux.yml` (images pin) +
documentation. Pas de code applicatif touche.

### D4 — Dette pair items selection

**Retenu** : resoudre 4 items P2 S53 dans la phase dette obligatoire :
- `node_key` permissions 0600 (securite)
- `gossip params struct` (refactoring qualite code)
- `route collision doc` update (docs securite)
- `periodic republish` timer (reliability gossip)

Priorite : securite d'abord (`node_key` perms), puis reliability
(`periodic republish`), puis qualite (`gossip params struct`), puis
docs (`route collision doc`).

**Rejete** :
- Inclure `outbox persistant` : necessite du design (format fichier,
  rotation, recovery) et depasse le budget d'une phase dette. Carry
  S55.
- Inclure `browse_request rate-limit` : necessite du design (per-peer
  tracking, decay, PoW escalation). Carry S55.
- Inclure `preflight process gap` dans le code : c'est un ajout doc
  a README.md §6.9, pas du code. Inclus comme item doc de la phase.

**Implications code** : `runtime.rs` (3 items code) +
`LOOPBACK_ENDPOINTS_TRUST_TIERS.md` + `README.md` §6.9.

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 4).

D3 ⚠️ (Woodpecker serveur requis) : acknowledge — proceder avec
option (a) serveur + agent sur le meme VPS sbfb-eu. Le serveur
Woodpecker en mode SQLite local est leger (~100MB RSS). Si la RAM
de 8GB est insuffisante pendant les builds Rust (~4-6GB peak),
scope cut le serveur Woodpecker a S55 et se contenter du GHA 9/9
+ images pin pour la Phase D.

---

## §5 Plan Phase outline A..E

### Phase A — Edition 2024 upgrade (MANDATORY 3/3)

**But** : migrer le workspace Rust de edition 2021 a edition 2024.
Wrapping mecanique de 70+ appels `set_var`/`remove_var` en `unsafe {}`
avec commentaires SAFETY.

- `cargo fix --edition 2024` pour auto-fix
- Revue manuelle des SAFETY comments
- Bump `edition = "2024"` dans Cargo.toml
- Verification : fmt, clippy, nextest, doctests, release build
- CLOSE P2-REVIEW-B-1-S51
- Commit : `feat(sprint54): Sprint 54 Phase A — Rust edition 2024
  upgrade + unsafe set_var wrapping`

### Phase B — Dette pair (§6.2.1 Regle 1)

**But** : resoudre 4 items P2 S53 + 1 doc process gap.

- `node_key` perms 0600 : `std::fs::set_permissions()` apres write
- `gossip params struct` : extraire `GossipTaskConfig` struct
- `periodic republish` : timer 45-60s jitter dans gossip task
- `route collision doc` : update LOOPBACK_ENDPOINTS_TRUST_TIERS.md
- `preflight process gap` : documenter exemption post-plan §6.9
- Commit : `feat(sprint54): Sprint 54 Phase B — dette pair quick
  P2 batch (node_key perms + gossip refactor + periodic republish)`

### Phase C — E2E wire task flow

**But** : cabler le chainon manquant `tasks_doc_ticket` dans le wire
format invite. Prerequis LT-7.

- `MintRequest` + `InviteRecord` : +champ `tasks_doc_ticket: String`
- `invite_api.rs` : exporter `DocsTicket` depuis `project_doc`
- `worker-core/invite.rs` : parser le ticket, synchroniser le doc
- Tests unitaires : invite encode/decode avec ticket
- Commit : `feat(sprint54): Sprint 54 Phase C — E2E wire
  tasks_doc_ticket in invite format`

### Phase D — CI infra (4 items 2/3)

**But** : consolider l'infra CI pour empecher l'escalation de 4 items
a 3/3 MANDATORY S55.

- Woodpecker agent install sur VPS sbfb-eu
- Pipeline validation (`.woodpecker/ci-linux.yml`)
- GHA re-run 9/9 confirmation
- CI images pinning
- nextest timeout profiling (investigation + documentation)
- Commit : `feat(sprint54): Sprint 54 Phase D — CI infra
  Woodpecker agent + GHA validation + images pin`

### Phase E — Wrap-up + verification + audit plan S55

**But** : cloturer le sprint.

- CLAUDE.md : update S54 CLOSED, carries S55
- HARDENING_ROADMAP : update last_validated S54
- verification.md : 24+ fail-fast rows
- sprint55_audit_plan.md : 7+ tracks
- Commit : `chore(sprint54): Phase E — wrap-up + verification +
  audit plan S55`

---

## §6 Items carry/dette

### Carries confirmes S54

- [phase A] **P2-REVIEW-B-1-S51** edition 2024 upgrade 3/3 MANDATORY :
  **ADRESSE Phase A** → CLOSE attendu.
- [phase B] **P2-S53-node_key perms 0600** 1/3 : **ADRESSE Phase B**.
- [phase B] **P2-S53-gossip params struct** 1/3 : **ADRESSE Phase B**.
- [phase B] **P2-S53-route collision doc** 1/3 : **ADRESSE Phase B**.
- [phase B] **P2-S53-periodic republish** 1/3 : **ADRESSE Phase B**.
- [phase B] **P2-S53-preflight E/F/G process gap** 1/3 :
  **ADRESSE Phase B** (doc README.md §6.9).
- [phase D] **P2-REVIEW-A-1-S52** nextest timeout 2/3 : **ADRESSE
  Phase D** (investigation + doc).
- [phase D] **P2-REVIEW-B-1-S52** Woodpecker E2E 2/3 : **ADRESSE
  Phase D** (agent deploy + validation).
- [phase D] **P2-REVIEW-B-2-S52** GHA 9/9 2/3 : **ADRESSE Phase D**
  (re-run confirmation).
- [phase D] **P2-AUDIT-1-S52** images CI pin 2/3 : **ADRESSE Phase D**
  (images pinning).
- [carry] **P2-A-1** rand blocker upstream 12+/3 : exemption externe
  (pas de release rand 0.9 ni fix getrandom).
- [carry] **P2-AUDIT-2** iroh transitives : herite pin 0.98
  (Day 0 #3).
- [carry] **P2-S53-outbox non-persistant** 1/3 : carry S55
  (necessite design).
- [carry] **P2-S53-browse_request rate-limit** 1/3 : carry S55
  (necessite design).

### S54 pair — phase dette obligatoire Phase B

4 items P2 S53 + 1 doc process gap resolus en Phase B.

### Carries residuels post-S54

| Item | Compteur S55 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-S53-outbox non-persistant | 2/3 | S53 Phase F review |
| P2-S53-browse_request rate-limit | 2/3 | S53 Phase G review |

**Attention S55 impair** : 2 items passent a 2/3. Les 4 items 2/3
S52 sont adresses en Phase D (0 escalation S55 si reussi).

---

## §7 Scope cuts

1. **LT-7 self-hosted build foundation** — S55 (depend Phase C E2E wire)
2. **Test E2E multi-noeuds automatise** — S55 (depend Phase C)
3. **Outbox persistant fichier** — S55 (> 100 LOC, necessite design)
4. **Browse_request rate-limit per-peer** — S55 (necessite design)
5. **VPS TLS + nginx** — S55 (post CI infra)
6. **VPS monitoring + alerting** — S55+
7. **systemd service VPS** — S55
8. **LT-1 Kudos-v2 fairness reform** — sprint dedie (S56+)
9. **Events SSE daemon-native** — post-v1.0
10. **MCP server Rust** — post-v1.0
11. **Pagination SQL-side LIMIT/OFFSET** — S55+
12. **Test infra mk_state() refactoring** — S55+

---

## §8 Tracabilite scope (S53 → S54)

| S53 scope cut | S54 disposition |
|---|---|
| Woodpecker agent VPS — S54 | **Phase D** |
| systemd service VPS — S54 | Scope cut reporte S55 |
| VPS TLS + nginx — S54 | Scope cut reporte S55 |
| VPS monitoring + alerting — S54+ | Scope cut reporte S55+ |
| Pagination SQL-side — S54+ | Scope cut reporte S55+ |
| Test infra mk_state() — S54+ | Scope cut reporte S55+ |
| Deploy scripts rewrite — S54 | Scope cut reporte S55 (absorbe dans LT-7) |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Edition 2024 casse des patterns non-anticipes (lifetime capture, match ergonomics) | Low | High | `cargo fix` + test suite complete 1206 tests |
| R2 | E2E wire necessite plus que prevu (discovery plumbing) | Medium | Medium | Scope cut parties > wire format au S55 |
| R3 | VPS Woodpecker agent ne compile pas (version mismatch) | Low | Low | Fallback image Docker |
| R4 | GHA re-run echoue (flaky tests, timeout Windows) | Medium | Low | Documenter echecs, re-run 2-3x |
| R5 | nextest timeout profiling sans root cause | Medium | Low | Documenter investigation, carry S55 si pas de cause |

---

## §10 Audit gate pattern — rappel

Phase 0 S53 jouee (PASS `2f5d76c`). Phase E produira
sprint55_audit_plan.md pour la session fraiche S55.

---

## §11 Checkpoint de validation

1. **D1** : edition 2024 via `cargo fix --edition` + SAFETY comments ?
   → oui (migration atomique, pas de refactoring tests)
2. **D2** : E2E wire `tasks_doc_ticket` dans invite format ?
   → oui (prerequis LT-7, pre-launch pas de bump version)
3. **D3** : CI infra VPS Woodpecker + GHA confirmation ?
   → oui (prevention escalation 4 items 2/3)
4. **D4** : dette pair 4 items quick + 1 doc ?
   → oui (sprint pair, obligation §6.2.1)
