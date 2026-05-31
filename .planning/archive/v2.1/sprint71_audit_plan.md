# Sprint 71 — Audit Plan (consomme par la session fraiche S72)

**Ecrit** : 2026-05-31 (Phase E Sprint 71).
**Sprint audite** : **Sprint 71** (assainissement compute + securite
Factory + reconciliation du bloc off-sprint).
**Executeur** : session fraiche S72, Phase 0 (Cas A audit gate).
**Produit attendu** : `.planning/active/sprint71_audit_findings.md`
(verdict PASS / CONDITIONAL PASS / FAIL).
**Tip audite** : commit Phase E `docs(sprint71)` (HEAD au demarrage S72).

> **Note de nommage** (lire en premier, session S72). Ce fichier
> remplace le `sprint71_audit_plan.md` **herite** migre en
> `archive/v2.1/` au Phase 0 S71 (kickoff §2.1). L'herite auditait S70
> selon la convention « executeur » ; il n'a pas servi (S71 Phase 0 a
> joue un **audit-absorb** du bloc off-sprint a la place — deviation
> PO-3 documentee). Ce nouveau plan adopte le nommage « sprint audite »
> : **il pilote l'audit de S71** et produit `sprint71_audit_findings.md`.
> Ne pas le confondre avec l'archive.

---

## Contexte d'entree pour l'audit S71 (situation NON-standard)

S71 n'a pas demarre sur un tip propre. La session S72 doit le savoir :

- **20 commits ahead origin, RIEN pousse** (pre-launch §2.3 actif). La
  reconciliation locale etait libre (pas de bump wire, canonical
  editable). La correction B-1 a change la cle de doc applicative
  (`tasks/{id}` → `task:{id}`) sans migration : legitime, aucun noeud
  tiers ne parle ce protocole.
- **Bloc off-sprint** : ~14 commits `feat/fix(factory)` + `docs(community)`
  ont lande APRES la cloture S70 (`201b24d`), hors cycle (zero preflight/
  review/Codex/body). S71 les a **absorbes** : Phase 0 = audit retroactif
  (`sprint70_audit_findings.md`), phases A-D = correction + reconciliation.
- **Deviation PO-3** : pas de rejeu de l'audit gate S70 sur tip propre.
  L'audit-absorb fusionne « audit du precedent » et « correction » parce
  que le bloc off-sprint et les fixes S71 touchent les memes fichiers
  (`operator_server.rs`, `llm_bridge.rs`, `validator.rs`).

L'audit S72 valide donc **a la fois** la reconciliation du bloc off-sprint
ET les phases A-D, sur un seul tip.

---

## Critere verdict audit S71

| Verdict | Condition |
|---------|-----------|
| **PASS** | 0 P0, 0 P1, >= 1 P2+ documente |
| **CONDITIONAL PASS** | 0 P0, 0 P1 mais conditions a surveiller S72 |
| **FAIL** | >= 1 P0 ou >= 1 P1 non resoluble dans l'audit |

Rigor signal G4 : PASS exige >= 1 P2+ documente. 0 P0/P1 **et** 0 P2+
= CONCERN (pas PASS) — signal d'un audit trop superficiel.

---

## Tracks audit S71 (ce que Phase 0 S72 doit verifier)

### Track 1 — Suites verification

Relancer la fail-fast §1 du `sprint71_verification.md` (31 rows
applicables). Attendu : **1528 Rust nextest / 0 skip**, 279 Vitest
(front non touche), 6/6 size-limit. Verifier la coherence du delta
annonce (+39 A-D ; exit 1528 = +42 vs 1486, ecart +3 explique §2 par
le bloc off-sprint + arrondi Phase A). **Important** : les E2E
worker-pump (`dispatched_task_is_claimed_...` + miroir S4) hang sur
Windows natif (P2-A-1) — auditer via **CI Linux / Docker**, jamais
poste Windows seul (`feedback_wsl_before_push`).

