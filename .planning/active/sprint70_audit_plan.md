# Sprint 70 — Audit Plan (pour Phase 0 S70)

**Ecrit** : 2026-05-23 (Phase E Sprint 69).
**Sprint audite** : Sprint 69 (Babel dogfood via Factory + pilote ferme + Gate 1).
**Tip attendu** : commit Phase E docs(sprint69).
**Source intake** : `.planning/research/process_portable_complete_s70.md`.

---

## S70 Objective — Process Portable Complete

S70 rend le process agent nexus-grid **completement portable** :
tout agent (Claude, Codex, GPT, LLM local, humain) peut reprendre
le travail a partir des fichiers du repo seuls, sans memoire de
chat privee.

Les 7 phases S70 (plan v3 ambitieux, derivees de
`.planning/research/process_portable_complete_s70.md` §5 + recadrage
PO 2026-05-24) sont :

| Phase | Titre | Livrable principal |
|---|---|---|
| A | Canon portable | `docs/agent/AGENT_SYSTEM.md` (carte du systeme) |
| B | Dette pair + P2 audit | `PATTERNS.md`, `docs/claude/README.md`, P2-I-3 3/3 |
| C | Prompt portability full | `handoff`, `phase-review`, `audit-gate`, `phase-auditor`, `commit-body` executables |
| D | Observabilite process Rust + Operator serve | `sbfb-factory process status-sprint`, `lint-planning`, `audit-commit`, `operator serve --once-smoke` |
| E | Factory Viewer protocole + Factory Operator local | `examples/sbfb-factory-viewer/` + `tools/factory-operator/`, connectes a `sbfb-factory operator serve` |
| F | Hooks + provider config + dogfood | Hooks dynamiques, provider flag, dogfood via Operator/Viewer |
| G | Contrat RRV/Factory + wrap-up | Modes `@` = alias sur roles, verification, audit plan S71 |

Gate 1 dogfood sert de surface de verification : l'outillage
Factory/Babel/Gate 1 est utilise pour prouver que le process
portable fonctionne de bout en bout. S70 separe Factory Viewer et
Factory Operator : le Viewer est une app SBFB sandboxee de consultation
et preuve ; l'Operator est un outil local privilegie, servi par Rust
via `sbfb-factory operator serve`. L'autorite reste `.planning/active/`,
les commits et les gates ; l'Operator pilote et journalise, il ne
fabrique pas seul les verdicts.

---

## Audit Tracks A-I

### Track A — Portable canon

`AGENT_SYSTEM.md` doit exister sans dupliquer `PROCESS.md`.
Verifier que le registre de roles (driver, researcher, reviewer,
auditor, product, security, release, memory) est complet et que
le mapping provider (Claude, Codex, GPT, local LLM, humain) est
coherent. Verifier le contrat artefact (quel role ecrit quel
fichier). Verifier que les non-goals sont explicites.

### Track B — Handoff

Verifier que `handoff.md` est cree et wire dans
`sbfb-factory process prompt --kind handoff`. Verifier qu'un handoff
genere suffit a reprendre le travail sans chat history. Verifier que
`AGENT_SYSTEM.md` est inclus dans `sbfb-factory process context`.
Verifier aussi le bootstrap session fraiche complet : `base.md`
orientation, `universal.md` lifecycle, runtime context, `handoff.md`
point-in-time, puis prompt specialise. Aucun champ ou assertion ne
doit rendre la chat history authoritative.

### Track C — Observabilite process Rust

Verifier que `sbfb-factory process status-sprint`, `lint-planning`,
`audit-commit --rev HEAD` et `operator serve` sont implantes et testes
en Rust. Verifier la sortie JSON pour consommation future RRV/Factory.
Verifier les tests sous `crates/sbfb-factory/tests/`.

### Track D — Gates et hooks

Verifier que les bypasses connus sont fermes :
- Exact `## Verdict: PASS` (pas `## Verdict : PASS` avec espace)
- `chore(sprintN): Sprint N Phase X` ne bypass pas Codex/9-section
- G8 preflight/pivot est gate portable pour tout phase commit reel
- Hooks Claude ne contiennent plus d'assumptions Sprint 67 stales
- `nexus-phase-auditor` et `nexus-phase-review-deep` routing aligne

### Track E — Hooks et CI

Verifier qu'il existe un CI process pour `sbfb-factory process`,
prompt assembly, hooks et fixtures negatives. Le CI doit prouver que la couche
portable n'est pas un document mort.

