# Sprint 56 — Audit plan (post-S55)

**Ecrit** : 2026-05-08, Sprint 55 Phase E wrap-up.
**Tip d'entree audit** : tip du commit Phase E wrap-up (post-merge).
**Auditeur** : session fraiche, cas A.

---

## Tracks

### Track A — LT-7 build executor correctness

Verifier que le build executor (`build_executor.rs`) :
1. Clone un repo Git reel (pas juste un test fixture local)
2. Checkout le commit exact specifie dans metadata.build.commit
3. Produit un SHA256 deterministe avec SOURCE_DATE_EPOCH
4. Refuse un build si metadata.build.repo manquant

Grep : `build_executor`, `BuildSubmission`, `task_type.*build`

### Track B — Quorum validator integrity

Verifier que le quorum validator :
1. Accumule exactement `redundancy_factor` resultats avant decision
2. Ne peut pas etre contourne (inference tasks bypass correct)
3. Table task_results en DB survit aux restarts (migration idempotente)
4. Outlier detection log le worker divergent

Grep : `AwaitingQuorum`, `task_results`, `redundancy_factor`, `quorum`

### Track C — 3/3 MANDATORY compliance

2 items passent 3/3 MANDATORY S56. Verifier que :
1. P2-S53-outbox non-persistant : l'outbox gossip Vec en memoire est
   documente comme MVP, le design persistant fichier est scope S56
2. P2-S53-browse_request rate-limit : la browse_request n'a pas de
   rate-limit per-peer, le design est scope S56

### Track D — P2 batch Phase D quality

Verifier les 4 items mecaniques Phase D :
1. Jitter : `gen_range(30..=60)` present dans runtime.rs, le timer
   fixe 45s est bien remplace
2. SAFETY : grep `// SAFETY:` sur tous les `unsafe` des 3 fichiers
   (launcher, test-harness, named_pipe_server)
3. Constante : `DEFAULT_PROJECT_NAME` utilise aux 2 sites d'appel
   invite_api.rs, plus de "sbfb" hardcode
4. Rename : `INVITE_FORMAT_VERSION: u16 = 2` present, plus de
   `INVITE_VERSION` dans le code source (planning seul)

### Track E — CI pipeline health

Verifier que :
1. Woodpecker CI ci.sbfb.world est accessible (curl healthz)
2. Le pipeline .woodpecker/ci-linux.yml est a jour avec les
   changements S55 (images pin, steps)
3. GHA rust-ci.yml ne reference plus nexus-core-py

### Track F — Sprint process compliance

1. Phase review files present : 5/5 (A, A.1, B, C, D)
2. Phase preflight files present : 5/5 (A, A.1, B, C, D)
3. Design review present : 1/1
4. Delta tests cumule : 1207 → 1216 (+9) documente dans commits
5. Scope cuts : 15/15 respectes dans chaque commit body
6. Commit discipline : chore(planning) avant feat phase, body riche

### Track G — Carries counter accuracy

Verifier les compteurs dans verification.md §4 :
1. 2 items a 3/3 MANDATORY (outbox, browse_request) — corrects ?
2. 5 items a 2/3 (forbid-deny-doc, lightcheck, windows-test,
   E2E multi-noeuds, rustfmt drift) — compteurs incrementes ?
3. 4 nouveaux P2 S55 (build-timeout, remap-path, jitter-scope,
   invite-u16-wire) — documentes dans reviews ?
4. 7 items CLOSED S55 — coherents avec Phase A + Phase D commits ?

---

## Severity calibration

- P0 : regression fonctionnelle (quorum casse, build executor fail)
- P1 : scope cut viole, test skip cache, commit discipline break
- P2 : code quality, nit cosmetic, doc outdated
- P3 : suggestion style, minor improvement