### Track 2 — Security review (le plus dense de S71)

Le coeur securite est le bloc Factory (Phase C) + retro (off-sprint) :
- **G2/P0 SSE gate** : verifier que `handle_chat_stream`
  (`operator_server.rs`) applique bien `SENSITIVE_ACTIONS` AVANT
  `spawn_claude_stream`, identique a `handle_chat_message`. Verifier
  qu'aucun autre chemin ne spawn `bypassPermissions` non garde.
- **G7/P1 auth** : `X-SBFB-Token` (compare `constant_time_eq`) + Host
  guard loopback + CORS pinne (0 `allow_origin(Any)`). Verifier qu'un
  POST cross-origin sur `/api/chat/.../stream` ou `/api/actions/run`
  est refuse. Pattern de reference P27 (daemon).
- **G9/P1 modele** : 0 hit `"sonnet"` ; defaut `claude-opus-4-8[1m]`.
- **G12/P1 spawn** : timeout effectif (process tue) + diagnostic
  `claude` introuvable.
- **Phase D securite** : verifier les fixes in-phase **git option
  injection (P1)** (`--rev` / git arg injection durcie) et
  **drive-prefix traversal (P2)** dans les endpoints sprint-history/diff.
  Verifier les 3 tests securite injection/traversal des endpoints.
- **Threat boundary D5 ⚠️** : confirmer que « process local hostile lit
  le token » reste hors-scope assume (sandbox OS niveau noeud), aligne
  sur le modele daemon loopback.

### Track 3 — Patterns review

Verifier la coherence post-S71 de :
- `docs/rust/PATTERNS.md §P53` (quorum deterministe B-2 + axes
  provider/backend D8 + dead-module cleanup + deps G13) et **§P54**
  (B-1 dispatch key + B-3 E2E + caveat Windows-pump P2-A-1).
- `docs/shell/PATTERNS.md P35` (Factory Operator loopback hardening,
  cross-ref P27). Verifier l'absence de duplication (sensibilite P2-C-1).

### Track 4 — Scope cuts compliance

