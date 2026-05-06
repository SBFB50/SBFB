# Sprint 52 — Audit findings (gate S53)

**Auditeur** : session fraiche (pas la session S52).
**Tip d'entree** : `71dfb2a` (HEAD master).
**Audit plan** : `sprint53_audit_plan.md`.
**Date** : 2026-05-02.

---

## Verdict : PASS

**0 P0, 0 P1, 1 P2, 2 P3.**
G4 rigor signal satisfait (>=1 P2 documente).
S53 Phase A peut demarrer directement.

---

## Track A — Dispatch shutdown fix (Phase A)

- [x] A-1 : `dispatch_shutdown: Option<oneshot::Sender<()>>` dans
  DaemonRuntime struct (`runtime.rs:197`). ✅
- [x] A-2 : `dispatch_loop::run()` prend `shutdown: oneshot::Receiver<()>`
  (`dispatch_loop.rs:27`) et utilise `tokio::select!` (`dispatch_loop.rs:32`). ✅
- [x] A-3 : `shutdown()` envoie le signal (`tx.send(())` a `runtime.rs:750`)
  AVANT join dispatch_handle (`runtime.rs:752-756`). Symetrique a
  http_shutdown. ✅
- [x] A-4 : test `dispatch_loop_writes_to_doc` cree le channel shutdown
  (`dispatch_loop.rs:94`) et passe le receiver a `run()` (`dispatch_loop.rs:96`).
  Envoie shutdown signal avant await handle (`dispatch_loop.rs:104`). ✅

## Track B — Docs legacy deletion (Phase A)

- [x] B-1 : `git ls-files` des 21 fichiers legacy retourne 0. ✅
- [x] B-2 : VISION_USE_CASES.md dans `.gitignore` (ligne 139). ✅
- [x] B-3 : 0 reference aux 20 fichiers supprimes dans `crates/`, `web/`,
  `.github/`. Le seul match (`blob_serve.rs:4`) reference
  `PROCESS_ARCHITECTURE.md` (faux positif, fichier existe dans
  `docs/security/`). ✅

## Track C — CLAUDE.md coherence (Phase A + C)

- [x] C-1 : ligne stale `P2-REVIEW-A-1-S51 release-attest.sh dead code`
  absente. ✅
- [x] C-2 : carries S53 = 3 P2 items (rand, iroh transitives, unsafe
  set_var 2/3) + LT items. Conforme a l'audit plan.
  Note : 3 NEW items 1/3 (nextest timeout, Woodpecker E2E, GHA 9/9)
  sont dans verification §4 et audit plan, pas dans CLAUDE.md — convention
  (CLAUDE.md liste carries >= 2/3 + exemptions seulement). ✅
- [x] C-3 : "Sprints 0-52 CLOSED" present (`CLAUDE.md:115`). ✅

## Track D — CI Woodpecker (Phase B)

- [x] D-1 : `.woodpecker/ci-linux.yml` presente, reproduit les blocs
  Rust (fmt/clippy/test/doctest) et Frontend (deps/typecheck/lint/test/
  build/size) + spdx-check. ✅
- [x] D-2 : syntaxe YAML Woodpecker valide (when/steps/image/commands). ✅
- [x] D-3 : 0 reference cosign ou release matrix dans le pipeline. ✅

## Track E — Self-hosted build design (Phase B)

- [x] E-1 : `docs/architecture/SELF_HOSTED_BUILD.md` present et tracke. ✅
- [x] E-2 : "n'est pas une extension triviale" explicite (ligne 30).
  task_type "build" = runtime separe. ✅
- [x] E-3 : LT-7 dans ROADMAP_COMMITMENTS comme "pre-v1.0 obligatoire"
  (`ROADMAP_COMMITMENTS.md:49`). ✅

## Track F — GHA release.yml matrix fix (Phase B)

- [x] F-1 : `os: [ubuntu-latest, macos-latest, windows-latest]` array
  (`release.yml:29`). ✅
