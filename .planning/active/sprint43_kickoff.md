# Sprint 43 — Kickoff (MANDATORY batch + Tier 5 routes API suite)

**Ecrit** : 2026-04-30 (post-audit gate S42 PASS `e1f7f00`).
**Type** : **sprint impair** — pas de phase dette obligatoire
(§6.2.1 Regle 1). Mais 7 items MANDATORY/OVERDUE doivent etre
resolus (§6.2.1 Regle 2).
**Tip master d'entree** : `e1f7f00`.
**Phase 0 audit Sprint 42** : **DEJA JOUE** — `e1f7f00` PASS.

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-04-30 (S42 meme jour).
  0 trigger actif. Pas de pre-research.

- **Technologies S43** : axum (deja dep daemon), serde_json (deja
  dep), rusqlite (deja dep). Pas de nouvelle dep externe.

- **Roadmap** : `.planning/roadmap_v1_migration_rust.md` §S43.

- **ROADMAP_COMMITMENTS** : aucun declenche (tous requierent tag
  v1.0, pas encore pose).

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 42 CLOSED + audit PASS. Tier 5 debut : deploy.rs (679 LOC)
et apps.rs (275 LOC) livres. 4 P2 dette resolus. Prochaine etape
per roadmap : porter les 4 routes API restantes (files, consent,
canary, contributor) + resoudre les 7 items MANDATORY/OVERDUE.

### §1.2 Compteurs tests entree (tip `e1f7f00`)

| Suite | Count |
|---|---|
| Rust nextest | 1089 |
| **Total** | **~2092** |

---

## §2 Goal en une phrase

Le sprint **resout les 7 items MANDATORY/OVERDUE** (conn() pub,
persist error, Mutex consolidation, rerun hash, MintRequest,
LOC kickoff, URL single-quote) et **complete le Tier 5** en
portant les 4 routes API Python restantes (files 323 LOC, consent
255, canary 212, contributor 141) vers des handlers axum Rust.
**Critere SMART : 28+ rows fail-fast verts au verification.md.**

---

## §3 Phase 0 — Audit gate Sprint 42

**DONE** — `e1f7f00`. Verdict PASS.

---

## §4 Decisions Day 0 (D1..D3 gelees)

### D1 — MANDATORY batch : 7 items resolus Phase A

**Retenu** : resoudre les 7 items MANDATORY/OVERDUE dans Phase A :

(a) **P2-REVIEW-A-1-S41 conn() pub** (3/3 MANDATORY) : `db.rs:306`
    `pub fn conn()` → `pub(crate)`. 0 appelant hors-crate. ~1 LOC.
(b) **P3-REVIEW-B-2-S39 persist error** (4/3 OVERDUE) :
    `canary_registry.rs:158,168` `let _ = self.persist()` → logger
    erreur via `tracing::warn!`. ~4 LOC.
(c) **P3-AUDIT-A-1-S39 URL single-quote** (4/3 OVERDUE) : grep
    confirme 0 instance restante. Fermeture documentee.
(d) **P3-REVIEW-B-1-S40 Manager Mutex** (4/3 OVERDUE) :
    `canary_input.rs:366-376` consolider 3 champs mtime/check en
    `Mutex<ReloadState>`. ~30-50 LOC.
(e) **P3-REVIEW-C-1-S40 rerun hash** (4/3 OVERDUE) :
    `rerun.rs:76-82` `DefaultHasher` → BLAKE3 deterministe (deja
    dep workspace). ~10 LOC.
(f) **P3-REVIEW-B-1-S41 MintRequest** (3/3 MANDATORY) :
    `invite.rs:27-36` ajouter `MintRequest::new()` constructor
    ergonomique. ~20-30 LOC.
(g) **P3-REVIEW-A-2-S39 LOC kickoff** (4/3 OVERDUE) : process
    fix. 0 LOC code. Convention deja documentee §6.7 — verification
    que les plans S43 ne contiennent pas d'estimation LOC prospective.

**Rejete** :
- Reporter encore : impossible, §6.2.1 Regle 2 s'applique (3+
  reports). Pas d'exemption applicable.

### D2 — Tier 5 routes API : files + consent + canary + contributor

**Retenu** : porter les 4 routes API Python restantes vers des
handlers axum natifs dans `crates/nexus-shell-daemon/src/`.

(a) **api/files.py** (323 LOC) : file listing, upload, download
    par hash. Port vers handler(s) axum.
(b) **api/consent.py** (255 LOC) : GPU consent levels get/set.
    Port vers handler(s) axum.
(c) **api/canary.py** (212 LOC) : canary status, observed, health.
    Port vers handler(s) axum.
(d) **api/contributor.py** (141 LOC) : contributor registration,
    attestation. Port vers handler(s) axum.

**Rejete** :
- Porter toutes les routes restantes (~700 LOC supplementaires) :
  trop pour un sprint. S44 per roadmap.
- Nouveau framework web : axum deja en place.
- Python suppression prematuree : S45 per roadmap (depend S44 routes
  completes).