16/16 scope cuts (plan §12) auto-reportes dans `verification.md §3`.
Verifier par grep exhaustif qu'aucune ligne S71 ne touche : ProviderRouter
(#1), routage reseau (#2-6), fork/projet/templates (#7-9), GPU/
cross-machine (#10-11), sharding (#12), logprobs (#13), kudos (#14),
tree-sitter (#15), packaging (#16). Les refs S75/cross-machine dans le
code (build_executor dormant, ROADMAP, PATTERNS) sont des deferrals,
**pas** des implementations — confirmer (deja confirme Codex Phase B).

### Track 5 — Tests delta coherence

Verifier les deltas par phase : A +1, B +8 net (+11 −3 dead), C +14,
D +16. Total A-D = +39. Verifier la decomposition annoncee dans chaque
body vs `nextest list`. Verifier que les −3 (module `redundancy`
supprime) ne laissent aucun appelant.

### Track 6 — Review files quality

4 preflight (A-D) : A EXECUTE, B PLAN-ADAPT, C SCOPE-CUT-CONSISTENT,
D PLAN-ADAPT. 4 reviews (A-D) toutes **PASS** (promues apres Codex).
4 codex_review (A-D) bruts. + retro-review off-sprint **RECONCILED**
+ retro-Codex brut. Verifier que chaque review final est `## Verdict: PASS`
(format exact, pas `## Verdict : PASS`) et que les PARTIEL/GAP Codex sont
reconcilies. **Note meta** : le body Phase D recap C comme « EXECUTE »
alors que le preflight C reel = SCOPE-CUT-CONSISTENT (P3 cosmetique,
verdicts reels dans `verification.md §4`).

### Track 7 — Carry-overs

- CLOSED S71 : B-1, B-2, B-3, G1, G2, G5, G6, G7, G9, G12, G13, D8
  (12 gaps fermes). Verifier chaque cloture (test + code).
- Nouveaux S72 : P2-A-1 (worker-pump Windows), P2-A-2 (E2E sans
  signature), P3-A-3, P3-B-1, P3-B-2, + 3×P2/3×P3 Phase C + 3×P2/1×P3
  Phase D. Verifier qu'ils sont documentes, pas oublies.
- Reconduits : P2-A-1(rand), P2-AUDIT-2(iroh), T-NN+2(iframe wasm),
  P2-F-3(prompt coupling 2/3), LT-2(Radicle), LT-5, LT-7. Verifier
  qu'aucun n'atteint 3 reports sans exemption (sinon escalade G7).

### Track 8 — HARDENING review

Le bloc Factory ajoute une **nouvelle surface reseau locale** (serveur
Operator qui ecrit/spawn). Verifier que `THREAT_MODEL.md` /
`HARDENING_ROADMAP.md` couvrent cette surface (token+Host+CORS, gate SSE,
spawn timeout) ou ouvrir un finding si la doc menace est en retard sur le
code. C'est la principale extension de surface d'attaque de S71.

### Track 9 — Meta-process

- Phase 0 audit-absorb = deviation PO-3 documentee (kickoff §3). Verifier
  que `sprint70_audit_findings.md` couvre S70 + le bloc off-sprint avec
  verdict CONDITIONAL (P0/P1 reconcilies in-sprint).
- 4/4 phases code G8 (0 DESIGN-CONFLICT). 2 PLAN-ADAPT (B, D) non
  consecutifs — verifier qu'ils portent une evidence OSS/structurelle
  concrete (pas de derive du plan).
- Commit discipline : 4 fix phases + chores. Bodies 9 sections phases
  code. Codex gate 4/4 + 1 retro-Codex. Stash WIP terminal resolu.
- Arbitrage §11 : mono-sprint acte (Phase D a tenu). Verifier que la
  reconciliation est **complete**, pas partielle (sinon le carry vers
  S72 doit etre explicite).

---

## S72 Objective — ProviderRouter + Factory hardening (contexte, hors audit)

Apres l'audit S71, S72 ouvre (roadmap v5 §3, Arc 3.5) le **routage
provider multi-LLM** : trait `ProviderRouter`
(ClaudeProvider / OllamaProvider / NetworkProvider), cablage du chat
Factory sur le routage de taches, et durcissement Factory. S71 a livre
le **socle compute assaini** (B-1 routage reel, B-2 quorum deterministe)
sur lequel S72 construit — sans B-1 corrige, le routage provider n'aurait
pas de dispatch fonctionnel. Le defaut modele `claude-opus-4-8[1m]` (D4)
est le point d'ancrage du futur router (pas un router en soi).

---

## Non-Goals (Phase 0 S72)

La Phase 0 S72 **audite**, elle ne re-implemente pas. Elle ne doit pas :
- Re-corriger un P2/P3 deja documente (le router des P2+ vers S72+ phases).
- Commencer ProviderRouter avant le verdict d'audit (sequence stricte
  audit → kickoff → phases).
- Toucher au canonical / wire sans S4 full scan (pre-launch toujours actif
  tant que rien n'est pousse).

---

## Exit Gate (audit S71)

L'audit S71 est complet quand `sprint71_audit_findings.md` :
1. Porte un verdict PASS / CONDITIONAL PASS / FAIL avec >= 1 P2+ (G4).
2. Couvre les 9 tracks ci-dessus (suites, securite, patterns, scope,
   delta, reviews, carries, HARDENING, meta).
3. Ingere le diff complet S71 (Phases 0 + A-D) ET re-confirme la
   reconciliation du bloc off-sprint.
4. Liste les findings P0/P1 (s'il y en a) avec commit `fix(sprint71)`
   prealable au kickoff S72.

**Critere SMART** : la fail-fast §1 du `verification.md` rejoue verte en
CI Linux (1528 Rust / 0 skip) + 0 P0/P1 non resolu = S72 kickoff debloque.