### Track F — CI coverage process

Verifier la couverture CI du process portable : prompt assembly,
pre-commit hooks, negative fixtures (prompt invalide, handoff
incomplet).

### Track G — RRV contract

Verifier que les modes `@research`, `@dev`, `@audit`, `@security`,
`@product` sont definis comme alias sur des roles du registre, pas
comme autorite parallele. RRV affiche l'etat du process et les
evidences, mais l'execution reste dans `.planning/active/` et les
gates.

### Track H — Factory contract

Verifier que Factory est scindee en deux surfaces :
- Factory Viewer consomme les artefacts/proofs/previews exportes ou
  publies, comme app SBFB sandboxee du protocole.
- Factory Operator produit les apps et preuves localement, comme outil
  Rust privilegie du noeud, sans devenir autorite de verification.
- Les deux surfaces partagent `tools/factory-ui/src/readonly` pour les
  modeles, labels, previews et cartes de preuve ; seul l'Operator importe
  les extensions locales privilegiees.
Babel reste une app creee avec Factory, pas le process lui-meme.

### Track I — Dogfood

Verifier qu'un changement reel Nexus a ete fait avec le flow
portable complet (`sbfb-factory process context` → handoff →
preflight/review/Codex via fichiers repo seuls). Le handoff plus les fichiers repo
doivent suffire a reprendre dans un autre provider sans memoire
chat privee.

---

## RRV/Factory Consumer Contract

Apres S70, RRV et Factory consomment le process portable :

- RRV peut lire `sbfb-factory process status-sprint`,
  `lint-planning`, `audit-commit` et `AGENT_SYSTEM.md` comme premier
  corpus process-aware.
- Factory Viewer peut afficher les preuves/previews publiees ou
  exportees par l'Operator.
- Factory Operator peut packager les templates/contrats/prompts pour
  les projets generes avec un backend Rust local.
- Factory Viewer et Factory Operator reutilisent le meme socle lecture,
  mais le Viewer ne contient ni endpoint Operator, ni import
  `factory-ui/operator`, ni capacite cachee d'execution locale.
- Les modes `@` sont des alias de roles, pas un systeme parallele.
- Factory ne possede pas l'autorite de verification — l'Operator publie
  des apps, le daemon signe la provenance, le process valide la qualite,
  le Viewer expose la preuve.

Sequencing post-S70 :
- `@dev LocalOnly`, seed source-only, `sbfb-search`, provider
  router, SearchManifest → planifies seulement apres que le
  contrat process n'est plus implicite.

---

## Non-Goals

S70 ne doit PAS :

- Construire RRV total (indexation large, UI recherche avancee)
- Construire SearchManifest (wire format + gossip)
- Ajouter du compute prive ou remote
- Ingerer un corpus OSS large comme apps verifiees
- Construire une route produit `/factory` dans `web/` ou deplacer
  l'autorite process dans Factory. Le Viewer SBFB et l'Operator local
  Rust sont in-scope.
- S'appuyer sur la memoire de chat comme source de verite

---

## Exit Gate

S70 est complet quand :

1. `AGENT_SYSTEM.md` existe et ne duplique pas `PROCESS.md`
2. `handoff.md` est wire dans `sbfb-factory process prompt --kind handoff`
3. `sbfb-factory process status-sprint`, `lint-planning`,
   `audit-commit` et `operator serve --once-smoke` fonctionnent et
   sont testes
4. Les bypasses connus sont fermes (verdict exact, Codex gate,
   hooks stales)
5. Factory Viewer est une app SBFB sandbox-compatible sans endpoint
   Operator ; `tools/factory-ui/src/readonly` est partage sans capacite
   locale privilegiee ; Factory Operator compile, lint, typecheck, affiche
   status + prompt + lint + audit depuis `sbfb-factory operator serve`,
   et peut preparer un brouillon repo/docs sur allowlist avec preview
   diff + confirmation. Avant tout code front, le prompt UX Claude
   Design est ecrit, colle dans Claude Design par l'operateur, puis le
   lien/export est reference dans
   `.planning/active/sprint70_factory_ux_design_handoff.md`. Verifier
   le flux operateur complet : nouveau contexte, selection agent,
   prompt, action allowlistee, draft avec preview, discussion Agent
   Chat, log, et frontiere d'autorite.