### D3 — Scope cuts S43

1. **Routes restantes (health, shell, tasks, kudos, etc.)** — S44
2. **Suppression coordinator Python** — S45
3. **CI/VPS/v1.0** — S46-48
4. **Kudos debit/stake** — interdit (Day 0 #7)
5. **@require_capability middleware enforcement** — S44 (quand
   toutes les routes sont portees)
6. **Background loops wire-up** — S44+

---

**Acknowledged review findings (G1)** :

*Sprint impair continuation Tier 5 — pattern etabli S42. Pas de
decision architecturale nouvelle. MANDATORY batch = items documentes
< 100 LOC chacun. Routes API = port direct pattern S42.*

---

## §5 Plan Phase outline A..D

### Phase A — MANDATORY batch (7 items)

**But** : resoudre les 7 items MANDATORY/OVERDUE.
- conn() pub → pub(crate) (db.rs)
- persist error → tracing::warn! (canary_registry.rs)
- URL single-quote → verification 0 instance, fermeture
- Manager Mutex → consolidation ReloadState (canary_input.rs)
- rerun hash → BLAKE3 deterministe (rerun.rs)
- MintRequest → constructor new() (invite.rs)
- LOC kickoff → process verification (0 LOC code)
- Commit : `feat(sprint43): Sprint 43 Phase A — MANDATORY batch
  7 items conn+persist+mutex+hash+mint+process`

### Phase B — Routes files + consent (578 LOC)

**But** : porter files.py + consent.py vers axum.
- files handler(s) : listing, upload, download par hash
- consent handler(s) : get/set GPU consent levels
- Tests integration HTTP
- Commit : `feat(sprint43): Sprint 43 Phase B — files + consent
  API Rust`

### Phase C — Routes canary + contributor (353 LOC)

**But** : porter canary.py + contributor.py vers axum.
- canary handler(s) : status, observed, network-health
- contributor handler(s) : register, attestation
- Tests integration HTTP
- Commit : `feat(sprint43): Sprint 43 Phase C — canary +
  contributor API Rust`

### Phase D — Wrap-up

---

## §6 Items carry/dette

### Resolus S43 (plan)

- [plan] P2-REVIEW-A-1-S41 conn() pub : Phase A
- [plan] P3-REVIEW-A-2-S39 LOC kickoff : Phase A
- [plan] P3-REVIEW-B-2-S39 persist error : Phase A
- [plan] P3-AUDIT-A-1-S39 URL single-quote : Phase A
- [plan] P3-REVIEW-B-1-S40 Manager Mutex : Phase A
- [plan] P3-REVIEW-C-1-S40 rerun hash : Phase A
- [plan] P3-REVIEW-B-1-S41 MintRequest : Phase A

### Carries confirmes S44

- [carry] P2-A-1 rand blocker upstream 8+/3 : exemption blocker
  externe (rand 0.9 pre-release dual DefaultHasher, re-evaluer)
- [carry] P2-AUDIT-2 transitives iroh : herite pin 0.98
- [carry] P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 5/3 : exemption
  dependance sequentielle S45 (Python wire parite)
- [carry] P2-REVIEW-A-1-S42 ChainResult mutations target 2/3
- [carry] P2-REVIEW-B-1-S42 pow_keypair identity doc 2/3
- [carry] P3-REVIEW-A-2-S42 babel-scraper untracked 2/3
- [carry] P3-REVIEW-C-1-S42 list_apps aggregate probe 2/3
- [carry] P2-AUDIT-E-1-S42 5 P3 OVERDUE batch → RESOLU Phase A
- [carry] P3-AUDIT-A-1-S42 couverture RNG rate>1 2/3
- [carry] P3-AUDIT-C-1-S42 Debug vs serde 2/3
- [carry] P3-AUDIT-C-2-S42 pagination limit/offset 2/3

---

## §7 Scope cuts

1. Routes restantes (health, shell, tasks, kudos, etc.) — S44
2. Suppression Python — S45
3. CI/VPS/v1.0 — S46-48
4. Kudos debit/stake — interdit
5. @require_capability middleware — S44
6. Background loops wire-up — S44+

---

## §8 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | 7 MANDATORY items touchent 5 fichiers differents | Low | Tous < 50 LOC, pattern connu, 0 changement API externe |
| R2 | canary.py 212 LOC depend de DKG/ceremony existant Rust | Medium | Modules deja portes S30, handler thin wrapper |
| R3 | files.py gere l'upload/download binaire | Medium | Pattern blob-serve existant, validation zip deja en place |

---

## §9 Checkpoint de validation

1. **D1** : 7 items MANDATORY faisables Phase A ? → oui, ~65-95
   LOC total, tous localises
2. **D2** : 4 routes en 2 phases ? → oui, ~931 LOC total, pattern
   S42 etabli (deploy 679 + apps 275 livres en 2 phases)
3. **D3** : scope cuts coherents roadmap ? → oui, S44 = routes
   restantes per roadmap
