# Sprint 42 — Kickoff (dette pair + Tier 5 routes API debut)

**Ecrit** : 2026-04-29 (post-audit gate S41 PASS `7edf04b`).
**Type** : **sprint pair** — phase dette obligatoire (§6.2.1 Regle 1).
**Tip master d'entree** : `7edf04b`.
**Phase 0 audit Sprint 41** : **DEJA JOUE** — `7edf04b` PASS.

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-04-29 (S41 meme jour).
  0 trigger actif. Pas de pre-research.

- **Technologies S42** : axum (deja dep daemon), serde_json (deja
  dep), rusqlite (deja dep). Pas de nouvelle dep externe.

- **Roadmap** : `.planning/roadmap_v1_migration_rust.md` §S42-S44.

- **ROADMAP_COMMITMENTS** : aucun declenche.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 41 CLOSED + audit PASS. Tier 4 complet (7/7 modules),
jalon "Python supprimable" atteint. Prochaine etape : Tier 5
routes API migration + dette pair obligatoire.

### §1.2 Compteurs tests entree (tip `7edf04b`)

| Suite | Count |
|---|---|
| Rust nextest | 1059 |
| **Total** | **~2062** |

---

## §2 Goal en une phrase

Le sprint **resout la dette pair** (4 items P2 a 2/3 : rand_range,
pseudo_random, Tripwire vs Mutation, warn threshold) et **debute
le Tier 5** en portant `api/deploy.py` (505 LOC) et `api/apps.py`
(350 LOC) vers des handlers axum Rust natifs dans le daemon HTTP.
**Critere SMART : 28+ rows fail-fast verts au verification.md.**

---

## §3 Phase 0 — Audit gate Sprint 41

**DONE** — `7edf04b`. Verdict PASS.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Dette pair : 4 items P2 resolus

**Retenu** : resoudre les 4 items P2 a 2/3 dans Phase A dette :

(a) **P2-REVIEW-B-1-S40 rand_range + P2-REVIEW-C-1-S41
    pseudo_random** : remplacer les 2 fonctions DefaultHasher+nanos
    par `rand::thread_rng().gen_range()` (canary_input.rs) et
    `rand::thread_rng().gen::<f64>()` (upload_queue.rs). `rand` est
    deja dep workspace.
(b) **P2-REVIEW-A-1-S39 Tripwire vs Mutation** : ajouter variant
    `Mutation { reason: String, replacement: String }` a
    `GuardrailOutcome` enum. Aucun guardrail ne l'utilise encore
    (scope cut S42) mais le trait supporte la mutation pour
    canary_input post-v1.0.
(c) **P2-REVIEW-B-1-S39 warn threshold** : documenter dans
    PATTERNS.md la decision WARN_THRESHOLD_DAYS=30 / ALARM=45 vs
    RFC 9591 cadence. Le choix Python est conserve pre-v1.0.

**Rejete** :
- Reporter a S43 (3/3 MANDATORY, pas de choix)
- SHA-256 vs BLAKE3 : exemption renouvelee — depend S45 suppression
  Python (dependance sequentielle interne). Wire format parite
  requise tant que les 2 coordinators coexistent.

### D2 — Tier 5 routes API : deploy + apps (855 LOC)

**Retenu** : porter les 2 plus grosses routes API Python vers des
handlers axum natifs dans `crates/nexus-shell-daemon/src/http.rs`.

(a) **api/deploy.py** (505 LOC) : verified deploy from source —
    clone repo, verify Keyoxide Ed25519, build zip, sign provenance
    SLSA L1. Port vers handler axum `POST /api/v1/deploy`.
(b) **api/apps.py** (350 LOC) : app listing, detail, search. Port
    vers handlers axum `GET /api/v1/apps`, `GET /api/v1/apps/:id`.

**Rejete** :
- Porter toutes les routes S42 (7+ fichiers, 2000+ LOC) : trop
  pour un sprint. S42 prend les 2 plus grosses, S43-S44 le reste.
- Nouveau framework web (warp, poem) : axum deja en place, pas
  de raison de changer.

### D3 — Scope cuts S42