6. Le dogfood prouve un changement repo-visible pilote depuis le
   flow portable, pas seulement une demo UI
7. L'UI parle en intentions comprehensibles ("Preparer la phase",
   "Verifier avant validation", "Transmettre a un autre agent") et
   garde `sbfb-factory`, `kind`, `provider`, `preflight` dans un panneau
   details techniques, pas dans les CTA principaux
8. CI/smoke prouve la couche portable minimale (prompt assembly,
   fixtures negatives, `operator serve --once-smoke`)
9. Les modes `@` sont documentes comme alias de roles
10. Factory Viewer consomme les preuves ; Factory Operator produit et
    publie localement ; aucun des deux n'est autorite finale.
11. L'Operator est explicitement action-gated et conversationnel :
    allowlist, preview, confirmation, journal JSONL, Agent Chat
    demarre depuis context-pack. Les operations sensibles
    (shell/commit/push/verdict final PASS) ne sont valides que via
    une vraie session agent + gates + preuves repo-visibles, pas
    comme simple bouton UI.
12. `context-pack` contient base/universal/handoff/context, path/hash,
    HEAD, dirty files, role/provider/intention, et marque
    `chat_history_authoritative: false`

**Critere SMART** : le handoff genere par
`sbfb-factory process prompt --kind handoff` suffit a un nouvel agent pour reprendre le sprint en cours
sans chat history.

---

## Critere verdict audit S69

| Verdict | Condition |
|---|---|
| **PASS** | 0 P0, 0 P1, >= 1 P2+ documente |
| **CONDITIONAL PASS** | 0 P0, 0 P1 mais conditions a surveiller S70 |
| **FAIL** | >= 1 P0 ou >= 1 P1 non resoluble dans l'audit |

Rigor signal G4 : PASS exige >= 1 P2+ documente. 0 P0/P1 et 0
P2+ = CONCERN (pas PASS).

---

## Tracks audit S69 (ce que Phase 0 S70 doit verifier)

### Track 1 — Suites verification

Relancer fail-fast 27/27 du verification.md S69. Verifier
1433 Rust / 279 Vitest / 6/6 size-limit. Verifier coherence
delta annonce (+14 Rust) vs reel.

### Track 2 — Security review

Scanner les 3 phases code S69 (A-C) :
- Phase A : audit_log JSONL — verifier pas d'injection via args.
  MAX_PREVIEW_ENTRIES — verifier le cap effectif.
- Phase B : FG8 provenance Ed25519 — verifier que verify_strict
  est utilise (pas verify). Pipeline FG4-FG8 — verifier sequencing
  et abort on fail.
- Phase C : Template static-reader — verifier pas de XSS dans
  le template index.html. Template engine — verifier que les
  placeholders ne permettent pas d'injection.

### Track 3 — Patterns review

Verifier coherence PATTERNS.md post-S69. Pas de nouveau pattern
documente S69 (confirmer). Factory gates patterns (FG4-FG8)
couverts par FACTORY_GATES.md (S65 Phase D).

### Track 4 — Scope cuts compliance

14/14 scope cuts auto-reportes dans verification.md. Verifier
par grep exhaustif.

### Track 5 — Tests delta coherence

Verifier deltas par phase : A +5, B +6, C +3, D +0. Total
1419→1433 (+14). Note : cumul Phase D body incorrect (cosmetic
P3, nombres par phase faux dans le recap mais total correct).

### Track 6 — Review files quality

5 preflight (A-E) tous EXECUTE. 4 reviews (A-D) toutes PASS.
4 codex_review (A-D) toutes brutes. Phase E docs-only.

### Track 7 — Carry-overs

- P2-I-2 3/3 CLOSED : verifier script + procedure
- P3-I-2 CLOSED : verifier 0 dead_code gates.rs
- P2-B-1 CLOSED : verifier MAX_PREVIEW_ENTRIES = 10
- P2-I-3 2/3 : verifier body Phase D complet
- 8 carries ouverts routes S70

### Track 8 — HARDENING review

THREAT_MODEL.md §13 preview surface (Phase A). Verifier
completude. Pas de nouvelle surface d'attaque S69 (Factory
est local, pas reseau).

### Track 9 — Meta-process

5 phases A-E avec G8 preflight systematique (5/5). Commit
discipline : 3 feat + 1 docs Phase D + 1 docs Phase E.
Codex gate 4/4 phases code. Agent Teams supervisor deploye.