- [x] F-2 : `binary: [nexus-worker, nexus-shell-daemon, nexus-launcher]`
  separe de include (`release.yml:30`). ✅
- [x] F-3 : `include:` ajoute os-label + shell par OS
  (`release.yml:31+`). ✅

## Track G — Process / meta

- [x] G-1 : G8 preflights 2/2 presents (A + B). ✅
- [x] G-2 : Phase reviews 3/3 presents (A + B + C). ✅
- [x] G-3 : Scope cuts 8/8 respectes (confirme par reviews A, B, C). ✅
- [x] G-4 : Delta tests cumule = +0 toutes suites. ✅
- [x] G-5 : Sprint pair, phase dette Phase A (3 items). ✅
- [x] G-6 : 3 carries CLOSED Phase A (dispatch S50, docs S51, CLAUDE.md
  audit S51). ✅
- [x] G-7 : HARDENING_ROADMAP last_validated = S52 (2026-05-02). ✅
- [x] G-8 : Phase B pivot documente dans preflight §Pivot utilisateur :
  plan original "GHA dry-run" remplace par CI Woodpecker + design doc
  self-hosted build (LT-7). Decision utilisateur documentee. ✅

---

## Findings

### P2-AUDIT-1 — Images CI Woodpecker non pinnees digest

`.woodpecker/ci-linux.yml` utilise `image: rust:1.94` et
`image: node:20` — tags minor-version, pas SHA256 digest ni
point-release (`rust:1.94.0`, `node:20.19.0`). Dans un projet
avec pipeline SLSA L1, cosign attestation et design doc
self-hosted build explicitement focuse sur la supply chain, les
images CI devraient etre pinnees au minimum a un point-release,
idealement a un digest SHA256 (`image: rust@sha256:...`).

**Severite** : P2 (hygiene supply-chain, non-bloquant pre-v1.0).
Le risque est attenue tant que le pipeline n'est pas deploye sur
un agent (scope S53 VPS).

**Carry S53** : hardening images refs au deploiement de l'agent
Woodpecker (P2-AUDIT-1-S52, 1/3).

### P3-AUDIT-1 — Formulation verification check #3

`verification.md §1` check #3 note "32 timeout pression
ressources" provenant d'un run intermediaire sous charge. Le run
final montre 1199/1199 0 timeout. La formulation pourrait
induire un auditeur en erreur sur la stabilite des tests.
Deja signale par Phase C review comme P2.

### P3-AUDIT-2 — Count mismatch plan vs execution docs legacy

Plan D2 et etapes §Phase A disent "21 fichiers DELETE". Execution
reelle : 20 `git rm` + 1 `.gitignore` (VISION_USE_CASES.md etait
deja untracked depuis le chore `54cf0d0`). Le resultat est correct
(0 legacy doc dans le workspace) mais la documentation pourrait
etre plus precise : "20 supprimes + 1 confirme exclu".

---

## Post-S52 note

Commit `71dfb2a docs(architecture): publish model` ajoute
`docs/architecture/PUBLISH_MODEL.md` apres la cloture S52.
Standalone docs commit, conforme a la convention (pas de phase
requise pour docs hors-sprint). Memory tip `ff9b886` sera mis a
jour en fin de session.

---

## Carries S53 confirmes (reprise verification §4 + audit)

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-B-1-S51 unsafe set_var futur | 2/3 | S51 review |
| P2-REVIEW-A-1-S52 nextest timeout profiling | 1/3 | NEW S52 Phase A review |
| P2-REVIEW-B-1-S52 Woodpecker E2E validation | 1/3 | NEW S52 Phase B review |
| P2-REVIEW-B-2-S52 GHA 9/9 re-run confirm | 1/3 | NEW S52 Phase B review |
| P2-AUDIT-1-S52 images CI Woodpecker non pinnees | 1/3 | NEW S52 audit |

S53 impair : pas de phase dette obligatoire.
P2-REVIEW-B-1-S51 unsafe set_var a 2/3 — si non adresse S53,
3/3 MANDATORY S54.