1. **Routes files/consent/canary/contributor** — S43
2. **Routes restantes (health, shell, tasks, kudos, etc.)** — S44
3. **Suppression coordinator Python** — S45
4. **CI/VPS/v1.0** — S46-48
5. **Kudos debit/stake** — interdit (Day 0 #7)
6. **CanaryInput mutation guardrail usage** — post-v1.0
7. **Background loops wire-up** — S43+ (avec routes)
8. **@require_capability middleware** — S43 (avec routes canary)

---

**Acknowledged review findings (G1)** :

*Sprint pair dette — design review simplifie (routes API = port
direct pattern etabli, dette = items documentes). Pas de decision
architecturale nouvelle.*

---

## §5 Plan Phase outline A..D

### Phase A — Dette pair MANDATORY

**But** : resoudre 4 items P2 a 2/3 + P3 cosmetics.
- rand_range → rand::thread_rng (canary_input.rs + upload_queue.rs)
- Tripwire → +Mutation variant dans GuardrailOutcome
- warn threshold → doc PATTERNS.md
- P3 cosmetics si temps (URL single-quote, LOC kickoff)
- Commit : `feat(sprint42): Sprint 42 Phase A — dette pair P2 batch
  rand + Mutation + warn threshold`

### Phase B — api/deploy.py port (505 LOC)

**But** : porter le verified deploy handler vers axum.
- POST /api/v1/deploy handler
- Clone repo, Keyoxide Ed25519, zip build, provenance SLSA L1
- Tests integration HTTP
- Commit : `feat(sprint42): Sprint 42 Phase B — deploy API Rust`

### Phase C — api/apps.py port (350 LOC)

**But** : porter les app listing handlers vers axum.
- GET /api/v1/apps, GET /api/v1/apps/:id handlers
- Tests integration HTTP
- Commit : `feat(sprint42): Sprint 42 Phase C — apps API Rust`

### Phase D — Wrap-up

---

## §6 Items carry/dette

### Resolus S42 (plan)

- [plan] P2-REVIEW-B-1-S40 rand_range : Phase A
- [plan] P2-REVIEW-C-1-S41 pseudo_random : Phase A
- [plan] P2-REVIEW-A-1-S39 Tripwire vs Mutation : Phase A
- [plan] P2-REVIEW-B-1-S39 warn threshold : Phase A

### Carries confirmes S43

- [carry] P2-A-1 rand blocker upstream 6+/3 : exemption externe
- [carry] P2-AUDIT-2 transitives iroh : herite pin 0.98
- [carry] P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 3/3 : exemption
  dependance sequentielle S45 (Python wire parite)
- [carry] P2-REVIEW-A-1-S41 conn() pub 2/3
- [carry] P2-REVIEW-C-1-S41 pseudo_random 2/3 → RESOLU Phase A
- [carry] P3-REVIEW-A-2-S39 LOC kickoff 3/3 → RESOLU Phase A si temps
- [carry] P3-REVIEW-B-2-S39 persist error 3/3 → evaluer Phase A
- [carry] P3-AUDIT-A-1-S39 URL single-quote 3/3 → RESOLU Phase A si temps
- [carry] P3-REVIEW-B-1-S40 Manager Mutex 3/3
- [carry] P3-REVIEW-C-1-S40 rerun hash 3/3
- [carry] P3-REVIEW-B-1-S41 MintRequest 2/3

---

## §7 Scope cuts

1. Routes files/consent/canary/contributor — S43
2. Routes restantes — S44
3. Suppression Python — S45
4. CI/VPS/v1.0 — S46-48
5. Kudos debit/stake — interdit
6. CanaryInput mutation usage — post-v1.0
7. Background loops wire-up — S43+
8. @require_capability middleware — S43

---

## §8 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | deploy.py 505 LOC = plus gros handler API, complexe (clone+crypto+zip) | High | Verified deploy deja teste E2E en Python. Port 1:1, memes checks. |
| R2 | Mutation variant ajoute mais jamais utilise = dead code | Low | Le variant est un enum arm, pas du code executif. Utilise post-v1.0. |
| R3 | SHA-256 3/3 non resolu, exemption | Low | Dependance S45 documentee, wire parite requise tant que Python existe. |

---

## §9 Checkpoint de validation

1. **D1** : 4 items dette en Phase A, faisable ? → oui, tous < 100 LOC
2. **D2** : 2 routes API en 2 phases ? → oui, deploy est gros mais
   bien structure, apps est simple
3. **D3** : SHA-256 exemption 3/3 ? → oui, dependance sequentielle S45
